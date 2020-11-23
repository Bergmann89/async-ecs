mod system_data;

pub use system_data::{DynamicSystemData, SystemData};

use futures::future::BoxFuture;

use crate::{
    access::{Accessor, AccessorCow, AccessorType},
    world::World,
};

/// A `System`, executed with a set of required [`Resource`]s.
///
/// [`Resource`]: trait.Resource.html
pub trait System<'a>: Sized {
    /// The resource bundle required to execute a system.
    ///
    /// You will mostly use a tuple of system data (which also implements
    /// `SystemData`). You can also create such a resource bundle by simply
    /// deriving `SystemData` for a struct.
    ///
    /// Every `SystemData` is also a `DynamicSystemData`.
    type SystemData: DynamicSystemData<'a>;

    /// Initialize the systems.
    fn init(&mut self) {}

    /// Executes the system with the required system data.
    fn run(&mut self, data: Self::SystemData);

    /// Return the accessor from the [`SystemData`].
    fn accessor<'b>(&'b self) -> AccessorCow<'a, 'b, Self::SystemData> {
        AccessorCow::Owned(
            AccessorType::<'a, Self::SystemData>::try_new()
                .expect("Missing implementation for `accessor`"),
        )
    }

    /// Sets up the `World` using `Self::SystemData::setup`.
    fn setup(&mut self, world: &mut World) {
        self.init();

        <Self::SystemData as DynamicSystemData>::setup(&self.accessor(), world)
    }

    /// Performs clean up that requires resources from the `World`.
    /// This commonly removes components from `world` which depend on external
    /// resources.
    fn dispose(self, world: &mut World)
    where
        Self: Sized,
    {
        let _ = world;
    }
}

/// A `System`, executed with a set of required [`Resource`]s asynchronous.
///
/// [`Resource`]: trait.Resource.html
pub trait AsyncSystem<'a>: Sized {
    /// The resource bundle required to execute a system.
    ///
    /// You will mostly use a tuple of system data (which also implements
    /// `SystemData`). You can also create such a resource bundle by simply
    /// deriving `SystemData` for a struct.
    ///
    /// Every `SystemData` is also a `DynamicSystemData`.
    type SystemData: DynamicSystemData<'a>;

    /// Initialize the systems.
    fn init(&mut self) {}

    /// Executes the system with the required system data asynchronous.
    fn run_async(&mut self, data: Self::SystemData) -> BoxFuture<'a, ()>;

    /// Return the accessor from the [`SystemData`].
    fn accessor<'b>(&'b self) -> AccessorCow<'a, 'b, Self::SystemData> {
        AccessorCow::Owned(
            AccessorType::<'a, Self::SystemData>::try_new()
                .expect("Missing implementation for `accessor`"),
        )
    }

    /// Sets up the `World` using `Self::SystemData::setup`.
    fn setup(&mut self, world: &mut World) {
        self.init();

        <Self::SystemData as DynamicSystemData>::setup(&self.accessor(), world)
    }

    /// Performs clean up that requires resources from the `World`.
    /// This commonly removes components from `world` which depend on external
    /// resources.
    fn dispose(self, world: &mut World)
    where
        Self: Sized,
    {
        let _ = world;
    }
}
