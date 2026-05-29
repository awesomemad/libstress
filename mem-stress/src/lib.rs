//! RAM allocation, fragmentation, and bandwidth pressure workloads.

mod alloc_patterns;
mod bandwidth;
mod fragmentation;
mod random_access;

use anyhow::Result;
use stress_core::{
    Intensity, MemorySafetyCap, SystemSnapshot, run_timed,
};
use std::time::Duration;
use tracing::info;

/// Run memory stress until `duration` elapses.
pub fn run(
    intensity: Intensity,
    duration: Duration,
    memory_target: Option<u64>,
    cap_override: bool,
) -> Result<()> {
    let snapshot = SystemSnapshot::capture();
    let cap = MemorySafetyCap {
        max_fraction: stress_core::safety::DEFAULT_RAM_FRACTION,
        override_cap: cap_override,
    };

    let target = cap.resolve_target_bytes(memory_target, snapshot.total_memory)?;
    MemorySafetyCap::check_available_headroom(snapshot.available_memory, target)?;

    info!(
        target_bytes = target,
        intensity = intensity.0,
        ?duration,
        "starting memory stress"
    );

    let mut state = alloc_patterns::MemoryState::new(target, intensity);

    run_timed(duration, |stop| {
        if stop.should_stop() {
            return;
        }
        state.cycle(stop);
    });

    Ok(())
}
