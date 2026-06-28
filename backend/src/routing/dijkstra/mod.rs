pub mod drivers;
pub mod policies;

use crate::algo::{EventClient, InteractiveAlgo};
use crate::graphs::{EdgeView, Graph, VertexId, VertexView};
use crate::presentation::{GraphEvent, HighlightMode, ServerAction};
use crate::routing::dijkstra::drivers::{Driver, PathTracker};
use crate::routing::dijkstra::policies::{Direction, DirectionPolicy, TerminationPolicy};
use crate::routing::{Weight, Weighted};
use crate::utils::pq::{self, GetIndex, Pq, SetIndex};

pub struct BidirectionalDrivenDijkstra<'g, G, D, H, DP, TP, T>
where
    G: Graph,
    G::E: Weighted,
    D: Driver<G::V, G::E>,
{
    graph: &'g G,
    bound: <G::E as Weighted>::Weight,
    forward: Controller<'g, G::V, G::E, D, H, T>,
    backward: Option<Controller<'g, G::V, G::E, D, H, T>>,
    direction_policy: DP,
    termination_policy: TP,
}

pub type Queue<D, T> = Pq<VertexId, D, pq::Min, T>;

pub struct Controller<'g, V, E, D, H, T>
where
    D: PathTracker<V, E>,
{
    pub vertex: VertexView<'g, V>,
    pub driver: &'g mut D,
    pub heuristic: H,
    pub queue: Queue<D::Distance, T>,
}

impl<'g, G, D, H, DP, TP, T, C> InteractiveAlgo<C>
    for BidirectionalDrivenDijkstra<'g, G, D, H, DP, TP, T>
where
    G: Graph,
    G::E: Weighted,
    D: Driver<G::V, G::E>,
    C: EventClient<GraphEvent<G::V, G::E>>,
    DP: DirectionPolicy<D::Distance>,
    TP: TerminationPolicy<G::V, D::Distance>,
    T: SetIndex<VertexId> + GetIndex<VertexId>,
{
    type Input = (
        &'g G,
        Controller<'g, G::V, G::E, D, H, T>,
        Option<Controller<'g, G::V, G::E, D, H, T>>,
        DP,
        TP,
    );
    type Event = GraphEvent<G::V, G::E>;
    type Result = D::Distance;

    fn init(
        (graph, mut forward, mut backward, direction_policy, termination_policy): Self::Input,
        client: &mut C,
    ) -> Self {
        let zero = D::Distance::zero();

        forward.queue.push(forward.vertex.id, zero);
        forward.driver.set_distance(forward.vertex, zero);
        Self::highlight_source(forward.vertex.id, client);

        if let Some(backward) = backward.as_mut() {
            backward.queue.push(backward.vertex.id, zero);
            backward.driver.set_distance(backward.vertex, zero);
        }

        Self {
            graph,
            bound: D::Distance::infinity(),
            forward,
            backward,
            direction_policy,
            termination_policy,
        }
    }

    fn step(&mut self, client: &mut C) -> bool {
        let direction = if let Some(backward) = self.backward.as_ref() {
            if let (
                Some((&forward_id, &total_forward_distance)),
                Some((&backward_id, &total_backward_distance)),
            ) = (self.forward.queue.peek(), backward.queue.peek())
            {
                let forward = (self.graph.get_vertex(forward_id), total_forward_distance);
                let backward = (self.graph.get_vertex(backward_id), total_backward_distance);

                if self
                    .termination_policy
                    .should_terminate(forward, backward, self.bound)
                {
                    return false;
                }
                self.direction_policy
                    .pick_direction(total_forward_distance, total_backward_distance)
            } else {
                return false;
            }
        } else if !self.forward.queue.is_empty() {
            Direction::Forward
        } else {
            return false;
        };

        let (controller, other_controller) = match direction {
            Direction::Forward => (&mut self.forward, self.backward.as_ref()),
            Direction::Backward => (self.backward.as_mut().unwrap(), Some(&self.forward)),
        };

        let (id, total_distance) = controller.queue.pop().unwrap();

        let vertex = self.graph.get_vertex(id);

        if total_distance != controller.driver.get_distance(vertex) {
            return true;
        }

        Self::highlight_visited(id, client);

        if !controller.driver.visit(vertex) {
            return false;
        }

        let handler = |edge| {
            Self::handle_edge(
                direction,
                edge,
                total_distance,
                &mut self.bound,
                controller,
                other_controller.map(|c| c.driver.get_distance(edge.end)),
                client,
            )
        };

        match direction {
            Direction::Forward => self.graph.iter_outgoing_edges(id).for_each(handler),
            Direction::Backward => self.graph.iter_incoming_edges(id).for_each(handler),
        }

        true
    }

    fn result(self) -> Self::Result {
        self.bound
    }
}

impl<'g, G, D, H, DP, TP, T> BidirectionalDrivenDijkstra<'g, G, D, H, DP, TP, T>
where
    G: Graph,
    G::E: Weighted,
    D: Driver<G::V, G::E>,
    T: SetIndex<VertexId> + GetIndex<VertexId>,
{
    fn handle_edge<C>(
        direction: Direction,
        original_edge: EdgeView<G::V, G::E>,
        total_distance: D::Distance,
        bound: &mut D::Distance,
        controller: &mut Controller<G::V, G::E, D, H, T>,
        remaining_distance: Option<D::Distance>,
        client: &mut C,
    ) where
        C: EventClient<GraphEvent<G::V, G::E>>,
        T: SetIndex<VertexId> + GetIndex<VertexId>,
    {
        let edge = match direction {
            Direction::Forward => original_edge,
            Direction::Backward => original_edge.flip(),
        };

        if !controller.driver.should_consider_edge(edge) {
            return;
        }

        let neighbor = edge.end;
        let neighbor_distance = controller.driver.get_distance(neighbor);

        let edge_weight = edge.weight();
        let new_total_distance = total_distance + edge_weight;

        if new_total_distance < neighbor_distance
            && controller
                .driver
                .should_consider_vertex(neighbor, new_total_distance)
        {
            controller.driver.set_distance(neighbor, new_total_distance);

            controller.driver.set_predecessor(original_edge);

            controller
                .queue
                .push_or_update(neighbor.id, new_total_distance);

            Self::highlight_awaiting(neighbor.id, client);
        }

        if let Some(remaining_distance) = remaining_distance {
            let candidate_bound =
                controller.driver.get_distance(edge.start) + edge_weight + remaining_distance;
            *bound = D::Distance::min(*bound, candidate_bound);
        }
    }
}

impl<'g, G, D, H, DP, TP, T> BidirectionalDrivenDijkstra<'g, G, D, H, DP, TP, T>
where
    G: Graph,
    G::E: Weighted,
    D: Driver<G::V, G::E>,
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

    fn highlight_visited<C>(vertex_id: VertexId, client: &mut C)
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

    fn highlight_awaiting<C>(vertex_id: VertexId, client: &mut C)
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
