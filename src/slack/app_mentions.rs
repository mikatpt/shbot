#![allow(dead_code)]
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::EnumString;

use crate::{
    films::FilmManager,
    slack::events::Event,
    store::{Client, Database},
    Error, Result,
};

/// Manager which handles all app_mention events.
pub(crate) struct AppMention<T: Client> {
    db: Database<T>,
}

impl<T: Client> AppMention<T> {
    pub(crate) fn new(db: Database<T>) -> Self {
        Self { db }
    }

    /// Given an app_mention event, does the following:
    ///
    /// 1. Parses desired command from the message.
    /// 2. Runs the requested command.
    /// 3. Returns a formatted `Response` with either an error or success msg
    pub(crate) async fn handle_event(&self, event: Event) -> Result<Response> {
        #[rustfmt::skip]
        let Event::AppMention { text, ts, channel, .. } = event;
        let cmd = match self.parse_command(&text) {
            Ok(c) => c,
            Err(e) => return Ok(Response::new(channel, e.to_string(), Some(ts))),
        };

        match self.run_command(cmd, text).await {
            Ok(msg) => Ok(Response::new(channel, msg, Some(ts))),
            Err(e) => Ok(Response::new(channel, e.to_string(), Some(ts))),
        }
    }

    /// Parse the event text for a command.
    ///
    /// Format: "<USER_ID> COMMAND MESSAGE"
    /// Example: "<@U0LAN0Z89> addfilms star wars, star trek"
    fn parse_command(&self, text: &str) -> Result<Command> {
        let cmd = text
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| Error::InvalidArg("Couldn't read your command!".into()))?;

        Ok(Command::from_str(cmd)?)
    }

    async fn run_command(&self, cmd: Command, text: String) -> Result<String> {
        match cmd {
            Command::AddFilms => {
                let manager = FilmManager::new(self.db.clone());
                let msg: String = text.split_whitespace().skip(2).collect();
                manager.insert_films(&msg).await
            }
            Command::RequestWork => Err(Error::InvalidArg("unimplemented".into())),
            Command::Deliver => Err(Error::InvalidArg("unimplemented".into())),
        }
    }
}

#[derive(Debug, Clone, Copy, EnumString, Serialize)]
#[strum(ascii_case_insensitive)]
enum Command {
    #[strum(serialize = "addfilms", serialize = "add-films", serialize = "addfilm")]
    AddFilms,
    #[strum(serialize = "requestwork", serialize = "request-work")]
    RequestWork,
    Deliver,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Response {
    pub channel: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_ts: Option<String>,
}

impl Response {
    #[rustfmt::skip]
    pub fn new(channel: String, text: String, thread_ts: Option<String>) -> Self {
        Self { channel, text, thread_ts }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::mock::MockClient;

    fn setup() -> AppMention<MockClient> {
        AppMention {
            db: Database::<MockClient>::new(),
        }
    }

    #[test]
    fn get_command() {
        let m = setup();
        let s = "<@U0LAN0Z89> addfilms star wars, star trek";
        let command = m.parse_command(s);
        assert!(command.is_ok());
    }
}
