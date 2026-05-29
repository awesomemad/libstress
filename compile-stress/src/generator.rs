use anyhow::{Context, Result};
use stress_core::Intensity;
use std::fs;
use std::path::{Path, PathBuf};

use crate::templates;

#[derive(Debug, Clone)]
pub struct GenerateOptions {
    pub modules: usize,
    pub intensity: Intensity,
    pub ecs_stress: bool,
    pub out_dir: PathBuf,
}

pub struct Stressgen {
    opts: GenerateOptions,
}

impl Stressgen {
    pub fn new(opts: GenerateOptions) -> Self {
        Self { opts }
    }

    pub fn generate_workspace(&self) -> Result<()> {
        let root = &self.opts.out_dir;
        if root.exists() {
            fs::remove_dir_all(root)
                .with_context(|| format!("clean {}", root.display()))?;
        }
        fs::create_dir_all(root.join("src"))
            .with_context(|| format!("create {}", root.display()))?;

        fs::write(root.join("Cargo.toml"), templates::workspace_manifest(&self.opts))?;
        fs::write(
            root.join("src").join("lib.rs"),
            templates::root_lib(&self.opts),
        )?;

        let modules_dir = root.join("src").join("modules");
        fs::create_dir_all(&modules_dir)?;

        let mut mod_decl = String::from("//! Auto-generated module tree.\n\n");
        for i in 0..self.opts.modules {
            let name = format!("mod_{i:05}");
            mod_decl.push_str(&format!("pub mod {name};\n"));
            let path = modules_dir.join(format!("{name}.rs"));
            fs::write(&path, templates::generated_module(i, &self.opts))?;
        }
        fs::write(modules_dir.join("mod.rs"), mod_decl)?;

        if self.opts.ecs_stress {
            fs::write(
                root.join("src").join("ecs_stress.rs"),
                templates::ecs_stress_module(),
            )?;
        }

        Ok(())
    }
}

pub fn touch_modules(root: &Path, count: usize) -> Result<()> {
    let modules_dir = root.join("src").join("modules");
    for i in 0..count {
        let path = modules_dir.join(format!("mod_{i:05}.rs"));
        if path.exists() {
            let mut content = fs::read_to_string(&path)?;
            content.push_str("\n// touched\n");
            fs::write(path, content)?;
        }
    }
    Ok(())
}
