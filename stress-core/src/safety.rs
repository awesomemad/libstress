use anyhow::{bail, Context, Result};

/// Default maximum fraction of system RAM libstress may target.
pub const DEFAULT_RAM_FRACTION: f64 = 0.70;

/// Enforces safe memory ceilings before allocation-heavy workloads run.
#[derive(Debug, Clone, Copy)]
pub struct MemorySafetyCap {
    pub max_fraction: f64,
    pub override_cap: bool,
}

impl Default for MemorySafetyCap {
    fn default() -> Self {
        Self {
            max_fraction: DEFAULT_RAM_FRACTION,
            override_cap: false,
        }
    }
}

impl MemorySafetyCap {
    pub fn resolve_target_bytes(
        &self,
        requested: Option<u64>,
        total_system_ram: u64,
    ) -> Result<u64> {
        let cap = if self.override_cap {
            total_system_ram
        } else {
            ((total_system_ram as f64) * self.max_fraction) as u64
        };

        let target = match requested {
            Some(bytes) if bytes > cap && !self.override_cap => {
                bail!(
                    "requested {} bytes exceeds safety cap of {} bytes ({:.0}% of {} total). \
                     Pass --memory-cap-override to acknowledge risk.",
                    bytes,
                    cap,
                    self.max_fraction * 100.0,
                    total_system_ram
                );
            }
            Some(bytes) => bytes,
            None => (cap / 2).max(64 * 1024 * 1024),
        };

        if target == 0 {
            bail!("memory target resolved to zero bytes");
        }

        Ok(target)
    }

    pub fn check_available_headroom(available_ram: u64, target: u64) -> Result<()> {
        // Keep at least 512 MiB free for the OS unless user overrode the cap.
        const MIN_HEADROOM: u64 = 512 * 1024 * 1024;
        if available_ram.saturating_sub(target) < MIN_HEADROOM {
            anyhow::bail!(
                "only {} bytes available; need headroom for OS stability",
                available_ram
            );
        }
        Ok(())
    }

    pub fn parse_memory_size(s: &str) -> Result<u64> {
        let s = s.trim().to_uppercase();
        let (num_str, unit) = if s.ends_with("GB") {
            (&s[..s.len() - 2], 1024_u64.pow(3))
        } else if s.ends_with("MB") {
            (&s[..s.len() - 2], 1024_u64.pow(2))
        } else if s.ends_with("KB") {
            (&s[..s.len() - 2], 1024)
        } else if s.ends_with('B') {
            (&s[..s.len() - 1], 1)
        } else {
            (s.as_str(), 1)
        };

        let value: f64 = num_str
            .trim()
            .parse()
            .with_context(|| format!("invalid memory size: {s}"))?;
        Ok((value * unit as f64) as u64)
    }
}
