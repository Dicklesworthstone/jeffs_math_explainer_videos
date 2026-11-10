//! Host-CPython Studio composition. No Python dependency enters the native CLI.
//!
//! Capture uses the normal stepped lifecycle and the shared Lumen renderer.
//! The worker discards live Python/Stage ownership before serving a bounded,
//! read-only native timeline; source reload is a new process, never replay of
//! a purportedly serializable Python closure.

use super::*;
use fmn_studio::{
    FrameEncoding, FramePayload, FrameStream, InspectorLimits, InspectorSnapshot, InspectorView,
    ProtocolDigest, ProtocolLimits, SpanRegistry, WorkerService, protocol_digest,
};
use std::time::Duration;

pub(super) mod bundle;
pub(super) mod live;

pub(super) struct Capture {
    bundle: Option<bundle::BundleCapture>,
    renderer: RetainedFrameRenderer,
    camera: Camera,
    rgba: FrameBuffer,
    light: Option<Mob>,
    scene: String,
    recorded: fmn_studio::recorded::RecordedTimeline,
    failure: Option<String>,
    live: bool,
    live_refresh: bool,
    pub(super) timeline: OutputTimeline,
}

impl Capture {
    #[allow(clippy::too_many_arguments)]
    fn new(
        scene: String,
        build: ProtocolDigest,
        source: ProtocolDigest,
        width: u32,
        height: u32,
        fps: u32,
        threads: usize,
        max_frames: usize,
        max_bytes: usize,
    ) -> PyResult<(Self, RuntimeConfig)> {
        if width == 0
            || height == 0
            || u64::from(width) * u64::from(height) > 16_777_216
            || !(1..=240).contains(&fps)
            || !(1..=96).contains(&threads)
        {
            return Err(PyValueError::new_err(
                "Studio requires at most 16M pixels, 1..240 FPS and 1..96 threads",
            ));
        }
        let recorded = fmn_studio::recorded::RecordedTimeline::new(
            scene.clone(),
            build,
            source,
            max_frames,
            max_bytes,
        )
        .map_err(native_error)?;
        let mut config = fmn_config::Config::resolve(&[], None)
            .map_err(native_error)?
            .config;
        config.camera.resolution = (width, height);
        config.camera.fps = fps;
        config.determinism.mode = fmn_config::config::DeterminismMode::Standard;
        let request = fmn_runtime::PlanRequest::standard(
            fmn_runtime::RenderIntent::Offline,
            fmn_runtime::SurfaceSpec::lumen(width, height),
            fmn_runtime::OutputPixelFormat::Rgba8,
        )
        .with_max_cpu_threads(threads);
        let plan = fmn_runtime::ExecutionPlan::derive(
            request,
            &fmn_platform::topology::HardwareTopology::current(),
            None,
        )
        .map_err(native_error)?;
        let engine = match plan.engine {
            // render_with_camera executes ThreeDJob's CPU route, not the
            // affine FastCpu FrameJob. Python/source execution is uncertified.
            fmn_runtime::ExecutionEngine::FastCpu | fmn_runtime::ExecutionEngine::CertifiedCpu => {
                EngineIdentity::certified()
            }
            _ => {
                return Err(CapabilityError::new_err(
                    "Python Studio capture requires a CPU execution plan",
                ));
            }
        };
        // The configured background (default #333333), as standalone fmn and
        // the planar FMTL player use: a planar bundle has no background track,
        // so this is the baseline the scene's camera must match on export.
        let background = fmn_core::color::Srgb::from_hex(&config.camera.background_color)
            .map_err(|error| PyValueError::new_err(error.to_string()))?
            .to_linear(config.camera.background_opacity);
        let camera = Camera::new(CameraConfig {
            resolution: (width, height),
            fps,
            background,
            ..CameraConfig::default()
        })
        .map_err(camera_error)?;
        let frame = FrameConfig::new(
            Viewport { width, height },
            ScreenMap {
                scale: f64::from(height) / config.sizes.frame_height,
                origin: [f64::from(width) / 2.0, f64::from(height) / 2.0],
                y_up: true,
            },
            camera.background(),
        )
        .with_aa_policy(config.render.aa);
        let renderer = RetainedFrameRenderer::new(RetainedFrameRendererConfig {
            frame,
            engine,
            tiling: Tiling {
                macro_tile: plan.macro_tile,
                fine_tile: plan.fine_tile,
            },
            threads: plan
                .render_teams
                .first()
                .map_or(1, fmn_runtime::TeamPlan::threads),
        })
        .map_err(native_error)?;
        let rgba = FrameBuffer::new(
            FrameLayout::tight(PixelFormat::Rgba8, width, height).map_err(native_error)?,
        );
        Ok((
            Self {
                bundle: None,
                renderer,
                camera,
                rgba,
                light: None,
                scene,
                recorded,
                failure: None,
                live: false,
                live_refresh: false,
                timeline: OutputTimeline::default(),
            },
            RuntimeConfig::from_config(&config),
        ))
    }

    pub(super) fn is_empty(&self) -> bool {
        self.bundle.as_ref().map_or_else(
            || self.recorded.frame_count() == 0,
            bundle::BundleCapture::is_empty,
        )
    }

    pub(super) fn bind_camera(
        &mut self,
        frame: fmn_scene::studio_bridge::CameraFrame,
        light: [f64; 3],
        background: Option<fmn_core::color::LinearRgba>,
        light_mob: Option<Mob>,
    ) -> PyResult<()> {
        *self.camera.frame_mut() = frame;
        self.camera
            .set_light_source_position(light)
            .map_err(camera_error)?;
        if let Some(background) = background {
            self.camera
                .set_background(background)
                .map_err(camera_error)?;
        }
        self.light = light_mob;
        if let Some(bundle) = &mut self.bundle {
            bundle.validate_camera(&self.camera)?;
        }
        Ok(())
    }

    pub(super) fn capture(
        &mut self,
        packet: fmn_scene::studio_bridge::FramePacket,
    ) -> Result<(), fmn_scene::IntegrationError> {
        // Live input is a paused edit boundary. play/wait still execute their
        // ordinary samples and updaters, but only the completed callback's
        // final state replaces the live view. Intermediate/failed callback
        // captures cannot publish a partial edit or grow recorded history.
        if self.bundle.is_some() && self.live {
            return Err(fmn_scene::IntegrationError::new(
                "fmtl-recording",
                "live Studio input cannot share a bundle export",
            ));
        }
        if self.live && !self.live_refresh {
            return Ok(());
        }
        if let Some(error) = &self.failure {
            return Err(fmn_scene::IntegrationError::new(
                "python-studio",
                error.clone(),
            ));
        }
        if let Err(error) = self.capture_inner(packet) {
            let message = error.to_string();
            self.failure = Some(message.clone());
            return Err(fmn_scene::IntegrationError::new("python-studio", message));
        }
        Ok(())
    }

    fn capture_inner(&mut self, packet: fmn_scene::studio_bridge::FramePacket) -> PyResult<()> {
        let stage = packet.materialize_stage();
        if let Some(light) = self.light {
            if stage.get(light).is_none() {
                return Err(PyRuntimeError::new_err("Studio camera light is stale"));
            }
            self.camera
                .set_light_source_position(stage.get_center(light))
                .map_err(camera_error)?;
        }
        if let Some(bundle) = &mut self.bundle {
            return bundle.capture(packet, &stage, &self.camera);
        }
        self.renderer
            .render_with_camera(&stage, &self.camera)
            .map_err(native_error)?;
        portal_video::convert_frame(self.renderer.frame(), &mut self.rgba, None)
            .map_err(native_error)?;
        let (width, height) = self.camera.pixel_shape();
        let bytes = fmn_codec::encode_rgba8(
            width,
            height,
            self.rgba.plane(0),
            fmn_codec::CompressionLevel::Fast,
        );
        let config = self.renderer.config();
        let backend = fmn_studio::native::camera_capture_backend(&self.camera, config)
            .map_err(native_error)?;
        let stream = FrameStream {
            scene: self.scene.clone(),
            frame_index: 0,
            width,
            height,
            stride: 0,
            encoding: FrameEncoding::Png,
            payload: FramePayload::Pipe {
                digest: protocol_digest(&bytes),
                bytes,
            },
            render_backends: vec![backend],
        };
        let mut snapshot =
            InspectorSnapshot::capture(&stage, &SpanRegistry::new(), InspectorLimits::default())
                .map_err(native_error)?;
        // Geometry is world-space inspection. This read-only view deliberately
        // advertises no events or overlay-based picking for projected cameras.
        snapshot.view = Some(
            InspectorView::new(
                0,
                1,
                self.camera.fps(),
                Viewport { width, height },
                ScreenMap {
                    scale: if self.live {
                        f64::from(height) / fmn_core::constants::FRAME_HEIGHT
                    } else {
                        1.0 / self.camera.pixel_size()
                    },
                    origin: [f64::from(width) / 2.0, f64::from(height) / 2.0],
                    y_up: true,
                },
                false,
            )
            .map_err(native_error)?,
        );
        if self.live {
            self.recorded
                .replace_live_frame(stream, snapshot)
                .map_err(native_error)
        } else {
            self.recorded.push(stream, snapshot).map_err(native_error)
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn begin(
    slf: &Bound<'_, PyScene>,
    name: String,
    build: &str,
    source: &str,
    width: u32,
    height: u32,
    fps: u32,
    threads: usize,
    seed: u64,
    max_frames: usize,
    max_bytes: usize,
) -> PyResult<()> {
    portal_video::check_scene_ownership(slf)?;
    let (capture, runtime) = Capture::new(
        name,
        ProtocolDigest::from_hex(build).map_err(native_error)?,
        ProtocolDigest::from_hex(source).map_err(native_error)?,
        width,
        height,
        fps,
        threads,
        max_frames,
        max_bytes,
    )?;
    let replacement = Scene::new(runtime, seed).map_err(native_error)?;
    let slot = Arc::clone(&slf.borrow().render);
    let mut slot = slot
        .lock()
        .map_err(|_| PyRuntimeError::new_err("Studio generation lock poisoned"))?;
    if slot.is_some() {
        return Err(PyRuntimeError::new_err(
            "a render generation is already active",
        ));
    }
    let mut scene = slf.try_borrow_mut()?;
    scene.engine = Rc::new(EngineState::new(replacement));
    scene.render_invocations.clear();
    scene.render_audio_inputs.clear();
    *slot = Some(PortalRenderSession::Preview(Box::new(capture)));
    Ok(())
}

pub(super) fn finish(slf: &Bound<'_, PyScene>) -> PyResult<Recording> {
    let original_engine = Rc::clone(&slf.borrow().engine);
    let render = Arc::clone(&slf.borrow().render);
    {
        let slot = render
            .lock()
            .map_err(|_| PyRuntimeError::new_err("Studio generation lock poisoned"))?;
        if !matches!(slot.as_ref(), Some(PortalRenderSession::Preview(c)) if c.bundle.is_none()) {
            return Err(PyRuntimeError::new_err("no Studio capture is active"));
        }
    }
    portal_playback::synchronize(slf)?;
    synchronize_portal_camera(slf)?;
    let empty = render
        .lock()
        .map_err(|_| PyRuntimeError::new_err("Studio generation lock poisoned"))?
        .as_ref()
        .is_some_and(PortalRenderSession::needs_final_capture);
    if empty {
        let engine = Rc::clone(&slf.borrow().engine);
        let mut sink = PortalSceneSink {
            render: Arc::clone(&render),
            ..PortalSceneSink::default()
        };
        engine.borrow_mut().show(&mut sink).map_err(native_error)?;
    }
    let mut slot = render
        .lock()
        .map_err(|_| PyRuntimeError::new_err("Studio generation lock poisoned"))?;
    // Camera descriptors can execute Python. A second owner check prevents
    // their re-entrant cancellation/replacement from consuming another session.
    if !matches!(slot.as_ref(), Some(PortalRenderSession::Preview(_)))
        || !Rc::ptr_eq(&original_engine, &slf.borrow().engine)
    {
        return Err(PyRuntimeError::new_err(
            "Studio capture ownership changed during finish",
        ));
    }
    let Some(PortalRenderSession::Preview(capture)) = slot.take() else {
        unreachable!()
    };
    if let Some(error) = capture.failure {
        return Err(PyRuntimeError::new_err(error));
    }
    Ok(Recording {
        timeline: Some(capture.recorded),
    })
}

/// Only owned PNGs, records and digests cross from authoring to serving.
#[pyclass(name = "_StudioRecording", module = "manimlib")]
pub(super) struct Recording {
    timeline: Option<fmn_studio::recorded::RecordedTimeline>,
}

#[pymethods]
impl Recording {
    #[getter]
    fn frame_count(&self) -> PyResult<usize> {
        Ok(self
            .timeline
            .as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("recording already served"))?
            .frame_count())
    }
    fn serve(&mut self, py: Python<'_>) -> PyResult<()> {
        let mut timeline = self
            .timeline
            .take()
            .ok_or_else(|| PyRuntimeError::new_err("recording already served"))?;
        py.detach(move || {
            fmn_studio::serve_worker(
                &mut timeline,
                &mut std::io::stdin().lock(),
                &mut std::io::stdout().lock(),
                ProtocolLimits::default(),
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
        })
        .map_err(PyRuntimeError::new_err)
    }
    // The same native worker methods support in-process consumers and tests;
    // no second inspector JSON writer or frame decoder lives in the portal.
    fn inspect(&mut self, frame: i64) -> PyResult<String> {
        let timeline = self
            .timeline
            .as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("recording already served"))?;
        let scene = timeline.active_scene().unwrap_or_default().to_owned();
        timeline
            .handle(fmn_studio::SupervisorRequest::Scrub {
                scene: scene.clone(),
                frame,
            })
            .map_err(native_error)?;
        let fmn_studio::WorkerResponse::StudioData { bytes, .. } = timeline
            .handle(fmn_studio::SupervisorRequest::Inspect { scene })
            .map_err(native_error)?
        else {
            return Err(PyRuntimeError::new_err(
                "invalid recorded inspector response",
            ));
        };
        String::from_utf8(bytes).map_err(native_error)
    }
}

struct PythonBuilder {
    callback: Py<PyAny>,
}
impl fmn_studio::RebuildDriver for PythonBuilder {
    fn rebuild(&mut self) -> Result<fmn_studio::WorkerArtifact, fmn_studio::BuildError> {
        Python::attach(|py| {
            let result = self
                .callback
                .call0(py)
                .map_err(|e| fmn_studio::BuildError::new(e.to_string()))?;
            type Launch = (String, Vec<String>, Vec<(String, String)>, String, String);
            let (executable, argv, env, cwd, build) = result
                .extract::<Launch>(py)
                .map_err(|e| fmn_studio::BuildError::new(e.to_string()))?;
            let build_id = ProtocolDigest::from_hex(&build)
                .map_err(|e| fmn_studio::BuildError::new(e.to_string()))?;
            Ok(fmn_studio::WorkerArtifact {
                executable: executable.into(),
                argv,
                env,
                cwd: Some(cwd.into()),
                build_id,
            })
        })
    }
}

struct RunningHost {
    shutdown: Arc<AtomicBool>,
    frames: fmn_studio::FrameHub,
    thread: Option<std::thread::JoinHandle<Result<(), String>>>,
}

/// The existing bounded, content-based native watcher, owned by one portal
/// polling thread. No Python import or authored callback runs while scanning.
#[pyclass(name = "_StudioSourceWatch", module = "manimlib")]
pub(super) struct SourceWatcher {
    watch: fmn_studio::project_watch::SourceWatch,
    started: std::time::Instant,
}

#[pymethods]
impl SourceWatcher {
    #[getter]
    fn fingerprint(&self) -> String {
        self.watch.content_fingerprint().to_hex()
    }

    #[getter]
    fn files(&self) -> PyResult<Vec<(String, String)>> {
        self.watch
            .source_files()
            .map(|(path, digest)| {
                let path = path.to_str().ok_or_else(|| {
                    PyValueError::new_err("Studio source identity requires UTF-8 paths")
                })?;
                Ok((path.to_owned(), digest.to_hex()))
            })
            .collect()
    }

    fn poll(&mut self, py: Python<'_>) -> PyResult<bool> {
        py.detach(|| {
            self.watch
                .poll(self.started.elapsed())
                .map_err(|e| e.to_string())
        })
        .map_err(PyRuntimeError::new_err)
    }
}
impl Drop for RunningHost {
    fn drop(&mut self) {
        // Drop must not wait for a client which needs the GIL to build a reload.
        // The serving thread owns/reaps the disposable worker when it exits.
        self.shutdown.store(true, Ordering::Release);
        self.frames.close();
    }
}

#[pyclass(name = "_StudioHost", module = "manimlib")]
pub(super) struct Host {
    url: String,
    running: Option<RunningHost>,
}

#[pymethods]
impl Host {
    #[new]
    fn new(
        py: Python<'_>,
        builder: Py<PyAny>,
        scene: String,
        token: &str,
        port: u16,
        timeout: u64,
    ) -> PyResult<Self> {
        if !(1..=900).contains(&timeout) {
            return Err(PyValueError::new_err(
                "Studio worker timeout must be 1..900 seconds",
            ));
        }
        if !builder.bind(py).is_callable() {
            return Err(PyTypeError::new_err("Studio builder must be callable"));
        }
        let token = fmn_studio::CapabilityToken::from_hex(token).map_err(native_error)?;
        let config = fmn_studio::StudioHostConfig {
            bind_addr: std::net::SocketAddr::from(([127, 0, 0, 1], port)),
            // FrameHub::close wakes idle streams on shutdown. Retain the
            // native idle interval so a paused preview does not reconnect
            // continuously while the author is inspecting it.
            ..fmn_studio::StudioHostConfig::default()
        };
        let (host, session, frames) = py
            .detach(move || {
                fmn_studio::recorded::start_host(
                    &scene,
                    Box::new(PythonBuilder { callback: builder }),
                    token,
                    config,
                    Duration::from_secs(timeout),
                )
                .map_err(|e| e.to_string())
            })
            .map_err(PyRuntimeError::new_err)?;
        let url = host.launch_url().map_err(native_error)?;
        let shutdown = Arc::new(AtomicBool::new(false));
        let signal = Arc::clone(&shutdown);
        let retained_frames = frames.clone();
        let thread = std::thread::Builder::new()
            .name("fmn-python-studio".into())
            .spawn(move || {
                let result = host.serve_until(&signal).map_err(|e| e.to_string());
                session.shutdown_worker();
                retained_frames.close();
                result
            })
            .map_err(native_error)?;
        Ok(Self {
            url,
            running: Some(RunningHost {
                shutdown,
                frames,
                thread: Some(thread),
            }),
        })
    }
    #[getter]
    fn url(&self) -> &str {
        &self.url
    }
    #[getter]
    fn alive(&self) -> bool {
        self.running
            .as_ref()
            .and_then(|r| r.thread.as_ref())
            .is_some_and(|t| !t.is_finished())
    }

    /// Directory inputs watch Python sources only; explicitly named files can
    /// also watch assets. Generated output, pyc caches and virtualenv trees are
    /// excluded so rendering itself does not cause an automatic reload loop.
    #[staticmethod]
    fn watch_sources(
        py: Python<'_>,
        paths: Vec<String>,
        debounce_ms: u64,
    ) -> PyResult<SourceWatcher> {
        if debounce_ms > 10_000 || paths.is_empty() || paths.len() > 64 {
            return Err(PyValueError::new_err(
                "watch requires 1..64 paths and debounce <= 10000 ms",
            ));
        }
        let watch = py
            .detach(move || {
                use fmn_studio::project_watch::{SourceWatch, WatchFilter, WatchLimits};
                SourceWatch::new_filtered(
                    paths.into_iter().map(PathBuf::from).collect(),
                    Vec::new(),
                    Duration::from_millis(debounce_ms),
                    WatchLimits::default(),
                    WatchFilter {
                        extensions: vec!["py".into(), "pyw".into()],
                        ignored_directories: [
                            "__pycache__",
                            ".venv",
                            "venv",
                            ".tox",
                            ".mypy_cache",
                            ".pytest_cache",
                        ]
                        .into_iter()
                        .map(str::to_owned)
                        .collect(),
                    },
                )
                .map_err(|e| e.to_string())
            })
            .map_err(PyRuntimeError::new_err)?;
        Ok(SourceWatcher {
            watch,
            started: std::time::Instant::now(),
        })
    }

    fn close(&mut self, py: Python<'_>) -> PyResult<()> {
        if let Some(mut running) = self.running.take() {
            running.shutdown.store(true, Ordering::Release);
            running.frames.close();
            if let Some(thread) = running.thread.take() {
                py.detach(move || {
                    thread
                        .join()
                        .map_err(|_| "Studio host thread panicked".to_owned())?
                })
                .map_err(PyRuntimeError::new_err)?;
            }
        }
        Ok(())
    }
}

/// Real captured artifacts for the mandatory Gauntlet lifecycle row.
#[cfg(any(test, feature = "gauntlet"))]
pub struct PortalStudioGauntletReport {
    pub first_png: Vec<u8>,
    pub last_png: Vec<u8>,
    pub last_inspector: Vec<u8>,
    pub frames: u64,
}

/// Exercise the production capture and worker protocol without publishing an
/// offline render or claiming that Python callbacks are replayable.
#[cfg(any(test, feature = "gauntlet"))]
pub fn run_portal_gauntlet_studio_capture() -> Result<PortalStudioGauntletReport, String> {
    crate::with_python_test_module("Studio native capture", |py, _module, globals| {
        let source = std::ffi::CString::new(
            r#"
import json
import manimlib as m
s = m.Scene()
s._begin_studio_capture('Moving', '11' * 32, '22' * 32, 96, 54, 8, 1, 0, 16, 1024*1024)
s.camera._core.set_pixel_shape(96, 54)
s.camera.fps = 8
box = m.Square(fill_opacity=1)
s.add(box)
s.play(box.animate.shift(m.RIGHT), run_time=0.5)
s.wait(0.25)
r = s._finish_studio_capture()
assert r.frame_count == 6, r.frame_count
first, last = json.loads(r.inspect(0)), json.loads(r.inspect(5))
assert first['view']['frame_count'] == 6
assert first['view']['input_events'] is False
assert first['nodes'] and last['nodes']
assert first['scene_time'] < last['scene_time']
assert first['nodes'] != last['nodes']
assert json.loads(r.inspect(0)) == first, 'scrub changed captured state'
static = m.Scene()
static._begin_studio_capture('Still', '33' * 32, '44' * 32, 96, 54, 8, 1, 0, 16, 1024*1024)
static.camera._core.set_pixel_shape(96, 54)
static.camera.fps = 8
static.add(m.Circle())
assert static._finish_studio_capture().frame_count == 1
"#,
        )
        .unwrap();
        py.run(source.as_c_str(), Some(globals), Some(globals))
            .inspect_err(|e| e.print(py))
            .map_err(|e| e.to_string())?;
        let object = globals
            .get_item("r")
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Studio capture produced no recording".to_owned())?;
        let mut recording = object
            .extract::<PyRefMut<'_, Recording>>()
            .map_err(|e| e.to_string())?;
        let timeline = recording
            .timeline
            .as_mut()
            .ok_or_else(|| "Studio recording was already consumed".to_owned())?;
        let scene = timeline.active_scene().unwrap_or_default().to_owned();
        let frames = timeline.frame_count() as u64;
        let png = |timeline: &mut fmn_studio::recorded::RecordedTimeline, index| {
            let response = timeline
                .handle(fmn_studio::SupervisorRequest::Scrub {
                    scene: scene.clone(),
                    frame: index,
                })
                .map_err(|e| e.to_string())?;
            match response {
                fmn_studio::WorkerResponse::Frame(FrameStream {
                    payload: FramePayload::Pipe { bytes, .. },
                    ..
                }) => Ok(bytes),
                _ => Err("Studio did not return an owned PNG".to_owned()),
            }
        };
        let first_png = png(timeline, 0)?;
        let last_png = png(timeline, 5)?;
        let fmn_studio::WorkerResponse::StudioData {
            bytes: last_inspector,
            ..
        } = timeline
            .handle(fmn_studio::SupervisorRequest::Inspect {
                scene: scene.clone(),
            })
            .map_err(|e| e.to_string())?
        else {
            return Err("Studio did not return its native inspector".to_owned());
        };
        Ok(PortalStudioGauntletReport {
            first_png,
            last_png,
            last_inspector,
            frames,
        })
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn portal_studio_capture_uses_stepped_scene_and_native_inspector() {
        let report = super::run_portal_gauntlet_studio_capture().unwrap();
        let first =
            fmn_codec::decode_png(&report.first_png, &fmn_codec::PngLimits::default()).unwrap();
        let last =
            fmn_codec::decode_png(&report.last_png, &fmn_codec::PngLimits::default()).unwrap();
        assert_eq!(report.frames, 6);
        assert_eq!((first.width, first.height), (96, 54));
        assert_ne!(
            first.rgba, last.rgba,
            "animation must reach the captured native pixels"
        );
        assert!(!report.last_inspector.is_empty());
    }
}
