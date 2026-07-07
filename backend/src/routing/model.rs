use std::collections::HashMap;

use kiddo::{KdTree, float::distance::SquaredEuclidean};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    graphs::{GraphElements, VertexId},
    routing::Weighted,
};

pub type Float = OrderedFloat<f64>;
pub type LatLng = crate::utils::geo::LatLng<Float>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Road {
    pub points: Vec<LatLng>,
    pub length: Float,
}

impl From<Float> for Road {
    fn from(value: Float) -> Self {
        Self {
            points: vec![],
            length: value,
        }
    }
}

impl Weighted for Road {
    type Weight = Float;

    fn weight(&self) -> Self::Weight {
        self.length
    }
}

pub struct RoutingNetwork<E> {
    pub graph_elements: GraphElements<LatLng, E>,
    pub polygon: Value,
    lat_lng_to_vertex_id: KdTree<Float, 2>,
}

impl<E> RoutingNetwork<E> {
    pub fn new(graph_elements: GraphElements<LatLng, E>, polygon: Value) -> Self {
        let mut lat_lng_to_vertex_id = KdTree::with_capacity(graph_elements.vertices.len());

        for (id, vertex) in graph_elements.vertices.iter().enumerate() {
            let lat_lng = [vertex.latitude, vertex.longitude];
            lat_lng_to_vertex_id.add(&lat_lng, id as u64);
        }

        Self {
            graph_elements,
            polygon,
            lat_lng_to_vertex_id,
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.graph_elements.vertices.len(), self.graph_elements.edges.len())
    }

    pub fn nerest_vertex_id(&self, lat_lng: LatLng) -> VertexId {
        let lat_lng = [lat_lng.latitude, lat_lng.longitude];
        let index = self
            .lat_lng_to_vertex_id
            .nearest_one::<SquaredEuclidean>(&lat_lng)
            .item;
        index as VertexId
    }
}

pub type RoutingInfo = HashMap<String, RoutingNetwork<Road>>;
