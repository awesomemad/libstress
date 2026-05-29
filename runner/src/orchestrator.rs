use anyhow::{bail, Result};
use std::time::{Duration, Instant};
use stress_core::{
    doctor, metrics::{self, MetricsCollector, ReportFormat},
    workload_label, FileConfig, Intensity, MemorySafetyCap, ReportFormat as CoreReport,
    StressReport, SystemSnapshot, WorkloadKind, WorkloadResult,
};

use crate::cli::{Cli, Commands, ReportOpt};
use crate::context::RunPlan;
use crate::gpu;

pub fn run(cli: Cli) -> Result<()> {
    match &cli.command {
        Commands::Doctor { report } => {
            let rep = doctor::run_checks();
            doctor::print_report(&rep);
            if let Some(fmt) = report {
                if matches!(fmt, ReportOpt::Json) {
                    println!("{}", serde_json::to_string_pretty(&rep)?);
                }
            }
            if rep.ok {
                Ok(())
            } else {
                bail!("doctor checks failed")
            }
        }
        Commands::Profiles => {
            let cfg = FileConfig::embedded()?;
            println!("Built-in profiles:");
            for name in cfg.profile_names() {
                println!("  - {name}");
            }
            if let Some((path, _)) = FileConfig::discover()? {
                println!("\nAlso loaded: {}", path.display());
            }
            Ok(())
        }
        Commands::Info => {
            let snap = SystemSnapshot::capture();
            println!("CPUs: {}", snap.cpu_count);
            println!(
                "RAM total: {} MiB, available: {} MiB",
                snap.total_memory / (1024 * 1024),
                snap.available_memory / (1024 * 1024)
            );
            println!(
                "Default memory cap (70%): {} MiB",
                ((snap.total_memory as f64) * MemorySafetyCap::default().max_fraction) as u64
                    / (1024 * 1024)
            );
            Ok(())
        }
        Commands::Bench { profile, rounds } => run_bench(profile, *rounds),
        _ => {
            let plan = RunPlan::from_cli(&cli)?;
            if plan.dry_run {
                print_dry_run(&plan);
                return Ok(());
            }
            execute_plan(&cli, &plan)
        }
    }
}

fn run_bench(profile: &str, rounds: u32) -> Result<()> {
    let file = FileConfig::embedded()?;
    let settings = file.resolve_profile(profile)?;
    let plan = crate::context::plan_from_profile(Some(profile.to_string()), settings);
    let cli = Cli {
        global: crate::cli::GlobalOpts {
            profile: Some(profile.to_string()),
            ..Default::default()
        },
        command: Commands::Run(crate::cli::RunArgs {
            cpu: false,
            mem: false,
            io: false,
            compile: false,
            gpu: false,
            net: false,
            sched: false,
            common: crate::cli::CommonArgs {
                intensity: plan.intensity,
                duration: plan.duration,
                threads: plan.threads,
            },
            mem_opts: plan.mem.clone(),
            io_opts: crate::cli::IoArgsInner {
                io_dir: plan.io_dir.clone(),
            },
            compile_opts: plan.compile.clone(),
        }),
    };
    let mut ok = 0u32;
    for i in 1..=rounds {
        println!("bench round {i}/{rounds} (profile={profile})");
        if execute_plan(&cli, &plan).is_ok() {
            ok += 1;
        }
    }
    println!("bench complete: {ok}/{rounds} succeeded");
    Ok(())
}

fn print_dry_run(plan: &RunPlan) {
    println!("dry-run plan:");
    if let Some(p) = &plan.profile_name {
        println!("  profile: {p}");
    }
    println!("  duration: {:?}", plan.duration);
    println!("  intensity: {}", plan.intensity);
    println!("  threads: {}", plan.effective_threads());
    print!("  workloads:");
    for w in &plan.workloads {
        print!(" {}", workload_label(*w));
    }
    println!();
}

fn execute_plan(cli: &Cli, plan: &RunPlan) -> Result<()> {
    let intensity = Intensity::new(plan.intensity);
    let threads = plan.effective_threads();
    let collector = if plan.live_stats || cli.global.report.is_some() {
        Some(MetricsCollector::start(Duration::from_secs(1)))
    } else {
        None
    };

    let mut results = Vec::new();
    let mut workloads = plan.workloads.clone();
    if workloads.iter().any(|w| *w == WorkloadKind::All) {
        workloads = vec![
            WorkloadKind::Cpu,
            WorkloadKind::Memory,
            WorkloadKind::Io,
        ];
    }

    for kind in workloads {
        let label = workload_label(kind).to_string();
        let started = Instant::now();
        let run_result = run_one(kind, plan, intensity, threads);
        let elapsed = started.elapsed().as_secs_f64();
        let success = run_result.is_ok();
        let error = run_result.as_ref().err().map(|e| e.to_string());
        results.push(WorkloadResult {
            workload: label,
            duration_secs: elapsed,
            success,
            error,
        });
        run_result?;
    }

    if let Some(c) = collector {
        let report = c.build_report(plan.profile_name.clone(), plan.intensity, results);
        if let Some(fmt) = cli.global.report {
            metrics::print_report(&report, map_report(fmt));
        } else if plan.live_stats {
            metrics::print_report(&report, CoreReport::Text);
        }
    } else if let Some(fmt) = cli.global.report {
        let report = minimal_report(plan.profile_name.clone(), plan.intensity, results);
        metrics::print_report(&report, map_report(fmt));
    }

    Ok(())
}

fn minimal_report(
    profile: Option<String>,
    intensity: u8,
    workloads: Vec<WorkloadResult>,
) -> StressReport {
    StressReport {
        hostname: std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "localhost".into()),
        started_at: format!("{:?}", std::time::SystemTime::now()),
        finished_at: format!("{:?}", std::time::SystemTime::now()),
        profile,
        intensity,
        workloads,
        peak_cpu_pct: 0.0,
        peak_used_memory_mib: 0,
        samples: vec![],
    }
}

fn map_report(r: ReportOpt) -> ReportFormat {
    match r {
        ReportOpt::Json => ReportFormat::Json,
        ReportOpt::Text => ReportFormat::Text,
    }
}

fn run_one(
    kind: WorkloadKind,
    plan: &RunPlan,
    intensity: Intensity,
    threads: usize,
) -> Result<()> {
    match kind {
        WorkloadKind::Cpu => cpu_stress::run(intensity, plan.duration, threads),
        WorkloadKind::Memory => {
            let target = match &plan.mem.memory {
                Some(s) => Some(MemorySafetyCap::parse_memory_size(s)?),
                None => None,
            };
            mem_stress::run(
                intensity,
                plan.duration,
                target,
                plan.mem.memory_cap_override,
            )
        }
        WorkloadKind::Io => {
            let dir = std::env::temp_dir().join(&plan.io_dir);
            let r = io_stress::run(intensity, plan.duration, threads, &dir);
            let _ = io_stress::cleanup(&dir);
            r
        }
        WorkloadKind::Compile => {
            let out = std::env::temp_dir().join(&plan.compile.compile_dir);
            compile_stress::run(
                intensity,
                plan.duration,
                plan.compile.modules,
                plan.compile.mode.into(),
                out,
                plan.compile.ecs,
            )
        }
        WorkloadKind::Gpu => gpu::run_optional(plan.intensity, plan.duration),
        WorkloadKind::Net => net_stress::run(intensity, plan.duration, plan.net_port, threads),
        WorkloadKind::Sched => sched_stress::run(intensity, plan.duration, threads),
        WorkloadKind::All => unreachable!(),
    }
}
