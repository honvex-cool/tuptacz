use std::marker::PhantomData;

use crate::graphs::{EdgeView, VertexId, VertexView};
use crate::routing::{Weight, Weighted};

pub trait PathTracker<V, E> {
    type Distance: Weight;

    fn get_distance(&self, vertex: VertexView<V>) -> Self::Distance;
    fn set_distance(&mut self, vertex: VertexView<V>, distance: Self::Distance);
    fn set_predecessor(&mut self, edge: EdgeView<V, E>);
}

impl<V, E, W> PathTracker<V, E> for Vec<(W, VertexId)>
where
    W: Copy + Weight,
{
    type Distance = W;

    fn get_distance(&self, vertex: VertexView<V>) -> Self::Distance {
        self[vertex.id].0
    }

    fn set_distance(&mut self, vertex: VertexView<V>, distance: Self::Distance) {
        self[vertex.id].0 = distance;
    }

    fn set_predecessor(&mut self, edge: EdgeView<V, E>) {
        self[edge.end.id].1 = edge.start.id;
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

#[macro_export]
macro_rules! delegate_distance_tracking {
    ($type:ident<$vertex_generic:ident, $edge_generic:ident, $inner_generic:ident $(, $generic:ident)*>, $inner:ident) => {
        impl<$vertex_generic, $edge_generic, $inner_generic, $($generic),*> PathTracker<$vertex_generic, $edge_generic> for $type<$vertex_generic, $edge_generic, $inner_generic, $($generic),*>
        where
            $inner_generic: PathTracker<$vertex_generic, $edge_generic>,
        {
            type Distance = $inner_generic::Distance;

            #[inline(always)]
            fn get_distance(&self, vertex: VertexView<$vertex_generic>) -> Self::Distance {
                self.$inner.get_distance(vertex)
            }

            #[inline(always)]
            fn set_distance(&mut self, vertex: VertexView<$vertex_generic>, distance: Self::Distance) {
                self.$inner.set_distance(vertex, distance);
            }

            #[inline(always)]
            fn set_predecessor(&mut self, edge: EdgeView<$vertex_generic, $edge_generic>) {
                self.$inner.set_predecessor(edge);
            }
        }
    };
}

pub struct LimitedDistanceDriver<V, E, D>
where
    D: PathTracker<V, E>,
{
    inner: D,
    limit: D::Distance,
}

impl<V, E, D> LimitedDistanceDriver<V, E, D>
where
    D: PathTracker<V, E>,
{
    pub fn new(limit: D::Distance, inner: D) -> Self {
        Self { limit, inner }
    }

    #[inline(always)]
    fn is_good(&self, weight: D::Distance) -> bool {
        weight <= self.limit
    }
}

delegate_distance_tracking!(LimitedDistanceDriver<V, E, D>, inner);

impl<V, E, D> Driver<V, E> for LimitedDistanceDriver<V, E, D>
where
    D: Driver<V, E>,
    E: Weighted,
{
    #[inline(always)]
    fn should_consider_edge(&self, edge: EdgeView<V, E>) -> bool {
        let hypothetical_weight = self.get_distance(edge.start) + edge.weight();
        self.is_good(hypothetical_weight)
    }

    #[inline(always)]
    fn should_consider_vertex(&self, _vertex: VertexView<V>, total_distance: E::Weight) -> bool {
        self.is_good(total_distance)
    }

    #[inline(always)]
    fn visit(&mut self, vertex: VertexView<V>) -> bool {
        self.inner.visit(vertex)
    }
}

pub struct LimitedVisitsDriver<V, E, D>
where
    D: PathTracker<V, E>,
{
    limit: usize,
    num_visits: usize,
    inner: D,
    _phantom: PhantomData<(V, E)>,
}

impl<V, E, D> LimitedVisitsDriver<V, E, D>
where
    D: PathTracker<V, E>,
{
    pub fn new(limit: usize, inner: D) -> Self {
        Self {
            limit,
            num_visits: 0,
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<V, E, D> LimitedVisitsDriver<V, E, D>
where
    D: PathTracker<V, E>,
{
    #[inline(always)]
    fn is_good(&self, vertex: VertexView<V>) -> bool {
        if self.num_visits == self.limit {
            self.get_distance(vertex).is_finite()
        } else {
            self.num_visits < self.limit
        }
    }
}

delegate_distance_tracking!(LimitedVisitsDriver<V, E, D>, inner);

impl<V, E, D> Driver<V, E> for LimitedVisitsDriver<V, E, D>
where
    D: Driver<V, E>,
    E: Weighted,
{
    #[inline(always)]
    fn should_consider_edge(&self, edge: EdgeView<V, E>) -> bool {
        self.is_good(edge.end)
    }

    #[inline(always)]
    fn should_consider_vertex(&self, vertex: VertexView<V>, total_distance: E::Weight) -> bool {
        self.is_good(vertex) && self.inner.should_consider_vertex(vertex, total_distance)
    }

    #[inline(always)]
    fn visit(&mut self, vertex: VertexView<V>) -> bool {
        self.num_visits += 1;
        self.num_visits < self.limit && self.inner.visit(vertex)
    }
}
