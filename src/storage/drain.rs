use hibitset::BitSet;

use crate::{component::Component, entity::Index, join::Join};

use super::MaskedStorage;

/// A draining storage wrapper which has a `Join` implementation
/// that removes the components.
pub struct Drain<'a, T: Component> {
    /// The masked storage
    pub data: &'a mut MaskedStorage<T>,
}

impl<'a, T> Join for Drain<'a, T>
where
    T: Component,
{
    type Mask = BitSet;
    type Type = T;
    type Value = &'a mut MaskedStorage<T>;

    unsafe fn open(self) -> (Self::Mask, Self::Value) {
        let mask = self.data.mask().clone();

        (mask, self.data)
    }

    unsafe fn get(value: &mut Self::Value, id: Index) -> T {
        value.remove(id).expect("Tried to access same index twice")
    }
}
