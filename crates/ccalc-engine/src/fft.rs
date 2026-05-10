//! FFT computation helpers — thin wrapper around `rustfft`.

use rustfft::FftPlanner;
use rustfft::num_complex::Complex;

/// Computes the forward DFT of `data` with output length `n`.
///
/// Input is zero-padded when `n > data.len()`; truncated when `n < data.len()`.
/// Returns `n` complex pairs `(re, im)`.
pub(crate) fn fft_forward(data: &[f64], n: usize) -> Vec<(f64, f64)> {
    let mut planner = FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(n);
    let mut buffer: Vec<Complex<f64>> = (0..n)
        .map(|i| Complex::new(if i < data.len() { data[i] } else { 0.0 }, 0.0))
        .collect();
    fft.process(&mut buffer);
    buffer.into_iter().map(|c| (c.re, c.im)).collect()
}

/// Computes the inverse DFT of `data`, normalised by 1/N.
///
/// Returns N complex pairs `(re, im)`.
pub(crate) fn fft_inverse(data: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let n = data.len();
    if n == 0 {
        return vec![];
    }
    let mut planner = FftPlanner::<f64>::new();
    let ifft = planner.plan_fft_inverse(n);
    let mut buffer: Vec<Complex<f64>> = data.iter().map(|&(re, im)| Complex::new(re, im)).collect();
    ifft.process(&mut buffer);
    let norm = n as f64;
    buffer
        .into_iter()
        .map(|c| (c.re / norm, c.im / norm))
        .collect()
}
