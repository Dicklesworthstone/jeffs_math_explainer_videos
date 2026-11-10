//! Attach Reel output to an existing Scene without replacing its arena or clock.
//!
//! Full renders configure a pristine Scene; insert recordings instead inherit
//! the live rational clock and RNG. Only publication state is new. The capture
//! sink, ordered emitter, codecs, audio mixer and ffmpeg boundary are shared.

use super::*;

pub(crate) fn install(module: &Bound<'_, PyModule>) -> PyResult<()> {
    portal_studio::bundle::install(module)?;
    // This runs after the schema bootstrap. Do not recreate PyO3's __all__:
    // these are private host boundaries, not a new wildcard import surface.
    module.setattr(
        "_portal_allocate_tracker",
        wrap_pyfunction!(portal_playback::_portal_allocate_tracker, module)?,
    )?;
    module.setattr(
        "_portal_begin_recording",
        wrap_pyfunction!(_portal_begin_recording, module)?,
    )?;
    module.setattr(
        "_portal_begin_audio_clip",
        wrap_pyfunction!(_portal_begin_audio_clip, module)?,
    )?;
    module.setattr(
        "_portal_scene_clock",
        wrap_pyfunction!(_portal_scene_clock, module)?,
    )?;
    module.setattr(
        "_portal_prepare_recording_scene",
        wrap_pyfunction!(_portal_prepare_recording_scene, module)?,
    )?;
    Ok(())
}

/// Configure a pristine Scene without acquiring or discarding an output sink.
/// Live recording must never use this boundary: only a fresh full-scene run
/// may replace its empty arena to select the native rational clock and seed.
#[pyfunction]
fn _portal_prepare_recording_scene(
    scene: &Bound<'_, PyScene>,
    width: u32,
    height: u32,
    fps: u32,
    seed: u64,
) -> PyResult<()> {
    if width == 0 || height == 0 || fps == 0 || u64::from(width) * u64::from(height) > 16_777_216 {
        return Err(PyValueError::new_err(
            "fresh recording requires positive dimensions and FPS, and at most 16777216 pixels",
        ));
    }
    check_available(scene)?;
    portal_video::check_scene_ownership(scene)?;
    let namespace = scene.getattr("__dict__")?;
    let namespace = namespace.cast::<PyDict>()?;
    for key in ["_fmn_owned_render_session", "_fmn_subdivision_session"] {
        if namespace
            .get_item(key)?
            .is_some_and(|owner| !owner.is_none())
        {
            return Err(PyRuntimeError::new_err(
                "this Scene already has an output owner",
            ));
        }
    }
    let mut config = fmn_config::Config::resolve(&[], None)
        .map_err(native_error)?
        .config;
    config.camera.resolution = (width, height);
    config.camera.fps = fps;
    config.determinism.mode = fmn_config::config::DeterminismMode::Standard;
    let replacement =
        Scene::new(RuntimeConfig::from_config(&config), seed).map_err(native_error)?;
    // No Python descriptors or callbacks run while the new engine is installed.
    // The same ownership check as ordinary rendering protects every live proxy.
    let mut owner = scene.try_borrow_mut()?;
    owner.engine = Rc::new(EngineState::new(replacement));
    owner.render_invocations.clear();
    owner.render_audio_inputs.clear();
    Ok(())
}

pub(crate) fn check_available(scene: &Bound<'_, PyScene>) -> PyResult<()> {
    let namespace = scene.getattr("__dict__")?;
    let namespace = namespace.cast::<PyDict>()?;
    if namespace
        .get_item("_fmn_scene_execution")?
        .is_some_and(|owner| !owner.is_none())
    {
        return Err(PyRuntimeError::new_err(
            "recording must start between play/wait calls, not inside an animation or updater",
        ));
    }
    let scene = scene.try_borrow()?;
    if scene
        .render
        .lock()
        .map_err(|_| PyRuntimeError::new_err("portal render session lock was poisoned"))?
        .is_some()
    {
        return Err(PyRuntimeError::new_err(
            "a portal render generation is already active",
        ));
    }
    Ok(())
}

#[pyfunction]
fn _portal_scene_clock(scene: &Bound<'_, PyScene>) -> PyResult<(u32, i64)> {
    let scene = scene.try_borrow()?;
    let engine = scene.engine.borrow();
    Ok((engine.fps(), engine.time().frames()))
}

fn recording_format(format: &str) -> PyResult<PortalOutputFormat> {
    let frame_format = match format {
        "png_sequence" => PortalFrameFormat::PngSequence,
        "gif" => PortalFrameFormat::Gif,
        "y4m" => PortalFrameFormat::Y4m,
        "mp4" => PortalFrameFormat::Mp4,
        "mov" => PortalFrameFormat::Mov,
        "wav" => return Ok(PortalOutputFormat::Wav),
        _ => {
            return Err(CapabilityError::new_err(
                "live recording requires png_sequence, gif, y4m, wav, mp4, or mov; use Camera.capture for a still",
            ));
        }
    };
    Ok(PortalOutputFormat::Frames(frame_format))
}

#[pyfunction]
#[allow(clippy::too_many_arguments)]
fn _portal_begin_recording(
    scene: &Bound<'_, PyScene>,
    destination: String,
    format: &str,
    width: u32,
    height: u32,
    fps: u32,
    threads: usize,
) -> PyResult<i64> {
    begin_recording(
        scene,
        destination,
        format,
        width,
        height,
        fps,
        threads,
        false,
    )
}

#[pyfunction]
#[allow(clippy::too_many_arguments)]
fn _portal_begin_audio_clip(
    scene: &Bound<'_, PyScene>,
    destination: String,
    format: &str,
    width: u32,
    height: u32,
    fps: u32,
    threads: usize,
) -> PyResult<i64> {
    if !matches!(format, "wav" | "mp4" | "mov") {
        return Err(PyValueError::new_err(
            "audio clips require wav, mp4, or mov",
        ));
    }
    begin_recording(
        scene,
        destination,
        format,
        width,
        height,
        fps,
        threads,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
fn begin_recording(
    scene: &Bound<'_, PyScene>,
    destination: String,
    format: &str,
    width: u32,
    height: u32,
    fps: u32,
    threads: usize,
    scene_audio: bool,
) -> PyResult<i64> {
    if destination.is_empty() || destination.contains('\0') {
        return Err(PyValueError::new_err(
            "recording destination must be nonempty and contain no NUL",
        ));
    }
    if width == 0
        || height == 0
        || u64::from(width) * u64::from(height) > 16_777_216
        || !(1..=96).contains(&threads)
    {
        return Err(PyValueError::new_err(
            "recording requires positive dimensions, at most 16777216 pixels, and 1..96 threads",
        ));
    }
    let destination = std::path::absolute(destination).map_err(native_error)?;
    let format = recording_format(format)?;
    check_available(scene)?;
    // Refuse known collisions before invoking descriptors or opening encoders.
    // This is only an early diagnostic: Reel's final create-only publication
    // still arbitrates files created while the recording is in progress.
    if fmn_platform::fs::FileSystem::node_kind_no_follow(&fmn_platform::fs::StdFs, &destination)
        .map_err(native_error)?
        .is_some()
    {
        return Err(pyo3::exceptions::PyFileExistsError::new_err(format!(
            "recording destination already exists: {}",
            destination.display()
        )));
    }
    if fps != scene.borrow().engine.borrow().fps() {
        return Err(PyValueError::new_err(
            "recording FPS must equal the live Scene clock; it cannot resample or reset an existing scene",
        ));
    }
    let video = match &format {
        PortalOutputFormat::Frames(frame @ (PortalFrameFormat::Mp4 | PortalFrameFormat::Mov)) => {
            Some(portal_video::PortalVideoConfig::from_recording_scene(
                scene, *frame, width, height, fps,
            )?)
        }
        _ => None,
    };
    let mut audio = portal_audio::PortalAudio::from_scene(scene, &format, video.as_ref())?;
    // Authored descriptors above may have run arbitrary Python. Recheck before
    // opening output; never steal a recursively acquired generation.
    check_available(scene)?;
    let engine = Rc::clone(&scene.borrow().engine);
    let (start_frame, first_cue) = {
        let live = engine.borrow();
        if live.fps() != fps {
            return Err(PyValueError::new_err(
                "the live Scene clock changed during recording preflight",
            ));
        }
        (live.time().frames(), live.sound_requests().len())
    };
    if start_frame < 0 {
        return Err(PyValueError::new_err(
            "recording cannot start at a negative scene frame",
        ));
    }
    if scene_audio {
        audio.use_scene_window();
    } else {
        audio.begin_at_request(first_cue);
    }
    let (mut session, _unused_runtime_config) = PortalRenderSession::new(
        destination,
        width,
        height,
        fps,
        threads,
        format,
        video,
        audio,
        false,
    )?;
    // Crucially, do NOT build or install a replacement Scene/EngineState here.
    // Every existing mobject, live NumPy view, updater and RNG keeps its owner.
    session.output_timeline_mut().start_at(start_frame)?;
    let slot = Arc::clone(&scene.borrow().render);
    let mut output = match slot.lock() {
        Ok(output) => output,
        Err(_) => {
            session.abort();
            return Err(PyRuntimeError::new_err(
                "portal render session lock was poisoned",
            ));
        }
    };
    if output.is_some() {
        session.abort();
        return Err(PyRuntimeError::new_err(
            "a portal render generation is already active",
        ));
    }
    {
        let mut owner = scene.borrow_mut();
        owner.render_invocations.clear();
        owner.render_audio_inputs.clear();
    }
    *output = Some(session);
    Ok(start_frame)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_does_not_admit_endpoint_only_or_unknown_formats() {
        for format in ["png", "svg", "jpeg", "", "PNG_SEQUENCE"] {
            assert!(recording_format(format).is_err(), "{format}");
        }
        for format in ["png_sequence", "gif", "y4m", "wav", "mp4", "mov"] {
            assert!(recording_format(format).is_ok(), "{format}");
        }
    }

    #[test]
    fn native_live_recording_acceptance() {
        crate::with_python_test_module("live recording", |py, _module, globals| {
            globals
                .set_item(
                    "__file__",
                    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/live_recording.py"),
                )
                .unwrap();
            let source = CString::new(include_str!("../tests/live_recording.py")).unwrap();
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("real native live recording and audio rebasing");
        });
    }

    #[test]
    fn native_output_ownership_acceptance() {
        crate::with_python_test_module("output publication ownership", |py, _module, globals| {
            globals
                .set_item(
                    "__file__",
                    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/output_no_clobber.py"),
                )
                .unwrap();
            let source = CString::new(include_str!("../tests/output_no_clobber.py")).unwrap();
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect(
                    "real native publication races, retries, symlinks and cross-filesystem video",
                );
        });
    }
}
