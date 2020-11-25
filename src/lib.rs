#![allow(dead_code)]

pub mod access;
pub mod component;
pub mod dispatcher;
pub mod entity;
pub mod error;
pub mod join;
pub mod misc;
pub mod resource;
pub mod storage;
pub mod system;
pub mod world;

pub use asparit;

pub use access::{Read, ReadStorage, Write, WriteStorage};
pub use component::Component;
pub use dispatcher::Dispatcher;
pub use entity::Builder;
pub use join::{Join, ParJoin};
pub use resource::{ResourceId, Resources};
pub use storage::{DenseVecStorage, HashMapStorage, VecStorage};
pub use system::{AsyncSystem, System};
pub use world::{Lazy, World};

pub type Entities<'a> = Read<'a, entity::Entities>;

#[macro_use]
#[allow(unused_imports)]
#[cfg(feature = "derive")]
extern crate async_ecs_derive;

#[doc(hidden)]
#[cfg(feature = "derive")]
pub use async_ecs_derive::*;
