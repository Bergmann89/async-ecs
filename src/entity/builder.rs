use crate::{access::WriteStorage, component::Component, system::SystemData, world::World};

use super::Entity;

pub struct Builder<'a> {
    world: &'a World,
    entity: Entity,
    built: bool,
}

impl<'a> Builder<'a> {
    pub fn new(world: &'a World) -> Self {
        let entity = world.entities_mut().allocate();

        Self {
            world,
            entity,
            built: false,
        }
    }

    #[inline]
    pub fn with<T: Component>(self, c: T) -> Self {
        {
            let mut storage = WriteStorage::<T>::fetch(&self.world);

            storage.insert(self.entity, c).unwrap();
        }

        self
    }

    #[inline]
    pub fn build(mut self) -> Entity {
        self.built = true;

        self.entity
    }
}

impl Drop for Builder<'_> {
    fn drop(&mut self) {
        if !self.built {
            self.world.entities_mut().kill(self.entity);
        }
    }
}
