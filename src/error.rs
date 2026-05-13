use thiserror::Error;

#[derive(Error, Debug)]
pub enum MonitorError {
    #[error("Telegram API error: {0}")]
    TelegramError(String),

    #[error("USTC mirrors API error: {0}")]
    ApiError(String),
}
