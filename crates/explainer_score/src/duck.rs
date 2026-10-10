//! Offline ducking with perfect look-ahead, and the speech pocket.
//!
//! The narration is known before a note is synthesized, so the music can make
//! room for every sentence exactly on time: speech regions come from the voice
//! bus itself; the bed drops 8 dB with a 0.3 s raised-cosine ramp that starts
//! 0.08 s early, holds 0.25 s after the voice stops and releases over 1.2 s;
//! and while ducked, a zero-phase EQ "pocket" opens 1–4 kHz (where consonants
//! carry intelligibility) and the 300 Hz–1 kHz centre where the narrator sits.
//! Plan §7.2; docs/research/sound_design/MEMO.md §2.3.

use crate::filter::{Biquad, Kind, filtfilt};
use crate::fx::rcos;
use crate::units::db_to_gain;
use crate::{FS, Stereo};

/// VAD frame length (10 ms at 48 kHz).
pub const VAD_FRAME: usize = 480;

/// Per-frame voice activity of a voice bus: a frame is active when its RMS is
/// within 25 dB of the bus's typical speech level, dilated by ±30 ms.
pub fn vad(voice: &Stereo) -> Vec<bool> {
    let n = voice.len() / VAD_FRAME;
    let db: Vec<f64> = (0..n)
        .map(|k| {
            let s = k * VAD_FRAME;
            let e: f64 = (s..s + VAD_FRAME)
                .map(|i| 0.5 * (voice.l[i] * voice.l[i] + voice.r[i] * voice.r[i]))
                .sum();
            10.0 * (e / VAD_FRAME as f64 + 1e-20).log10()
        })
        .collect();
    // typical speech level: median of the frames above −60 dBFS
    let mut loud: Vec<f64> = db.iter().copied().filter(|&d| d > -60.0).collect();
    if loud.is_empty() {
        return vec![false; n];
    }
    loud.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let typical = loud[loud.len() / 2];
    let raw: Vec<bool> = db.iter().map(|&d| d > typical - 25.0).collect();
    let mut out = vec![false; n];
    for (k, &a) in raw.iter().enumerate() {
        if a {
            for o in out.iter_mut().take((k + 4).min(n)).skip(k.saturating_sub(3)) {
                *o = true;
            }
        }
    }
    out
}

/// Speech regions in seconds, merging gaps shorter than `merge_gap_s`.
pub fn regions(active: &[bool], merge_gap_s: f64) -> Vec<(f64, f64)> {
    let frame_s = VAD_FRAME as f64 / FS;
    let mut out: Vec<(f64, f64)> = Vec::new();
    let mut start = None;
    for (k, &a) in active.iter().chain(std::iter::once(&false)).enumerate() {
        match (a, start) {
            (true, None) => start = Some(k),
            (false, Some(s)) => {
                let (t0, t1) = (s as f64 * frame_s, k as f64 * frame_s);
                match out.last_mut() {
                    Some(last) if t0 - last.1 < merge_gap_s => last.1 = t1,
                    _ => out.push((t0, t1)),
                }
                start = None;
            }
            _ => {}
        }
    }
    out
}

/// Gaps between speech regions of at least `min_s` seconds, within [0, total].
pub fn gaps(regions: &[(f64, f64)], total_s: f64, min_s: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    let mut t = 0.0;
    for &(a, b) in regions {
        if a - t >= min_s {
            out.push((t, a));
        }
        t = t.max(b);
    }
    if total_s - t >= min_s {
        out.push((t, total_s));
    }
    out
}

/// Duck shape timing.
#[derive(Clone, Copy, Debug)]
pub struct DuckShape {
    pub attack: f64,
    pub pre_roll: f64,
    pub hold: f64,
    pub release: f64,
}

impl Default for DuckShape {
    fn default() -> Self {
        Self {
            attack: 0.30,
            pre_roll: 0.08,
            hold: 0.25,
            release: 1.2,
        }
    }
}

/// Per-sample duck amount u ∈ [0, 1] (1 = fully ducked): the maximum over all
/// regions of a raised-cosine attack, a hold and a raised-cosine release, so
/// overlapping ramps never re-open a gap.
pub fn duck_curve(regions: &[(f64, f64)], n: usize, shape: DuckShape) -> Vec<f64> {
    let mut u = vec![0.0_f64; n];
    for &(s, e) in regions {
        let a0 = s - shape.pre_roll - shape.attack;
        let full = s - shape.pre_roll;
        let h1 = e + shape.hold;
        let r1 = h1 + shape.release;
        let i0 = (a0.max(0.0) * FS) as usize;
        let i1 = ((r1 * FS) as usize).min(n);
        for (i, v) in u.iter_mut().enumerate().take(i1).skip(i0) {
            let t = i as f64 / FS;
            let w = if t < full {
                rcos((t - a0) / shape.attack)
            } else if t <= h1 {
                1.0
            } else {
                1.0 - rcos((t - h1) / shape.release)
            };
            *v = v.max(w);
        }
    }
    u
}

/// Release the duck inside authored windows (`(start_s, length_s)`): music
/// breaths and listen windows, where the music is meant to be heard. The duck
/// fades out over `ramp_s` at the window's start and back in at its end.
pub fn release_windows(u: &mut [f64], windows: &[(f64, f64)], ramp_s: f64) {
    for &(start, len) in windows {
        let (a, b) = (start, start + len);
        let i0 = (a.max(0.0) * FS) as usize;
        let i1 = ((b * FS) as usize).min(u.len());
        for (i, v) in u.iter_mut().enumerate().take(i1).skip(i0) {
            let t = i as f64 / FS;
            let open = rcos(((t - a) / ramp_s).min((b - t) / ramp_s));
            *v *= 1.0 - open;
        }
    }
}

/// The pocket EQ (zero phase): −3 dB 300 Hz–1 kHz on the mid channel only (the
/// narrator's centre), then −5 dB around 2 kHz (≈1–4 kHz) on both channels.
pub fn pocket(x: &Stereo) -> Stereo {
    let mid: Vec<f64> = x.l.iter().zip(&x.r).map(|(a, b)| 0.5 * (a + b)).collect();
    let side: Vec<f64> = x.l.iter().zip(&x.r).map(|(a, b)| 0.5 * (a - b)).collect();
    // filtfilt squares the magnitude: design each pass for half the dB
    let mid = filtfilt(&mid, || Biquad::new(Kind::Peaking { q: 0.8, db: -1.5 }, 550.0));
    let l: Vec<f64> = mid.iter().zip(&side).map(|(m, s)| m + s).collect();
    let r: Vec<f64> = mid.iter().zip(&side).map(|(m, s)| m - s).collect();
    let bell = || Biquad::new(Kind::Peaking { q: 0.7, db: -2.5 }, 2_000.0);
    // hard rule 2: under speech, content above 2.5 kHz is cut by ≥ 12 dB; a
    // zero-phase 2.4 kHz Butterworth (run twice: 4th-order slope) does that
    let lp = || {
        Biquad::new(
            Kind::LowPass {
                q: std::f64::consts::FRAC_1_SQRT_2,
            },
            2_400.0,
        )
    };
    Stereo {
        l: filtfilt(&filtfilt(&l, bell), lp),
        r: filtfilt(&filtfilt(&r, bell), lp),
    }
}

/// Apply the duck (`depth_db`) and the pocket, crossfaded by `u`. Because the
/// pocket is zero-phase, the crossfade is an exact, click-free time-varying EQ.
pub fn apply(music: &Stereo, u: &[f64], depth_db: f64) -> Stereo {
    let p = pocket(music);
    let mut out = Stereo::silence(music.len());
    for i in 0..music.len() {
        let w = u.get(i).copied().unwrap_or(0.0);
        let g = db_to_gain(-depth_db * w);
        out.l[i] = g * ((1.0 - w) * music.l[i] + w * p.l[i]);
        out.r[i] = g * ((1.0 - w) * music.r[i] + w * p.r[i]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::TAU;

    /// Speech-like test signal: bursts of noise-modulated tone at the given
    /// spans (seconds) over `total` seconds.
    fn bursts(spans: &[(f64, f64)], total: f64) -> Stereo {
        let n = (total * FS) as usize;
        let mut w = crate::osc::White::new(1);
        let m: Vec<f64> = (0..n)
            .map(|i| {
                let t = i as f64 / FS;
                if spans.iter().any(|&(a, b)| t >= a && t < b) {
                    0.1 * (TAU * 150.0 * t).sin() * (0.6 + 0.4 * w.sample())
                } else {
                    0.0
                }
            })
            .collect();
        Stereo::from_mono(&m)
    }

    #[test]
    fn vad_finds_speech_and_regions_merge_short_pauses() {
        let v = bursts(&[(1.0, 3.0), (3.5, 5.0), (8.0, 9.0)], 10.0);
        let r = regions(&vad(&v), 1.2);
        assert_eq!(r.len(), 2, "{r:?}");
        assert!((r[0].0 - 1.0).abs() < 0.05 && (r[0].1 - 5.0).abs() < 0.05, "{r:?}");
        assert!((r[1].0 - 8.0).abs() < 0.05 && (r[1].1 - 9.0).abs() < 0.05, "{r:?}");
        // gaps of at least 2 s: only 5–8 (0–1 and 9–10 are 1 s each)
        let g = gaps(&r, 10.0, 2.0);
        assert_eq!(g.len(), 1, "{g:?}");
        assert!((g[0].0 - 5.0).abs() < 0.05 && (g[0].1 - 8.0).abs() < 0.05, "{g:?}");
    }

    #[test]
    fn duck_curve_shape_and_continuity() {
        let n = (10.0 * FS) as usize;
        let u = duck_curve(&[(2.0, 4.0)], n, DuckShape::default());
        let at = |t: f64| u[(t * FS) as usize];
        assert_eq!(at(1.0), 0.0);
        assert!((at(1.92) - 1.0).abs() < 1e-6, "full duck 0.08 s before speech");
        assert!((at(3.0) - 1.0).abs() < 1e-12);
        assert!((at(4.25) - 1.0).abs() < 1e-3, "hold 0.25 s past the end");
        assert!(at(5.45).abs() < 1e-3, "released after 1.2 s");
        // continuity: no step larger than a smooth ramp allows
        let max_step = u.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0, f64::max);
        assert!(max_step < 1e-3, "{max_step}");
        // monotone on each ramp
        let (a, b) = ((1.62 * FS) as usize, (1.92 * FS) as usize);
        assert!(u[a..b].windows(2).all(|w| w[1] >= w[0]));
    }

    #[test]
    fn overlapping_regions_never_reopen_the_gap() {
        let n = (10.0 * FS) as usize;
        let u = duck_curve(&[(1.0, 2.0), (3.0, 4.0)], n, DuckShape::default());
        // between 2.25 (hold end) and 2.62 (next attack start) the release and
        // the next attack overlap: the max of the two never dips to zero
        let dip = u[(2.25 * FS) as usize..(2.92 * FS) as usize]
            .iter()
            .cloned()
            .fold(1.0, f64::min);
        assert!(dip > 0.5, "{dip}");
    }

    #[test]
    fn listen_windows_release_the_duck_fully() {
        let n = (10.0 * FS) as usize;
        let mut u = duck_curve(&[(1.0, 9.0)], n, DuckShape::default());
        release_windows(&mut u, &[(4.0, 3.0)], 0.3);
        let at = |t: f64| u[(t * FS) as usize];
        assert!((at(3.5) - 1.0).abs() < 1e-9, "outside the window: still ducked");
        assert!(at(5.5).abs() < 1e-9, "inside: fully released");
        assert!(at(4.15) > 0.0 && at(4.15) < 1.0, "smooth ramp in");
        let max_step = u.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0, f64::max);
        assert!(max_step < 1e-3, "{max_step}");
    }

    #[test]
    fn pocket_carves_the_speech_band_cuts_the_top_and_spares_the_bass() {
        let tone = |hz: f64| -> Stereo {
            let m: Vec<f64> = (0..48_000).map(|i| (TAU * hz * i as f64 / FS).sin()).collect();
            Stereo::from_mono(&m)
        };
        let gain_db = |hz: f64| {
            let x = tone(hz);
            let y = pocket(&x);
            let e = |v: &[f64]| v[12_000..36_000].iter().map(|s| s * s).sum::<f64>();
            10.0 * (e(&y.l) / e(&x.l)).log10()
        };
        assert!(gain_db(80.0).abs() < 0.3, "bass spared: {}", gain_db(80.0));
        assert!((gain_db(550.0) + 3.0).abs() < 1.2, "centre: {}", gain_db(550.0));
        assert!(gain_db(1_500.0) < -4.0, "consonant band: {}", gain_db(1_500.0));
        for hz in [3_500.0, 5_000.0, 8_000.0] {
            assert!(gain_db(hz) <= -12.0, "{hz} Hz: {} (hard rule 2)", gain_db(hz));
        }
    }

    #[test]
    fn apply_ducks_by_the_requested_depth() {
        let m: Vec<f64> = (0..48_000).map(|i| 0.5 * (TAU * 120.0 * i as f64 / FS).sin()).collect();
        let x = Stereo::from_mono(&m);
        let u = vec![1.0; x.len()];
        let y = apply(&x, &u, 8.0);
        let r = |v: &[f64]| (v[12_000..36_000].iter().map(|s| s * s).sum::<f64>()).sqrt();
        let g = 20.0 * (r(&y.l) / r(&x.l)).log10();
        assert!((g + 8.0).abs() < 0.3, "{g}");
        let none = apply(&x, &vec![0.0; x.len()], 8.0);
        assert_eq!(none.l, x.l);
    }
}
