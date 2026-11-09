//! Animated native cameras keep the same clock, snapshots and bounded pipeline.

use std::cell::Cell;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use fmn::prelude::*;
use fmn::rendering::{CameraConfig, NativeFramePipeline, render_camera_with_fs};
use fmn_codec::{PngLimits, decode_png};
use fmn_frame::convert::rgba16f_to_rgba8;
use fmn_frame::{FrameBuffer, FrameLayout, PixelFormat};
use fmn_output::{EmitterConfig, OrderedEmitter, SinkBinding, SinkWrite};
use fmn_platform::fs::{FileSystem, VirtualFs};
use fmn_platform::topology::HardwareTopology;
use fmn_render::{
    Camera, EngineIdentity, FrameConfig, RetainedFrameRenderer, RetainedFrameRendererConfig,
    ScreenMap, Tiling, Viewport,
};
use fmn_runtime::{ExecutionPlan, OutputPixelFormat, PlanRequest, RenderIntent, SurfaceSpec};
use fmn_scene::CameraRig;

fn options(path: &str, window: usize) -> RenderOptions {
    let mut options = RenderOptions::new(path).unwrap();
    options.config.camera.resolution = (48, 32);
    options.config.camera.fps = 8;
    options.config.sizes.frame_height = 4.0;
    options.config.render.threads = fmn_config::config::ThreadPolicy::Fixed(4);
    options.config.determinism.mode = fmn_config::config::DeterminismMode::Certified;
    options.frames_in_flight = window;
    options
}

fn config(plan: &ExecutionPlan, camera: &CameraConfig) -> RetainedFrameRendererConfig {
    RetainedFrameRendererConfig {
        frame: FrameConfig::new(
            Viewport {
                width: 48,
                height: 32,
            },
            ScreenMap {
                scale: 8.0,
                origin: [24.0, 16.0],
                y_up: true,
            },
            camera.background,
        ),
        tiling: Tiling {
            macro_tile: plan.macro_tile,
            fine_tile: plan.fine_tile,
        },
        engine: EngineIdentity::certified(),
        threads: 1,
    }
}

fn populate(scene: &mut Scene) -> fmn::Result<()> {
    scene.add_mobject(Cube::new(1.2).color(BLUE))?;
    let circle = scene.add_mobject(Circle::new().radius(0.25).color(WHITE))?;
    scene
        .stage_mut()
        .set_fill(circle, Some(WHITE), Some(1.0), None, true);
    scene.stage_mut().shift(circle, [-1.5, 0.0, 0.0]);
    Ok(())
}

fn target(base: &CameraConfig) -> CameraConfig {
    let mut target = base.clone();
    target
        .frame
        .set_euler_angles(Some(0.6), Some(0.8), None)
        .unwrap();
    target.frame.set_center([0.2, -0.1, 0.0]).unwrap();
    target.frame.set_width(base.frame.width() * 0.85).unwrap();
    target.frame.set_field_of_view(0.9).unwrap();
    target.light_source_position = [5.0, 4.0, 8.0];
    target
}

struct Orbit {
    rig: CameraRig,
    target: CameraConfig,
}
impl SceneConstruct for Orbit {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        populate(stage.scene_mut())?;
        let animation = self.rig.animate_to(stage.scene_mut(), &self.target)?;
        stage.play_prepared_with(
            vec![Box::new(animation)],
            PlayOverrides {
                run_time: Some(0.5),
                rate_func: Some(RateFunc::linear()),
                ..PlayOverrides::default()
            },
        )?;
        Ok(())
    }
}

fn factory(scene: &mut Scene, base: &CameraConfig) -> fmn::Result<(Orbit, CameraRig)> {
    let rig = CameraRig::new(scene, base)?;
    Ok((
        Orbit {
            rig,
            target: target(base),
        },
        rig,
    ))
}

struct Serial {
    rig: CameraRig,
    base: CameraConfig,
    config: RetainedFrameRendererConfig,
    images: Vec<Vec<u8>>,
}
impl SceneSink for Serial {
    fn capture(&mut self, _: CaptureReason, packet: FramePacket) -> Result<(), IntegrationError> {
        let stage = packet.materialize_stage();
        let camera = Camera::new(self.rig.sample(&stage, &self.base).unwrap()).unwrap();
        // Fresh live retained renderer: no pipeline and no projection cache
        // shared across poses, so a stale-camera implementation cannot agree.
        let mut renderer = RetainedFrameRenderer::new(self.config).unwrap();
        renderer.render_with_camera(&stage, &camera).unwrap();
        let mut pixels = FrameBuffer::new(FrameLayout::tight(PixelFormat::Rgba8, 48, 32).unwrap());
        rgba16f_to_rgba8(renderer.frame(), &mut pixels).unwrap();
        self.images.push(pixels.as_bytes().to_vec());
        Ok(())
    }
}

fn images(fs: &VirtualFs, path: &str, count: u64) -> Vec<Vec<u8>> {
    (0..count)
        .map(|index| {
            decode_png(
                &fs.read(&Path::new(path).join(format!("frame_{index:06}.png")))
                    .unwrap(),
                &PngLimits::default(),
            )
            .unwrap()
            .rgba
        })
        .collect()
}

#[test]
fn camera_orbit_zoom_fov_and_light_match_serial_snapshots_at_every_window() {
    let mut first = None;
    for window in [1, 2, 4] {
        let fs = Arc::new(VirtualFs::new());
        let options = options("/orbit", window);
        let base = options.camera_config().unwrap();
        let mut scene = Scene::new(
            RuntimeConfig::from_config(&options.config),
            options.config.determinism.seed,
        )
        .unwrap();
        let rig = CameraRig::new(&mut scene, &base).unwrap();
        populate(&mut scene).unwrap();
        let animation = rig.animate_to(&mut scene, &target(&base)).unwrap();
        let report = render_camera_with_fs(factory, options, fs.clone()).unwrap();
        let stats = report.frame_pipeline.as_ref().unwrap();
        assert_eq!(
            (stats.submitted, stats.emitted, stats.outstanding_slots),
            (4, 4, 0)
        );
        assert!(stats.max_in_flight <= report.execution_plan.frames_in_flight);
        assert_eq!(report.scene.play_count, 1);
        assert_eq!(report.scene.time.frames(), 4);
        let mut serial = Serial {
            rig,
            base: base.clone(),
            config: config(&report.execution_plan, &base),
            images: Vec::new(),
        };
        scene
            .play(
                vec![Box::new(animation)],
                PlayOverrides {
                    run_time: Some(0.5),
                    rate_func: Some(RateFunc::linear()),
                    ..PlayOverrides::default()
                },
                &mut serial,
            )
            .unwrap();
        let got = images(&fs, "/orbit", 4);
        assert_eq!(got, serial.images);
        assert_ne!(got[0], got[3], "the camera, not the geometry, must move");
        if let Some(first) = &first {
            assert_eq!(&got, first);
        } else {
            first = Some(got);
        }
    }
}

#[test]
fn non_send_camera_updaters_execute_on_scene_owner_and_affect_the_same_capture() {
    struct Wait;
    impl SceneConstruct for Wait {
        fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
            populate(stage.scene_mut())?;
            stage.wait(0.5)?;
            Ok(())
        }
    }
    let fs = Arc::new(VirtualFs::new());
    let count = Rc::new(Cell::new(0));
    let seen = count.clone();
    let owner = std::thread::current().id();
    let options = options("/updater", 4);
    let base = options.camera_config().unwrap();
    let report = render_camera_with_fs(
        move |scene, base| {
            let rig = CameraRig::new(scene, base)?;
            scene.stage_mut().add_dt_updater(
                rig.center()[0],
                move |stage, mob, dt| {
                    assert_eq!(std::thread::current().id(), owner);
                    if dt > 0.0 {
                        seen.set(seen.get() + 1);
                    }
                    let value = stage.tracker_value(mob).unwrap();
                    stage.set_tracker_value(mob, value + dt).unwrap();
                },
                false,
            )?;
            Ok((Wait, rig))
        },
        options,
        fs.clone(),
    )
    .unwrap();
    assert_eq!(count.get(), 4);
    let mut expected = Vec::new();
    let mut scene = Scene::default();
    populate(&mut scene).unwrap();
    for index in 1..=4 {
        let mut pose = base.clone();
        pose.frame
            .set_center([f64::from(index) / 8.0, 0.0, 0.0])
            .unwrap();
        let mut renderer =
            RetainedFrameRenderer::new(config(&report.execution_plan, &base)).unwrap();
        renderer
            .render_with_camera(scene.stage(), &Camera::new(pose).unwrap())
            .unwrap();
        let mut pixels = FrameBuffer::new(FrameLayout::tight(PixelFormat::Rgba8, 48, 32).unwrap());
        rgba16f_to_rgba8(renderer.frame(), &mut pixels).unwrap();
        expected.push(pixels.as_bytes().to_vec());
    }
    assert_eq!(images(&fs, "/updater", 4), expected);
}

#[test]
fn invalid_camera_changes_do_not_consume_frames_or_replace_a_good_pose() {
    let base = options("/unused", 4).camera_config().unwrap();
    let plan = ExecutionPlan::derive(
        PlanRequest::certified(
            RenderIntent::Offline,
            SurfaceSpec::lumen(48, 32),
            OutputPixelFormat::Rgba8,
        )
        .with_max_frames_in_flight(4),
        &HardwareTopology::from_group_sizes(&[2, 2]).unwrap(),
        None,
    )
    .unwrap();
    let received = Arc::new(Mutex::new(Vec::new()));
    let copy = received.clone();
    let emitter = OrderedEmitter::new(
        EmitterConfig::new(
            FrameLayout::tight(PixelFormat::Rgba8, 48, 32).unwrap(),
            plan.frames_in_flight,
            0,
        )
        .unwrap(),
        vec![SinkBinding::reliable(
            "record",
            move |_, frame: &FrameBuffer| {
                copy.lock().unwrap().push(frame.as_bytes().to_vec());
                Ok(SinkWrite::Consumed)
            },
        )],
    )
    .unwrap();
    let mut pipeline = NativeFramePipeline::new(
        plan.clone(),
        config(&plan, &base),
        Some(Camera::new(base.clone()).unwrap()),
        emitter.handle(),
    )
    .unwrap();
    let mut scene = Scene::default();
    populate(&mut scene).unwrap();
    pipeline.capture(scene.stage(), 0).unwrap();
    for index in 0..6 {
        let mut bad = base.clone();
        match index {
            0 => bad.resolution = (96, 32),
            1 => bad.fps += 1,
            2 => bad.samples += 1,
            3 => bad.background.a = 0.5,
            4 => bad.max_allowable_norm += 1.0,
            _ => bad.light_source_position = [f64::NAN, 0.0, 0.0],
        }
        assert!(pipeline.update_camera(bad).is_err());
        assert_eq!(emitter.stats().reserved, 1);
    }
    pipeline.capture(scene.stage(), 1).unwrap();
    pipeline.update_camera(target(&base)).unwrap();
    pipeline.capture(scene.stage(), 2).unwrap();
    pipeline.update_camera(base.clone()).unwrap();
    pipeline.capture(scene.stage(), 3).unwrap();
    let stats = pipeline.finish().unwrap();
    emitter.finish().unwrap();
    assert_eq!(stats.emitted, 4);
    assert!(stats.render_team_frames.iter().all(|count| *count > 0));
    let images = received.lock().unwrap();
    assert_eq!(images[0], images[1]);
    assert_eq!(images[0], images[3]);
    assert_ne!(images[0], images[2]);
}

#[test]
fn invalid_rigs_and_factory_playback_abort_without_publication() {
    struct Empty;
    impl SceneConstruct for Empty {
        fn construct(&mut self, _: &mut Stage<'_>) -> fmn::Result<()> {
            Ok(())
        }
    }
    let fs = Arc::new(VirtualFs::new());
    for kind in 0..3 {
        let result = render_camera_with_fs(
            |scene, base| {
                let rig = if kind == 0 {
                    CameraRig::new(&mut Scene::default(), base)?
                } else {
                    CameraRig::new(scene, base)?
                };
                if kind == 1 {
                    scene.stage_mut().detach(rig.root(), rig.width());
                }
                if kind == 2 {
                    scene.wait(Some(0.125), &mut NullSceneSink)?;
                }
                Ok((Empty, rig))
            },
            options("/retry", 4),
            fs.clone(),
        );
        assert!(result.is_err());
        assert!(!fs.exists(Path::new("/retry")));
    }
    let report = render_camera_with_fs(
        |scene, base| Ok((Empty, CameraRig::new(scene, base)?)),
        options("/retry", 4),
        fs.clone(),
    )
    .unwrap();
    assert_eq!(report.artifact.frame_count, 1);
}

#[test]
fn bad_options_do_not_run_factory_and_scene_unwind_joins_workers() {
    let fs = Arc::new(VirtualFs::new());
    let mut invalid = options("/bad", 4);
    invalid.frames_in_flight = 0;
    let called = Cell::new(false);
    assert!(
        render_camera_with_fs(
            |scene, base| {
                called.set(true);
                factory(scene, base)
            },
            invalid,
            fs.clone()
        )
        .is_err()
    );
    assert!(!called.get());
    struct Panics;
    impl SceneConstruct for Panics {
        fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
            populate(stage.scene_mut())?;
            stage.wait(0.25)?;
            panic!("abort after camera captures")
        }
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        render_camera_with_fs(
            |scene, base| Ok((Panics, CameraRig::new(scene, base)?)),
            options("/retry", 4),
            fs.clone(),
        )
    }));
    assert!(result.is_err());
    assert!(!fs.exists(Path::new("/retry")));
    render_camera_with_fs(factory, options("/retry", 4), fs).unwrap();
}

#[test]
fn camera_animation_gif_and_y4m_are_window_independent() {
    for format in [RenderFormat::Gif, RenderFormat::Y4m] {
        let mut expected = None;
        for window in [1, 2, 4] {
            let fs = Arc::new(VirtualFs::new());
            let mut options = options("/movie", window);
            options.format = format;
            let report = render_camera_with_fs(factory, options, fs.clone()).unwrap();
            assert_eq!(report.frame_pipeline.unwrap().emitted, 4);
            let bytes = fs.read(Path::new("/movie")).unwrap();
            if let Some(expected) = &expected {
                assert_eq!(&bytes, expected);
            } else {
                expected = Some(bytes);
            }
        }
    }
}
