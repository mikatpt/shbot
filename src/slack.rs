use serde_json::Value;
use tracing::info;

use crate::{server::Result, Error};

pub mod events;
pub mod slash;

use events::Challenge;

// Events API handlers.

pub fn auth_challenge(request: Value) -> Result<String> {
    info!("Auth challenge received");

    let request: Challenge = serde_json::from_value(request).map_err(Into::<Error>::into)?;
    Ok(request.challenge)
}
