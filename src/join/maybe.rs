use hibitset::{BitSetAll, BitSetLike};

use crate::entity::Index;

use super::{Join, ParJoin};

pub struct MaybeJoin<J: Join>(pub J);

impl<T> Join for MaybeJoin<T>
where
    T: Join,
{
    type Mask = BitSetAll;
    type Type = Option<<T as Join>::Type>;
    type Value = (<T as Join>::Mask, <T as Join>::Value);

    fn open(self) -> (Self::Mask, Self::Value) {
        let (mask, value) = self.0.open();

        (BitSetAll, (mask, value))
    }

    fn get((mask, value): &mut Self::Value, index: Index) -> Self::Type {
        if mask.contains(index) {
            Some(<T as Join>::get(value, index))
        } else {
            None
        }
    }

    fn is_unconstrained() -> bool {
        true
    }
}

impl<T> ParJoin for MaybeJoin<T> where T: ParJoin {}
