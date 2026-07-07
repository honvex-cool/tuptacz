pub mod repr;

use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::id_type;

pub type VertexId = usize;

id_type!(EdgeId, usize);

#[derive(Debug)]
pub struct VertexView<'g, V> {
    pub id: VertexId,
    props: &'g V,
}

impl<'g, V> VertexView<'g, V>
where
    V: Clone,
{
    pub fn detach(&self) -> Vertex<V> {
        Vertex {
            id: self.id,
            props: self.props.clone(),
        }
    }
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

#[derive(Debug)]
pub struct EdgeView<'g, V, E> {
    pub id: EdgeId,
    pub start: VertexView<'g, V>,
    pub end: VertexView<'g, V>,
    is_virtual: bool,
    index_within_host: usize,
    props: &'g E,
}

impl<'g, V, E> EdgeView<'g, V, E>
where
    V: Clone,
    E: Clone,
{
    pub fn detach(&self) -> Edge<V, E> {
        Edge {
            start: self.start.detach(),
            end: self.end.detach(),
            id: self.id,
            descriptor: self.descriptor(),
            props: self.props.clone(),
        }
    }
}

impl<'g, V, E> EdgeView<'g, V, E> {
    #[inline(always)]
    pub fn flip(&self) -> Self {
        Self {
            id: self.id,
            start: self.end,
            end: self.start,
            is_virtual: !self.is_virtual,
            index_within_host: self.index_within_host,
            props: self.props,
        }
    }

    #[inline(always)]
    pub fn descriptor(&self) -> EdgeDescriptor {
        EdgeDescriptor {
            host_id: self.host().id,
            index_within_host: self.index_within_host,
        }
    }

    #[inline(always)]
    fn host(&self) -> VertexView<'_, V> {
        if self.is_virtual {
            self.end
        } else {
            self.start
        }
    }
}

impl<'g, V, E> Clone for EdgeView<'g, V, E> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'g, V, E> Copy for EdgeView<'g, V, E> {}

impl<'g, V, E> Deref for EdgeView<'g, V, E> {
    type Target = E;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.props
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EdgeDescriptor {
    host_id: VertexId,
    index_within_host: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vertex<V> {
    pub id: VertexId,
    pub props: V,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Edge<V, E> {
    pub start: Vertex<V>,
    pub end: Vertex<V>,
    pub id: EdgeId,
    pub descriptor: EdgeDescriptor,
    pub props: E,
}

pub type Path<V, E> = Vec<Edge<V, E>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphElements<V, E> {
    pub vertices: Vec<V>,
    pub edges: Vec<(VertexId, VertexId, E, bool)>,
}

impl<V, E> GraphElements<V, E>
where
    V: Clone,
    E: Clone,
{
    pub fn to_graph<G>(self) -> G
    where
        G: Graph<V = V, E = E>,
    {
        let mut graph = G::with_estimates(self.vertices.len(), self.edges.len());
        for vertex_props in self.vertices {
            graph.add_vertex(vertex_props);
        }
        for (start_id, end_id, edge_props, is_bidirectional) in self.edges {
            if is_bidirectional {
                graph.add_edge(end_id, start_id, edge_props.clone());
            }
            graph.add_edge(start_id, end_id, edge_props);
        }
        graph
    }
}

// Contract: VertexId is just an index into a vector
pub trait Graph {
    type V: Clone;
    type E: Clone;

    fn with_estimates(num_vertices: usize, num_edges: usize) -> Self;

    fn num_vertices(&self) -> usize;
    fn num_edges(&self) -> usize;

    fn add_vertex(&mut self, props: Self::V) -> VertexId;
    fn add_edge(&mut self, start_id: VertexId, end_id: VertexId, props: Self::E) -> EdgeId;

    fn get_vertex(&self, id: VertexId) -> VertexView<'_, Self::V>;
    fn get_edge(&self, descriptor: EdgeDescriptor) -> EdgeView<'_, Self::V, Self::E>;

    fn num_outgoing_edges(&self, id: VertexId) -> usize;
    fn iter_outgoing_edges(
        &self,
        id: VertexId,
    ) -> impl Iterator<Item = EdgeView<'_, Self::V, Self::E>> + '_;

    fn num_incoming_edges(&self, id: VertexId) -> usize;
    fn iter_incoming_edges(
        &self,
        id: VertexId,
    ) -> impl Iterator<Item = EdgeView<'_, Self::V, Self::E>> + '_;

    fn iter_vertices(&self) -> impl Iterator<Item = VertexView<'_, Self::V>> + '_ {
        (0..self.num_vertices()).map(|id| self.get_vertex(id))
    }

    fn iter_edges(&self) -> impl Iterator<Item = EdgeView<'_, Self::V, Self::E>> + '_ {
        self.iter_vertices()
            .flat_map(|vertex| self.iter_outgoing_edges(vertex.id))
    }
}
