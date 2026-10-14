//! Video F's teaching sounds (storyboard, "Sound plan"): the countdown of a
//! quiz, one mallet note a second; the one short plucked sound of the move;
//! the dry ticks of the machine at 09_3. Each is the sound of a listen
//! window, written at about the narration's level; the score re-levels it.
//! None has been auditioned yet: the score bead tunes them.
#![allow(dead_code, reason = "shared sounds land before the chapters that play them")]

use std::path::PathBuf;
use std::sync::Mutex;

use explainer_score::filter::{Biquad, Kind};
use explainer_score::instr::{Instrument, Note, render};
use explainer_score::{FS, Stereo};

use super::draw::fail;

/// A sonification's integrated loudness as written (the narration's).
const LEVEL_LUFS: f64 = -23.0;
/// The fade every sound ends with, seconds.
const TAIL_S: f64 = 0.25;
/// Every sound's top, a 4th-order low-pass. The score's own 5.5 kHz top
/// still let the mallet's upper partials and the ticks' edges past 8 kHz at
/// about -60 dB, which its QA counts as clicks where the bed is quiet (the
/// 46² countdown just after 11 opens, the machine at 09_3; 2026-10-13).
const TOP_HZ: f64 = 3_000.0;
/// The pitch of the machine's knock at 09_3.
const KNOCK_HZ: f64 = 400.0;

/// D, E, F sharp, A and B above the tonic, low to high: the only notes a
/// countdown plays.
const SCALE: [f64; 7] = [74.0, 76.0, 78.0, 81.0, 83.0, 86.0, 88.0];

/// How long the answer has the screen to itself after the countdown's last
/// second, before the next line (the last note rings here).
pub const LAND_S: f64 = 0.7;
/// The move: two seconds, the narrator silent.
pub const MOVE_S: f64 = 2.0;

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
        let g = 0.5 - 0.5 * (std::f64::consts::PI * k as f64 / tail as f64).cos();
        s.l[n - 1 - k] *= g;
        s.r[n - 1 - k] *= g;
    }
    let lufs = explainer_score::meter::integrated(&s);
    if lufs.is_finite() {
        s.scale(10f64.powf((LEVEL_LUFS - lufs) / 20.0));
    }
    s
}

/// `sound_file` names a file by its samples and renames a temporary of the
/// process's own into place, so two chapter tests (threads of one process)
/// that write the same sound rename each other's temporary away. A render is
/// one chapter per process and cannot hit it; the tests take this lock.
static WRITING: Mutex<()> = Mutex::new(());

fn wav(label: &str, s: &Stereo) -> fmn::Result<PathBuf> {
    let _one_at_a_time = WRITING.lock().unwrap_or_else(|e| e.into_inner());
    explainer_kit::sound_file(label, s).map_err(|e| fail(format!("sonification {label}: {e}")))
}

/// The notes of a countdown of `seconds`, one a second from its first
/// second to the moment the answer lands: they walk down the scale and end
/// F sharp, E, D, so the last three seconds always sound the same and the
/// first note tells the length (4 s starts on B, 5 s on the upper D, 6 s on
/// the upper E).
pub fn countdown_pitches(seconds: u32) -> Vec<f64> {
    let n = (seconds as usize + 1).min(SCALE.len());
    SCALE[..n].iter().rev().copied().collect()
}

/// A quiz's countdown: one mallet note a second, and the last one as the
/// answer lands; with `lands` false the last note is left out (13_1's ring
/// empties into silence).
pub fn countdown(seconds: u32, lands: bool) -> fmn::Result<PathBuf> {
    let total = f64::from(seconds) + LAND_S;
    let mut out = Stereo::silence(frames(total + 4.0));
    let pitches = countdown_pitches(seconds);
    let last = pitches.len() - 1;
    for (k, &pitch) in pitches.iter().enumerate() {
        if k == last && !lands {
            break;
        }
        let velocity = if k == last { 0.8 } else { 0.55 };
        let note = Note::new(pitch, velocity, 0.6, k as u64 + 1);
        out.mix_in(&render(Instrument::Marimba, &note), frames(k as f64), 1.0);
    }
    let label = format!("countdown_{seconds}_{}", if lands { "lands" } else { "empty" });
    wav(&label, &finish(out, total))
}

/// The move's one short plucked sound: a low note as the strip lifts and
/// the fifth above as it stands.
pub fn the_move() -> fmn::Result<PathBuf> {
    wav("the_move", &the_move_audio())
}

fn the_move_audio() -> Stereo {
    let mut out = Stereo::silence(frames(MOVE_S + 4.0));
    for (k, (pitch, at)) in [(62.0, 0.1), (69.0, 1.45)].into_iter().enumerate() {
        let note = Note::new(pitch, 0.7, 0.35, k as u64 + 11);
        out.mix_in(&render(Instrument::Pluck, &note), frames(at), 1.0);
    }
    finish(out, MOVE_S)
}

/// The recall after 14_3's three sums: two seconds, the narrator silent.
pub const RECALL_S: f64 = 2.0;
/// When each of the recall's three plucks starts, seconds into its window:
/// the scene hops one square's strip at each (14_3).
pub const RECALL_AT: [f64; 3] = [0.1, 0.75, 1.4];
/// The pitch of the recall's plucks: the move's low note, the one that
/// sounds as a strip lifts.
const LIFT_MIDI: f64 = 62.0;

/// The recall's sound: [`the_move`]'s lifting note, the same pluck three
/// times, once for each of 14_3's three squares.
fn recall() -> Stereo {
    let mut out = Stereo::silence(frames(RECALL_S + 4.0));
    for &at in &RECALL_AT {
        let note = Note::new(LIFT_MIDI, 0.7, 0.35, 11);
        out.mix_in(&render(Instrument::Pluck, &note), frames(at), 1.0);
    }
    finish(out, RECALL_S)
}

/// The recall's WAV (see [`recall`]), for a listen window of [`RECALL_S`].
pub fn the_move_thrice() -> fmn::Result<PathBuf> {
    wav("the_move_thrice", &recall())
}

/// One dry knock of the machine: [`KNOCK_HZ`] dying in about 30 ms, over a
/// 3 ms raised-cosine rise. The score's `tick` (500 Hz at its lowest, with a
/// partial at 2.3 times that and a 1 ms edge) is the loudest sound in 09, so
/// QA's 8 kHz high-pass still caught each one as a click over the quiet bed
/// (2026-10-13).
fn knock() -> Vec<f64> {
    let rise = frames(0.003);
    (0..frames(0.05))
        .map(|i| {
            let t = i as f64 / FS;
            let g = if i < rise {
                0.5 - 0.5 * (std::f64::consts::PI * i as f64 / rise as f64).cos()
            } else {
                1.0
            };
            g * (-t / 0.008).exp() * (std::f64::consts::TAU * KNOCK_HZ * t).sin()
        })
        .collect()
}

/// Dry ticks that slow to a stop: `times` are the seconds each one sounds.
pub fn ticks(times: &[f64], seconds: f64) -> fmn::Result<PathBuf> {
    let mut out = Stereo::silence(frames(seconds + 1.0));
    let t = Stereo::from_mono(&knock());
    for &at in times {
        out.mix_in(&t, frames(at), 0.8);
    }
    wav("machine_ticks", &finish(out, seconds))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_countdown_ends_f_sharp_e_d_and_its_first_note_tells_its_length() {
        for (seconds, first) in [(4, 83.0), (5, 86.0), (6, 88.0)] {
            let p = countdown_pitches(seconds);
            assert_eq!(
                p.len(),
                seconds as usize + 1,
                "one note a second and one as the answer lands"
            );
            assert_eq!(p[0], first);
            assert_eq!(p[p.len() - 3..], [78.0, 76.0, 74.0]);
            assert!(p.windows(2).all(|w| w[0] > w[1]), "it only walks down");
        }
    }

    /// The seconds at which a pluck begins: the first 5 ms block in which the
    /// level is at least twice that of anything in the 20 ms before it, and a
    /// tenth of the sound's loudest (a pluck rings on, so a new one is heard
    /// as a rise over the old one's tail, not after silence).
    fn onsets(s: &Stereo) -> Vec<f64> {
        let block = frames(0.005);
        let level: Vec<f64> =
            s.l.chunks(block)
                .map(|c| (c.iter().map(|x| x * x).sum::<f64>() / c.len() as f64).sqrt())
                .collect();
        let top = level.iter().copied().fold(0.0, f64::max);
        let mut out: Vec<f64> = Vec::new();
        for i in 1..level.len() {
            let before = level[i.saturating_sub(4)..i].iter().copied().fold(0.0, f64::max);
            let t = i as f64 * 0.005;
            if level[i] > 0.1 * top && level[i] >= 2.0 * before && out.last().is_none_or(|&last| t - last > 0.2) {
                out.push(t);
            }
        }
        out
    }

    /// The recall has a pluck for each square, each where the scene hops its
    /// strip (`RECALL_AT`, which 14_3 reads); it is written the way every
    /// sound of the video is: -23 LUFS, nothing above 3 kHz that the score's
    /// QA could count as a click, an exact zero for its last sample.
    #[test]
    fn the_recall_has_three_plucks_where_the_scene_hops_a_strip() {
        let s = recall();
        assert_eq!(s.len(), frames(RECALL_S));
        let found = onsets(&s);
        assert_eq!(found.len(), 3, "{found:?}");
        for (got, want) in found.iter().zip(RECALL_AT) {
            assert!(
                (got - want).abs() <= 0.01,
                "a pluck starts at {got} s, the scene hops at {want} s"
            );
        }
        assert!(
            RECALL_AT.windows(2).all(|w| w[1] - w[0] >= 0.6),
            "the plucks are far enough apart to hop one after another"
        );
        const {
            assert!(
                RECALL_AT[2] + 0.35 <= RECALL_S,
                "the last pluck rings out inside the window"
            )
        };
        let lufs = explainer_score::meter::integrated(&s);
        assert!((lufs - LEVEL_LUFS).abs() < 0.2, "{lufs} LUFS");
        assert_eq!((s.l[s.len() - 1], s.r[s.len() - 1]), (0.0, 0.0));
        // the score's QA counts what is left above 8 kHz as clicks where the bed
        // is quiet: the recall has no more of it than the move's own sound
        let above_8k = |x: &Stereo| {
            let (mut a, mut b) = (
                Biquad::new(Kind::HighPass { q: 0.707 }, 8_000.0),
                Biquad::new(Kind::HighPass { q: 0.707 }, 8_000.0),
            );
            let high: f64 = x.l.iter().map(|&v| b.process(a.process(v)).powi(2)).sum();
            10.0 * (high / x.l.iter().map(|v| v * v).sum::<f64>()).log10()
        };
        assert!(above_8k(&s) < -60.0, "{:.1} dB above 8 kHz", above_8k(&s));
        assert!(
            above_8k(&s) <= above_8k(&the_move_audio()) + 3.0,
            "more above 8 kHz than the move's own sound"
        );
    }

    /// Chapter tests are threads of one process, and two that write the same
    /// new sound at once rename each other's temporary away (ENOENT). A sound
    /// no run has written yet, from many threads at once, must still come out
    /// of `wav` for each of them.
    #[test]
    fn many_threads_writing_one_new_sound_all_get_its_file() {
        let s = std::sync::Arc::new(recall());
        for round in 0..3 {
            let label = format!("race_{}_{round}", std::process::id());
            let threads: Vec<_> = (0..16)
                .map(|_| {
                    let (s, label) = (std::sync::Arc::clone(&s), label.clone());
                    std::thread::spawn(move || wav(&label, &s).map(|p| p.exists()))
                })
                .collect();
            for t in threads {
                assert!(t.join().unwrap().unwrap(), "round {round}");
            }
        }
    }

    #[test]
    fn every_quiz_has_a_countdown_in_the_scale() {
        for q in crate::model::QUIZZES {
            assert!((4..=6).contains(&q.seconds), "{}: {} s", q.line, q.seconds);
        }
    }
}
