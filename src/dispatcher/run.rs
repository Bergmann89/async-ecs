use std::ops::Deref;

use futures::future::BoxFuture;

use crate::{
    system::{AsyncSystem, DynamicSystemData, System},
    world::World,
};

pub type ThreadRun = Box<dyn for<'a> Run<'a> + Send>;
pub type LocalRun = Box<dyn for<'a> Run<'a>>;

pub type ThreadRunAsync = Box<dyn for<'a> RunAsync<'a> + Send>;
pub type LocalRunAsync = Box<dyn for<'a> RunAsync<'a>>;

/// Trait for fetching data and running systems.
/// Automatically implemented for systems.
pub trait Run<'a> {
    /// Runs the system now.
    ///
    /// # Panics
    ///
    /// Panics if the system tries to fetch resources
    /// which are borrowed in an incompatible way already
    /// (tries to read from a resource which is already written to or
    /// tries to write to a resource which is read from).
    fn run(&mut self, world: &'a World);
}

impl<'a, T> Run<'a> for T
where
    T: System<'a>,
{
    fn run(&mut self, world: &'a World) {
        let data = T::SystemData::fetch(self.accessor().deref(), world);

        self.run(data)
    }
}

/// Trait for fetching data and running systems with async/await.
/// Automatically implemented for systems.
pub trait RunAsync<'a> {
    /// Runs the system now.
    ///
    /// # Panics
    ///
    /// Panics if the system tries to fetch resources
    /// which are borrowed in an incompatible way already
    /// (tries to read from a resource which is already written to or
    /// tries to write to a resource which is read from).
    fn run(&mut self, world: &'a World) -> BoxFuture<'a, ()>;
}

impl<'a, T> RunAsync<'a> for T
where
    T: AsyncSystem<'a>,
{
    fn run(&mut self, world: &'a World) -> BoxFuture<'a, ()> {
        let data = T::SystemData::fetch(self.accessor().deref(), world);

        self.run_async(data)
    }
}
