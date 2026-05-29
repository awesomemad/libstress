//! Shared configuration, safety limits, and runtime helpers for libstress.

pub mod config;
pub mod doctor;
pub mod metrics;
pub mod profile;
pub mod safety;
pub mod system;
pub mod workload;

pub use config::{CompileMode, Intensity, StressConfig, WorkloadKind};
pub use doctor::{run_checks, DoctorReport};
pub use metrics::{
    workload_label, MetricsCollector, ReportFormat, StressReport, WorkloadResult,
};
pub use profile::{FileConfig, ProfileSettings};
pub use safety::MemorySafetyCap;
pub use system::SystemSnapshot;
pub use workload::{run_timed, StopSignal};
