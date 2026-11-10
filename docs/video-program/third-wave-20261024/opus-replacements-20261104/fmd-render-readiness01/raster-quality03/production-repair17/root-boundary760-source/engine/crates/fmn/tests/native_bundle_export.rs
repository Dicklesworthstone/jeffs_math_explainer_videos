//! Real SceneConstruct -> FMTL -> native pixels, including publication failures.
use std::cell::Cell;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

use fmn::prelude::*;
use fmn::rendering::render_bundle_with_fs;
use fmn_config::config::{DeterminismMode, ThreadPolicy};
use fmn_platform::fs::{FileSystem, VirtualFs};
use fmn_scene::TimelineBundle;

fn options() -> BundleExportOptions {
    let mut options = BundleExportOptions::new().unwrap();
    options.config.camera.fps = 10;
    options.config.camera.resolution = (64, 48);
    options.config.sizes.frame_height = 4.0;
    options.config.determinism.mode = DeterminismMode::Certified;
    options.config.render.threads = ThreadPolicy::Fixed(1);
    options
}

fn render_options(path: &str, config: &Config, threads: u32) -> RenderOptions {
    let mut options = RenderOptions::new(path).unwrap();
    options.config = config.clone();
    options.config.render.threads = ThreadPolicy::Fixed(threads);
    options
}

struct StatefulScene(Rc<Cell<usize>>);

impl SceneConstruct for StatefulScene {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let circle = stage.add(Circle::new().radius(0.4).color(WHITE))?;
        stage.set_fill(circle, Some(WHITE), Some(1.0), None, true);
        let follower = stage.add(Dot::new().radius(0.12).color(BLUE))?;
        let counter = self.0.clone();
        stage.add_updater(
            follower,
            move |stage, me| {
                counter.set(counter.get() + 1);
                let center = stage.get_center(circle);
                stage.set_x(me, center[0] * center[0] - 1.0);
                stage.set_y(me, 0.7);
            },
            false,
        )?;
        // A callable outside the rate catalog must remain fully supported.
        let animation = circle
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.2),
                rate_func: Some(|alpha| alpha * alpha),
                ..AnimateArgs::default()
            })?
            .shift(RIGHT)?;
        stage.play(animation)?;
        stage.wait(0.1)?;
        Ok(())
    }
}

#[test]
fn exported_stateful_scene_replays_identical_png_bytes_at_different_thread_counts() {
    let fs = Arc::new(VirtualFs::new());
    let config = options().config;
    let recorded_calls = Rc::new(Cell::new(0));
    let mut program = StatefulScene(recorded_calls.clone());
    // Exercise the public trait-object boundary as well as ordinary generics.
    let receipt = export_bundle_with_fs(
        &mut program as &mut dyn SceneConstruct,
        "/scene.fmtl",
        options(),
        fs.clone(),
    )
    .unwrap();
    let bytes = fs.read(Path::new("/scene.fmtl")).unwrap();
    let bundle = TimelineBundle::from_bytes(&bytes).unwrap();
    assert_eq!(receipt.frame_count, bundle.frame_count());
    assert_eq!(receipt.bytes, bytes.len());
    assert_eq!(receipt.scene.play_count, 2);
    let calls_at_export = recorded_calls.get();
    assert!(calls_at_export > 0);
    let direct = render_with_fs(
        &mut StatefulScene(Rc::new(Cell::new(0))),
        render_options("/direct", &config, 1),
        fs.clone(),
    )
    .unwrap();
    assert_eq!(direct.artifact.frame_count, u64::from(receipt.frame_count));
    for threads in [1, 4] {
        let output = format!("/replayed-{threads}");
        let replay = render_bundle_with_fs(
            &bytes,
            render_options(&output, &config, threads),
            fs.clone(),
        )
        .unwrap();
        assert_eq!(replay.artifact.frame_count, direct.artifact.frame_count);
        for frame in 0..receipt.frame_count {
            let name = format!("frame_{frame:06}.png");
            assert_eq!(
                fs.read(&Path::new("/direct").join(&name)).unwrap(),
                fs.read(&Path::new(&output).join(&name)).unwrap(),
            );
        }
    }
    assert_eq!(
        recorded_calls.get(),
        calls_at_export,
        "replay cannot invoke source callbacks"
    );
    let again = export_bundle_bytes(&mut StatefulScene(Rc::new(Cell::new(0))), options()).unwrap();
    assert_eq!(again.bundle.bytes, bytes);
    assert_eq!(again.bundle.digest.to_hex(), receipt.digest);
}

struct StaticScene;

impl SceneConstruct for StaticScene {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add(Circle::new().radius(0.5))?;
        Ok(())
    }
}

#[test]
fn static_scene_has_one_replayable_still_without_advancing_scene_time() {
    let artifact = export_bundle_bytes(&mut StaticScene, options()).unwrap();
    assert_eq!(artifact.bundle.frame_count, 1);
    assert_eq!(artifact.scene.time.frames(), 0);
    assert_eq!(artifact.scene.play_count, 0);
    let bundle = TimelineBundle::from_bytes(&artifact.bundle.bytes).unwrap();
    assert!(!bundle.stage_at(0).unwrap().roots().is_empty());
}

struct FailingScene;

impl SceneConstruct for FailingScene {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.wait(0.1)?;
        Err(SceneError::InvalidLifecycle("deliberate post-capture failure").into())
    }
}

#[test]
fn scene_errors_and_output_budgets_never_publish_partial_files() {
    let fs = Arc::new(VirtualFs::new());
    let failure = export_bundle_with_fs(&mut FailingScene, "/failed.fmtl", options(), fs.clone());
    let error = failure.unwrap_err();
    assert!(matches!(
        error,
        BundleExportError::Scene(fmn::Error::Scene(SceneError::InvalidLifecycle(
            "deliberate post-capture failure"
        )))
    ));
    assert!(!fs.exists(Path::new("/failed.fmtl")));
    let mut limited = options();
    limited.max_output_bytes = 1;
    let result = export_bundle_with_fs(&mut StaticScene, "/large.fmtl", limited, fs.clone());
    assert!(matches!(
        result,
        Err(BundleExportError::OutputLimit { limit: 1, .. })
    ));
    assert!(!fs.exists(Path::new("/large.fmtl")));
}

#[test]
fn publication_never_overwrites_an_existing_destination() {
    let fs = Arc::new(VirtualFs::new());
    fs.write_atomic(Path::new("/occupied.fmtl"), b"existing project data")
        .unwrap();
    let result = export_bundle_with_fs(&mut StaticScene, "/occupied.fmtl", options(), fs.clone());
    assert!(matches!(result, Err(BundleExportError::FileSystem(_))));
    assert_eq!(
        fs.read(Path::new("/occupied.fmtl")).unwrap(),
        b"existing project data"
    );
}

struct SwallowedCaptureFailure;

impl SceneConstruct for SwallowedCaptureFailure {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let _ = stage.wait(0.5);
        Ok(())
    }
}

#[test]
fn swallowed_sink_failure_still_refuses_the_whole_export() {
    let fs = Arc::new(VirtualFs::new());
    let mut options = options();
    options.limits.max_frames = 1;
    let result = export_bundle_with_fs(
        &mut SwallowedCaptureFailure,
        "/partial.fmtl",
        options,
        fs.clone(),
    );
    assert!(matches!(result, Err(BundleExportError::Recording(_))));
    assert!(!fs.exists(Path::new("/partial.fmtl")));
}

struct AudioScene;

impl SceneConstruct for AudioScene {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.scene_mut().add_sound("sound.wav", 0.0, None, None)?;
        stage.wait(0.1)?;
        Ok(())
    }
}

#[test]
fn unrepresentable_audio_is_an_explicit_capability_error() {
    let fs = Arc::new(VirtualFs::new());
    let result = export_bundle_with_fs(&mut AudioScene, "/audio.fmtl", options(), fs.clone());
    assert!(matches!(result, Err(BundleExportError::Capability(_))));
    assert!(!fs.exists(Path::new("/audio.fmtl")));
}

struct MustNotRun;

impl SceneConstruct for MustNotRun {
    fn construct(&mut self, _: &mut Stage<'_>) -> fmn::Result<()> {
        panic!("invalid options must be refused before user code");
    }
}

#[test]
fn invalid_options_are_preflighted_and_unwinding_publishes_nothing() {
    for field in 0..4 {
        let mut options = options();
        match field {
            0 => options.config.camera.fps = 0,
            1 => options.limits.max_frames = 0,
            2 => options.limits.max_capture_bytes = 0,
            _ => options.max_output_bytes = 0,
        }
        assert!(matches!(
            export_bundle_bytes(&mut MustNotRun, options),
            Err(BundleExportError::InvalidOptions(_))
        ));
    }
    let fs = Arc::new(VirtualFs::new());
    let invalid = export_bundle_with_fs(&mut MustNotRun, "", options(), fs.clone());
    assert!(matches!(invalid, Err(BundleExportError::InvalidOptions(_))));
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        export_bundle_with_fs(&mut MustNotRun, "/panic.fmtl", options(), fs.clone())
    }));
    assert!(panic.is_err());
    assert!(!fs.exists(Path::new("/panic.fmtl")));
}

struct EndsEarly;

impl SceneConstruct for EndsEarly {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.wait(0.1)?;
        stage.end()
    }
}

#[test]
fn normal_early_scene_termination_keeps_its_completed_frames() {
    let export = export_bundle_bytes(&mut EndsEarly, options()).unwrap();
    assert!(export.scene.ended_early);
    assert_eq!(export.scene.play_count, 1);
    assert_eq!(
        i64::from(export.bundle.frame_count),
        export.scene.time.frames()
    );
    assert!(export.bundle.frame_count > 0);
}
