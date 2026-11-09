//! Compiled artifacts must reconstruct on real render teams, with no new law.

use fmn::rendering::{
    NativeFramePipeline, RenderError, RenderFormat, RenderOptions, RenderPlanLimits,
    render_bundle_with_fs,
};
use fmn_anim::{Timeline, prepare_animation};
use fmn_core::{rate, rng::RngRoot};
use fmn_frame::convert::rgba16f_to_rgba8;
use fmn_frame::{FrameBuffer, FrameLayout, PixelFormat};
use fmn_mobject::animate::AnimateArgs;
use fmn_mobject::{Mobject, RecordBuffer, RecordSchema, Stage};
use fmn_output::{EmitterConfig, OrderedEmitter, SinkBinding, SinkWrite};
use fmn_platform::fs::{FileSystem, VirtualFs};
use fmn_platform::topology::HardwareTopology;
use fmn_render::{
    Camera, EngineIdentity, FrameConfig, RenderPlan, RetainedFrameRenderer,
    RetainedFrameRendererConfig, ScreenMap, Tiling, Viewport,
};
use fmn_runtime::{ExecutionPlan, OutputPixelFormat, PlanRequest, RenderIntent, SurfaceSpec};
use fmn_scene::timeline_bundle::{
    BundleSegmentKind, SharedTimelineBundle, TimelineBundle, export_timeline_bundle,
};
use std::path::Path;
use std::sync::{Arc, Mutex};

fn vector() -> Mobject {
    let mut records = RecordBuffer::new(RecordSchema::vmobject(), 3).unwrap();
    records.write_range(
        "point",
        0,
        &[-1.5, -1.0, 0.0, 0.0, 1.25, 0.0, 1.5, -1.0, 0.0],
    );
    records.write_range("fill_rgba", 0, &[0.2, 0.6, 1.0, 1.0].repeat(3));
    Mobject::from_buffer(records)
}

fn bundle(camera: bool, stateful: bool) -> Vec<u8> {
    let mut stage = Stage::new();
    let mob = stage.add(if camera {
        fmn::library::Cube::new(1.0).into()
    } else {
        vector()
    });
    stage.add_to_scene(mob).unwrap();
    let mut timeline = Timeline::new(8).unwrap();
    if stateful {
        stage
            .add_dt_updater(
                mob,
                |stage, mob, dt| {
                    stage.shift(mob, [dt, 0.0, 0.0]);
                },
                false,
            )
            .unwrap();
        timeline.wait(0.5).unwrap();
    } else {
        let builder = mob
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.5),
                rate_func: Some(rate::linear),
                ..AnimateArgs::default()
            })
            .unwrap()
            .shift([1.0, 0.0, 0.0])
            .unwrap();
        let animation = prepare_animation(builder, &mut stage).unwrap();
        timeline.play(vec![animation]).unwrap();
    }
    export_timeline_bundle(timeline, &mut stage, &RngRoot::from_seed(17)).unwrap()
}

fn options(path: &str, window: usize, camera: bool) -> RenderOptions {
    let mut options = RenderOptions::new(path).unwrap();
    options.config.camera.resolution = (32, 24);
    options.config.camera.fps = 8;
    options.config.sizes.frame_height = 6.0;
    options.config.determinism.mode = fmn_config::config::DeterminismMode::Certified;
    options.config.render.threads = fmn_config::config::ThreadPolicy::Fixed(4);
    options.frames_in_flight = window;
    if camera {
        let mut config = options.camera_config().unwrap();
        config
            .frame
            .set_euler_angles(Some(0.4), Some(0.7), None)
            .unwrap();
        options.camera = Some(config);
    }
    options
}

fn config(plan: &ExecutionPlan, options: &RenderOptions) -> RetainedFrameRendererConfig {
    RetainedFrameRendererConfig {
        frame: FrameConfig::new(
            Viewport {
                width: 32,
                height: 24,
            },
            ScreenMap {
                scale: 4.0,
                origin: [16.0, 12.0],
                y_up: true,
            },
            fmn::core::color::Srgb::from_hex(&options.config.camera.background_color)
                .unwrap()
                .to_linear(options.config.camera.background_opacity),
        ),
        tiling: Tiling {
            macro_tile: plan.macro_tile,
            fine_tile: plan.fine_tile,
        },
        engine: EngineIdentity::certified(),
        threads: 1,
    }
}

#[test]
fn compiled_pure_and_stateful_captures_match_serial_pixels_on_every_render_team() {
    for camera in [false, true] {
        for stateful in [false, true] {
            let bytes = bundle(camera, stateful);
            let serial = TimelineBundle::from_bytes(&bytes).unwrap();
            assert_eq!(
                serial.segment_kind(0),
                Some(if stateful {
                    BundleSegmentKind::Stateful
                } else {
                    BundleSegmentKind::Pure
                })
            );
            let shared = SharedTimelineBundle::from_bytes(&bytes).unwrap();
            let plan = ExecutionPlan::derive(
                PlanRequest::certified(
                    RenderIntent::Offline,
                    SurfaceSpec::lumen(32, 24),
                    OutputPixelFormat::Rgba8,
                )
                .with_max_frames_in_flight(4),
                &HardwareTopology::from_group_sizes(&[2, 2]).unwrap(),
                None,
            )
            .unwrap();
            let options = options("/unused", 4, camera);
            let config = config(&plan, &options);
            let camera = options.camera.map(|config| Camera::new(config).unwrap());
            let received = Arc::new(Mutex::new(Vec::new()));
            let output = received.clone();
            let emitter = OrderedEmitter::new(
                EmitterConfig::new(
                    FrameLayout::tight(PixelFormat::Rgba8, 32, 24).unwrap(),
                    plan.frames_in_flight,
                    0,
                )
                .unwrap(),
                vec![SinkBinding::reliable(
                    "record",
                    move |sequence, frame: &FrameBuffer| {
                        output
                            .lock()
                            .unwrap()
                            .push((sequence, frame.as_bytes().to_vec()));
                        Ok(SinkWrite::Consumed)
                    },
                )],
            )
            .unwrap();
            let mut pipeline =
                NativeFramePipeline::new(plan.clone(), config, camera.clone(), emitter.handle())
                    .unwrap();
            let jobs: Vec<_> = [3, 0, 2, 1]
                .into_iter()
                .map(|index| shared.frame_job(index).unwrap())
                .collect();
            drop(shared);
            drop(bytes);
            let mut expected = Vec::new();
            for (sequence, job) in jobs.into_iter().enumerate() {
                // Independent serial reconstruction/renderer, not the worker
                // cache or its state. Rebase output order without retiming jobs.
                let stage = serial.stage_at(job.index()).unwrap();
                let mut renderer = RetainedFrameRenderer::new(config).unwrap();
                if let Some(camera) = &camera {
                    renderer.render_with_camera(&stage, camera).unwrap();
                } else {
                    renderer.render(&stage, 0).unwrap();
                }
                let mut rgba =
                    FrameBuffer::new(FrameLayout::tight(PixelFormat::Rgba8, 32, 24).unwrap());
                rgba16f_to_rgba8(renderer.frame(), &mut rgba).unwrap();
                expected.push((sequence as u64, rgba.as_bytes().to_vec()));
                pipeline.capture_compiled(job, sequence as u64).unwrap();
            }
            assert_eq!(pipeline.flush().unwrap().emitted, 4);
            let stats = pipeline.finish().unwrap();
            emitter.finish().unwrap();
            assert_eq!(
                (stats.submitted, stats.emitted, stats.outstanding_slots),
                (4, 4, 0)
            );
            assert!(stats.render_team_frames.iter().all(|frames| *frames > 0));
            assert_eq!(*received.lock().unwrap(), expected);
        }
    }
}

#[test]
fn native_bundle_export_preserves_clock_and_format_bytes_across_windows() {
    for camera in [false, true] {
        for format in [
            RenderFormat::PngSequence,
            RenderFormat::Gif,
            RenderFormat::Y4m,
        ] {
            let bytes = bundle(camera, false);
            let mut first = None;
            for window in [1, 2, 4] {
                let fs = Arc::new(VirtualFs::new());
                let mut options = options("/output", window, camera);
                options.format = format;
                let report = render_bundle_with_fs(&bytes, options, fs.clone()).unwrap();
                assert_eq!(report.scene.time.frames(), 4);
                assert_eq!(report.scene.play_count, 1);
                assert_eq!(report.artifact.frame_count, 4);
                assert_eq!(report.frame_pipeline.unwrap().outstanding_slots, 0);
                let frames = if format == RenderFormat::PngSequence {
                    (0..4)
                        .map(|index| {
                            fs.read(&Path::new("/output").join(format!("frame_{index:06}.png")))
                                .unwrap()
                        })
                        .collect::<Vec<_>>()
                } else {
                    vec![fs.read(Path::new("/output")).unwrap()]
                };
                if let Some(first) = &first {
                    assert_eq!(&frames, first);
                } else {
                    first = Some(frames);
                }
            }
        }
    }
}

#[test]
fn compiled_refusals_do_not_publish_and_worker_failure_allows_retry() {
    let fs = Arc::new(VirtualFs::new());
    let bytes = bundle(false, false);
    let mut wrong_fps = options("/retry", 4, false);
    wrong_fps.config.camera.fps = 10;
    assert!(matches!(
        render_bundle_with_fs(&bytes, wrong_fps, fs.clone()),
        Err(RenderError::InvalidOptions(_))
    ));
    let mut limited = options("/retry", 4, false);
    limited.max_frames = 3;
    assert!(matches!(
        render_bundle_with_fs(&bytes, limited, fs.clone()),
        Err(RenderError::InvalidOptions(_))
    ));
    assert!(!fs.exists(Path::new("/retry")));
    // A 3D bundle on the affine route fails inside a real render worker.
    assert!(matches!(
        render_bundle_with_fs(
            &bundle(true, false),
            options("/retry", 4, false),
            fs.clone()
        ),
        Err(RenderError::Pipeline(_))
    ));
    assert!(!fs.exists(Path::new("/retry")));
    render_bundle_with_fs(&bytes, options("/retry", 4, false), fs).unwrap();
}

/// One stroke with paint on every record, the shape that retains profile knots. It holds still,
/// so every frame asks for the same style row, and the demand is exact however the frames fall
/// across the workers.
fn painted_bundle() -> Vec<u8> {
    let mut records = RecordBuffer::new(RecordSchema::vmobject(), 5).unwrap();
    records.write_range(
        "point",
        0,
        &[
            -2.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 2.0, 0.0, 0.0,
        ],
    );
    records.write_range("stroke_width", 0, &[40.0; 5]);
    records.write_range(
        "stroke_rgba",
        0,
        &[
            1.0, 0.0, 0.0, 1.0, 0.8, 0.2, 0.0, 1.0, 0.6, 0.4, 0.0, 1.0, 0.4, 0.6, 0.0, 1.0, 0.2,
            0.8, 0.0, 1.0,
        ],
    );
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::from_buffer(records));
    stage.add_to_scene(mob).unwrap();
    let mut timeline = Timeline::new(8).unwrap();
    timeline.wait(0.5).unwrap();
    export_timeline_bundle(timeline, &mut stage, &RngRoot::from_seed(17)).unwrap()
}

#[test]
fn a_declared_plan_budget_reaches_the_compiled_workers_on_both_routes() {
    let bytes = painted_bundle();
    // Read from a plan no worker touches, so a worker's own answer cannot supply it.
    let stage = TimelineBundle::from_bytes(&bytes)
        .unwrap()
        .stage_at(0)
        .unwrap();
    let mut serial = RenderPlan::new();
    serial.sync(&stage, 0).unwrap();
    let demand = serial.styles().profile_knots() as u64;
    assert!(demand > 1, "the fixture lost its per-record paint");

    let declared = |path: &str, camera: bool, knots: Option<u64>| {
        let mut options = options(path, 4, camera);
        options.format = RenderFormat::PngSequence;
        if let Some(max_retained_profile_knots) = knots {
            options.plan_limits = RenderPlanLimits {
                max_retained_profile_knots,
                ..RenderPlanLimits::default()
            };
        }
        options
    };
    let frames = |fs: &VirtualFs, path: &str| -> Vec<Vec<u8>> {
        (0..4)
            .map(|index| {
                fs.read(&Path::new(path).join(format!("frame_{index:06}.png")))
                    .unwrap()
            })
            .collect()
    };
    for camera in [false, true] {
        let fs = Arc::new(VirtualFs::new());
        // Compiled frames are reconstructed and compiled only inside a render worker. The
        // default admits this scene, so a worker that kept it would render one knot short.
        let Err(error) = render_bundle_with_fs(
            &bytes,
            declared("/short", camera, Some(demand - 1)),
            fs.clone(),
        ) else {
            panic!("camera route {camera}: a worker admitted one knot over its declared budget");
        };
        assert!(matches!(error, RenderError::Pipeline(_)), "{error:?}");
        let words = format!("{error:?}");
        let refusal = format!(
            "resource: \"retained profile knots\", requested: {demand}, limit: {}",
            demand - 1
        );
        assert!(words.contains(&refusal), "{words}");
        assert!(!fs.exists(Path::new("/short")));

        // The exact demand is admitted on every frame, and a budget never moves a pixel.
        render_bundle_with_fs(&bytes, declared("/exact", camera, Some(demand)), fs.clone())
            .unwrap();
        render_bundle_with_fs(&bytes, declared("/default", camera, None), fs.clone()).unwrap();
        assert_eq!(frames(&fs, "/exact"), frames(&fs, "/default"));
    }
}

#[test]
fn empty_bundle_and_existing_native_outputs_are_not_silently_replaced() {
    let mut stage = Stage::new();
    let empty = export_timeline_bundle(
        Timeline::new(8).unwrap(),
        &mut stage,
        &RngRoot::from_seed(0),
    )
    .unwrap();
    let fs = Arc::new(VirtualFs::new());
    assert!(render_bundle_with_fs(&empty, options("/empty", 2, false), fs.clone()).is_err());
    assert!(!fs.exists(Path::new("/empty")));
    for format in [RenderFormat::Gif, RenderFormat::Y4m] {
        let fs = Arc::new(VirtualFs::new());
        fs.insert("/existing", b"owned by someone else".to_vec());
        let mut options = options("/existing", 2, false);
        options.format = format;
        assert!(render_bundle_with_fs(&bundle(false, false), options, fs.clone()).is_err());
        assert_eq!(
            fs.read(Path::new("/existing")).unwrap(),
            b"owned by someone else"
        );
    }
}
