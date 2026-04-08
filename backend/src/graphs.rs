use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::algo::InteractiveAlgo;

pub enum Event {
    HighlightVertex,
    HideVertex,
    HighlightEdge,
    HideEdge,
}

pub struct Vertex<V, E> {
    pub properties: V,
    pub edges: Vec<Edge<E>>,
}

pub struct Edge<E> {
    pub target_index: usize,
    pub properties: E,
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

#[derive(Clone, Copy, PartialEq, Eq)]
struct Route {
    destination_index: usize,
    total_distance: Num,
}

impl Ord for Route {
    fn cmp(&self, other: &Self) -> Ordering {
        let total_distance_ordering = self.total_distance.cmp(&other.total_distance).reverse();
        let destination_index_ordering = self.destination_index.cmp(&other.destination_index);
        total_distance_ordering.then(destination_index_ordering)
    }
}

impl PartialOrd for Route {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Dijkstra<'a, V, E>
where
    E: Distance,
{
    graph: &'a Graph<V, E>,
    distances: Vec<Num>,
    pending_routes: BinaryHeap<Route>,
}

impl<'a, V, E> InteractiveAlgo<(&'a Graph<V, E>, usize)> for Dijkstra<'a, V, E>
where
    E: Distance,
{
    type Event = Event;
    type Result = Vec<Num>;

    fn init((graph, source_index): (&'a Graph<V, E>, usize)) -> (Vec<Self::Event>, Self) {
        let source_route = Route {
            destination_index: source_index,
            total_distance: 0,
        };
        let mut pending_routes = BinaryHeap::new();
        pending_routes.push(source_route);

        let mut distances = vec![Num::MAX; graph.len()];
        distances[source_index] = 0;

        let state = Self {
            graph,
            distances,
            pending_routes,
        };

        (vec![], state)
    }

    fn step(&mut self) -> Vec<Event> {
        let Some(route) = self.pending_routes.pop() else {
            return vec![];
        };

        if route.total_distance != self.distances[route.destination_index] {
            return vec![];
        }

        for edge in &self.graph[route.destination_index].edges {
            let neighbor_index = edge.target_index;
            let neighbor_distance = &mut self.distances[neighbor_index];

            let new_total_distance = route.total_distance + edge.properties.distance();

            if new_total_distance < *neighbor_distance {
                *neighbor_distance = new_total_distance;

                let new_route = Route {
                    destination_index: neighbor_index,
                    total_distance: *neighbor_distance,
                };
                self.pending_routes.push(new_route);
            }
        }

        vec![]
    }

    fn result(&self) -> Option<Self::Result> {
        if self.pending_routes.is_empty() {
            Some(self.distances.clone())
        } else {
            None
        }
    }
}
