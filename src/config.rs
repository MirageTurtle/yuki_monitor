use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MatchType {
    Contains,
    Regex,
    JsonContains,
}

impl Default for MatchType {
    fn default() -> Self {
        MatchType::Contains
    }
}

fn default_timeout() -> u64 {
    30
}

fn default_invert_match() -> bool {
    false
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub telegram_bot_token: String,
    pub telegram_chat_id: String,
    pub monitor_command: String,
    pub monitor_pattern: String,

    #[serde(default)]
    pub monitor_match_type: MatchType,

    #[serde(default)]
    pub monitor_alert_message: Option<String>,

    #[serde(default = "default_invert_match")]
    pub monitor_invert_match: bool,

    #[serde(default = "default_timeout")]
    pub monitor_timeout_secs: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        envy::from_env::<Config>().context("Failed to parse environment variables. Make sure all required variables are set:\n  - TELEGRAM_BOT_TOKEN\n  - TELEGRAM_CHAT_ID\n  - MONITOR_COMMAND\n  - MONITOR_PATTERN")
    }

    pub fn validate(&self) -> Result<()> {
        if self.telegram_bot_token.is_empty() {
            anyhow::bail!("TELEGRAM_BOT_TOKEN cannot be empty");
        }
        if self.telegram_chat_id.is_empty() {
            anyhow::bail!("TELEGRAM_CHAT_ID cannot be empty");
        }
        if self.monitor_command.is_empty() {
            anyhow::bail!("MONITOR_COMMAND cannot be empty");
        }
        if self.monitor_pattern.is_empty() {
            anyhow::bail!("MONITOR_PATTERN cannot be empty");
        }
        if self.monitor_timeout_secs == 0 {
            anyhow::bail!("MONITOR_TIMEOUT_SECS must be greater than 0");
        }
        Ok(())
    }
}
