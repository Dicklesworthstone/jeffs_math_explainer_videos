//! Camera-bearing FMTL playback through the native Lumen composition seam.
//!
//! The shared bundle reader owns camera reconstruction, aspect adaptation and
//! frame-indexed revisions. This adapter only selects the camera-aware renderer
//! and transfers its raw frame to canvas storage. It is single-threaded on Wasm;
//! using the certified CPU algorithm does not add Wasm to the certified matrix.

use fmn_frame::convert::rgba16f_to_rgba8_slice;
use fmn_render::{EngineIdentity, RetainedFrameRenderer, RetainedFrameRendererConfig};

use super::{PlayerCore, PlayerError};

/// Called only after the public player has validated viewport, frame index and
/// destination length, before any snapshot materialization or rasterization.
pub(super) fn render_into(
    core: &PlayerCore,
    index: u32,
    dst: &mut [u8],
) -> Result<(), PlayerError> {
    let (stage, camera) = core
        .bundle
        .stage_at_with_camera(index, (core.width, core.height))?;
    let camera = camera.ok_or(PlayerError::PlanInconsistent("missing camera track"))?;
    let mut renderer = RetainedFrameRenderer::new(RetainedFrameRendererConfig {
        frame: crate::frame_config(core.width, core.height),
        tiling: crate::TILING,
        engine: EngineIdentity::certified(),
        threads: crate::RENDER_THREADS,
    })
    .map_err(|e| PlayerError::Render(e.to_string()))?;
    crate::note_rasterized_surface();
    renderer
        .render_with_camera(&stage, &camera)
        .map_err(|e| PlayerError::Render(e.to_string()))?;
    rgba16f_to_rgba8_slice(renderer.frame(), dst).map_err(|e| PlayerError::Render(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_frame::convert::rgba16f_to_rgba8;
    use fmn_frame::{FrameBuffer, FrameLayout, PixelFormat};
    use fmn_mobject::Stage;
    use fmn_render::{Camera, CameraConfig, FrameConfig, ScreenMap, Viewport};
    use fmn_scene::recording::SceneBundleRecorder;
    use fmn_scene::{
        BundleExportLimits, CaptureReason, IntegrationError, LifecycleEvent, RuntimeConfig, Scene,
        SceneSink,
    };

    const WIDTH: u32 = 48;
    const HEIGHT: u32 = 32;

    // Render original live captures, BEFORE encoding the bundle, through the
    // native front door and framebuffer transfer API. The player must reproduce
    // these pixels, not just agree with another decode of its own input.
    fn native_pixels(stage: &Stage, camera: &Camera) -> Vec<u8> {
        let mut renderer = RetainedFrameRenderer::new(RetainedFrameRendererConfig {
            frame: FrameConfig::new(
                Viewport {
                    width: WIDTH,
                    height: HEIGHT,
                },
                ScreenMap {
                    scale: 1.0,
                    origin: [0.0, 0.0],
                    y_up: true,
                },
                camera.background(),
            ),
            tiling: crate::TILING,
            engine: EngineIdentity::certified(),
            threads: 1,
        })
        .unwrap();
        renderer.render_with_camera(stage, camera).unwrap();
        let mut frame =
            FrameBuffer::new(FrameLayout::tight(PixelFormat::Rgba8, WIDTH, HEIGHT).unwrap());
        rgba16f_to_rgba8(renderer.frame(), &mut frame).unwrap();
        frame.as_bytes().to_vec()
    }

    struct Capture {
        recorder: SceneBundleRecorder,
        camera: Camera,
        observed: Vec<Vec<u8>>,
    }

    impl SceneSink for Capture {
        fn event(&mut self, event: LifecycleEvent) -> Result<(), IntegrationError> {
            self.recorder.event(event)
        }

        fn capture(
            &mut self,
            reason: CaptureReason,
            packet: fmn_anim::FramePacket,
        ) -> Result<(), IntegrationError> {
            self.observed
                .push(native_pixels(&packet.materialize_stage(), &self.camera));
            self.recorder
                .capture_with_camera(reason, packet, &self.camera)
        }
    }

    fn recording() -> (Vec<u8>, Vec<Vec<u8>>) {
        let mut scene = Scene::new(
            RuntimeConfig {
                fps: 8,
                ..RuntimeConfig::default()
            },
            0,
        )
        .unwrap();
        let build = crate::build_scene("circle_shift", WIDTH, HEIGHT).unwrap();
        *scene.stage_mut() = build.packets[0].materialize_stage();
        let base = CameraConfig {
            resolution: (WIDTH, HEIGHT),
            fps: 8,
            samples: 4,
            ..CameraConfig::default()
        };
        let mut sink = Capture {
            recorder: SceneBundleRecorder::new_render_only_with_camera(
                8,
                BundleExportLimits::default(),
            )
            .unwrap(),
            camera: Camera::new(base.clone()).unwrap(),
            observed: Vec::new(),
        };
        for step in 0..3 {
            let mut config = base.clone();
            let t = f64::from(step);
            config.frame.set_center([t * 0.75, t * 0.1, 0.0]).unwrap();
            config.frame.set_width(8.0 - t * 0.5).unwrap();
            config
                .frame
                .set_orientation([1.0, t * 0.05, t * 0.1, 0.0])
                .unwrap();
            config.frame.set_field_of_view(0.8 + t * 0.1).unwrap();
            config.light_source_position = [4.0 - t, 3.0, 8.0];
            sink.camera = Camera::new(config).unwrap();
            if step == 2 {
                sink.camera
                    .set_background(fmn_core::color::Srgb::from_rgb8(12, 34, 56).to_linear(1.0))
                    .unwrap();
            }
            scene.show(&mut sink).unwrap();
        }
        (sink.recorder.finish().unwrap().bytes, sink.observed)
    }

    #[test]
    fn camera_scrubs_match_original_native_captures_and_reuse_caller_storage() {
        let (bytes, expected) = recording();
        assert_eq!(expected.len(), 3);
        assert_ne!(
            expected[0], expected[1],
            "camera motion must reach the pixels"
        );
        assert_ne!(
            expected[1], expected[2],
            "background changes must reach the pixels"
        );
        let mut core = PlayerCore::load(&bytes).unwrap();
        assert_eq!(core.frame_count(), 3);
        core.set_viewport(WIDTH, HEIGHT).unwrap();
        let mut scratch = vec![0xA5; expected[0].len()];
        let storage = scratch.as_ptr();
        for index in [2, 0, 1, 2, 1, 0] {
            core.seek_frame(index).unwrap();
            assert_eq!(core.render_index(index).unwrap(), expected[index as usize]);
            let owned = crate::owned_rgba8_output_allocations();
            core.render_index_into(index, &mut scratch).unwrap();
            assert_eq!(scratch, expected[index as usize]);
            assert_eq!(scratch.as_ptr(), storage);
            assert_eq!(crate::owned_rgba8_output_allocations(), owned);
            assert_eq!(core.cursor, index);
        }
    }

    #[test]
    fn camera_viewport_changes_do_not_mutate_the_recorded_camera() {
        let (bytes, expected) = recording();
        let mut core = PlayerCore::load(&bytes).unwrap();
        for (width, height) in [(96, 64), (32, 32), (24, 48), (WIDTH, HEIGHT)] {
            core.set_viewport(width, height).unwrap();
            let pixels = core.render_index(1).unwrap();
            assert_eq!(pixels.len(), (width * height * 4) as usize);
            let mut scratch = vec![0; pixels.len()];
            core.render_index_into(1, &mut scratch).unwrap();
            assert_eq!(scratch, pixels);
        }
        assert_eq!(core.render_index(1).unwrap(), expected[1]);
    }

    #[test]
    fn camera_refusals_preserve_destination_and_cursor_before_rendering() {
        let (bytes, _) = recording();
        let mut core = PlayerCore::load(&bytes).unwrap();
        assert!(matches!(
            core.render_index(0),
            Err(PlayerError::Viewport(_))
        ));
        core.set_viewport(WIDTH, HEIGHT).unwrap();
        core.seek_frame(1).unwrap();
        let renders = crate::rasterized_surface_count();
        let owned = crate::owned_rgba8_output_allocations();
        let mut short = vec![0xA5; (WIDTH * HEIGHT * 4 - 1) as usize];
        assert!(matches!(
            core.render_index_into(0, &mut short),
            Err(PlayerError::DestinationLength { .. })
        ));
        assert!(matches!(
            core.render_index_into(core.frame_count(), &mut short),
            Err(PlayerError::FrameOutOfRange { .. })
        ));
        assert!(matches!(
            core.render_index(core.frame_count()),
            Err(PlayerError::FrameOutOfRange { .. })
        ));
        assert!(core.set_viewport(0, HEIGHT).is_err());
        assert!(core.seek_frame(core.frame_count()).is_err());
        assert_eq!((core.width, core.height, core.cursor), (WIDTH, HEIGHT, 1));
        assert_eq!(crate::rasterized_surface_count(), renders);
        assert_eq!(crate::owned_rgba8_output_allocations(), owned);
        assert!(short.iter().all(|&byte| byte == 0xA5));
    }
}
