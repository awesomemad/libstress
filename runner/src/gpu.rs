//! Optional GPU hooks — only active with `--features gpu`.

use anyhow::{bail, Result};
use std::time::Duration;

pub fn run_optional(intensity: u8, duration: Duration) -> Result<()> {
    #[cfg(feature = "gpu")]
    {
        return run_wgpu(intensity, duration);
    }

    #[cfg(not(feature = "gpu"))]
    {
        let _ = (intensity, duration);
        bail!(
            "GPU stress requires building with `--features gpu` \
             (cargo build -p libstress --features gpu)"
        );
    }
}

#[cfg(feature = "gpu")]
fn run_wgpu(intensity: u8, duration: Duration) -> Result<()> {
    use std::time::Instant;

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok_or_else(|| anyhow::anyhow!("no GPU adapter found"))?;

    let info = adapter.get_info();
    tracing::info!(name = %info.name, backend = ?info.backend, "GPU adapter available");

    let deadline = Instant::now() + duration;
    let mut acc: u64 = 0;
    let step = u64::from(intensity.clamp(1, 10)) * 50_000;

    while Instant::now() < deadline {
        for i in 0..step {
            acc = acc.wrapping_add(i.wrapping_mul(0x9E37_79B9));
        }
    }

    tracing::info!(acc, "GPU probe window finished");
    Ok(())
}
