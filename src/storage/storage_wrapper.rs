use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Not};

use hibitset::{BitSet, BitSetNot};

use crate::{
    access::Join,
    component::Component,
    entity::{Entities, Entity, Index},
    error::Error,
    resource::Ref,
    storage::MaskedStorage,
};

use super::Storage;

pub struct StorageWrapper<'a, T, D> {
    data: D,
    entities: Ref<'a, Entities>,
    phantom: PhantomData<T>,
}

pub struct AntiStorage<'a>(pub &'a BitSet);

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
        AntiStorage(&self.data.mask)
    }
}

impl<'a, 'e, T, D> Not for &'a StorageWrapper<'e, T, D>
where
    T: Component,
    D: Deref<Target = MaskedStorage<T>>,
{
    type Output = AntiStorage<'a>;

    fn not(self) -> Self::Output {
        AntiStorage(&self.data.mask)
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
        (&self.data.mask, &self.data.inner)
    }

    fn get(v: &mut Self::Value, i: Index) -> &'a T {
        v.get(i)
    }
}

impl<'a, 'e, T, D> Join for &'a mut StorageWrapper<'e, T, D>
where
    T: Component,
    D: DerefMut<Target = MaskedStorage<T>>,
{
    type Mask = &'a BitSet;
    type Type = &'a mut T;
    type Value = &'a mut T::Storage;

    fn open(self) -> (Self::Mask, Self::Value) {
        self.data.open_mut()
    }

    fn get(v: &mut Self::Value, i: Index) -> &'a mut T {
        // HACK
        let value: *mut Self::Value = v as *mut Self::Value;

        unsafe { (*value).get_mut(i) }
    }
}

impl<'a> Join for AntiStorage<'a> {
    type Mask = BitSetNot<&'a BitSet>;
    type Type = ();
    type Value = ();

    fn open(self) -> (Self::Mask, ()) {
        (BitSetNot(self.0), ())
    }

    fn get(_: &mut (), _: Index) {}
}
