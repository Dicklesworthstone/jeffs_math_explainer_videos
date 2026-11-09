//! Python-to-FMTL composition over the shared SceneBundleRecorder.
//!
//! The existing host segment observer supplies begin/end boundaries; the
//! existing native capture sink supplies the actual post-updater FramePackets.
//! No callback, animation law, serializer, or frame clock is reimplemented.

use super::*;
use fmn_anim::SegmentKind;
use fmn_platform::fs::{FileSystem, StdFs};
use fmn_scene::recording::{RecordingError, SceneBundleRecorder};
use fmn_scene::{BundleExportLimits, CaptureReason, LifecycleEvent, LifecyclePhase, SceneSink};

pub(crate) fn install(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.setattr(
        "_portal_begin_bundle",
        wrap_pyfunction!(_portal_begin_bundle, module)?,
    )?;
    module.setattr(
        "_portal_begin_camera_bundle",
        wrap_pyfunction!(_portal_begin_camera_bundle, module)?,
    )?;
    module.setattr(
        "_portal_bundle_segment",
        wrap_pyfunction!(_portal_bundle_segment, module)?,
    )?;
    module.setattr(
        "_portal_finish_bundle",
        wrap_pyfunction!(_portal_finish_bundle, module)?,
    )?;
    Ok(())
}

pub(super) struct BundleCapture {
    recorder: SceneBundleRecorder,
    destination: PathBuf,
    max_output_bytes: usize,
    expected_camera: Camera,
    camera_track: bool,
    active: Option<(SegmentKind, u64)>,
    failure: Option<String>,
}

fn capability(detail: &str) -> PyErr {
    CapabilityError::new_err(format!(
        "FMTL/1 export: {detail}; use native PNG/video rendering instead"
    ))
}

impl BundleCapture {
    pub(super) fn is_empty(&self) -> bool {
        self.recorder.frame_count() == 0
    }

    fn fail(&mut self, error: &PyErr) {
        if self.failure.is_none() {
            self.failure = Some(error.to_string());
        }
    }

    pub(super) fn validate_camera(&mut self, camera: &Camera) -> PyResult<()> {
        let expected = &self.expected_camera;
        if self.camera_track {
            // The export viewport/FPS are frozen before acquiring this owner.
            // Pose, lighting and background may change at every capture.
            if camera.fps() != expected.fps() || camera.pixel_shape() != expected.pixel_shape() {
                let error =
                    capability("camera FPS and pixel dimensions cannot change during export");
                self.fail(&error);
                return Err(error);
            }
            return Ok(());
        }
        let frame = camera.frame();
        let baseline = expected.frame();
        // Revisions are not semantics: a descriptor may set a value to itself.
        // Minor-0 FMTL has no camera/background track; never silently drop one.
        if frame.center() != baseline.center()
            || frame.shape() != baseline.shape()
            || frame.orientation() != baseline.orientation()
            || frame.field_of_view() != baseline.field_of_view()
            || camera.background() != expected.background()
            || camera.light_source_position() != expected.light_source_position()
        {
            let error = capability("camera, light and background tracks are not representable");
            self.fail(&error);
            return Err(error);
        }
        Ok(())
    }

    pub(super) fn capture(
        &mut self,
        packet: fmn_anim::FramePacket,
        stage: &Stage,
        camera: &Camera,
    ) -> PyResult<()> {
        self.validate_camera(camera)?;
        if !self.camera_track {
            Self::validate_stage(stage)?;
        }
        let reason = if packet.segment_frame() == 0 {
            CaptureReason::Show
        } else {
            CaptureReason::Segment
        };
        if self.camera_track {
            self.recorder.capture_with_camera(reason, packet, camera)
        } else {
            self.recorder.capture(reason, packet)
        }
        .map_err(native_error)
    }

    fn validate_stage(stage: &Stage) -> PyResult<()> {
        // A flat bundle is portable to both existing consumers. In particular,
        // a z-bearing Vector does not currently set requires_camera(), so the
        // shared rule tests actual point lanes rather than only program tags.
        match crate::planar_stage_refusal(stage) {
            Some(reason) => Err(capability(reason)),
            None => Ok(()),
        }
    }
}

#[pyfunction]
#[allow(clippy::too_many_arguments)]
fn _portal_begin_bundle(
    scene: &Bound<'_, PyScene>,
    destination: String,
    width: u32,
    height: u32,
    fps: u32,
    seed: u64,
    max_frames: u64,
    max_capture_bytes: usize,
    max_output_bytes: usize,
) -> PyResult<()> {
    begin_bundle(
        scene,
        destination,
        width,
        height,
        fps,
        seed,
        max_frames,
        max_capture_bytes,
        max_output_bytes,
        false,
    )
}

#[pyfunction]
#[allow(clippy::too_many_arguments)]
fn _portal_begin_camera_bundle(
    scene: &Bound<'_, PyScene>,
    destination: String,
    width: u32,
    height: u32,
    fps: u32,
    seed: u64,
    max_frames: u64,
    max_capture_bytes: usize,
    max_output_bytes: usize,
) -> PyResult<()> {
    begin_bundle(
        scene,
        destination,
        width,
        height,
        fps,
        seed,
        max_frames,
        max_capture_bytes,
        max_output_bytes,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
fn begin_bundle(
    scene: &Bound<'_, PyScene>,
    destination: String,
    width: u32,
    height: u32,
    fps: u32,
    seed: u64,
    max_frames: u64,
    max_capture_bytes: usize,
    max_output_bytes: usize,
    camera_track: bool,
) -> PyResult<()> {
    if destination.is_empty() || destination.contains('\0') {
        return Err(PyValueError::new_err(
            "bundle destination must be nonempty and contain no NUL",
        ));
    }
    let maximum = fmn_hash::serial::Limits::DEFAULT.max_total;
    if max_frames == 0
        || max_frames > u64::from(u32::MAX)
        || max_capture_bytes == 0
        || max_capture_bytes > maximum
        || max_output_bytes == 0
        || max_output_bytes > maximum
    {
        return Err(PyValueError::new_err(
            "bundle limits require 1..u32::MAX frames and 1..256 MiB byte budgets",
        ));
    }
    let destination = std::path::absolute(destination).map_err(native_error)?;
    if StdFs
        .node_kind_no_follow(&destination)
        .map_err(native_error)?
        .is_some()
    {
        return Err(pyo3::exceptions::PyFileExistsError::new_err(
            "bundle destination already exists",
        ));
    }
    portal_recording::check_available(scene)?;
    portal_video::check_scene_ownership(scene)?;
    let limits = BundleExportLimits {
        max_frames,
        max_capture_bytes,
    };
    // Portable bundles replay pictures, not the arena's authoring history.
    // Do not retain hidden target/saved copies or Python proxy pins per frame.
    let recorder = if camera_track {
        SceneBundleRecorder::new_render_only_with_camera(fps, limits)
    } else {
        SceneBundleRecorder::new_render_only(fps, limits)
    }
    .map_err(native_error)?;
    // Reuse the capture owner's configured camera and validation. The bundle
    // branch never rasterizes, populates Studio history, or starts a worker.
    let identity = protocol_digest(&[]);
    let (mut capture, runtime) = Capture::new(
        "fmtl-export".to_owned(),
        identity,
        identity,
        width,
        height,
        fps,
        1,
        1, // The unused Studio history is empty; only recorder limits apply.
        1,
    )?;
    // The capture camera's frame was resized to the export's pixel aspect, but
    // the Scene's CameraFrame keeps its configured shape. A planar bundle
    // refuses any departure from that default; comparing against the resized
    // frame instead refused every export whose aspect is not 16:9.
    let mut expected_camera = capture.camera.clone();
    *expected_camera.frame_mut() = CameraConfig::default().frame;
    capture.bundle = Some(BundleCapture {
        recorder,
        destination,
        max_output_bytes,
        expected_camera,
        camera_track,
        active: None,
        failure: None,
    });
    let replacement = Scene::new(runtime, seed).map_err(native_error)?;
    let slot = Arc::clone(&scene.borrow().render);
    let mut slot = slot
        .lock()
        .map_err(|_| PyRuntimeError::new_err("bundle output lock poisoned"))?;
    if slot.is_some() {
        return Err(PyRuntimeError::new_err(
            "a render generation is already active",
        ));
    }
    let mut owner = scene.try_borrow_mut()?;
    owner.engine = Rc::new(EngineState::new(replacement));
    owner.render_invocations.clear();
    owner.render_audio_inputs.clear();
    *slot = Some(PortalRenderSession::Preview(Box::new(capture)));
    Ok(())
}

/// Called by the existing Scene.play/wait segment-owner hook, not a frame loop.
#[pyfunction]
fn _portal_bundle_segment(scene: &Bound<'_, PyScene>, kind: &str, begin: bool) -> PyResult<()> {
    let kind = match kind {
        "play" => SegmentKind::Play,
        "wait" => SegmentKind::Wait,
        _ => return Err(PyValueError::new_err("bundle segment must be play or wait")),
    };
    let engine = Rc::clone(&scene.borrow().engine);
    let live = engine.borrow();
    let time = live.time();
    let index = live.play_count();
    let skipping = live.is_skipping();
    drop(live);
    let slot = Arc::clone(&scene.borrow().render);
    let mut slot = slot
        .lock()
        .map_err(|_| PyRuntimeError::new_err("bundle output lock poisoned"))?;
    let Some(PortalRenderSession::Preview(capture)) = slot.as_mut() else {
        return Err(PyRuntimeError::new_err("no bundle capture is active"));
    };
    let bundle = capture
        .bundle
        .as_mut()
        .ok_or_else(|| PyRuntimeError::new_err("active capture is not a bundle"))?;
    if let Some(error) = &bundle.failure {
        return Err(PyRuntimeError::new_err(error.clone()));
    }
    let result = (|| {
        let (phase, play_index) = if begin {
            if bundle.active.is_some() {
                return Err(PyRuntimeError::new_err("nested bundle segment"));
            }
            bundle.active = Some((kind, index));
            (LifecyclePhase::DriveSegment, index)
        } else {
            let (started_kind, started_index) = bundle
                .active
                .take()
                .ok_or_else(|| PyRuntimeError::new_err("bundle segment did not begin"))?;
            if started_kind != kind || started_index.checked_add(1) != Some(index) {
                return Err(PyRuntimeError::new_err(
                    "bundle segment did not complete its native lifecycle",
                ));
            }
            (LifecyclePhase::FinishSegment, started_index)
        };
        bundle
            .recorder
            .event(LifecycleEvent {
                phase,
                play_index,
                time,
                segment: Some(kind),
                skipping,
            })
            .map_err(native_error)
    })();
    if let Err(error) = &result {
        bundle.fail(error);
    }
    result
}

#[pyfunction]
fn _portal_finish_bundle(
    scene: &Bound<'_, PyScene>,
) -> PyResult<(String, u32, usize, usize, String)> {
    let engine = Rc::clone(&scene.borrow().engine);
    let slot = Arc::clone(&scene.borrow().render);
    // Validate ownership before invoking any authored camera descriptor.
    {
        let slot = slot
            .lock()
            .map_err(|_| PyRuntimeError::new_err("bundle output lock poisoned"))?;
        if !matches!(slot.as_ref(), Some(PortalRenderSession::Preview(c)) if c.bundle.is_some()) {
            return Err(PyRuntimeError::new_err("no bundle capture is active"));
        }
    }
    portal_playback::synchronize(scene)?;
    synchronize_portal_camera(scene)?;
    let mut slot = slot
        .lock()
        .map_err(|_| PyRuntimeError::new_err("bundle output lock poisoned"))?;
    if !Rc::ptr_eq(&engine, &scene.borrow().engine)
        || !matches!(slot.as_ref(), Some(PortalRenderSession::Preview(c)) if c.bundle.is_some())
    {
        return Err(PyRuntimeError::new_err(
            "bundle ownership changed during finish",
        ));
    }
    let Some(PortalRenderSession::Preview(mut capture)) = slot.take() else {
        unreachable!()
    };
    drop(slot);
    if capture.live {
        return Err(capability("live Studio input cannot share a bundle export"));
    }
    if let Some(error) = capture.failure.take() {
        return Err(PyRuntimeError::new_err(error));
    }
    let mut bundle = capture.bundle.take().expect("checked bundle owner");
    if let Some(error) = bundle.failure.take() {
        return Err(PyRuntimeError::new_err(error));
    }
    let live = engine.borrow();
    if live.is_skipping() {
        return Err(capability(
            "skipped playback is not an exportable full timeline",
        ));
    }
    if !live.sound_requests().is_empty() {
        return Err(capability("audio is not carried by this format"));
    }
    if bundle.is_empty() {
        if bundle.camera_track {
            bundle
                .recorder
                .capture_terminal_still_with_camera(live.stage(), &capture.camera)
        } else {
            BundleCapture::validate_stage(live.stage())?;
            bundle.recorder.capture_terminal_still(live.stage())
        }
        .map_err(native_error)?;
    }
    drop(live);
    let artifact = bundle
        .recorder
        .finish_with_max_bytes(bundle.max_output_bytes)
        .map_err(|error| match error {
            // A spent output budget is a runtime limit, not a bad argument:
            // raise it as the capture-budget refusals above are raised.
            RecordingError::OutputLimit { .. } => PyRuntimeError::new_err(error.to_string()),
            other => native_error(other),
        })?;
    let result = (
        bundle.destination.to_string_lossy().into_owned(),
        artifact.frame_count,
        artifact.segment_count,
        artifact.bytes.len(),
        artifact.digest.to_hex(),
    );
    // All validation completes before filesystem mutation. commit_new is the
    // race-safe arbiter, including existing directories and dangling symlinks.
    let mut writer = Arc::new(StdFs)
        .begin_atomic_file(&bundle.destination)
        .map_err(native_error)?;
    writer.write(&artifact.bytes).map_err(native_error)?;
    writer
        .prepare()
        .map_err(native_error)?
        .commit_new()
        .map_err(|error| match &error {
            // Losing the create-only race is the same condition begin_bundle
            // reports for an occupied destination.
            fmn_platform::fs::FsError::Io { err, .. }
                if err.kind() == std::io::ErrorKind::AlreadyExists =>
            {
                pyo3::exceptions::PyFileExistsError::new_err(error.to_string())
            }
            _ => native_error(error),
        })?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pyfunction]
    fn _test_bundle_frames(
        bytes: Vec<u8>,
        indices: Vec<u32>,
        width: u32,
        height: u32,
        threads: usize,
    ) -> PyResult<Vec<Vec<u8>>> {
        let bundle = fmn_scene::TimelineBundle::from_bytes(&bytes).map_err(native_error)?;
        assert!(!bundle.requires_camera());
        let identity = protocol_digest(&[]);
        let (mut output, _) = Capture::new(
            "replay-test".to_owned(),
            identity,
            identity,
            width,
            height,
            bundle.fps(),
            threads,
            1,
            1,
        )?;
        indices
            .into_iter()
            .map(|index| {
                let stage = bundle.stage_at(index).map_err(native_error)?;
                output.renderer.render(&stage, 0).map_err(native_error)?;
                portal_video::convert_frame(output.renderer.frame(), &mut output.rgba, None)
                    .map_err(native_error)?;
                Ok(output.rgba.plane(0).to_vec())
            })
            .collect()
    }

    #[pyfunction]
    fn _test_camera_bundle_frames(
        bytes: Vec<u8>,
        indices: Vec<u32>,
        width: u32,
        height: u32,
        threads: usize,
    ) -> PyResult<Vec<Vec<u8>>> {
        let bundle = fmn_scene::TimelineBundle::from_bytes(&bytes).map_err(native_error)?;
        assert!(bundle.has_camera_track());
        let identity = protocol_digest(&[]);
        let (mut output, _) = Capture::new(
            "camera-replay-test".to_owned(),
            identity,
            identity,
            width,
            height,
            bundle.fps(),
            threads,
            1,
            1,
        )?;
        indices
            .into_iter()
            .map(|index| {
                let (stage, camera) = bundle
                    .stage_at_with_camera(index, (width, height))
                    .map_err(native_error)?;
                output
                    .renderer
                    .render_with_camera(&stage, &camera.expect("validated camera track"))
                    .map_err(native_error)?;
                portal_video::convert_frame(output.renderer.frame(), &mut output.rgba, None)
                    .map_err(native_error)?;
                Ok(output.rgba.plane(0).to_vec())
            })
            .collect()
    }

    #[pyfunction]
    fn _test_png_pixels(path: &str) -> PyResult<Vec<u8>> {
        let bytes = std::fs::read(path).map_err(native_error)?;
        let image = fmn_codec::decode_png(&bytes, &fmn_codec::PngLimits::default())
            .map_err(native_error)?;
        Ok(image.rgba)
    }

    #[test]
    fn native_python_scene_bundle_export() {
        crate::with_python_test_module("Python scene bundle export", |py, module, globals| {
            globals
                .set_item(
                    "_test_bundle_frames",
                    wrap_pyfunction!(_test_bundle_frames, module).unwrap(),
                )
                .unwrap();
            globals
                .set_item(
                    "_test_png_pixels",
                    wrap_pyfunction!(_test_png_pixels, module).unwrap(),
                )
                .unwrap();
            globals
                .set_item(
                    "__file__",
                    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/bundle_export.py"),
                )
                .unwrap();
            let source = CString::new(include_str!("../../tests/bundle_export.py")).unwrap();
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("real Scene capture, FMTL replay, pixels and no-clobber publication");
        });
    }

    #[test]
    fn compact_python_scene_bundle_export() {
        crate::with_python_test_module(
            "Compact Python scene bundle export",
            |py, module, globals| {
                globals
                    .set_item(
                        "_test_bundle_frames",
                        wrap_pyfunction!(_test_bundle_frames, module).unwrap(),
                    )
                    .unwrap();
                globals
                    .set_item(
                        "__file__",
                        concat!(
                            env!("CARGO_MANIFEST_DIR"),
                            "/tests/compact_bundle_export.py"
                        ),
                    )
                    .unwrap();
                let source =
                    CString::new(include_str!("../../tests/compact_bundle_export.py")).unwrap();
                py.run(source.as_c_str(), Some(globals), Some(globals))
                    .inspect_err(|error| error.print(py))
                    .expect("render-only FMTL history independence, replay and atomic publication");
            },
        );
    }

    #[test]
    fn camera_python_scene_bundle_export() {
        crate::with_python_test_module(
            "Camera Python scene bundle export",
            |py, module, globals| {
                globals
                    .set_item(
                        "_test_camera_bundle_frames",
                        wrap_pyfunction!(_test_camera_bundle_frames, module).unwrap(),
                    )
                    .unwrap();
                globals
                    .set_item(
                        "_test_png_pixels",
                        wrap_pyfunction!(_test_png_pixels, module).unwrap(),
                    )
                    .unwrap();
                globals
                    .set_item(
                        "__file__",
                        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/camera_bundle_export.py"),
                    )
                    .unwrap();
                let source =
                    CString::new(include_str!("../../tests/camera_bundle_export.py")).unwrap();
                py.run(source.as_c_str(), Some(globals), Some(globals))
                    .inspect_err(|error| error.print(py))
                    .expect("camera-aware export, real replay, pixels and no-clobber publication");
            },
        );
    }
}
