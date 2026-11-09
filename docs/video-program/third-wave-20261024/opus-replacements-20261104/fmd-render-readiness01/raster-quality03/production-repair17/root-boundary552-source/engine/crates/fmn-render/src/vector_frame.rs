//! Owned affine frames for bounded, multi-frame CPU execution.
//!
//! Marionette and its callbacks stay on their scene owner. Only the existing
//! compiled Lumen IR crosses threads. Workers use the ordinary `FrameJob` and
//! content-keyed tile cache, so scheduling changes neither pixels nor semantics.

use std::sync::Arc;

use fmn_frame::{FrameBuffer, FrameError, FrameLayout};
use fmn_mobject::{ProgramKind, Stage};

use crate::{
    Binning, CachedRenderStats, FrameArena, FrameJob, MonoTable, PixelTileCache, RenderPlan,
    RenderPlanLimits, RetainedFrameRendererConfig, RetainedFrameRendererError, SyncStats,
};

/// Serial retained compiler whose captured frames own immutable IR generations.
///
/// Compiled shapes/styles retain their normal revision-based reuse. An IR
/// generation is copied only when a subsequent capture mutates it while an
/// earlier worker still holds it; no live arena or callback is made `Send`.
#[derive(Debug)]
pub struct VectorFrameCompiler {
    plan: Arc<RenderPlan>,
    config: RetainedFrameRendererConfig,
    plan_limits: RenderPlanLimits,
}

impl VectorFrameCompiler {
    /// Construct without allocating a raw frame or spawning any workers.
    ///
    /// # Errors
    /// Refuses an empty team or an invalid raw-frame layout.
    pub fn new(config: RetainedFrameRendererConfig) -> Result<Self, RetainedFrameRendererError> {
        if config.threads == 0 {
            return Err(RetainedFrameRendererError::InvalidThreads);
        }
        config.frame.layout()?;
        Ok(Self {
            plan: Arc::new(RenderPlan::new()),
            config,
            plan_limits: RenderPlanLimits::default(),
        })
    }

    /// Admit later captures under `limits` in place of [`RenderPlanLimits::default`].
    ///
    /// A larger budget costs retained memory and moves compaction later; frame
    /// bits do not depend on it. Downstream monotone and binning budgets still
    /// apply and refuse by name.
    pub fn set_plan_limits(&mut self, limits: RenderPlanLimits) {
        self.plan_limits = limits;
    }

    /// Freeze one affine frame after the caller has acquired pipeline capacity.
    ///
    /// Camera-only primitives are refused rather than flattened or omitted.
    /// Geometry/binning errors remain the existing typed Lumen errors.
    ///
    /// # Errors
    /// Refuses invalid records or content requiring the camera route.
    pub fn capture(
        &mut self,
        stage: &Stage,
        camera_revision: u64,
    ) -> Result<OwnedVectorFrame, RetainedFrameRendererError> {
        if let Some(item) = stage
            .draw_plan()
            .items()
            .iter()
            .find(|item| item.key.program != ProgramKind::Vector)
        {
            return Err(RetainedFrameRendererError::CameraRequired {
                program: item.key.program,
            });
        }
        Arc::make_mut(&mut self.plan).sync_with_limits(stage, camera_revision, self.plan_limits)?;
        Ok(OwnedVectorFrame {
            plan: Arc::clone(&self.plan),
            config: self.config,
            camera_revision,
        })
    }

    /// Work counters from the authoritative retained-plan synchronization.
    #[must_use]
    pub fn stats(&self) -> SyncStats {
        self.plan.stats()
    }
}

/// Immutable compiled affine frame, independent of the source stage's lifetime.
///
/// The scheduler owns the number of these in flight. Geometry is governed by
/// `RenderPlan`'s existing limits, separately from pixel-storage admission.
#[derive(Debug, Clone)]
pub struct OwnedVectorFrame {
    plan: Arc<RenderPlan>,
    config: RetainedFrameRendererConfig,
    camera_revision: u64,
}

impl OwnedVectorFrame {
    /// The RGBA16F surface layout [`Self::render_cached_into`] requires.
    ///
    /// # Errors
    /// Refuses a viewport too large for one frame allocation.
    pub fn raster_layout(&self) -> Result<FrameLayout, FrameError> {
        self.config.frame.layout()
    }

    /// Execute with caller-owned, worker-local scratch and retained pixels.
    ///
    /// Workers may receive nonconsecutive frames: cache keys name content, not
    /// completion order. Each worker must own its arena/cache exclusively.
    /// The returned frame is owned by the next pipeline stage, never by a live
    /// scene. No second rasterizer, sampling policy or color conversion exists.
    ///
    /// # Errors
    /// Preserves all ordinary preparation, binning and cached-render errors.
    pub fn render_cached(
        &self,
        threads: usize,
        arena: &mut FrameArena,
        cache: &mut PixelTileCache,
    ) -> Result<(FrameBuffer, CachedRenderStats), RetainedFrameRendererError> {
        let mut frame = FrameBuffer::new(self.config.frame.layout()?);
        let stats = self.render_cached_into(threads, arena, cache, &mut frame, None)?;
        Ok((frame, stats))
    }

    /// [`Self::render_cached`] into a caller-recycled RGBA16F surface.
    ///
    /// Every pixel is overwritten: reused tiles are restored and every other
    /// tile writes its background before any coverage, so a recycled surface
    /// yields the same bytes as a fresh one. Recycling skips the frame-sized
    /// zeroed allocation (66 MB at 4K) and its page faults on every frame.
    /// Row padding is never written. The surface must have this frame's
    /// RGBA16F layout.
    ///
    /// `resume` is the cache's [`PixelTileCache::serial`] from the render that
    /// last filled `frame`, when nothing has written it since. Reused tiles
    /// are then left in place instead of copied back from the cache. See
    /// [`crate::engine::FrameJob::render_into_cached_resuming`].
    ///
    /// # Errors
    /// As [`Self::render_cached`], plus a layout mismatch for `frame`.
    pub fn render_cached_into(
        &self,
        threads: usize,
        arena: &mut FrameArena,
        cache: &mut PixelTileCache,
        frame: &mut FrameBuffer,
        resume: Option<u64>,
    ) -> Result<CachedRenderStats, RetainedFrameRendererError> {
        if threads == 0 {
            return Err(RetainedFrameRendererError::InvalidThreads);
        }
        let mono = MonoTable::build(&self.plan, self.config.frame.map)?;
        let mut binning = Binning::build(
            &self.plan,
            self.config.frame.viewport,
            self.config.tiling,
            self.config.frame.map,
        )?;
        binning.prune_occluded(&self.plan)?;
        let job = FrameJob::with_identity_in(
            arena,
            &self.plan,
            &mono,
            &binning,
            self.config.frame,
            self.config.engine,
        )?;
        Ok(job.render_into_cached_resuming(threads, frame, self.camera_revision, cache, resume)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EngineIdentity, FrameConfig, RetainedFrameRenderer, ScreenMap, Tiling, Viewport};
    use fmn_core::color::LinearRgba;
    use fmn_mobject::{Mobject, RecordBuffer, RecordSchema, RenderPrimitive};

    fn config() -> RetainedFrameRendererConfig {
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
                LinearRgba {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
            ),
            tiling: Tiling {
                macro_tile: 16,
                fine_tile: 8,
            },
            engine: EngineIdentity::certified(),
            threads: 1,
        }
    }

    fn vector() -> Mobject {
        let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), 3).unwrap();
        buffer.write_range(
            "point",
            0,
            &[-1.5, -1.0, 0.0, 0.0, 1.25, 0.0, 1.5, -1.0, 0.0],
        );
        buffer.write_range("fill_rgba", 0, &[0.1, 0.3, 1.0, 0.8].repeat(3));
        Mobject::from_buffer(buffer)
    }

    #[test]
    fn immutable_ir_is_send_sync_and_outlives_stage() {
        fn send_sync<T: Send + Sync>() {}
        send_sync::<OwnedVectorFrame>();
        let mut compiler = VectorFrameCompiler::new(config()).unwrap();
        let frame = {
            let mut stage = Stage::new();
            let mob = stage.add(vector());
            stage.add_to_scene(mob).unwrap();
            compiler.capture(&stage, 0).unwrap()
        };
        drop(compiler);
        std::thread::spawn(move || {
            let (pixels, _) = frame
                .render_cached(2, &mut FrameArena::new(), &mut PixelTileCache::new())
                .unwrap();
            assert_eq!(pixels.layout().width(), 32);
        })
        .join()
        .unwrap();
    }

    #[test]
    fn frozen_frames_match_retained_renderer_after_later_mutation_and_out_of_order_execution() {
        let mut stage = Stage::new();
        let mob = stage.add(vector());
        stage.add_to_scene(mob).unwrap();
        let mut compiler = VectorFrameCompiler::new(config()).unwrap();
        let mut serial = RetainedFrameRenderer::new(config()).unwrap();
        let mut frames = Vec::new();
        let mut expected = Vec::new();
        for _ in 0..5 {
            frames.push(compiler.capture(&stage, 0).unwrap());
            serial.render(&stage, 0).unwrap();
            expected.push(serial.frame().as_bytes().to_vec());
            stage.shift(mob, [0.25, 0.1, 0.0]);
        }
        stage.remove_from_scene(mob);
        assert_ne!(expected[0], expected[4]);
        for threads in [1, 4, 16] {
            let mut arena = FrameArena::new();
            let mut cache = PixelTileCache::new();
            for index in [4, 1, 3, 0, 2, 2] {
                let (pixels, _) = frames[index]
                    .render_cached(threads, &mut arena, &mut cache)
                    .unwrap();
                assert_eq!(pixels.as_bytes(), expected[index]);
            }
        }
    }

    #[test]
    fn static_captures_keep_shape_reuse_and_worker_tile_hits() {
        let mut stage = Stage::new();
        let mob = stage.add(vector());
        stage.add_to_scene(mob).unwrap();
        let mut compiler = VectorFrameCompiler::new(config()).unwrap();
        let mut arena = FrameArena::new();
        let mut cache = PixelTileCache::new();
        let first = compiler.capture(&stage, 0).unwrap();
        let (before, _) = first.render_cached(1, &mut arena, &mut cache).unwrap();
        let second = compiler.capture(&stage, 0).unwrap();
        assert_eq!(compiler.stats().shapes_compiled, 0);
        assert_eq!(compiler.stats().shapes_reused, 1);
        let (after, stats) = second.render_cached(4, &mut arena, &mut cache).unwrap();
        assert_eq!(before.as_bytes(), after.as_bytes());
        assert!(stats.cache.hits > 0);
    }

    #[test]
    fn camera_content_and_zero_teams_are_not_silently_accepted() {
        let mut invalid = config();
        invalid.threads = 0;
        assert!(matches!(
            VectorFrameCompiler::new(invalid),
            Err(RetainedFrameRendererError::InvalidThreads)
        ));
        let mut stage = Stage::new();
        let mob = stage.add(vector().with_render_primitive(RenderPrimitive::DotCloud));
        stage.add_to_scene(mob).unwrap();
        let mut compiler = VectorFrameCompiler::new(config()).unwrap();
        assert!(matches!(
            compiler.capture(&stage, 0),
            Err(RetainedFrameRendererError::CameraRequired { .. })
        ));
    }
}
