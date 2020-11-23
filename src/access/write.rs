use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use crate::{
    resource::{RefMut, Resource, ResourceId},
    system::SystemData,
    world::{DefaultSetupHandler, PanicHandler, SetupHandler, World},
};

/// Allows to fetch a resource in a system mutably.
/// **This will panic if the resource does not exist.**
/// Usage of `Write` or `Option<Write>` is therefore recommended.
pub type WriteExpect<'a, T> = Write<'a, T, PanicHandler>;

/// Allows to fetch a resource in a system mutably.
///
/// If the resource isn't strictly required, you should use `Option<Write<T>>`.
///
/// # Type parameters
///
/// * `T`: The type of the resource
/// * `F`: The setup handler (default: `DefaultProvider`)
pub struct Write<'a, T: 'a, F = DefaultSetupHandler> {
    inner: RefMut<'a, T>,
    marker: PhantomData<F>,
}

impl<'a, T, F> Write<'a, T, F> {
    pub fn new(inner: RefMut<'a, T>) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }
}

impl<'a, T, F> Deref for Write<'a, T, F>
where
    T: Resource,
{
    type Target = T;

    fn deref(&self) -> &T {
        &*self.inner
    }
}

impl<'a, T, F> DerefMut for Write<'a, T, F>
where
    T: Resource,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.inner
    }
}

impl<'a, T, F> SystemData<'a> for Write<'a, T, F>
where
    T: Resource,
    F: SetupHandler<T>,
{
    fn setup(world: &mut World) {
        F::setup(world)
    }

    fn fetch(world: &'a World) -> Self {
        Self::new(world.borrow_mut())
    }

    fn reads() -> Vec<ResourceId> {
        vec![]
    }

    fn writes() -> Vec<ResourceId> {
        vec![ResourceId::new::<T>()]
    }
}
