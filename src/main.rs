mod command;
mod config;
mod error;
mod matcher;
mod telegram;
mod yuki_meta;

use anyhow::Result;
use command::CommandRunner;
use config::Config;
use matcher::Matcher;
use telegram::TelegramClient;
use yuki_meta::YukiMetaChecker;

fn main() -> Result<()> {
    // 1. Load and validate configuration from environment variables
    let config = Config::from_env()?;
    config.validate()?;

    // 2. Execute the command
    println!("Executing command: {}", config.monitor_command);
    let runner = CommandRunner::new(config.monitor_timeout_secs);
    let result = runner.execute(&config.monitor_command)?;

    // Print command results for debugging
    if !result.stdout.is_empty() {
        println!("Command stdout: {}", result.stdout.trim());
    }
    if !result.stderr.is_empty() {
        eprintln!("Command stderr: {}", result.stderr.trim());
    }
    println!("Exit code: {}", result.exit_code);

    // 3. Check if output matches the pattern or check yuki meta
    if config.monitor_command.contains("yuki meta ls") {
        // Special handling for yuki meta ls command
        let threshold_days: i64 = config
            .monitor_pattern
            .parse()
            .unwrap_or(7);
        let checker = YukiMetaChecker::new(threshold_days);
        let outdated = checker.check(&result.stdout)?;

        if !outdated.is_empty() {
            println!(
                "\n✓ Found {} outdated mirror(s)! Sending Telegram alert...",
                outdated.len()
            );

            let telegram = TelegramClient::new(config.telegram_bot_token.clone(), config.telegram_chat_id.clone());
            let message = checker.format_alert(&outdated);
            telegram.send_message(&message)?;

            println!("✓ Alert sent successfully!");
        } else {
            println!("\nNo outdated mirrors found. No alert sent.");
        }
    } else {
        // Regular pattern matching
        let matcher = Matcher::new(
            config.monitor_pattern.clone(),
            config.monitor_match_type,
            config.monitor_invert_match,
        );
        let match_result = matcher.check(&result.stdout)?;

        if match_result.matched {
            println!("\nPattern matched! Sending Telegram alert...");

            let telegram = TelegramClient::new(config.telegram_bot_token.clone(), config.telegram_chat_id.clone());

            let message = format_alert_message(&config, &match_result, &result);

            telegram.send_message(&message)?;

            println!("✓ Alert sent successfully!");
        } else {
            println!("\nNo match found. No alert sent.");
        }
    }

    Ok(())
}

// TODO
fn format_alert_message(
    config: &Config,
    match_result: &matcher::MatchResult,
    command_result: &command::CommandResult,
) -> String {
    // Use custom message template if provided, otherwise use default
    if let Some(template) = &config.monitor_alert_message {
        // Replace placeholders in the template
        template
            .replace(
                "{match}",
                match_result
                    .matched_content
                    .as_deref()
                    .unwrap_or("(matched)"),
            )
            .replace("{output}", &command_result.stdout)
            .replace("{pattern}", &config.monitor_pattern)
            .replace("{command}", &config.monitor_command)
    } else {
        // Default message format
        let match_type = format!("{:?}", config.monitor_match_type).to_lowercase();
        let invert_text = if config.monitor_invert_match {
            " (inverted)"
        } else {
            ""
        };

        format!(
            "🚨 *Monitor Alert*\n\n\
            *Command:* `{}`\n\
            *Pattern:* `{}`\n\
            *Match Type:* {}{}\n\n\
            *Output:*\n```\n{}\n```",
            config.monitor_command,
            config.monitor_pattern,
            match_type,
            invert_text,
            command_result.stdout.trim()
        )
    }
}
