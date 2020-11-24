use std::ops::{Deref, DerefMut};

use crate::{
    access::{read::Read, write::Write},
    entity::Index,
    misc::BitAnd,
    resource::{Ref, RefMut, Resource},
};

use super::{Join, ParJoin};

macro_rules! define_tuple_join {
    ($($from:ident),*) => {
        impl<$($from,)*> Join for ($($from),*,)
            where $($from: Join),*,
                  ($(<$from as Join>::Mask,)*): BitAnd,
        {
            type Type = ($($from::Type),*,);
            type Value = ($($from::Value),*,);
            type Mask = <($($from::Mask,)*) as BitAnd>::Value;

            #[allow(non_snake_case)]
            unsafe fn open(self) -> (Self::Mask, Self::Value) {
                let ($($from,)*) = self;
                let ($($from,)*) = ($($from.open(),)*);

                (
                    ($($from.0),*,).and(),
                    ($($from.1),*,)
                )
            }

            #[allow(non_snake_case)]
            unsafe fn get(v: &mut Self::Value, i: Index) -> Self::Type {
                let ($(ref mut $from,)*) = v;

                ($($from::get($from, i),)*)
            }

            #[inline]
            fn is_unconstrained() -> bool {
                let mut unconstrained = true;

                $( unconstrained = unconstrained && $from::is_unconstrained(); )*

                unconstrained
            }
        }

        impl<$($from,)*> ParJoin for ($($from),*,)
            where $($from: ParJoin),*,
                  ($(<$from as Join>::Mask,)*): BitAnd,
        {}
    }
}

define_tuple_join! { A }
define_tuple_join! { A, B }
define_tuple_join! { A, B, C }
define_tuple_join! { A, B, C, D }
define_tuple_join! { A, B, C, D, E }
define_tuple_join! { A, B, C, D, E, F }
define_tuple_join! { A, B, C, D, E, F, G }
define_tuple_join! { A, B, C, D, E, F, G, H }
define_tuple_join! { A, B, C, D, E, F, G, H, I }
define_tuple_join! { A, B, C, D, E, F, G, H, I, J }
define_tuple_join! { A, B, C, D, E, F, G, H, I, J, K }
define_tuple_join! { A, B, C, D, E, F, G, H, I, J, K, L }
define_tuple_join! { A, B, C, D, E, F, G, H, I, J, K, L, M }
define_tuple_join! { A, B, C, D, E, F, G, H, I, J, K, L, M, N }
define_tuple_join! { A, B, C, D, E, F, G, H, I, J, K, L, M, N, O }
define_tuple_join! { A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P }

macro_rules! define_mutable_join {
    ($ty:ty) => {
        impl<'a, 'b, T> Join for &'a mut $ty
        where
            &'a mut T: Join,
            T: Resource,
        {
            type Type = <&'a mut T as Join>::Type;
            type Value = <&'a mut T as Join>::Value;
            type Mask = <&'a mut T as Join>::Mask;

            unsafe fn open(self) -> (Self::Mask, Self::Value) {
                self.deref_mut().open()
            }

            unsafe fn get(v: &mut Self::Value, i: Index) -> Self::Type {
                <&'a mut T as Join>::get(v, i)
            }

            #[inline]
            fn is_unconstrained() -> bool {
                <&'a mut T as Join>::is_unconstrained()
            }
        }

        impl<'a, 'b, T> ParJoin for &'a mut $ty
        where
            &'a mut T: ParJoin,
            T: Resource,
        {
        }
    };
}

define_mutable_join!(Write<'b, T>);
define_mutable_join!(RefMut<'b, T>);

macro_rules! define_immutable_join {
    ($ty:ty) => {
        impl<'a, 'b, T> Join for &'a $ty
        where
            &'a T: Join,
            T: Resource,
        {
            type Type = <&'a T as Join>::Type;
            type Value = <&'a T as Join>::Value;
            type Mask = <&'a T as Join>::Mask;

            unsafe fn open(self) -> (Self::Mask, Self::Value) {
                self.deref().open()
            }

            unsafe fn get(v: &mut Self::Value, i: Index) -> Self::Type {
                <&'a T as Join>::get(v, i)
            }

            #[inline]
            fn is_unconstrained() -> bool {
                <&'a T as Join>::is_unconstrained()
            }
        }

        impl<'a, 'b, T> ParJoin for &'a $ty
        where
            &'a T: ParJoin,
            T: Resource,
        {
        }
    };
}

define_immutable_join!(Read<'b, T>);
define_immutable_join!(Ref<'b, T>);
