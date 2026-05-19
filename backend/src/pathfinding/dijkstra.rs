use crate::algo::{EventClient, InteractiveAlgo};
use crate::graphs::{Graph, VertexId};

use crate::presentation::{GraphEvent, HighlightMode, ServerAction};

use crate::pathfinding::{Distance, Num};

use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub struct Dijkstra<G>
where
    G: Graph,
    G::E: Distance,
{
    graph: G,
    distances: Vec<Num>,
    pending_routes: BinaryHeap<Route>,
}

impl<G> Dijkstra<G>
where
    G::E: Distance,
    G: Graph + Clone,
{
    fn highlight_source<C>(vertex_id: VertexId, client: &mut C)
    where
        C: EventClient<GraphEvent<G::V, G::E>>,
    {
        client.consume(GraphEvent {
            action: ServerAction::HighlightVertex {
                id: vertex_id,
                mode: HighlightMode::Source,
            },
            comment: "Starting from vertex".to_owned(),
        });
    }

    fn highlight_visited<C>(&self, vertex_id: VertexId, client: &mut C)
    where
        C: EventClient<GraphEvent<G::V, G::E>>,
    {
        client.consume(GraphEvent {
            action: ServerAction::HighlightVertex {
                id: vertex_id,
                mode: HighlightMode::Visited,
            },
            comment: "Visited vertex".to_owned(),
        });
    }

    fn highlight_awaiting<C>(&self, vertex_id: VertexId, client: &mut C)
    where
        C: EventClient<GraphEvent<G::V, G::E>>,
    {
        client.consume(GraphEvent {
            action: ServerAction::HighlightVertex {
                id: vertex_id,
                mode: HighlightMode::Awaiting,
            },
            comment: "Put vertex to queue".to_owned(),
        });
    }
}

impl<G, C> InteractiveAlgo<(G, VertexId), GraphEvent<G::V, G::E>, C> for Dijkstra<G>
where
    G: Graph + Clone,
    G::E: Distance,
    C: EventClient<GraphEvent<G::V, G::E>>,
{
    type Result = Vec<Num>;

    fn init((graph, source_index): (G, VertexId), client: &mut C) -> Self {
        let source_route = Route {
            destination_index: source_index,
            total_distance: 0,
        };
        let mut pending_routes = BinaryHeap::new();
        pending_routes.push(source_route);

        let mut distances = vec![Num::MAX; graph.get_num_vertices()];
        distances[source_index] = 0;

        Self::highlight_source(source_index, client);

        Self {
            graph,
            distances,
            pending_routes,
        }
    }

    fn step(&mut self, client: &mut C) {
        let Some(route) = self.pending_routes.pop() else {
            return;
        };

        if route.total_distance != self.distances[route.destination_index] {
            return;
        }

        self.highlight_visited(route.destination_index, client);

        for edge in self.graph.get_edges(route.destination_index) {
            let neighbor_index = edge.end_id;
            let neighbor_distance = &mut self.distances[neighbor_index];

            let new_total_distance = route.total_distance + edge.props.distance();

            if new_total_distance < *neighbor_distance {
                *neighbor_distance = new_total_distance;

                let new_route = Route {
                    destination_index: neighbor_index,
                    total_distance: *neighbor_distance,
                };
                self.pending_routes.push(new_route);
                self.highlight_awaiting(new_route.destination_index, client);
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

#[derive(Clone, Copy, PartialEq, Eq)]
struct Route {
    destination_index: VertexId,
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
