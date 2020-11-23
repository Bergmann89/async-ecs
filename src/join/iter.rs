use std::iter::Iterator;

use hibitset::{BitIter, BitSetLike};
use log::warn;

use crate::entity::{Entities, Entity};

use super::Join;

/// `JoinIter` is an `Iterator` over a group of `Storages`.
pub struct JoinIter<J: Join> {
    keys: BitIter<J::Mask>,
    values: J::Value,
}

impl<J: Join> JoinIter<J> {
    pub fn new(j: J) -> Self {
        if <J as Join>::is_unconstrained() {
            warn!(
                "`Join` possibly iterating through all indices, you might've made a join with all `MaybeJoin`s, which is unbounded in length."
            );
        }

        let (keys, values) = j.open();

        JoinIter {
            keys: keys.iter(),
            values,
        }
    }

    /// Allows getting joined values for specific entity.
    ///
    /// ## Example
    ///
    /// ```
    /// # use async_ecs::*;
    /// # #[derive(Debug, PartialEq)]
    /// # struct Pos; impl Component for Pos { type Storage = VecStorage<Self>; }
    /// # #[derive(Debug, PartialEq)]
    /// # struct Vel; impl Component for Vel { type Storage = VecStorage<Self>; }
    /// let mut world = World::default();
    ///
    /// world.register_component::<Pos>();
    /// world.register_component::<Vel>();
    ///
    /// // This entity could be stashed anywhere (into `Component`, `Resource`, `System`s data, etc.) as it's just a number.
    /// let entity = world
    ///     .create_entity()
    ///     .with(Pos)
    ///     .with(Vel)
    ///     .build();
    ///
    /// // Later
    /// {
    ///     let mut pos = world.component_mut::<Pos>();
    ///     let vel = world.component::<Vel>();
    ///
    ///     assert_eq!(
    ///         Some((&mut Pos, &Vel)),
    ///         (&mut pos, &vel).join().get(entity, &world.entities()),
    ///         "The entity that was stashed still has the needed components and is alive."
    ///     );
    /// }
    ///
    /// // The entity has found nice spot and doesn't need to move anymore.
    /// world.component_mut::<Vel>().remove(entity);
    ///
    /// // Even later
    /// {
    ///     let mut pos = world.component_mut::<Pos>();
    ///     let vel = world.component::<Vel>();
    ///
    ///     assert_eq!(
    ///         None,
    ///         (&mut pos, &vel).join().get(entity, &world.entities()),
    ///         "The entity doesn't have velocity anymore."
    ///     );
    /// }
    /// ```
    pub fn get(&mut self, entity: Entity, entities: &Entities) -> Option<J::Type> {
        if self.keys.contains(entity.index()) && entities.is_alive(entity) {
            Some(J::get(&mut self.values, entity.index()))
        } else {
            None
        }
    }
}

impl<J: Join> Iterator for JoinIter<J> {
    type Item = J::Type;

    fn next(&mut self) -> Option<J::Type> {
        self.keys.next().map(|idx| J::get(&mut self.values, idx))
    }
}

impl<J: Join> Clone for JoinIter<J>
where
    J::Mask: Clone,
    J::Value: Clone,
{
    fn clone(&self) -> Self {
        Self {
            keys: self.keys.clone(),
            values: self.values.clone(),
        }
    }
}
