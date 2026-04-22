use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Vertex<V, E> {
    pub id: usize,
    pub properties: V,
    pub edges: Vec<Edge<E>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Edge<E> {
    pub id: usize,
    pub end_index: usize,
    pub properties: E,
}

pub type Graph<V, E> = Vec<Vertex<V, E>>;
