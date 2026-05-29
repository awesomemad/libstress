//! CLI tool to generate synthetic Rust workspaces for compiler stress.

use anyhow::Result;
use clap::Parser;
use compile_stress::generator::{GenerateOptions, Stressgen};
use stress_core::Intensity;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "stressgen", about = "Generate synthetic Rust modules for compile stress")]
struct Args {
    /// Output directory for the generated Cargo workspace
    #[arg(long, default_value = "target/stressgen-out")]
    out_dir: PathBuf,

    /// Number of modules to generate
    #[arg(long, default_value_t = 100)]
    modules: usize,

    /// Intensity 1-10 (controls generic depth and payload size)
    #[arg(long, default_value_t = 5)]
    intensity: u8,

    /// Enable Bevy ECS-style optional stress module
    #[arg(long)]
    ecs: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let out = args.out_dir.clone();
    let gen = Stressgen::new(GenerateOptions {
        modules: args.modules,
        intensity: Intensity::new(args.intensity),
        ecs_stress: args.ecs,
        out_dir: out.clone(),
    });
    gen.generate_workspace()?;
    println!("Generated workspace at {}", out.display());
    Ok(())
}
