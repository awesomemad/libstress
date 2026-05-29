use stress_core::Intensity;

/// Random stride access to defeat prefetch and exercise TLB pressure.
pub fn touch(blocks: &mut [Vec<u8>], intensity: Intensity) {
    let passes = intensity.scale_usize(64).max(8);
    let mut seed: u64 = 0xDEAD_BEEF_CAFE_BABE;

    for _ in 0..passes {
        for block in blocks.iter_mut() {
            if block.is_empty() {
                continue;
            }
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let idx = (seed as usize) % block.len();
            block[idx] = block[idx].wrapping_add(1);
            seed = seed.wrapping_add(block[idx] as u64);
        }
    }
}
