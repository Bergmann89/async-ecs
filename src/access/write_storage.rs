use crate::{
    component::Component,
    entity::Entities,
    misc::TryDefault,
    resource::{RefMut, ResourceId},
    storage::{MaskedStorage, StorageWrapper},
    system::SystemData,
    world::World,
};

pub type WriteStorage<'a, T> = StorageWrapper<'a, T, RefMut<'a, MaskedStorage<T>>>;

impl<'a, T> SystemData<'a> for WriteStorage<'a, T>
where
    T: Component,
{
    fn setup(world: &mut World) {
        world.register_component_with_storage::<T, _>(TryDefault::unwrap_default);
    }

    fn fetch(world: &'a World) -> Self {
        Self::new(world.borrow_mut(), world.borrow())
    }

    fn reads() -> Vec<ResourceId> {
        vec![ResourceId::new::<Entities>()]
    }

    fn writes() -> Vec<ResourceId> {
        vec![ResourceId::new::<MaskedStorage<T>>()]
    }
}
