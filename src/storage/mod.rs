mod anti_storage;
mod btree_storage;
mod dense_vec_storage;
mod drain;
mod hash_map_storage;
mod masked_storage;
mod storage_wrapper;
mod vec_storage;

pub use anti_storage::AntiStorage;
pub use btree_storage::BTreeStorage;
pub use dense_vec_storage::DenseVecStorage;
pub use drain::Drain;
pub use hash_map_storage::HashMapStorage;
pub use masked_storage::MaskedStorage;
pub use storage_wrapper::StorageWrapper;
pub use vec_storage::VecStorage;

use hibitset::BitSetLike;

use crate::{entity::Index, misc::TryDefault};

/// Used by the framework to quickly join components.
pub trait Storage<T>: TryDefault {
    /// Tries reading the data associated with an `Index`.
    /// This is unsafe because the external set used
    /// to protect this storage is absent.
    ///
    /// # Safety
    ///
    /// May only be called after a call to `insert` with `id` and
    /// no following call to `remove` with `id`.
    ///
    /// A mask should keep track of those states, and an `id` being contained
    /// in the tracking mask is sufficient to call this method.
    unsafe fn get(&self, index: Index) -> &T;

    /// Tries mutating the data associated with an `Index`.
    /// This is unsafe because the external set used
    /// to protect this storage is absent.
    ///
    /// # Safety
    ///
    /// May only be called after a call to `insert` with `id` and
    /// no following call to `remove` with `id`.
    ///
    /// A mask should keep track of those states, and an `id` being contained
    /// in the tracking mask is sufficient to call this method.
    unsafe fn get_mut(&mut self, index: Index) -> &mut T;

    /// Inserts new data for a given `Index`.
    ///
    /// # Safety
    ///
    /// May only be called if `insert` was not called with `id` before, or
    /// was reverted by a call to `remove` with `id.
    ///
    /// A mask should keep track of those states, and an `id` missing from the
    /// mask is sufficient to call `insert`.
    unsafe fn insert(&mut self, index: Index, value: T);

    /// Removes the data associated with an `Index`.
    ///
    /// # Safety
    ///
    /// May only be called if an element with `id` was `insert`ed and not yet
    /// removed / dropped.
    unsafe fn remove(&mut self, index: Index) -> T;

    /// Clean the storage given a bitset with bits set for valid indices.
    /// Allows us to safely drop the storage.
    ///
    /// # Safety
    ///
    /// May only be called with the mask which keeps track of the elements
    /// existing in this storage.
    unsafe fn clean<B>(&mut self, has: B)
    where
        B: BitSetLike;

    /// Drops the data associated with an `Index`.
    /// This could be used when a more efficient implementation for it exists than `remove` when the data
    /// is no longer needed.
    /// Defaults to simply calling `remove`.
    ///
    /// # Safety
    ///
    /// May only be called if an element with `id` was `insert`ed and not yet
    /// removed / dropped.
    unsafe fn drop(&mut self, index: Index) {
        self.remove(index);
    }
}

/// This is a marker trait which requires you to uphold the following guarantee:
///
/// > Multiple threads may call `get_mut()` with distinct indices without
/// causing > undefined behavior.
///
/// This is for example valid for `Vec`:
///
/// ```rust
/// vec![1, 2, 3];
/// ```
///
/// We may modify both element 1 and 2 at the same time; indexing the vector
/// mutably does not modify anything else than the respective elements.
///
/// As a counter example, we may have some kind of cached storage; it caches
/// elements when they're retrieved, so pushes a new element to some
/// cache-vector. This storage is not allowed to implement `DistinctStorage`.
///
/// Implementing this trait marks the storage safe for concurrent mutation (of
/// distinct elements), thus allows `join_par()`.
pub trait DistinctStorage {}
