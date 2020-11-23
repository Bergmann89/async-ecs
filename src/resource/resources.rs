use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use hashbrown::HashMap;
use mopa::Any;

use super::{
    cell::{Cell, Ref as CellRef, RefMut as CellRefMut},
    entry::Entry,
    Resource, ResourceId,
};

#[derive(Default)]
pub struct Resources {
    resources: HashMap<ResourceId, Cell<Box<dyn Resource>>>,
}

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

/// A [Resource] container, which provides methods to insert, access and manage
/// the contained resources.
///
/// Many methods take `&self` which works because everything
/// is stored with **interior mutability**. In case you violate
/// the borrowing rules of Rust (multiple reads xor one write),
/// you will get a panic.
///
/// # Resource Ids
///
/// Resources are identified by `ResourceId`s, which consist of a `TypeId`.
impl Resources {
    /// Returns an entry for the resource with type `R`.
    pub fn entry<R>(&mut self) -> Entry<R>
    where
        R: Resource,
    {
        Entry::new(self.resources.entry(ResourceId::new::<R>()))
    }

    /// Inserts a resource into this container. If the resource existed before,
    /// it will be overwritten.
    ///
    /// # Examples
    ///
    /// Every type satisfying `Any + Send + Sync` automatically
    /// implements `Resource`, thus can be added:
    ///
    /// ```rust
    /// # #![allow(dead_code)]
    /// struct MyRes(i32);
    /// ```
    ///
    /// When you have a resource, simply insert it like this:
    ///
    /// ```rust
    /// # struct MyRes(i32);
    /// use async_ecs::World;
    ///
    /// let mut world = World::default();
    /// world.insert(MyRes(5));
    /// ```
    pub fn insert<R>(&mut self, r: R)
    where
        R: Resource,
    {
        self.resources
            .insert(ResourceId::new::<R>(), Cell::new(Box::new(r)));
    }

    /// Removes a resource of type `R` from this container and returns its
    /// ownership to the caller. In case there is no such resource in this,
    /// container, `None` will be returned.
    ///
    /// Use this method with caution; other functions and systems might assume
    /// this resource still exists. Thus, only use this if you're sure no
    /// system will try to access this resource after you removed it (or else
    /// you will get a panic).
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

    /// Returns true if the specified resource type `R` exists in `self`.
    pub fn contains<R>(&self) -> bool
    where
        R: Resource,
    {
        self.resources.contains_key(&ResourceId::new::<R>())
    }

    /// Fetches the resource with the specified type `T` or panics if it doesn't
    /// exist.
    ///
    /// # Panics
    ///
    /// Panics if the resource doesn't exist.
    /// Panics if the resource is being accessed mutably.
    pub fn borrow<R>(&self) -> Ref<R>
    where
        R: Resource,
    {
        self.try_borrow().unwrap_or_else(|| fetch_panic!())
    }

    /// Like `fetch`, but returns an `Option` instead of inserting a default
    /// value in case the resource does not exist.
    pub fn try_borrow<R>(&self) -> Option<Ref<R>>
    where
        R: Resource,
    {
        self.resources.get(&ResourceId::new::<R>()).map(|r| Ref {
            inner: CellRef::map(r.borrow(), Box::as_ref),
            phantom: PhantomData,
        })
    }

    /// Fetches the resource with the specified type `T` mutably.
    ///
    /// Please see `fetch` for details.
    ///
    /// # Panics
    ///
    /// Panics if the resource doesn't exist.
    /// Panics if the resource is already being accessed.
    pub fn borrow_mut<R>(&self) -> RefMut<R>
    where
        R: Resource,
    {
        self.try_borrow_mut().unwrap_or_else(|| fetch_panic!())
    }

    /// Like `fetch_mut`, but returns an `Option` instead of inserting a default
    /// value in case the resource does not exist.
    pub fn try_borrow_mut<R>(&self) -> Option<RefMut<R>>
    where
        R: Resource,
    {
        self.resources.get(&ResourceId::new::<R>()).map(|r| RefMut {
            inner: r.borrow_mut().map(Box::as_mut),
            phantom: PhantomData,
        })
    }

    /// Retrieves a resource without fetching, which is cheaper, but only
    /// available with `&mut self`.
    pub fn get_mut<R: Resource>(&mut self) -> Option<&mut R> {
        self.get_resource_mut(ResourceId::new::<R>())
            .map(|res| res.downcast_mut().unwrap())
    }

    /// Retrieves a resource without fetching, which is cheaper, but only
    /// available with `&mut self`.
    pub fn get_resource_mut(&mut self, id: ResourceId) -> Option<&mut dyn Resource> {
        self.resources
            .get_mut(&id)
            .map(Cell::get_mut)
            .map(Box::as_mut)
    }
}

/* Resource */

impl<T> Resource for T where T: Any + Send + Sync {}

mod __resource_mopafy_scope {
    #![allow(clippy::all)]

    use mopa::mopafy;

    use super::super::Resource;

    mopafy!(Resource);
}

/* Ref */

impl<'a, R> Ref<'a, R> {
    pub fn new(inner: CellRef<'a, dyn Resource>) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }
}

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

impl<'a, R> RefMut<'a, R> {
    pub fn new(inner: CellRefMut<'a, dyn Resource>) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }
}

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
