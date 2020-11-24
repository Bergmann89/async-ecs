use hibitset::{BitSet, BitSetNot};

use crate::{
    entity::Index,
    join::{Join, ParJoin},
};

use super::DistinctStorage;

/// An inverted storage type, only useful to iterate entities
/// that do not have a particular component type.
pub struct AntiStorage<'a>(pub &'a BitSet);

impl<'a> DistinctStorage for AntiStorage<'a> {}

impl<'a> Join for AntiStorage<'a> {
    type Mask = BitSetNot<&'a BitSet>;
    type Type = ();
    type Value = ();

    unsafe fn open(self) -> (Self::Mask, ()) {
        (BitSetNot(self.0), ())
    }

    unsafe fn get(_: &mut Self::Value, _: Index) {}
}

impl<'a> ParJoin for AntiStorage<'a> {}
