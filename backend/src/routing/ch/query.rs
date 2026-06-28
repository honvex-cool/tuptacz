use std::marker::PhantomData;

use crate::{
    algo::InteractiveAlgo, graphs::{EdgeView, Graph, VertexView}, routing::{
        Weight, Weighted,
        ch::Rank,
        dijkstra::{
            BidirectionalDrivenDijkstra,
            drivers::{Driver, PathTracker},
            policies::{Alternating, BoundReachedSeparately},
        },
    }, utils::pq::NullTracker,
};

pub struct Query<'q, G>
where
    G: Graph,
    G::E: Weighted,
{
    inner_dijkstra: BidirectionalDrivenDijkstra<
        'q,
        G,
        RankBasedDriver<'q, G::V, G::E, <G::E as Weighted>::Weight>,
        (),
        Alternating,
        BoundReachedSeparately,
        NullTracker,
    >,
}

struct RankBasedDriver<'r, V, E, W> {
    ranks: &'r [Rank],
    distances: Vec<W>,
    _phantom: PhantomData<(V, E)>,
}

impl<'r, V, E, W> PathTracker<V, E> for RankBasedDriver<'r, V, E, W>
where
    W: Weight,
{
    type Distance = W;

    fn get_distance(&self, vertex: VertexView<V>) -> Self::Distance {
        self.distances[vertex.id]
    }

    fn set_distance(&mut self, vertex: VertexView<V>, distance: Self::Distance) {
        self.distances[vertex.id] = distance;
    }

    fn set_predecessor(&mut self, _edge: EdgeView<V, E>) {}
}

impl<'r, V, E> Driver<V, E> for RankBasedDriver<'r, V, E, E::Weight>
where
    E: Weighted,
{
    fn should_consider_edge(&self, edge: EdgeView<V, E>) -> bool {
        self.ranks[edge.start.id] < self.ranks[edge.end.id]
    }
}
