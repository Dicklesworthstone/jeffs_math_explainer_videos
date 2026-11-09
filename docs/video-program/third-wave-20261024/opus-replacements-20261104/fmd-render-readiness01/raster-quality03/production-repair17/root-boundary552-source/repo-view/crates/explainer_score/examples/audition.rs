//! Audition an identity on real speech before its video's narration exists:
//! three chapters paced like the videos' (sentences with 0.6 s breaths, the
//! title's listen window, an authored 2.5 s music breath, a listen window
//! carrying a sonification on the identity's listen scale, a chapter in one
//! of its moods, and a closing chapter whose listen window resolves the
//! theme), scored and judged by the same QA as `./x <slug> score`.
//!
//! ```text
//! cargo run --release -p explainer_score --example audition -- IDENTITY SENTENCE_DIR OUT_DIR
//! ```
//!
//! IDENTITY is `capture_recapture`, `raptorq_gf256`, `bakery_algorithm` or
//! `hoeffding_d`; SENTENCE_DIR holds sentence WAVs (any rate), used in name
//! order and cycled. Writes to OUT_DIR: `<chapter>.timeline.tsv`,
//! `stems/<chapter>.music.f32.wav`, `qa/<chapter>.json`, `voice.wav`,
//! `music.wav` and `mix.wav` (voice and music summed, then one fixed gain to
//! −16 LUFS or −1.5 dBTP, whichever binds: the master's rule), and prints
//! each chapter's QA verdict. Exit status 1 when any chapter fails QA.

use std::fs;
use std::path::{Path, PathBuf};

use explainer_score::instr::{self, Note};
use explainer_score::qa::{self, ScoreFacts};
use explainer_score::score::{self, ChapterPlan, Identity, Section, Theme};
use explainer_score::timeline::{ChapterTimeline, LinePlacement, Window};
use explainer_score::units::db_to_gain;
use explainer_score::{FS, Stereo, meter, wav};

const FPS: u32 = 60;
/// The narration's breath between sentences (s).
const BREATH: f64 = 0.6;

/// One beat of a chapter: a sentence, or a window.
enum Beat {
    Line,
    Breath(f64),
    Listen(f64, &'static str),
    Sonify(f64),
    Hold(f64),
}

struct Sentences {
    wavs: Vec<(PathBuf, f64)>,
    next: usize,
}

impl Sentences {
    fn read(dir: &Path) -> Result<Self, String> {
        let mut paths: Vec<PathBuf> = fs::read_dir(dir)
            .map_err(|e| format!("{}: {e}", dir.display()))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "wav"))
            .collect();
        paths.sort();
        let wavs = paths
            .into_iter()
            .map(|p| {
                let d = wav::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
                let secs = d.frames() as f64 / f64::from(d.rate);
                Ok((p, secs))
            })
            .collect::<Result<Vec<_>, String>>()?;
        if wavs.is_empty() {
            return Err(format!("no .wav files in {}", dir.display()));
        }
        Ok(Self { wavs, next: 0 })
    }

    fn take(&mut self) -> (PathBuf, f64) {
        let w = self.wavs[self.next % self.wavs.len()].clone();
        self.next += 1;
        w
    }
}

/// A sonification on the identity's listen scale: rising notes on its
/// answer instrument, every third on its melody instrument (A's tagged fish
/// among the untagged, a byte's set bits, a ticket's degrees), written near
/// the narration's −23 LUFS (the score re-levels it).
fn sonification(id: &Identity, seconds: f64) -> Stereo {
    let scale = id.listen_scale();
    let pitches: Vec<i32> = (72..=84).filter(|m| scale.contains(&(m % 12))).collect();
    let mut out = Stereo::silence((seconds * FS) as usize);
    let count = ((seconds - 0.6) / 0.3).floor() as usize;
    for k in 0..count {
        // C5–C6 on the answer instrument; every third note on the melody
        // instrument an octave down, where the motif lives (a mallet struck
        // above E5 puts an upper mode past 7 kHz, which QA hears as a click)
        let (inst, pitch) = match k % 3 {
            2 => (id.palette.melody, pitches[(k * 2) % pitches.len()] - 12),
            _ => (id.palette.answer, pitches[(k * 2) % pitches.len()]),
        };
        let mut note = Note::new(f64::from(pitch), 0.45, 0.25, 1_000 + k as u64);
        note.pan = if k % 2 == 0 { -0.3 } else { 0.3 };
        out.mix_in(&instr::render(inst, &note), (k as f64 * 0.3 * FS) as usize, 1.0);
    }
    let lufs = meter::integrated(&out);
    out.scale(db_to_gain(-23.0 - lufs));
    out
}

fn chapter(
    name: &str,
    beats: &[Beat],
    sentences: &mut Sentences,
    id: &Identity,
    out: &Path,
) -> Result<ChapterTimeline, String> {
    let mut t = 0.5;
    let mut tl = ChapterTimeline {
        chapter: name.into(),
        fps: FPS,
        ..Default::default()
    };
    let frame = |t: f64| (t * f64::from(FPS)).round() as u64;
    for (i, b) in beats.iter().enumerate() {
        match b {
            Beat::Line => {
                let (wav, secs) = sentences.take();
                tl.lines.push(LinePlacement {
                    id: format!("{name}_{i}"),
                    start_frame: frame(t),
                    gain_db: 0.0,
                    wav,
                });
                t += secs + BREATH;
            }
            Beat::Breath(s) => {
                tl.breaths.push(Window {
                    frame: frame(t),
                    seconds: *s,
                    label: String::new(),
                    sound: None,
                });
                t += s;
            }
            Beat::Listen(s, label) => {
                tl.listens.push(Window {
                    frame: frame(t),
                    seconds: *s,
                    label: (*label).into(),
                    sound: None,
                });
                t += s;
            }
            Beat::Sonify(s) => {
                let path = out.join(format!("{name}_sonification.wav"));
                wav::write(&path, &sonification(id, *s), wav::Format::F32).map_err(|e| e.to_string())?;
                tl.listens.push(Window {
                    frame: frame(t),
                    seconds: *s,
                    label: format!("{name}_listen"),
                    sound: Some(path),
                });
                t += s;
            }
            Beat::Hold(s) => t += s,
        }
    }
    tl.frames = frame(t);
    fs::write(out.join(format!("{name}.timeline.tsv")), tl.to_text()).map_err(|e| e.to_string())?;
    Ok(tl)
}

fn run(args: &[String]) -> Result<bool, String> {
    let [name, dir, out] = args else {
        return Err("usage: audition IDENTITY SENTENCE_DIR OUT_DIR".into());
    };
    let id = match name.as_str() {
        "capture_recapture" => Identity::capture_recapture(),
        "raptorq_gf256" => Identity::raptorq_gf256(),
        "bakery_algorithm" => Identity::bakery_algorithm(),
        "hoeffding_d" => Identity::hoeffding(),
        other => return Err(format!("no identity {other}")),
    };
    let out = PathBuf::from(out);
    fs::create_dir_all(out.join("stems")).map_err(|e| e.to_string())?;
    fs::create_dir_all(out.join("qa")).map_err(|e| e.to_string())?;
    let mut sentences = Sentences::read(Path::new(dir))?;
    let theme = |want: Theme| id.themes.iter().find(|t| t.1 == want).map_or("title", |t| t.0);
    let (title, close) = (theme(Theme::Open), theme(Theme::Resolved));
    // a mood the title chapter does not play, else the home progression
    let mood = id
        .moods
        .iter()
        .find(|m| !m.chapters.contains(&"01"))
        .map_or("05", |m| m.chapters[0]);
    let close_number = close.split('_').next().filter(|n| n.len() == 2).unwrap_or("99");
    use Beat::{Breath, Hold, Line, Listen, Sonify};
    let plan = [
        (
            "01_title".to_string(),
            vec![
                Line,
                Line,
                Line,
                Listen(3.0, title),
                Line,
                Line,
                Line,
                Breath(2.5),
                Line,
                Line,
                Hold(2.0),
            ],
        ),
        (
            format!("{mood}_mood"),
            vec![
                Line,
                Line,
                Line,
                Line,
                Sonify(3.0),
                Line,
                Line,
                Line,
                Breath(2.5),
                Line,
                Line,
                Hold(1.5),
            ],
        ),
        (
            format!("{close_number}_close"),
            vec![Line, Line, Line, Listen(5.0, close), Line, Line, Hold(4.0)],
        ),
    ];
    let mut sections = Vec::new();
    for (k, (name, beats)) in plan.iter().enumerate() {
        let timeline = chapter(name, beats, &mut sentences, &id, &out)?;
        sections.push(Section {
            timeline,
            plan: ChapterPlan {
                intensity: [0.35, 0.5, 0.45][k],
                resolve: k == plan.len() - 1,
            },
        });
    }
    let s = score::render(&id, &sections, None).map_err(|e| e.to_string())?;
    println!(
        "{}: {:.1} s, {} chords, {} notes, {} phrases; voice {:.1} LUFS; stem {:.1} dBTP",
        id.name,
        s.music.len() as f64 / FS,
        s.chords.len(),
        s.events.len(),
        s.phrases.len(),
        s.voice_lufs,
        s.true_peak_db
    );
    for c in &s.chords {
        println!(
            "  {:6.2}-{:6.2} s  {:?}  bass {} upper {:?}",
            c.start, c.end, c.chord, c.voicing.bass, c.voicing.upper
        );
    }
    if !s.skipped_marks.is_empty() {
        println!("marks skipped: {:?}", s.skipped_marks);
    }
    let mut all_pass = true;
    for k in 0..s.chapters.len() {
        let key = &s.chapters[k].0;
        let music = s.chapter_music(k);
        wav::write(
            &out.join(format!("stems/{key}.music.f32.wav")),
            &music,
            wav::Format::F32,
        )
        .map_err(|e| e.to_string())?;
        let r = qa::evaluate(
            key,
            &s.chapter_voice(k),
            &music,
            music.len(),
            Some(&ScoreFacts::chapter(&s, k)),
        );
        fs::write(out.join(format!("qa/{key}.json")), r.to_json()).map_err(|e| e.to_string())?;
        let failed: Vec<&str> = r.failures().iter().map(|m| m.name.as_str()).collect();
        println!(
            "{key:<14} QA {}",
            if failed.is_empty() {
                "PASS".into()
            } else {
                format!("FAIL {failed:?}")
            }
        );
        for name in [
            "voice_minus_music_median_lu",
            "voice_minus_music_p5_lu",
            "music_loudest_400ms_re_speech_lu",
            "sii_loss_median",
            "sii_loss_max",
            "music_to_voice_1_4khz_db",
            "onsets_per_s_gaps",
            "exposed_beating_db",
        ] {
            if let Some(m) = r.get(name) {
                println!("    {name:<34} {:>8.3}  ({})", m.value, m.limit);
            }
        }
        all_pass &= r.passed();
    }
    // the whole audition, summed and brought to the delivery level by one gain
    let mut mix = s.voice.clone();
    for (a, b) in mix.l.iter_mut().zip(&s.music.l) {
        *a += b;
    }
    for (a, b) in mix.r.iter_mut().zip(&s.music.r) {
        *a += b;
    }
    let gain = (-16.0 - meter::integrated(&mix)).min(-1.5 - meter::true_peak_db(&mix));
    mix.scale(db_to_gain(gain));
    for (file, x) in [("voice.wav", &s.voice), ("music.wav", &s.music), ("mix.wav", &mix)] {
        wav::write(&out.join(file), x, wav::Format::F32).map_err(|e| e.to_string())?;
    }
    println!("mix gain {gain:+.1} dB -> {}", out.join("mix.wav").display());
    Ok(all_pass)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(pass) => std::process::exit(i32::from(!pass)),
        Err(e) => {
            eprintln!("audition: {e}");
            std::process::exit(2);
        }
    }
}
