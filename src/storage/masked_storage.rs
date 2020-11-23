use std::mem::swap;

use hibitset::BitSet;

use crate::{
    component::Component,
    entity::{Entity, Index},
    storage::Storage,
};

/// The `Storage` together with the `BitSet` that knows
/// about which elements are stored, and which are not.
pub struct MaskedStorage<T: Component> {
    mask: BitSet,
    inner: T::Storage,
}

impl<T: Component> MaskedStorage<T> {
    /// Create new masked storage.
    pub fn new(inner: T::Storage) -> Self {
        Self {
            mask: BitSet::new(),
            inner,
        }
    }

    /// Get the mask of living elements.
    pub fn mask(&self) -> &BitSet {
        &self.mask
    }

    /// Get areference to the inner storage.
    pub fn storage(&self) -> &T::Storage {
        &self.inner
    }

    /// Get a mutable reference to the inner storage.
    pub fn storage_mut(&mut self) -> &mut T::Storage {
        &mut self.inner
    }

    /// Insert new element
    pub fn insert(&mut self, entity: Entity, mut component: T) -> Option<T> {
        let index = entity.index();

        if self.mask.contains(index) {
            swap(&mut component, self.inner.get_mut(index));

            Some(component)
        } else {
            self.mask.add(index);
            self.inner.insert(index, component);

            None
        }
    }

    /// Clear the contents of this storage.
    pub fn clear(&mut self) {
        self.inner.clean(&self.mask);

        self.mask.clear();
    }

    /// Remove an element by a given index.
    pub fn remove(&mut self, index: Index) -> Option<T> {
        if self.mask.remove(index) {
            Some(self.inner.remove(index))
        } else {
            None
        }
    }

    /// Drop an element by a given index.
    pub fn drop(&mut self, index: Index) {
        if self.mask.remove(index) {
            self.inner.drop(index);
        }
    }
}
