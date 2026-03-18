use thiserror::Error;

#[derive(Error, Debug)]
pub enum MonitorError {
    #[error("Command timed out after {0} seconds")]
    CommandTimeout(u64),

    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("Telegram API error: {0}")]
    TelegramError(String),
}
