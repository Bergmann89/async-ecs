use hibitset::{BitSetLike, DrainableBitSet};

use crate::{entity::Index, misc::Split};

/* BitSetAnd */

#[derive(Debug, Clone, Copy)]
pub struct BitSetAnd<A: BitSetLike, B: BitSetLike>(pub A, pub B);

impl<A, B> BitSetLike for BitSetAnd<A, B>
where
    A: BitSetLike,
    B: BitSetLike,
{
    #[inline]
    fn layer3(&self) -> usize {
        self.0.layer3() & self.1.layer3()
    }
    #[inline]
    fn layer2(&self, i: usize) -> usize {
        self.0.layer2(i) & self.1.layer2(i)
    }
    #[inline]
    fn layer1(&self, i: usize) -> usize {
        self.0.layer1(i) & self.1.layer1(i)
    }
    #[inline]
    fn layer0(&self, i: usize) -> usize {
        self.0.layer0(i) & self.1.layer0(i)
    }
    #[inline]
    fn contains(&self, i: Index) -> bool {
        self.0.contains(i) && self.1.contains(i)
    }
}

impl<A, B> DrainableBitSet for BitSetAnd<A, B>
where
    A: DrainableBitSet,
    B: DrainableBitSet,
{
    #[inline]
    fn remove(&mut self, i: Index) -> bool {
        if self.contains(i) {
            self.0.remove(i);
            self.1.remove(i);
            true
        } else {
            false
        }
    }
}

/* BitAnd */

pub trait BitAnd {
    type Value: BitSetLike;

    fn and(self) -> Self::Value;
}

impl<A> BitAnd for (A,)
where
    A: BitSetLike,
{
    type Value = A;

    fn and(self) -> Self::Value {
        self.0
    }
}

macro_rules! bitset_and {
    ($($from:ident),*) => {
        impl<$($from),*> BitAnd for ($($from),*)
            where $($from: BitSetLike),*
        {
            type Value = BitSetAnd<
                <<Self as Split>::Left as BitAnd>::Value,
                <<Self as Split>::Right as BitAnd>::Value
            >;

            fn and(self) -> Self::Value {
                let (l, r) = self.split();

                BitSetAnd(l.and(), r.and())
            }
        }
    }
}

bitset_and! { A, B }
bitset_and! { A, B, C }
bitset_and! { A, B, C, D }
bitset_and! { A, B, C, D, E }
bitset_and! { A, B, C, D, E, F }
bitset_and! { A, B, C, D, E, F, G }
bitset_and! { A, B, C, D, E, F, G, H }
bitset_and! { A, B, C, D, E, F, G, H, I }
bitset_and! { A, B, C, D, E, F, G, H, I, J }
bitset_and! { A, B, C, D, E, F, G, H, I, J, K }
bitset_and! { A, B, C, D, E, F, G, H, I, J, K, L }
bitset_and! { A, B, C, D, E, F, G, H, I, J, K, L, M }
bitset_and! { A, B, C, D, E, F, G, H, I, J, K, L, M, N }
bitset_and! { A, B, C, D, E, F, G, H, I, J, K, L, M, N, O }
bitset_and! { A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P }
