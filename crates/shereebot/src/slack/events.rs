use serde::{Deserialize, Serialize};
use strum::AsRefStr;
use tracing::{error, info};

use crate::{server::State, store::Client, Result};

/// This challenge is sent when the Event API first queries your event endpoint.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Challenge {
    pub token: String,
    pub challenge: String,
    #[serde(rename = "type")]
    pub event_type: EventType,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventRequest {
    pub token: String,
    pub team_id: String,
    pub api_app_id: String,
    pub event: Event,

    #[serde(rename = "type")]
    pub event_type: EventType,
    pub authorizations: Vec<serde_json::Value>,

    pub event_context: String,
    pub event_id: String,
    pub event_time: serde_json::Number,

    #[deprecated]
    pub authed_users: Option<Vec<String>>,
    #[deprecated]
    pub authed_teams: Option<Vec<String>>,
}

#[derive(Debug, Clone, AsRefStr, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    EventCallback,
    UrlVerification,
}

#[derive(Debug, Clone, AsRefStr, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Event {
    AppMention {
        user: String,
        text: String,
        ts: String,
        channel: String,
        event_ts: String,
    },
}

impl EventRequest {
    /// Once an event is called, we can handle it totally async.
    ///
    /// Normally, we log errors right before reporting them to the user.
    /// Since this can be a long-running task, we will log errors here.
    pub(crate) async fn handle_event<T: Client>(self, state: State<T>) {
        let result = match self.event {
            Event::AppMention { .. } => self.handle_app_mention(state).await,
            // _ => Ok(()),
        };
        match result {
            Ok(_) => info!("Completed event!"),
            Err(e) => error!("Slack event failure: {e}"),
        }
    }

    /// Entry gateway for mentions: branch out based on the parsed operation request.
    async fn handle_app_mention<T: Client>(self, state: State<T>) -> Result<()> {
        let manager = super::app_mentions::AppMention::new(state.clone(), self.event);

        let res = manager.handle_event().await?;

        state
            .req_client
            .post("https://slack.com/api/chat.postMessage")
            .bearer_auth(&state.oauth_token)
            .json(&res)
            .send()
            .await?;

        Ok(())
    }
}