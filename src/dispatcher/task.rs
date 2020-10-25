use log::info;

use super::{LocalRun, Receiver, Run, Sender, SharedWorld, ThreadRun};

pub async fn execute_thread(
    name: String,
    mut run: ThreadRun,
    sender: Sender,
    receivers: Vec<Receiver>,
    world: SharedWorld,
) {
    info!("System started: {}", &name);

    execute_inner(run.as_mut(), sender, receivers, world).await;

    info!("System finished: {}", &name);
}

pub async fn execute_local(
    name: String,
    mut run: LocalRun,
    sender: Sender,
    receivers: Vec<Receiver>,
    world: SharedWorld,
) {
    info!("System started (local): {}", &name);

    execute_inner(run.as_mut(), sender, receivers, world).await;

    info!("System finished (local): {}", &name);
}

async fn execute_inner<R: for<'a> Run<'a> + ?Sized>(
    run: &mut R,
    sender: Sender,
    mut receivers: Vec<Receiver>,
    world: SharedWorld,
) {
    loop {
        for receiver in &mut receivers {
            match receiver.changed().await {
                Ok(()) => (),
                Err(_) => return,
            }
        }

        let world = world.borrow();
        let world = world.as_ref().unwrap();

        run.run(world);

        match sender.send(()) {
            Ok(()) => (),
            Err(_) => return,
        }
    }
}
