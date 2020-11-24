use std::collections::BTreeMap;

use hibitset::BitSetLike;

use crate::entity::Index;

use super::{DistinctStorage, Storage};

pub struct BTreeStorage<T>(BTreeMap<Index, T>);

impl<T> Default for BTreeStorage<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> Storage<T> for BTreeStorage<T> {
    fn get(&self, index: Index) -> &T {
        &self.0[&index]
    }

    fn get_mut(&mut self, index: Index) -> &mut T {
        self.0.get_mut(&index).unwrap()
    }

    fn insert(&mut self, index: Index, value: T) {
        self.0.insert(index, value);
    }

    fn remove(&mut self, index: Index) -> T {
        self.0.remove(&index).unwrap()
    }

    fn clean<B>(&mut self, _has: B)
    where
        B: BitSetLike,
    {
    }
}

impl<T> DistinctStorage for BTreeStorage<T> {}
