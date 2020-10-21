use thiserror::Error;

use crate::entity::Entity;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Entity is not alive: {0}!")]
    EntityIsNotAlive(Entity),
}
