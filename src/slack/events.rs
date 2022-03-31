use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::{server::State, Result};

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
    pub authorizations: serde_json::Value,

    pub event_context: String,
    pub event_id: String,
    pub event_time: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    AppMention,
    Other,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventResponse {
    pub channel: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_ts: Option<String>,
}

impl EventResponse {
    #[rustfmt::skip]
    pub fn new(channel: String, text: String, thread_ts: Option<String>) -> Self {
        Self { channel, text, thread_ts }
    }
}

impl EventRequest {
    /// Once an event is called, we can handle it totally async.
    ///
    /// Normally, we log errors right before reporting them to the user.
    /// Since this can be a long-running task, we will log errors here.
    #[tracing::instrument]
    pub(crate) async fn handle_event(self, state: State) {
        info!("Handling async slack event...");

        let result = match self.event_type {
            EventType::AppMention => self.handle_app_mention(state).await,
            _ => Ok(()),
        };
        match result {
            Ok(_) => info!("Completed event!"),
            Err(e) => error!("Slack event failure: {e}"),
        }
    }

    async fn handle_app_mention(self, state: State) -> Result<()> {
        info!("Handling app mention");

        debug!("{:?}", self.event);
        #[rustfmt::skip]
        let Event::AppMention { user, text, ts, channel, .. } = self.event;

        let _a = (user, text);

        // 1. Parse the text from the event
        // 2. Run the requested operation
        // 3. Return a formatted EventResponse with either an error or success msg
        let msg = String::from("testing first iteration of response api!");

        let res = EventResponse::new(channel, msg, Some(ts));

        state
            .req_client
            .post("https://slack.com/api/chat.postMessage")
            .bearer_auth(&state.oauth_token)
            .json(&res)
            .send()
            .await?;

        // Send a response to the user. Use the oAuth token from .env

        Ok(())
    }
}
