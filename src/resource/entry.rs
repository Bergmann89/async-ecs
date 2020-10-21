use std::marker::PhantomData;

use hashbrown::hash_map::{DefaultHashBuilder, Entry as HbEntry};

use super::{cell::Cell, RefMut, Resource, ResourceId};

pub struct Entry<'a, T: 'a> {
    inner: Inner<'a>,
    marker: PhantomData<T>,
}

pub type Inner<'a> = HbEntry<'a, ResourceId, Cell<Box<dyn Resource>>, DefaultHashBuilder>;

impl<'a, T> Entry<'a, T>
where
    T: Resource + 'a,
{
    pub fn new(inner: Inner<'a>) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }

    pub fn or_insert(self, v: T) -> RefMut<'a, T> {
        self.or_insert_with(move || v)
    }

    pub fn or_insert_with<F>(self, f: F) -> RefMut<'a, T>
    where
        F: FnOnce() -> T,
    {
        let inner = self.inner.or_insert_with(move || Cell::new(Box::new(f())));
        let inner = inner.borrow_mut().map(Box::as_mut);

        RefMut {
            inner,
            phantom: PhantomData,
        }
    }
}
