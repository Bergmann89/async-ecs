use std::mem::MaybeUninit;

use crate::entity::Index;

use super::{DistinctStorage, Storage};

pub struct VecStorage<T>(Vec<MaybeUninit<T>>);

impl<T> Storage<T> for VecStorage<T> {
    fn get(&self, index: Index) -> &T {
        unsafe { &*self.0.get_unchecked(index as usize).as_ptr() }
    }

    fn get_mut(&mut self, index: Index) -> &mut T {
        unsafe { &mut *self.0.get_unchecked_mut(index as usize).as_mut_ptr() }
    }

    fn insert(&mut self, index: Index, value: T) {
        let index = index as usize;

        if self.0.len() <= index {
            let delta = index + 1 - self.0.len();

            self.0.reserve(delta);

            unsafe {
                self.0.set_len(index + 1);
            }
        }

        unsafe {
            *self.0.get_unchecked_mut(index) = MaybeUninit::new(value);
        }
    }
}

impl<T> DistinctStorage for VecStorage<T> {}

impl<T> Default for VecStorage<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}
