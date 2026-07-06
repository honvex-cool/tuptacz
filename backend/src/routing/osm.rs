use crate::graphs::{GraphElements, VertexId};
use crate::routing::model::{LatLng, Road, RoutingNetwork};

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;

pub fn load_routing_network<P>(directory_path: P) -> io::Result<RoutingNetwork<Road>>
where
    P: AsRef<Path>,
{
    let directory_path = directory_path.as_ref();

    let osm_path_buf = directory_path.join("roads.osm.pbf");
    let osm_path = osm_path_buf.as_path();

    let graph_path_buf = directory_path.join("graph.bin");
    let graph_path = graph_path_buf.as_path();

    let polygon_path_buf = directory_path.join("polygon.json");
    let polygon_path = polygon_path_buf.as_path();

    let graph_elements = load_graph_elements(osm_path, graph_path)?;

    let polygon = {
        let file = File::open(polygon_path)?;
        let mut reader = BufReader::new(file);
        serde_json::from_reader(&mut reader)?
    };

    let routing_network = RoutingNetwork::new(graph_elements, polygon);

    Ok(routing_network)
}

pub fn load_graph_elements<PO, PG>(
    osm_path: PO,
    graph_path: PG,
) -> io::Result<GraphElements<LatLng, Road>>
where
    PO: AsRef<Path>,
    PG: AsRef<Path>,
{
    let osm_path = osm_path.as_ref();
    let graph_path = graph_path.as_ref();

    load_initial(graph_path, osm_path)?;

    eprintln!("Reading graph elements from {}", graph_path.display());

    let graph_elements: GraphElements<_, _> = {
        let file = File::open(graph_path)?;
        let mut reader = BufReader::new(file);
        bincode::deserialize_from(&mut reader).map_err(io::Error::other)?
    };

    Ok(graph_elements)
}

fn load_initial(graph_path: &Path, osm_path: &Path) -> io::Result<()>
where
{
    if !graph_path.is_file() {
        let osm_path_display = osm_path.display();

        eprintln!(
            "osm4routing is reading {} for the first time, this may take a while",
            osm_path_display
        );

        let (mut nodes, mut edges) = osm4routing::read(osm_path).map_err(io::Error::other)?;

        eprintln!("osm4routing finished reading {}", osm_path_display);

        eprintln!("Filterting relevant edges");

        edges.retain(is_relevant_edge);

        eprintln!("Filtering endpoints");

        let endpoints: HashSet<_> = edges
            .iter()
            .flat_map(|edge| [edge.source, edge.target].into_iter())
            .collect();
        nodes.retain(|node| endpoints.contains(&node.id));

        let mut node_ids_to_vertices = HashMap::new();
        for node in nodes {
            let vertex_id = node_ids_to_vertices.len();
            let lat_lng = get_lat_lng(&node.coord);
            node_ids_to_vertices.insert(node.id, (vertex_id, lat_lng));
        }

        eprintln!("Collecting edges");

        let edges = edges
            .into_iter()
            .map(|edge| get_edge(&node_ids_to_vertices, edge))
            .collect();

        eprintln!("Collecting vertices");

        let mut vertices: Vec<_> = node_ids_to_vertices.into_values().collect();
        vertices.sort_by_key(|(id, _)| *id);

        let vertices = vertices.into_iter().map(|(_, lat_lng)| lat_lng).collect();

        eprintln!("Preparing convenient graph file");

        let graph_elements = GraphElements { vertices, edges };

        let mut writer = {
            let file = File::create(graph_path)?;
            BufWriter::new(file)
        };
        bincode::serialize_into(&mut writer, &graph_elements).map_err(io::Error::other)?;

        eprintln!("Graph written to {}", graph_path.display());
    }

    Ok(())
}

fn get_lat_lng(coord: &geo_types::Coord) -> LatLng {
    LatLng {
        latitude: coord.y.into(),
        longitude: coord.x.into(),
    }
}

fn get_edge(
    node_ids_to_vertices: &HashMap<osm4routing::NodeId, (VertexId, LatLng)>,
    edge: osm4routing::Edge,
) -> (VertexId, VertexId, Road, bool) {
    let start_id = node_ids_to_vertices.get(&edge.source).unwrap().0;
    let end_id = node_ids_to_vertices.get(&edge.target).unwrap().0;

    let points: Vec<_> = edge.geometry.iter().map(get_lat_lng).collect();
    let length = LatLng::poly_distance_meters(&points);

    let mut road = Road { points, length };

    if !is_car_accessible(edge.properties.car_forward) {
        road.points.reverse();
        (end_id, start_id, road, false)
    } else {
        (
            start_id,
            end_id,
            road,
            is_car_accessible(edge.properties.car_backward),
        )
    }
}

fn is_relevant_edge(edge: &osm4routing::Edge) -> bool {
    edge.source != edge.target && is_road(edge)
}

fn is_road(edge: &osm4routing::Edge) -> bool {
    is_car_accessible(edge.properties.car_forward)
        || is_car_accessible(edge.properties.car_backward)
}

fn is_car_accessible(car_accessibility: osm4routing::CarAccessibility) -> bool {
    match car_accessibility {
        osm4routing::CarAccessibility::Forbidden => false,
        osm4routing::CarAccessibility::Unknown => false,
        _ => true,
    }
}
