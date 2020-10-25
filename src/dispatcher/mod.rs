pub mod builder;
pub mod error;
pub mod run;
pub mod task;

pub use builder::Builder;
pub use error::Error;
pub use run::{LocalRun, Run, ThreadRun};

use std::ptr::null;
use std::ops::Deref;
use std::sync::Arc;
use std::cell::RefCell;

use tokio::sync::watch::{Receiver as WatchReceiver, Sender as WatchSender};

use crate::world::World;

pub struct Dispatcher {
    sender: Sender,
    receivers: Vec<Receiver>,
    world: SharedWorld,
}

#[derive(Clone)]
pub struct SharedWorld(Arc<RefCell<* const World>>);

struct WorldGuard<'a>(&'a mut SharedWorld);

type Sender = WatchSender<()>;
type Receiver = WatchReceiver<()>;

impl Dispatcher {
    pub fn builder() -> Builder {
        Builder::default()
    }

    pub async fn dispatch(&mut self, world: &World) -> Result<(), Error> {
        let _guard = self.world.set(world);

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

        Ok(())
    }
}

impl SharedWorld {
    fn set(&mut self, world: &World) -> WorldGuard {
        *self.0.borrow_mut() = world as * const _;

        WorldGuard(self)
    }

    fn clear(&mut self) {
        *self.0.borrow_mut() = null();
    }
}

unsafe impl Send for SharedWorld {}
unsafe impl Sync for SharedWorld {}

impl Default for SharedWorld {
    fn default() -> Self {
        Self(Arc::new(RefCell::new(null())))
    }
}

impl Deref for SharedWorld {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        let world = self.0.borrow();

        if world.is_null() {
            panic!("No World assigned!");
        }

        unsafe { &**world }
    }
}

impl Drop for WorldGuard<'_> {
    fn drop(&mut self) {
        self.0.clear()
    }
}
