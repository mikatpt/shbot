pub mod app_mentions;
pub mod events;
pub mod slash;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub ok: bool,
    pub user: SlackUser,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlackUser {
    pub id: String,
    pub real_name: String,
}
