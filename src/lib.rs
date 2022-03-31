pub mod server;
pub mod store;

mod models;
mod queue;
mod slack;
mod utils;

pub use utils::{config, errors, logger};

pub type Result<T> = std::result::Result<T, Error>;
use crate::errors::{Error, UserError};
