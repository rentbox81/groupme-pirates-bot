use std::env;
use crate::error::{BotError, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub groupme_bot_id: String,
    pub groupme_bot_name: String,
    pub sheet_id: String,
    pub calendar_webcal_url: Option<String>,
    pub google_api_key: String,
    pub port: u16,
    pub reminder_start_hour: u32,
    pub reminder_end_hour: u32,
    pub admin_user_id: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let groupme_bot_id = env::var("GROUPME_BOT_ID")
            .map_err(|_| BotError::EnvVar("GROUPME_BOT_ID".to_string()))?;
        
        let groupme_bot_name = env::var("GROUPME_BOT_NAME")
            .map_err(|_| BotError::EnvVar("GROUPME_BOT_NAME".to_string()))?;
        
        let sheet_id = env::var("SHEET_ID")
            .map_err(|_| BotError::EnvVar("SHEET_ID".to_string()))?;
        
        let calendar_webcal_url = env::var("CALENDAR_WEBCAL_URL").ok();
        
        let google_api_key = env::var("GOOGLE_API_KEY")
            .map_err(|_| BotError::EnvVar("GOOGLE_API_KEY".to_string()))?;
        
        let port = env::var("PORT")
            .unwrap_or_else(|_| "18080".to_string())
            .parse()
            .map_err(|_| BotError::EnvVar("PORT must be a valid number".to_string()))?;

        let reminder_start_hour = env::var("REMINDER_START_HOUR")
            .unwrap_or_else(|_| "9".to_string())
            .parse()
            .map_err(|_| BotError::EnvVar("REMINDER_START_HOUR must be a valid number (0-23)".to_string()))?;

        let reminder_end_hour = env::var("REMINDER_END_HOUR")
            .unwrap_or_else(|_| "21".to_string())
            .parse()
            .map_err(|_| BotError::EnvVar("REMINDER_END_HOUR must be a valid number (0-23)".to_string()))?;

        // Basic validation
        if groupme_bot_id.is_empty() {
            return Err(BotError::EnvVar("GROUPME_BOT_ID cannot be empty".to_string()));
        }
        
        if google_api_key.is_empty() {
            return Err(BotError::EnvVar("GOOGLE_API_KEY cannot be empty".to_string()));
        }

        if reminder_start_hour >= 24 {
            return Err(BotError::EnvVar("REMINDER_START_HOUR must be between 0 and 23".to_string()));
        }

        if reminder_end_hour > 24 {
            return Err(BotError::EnvVar("REMINDER_END_HOUR must be between 1 and 24".to_string()));
        }

        if reminder_start_hour >= reminder_end_hour {
            return Err(BotError::EnvVar("REMINDER_START_HOUR must be less than REMINDER_END_HOUR".to_string()));
        }

        let admin_user_id = env::var("ADMIN_USER_ID")
            .map_err(|_| BotError::EnvVar("ADMIN_USER_ID".to_string()))?;


        Ok(Config {
            groupme_bot_id,
            groupme_bot_name,
            sheet_id,
            calendar_webcal_url,
            google_api_key,
            port,
            reminder_start_hour,
            reminder_end_hour,
            admin_user_id,
        })
    }
}
