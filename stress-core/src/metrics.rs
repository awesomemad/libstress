use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sysinfo::System;

use crate::config::WorkloadKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportFormat {
    Json,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sample {
    pub elapsed_secs: f64,
    pub cpu_usage_pct: f32,
    pub used_memory_mib: u64,
    pub available_memory_mib: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadResult {
    pub workload: String,
    pub duration_secs: f64,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressReport {
    pub hostname: String,
    pub started_at: String,
    pub finished_at: String,
    pub profile: Option<String>,
    pub intensity: u8,
    pub workloads: Vec<WorkloadResult>,
    pub peak_cpu_pct: f32,
    pub peak_used_memory_mib: u64,
    pub samples: Vec<Sample>,
}

pub struct MetricsCollector {
    started: Instant,
    hostname: String,
    samples: Arc<Mutex<Vec<Sample>>>,
    stop: Arc<Mutex<bool>>,
}

impl MetricsCollector {
    pub fn start(interval: Duration) -> Self {
        let collector = Self {
            started: Instant::now(),
            hostname: hostname(),
            samples: Arc::new(Mutex::new(Vec::new())),
            stop: Arc::new(Mutex::new(false)),
        };
        let samples = collector.samples.clone();
        let stop = collector.stop.clone();
        let started = collector.started;
        std::thread::spawn(move || {
            let mut sys = System::new();
            sys.refresh_cpu_usage();
            std::thread::sleep(Duration::from_millis(200));
            while !*stop.lock().unwrap() {
                sys.refresh_cpu_usage();
                sys.refresh_memory();
                let elapsed = started.elapsed().as_secs_f64();
                let cpu: f32 = sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>()
                    / sys.cpus().len().max(1) as f32;
                let used = sys.used_memory() / (1024 * 1024);
                let avail = sys.available_memory() / (1024 * 1024);
                samples.lock().unwrap().push(Sample {
                    elapsed_secs: elapsed,
                    cpu_usage_pct: cpu,
                    used_memory_mib: used,
                    available_memory_mib: avail,
                });
                std::thread::sleep(interval);
            }
        });
        collector
    }

    pub fn stop(&self) {
        *self.stop.lock().unwrap() = true;
    }

    pub fn build_report(
        &self,
        profile: Option<String>,
        intensity: u8,
        workloads: Vec<WorkloadResult>,
    ) -> StressReport {
        let samples = self.samples.lock().unwrap().clone();
        let peak_cpu = samples
            .iter()
            .map(|s| s.cpu_usage_pct)
            .fold(0.0f32, f32::max);
        let peak_mem = samples
            .iter()
            .map(|s| s.used_memory_mib)
            .max()
            .unwrap_or(0);
        let now = chrono_now();
        StressReport {
            hostname: self.hostname.clone(),
            started_at: now.clone(),
            finished_at: now,
            profile,
            intensity,
            workloads,
            peak_cpu_pct: peak_cpu,
            peak_used_memory_mib: peak_mem,
            samples,
        }
    }
}

impl Drop for MetricsCollector {
    fn drop(&mut self) {
        self.stop();
    }
}

pub fn workload_label(kind: WorkloadKind) -> &'static str {
    match kind {
        WorkloadKind::Cpu => "cpu",
        WorkloadKind::Memory => "memory",
        WorkloadKind::Io => "io",
        WorkloadKind::Compile => "compile",
        WorkloadKind::Gpu => "gpu",
        WorkloadKind::Net => "net",
        WorkloadKind::Sched => "sched",
        WorkloadKind::All => "all",
    }
}

fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "localhost".into())
}

fn chrono_now() -> String {
    // Avoid chrono dependency: RFC3339-ish from system time
    format!("{:?}", std::time::SystemTime::now())
}

pub fn print_report(report: &StressReport, format: ReportFormat) {
    match format {
        ReportFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".into())
            );
        }
        ReportFormat::Text => {
            println!("libstress report");
            println!("  host:     {}", report.hostname);
            if let Some(p) = &report.profile {
                println!("  profile:  {p}");
            }
            println!("  intensity: {}", report.intensity);
            println!("  peak CPU: {:.1}%", report.peak_cpu_pct);
            println!("  peak RAM: {} MiB", report.peak_used_memory_mib);
            for w in &report.workloads {
                let status = if w.success { "ok" } else { "FAIL" };
                println!(
                    "  - {}: {:.1}s [{status}]",
                    w.workload, w.duration_secs
                );
                if let Some(e) = &w.error {
                    println!("      {e}");
                }
            }
        }
    }
}
