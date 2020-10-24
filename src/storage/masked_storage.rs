use std::mem::swap;

use hibitset::BitSet;

use crate::{component::Component, entity::Entity, storage::Storage};

pub struct MaskedStorage<T: Component> {
    pub(crate) mask: BitSet,
    pub(crate) inner: T::Storage,
}

impl<T: Component> MaskedStorage<T> {
    pub fn new(inner: T::Storage) -> Self {
        Self {
            mask: BitSet::new(),
            inner,
        }
    }

    pub fn open_mut(&mut self) -> (&BitSet, &mut T::Storage) {
        (&self.mask, &mut self.inner)
    }

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
}
