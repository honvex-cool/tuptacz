pub trait InteractiveAlgo<C>
where
    C: EventClient<Self::Event>,
{
    type Input;
    type Event;
    type Result;

    fn init(input: Self::Input, client: &mut C) -> Self;
    fn step(&mut self, client: &mut C) -> bool;
    fn result(self) -> Self::Result;
}

pub trait EventClient<E> {
    fn consume(&mut self, event: E);
}

#[derive(Default)]
pub struct NullClient;

impl<E> EventClient<E> for NullClient {
    #[inline(always)]
    fn consume(&mut self, _event: E) {}
}

pub fn run_to_completion<A, C>(input: A::Input, client: &mut C) -> A::Result
where
    A: InteractiveAlgo<C>,
    C: EventClient<A::Event>,
{
    let mut algo = A::init(input, client);
    while algo.step(client) {}
    algo.result()
}
