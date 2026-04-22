pub trait InteractiveAlgo<I, E, C>
where C: EventClient<E>
{
    type Result;
    fn init(input: I, client: &mut C) -> Self;
    fn step(&mut self, client: &mut C);
    fn result(&self) -> Option<Self::Result>;
}

pub trait EventClient<E> {
    fn consume(&mut self, event: E);
}
