use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// blotter — normalize and explore public crime/incident data.
///
/// Cities publish incident data as inconsistent CSVs. blotter applies a
/// per-city column-mapping config to produce one common schema, then lets
/// you explore it. Fully offline: downloading source data is your job.
#[derive(Parser)]
#[command(name = "blotter", version, about, long_about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Apply a column-mapping config to a raw source CSV, producing a normalized CSV.
    #[command(name = "normalize")]
    Normalize(NormalizeArgs),

    /// Print basic stats (counts by category, month, district) for a normalized CSV.
    #[command(name = "stats")]
    Stats(StatsArgs),

    /// Convert a normalized CSV to JSON on stdout.
    #[command(name = "to-json")]
    ToJson(ToJsonArgs),
}

#[derive(Args)]
pub struct NormalizeArgs {
    /// TOML column-mapping config for the source city.
    #[arg(long)]
    pub config: PathBuf,

    /// Raw source CSV to normalize.
    #[arg(long)]
    pub input: PathBuf,

    /// Output path for the normalized CSV.
    #[arg(long)]
    pub output: PathBuf,
}

#[derive(Args)]
pub struct StatsArgs {
    /// A normalized CSV produced by `blotter normalize`.
    pub input: PathBuf,
}

#[derive(Args)]
pub struct ToJsonArgs {
    /// A normalized CSV produced by `blotter normalize`.
    pub input: PathBuf,
}
