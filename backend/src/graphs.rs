use std::collections::BinaryHeap;

use crate::algo::Algo;

pub enum Event {
    HighlightVertex,
    HideVertex,
    HighlightEdge,
    HideEdge,
}

pub struct Vertex<V, E> {
    properties: V,
    edges: Vec<Edge<E>>,
}

pub struct Edge<E> {
    target_index: usize,
    properties: E,
}

pub type Graph<V, E> = [Vertex<V, E>];

pub type Num = i64;

pub trait Distance {
    fn distance(&self) -> Num;
}

impl Distance for Num {
    fn distance(&self) -> Num {
        *self
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Route {
    destination: usize,
    total_distance: Num,
}

pub struct Dijkstra<'a, V, E>
where
    E: Distance,
{
    graph: &'a Graph<V, E>,
    distances: Vec<Num>,
    pending_indices: BinaryHeap<Route>,
}

impl<'a, V, E> Algo<(&'a Graph<V, E>, usize), Event, Vec<Num>> for Dijkstra<'a, V, E>
where
    E: Distance,
{
    fn init((graph, source_index): (&'a Graph<V, E>, usize)) -> (Vec<Event>, Self) {
        let state = Self {
            graph,
            distances: vec![0i64; graph.len()],
            pending_indices: BinaryHeap::new(),
        };
        (vec![], state)
    }

    fn step(&mut self) -> Vec<Event> {
        vec![]
    }

    fn result(&self) -> Option<Vec<Num>> {
        if self.pending_indices.is_empty() {
            Some(self.distances.clone())
        } else {
            None
        }
    }
}
