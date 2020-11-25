use crate::{access::WriteStorage, component::Component, system::SystemData, world::World};

use super::Entity;

/// A common trait for `EntityBuilder` and `LazyBuilder`, allowing either to be
/// used. Entity is definitely alive, but the components may or may not exist
/// before a call to `World::maintain`.
pub trait Builder {
    /// Appends a component and associates it with the entity.
    ///
    /// If a component was already associated with the entity, it should
    /// overwrite the previous component.
    ///
    /// # Panics
    ///
    /// Panics if the component hasn't been `register()`ed in the
    /// `World`.
    fn with<C: Component + Send + Sync>(self, component: C) -> Self;

    /// Finishes the building and returns the entity.
    fn build(self) -> Entity;
}

/// The entity builder, allowing to
/// build an entity together with its components.
///
/// ## Examples
///
/// ```
/// use async_ecs::*;
///
/// struct Health(f32);
///
/// impl Component for Health {
///     type Storage = HashMapStorage<Self>;
/// }
///
/// struct Pos {
///     x: f32,
///     y: f32,
/// }
///
/// impl Component for Pos {
///     type Storage = DenseVecStorage<Self>;
/// }
///
/// let mut world = World::default();
/// world.register_component::<Health>();
/// world.register_component::<Pos>();
///
/// let entity = world
///     .create_entity() // This call returns `EntityBuilder`
///     .with(Health(4.0))
///     .with(Pos { x: 1.0, y: 3.0 })
///     .build(); // Returns the `Entity`
/// ```
///
/// ### Distinguishing Mandatory Components from Optional Components
///
/// ```
/// use async_ecs::*;
///
/// struct MandatoryHealth(f32);
///
/// impl Component for MandatoryHealth {
///     type Storage = HashMapStorage<Self>;
/// }
///
/// struct OptionalPos {
///     x: f32,
///     y: f32,
/// }
///
/// impl Component for OptionalPos {
///     type Storage = DenseVecStorage<Self>;
/// }
///
/// let mut world = World::default();
/// world.register_component::<MandatoryHealth>();
/// world.register_component::<OptionalPos>();
///
/// let mut entitybuilder = world.create_entity().with(MandatoryHealth(4.0));
///
/// // something trivial to serve as our conditional
/// let include_optional = true;
///
/// if include_optional == true {
///     entitybuilder = entitybuilder.with(OptionalPos { x: 1.0, y: 3.0 })
/// }
///
/// let entity = entitybuilder.build();
/// ```
pub struct EntityBuilder<'a> {
    world: &'a World,
    entity: Entity,
    built: bool,
}

impl<'a> EntityBuilder<'a> {
    /// Create new entity builder.
    pub fn new(world: &'a World) -> Self {
        let entity = world.entities_mut().allocate();

        Self {
            world,
            entity,
            built: false,
        }
    }
}

impl<'a> Builder for EntityBuilder<'a> {
    /// Inserts a component for this entity.
    ///
    /// If a component was already associated with the entity, it will
    /// overwrite the previous component.
    #[inline]
    fn with<T: Component>(self, c: T) -> Self {
        {
            let mut storage = WriteStorage::<T>::fetch(&self.world);

            storage.insert(self.entity, c).unwrap();
        }

        self
    }

    /// Finishes the building and returns the entity. As opposed to
    /// `LazyBuilder`, the components are available immediately.
    #[inline]
    fn build(mut self) -> Entity {
        self.built = true;

        self.entity
    }
}

impl Drop for EntityBuilder<'_> {
    fn drop(&mut self) {
        if !self.built {
            self.world.entities_mut().delete(self.entity).unwrap();
        }
    }
}
