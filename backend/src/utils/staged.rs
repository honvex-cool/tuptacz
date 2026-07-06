use std::cell::Cell;

pub type Epoch = usize;

pub const STARTING_EPOCH: Epoch = 0;

pub struct Staged<'a, T> {
    default: T,
    epoch: Epoch,
    time_stamps: &'a [Cell<Epoch>],
    items: &'a mut [Cell<T>],
}

impl<'a, T> Staged<'a, T> {
    pub fn new_with_default(
        epoch: Epoch,
        time_stamps: &'a [Cell<Epoch>],
        items: &'a mut [Cell<T>],
        default: T,
    ) -> Self {
        Self {
            epoch,
            default,
            time_stamps,
            items,
        }
    }
}

impl<'a, T> Staged<'a, T>
where
    T: Default,
{
    pub fn new(epoch: Epoch, time_stamps: &'a [Cell<Epoch>], items: &'a mut [Cell<T>]) -> Self {
        Self::new_with_default(epoch, time_stamps, items, T::default())
    }
}

impl<'a, T> Staged<'a, T>
where
    T: Copy,
{
    pub fn get(&self, index: usize) -> T {
        let time_stamp_cell = &self.time_stamps[index];
        let item_cell = &self.items[index];
        if time_stamp_cell.get() < self.epoch {
            item_cell.set(self.default);
            time_stamp_cell.set(self.epoch);
        }
        item_cell.get()
    }

    pub fn set(&mut self, index: usize, value: T) {
        self.time_stamps[index].set(self.epoch);
        self.items[index].set(value);
    }
}
