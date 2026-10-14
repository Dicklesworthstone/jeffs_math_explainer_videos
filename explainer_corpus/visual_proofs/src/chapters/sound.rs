//! Video I's teaching sounds (storyboard, "Sound plan"): the countdown of a
//! quiz, the notes of pieces landing, the halving heard as a rhythm and the
//! held tonic of the guess nothing passes. Each is the sound of a listen
//! window, written at about the narration's level; the score re-levels it,
//! and every sound is under a 3 kHz top. Nothing here has been auditioned
//! yet: the score bead tunes them.
//!
//! The video's key is C major and its bed holds four chords made of the
//! major pentatonic alone (C D E G A, `score::identity`), so every pitch
//! below, drawn from that pentatonic, is at least a whole tone from every
//! tone of whichever chord the bed is on: nothing here can rub against it.
//! Every pitch is read from the identity, and the times from the model.
//!
//! **A countdown** is one marimba note a second, stepping down the
//! pentatonic, the last three always E, D, C (3̂ 2̂ 1̂): the last note is the
//! tonic pitch, and it sounds over a chord the sound itself holds under the
//! last three seconds (a bass and a few tones of a pad). That chord is a
//! different one at each quiz and is the tonic chord at the last quiz only
//! (14_4), so a countdown ends on the tonic's pitch but comes home to its
//! harmony once, and the motif's resolution is kept for 15_6.

use std::f64::consts::PI;
use std::path::PathBuf;

use explainer_score::filter::{Biquad, Kind};
use explainer_score::instr::{Instrument, Note, render};
use explainer_score::theory::Chord;
use explainer_score::{FS, Stereo};

use super::draw::fail;
use crate::model::{Q, QUIZZES, Quiz};

/// A sonification's integrated loudness as written (the narration's).
const LEVEL_LUFS: f64 = -23.0;
/// The fade every sound ends with, seconds.
const TAIL_S: f64 = 0.25;
/// Every sound's top, a 4th-order low-pass. The score's own 5.5 kHz top
/// still let a mallet's upper partials past 8 kHz at about -60 dB, which
/// its QA counts as clicks where the bed is quiet (Video F, 2026-10-13).
const TOP_HZ: f64 = 3_000.0;

/// How long the answer has the screen to itself after the countdown's last
/// second, before the next line (the last note, and the chord under it,
/// ring here).
pub const LAND_S: f64 = 1.0;

/// The chord a countdown sounds over is held for its last three seconds
/// and the answer's beat.
const CHORD_FROM_END: u32 = 2;
/// The pad chord's strength against the mallets' (velocity).
const CHORD_VELOCITY: f64 = 0.35;
/// The bass note's.
const BASS_VELOCITY: f64 = 0.5;
/// The pad's tones are placed from this pitch up (G3), the bass from C2.
const PAD_FLOOR: i32 = 55;
const BASS_FLOOR: i32 = 36;

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

// ----------------------------------------------------------------- pitches

/// The degrees a countdown steps down through, low to high: 1̂ 2̂ 3̂ 5̂ 6̂ of
/// the pentatonic and the same an octave up, and 1̂ 2̂ 3̂ 5̂ of the next, so
/// the longest countdown (8 s, nine notes) has them all.
const RUNG_DEGREES: [i32; 9] = [1, 2, 3, 5, 6, 8, 9, 10, 12];

/// The pitches of the pentatonic from the tonic an octave above the
/// identity's, low to high (MIDI).
pub fn rungs() -> Vec<f64> {
    let id = crate::score::identity();
    RUNG_DEGREES
        .iter()
        .map(|&d| f64::from(id.key.degree(d, id.tonic_midi + 12)))
        .collect()
}

/// The notes of a countdown of `seconds`, one a second from its first
/// second to the moment the answer lands: the lowest `seconds + 1` rungs,
/// walked down, so the last three are always 3̂ 2̂ 1̂ and the first note tells
/// the length.
pub fn countdown_pitches(seconds: u32) -> Vec<f64> {
    let rungs = rungs();
    let n = (seconds as usize + 1).min(rungs.len());
    rungs[..n].iter().rev().copied().collect()
}

/// The pitch piece `i` (from 0) of a landing sounds: the pentatonic climbed
/// from the tonic an octave below the identity's, a note a piece, five to
/// an octave (the seventeen pieces of the cold open end on the D above the
/// tonic's octave): a finished picture is a finished chord.
pub fn landing_pitch(i: usize) -> f64 {
    let id = crate::score::identity();
    const DEGREES: [i32; 5] = [1, 2, 3, 5, 6];
    f64::from(id.key.degree(DEGREES[i % 5], id.tonic_midi - 12 + 12 * (i / 5) as i32))
}

// ------------------------------------------------------------- the chords

/// The chords the last note of a countdown sounds over when it is not the
/// last quiz: no tonic chord among them, all drawn from the pentatonic, and
/// all different. (vi7, ii7sus4, Vsus4, vi, ii9sus4, V sus4 add 9, Isus
/// four-of-vi, ii sus4.)
fn off_tonic() -> Vec<Chord> {
    vec![
        Chord::new(9, &[0, 3, 7, 10]),
        Chord::dom7sus4(2),
        Chord::sus4(7),
        Chord::new(9, &[0, 3, 7]),
        Chord::new(2, &[0, 5, 7, 10, 14]),
        Chord::new(7, &[0, 5, 7, 14]),
        Chord::new(9, &[0, 5, 7]),
        Chord::new(2, &[0, 5, 7]),
    ]
}

/// The chord quiz `quiz` is counted down over: the tonic chord at the last
/// quiz of the video and a different off-tonic chord at every other.
pub fn landing_chord(quiz: &Quiz) -> Chord {
    let at = QUIZZES.iter().position(|q| q.line == quiz.line).unwrap_or(0);
    if at + 1 == QUIZZES.len() {
        crate::score::identity().progression[0].clone()
    } else {
        let all = off_tonic();
        all[at % all.len()].clone()
    }
}

/// A chord as notes (MIDI): its bass from [`BASS_FLOOR`] up, and its tones
/// from [`PAD_FLOOR`] up, lowest first.
fn voicing(chord: &Chord) -> (f64, Vec<f64>) {
    let bass = BASS_FLOOR + (chord.bass - BASS_FLOOR).rem_euclid(12);
    let mut upper: Vec<i32> = chord
        .pitch_classes()
        .iter()
        .map(|&pc| PAD_FLOOR + (pc - PAD_FLOOR).rem_euclid(12))
        .collect();
    upper.sort_unstable();
    upper.dedup();
    (f64::from(bass), upper.into_iter().map(f64::from).collect())
}

// -------------------------------------------------------------- the countdown

/// A quiz's countdown: one mallet note a second while the ring drains and
/// one as the answer lands, walking down the pentatonic; under the last
/// three the quiz's chord, a bass and a pad (see [`landing_chord`]).
pub fn countdown_buffer(quiz: &Quiz) -> Stereo {
    let seconds = quiz.seconds;
    let total = f64::from(seconds) + LAND_S;
    let mut out = Stereo::silence(frames(total + 4.0));
    let id = crate::score::identity();
    let pitches = countdown_pitches(seconds);
    let last = pitches.len() - 1;
    for (k, &pitch) in pitches.iter().enumerate() {
        let velocity = if k == last { 0.8 } else { 0.55 };
        let note = Note::new(pitch, velocity, 0.6, k as u64 + 1);
        out.mix_in(&render(id.palette.melody, &note), frames(k as f64), 1.0);
    }
    // the chord comes in under the third note from the end and rings on
    let from = f64::from(seconds.saturating_sub(CHORD_FROM_END));
    let (bass, upper) = voicing(&landing_chord(quiz));
    let held = total - from;
    let low = Note::new(bass, BASS_VELOCITY, held, 31);
    out.mix_in(&render(Instrument::Bass, &low), frames(from), 1.0);
    for (j, &p) in upper.iter().enumerate() {
        let note = Note::new(p, CHORD_VELOCITY, held, 41 + j as u64);
        out.mix_in(&render(Instrument::Pad { brightness: 0.25 }, &note), frames(from), 1.0);
    }
    finish(out, total)
}

/// [`countdown_buffer`] as a file the listen window carries.
pub fn countdown(quiz: &Quiz) -> fmn::Result<PathBuf> {
    wav(&format!("proofs_countdown_{}", quiz.line), &countdown_buffer(quiz))
}

// ------------------------------------------------------------ the landings

/// A note struck at `at` seconds into a sonification: its MIDI pitch and
/// its strength.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Strike {
    pub at: f64,
    pub pitch: f64,
    pub velocity: f64,
}

/// The sonification `label` of pieces landing: one mallet note a strike,
/// `total` seconds long (a listen window's length). Returns the WAV.
pub fn strikes(label: &str, notes: &[Strike], total: f64) -> fmn::Result<PathBuf> {
    let id = crate::score::identity();
    let mut out = Stereo::silence(frames(total + 4.0));
    for (k, s) in notes.iter().enumerate() {
        let note = Note::new(s.pitch, s.velocity, 0.6, k as u64 + 1);
        out.mix_in(&render(id.palette.melody, &note), frames(s.at), 1.0);
    }
    wav(label, &finish(out, total))
}

// -------------------------------------------------- the halving (10_3, 10_4)

/// Where the notes of the halving fall in a bar `seconds` long: the bar's
/// downbeat, then one note at each half of what is left (1/2, 3/4, 7/8 ...
/// of the bar, from the model's `Q::half_to`), stopping once the next would
/// be nearer the end than `MIN_GAP_S`. The bar's end, the downbeat the
/// halving never reaches, has no note.
pub fn halving_onsets(seconds: f64) -> Vec<f64> {
    /// The nearest a note may come to the end of the bar (s).
    const MIN_GAP_S: f64 = 0.12;
    let mut out = vec![0.0];
    for k in 1..=24u32 {
        let t = seconds * (1.0 - Q::half_to(k).to_f64());
        if seconds - t < MIN_GAP_S {
            break;
        }
        out.push(t);
    }
    out
}

/// The halving heard as rhythm: a window `seconds` long as one bar, a soft
/// note on its downbeat (the whole square) and one at each half of what is
/// left, each a step higher on the pentatonic (from the tonic an octave
/// above the identity's, as a countdown's), crowding toward the bar's end
/// and never reaching it: the gap that never closes. The window ends on the
/// silence where the downbeat would be.
pub fn halving_buffer(seconds: f64) -> Stereo {
    let id = crate::score::identity();
    let rungs = rungs();
    let mut out = Stereo::silence(frames(seconds + 4.0));
    for (k, &t) in halving_onsets(seconds).iter().enumerate() {
        let note = Note::new(rungs[k.min(rungs.len() - 1)], 0.5, 0.3, k as u64 + 1);
        out.mix_in(&render(id.palette.melody, &note), frames(t), 1.0);
    }
    finish(out, seconds)
}

/// [`halving_buffer`] as a file the listen window carries.
pub fn halving(seconds: f64) -> fmn::Result<PathBuf> {
    wav("proofs_halving", &halving_buffer(seconds))
}

// ------------------------------------------------- the guess of one (10_6)

/// The guess nothing passes holds the tonic: the tonic chord (the bed's
/// first, 6/9 on C) as one soft bloom, bass first and the tones above it
/// arriving a tenth of a second apart, ringing for `seconds`.
pub fn tonic_hold_buffer(seconds: f64) -> Stereo {
    let id = crate::score::identity();
    let (bass, upper) = voicing(&id.progression[0]);
    let mut out = Stereo::silence(frames(seconds + 4.0));
    let low = Note::new(bass, BASS_VELOCITY, seconds, 51);
    out.mix_in(&render(Instrument::Bass, &low), 0, 1.0);
    for (j, &p) in upper.iter().enumerate() {
        let note = Note::new(p, CHORD_VELOCITY + 0.1, seconds, 61 + j as u64);
        out.mix_in(
            &render(Instrument::Pad { brightness: 0.25 }, &note),
            frames(0.1 * (j + 1) as f64),
            1.0,
        );
    }
    // the tonic itself, struck above them
    let top = Note::new(f64::from(id.tonic_midi + 12), 0.6, 0.6, 71);
    out.mix_in(&render(id.palette.melody, &top), frames(0.5), 1.0);
    finish(out, seconds)
}

/// [`tonic_hold_buffer`] as a file the listen window carries.
pub fn tonic_hold(seconds: f64) -> fmn::Result<PathBuf> {
    wav("proofs_tonic_hold", &tonic_hold_buffer(seconds))
}

#[cfg(test)]
mod tests {
    use explainer_score::fft::welch;
    use explainer_score::score::clashes;

    use super::*;

    /// Every countdown the storyboard lists is a quiz, in order and for its
    /// seconds, and each has a sound of its own: nine countdown rows, nine
    /// quizzes, nine chords (eight off the tonic and the tonic's at the last).
    #[test]
    fn every_countdown_row_of_the_storyboard_has_its_quiz_and_its_chord() {
        let board = explainer_kit::storyboard::parse(crate::STORYBOARD).unwrap();
        let rows: Vec<(String, f64)> = board
            .iter()
            .flat_map(|c| &c.beats)
            .filter(|b| b.hold.contains("countdown"))
            .map(|b| (b.id.clone(), b.listen_s().expect("a countdown row names its seconds")))
            .collect();
        let quizzes: Vec<(String, f64)> = QUIZZES
            .iter()
            .map(|q| (q.line.to_owned(), f64::from(q.seconds)))
            .collect();
        assert_eq!(rows, quizzes);
        assert_eq!(rows.len(), 9);
        let chords: Vec<Chord> = QUIZZES.iter().map(landing_chord).collect();
        assert_eq!(chords.len(), rows.len());
        assert_eq!(off_tonic().len(), rows.len() - 1);
    }

    fn peak_between(s: &Stereo, from: f64, to: f64) -> f64 {
        let (a, b) = (frames(from), frames(to).min(s.len()));
        s.l[a..b].iter().chain(&s.r[a..b]).fold(0.0_f64, |m, x| m.max(x.abs()))
    }

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

    #[test]
    fn every_pitch_is_the_pentatonic_and_clear_of_every_chord_of_the_bed() {
        let id = crate::score::identity();
        let pentatonic = [0, 2, 4, 7, 9];
        let mut all: Vec<f64> = rungs();
        all.extend((0..15).map(landing_pitch));
        for p in all {
            let pc = (p as i32).rem_euclid(12);
            assert!(pentatonic.contains(&pc), "{p} is not the pentatonic");
            for c in &id.progression {
                assert!(!clashes(p as i32, c), "{p} is a semitone from a tone of {c:?}");
            }
        }
        // the chords the sound holds are the pentatonic too, with no semitone inside
        let mut chords = off_tonic();
        chords.push(id.progression[0].clone());
        for c in &chords {
            let pcs = c.pitch_classes();
            assert!(pcs.len() >= 3, "{c:?}");
            assert!(pcs.iter().all(|pc| pentatonic.contains(pc)), "{c:?}");
            for (i, &a) in pcs.iter().enumerate() {
                for &b in &pcs[i + 1..] {
                    assert!(![1, 11].contains(&(a - b).rem_euclid(12)), "{c:?}");
                }
            }
        }
    }

    #[test]
    fn a_countdown_steps_down_and_always_ends_e_d_c() {
        let id = crate::score::identity();
        let tonic = f64::from(id.tonic_midi + 12);
        for q in QUIZZES {
            let p = countdown_pitches(q.seconds);
            assert_eq!(
                p.len(),
                q.seconds as usize + 1,
                "{}: a note a second and one more",
                q.line
            );
            assert!(p.windows(2).all(|w| w[0] > w[1]), "{}: it only steps down", q.line);
            // the last three always 3̂ 2̂ 1̂: E, D, C, and the last is the tonic pitch
            assert_eq!(p[p.len() - 3..], [tonic + 4.0, tonic + 2.0, tonic], "{}", q.line);
        }
        // the first note tells the length
        let firsts: Vec<f64> = (4..=8).map(|s| countdown_pitches(s)[0]).collect();
        assert!(firsts.windows(2).all(|w| w[0] < w[1]), "{firsts:?}");
        // the longest quiz has all nine rungs
        assert_eq!(QUIZZES.iter().map(|q| q.seconds).max(), Some(8));
        assert_eq!(countdown_pitches(8).len(), rungs().len());
    }

    #[test]
    fn the_last_note_is_over_the_tonic_chord_only_at_the_last_quiz() {
        let id = crate::score::identity();
        let tonic = &id.progression[0];
        let last = QUIZZES.last().unwrap();
        assert_eq!(last.line, "14_4", "the last quiz of the video");
        assert_eq!(&landing_chord(last), tonic);
        let mut seen: Vec<Chord> = Vec::new();
        for q in &QUIZZES[..QUIZZES.len() - 1] {
            let c = landing_chord(q);
            assert_ne!(&c, tonic, "{}: the tonic chord before the last quiz", q.line);
            assert_ne!(c.root, id.key.tonic, "{}: a chord on the tonic", q.line);
            assert!(!seen.contains(&c), "{}: the same chord as an earlier quiz", q.line);
            // the last three notes are not a semitone from it
            for &p in &countdown_pitches(q.seconds)[q.seconds as usize - 2..] {
                assert!(!clashes(p as i32, &c), "{}: {p} over {c:?}", q.line);
            }
            seen.push(c);
        }
        // eight quizzes and eight different chords
        assert_eq!(seen.len(), 8);
    }

    #[test]
    fn a_countdown_is_the_ring_and_the_answers_beat_under_the_top() {
        for q in QUIZZES {
            let s = countdown_buffer(q);
            let total = f64::from(q.seconds) + LAND_S;
            assert_eq!(s.len(), frames(total), "{}: the window's length", q.line);
            assert!(s.is_finite());
            let lufs = explainer_score::meter::integrated(&s);
            assert!((lufs - LEVEL_LUFS).abs() < 0.2, "{}: {lufs} LUFS", q.line);
            assert_eq!((s.l[s.len() - 1], s.r[s.len() - 1]), (0.0, 0.0));
            for k in 0..=q.seconds {
                let from = f64::from(k);
                assert!(peak_between(&s, from, from + 0.3) > 0.01, "{}: silent at {k} s", q.line);
            }
            assert!(energy_above(&s, 4_000.0) < 1e-4, "{}: energy above 4 kHz", q.line);
        }
    }

    #[test]
    fn the_landings_climb_the_pentatonic_a_note_a_piece() {
        let id = crate::score::identity();
        let tonic = f64::from(id.tonic_midi - 12);
        // the cold open's seventeen pieces, the most there are
        let p: Vec<f64> = (0..17).map(landing_pitch).collect();
        assert_eq!(p[0], tonic, "the first piece lands on the tonic");
        assert!(p.windows(2).all(|w| w[1] > w[0]), "{p:?}");
        assert_eq!(p[5] - p[0], 12.0, "five to an octave");
        // and the last is under the 3 kHz top the sounds are written under
        assert!(explainer_score::units::midi_to_hz(p[16]) < TOP_HZ, "{}", p[16]);
        let lands = strikes(
            "proofs_test_landings",
            &[
                Strike {
                    at: 0.1,
                    pitch: p[0],
                    velocity: 0.55,
                },
                Strike {
                    at: 0.6,
                    pitch: p[1],
                    velocity: 0.8,
                },
            ],
            1.5,
        )
        .unwrap();
        assert!(lands.exists());
    }

    /// The halving is the model's: the notes sit at one minus the model's
    /// 1/2^k of the bar, the next is always half the bar that is left, and
    /// none ever reaches the end.
    #[test]
    fn the_halving_is_heard_as_rhythm_and_never_reaches_the_downbeat() {
        let bar = 4.0;
        let t = halving_onsets(bar);
        assert_eq!(t[0], 0.0);
        assert!((t[1] - bar / 2.0).abs() < 1e-12);
        assert!((t[2] - bar * 3.0 / 4.0).abs() < 1e-12);
        for w in t.windows(3) {
            let (left_a, left_b) = (bar - w[1], bar - w[2]);
            if w[1] > 0.0 {
                assert!(
                    (left_b - left_a / 2.0).abs() < 1e-9,
                    "each note half the bar left: {t:?}"
                );
            }
        }
        assert!(t.iter().all(|&x| x < bar - 0.12 + 1e-9), "{t:?}");
        assert!(t.len() >= 5, "{t:?}");
        // rendered: a note at each onset, silence where the downbeat would be
        let s = halving_buffer(bar);
        assert_eq!(s.len(), frames(bar));
        assert!(s.is_finite());
        let lufs = explainer_score::meter::integrated(&s);
        assert!((lufs - LEVEL_LUFS).abs() < 0.2, "{lufs} LUFS");
        for &x in &t {
            assert!(peak_between(&s, x, x + 0.05) > 0.01, "no note at {x} s");
        }
        assert!(energy_above(&s, 4_000.0) < 1e-4);
    }

    #[test]
    fn the_guess_of_one_holds_the_tonic_chord() {
        let id = crate::score::identity();
        let (bass, upper) = voicing(&id.progression[0]);
        assert_eq!((bass as i32).rem_euclid(12), id.key.tonic, "the bass is the tonic");
        assert!(upper.iter().all(|&p| p >= f64::from(PAD_FLOOR)));
        let s = tonic_hold_buffer(2.5);
        assert_eq!(s.len(), frames(2.5));
        assert!(s.is_finite());
        let lufs = explainer_score::meter::integrated(&s);
        assert!((lufs - LEVEL_LUFS).abs() < 0.2, "{lufs} LUFS");
        assert_eq!((s.l[s.len() - 1], s.r[s.len() - 1]), (0.0, 0.0));
        assert!(energy_above(&s, 4_000.0) < 1e-4);
    }
}
