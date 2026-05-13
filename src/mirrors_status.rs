use crate::error::MonitorError;
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use std::collections::HashSet;

const STATUS_API_URL: &str = "https://mirrors.ustc.edu.cn/status/json";

#[derive(Debug, Deserialize)]
pub struct MirrorStatus {
    pub name: String,
    #[serde(default, rename = "lastSuccess")]
    pub last_success: i64,
}

#[derive(Debug)]
pub struct OutdatedEntry {
    pub name: String,
}

pub struct MirrorsStatusChecker {
    threshold_days: i64,
    whitelist: HashSet<String>,
}

impl MirrorsStatusChecker {
    pub fn new(threshold_days: i64, whitelist: HashSet<String>) -> Self {
        Self {
            threshold_days,
            whitelist,
        }
    }

    /// Fetch mirror status JSON from USTC mirrors API
    pub fn fetch(&self) -> Result<Vec<MirrorStatus>> {
        let response = reqwest::blocking::get(STATUS_API_URL)
            .map_err(|e| MonitorError::ApiError(format!("HTTP request failed: {}", e)))?;

        let statuses: Vec<MirrorStatus> = response
            .json()
            .map_err(|e| MonitorError::ApiError(format!("JSON parse failed: {}", e)))?;

        Ok(statuses)
    }

    /// Find repos whose lastSuccess is older than threshold
    pub fn find_outdated(&self, entries: Vec<MirrorStatus>) -> Vec<OutdatedEntry> {
        let now = Utc::now();
        let threshold = Duration::days(self.threshold_days);

        entries
            .into_iter()
            .filter_map(|entry| {
                let last_success = DateTime::from_timestamp(entry.last_success, 0);
                let is_outdated = match last_success {
                    Some(ts) => now.signed_duration_since(ts) > threshold,
                    None => true, // invalid timestamp = treat as outdated
                };
                if is_outdated && !self.whitelist.contains(&entry.name) {
                    Some(OutdatedEntry { name: entry.name })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Fetch and check in one call
    pub fn check(&self) -> Result<Vec<OutdatedEntry>> {
        let entries = self.fetch()?;
        Ok(self.find_outdated(entries))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json() {
        let json = r#"[
            {"name":"adoptium.apt","lastSuccess":1778624770},
            {"name":"anthon","lastSuccess":1778609814}
        ]"#;

        let statuses: Vec<MirrorStatus> = serde_json::from_str(json).unwrap();
        assert_eq!(statuses.len(), 2);
        assert_eq!(statuses[0].name, "adoptium.apt");
        assert_eq!(statuses[0].last_success, 1778624770);
        assert_eq!(statuses[1].name, "anthon");
        assert_eq!(statuses[1].last_success, 1778609814);
    }

    #[test]
    fn test_whitelist_filtering() {
        let entries = vec![
            MirrorStatus {
                name: "adoptium.apt".into(),
                last_success: 100000, // very old — outdated
            },
            MirrorStatus {
                name: "anthon".into(),
                last_success: 100000, // very old — outdated
            },
        ];

        let mut whitelist = HashSet::new();
        whitelist.insert("adoptium.apt".to_string());

        let checker = MirrorsStatusChecker::new(7, whitelist);
        let outdated = checker.find_outdated(entries);

        assert_eq!(outdated.len(), 1);
        assert_eq!(outdated[0].name, "anthon");
    }

    #[test]
    fn test_zero_timestamp_treated_as_outdated() {
        let entries = vec![MirrorStatus {
            name: "never-synced".into(),
            last_success: 0,
        }];

        let checker = MirrorsStatusChecker::new(7, HashSet::new());
        let outdated = checker.find_outdated(entries);

        assert_eq!(outdated.len(), 1);
        assert_eq!(outdated[0].name, "never-synced");
    }
}
