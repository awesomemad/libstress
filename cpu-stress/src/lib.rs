//! Multi-algorithm CPU saturation workloads.

mod branch_chaos;
mod hash_loop;
mod matrix;
mod parallel;

use anyhow::Result;
use stress_core::{Intensity, run_timed};
use std::time::Duration;
use tracing::info;

pub use parallel::CpuWorkload;

/// Run all CPU stress algorithms for the given duration.
pub fn run(
    intensity: Intensity,
    duration: Duration,
    threads: usize,
) -> Result<()> {
    info!(
        intensity = intensity.0,
        ?duration,
        threads,
        "starting CPU stress"
    );

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()?;

    pool.install(|| {
        run_timed(duration, |stop| {
            if stop.should_stop() {
                return;
            }
            parallel::dispatch_round(intensity, stop);
        });
    });

    Ok(())
}
