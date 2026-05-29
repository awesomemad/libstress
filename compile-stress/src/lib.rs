//! Generate synthetic Cargo projects and drive `cargo` for compiler stress.

mod cargo_runner;
pub mod generator;
mod templates;

use anyhow::Result;
use stress_core::{CompileMode, Intensity};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::info;

pub use generator::{GenerateOptions, Stressgen};

/// Repeatedly generate (if needed) and compile synthetic workspaces.
pub fn run(
    intensity: Intensity,
    duration: Duration,
    modules: usize,
    mode: CompileMode,
    out_dir: impl AsRef<Path>,
    ecs: bool,
) -> Result<()> {
    let out_dir = out_dir.as_ref().to_path_buf();
    info!(
        intensity = intensity.0,
        ?duration,
        modules,
        ?mode,
        ecs,
        dir = %out_dir.display(),
        "starting compile stress"
    );

    let gen = Stressgen::new(GenerateOptions {
        modules: modules.max(1),
        intensity,
        ecs_stress: ecs,
        out_dir: out_dir.clone(),
    });
    gen.generate_workspace()?;

    let deadline = std::time::Instant::now() + duration;
    let mut round = 0u64;

    while std::time::Instant::now() < deadline {
        cargo_runner::run_cargo(&out_dir, mode, intensity)?;
        round += 1;

        // Incremental mode: touch subset of modules between builds.
        if matches!(mode, CompileMode::Incremental) && round % 2 == 0 {
            generator::touch_modules(&out_dir, intensity.scale_usize(10).max(1))?;
        }
    }

    Ok(())
}

pub fn default_generated_dir(base: &Path) -> PathBuf {
    base.join("compile-stress-gen")
}
