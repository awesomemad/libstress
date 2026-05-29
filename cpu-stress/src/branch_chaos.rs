/// Branch-heavy chaotic iteration resisting prediction.
pub fn chaotic_branches(steps: usize, seed: u64) -> u64 {
    let mut x = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut acc = 0u64;

    for _ in 0..steps {
        x ^= x.rotate_left(13);
        x = x.wrapping_mul(0x2545_F491_4F6C_DD1D);

        acc = acc.wrapping_add(match x & 7 {
            0 => x.wrapping_mul(3),
            1 => x.rotate_right(5),
            2 => x ^ acc,
            3 => acc.wrapping_sub(x),
            4 => x.count_ones() as u64,
            5 => x.leading_zeros() as u64,
            6 => x.wrapping_add(acc >> 3),
            _ => !x,
        });
    }

    acc
}
