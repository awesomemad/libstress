use stress_core::Intensity;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;

pub fn write_cycle(work_dir: &Path, intensity: Intensity, threads: usize) {
    let file_count = intensity.scale_usize(8).max(2);
    let buffer_kb = intensity.scale_usize(256).max(64);
    let buffer = vec![0xABu8; buffer_kb * 1024];

    rayon::scope(|scope| {
        for tid in 0..threads {
            let buf = buffer.clone();
            let dir = work_dir.to_path_buf();
            scope.spawn(move |_| {
                for i in 0..file_count {
                    let path = dir.join(format!("stress-{tid}-{i}.bin"));
                    if let Ok(mut file) = File::create(&path) {
                        let _ = file.write_all(&buf);
                    }
                }
            });
        }
    });
}

pub fn delete_cycle(work_dir: &Path) {
    let Ok(entries) = fs::read_dir(work_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let _ = fs::remove_file(path);
        }
    }
}

pub fn recreate_small_files(work_dir: &Path, count: usize) {
    let _ = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true);
    for i in 0..count {
        let path = work_dir.join(format!("tiny-{i}.tmp"));
        if let Ok(mut f) = File::create(path) {
            let _ = f.write_all(b"libstress");
        }
    }
}
