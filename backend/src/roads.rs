use osm4routing::CarAccessibility;

use crate::geo::Coords;
use crate::graphs::{Graph, VertexId};

use std::collections::HashMap;
use std::path::Path;

pub type Intersection = Coords;
pub type Road = Vec<Coords>;

pub fn load<P, G>(path: P) -> Option<G>
where
    P: AsRef<Path>,
    G: Graph<V = Intersection, E = Road> + Default,
{
    let (nodes, edges) = osm4routing::read(path).ok()?;

    let edges: Vec<_> = edges.into_iter().filter(is_road).collect();

    let node_ids_to_coords: HashMap<_, Coords> = nodes
        .iter()
        .map(|node| (node.id, get_coords(&node.coord)))
        .collect();

    let mut graph = G::default();

    let node_ids_to_vertex_ids: HashMap<_, VertexId> = edges
        .iter()
        .map(|edge| [edge.source, edge.target])
        .flatten()
        .map(|node_id| (node_id, graph.add_vertex(node_ids_to_coords[&node_id])))
        .collect();

    for edge in &edges {
        let start_id = node_ids_to_vertex_ids[&edge.source];
        let end_id = node_ids_to_vertex_ids[&edge.target];
        let road = edge.geometry.iter().map(get_coords).collect();
        graph.add_edge(start_id, end_id, road);
    }

    Some(graph)
}

fn get_coords(coord: &geo_types::Coord) -> Coords {
    (coord.y, coord.x).into()
}

fn is_road(edge: &osm4routing::Edge) -> bool {
    is_car_accessible(edge.properties.car_forward)
        || is_car_accessible(edge.properties.car_backward)
}

fn is_car_accessible(car_accessibility: osm4routing::CarAccessibility) -> bool {
    match car_accessibility {
        CarAccessibility::Forbidden => false,
        CarAccessibility::Unknown => false,
        _ => true,
    }
}
