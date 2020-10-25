mod system_data;

pub use system_data::{DynamicSystemData, SystemData, WithSystemData};

use futures::future::{ready, Future, Ready};

use crate::{
    access::{Accessor, AccessorCow, AccessorType},
    world::World,
};

/* System */

pub trait System<'a>: Sized {
    type SystemData: DynamicSystemData<'a>;

    fn init(&mut self) {}

    fn run(&mut self, data: Self::SystemData);

    fn accessor<'b>(&'b self) -> AccessorCow<'a, 'b, Self> {
        AccessorCow::Owned(
            AccessorType::<'a, Self>::try_new().expect("Missing implementation for `accessor`"),
        )
    }

    fn setup(&mut self, world: &mut World) {
        self.init();

        <Self::SystemData as DynamicSystemData>::setup(&self.accessor(), world)
    }

    fn dispose(self, world: &mut World)
    where
        Self: Sized,
    {
        let _ = world;
    }
}

/* AsyncSystem */

pub trait AsyncSystem<'a>: Sized {
    type SystemData: DynamicSystemData<'a>;
    type Future: Future<Output = ()> + Send + 'a;

    fn init(&mut self) {}

    fn run(&mut self, data: Self::SystemData) -> Self::Future;

    fn accessor<'b>(&'b self) -> AccessorCow<'a, 'b, Self> {
        AccessorCow::Owned(
            AccessorType::<'a, Self>::try_new().expect("Missing implementation for `accessor`"),
        )
    }

    fn setup(&mut self, world: &mut World) {
        self.init();

        <Self::SystemData as DynamicSystemData>::setup(&self.accessor(), world)
    }

    fn dispose(self, world: &mut World)
    where
        Self: Sized,
    {
        let _ = world;
    }
}

impl<'a, T> AsyncSystem<'a> for T
where
    T: System<'a>,
{
    type SystemData = T::SystemData;
    type Future = Ready<()>;

    fn init(&mut self) {
        T::init(self);
    }

    fn run(&mut self, data: Self::SystemData) -> Self::Future {
        T::run(self, data);

        ready(())
    }

    fn accessor<'b>(&'b self) -> AccessorCow<'a, 'b, Self> {
        T::accessor(self)
    }

    fn setup(&mut self, world: &mut World) {
        T::setup(self, world)
    }

    fn dispose(self, world: &mut World)
    where
        Self: Sized,
    {
        T::dispose(self, world)
    }
}

impl<'a, T> WithSystemData<'a> for T
where
    T: AsyncSystem<'a>,
{
    type SystemData = <T as AsyncSystem<'a>>::SystemData;
}
