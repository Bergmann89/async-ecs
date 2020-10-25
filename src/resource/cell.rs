use std::cell::UnsafeCell;
use std::mem::forget;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct Cell<T> {
    flag: AtomicUsize,
    inner: UnsafeCell<T>,
}

#[derive(Debug)]
pub struct Ref<'a, T>
where
    T: ?Sized + 'a,
{
    flag: &'a AtomicUsize,
    value: &'a T,
}

#[derive(Debug)]
pub struct RefMut<'a, T>
where
    T: ?Sized + 'a,
{
    flag: &'a AtomicUsize,
    value: &'a mut T,
}

macro_rules! borrow_panic {
    ($s:expr) => {{
        panic!(
            "Tried to fetch data of type {:?}, but it was already borrowed{}.",
            ::std::any::type_name::<T>(),
            $s,
        )
    }};
}

/* Cell */

impl<T> Cell<T> {
    pub fn new(inner: T) -> Self {
        Cell {
            flag: AtomicUsize::new(0),
            inner: UnsafeCell::new(inner),
        }
    }

    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }

    pub fn borrow(&self) -> Ref<T> {
        if !self.check_flag_read() {
            borrow_panic!(" mutably");
        }

        Ref {
            flag: &self.flag,
            value: unsafe { &*self.inner.get() },
        }
    }

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

    pub fn borrow_mut(&self) -> RefMut<T> {
        if !self.check_flag_write() {
            borrow_panic!("");
        }

        RefMut {
            flag: &self.flag,
            value: unsafe { &mut *self.inner.get() },
        }
    }

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

    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.inner.get() }
    }

    fn check_flag_read(&self) -> bool {
        loop {
            let val = self.flag.load(Ordering::Acquire);

            if val == usize::MAX {
                return false;
            }

            if self.flag.compare_and_swap(val, val + 1, Ordering::AcqRel) == val {
                return true;
            }
        }
    }

    fn check_flag_write(&self) -> bool {
        self.flag.compare_and_swap(0, usize::MAX, Ordering::AcqRel) == 0
    }
}

unsafe impl<T> Sync for Cell<T> where T: Sync {}

/* Ref */

impl<'a, T> Ref<'a, T>
where
    T: ?Sized,
{
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

/* RefMut */

impl<'a, T> RefMut<'a, T>
where
    T: ?Sized,
{
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
    #[should_panic(expected = "but it was already borrowed")]
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
    #[should_panic(expected = "but it was already borrowed")]
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
