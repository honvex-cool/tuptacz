use serde::{Deserialize, Serialize};

use crate::graphs::{EdgeDescriptor, EdgeId, EdgeView, Graph, VertexId, VertexView};

use std::ops::{Index, IndexMut};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct AdjLists<V, E> {
    next_edge_id: EdgeId,
    vertices: Vec<Vertex<V, E>>,
}

impl<V, E> AdjLists<V, E> {
    fn with_vertices(vertices: Vec<Vertex<V, E>>) -> Self {
        Self {
            next_edge_id: EdgeId(0),
            vertices,
        }
    }
}

impl<V, E> AdjLists<V, E>
where
    V: Default,
{
    pub fn with_size(size: usize) -> Self {
        let vertices = (0..size).map(|_| Vertex::new(V::default())).collect();
        Self::with_vertices(vertices)
    }
}

impl<V, E> Index<VertexId> for AdjLists<V, E> {
    type Output = V;

    fn index(&self, index: VertexId) -> &Self::Output {
        &self.vertices[index].props
    }
}

impl<V, E> IndexMut<VertexId> for AdjLists<V, E> {
    fn index_mut(&mut self, index: VertexId) -> &mut Self::Output {
        &mut self.vertices[index].props
    }
}

impl<V, E> Graph for AdjLists<V, E> {
    type V = V;
    type E = E;

    fn with_estimates(num_vertices: usize, _num_edges: usize) -> Self {
        let vertices = Vec::with_capacity(num_vertices);
        Self::with_vertices(vertices)
    }

    #[inline(always)]
    fn num_vertices(&self) -> usize {
        self.vertices.len()
    }

    #[inline(always)]
    fn num_edges(&self) -> usize {
        self.next_edge_id.0
    }

    #[inline(always)]
    fn get_vertex(&self, id: VertexId) -> VertexView<'_, Self::V> {
        VertexView {
            id,
            props: &self.vertices[id].props,
        }
    }

    #[inline(always)]
    fn get_edge(&self, descriptor: EdgeDescriptor) -> EdgeView<'_, Self::V, Self::E> {
        let host_id = descriptor.host_id;
        let edge = &self.vertices[host_id].outgoing_edges[descriptor.index_within_host];
        edge.view_in(descriptor.host_id, descriptor.index_within_host, self)
    }

    fn add_vertex(&mut self, props: Self::V) -> VertexId {
        let len = self.num_vertices();
        self.vertices.push(Vertex::new(props));
        len
    }

    fn add_edge(&mut self, start_id: VertexId, end_id: VertexId, props: Self::E) -> EdgeId {
        let id = self.next_edge_id;
        self.next_edge_id.0 += 1;

        let outgoing_edges = &mut self.vertices[start_id].outgoing_edges;

        let edge = Edge { id, end_id, props };
        let edge_index = (start_id, outgoing_edges.len());

        outgoing_edges.push(edge);
        self.vertices[end_id].incoming_edge_indices.push(edge_index);

        id
    }

    #[inline(always)]
    fn num_outgoing_edges(&self, id: VertexId) -> usize {
        self.vertices[id].outgoing_edges.len()
    }

    #[inline(always)]
    fn iter_outgoing_edges(
        &self,
        id: VertexId,
    ) -> impl Iterator<Item = EdgeView<'_, Self::V, Self::E>> + '_ {
        self.vertices[id]
            .outgoing_edges
            .iter()
            .enumerate()
            .map(move |(index, edge)| edge.view_in(id, index, self))
    }

    #[inline(always)]
    fn num_incoming_edges(&self, id: VertexId) -> usize {
        self.vertices[id].incoming_edge_indices.len()
    }

    #[inline(always)]
    fn iter_incoming_edges(
        &self,
        id: VertexId,
    ) -> impl Iterator<Item = EdgeView<'_, Self::V, Self::E>> + '_ {
        self.vertices[id]
            .incoming_edge_indices
            .iter()
            .map(|&(vertex_id, edge_index)| {
                self.vertices[vertex_id].outgoing_edges[edge_index]
                    .view_in(vertex_id, edge_index, self)
            })
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct Vertex<V, E> {
    props: V,
    outgoing_edges: Vec<Edge<E>>,
    incoming_edge_indices: Vec<(VertexId, usize)>,
}

impl<V, E> Vertex<V, E> {
    #[inline(always)]
    fn new(props: V) -> Self {
        Self {
            props,
            outgoing_edges: vec![],
            incoming_edge_indices: vec![],
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct Edge<E> {
    pub id: EdgeId,
    pub end_id: VertexId,
    pub props: E,
}

impl<E> Edge<E> {
    #[inline(always)]
    fn view_in<'g, G>(
        &'g self,
        start_id: VertexId,
        index_within_host: usize,
        graph: &'g G,
    ) -> EdgeView<'g, G::V, E>
    where
        G: Graph<E = E>,
    {
        EdgeView {
            id: self.id,
            start: graph.get_vertex(start_id),
            end: graph.get_vertex(self.end_id),
            index_within_host,
            is_virtual: false,
            props: &self.props,
        }
    }
}
