use crate::{
    graphs::{EdgeDescriptor, EdgeView, Graph, VertexId, VertexView},
    routing::{
        Weight, Weighted,
        ch::{Rank, ShortcutBreakdown},
        dijkstra::{
            self, BidirectionalDrivenDijkstra, Controller, Search,
            drivers::{Driver, LimitedDistanceDriver, PathTracker},
            heuristics::ZeroHeuristic,
            policies::{AlwaysForward, NeverEarly},
        },
        presentation::{GraphEvent, consider_progress_event},
    },
    utils::{
        algo::{self, EventClient, InteractiveAlgo, NullClient},
        pq::{self, NullTracker, Pq},
        staged::{Stageable, Staged},
    },
};

pub type Priority = i64;

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

pub struct Config {
    pub allowed_lazy_updates_to_contractions_ratio: f64,
    pub allowed_time_between_global_updates: usize,
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

    queue: Queue,

    num_original_edges: usize,
    breakdowns: Vec<ShortcutBreakdown>,

    vertex_states: Stageable<Vs<G::E>>,
    since_global_update: Stats,

    total_num_contractions: usize,
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

        let mut algo = Self {
            graph,
            config,

            ranks: vec![Rank::MAX; num_vertices],

            num_original_edges: num_edges,
            breakdowns: vec![],

            vertex_states: Stageable::new(num_vertices),

            queue: Pq::with_index_tracker(vec![None; num_vertices]),
            since_global_update: Stats::default(),

            total_num_contractions: 0,
        };
        algo.perform_global_update();

        algo
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
        let Some((id, _)) = self.queue.pop() else {
            return false;
        };

        let (priority_to_update, shortcuts) = self.consider_lazy_update(id);
        if let Some(current_priority) = priority_to_update {
            self.lazy_update_priority(id, current_priority);
        } else {
            self.contract(id, shortcuts);
        }

        self.since_global_update.time += 1;

        consider_progress_event(
            self.total_num_contractions,
            self.graph.num_vertices(),
            client,
        );

        true
    }

    fn result(self) -> Self::Result {
        assert!(!self.ranks.contains(&Rank::MAX));
        (
            self.graph,
            self.ranks,
            self.num_original_edges,
            self.breakdowns,
        )
    }

    fn result_dyn(self: Box<Self>) -> Self::Result {
        assert!(!self.ranks.contains(&Rank::MAX));
        (
            self.graph,
            self.ranks,
            self.num_original_edges,
            self.breakdowns,
        )
    }
}

#[derive(Default)]
struct Stats {
    num_lazy_updates: usize,
    num_contractions: usize,
    time: usize,
}

impl Stats {
    fn lazy_updates_to_contractions_ratio(&self) -> f64 {
        (self.num_lazy_updates as f64) / ((self.num_contractions + 1) as f64)
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}

type Queue = Pq<VertexId, Priority, pq::Min, Vec<Option<usize>>>;

impl<G> Contraction<G>
where
    G: Graph,
    G::E: Weighted,
{
    fn contract(&mut self, id: VertexId, shortcuts: Option<Vec<Shortcut<G>>>) {
        self.save_rank(id);

        let shortcuts = shortcuts.unwrap_or_else(|| {
            Self::calculate_shortcuts(id, &self.graph, &self.ranks, &mut self.vertex_states).0
        });
        self.insert_shortcuts(&shortcuts);

        self.update_priorities_post_contraction(id);
    }

    fn save_rank(&mut self, id: VertexId) {
        self.ranks[id] = self.total_num_contractions;
    }

    fn insert_shortcuts(&mut self, shortcuts: &[Shortcut<G>]) {
        self.total_num_contractions += 1;
        self.since_global_update.num_contractions += 1;

        for shortcut in shortcuts {
            let edge = shortcut.weight.into();
            self.graph
                .add_edge(shortcut.start_id, shortcut.end_id, edge);
            self.breakdowns.push(shortcut.breakdown);
        }
    }

    fn calculate_shortcuts(
        id: VertexId,
        graph: &G,
        ranks: &[Rank],
        vertex_states: &mut Stageable<Vs<G::E>>,
    ) -> (Vec<Shortcut<G>>, usize) {
        let outgoing_edges: Vec<_> =
            to_uncontracted(graph.iter_outgoing_edges(id), ranks).collect();

        let incoming_edges = from_uncontracted(graph.iter_incoming_edges(id), ranks);

        if outgoing_edges.is_empty() {
            return (vec![], incoming_edges.count());
        }

        let max_outgoing_weight = outgoing_edges
            .iter()
            .map(|edge| edge.weight())
            .max()
            .unwrap();

        let mut shortcuts = vec![];

        let mut num_incoming_edges = 0;

        for incoming_edge in incoming_edges {
            num_incoming_edges += 1;

            let incoming_weight = incoming_edge.weight();

            let distance_bound = incoming_weight + max_outgoing_weight;

            let mut local_vertex_states = vertex_states.stage();
            let driver =
                LocalDijkstraDriver::new(id, &outgoing_edges, ranks, &mut local_vertex_states);

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
                graph,
                forward,
                backward,
                direction_policy,
                termination_policy,
                &mut client,
            );
            algo::complete(&mut dijkstra, &mut client);

            for outgoing_edge in &outgoing_edges {
                if incoming_edge.start.id == outgoing_edge.end.id {
                    continue;
                }
                let distance_through_contracted = incoming_weight + outgoing_edge.weight();
                let found_distance = local_vertex_states.get(outgoing_edge.end.id).distance;
                if found_distance > distance_through_contracted {
                    let breakdown = [incoming_edge.descriptor(), outgoing_edge.descriptor()];
                    shortcuts.push(Shortcut {
                        start_id: incoming_edge.start.id,
                        end_id: outgoing_edge.end.id,
                        weight: distance_through_contracted,
                        breakdown,
                    });
                }
            }
        }

        (shortcuts, num_incoming_edges + outgoing_edges.len())
    }

    fn update_priorities_post_contraction(&mut self, id: VertexId) {
        if self.is_global_update_needed() {
            self.perform_global_update();
        } else {
            self.update_neighbor_priorities(id);
        }
    }

    fn consider_lazy_update(
        &mut self,
        id: VertexId,
    ) -> (Option<Priority>, Option<Vec<Shortcut<G>>>) {
        let (current_priority, shortcuts) =
            Self::calculate_priority(id, &self.graph, &self.ranks, &mut self.vertex_states);

        let priority_to_update = self
            .queue
            .peek()
            .and_then(|(_, &priority)| (current_priority > priority).then_some(current_priority));

        (priority_to_update, shortcuts)
    }

    fn lazy_update_priority(&mut self, id: VertexId, priority: Priority) {
        self.queue.push_or_update(id, priority);
        self.since_global_update.num_lazy_updates += 1;
    }

    fn is_global_update_needed(&self) -> bool {
        let time_trigger =
            self.since_global_update.time >= self.config.allowed_time_between_global_updates;
        let ratio_trigger = self
            .since_global_update
            .lazy_updates_to_contractions_ratio()
            >= self.config.allowed_lazy_updates_to_contractions_ratio;

        time_trigger || ratio_trigger
    }

    fn perform_global_update(&mut self) {
        Self::refresh_priorities(
            self.graph.iter_vertices(),
            &self.graph,
            &self.ranks,
            &mut self.vertex_states,
            &mut self.queue,
        );

        self.since_global_update.reset();
    }

    fn update_neighbor_priorities(&mut self, id: VertexId) {
        Self::refresh_priorities(
            self.graph.iter_outgoing_edges(id).map(|edge| edge.end),
            &self.graph,
            &self.ranks,
            &mut self.vertex_states,
            &mut self.queue,
        );
        Self::refresh_priorities(
            self.graph.iter_incoming_edges(id).map(|edge| edge.start),
            &self.graph,
            &self.ranks,
            &mut self.vertex_states,
            &mut self.queue,
        );
    }

    fn refresh_priorities<'g>(
        vertices: impl Iterator<Item = VertexView<'g, G::V>>,
        graph: &'g G,
        ranks: &[Rank],
        vertex_states: &mut Stageable<Vs<G::E>>,
        queue: &mut Queue,
    ) {
        for vertex in vertices {
            if is_uncontracted(vertex.id, ranks) {
                let (current_priority, _) =
                    Self::calculate_priority(vertex.id, graph, ranks, vertex_states);
                queue.push_or_update(vertex.id, current_priority);
            }
        }
    }

    fn calculate_priority(
        id: VertexId,
        graph: &G,
        ranks: &[Rank],
        vertex_states: &mut Stageable<Vs<G::E>>,
    ) -> (Priority, Option<Vec<Shortcut<G>>>) {
        let (shortcuts, num_removed_edges) =
            Self::calculate_shortcuts(id, graph, ranks, vertex_states);

        let priority = (shortcuts.len() as Priority) - (num_removed_edges as Priority);

        (priority, Some(shortcuts))
    }
}

#[inline(always)]
fn to_uncontracted<'g, V, E>(
    edges: impl Iterator<Item = EdgeView<'g, V, E>> + 'g,
    ranks: &'g [Rank],
) -> impl Iterator<Item = EdgeView<'g, V, E>> + 'g
where
    V: 'g,
    E: 'g,
{
    edges.filter(|edge| is_uncontracted(edge.end.id, ranks))
}

#[inline(always)]
fn from_uncontracted<'g, V, E>(
    edges: impl Iterator<Item = EdgeView<'g, V, E>> + 'g,
    ranks: &'g [Rank],
) -> impl Iterator<Item = EdgeView<'g, V, E>> + 'g
where
    V: 'g,
    E: 'g,
{
    edges.filter(|edge| is_uncontracted(edge.start.id, ranks))
}

#[inline(always)]
fn is_uncontracted(id: VertexId, ranks: &[Rank]) -> bool {
    ranks[id] == Rank::MAX
}

#[derive(Debug, Clone, Copy)]
struct VertexState<W> {
    distance: W,
    should_be_visited: bool,
}

impl<W> Default for VertexState<W>
where
    W: Weight,
{
    fn default() -> Self {
        Self {
            distance: W::infinity(),
            should_be_visited: false,
        }
    }
}

struct LocalDijkstraDriver<'s, 'g, W> {
    center_id: VertexId,
    ranks: &'g [Rank],
    vertex_states: &'s mut Staged<'g, VertexState<W>>,
    num_to_visit: usize,
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
            num_to_visit: outgoing_edges.len(),
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
        if self.vertex_states.get(vertex.id).should_be_visited {
            self.num_visited += 1;
        }
        self.num_visited <= self.num_to_visit
    }
}
