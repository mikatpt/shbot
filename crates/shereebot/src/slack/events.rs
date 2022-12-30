#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use strum::AsRefStr;
use tracing::{error, info};

use crate::{server::State, Error, Result};

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
    pub authorizations: Vec<Authorization>,

    pub event_context: String,
    pub event_id: String,
    pub event_time: serde_json::Number,

    #[deprecated]
    pub authed_users: Option<Vec<String>>,
    #[deprecated]
    pub authed_teams: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Authorization {
    pub is_bot: bool,
}

#[derive(Debug, Clone, AsRefStr, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    EventCallback,
    UrlVerification,
}

#[derive(Debug, Clone, AsRefStr, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    AppHome,
    Im,
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
    Message {
        user: String,
        text: String,
        channel_type: ChannelType,
        subtype: Option<String>,
        files: Option<Vec<File>>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct File {
    pub id: String,
    pub name: String,
    pub filetype: String,
    pub url_private: String,
    pub url_private_download: String,
}

impl EventRequest {
    /// We handle events totally asynchronously, dispatching them back to slack via POST.
    ///
    /// Normally, we log errors right before reporting them to the user.
    /// Since this can be a long-running task, we will log errors here.
    pub(crate) async fn handle_event(self, state: State) {
        let result = match self.event {
            Event::AppMention { .. } => self.handle_app_mention(state).await,
            Event::Message { .. } => self.handle_message(state).await,
        };
        match result {
            Ok(_) => info!("Completed event!"),
            Err(e) => error!("Slack event failure: {e}"),
        }
    }

    async fn handle_message(self, state: State) -> Result<()> {
        let (user, channel_type, text, subtype, files) = if let Event::Message {
            user,
            channel_type,
            text,
            subtype,
            files,
        } = self.event
        {
            (user, channel_type, text, subtype, files)
        } else {
            return Err(Error::Unreachable);
        };

        let manager = super::message::Message::new(state, user, text, channel_type, subtype, files);
        manager.handle_event().await?;
        Ok(())
    }

    /// Entry gateway for mentions: branch out based on the parsed operation request.
    async fn handle_app_mention(self, state: State) -> Result<()> {
        #[rustfmt::skip]
        let (user, ts, channel, text) = if let Event::AppMention { user, ts, channel, text, .. } = self.event {
            (user, ts, channel, text)
        } else {
            return Err(Error::Unreachable);
        };
        let manager = super::app_mentions::AppMention::new(state.clone(), text, ts, channel, user);

        manager.handle_event().await?;

        Ok(())
    }
}
