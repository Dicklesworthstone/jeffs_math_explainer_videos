//! The shared CLI end to end on a toy two-chapter explainer: the score's
//! two passes (`render --timeline-only`, `score`, `render --score`), the
//! fingerprint cache, the stem-length guard, and the facts self-check that
//! runs before anything. Needs ffmpeg on PATH (the renders are MP4).

use std::path::{Path, PathBuf};

use explainer_kit::{Chapter, Explainer, Facts, Kit, Palette, v};
use explainer_score::score::Identity;
use explainer_score::{Stereo, wav};
use fmn::prelude::*;

struct Toy {
    kit: Kit,
    lines: [&'static str; 2],
}

impl SceneConstruct for Toy {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, self.lines[0])?;
        let t = kit.text(stage, "toy", 30.0, Palette::ink(), v(0.0, 0.0))?;
        stage.play(write(stage.arena(), t))?;
        kit.say(stage, self.lines[1])?;
        kit.finish(stage)
    }
}

const SCRIPT: &str = "01_1\tOne.\n01_2\tTwo.\n02_1\tThree.\n02_2\tFour.\n";

fn chapters() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "01_first",
            make: |kit| {
                Box::new(Toy {
                    kit,
                    lines: ["01_1", "01_2"],
                })
            },
            intensity: 0.4,
        },
        Chapter {
            key: "02_second",
            make: |kit| {
                Box::new(Toy {
                    kit,
                    lines: ["02_1", "02_2"],
                })
            },
            intensity: 0.6,
        },
    ]
}

fn good_facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    f.pin("c_10_4", explainer_kit::choose(10, 4), "210", "math.comb")?;
    Ok(f.clone())
}

fn bad_facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    f.pin("c_10_4", explainer_kit::choose(10, 4), "211", "a typo")?;
    Ok(f.clone())
}

fn explainer(facts: fn() -> Result<Facts, String>) -> Explainer {
    Explainer {
        crate_name: "toy",
        script: SCRIPT,
        chapters,
        facts,
        glyph_text: &["toy"],
        glyph_tex: &[r"x^2"],
        glyph_mono: &["0x1D"],
        identity: Identity::hoeffding,
        narration_gain_db: explainer_kit::narration::GAIN_DB,
        line_owner: None,
        thumbnail: None,
    }
}

/// A thumbnail: one picture and a title, held for the one captured frame.
struct ToyThumbnail {
    kit: Kit,
}

impl SceneConstruct for ToyThumbnail {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let disc = Circle::new()
            .radius(2.0)
            .style(Style::default().fill(Palette::hero(), 1.0))
            .build();
        stage.add(disc)?;
        let title = self.kit.text(stage, "toy", 60.0, Palette::ink(), v(0.0, -3.0))?;
        stage.add_to_scene(title)?;
        stage.wait(1.0)?;
        Ok(())
    }
}

/// Width and height from a PNG's IHDR chunk.
fn png_size(path: &Path) -> (u32, u32) {
    let b = std::fs::read(path).unwrap();
    assert_eq!(&b[..8], b"\x89PNG\r\n\x1a\n", "{} is a PNG", path.display());
    let be = |i: usize| u32::from_be_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
    (be(16), be(20))
}

#[test]
fn the_thumbnail_is_one_png_at_the_asked_size() {
    let out = std::env::temp_dir().join(format!("explainer_kit_thumbnail_{}", std::process::id()));
    let mut e = explainer(good_facts);
    let argv = |s: &str| args(&format!("{s} --out {}", out.display()));
    // a video without a thumbnail scene says so
    assert_ne!(explainer_kit::cli::run(&e, argv("thumbnail --res 320x180")), 0);
    e.thumbnail = Some(|kit| Box::new(ToyThumbnail { kit }));
    assert_eq!(explainer_kit::cli::run(&e, argv("thumbnail --res 320x180")), 0);
    let png = out.join("toy_thumbnail.png");
    assert_eq!(png_size(&png), (320, 180));
    // a second run moves the first aside instead of overwriting it
    assert_eq!(explainer_kit::cli::run(&e, argv("thumbnail --res 640x360")), 0);
    assert_eq!(png_size(&png), (640, 360));
    assert!(
        out.join("previous/toy_thumbnail.v1.png").is_file(),
        "the first thumbnail kept"
    );
}

fn args(s: &str) -> Vec<String> {
    s.split_whitespace().map(str::to_owned).collect()
}

/// A speech-like line: 2.5 s of pink noise low-passed near 4 kHz (roughly
/// the long-term spectrum of speech: white noise would put most of its
/// energy where speech has little, and the SII-lite gate would rightly judge
/// it masked) with a 4 Hz syllable envelope, as a 48 kHz f32 file.
fn write_line(path: &Path, seed: u64) {
    let mut pink = explainer_score::osc::Pink::new(seed);
    let mut lp = explainer_score::filter::OnePole::new(4_000.0);
    let n = (2.5 * 48_000.0) as usize;
    let m: Vec<f64> = (0..n)
        .map(|i| {
            let t = i as f64 / 48_000.0;
            0.3 * (0.6 + 0.4 * (std::f64::consts::TAU * 4.0 * t).sin()) * lp.lp(pink.sample())
        })
        .collect();
    wav::write(path, &Stereo::from_mono(&m), wav::Format::F32).unwrap();
}

#[test]
fn the_two_pass_score_runs_end_to_end_through_the_cli() {
    assert!(
        std::process::Command::new("ffmpeg")
            .arg("-version")
            .output()
            .is_ok_and(|o| o.status.success()),
        "this test needs ffmpeg on PATH"
    );
    let root = std::env::temp_dir().join(format!("explainer_kit_cli_{}", std::process::id()));
    let narration = root.join("narration");
    let out = root.join("out");
    std::fs::create_dir_all(&narration).unwrap();
    for (k, id) in ["01_1", "01_2", "02_1", "02_2"].iter().enumerate() {
        write_line(&narration.join(format!("{id}.wav")), k as u64 + 1);
    }
    let e = explainer(good_facts);
    let o = out.display().to_string();
    let n = narration.display().to_string();

    // a wrong pinned fact stops every command before it starts
    assert_eq!(explainer_kit::cli::run(&explainer(bad_facts), args("chapters")), 1);
    assert_eq!(explainer_kit::cli::run(&e, args("chapters")), 0);
    assert_eq!(
        explainer_kit::cli::run(&e, args("render --fsp 30")),
        2,
        "a typo is a usage error"
    );

    // pass 1: timelines
    assert_eq!(
        explainer_kit::cli::run(&e, args(&format!("render --timeline-only --narration {n} --out {o}"))),
        0
    );
    for key in ["01_first", "02_second"] {
        assert!(out.join(format!("{key}.timeline.tsv")).exists(), "{key}");
        assert!(out.join(format!("{key}.cuelog.tsv")).exists());
    }

    // the score, its QA, and the cache
    assert_eq!(
        explainer_kit::cli::run(&e, args(&format!("score --out {o}"))),
        0,
        "QA passes"
    );
    let stems = out.join("stems");
    for key in ["01_first", "02_second"] {
        let tl = explainer_score::timeline::ChapterTimeline::read(&out.join(format!("{key}.timeline.tsv"))).unwrap();
        let d = wav::read(&stems.join(format!("{key}.music.f32.wav"))).unwrap();
        assert_eq!(d.frames(), tl.samples(), "{key}: the stem is the scene's length");
        assert!(stems.join(format!("qa/{key}.json")).exists());
    }
    let fp = std::fs::read_to_string(stems.join("score.fingerprint")).unwrap();
    assert_eq!(
        explainer_kit::cli::run(&e, args(&format!("score --out {o}"))),
        0,
        "up to date"
    );
    assert_eq!(std::fs::read_to_string(stems.join("score.fingerprint")).unwrap(), fp);
    assert!(!stems.join("previous").exists(), "a cached score rewrites nothing");

    // pass 2: the chapters with their stems
    let s = stems.display().to_string();
    assert_eq!(
        explainer_kit::cli::run(
            &e,
            args(&format!("render --res 64x36 --narration {n} --score {s} --out {o}"))
        ),
        0
    );
    let log = std::fs::read_to_string(out.join("01_first.cuelog.tsv")).unwrap();
    assert_eq!(log.lines().count(), 1 + 3, "two lines and the stem: {log}");
    assert!(
        log.lines().last().unwrap().ends_with("\t0"),
        "the stem starts at sample 0"
    );
    let probe = std::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "a",
            "-show_entries",
            "stream=codec_name",
            "-of",
            "csv=p=0",
        ])
        .arg(out.join("01_first.mp4"))
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&probe.stdout).trim(), "aac");

    // a stem scored for another chapter is refused, not stretched
    let wrong = root.join("wrong");
    std::fs::create_dir_all(&wrong).unwrap();
    let silent = Stereo::silence(1_000);
    wav::write(&wrong.join("01_first.music.f32.wav"), &silent, wav::Format::F32).unwrap();
    let w = wrong.display().to_string();
    let wout: PathBuf = root.join("wrong_out");
    assert_eq!(
        explainer_kit::cli::run(
            &e,
            args(&format!(
                "render 01_first --res 64x36 --narration {n} --score {w} --out {}",
                wout.display()
            ))
        ),
        1
    );
    assert!(!wout.join("01_first.mp4").exists(), "nothing published");
}

#[test]
fn the_pcm_master_replays_nulls_and_joins() {
    assert!(
        std::process::Command::new("ffmpeg")
            .arg("-version")
            .output()
            .is_ok_and(|o| o.status.success()),
        "this test needs ffmpeg on PATH"
    );
    let root = std::env::temp_dir().join(format!("explainer_kit_master_e2e_{}", std::process::id()));
    let narration = root.join("narration");
    let out = root.join("out");
    std::fs::create_dir_all(&narration).unwrap();
    for (k, id) in ["01_1", "01_2", "02_1", "02_2"].iter().enumerate() {
        write_line(&narration.join(format!("{id}.wav")), k as u64 + 11);
    }
    let e = explainer(good_facts);
    let (o, n) = (out.display().to_string(), narration.display().to_string());
    let run = |s: &str| explainer_kit::cli::run(&e, args(s));
    assert_eq!(run(&format!("render --timeline-only --narration {n} --out {o}")), 0);
    assert_eq!(run(&format!("score --out {o}")), 0);
    let s = out.join("stems").display().to_string();
    // a delivery render (x264 via the quality runner) captures each soundtrack
    assert_eq!(
        run(&format!(
            "render --res 64x36 --crf 20 --preset ultrafast --narration {n} --score {s} --out {o}"
        )),
        0
    );
    assert!(out.join("soundtrack/01_first/soundtrack_0.wav").exists());
    // the master: replays null against fmn's own soundtracks, joins, gates
    assert_eq!(run(&format!("master --out {o}")), 0);
    let master = out.join("toy_master.mp4");
    let probe = |entries: &str, stream: &str| {
        let o = std::process::Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                stream,
                "-count_packets",
                "-show_entries",
                entries,
                "-of",
                "csv=p=0",
            ])
            .arg(&master)
            .output()
            .unwrap();
        String::from_utf8_lossy(&o.stdout).trim().to_owned()
    };
    let frames: u64 = probe("stream=nb_read_packets", "v:0").parse().unwrap();
    let chapter_frames: u64 = ["01_first", "02_second"]
        .iter()
        .map(|k| {
            explainer_score::timeline::ChapterTimeline::read(&out.join(format!("{k}.timeline.tsv")))
                .unwrap()
                .frames
        })
        .sum();
    assert_eq!(frames, chapter_frames, "the join keeps every frame");
    assert_eq!(probe("stream=codec_name", "a:0"), "aac");
    explainer_kit::master::check_timestamps(&master, 60, chapter_frames).expect("every frame on its slot");
    // verify_video.sh's decode gate: the whole file decodes, video and
    // audio, without a single complaint
    let decode = std::process::Command::new("ffmpeg")
        .args(["-nostdin", "-v", "error", "-xerror", "-i"])
        .arg(&master)
        .args(["-f", "null", "-"])
        .output()
        .unwrap();
    assert!(
        decode.status.success() && decode.stderr.is_empty(),
        "decode: {}",
        String::from_utf8_lossy(&decode.stderr)
    );
    // planted negative: the raw Annex-B join this replaced loses the
    // presentation times of B-frames, and the timestamp gate must see it
    let raw = root.join("annexb");
    std::fs::create_dir_all(&raw).unwrap();
    let mut bytes = Vec::new();
    for k in ["01_first", "02_second"] {
        let part = raw.join(format!("{k}.h264"));
        let ok = std::process::Command::new("ffmpeg")
            .args(["-nostdin", "-v", "error", "-y", "-i"])
            .arg(out.join(format!("{k}.mp4")))
            .args([
                "-map",
                "0:v:0",
                "-c:v",
                "copy",
                "-bsf:v",
                "h264_mp4toannexb",
                "-f",
                "h264",
            ])
            .arg(&part)
            .status()
            .unwrap();
        assert!(ok.success());
        bytes.extend(std::fs::read(&part).unwrap());
    }
    std::fs::write(raw.join("all.h264"), bytes).unwrap();
    let broken = raw.join("broken.mp4");
    let ok = std::process::Command::new("ffmpeg")
        .args(["-nostdin", "-v", "error", "-y", "-framerate", "60", "-f", "h264", "-i"])
        .arg(raw.join("all.h264"))
        .args(["-c:v", "copy"])
        .arg(&broken)
        .status()
        .unwrap();
    assert!(ok.success());
    assert!(
        explainer_kit::master::check_timestamps(&broken, 60, chapter_frames).is_err(),
        "the Annex-B join's timestamps are refused"
    );
    // planted negatives: a changed narration file, a missing capture
    write_line(&narration.join("02_2.wav"), 99);
    assert_eq!(run(&format!("master --out {o}")), 1, "a cue changed since the render");
    write_line(&narration.join("02_2.wav"), 14); // the original bytes again (same seed)
    std::fs::rename(out.join("soundtrack/02_second"), root.join("moved_capture")).unwrap();
    assert_eq!(
        run(&format!("master --out {o}")),
        1,
        "no captured soundtrack to null against"
    );
}
