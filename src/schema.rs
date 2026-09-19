//! The one common schema every normalized dataset uses.

use serde::{Deserialize, Serialize};

/// Standard categories every source category is remapped into.
pub const STANDARD_CATEGORIES: [&str; 10] = [
    "theft",
    "assault",
    "burglary",
    "robbery",
    "vandalism",
    "fraud",
    "homicide",
    "drug",
    "traffic",
    "other",
];

pub fn is_standard_category(s: &str) -> bool {
    STANDARD_CATEGORIES.contains(&s)
}

/// A single normalized incident record.
///
/// `datetime` is stored as an ISO 8601 / RFC 3339-ish string
/// (`YYYY-MM-DDTHH:MM:SS`) produced from the source value using the
/// format string in the mapping config.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NormalizedIncident {
    pub incident_id: String,
    pub datetime: String,
    pub category: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub district_or_neighborhood: Option<String>,
    pub source_city: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
}

pub const SCHEMA_FIELDS: [&str; 9] = [
    "incident_id",
    "datetime",
    "category",
    "description",
    "latitude",
    "longitude",
    "district_or_neighborhood",
    "source_city",
    "source_url",
];
