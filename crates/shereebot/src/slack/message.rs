#![allow(dead_code)]
use crate::{manager::Manager, server::State, store::Client, Result};

use super::{
    app_mentions::Response,
    events::{ChannelType, File},
};

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
        let chan = self.user.clone();
        let manager = Manager::new(self.state.clone());

        let msg = if self.text.contains("request-work") {
            manager.request_work(&self.user, "0", &self.user).await
        } else if self.text.contains("deliver") {
            manager.deliver_work(&self.user).await
        } else if let Some(files) = &self.files {
            manager.insert_from_files(files).await?
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
