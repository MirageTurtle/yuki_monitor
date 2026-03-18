use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub telegram_bot_token: String,
    pub telegram_chat_id: String,
    #[serde(default)]
    pub repo_whitelist: Option<String>,
    #[serde(default)]
    pub yuki_command: Option<String>,
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

    pub fn parse_whitelist(&self) -> HashSet<String> {
        self.repo_whitelist
            .as_ref()
            .map(|s| {
                s.split(',')
                    .map(|name| name.trim().to_string())
                    .filter(|name| !name.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }
}
