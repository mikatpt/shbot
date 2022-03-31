use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlashRequest {
    pub token: String,        // KXcvC55555555peKWVe9axCl
    pub team_id: String,      // T0385559PDH
    pub team_domain: String,  // shereebot
    pub channel_id: String,   // C03905M3EKB
    pub channel_name: String, // writing-shereebot
    pub user_id: String,      // U038MGZT5T4
    pub user_name: String,    // mikatpt
    pub command: String,
    pub text: String,                // testing testing
    pub api_app_id: String,          // A038BV5B8SK
    pub response_url: String,        // https://hooks.slack.com/commands/1234/5678
    pub trigger_id: String, // 3321793495218.3293569329459.5c003e55aa6c7471a55aee9af1038f56"
    pub is_enterprise_install: bool, // false

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterprise_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterprise_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlashResponse {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_type: Option<ResponseType>,
}

impl SlashResponse {
    pub fn new(text: String, response_type: Option<ResponseType>) -> Self {
        SlashResponse {
            text,
            response_type,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ResponseType {
    #[serde(rename = "ephemeral")]
    Ephemeral,
    #[serde(rename = "in_channel")]
    InChannel,
}
