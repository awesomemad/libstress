use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use stress_core::profile::{merge_configs, FileConfig, ProfileSettings};
use stress_core::{CompileMode, WorkloadKind};

use crate::cli::{
    Cli, CliCompileMode, Commands, CommonArgs, CompileArgsInner, MemArgsInner,
};

/// Resolved execution plan after merging config file, profile, and CLI flags.
#[derive(Debug, Clone)]
pub struct RunPlan {
    pub profile_name: Option<String>,
    pub intensity: u8,
    pub duration: Duration,
    pub threads: usize,
    pub workloads: Vec<WorkloadKind>,
    pub mem: MemArgsInner,
    pub io_dir: PathBuf,
    pub compile: CompileArgsInner,
    pub net_port: u16,
    pub dry_run: bool,
    pub live_stats: bool,
}

impl RunPlan {
    pub fn from_cli(cli: &Cli) -> Result<Self> {
        let mut file = FileConfig::embedded().context("load embedded profiles")?;
        if let Some(path) = &cli.global.config {
            let user = FileConfig::load(path)?;
            file = merge_configs(file, user);
        } else if let Some((_, user)) = FileConfig::discover()? {
            file = merge_configs(file, user);
        }

        let mut plan = if let Some(name) = &cli.global.profile {
            let p = file.resolve_profile(name)?;
            plan_from_profile(Some(name.clone()), p)
        } else {
            plan_from_profile(None, file.defaults.unwrap_or_default())
        };

        apply_command(&mut plan, &cli.command)?;
        plan.dry_run = cli.global.dry_run;
        plan.live_stats = cli.global.live_stats;
        Ok(plan)
    }

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

pub fn plan_from_profile(name: Option<String>, p: ProfileSettings) -> RunPlan {
    RunPlan {
        profile_name: name,
        intensity: p.intensity,
        duration: p.duration(),
        threads: p.threads,
        workloads: p.parse_workloads(),
        mem: MemArgsInner {
            memory: p.memory.clone(),
            memory_cap_override: p.memory_cap_override,
        },
        io_dir: PathBuf::from(p.io_dir.clone()),
        compile: CompileArgsInner {
            modules: p.compile_modules,
            mode: match p.compile_mode() {
                CompileMode::Check => CliCompileMode::Check,
                CompileMode::Incremental => CliCompileMode::Incremental,
                CompileMode::BuildRelease => CliCompileMode::BuildRelease,
            },
            ecs: p.ecs,
            compile_dir: PathBuf::from(p.compile_dir.unwrap_or_else(|| "libstress-compile".into())),
        },
        net_port: p.net_port.unwrap_or(19456),
        dry_run: false,
        live_stats: false,
    }
}

fn apply_command(plan: &mut RunPlan, cmd: &Commands) -> Result<()> {
    match cmd {
        Commands::Run(args) => {
            apply_common(plan, &args.common);
            if args.any_selected() {
                plan.workloads.clear();
                if args.cpu {
                    plan.workloads.push(WorkloadKind::Cpu);
                }
                if args.mem {
                    plan.workloads.push(WorkloadKind::Memory);
                }
                if args.io {
                    plan.workloads.push(WorkloadKind::Io);
                }
                if args.compile {
                    plan.workloads.push(WorkloadKind::Compile);
                }
                if args.gpu {
                    plan.workloads.push(WorkloadKind::Gpu);
                }
                if args.net {
                    plan.workloads.push(WorkloadKind::Net);
                }
                if args.sched {
                    plan.workloads.push(WorkloadKind::Sched);
                }
            }
            merge_mem(plan, &args.mem_opts);
            if args.io_opts.io_dir != PathBuf::from("libstress-io") {
                plan.io_dir = args.io_opts.io_dir.clone();
            }
            merge_compile(plan, &args.compile_opts);
        }
        Commands::Cpu(args) => {
            apply_common(plan, args);
            plan.workloads = vec![WorkloadKind::Cpu];
        }
        Commands::Mem(args) => {
            apply_common(plan, &args.common);
            plan.workloads = vec![WorkloadKind::Memory];
            merge_mem(plan, &args.inner);
        }
        Commands::Io(args) => {
            apply_common(plan, &args.common);
            plan.workloads = vec![WorkloadKind::Io];
            plan.io_dir = args.inner.io_dir.clone();
        }
        Commands::Compile(args) => {
            apply_common(plan, &args.common);
            plan.workloads = vec![WorkloadKind::Compile];
            merge_compile(plan, &args.inner);
        }
        Commands::Gpu(args) => {
            apply_common(plan, args);
            plan.workloads = vec![WorkloadKind::Gpu];
        }
        Commands::Net(args) => {
            apply_common(plan, &args.common);
            plan.workloads = vec![WorkloadKind::Net];
            if let Some(p) = args.port {
                plan.net_port = p;
            }
        }
        Commands::Sched(args) => {
            apply_common(plan, args);
            plan.workloads = vec![WorkloadKind::Sched];
        }
        Commands::Extras { command } => match command {
            crate::cli::ExtrasCommands::Net(args) => {
                apply_common(plan, &args.common);
                // port handled below
                plan.workloads = vec![WorkloadKind::Net];
                if let Some(p) = args.port {
                    plan.net_port = p;
                }
            }
            crate::cli::ExtrasCommands::Sched(args) => {
                apply_common(plan, args);
                plan.workloads = vec![WorkloadKind::Sched];
            }
        },
        Commands::Doctor { .. }
        | Commands::Profiles
        | Commands::Info
        | Commands::Bench { .. } => {}
    }
    Ok(())
}

fn apply_common(plan: &mut RunPlan, common: &CommonArgs) {
    if common.intensity != 5 || plan.profile_name.is_none() {
        // CLI always wins when explicitly set; clap defaults to 5
        plan.intensity = common.intensity;
    }
    if common.duration != Duration::from_secs(30) || plan.profile_name.is_none() {
        plan.duration = common.duration;
    }
    if common.threads != 0 {
        plan.threads = common.threads;
    }
}

fn merge_mem(plan: &mut RunPlan, inner: &MemArgsInner) {
    if inner.memory.is_some() {
        plan.mem.memory = inner.memory.clone();
    }
    if inner.memory_cap_override {
        plan.mem.memory_cap_override = true;
    }
}

fn merge_compile(plan: &mut RunPlan, inner: &CompileArgsInner) {
    if inner.modules != 50 {
        plan.compile.modules = inner.modules;
    }
    plan.compile.mode = inner.mode;
    if inner.ecs {
        plan.compile.ecs = true;
    }
    if inner.compile_dir != PathBuf::from("libstress-compile") {
        plan.compile.compile_dir = inner.compile_dir.clone();
    }
}
