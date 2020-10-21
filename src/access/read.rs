use std::marker::PhantomData;
use std::ops::Deref;

use crate::{
    resource::{Ref, Resource, ResourceId},
    system::SystemData,
    world::{DefaultSetupHandler, SetupHandler, World},
};

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
