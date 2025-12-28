use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum MonitorError {
    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Command timed out after {0} seconds")]
    CommandTimeout(u64),

    #[error("Pattern matching failed: {0}")]
    PatternError(String),

    #[error("Telegram API error: {0}")]
    TelegramError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
}
