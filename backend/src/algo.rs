pub trait Algo<I, E, R> {
    fn init(input: I) -> (Vec<E>, Self);
    fn step(&mut self) -> Vec<E>;
    fn result(&self) -> Option<R>;
}