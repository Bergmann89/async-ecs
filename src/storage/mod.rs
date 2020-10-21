mod masked_storage;
mod storage_wrapper;
mod vec_storage;

pub use masked_storage::MaskedStorage;
pub use storage_wrapper::StorageWrapper;
pub use vec_storage::VecStorage;

use crate::misc::TryDefault;

pub trait Storage<T>: TryDefault {}
