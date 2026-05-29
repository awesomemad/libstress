use stress_core::Intensity;

/// Simulate allocator fragmentation by splitting and merging blocks.
pub fn simulate(blocks: &mut Vec<Vec<u8>>, intensity: Intensity) {
    if blocks.is_empty() {
        return;
    }

    let splits = intensity.scale_usize(8).max(2);
    let mut new_blocks = Vec::with_capacity(blocks.len() + splits);

    for block in blocks.drain(..) {
        if block.len() > 4096 && splits > 0 {
            let mid = block.len() / 2;
            let (left, right) = block.split_at(mid);
            new_blocks.push(left.to_vec());
            new_blocks.push(right.to_vec());
        } else {
            new_blocks.push(block);
        }
    }

    // Merge adjacent small blocks occasionally.
    if new_blocks.len() > 4 {
        let a = new_blocks.remove(0);
        let b = new_blocks.remove(0);
        let mut merged = a;
        merged.extend_from_slice(&b);
        new_blocks.push(merged);
    }

    *blocks = new_blocks;
}
