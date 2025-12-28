use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone)]
pub struct MetaEntry {
    pub name: String,
    pub last_success: DateTime<Utc>,
}

#[derive(Debug)]
pub struct OutdatedEntry {
    pub name: String,
    pub last_success: DateTime<Utc>,
    pub days_since: i64,
}

pub struct YukiMetaChecker {
    threshold_days: i64,
}

impl YukiMetaChecker {
    pub fn new(threshold_days: i64) -> Self {
        Self { threshold_days }
    }

    /// Parse yuki meta ls output and return entries
    pub fn parse_output(&self, output: &str) -> Result<Vec<MetaEntry>> {
        let mut entries = Vec::new();
        let lines: Vec<&str> = output.lines().collect();

        // Skip header line
        if lines.is_empty() {
            return Ok(entries);
        }

        // Find column positions from header
        let header = lines[0];
        let name_start = header.find("NAME").context("NAME column not found")?;
        let last_success_start = header
            .find("LAST-SUCCESS")
            .context("LAST-SUCCESS column not found")?;
        let next_run_start = header
            .find("NEXT-RUN")
            .context("NEXT-RUN column not found")?;

        // Parse each data line
        for line in &lines[1..] {
            if line.trim().is_empty() {
                continue;
            }

            // Extract fields based on column positions
            let name = extract_field(line, name_start, last_success_start).trim();
            let last_success_str =
                extract_field(line, last_success_start, next_run_start).trim();

            if name.is_empty() || last_success_str.is_empty() {
                continue;
            }

            // Parse LAST-SUCCESS timestamp
            let last_success = DateTime::parse_from_rfc3339(last_success_str)
                .context(format!(
                    "Failed to parse LAST-SUCCESS timestamp for {}: {}",
                    name, last_success_str
                ))?
                .with_timezone(&Utc);

            entries.push(MetaEntry {
                name: name.to_string(),
                last_success,
            });
        }

        Ok(entries)
    }

    /// Find entries that have LAST-SUCCESS older than threshold
    pub fn find_outdated(&self, entries: Vec<MetaEntry>) -> Vec<OutdatedEntry> {
        let now = Utc::now();
        let threshold = Duration::days(self.threshold_days);

        entries
            .into_iter()
            .filter_map(|entry| {
                let age = now.signed_duration_since(entry.last_success);
                if age > threshold {
                    Some(OutdatedEntry {
                        name: entry.name,
                        last_success: entry.last_success,
                        days_since: age.num_days(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check output and return outdated entries
    pub fn check(&self, output: &str) -> Result<Vec<OutdatedEntry>> {
        let entries = self.parse_output(output)?;
        Ok(self.find_outdated(entries))
    }

    /// Format outdated entries as alert message
    pub fn format_alert(&self, outdated: &[OutdatedEntry]) -> String {
        if outdated.is_empty() {
            return "No outdated mirrors found.".to_string();
        }

        let mut message = format!(
            "🚨 *Yuki Meta Alert - {} Outdated Mirror(s)*\n\n",
            outdated.len()
        );

        for entry in outdated {
            message.push_str(&format!(
                "• *{}*\n  Last Success: {}\n  Days Since: {}\n\n",
                entry.name,
                entry.last_success.format("%Y-%m-%d %H:%M:%S UTC"),
                entry.days_since
            ));
        }

        message
    }
}

/// Extract field content between two column positions
fn extract_field(line: &str, start: usize, end: usize) -> &str {
    if start >= line.len() {
        return "";
    }
    let actual_end = std::cmp::min(end, line.len());
    &line[start..actual_end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_output() {
        let output = r#"NAME                         UPSTREAM                                                                SYNCING   SIZE       LAST-SUCCESS                NEXT-RUN
adoptium.apt                 https://packages.adoptium.net/artifactory/                              false     149.1GiB   2025-12-28T06:26:13+08:00   2025-12-29T06:25:00+08:00
anthon                       rsync://repo.aosc.io/anthon/                                            false     1.724TiB   2025-12-26T02:19:00+08:00   2025-12-29T02:13:00+08:00"#;

        let checker = YukiMetaChecker::new(7);
        let entries = checker.parse_output(output).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "adoptium.apt");
        assert_eq!(entries[1].name, "anthon");
    }

    #[test]
    fn test_find_outdated() {
        let entries = vec![
            MetaEntry {
                name: "old_mirror".to_string(),
                last_success: Utc::now() - Duration::days(10),
                syncing: false,
            },
            MetaEntry {
                name: "new_mirror".to_string(),
                last_success: Utc::now() - Duration::days(2),
                syncing: false,
            },
        ];

        let checker = YukiMetaChecker::new(7);
        let outdated = checker.find_outdated(entries);

        assert_eq!(outdated.len(), 1);
        assert_eq!(outdated[0].name, "old_mirror");
    }
}
