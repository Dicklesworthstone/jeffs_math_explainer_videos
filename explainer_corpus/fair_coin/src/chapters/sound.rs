//! Video E's teaching sounds (the storyboard's listen column; review R1's C2
//! and C11): the motif's two notes, 5̂ and 6̂, stand for the two orders a
//! deciding pair can fall in. Heads then tails (Alice's) rises from the
//! lower to the upper; tails then heads (Bob's) falls from the upper to the
//! lower; a thrown-away pair is a soft tick.
//!
//! - 02_8: the trick's run as it lands. Its first games pair by pair, ticks
//!   and the two notes in the order they fell; then the race, each game the
//!   note its deciding pair ends on, an even patter.
//! - 03_2: the copy's flip by the clock. The two notes as it lifts off the
//!   green block, the same two swapped as it lands on the violet one.
//! - 05_2: the even patter of 02_8, then the two coins' games the same way,
//!   the upper note four times in five.
//!
//! Each sound is written at about the narration's level, under a 3 kHz top,
//! and fades to an exact zero; the score plays it at the identity's listen
//! level. Every game comes from `facts` (runs 2 and 5); the two pitches come
//! from the identity's motif.

use std::path::PathBuf;
use std::sync::Mutex;

use explainer_score::filter::{Biquad, Kind};
use explainer_score::instr::{Note, render};
use explainer_score::{FS, Stereo};

use super::draw::fail;
use crate::facts::{trick_run, two_coin_run};
use crate::model::{Face, Game};

/// A sonification's integrated loudness as written (the narration's).
const LEVEL_LUFS: f64 = -23.0;
/// The fade every sound ends with, seconds.
const TAIL_S: f64 = 0.25;
/// Every sound's top, a 4th-order low-pass: QA counts what passes 8 kHz over
/// a quiet bed as clicks (F's mallets and ticks, 2026-10-13).
const TOP_HZ: f64 = 3_000.0;
/// The soft tick of a thrown-away pair: a short knock this high.
const TICK_HZ: f64 = 520.0;

/// The three windows' length, seconds (each chapter's `kit.listen` had it).
pub const WINDOW_S: f64 = 3.0;
/// 02_8: the time each pair of the first games takes, the gap between a
/// deciding pair's two notes, and how long the pairs go on before the race.
pub const PAIR_S: f64 = 0.13;
pub const PAIR_GAP_S: f64 = 0.065;
pub const PAIRS_S: f64 = 1.55;
/// The race: one note a game, this far apart, from [`RACE_AT`] seconds on;
/// it stops this long before the window ends, so the last note rings out.
pub const RACE_STEP_S: f64 = 0.1;
pub const RACE_AT: f64 = 1.7;
const RACE_CLEAR_S: f64 = 0.35;
/// 03_2: the flip by the clock starts this long into the window and takes
/// this long; the copy has lifted after [`super::draw::LIFT_SHARE`] of it.
pub const FLIP_DELAY_S: f64 = 0.15;
pub const FLIP_S: f64 = 2.1;
/// 05_2: the even patter, then the two coins' games from this time.
pub const FOUR_TO_ONE_AT: f64 = 1.3;

fn frames(seconds: f64) -> usize {
    (seconds * FS).round() as usize
}

/// The motif's two notes, lower and upper: its first two degrees over the
/// identity's tonic (5̂ and 6̂: D and E over G).
pub fn the_two() -> (f64, f64) {
    let id = crate::score::identity();
    let pitch = |k: usize| f64::from(id.key.degree(id.motif.degrees[k], id.tonic_midi));
    let (a, b) = (pitch(0), pitch(1));
    (a.min(b), a.max(b))
}

/// One note of the sound: when (seconds), what (a pitch, or None for the
/// tick) and how hard.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Strike {
    pub at: f64,
    pub pitch: Option<f64>,
    pub velocity: f64,
}

/// A deciding pair's two notes in the order its tosses fell: heads first
/// rises, tails first falls.
fn pair_notes(first: Face, at: f64, velocity: f64) -> [Strike; 2] {
    let (low, high) = the_two();
    let (a, b) = if first == Face::H { (low, high) } else { (high, low) };
    [
        Strike {
            at,
            pitch: Some(a),
            velocity,
        },
        Strike {
            at: at + PAIR_GAP_S,
            pitch: Some(b),
            velocity,
        },
    ]
}

/// The note a game ends on in the race: its deciding pair's second.
fn race_note(game: &Game, at: f64) -> Option<Strike> {
    let (low, high) = the_two();
    game.last().map(|&(first, _)| Strike {
        at,
        pitch: Some(if first == Face::H { high } else { low }),
        velocity: 0.42,
    })
}

/// The strikes of the first whole games of `games` that fit in `seconds`
/// from `from`, pair by pair: a tick for a pair of the same face, the two
/// notes for a deciding pair. Returns them and how many games were played.
fn pairs(games: &[Game], from: f64, seconds: f64) -> (Vec<Strike>, usize) {
    let mut out = Vec::new();
    let mut at = from;
    let mut played = 0;
    for game in games {
        if at + PAIR_S * game.len() as f64 > from + seconds + 1e-9 {
            break;
        }
        for &(a, b) in game {
            if a == b {
                out.push(Strike {
                    at,
                    pitch: None,
                    velocity: 0.5,
                });
            } else {
                out.extend(pair_notes(a, at, 0.55));
            }
            at += PAIR_S;
        }
        played += 1;
    }
    (out, played)
}

/// The race over `games` from `from` to `until`, a note a game.
fn race(games: &[Game], from: f64, until: f64) -> Vec<Strike> {
    let n = ((until - from) / RACE_STEP_S).floor().max(0.0) as usize;
    games
        .iter()
        .take(n)
        .enumerate()
        .filter_map(|(k, g)| race_note(g, from + k as f64 * RACE_STEP_S))
        .collect()
}

/// 02_8's strikes: run 2's first games pair by pair, then the race of the
/// games after them.
pub fn the_trick_strikes() -> Vec<Strike> {
    let games = trick_run();
    let (mut out, played) = pairs(&games, 0.0, PAIRS_S);
    out.extend(race(&games[played..], RACE_AT, WINDOW_S - RACE_CLEAR_S));
    out
}

/// 03_2's strikes: the two notes rising as the copy lifts, falling as it
/// lands.
pub fn the_flip_strikes() -> Vec<Strike> {
    let lifted = FLIP_DELAY_S + super::draw::LIFT_SHARE * FLIP_S;
    let landed = FLIP_DELAY_S + FLIP_S;
    let mut out = pair_notes(Face::H, FLIP_DELAY_S, 0.6).to_vec();
    out[1].at = lifted;
    let mut down = pair_notes(Face::T, landed - (lifted - FLIP_DELAY_S), 0.6);
    down[1].at = landed;
    out.extend(down);
    out
}

/// 05_2's strikes: run 2's race (the even patter), then run 5's (two
/// different coins: Alice four games in five).
pub fn the_two_coins_strikes() -> Vec<Strike> {
    let mut out = race(&trick_run(), 0.1, FOUR_TO_ONE_AT - 0.1);
    out.extend(race(&two_coin_run(), FOUR_TO_ONE_AT + 0.1, WINDOW_S - RACE_CLEAR_S));
    out
}

/// The soft tick: [`TICK_HZ`] dying in about 25 ms over a 3 ms rise.
fn tick() -> Vec<f64> {
    let rise = frames(0.003);
    (0..frames(0.05))
        .map(|i| {
            let t = i as f64 / FS;
            let g = if i < rise {
                0.5 - 0.5 * (std::f64::consts::PI * i as f64 / rise as f64).cos()
            } else {
                1.0
            };
            g * (-t / 0.007).exp() * (std::f64::consts::TAU * TICK_HZ * t).sin()
        })
        .collect()
}

/// The strikes played: the notes on the identity's answer instrument (a
/// small struck thing, like a coin on a table), the ticks dry.
pub fn audio(strikes: &[Strike], seconds: f64) -> Stereo {
    let inst = crate::score::identity().palette.answer;
    let mut out = Stereo::silence(frames(seconds + 3.0));
    let knock = Stereo::from_mono(&tick());
    for (k, s) in strikes.iter().enumerate() {
        match s.pitch {
            Some(p) => {
                let note = Note::new(p, s.velocity, 0.3, k as u64 + 1);
                out.mix_in(&render(inst, &note), frames(s.at), 1.0);
            }
            None => out.mix_in(&knock, frames(s.at), s.velocity),
        }
    }
    finish(out, seconds)
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

/// `sound_file` renames a temporary of the process's own into place, so two
/// chapter tests (threads of one process) writing the same sound would rename
/// each other's away; a render is one chapter per process. The tests take
/// this lock.
static WRITING: Mutex<()> = Mutex::new(());

fn wav(label: &str, s: &Stereo) -> fmn::Result<PathBuf> {
    let _one_at_a_time = WRITING.lock().unwrap_or_else(|e| e.into_inner());
    explainer_kit::sound_file(label, s).map_err(|e| fail(format!("sonification {label}: {e}")))
}

/// 02_8's sound as a file.
pub fn the_trick() -> fmn::Result<PathBuf> {
    wav("the_trick", &audio(&the_trick_strikes(), WINDOW_S))
}

/// 03_2's sound as a file.
pub fn the_flip() -> fmn::Result<PathBuf> {
    wav("the_flip", &audio(&the_flip_strikes(), WINDOW_S))
}

/// 05_2's sound as a file.
pub fn the_two_coins() -> fmn::Result<PathBuf> {
    wav("the_two_coins", &audio(&the_two_coins_strikes(), WINDOW_S))
}

#[cfg(test)]
mod tests {
    use explainer_score::score::clashes;

    use super::*;
    use crate::model::heads_first_wins;

    fn notes(s: &[Strike]) -> Vec<f64> {
        s.iter().filter_map(|x| x.pitch).collect()
    }

    /// The two notes are the motif's first two, D and E over G, and neither is
    /// a semitone from any chord the video plays (the identity's progression
    /// and every mood's): rule 7 never has to move them.
    #[test]
    fn the_two_notes_are_the_motifs_and_clash_with_no_chord() {
        let (low, high) = the_two();
        assert_eq!((low, high), (74.0, 76.0));
        let id = crate::score::identity();
        let chords = id
            .progression
            .iter()
            .chain(id.moods.iter().flat_map(|m| &m.progression));
        for c in chords {
            for p in [low, high] {
                assert!(!clashes(p as i32, c), "{p} against {c:?}");
            }
        }
    }

    /// 02_8 plays run 2's first games as they fell: a tick for each pair of
    /// the same face, a deciding pair rising for heads first and falling for
    /// tails first, whole games only; then the race, a note a game in the
    /// games' order, about as many of each as the run's wins.
    #[test]
    fn the_trick_plays_the_runs_first_games_then_an_even_race() {
        let games = trick_run();
        let strikes = the_trick_strikes();
        let (low, high) = the_two();
        let (first, rest): (Vec<Strike>, Vec<Strike>) = strikes.iter().partition(|s| s.at < RACE_AT - 1e-9);
        // the pairs, in order, against the games
        let mut want = Vec::new();
        let mut at = 0.0;
        for game in &games {
            if at + PAIR_S * game.len() as f64 > PAIRS_S + 1e-9 {
                break;
            }
            for &(a, b) in game {
                if a == b {
                    want.push(None);
                } else if a == Face::H {
                    want.extend([Some(low), Some(high)]);
                } else {
                    want.extend([Some(high), Some(low)]);
                }
                at += PAIR_S;
            }
        }
        let have: Vec<Option<f64>> = first.iter().map(|s| s.pitch).collect();
        assert_eq!(have, want);
        assert!(have.contains(&None) && have.len() >= 10, "{have:?}");
        // the race: the games after them, a note each, in order
        let played = want.iter().filter(|p| p.is_some()).count() / 2;
        let race_games = &games[played..played + rest.len()];
        for (s, g) in rest.iter().zip(race_games) {
            let alice = g.last().unwrap().0 == Face::H;
            assert_eq!(s.pitch, Some(if alice { high } else { low }));
        }
        assert!(rest.len() >= 8, "{} race notes", rest.len());
        let alice = heads_first_wins(race_games);
        assert!(
            alice > 0 && alice < race_games.len(),
            "an even patter has both: {alice} of {}",
            race_games.len()
        );
        // everything sounds inside the window, the last note clear of its end
        assert!(
            strikes
                .iter()
                .all(|s| s.at >= 0.0 && s.at <= WINDOW_S - RACE_CLEAR_S + 1e-9)
        );
    }

    /// 03_2: the two notes rise as the copy lifts off and fall as it lands,
    /// at the times the flip by the clock does.
    #[test]
    fn the_flip_rises_on_the_lift_and_falls_on_the_landing() {
        let s = the_flip_strikes();
        let (low, high) = the_two();
        assert_eq!(notes(&s), vec![low, high, high, low]);
        assert!((s[0].at - FLIP_DELAY_S).abs() < 1e-9);
        assert!((s[1].at - (FLIP_DELAY_S + super::super::draw::LIFT_SHARE * FLIP_S)).abs() < 1e-9);
        assert!(
            (s[3].at - (FLIP_DELAY_S + FLIP_S)).abs() < 1e-9,
            "the last note as it lands"
        );
        assert!(s[3].at < WINDOW_S - TAIL_S, "and rings before the window ends");
    }

    /// 05_2: the even patter, then the two coins' games: the upper note four
    /// times in five or so, against about half before.
    #[test]
    fn the_two_coins_go_from_even_to_four_to_one() {
        let s = the_two_coins_strikes();
        let (_, high) = the_two();
        let share = |v: Vec<&Strike>| v.iter().filter(|x| x.pitch == Some(high)).count() as f64 / v.len() as f64;
        let before: Vec<&Strike> = s.iter().filter(|x| x.at < FOUR_TO_ONE_AT).collect();
        let after: Vec<&Strike> = s.iter().filter(|x| x.at > FOUR_TO_ONE_AT).collect();
        assert!(
            before.len() >= 8 && after.len() >= 10,
            "{} then {}",
            before.len(),
            after.len()
        );
        let (b, a) = (share(before), share(after));
        assert!((0.3..=0.7).contains(&b), "the patter is even: {b:.2}");
        assert!(a >= 0.65, "four to one: {a:.2}");
    }

    /// Each sound is the window long, under the top, at the narration's level,
    /// and ends in an exact zero.
    #[test]
    fn each_sound_is_the_windows_length_level_and_top() {
        for strikes in [the_trick_strikes(), the_flip_strikes(), the_two_coins_strikes()] {
            let s = audio(&strikes, WINDOW_S);
            assert_eq!(s.len(), frames(WINDOW_S));
            assert_eq!((s.l[s.len() - 1], s.r[s.len() - 1]), (0.0, 0.0));
            let lufs = explainer_score::meter::integrated(&s);
            assert!((lufs - LEVEL_LUFS).abs() < 0.3, "{lufs}");
            // what passes 8 kHz (QA's click band) is 40 dB under the whole: it
            // measured 42 dB under it with the 3 kHz top
            let mut hp = Biquad::new(Kind::HighPass { q: 0.707 }, 8_000.0);
            let high: f64 = s.l.iter().map(|&v| hp.process(v).powi(2)).sum();
            let all: f64 = s.l.iter().map(|v| v * v).sum();
            let db = 10.0 * (high / all).log10();
            assert!(db < -40.0, "{db:.1} dB above 8 kHz");
        }
    }
}
