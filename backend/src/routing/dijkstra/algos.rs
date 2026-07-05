use crate::{
    graphs::{Graph, GraphElements, Path, VertexId},
    routing::{BasicVertexDataArray, Weighted, presentation::GraphEvent},
    utils::algo::{self, EventClient, InteractiveAlgo},
};

pub struct Dijkstra<V, E> {
    is_bidirectional: bool,
    graph_elements: GraphElements<V, E>,
}

impl<V, E, C> InteractiveAlgo<C, GraphEvent<V, E>> for Dijkstra<V, E>
where
    V: Clone,
    E: Clone + Weighted,
    C: EventClient<GraphEvent<V, E>>,
{
    type Result = Box<crate::routing::Pathfinder<V, E, C>>;

    fn step(&mut self, _client: &mut C) -> bool {
        todo!()
    }

    fn result(self) -> Self::Result {
        todo!()
    }

    fn result_dyn(self: Box<Self>) -> Self::Result {
        todo!()
    }
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

impl<G, C> algo::QueryEngine<C, GraphEvent<G::V, G::E>> for Pathfinder<G>
where
    G: Graph,
    G::E: Weighted,
    C: EventClient<GraphEvent<G::V, G::E>>,
{
    type Input = (VertexId, VertexId);

    type Result = Path<G::V, G::E>;

    fn query<'a>(
        &'a mut self,
        _query: Self::Input,
        _client: &mut C,
    ) -> Box<dyn algo::InteractiveAlgo<C, GraphEvent<G::V, G::E>, Result = Self::Result> + 'a> {
        todo!();
    }
}
