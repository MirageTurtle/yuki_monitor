use crate::config::MatchType;
use anyhow::{Context, Result};
use regex::Regex;
use serde_json::Value;

#[derive(Debug)]
pub struct MatchResult {
    pub matched: bool,
    pub matched_content: Option<String>,
}

pub struct Matcher {
    pattern: String,
    match_type: MatchType,
    invert: bool,
}

impl Matcher {
    pub fn new(pattern: String, match_type: MatchType, invert: bool) -> Self {
        Self {
            pattern,
            match_type,
            invert,
        }
    }

    pub fn check(&self, output: &str) -> Result<MatchResult> {
        let (matched, content) = match self.match_type {
            MatchType::Contains => (self.check_contains(output), Some(output.to_string())),
            MatchType::Regex => self.check_regex(output)?,
            MatchType::JsonContains => {
                (self.check_json_contains(output)?, Some(output.to_string()))
            }
        };

        // Apply invert logic
        let final_matched = if self.invert { !matched } else { matched };

        Ok(MatchResult {
            matched: final_matched,
            matched_content: if final_matched { content } else { None },
        })
    }

    fn check_contains(&self, output: &str) -> bool {
        output.contains(&self.pattern)
    }

    fn check_regex(&self, output: &str) -> Result<(bool, Option<String>)> {
        let re = Regex::new(&self.pattern).context("Invalid regex pattern")?;

        if let Some(captures) = re.captures(output) {
            let matched_text = captures.get(0).map(|m| m.as_str().to_string());
            Ok((true, matched_text))
        } else {
            Ok((false, None))
        }
    }

    fn check_json_contains(&self, output: &str) -> Result<bool> {
        // Parse output as JSON
        let output_json: Value =
            serde_json::from_str(output).context("Failed to parse command output as JSON")?;

        // Parse pattern as JSON
        let pattern_json: Value =
            serde_json::from_str(&self.pattern).context("Failed to parse pattern as JSON")?;

        // Check if pattern is contained in output
        Ok(json_contains(&output_json, &pattern_json))
    }
}

// Helper function to check if pattern JSON is contained in output JSON
fn json_contains(output: &Value, pattern: &Value) -> bool {
    match (output, pattern) {
        // If pattern is an object, check all its key-value pairs exist in output
        (Value::Object(output_map), Value::Object(pattern_map)) => {
            pattern_map.iter().all(|(key, pattern_value)| {
                output_map
                    .get(key)
                    .map(|output_value| json_contains(output_value, pattern_value))
                    .unwrap_or(false)
            })
        }
        // If pattern is an array, check if all pattern elements are in output array
        (Value::Array(output_arr), Value::Array(pattern_arr)) => {
            pattern_arr.iter().all(|pattern_item| {
                output_arr
                    .iter()
                    .any(|output_item| json_contains(output_item, pattern_item))
            })
        }
        // For primitives, check equality
        _ => output == pattern,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_match() {
        let matcher = Matcher::new("error".to_string(), MatchType::Contains, false);
        let result = matcher.check("This is an error message").unwrap();
        assert!(result.matched);
    }

    #[test]
    fn test_contains_no_match() {
        let matcher = Matcher::new("warning".to_string(), MatchType::Contains, false);
        let result = matcher.check("This is an error message").unwrap();
        assert!(!result.matched);
    }

    #[test]
    fn test_regex_match() {
        let matcher = Matcher::new(r"\d{3}".to_string(), MatchType::Regex, false);
        let result = matcher.check("Error code: 404").unwrap();
        assert!(result.matched);
        assert_eq!(result.matched_content, Some("404".to_string()));
    }

    #[test]
    fn test_invert_match() {
        let matcher = Matcher::new("success".to_string(), MatchType::Contains, true);
        let result = matcher.check("This is an error message").unwrap();
        assert!(result.matched); // Should match because "success" is NOT in the output
    }

    #[test]
    fn test_json_contains() {
        let matcher = Matcher::new(
            r#"{"status":"error"}"#.to_string(),
            MatchType::JsonContains,
            false,
        );
        let result = matcher.check(r#"{"status":"error","code":500}"#).unwrap();
        assert!(result.matched);
    }
}
