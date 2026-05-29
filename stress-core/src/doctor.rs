use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::safety::MemorySafetyCap;
use crate::system::SystemSnapshot;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub ok: bool,
    pub checks: Vec<CheckResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

pub fn run_checks() -> DoctorReport {
    let mut checks = Vec::new();

    checks.push(check_cargo());
    checks.push(check_rustc());
    checks.push(check_memory());
    checks.push(check_temp_dir());
    checks.push(check_parallelism());

    let ok = checks.iter().all(|c| c.passed);
    DoctorReport { ok, checks }
}

pub fn print_report(report: &DoctorReport) {
    println!("libstress doctor");
    for c in &report.checks {
        let mark = if c.passed { "PASS" } else { "FAIL" };
        println!("  [{mark}] {} — {}", c.name, c.detail);
    }
    println!(
        "\n{}",
        if report.ok {
            "All checks passed. Ready to stress."
        } else {
            "Some checks failed. Fix issues before heavy runs."
        }
    );
}

fn check_cargo() -> CheckResult {
    match std::process::Command::new("cargo").arg("--version").output() {
        Ok(o) if o.status.success() => CheckResult {
            name: "cargo".into(),
            passed: true,
            detail: String::from_utf8_lossy(&o.stdout).trim().into(),
        },
        Ok(o) => CheckResult {
            name: "cargo".into(),
            passed: false,
            detail: format!("exit {}", o.status),
        },
        Err(e) => CheckResult {
            name: "cargo".into(),
            passed: false,
            detail: e.to_string(),
        },
    }
}

fn check_rustc() -> CheckResult {
    match std::process::Command::new("rustc").arg("--version").output() {
        Ok(o) if o.status.success() => CheckResult {
            name: "rustc".into(),
            passed: true,
            detail: String::from_utf8_lossy(&o.stdout).trim().into(),
        },
        Ok(o) => CheckResult {
            name: "rustc".into(),
            passed: false,
            detail: format!("exit {}", o.status),
        },
        Err(e) => CheckResult {
            name: "rustc".into(),
            passed: false,
            detail: e.to_string(),
        },
    }
}

fn check_memory() -> CheckResult {
    let snap = SystemSnapshot::capture();
    let cap = MemorySafetyCap::default();
    let max = ((snap.total_memory as f64) * cap.max_fraction) as u64;
    let ok = snap.available_memory > 512 * 1024 * 1024;
    CheckResult {
        name: "memory".into(),
        passed: ok,
        detail: format!(
            "available {} MiB, stress cap ~{} MiB (70%)",
            snap.available_memory / (1024 * 1024),
            max / (1024 * 1024)
        ),
    }
}

fn check_temp_dir() -> CheckResult {
    let dir = std::env::temp_dir();
    let writable = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dir.join("libstress-doctor-probe"))
        .is_ok();
    if writable {
        let _ = std::fs::remove_file(dir.join("libstress-doctor-probe"));
    }
    CheckResult {
        name: "temp_dir".into(),
        passed: writable,
        detail: format!("{}", dir.display()),
    }
}

fn check_parallelism() -> CheckResult {
    let n = std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1);
    CheckResult {
        name: "cpu_threads".into(),
        passed: n >= 1,
        detail: format!("{n} logical CPUs"),
    }
}

pub fn run() -> Result<()> {
    let report = run_checks();
    print_report(&report);
    if report.ok {
        Ok(())
    } else {
        anyhow::bail!("doctor checks failed")
    }
}
