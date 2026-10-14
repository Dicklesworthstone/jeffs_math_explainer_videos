//! Offline ducking with perfect look-ahead, and the speech pocket.
//!
//! The narration is known before a note is synthesized, so the music can make
//! room for every sentence exactly on time: speech regions come from the voice
//! bus itself; the bed drops 8 dB with a 0.3 s raised-cosine ramp that starts
//! 0.08 s early, holds 0.25 s after the voice stops and releases over 1.2 s;
//! and while ducked, a zero-phase EQ "pocket" opens 1–4 kHz (where consonants
//! carry intelligibility) and the 300 Hz–1 kHz centre where the narrator sits,
//! and a low-pass takes everything above 2.5 kHz down by ≥ 12 dB (hard rule 2).
//! Plan §7.2; docs/research/sound_design/MEMO.md §2.3.

use crate::filter::{Biquad, Kind, filtfilt};
use crate::fx::{from_mid_side, mid_side, rcos};
use crate::meter;
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
/// fades out over `shape.attack` from the window's start and is fully back
/// where the next line's own duck attack begins (`pre_roll + attack` before
/// the window ends, since the next line starts when the window does), so the
/// voice never enters over a half-ducked bed.
pub fn release_windows(u: &mut [f64], windows: &[(f64, f64)], shape: DuckShape) {
    let ramp = shape.attack;
    for &(start, len) in windows {
        let a = start;
        let b = start + len - shape.pre_roll - shape.attack;
        let i0 = (a.max(0.0) * FS) as usize;
        let i1 = ((b.max(0.0) * FS) as usize).min(u.len());
        for (i, v) in u.iter_mut().enumerate().take(i1).skip(i0) {
            let t = i as f64 / FS;
            let open = rcos(((t - a) / ramp).min((b - t) / ramp));
            *v *= 1.0 - open;
        }
    }
}

/// The duck is fully back this long before a voice that returns from an
/// authored window (s).
pub const RECEDE_LEAD: f64 = 0.2;

/// Let the music recede before the voice returns at each of `onsets` (the
/// ends of authored windows the narration resumes from): the duck comes
/// back over `settle` seconds and is full [`RECEDE_LEAD`] before the onset,
/// so a phrase or a sonification ends in space rather than under the first
/// syllable. Never lifts the duck anywhere.
pub fn recede(u: &mut [f64], onsets: &[f64], settle: f64) {
    for &e in onsets {
        let (a, b) = (e - RECEDE_LEAD - settle, e - RECEDE_LEAD);
        let i0 = (a.max(0.0) * FS) as usize;
        let i1 = ((e.max(0.0) * FS) as usize).min(u.len());
        for (i, v) in u.iter_mut().enumerate().take(i1).skip(i0) {
            let t = i as f64 / FS;
            let w = if t >= b { 1.0 } else { rcos((t - a) / settle) };
            *v = v.max(w);
        }
    }
}

/// The pocket EQ (zero phase): −3 dB 300 Hz–1 kHz on the mid channel only (the
/// narrator's centre), then −5 dB around 2 kHz (≈1–4 kHz) and a 2.2 kHz
/// low-pass on both channels: ≈ −9.5 dB at 2 kHz, ≥ 12 dB down from 2.5 kHz up.
pub fn pocket(x: &Stereo) -> Stereo {
    let (mid, side) = mid_side(x);
    // filtfilt squares the magnitude: design each pass for half the dB
    let mid = filtfilt(&mid, || Biquad::new(Kind::Peaking { q: 0.8, db: -1.5 }, 550.0));
    let Stereo { l, r } = from_mid_side(&mid, &side);
    let bell = || Biquad::new(Kind::Peaking { q: 0.7, db: -2.5 }, 2_000.0);
    // hard rule 2: under speech, content above 2.5 kHz is cut by ≥ 12 dB; a
    // 2.2 kHz Butterworth run forward and back (≈ −8.5 dB at 2.5 kHz, 4th-order
    // slope) plus the bell's ≈ −4.4 dB there does that
    let lp = || {
        Biquad::new(
            Kind::LowPass {
                q: std::f64::consts::FRAC_1_SQRT_2,
            },
            2_200.0,
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

/// The duck held for the whole of `music`: [`apply`] with `u` = 1 everywhere
/// (the same to rounding), without the array.
pub fn apply_held(music: &Stereo, depth_db: f64) -> Stereo {
    let mut out = pocket(music);
    out.scale(db_to_gain(-depth_db));
    out
}

/// The percentile of the momentary loudness under speech that a steady bed's
/// ceiling sits at: the median, the bed's ducked level itself.
pub const CEILING_PERCENTILE: f64 = 50.0;
/// What the stem may stand over the ceiling before anything is cut (dB): a
/// stem that is as steady as its ducked level is not touched at all.
pub const CEILING_TOLERANCE_DB: f64 = 0.5;
/// How far from a speech region's edge a ceiling's cut has eased to nothing (s).
pub const CEILING_EDGE_S: f64 = 0.05;
/// How many times the ceiling measures and cuts: a cut that changes over a
/// 0.4 s window leaves a little of what it was cutting, which the next takes.
pub const CEILING_PASSES: usize = 2;

/// What [`ceiling`] did.
#[derive(Clone, Debug, PartialEq)]
pub struct Ceiling {
    /// The ceiling (LUFS, momentary); NaN when there was no speech to set it by.
    pub level: f64,
    /// The most any 0.1 s of the stem was cut (dB, positive).
    pub max_cut_db: f64,
    /// The share of the stem's 0.1 s steps cut by more than 0.5 dB.
    pub share_cut: f64,
}

/// A ceiling for a bed held at its ducked level (`apply_held`): the momentary
/// (400 ms) loudness of the stem outside the `speech` regions (seconds) is
/// kept at or under the `CEILING_PERCENTILE`th percentile of what it is inside
/// them, by one smooth gain (its dB value interpolated between the 0.1 s
/// steps' centres, eased to nothing within `CEILING_EDGE_S` of a speech
/// region), never above 1. What the voice is over is not touched: inside a
/// speech region the stem is the same samples, and a stem that never rises
/// over its ducked level is the same samples everywhere. This is a level
/// that follows the music over 0.4 s, not a peak limiter: it takes a phrase's
/// notes down with the window they sound in, so a phrase written into a gap
/// or a chapter's end stands no higher than the bed does under the voice.
pub fn ceiling(x: &mut Stereo, speech: &[(f64, f64)]) -> Ceiling {
    const HOP: f64 = 0.1;
    let centre = |k: usize| k as f64 * HOP + 0.2;
    let inside = |t: f64| speech.iter().any(|&(a, b)| t >= a && t < b);
    let mut out = Ceiling {
        level: f64::NAN,
        max_cut_db: 0.0,
        share_cut: 0.0,
    };
    let mut total: Vec<f64> = Vec::new();
    for pass in 0..CEILING_PASSES {
        let m = meter::windowed(x, 0.4, HOP);
        if pass == 0 {
            let under: Vec<f64> = (0..m.len())
                .filter(|&k| inside(centre(k)) && m[k] > -100.0)
                .map(|k| m[k])
                .collect();
            if under.is_empty() {
                return out;
            }
            out.level = meter::percentile(&under, CEILING_PERCENTILE);
            total = vec![0.0; m.len()];
        }
        let cut: Vec<f64> = (0..m.len())
            .map(|k| {
                if inside(centre(k)) || m[k] <= -100.0 {
                    0.0
                } else {
                    (out.level + CEILING_TOLERANCE_DB - m[k]).min(0.0)
                }
            })
            .collect();
        if cut.iter().all(|&c| c == 0.0) {
            break;
        }
        for (t, c) in total.iter_mut().zip(&cut) {
            *t += c;
        }
        apply_cut(x, &cut, speech);
    }
    out.max_cut_db = -total.iter().copied().fold(0.0, f64::min);
    out.share_cut = total.iter().filter(|&&c| c < -0.5).count() as f64 / total.len().max(1) as f64;
    out
}

/// The gain (dB, `cut` one value per 0.1 s step, centred 0.2 s + k·0.1 s in:
/// the steps of a 400 ms momentary meter) applied to `x`, interpolated in dB
/// between the centres and eased to nothing at a speech region's edge.
fn apply_cut(x: &mut Stereo, cut: &[f64], speech: &[(f64, f64)]) {
    const HOP: f64 = 0.1;
    let last = cut.len() - 1;
    for i in 0..x.len() {
        let p = ((i as f64 / FS - 0.2) / HOP).clamp(0.0, last as f64);
        let (j, f) = (p.floor() as usize, p.fract());
        let c = cut[j] * (1.0 - f) + cut[(j + 1).min(last)] * f;
        if c == 0.0 {
            continue;
        }
        let w = edge_ease(i as f64 / FS, speech);
        if w == 0.0 {
            continue;
        }
        let g = db_to_gain(c * w);
        x.l[i] *= g;
        x.r[i] *= g;
    }
}

/// What a chapter join may keep of its step after [`carry`] (dB): the join
/// is brought to this much quieter after than before, not to nothing.
pub const CARRY_TOLERANCE_DB: f64 = 1.0;
/// The stretch before a join that [`carry`] holds at its cut (s), then the
/// time it eases in over, and the time it takes to give the cut back after
/// the join.
pub const CARRY_HOLD_S: f64 = 1.7;
pub const CARRY_EASE_IN_S: f64 = 1.5;
pub const CARRY_EASE_OUT_S: f64 = 0.2;

/// Carry a steady bed across the chapter joins at `joins` (seconds). Where
/// the bed is quieter just after a join than just before it by more than
/// `CARRY_TOLERANCE_DB` (the step [`meter::join_step_db`] reads: the end of a
/// chapter, with its last phrase, against the start of the next, which has
/// its own level), the stretch before the join comes down by the difference,
/// held for `CARRY_HOLD_S` and eased in over `CARRY_EASE_IN_S`, and nowhere
/// inside speech. What comes after a join is never touched (the voice is
/// there within a second), and neither is a bed that is already as loud after
/// the join as before it. Returns the cut made at each join (dB, 0 for none).
pub fn carry(x: &mut Stereo, joins: &[f64], speech: &[(f64, f64)]) -> Vec<f64> {
    const HOP: f64 = 0.1;
    let steps = (x.len().saturating_sub((0.4 * FS) as usize)) / (HOP * FS) as usize + 1;
    let mut made = Vec::new();
    for &j in joins {
        let cut_db = meter::join_step_db(x, j).map_or(0.0, |s| (s + CARRY_TOLERANCE_DB).min(0.0));
        made.push(cut_db);
        if cut_db == 0.0 {
            continue;
        }
        let cut: Vec<f64> = (0..steps)
            .map(|k| {
                let d = k as f64 * HOP + 0.2 - j;
                let shape = if d < -(CARRY_HOLD_S + CARRY_EASE_IN_S) {
                    0.0
                } else if d < -CARRY_HOLD_S {
                    rcos((d + CARRY_HOLD_S + CARRY_EASE_IN_S) / CARRY_EASE_IN_S)
                } else if d <= -0.05 {
                    1.0
                } else if d < -0.05 + CARRY_EASE_OUT_S {
                    1.0 - rcos((d + 0.05) / CARRY_EASE_OUT_S)
                } else {
                    0.0
                };
                cut_db * shape
            })
            .collect();
        apply_cut(x, &cut, speech);
    }
    made
}

/// 0 inside a speech region, rising to 1 over `CEILING_EDGE_S` outside it.
fn edge_ease(t: f64, speech: &[(f64, f64)]) -> f64 {
    let after = speech.partition_point(|r| r.1 <= t);
    let mut d = f64::INFINITY;
    if let Some(r) = speech.get(after) {
        d = d.min((r.0 - t).max(0.0));
    }
    if after > 0 {
        d = d.min(t - speech[after - 1].1);
    }
    (d / CEILING_EDGE_S).min(1.0)
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
        release_windows(&mut u, &[(4.0, 3.0)], DuckShape::default());
        let at = |t: f64| u[(t * FS) as usize];
        assert!((at(3.5) - 1.0).abs() < 1e-9, "outside the window: still ducked");
        assert!(at(5.5).abs() < 1e-9, "inside: fully released");
        assert!(at(4.15) > 0.0 && at(4.15) < 1.0, "smooth ramp in");
        assert!((at(6.62) - 1.0).abs() < 1e-9, "back down before the window ends");
        let max_step = u.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0, f64::max);
        assert!(max_step < 1e-3, "{max_step}");
    }

    #[test]
    fn a_two_and_a_half_second_breath_is_heard_and_the_next_line_is_ducked() {
        // A line ends at 5.0 s, `music_breath(2.5)`, the next line at 7.5 s.
        let shape = DuckShape::default();
        let n = (12.0 * FS) as usize;
        let mut u = duck_curve(&[(0.5, 5.0), (7.5, 11.0)], n, shape);
        release_windows(&mut u, &[(5.0, 2.5)], shape);
        let at = |t: f64| u[(t * FS) as usize];
        // open from one ramp after the breath starts until the next attack's ramp
        for t in [5.31, 5.8, 6.4, 6.8] {
            assert!(at(t) < 1e-9, "t = {t}: u = {}", at(t));
        }
        // fully ducked where the next line's pre-roll begins, and while it speaks
        assert!((at(7.42) - 1.0).abs() < 1e-9, "{}", at(7.42));
        assert!((at(7.5) - 1.0).abs() < 1e-9);
        let max_step = u.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0, f64::max);
        assert!(max_step < 1e-3, "{max_step}");
    }

    #[test]
    fn a_long_settle_lets_the_music_recede_before_the_voice_returns() {
        // the breath of the test above with a 0.8 s settle: open as long as
        // the ramps allow, half closed 0.4 s into the settle, fully back at
        // the same point before the next line
        let shape = DuckShape::default();
        let n = (12.0 * FS) as usize;
        let mut u = duck_curve(&[(0.5, 5.0), (7.5, 11.0)], n, shape);
        release_windows(&mut u, &[(5.0, 2.5)], shape);
        recede(&mut u, &[7.5], 0.8);
        let at = |t: f64| u[(t * FS) as usize];
        assert!(at(5.6) < 1e-9 && at(6.4) < 1e-9, "{} {}", at(5.6), at(6.4));
        // the settle runs 6.5–7.3 (fully back RECEDE_LEAD before the voice)
        assert!((at(6.9) - 0.5).abs() < 0.01, "{}", at(6.9));
        assert!((at(7.3) - 1.0).abs() < 1e-3 && (at(7.5) - 1.0).abs() < 1e-9);
        // the first line, untouched
        assert!((at(3.0) - 1.0).abs() < 1e-9);
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
        // the designed response (each biquad's magnitude, doubled by filtfilt;
        // a centred tone has no side, so the mid-only bell applies in full)
        let designed = |hz: f64| {
            2.0 * (Biquad::new(Kind::Peaking { q: 0.8, db: -1.5 }, 550.0).magnitude_db(hz)
                + Biquad::new(Kind::Peaking { q: 0.7, db: -2.5 }, 2_000.0).magnitude_db(hz)
                + Biquad::new(
                    Kind::LowPass {
                        q: std::f64::consts::FRAC_1_SQRT_2,
                    },
                    2_200.0,
                )
                .magnitude_db(hz))
        };
        for hz in [80.0, 300.0, 550.0, 1_000.0, 2_000.0, 2_500.0, 4_000.0] {
            let (m, d) = (gain_db(hz), designed(hz));
            assert!((m - d).abs() < 0.5, "{hz} Hz: measured {m:.2}, designed {d:.2}");
        }
        assert!(gain_db(80.0).abs() < 0.3, "bass spared: {}", gain_db(80.0));
        assert!((gain_db(550.0) + 3.0).abs() < 1.2, "centre: {}", gain_db(550.0));
        assert!(gain_db(1_500.0) < -4.0, "consonant band: {}", gain_db(1_500.0));
        for hz in [2_500.0, 3_500.0, 5_000.0, 8_000.0] {
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
    /// A 220 Hz tone at 0.05 for `total` s, `extra_db` louder inside each span.
    fn tone(total: f64, plants: &[(f64, f64, f64)]) -> Stereo {
        let n = (total * FS) as usize;
        let m: Vec<f64> = (0..n)
            .map(|i| {
                let t = i as f64 / FS;
                let extra: f64 = plants.iter().filter(|p| t >= p.0 && t < p.1).map(|p| p.2).sum();
                0.05 * db_to_gain(extra) * (TAU * 220.0 * t).sin()
            })
            .collect();
        Stereo::from_mono(&m)
    }

    #[test]
    fn a_held_duck_is_the_duck_with_u_at_one() {
        let x = bursts(&[(0.5, 2.0)], 3.0);
        let (a, b) = (apply(&x, &vec![1.0; x.len()], 8.0), apply_held(&x, 8.0));
        let worst = a.l.iter().zip(&b.l).fold(0.0_f64, |m, (x, y)| m.max((x - y).abs()));
        assert!(worst <= 1e-15 * a.peak().max(1e-9), "{worst:e}");
    }

    #[test]
    fn the_ceiling_keeps_a_swell_to_the_level_under_speech_and_leaves_speech_alone() {
        let speech = [(2.0, 8.0), (12.0, 18.0)];
        // 10 dB up for 3 s in the gap between the two
        let swollen = tone(20.0, &[(8.5, 11.5, 10.0)]);
        let mut held = swollen.clone();
        let c = ceiling(&mut held, &speech);
        let level = |x: &Stereo, a: f64, b: f64| {
            let m = meter::windowed(x, 0.4, 0.1);
            let v: Vec<f64> = (0..m.len())
                .filter(|&k| (a..b).contains(&(k as f64 * 0.1 + 0.2)))
                .map(|k| m[k])
                .collect();
            meter::percentile(&v, 50.0)
        };
        let (before, after, under) = (
            level(&swollen, 9.2, 10.8),
            level(&held, 9.2, 10.8),
            level(&held, 3.0, 7.0),
        );
        assert!((before - under - 10.0).abs() < 0.3, "planted {:.1} dB", before - under);
        assert!(
            after - under < 0.5,
            "the gap stands {:.2} dB over the speech after it",
            after - under
        );
        assert!(
            after - under > -1.0,
            "and is not taken far under it: {:.2}",
            after - under
        );
        assert!((c.level - under).abs() < 0.5, "{c:?} against {under:.2}");
        assert!(c.max_cut_db > 9.0 && c.max_cut_db < 11.0, "{c:?}");
        assert!(c.share_cut > 0.1 && c.share_cut < 0.3, "{c:?}");
        // the stem inside each speech region is the very same samples, and so
        // is everything 0.1 s clear of the swell
        for (i, (a, b)) in swollen.l.iter().zip(&held.l).enumerate() {
            let t = i as f64 / FS;
            if speech.iter().any(|&(s, e)| (s..e).contains(&t)) || !(8.3..11.7).contains(&t) {
                assert!(a == b, "sample {i} at {t:.3} s moved");
            }
        }
    }

    #[test]
    fn a_stem_that_never_rises_over_its_ducked_level_is_not_touched() {
        let speech = [(2.0, 8.0), (12.0, 18.0)];
        // the gap is 6 dB under the speech, as a thinned bed is
        let flat = tone(20.0, &[(8.5, 11.5, -6.0)]);
        let mut same = flat.clone();
        let c = ceiling(&mut same, &speech);
        assert!(same.l == flat.l && same.r == flat.r);
        assert_eq!((c.max_cut_db, c.share_cut), (0.0, 0.0));
        // and with no speech there is no level to hold it to
        let mut none = tone(5.0, &[(1.0, 2.0, 10.0)]);
        let before = none.clone();
        assert!(ceiling(&mut none, &[]).level.is_nan());
        assert!(none.l == before.l);
    }
    #[test]
    fn a_join_that_steps_down_is_carried_by_taking_the_end_of_the_chapter_down() {
        let speech = [(2.0, 8.0), (15.6, 28.0)];
        // the chapter's end, after its last line, 4 dB over the next one's start
        let stepped = tone(30.0, &[(8.5, 15.0, 4.0)]);
        let step = |x: &Stereo| meter::join_step_db(x, 15.0).unwrap();
        assert!((step(&stepped) + 4.0).abs() < 0.3, "{:.2}", step(&stepped));
        let mut carried = stepped.clone();
        let made = carry(&mut carried, &[15.0], &speech);
        // brought to the tolerance, not to nothing
        assert!((made[0] + 3.0).abs() < 0.3, "{made:?}");
        assert!(
            (step(&carried) + CARRY_TOLERANCE_DB).abs() < 0.3,
            "{:.2}",
            step(&carried)
        );
        // the voice's frames, and the chapter after the join, are the same samples
        // (the cut is given back over 0.2 s from just before the join)
        for (i, (a, b)) in stepped.l.iter().zip(&carried.l).enumerate() {
            let t = i as f64 / FS;
            if speech.iter().any(|&(s, e)| (s..e).contains(&t)) || t > 15.2 || t < 9.0 {
                assert!(a == b, "sample {i} at {t:.3} s moved");
            }
        }
        // no step in the gain: the sample-to-sample ratio of the cut is smooth
        let g: Vec<f64> = (0..stepped.len())
            .filter(|&i| stepped.l[i].abs() > 0.04)
            .map(|i| carried.l[i] / stepped.l[i])
            .collect();
        assert!(g.windows(2).all(|w| (w[1] / w[0] - 1.0).abs() < 5e-3), "the cut steps");
    }

    #[test]
    fn a_join_within_the_tolerance_or_stepping_up_is_left_alone() {
        let speech = [(2.0, 8.0), (15.6, 28.0)];
        for plant in [0.8, 0.0, -0.8, -3.0] {
            // the end of the chapter `plant` dB over the next one's start
            let flat = tone(30.0, &[(8.5, 15.0, plant)]);
            let mut same = flat.clone();
            assert_eq!(carry(&mut same, &[15.0], &speech), vec![0.0]);
            assert!(same.l == flat.l && same.r == flat.r, "{plant}");
        }
        // a join with nothing to measure (silent) makes no cut and no noise
        let mut quiet = Stereo::silence(30 * 48_000);
        assert_eq!(carry(&mut quiet, &[15.0], &speech), vec![0.0]);
    }
}
