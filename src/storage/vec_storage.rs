use std::mem::MaybeUninit;
use std::ptr::{drop_in_place, read};

use hibitset::BitSetLike;

use crate::entity::Index;

use super::{DistinctStorage, Storage};

/// Vector storage. Uses a simple `Vec`. Supposed to have maximum
/// performance for the components mostly present in entities.
///
/// `as_slice()` and `as_mut_slice()` indices correspond to
/// entity IDs. These can be compared to other `VecStorage`s, to
/// other `DefaultVecStorage`s, and to `Entity::id()`s for live
/// entities.
pub struct VecStorage<T>(Vec<MaybeUninit<T>>);

impl<T> Storage<T> for VecStorage<T> {
    unsafe fn get(&self, index: Index) -> &T {
        &*self.0.get_unchecked(index as usize).as_ptr()
    }

    unsafe fn get_mut(&mut self, index: Index) -> &mut T {
        &mut *self.0.get_unchecked_mut(index as usize).as_mut_ptr()
    }

    unsafe fn insert(&mut self, index: Index, value: T) {
        let index = index as usize;

        if self.0.len() <= index {
            let delta = index + 1 - self.0.len();

            self.0.reserve(delta);

            self.0.set_len(index + 1);
        }

        *self.0.get_unchecked_mut(index) = MaybeUninit::new(value);
    }

    unsafe fn remove(&mut self, index: Index) -> T {
        read(self.get(index))
    }

    unsafe fn clean<B>(&mut self, has: B)
    where
        B: BitSetLike,
    {
        for (i, v) in self.0.iter_mut().enumerate() {
            if has.contains(i as u32) {
                // drop in place
                drop_in_place(&mut *v.as_mut_ptr());
            }
        }

        self.0.set_len(0);
    }
}

impl<T> DistinctStorage for VecStorage<T> {}

impl<T> Default for VecStorage<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}
