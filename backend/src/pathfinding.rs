use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;

use crate::algo::InteractiveAlgo;
use crate::graphs::Graph;
use crate::presentation::{EventClient, GraphEvent, ServerAction};


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

pub struct Dijkstra<V, E, C>
where
    E: Distance,
    C: EventClient<V, E>,
{
    graph: Arc<Graph<V, E>>,
    distances: Vec<Num>,
    pending_routes: BinaryHeap<Route>,
    client: C,
}

impl<'a, V, E, C> InteractiveAlgo<(Arc<Graph<V, E>>, usize), C> for Dijkstra<V, E, C>
where
    V: Clone,
    E: Distance + Clone,
    C: EventClient<V, E>,
{
    type Result = Vec<Num>;

    async fn init(
        (graph, source_index): (Arc<Graph<V, E>>, usize),
        mut client: C,
    ) -> Self {
        let source_route = Route {
            destination_index: source_index,
            total_distance: 0,
        };
        let mut pending_routes = BinaryHeap::new();
        pending_routes.push(source_route);

        let mut distances = vec![Num::MAX; graph.len()];
        distances[source_index] = 0;

        client.consume(
            &GraphEvent { action: ServerAction::InitGraph { graph: graph.to_vec() }, comment: String::new() }
        ).await;

        Self {
            graph,
            distances,
            pending_routes,
            client
        }
    }

    fn step(&mut self) {
        let Some(route) = self.pending_routes.pop() else {
            return;
        };

        if route.total_distance != self.distances[route.destination_index] {
            return;
        }

        for edge in &self.graph[route.destination_index].edges {
            let neighbor_index = edge.end_index;
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
    }

    fn result(&self) -> Option<Self::Result> {
        if self.pending_routes.is_empty() {
            Some(self.distances.clone())
        } else {
            None
        }
    }
}
