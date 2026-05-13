mod config;
mod error;
mod mirrors_status;
mod telegram;

use anyhow::Result;
use config::Config;
use mirrors_status::MirrorsStatusChecker;
use telegram::TelegramClient;

fn main() -> Result<()> {
    let config = Config::from_env()?;
    config.validate()?;

    let threshold_days = 7;
    let whitelist = config.parse_whitelist();
    let checker = MirrorsStatusChecker::new(threshold_days, whitelist);

    println!("Fetching mirror status from API...");
    let outdated = checker.check()?;

    if !outdated.is_empty() {
        println!(
            "\n✓ Found {} repo(s) failed to sync more than {} days! Sending Telegram alert...",
            outdated.len(),
            threshold_days
        );

        let telegram = TelegramClient::new(
            config.telegram_bot_token.clone(),
            config.telegram_chat_id.clone(),
        );

        let repo_names: Vec<String> = outdated.iter().map(|e| e.name.clone()).collect();
        let message = format!(
            "*[USTC LUG Mirrors]* Repo(s) failed to sync more than {} days: {}",
            threshold_days,
            repo_names.join(", ")
        );

        telegram.send_message(&message)?;

        println!("✓ Alert sent successfully!");
    } else {
        println!(
            "\n✓ No repo failed to sync successfully more than {} days.",
            threshold_days
        );
    }

    Ok(())
}
