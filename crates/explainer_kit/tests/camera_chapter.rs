//! A chapter on the camera route through the shared CLI (plan §6.3): the
//! chapter asks for the camera in its `make` (`Kit::request_camera`), the CLI
//! renders it through the kit's camera wrapper with the retained route's
//! encoder, and the scene gets its rig, moves the camera and pins a HUD
//! count. A retained chapter in the same run is unaffected. Needs ffmpeg on
//! PATH (the renders are MP4).

use std::cell::Cell;
use std::rc::Rc;

use explainer_kit::kit::CameraShot;
use explainer_kit::{Chapter, Explainer, Facts, Kit, Palette, Timed, play, tween, v};
use explainer_score::score::Identity;
use fmn::library::Prism;
use fmn::prelude::*;

thread_local! {
    /// What the camera chapter saw: (a rig was bound, the camera moved).
    static SEEN: Cell<(bool, bool)> = const { Cell::new((false, false)) };
}

struct Cube {
    kit: Kit,
}

impl SceneConstruct for Cube {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let bound = kit.camera().is_some();
        stage.add(Prism::new(1.2, 1.2, 1.8).color(Palette::hero()))?;
        let count = stage.add_value_tracker(0.0);
        kit.live_count_hud(stage, count, 20.0, Palette::ink(), v(0.0, 1.5))?;
        let turn = kit.camera_to(stage, 0.9, 1.0)?;
        play!(stage; turn.rt(0.5), tween(count, 1_000.0, 0.5)?);
        SEEN.with(|s| s.set((bound, true)));
        kit.finish(stage)
    }
}

struct Flat {
    kit: Kit,
}

impl SceneConstruct for Flat {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        // off the camera route there is no rig, and a camera move is refused
        assert!(kit.camera().is_none() && kit.camera_shot().is_none());
        assert!(kit.camera_to(stage, 0.5, 0.5).is_err());
        let t = kit.text(stage, "flat", 20.0, Palette::ink(), v(0.0, 0.0))?;
        stage.play(write(stage.arena(), t))?;
        kit.finish(stage)
    }
}

fn chapters() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "01_flat",
            make: |kit| Box::new(Flat { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "02_cube",
            make: |kit| {
                kit.request_camera(CameraShot { theta: 0.0, phi: 0.0 });
                Box::new(Cube { kit })
            },
            intensity: 0.5,
        },
    ]
}

fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    f.pin("c_10_4", explainer_kit::choose(10, 4), "210", "math.comb")?;
    Ok(f)
}

fn frames(path: &std::path::Path) -> u64 {
    let probe = std::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-count_frames",
            "-select_streams",
            "v",
            "-show_entries",
            "stream=nb_read_frames,width,height",
            "-of",
            "csv=p=0",
        ])
        .arg(path)
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&probe.stdout).trim().to_owned();
    let fields: Vec<&str> = out.split(',').collect();
    assert_eq!(&fields[..2], ["64", "36"], "{}: {out}", path.display());
    fields[2].parse().unwrap()
}

#[test]
fn a_chapter_can_ask_for_the_camera_route() {
    assert!(
        std::process::Command::new("ffmpeg")
            .arg("-version")
            .output()
            .is_ok_and(|o| o.status.success()),
        "this test needs ffmpeg on PATH"
    );
    let out = std::env::temp_dir().join(format!("explainer_kit_camera_{}", std::process::id()));
    let e = Explainer {
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
        thumbnail: None,
    };
    let argv = format!("render --res 64x36 --fps 10 --silent --out {}", out.display());
    let code = explainer_kit::cli::run(&e, argv.split_whitespace().map(str::to_owned));
    assert_eq!(code, 0);
    assert_eq!(
        SEEN.with(Cell::get),
        (true, true),
        "the camera chapter got its rig and moved it"
    );
    // a half-second turn at 10 fps, then the kit's closing hold (none silent)
    assert!(frames(&out.join("02_cube.mp4")) >= 5);
    assert!(frames(&out.join("01_flat.mp4")) >= 1);
    // the same kit clone shares the camera slot (what the CLI relies on)
    let kit = Kit::new().unwrap();
    let shared = Rc::new(kit.clone());
    kit.request_camera(CameraShot { theta: 0.3, phi: 0.7 });
    assert_eq!(shared.camera_shot(), Some(CameraShot { theta: 0.3, phi: 0.7 }));
}
