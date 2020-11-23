pub mod builder;
pub mod error;
pub mod run;
pub mod task;

pub use builder::Builder;
pub use error::Error;
pub use run::{LocalRun, LocalRunAsync, Run, RunAsync, ThreadRun, ThreadRunAsync};

use std::cell::RefCell;
use std::ops::Deref;
use std::ptr::null;
use std::sync::Arc;

use tokio::sync::watch::{Receiver as WatchReceiver, Sender as WatchSender};

use crate::world::World;

type Sender = WatchSender<()>;
type Receiver = WatchReceiver<()>;

/// The dispatcher struct, allowing
/// systems to be executed in parallel.
pub struct Dispatcher {
    sender: Sender,
    receivers: Vec<Receiver>,
    world: SharedWorld,
}

impl Dispatcher {
    /// Create builder to build a new dispatcher.
    pub fn builder() -> Builder<'static> {
        Builder::new(None)
    }

    /// Create builder to build a new dispatcher that
    /// invokes the setup for each passed system.
    pub fn setup_builder(world: &mut World) -> Builder<'_> {
        Builder::new(Some(world))
    }

    /// Dispatch all the systems with given resources and context
    /// and then run thread local systems.
    ///
    /// Please note that this method assumes that no resource
    /// is currently borrowed. If that's the case, it panics.
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

/// Helper type to share the world parameter passed to `Dispatcher::dispatch`.
#[derive(Clone)]
pub struct SharedWorld(Arc<RefCell<*const World>>);

impl SharedWorld {
    fn set(&mut self, world: &World) -> WorldGuard {
        *self.0.borrow_mut() = world as *const _;

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

/// Guard to share the world parameter passed to `Dispatcher::dispatch`.
struct WorldGuard<'a>(&'a mut SharedWorld);

impl Drop for WorldGuard<'_> {
    fn drop(&mut self) {
        self.0.clear()
    }
}
