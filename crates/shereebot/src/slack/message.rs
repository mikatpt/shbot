#![allow(dead_code)]

use tracing::info;

use super::{
    app_mentions::Response,
    events::{ChannelType, File},
};
use crate::{manager::Manager, server::State, Result};

const HELLO: &str =
    ":wave: Hi! I'm ShereeBot. Sheree's brother built me to help her manage your film assignments!
For a list of my commands, type `help`";

const HELP: &str = "
To request work, message me and say `request-work`.
To deliver your work, message me and say `deliver-work`.

As soon as there's work ready to be picked up, I'll let you know!";

pub(crate) struct Message {
    state: State,
    user: String,
    text: String,
    channel_type: ChannelType,
    subtype: Option<String>,
    files: Option<Vec<File>>,
}

impl Message {
    #[rustfmt::skip]
    pub(crate) fn new(
        state: State,
        user: String,
        text: String,
        channel_type: ChannelType,
        subtype: Option<String>,
        files: Option<Vec<File>>,
    ) -> Self {
        Self { state, user, text, channel_type, subtype, files }
    }

    /// Dispatches event and responds to user async.
    #[tracing::instrument(name = "message", skip_all)]
    pub(crate) async fn handle_event(&self) -> Result<()> {
        info!("Handling message from {}: {}", self.user, self.text);

        let manager = Manager::new(self.state.clone());

        let text = self.text.trim().to_lowercase();

        let mut msg = "".to_string();
        msg = match text.as_ref() {
            "request-work" => manager.request_work(&self.user, "0", &self.user).await,
            "deliver-work" => manager.deliver_work(&self.user).await,
            "help" => HELP.to_string(),
            _ => msg,
        };

        if let Some(files) = &self.files {
            msg = manager.insert_from_files(files).await?
        }

        if msg.is_empty() {
            msg = HELLO.to_string();
        }

        self.send_response(msg).await?;

        Ok(())
    }

    async fn send_response(&self, msg: String) -> Result<()> {
        let res = Response::new(self.user.clone(), msg, None);
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
