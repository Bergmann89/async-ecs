use crate::{
    system::{DynamicSystemData, System},
    world::World,
};

pub type BoxedDispatchable = Box<dyn for<'a> Dispatchable<'a> + Send>;

pub trait Dispatchable<'a> {
    fn run(&mut self, world: &'a World);
}

impl<'a, T> Dispatchable<'a> for T
where
    T: System<'a>,
{
    fn run(&mut self, world: &'a World) {
        let data = T::SystemData::fetch(&self.accessor(), world);

        self.run(data);
    }
}
