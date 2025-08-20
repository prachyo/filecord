use anyhow::*;
use reqwest::Client as Http;
use serde_json::json;
use crate::types::{Msg, Thread};
use tracing::instrument;

pub struct DiscordClient {
    http: Http,
    token: String,
    base: String,
}

impl DiscordClient {
    pub fn new(bot_token: impl Into<String>) -> Self {
        Self {
            http: Http::new(),
            token: bot_token.into(),
            base: "https://discord.com/api/v10".to_string(),
        }
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.bearer_auth(&self.token)
    }

    #[instrument(skip(self))]
    pub async fn post_message(&self, channel_id: &str, content: &str) -> Result<Msg> {
        let url = format!("{}/channels/{}/messages", self.base, channel_id);
        let body = json!({ "content": content });
        let resp = self.auth(self.http.post(url)).json(&body).send().await?;
        ensure!(resp.status().is_success(), "post_message failed: {}", resp.status());
        Ok(resp.json::<Msg>().await?)
    }

    #[instrument(skip(self))]
    pub async fn edit_message(&self, channel_id: &str, message_id: &str, content: &str) -> Result<()> {
        let url = format!("{}/channels/{}/messages/{}", self.base, channel_id, message_id);
        let body = json!({ "content": content });
        let resp = self.auth(self.http.patch(url)).json(&body).send().await?;
        ensure!(resp.status().is_success(), "edit_message failed: {}", resp.status());
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn pin_message(&self, channel_id: &str, message_id: &str) -> Result<()> {
        let url = format!("{}/channels/{}/pins/{}", self.base, channel_id, message_id);
        let resp = self.auth(self.http.put(url)).send().await?;
        ensure!(resp.status().is_success(), "pin_message failed: {}", resp.status());
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn add_reaction(&self, channel_id: &str, message_id: &str, emoji: &str) -> Result<()> {
        let encoded = urlencoding::encode(emoji);
        let url = format!("{}/channels/{}/messages/{}/reactions/{}/@me", self.base, channel_id, message_id, encoded);
        let resp = self.auth(self.http.put(url)).send().await?;
        ensure!(resp.status().is_success(), "add_reaction failed: {}", resp.status());
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn create_thread(&self, parent_channel_id: &str, name: &str) -> Result<Thread> {
        let url = format!("{}/channels/{}/threads", self.base, parent_channel_id);
        let body = json!({ "name": name, "type": 11 }); // public thread by default
        let resp = self.auth(self.http.post(url)).json(&body).send().await?;
        ensure!(resp.status().is_success(), "create_thread failed: {}", resp.status());
        Ok(resp.json::<Thread>().await?)
    }
}
