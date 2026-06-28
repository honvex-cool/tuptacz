use std::ops::{Add, AddAssign};

pub mod ch;
pub mod dijkstra;

pub trait Weight: Copy + Ord + Add<Output = Self> + AddAssign {
    fn zero() -> Self;
    fn infinity() -> Self;

    fn is_finite(&self) -> bool;
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
