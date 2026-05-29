use blake3::Hasher as Blake3;
use sha2::{Digest, Sha256};
use stress_core::Intensity;
use std::fs;
use std::io::Read;
use std::path::Path;

pub fn hash_tree(work_dir: &Path, intensity: Intensity) {
    let Ok(entries) = fs::read_dir(work_dir) else {
        return;
    };

    let use_blake3 = intensity.0 >= 5;
    let mut acc = 0u64;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        acc = acc.wrapping_add(hash_file(&path, use_blake3));
    }

    let _ = acc;
}

fn hash_file(path: &Path, use_blake3: bool) -> u64 {
    let Ok(mut file) = fs::File::open(path) else {
        return 0;
    };

    let mut buf = vec![0u8; 256 * 1024];
    if use_blake3 {
        let mut hasher = Blake3::new();
        while let Ok(n) = file.read(&mut buf) {
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        let hash = hasher.finalize();
        u64::from_le_bytes(hash.as_bytes()[0..8].try_into().unwrap())
    } else {
        let mut hasher = Sha256::new();
        while let Ok(n) = file.read(&mut buf) {
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        let digest = hasher.finalize();
        u64::from_le_bytes(digest[0..8].try_into().unwrap())
    }
}
