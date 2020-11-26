use std::sync::Arc;

use crossbeam_queue::SegQueue;
use futures::future::BoxFuture;
use log::warn;

use crate::{
    access::WriteStorage,
    component::Component,
    entity::{Builder, Entity},
    system::SystemData,
};

use super::World;

/// Lazy updates can be used for world updates that need to borrow a lot of resources
/// and as such should better be done at the end. They work lazily in the sense that
/// they are dispatched when calling `world.maintain()`.
///
/// Lazy updates are dispatched in the order that they are requested. Multiple updates
/// sent from one system may be overridden by updates sent from other systems.
///
/// Please note that the provided methods take `&self` so there's no need to get
/// `Lazy` mutably. This resource is added to the world by default.
pub struct Lazy {
    queue: Arc<SegQueue<LazyUpdate>>,
}

impl Lazy {
    /// Lazily executes a closure with world access.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use async_ecs::*;
    /// #
    /// struct Pos;
    ///
    /// impl Component for Pos {
    ///     type Storage = VecStorage<Self>;
    /// }
    ///
    /// struct Execution;
    ///
    /// impl<'a> System<'a> for Execution {
    ///     type SystemData = (Entities<'a>, Read<'a, Lazy>);
    ///
    ///     fn run(&mut self, (ent, lazy): Self::SystemData) {
    ///         for entity in ent.join() {
    ///             lazy.exec(move |world| {
    ///                 if world.is_alive(entity) {
    ///                     println!("Entity {:?} is alive.", entity);
    ///                 }
    ///             });
    ///         }
    ///     }
    /// }
    /// ```
    pub fn exec<F>(&self, f: F)
    where
        F: FnOnce(&mut World) + Send + Sync + 'static,
    {
        self.queue.push(LazyUpdate::Sync(Box::new(f)));
    }

    /// Same as `Lazy::exec` but with async response.
    pub fn exec_async<F>(&self, f: F)
    where
        F: FnOnce(&mut World) -> BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        self.queue.push(LazyUpdate::Async(Box::new(f)));
    }

    /// Lazily inserts a component for an entity.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use async_ecs::*;
    /// #
    /// struct Pos(f32, f32);
    ///
    /// impl Component for Pos {
    ///     type Storage = VecStorage<Self>;
    /// }
    ///
    /// struct InsertPos;
    ///
    /// impl<'a> System<'a> for InsertPos {
    ///     type SystemData = (Entities<'a>, Read<'a, Lazy>);
    ///
    ///     fn run(&mut self, (ent, lazy): Self::SystemData) {
    ///         let a = ent.create();
    ///         lazy.insert(a, Pos(1.0, 1.0));
    ///     }
    /// }
    /// ```
    pub fn insert<C>(&self, e: Entity, c: C)
    where
        C: Component + Send + Sync,
    {
        self.exec(move |world| {
            let mut storage: WriteStorage<C> = SystemData::fetch(world);

            if storage.insert(e, c).is_err() {
                warn!("Lazy insert of component failed because {:?} was dead.", e);
            }
        });
    }

    /// Lazily inserts components for entities.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use async_ecs::*;
    /// #
    /// struct Pos(f32, f32);
    ///
    /// impl Component for Pos {
    ///     type Storage = VecStorage<Self>;
    /// }
    ///
    /// struct InsertPos;
    ///
    /// impl<'a> System<'a> for InsertPos {
    ///     type SystemData = (Entities<'a>, Read<'a, Lazy>);
    ///
    ///     fn run(&mut self, (ent, lazy): Self::SystemData) {
    ///         let a = ent.create();
    ///         let b = ent.create();
    ///
    ///         lazy.insert_many(vec![(a, Pos(3.0, 1.0)), (b, Pos(0.0, 4.0))]);
    ///     }
    /// }
    /// ```
    pub fn insert_many<C, I>(&self, iter: I)
    where
        C: Component + Send + Sync,
        I: IntoIterator<Item = (Entity, C)> + Send + Sync + 'static,
    {
        self.exec(move |world| {
            let mut storage: WriteStorage<C> = SystemData::fetch(world);

            for (e, c) in iter {
                if storage.insert(e, c).is_err() {
                    log::warn!("Lazy insert of component failed because {:?} was dead.", e);
                }
            }
        });
    }

    /// Lazily removes a component.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use async_ecs::*;
    /// #
    /// struct Pos;
    ///
    /// impl Component for Pos {
    ///     type Storage = VecStorage<Self>;
    /// }
    ///
    /// struct RemovePos;
    ///
    /// impl<'a> System<'a> for RemovePos {
    ///     type SystemData = (Entities<'a>, Read<'a, Lazy>);
    ///
    ///     fn run(&mut self, (ent, lazy): Self::SystemData) {
    ///         for entity in ent.join() {
    ///             lazy.remove::<Pos>(entity);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn remove<C>(&self, e: Entity)
    where
        C: Component,
    {
        self.exec(move |world| {
            let mut storage: WriteStorage<C> = SystemData::fetch(world);

            storage.remove(e);
        });
    }

    /// Lazily removes a component.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use async_ecs::*;
    /// #
    /// struct Pos;
    ///
    /// impl Component for Pos {
    ///     type Storage = VecStorage<Self>;
    /// }
    ///
    /// struct RemovePos;
    ///
    /// impl<'a> System<'a> for RemovePos {
    ///     type SystemData = (Entities<'a>, Read<'a, Lazy>);
    ///
    ///     fn run(&mut self, (ent, lazy): Self::SystemData) {
    ///         for entity in ent.join() {
    ///             lazy.remove::<Pos>(entity);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn remove_many<C, I>(&self, iter: I)
    where
        C: Component,
        I: IntoIterator<Item = Entity> + Send + Sync + 'static,
    {
        self.exec(move |world| {
            let mut storage: WriteStorage<C> = SystemData::fetch(world);

            for e in iter {
                storage.remove(e);
            }
        });
    }

    /// Creates a new `LazyBuilder` which inserts components
    /// using `Lazy`. This means that the components won't
    /// be available immediately, but only after a `maintain`
    /// on `World` is performed.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use async_ecs::*;
    /// # let mut world = World::default();
    /// struct Pos(f32, f32);
    ///
    /// impl Component for Pos {
    ///     type Storage = VecStorage<Self>;
    /// }
    ///
    /// # let lazy = world.resource::<Lazy>();
    /// let my_entity = lazy.create_entity(&world).with(Pos(1.0, 3.0)).build();
    /// ```
    pub fn create_entity(&self, world: &World) -> LazyBuilder {
        let entity = world.entities().create();

        LazyBuilder { entity, lazy: self }
    }

    /// Executes all stored lazy updates
    pub async fn maintain(&self, world: &mut World) {
        while let Some(update) = self.queue.pop() {
            match update {
                LazyUpdate::Sync(update) => update(world),
                LazyUpdate::Async(update) => update(world).await,
            }
        }
    }
}

impl Default for Lazy {
    fn default() -> Self {
        Self {
            queue: Arc::new(SegQueue::new()),
        }
    }
}

impl Clone for Lazy {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
        }
    }
}

enum LazyUpdate {
    Sync(Box<dyn FnOnce(&mut World) + Send + Sync + 'static>),
    Async(Box<dyn FnOnce(&mut World) -> BoxFuture<'static, ()> + Send + Sync + 'static>),
}

/* LazyBuilder */

/// Like `EntityBuilder`, but inserts the component
/// lazily, meaning on `maintain`.
/// If you need those components to exist immediately,
/// you have to insert them into the storages yourself.
pub struct LazyBuilder<'a> {
    pub entity: Entity,
    pub lazy: &'a Lazy,
}

impl<'a> Builder for LazyBuilder<'a> {
    /// Inserts a component using [Lazy].
    ///
    /// If a component was already associated with the entity, it will
    /// overwrite the previous component.
    fn with<C>(self, component: C) -> Self
    where
        C: Component + Send + Sync,
    {
        let entity = self.entity;
        self.lazy.exec(move |world| {
            let mut storage: WriteStorage<C> = SystemData::fetch(world);

            if storage.insert(entity, component).is_err() {
                warn!(
                    "Lazy insert of component failed because {:?} was dead.",
                    entity
                );
            }
        });

        self
    }

    /// Finishes the building and returns the built entity.
    /// Please note that no component is associated to this
    /// entity until you call [`World::maintain`].
    fn build(self) -> Entity {
        self.entity
    }
}
