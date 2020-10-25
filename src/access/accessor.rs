use std::marker::PhantomData;
use std::ops::Deref;

use crate::{
    resource::ResourceId,
    system::{DynamicSystemData, SystemData, WithSystemData},
};

pub trait Accessor: Sized {
    fn reads(&self) -> Vec<ResourceId> {
        Vec::new()
    }

    fn writes(&self) -> Vec<ResourceId> {
        Vec::new()
    }

    fn try_new() -> Option<Self> {
        None
    }
}

#[derive(Default)]
pub struct StaticAccessor<T> {
    marker: PhantomData<fn() -> T>,
}

pub enum AccessorCow<'a, 'b, T>
where
    AccessorType<'a, T>: 'b,
    T: WithSystemData<'a> + ?Sized,
    'a: 'b,
{
    Borrow(&'b AccessorType<'a, T>),
    Owned(AccessorType<'a, T>),
}

pub type AccessorType<'a, T> =
    <<T as WithSystemData<'a>>::SystemData as DynamicSystemData<'a>>::Accessor;

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

/* AccessorCow */

impl<'a, 'b, T> Deref for AccessorCow<'a, 'b, T>
where
    AccessorType<'a, T>: 'b,
    T: WithSystemData<'a> + ?Sized + 'b,
    'a: 'b,
{
    type Target = AccessorType<'a, T>;

    fn deref(&self) -> &AccessorType<'a, T> {
        match *self {
            AccessorCow::Borrow(r) => &*r,
            AccessorCow::Owned(ref o) => o,
        }
    }
}
