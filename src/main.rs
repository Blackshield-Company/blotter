use anyhow::Result;
use clap::Parser;

mod cli;
mod config;
mod normalize;
mod schema;
mod stats;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Normalize(args) => normalize::run(&args),
        Commands::Stats(args) => stats::run(&args),
        Commands::ToJson(args) => stats::run_to_json(&args),
    }
}
