use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Vertex<V, E> {
    pub id: usize,
    pub properties: V,
    pub edges: Vec<Edge<E>>,
}

impl<V, E> Vertex<V, E>
where V: Default
{
    pub fn new(id: usize) -> Self {
        Self {
            id,
            properties: V::default(),
            edges: vec![]
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Edge<E> {
    pub id: usize,
    pub end_id: usize,
    pub properties: E,
}

pub type Graph<V, E> = Vec<Vertex<V, E>>;
