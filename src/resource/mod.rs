pub mod cell;
pub mod entry;
pub mod resources;

pub use cell::Cell;
pub use resources::{Ref, RefMut, Resources};

use std::any::TypeId;

use mopa::Any;

/// A resource is a data slot which lives in the `World` can only be accessed
/// according to Rust's typical borrowing model (one writer xor multiple
/// readers).
pub trait Resource: Any + Send + Sync + 'static {}

/// The id of a [`Resource`], which simply wraps a type id and a "dynamic ID".
/// The "dynamic ID" is usually just left `0`, and, unless such documentation
/// says otherwise, other libraries will assume that it is always `0`; non-zero
/// IDs are only used for special resource types that are specifically defined
/// in a more dynamic way, such that resource types can essentially be created
/// at run time, without having different static types.
///
/// [`Resource`]: trait.Resource.html
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResourceId(TypeId);

impl ResourceId {
    /// Creates a new resource id from a given type.
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
