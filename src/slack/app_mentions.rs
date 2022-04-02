use std::str::FromStr;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::EnumString;

use crate::{
    films::FilmManager, server::State, slack::events::Event, store::Client, Error, Result,
};

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
pub(crate) struct AppMention<T: Client> {
    state: State<T>,
    event: Event,
}

impl<T: Client> AppMention<T> {
    #[rustfmt::skip]
    pub(crate) fn new(state: State<T>, event: Event) -> Self {
        Self { state, event }
    }

    /// Given an app_mention event, does the following:
    ///
    /// 1. Parses desired command from the message.
    /// 2. Runs the requested command.
    /// 3. Returns a formatted `Response` with either an error or success msg
    pub(crate) async fn handle_event(&self) -> Result<Response> {
        #[rustfmt::skip]
        let Event::AppMention { ts, channel, text, .. } = &self.event;
        let (ts, channel) = (ts.clone(), channel.clone());

        if text.split_whitespace().nth(1).is_none() {
            return Ok(Response::new(channel, HELLO.to_string(), None));
        }

        let cmd = match self.parse_command() {
            Ok(c) => c,
            Err(_) => return Ok(Response::new(channel, CMD_ERR.to_string(), Some(ts))),
        };

        let msg = self.run_command(cmd).await;
        Ok(Response::new(channel, msg, Some(ts)))
    }

    /// Parse the event text for a command.
    ///
    /// Format: "<USER_ID> COMMAND MESSAGE"
    /// Example: "<@U0LAN0Z89> addfilms star wars, star trek"
    fn parse_command(&self) -> Result<Command> {
        let Event::AppMention { text, .. } = &self.event;
        let cmd = text
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| Error::InvalidArg(CMD_ERR.into()))?;

        Ok(Command::from_str(cmd)?)
    }

    async fn run_command(&self, cmd: Command) -> String {
        #[rustfmt::skip]
        let Event::AppMention { text, ts, channel, user, .. } = &self.event;

        let manager = FilmManager::new(self.state.clone());
        match cmd {
            Command::AddFilms => {
                let msg: String =
                    Itertools::intersperse(text.split_whitespace().skip(2), " ").collect();
                manager.insert_films(&msg).await
            }
            Command::Help => HELP.to_string(),
            Command::RequestWork => manager.request_work(user, ts, channel).await,
            Command::DeliverWork => manager.deliver_work(user).await,
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
    #[rustfmt::skip]
    pub fn new(channel: String, text: String, thread_ts: Option<String>) -> Self {
        Self { channel, text, thread_ts }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{server::InnerState, store::mock::MockClient};

    fn setup() -> AppMention<MockClient> {
        let event = Event::AppMention {
            user: "".to_string(),
            ts: "".to_string(),
            channel: "".to_string(),
            text: "".to_string(),
            event_ts: "".to_string(),
        };
        let state = InnerState::<MockClient>::_new();
        AppMention { state, event }
    }

    #[test]
    fn get_command() {
        let mut m = setup();
        let Event::AppMention { text, .. } = &mut m.event;
        *text = "<@U0LAN0Z89> addfilms star wars, star trek".to_string();
        let command = m.parse_command();
        assert!(command.is_ok());
    }
}
