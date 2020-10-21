use std::mem::MaybeUninit;

use super::Storage;

pub struct VecStorage<T>(Vec<MaybeUninit<T>>);

impl<T> Storage<T> for VecStorage<T> {}

impl<T> Default for VecStorage<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}
