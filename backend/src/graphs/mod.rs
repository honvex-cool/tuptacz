pub mod repr;

use std::ops::{Deref, Index, IndexMut};

use serde::{Deserialize, Serialize};

pub type VertexId = usize;

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EdgeId(pub usize);

pub struct VertexView<'g, V> {
    pub id: VertexId,
    props: &'g V,
}

impl<'g, V> Clone for VertexView<'g, V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'g, V> Copy for VertexView<'g, V> {}

impl<'g, V> Deref for VertexView<'g, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.props
    }
}

pub struct EdgeView<'g, V, E> {
    pub id: EdgeId,
    pub start: VertexView<'g, V>,
    pub end: VertexView<'g, V>,
    pub index_within_vertex: usize,
    props: &'g E,
}

impl<'g, V, E> EdgeView<'g, V, E> {
    pub fn flip(&self) -> Self {
        let mut copy = *self;
        std::mem::swap(&mut copy.start, &mut copy.end);
        copy
    }
}

impl<'g, V, E> Clone for EdgeView<'g, V, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'g, V, E> Copy for EdgeView<'g, V, E> {}

impl<'g, V, E> Deref for EdgeView<'g, V, E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        self.props
    }
}

// Contract: VertexId is just an index into a vector
pub trait Graph: Index<VertexId, Output = Self::V> + IndexMut<VertexId, Output = Self::V> {
    type V;
    type E;

    fn num_vertices(&self) -> usize;

    fn add_vertex(&mut self, props: Self::V) -> VertexId;
    fn add_edge(&mut self, start_id: VertexId, end_id: VertexId, props: Self::E) -> EdgeId;

    fn get_vertex(&self, id: VertexId) -> VertexView<'_, Self::V>;

    fn num_outgoing_edges(&self, id: VertexId) -> usize;
    fn iter_outgoing_edges(&self, id: VertexId)
    -> impl Iterator<Item = EdgeView<'_, Self::V, Self::E>> + '_;

    fn num_incoming_edges(&self, id: VertexId) -> usize;
    fn iter_incoming_edges(&self, id: VertexId)
    -> impl Iterator<Item = EdgeView<'_, Self::V, Self::E>> + '_;

    fn iter_vertices(&self) -> impl Iterator<Item = VertexView<'_, Self::V>> + '_ {
        (0..self.num_vertices()).map(|id| self.get_vertex(id))
    }

    fn iter_edges(&self) -> impl Iterator<Item = EdgeView<'_, Self::V, Self::E>> + '_ {
        self.iter_vertices()
            .flat_map(|vertex| self.iter_outgoing_edges(vertex.id))
    }
}
