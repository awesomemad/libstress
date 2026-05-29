use stress_core::{Intensity, StopSignal};

use crate::bandwidth;
use crate::fragmentation;
use crate::random_access;

pub struct MemoryState {
    target_bytes: u64,
    intensity: Intensity,
    phase: usize,
    large_blocks: Vec<Vec<u8>>,
}

impl MemoryState {
    pub fn new(target_bytes: u64, intensity: Intensity) -> Self {
        Self {
            target_bytes,
            intensity,
            phase: 0,
            large_blocks: Vec::new(),
        }
    }

    pub fn cycle(&mut self, stop: &StopSignal) {
        if stop.should_stop() {
            return;
        }

        match self.phase % 4 {
            0 => self.allocate_large_vectors(),
            1 => fragmentation::simulate(&mut self.large_blocks, self.intensity),
            2 => random_access::touch(&mut self.large_blocks, self.intensity),
            _ => bandwidth::saturate(&mut self.large_blocks, self.target_bytes, self.intensity),
        }

        self.phase += 1;
    }

    fn allocate_large_vectors(&mut self) {
        let chunk = self.intensity.scale_usize(4 * 1024 * 1024).max(1 << 20);
        let mut held: u64 = self.large_blocks.iter().map(|v| v.len() as u64).sum();

        while held < self.target_bytes {
            let mut block = vec![0u8; chunk];
            for (i, byte) in block.iter_mut().enumerate() {
                *byte = (i & 0xFF) as u8;
            }
            held += block.len() as u64;
            self.large_blocks.push(block);
        }

        // Release half the blocks periodically to simulate churn without unbounded growth.
        if self.large_blocks.len() > 8 {
            let drain = self.large_blocks.len() / 2;
            self.large_blocks.drain(0..drain);
        }
    }
}
