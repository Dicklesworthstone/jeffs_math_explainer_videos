//! Native output for compiled timelines, with reconstruction on render teams.

use fmn_anim::RationalFrameClock;
use fmn_platform::fs::{FileSystem, StdFs};
use fmn_scene::SceneRunReport;
use fmn_scene::timeline_bundle::TimelineBundle;
use std::sync::Arc;

use super::{RenderError, RenderOptions, RenderReport, RenderSink};

/// Render a validated FMTL artifact through the native bounded CPU pipeline.
///
/// The bundle's clock and export-time reconstruction proof remain authoritative.
/// Pure segments reconstruct from their endpoints on independent render teams;
/// stateful segments use their recorded captures and never replay callbacks.
/// Output is strictly frame-index ordered. This is compiled replay, not a claim
/// that arbitrary live native scenes have become frame-parallel.
///
/// `options.config.camera.fps` must equal the artifact's frame rate. No frame
/// resampling, alpha-zero insertion or terminal still is performed. An empty
/// compiled artifact is rejected before creating an output sink. `options.camera`
/// selects the existing fixed-camera path for surfaces, images and dot clouds.
/// Camera-bearing minor-1 bundles instead select the native camera route
/// automatically and replay their recorded camera/light/background samples.
/// An explicit `options.camera` is refused for those bundles, not silently
/// substituted for the recorded view. Minor-0 behavior is unchanged.
///
/// The loaded bundle and worker-local reconstructed geometry are additional
/// to `max_resident_bytes`, which bounds planned pixels and native sink storage.
/// The production bundle decoder bounds canonical input/snapshot storage.
/// The receipt is not a full input-closure certification or an encoded-video
/// determinism promise.
///
/// # Errors
/// Preserves bundle/schema/engine failures and normal native output failures.
/// Earlier private frames are aborted on reconstruction, raster or sink error.
pub fn render_bundle(bytes: &[u8], options: RenderOptions) -> Result<RenderReport, RenderError> {
    render_bundle_with_fs(bytes, options, Arc::new(StdFs))
}

/// [`render_bundle`] with an explicit filesystem capability.
///
/// # Errors
/// Returns the same typed failures as [`render_bundle`].
pub fn render_bundle_with_fs(
    bytes: &[u8],
    mut options: RenderOptions,
    fs: Arc<dyn FileSystem>,
) -> Result<RenderReport, RenderError> {
    let bundle = TimelineBundle::from_bytes(bytes).map_err(RenderError::Bundle)?;
    if bundle.fps() != options.config.camera.fps {
        return Err(RenderError::InvalidOptions(
            "compiled frame rate must match export configuration",
        ));
    }
    if bundle.frame_count() == 0 {
        return Err(RenderError::InvalidOptions(
            "compiled artifact has no frames to render",
        ));
    }
    if u64::from(bundle.frame_count()) > options.max_frames {
        return Err(RenderError::InvalidOptions(
            "compiled artifact exceeds max_frames",
        ));
    }
    if bundle.has_camera_track() {
        if options.camera.is_some() {
            return Err(RenderError::InvalidOptions(
                "an explicit camera cannot override a recorded FMTL camera track",
            ));
        }
        // Select the correctly budgeted camera CPU route. Each compiled job
        // supplies its exact captured pose/background/light, not this baseline.
        options.camera = Some(options.camera_config()?);
    }
    let shared = bundle.into_shared().map_err(RenderError::Bundle)?;
    let mut clock = RationalFrameClock::new(shared.fps())
        .map_err(|error| RenderError::Scene(fmn_anim::AnimError::Clock(error).into()))?;
    clock
        .advance_frames(i64::from(shared.frame_count()))
        .map_err(|error| RenderError::Scene(fmn_anim::AnimError::Clock(error).into()))?;
    let scene = SceneRunReport {
        ended_early: false,
        play_count: u64::try_from(shared.segment_count())
            .map_err(|_| RenderError::InvalidOptions("compiled segment count exceeds u64"))?,
        time: clock.now(),
    };
    let mut sink = RenderSink::new(options, fs)?;
    for index in 0..shared.frame_count() {
        let job = shared.frame_job(index).map_err(RenderError::Bundle)?;
        let pipeline = sink.pipeline.as_mut().ok_or(RenderError::InvalidOptions(
            "compiled pipeline was already finalized",
        ))?;
        if let Err(error) = pipeline.capture_compiled(job, u64::from(index)) {
            if matches!(&error, RenderError::Pipeline(_)) {
                if let Some(emitter) = &sink.emitter {
                    emitter.cancel();
                }
                if let Some(pipeline) = sink.pipeline.take() {
                    pipeline.finish()?;
                }
            }
            return Err(error);
        }
    }
    let finished = sink.finish(None)?;
    Ok(RenderReport {
        scene,
        artifact: finished.artifact,
        emission: finished.emission,
        execution_plan: sink.plan.clone(),
        frame_pipeline: finished.frame_pipeline,
        typesetting: fmn_tex::TypesetSessionReport::default(),
    })
}
