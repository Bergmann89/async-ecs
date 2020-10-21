use hibitset::BitSet;

use crate::component::Component;

pub struct MaskedStorage<T: Component> {
    mask: BitSet,
    inner: T::Storage,
}

impl<T: Component> MaskedStorage<T> {
    pub fn new(inner: T::Storage) -> Self {
        Self {
            mask: BitSet::new(),
            inner,
        }
    }
}
