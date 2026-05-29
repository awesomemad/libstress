//! Parallel file I/O, hashing, and artifact churn workloads.

mod artifacts;
mod hash_files;
mod rw_loop;

use anyhow::{Context, Result};
use stress_core::{Intensity, run_timed};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::info;

/// Run disk I/O stress in `work_dir` for `duration`.
pub fn run(
    intensity: Intensity,
    duration: Duration,
    threads: usize,
    work_dir: impl AsRef<Path>,
) -> Result<()> {
    let work_dir = work_dir.as_ref().to_path_buf();
    std::fs::create_dir_all(&work_dir)
        .with_context(|| format!("create io work dir {}", work_dir.display()))?;

    info!(
        intensity = intensity.0,
        ?duration,
        threads,
        dir = %work_dir.display(),
        "starting I/O stress"
    );

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()?;

    pool.install(|| {
        run_timed(duration, |stop| {
            if stop.should_stop() {
                return;
            }
            rw_loop::write_cycle(&work_dir, intensity, threads);
            hash_files::hash_tree(&work_dir, intensity);
            artifacts::explosion_cycle(&work_dir, intensity);
            rw_loop::delete_cycle(&work_dir);
        });
    });

    Ok(())
}

/// Remove the I/O working directory (best-effort).
pub fn cleanup(work_dir: &Path) -> Result<()> {
    if work_dir.exists() {
        std::fs::remove_dir_all(work_dir)
            .with_context(|| format!("cleanup {}", work_dir.display()))?;
    }
    Ok(())
}

pub fn default_artifact_path(base: &Path) -> PathBuf {
    base.join("artifacts")
}
