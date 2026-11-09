//! Public camera export must use the bounded pipeline without revising pixels.

use fmn::prelude::*;
use fmn_anim::FramePacket;
use fmn_codec::{PngLimits, decode_png};
use fmn_config::config::{DeterminismMode, ThreadPolicy};
use fmn_frame::convert::rgba16f_to_rgba8;
use fmn_frame::{FrameBuffer, FrameLayout, PixelFormat};
use fmn_platform::fs::{FileSystem, VirtualFs};
use fmn_render::{
    Camera, EngineIdentity, FrameConfig, RetainedFrameRenderer, RetainedFrameRendererConfig,
    ScreenMap, Tiling, Viewport,
};
use fmn_scene::{CaptureReason, IntegrationError, RuntimeConfig, SceneSink};
use std::path::Path;
use std::sync::Arc;

fn options(path: &str, format: RenderFormat, window: usize, threads: u32) -> RenderOptions {
    let mut options = RenderOptions::new(path).unwrap();
    options.format = format;
    options.config.camera.resolution = (48, 32);
    options.config.camera.fps = 10;
    options.config.sizes.frame_height = 4.0;
    options.config.determinism.mode = DeterminismMode::Certified;
    options.config.render.threads = ThreadPolicy::Fixed(threads);
    options.frames_in_flight = window;
    let mut camera = options.camera_config().unwrap();
    camera
        .frame
        .set_euler_angles(Some(0.4), Some(0.7), None)
        .unwrap();
    options.camera = Some(camera);
    options
}

struct Moving;
impl SceneConstruct for Moving {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let cube = stage.add(Cube::new(1.1).color(BLUE))?;
        let circle = stage.add(Circle::new().radius(0.3).color(WHITE))?;
        stage.set_fill(circle, Some(WHITE), Some(1.0), None, true);
        stage.shift(circle, [-1.5, 0.0, 0.0]);
        stage.play(
            cube.animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.4),
                    rate_func: Some(fmn::core::rate::linear),
                    ..AnimateArgs::default()
                })?
                .shift(RIGHT)?,
        )?;
        Ok(())
    }
}

struct Static;
impl SceneConstruct for Static {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add(Cube::new(1.1).color(BLUE))?;
        Ok(())
    }
}

struct SerialReference {
    camera: Camera,
    renderer: RetainedFrameRenderer,
    frames: Vec<Vec<u8>>,
}
impl SceneSink for SerialReference {
    fn capture(&mut self, _: CaptureReason, packet: FramePacket) -> Result<(), IntegrationError> {
        // Deliberately bypass FrameStream and PreparedCameraFrame's owned path.
        self.renderer
            .render_with_camera(&packet.materialize_stage(), &self.camera)
            .unwrap();
        let mut output = FrameBuffer::new(FrameLayout::tight(PixelFormat::Rgba8, 48, 32).unwrap());
        rgba16f_to_rgba8(self.renderer.frame(), &mut output).unwrap();
        self.frames.push(output.as_bytes().to_vec());
        Ok(())
    }
}

#[test]
fn queued_camera_pixels_equal_serial_capture_at_every_window() {
    let mut expected: Option<Vec<Vec<u8>>> = None;
    for (window, threads) in [(1, 1), (2, 2), (4, 4)] {
        let fs = Arc::new(VirtualFs::new());
        let options = options("/frames", RenderFormat::PngSequence, window, threads);
        let camera = Camera::new(options.camera.clone().unwrap()).unwrap();
        let runtime = RuntimeConfig::from_config(&options.config);
        let seed = options.config.determinism.seed;
        let report = render_with_fs(&mut Moving, options, fs.clone()).unwrap();
        let stats = report
            .frame_pipeline
            .as_ref()
            .expect("camera pipeline was bypassed");
        // Exact f64 0.4 is above 2/5, so four samples do not cover it.
        assert_eq!(report.artifact.frame_count, 5);
        assert_eq!(stats.submitted, 5);
        assert_eq!(stats.emitted, 5);
        assert_eq!(stats.outstanding_slots, 0);
        assert!(stats.max_in_flight <= report.execution_plan.frames_in_flight);
        assert!(report.execution_plan.frames_in_flight <= window);
        let pixels: Vec<Vec<u8>> = (0..5)
            .map(|sequence| {
                let bytes = fs
                    .read(&Path::new("/frames").join(format!("frame_{sequence:06}.png")))
                    .unwrap();
                decode_png(&bytes, &PngLimits::default()).unwrap().rgba
            })
            .collect();
        assert_ne!(pixels[0], pixels[4], "animation must move the visible cube");
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
                    camera.background(),
                ),
                tiling: Tiling {
                    macro_tile: report.execution_plan.macro_tile,
                    fine_tile: report.execution_plan.fine_tile,
                },
                engine: EngineIdentity::certified(),
                threads: 1,
            })
            .unwrap(),
            camera,
            frames: Vec::new(),
        };
        fmn::run_scene(&mut Moving, runtime, seed, &mut reference).unwrap();
        assert_eq!(pixels, reference.frames);
        if let Some(expected) = &expected {
            assert_eq!(&pixels, expected);
        } else {
            expected = Some(pixels);
        }
    }
}

#[test]
fn native_gif_and_y4m_are_window_independent() {
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
            assert_eq!(report.frame_pipeline.unwrap().emitted, 5);
            assert_eq!(report.artifact.frame_count, 5);
            let bytes = fs.read(Path::new("/movie")).unwrap();
            if let Some(expected) = &expected {
                assert_eq!(&bytes, expected);
            } else {
                expected = Some(bytes);
            }
        }
    }
}

#[test]
fn failed_scene_cancels_camera_workers_before_immediate_retry() {
    for window in [1, 2, 4] {
        let fs = Arc::new(VirtualFs::new());
        let mut limited = options("/retry", RenderFormat::PngSequence, window, 4);
        limited.max_frames = 1;
        assert!(matches!(
            render_with_fs(&mut Moving, limited, fs.clone()),
            Err(RenderError::InvalidOptions("scene exceeded max_frames"))
        ));
        assert!(!fs.exists(Path::new("/retry")));
        let report = render_with_fs(
            &mut Static,
            options("/retry", RenderFormat::PngSequence, window, 4),
            fs.clone(),
        )
        .unwrap();
        assert_eq!(report.frame_pipeline.unwrap().emitted, 1);
        assert_eq!(report.artifact.frame_count, 1);
    }
}

#[test]
fn unwinding_scene_joins_camera_pipeline_and_aborts_artifact() {
    struct Panicking;
    impl SceneConstruct for Panicking {
        fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
            stage.add(Cube::new(1.1))?;
            stage.wait(0.1)?;
            panic!("intentional panic after queued camera capture")
        }
    }
    let fs = Arc::new(VirtualFs::new());
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = render_with_fs(
            &mut Panicking,
            options("/panic", RenderFormat::PngSequence, 4, 4),
            fs.clone(),
        );
    }));
    assert!(result.is_err());
    assert!(!fs.exists(Path::new("/panic")));
    render_with_fs(
        &mut Static,
        options("/panic", RenderFormat::PngSequence, 4, 4),
        fs,
    )
    .unwrap();
}

#[test]
fn cached_affine_exports_use_the_shared_frame_pipeline() {
    let fs = Arc::new(VirtualFs::new());
    let mut options = options("/affine", RenderFormat::PngSequence, 2, 1);
    options.camera = None;
    struct Empty;
    impl SceneConstruct for Empty {
        fn construct(&mut self, _: &mut Stage<'_>) -> fmn::Result<()> {
            Ok(())
        }
    }
    let report = render_with_fs(&mut Empty, options, fs).unwrap();
    let stats = report.frame_pipeline.expect("affine pipeline was bypassed");
    assert_eq!(stats.emitted, 1);
    assert_eq!(stats.outstanding_slots, 0);
    assert_eq!(report.artifact.frame_count, 1);
}
