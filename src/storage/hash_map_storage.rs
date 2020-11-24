use std::collections::HashMap;

use hibitset::BitSetLike;

use crate::entity::Index;

use super::{DistinctStorage, Storage};

/// `HashMap`-based storage. Best suited for rare components.
///
/// This uses the [hashbrown::HashMap] internally.
pub struct HashMapStorage<T>(HashMap<Index, T>);

impl<T> Default for HashMapStorage<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> Storage<T> for HashMapStorage<T> {
    unsafe fn get(&self, id: Index) -> &T {
        &self.0[&id]
    }

    unsafe fn get_mut(&mut self, id: Index) -> &mut T {
        self.0.get_mut(&id).unwrap()
    }

    unsafe fn insert(&mut self, id: Index, v: T) {
        self.0.insert(id, v);
    }

    unsafe fn remove(&mut self, index: Index) -> T {
        self.0.remove(&index).unwrap()
    }

    unsafe fn clean<B>(&mut self, _has: B)
    where
        B: BitSetLike,
    {
        // No Op
    }
}

impl<T> DistinctStorage for HashMapStorage<T> {}
