//! A film's options hook governs the render on every route the shared CLI has: a draft and a
//! `--crf` delivery, each on the retained and the camera route, plus the thumbnail and the glyph
//! capture.
//!
//! The observable is `max_output_bytes`, because `encode::delivery_options` raises it. A hook that
//! ran before a delivery or camera render rebuilt its options would be overwritten, the render would
//! succeed, and this test would fail. A hook that runs last leaves one byte to write, so the render
//! is refused. The same command with a hook that changes nothing shows the route itself works.
//!
//! It names no field the pinned engine lacks, so it builds on the workspace's pin. Needs ffmpeg on
//! PATH (the renders are MP4).

use std::path::{Path, PathBuf};

use explainer_kit::kit::CameraShot;
use explainer_kit::{Chapter, Explainer, Facts, Kit, Palette, v};
use explainer_score::score::Identity;
use fmn::prelude::*;

fn change_nothing(_: &mut RenderOptions) {}

fn leave_one_byte(opts: &mut RenderOptions) {
    opts.max_output_bytes = 1;
}

struct Flat {
    kit: Kit,
}

impl SceneConstruct for Flat {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let t = kit.text(stage, "flat", 20.0, Palette::ink(), v(0.0, 0.0))?;
        stage.play(write(stage.arena(), t))?;
        kit.finish(stage)
    }
}

fn make_retained(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Flat { kit })
}

fn make_camera(kit: Kit) -> Box<dyn SceneConstruct> {
    kit.request_camera(CameraShot { theta: 0.0, phi: 0.0 });
    Box::new(Flat { kit })
}

fn only_retained() -> Vec<Chapter> {
    vec![Chapter {
        key: "01_retained",
        make: make_retained,
        intensity: 0.4,
    }]
}

fn only_camera() -> Vec<Chapter> {
    vec![Chapter {
        key: "02_camera",
        make: make_camera,
        intensity: 0.4,
    }]
}

fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    f.pin("c_10_4", explainer_kit::choose(10, 4), "210", "math.comb")?;
    Ok(f)
}

fn explainer(chapters: fn() -> Vec<Chapter>) -> Explainer {
    Explainer {
        crate_name: "toy",
        script: "",
        chapters,
        facts,
        glyph_text: &["flat"],
        glyph_tex: &[],
        glyph_mono: &[],
        identity: Identity::hoeffding,
        narration_gain_db: explainer_kit::narration::GAIN_DB,
        line_gaps: explainer_kit::narration::LineGaps::Constant,
        line_owner: None,
        thumbnail: Some(make_retained),
    }
}

fn fresh_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("explainer_kit_hook_{tag}_{}_{nanos}", std::process::id()))
}

fn run(chapters: fn() -> Vec<Chapter>, line: &str, topic: &Path, adjust: fn(&mut RenderOptions)) -> i32 {
    let argv = line.split_whitespace().map(str::to_owned);
    explainer_kit::cli::run_with_contract_and_options(&explainer(chapters), argv, topic, adjust)
}

/// The route works with the hook idle, and is refused with it starving the output.
fn governed(route: &str, chapters: fn() -> Vec<Chapter>, line: impl Fn(&Path) -> String) {
    let (open, tight) = (fresh_dir("open"), fresh_dir("tight"));
    assert_eq!(
        run(chapters, &line(&open), &open, change_nothing),
        0,
        "{route} should work"
    );
    assert_ne!(
        run(chapters, &line(&tight), &tight, leave_one_byte),
        0,
        "{route} rendered past the hook"
    );
}

#[test]
fn the_hook_governs_every_render_route() {
    assert!(
        std::process::Command::new("ffmpeg")
            .arg("-version")
            .output()
            .is_ok_and(|o| o.status.success()),
        "this test needs ffmpeg on PATH"
    );
    const DRAFT: &str = "--res 64x36 --fps 10 --silent";
    const DELIVERY: &str = "--res 64x36 --fps 10 --silent --crf 28 --preset ultrafast";
    for (route, chapters, flags) in [
        ("retained draft", only_retained as fn() -> Vec<Chapter>, DRAFT),
        ("camera draft", only_camera, DRAFT),
        ("retained delivery", only_retained, DELIVERY),
        ("camera delivery", only_camera, DELIVERY),
    ] {
        let chapter = chapters()[0].key;
        governed(route, chapters, |out| {
            format!("render {chapter} {flags} --out {}", out.display())
        });
    }
    governed("thumbnail", only_retained, |out| {
        format!("thumbnail --res 64x36 --out {}", out.display())
    });
    for (route, chapters) in [
        ("retained glyph capture", only_retained as fn() -> Vec<Chapter>),
        ("camera glyph capture", only_camera),
    ] {
        governed(route, chapters, |out| format!("glyphs {} --res 960x540", out.display()));
    }
}
