mod command;
mod config;
mod error;
mod telegram;
mod yuki_meta;

use anyhow::Result;
use command::CommandRunner;
use config::Config;
use telegram::TelegramClient;
use yuki_meta::YukiMetaChecker;

fn main() -> Result<()> {
    // 1. Load and validate configuration from environment variables
    let config = Config::from_env()?;
    config.validate()?;

    // 2. Execute yuki meta ls command with path fallback
    println!("Executing: yuki meta ls");
    let runner = CommandRunner::new(30);
    let result = runner.execute_yuki(config.yuki_command.as_deref(), "meta ls")?;

    if !result.stdout.is_empty() {
        println!("Command stdout: {}", result.stdout.trim());
    }
    if !result.stderr.is_empty() {
        eprintln!("Command stderr: {}", result.stderr.trim());
    }
    println!("Exit code: {}", result.exit_code);

    // 3. Parse output and find repos that failed to sync more than 7 days
    let threshold_days = 7;
    let whitelist = config.parse_whitelist();
    let checker = YukiMetaChecker::new(threshold_days, whitelist);
    let outdated = checker.check(&result.stdout)?;

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

        // Format repo names as comma-separated list
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
