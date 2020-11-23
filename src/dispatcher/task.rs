use log::info;

use super::{
    LocalRun, LocalRunAsync, Receiver, Run, RunAsync, Sender, SharedWorld, ThreadRun,
    ThreadRunAsync,
};

/// Long running task of a `System` that is executed in a separate thread.
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

/// Long running task of a `System` that is executed in the thread local context.
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

/// Long running task of a `System` that is executed in a separate thread.
pub async fn execute_thread_async(
    name: String,
    mut run: ThreadRunAsync,
    sender: Sender,
    receivers: Vec<Receiver>,
    world: SharedWorld,
) {
    info!("System started: {}", &name);

    execute_inner_async(run.as_mut(), sender, receivers, world).await;

    info!("System finished: {}", &name);
}

/// Long running task of a `System` that is executed in the thread local context.
pub async fn execute_local_async(
    name: String,
    mut run: LocalRunAsync,
    sender: Sender,
    receivers: Vec<Receiver>,
    world: SharedWorld,
) {
    info!("System started (local): {}", &name);

    execute_inner_async(run.as_mut(), sender, receivers, world).await;

    info!("System finished (local): {}", &name);
}

/// Actual tasks that is running the system.
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

        run.run(&world);

        match sender.send(()) {
            Ok(()) => (),
            Err(_) => return,
        }
    }
}

/// Actual tasks that is running the system.
async fn execute_inner_async<R: for<'a> RunAsync<'a> + ?Sized>(
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

        run.run(&world).await;

        match sender.send(()) {
            Ok(()) => (),
            Err(_) => return,
        }
    }
}
