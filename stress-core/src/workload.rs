use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tracing::info;

/// Cooperative stop flag checked by long-running loops.
#[derive(Clone, Default)]
pub struct StopSignal(Arc<AtomicBool>);

impl StopSignal {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stop(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn should_stop(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

/// Run `work` until `duration` elapses or the stop signal fires.
pub fn run_timed<F>(duration: Duration, mut work: F)
where
    F: FnMut(&StopSignal),
{
    let stop = StopSignal::new();
    let stop_clone = stop.clone();
    let deadline = Instant::now() + duration;

    std::thread::scope(|scope| {
        scope.spawn(move || {
            while Instant::now() < deadline && !stop_clone.should_stop() {
                std::thread::sleep(Duration::from_millis(50));
            }
            stop_clone.stop();
        });

        while Instant::now() < deadline && !stop.should_stop() {
            work(&stop);
        }
    });

    info!(?duration, "workload window completed");
}
