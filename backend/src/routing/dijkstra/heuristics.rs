use crate::{graphs::VertexView, routing::Weight};

pub trait Heuristic<V, W> {
    fn calculate(&self, vertex: VertexView<V>) -> W;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ZeroHeuristic;

impl<V, W> Heuristic<V, W> for ZeroHeuristic where W: Weight{
    #[inline(always)]
    fn calculate(&self, _vertex: VertexView<V>) -> W {
        W::zero()
    }
}
