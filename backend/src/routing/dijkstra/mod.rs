pub mod algos;
pub mod drivers;
pub mod heuristics;
pub mod policies;

use num_traits::Zero;

use crate::graphs::{EdgeView, Graph, VertexId, VertexView};
use crate::routing::dijkstra::drivers::{Driver, PathTracker};
use crate::routing::dijkstra::heuristics::Heuristic;
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
    num_settled_vertices: usize,
    num_inspected_edges: usize,
}

pub type Queue<D, T> = Pq<VertexId, (D, D), pq::Min, T>;

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
    pub meeting_id: Option<VertexId>,
}

impl<'g, G, D, H, DP, TP, T> BidirectionalDrivenDijkstra<'g, G, D, H, DP, TP, T>
where
    G: Graph,
    G::E: Weighted,
    D: Driver<G::V, G::E>,
    H: Heuristic<G::V, D::Distance>,
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
        let forward_key = (forward.heuristic.calculate(forward_vertex), zero);
        forward.queue.push(forward_vertex.id, forward_key);
        forward.search.driver.set_distance(forward_vertex, zero);
        Self::highlight_source(forward_vertex, client);

        if let Some(backward) = backward.as_mut() {
            let backward_vertex = graph.get_vertex(backward.search.id);
            let backward_key = (backward.heuristic.calculate(backward_vertex), zero);
            backward.queue.push(backward_vertex.id, backward_key);
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
            num_settled_vertices: 0,
            num_inspected_edges: 0,
        }
    }

    #[inline(always)]
    fn highlight_source<C>(vertex: VertexView<G::V>, client: &mut C)
    where
        C: EventClient<GraphEvent<G::V, G::E>>,
    {
        client.consume(GraphEvent {
            action: GraphAction::HighlightVertex {
                vertex: vertex.detach(),
                mode: HighlightMode::Source,
            },
            comment: "Starting from vertex".to_owned(),
        });
    }

    #[inline(always)]
    fn highlight_visited<C>(vertex: VertexView<G::V>, client: &mut C)
    where
        C: EventClient<GraphEvent<G::V, G::E>>,
    {
        client.consume(GraphEvent {
            action: GraphAction::HighlightVertex {
                vertex: vertex.detach(),
                mode: HighlightMode::Visited,
            },
            comment: "Visited vertex".to_owned(),
        });
    }

    #[inline(always)]
    fn highlight_awaiting<C>(vertex: VertexView<G::V>, client: &mut C)
    where
        C: EventClient<GraphEvent<G::V, G::E>>,
    {
        client.consume(GraphEvent {
            action: GraphAction::HighlightVertex {
                vertex: vertex.detach(),
                mode: HighlightMode::Awaiting,
            },
            comment: "Put vertex to queue".to_owned(),
        });
    }

    #[inline(always)]
    fn emit_query_summary<C>(&self, client: &mut C)
    where
        C: EventClient<GraphEvent<G::V, G::E>>,
    {
        client.consume(GraphEvent {
            action: GraphAction::QuerySummary {
                num_settled_vertices: self.num_settled_vertices,
                num_inspected_edges: self.num_inspected_edges,
            },
            comment: "Summary of the query phase".to_owned(),
        });
    }
}

impl<'g, G, D, H, DP, TP, T, C> InteractiveAlgo<C, GraphEvent<G::V, G::E>>
    for BidirectionalDrivenDijkstra<'g, G, D, H, DP, TP, T>
where
    G: Graph,
    G::E: Weighted,
    D: Driver<G::V, G::E>,
    H: Heuristic<G::V, D::Distance>,
    C: EventClient<GraphEvent<G::V, G::E>>,
    DP: DirectionPolicy<D::Distance>,
    TP: TerminationPolicy<G::V, D::Distance>,
    T: SetIndex<VertexId> + GetIndex<VertexId>,
{
    type Result = SearchResult<D::Distance, D>;

    fn step(&mut self, client: &mut C) -> bool {
        let direction = if let Some(backward) = self.backward.as_ref() {
            if let (
                Some((&forward_id, &forward_distance)),
                Some((&backward_id, &backward_distance)),
            ) = (self.forward.queue.peek(), backward.queue.peek())
            {
                let forward = (self.graph.get_vertex(forward_id), forward_distance);
                let backward = (self.graph.get_vertex(backward_id), backward_distance);

                if self
                    .termination_policy
                    .should_terminate(forward, backward, self.bound)
                {
                    return false;
                }
                self.direction_policy
                    .pick_direction(forward_distance.0, backward_distance.0)
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

        let (id, (_, total_distance)) = controller.queue.pop().unwrap();

        let vertex = self.graph.get_vertex(id);

        self.num_settled_vertices += 1;

        if total_distance != controller.search.driver.get_distance(vertex) {
            return true;
        }

        Self::highlight_visited(vertex, client);

        if !controller.search.driver.visit(vertex) {
            return false;
        }

        let handler = |edge: EdgeView<'_, _, _>| {
            let edge_end = match direction {
                Direction::Forward => edge.end,
                Direction::Backward => edge.start,
            };
            Self::handle_edge(
                direction,
                edge,
                total_distance,
                &mut self.bound,
                &mut self.meeting_vertex,
                controller,
                other_controller.map(|c| c.search.driver.get_distance(edge_end)),
                &mut self.num_inspected_edges,
                client,
            )
        };

        match direction {
            Direction::Forward => self.graph.iter_outgoing_edges(id).for_each(handler),
            Direction::Backward => self.graph.iter_incoming_edges(id).for_each(handler),
        }

        true
    }

    fn result(self, client: &mut C) -> Self::Result {
        self.emit_query_summary(client);
        SearchResult {
            forward: self.forward.search,
            backward: self.backward.map(|backward| backward.search),
            bound: self.bound,
            meeting_id: self.meeting_vertex,
        }
    }

    fn result_dyn(self: Box<Self>, client: &mut C) -> Self::Result {
        self.result(client)
    }
}

impl<'g, G, D, H, DP, TP, T> BidirectionalDrivenDijkstra<'g, G, D, H, DP, TP, T>
where
    G: Graph,
    G::E: Weighted,
    D: Driver<G::V, G::E>,
    H: Heuristic<G::V, D::Distance>,
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
        num_inspected_edges: &mut usize,
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
        };

        *num_inspected_edges += 1;

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

            controller.search.driver.set_predecessor(edge);

            let estimate = new_total_distance + controller.heuristic.calculate(neighbor);
            let key = (estimate, new_total_distance);
            controller.queue.push_or_update(neighbor.id, key);

            Self::highlight_awaiting(neighbor, client);
        }

        if let Some(remaining_distance) = remaining_distance {
            let candidate_bound = controller.search.driver.get_distance(edge.start)
                + edge_weight
                + remaining_distance;
            if candidate_bound < *bound {
                *bound = candidate_bound;
                *meeting_vertex = Some(edge.end.id);
            }
        }
    }
}
