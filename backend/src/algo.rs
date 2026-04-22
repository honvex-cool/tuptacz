use std::process::Output;

pub trait InteractiveAlgo<I, C>
{
    type Result;
    fn init(input: I, client: C) -> impl Future<Output=Self>;
    fn step(&mut self) -> impl Future<Output=()>;
    fn result(&self) -> Option<Self::Result>;
}
