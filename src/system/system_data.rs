use std::marker::PhantomData;

use crate::resource::{ResourceId, Resources};

use super::accessor::{Accessor, StaticAccessor};

pub trait SystemData<'a> {
    fn setup(resources: &mut Resources);

    fn fetch(resources: &'a Resources) -> Self;

    fn reads() -> Vec<ResourceId>;

    fn writes() -> Vec<ResourceId>;
}

pub trait DynamicSystemData<'a> {
    type Accessor: Accessor;

    fn setup(accessor: &Self::Accessor, resources: &mut Resources);

    fn fetch(access: &Self::Accessor, resources: &'a Resources) -> Self;
}

/* SystemData */

impl<'a, T> SystemData<'a> for PhantomData<T>
where
    T: ?Sized,
{
    fn setup(_: &mut Resources) {}

    fn fetch(_: &Resources) -> Self {
        PhantomData
    }

    fn reads() -> Vec<ResourceId> {
        vec![]
    }

    fn writes() -> Vec<ResourceId> {
        vec![]
    }
}

/* DynamicSystemData */

impl<'a, T> DynamicSystemData<'a> for T
where
    T: SystemData<'a>,
{
    type Accessor = StaticAccessor<T>;

    fn setup(_: &StaticAccessor<T>, resources: &mut Resources) {
        T::setup(resources);
    }

    fn fetch(_: &StaticAccessor<T>, resources: &'a Resources) -> Self {
        T::fetch(resources)
    }
}

mod impl_system_data {
    use super::*;

    macro_rules! impl_system_data {
        ( $($ty:ident),* ) => {
            impl<'a, $($ty),*> SystemData<'a> for ( $( $ty , )* )
                where $( $ty : SystemData<'a> ),*
                {
                    fn setup(resources: &mut Resources) {
                        #![allow(unused_variables)]

                        $(
                            <$ty as SystemData>::setup(&mut *resources);
                         )*
                    }

                    fn fetch(resources: &'a Resources) -> Self {
                        #![allow(unused_variables)]

                        ( $( <$ty as SystemData<'a>>::fetch(resources), )* )
                    }

                    fn reads() -> Vec<ResourceId> {
                        #![allow(unused_mut)]

                        let mut r = Vec::new();

                        $( {
                            let mut reads = <$ty as SystemData>::reads();
                            r.append(&mut reads);
                        } )*

                        r
                    }

                    fn writes() -> Vec<ResourceId> {
                        #![allow(unused_mut)]

                        let mut r = Vec::new();

                        $( {
                            let mut writes = <$ty as SystemData>::writes();
                            r.append(&mut writes);
                        } )*

                        r
                    }
                }
        };
    }

    impl_system_data!(A);
    impl_system_data!(A, B);
    impl_system_data!(A, B, C);
    impl_system_data!(A, B, C, D);
    impl_system_data!(A, B, C, D, E);
    impl_system_data!(A, B, C, D, E, F);
    impl_system_data!(A, B, C, D, E, F, G);
    impl_system_data!(A, B, C, D, E, F, G, H);
    impl_system_data!(A, B, C, D, E, F, G, H, I);
    impl_system_data!(A, B, C, D, E, F, G, H, I, J);
    impl_system_data!(A, B, C, D, E, F, G, H, I, J, K);
    impl_system_data!(A, B, C, D, E, F, G, H, I, J, K, L);
    impl_system_data!(A, B, C, D, E, F, G, H, I, J, K, L, M);
    impl_system_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
    impl_system_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
    impl_system_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
    impl_system_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
    impl_system_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
    impl_system_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
    impl_system_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
    impl_system_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
    impl_system_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
    impl_system_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
    impl_system_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
    impl_system_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
    impl_system_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
}
