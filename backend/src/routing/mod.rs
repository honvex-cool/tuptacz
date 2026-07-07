pub mod a_star;
pub mod ch;
pub mod dijkstra;
pub mod model;
pub mod osm;
pub mod pathfinding;
pub mod presentation;

use std::{
    fmt::{Debug, Display},
    ops::Add,
};

use num_traits::{Float, Zero};

use crate::{
    graphs::{EdgeDescriptor, Path, VertexId},
    routing::presentation::GraphEvent,
    utils::{
        algo::{EventClient, InteractiveAlgo, QueryEngine},
        staged::{Stageable, Staged},
    },
};

pub trait Weight: Debug + Display + Copy + Ord + Zero + Add<Output = Self> {
    fn infinity() -> Self;
}

pub trait Weighted: From<Self::Weight> {
    type Weight: Weight;

    fn weight(&self) -> Self::Weight;
}

impl<W> Weighted for W
where
    W: Weight + Copy,
{
    type Weight = Self;

    fn weight(&self) -> Self::Weight {
        *self
    }
}

impl<F> Weight for F
where
    F: Float + Ord + Debug + Display,
{
    fn infinity() -> Self {
        <Self as Float>::infinity()
    }
}

pub type Pathfinder<'a, V, E, C> = dyn QueryEngine<C, GraphEvent<V, E>, Input = (VertexId, VertexId), Result = Option<Path<V, E>>>
    + 'a;
pub type RoutingAlgo<'a, V, E, C> =
    dyn InteractiveAlgo<C, GraphEvent<V, E>, Result = Box<Pathfinder<'a, V, E, C>>>;

pub struct NoPreprocessing<'a, V, E, C>(Box<Pathfinder<'a, V, E, C>>);

impl<'a, V, E, C> InteractiveAlgo<C, GraphEvent<V, E>> for NoPreprocessing<'a, V, E, C>
where
    C: EventClient<GraphEvent<V, E>>,
{
    type Result = Box<Pathfinder<'a, V, E, C>>;

    fn step(&mut self, _client: &mut C) -> bool {
        false
    }

    #[inline(always)]
    fn result(self, _client: &mut C) -> Self::Result {
        self.0
    }

    #[inline(always)]
    fn result_dyn(self: Box<Self>, client: &mut C) -> Self::Result {
        self.result(client)
    }
}

type BasicVertexData<W> = (W, Option<EdgeDescriptor>);

pub struct BasicVertexDataArray<W> {
    forward: Stageable<BasicVertexData<W>>,
    backward: Stageable<BasicVertexData<W>>,
}

impl<W> BasicVertexDataArray<W>
where
    W: Weight,
{
    pub fn with_size(num_vertices: usize) -> Self {
        let default = Self::default();
        Self {
            forward: Stageable::new_with_default(num_vertices, default),
            backward: Stageable::new_with_default(num_vertices, default),
        }
    }

    pub fn stage(
        &mut self,
    ) -> (
        Staged<'_, BasicVertexData<W>>,
        Staged<'_, BasicVertexData<W>>,
    ) {
        (self.forward.stage(), self.backward.stage())
    }

    fn default() -> BasicVertexData<W> {
        (W::infinity(), None)
    }
}
