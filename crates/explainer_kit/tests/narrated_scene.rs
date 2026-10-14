//! The narrator in a real scene, end to end: lines placed frame-exactly
//! after their breaths, a mark, a music breath, a sonification window, a
//! stem laid from t = 0, the timeline the score engine reads (round-tripped
//! through its text format and turned into a voice bus), and the exact cue
//! log.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use explainer_kit::{Kit, LineGaps, Narrator, Palette, Script, v};
use explainer_score::timeline::ChapterTimeline;
use explainer_score::{Stereo, wav};
use fmn::prelude::*;

/// 258 frames at 60 fps, with every placement predictable by hand.
struct Talk {
    kit: Kit,
    stem: PathBuf,
    timeline: Rc<RefCell<Option<ChapterTimeline>>>,
}

impl SceneConstruct for Talk {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        // the first line waits a breath too (the v5 cue log starts at 0.600 s)
        kit.say(stage, "01_1")?; // frame 36
        let t = kit.text(stage, "Hello", 30.0, Palette::ink(), v(0.0, 0.0))?;
        stage.play(write(stage.arena(), t))?; // 1 s: frame 96
        kit.say(stage, "01_2")?; // due at 36 + 15 + 36 = 87, already past: frame 96
        kit.mark(stage, "reveal_x"); // frame 96
        kit.music_breath(stage, 1.0)?; // after 01_2 + breath: 96 + 15 + 36 = 147; then 60 frames
        kit.say(stage, "01_3")?; // right after the breath: frame 207
        kit.hold(stage)?; // 207 + 15 + 36 = 258
        let n = kit.narrator.as_ref().expect("a narrator");
        let secs = stage.scene().time().to_f64();
        n.sound(stage, &self.stem, -secs, Some(0.0))
            .expect("the stem is a valid cue");
        *self.timeline.borrow_mut() = Some(n.timeline(stage, "01_talk"));
        Ok(())
    }
}

/// A directory for this test's output that no run has used: the process id
/// and the clock, skipping any path that already exists. Nothing is ever
/// overwritten or deleted (AGENTS.md, rule 1); the OS's temp cleaning
/// reclaims it.
fn fresh_dir(tag: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    (0u32..)
        .map(|k| std::env::temp_dir().join(format!("explainer_kit_{tag}_{}_{nanos}_{k}", std::process::id())))
        .find(|p| !p.exists())
        .expect("an unused name")
}

fn write_line(path: &Path, seconds: f64) {
    let n = (seconds * 48_000.0) as usize;
    let m: Vec<f64> = (0..n).map(|i| 0.1 * (i as f64 * 0.05).sin()).collect();
    wav::write(path, &Stereo::from_mono(&m), wav::Format::F32).unwrap();
}

/// Metadata for these synthetic 0.25 s test lines, not an ASR result.
fn write_report(dir: &Path, script: &Script) {
    let lines: serde_json::Map<String, serde_json::Value> = script
        .lines()
        .iter()
        .map(|(id, text)| {
            (
                id.clone(),
                serde_json::json!([{
                    "sentence": text, "out_samples": 12_000, "gap_after_samples": 0
                }]),
            )
        })
        .collect();
    std::fs::write(
        dir.join("report.json"),
        serde_json::json!({"meta": {"rate": 48_000}, "lines": lines}).to_string(),
    )
    .unwrap();
}

#[test]
fn the_narrator_places_lines_frame_exactly_and_logs_everything() {
    let dir = fresh_dir("narrated");
    std::fs::create_dir_all(&dir).unwrap();
    write_line(&dir.join("01_1.wav"), 0.25);
    write_line(&dir.join("01_2.wav"), 0.25);
    write_line(&dir.join("01_3.wav"), 0.25);
    let script = Script::parse("01_1\tHello.\n01_2\tAgain.\n01_3\tOnce more.\n").unwrap();
    write_report(&dir, &script);
    let narrator = Narrator::load(&script, &dir, 0.0).unwrap();
    let stem = dir.join("stem.wav");
    // a stem exactly the scene's length
    wav::write(&stem, &Stereo::silence(258 * 800), wav::Format::F32).unwrap();
    let got = Rc::new(RefCell::new(None));
    let mut scene = Talk {
        kit: Kit::new().unwrap().with_narrator(Rc::clone(&narrator)),
        stem,
        timeline: Rc::clone(&got),
    };
    let out = dir.join("frames");
    let mut opts = RenderOptions::with_format(&out, RenderFormat::PngSequence).unwrap();
    opts.config.camera.resolution = (32, 18);
    opts.config.camera.fps = 60;
    let report = fmn::render(&mut scene, opts).expect("render");
    assert_eq!(report.artifact.frame_count, 258);

    // the cue log
    let cues = narrator.cues();
    let starts: Vec<u64> = cues.iter().map(|c| c.frame).collect();
    assert_eq!(
        starts,
        vec![36, 96, 207, 258],
        "lines frame-exact; the stem logged at the end"
    );
    assert_eq!(cues[3].start_sample(), 0, "the stem is laid from t = 0");
    assert_eq!(cues[1].start_sample(), 96 * 800);
    assert!(cues.iter().all(|c| c.file.sha256.len() == 64 && c.fps == 60));
    let log = narrator.cue_log();
    assert_eq!(log.lines().count(), 1 + 4, "a header and one row per cue");
    assert!(
        log.lines().nth(2).unwrap().ends_with("\t76800"),
        "01_2 starts at sample 76,800"
    );
    assert!(log.contains("0x0000000000000000"), "gain 0 dB logged as exact bits");

    // the timeline, as the score engine will read it
    let tl = got.borrow().clone().unwrap();
    assert_eq!((tl.fps, tl.frames), (60, 258));
    let lines: Vec<(&str, u64)> = tl.lines.iter().map(|l| (l.id.as_str(), l.start_frame)).collect();
    assert_eq!(lines, vec![("01_1", 36), ("01_2", 96), ("01_3", 207)]);
    assert_eq!(tl.marks.len(), 1);
    assert_eq!((tl.marks[0].label.as_str(), tl.marks[0].frame), ("reveal_x", 96));
    assert_eq!(tl.breaths.len(), 1);
    assert_eq!((tl.breaths[0].frame, tl.breaths[0].seconds), (147, 1.0));
    // exact through the score engine's text format
    let back = ChapterTimeline::parse(&tl.to_text()).unwrap();
    assert_eq!(back, tl);
    // and the voice bus puts the second line at sample 76,800 exactly
    let bus = back.voice_bus(None).unwrap();
    assert_eq!(bus.len(), 258 * 800);
    assert!(bus.l[76_790..76_800].iter().all(|x| *x == 0.0));
    assert!(bus.l[76_800..76_800 + 200].iter().any(|x| x.abs() > 0.01));
}

#[test]
fn a_bad_cue_is_refused_when_placed() {
    let dir = fresh_dir("badcue");
    std::fs::create_dir_all(&dir).unwrap();
    write_line(&dir.join("01_1.wav"), 0.25);
    let script = Script::parse("01_1\tHello.\n02_1\tMissing.\n").unwrap();
    // a scripted line with no file is an error before anything renders
    let err = Narrator::load(&script, &dir, 0.0).err().expect("missing 02_1.wav");
    assert!(err.to_string().contains("02_1.wav"), "{err}");
}

#[test]
fn rewording_a_line_with_the_same_id_requires_new_narration() {
    let dir = fresh_dir("stale_read");
    std::fs::create_dir_all(&dir).unwrap();
    write_line(&dir.join("01_1.wav"), 0.25);
    write_line(&dir.join("01_2.wav"), 0.25);
    let original = Script::parse("01_1\tHello.\n01_2\tAgain.\n").unwrap();
    write_report(&dir, &original);
    Narrator::load(&original, &dir, 0.0).expect("matching text and audio load");
    let changed = Script::parse("01_1\tHello.\n01_2\tSomething else.\n").unwrap();
    let err = Narrator::load(&changed, &dir, 0.0).err().expect("old read refused");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("01_2"), "{err}");
    assert!(err.to_string().contains("differs from script.tsv"), "{err}");
}

#[test]
fn real_audio_requires_its_report_and_matching_timing() {
    let script = Script::parse("01_1\tHello.\n").unwrap();
    let absent = fresh_dir("missing_report");
    std::fs::create_dir_all(&absent).unwrap();
    write_line(&absent.join("01_1.wav"), 0.25);
    let err = Narrator::load(&script, &absent, 0.0).err().expect("report required");
    assert!(err.to_string().contains("report.json"), "{err}");

    let mistimed = fresh_dir("mistimed_report");
    std::fs::create_dir_all(&mistimed).unwrap();
    write_line(&mistimed.join("01_1.wav"), 0.5);
    write_report(&mistimed, &script); // claims 0.25 s for a 0.5 s WAV
    let err = Narrator::load(&script, &mistimed, 0.0)
        .err()
        .expect("wrong timing refused");
    assert!(err.to_string().contains("01_1 accounts for"), "{err}");
}

struct WrongLine {
    kit: Kit,
    duplicate: bool,
}

impl SceneConstruct for WrongLine {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        if self.duplicate {
            self.kit.say(stage, "01_1")?;
        }
        self.kit.say(stage, if self.duplicate { "01_1" } else { "missing" })
    }
}

#[test]
fn an_unknown_or_repeated_line_returns_a_render_error() {
    let script = Script::parse("01_1\tOne.\n").unwrap();
    for duplicate in [false, true] {
        let dir = fresh_dir("wrong_line");
        let mut scene = WrongLine {
            kit: Kit::new().unwrap().with_narrator(Narrator::estimate(&script)),
            duplicate,
        };
        let mut opts = RenderOptions::with_format(&dir, RenderFormat::PngSequence).unwrap();
        opts.config.camera.resolution = (32, 18);
        opts.config.camera.fps = 60;
        let err = fmn::render(&mut scene, opts).expect_err("bad line refuses the render");
        let reason = if duplicate { "said twice" } else { "missing" };
        assert!(err.to_string().contains(reason), "{err}");
    }
}

/// A sonification window: it opens after the line and its breath, does not
/// wait (the scene animates through it), and holds the next line off until
/// it ends.
struct Sonified {
    kit: Kit,
    sound: PathBuf,
    timeline: Rc<RefCell<Option<ChapterTimeline>>>,
}

impl SceneConstruct for Sonified {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "01_1")?; // frame 36, 15 frames long
        kit.sonify(stage, 2.0, "cloud", &self.sound)?; // opens at 36 + 15 + 36 = 87
        assert_eq!(stage.scene().time().frames(), 87, "sonify does not wait");
        let t = kit.text(stage, "listen", 30.0, Palette::ink(), v(0.0, 0.0))?;
        stage.play(write(stage.arena(), t))?; // 1 s inside the window: frame 147
        kit.say(stage, "01_2")?; // held to the window's end: 87 + 120 = 207
        kit.hold(stage)?;
        *self.timeline.borrow_mut() = Some(kit.narrator.as_ref().unwrap().timeline(stage, "01_son"));
        Ok(())
    }
}

#[test]
fn a_sonification_window_lets_the_scene_animate_and_holds_the_next_line() {
    let dir = fresh_dir("sonify");
    std::fs::create_dir_all(&dir).unwrap();
    write_line(&dir.join("01_1.wav"), 0.25);
    write_line(&dir.join("01_2.wav"), 0.25);
    let sound = dir.join("cloud.wav");
    write_line(&sound, 3.0);
    let script = Script::parse("01_1\tHello.\n01_2\tAfter the cloud.\n").unwrap();
    write_report(&dir, &script);
    for with_stem in [false, true] {
        let narrator = Narrator::load(&script, &dir, 0.0).unwrap();
        let mut kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        if with_stem {
            kit.stem = Some(dir.join("not_placed_here.wav"));
        }
        let got = Rc::new(RefCell::new(None));
        let mut scene = Sonified {
            kit,
            sound: sound.clone(),
            timeline: Rc::clone(&got),
        };
        let mut opts =
            RenderOptions::with_format(dir.join(format!("frames_{with_stem}")), RenderFormat::PngSequence).unwrap();
        opts.config.camera.resolution = (32, 18);
        opts.config.camera.fps = 60;
        fmn::render(&mut scene, opts).expect("render");
        let tl = got.borrow().clone().unwrap();
        let lines: Vec<(&str, u64)> = tl.lines.iter().map(|l| (l.id.as_str(), l.start_frame)).collect();
        assert_eq!(
            lines,
            vec![("01_1", 36), ("01_2", 207)],
            "the next line waits for the window"
        );
        assert_eq!(tl.listens.len(), 1);
        let w = &tl.listens[0];
        assert_eq!((w.frame, w.seconds, w.label.as_str()), (87, 2.0, "cloud"));
        assert_eq!(w.sound.as_deref(), Some(sound.as_path()));
        assert_eq!(ChapterTimeline::parse(&tl.to_text()).unwrap(), tl);
        // a narrated draft plays it directly; under a score stem, the stem has it
        let placed: Vec<(u64, Option<f64>)> = narrator
            .cues()
            .iter()
            .filter(|c| c.file.path == sound)
            .map(|c| (c.frame, c.gain_db))
            .collect();
        if with_stem {
            assert!(placed.is_empty(), "{placed:?}");
        } else {
            assert_eq!(placed, vec![(87, Some(explainer_kit::kit::DRAFT_SONIFY_DB))]);
        }
    }
}

/// Five 0.25 s lines whose neighbours are a continuation (`Then ...`), a beat,
/// a topic (`Your turn.`), then a music breath before the last: where each
/// starts depends on the line-gap rule alone.
struct Thoughts {
    kit: Kit,
    timeline: Rc<RefCell<Option<ChapterTimeline>>>,
}

impl SceneConstruct for Thoughts {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "01_1")?;
        kit.hold(stage)?;
        kit.mark(stage, "after_1");
        kit.say(stage, "01_2")?;
        kit.hold(stage)?;
        kit.mark(stage, "after_2");
        kit.say(stage, "01_3")?;
        kit.say(stage, "01_4")?;
        kit.music_breath(stage, 1.0)?;
        kit.say(stage, "01_5")?;
        kit.hold(stage)?;
        *self.timeline.borrow_mut() = Some(kit.narrator.as_ref().unwrap().timeline(stage, "01_thoughts"));
        Ok(())
    }
}

fn tiny_render(dir: PathBuf) -> RenderOptions {
    let mut opts = RenderOptions::with_format(dir, RenderFormat::PngSequence).unwrap();
    opts.config.camera.resolution = (32, 18);
    opts.config.camera.fps = 60;
    opts
}

/// The timeline of `Thoughts` under a line-gap rule.
fn thoughts(gaps: LineGaps) -> ChapterTimeline {
    let dir = fresh_dir("thoughts");
    std::fs::create_dir_all(&dir).unwrap();
    let script = Script::parse(
        "01_1\tWe start here.\n01_2\tThen we go on.\n01_3\tThat makes a pair.\n01_4\tYour turn.\n01_5\tIt ends here.\n",
    )
    .unwrap()
    .with_line_gaps(gaps);
    for (id, _) in script.lines() {
        write_line(&dir.join(format!("{id}.wav")), 0.25);
    }
    write_report(&dir, &script);
    let got = Rc::new(RefCell::new(None));
    let mut scene = Thoughts {
        kit: Kit::new()
            .unwrap()
            .with_narrator(Narrator::load(&script, &dir, 0.0).unwrap()),
        timeline: Rc::clone(&got),
    };
    fmn::render(&mut scene, tiny_render(dir.join("frames"))).expect("render");
    let tl = got.borrow().clone();
    tl.expect("the scene recorded its timeline")
}

fn starts_of(tl: &ChapterTimeline) -> Vec<u64> {
    tl.lines.iter().map(|l| l.start_frame).collect()
}

#[test]
fn thought_gaps_move_the_lines_and_leave_authored_windows_and_the_off_rule_alone() {
    // the constant breath, as it always was: every line 0.6 s after the one before
    let off = thoughts(LineGaps::Constant);
    assert_eq!(starts_of(&off), vec![36, 87, 138, 189, 300]);
    assert_eq!(off.marks.iter().map(|m| m.frame).collect::<Vec<_>>(), vec![87, 138]);
    assert_eq!((off.breaths[0].frame, off.breaths[0].seconds), (240, 1.0));
    assert_eq!(off.frames, 351);

    // the thought: the first line has nothing before it (the breath); `Then ...`
    // carries on after 0.58 s; the beat after it waits 0.81 s; `Your turn.`
    // opens a topic, 0.99 s
    let on = thoughts(LineGaps::Thought);
    assert_eq!(starts_of(&on), vec![36, 86, 150, 225, 336]);
    // `hold` waits the shorter gap of a continuation, and never longer than the
    // breath before a beat, so what an author puts after a hold keeps its place
    assert_eq!(on.marks.iter().map(|m| m.frame).collect::<Vec<_>>(), vec![86, 137]);
    // the window is 0.6 s after the line before it, as before, and the next line
    // starts at its end, not a breath later
    assert_eq!((on.breaths[0].frame, on.breaths[0].seconds), (276, 1.0));
    assert_eq!(
        on.breaths[0].frame - on.lines[3].start_frame,
        off.breaths[0].frame - off.lines[3].start_frame
    );
    assert_eq!(on.lines[4].start_frame - on.breaths[0].frame, 60);
    assert_eq!(off.lines[4].start_frame - off.breaths[0].frame, 60);
    assert_eq!(on.frames, 387);
    // the gaps between the first four lines are no longer one value (the spread
    // check on a timeline with its windows and waits is its own test, below)
    let gaps = |tl: &ChapterTimeline| -> Vec<f64> {
        [(0, 1), (1, 2), (2, 3)]
            .iter()
            .map(|&(a, b)| (tl.lines[b].start_frame - tl.lines[a].start_frame) as f64 / 60.0 - 0.25)
            .collect()
    };
    let (g_on, g_off) = (gaps(&on), gaps(&off));
    assert!(g_on.windows(2).all(|w| (w[1] - w[0]).abs() > 0.1), "{g_on:?}");
    assert!(g_off.windows(2).all(|w| (w[1] - w[0]).abs() < 0.02), "{g_off:?}");
    // and each timeline round-trips through the score engine's text format
    for tl in [&on, &off] {
        assert_eq!(&ChapterTimeline::parse(&tl.to_text()).unwrap(), tl);
    }
}

/// One line said at a chapter's start.
struct Opening {
    kit: Kit,
    id: &'static str,
}

impl SceneConstruct for Opening {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        self.kit.say(stage, self.id)
    }
}

#[test]
fn a_chapter_rendered_alone_opens_where_the_whole_render_opens_it() {
    // 02_1 opens a topic: 0.99 s of silence before it under the thought rule,
    // the breath otherwise, whether or not chapter 1 is in the render
    let opening = |gaps: LineGaps, partial: bool| -> u64 {
        let dir = fresh_dir("opening");
        std::fs::create_dir_all(&dir).unwrap();
        let full = Script::parse("01_1\tWe start here.\n02_1\tYour turn.\n")
            .unwrap()
            .with_line_gaps(gaps);
        let script = if partial {
            full.select(|id| id.starts_with("02_"))
        } else {
            full
        };
        for (id, _) in script.lines() {
            write_line(&dir.join(format!("{id}.wav")), 0.25);
        }
        write_report(&dir, &script);
        let narrator = Narrator::load(&script, &dir, 0.0).unwrap();
        let mut scene = Opening {
            kit: Kit::new().unwrap().with_narrator(Rc::clone(&narrator)),
            id: "02_1",
        };
        fmn::render(&mut scene, tiny_render(dir.join("frames"))).expect("render");
        narrator.cues()[0].frame
    };
    assert_eq!(opening(LineGaps::Thought, false), 60);
    assert_eq!(opening(LineGaps::Thought, true), 60);
    assert_eq!(opening(LineGaps::Constant, false), 36);
    assert_eq!(opening(LineGaps::Constant, true), 36);
}

/// A take as a file: `lead` s of silence, `voiced` s of a tone, `tail` s of silence.
fn write_edged(path: &Path, lead: f64, voiced: f64, tail: f64) {
    let n = |s: f64| (s * 48_000.0).round() as usize;
    let mut m = vec![0.0; n(lead)];
    m.extend((0..n(voiced)).map(|i| 0.1 * (i as f64 * 0.05).sin()));
    m.extend(vec![0.0; n(tail)]);
    wav::write(path, &Stereo::from_mono(&m), wav::Format::F32).unwrap();
}

/// The report for takes of any length: one sentence per line, its samples read from the file.
fn write_report_for(dir: &Path, script: &Script) {
    let lines: serde_json::Map<String, serde_json::Value> = script
        .lines()
        .iter()
        .map(|(id, text)| {
            let frames = wav::read(&dir.join(format!("{id}.wav"))).unwrap().frames();
            (
                id.clone(),
                serde_json::json!([{"sentence": text, "out_samples": frames, "gap_after_samples": 0}]),
            )
        })
        .collect();
    let report = serde_json::json!({"meta": {"rate": 48_000}, "lines": lines});
    std::fs::write(dir.join("report.json"), report.to_string()).unwrap();
}

/// A line, a window (a sonification when `sound` is set, a plain listen window
/// otherwise), then a line the thought rule would put 0.99 s after the one
/// before: it starts exactly where the window ends.
struct AfterWindow {
    kit: Kit,
    sound: Option<PathBuf>,
    timeline: Rc<RefCell<Option<ChapterTimeline>>>,
}

impl SceneConstruct for AfterWindow {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "01_1")?; // frame 36, 15 frames long
        match &self.sound {
            Some(sound) => kit.sonify(stage, 2.0, "cloud", sound)?, // opens at 87, does not wait
            None => kit.listen(stage, 2.0, "question")?,            // opens at 87 and waits 120 frames
        }
        kit.say(stage, "01_2")?;
        *self.timeline.borrow_mut() = Some(kit.narrator.as_ref().unwrap().timeline(stage, "01_window"));
        Ok(())
    }
}

#[test]
fn the_line_after_a_listen_or_sonify_window_starts_exactly_at_its_end_under_the_thought_rule() {
    let dir = fresh_dir("after_window");
    std::fs::create_dir_all(&dir).unwrap();
    for id in ["01_1", "01_2"] {
        write_line(&dir.join(format!("{id}.wav")), 0.25);
    }
    let sound = dir.join("cloud.wav");
    write_line(&sound, 3.0);
    // "Your turn." opens a topic: after the line before it the thought rule would wait 0.99 s,
    // but after a window the window's own breath stands
    let script = Script::parse("01_1\tWe start here.\n01_2\tYour turn.\n").unwrap();
    write_report(&dir, &script);
    for gaps in [LineGaps::Constant, LineGaps::Thought] {
        for with_sound in [false, true] {
            let narrator = Narrator::load(&script.clone().with_line_gaps(gaps), &dir, 0.0).unwrap();
            let got = Rc::new(RefCell::new(None));
            let mut scene = AfterWindow {
                kit: Kit::new().unwrap().with_narrator(narrator),
                sound: with_sound.then(|| sound.clone()),
                timeline: Rc::clone(&got),
            };
            let tag = format!("{gaps:?} sound {with_sound}");
            fmn::render(
                &mut scene,
                tiny_render(dir.join(format!("frames_{tag}").replace(' ', "_"))),
            )
            .expect("render");
            let tl = got.borrow().clone().expect("the scene recorded its timeline");
            let starts: Vec<u64> = tl.lines.iter().map(|l| l.start_frame).collect();
            assert_eq!(tl.listens.len(), 1, "{tag}");
            let w = &tl.listens[0];
            assert_eq!((w.frame, w.seconds), (87, 2.0), "{tag}");
            assert_eq!(
                starts,
                vec![36, 207],
                "{tag}: the line starts where the window ends (87 + 120)"
            );
        }
    }
}

/// Chapter 1 of two: two lines, the last of them held by a window (a music
/// breath) or not.
struct FirstChapter {
    kit: Kit,
    ends_in_window: bool,
}

impl SceneConstruct for FirstChapter {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        self.kit.say(stage, "01_1")?;
        self.kit.say(stage, "01_2")?;
        if self.ends_in_window {
            self.kit.music_breath(stage, 1.0)?;
        }
        Ok(())
    }
}

#[test]
fn one_narrator_over_two_chapters_opens_the_second_as_a_partial_render_does() {
    // 02_1 opens a topic: 0.99 s before it under the thought rule (its own take starts
    // at once), whatever chapter 1 ended with. Chapter 1's last take has a 0.4 s quiet
    // tail, which a render of chapter 2 alone does not have in front of it, and chapter 1
    // may end in a window, whose own breath is not chapter 2's.
    let make = |tag: &str, script: &Script| -> (PathBuf, Rc<Narrator>) {
        let dir = fresh_dir(tag);
        std::fs::create_dir_all(&dir).unwrap();
        write_edged(&dir.join("01_1.wav"), 0.0, 0.2, 0.05);
        write_edged(&dir.join("01_2.wav"), 0.0, 0.2, 0.40);
        write_edged(&dir.join("02_1.wav"), 0.0, 0.2, 0.05);
        write_report_for(&dir, script);
        let narrator = Narrator::load(script, &dir, 0.0).unwrap();
        (dir, narrator)
    };
    let full = Script::parse("01_1\tWe start here.\n01_2\tThen we go on.\n02_1\tYour turn.\n")
        .unwrap()
        .with_line_gaps(LineGaps::Thought);
    // chapter 2 alone, from the script cut to it
    let partial = full.select(|id| id.starts_with("02_"));
    let (dir, alone) = make("two_chapters_alone", &partial);
    let mut scene = Opening {
        kit: Kit::new().unwrap().with_narrator(Rc::clone(&alone)),
        id: "02_1",
    };
    fmn::render(&mut scene, tiny_render(dir.join("frames"))).expect("render");
    let want = alone.cues()[0].frame;
    assert_eq!(want, 60, "the partial render opens the topic 0.99 s in");

    for ends_in_window in [false, true] {
        let (dir, narrator) = make("two_chapters", &full);
        let mut first = FirstChapter {
            kit: Kit::new().unwrap().with_narrator(Rc::clone(&narrator)),
            ends_in_window,
        };
        fmn::render(&mut first, tiny_render(dir.join("frames_1"))).expect("render chapter 1");
        assert_eq!(narrator.cues().len(), 2, "chapter 1 placed its two lines");
        // what the CLI does between chapters
        narrator.reset();
        let mut second = Opening {
            kit: Kit::new().unwrap().with_narrator(Rc::clone(&narrator)),
            id: "02_1",
        };
        fmn::render(&mut second, tiny_render(dir.join("frames_2"))).expect("render chapter 2");
        let got = narrator.cues();
        assert_eq!(got.len(), 1, "reset left only chapter 2's cues");
        assert_eq!(
            got[0].frame, want,
            "chapter 2 after chapter 1 (ending in a window: {ends_in_window}) opens where the partial render does"
        );
    }
}

/// Seven lines with edged takes: three plain boundaries, a listen window, one more
/// plain boundary, then a picture wait of 2.5 s before the last line. (A window of 0.3 s.)
struct Mixed {
    kit: Kit,
    timeline: Rc<RefCell<Option<ChapterTimeline>>>,
}

impl SceneConstruct for Mixed {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        for id in ["01_1", "01_2", "01_3", "01_4"] {
            kit.say(stage, id)?;
        }
        // a short window: the pair it sits in is 0.9 s apart, under the plain limit,
        // so only the window itself keeps it out
        kit.listen(stage, 0.3, "question")?;
        kit.say(stage, "01_5")?;
        kit.say(stage, "01_6")?;
        stage.wait(2.5)?; // the picture is not ready: the next line waits for it
        kit.say(stage, "01_7")?;
        *self.timeline.borrow_mut() = Some(kit.narrator.as_ref().unwrap().timeline(stage, "01_mixed"));
        Ok(())
    }
}

#[test]
fn the_spread_check_reads_a_real_timeline_without_its_windows_and_its_waits() {
    let dir = fresh_dir("mixed");
    std::fs::create_dir_all(&dir).unwrap();
    let text = "01_1\tWe start here.\n01_2\tThen we go on.\n01_3\tThat makes a pair.\n01_4\tYour turn.\n\
                01_5\tIt ends here.\n01_6\tAnd one more thing.\n01_7\tHere's the last one.\n";
    let script = Script::parse(text).unwrap();
    for (id, _) in script.lines() {
        write_edged(&dir.join(format!("{id}.wav")), 0.12, 0.3, 0.12);
    }
    write_report_for(&dir, &script);
    let (mut spread, mut counts) = (Vec::new(), Vec::new());
    for gaps in [LineGaps::Thought, LineGaps::Constant] {
        let narrator = Narrator::load(&script.clone().with_line_gaps(gaps), &dir, 0.0).unwrap();
        let got = Rc::new(RefCell::new(None));
        let mut scene = Mixed {
            kit: Kit::new().unwrap().with_narrator(Rc::clone(&narrator)),
            timeline: Rc::clone(&got),
        };
        fmn::render(&mut scene, tiny_render(dir.join(format!("frames_{gaps:?}")))).expect("render");
        let tl = got.borrow().clone().expect("the scene recorded its timeline");
        let lines: Vec<(f64, f64)> = tl
            .lines
            .iter()
            .map(|l| (l.start_frame as f64 / 60.0, narrator.length(&l.id).unwrap()))
            .collect();
        let windows: Vec<f64> = tl
            .listens
            .iter()
            .chain(&tl.breaths)
            .map(|w| w.frame as f64 / 60.0)
            .collect();
        assert_eq!((lines.len(), windows.len()), (7, 1));
        let plain = explainer_kit::narration::plain_line_gaps(&lines, &windows);
        // the pair with the window and the pair with the picture wait are out; four are left
        let every: Vec<f64> = lines.windows(2).map(|p| p[1].0 - (p[0].0 + p[0].1)).collect();
        assert_eq!((every.len(), plain.len()), (6, 4), "{gaps:?}: {every:?} -> {plain:?}");
        assert!(
            every[3] > 0.8 && every[3] < explainer_kit::narration::PLAIN_GAP_MAX_S,
            "{gaps:?}: the window's pair is under the plain limit: {every:?}"
        );
        assert!(every[5] > 1.5, "{gaps:?}: the picture wait: {every:?}");
        spread.push((
            gaps,
            explainer_kit::narration::check_gap_spread(&plain),
            explainer_kit::narration::gap_cv(&every).unwrap(),
        ));
        counts.push(plain);
    }
    // under the thought rule the plain gaps are the classes less the takes' 0.24 s of quiet edges
    let (thought, constant) = (&counts[0], &counts[1]);
    let near = |a: f64, b: f64| (a - b).abs() < 0.04;
    let classes = [0.58, 0.81, 0.99, 0.58];
    assert!(
        thought.iter().zip(classes).all(|(g, c)| near(*g, c - 0.24)),
        "{thought:?}"
    );
    // under the constant breath every plain gap is 0.6 s
    assert!(constant.iter().all(|g| near(*g, 0.6)), "{constant:?}");
    assert!(spread[0].1.is_ok(), "{:?}", spread[0].1);
    assert!(
        spread[1].1.is_err(),
        "the constant rule is the planted negative: {:?}",
        spread[1].1
    );
    // counted over every pair, windows and waits included, the constant rule would pass
    assert!(spread[1].2 > explainer_kit::narration::MIN_GAP_CV, "{}", spread[1].2);
}
