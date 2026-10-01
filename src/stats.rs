//! `blotter stats` and `blotter to-json`.

use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeMap;

use crate::cli::{StatsArgs, ToJsonArgs};
use crate::normalize::read_normalized;
use crate::schema::NormalizedIncident;

/// Counts over a normalized dataset. BTreeMaps keep output deterministic.
#[derive(Debug, Default, PartialEq, Serialize)]
pub struct Stats {
    pub total: usize,
    pub by_category: BTreeMap<String, usize>,
    pub by_month: BTreeMap<String, usize>,
    pub by_district: BTreeMap<String, usize>,
}

pub fn compute_stats(records: &[NormalizedIncident]) -> Stats {
    let mut s = Stats {
        total: records.len(),
        ..Stats::default()
    };
    for r in records {
        *s.by_category.entry(r.category.clone()).or_default() += 1;
        let month = month_of(&r.datetime);
        *s.by_month.entry(month).or_default() += 1;
        let district = r
            .district_or_neighborhood
            .clone()
            .unwrap_or_else(|| "(unknown)".to_string());
        *s.by_district.entry(district).or_default() += 1;
    }
    s
}

/// "YYYY-MM-DDTHH:MM:SS" -> "YYYY-MM". Anything unexpected -> "(unparseable)".
fn month_of(datetime: &str) -> String {
    if datetime.len() >= 7 && datetime.as_bytes().get(4) == Some(&b'-') {
        datetime[..7].to_string()
    } else {
        "(unparseable)".to_string()
    }
}

pub fn run(args: &StatsArgs) -> Result<()> {
    let records = read_normalized(&args.input)?;
    let stats = compute_stats(&records);

    println!("total incidents: {}", stats.total);
    print_section("by category", &stats.by_category);
    print_section("by month", &stats.by_month);
    print_section("by district", &stats.by_district);
    Ok(())
}

fn print_section(title: &str, counts: &BTreeMap<String, usize>) {
    println!("\n{}:", title);
    let mut sorted: Vec<(&String, &usize)> = counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    let width = sorted.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    for (k, v) in sorted {
        println!("  {:<width$}  {}", k, v, width = width);
    }
}

pub fn run_to_json(args: &ToJsonArgs) -> Result<()> {
    let records = read_normalized(&args.input)?;
    let json = serde_json::to_string_pretty(&records)?;
    println!("{}", json);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(id: &str, dt: &str, cat: &str, district: Option<&str>) -> NormalizedIncident {
        NormalizedIncident {
            incident_id: id.to_string(),
            datetime: dt.to_string(),
            category: cat.to_string(),
            description: String::new(),
            latitude: None,
            longitude: None,
            district_or_neighborhood: district.map(|s| s.to_string()),
            source_city: "Testville".to_string(),
            source_url: None,
        }
    }

    #[test]
    fn stats_counts_are_correct() {
        let records = vec![
            rec("1", "2026-01-05T10:00:00", "theft", Some("Northloop")),
            rec("2", "2026-01-20T11:00:00", "theft", Some("Northloop")),
            rec("3", "2026-02-14T12:00:00", "assault", Some("Millhurst")),
            rec("4", "2026-02-14T13:00:00", "vandalism", None),
        ];
        let s = compute_stats(&records);
        assert_eq!(s.total, 4);
        assert_eq!(s.by_category.get("theft"), Some(&2));
        assert_eq!(s.by_category.get("assault"), Some(&1));
        assert_eq!(s.by_category.get("vandalism"), Some(&1));
        assert_eq!(s.by_month.get("2026-01"), Some(&2));
        assert_eq!(s.by_month.get("2026-02"), Some(&2));
        assert_eq!(s.by_district.get("Northloop"), Some(&2));
        assert_eq!(s.by_district.get("Millhurst"), Some(&1));
        assert_eq!(s.by_district.get("(unknown)"), Some(&1));
    }

    #[test]
    fn month_extraction_handles_malformed() {
        assert_eq!(month_of("2026-03-14T21:47:00"), "2026-03");
        assert_eq!(month_of("garbage"), "(unparseable)");
    }
}
