use hibitset::BitSet;

use crate::join::{Join, ParJoin};

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
    cache: Vec<u32>,
    generations: Vec<u32>,
}

impl Entities {
    /// Returns `true` if the specified entity is alive.
    pub fn is_alive(&self, entity: Entity) -> bool {
        let i = entity.index();
        let g = entity.generation();

        self.alive.contains(i) && self.generations.get(i as usize) == Some(&g)
    }

    pub fn allocate(&mut self) -> Entity {
        let i = match self.cache.pop() {
            Some(i) => i,
            None => {
                let i = self.generations.len() as u32;
                let c = i.checked_add(1).expect("No entity left to allocate");

                self.generations.resize(c as usize, 0);

                i
            }
        };

        let g = self.generations[i as usize].wrapping_add(1);

        self.generations[i as usize] = g;
        self.alive.add(i);

        Entity::from_parts(i, g)
    }

    pub fn kill(&mut self, entity: Entity) -> bool {
        if self.is_alive(entity) {
            let i = entity.index();

            self.alive.remove(i);
            self.cache.push(i);

            true
        } else {
            false
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
