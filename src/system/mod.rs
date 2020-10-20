mod accessor;
mod system_data;

pub use accessor::{Accessor, AccessorCow};
pub use system_data::{DynamicSystemData, SystemData};

use crate::resources::Resources;

use accessor::AccessorType;

pub trait System<'a> {
    type SystemData: DynamicSystemData<'a>;

    fn run(&mut self, data: Self::SystemData);

    fn accessor<'b>(&'b self) -> AccessorCow<'a, 'b, Self> {
        AccessorCow::Owned(
            AccessorType::<'a, Self>::try_new().expect("Missing implementation for `accessor`"),
        )
    }

    fn setup(&mut self, resources: &mut Resources) {
        <Self::SystemData as DynamicSystemData>::setup(&self.accessor(), resources)
    }

    fn dispose(self, resources: &mut Resources)
    where
        Self: Sized,
    {
        let _ = resources;
    }
}
