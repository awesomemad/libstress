use sha2::{Digest, Sha256};

/// High-iteration hashing loop to burn CPU cycles deterministically.
pub fn hash_iterations(iterations: usize) -> u64 {
    let mut state = [0u8; 32];
    let mut acc: u64 = 0;

    for i in 0..iterations {
        state[0] = (i & 0xFF) as u8;
        state[1] = ((i >> 8) & 0xFF) as u8;
        let digest = Sha256::digest(state);
        acc = acc.wrapping_add(u64::from_le_bytes(digest[0..8].try_into().unwrap()));
        state.copy_from_slice(&digest);
    }

    acc
}
