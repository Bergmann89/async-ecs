use std::ops::{Deref, DerefMut};

use crate::{component::Component, entity::Entities, resource::Resources, storage::MaskedStorage};

pub struct World(Resources);

impl World {
    pub fn register<T: Component>(&mut self)
    where
        T::Storage: Default,
    {
        self.register_with_storage::<T, _>(Default::default);
    }

    pub fn register_with_storage<T, F>(&mut self, storage: F)
    where
        T: Component,
        F: FnOnce() -> T::Storage,
    {
        self.entry()
            .or_insert_with(move || MaskedStorage::<T>::new(storage()));
    }
}

impl Default for World {
    fn default() -> Self {
        let mut resources = Resources::default();

        resources.insert(Entities::default());

        Self(resources)
    }
}

impl Deref for World {
    type Target = Resources;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for World {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
