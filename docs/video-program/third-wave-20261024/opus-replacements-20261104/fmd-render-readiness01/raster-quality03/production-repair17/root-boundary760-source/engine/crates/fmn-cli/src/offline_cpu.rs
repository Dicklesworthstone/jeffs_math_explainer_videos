//! Shipping CLI adapter to the same bounded CPU pipeline as native `render()`.
//!
//! The CLI retains ownership of sink finalization and artifact publication.
//! No scene callback is moved to a worker and no sampling loop is replaced.

use fmn::rendering::{NativeFramePipeline, RenderError};
use fmn_output::EmitterHandle;
use fmn_render::RetainedFrameRendererConfig;
use fmn_runtime::{ExecutionPlan, PipelineStats};
use fmn_scene::{IntegrationError, timeline_bundle::TimelineFrameJob};

use super::CliError;

pub(super) struct CpuRenderer {
    pipeline: Option<NativeFramePipeline>,
}

impl CpuRenderer {
    pub(super) fn new_with_camera(
        plan: ExecutionPlan,
        config: RetainedFrameRendererConfig,
        output: EmitterHandle,
        camera: Option<fmn_render::Camera>,
    ) -> Result<Self, CliError> {
        NativeFramePipeline::new(plan, config, camera, output)
            .map(|pipeline| Self {
                pipeline: Some(pipeline),
            })
            .map_err(|error| CliError::new("render", error.to_string()))
    }

    pub(super) fn capture(
        &mut self,
        stage: &fmn_scene::studio_bridge::Stage,
        sequence: u64,
    ) -> Result<(), IntegrationError> {
        let pipeline = self.pipeline.as_mut().ok_or_else(|| {
            IntegrationError::new("lumen", "CPU frame pipeline was already finalized")
        })?;
        let result = pipeline.capture(stage, sequence);
        self.capture_result(result)
    }

    pub(super) fn capture_compiled(
        &mut self,
        job: TimelineFrameJob,
        sequence: u64,
    ) -> Result<(), IntegrationError> {
        let pipeline = self.pipeline.as_mut().ok_or_else(|| {
            IntegrationError::new("lumen", "CPU frame pipeline was already finalized")
        })?;
        let result = pipeline.capture_compiled(job, sequence);
        self.capture_result(result)
    }

    fn capture_result(&mut self, result: Result<(), RenderError>) -> Result<(), IntegrationError> {
        let Err(mut error) = result else {
            return Ok(());
        };
        // Source admission may only see Closed. Recover the actual failing
        // raster/conversion/emit stage before the outer scene hides it behind
        // IntegrationError. A geometric preparation error stays geometric.
        if matches!(&error, RenderError::Pipeline(_))
            && let Some(pipeline) = self.pipeline.take()
            && let Err(root) = pipeline.finish()
        {
            error = root;
        }
        Err(IntegrationError::new("lumen", error.to_string()))
    }

    pub(super) fn finish(&mut self) -> Result<PipelineStats, CliError> {
        self.pipeline
            .take()
            .ok_or_else(|| CliError::new("render", "CPU frame pipeline was already finalized"))?
            .finish()
            .map_err(|error| CliError::new("render", error.to_string()))
    }

    pub(super) fn abort(&mut self) {
        // NativeFramePipeline wakes the output ring before joining workers.
        drop(self.pipeline.take());
    }
}

impl super::RenderSink {
    /// Submit immutable FMTL input; CPU reconstruction happens after admission
    /// on the assigned render team. Keep the annex's serial path explicit.
    pub(super) fn render_compiled(
        &mut self,
        job: TimelineFrameJob,
    ) -> Result<(), IntegrationError> {
        match &mut self.renderer {
            super::OfflineFrameRenderer::Cpu(renderer) => {
                renderer.capture_compiled(job, self.next_sequence)?;
                self.backend.record_cpu_frame()?;
                self.backend.route =
                    if matches!(self.backend.route, "camera-cpu" | "compiled-camera-cpu") {
                        "compiled-camera-cpu"
                    } else {
                        "compiled-cpu"
                    };
                self.next_sequence = self
                    .next_sequence
                    .checked_add(1)
                    .ok_or_else(|| IntegrationError::new("reel", "frame sequence exhausted"))?;
                Ok(())
            }
            #[cfg(feature = "metal")]
            super::OfflineFrameRenderer::Metal { .. } => {
                let revision = u64::from(job.index()) + 1;
                let stage = job
                    .materialize()
                    .map_err(|error| IntegrationError::new("compiled", error.to_string()))?;
                self.render_stage(&stage, revision)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use fmn::mobject::{Mobject, RecordBuffer, RecordSchema, Stage};
    use fmn_codec::{PngLimits, decode_png};
    use fmn_platform::fs::VirtualFs;
    use fmn_platform::topology::HardwareTopology;
    use fmn_runtime::{ExecutionPlan, OutputPixelFormat, PlanRequest, RenderIntent, SurfaceSpec};

    fn config() -> fmn_config::Config {
        let mut config = fmn_config::Config::resolve(&[], None).unwrap().config;
        config.camera.resolution = (32, 24);
        config.camera.fps = 8;
        config.sizes.frame_height = 6.0;
        config.determinism.mode = fmn_config::config::DeterminismMode::Certified;
        config
    }

    fn plan(format: OutputPixelFormat, window: usize) -> ExecutionPlan {
        let mut surface = SurfaceSpec::lumen(32, 24);
        surface.working_bytes_per_pixel = if format == OutputPixelFormat::Rgba8 {
            16
        } else {
            20
        };
        ExecutionPlan::derive(
            PlanRequest::certified(RenderIntent::Offline, surface, format)
                .with_max_frames_in_flight(window),
            &HardwareTopology::from_group_sizes(&[2, 2]).unwrap(),
            None,
        )
        .unwrap()
    }

    fn shape() -> Mobject {
        let mut records = RecordBuffer::new(RecordSchema::vmobject(), 3).unwrap();
        records.write_range(
            "point",
            0,
            &[-1.5, -1.0, 0.0, 0.0, 1.25, 0.0, 1.5, -1.0, 0.0],
        );
        records.write_range("fill_rgba", 0, &[0.2, 0.6, 1.0, 1.0].repeat(3));
        Mobject::from_buffer(records)
    }

    #[test]
    fn compiled_jobs_reconstruct_on_all_cli_render_teams() {
        let mut stage = Stage::new();
        let mob = stage.add(shape());
        stage.add_to_scene(mob).unwrap();
        let mut timeline = fmn::animation::Timeline::new(8).unwrap();
        timeline.wait(1.0).unwrap();
        let bytes = fmn_scene::export_timeline_bundle(timeline, &mut stage, &RngRoot::from_seed(0))
            .unwrap();
        let shared = fmn_scene::timeline_bundle::SharedTimelineBundle::from_bytes(&bytes).unwrap();
        let fs = Arc::new(VirtualFs::new());
        let plan = plan(OutputPixelFormat::Rgba8, 4);
        assert_eq!(plan.render_teams.len(), 2);
        let mut sink = RenderSink::new(
            fs,
            &config(),
            &plan,
            &RenderTarget::Native(NativeFrameFormat::PngSequence),
            PathBuf::from("/compiled"),
        )
        .unwrap();
        for index in 0..shared.frame_count() {
            sink.render_compiled(shared.frame_job(index).unwrap())
                .unwrap();
        }
        let finished = sink.finish(None).unwrap();
        assert_eq!(finished.backend.route, "compiled-cpu");
        let stats = finished.backend.pipeline.unwrap();
        assert_eq!(
            (stats.submitted, stats.emitted, stats.outstanding_slots),
            (8, 8, 0)
        );
        assert_eq!(stats.render_team_frames.len(), 2);
        assert!(stats.render_team_frames.iter().all(|n| *n > 0));
    }

    #[test]
    fn compiled_camera_refusal_occurs_on_worker_not_serial_capture() {
        let mut stage = Stage::new();
        let mob = stage.add(Mobject::from(fmn::library::Cube::new(1.0)));
        stage.add_to_scene(mob).unwrap();
        let mut timeline = fmn::animation::Timeline::new(8).unwrap();
        timeline.wait(0.125).unwrap();
        let bytes = fmn_scene::export_timeline_bundle(timeline, &mut stage, &RngRoot::from_seed(0))
            .unwrap();
        let shared = fmn_scene::timeline_bundle::SharedTimelineBundle::from_bytes(&bytes).unwrap();
        let fs = Arc::new(VirtualFs::new());
        let mut sink = RenderSink::new(
            fs.clone(),
            &config(),
            &plan(OutputPixelFormat::Rgba8, 1),
            &RenderTarget::Native(NativeFrameFormat::PngSequence),
            PathBuf::from("/refused"),
        )
        .unwrap();
        // Ordinary affine capture refuses Cube before queueing. The compiled
        // route must admit its input and let the worker discover that refusal.
        sink.render_compiled(shared.frame_job(0).unwrap()).unwrap();
        assert!(sink.finish(None).is_err());
        assert!(!fs.exists(Path::new("/refused")));
    }

    #[test]
    fn cli_sink_uses_all_teams_and_matches_live_serial_pixels() {
        let config = config();
        for window in [1, 2, 4] {
            let plan = plan(OutputPixelFormat::Rgba8, window);
            let fs = Arc::new(VirtualFs::new());
            let mut sink = RenderSink::new(
                fs.clone(),
                &config,
                &plan,
                &RenderTarget::Native(NativeFrameFormat::PngSequence),
                PathBuf::from("/frames"),
            )
            .unwrap();
            let mut serial = RetainedFrameRenderer::new(RetainedFrameRendererConfig {
                frame: resolved_frame_config(&config).unwrap(),
                tiling: Tiling {
                    macro_tile: plan.macro_tile,
                    fine_tile: plan.fine_tile,
                },
                engine: EngineIdentity::certified(),
                threads: 1,
            })
            .unwrap();
            let mut stage = Stage::new();
            let mob = stage.add(shape());
            stage.add_to_scene(mob).unwrap();
            let mut expected = Vec::new();
            for index in 0..12 {
                serial.render(&stage, 0).unwrap();
                let mut pixels =
                    FrameBuffer::new(FrameLayout::tight(PixelFormat::Rgba8, 32, 24).unwrap());
                rgba16f_to_rgba8(serial.frame(), &mut pixels).unwrap();
                expected.push(pixels.as_bytes().to_vec());
                sink.render_stage(&stage, index).unwrap();
                stage.shift(mob, [0.1, 0.0, 0.0]);
            }
            drop(stage);
            let finished = sink.finish(None).unwrap();
            let stats = finished.backend.pipeline.unwrap();
            assert_eq!((stats.submitted, stats.emitted), (12, 12));
            assert_eq!(stats.outstanding_slots, 0);
            assert!(stats.max_in_flight <= plan.frames_in_flight);
            assert_eq!(stats.render_team_frames.len(), plan.render_teams.len());
            assert!(stats.render_team_frames.iter().all(|n| *n > 0));
            for (index, expected) in expected.iter().enumerate() {
                let bytes = fs
                    .read(Path::new(&format!("/frames/frame_{index:06}.png")))
                    .unwrap();
                assert_eq!(
                    &decode_png(&bytes, &PngLimits::default()).unwrap().rgba,
                    expected
                );
            }
        }
    }

    #[test]
    fn gif_and_y4m_bytes_are_identical_across_cli_windows() {
        for (format, output) in [
            (NativeFrameFormat::Gif, OutputPixelFormat::Rgba8),
            (NativeFrameFormat::Y4m, OutputPixelFormat::Nv12),
        ] {
            let mut expected = None;
            for window in [1, 2, 4] {
                let fs = Arc::new(VirtualFs::new());
                let mut sink = RenderSink::new(
                    fs.clone(),
                    &config(),
                    &plan(output, window),
                    &RenderTarget::Native(format),
                    PathBuf::from("/movie"),
                )
                .unwrap();
                let mut stage = Stage::new();
                let mob = stage.add(shape());
                stage.add_to_scene(mob).unwrap();
                for index in 0..8 {
                    sink.render_stage(&stage, index).unwrap();
                    stage.shift(mob, [0.1, 0.0, 0.0]);
                }
                let finished = sink.finish(None).unwrap();
                assert_eq!(finished.backend.pipeline.unwrap().emitted, 8);
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
    fn occupied_destinations_and_competing_writers_are_preserved_at_commit() {
        for format in [
            NativeFrameFormat::Png,
            NativeFrameFormat::Gif,
            NativeFrameFormat::Y4m,
        ] {
            for competing in [false, true] {
                let fs = Arc::new(VirtualFs::new());
                let output = if matches!(format, NativeFrameFormat::Y4m) {
                    OutputPixelFormat::Nv12
                } else {
                    OutputPixelFormat::Rgba8
                };
                if !competing {
                    fs.insert("/movie", b"foreign generation".to_vec());
                }
                let mut sink = RenderSink::new(
                    fs.clone(),
                    &config(),
                    &plan(output, 2),
                    &RenderTarget::Native(format),
                    PathBuf::from("/movie"),
                )
                .unwrap();
                sink.render_stage(&Stage::new(), 0).unwrap();
                if competing {
                    fs.insert("/movie", b"foreign generation".to_vec());
                }
                assert!(sink.finish(None).is_err());
                assert_eq!(fs.read(Path::new("/movie")).unwrap(), b"foreign generation");
            }
        }
    }

    #[test]
    fn dropping_or_unwinding_cli_capture_joins_before_same_destination_retry() {
        for panic in [false, true] {
            let fs = Arc::new(VirtualFs::new());
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut sink = RenderSink::new(
                    fs.clone(),
                    &config(),
                    &plan(OutputPixelFormat::Rgba8, 4),
                    &RenderTarget::Native(NativeFrameFormat::PngSequence),
                    PathBuf::from("/retry"),
                )
                .unwrap();
                for index in 0..3 {
                    sink.render_stage(&Stage::new(), index).unwrap();
                }
                if panic {
                    panic!("scene failure after queued captures");
                }
                drop(sink);
            }));
            assert_eq!(result.is_err(), panic);
            assert!(!fs.exists(Path::new("/retry")));
            let mut retry = RenderSink::new(
                fs.clone(),
                &config(),
                &plan(OutputPixelFormat::Rgba8, 4),
                &RenderTarget::Native(NativeFrameFormat::PngSequence),
                PathBuf::from("/retry"),
            )
            .unwrap();
            retry.render_stage(&Stage::new(), 0).unwrap();
            assert_eq!(retry.finish(None).unwrap().backend.frames, 1);
        }
    }
}
