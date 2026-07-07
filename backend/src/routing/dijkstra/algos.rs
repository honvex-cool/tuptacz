use crate::{
    graphs::{Graph, GraphElements, Path, VertexId, repr::AdjLists},
    routing::{
        BasicVertexDataArray, NoPreprocessing, RoutingAlgo, Weighted,
        dijkstra::{
            BidirectionalDrivenDijkstra, Controller, Queue, Search,
            heuristics::ZeroHeuristic,
            policies::{Alternating, AlwaysForward, BoundReachedJointly, EndToEnd},
        },
        pathfinding::reconstruct_path,
        presentation::GraphEvent,
    },
    utils::{
        algo::{self, EventClient, InteractiveAlgo, QueryEngine},
        pq::NullTracker,
    },
};

pub fn dijkstra<V, E, C>(
    graph_elements: GraphElements<V, E>,
    is_bidirectional: bool,
) -> Box<RoutingAlgo<'static, V, E, C>>
where
    V: Clone + 'static,
    E: Clone + Weighted + 'static,
    C: EventClient<GraphEvent<V, E>> + 'static,
{
    let graph: AdjLists<_, _> = graph_elements.to_graph();
    let pathfinder = Box::new(Pathfinder::new(graph, is_bidirectional));
    Box::new(NoPreprocessing(pathfinder))
}

struct Pathfinder<G>
where
    G: Graph,
    G::E: Weighted,
{
    graph: G,
    is_bidirectional: bool,
    vertex_data: BasicVertexDataArray<<G::E as Weighted>::Weight>,
}

impl<G> Pathfinder<G>
where
    G: Graph,
    G::E: Weighted,
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
    ) -> Box<dyn InteractiveAlgo<C, GraphEvent<G::V, G::E>, Result = Self::Result> + 'a> {
        let (forward_driver, backward_driver) = self.vertex_data.stage();

        let forward_search = Search {
            id: source_id,
            driver: forward_driver,
        };
        let forward = Controller {
            search: forward_search,
            heuristic: ZeroHeuristic,
            queue: Queue::with_index_tracker(NullTracker),
        };

        let backward_search = Search {
            id: target_id,
            driver: backward_driver,
        };
        let backward = Controller {
            search: backward_search,
            heuristic: ZeroHeuristic,
            queue: Queue::with_index_tracker(NullTracker),
        };

        if self.is_bidirectional {
            let dijkstra = BidirectionalDrivenDijkstra::new(
                &self.graph,
                forward,
                Some(backward),
                Alternating::default(),
                BoundReachedJointly,
                client,
            );
            Box::new(algo::map(dijkstra, |result| {
                reconstruct_path(&self.graph, result)
            }))
        } else {
            let dijkstra = BidirectionalDrivenDijkstra::new(
                &self.graph,
                forward,
                Some(backward),
                AlwaysForward,
                EndToEnd {
                    source_id,
                    target_id,
                },
                client,
            );
            Box::new(algo::map(dijkstra, |result| {
                reconstruct_path(&self.graph, result)
            }))
        }
    }
}
