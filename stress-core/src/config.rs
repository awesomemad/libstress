use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Normalized load level (1 = light, 10 = extreme).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Intensity(pub u8);

impl Intensity {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 10;

    pub fn new(raw: u8) -> Self {
        Self(raw.clamp(Self::MIN, Self::MAX))
    }

    /// Scale a base value by intensity (1.0 at level 5, up to ~2.0 at 10).
    pub fn scale_f64(&self, base: f64) -> f64 {
        let factor = 0.4 + (f64::from(self.0) / 10.0) * 1.6;
        base * factor
    }

    pub fn scale_usize(&self, base: usize) -> usize {
        ((self.scale_f64(base as f64)).round() as usize).max(1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkloadKind {
    Cpu,
    Memory,
    Io,
    Compile,
    Gpu,
    Net,
    Sched,
    All,
}

/// Common runner configuration shared across stress modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressConfig {
    pub intensity: Intensity,
    pub duration: Duration,
    /// `0` means auto-detect logical CPUs.
    pub threads: usize,
    pub workloads: Vec<WorkloadKind>,
    pub io_dir: PathBuf,
    pub memory_target: Option<u64>,
    pub memory_cap_override: bool,
    pub compile_modules: usize,
    pub compile_mode: CompileMode,
    pub gpu_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CompileMode {
    #[default]
    BuildRelease,
    Check,
    Incremental,
}

impl Default for StressConfig {
    fn default() -> Self {
        Self {
            intensity: Intensity(5),
            duration: Duration::from_secs(30),
            threads: 0,
            workloads: vec![WorkloadKind::All],
            io_dir: std::env::temp_dir().join("libstress-io"),
            memory_target: None,
            memory_cap_override: false,
            compile_modules: 50,
            compile_mode: CompileMode::default(),
            gpu_enabled: false,
        }
    }
}

impl StressConfig {
    pub fn effective_threads(&self) -> usize {
        if self.threads > 0 {
            self.threads
        } else {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4)
        }
    }
}
