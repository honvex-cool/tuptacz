pub mod ch;
pub mod dijkstra;

pub type Num = i64;

pub trait Distance {
    fn distance(&self) -> Num;
}

impl Distance for Num {
    fn distance(&self) -> Num {
        *self
    }
}
