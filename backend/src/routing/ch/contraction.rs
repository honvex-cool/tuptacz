use crate::{
    graphs::{EdgeDescriptor, EdgeId, EdgeView, Graph, VertexId, VertexView},
    routing::{
        Weight, Weighted,
        ch::{Config, Priority, PriorityParts, Rank, ShortcutBreakdown, Stats},
        dijkstra::{
            self, BidirectionalDrivenDijkstra, Controller, Search,
            drivers::{Driver, LimitedDistanceDriver, PathTracker},
            heuristics::ZeroHeuristic,
            policies::{AlwaysForward, NeverEarly},
        },
        presentation::{GraphAction, GraphEvent, consider_progress_event},
    },
    utils::{
        algo::{self, EventClient, InteractiveAlgo, NullClient},
        pq::{self, NullTracker, Pq},
        staged::{Stageable, Staged},
    },
};

struct Shortcut<G>
where
    G: Graph,
    G::E: Weighted,
{
    start_id: VertexId,
    end_id: VertexId,
    weight: <G::E as Weighted>::Weight,
    breakdown: ShortcutBreakdown,
}

struct ShortcutSearchResult<G>
where
    G: Graph,
    G::E: Weighted,
{
    shortcuts: Vec<Shortcut<G>>,
    num_removed_edges: usize,
    sum_search_space_sizes: usize,
}

#[allow(type_alias_bounds)]
type Vs<W>
where
    W: Weighted,
= VertexState<W::Weight>;

pub struct Contraction<G>
where
    G: Graph,
    G::E: Weighted,
{
    graph: G,
    config: Config,

    ranks: Vec<Rank>,

    // For Q heuristic
    depth_estimates: Vec<usize>,

    queue: Queue,

    num_original_edges: usize,
    breakdowns: Vec<ShortcutBreakdown>,

    vertex_states: Stageable<Vs<G::E>>,

    since_global_update: Stats,
    total: Stats,

    to_update: Vec<VertexId>,
}

impl<G> Contraction<G>
where
    G: Graph,
    G::E: Weighted,
{
    #[inline(always)]
    pub fn new(graph: G, config: Config) -> Self {
        let num_vertices = graph.num_vertices();
        let num_edges = graph.num_edges();

        let mut queue = Pq::with_index_tracker(vec![None; num_vertices]);

        /* Temporarily inserting every vertex with priority 0.
         * This will be immediately corrected by the first global update which
         * updates all vertex in the queue.
         */
        (0..num_vertices).for_each(|id| queue.push_or_update(id, 0));

        let mut contraction = Self {
            graph,
            config,

            ranks: vec![Rank::MAX; num_vertices],

            depth_estimates: vec![0; num_vertices],

            num_original_edges: num_edges,
            breakdowns: vec![],

            vertex_states: Stageable::new(num_vertices),

            queue,

            since_global_update: Stats::default(),
            total: Stats::default(),

            to_update: Vec::with_capacity(num_vertices),
        };
        contraction.start_global_update();

        contraction
    }

    #[inline(always)]
    fn emit_update_in_global<C>(
        &self,
        vertex: VertexView<G::V>,
        terms: PriorityParts,
        priority: Priority,
        client: &mut C,
    ) where
        C: EventClient<GraphEvent<G::V, G::E>>,
    {
        client.consume(GraphEvent {
            action: GraphAction::UpdateInGlobal {
                vertex: vertex.detach(),
                coefficients: self.config.coefficients,
                terms,
                priority,
            },
            comment: "Priority calculated during global update".to_owned(),
        });
    }

    #[inline(always)]
    fn emit_interrupt<C>(client: &mut C)
    where
        C: EventClient<GraphEvent<G::V, G::E>>,
    {
        client.consume(GraphEvent {
            action: GraphAction::Interrupt,
            comment: "Global update finished".to_owned(),
        });
    }

    #[inline(always)]
    fn emit_lazy_update<C>(vertex: VertexView<G::V>, client: &mut C)
    where
        C: EventClient<GraphEvent<G::V, G::E>>,
    {
        client.consume(GraphEvent {
            action: GraphAction::LazyUpdate {
                vertex: vertex.detach(),
            },
            comment: "Vertex priority not up to date, performing lazy update".to_owned(),
        });
    }

    #[inline(always)]
    fn emit_contraction<C>(&self, vertex_id: VertexId, shortcuts: &[Shortcut<G>], client: &mut C)
    where
        C: EventClient<GraphEvent<G::V, G::E>>,
    {
        client.consume(GraphEvent {
            action: GraphAction::Contraction {
                vertex: self.graph.get_vertex(vertex_id).detach(),
                shortcuts: shortcuts
                    .iter()
                    .enumerate()
                    .map(|(i, shortcut)| {
                        let [first, second] = shortcut.breakdown.descriptors;
                        (
                            self.graph.num_edges() - shortcuts.len() + i,
                            self.graph.get_edge(first).detach(),
                            self.graph.get_edge(second).detach(),
                        )
                    })
                    .collect(),
            },
            comment: "Vertex contracted, shortcuts added".to_owned(),
        });
    }

    #[inline(always)]
    fn emit_contraction_summary<C>(&self, client: &mut C)
    where
        C: EventClient<GraphEvent<G::V, G::E>>,
    {
        client.consume(GraphEvent {
            action: GraphAction::ContractionSummary {
                stats: self.total.clone(),
            },
            comment: "Summary of the contraction phase".to_owned(),
        });
    }

    #[inline(always)]
    fn emit_global_update_triggered<C>(client: &mut C)
    where
        C: EventClient<GraphEvent<G::V, G::E>>,
    {
        client.consume(GraphEvent {
            action: GraphAction::GlobalUpdateTriggered,
            comment: "Global update triggered".to_owned(),
        });
    }
}

impl<G, C> InteractiveAlgo<C, GraphEvent<G::V, G::E>> for Contraction<G>
where
    G: Graph,
    G::E: Weighted,
    C: EventClient<GraphEvent<G::V, G::E>>,
{
    type Result = (G, Vec<Rank>, usize, Vec<ShortcutBreakdown>);

    #[inline(always)]
    fn step(&mut self, client: &mut C) -> bool {
        let is_global_update_at_start = !self.queue.is_empty();

        let is_running = self.step_internal(client);

        consider_progress_event(
            self.total.num_contractions,
            self.graph.num_vertices(),
            client,
        );

        let is_global_update_at_end = !self.to_update.is_empty();

        if is_running && !is_global_update_at_start && is_global_update_at_end {
            Self::emit_global_update_triggered(client);
        }

        is_running
    }

    fn result(self, client: &mut C) -> Self::Result {
        self.emit_contraction_summary(client);
        (
            self.graph,
            self.ranks,
            self.num_original_edges,
            self.breakdowns,
        )
    }

    fn result_dyn(self: Box<Self>, client: &mut C) -> Self::Result {
        self.result(client)
    }
}

type Queue = Pq<VertexId, Priority, pq::Min, Vec<Option<usize>>>;

impl<G> Contraction<G>
where
    G: Graph,
    G::E: Weighted,
{
    fn step_internal<C>(&mut self, client: &mut C) -> bool
    where
        C: EventClient<GraphEvent<G::V, G::E>>,
    {
        if let Some(id) = self.to_update.pop() {
            // Global update in progress
            let (terms, priority) = self.update_priority(id);

            self.emit_update_in_global(self.graph.get_vertex(id), terms, priority, client);

            if self.to_update.is_empty() {
                Self::emit_interrupt(client);
            }

            return true;
        }

        let Some((id, _)) = self.queue.pop() else {
            return false;
        };

        let (priority_to_update, shortcuts) = self.consider_lazy_update(id);
        if let Some(current_priority) = priority_to_update {
            self.lazy_update_priority(id, current_priority);

            Self::emit_lazy_update(self.graph.get_vertex(id), client);
        } else {
            let shortcuts = shortcuts.unwrap_or_else(|| self.calculate_shortcuts(id).shortcuts);

            self.emit_contraction(id, &shortcuts, client);

            self.contract(id, shortcuts);
        }

        self.since_global_update.num_steps += 1;
        self.total.num_steps += 1;

        true
    }

    fn contract(&mut self, id: VertexId, shortcuts: Vec<Shortcut<G>>) {
        self.save_rank(id);

        self.insert_shortcuts(shortcuts);

        self.since_global_update.num_contractions += 1;
        self.total.num_contractions += 1;

        self.update_priorities_post_contraction(id);
    }

    fn save_rank(&mut self, id: VertexId) {
        self.ranks[id] = self.total.num_contractions;
    }

    fn insert_shortcuts(&mut self, shortcuts: Vec<Shortcut<G>>) {
        let num_shortcuts = shortcuts.len();

        self.since_global_update.num_shortcuts += num_shortcuts;
        self.total.num_shortcuts += num_shortcuts;

        for shortcut in shortcuts {
            let edge = shortcut.weight.into();
            self.graph
                .add_edge(shortcut.start_id, shortcut.end_id, edge);
            self.breakdowns.push(shortcut.breakdown);
        }
    }

    fn calculate_shortcuts(&mut self, id: VertexId) -> ShortcutSearchResult<G> {
        let incoming_edges: Vec<_> =
            Self::edges_from_uncontracted_to(id, &self.graph, &self.ranks).collect();
        let outgoing_edges: Vec<_> =
            Self::edges_to_uncontracted_from(id, &self.graph, &self.ranks).collect();

        let mut sum_search_space_sizes = 0;

        if outgoing_edges.is_empty() {
            return ShortcutSearchResult {
                shortcuts: vec![],
                num_removed_edges: incoming_edges.len(),
                sum_search_space_sizes,
            };
        }

        let max_outgoing_weight = outgoing_edges
            .iter()
            .map(|edge| edge.weight())
            .max()
            .unwrap();

        let mut shortcuts = vec![];

        for incoming_edge in &incoming_edges {
            let incoming_weight = incoming_edge.weight();

            let distance_bound = incoming_weight + max_outgoing_weight;

            let mut local_vertex_states = self.vertex_states.stage();
            let driver = LocalDijkstraDriver::new(
                id,
                &outgoing_edges,
                &self.ranks,
                &mut local_vertex_states,
            );

            let driver = LimitedDistanceDriver::new(distance_bound, driver);
            let search = Search {
                id: incoming_edge.start.id,
                driver,
            };
            let forward = Controller {
                search,
                heuristic: ZeroHeuristic,
                queue: dijkstra::Queue::with_index_tracker(NullTracker),
            };
            let backward = None;

            let direction_policy = AlwaysForward;
            let termination_policy = NeverEarly;

            let mut client = NullClient;
            let mut dijkstra = BidirectionalDrivenDijkstra::new(
                &self.graph,
                forward,
                backward,
                direction_policy,
                termination_policy,
                &mut client,
            );
            algo::complete(&mut dijkstra, &mut client);
            sum_search_space_sizes += dijkstra
                .result(&mut client)
                .forward
                .driver
                .into_inner()
                .num_visited;

            for outgoing_edge in &outgoing_edges {
                if incoming_edge.start.id == outgoing_edge.end.id {
                    continue;
                }
                let distance_through_contracted = incoming_weight + outgoing_edge.weight();
                let found_distance = local_vertex_states.get(outgoing_edge.end.id).distance;
                if found_distance > distance_through_contracted {
                    let descriptors = [incoming_edge.descriptor(), outgoing_edge.descriptor()];

                    let num_real_edges_on_first_part = Self::get_num_real_edges(
                        incoming_edge.id,
                        self.num_original_edges,
                        &self.breakdowns,
                    );
                    let num_real_edges_on_second_part = Self::get_num_real_edges(
                        outgoing_edge.id,
                        self.num_original_edges,
                        &self.breakdowns,
                    );

                    let num_real_edges =
                        num_real_edges_on_first_part + num_real_edges_on_second_part;

                    let breakdown = ShortcutBreakdown {
                        descriptors,
                        num_real_edges,
                    };

                    shortcuts.push(Shortcut {
                        start_id: incoming_edge.start.id,
                        end_id: outgoing_edge.end.id,
                        weight: distance_through_contracted,
                        breakdown,
                    });
                }
            }
        }

        ShortcutSearchResult {
            shortcuts,
            num_removed_edges: incoming_edges.len() + outgoing_edges.len(),
            sum_search_space_sizes,
        }
    }

    fn update_priorities_post_contraction(&mut self, id: VertexId) {
        // For Q heuristic
        let estimate_through_self = self.depth_estimates[id] + 1;

        for neighbor in Self::uncontracted_neighbors(id, &self.graph, &self.ranks) {
            let neighbor_estimate = &mut self.depth_estimates[neighbor.id];
            *neighbor_estimate = usize::max(*neighbor_estimate, estimate_through_self);
        }

        if self.is_global_update_needed() {
            self.start_global_update();
        } else {
            self.update_neighbor_priorities(id);
        }
    }

    fn consider_lazy_update(
        &mut self,
        id: VertexId,
    ) -> (Option<Priority>, Option<Vec<Shortcut<G>>>) {
        let (_, current_priority, shortcuts) = self.calculate_priority(id);

        let priority_to_update = self
            .queue
            .peek()
            .and_then(|(_, &priority)| (current_priority > priority).then_some(current_priority));

        (priority_to_update, shortcuts)
    }

    fn lazy_update_priority(&mut self, id: VertexId, priority: Priority) {
        self.queue.push_or_update(id, priority);

        self.since_global_update.num_lazy_updates += 1;
        self.total.num_lazy_updates += 1;
    }

    fn is_global_update_needed(&self) -> bool {
        let time_trigger =
            self.since_global_update.num_steps >= self.config.allowed_time_between_global_updates;
        let ratio_trigger = self
            .since_global_update
            .lazy_updates_to_contractions_ratio()
            >= self.config.allowed_lazy_updates_to_contractions_ratio;

        time_trigger || ratio_trigger
    }

    fn start_global_update(&mut self) {
        self.since_global_update.num_lazy_updates += 1;
        self.total.num_global_updates += 1;

        self.to_update.extend(self.queue.iter_values());
        self.queue.clear();

        self.since_global_update.reset();
    }

    fn update_neighbor_priorities(&mut self, id: VertexId) {
        self.to_update.extend(
            Self::uncontracted_neighbors(id, &self.graph, &self.ranks).map(|vertex| vertex.id),
        );

        self.update_priorities_for_all_remaining();
    }

    fn update_priorities_for_all_remaining(&mut self) {
        while let Some(id) = self.to_update.pop() {
            self.update_priority(id);
        }
    }

    fn update_priority(&mut self, id: VertexId) -> (PriorityParts, Priority) {
        let (terms, current_priority, _) = self.calculate_priority(id);
        self.queue.push_or_update(id, current_priority);
        (terms, current_priority)
    }

    fn calculate_priority(
        &mut self,
        id: VertexId,
    ) -> (PriorityParts, Priority, Option<Vec<Shortcut<G>>>) {
        let coefficients = self.config.coefficients;

        let (shortcuts, term_e, term_s, term_o) =
            if coefficients.e != 0 || coefficients.s != 0 || coefficients.o != 0 {
                let result = self.calculate_shortcuts(id);

                let term_e =
                    (result.shortcuts.len() as Priority) - (result.num_removed_edges as Priority);

                let term_s = result.sum_search_space_sizes as Priority;

                let term_o: Priority = if coefficients.o != 0 {
                    result
                        .shortcuts
                        .iter()
                        .map(|shortcut| shortcut.breakdown.num_real_edges as Priority)
                        .sum()
                } else {
                    0
                };

                (Some(result.shortcuts), term_e, term_s, term_o)
            } else {
                (None, 0, 0, 0)
            };

        let term_d = if coefficients.d != 0 {
            let mut local_vertex_states = self.vertex_states.stage();
            let mut num_uncontracted_neighbors = 0;
            for neighbor in Self::uncontracted_neighbors(id, &self.graph, &self.ranks) {
                if !local_vertex_states.get(neighbor.id).is_counted {
                    num_uncontracted_neighbors += 1;
                    local_vertex_states.get_mut(neighbor.id).is_counted = true;
                }
            }
            num_uncontracted_neighbors as Priority
        } else {
            0
        };

        let term_q = self.depth_estimates[id] as Priority;

        let terms = PriorityParts {
            e: term_e,
            s: term_s,
            d: term_d,
            o: term_o,
            q: term_q,
        };

        let priority = coefficients.dot(&terms);

        (terms, priority, shortcuts)
    }

    #[inline(always)]
    fn edges_to_uncontracted_from<'g>(
        id: VertexId,
        graph: &'g G,
        ranks: &[Rank],
    ) -> impl Iterator<Item = EdgeView<'g, G::V, G::E>> {
        graph
            .iter_outgoing_edges(id)
            .filter(|edge| is_uncontracted(edge.end.id, ranks))
    }

    #[inline(always)]
    fn edges_from_uncontracted_to<'g>(
        id: VertexId,
        graph: &'g G,
        ranks: &[Rank],
    ) -> impl Iterator<Item = EdgeView<'g, G::V, G::E>> {
        graph
            .iter_incoming_edges(id)
            .filter(|edge| is_uncontracted(edge.start.id, ranks))
    }

    #[inline(always)]
    fn uncontracted_neighbors<'g>(
        id: VertexId,
        graph: &'g G,
        ranks: &[Rank],
    ) -> impl Iterator<Item = VertexView<'g, G::V>> {
        let incoming_neighbors =
            Self::edges_from_uncontracted_to(id, graph, ranks).map(|edge| edge.start);
        let outgoing_neighbors =
            Self::edges_to_uncontracted_from(id, graph, ranks).map(|edge| edge.end);
        std::iter::chain(incoming_neighbors, outgoing_neighbors)
    }

    #[inline(always)]
    fn get_num_real_edges(
        id: EdgeId,
        num_original_edges: usize,
        breakdowns: &[ShortcutBreakdown],
    ) -> usize {
        let id = id.0;
        if id < num_original_edges {
            1
        } else {
            breakdowns[id - num_original_edges].num_real_edges
        }
    }
}

#[inline(always)]
fn is_uncontracted(id: VertexId, ranks: &[Rank]) -> bool {
    ranks[id] == Rank::MAX
}

#[derive(Debug, Clone, Copy)]
struct VertexState<W> {
    distance: W,
    should_be_visited: bool,
    is_counted: bool,
}

impl<W> Default for VertexState<W>
where
    W: Weight,
{
    fn default() -> Self {
        Self {
            distance: W::infinity(),
            should_be_visited: false,
            is_counted: false,
        }
    }
}

struct LocalDijkstraDriver<'s, 'g, W> {
    center_id: VertexId,
    ranks: &'g [Rank],
    vertex_states: &'s mut Staged<'g, VertexState<W>>,
    num_targets: usize,
    num_visited_targets: usize,
    num_visited: usize,
}

impl<'s, 'g, W> LocalDijkstraDriver<'s, 'g, W>
where
    W: Weight,
{
    fn new<V, E>(
        center_id: VertexId,
        outgoing_edges: &[EdgeView<V, E>],
        ranks: &'g [Rank],
        vertex_states: &'s mut Staged<'g, VertexState<W>>,
    ) -> Self {
        for edge in outgoing_edges {
            vertex_states.get_mut(edge.end.id).should_be_visited = true;
        }

        Self {
            center_id,
            ranks,
            vertex_states,
            num_targets: outgoing_edges.len(),
            num_visited_targets: 0,
            num_visited: 0,
        }
    }
}

impl<'s, 'g, V, E, W> PathTracker<V, E> for LocalDijkstraDriver<'s, 'g, W>
where
    W: Weight,
{
    type Distance = W;

    fn get_distance(&self, vertex: VertexView<V>) -> Self::Distance {
        self.vertex_states.get(vertex.id).distance
    }

    fn set_distance(&mut self, vertex: VertexView<V>, distance: Self::Distance) {
        self.vertex_states.get_mut(vertex.id).distance = distance;
    }

    fn get_predecessor(&self, _vertex: VertexView<V>) -> Option<EdgeDescriptor> {
        None
    }
    fn set_predecessor(&mut self, _edge: EdgeView<V, E>) {}
}

impl<'s, 'g, V, E> Driver<V, E> for LocalDijkstraDriver<'s, 'g, E::Weight>
where
    E: Weighted,
{
    fn should_consider_edge(&self, edge: EdgeView<V, E>) -> bool {
        edge.start.id != self.center_id
            && edge.end.id != self.center_id
            && is_uncontracted(edge.end.id, self.ranks)
    }

    fn should_consider_vertex(&self, vertex: VertexView<V>, _total_weight: E::Weight) -> bool {
        vertex.id != self.center_id && is_uncontracted(vertex.id, self.ranks)
    }

    fn visit(&mut self, vertex: VertexView<V>) -> bool {
        self.num_visited += 1;
        if self.vertex_states.get(vertex.id).should_be_visited {
            self.num_visited_targets += 1;
        }
        self.num_visited_targets <= self.num_targets
    }
}
