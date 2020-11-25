mod lazy;
mod setup;

pub use lazy::Lazy;
pub use setup::{DefaultSetupHandler, PanicHandler, SetupHandler};

use std::ops::{Deref, DerefMut};

use crate::{
    access::{Read, ReadStorage, WriteStorage},
    component::Component,
    entity::{Entities, Entity, EntityBuilder},
    resource::{Ref, RefMut, Resource, Resources},
    storage::MaskedStorage,
    system::SystemData,
};

pub struct World(Resources);

impl World {
    pub fn register_component<T: Component>(&mut self)
    where
        T::Storage: Default,
    {
        self.register_component_with_storage::<T, _>(Default::default);
    }

    pub fn register_component_with_storage<T, F>(&mut self, storage: F)
    where
        T: Component,
        F: FnOnce() -> T::Storage,
    {
        self.entry()
            .or_insert_with(move || MaskedStorage::<T>::new(storage()));
    }

    pub fn register_resource<T: Resource>(&mut self, res: T) {
        self.0.insert(res);
    }

    pub fn resource<T: Resource>(&self) -> Ref<T> {
        self.0.borrow()
    }

    pub fn resource_mut<T: Resource>(&self) -> RefMut<T> {
        self.0.borrow_mut()
    }

    pub fn entities(&self) -> Read<Entities> {
        Read::fetch(&self)
    }

    pub fn entities_mut(&self) -> RefMut<Entities> {
        self.resource_mut()
    }

    pub fn lazy(&self) -> Read<Lazy> {
        Read::fetch(&self)
    }

    pub fn component<T: Component>(&self) -> ReadStorage<T> {
        ReadStorage::fetch(&self)
    }

    pub fn component_mut<T: Component>(&self) -> WriteStorage<T> {
        WriteStorage::fetch(&self)
    }

    pub fn create_entity(&mut self) -> EntityBuilder {
        EntityBuilder::new(self)
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities().is_alive(entity)
    }
}

impl Default for World {
    fn default() -> Self {
        let mut resources = Resources::default();

        resources.insert(Entities::default());
        resources.insert(Lazy::default());

        Self(resources)
    }
}

impl Deref for World {
    type Target = Resources;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for World {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
