#![allow(dead_code)]

use tracing::info;

use super::{
    app_mentions::Response,
    events::{ChannelType, File},
};
use crate::{manager::Manager, server::State, store::Client, Result};

const HELLO: &str =
    ":wave: Hi! I'm ShereeBot. Sheree's brother built me to help her manage your film assignments!
For a list of my commands, type @ShereeBot help";

const HELP: &str = "
To request work, message me and say `request-work`.
To deliver your work, message me and say `deliver-work`.

As soon as there's work ready to be picked up, I'll let you know!";

pub(crate) struct Message<T: Client> {
    state: State<T>,
    user: String,
    text: String,
    channel_type: ChannelType,
    subtype: Option<String>,
    files: Option<Vec<File>>,
}

/// only handle files for now!
impl<T: Client> Message<T> {
    #[rustfmt::skip]
    pub(crate) fn new(
        state: State<T>,
        user: String,
        text: String,
        channel_type: ChannelType,
        subtype: Option<String>,
        files: Option<Vec<File>>,
    ) -> Self {
        Self { state, user, text, channel_type, subtype, files }
    }

    #[tracing::instrument(name = "message", skip_all)]
    pub(crate) async fn handle_event(&self) -> Result<()> {
        // ignore bot messages to avoid infinite loop.
        if let Some(subtype) = &self.subtype {
            if subtype == "bot_message" {
                return Ok(());
            }
        };

        info!("Handling message from {}: {}", self.user, self.text);
        let chan = self.user.clone();
        let manager = Manager::new(self.state.clone());

        let text = self.text.to_lowercase();

        let msg = if text.contains("request-work") {
            manager.request_work(&self.user, "0", &self.user).await
        } else if text.contains("deliver") {
            manager.deliver_work(&self.user).await
        } else if let Some(files) = &self.files {
            manager.insert_from_files(files).await?
        } else if text.contains("help") {
            HELP.to_string()
        } else if contains_greeting(&text) {
            HELLO.to_string()
        } else {
            "Invalid request!".to_string()
        };
        self.send_response(Response::new(chan, msg, None)).await?;

        Ok(())
    }

    async fn send_response(&self, res: Response) -> Result<()> {
        self.state
            .req_client
            .post("https://slack.com/api/chat.postMessage")
            .bearer_auth(&self.state.oauth_token)
            .json(&res)
            .send()
            .await?;
        Ok(())
    }
}

fn contains_greeting(msg: &str) -> bool {
    let greetings = ["hi", "hello", "hey", "hola"];
    for g in greetings {
        if msg.contains(g) {
            return true;
        }
    }
    false
}
