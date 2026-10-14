//! Video G's teaching sound (storyboard, "the quiz device"): the countdown
//! of a quiz, the music alone, holding the motif open until the answer
//! lands. One vibraphone note a second climbs the motif's own arpeggio (1̂ 3̂
//! 5̂, then 6̂, a step above where a plain arpeggio would come round), left
//! on 6̂ while the ring drains; as it reaches zero the identity's resolution
//! winds down 5̂ 3̂ 1̂ and rings alone before the next line. Every pitch is
//! the identity's own (`score::identity`), so a retuned motif retunes the
//! countdown. The sound is written at about the narration's level; the score
//! re-levels it, and it stays under a 3 kHz top. Nothing here has been
//! auditioned yet: the score bead tunes it.

use std::f64::consts::PI;
use std::path::PathBuf;

use explainer_score::filter::{Biquad, Kind};
use explainer_score::instr::{Instrument, Note, render};
use explainer_score::sonify::motif;
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

/// The rungs a countdown climbs, low to high (MIDI): the motif's own notes
/// (1̂ 3̂ 5̂ 6̂) and, under them, its 3̂ and 5̂ an octave down, so a longer
/// countdown starts lower on the same arpeggio.
pub fn rungs() -> Vec<f64> {
    let id = crate::score::identity();
    let m = id.motif.pitches(&id.key, id.tonic_midi);
    let mut out = vec![f64::from(m[1] - 12), f64::from(m[2] - 12)];
    out.extend(m.iter().map(|&p| f64::from(p)));
    out
}

/// The notes of a countdown of `seconds`, one a second from its first
/// second to its last: the last `seconds` rungs, so the climb always ends
/// on 6̂ and the first note tells the length.
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
    for (k, &pitch) in countdown_pitches(seconds).iter().enumerate() {
        let note = Note::new(pitch, 0.55, 0.6, k as u64 + 1);
        out.mix_in(&render(Instrument::Vibraphone, &note), frames(k as f64), 1.0);
    }
    let id = crate::score::identity();
    let (answer, _) = motif(&id, &id.resolution, id.palette.answer, 0, 0.75, 21);
    out.mix_in(&answer, frames(f64::from(seconds)), 1.0);
    finish(out, total)
}

/// [`countdown_buffer`] as a file the listen window carries.
pub fn countdown(seconds: u32) -> fmn::Result<PathBuf> {
    wav(&format!("g_countdown_{seconds}"), &countdown_buffer(seconds))
}

#[cfg(test)]
mod tests {
    use explainer_score::fft::welch;

    use super::*;

    /// The most seconds a countdown can have, one a rung.
    const MOST_SECONDS: u32 = 6;

    /// The pitches the answer winds down: the identity's resolution (5̂ 3̂ 1̂).
    fn resolution_pitches() -> Vec<f64> {
        let id = crate::score::identity();
        id.resolution
            .pitches(&id.key, id.tonic_midi)
            .into_iter()
            .map(f64::from)
            .collect()
    }

    /// The loudest sample of `s` (either channel) between `from` and `to`
    /// seconds.
    fn peak_between(s: &Stereo, from: f64, to: f64) -> f64 {
        let (a, b) = (frames(from), frames(to).min(s.len()));
        s.l[a..b].iter().chain(&s.r[a..b]).fold(0.0_f64, |m, x| m.max(x.abs()))
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

    #[test]
    fn the_climb_is_the_motifs_own_arpeggio_and_is_left_on_the_sixth() {
        let id = crate::score::identity();
        let motif: Vec<f64> = id
            .motif
            .pitches(&id.key, id.tonic_midi)
            .into_iter()
            .map(f64::from)
            .collect();
        // the motif is 1̂ 3̂ 5̂ 6̂ of F major: F A C D
        assert_eq!(motif, vec![65.0, 69.0, 72.0, 74.0]);
        for seconds in 4..=MOST_SECONDS {
            let p = countdown_pitches(seconds);
            assert_eq!(p.len(), seconds as usize, "one note a second");
            assert!(p.windows(2).all(|w| w[0] < w[1]), "it only climbs: {p:?}");
            // it ends on the motif's own last note, the sixth, not the tonic
            assert_eq!(&p[p.len() - 4..], &motif[..], "the last four notes are the motif");
            assert_ne!(
                (p[p.len() - 1] as i32).rem_euclid(12),
                id.key.tonic,
                "it ends on the tonic"
            );
            let key = id.key.pitch_classes();
            assert!(
                p.iter().all(|&n| key.contains(&(n as i32).rem_euclid(12))),
                "{p:?} leaves the key"
            );
        }
        // the first note tells the length
        let first: Vec<f64> = (4..=6).map(|s| countdown_pitches(s)[0]).collect();
        assert_eq!(first, vec![65.0, 60.0, 57.0]);
        // beyond the rungs there are no more notes: a longer countdown is refused by the next test
        assert_eq!(rungs().len(), MOST_SECONDS as usize);
        assert_eq!(countdown_pitches(MOST_SECONDS + 3).len(), rungs().len());
    }

    #[test]
    fn the_answer_winds_down_the_resolution_to_the_tonic() {
        let id = crate::score::identity();
        let r = resolution_pitches();
        assert_eq!(r, vec![72.0, 69.0, 65.0], "5̂ 3̂ 1̂: C A F");
        assert!(r.windows(2).all(|w| w[0] > w[1]), "it winds down");
        assert_eq!((r[r.len() - 1] as i32).rem_euclid(12), id.key.tonic);
        // it starts just under where the climb was left
        let top = *countdown_pitches(4).last().unwrap();
        assert!(r[0] < top && top - r[0] <= 2.0, "{} under {top}", r[0]);
    }

    #[test]
    fn every_quiz_has_a_countdown_the_rungs_can_play() {
        assert_eq!(crate::model::QUIZZES.len(), 15, "one quiz a chapter");
        for q in crate::model::QUIZZES {
            assert!((4..=MOST_SECONDS).contains(&q.seconds), "{}: {} s", q.line, q.seconds);
            assert_eq!(countdown_pitches(q.seconds).len(), q.seconds as usize);
        }
    }

    #[test]
    fn a_countdown_is_the_ring_and_the_answers_beat_under_the_top() {
        for seconds in 4..=MOST_SECONDS {
            let s = countdown_buffer(seconds);
            let total = f64::from(seconds) + LAND_S;
            assert_eq!(s.len(), frames(total), "the window's length");
            assert!(s.is_finite());
            let lufs = explainer_score::meter::integrated(&s);
            assert!((lufs - LEVEL_LUFS).abs() < 0.2, "{lufs} LUFS");
            // exactly zero where the window ends
            assert_eq!((s.l[s.len() - 1], s.r[s.len() - 1]), (0.0, 0.0));
            // a note sounds in every second of the countdown, and the three
            // notes of the resolution each in their own slot
            for k in 0..seconds {
                let from = f64::from(k);
                assert!(
                    peak_between(&s, from, from + 0.3) > 0.01,
                    "{seconds} s: silent at {k} s"
                );
            }
            let id = crate::score::identity();
            for j in 0..3 {
                let at = f64::from(seconds) + id.beat_s * 0.5 * f64::from(j);
                assert!(
                    peak_between(&s, at, at + 0.2) > 0.01,
                    "{seconds} s: no resolution note {j}"
                );
            }
            // the answer's notes are the loudest of the countdown
            let answer = peak_between(&s, f64::from(seconds), total);
            let climb = peak_between(&s, 0.0, f64::from(seconds));
            assert!(answer > climb, "the answer {answer} is not over the climb {climb}");
            assert!(energy_above(&s, 4_000.0) < 1e-4, "{seconds} s: energy above 4 kHz");
        }
    }

    /// The card counts down as a listen window that carries the sound: it
    /// opens once the question has been spoken, lasts the ring's seconds and
    /// the answer's beat, carries the countdown's file, and the scene runs
    /// through all of it, so the next line cannot start inside the beat.
    #[test]
    fn the_cards_window_carries_the_countdown_and_the_scene_runs_through_it() {
        use std::cell::RefCell;
        use std::rc::Rc;

        use explainer_kit::{Kit, Narrator, Script, v};
        use fmn::prelude::{SceneConstruct, Stage};

        use super::super::draw::{QuizCard, QuizLook, say};
        use super::super::the_book;

        struct Probe<F>(F);
        impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                (self.0)(stage)
            }
        }
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let mut kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        kit.chapter = "01_the_book".to_owned();
        let seen: RefCell<Option<(explainer_score::timeline::Window, f64, f64)>> = RefCell::new(None);
        fmn::run_scene(
            &mut Probe(|stage: &mut Stage<'_>| {
                say(&kit, stage, "01_1")?;
                let card = QuizCard::build(&kit, stage, the_book::ASK, QuizLook::at(v(0.0, 0.0), 1.0))?;
                card.show(stage, 0.3)?;
                card.count(&kit, stage)?;
                let after = stage.scene().time().to_f64();
                let fps = f64::from(narrator.timeline(stage, "01_the_book").fps);
                let timeline = narrator.timeline(stage, "01_the_book");
                let window = timeline.listens.first().cloned().expect("a listen window");
                assert_eq!(timeline.listens.len(), 1, "one window for one quiz");
                *seen.borrow_mut() = Some((window, after, fps));
                Ok(())
            }),
            fmn::scene::RuntimeConfig::default(),
            7,
            &mut fmn::scene::NullSceneSink,
        )
        .expect("the probe scene runs");
        let (window, after, fps) = seen.into_inner().expect("the scene ran");
        let n = f64::from(the_book::ASK.seconds().unwrap());
        assert_eq!(window.label, "01_the_book:01_1");
        assert!(
            (window.seconds - (n + LAND_S)).abs() < 1e-9,
            "{} s for {n} s",
            window.seconds
        );
        let path = window.sound.as_ref().expect("the window carries the countdown's file");
        assert!(path.exists());
        // the window opens when the question has been spoken
        let said = narrator.said().into_iter().find(|(id, _)| id == "01_1").unwrap();
        let line_end = said.1 + narrator.length("01_1").unwrap();
        let start = window.frame as f64 / fps;
        assert!(
            start >= line_end - 1e-6,
            "the window opens at {start} s, inside the question to {line_end} s"
        );
        // the scene ran through the ring and the answer's beat, and not far past it
        let end = start + window.seconds;
        assert!(
            after >= end - 1.5 / fps,
            "the scene stopped at {after} s, before the window's end {end} s"
        );
        assert!(
            after <= end + 0.5,
            "the scene ran on to {after} s, past the window's end {end} s"
        );
    }
}
