use hibitset::{BitSet, BitSetNot};

use crate::{
    entity::Index,
    join::{Join, ParJoin},
};

use super::DistinctStorage;

pub struct AntiStorage<'a>(pub &'a BitSet);

impl<'a> DistinctStorage for AntiStorage<'a> {}

impl<'a> Join for AntiStorage<'a> {
    type Mask = BitSetNot<&'a BitSet>;
    type Type = ();
    type Value = ();

    fn open(self) -> (Self::Mask, ()) {
        (BitSetNot(self.0), ())
    }

    fn get(_: &mut Self::Value, _: Index) {}
}

impl<'a> ParJoin for AntiStorage<'a> {}
