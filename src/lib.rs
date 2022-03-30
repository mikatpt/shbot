pub mod server;
pub mod store;

mod models;
mod utils;

pub use utils::{config, errors, logger};

pub type Result<T> = std::result::Result<T, Error>;
use crate::errors::Error;
