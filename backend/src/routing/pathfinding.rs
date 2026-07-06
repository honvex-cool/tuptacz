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
    G::E: Weighted,
    D: Driver<G::V, G::E>,
{
    let Search {
        id: target_id,
        driver: backward_driver,
    } = result.backward?;

    let meeting_id = result.meeting_id?;

    let Search {
        id: source_id,
        driver: forward_driver,
    } = result.forward;

    let mut path = vec![];

    let handler = |id, path: &mut Path<_, _>, driver: &D, label: &str| {
        let vertex = graph.get_vertex(id);
        let edge_descriptor = driver.get_predecessor(vertex).unwrap_or_else(|| {
            panic!(
                "no {} predecessor for vertex {} (meeting_id={}, source_id={})",
                label, id, meeting_id, source_id
            );
        });
        let edge = graph.get_edge(edge_descriptor);

        path.push(edge.detach());

        edge
    };

    let mut id = meeting_id;
    while id != source_id {
        id = handler(id, &mut path, &forward_driver, "fwd").start.id;
    }

    path.reverse();

    let mut id = meeting_id;
    while id != target_id {
        id = handler(id, &mut path, &backward_driver, "bwd").end.id;
    }

    Some(path)
}
