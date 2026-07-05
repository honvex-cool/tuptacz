use crate::{
    graphs::{Graph, Path},
    routing::{
        Weighted,
        dijkstra::{Search, SearchResult, drivers::Driver},
    },
};

pub fn reconstruct_path<G, D>(
    graph: &G,
    result: SearchResult<<G::E as Weighted>::Weight, D>,
) -> Option<Path<G::V, G::E>>
where
    G: Graph,
    G::V: Clone,
    G::E: Clone + Weighted,
    D: Driver<G::V, G::E>,
{
    let Some(Search { mut id, mut driver }) = result.backward else {
        return None;
    };

    let Some(meeting_id) = result.meeting_vertex else {
        return None;
    };

    let Search {
        id: source_id,
        driver: forward_driver,
    } = result.forward;

    let mut path = vec![];

    let mut handler = |id, driver: &D| {
        let vertex = graph.get_vertex(id);
        let edge_descriptor = driver.get_predecessor(vertex).unwrap();
        let edge = graph.get_edge(edge_descriptor);

        path.push(edge.detach());

        edge.start.id
    };

    while id != meeting_id {
        id = handler(id, &driver);
    }

    driver = forward_driver;

    while id != source_id {
        id = handler(id, &driver);
    }

    Some(path)
}
