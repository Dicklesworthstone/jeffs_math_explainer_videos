//! Spike S-9 made permanent (plan §7.7): one chapter in two segments, the
//! first on the retained route and the second on the camera route, scored
//! as one piece with a stem that spans the cut, then mastered: each
//! segment's cue log replayed through fmn's own mixer and nulled against the
//! soundtrack fmn muxed, the programme concatenated, the two routes' video
//! joined. Needs ffmpeg on PATH.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use explainer_kit::encode::{self, Quality};
use explainer_kit::{BACKGROUND, Kit, Narrator, Palette, Script, master, v};
use explainer_score::score::{self, ChapterPlan, Identity, Section};
use explainer_score::timeline::ChapterTimeline;
use explainer_score::{Stereo, wav};
use fmn::library::Prism;
use fmn::prelude::*;
use fmn::rendering::CameraConfig;
use fmn::scene::CameraRig;

/// Segment A, retained route: a line and a title.
struct Flat {
    kit: Kit,
}

impl SceneConstruct for Flat {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "01_1")?;
        let t = kit.text(stage, "flat", 30.0, Palette::ink(), v(0.0, 0.0))?;
        stage.play(write(stage.arena(), t))?;
        kit.finish(stage)
    }
}

/// Segment B, camera route: a lit solid, a line, a short orbit.
struct Solid {
    kit: Kit,
    rig: CameraRig,
    target: CameraConfig,
}

impl SceneConstruct for Solid {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        stage.add(Prism::new(1.2, 1.2, 1.8).color(explainer_kit::hex("#FFD866")))?;
        kit.say(stage, "01_2")?;
        let a = self.rig.animate_to(stage.scene_mut(), &self.target)?;
        stage.play_prepared_with(
            vec![Box::new(a)],
            PlayOverrides {
                run_time: Some(1.5),
                ..PlayOverrides::default()
            },
        )?;
        kit.finish(stage)
    }
}

/// A speech-like line (as in cli_pipeline.rs), 48 kHz f32.
fn write_line(path: &Path, seconds: f64, seed: u64) {
    let mut pink = explainer_score::osc::Pink::new(seed);
    let mut lp = explainer_score::filter::OnePole::new(4_000.0);
    let m: Vec<f64> = (0..(seconds * 48_000.0) as usize)
        .map(|i| {
            let t = i as f64 / 48_000.0;
            0.3 * (0.6 + 0.4 * (std::f64::consts::TAU * 4.0 * t).sin()) * lp.lp(pink.sample())
        })
        .collect();
    wav::write(path, &Stereo::from_mono(&m), wav::Format::F32).unwrap();
}

fn kit_for(narrator: &Rc<Narrator>, stem: Option<PathBuf>) -> Kit {
    let mut kit = Kit::new().unwrap().with_narrator(Rc::clone(narrator));
    kit.chapter = "01_split".into();
    kit.stem = stem;
    kit
}

/// Options for a segment: tiny PNG frames for the timeline pass, or an MP4
/// through the delivery encoder that captures fmn's soundtrack.
fn options(out: &Path, name: &str, capture: Option<&Path>) -> RenderOptions {
    let mut o = match capture {
        None => RenderOptions::with_format(out.join(name), RenderFormat::PngSequence).unwrap(),
        Some(dir) => encode::delivery_options(
            out.join(format!("{name}.mp4")),
            RenderFormat::Mp4,
            Quality {
                quality: 20,
                preset: "ultrafast",
                capture_dir: Some(dir.to_path_buf()),
                ..Quality::delivery()
            },
        )
        .unwrap(),
    };
    o.config.camera.resolution = (64, 36);
    o.config.camera.fps = 60;
    o.config.camera.background_color = BACKGROUND.to_owned();
    o
}

/// Render both segments (each attempt from a fresh scene, its narrator reset,
/// through the kit's retry for the Darwin EPERM race); returns their frame
/// counts and timelines.
fn render_segments(
    out: &Path,
    narration: &[Rc<Narrator>; 2],
    stems: [Option<PathBuf>; 2],
    capture: Option<[PathBuf; 2]>,
) -> ([u64; 2], [ChapterTimeline; 2]) {
    let capture_of = |k: usize| capture.as_ref().map(|c| c[k].as_path());
    let finished_a: Rc<RefCell<Option<ChapterTimeline>>> = Rc::default();
    let (ra, _) = encode::retrying("segment A", || {
        narration[0].reset();
        let kit = kit_for(&narration[0], stems[0].clone());
        *finished_a.borrow_mut() = None;
        let shared = Rc::clone(&kit.finished);
        let r = fmn::render(&mut Flat { kit }, options(out, "a", capture_of(0)));
        *finished_a.borrow_mut() = shared.borrow().clone();
        r
    });
    let ra = ra.expect("segment A");
    let finished_b: Rc<RefCell<Option<ChapterTimeline>>> = Rc::default();
    let (rb, _) = encode::retrying("segment B", || {
        narration[1].reset();
        let kit = kit_for(&narration[1], stems[1].clone());
        let shared = Rc::clone(&kit.finished);
        let r = encode::render_camera(
            move |scene, base| {
                let rig = CameraRig::new(scene, base)?;
                let mut target = base.clone();
                target
                    .frame
                    .set_euler_angles(Some(0.6), Some(1.0), None)
                    .map_err(SceneError::from)?;
                Ok((Solid { kit, rig, target }, rig))
            },
            options(out, "b", capture_of(1)),
        );
        *finished_b.borrow_mut() = shared.borrow().clone();
        r
    });
    let rb = rb.expect("segment B");
    let ta = finished_a.borrow().clone().expect("A's timeline");
    let tb = finished_b.borrow().clone().expect("B's timeline");
    ([ra.artifact.frame_count, rb.artifact.frame_count], [ta, tb])
}

#[test]
fn a_two_route_chapter_masters_with_its_stem_spanning_the_cut() {
    assert!(
        std::process::Command::new("ffmpeg")
            .arg("-version")
            .output()
            .is_ok_and(|o| o.status.success()),
        "this test needs ffmpeg on PATH"
    );
    let root = std::env::temp_dir().join(format!("explainer_kit_s9_{}", std::process::id()));
    let narration = root.join("narration");
    std::fs::create_dir_all(&narration).unwrap();
    write_line(&narration.join("01_1.wav"), 1.2, 3);
    write_line(&narration.join("01_2.wav"), 1.0, 4);
    let script = Script::parse("01_1\tOne.\n01_2\tTwo.\n").unwrap();
    // Metadata for the synthetic speech above, not an ASR acceptance claim.
    std::fs::write(
        narration.join("report.json"),
        serde_json::json!({"meta": {"rate": 48_000}, "lines": {
            "01_1": [{"sentence": "One.", "out_samples": 57_600, "gap_after_samples": 0}],
            "01_2": [{"sentence": "Two.", "out_samples": 48_000, "gap_after_samples": 0}]
        }})
        .to_string(),
    )
    .unwrap();
    let narrators = || {
        [
            Narrator::load(&script, &narration, 0.0).unwrap(),
            Narrator::load(&script, &narration, 0.0).unwrap(),
        ]
    };

    // pass 1: each segment's timeline, appended into one chapter
    let pass1 = root.join("pass1");
    let ([fa, fb], [ta, tb]) = render_segments(&pass1, &narrators(), [None, None], None);
    assert_eq!((ta.frames, tb.frames), (fa, fb));
    let mut chapter = ta.clone();
    chapter.append(&tb).unwrap();
    assert_eq!(chapter.lines[1].start_frame, fa + tb.lines[0].start_frame);

    // the score, as one piece, sliced at the cut
    let section = Section {
        timeline: chapter.clone(),
        plan: ChapterPlan {
            intensity: 0.5,
            resolve: true,
        },
    };
    // 2.2 s of noise is no voice to measure a register from, and the route
    // segments are the point here: a chosen register
    let robert = explainer_score::theory::Register::ROBERT;
    let s = score::render_in(&Identity::hoeffding(), &[section], None, robert).unwrap();
    let stem = s.chapter_music(0);
    let cut = explainer_score::units::frame_to_sample(fa, 60) as usize;
    assert_eq!(stem.len(), chapter.samples());
    let slice = |a: usize, b: usize| Stereo {
        l: stem.l[a..b].to_vec(),
        r: stem.r[a..b].to_vec(),
    };
    let stems = [root.join("stem_a.f32.wav"), root.join("stem_b.f32.wav")];
    wav::write(&stems[0], &slice(0, cut), wav::Format::F32).unwrap();
    wav::write(&stems[1], &slice(cut, stem.len()), wav::Format::F32).unwrap();

    // pass 2: both segments with their slices, through the delivery encoder
    let pass2 = root.join("pass2");
    let captures = [pass2.join("soundtrack/a"), pass2.join("soundtrack/b")];
    let n = narrators();
    let (frames, _) = render_segments(
        &pass2,
        &n,
        [Some(stems[0].clone()), Some(stems[1].clone())],
        Some(captures.clone()),
    );
    assert_eq!(frames, [fa, fb], "the same scenes, the same lengths");

    // each segment replays bit for bit against the soundtrack fmn muxed
    let mut programme = Stereo::default();
    for (k, narrator) in n.iter().enumerate() {
        let cues = master::parse_cue_log(&narrator.cue_log()).unwrap();
        let replay = master::replay(&cues, frames[k], 60).unwrap();
        let captured = std::fs::read(captures[k].join("soundtrack_0.wav")).unwrap();
        let null = master::null_test(&replay, &captured);
        assert!(null.is_ok(), "segment {k} must null against its soundtrack: {null:?}");
        let audio = master::to_stereo(&replay);
        programme.l.extend(&audio.l);
        programme.r.extend(&audio.r);
    }
    // and the joined programme is the voice plus the whole stem: no seam
    let voice = chapter.voice_bus(None).unwrap();
    assert_eq!(programme.len(), voice.len());
    let worst = (0..programme.len())
        .map(|i| {
            let l = (programme.l[i] - voice.l[i] - stem.l[i]).abs();
            let r = (programme.r[i] - voice.r[i] - stem.r[i]).abs();
            l.max(r)
        })
        .fold(0.0_f64, f64::max);
    assert!(worst < 1e-5, "programme vs voice + stem: {worst:e}");
    // planted negative: the same check against the stem one frame late fails
    let late = (800..programme.len())
        .map(|i| (programme.l[i] - voice.l[i] - stem.l[i - 800]).abs())
        .fold(0.0_f64, f64::max);
    assert!(late > 1e-3, "a one-frame slip must show: {late:e}");

    // the two routes' video joins without re-encoding
    let wav_path = pass2.join("master.wav");
    wav::write_master(&wav_path, &programme, wav::Format::F32).unwrap();
    let out = pass2.join("master.mp4");
    master::join(
        &[pass2.join("a.mp4"), pass2.join("b.mp4")],
        &wav_path,
        &pass2.join("join"),
        &out,
    )
    .unwrap();
    master::check_timestamps(&out, 60, fa + fb).expect("every frame on its slot");
    let decode = std::process::Command::new("ffmpeg")
        .args(["-nostdin", "-v", "error", "-xerror", "-i"])
        .arg(&out)
        .args(["-f", "null", "-"])
        .output()
        .unwrap();
    assert!(
        decode.status.success() && decode.stderr.is_empty(),
        "decode: {}",
        String::from_utf8_lossy(&decode.stderr)
    );
    let (joined, secs) = master::probe(&out).unwrap();
    assert_eq!(joined, fa + fb);
    assert!((secs - (fa + fb) as f64 / 60.0).abs() <= 1024.0 / 48_000.0 + 1e-3);
}
