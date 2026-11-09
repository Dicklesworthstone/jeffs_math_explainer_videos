//! Real native exports, compared against serial Lumen rather than self-goldens.

use std::path::Path;
use std::sync::Arc;

use fmn::prelude::*;
use fmn::rendering::{NativeFrameError, RenderError};
use fmn_anim::FramePacket;
use fmn_codec::{PngLimits, decode_png, decode_y4m};
use fmn_config::config::{DeterminismMode, ThreadPolicy};
use fmn_core::color::Srgb;
use fmn_frame::convert::rgba16f_to_rgba8;
use fmn_frame::{FrameBuffer, FrameLayout, PixelFormat};
use fmn_platform::fs::{FileSystem, VirtualFs};
use fmn_render::{
    EngineIdentity, FrameConfig, FrameJobError, RetainedFrameRenderer, RetainedFrameRendererConfig,
    RetainedFrameRendererError, ScreenMap, Tiling, Viewport,
};
use fmn_runtime::{FrameStreamError, PipelineError, PipelineStage};
use fmn_scene::{CaptureReason, IntegrationError, RuntimeConfig, SceneSink};

fn options(path: &str, format: RenderFormat, window: usize, threads: u32) -> RenderOptions {
    let mut options = RenderOptions::new(path).unwrap();
    options.format = format;
    options.config.camera.resolution = (48, 32);
    options.config.camera.fps = 8;
    options.config.sizes.frame_height = 4.0;
    options.config.determinism.mode = DeterminismMode::Certified;
    options.config.render.threads = ThreadPolicy::Fixed(threads);
    options.frames_in_flight = window;
    options
}

struct Moving;
impl SceneConstruct for Moving {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let circle = stage.add(Circle::new().radius(0.5).color(BLUE))?;
        stage.set_fill(circle, Some(BLUE), Some(0.8), None, true);
        stage.play(
            circle
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.5),
                    rate_func: Some(fmn::core::rate::linear),
                    ..AnimateArgs::default()
                })?
                .shift(RIGHT)?,
        )?;
        Ok(())
    }
}

struct Still;
impl SceneConstruct for Still {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add(Circle::new().radius(0.5))?;
        Ok(())
    }
}

struct SerialReference {
    renderer: RetainedFrameRenderer,
    frames: Vec<Vec<u8>>,
}
impl SceneSink for SerialReference {
    fn capture(&mut self, _: CaptureReason, packet: FramePacket) -> Result<(), IntegrationError> {
        // No OwnedVectorFrame, FrameStream or worker cache in this reference.
        self.renderer
            .render(&packet.materialize_stage(), 0)
            .unwrap();
        let mut output = FrameBuffer::new(FrameLayout::tight(PixelFormat::Rgba8, 48, 32).unwrap());
        rgba16f_to_rgba8(self.renderer.frame(), &mut output).unwrap();
        self.frames.push(output.as_bytes().to_vec());
        Ok(())
    }
}

#[test]
fn affine_export_uses_pipeline_and_matches_live_serial_renderer() {
    for mode in [DeterminismMode::Certified, DeterminismMode::Standard] {
        let mut expected = None;
        for (window, threads) in [(1, 1), (2, 4), (4, 16)] {
            let fs = Arc::new(VirtualFs::new());
            let mut options = options("/frames", RenderFormat::PngSequence, window, threads);
            options.config.determinism.mode = mode;
            let runtime = RuntimeConfig::from_config(&options.config);
            let seed = options.config.determinism.seed;
            let background = Srgb::from_hex(&options.config.camera.background_color)
                .unwrap()
                .to_linear(options.config.camera.background_opacity);
            let aa = options.config.render.aa;
            let report = render_with_fs(&mut Moving, options, fs.clone()).unwrap();
            let stats = report
                .frame_pipeline
                .as_ref()
                .expect("affine pipeline was bypassed");
            assert_eq!(report.artifact.frame_count, 4);
            assert_eq!(report.scene.play_count, 1);
            assert_eq!(
                (
                    stats.submitted,
                    stats.prepared,
                    stats.rasterized,
                    stats.converted,
                    stats.emitted
                ),
                (4, 4, 4, 4, 4)
            );
            assert_eq!(stats.outstanding_slots, 0);
            assert!(stats.max_in_flight <= report.execution_plan.frames_in_flight);
            assert!(report.execution_plan.frames_in_flight <= window);
            assert_eq!(stats.render_team_frames.iter().sum::<u64>(), 4);
            let encoded: Vec<Vec<u8>> = (0..4)
                .map(|sequence| {
                    fs.read(&Path::new("/frames").join(format!("frame_{sequence:06}.png")))
                        .unwrap()
                })
                .collect();
            let pixels: Vec<Vec<u8>> = encoded
                .iter()
                .map(|bytes| decode_png(bytes, &PngLimits::default()).unwrap().rgba)
                .collect();
            assert_ne!(pixels[0], pixels[3], "capture must observe the animation");
            let mut reference = SerialReference {
                renderer: RetainedFrameRenderer::new(RetainedFrameRendererConfig {
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
                        background,
                    )
                    .with_aa_policy(aa),
                    tiling: Tiling {
                        macro_tile: report.execution_plan.macro_tile,
                        fine_tile: report.execution_plan.fine_tile,
                    },
                    engine: if mode == DeterminismMode::Certified {
                        EngineIdentity::certified()
                    } else {
                        EngineIdentity::fast()
                    },
                    threads: 1,
                })
                .unwrap(),
                frames: Vec::new(),
            };
            fmn::run_scene(&mut Moving, runtime, seed, &mut reference).unwrap();
            assert_eq!(pixels, reference.frames);
            if let Some(expected) = &expected {
                assert_eq!(&encoded, expected);
            } else {
                expected = Some(encoded);
            }
        }
    }
}

#[test]
fn affine_gif_and_y4m_keep_every_sample_at_every_window() {
    for format in [RenderFormat::Gif, RenderFormat::Y4m] {
        let mut expected = None;
        for window in [1, 2, 4] {
            let fs = Arc::new(VirtualFs::new());
            let report = render_with_fs(
                &mut Moving,
                options("/movie", format, window, 4),
                fs.clone(),
            )
            .unwrap();
            assert_eq!(report.frame_pipeline.unwrap().emitted, 4);
            assert_eq!(report.artifact.frame_count, 4);
            let bytes = fs.read(Path::new("/movie")).unwrap();
            if format == RenderFormat::Y4m {
                let movie = decode_y4m(&bytes).unwrap();
                assert_eq!(movie.fps, (8, 1));
                assert_eq!(movie.frames.len(), 4);
                assert_ne!(movie.frames[0], movie.frames[3]);
            } else {
                assert!(bytes.starts_with(b"GIF89a"));
                assert_eq!(bytes.last(), Some(&0x3b));
            }
            if let Some(expected) = &expected {
                assert_eq!(&bytes, expected);
            } else {
                expected = Some(bytes);
            }
        }
    }
}

#[test]
fn worker_refusal_preserves_original_error_and_cancels_artifact() {
    struct OutOfPlane;
    impl SceneConstruct for OutOfPlane {
        fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
            let circle = stage.add(Circle::new().radius(0.5))?;
            stage.shift(circle, [0.0, 0.0, 1.0]);
            Ok(())
        }
    }
    for window in [1, 2, 4] {
        let fs = Arc::new(VirtualFs::new());
        let error = render_with_fs(
            &mut OutOfPlane,
            options("/retry", RenderFormat::PngSequence, window, 4),
            fs.clone(),
        )
        .unwrap_err();
        let RenderError::Pipeline(FrameStreamError::Pipeline(failure)) = error else {
            panic!("lost original raster-stage error: {error}");
        };
        assert!(matches!(
            failure.error,
            PipelineError::Stage {
                stage: PipelineStage::Raster,
                source: NativeFrameError::Renderer(RetainedFrameRendererError::Prepare(
                    FrameJobError::CameraProjectionRequired { .. }
                )),
                ..
            }
        ));
        assert_eq!(failure.stats.outstanding_slots, 0);
        assert!(!fs.exists(Path::new("/retry")));
        let report = render_with_fs(
            &mut Still,
            options("/retry", RenderFormat::PngSequence, window, 4),
            fs,
        )
        .unwrap();
        assert_eq!(report.frame_pipeline.unwrap().emitted, 1);
    }
}

#[test]
fn source_failure_or_panic_joins_workers_before_retry() {
    struct Panicking;
    impl SceneConstruct for Panicking {
        fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
            Moving.construct(stage)?;
            panic!("intentional failure after queuing affine frames");
        }
    }
    for format in [
        RenderFormat::PngSequence,
        RenderFormat::Gif,
        RenderFormat::Y4m,
    ] {
        let fs = Arc::new(VirtualFs::new());
        let mut limited = options("/retry", format, 4, 4);
        limited.max_frames = 1;
        assert!(matches!(
            render_with_fs(&mut Moving, limited, fs.clone()),
            Err(RenderError::InvalidOptions("scene exceeded max_frames"))
        ));
        assert!(!fs.exists(Path::new("/retry")));
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = render_with_fs(&mut Panicking, options("/retry", format, 4, 4), fs.clone());
            }))
            .is_err()
        );
        assert!(!fs.exists(Path::new("/retry")));
        let report = render_with_fs(&mut Still, options("/retry", format, 4, 4), fs).unwrap();
        assert_eq!(report.frame_pipeline.unwrap().emitted, 1);
    }
}

#[test]
fn retained_worker_pixels_are_included_in_preflight_budget() {
    let fs = Arc::new(VirtualFs::new());
    let report = render_with_fs(
        &mut Still,
        options("/first", RenderFormat::PngSequence, 1, 1),
        fs.clone(),
    )
    .unwrap();
    struct MustNotRun;
    impl SceneConstruct for MustNotRun {
        fn construct(&mut self, _: &mut Stage<'_>) -> fmn::Result<()> {
            panic!("pixel budget refusal must precede scene construction");
        }
    }
    let mut budget = options("/limited", RenderFormat::PngSequence, 1, 1);
    budget.max_resident_bytes =
        report.execution_plan.estimated_in_flight_bytes as u64 + 64 * 1024 * 1024;
    assert!(matches!(
        render_with_fs(&mut MustNotRun, budget, fs.clone()),
        Err(RenderError::InvalidOptions(
            "render plan exceeds max_resident_bytes"
        ))
    ));
    assert!(!fs.exists(Path::new("/limited")));
}
