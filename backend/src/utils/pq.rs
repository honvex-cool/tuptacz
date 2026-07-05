use std::marker::PhantomData;

use sealed::sealed;

pub trait SetIndex<V> {
    fn set_index(&mut self, value: &V, index: Option<usize>);

    fn put_index(&mut self, value: &V, index: usize) {
        self.set_index(value, Some(index))
    }

    fn reset_index(&mut self, value: &V) {
        self.set_index(value, None);
    }
}

pub trait GetIndex<V> {
    fn get_index(&self, value: &V) -> Option<usize>;
}

// This tracker consumes the indices but does not remember them, effectively allowing for duplicates
#[derive(Default)]
pub struct NullTracker;

impl<V> SetIndex<V> for NullTracker {
    #[inline(always)]
    fn set_index(&mut self, _value: &V, _index: Option<usize>) {}
}

impl<V> GetIndex<V> for NullTracker {
    #[inline(always)]
    fn get_index(&self, _value: &V) -> Option<usize> {
        None
    }
}

impl<V> SetIndex<V> for Vec<Option<usize>>
where
    V: Copy + Into<usize>,
{
    fn set_index(&mut self, value: &V, index: Option<usize>) {
        self[(*value).into()] = index;
    }
}

impl<V> GetIndex<V> for Vec<Option<usize>>
where
    V: Copy + Into<usize>,
{
    fn get_index(&self, value: &V) -> Option<usize> {
        self[(*value).into()]
    }
}

#[sealed]
pub trait KeyOrder<K> {
    fn is_better(a: &K, b: &K) -> bool;
}

pub struct Min;

#[sealed]
impl<K> KeyOrder<K> for Min
where
    K: Ord,
{
    fn is_better(a: &K, b: &K) -> bool {
        a < b
    }
}

pub struct Max;

#[sealed]
impl<K> KeyOrder<K> for Max
where
    K: Ord,
{
    fn is_better(a: &K, b: &K) -> bool {
        a > b
    }
}

pub struct Pq<V, K, M, T = NullTracker> {
    keys: Vec<K>,
    values: Vec<V>,
    index_tracker: T,
    _phantom: PhantomData<M>,
}

impl<V, K, M, T> Default for Pq<V, K, M, T>
where
    T: Default,
{
    fn default() -> Self {
        Self::with_index_tracker(T::default())
    }
}

impl<V, K, M, T> Pq<V, K, M, T> {
    pub fn with_index_tracker(index_tracker: T) -> Self {
        Self {
            keys: vec![],
            values: vec![],
            index_tracker,
            _phantom: PhantomData,
        }
    }
}

impl<V, K, M, T> Pq<V, K, M, T>
where
    M: KeyOrder<K>,
    T: SetIndex<V>,
{
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push(&mut self, value: V, key: K) {
        let index = self.len();

        self.keys.push(key);
        self.values.push(value);

        self.up_heap(index);
    }

    pub fn peek(&self) -> Option<(&V, &K)> {
        Option::zip(self.values.first(), self.keys.first())
    }

    pub fn pop(&mut self) -> Option<(V, K)> {
        self.pop_at(0)
    }

    fn pop_at(&mut self, index: usize) -> Option<(V, K)> {
        let len = self.len();

        if len > 0 {
            self.swap(index, len - 1);
        }

        let key = self.keys.pop()?;
        let value = self.values.pop()?;

        self.index_tracker.reset_index(&value);

        self.down_heap(index);

        Some((value, key))
    }

    fn up_heap(&mut self, mut index: usize) {
        while index != 0 {
            let parent_index = (index - 1) / 2;

            if !M::is_better(&self.keys[index], &self.keys[parent_index]) {
                break;
            }

            self.swap(index, parent_index);
            index = parent_index;
        }
    }

    fn down_heap(&mut self, mut index: usize) {
        let len = self.len();

        loop {
            let left_child_index = 2 * index + 1;
            if left_child_index >= len {
                break;
            }

            let right_child_index = left_child_index + 1;

            let better_child_index = if right_child_index >= len
                || !M::is_better(&self.keys[right_child_index], &self.keys[left_child_index])
            {
                left_child_index
            } else {
                right_child_index
            };

            if !M::is_better(&self.keys[better_child_index], &self.keys[index]) {
                break;
            }

            self.swap(index, better_child_index);
            index = better_child_index;
        }
    }

    fn swap(&mut self, first_index: usize, second_index: usize) {
        self.keys.swap(first_index, second_index);
        self.values.swap(first_index, second_index);

        self.index_tracker
            .put_index(&self.values[first_index], first_index);
        self.index_tracker
            .put_index(&self.values[second_index], second_index);
    }
}

impl<V, K, M, T> Pq<V, K, M, T>
where
    M: KeyOrder<K>,
    T: SetIndex<V> + GetIndex<V>,
{
    pub fn update_key(&mut self, value: &V, key: K) {
        self.update_key_at_index(self.get_index(value).unwrap(), key);
    }

    pub fn push_or_update(&mut self, value: V, key: K) {
        if let Some(index) = self.get_index(&value) {
            self.update_key_at_index(index, key);
        } else {
            self.push(value, key);
        }
    }

    fn update_key_at_index(&mut self, index: usize, key: K) {
        self.keys[index] = key;

        self.down_heap(index);
        self.up_heap(index);
    }

    fn get_index(&self, value: &V) -> Option<usize> {
        self.index_tracker.get_index(value)
    }
}
