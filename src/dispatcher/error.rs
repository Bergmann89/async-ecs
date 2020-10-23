use std::fmt::Debug;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("A System with this name was already registered: {0}!")]
    NameAlreadyRegistered(String),

    #[error("Dependency of the given system was not found: {0}!")]
    DependencyWasNotFound(String),

    #[error("Unable to start dispatching!")]
    DispatchSend,

    #[error("Unable to wait for systems to finish!")]
    DispatchReceive,
}
