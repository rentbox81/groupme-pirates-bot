use reqwest::Client;
use tracing::{info, error};

use crate::config::Config;
use crate::error::{BotError, Result};
use crate::models::{GroupMePostMessage, GroupMeMessageInfo};

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

    /// List messages from the group (requires access token and group ID)
    pub async fn list_messages(&self, limit: u32, before_id: Option<String>) -> Result<Vec<GroupMeMessageInfo>> {
        let access_token = self.config.groupme_access_token.as_ref()
            .ok_or_else(|| BotError::Config("GROUPME_ACCESS_TOKEN not configured".to_string()))?;
        let group_id = self.config.groupme_group_id.as_ref()
            .ok_or_else(|| BotError::Config("GROUPME_GROUP_ID not configured".to_string()))?;

        let mut url = format!(
            "https://api.groupme.com/v3/groups/{}/messages?token={}&limit={}",
            group_id, access_token, limit
        );

        if let Some(before) = before_id {
            url.push_str(&format!("&before_id={}", before));
        }

        info!("Fetching messages from GroupMe (limit: {})", limit);

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if response.status().is_success() {
            #[derive(serde::Deserialize)]
            struct MessagesResponse {
                response: MessagesData,
            }
            #[derive(serde::Deserialize)]
            struct MessagesData {
                messages: Vec<GroupMeMessageInfo>,
            }

            let data: MessagesResponse = response.json().await?;
            info!("Fetched {} messages", data.response.messages.len());
            Ok(data.response.messages)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to fetch messages. Status: {} - {}", status, error_text);
            Err(BotError::GroupMeApi(format!("GroupMe API returned {}: {}", status, error_text)))
        }
    }
}
