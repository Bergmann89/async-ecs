use std::marker::PhantomData;

use hashbrown::hash_map::{DefaultHashBuilder, Entry as HbEntry};

use super::{cell::Cell, RefMut, Resource, ResourceId};

pub struct Entry<'a, T: 'a> {
    inner: Inner<'a>,
    marker: PhantomData<T>,
}

pub type Inner<'a> = HbEntry<'a, ResourceId, Cell<Box<dyn Resource>>, DefaultHashBuilder>;

/// An entry to a resource container.
/// This is similar to the Entry API found in the standard library.
///
/// ## Examples
///
/// ```
/// use async_ecs::World;
///
/// #[derive(Debug)]
/// struct Res(i32);
///
/// let mut world = World::default();
///
/// let value = world.entry().or_insert(Res(4));
/// println!("{:?}", value.0 * 2);
/// ```
impl<'a, T> Entry<'a, T>
where
    T: Resource + 'a,
{
    /// Create new entry.
    pub fn new(inner: Inner<'a>) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }

    /// Returns this entry's value, inserts and returns `v` otherwise.
    ///
    /// Please note that you should use `or_insert_with` in case the creation of
    /// the value is expensive.
    pub fn or_insert(self, v: T) -> RefMut<'a, T> {
        self.or_insert_with(move || v)
    }

    /// Returns this entry's value, inserts and returns the return value of `f`
    /// otherwise.
    pub fn or_insert_with<F>(self, f: F) -> RefMut<'a, T>
    where
        F: FnOnce() -> T,
    {
        let inner = self.inner.or_insert_with(move || Cell::new(Box::new(f())));
        let inner = inner.borrow_mut().map(Box::as_mut);

        RefMut::new(inner)
    }
}
