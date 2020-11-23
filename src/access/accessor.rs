use std::marker::PhantomData;
use std::ops::Deref;

use crate::{
    resource::ResourceId,
    system::{DynamicSystemData, SystemData},
};

/// A trait for accessing read/write multiple resources from a system. This can
/// be used to create dynamic systems that don't specify what they fetch at
/// compile-time.
///
/// For compile-time system data this will all be done for you using
/// `StaticAccessor`.
pub trait Accessor: Sized {
    /// A list of [`ResourceId`]s the bundle needs read access to in order to
    /// build the target resource bundle.
    ///
    /// # Contract
    ///
    /// Exactly return the dependencies you're going to `fetch`! Doing otherwise
    /// *will* cause a panic.
    ///
    /// This method is only executed once, thus the returned value may never change
    /// (otherwise it has no effect).
    ///
    /// [`ResourceId`]: struct.ResourceId.html
    fn reads(&self) -> Vec<ResourceId> {
        Vec::new()
    }

    /// A list of [`ResourceId`]s the bundle needs write access to in order to
    /// build the target resource bundle.
    ///
    /// # Contract
    ///
    /// Exactly return the dependencies you're going to `fetch`! Doing otherwise
    /// *will* cause a panic.
    ///
    /// This method is only executed once, thus the returned value may never change
    /// (otherwise it has no effect).
    ///
    /// [`ResourceId`]: struct.ResourceId.html
    fn writes(&self) -> Vec<ResourceId> {
        Vec::new()
    }

    /// Tries to create a new instance of this type. This one returns `Some` in
    /// case there is a default, otherwise the system needs to override
    /// `System::accessor`.
    fn try_new() -> Option<Self> {
        None
    }
}

pub type AccessorType<'a, T> = <T as DynamicSystemData<'a>>::Accessor;

/// The static accessor that is used for `SystemData`.
#[derive(Default)]
pub struct StaticAccessor<T> {
    marker: PhantomData<fn() -> T>,
}

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

pub enum AccessorCow<'a, 'b, T>
where
    T: DynamicSystemData<'a> + ?Sized,
    T::Accessor: 'b,
    'a: 'b,
{
    Borrow(&'b T::Accessor),
    Owned(T::Accessor),
}

impl<'a, 'b, T> Deref for AccessorCow<'a, 'b, T>
where
    AccessorType<'a, T>: 'b,
    T: DynamicSystemData<'a> + ?Sized + 'b,
    'a: 'b,
{
    type Target = T::Accessor;

    fn deref(&self) -> &T::Accessor {
        match *self {
            AccessorCow::Borrow(r) => &*r,
            AccessorCow::Owned(ref o) => o,
        }
    }
}
