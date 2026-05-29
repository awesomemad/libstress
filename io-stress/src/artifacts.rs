use stress_core::Intensity;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Simulate build-artifact explosion (many small + few large files).
pub fn explosion_cycle(work_dir: &Path, intensity: Intensity) {
    let artifacts = work_dir.join("artifacts");
    let _ = fs::create_dir_all(&artifacts);

    let small_count = intensity.scale_usize(200).max(20);
    let large_count = intensity.scale_usize(4).max(1);
    let large_size = intensity.scale_usize(512 * 1024).max(64 * 1024);

    for i in 0..small_count {
        let path = artifacts.join(format!("obj-{i}.o"));
        if let Ok(mut f) = File::create(path) {
            let _ = f.write_all(&[i as u8; 64]);
        }
    }

    for i in 0..large_count {
        let path = artifacts.join(format!("lib-{i}.rlib"));
        if let Ok(mut f) = File::create(path) {
            let chunk = vec![0xCDu8; large_size];
            let _ = f.write_all(&chunk);
        }
    }

    // Delete a subset to force recreate cycles (incremental-build pattern).
    if let Ok(entries) = fs::read_dir(&artifacts) {
        for (idx, entry) in entries.flatten().enumerate() {
            if idx % 3 == 0 {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}
