mod impls;
mod iter;
mod maybe;
mod parallel;

pub use iter::JoinIter;
pub use maybe::MaybeJoin;
pub use parallel::JoinParIter;

use hibitset::BitSetLike;

use crate::entity::Index;

pub trait Join {
    type Type;
    type Value;
    type Mask: BitSetLike;

    fn open(self) -> (Self::Mask, Self::Value);

    fn get(value: &mut Self::Value, index: Index) -> Self::Type;

    fn join(self) -> JoinIter<Self>
    where
        Self: Sized,
    {
        JoinIter::new(self)
    }

    fn maybe(self) -> MaybeJoin<Self>
    where
        Self: Sized,
    {
        MaybeJoin(self)
    }

    #[inline]
    fn is_unconstrained() -> bool {
        false
    }
}

pub trait ParJoin: Join {
    fn par_join(self) -> JoinParIter<Self>
    where
        Self: Sized,
    {
        if <Self as Join>::is_unconstrained() {
            log::warn!(
                "`ParJoin` possibly iterating through all indices, you might've made a join with all `MaybeJoin`s, which is unbounded in length."
            );
        }

        JoinParIter::new(self)
    }
}
