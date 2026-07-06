pub trait InteractiveAlgo<C, E>
where
    C: EventClient<E>,
{
    type Result;

    fn step(&mut self, client: &mut C) -> bool;

    fn result(self) -> Self::Result;
    fn result_dyn(self: Box<Self>) -> Self::Result;
}

pub fn map<A, C, E, F, R>(inner: A, map: F) -> impl InteractiveAlgo<C, E, Result = R>
where
    A: InteractiveAlgo<C, E>,
    C: EventClient<E>,
    F: FnOnce(A::Result) -> R,
{
    Map { inner, map }
}

pub struct Map<A, F> {
    inner: A,
    map: F,
}

impl<A, C, E, F, R> InteractiveAlgo<C, E> for Map<A, F>
where
    A: InteractiveAlgo<C, E>,
    C: EventClient<E>,
    F: FnOnce(A::Result) -> R,
{
    type Result = R;

    #[inline(always)]
    fn step(&mut self, client: &mut C) -> bool {
        self.inner.step(client)
    }

    #[inline(always)]
    fn result(self) -> Self::Result {
        (self.map)(self.inner.result())
    }

    #[inline(always)]
    fn result_dyn(self: Box<Self>) -> Self::Result {
        (self.map)(self.inner.result())
    }
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

#[inline(always)]
pub fn complete<A, C, E>(algo: &mut A, client: &mut C)
where
    A: InteractiveAlgo<C, E>,
    C: EventClient<E>,
{
    while algo.step(client) {}
}

#[inline(always)]
pub fn complete_dyn<C, E, Result>(
    algo: &mut dyn InteractiveAlgo<C, E, Result = Result>,
    client: &mut C,
) where
    C: EventClient<E>,
{
    while algo.step(client) {}
}

pub trait QueryEngine<C, E>
where
    C: EventClient<E>,
{
    type Input;
    type Result;

    fn query<'a>(
        &'a mut self,
        query: Self::Input,
        client: &mut C,
    ) -> Box<dyn InteractiveAlgo<C, E, Result = Self::Result> + 'a>;
}
