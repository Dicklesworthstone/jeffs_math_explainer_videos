//! Video F's teaching sounds (storyboard, "Sound plan"): the countdown of a
//! quiz, one mallet note a second; the one short plucked sound of the move;
//! the dry ticks of the machine at 09_3. Each is the sound of a listen
//! window, written at about the narration's level; the score re-levels it.
//! None has been auditioned yet: the score bead tunes them.
#![allow(dead_code, reason = "shared sounds land before the chapters that play them")]

use std::path::PathBuf;

use explainer_score::instr::{Instrument, Note, render, tick};
use explainer_score::{FS, Stereo};

use super::draw::fail;

/// A sonification's integrated loudness as written (the narration's).
const LEVEL_LUFS: f64 = -23.0;
/// The fade every sound ends with, seconds.
const TAIL_S: f64 = 0.25;

/// D, E, F sharp, A and B above the tonic, low to high: the only notes a
/// countdown plays.
const SCALE: [f64; 7] = [74.0, 76.0, 78.0, 81.0, 83.0, 86.0, 88.0];

/// How long the answer has the screen to itself after the countdown's last
/// second, before the next line (the last note rings here).
pub const LAND_S: f64 = 1.0;
/// The move: two seconds, the narrator silent.
pub const MOVE_S: f64 = 2.0;

fn frames(seconds: f64) -> usize {
    (seconds * FS).round() as usize
}

/// Cut `s` to `seconds`, fade its last [`TAIL_S`] to an exact zero, and
/// scale it to [`LEVEL_LUFS`].
fn finish(mut s: Stereo, seconds: f64) -> Stereo {
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

fn wav(label: &str, s: &Stereo) -> fmn::Result<PathBuf> {
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
    let mut out = Stereo::silence(frames(MOVE_S + 4.0));
    for (k, (pitch, at)) in [(62.0, 0.1), (69.0, 1.45)].into_iter().enumerate() {
        let note = Note::new(pitch, 0.7, 0.35, k as u64 + 11);
        out.mix_in(&render(Instrument::Pluck, &note), frames(at), 1.0);
    }
    wav("the_move", &finish(out, MOVE_S))
}

/// Dry ticks that slow to a stop: `times` are the seconds each one sounds.
pub fn ticks(times: &[f64], seconds: f64) -> fmn::Result<PathBuf> {
    let mut out = Stereo::silence(frames(seconds + 1.0));
    for (k, &at) in times.iter().enumerate() {
        let t = Stereo::from_mono(&tick(700.0, k as u64 + 3));
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

    #[test]
    fn every_quiz_has_a_countdown_in_the_scale() {
        for q in crate::model::QUIZZES {
            assert!((4..=6).contains(&q.seconds), "{}: {} s", q.line, q.seconds);
        }
    }
}
