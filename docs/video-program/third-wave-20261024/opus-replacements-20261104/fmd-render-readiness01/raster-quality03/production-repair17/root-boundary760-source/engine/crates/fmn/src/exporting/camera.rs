//! Native camera-rig authoring compiled into code-free camera-bearing FMTL.

use std::path::Path;
use std::sync::Arc;

use fmn_anim::FramePacket;
use fmn_platform::fs::{FileSystem, StdFs};
use fmn_render::{Camera, CameraConfig};
use fmn_scene::recording::{RecordingError, SceneBundleRecorder};
use fmn_scene::{
    CameraRig, CaptureReason, IntegrationError, LifecycleEvent, RuntimeConfig, Scene, SceneError,
    SceneSink,
};

use super::{BundleExportError, BundleExportOptions, BundleExportReport, SceneBundleExport};
use crate::{ProgramAdapter, SceneConstruct};

fn scene_error(error: SceneError) -> BundleExportError {
    BundleExportError::Scene(error.into())
}

fn recording_error(error: RecordingError) -> BundleExportError {
    match error {
        RecordingError::OutputLimit { needed, limit } => {
            BundleExportError::OutputLimit { needed, limit }
        }
        error => BundleExportError::Recording(error),
    }
}

/// Run a native camera-rig scene once and record its observed pictures and view.
///
/// The factory allocates a [`CameraRig`] in the actual Scene and returns its
/// normal [`SceneConstruct`] program. It may create objects and updaters but
/// cannot run playback before capture begins. The existing Scene lifecycle and
/// rational clock own animation: the rig is sampled from the same immutable
/// post-updater packet as geometry, never from a later live Stage.
///
/// The camera supplies capture dimensions/aspect, background and sampling
/// policy; its FPS must match the effective Scene FPS in `options`. The program
/// animates pose, field of view and light via the rig's existing trackers. A
/// static scene produces one terminal still without running updaters again.
/// FMTL/1 minor 1 records the resulting cameras, not the process-local rig.
/// Replay through `rendering::render_bundle` or the standalone CLI needs neither
/// this factory nor the original scene program. It never reruns their code.
///
/// # Errors
/// Invalid budgets, camera values or FPS fail before invoking the factory.
/// Invalid/foreign rig handles, capture failures and errors caught by scene code
/// remain failures: no partial bundle is returned. Audio is refused explicitly.
/// Byte limits cover capture and encoding, not arbitrary authored allocations.
/// No filesystem operation occurs here, including during unwinding.
pub fn export_camera_bundle_bytes<F, P>(
    factory: F,
    camera: CameraConfig,
    options: BundleExportOptions,
) -> Result<SceneBundleExport, BundleExportError>
where
    F: FnOnce(&mut Scene, &CameraConfig) -> crate::Result<(P, CameraRig)>,
    P: SceneConstruct,
{
    options.validate()?;
    let runtime = RuntimeConfig::from_config(&options.config);
    let fps = runtime.effective_fps();
    if camera.fps != fps {
        return Err(BundleExportError::InvalidOptions(
            "camera frame rate must match the effective scene frame rate",
        ));
    }
    Camera::new(camera.clone()).map_err(|error| scene_error(error.into()))?;
    let mut scene = Scene::new(runtime, options.config.determinism.seed).map_err(scene_error)?;
    let recorder = SceneBundleRecorder::new_render_only_with_camera(fps, options.limits)
        .map_err(recording_error)?;
    let (mut program, rig) = factory(&mut scene, &camera).map_err(BundleExportError::Scene)?;
    if scene.time().frames() != 0 || scene.play_count() != 0 {
        return Err(BundleExportError::InvalidOptions(
            "camera factory must not run playback before returning its program",
        ));
    }
    let mut sink = RigRecorder {
        recorder,
        rig,
        camera,
        failure: None,
    };
    sink.sample(scene.stage())?;
    let typesetting = fmn_tex::TexSession::memory(options.config.tex.template.clone());
    let mut adapter = ProgramAdapter {
        program: &mut program,
        front_door_error: None,
        typesetting: &typesetting,
        preflight_workers: crate::typesetting::DEFAULT_PREFLIGHT_WORKERS,
    };
    let run = scene.run(&mut adapter, &mut sink);
    if let Some(error) = sink.failure.take() {
        return Err(error);
    }
    let result = match adapter.front_door_error.take() {
        Some(error) => Err(error),
        None => run.map_err(crate::Error::from),
    };
    let report = match result {
        Ok(report) => report,
        Err(error) => {
            return Err(match sink.recorder.into_error() {
                Some(error) => recording_error(error),
                None => BundleExportError::Scene(error),
            });
        }
    };
    if !scene.sound_requests().is_empty() {
        return Err(BundleExportError::Capability(
            "FMTL/1 cannot carry audio; use the audio-capable render/output path instead",
        ));
    }
    if sink.recorder.frame_count() == 0 {
        let camera = sink.sample(scene.stage())?;
        // The recorder retains a typed terminal-capture refusal through finish.
        let _ = sink
            .recorder
            .capture_terminal_still_with_camera(scene.stage(), &camera);
    }
    let bundle = sink
        .recorder
        .finish_with_max_bytes(options.max_output_bytes)
        .map_err(recording_error)?;
    Ok(SceneBundleExport {
        scene: report,
        bundle,
    })
}

/// Publish a camera-rig scene as a new code-free `.fmtl` file.
///
/// # Errors
/// As [`export_camera_bundle_bytes`], plus atomic create-only filesystem errors.
/// All scene work and validation finish before staging the destination.
pub fn export_camera_bundle<F, P>(
    factory: F,
    output: impl AsRef<Path>,
    camera: CameraConfig,
    options: BundleExportOptions,
) -> Result<BundleExportReport, BundleExportError>
where
    F: FnOnce(&mut Scene, &CameraConfig) -> crate::Result<(P, CameraRig)>,
    P: SceneConstruct,
{
    export_camera_bundle_with_fs(factory, output, camera, options, Arc::new(StdFs))
}

/// [`export_camera_bundle`] with an explicit filesystem capability.
///
/// # Errors
/// Preserves original export/FS failures and never replaces an existing node.
pub fn export_camera_bundle_with_fs<F, P>(
    factory: F,
    output: impl AsRef<Path>,
    camera: CameraConfig,
    options: BundleExportOptions,
    fs: Arc<dyn FileSystem>,
) -> Result<BundleExportReport, BundleExportError>
where
    F: FnOnce(&mut Scene, &CameraConfig) -> crate::Result<(P, CameraRig)>,
    P: SceneConstruct,
{
    let output = output.as_ref();
    if output.as_os_str().is_empty() || output.file_name().is_none() {
        return Err(BundleExportError::InvalidOptions(
            "bundle output must name a file",
        ));
    }
    let export = export_camera_bundle_bytes(factory, camera, options)?;
    super::publish(export, output, fs)
}

struct RigRecorder {
    recorder: SceneBundleRecorder,
    rig: CameraRig,
    camera: CameraConfig,
    failure: Option<BundleExportError>,
}

impl RigRecorder {
    fn sample(&self, stage: &fmn_mobject::Stage) -> Result<Camera, BundleExportError> {
        let config = self.rig.sample(stage, &self.camera).map_err(scene_error)?;
        Camera::new(config).map_err(|error| scene_error(error.into()))
    }
}

impl SceneSink for RigRecorder {
    fn event(&mut self, event: LifecycleEvent) -> Result<(), IntegrationError> {
        self.recorder.event(event)
    }

    fn capture(
        &mut self,
        reason: CaptureReason,
        packet: FramePacket,
    ) -> Result<(), IntegrationError> {
        if let Some(error) = &self.failure {
            return Err(IntegrationError::new("fmtl-camera", error.to_string()));
        }
        match self.sample(&packet.materialize_stage()) {
            Ok(camera) => self.recorder.capture_with_camera(reason, packet, &camera),
            Err(error) => {
                let message = error.to_string();
                self.failure = Some(error);
                Err(IntegrationError::new("fmtl-camera", message))
            }
        }
    }
}
