//! Real Scene captures, portable cameras, and standalone-consumer pixel inputs.

use std::path::Path;
use std::sync::Arc;

use fmn::prelude::*;
use fmn::rendering::{RenderOptions, render_bundle_with_fs};
use fmn_codec::{PngLimits, decode_png};
use fmn_frame::convert::rgba16f_to_rgba8;
use fmn_frame::{FrameBuffer, FrameLayout, PixelFormat};
use fmn_hash::serial::{Limits, Reader, UnknownPolicy};
use fmn_hash::sha256;
use fmn_platform::fs::{FileSystem, VirtualFs};
use fmn_render::{
    Camera, CameraConfig, EngineIdentity, FrameConfig, RetainedFrameRenderer,
    RetainedFrameRendererConfig, ScreenMap, Tiling, Viewport,
};
use fmn_scene::recording::{RecordedSceneBundle, RecordingError, SceneBundleRecorder};
use fmn_scene::timeline_bundle::{BundleReadError, TIMELINE_BUNDLE_SCHEMA, TimelineFrameCache};
use fmn_scene::{BundleExportLimits, CameraRig, LifecycleEvent, TimelineBundle};

const RESOLUTION: (u32, u32) = (48, 32);

fn camera_config() -> CameraConfig {
    let mut config = CameraConfig {
        resolution: RESOLUTION,
        fps: 8,
        samples: 4,
        ..CameraConfig::default()
    };
    config.frame.set_width(5.0).unwrap();
    config.frame.set_orientation([1.0, 2.3, -4.7, 0.9]).unwrap();
    config
}

fn pixels(stage: &fmn_mobject::Stage, camera: &Camera) -> Vec<u8> {
    let mut renderer = RetainedFrameRenderer::new(RetainedFrameRendererConfig {
        frame: FrameConfig::new(
            Viewport {
                width: RESOLUTION.0,
                height: RESOLUTION.1,
            },
            ScreenMap {
                scale: 8.0,
                origin: [24.0, 16.0],
                y_up: true,
            },
            camera.background(),
        ),
        tiling: Tiling::default(),
        engine: EngineIdentity::certified(),
        threads: 1,
    })
    .unwrap();
    renderer.render_with_camera(stage, camera).unwrap();
    let mut frame = FrameBuffer::new(
        FrameLayout::tight(PixelFormat::Rgba8, RESOLUTION.0, RESOLUTION.1).unwrap(),
    );
    rgba16f_to_rgba8(renderer.frame(), &mut frame).unwrap();
    frame.as_bytes().to_vec()
}

struct Capture {
    recorder: SceneBundleRecorder,
    rig: CameraRig,
    base: CameraConfig,
    observed: Vec<Vec<u8>>,
}

impl SceneSink for Capture {
    fn event(&mut self, event: LifecycleEvent) -> Result<(), IntegrationError> {
        self.recorder.event(event)
    }

    fn capture(
        &mut self,
        reason: CaptureReason,
        packet: FramePacket,
    ) -> Result<(), IntegrationError> {
        let stage = packet.materialize_stage();
        let mut camera = Camera::new(self.rig.sample(&stage, &self.base).unwrap()).unwrap();
        // Authored background is part of the actual capture, not replay policy.
        if packet.time().frames() >= 3 {
            camera.set_background(RED.to_linear(1.0)).unwrap();
        }
        self.observed.push(pixels(&stage, &camera));
        self.recorder.capture_with_camera(reason, packet, &camera)
    }
}

fn recording() -> (RecordedSceneBundle, Vec<Vec<u8>>) {
    let base = camera_config();
    let mut scene = Scene::new(
        RuntimeConfig {
            fps: 8,
            ..RuntimeConfig::default()
        },
        19,
    )
    .unwrap();
    let rig = CameraRig::new(&mut scene, &base).unwrap();
    scene.add_mobject(Cube::new(1.4).color(BLUE)).unwrap();
    let marker = scene
        .add_mobject(Circle::new().radius(0.25).color(YELLOW))
        .unwrap();
    scene
        .stage_mut()
        .set_fill(marker, Some(YELLOW), Some(1.0), None, true);
    scene.stage_mut().shift(marker, [-1.1, 0.3, 0.2]);
    let mut target = base.clone();
    target.frame.set_orientation([0.1, 0.8, 0.2, 0.5]).unwrap();
    target.frame.set_center([0.25, -0.1, 0.0]).unwrap();
    target.frame.set_width(3.5).unwrap();
    target.frame.set_field_of_view(0.9).unwrap();
    target.light_source_position = [4.0, 3.0, 8.0];
    let animation = rig.animate_to(&mut scene, &target).unwrap();
    let mut sink = Capture {
        recorder: SceneBundleRecorder::new_render_only_with_camera(
            8,
            BundleExportLimits::default(),
        )
        .unwrap(),
        rig,
        base,
        observed: Vec::new(),
    };
    scene
        .play(
            vec![Box::new(animation)],
            PlayOverrides {
                run_time: Some(0.5),
                rate_func: Some(RateFunc::linear()),
                ..PlayOverrides::default()
            },
            &mut sink,
        )
        .unwrap();
    scene.show(&mut sink).unwrap();
    (sink.recorder.finish().unwrap(), sink.observed)
}

fn options(threads: u32, window: usize) -> RenderOptions {
    let mut options = RenderOptions::new("/movie").unwrap();
    options.config.camera.resolution = RESOLUTION;
    options.config.camera.fps = 8;
    options.config.render.threads = fmn_config::config::ThreadPolicy::Fixed(threads);
    options.config.determinism.mode = fmn_config::config::DeterminismMode::Certified;
    options.frames_in_flight = window;
    options
}

#[test]
fn moving_camera_light_background_and_depth_replay_through_native_workers() {
    let (artifact, expected) = recording();
    assert_eq!(artifact.frame_count, 5);
    assert_ne!(
        expected[0], expected[1],
        "static geometry must move through the camera"
    );
    assert_ne!(
        expected[1], expected[3],
        "later captures include the changed background"
    );
    for threads in [1, 4, 16] {
        for window in [1, 4] {
            let fs = Arc::new(VirtualFs::new());
            let report =
                render_bundle_with_fs(&artifact.bytes, options(threads, window), fs.clone())
                    .unwrap();
            let stats = report.frame_pipeline.as_ref().unwrap();
            assert_eq!(
                (stats.submitted, stats.emitted, stats.outstanding_slots),
                (5, 5, 0)
            );
            for (index, expected) in expected.iter().enumerate() {
                let bytes = fs
                    .read(&Path::new("/movie").join(format!("frame_{index:06}.png")))
                    .unwrap();
                assert_eq!(
                    &decode_png(&bytes, &PngLimits::default()).unwrap().rgba,
                    expected
                );
            }
        }
    }
}

#[test]
fn random_access_and_shared_cache_keep_camera_and_geometry_paired() {
    let (artifact, expected) = recording();
    let bundle = TimelineBundle::from_bytes(&artifact.bytes).unwrap();
    assert!(bundle.requires_camera());
    assert!(bundle.has_camera_track());
    assert!(matches!(
        bundle.stage_at(0),
        Err(BundleReadError::CameraTrackRequired)
    ));
    assert!(bundle.camera_at(5, RESOLUTION).is_err());
    assert!(bundle.camera_at(0, (0, 32)).is_err());
    for index in [4, 0, 3, 1, 2, 0] {
        let (stage, camera) = bundle.stage_at_with_camera(index, RESOLUTION).unwrap();
        let camera = camera.unwrap();
        assert_eq!(camera.revision(), u64::from(index) + 1);
        assert_eq!(pixels(&stage, &camera), expected[index as usize]);
        if index == 0 {
            assert_ne!(
                pixels(&stage, &Camera::new(camera_config()).unwrap()),
                expected[0]
            );
        }
    }
    let shared = bundle.into_shared().unwrap();
    let mut cache = TimelineFrameCache::default();
    for index in [4, 0, 3, 1, 2, 0] {
        let job = shared.frame_job(index).unwrap();
        assert!(job.has_camera_track());
        assert!(matches!(
            job.materialize(),
            Err(BundleReadError::CameraTrackRequired)
        ));
        let (stage, camera) = cache.materialize_with_camera(&job, RESOLUTION).unwrap();
        assert_eq!(pixels(&stage, &camera.unwrap()), expected[index as usize]);
    }
}

fn rehash(bytes: &mut [u8]) {
    let end = bytes.len() - 32;
    let digest = sha256(&bytes[..end]);
    bytes[end..].copy_from_slice(digest.as_bytes());
}

#[test]
fn versioned_camera_payload_rejects_old_consumers_and_corrupt_tracks() {
    let (artifact, _) = recording();
    assert_eq!(&artifact.bytes[10..12], &1u16.to_le_bytes());
    assert!(
        Reader::open(
            &artifact.bytes,
            TIMELINE_BUNDLE_SCHEMA,
            Limits::DEFAULT,
            UnknownPolicy::Strict
        )
        .is_err()
    );
    let start = artifact.bytes.len() - 32 - 5 * fmn_render::camera::CameraSample::WIRE_BYTES;
    for (offset, replacement) in [
        (start - 4, 4u32.to_le_bytes().to_vec()),
        (start - 4, u32::MAX.to_le_bytes().to_vec()),
        (start, 0u32.to_le_bytes().to_vec()),
        (start + 48, vec![0; 32]),
        (start + 80, 0f64.to_le_bytes().to_vec()),
    ] {
        let mut bytes = artifact.bytes.clone();
        bytes[offset..offset + replacement.len()].copy_from_slice(&replacement);
        rehash(&mut bytes);
        assert!(TimelineBundle::from_bytes(&bytes).is_err());
    }
    let mut newer = artifact.bytes.clone();
    newer[10..12].copy_from_slice(&2u16.to_le_bytes());
    rehash(&mut newer);
    assert!(TimelineBundle::from_bytes(&newer).is_err());
}

#[test]
fn terminal_stills_legacy_bytes_and_camera_mode_refusals_are_explicit() {
    let stage = fmn_mobject::Stage::new();
    let camera = Camera::new(camera_config()).unwrap();
    let legacy = || {
        let mut recorder =
            SceneBundleRecorder::new_render_only(8, BundleExportLimits::default()).unwrap();
        recorder.capture_terminal_still(&stage).unwrap();
        recorder.finish().unwrap()
    };
    let old = legacy();
    assert_eq!(old.bytes, legacy().bytes);
    assert_eq!(&old.bytes[10..12], &[0, 0]);
    let decoded = TimelineBundle::from_bytes(&old.bytes).unwrap();
    assert!(!decoded.has_camera_track());
    assert!(
        decoded
            .stage_at_with_camera(0, RESOLUTION)
            .unwrap()
            .1
            .is_none()
    );
    let build = |limit| {
        let mut recorder =
            SceneBundleRecorder::new_render_only_with_camera(8, BundleExportLimits::default())
                .unwrap();
        recorder
            .capture_terminal_still_with_camera(&stage, &camera)
            .unwrap();
        recorder.finish_with_max_bytes(limit)
    };
    let artifact = build(Limits::DEFAULT.max_total).unwrap();
    assert_eq!(build(artifact.bytes.len()).unwrap().bytes, artifact.bytes);
    assert!(matches!(
        build(artifact.bytes.len() - 1),
        Err(RecordingError::OutputLimit { .. })
    ));
    let mut missing =
        SceneBundleRecorder::new_render_only_with_camera(8, BundleExportLimits::default()).unwrap();
    assert!(missing.capture_terminal_still(&stage).is_err());
    assert!(
        missing
            .capture_terminal_still_with_camera(&stage, &camera)
            .is_err()
    );
    assert!(missing.finish().is_err());
    let mut wrong_mode = SceneBundleRecorder::new(8, BundleExportLimits::default()).unwrap();
    assert!(
        wrong_mode
            .capture_terminal_still_with_camera(&stage, &camera)
            .is_err()
    );
    assert!(wrong_mode.finish().is_err());
}

#[test]
fn camera_capture_budgets_and_explicit_replay_override_fail_without_publication() {
    let stage = fmn_mobject::Stage::new();
    let camera = Camera::new(camera_config()).unwrap();
    let mut limited = SceneBundleRecorder::new_render_only_with_camera(
        8,
        BundleExportLimits {
            max_frames: 1,
            max_capture_bytes: fmn_render::camera::CameraSample::WIRE_BYTES,
        },
    )
    .unwrap();
    assert!(
        limited
            .capture_terminal_still_with_camera(&stage, &camera)
            .is_err()
    );
    assert!(limited.finish().is_err());
    let (artifact, _) = recording();
    let fs = Arc::new(VirtualFs::new());
    let mut opts = options(1, 1);
    opts.camera = Some(opts.camera_config().unwrap());
    assert!(render_bundle_with_fs(&artifact.bytes, opts, fs.clone()).is_err());
    assert!(!fs.exists(Path::new("/movie")));
}

// Exercise Cargo's actual shipping CLI artifact through the existing Gauntlet.
fn cli_camera_scenario(
    ctx: &mut fmn_conformance::e2e::RunCtx,
) -> Result<fmn_conformance::e2e::RunOutcome, fmn_conformance::e2e::ScenarioError> {
    use fmn_conformance::e2e::{LogEvent, RunOutcome, ScenarioError, counters, spans};
    let error = |error: std::io::Error| ScenarioError::new(error.to_string());
    ctx.set_fps((8, 1));
    ctx.record_asset(
        "camera-bundle.scene.source",
        include_bytes!("camera_bundle.rs"),
    );
    ctx.event(LogEvent::new(spans::PREFLIGHT).field("mode", "recorded-camera"));
    let (artifact, expected) = recording();
    ctx.event(
        LogEvent::new(spans::SCENE_CONSTRUCT).field("frames", u64::from(artifact.frame_count)),
    );
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| ScenarioError::new(e.to_string()))?
        .as_nanos();
    let directory = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("camera-cli-{}-{nonce}", std::process::id()));
    std::fs::create_dir(&directory).map_err(error)?;
    let source = directory.join("camera.fmtl");
    std::fs::write(&source, &artifact.bytes).map_err(error)?;
    let destination = directory.join("output");
    let mut command = std::process::Command::new(env!("CARGO_BIN_FILE_FMN_CLI_fmn"));
    command
        .args([
            "--robot",
            "--reproducible",
            "--format",
            "png_sequence",
            "--resolution",
            "48x32",
            "--threads",
            "4",
            "--video_dir",
        ])
        .arg(&destination)
        .arg(&source)
        .arg("CapturedCamera")
        .env_clear()
        .env("PATH", "")
        .current_dir(&directory)
        .stdin(std::process::Stdio::null());
    if cfg!(windows)
        && let Some(root) = std::env::var_os("SystemRoot")
    {
        command.env("SystemRoot", root);
    }
    let output = command.output().map_err(error)?;
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(ScenarioError::new(format!(
            "native camera CLI failed: {:?} {} {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    let robot = String::from_utf8(output.stdout).map_err(|e| ScenarioError::new(e.to_string()))?;
    if robot.lines().count() != 1
        || !robot.contains("\"frames\":5")
        || !robot.contains("\"source\":\"compiled\"")
    {
        return Err(ScenarioError::new(
            "CLI did not report the recorded frame grid",
        ));
    }
    let sequence = destination.join("CapturedCamera");
    if !sequence.join("FMN_COMPLETE").is_file() {
        return Err(ScenarioError::new(
            "CLI did not atomically complete its sequence",
        ));
    }
    for (index, expected) in expected.iter().enumerate() {
        let png = std::fs::read(sequence.join(format!("frame_{index:06}.png"))).map_err(error)?;
        let actual = decode_png(&png, &PngLimits::default())
            .map_err(|e| ScenarioError::new(e.to_string()))?;
        if actual.rgba != *expected {
            return Err(ScenarioError::new(format!(
                "CLI lost captured camera at frame {index}"
            )));
        }
    }
    ctx.event(
        LogEvent::new(spans::RENDER_FRAME)
            .field("equal", true)
            .field("frames", 5u64),
    );
    ctx.counter(counters::FRAMES_RASTERIZED, 5);
    Ok(RunOutcome::ok()
        .with_artifact("camera.fmtl", artifact.bytes)
        .with_artifact("camera.frame", expected[0].clone())
        .with_counter("frames", 5))
}

#[test]
fn standalone_camera_bundle_fast_scenario() {
    use fmn_conformance::e2e::{
        Assertion, FieldPred, Invocation, LogExpect, Runner, ScenarioClass, ScenarioSpec,
        StructuralAssert, Surface, counters, spans,
    };
    let scenario = ScenarioSpec::new(
        "recording.camera.v1",
        ScenarioClass::ParityDrill,
        Surface::RustApi,
        Invocation::new(cli_camera_scenario),
    )
    .assertions(vec![
        Assertion::ExitCode(0),
        Assertion::Structural(StructuralAssert::ArtifactCountEq(2)),
        Assertion::Structural(StructuralAssert::NoEmptyArtifacts),
        Assertion::Structural(StructuralAssert::CounterEq("frames", 5)),
        Assertion::FileInventory(vec!["camera.fmtl".to_owned(), "camera.frame".to_owned()]),
        Assertion::NdjsonSchema,
    ])
    .logs(vec![
        LogExpect::span_present(spans::RENDER_FRAME, vec![FieldPred::bool_eq("equal", true)]),
        LogExpect::event_order(spans::SCENE_CONSTRUCT, spans::RENDER_FRAME),
        LogExpect::counter_ge(counters::FRAMES_RASTERIZED, 5),
    ]);
    let scratch = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let runner = Runner::new(
        scratch.join("camera_bundle_e2e_logs"),
        scratch.join("camera_bundle_e2e_goldens"),
        fmn_conformance::golden::Mode::Check,
    );
    let report = runner.run_gated(scenario, false);
    assert!(report.is_pass(), "{}", report.summary());
}
