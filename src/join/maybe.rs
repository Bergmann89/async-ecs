use hibitset::{BitSetAll, BitSetLike};

use crate::entity::Index;

use super::{Join, ParJoin};

/// A `Join`-able structure that yields all indices, returning `None` for all
/// missing elements and `Some(T)` for found elements.
///
/// For usage see [`Join::maybe()`].
///
/// WARNING: Do not have a join of only `MaybeJoin`s. Otherwise the join will
/// iterate over every single index of the bitset. If you want a join with
/// all `MaybeJoin`s, add an `Entities` to the join as well to bound the
/// join to all entities that are alive.
///
/// [`Join::maybe()`]: trait.Join.html#method.maybe
pub struct MaybeJoin<J: Join>(pub J);

impl<T> Join for MaybeJoin<T>
where
    T: Join,
{
    type Mask = BitSetAll;
    type Type = Option<<T as Join>::Type>;
    type Value = (<T as Join>::Mask, <T as Join>::Value);

    unsafe fn open(self) -> (Self::Mask, Self::Value) {
        let (mask, value) = self.0.open();

        (BitSetAll, (mask, value))
    }

    unsafe fn get((mask, value): &mut Self::Value, index: Index) -> Self::Type {
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
