use crate::{
    component::Component,
    entity::Entities,
    misc::TryDefault,
    resource::{RefMut, ResourceId},
    storage::{MaskedStorage, StorageWrapper},
    system::SystemData,
    world::World,
};

/// A storage with read and write access.
///
/// Additionally to what `ReadStorage` can do a storage with mutable access
/// allows:
///
/// ## Aliasing
///
/// **It is strictly disallowed to fetch both a `ReadStorage` and a
/// `WriteStorage` of the same component.**
/// Because Specs uses interior mutability for its resources, we can't check
/// this at compile time. If you try to do this, you will get a panic.
///
/// It is also disallowed to fetch multiple `WriteStorage`s for the same
/// component.
///
/// ## Retrieve components mutably
///
/// This works just like `Storage::get`, but returns a mutable reference:
///
/// ```
/// # use async_ecs::*;
/// # #[derive(Debug, PartialEq)]
/// # struct Pos(f32); impl Component for Pos { type Storage = VecStorage<Self>; }
/// #
/// # let mut world = World::default(); world.register_component::<Pos>();
/// let entity = world.create_entity().with(Pos(2.0)).build();
/// # let mut pos_storage = world.component_mut::<Pos>();
///
/// assert_eq!(pos_storage.get_mut(entity), Some(&mut Pos(2.0)));
/// if let Some(pos) = pos_storage.get_mut(entity) {
///     *pos = Pos(4.5);
/// }
///
/// assert_eq!(pos_storage.get(entity), Some(&Pos(4.5)));
/// ```
///
/// ## Inserting and removing components
///
/// You can insert components using `Storage::insert` and remove them
/// again with `Storage::remove`.
///
/// ```
/// # use async_ecs::*;
///
/// # #[derive(Debug, PartialEq)]
/// # struct Pos(f32);
/// # impl Component for Pos { type Storage = VecStorage<Self>; }
/// #
/// # let mut world = World::default();
/// # world.register_component::<Pos>();
/// #
/// let entity = world.create_entity().with(Pos(0.1)).build();
/// # let mut pos_storage = world.component_mut::<Pos>();
///
/// if let Ok(Some(p)) = pos_storage.insert(entity, Pos(4.0)) {
///     println!("Overwrote {:?} with a new position", p);
/// }
/// ```
///
/// There's also an Entry-API similar to the one provided by
/// `std::collections::HashMap`.
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
