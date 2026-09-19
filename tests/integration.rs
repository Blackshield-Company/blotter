//! End-to-end tests: normalize the bundled synthetic Exampletown example,
//! then verify schema, category remapping, and stats on the output.

use std::path::{Path, PathBuf};

// These internals are exercised through the compiled binary's modules;
// integration tests drive the CLI binary itself for the real end-to-end path.
use std::process::Command;

fn bin() -> PathBuf {
    // cargo sets CARGO_BIN_EXE_<name> for integration tests
    PathBuf::from(env!("CARGO_BIN_EXE_blotter"))
}

fn examples_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples")
}

fn read_csv(path: &Path) -> Vec<std::collections::HashMap<String, String>> {
    let mut rdr = csv::Reader::from_path(path).expect("open normalized csv");
    let headers = rdr.headers().expect("headers").clone();
    rdr.records()
        .map(|r| {
            let r = r.expect("record");
            headers
                .iter()
                .zip(r.iter())
                .map(|(h, v)| (h.to_string(), v.to_string()))
                .collect()
        })
        .collect()
}

fn normalize_example(tmp: &Path) -> Vec<std::collections::HashMap<String, String>> {
    let out = tmp.join("normalized.csv");
    let status = Command::new(bin())
        .arg("normalize")
        .arg("--config")
        .arg(examples_dir().join("exampletown-mapping.toml"))
        .arg("--input")
        .arg(examples_dir().join("exampletown-incidents.csv"))
        .arg("--output")
        .arg(&out)
        .status()
        .expect("run blotter normalize");
    assert!(status.success(), "normalize exited non-zero");
    read_csv(&out)
}

#[test]
fn bundled_example_normalizes_to_schema() {
    let tmp = tempfile::tempdir().unwrap();
    let rows = normalize_example(tmp.path());

    assert_eq!(rows.len(), 12, "all 12 example rows should normalize");

    // Exact schema header set on every row
    let expected: std::collections::HashSet<&str> = [
        "incident_id",
        "datetime",
        "category",
        "description",
        "latitude",
        "longitude",
        "district_or_neighborhood",
        "source_city",
        "source_url",
    ]
    .into_iter()
    .collect();
    for row in &rows {
        let keys: std::collections::HashSet<&str> = row.keys().map(|k| k.as_str()).collect();
        assert_eq!(keys, expected, "normalized row must use the common schema");
        assert_eq!(row["source_city"], "Exampletown");
    }

    // Spot-check a full row end to end
    let first = &rows[0];
    assert_eq!(first["incident_id"], "EX-2026-0001");
    assert_eq!(first["datetime"], "2026-01-05T09:12:00");
    assert_eq!(first["category"], "theft");
    assert_eq!(first["district_or_neighborhood"], "Northloop");
    assert!(first["latitude"].starts_with("42.36"));

    // Row with missing lat/lon (EX-2026-0011) must have empty optionals
    let no_coords = rows
        .iter()
        .find(|r| r["incident_id"] == "EX-2026-0011")
        .unwrap();
    assert_eq!(no_coords["latitude"], "");
    assert_eq!(no_coords["longitude"], "");
}

#[test]
fn category_remapping_works() {
    let tmp = tempfile::tempdir().unwrap();
    let rows = normalize_example(tmp.path());

    let cat_of = |id: &str| -> String {
        rows.iter()
            .find(|r| r["incident_id"] == id)
            .unwrap_or_else(|| panic!("row {id} missing"))
            .get("category")
            .unwrap()
            .clone()
    };

    assert_eq!(cat_of("EX-2026-0001"), "theft"); // LARCENY / THEFT
    assert_eq!(cat_of("EX-2026-0002"), "theft"); // SHOPLIFTING
    assert_eq!(cat_of("EX-2026-0003"), "assault"); // AGGRAVATED ASSAULT
    assert_eq!(cat_of("EX-2026-0004"), "burglary"); // BURGLARY - RESIDENTIAL
    assert_eq!(cat_of("EX-2026-0005"), "robbery"); // ARMED ROBBERY
    assert_eq!(cat_of("EX-2026-0006"), "vandalism"); // GRAFFITI / VANDALISM
    assert_eq!(cat_of("EX-2026-0007"), "fraud"); // WIRE FRAUD
    assert_eq!(cat_of("EX-2026-0008"), "drug"); // NARCOTICS POSSESSION
    assert_eq!(cat_of("EX-2026-0009"), "traffic"); // TRAFFIC COLLISION - DUI
                                                   // Unmapped source category falls back to "other"
    assert_eq!(cat_of("EX-2026-0010"), "other"); // SUSPICIOUS CIRCUMSTANCES
    assert_eq!(cat_of("EX-2026-0012"), "burglary"); // BURGLARY - COMMERCIAL
}

#[test]
fn stats_output_counts_are_correct() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("normalized.csv");
    let status = Command::new(bin())
        .arg("normalize")
        .arg("--config")
        .arg(examples_dir().join("exampletown-mapping.toml"))
        .arg("--input")
        .arg(examples_dir().join("exampletown-incidents.csv"))
        .arg("--output")
        .arg(&out)
        .status()
        .unwrap();
    assert!(status.success());

    let output = Command::new(bin())
        .arg("stats")
        .arg(&out)
        .output()
        .expect("run blotter stats");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("total incidents: 12"), "{stdout}");

    // Parse "name  count" lines independent of column padding.
    let has_row = |name: &str, count: usize| -> bool {
        stdout.lines().any(|line| {
            let toks: Vec<&str> = line.split_whitespace().collect();
            match toks.split_last() {
                Some((n, rest)) => rest.join(" ") == name && n.parse::<usize>() == Ok(count),
                None => false,
            }
        })
    };

    // by category
    assert!(has_row("theft", 2), "{stdout}");
    assert!(has_row("assault", 2), "{stdout}");
    assert!(has_row("burglary", 2), "{stdout}");
    assert!(has_row("robbery", 1), "{stdout}");
    assert!(has_row("vandalism", 1), "{stdout}");
    assert!(has_row("fraud", 1), "{stdout}");
    assert!(has_row("drug", 1), "{stdout}");
    assert!(has_row("traffic", 1), "{stdout}");
    assert!(has_row("other", 1), "{stdout}");
    // by month
    assert!(has_row("2026-01", 3), "{stdout}");
    assert!(has_row("2026-02", 3), "{stdout}");
    assert!(has_row("2026-03", 6), "{stdout}");
    // by district
    assert!(has_row("Northloop", 4), "{stdout}");
    assert!(has_row("Millhurst", 3), "{stdout}");
    assert!(has_row("Quarry Bend", 3), "{stdout}");
    assert!(has_row("Fog Hollow", 2), "{stdout}");
}

#[test]
fn to_json_emits_valid_json_array() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("normalized.csv");
    let status = Command::new(bin())
        .arg("normalize")
        .arg("--config")
        .arg(examples_dir().join("exampletown-mapping.toml"))
        .arg("--input")
        .arg(examples_dir().join("exampletown-incidents.csv"))
        .arg("--output")
        .arg(&out)
        .status()
        .unwrap();
    assert!(status.success());

    let output = Command::new(bin())
        .arg("to-json")
        .arg(&out)
        .output()
        .expect("run blotter to-json");
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("to-json output is valid JSON");
    let arr = parsed.as_array().expect("top-level JSON array");
    assert_eq!(arr.len(), 12);
    assert_eq!(arr[0]["incident_id"], "EX-2026-0001");
    assert_eq!(arr[0]["source_city"], "Exampletown");
    // latitude is optional in JSON too: EX-2026-0011 had no coords
    let no_coords = arr
        .iter()
        .find(|v| v["incident_id"] == "EX-2026-0011")
        .unwrap();
    assert!(no_coords.get("latitude").is_none());
}

#[test]
fn datetime_format_string_is_respected() {
    // Write a fixture whose dates use a different format, plus a matching config,
    // and verify the config's format string — not a hardcoded one — drives parsing.
    let tmp = tempfile::tempdir().unwrap();
    let csv_in = tmp.path().join("in.csv");
    std::fs::write(
        &csv_in,
        "ID,WHEN,TYPE\nA1,2026.07.04 13-05,BREAK-IN\nA2,2026.12.31 23-59,BREAK-IN\n",
    )
    .unwrap();
    let cfg = tmp.path().join("mapping.toml");
    std::fs::write(
        &cfg,
        r#"
source_city = "Testville"
datetime_format = "%Y.%m.%d %H-%M"
[columns]
incident_id = "ID"
datetime = "WHEN"
category = "TYPE"
[categories]
"BREAK-IN" = "burglary"
"#,
    )
    .unwrap();

    let out = tmp.path().join("out.csv");
    let status = Command::new(bin())
        .arg("normalize")
        .arg("--config")
        .arg(&cfg)
        .arg("--input")
        .arg(&csv_in)
        .arg("--output")
        .arg(&out)
        .status()
        .unwrap();
    assert!(status.success());

    let rows = read_csv(&out);
    assert_eq!(rows[0]["datetime"], "2026-07-04T13:05:00");
    assert_eq!(rows[1]["datetime"], "2026-12-31T23:59:00");
    assert_eq!(rows[0]["category"], "burglary");
}

#[test]
fn rows_with_bad_datetimes_are_skipped_not_fatal() {
    let tmp = tempfile::tempdir().unwrap();
    let csv_in = tmp.path().join("in.csv");
    std::fs::write(
        &csv_in,
        "ID,WHEN,TYPE\nA1,01/02/2026 10:00,THEFT\nA2,not-a-date,THEFT\n",
    )
    .unwrap();
    let cfg = tmp.path().join("mapping.toml");
    std::fs::write(
        &cfg,
        r#"
source_city = "Testville"
datetime_format = "%m/%d/%Y %H:%M"
[columns]
incident_id = "ID"
datetime = "WHEN"
category = "TYPE"
[categories]
"THEFT" = "theft"
"#,
    )
    .unwrap();

    let out = tmp.path().join("out.csv");
    let status = Command::new(bin())
        .arg("normalize")
        .arg("--config")
        .arg(&cfg)
        .arg("--input")
        .arg(&csv_in)
        .arg("--output")
        .arg(&out)
        .status()
        .unwrap();
    assert!(status.success(), "bad rows should not fail the run");

    let rows = read_csv(&out);
    assert_eq!(rows.len(), 1, "only the parseable row survives");
    assert_eq!(rows[0]["incident_id"], "A1");
}
