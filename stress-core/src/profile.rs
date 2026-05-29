use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::{CompileMode, Intensity, WorkloadKind};

/// Top-level config file (`libstress.toml`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileConfig {
    #[serde(default)]
    pub profile: HashMap<String, ProfileSettings>,
    #[serde(default)]
    pub defaults: Option<ProfileSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSettings {
    #[serde(default = "default_duration_secs")]
    pub duration_secs: u64,
    #[serde(default = "default_intensity")]
    pub intensity: u8,
    #[serde(default)]
    pub threads: usize,
    #[serde(default)]
    pub workloads: Vec<String>,
    #[serde(default)]
    pub memory: Option<String>,
    #[serde(default)]
    pub memory_cap_override: bool,
    #[serde(default = "default_io_dir")]
    pub io_dir: String,
    #[serde(default = "default_compile_modules")]
    pub compile_modules: usize,
    #[serde(default)]
    pub compile_mode: Option<String>,
    #[serde(default)]
    pub compile_dir: Option<String>,
    #[serde(default)]
    pub ecs: bool,
    #[serde(default)]
    pub max_disk_gb: Option<f64>,
    #[serde(default)]
    pub net_port: Option<u16>,
}

fn default_duration_secs() -> u64 {
    30
}
fn default_intensity() -> u8 {
    5
}
fn default_io_dir() -> String {
    "libstress-io".into()
}
fn default_compile_modules() -> usize {
    50
}

impl Default for ProfileSettings {
    fn default() -> Self {
        Self {
            duration_secs: default_duration_secs(),
            intensity: default_intensity(),
            threads: 0,
            workloads: vec![],
            memory: None,
            memory_cap_override: false,
            io_dir: default_io_dir(),
            compile_modules: default_compile_modules(),
            compile_mode: None,
            compile_dir: None,
            ecs: false,
            max_disk_gb: None,
            net_port: None,
        }
    }
}

impl ProfileSettings {
    pub fn duration(&self) -> Duration {
        Duration::from_secs(self.duration_secs)
    }

    pub fn intensity(&self) -> Intensity {
        Intensity::new(self.intensity)
    }

    pub fn parse_workloads(&self) -> Vec<WorkloadKind> {
        if self.workloads.is_empty() {
            return vec![
                WorkloadKind::Cpu,
                WorkloadKind::Memory,
                WorkloadKind::Io,
            ];
        }
        self.workloads
            .iter()
            .filter_map(|s| match s.to_lowercase().as_str() {
                "cpu" => Some(WorkloadKind::Cpu),
                "mem" | "memory" => Some(WorkloadKind::Memory),
                "io" | "disk" => Some(WorkloadKind::Io),
                "compile" | "compiler" => Some(WorkloadKind::Compile),
                "gpu" => Some(WorkloadKind::Gpu),
                "net" | "network" => Some(WorkloadKind::Net),
                "sched" | "threads" => Some(WorkloadKind::Sched),
                "all" => Some(WorkloadKind::All),
                _ => None,
            })
            .collect()
    }

    pub fn compile_mode(&self) -> CompileMode {
        match self
            .compile_mode
            .as_deref()
            .unwrap_or("build-release")
            .to_lowercase()
            .as_str()
        {
            "check" => CompileMode::Check,
            "incremental" => CompileMode::Incremental,
            _ => CompileMode::BuildRelease,
        }
    }
}

const EMBEDDED: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../libstress.toml"));

impl FileConfig {
    pub fn embedded() -> Result<Self> {
        toml::from_str(EMBEDDED).context("parse embedded libstress.toml")
    }

    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("read config {}", path.display()))?;
        toml::from_str(&text).with_context(|| format!("parse config {}", path.display()))
    }

    pub fn discover() -> Result<Option<(PathBuf, Self)>> {
        let mut candidates = vec![PathBuf::from("libstress.toml")];
        if let Some(p) = dirs_config() {
            candidates.push(p);
        }
        for path in candidates {
            if path.is_file() {
                return Ok(Some((path.clone(), Self::load(&path)?)));
            }
        }
        Ok(None)
    }

    pub fn resolve_profile(&self, name: &str) -> Result<ProfileSettings> {
        let mut base = self.defaults.clone().unwrap_or_default();
        let Some(overlay) = self.profile.get(name) else {
            anyhow::bail!("unknown profile '{name}'");
        };
        merge_profile(&mut base, overlay);
        Ok(base)
    }

    pub fn profile_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.profile.keys().cloned().collect();
        names.sort();
        names
    }
}

fn merge_profile(base: &mut ProfileSettings, overlay: &ProfileSettings) {
    base.duration_secs = overlay.duration_secs;
    base.intensity = overlay.intensity;
    if overlay.threads != 0 {
        base.threads = overlay.threads;
    }
    if !overlay.workloads.is_empty() {
        base.workloads = overlay.workloads.clone();
    }
    if overlay.memory.is_some() {
        base.memory = overlay.memory.clone();
    }
    if overlay.memory_cap_override {
        base.memory_cap_override = true;
    }
    if overlay.io_dir != default_io_dir() {
        base.io_dir = overlay.io_dir.clone();
    }
    base.compile_modules = overlay.compile_modules;
    if overlay.compile_mode.is_some() {
        base.compile_mode = overlay.compile_mode.clone();
    }
    if overlay.compile_dir.is_some() {
        base.compile_dir = overlay.compile_dir.clone();
    }
    if overlay.ecs {
        base.ecs = true;
    }
    if overlay.max_disk_gb.is_some() {
        base.max_disk_gb = overlay.max_disk_gb;
    }
    if overlay.net_port.is_some() {
        base.net_port = overlay.net_port;
    }
}

fn dirs_config() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(|h| PathBuf::from(h).join(".config/libstress.toml"))
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config/libstress/libstress.toml"))
    }
}

pub fn merge_configs(mut base: FileConfig, overlay: FileConfig) -> FileConfig {
    if let Some(d) = overlay.defaults {
        let mut b = base.defaults.take().unwrap_or_default();
        merge_profile(&mut b, &d);
        base.defaults = Some(b);
    }
    for (k, v) in overlay.profile {
        base.profile.insert(k, v);
    }
    base
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_profiles_parse() {
        let cfg = FileConfig::embedded().unwrap();
        assert!(cfg.profile.contains_key("ci-quick"));
    }

    #[test]
    fn resolve_ci_quick() {
        let cfg = FileConfig::embedded().unwrap();
        let p = cfg.resolve_profile("ci-quick").unwrap();
        assert_eq!(p.duration_secs, 15);
        assert_eq!(p.intensity, 4);
    }
}
