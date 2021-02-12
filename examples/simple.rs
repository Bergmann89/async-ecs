use async_ecs::{
    dispatcher, AsyncSystem, Builder, Component, Dispatcher, Entities, Join, ReadStorage,
    SystemData, VecStorage, World, WriteStorage,
};
use futures::{future::BoxFuture, stream, StreamExt, TryStreamExt};
use tokio::runtime;

#[derive(Debug)]
pub struct Vel(f32);

impl Component for Vel {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
pub struct Pos(f32);

impl Component for Pos {
    type Storage = VecStorage<Self>;
}

#[derive(SystemData)]
pub struct PositionUpdateSystemData<'s> {
    velocities: ReadStorage<'s, Vel>,
    positions: WriteStorage<'s, Pos>,
}

struct PositionUpdateSystem;

impl<'s> AsyncSystem<'s> for PositionUpdateSystem {
    // Resources used during system execution.
    type SystemData = PositionUpdateSystemData<'s>;

    fn run_async(&mut self, system_data: Self::SystemData) -> BoxFuture<'s, ()> {
        Box::pin(async move {
            let PositionUpdateSystemData {
                velocities,
                mut positions,
            } = system_data;

            // The `.join()` combines multiple components, so we only access those entities
            // which have both of them.
            for (vel, pos) in (&velocities, &mut positions).join() {
                pos.0 += vel.0;
            }
        })
    }
}

#[derive(SystemData)]
pub struct PrintSystemData<'s> {
    entities: Entities<'s>,
    positions: ReadStorage<'s, Pos>,
}

/// System that prints entity positions.
struct PrintSystem;

impl<'s> AsyncSystem<'s> for PrintSystem {
    type SystemData = PrintSystemData<'s>;

    fn run_async(&mut self, system_data: Self::SystemData) -> BoxFuture<'s, ()> {
        Box::pin(async move {
            let PrintSystemData {
                entities,
                positions,
            } = system_data;

            for (entity, pos) in (&entities, &positions).join() {
                eprintln!("entity {id}: {pos}", id = entity.id(), pos = pos.0);
            }
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_multi_thread().build()?;

    rt.block_on(async {
        // The `World` is our container for components and other resources.
        let mut world = World::default();

        // This builds a dispatcher.
        // The third parameter of `add` specifies logical dependencies on other systems.
        //
        // `AsyncSystem::setup()` is called for each system as they are added.
        let dispatcher = Dispatcher::setup_builder(&mut world)
            .with_async(PrintSystem, "print_system", &[])?
            .with_async(
                PositionUpdateSystem,
                "position_update_system",
                &["print_system"],
            )?
            .build();

        // Entities with `Pos` and `Vel` components.
        world.create_entity().with(Pos(1.0)).with(Vel(1.0)).build();
        world.create_entity().with(Pos(2.0)).with(Vel(2.0)).build();
        world.create_entity().with(Pos(3.0)).with(Vel(3.0)).build();

        // This entity does not have `Vel`, so its position won't be updated.
        world.create_entity().with(Pos(10.0)).build();

        let world = &world;
        stream::iter(0..4)
            .map(Result::<usize, dispatcher::Error>::Ok)
            .try_fold(dispatcher, |mut dispatcher, n| async move {
                eprintln!("Iteration {}", n);
                dispatcher.dispatch(world).await?;
                eprintln!();

                Ok(dispatcher)
            })
            .await?;

        Result::<(), dispatcher::Error>::Ok(())
    })?;

    Ok(())
}
