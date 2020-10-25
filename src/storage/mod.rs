mod anti_storage;
mod masked_storage;
mod storage_wrapper;
mod vec_storage;

pub use anti_storage::AntiStorage;
pub use masked_storage::MaskedStorage;
pub use storage_wrapper::StorageWrapper;
pub use vec_storage::VecStorage;

use crate::{entity::Index, misc::TryDefault};

pub trait Storage<T>: TryDefault {
    fn get(&self, index: Index) -> &T;
    fn get_mut(&mut self, index: Index) -> &mut T;
    fn insert(&mut self, index: Index, value: T);
}

pub trait DistinctStorage {}
