use crate::error::MonitorError;
use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct SendMessageRequest {
    chat_id: String,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parse_mode: Option<String>,
}

#[derive(Deserialize, Debug)]
struct TelegramResponse {
    ok: bool,
    #[serde(default)]
    description: Option<String>,
}

pub struct TelegramClient {
    bot_token: String,
    chat_id: String,
    client: Client,
}

impl TelegramClient {
    pub fn new(bot_token: String, chat_id: String) -> Self {
        Self {
            bot_token,
            chat_id,
            client: Client::new(),
        }
    }

    pub fn send_message(&self, message: &str) -> Result<()> {
        let url = format!(
            "https://mtelegra.mirageturtle.top/bot{}/sendMessage",
            self.bot_token
        );

        let request = SendMessageRequest {
            chat_id: self.chat_id.clone(),
            text: message.to_string(),
            parse_mode: Some("Markdown".to_string()),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .context("Failed to send HTTP request to Telegram API")?;

        let status = response.status();
        let telegram_response: TelegramResponse = response
            .json()
            .context("Failed to parse Telegram API response")?;

        if !telegram_response.ok {
            let error_msg = telegram_response
                .description
                .unwrap_or_else(|| format!("HTTP {}", status));
            return Err(MonitorError::TelegramError(error_msg).into());
        }

        Ok(())
    }
}
