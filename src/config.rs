use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub telegram_bot_token: String,
    pub telegram_chat_id: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        envy::from_env::<Config>().context("Failed to parse environment variables. Make sure all required variables are set:\n  - TELEGRAM_BOT_TOKEN\n  - TELEGRAM_CHAT_ID")
    }

    pub fn validate(&self) -> Result<()> {
        if self.telegram_bot_token.is_empty() {
            anyhow::bail!("TELEGRAM_BOT_TOKEN cannot be empty");
        }
        if self.telegram_chat_id.is_empty() {
            anyhow::bail!("TELEGRAM_CHAT_ID cannot be empty");
        }
        Ok(())
    }
}
