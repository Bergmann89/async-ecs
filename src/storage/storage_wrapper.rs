use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Not};

use hibitset::{BitSet, BitSetLike};

use crate::{
    component::Component,
    entity::{Entities, Entity, Index},
    error::Error,
    join::{Join, ParJoin},
    resource::Ref,
    storage::MaskedStorage,
};

use super::{AntiStorage, DistinctStorage, Drain, Storage};

/// A wrapper around the masked storage and the generations vector.
/// Can be used for safe lookup of components, insertions and removes.
/// This is what `World::read/write` fetches for the user.
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
    D: Deref<Target = MaskedStorage<T>>,
{
    /// Returns the `Entities` resource fetched by this storage.
    /// **This does not have anything to do with the components inside.**
    /// You only want to use this when implementing additional methods
    /// for `Storage` via an extension trait.
    pub fn fetched_entities(&self) -> &Entities {
        &self.entities
    }

    /// Tries to read the data associated with an `Entity`.
    pub fn get(&self, e: Entity) -> Option<&T> {
        let index = e.index();

        if self.data.mask().contains(index) && self.entities.is_alive(e) {
            Some(unsafe { self.data.storage().get(index) })
        } else {
            None
        }
    }

    /// Computes the number of elements this `Storage` contains by counting the
    /// bits in the bit set. This operation will never be performed in
    /// constant time.
    pub fn count(&self) -> usize {
        self.mask().iter().count()
    }

    /// Checks whether this `Storage` is empty. This operation is very cheap.
    pub fn is_empty(&self) -> bool {
        self.mask().is_empty()
    }

    /// Returns true if the storage has a component for this entity, and that
    /// entity is alive.
    pub fn contains(&self, e: Entity) -> bool {
        let index = e.index();

        self.data.mask().contains(index) && self.entities.is_alive(e)
    }

    /// Returns a reference to the bitset of this storage which allows filtering
    /// by the component type without actually getting the component.
    pub fn mask(&self) -> &BitSet {
        &self.data.mask()
    }

    pub fn not(&self) -> AntiStorage<'_> {
        AntiStorage(&self.data.mask())
    }
}

impl<'a, T, D> StorageWrapper<'a, T, D>
where
    T: Component,
    D: DerefMut<Target = MaskedStorage<T>>,
{
    /// Tries to mutate the data associated with an `Entity`.
    pub fn get_mut(&mut self, e: Entity) -> Option<&mut T> {
        let index = e.index();

        if self.data.mask().contains(index) && self.entities.is_alive(e) {
            Some(unsafe { self.data.storage_mut().get_mut(index) })
        } else {
            None
        }
    }

    /// Inserts new data for a given `Entity`.
    /// Returns the result of the operation as a `InsertResult<T>`
    ///
    /// If a component already existed for the given `Entity`, then it will
    /// be overwritten with the new component. If it did overwrite, then the
    /// result will contain `Some(T)` where `T` is the previous component.
    pub fn insert(&mut self, entity: Entity, component: T) -> Result<Option<T>, Error> {
        if !self.entities.is_alive(entity) {
            return Err(Error::EntityIsNotAlive(entity));
        }

        Ok(self.data.insert(entity, component))
    }

    /// Removes the data associated with an `Entity`.
    pub fn remove(&mut self, e: Entity) -> Option<T> {
        let index = e.index();
        if self.entities.is_alive(e) {
            self.data.remove(index)
        } else {
            None
        }
    }

    /// Clears the contents of the storage.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Creates a draining storage wrapper which can be `.join`ed
    /// to get a draining iterator.
    pub fn drain(&mut self) -> Drain<T> {
        Drain {
            data: &mut self.data,
        }
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

    unsafe fn open(self) -> (Self::Mask, Self::Value) {
        (self.data.mask(), self.data.storage())
    }

    unsafe fn get(v: &mut Self::Value, i: Index) -> &'a T {
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

    unsafe fn open(self) -> (Self::Mask, Self::Value) {
        (self.data.mask(), self.data.storage())
    }

    unsafe fn get(v: &mut Self::Value, i: Index) -> &'a mut T {
        let value: *mut T::Storage = *v as *const T::Storage as *mut T::Storage;

        (*value).get_mut(i)
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
