use stress_core::Intensity;

/// Sequential read/write/copy loop to saturate memory bandwidth.
pub fn saturate(blocks: &mut [Vec<u8>], target_bytes: u64, intensity: Intensity) {
    if blocks.is_empty() {
        return;
    }

    let passes = intensity.scale_usize(4).max(1);
    let mut scratch = vec![0u8; intensity.scale_usize(1 << 20).max(1 << 18)];

    for _ in 0..passes {
        for block in blocks.iter_mut() {
            let len = block.len().min(scratch.len());
            if len == 0 {
                continue;
            }
            scratch[..len].copy_from_slice(&block[..len]);
            for i in 0..len {
                block[i] = scratch[i].wrapping_add((i & 0xFF) as u8);
            }
        }
    }

    let _ = target_bytes; // reserved for future adaptive sizing telemetry
}
