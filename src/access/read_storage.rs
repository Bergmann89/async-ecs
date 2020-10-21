use crate::{
    component::Component,
    entity::Entities,
    misc::TryDefault,
    resource::{Ref, ResourceId},
    storage::{MaskedStorage, StorageWrapper},
    system::SystemData,
    world::World,
};

pub type ReadStorage<'a, T> = StorageWrapper<'a, T, Ref<'a, MaskedStorage<T>>>;

impl<'a, T> SystemData<'a> for ReadStorage<'a, T>
where
    T: Component,
{
    fn setup(world: &mut World) {
        world.register_component_with_storage::<T, _>(TryDefault::unwrap_default);
    }

    fn fetch(world: &'a World) -> Self {
        Self::new(world.borrow(), world.borrow())
    }

    fn reads() -> Vec<ResourceId> {
        vec![
            ResourceId::new::<Entities>(),
            ResourceId::new::<MaskedStorage<T>>(),
        ]
    }

    fn writes() -> Vec<ResourceId> {
        vec![]
    }
}
