use std::marker::PhantomData;
use std::ops::Deref;

use crate::resource::ResourceId;

use super::{DynamicSystemData, System, SystemData};

pub trait Accessor: Sized {
    fn try_new() -> Option<Self>;

    fn reads(&self) -> Vec<ResourceId>;

    fn writes(&self) -> Vec<ResourceId>;
}

pub enum AccessorCow<'a, 'b, T>
where
    AccessorType<'a, T>: 'b,
    T: System<'a> + ?Sized,
    'a: 'b,
{
    /// A reference to an accessor.
    Ref(&'b AccessorType<'a, T>),
    /// An owned accessor.
    Owned(AccessorType<'a, T>),
}

#[derive(Default)]
pub struct StaticAccessor<T> {
    marker: PhantomData<fn() -> T>,
}

pub type AccessorType<'a, T> = <<T as System<'a>>::SystemData as DynamicSystemData<'a>>::Accessor;

/* Accessor */

impl Accessor for () {
    fn try_new() -> Option<Self> {
        None
    }

    fn reads(&self) -> Vec<ResourceId> {
        Vec::new()
    }

    fn writes(&self) -> Vec<ResourceId> {
        Vec::new()
    }
}

impl<T> Accessor for PhantomData<T>
where
    T: ?Sized,
{
    fn try_new() -> Option<Self> {
        None
    }

    fn reads(&self) -> Vec<ResourceId> {
        Vec::new()
    }

    fn writes(&self) -> Vec<ResourceId> {
        Vec::new()
    }
}

/* AccessorCow */

impl<'a, 'b, T> Deref for AccessorCow<'a, 'b, T>
where
    AccessorType<'a, T>: 'b,
    T: System<'a> + ?Sized + 'b,
    'a: 'b,
{
    type Target = AccessorType<'a, T>;

    fn deref(&self) -> &AccessorType<'a, T> {
        match *self {
            AccessorCow::Ref(r) => &*r,
            AccessorCow::Owned(ref o) => o,
        }
    }
}

/* StaticAccessor */

impl<'a, T> Accessor for StaticAccessor<T>
where
    T: SystemData<'a>,
{
    fn try_new() -> Option<Self> {
        Some(StaticAccessor {
            marker: PhantomData,
        })
    }

    fn reads(&self) -> Vec<ResourceId> {
        T::reads()
    }

    fn writes(&self) -> Vec<ResourceId> {
        T::writes()
    }
}
