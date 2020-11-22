use std::iter::Iterator;

use hibitset::{BitIter, BitSetLike};
use log::warn;

use crate::entity::{Entities, Entity};

use super::Join;

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
