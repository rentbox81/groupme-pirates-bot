use thiserror::Error;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("JSON parsing failed: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Date parsing failed: {0}")]
    DateParse(#[from] chrono::ParseError),
    
    #[error("Environment variable missing: {0}")]
    EnvVar(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Google API error: {0}")]
    GoogleApi(String),
    
    #[error("GroupMe API error: {0}")]
    GroupMeApi(String),
    
    #[error("No event found for the specified criteria")]
    EventNotFound,
    
    #[error("{0}")]
    InvalidCommand(String),
}

pub type Result<T> = std::result::Result<T, BotError>;
