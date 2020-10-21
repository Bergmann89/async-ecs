use std::marker::PhantomData;

use crate::{entity::Entities, resource::Ref};

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
