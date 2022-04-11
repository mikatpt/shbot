pub mod queue;
pub mod server;
pub mod store;

mod manager;
mod slack;
mod utils;

pub use utils::{config, errors, logger};

pub type Result<T> = std::result::Result<T, Error>;
pub use crate::errors::{Error, UserError};
