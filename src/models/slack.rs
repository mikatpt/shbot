use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppMention {
    r#type: String,
    user: String,
    text: String,
    ts: f32,
    channel: String,
    event_ts: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthChallenge {
    pub token: String,
    pub challenge: String,
    pub r#type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlashRequest {
    pub token: String,

    #[serde(rename = "&team_id")]
    pub team_id: String, // T0001,
    #[serde(rename = "&team_domain")]
    pub team_domain: String, // example,
    #[serde(rename = "&enterprise_id")]
    pub enterprise_id: String, // E0001,
    #[serde(rename = "&enterprise_name")]
    pub enterprise_name: String, // Globular%20Construct%20Inc,
    #[serde(rename = "&channel_id")]
    pub channel_id: String, // C2147483705,
    #[serde(rename = "&channel_name")]
    pub channel_name: String, // test,
    #[serde(rename = "&user_id")]
    pub user_id: String, // U2147483697,
    #[serde(rename = "&user_name")]
    pub user_name: String, // Steve,
    #[serde(rename = "&command")]
    pub command: String, // /weather,
    #[serde(rename = "&text")]
    pub text: String, // 94070,
    #[serde(rename = "&response_url")]
    pub response_url: String, // https://hooks.slack.com/commands/1234/5678,
    #[serde(rename = "&trigger_id")]
    pub trigger_id: String, // 13345224609.738474920.8088930838d88f008e0,
    #[serde(rename = "&api_app_id")]
    pub api_app_id: String, // A123456,
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
