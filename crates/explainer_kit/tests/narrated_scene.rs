//! The narrator in a real scene, end to end: lines placed frame-exactly
//! after their breaths, a mark, a music breath, a sonification window, a
//! stem laid from t = 0, the timeline the score engine reads (round-tripped
//! through its text format and turned into a voice bus), and the exact cue
//! log.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use explainer_kit::{Kit, Narrator, Palette, Script, v};
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

fn write_line(path: &Path, seconds: f64) {
    let n = (seconds * 48_000.0) as usize;
    let m: Vec<f64> = (0..n).map(|i| 0.1 * (i as f64 * 0.05).sin()).collect();
    wav::write(path, &Stereo::from_mono(&m), wav::Format::F32).unwrap();
}

#[test]
fn the_narrator_places_lines_frame_exactly_and_logs_everything() {
    let dir = std::env::temp_dir().join(format!("explainer_kit_narrated_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    write_line(&dir.join("01_1.wav"), 0.25);
    write_line(&dir.join("01_2.wav"), 0.25);
    write_line(&dir.join("01_3.wav"), 0.25);
    let script = Script::parse("01_1\tHello.\n01_2\tAgain.\n01_3\tOnce more.\n").unwrap();
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
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn a_bad_cue_is_refused_when_placed() {
    let dir = std::env::temp_dir().join(format!("explainer_kit_badcue_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    write_line(&dir.join("01_1.wav"), 0.25);
    let script = Script::parse("01_1\tHello.\n02_1\tMissing.\n").unwrap();
    // a scripted line with no file is an error before anything renders
    let err = Narrator::load(&script, &dir, 0.0).err().expect("missing 02_1.wav");
    assert!(err.to_string().contains("02_1.wav"), "{err}");
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
    let dir = std::env::temp_dir().join(format!("explainer_kit_sonify_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    write_line(&dir.join("01_1.wav"), 0.25);
    write_line(&dir.join("01_2.wav"), 0.25);
    let sound = dir.join("cloud.wav");
    write_line(&sound, 3.0);
    let script = Script::parse("01_1\tHello.\n01_2\tAfter the cloud.\n").unwrap();
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
