pub mod config;
pub mod logger;
pub mod server;
pub mod store;

mod errors;
mod models;
mod utils;

use crate::errors::Error;
pub type Result<T> = std::result::Result<T, Error>;
