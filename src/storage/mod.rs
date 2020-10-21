mod masked;
mod vec;

pub use masked::MaskedStorage;
pub use vec::VecStorage;

pub trait Storage<T> {}
