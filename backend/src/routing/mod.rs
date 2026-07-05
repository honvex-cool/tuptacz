pub mod ch;
pub mod dijkstra;
pub mod model;
pub mod osm;
pub mod pathfinding;
pub mod presentation;

use std::{cell::Cell, ops::Add};

use num_traits::{Float, Zero};

use crate::{
    graphs::{EdgeDescriptor, Path, VertexId}, routing::presentation::GraphEvent, utils::{
        algo::{EventClient, InteractiveAlgo, QueryEngine}, staged::{Epoch, STARTING_EPOCH, Staged},
    },
};

pub trait Weight: Copy + Ord + Zero + Add<Output = Self> {
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
    F: Float + Ord,
{
    fn infinity() -> Self {
        <Self as Float>::infinity()
    }
}

pub type Pathfinder<V, E, C> =
    dyn QueryEngine<C, GraphEvent<V, E>, Input = (VertexId, VertexId), Result = Option<Path<V, E>>>;
pub type RoutingAlgo<V, E, C> =
    dyn InteractiveAlgo<C, GraphEvent<V, E>, Result = Box<Pathfinder<V, E, C>>>;

pub struct NoPreprocessing<Q>(Q);

impl<Q, C, E> InteractiveAlgo<C, E> for NoPreprocessing<Q>
where
    C: EventClient<E>,
{
    type Result = Q;

    fn step(&mut self, _client: &mut C) -> bool {
        false
    }

    fn result(self) -> Self::Result {
        self.0
    }

    fn result_dyn(self: Box<Self>) -> Self::Result {
        self.0
    }
}

type BasicVertexData<W> = (W, Option<EdgeDescriptor>);

pub struct BasicVertexDataArray<W> {
    epoch: Epoch,
    time_stamps: Vec<Cell<Epoch>>,
    forward_data: Vec<Cell<BasicVertexData<W>>>,
    backward_data: Vec<Cell<BasicVertexData<W>>>,
}

impl<W> BasicVertexDataArray<W>
where
    W: Weight,
{
    pub fn with_size(num_vertices: usize) -> Self {
        let default = Self::default();
        Self {
            epoch: STARTING_EPOCH,
            time_stamps: vec![Cell::new(STARTING_EPOCH); num_vertices],
            forward_data: vec![Cell::new(default); num_vertices],
            backward_data: vec![Cell::new(default); num_vertices],
        }
    }

    pub fn stage(
        &mut self,
    ) -> (
        Staged<'_, BasicVertexData<W>>,
        Staged<'_, BasicVertexData<W>>,
    ) {
        self.epoch += 1;

        let default = Self::default();

        let forward_staged = Staged::new_with_default(
            self.epoch,
            &self.time_stamps,
            &mut self.forward_data,
            default,
        );
        let backward_staged = Staged::new_with_default(
            self.epoch,
            &self.time_stamps,
            &mut self.backward_data,
            default,
        );

        (forward_staged, backward_staged)
    }

    fn default() -> BasicVertexData<W> {
        (W::infinity(), None)
    }
}
