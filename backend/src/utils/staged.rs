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
        self.refresh(index)
    }

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        self.refresh(index);
        self.items[index].get_mut()
    }

    fn refresh(&self, index: usize) -> T {
        let time_stamp_cell = &self.time_stamps[index];
        let item_cell = &self.items[index];
        if time_stamp_cell.get() < self.epoch {
            item_cell.set(self.default);
            time_stamp_cell.set(self.epoch);
        }
        item_cell.get()
    }
}

pub struct Stageable<T> {
    default: T,
    current_epoch: Epoch,
    time_stamps: Vec<Cell<Epoch>>,
    items: Vec<Cell<T>>,
}

impl<T> Stageable<T> where T: Copy + Default {
    pub fn new(size: usize) -> Self {
        Self::new_with_default(size, T::default())
    }
}

impl<T> Stageable<T>
where
    T: Copy,
{
    pub fn new_with_default(size: usize, default: T) -> Self {
        Self {
            default,
            current_epoch: STARTING_EPOCH,
            time_stamps: vec![Cell::new(STARTING_EPOCH); size],
            items: vec![Cell::new(default); size],
        }
    }

    pub fn stage(&mut self) -> Staged<'_, T> {
        self.current_epoch += 1;
        Staged::new_with_default(
            self.current_epoch,
            &self.time_stamps,
            &mut self.items,
            self.default,
        )
    }
}
