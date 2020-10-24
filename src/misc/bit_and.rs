use hibitset::{BitSetAnd, BitSetLike};

use crate::misc::Split;

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
