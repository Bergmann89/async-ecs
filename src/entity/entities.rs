use std::iter::Iterator;
use std::sync::atomic::{AtomicU32, Ordering};

use hibitset::{AtomicBitSet, BitSet, BitSetLike};
use thiserror::Error;

use crate::{
    access::WriteStorage,
    component::Component,
    join::{Join, ParJoin},
};

use super::{Entity, Index};

/// The entities of this ECS. This is a resource, stored in the `World`.
/// If you just want to access it in your system, you can also use
/// `Read<Entities>`.
///
/// **Please note that you should never get
/// this mutably in a system, because it would
/// block all the other systems.**
///
/// You need to call `World::maintain` after creating / deleting
/// entities with this struct.
#[derive(Default)]
pub struct Entities {
    alive: BitSet,
    raised: AtomicBitSet,
    killed: AtomicBitSet,

    cache: IndexCache,
    generations: Vec<u32>,
    max_index: AtomicU32,
}

impl Entities {
    /// Creates a new entity. This will be persistent after this call.
    pub fn allocate(&mut self) -> Entity {
        let index = self.cache.pop().unwrap_or_else(|| {
            let index = self.max_index.get_mut();
            *index = index.checked_add(1).expect("No entity left to allocate");

            *index
        });

        self.update_generations(index as usize);

        self.alive.add(index);

        let generation = &mut self.generations[index as usize];
        *generation = generation.wrapping_add(1);

        Entity::from_parts(index, *generation)
    }

    /// Creates a new entity atomically. This will be persistent as soon
    /// as you call `World::maintain`.
    ///
    /// If you want a lazy entity builder, take a look at `LazyUpdate::create_entity`.
    ///
    /// In case you have access to the `World`, you can also use `World::create_entity`
    /// which creates the entity and the components immediately.
    pub fn create(&self) -> Entity {
        let index = self.cache.pop_atomic().unwrap_or_else(|| {
            atomic_increment(&self.max_index).expect("No entity left to allocate") as Index
        });

        self.raised.add_atomic(index);

        let generation = self
            .generations
            .get(index as usize)
            .map(|g| g.wrapping_add(1))
            .unwrap_or_default();

        Entity::from_parts(index, generation)
    }

    /// Returns an iterator which creates new entities atomically.
    /// They will be persistent as soon as you call `World::maintain`.
    pub fn create_iter(&self) -> CreateIterAtomic {
        CreateIterAtomic(&self)
    }

    /// Similar to the `create` method above this creates an entity atomically,
    /// and then returns a builder which can be used to insert components into
    /// various storages if available.
    pub fn build_entity(&self) -> AtomicBuilder {
        let entity = self.create();

        AtomicBuilder {
            entities: self,
            entity,
            built: false,
        }
    }

    /// Kills a list of entities immediately.
    pub fn kill(&mut self, delete: &[Entity]) -> Result<(), Error> {
        for &entity in delete {
            if !self.is_alive(entity) {
                return Err(Error::EntityIsDead {
                    id: entity.id(),
                    op: "kill",
                });
            }

            let index = entity.index();

            self.alive.remove(index);
            self.killed.remove(index);

            self.update_generations(index as usize);

            if self.raised.remove(index) {
                let gen = &mut self.generations[index as usize];
                *gen = gen.wrapping_add(1);
            }
        }

        self.cache.extend(delete.iter().map(Entity::index));

        Ok(())
    }

    /// Deletes an entity atomically.
    /// The associated components will be deleted as soon as you call `World::maintain`.
    pub fn delete(&self, entity: Entity) -> Result<(), Error> {
        if !self.is_alive(entity) {
            return Err(Error::EntityIsDead {
                id: entity.id(),
                op: "delete",
            });
        }

        let index = entity.index();

        self.killed.add_atomic(index);

        Ok(())
    }

    /// Returns `true` if the specified entity is alive.
    #[inline]
    pub fn is_alive(&self, entity: Entity) -> bool {
        let idx = entity.index();
        let gen = entity.generation();

        match self.generations.get(idx as usize) {
            Some(g) if self.raised.contains(idx) => gen == g.wrapping_add(1),
            Some(g) => self.alive.contains(idx) && gen == *g,
            None if self.raised.contains(idx) => gen == 0,
            None => false,
        }
    }

    /// Maintains the allocated entities, mainly dealing with atomically
    /// allocated or killed entities.
    pub fn maintain(&mut self) -> Vec<Entity> {
        let mut deleted = vec![];

        let max_index = *self.max_index.get_mut();
        self.update_generations(max_index as usize + 1);

        for index in (&self.raised).iter() {
            let generation = &mut self.generations[index as usize];
            *generation = generation.wrapping_add(1);

            self.alive.add(index);
        }
        self.raised.clear();

        for index in (&self.killed).iter() {
            self.alive.remove(index);
            deleted.push(Entity::from_parts(index, self.generations[index as usize]));
        }

        self.cache.extend(deleted.iter().map(Entity::index));

        deleted
    }

    fn update_generations(&mut self, index: usize) {
        if self.generations.len() <= index {
            self.generations.resize(index + 1, 0);
        }
    }
}

impl<'a> Join for &'a Entities {
    type Mask = &'a BitSet;
    type Type = Entity;
    type Value = Self;

    unsafe fn open(self) -> (Self::Mask, Self) {
        (&self.alive, self)
    }

    unsafe fn get(v: &mut &'a Entities, index: Index) -> Entity {
        let generation = v
            .generations
            .get(index as usize)
            .copied()
            .unwrap_or_default();

        Entity::from_parts(index, generation)
    }
}

impl<'a> ParJoin for &'a Entities {}

/* Error */

#[derive(Debug, Error)]
pub enum Error {
    #[error("Entity is not alive (id = {id}, operation = {op})!")]
    EntityIsDead { id: u64, op: &'static str },
}

/* CreateIterAtomic */

pub struct CreateIterAtomic<'a>(&'a Entities);

impl<'a> Iterator for CreateIterAtomic<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Entity> {
        Some(self.0.create())
    }
}

/* AtomicBuilder */

/// An entity builder from `EntitiesRes`.  Allows building an entity with its
/// components if you have mutable access to the component storages.
pub struct AtomicBuilder<'a> {
    entities: &'a Entities,
    entity: Entity,
    built: bool,
}

impl<'a> AtomicBuilder<'a> {
    /// Appends a component and associates it with the entity.
    pub fn with<T: Component>(self, storage: &mut WriteStorage<T>, component: T) -> Self {
        storage.insert(self.entity, component).unwrap();

        self
    }

    /// Finishes the building and returns the entity.
    pub fn build(mut self) -> Entity {
        self.built = true;

        self.entity
    }
}

impl<'a> Drop for AtomicBuilder<'a> {
    fn drop(&mut self) {
        if !self.built {
            self.entities.delete(self.entity).unwrap();
        }
    }
}

/* IndexCache */

#[derive(Default, Debug)]
struct IndexCache {
    cache: Vec<Index>,
    len: AtomicU32,
}

impl IndexCache {
    fn push(&mut self, index: Index) {
        self.maintain();

        self.cache.push(index);

        *self.len.get_mut() = self.cache.len() as u32;
    }

    fn pop_atomic(&self) -> Option<Index> {
        atomic_decrement(&self.len).map(|x| self.cache[x as usize - 1])
    }

    fn pop(&mut self) -> Option<Index> {
        self.maintain();

        let x = self.cache.pop();

        *self.len.get_mut() = self.cache.len() as u32;

        x
    }

    fn maintain(&mut self) {
        self.cache.truncate(*self.len.get_mut() as usize);
    }
}

impl Extend<Index> for IndexCache {
    fn extend<T: IntoIterator<Item = Index>>(&mut self, iter: T) {
        self.maintain();

        self.cache.extend(iter);

        *self.len.get_mut() = self.cache.len() as u32;
    }
}

/// Increments `i` atomically without wrapping on overflow.
/// Resembles a `fetch_add(1, Ordering::Relaxed)` with
/// checked overflow, returning `None` instead.
fn atomic_increment(i: &AtomicU32) -> Option<u32> {
    let mut prev = i.load(Ordering::Relaxed);

    while prev != u32::MAX {
        match i.compare_exchange_weak(prev, prev + 1, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(x) => return Some(x),
            Err(next_prev) => prev = next_prev,
        }
    }

    None
}

/// Increments `i` atomically without wrapping on overflow.
/// Resembles a `fetch_sub(1, Ordering::Relaxed)` with
/// checked underflow, returning `None` instead.
fn atomic_decrement(i: &AtomicU32) -> Option<u32> {
    let mut prev = i.load(Ordering::Relaxed);

    while prev != 0 {
        match i.compare_exchange_weak(prev, prev - 1, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(x) => return Some(x),
            Err(next_prev) => prev = next_prev,
        }
    }

    None
}
