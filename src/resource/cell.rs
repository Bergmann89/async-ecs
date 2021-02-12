use std::cell::UnsafeCell;
use std::mem::forget;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicUsize, Ordering};

macro_rules! borrow_panic {
    ($s:expr) => {{
        panic!(
            "Tried to fetch data of type {:?}, but it was already borrowed{}.",
            ::std::any::type_name::<T>(),
            $s,
        )
    }};
}

/// A custom cell container that is a `RefCell` with thread-safety.
#[derive(Debug)]
pub struct Cell<T> {
    flag: AtomicUsize,
    inner: UnsafeCell<T>,
}

impl<T> Cell<T> {
    /// Create a new cell, similar to `RefCell::new`
    pub fn new(inner: T) -> Self {
        Cell {
            flag: AtomicUsize::new(0),
            inner: UnsafeCell::new(inner),
        }
    }

    /// Consumes this cell and returns ownership of `T`.
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }

    /// Get an immutable reference to the inner data.
    ///
    /// Absence of write accesses is checked at run-time.
    ///
    /// # Panics
    ///
    /// This function will panic if there is a mutable reference to the data
    /// already in use.
    pub fn borrow(&self) -> Ref<T> {
        if !self.check_flag_read() {
            borrow_panic!(" mutably");
        }

        Ref {
            flag: &self.flag,
            value: unsafe { &*self.inner.get() },
        }
    }

    /// Get an immutable reference to the inner data.
    ///
    /// Absence of write accesses is checked at run-time. If access is not
    /// possible, an error is returned.
    pub fn try_borrow(&self) -> Option<Ref<T>> {
        if self.check_flag_read() {
            Some(Ref {
                flag: &self.flag,
                value: unsafe { &*self.inner.get() },
            })
        } else {
            None
        }
    }

    /// Get a mutable reference to the inner data.
    ///
    /// Exclusive access is checked at run-time.
    ///
    /// # Panics
    ///
    /// This function will panic if there are any references to the data already
    /// in use.
    pub fn borrow_mut(&self) -> RefMut<T> {
        if !self.check_flag_write() {
            borrow_panic!("");
        }

        RefMut {
            flag: &self.flag,
            value: unsafe { &mut *self.inner.get() },
        }
    }

    /// Get a mutable reference to the inner data.
    ///
    /// Exclusive access is checked at run-time. If access is not possible, an
    /// error is returned.
    pub fn try_borrow_mut(&self) -> Option<RefMut<T>> {
        if self.check_flag_write() {
            Some(RefMut {
                flag: &self.flag,
                value: unsafe { &mut *self.inner.get() },
            })
        } else {
            None
        }
    }

    /// Gets exclusive access to the inner value, bypassing the Cell.
    ///
    /// Exclusive access is checked at compile time.
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.inner.get() }
    }

    /// Make sure we are allowed to aquire a read lock, and increment the read
    /// count by 1
    fn check_flag_read(&self) -> bool {
        loop {
            let val = self.flag.load(Ordering::Acquire);

            if val == usize::MAX {
                return false;
            }

            if self
                .flag
                .compare_exchange(val, val + 1, Ordering::AcqRel, Ordering::Acquire)
                == Ok(val)
            {
                return true;
            }
        }
    }

    /// Make sure we are allowed to aquire a write lock, and then set the write
    /// lock flag.
    fn check_flag_write(&self) -> bool {
        self.flag
            .compare_exchange(0, usize::MAX, Ordering::AcqRel, Ordering::Acquire)
            == Ok(0)
    }
}

unsafe impl<T> Sync for Cell<T> where T: Sync {}

/// An immutable reference to data in a `Cell`.
///
/// Access the value via `std::ops::Deref` (e.g. `*val`)
#[derive(Debug)]
pub struct Ref<'a, T>
where
    T: ?Sized + 'a,
{
    flag: &'a AtomicUsize,
    value: &'a T,
}

impl<'a, T> Ref<'a, T>
where
    T: ?Sized,
{
    /// Makes a new `Ref` for a component of the borrowed data which preserves
    /// the existing borrow.
    ///
    /// The `Cell` is already immutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as `Ref::map(...)`.
    /// A method would interfere with methods of the same name on the contents
    /// of a `Ref` used through `Deref`. Further this preserves the borrow of
    /// the value and hence does the proper cleanup when it's dropped.
    ///
    /// # Examples
    ///
    /// This can be used to avoid pointer indirection when a boxed item is
    /// stored in the `Cell`.
    ///
    /// ```
    /// use async_ecs::resource::cell::*;
    ///
    /// let cb = Cell::new(Box::new(5));
    ///
    /// // Borrowing the cell causes the `Ref` to store a reference to the `Box`, which is a
    /// // pointer to the value on the heap, not the actual value.
    /// let boxed_ref: Ref<'_, Box<usize>> = cb.borrow();
    /// assert_eq!(**boxed_ref, 5); // Notice the double deref to get the actual value.
    ///
    /// // By using `map` we can let `Ref` store a reference directly to the value on the heap.
    /// let pure_ref: Ref<'_, usize> = Ref::map(boxed_ref, Box::as_ref);
    ///
    /// assert_eq!(*pure_ref, 5);
    /// ```
    ///
    /// We can also use `map` to get a reference to a sub-part of the borrowed
    /// value.
    ///
    /// ```rust
    /// # use async_ecs::resource::cell::*;
    ///
    /// let c = Cell::new((5, 'b'));
    /// let b1: Ref<'_, (u32, char)> = c.borrow();
    /// let b2: Ref<'_, u32> = Ref::map(b1, |t| &t.0);
    /// assert_eq!(*b2, 5);
    /// ```
    pub fn map<U, F>(self, f: F) -> Ref<'a, U>
    where
        F: FnOnce(&T) -> &U,
        U: ?Sized,
    {
        let flag = unsafe { &*(self.flag as *const _) };
        let value = unsafe { &*(self.value as *const _) };

        forget(self);

        Ref {
            flag,
            value: f(value),
        }
    }
}

impl<'a, T> Deref for Ref<'a, T>
where
    T: ?Sized,
{
    type Target = T;

    fn deref(&self) -> &T {
        self.value
    }
}

impl<'a, T> Drop for Ref<'a, T>
where
    T: ?Sized,
{
    fn drop(&mut self) {
        self.flag.fetch_sub(1, Ordering::Release);
    }
}

impl<'a, T> Clone for Ref<'a, T>
where
    T: ?Sized,
{
    fn clone(&self) -> Self {
        self.flag.fetch_add(1, Ordering::Release);

        Ref {
            flag: self.flag,
            value: self.value,
        }
    }
}

/// A mutable reference to data in a `Cell`.
///
/// Access the value via `std::ops::DerefMut` (e.g. `*val`)
#[derive(Debug)]
pub struct RefMut<'a, T>
where
    T: ?Sized + 'a,
{
    flag: &'a AtomicUsize,
    value: &'a mut T,
}

impl<'a, T> RefMut<'a, T>
where
    T: ?Sized,
{
    /// Makes a new `RefMut` for a component of the borrowed data which
    /// preserves the existing borrow.
    ///
    /// The `Cell` is already mutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `RefMut::map(...)`. A method would interfere with methods of the
    /// same name on the contents of a `RefMut` used through `DerefMut`.
    /// Further this preserves the borrow of the value and hence does the
    /// proper cleanup when it's dropped.
    ///
    /// # Examples
    ///
    /// This can also be used to avoid pointer indirection when a boxed item is
    /// stored in the `Cell`.
    ///
    /// ```
    /// use async_ecs::resource::cell::*;
    ///
    /// let cb = Cell::new(Box::new(5));
    ///
    /// // Borrowing the cell causes the `RefMut` to store a reference to the `Box`, which is a
    /// // pointer to the value on the heap, and not a reference directly to the value.
    /// let boxed_ref: RefMut<'_, Box<usize>> = cb.borrow_mut();
    /// assert_eq!(**boxed_ref, 5); // Notice the double deref to get the actual value.
    ///
    /// // By using `map` we can let `RefMut` store a reference directly to the value on the heap.
    /// let pure_ref: RefMut<'_, usize> = RefMut::map(boxed_ref, Box::as_mut);
    ///
    /// assert_eq!(*pure_ref, 5);
    /// ```
    ///
    /// We can also use `map` to get a reference to a sub-part of the borrowed
    /// value.
    ///
    /// ```rust
    /// # use async_ecs::resource::cell::*;
    ///
    /// let c = Cell::new((5, 'b'));
    ///
    /// let b1: RefMut<'_, (u32, char)> = c.borrow_mut();
    /// let b2: RefMut<'_, u32> = RefMut::map(b1, |t| &mut t.0);
    /// assert_eq!(*b2, 5);
    /// ```
    pub fn map<U, F>(self, f: F) -> RefMut<'a, U>
    where
        F: FnOnce(&mut T) -> &mut U,
        U: ?Sized,
    {
        let flag = unsafe { &*(self.flag as *const _) };
        let value = unsafe { &mut *(self.value as *mut _) };

        forget(self);

        RefMut {
            flag,
            value: f(value),
        }
    }
}

impl<'a, T> Deref for RefMut<'a, T>
where
    T: ?Sized,
{
    type Target = T;

    fn deref(&self) -> &T {
        self.value
    }
}

impl<'a, T> DerefMut for RefMut<'a, T>
where
    T: ?Sized,
{
    fn deref_mut(&mut self) -> &mut T {
        self.value
    }
}

impl<'a, T> Drop for RefMut<'a, T>
where
    T: ?Sized,
{
    fn drop(&mut self) {
        self.flag.store(0, Ordering::Release)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allow_multiple_reads() {
        let cell = Cell::new(5);

        let a = cell.borrow();
        let b = cell.borrow();

        assert_eq!(10, *a + *b);
    }

    #[test]
    fn allow_clone_reads() {
        let cell = Cell::new(5);

        let a = cell.borrow();
        let b = a.clone();

        assert_eq!(10, *a + *b);
    }

    #[test]
    fn allow_single_write() {
        let cell = Cell::new(5);

        {
            let mut a = cell.borrow_mut();
            *a += 2;
            *a += 3;
        }

        assert_eq!(10, *cell.borrow());
    }

    #[test]
    #[should_panic(expected = "but it was already borrowed mutably")]
    fn panic_write_and_read() {
        let cell = Cell::new(5);

        let mut a = cell.borrow_mut();
        *a = 7;

        assert_eq!(7, *cell.borrow());
    }

    #[test]
    #[should_panic(expected = "but it was already borrowed")]
    fn panic_write_and_write() {
        let cell = Cell::new(5);

        let mut a = cell.borrow_mut();
        *a = 7;

        assert_eq!(7, *cell.borrow_mut());
    }

    #[test]
    #[should_panic(expected = "Tried to fetch data of type \"i32\", but it was already borrowed.")]
    fn panic_read_and_write() {
        let cell = Cell::new(5);

        let _a = cell.borrow();

        assert_eq!(7, *cell.borrow_mut());
    }

    #[test]
    fn try_write_and_read() {
        let cell = Cell::new(5);

        let mut a = cell.try_borrow_mut().unwrap();
        *a = 7;

        assert!(cell.try_borrow().is_none());

        *a = 8;
    }

    #[test]
    fn try_write_and_write() {
        let cell = Cell::new(5);

        let mut a = cell.try_borrow_mut().unwrap();
        *a = 7;

        assert!(cell.try_borrow_mut().is_none());

        *a = 8;
    }

    #[test]
    fn try_read_and_write() {
        let cell = Cell::new(5);

        let _a = cell.try_borrow().unwrap();

        assert!(cell.try_borrow_mut().is_none());
    }

    #[test]
    fn cloned_borrow_does_not_allow_write() {
        let cell = Cell::new(5);

        let a = cell.borrow();
        let b = a.clone();

        drop(a);

        assert!(cell.try_borrow_mut().is_none());
        assert_eq!(5, *b);
    }

    #[test]
    fn ref_with_non_sized() {
        let r: Ref<'_, [i32]> = Ref {
            flag: &AtomicUsize::new(1),
            value: &[2, 3, 4, 5][..],
        };

        assert_eq!(&*r, &[2, 3, 4, 5][..]);
    }

    #[test]
    fn ref_with_non_sized_clone() {
        let r: Ref<'_, [i32]> = Ref {
            flag: &AtomicUsize::new(1),
            value: &[2, 3, 4, 5][..],
        };
        let rr = r.clone();

        assert_eq!(&*r, &[2, 3, 4, 5][..]);
        assert_eq!(r.flag.load(Ordering::SeqCst), 2);

        assert_eq!(&*rr, &[2, 3, 4, 5][..]);
        assert_eq!(rr.flag.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn ref_with_trait_obj() {
        let ra: Ref<'_, dyn std::any::Any> = Ref {
            flag: &AtomicUsize::new(1),
            value: &2i32,
        };

        assert_eq!(ra.downcast_ref::<i32>().unwrap(), &2i32);
    }

    #[test]
    fn ref_mut_with_non_sized() {
        let mut r: RefMut<'_, [i32]> = RefMut {
            flag: &AtomicUsize::new(1),
            value: &mut [2, 3, 4, 5][..],
        };

        assert_eq!(&mut *r, &mut [2, 3, 4, 5][..]);
    }

    #[test]
    fn ref_mut_with_trait_obj() {
        let mut ra: RefMut<'_, dyn std::any::Any> = RefMut {
            flag: &AtomicUsize::new(1),
            value: &mut 2i32,
        };

        assert_eq!(ra.downcast_mut::<i32>().unwrap(), &mut 2i32);
    }

    #[test]
    fn ref_map_box() {
        let cell = Cell::new(Box::new(10));

        let r: Ref<'_, Box<usize>> = cell.borrow();
        assert_eq!(&**r, &10);

        let rr: Ref<'_, usize> = cell.borrow().map(Box::as_ref);
        assert_eq!(&*rr, &10);
    }

    #[test]
    fn ref_map_preserves_flag() {
        let cell = Cell::new(Box::new(10));

        let r: Ref<'_, Box<usize>> = cell.borrow();
        assert_eq!(cell.flag.load(Ordering::SeqCst), 1);
        let _nr: Ref<'_, usize> = r.map(Box::as_ref);
        assert_eq!(cell.flag.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn ref_map_retains_borrow() {
        let cell = Cell::new(Box::new(10));

        let _r: Ref<'_, usize> = cell.borrow().map(Box::as_ref);
        assert_eq!(cell.flag.load(Ordering::SeqCst), 1);

        let _rr: Ref<'_, usize> = cell.borrow().map(Box::as_ref);
        assert_eq!(cell.flag.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn ref_map_drops_borrow() {
        let cell = Cell::new(Box::new(10));

        let r: Ref<'_, usize> = cell.borrow().map(Box::as_ref);

        assert_eq!(cell.flag.load(Ordering::SeqCst), 1);
        drop(r);
        assert_eq!(cell.flag.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn ref_mut_map_box() {
        let cell = Cell::new(Box::new(10));

        {
            let mut r: RefMut<'_, Box<usize>> = cell.borrow_mut();
            assert_eq!(&mut **r, &mut 10);
        }
        {
            let mut rr: RefMut<'_, usize> = cell.borrow_mut().map(Box::as_mut);
            assert_eq!(&mut *rr, &mut 10);
        }
    }

    #[test]
    fn ref_mut_map_preserves_flag() {
        let cell = Cell::new(Box::new(10));

        let r: RefMut<'_, Box<usize>> = cell.borrow_mut();
        assert_eq!(cell.flag.load(Ordering::SeqCst), std::usize::MAX);
        let _nr: RefMut<'_, usize> = r.map(Box::as_mut);
        assert_eq!(cell.flag.load(Ordering::SeqCst), std::usize::MAX);
    }

    #[test]
    #[should_panic(
        expected = "Tried to fetch data of type \"alloc::boxed::Box<usize>\", but it was already borrowed."
    )]
    fn ref_mut_map_retains_mut_borrow() {
        let cell = Cell::new(Box::new(10));

        let _rr: RefMut<'_, usize> = cell.borrow_mut().map(Box::as_mut);

        let _ = cell.borrow_mut();
    }

    #[test]
    fn ref_mut_map_drops_borrow() {
        let cell = Cell::new(Box::new(10));

        let r: RefMut<'_, usize> = cell.borrow_mut().map(Box::as_mut);

        assert_eq!(cell.flag.load(Ordering::SeqCst), std::usize::MAX);
        drop(r);
        assert_eq!(cell.flag.load(Ordering::SeqCst), 0);
    }
}
