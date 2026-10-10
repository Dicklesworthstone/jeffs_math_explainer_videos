//! Loudness (ITU-R BS.1770 / EBU R128) and true peak, at 48 kHz.
//!
//! Integrated loudness gates 400 ms blocks (75% overlap) absolutely at −70 LUFS
//! and relatively at −10 LU; momentary uses 400 ms and short-term 3 s windows.
//! True peak oversamples 4× with a windowed-sinc interpolator.

use crate::filter::Biquad;
use crate::{FS, Stereo};

/// The K-weighting pre-filter (BS.1770 stage 1 shelf + stage 2 high-pass),
/// coefficients as published for 48 kHz.
fn k_weight(x: &[f64]) -> Vec<f64> {
    let mut s1 = Biquad::from_coefficients(
        [1.535_124_859_586_97, -2.691_696_189_406_38, 1.198_392_810_852_85],
        [-1.690_659_293_182_41, 0.732_480_774_215_85],
    );
    let mut s2 = Biquad::from_coefficients([1.0, -2.0, 1.0], [-1.990_047_454_833_98, 0.990_072_250_366_21]);
    x.iter().map(|&v| s2.process(s1.process(v))).collect()
}

/// Per-block mean-square sums (both channels, weights 1) for blocks of
/// `block` samples stepping by `step`.
fn block_powers(audio: &Stereo, block: usize, step: usize) -> Vec<f64> {
    let (l, r) = (k_weight(&audio.l), k_weight(&audio.r));
    let n = audio.len();
    if n < block {
        return Vec::new();
    }
    // prefix sums of squares for O(1) window means
    let mut pl = vec![0.0; n + 1];
    let mut pr = vec![0.0; n + 1];
    for i in 0..n {
        pl[i + 1] = pl[i] + l[i] * l[i];
        pr[i + 1] = pr[i] + r[i] * r[i];
    }
    (0..=(n - block) / step)
        .map(|k| {
            let (a, b) = (k * step, k * step + block);
            ((pl[b] - pl[a]) + (pr[b] - pr[a])) / block as f64
        })
        .collect()
}

fn lufs(power: f64) -> f64 {
    -0.691 + 10.0 * power.max(1e-30).log10()
}

/// Integrated loudness (LUFS); −∞ (as f64::NEG_INFINITY) for silence.
pub fn integrated(audio: &Stereo) -> f64 {
    let block = (0.4 * FS) as usize;
    let p = block_powers(audio, block, block / 4);
    let abs: Vec<f64> = p.into_iter().filter(|&z| lufs(z) > -70.0).collect();
    if abs.is_empty() {
        return f64::NEG_INFINITY;
    }
    let rel_gate = lufs(abs.iter().sum::<f64>() / abs.len() as f64) - 10.0;
    let gated: Vec<f64> = abs.into_iter().filter(|&z| lufs(z) > rel_gate).collect();
    lufs(gated.iter().sum::<f64>() / gated.len() as f64)
}

/// Loudness in sliding windows of `window_s` seconds, one value per `hop_s`
/// (momentary = 0.4 s, short-term = 3 s). Values are LUFS.
pub fn windowed(audio: &Stereo, window_s: f64, hop_s: f64) -> Vec<f64> {
    let block = (window_s * FS) as usize;
    let step = (hop_s * FS).max(1.0) as usize;
    block_powers(audio, block, step).into_iter().map(lufs).collect()
}

/// Maximum momentary (400 ms) loudness, in LUFS.
pub fn max_momentary(audio: &Stereo) -> f64 {
    windowed(audio, 0.4, 0.1).into_iter().fold(f64::NEG_INFINITY, f64::max)
}

/// True peak (dBTP): the largest absolute value of the 4× oversampled signal.
pub fn true_peak_db(audio: &Stereo) -> f64 {
    const TAPS: i64 = 48; // per phase span (in input samples)
    let mut peak = 0.0_f64;
    for ch in [&audio.l, &audio.r] {
        let n = ch.len() as i64;
        for i in 0..n {
            peak = peak.max(ch[i as usize].abs());
            for phase in 1..4 {
                let t = i as f64 + phase as f64 / 4.0;
                let mut acc = 0.0;
                for k in (i - TAPS / 2 + 1)..=(i + TAPS / 2) {
                    if k < 0 || k >= n {
                        continue;
                    }
                    let d = t - k as f64;
                    let w = 0.42
                        + 0.5 * (std::f64::consts::PI * d / (TAPS as f64 / 2.0)).cos()
                        + 0.08 * (2.0 * std::f64::consts::PI * d / (TAPS as f64 / 2.0)).cos();
                    let s = if d.abs() < 1e-12 {
                        1.0
                    } else {
                        (std::f64::consts::PI * d).sin() / (std::f64::consts::PI * d)
                    };
                    acc += ch[k as usize] * s * w;
                }
                peak = peak.max(acc.abs());
            }
        }
    }
    20.0 * peak.max(1e-12).log10()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::TAU;

    /// A stereo 1 kHz sine at `dbfs` peak level for `seconds` (EBU Tech 3341's
    /// test-signal definition).
    fn tone(dbfs: f64, seconds: f64) -> Stereo {
        let a = 10f64.powf(dbfs / 20.0);
        let n = (seconds * FS) as usize;
        let m: Vec<f64> = (0..n).map(|i| a * (TAU * 1_000.0 * i as f64 / FS).sin()).collect();
        Stereo::from_mono(&m)
    }

    fn concat(parts: &[Stereo]) -> Stereo {
        let mut s = Stereo::default();
        for p in parts {
            s.l.extend_from_slice(&p.l);
            s.r.extend_from_slice(&p.r);
        }
        s
    }

    #[test]
    fn ebu_tech_3341_integrated_cases() {
        // Case 1 and 2: steady tones.
        assert!((integrated(&tone(-23.0, 20.0)) + 23.0).abs() < 0.1);
        assert!((integrated(&tone(-33.0, 20.0)) + 33.0).abs() < 0.1);
        // Case 3: the relative gate ignores the quiet ends.
        let c3 = concat(&[tone(-36.0, 10.0), tone(-23.0, 60.0), tone(-36.0, 10.0)]);
        assert!((integrated(&c3) + 23.0).abs() < 0.1, "{}", integrated(&c3));
        // Case 4: absolute and relative gates together.
        let c4 = concat(&[
            tone(-72.0, 10.0),
            tone(-36.0, 10.0),
            tone(-23.0, 60.0),
            tone(-36.0, 10.0),
            tone(-72.0, 10.0),
        ]);
        assert!((integrated(&c4) + 23.0).abs() < 0.1, "{}", integrated(&c4));
        // Case 5: −26 / −20 / −26 dBFS integrates to −23.
        let c5 = concat(&[tone(-26.0, 20.0), tone(-20.0, 20.1), tone(-26.0, 20.0)]);
        assert!((integrated(&c5) + 23.0).abs() < 0.1, "{}", integrated(&c5));
        assert_eq!(integrated(&Stereo::silence(48_000)), f64::NEG_INFINITY);
    }

    #[test]
    fn momentary_and_short_term_track_a_step() {
        let s = concat(&[tone(-40.0, 5.0), tone(-20.0, 5.0)]);
        let m = windowed(&s, 0.4, 0.1);
        assert!((m[5] + 40.0).abs() < 0.2, "{}", m[5]);
        assert!((m[m.len() - 1] + 20.0).abs() < 0.2);
        assert!((max_momentary(&s) + 20.0).abs() < 0.2);
    }

    #[test]
    fn true_peak_sees_inter_sample_peaks() {
        // fs/4 sine at 45° phase: samples sit at ±0.707·A, the waveform reaches A.
        let a = 0.5;
        let n = 4_800;
        let m: Vec<f64> = (0..n).map(|i| a * (TAU * 0.25 * i as f64 + TAU / 8.0).sin()).collect();
        let s = Stereo::from_mono(&m);
        let sample_peak = 20.0 * s.peak().log10();
        let tp = true_peak_db(&s);
        assert!((tp - 20.0 * a.log10()).abs() < 0.2, "true peak {tp:.2} dB");
        assert!(tp > sample_peak + 2.5, "sample peak {sample_peak:.2}, true {tp:.2}");
    }
}
