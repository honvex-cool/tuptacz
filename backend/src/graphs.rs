use std::ops::{Index, IndexMut};

use serde::{Deserialize, Serialize};

pub type VertexId = usize;

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EdgeId(pub usize);

#[derive(Serialize, Deserialize, Clone)]
pub struct Vertex<V, E> {
    props: V,
    edges: Vec<Edge<E>>,
    is_active: bool,
}

impl<V, E> Vertex<V, E> {
    pub fn new(props: V) -> Self {
        Self {
            props,
            edges: vec![],
            is_active: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
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
}

#[derive(Clone)]
pub struct AdjListRepr<V, E> {
    next_edge_id: EdgeId,
    vertices: Vec<Vertex<V, E>>,
}

impl<V, E> AdjListRepr<V, E> {
    pub fn new() -> Self {
        Self {
            next_edge_id: EdgeId(0),
            vertices: vec![],
        }
    }
}

impl<V, E> AdjListRepr<V, E>
where
    V: Default,
{
    pub fn with_size(size: usize) -> Self {
        let vertices = (0..size).map(|_| Vertex::new(V::default())).collect();
        Self {
            next_edge_id: EdgeId(0),
            vertices,
        }
    }
}

impl<V, E> Index<VertexId> for AdjListRepr<V, E> {
    type Output = V;

    fn index(&self, index: VertexId) -> &Self::Output {
        &self.vertices[index].props
    }
}

impl<V, E> IndexMut<VertexId> for AdjListRepr<V, E> {
    fn index_mut(&mut self, index: VertexId) -> &mut Self::Output {
        &mut self.vertices[index].props
    }
}

impl<V, E> Graph for AdjListRepr<V, E> {
    type V = V;
    type E = E;

    fn get_num_vertices(&self) -> usize {
        self.vertices.len()
    }

    fn add_vertex(&mut self, props: Self::V) -> VertexId {
        let len = self.get_num_vertices();
        self.vertices.push(Vertex::new(props));
        len
    }

    fn deactivate_vertex(&mut self, id: VertexId) {
        self.vertices[id].is_active = false;
    }

    fn add_edge(&mut self, start_id: VertexId, end_id: VertexId, props: Self::E) -> EdgeId {
        let id = self.next_edge_id;
        self.next_edge_id.0 += 1;

        let edge = Edge { id, end_id, props };
        self.vertices[start_id].edges.push(edge);

        id
    }

    fn get_edges<'a>(&'a self, id: VertexId) -> impl Iterator<Item = &'a Edge<Self::E>> + 'a
    where
        E: 'a,
    {
        self.vertices[id]
            .edges
            .iter()
            .filter(|edge| self.vertices[edge.end_id].is_active)
    }
}
