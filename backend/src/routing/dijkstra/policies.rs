use crate::{
    graphs::{VertexId, VertexView},
    routing::Weight,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
}

pub trait DirectionPolicy<X> {
    fn pick_direction(&mut self, forward: X, backward: X) -> Direction;
}

#[derive(Default)]
pub struct AlwaysForward;

impl<X> DirectionPolicy<X> for AlwaysForward {
    fn pick_direction(&mut self, _forward: X, _backward: X) -> Direction {
        Direction::Forward
    }
}

#[derive(Default)]
pub struct Alternating(bool);

impl<X> DirectionPolicy<X> for Alternating {
    fn pick_direction(&mut self, _forward: X, _backward: X) -> Direction {
        let picked = if self.0 {
            Direction::Backward
        } else {
            Direction::Forward
        };
        self.0 = !self.0;
        picked
    }
}

pub trait TerminationPolicy<V, W> {
    fn should_terminate(
        &self,
        forward: (VertexView<V>, W),
        backward: (VertexView<V>, W),
        bound: W,
    ) -> bool;
}

#[derive(Default)]
pub struct NeverEarly;

impl<V, W> TerminationPolicy<V, W> for NeverEarly {
    fn should_terminate(
        &self,
        _forward: (VertexView<V>, W),
        _backward: (VertexView<V>, W),
        _bound: W,
    ) -> bool {
        false
    }
}

#[derive(Default)]
pub struct EndToEnd {
    pub source_id: VertexId,
    pub target_id: VertexId,
}

impl<V, W> TerminationPolicy<V, W> for EndToEnd {
    fn should_terminate(
        &self,
        (forward_vertex, _): (VertexView<V>, W),
        (backward_vertex, _): (VertexView<V>, W),
        _bound: W,
    ) -> bool {
        forward_vertex.id == self.target_id || backward_vertex.id == self.source_id
    }
}

#[derive(Default)]
pub struct BoundReachedJointly;

impl<V, W> TerminationPolicy<V, W> for BoundReachedJointly
where
    W: Weight,
{
    fn should_terminate(
        &self,
        (_, total_forwad_distance): (VertexView<V>, W),
        (_, total_backward_distance): (VertexView<V>, W),
        bound: W,
    ) -> bool {
        total_forwad_distance + total_backward_distance >= bound
    }
}

#[derive(Default)]
pub struct BoundReachedSeparately;

impl<V, W> TerminationPolicy<V, W> for BoundReachedSeparately
where
    W: Weight,
{
    fn should_terminate(
        &self,
        (_, total_forwad_distance): (VertexView<V>, W),
        (_, total_backward_distance): (VertexView<V>, W),
        bound: W,
    ) -> bool {
        total_forwad_distance >= bound && total_backward_distance >= bound
    }
}
