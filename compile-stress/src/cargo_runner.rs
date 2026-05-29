use anyhow::{bail, Context, Result};
use stress_core::{CompileMode, Intensity};
use std::path::Path;
use std::process::Command;

pub fn run_cargo(project_dir: &Path, mode: CompileMode, intensity: Intensity) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(project_dir);

    match mode {
        CompileMode::BuildRelease => {
            cmd.args(["build", "--release"]);
        }
        CompileMode::Check => {
            cmd.arg("check");
        }
        CompileMode::Incremental => {
            cmd.arg("build");
        }
    }

    if intensity.0 >= 8 {
        cmd.env("RUSTFLAGS", "-C debuginfo=2");
    }

    let status = cmd
        .status()
        .context("failed to spawn cargo — is Rust installed?")?;

    if !status.success() {
        bail!("cargo exited with {}", status);
    }

    Ok(())
}
