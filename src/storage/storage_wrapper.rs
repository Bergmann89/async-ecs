use std::marker::PhantomData;
use std::ops::DerefMut;

use crate::{
    component::Component,
    entity::{Entities, Entity},
    error::Error,
    resource::Ref,
    storage::MaskedStorage,
};

pub struct StorageWrapper<'a, T, D> {
    data: D,
    entities: Ref<'a, Entities>,
    phantom: PhantomData<T>,
}

impl<'a, T, D> StorageWrapper<'a, T, D> {
    pub fn new(data: D, entities: Ref<'a, Entities>) -> Self {
        Self {
            data,
            entities,
            phantom: PhantomData,
        }
    }
}

impl<'a, T, D> StorageWrapper<'a, T, D>
where
    T: Component,
    D: DerefMut<Target = MaskedStorage<T>>,
{
    pub fn insert(&mut self, entity: Entity, component: T) -> Result<Option<T>, Error> {
        if !self.entities.is_alive(entity) {
            return Err(Error::EntityIsNotAlive(entity));
        }

        Ok(self.data.insert(entity, component))
    }
}
