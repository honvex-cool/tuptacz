use std::{cell::Cell, cmp::Ordering};

use crate::{
    graphs::{EdgeDescriptor, EdgeView, Graph, VertexId, VertexView},
    routing::{
        Weight, Weighted,
        ch::{Rank, ShortcutBreakdown},
        dijkstra::{
            self, BidirectionalDrivenDijkstra, Controller, Search,
            drivers::{Driver, LimitedDistanceDriver, PathTracker},
            policies::{AlwaysForward, NeverEarly},
        },
        presentation::GraphEvent,
    },
    utils::{
        algo::{self, EventClient, InteractiveAlgo, NullClient},
        pq::{self, NullTracker, Pq},
        staged::{Epoch, STARTING_EPOCH},
    },
};

pub type Priority = f64;

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

pub struct Contraction<G>
where
    G: Graph,
    G::E: Weighted,
{
    graph: G,
    config: Config,

    queue: Queue,

    state: State<G>,
    since_global_update: Stats,

    total_num_contractions: usize,
}

impl<G> Contraction<G>
where
    G: Graph,
    G::E: Weighted,
{
    #[inline(always)]
    fn new(graph: G, config: Config) -> Self {
        let num_vertices = graph.num_vertices();

        let mut algo = Self {
            graph,
            config,

            queue: Pq::with_index_tracker(vec![None; num_vertices]),
            state: State::with_size(num_vertices),
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
    type Result = (G, Vec<Rank>);

    #[inline(always)]
    fn step(&mut self, _client: &mut C) -> bool {
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

        true
    }

    fn result(self) -> Self::Result {
        (self.graph, self.state.ranks)
    }

    fn result_dyn(self: Box<Self>) -> Self::Result {
        (self.graph, self.state.ranks)
    }
}

#[derive(PartialEq, PartialOrd, Default)]
struct OrdPriority(Priority);

impl Eq for OrdPriority {}

impl Ord for OrdPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap()
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

type Queue = Pq<VertexId, OrdPriority, pq::Min, Vec<Option<usize>>>;

impl<G> Contraction<G>
where
    G: Graph,
    G::E: Weighted,
{
    fn contract(&mut self, id: VertexId, shortcuts: Option<Vec<Shortcut<G>>>) {
        self.save_rank(id);

        let shortcuts = shortcuts
            .unwrap_or_else(|| Self::calculate_shortcuts(id, &self.graph, &mut self.state).0);
        self.insert_shortcuts(&shortcuts);

        self.update_priorities_post_contraction(id);
    }

    fn save_rank(&mut self, id: VertexId) {
        self.state.ranks[id] = self.total_num_contractions;
    }

    fn insert_shortcuts(&mut self, shortcuts: &[Shortcut<G>]) {
        self.total_num_contractions += 1;
        self.since_global_update.num_contractions += 1;

        for shortcut in shortcuts {
            let edge = shortcut.weight.into();
            self.graph
                .add_edge(shortcut.start_id, shortcut.end_id, edge);
        }
    }

    fn calculate_shortcuts(
        id: VertexId,
        graph: &G,
        state: &mut State<G>,
    ) -> (Vec<Shortcut<G>>, usize) {
        let outgoing_edges: Vec<_> =
            to_uncontracted(graph.iter_outgoing_edges(id), &state.ranks).collect();

        let incoming_edges = from_uncontracted(graph.iter_incoming_edges(id), &state.ranks);

        if outgoing_edges.is_empty() {
            return (vec![], incoming_edges.count());
        }

        let max_outgoing_weight = outgoing_edges
            .iter()
            .map(|edge| edge.weight())
            .max_by(|lhs, rhs| lhs.partial_cmp(rhs).unwrap_or(Ordering::Equal))
            .unwrap();

        let mut shortcuts = vec![];

        let mut num_incoming_edges = 0;

        for incoming_edge in incoming_edges {
            num_incoming_edges += 1;

            let incoming_weight = incoming_edge.weight();

            let distance_bound = incoming_weight + max_outgoing_weight;

            let current_epoch = state.advance_epoch();

            let driver = LocalDijkstraDriver::new(
                current_epoch,
                &outgoing_edges,
                &mut state.distances,
                &mut state.should_be_visited,
                &state.time_stamps,
                &state.ranks,
            );

            let driver = LimitedDistanceDriver::new(distance_bound, driver);

            let search = Search {
                id: incoming_edge.start.id,
                driver,
            };
            let forward = Controller {
                search,
                heuristic: (),
                queue: dijkstra::Queue::with_index_tracker(NullTracker),
            };
            let backward = None;

            let direction_policy = AlwaysForward;
            let termination_policy = NeverEarly;

            let mut client = NullClient::default();
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
                let distance_through_contracted = incoming_weight + outgoing_edge.weight();
                if state.distances[outgoing_edge.end.id].get() > distance_through_contracted {
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
            Self::calculate_priority(id, &self.graph, &mut self.state);

        let priority_to_update = self.queue.peek().and_then(|(_, &OrdPriority(priority))| {
            (current_priority > priority).then_some(current_priority)
        });

        (priority_to_update, shortcuts)
    }

    fn lazy_update_priority(&mut self, id: VertexId, priority: Priority) {
        self.queue.push(id, OrdPriority(priority));
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
            &mut self.state,
            &mut self.queue,
        );

        self.since_global_update.reset();
    }

    fn update_neighbor_priorities(&mut self, id: VertexId) {
        Self::refresh_priorities(
            self.graph.iter_outgoing_edges(id).map(|edge| edge.end),
            &self.graph,
            &mut self.state,
            &mut self.queue,
        );
        Self::refresh_priorities(
            self.graph.iter_incoming_edges(id).map(|edge| edge.start),
            &self.graph,
            &mut self.state,
            &mut self.queue,
        );
    }

    fn refresh_priorities<'g>(
        vertices: impl Iterator<Item = VertexView<'g, G::V>>,
        graph: &'g G,
        state: &mut State<G>,
        queue: &mut Queue,
    ) {
        for vertex in vertices {
            if is_uncontracted(vertex.id, &state.ranks) {
                let (current_priority, _) = Self::calculate_priority(vertex.id, graph, state);
                queue.push_or_update(vertex.id, OrdPriority(current_priority));
            }
        }
    }

    fn calculate_priority(
        id: VertexId,
        graph: &G,
        state: &mut State<G>,
    ) -> (Priority, Option<Vec<Shortcut<G>>>) {
        let (shortcuts, num_removed_edges) = Self::calculate_shortcuts(id, graph, state);

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

struct State<G>
where
    G: Graph,
    G::E: Weighted,
{
    epoch: Cell<Epoch>,
    distances: Vec<Cell<<G::E as Weighted>::Weight>>,
    ranks: Vec<Rank>,
    time_stamps: Vec<Cell<Epoch>>,
    should_be_visited: Vec<Cell<bool>>,
}

impl<G> State<G>
where
    G: Graph,
    G::E: Weighted,
{
    fn advance_epoch(&self) -> Epoch {
        self.epoch.update(|epoch| epoch + 1);
        self.epoch.get()
    }

    fn with_size(size: usize) -> Self {
        Self {
            epoch: Cell::new(STARTING_EPOCH),
            distances: vec![Cell::new(<G::E as Weighted>::Weight::infinity()); size],
            ranks: vec![Rank::MAX; size],
            time_stamps: vec![Cell::new(STARTING_EPOCH); size],
            should_be_visited: vec![Cell::new(false); size],
        }
    }
}

struct LocalDijkstraDriver<'s, W> {
    current_epoch: Epoch,
    distances: &'s mut [Cell<W>],
    should_be_visited: &'s mut [Cell<bool>],
    time_stamps: &'s [Cell<Epoch>],
    ranks: &'s [Rank],
    num_to_visit: usize,
    num_visited: usize,
}

impl<'s, W> LocalDijkstraDriver<'s, W>
where
    W: Weight,
{
    fn new<V, E>(
        current_epoch: Epoch,
        outgoing_edges: &[EdgeView<V, E>],
        distances: &'s mut [Cell<W>],
        should_be_visited: &'s mut [Cell<bool>],
        time_stamps: &'s [Cell<Epoch>],
        ranks: &'s [Rank],
    ) -> Self {
        let driver = Self {
            current_epoch,
            distances,
            should_be_visited,
            time_stamps,
            ranks,
            num_to_visit: outgoing_edges.len(),
            num_visited: 0,
        };

        for edge in outgoing_edges {
            driver.refresh(edge.end.id);
            driver.should_be_visited[edge.end.id].set(true);
        }

        driver
    }
}

impl<'s, W> LocalDijkstraDriver<'s, W>
where
    W: Weight,
{
    fn refresh(&self, id: VertexId) {
        if self.time_stamps[id].get() < self.current_epoch {
            self.distances[id].set(W::infinity());
            self.should_be_visited[id].set(false);
            self.time_stamps[id].set(self.current_epoch)
        }
    }
}

impl<'s, V, E, W> PathTracker<V, E> for LocalDijkstraDriver<'s, W>
where
    W: Weight,
{
    type Distance = W;

    fn get_distance(&self, vertex: VertexView<V>) -> Self::Distance {
        self.refresh(vertex.id);
        self.distances[vertex.id].get()
    }

    fn set_distance(&mut self, vertex: VertexView<V>, distance: Self::Distance) {
        self.refresh(vertex.id);
        self.distances[vertex.id].set(distance);
    }

    fn get_predecessor(&self, _vertex: VertexView<V>) -> Option<EdgeDescriptor> {
        None
    }
    fn set_predecessor(&mut self, _edge: EdgeView<V, E>) {}
}

impl<'s, V, E> Driver<V, E> for LocalDijkstraDriver<'s, E::Weight>
where
    E: Weighted,
{
    fn should_consider_edge(&self, edge: EdgeView<V, E>) -> bool {
        self.refresh(edge.end.id);
        is_uncontracted(edge.end.id, self.ranks)
    }

    fn should_consider_vertex(&self, vertex: VertexView<V>, _total_weight: E::Weight) -> bool {
        self.refresh(vertex.id);
        is_uncontracted(vertex.id, self.ranks)
    }

    fn visit(&mut self, vertex: VertexView<V>) -> bool {
        self.refresh(vertex.id);
        if self.should_be_visited[vertex.id].get() {
            self.num_visited += 1;
        }
        self.num_visited <= self.num_to_visit
    }
}
