use log::info;

use super::{BoxedDispatchable, Receiver, Sender, SharedWorld};

pub async fn execute(
    name: String,
    dispatchable: BoxedDispatchable,
    sender: Sender,
    receivers: Vec<Receiver>,
    world: SharedWorld,
) {
    info!("System started: {}", &name);

    run(dispatchable, sender, receivers, world).await;

    info!("System finished: {}", &name);
}

async fn run(
    mut dispatchable: BoxedDispatchable,
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

        dispatchable.run(world);

        match sender.send(()) {
            Ok(()) => (),
            Err(_) => return,
        }
    }
}
