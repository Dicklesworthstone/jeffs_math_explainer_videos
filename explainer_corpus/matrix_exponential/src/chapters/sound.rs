//! Video J's teaching sounds (storyboard, "sound that teaches"): the
//! countdown of a quiz, one soft vibraphone note a second, each a step
//! above the last and the climb left unfinished until the answer lands (the
//! motif's rising steps, unresolved); the terms of e^{5J} at 08_4, one
//! marimba note each, as loud as its bar; and the two pairs of pitches of
//! 12_6, one pair settling and one trading places. Each is the sound of a
//! listen window, written at about the narration's level; the score
//! re-levels it. The identity has no listen chord, so the bed's progression
//! goes on under every window: every pitch is an A, a D or an E, the three
//! pitch classes no chord of the progression (A minor 9, C major 9, E minor
//! 9, D seventh sus 4) holds a semitone from, and every sound is under a
//! 3 kHz top. None has been auditioned yet: the score bead tunes them.

use std::f64::consts::{PI, TAU};
use std::path::PathBuf;

use explainer_score::filter::{Biquad, Kind};
use explainer_score::fx::pan;
use explainer_score::instr::{Instrument, Note, render};
use explainer_score::{FS, Stereo};

use super::draw::fail;
use super::heat_and_chance::pair_heat;
use crate::facts::quantum_pair;

/// A sonification's integrated loudness as written (the narration's).
const LEVEL_LUFS: f64 = -23.0;
/// The fade every sound ends with, seconds.
const TAIL_S: f64 = 0.25;
/// Every sound's top, a 4th-order low-pass. The score's own 5.5 kHz top
/// still let a mallet's upper partials past 8 kHz at about -60 dB, which
/// its QA counts as clicks where the bed is quiet (Video F, 2026-10-13).
const TOP_HZ: f64 = 3_000.0;

/// How long the answer has the screen to itself after the countdown's last
/// second, before the next line (the answer's note rings here).
pub const LAND_S: f64 = 0.7;

/// The rungs a countdown climbs, low to high (MIDI): A3, D4, E4, A4, D5 and
/// E5, a step and then a leap, again and again, each landing higher. The
/// top rung is E, the fifth, which leans on the A above it.
const LADDER: [f64; 6] = [57.0, 62.0, 64.0, 69.0, 74.0, 76.0];
/// Where the climb comes home when the answer lands: A5 (and A4 under it).
const HOME: f64 = 81.0;

/// The seconds between one term's note and the next at 08_4.
pub const TERM_STEP_S: f64 = 0.2;
/// How long a term's note sounds before it is let go (a mallet note is cut,
/// so one term's ring never sits under the next term's note).
const TERM_NOTE_S: f64 = 0.17;
/// The release that cut takes.
const TERM_RELEASE_S: f64 = 0.03;
/// The room after the last term's note before the window ends.
const TERM_TAIL_S: f64 = 0.6;
/// Every term sounds at this pitch (A4): only its loudness is the term's.
const TERM_PITCH: f64 = 69.0;
/// The note's strike: the same one for every term, so its size is the only
/// difference between two notes (a round-robin seed would vary it).
const TERM_SEED: u64 = 7;

/// 12_6's sweep: the tracker runs from 0 to [`SLOSH_END`] (the half turn
/// at which the quantum pair has gone across and back) over [`SLOSH_S`]
/// seconds; the picture and the sound read the same two numbers.
pub const SLOSH_END: f64 = PI;
pub const SLOSH_S: f64 = 6.0;
/// The tones go on this long after the sweep ends (held where it ended).
const SLOSH_TAIL_S: f64 = 0.4;
/// The listen window of 12_6.
pub const SLOSH_WINDOW_S: f64 = SLOSH_S + SLOSH_TAIL_S;
/// The pitches of the four bars of 12_6: the heat panel's two nodes (A4 and
/// E5, a fifth) on the left, the quantum panel's (D5 and A5, a fifth) on the
/// right.
const SLOSH_PITCHES: [f64; 4] = [69.0, 76.0, 74.0, 81.0];
/// Where each panel sounds in the stereo image (-1 left, +1 right).
const SLOSH_PAN: [f64; 4] = [-0.7, -0.7, 0.7, 0.7];
/// How long the tones take to come in at the window's start (s).
const SLOSH_ONSET_S: f64 = 0.04;
/// The levels of the four bars are read this often and joined by straight
/// lines (s): the envelope is smooth at this step.
const SLOSH_CONTROL_S: f64 = 0.01;

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

/// The notes of a countdown of `seconds`, one a second from its first
/// second to its last: each a rung above the one before, so the climb is
/// unfinished. They always end on E, whatever the length, and the first one
/// tells the length (4 s starts on E4, 5 s on D4, 6 s on A3).
pub fn countdown_pitches(seconds: u32) -> Vec<f64> {
    let n = (seconds as usize).min(LADDER.len());
    LADDER[LADDER.len() - n..].to_vec()
}

/// The pitches the answer lands on: A, an octave apart.
pub fn home_pitches() -> [f64; 2] {
    [HOME, HOME - 12.0]
}

/// A quiz's countdown: one note a second while the ring drains, then, as it
/// reaches zero, the answer's A over the E the climb left hanging, ringing
/// for [`LAND_S`].
pub fn countdown_buffer(seconds: u32) -> Stereo {
    let total = f64::from(seconds) + LAND_S;
    let mut out = Stereo::silence(frames(total + 4.0));
    for (k, &pitch) in countdown_pitches(seconds).iter().enumerate() {
        let note = Note::new(pitch, 0.55, 0.6, k as u64 + 1);
        out.mix_in(&render(Instrument::Vibraphone, &note), frames(k as f64), 1.0);
    }
    for (j, (pitch, velocity)) in home_pitches().into_iter().zip([0.8, 0.6]).enumerate() {
        let note = Note::new(pitch, velocity, 0.6, 21 + j as u64);
        out.mix_in(&render(Instrument::Vibraphone, &note), frames(f64::from(seconds)), 1.0);
    }
    finish(out, total)
}

/// [`countdown_buffer`] as a file the listen window carries.
pub fn countdown(seconds: u32) -> fmn::Result<PathBuf> {
    wav(&format!("j_countdown_{seconds}"), &countdown_buffer(seconds))
}

// ------------------------------------------------------ the terms (08_4)

/// How long the listen window of 08_4 runs for `count` terms.
pub fn terms_seconds(count: usize) -> f64 {
    TERM_STEP_S * count as f64 + TERM_TAIL_S
}

/// Each term's loudness as a share of the loudest: its bar's height in the
/// chart, so a note is as loud as its bar is tall.
pub fn term_gains(sizes: &[f64]) -> Vec<f64> {
    let peak = sizes.iter().copied().fold(0.0, f64::max);
    sizes.iter().map(|s| s / peak).collect()
}

/// The terms of the series, one note each [`TERM_STEP_S`] apart, each at the
/// gain of its bar.
pub fn terms_buffer(sizes: &[f64]) -> Stereo {
    let seconds = terms_seconds(sizes.len());
    let mut out = Stereo::silence(frames(seconds + 2.0));
    let note = Note::new(TERM_PITCH, 0.7, 0.1, TERM_SEED);
    let mut strike = render(Instrument::Marimba, &note);
    let keep = frames(TERM_NOTE_S).min(strike.len());
    let release = frames(TERM_RELEASE_S).min(keep);
    strike.l.truncate(keep);
    strike.r.truncate(keep);
    for j in 0..release {
        let g = 0.5 - 0.5 * (PI * j as f64 / release as f64).cos();
        strike.l[keep - 1 - j] *= g;
        strike.r[keep - 1 - j] *= g;
    }
    for (k, gain) in term_gains(sizes).into_iter().enumerate() {
        out.mix_in(&strike, frames(TERM_STEP_S * k as f64), gain);
    }
    finish(out, seconds)
}

/// [`terms_buffer`] as a file the listen window carries.
pub fn terms(sizes: &[f64]) -> fmn::Result<PathBuf> {
    wav("j_terms", &terms_buffer(sizes))
}

// ------------------------------------------------------ the pairs (12_6)

/// The four bars of 12_6 `seconds` into its sweep: the heat panel's two
/// nodes, then the quantum panel's. The tracker's value runs linearly from
/// 0 to [`SLOSH_END`] over [`SLOSH_S`] seconds, and stays there after.
pub fn slosh_levels(seconds: f64) -> [f64; 4] {
    let t = SLOSH_END * (seconds / SLOSH_S).clamp(0.0, 1.0);
    let (heat, chance) = (pair_heat(t), quantum_pair(t));
    [heat[0], heat[1], chance[0], chance[1]]
}

/// The pairs of 12_6: four sustained tones, one per bar, each as loud as
/// its bar is tall as the sweep runs, the heat panel's pair on the left and
/// the quantum panel's on the right. Heat's two tones come together and stay
/// together; the quantum pair's trade places, across at half the sweep and
/// back at its end.
pub fn slosh_buffer() -> Stereo {
    let seconds = SLOSH_WINDOW_S;
    let n = frames(seconds + 1.0);
    let control = frames(SLOSH_CONTROL_S);
    // the bars' levels at each control point, joined by straight lines
    let points: Vec<[f64; 4]> = (0..=(n / control) + 1)
        .map(|i| slosh_levels(i as f64 * SLOSH_CONTROL_S))
        .collect();
    let onset = frames(SLOSH_ONSET_S);
    let mut out = Stereo::silence(n);
    for (b, (&pitch, &at)) in SLOSH_PITCHES.iter().zip(&SLOSH_PAN).enumerate() {
        let hz = explainer_score::units::midi_to_hz(pitch);
        let tone: Vec<f64> = (0..n)
            .map(|i| {
                let (p, frac) = (i / control, (i % control) as f64 / control as f64);
                let level = points[p][b] * (1.0 - frac) + points[p + 1][b] * frac;
                let rise = if i < onset {
                    0.5 - 0.5 * (PI * i as f64 / onset as f64).cos()
                } else {
                    1.0
                };
                let t = i as f64 / FS;
                let y = (TAU * hz * t).sin() + 0.3 * (TAU * 2.0 * hz * t).sin() + 0.1 * (TAU * 3.0 * hz * t).sin();
                0.5 * rise * level * y
            })
            .collect();
        out.mix_in(&pan(&tone, at), 0, 1.0);
    }
    finish(out, seconds)
}

/// [`slosh_buffer`] as a file the listen window carries.
pub fn slosh() -> fmn::Result<PathBuf> {
    wav("j_slosh", &slosh_buffer())
}

#[cfg(test)]
mod tests {
    use explainer_score::fft::welch;

    use super::super::draw::numbers;
    use super::*;

    /// Whether `pitch` is a semitone from no tone of any chord the bed
    /// plays (the identity has no listen chord: its whole progression goes
    /// on under a sonification), and a tone of its key.
    fn in_scale(pitch: f64) -> bool {
        let id = crate::score::identity();
        let p = pitch as i32;
        id.key.pitch_classes().contains(&p.rem_euclid(12))
            && id.progression.iter().all(|c| !explainer_score::score::clashes(p, c))
    }

    /// The loudest sample of `s` (either channel) between `from` and `to`
    /// seconds.
    fn peak_between(s: &Stereo, from: f64, to: f64) -> f64 {
        let (a, b) = (frames(from), frames(to).min(s.len()));
        s.l[a..b].iter().chain(&s.r[a..b]).fold(0.0_f64, |m, x| m.max(x.abs()))
    }

    /// The strength of the tone at `pitch` in one channel `ch` between `from`
    /// and `to` seconds: the magnitude of its Fourier coefficient, per sample.
    fn strength(ch: &[f64], pitch: f64, from: f64, to: f64) -> f64 {
        let (a, b) = (frames(from), frames(to).min(ch.len()));
        let hz = explainer_score::units::midi_to_hz(pitch);
        let (mut re, mut im) = (0.0, 0.0);
        for (i, x) in ch[a..b].iter().enumerate() {
            let t = (a + i) as f64 / FS;
            re += x * (TAU * hz * t).cos();
            im += x * (TAU * hz * t).sin();
        }
        (re * re + im * im).sqrt() / (b - a) as f64
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

    /// The only pitch classes any sound here uses are the three no chord of
    /// the progression is a semitone from: the claim of the module's doc.
    #[test]
    fn the_three_pitch_classes_are_the_ones_no_chord_of_the_progression_brushes() {
        let id = crate::score::identity();
        assert!(
            id.listen_chord.is_none(),
            "a listen chord would hold its own chord instead"
        );
        let safe: Vec<i32> = id
            .key
            .pitch_classes()
            .into_iter()
            .filter(|&pc| id.progression.iter().all(|c| !explainer_score::score::clashes(pc, c)))
            .collect();
        let mut got = safe;
        got.sort_unstable();
        // D, E and A
        assert_eq!(got, vec![2, 4, 9]);
        for p in LADDER
            .iter()
            .chain(&[HOME, HOME - 12.0, TERM_PITCH])
            .chain(&SLOSH_PITCHES)
        {
            assert!(in_scale(*p), "{p} is a semitone from a chord of the bed");
        }
    }

    #[test]
    fn every_countdown_climbs_a_rung_a_second_and_is_left_on_the_e() {
        for seconds in 4..=6u32 {
            let p = countdown_pitches(seconds);
            assert_eq!(p.len(), seconds as usize, "one note a second");
            assert!(p.windows(2).all(|w| w[0] < w[1]), "it only climbs: {p:?}");
            assert!(p.iter().all(|&n| in_scale(n)), "{p:?} leaves the safe pitches");
            // unfinished: the last note is the fifth, not the tonic
            assert_eq!(p[p.len() - 1], 76.0);
            assert_ne!((p[p.len() - 1] as i32).rem_euclid(12), 9, "it ends on the tonic");
        }
        // the first note tells the length
        let first: Vec<f64> = (4..=6).map(|s| countdown_pitches(s)[0]).collect();
        assert_eq!(first, vec![64.0, 62.0, 57.0]);
        // and the answer comes home: an A, above the E it leaves
        let home = home_pitches();
        assert!(home.iter().all(|&n| (n as i32).rem_euclid(12) == 9 && in_scale(n)));
        assert!(home[0] > 76.0);
    }

    #[test]
    fn every_quiz_has_a_countdown_the_ladder_can_play() {
        for q in crate::model::QUIZZES {
            assert!((4..=6).contains(&q.seconds), "{}: {} s", q.line, q.seconds);
            assert_eq!(countdown_pitches(q.seconds).len(), q.seconds as usize);
        }
    }

    #[test]
    fn a_countdown_is_the_ring_and_the_answer_s_beat_under_the_top() {
        for seconds in 4..=6u32 {
            let s = countdown_buffer(seconds);
            let total = f64::from(seconds) + LAND_S;
            assert_eq!(s.len(), frames(total), "the window's length");
            assert!(s.is_finite());
            let lufs = explainer_score::meter::integrated(&s);
            assert!((lufs - LEVEL_LUFS).abs() < 0.2, "{lufs} LUFS");
            // exactly zero where the window ends
            assert_eq!((s.l[s.len() - 1], s.r[s.len() - 1]), (0.0, 0.0));
            // a note sounds in every second of the countdown and in the answer's beat
            for k in 0..seconds {
                let from = f64::from(k);
                assert!(
                    peak_between(&s, from, from + 0.3) > 0.01,
                    "{seconds} s: silent at {k} s"
                );
            }
            assert!(peak_between(&s, f64::from(seconds), f64::from(seconds) + 0.3) > 0.01);
            // the answer is the loudest note of the countdown
            let answer = peak_between(&s, f64::from(seconds), total);
            let climb = peak_between(&s, 0.0, f64::from(seconds));
            assert!(answer > climb, "the answer {answer} is not over the climb {climb}");
            assert!(energy_above(&s, 4_000.0) < 1e-4, "{seconds} s: energy above 4 kHz");
        }
    }

    #[test]
    fn each_term_is_as_loud_as_its_bar_is_tall() {
        let sizes = numbers("F-S.terms").unwrap();
        assert_eq!(sizes.len(), 21, "k = 0 to 20");
        let gains = term_gains(&sizes);
        assert!(gains.iter().all(|g| (0.0..=1.0).contains(g)));
        assert!(gains.iter().any(|&g| (g - 1.0).abs() < 1e-12));
        let s = terms_buffer(&sizes);
        assert_eq!(s.len(), frames(terms_seconds(sizes.len())));
        assert!(s.is_finite());
        let loudest = (0..sizes.len())
            .map(|k| peak_between(&s, TERM_STEP_S * k as f64, TERM_STEP_S * (k + 1) as f64))
            .fold(0.0, f64::max);
        for (k, &g) in gains.iter().enumerate() {
            let at = TERM_STEP_S * k as f64;
            let heard = peak_between(&s, at, at + TERM_STEP_S) / loudest;
            // the strike is the same everywhere and the filter is linear:
            // a note's peak is its bar's share of the tallest, to a hair
            assert!((heard - g).abs() < 0.01 + 0.02 * g, "k = {k}: heard {heard}, bar {g}");
        }
        // swelling, then collapsing: the notes rise to k = 4 and 5, then fall
        let heard: Vec<f64> = (0..sizes.len())
            .map(|k| peak_between(&s, TERM_STEP_S * k as f64, TERM_STEP_S * (k + 1) as f64))
            .collect();
        assert!(heard[..=4].windows(2).all(|w| w[0] < w[1]));
        assert!(heard[5..=14].windows(2).all(|w| w[0] > w[1]));
        // no note rings into the next term's slot: the last 20 ms of every
        // slot is under a hundredth of the note's own peak
        for (k, &peak) in heard.iter().enumerate().take(sizes.len() - 1) {
            let tail = peak_between(&s, TERM_STEP_S * (k + 1) as f64 - 0.02, TERM_STEP_S * (k + 1) as f64);
            assert!(tail < 0.01 * peak + 1e-9, "k = {k}: {tail} after a note of {peak}");
        }
        assert!(energy_above(&s, 4_000.0) < 1e-4);
    }

    #[test]
    fn the_pairs_of_12_6_are_the_facts_the_bars_show() {
        // the start: all the heat and all the chance on the first node
        assert_eq!(slosh_levels(0.0), [1.0, 0.0, 1.0, 0.0]);
        // the quantum pair goes all across at half the sweep and all back at its end
        let half = slosh_levels(SLOSH_S / 2.0);
        let want = numbers("F-D.pair.quantum.half").unwrap();
        assert!(
            (half[2] - want[0]).abs() < 1e-6 && (half[3] - want[1]).abs() < 1e-6,
            "{half:?}"
        );
        assert!(half[3] > 0.999999);
        let full = slosh_levels(SLOSH_S);
        let want = numbers("F-D.pair.quantum.full").unwrap();
        assert!(
            (full[2] - want[0]).abs() < 1e-6 && (full[3] - want[1]).abs() < 1e-6,
            "{full:?}"
        );
        assert!(full[2] > 0.999999);
        // the heat pair settles and stays: level at the end, never going back
        assert!((full[0] - 0.5).abs() < 1e-2 && (full[1] - 0.5).abs() < 1e-2, "{full:?}");
        let run: Vec<[f64; 4]> = (0..=60).map(|i| slosh_levels(SLOSH_S * f64::from(i) / 60.0)).collect();
        assert!(
            run.windows(2)
                .all(|w| w[1][0] <= w[0][0] + 1e-12 && w[1][1] >= w[0][1] - 1e-12)
        );
        // the facts' own heat row at t = 1
        let at = SLOSH_S / SLOSH_END;
        let row = numbers("F-D.pair.heat.t1").unwrap();
        let got = slosh_levels(at);
        assert!(
            (got[0] - row[0]).abs() < 1e-6 && (got[1] - row[1]).abs() < 1e-6,
            "{got:?}"
        );
        // nothing is created or lost: each pair's two bars add up to one
        for l in &run {
            assert!((l[0] + l[1] - 1.0).abs() < 1e-9 && (l[2] + l[3] - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn the_tones_of_12_6_follow_their_bars_and_keep_to_the_scale() {
        assert!(SLOSH_PITCHES.iter().all(|&p| in_scale(p)));
        assert!((SLOSH_WINDOW_S - SLOSH_S - SLOSH_TAIL_S).abs() < 1e-12);
        assert!((SLOSH_END - PI).abs() < 1e-12, "the sweep's end is the half turn");
        let s = slosh_buffer();
        assert_eq!(s.len(), frames(SLOSH_WINDOW_S));
        assert!(s.is_finite());
        let lufs = explainer_score::meter::integrated(&s);
        assert!((lufs - LEVEL_LUFS).abs() < 0.2, "{lufs} LUFS");
        assert_eq!((s.l[s.len() - 1], s.r[s.len() - 1]), (0.0, 0.0));
        // each tone is as strong as its bar is tall, in its panel's channel:
        // heat on the left (A for the first node, E for the second), the
        // quantum pair on the right (D, then A above it)
        let (a4, e5, d5, a5) = (SLOSH_PITCHES[0], SLOSH_PITCHES[1], SLOSH_PITCHES[2], SLOSH_PITCHES[3]);
        // at the start the second nodes are silent: heat's E and quantum's A
        assert!(strength(&s.l, e5, 0.05, 0.3) < 0.2 * strength(&s.l, a4, 0.05, 0.3));
        assert!(strength(&s.r, a5, 0.05, 0.3) < 0.2 * strength(&s.r, d5, 0.05, 0.3));
        // across at half the sweep: the quantum pair has gone to the second node
        let mid = SLOSH_S / 2.0;
        assert!(strength(&s.r, d5, mid - 0.15, mid + 0.15) < 0.1 * strength(&s.r, a5, mid - 0.15, mid + 0.15));
        // and back at its end, while heat's two nodes have levelled
        let end = SLOSH_S;
        assert!(strength(&s.r, a5, end - 0.3, end) < 0.1 * strength(&s.r, d5, end - 0.3, end));
        let (heat_a, heat_e) = (
            strength(&s.l, a4, end - 0.5, end - 0.2),
            strength(&s.l, e5, end - 0.5, end - 0.2),
        );
        assert!(
            (heat_a / heat_e - 1.0).abs() < 0.1,
            "heat has not levelled: {heat_a} against {heat_e}"
        );
        assert!(energy_above(&s, 4_000.0) < 1e-4);
    }
}
