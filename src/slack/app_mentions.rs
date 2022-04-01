#![allow(dead_code)]
use std::marker::Send;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::EnumString;

use crate::{
    films::FilmManager, server::State, slack::events::Event, store::Client, Error, Result,
};

pub(crate) struct AppMention<T: Client + Send> {
    state: State<T>,
}
pub(crate) async fn handle_event(event: Event) -> Result<Response> {
    #[rustfmt::skip]
    let Event::AppMention { user, text, ts, channel, .. } = event;

    todo!();
}

impl<T: Client + Send> AppMention<T> {
    pub(crate) fn new(state: State<T>) -> Self {
        Self { state }
    }

    /// Given an app_mention event, does the following:
    ///
    /// 1. Parses desired command from the message.
    /// 2. Runs the requested command.
    /// 3. Returns a formatted `Response` with either an error or success msg
    pub(crate) async fn handle_event(&self, event: Event) -> Result<Response> {
        #[rustfmt::skip]
        let Event::AppMention { user, text, ts, channel, .. } = event;

        todo!();
    }

    /// Parse the event text for a command.
    ///
    /// Format: "<USER_ID> COMMAND MESSAGE"
    /// Example: "<@U0LAN0Z89> addfilms star wars, star trek"
    // fn parse_command(text: String) -> Result<Command> {
    //     let cmd = text
    //         .split_whitespace()
    //         .nth(1)
    //         .ok_or_else(|| Error::InvalidArg("Couldn't read your command!".into()))?;

    //     Ok(Command::from_str(cmd)?)
    // }
    // fn run_command(cmd: Command) -> Result<String> {
    //     match cmd {
    //         Command::AddFilms => {
    //             let manager = FilmManager::new

    //         },
    //         Command::RequestWork => {},
    //         Command::Deliver => {},
    //     }
    //     todo!();

    // }

    /// Formats either the success or error message.
    fn format_response(res: String) -> Response {
        todo!();
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn get_command() {
//         let s = "<@U0LAN0Z89> addfilms star wars, star trek".to_string();
//         let command = parse_command(s);
//         assert!(command.is_ok());
//     }
// }
