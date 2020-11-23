use std::marker::PhantomData;

use crate::{
    access::{Accessor, StaticAccessor},
    resource::ResourceId,
    world::World,
};

/// A static system data that can specify its dependencies at statically (at
/// compile-time). Most system data is a `SystemData`, the `DynamicSystemData`
/// type is only needed for very special setups.
///
/// You can derive this using the `#[derive(SystemData)]` macro provided by
/// `async-ecs-derive`. That is as simple as enabling the `derive` feature.
///
/// # Examples
///
/// ```rust
/// use async_ecs::{Read, ResourceId, SystemData, World, Write};
///
/// #[derive(Default)]
/// pub struct Clock;
/// #[derive(Default)]
/// pub struct Timer;
///
/// // This will implement `SystemData` for `MySystemData`.
/// // Please note that this will only work if `SystemData`, `World` and `ResourceId` are included.
/// # #[cfg(feature = "derive")]
/// #[derive(SystemData)]
/// pub struct MySystemData<'a> {
///     pub clock: Read<'a, Clock>,
///     pub timer: Write<'a, Timer>,
/// }
/// #
/// # // The following is required for the snippet to compile without the `derive` feature.
/// #
/// # #[cfg(not(feature = "derive"))]
/// # struct MySystemData<'a> {
/// #     pub clock: Read<'a, Clock>,
/// #     pub timer: Write<'a, Timer>,
/// # }
/// #
/// # #[cfg(not(feature = "derive"))]
/// # impl<'a> SystemData<'a> for MySystemData<'a> {
/// #     fn setup(world: &mut World) {
/// #         Read::<'_, Clock>::setup(world);
/// #         Write::<'_, Timer>::setup(world);
/// #     }
/// #
/// #     fn fetch(world: &'a World) -> Self {
/// #         Self {
/// #             clock: Read::<'_, Clock>::fetch(world),
/// #             timer: Write::<'_, Timer>::fetch(world),
/// #         }
/// #     }
/// #
/// #     fn reads() -> Vec<ResourceId> {
/// #         Read::<'_, Clock>::reads()
/// #     }
/// #
/// #     fn writes() -> Vec<ResourceId> {
/// #         Write::<'_, Timer>::writes()
/// #     }
/// # }
/// ```
pub trait SystemData<'a> {
    /// Sets up the system data for fetching it from the `World`.
    fn setup(world: &mut World);

    /// Fetches the system data from `World`. Note that this is only specified
    /// for one concrete lifetime `'a`, you need to implement the
    /// `SystemData` trait for every possible lifetime.
    fn fetch(world: &'a World) -> Self;

    /// Returns all read dependencies as fetched from `Self::fetch`.
    ///
    /// Please note that returning wrong dependencies can lead to a panic.
    fn reads() -> Vec<ResourceId>;

    /// Returns all write dependencies as fetched from `Self::fetch`.
    ///
    /// Please note that returning wrong dependencies can lead to a panic.
    fn writes() -> Vec<ResourceId>;
}

/// A struct implementing system data indicates that it bundles some resources
/// which are required for the execution.
///
/// This is the more flexible, but complex variant of `SystemData`.
pub trait DynamicSystemData<'a> {
    /// The accessor of the `SystemData`, which specifies the read and write
    /// dependencies and does the fetching.
    type Accessor: Accessor;

    /// Sets up `World` for fetching this system data.
    fn setup(accessor: &Self::Accessor, world: &mut World);

    /// Creates a new resource bundle
    /// by fetching the required resources
    /// from the [`World`] struct.
    ///
    /// # Contract
    ///
    /// Only fetch the resources you returned from `reads` / `writes`!
    ///
    /// # Panics
    ///
    /// This function may panic if the above contract is violated.
    /// This function may panic if the resource doesn't exist. This is only the
    /// case if either `setup` was not called or it didn't insert any
    /// fallback value.
    ///
    /// [`World`]: trait.World.html
    fn fetch(access: &Self::Accessor, world: &'a World) -> Self;
}

/* SystemData */

impl<'a, T> SystemData<'a> for PhantomData<T>
where
    T: ?Sized,
{
    fn setup(_: &mut World) {}

    fn fetch(_: &World) -> Self {
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

    fn setup(_: &StaticAccessor<T>, world: &mut World) {
        T::setup(world);
    }

    fn fetch(_: &StaticAccessor<T>, world: &'a World) -> Self {
        T::fetch(world)
    }
}

mod impl_system_data {
    use super::*;

    macro_rules! impl_system_data {
        ( $($ty:ident),* ) => {
            impl<'a, $($ty),*> SystemData<'a> for ( $( $ty , )* )
                where $( $ty : SystemData<'a> ),*
                {
                    fn setup(world: &mut World) {
                        #![allow(unused_variables)]

                        $(
                            <$ty as SystemData>::setup(&mut *world);
                         )*
                    }

                    fn fetch(world: &'a World) -> Self {
                        #![allow(unused_variables)]

                        ( $( <$ty as SystemData<'a>>::fetch(world), )* )
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
