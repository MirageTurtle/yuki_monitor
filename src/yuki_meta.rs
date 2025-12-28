use anyhow::{Context, Result};
use chrono::{DateTime, Duration, FixedOffset, Utc};

#[derive(Debug, Clone)]
pub struct MetaEntry {
    pub name: String,
    pub last_success: DateTime<FixedOffset>,
}

#[derive(Debug)]
pub struct OutdatedEntry {
    pub name: String,
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
        let last_success_pos = header
            .find("LAST-SUCCESS")
            .context("LAST-SUCCESS column not found")?;

        // Parse each data line
        for line in &lines[1..] {
            if line.trim().is_empty() {
                continue;
            }

            // Extract name: from start to where the name field ends (find first multiple spaces)
            let name = line
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();

            if name.is_empty() {
                continue;
            }

            // Extract LAST-SUCCESS timestamp starting from the position we found in header
            let remaining = &line[last_success_pos.min(line.len())..];
            let last_success_str = remaining
                .split_whitespace()
                .next()
                .context("Failed to find LAST-SUCCESS timestamp")?;

            // Parse LAST-SUCCESS timestamp (preserving original timezone)
            let last_success = DateTime::parse_from_rfc3339(last_success_str)
                .context(format!(
                    "Failed to parse LAST-SUCCESS timestamp for {}: {}",
                    name, last_success_str
                ))?;

            entries.push(MetaEntry {
                name,
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
                let age = now.signed_duration_since(entry.last_success.with_timezone(&Utc));
                if age > threshold {
                    Some(OutdatedEntry { name: entry.name })
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
        assert_eq!(
            entries[0].last_success.to_rfc3339(),
            "2025-12-28T06:26:13+08:00"
        );
        assert_eq!(
            entries[1].last_success.to_rfc3339(),
            "2025-12-26T02:19:00+08:00"
        );
    }
}
