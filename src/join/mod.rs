mod impls;
mod iter;
mod maybe;
mod parallel;

pub use iter::JoinIter;
pub use maybe::MaybeJoin;
pub use parallel::JoinParIter;

use hibitset::BitSetLike;

use crate::entity::Index;

/// The purpose of the `Join` trait is to provide a way
/// to access multiple storages at the same time with
/// the merged bit set.
///
/// Joining component storages means that you'll only get values where
/// for a given entity every storage has an associated component.
///
/// ## Example
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
/// let mut world = World::default();
///
/// world.register_component::<Pos>();
/// world.register_component::<Vel>();
///
/// {
///     let pos = world.component::<Pos>();
///     let vel = world.component::<Vel>();
///
///     // There are no entities yet, so no pair will be returned.
///     let joined: Vec<_> = (&pos, &vel).join().collect();
///     assert_eq!(joined, vec![]);
/// }
///
/// world.create_entity().with(Pos).build();
///
/// {
///     let pos = world.component::<Pos>();
///     let vel = world.component::<Vel>();
///
///     // Although there is an entity, it only has `Pos`.
///     let joined: Vec<_> = (&pos, &vel).join().collect();
///     assert_eq!(joined, vec![]);
/// }
///
/// let ent = world.create_entity().with(Pos).with(Vel).build();
///
/// {
///     let pos = world.component::<Pos>();
///     let vel = world.component::<Vel>();
///
///     // Now there is one entity that has both a `Vel` and a `Pos`.
///     let joined: Vec<_> = (&pos, &vel).join().collect();
///     assert_eq!(joined, vec![(&Pos, &Vel)]);
///
///     // If we want to get the entity the components are associated to,
///     // we need to join over `Entities`:
///
///     let entities = world.entities();
///     // note: `Entities` is the fetched resource; we get back
///     // `Read<Entities>`.
///     // `Read<Entities>` can also be referred to by `Entities` which
///     // is a shorthand type definition to the former type.
///
///     let joined: Vec<_> = (&entities, &pos, &vel).join().collect();
///     assert_eq!(joined, vec![(ent, &Pos, &Vel)]);
/// }
/// ```
///
/// ## Iterating over a single storage
///
/// `Join` can also be used to iterate over a single
/// storage, just by writing `(&storage).join()`.
pub trait Join {
    /// Type of joined components.
    type Type;

    /// Type of joined storages.
    type Value;

    /// Type of joined bit mask.
    type Mask: BitSetLike;

    /// Create a joined iterator over the contents.
    fn join(self) -> JoinIter<Self>
    where
        Self: Sized,
    {
        JoinIter::new(self)
    }

    /// Returns a `Join`-able structure that yields all indices, returning
    /// `None` for all missing elements and `Some(T)` for found elements.
    ///
    /// WARNING: Do not have a join of only `MaybeJoin`s. Otherwise the join
    /// will iterate over every single index of the bitset. If you want a
    /// join with all `MaybeJoin`s, add an `Entities` to the join as well
    /// to bound the join to all entities that are alive.
    ///
    /// ```
    /// # use async_ecs::*;
    /// #
    /// # #[derive(Debug, PartialEq)]
    /// # struct Pos { x: i32, y: i32 }
    /// # impl Component for Pos { type Storage = VecStorage<Self>; }
    /// #
    /// # #[derive(Debug, PartialEq)]
    /// # struct Vel { x: i32, y: i32 }
    /// # impl Component for Vel { type Storage = VecStorage<Self>; }
    /// #
    /// struct ExampleSystem;
    ///
    /// impl<'a> System<'a> for ExampleSystem {
    ///     type SystemData = (WriteStorage<'a, Pos>, ReadStorage<'a, Vel>);
    ///
    ///     fn run(&mut self, (mut positions, velocities): Self::SystemData) {
    ///         for (mut position, maybe_velocity) in (&mut positions, velocities.maybe()).join() {
    ///             if let Some(velocity) = maybe_velocity {
    ///                 position.x += velocity.x;
    ///                 position.y += velocity.y;
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut world = World::default();
    ///     let mut dispatcher = Dispatcher::setup_builder(&mut world)
    ///         .with(ExampleSystem, "example_system", &[])
    ///         .unwrap()
    ///         .build();
    ///
    ///     let e1 = world
    ///         .create_entity()
    ///         .with(Pos { x: 0, y: 0 })
    ///         .with(Vel { x: 5, y: 2 })
    ///         .build();
    ///
    ///     let e2 = world.create_entity().with(Pos { x: 0, y: 0 }).build();
    ///
    ///     dispatcher.dispatch(&mut world).await;
    ///
    ///     let positions = world.component::<Pos>();
    ///     assert_eq!(positions.get(e1), Some(&Pos { x: 5, y: 2 }));
    ///     assert_eq!(positions.get(e2), Some(&Pos { x: 0, y: 0 }));
    /// }
    /// ```
    fn maybe(self) -> MaybeJoin<Self>
    where
        Self: Sized,
    {
        MaybeJoin(self)
    }

    /// Open this join by returning the mask and the storages.
    ///
    /// # Safety
    ///
    /// This is unsafe because implementations of this trait can permit
    /// the `Value` to be mutated independently of the `Mask`.
    /// If the `Mask` does not correctly report the status of the `Value`
    /// then illegal memory access can occur.
    unsafe fn open(self) -> (Self::Mask, Self::Value);

    /// Get a joined component value by a given index.
    ///
    /// # Safety
    ///
    /// * A call to `get` must be preceded by a check if `id` is part of
    ///   `Self::Mask`
    /// * The implementation of this method may use unsafe code, but has no
    ///   invariants to meet
    unsafe fn get(value: &mut Self::Value, index: Index) -> Self::Type;

    /// If this `Join` typically returns all indices in the mask, then iterating
    /// over only it or combined with other joins that are also dangerous
    /// will cause the `JoinIter`/`ParJoin` to go through all indices which
    /// is usually not what is wanted and will kill performance.
    #[inline]
    fn is_unconstrained() -> bool {
        false
    }
}

// SAFETY: This is safe as long as `T` implements `ParJoin` safely. `MaybeJoin`
// relies on `T as Join` for all storage access and safely wraps the inner
// `Join` API, so it should also be able to implement `ParJoin`.
pub trait ParJoin: Join {
    fn par_join(self) -> JoinParIter<Self>
    where
        Self: Sized,
    {
        if <Self as Join>::is_unconstrained() {
            log::warn!(
                "`ParJoin` possibly iterating through all indices, you might've made a join with all `MaybeJoin`s, which is unbounded in length."
            );
        }

        JoinParIter::new(self)
    }
}
