//! Blotter desktop. Same local core as the CLI. Nothing leaves the machine.

use blotter::config::MappingConfig;
use blotter::normalize::{normalize_file, read_normalized, NormalizeSummary};
use blotter::schema::NormalizedIncident;
use blotter::stats::{compute_stats, Stats};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
struct NormalizeResult {
    written: usize,
    skipped: Vec<(usize, String)>,
    stats: Stats,
    output_path: String,
}

#[derive(Serialize)]
struct LoadResult {
    stats: Stats,
    records: Vec<NormalizedIncident>,
}

fn trimmed_path(label: &str, raw: &str) -> Result<PathBuf, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(format!("{label} is empty"));
    }
    Ok(PathBuf::from(trimmed))
}

#[tauri::command]
fn normalize(
    config_path: String,
    input_path: String,
    output_path: String,
) -> Result<NormalizeResult, String> {
    let config_path = trimmed_path("config path", &config_path)?;
    let input_path = trimmed_path("input path", &input_path)?;
    let output_path = trimmed_path("output path", &output_path)?;

    let cfg = MappingConfig::load(&config_path).map_err(|e| e.to_string())?;
    let summary: NormalizeSummary =
        normalize_file(&cfg, &input_path, &output_path).map_err(|e| e.to_string())?;
    let records = read_normalized(&output_path).map_err(|e| e.to_string())?;
    let stats = compute_stats(&records);

    Ok(NormalizeResult {
        written: summary.written,
        skipped: summary.skipped,
        stats,
        output_path: output_path.display().to_string(),
    })
}

#[tauri::command]
fn load(path: String) -> Result<LoadResult, String> {
    let path = trimmed_path("path", &path)?;
    let records = read_normalized(&path).map_err(|e| e.to_string())?;
    let stats = compute_stats(&records);
    Ok(LoadResult { stats, records })
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![normalize, load])
        .run(tauri::generate_context!())
        .expect("error while running blotter");
}
