mod cli;
mod context;
mod gpu;
mod orchestrator;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use crate::cli::Cli;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("libstress=info".parse()?))
        .init();

    let cli = Cli::parse();
    orchestrator::run(cli)
}
