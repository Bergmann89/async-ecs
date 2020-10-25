use std::ops::{Deref, DerefMut};

use hibitset::{BitIter, BitSetAll, BitSetLike};
use log::warn;

use crate::{
    entity::{Entities, Entity, Index},
    misc::BitAnd,
    resource::{Ref, RefMut, Resource},
};

use super::{read::Read, write::Write, ParJoin};

/* Join */

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

/* MaybeJoin */

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

/* ParJoin */

impl<T> ParJoin for MaybeJoin<T> where T: ParJoin {}

/* JoinIter */

pub struct JoinIter<J: Join> {
    keys: BitIter<J::Mask>,
    values: J::Value,
}

impl<J: Join> JoinIter<J> {
    pub fn new(j: J) -> Self {
        if <J as Join>::is_unconstrained() {
            warn!(
                "`Join` possibly iterating through all indices, you might've made a join with all `MaybeJoin`s, which is unbounded in length."
            );
        }

        let (keys, values) = j.open();

        JoinIter {
            keys: keys.iter(),
            values,
        }
    }

    pub fn get(&mut self, entity: Entity, entities: &Entities) -> Option<J::Type> {
        if self.keys.contains(entity.index()) && entities.is_alive(entity) {
            Some(J::get(&mut self.values, entity.index()))
        } else {
            None
        }
    }
}

impl<J: Join> std::iter::Iterator for JoinIter<J> {
    type Item = J::Type;

    fn next(&mut self) -> Option<J::Type> {
        self.keys.next().map(|idx| J::get(&mut self.values, idx))
    }
}

impl<J: Join> Clone for JoinIter<J>
where
    J::Mask: Clone,
    J::Value: Clone,
{
    fn clone(&self) -> Self {
        Self {
            keys: self.keys.clone(),
            values: self.values.clone(),
        }
    }
}

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
            fn open(self) -> (Self::Mask, Self::Value) {
                let ($($from,)*) = self;
                let ($($from,)*) = ($($from.open(),)*);

                (
                    ($($from.0),*,).and(),
                    ($($from.1),*,)
                )
            }

            #[allow(non_snake_case)]
            fn get(v: &mut Self::Value, i: Index) -> Self::Type {
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

            fn open(self) -> (Self::Mask, Self::Value) {
                self.deref_mut().open()
            }

            fn get(v: &mut Self::Value, i: Index) -> Self::Type {
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

            fn open(self) -> (Self::Mask, Self::Value) {
                self.deref().open()
            }

            fn get(v: &mut Self::Value, i: Index) -> Self::Type {
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
