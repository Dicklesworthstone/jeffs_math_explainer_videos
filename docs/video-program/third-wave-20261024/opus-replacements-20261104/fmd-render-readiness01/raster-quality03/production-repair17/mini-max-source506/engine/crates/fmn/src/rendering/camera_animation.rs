//! Camera motion sampled from the same immutable scene state as the geometry.

use std::sync::Arc;

use fmn_anim::FramePacket;
use fmn_platform::fs::{FileSystem, StdFs};
use fmn_scene::{CameraRig, CaptureReason, IntegrationError, RuntimeConfig, Scene};

use super::{CameraConfig, RenderError, RenderOptions, RenderReport, RenderSink};
use crate::{ProgramAdapter, SceneConstruct};

/// Render ordinary native animation with a snapshot-native camera rig.
///
/// The factory creates a rig in the actual Scene, then returns the program
/// that animates it. It may allocate mobjects, attach updaters and draw random
/// values, but must not run playback or advance the clock before returning.
/// The program uses the normal Scene play/wait lifecycle. Camera trackers are
/// sampled after scene updaters from the very same capture as the geometry;
/// there is no second clock and no renderer-side animation callback.
///
/// `options.camera` supplies the initial pose and immutable capture policy.
/// When absent, the camera matches `options.camera_config()`. Center, width,
/// orientation, field of view and light are animated via [`CameraRig`].
/// Its existing quaternion interpolation is normalized linear interpolation,
/// not spherical interpolation or a constant-angular-speed promise.
///
/// A successful report is an artifact receipt, not a certified input closure.
/// Factory callbacks and external inputs still require closure attestation.
///
/// # Errors
/// Invalid options fail before invoking the factory. Invalid rig topology,
/// values or foreign handles fail without publishing a partial artifact.
/// Factory/scene errors retain their typed sources; unwinding cancels and
/// joins all native workers before control returns to the caller.
pub fn render_camera<F, P>(factory: F, options: RenderOptions) -> Result<RenderReport, RenderError>
where
    F: FnOnce(&mut Scene, &CameraConfig) -> crate::Result<(P, CameraRig)>,
    P: SceneConstruct,
{
    render_camera_with_fs(factory, options, Arc::new(StdFs))
}

/// [`render_camera`] with an explicit filesystem capability.
///
/// # Errors
/// Returns the same typed failures as [`render_camera`].
pub fn render_camera_with_fs<F, P>(
    factory: F,
    mut options: RenderOptions,
    fs: Arc<dyn FileSystem>,
) -> Result<RenderReport, RenderError>
where
    F: FnOnce(&mut Scene, &CameraConfig) -> crate::Result<(P, CameraRig)>,
    P: SceneConstruct,
{
    let base = match &options.camera {
        Some(camera) => camera.clone(),
        None => options.camera_config()?,
    };
    options.camera = Some(base.clone());
    let runtime = RuntimeConfig::from_config(&options.config);
    let mut scene = Scene::new(runtime, options.config.determinism.seed)
        .map_err(|error| RenderError::Scene(error.into()))?;
    let typesetting = options.typesetting_session(Arc::clone(&fs));
    let preflight_workers = options.typeset_preflight_workers;
    let inner = RenderSink::new(options, fs)?;
    let (mut program, rig) = factory(&mut scene, &base).map_err(RenderError::Scene)?;
    if scene.time().frames() != 0 || scene.play_count() != 0 {
        return Err(RenderError::InvalidOptions(
            "camera factory must not run playback before returning its program",
        ));
    }
    let mut sink = RigSink { inner, rig, base };
    sink.sync_camera(scene.stage())?;
    let mut adapter = ProgramAdapter {
        program: &mut program,
        front_door_error: None,
        typesetting: &typesetting,
        preflight_workers,
    };
    let run = scene.run(
        &mut adapter,
        &mut crate::SegmentAccountingSink {
            inner: &mut sink,
            typesetting: &typesetting,
        },
    );
    sink.inner.surface_failure()?;
    if let Some(error) = adapter.front_door_error.take() {
        return Err(RenderError::Scene(error));
    }
    let scene_report = run.map_err(|error| RenderError::Scene(error.into()))?;
    if sink.inner.next_sequence == 0 {
        sink.render_stage(scene.stage())?;
    }
    let finished = sink.inner.finish(Some(&scene))?;
    Ok(RenderReport {
        scene: scene_report,
        artifact: finished.artifact,
        emission: finished.emission,
        execution_plan: sink.inner.plan.clone(),
        frame_pipeline: finished.frame_pipeline,
        typesetting: typesetting.report(),
    })
}

struct RigSink {
    inner: RenderSink,
    rig: CameraRig,
    base: CameraConfig,
}

impl RigSink {
    fn sync_camera(&mut self, stage: &fmn_mobject::Stage) -> Result<(), RenderError> {
        let config = self
            .rig
            .sample(stage, &self.base)
            .map_err(|error| RenderError::Scene(error.into()))?;
        self.inner
            .pipeline
            .as_mut()
            .ok_or(RenderError::InvalidOptions(
                "camera frame pipeline was already finalized",
            ))?
            .update_camera(config)
    }

    fn render_stage(&mut self, stage: &fmn_mobject::Stage) -> Result<(), RenderError> {
        self.sync_camera(stage)?;
        self.inner.render_stage(stage)
    }
}

impl fmn_scene::SceneSink for RigSink {
    fn event(&mut self, event: fmn_scene::LifecycleEvent) -> Result<(), IntegrationError> {
        fmn_scene::SceneSink::event(&mut self.inner, event)
    }

    fn capture(&mut self, _: CaptureReason, packet: FramePacket) -> Result<(), IntegrationError> {
        if let Some(error) = &self.inner.failure {
            return Err(IntegrationError::new("native-camera", error.to_string()));
        }
        self.render_stage(&packet.materialize_stage())
            .map_err(|error| {
                let message = error.to_string();
                self.inner.failure = Some(error);
                IntegrationError::new("native-camera", message)
            })
    }
}
