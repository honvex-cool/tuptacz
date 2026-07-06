use crate::{
    graphs::{Graph, GraphElements, Path, VertexId, VertexView, repr::AdjLists},
    routing::{
        BasicVertexDataArray, NoPreprocessing, RoutingAlgo, Weighted,
        dijkstra::{
            BidirectionalDrivenDijkstra, Controller, Queue, Search,
            heuristics::Heuristic,
            policies::{Alternating, AlwaysForward, BoundReachedByEither, EndToEnd},
        },
        model::{Float, LatLng},
        pathfinding::reconstruct_path,
        presentation::GraphEvent,
    },
    utils::{
        algo::{self, EventClient, InteractiveAlgo, QueryEngine},
        pq::NullTracker,
    },
};

pub fn a_star<E, C>(
    graph_elements: GraphElements<LatLng, E>,
    is_bidirectional: bool,
) -> Box<RoutingAlgo<'static, LatLng, E, C>>
where
    E: Clone + Weighted<Weight = Float> + 'static,
    C: EventClient<GraphEvent<LatLng, E>> + 'static,
{
    let graph: AdjLists<_, _> = graph_elements.to_graph();
    let pathfinder = Box::new(Pathfinder::new(graph, is_bidirectional));
    Box::new(NoPreprocessing(pathfinder))
}

struct Pathfinder<G>
where
    G: Graph,
    G::E: Weighted<Weight = Float>,
{
    graph: G,
    is_bidirectional: bool,
    vertex_data: BasicVertexDataArray<<G::E as Weighted>::Weight>,
}

impl<G> Pathfinder<G>
where
    G: Graph,
    G::E: Weighted<Weight = Float>,
{
    fn new(graph: G, is_bidirectional: bool) -> Self {
        let num_vertices = graph.num_vertices();
        Self {
            graph,
            is_bidirectional,
            vertex_data: BasicVertexDataArray::with_size(num_vertices),
        }
    }
}

impl<G, C> QueryEngine<C, GraphEvent<LatLng, G::E>> for Pathfinder<G>
where
    G: Graph<V = LatLng>,
    G::E: Weighted<Weight = Float>,
    C: EventClient<GraphEvent<G::V, G::E>> + 'static,
{
    type Input = (VertexId, VertexId);

    type Result = Option<Path<G::V, G::E>>;

    fn query<'a>(
        &'a mut self,
        (source_id, target_id): Self::Input,
        client: &mut C,
    ) -> Box<dyn InteractiveAlgo<C, GraphEvent<G::V, G::E>, Result = Self::Result> + 'a> {
        let (forward_driver, backward_driver) = self.vertex_data.stage();

        let source = *self.graph.get_vertex(source_id);
        let target = *self.graph.get_vertex(target_id);

        let forward_search = Search {
            id: source_id,
            driver: forward_driver,
        };

        let backward_search = Search {
            id: target_id,
            driver: backward_driver,
        };

        if self.is_bidirectional {
            let forward = Controller {
                search: forward_search,
                heuristic: AdjustedEuclidean { source, target },
                queue: Queue::with_index_tracker(NullTracker),
            };
            let backward = Controller {
                search: backward_search,
                heuristic: AdjustedEuclidean {
                    source: target,
                    target: source,
                },
                queue: Queue::with_index_tracker(NullTracker),
            };
            let a_star = BidirectionalDrivenDijkstra::new(
                &self.graph,
                forward,
                Some(backward),
                Alternating::default(),
                BoundReachedByEither,
                client,
            );
            Box::new(algo::map(a_star, |result| {
                reconstruct_path(&self.graph, result)
            }))
        } else {
            let forward = Controller {
                search: forward_search,
                heuristic: Euclidean(target),
                queue: Queue::with_index_tracker(NullTracker),
            };
            let backward = Controller {
                search: backward_search,
                heuristic: Euclidean(source),
                queue: Queue::with_index_tracker(NullTracker),
            };
            let a_star = BidirectionalDrivenDijkstra::new(
                &self.graph,
                forward,
                Some(backward),
                AlwaysForward::default(),
                EndToEnd {
                    source_id,
                    target_id,
                },
                client,
            );
            Box::new(algo::map(a_star, |result| {
                reconstruct_path(&self.graph, result)
            }))
        }
    }
}

struct Euclidean(LatLng);

impl Heuristic<LatLng, Float> for Euclidean {
    fn calculate(&self, vertex: VertexView<LatLng>) -> Float {
        self.0.distance_meters(*vertex)
    }
}

struct AdjustedEuclidean {
    source: LatLng,
    target: LatLng,
}

impl Heuristic<LatLng, Float> for AdjustedEuclidean {
    fn calculate(&self, vertex: VertexView<LatLng>) -> Float {
        let vertex = *vertex;
        (self.target.distance_meters(vertex) - self.source.distance_meters(vertex)) / 2.0
    }
}
