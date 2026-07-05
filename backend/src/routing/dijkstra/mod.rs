pub mod algos;
pub mod drivers;
pub mod policies;

use num_traits::Zero;

use crate::graphs::{EdgeView, Graph, VertexId};
use crate::routing::dijkstra::drivers::{Driver, PathTracker};
use crate::routing::dijkstra::policies::{Direction, DirectionPolicy, TerminationPolicy};
use crate::routing::presentation::{GraphAction, GraphEvent, HighlightMode};
use crate::routing::{Weight, Weighted};
use crate::utils::algo::{EventClient, InteractiveAlgo};
use crate::utils::pq::{self, GetIndex, Pq, SetIndex};

pub struct BidirectionalDrivenDijkstra<'g, G, D, H, DP, TP, T>
where
    G: Graph,
    G::E: Weighted,
    D: Driver<G::V, G::E>,
{
    graph: &'g G,
    bound: <G::E as Weighted>::Weight,
    meeting_vertex: Option<VertexId>,
    forward: Controller<G::V, G::E, D, H, T>,
    backward: Option<Controller<G::V, G::E, D, H, T>>,
    direction_policy: DP,
    termination_policy: TP,
}

pub type Queue<D, T> = Pq<VertexId, D, pq::Min, T>;

pub struct Controller<V, E, D, H, T>
where
    D: PathTracker<V, E>,
{
    pub search: Search<D>,
    pub heuristic: H,
    pub queue: Queue<D::Distance, T>,
}

pub struct Search<D> {
    pub id: VertexId,
    pub driver: D,
}

pub struct SearchResult<W, D> {
    pub forward: Search<D>,
    pub backward: Option<Search<D>>,
    pub bound: W,
    pub meeting_vertex: Option<VertexId>,
}

impl<'g, G, D, H, DP, TP, T> BidirectionalDrivenDijkstra<'g, G, D, H, DP, TP, T>
where
    G: Graph,
    G::E: Weighted,
    D: Driver<G::V, G::E>,
    T: SetIndex<VertexId> + GetIndex<VertexId>,
{
    pub fn new<C>(
        graph: &'g G,
        mut forward: Controller<G::V, G::E, D, H, T>,
        mut backward: Option<Controller<G::V, G::E, D, H, T>>,
        direction_policy: DP,
        termination_policy: TP,
        client: &mut C,
    ) -> Self
    where
        C: EventClient<GraphEvent<G::V, G::E>>,
    {
        let zero = D::Distance::zero();

        let forward_vertex = graph.get_vertex(forward.search.id);
        forward.queue.push(forward_vertex.id, zero);
        forward.search.driver.set_distance(forward_vertex, zero);
        Self::highlight_source(forward_vertex.id, client);

        if let Some(backward) = backward.as_mut() {
            let backward_vertex = graph.get_vertex(backward.search.id);
            backward.queue.push(backward_vertex.id, zero);
            backward.search.driver.set_distance(backward_vertex, zero);
        }

        Self {
            graph,
            bound: D::Distance::infinity(),
            meeting_vertex: None,
            forward,
            backward,
            direction_policy,
            termination_policy,
        }
    }

    fn highlight_source<C>(vertex_id: VertexId, client: &mut C)
    where
        C: EventClient<GraphEvent<G::V, G::E>>,
    {
        client.consume(GraphEvent {
            action: GraphAction::HighlightVertex {
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
            action: GraphAction::HighlightVertex {
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
            action: GraphAction::HighlightVertex {
                id: vertex_id,
                mode: HighlightMode::Awaiting,
            },
            comment: "Put vertex to queue".to_owned(),
        });
    }
}

impl<'g, G, D, H, DP, TP, T, C> InteractiveAlgo<C, GraphEvent<G::V, G::E>>
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
    type Result = SearchResult<D::Distance, D>;

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

        if total_distance != controller.search.driver.get_distance(vertex) {
            return true;
        }

        Self::highlight_visited(id, client);

        if !controller.search.driver.visit(vertex) {
            return false;
        }

        let handler = |edge| {
            Self::handle_edge(
                direction,
                edge,
                total_distance,
                &mut self.bound,
                &mut self.meeting_vertex,
                controller,
                other_controller.map(|c| c.search.driver.get_distance(edge.end)),
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
        SearchResult {
            forward: self.forward.search,
            backward: self.backward.map(|backward| backward.search),
            bound: self.bound,
            meeting_vertex: self.meeting_vertex,
        }
    }

    fn result_dyn(self: Box<Self>) -> Self::Result {
        SearchResult {
            forward: self.forward.search,
            backward: self.backward.map(|backward| backward.search),
            bound: self.bound,
            meeting_vertex: self.meeting_vertex,
        }
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
        meeting_vertex: &mut Option<VertexId>,
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

        if !controller.search.driver.should_consider_edge(edge) {
            return;
        }

        let neighbor = edge.end;
        let neighbor_distance = controller.search.driver.get_distance(neighbor);

        let edge_weight = edge.weight();
        let new_total_distance = total_distance + edge_weight;

        if new_total_distance < neighbor_distance
            && controller
                .search
                .driver
                .should_consider_vertex(neighbor, new_total_distance)
        {
            controller
                .search
                .driver
                .set_distance(neighbor, new_total_distance);

            controller.search.driver.set_predecessor(original_edge);

            controller
                .queue
                .push_or_update(neighbor.id, new_total_distance);

            Self::highlight_awaiting(neighbor.id, client);
        }

        if let Some(remaining_distance) = remaining_distance {
            let candidate_bound = controller.search.driver.get_distance(edge.start)
                + edge_weight
                + remaining_distance;
            *bound = D::Distance::min(*bound, candidate_bound);
            *meeting_vertex = Some(edge.end.id);
        }
    }
}
