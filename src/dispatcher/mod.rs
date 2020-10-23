pub mod builder;
pub mod dispatchable;
pub mod error;
pub mod task;

pub use builder::Builder;
pub use dispatchable::{BoxedDispatchable, Dispatchable};
pub use error::Error;

use std::cell::RefCell;
use std::ops::Deref;
use std::sync::Arc;

use tokio::sync::watch::{Receiver as WatchReceiver, Sender as WatchSender};

use crate::world::World;

pub struct Dispatcher {
    sender: Sender,
    receivers: Vec<Receiver>,
    world: SharedWorld,
}

#[derive(Clone, Default)]
pub struct SharedWorld(Arc<RefCell<Option<World>>>);

type Sender = WatchSender<()>;
type Receiver = WatchReceiver<()>;

impl Dispatcher {
    pub fn builder() -> Builder {
        Builder::default()
    }

    pub async fn dispatch(&mut self, world: World) -> Result<World, Error> {
        *self.world.borrow_mut() = Some(world);

        match self.sender.send(()) {
            Ok(()) => (),
            Err(_) => return Err(Error::DispatchSend),
        }

        for receiver in &mut self.receivers {
            match receiver.changed().await {
                Ok(()) => (),
                Err(_) => return Err(Error::DispatchReceive),
            }
        }

        Ok(self.world.borrow_mut().take().unwrap())
    }
}

impl Deref for SharedWorld {
    type Target = RefCell<Option<World>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl Send for SharedWorld {}
unsafe impl Sync for SharedWorld {}
