use std::mem::MaybeUninit;

use hibitset::BitSetLike;

use crate::{entity::Index, storage::Storage};

use super::DistinctStorage;

/// Dense vector storage. Has a redirection 2-way table
/// between entities and components, allowing to leave
/// no gaps within the data.
///
/// Note that this only stores the data (`T`) densely; indices
/// to the data are stored in a sparse `Vec`.
///
/// `as_slice()` and `as_mut_slice()` indices are local to this
/// `DenseVecStorage` at this particular moment. These indices
/// cannot be compared with indices from any other storage, and
/// a particular entity's position within this slice may change
/// over time.
pub struct DenseVecStorage<T> {
    data: Vec<T>,
    entity_id: Vec<Index>,
    data_id: Vec<MaybeUninit<Index>>,
}

impl<T> Default for DenseVecStorage<T> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            entity_id: Default::default(),
            data_id: Default::default(),
        }
    }
}

impl<T> Storage<T> for DenseVecStorage<T> {
    unsafe fn get(&self, index: Index) -> &T {
        let index = self.data_id.get_unchecked(index as usize).assume_init();

        self.data.get_unchecked(index as usize)
    }

    unsafe fn get_mut(&mut self, index: Index) -> &mut T {
        let index = self.data_id.get_unchecked(index as usize).assume_init();

        self.data.get_unchecked_mut(index as usize)
    }

    unsafe fn insert(&mut self, index: Index, v: T) {
        let index = index as usize;

        if self.data_id.len() <= index {
            let delta = index + 1 - self.data_id.len();
            self.data_id.reserve(delta);
            self.data_id.set_len(index + 1);
        }

        self.data_id
            .get_unchecked_mut(index)
            .as_mut_ptr()
            .write(self.data.len() as Index);
        self.entity_id.push(index as Index);
        self.data.push(v);
    }

    unsafe fn remove(&mut self, index: Index) -> T {
        let index = self.data_id.get_unchecked(index as usize).assume_init();
        let last = *self.entity_id.last().unwrap();
        self.data_id
            .get_unchecked_mut(last as usize)
            .as_mut_ptr()
            .write(index);
        self.entity_id.swap_remove(index as usize);
        self.data.swap_remove(index as usize)
    }

    unsafe fn clean<B>(&mut self, _has: B)
    where
        B: BitSetLike,
    {
        // No Op
    }
}

impl<T> DistinctStorage for DenseVecStorage<T> {}
