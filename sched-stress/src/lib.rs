//! Thread spawn / join churn with a hard cap for safety.

use anyhow::{bail, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use stress_core::{Intensity, run_timed};
use tracing::info;

const MAX_THREADS: usize = 256;

/// Spawn short-lived worker threads until duration elapses.
pub fn run(intensity: Intensity, duration: Duration, threads: usize) -> Result<()> {
    let workers = threads.min(MAX_THREADS).max(1);
    if threads > MAX_THREADS {
        bail!("sched-stress capped at {MAX_THREADS} threads (requested {threads})");
    }

    info!(workers, "starting scheduler stress");
    let counter = AtomicU64::new(0);
    let iters = intensity.scale_usize(200).max(20);

    run_timed(duration, |stop| {
        if stop.should_stop() {
            return;
        }
        std::thread::scope(|scope| {
            for _ in 0..workers {
                scope.spawn(|| {
                    for _ in 0..iters {
                        counter.fetch_add(1, Ordering::Relaxed);
                        std::hint::spin_loop();
                    }
                });
            }
        });
    });

    let _ = counter.load(Ordering::Relaxed);
    Ok(())
}
