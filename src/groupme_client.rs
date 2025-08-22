use reqwest::Client;
use tracing::{info, error};

use crate::config::Config;
use crate::error::{BotError, Result};
use crate::models::GroupMePostMessage;

#[derive(Clone)]
pub struct GroupMeClient {
    client: Client,
    config: Config,
}

impl GroupMeClient {
    pub fn new(config: Config) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub async fn send_message(&self, message: &str) -> Result<()> {
        let url = "https://api.groupme.com/v3/bots/post";
        
        let payload = GroupMePostMessage {
            bot_id: self.config.groupme_bot_id.clone(),
            text: message.to_string(),
        };

        info!("Sending message to GroupMe: '{}'", message);

        let response = self.client
            .post(url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            info!("Successfully sent message to GroupMe");
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to send GroupMe message. Status: {} - {}", status, error_text);
            Err(BotError::GroupMeApi(format!("GroupMe API returned {}: {}", status, error_text)))
        }
    }
}
