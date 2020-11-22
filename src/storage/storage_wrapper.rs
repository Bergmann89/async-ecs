use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Not};

use hibitset::BitSet;

use crate::{
    component::Component,
    entity::{Entities, Entity, Index},
    error::Error,
    join::{Join, ParJoin},
    resource::Ref,
    storage::MaskedStorage,
};

use super::{AntiStorage, DistinctStorage, Storage};

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

    pub fn not(&self) -> AntiStorage<'_> {
        AntiStorage(&self.data.mask())
    }
}

impl<'a, T: Component, D> DistinctStorage for StorageWrapper<'a, T, D> where
    T::Storage: DistinctStorage
{
}

impl<'a, 'e, T, D> Not for &'a StorageWrapper<'e, T, D>
where
    T: Component,
    D: Deref<Target = MaskedStorage<T>>,
{
    type Output = AntiStorage<'a>;

    fn not(self) -> Self::Output {
        AntiStorage(&self.data.mask())
    }
}

impl<'a, 'e, T, D> Join for &'a StorageWrapper<'e, T, D>
where
    T: Component,
    D: Deref<Target = MaskedStorage<T>>,
{
    type Mask = &'a BitSet;
    type Type = &'a T;
    type Value = &'a T::Storage;

    fn open(self) -> (Self::Mask, Self::Value) {
        (self.data.mask(), self.data.storage())
    }

    fn get(v: &mut Self::Value, i: Index) -> &'a T {
        (**v).get(i)
    }
}

impl<'a, 'e, T, D> Join for &'a mut StorageWrapper<'e, T, D>
where
    T: Component,
    D: DerefMut<Target = MaskedStorage<T>>,
{
    type Mask = &'a BitSet;
    type Type = &'a mut T;
    type Value = &'a T::Storage;

    fn open(self) -> (Self::Mask, Self::Value) {
        (self.data.mask(), self.data.storage())
    }

    fn get(v: &mut Self::Value, i: Index) -> &'a mut T {
        unsafe {
            let value: *mut T::Storage = *v as *const T::Storage as *mut T::Storage;

            (*value).get_mut(i)
        }
    }
}

impl<'a, 'e, T, D> ParJoin for &'a StorageWrapper<'e, T, D>
where
    T: Component,
    D: Deref<Target = MaskedStorage<T>>,
    T::Storage: Sync,
{
}

impl<'a, 'e, T, D> ParJoin for &'a mut StorageWrapper<'e, T, D>
where
    T: Component,
    D: DerefMut<Target = MaskedStorage<T>>,
    T::Storage: Sync + DistinctStorage,
{
}
