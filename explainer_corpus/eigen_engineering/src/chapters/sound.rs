//! Video K's teaching sounds (storyboard, "sound that teaches"): the
//! countdown of a quiz, the three modes of the truss as tones in their true
//! ratios (07_3), a hum sliding through the bathroom's tones (09_5) and the
//! same hum in the hall (10_6). Each is the sound of a listen window,
//! written at about the narration's level; the score re-levels it, and every
//! sound is under a 3 kHz top. Nothing here has been auditioned yet: the
//! score bead tunes them.
//!
//! The countdown is one soft vibraphone note a second climbing the overtones
//! of one note, the identity's motif (1̂ 5̂ 1̂ 3̂ of E major) and its two
//! lower notes, left open on the third while the ring drains; as it reaches
//! zero the identity's resolution walks back 5̂ 3̂ 1̂ on the plucked answer
//! and rings alone before the next line. Every pitch is read from the
//! identity (`score::identity`).

use std::f64::consts::{PI, TAU};
use std::path::PathBuf;

use explainer_score::filter::{Biquad, Kind};
use explainer_score::instr::{Note, render};
use explainer_score::sonify::motif;
use explainer_score::units::midi_to_hz;
use explainer_score::{FS, Stereo};

use super::draw::fail;

/// A sonification's integrated loudness as written (the narration's).
const LEVEL_LUFS: f64 = -23.0;
/// The fade every sound ends with, seconds.
const TAIL_S: f64 = 0.25;
/// Every sound's top, a 4th-order low-pass. The score's own 5.5 kHz top
/// still let a mallet's upper partials past 8 kHz at about -60 dB, which
/// its QA counts as clicks where the bed is quiet (Video F, 2026-10-13).
const TOP_HZ: f64 = 3_000.0;

/// How long the answer has the screen to itself after the countdown's last
/// second, before the next line: the resolution's three notes at the
/// identity's tempo and the last one's ring.
pub const LAND_S: f64 = 1.2;
/// The plucked answer's gain over the vibraphone climb: a plucked string
/// peaks about a third lower than a struck bar at the same velocity, and the
/// answer has to land over the climb.
const ANSWER_GAIN: f64 = 1.8;

/// The listen windows of the three sonifications, seconds (the
/// storyboard's `listen 3 s`, `2 s` and `2 s`).
pub const BRIDGE_S: f64 = 3.0;
pub const ROOM_S: f64 = 2.0;
pub const HALL_S: f64 = 2.0;

fn frames(seconds: f64) -> usize {
    (seconds * FS).round() as usize
}

/// Take `s` under [`TOP_HZ`], cut it to `seconds`, fade its last [`TAIL_S`]
/// to an exact zero, and scale it to [`LEVEL_LUFS`].
fn finish(mut s: Stereo, seconds: f64) -> Stereo {
    for ch in [&mut s.l, &mut s.r] {
        let lp = || Biquad::new(Kind::LowPass { q: 0.707 }, TOP_HZ);
        let (mut a, mut b) = (lp(), lp());
        for v in ch.iter_mut() {
            *v = b.process(a.process(*v));
        }
    }
    let n = frames(seconds);
    s.l.resize(n, 0.0);
    s.r.resize(n, 0.0);
    let tail = frames(TAIL_S).min(n);
    for k in 0..tail {
        let g = 0.5 - 0.5 * (PI * k as f64 / tail as f64).cos();
        s.l[n - 1 - k] *= g;
        s.r[n - 1 - k] *= g;
    }
    let lufs = explainer_score::meter::integrated(&s);
    if lufs.is_finite() {
        s.scale(10f64.powf((LEVEL_LUFS - lufs) / 20.0));
    }
    s
}

/// One sound file is written at a time in a process: the kit writes through
/// a temporary name made from the process id, so two scenes of one process
/// (the chapter tests run as threads) writing the same countdown would
/// rename each other's file away. A render runs one chapter per process.
static WRITING: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn wav(label: &str, s: &Stereo) -> fmn::Result<PathBuf> {
    let _one_at_a_time = WRITING.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    explainer_kit::sound_file(label, s).map_err(|e| fail(format!("sonification {label}: {e}")))
}

// ----------------------------------------------------------- the countdown

/// The rungs a countdown climbs, low to high (MIDI): the motif's own notes
/// (1̂ 5̂ 1̂ 3̂ of E major, the overtones of one note) and, under them, its
/// first two an octave down, so a longer countdown starts lower.
pub fn rungs() -> Vec<f64> {
    let id = crate::score::identity();
    let m = id.motif.pitches(&id.key, id.tonic_midi);
    let mut out = vec![f64::from(m[0] - 12), f64::from(m[1] - 12)];
    out.extend(m.iter().map(|&p| f64::from(p)));
    out
}

/// The notes of a countdown of `seconds`, one a second from its first
/// second to its last: the last `seconds` rungs, so the climb always ends on
/// the motif's third and the first note tells the length.
pub fn countdown_pitches(seconds: u32) -> Vec<f64> {
    let rungs = rungs();
    let n = (seconds as usize).min(rungs.len());
    rungs[rungs.len() - n..].to_vec()
}

/// A quiz's countdown: one note a second while the ring drains, then, as it
/// reaches zero, the resolution on the answer instrument, ringing for
/// [`LAND_S`].
pub fn countdown_buffer(seconds: u32) -> Stereo {
    let total = f64::from(seconds) + LAND_S;
    let mut out = Stereo::silence(frames(total + 4.0));
    let id = crate::score::identity();
    for (k, &pitch) in countdown_pitches(seconds).iter().enumerate() {
        let note = Note::new(pitch, 0.55, 0.6, k as u64 + 1);
        out.mix_in(&render(id.palette.melody, &note), frames(k as f64), 1.0);
    }
    let (answer, _) = motif(&id, &id.resolution, id.palette.answer, 0, 0.8, 21);
    out.mix_in(&answer, frames(f64::from(seconds)), ANSWER_GAIN);
    finish(out, total)
}

/// [`countdown_buffer`] as a file the listen window carries.
pub fn countdown(seconds: u32) -> fmn::Result<PathBuf> {
    wav(&format!("k_countdown_{seconds}"), &countdown_buffer(seconds))
}

// ------------------------------------------------- the bridge's modes (07_3)

/// How long each mode sounds alone, and the gap between them (seconds):
/// three tones one at a time, then all three together to the window's end.
pub const BRIDGE_SOLO_S: f64 = 0.5;
pub const BRIDGE_STEP_S: f64 = 0.55;
/// Where the three sound together.
pub const BRIDGE_TOGETHER_AT: f64 = 1.7;

/// The three tones of the truss's modes in their true ratios: the lowest
/// mode on the tonic an octave below the motif's (E3), the others at the
/// lowest's multiples the modes' frequencies make (`hertz[k] / hertz[0]`,
/// 1 : 1.99 : 2.51 for the video's bridge), unsnapped.
pub fn bridge_tones(hertz: &[f64]) -> [f64; 3] {
    let id = crate::score::identity();
    let base = midi_to_hz(f64::from(id.tonic_midi - 12));
    [base, base * hertz[1] / hertz[0], base * hertz[2] / hertz[0]]
}

/// A frequency as a (fractional) MIDI pitch.
fn midi_of(hz: f64) -> f64 {
    69.0 + 12.0 * (hz / 440.0).log2()
}

/// The three modes as struck tones: one at a time, then together.
pub fn bridge_buffer(hertz: &[f64]) -> Stereo {
    let id = crate::score::identity();
    let tones = bridge_tones(hertz);
    let mut out = Stereo::silence(frames(BRIDGE_S + 4.0));
    for (k, &hz) in tones.iter().enumerate() {
        let note = Note::new(midi_of(hz), 0.6, BRIDGE_SOLO_S, k as u64 + 1);
        out.mix_in(&render(id.palette.melody, &note), frames(BRIDGE_STEP_S * k as f64), 1.0);
        let together = Note::new(midi_of(hz), 0.5, BRIDGE_S - BRIDGE_TOGETHER_AT, k as u64 + 11);
        out.mix_in(&render(id.palette.melody, &together), frames(BRIDGE_TOGETHER_AT), 1.0);
    }
    finish(out, BRIDGE_S)
}

/// [`bridge_buffer`] as a file the listen window carries.
pub fn bridge(hertz: &[f64]) -> fmn::Result<PathBuf> {
    wav("k_bridge_modes", &bridge_buffer(hertz))
}

// --------------------------------------------- the room's and hall's hum

/// A hum's partials (harmonic, share of the fundamental): a voice's, with
/// its first five harmonics, so a pitch of 60 hertz can be heard at all.
const HUM_PARTIALS: [(f64, f64); 5] = [(1.0, 1.0), (2.0, 0.6), (3.0, 0.35), (4.0, 0.2), (5.0, 0.1)];
/// How loud the hum is between tones, as a share of its loudness on one:
/// the room answers louder at its tones, and the hum is still heard off them.
const HUM_FLOOR: f64 = 0.3;
/// How long the hum takes to come in (s).
const HUM_ONSET_S: f64 = 0.04;
/// The hum's level is read this often and joined by straight lines (s).
const HUM_CONTROL_S: f64 = 0.01;
/// How long the hall's hum slides before it stops and its tail runs on (s).
pub const HALL_SLIDE_S: f64 = 1.4;

/// How near `hz` is to the nearest of the sorted `tones`: 1 on one, falling
/// to 0 at `glow_hz` away (the same glow the picture's outline has).
fn near(tones: &[f64], hz: f64, glow_hz: f64) -> f64 {
    let i = tones.partition_point(|&t| t < hz);
    let below = i.checked_sub(1).map(|j| tones[j]);
    let above = tones.get(i).copied();
    let d = [below, above]
        .into_iter()
        .flatten()
        .map(|t| (hz - t).abs())
        .fold(f64::INFINITY, f64::min);
    (1.0 - d / glow_hz).max(0.0)
}

/// The pitch of a slide from `from` to `to` over `seconds`, `t` seconds in.
fn slide_hz(from: f64, to: f64, seconds: f64, t: f64) -> f64 {
    from + (to - from) * (t / seconds).clamp(0.0, 1.0)
}

/// A hum: instantaneous pitch `pitch(t)` and level `level(t)` (0..1) over
/// `seconds`, as a stereo buffer.
fn hum(seconds: f64, pitch: impl Fn(f64) -> f64, level: impl Fn(f64) -> f64) -> Stereo {
    let n = frames(seconds);
    let control = frames(HUM_CONTROL_S).max(1);
    let points: Vec<f64> = (0..=(n / control) + 1)
        .map(|i| level(i as f64 * HUM_CONTROL_S))
        .collect();
    let norm: f64 = HUM_PARTIALS.iter().map(|p| p.1).sum();
    let onset = frames(HUM_ONSET_S).max(1);
    let mut phase = 0.0_f64;
    let mono: Vec<f64> = (0..n)
        .map(|i| {
            let t = i as f64 / FS;
            phase = (phase + TAU * pitch(t) / FS) % TAU;
            let (p, frac) = (i / control, (i % control) as f64 / control as f64);
            let g = points[p] * (1.0 - frac) + points[p + 1] * frac;
            let rise = if i < onset {
                0.5 - 0.5 * (PI * i as f64 / onset as f64).cos()
            } else {
                1.0
            };
            let y: f64 = HUM_PARTIALS.iter().map(|&(h, a)| a * (h * phase).sin()).sum();
            0.5 * rise * g * y / norm
        })
        .collect();
    Stereo::from_mono(&mono)
}

/// The bathroom's hum (09_5): a slide from `from_hz` to `to_hz` over
/// [`ROOM_S`] seconds, loud where the room answers: at each of its `tones`
/// (sorted, Hz) the level rises to its full over `glow_hz` on each side,
/// the glow the picture's outline has, and falls to [`HUM_FLOOR`] between.
pub fn room_hum_buffer(from_hz: f64, to_hz: f64, tones: &[f64], glow_hz: f64) -> Stereo {
    let pitch = move |t: f64| slide_hz(from_hz, to_hz, ROOM_S, t);
    let level = |t: f64| HUM_FLOOR + (1.0 - HUM_FLOOR) * near(tones, pitch(t), glow_hz);
    finish(hum(ROOM_S, pitch, level), ROOM_S)
}

/// [`room_hum_buffer`] as a file the listen window carries.
pub fn room_hum(from_hz: f64, to_hz: f64, tones: &[f64], glow_hz: f64) -> fmn::Result<PathBuf> {
    wav("k_room_hum", &room_hum_buffer(from_hz, to_hz, tones, glow_hz))
}

/// The hall's hum (10_6): the same slide, over [`HALL_SLIDE_S`] seconds,
/// then the hum stops and its tail dies away at the hall's own pace, 60 dB
/// in `t60` seconds. The hall's `modes` (sorted, Hz) are so many in every
/// hertz that the room is always on one: the level has no peaks, and the
/// tail is even, with no answer.
pub fn hall_hum_buffer(from_hz: f64, to_hz: f64, modes: &[f64], glow_hz: f64, t60: f64) -> Stereo {
    let pitch = move |t: f64| slide_hz(from_hz, to_hz, HALL_SLIDE_S, t);
    let level = |t: f64| {
        let on = HUM_FLOOR + (1.0 - HUM_FLOOR) * near(modes, pitch(t), glow_hz);
        let after = (t - HALL_SLIDE_S).max(0.0);
        on * 10f64.powf(-3.0 * after / t60)
    };
    finish(hum(HALL_S, pitch, level), HALL_S)
}

/// [`hall_hum_buffer`] as a file the listen window carries.
pub fn hall_hum(from_hz: f64, to_hz: f64, modes: &[f64], glow_hz: f64, t60: f64) -> fmn::Result<PathBuf> {
    wav("k_hall_hum", &hall_hum_buffer(from_hz, to_hz, modes, glow_hz, t60))
}

#[cfg(test)]
mod tests {
    use explainer_score::fft::welch;

    use super::super::draw::numbers;
    use super::*;
    use crate::model::rooms::{BATHROOM, HALL, HALL_T60_S};
    use crate::model::truss::{DECK_KG, Truss};

    /// The most seconds a countdown can have, one a rung.
    const MOST_SECONDS: u32 = 6;
    /// The picture's glow width (09_5's `GLOW_HZ`).
    const GLOW_HZ: f64 = 1.3;

    /// The pitches the answer winds down: the identity's resolution (5̂ 3̂ 1̂).
    fn resolution_pitches() -> Vec<f64> {
        let id = crate::score::identity();
        id.resolution
            .pitches(&id.key, id.tonic_midi)
            .into_iter()
            .map(f64::from)
            .collect()
    }

    fn peak_between(s: &Stereo, from: f64, to: f64) -> f64 {
        let (a, b) = (frames(from), frames(to).min(s.len()));
        s.l[a..b].iter().chain(&s.r[a..b]).fold(0.0_f64, |m, x| m.max(x.abs()))
    }

    /// The root mean square of `s` (left channel) between `from` and `to`.
    fn rms_between(s: &Stereo, from: f64, to: f64) -> f64 {
        let (a, b) = (frames(from), frames(to).min(s.len()));
        (s.l[a..b].iter().map(|x| x * x).sum::<f64>() / (b - a) as f64).sqrt()
    }

    /// The share of `s`'s energy above `hz`.
    fn energy_above(s: &Stereo, hz: f64) -> f64 {
        let mono: Vec<f64> = s.l.iter().zip(&s.r).map(|(l, r)| 0.5 * (l + r)).collect();
        let (df, psd) = welch(&mono, FS, 4096);
        let total: f64 = psd.iter().sum();
        let above: f64 = psd
            .iter()
            .enumerate()
            .filter(|(i, _)| *i as f64 * df > hz)
            .map(|(_, p)| p)
            .sum();
        above / total
    }

    /// The strength of the tone at `hz` in the left channel of `s` between
    /// `from` and `to` seconds: the magnitude of its Fourier coefficient.
    fn strength(s: &Stereo, hz: f64, from: f64, to: f64) -> f64 {
        let (a, b) = (frames(from), frames(to).min(s.len()));
        let (mut re, mut im) = (0.0, 0.0);
        for (i, x) in s.l[a..b].iter().enumerate() {
            let t = (a + i) as f64 / FS;
            re += x * (TAU * hz * t).cos();
            im += x * (TAU * hz * t).sin();
        }
        (re * re + im * im).sqrt() / (b - a) as f64
    }

    fn bridge_hertz() -> Vec<f64> {
        Truss::pratt().hertz(DECK_KG)
    }

    #[test]
    fn the_climb_is_the_overtones_of_one_note_and_is_left_on_the_third() {
        let id = crate::score::identity();
        let motif: Vec<f64> = id
            .motif
            .pitches(&id.key, id.tonic_midi)
            .into_iter()
            .map(f64::from)
            .collect();
        // the motif is 1̂ 5̂ 1̂ 3̂ of E major: E4 B4 E5 G#5
        assert_eq!(motif, vec![64.0, 71.0, 76.0, 80.0]);
        for seconds in 4..=MOST_SECONDS {
            let p = countdown_pitches(seconds);
            assert_eq!(p.len(), seconds as usize, "one note a second");
            assert!(p.windows(2).all(|w| w[0] < w[1]), "it only climbs: {p:?}");
            assert_eq!(&p[p.len() - 4..], &motif[..], "the last four notes are the motif");
            // every rung is the key's 1̂, 5̂ or 3̂: an overtone of the tonic
            assert!(
                p.iter().all(|&n| [4, 8, 11].contains(&(n as i32).rem_euclid(12))),
                "{p:?} leaves E, G# and B"
            );
            // left open on the third, not the tonic
            assert_eq!((p[p.len() - 1] as i32).rem_euclid(12), 8);
        }
        // the first note tells the length
        let first: Vec<f64> = (4..=6).map(|s| countdown_pitches(s)[0]).collect();
        assert_eq!(first, vec![64.0, 59.0, 52.0]);
        assert_eq!(rungs().len(), MOST_SECONDS as usize);
        assert_eq!(countdown_pitches(MOST_SECONDS + 3).len(), rungs().len());
    }

    #[test]
    fn the_answer_walks_back_to_the_tonic() {
        let id = crate::score::identity();
        let r = resolution_pitches();
        assert_eq!(r, vec![71.0, 68.0, 64.0], "5̂ 3̂ 1̂: B G# E");
        assert!(r.windows(2).all(|w| w[0] > w[1]), "it walks back");
        assert_eq!((r[r.len() - 1] as i32).rem_euclid(12), id.key.tonic);
    }

    #[test]
    fn every_quiz_has_a_countdown_the_rungs_can_play() {
        assert_eq!(crate::model::QUIZZES.len(), 7, "seven countdowns");
        for q in crate::model::QUIZZES {
            assert!((4..=MOST_SECONDS).contains(&q.seconds), "{}: {} s", q.line, q.seconds);
            assert_eq!(countdown_pitches(q.seconds).len(), q.seconds as usize);
        }
    }

    #[test]
    fn a_countdown_is_the_ring_and_the_answers_beat_under_the_top() {
        let id = crate::score::identity();
        for seconds in 4..=MOST_SECONDS {
            let s = countdown_buffer(seconds);
            let total = f64::from(seconds) + LAND_S;
            assert_eq!(s.len(), frames(total), "the window's length");
            assert!(s.is_finite());
            let lufs = explainer_score::meter::integrated(&s);
            assert!((lufs - LEVEL_LUFS).abs() < 0.2, "{lufs} LUFS");
            assert_eq!((s.l[s.len() - 1], s.r[s.len() - 1]), (0.0, 0.0));
            for k in 0..seconds {
                let from = f64::from(k);
                assert!(
                    peak_between(&s, from, from + 0.3) > 0.01,
                    "{seconds} s: silent at {k} s"
                );
            }
            for j in 0..3 {
                let at = f64::from(seconds) + id.beat_s * 0.5 * f64::from(j);
                assert!(
                    peak_between(&s, at, at + 0.2) > 0.01,
                    "{seconds} s: no resolution note {j}"
                );
            }
            let answer = peak_between(&s, f64::from(seconds), total);
            let climb = peak_between(&s, 0.0, f64::from(seconds));
            assert!(answer > climb, "the answer {answer} is not over the climb {climb}");
            assert!(energy_above(&s, 4_000.0) < 1e-4, "{seconds} s: energy above 4 kHz");
        }
    }

    #[test]
    fn the_bridge_s_three_tones_are_its_modes_in_their_true_ratios() {
        let hz = bridge_hertz();
        let tones = bridge_tones(&hz);
        // the facts' own ratios, to their two places
        let (r2, r3) = (tones[1] / tones[0], tones[2] / tones[0]);
        assert_eq!(format!("{r2:.2}"), crate::chapters::draw::fact("K-V1.ratio.2").unwrap());
        assert_eq!(format!("{r3:.2}"), crate::chapters::draw::fact("K-V1.ratio.3").unwrap());
        assert!((tones[0] - midi_to_hz(52.0)).abs() < 1e-9, "the lowest is E3");
        // the numbers the narration says, to the tenth
        let said = numbers("K-V1.hz.1").unwrap()[0];
        assert!((hz[0] - said).abs() < 0.05);
        // rendered: each mode is the strongest of its three neighbours in its own
        // slot, and all three sound together at the end
        let s = bridge_buffer(&hz);
        assert_eq!(s.len(), frames(BRIDGE_S));
        assert!(s.is_finite());
        assert!(energy_above(&s, 4_000.0) < 1e-4);
        for (k, &f) in tones.iter().enumerate() {
            let from = BRIDGE_STEP_S * k as f64 + 0.05;
            let to = BRIDGE_STEP_S * k as f64 + BRIDGE_SOLO_S;
            let on = strength(&s, f, from, to);
            for off in [0.94, 1.06] {
                assert!(
                    on > 2.0 * strength(&s, f * off, from, to),
                    "mode {} at {f:.1} Hz is not the strongest in its slot",
                    k + 1
                );
            }
        }
        let (a, b) = (BRIDGE_TOGETHER_AT + 0.1, BRIDGE_S - 0.3);
        let all: Vec<f64> = tones.iter().map(|&f| strength(&s, f, a, b)).collect();
        let floor = strength(&s, tones[0] * 1.5, a, b);
        assert!(all.iter().all(|&x| x > 2.0 * floor), "{all:?} against {floor}");
    }

    #[test]
    fn the_bathroom_answers_at_its_tones_and_the_hum_follows_the_slide() {
        let tones: Vec<f64> = BATHROOM.modes_below(150).iter().map(|m| m.1).collect();
        assert_eq!(tones.len(), 9, "K-R2.tones_to_150_hz");
        let (from, to) = (127.0, 150.0);
        let s = room_hum_buffer(from, to, &tones, GLOW_HZ);
        assert_eq!(s.len(), frames(ROOM_S));
        assert!(s.is_finite());
        let lufs = explainer_score::meter::integrated(&s);
        assert!((lufs - LEVEL_LUFS).abs() < 0.2, "{lufs} LUFS");
        assert_eq!((s.l[s.len() - 1], s.r[s.len() - 1]), (0.0, 0.0));
        // the slide crosses 137.2 and 142.9 Hz: the hum is louder there than
        // between them, by well over 6 dB
        let at = |hz: f64| (hz - from) / (to - from) * ROOM_S;
        let on = rms_between(&s, at(137.2) - 0.03, at(137.2) + 0.03);
        let between = rms_between(&s, at(140.0) - 0.03, at(140.0) + 0.03);
        assert!(on > 2.0 * between, "on a tone {on}, between {between}");
        // and the pitch is the slide's: the fundamental at the middle is
        // stronger than a tenth either side
        let mid = ROOM_S / 2.0;
        let f_mid = slide_hz(from, to, ROOM_S, mid);
        let here = strength(&s, f_mid, mid - 0.1, mid + 0.1);
        for off in [0.9, 1.1] {
            assert!(here > strength(&s, f_mid * off, mid - 0.1, mid + 0.1), "{off}");
        }
        assert!(energy_above(&s, 4_000.0) < 1e-4);
    }

    #[test]
    fn the_hall_never_answers_and_its_tail_is_even() {
        let modes: Vec<f64> = HALL.modes_below(150).iter().map(|m| m.1).collect();
        assert!(modes.len() > 1000, "{} modes below 150 Hz", modes.len());
        let mut sorted = modes.clone();
        sorted.sort_by(f64::total_cmp);
        // so many modes in every hertz that the hall is always on one
        for hz in (60..=150).step_by(3) {
            assert!(near(&sorted, f64::from(hz), GLOW_HZ) > 0.5, "{hz} Hz");
        }
        let s = hall_hum_buffer(60.0, 150.0, &sorted, GLOW_HZ, HALL_T60_S);
        assert_eq!(s.len(), frames(HALL_S));
        assert!(s.is_finite());
        let lufs = explainer_score::meter::integrated(&s);
        assert!((lufs - LEVEL_LUFS).abs() < 0.2, "{lufs} LUFS");
        assert_eq!((s.l[s.len() - 1], s.r[s.len() - 1]), (0.0, 0.0));
        // no peaks through the slide: every 100 ms window within 3 dB of the loudest
        let levels: Vec<f64> = (2..13)
            .map(|k| {
                let t = 0.1 * f64::from(k);
                rms_between(&s, t, t + 0.1)
            })
            .collect();
        let (lo, hi) = levels
            .iter()
            .fold((f64::MAX, 0.0_f64), |(lo, hi), &x| (lo.min(x), hi.max(x)));
        // the hum's own harmonics make low pitches quieter to the meter, so
        // allow what the slide's rising fundamental does by itself: the test
        // is that nothing stands out, never that the slide is flat to a hair
        assert!(hi / lo < 4.0, "the slide's levels run {lo} to {hi}");
        let steps: Vec<f64> = levels.windows(2).map(|w| (w[1] / w[0]).max(w[0] / w[1])).collect();
        assert!(steps.iter().all(|&r| r < 1.6), "a step of more than 4 dB: {steps:?}");
        // the tail dies at the hall's pace: 60 dB in the decay time
        // (measured inside the stretch the closing fade has not reached)
        let (t0, t1) = (HALL_SLIDE_S + 0.05, HALL_SLIDE_S + 0.25);
        let drop_db = 20.0 * (rms_between(&s, t0, t0 + 0.05) / rms_between(&s, t1, t1 + 0.05)).log10();
        let want = 60.0 * (t1 - t0) / HALL_T60_S;
        assert!(
            (drop_db - want).abs() < 3.0,
            "{drop_db} dB in {:.1} s, wanted {want}",
            t1 - t0
        );
        assert!(energy_above(&s, 4_000.0) < 1e-4);
    }
}
