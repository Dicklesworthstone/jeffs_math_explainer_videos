//! Video H's teaching sound (storyboard, "Sound plan"): the countdown of a
//! "your turn", one soft vibraphone note a second, never a tick, in D
//! Dorian (the video's key) and never closing on the tonic, so the
//! resolution that the motif keeps for 14_6 is not spent. It is the sound
//! of a listen window, written at about the narration's level; the score
//! re-levels it. Nothing here has been auditioned yet: the score bead
//! tunes it.
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
/// The sound's top, a 4th-order low-pass: the mallet's upper partials past
/// 8 kHz count as clicks to the score's QA where the bed is quiet.
const TOP_HZ: f64 = 3_000.0;

/// D Dorian from E5 up, low to high (MIDI): the only notes a countdown
/// plays. D itself is the tonic, which the last note never is.
const SCALE: [f64; 7] = [76.0, 77.0, 79.0, 81.0, 83.0, 84.0, 86.0];

/// How long the answer has the screen to itself after the countdown's last
/// second (the last note rings here), seconds.
pub const LAND_S: f64 = 0.7;

/// The authored mark a search of the map sounds on (storyboard, "Sound
/// plan": the search sound has one meaning, a search that came back empty).
/// It is a mark and not a listen window because a search happens inside a
/// line, between two of its sentences, and a window cannot open there; the
/// score answers a `whoosh:` with pitchless noise, never an answering note.
pub const SEARCH_MARK: &str = "whoosh:search";

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

/// The notes of a countdown of `seconds`, one a second from its first
/// second to the moment the answer lands: they walk down the scale and end
/// on E, the second degree, so the first note tells the length (4 s starts
/// on B, 5 s on C, 6 s on D) and the last three always sound the same.
pub fn countdown_pitches(seconds: u32) -> Vec<f64> {
    let n = (seconds as usize + 1).min(SCALE.len());
    SCALE[..n].iter().rev().copied().collect()
}

/// A countdown's audio: one vibraphone note a second, the last as the answer
/// lands, [`LEVEL_LUFS`] and ending at an exact zero.
fn countdown_audio(seconds: u32) -> Stereo {
    let total = f64::from(seconds) + LAND_S;
    let mut out = Stereo::silence(frames(total + 4.0));
    let pitches = countdown_pitches(seconds);
    let last = pitches.len() - 1;
    for (k, &pitch) in pitches.iter().enumerate() {
        let velocity = if k == last { 0.8 } else { 0.55 };
        let note = Note::new(pitch, velocity, 0.6, k as u64 + 1);
        out.mix_in(&render(Instrument::Vibraphone, &note), frames(k as f64), 1.0);
    }
    finish(out, total)
}

/// One countdown is written at a time: `sound_file` writes through a temp
/// name of its own per process, so two chapter tests (threads of one
/// process) writing the same countdown rename each other's file away.
static WRITING: Mutex<()> = Mutex::new(());

/// A quiz's countdown as a WAV: one note a second, and the last as the
/// answer lands.
pub fn countdown(seconds: u32) -> fmn::Result<PathBuf> {
    let label = format!("h_countdown_{seconds}");
    let audio = countdown_audio(seconds);
    let _one_at_a_time = WRITING.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    explainer_kit::sound_file(&label, &audio).map_err(|e| fail(format!("sonification {label}: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// D Dorian's pitch classes: D E F G A B C.
    const DORIAN: [i32; 7] = [2, 4, 5, 7, 9, 11, 0];
    /// The tonic's pitch class (D).
    const TONIC: i32 = 2;

    #[test]
    fn every_countdown_stays_in_the_key_walks_down_and_never_closes_on_the_tonic() {
        for seconds in 4..=6u32 {
            let p = countdown_pitches(seconds);
            assert_eq!(
                p.len(),
                seconds as usize + 1,
                "one note a second and one as the answer lands"
            );
            assert!(p.windows(2).all(|w| w[0] > w[1]), "it only walks down");
            for &n in &p {
                assert!(DORIAN.contains(&(n as i32).rem_euclid(12)), "{n} is not in D Dorian");
            }
            let last = *p.last().unwrap();
            assert_ne!((last as i32).rem_euclid(12), TONIC, "the last note is the tonic");
            assert_eq!(
                &p[p.len() - 3..],
                &[79.0, 77.0, 76.0],
                "the last three always sound the same"
            );
        }
        // the first note tells the length
        let first: Vec<f64> = (4..=6).map(|s| countdown_pitches(s)[0]).collect();
        assert_eq!(first, vec![83.0, 84.0, 86.0]);
    }

    /// The search's mark is the score's pitchless cue kind: a whoosh, never
    /// a reveal (a soft dyad would answer an empty search like a find).
    #[test]
    fn the_search_mark_is_a_whoosh() {
        use explainer_score::score::CueKind;
        assert_eq!(CueKind::from_label(SEARCH_MARK), CueKind::Whoosh);
    }

    /// Root-mean-square of both channels over `[from_s, to_s)`.
    fn rms(s: &Stereo, from_s: f64, to_s: f64) -> f64 {
        let (a, b) = (frames(from_s), frames(to_s).min(s.len()));
        if b <= a {
            return 0.0;
        }
        let sum: f64 = (a..b).map(|i| s.l[i] * s.l[i] + s.r[i] * s.r[i]).sum();
        (sum / (2 * (b - a)) as f64).sqrt()
    }

    /// The Goertzel power of the mono mix over `[from_s, to_s)` at `hz`.
    fn power_at(s: &Stereo, from_s: f64, to_s: f64, hz: f64) -> f64 {
        let (a, b) = (frames(from_s), frames(to_s));
        let w = 2.0 * std::f64::consts::PI * hz / FS;
        let (mut s1, mut s2) = (0.0_f64, 0.0_f64);
        for i in a..b {
            let x = 0.5 * (s.l[i] + s.r[i]);
            let s0 = x + 2.0 * w.cos() * s1 - s2;
            s2 = s1;
            s1 = s0;
        }
        s1 * s1 + s2 * s2 - 2.0 * w.cos() * s1 * s2
    }

    fn hz(midi: f64) -> f64 {
        440.0 * 2f64.powf((midi - 69.0) / 12.0)
    }

    /// The file is the countdown's seconds and the landing long, finite, and
    /// fades to an exact zero (a sound that ended on a sample would click
    /// where the score cuts it in).
    #[test]
    fn a_countdown_is_its_seconds_and_the_landing_long_and_ends_at_zero() {
        for seconds in 4..=6u32 {
            let s = countdown_audio(seconds);
            assert_eq!(s.len(), frames(f64::from(seconds) + LAND_S), "{seconds} s");
            assert!(s.is_finite());
            assert_eq!(*s.l.last().unwrap(), 0.0);
            assert_eq!(*s.r.last().unwrap(), 0.0);
            assert_eq!(s.l[0], s.r[0], "a countdown sits in the middle");
        }
    }

    /// At the narration's level (the score re-levels it from there), with
    /// headroom.
    #[test]
    fn a_countdown_sits_at_the_narrations_level_with_headroom() {
        for seconds in 4..=6u32 {
            let s = countdown_audio(seconds);
            let lufs = explainer_score::meter::integrated(&s);
            assert!((lufs - LEVEL_LUFS).abs() < 0.1, "{seconds} s: {lufs} LUFS");
            let tp = explainer_score::meter::true_peak_db(&s);
            assert!(tp < -3.0, "{seconds} s: true peak {tp} dBTP");
        }
    }

    /// Every second of the countdown starts a note (it is louder than the
    /// tail of the note before it) and each rings and fades over its second
    /// (a mallet, not a drone and not a tick).
    #[test]
    fn a_countdown_has_one_note_a_second_and_each_dies_away() {
        for seconds in 4..=6u32 {
            let s = countdown_audio(seconds);
            for k in 0..=seconds {
                let at = f64::from(k);
                let onset = rms(&s, at, at + 0.1);
                assert!(onset > 1e-3, "{seconds} s: no note at {k} s ({onset})");
                if k > 0 {
                    let before = rms(&s, at - 0.1, at);
                    assert!(
                        onset > 2.0 * before,
                        "{seconds} s: the note at {k} s is no onset ({onset} vs {before})"
                    );
                }
                if k < seconds {
                    let ring = rms(&s, at + 0.2, at + 0.5);
                    let late = rms(&s, at + 0.8, at + 0.9);
                    assert!(
                        ring > 0.3 * onset,
                        "{seconds} s: the note at {k} s is a tick ({ring} after {onset})"
                    );
                    assert!(
                        late < 0.7 * onset,
                        "{seconds} s: the note at {k} s does not die away ({late})"
                    );
                }
            }
        }
    }

    /// The audio plays the pitches the list gives: of the seven notes of the
    /// scale, the one at each second's note is the loudest in its window.
    #[test]
    fn each_note_sounds_the_pitch_the_countdown_gives_it() {
        for seconds in 4..=6u32 {
            let s = countdown_audio(seconds);
            for (k, &pitch) in countdown_pitches(seconds).iter().enumerate() {
                let at = k as f64;
                let loudest = SCALE
                    .iter()
                    .copied()
                    .max_by(|&a, &b| {
                        power_at(&s, at + 0.05, at + 0.5, hz(a)).total_cmp(&power_at(&s, at + 0.05, at + 0.5, hz(b)))
                    })
                    .unwrap();
                assert_eq!(loudest, pitch, "{seconds} s: the note at {k} s");
            }
        }
    }

    /// What a chapter left for the score, run headless at 10 frames a second
    /// with the scratch voice (every line lasts its estimate, so the times
    /// are the structure's: where in the line a mark sits, not the real
    /// narration's seconds).
    struct Ran {
        timeline: explainer_score::timeline::ChapterTimeline,
        /// Each line said: its id, when it began and how long it lasts.
        lines: Vec<(String, f64, f64)>,
    }

    impl Ran {
        /// The marks whose label starts with `prefix`, with their times.
        fn marks(&self, prefix: &str) -> Vec<(String, f64)> {
            let fps = f64::from(self.timeline.fps);
            self.timeline
                .marks
                .iter()
                .filter(|m| m.label.starts_with(prefix))
                .map(|m| (m.label.clone(), m.frame as f64 / fps))
                .collect()
        }

        /// The end of line `id`.
        fn line_end(&self, id: &str) -> f64 {
            let (_, at, len) = self
                .lines
                .iter()
                .find(|l| l.0 == id)
                .unwrap_or_else(|| panic!("{id} was not said"));
            at + len
        }
    }

    fn run(key: &str) -> Ran {
        use std::rc::Rc;

        use explainer_kit::{Kit, Narrator, Script};
        use fmn::prelude::*;

        struct Boxed(Box<dyn SceneConstruct>);
        impl SceneConstruct for Boxed {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                self.0.construct(stage)
            }
        }
        let chapter = crate::chapters::registry()
            .into_iter()
            .find(|c| c.key == key)
            .unwrap_or_else(|| panic!("no chapter {key} in the registry"));
        let narrator = Narrator::estimate(&Script::parse(crate::SCRIPT).unwrap());
        let mut kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        kit.chapter = key.to_owned();
        let mut scene = Boxed((chapter.make)(kit.clone()));
        let config = fmn::scene::RuntimeConfig {
            fps: 10,
            ..fmn::scene::RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut fmn::scene::NullSceneSink).unwrap_or_else(|e| panic!("{key}: {e}"));
        let timeline = kit
            .finished
            .borrow()
            .clone()
            .unwrap_or_else(|| panic!("{key} left no timeline"));
        let lines = narrator
            .said()
            .into_iter()
            .map(|(id, at)| {
                let len = narrator.length(&id).unwrap_or(0.0);
                (id, at, len)
            })
            .collect();
        Ran { timeline, lines }
    }

    /// The storyboard's sound plan, as the chapters leave it for the score
    /// (every chapter run once, in this one test: the countdown files they
    /// write are one per content). The search sound has one meaning and is
    /// heard on a search of the map that came back empty only: 01_2, 06_1,
    /// 07_5 (two: C, then A), the first search of 08_4, 11_1, 12_5, 14_4;
    /// one plucked note at 11_6 where the track reaches the man, in the gap
    /// after the line and no sooner; the motif resolves once and no chapter
    /// marks a resolution of its own; two silences of two and a half
    /// seconds, at 02 and 14; the title's 3 s and the receipts' 4 s
    /// windows, and a countdown window of each quiz's seconds plus the
    /// landing with its sound.
    #[test]
    fn the_chapters_leave_the_sound_plan_for_the_score() {
        let whooshes = [
            ("01_the_question", 1),
            ("06_nothing_found", 1),
            ("07_bayes_rule", 2),
            ("08_where_next", 1),
            ("11_twelve_hours", 1),
            ("12_the_silence", 1),
            ("14_four_maps", 1),
        ];
        let mut countdowns: Vec<(String, f64)> = Vec::new();
        let mut windows: Vec<(String, f64)> = Vec::new();
        for chapter in crate::chapters::registry() {
            let ran = run(chapter.key);
            let want = whooshes.iter().find(|w| w.0 == chapter.key).map_or(0, |w| w.1);
            let got = ran.marks(SEARCH_MARK);
            assert_eq!(got.len(), want, "{}: the search sound at {got:?}", chapter.key);
            assert!(
                ran.marks("resolve").is_empty(),
                "{}: the motif resolves once, in the score's gap, not at a mark",
                chapter.key
            );
            let pizz = ran.marks("pizz");
            if chapter.key == "11_twelve_hours" {
                assert_eq!(pizz.len(), 1, "one plucked note: {pizz:?}");
                assert_eq!(pizz[0].0, "pizz:found");
                // after the line that tells it (a note struck inside a
                // sentence is skipped by the score: no clear second)
                let end = ran.line_end("11_6");
                assert!(
                    pizz[0].1 >= end - 0.05,
                    "pizz:found at {} s, inside 11_6 which ends at {end} s",
                    pizz[0].1
                );
            } else {
                assert!(pizz.is_empty(), "{}: a plucked note at {pizz:?}", chapter.key);
            }
            let silences = ran.marks("silence:");
            let ends = ran.marks("silence_end:");
            if chapter.key == "02_ninety_nine" || chapter.key == "14_four_maps" {
                assert_eq!(
                    (silences.len(), ends.len()),
                    (1, 1),
                    "{}: {silences:?} {ends:?}",
                    chapter.key
                );
                assert_eq!(silences[0].0["silence:".len()..], ends[0].0["silence_end:".len()..]);
                // two and a half seconds of nothing after the last line
                // said before the silence ends: no music breath, no cue
                let before = ran
                    .lines
                    .iter()
                    .rfind(|l| l.1 < ends[0].1)
                    .expect("a line before the silence ends");
                let gap = ends[0].1 - (before.1 + before.2);
                assert!(
                    (2.5..=2.7).contains(&gap),
                    "{}: {gap} s of silence after {}",
                    chapter.key,
                    before.0
                );
            } else {
                assert!(silences.is_empty() && ends.is_empty(), "{}: a silence", chapter.key);
            }
            for w in &ran.timeline.listens {
                match &w.sound {
                    Some(_) => countdowns.push((w.label.clone(), w.seconds)),
                    None => windows.push((w.label.clone(), w.seconds)),
                }
            }
        }
        windows.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(
            windows,
            vec![
                ("01_the_question:01_3".to_owned(), 3.0),
                ("14_four_maps:14_7".to_owned(), 4.0)
            ],
            "the title's motif left open and the receipts' resolved"
        );
        assert_eq!(countdowns.len(), crate::model::QUIZZES.len(), "{countdowns:?}");
        for q in crate::model::QUIZZES {
            let w = countdowns
                .iter()
                .find(|c| c.0.ends_with(&format!(":{}", q.line)))
                .unwrap_or_else(|| panic!("no countdown window for {}", q.line));
            assert!(
                (w.1 - (f64::from(q.seconds) + LAND_S)).abs() < 1e-9,
                "{}: {} s",
                q.line,
                w.1
            );
        }
    }

    /// The same countdown twice is the same audio, so the sound file (named
    /// by its content) and the score's fingerprint do not move between runs.
    #[test]
    fn a_countdown_renders_the_same_every_time() {
        let (a, b) = (countdown_audio(5), countdown_audio(5));
        assert_eq!(a.l, b.l);
        assert_eq!(a.r, b.r);
    }

    #[test]
    fn every_quiz_has_a_countdown_the_scale_can_play() {
        for q in crate::model::QUIZZES {
            assert!((4..=6).contains(&q.seconds), "{}: {} s", q.line, q.seconds);
        }
    }
}
