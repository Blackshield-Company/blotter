//! `blotter normalize` — apply a mapping config to a raw source CSV.

use anyhow::{anyhow, Context, Result};
use std::path::Path;

use crate::cli::NormalizeArgs;
use crate::config::{parse_datetime, MappingConfig};
use crate::schema::{NormalizedIncident, SCHEMA_FIELDS};

pub fn run(args: &NormalizeArgs) -> Result<()> {
    let cfg = MappingConfig::load(&args.config)?;
    let summary = normalize_file(&cfg, &args.input, &args.output)?;

    eprintln!(
        "blotter: normalized {} rows ({} skipped) -> {}",
        summary.written,
        summary.skipped.len(),
        args.output.display()
    );
    for s in &summary.skipped {
        eprintln!("  skipped row {}: {}", s.0, s.1);
    }
    Ok(())
}

pub struct NormalizeSummary {
    pub written: usize,
    pub skipped: Vec<(usize, String)>,
}

/// Pure-ish core used by both the CLI and tests.
pub fn normalize_file(
    cfg: &MappingConfig,
    input: &Path,
    output: &Path,
) -> Result<NormalizeSummary> {
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(false)
        .from_path(input)
        .with_context(|| format!("opening input CSV {}", input.display()))?;

    let headers = rdr.headers()?.clone();
    let col = |name: &Option<String>| -> Result<Option<usize>> {
        match name {
            None => Ok(None),
            Some(n) => headers
                .iter()
                .position(|h| h == n)
                .map(Some)
                .ok_or_else(|| anyhow!("input CSV has no column {:?}", n)),
        }
    };
    let col_req = |name: &str| -> Result<usize> {
        headers
            .iter()
            .position(|h| h == name)
            .ok_or_else(|| anyhow!("input CSV has no column {:?}", name))
    };

    let idx_id = col_req(&cfg.columns.incident_id)?;
    let idx_dt = col_req(&cfg.columns.datetime)?;
    let idx_cat = col_req(&cfg.columns.category)?;
    let idx_desc = col(&cfg.columns.description)?;
    let idx_lat = col(&cfg.columns.latitude)?;
    let idx_lon = col(&cfg.columns.longitude)?;
    let idx_dist = col(&cfg.columns.district_or_neighborhood)?;
    let idx_url = col(&cfg.columns.source_url)?;

    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
    }
    let mut wtr = csv::Writer::from_path(output)
        .with_context(|| format!("opening output CSV {}", output.display()))?;
    wtr.write_record(SCHEMA_FIELDS)?;

    let get = |rec: &csv::StringRecord, idx: Option<usize>| -> String {
        idx.and_then(|i| rec.get(i))
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    };

    let mut written = 0usize;
    let mut skipped: Vec<(usize, String)> = Vec::new();

    for (i, rec) in rdr.records().enumerate() {
        let row_num = i + 1; // 1-based, excluding header
        let rec = match rec {
            Ok(r) => r,
            Err(e) => {
                skipped.push((row_num, format!("CSV parse error: {}", e)));
                continue;
            }
        };

        match build_incident(
            cfg, &rec, idx_id, idx_dt, idx_cat, idx_desc, idx_lat, idx_lon, idx_dist, idx_url, &get,
        ) {
            Ok(incident) => {
                write_incident(&mut wtr, &incident)?;
                written += 1;
            }
            Err(e) => skipped.push((row_num, e.to_string())),
        }
    }

    wtr.flush()?;
    Ok(NormalizeSummary { written, skipped })
}

#[allow(clippy::too_many_arguments)]
fn build_incident(
    cfg: &MappingConfig,
    rec: &csv::StringRecord,
    idx_id: usize,
    idx_dt: usize,
    idx_cat: usize,
    idx_desc: Option<usize>,
    idx_lat: Option<usize>,
    idx_lon: Option<usize>,
    idx_dist: Option<usize>,
    idx_url: Option<usize>,
    get: &dyn Fn(&csv::StringRecord, Option<usize>) -> String,
) -> Result<NormalizedIncident> {
    let incident_id = get(rec, Some(idx_id));
    if incident_id.is_empty() {
        return Err(anyhow!("missing incident_id"));
    }

    let raw_dt = get(rec, Some(idx_dt));
    let datetime = parse_datetime(&raw_dt, &cfg.datetime_format)?;

    let raw_cat = get(rec, Some(idx_cat));
    let category = cfg.remap_category(&raw_cat);

    let opt_str = |v: String| if v.is_empty() { None } else { Some(v) };
    let opt_f64 = |v: String| -> Option<f64> { v.parse::<f64>().ok() };

    Ok(NormalizedIncident {
        incident_id,
        datetime,
        category,
        description: get(rec, idx_desc),
        latitude: opt_f64(get(rec, idx_lat)),
        longitude: opt_f64(get(rec, idx_lon)),
        district_or_neighborhood: opt_str(get(rec, idx_dist)),
        source_city: cfg.source_city.clone(),
        source_url: opt_str(get(rec, idx_url)),
    })
}

fn write_incident(wtr: &mut csv::Writer<std::fs::File>, i: &NormalizedIncident) -> Result<()> {
    wtr.write_record([
        i.incident_id.as_str(),
        i.datetime.as_str(),
        i.category.as_str(),
        i.description.as_str(),
        &i.latitude.map(|v| v.to_string()).unwrap_or_default(),
        &i.longitude.map(|v| v.to_string()).unwrap_or_default(),
        i.district_or_neighborhood.as_deref().unwrap_or(""),
        i.source_city.as_str(),
        i.source_url.as_deref().unwrap_or(""),
    ])?;
    Ok(())
}

/// Read a normalized CSV back into records (used by stats / to-json / tests).
pub fn read_normalized(path: &Path) -> Result<Vec<NormalizedIncident>> {
    let mut rdr = csv::Reader::from_path(path)
        .with_context(|| format!("opening normalized CSV {}", path.display()))?;
    let mut out = Vec::new();
    for rec in rdr.deserialize() {
        out.push(rec?);
    }
    Ok(out)
}
