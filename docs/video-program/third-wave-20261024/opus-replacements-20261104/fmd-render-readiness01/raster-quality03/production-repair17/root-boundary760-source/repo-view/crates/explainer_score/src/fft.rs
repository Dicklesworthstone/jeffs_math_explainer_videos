//! A small iterative radix-2 FFT and a Welch power-spectrum estimate, enough
//! for score QA (third-octave band levels, spectral checks). std only.

use std::f64::consts::TAU;

/// In-place complex FFT of length 2^k (`re`, `im` of equal power-of-two length).
pub fn fft(re: &mut [f64], im: &mut [f64]) {
    let n = re.len();
    assert!(
        n.is_power_of_two() && im.len() == n,
        "fft length must be a power of two"
    );
    // bit-reversal permutation
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= n {
        let ang = -TAU / len as f64;
        let (wr, wi) = (ang.cos(), ang.sin());
        for start in (0..n).step_by(len) {
            let (mut cr, mut ci) = (1.0, 0.0);
            for k in 0..len / 2 {
                let (a, b) = (start + k, start + k + len / 2);
                let (tr, ti) = (re[b] * cr - im[b] * ci, re[b] * ci + im[b] * cr);
                re[b] = re[a] - tr;
                im[b] = im[a] - ti;
                re[a] += tr;
                im[a] += ti;
                let ncr = cr * wr - ci * wi;
                ci = cr * wi + ci * wr;
                cr = ncr;
            }
        }
        len <<= 1;
    }
}

/// One-sided power spectral density by Welch's method (Hann window, 50%
/// overlap). Returns (bin width in Hz, PSD per bin) with `nfft / 2 + 1` bins,
/// scaled so that Σ PSD · df equals the signal's mean square.
pub fn welch(x: &[f64], fs: f64, nfft: usize) -> (f64, Vec<f64>) {
    assert!(nfft.is_power_of_two());
    let w: Vec<f64> = (0..nfft)
        .map(|i| 0.5 - 0.5 * (TAU * i as f64 / nfft as f64).cos())
        .collect();
    let wpow: f64 = w.iter().map(|v| v * v).sum();
    let mut psd = vec![0.0; nfft / 2 + 1];
    let mut segments = 0usize;
    let hop = nfft / 2;
    let mut start = 0;
    while start + nfft <= x.len() {
        let mut re: Vec<f64> = (0..nfft).map(|i| x[start + i] * w[i]).collect();
        let mut im = vec![0.0; nfft];
        fft(&mut re, &mut im);
        for (k, p) in psd.iter_mut().enumerate() {
            let mag = re[k] * re[k] + im[k] * im[k];
            let one_sided = if k == 0 || k == nfft / 2 { 1.0 } else { 2.0 };
            *p += one_sided * mag / (fs * wpow);
        }
        segments += 1;
        start += hop;
    }
    if segments > 0 {
        for p in &mut psd {
            *p /= segments as f64;
        }
    }
    (fs / nfft as f64, psd)
}

/// Third-octave centre frequencies used by the SII-lite estimate (160 Hz–8 kHz).
pub const THIRD_OCTAVES: [f64; 18] = [
    160.0, 200.0, 250.0, 315.0, 400.0, 500.0, 630.0, 800.0, 1_000.0, 1_250.0, 1_600.0, 2_000.0, 2_500.0, 3_150.0,
    4_000.0, 5_000.0, 6_300.0, 8_000.0,
];

/// Band levels (dB) of `x` in the [`THIRD_OCTAVES`] bands.
pub fn third_octave_levels(x: &[f64], fs: f64) -> [f64; 18] {
    let nfft = 8_192.min(x.len().next_power_of_two() / 2).max(256);
    let (df, psd) = welch(x, fs, nfft);
    let mut out = [0.0; 18];
    for (o, &c) in out.iter_mut().zip(&THIRD_OCTAVES) {
        let (lo, hi) = (c / 2f64.powf(1.0 / 6.0), c * 2f64.powf(1.0 / 6.0));
        let e: f64 = psd
            .iter()
            .enumerate()
            .filter(|(k, _)| (lo..hi).contains(&(*k as f64 * df)))
            .map(|(_, p)| p)
            .sum();
        *o = 10.0 * (e * df + 1e-30).log10();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fft_of_a_pure_tone_has_one_peak() {
        let n = 1_024;
        let mut re: Vec<f64> = (0..n).map(|i| (TAU * 37.0 * i as f64 / n as f64).cos()).collect();
        let mut im = vec![0.0; n];
        fft(&mut re, &mut im);
        let mags: Vec<f64> = re.iter().zip(&im).map(|(a, b)| (a * a + b * b).sqrt()).collect();
        assert!((mags[37] - n as f64 / 2.0).abs() < 1e-6);
        assert!(
            mags.iter()
                .enumerate()
                .filter(|(k, _)| *k != 37 && *k != n - 37)
                .all(|(_, m)| *m < 1e-6)
        );
    }

    #[test]
    fn welch_conserves_power() {
        // A 1 kHz sine of amplitude 0.5 has mean square 0.125.
        let x: Vec<f64> = (0..48_000)
            .map(|i| 0.5 * (TAU * 1_000.0 * i as f64 / 48_000.0).sin())
            .collect();
        let (df, psd) = welch(&x, 48_000.0, 4_096);
        let total: f64 = psd.iter().sum::<f64>() * df;
        assert!((total - 0.125).abs() < 0.002, "{total}");
        let lv = third_octave_levels(&x, 48_000.0);
        let k = THIRD_OCTAVES.iter().position(|&c| c == 1_000.0).unwrap();
        assert!((lv[k] - 10.0 * 0.125f64.log10()).abs() < 0.2, "{}", lv[k]);
        assert!(lv[0] < lv[k] - 40.0);
    }
}
