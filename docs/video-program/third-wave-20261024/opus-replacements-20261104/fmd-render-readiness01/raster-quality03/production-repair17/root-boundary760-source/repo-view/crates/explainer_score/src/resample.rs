//! Windowed-sinc resampling (64 taps, Blackman window, DC-normalized), the same
//! design franken_manim's mixer uses. Used to bring 24 kHz narration to 48 kHz
//! for ducking and QA. (The PCM master replays cues through franken_manim's own
//! mixer instead, so it matches bit for bit; this one only needs to be good.)

use std::f64::consts::PI;

const TAPS: i64 = 64;

fn blackman(n: f64, len: f64) -> f64 {
    // n in [0, len]
    let a = 2.0 * PI * n / len;
    0.42 - 0.5 * a.cos() + 0.08 * (2.0 * a).cos()
}

fn sinc(x: f64) -> f64 {
    if x.abs() < 1e-12 {
        1.0
    } else {
        (PI * x).sin() / (PI * x)
    }
}

/// Resample `x` from `from` Hz to `to` Hz. Output length is
/// `round(len * to / from)`.
pub fn resample(x: &[f64], from: u32, to: u32) -> Vec<f64> {
    if from == to {
        return x.to_vec();
    }
    let ratio = f64::from(to) / f64::from(from);
    let out_len = (x.len() as f64 * ratio).round() as usize;
    // Low-pass at the lower Nyquist (anti-aliasing when downsampling).
    let fc = ratio.min(1.0);
    let half = TAPS / 2;
    let mut y = Vec::with_capacity(out_len);
    for j in 0..out_len {
        let p = j as f64 / ratio; // position in input samples
        let base = p.floor() as i64;
        let (mut acc, mut wsum) = (0.0, 0.0);
        for k in (base - half + 1)..=(base + half) {
            let d = p - k as f64;
            let w = blackman(d + half as f64, TAPS as f64) * fc * sinc(fc * d);
            wsum += w;
            if k >= 0 && (k as usize) < x.len() {
                acc += w * x[k as usize];
            }
        }
        // DC normalization: a constant input stays exactly that constant.
        y.push(if wsum.abs() > 1e-12 { acc / wsum } else { 0.0 });
    }
    y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_and_lengths() {
        let x: Vec<f64> = (0..100).map(|i| i as f64).collect();
        assert_eq!(resample(&x, 48_000, 48_000), x);
        assert_eq!(resample(&x, 24_000, 48_000).len(), 200);
        assert_eq!(resample(&x, 48_000, 24_000).len(), 50);
    }

    #[test]
    fn dc_is_preserved_in_the_interior() {
        let x = vec![0.25; 2_000];
        let y = resample(&x, 24_000, 48_000);
        for v in &y[100..3_900] {
            assert!((v - 0.25).abs() < 1e-9, "{v}");
        }
    }

    #[test]
    fn a_tone_survives_upsampling() {
        // 1 kHz at 24 kHz -> 48 kHz: amplitude and frequency preserved.
        let x: Vec<f64> = (0..24_000)
            .map(|i| (2.0 * PI * 1_000.0 * i as f64 / 24_000.0).sin())
            .collect();
        let y = resample(&x, 24_000, 48_000);
        let peak = y[2_000..46_000].iter().fold(0.0_f64, |m, v| m.max(v.abs()));
        assert!((peak - 1.0).abs() < 0.01, "{peak}");
        // zero crossings ≈ 2 per period → 2,000 per second over 0.5 s windows
        let zc = y[10_000..34_000]
            .windows(2)
            .filter(|w| w[0] <= 0.0 && w[1] > 0.0)
            .count();
        assert!((zc as i64 - 500).abs() <= 1, "{zc}");
    }
}
