pub mod cell;
pub mod entry;

use std::any::TypeId;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use hashbrown::HashMap;
use mopa::Any;

use cell::{Cell, Ref as CellRef, RefMut as CellRefMut};
use entry::Entry;

#[derive(Default)]
pub struct Resources {
    resources: HashMap<ResourceId, Cell<Box<dyn Resource>>>,
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct ResourceId(TypeId);

pub trait Resource: Any + Send + Sync + 'static {}

pub struct Ref<'a, R: 'a> {
    inner: CellRef<'a, dyn Resource>,
    phantom: PhantomData<&'a R>,
}

pub struct RefMut<'a, R: 'a> {
    inner: CellRefMut<'a, dyn Resource>,
    phantom: PhantomData<&'a R>,
}

macro_rules! fetch_panic {
    () => {{
        panic!(
            "\
            Tried to fetch resource from the resources map, but the resource does not exist.\n\
\n\
            Resource: `{resource_name_full}`\n\
\n\
            You may ensure the resource exists!\
            ",
            resource_name_full = std::any::type_name::<R>(),
        )
    }};
}

/* Resources */

impl Resources {
    pub fn entry<R>(&mut self) -> Entry<R>
    where
        R: Resource,
    {
        Entry::new(self.resources.entry(ResourceId::new::<R>()))
    }

    pub fn insert<R>(&mut self, r: R)
    where
        R: Resource,
    {
        self.resources
            .insert(ResourceId::new::<R>(), Cell::new(Box::new(r)));
    }

    pub fn remove<R>(&mut self) -> Option<R>
    where
        R: Resource,
    {
        self.resources
            .remove(&ResourceId::new::<R>())
            .map(Cell::into_inner)
            .map(|x: Box<dyn Resource>| x.downcast())
            .map(|x: Result<Box<R>, _>| x.ok().unwrap())
            .map(|x| *x)
    }

    pub fn contains<R>(&self) -> bool
    where
        R: Resource,
    {
        self.resources.contains_key(&ResourceId::new::<R>())
    }

    pub fn borrow<R>(&self) -> Ref<R>
    where
        R: Resource,
    {
        self.try_borrow().unwrap_or_else(|| fetch_panic!())
    }

    pub fn try_borrow<R>(&self) -> Option<Ref<R>>
    where
        R: Resource,
    {
        self.resources.get(&ResourceId::new::<R>()).map(|r| Ref {
            inner: CellRef::map(r.borrow(), Box::as_ref),
            phantom: PhantomData,
        })
    }

    pub fn borrow_mut<R>(&self) -> RefMut<R>
    where
        R: Resource,
    {
        self.try_borrow_mut().unwrap_or_else(|| fetch_panic!())
    }

    pub fn try_borrow_mut<R>(&self) -> Option<RefMut<R>>
    where
        R: Resource,
    {
        self.resources.get(&ResourceId::new::<R>()).map(|r| RefMut {
            inner: r.borrow_mut().map(Box::as_mut),
            phantom: PhantomData,
        })
    }

    pub fn get_mut<R: Resource>(&mut self) -> Option<&mut R> {
        self.get_resource_mut(ResourceId::new::<R>())
            .map(|res| res.downcast_mut().unwrap())
    }

    pub fn get_resource_mut(&mut self, id: ResourceId) -> Option<&mut dyn Resource> {
        self.resources
            .get_mut(&id)
            .map(Cell::get_mut)
            .map(Box::as_mut)
    }
}

/* ResourceId */

impl ResourceId {
    pub fn new<R>() -> Self
    where
        R: Resource,
    {
        Self(TypeId::of::<R>())
    }
}

/* Resource */

impl<T> Resource for T where T: Any + Send + Sync {}

mod __resource_mopafy_scope {
    #![allow(clippy::all)]

    use mopa::mopafy;

    use super::Resource;

    mopafy!(Resource);
}

/* Ref */

impl<'a, R> Deref for Ref<'a, R>
where
    R: Resource,
{
    type Target = R;

    fn deref(&self) -> &R {
        unsafe { self.inner.downcast_ref_unchecked() }
    }
}

impl<'a, R> Clone for Ref<'a, R> {
    fn clone(&self) -> Self {
        Ref {
            inner: self.inner.clone(),
            phantom: PhantomData,
        }
    }
}

/* RefMut */

impl<'a, R> Deref for RefMut<'a, R>
where
    R: Resource,
{
    type Target = R;

    fn deref(&self) -> &R {
        unsafe { self.inner.downcast_ref_unchecked() }
    }
}

impl<'a, R> DerefMut for RefMut<'a, R>
where
    R: Resource,
{
    fn deref_mut(&mut self) -> &mut R {
        unsafe { self.inner.downcast_mut_unchecked() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Res;

    #[test]
    fn insert() {
        struct Foo;

        let mut resources = Resources::default();
        resources.insert(Res);

        assert!(resources.contains::<Res>());
        assert!(!resources.contains::<Foo>());
    }

    #[test]
    #[should_panic(expected = "but it was already borrowed")]
    fn read_write_fails() {
        let mut resources = Resources::default();
        resources.insert(Res);

        let _read = resources.borrow::<Res>();
        let _write = resources.borrow_mut::<Res>();
    }

    #[test]
    #[should_panic(expected = "but it was already borrowed mutably")]
    fn write_read_fails() {
        let mut resources = Resources::default();
        resources.insert(Res);

        let _write = resources.borrow_mut::<Res>();
        let _read = resources.borrow::<Res>();
    }

    #[test]
    fn remove_insert() {
        let mut resources = Resources::default();
        resources.insert(Res);

        assert!(resources.contains::<Res>());

        resources.remove::<Res>().unwrap();

        assert!(!resources.contains::<Res>());

        resources.insert(Res);

        assert!(resources.contains::<Res>());
    }
}
