mod lazy;
mod meta;
mod setup;

pub use self::meta::{CastFrom, MetaTable};
pub use lazy::Lazy;
pub use setup::{DefaultSetupHandler, PanicHandler, SetupHandler};

use std::ops::{Deref, DerefMut};

use crate::{
    access::{Read, ReadStorage, WriteStorage},
    component::Component,
    entity::{Entities, Entity, EntityBuilder},
    resource::{Cell, Ref, RefMut, Resource, ResourceId, Resources},
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
        self.entry::<MetaTable<dyn AnyStorage>>()
            .or_insert_with(Default::default);
        self.resource_mut::<MetaTable<dyn AnyStorage>>()
            .register(&*self.resource::<MaskedStorage<T>>());
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

    pub fn resource_raw(&self, id: &ResourceId) -> Option<&Cell<Box<dyn Resource>>> {
        self.0.get_raw(id)
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

    pub async fn maintain(&mut self) {
        let lazy = self.resource_mut::<Lazy>().clone();
        lazy.maintain(self).await;

        let deleted = self.entities_mut().maintain();
        if !deleted.is_empty() {
            self.entry::<MetaTable<dyn AnyStorage>>()
                .or_insert_with(Default::default);
            for storage in self
                .resource_mut::<MetaTable<dyn AnyStorage>>()
                .iter_mut(&self)
            {
                storage.drop(&deleted);
            }
        }
    }
}

impl Default for World {
    fn default() -> Self {
        let mut resources = Resources::default();

        resources.insert(Entities::default());
        resources.insert(Lazy::default());
        resources.insert(MetaTable::<dyn AnyStorage>::default());

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

/* AnyStorage */

pub trait AnyStorage {
    fn drop(&mut self, entities: &[Entity]);
}

unsafe impl<T> CastFrom<T> for dyn AnyStorage
where
    T: AnyStorage + 'static,
{
    fn cast(t: &T) -> &Self {
        t
    }

    fn cast_mut(t: &mut T) -> &mut Self {
        t
    }
}

impl<T> AnyStorage for MaskedStorage<T>
where
    T: Component,
{
    fn drop(&mut self, entities: &[Entity]) {
        for entity in entities {
            MaskedStorage::drop(self, entity.index());
        }
    }
}
