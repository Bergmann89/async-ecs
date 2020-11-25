pub mod builder;
pub mod entities;
#[allow(clippy::module_inception)]
pub mod entity;

pub use builder::{Builder, EntityBuilder};
pub use entities::Entities;
pub use entity::{Entity, Generation, Index};
