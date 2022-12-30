use std::str::FromStr;

use itertools::Itertools;
use models::{Film, Priority};
use serde::{Deserialize, Serialize};
use strum::EnumString;
use tracing::debug;

use crate::{manager::Manager, server::State, Error, Result};

const HELLO: &str =
    ":wave: Hi! I'm ShereeBot. Sheree's brother built me to help her manage your film assignments!
For a list of my commands, type @ShereeBot help";

const HELP: &str = "Sheree commands:
`add-films [HIGH or LOW] [film1, film2, film3...]`

To deliver your work, type `@ShereeBot deliver`.
Once you're ready to move on to the next step, type `@ShereeBot request-work`.
As soon as there's work ready to be picked up, I'll let you know!";

const CMD_ERR: &str = "I couldn't read your command :cry:
Valid commands include `deliver-work`, and `request-work`!
Sheree can also run the `add-films` command!";

/// Manager which handles all app_mention events.
pub(crate) struct AppMention {
    state: State,
    text: String,
    ts: String,
    channel: String,
    user: String,
}

impl AppMention {
    #[rustfmt::skip]
    pub(crate) fn new(
        state: State,
        text: String,
        ts: String,
        channel: String,
        user: String,
    ) -> Self {
        Self { state, text, ts, channel, user }
    }

    /// Given an app_mention event, does the following:
    ///
    /// 1. Parses desired command from the message.
    /// 2. Runs the requested command.
    /// 3. Returns a formatted `Response` with either an error or success msg
    #[tracing::instrument(name = "app_mention", skip_all)]
    pub(crate) async fn handle_event(&self) -> Result<()> {
        debug!("[handle_event]: {:?}", self.text);
        let (ts, channel) = (self.ts.clone(), self.channel.clone());

        let res = match self.run_event().await {
            Ok(msg) => Response::new(channel, msg, Some(ts)),
            Err(e) => Response::new(channel, e.to_string(), Some(ts)),
        };

        self.state
            .req_client
            .post("https://slack.com/api/chat.postMessage")
            .bearer_auth(&self.state.oauth_token)
            .json(&res)
            .send()
            .await?;

        Ok(())
    }

    async fn run_event(&self) -> Result<String> {
        if self.text.split_whitespace().nth(1).is_none() {
            return Ok(HELLO.to_string());
        }

        let cmd = match self.parse_command() {
            Ok(c) => c,
            Err(e) => return Ok(e.to_string()),
        };

        match self.run_command(cmd).await {
            Ok(msg) => Ok(msg),
            Err(e) => Ok(e.to_string()),
        }
    }

    /// Parse the event text for a command.
    ///
    /// Format: "<USER_ID> COMMAND MESSAGE"
    /// Example: "<@U0LAN0Z89> addfilms HIGH 1 star wars, star trek"
    fn parse_command(&self) -> Result<Command> {
        let cmd = self
            .text
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| Error::InvalidArg(CMD_ERR.into()))?;

        Ok(Command::from_str(&cmd.to_uppercase())?)
    }

    async fn run_command(&self, cmd: Command) -> Result<String> {
        let manager = Manager::new(self.state.clone());
        match cmd {
            Command::AddFilms => {
                // <USER_ID> addfilms <PRI> <GROUP> <FILMS>
                let msg: String =
                    Itertools::intersperse(self.text.split_whitespace().skip(2), " ").collect();
                let (priority, rest) = msg
                    .split_once(' ')
                    .ok_or_else(|| Error::InvalidArg(CMD_ERR.into()))?;

                let (group, films) = rest
                    .split_once(' ')
                    .ok_or_else(|| Error::InvalidArg(CMD_ERR.into()))?;
                let priority = Priority::from_str(&priority.to_uppercase())?;
                let group = group.parse::<i32>().unwrap_or_default();

                let films: Vec<Film> = films
                    .split(',')
                    .map(|s| Film::new(s.trim(), priority, group))
                    .collect();

                Ok(manager.insert_films(films).await)
            }
            Command::Help => Ok(HELP.to_string()),
            Command::RequestWork => Ok(manager
                .request_work(&self.user, &self.ts, &self.channel)
                .await),
            Command::DeliverWork => Ok(manager.deliver_work(&self.user).await),
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
    #[strum(serialize = "deliverwork", serialize = "deliver-work")]
    DeliverWork,
    Help,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Response {
    pub channel: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_ts: Option<String>,
}

impl Response {
    pub fn new(channel: String, text: String, thread_ts: Option<String>) -> Self {
        Self {
            channel,
            text,
            thread_ts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::InnerState;

    fn setup() -> AppMention {
        let state = InnerState::_new();
        AppMention {
            state,
            user: "".to_string(),
            ts: "".to_string(),
            channel: "".to_string(),
            text: "".to_string(),
        }
    }

    #[test]
    fn get_command() {
        let mut m = setup();

        m.text = "<@U0LAN0Z89> addfilms star wars, star trek".to_string();
        let command = m.parse_command();
        assert!(command.is_ok());
    }
}
