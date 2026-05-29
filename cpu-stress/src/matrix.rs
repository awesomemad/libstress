/// Dense f64 matrix multiply (O(n³)) for sustained FP throughput.
pub fn matrix_multiply(size: usize) -> f64 {
    let n = size.max(8);
    let mut a = vec![0.0f64; n * n];
    let mut b = vec![0.0f64; n * n];
    let mut c = vec![0.0f64; n * n];

    for i in 0..n {
        a[i * n + i] = 1.0;
        b[i * n + i] = (i as f64) * 0.001 + 1.0;
    }

    for i in 0..n {
        for k in 0..n {
            let a_ik = a[i * n + k];
            for j in 0..n {
                c[i * n + j] += a_ik * b[k * n + j];
            }
        }
    }

    c.iter().sum()
}
