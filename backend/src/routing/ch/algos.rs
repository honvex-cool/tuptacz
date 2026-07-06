use crate::{
    graphs::{
        EdgeDescriptor, EdgeView, Graph, GraphElements, Path, VertexId, VertexView, repr::AdjLists,
    },
    routing::{
        self, BasicVertexDataArray, RoutingAlgo, Weighted,
        ch::{
            Rank, ShortcutBreakdown,
            contraction::{Config, Contraction},
        },
        dijkstra::{
            BidirectionalDrivenDijkstra, Controller, Queue, Search,
            drivers::{Driver, PathTracker, VertexTracker},
            heuristics::ZeroHeuristic,
            policies::{Alternating, BoundReachedSeparately},
        },
        pathfinding::reconstruct_path,
        presentation::GraphEvent,
    },
    utils::{
        algo::{self, EventClient, QueryEngine},
        pq::NullTracker,
        staged::Staged,
    },
};

pub fn ch<V, E, C>(
    graph_elements: GraphElements<V, E>,
    config: Config,
) -> Box<RoutingAlgo<'static, V, E, C>>
where
    V: Clone + 'static,
    E: Clone + Weighted + 'static,
    C: EventClient<GraphEvent<V, E>> + 'static,
{
    let graph: AdjLists<_, _> = graph_elements.to_graph();

    let preprocessing = Contraction::new(graph, config);
    let preprocessing = algo::map(
        preprocessing,
        |(graph, ranks, num_original_edges, breakdowns)| -> Box<routing::Pathfinder<V, E, C>> {
            Box::new(Pathfinder::new(
                graph,
                ranks,
                num_original_edges,
                breakdowns,
            ))
        },
    );

    Box::new(preprocessing)
}

struct Pathfinder<G>
where
    G: Graph,
    G::E: Weighted,
{
    graph: G,
    ranks: Vec<Rank>,

    num_original_edges: usize,
    breakdowns: Vec<ShortcutBreakdown>,

    vertex_data: BasicVertexDataArray<<G::E as Weighted>::Weight>,
}

impl<G> Pathfinder<G>
where
    G: Graph,
    G::E: Weighted,
{
    fn new(
        graph: G,
        ranks: Vec<Rank>,
        num_original_edges: usize,
        breakdowns: Vec<ShortcutBreakdown>,
    ) -> Self {
        let num_vertices = graph.num_vertices();
        Self {
            graph,
            ranks,

            num_original_edges,
            breakdowns,

            vertex_data: BasicVertexDataArray::with_size(num_vertices),
        }
    }
}

impl<G, C> QueryEngine<C, GraphEvent<G::V, G::E>> for Pathfinder<G>
where
    G: Graph,
    G::E: Weighted,
    C: EventClient<GraphEvent<G::V, G::E>> + 'static,
{
    type Input = (VertexId, VertexId);

    type Result = Option<Path<G::V, G::E>>;

    fn query<'a>(
        &'a mut self,
        (source_id, target_id): Self::Input,
        client: &mut C,
    ) -> Box<
        dyn crate::utils::algo::InteractiveAlgo<C, GraphEvent<G::V, G::E>, Result = Self::Result>
            + 'a,
    > {
        let (forward, backward) = self.vertex_data.stage();

        let forward = Search {
            id: source_id,
            driver: RankBasedDriver {
                ranks: &self.ranks,
                inner: forward,
            },
        };
        let forward = Controller {
            search: forward,
            heuristic: ZeroHeuristic,
            queue: Queue::with_index_tracker(NullTracker),
        };

        let backward = Search {
            id: target_id,
            driver: RankBasedDriver {
                ranks: &self.ranks,
                inner: backward,
            },
        };
        let backward = Controller {
            search: backward,
            heuristic: ZeroHeuristic,
            queue: Queue::with_index_tracker(NullTracker),
        };

        let dijkstra = BidirectionalDrivenDijkstra::new(
            &self.graph,
            forward,
            Some(backward),
            Alternating::default(),
            BoundReachedSeparately,
            client,
        );

        let dijkstra = algo::map(dijkstra, |result| reconstruct_path(&self.graph, result));

        let dijkstra = algo::map(dijkstra, |path| {
            path.map(|p| {
                unpack_shortcuts(&self.graph, &self.breakdowns, self.num_original_edges, p)
            })
        });

        Box::new(dijkstra)
    }
}

struct RankBasedDriver<'r, W> {
    ranks: &'r [Rank],
    inner: Staged<'r, (W, Option<EdgeDescriptor>)>,
}

impl<'r, V, E> PathTracker<V, E> for RankBasedDriver<'r, E::Weight>
where
    E: Weighted,
    (E::Weight, Option<EdgeDescriptor>): VertexTracker<V, E, Distance = E::Weight>,
{
    type Distance = E::Weight;

    #[inline(always)]
    fn get_distance(&self, vertex: VertexView<V>) -> Self::Distance {
        self.inner.get_distance(vertex)
    }

    #[inline(always)]
    fn set_distance(&mut self, vertex: VertexView<V>, distance: Self::Distance) {
        self.inner.set_distance(vertex, distance);
    }

    #[inline(always)]
    fn get_predecessor(&self, vertex: VertexView<V>) -> Option<EdgeDescriptor> {
        self.inner.get_predecessor(vertex)
    }

    #[inline(always)]
    fn set_predecessor(&mut self, edge: EdgeView<V, E>) {
        self.inner.set_predecessor(edge);
    }
}

impl<'r, V, E> Driver<V, E> for RankBasedDriver<'r, E::Weight>
where
    E: Weighted,
{
    fn should_consider_edge(&self, edge: EdgeView<V, E>) -> bool {
        self.ranks[edge.start.id] < self.ranks[edge.end.id]
    }
}

fn unpack_shortcuts<G>(
    graph: &G,
    breakdowns: &[ShortcutBreakdown],
    num_original_edges: usize,
    path: Path<G::V, G::E>,
) -> Path<G::V, G::E>
where
    G: Graph,
    G::V: Clone,
    G::E: Clone,
{
    let mut unpacked_path = vec![];

    for edge in path {
        let mut stack = vec![edge.descriptor];
        while let Some(descriptor) = stack.pop() {
            let edge = graph.get_edge(descriptor);
            if edge.id.0 < num_original_edges {
                unpacked_path.push(edge.detach());
            } else {
                let [first, second] = breakdowns[edge.id.0 - num_original_edges];
                stack.push(second);
                stack.push(first);
            }
        }
    }

    unpacked_path
}
