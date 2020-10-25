use std::ops::Deref;
use std::pin::Pin;

use futures::future::{Future, FutureExt};

use crate::{
    system::{AsyncSystem, DynamicSystemData},
    world::World,
};

pub type BoxedFuture<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
pub type BoxedDispatchable = Box<dyn for<'a> Dispatchable<'a> + Send>;

pub trait Dispatchable<'a> {
    fn run(&mut self, world: &'a World) -> BoxedFuture<'a>;
}

impl<'a, T> Dispatchable<'a> for T
where
    T: AsyncSystem<'a>,
    <T as AsyncSystem<'a>>::Future: Send,
{
    fn run(&mut self, world: &'a World) -> BoxedFuture<'a> {
        let data = T::SystemData::fetch(self.accessor().deref(), world);

        self.run(data).boxed()
    }
}
