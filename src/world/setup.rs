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
