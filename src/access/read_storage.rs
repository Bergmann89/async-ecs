use crate::{
    component::Component,
    entity::Entities,
    misc::TryDefault,
    resource::{Ref, ResourceId},
    storage::{MaskedStorage, StorageWrapper},
    system::SystemData,
    world::World,
};

/// A storage with read access.
///
/// This is just a type alias for a fetched component storage.
///
/// The main functionality it provides is listed in the following,
/// however make sure to also check out the documentation for the
/// respective methods on `Storage`.
///
/// ## Aliasing
///
/// **It is strictly disallowed to get both a `ReadStorage` and a `WriteStorage`
/// of the same component.**
/// Because Specs uses interior mutability for its resources, we can't check
/// this at compile time. If you try to do this, you will get a panic.
///
/// It is explicitly allowed to get multiple `ReadStorage`s for the same
/// component.
///
/// ## Joining storages
///
/// `&ReadStorage` implements `Join`, which allows to do
/// something like this:
///
/// ```
/// use async_ecs::*;
///
/// struct Pos;
///
/// impl Component for Pos {
///     type Storage = VecStorage<Self>;
/// }
///
/// struct Vel;
///
/// impl Component for Vel {
///     type Storage = VecStorage<Self>;
/// }
///
/// let mut world = World::default();
/// world.register_component::<Pos>();
/// world.register_component::<Vel>();
///
/// let pos_storage = world.component::<Pos>();
/// let vel_storage = world.component::<Vel>();
///
/// for (pos, vel) in (&pos_storage, &vel_storage).join() {}
/// ```
///
/// This joins the position and the velocity storage, which means it only
/// iterates over the components of entities that have both a position
/// **and** a velocity.
///
/// ## Retrieving single components
///
/// If you have an entity (for example because you stored it before
/// or because you're joining over `Entities`), you can get a single
/// component by calling `Storage::get`:
///
/// ```
/// # use async_ecs::*;
/// #
/// # #[derive(Debug, PartialEq)]
/// # struct Pos;
/// # impl Component for Pos { type Storage = VecStorage<Self>; }
/// #
/// # #[derive(Debug, PartialEq)]
/// # struct Vel;
/// # impl Component for Vel { type Storage = VecStorage<Self>; }
/// #
/// # let mut world = World::default();
/// # world.register_component::<Pos>();
/// # world.register_component::<Vel>();
/// #
/// let entity1 = world.create_entity().with(Pos).build();
/// let entity2 = world.create_entity().with(Vel).build();
///
/// # let pos_storage = world.component::<Pos>();
/// # let vel_storage = world.component::<Vel>();
/// assert_eq!(pos_storage.get(entity1), Some(&Pos));
/// assert_eq!(pos_storage.get(entity2), None);
///
/// assert_eq!(vel_storage.get(entity1), None);
/// assert_eq!(vel_storage.get(entity2), Some(&Vel));
/// ```
///
/// ## Usage as `SystemData`
///
/// `ReadStorage` implements `SystemData` which allows you to
/// get it inside a system by simply adding it to the tuple:
///
/// ```
/// # use async_ecs::*;
/// #[derive(Debug)]
/// struct Pos {
///     x: f32,
///     y: f32,
/// }
///
/// impl Component for Pos {
///     type Storage = VecStorage<Self>;
/// }
///
/// struct Sys;
///
/// impl<'a> System<'a> for Sys {
///     type SystemData = (Entities<'a>, ReadStorage<'a, Pos>);
///
///     fn run(&mut self, (ent, pos): Self::SystemData) {
///         for (ent, pos) in (&*ent, &pos).join() {
///             println!("Entitiy with id {} has a position of {:?}", ent.id(), pos);
///         }
///     }
/// }
/// ```
///
/// These operations can't mutate anything; if you want to do
/// insertions or modify components, you need to use `WriteStorage`.
/// Note that you can also use `LazyUpdate` , which does insertions on
/// `World::maintain`. This allows more concurrency and is designed
/// to be used for entity initialization.
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
