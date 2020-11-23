use crate::resource::Resource;

use super::World;

pub trait SetupHandler<T>: Sized {
    fn setup(world: &mut World);
}

pub struct DefaultSetupHandler;

impl<T> SetupHandler<T> for DefaultSetupHandler
where
    T: Default + Resource,
{
    fn setup(world: &mut World) {
        world.entry().or_insert_with(T::default);
    }
}

/// A setup handler that simply does nothing and thus will cause a panic on
/// fetching.
///
/// A typedef called `ReadExpect` exists, so you usually don't use this type
/// directly.
pub struct PanicHandler;

impl<T> SetupHandler<T> for PanicHandler
where
    T: Resource,
{
    fn setup(_: &mut World) {}
}
