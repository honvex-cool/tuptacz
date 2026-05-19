pub mod repr;

use std::ops::{Index, IndexMut};

use serde::{Deserialize, Serialize};

pub type VertexId = usize;

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EdgeId(pub usize);

#[derive(Clone, Serialize, Deserialize)]
pub struct Edge<E> {
    pub id: EdgeId,
    pub end_id: VertexId,
    pub props: E,
}

// Contract: VertexId is just an index into a vector
pub trait Graph: Index<VertexId, Output = Self::V> + IndexMut<VertexId, Output = Self::V> {
    type V;
    type E;

    fn get_num_vertices(&self) -> usize;
    fn add_vertex(&mut self, props: Self::V) -> VertexId;
    fn deactivate_vertex(&mut self, id: VertexId);
    fn add_edge(&mut self, start_id: VertexId, end_id: VertexId, props: Self::E) -> EdgeId;
    fn get_edges<'a>(&'a self, id: VertexId) -> impl Iterator<Item = &'a Edge<Self::E>> + 'a
    where
        Self::E: 'a;
    fn iter_vertices(&self) -> impl Iterator<Item = &Self::V>;
    fn iter_edges(&self) -> impl Iterator<Item = (VertexId, VertexId, &Self::E)>;
}
