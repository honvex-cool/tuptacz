pub trait InteractiveAlgo<I> {
    type Event;
    type Result;
    fn init(input: I) -> (Vec<Self::Event>, Self);
    fn step(&mut self) -> Vec<Self::Event>;
    fn result(&self) -> Option<Self::Result>;
}
