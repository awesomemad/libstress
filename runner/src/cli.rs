use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use stress_core::CompileMode;

#[derive(Parser, Debug)]
#[command(
    name = "libstress",
    version,
    about = "Professional system stress testing: CPU, RAM, I/O, compiler, and optional extras",
    after_help = "Examples:\n  libstress run --profile ci-quick\n  libstress run --cpu --mem --duration 60s --report json\n  libstress doctor\n  libstress profiles"
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalOpts,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Debug, Clone, Default)]
pub struct GlobalOpts {
    /// Path to libstress.toml (default: ./libstress.toml or embedded profiles)
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// Named profile from config
    #[arg(long, global = true)]
    pub profile: Option<String>,

    /// Emit report after run
    #[arg(long, global = true, value_enum)]
    pub report: Option<ReportOpt>,

    /// Print planned workloads without executing
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Sample CPU/RAM every second during runs
    #[arg(long, global = true)]
    pub live_stats: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ReportOpt {
    Json,
    Text,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run workloads (see --profile or module flags)
    Run(RunArgs),
    /// CPU-only stress
    Cpu(CommonArgs),
    /// Memory-only stress
    Mem(MemArgs),
    /// Disk I/O stress
    Io(IoArgs),
    /// Compiler / Cargo stress
    Compile(CompileArgs),
    /// Loopback network stress (requires `extras` feature)
    Net(NetArgs),
    /// Thread scheduler stress (requires `extras` feature)
    Sched(CommonArgs),
    /// Optional GPU probe (`gpu` feature)
    Gpu(CommonArgs),
    /// Extra workloads (alias for net/sched)
    Extras {
        #[command(subcommand)]
        command: ExtrasCommands,
    },
    /// Preflight checks (cargo, RAM, temp dir)
    Doctor {
        #[arg(long, value_enum)]
        report: Option<ReportOpt>,
    },
    /// List available profiles
    Profiles,
    /// Repeat a profile N times and summarize
    Bench {
        #[arg(long, default_value = "ci-quick")]
        profile: String,
        #[arg(long, default_value_t = 3)]
        rounds: u32,
    },
    /// Print host snapshot
    Info,
}

#[derive(Subcommand, Debug)]
pub enum ExtrasCommands {
    Net(NetArgs),
    Sched(CommonArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct CommonArgs {
    #[arg(short, long, default_value_t = 5, value_parser = parse_intensity)]
    pub intensity: u8,

    #[arg(short, long, default_value = "30s", value_parser = humantime::parse_duration)]
    pub duration: std::time::Duration,

    #[arg(short = 'j', long, default_value_t = 0)]
    pub threads: usize,
}

#[derive(Parser, Debug, Clone)]
pub struct NetArgs {
    #[command(flatten)]
    pub common: CommonArgs,

    #[arg(long)]
    pub port: Option<u16>,
}

#[derive(Parser, Debug, Clone)]
pub struct RunArgs {
    #[arg(long)]
    pub cpu: bool,
    #[arg(long)]
    pub mem: bool,
    #[arg(long)]
    pub io: bool,
    #[arg(long)]
    pub compile: bool,
    #[arg(long)]
    pub gpu: bool,
    #[arg(long)]
    pub net: bool,
    #[arg(long)]
    pub sched: bool,

    #[command(flatten)]
    pub common: CommonArgs,

    #[command(flatten)]
    pub mem_opts: MemArgsInner,

    #[command(flatten)]
    pub io_opts: IoArgsInner,

    #[command(flatten)]
    pub compile_opts: CompileArgsInner,
}

#[derive(Parser, Debug, Clone)]
pub struct MemArgs {
    #[command(flatten)]
    pub common: CommonArgs,

    #[command(flatten)]
    pub inner: MemArgsInner,
}

#[derive(Parser, Debug, Clone, Default)]
pub struct MemArgsInner {
    #[arg(long)]
    pub memory: Option<String>,

    #[arg(long)]
    pub memory_cap_override: bool,
}

#[derive(Parser, Debug, Clone)]
pub struct IoArgs {
    #[command(flatten)]
    pub common: CommonArgs,

    #[command(flatten)]
    pub inner: IoArgsInner,
}

#[derive(Parser, Debug, Clone)]
pub struct IoArgsInner {
    #[arg(long, default_value = "libstress-io")]
    pub io_dir: PathBuf,
}

#[derive(Parser, Debug, Clone)]
pub struct CompileArgs {
    #[command(flatten)]
    pub common: CommonArgs,

    #[command(flatten)]
    pub inner: CompileArgsInner,
}

#[derive(Parser, Debug, Clone)]
pub struct CompileArgsInner {
    #[arg(long, default_value_t = 50)]
    pub modules: usize,

    #[arg(long, value_enum, default_value_t = CliCompileMode::BuildRelease)]
    pub mode: CliCompileMode,

    #[arg(long)]
    pub ecs: bool,

    #[arg(long, default_value = "libstress-compile")]
    pub compile_dir: PathBuf,
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum CliCompileMode {
    #[default]
    BuildRelease,
    Check,
    Incremental,
}

impl Default for IoArgsInner {
    fn default() -> Self {
        Self {
            io_dir: PathBuf::from("libstress-io"),
        }
    }
}

impl Default for CompileArgsInner {
    fn default() -> Self {
        Self {
            modules: 50,
            mode: CliCompileMode::BuildRelease,
            ecs: false,
            compile_dir: PathBuf::from("libstress-compile"),
        }
    }
}

impl From<CliCompileMode> for CompileMode {
    fn from(m: CliCompileMode) -> Self {
        match m {
            CliCompileMode::BuildRelease => CompileMode::BuildRelease,
            CliCompileMode::Check => CompileMode::Check,
            CliCompileMode::Incremental => CompileMode::Incremental,
        }
    }
}

fn parse_intensity(s: &str) -> Result<u8, String> {
    let v: u8 = s.parse().map_err(|e: std::num::ParseIntError| e.to_string())?;
    if (1..=10).contains(&v) {
        Ok(v)
    } else {
        Err("intensity must be between 1 and 10".into())
    }
}

impl RunArgs {
    pub fn any_selected(&self) -> bool {
        self.cpu || self.mem || self.io || self.compile || self.gpu || self.net || self.sched
    }
}
