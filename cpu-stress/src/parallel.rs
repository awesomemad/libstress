use stress_core::{Intensity, StopSignal};

use crate::branch_chaos;
use crate::hash_loop;
use crate::matrix;

#[derive(Debug, Clone, Copy)]
pub enum CpuWorkload {
    Hashing,
    Matrix,
    BranchChaos,
    Mixed,
}

/// One round-robin dispatch across CPU algorithms.
pub fn dispatch_round(intensity: Intensity, stop: &StopSignal) {
    static ROUND: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let round = ROUND.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 4;

    let base_iters = intensity.scale_usize(50_000);
    let matrix_size = intensity.scale_usize(64).clamp(32, 256);

    match round {
        0 => {
            rayon::scope(|s| {
                for tid in 0..rayon::current_num_threads() {
                    let iters = base_iters + tid * 10_000;
                    s.spawn(move |_| {
                        let _ = hash_loop::hash_iterations(iters);
                    });
                }
            });
        }
        1 => {
            let _ = matrix::matrix_multiply(matrix_size);
        }
        2 => {
            rayon::scope(|s| {
                for tid in 0..rayon::current_num_threads() {
                    s.spawn(move |_| {
                        let _ = branch_chaos::chaotic_branches(base_iters, tid as u64 + 1);
                    });
                }
            });
        }
        _ => {
            if stop.should_stop() {
                return;
            }
            let _ = hash_loop::hash_iterations(base_iters / 2);
            let _ = matrix::matrix_multiply(matrix_size / 2);
            let _ = branch_chaos::chaotic_branches(base_iters / 2, 42);
        }
    }
}

pub fn run_workload(kind: CpuWorkload, intensity: Intensity, stop: &StopSignal) {
    while !stop.should_stop() {
        dispatch_round_for(kind, intensity);
    }
}

fn dispatch_round_for(kind: CpuWorkload, intensity: Intensity) {
    let base_iters = intensity.scale_usize(80_000);
    let matrix_size = intensity.scale_usize(96).clamp(32, 320);

    match kind {
        CpuWorkload::Hashing => {
            let _ = hash_loop::hash_iterations(base_iters);
        }
        CpuWorkload::Matrix => {
            let _ = matrix::matrix_multiply(matrix_size);
        }
        CpuWorkload::BranchChaos => {
            let _ = branch_chaos::chaotic_branches(base_iters, 7);
        }
        CpuWorkload::Mixed => dispatch_round(intensity, &StopSignal::new()),
    }
}
