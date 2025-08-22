use std::env;
use crate::error::{BotError, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub groupme_bot_id: String,
    pub groupme_bot_name: String,
    pub sheet_id: String,
    pub calendar_webcal_url: String,
    pub google_api_key: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let groupme_bot_id = env::var("GROUPME_BOT_ID")
            .map_err(|_| BotError::EnvVar("GROUPME_BOT_ID".to_string()))?;
        
        let groupme_bot_name = env::var("GROUPME_BOT_NAME")
            .map_err(|_| BotError::EnvVar("GROUPME_BOT_NAME".to_string()))?;
        
        let sheet_id = env::var("SHEET_ID")
            .map_err(|_| BotError::EnvVar("SHEET_ID".to_string()))?;
        
        let calendar_webcal_url = env::var("CALENDAR_WEBCAL_URL")
            .map_err(|_| BotError::EnvVar("CALENDAR_WEBCAL_URL".to_string()))?;
        
        let google_api_key = env::var("GOOGLE_API_KEY")
            .map_err(|_| BotError::EnvVar("GOOGLE_API_KEY".to_string()))?;
        
        let port = env::var("PORT")
            .unwrap_or_else(|_| "18080".to_string())
            .parse()
            .map_err(|_| BotError::EnvVar("PORT must be a valid number".to_string()))?;

        // Basic validation
        if groupme_bot_id.is_empty() {
            return Err(BotError::EnvVar("GROUPME_BOT_ID cannot be empty".to_string()));
        }
        
        if google_api_key.is_empty() {
            return Err(BotError::EnvVar("GOOGLE_API_KEY cannot be empty".to_string()));
        }

        Ok(Config {
            groupme_bot_id,
            groupme_bot_name,
            sheet_id,
            calendar_webcal_url,
            google_api_key,
            port,
        })
    }
}
