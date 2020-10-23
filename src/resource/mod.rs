pub mod cell;
pub mod entry;
pub mod resources;

pub use resources::{Ref, RefMut, Resources};

use std::any::TypeId;

use mopa::Any;

pub trait Resource: Any + Send + Sync + 'static {}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResourceId(TypeId);

impl ResourceId {
    pub fn new<R>() -> Self
    where
        R: Resource,
    {
        Self(TypeId::of::<R>())
    }
}

impl From<TypeId> for ResourceId {
    fn from(id: TypeId) -> Self {
        Self(id)
    }
}
