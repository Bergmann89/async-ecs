use std::marker::PhantomData;
use std::ops::Deref;

use crate::{
    resource::{Ref, Resource, ResourceId},
    system::SystemData,
    world::{DefaultSetupHandler, PanicHandler, SetupHandler, World},
};

/// Allows to fetch a resource in a system immutably.
/// **This will panic if the resource does not exist.**
/// Usage of `Read` or `Option<Read>` is therefore recommended.
pub type ReadExpect<'a, T> = Read<'a, T, PanicHandler>;

/// Allows to fetch a resource in a system immutably.
///
/// If the resource isn't strictly required, you should use `Option<Read<T>>`.
///
/// # Type parameters
///
/// * `T`: The type of the resource
/// * `F`: The setup handler (default: `DefaultProvider`)
pub struct Read<'a, T: 'a, F = DefaultSetupHandler> {
    inner: Ref<'a, T>,
    marker: PhantomData<F>,
}

impl<'a, T, F> Read<'a, T, F> {
    pub fn new(inner: Ref<'a, T>) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }
}

impl<'a, T, F> From<Ref<'a, T>> for Read<'a, T, F> {
    fn from(inner: Ref<'a, T>) -> Self {
        Read {
            inner,
            marker: PhantomData,
        }
    }
}

impl<'a, T, F> Deref for Read<'a, T, F>
where
    T: Resource,
{
    type Target = T;

    fn deref(&self) -> &T {
        &*self.inner
    }
}

impl<'a, T, F> SystemData<'a> for Read<'a, T, F>
where
    T: Resource,
    F: SetupHandler<T>,
{
    fn setup(world: &mut World) {
        F::setup(world)
    }

    fn fetch(world: &'a World) -> Self {
        Self::new(world.borrow())
    }

    fn reads() -> Vec<ResourceId> {
        vec![ResourceId::new::<T>()]
    }

    fn writes() -> Vec<ResourceId> {
        vec![]
    }
}

impl<'a, T, F> SystemData<'a> for Option<Read<'a, T, F>>
where
    T: Resource,
{
    fn setup(_: &mut World) {}

    fn fetch(world: &'a World) -> Self {
        world.try_borrow().map(Into::into)
    }

    fn reads() -> Vec<ResourceId> {
        vec![ResourceId::new::<T>()]
    }

    fn writes() -> Vec<ResourceId> {
        vec![]
    }
}
