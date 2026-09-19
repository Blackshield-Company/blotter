//! Per-city mapping config (TOML).

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::schema::{is_standard_category, STANDARD_CATEGORIES};

/// Which source CSV column feeds which schema field.
/// Values are column *names* in the source CSV header.
/// Optional fields may be omitted.
#[derive(Debug, Clone, Deserialize)]
pub struct ColumnMap {
    pub incident_id: String,
    pub datetime: String,
    pub category: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub latitude: Option<String>,
    #[serde(default)]
    pub longitude: Option<String>,
    #[serde(default)]
    pub district_or_neighborhood: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
}

/// A mapping config for one source city.
#[derive(Debug, Clone, Deserialize)]
pub struct MappingConfig {
    /// City name stamped onto every output record's `source_city` field.
    pub source_city: String,

    /// chrono-compatible datetime format string matching the source column,
    /// e.g. "%m/%d/%Y %H:%M" or "%Y-%m-%d". Date-only formats are allowed.
    pub datetime_format: String,

    #[serde(rename = "columns")]
    pub columns: ColumnMap,

    /// Raw source category string -> standard category.
    /// Lookup is case-insensitive and trims whitespace.
    #[serde(rename = "categories", default)]
    pub categories: HashMap<String, String>,
}

impl MappingConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading mapping config {}", path.display()))?;
        let cfg: MappingConfig = toml::from_str(&text)
            .with_context(|| format!("parsing mapping config {}", path.display()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> Result<()> {
        if self.source_city.trim().is_empty() {
            bail!("mapping config: source_city must not be empty");
        }
        if self.datetime_format.trim().is_empty() {
            bail!("mapping config: datetime_format must not be empty");
        }
        for (raw, mapped) in &self.categories {
            if !is_standard_category(mapped) {
                bail!(
                    "mapping config: category {:?} maps to {:?}, which is not a \
                     standard category (one of: {})",
                    raw,
                    mapped,
                    STANDARD_CATEGORIES.join(", ")
                );
            }
        }
        Ok(())
    }

    /// Remap a raw source category to a standard category.
    /// Unknown categories fall back to "other".
    pub fn remap_category(&self, raw: &str) -> String {
        let key = raw.trim().to_lowercase();
        for (k, v) in &self.categories {
            if k.trim().to_lowercase() == key {
                return v.clone();
            }
        }
        "other".to_string()
    }
}

/// Parse a source datetime string using the config's format string.
/// Supports both full datetime formats and date-only formats.
/// Returns an ISO-ish string: "YYYY-MM-DDTHH:MM:SS".
pub fn parse_datetime(raw: &str, format: &str) -> Result<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(anyhow!("empty datetime value"));
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(raw, format) {
        return Ok(dt.format("%Y-%m-%dT%H:%M:%S").to_string());
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(raw, format) {
        return Ok(d
            .and_hms_opt(0, 0, 0)
            .expect("midnight is always valid")
            .format("%Y-%m-%dT%H:%M:%S")
            .to_string());
    }
    Err(anyhow!(
        "datetime {:?} does not match format {:?}",
        raw,
        format
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_datetime_with_config_format() {
        let out = parse_datetime("03/14/2026 21:47", "%m/%d/%Y %H:%M").unwrap();
        assert_eq!(out, "2026-03-14T21:47:00");
    }

    #[test]
    fn parses_date_only_format_as_midnight() {
        let out = parse_datetime("2026-03-14", "%Y-%m-%d").unwrap();
        assert_eq!(out, "2026-03-14T00:00:00");
    }

    #[test]
    fn rejects_mismatched_datetime() {
        assert!(parse_datetime("03/14/2026", "%Y-%m-%d %H:%M").is_err());
    }

    #[test]
    fn rejects_empty_datetime() {
        assert!(parse_datetime("  ", "%Y-%m-%d").is_err());
    }
}
