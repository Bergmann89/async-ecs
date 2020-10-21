use hibitset::BitSet;

use super::Entity;

#[derive(Default)]
pub struct Entities {
    alive: BitSet,
    cache: Vec<u32>,
    generations: Vec<u32>,
}

impl Entities {
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
