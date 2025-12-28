use thiserror::Error;

#[derive(Error, Debug)]
pub enum MonitorError {
    #[error("Command timed out after {0} seconds")]
    CommandTimeout(u64),

    #[error("Telegram API error: {0}")]
    TelegramError(String),
}
