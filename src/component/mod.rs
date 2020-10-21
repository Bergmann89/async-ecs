use std::any::Any;

use crate::storage::Storage;

pub trait Component: Any + Sized {
    type Storage: Storage<Self> + Any + Send + Sync;
}
