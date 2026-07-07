use std::cell::Cell;

use crate::graphs::{EdgeDescriptor, EdgeView, VertexView};
use crate::routing::{Weight, Weighted};
use crate::utils::staged::Staged;

pub trait VertexTracker<V, E> {
    type Distance: Weight;

    fn get_distance(&self) -> Self::Distance;
    fn set_distance(&mut self, distance: Self::Distance);

    fn get_predecessor(&self) -> Option<EdgeDescriptor>;
    fn set_predecessor(&mut self, edge: EdgeView<V, E>);
}

impl<V, E> VertexTracker<V, E> for (E::Weight, Option<EdgeDescriptor>)
where
    E: Weighted,
{
    type Distance = E::Weight;

    #[inline(always)]
    fn get_distance(&self) -> Self::Distance {
        self.0
    }

    #[inline(always)]
    fn set_distance(&mut self, distance: Self::Distance) {
        self.0 = distance;
    }

    #[inline(always)]
    fn get_predecessor(&self) -> Option<EdgeDescriptor> {
        self.1
    }

    #[inline(always)]
    fn set_predecessor(&mut self, edge: EdgeView<V, E>) {
        self.1 = Some(edge.descriptor());
    }
}

impl<V, E, T> VertexTracker<V, E> for Cell<T>
where
    T: VertexTracker<V, E> + Copy,
{
    type Distance = T::Distance;

    #[inline(always)]
    fn get_distance(&self) -> Self::Distance {
        self.get().get_distance()
    }

    #[inline(always)]
    fn set_distance(&mut self, distance: Self::Distance) {
        self.get_mut().set_distance(distance);
    }

    #[inline(always)]
    fn get_predecessor(&self) -> Option<EdgeDescriptor> {
        self.get().get_predecessor()
    }

    #[inline(always)]
    fn set_predecessor(&mut self, edge: EdgeView<V, E>) {
        self.get_mut().set_predecessor(edge);
    }
}

pub trait PathTracker<V, E> {
    type Distance: Weight;

    fn get_distance(&self, vertex: VertexView<V>) -> Self::Distance;
    fn set_distance(&mut self, vertex: VertexView<V>, distance: Self::Distance);

    fn get_predecessor(&self, vertex: VertexView<V>) -> Option<EdgeDescriptor>;
    fn set_predecessor(&mut self, edge: EdgeView<V, E>);
}

impl<V, E, T> PathTracker<V, E> for [T]
where
    T: VertexTracker<V, E>,
{
    type Distance = T::Distance;

    #[inline(always)]
    fn get_distance(&self, vertex: VertexView<V>) -> Self::Distance {
        self[vertex.id].get_distance()
    }

    #[inline(always)]
    fn set_distance(&mut self, vertex: VertexView<V>, distance: Self::Distance) {
        self[vertex.id].set_distance(distance);
    }

    #[inline(always)]
    fn get_predecessor(&self, vertex: VertexView<V>) -> Option<EdgeDescriptor> {
        self[vertex.id].get_predecessor()
    }

    #[inline(always)]
    fn set_predecessor(&mut self, edge: EdgeView<V, E>) {
        self[edge.end.id].set_predecessor(edge);
    }
}

pub trait Driver<V, E>: PathTracker<V, E, Distance = E::Weight>
where
    E: Weighted,
{
    #[inline(always)]
    fn should_consider_edge(&self, _edge: EdgeView<V, E>) -> bool {
        true
    }

    #[inline(always)]
    fn should_consider_vertex(&self, _vertex: VertexView<V>, _total_distance: E::Weight) -> bool {
        true
    }

    #[inline(always)]
    fn visit(&mut self, _vertex: VertexView<V>) -> bool {
        true
    }
}

impl<V, E, T> Driver<V, E> for [T]
where
    E: Weighted,
    [T]: PathTracker<V, E, Distance = E::Weight>,
{
}

impl<'a, V, E, T> PathTracker<V, E> for Staged<'a, T>
where
    T: VertexTracker<V, E> + Copy,
{
    type Distance = T::Distance;

    #[inline(always)]
    fn get_distance(&self, vertex: VertexView<V>) -> Self::Distance {
        self.get(vertex.id).get_distance()
    }

    #[inline(always)]
    fn set_distance(&mut self, vertex: VertexView<V>, distance: Self::Distance) {
        self.get_mut(vertex.id).set_distance(distance);
    }

    #[inline(always)]
    fn get_predecessor(&self, vertex: VertexView<V>) -> Option<EdgeDescriptor> {
        self.get(vertex.id).get_predecessor()
    }

    #[inline(always)]
    fn set_predecessor(&mut self, edge: EdgeView<V, E>) {
        self.get_mut(edge.end.id).set_predecessor(edge);
    }
}

impl<'a, V, E, T> Driver<V, E> for Staged<'a, T>
where
    E: Weighted,
    Staged<'a, T>: PathTracker<V, E, Distance = E::Weight>,
{
}

#[macro_export]
macro_rules! delegate_path_tracking {
    ($vertex_generic:ident, $edge_generic:ident, $distance_ty:ty, $inner:ident) => {
        type Distance = $distance_ty;

        #[inline(always)]
        fn get_distance(&self, vertex: $crate::graphs::VertexView<$vertex_generic>) -> Self::Distance {
            self.$inner.get_distance(vertex)
        }

        #[inline(always)]
        fn set_distance(&mut self, vertex: $crate::graphs::VertexView<$vertex_generic>, distance: Self::Distance) {
            self.$inner.set_distance(vertex, distance);
        }

        #[inline(always)]
        fn get_predecessor(&self, vertex: $crate::graphs::VertexView<$vertex_generic>) -> Option<$crate::graphs::EdgeDescriptor> {
            self.$inner.get_predecessor(vertex)
        }

        #[inline(always)]
        fn set_predecessor(&mut self, edge: $crate::graphs::EdgeView<$vertex_generic, $edge_generic>) {
            self.$inner.set_predecessor(edge);
        }
    };
}

pub struct LimitedDistanceDriver<D, W> {
    inner: D,
    limit: W,
}

impl<D, W> LimitedDistanceDriver<D, W>
where
    W: Weight,
{
    pub fn new(limit: W, inner: D) -> Self {
        Self { limit, inner }
    }

    #[inline(always)]
    fn is_good(&self, weight: W) -> bool {
        weight <= self.limit
    }
}

impl<V, E, D> PathTracker<V, E> for LimitedDistanceDriver<D, D::Distance>
where
    D: PathTracker<V, E>,
{
    delegate_path_tracking!(V, E, D::Distance, inner);
}

impl<V, E, D> Driver<V, E> for LimitedDistanceDriver<D, D::Distance>
where
    D: Driver<V, E>,
    E: Weighted,
{
    #[inline(always)]
    fn should_consider_edge(&self, edge: EdgeView<V, E>) -> bool {
        let hypothetical_weight = self.get_distance(edge.start) + edge.weight();
        self.is_good(hypothetical_weight) && self.inner.should_consider_edge(edge)
    }

    #[inline(always)]
    fn should_consider_vertex(&self, vertex: VertexView<V>, total_distance: E::Weight) -> bool {
        self.is_good(total_distance) && self.inner.should_consider_vertex(vertex, total_distance)
    }

    #[inline(always)]
    fn visit(&mut self, vertex: VertexView<V>) -> bool {
        self.inner.visit(vertex)
    }
}

pub struct LimitedVisitsDriver<D> {
    limit: usize,
    num_visits: usize,
    inner: D,
}

impl<D> LimitedVisitsDriver<D> {
    pub fn new(limit: usize, inner: D) -> Self {
        Self {
            limit,
            num_visits: 0,
            inner,
        }
    }
}

impl<V, E, D> PathTracker<V, E> for LimitedVisitsDriver<D>
where
    D: PathTracker<V, E>,
{
    delegate_path_tracking!(V, E, D::Distance, inner);
}

impl<V, E, D> Driver<V, E> for LimitedVisitsDriver<D>
where
    D: Driver<V, E>,
    E: Weighted,
{
    #[inline(always)]
    fn should_consider_edge(&self, edge: EdgeView<V, E>) -> bool {
        is_good(&self.inner, edge.end, self.num_visits, self.limit)
            && self.inner.should_consider_edge(edge)
    }

    #[inline(always)]
    fn should_consider_vertex(&self, vertex: VertexView<V>, total_distance: E::Weight) -> bool {
        is_good(&self.inner, vertex, self.num_visits, self.limit)
            && self.inner.should_consider_vertex(vertex, total_distance)
    }

    #[inline(always)]
    fn visit(&mut self, vertex: VertexView<V>) -> bool {
        self.num_visits += 1;
        self.num_visits < self.limit && self.inner.visit(vertex)
    }
}

#[inline(always)]
fn is_good<V, E, D>(driver: &D, vertex: VertexView<V>, num_visits: usize, limit: usize) -> bool
where
    D: PathTracker<V, E>,
{
    if num_visits == limit {
        driver.get_distance(vertex) != D::Distance::infinity()
    } else {
        num_visits < limit
    }
}
