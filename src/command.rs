use crate::error::MonitorError;
use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

#[derive(Debug)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

pub struct CommandRunner {
    timeout: Duration,
}

impl CommandRunner {
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    fn find_yuki_in_path(&self) -> Option<String> {
        let result = self.execute("which yuki").ok()?;
        if result.exit_code == 0 {
            Some(result.stdout.trim().to_string())
        } else {
            None
        }
    }

    pub fn execute_yuki(&self, custom_yuki: Option<&str>, args: &str) -> Result<CommandResult> {
        if let Some(yuki_cmd) = custom_yuki {
            let path = Path::new(yuki_cmd);
            if path.exists() {
                let command = format!("{} {}", yuki_cmd, args);
                return self.execute(&command);
            } else {
                eprintln!(
                    "Warning: Custom yuki command '{}' does not exist, falling back to PATH search...",
                    yuki_cmd
                );
            }
        }

        if let Some(yuki_path) = self.find_yuki_in_path() {
            let command = format!("{} {}", yuki_path, args);
            return self.execute(&command);
        }

        Err(MonitorError::CommandNotFound("yuki".to_string()).into())
    }

    pub fn execute(&self, command_str: &str) -> Result<CommandResult> {
        // Use shell execution for simplicity and to support pipes, redirects, etc.
        let (shell, shell_arg) = if cfg!(target_os = "windows") {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        };

        let mut child = Command::new(shell)
            .arg(shell_arg)
            .arg(command_str)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn command")?;

        // Wait for command with timeout
        let status_code = match child
            .wait_timeout(self.timeout)
            .context("Failed to wait for command")?
        {
            Some(status) => status.code().unwrap_or(-1),
            None => {
                // Timeout occurred, kill the process
                child.kill().context("Failed to kill timed-out process")?;
                child
                    .wait()
                    .context("Failed to wait after killing process")?;
                return Err(MonitorError::CommandTimeout(self.timeout.as_secs()).into());
            }
        };

        // Collect output
        let output = child
            .wait_with_output()
            .context("Failed to collect command output")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(CommandResult {
            stdout,
            stderr,
            exit_code: status_code,
        })
    }
}
