//! The production PyO3 `manimlib` bridge (§15.2, fm-aqv).
//!
//! The engine boundary is intentionally narrow. Ordinary Python semantics
//! (cooperative constructors, descriptors, mutable containers, copy/pickle,
//! and schema-generated modules) live in the embedded bootstrap. Rust owns
//! arena identity, RecordBuffer generations, lifecycle dispatch points, and
//! typed exception mapping.
//!
//! This is the sole authoritative crate allowed to contain unsafe code (D3).
//! The project-authored unsafe surface is confined to the two mechanical
//! CPython buffer slots on `PyRecordView`; every other operation is safe
//! Rust and no engine borrow crosses a Python callback.
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(test)]
mod corpus;
mod crossing;
mod ladder;
#[cfg(any(test, feature = "gauntlet"))]
mod live_tex;
mod method_cache;
pub mod perf_harness;
mod portal_assignment;
mod portal_audio;
#[cfg(any(test, feature = "gauntlet"))]
mod portal_console;
mod portal_network;
#[cfg(feature = "gauntlet")]
pub use portal_console::run_portal_gauntlet_console;
mod portal_playback;
mod portal_readback;
mod portal_recording;
mod portal_streamlines;
mod portal_studio;
mod portal_svg;
mod portal_texture;
mod portal_typography;
mod portal_video;
#[cfg(any(test, feature = "gauntlet"))]
mod structural_parity;
#[cfg(feature = "gauntlet")]
pub use portal_texture::{run_portal_gauntlet_surface_lifecycle, run_portal_gauntlet_textures};
#[cfg(feature = "gauntlet")]
pub use structural_parity::{
    CorpusRegressionCase, EnvelopeDrillReport, StructuralParityReport,
    run_portal_gauntlet_class_sweep, run_portal_gauntlet_envelope_drill,
    run_portal_gauntlet_reference_corpus_regressions, run_portal_gauntlet_structural_facts,
};
mod portal_provenance;
mod report;

#[cfg(feature = "gauntlet")]
pub use live_tex::{PortalLiveTexReport, run_portal_gauntlet_live_tex};

mod complex_readouts;
#[cfg(feature = "gauntlet")]
pub use complex_readouts::run_portal_gauntlet_complex_readouts;

mod tex_preamble;
#[cfg(feature = "gauntlet")]
pub use tex_preamble::run_portal_gauntlet_tex_preamble;

#[cfg(feature = "gauntlet")]
pub use portal_svg::{PortalSvgReport, run_portal_gauntlet_svg};

use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString, c_int, c_void};
use std::path::PathBuf;
use std::ptr;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crossing::CrossingClass;
use portal_playback::OutputTimeline;

use fmn_frame::{FrameBuffer, FrameLayout, PixelFormat};
use fmn_mobject::{
    JointType, Mob, Mobject, RecordBuffer, RecordError, RecordSchema, RecordView, Snapshot, Stage,
    StageError, Uniforms,
};
use fmn_output::{
    EmitterConfig, GifSink, GifSinkConfig, NativeArtifactReport, OrderedEmitter, PngSink,
    PngSinkConfig, PngTarget, SinkLimits, SinkReceipt, Y4mSink, Y4mSinkConfig,
};
use fmn_render::{
    Camera, CameraConfig, EngineIdentity, FrameConfig, RetainedFrameRenderer,
    RetainedFrameRendererConfig, ScreenMap, Tiling, Viewport,
};
use fmn_scene::{RuntimeConfig, Scene};
use pyo3::basic::CompareOp;
use pyo3::buffer::PyBuffer;
use pyo3::create_exception;
use pyo3::exceptions::{
    PyBufferError, PyException, PyImportError, PyKeyError, PyOSError, PyOverflowError,
    PyRuntimeError, PyTypeError, PyValueError,
};
use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyByteArray, PyBytes, PyDict, PyList, PyModule, PyTuple};

create_exception!(
    manimlib,
    StaleHandleError,
    PyRuntimeError,
    "The engine entry behind this proxy is stale, deleted, or unbound."
);
create_exception!(
    manimlib,
    ForeignStageError,
    PyRuntimeError,
    "A mobject cannot cross Scene arenas by reference; copy it instead."
);
create_exception!(
    manimlib,
    FamilyCycleError,
    PyValueError,
    "The requested submobject relation would make the family graph cyclic."
);
create_exception!(
    manimlib,
    CapabilityError,
    PyRuntimeError,
    "An optional host capability required by this operation is unavailable."
);
create_exception!(
    manimlib,
    TexError,
    PyValueError,
    "Native TeX parsing or span provenance failed."
);
create_exception!(
    manimlib,
    EndScene,
    PyException,
    "Normal early termination of a Scene."
);

type Engine = Rc<EngineState>;
type ProxyPairs = Vec<(Py<PyAny>, Py<PyAny>)>;
type SoundRequestFact = (String, i64, u32, f64, Option<f64>, Option<f64>);
type BoundingBoxRows = ([f64; 3], [f64; 3], [f64; 3]);
type PointRun = Vec<[f64; 3]>;
type AlignedPointRuns = (PointRun, PointRun);

const PORTAL_MAX_RENDER_FRAMES: u64 = 1_000_000;
const PORTAL_PICKLE_STATE_VERSION: u8 = 1;

#[derive(Clone, Copy, PartialEq, Eq)]
enum PortalFrameFormat {
    Png,
    PngSequence,
    Gif,
    Y4m,
    Mp4,
    Mov,
}

struct PortalArtifactReport {
    path: PathBuf,
    frame_count: u64,
    bytes: u64,
    digest: fmn_output::ArtifactDigest,
    invocations: Vec<fmn_output::InvocationReport>,
    audio_inputs: Vec<portal_audio::PortalAudioInput>,
    /// Frames rasterized on Lumen's retained planar route and on the camera
    /// route (fm-sq8.11).
    routes: PortalRoutes,
}

/// Which Lumen route rasterized a render's frames (fm-sq8.11): counts for
/// the retained planar route and the camera route, plus why the first
/// camera-route frame could not take the planar one. It also records what
/// the render used of its execution plan: teams used against teams planned,
/// threads per team, and the emitter's frames in flight.
#[derive(Clone, Copy, Default, Debug)]
struct PortalRoutes {
    planar: u64,
    camera: u64,
    first_camera_reason: Option<&'static str>,
    teams_used: u64,
    teams_planned: u64,
    team_threads: u64,
    frames_in_flight: u64,
}

impl From<NativeArtifactReport> for PortalArtifactReport {
    fn from(report: NativeArtifactReport) -> Self {
        Self {
            path: report.path,
            frame_count: report.frame_count,
            bytes: report.bytes,
            digest: report.digest,
            invocations: Vec::new(),
            audio_inputs: Vec::new(),
            routes: PortalRoutes::default(),
        }
    }
}

enum PortalReceipt {
    Native(SinkReceipt<NativeArtifactReport>),
    Video(SinkReceipt<fmn_output::FfmpegArtifactReport>),
}

enum PortalOutputFormat {
    Frames(PortalFrameFormat),
    Svg,
    Wav,
}

enum PortalRenderSession {
    Preview(Box<portal_studio::Capture>),
    Vector(Box<portal_svg::SvgSession>),
    Frames(Box<PortalFrameSession>),
    Soundtrack {
        destination: PathBuf,
        threads: usize,
        timeline: OutputTimeline,
        audio: Box<portal_audio::PortalAudio>,
    },
}

impl PortalRenderSession {
    #[allow(clippy::too_many_arguments)]
    fn new(
        destination: PathBuf,
        width: u32,
        height: u32,
        fps: u32,
        threads: usize,
        format: PortalOutputFormat,
        video: Option<portal_video::PortalVideoConfig>,
        audio: Box<portal_audio::PortalAudio>,
        reproducible: bool,
    ) -> PyResult<(Self, RuntimeConfig)> {
        match format {
            PortalOutputFormat::Svg => {
                let (session, runtime) =
                    portal_svg::SvgSession::new(destination, width, height, fps, threads)?;
                Ok((Self::Vector(Box::new(session)), runtime))
            }
            PortalOutputFormat::Frames(format) => {
                let (session, config) = PortalFrameSession::new(
                    destination,
                    width,
                    height,
                    fps,
                    threads,
                    format,
                    video,
                    audio,
                    reproducible,
                )?;
                Ok((Self::Frames(Box::new(session)), config))
            }
            PortalOutputFormat::Wav => {
                if width == 0 || height == 0 || fps == 0 || threads == 0 {
                    return Err(PyValueError::new_err(
                        "output resolution, fps, and thread limit must be nonzero",
                    ));
                }
                let mut config = fmn_config::Config::resolve(&[], None)
                    .map_err(native_error)?
                    .config;
                config.camera.resolution = (width, height);
                config.camera.fps = fps;
                config.determinism.mode = if reproducible {
                    fmn_config::config::DeterminismMode::Certified
                } else {
                    fmn_config::config::DeterminismMode::Standard
                };
                // Run the same sampled lifecycle so updaters can add cues at
                // their real call-site times. Only rasterization is absent.
                Ok((
                    Self::Soundtrack {
                        destination,
                        threads,
                        timeline: OutputTimeline::default(),
                        audio,
                    },
                    RuntimeConfig::from_config(&config),
                ))
            }
        }
    }

    fn capture(
        &mut self,
        packet: fmn_scene::studio_bridge::FramePacket,
    ) -> Result<(), fmn_scene::IntegrationError> {
        match self {
            Self::Frames(session) => session.capture(packet),
            Self::Preview(session) => session.capture(packet),
            Self::Soundtrack { .. } => Ok(()),
            Self::Vector(_) => Ok(()),
        }
    }

    fn bind_camera(
        &mut self,
        frame: fmn_scene::studio_bridge::CameraFrame,
        light_position: [f64; 3],
        background: Option<fmn_core::color::LinearRgba>,
        light_mob: Option<Mob>,
    ) -> PyResult<()> {
        match self {
            Self::Preview(session) => {
                session.bind_camera(frame, light_position, background, light_mob)
            }
            Self::Frames(session) => {
                session.bind_camera(frame, light_position, background, light_mob)
            }
            Self::Soundtrack { .. } => Ok(()),
            Self::Vector(session) => session.bind_camera(frame, background),
        }
    }

    fn needs_final_capture(&self) -> bool {
        matches!(self, Self::Frames(session) if session.frame_count() == 0)
            || matches!(self, Self::Preview(session) if session.is_empty())
            || matches!(self, Self::Vector(_))
    }

    fn abort(self) {
        if let Self::Frames(session) = self {
            session.abort();
        }
    }

    fn finish(self, scene: &Scene) -> PyResult<(PortalArtifactReport, String, usize)> {
        match self {
            Self::Preview(_) => Err(PyRuntimeError::new_err(
                "Studio capture must finish through _finish_studio_capture, not file publication",
            )),
            Self::Vector(session) => session.finish(scene),
            // ubs:ignore — finalizes frame publication; no security token or randomness is generated.
            Self::Frames(session) => session.finish(scene),
            Self::Soundtrack {
                destination,
                threads,
                timeline,
                mut audio,
            } => {
                let mix = audio.mix(scene, threads, &timeline)?.ok_or_else(|| {
                    PyRuntimeError::new_err("WAV output requires at least one Scene.add_sound cue")
                })?;
                let config = fmn_output::MixerConfig::default();
                // S16 stereo plus the WAV header, bounded by Reel's native
                // timeline budget. Reel performs atomic publication only
                // after every cue has decoded and the complete mix succeeds.
                let max_artifact_bytes = config
                    .max_output_frames
                    .checked_mul(u64::from(config.channels) * 2)
                    .and_then(|bytes| bytes.checked_add(1024))
                    .ok_or_else(|| PyOverflowError::new_err("WAV artifact budget overflow"))?;
                let report = fmn_output::publish_wav_new(
                    &fmn_platform::fs::StdFs,
                    &fmn_output::WavPublicationConfig {
                        destination,
                        format: fmn_codec::SampleFormat::S16,
                        dither: fmn_output::DitherPolicy::None,
                        max_artifact_bytes,
                        profile: None,
                    },
                    &mix,
                )
                .map_err(native_error)?;
                Ok((
                    PortalArtifactReport {
                        path: report.path,
                        frame_count: report.sample_frames,
                        bytes: report.bytes,
                        digest: report.digest,
                        invocations: audio.invocations(),
                        audio_inputs: std::mem::take(&mut audio.inputs),
                        routes: PortalRoutes::default(),
                    },
                    "native-sound-mixer".to_owned(),
                    threads,
                ))
            }
        }
    }
}

/// One fmn-python render generation, retained across every `play` and `wait`.
///
/// Lumen owns the renderer state; Reel owns bounded ordered publication. This
/// portal adapter owns only their lifetime and the Python-facing report.
struct PortalFrameSession {
    renderer: RetainedFrameRenderer,
    camera: Camera,
    emitter: Option<OrderedEmitter>,
    receipt: PortalReceipt,
    soundtrack: Option<fmn_output::FfmpegSoundtrack>,
    audio: Box<portal_audio::PortalAudio>,
    light_mob: Option<Mob>,
    rgba8_scratch: Option<FrameBuffer>,
    opaque_video: bool,
    next_sequence: u64,
    timeline: OutputTimeline,
    /// The default camera frame. While the scene's frame equals it and the
    /// output aspect is the frame's, planar vector content takes the retained
    /// 2D route (tile cache, adaptive AA) instead of the camera route.
    planar_baseline: fmn_scene::studio_bridge::CameraFrame,
    planar_map_matches: bool,
    routes: PortalRoutes,
}

impl PortalFrameSession {
    #[allow(clippy::too_many_arguments)]
    fn new(
        destination: PathBuf,
        width: u32,
        height: u32,
        fps: u32,
        max_threads: usize,
        format: PortalFrameFormat,
        video: Option<portal_video::PortalVideoConfig>,
        audio: Box<portal_audio::PortalAudio>,
        reproducible: bool,
    ) -> PyResult<(Self, RuntimeConfig)> {
        if width == 0 || height == 0 {
            return Err(PyValueError::new_err(
                "render resolution dimensions must be nonzero",
            ));
        }
        if fps == 0 {
            return Err(PyValueError::new_err("render fps must be nonzero"));
        }
        if max_threads == 0 {
            return Err(PyValueError::new_err("render thread limit must be nonzero"));
        }

        let mut config = fmn_config::Config::resolve(&[], None)
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?
            .config;
        config.camera.resolution = (width, height);
        config.camera.fps = fps;
        config.determinism.mode = if reproducible {
            fmn_config::config::DeterminismMode::Certified
        } else {
            fmn_config::config::DeterminismMode::Standard
        };

        let opaque_video = video
            .as_ref()
            .is_some_and(|video| video.job.container != fmn_output::Container::MovTransparent);

        // ubs:ignore — compares a public output-format enum, not a secret.
        let wire = if let Some(video) = &video {
            video.job.wire
        } else if format == PortalFrameFormat::Y4m {
            fmn_output::WireFormat::Nv12
        } else {
            fmn_output::WireFormat::Rgba8
        };
        let output_format = match wire {
            fmn_output::WireFormat::Rgba8 => fmn_runtime::OutputPixelFormat::Rgba8,
            fmn_output::WireFormat::Bgra8 => fmn_runtime::OutputPixelFormat::Bgra8,
            fmn_output::WireFormat::Nv12 => fmn_runtime::OutputPixelFormat::Nv12,
            fmn_output::WireFormat::P010 => fmn_runtime::OutputPixelFormat::P010,
        };
        let request = if reproducible {
            fmn_runtime::PlanRequest::certified(
                fmn_runtime::RenderIntent::Offline,
                fmn_runtime::SurfaceSpec::lumen(width, height),
                output_format,
            )
            .with_max_cpu_threads(max_threads)
        } else {
            fmn_runtime::PlanRequest::standard(
                fmn_runtime::RenderIntent::Offline,
                fmn_runtime::SurfaceSpec::lumen(width, height),
                output_format,
            )
            .with_max_cpu_threads(max_threads)
        };
        let plan = fmn_runtime::ExecutionPlan::derive(
            request,
            &fmn_platform::topology::HardwareTopology::current(),
            None,
        )
        .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        let engine = match plan.engine {
            fmn_runtime::ExecutionEngine::CertifiedCpu => EngineIdentity::certified(),
            fmn_runtime::ExecutionEngine::FastCpu => EngineIdentity::fast(),
            fmn_runtime::ExecutionEngine::Metal | fmn_runtime::ExecutionEngine::Cuda => {
                return Err(CapabilityError::new_err(
                    "fmn-python render currently accepts only CPU execution plans",
                ));
            }
        };
        let background = fmn_core::color::Srgb::from_hex(&config.camera.background_color)
            .map_err(|error| PyValueError::new_err(error.to_string()))?
            .to_linear(config.camera.background_opacity);
        let frame_config = FrameConfig::new(
            Viewport { width, height },
            ScreenMap {
                scale: f64::from(height) / config.sizes.frame_height,
                origin: [f64::from(width) / 2.0, f64::from(height) / 2.0],
                y_up: true,
            },
            background,
        )
        .with_aa_policy(config.render.aa);
        let renderer = RetainedFrameRenderer::new(RetainedFrameRendererConfig {
            frame: frame_config,
            tiling: Tiling {
                macro_tile: plan.macro_tile,
                fine_tile: plan.fine_tile,
            },
            engine,
            threads: plan
                .render_teams
                .first()
                .map_or(1, fmn_runtime::TeamPlan::threads),
        })
        .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        let camera = Camera::new(CameraConfig {
            resolution: (width, height),
            fps,
            background,
            ..CameraConfig::default()
        })
        .map_err(|error| PyValueError::new_err(error.to_string()))?;
        // The retained 2D route maps scene units by output height over the
        // configured frame height with the origin at the centre. It draws what
        // the camera route draws only while the scene frame is the default one
        // and that frame has the output's aspect.
        let planar_baseline = CameraConfig::default().frame;
        let [frame_width, frame_height] = planar_baseline.shape();
        let planar_map_matches = (frame_height - config.sizes.frame_height).abs() < 1e-12
            && (frame_width - frame_height * f64::from(width) / f64::from(height)).abs() < 1e-9;
        // ubs:ignore — compares a public output-format enum, not a secret.
        let pixel_format = wire.frame_format();
        let output_layout = FrameLayout::tight(pixel_format, width, height)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        // ubs:ignore — compares a public output-format enum, not a secret.
        let rgba8_scratch = if pixel_format != PixelFormat::Rgba8 {
            Some(FrameBuffer::new(
                FrameLayout::tight(PixelFormat::Rgba8, width, height)
                    .map_err(|error| PyValueError::new_err(error.to_string()))?,
            ))
        } else {
            None
        };
        let frame_bytes = u64::try_from(output_layout.total_bytes())
            .map_err(|_| PyOverflowError::new_err("render frame size exceeds u64"))?;
        // ubs:ignore — compares a public output-format enum, not a secret.
        let single_frame = format == PortalFrameFormat::Png;
        let max_frames = if single_frame {
            1
        } else {
            PORTAL_MAX_RENDER_FRAMES
        };
        let max_stream_bytes = frame_bytes
            .checked_mul(max_frames)
            .ok_or_else(|| PyOverflowError::new_err("render stream budget exceeds u64"))?;
        let max_resident_bytes = frame_bytes
            .checked_mul(3)
            .and_then(|bytes| bytes.checked_add(4 * 1024 * 1024))
            .ok_or_else(|| PyOverflowError::new_err("render resident budget exceeds u64"))?;
        let max_artifact_bytes = max_stream_bytes
            .checked_mul(2)
            .ok_or_else(|| PyOverflowError::new_err("render artifact budget exceeds u64"))?;
        let limits = SinkLimits::new(
            max_frames,
            max_resident_bytes,
            max_stream_bytes,
            max_artifact_bytes,
        )
        .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let fs: Arc<dyn fmn_platform::fs::FileSystem> = Arc::new(fmn_platform::fs::StdFs);
        let mut soundtrack = None;
        let (binding, receipt) = match format {
            PortalFrameFormat::Png | PortalFrameFormat::PngSequence => {
                let target = if single_frame {
                    PngTarget::Single(destination)
                } else {
                    PngTarget::Sequence {
                        directory: destination,
                        stem: "frame".to_owned(),
                        digits: 6,
                    }
                };
                let (binding, receipt) = PngSink::new(
                    fs,
                    PngSinkConfig {
                        target,
                        width,
                        height,
                        first_sequence: 0,
                        // Certified PNGs are the canonical (Best) encoding,
                        // byte-identical to native fmn's under --reproducible.
                        compression: if reproducible {
                            fmn_codec::CompressionLevel::Best
                        } else {
                            fmn_codec::CompressionLevel::Default
                        },
                        threads: plan.encoder_threads().max(1),
                        limits,
                        profile: None,
                    },
                )
                .map_err(|error| PyValueError::new_err(error.to_string()))?
                .with_no_clobber()
                .into_binding(if single_frame {
                    "python-png"
                } else {
                    "python-png-sequence"
                });
                (binding, PortalReceipt::Native(receipt))
            }
            PortalFrameFormat::Gif => {
                let (binding, receipt) = GifSink::new(
                    fs,
                    GifSinkConfig {
                        destination,
                        width,
                        height,
                        fps: (fps, 1),
                        loop_forever: true,
                        first_sequence: 0,
                        limits,
                        profile: None,
                    },
                )
                .map_err(|error| PyValueError::new_err(error.to_string()))?
                .with_no_clobber()
                .into_binding("python-gif");
                (binding, PortalReceipt::Native(receipt))
            }
            PortalFrameFormat::Y4m => {
                let (binding, receipt) = Y4mSink::new(
                    fs,
                    Y4mSinkConfig {
                        destination,
                        width,
                        height,
                        fps: (fps, 1),
                        colorspace: fmn_codec::Y4mColorspace::C420Mpeg2,
                        first_sequence: 0,
                        limits,
                        profile: None,
                    },
                )
                .map_err(|error| PyValueError::new_err(error.to_string()))?
                .with_no_clobber()
                .into_binding("python-y4m");
                (binding, PortalReceipt::Native(receipt))
            }
            PortalFrameFormat::Mp4 | PortalFrameFormat::Mov => {
                use fmn_platform::process::{
                    FfmpegLocator as _, StdFfmpegLocator, StdProcessRunner,
                };
                let video =
                    video.ok_or_else(|| PyValueError::new_err("missing video negotiation"))?;
                let encoder = video
                    .job
                    .resolved_encoder()
                    .map_err(native_error)?
                    .ok_or_else(|| PyValueError::new_err("video requires an encoder"))?;
                let runner: Arc<dyn fmn_platform::process::ProcessRunner> =
                    Arc::new(StdProcessRunner);
                let executable = StdFfmpegLocator::from_host_path()
                    .locate_ffmpeg(std::path::Path::new(&video.ffmpeg_bin))
                    .map_err(|error| {
                        CapabilityError::new_err(format!(
                            "ffmpeg is unavailable: {error}; {}",
                            fmn_output::NATIVE_ALTERNATIVE
                        ))
                    })?;
                let workdir_root = std::env::temp_dir();
                let tool =
                    fmn_output::FfmpegTool::resolve(executable, runner.as_ref(), &workdir_root)
                        .map_err(|error| {
                            CapabilityError::new_err(format!("ffmpeg boundary: {error}"))
                        })?;
                let capabilities = fmn_output::EncoderCapabilities::probe(&tool, runner.as_ref())
                    .map_err(|error| {
                    CapabilityError::new_err(format!("ffmpeg encoders: {error}"))
                })?;
                if !capabilities.offers(&encoder) {
                    return Err(CapabilityError::new_err(format!(
                        "installed ffmpeg does not offer encoder {:?}; {}",
                        encoder,
                        fmn_output::NATIVE_ALTERNATIVE,
                    )));
                }
                let sink = fmn_output::FfmpegSink::new(
                    runner,
                    fmn_output::FfmpegSinkConfig {
                        tool,
                        capabilities,
                        job: video.job,
                        audio: None,
                        destination,
                        workdir_root,
                        job_limits: fmn_output::JobLimits::default(),
                        first_sequence: 0,
                        limits,
                        profile: None,
                    },
                )
                .map_err(native_error)?;
                let sink = sink.with_no_clobber();
                let (sink, completion) = sink.with_deferred_soundtrack().map_err(native_error)?;
                soundtrack = Some(completion);
                let (binding, receipt) = sink.into_binding("python-ffmpeg-video");
                (binding, PortalReceipt::Video(receipt))
            }
        };
        let emitter = OrderedEmitter::new(
            EmitterConfig::new(output_layout, plan.frames_in_flight, 0)
                .map_err(|error| PyRuntimeError::new_err(error.to_string()))?,
            vec![binding],
        )
        .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        let mut runtime_config = RuntimeConfig::from_config(&config);
        if single_frame {
            // Match the native front door's final-state PNG semantics: run
            // every segment to its semantic endpoint without ordinary frame
            // capture, then `_finish_render` publishes one explicit `show`.
            runtime_config.skip_animations = true;
            runtime_config.preview_while_skipping = false;
        }
        Ok((
            Self {
                renderer,
                camera,
                emitter: Some(emitter),
                receipt,
                soundtrack,
                audio,
                light_mob: None,
                rgba8_scratch,
                opaque_video,
                next_sequence: 0,
                timeline: OutputTimeline::new(single_frame),
                planar_baseline,
                planar_map_matches,
                // One renderer on the first team: fm-sq8.5 item 6 routes the
                // portal through every team.
                routes: PortalRoutes {
                    teams_used: 1,
                    teams_planned: plan.render_teams.len() as u64,
                    team_threads: plan
                        .render_teams
                        .first()
                        .map_or(1, fmn_runtime::TeamPlan::threads)
                        as u64,
                    frames_in_flight: plan.frames_in_flight as u64,
                    ..PortalRoutes::default()
                },
            },
            runtime_config,
        ))
    }

    fn capture(
        &mut self,
        packet: fmn_scene::studio_bridge::FramePacket,
    ) -> Result<(), fmn_scene::IntegrationError> {
        let stage = packet.materialize_stage();
        if let Some(light) = self.light_mob {
            if stage.get(light).is_none() {
                return Err(fmn_scene::IntegrationError::new(
                    "portal-render",
                    "camera light handle is stale",
                ));
            }
            self.camera
                .set_light_source_position(stage.get_center(light))
                .map_err(|error| {
                    fmn_scene::IntegrationError::new("portal-render", error.to_string())
                })?;
        }
        // fm-sq8.11: planar vector content under the default camera frame
        // takes Lumen's retained 2D route, the one standalone fmn uses (tile
        // cache, adaptive AA, revisioned plan reuse); its +Y-up screen map
        // renders upright since fm-sq8.9. Anything else (a moved or rotated
        // frame, another output aspect, depth, lighting, clip planes, raster
        // primitives, non-planar points) keeps the camera route.
        let frame = self.camera.frame();
        let baseline = &self.planar_baseline;
        let refusal = if !self.planar_map_matches {
            Some("output aspect differs from the default frame")
        } else if frame.center() != baseline.center()
            || frame.shape() != baseline.shape()
            || frame.orientation() != baseline.orientation()
            || frame.field_of_view() != baseline.field_of_view()
        {
            Some("camera frame differs from the default frame")
        } else {
            planar_stage_refusal(&stage)
        };
        if let Some(reason) = refusal {
            self.renderer
                .render_with_camera(&stage, &self.camera)
                .map_err(|error| fmn_scene::IntegrationError::new("lumen", error.to_string()))?;
            self.routes.camera += 1;
            self.routes.first_camera_reason.get_or_insert(reason);
        } else {
            self.renderer.set_background(self.camera.background());
            self.renderer
                .render(&stage, 0)
                .map_err(|error| fmn_scene::IntegrationError::new("lumen", error.to_string()))?;
            self.routes.planar += 1;
        }
        let mut reservation = self
            .emitter
            .as_ref()
            .ok_or_else(|| {
                fmn_scene::IntegrationError::new("reel", "render generation is already closed")
            })?
            .reserve(self.next_sequence)
            .map_err(|error| fmn_scene::IntegrationError::new("reel", error.to_string()))?;
        portal_video::convert_frame(
            self.renderer.frame(),
            reservation.frame_mut(),
            self.rgba8_scratch.as_mut(),
        )
        .map_err(|error| fmn_scene::IntegrationError::new("reel", error.to_string()))?;
        reservation
            .publish()
            .map_err(|error| fmn_scene::IntegrationError::new("reel", error.to_string()))?;
        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or_else(|| fmn_scene::IntegrationError::new("reel", "frame sequence exhausted"))?;
        Ok(())
    }

    fn bind_camera(
        &mut self,
        frame: fmn_scene::studio_bridge::CameraFrame,
        light_position: [f64; 3],
        background: Option<fmn_core::color::LinearRgba>,
        light_mob: Option<Mob>,
    ) -> PyResult<()> {
        *self.camera.frame_mut() = frame;
        self.camera
            .set_light_source_position(light_position)
            .map_err(camera_error)?;
        if let Some(background) = background {
            if self.opaque_video && background.a < 1.0 {
                return Err(PyValueError::new_err(
                    "portal-render: alpha cannot be added to an opaque video generation; configure a transparent MOV before rendering",
                ));
            }
            self.camera
                .set_background(background)
                .map_err(camera_error)?;
        }
        self.light_mob = light_mob;
        Ok(())
    }

    const fn frame_count(&self) -> u64 {
        self.next_sequence
    }

    fn abort(mut self) {
        self.cancel_and_join();
    }

    fn finish(mut self, scene: &Scene) -> PyResult<(PortalArtifactReport, String, usize)> {
        let renderer = self.renderer.config();
        if let Some(soundtrack) = self.soundtrack.take() {
            soundtrack
                .finish(self.audio.mix(scene, renderer.threads, &self.timeline)?)
                .map_err(native_error)?;
        }
        self.emitter
            .take()
            .ok_or_else(|| PyRuntimeError::new_err("portal render generation is already closed"))?
            .finish()
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        let mut report: PortalArtifactReport = match &self.receipt {
            PortalReceipt::Native(receipt) => receipt.take().map_err(native_error)?.into(),
            PortalReceipt::Video(receipt) => {
                let report = receipt.take().map_err(native_error)?;
                PortalArtifactReport {
                    path: report.boundary.destination,
                    frame_count: report.frame_count,
                    bytes: report.boundary.artifact_bytes,
                    digest: report.boundary.artifact_digest,
                    invocations: report.boundary.invocations,
                    audio_inputs: Vec::new(),
                    routes: PortalRoutes::default(),
                }
            }
        };
        // The video encoder starts first; cue decodes finish before the final
        // mux. Preserve that ordering and do not conceal external audio work.
        let index = usize::from(!report.invocations.is_empty());
        report
            .invocations
            .splice(index..index, self.audio.invocations());
        report.audio_inputs = std::mem::take(&mut self.audio.inputs);
        report.routes = self.routes;
        Ok((report, renderer.engine.closure_string(), renderer.threads))
    }

    fn cancel_and_join(&mut self) {
        if let Some(emitter) = self.emitter.take() {
            emitter.cancel();
            let _ = emitter.finish();
        }
    }
}

impl Drop for PortalFrameSession {
    fn drop(&mut self) {
        self.cancel_and_join();
    }
}

/// One scene worker's runtime plus pin releases deferred by proxy destruction.
///
/// Python can decref a proxy at nearly any boundary. If that happens while a
/// Scene borrow is live, `Drop` queues the unpin instead of silently leaking
/// it or panicking through CPython. Every subsequent runtime borrow drains the
/// queue before exposing the Scene.
struct EngineState {
    scene: RefCell<Scene>,
    deferred_unpins: RefCell<Vec<Mob>>,
}

impl EngineState {
    fn new(scene: Scene) -> Self {
        Self {
            scene: RefCell::new(scene),
            deferred_unpins: RefCell::new(Vec::new()),
        }
    }

    fn drain_deferred_unpins(&self) {
        let pending = std::mem::take(&mut *self.deferred_unpins.borrow_mut());
        if pending.is_empty() {
            return;
        }
        let mut scene = self.scene.borrow_mut();
        for mob in pending {
            scene.stage_mut().unpin(mob);
        }
    }

    fn borrow(&self) -> Ref<'_, Scene> {
        self.drain_deferred_unpins();
        self.scene.borrow()
    }

    fn borrow_mut(&self) -> RefMut<'_, Scene> {
        self.drain_deferred_unpins();
        self.scene.borrow_mut()
    }

    fn release_pin(&self, mob: Mob) {
        if let Ok(mut scene) = self.scene.try_borrow_mut() {
            let pending = std::mem::take(&mut *self.deferred_unpins.borrow_mut());
            for deferred in pending {
                scene.stage_mut().unpin(deferred);
            }
            scene.stage_mut().unpin(mob);
        } else {
            self.deferred_unpins.borrow_mut().push(mob);
        }
    }
}

/// A detached proxy's builder state: a private, proxy-owned Stage (the
/// "nursery") plus the root handle its mobject was added with.
///
/// Arena residency without scene membership is a supported Stage mode, so
/// positional/geometry operations run through the exact same Stage code
/// path in both proxy states — the nursery before Scene.add, the scene's
/// stage after. Each nursery holds exactly one root: detached family
/// structure stays in the Python `submobjects` list until `Scene.add`
/// binds the whole graph.
struct Nursery {
    stage: Stage,
    root: Mob,
}

impl Nursery {
    fn new(mobject: Mobject) -> Self {
        let mut stage = Stage::new();
        let root = stage.add(mobject);
        Self { stage, root }
    }

    /// A nursery whose root is a native value tracker (§8.6): `kind` is
    /// 0 = Plain, 1 = Exponential, 2 = Complex (re/im in `value`/`im`).
    fn value_tracker(kind: u8, value: f64, im: f64) -> Self {
        let mut stage = Stage::new();
        let root = match kind {
            1 => stage.add_exponential_value_tracker(value),
            2 => stage.add_complex_value_tracker(value, im),
            _ => stage.add_value_tracker(value),
        };
        Self { stage, root }
    }
}

/// Subclassable Python proxy over either detached builder state (a private
/// nursery Stage) or one Stage-scoped Marionette handle.
#[pyclass(subclass, weakref, dict, unsendable, name = "_BridgeMobject")]
struct BridgeMobject {
    nursery: Option<Nursery>,
    engine: Option<Engine>,
    mob: Option<Mob>,
    initialized: bool,
}

/// Subclassable scene seam. The worker is deliberately single-threaded;
/// `unsendable` turns accidental cross-thread proxy use into a Python error.
#[pyclass(subclass, weakref, dict, unsendable, name = "_SceneCore")]
struct PyScene {
    engine: Engine,
    /// Handle → weakref(proxy), preserving one Python identity per live entry.
    proxies: RefCell<HashMap<Mob, Py<PyAny>, ByAddress>>,
    /// Bumped by every `register_proxy`, so a reseated handle→proxy mapping
    /// invalidates `roots_projection`.
    proxy_generation: Cell<u64>,
    /// `_engine_roots`' last complete projection, keyed by the Stage topology
    /// epoch and `proxy_generation` it was made at (fm-5wq.31).
    roots_projection: RefCell<Option<RootsProjection>>,
    /// Optional production output session shared by every play/wait sink.
    render: Arc<Mutex<Option<PortalRenderSession>>>,
    render_invocations: Vec<fmn_output::InvocationReport>,
    render_audio_inputs: Vec<portal_audio::PortalAudioInput>,
    /// The last render's frame routes (fm-sq8.11).
    render_routes: PortalRoutes,
}

/// Why a stage cannot take Lumen's retained planar route, or `None` when it
/// can. This mirrors the planar engine's own refusal
/// (`FrameJobError::CameraProjectionRequired`): only vector programs without
/// fixed-in-frame placement, zoom-scaled strokes, depth testing, lighting or
/// clip planes, with every drawn point at world z = 0. Planar FMTL export and
/// the portal's planar render route share this one rule (fm-sq8.11).
pub(crate) fn planar_stage_refusal(stage: &Stage) -> Option<&'static str> {
    for item in stage.draw_plan().items() {
        let Some(entry) = stage.get(item.mob) else {
            return Some("stale drawable");
        };
        let uniforms = entry.uniforms();
        if item.key.program != fmn_mobject::ProgramKind::Vector
            || uniforms.depth_test
            || uniforms.shading != [0.0; 3]
            || uniforms.clip_planes != [[0.0; 4]; 4]
        {
            return Some(
                "depth, raster primitives, lighting or clip planes require a camera-bearing bundle",
            );
        }
        if uniforms.is_fixed_in_frame != 0.0 || uniforms.scale_stroke_with_zoom {
            return Some("fixed-in-frame or zoom-scaled strokes require a camera-bearing bundle");
        }
        // Test world z: positional operations compose into the entry's
        // object-to-world placement, so a shift(OUT) leaves the stored record
        // points planar while the drawn geometry is not.
        let placement = entry.placement();
        if let Some(points) = entry.buffer.read_column("point")
            && points
                .as_chunks::<3>()
                .0
                .iter()
                .any(|point| placement.apply_point(point.map(f64::from))[2] != 0.0)
        {
            return Some("nonplanar geometry requires a camera-bearing bundle");
        }
    }
    None
}

/// Owns one pinned RecordBuffer generation while a NumPy array exports it.
#[pyclass(unsendable, name = "_RecordViewOwner")]
struct PyRecordView {
    view: RecordView,
}

fn stage_error(error: StageError) -> PyErr {
    match error {
        StageError::StaleHandle => StaleHandleError::new_err(error.to_string()),
        StageError::CycleDetected => FamilyCycleError::new_err(error.to_string()),
        other => PyRuntimeError::new_err(other.to_string()),
    }
}

fn bound_parts(proxy: &BridgeMobject) -> PyResult<(Engine, Mob)> {
    match (&proxy.engine, proxy.mob) {
        (Some(engine), Some(mob)) => Ok((Rc::clone(engine), mob)),
        _ => Err(StaleHandleError::new_err(
            "mobject is detached; add it to a Scene before using a Scene-only operation",
        )),
    }
}

fn same_engine(left: &Engine, right: &Engine) -> bool {
    Rc::ptr_eq(left, right)
}

/// The typed Python surface of a record sizing refusal (fm-vek.2):
/// schema stride and buffer shape overflows raise `OverflowError`, the
/// same exception class the bridge has always used for shape arithmetic.
fn record_error_to_py(error: RecordError) -> PyErr {
    match error {
        RecordError::StrideOverflow => {
            PyOverflowError::new_err("record dtype stride overflows usize")
        }
        RecordError::SizeOverflow { .. } => {
            PyOverflowError::new_err("RecordBuffer shape overflows usize")
        }
    }
}

fn with_buffer<T>(
    proxy: &Bound<'_, BridgeMobject>,
    operation: impl FnOnce(&mut RecordBuffer) -> T,
) -> PyResult<T> {
    let bound = {
        let cell = proxy.borrow();
        cell.engine
            .as_ref()
            .zip(cell.mob)
            .map(|(engine, mob)| (Rc::clone(engine), mob))
    };
    if let Some((engine, mob)) = bound {
        let mut scene = engine.borrow_mut();
        let stage = scene.stage_mut();
        stage.bake_placement(mob).map_err(stage_error)?;
        let entry = stage
            .get_mut(mob)
            .ok_or_else(|| StaleHandleError::new_err("mobject handle no longer resolves"))?;
        return Ok(operation(&mut entry.buffer));
    }
    let mut cell = proxy.borrow_mut();
    let nursery = cell
        .nursery
        .as_mut()
        .ok_or_else(|| StaleHandleError::new_err("mobject has no detached or bound state"))?;
    let root = nursery.root;
    nursery.stage.bake_placement(root).map_err(stage_error)?;
    let entry = nursery
        .stage
        .get_mut(root)
        .ok_or_else(|| StaleHandleError::new_err("nursery root no longer resolves"))?;
    Ok(operation(&mut entry.buffer))
}

fn with_buffer_ref<T>(
    proxy: &Bound<'_, BridgeMobject>,
    operation: impl FnOnce(&RecordBuffer) -> T,
) -> PyResult<T> {
    let bound = {
        let cell = proxy.borrow();
        cell.engine
            .as_ref()
            .zip(cell.mob)
            .map(|(engine, mob)| (Rc::clone(engine), mob))
    };
    if let Some((engine, mob)) = bound {
        // RecordBuffer is the live `mobject.data` contract. Materialize any
        // pending affine placement before a Python read so the zero-copy view
        // remains authoritative and exposes the same world-space points as
        // manim's data array (§8.2, fm-7if).
        let mut scene = engine.borrow_mut();
        let stage = scene.stage_mut();
        stage.bake_placement(mob).map_err(stage_error)?;
        let entry = stage
            .get(mob)
            .ok_or_else(|| StaleHandleError::new_err("mobject handle no longer resolves"))?;
        return Ok(operation(&entry.buffer));
    }
    let mut cell = proxy.borrow_mut();
    let nursery = cell
        .nursery
        .as_mut()
        .ok_or_else(|| StaleHandleError::new_err("mobject has no detached or bound state"))?;
    let root = nursery.root;
    // The nursery is one Stage code path with the bound branch: a pending
    // placement bakes before Python observes the buffer.
    nursery.stage.bake_placement(root).map_err(stage_error)?;
    let entry = nursery
        .stage
        .get(root)
        .ok_or_else(|| StaleHandleError::new_err("nursery root no longer resolves"))?;
    Ok(operation(&entry.buffer))
}

/// Route one positional/geometry operation to the proxy's Stage in either
/// state: the scene's stage when bound, the private nursery when detached.
/// This is the single seam every positional binding uses.
fn with_stage<T>(
    proxy: &Bound<'_, BridgeMobject>,
    operation: impl FnOnce(&mut Stage, Mob) -> T,
) -> PyResult<T> {
    let bound = {
        let cell = proxy.borrow();
        cell.engine
            .as_ref()
            .zip(cell.mob)
            .map(|(engine, mob)| (Rc::clone(engine), mob))
    };
    if let Some((engine, mob)) = bound {
        let mut scene = engine.borrow_mut();
        return Ok(operation(scene.stage_mut(), mob));
    }
    let mut cell = proxy.borrow_mut();
    let nursery = cell
        .nursery
        .as_mut()
        .ok_or_else(|| StaleHandleError::new_err("mobject has no detached or bound state"))?;
    let root = nursery.root;
    Ok(operation(&mut nursery.stage, root))
}

/// Reconstruct the current shared-anchor geometry under the VMobject's live
/// path-building controls.  The Python object owns these public mutable
/// attributes; Chisel owns every operation performed on the resulting path.
fn configured_quad_path(proxy: &Bound<'_, BridgeMobject>) -> PyResult<fmn_library::QuadPath> {
    let tolerance = proxy
        .getattr("tolerance_for_point_equality")?
        .extract::<f64>()?;
    let long_lines = proxy.getattr("long_lines")?.extract::<bool>()?;
    let points = with_stage(proxy, |stage, mob| {
        stage.get_points(mob).unwrap_or_default()
    })?;
    let mut path = fmn_library::QuadPath::from_points(points).map_err(native_error)?;
    path.set_tolerance_for_point_equality(tolerance);
    path.set_long_lines(long_lines);
    Ok(path)
}

/// Convert an `operator.index`-normalized Python subdivision count without
/// ever narrowing an attacker-controlled large integer. Returning the native
/// cap makes QuadPath produce its ordinary typed budget refusal.
fn bounded_subdivision_count(value: &Bound<'_, PyAny>) -> PyResult<usize> {
    if value
        .rich_compare(fmn_library::MAX_SUBDIVIDED_CURVES, CompareOp::Gt)?
        .is_truthy()?
    {
        Ok(fmn_library::MAX_SUBDIVIDED_CURVES)
    } else {
        value.extract::<usize>()
    }
}

/// Reference count coercion: non-positive values mean no split; positive
/// values must implement `__index__`, as NumPy's `linspace(num=...)` requires.
fn positive_subdivision_count(value: &Bound<'_, PyAny>) -> PyResult<usize> {
    if !value.rich_compare(0, CompareOp::Gt)?.is_truthy()? {
        return Ok(0);
    }
    let indexed = value
        .py()
        .import("operator")?
        .getattr("index")?
        .call1((value,))?;
    bounded_subdivision_count(&indexed)
}

/// Apply one native path-building operation without mutating the Stage and
/// return only the appended rows.  Python commits those rows through its
/// existing `append_points` surface so all non-geometry RecordBuffer lanes
/// follow the Reference's resize/copy rules.  Computing first also makes
/// every typed refusal atomic at the portal boundary.
/// Point rows from a C-contiguous float64 (N, 3) buffer (the bootstrap's
/// `_point_array`) without one Python float per coordinate, else through the
/// ordinary sequence extraction with its ordinary errors (fm-5wq.31: a live
/// graph's redraw converted every row array to lists and back each frame).
fn point_rows(points: &Bound<'_, PyAny>) -> PyResult<Vec<[f64; 3]>> {
    if let Ok(buffer) = PyBuffer::<f64>::get(points)
        && buffer.dimensions() == 2
        && buffer.shape().get(1) == Some(&3)
        && buffer.is_c_contiguous()
    {
        let flat = buffer.to_vec(points.py())?;
        // A C-contiguous (N, 3) buffer holds exactly 3N values.
        return Ok(flat.as_chunks::<3>().0.to_vec());
    }
    points.extract()
}

/// Point rows as a writable float64 (N, 3) ndarray, built from packed bytes
/// rather than N lists of three Python floats.
fn point_array<'py>(py: Python<'py>, rows: &[[f64; 3]]) -> PyResult<Bound<'py, PyAny>> {
    let mut bytes = Vec::with_capacity(rows.len() * 3 * std::mem::size_of::<f64>());
    for value in rows.iter().flatten() {
        bytes.extend_from_slice(&value.to_ne_bytes());
    }
    let numpy = py.import("numpy")?;
    numpy
        .getattr("frombuffer")?
        .call1((PyByteArray::new(py, &bytes), "=f8"))?
        .call_method1("reshape", ((rows.len(), 3),))
}

fn quad_path_tail(
    proxy: &Bound<'_, BridgeMobject>,
    operation: impl FnOnce(&mut fmn_library::QuadPath) -> PyResult<()>,
) -> PyResult<Vec<[f64; 3]>> {
    let mut path = configured_quad_path(proxy)?;
    let old_len = path.num_points();
    operation(&mut path)?;
    path.points()
        .get(old_len..)
        .map(<[_]>::to_vec)
        .ok_or_else(|| PyRuntimeError::new_err("native path operation removed existing points"))
}

/// Copy one proxy's root entry into an unrelated Stage without copying its
/// descendants. Callers handle the same-Stage fast path before entering this
/// helper, so borrowing a bound source here can never alias `target`.
fn copy_proxy_entry_into(source: &Bound<'_, BridgeMobject>, target: &mut Stage) -> PyResult<Mob> {
    let bound = {
        let cell = source.borrow();
        cell.engine
            .as_ref()
            .zip(cell.mob)
            .map(|(engine, mob)| (Rc::clone(engine), mob))
    };
    if let Some((engine, mob)) = bound {
        return engine
            .borrow()
            .stage()
            .copy_entry_into(mob, target)
            .map_err(stage_error);
    }
    let cell = source.borrow();
    let nursery = cell.nursery.as_ref().ok_or_else(|| {
        StaleHandleError::new_err("partial source has no detached or bound state")
    })?;
    nursery
        .stage
        .copy_entry_into(nursery.root, target)
        .map_err(stage_error)
}

fn with_uniforms<T>(
    proxy: &Bound<'_, BridgeMobject>,
    operation: impl FnOnce(&mut Uniforms) -> PyResult<T>,
) -> PyResult<T> {
    let bound = {
        let cell = proxy.borrow();
        cell.engine
            .as_ref()
            .zip(cell.mob)
            .map(|(engine, mob)| (Rc::clone(engine), mob))
    };
    if let Some((engine, mob)) = bound {
        let mut scene = engine.borrow_mut();
        let entry = scene
            .stage_mut()
            .get_mut(mob)
            .ok_or_else(|| StaleHandleError::new_err("mobject handle no longer resolves"))?;
        return operation(entry.uniforms_mut());
    }
    let mut cell = proxy.borrow_mut();
    let nursery = cell
        .nursery
        .as_mut()
        .ok_or_else(|| StaleHandleError::new_err("mobject has no detached or bound state"))?;
    let root = nursery.root;
    let entry = nursery
        .stage
        .get_mut(root)
        .ok_or_else(|| StaleHandleError::new_err("nursery root no longer resolves"))?;
    operation(entry.uniforms_mut())
}

fn uniforms_snapshot(proxy: &Bound<'_, BridgeMobject>) -> PyResult<Uniforms> {
    let bound = {
        let cell = proxy.borrow();
        cell.engine
            .as_ref()
            .zip(cell.mob)
            .map(|(engine, mob)| (Rc::clone(engine), mob))
    };
    if let Some((engine, mob)) = bound {
        let scene = engine.borrow();
        return scene
            .stage()
            .get(mob)
            .map(|entry| *entry.uniforms())
            .ok_or_else(|| StaleHandleError::new_err("mobject handle no longer resolves"));
    }
    let cell = proxy.borrow();
    cell.nursery
        .as_ref()
        .and_then(|nursery| nursery.stage.get(nursery.root))
        .map(|entry| *entry.uniforms())
        .ok_or_else(|| StaleHandleError::new_err("mobject has no detached or bound state"))
}

fn extract_string_list(value: &Bound<'_, PyAny>, label: &str) -> PyResult<Vec<String>> {
    let mut result = Vec::new();
    for item in value.try_iter()? {
        result.push(
            item?.extract::<String>().map_err(|_| {
                PyTypeError::new_err(format!("{label} entries must all be strings"))
            })?,
        );
    }
    Ok(result)
}

fn extract_shape_width(value: &Bound<'_, PyAny>) -> PyResult<usize> {
    if let Ok(width) = value.extract::<usize>() {
        return Ok(width);
    }
    let mut width = 1usize;
    let mut dimensions = 0usize;
    for dimension in value.try_iter().map_err(|_| {
        PyTypeError::new_err("data_dtype shape must be an integer or an iterable of integers")
    })? {
        let dimension = dimension?.extract::<usize>().map_err(|_| {
            PyTypeError::new_err("data_dtype shape dimensions must be non-negative integers")
        })?;
        width = width
            .checked_mul(dimension)
            .ok_or_else(|| PyOverflowError::new_err("data_dtype lane count overflows usize"))?;
        dimensions += 1;
    }
    if dimensions == 0 {
        return Err(PyValueError::new_err(
            "data_dtype shape declares no dimensions",
        ));
    }
    Ok(width)
}

/// The `numpy` module, imported once (fm-ztjj: `.data` reads and schema
/// parsing both resolved it on every call).
fn numpy_module(py: Python<'_>) -> PyResult<&Bound<'_, PyModule>> {
    static NUMPY: pyo3::sync::PyOnceLock<Py<PyModule>> = pyo3::sync::PyOnceLock::new();
    NUMPY
        .get_or_try_init(py, || py.import("numpy").map(Bound::unbind))
        .map(|numpy| numpy.bind(py))
}

fn validate_field_dtype(value: &Bound<'_, PyAny>) -> PyResult<()> {
    static FLOAT32: pyo3::sync::PyOnceLock<Py<pyo3::types::PyType>> = pyo3::sync::PyOnceLock::new();
    let py = value.py();
    let numpy = numpy_module(py).map_err(|error| {
        PyImportError::new_err(format!(
            "NumPy is required to interpret a three-item data_dtype entry: {error}"
        ))
    })?;
    // `numpy.float32`, the field type stock data_dtypes name, is native
    // float32 by definition; anything else takes the full check.
    if value.is(FLOAT32.import(py, "numpy", "float32")?) {
        return Ok(());
    }
    let dtype = numpy.getattr("dtype")?.call1((value,))?;
    let kind: String = dtype.getattr("kind")?.extract()?;
    let itemsize: usize = dtype.getattr("itemsize")?.extract()?;
    let is_native: bool = dtype.getattr("isnative")?.extract()?;
    if kind != "f" || itemsize != std::mem::size_of::<f32>() || !is_native {
        return Err(PyTypeError::new_err(
            "RecordBuffer data_dtype fields must use native-endian float32",
        ));
    }
    Ok(())
}

/// Accept both the compact bridge descriptor `(name, lanes)` and NumPy's
/// ordinary `(name, dtype, shape)` field descriptor used by manim subclasses.
fn parse_schema(proxy: &Bound<'_, BridgeMobject>) -> PyResult<RecordSchema> {
    let dtype = proxy.getattr("data_dtype")?;
    let mut names = Vec::new();
    let mut widths = Vec::new();
    let mut seen = HashSet::new();
    for item in dtype.try_iter()? {
        let item = item?;
        let tuple = item.cast::<PyTuple>().map_err(|_| {
            PyTypeError::new_err("data_dtype entries must be (name, lanes) or (name, dtype, shape)")
        })?;
        let name = tuple
            .get_item(0)?
            .extract::<String>()
            .map_err(|_| PyTypeError::new_err("data_dtype field names must be strings"))?;
        let width = match tuple.len() {
            2 => tuple.get_item(1)?.extract::<usize>().map_err(|_| {
                PyTypeError::new_err("two-item data_dtype entries must use an integer lane count")
            })?,
            3 => {
                validate_field_dtype(&tuple.get_item(1)?)?;
                extract_shape_width(&tuple.get_item(2)?)?
            }
            _ => {
                return Err(PyTypeError::new_err(
                    "data_dtype entries must contain exactly two or three items",
                ));
            }
        };
        if name.is_empty() {
            return Err(PyValueError::new_err(
                "data_dtype field names cannot be empty",
            ));
        }
        if width == 0 {
            return Err(PyValueError::new_err(format!(
                "data_dtype field `{name}` has zero lanes"
            )));
        }
        if !seen.insert(name.clone()) {
            return Err(PyValueError::new_err(format!(
                "data_dtype field `{name}` is declared more than once"
            )));
        }
        names.push(name);
        widths.push(width);
    }
    if names.is_empty() {
        return Err(PyValueError::new_err("data_dtype declares no fields"));
    }
    widths.iter().try_fold(0usize, |stride, width| {
        stride
            .checked_add(*width)
            .ok_or_else(|| PyOverflowError::new_err("data_dtype stride overflows usize"))
    })?;

    let aligned = extract_string_list(&proxy.getattr("aligned_data_keys")?, "aligned_data_keys")?;
    let pointlike = extract_string_list(
        &proxy.getattr("pointlike_data_keys")?,
        "pointlike_data_keys",
    )?;
    for key in aligned.iter().chain(pointlike.iter()) {
        if !seen.contains(key) {
            return Err(PyValueError::new_err(format!(
                "record key `{key}` is not declared by data_dtype"
            )));
        }
    }
    let fields: Vec<(&str, usize)> = names
        .iter()
        .zip(widths.iter().copied())
        .map(|(name, width)| (name.as_str(), width))
        .collect();
    let aligned_refs: Vec<&str> = aligned.iter().map(String::as_str).collect();
    let pointlike_refs: Vec<&str> = pointlike.iter().map(String::as_str).collect();
    // The stride was already proved above; the fallible schema constructor
    // (fm-vek.2) re-proves it and any refusal surfaces as the same typed
    // Python exception.
    RecordSchema::new(&fields, &aligned_refs, &pointlike_refs).map_err(record_error_to_py)
}

fn proxy_children<'py>(proxy: &Bound<'py, PyAny>) -> PyResult<Vec<Bound<'py, PyAny>>> {
    let children = proxy.getattr("submobjects")?;
    children.try_iter()?.collect()
}

fn collect_proxy_graph<'py>(root: &Bound<'py, PyAny>) -> PyResult<Vec<Py<PyAny>>> {
    fn visit<'py>(
        proxy: &Bound<'py, PyAny>,
        seen: &mut HashSet<usize>,
        visiting: &mut HashSet<usize>,
        output: &mut Vec<Py<PyAny>>,
    ) -> PyResult<()> {
        proxy
            .cast::<BridgeMobject>()
            .map_err(|_| PyTypeError::new_err("submobjects must be Mobject instances"))?;
        let marker = proxy.as_ptr() as usize;
        if visiting.contains(&marker) {
            return Err(FamilyCycleError::new_err(
                "submobjects would create a family cycle",
            ));
        }
        if !seen.insert(marker) {
            return Ok(());
        }
        visiting.insert(marker);
        output.push(proxy.clone().unbind());
        for child in proxy_children(proxy)? {
            visit(&child, seen, visiting, output)?;
        }
        visiting.remove(&marker);
        Ok(())
    }

    let mut output = Vec::new();
    visit(root, &mut HashSet::new(), &mut HashSet::new(), &mut output)?;
    Ok(output)
}

fn register_proxy(
    py: Python<'_>,
    scene: &Bound<'_, PyScene>,
    mob: Mob,
    proxy: &Bound<'_, PyAny>,
) -> PyResult<()> {
    let weakref = py.import("weakref")?.call_method1("ref", (proxy,))?;
    let scene = scene.borrow();
    scene.proxies.borrow_mut().insert(mob, weakref.unbind());
    scene
        .proxy_generation
        .set(scene.proxy_generation.get().wrapping_add(1));
    Ok(())
}

/// A complete `_engine_roots` projection: every member of every root family
/// had a registered live proxy whose Python `submobjects` list held exactly
/// its native children, in order.
struct RootsProjection {
    /// (Stage topology epoch, proxy generation) when the projection finished.
    key: (u64, u64),
    /// The registry's weakrefs to the root proxies, in draw order.
    roots: Vec<Py<PyAny>>,
    /// Each member's `Entry::children_epoch` when its Python child list was
    /// last seen to hold exactly its native children.
    verified: HashMap<Mob, u64, ByAddress>,
}

/// Keep a finished projection when nothing moved while it ran: shells
/// registered during it, or Python run by `_replace_projection`, change the
/// key, and the next read then projects again.
fn store_projection(
    scene: &Bound<'_, PyScene>,
    key: (u64, u64),
    roots: &[Mob],
    verified: HashMap<Mob, u64, ByAddress>,
) {
    let py = scene.py();
    let finished = projection_key(scene);
    let scene = scene.borrow();
    let weakrefs: Option<Vec<Py<PyAny>>> = {
        let registry = scene.proxies.borrow();
        roots
            .iter()
            .map(|mob| registry.get(mob).map(|weak| weak.clone_ref(py)))
            .collect()
    };
    *scene.roots_projection.borrow_mut() = match weakrefs {
        Some(roots) if finished == key => Some(RootsProjection {
            key,
            roots,
            verified,
        }),
        _ => None,
    };
}

/// `_engine_roots` after a topology change that left every proxy identity in
/// place. A member whose child list still has the epoch it was verified at
/// holds exactly its native children, so only members whose list moved are
/// compared, and re-projected where they differ, in the full walk's order.
/// `None` when a needed proxy is not live: creating shells is the full
/// walk's job, since a new shell's own list starts empty.
fn reproject_moved<'py>(
    scene: &Bound<'py, PyScene>,
    key: (u64, u64),
    verified: &HashMap<Mob, u64, ByAddress>,
) -> PyResult<Option<Vec<Py<PyAny>>>> {
    let py = scene.py();
    let engine = Rc::clone(&scene.borrow().engine);
    let mut epochs: HashMap<Mob, u64, ByAddress> = HashMap::default();
    epochs.reserve(verified.len());
    let mut moved = Vec::new();
    let roots = {
        let runtime = engine.borrow();
        let stage = runtime.stage();
        let roots = stage.roots().to_vec();
        for root in &roots {
            for mob in stage.family(*root) {
                if epochs.contains_key(&mob) {
                    continue;
                }
                let entry = stage.get(mob).ok_or_else(|| {
                    StaleHandleError::new_err("scene family contains a stale handle")
                })?;
                let epoch = entry.children_epoch();
                if verified.get(&mob) != Some(&epoch) {
                    moved.push((mob, entry.submobjects().to_vec()));
                }
                epochs.insert(mob, epoch);
            }
        }
        roots
    };
    let mut proxies: HashMap<Mob, Bound<'py, PyAny>, ByAddress> = HashMap::default();
    let mut proxy = |mob: Mob| -> Option<Bound<'py, PyAny>> {
        if let Some(found) = proxies.get(&mob) {
            return Some(found.clone());
        }
        let found = live_proxy(py, scene, mob)?;
        proxies.insert(mob, found.clone());
        Some(found)
    };
    let mut root_proxies = Vec::with_capacity(roots.len());
    for &root in &roots {
        let Some(found) = proxy(root) else {
            return Ok(None);
        };
        root_proxies.push(found);
    }
    let submobjects = pyo3::intern!(py, "submobjects");
    for (mob, children) in &moved {
        let Some(owner) = proxy(*mob) else {
            return Ok(None);
        };
        let mut expected = Vec::with_capacity(children.len());
        for &child in children {
            let Some(found) = proxy(child) else {
                return Ok(None);
            };
            expected.push(found);
        }
        let live = owner.getattr(submobjects)?;
        if let Ok(current) = live.cast::<PyList>()
            && current.len() == expected.len()
            && current
                .iter()
                .zip(&expected)
                .all(|(item, child)| item.is(child))
        {
            continue;
        }
        live.call_method1("_replace_projection", (PyList::new(py, &expected)?,))?;
    }
    store_projection(scene, key, &roots, epochs);
    Ok(Some(root_proxies.into_iter().map(Bound::unbind).collect()))
}

fn projection_key(scene: &Bound<'_, PyScene>) -> (u64, u64) {
    let scene = scene.borrow();
    let epoch = scene.engine.borrow().stage().topology_epoch();
    (epoch, scene.proxy_generation.get())
}

/// The cached roots, when nothing a projection reads has changed since it
/// was made: the same topology epoch (no root or edge change), the same
/// proxy generation (no reseated identity), and every root proxy alive.
/// Then every member's registered proxy and Python child list are still the
/// ones the projection established, so re-projecting would change nothing.
fn projected_roots(scene: &Bound<'_, PyScene>, key: (u64, u64)) -> Option<Vec<Py<PyAny>>> {
    let py = scene.py();
    let weakrefs: Vec<Py<PyAny>> = {
        let scene = scene.borrow();
        let cache = scene.roots_projection.borrow();
        let projection = cache.as_ref().filter(|projection| projection.key == key)?;
        projection
            .roots
            .iter()
            .map(|weak| weak.clone_ref(py))
            .collect()
    };
    weakrefs
        .iter()
        .map(|weak| {
            let target = weak.bind(py).call0().ok()?;
            (!target.is_none()).then(|| target.unbind())
        })
        .collect()
}

fn live_proxy<'py>(
    py: Python<'py>,
    scene: &Bound<'py, PyScene>,
    mob: Mob,
) -> Option<Bound<'py, PyAny>> {
    let weak = {
        let scene = scene.borrow();
        scene
            .proxies
            .borrow()
            .get(&mob)
            .map(|weak| weak.clone_ref(py))
    }?;
    let target = weak.bind(py).call0().ok()?;
    (!target.is_none()).then_some(target)
}

fn bind_graph<'py>(
    py: Python<'py>,
    scene: &Bound<'py, PyScene>,
    root: &Bound<'py, BridgeMobject>,
) -> PyResult<Mob> {
    let graph = collect_proxy_graph(root.as_any())?;
    let engine = Rc::clone(&scene.borrow().engine);

    for object in &graph {
        let proxy = object.bind(py).cast::<BridgeMobject>()?;
        let cell = proxy.borrow();
        if let Some(existing) = &cell.engine
            && !same_engine(existing, &engine)
        {
            return Err(ForeignStageError::new_err(
                "a bound mobject belongs to a different Scene",
            ));
        }
        if cell.engine.is_none() && (!cell.initialized || cell.nursery.is_none()) {
            return Err(StaleHandleError::new_err(
                "uninitialized _BridgeMobject cannot enter a Scene",
            ));
        }
    }

    for object in &graph {
        let proxy = object.bind(py).cast::<BridgeMobject>()?;
        if proxy.borrow().engine.is_some() {
            continue;
        }
        // Adoption: move the nursery family into the scene's stage, then
        // retire the nursery. Moving keeps each record buffer's storage, so
        // NumPy views exported before scene.add stay live (§8.2).
        let mob = {
            let mut cell = proxy.borrow_mut();
            let nursery = cell.nursery.as_mut().expect("validated detached state");
            let mut runtime = engine.borrow_mut();
            let mob = nursery
                .stage
                .move_into(nursery.root, runtime.stage_mut())
                .map_err(stage_error)?;
            runtime.stage_mut().pin(mob).map_err(stage_error)?;
            mob
        };
        {
            let mut cell = proxy.borrow_mut();
            cell.nursery = None;
            cell.engine = Some(Rc::clone(&engine));
            cell.mob = Some(mob);
        }
        register_proxy(py, scene, mob, proxy.as_any())?;
        proxy.setattr("_scene", scene)?;
    }

    // Every Python access completed before the arena borrow. The edge
    // mutation below therefore cannot re-enter Python.
    let mut relations = Vec::new();
    for object in &graph {
        let parent = object.bind(py).cast::<BridgeMobject>()?;
        let parent_mob = parent.borrow().mob.expect("bound above");
        let mut children = Vec::new();
        for child in proxy_children(parent.as_any())? {
            let child = child.cast::<BridgeMobject>()?;
            children.push(child.borrow().mob.expect("graph bound every child"));
        }
        relations.push((parent_mob, children));
    }
    {
        let mut runtime = engine.borrow_mut();
        for (parent, children) in relations {
            for child in children {
                runtime
                    .stage_mut()
                    .attach(parent, child)
                    .map_err(stage_error)?;
            }
        }
    }
    root.borrow()
        .mob
        .ok_or_else(|| StaleHandleError::new_err("root did not bind"))
}

#[pymethods]
impl PyRecordView {
    /// CPython's buffer slot. This is the entire project-authored unsafe FFI
    /// surface: validate the destination, publish the pinned generation's
    /// stable pointer, and make the exporter own the lifetime.
    unsafe fn __getbuffer__(
        slf: Bound<'_, Self>,
        view: *mut ffi::Py_buffer,
        flags: c_int,
    ) -> PyResult<()> {
        if view.is_null() {
            return Err(PyBufferError::new_err(
                "CPython supplied a null Py_buffer destination",
            ));
        }
        let (data, byte_len, writable) = {
            let owner = slf.borrow();
            (
                owner.view.foreign_data_ptr(),
                owner.view.foreign_byte_len(),
                owner.view.is_writable(),
            )
        };
        if !writable && flags & ffi::PyBUF_WRITABLE == ffi::PyBUF_WRITABLE {
            return Err(PyBufferError::new_err(
                "the RecordBuffer view was exported read-only",
            ));
        }
        let byte_len = isize::try_from(byte_len)
            .map_err(|_| PyOverflowError::new_err("RecordBuffer exceeds Py_ssize_t"))?;
        // ubs:ignore — CPython buffer-request flag bits are public ABI metadata.
        let format = if flags & ffi::PyBUF_FORMAT == ffi::PyBUF_FORMAT {
            CString::new("B")
                .expect("static buffer format contains no NUL")
                .into_raw()
        } else {
            ptr::null_mut()
        };
        let owner = slf.into_any();
        // SAFETY: `view` was checked non-null. `data` belongs to the
        // RecordView stored in `owner`; assigning owner.into_ptr() transfers
        // one Python reference to Py_buffer.obj, so CPython keeps that view
        // (and its Arc generation) alive until release. Shape/stride pointers
        // refer to fields inside this Py_buffer, exactly as CPython permits.
        unsafe {
            (*view).obj = owner.into_ptr();
            (*view).buf = data.cast::<c_void>();
            (*view).len = byte_len;
            (*view).readonly = i32::from(!writable);
            (*view).itemsize = 1;
            (*view).format = format;
            (*view).ndim = 1;
            (*view).shape = if flags & ffi::PyBUF_ND == ffi::PyBUF_ND {
                &raw mut (*view).len
            } else {
                ptr::null_mut()
            };
            (*view).strides = if flags & ffi::PyBUF_STRIDES == ffi::PyBUF_STRIDES {
                &raw mut (*view).itemsize
            } else {
                ptr::null_mut()
            };
            (*view).suboffsets = ptr::null_mut();
            (*view).internal = ptr::null_mut();
        }
        Ok(())
    }

    unsafe fn __releasebuffer__(&self, view: *mut ffi::Py_buffer) {
        if view.is_null() {
            return;
        }
        // SAFETY: this slot receives the exact Py_buffer initialized above.
        // A non-null format was allocated with CString::into_raw once, and
        // CPython invokes release at most once for this export.
        let format = unsafe { (*view).format };
        if !format.is_null() {
            // SAFETY: paired with CString::into_raw in __getbuffer__.
            drop(unsafe { CString::from_raw(format) });
            // SAFETY: leave no dangling pointer for diagnostics.
            unsafe {
                (*view).format = ptr::null_mut();
            }
        }
    }
}

const UNIFORM_NAMES: &[&str] = &[
    "is_fixed_in_frame",
    "shading",
    "clip_planes",
    "anti_alias_width",
    "joint_type",
    "flat_stroke",
    "scale_stroke_with_zoom",
    "stroke_behind",
    "depth_test",
    "use_winding_fill",
];

fn uniform_value<'py>(
    py: Python<'py>,
    uniforms: Uniforms,
    name: &str,
) -> PyResult<Bound<'py, PyAny>> {
    match name {
        "is_fixed_in_frame" => Ok(uniforms.is_fixed_in_frame.into_pyobject(py)?.into_any()),
        "shading" => Ok(uniforms.shading.to_vec().into_pyobject(py)?.into_any()),
        "clip_planes" => Ok(uniforms
            .clip_planes
            .iter()
            .map(|plane| plane.to_vec())
            .collect::<Vec<_>>()
            .into_pyobject(py)?
            .into_any()),
        "anti_alias_width" => Ok(uniforms.anti_alias_width.into_pyobject(py)?.into_any()),
        "joint_type" => Ok(uniforms.joint_type.to_code().into_pyobject(py)?.into_any()),
        "flat_stroke" => Ok(uniforms
            .flat_stroke
            .into_pyobject(py)?
            .to_owned()
            .into_any()),
        "scale_stroke_with_zoom" => Ok(uniforms
            .scale_stroke_with_zoom
            .into_pyobject(py)?
            .to_owned()
            .into_any()),
        "stroke_behind" => Ok(uniforms
            .stroke_behind
            .into_pyobject(py)?
            .to_owned()
            .into_any()),
        "depth_test" => Ok(uniforms.depth_test.into_pyobject(py)?.to_owned().into_any()),
        "use_winding_fill" => Ok(uniforms
            .use_winding_fill
            .into_pyobject(py)?
            .to_owned()
            .into_any()),
        _ => Err(PyKeyError::new_err(name.to_owned())),
    }
}

fn set_uniform(uniforms: &mut Uniforms, name: &str, value: &Bound<'_, PyAny>) -> PyResult<()> {
    match name {
        "is_fixed_in_frame" => uniforms.is_fixed_in_frame = value.extract()?,
        "shading" => {
            let values: Vec<f64> = value.extract()?;
            uniforms.shading = values.try_into().map_err(|values: Vec<f64>| {
                PyValueError::new_err(format!(
                    "shading requires exactly 3 values, got {}",
                    values.len()
                ))
            })?;
        }
        "clip_planes" => {
            let values: Vec<Vec<f64>> = value.extract()?;
            if values.len() != 4 || values.iter().any(|plane| plane.len() != 4) {
                return Err(PyValueError::new_err(
                    "clip_planes requires exactly four four-value planes",
                ));
            }
            for (destination, source) in uniforms.clip_planes.iter_mut().zip(values) {
                destination.copy_from_slice(&source);
            }
        }
        "anti_alias_width" => uniforms.anti_alias_width = value.extract()?,
        "joint_type" => uniforms.joint_type = JointType::from_code(value.extract()?),
        "flat_stroke" => uniforms.flat_stroke = value.extract()?,
        "scale_stroke_with_zoom" => uniforms.scale_stroke_with_zoom = value.extract()?,
        "stroke_behind" => uniforms.stroke_behind = value.extract()?,
        "depth_test" => uniforms.depth_test = value.extract()?,
        "use_winding_fill" => uniforms.use_winding_fill = value.extract()?,
        _ => return Err(PyKeyError::new_err(name.to_owned())),
    }
    Ok(())
}

fn isolated_native_state<'py>(
    py: Python<'py>,
    stage: &Stage,
    mob: Mob,
) -> PyResult<Bound<'py, PyDict>> {
    if !stage.updater_ids(mob).is_empty() {
        return Err(PyValueError::new_err(
            "native updater callables cannot enter Python pickle state",
        ));
    }

    // Python's pickle memo owns the proxy graph: every child is reduced once
    // and aliases are rebuilt by pickle. Isolate this one native entry so a
    // bound parent's arena family cannot be serialized again inside the
    // parent's payload and then duplicated when Python reconnects children.
    let mut isolated = Stage::new();
    let root = stage
        .copy_entry_into(mob, &mut isolated)
        .map_err(stage_error)?;
    isolated.add_to_scene(root).map_err(stage_error)?;

    let entry = isolated
        .get(root)
        .ok_or_else(|| StaleHandleError::new_err("isolated pickle root no longer resolves"))?;
    let fields: Vec<(String, usize)> = entry
        .buffer
        .schema()
        .fields()
        .iter()
        .map(|field| (field.name.clone(), field.width))
        .collect();
    let snapshot = isolated.snapshot_bytes().map_err(native_error)?;
    let state = PyDict::new(py);
    state.set_item("version", PORTAL_PICKLE_STATE_VERSION)?;
    state.set_item("snapshot", PyBytes::new(py, &snapshot))?;
    // Detached become/restore uses this cheap schema identity without
    // decoding either operand. The authoritative state remains the FMNA
    // snapshot above.
    state.set_item("fields", fields)?;
    Ok(state)
}

fn native_state<'py>(
    py: Python<'py>,
    proxy: &Bound<'py, BridgeMobject>,
) -> PyResult<Bound<'py, PyDict>> {
    let cell = proxy.borrow();
    if let (Some(engine), Some(mob)) = (&cell.engine, cell.mob) {
        let runtime = engine.borrow();
        return isolated_native_state(py, runtime.stage(), mob);
    }
    let nursery = cell
        .nursery
        .as_ref()
        .ok_or_else(|| StaleHandleError::new_err("mobject has no detached or bound state"))?;
    isolated_native_state(py, &nursery.stage, nursery.root)
}

fn restore_native_state(
    proxy: &Bound<'_, BridgeMobject>,
    state: &Bound<'_, PyDict>,
) -> PyResult<()> {
    {
        let cell = proxy.borrow();
        if cell.engine.is_some() || cell.mob.is_some() {
            return Err(PyRuntimeError::new_err(
                "cannot restore detached pickle state over a bound mobject",
            ));
        }
    }
    let version: u8 = state
        .get_item("version")?
        .ok_or_else(|| PyKeyError::new_err("version"))?
        .extract()?;
    if version != PORTAL_PICKLE_STATE_VERSION {
        return Err(PyValueError::new_err(format!(
            "unsupported Python portal pickle-state version {version}"
        )));
    }
    let declared_fields: Vec<(String, usize)> = state
        .get_item("fields")?
        .ok_or_else(|| PyKeyError::new_err("fields"))?
        .extract()?;
    let snapshot = state
        .get_item("snapshot")?
        .ok_or_else(|| PyKeyError::new_err("snapshot"))?;
    let snapshot = snapshot
        .cast::<PyBytes>()
        .map_err(|_| PyTypeError::new_err("pickle-state snapshot must be bytes"))?;

    // Durable decode validates container checksum/schema/version, arena
    // invariants, the 256 MiB encoded cap, and the aggregate decoded-
    // allocation budget before this proxy is touched. Decode and restore a
    // private candidate nursery first so every refusal is atomic.
    let mut stage = Stage::new();
    let decoded = Snapshot::from_bytes(snapshot.as_bytes(), &stage).map_err(native_error)?;
    if !decoded.updaters.entries.is_empty() {
        return Err(PyValueError::new_err(
            "Python pickle state cannot restore native updater identities without callables",
        ));
    }
    stage.restore(&decoded.snapshot);
    let [root] = stage.roots() else {
        return Err(PyValueError::new_err(
            "Python pickle state must contain exactly one native root",
        ));
    };
    let root = *root;
    if stage.family(root).len() != 1 {
        return Err(PyValueError::new_err(
            "Python pickle state native root must not contain a family graph",
        ));
    }
    let actual_fields: Vec<(String, usize)> = stage
        .get(root)
        .expect("validated snapshot root is live")
        .buffer
        .schema()
        .fields()
        .iter()
        .map(|field| (field.name.clone(), field.width))
        .collect();
    if declared_fields != actual_fields {
        return Err(PyValueError::new_err(
            "Python pickle-state field summary does not match its native snapshot",
        ));
    }
    let nursery = Nursery { stage, root };

    let mut cell = proxy.borrow_mut();
    cell.nursery = Some(nursery);
    cell.mob = None;
    cell.engine = None;
    cell.initialized = true;
    Ok(())
}

fn numpy_array<'py>(
    py: Python<'py>,
    proxy: &Bound<'py, BridgeMobject>,
    writable: bool,
) -> PyResult<Bound<'py, PyAny>> {
    // Every `.data` read builds one view (fm-ztjj: ~4 us each, 8 per
    // GlowDot). The NumPy module and ndarray type are resolved once.
    static NDARRAY: pyo3::sync::PyOnceLock<Py<pyo3::types::PyType>> = pyo3::sync::PyOnceLock::new();
    let view = with_buffer(proxy, |buffer| buffer.export_view(writable))?;
    let len = view.len();
    let stride_bytes = view
        .schema()
        .stride()
        .checked_mul(std::mem::size_of::<f32>())
        .ok_or_else(|| PyOverflowError::new_err("NumPy stride overflows usize"))?;
    let numpy = numpy_module(py).map_err(|error| {
        PyImportError::new_err(format!(
            "NumPy is required for the live `data` view: {error}"
        ))
    })?;
    let dtype = record_dtype(py, numpy, view.schema(), stride_bytes)?;
    let owner = Py::new(py, PyRecordView { view })?;
    let ndarray = NDARRAY.import(py, "numpy", "ndarray")?;
    // ndarray(shape, dtype, buffer, offset, strides), positionally.
    ndarray.call1(((len,), dtype, owner, 0, (stride_bytes,)))
}

/// The NumPy dtype of each RecordBuffer layout seen so far, keyed by its
/// record stride and (field name, lane count) list. Building the structured
/// dtype was about half of every `.data` access (fm-5wq.31); dtypes are
/// immutable, so one per layout is shared by every view of it. A static is
/// never dropped, so no reference is released after interpreter shutdown.
type DtypeKey = (usize, Vec<(String, usize)>);
/// (layout, dtype, the `names` tuple the dtype was built with).
type DtypeEntry = (DtypeKey, Py<PyAny>, Py<PyAny>);
static RECORD_DTYPES: Mutex<Vec<DtypeEntry>> = Mutex::new(Vec::new());

fn record_dtype<'py>(
    py: Python<'py>,
    numpy: &Bound<'py, PyModule>,
    schema: &RecordSchema,
    stride_bytes: usize,
) -> PyResult<Bound<'py, PyAny>> {
    let same = |(stride, fields): &DtypeKey| {
        *stride == stride_bytes
            && fields.len() == schema.fields().len()
            && fields
                .iter()
                .zip(schema.fields())
                .all(|((name, width), field)| *name == field.name && *width == field.width)
    };
    let cached = RECORD_DTYPES
        .lock()
        .map_err(|_| PyRuntimeError::new_err("the record dtype cache was poisoned"))?
        .iter()
        .find(|(key, _, _)| same(key))
        .map(|(_, dtype, names)| (dtype.clone_ref(py), names.clone_ref(py)));
    if let Some((dtype, built_names)) = cached {
        let dtype = dtype.into_bound(py);
        // `dtype.names` is the one assignable dtype property: a view whose
        // fields an author renamed must not rename every later view. NumPy
        // returns the same tuple until `names` is reassigned, so the tuple
        // the dtype was built with, by identity, settles the common case.
        let names = dtype.getattr(pyo3::intern!(py, "names"))?;
        if names.is(&built_names) {
            return Ok(dtype);
        }
        let kept = names.len()? == schema.fields().len()
            && names
                .try_iter()?
                .zip(schema.fields())
                .map(|(name, field)| name.and_then(|name| name.eq(field.name.as_str())))
                .collect::<PyResult<Vec<bool>>>()?
                .into_iter()
                .all(|equal| equal);
        if kept {
            return Ok(dtype);
        }
    }
    // Built outside the lock: numpy.dtype runs Python code.
    let descriptors = PyList::empty(py);
    for field in schema.fields() {
        descriptors.append((field.name.as_str(), "=f4", (field.width,)))?;
    }
    let dtype = numpy.getattr("dtype")?.call1((descriptors,))?;
    if dtype.getattr("itemsize")?.extract::<usize>()? != stride_bytes {
        return Err(PyRuntimeError::new_err(
            "NumPy packed the all-f32 RecordBuffer dtype at an unexpected itemsize",
        ));
    }
    let names = dtype.getattr(pyo3::intern!(py, "names"))?.unbind();
    let mut cache = RECORD_DTYPES
        .lock()
        .map_err(|_| PyRuntimeError::new_err("the record dtype cache was poisoned"))?;
    match cache.iter_mut().find(|(key, _, _)| same(key)) {
        Some((_, entry, entry_names)) => {
            *entry = dtype.clone().unbind();
            *entry_names = names;
        }
        None => {
            let fields = schema
                .fields()
                .iter()
                .map(|field| (field.name.clone(), field.width))
                .collect();
            cache.push(((stride_bytes, fields), dtype.clone().unbind(), names));
        }
    }
    Ok(dtype)
}

fn flat_records(proxy: &Bound<'_, BridgeMobject>) -> PyResult<(RecordSchema, usize, Vec<f32>)> {
    with_buffer_ref(proxy, |buffer| {
        let schema = buffer.schema().clone();
        let mut records = Vec::with_capacity(buffer.len() * schema.stride());
        for index in 0..buffer.len() {
            for field in schema.fields() {
                records.extend(
                    buffer
                        .read(index, &field.name)
                        .expect("iterated schema field exists"),
                );
            }
        }
        (schema, buffer.len(), records)
    })
}

fn location_compatible(
    left: &Bound<'_, BridgeMobject>,
    right: &Bound<'_, BridgeMobject>,
) -> PyResult<()> {
    let left_engine = left.borrow().engine.as_ref().map(Rc::clone);
    let right_engine = right.borrow().engine.as_ref().map(Rc::clone);
    match (left_engine, right_engine) {
        (Some(left), Some(right)) if same_engine(&left, &right) => Ok(()),
        (None, None) => Ok(()),
        _ => Err(ForeignStageError::new_err(
            "interpolation operands must all be detached or bound to one Scene",
        )),
    }
}

fn new_bound_proxy<'py>(
    py: Python<'py>,
    scene: &Bound<'py, PyScene>,
    old_proxy: &Bound<'py, PyAny>,
    engine: &Engine,
    mob: Mob,
) -> PyResult<Bound<'py, PyAny>> {
    let class = old_proxy.get_type();
    let proxy = class.call_method1("__new__", (&class,))?;
    {
        let bridge = proxy.cast::<BridgeMobject>()?;
        let mut cell = bridge.borrow_mut();
        cell.nursery = None;
        cell.engine = Some(Rc::clone(engine));
        cell.mob = Some(mob);
        cell.initialized = true;
    }
    engine
        .borrow_mut()
        .stage_mut()
        .pin(mob)
        .map_err(stage_error)?;
    register_proxy(py, scene, mob, &proxy)?;
    proxy.setattr("_scene", scene)?;
    Ok(proxy)
}

#[pymethods]
impl BridgeMobject {
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn py_new(_args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>) -> Self {
        Self {
            nursery: Some(Nursery::new(Mobject::new())),
            engine: None,
            mob: None,
            initialized: false,
        }
    }

    #[classattr]
    fn data_dtype() -> Vec<(&'static str, usize)> {
        vec![("point", 3), ("rgba", 4)]
    }

    #[classattr]
    fn aligned_data_keys() -> Vec<&'static str> {
        vec!["point"]
    }

    #[classattr]
    fn pointlike_data_keys() -> Vec<&'static str> {
        vec!["point"]
    }

    /// Allocate detached RecordBuffer state from the subclass's dtype, then
    /// drive the three initialization hooks through normal Python MRO.
    fn _engine_init(slf: &Bound<'_, Self>) -> PyResult<()> {
        if slf.borrow().initialized {
            return Err(PyRuntimeError::new_err(
                "Mobject engine initialization may run only once",
            ));
        }
        let schema = parse_schema(slf)?;
        {
            let mut cell = slf.borrow_mut();
            cell.nursery = Some(Nursery::new(Mobject::from_buffer(
                RecordBuffer::new(schema, 0).map_err(record_error_to_py)?,
            )));
            cell.initialized = true;
        }
        // No engine or proxy borrow is live across these calls. Each hook
        // dispatches through the fm-zoi method-resolution cache (one native
        // →Python method_dispatch crossing per hook).
        crossing::record(CrossingClass::MethodDispatch);
        method_cache::call_cached0(slf.as_any(), "init_data")?;
        crossing::record(CrossingClass::MethodDispatch);
        method_cache::call_cached0(slf.as_any(), "init_points")?;
        crossing::record(CrossingClass::MethodDispatch);
        method_cache::call_cached0(slf.as_any(), "init_uniforms")?;
        Ok(())
    }

    fn init_data(_slf: &Bound<'_, Self>) {}

    fn init_points(_slf: &Bound<'_, Self>) {}

    fn init_uniforms(_slf: &Bound<'_, Self>) {}

    fn resize(slf: &Bound<'_, Self>, len: usize) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        // The sizing proof lives in the fallible resize itself (fm-vek.2);
        // a refusal surfaces to Python as a typed OverflowError, and the
        // buffer (plus every exported NumPy view) is left untouched.
        with_buffer(slf, |buffer| buffer.resize(len))?.map_err(record_error_to_py)
    }

    /// Reference `resize_preserving_order`, exposed only to the Python skin's
    /// semantic methods (not as a public manim API addition).
    fn _resize_preserving_order(slf: &Bound<'_, Self>, len: usize) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        with_buffer(slf, |buffer| buffer.resize_preserving_order(len))?.map_err(record_error_to_py)
    }

    fn n_records(slf: &Bound<'_, Self>) -> PyResult<usize> {
        crossing::record(CrossingClass::Other);
        with_buffer_ref(slf, RecordBuffer::len)
    }

    fn revision(slf: &Bound<'_, Self>) -> PyResult<u64> {
        crossing::record(CrossingClass::Other);
        with_buffer_ref(slf, RecordBuffer::revision)
    }

    fn field_names(slf: &Bound<'_, Self>) -> PyResult<Vec<String>> {
        crossing::record(CrossingClass::Other);
        with_buffer_ref(slf, |buffer| {
            buffer
                .schema()
                .fields()
                .iter()
                .map(|field| field.name.clone())
                .collect()
        })
    }

    fn get_field(slf: &Bound<'_, Self>, field: &str, index: usize) -> PyResult<Vec<f32>> {
        crossing::record(CrossingClass::Other);
        with_buffer_ref(slf, |buffer| buffer.read(index, field))?
            .ok_or_else(|| PyKeyError::new_err(format!("no `{field}` record at index {index}")))
    }

    fn set_field(
        slf: &Bound<'_, Self>,
        field: &str,
        index: usize,
        values: Vec<f32>,
    ) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        if with_buffer(slf, |buffer| buffer.write(index, field, &values))? {
            Ok(())
        } else {
            Err(PyKeyError::new_err(format!(
                "no writable `{field}` record at index {index} with {} lanes",
                values.len()
            )))
        }
    }

    /// Owned f64 rows for one RecordBuffer field. Arbitrary Python point-map
    /// callables operate on NumPy-owned arrays, then commit through the
    /// engine-mediated write path instead of leaving a zero-copy writer alive
    /// across bounding-box cache installation.
    fn _field_rows(slf: &Bound<'_, Self>, field: &str) -> PyResult<Vec<Vec<f64>>> {
        crossing::record(CrossingClass::Other);
        with_buffer_ref(slf, |buffer| {
            if !buffer
                .schema()
                .fields()
                .iter()
                .any(|candidate| candidate.name == field)
            {
                return None;
            }
            Some(
                (0..buffer.len())
                    .map(|index| {
                        buffer
                            .read(index, field)
                            .expect("validated schema field exists")
                            .into_iter()
                            .map(f64::from)
                            .collect()
                    })
                    .collect(),
            )
        })?
        .ok_or_else(|| PyKeyError::new_err(format!("no `{field}` record field")))
    }

    /// Atomically validate and engine-write one complete field column.
    fn _set_field_rows(slf: &Bound<'_, Self>, field: &str, rows: Vec<Vec<f64>>) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        with_buffer(slf, |buffer| {
            let Some(field_index) = buffer
                .schema()
                .fields()
                .iter()
                .position(|candidate| candidate.name == field)
            else {
                return Err(PyKeyError::new_err(format!("no `{field}` record field")));
            };
            let width = buffer.schema().fields()[field_index].width;
            if rows.len() != buffer.len() || rows.iter().any(|row| row.len() != width) {
                return Err(PyValueError::new_err(format!(
                    "field `{field}` expects {} rows of width {width}",
                    buffer.len()
                )));
            }
            for (index, row) in rows.iter().enumerate() {
                let values = row.iter().map(|value| *value as f32).collect::<Vec<_>>();
                let wrote = buffer.write(index, field, &values);
                debug_assert!(wrote, "validated field row must be writable");
            }
            Ok(())
        })?
    }

    /// The scalar case of `set_rgba_array_by_color` on this record's own
    /// rows: `data[field][:, :3] = rgb` and/or `data[field][:, 3] = alpha`,
    /// without materializing a record view per family member. Returns false,
    /// writing nothing, unless `field` is a four-lane column, so the caller
    /// keeps the view path and its exact errors for anything else.
    #[pyo3(signature = (field, rgb, alpha))]
    fn _fill_rgba_lanes(
        slf: &Bound<'_, Self>,
        field: &str,
        rgb: Option<[f64; 3]>,
        alpha: Option<f64>,
    ) -> PyResult<bool> {
        crossing::record(CrossingClass::FieldWrite);
        with_buffer(slf, |buffer| {
            if buffer.schema().field_width(field) != Some(4) {
                return false;
            }
            let mut column = buffer.read_column(field).expect("four-lane field exists");
            for lanes in column.as_chunks_mut::<4>().0 {
                if let Some(rgb) = rgb {
                    #[allow(clippy::cast_possible_truncation)]
                    for (lane, value) in lanes.iter_mut().zip(rgb) {
                        *lane = value as f32;
                    }
                }
                if let Some(alpha) = alpha {
                    #[allow(clippy::cast_possible_truncation)]
                    {
                        lanes[3] = alpha as f32;
                    }
                }
            }
            buffer.write_range(field, 0, &column)
        })
    }

    #[pyo3(signature = (writable = true))]
    fn _data_array<'py>(slf: &Bound<'py, Self>, writable: bool) -> PyResult<Bound<'py, PyAny>> {
        crossing::record(CrossingClass::Other);
        numpy_array(slf.py(), slf, writable)
    }

    #[staticmethod]
    fn _uniform_names() -> Vec<&'static str> {
        UNIFORM_NAMES.to_vec()
    }

    fn _get_uniform<'py>(slf: &Bound<'py, Self>, name: &str) -> PyResult<Bound<'py, PyAny>> {
        crossing::record(CrossingClass::Other);
        uniform_value(slf.py(), uniforms_snapshot(slf)?, name)
    }

    fn _set_uniform(slf: &Bound<'_, Self>, name: &str, value: &Bound<'_, PyAny>) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        with_uniforms(slf, |uniforms| set_uniform(uniforms, name, value))
    }

    // ------------------------------------------------ positional primitives
    //
    // fm-d3gt: the engine seam under the bootstrap's Reference-signature
    // positional surface. Each primitive routes to the ONE Stage
    // implementation via `with_stage`; pivots arrive pre-resolved from the
    // Python layer (which reads them off the same Stage bounding box), so
    // family distribution in the detached state stays exact.

    /// Whether this proxy is bound to a Scene's stage. When false its Stage
    /// is the private nursery, whose family is exactly one root — the
    /// bootstrap then distributes transforms over the Python family list.
    fn _is_bound(slf: &Bound<'_, Self>) -> bool {
        let cell = slf.borrow();
        cell.engine.is_some() && cell.mob.is_some()
    }

    /// `(min, mid, max)` rows of the Stage-visible family bounding box.
    fn _get_bbox(slf: &Bound<'_, Self>) -> PyResult<BoundingBoxRows> {
        crossing::record(CrossingClass::Other);
        with_stage(slf, |stage, mob| {
            let bbox = stage.get_bounding_box(mob);
            (bbox.min, bbox.mid, bbox.max)
        })
    }

    /// A still-current bounding box installed by the Reference compatibility
    /// path, rather than lazily materialized from point extrema.
    fn _get_installed_bbox(slf: &Bound<'_, Self>) -> PyResult<Option<BoundingBoxRows>> {
        crossing::record(CrossingClass::Other);
        with_stage(slf, |stage, mob| {
            stage
                .installed_bounding_box_cache(mob)
                .map(|bbox| (bbox.min, bbox.mid, bbox.max))
        })
    }

    /// Whether this entry itself has point records (Reference `has_points`,
    /// not recursing into the family).
    fn _has_points(slf: &Bound<'_, Self>) -> PyResult<bool> {
        crossing::record(CrossingClass::Other);
        with_stage(slf, |stage, mob| {
            stage
                .get_points(mob)
                .is_some_and(|points| !points.is_empty())
        })
    }

    /// `Stage::shift`: translate the Stage-visible family.
    fn _shift(slf: &Bound<'_, Self>, vector: [f64; 3]) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| {
            stage.shift(mob, vector);
        })
    }

    /// `Stage::scale_about` with an explicit pre-resolved pivot.
    fn _scale_about(slf: &Bound<'_, Self>, factor: f64, about_point: [f64; 3]) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| {
            stage.scale_about(mob, factor, Some(about_point), None);
        })
    }

    /// `Stage::stretch_about` with an explicit pre-resolved pivot.
    fn _stretch_about(
        slf: &Bound<'_, Self>,
        factor: f64,
        dim: usize,
        about_point: [f64; 3],
    ) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        if dim > 2 {
            return Err(PyValueError::new_err("stretch dim must be 0, 1, or 2"));
        }
        with_stage(slf, |stage, mob| {
            stage.stretch_about(mob, factor, dim, Some(about_point), None);
        })
    }

    /// `Stage::rotate` with an explicit pre-resolved pivot.
    fn _rotate_about(
        slf: &Bound<'_, Self>,
        angle: f64,
        axis: [f64; 3],
        about_point: [f64; 3],
    ) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| {
            stage.rotate(mob, angle, axis, Some(about_point), None);
        })
    }

    /// `Stage::to_edge` (`align_on_border`): the single-target engine path
    /// used when the proxy is bound; the bootstrap's detached branch
    /// decomposes over the frame radii instead.
    fn _to_edge(slf: &Bound<'_, Self>, direction: [f64; 3], buff: f64) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| {
            stage.to_edge(mob, direction, buff);
        })
    }

    /// `Stage::refresh_bounding_box` without the upward walk: return this
    /// entry (and, with `recurse_down`, its Stage family) to the origin when
    /// it has no point records (fm-fc53). The bootstrap walks parents over
    /// the Python graph, which also spans detached owners.
    fn _refresh_point_free_anchors(slf: &Bound<'_, Self>, recurse_down: bool) -> PyResult<()> {
        crossing::record(CrossingClass::Other);
        with_stage(slf, |stage, mob| {
            stage.refresh_bounding_box(mob, recurse_down, false);
        })
    }

    /// `Stage::set_point_free_anchor`: place a point-less entry's degenerate
    /// box at `point` (the Reference's `works_on_bounding_box=True` map of
    /// its cached box). A no-op returning `False` for an entry with records.
    fn _set_point_free_anchor(slf: &Bound<'_, Self>, point: [f64; 3]) -> PyResult<bool> {
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| stage.set_point_free_anchor(mob, point))?.map_err(stage_error)
    }

    /// The frame half-extents `(FRAME_X_RADIUS, FRAME_Y_RADIUS)` the border
    /// alignment surface is defined against.
    #[staticmethod]
    fn _frame_radii() -> (f64, f64) {
        (
            fmn_core::constants::FRAME_X_RADIUS,
            fmn_core::constants::FRAME_Y_RADIUS,
        )
    }

    /// fmn-core's one color model (D4): parse `#RRGGBB`/`#RGB` into sRGB
    /// components in `[0, 1]`. Anything else is a precise refusal — the
    /// bootstrap never hand-rolls color arithmetic.
    #[staticmethod]
    fn _hex_to_rgb(value: &str) -> PyResult<(f64, f64, f64)> {
        fmn_core::color::Srgb::from_hex(value)
            .map(|color| (color.r, color.g, color.b))
            .map_err(|error| PyValueError::new_err(format!("invalid color {value:?}: {error}")))
    }

    /// Format sRGB components as the Reference portal's uppercase
    /// `colour.rgb2hex(..., force_long=True)`. The tiny subtraction is
    /// colour 0.1.5's declared half-boundary rule (`FLOAT_ERROR = 5e-7`),
    /// which intentionally maps an exact 0.5 component to `0x7F`.
    #[staticmethod]
    fn _rgb_to_hex(rgb: (f64, f64, f64)) -> String {
        let quantize = |component: f64| (component * 255.0 + 0.5 - 5e-7) as u8;
        let [r, g, b] = [quantize(rgb.0), quantize(rgb.1), quantize(rgb.2)];
        format!("#{r:02X}{g:02X}{b:02X}")
    }

    /// Route both Reference `color_gradient` branches through fmn-core's
    /// single color authority. Python performs only its normal public color
    /// coercion; all interpolation and sample-position semantics live here.
    #[staticmethod]
    fn _color_gradient(
        colors: Vec<[f64; 3]>,
        length: usize,
        interp_by_hsl: bool,
    ) -> PyResult<Vec<[f64; 3]>> {
        if length > 0 && colors.len() < 2 {
            return Err(PyValueError::new_err(
                "color_gradient needs at least two reference colors",
            ));
        }
        let colors = colors
            .into_iter()
            .map(|[r, g, b]| fmn_core::color::Srgb { r, g, b })
            .collect::<Vec<_>>();
        let gradient = if interp_by_hsl {
            fmn_core::color::color_gradient_by_hsl(&colors, length)
        } else {
            fmn_core::color::color_gradient(&colors, length)
        };
        Ok(gradient
            .into_iter()
            .map(|color| [color.r, color.g, color.b])
            .collect())
    }

    /// Public `interpolate_color` delegates to fmn-core's single declared
    /// color model, including the Reference's opt-in HSL branch.
    #[staticmethod]
    fn _interpolate_color(
        color1: [f64; 3],
        color2: [f64; 3],
        alpha: f64,
        interp_by_hsl: bool,
    ) -> [f64; 3] {
        let color1 = fmn_core::color::Srgb {
            r: color1[0],
            g: color1[1],
            b: color1[2],
        };
        let color2 = fmn_core::color::Srgb {
            r: color2[0],
            g: color2[1],
            b: color2[2],
        };
        let color = if interp_by_hsl {
            fmn_core::color::interpolate_color_by_hsl(color1, color2, alpha)
        } else {
            fmn_core::color::interpolate_color(color1, color2, alpha)
        };
        [color.r, color.g, color.b]
    }

    /// Public `average_color` is fmn-core's Reference-compatible RMS color
    /// average, not a second portal-side color model.
    #[staticmethod]
    fn _average_color(colors: Vec<[f64; 3]>) -> [f64; 3] {
        let colors = colors
            .into_iter()
            .map(|[r, g, b]| fmn_core::color::Srgb { r, g, b })
            .collect::<Vec<_>>();
        let color = fmn_core::color::average_color(&colors);
        [color.r, color.g, color.b]
    }

    /// Chisel owns the exact clamped integer interpolation used throughout
    /// the animation and path surfaces.
    #[staticmethod]
    fn _integer_interpolate(start: i64, end: i64, alpha: f64) -> PyResult<(i64, f64)> {
        fmn_library::integer_interpolate(start, end, alpha)
            .map_err(|error| PyValueError::new_err(error.to_string()))
    }

    /// Preflight the Python-proxy half of `Mobject.add_n_more_submobjects`
    /// against Marionette's one family-alignment resource ceiling.
    #[staticmethod]
    fn _aligned_submobject_target(current: usize, additional: usize) -> PyResult<usize> {
        let requested = current.checked_add(additional).ok_or_else(|| {
            stage_error(fmn_mobject::StageError::SubmobjectBudgetExceeded {
                requested: usize::MAX,
                max: fmn_mobject::MAX_ALIGNED_SUBMOBJECTS,
            })
        })?;
        if requested > fmn_mobject::MAX_ALIGNED_SUBMOBJECTS {
            return Err(stage_error(
                fmn_mobject::StageError::SubmobjectBudgetExceeded {
                    requested,
                    max: fmn_mobject::MAX_ALIGNED_SUBMOBJECTS,
                },
            ));
        }
        Ok(requested)
    }

    /// `Mobject.become` over `Stage::become_mobject`: per-member data,
    /// uniform, and placement assignment across zipped families after the
    /// Python proxy graph has run Reference `align_family`. Schema drift or
    /// an unsynchronized proxy/native graph remains a precise refusal.
    #[pyo3(signature = (other, match_updaters = false))]
    fn _become(
        slf: &Bound<'_, Self>,
        other: &Bound<'_, BridgeMobject>,
        match_updaters: bool,
    ) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        let self_bound = {
            let cell = slf.borrow();
            cell.engine.as_ref().map(Rc::clone).zip(cell.mob)
        };
        if let Some((engine, mob)) = self_bound {
            let (other_engine, other_mob) = bound_parts(&other.borrow())?;
            if !same_engine(&engine, &other_engine) {
                return Err(ForeignStageError::new_err(
                    "become endpoints must belong to one Scene",
                ));
            }
            return engine
                .borrow_mut()
                .stage_mut()
                .become_mobject(mob, other_mob, match_updaters)
                .map_err(stage_error);
        }
        // Detached self: bring a copy of the source family into the
        // nursery, become it, and drop the temp — the Reference's
        // become-is-a-data-copy semantics without any scene requirement.
        let other_location = {
            let cell = other.borrow();
            (cell.engine.as_ref().map(Rc::clone), cell.mob)
        };
        let mut self_cell = slf.borrow_mut();
        let root = self_cell
            .nursery
            .as_ref()
            .map(|nursery| nursery.root)
            .ok_or_else(|| StaleHandleError::new_err("mobject has no detached or bound state"))?;
        let nursery = self_cell.nursery.as_mut().expect("checked above");
        let temp = match other_location {
            (Some(other_engine), Some(other_mob)) => {
                let scene = other_engine.borrow();
                // A childless nursery root keeps its family in Python, and
                // Python.become pairs those members itself, so the root takes
                // only the source root's entry. A Scene-owned source (adopted
                // because a receiver member is Scene-bound) otherwise brings
                // its whole Stage subtree and can never match the root's shape.
                let childless = nursery
                    .stage
                    .get(root)
                    .is_some_and(|entry| entry.submobjects().is_empty());
                if childless {
                    scene
                        .stage()
                        .copy_entry_into(other_mob, &mut nursery.stage)
                        .map_err(stage_error)?
                } else {
                    scene
                        .stage()
                        .copy_into(other_mob, &mut nursery.stage)
                        .map_err(stage_error)?
                }
            }
            _ => {
                let other_cell = other.borrow();
                let other_nursery = other_cell.nursery.as_ref().ok_or_else(|| {
                    StaleHandleError::new_err("become source has no detached or bound state")
                })?;
                other_nursery
                    .stage
                    .copy_into(other_nursery.root, &mut nursery.stage)
                    .map_err(stage_error)?
            }
        };
        let outcome = nursery
            .stage
            .become_mobject(root, temp, match_updaters)
            .map_err(stage_error);
        nursery.stage.delete(temp).map_err(stage_error)?;
        outcome
    }

    /// `TracingTail` construction: create the native tracer — a bound
    /// stage entry whose native dt-updater follows the traced mobject's
    /// center (fmn-library fields.rs) — and bind THIS proxy to it.
    #[pyo3(signature = (
        scene,
        traced,
        time_traced,
        time_per_anchor,
        stroke_color,
        stroke_width_taper,
        stroke_opacity_taper,
    ))]
    // The argument count mirrors the Reference TracingTail constructor
    // surface one-to-one; collapsing it would change the Python-visible API.
    #[allow(clippy::too_many_arguments)]
    fn _init_native_tracer(
        slf: &Bound<'_, Self>,
        scene: &Bound<'_, PyScene>,
        traced: &Bound<'_, BridgeMobject>,
        time_traced: f64,
        time_per_anchor: f64,
        stroke_color: Option<&Bound<'_, PyAny>>,
        stroke_width_taper: Vec<f64>,
        stroke_opacity_taper: Vec<f64>,
    ) -> PyResult<()> {
        if slf.borrow().engine.is_some() {
            return Err(PyRuntimeError::new_err(
                "a tracing tail initializes before scene entry",
            ));
        }
        let engine = Rc::clone(&scene.borrow().engine);
        let (traced_engine, traced_mob) = bound_parts(&traced.borrow())?;
        if !same_engine(&engine, &traced_engine) {
            return Err(ForeignStageError::new_err(
                "the traced mobject belongs to a different Scene",
            ));
        }
        let mut tail = fmn_library::TracingTail::new()
            .with_time_traced(time_traced)
            .with_time_per_anchor(time_per_anchor)
            .map_err(|error| PyValueError::new_err(error.to_string()))?
            .with_stroke_width_taper(stroke_width_taper)
            .with_stroke_opacity_taper(stroke_opacity_taper);
        if let Some(color) = stroke_color {
            tail = tail.with_stroke_color(srgb_from_py(color)?);
        }
        let mob = {
            let mut runtime = engine.borrow_mut();
            let mob = tail
                .add_to_stage(runtime.stage_mut(), traced_mob)
                .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
            runtime.stage_mut().pin(mob).map_err(stage_error)?;
            mob
        };
        {
            let mut cell = slf.borrow_mut();
            cell.nursery = None;
            cell.engine = Some(Rc::clone(&engine));
            cell.mob = Some(mob);
            cell.initialized = true;
        }
        register_proxy(slf.py(), scene, mob, slf.as_any())?;
        slf.as_any().setattr("_scene", scene)?;
        Ok(())
    }

    /// `Stage::save_state`: snapshot this entry's family for `Restore`.
    fn _save_state(slf: &Bound<'_, Self>) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| stage.save_state(mob))?
            .map(|_| ())
            .map_err(stage_error)
    }

    /// Recreate the Reference's `saved_state` pointer after a detached
    /// mobject and its previously-copied state have both entered a Scene.
    fn _link_saved_state(slf: &Bound<'_, Self>, saved: &Bound<'_, BridgeMobject>) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        let (engine, mob) = bound_parts(&slf.borrow())?;
        let (saved_engine, saved_mob) = bound_parts(&saved.borrow())?;
        if !same_engine(&engine, &saved_engine) {
            return Err(ForeignStageError::new_err(
                "saved state belongs to a different Scene",
            ));
        }
        engine
            .borrow_mut()
            .stage_mut()
            .link_saved_state(mob, saved_mob)
            .map_err(stage_error)
    }

    /// Reference `get_start`: this entry's own first world-space point.
    fn _get_start(slf: &Bound<'_, Self>) -> PyResult<[f64; 3]> {
        crossing::record(CrossingClass::Other);
        with_stage(slf, |stage, mob| stage.get_start(mob))?
            .ok_or_else(|| PyValueError::new_err("Cannot get points of Mobject with no points"))
    }

    /// Reference `get_end`: this entry's own last world-space point.
    fn _get_end(slf: &Bound<'_, Self>) -> PyResult<[f64; 3]> {
        crossing::record(CrossingClass::Other);
        with_stage(slf, |stage, mob| stage.get_end(mob))?
            .ok_or_else(|| PyValueError::new_err("Cannot get points of Mobject with no points"))
    }

    /// Materialize this entry's object-to-world placement into its point
    /// records before exposing manim's writable `get_points()` view.
    /// This changes representation only; geometry and placement together
    /// remain identical.
    fn _bake_placement(slf: &Bound<'_, Self>) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| stage.bake_placement(mob))?
            .map(|_| ())
            .map_err(stage_error)
    }

    /// Preserve the Reference's opt-in transformed bounding-box rows after a
    /// Python callable has written the pointlike records. Marionette keys the
    /// value to the current subtree signature, so all later mutations still
    /// invalidate it through the ordinary native revision path.
    fn _cache_bbox_rows(slf: &Bound<'_, Self>, rows: [[f64; 3]; 3]) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| {
            stage.install_bounding_box_cache(
                mob,
                fmn_mobject::BoundingBox {
                    min: rows[0],
                    mid: rows[1],
                    max: rows[2],
                },
            );
        })
    }

    /// Native family matrix application for the common linear transform
    /// branch. The bootstrap distributes this over detached nursery roots but
    /// calls it once for a scene-bound family.
    fn _apply_matrix(
        slf: &Bound<'_, Self>,
        matrix: [[f64; 3]; 3],
        about_point: Option<[f64; 3]>,
        about_edge: Option<[f64; 3]>,
    ) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| {
            stage.apply_points_function(
                mob,
                |point| {
                    [
                        matrix[0][0] * point[0] + matrix[0][1] * point[1] + matrix[0][2] * point[2],
                        matrix[1][0] * point[0] + matrix[1][1] * point[1] + matrix[1][2] * point[2],
                        matrix[2][0] * point[0] + matrix[2][1] * point[1] + matrix[2][2] * point[2],
                    ]
                },
                about_point,
                about_edge,
            );
        })
    }

    /// Chisel's shared-anchor encoding for a corner polyline, installed
    /// into the current Stage entry with the normal schema/revision rules.
    fn _set_points_as_corners(slf: &Bound<'_, Self>, anchors: &Bound<'_, PyAny>) -> PyResult<()> {
        let anchors = point_rows(anchors)?;
        crossing::record(CrossingClass::FieldWrite);
        let mut path = fmn_library::QuadPath::new();
        path.set_points_as_corners(&anchors).map_err(native_error)?;
        with_stage(slf, |stage, mob| stage.set_points(mob, path.points()))?.map_err(stage_error)
    }

    /// Build a complete shared-anchor run from separate anchors and handles.
    /// The caller commits the result through `VMobject.set_points`, retaining
    /// Python's exact structured-record resize semantics.
    fn _set_anchors_and_handles_points(
        &self,
        anchors: Vec<[f64; 3]>,
        handles: Vec<[f64; 3]>,
    ) -> PyResult<Vec<[f64; 3]>> {
        crossing::record(CrossingClass::Other);
        let mut path = fmn_library::QuadPath::new();
        path.set_anchors_and_handles(&anchors, &handles)
            .map_err(native_error)?;
        Ok(path.points().to_vec())
    }

    /// Chisel `start_new_path`, returned as append-only geometry rows.
    fn _start_new_path_points(slf: &Bound<'_, Self>, point: [f64; 3]) -> PyResult<Vec<[f64; 3]>> {
        crossing::record(CrossingClass::Other);
        quad_path_tail(slf, |path| {
            path.start_new_path(point);
            Ok(())
        })
    }

    /// Construct a complete cubic-start operation before committing any row.
    /// BN-13's one error-bounded cubic-to-quadratic converter owns the emitted
    /// topology; the Reference's fixed heuristic approximation is retired.
    fn _add_cubic_bezier_curve_points(
        slf: &Bound<'_, Self>,
        anchor1: [f64; 3],
        handle1: [f64; 3],
        handle2: [f64; 3],
        anchor2: [f64; 3],
    ) -> PyResult<Vec<[f64; 3]>> {
        crossing::record(CrossingClass::Other);
        quad_path_tail(slf, |path| {
            path.start_new_path(anchor1);
            path.add_cubic_bezier_curve_to(handle1, handle2, anchor2)
                .map(|_| ())
                .map_err(native_error)
        })
    }

    /// BN-13 cubic reduction appended to the current shared-anchor path.
    fn _add_cubic_bezier_curve_to_points(
        slf: &Bound<'_, Self>,
        handle1: [f64; 3],
        handle2: [f64; 3],
        anchor: [f64; 3],
    ) -> PyResult<Vec<[f64; 3]>> {
        crossing::record(CrossingClass::Other);
        quad_path_tail(slf, |path| {
            path.add_cubic_bezier_curve_to(handle1, handle2, anchor)
                .map(|_| ())
                .map_err(native_error)
        })
    }

    /// Append one native quadratic segment, including null-curve policy and
    /// break-marker avoidance.
    fn _add_quadratic_bezier_curve_to_points(
        slf: &Bound<'_, Self>,
        handle: [f64; 3],
        anchor: [f64; 3],
        allow_null_curve: bool,
    ) -> PyResult<Vec<[f64; 3]>> {
        crossing::record(CrossingClass::Other);
        quad_path_tail(slf, |path| {
            path.add_quadratic_bezier_curve_to(handle, anchor, allow_null_curve)
                .map(|_| ())
                .map_err(native_error)
        })
    }

    /// Append one native line, honoring both `long_lines` and null-line
    /// policy from the live Python VMobject.
    fn _add_line_to_points(
        slf: &Bound<'_, Self>,
        point: [f64; 3],
        allow_null_line: bool,
    ) -> PyResult<Vec<[f64; 3]>> {
        crossing::record(CrossingClass::Other);
        quad_path_tail(slf, |path| {
            path.add_line_to(point, allow_null_line)
                .map(|_| ())
                .map_err(native_error)
        })
    }

    /// Continue the current native path with a reflected quadratic handle.
    fn _add_smooth_curve_to_points(
        slf: &Bound<'_, Self>,
        point: [f64; 3],
    ) -> PyResult<Vec<[f64; 3]>> {
        crossing::record(CrossingClass::Other);
        quad_path_tail(slf, |path| {
            path.add_smooth_curve_to(point)
                .map(|_| ())
                .map_err(native_error)
        })
    }

    /// Continue the current native path with BN-13 cubic reduction.
    fn _add_smooth_cubic_curve_to_points(
        slf: &Bound<'_, Self>,
        handle: [f64; 3],
        point: [f64; 3],
    ) -> PyResult<Vec<[f64; 3]>> {
        crossing::record(CrossingClass::Other);
        quad_path_tail(slf, |path| {
            path.add_smooth_cubic_curve_to(handle, point)
                .map(|_| ())
                .map_err(native_error)
        })
    }

    /// Append a native endpoint arc under the public threshold and explicit
    /// component policy. BN-09 owns the default density.
    fn _add_arc_to_points(
        slf: &Bound<'_, Self>,
        point: [f64; 3],
        angle: f64,
        n_components: Option<usize>,
        threshold: f64,
    ) -> PyResult<Vec<[f64; 3]>> {
        crossing::record(CrossingClass::Other);
        if !threshold.is_finite() || threshold < 0.0 {
            return Err(PyValueError::new_err(
                "arc threshold must be a finite non-negative number",
            ));
        }
        quad_path_tail(slf, |path| {
            path.add_arc_to_with_threshold(point, angle, n_components, threshold)
                .map(|_| ())
                .map_err(native_error)
        })
    }

    /// Append a sequence of native straight quadratic segments atomically.
    fn _add_points_as_corners_points<'py>(
        slf: &Bound<'py, Self>,
        points: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let points = point_rows(points)?;
        crossing::record(CrossingClass::Other);
        let rows = quad_path_tail(slf, |path| {
            path.add_points_as_corners(&points)
                .map(|_| ())
                .map_err(native_error)
        })?;
        point_array(slf.py(), &rows)
    }

    /// Append a complete validated native subpath atomically.
    fn _add_subpath_points<'py>(
        slf: &Bound<'py, Self>,
        points: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let points = point_rows(points)?;
        crossing::record(CrossingClass::Other);
        let rows = quad_path_tail(slf, |path| {
            path.add_subpath(&points).map(|_| ()).map_err(native_error)
        })?;
        point_array(slf.py(), &rows)
    }

    /// Close only the current native subpath and return its appended rows.
    fn _close_path_points(slf: &Bound<'_, Self>, smooth: bool) -> PyResult<Vec<[f64; 3]>> {
        crossing::record(CrossingClass::Other);
        quad_path_tail(slf, |path| {
            path.close_path(smooth).map(|_| ()).map_err(native_error)
        })
    }

    fn _has_new_path_started(slf: &Bound<'_, Self>) -> PyResult<bool> {
        crossing::record(CrossingClass::Other);
        Ok(configured_quad_path(slf)?.has_new_path_started())
    }

    fn _is_path_closed(slf: &Bound<'_, Self>) -> PyResult<bool> {
        crossing::record(CrossingClass::Other);
        Ok(configured_quad_path(slf)?.is_closed())
    }

    fn _path_area_vector(slf: &Bound<'_, Self>) -> PyResult<[f64; 3]> {
        crossing::record(CrossingClass::Other);
        Ok(configured_quad_path(slf)?.area_vector())
    }

    fn _path_unit_normal(slf: &Bound<'_, Self>) -> PyResult<[f64; 3]> {
        crossing::record(CrossingClass::Other);
        Ok(configured_quad_path(slf)?.unit_normal())
    }

    fn _consider_path_points_equal(
        slf: &Bound<'_, Self>,
        p0: [f64; 3],
        p1: [f64; 3],
    ) -> PyResult<bool> {
        crossing::record(CrossingClass::Other);
        Ok(configured_quad_path(slf)?.consider_points_equal(p0, p1))
    }

    fn _is_path_smooth(slf: &Bound<'_, Self>, angle_tol: f64) -> PyResult<bool> {
        crossing::record(CrossingClass::Other);
        Ok(configured_quad_path(slf)?.is_smooth(angle_tol))
    }

    /// Rebuild this VMobject's handles through Chisel's one anchor-mode
    /// implementation, then commit through the Stage's RecordBuffer path.
    fn _change_anchor_mode(slf: &Bound<'_, Self>, mode: &str) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        let mode = match mode {
            "jagged" => fmn_library::AnchorMode::Jagged,
            "approx_smooth" => fmn_library::AnchorMode::ApproxSmooth,
            "true_smooth" => fmn_library::AnchorMode::TrueSmooth,
            other => {
                return Err(PyValueError::new_err(format!(
                    "unknown VMobject anchor mode {other:?}; expected jagged, approx_smooth, or true_smooth"
                )));
            }
        };
        let mut path = configured_quad_path(slf)?;
        path.change_anchor_mode(mode).map_err(native_error)?;
        with_stage(slf, |stage, mob| stage.set_points(mob, path.points()))?.map_err(stage_error)
    }

    /// Chisel's bounded longest-curve insertion over an arbitrary valid
    /// shared-anchor point run.  This is intentionally a pure helper: the
    /// Python surface owns Reference family recursion and commits the result
    /// through `VMobject.set_points`, so RecordBuffer style lanes and view
    /// generations follow the same path as every other portal point edit.
    fn _insert_n_curves_to_point_list(
        &self,
        n: usize,
        points: Vec<[f64; 3]>,
        tolerance: f64,
    ) -> PyResult<Vec<[f64; 3]>> {
        crossing::record(CrossingClass::Other);
        fmn_library::QuadPath::insert_n_curves_to_point_list(n, &points, tolerance)
            .map_err(native_error)
    }

    /// Run Marionette's complete VMobject subpath-alignment algorithm. A
    /// same-Scene pair can mutate its shared Stage directly. Detached,
    /// mixed-state, and cross-Scene pairs are aligned as complete native
    /// entry copies first; Python then commits the two proven point runs
    /// through each receiver's ordinary RecordBuffer resize semantics.
    fn _align_vmobject_points(
        slf: &Bound<'_, Self>,
        other: &Bound<'_, BridgeMobject>,
        tolerance: f64,
    ) -> PyResult<Option<AlignedPointRuns>> {
        crossing::record(CrossingClass::FieldWrite);
        let left_bound = {
            let cell = slf.borrow();
            cell.engine.as_ref().map(Rc::clone).zip(cell.mob)
        };
        let right_bound = {
            let cell = other.borrow();
            cell.engine.as_ref().map(Rc::clone).zip(cell.mob)
        };
        if let (Some((left_engine, left_mob)), Some((right_engine, right_mob))) =
            (&left_bound, &right_bound)
            && same_engine(left_engine, right_engine)
        {
            left_engine
                .borrow_mut()
                .stage_mut()
                .align_points_with_tolerance(*left_mob, *right_mob, tolerance)
                .map_err(stage_error)?;
            return Ok(None);
        }

        let mut staging = Stage::new();
        let left = copy_proxy_entry_into(slf, &mut staging)?;
        let right = copy_proxy_entry_into(other, &mut staging)?;
        staging
            .align_points_with_tolerance(left, right, tolerance)
            .map_err(stage_error)?;
        let left_points = staging
            .get_points(left)
            .ok_or_else(|| StaleHandleError::new_err("aligned receiver no longer resolves"))?;
        let right_points = staging
            .get_points(right)
            .ok_or_else(|| StaleHandleError::new_err("aligned peer no longer resolves"))?;
        Ok(Some((left_points, right_points)))
    }

    /// Chisel's bounded sharp-curve subdivision over this entry's current
    /// shared-anchor path. The caller plans a whole requested family before
    /// committing any member, so a malformed threshold or budget refusal
    /// cannot leave a partially subdivided Python family.
    fn _subdivide_sharp_curve_points(
        slf: &Bound<'_, Self>,
        angle_threshold: f64,
    ) -> PyResult<Vec<[f64; 3]>> {
        crossing::record(CrossingClass::Other);
        let mut path = configured_quad_path(slf)?;
        path.subdivide_sharp_curves(angle_threshold)
            .map_err(native_error)?;
        Ok(path.points().to_vec())
    }

    /// Apply caller-supplied subdivision counts through Chisel's bounded
    /// preflight. Python evaluates the host callback exactly once for every
    /// live quadratic before entering this method, so no Stage borrow crosses
    /// user code and a whole requested family can be planned before its first
    /// RecordBuffer write.
    fn _subdivide_curve_points_by_counts(
        slf: &Bound<'_, Self>,
        subdivision_counts: &Bound<'_, PyAny>,
    ) -> PyResult<Vec<[f64; 3]>> {
        crossing::record(CrossingClass::Other);
        let mut path = configured_quad_path(slf)?;
        let mut counts = Vec::with_capacity(path.num_curves());
        for count in subdivision_counts.try_iter()? {
            let count = count?;
            counts.push(bounded_subdivision_count(&count)?);
        }
        if counts.len() != path.num_curves() {
            return Err(PyValueError::new_err(format!(
                "subdivision count length {} does not match the live curve count {}",
                counts.len(),
                path.num_curves()
            )));
        }
        let cursor = Cell::new(0usize);
        path.subdivide_curves_by_condition(|_| {
            let index = cursor.get();
            cursor.set(index + 1);
            counts
                .get(index)
                .copied()
                .unwrap_or(fmn_library::MAX_SUBDIVIDED_CURVES)
        })
        .map_err(native_error)?;
        Ok(path.points().to_vec())
    }

    /// Reference `subdivide_intersections` with both the strict crossing test
    /// and the bounded split performed in Chisel. Count coercion remains lazy:
    /// just like the Reference, an invalid positive count is never inspected
    /// when no curve intersects the captured anchor path.
    fn _subdivide_intersection_curve_points(
        slf: &Bound<'_, Self>,
        intersection_path: Vec<[f64; 3]>,
        n_subdivisions: &Bound<'_, PyAny>,
    ) -> PyResult<Vec<[f64; 3]>> {
        crossing::record(CrossingClass::Other);
        let mut path = configured_quad_path(slf)?;
        let intersects = path
            .bezier_tuples()
            .any(|[b0, b1, _]| fmn_library::line_intersects_path(b0, b1, &intersection_path));
        if !intersects {
            return Ok(path.points().to_vec());
        }
        let count = positive_subdivision_count(n_subdivisions)?;
        path.subdivide_curves_by_condition(|[b0, b1, _]| {
            if fmn_library::line_intersects_path(b0, b1, &intersection_path) {
                count
            } else {
                0
            }
        })
        .map_err(native_error)?;
        Ok(path.points().to_vec())
    }

    /// Return this VMobject's native shared-anchor points with curves whose
    /// handle and end anchor are both within `atol` of the start removed.
    /// Chisel owns the path invariant and the null-curve predicate; the
    /// Python layer only materializes the returned rows as a NumPy array.
    fn _get_points_without_null_curves(
        slf: &Bound<'_, Self>,
        atol: f64,
    ) -> PyResult<Vec<[f64; 3]>> {
        crossing::record(CrossingClass::Other);
        let path = configured_quad_path(slf)?;
        Ok(path.points_without_null_curves(atol))
    }

    /// Revalidate a Python-authored VMobject point run and refresh the native
    /// shared-anchor metadata through Marionette's one `Stage::set_points`
    /// path.  The Python layer owns Reference record-resize semantics; this
    /// call owns the geometry invariant and derived joint-angle column.
    fn _refresh_vmobject_path_metadata(slf: &Bound<'_, Self>) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| {
            let points = stage.get_points(mob).ok_or(StageError::StaleHandle)?;
            stage.set_points(mob, &points)
        })?
        .map_err(stage_error)
    }

    /// Chisel's anchor-mode smoothing over this entry's current world-space
    /// shared-anchor path. Python owns family recursion so `recurse=False`
    /// remains exact without introducing a second Stage traversal contract.
    fn _make_smooth(slf: &Bound<'_, Self>, approx: bool) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        let mut path = configured_quad_path(slf)?;
        if path.num_points() < 3 {
            return Ok(());
        }
        path.make_smooth(approx).map_err(native_error)?;
        with_stage(slf, |stage, mob| stage.set_points(mob, path.points()))?.map_err(stage_error)
    }

    /// Reference `reverse_points` through Marionette's family operation,
    /// which reverses every record row with its point and repairs path
    /// break handles/base normals.
    fn _reverse_points(slf: &Bound<'_, Self>, repair_recurse: bool) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| {
            stage.reverse_family_points_with_scope(mob, repair_recurse)
        })?
        .map_err(stage_error)
    }

    /// Whether this node's updater traversal is suspended. Ancestor pruning
    /// is applied by the scene target collector; this is the Reference's
    /// public self-state query used by `Mobject.update`.
    fn _is_updating_suspended(slf: &Bound<'_, Self>) -> PyResult<bool> {
        crossing::record(CrossingClass::Other);
        with_stage(slf, |stage, mob| stage.is_updating_suspended(mob))
    }

    /// Route `Mobject.suspend_updating` to Marionette's durable updater flag.
    fn _suspend_updating(slf: &Bound<'_, Self>, recurse: bool) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| stage.suspend_updating(mob, recurse))
    }

    /// Clear suspension on the selected family and ancestor chain. Python
    /// owns the immediate callback pass so no Stage borrow crosses a host
    /// callable; `Mobject.resume_updating` invokes it after this returns.
    fn _resume_updating(slf: &Bound<'_, Self>, recurse: bool) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| {
            stage.resume_updating(mob, recurse, false);
        })
    }

    /// Clear Marionette-owned updater slots alongside the bootstrap's host
    /// callback list. Python owns callable identity; Marionette owns native
    /// updater identities installed by engine-backed mobject features.
    fn _clear_native_updaters(slf: &Bound<'_, Self>, recurse: bool) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| stage.clear_updaters(mob, recurse))
    }

    /// Whether this mobject's Stage family holds any Marionette-owned
    /// updater. Python callables live in `updaters`; this answers for the
    /// native slots engine-backed features install.
    fn _has_native_updaters_in_family(slf: &Bound<'_, Self>) -> PyResult<bool> {
        crossing::record(CrossingClass::Other);
        with_stage(slf, |stage, mob| stage.has_updaters_in_family(mob))
    }

    /// Run only Marionette-owned updaters for this mobject. The bootstrap
    /// first performs the matching Python family pass outside any Stage
    /// borrow, preserving the portal's callback-safety boundary.
    fn _update_native_mobject(slf: &Bound<'_, Self>, dt: f64, recurse: bool) -> PyResult<()> {
        with_stage(slf, |stage, mob| {
            stage.update_mobject_with_recurse(mob, dt, recurse);
        })
    }

    /// True arc length over this entry's current world-space shared-anchor
    /// path. The optional sampling parameter on manim's VMobject surface is
    /// deliberately unnecessary: Chisel's error-bounded quadrature is the
    /// definition (BN-03), including for negative `path_arc` lines.
    fn _get_arc_length(slf: &Bound<'_, Self>) -> PyResult<f64> {
        crossing::record(CrossingClass::Other);
        with_stage(slf, |stage, mob| {
            let points = stage.get_points(mob).unwrap_or_default();
            fmn_library::VMobject::from_points(points)
                .path()
                .map(|path| path.get_arc_length())
        })?
        .map_err(native_error)
    }

    /// Atlas's `Line.get_vector` after Python performs Reference-style virtual
    /// endpoint dispatch (notably for `DashedLine`'s first/last children).
    fn _line_vector(_slf: &Bound<'_, Self>, start: [f64; 3], end: [f64; 3]) -> PyResult<[f64; 3]> {
        crossing::record(CrossingClass::Other);
        let line = fmn_library::line::Line::new(start, end)
            .build()
            .map_err(native_error)?;
        Ok(fmn_library::line::line_vector(&line))
    }

    /// Atlas's `Line.get_unit_vector` after virtual endpoint dispatch.
    fn _line_unit_vector(
        _slf: &Bound<'_, Self>,
        start: [f64; 3],
        end: [f64; 3],
    ) -> PyResult<[f64; 3]> {
        crossing::record(CrossingClass::Other);
        let line = fmn_library::line::Line::new(start, end)
            .build()
            .map_err(native_error)?;
        Ok(fmn_library::line::line_unit_vector(&line))
    }

    /// Atlas's planar `Line.get_angle` after virtual endpoint dispatch.
    fn _line_angle(_slf: &Bound<'_, Self>, start: [f64; 3], end: [f64; 3]) -> PyResult<f64> {
        crossing::record(CrossingClass::Other);
        let line = fmn_library::line::Line::new(start, end)
            .build()
            .map_err(native_error)?;
        Ok(fmn_library::line::line_angle(&line))
    }

    /// Atlas's deterministic-math `Line.get_slope`.
    fn _line_slope(_slf: &Bound<'_, Self>, start: [f64; 3], end: [f64; 3]) -> PyResult<f64> {
        crossing::record(CrossingClass::Other);
        let line = fmn_library::line::Line::new(start, end)
            .build()
            .map_err(native_error)?;
        Ok(fmn_library::line::line_slope(&line))
    }

    /// Atlas's `Line.get_projection`, preserving all three coordinates.
    fn _line_projection(
        _slf: &Bound<'_, Self>,
        start: [f64; 3],
        end: [f64; 3],
        point: [f64; 3],
    ) -> PyResult<[f64; 3]> {
        crossing::record(CrossingClass::Other);
        let line = fmn_library::line::Line::new(start, end)
            .build()
            .map_err(native_error)?;
        Ok(fmn_library::line::line_projection(&line, point))
    }

    /// The project contract's true-arclength `point_from_proportion`
    /// (BN-03), routed to Chisel through Marionette for both proxy states.
    fn _point_from_proportion(slf: &Bound<'_, Self>, alpha: f64) -> PyResult<[f64; 3]> {
        crossing::record(CrossingClass::Other);
        with_stage(slf, |stage, mob| stage.point_from_proportion(mob, alpha))?.map_err(stage_error)
    }

    /// Reference `VMobject.pointwise_become_partial`, routed to the same
    /// Marionette operation Choreo's creation animations use. The source may
    /// be detached or scene-bound; an unrelated Stage contributes a temporary
    /// root-entry copy so record data never crosses an active arena borrow.
    fn _pointwise_become_partial(
        slf: &Bound<'_, Self>,
        source: &Bound<'_, BridgeMobject>,
        a: f64,
        b: f64,
    ) -> PyResult<()> {
        if !a.is_finite() || !b.is_finite() {
            return Err(PyValueError::new_err("partial-curve bounds must be finite"));
        }
        crossing::record(CrossingClass::FieldWrite);

        let self_bound = {
            let cell = slf.borrow();
            cell.engine
                .as_ref()
                .zip(cell.mob)
                .map(|(engine, mob)| (Rc::clone(engine), mob))
        };
        if let Some((engine, mob)) = self_bound {
            let source_bound = {
                let cell = source.borrow();
                cell.engine
                    .as_ref()
                    .zip(cell.mob)
                    .map(|(source_engine, source_mob)| (Rc::clone(source_engine), source_mob))
            };
            if let Some((source_engine, source_mob)) = source_bound
                && same_engine(&engine, &source_engine)
            {
                return engine
                    .borrow_mut()
                    .stage_mut()
                    .pointwise_become_partial(mob, source_mob, a, b)
                    .map_err(stage_error);
            }

            let mut runtime = engine.borrow_mut();
            let stage = runtime.stage_mut();
            let temporary = copy_proxy_entry_into(source, stage)?;
            let result = stage
                .pointwise_become_partial(mob, temporary, a, b)
                .map_err(stage_error);
            let cleanup = stage.delete(temporary).map_err(stage_error);
            result.and(cleanup)
        } else {
            let mut cell = slf.borrow_mut();
            let nursery = cell.nursery.as_mut().ok_or_else(|| {
                StaleHandleError::new_err("partial target has no detached or bound state")
            })?;
            if slf.is(source) {
                return nursery
                    .stage
                    .pointwise_become_partial(nursery.root, nursery.root, a, b)
                    .map_err(stage_error);
            }
            let temporary = copy_proxy_entry_into(source, &mut nursery.stage)?;
            let result = nursery
                .stage
                .pointwise_become_partial(nursery.root, temporary, a, b)
                .map_err(stage_error);
            let cleanup = nursery.stage.delete(temporary).map_err(stage_error);
            result.and(cleanup)
        }
    }

    /// Reference `Surface.pointwise_become_partial`, routed to the same
    /// Marionette UV-grid operation used by Choreo's creation animations.
    /// Surface resolution is Python constructor state, so the authored
    /// public method supplies the source grid explicitly after resolving the
    /// Reference's preferred-axis default. As with the VMobject sibling, an
    /// unrelated Stage contributes a temporary root-entry copy and no Python
    /// operation runs while either Stage is borrowed.
    fn _surface_pointwise_become_partial(
        slf: &Bound<'_, Self>,
        source: &Bound<'_, BridgeMobject>,
        resolution: (usize, usize),
        axis: usize,
        a: f64,
        b: f64,
    ) -> PyResult<()> {
        if !a.is_finite() || !b.is_finite() {
            return Err(PyValueError::new_err(
                "partial-surface bounds must be finite",
            ));
        }
        crossing::record(CrossingClass::FieldWrite);

        let self_bound = {
            let cell = slf.borrow();
            cell.engine
                .as_ref()
                .zip(cell.mob)
                .map(|(engine, mob)| (Rc::clone(engine), mob))
        };
        if let Some((engine, mob)) = self_bound {
            let source_bound = {
                let cell = source.borrow();
                cell.engine
                    .as_ref()
                    .zip(cell.mob)
                    .map(|(source_engine, source_mob)| (Rc::clone(source_engine), source_mob))
            };
            if let Some((source_engine, source_mob)) = source_bound
                && same_engine(&engine, &source_engine)
            {
                return engine
                    .borrow_mut()
                    .stage_mut()
                    .surface_pointwise_become_partial(mob, source_mob, resolution, axis, a, b)
                    .map_err(stage_error);
            }

            let mut runtime = engine.borrow_mut();
            let stage = runtime.stage_mut();
            let temporary = copy_proxy_entry_into(source, stage)?;
            let result = stage
                .surface_pointwise_become_partial(mob, temporary, resolution, axis, a, b)
                .map_err(stage_error);
            let cleanup = stage.delete(temporary).map_err(stage_error);
            result.and(cleanup)
        } else {
            let mut cell = slf.borrow_mut();
            let nursery = cell.nursery.as_mut().ok_or_else(|| {
                StaleHandleError::new_err("partial surface target has no detached or bound state")
            })?;
            if slf.is(source) {
                return nursery
                    .stage
                    .surface_pointwise_become_partial(
                        nursery.root,
                        nursery.root,
                        resolution,
                        axis,
                        a,
                        b,
                    )
                    .map_err(stage_error);
            }
            let temporary = copy_proxy_entry_into(source, &mut nursery.stage)?;
            let result = nursery
                .stage
                .surface_pointwise_become_partial(nursery.root, temporary, resolution, axis, a, b)
                .map_err(stage_error);
            let cleanup = nursery.stage.delete(temporary).map_err(stage_error);
            result.and(cleanup)
        }
    }

    /// Atlas/Chisel's true-arclength dash placement expressed as the
    /// curve-index windows consumed by `pointwise_become_partial`.
    fn _dash_curve_intervals(
        slf: &Bound<'_, Self>,
        num_dashes: usize,
        positive_space_ratio: f64,
    ) -> PyResult<Vec<(f64, f64)>> {
        crossing::record(CrossingClass::Other);
        with_stage(slf, |stage, mob| {
            let source =
                fmn_library::VMobject::from_points(stage.get_points(mob).unwrap_or_default());
            fmn_library::vmobject::dash_curve_intervals(
                &source,
                num_dashes,
                positive_space_ratio,
                0.0,
            )
        })?
        .map_err(native_error)
    }

    /// Position an existing Python-owned tip with Atlas's true terminal
    /// tangent.  Only the completed point run crosses back to Python, which
    /// preserves the tip proxy's identity and style records.
    fn _position_tip_points(
        slf: &Bound<'_, Self>,
        tip: &Bound<'_, BridgeMobject>,
        at_start: bool,
    ) -> PyResult<Vec<[f64; 3]>> {
        crossing::record(CrossingClass::Other);
        let path = configured_quad_path(slf)?;
        let shape = fmn_library::VMobject::from_path(&path);
        let tip_points = with_stage(tip, |stage, mob| stage.get_points(mob).unwrap_or_default())?;
        let end = if at_start {
            fmn_library::TipEnd::Start
        } else {
            fmn_library::TipEnd::End
        };
        Ok(fmn_library::tip::position_tip(
            &shape,
            fmn_library::VMobject::from_points(tip_points),
            end,
        )
        .points()
        .to_vec())
    }

    /// Pull this shaft back to a positioned tip's base through Atlas's
    /// true-arc-length trim (BN-03).  The detached native result is complete
    /// before Marionette commits the point run, so geometry refusal cannot
    /// partially rewrite the live record buffer.
    fn _trim_to_tip(
        slf: &Bound<'_, Self>,
        tip: &Bound<'_, BridgeMobject>,
        at_start: bool,
    ) -> PyResult<()> {
        let path = configured_quad_path(slf)?;
        let shape = fmn_library::VMobject::from_path(&path);
        let tip_points = with_stage(tip, |stage, mob| stage.get_points(mob).unwrap_or_default())?;
        let tip = fmn_library::VMobject::from_points(tip_points);
        let end = if at_start {
            fmn_library::TipEnd::Start
        } else {
            fmn_library::TipEnd::End
        };
        let points = fmn_library::tip::trim_to_tip(shape, &tip, end)
            .points()
            .to_vec();
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| stage.set_points(mob, &points))?.map_err(stage_error)
    }

    /// `Arc.get_arc_center` over this entry's current world-space points.
    fn _arc_center(slf: &Bound<'_, Self>) -> PyResult<[f64; 3]> {
        crossing::record(CrossingClass::Other);
        with_stage(slf, |stage, mob| {
            fmn_library::arc::arc_center_of(&stage.get_points(mob).unwrap_or_default())
        })?
        .ok_or_else(|| {
            PyValueError::new_err("an arc center requires at least one quadratic component")
        })
    }

    /// `Circle.get_radius` over this entry's current world-space points.
    fn _circle_radius(slf: &Bound<'_, Self>) -> PyResult<f64> {
        crossing::record(CrossingClass::Other);
        with_stage(slf, |stage, mob| {
            let points = stage.get_points(mob).unwrap_or_default();
            fmn_library::arc::radius_of(&fmn_library::VMobject::from_points(points))
        })
    }

    /// `Stage::put_start_and_end_on` over the Stage-visible family.
    fn _put_start_and_end_on(
        slf: &Bound<'_, Self>,
        start: [f64; 3],
        end: [f64; 3],
    ) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| {
            stage.put_start_and_end_on(mob, start, end)
        })?
        .map_err(stage_error)
    }

    // ------------------------------------------------- native builders
    //
    // fm-d3gt: the schema-class constructor seam. Each method drives one
    // fmn-library builder and installs the built family via
    // `install_native_tree`; the returned nested `(shell, children)` specs
    // are hung on the Python family lists by the bootstrap.

    /// `CubicBezier(a0, h0, h1, a1)` over Chisel's one bounded
    /// cubic-to-quadratic converter. The detached native value is complete
    /// before it replaces the constructing proxy, so conversion refusal is
    /// failure-atomic at the Python boundary.
    fn _build_cubic_bezier<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        a0: [f64; 3],
        h0: [f64; 3],
        h1: [f64; 3],
        a1: [f64; 3],
    ) -> PyResult<Bound<'py, PyList>> {
        let built = fmn_library::CubicBezier::new(a0, h0, h1, a1)
            .build()
            .map_err(native_error)?;
        install_native_tree(slf, factory, built)
    }

    /// `Polygon(*vertices)` over Atlas's shared-anchor polygon builder.
    fn _build_polygon<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        vertices: Vec<[f64; 3]>,
    ) -> PyResult<Bound<'py, PyList>> {
        install_native_tree(slf, factory, fmn_library::Polygon::new(vertices).build())
    }

    /// `Union`/`Difference`/`Intersection`/`Exclusion` over Chisel path booleans.
    fn _build_boolean<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        operation: &str,
        operands: Vec<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyList>> {
        let operation = match operation {
            "union" => fmn_library::BooleanOperation::Union,
            "intersection" => fmn_library::BooleanOperation::Intersection,
            "difference" => fmn_library::BooleanOperation::Difference,
            "exclusion" => fmn_library::BooleanOperation::Exclusion,
            other => {
                return Err(PyValueError::new_err(format!(
                    "unknown boolean operation `{other}`"
                )));
            }
        };
        if operands.len() < 2 {
            return Err(PyValueError::new_err(
                "boolean constructors need at least two VMobject operands",
            ));
        }
        let mut paths = Vec::with_capacity(operands.len());
        for operand in &operands {
            let proxy = operand.cast::<BridgeMobject>()?;
            let points = with_stage(proxy, |stage, mob| {
                stage.get_points(mob).unwrap_or_default()
            })?;
            paths.push(fmn_library::QuadPath::from_points(points).map_err(native_error)?);
        }
        let mut acc = paths.remove(0);
        for clip in paths {
            let result = fmn_library::path_boolean(
                &acc,
                &clip,
                operation,
                fmn_library::BooleanOptions::default(),
            )
            .map_err(native_error)?;
            acc = result.path;
        }
        install_native_tree(slf, factory, fmn_library::VMobject::from_path(&acc))
    }

    /// `Polyline(*vertices)` over Atlas's open shared-anchor polygon builder.
    fn _build_polyline<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        vertices: Vec<[f64; 3]>,
    ) -> PyResult<Bound<'py, PyList>> {
        install_native_tree(
            slf,
            factory,
            fmn_library::Polygon::polyline(vertices).build(),
        )
    }

    /// `VectorizedPoint(location)` over Atlas's one-record native value.
    fn _build_vectorized_point<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        location: [f64; 3],
    ) -> PyResult<Bound<'py, PyList>> {
        install_native_tree(
            slf,
            factory,
            fmn_library::vmobject::vectorized_point(location),
        )
    }

    /// `MotionMobject(mobject)` over Atlas's native wrapper builder. The
    /// builder establishes the root composition; Python then hangs the
    /// original proxy below that root so the Reference's child identity is
    /// preserved instead of substituting a cloned compatibility shell.
    fn _build_motion_mobject<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        source: &Bound<'_, BridgeMobject>,
    ) -> PyResult<Bound<'py, PyList>> {
        let points = with_stage(source, |stage, mob| {
            stage.get_points(mob).unwrap_or_default()
        })?;
        let built =
            fmn_library::MotionMobject::new(fmn_library::VMobject::from_points(points)).build();
        let mut root = fmn_mobject::Mobject::from(built);
        root.submobjects.clear();
        install_native_tree(slf, factory, root)
    }

    /// `Button(mobject, on_click)` over Atlas's native arbitrary-mobject
    /// button builder. The builder establishes the wrapper root; Python then
    /// retains the caller's original proxy as the sole Reference child.
    fn _build_button<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        source: &Bound<'_, BridgeMobject>,
    ) -> PyResult<Bound<'py, PyList>> {
        let points = with_stage(source, |stage, mob| {
            stage.get_points(mob).unwrap_or_default()
        })?;
        let built = with_font_book(|book| {
            fmn_library::Button::of(fmn_library::VMobject::from_points(points))
                .build(book)
                .map_err(native_error)
        })?;
        let mut root = fmn_mobject::Mobject::from(built);
        root.submobjects.clear();
        install_native_tree(slf, factory, root)
    }

    /// `Checkbox(value)` over Atlas's native bool-control composition.
    /// The constructing Python proxy remains a live `ValueTracker`; Atlas
    /// supplies its two visual children in Reference order: box, then mark.
    #[allow(clippy::too_many_arguments)]
    fn _build_checkbox<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        value: bool,
        box_width: f64,
        box_height: f64,
        fill_opacity: f64,
        checkmark_color: [f64; 3],
        checkmark_stroke_width: f64,
        cross_color: [f64; 3],
        cross_stroke_width: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let mut composition = fmn_mobject::Mobject::from(
            fmn_library::Checkbox::new(value)
                .box_width(box_width)
                .box_height(box_height)
                .box_fill_opacity(fill_opacity)
                .checkmark_color(fmn_core::color::Srgb {
                    r: checkmark_color[0],
                    g: checkmark_color[1],
                    b: checkmark_color[2],
                })
                .checkmark_stroke_width(checkmark_stroke_width)
                .cross_color(fmn_core::color::Srgb {
                    r: cross_color[0],
                    g: cross_color[1],
                    b: cross_color[2],
                })
                .cross_stroke_width(cross_stroke_width)
                .composition(),
        );
        let children = std::mem::take(&mut composition.submobjects);
        if children.len() != 2 {
            return Err(PyRuntimeError::new_err(format!(
                "native Checkbox family contract drift: expected 2 children, got {}",
                children.len()
            )));
        }
        native_shell_specs(slf.py(), factory, children)
    }

    /// `EnableDisableButton(value)` over Atlas's native one-box bool
    /// control. The Python root remains the live `ValueTracker`; `colored`
    /// distinguishes Reference construction (default white) from a later
    /// `set_value_anim` update (GREEN/RED).
    #[allow(clippy::too_many_arguments)]
    fn _build_enable_disable_button<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        value: bool,
        colored: bool,
        width: f64,
        height: f64,
        fill_opacity: f64,
        enable_color: [f64; 3],
        disable_color: [f64; 3],
    ) -> PyResult<Bound<'py, PyList>> {
        let mut control = fmn_library::EnableDisableButton::new(value)
            .width(width)
            .height(height)
            .fill_opacity(fill_opacity)
            .enable_color(fmn_core::color::Srgb {
                r: enable_color[0],
                g: enable_color[1],
                b: enable_color[2],
            })
            .disable_color(fmn_core::color::Srgb {
                r: disable_color[0],
                g: disable_color[1],
                b: disable_color[2],
            });
        if colored {
            control.set_value(value);
        }
        let mut composition = fmn_mobject::Mobject::from(control.composition());
        let children = std::mem::take(&mut composition.submobjects);
        if children.len() != 1 {
            return Err(PyRuntimeError::new_err(format!(
                "native EnableDisableButton family contract drift: expected 1 child, got {}",
                children.len()
            )));
        }
        native_shell_specs(slf.py(), factory, children)
    }

    /// `LinearNumberSlider(...)` over Atlas's native rounded bar, handle,
    /// and invisible-axis builder. The Python `ControlMobject` root remains
    /// the live tracker while these native parts become its children.
    #[allow(clippy::too_many_arguments)]
    fn _build_linear_number_slider<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        value: f64,
        min_value: f64,
        max_value: f64,
        step: f64,
        bar_width: f64,
        bar_height: f64,
        corner_radius: f64,
        handle_radius: f64,
        handle_fill_opacity: f64,
        handle_stroke_color: [f64; 3],
        handle_fill_color: [f64; 3],
    ) -> PyResult<Bound<'py, PyList>> {
        let mut slider = fmn_library::LinearNumberSlider::new(value).map_err(native_error)?;
        if min_value > 10.0 {
            slider = slider
                .max_value(max_value)
                .and_then(|slider| slider.min_value(min_value))
                .map_err(native_error)?;
        } else {
            slider = slider
                .min_value(min_value)
                .and_then(|slider| slider.max_value(max_value))
                .map_err(native_error)?;
        }
        slider = slider
            .step(step)
            .and_then(|slider| slider.bar_width(bar_width))
            .and_then(|slider| slider.bar_height(bar_height))
            .and_then(|slider| slider.corner_radius(corner_radius))
            .map_err(native_error)?
            .handle_radius(handle_radius)
            .handle_fill_opacity(handle_fill_opacity)
            .handle_stroke_color(fmn_core::color::Srgb {
                r: handle_stroke_color[0],
                g: handle_stroke_color[1],
                b: handle_stroke_color[2],
            })
            .handle_fill_color(fmn_core::color::Srgb {
                r: handle_fill_color[0],
                g: handle_fill_color[1],
                b: handle_fill_color[2],
            });
        let mut composition = fmn_mobject::Mobject::from(slider.composition());
        let children = std::mem::take(&mut composition.submobjects);
        if children.len() != 3 {
            return Err(PyRuntimeError::new_err(format!(
                "native LinearNumberSlider family contract drift: expected 3 children, got {}",
                children.len()
            )));
        }
        native_shell_specs(slf.py(), factory, children)
    }

    /// `ColorSliders(...)` over Atlas's aggregate swatch plus four-slider
    /// composition. Python retains the two top-level shells so later native
    /// rebuilds can update their records without changing public identities.
    #[allow(clippy::too_many_arguments)]
    fn _build_color_sliders<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        red: f64,
        green: f64,
        blue: f64,
        alpha: f64,
        rect_width: f64,
        rect_height: f64,
        grid_colors: Vec<[f64; 3]>,
        single_square_len: f64,
        sliders_buff: f64,
        slider_min_value: Option<f64>,
        slider_max_value: Option<f64>,
        slider_step: Option<f64>,
        slider_bar_width: Option<f64>,
        slider_bar_height: Option<f64>,
        slider_corner_radius: Option<f64>,
        slider_handle_radius: Option<f64>,
        slider_handle_fill_opacity: Option<f64>,
        slider_handle_stroke_color: Option<[f64; 3]>,
        slider_handle_fill_color: Option<[f64; 3]>,
        apply_value: bool,
    ) -> PyResult<(Bound<'py, PyList>, Vec<f64>)> {
        let mut sliders = fmn_library::ColorSliders::new()
            .map_err(native_error)?
            .slider_overrides(
                slider_min_value,
                slider_max_value,
                slider_step,
                slider_bar_width,
                slider_bar_height,
                slider_corner_radius,
                slider_handle_radius,
                slider_handle_fill_opacity,
                slider_handle_stroke_color.map(|[r, g, b]| fmn_core::color::Srgb { r, g, b }),
                slider_handle_fill_color.map(|[r, g, b]| fmn_core::color::Srgb { r, g, b }),
            )
            .map_err(native_error)?
            .rect_size(rect_width, rect_height)
            .map_err(native_error)?
            .background_grid(
                grid_colors
                    .into_iter()
                    .map(|[r, g, b]| fmn_core::color::Srgb { r, g, b })
                    .collect(),
                single_square_len,
            )
            .map_err(native_error)?
            .sliders_buff(sliders_buff);
        if apply_value {
            sliders
                .set_value(red, green, blue, alpha)
                .map_err(native_error)?;
        }
        let component_values = sliders
            .sliders()
            .iter()
            .map(fmn_library::LinearNumberSlider::value)
            .collect();
        let mut composition = fmn_mobject::Mobject::from(sliders.composition());
        let children = std::mem::take(&mut composition.submobjects);
        if children.len() != 2 {
            return Err(PyRuntimeError::new_err(format!(
                "native ColorSliders family contract drift: expected 2 children, got {}",
                children.len()
            )));
        }
        Ok((
            native_shell_specs(slf.py(), factory, children)?,
            component_values,
        ))
    }

    /// Build a `Textbox` candidate with the bundled FontBook. Supplying
    /// `replacement` routes through Atlas's string `set_value` mutator before
    /// any Python proxy is changed, making rejected re-typesets atomic.
    /// `box` is `(width, height, fill_rgb, fill_opacity)`.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    fn _build_textbox<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        current: &str,
        replacement: Option<&str>,
        initially_active: bool,
        box_spec: (f64, f64, [f64; 3], f64),
        text_color: [f64; 3],
        text_buff: f64,
        active_color: [f64; 3],
        deactive_color: [f64; 3],
    ) -> PyResult<Bound<'py, PyList>> {
        let (box_width, box_height, box_fill, box_fill_opacity) = box_spec;
        let book = Rc::new(fmn_library::FontBook::bundled().map_err(|error| {
            PyRuntimeError::new_err(format!("bundled FontBook unavailable: {error}"))
        })?);
        let mut textbox = fmn_library::Textbox::new(book, current)
            .map_err(textbox_error)?
            .box_size(box_width, box_height)
            .map_err(textbox_error)?
            .box_fill(
                fmn_core::color::Srgb {
                    r: box_fill[0],
                    g: box_fill[1],
                    b: box_fill[2],
                },
                box_fill_opacity,
            )
            .text_color(fmn_core::color::Srgb {
                r: text_color[0],
                g: text_color[1],
                b: text_color[2],
            })
            .map_err(textbox_error)?
            .text_buff(text_buff)
            .map_err(textbox_error)?
            .active_color(fmn_core::color::Srgb {
                r: active_color[0],
                g: active_color[1],
                b: active_color[2],
            })
            .deactive_color(fmn_core::color::Srgb {
                r: deactive_color[0],
                g: deactive_color[1],
                b: deactive_color[2],
            })
            .initially_active(initially_active);
        if let Some(value) = replacement {
            textbox.set_value(value).map_err(textbox_error)?;
        }
        let mut composition = fmn_mobject::Mobject::from(textbox.composition());
        let mut children = std::mem::take(&mut composition.submobjects);
        if children.len() != 2 {
            return Err(PyRuntimeError::new_err(format!(
                "native Textbox family contract drift: expected 2 children, got {}",
                children.len()
            )));
        }
        hoist_descendant_records(&mut children[1], false)?;
        native_shell_specs(slf.py(), factory, children)
    }

    /// `ControlPanel(*controls)` over Atlas's native panel, opener tab, and
    /// controls column. The Python root is a `Group`; these three children
    /// keep stable shell identities across open/close rebuilds.
    /// `panel` is `(width, height, fill_rgb, fill_opacity, stroke_rgb,
    /// stroke_width, stroke_opacity)`; `opener` is the same 7-tuple.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    fn _build_control_panel<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        control_extents: Vec<([f64; 3], [f64; 3])>,
        opener_text: &str,
        opener_font_size: f64,
        opener_text_color: [f64; 3],
        open: bool,
        panel: (f64, f64, [f64; 3], f64, [f64; 3], f64, f64),
        opener: (f64, f64, [f64; 3], f64, [f64; 3], f64, f64),
    ) -> PyResult<Bound<'py, PyList>> {
        let (
            panel_width,
            panel_height,
            panel_fill,
            panel_fill_opacity,
            panel_stroke,
            panel_stroke_width,
            panel_stroke_opacity,
        ) = panel;
        let (
            opener_width,
            opener_height,
            opener_fill,
            opener_fill_opacity,
            opener_stroke,
            opener_stroke_width,
            opener_stroke_opacity,
        ) = opener;
        let controls = control_extents
            .into_iter()
            .map(|extent| matcher_extent_vmobject(Some(extent)));
        let built = with_font_book(|book| {
            let mut panel = fmn_library::ControlPanel::new(controls)
                .opener_text(opener_text)
                .opener_font_size(opener_font_size)
                .opener_text_color(fmn_core::color::Srgb {
                    r: opener_text_color[0],
                    g: opener_text_color[1],
                    b: opener_text_color[2],
                })
                .panel_width(panel_width)
                .panel_height(panel_height)
                .panel_fill(
                    fmn_core::color::Srgb {
                        r: panel_fill[0],
                        g: panel_fill[1],
                        b: panel_fill[2],
                    },
                    panel_fill_opacity,
                )
                .panel_stroke(
                    fmn_core::color::Srgb {
                        r: panel_stroke[0],
                        g: panel_stroke[1],
                        b: panel_stroke[2],
                    },
                    panel_stroke_width,
                    panel_stroke_opacity,
                )
                .opener_width(opener_width)
                .opener_height(opener_height)
                .opener_fill(
                    fmn_core::color::Srgb {
                        r: opener_fill[0],
                        g: opener_fill[1],
                        b: opener_fill[2],
                    },
                    opener_fill_opacity,
                )
                .opener_stroke(
                    fmn_core::color::Srgb {
                        r: opener_stroke[0],
                        g: opener_stroke[1],
                        b: opener_stroke[2],
                    },
                    opener_stroke_width,
                    opener_stroke_opacity,
                )
                .build(book)
                .map_err(native_error)?;
            if open {
                panel.open_panel();
            } else {
                panel.close_panel();
            }
            Ok(panel)
        })?;
        let mut composition = fmn_mobject::Mobject::from(built.composition());
        let children = std::mem::take(&mut composition.submobjects);
        if children.len() != 3 {
            return Err(PyRuntimeError::new_err(format!(
                "native ControlPanel family contract drift: expected 3 children, got {}",
                children.len()
            )));
        }
        native_shell_specs(slf.py(), factory, children)
    }

    /// Re-layout a live portal `ControlPanel` after its controls family
    /// changes. Atlas owns the panel/opener geometry law; Python only grafts
    /// the original control proxies onto the returned native targets.
    fn _layout_control_panel_against_opener<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        panel_extent: ([f64; 3], [f64; 3]),
        opener_extent: ([f64; 3], [f64; 3]),
        control_extents: Vec<([f64; 3], [f64; 3])>,
    ) -> PyResult<Bound<'py, PyList>> {
        let panel = matcher_extent_vmobject(Some(panel_extent));
        let opener = matcher_extent_vmobject(Some(opener_extent));
        let controls = fmn_library::VMobject::new().with_children(
            control_extents
                .into_iter()
                .map(|extent| matcher_extent_vmobject(Some(extent))),
        );
        let (panel, controls) =
            fmn_library::ControlPanelMobject::layout_against_opener(panel, &opener, controls);
        native_shell_specs(slf.py(), factory, vec![panel.into(), controls.into()])
    }

    /// `SVGMobject(file_name=…, svg_string=…)` over Chisel's hardened
    /// user-SVG document processor (fm-5wq.4.50, G2 criterion 4). The file
    /// is read natively under the processor's declared byte budget — the
    /// size is checked against filesystem metadata *before* the read, so a
    /// bomb file never allocates — and every processor refusal (bombs,
    /// nesting, DOCTYPE, unsupported features, malformed XML) is fmn-geom's
    /// typed `SvgError`, surfaced verbatim as the named Python error.
    fn _build_svg_mobject<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        file_name: &str,
        svg_string: Option<&str>,
    ) -> PyResult<Bound<'py, PyList>> {
        let limits = fmn_library::svg::SvgLimits::default();
        let document = if let Some(text) = svg_string {
            fmn_library::svg::SvgDocument::parse_with_limits(text.as_bytes(), &limits)
                .map_err(native_error)?
        } else {
            let contents = read_svg_source(slf.py(), file_name, &limits)?;
            fmn_library::svg::SvgDocument::parse_with_limits(&contents, &limits)
                .map_err(native_error)?
        };
        install_native_tree(slf, factory, fmn_library::svg_document_mobject(&document))
    }

    /// `SVGMobject.file_name_to_svg_string` over the same governed reader
    /// the constructor uses: identical byte budget, identical error surface,
    /// so the Python seam cannot read what the parser would refuse.
    #[staticmethod]
    fn _read_svg_file(py: Python<'_>, file_name: &str) -> PyResult<String> {
        let limits = fmn_library::svg::SvgLimits::default();
        let contents = read_svg_source(py, file_name, &limits)?;
        String::from_utf8(contents).map_err(|error| {
            PyOSError::new_err(format!(
                "SVGMobject source {file_name:?} is not UTF-8: {error}"
            ))
        })
    }

    /// `VMobjectFromSVGPath(path_obj)` over the same hardened Chisel path
    /// parser as `SVGMobject`. The wrapper contains exactly one path, whose
    /// native child becomes this proxy's root rather than an extra family
    /// level.
    fn _build_svg_path<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        path_data: &str,
    ) -> PyResult<Bound<'py, PyList>> {
        let limits = fmn_library::svg::SvgLimits::default();
        let escaped = path_data
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;");
        let source = format!("<svg><path d=\"{escaped}\"/></svg>");
        if source.len() > limits.max_bytes {
            return Err(native_error(fmn_library::svg::SvgError::TooLarge {
                bytes: source.len(),
                limit: limits.max_bytes,
            }));
        }
        let document = fmn_library::svg::SvgDocument::parse_with_limits(source.as_bytes(), &limits)
            .map_err(native_error)?;
        let family = fmn_library::svg_document_mobject(&document);
        let path = family.children().first().cloned().unwrap_or_default();
        install_native_tree(slf, factory, path)
    }

    /// Split a source's live shared-anchor run with Atlas's
    /// `CurvesAsSubmobjects` builder. Python reapplies the source style to
    /// each returned child, matching the Reference's constructor body.
    fn _build_curves_as_submobjects<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        source: &Bound<'_, BridgeMobject>,
    ) -> PyResult<Bound<'py, PyList>> {
        let points = with_stage(source, |stage, mob| {
            stage.get_points(mob).unwrap_or_default()
        })?;
        let source = fmn_library::VMobject::from_points(points);
        install_native_tree(
            slf,
            factory,
            fmn_library::vmobject::curves_as_submobjects(&source),
        )
    }

    /// `RegularPolygon(n, radius, start_angle)`: the bounded native
    /// compass-direction kernel owns both the vertex count and orientation.
    fn _build_regular_polygon<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        n: usize,
        radius: f64,
        start_angle: Option<f64>,
    ) -> PyResult<Bound<'py, PyList>> {
        let mut polygon = fmn_library::RegularPolygon::new(n).radius(radius);
        if let Some(angle) = start_angle {
            polygon = polygon.start_angle(angle);
        }
        install_native_tree(slf, factory, polygon.build().map_err(native_error)?)
    }

    /// `ArrowTip(...)` over the one native tip builder.  As in the
    /// Reference, values other than the two special style codes fall back
    /// to the ordinary triangular tip.
    #[allow(clippy::too_many_arguments)]
    fn _build_arrow_tip<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        angle: f64,
        width: f64,
        length: f64,
        tip_style: i64,
    ) -> PyResult<Bound<'py, PyList>> {
        let tip_style = match tip_style {
            1 => fmn_library::TipStyle::InnerSmooth,
            2 => fmn_library::TipStyle::Dot,
            _ => fmn_library::TipStyle::Triangle,
        };
        let built = fmn_library::ArrowTip::new()
            .angle(angle)
            .width(width)
            .length(length)
            .tip_style(tip_style)
            .build();
        install_native_tree(slf, factory, built)
    }

    /// Mutate a detached or scene-bound polygon through Atlas's native
    /// rounded-corner algorithm.  The complete result is built before the
    /// Stage write, so malformed geometry cannot leave a partial mutation.
    fn _round_polygon_corners(
        slf: &Bound<'_, Self>,
        vertices: Vec<[f64; 3]>,
        radius: Option<f64>,
    ) -> PyResult<()> {
        let rounded = fmn_library::Polygon::new(vertices)
            .round_corners(radius)
            .map_err(native_error)?;
        with_stage(slf, |stage, mob| stage.set_points(mob, rounded.points()))?.map_err(stage_error)
    }

    /// `Rectangle(width, height)` over the polygon shelf.
    fn _build_rectangle<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        width: f64,
        height: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let built = fmn_library::Rectangle::new()
            .width(width)
            .height(height)
            .build()
            .map_err(native_error)?;
        install_native_tree(slf, factory, built)
    }

    /// `RoundedRectangle(width, height, corner_radius)` over the same Atlas
    /// rectangle builder.  Atlas rounds only after applying the requested
    /// extent, so the corners remain circular rather than stretched arcs.
    fn _build_rounded_rectangle<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        width: f64,
        height: f64,
        corner_radius: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let built = fmn_library::Rectangle::new()
            .width(width)
            .height(height)
            .corner_radius(corner_radius)
            .build()
            .map_err(native_error)?;
        install_native_tree(slf, factory, built)
    }

    /// `ScreenRectangle(aspect_ratio, height)` over Atlas's frame helper.
    fn _build_screen_rectangle<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        aspect_ratio: f64,
        height: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let built = fmn_library::poly::screen_rectangle(aspect_ratio, height)
            .build()
            .map_err(native_error)?;
        install_native_tree(slf, factory, built)
    }

    /// `SurroundingRectangle(mobject, buff)`: feed Marionette's
    /// authoritative family extent into Atlas's one shape-matcher
    /// implementation.  `has_extent` distinguishes an empty family from a
    /// genuine zero-size family at the origin.
    fn _build_surrounding_rectangle<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        min: [f64; 3],
        max: [f64; 3],
        has_extent: bool,
        buff: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let extent = has_extent.then_some((min, max));
        let built = fmn_library::SurroundingRectangle::from_extent(extent)
            .buff(buff)
            .build();
        install_native_tree(slf, factory, built)
    }

    /// `Cross(mobject, ...)` over Atlas's tapered two-arm matcher. The
    /// source proxy contributes its authoritative live family geometry;
    /// Atlas alone derives the extent, arm paths, and default taper.
    fn _build_cross<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        extent: Option<([f64; 3], [f64; 3])>,
        color: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyList>> {
        let target = matcher_extent_vmobject(extent);
        let built = fmn_library::cross(&target, srgb_from_py(color)?, 6.0);
        install_native_tree(slf, factory, built)
    }

    /// `Underline(mobject, ...)` over Atlas's extent-driven tapered rule.
    /// Non-finite placement controls refuse before any nursery mutation.
    fn _build_underline<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        extent: Option<([f64; 3], [f64; 3])>,
        color: &Bound<'_, PyAny>,
        buff: f64,
        stretch_factor: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        if !buff.is_finite() || !stretch_factor.is_finite() {
            return Err(PyValueError::new_err(
                "Underline buff and stretch_factor must be finite",
            ));
        }
        let target = matcher_extent_vmobject(extent);
        let built = fmn_library::underline(&target, srgb_from_py(color)?, buff, stretch_factor);
        install_native_tree(slf, factory, built)
    }

    /// The paired `pifont` marks are native drawn paths (BN-08), wrapped in
    /// one empty TexText-family root so inherited indexing and selector
    /// behavior retain the Reference's one-glyph family shape.
    fn _build_checkmark<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        color: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyList>> {
        let mark = fmn_library::checkmark(srgb_from_py(color)?);
        let tree = fmn_library::VMobject::new().with_children([mark]);
        install_native_tree(slf, factory, tree)
    }

    /// Native drawn sibling of [`Self::_build_checkmark`].
    fn _build_exmark<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        color: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyList>> {
        let mark = fmn_library::exmark(srgb_from_py(color)?);
        let tree = fmn_library::VMobject::new().with_children([mark]);
        install_native_tree(slf, factory, tree)
    }

    /// Retarget a scene-bound `SurroundingRectangle` without replacing its
    /// arena entry. Atlas remains the one geometry implementation; only the
    /// newly built world-space point run and primitive hint replace the live
    /// entry, while the Python layer reapplies its existing style.
    fn _rebuild_surrounding_rectangle(
        slf: &Bound<'_, Self>,
        min: [f64; 3],
        max: [f64; 3],
        has_extent: bool,
        buff: f64,
    ) -> PyResult<()> {
        let extent = has_extent.then_some((min, max));
        let built = fmn_library::SurroundingRectangle::from_extent(extent)
            .buff(buff)
            .build();
        let points = built.points().to_vec();
        let shape = built.shape();
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| {
            stage.set_points(mob, &points)?;
            stage.set_shape(mob, shape);
            Ok(())
        })?
        .map_err(stage_error)
    }

    /// Native `Brace(mobject, direction, buff)` over the target's live
    /// world-space family geometry. The returned point index tracks the
    /// analytic curl tip through later affine transforms.
    fn _build_brace<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        target: &Bound<'_, PyAny>,
        direction: [f64; 3],
        buff: f64,
    ) -> PyResult<(Bound<'py, PyList>, usize)> {
        // The Reference sizes a brace from the target's corners, i.e. its
        // whole family's points (`get_all_points`). Read them through the
        // public surface: a detached composite's children live in their own
        // nurseries, so the root's native stage family alone is empty and the
        // brace collapsed to width 0 at the origin (fm-5wq.43).
        let points: Vec<[f64; 3]> = target
            .call_method0("get_all_points")?
            .call_method1("tolist", ())?
            .extract()?;
        if points.iter().flatten().any(|value| !value.is_finite()) {
            return Err(PyValueError::new_err("brace target points must be finite"));
        }
        let target = fmn_library::VMobject::from_points(points);
        let brace = fmn_library::Brace::around(&target, direction).buff(buff);
        install_brace_tree(slf, factory, brace)
    }

    /// Native `LineBrace`: Atlas owns its arbitrary-angle geometry and this
    /// portal only installs the resulting retained family.
    fn _build_line_brace<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        start: [f64; 3],
        end: [f64; 3],
        direction: [f64; 3],
        buff: f64,
    ) -> PyResult<(Bound<'py, PyList>, usize)> {
        let brace = fmn_library::line_brace(start, end, direction).buff(buff);
        install_brace_tree(slf, factory, brace)
    }

    /// `ValueTracker` initialization: replace the detached nursery with a
    /// native tracker entry (`Stage::add_value_tracker` and kin) —
    /// state-real in both proxy states, and `copy_into` carries the
    /// tracker through scene adoption.
    fn _init_value_tracker(slf: &Bound<'_, Self>, kind: u8, value: f64, im: f64) -> PyResult<()> {
        let mut cell = slf.borrow_mut();
        if cell.engine.is_some() {
            return Err(PyRuntimeError::new_err(
                "a value tracker initializes before scene entry",
            ));
        }
        cell.nursery = Some(Nursery::value_tracker(kind, value, im));
        cell.initialized = true;
        Ok(())
    }

    /// The decoded scalar tracker value (Plain or Exponential).
    fn _tracker_value(slf: &Bound<'_, Self>) -> PyResult<f64> {
        crossing::record(CrossingClass::Other);
        with_stage(slf, |stage, mob| stage.tracker_value(mob))?
            .ok_or_else(|| StaleHandleError::new_err("no scalar value tracker behind this proxy"))
    }

    /// The complex tracker value as `(re, im)`.
    fn _tracker_complex_value(slf: &Bound<'_, Self>) -> PyResult<(f64, f64)> {
        crossing::record(CrossingClass::Other);
        with_stage(slf, |stage, mob| stage.tracker_complex_value(mob))?
            .ok_or_else(|| StaleHandleError::new_err("no complex value tracker behind this proxy"))
    }

    fn _set_tracker_value(slf: &Bound<'_, Self>, value: f64) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| stage.set_tracker_value(mob, value))?.map_err(stage_error)
    }

    fn _set_tracker_complex_value(slf: &Bound<'_, Self>, re: f64, im: f64) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| {
            stage.set_tracker_complex_value(mob, re, im)
        })?
        .map_err(stage_error)
    }

    fn _increment_tracker_value(slf: &Bound<'_, Self>, d_value: f64) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| {
            stage.increment_tracker_value(mob, d_value)
        })?
        .map_err(stage_error)
    }

    /// `Stage::set_z_index` for this entry alone; the bootstrap
    /// distributes over the family list in both proxy states.
    fn _set_z_index(slf: &Bound<'_, Self>, z_index: i32) -> PyResult<()> {
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| {
            stage.set_z_index(mob, z_index, false);
        })
    }

    /// This entry's own sort key, the Reference's `mobject.z_index`.
    fn _get_z_index(slf: &Bound<'_, Self>) -> PyResult<i32> {
        crossing::record(CrossingClass::Other);
        with_stage(slf, |stage, mob| stage.z_index(mob))
    }

    /// `space_ops.rotate_vector` over the ONE rotation implementation
    /// (fmn-geom's scipy-exact quaternion `rotation_matrix`, the same
    /// kernel `Stage::rotate` composes).
    #[staticmethod]
    fn _rotate_vector(vector: [f64; 3], angle: f64, axis: [f64; 3]) -> [f64; 3] {
        let matrix = fmn_library::rotation_matrix(angle, axis);
        [
            matrix[0][0] * vector[0] + matrix[0][1] * vector[1] + matrix[0][2] * vector[2],
            matrix[1][0] * vector[0] + matrix[1][1] * vector[1] + matrix[1][2] * vector[2],
            matrix[2][0] * vector[0] + matrix[2][1] * vector[1] + matrix[2][2] * vector[2],
        ]
    }

    /// Chisel's deterministic polar-angle kernel behind the familiar public
    /// utility function.
    #[staticmethod]
    fn _angle_of_vector(vector: [f64; 3]) -> f64 {
        fmn_library::angle_of_vector(vector)
    }

    /// Chisel's deterministic three-dimensional angle kernel. The Python
    /// surface retains the Reference's arbitrary-dimensional fallback.
    #[staticmethod]
    fn _angle_between_vectors(v1: [f64; 3], v2: [f64; 3]) -> f64 {
        fmn_library::angle_between_vectors(v1, v2)
    }

    /// Chisel's exact strict-crossing predicate for a line segment and a
    /// polygonal path. Only xy coordinates participate, matching the pinned
    /// Reference for both two- and three-dimensional inputs.
    #[staticmethod]
    fn _line_intersects_path(start: [f64; 3], end: [f64; 3], path: Vec<[f64; 3]>) -> bool {
        fmn_library::line_intersects_path(start, end, &path)
    }

    // -------------------------------------------------------------------
    // space_ops utility seams (fm-fnun): every remaining public
    // `manimlib.utils.space_ops` function routes through fmn-geom's one
    // deterministic authority (D4). The bootstrap performs only NumPy
    // shape coercion and never re-derives the mathematics.
    // -------------------------------------------------------------------

    /// Reference varargs quaternion product, scipy scalar-last order.
    #[staticmethod]
    fn _quaternion_mult(quats: Vec<[f64; 4]>) -> [f64; 4] {
        fmn_library::quaternion_mult(&quats)
    }

    /// Negate the vector part, keep the scalar part (scipy order).
    #[staticmethod]
    fn _quaternion_conjugate(quat: [f64; 4]) -> [f64; 4] {
        fmn_library::quaternion_conjugate(quat)
    }

    /// `Rotation.from_rotvec(angle * normalize(axis)).as_quat()`.
    #[staticmethod]
    fn _quaternion_from_angle_axis(angle: f64, axis: [f64; 3]) -> [f64; 4] {
        fmn_library::quaternion_from_angle_axis(angle, axis)
    }

    /// `Rotation.from_quat(quat).as_rotvec()` split into magnitude and
    /// unit direction. A zero-norm quaternion is the precise refusal
    /// scipy's `ValueError` names.
    #[staticmethod]
    fn _angle_axis_from_quaternion(quat: [f64; 4]) -> PyResult<(f64, [f64; 3])> {
        fmn_library::angle_axis_from_quaternion(quat).ok_or_else(|| {
            PyValueError::new_err("angle_axis_from_quaternion requires a nonzero-norm quaternion")
        })
    }

    /// The four rotation-matrix members. `*_transpose_from_quaternion` is
    /// scipy `as_matrix` verbatim (the pinned naming quirk); the plain
    /// spellings are its transpose, exactly as `space_ops.py` defines
    /// them. Zero-norm quaternions refuse precisely where scipy would.
    #[staticmethod]
    fn _rotation_matrix_transpose_from_quaternion(quat: [f64; 4]) -> PyResult<fmn_library::Mat3> {
        fmn_library::rotation_matrix_transpose_from_quaternion(quat).ok_or_else(|| {
            PyValueError::new_err("rotation matrix requires a nonzero-norm quaternion")
        })
    }

    #[staticmethod]
    fn _rotation_matrix_from_quaternion(quat: [f64; 4]) -> PyResult<fmn_library::Mat3> {
        fmn_library::rotation_matrix_from_quaternion(quat).ok_or_else(|| {
            PyValueError::new_err("rotation matrix requires a nonzero-norm quaternion")
        })
    }

    #[staticmethod]
    fn _rotation_matrix(angle: f64, axis: [f64; 3]) -> fmn_library::Mat3 {
        fmn_library::rotation_matrix(angle, axis)
    }

    #[staticmethod]
    fn _rotation_matrix_transpose(angle: f64, axis: [f64; 3]) -> fmn_library::Mat3 {
        fmn_library::rotation_matrix_transpose(angle, axis)
    }

    #[staticmethod]
    fn _rotation_about_z(angle: f64) -> fmn_library::Mat3 {
        fmn_library::rotation_about_z(angle)
    }

    #[staticmethod]
    fn _rotation_between_vectors(v1: [f64; 3], v2: [f64; 3]) -> fmn_library::Mat3 {
        fmn_library::rotation_between_vectors(v1, v2)
    }

    #[staticmethod]
    fn _z_to_vector(vector: [f64; 3]) -> fmn_library::Mat3 {
        fmn_library::z_to_vector(vector)
    }

    /// Chisel's polyline length over the xy plane's z-padded points.
    #[staticmethod]
    fn _poly_line_length(points: Vec<[f64; 3]>) -> f64 {
        fmn_library::poly_line_length(&points)
    }

    /// Chisel's degeneracy-aware normal with the Reference default
    /// tolerance applied in the bootstrap.
    #[staticmethod]
    fn _get_unit_normal(v1: [f64; 3], v2: [f64; 3], tol: f64) -> [f64; 3] {
        fmn_library::get_unit_normal(v1, v2, tol)
    }

    #[staticmethod]
    fn _project_along_vector(point: [f64; 3], vector: [f64; 3]) -> [f64; 3] {
        fmn_library::project_along_vector(point, vector)
    }

    /// Parallel lines are the precise refusal the Reference raises for;
    /// the deterministic solver covers the rest.
    #[staticmethod]
    fn _line_intersection(
        line1: ([f64; 3], [f64; 3]),
        line2: ([f64; 3], [f64; 3]),
    ) -> PyResult<[f64; 3]> {
        fmn_library::line_intersection(line1, line2).ok_or_else(|| {
            PyValueError::new_err("LineIntersectionException: Lines do not intersect")
        })
    }

    #[staticmethod]
    fn _find_intersection(
        p0: [f64; 3],
        v0: [f64; 3],
        p1: [f64; 3],
        v1: [f64; 3],
        threshold: f64,
    ) -> [f64; 3] {
        fmn_library::find_intersection(p0, v0, p1, v1, threshold)
    }

    #[staticmethod]
    fn _get_closest_point_on_line(a: [f64; 3], b: [f64; 3], p: [f64; 3]) -> [f64; 3] {
        fmn_library::get_closest_point_on_line(a, b, p)
    }

    #[staticmethod]
    fn _get_winding_number(points: Vec<[f64; 3]>) -> f64 {
        fmn_library::get_winding_number(&points)
    }

    #[staticmethod]
    fn _tri_area(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
        fmn_library::tri_area(a, b, c)
    }

    #[staticmethod]
    fn _is_inside_triangle(p: [f64; 3], a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> bool {
        fmn_library::is_inside_triangle(p, a, b, c)
    }

    /// Bounded square mask as row-major rows; the bootstrap materializes
    /// the Reference's uint8 ndarray from it.
    #[staticmethod]
    fn _thick_diagonal(dim: usize, thickness: usize) -> PyResult<Vec<Vec<i64>>> {
        fmn_library::thick_diagonal(dim, thickness)
            .map(|rows| {
                rows.into_iter()
                    .map(|row| row.iter().map(|&cell| i64::from(cell)).collect())
                    .collect()
            })
            .map_err(|error| PyValueError::new_err(format!("thick_diagonal refused: {error}")))
    }

    /// One earcut group through Chisel's dependency-free triangulator:
    /// outer ring plus already-grouped hole rings, triangle indices over
    /// the flattened `[outer, holes…]` vertex order. Degenerate or
    /// interfering geometry is the named refusal, never a degraded mesh.
    #[staticmethod]
    fn _earclip_triangulate_group(
        outer: Vec<[f64; 2]>,
        holes: Vec<Vec<[f64; 2]>>,
    ) -> PyResult<Vec<[u32; 3]>> {
        let hole_refs: Vec<&[[f64; 2]]> = holes.iter().map(Vec::as_slice).collect();
        fmn_library::earclip_triangulate(&outer, &hole_refs).map_err(|error| {
            PyValueError::new_err(format!("earclip_triangulation refused: {error}"))
        })
    }

    /// Chisel's quadratic handle approximation over an anchor sequence.
    #[staticmethod]
    fn _approx_smooth_quadratic_handles(points: Vec<[f64; 3]>) -> Vec<[f64; 3]> {
        fmn_library::approx_smooth_quadratic_handles(&points)
    }

    /// Chisel's banded (open) and dense (closed) cubic-handle solver —
    /// the one authority behind `get_smooth_cubic_bezier_handle_points`.
    #[allow(clippy::type_complexity)]
    #[staticmethod]
    fn _smooth_cubic_handles(points: Vec<[f64; 3]>) -> PyResult<(Vec<[f64; 3]>, Vec<[f64; 3]>)> {
        fmn_library::smooth_cubic_handles(&points).map_err(|error| {
            PyValueError::new_err(format!("smooth cubic handles refused: {error}"))
        })
    }

    /// Chisel's smooth quadratic spline: rotation handling, per-segment
    /// error contract, and the one error-bounded cubic converter inside.
    #[staticmethod]
    fn _smooth_quadratic_path(points: Vec<[f64; 3]>, tolerance: f64) -> PyResult<Vec<[f64; 3]>> {
        fmn_library::smooth_quadratic_path(&points, tolerance).map_err(|error| {
            PyValueError::new_err(format!("smooth_quadratic_path refused: {error}"))
        })
    }

    /// `Arc(start_angle, angle, radius, arc_center)` over the arc shelf.
    #[allow(clippy::too_many_arguments)]
    fn _build_arc<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        start_angle: f64,
        angle: f64,
        radius: f64,
        arc_center: [f64; 3],
        n_components: Option<usize>,
    ) -> PyResult<Bound<'py, PyList>> {
        let mut arc = fmn_library::Arc::new()
            .start_angle(start_angle)
            .angle(angle)
            .radius(radius)
            .arc_center(arc_center);
        if let Some(n) = n_components {
            arc = arc.n_components(n);
        }
        let built = arc.build().map_err(native_error)?;
        install_native_tree(slf, factory, built)
    }

    /// `ArcBetweenPoints(start, end, angle)` over the native true-arc shelf.
    fn _build_arc_between_points<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        start: [f64; 3],
        end: [f64; 3],
        angle: f64,
        n_components: Option<usize>,
    ) -> PyResult<Bound<'py, PyList>> {
        let mut arc = fmn_library::ArcBetweenPoints::new(start, end).angle(angle);
        if let Some(n) = n_components {
            arc = arc.n_components(n);
        }
        let built = arc.build().map_err(native_error)?;
        install_native_tree(slf, factory, built)
    }

    /// The curved-arrow pair over Atlas's native tip-attachment algebra.
    fn _build_curved_arrow<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        start: [f64; 3],
        end: [f64; 3],
        angle: f64,
        n_components: Option<usize>,
        double: bool,
    ) -> PyResult<Bound<'py, PyList>> {
        let style = fmn_library::Style::default();
        let mut arc = fmn_library::ArcBetweenPoints::new(start, end)
            .angle(angle)
            .style(style);
        if let Some(n) = n_components {
            arc = arc.n_components(n);
        }
        let shaft = arc.build().map_err(native_error)?;
        let built = fmn_library::tip::attach_tip(
            shaft,
            fmn_library::ArrowTip::new(),
            fmn_library::TipEnd::End,
        );
        let built = if double {
            fmn_library::tip::attach_tip(
                built,
                fmn_library::ArrowTip::new(),
                fmn_library::TipEnd::Start,
            )
        } else {
            built
        };
        install_native_tree(slf, factory, built)
    }

    /// `Circle(start_angle, radius, arc_center)` — the native circle
    /// builder, keeping its semantic shape tag.
    #[pyo3(signature = (factory, start_angle, radius, arc_center, n_components=None))]
    fn _build_circle<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        start_angle: f64,
        radius: f64,
        arc_center: [f64; 3],
        n_components: Option<usize>,
    ) -> PyResult<Bound<'py, PyList>> {
        let circle = fmn_library::Circle::new()
            .start_angle(start_angle)
            .radius(radius)
            .arc_center(arc_center);
        let built = match n_components {
            Some(count) => circle.build_with_components(count).map_err(native_error)?,
            None => circle.build(),
        };
        install_native_tree(slf, factory, built)
    }

    /// `Ellipse(width, height, arc_center)` over the native stretched-circle
    /// builder. The builder deliberately demotes the circle shape hint.
    fn _build_ellipse<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        width: f64,
        height: f64,
        arc_center: [f64; 3],
        start_angle: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let built = fmn_library::Ellipse::new()
            .width(width)
            .height(height)
            .arc_center(arc_center)
            .start_angle(start_angle)
            .build();
        install_native_tree(slf, factory, built)
    }

    /// `AnnularSector` and `Sector` share Atlas's one concentric-arc
    /// implementation; `Sector` supplies `inner_radius = 0` in Python.
    #[allow(clippy::too_many_arguments)]
    fn _build_annular_sector<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        angle: f64,
        start_angle: f64,
        inner_radius: f64,
        outer_radius: f64,
        arc_center: [f64; 3],
    ) -> PyResult<Bound<'py, PyList>> {
        let built = fmn_library::AnnularSector::new()
            .angle(angle)
            .start_angle(start_angle)
            .inner_radius(inner_radius)
            .outer_radius(outer_radius)
            .arc_center(arc_center)
            .build()
            .map_err(native_error)?;
        install_native_tree(slf, factory, built)
    }

    /// `Annulus(inner_radius, outer_radius, center)` with a counter-wound
    /// inner contour for a real nonzero-winding hole.
    fn _build_annulus<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        inner_radius: f64,
        outer_radius: f64,
        center: [f64; 3],
    ) -> PyResult<Bound<'py, PyList>> {
        let built = fmn_library::Annulus::new()
            .inner_radius(inner_radius)
            .outer_radius(outer_radius)
            .center(center)
            .build();
        install_native_tree(slf, factory, built)
    }

    /// `Dot(point, radius)` — a filled disc with the Reference defaults.
    fn _build_dot<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        point: [f64; 3],
        radius: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let built = fmn_library::Dot::new().point(point).radius(radius).build();
        install_native_tree(slf, factory, built)
    }

    /// `PGroup(*pmobs)` over Atlas's point-cloud family builder. The native
    /// builder owns the root's PMobject record schema; Python grafts the
    /// already-live child proxies afterward so their identities are kept.
    fn _build_p_group<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyList>> {
        install_native_tree(slf, factory, fmn_library::p_group([]))
    }

    /// `DotCloud(points, color, opacity, radius, glow_factor,
    /// anti_alias_width)` over the pointcloud shelf — the DotCloud record
    /// schema (`point`/`radius`/`rgba`/`glow_factor`), not a VMobject.
    #[allow(clippy::too_many_arguments)]
    fn _build_dot_cloud<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        points: Vec<[f64; 3]>,
        color: Option<&Bound<'py, PyAny>>,
        opacity: f64,
        radius: f64,
        glow_factor: f64,
        anti_alias_width: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let mut cloud = fmn_library::DotCloud::new(points)
            .with_radius(radius)
            .with_glow_factor(glow_factor)
            .with_anti_alias_width(anti_alias_width);
        if let Some(color) = color {
            cloud = cloud.colored(srgb_from_py(color)?, opacity);
        } else if opacity != 1.0 {
            cloud = cloud.colored(
                fmn_core::color::Srgb::from_hex("#888888").expect("grey"),
                opacity,
            );
        }
        install_native_tree(slf, factory, cloud)
    }

    /// `ImageMobject(path, height)`: Python owns local path resolution, then
    /// Atlas/fmn-codec own format sniffing, bounded decode, quad geometry,
    /// and the immutable renderer resource. No decoded pixel copy returns to
    /// Python and no second image implementation exists in the portal.
    fn _build_image<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        payload: &Bound<'py, PyBytes>,
        height: f64,
        opacity: f64,
        z_index: i32,
    ) -> PyResult<Bound<'py, PyList>> {
        let image = fmn_library::ImageMobject::from_bytes(payload.as_bytes())
            .map_err(native_error)?
            .with_height(height)
            .with_opacity(opacity)
            .with_z_index(z_index);
        install_native_tree(slf, factory, image)
    }

    /// `ThreeDModel(obj_file, height)`: Atlas owns the bounded OBJ-subset
    /// parser, normal resolution, centering, height normalization, and the
    /// lit triangle-mesh records. The public Reference object is a Group, so
    /// the native mesh is returned as its one Surface child rather than
    /// installing point records on the group shell itself.
    fn _build_three_d_model<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        payload: &Bound<'py, PyBytes>,
        height: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let model = fmn_library::ThreeDModel::from_obj(payload.as_bytes())
            .map_err(native_error)?
            .with_height(height);
        native_shell_specs(slf.py(), factory, vec![model.into()])
    }

    /// Sample the durable image attached to this entry using the Reference's
    /// axis-aligned family-box convention. Placement is baked first, so the
    /// result observes live shifts/scales exactly as ordinary point reads do.
    fn _image_point_to_rgb(slf: &Bound<'_, Self>, point: [f64; 3]) -> PyResult<Option<[f64; 3]>> {
        crossing::record(CrossingClass::Other);
        with_stage(slf, |stage, mob| -> PyResult<Option<[f64; 3]>> {
            stage.bake_placement(mob).map_err(stage_error)?;
            let entry = stage.get(mob).ok_or_else(|| {
                StaleHandleError::new_err("image mobject handle no longer resolves")
            })?;
            let image = entry.image_resource().ok_or_else(|| {
                PyRuntimeError::new_err("image mobject has no durable image resource")
            })?;
            let points = entry
                .buffer
                .read_column("point")
                .ok_or_else(|| PyRuntimeError::new_err("image mobject has no point field"))?;
            let mut min_x = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;
            let mut min_y = f64::INFINITY;
            let mut max_y = f64::NEG_INFINITY;
            for row in points.as_chunks::<3>().0 {
                min_x = min_x.min(f64::from(row[0]));
                max_x = max_x.max(f64::from(row[0]));
                min_y = min_y.min(f64::from(row[1]));
                max_y = max_y.max(f64::from(row[1]));
            }
            let width = max_x - min_x;
            let height = max_y - min_y;
            if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
                return Ok(None);
            }
            let x_alpha = (point[0] - min_x) / width;
            let y_alpha = (max_y - point[1]) / height;
            if !(0.0..=1.0).contains(&x_alpha) || !(0.0..=1.0).contains(&y_alpha) {
                return Ok(None);
            }
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss
            )]
            let pixel_x = (f64::from(image.width() - 1) * x_alpha) as usize;
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss
            )]
            let pixel_y = (f64::from(image.height() - 1) * y_alpha) as usize;
            let offset = pixel_y
                .checked_mul(image.width() as usize)
                .and_then(|row| row.checked_add(pixel_x))
                .and_then(|pixel| pixel.checked_mul(4))
                .ok_or_else(|| PyOverflowError::new_err("image sample offset overflows usize"))?;
            let rgb = image.pixels().get(offset..offset + 3).ok_or_else(|| {
                PyRuntimeError::new_err("image sample falls outside the durable resource")
            })?;
            Ok(Some([
                f64::from(rgb[0]) / 255.0,
                f64::from(rgb[1]) / 255.0,
                f64::from(rgb[2]) / 255.0,
            ]))
        })?
    }

    /// Pixel dimensions of the same durable resource consumed by Lumen.
    fn _image_dimensions(slf: &Bound<'_, Self>) -> PyResult<(u32, u32)> {
        crossing::record(CrossingClass::Other);
        with_stage(slf, |stage, mob| -> PyResult<(u32, u32)> {
            let entry = stage.get(mob).ok_or_else(|| {
                StaleHandleError::new_err("image mobject handle no longer resolves")
            })?;
            let image = entry.image_resource().ok_or_else(|| {
                PyRuntimeError::new_err("image mobject has no durable image resource")
            })?;
            Ok((image.width(), image.height()))
        })?
    }

    /// `Prism(width, height, depth)` over the solids shelf: six sampled
    /// quads as an SGroup family.
    fn _build_prism<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        width: f64,
        height: f64,
        depth: f64,
        z_index: i32,
    ) -> PyResult<Bound<'py, PyList>> {
        let prism = fmn_library::Prism::new(width, height, depth)
            .build()
            .with_z_index(z_index);
        install_native_tree(slf, factory, prism)
    }

    /// `Cube(side_length)` over the solids shelf.
    fn _build_cube<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        side_length: f64,
        square_resolution: (usize, usize),
        z_index: i32,
    ) -> PyResult<Bound<'py, PyList>> {
        let cube = fmn_library::Cube::new(side_length)
            .square_resolution(square_resolution.0, square_resolution.1)
            .build()
            .with_z_index(z_index);
        install_native_tree(slf, factory, cube)
    }

    /// `VCube(side_length, ...)` over Atlas's six vectorized faces. Style
    /// is part of the native build so the first observable Python state is
    /// already the Reference's filled, unstroked cube.
    fn _build_vcube<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        side_length: f64,
        fill_color: &Bound<'_, PyAny>,
        fill_opacity: f64,
        stroke_width: f64,
        z_index: i32,
    ) -> PyResult<Bound<'py, PyList>> {
        let cube = fmn_library::VCube::new(side_length)
            .fill_color(srgb_from_py(fill_color)?)
            .fill_opacity(fill_opacity)
            .stroke_width(stroke_width)
            .build()
            .with_z_index(z_index);
        install_native_tree(slf, factory, cube)
    }

    /// `VPrism(width, height, depth)` over Atlas's vectorized cube stretch.
    #[allow(clippy::too_many_arguments)]
    fn _build_vprism<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        width: f64,
        height: f64,
        depth: f64,
        fill_color: &Bound<'_, PyAny>,
        fill_opacity: f64,
        stroke_width: f64,
        z_index: i32,
    ) -> PyResult<Bound<'py, PyList>> {
        let prism = fmn_library::VPrism::new(width, height, depth)
            .fill_color(srgb_from_py(fill_color)?)
            .fill_opacity(fill_opacity)
            .stroke_width(stroke_width)
            .build()
            .with_z_index(z_index);
        install_native_tree(slf, factory, prism)
    }

    /// The twelve native pentagons of the Reference `Dodecahedron`.
    #[allow(clippy::too_many_arguments)]
    fn _build_dodecahedron<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        fill_color: &Bound<'_, PyAny>,
        fill_opacity: f64,
        stroke_color: &Bound<'_, PyAny>,
        stroke_width: f64,
        shading: [f64; 3],
        z_index: i32,
    ) -> PyResult<Bound<'py, PyList>> {
        let solid = fmn_library::Dodecahedron::new()
            .fill_color(srgb_from_py(fill_color)?)
            .fill_opacity(fill_opacity)
            .stroke_color(srgb_from_py(stroke_color)?)
            .stroke_width(stroke_width)
            .shading(shading)
            .build()
            .with_z_index(z_index);
        install_native_tree(slf, factory, solid)
    }

    /// `Prismify(vmobject, depth, direction)`: Atlas owns the straight-edge
    /// extrusion geometry. The bootstrap reapplies the source's live style
    /// arrays to the returned base/walls/top, matching the Reference's
    /// `copy`/`match_style` construction without a second point kernel.
    fn _build_prismify<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        source: &Bound<'_, BridgeMobject>,
        depth: f64,
        direction: [f64; 3],
    ) -> PyResult<Bound<'py, PyList>> {
        let points = with_stage(source, |stage, mob| {
            stage.get_points(mob).unwrap_or_default()
        })?;
        let source = fmn_library::VMobject::from_points(points);
        let prism = fmn_library::Prismify::new(source)
            .depth(depth)
            .direction(direction);
        install_native_tree(slf, factory, prism)
    }

    /// `Prismify` over a VMobject FAMILY (fm-5wq.4.81): one native
    /// extrusion tree per pointful family member, in family order, hung
    /// under one group root — the same builder, composed per child.
    fn _build_prismify_family<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        source: &Bound<'_, BridgeMobject>,
        depth: f64,
        direction: [f64; 3],
    ) -> PyResult<Bound<'py, PyList>> {
        let mut point_sets = Vec::new();
        for object in collect_proxy_graph(source.as_any())? {
            let proxy = object.bind(source.py()).cast::<BridgeMobject>()?;
            let points = with_stage(proxy, |stage, mob| {
                stage.get_points(mob).unwrap_or_default()
            })?;
            if !points.is_empty() {
                point_sets.push(points);
            }
        }
        if point_sets.is_empty() {
            return Err(PyValueError::new_err(
                "Prismify family has no pointful members to extrude",
            ));
        }
        let mut root =
            fmn_mobject::Mobject::from(fmn_library::vmobject::v_group(std::iter::empty::<
                fmn_library::VMobject,
            >()));
        for points in point_sets {
            let prism = fmn_library::Prismify::new(fmn_library::VMobject::from_points(points))
                .depth(depth)
                .direction(direction);
            root.submobjects.push(prism.into());
        }
        install_native_tree(slf, factory, root)
    }

    /// `Sphere(radius, ...)` over the solids shelf: the Reference's UV
    /// grid with radial true normals.
    #[allow(clippy::too_many_arguments)]
    fn _build_sphere<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        radius: f64,
        u_range: (f64, f64),
        v_range: (f64, f64),
        resolution: (usize, usize),
        true_normals: bool,
        clockwise: bool,
        preferred_creation_axis: usize,
        epsilon: f64,
        normal_nudge: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let sphere = fmn_library::Sphere::new(radius)
            .u_range(u_range.0, u_range.1)
            .v_range(v_range.0, v_range.1)
            .resolution(resolution.0, resolution.1)
            .true_normals(true_normals)
            .clockwise(clockwise)
            .preferred_creation_axis(preferred_creation_axis)
            .epsilon(epsilon)
            .normal_nudge(normal_nudge);
        install_native_tree(slf, factory, sphere.build())
    }

    /// `Cylinder(height, radius, axis, ...)` over the solids shelf: the
    /// Reference UV grid followed by its radius/depth/axis transform.
    #[allow(clippy::too_many_arguments)]
    fn _build_cylinder<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        height: f64,
        radius: f64,
        axis: [f64; 3],
        u_range: (f64, f64),
        v_range: (f64, f64),
        resolution: (usize, usize),
        preferred_creation_axis: usize,
        epsilon: f64,
        normal_nudge: f64,
        z_index: i32,
    ) -> PyResult<Bound<'py, PyList>> {
        let cylinder = fmn_library::Cylinder::new(height, radius)
            .axis(axis)
            .u_range(u_range.0, u_range.1)
            .v_range(v_range.0, v_range.1)
            .resolution(resolution.0, resolution.1)
            .preferred_creation_axis(preferred_creation_axis)
            .epsilon(epsilon)
            .normal_nudge(normal_nudge)
            .build()
            .with_z_index(z_index);
        install_native_tree(slf, factory, cylinder)
    }

    /// `Torus(r1, r2, ...)` over Atlas's native UV-grid sampler.
    #[allow(clippy::too_many_arguments)]
    fn _build_torus<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        r1: f64,
        r2: f64,
        u_range: (f64, f64),
        v_range: (f64, f64),
        resolution: (usize, usize),
        preferred_creation_axis: usize,
        epsilon: f64,
        normal_nudge: f64,
        z_index: i32,
    ) -> PyResult<Bound<'py, PyList>> {
        let torus = fmn_library::Torus::new(r1, r2)
            .u_range(u_range.0, u_range.1)
            .v_range(v_range.0, v_range.1)
            .resolution(resolution.0, resolution.1)
            .preferred_creation_axis(preferred_creation_axis)
            .epsilon(epsilon)
            .normal_nudge(normal_nudge)
            .build()
            .with_z_index(z_index);
        install_native_tree(slf, factory, torus)
    }

    /// `Cone(height, radius, axis, ...)` over Atlas's native tapered
    /// cylinder, preserving the Reference's non-centered `v = (0, 1)` grid.
    #[allow(clippy::too_many_arguments)]
    fn _build_cone<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        height: f64,
        radius: f64,
        axis: [f64; 3],
        u_range: (f64, f64),
        v_range: (f64, f64),
        resolution: (usize, usize),
        preferred_creation_axis: usize,
        epsilon: f64,
        normal_nudge: f64,
        z_index: i32,
    ) -> PyResult<Bound<'py, PyList>> {
        let cone = fmn_library::Cone::new(height, radius)
            .axis(axis)
            .u_range(u_range.0, u_range.1)
            .v_range(v_range.0, v_range.1)
            .resolution(resolution.0, resolution.1)
            .preferred_creation_axis(preferred_creation_axis)
            .epsilon(epsilon)
            .normal_nudge(normal_nudge)
            .build()
            .with_z_index(z_index);
        install_native_tree(slf, factory, cone)
    }

    /// `Line3D(start, end, width, ...)` over Atlas's thin-cylinder builder.
    #[allow(clippy::too_many_arguments)]
    fn _build_line3d<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        start: [f64; 3],
        end: [f64; 3],
        width: f64,
        u_range: (f64, f64),
        v_range: (f64, f64),
        resolution: (usize, usize),
        preferred_creation_axis: usize,
        epsilon: f64,
        normal_nudge: f64,
        z_index: i32,
    ) -> PyResult<Bound<'py, PyList>> {
        let line = fmn_library::Line3D::new(start, end)
            .width(width)
            .u_range(u_range.0, u_range.1)
            .v_range(v_range.0, v_range.1)
            .resolution(resolution.0, resolution.1)
            .preferred_creation_axis(preferred_creation_axis)
            .epsilon(epsilon)
            .normal_nudge(normal_nudge)
            .build()
            .with_z_index(z_index);
        install_native_tree(slf, factory, line)
    }

    /// `Disk3D(radius, ...)` over Atlas's native polar surface grid.
    #[allow(clippy::too_many_arguments)]
    fn _build_disk3d<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        radius: f64,
        u_range: (f64, f64),
        v_range: (f64, f64),
        resolution: (usize, usize),
        preferred_creation_axis: usize,
        epsilon: f64,
        normal_nudge: f64,
        z_index: i32,
    ) -> PyResult<Bound<'py, PyList>> {
        let disk = fmn_library::Disk3D::new(radius)
            .u_range(u_range.0, u_range.1)
            .v_range(v_range.0, v_range.1)
            .resolution(resolution.0, resolution.1)
            .preferred_creation_axis(preferred_creation_axis)
            .epsilon(epsilon)
            .normal_nudge(normal_nudge)
            .build()
            .with_z_index(z_index);
        install_native_tree(slf, factory, disk)
    }

    /// `Square3D(side_length, ...)` over Atlas's native planar surface grid.
    #[allow(clippy::too_many_arguments)]
    fn _build_square3d<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        side_length: f64,
        u_range: (f64, f64),
        v_range: (f64, f64),
        resolution: (usize, usize),
        preferred_creation_axis: usize,
        epsilon: f64,
        normal_nudge: f64,
        z_index: i32,
    ) -> PyResult<Bound<'py, PyList>> {
        let square = fmn_library::Square3D::new(side_length)
            .u_range(u_range.0, u_range.1)
            .v_range(v_range.0, v_range.1)
            .resolution(resolution.0, resolution.1)
            .preferred_creation_axis(preferred_creation_axis)
            .epsilon(epsilon)
            .normal_nudge(normal_nudge)
            .build()
            .with_z_index(z_index);
        install_native_tree(slf, factory, square)
    }

    /// `Cylinder.uv_func` through the exact object-space parameterization
    /// sampled by the native builder.
    #[staticmethod]
    fn _cylinder_uv(u: f64, v: f64) -> [f64; 3] {
        fmn_library::Cylinder::uv_func(u, v)
    }

    /// `Torus.uv_func` through the exact native function sampled by build.
    #[staticmethod]
    fn _torus_uv(r1: f64, r2: f64, u: f64, v: f64) -> [f64; 3] {
        fmn_library::Torus::new(r1, r2).uv_func(u, v)
    }

    /// `Cone.uv_func` through the exact native function sampled by build.
    #[staticmethod]
    fn _cone_uv(u: f64, v: f64) -> [f64; 3] {
        fmn_library::Cone::uv_func(u, v)
    }

    /// `Disk3D.uv_func` through the exact native function sampled by build.
    #[staticmethod]
    fn _disk3d_uv(u: f64, v: f64) -> [f64; 3] {
        fmn_library::Disk3D::uv_func(u, v)
    }

    /// `Square3D.uv_func` through the exact native function sampled by build.
    #[staticmethod]
    fn _square3d_uv(u: f64, v: f64) -> [f64; 3] {
        fmn_library::Square3D::uv_func(u, v)
    }

    /// `Sphere.uv_func` through the exact function used by the native
    /// surface builder (including its fmn-dmath transcendental path).
    #[staticmethod]
    fn _sphere_uv(radius: f64, clockwise: bool, u: f64, v: f64) -> [f64; 3] {
        fmn_library::Sphere::new(radius)
            .clockwise(clockwise)
            .uv_func(u, v)
    }

    /// `ParametricSurface(uv_func, ...)`: the native sampler over a
    /// Python callable. The callable runs during construction only (no
    /// engine borrow is held); its first error aborts the build.
    // The argument count mirrors the Reference ParametricSurface constructor
    // surface one-to-one; collapsing it would change the Python-visible API.
    #[allow(clippy::too_many_arguments)]
    fn _build_parametric_surface<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        uv_func: &Bound<'py, PyAny>,
        u_range: (f64, f64),
        v_range: (f64, f64),
        resolution: (usize, usize),
        epsilon: f64,
        normal_nudge: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        if slf.borrow().engine.is_some() {
            return Err(PyRuntimeError::new_err(
                "a native surface builder requires a detached target",
            ));
        }
        let spec = fmn_library::solids::SurfaceSpec {
            u_range,
            v_range,
            resolution,
            epsilon,
            normal_nudge,
            ..fmn_library::solids::SurfaceSpec::default()
        };
        // No engine borrow spans user code. The fallible native loop returns
        // immediately, without sentinel probes or constructing a doomed grid.
        // Fixed-array extraction also prevents an unbounded result Vec allocation.
        let surface = spec
            .try_sample(|u, v| uv_func.call1((u, v))?.extract::<[f64; 3]>())
            .map_err(|error| match error {
                fmn_library::solids::SurfaceSampleError::Callback(error) => error,
                other => PyValueError::new_err(other.to_string()),
            })?;
        install_native_tree(slf, factory, surface)
    }

    /// `ParametricCurve(t_func, ...)`: Atlas owns the bounded range
    /// sampling and Chisel owns the shared-anchor smoothing. The Python
    /// callback is evaluated only while constructing the detached value;
    /// its first exception is preserved verbatim.
    fn _build_parametric_curve<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        t_func: &Bound<'py, PyAny>,
        t_range: &Bound<'py, PyAny>,
        epsilon: f64,
        discontinuities: Vec<f64>,
        use_smoothing: bool,
    ) -> PyResult<Bound<'py, PyList>> {
        if slf.borrow().engine.is_some() {
            return Err(PyRuntimeError::new_err(
                "a native graph builder requires a detached target",
            ));
        }
        let spec = fmn_library::graphs::ParametricCurveSpec {
            t_range: range3(t_range)?,
            epsilon,
            discontinuities,
            use_smoothing,
            ..fmn_library::graphs::ParametricCurveSpec::default()
        };
        let curve = spec
            .try_sample(|t| t_func.call1((t,))?.extract::<[f64; 3]>())
            .map_err(graph_sampling_error)?;
        install_native_tree(slf, factory, curve)
    }

    /// `FunctionGraph(function, ...)`: Atlas owns the bounded scalar graph
    /// sampling while Python retains the caller-visible function metadata.
    /// The callback is construction-only and its first exception crosses the
    /// portal unchanged.
    fn _build_function_graph<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        function: &Bound<'py, PyAny>,
        x_range: &Bound<'py, PyAny>,
        epsilon: f64,
        discontinuities: Vec<f64>,
        use_smoothing: bool,
    ) -> PyResult<Bound<'py, PyList>> {
        if slf.borrow().engine.is_some() {
            return Err(PyRuntimeError::new_err(
                "a native graph builder requires a detached target",
            ));
        }
        let spec = fmn_library::graphs::ParametricCurveSpec {
            t_range: range3(x_range)?,
            epsilon,
            discontinuities,
            use_smoothing,
            style: fmn_library::Style::default().color(fmn_core::constants::YELLOW),
            ..fmn_library::graphs::ParametricCurveSpec::default()
        };
        let graph = spec
            .try_sample(|x| Ok([x, function.call1((x,))?.extract::<f64>()?, 0.0]))
            .map_err(graph_sampling_error)?;
        install_native_tree(slf, factory, graph)
    }

    /// `ImplicitFunction(func, ...)`: Chisel extracts the bounded zero set
    /// and Atlas materializes its shared-anchor path. A callback error terminates
    /// the actual native traversal immediately; a returned NaN still means an
    /// undefined region, not cancellation.
    #[allow(clippy::too_many_arguments)]
    fn _build_implicit_function<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        func: &Bound<'py, PyAny>,
        x_range: (f64, f64),
        y_range: (f64, f64),
        min_depth: u32,
        max_quads: usize,
        use_smoothing: bool,
    ) -> PyResult<Bound<'py, PyList>> {
        if slf.borrow().engine.is_some() {
            return Err(PyRuntimeError::new_err(
                "a native graph builder requires a detached target",
            ));
        }
        let mut spec = fmn_library::graphs::ImplicitFunctionSpec {
            x_range: [x_range.0, x_range.1],
            y_range: [y_range.0, y_range.1],
            use_smoothing,
            ..fmn_library::graphs::ImplicitFunctionSpec::default()
        };
        spec.config.min_depth = min_depth;
        spec.config.max_quads = max_quads;
        let implicit = spec
            .try_sample(|x, y| func.call1((x, y))?.extract::<f64>())
            .map_err(graph_sampling_error)?;
        install_native_tree(slf, factory, implicit)
    }

    /// Build the native VectorField geometry from already-evaluated portal
    /// samples. Python owns callback dispatch so it can release the Scene
    /// borrow; Atlas/Lumen still own arrow geometry, tanh length mapping,
    /// joint policy, and per-record stroke columns.
    #[allow(clippy::too_many_arguments)]
    fn _build_vector_field_samples<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        sample_points: Vec<[f64; 3]>,
        out_vects: Vec<[f64; 3]>,
        output_norms: Vec<f64>,
        max_displayed_vect_len: f64,
        stroke_width: f64,
        stroke_opacity: f64,
        tip_width_ratio: f64,
        tip_len_to_width: f64,
        flat_stroke: bool,
        use_default_color_map: bool,
        color: Option<&Bound<'py, PyAny>>,
        magnitude_range: Option<(f64, f64)>,
    ) -> PyResult<Bound<'py, PyList>> {
        let n = sample_points.len();
        if n < 2 {
            return Err(PyValueError::new_err(
                "VectorField needs at least two sample points",
            ));
        }
        let sample_budget = fmn_library::SamplingBudget::DEFAULT.max_samples();
        if n > sample_budget {
            return Err(PyValueError::new_err(format!(
                "VectorField sample grid exceeds the {sample_budget}-point resource budget"
            )));
        }
        if out_vects.len() != n || output_norms.len() != n {
            return Err(PyValueError::new_err(format!(
                "VectorField callback returned {} vectors and {} norms for {n} samples",
                out_vects.len(),
                output_norms.len()
            )));
        }
        if max_displayed_vect_len.is_nan() || max_displayed_vect_len <= 0.0 {
            return Err(PyValueError::new_err(
                "VectorField max displayed length must be positive and non-NaN",
            ));
        }
        if [
            stroke_width,
            stroke_opacity,
            tip_width_ratio,
            tip_len_to_width,
        ]
        .iter()
        .any(|value| !value.is_finite())
        {
            return Err(PyValueError::new_err(
                "VectorField style controls must be finite",
            ));
        }

        let mut style = fmn_library::VectorFieldStyle {
            stroke_width,
            stroke_opacity,
            tip_width_ratio,
            tip_len_to_width,
            flat_stroke,
            magnitude_range: Some(
                magnitude_range
                    .unwrap_or_else(|| (0.0, output_norms.iter().copied().fold(0.0, f64::max))),
            ),
            ..fmn_library::VectorFieldStyle::default()
        };
        if !use_default_color_map {
            style.color_map = None;
        }
        if let Some(color) = color {
            style.color = Some(srgb_from_py(color)?);
            style.color_map = None;
        }
        let geometry = fmn_library::fields::vector_field_geometry(
            &style,
            &sample_points,
            &out_vects,
            &output_norms,
            max_displayed_vect_len,
        );
        let stroke_color = style
            .color
            .unwrap_or(fmn_core::constants::DEFAULT_MOBJECT_COLOR);
        let mut visual_style = fmn_library::Style::default().stroke(
            stroke_color,
            style.stroke_width,
            style.stroke_opacity,
        );
        visual_style.fill_opacity = 0.0;
        let vmob = fmn_library::VMobject::from_points(geometry.points)
            .with_style(visual_style)
            .with_joint_type(fmn_mobject::JointType::NoJoint)
            .with_flat_stroke(style.flat_stroke)
            .with_stroke_profile(geometry.stroke_widths);
        let mut tree = fmn_mobject::Mobject::from(vmob);
        if !geometry.stroke_rgba.is_empty() {
            #[allow(clippy::cast_possible_truncation)]
            let rgba: Vec<f32> = geometry
                .stroke_rgba
                .iter()
                .flat_map(|row| row.iter().map(|value| *value as f32))
                .collect();
            tree.buffer.write_range("stroke_rgba", 0, &rgba);
        }
        install_native_tree(slf, factory, tree)
    }

    /// `StreamLines` (fm-5wq.4.74): run the ONE native RK45 stream-line
    /// integrator over a construction-time Python field callback and a
    /// Python coordinate-system adapter, then hang the built line family.
    /// Returns `(shell_specs, virtual_times)` — one virtual time per line,
    /// which `AnimatedStreamLines` paces its flash windows by.
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (factory, func, c2p, p2c, ranges, dimension, seed, density,
        n_repeats, noise_factor, solution_time, dt, arc_len, max_time_steps,
        n_samples_per_line, cutoff_norm, stroke_width, stroke_color, stroke_opacity,
        color_by_magnitude, magnitude_range, taper_stroke_width, sample_coords=None))]
    fn _build_stream_lines<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        func: Py<PyAny>,
        c2p: Py<PyAny>,
        p2c: Py<PyAny>,
        ranges: Vec<[f64; 3]>,
        dimension: usize,
        seed: u64,
        density: f64,
        n_repeats: usize,
        noise_factor: Option<f64>,
        solution_time: f64,
        dt: f64,
        arc_len: f64,
        max_time_steps: usize,
        n_samples_per_line: usize,
        cutoff_norm: f64,
        stroke_width: f64,
        stroke_color: Option<&Bound<'py, PyAny>>,
        stroke_opacity: f64,
        color_by_magnitude: bool,
        magnitude_range: (f64, f64),
        taper_stroke_width: bool,
        sample_coords: Option<Vec<[f64; 3]>>,
    ) -> PyResult<(Bound<'py, PyList>, Vec<f64>, u64)> {
        let py = slf.py();
        let callback_error: Arc<Mutex<Option<PyErr>>> = Arc::new(Mutex::new(None));

        struct PyCoordinateSystem {
            c2p: Py<PyAny>,
            p2c: Py<PyAny>,
            ranges: Vec<[f64; 3]>,
            dimension: usize,
            error: Arc<Mutex<Option<PyErr>>>,
        }
        impl fmn_library::coords::CoordinateSystem for PyCoordinateSystem {
            fn c2p(&self, coords: &[f64]) -> fmn_core::types::Vec3 {
                if self.error.lock().unwrap().is_some() {
                    return [0.0; 3];
                }
                Python::attach(|py| {
                    match self
                        .c2p
                        .bind(py)
                        .call1((coords.to_vec(),))
                        .and_then(|value| value.extract::<[f64; 3]>())
                    {
                        Ok(point) => point,
                        Err(error) => {
                            self.error.lock().unwrap().get_or_insert(error);
                            [0.0; 3]
                        }
                    }
                })
            }
            fn p2c(&self, point: fmn_core::types::Vec3) -> [f64; 3] {
                if self.error.lock().unwrap().is_some() {
                    return [0.0; 3];
                }
                Python::attach(|py| {
                    match self
                        .p2c
                        .bind(py)
                        .call1((point,))
                        .and_then(|value| value.extract::<[f64; 3]>())
                    {
                        Ok(coords) => coords,
                        Err(error) => {
                            self.error.lock().unwrap().get_or_insert(error);
                            [0.0; 3]
                        }
                    }
                })
            }
            fn all_ranges(&self) -> Vec<[f64; 3]> {
                self.ranges.clone()
            }
            fn dimension(&self) -> usize {
                self.dimension
            }
        }

        let adapter = PyCoordinateSystem {
            c2p,
            p2c,
            ranges,
            dimension,
            error: Arc::clone(&callback_error),
        };
        let field_error = Arc::clone(&callback_error);
        let field = move |rows: &[[f64; 3]]| -> Vec<[f64; 3]> {
            if field_error.lock().unwrap().is_some() {
                return vec![[0.0; 3]; rows.len()];
            }
            Python::attach(|py| {
                match func
                    .bind(py)
                    .call1((rows.to_vec(),))
                    .and_then(|value| value.extract::<Vec<[f64; 3]>>())
                {
                    Ok(outputs) if outputs.len() == rows.len() => outputs,
                    Ok(outputs) => {
                        field_error.lock().unwrap().get_or_insert_with(|| {
                            PyValueError::new_err(format!(
                                "StreamLines func must return one vector per sample; \
                                 got {} vectors for {} samples",
                                outputs.len(),
                                rows.len()
                            ))
                        });
                        vec![[0.0; 3]; rows.len()]
                    }
                    Err(error) => {
                        field_error.lock().unwrap().get_or_insert(error);
                        vec![[0.0; 3]; rows.len()]
                    }
                }
            })
        };

        let style = fmn_library::fields::StreamLineStyle {
            stroke_width,
            stroke_color: match stroke_color {
                Some(color) => srgb_from_py(color)?,
                None => fmn_core::constants::DEFAULT_MOBJECT_COLOR,
            },
            stroke_opacity,
            color_by_magnitude,
            magnitude_range,
            taper_stroke_width,
            ..Default::default()
        };

        let rng = fmn_core::rng::RngRoot::from_seed(seed);
        let mut builder = fmn_library::fields::StreamLines::new(field, adapter, &rng)
            .with_density(density)
            .with_n_repeats(n_repeats)
            .with_solution_time(solution_time)
            .with_dt(dt)
            .with_arc_len(arc_len)
            .with_max_time_steps(max_time_steps)
            .with_samples_per_line(n_samples_per_line)
            .with_cutoff_norm(cutoff_norm)
            .with_style_config(style);
        if let Some(noise) = noise_factor {
            builder = builder.with_noise_factor(noise);
        }
        if let Some(coords) = sample_coords {
            builder = builder.with_sample_coords(coords);
        }
        // VERBATIM refusal doctrine: the field error, if any, outranks the
        // build result — a poisoned integration must never look successful.
        let built = builder.build();
        if let Some(error) = callback_error.lock().unwrap().take() {
            return Err(error);
        }
        let built = built.map_err(|error| {
            PyValueError::new_err(format!("StreamLines integration refused: {error}"))
        })?;
        let virtual_times: Vec<f64> = built.lines().iter().map(|line| line.virtual_time).collect();
        let rng_draws = built.rng_draws();
        let tree = fmn_mobject::Mobject::from(built.vmob().clone());
        let specs = install_native_tree(slf, factory, tree)?;
        let _ = py;
        Ok((specs, virtual_times, rng_draws))
    }

    /// `AnimatedStreamLines` lag times (vector_field.py:466): `count`
    /// U[0,1) draws from the named stream-lines substream, advanced past
    /// the seed jitter's `prior_draws` — the one continuous deterministic
    /// sequence the native updater uses.
    #[staticmethod]
    fn _stream_line_lag_uniforms(seed: u64, prior_draws: u64, count: usize) -> Vec<f64> {
        let rng = fmn_core::rng::RngRoot::from_seed(seed);
        let substream = rng.substream(fmn_library::fields::STREAM_LINES_SUBSTREAM);
        let mut sequential = substream.sequential();
        for _ in 0..prior_draws {
            let _ = sequential.next_f64();
        }
        (0..count).map(|_| sequential.next_f64()).collect()
    }

    /// The finished Atlas implementation of the Reference's older
    /// `VectorField.get_sample_points` rectilinear helper.
    #[staticmethod]
    #[allow(clippy::too_many_arguments)]
    fn _vector_field_grid_sample_points(
        center: [f64; 3],
        width: f64,
        height: f64,
        depth: f64,
        x_density: f64,
        y_density: f64,
        z_density: f64,
    ) -> PyResult<Vec<[f64; 3]>> {
        if [x_density, y_density, z_density]
            .iter()
            .any(|density| !density.is_finite() || *density <= 0.0)
        {
            return Err(PyValueError::new_err(
                "VectorField sample-point densities must be positive and finite",
            ));
        }
        let points = fmn_library::fields::grid_sample_points(
            center,
            [width / 2.0, height / 2.0, depth / 2.0],
            [x_density, y_density, z_density],
            fmn_library::SamplingBudget::DEFAULT,
        );
        if points.is_empty() && width >= 0.0 && height >= 0.0 && depth >= 0.0 {
            return Err(PyValueError::new_err(format!(
                "VectorField sample-point grid exceeds the {}-point resource budget",
                fmn_library::SamplingBudget::DEFAULT.max_samples()
            )));
        }
        Ok(points)
    }

    /// The native 3b1b colormap interpolation used by the field helpers.
    #[staticmethod]
    fn _vector_field_gradient(min_value: f64, max_value: f64, values: Vec<f64>) -> Vec<[f64; 3]> {
        fmn_library::fields::colormap_gradient(
            &fmn_core::constants::COLORMAP_3B1B,
            min_value,
            max_value,
            &values,
        )
        .into_iter()
        .map(|color| [color.r, color.g, color.b])
        .collect()
    }

    /// `SurfaceMesh(uv_surface, ...)` over the source's live sampled grid.
    /// Read authored accessors without an arena borrow, then give one owned
    /// snapshot to Atlas. Never re-evaluate the source's original UV recipe.
    #[allow(clippy::too_many_arguments)]
    fn _build_surface_mesh<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        source: &Bound<'py, PyAny>,
        resolution: (usize, usize),
        normal_nudge: f64,
        stroke_width: f64,
        stroke_color: Option<&Bound<'py, PyAny>>,
        depth_test: bool,
        joint_code: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let source_resolution = source.getattr("resolution")?.extract::<(usize, usize)>()?;
        // The source is a dense UV chart; Atlas bounds the wire output itself.
        let budget = fmn_library::SURFACE_SAMPLING_BUDGET.max_samples();
        let count = source_resolution
            .0
            .checked_mul(source_resolution.1)
            .filter(|&count| {
                count <= budget && source_resolution.0 <= budget && source_resolution.1 <= budget
            })
            .ok_or_else(|| {
                PyValueError::new_err("SurfaceMesh source exceeds the Atlas sampling budget")
            })?;
        let points = source.call_method0("get_points")?;
        if points.len()? != count {
            return Err(PyValueError::new_err(
                "SurfaceMesh source resolution does not match its point records",
            ));
        }
        let points = points.extract::<Vec<[f64; 3]>>()?;
        let normals = source.call_method0("get_unit_normals")?;
        if normals.len()? != count {
            return Err(PyValueError::new_err(
                "SurfaceMesh source resolution does not match its normal records",
            ));
        }
        let normals = normals.extract::<Vec<[f64; 3]>>()?;
        let mut mesh = fmn_library::SurfaceMesh::from_samples(points, normals, source_resolution)
            .map_err(|error| PyValueError::new_err(error.to_string()))?
            .resolution(resolution.0, resolution.1)
            .normal_nudge(normal_nudge)
            .stroke_width(stroke_width)
            .depth_test(depth_test)
            .joint_type(fmn_mobject::JointType::from_code(joint_code));
        if let Some(color) = stroke_color {
            mesh = mesh.stroke_color(srgb_from_py(color)?);
        }
        let mesh = mesh
            .try_build()
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        install_native_tree(slf, factory, mesh)
    }

    /// `DecimalNumber(number, ...)` over the numbers shelf (the de-TeX'd
    /// native builder with glyph-recycling updates).
    #[allow(clippy::too_many_arguments)]
    fn _build_decimal_number<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        number: f64,
        num_decimal_places: usize,
        min_total_width: usize,
        include_sign: bool,
        group_with_commas: bool,
        digit_buff_per_font_unit: f64,
        show_ellipsis: bool,
        unit: Option<String>,
        include_background_rectangle: bool,
        edge_to_fix: [f64; 3],
        font_size: f64,
        color: Option<&Bound<'py, PyAny>>,
        stroke_width: f64,
        fill_opacity: f64,
        fill_border_width: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let mut decimal = fmn_library::DecimalNumber::new(number)
            .num_decimal_places(num_decimal_places)
            .min_total_width(min_total_width)
            .include_sign(include_sign)
            .group_with_commas(group_with_commas)
            .digit_buff_per_font_unit(digit_buff_per_font_unit)
            .show_ellipsis(show_ellipsis)
            .include_background_rectangle(include_background_rectangle)
            .edge_to_fix(edge_to_fix)
            .font_size(font_size)
            .stroke_width(stroke_width)
            .fill_opacity(fill_opacity)
            .fill_border_width(fill_border_width);
        if let Some(unit) = &unit {
            decimal = decimal.unit(unit);
        }
        if let Some(color) = color {
            decimal = decimal.color(srgb_from_py(color)?);
        }
        let built = with_font_book(|book| decimal.build(book).map_err(native_error))?;
        let mut tree = fmn_mobject::Mobject::from(built.into_vmob());
        if include_background_rectangle && !tree.submobjects.is_empty() {
            // fm-5wq.4.92: the PORTAL contract hoists the background
            // rectangle onto the ROOT's own records (so set_value's
            // become seam can rebuild root records in place across digit
            // count changes); the native builder keeps the Reference's
            // child-0 shape for the Rust front door.
            let rect = tree.submobjects.remove(0);
            let glyphs = std::mem::take(&mut tree.submobjects);
            let mut root = rect;
            root.submobjects = glyphs;
            return install_native_tree(slf, factory, root);
        }
        install_native_tree(slf, factory, tree)
    }

    /// `Elbow(width, angle)` over Atlas's native corner-mark builder.
    fn _build_elbow<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        width: f64,
        angle: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let built = fmn_library::Elbow::new().width(width).angle(angle).build();
        install_native_tree(slf, factory, built)
    }

    /// `Line(start, end, buff, path_arc)` over the line shelf.
    fn _build_line<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        start: [f64; 3],
        end: [f64; 3],
        buff: f64,
        path_arc: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let built = fmn_library::line::Line::new(start, end)
            .buff(buff)
            .path_arc(path_arc)
            .build()
            .map_err(native_error)?;
        install_native_tree(slf, factory, built)
    }

    /// Rebuild a scene-bound Line without changing its arena or Python identity.
    /// Atlas computes the complete point run first, so invalid arc parameters
    /// cannot partially mutate the live Marionette entry.
    fn _rebuild_line(
        slf: &Bound<'_, Self>,
        start: [f64; 3],
        end: [f64; 3],
        buff: f64,
        path_arc: f64,
    ) -> PyResult<()> {
        let built = fmn_library::line::Line::new(start, end)
            .buff(buff)
            .path_arc(path_arc)
            .build()
            .map_err(native_error)?;
        let points = built.points().to_vec();
        let shape = built.shape();
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| {
            stage.set_points(mob, &points)?;
            stage.set_shape(mob, shape);
            Ok(())
        })?
        .map_err(stage_error)
    }

    /// `DashedLine(start, end, dash_length, positive_space_ratio, buff)`.
    #[allow(clippy::too_many_arguments)]
    fn _build_dashed_line<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        start: [f64; 3],
        end: [f64; 3],
        dash_length: f64,
        positive_space_ratio: f64,
        buff: f64,
        path_arc: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let built = fmn_library::line::DashedLine::new(start, end)
            .dash_length(dash_length)
            .positive_space_ratio(positive_space_ratio)
            .buff(buff)
            .path_arc(path_arc)
            .build()
            .map_err(native_error)?;
        install_native_tree(slf, factory, built)
    }

    /// The bounded native dash-count rule over the line's current virtual
    /// endpoints. Atlas measures curved lines by true arc length (BN-03), so
    /// this stays in lockstep with `_build_dashed_line` rather than
    /// re-deriving its arithmetic in the Python skin.
    fn _calculate_dashed_line_num_dashes(
        _slf: &Bound<'_, Self>,
        start: [f64; 3],
        end: [f64; 3],
        dash_length: f64,
        positive_space_ratio: f64,
        path_arc: f64,
    ) -> PyResult<usize> {
        crossing::record(CrossingClass::Other);
        fmn_library::line::DashedLine::new(start, end)
            .dash_length(dash_length)
            .positive_space_ratio(positive_space_ratio)
            .path_arc(path_arc)
            .num_dashes()
            .map_err(native_error)
    }

    /// `TangentLine(vmob, alpha, length, d_alpha)` over Atlas's BN-03
    /// true-arclength tangent construction. The source is materialized from
    /// its live Marionette records in either detached or scene-bound state.
    fn _build_tangent_line<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        source: &Bound<'_, BridgeMobject>,
        alpha: f64,
        length: f64,
        d_alpha: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let points = with_stage(source, |stage, mob| {
            stage.get_points(mob).unwrap_or_default()
        })?;
        let source = fmn_library::VMobject::from_points(points);
        let built = fmn_library::line::tangent_line(
            &source,
            alpha,
            length,
            d_alpha,
            fmn_library::Style::default(),
        );
        install_native_tree(slf, factory, built)
    }

    /// `StrokeArrow(start, end, ...)` over Atlas's single-path terminal
    /// stroke taper. Every public ratio reaches the native builder; the
    /// Python skin owns only Reference object lifecycle and record dispatch.
    #[allow(clippy::too_many_arguments)]
    fn _build_stroke_arrow<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        start: [f64; 3],
        end: [f64; 3],
        stroke_width: f64,
        buff: f64,
        path_arc: f64,
        tip_width_ratio: f64,
        tip_len_to_width: f64,
        max_tip_length_to_length_ratio: f64,
        max_width_to_length_ratio: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let built = fmn_library::line::StrokeArrow::new(start, end)
            .buff(buff)
            .path_arc(path_arc)
            .tip_width_ratio(tip_width_ratio)
            .tip_len_to_width(tip_len_to_width)
            .max_tip_length_to_length_ratio(max_tip_length_to_length_ratio)
            .max_width_to_length_ratio(max_width_to_length_ratio)
            .style(fmn_library::Style::default().stroke(
                fmn_core::constants::DEFAULT_LIGHT_COLOR,
                stroke_width,
                1.0,
            ))
            .build()
            .map_err(native_error)?;
        install_native_tree(slf, factory, built)
    }

    /// Rebuild a StrokeArrow in place while retaining the root proxy and its
    /// arena identity. Stage preserves the other record columns across the
    /// resize; the native stroke profile then replaces the width column.
    #[allow(clippy::too_many_arguments)]
    fn _rebuild_stroke_arrow(
        slf: &Bound<'_, Self>,
        start: [f64; 3],
        end: [f64; 3],
        stroke_width: f64,
        buff: f64,
        path_arc: f64,
        tip_width_ratio: f64,
        tip_len_to_width: f64,
        max_tip_length_to_length_ratio: f64,
        max_width_to_length_ratio: f64,
    ) -> PyResult<()> {
        let built = fmn_library::line::StrokeArrow::new(start, end)
            .buff(buff)
            .path_arc(path_arc)
            .tip_width_ratio(tip_width_ratio)
            .tip_len_to_width(tip_len_to_width)
            .max_tip_length_to_length_ratio(max_tip_length_to_length_ratio)
            .max_width_to_length_ratio(max_width_to_length_ratio)
            .style(fmn_library::Style::default().stroke(
                fmn_core::constants::DEFAULT_LIGHT_COLOR,
                stroke_width,
                1.0,
            ))
            .build()
            .map_err(native_error)?;
        let points = built.points().to_vec();
        let shape = built.shape();
        #[allow(clippy::cast_possible_truncation)]
        let widths: Vec<f32> = built
            .stroke_profile()
            .unwrap_or_default()
            .iter()
            .map(|width| *width as f32)
            .collect();
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| {
            stage.set_points(mob, &points)?;
            stage.set_shape(mob, shape);
            let entry = stage.get_mut(mob).ok_or(StageError::StaleHandle)?;
            entry.buffer.write_range("stroke_width", 0, &widths);
            Ok(())
        })?
        .map_err(stage_error)
    }

    /// `Arrow(start, end, ...)`: one filled path with the native tip
    /// proportions and caller-selected Reference ratio caps.
    #[allow(clippy::too_many_arguments)]
    fn _build_arrow<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        start: [f64; 3],
        end: [f64; 3],
        buff: f64,
        path_arc: f64,
        thickness: f64,
        tip_width_ratio: f64,
        tip_angle: f64,
        max_tip_length_to_length_ratio: f64,
        max_width_to_length_ratio: f64,
    ) -> PyResult<(Bound<'py, PyList>, usize)> {
        let (built, tip_index) = fmn_library::line::Arrow::new(start, end)
            .buff(buff)
            .path_arc(path_arc)
            .thickness(thickness)
            .tip_width_ratio(tip_width_ratio)
            .tip_angle(tip_angle)
            .max_tip_length_to_length_ratio(max_tip_length_to_length_ratio)
            .max_width_to_length_ratio(max_width_to_length_ratio)
            .build_with_tip_index()
            .map_err(native_error)?;
        Ok((install_native_tree(slf, factory, built)?, tip_index))
    }

    /// Rebuild a scene-bound filled Arrow at new endpoints without changing
    /// its arena identity. Atlas owns the outline/tip proportions; Marionette
    /// writes the new world-space run and resets any old affine placement.
    #[allow(clippy::too_many_arguments)]
    fn _rebuild_arrow(
        slf: &Bound<'_, Self>,
        start: [f64; 3],
        end: [f64; 3],
        buff: f64,
        path_arc: f64,
        thickness: f64,
        tip_width_ratio: f64,
        tip_angle: f64,
        max_tip_length_to_length_ratio: f64,
        max_width_to_length_ratio: f64,
    ) -> PyResult<usize> {
        let (built, tip_index) = fmn_library::line::Arrow::new(start, end)
            .buff(buff)
            .path_arc(path_arc)
            .thickness(thickness)
            .tip_width_ratio(tip_width_ratio)
            .tip_angle(tip_angle)
            .max_tip_length_to_length_ratio(max_tip_length_to_length_ratio)
            .max_width_to_length_ratio(max_width_to_length_ratio)
            .build_with_tip_index()
            .map_err(native_error)?;
        let points = built.points().to_vec();
        let shape = built.shape();
        crossing::record(CrossingClass::FieldWrite);
        with_stage(slf, |stage, mob| {
            stage.set_points(mob, &points)?;
            stage.set_shape(mob, shape);
            Ok(())
        })?
        .map_err(stage_error)?;
        Ok(tip_index)
    }

    /// `Arrow.get_key_dimensions` through Atlas's single ratio-cap formula.
    #[staticmethod]
    #[allow(clippy::too_many_arguments)]
    fn _arrow_key_dimensions(
        length: f64,
        thickness: f64,
        tip_width_ratio: f64,
        tip_angle: f64,
        max_tip_length_to_length_ratio: f64,
        max_width_to_length_ratio: f64,
    ) -> (f64, f64, f64) {
        fmn_library::line::Arrow::new([0.0; 3], [1.0, 0.0, 0.0])
            .thickness(thickness)
            .tip_width_ratio(tip_width_ratio)
            .tip_angle(tip_angle)
            .max_tip_length_to_length_ratio(max_tip_length_to_length_ratio)
            .max_width_to_length_ratio(max_width_to_length_ratio)
            .key_dimensions(length)
    }

    /// `NumberLine(x_range, **config)` over the coords shelf.
    fn _build_number_line<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        x_range: &Bound<'py, PyAny>,
        config: &Bound<'py, PyDict>,
    ) -> PyResult<Bound<'py, PyList>> {
        let line = number_line_from_config(range3(x_range)?, config)?;
        let built = with_font_book(|book| line.build_numbered(book).map_err(native_error))?;
        install_native_tree(slf, factory, built.into_vmob())
    }

    /// `Slider(value_tracker, ...)` over Atlas's native static composition.
    /// Python installs the tracker-driven tip/decimal/label updaters after
    /// this builder returns; all geometry and initial layout stay native.
    #[allow(clippy::too_many_arguments)]
    fn _build_slider<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        value: f64,
        x_range: [f64; 2],
        var_name: Option<&str>,
        width: f64,
        _unit_size: f64,
        arrow_width: f64,
        arrow_length: f64,
        arrow_color: &Bound<'py, PyAny>,
        font_size: f64,
        label_buff: f64,
        num_decimal_places: usize,
        tick_size: f64,
        angle: f64,
        label_direction: Option<[f64; 3]>,
        add_tick_labels: bool,
        tick_label_font_size: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let mut slider = fmn_library::Slider::new(value)
            .x_range(x_range)
            .width(width)
            .arrow_size(arrow_width, arrow_length)
            .arrow_color(srgb_from_py(arrow_color)?)
            .font_size(font_size)
            .label_buff(label_buff)
            .num_decimal_places(num_decimal_places)
            .tick_size(tick_size)
            .angle(angle)
            .add_tick_labels(add_tick_labels)
            .tick_label_font_size(tick_label_font_size);
        if let Some(name) = var_name {
            slider = slider.var_name(name);
        }
        if let Some(direction) = label_direction {
            slider = slider.label_direction(direction);
        }
        let built = with_font_book(|book| slider.build(book).map_err(native_error))?;
        install_native_tree(slf, factory, built)
    }

    /// `Axes(...)` over the coords shelf; children are `[x_axis, y_axis]`.
    #[allow(clippy::too_many_arguments)]
    fn _build_axes<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        x_range: &Bound<'py, PyAny>,
        y_range: &Bound<'py, PyAny>,
        axis_config: Option<&Bound<'py, PyDict>>,
        x_axis_config: Option<&Bound<'py, PyDict>>,
        y_axis_config: Option<&Bound<'py, PyDict>>,
        height: Option<f64>,
        width: Option<f64>,
        unit_size: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let axes = axes_builder(
            range3(x_range)?,
            range3(y_range)?,
            axis_config,
            x_axis_config,
            y_axis_config,
            height,
            width,
            unit_size,
            None,
        )?;
        let built = with_font_book(|book| axes.build(book).map_err(native_error))?;
        install_native_tree(slf, factory, built.into_vmob())
    }

    /// `ThreeDAxes(...)`; children are `[x_axis, y_axis, z_axis]`.
    #[allow(clippy::too_many_arguments)]
    fn _build_three_d_axes<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        x_range: &Bound<'py, PyAny>,
        y_range: &Bound<'py, PyAny>,
        z_range: &Bound<'py, PyAny>,
        axis_config: Option<&Bound<'py, PyDict>>,
        x_axis_config: Option<&Bound<'py, PyDict>>,
        y_axis_config: Option<&Bound<'py, PyDict>>,
        z_axis_config: Option<&Bound<'py, PyDict>>,
        height: Option<f64>,
        width: Option<f64>,
        depth: Option<f64>,
        z_normal: Option<[f64; 3]>,
        unit_size: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let mut axes = fmn_library::ThreeDAxes::new()
            .x_range(range3(x_range)?)
            .y_range(range3(y_range)?)
            .z_range(range3(z_range)?)
            .axis_config(axis_config_from(axis_config)?)
            .x_axis_config(axis_config_from(x_axis_config)?)
            .y_axis_config(axis_config_from(y_axis_config)?)
            .z_axis_config(axis_config_from(z_axis_config)?)
            .unit_size(unit_size);
        if let Some(height) = height {
            axes = axes.height(height);
        }
        if let Some(width) = width {
            axes = axes.width(width);
        }
        if let Some(depth) = depth {
            axes = axes.depth(depth);
        }
        if let Some(normal) = z_normal {
            axes = axes.z_normal(normal);
        }
        let built = with_font_book(|book| axes.build(book).map_err(native_error))?;
        install_native_tree(slf, factory, built.into_vmob())
    }

    /// `NumberPlane(...)`; children are
    /// `[faded_lines, background_lines, x_axis, y_axis]`.
    #[allow(clippy::too_many_arguments)]
    fn _build_number_plane<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        x_range: &Bound<'py, PyAny>,
        y_range: &Bound<'py, PyAny>,
        axis_config: Option<&Bound<'py, PyDict>>,
        x_axis_config: Option<&Bound<'py, PyDict>>,
        y_axis_config: Option<&Bound<'py, PyDict>>,
        background_line_style: Option<&Bound<'py, PyDict>>,
        faded_line_style: Option<&Bound<'py, PyDict>>,
        faded_line_ratio: usize,
        height: Option<f64>,
        width: Option<f64>,
        unit_size: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let mut plane = fmn_library::NumberPlane::new()
            .x_range(range3(x_range)?)
            .y_range(range3(y_range)?)
            .axis_config(axis_config_from(axis_config)?)
            .x_axis_config(axis_config_from(x_axis_config)?)
            .y_axis_config(axis_config_from(y_axis_config)?)
            .background_line_style(line_family_style_from(background_line_style)?)
            .faded_line_style(faded_line_style_from(faded_line_style)?)
            .faded_line_ratio(faded_line_ratio)
            .unit_size(unit_size);
        if let Some(height) = height {
            plane = plane.height(height);
        }
        if let Some(width) = width {
            plane = plane.width(width);
        }
        let built = with_font_book(|book| plane.build(book).map_err(native_error))?;
        install_native_tree(slf, factory, built.into_vmob())
    }

    /// `ComplexPlane(...)`; same family shape as `NumberPlane`.
    #[allow(clippy::too_many_arguments)]
    fn _build_complex_plane<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        x_range: &Bound<'py, PyAny>,
        y_range: &Bound<'py, PyAny>,
        axis_config: Option<&Bound<'py, PyDict>>,
        x_axis_config: Option<&Bound<'py, PyDict>>,
        y_axis_config: Option<&Bound<'py, PyDict>>,
        background_line_style: Option<&Bound<'py, PyDict>>,
        faded_line_style: Option<&Bound<'py, PyDict>>,
        faded_line_ratio: usize,
        height: Option<f64>,
        width: Option<f64>,
        unit_size: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let plane = complex_plane_builder(
            range3(x_range)?,
            range3(y_range)?,
            axis_config,
            x_axis_config,
            y_axis_config,
            background_line_style,
            faded_line_style,
            faded_line_ratio,
            height,
            width,
            unit_size,
        )?;
        let built = with_font_book(|book| plane.build(book).map_err(native_error))?;
        install_native_tree(slf, factory, built.into_vmob())
    }

    /// `BulletedList(*items, ...)` over Atlas's de-TeX'd native list
    /// composition. Bundled text glyphs, list labels, arrangement, and the
    /// initial family topology all come from the one fmn-library builder.
    fn _build_bulleted_list<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        items: Vec<String>,
        buff: f64,
        aligned_edge: [f64; 3],
        numbered: bool,
        font_size: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let item_refs: Vec<&str> = items.iter().map(String::as_str).collect();
        let built = with_font_book(|book| {
            fmn_library::BulletedList::new(&item_refs)
                .buff(buff)
                .aligned_edge(aligned_edge)
                .numbered(numbered)
                .font_size(font_size)
                .build(book)
                .map_err(native_error)
        })?;
        install_native_tree(slf, factory, built.vmob)
    }

    /// `Title(*text_parts, ...)` over Atlas's de-TeX'd title composition.
    /// The native builder owns bundled-font layout, source-part grouping,
    /// frame-top placement, and optional underline geometry.
    #[allow(clippy::too_many_arguments)]
    fn _build_title<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        text_parts: Vec<String>,
        font_size: f64,
        include_underline: bool,
        underline_width: f64,
        match_underline_width_to_text: bool,
        underline_buff: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let part_refs: Vec<&str> = text_parts.iter().map(String::as_str).collect();
        let built = with_font_book(|book| {
            fmn_library::Title::new(&part_refs)
                .font_size(font_size)
                .include_underline(include_underline)
                .underline_width(underline_width)
                .match_underline_width_to_text(match_underline_width_to_text)
                .underline_buff(underline_buff)
                .build(book)
                .map_err(native_error)
        })?;
        install_native_tree(slf, factory, built.vmob)
    }

    /// `Matrix`/`TexMatrix` over Atlas's native scalar grid engine. The
    /// portal decides only that every entry belongs to the string route;
    /// entry typesetting, placement, brackets, height capping, and ellipses
    /// all remain inside the native builder.
    #[allow(clippy::too_many_arguments)]
    fn _build_tex_matrix<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        entries: Vec<Vec<String>>,
        v_buff: f64,
        h_buff: f64,
        bracket_h_buff: f64,
        bracket_v_buff: f64,
        height: Option<f64>,
        element_alignment_corner: [f64; 3],
        ellipses_row: Option<isize>,
        ellipses_col: Option<isize>,
        ellipses_height_ratio: f64,
        ellipses_width_ratio: f64,
        font_size: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let entry_refs: Vec<Vec<&str>> = entries
            .iter()
            .map(|row| row.iter().map(String::as_str).collect())
            .collect();
        let mut builder = fmn_library::TexMatrix::new(entry_refs)
            .v_buff(v_buff)
            .h_buff(h_buff)
            .bracket_h_buff(bracket_h_buff)
            .bracket_v_buff(bracket_v_buff)
            .element_alignment_corner(element_alignment_corner)
            .ellipses_ratios(ellipses_height_ratio, ellipses_width_ratio)
            .font_size(font_size);
        if let Some(height) = height {
            builder = builder.height(height);
        }
        if let Some(row) = ellipses_row {
            builder = builder.ellipses_row(row);
        }
        if let Some(column) = ellipses_col {
            builder = builder.ellipses_col(column);
        }
        let built = with_tex_engine(|engine| builder.build(engine).map_err(native_error))?;
        install_native_tree(slf, factory, built.vmob)
    }

    /// `Matrix`'s heterogeneous post-`element_to_mobject` route. Each
    /// tagged scalar is first built by its native Tex or DecimalNumber
    /// engine, then transferred into Atlas's base Matrix grid so placement,
    /// brackets, height capping, and ellipses still have one owner.
    #[allow(clippy::too_many_arguments)]
    fn _build_mixed_matrix<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        entries: Vec<Vec<(bool, f64, String)>>,
        num_decimal_places: usize,
        v_buff: f64,
        h_buff: f64,
        bracket_h_buff: f64,
        bracket_v_buff: f64,
        height: Option<f64>,
        element_alignment_corner: [f64; 3],
        ellipses_row: Option<isize>,
        ellipses_col: Option<isize>,
        ellipses_height_ratio: f64,
        ellipses_width_ratio: f64,
        font_size: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let grid = with_tex_engine(|engine| {
            with_font_book(|book| {
                entries
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|(is_decimal, number, tex)| {
                                if *is_decimal {
                                    fmn_library::DecimalNumber::new(*number)
                                        .num_decimal_places(num_decimal_places)
                                        .font_size(font_size)
                                        .build(book)
                                        .map(fmn_library::DecimalNumber::into_vmob)
                                        .map_err(native_error)
                                } else {
                                    fmn_library::Tex::new(tex)
                                        .font_size(font_size)
                                        .build(engine)
                                        .map(|built| built.vmob)
                                        .map_err(native_error)
                                }
                            })
                            .collect::<PyResult<Vec<_>>>()
                    })
                    .collect::<PyResult<Vec<_>>>()
            })
        })?;
        let mut builder = fmn_library::Matrix::new(grid)
            .v_buff(v_buff)
            .h_buff(h_buff)
            .bracket_h_buff(bracket_h_buff)
            .bracket_v_buff(bracket_v_buff)
            .element_alignment_corner(element_alignment_corner)
            .ellipses_ratios(ellipses_height_ratio, ellipses_width_ratio);
        if let Some(height) = height {
            builder = builder.height(height);
        }
        if let Some(row) = ellipses_row {
            builder = builder.ellipses_row(row);
        }
        if let Some(column) = ellipses_col {
            builder = builder.ellipses_col(column);
        }
        let built = with_tex_engine(|engine| builder.build(engine).map_err(native_error))?;
        install_native_tree(slf, factory, built.vmob)
    }

    /// `DecimalMatrix`/`IntegerMatrix` over Atlas's native number grid.
    /// `integer=true` selects the dedicated Integer shelf when the public
    /// constructor retains its default zero decimal places; a non-zero
    /// explicit value follows the Reference's DecimalMatrix inheritance.
    #[allow(clippy::too_many_arguments)]
    fn _build_decimal_matrix<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        entries: Vec<Vec<f64>>,
        integer: bool,
        num_decimal_places: usize,
        v_buff: f64,
        h_buff: f64,
        bracket_h_buff: f64,
        bracket_v_buff: f64,
        height: Option<f64>,
        element_alignment_corner: [f64; 3],
        ellipses_row: Option<isize>,
        ellipses_col: Option<isize>,
        ellipses_height_ratio: f64,
        ellipses_width_ratio: f64,
        font_size: f64,
    ) -> PyResult<Bound<'py, PyList>> {
        let built = if integer && num_decimal_places == 0 {
            let mut builder = fmn_library::IntegerMatrix::new(entries)
                .v_buff(v_buff)
                .h_buff(h_buff)
                .bracket_h_buff(bracket_h_buff)
                .bracket_v_buff(bracket_v_buff)
                .element_alignment_corner(element_alignment_corner)
                .ellipses_ratios(ellipses_height_ratio, ellipses_width_ratio)
                .font_size(font_size);
            if let Some(height) = height {
                builder = builder.height(height);
            }
            if let Some(row) = ellipses_row {
                builder = builder.ellipses_row(row);
            }
            if let Some(column) = ellipses_col {
                builder = builder.ellipses_col(column);
            }
            with_tex_engine(|engine| {
                with_font_book(|book| builder.build(engine, book).map_err(native_error))
            })?
        } else {
            let mut builder = fmn_library::DecimalMatrix::new(entries)
                .num_decimal_places(num_decimal_places)
                .v_buff(v_buff)
                .h_buff(h_buff)
                .bracket_h_buff(bracket_h_buff)
                .bracket_v_buff(bracket_v_buff)
                .element_alignment_corner(element_alignment_corner)
                .ellipses_ratios(ellipses_height_ratio, ellipses_width_ratio)
                .font_size(font_size);
            if let Some(height) = height {
                builder = builder.height(height);
            }
            if let Some(row) = ellipses_row {
                builder = builder.ellipses_row(row);
            }
            if let Some(column) = ellipses_col {
                builder = builder.ellipses_col(column);
            }
            with_tex_engine(|engine| {
                with_font_book(|book| builder.build(engine, book).map_err(native_error))
            })?
        };
        install_native_tree(slf, factory, built.vmob)
    }

    /// `MobjectMatrix(group, ...)` over Atlas's extent-driven native grid.
    /// The bootstrap applies the resulting affine placement to the original
    /// Python entries, preserving their identities and complete record data.
    #[allow(clippy::too_many_arguments)]
    fn _build_mobject_matrix<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        extents: Vec<([f64; 3], [f64; 3])>,
        n_rows: usize,
        n_cols: usize,
        v_buff: f64,
        h_buff: f64,
        bracket_h_buff: f64,
        bracket_v_buff: f64,
        height: f64,
        element_alignment_corner: [f64; 3],
        ellipses_row: Option<isize>,
        ellipses_col: Option<isize>,
    ) -> PyResult<Bound<'py, PyList>> {
        let entries = extents
            .into_iter()
            .map(|extent| matcher_extent_vmobject(Some(extent)))
            .collect();
        let mut builder = fmn_library::MobjectMatrix::new(entries)
            .n_rows(n_rows)
            .n_cols(n_cols)
            .v_buff(v_buff)
            .h_buff(h_buff)
            .bracket_h_buff(bracket_h_buff)
            .bracket_v_buff(bracket_v_buff)
            .height(height)
            .element_alignment_corner(element_alignment_corner);
        if let Some(row) = ellipses_row {
            builder = builder.ellipses_row(row);
        }
        if let Some(column) = ellipses_col {
            builder = builder.ellipses_col(column);
        }
        let built = with_tex_engine(|engine| builder.build(engine).map_err(native_error))?;
        install_native_tree(slf, factory, built.vmob)
    }

    /// `Text(...)` over the Scribe bridge: one glyph per child from the
    /// bundled FontBook, decorations trailing.
    #[allow(clippy::too_many_arguments)]
    fn _build_text<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        text: &str,
        markup: bool,
        font_size: f64,
        justify: bool,
        indent: f64,
        line_width: Option<f64>,
        disable_ligatures: bool,
        hoist_records: bool,
    ) -> PyResult<Bound<'py, PyList>> {
        let mut builder = if markup {
            fmn_library::Text::markup(text)
        } else {
            fmn_library::Text::new(text)
        };
        builder = builder
            .font_size(font_size)
            .ligatures(!disable_ligatures)
            .justify(justify)
            .indent(indent);
        if let Some(width) = line_width {
            builder = builder.width(width);
        }
        let built = with_font_book(|book| builder.build(book).map_err(native_error))?;
        let spans: Vec<(usize, usize)> =
            built.layout.glyphs.iter().map(|glyph| glyph.span).collect();
        let paths: Vec<Vec<usize>> = (0..spans.len()).map(|index| vec![index]).collect();
        slf.as_any().setattr("_string_sub_spans", spans)?;
        slf.as_any().setattr("_string_sub_paths", paths)?;
        let mut tree = fmn_mobject::Mobject::from(built.vmob);
        if hoist_records {
            hoist_descendant_records(&mut tree, true)?;
        }
        install_native_tree(slf, factory, tree)
    }

    /// `Tex(...)` / `TexText(...)` over fmd-math. An unsupported construct
    /// is fmd-math's typed refusal, surfaced VERBATIM (the fm-rqc ratchet
    /// consumes the named constructs from this exact message).
    ///
    /// With more than one part, glyph children regroup per part by the
    /// typeset's native source spans (`typeset.subs[i].span` — §11.4's
    /// span map, no heuristic splitting), matching the Reference's
    /// per-`SingleStringTex` submobject structure.
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (factory, parts, separator, text_mode, font_size, t2c, group_single_part, template="", preamble="", line_align="left"))]
    fn _build_tex<'py>(
        slf: &Bound<'py, Self>,
        factory: &Bound<'py, PyAny>,
        parts: Vec<String>,
        separator: &str,
        text_mode: bool,
        font_size: f64,
        t2c: Option<&Bound<'py, PyDict>>,
        group_single_part: bool,
        template: &str,
        preamble: &str,
        line_align: &str,
    ) -> PyResult<Bound<'py, PyList>> {
        let align = match line_align {
            "left" => fmn_library::LineAlign::Left,
            "center" => fmn_library::LineAlign::Center,
            "right" => fmn_library::LineAlign::Right,
            other => {
                return Err(PyValueError::new_err(format!(
                    "unknown TexText line alignment '{other}'"
                )));
            }
        };
        let source = parts.join(separator);
        let pairs = t2c_pairs(t2c)?;
        let refs: Vec<(&str, fmn_core::color::Srgb)> = pairs
            .iter()
            .map(|(needle, color)| (needle.as_str(), *color))
            .collect();
        let built = portal_typography::with_tex_template(template, |engine| {
            if text_mode {
                fmn_library::TexText::new(&source)
                    .preamble(preamble)
                    .line_align(align)
                    .font_size(font_size)
                    .t2c(&refs)
                    .build(engine)
            } else {
                fmn_library::Tex::new(&source)
                    .preamble(preamble)
                    .font_size(font_size)
                    .t2c(&refs)
                    .build(engine)
            }
            // VERBATIM: fmd-math's named-construct refusal is the ratchet's
            // input; never wrap it in a generic message.
            .map_err(tex_error)
        })?;
        let spans: Vec<(usize, usize)> = built
            .typeset
            .subs
            .iter()
            .map(|sub| (sub.span.start, sub.span.end))
            .collect();
        // Which primitive each sub is, parallel to the spans: the portal
        // classes glyphs and paths as the Reference's VMobjectFromSVGPath
        // and rules as its Rectangle (dvisvgm's <rect>) (fm-15k9).
        let kinds: Vec<&str> = built
            .typeset
            .subs
            .iter()
            .map(|sub| match sub.prim {
                fmn_library::TexPrim::Glyph(_) => "glyph",
                fmn_library::TexPrim::Rule(_) => "rule",
                fmn_library::TexPrim::Path(_) => "path",
            })
            .collect();
        slf.as_any().setattr("_string_sub_kinds", kinds)?;
        if parts.is_empty() || (parts.len() == 1 && !group_single_part) {
            let paths: Vec<Vec<usize>> = (0..spans.len()).map(|index| vec![index]).collect();
            slf.as_any().setattr("_string_sub_spans", spans)?;
            slf.as_any().setattr("_string_sub_paths", paths)?;
            return install_native_tree(slf, factory, built.vmob);
        }
        // Half-open byte ranges of each part in the joined source; a part
        // owns its trailing separator so every source byte has one owner.
        let mut ranges = Vec::with_capacity(parts.len());
        let mut cursor = 0usize;
        for (index, part) in parts.iter().enumerate() {
            let start = cursor;
            cursor += part.len();
            if index + 1 < parts.len() {
                cursor += separator.len();
            }
            ranges.push((start, cursor));
        }
        let subs = &built.typeset.subs;
        let mut tree = Mobject::from(built.vmob.clone());
        let children = std::mem::take(&mut tree.submobjects);
        if children.len() != spans.len() {
            return Err(PyRuntimeError::new_err(format!(
                "native Tex span table has {} entries for {} primitives",
                spans.len(),
                children.len()
            )));
        }
        let mut buckets: Vec<Vec<Mobject>> = (0..parts.len()).map(|_| Vec::new()).collect();
        let mut paths = vec![Vec::new(); children.len()];
        for (index, child) in children.into_iter().enumerate() {
            let start = subs.get(index).map_or(0, |sub| sub.span.start);
            let part = ranges
                .iter()
                .position(|&(from, to)| from <= start && start < to)
                .unwrap_or(parts.len() - 1);
            paths[index] = vec![part, buckets[part].len()];
            buckets[part].push(child);
        }
        tree.submobjects = buckets
            .into_iter()
            .map(|kids| {
                // A vmobject-schema group node, so the style surface sees
                // the stroke/fill fields on the part wrapper too.
                let mut node = Mobject::from(fmn_library::vmobject::v_group(std::iter::empty::<
                    fmn_library::VMobject,
                >()));
                node.submobjects = kids;
                node
            })
            .collect();
        slf.as_any().setattr("_string_sub_spans", spans)?;
        slf.as_any().setattr("_string_sub_paths", paths)?;
        install_native_tree(slf, factory, tree)
    }

    /// `NumberLine.add_numbers`: rebuild the native line at the proxy's
    /// current width, run the native labeler at `font_size`, and return
    /// the trailing label group (shifted onto the current center) as one
    /// shell spec. Same rebuild caveat as `_axes_label_shells`.
    #[staticmethod]
    #[allow(clippy::too_many_arguments)]
    fn _number_line_label_shells<'py>(
        factory: &Bound<'py, PyAny>,
        x_range: &Bound<'py, PyAny>,
        config: &Bound<'py, PyDict>,
        font_size: f64,
        current_width: f64,
        current_center: [f64; 3],
        x_values: Option<Vec<f64>>,
        excluding: Option<Vec<f64>>,
        direction: Option<[f64; 3]>,
        buff: Option<f64>,
        num_decimal_places: Option<usize>,
    ) -> PyResult<Bound<'py, PyList>> {
        let mut line = number_line_from_config(range3(x_range)?, config)?
            .numbers_font_size(font_size)
            .width(current_width);
        if let Some(direction) = direction {
            line = line.line_to_number_direction(direction);
        }
        if let Some(buff) = buff {
            line = line.line_to_number_buff(buff);
        }
        if let Some(num_decimal_places) = num_decimal_places {
            line = line.num_decimal_places(num_decimal_places);
        }
        let mut built = line.build().map_err(native_error)?;
        let before = built.vmob().children().len();
        with_font_book(|book| {
            built
                .add_numbers(book, x_values.as_deref(), excluding.as_deref())
                .map_err(native_error)
        })?;
        let groups: Vec<Mobject> = built.vmob().children()[before..]
            .iter()
            .map(|group| Mobject::from(group.clone().shifted(current_center)))
            .collect();
        native_shell_specs(factory.py(), factory, groups)
    }

    /// `Axes.add_coordinate_labels`: rebuild the native axes at the
    /// proxy's CURRENT width/height, run the native labeler, and return
    /// the two trailing label groups (shifted onto the proxy's current
    /// center) as shell specs for `x_axis`/`y_axis`.
    ///
    /// Until live-state cores land (fm-p107 territory), the rebuild
    /// reproduces uniform rescales and translations exactly; a rotated or
    /// stretched axes would label at the unrotated positions.
    #[staticmethod]
    #[allow(clippy::too_many_arguments)]
    fn _axes_label_shells<'py>(
        factory: &Bound<'py, PyAny>,
        x_range: &Bound<'py, PyAny>,
        y_range: &Bound<'py, PyAny>,
        axis_config: Option<&Bound<'py, PyDict>>,
        x_axis_config: Option<&Bound<'py, PyDict>>,
        y_axis_config: Option<&Bound<'py, PyDict>>,
        unit_size: f64,
        current_width: f64,
        current_height: f64,
        current_center: [f64; 3],
        x_values: Option<Vec<f64>>,
        y_values: Option<Vec<f64>>,
        excluding: Vec<f64>,
        font_size: Option<f64>,
    ) -> PyResult<(Bound<'py, PyList>, Bound<'py, PyList>)> {
        let axes = axes_builder(
            range3(x_range)?,
            range3(y_range)?,
            axis_config,
            x_axis_config,
            y_axis_config,
            Some(current_height),
            Some(current_width),
            unit_size,
            font_size,
        )?;
        let mut built = with_font_book(|book| axes.build(book).map_err(native_error))?;
        let before_x = built.x_axis().vmob().children().len();
        let before_y = built.y_axis().vmob().children().len();
        with_font_book(|book| {
            built
                .add_coordinate_labels(
                    book,
                    x_values.as_deref(),
                    y_values.as_deref(),
                    Some(&excluding),
                    font_size,
                )
                .map_err(native_error)
        })?;
        let py = factory.py();
        let label_specs =
            |axis: &fmn_library::NumberLine, before: usize| -> PyResult<Bound<'py, PyList>> {
                let groups: Vec<Mobject> = axis.vmob().children()[before..]
                    .iter()
                    .map(|group| Mobject::from(group.clone().shifted(current_center)))
                    .collect();
                native_shell_specs(py, factory, groups)
            };
        Ok((
            label_specs(built.x_axis(), before_x)?,
            label_specs(built.y_axis(), before_y)?,
        ))
    }

    /// `ComplexPlane.add_coordinate_labels`: rebuild at the proxy's
    /// current width/height (with `font_size` routed to the axes'
    /// `decimal_number_config`), run the native labeler, and return the
    /// trailing label group (shifted onto the current center) as one
    /// shell spec. Same rebuild caveat as `_axes_label_shells`.
    #[staticmethod]
    #[allow(clippy::too_many_arguments)]
    fn _complex_plane_label_shells<'py>(
        factory: &Bound<'py, PyAny>,
        x_range: &Bound<'py, PyAny>,
        y_range: &Bound<'py, PyAny>,
        axis_config: Option<&Bound<'py, PyDict>>,
        x_axis_config: Option<&Bound<'py, PyDict>>,
        y_axis_config: Option<&Bound<'py, PyDict>>,
        background_line_style: Option<&Bound<'py, PyDict>>,
        faded_line_style: Option<&Bound<'py, PyDict>>,
        faded_line_ratio: usize,
        unit_size: f64,
        current_width: f64,
        current_height: f64,
        current_center: [f64; 3],
        numbers: Option<Vec<[f64; 2]>>,
        font_size: Option<f64>,
    ) -> PyResult<Bound<'py, PyList>> {
        let mut axis_cfg = axis_config_from(axis_config)?;
        let mut x_cfg = axis_config_from(x_axis_config)?;
        let mut y_cfg = axis_config_from(y_axis_config)?;
        if let Some(font_size) = font_size {
            axis_cfg.number_font_size = Some(font_size);
            x_cfg.number_font_size = Some(font_size);
            y_cfg.number_font_size = Some(font_size);
        }
        let plane = fmn_library::ComplexPlane::new()
            .x_range(range3(x_range)?)
            .y_range(range3(y_range)?)
            .axis_config(axis_cfg)
            .x_axis_config(x_cfg)
            .y_axis_config(y_cfg)
            .background_line_style(line_family_style_from(background_line_style)?)
            .faded_line_style(faded_line_style_from(faded_line_style)?)
            .faded_line_ratio(faded_line_ratio)
            .unit_size(unit_size)
            .height(current_height)
            .width(current_width);
        let mut built = with_font_book(|book| plane.build(book).map_err(native_error))?;
        let before = built.vmob().children().len();
        with_font_book(|book| {
            match &numbers {
                Some(values) => built.add_coordinate_labels_for(values, book),
                None => built.add_coordinate_labels(book),
            }
            .map_err(native_error)
        })?;
        let groups: Vec<Mobject> = built.vmob().children()[before..]
            .iter()
            .map(|group| Mobject::from(group.clone().shifted(current_center)))
            .collect();
        native_shell_specs(factory.py(), factory, groups)
    }

    fn _replace_submobjects(
        slf: &Bound<'_, Self>,
        children: Vec<Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        let mut seen = HashSet::new();
        let mut proxies = Vec::with_capacity(children.len());
        for child in children {
            let child = child
                .cast_into::<BridgeMobject>()
                .map_err(|_| PyTypeError::new_err("submobjects must be Mobject instances"))?;
            let marker = child.as_ptr() as usize;
            if !seen.insert(marker) {
                return Err(PyValueError::new_err(
                    "one submobject cannot appear twice under the same parent",
                ));
            }
            proxies.push(child);
        }
        let location_of = |proxy: &Bound<'_, BridgeMobject>| {
            let cell = proxy.borrow();
            (
                cell.engine.as_ref().map(Rc::clone),
                cell.mob,
                cell.initialized && cell.nursery.is_some(),
            )
        };
        let mut child_locations: Vec<_> = proxies.iter().map(&location_of).collect();

        let parent_locator = |slf: &Bound<'_, Self>| {
            let cell = slf.borrow();
            (
                cell.engine.as_ref().map(Rc::clone),
                cell.mob,
                cell.initialized && cell.nursery.is_some(),
            )
        };
        let mut parent_location = parent_locator(slf);
        if parent_location.0.is_none() {
            if !parent_location.2 {
                return Err(StaleHandleError::new_err(
                    "uninitialized mobject cannot own submobjects",
                ));
            }
            let bound_child = proxies
                .iter()
                .zip(&child_locations)
                .find_map(|(proxy, location)| location.0.is_some().then_some(proxy));
            match bound_child {
                None => {
                    if child_locations.iter().any(|(_, _, detached)| !detached) {
                        return Err(ForeignStageError::new_err(
                            "a detached parent may contain only detached mobjects",
                        ));
                    }
                    // The Python live list is authoritative until Scene.add
                    // binds the complete graph in one transaction.
                    return Ok(());
                }
                Some(child) => {
                    // Mirror adoption (fm-p107): a detached parent
                    // ingesting a bound child adopts INTO the child's
                    // scene first — the Reference's global-mobject model;
                    // the parent is typically scene-added right after.
                    let scene_object = child.getattr("_scene")?;
                    let scene = scene_object.cast::<PyScene>().map_err(|_| {
                        PyRuntimeError::new_err("bound proxy's `_scene` is not a Scene")
                    })?;
                    bind_graph(slf.py(), scene, slf)?;
                    parent_location = parent_locator(slf);
                }
            }
        }
        let (Some(engine), Some(parent), _) = parent_location else {
            return Err(StaleHandleError::new_err(
                "parent adoption did not bind the mobject",
            ));
        };

        // Adoption-on-attach (fm-d3gt): a detached child joining a
        // scene-bound parent adopts into the parent's scene first, through
        // the SAME bind_graph path Scene.add uses — nursery copy_into,
        // pinning, proxy registration, `_scene`, Python identity intact.
        // Mixed lists work; a child bound to a different scene still
        // refuses below.
        if child_locations
            .iter()
            .any(|(child_engine, _, detached)| child_engine.is_none() && *detached)
        {
            let scene_object = slf.getattr("_scene")?;
            let scene = scene_object
                .cast::<PyScene>()
                .map_err(|_| PyRuntimeError::new_err("bound proxy's `_scene` is not a Scene"))?;
            for (proxy, location) in proxies.iter().zip(&child_locations) {
                if location.0.is_none() && location.2 {
                    // bind_graph registers proxies and sets `_scene` itself.
                    bind_graph(slf.py(), scene, proxy)?;
                }
            }
            child_locations = proxies.iter().map(&location_of).collect();
        }

        let mut candidate = Vec::with_capacity(child_locations.len());
        for (child_engine, child_mob, _) in child_locations {
            let (Some(child_engine), Some(child_mob)) = (child_engine, child_mob) else {
                return Err(ForeignStageError::new_err(
                    "a bound parent may contain only mobjects from the same Scene",
                ));
            };
            if !same_engine(&engine, &child_engine) {
                return Err(ForeignStageError::new_err(
                    "submobject belongs to a different Scene",
                ));
            }
            candidate.push(child_mob);
        }

        let mut runtime = engine.borrow_mut();
        if runtime.stage().get(parent).is_none() {
            return Err(StaleHandleError::new_err(
                "parent handle no longer resolves",
            ));
        }
        // One pass; a refused child leaves the graph exactly as it was.
        runtime
            .stage_mut()
            .replace_children(parent, &candidate)
            .map_err(stage_error)
    }

    fn family_size(slf: &Bound<'_, Self>) -> PyResult<usize> {
        crossing::record(CrossingClass::Other);
        let location = {
            let cell = slf.borrow();
            cell.engine
                .as_ref()
                .zip(cell.mob)
                .map(|(engine, mob)| (Rc::clone(engine), mob))
        };
        if let Some((engine, mob)) = location {
            return Ok(engine.borrow().stage().family(mob).len());
        }
        Ok(collect_proxy_graph(slf.as_any())?.len())
    }

    fn interpolate(
        slf: &Bound<'_, Self>,
        start: &Bound<'_, BridgeMobject>,
        target: &Bound<'_, BridgeMobject>,
        alpha: f64,
    ) -> PyResult<()> {
        location_compatible(slf, start)?;
        location_compatible(slf, target)?;
        let (start_schema, start_len, start_records) = flat_records(start)?;
        let (target_schema, target_len, target_records) = flat_records(target)?;
        if start_schema != target_schema || start_len != target_len {
            return Err(PyValueError::new_err(
                "interpolation endpoints require identical schemas and record counts; align first",
            ));
        }
        let (own_schema, own_len, _) = flat_records(slf)?;
        if own_schema != start_schema || own_len != start_len {
            return Err(PyValueError::new_err(
                "interpolating mobject does not match its endpoints",
            ));
        }
        let alpha = alpha as f32;
        // fm-zoi GIL discipline (§17.4): the mixing kernel touches only
        // owned f32 vectors, so it runs with the GIL released and the
        // interpreter overlaps the native conversion. Bit-identical lane
        // order: from + (to - from) * alpha, record-major.
        let mixed: Vec<f32> = slf.py().detach(move || {
            start_records
                .iter()
                .zip(target_records.iter())
                .map(|(from, to)| from + (to - from) * alpha)
                .collect()
        });
        crossing::record(CrossingClass::Other);
        with_buffer(slf, |buffer| {
            let fields: Vec<(String, usize)> = buffer
                .schema()
                .fields()
                .iter()
                .map(|field| (field.name.clone(), field.width))
                .collect();
            let mut cursor = 0usize;
            for index in 0..buffer.len() {
                for (field, width) in &fields {
                    let end = cursor + width;
                    let wrote = buffer.write(index, field, &mixed[cursor..end]);
                    debug_assert!(wrote, "schema and loop are identical");
                    cursor = end;
                }
            }
        })
    }

    /// Engine-side family copy, returning Python proxy shells for the
    /// bootstrap's shallow/deep `__dict__` pass. Detached graphs return None.
    fn _copy_family_shells<'py>(slf: &Bound<'py, Self>) -> PyResult<Option<ProxyPairs>> {
        let (engine, root) = {
            let cell = slf.borrow();
            match (&cell.engine, cell.mob) {
                (Some(engine), Some(mob)) => (Rc::clone(engine), mob),
                _ => return Ok(None),
            }
        };
        let scene_object = slf.getattr("_scene")?;
        let scene = scene_object
            .cast::<PyScene>()
            .map_err(|_| PyRuntimeError::new_err("bound proxy's `_scene` is not a Scene"))?;
        let copy_map = engine
            .borrow_mut()
            .stage_mut()
            .copy_family_mapped(root)
            .map_err(stage_error)?;
        let py = slf.py();
        let mut pairs = Vec::with_capacity(copy_map.len());
        for &(old, new) in copy_map.pairs() {
            let old_proxy = live_proxy(py, scene, old).ok_or_else(|| {
                PyRuntimeError::new_err("engine family member has no live Python proxy during copy")
            })?;
            let new_proxy = new_bound_proxy(py, scene, &old_proxy, &engine, new)?;
            pairs.push((old_proxy.unbind(), new_proxy.unbind()));
        }
        Ok(Some(pairs))
    }

    /// Copy one detached nursery into another proxy through Marionette's
    /// cross-stage transfer. Unlike the pickle-shaped record-state path,
    /// this preserves renderer-owned entry metadata such as placements,
    /// shape tags, render primitives, and durable image resources.
    fn _copy_detached_state_to(
        slf: &Bound<'_, Self>,
        target: &Bound<'_, BridgeMobject>,
    ) -> PyResult<()> {
        if slf.is(target) {
            return Err(PyValueError::new_err(
                "detached copy target must be a distinct mobject",
            ));
        }
        let nursery = {
            let source = slf.borrow();
            if source.engine.is_some() || source.mob.is_some() {
                return Err(PyRuntimeError::new_err(
                    "detached-state copy requires a detached source mobject",
                ));
            }
            let source_nursery = source.nursery.as_ref().ok_or_else(|| {
                PyRuntimeError::new_err("detached source mobject has no native nursery")
            })?;
            let mut stage = Stage::new();
            let root = source_nursery
                .stage
                .copy_into(source_nursery.root, &mut stage)
                .map_err(stage_error)?;
            Nursery { stage, root }
        };
        let mut destination = target.borrow_mut();
        if destination.engine.is_some() || destination.mob.is_some() {
            return Err(PyRuntimeError::new_err(
                "detached-state copy cannot overwrite a bound target mobject",
            ));
        }
        destination.nursery = Some(nursery);
        destination.initialized = true;
        Ok(())
    }

    fn _engine_state<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyDict>> {
        native_state(slf.py(), slf)
    }

    fn _restore_engine_state(slf: &Bound<'_, Self>, state: &Bound<'_, PyDict>) -> PyResult<()> {
        restore_native_state(slf, state)
    }

    fn is_alive(slf: &Bound<'_, Self>) -> bool {
        crossing::record(CrossingClass::Other);
        let cell = slf.borrow();
        match (&cell.engine, cell.mob) {
            (Some(engine), Some(mob)) => engine.borrow().stage().contains(mob),
            _ => cell.nursery.is_some(),
        }
    }

    fn delete(slf: &Bound<'_, Self>) -> PyResult<()> {
        let (engine, mob) = bound_parts(&slf.borrow())?;
        engine
            .borrow_mut()
            .stage_mut()
            .delete(mob)
            .map_err(stage_error)
    }

    fn noop(_slf: &Bound<'_, Self>) {}
}

impl Drop for BridgeMobject {
    fn drop(&mut self) {
        if let (Some(engine), Some(mob)) = (&self.engine, self.mob) {
            engine.release_pin(mob);
        }
    }
}

fn scene_proxy_handles(
    scene: &Bound<'_, PyScene>,
    objects: &Bound<'_, PyTuple>,
) -> PyResult<Vec<Mob>> {
    let engine = Rc::clone(&scene.borrow().engine);
    let mut handles = Vec::with_capacity(objects.len());
    for object in objects.iter() {
        let proxy = object.cast::<BridgeMobject>().map_err(|_| {
            PyTypeError::new_err("Scene membership operations require Mobject instances")
        })?;
        let (object_engine, mob) = bound_parts(&proxy.borrow())?;
        if !same_engine(&engine, &object_engine) {
            return Err(ForeignStageError::new_err(
                "mobject belongs to a different Scene",
            ));
        }
        handles.push(mob);
    }
    Ok(handles)
}

/// Handles for the Reference's `Scene.remove`, whose removal set is the
/// family of every argument. A bound mobject contributes its handle (the
/// stage expands its family). A detached one, e.g. a never-added group of
/// added members, contributes its bound descendants. A wholly detached family
/// contributes nothing, because removing an absent mobject is a no-op.
fn scene_removal_handles(
    scene: &Bound<'_, PyScene>,
    objects: &Bound<'_, PyTuple>,
) -> PyResult<Vec<Mob>> {
    let engine = Rc::clone(&scene.borrow().engine);
    let mut handles = Vec::new();
    let mut seen = HashSet::new();
    let mut pending: Vec<Bound<'_, PyAny>> = objects.iter().collect();
    while let Some(object) = pending.pop() {
        if !seen.insert(object.as_ptr() as usize) {
            continue;
        }
        let proxy = object.cast::<BridgeMobject>().map_err(|_| {
            PyTypeError::new_err("Scene membership operations require Mobject instances")
        })?;
        let parts = bound_parts(&proxy.borrow());
        match parts {
            Ok((object_engine, mob)) => {
                if !same_engine(&engine, &object_engine) {
                    return Err(ForeignStageError::new_err(
                        "mobject belongs to a different Scene",
                    ));
                }
                handles.push(mob);
            }
            Err(_) => {
                for child in object.getattr("submobjects")?.try_iter()? {
                    pending.push(child?);
                }
            }
        }
    }
    Ok(handles)
}

fn bind_scene_mobjects(
    scene: &Bound<'_, PyScene>,
    objects: &Bound<'_, PyTuple>,
    method: &str,
) -> PyResult<Vec<Mob>> {
    let mut proxies = Vec::with_capacity(objects.len());
    for object in objects.iter() {
        proxies.push(object.cast_into::<BridgeMobject>().map_err(|_| {
            PyTypeError::new_err(format!("Scene.{method} accepts only Mobject instances"))
        })?);
    }

    let py = scene.py();
    proxies
        .into_iter()
        .map(|proxy| bind_graph(py, scene, &proxy))
        .collect()
}

/// Resolve `object` only when it is the exact top-level member that the
/// Reference's `Scene.replace` membership test would find. Non-mobjects,
/// detached mobjects, foreign-scene mobjects, and family descendants are all
/// absent from this scene and therefore make replacement a side-effect-free
/// no-op, including leaving replacement arguments detached.
fn scene_root_handle(scene: &Bound<'_, PyScene>, object: &Bound<'_, PyAny>) -> Option<Mob> {
    let proxy = object.cast::<BridgeMobject>().ok()?;
    let engine = Rc::clone(&scene.borrow().engine);
    let (object_engine, mob) = {
        let cell = proxy.borrow();
        (Rc::clone(cell.engine.as_ref()?), cell.mob?)
    };
    if !same_engine(&engine, &object_engine) {
        return None;
    }
    engine.borrow().mobjects().contains(&mob).then_some(mob)
}

/// Collect one unsuspended updater subtree in the Reference's child-first
/// order. A suspended parent prunes its entire subtree even when descendants
/// are not individually marked suspended. The explicit stack keeps a valid
/// deeply nested family from consuming the scene worker's call stack.
/// `seen` mirrors `targets` membership, so a mobject reachable through
/// several roots or parents costs one hash probe, not a scan of `targets`.
fn collect_update_targets(
    stage: &Stage,
    root: Mob,
    targets: &mut Vec<Mob>,
    seen: &mut HashSet<Mob>,
) {
    let mut stack = vec![(root, false)];
    while let Some((mob, visited)) = stack.pop() {
        if visited {
            if seen.insert(mob) {
                targets.push(mob);
            }
            continue;
        }
        if stage.is_updating_suspended(mob) || seen.contains(&mob) {
            continue;
        }
        stack.push((mob, true));
        if let Some(entry) = stage.get(mob) {
            stack.extend(
                entry
                    .submobjects()
                    .iter()
                    .rev()
                    .map(|&child| (child, false)),
            );
        }
    }
}

/// Mobjects receiving updater dispatch this frame, in the same child-first,
/// suspension-pruned order as Marionette's native updater pass.
fn update_targets(scene: &Bound<'_, PyScene>) -> Vec<Mob> {
    let scene_cell = scene.borrow();
    let runtime = scene_cell.engine.borrow();
    let mut targets = Vec::new();
    let mut seen = HashSet::new();
    for &root in runtime.stage().roots() {
        collect_update_targets(runtime.stage(), root, &mut targets, &mut seen);
    }
    targets
}

/// The Reference keeps the camera frame at the head of `Scene.mobjects`, so
/// its host-language updaters run before every drawable-root updater.  The
/// portal deliberately keeps CameraFrame out of Marionette's drawable Stage;
/// recover that same update ordering through the real Python frame object.
fn camera_frame<'py>(scene: &Bound<'py, PyScene>) -> PyResult<Bound<'py, PyAny>> {
    scene.getattr("frame")
}

fn run_proxy_updaters(proxy: &Bound<'_, PyAny>, dt: f64) -> PyResult<()> {
    let py = proxy.py();
    crossing::record(CrossingClass::Other);
    let updaters = proxy.getattr("updaters")?;
    let snapshot: Vec<Py<PyAny>> = updaters
        .try_iter()?
        .map(|item| item.map(Bound::unbind))
        .collect::<PyResult<_>>()?;
    for updater in snapshot {
        crossing::record(CrossingClass::UpdaterCall);
        let args = PyTuple::new(
            py,
            [updater.bind(py).clone(), dt.into_pyobject(py)?.into_any()],
        )?;
        method_cache::call_cached1(proxy, "_dispatch_updater", &args)?;
    }
    Ok(())
}

/// Run the portal half of Scene.update_mobjects with no Scene/Stage borrow
/// live. Choreo's stepped play releases exactly at this call site; ordinary
/// `Scene.update(dt)` uses the same helper before its native updater pass.
fn run_python_updaters(scene: &Bound<'_, PyScene>, dt: f64) -> PyResult<u64> {
    let targets = update_targets(scene);
    let py = scene.py();
    let python_start = Instant::now();
    run_proxy_updaters(&camera_frame(scene)?, dt)?;
    for target in targets {
        let Some(proxy) = live_proxy(py, scene, target) else {
            continue;
        };
        run_proxy_updaters(&proxy, dt)?;
    }
    Ok(u64::try_from(python_start.elapsed().as_nanos()).unwrap_or(u64::MAX))
}

/// Model `Mobject.resume_updating(call_updater=True)` for the host-language
/// updater layer after Choreo has finished an animation lifecycle.  This is
/// deliberately root-scoped; the subsequent scene-wide zero-`dt` pass is a
/// separate Reference event.
fn run_resumed_python_updaters(scene: &Bound<'_, PyScene>, roots: &[Mob]) -> PyResult<()> {
    let targets = {
        let scene_cell = scene.borrow();
        let runtime = scene_cell.engine.borrow();
        let mut targets = Vec::new();
        let mut seen = HashSet::new();
        for &root in roots {
            collect_update_targets(runtime.stage(), root, &mut targets, &mut seen);
        }
        targets
    };
    let py = scene.py();
    for target in targets {
        if let Some(proxy) = live_proxy(py, scene, target) {
            run_proxy_updaters(&proxy, 0.0)?;
        }
    }
    Ok(())
}

fn has_python_updaters(scene: &Bound<'_, PyScene>) -> PyResult<bool> {
    let py = scene.py();
    if camera_frame(scene)?.getattr("updaters")?.is_truthy()? {
        return Ok(true);
    }
    for target in update_targets(scene) {
        if let Some(proxy) = live_proxy(py, scene, target)
            && proxy.getattr("updaters")?.is_truthy()?
        {
            return Ok(true);
        }
    }
    Ok(false)
}

struct PortalRenderRequest {
    destination: String,
    width: u32,
    height: u32,
    fps: u32,
    threads: usize,
    seed: u64,
    format: PortalOutputFormat,
    reproducible: bool,
}

fn portal_has_frame_render(scene: &Bound<'_, PyScene>) -> PyResult<bool> {
    let render = Arc::clone(&scene.borrow().render);
    let render = render
        .lock()
        .map_err(|_| PyRuntimeError::new_err("portal render session lock was poisoned"))?;
    Ok(matches!(
        render.as_ref(),
        Some(
            PortalRenderSession::Frames(_)
                | PortalRenderSession::Preview(_)
                | PortalRenderSession::Vector(_)
        )
    ))
}

/// Freeze host camera state while no Scene borrow is live. The existing
/// stepped release boundary then runs native updaters and captures the frame.
fn synchronize_portal_camera(scene: &Bound<'_, PyScene>) -> PyResult<()> {
    if !portal_has_frame_render(scene)? {
        return Ok(());
    }
    let camera = scene.getattr("camera")?;
    let core = camera.getattr("frame")?.getattr("_core")?;
    let frame = core
        .extract::<PyRef<'_, PyCameraFrameCore>>()?
        .frame
        .clone();
    let rgba: [f64; 4] = camera.getattr("background_rgba")?.extract()?;
    if !rgba.iter().all(|component| component.is_finite()) {
        return Err(PyValueError::new_err(
            "portal-render: camera background must be finite",
        ));
    }
    let background = fmn_core::color::Srgb {
        r: rgba[0],
        g: rgba[1],
        b: rgba[2],
    }
    .to_linear(rgba[3]);
    let light = camera.getattr("light_source")?;
    let center_method = light.getattr("get_center")?;
    let light_position = center_method.call0()?.extract::<[f64; 3]>()?;
    let default_center = scene
        .py()
        .import("manimlib")?
        .getattr("Mobject")?
        .getattr("get_center")?;
    let native_center = center_method
        .getattr("__func__")
        .is_ok_and(|method| method.is(&default_center));
    // A light in this Scene can still move in the native updater phase after
    // this release. Resolve its position from the immutable captured Stage.
    let engine = Rc::clone(&scene.borrow().engine);
    let light_mob = light
        .extract::<PyRef<'_, BridgeMobject>>()
        .ok()
        .and_then(|proxy| {
            proxy
                .engine
                .as_ref()
                .filter(|owner| native_center && Rc::ptr_eq(owner, &engine))
                .and(proxy.mob)
        });
    let render = Arc::clone(&scene.borrow().render);
    let mut render = render
        .lock()
        .map_err(|_| PyRuntimeError::new_err("portal render session lock was poisoned"))?;
    if let Some(session) = render.as_mut() {
        session.bind_camera(frame, light_position, Some(background), light_mob)?;
    }
    Ok(())
}

fn begin_portal_render(slf: &Bound<'_, PyScene>, request: PortalRenderRequest) -> PyResult<()> {
    let PortalRenderRequest {
        destination,
        width,
        height,
        fps,
        threads,
        seed,
        format,
        reproducible,
    } = request;
    if destination.is_empty() {
        return Err(PyValueError::new_err(
            "render destination must not be empty",
        ));
    }
    if reproducible {
        portal_provenance::certified_build_check()?;
        // C4 must be statable before any frame exists (fm-arog).
        portal_provenance::certified_render_config((width, height), fps, threads, seed)?;
    }
    portal_video::check_scene_ownership(slf)?;
    let render_slot = Arc::clone(&slf.borrow().render);
    let video = match &format {
        PortalOutputFormat::Frames(
            frame_format @ (PortalFrameFormat::Mp4 | PortalFrameFormat::Mov),
        ) => Some(portal_video::PortalVideoConfig::from_scene(
            slf,
            *frame_format,
            width,
            height,
            fps,
        )?),
        _ => None,
    };
    let audio = portal_audio::PortalAudio::from_scene(slf, &format, video.as_ref())?;
    portal_video::check_scene_ownership(slf)?;
    let (session, runtime_config) = PortalRenderSession::new(
        PathBuf::from(destination),
        width,
        height,
        fps,
        threads,
        format,
        video,
        audio,
        reproducible,
    )?;
    let replacement = match Scene::new(runtime_config, seed) {
        Ok(scene) => Rc::new(EngineState::new(scene)),
        Err(error) => {
            session.abort();
            return Err(PyRuntimeError::new_err(error.to_string()));
        }
    };
    let mut render = match render_slot.lock() {
        Ok(render) => render,
        Err(_) => {
            session.abort();
            return Err(PyRuntimeError::new_err(
                "portal render session lock was poisoned",
            ));
        }
    };
    if render.is_some() {
        session.abort();
        return Err(PyRuntimeError::new_err(
            "a portal render generation is already active",
        ));
    }
    {
        let mut scene = slf.borrow_mut();
        scene.engine = replacement;
        scene.render_invocations.clear();
        scene.render_audio_inputs.clear();
        scene.render_routes = PortalRoutes::default();
    }
    *render = Some(session);
    Ok(())
}

#[pymethods]
impl PyScene {
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn py_new(_args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        let runtime = Scene::new(RuntimeConfig::default(), 0)
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        Ok(Self {
            engine: Rc::new(EngineState::new(runtime)),
            proxies: RefCell::new(HashMap::default()),
            proxy_generation: Cell::new(0),
            roots_projection: RefCell::new(None),
            render: Arc::new(Mutex::new(None)),
            render_invocations: Vec::new(),
            render_audio_inputs: Vec::new(),
            render_routes: PortalRoutes::default(),
        })
    }

    /// Raise Proscenium's normal early-termination signal as the portal's
    /// exact `manimlib.scene.scene.EndScene` exception type.
    fn _end_scene(&self) -> PyResult<()> {
        self.engine.borrow_mut().end::<()>().map_err(scene_error)
    }

    /// Select standard-mode PNG output with the default Reel limits.
    #[pyo3(signature = (destination, width, height, fps, threads, seed))]
    fn _begin_png(
        slf: &Bound<'_, Self>,
        destination: String,
        width: u32,
        height: u32,
        fps: u32,
        threads: usize,
        seed: u64,
    ) -> PyResult<()> {
        begin_portal_render(
            slf,
            PortalRenderRequest {
                destination,
                width,
                height,
                fps,
                threads,
                seed,
                format: PortalOutputFormat::Frames(PortalFrameFormat::Png),
                reproducible: false,
            },
        )
    }

    /// Start one no-clobber native PNG-sequence generation before lifecycle
    /// execution. Configuration is accepted only while the Scene is pristine,
    /// so changing fps/seed cannot reinterpret already-created engine state.
    #[pyo3(signature = (destination, width, height, fps, threads, seed))]
    fn _begin_png_sequence(
        slf: &Bound<'_, Self>,
        destination: String,
        width: u32,
        height: u32,
        fps: u32,
        threads: usize,
        seed: u64,
    ) -> PyResult<()> {
        begin_portal_render(
            slf,
            PortalRenderRequest {
                destination,
                width,
                height,
                fps,
                threads,
                seed,
                format: PortalOutputFormat::Frames(PortalFrameFormat::PngSequence),
                reproducible: false,
            },
        )
    }

    /// Select a Reel format without changing the scene clock or publication
    /// protocol. Video containers use only the governed ffmpeg boundary.
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (destination, format, width, height, fps, threads, seed, reproducible=false))]
    fn _begin_native_output(
        slf: &Bound<'_, Self>,
        destination: String,
        format: &str,
        width: u32,
        height: u32,
        fps: u32,
        threads: usize,
        seed: u64,
        reproducible: bool,
    ) -> PyResult<()> {
        let format = match format {
            "png" => PortalOutputFormat::Frames(PortalFrameFormat::Png),
            "png_sequence" => PortalOutputFormat::Frames(PortalFrameFormat::PngSequence),
            "gif" => PortalOutputFormat::Frames(PortalFrameFormat::Gif),
            "y4m" => PortalOutputFormat::Frames(PortalFrameFormat::Y4m),
            "wav" => PortalOutputFormat::Wav,
            "svg" => PortalOutputFormat::Svg,
            "mp4" => PortalOutputFormat::Frames(PortalFrameFormat::Mp4),
            "mov" => PortalOutputFormat::Frames(PortalFrameFormat::Mov),
            _ => {
                return Err(CapabilityError::new_err(
                    "portal output requires png, png_sequence, gif, y4m, wav, svg, mp4, or mov",
                ));
            }
        };
        begin_portal_render(
            slf,
            PortalRenderRequest {
                destination,
                width,
                height,
                fps,
                threads,
                seed,
                format,
                reproducible,
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn _begin_studio_capture(
        slf: &Bound<'_, Self>,
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
        portal_studio::begin(
            slf, name, build, source, width, height, fps, threads, seed, max_frames, max_bytes,
        )
    }

    fn _finish_studio_capture(slf: &Bound<'_, Self>) -> PyResult<portal_studio::Recording> {
        portal_studio::finish(slf)
    }

    fn _serve_studio_live(slf: &Bound<'_, Self>) -> PyResult<()> {
        portal_studio::live::serve(slf)
    }

    /// Cancel an active generation and join its ordered output worker.
    fn _abort_render(slf: &Bound<'_, Self>) -> PyResult<()> {
        let render = Arc::clone(&slf.borrow().render);
        let session = render
            .lock()
            .map_err(|_| PyRuntimeError::new_err("portal render session lock was poisoned"))?
            .take();
        if let Some(session) = session {
            session.abort();
        }
        Ok(())
    }

    /// Finalize the active native generation and return its exact artifact
    /// receipt: path, frames, bytes, digest, renderer identity, threads.
    #[pyo3(signature = (camera=None, light_position=None))]
    fn _finish_render(
        slf: &Bound<'_, Self>,
        camera: Option<PyRef<'_, PyCameraFrameCore>>,
        light_position: Option<[f64; 3]>,
    ) -> PyResult<(String, u64, u64, String, String, usize)> {
        if matches!(
            slf.borrow()
                .render
                .lock()
                .map_err(|_| PyRuntimeError::new_err("render generation lock poisoned"))?
                .as_ref(),
            Some(PortalRenderSession::Preview(_))
        ) {
            return Err(PyRuntimeError::new_err(
                "Studio capture is not a file render; use _finish_studio_capture",
            ));
        }
        portal_playback::synchronize(slf)?;
        synchronize_portal_camera(slf)?;
        let engine = Rc::clone(&slf.borrow().engine);
        let render = Arc::clone(&slf.borrow().render);
        match (camera, light_position) {
            (Some(camera), Some(light_position)) => render
                .lock()
                .map_err(|_| PyRuntimeError::new_err("portal render session lock was poisoned"))?
                .as_mut()
                .ok_or_else(|| PyRuntimeError::new_err("no portal render generation is active"))?
                .bind_camera(camera.frame.clone(), light_position, None, None)?,
            (None, None) => {}
            _ => {
                return Err(PyValueError::new_err(
                    "camera frame and light position must be supplied together",
                ));
            }
        }
        let needs_final_capture = render
            .lock()
            .map_err(|_| PyRuntimeError::new_err("portal render session lock was poisoned"))?
            .as_ref()
            .is_some_and(PortalRenderSession::needs_final_capture);
        if needs_final_capture {
            let mut sink = PortalSceneSink {
                render: Arc::clone(&render),
                ..PortalSceneSink::default()
            };
            engine
                .borrow_mut()
                .show(&mut sink)
                .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        }
        let session = render
            .lock()
            .map_err(|_| PyRuntimeError::new_err("portal render session lock was poisoned"))?
            .take()
            .ok_or_else(|| PyRuntimeError::new_err("no portal render generation is active"))?;
        // ubs:ignore — finalizes a frame-render session; no token, secret, or randomness exists.
        let (report, engine, threads) = session.finish(&engine.borrow())?;
        {
            let mut scene = slf.borrow_mut();
            scene.render_invocations = report.invocations;
            scene.render_audio_inputs = report.audio_inputs;
            scene.render_routes = report.routes;
        }
        Ok((
            report.path.to_string_lossy().into_owned(),
            report.frame_count,
            report.bytes,
            report.digest.to_hex(),
            engine,
            threads,
        ))
    }

    /// The last render's `(planar, camera, reason)`: frames rasterized on
    /// Lumen's retained 2D route and on the camera route, and why the first
    /// camera-route frame could not take the planar one (fm-sq8.11).
    #[getter]
    fn _render_routes(slf: &Bound<'_, Self>) -> (u64, u64, Option<&'static str>) {
        let routes = slf.borrow().render_routes;
        (routes.planar, routes.camera, routes.first_camera_reason)
    }

    /// The last render's `(teams used, teams planned, threads per team,
    /// frames in flight)`: what it used of its execution plan (fm-sq8.11).
    #[getter]
    fn _render_plan(slf: &Bound<'_, Self>) -> (u64, u64, u64, u64) {
        let routes = slf.borrow().render_routes;
        (
            routes.teams_used,
            routes.teams_planned,
            routes.team_threads,
            routes.frames_in_flight,
        )
    }

    #[getter]
    fn _render_audio_inputs(slf: &Bound<'_, Self>) -> PyResult<Py<PyList>> {
        portal_audio::input_facts(slf)
    }

    #[getter]
    fn _render_invocations(slf: &Bound<'_, Self>) -> PyResult<Py<PyList>> {
        let invocations = PyList::empty(slf.py());
        for invocation in &slf.borrow().render_invocations {
            let fact = PyDict::new(slf.py());
            let provenance = &invocation.provenance;
            fact.set_item("tool_path", provenance.tool_path.to_string_lossy().as_ref())?;
            fact.set_item("tool_sha256", &provenance.tool_sha256_hex)?;
            fact.set_item("tool_version", &provenance.tool_version)?;
            fact.set_item(
                "bound_tool_path",
                provenance.bound_tool_path.to_string_lossy().as_ref(),
            )?;
            fact.set_item("process_mechanism", &provenance.process_mechanism)?;
            fact.set_item("process_policy_version", provenance.process_policy_version)?;
            let image = PyDict::new(slf.py());
            image.set_item(
                "format",
                match provenance.native_image.format {
                    fmn_platform::process::NativeExecutableFormat::Elf64 => "elf64",
                    fmn_platform::process::NativeExecutableFormat::MachO64 => "macho64",
                    fmn_platform::process::NativeExecutableFormat::MachOUniversal => {
                        "macho-universal"
                    }
                    fmn_platform::process::NativeExecutableFormat::Pe32Plus => "pe32plus",
                },
            )?;
            image.set_item(
                "architecture",
                match provenance.native_image.architecture {
                    fmn_platform::process::NativeExecutableArchitecture::X86_64 => "x86_64",
                    fmn_platform::process::NativeExecutableArchitecture::Aarch64 => "aarch64",
                },
            )?;
            image.set_item("file_bytes", provenance.native_image.file_bytes)?;
            image.set_item("policy_version", provenance.native_image.policy_version)?;
            fact.set_item("native_image", image)?;
            fact.set_item("encoder", &provenance.encoder)?;
            fact.set_item("argv", &provenance.argv)?;
            invocations.append(fact)?;
        }
        Ok(invocations.unbind())
    }

    #[pyo3(signature = (*mobjects))]
    fn add<'py>(
        slf: &Bound<'py, Self>,
        mobjects: &Bound<'py, PyTuple>,
    ) -> PyResult<Bound<'py, Self>> {
        let handles = bind_scene_mobjects(slf, mobjects, "add")?;
        slf.borrow()
            .engine
            .borrow_mut()
            .add(&handles)
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        Ok(slf.clone())
    }

    #[pyo3(signature = (*mobjects))]
    fn bring_to_front<'py>(
        slf: &Bound<'py, Self>,
        mobjects: &Bound<'py, PyTuple>,
    ) -> PyResult<Bound<'py, Self>> {
        let handles = bind_scene_mobjects(slf, mobjects, "bring_to_front")?;
        slf.borrow()
            .engine
            .borrow_mut()
            .bring_to_front(&handles)
            .map_err(native_error)?;
        Ok(slf.clone())
    }

    #[pyo3(signature = (*mobjects))]
    fn bring_to_back<'py>(
        slf: &Bound<'py, Self>,
        mobjects: &Bound<'py, PyTuple>,
    ) -> PyResult<Bound<'py, Self>> {
        let handles = bind_scene_mobjects(slf, mobjects, "bring_to_back")?;
        slf.borrow()
            .engine
            .borrow_mut()
            .bring_to_back(&handles)
            .map_err(native_error)?;
        Ok(slf.clone())
    }

    #[pyo3(signature = (mobject, *replacements))]
    fn replace<'py>(
        slf: &Bound<'py, Self>,
        mobject: &Bound<'py, PyAny>,
        replacements: &Bound<'py, PyTuple>,
    ) -> PyResult<Bound<'py, Self>> {
        let Some(source) = scene_root_handle(slf, mobject) else {
            return Ok(slf.clone());
        };
        let replacements = bind_scene_mobjects(slf, replacements, "replace")?;
        slf.borrow()
            .engine
            .borrow_mut()
            .replace(source, &replacements)
            .map_err(native_error)?;
        Ok(slf.clone())
    }

    fn get_mobjects<'py>(slf: &Bound<'py, Self>) -> PyResult<Vec<Py<PyAny>>> {
        slf.getattr("mobjects")?.extract()
    }

    /// Reassert the exact handle-to-Python identity mapping for an already
    /// restored engine root batch. This deliberately performs no Stage write:
    /// SceneState's canonical checkpoint bytes must survive proxy reseating.
    #[pyo3(signature = (*mobjects))]
    fn _reseat_engine_roots(slf: &Bound<'_, Self>, mobjects: &Bound<'_, PyTuple>) -> PyResult<()> {
        let handles = scene_proxy_handles(slf, mobjects)?;
        let roots = slf.borrow().engine.borrow().stage().roots().to_vec();
        if handles != roots {
            return Err(PyRuntimeError::new_err(
                "captured SceneState proxies do not match restored engine roots",
            ));
        }
        for (handle, proxy) in handles.into_iter().zip(mobjects.iter()) {
            register_proxy(slf.py(), slf, handle, &proxy)?;
        }
        Ok(())
    }

    #[pyo3(signature = (*mobjects_to_keep))]
    fn remove_all_except<'py>(
        slf: &Bound<'py, Self>,
        mobjects_to_keep: &Bound<'py, PyTuple>,
    ) -> PyResult<Bound<'py, Self>> {
        let handles = bind_scene_mobjects(slf, mobjects_to_keep, "remove_all_except")?;
        slf.borrow()
            .engine
            .borrow_mut()
            .remove_all_except(&handles)
            .map_err(native_error)?;
        Ok(slf.clone())
    }

    #[pyo3(signature = (*mobjects))]
    fn remove<'py>(
        slf: &Bound<'py, Self>,
        mobjects: &Bound<'py, PyTuple>,
    ) -> PyResult<Bound<'py, Self>> {
        let handles = scene_removal_handles(slf, mobjects)?;
        slf.borrow().engine.borrow_mut().remove(&handles);
        Ok(slf.clone())
    }

    fn clear<'py>(slf: &Bound<'py, Self>) -> Bound<'py, Self> {
        slf.borrow().engine.borrow_mut().clear();
        slf.clone()
    }

    fn root_count(&self) -> usize {
        crossing::record(CrossingClass::Other);
        self.engine.borrow().stage().roots().len()
    }

    /// The Reference's `Scene.time` float attribute (scene.py:121), read
    /// from the native clock; scene updaters read `self.time` as a value.
    #[getter]
    fn time(&self) -> f64 {
        crossing::record(CrossingClass::Other);
        self.engine.borrow().stage().time()
    }

    /// The Reference advances `self.time += dt` itself; here the drift-free
    /// rational frame clock (BN-02) owns time, so assignment is refused.
    #[setter]
    fn set_time(&self, _value: &Bound<'_, PyAny>) -> PyResult<()> {
        Err(pyo3::exceptions::PyAttributeError::new_err(
            "Scene.time is read-only: FrankenManim's drift-free rational frame clock \
             (BN-02) owns scene time, so it cannot be assigned or rewound",
        ))
    }

    fn increment_time(&self, dt: f64) {
        crossing::record(CrossingClass::Other);
        self.engine.borrow_mut().stage_mut().increment_time(dt);
    }

    #[pyo3(signature = (sound_file, time_offset=0.0, gain=None, gain_to_background=None))]
    fn _add_sound(
        slf: &Bound<'_, Self>,
        sound_file: String,
        time_offset: f64,
        gain: Option<f64>,
        gain_to_background: Option<f64>,
    ) -> PyResult<()> {
        crossing::record(CrossingClass::Other);
        if portal_playback::requested_skip(slf)? {
            return Ok(());
        }
        let mut sound_file = PathBuf::from(sound_file);
        let active = slf
            .borrow()
            .render
            .lock()
            .map_err(|_| PyRuntimeError::new_err("portal render session lock was poisoned"))?
            .is_some();
        if active && sound_file.is_relative() {
            sound_file = std::env::current_dir()
                .map_err(|error| PyOSError::new_err(error.to_string()))?
                .join(sound_file);
        }
        let engine = Rc::clone(&slf.borrow().engine);
        let mut scene = engine.borrow_mut();
        // A cue may follow stop_skipping before the next play/wait. Record it
        // without changing the capture policy of an in-flight segment (a
        // Python updater can call this same method inside its release window).
        let skipping = scene.is_skipping();
        scene.set_skipping(false);
        let result = scene
            .add_sound(sound_file, time_offset, gain, gain_to_background)
            .map(|_| ())
            .map_err(native_error);
        scene.set_skipping(skipping);
        result
    }

    /// Engine-truth diagnostics for the permanent bridge acceptance suite.
    /// Each tuple is `(path, frame, fps, offset, gain, background_gain)`.
    fn _sound_request_facts(&self) -> Vec<SoundRequestFact> {
        crossing::record(CrossingClass::Other);
        self.engine
            .borrow()
            .sound_requests()
            .iter()
            .map(|request| {
                (
                    request.sound_file.to_string_lossy().into_owned(),
                    request.time.frames(),
                    request.time.fps(),
                    request.time_offset,
                    request.gain,
                    request.gain_to_background,
                )
            })
            .collect()
    }

    fn _engine_roots<'py>(
        slf: &Bound<'py, Self>,
        shell_factory: &Bound<'py, PyAny>,
    ) -> PyResult<Vec<Py<PyAny>>> {
        let py = slf.py();
        // fm-5wq.31: this projection walks the whole scene family, and
        // per-frame admission reads `Scene.mobjects` every frame (62 ms of
        // PrimeRace's frames at ~19K members). Skip it while no root, edge
        // or proxy identity has changed since the last complete one, and
        // after edge changes re-verify only the lists whose child epoch
        // moved. What that forgoes: a Python child list edited around its
        // live-list methods (`list.append(mob.submobjects, x)`) is re-synced
        // when its native list next changes, not at this read.
        let key = projection_key(slf);
        if let Some(roots) = projected_roots(slf, key) {
            return Ok(roots);
        }
        // An edge or root change with no proxy-identity change (most
        // frames of an edited scene) re-verifies only the moved lists.
        let previous = slf.borrow().roots_projection.borrow_mut().take();
        if let Some(previous) = previous.filter(|previous| previous.key.1 == key.1)
            && let Some(roots) = reproject_moved(slf, key, &previous.verified)?
        {
            return Ok(roots);
        }
        let engine = Rc::clone(&slf.borrow().engine);
        // Choreo can create composition containers without a Python proxy.
        // Snapshot the native graph first, then release the Stage borrow
        // before allocating shells or touching Python family/back edges.
        let (roots, graph) = {
            let runtime = engine.borrow();
            let stage = runtime.stage();
            let roots = stage.roots().to_vec();
            let mut seen: HashSet<Mob, ByAddress> = HashSet::default();
            let mut graph = Vec::new();
            for root in &roots {
                for mob in stage.family(*root) {
                    if seen.insert(mob) {
                        let entry = stage.get(mob).ok_or_else(|| {
                            StaleHandleError::new_err("scene family contains a stale handle")
                        })?;
                        graph.push((
                            mob,
                            entry.buffer.schema().offset("fill_rgba").is_some(),
                            entry.submobjects().to_vec(),
                            entry.children_epoch(),
                        ));
                    }
                }
            }
            (roots, graph)
        };
        let mut proxies: HashMap<Mob, Bound<'py, PyAny>, ByAddress> =
            HashMap::with_capacity_and_hasher(graph.len(), ByAddress::default());
        for (mob, vector_records, _, _) in &graph {
            let proxy = if let Some(proxy) = live_proxy(py, slf, *mob) {
                proxy
            } else {
                let proxy = shell_factory.call1((*vector_records,))?;
                {
                    let bridge = proxy.cast::<BridgeMobject>()?;
                    let mut cell = bridge.borrow_mut();
                    cell.nursery = None;
                    cell.engine = Some(Rc::clone(&engine));
                    cell.mob = Some(*mob);
                    cell.initialized = true;
                }
                engine
                    .borrow_mut()
                    .stage_mut()
                    .pin(*mob)
                    .map_err(stage_error)?;
                register_proxy(py, slf, *mob, &proxy)?;
                proxy.setattr("_scene", slf)?;
                proxy
            };
            proxies.insert(*mob, proxy);
        }
        for (mob, _, children, _) in &graph {
            let submobjects = proxies[mob].getattr("submobjects")?;
            // Re-projecting a list that already holds exactly these proxies
            // in this order clears and re-extends the same items with no
            // back-edge change. Skip that call: this walk covers the whole
            // scene family, so unchanged lists would otherwise cost one
            // Python call per member on every `Scene.mobjects` read.
            if let Ok(current) = submobjects.cast::<PyList>()
                && current.len() == children.len()
                && current
                    .iter()
                    .zip(children)
                    .all(|(item, child)| item.is(&proxies[child]))
            {
                continue;
            }
            let children = PyList::new(py, children.iter().map(|child| &proxies[child]))?;
            submobjects.call_method1("_replace_projection", (children,))?;
        }
        let verified = graph
            .iter()
            .map(|(mob, _, _, epoch)| (*mob, *epoch))
            .collect();
        store_projection(slf, key, &roots, verified);
        Ok(roots
            .into_iter()
            .map(|mob| proxies[&mob].clone().unbind())
            .collect())
    }

    /// Rung 0 (always-correct default): Python updater callbacks run with
    /// no Scene/Stage borrow live, one native→Python crossing per updater.
    /// After they finish, Marionette advances time and runs native updaters.
    fn update(slf: &Bound<'_, Self>, dt: f64) -> PyResult<()> {
        let python_ns = run_python_updaters(slf, dt)?;
        let native_start = Instant::now();
        slf.borrow().engine.borrow_mut().stage_mut().update(dt);
        let native_ns = u64::try_from(native_start.elapsed().as_nanos()).unwrap_or(u64::MAX);
        crossing::record_phase(python_ns, native_ns);
        Ok(())
    }

    /// Rung 1 (explicit opt-in, fm-zoi §15.2 Rev 4): the updater phase
    /// crosses native→Python ONCE per frame. The bootstrap's
    /// `Scene._dispatch_updater_batch` staticmethod iterates the same target
    /// list in the same order, snapshots each mobject's `updaters` at that
    /// mobject's turn (identical to rung 0's lazy snapshot), and invokes
    /// `_dispatch_updater` per updater inside Python. The batch's return is
    /// the single batched dirty-propagation crossing for the whole callback
    /// group. Declared semantics: identical ordering and identical
    /// observable state after each frame; liveness of proxies is resolved
    /// once at frame start (frame-atomic).
    fn update_batched(slf: &Bound<'_, Self>, dt: f64) -> PyResult<()> {
        let targets = update_targets(slf);
        let py = slf.py();
        let mut batch = Vec::with_capacity(targets.len() + 1);
        batch.push(camera_frame(slf)?);
        for target in targets {
            if let Some(proxy) = live_proxy(py, slf, target) {
                batch.push(proxy);
            }
        }
        let python_start = Instant::now();
        if !batch.is_empty() {
            crossing::record(CrossingClass::MethodDispatch);
            let args = PyTuple::new(
                py,
                [
                    PyTuple::new(py, batch)?.into_any(),
                    dt.into_pyobject(py)?.into_any(),
                ],
            )?;
            method_cache::call_static_cached1(slf.as_any(), "_dispatch_updater_batch", &args)?;
            // The batch return transfers the frame's accumulated dirty state
            // to native in one crossing (batched per callback group).
            crossing::record(CrossingClass::DirtyPropagation);
        }
        let python_ns = u64::try_from(python_start.elapsed().as_nanos()).unwrap_or(u64::MAX);
        let native_start = Instant::now();
        slf.borrow().engine.borrow_mut().stage_mut().update(dt);
        let native_ns = u64::try_from(native_start.elapsed().as_nanos()).unwrap_or(u64::MAX);
        crossing::record_phase(python_ns, native_ns);
        Ok(())
    }

    fn run_transform(
        slf: &Bound<'_, Self>,
        mobject: &Bound<'_, BridgeMobject>,
        target: &Bound<'_, BridgeMobject>,
        steps: usize,
    ) -> PyResult<()> {
        let engine = Rc::clone(&slf.borrow().engine);
        for endpoint in [mobject, target] {
            let (endpoint_engine, _) = bound_parts(&endpoint.borrow())?;
            if !same_engine(&engine, &endpoint_engine) {
                return Err(ForeignStageError::new_err(
                    "transform endpoints must belong to this Scene",
                ));
            }
        }
        crossing::record(CrossingClass::MethodDispatch);
        let copy = method_cache::call_cached0(mobject.as_any(), "__copy__")?;
        for step in 0..=steps {
            let alpha = if steps == 0 {
                1.0
            } else {
                step as f64 / steps as f64
            };
            crossing::record(CrossingClass::MethodDispatch);
            let args = PyTuple::new(
                slf.py(),
                [
                    copy.clone(),
                    target.as_any().clone(),
                    alpha.into_pyobject(slf.py())?.into_any(),
                ],
            )?;
            method_cache::call_cached1(mobject.as_any(), "interpolate", &args)?;
        }
        Ok(())
    }

    /// setup → construct → tear_down through Python MRO. tear_down runs even
    /// when construct raises; the original construct exception remains primary.
    fn _run_lifecycle(slf: &Bound<'_, Self>) -> PyResult<()> {
        crossing::record(CrossingClass::MethodDispatch);
        method_cache::call_cached0(slf.as_any(), "setup")?;
        crossing::record(CrossingClass::MethodDispatch);
        let construct = method_cache::call_cached0(slf.as_any(), "construct");
        crossing::record(CrossingClass::MethodDispatch);
        let teardown = method_cache::call_cached0(slf.as_any(), "tear_down");
        match (construct, teardown) {
            // EndScene is normal early completion, not a primary failure.
            // A teardown error must still abort the owned output generation.
            (Err(error), Err(teardown)) if error.is_instance_of::<EndScene>(slf.py()) => {
                Err(teardown)
            }
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(error),
            (Ok(_), Ok(_)) => Ok(()),
        }
    }

    /// Construct the ordinary native animation for ordered lifecycle calls
    /// inside a mixed composition. Endpoints are already adopted by Python;
    /// construction does not begin the animation or capture its start state.
    fn _native_animation_driver(
        &self,
        spec: &Bound<'_, PyAny>,
    ) -> PyResult<PyNativeAnimationDriver> {
        let spec = parse_anim_spec(&self.engine, spec)?;
        let animation = build_native_animation(self.engine.borrow_mut().stage_mut(), spec)?;
        Ok(PyNativeAnimationDriver {
            engine: Rc::clone(&self.engine),
            animation,
            begun: false,
            finished: false,
        })
    }

    /// Drive Choreo's six-step play contract and the active output session.
    /// Specs are `(kind, mobject, target, run_time, rate_func, lag_ratio,
    /// params)`, with composition members under `params["members"]`.
    /// Native groups derive the timing slots; ordered host callbacks run
    /// between Stage borrows. An optional camera track shares the segment.
    #[pyo3(signature = (specs, callbacks, camera, run_time, rate_func, lag_ratio))]
    fn _play_animations(
        slf: &Bound<'_, Self>,
        specs: Vec<Bound<'_, PyAny>>,
        callbacks: Vec<Option<Py<PyAny>>>,
        camera: Option<(Bound<'_, PyCameraFrameCore>, Bound<'_, PyCameraFrameCore>)>,
        run_time: Option<f64>,
        rate_func: Option<Bound<'_, PyAny>>,
        lag_ratio: Option<f64>,
    ) -> PyResult<Vec<f64>> {
        portal_playback::synchronize(slf)?;
        let engine = Rc::clone(&slf.borrow().engine);
        if callbacks.len() != specs.len() {
            return Err(PyValueError::new_err(
                "animation callback table must align one-for-one with specs",
            ));
        }
        // fm-5wq.4.97: a mixed play rides the same per-animation release
        // loop — the engine yields one boundary per animation in index
        // order (native ones already interpolated inside prepare), so
        // Python-authored members interleave with native kinds and their
        // mutations stay visible to later animations in the same frame.
        let callback_count = callbacks.iter().flatten().count();

        // Resolve every Python-side value before the engine borrow.
        let play_rate = rate_func.as_ref().map(rate_func_from_py).transpose()?;
        let mut resolved = Vec::with_capacity(specs.len());
        for spec in &specs {
            resolved.push(parse_anim_spec(&engine, spec)?);
        }

        let release_for_python_updaters = portal_playback::requires_public_scene_update(slf)?
            || has_python_updaters(slf)?
            || portal_has_frame_render(slf)?;
        for callback in callbacks.iter().flatten() {
            crossing::record(CrossingClass::MethodDispatch);
            callback.bind(slf.py()).call_method0("begin")?;
        }
        let (animations, start_time) = {
            let mut scene = engine.borrow_mut();
            let start_time = scene.stage().time();
            let mut animations: Vec<Box<dyn fmn_anim::Animation>> =
                Vec::with_capacity(resolved.len());
            for spec in resolved {
                animations.push(build_native_animation(scene.stage_mut(), spec)?);
            }
            (animations, start_time)
        };
        let effective_run_time = run_time.unwrap_or(fmn_anim::DEFAULT_ANIMATION_RUN_TIME);
        let camera_lerp = camera
            .map(|(live, target)| -> PyResult<CameraLerp> {
                Ok(CameraLerp {
                    start: live.borrow().frame.clone(),
                    end: target.borrow().frame.clone(),
                    core: live.unbind(),
                    start_time,
                    run_time: effective_run_time,
                    rate: play_rate.clone().unwrap_or_default(),
                })
            })
            .transpose()?;
        let overrides = fmn_scene::PlayOverrides {
            run_time,
            rate_func: play_rate,
            lag_ratio,
        };
        let mut sink = PortalSceneSink {
            camera: camera_lerp,
            render: Arc::clone(&slf.borrow().render),
            ..PortalSceneSink::default()
        };
        let map_play_error = |error: fmn_scene::SceneError| {
            let text = error.to_string();
            if text.contains("become between records of different schemas") {
                // The engine's precise cross-schema refusal, plus the design
                // it awaits: revealing a sampled Surface is the Reference's
                // Surface.pointwise_become_partial u-slice mechanism — an
                // fmn-anim/fmn-mobject tranche, not a binding.
                PyRuntimeError::new_err(format!(
                    "{text}; revealing a sampled Surface awaits the engine's \
                     surface partial-reveal mechanism \
                     (Surface.pointwise_become_partial)"
                ))
            } else {
                scene_error(error)
            }
        };
        if animations.is_empty() {
            if sink.camera.is_none() {
                return Ok(Vec::new());
            }
            if release_for_python_updaters {
                // A camera-only play uses the wait cursor. Preserve its stock
                // prologue; authored public updates own the whole zero-dt pass.
                let public_initial = if portal_playback::requires_public_scene_update(slf)? {
                    portal_playback::run_scene_update_phase(slf, 0.0)?.1
                } else {
                    false
                };
                let mut wait = {
                    let mut runtime = engine.borrow_mut();
                    if public_initial {
                        runtime
                            .begin_stepped_wait_after_updaters(Some(effective_run_time), &mut sink)
                    } else {
                        runtime.begin_stepped_wait(Some(effective_run_time), &mut sink)
                    }
                }
                .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
                let drive_result: PyResult<()> = (|| {
                    loop {
                        let release = engine
                            .borrow_mut()
                            .prepare_stepped_wait_frame(&mut wait)
                            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
                        let Some(release) = release else {
                            break;
                        };
                        sink.apply_camera_before_updaters(slf.py(), release.time.to_f64())?;
                        let (python_ns, completed) =
                            portal_playback::run_scene_update_phase(slf, release.dt)?;
                        if completed {
                            wait.mark_scene_updaters_complete().map_err(scene_error)?;
                        }
                        synchronize_portal_camera(slf)?;
                        let native_start = Instant::now();
                        engine
                            .borrow_mut()
                            .complete_stepped_wait_frame(&mut wait, &mut sink)
                            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
                        let native_ns =
                            u64::try_from(native_start.elapsed().as_nanos()).unwrap_or(u64::MAX);
                        crossing::record_phase(python_ns, native_ns);
                    }
                    sink.finish_camera_before_updaters(slf.py())?;
                    let (_, completed) = portal_playback::run_scene_update_phase(slf, 0.0)?;
                    synchronize_portal_camera(slf)?;
                    if !completed {
                        engine.borrow_mut().stage_mut().update(0.0);
                    }
                    Ok(())
                })();
                if let Err(error) = drive_result {
                    engine.borrow_mut().abort_stepped_wait(wait, &mut sink);
                    return Err(error);
                }
                engine
                    .borrow_mut()
                    .finish_stepped_wait(wait, &mut sink)
                    .map(|_| ())
                    .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
            } else {
                engine
                    .borrow_mut()
                    .wait(Some(effective_run_time), &mut sink)
                    .map(|_| ())
                    .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
            }
        } else if release_for_python_updaters || callback_count != 0 {
            // Choreo stops after interpolation + rational clock advance. The
            // Scene RefCell borrow is then genuinely gone while Python
            // updaters run; completing the same frame performs native scene
            // updaters, event dispatch, immutable freeze, and capture.
            let mut play = engine
                .borrow_mut()
                .begin_stepped_play(animations, overrides, &mut sink)
                .map_err(&map_play_error)?
                .ok_or_else(|| PyRuntimeError::new_err("a nonempty play did not open"))?;
            let drive_result: PyResult<()> = (|| {
                loop {
                    if callback_count != 0 {
                        loop {
                            let animation = engine
                                .borrow_mut()
                                .prepare_stepped_play_animation(&mut play)
                                .map_err(&map_play_error)?;
                            let Some(animation) = animation else {
                                break;
                            };
                            let Some(callback) = callbacks[animation.animation_index].as_ref()
                            else {
                                // A native-kind animation in a mixed play:
                                // its interpolate already ran natively in
                                // prepare; this boundary is release
                                // ordering only.
                                continue;
                            };
                            crossing::record(CrossingClass::MethodDispatch);
                            if callback.bind(slf.py()).hasattr("update_mobjects")? {
                                callback
                                    .bind(slf.py())
                                    .call_method1("update_mobjects", (animation.dt,))?;
                            }
                            callback
                                .bind(slf.py())
                                .call_method1("interpolate", (animation.alpha,))?;
                        }
                    }
                    let release = engine
                        .borrow_mut()
                        .prepare_stepped_play_frame(&mut play)
                        .map_err(&map_play_error)?;
                    let Some(release) = release else {
                        break;
                    };
                    sink.apply_camera_before_updaters(slf.py(), release.time.to_f64())?;
                    let (python_ns, completed) =
                        portal_playback::run_scene_update_phase(slf, release.dt)?;
                    if completed {
                        play.mark_scene_updaters_complete()
                            .map_err(&map_play_error)?;
                    }
                    synchronize_portal_camera(slf)?;
                    let native_start = Instant::now();
                    engine
                        .borrow_mut()
                        .complete_stepped_play_frame(&mut play, &mut sink)
                        .map_err(&map_play_error)?;
                    let native_ns =
                        u64::try_from(native_start.elapsed().as_nanos()).unwrap_or(u64::MAX);
                    crossing::record_phase(python_ns, native_ns);
                }
                for callback in callbacks.iter().flatten() {
                    crossing::record(CrossingClass::MethodDispatch);
                    callback.bind(slf.py()).call_method0("finish")?;
                    if callback.bind(slf.py()).hasattr("clean_up_from_scene")? {
                        callback
                            .bind(slf.py())
                            .call_method1("clean_up_from_scene", (slf,))?;
                    }
                }
                sink.finish_camera_before_updaters(slf.py())?;
                Ok(())
            })();
            if let Err(error) = drive_result {
                engine.borrow_mut().abort_stepped_play(play, &mut sink);
                return Err(error);
            }
            // Reference Scene.finish_animations first finishes every
            // animation (which resumes animation-suspended mobjects), then
            // performs one final update_mobjects(0). Split that updater pass
            // at the same host-language release boundary as ordinary frames:
            // Python first, native second, with no Scene borrow crossing the
            // callback.
            let resumed = match engine
                .borrow_mut()
                .finish_stepped_play_animations(&mut play)
                .map_err(&map_play_error)
            {
                Ok(resumed) => resumed,
                Err(error) => {
                    engine.borrow_mut().abort_stepped_play(play, &mut sink);
                    return Err(error);
                }
            };
            if let Err(error) = run_resumed_python_updaters(slf, &resumed) {
                engine.borrow_mut().abort_stepped_play(play, &mut sink);
                return Err(error);
            }
            let final_phase =
                portal_playback::run_scene_update_phase(slf, 0.0).and_then(|(_, completed)| {
                    if completed {
                        play.mark_final_scene_updaters_complete()
                            .map_err(&map_play_error)?;
                    }
                    Ok(())
                });
            if let Err(error) = final_phase {
                engine.borrow_mut().abort_stepped_play(play, &mut sink);
                return Err(error);
            }
            if let Err(error) = synchronize_portal_camera(slf) {
                engine.borrow_mut().abort_stepped_play(play, &mut sink);
                return Err(error);
            }
            engine
                .borrow_mut()
                .finish_stepped_play(play, &mut sink)
                .map(|_| ())
                .map_err(&map_play_error)?;
        } else {
            engine
                .borrow_mut()
                .play(animations, overrides, &mut sink)
                .map(|_| ())
                .map_err(&map_play_error)?;
        }
        let PortalSceneSink {
            alphas,
            camera,
            camera_error,
            render: _,
            camera_preapplied: _,
            camera_finished_before_updaters,
        } = sink;
        if let Some(error) = camera_error {
            return Err(error);
        }
        if let Some(camera) = camera
            && !camera_finished_before_updaters
        {
            camera.finish_exact(slf.py())?;
        }
        Ok(alphas)
    }

    /// Engine-truth structural facts (the corpus baselines): draw-list
    /// root count, total family membership, and the aggregate family bbox
    /// (zero boxes skipped) — measured on the Stage directly, so the
    /// numbers never depend on Python proxy liveness or GC timing.
    #[allow(clippy::type_complexity)]
    fn _engine_facts(slf: &Bound<'_, Self>) -> PyResult<(usize, usize, [f64; 3], [f64; 3])> {
        let engine = Rc::clone(&slf.borrow().engine);
        let scene = engine.borrow();
        let stage = scene.stage();
        let roots = stage.roots().to_vec();
        let mut family_total = 0usize;
        let mut low = [f64::INFINITY; 3];
        let mut high = [f64::NEG_INFINITY; 3];
        let mut any_box = false;
        for &root in &roots {
            family_total += stage.family(root).len();
            let bbox = stage.get_bounding_box(root);
            if bbox.min == [0.0; 3] && bbox.max == [0.0; 3] {
                continue;
            }
            any_box = true;
            for axis in 0..3 {
                low[axis] = low[axis].min(bbox.min[axis]);
                high[axis] = high[axis].max(bbox.max[axis]);
            }
        }
        if !any_box {
            low = [0.0; 3];
            high = [0.0; 3];
        }
        Ok((roots.len(), family_total, low, high))
    }

    /// Adopt a detached mobject graph into this scene's arena WITHOUT
    /// adding it to the draw list — the `.animate` target seam.
    fn _adopt(slf: &Bound<'_, Self>, mobject: &Bound<'_, BridgeMobject>) -> PyResult<()> {
        bind_graph(slf.py(), slf, mobject)?;
        Ok(())
    }

    /// Attach a [`PyFieldProbe`] to a bound mobject: a native updater
    /// records `field[lane]` of record 0 every frame (diagnostics only).
    fn _record_field_probe(
        slf: &Bound<'_, Self>,
        mobject: &Bound<'_, BridgeMobject>,
        field: String,
        lane: usize,
    ) -> PyResult<PyFieldProbe> {
        let engine = Rc::clone(&slf.borrow().engine);
        let (mob_engine, mob) = bound_parts(&mobject.borrow())?;
        if !same_engine(&engine, &mob_engine) {
            return Err(ForeignStageError::new_err(
                "the probe target must belong to this Scene",
            ));
        }
        let values: Rc<RefCell<Vec<f64>>> = Rc::new(RefCell::new(Vec::new()));
        let sink = Rc::clone(&values);
        engine
            .borrow_mut()
            .stage_mut()
            .add_updater(
                mob,
                move |stage: &mut fmn_mobject::Stage, target: Mob| {
                    // Animation begin-time copies carry updaters; record
                    // only the original entry, not its starting/target
                    // copies.
                    if target != mob {
                        return;
                    }
                    if let Some(entry) = stage.get(target)
                        && let Some(lanes) = entry.buffer.read(0, &field)
                        && let Some(value) = lanes.get(lane)
                    {
                        sink.borrow_mut().push(f64::from(*value));
                    }
                },
                false,
            )
            .map_err(stage_error)?;
        Ok(PyFieldProbe { values })
    }

    /// `Scene.wait(duration, stop_condition=..., ignore_presenter_mode=...)`
    /// over the native wait segment. An active portal render generation
    /// captures every immutable frame through Lumen + Reel; ordinary lifecycle
    /// probes keep the same sink with output disabled.
    #[pyo3(signature = (duration = None, stop_condition = None, ignore_presenter_mode = false))]
    fn _wait(
        slf: &Bound<'_, Self>,
        duration: Option<f64>,
        stop_condition: Option<&Bound<'_, PyAny>>,
        ignore_presenter_mode: bool,
    ) -> PyResult<()> {
        let engine = Rc::clone(&slf.borrow().engine);
        portal_playback::synchronize(slf)?;
        let needs_host_updates =
            portal_playback::requires_public_scene_update(slf)? || has_python_updaters(slf)?;
        if !needs_host_updates && stop_condition.is_none() && !portal_has_frame_render(slf)? {
            let mut sink = PortalSceneSink {
                render: Arc::clone(&slf.borrow().render),
                ..PortalSceneSink::default()
            };
            let result = if ignore_presenter_mode {
                engine
                    .borrow_mut()
                    .wait_ignoring_presenter(duration, &mut sink)
            } else {
                engine.borrow_mut().wait(duration, &mut sink)
            };
            return result
                .map(|_| ())
                .map_err(|error| PyRuntimeError::new_err(error.to_string()));
        }

        // Complete the initial zero-dt host phase while unborrowed. If it
        // dispatched a public update override, it also consumed native slots.
        let initial_completed = if needs_host_updates {
            portal_playback::run_scene_update_phase(slf, 0.0)?.1
        } else {
            false
        };
        let mut sink = PortalSceneSink {
            render: Arc::clone(&slf.borrow().render),
            ..PortalSceneSink::default()
        };
        let mut wait = {
            let mut runtime = engine.borrow_mut();
            if initial_completed {
                runtime.begin_stepped_wait_after_updaters(duration, &mut sink)
            } else {
                runtime.begin_stepped_wait(duration, &mut sink)
            }
        }
        .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        let drive_result: PyResult<()> = (|| {
            loop {
                let release = engine
                    .borrow_mut()
                    .prepare_stepped_wait_frame(&mut wait)
                    .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
                let Some(release) = release else {
                    break;
                };
                // Reinspect at every release: a stop condition, callback or
                // captured-frame observer can install a new update protocol.
                let (python_ns, completed) =
                    portal_playback::run_scene_update_phase(slf, release.dt)?;
                if completed {
                    wait.mark_scene_updaters_complete().map_err(scene_error)?;
                }
                synchronize_portal_camera(slf)?;
                let native_start = Instant::now();
                engine
                    .borrow_mut()
                    .complete_stepped_wait_frame(&mut wait, &mut sink)
                    .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
                let native_ns =
                    u64::try_from(native_start.elapsed().as_nanos()).unwrap_or(u64::MAX);
                crossing::record_phase(python_ns, native_ns);
                // The Reference checks after emitting the frame where the
                // predicate turns true. Keep the Scene borrow released so a
                // Python condition may safely inspect portal state.
                if let Some(condition) = stop_condition {
                    crossing::record(CrossingClass::Other);
                    if condition.call0()?.is_truthy()? {
                        break;
                    }
                }
            }
            Ok(())
        })();
        if let Err(error) = drive_result {
            engine.borrow_mut().abort_stepped_wait(wait, &mut sink);
            return Err(error);
        }
        engine
            .borrow_mut()
            .finish_stepped_wait(wait, &mut sink)
            .map(|_| ())
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))
    }

    fn _checkpoint_bytes(&self) -> PyResult<Vec<u8>> {
        self.engine
            .borrow_mut()
            .state_bytes()
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))
    }

    /// Restore the canonical SceneState document captured by
    /// `_checkpoint_bytes`. Durable decode validates the envelope and nested
    /// arena snapshot before the runtime reapplies its clock, RNG, play
    /// count, roots, records, and family graph.
    fn _restore_checkpoint_bytes(&self, bytes: Vec<u8>) -> PyResult<()> {
        let mut engine = self.engine.borrow_mut();
        engine
            .restore_state_bytes(&bytes)
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        let restored = engine
            .state_bytes()
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        if restored != bytes {
            let first_difference = restored
                .iter()
                .zip(&bytes)
                .position(|(restored, captured)| restored != captured)
                .unwrap_or_else(|| restored.len().min(bytes.len()));
            return Err(PyRuntimeError::new_err(format!(
                "native checkpoint restore did not round-trip: captured_len={}, restored_len={}, first_difference={first_difference}",
                bytes.len(),
                restored.len(),
            )));
        }
        Ok(())
    }
}

/// The engine's named rate-function catalog (fmn-core's single-argument
/// rate functions, the same pointers Choreo composes). Parameterized and
/// arbitrary Python callables refuse precisely — a per-frame Python
/// rate_func is a crossing-budget decision for a later rung.
fn anim_error(error: fmn_anim::AnimError) -> PyErr {
    PyRuntimeError::new_err(error.to_string())
}

fn scene_error(error: fmn_scene::SceneError) -> PyErr {
    match error {
        fmn_scene::SceneError::EndScene(error) => EndScene::new_err(error.to_string()),
        error => PyRuntimeError::new_err(error.to_string()),
    }
}

/// A native leaf driven inside an ordered mixed-composition release window.
/// Each method borrows the Stage only for native work. The enclosing Python
/// driver can therefore finish one child before the next native child takes
/// its starting copy, with no Scene borrow crossing a host callback.
#[pyclass(unsendable, name = "_NativeAnimationDriver")]
struct PyNativeAnimationDriver {
    engine: Engine,
    animation: Box<dyn fmn_anim::Animation>,
    begun: bool,
    finished: bool,
}

#[pymethods]
impl PyNativeAnimationDriver {
    fn get_run_time(&self) -> f64 {
        self.animation.get_run_time()
    }

    fn begin(&mut self) -> PyResult<()> {
        self.begun = true;
        self.finished = false;
        self.animation
            .begin(self.engine.borrow_mut().stage_mut())
            .map_err(anim_error)
    }

    fn update_mobjects(&mut self, dt: f64) {
        self.animation
            .update_mobjects(self.engine.borrow_mut().stage_mut(), dt);
    }

    fn interpolate(&mut self, alpha: f64) -> PyResult<()> {
        self.animation
            .interpolate(self.engine.borrow_mut().stage_mut(), alpha);
        self.animation
            .deferred_error()
            .map_or(Ok(()), |error| Err(anim_error(error)))
    }

    fn finish(&mut self, scene: &Bound<'_, PyScene>) -> PyResult<()> {
        if !same_engine(&self.engine, &scene.borrow().engine) {
            return Err(ForeignStageError::new_err(
                "animation belongs to another Scene",
            ));
        }
        if self.begun && !self.finished {
            self.animation.finish(self.engine.borrow_mut().stage_mut());
            self.finished = true;
            let mut resumed = Vec::new();
            self.animation
                .collect_resumed_updater_mobjects(&mut resumed);
            run_resumed_python_updaters(scene, &resumed)?;
        }
        self.animation
            .deferred_error()
            .map_or(Ok(()), |error| Err(anim_error(error)))
    }

    fn clean_up_from_scene(&mut self) {
        if self.finished {
            self.animation
                .clean_up_from_scene(self.engine.borrow_mut().stage_mut());
        }
    }

    fn abort(&mut self) {
        if self.begun && !self.finished {
            self.animation.abort(self.engine.borrow_mut().stage_mut());
            self.finished = true;
        }
    }
}

/// Timing/lifecycle slot for a top-level Python-authored Animation.
///
/// Its interpolation is intentionally empty: Choreo yields immediately after
/// this slot in step 2, the bridge releases the Scene borrow, and invokes the
/// Python animation at the slot's exact raw alpha before advancing to the next
/// animation. The slot still owns native animating/remover semantics and the
/// segment's authoritative timing.
struct PythonAnimationSlot {
    state: fmn_anim::AnimState,
}

impl PythonAnimationSlot {
    fn new(mobject: Mob, remover: bool) -> Self {
        let mut config = fmn_anim::AnimConfig {
            name: "PythonAnimationSlot".to_owned(),
            ..fmn_anim::AnimConfig::default()
        };
        config.remover = remover;
        Self {
            state: fmn_anim::AnimState::new(mobject, config),
        }
    }
}

impl fmn_anim::Animation for PythonAnimationSlot {
    fn state(&self) -> &fmn_anim::AnimState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut fmn_anim::AnimState {
        &mut self.state
    }

    fn interpolate_submobject(
        &mut self,
        _stage: &mut fmn_mobject::Stage,
        _mobs: &[Mob],
        _sub_alpha: f64,
    ) {
    }
}

/// One parsed play spec: a leaf animation or a composition carrying
/// nested members (fm-d3gt, the explicit-animation seam).
struct AnimSpec {
    kind: String,
    mob: Option<Mob>,
    target: Option<Mob>,
    run_time: Option<f64>,
    rate: Option<fmn_anim::RateFunc>,
    lag: Option<f64>,
    shift: [f64; 3],
    scale: f64,
    angle: f64,
    axis: [f64; 3],
    about_point: Option<[f64; 3]>,
    about_edge: [f64; 3],
    about_edge_opt: Option<[f64; 3]>,
    path_arc: f64,
    path_arc_axis: [f64; 3],
    stroke_color: Option<[f64; 3]>,
    stroke_width: Option<f64>,
    bounds_kind: String,
    mobs: Vec<Mob>,
    pairs: Vec<(Mob, Mob)>,
    int_round: String,
    stretch: Option<bool>,
    dim_to_match: Option<usize>,
    point: [f64; 3],
    point_color: Option<[f64; 3]>,
    color: Option<[f64; 3]>,
    scale_factor: f64,
    scale_value: f64,
    rotation_angle: f64,
    n_wiggles: f64,
    scale_about_point: Option<[f64; 3]>,
    rotate_about_point: Option<[f64; 3]>,
    time_width: f64,
    taper_width: f64,
    direction: [f64; 3],
    amplitude: f64,
    time_span: Option<(f64, f64)>,
    group_lag: f64,
    remover: bool,
    final_alpha_value: Option<f64>,
    suspend_mobject_updating: bool,
    surface_resolution: (usize, usize),
    surface_axis: usize,
    source_keys: Vec<(Mob, String)>,
    target_keys: Vec<(Mob, String)>,
    members: Vec<AnimSpec>,
}

#[allow(clippy::type_complexity)]
fn parse_anim_spec(engine: &Engine, spec: &Bound<'_, PyAny>) -> PyResult<AnimSpec> {
    let (kind, mobject, target, run_time, rate, lag, params): (
        String,
        Option<Bound<'_, BridgeMobject>>,
        Option<Bound<'_, BridgeMobject>>,
        Option<f64>,
        Option<Bound<'_, PyAny>>,
        Option<f64>,
        Bound<'_, PyDict>,
    ) = spec.extract()?;
    let rate = rate.as_ref().map(rate_func_from_py).transpose()?;
    let resolve = |proxy: &Bound<'_, BridgeMobject>| -> PyResult<Mob> {
        let (mob_engine, mob) = bound_parts(&proxy.borrow())?;
        if same_engine(engine, &mob_engine) {
            Ok(mob)
        } else {
            Err(ForeignStageError::new_err(
                "play endpoints must belong to this Scene",
            ))
        }
    };
    let get3 = |name: &str, default: [f64; 3]| -> PyResult<[f64; 3]> {
        match params.get_item(name)? {
            Some(value) => Ok(value.extract()?),
            None => Ok(default),
        }
    };
    let get1 = |name: &str, default: f64| -> PyResult<f64> {
        match params.get_item(name)? {
            Some(value) => Ok(value.extract()?),
            None => Ok(default),
        }
    };
    let get_bool = |name: &str, default: bool| -> PyResult<bool> {
        match params.get_item(name)? {
            Some(value) => value.extract(),
            None => Ok(default),
        }
    };
    let get_usize = |name: &str, default: usize| -> PyResult<usize> {
        match params.get_item(name)? {
            Some(value) => value.extract(),
            None => Ok(default),
        }
    };
    let get_usize_pair = |name: &str, default: (usize, usize)| -> PyResult<(usize, usize)> {
        match params.get_item(name)? {
            Some(value) => value.extract(),
            None => Ok(default),
        }
    };
    let get_keyed_parts = |name: &str| -> PyResult<Vec<(Mob, String)>> {
        let Some(values) = params.get_item(name)? else {
            return Ok(Vec::new());
        };
        values
            .try_iter()?
            .map(|item| {
                let (proxy, key): (Bound<'_, BridgeMobject>, String) = item?.extract()?;
                Ok((resolve(&proxy)?, key))
            })
            .collect()
    };
    let members = match params.get_item("members")? {
        Some(list) => list
            .try_iter()?
            .map(|item| parse_anim_spec(engine, &item?))
            .collect::<PyResult<Vec<_>>>()?,
        None => Vec::new(),
    };
    let mob = if kind == "native_callback" {
        let driver = params.get_item("driver")?.ok_or_else(|| {
            PyValueError::new_err("native_callback requires its native animation driver")
        })?;
        let driver = driver.cast::<PyNativeAnimationDriver>()?.borrow();
        if !same_engine(engine, &driver.engine) {
            return Err(ForeignStageError::new_err(
                "animation belongs to another Scene",
            ));
        }
        Some(driver.animation.state().mobject())
    } else {
        mobject.as_ref().map(&resolve).transpose()?
    };
    Ok(AnimSpec {
        kind,
        mob,
        target: target.as_ref().map(&resolve).transpose()?,
        run_time,
        rate,
        lag,
        shift: get3("shift", [0.0; 3])?,
        scale: get1("scale", 1.0)?,
        angle: get1("angle", std::f64::consts::PI)?,
        axis: get3("axis", [0.0, 0.0, 1.0])?,
        about_point: params
            .get_item("about_point")?
            .map(|value| value.extract())
            .transpose()?,
        about_edge: get3("about_edge", [0.0; 3])?,
        about_edge_opt: params
            .get_item("about_edge")?
            .map(|value| value.extract())
            .transpose()?,
        path_arc: get1("path_arc", 0.0)?,
        path_arc_axis: get3("path_arc_axis", [0.0, 0.0, 1.0])?,
        stroke_color: params
            .get_item("stroke_color")?
            .map(|value| value.extract())
            .transpose()?,
        stroke_width: params
            .get_item("stroke_width")?
            .map(|value| value.extract())
            .transpose()?,
        bounds_kind: match params.get_item("bounds_kind")? {
            Some(value) => value.extract()?,
            None => String::new(),
        },
        mobs: match params.get_item("mobs")? {
            Some(values) => values
                .try_iter()?
                .map(|item| {
                    let proxy: Bound<'_, BridgeMobject> = item?.extract()?;
                    resolve(&proxy)
                })
                .collect::<PyResult<Vec<_>>>()?,
            None => Vec::new(),
        },
        pairs: match params.get_item("matched_pairs")? {
            Some(values) => values
                .try_iter()?
                .map(|item| {
                    let (a, b): (Bound<'_, BridgeMobject>, Bound<'_, BridgeMobject>) =
                        item?.extract()?;
                    Ok((resolve(&a)?, resolve(&b)?))
                })
                .collect::<PyResult<Vec<_>>>()?,
            None => Vec::new(),
        },
        int_round: match params.get_item("int_round")? {
            Some(value) => value.extract()?,
            None => String::new(),
        },
        stretch: params
            .get_item("stretch")?
            .map(|value| value.extract())
            .transpose()?,
        dim_to_match: params
            .get_item("dim_to_match")?
            .map(|value| value.extract())
            .transpose()?,
        point: get3("point", [0.0; 3])?,
        point_color: params
            .get_item("point_color")?
            .map(|value| value.extract())
            .transpose()?,
        color: params
            .get_item("color")?
            .map(|value| value.extract())
            .transpose()?,
        scale_factor: get1("scale_factor", 1.2)?,
        scale_value: get1("scale_value", 1.1)?,
        rotation_angle: get1("rotation_angle", 0.01 * std::f64::consts::TAU)?,
        n_wiggles: get1("n_wiggles", 6.0)?,
        scale_about_point: params
            .get_item("scale_about_point")?
            .map(|value| value.extract())
            .transpose()?,
        rotate_about_point: params
            .get_item("rotate_about_point")?
            .map(|value| value.extract())
            .transpose()?,
        time_width: get1("time_width", 0.3)?,
        taper_width: get1("taper_width", 0.05)?,
        direction: get3("direction", [0.0, 1.0, 0.0])?,
        amplitude: get1("amplitude", 0.2)?,
        time_span: params
            .get_item("time_span")?
            .map(|value| value.extract())
            .transpose()?,
        group_lag: get1("lag_ratio", 0.0)?,
        remover: get_bool("remover", false)?,
        final_alpha_value: params
            .get_item("final_alpha_value")?
            .map(|value| value.extract())
            .transpose()?,
        suspend_mobject_updating: get_bool("suspend_mobject_updating", false)?,
        surface_resolution: get_usize_pair("surface_resolution", (0, 0))?,
        surface_axis: get_usize("surface_axis", 1)?,
        source_keys: get_keyed_parts("source_keys")?,
        target_keys: get_keyed_parts("target_keys")?,
        members,
    })
}

/// Build one native animation from a parsed spec, recursing into
/// composition members. The native constructors own all timing math.
#[allow(clippy::too_many_lines)]
fn build_native_animation(
    stage: &mut fmn_mobject::Stage,
    spec: AnimSpec,
) -> PyResult<Box<dyn fmn_anim::Animation>> {
    let need_target = |target: Option<Mob>| {
        target.ok_or_else(|| PyValueError::new_err("this animation requires a target"))
    };
    let need_mob = |mob: Option<Mob>| {
        mob.ok_or_else(|| PyValueError::new_err("this animation requires a mobject"))
    };
    let is_composition = matches!(
        spec.kind.as_str(),
        "animation_group"
            | "broadcast"
            | "cyclic_replace"
            | "swap"
            | "flash"
            | "flashy_fade_in"
            | "lagged_start"
            | "show_creation_then_destruction_around"
            | "show_creation_then_fade_around"
            | "show_creation_then_fade_out"
            | "succession"
            | "transform_matching_parts"
            | "transform_matching_shapes"
            | "transform_matching_strings"
            | "transform_matching_tex"
    );
    let mut animation: Box<dyn fmn_anim::Animation> = match spec.kind.as_str() {
        "python_callback" | "native_callback" => {
            Box::new(PythonAnimationSlot::new(need_mob(spec.mob)?, spec.remover))
        }
        "animation_group" | "lagged_start" => {
            let mut members = Vec::with_capacity(spec.members.len());
            for member in spec.members {
                members.push(build_native_animation(stage, member)?);
            }
            let mut group =
                fmn_anim::AnimationGroup::with_lag_ratio(stage, members, spec.group_lag)
                    .map_err(anim_error)?;
            if spec.kind == "lagged_start" {
                group = group.with_name("LaggedStart");
            }
            Box::new(group)
        }
        "succession" => {
            let mut members = Vec::with_capacity(spec.members.len());
            for member in spec.members {
                members.push(build_native_animation(stage, member)?);
            }
            Box::new(
                fmn_anim::Succession::with_lag_ratio(stage, members, spec.group_lag)
                    .map_err(anim_error)?,
            )
        }
        "flash" => {
            let mut members = Vec::with_capacity(spec.members.len());
            for member in spec.members {
                members.push(build_native_animation(stage, member)?);
            }
            Box::new(fmn_anim::flash(stage, members, spec.group_lag).map_err(anim_error)?)
        }
        "show_creation_then_fade_out" => {
            let mut members = Vec::with_capacity(spec.members.len());
            for member in spec.members {
                members.push(build_native_animation(stage, member)?);
            }
            Box::new(
                fmn_anim::show_creation_then_fade_out(stage, members, spec.group_lag, spec.remover)
                    .map_err(anim_error)?,
            )
        }
        "broadcast"
        | "flashy_fade_in"
        | "show_creation_then_destruction_around"
        | "show_creation_then_fade_around" => {
            let mut members = Vec::with_capacity(spec.members.len());
            for member in spec.members {
                members.push(build_native_animation(stage, member)?);
            }
            let name = match spec.kind.as_str() {
                "broadcast" => "Broadcast",
                "flashy_fade_in" => "FlashyFadeIn",
                "show_creation_then_destruction_around" => "ShowCreationThenDestructionAround",
                "show_creation_then_fade_around" => "ShowCreationThenFadeAround",
                _ => unreachable!(),
            };
            let mut group =
                fmn_anim::AnimationGroup::with_lag_ratio(stage, members, spec.group_lag)
                    .map_err(anim_error)?
                    .with_name(name);
            fmn_anim::Animation::state_mut(&mut group).config.remover = spec.remover;
            Box::new(group)
        }
        "transform_matching_tex" | "transform_matching_strings" => {
            let members = fmn_anim::transform_matching_keys(
                stage,
                need_mob(spec.mob)?,
                need_target(spec.target)?,
                &spec.source_keys,
                &spec.target_keys,
            )
            .map_err(anim_error)?;
            let name = if spec.kind == "transform_matching_strings" {
                "TransformMatchingStrings"
            } else {
                "TransformMatchingTex"
            };
            Box::new(
                fmn_anim::AnimationGroup::new(stage, members)
                    .map_err(anim_error)?
                    .with_name(name),
            )
        }
        "transform" => {
            let mut transform =
                fmn_anim::Transform::new(need_mob(spec.mob)?, need_target(spec.target)?);
            if spec.path_arc != 0.0 {
                transform = transform.with_path_arc(spec.path_arc, spec.path_arc_axis);
            }
            Box::new(transform)
        }
        "replacement_transform" => Box::new(fmn_anim::replacement_transform(
            need_mob(spec.mob)?,
            need_target(spec.target)?,
        )),
        "transform_from_copy" => Box::new(
            fmn_anim::transform_from_copy(stage, need_mob(spec.mob)?, need_target(spec.target)?)
                .map_err(anim_error)?,
        ),
        "fade_in" | "fade_out" => {
            let mob = need_mob(spec.mob)?;
            let mut fade = if spec.kind == "fade_in" {
                fmn_anim::fade_in(stage, mob, spec.shift, spec.scale)
            } else {
                fmn_anim::fade_out(stage, mob, spec.shift, spec.scale)
            }
            .map_err(anim_error)?;
            // Fade is a Transform: its path_arc bends the travel
            // (FadeInFromPoint/FadeOutToPoint arrive here as fades too).
            if spec.path_arc != 0.0 {
                fade = fade.with_path_arc(spec.path_arc, spec.path_arc_axis);
            }
            Box::new(fade)
        }
        "v_fade_in" => Box::new(fmn_anim::v_fade_in(need_mob(spec.mob)?)),
        "v_fade_out" => Box::new(fmn_anim::v_fade_out(need_mob(spec.mob)?)),
        "v_fade_in_then_out" => Box::new(fmn_anim::v_fade_in_then_out(need_mob(spec.mob)?)),
        "show_creation" => Box::new(fmn_anim::show_creation(need_mob(spec.mob)?)),
        "show_surface_creation" => Box::new(fmn_anim::show_surface_creation(
            need_mob(spec.mob)?,
            spec.surface_resolution,
            spec.surface_axis,
        )),
        "uncreate" => Box::new(fmn_anim::uncreate(need_mob(spec.mob)?)),
        "uncreate_surface" => Box::new(fmn_anim::uncreate_surface(
            need_mob(spec.mob)?,
            spec.surface_resolution,
            spec.surface_axis,
        )),
        "maintain_position_relative_to" => Box::new(fmn_anim::MaintainPositionRelativeTo::new(
            // fm-5wq.4.62: update.py:53 — the construction-time offset is
            // captured here, at spec build, exactly the Reference's
            // `self.diff = mobject.get_center() - tracked.get_center()`.
            stage,
            need_mob(spec.mob)?,
            need_target(spec.target)?,
        )),
        "show_increasing_subsets" | "show_submobjects_one_by_one" => {
            // fm-5wq.4.58: Choreo's subset reveal. The constructors seed
            // the Reference default rounding; an explicit `int_func` from
            // the bootstrap overrides it as data.
            let mut subsets = if spec.kind == "show_submobjects_one_by_one" {
                fmn_anim::show_submobjects_one_by_one(stage, need_mob(spec.mob)?)
            } else {
                fmn_anim::show_increasing_subsets(stage, need_mob(spec.mob)?)
            }
            .map_err(anim_error)?;
            match spec.int_round.as_str() {
                "" => {}
                "round" => subsets = subsets.with_int_round(fmn_anim::IntRound::Round),
                "ceil" => subsets = subsets.with_int_round(fmn_anim::IntRound::Ceil),
                other => {
                    return Err(PyValueError::new_err(format!(
                        "int_func {other:?} is not a native rounding rule \
                         (np.round or np.ceil)"
                    )));
                }
            }
            Box::new(subsets)
        }
        "write" => {
            let mut write = fmn_anim::write(stage, need_mob(spec.mob)?);
            if let Some(rgb) = spec.stroke_color {
                #[allow(clippy::cast_possible_truncation)]
                let rgb = [rgb[0] as f32, rgb[1] as f32, rgb[2] as f32];
                write = write.with_stroke_color(Some(rgb));
            }
            Box::new(write)
        }
        "draw_border_then_fill" => {
            let mut border = fmn_anim::DrawBorderThenFill::new(need_mob(spec.mob)?);
            if let Some(width) = spec.stroke_width {
                border = border.with_stroke_width(width);
            }
            if let Some(rgb) = spec.stroke_color {
                #[allow(clippy::cast_possible_truncation)]
                let rgb = [rgb[0] as f32, rgb[1] as f32, rgb[2] as f32];
                border = border.with_stroke_color(Some(rgb));
            }
            Box::new(border)
        }
        "show_partial" => {
            // The two Reference bounds vocabularies; the bootstrap
            // classifies a subclass's `get_bounds` into one of them and
            // refuses anything else before the spec is built.
            let bounds = match spec.bounds_kind.as_str() {
                "creation" => fmn_anim::RevealBounds::Creation,
                "passing_flash" => fmn_anim::RevealBounds::PassingFlash {
                    time_width: spec.time_width,
                },
                other => {
                    return Err(PyValueError::new_err(format!(
                        "ShowPartial bounds rule `{other}` is not a native reveal rule"
                    )));
                }
            };
            Box::new(fmn_anim::ShowPartial::new(need_mob(spec.mob)?, bounds))
        }
        "move_along_path" => Box::new(fmn_anim::MoveAlongPath::new(
            need_mob(spec.mob)?,
            need_target(spec.target)?,
        )),
        "transform_matching_parts" | "transform_matching_shapes" => {
            let members = fmn_anim::transform_matching_parts(
                stage,
                need_mob(spec.mob)?,
                need_target(spec.target)?,
                &spec.pairs,
            )
            .map_err(anim_error)?;
            let name = if spec.kind == "transform_matching_shapes" {
                "TransformMatchingShapes"
            } else {
                "TransformMatchingParts"
            };
            Box::new(
                fmn_anim::AnimationGroup::new(stage, members)
                    .map_err(anim_error)?
                    .with_name(name),
            )
        }
        "cyclic_replace" | "swap" => {
            let name = if spec.kind == "swap" {
                "Swap"
            } else {
                "CyclicReplace"
            };
            let transforms =
                fmn_anim::cyclic_replace(stage, &spec.mobs, spec.path_arc).map_err(anim_error)?;
            let members: Vec<Box<dyn fmn_anim::Animation>> = transforms
                .into_iter()
                .map(|mut transform| {
                    fmn_anim::Animation::state_mut(&mut transform).config.name = name.to_owned();
                    Box::new(transform) as Box<dyn fmn_anim::Animation>
                })
                .collect();
            Box::new(
                fmn_anim::AnimationGroup::new(stage, members)
                    .map_err(anim_error)?
                    .with_name(name),
            )
        }
        "rotate" => {
            let mut rotating = fmn_anim::rotate(need_mob(spec.mob)?, spec.angle)
                .with_axis(spec.axis)
                .with_about_edge(spec.about_edge);
            if let Some(point) = spec.about_point {
                rotating = rotating.with_about_point(point);
            }
            Box::new(rotating)
        }
        "rotating" => {
            // Rotating's Reference defaults (TAU, 5 s, linear) live in the
            // native constructor; both pivots stay None unless given,
            // exactly the Reference's signature.
            let mut rotating = fmn_anim::Rotating::new(need_mob(spec.mob)?)
                .with_angle(spec.angle)
                .with_axis(spec.axis);
            if let Some(point) = spec.about_point {
                rotating = rotating.with_about_point(point);
            }
            if let Some(edge) = spec.about_edge_opt {
                rotating = rotating.with_about_edge(edge);
            }
            Box::new(rotating)
        }
        "grow_from_point"
        | "grow_from_center"
        | "grow_from_edge"
        | "grow_arrow"
        | "spin_in_from_nothing" => {
            #[allow(clippy::cast_possible_truncation)]
            let point_color = spec
                .point_color
                .map(|rgb| [rgb[0] as f32, rgb[1] as f32, rgb[2] as f32]);
            let mut growing =
                fmn_anim::grow_from_point(stage, need_mob(spec.mob)?, spec.point, point_color)
                    .map_err(anim_error)?;
            if spec.path_arc != 0.0 {
                growing = growing.with_path_arc(spec.path_arc, spec.path_arc_axis);
            }
            fmn_anim::Animation::state_mut(&mut growing).config.name = match spec.kind.as_str() {
                "grow_from_center" => "GrowFromCenter",
                "grow_from_edge" => "GrowFromEdge",
                "grow_arrow" => "GrowArrow",
                "spin_in_from_nothing" => "SpinInFromNothing",
                _ => "GrowFromPoint",
            }
            .to_owned();
            Box::new(growing)
        }
        "focus_on" => Box::new(fmn_anim::focus_on(
            need_mob(spec.mob)?,
            need_target(spec.target)?,
            spec.remover,
        )),
        "indicate" => {
            #[allow(clippy::cast_possible_truncation)]
            let color = spec
                .color
                .map(|rgb| [rgb[0] as f32, rgb[1] as f32, rgb[2] as f32]);
            Box::new(
                fmn_anim::indicate(stage, need_mob(spec.mob)?, spec.scale_factor, color)
                    .map_err(anim_error)?,
            )
        }
        "circle_indicate" => Box::new(fmn_anim::circle_indicate(
            need_mob(spec.mob)?,
            need_target(spec.target)?,
            spec.remover,
        )),
        "turn_inside_out" => Box::new(
            fmn_anim::turn_inside_out(stage, need_mob(spec.mob)?, spec.path_arc)
                .map_err(anim_error)?,
        ),
        "wiggle_out_then_in" => {
            let mut wiggle = fmn_anim::WiggleOutThenIn::new(need_mob(spec.mob)?)
                .with_scale_value(spec.scale_value)
                .with_rotation_angle(spec.rotation_angle)
                .with_n_wiggles(spec.n_wiggles);
            if let Some(point) = spec.scale_about_point {
                wiggle = wiggle.with_scale_about_point(point);
            }
            if let Some(point) = spec.rotate_about_point {
                wiggle = wiggle.with_rotate_about_point(point);
            }
            Box::new(wiggle)
        }
        "show_passing_flash" | "show_creation_then_destruction" => {
            let mut flash = fmn_anim::show_passing_flash(need_mob(spec.mob)?, spec.time_width);
            if spec.kind == "show_creation_then_destruction" {
                fmn_anim::Animation::state_mut(&mut flash).config.name =
                    "ShowCreationThenDestruction".to_owned();
            }
            Box::new(flash)
        }
        "v_show_passing_flash" => Box::new(
            fmn_anim::VShowPassingFlash::new(need_mob(spec.mob)?)
                .with_time_width(spec.time_width)
                .with_taper_width(spec.taper_width),
        ),
        "apply_wave" => Box::new(fmn_anim::apply_wave(
            stage,
            need_mob(spec.mob)?,
            spec.direction,
            spec.amplitude,
        )),
        "fade_transform" => {
            // fm-5wq.4.98: stretch/dim_to_match ride the native builders.
            let mut fade =
                fmn_anim::fade_transform(stage, need_mob(spec.mob)?, need_target(spec.target)?)
                    .map_err(anim_error)?;
            if let Some(stretch) = spec.stretch {
                fade = fade.with_stretch(stretch);
            }
            if let Some(dim) = spec.dim_to_match {
                fade = fade.with_dim_to_match(dim);
            }
            Box::new(fade)
        }
        "fade_transform_pieces" => Box::new(
            fmn_anim::fade_transform_pieces(stage, need_mob(spec.mob)?, need_target(spec.target)?)
                .map_err(anim_error)?,
        ),
        "restore" => {
            let mut restore = fmn_anim::restore(stage, need_mob(spec.mob)?).map_err(anim_error)?;
            if spec.path_arc != 0.0 {
                restore = restore.with_path_arc(spec.path_arc, spec.path_arc_axis);
            }
            fmn_anim::Animation::state_mut(&mut restore).config.remover = spec.remover;
            Box::new(restore)
        }
        other => {
            return Err(PyValueError::new_err(format!(
                "animation kind `{other}` is not routed to the native shelf"
            )));
        }
    };
    {
        let config = &mut animation.state_mut().config;
        if let Some(value) = spec.run_time {
            config.run_time = value;
        }
        if let Some(rate) = spec.rate {
            config.rate_func = rate;
        }
        if let Some(value) = spec.lag
            && !is_composition
        {
            config.lag_ratio = value;
        }
        if let Some(span) = spec.time_span {
            config.time_span = Some(span);
        }
        if matches!(
            spec.kind.as_str(),
            "v_fade_in" | "v_fade_out" | "v_fade_in_then_out"
        ) {
            config.remover = spec.remover;
            if let Some(value) = spec.final_alpha_value {
                config.final_alpha_value = value;
            }
        }
        config.suspend_mobject_updating = spec.suspend_mobject_updating;
    }
    Ok(animation)
}

/// A play-surface rate value: a catalog NAME, or a pre-sampled curve (a
/// sequence of at least two floats on the uniform `[0, 1]` grid) — the
/// bootstrap samples pure Python callables into the latter before the
/// segment runs, so no interpreter crossing ever happens mid-segment.
fn rate_func_from_py(value: &Bound<'_, PyAny>) -> PyResult<fmn_anim::RateFunc> {
    if let Ok(name) = value.extract::<String>() {
        return named_rate_func(&name);
    }
    let samples: Vec<f64> = value.extract().map_err(|_| {
        PyTypeError::new_err("rate_func must be a catalog name or a pre-sampled sequence of floats")
    })?;
    if samples.len() < 2 {
        return Err(PyValueError::new_err(
            "a sampled rate curve needs at least two samples",
        ));
    }
    if samples.iter().any(|sample| !sample.is_finite()) {
        return Err(PyValueError::new_err(
            "a sampled rate curve must be finite everywhere",
        ));
    }
    Ok(fmn_anim::RateFunc::Sampled(samples.into()))
}

fn named_rate_func(name: &str) -> PyResult<fmn_anim::RateFunc> {
    let function: fn(f64) -> f64 = match name {
        "linear" => fmn_core::rate::linear,
        "smooth" => fmn_core::rate::smooth,
        "rush_into" => fmn_core::rate::rush_into,
        "rush_from" => fmn_core::rate::rush_from,
        "slow_into" => fmn_core::rate::slow_into,
        "double_smooth" => fmn_core::rate::double_smooth,
        "there_and_back" => fmn_core::rate::there_and_back,
        "lingering" => fmn_core::rate::lingering,
        other => {
            return Err(PyValueError::new_err(format!(
                "rate_func `{other}` is not in the engine's named catalog \
                 (linear, smooth, rush_into, rush_from, slow_into, \
                 double_smooth, there_and_back, lingering); parameterized \
                 and custom callables await the crossing-budget rung"
            )));
        }
    };
    Ok(fmn_anim::RateFunc::Base(function))
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    (1.0 - t) * a + t * b
}

/// Cut T3 (fm-d3gt): the camera-frame interpolation a play segment
/// carries. The camera core is its OWN pyclass cell, independent of the
/// Scene RefCell the segment driver holds — so per-frame interpolation
/// inside the sink never crosses the live engine borrow, and no Python
/// dispatch happens mid-segment (the lerp is pure Rust). Semantics mirror
/// the Reference's frame Transform: componentwise quaternion lerp
/// (normalized on write, scipy's own read-side rule), linear
/// center/shape/fovy.
struct CameraLerp {
    core: Py<PyCameraFrameCore>,
    start: fmn_scene::studio_bridge::CameraFrame,
    end: fmn_scene::studio_bridge::CameraFrame,
    start_time: f64,
    run_time: f64,
    rate: fmn_anim::RateFunc,
}

impl CameraLerp {
    fn apply(&self, py: Python<'_>, raw_alpha: f64) -> PyResult<()> {
        let alpha = self.rate.eval(raw_alpha.clamp(0.0, 1.0));
        let mut core = self.core.bind(py).borrow_mut();
        let (start, end) = (&self.start, &self.end);
        let center = [
            lerp(start.center()[0], end.center()[0], alpha),
            lerp(start.center()[1], end.center()[1], alpha),
            lerp(start.center()[2], end.center()[2], alpha),
        ];
        let shape = [
            lerp(start.shape()[0], end.shape()[0], alpha),
            lerp(start.shape()[1], end.shape()[1], alpha),
        ];
        let fovy = lerp(start.field_of_view(), end.field_of_view(), alpha);
        let orientation = [
            lerp(start.orientation()[0], end.orientation()[0], alpha),
            lerp(start.orientation()[1], end.orientation()[1], alpha),
            lerp(start.orientation()[2], end.orientation()[2], alpha),
            lerp(start.orientation()[3], end.orientation()[3], alpha),
        ];
        core.frame.set_center(center).map_err(camera_error)?;
        core.frame.set_shape(shape).map_err(camera_error)?;
        core.frame.set_field_of_view(fovy).map_err(camera_error)?;
        core.frame
            .set_orientation(orientation)
            .map_err(camera_error)?;
        Ok(())
    }

    fn finish_exact(&self, py: Python<'_>) -> PyResult<()> {
        let mut core = self.core.bind(py).borrow_mut();
        core.frame = self.end.clone();
        Ok(())
    }
}

/// The portal's lifecycle sink: validates and records capture alphas, applies
/// camera interpolation, and forwards immutable packets into an optional
/// production Lumen/Reel generation.
#[derive(Default)]
struct PortalSceneSink {
    alphas: Vec<f64>,
    camera: Option<CameraLerp>,
    camera_error: Option<PyErr>,
    render: Arc<Mutex<Option<PortalRenderSession>>>,
    camera_preapplied: bool,
    camera_finished_before_updaters: bool,
}

impl PortalSceneSink {
    fn apply_camera_at_time(&self, py: Python<'_>, time: f64) -> PyResult<()> {
        let Some(camera) = &self.camera else {
            return Ok(());
        };
        let raw = if camera.run_time > 0.0 {
            (time - camera.start_time) / camera.run_time
        } else {
            1.0
        };
        camera.apply(py, raw)
    }

    fn apply_camera_before_updaters(&mut self, py: Python<'_>, time: f64) -> PyResult<()> {
        self.apply_camera_at_time(py, time)?;
        self.camera_preapplied = true;
        Ok(())
    }

    fn finish_camera_before_updaters(&mut self, py: Python<'_>) -> PyResult<()> {
        if let Some(camera) = &self.camera {
            camera.finish_exact(py)?;
            self.camera_finished_before_updaters = true;
        }
        Ok(())
    }
}

impl fmn_scene::SceneSink for PortalSceneSink {
    fn capture(
        &mut self,
        _reason: fmn_scene::CaptureReason,
        packet: fmn_scene::studio_bridge::FramePacket,
    ) -> Result<(), fmn_scene::IntegrationError> {
        self.alphas.push(packet.alpha());
        if self.camera.is_some() && self.camera_error.is_none() && !self.camera_preapplied {
            // The GIL is held by the pymethod driving this segment;
            // attach re-enters it. Only the camera core cell is borrowed
            // — never the Scene.
            Python::attach(|py| {
                if let Err(error) = self.apply_camera_at_time(py, packet.time().to_f64()) {
                    self.camera_error = Some(error);
                }
            });
        }
        let mut render = self.render.lock().map_err(|_| {
            fmn_scene::IntegrationError::new("portal-render", "render session lock was poisoned")
        })?;
        if let Some(session) = render.as_mut() {
            session.capture(packet)?;
        }
        Ok(())
    }
}

/// A per-frame record-field recorder (test/diagnostic seam): a NATIVE
/// stage updater appends one lane of a mobject's first record at every
/// frame update — inside the six-step update slot, no Python crossing,
/// so it may observe an engine-driven play segment mid-flight.
#[pyclass(unsendable, name = "_FieldProbe")]
struct PyFieldProbe {
    values: Rc<RefCell<Vec<f64>>>,
}

#[pymethods]
impl PyFieldProbe {
    fn values(&self) -> Vec<f64> {
        self.values.borrow().clone()
    }
}

/// Deterministic GIL-release verification probe (fm-zoi §17.4).
///
/// Holds only `Arc` atomics, so it is `Send` and usable from any Python
/// thread. The intended protocol (see tests/bridge.py):
///
/// 1. a Python worker thread spins on [`PyGilProbe::native_started`], then
///    calls [`PyGilProbe::tick`] in a loop — each tick requires the GIL;
/// 2. the main thread calls [`PyGilProbe::run_native`], which flips
///    `started` and runs a deterministic native kernel with the GIL
///    RELEASED (`Python::detach`), returning the tick count observed at
///    kernel end;
/// 3. observed > 0 proves the interpreter made progress during a long
///    native wait. If the GIL were held across the kernel, the worker could
///    never tick after `started` and the probe deterministically returns 0.
///
/// No wall-clock assertions anywhere: termination depends only on the fixed
/// work-unit count, and the pass/fail signal is a counter.
#[pyclass(name = "_GilProbe")]
struct PyGilProbe {
    progress: Arc<AtomicUsize>,
    started: Arc<AtomicBool>,
}

#[pymethods]
impl PyGilProbe {
    #[new]
    fn new() -> Self {
        Self {
            progress: Arc::new(AtomicUsize::new(0)),
            started: Arc::new(AtomicBool::new(false)),
        }
    }

    /// One unit of Python-thread progress. Requires the GIL to execute —
    /// that is the point of the probe.
    fn tick(&self) {
        self.progress.fetch_add(1, Ordering::Relaxed);
    }

    /// Ticks observed so far.
    fn observed(&self) -> usize {
        self.progress.load(Ordering::Acquire)
    }

    /// Whether the native kernel has begun (the worker waits for this).
    fn native_started(&self) -> bool {
        self.started.load(Ordering::Acquire)
    }

    /// Run `work_units` iterations of a deterministic native kernel with the
    /// GIL released; return the number of Python ticks observed at the end.
    /// This is the seam shape every long native wait (compilation,
    /// rasterization, conversion, output) uses: owned `Send` state in,
    /// `py.detach`, owned result out.
    fn run_native(&self, py: Python<'_>, work_units: u64) -> usize {
        let progress = Arc::clone(&self.progress);
        let started = Arc::clone(&self.started);
        py.detach(move || {
            started.store(true, Ordering::Release);
            // SplitMix64-style mixing; black_box keeps the kernel honest
            // (un-elidable) while remaining fully deterministic.
            let mut acc = 0x9E37_79B9_7F4A_7C15_u64;
            for i in 0..work_units {
                acc = acc.wrapping_add(i).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                acc ^= acc >> 31;
                std::hint::black_box(acc);
            }
            progress.load(Ordering::Acquire)
        })
    }
}

// --------------------------------------------------------------------------
// The native-builder seam (fm-d3gt): designated manimlib classes construct
// by calling an fmn-library builder, whose built `fmn_mobject::Mobject`
// family is split across proxy nurseries — the root's own records replace
// the constructing proxy's nursery, and every descendant becomes a fresh
// factory-made shell hung on the Python family list. Native geometry is the
// ONE implementation (D4); the bootstrap never re-derives point math.

thread_local! {
    /// The bundled typesetting handle numbered builders take, parsed once
    /// per interpreter thread (the worker is single-threaded by design).
    static FONT_BOOK: std::cell::OnceCell<fmn_library::FontBook> =
        const { std::cell::OnceCell::new() };
}

thread_local! {
    /// The math-typesetting engine over the default fmd-math pack,
    /// constructed once per interpreter thread like [`FONT_BOOK`].
    static TEX_ENGINE: std::cell::OnceCell<fmn_library::TexEngine> =
        const { std::cell::OnceCell::new() };
}

fn with_tex_engine<T>(
    operation: impl FnOnce(&fmn_library::TexEngine) -> PyResult<T>,
) -> PyResult<T> {
    TEX_ENGINE.with(|cell| {
        if cell.get().is_none() {
            let engine =
                fmn_library::TexEngine::new("fmd-math/pack/default", None).map_err(|error| {
                    PyRuntimeError::new_err(format!("fmd-math engine unavailable: {error}"))
                })?;
            let _ = cell.set(engine);
        }
        operation(cell.get().expect("set above"))
    })
}

/// `t2c=` entries as owned pairs the borrowed builder slices point into.
fn t2c_pairs(t2c: Option<&Bound<'_, PyDict>>) -> PyResult<Vec<(String, fmn_core::color::Srgb)>> {
    let mut pairs = Vec::new();
    if let Some(map) = t2c {
        for (key, value) in map.iter() {
            let key: String = key
                .extract()
                .map_err(|_| PyTypeError::new_err("t2c keys must be strings"))?;
            pairs.push((key, srgb_from_py(&value)?));
        }
    }
    Ok(pairs)
}

fn with_font_book<T>(operation: impl FnOnce(&fmn_library::FontBook) -> PyResult<T>) -> PyResult<T> {
    FONT_BOOK.with(|cell| {
        if cell.get().is_none() {
            let book = fmn_library::FontBook::bundled().map_err(|error| {
                PyRuntimeError::new_err(format!("bundled FontBook unavailable: {error}"))
            })?;
            let _ = cell.set(book);
        }
        operation(cell.get().expect("set above"))
    })
}

fn native_error(error: impl std::fmt::Display) -> PyErr {
    PyValueError::new_err(error.to_string())
}

/// Read one user SVG file under the processor's byte budget. The size is
/// checked against metadata before the read, so a bomb never allocates.
/// Raises CPython's `open` audit event first, as `open()` would: a native
/// read is otherwise invisible to audit hooks, including the certified
/// session's effect recorder (fmn_python.effect_audit).
fn read_svg_source(
    py: Python<'_>,
    file_name: &str,
    limits: &fmn_library::svg::SvgLimits,
) -> PyResult<Vec<u8>> {
    py.import("sys")?
        .call_method1("audit", ("open", file_name, "r", 0))?;
    let metadata = std::fs::metadata(file_name).map_err(|error| {
        PyOSError::new_err(format!("SVGMobject cannot read {file_name:?}: {error}"))
    })?;
    if !metadata.is_file() {
        return Err(PyOSError::new_err(format!(
            "SVGMobject source {file_name:?} is not a regular file"
        )));
    }
    let bytes = usize::try_from(metadata.len()).unwrap_or(usize::MAX);
    if bytes > limits.max_bytes {
        return Err(native_error(fmn_library::svg::SvgError::TooLarge {
            bytes,
            limit: limits.max_bytes,
        }));
    }
    std::fs::read(file_name).map_err(|error| {
        PyOSError::new_err(format!("SVGMobject cannot read {file_name:?}: {error}"))
    })
}

fn graph_sampling_error(error: fmn_library::graphs::GraphError<PyErr>) -> PyErr {
    match error {
        fmn_library::graphs::GraphError::Callback(error) => error,
        other => native_error(other),
    }
}

/// Preserve Scribe's precise Textbox diagnostic while naming the portal's
/// schema-level unmapped-glyph refusal for callers that classify errors.
fn textbox_error(error: fmn_library::TextMobjectError) -> PyErr {
    let diagnostic = error.to_string();
    if matches!(&error, fmn_library::TextMobjectError::Text(_))
        && diagnostic.contains("has no glyph")
    {
        PyValueError::new_err(format!("unmapped glyph: {diagnostic}"))
    } else {
        PyValueError::new_err(diagnostic)
    }
}

fn tex_error(error: impl std::fmt::Display) -> PyErr {
    TexError::new_err(error.to_string())
}

/// Materialize only the authoritative family extent needed by Atlas's
/// extent-driven matcher builders. No path geometry is reconstructed here.
fn matcher_extent_vmobject(extent: Option<([f64; 3], [f64; 3])>) -> fmn_library::VMobject {
    if let Some((min, max)) = extent {
        fmn_library::VMobject::from_points(vec![min, max])
    } else {
        fmn_library::VMobject::new()
    }
}

/// `(min, max)` or `(min, max, step)` — the Reference's RangeSpecifier.
fn range3(value: &Bound<'_, PyAny>) -> PyResult<[f64; 3]> {
    let items: Vec<f64> = value
        .extract()
        .map_err(|_| PyTypeError::new_err("a range specifier must be a sequence of numbers"))?;
    match items.len() {
        2 => Ok([items[0], items[1], 1.0]),
        3 => Ok([items[0], items[1], items[2]]),
        other => Err(PyValueError::new_err(format!(
            "a range specifier needs 2 or 3 entries, got {other}"
        ))),
    }
}

fn srgb_from_py(value: &Bound<'_, PyAny>) -> PyResult<fmn_core::color::Srgb> {
    if let Ok(text) = value.extract::<String>() {
        return fmn_core::color::Srgb::from_hex(&text)
            .map_err(|error| PyValueError::new_err(format!("invalid color {text:?}: {error}")));
    }
    // Reference color_to_rgb also takes colour.Color: anything with get_rgb().
    let rgb: Vec<f64> = match value.getattr("get_rgb") {
        Ok(get_rgb) if get_rgb.is_callable() => get_rgb.call0()?.extract()?,
        _ => value.extract().map_err(|_| {
            PyTypeError::new_err("colors must be hex strings, (r, g, b) sequences or Color objects")
        })?,
    };
    if rgb.len() < 3 {
        return Err(PyValueError::new_err("an rgb color needs three components"));
    }
    Ok(fmn_core::color::Srgb {
        r: rgb[0],
        g: rgb[1],
        b: rgb[2],
    })
}

/// One axis-config entry onto [`fmn_library::AxisConfig`]. Returns false
/// for a key this record does not carry (the caller decides whether that
/// is an error or a class-specific key).
fn apply_axis_config_key(
    config: &mut fmn_library::AxisConfig,
    key: &str,
    value: &Bound<'_, PyAny>,
) -> PyResult<bool> {
    match key {
        "color" => config.color = Some(srgb_from_py(value)?),
        // VMobject.__init__: `stroke_color or color`, so None defers to color.
        "stroke_color" => {
            if !value.is_none() {
                config.stroke_color = Some(srgb_from_py(value)?);
            }
        }
        "stroke_width" => config.stroke_width = Some(value.extract()?),
        "stroke_opacity" => config.stroke_opacity = Some(value.extract()?),
        "unit_size" => config.unit_size = Some(value.extract()?),
        "include_ticks" => config.include_ticks = Some(value.extract()?),
        "tick_size" => config.tick_size = Some(value.extract()?),
        "longer_tick_multiple" => config.longer_tick_multiple = Some(value.extract()?),
        "tick_offset" => config.tick_offset = Some(value.extract()?),
        "big_tick_spacing" => config.big_tick_spacing = Some(value.extract()?),
        "include_numbers" => config.include_numbers = Some(value.extract()?),
        "line_to_number_direction" => {
            config.line_to_number_direction = Some(value.extract::<[f64; 3]>()?);
        }
        "line_to_number_buff" => config.line_to_number_buff = Some(value.extract()?),
        "include_tip" => config.include_tip = Some(value.extract()?),
        "numbers_to_exclude" => config.numbers_to_exclude = Some(value.extract()?),
        "decimal_number_config" => {
            let entries = value
                .cast::<PyDict>()
                .map_err(|_| PyTypeError::new_err("decimal_number_config must be a dict"))?;
            for (inner_key, inner_value) in entries.iter() {
                let inner_key: String = inner_key.extract()?;
                match inner_key.as_str() {
                    "num_decimal_places" => {
                        config.num_decimal_places = Some(inner_value.extract()?);
                    }
                    "font_size" => config.number_font_size = Some(inner_value.extract()?),
                    other => {
                        return Err(PyTypeError::new_err(format!(
                            "unsupported decimal_number_config key `{other}`"
                        )));
                    }
                }
            }
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn axis_config_from(config: Option<&Bound<'_, PyDict>>) -> PyResult<fmn_library::AxisConfig> {
    let mut out = fmn_library::AxisConfig::default();
    if let Some(config) = config {
        for (key, value) in config.iter() {
            let key: String = key
                .extract()
                .map_err(|_| PyTypeError::new_err("axis config keys must be strings"))?;
            if !apply_axis_config_key(&mut out, &key, &value)? {
                return Err(PyTypeError::new_err(format!(
                    "unsupported axis config key `{key}`"
                )));
            }
        }
    }
    Ok(out)
}

fn line_family_style_from(
    style: Option<&Bound<'_, PyDict>>,
) -> PyResult<fmn_library::planes::LineFamilyStyle> {
    let mut out = fmn_library::planes::LineFamilyStyle::default();
    if let Some(style) = style {
        for (key, value) in style.iter() {
            let key: String = key.extract()?;
            match key.as_str() {
                "stroke_color" => out.stroke_color = srgb_from_py(&value)?,
                "stroke_width" => out.stroke_width = value.extract()?,
                "stroke_opacity" => out.stroke_opacity = value.extract()?,
                other => {
                    return Err(PyTypeError::new_err(format!(
                        "unsupported background_line_style key `{other}`"
                    )));
                }
            }
        }
    }
    Ok(out)
}

fn faded_line_style_from(
    style: Option<&Bound<'_, PyDict>>,
) -> PyResult<fmn_library::planes::FadedLineStyle> {
    let mut out = fmn_library::planes::FadedLineStyle::default();
    if let Some(style) = style {
        for (key, value) in style.iter() {
            let key: String = key.extract()?;
            match key.as_str() {
                "stroke_color" => out.stroke_color = Some(srgb_from_py(&value)?),
                "stroke_width" => out.stroke_width = value.extract()?,
                "stroke_opacity" => out.stroke_opacity = value.extract()?,
                other => {
                    return Err(PyTypeError::new_err(format!(
                        "unsupported faded_line_style key `{other}`"
                    )));
                }
            }
        }
    }
    Ok(out)
}

/// The Reference's NumberLine constructor surface onto the native builder:
/// the AxisConfig subset plus the NumberLine-only keys. Unknown keys
/// refuse precisely — never silently dropped.
fn number_line_from_config(
    x_range: [f64; 3],
    config: &Bound<'_, PyDict>,
) -> PyResult<fmn_library::NumberLine> {
    let mut axis_config = fmn_library::AxisConfig::default();
    let mut width: Option<f64> = None;
    let mut big_tick_numbers: Option<Vec<f64>> = None;
    let mut tip_size: Option<f64> = None;
    for (key, value) in config.iter() {
        let key: String = key.extract()?;
        match key.as_str() {
            "width" => width = value.extract()?,
            "big_tick_numbers" => big_tick_numbers = Some(value.extract()?),
            "tip_config" => {
                let entries = value
                    .cast::<PyDict>()
                    .map_err(|_| PyTypeError::new_err("tip_config must be a dict"))?;
                let mut tip_width: Option<f64> = None;
                let mut tip_length: Option<f64> = None;
                for (inner_key, inner_value) in entries.iter() {
                    let inner_key: String = inner_key.extract()?;
                    match inner_key.as_str() {
                        "width" => tip_width = Some(inner_value.extract()?),
                        "length" => tip_length = Some(inner_value.extract()?),
                        other => {
                            return Err(PyTypeError::new_err(format!(
                                "unsupported tip_config key `{other}`"
                            )));
                        }
                    }
                }
                match (tip_width, tip_length) {
                    (None, None) => {}
                    (Some(w), Some(l)) if w == l => tip_size = Some(w),
                    (Some(w), None) | (None, Some(w)) => tip_size = Some(w),
                    (Some(_), Some(_)) => {
                        return Err(PyValueError::new_err(
                            "the native arrow tip is square; tip_config width and \
                             length must agree",
                        ));
                    }
                }
            }
            other => {
                if !apply_axis_config_key(&mut axis_config, other, &value)? {
                    return Err(PyTypeError::new_err(format!(
                        "NumberLine() got an unexpected keyword argument `{other}`"
                    )));
                }
            }
        }
    }
    let mut line = fmn_library::create_axis(x_range, axis_config, width);
    if let Some(values) = big_tick_numbers {
        line = line.big_tick_numbers(values);
    }
    if let Some(size) = tip_size {
        line = line.tip_size(size);
    }
    Ok(line)
}

#[allow(clippy::too_many_arguments)]
fn axes_builder(
    x_range: [f64; 3],
    y_range: [f64; 3],
    axis_config: Option<&Bound<'_, PyDict>>,
    x_axis_config: Option<&Bound<'_, PyDict>>,
    y_axis_config: Option<&Bound<'_, PyDict>>,
    height: Option<f64>,
    width: Option<f64>,
    unit_size: f64,
    number_font_size: Option<f64>,
) -> PyResult<fmn_library::Axes> {
    let mut axis_config = axis_config_from(axis_config)?;
    let mut x_axis_config = axis_config_from(x_axis_config)?;
    let mut y_axis_config = axis_config_from(y_axis_config)?;
    if let Some(font_size) = number_font_size {
        axis_config.number_font_size = Some(font_size);
        x_axis_config.number_font_size = Some(font_size);
        y_axis_config.number_font_size = Some(font_size);
    }
    let mut axes = fmn_library::Axes::new()
        .x_range(x_range)
        .y_range(y_range)
        .axis_config(axis_config)
        .x_axis_config(x_axis_config)
        .y_axis_config(y_axis_config)
        .unit_size(unit_size);
    if let Some(height) = height {
        axes = axes.height(height);
    }
    if let Some(width) = width {
        axes = axes.width(width);
    }
    Ok(axes)
}

#[allow(clippy::too_many_arguments)]
fn complex_plane_builder(
    x_range: [f64; 3],
    y_range: [f64; 3],
    axis_config: Option<&Bound<'_, PyDict>>,
    x_axis_config: Option<&Bound<'_, PyDict>>,
    y_axis_config: Option<&Bound<'_, PyDict>>,
    background_line_style: Option<&Bound<'_, PyDict>>,
    faded_line_style: Option<&Bound<'_, PyDict>>,
    faded_line_ratio: usize,
    height: Option<f64>,
    width: Option<f64>,
    unit_size: f64,
) -> PyResult<fmn_library::ComplexPlane> {
    let mut plane = fmn_library::ComplexPlane::new()
        .x_range(x_range)
        .y_range(y_range)
        .axis_config(axis_config_from(axis_config)?)
        .x_axis_config(axis_config_from(x_axis_config)?)
        .y_axis_config(axis_config_from(y_axis_config)?)
        .background_line_style(line_family_style_from(background_line_style)?)
        .faded_line_style(faded_line_style_from(faded_line_style)?)
        .faded_line_ratio(faded_line_ratio)
        .unit_size(unit_size);
    if let Some(height) = height {
        plane = plane.height(height);
    }
    if let Some(width) = width {
        plane = plane.width(width);
    }
    Ok(plane)
}

/// One factory shell per family node, recursively: the node's own records
/// become the shell's single-root nursery; descendants are returned as
/// nested `(shell, children)` specs for the bootstrap to hang on the
/// Python family lists.
fn native_shell_specs<'py>(
    py: Python<'py>,
    factory: &Bound<'py, PyAny>,
    nodes: Vec<Mobject>,
) -> PyResult<Bound<'py, PyList>> {
    let out = PyList::empty(py);
    for mut node in nodes {
        let defaults_row = node.defaults_row.take();
        let children = std::mem::take(&mut node.submobjects);
        let shell = factory.call0()?;
        {
            let bridge = shell.cast::<BridgeMobject>().map_err(|_| {
                PyTypeError::new_err("the native shell factory must return a Mobject")
            })?;
            let mut cell = bridge.borrow_mut();
            cell.nursery = Some(Nursery::new(node));
            cell.initialized = true;
        }
        if let Some(row) = defaults_row {
            // A point-free native node's constructor style is the
            // Reference's `_data_defaults` row; the portal color getters
            // answer from it until real records arrive.
            shell.call_method1("_seed_data_defaults", (row,))?;
        }
        let child_specs = native_shell_specs(py, factory, children)?;
        out.append((shell, child_specs))?;
    }
    Ok(out)
}

/// Flatten pointful descendants onto a portal shell's own VMobject records.
///
/// Native Scribe text intentionally remains a recordless root with one child
/// per glyph. Some compatibility proxies, such as `Textbox.text`, must keep a
/// single Python identity across `become` rebuilds, so their portal candidate
/// owns a flattened copy of the glyph outlines instead. Separate glyph paths
/// are joined with the shared-anchor model's null-curve marker; per-record
/// style lanes are copied unchanged. Callers that still need glyph-level
/// family traversal may retain those children alongside the flattened root.
fn hoist_descendant_records(root: &mut Mobject, retain_children: bool) -> PyResult<()> {
    fn collect_pointful<'a>(node: &'a Mobject, out: &mut Vec<&'a RecordBuffer>) {
        if !node.buffer.is_empty() {
            out.push(&node.buffer);
        }
        for child in &node.submobjects {
            collect_pointful(child, out);
        }
    }

    let schema = root.buffer.schema().clone();
    let hoisted = {
        let mut sources = Vec::new();
        for child in &root.submobjects {
            collect_pointful(child, &mut sources);
        }
        for source in &sources {
            // ubs:ignore — compares RecordBuffer layouts, not authentication material.
            if source.schema() != &schema {
                return Err(PyRuntimeError::new_err(
                    "native text descendant schema cannot be hoisted onto its portal shell",
                ));
            }
        }
        let source_records = sources.iter().try_fold(0usize, |total, source| {
            total.checked_add(source.len()).ok_or_else(|| {
                PyOverflowError::new_err("native text descendant record count overflows usize")
            })
        })?;
        let separators = sources
            .len()
            .saturating_sub(1)
            .checked_mul(2)
            .ok_or_else(|| {
                PyOverflowError::new_err("native text separator record count overflows usize")
            })?;
        let total = source_records.checked_add(separators).ok_or_else(|| {
            PyOverflowError::new_err("native text flattened record count overflows usize")
        })?;
        let mut buffer = RecordBuffer::new(schema.clone(), total).map_err(record_error_to_py)?;
        let mut output = 0;
        for (index, source) in sources.iter().enumerate() {
            if index > 0 {
                let previous = sources[index - 1];
                for field in schema.fields() {
                    let previous_value = previous
                        .read(previous.len() - 1, &field.name)
                        .expect("pointful source and iterated field exist");
                    let first_value = source
                        .read(0, &field.name)
                        .expect("pointful source and iterated field exist");
                    assert!(buffer.write(output, &field.name, &previous_value));
                    assert!(buffer.write(output + 1, &field.name, &first_value));
                }
                output += 2;
            }
            for field in schema.fields() {
                let values = source
                    .read_column(&field.name)
                    .expect("iterated source field exists");
                assert!(buffer.write_range(&field.name, output, &values));
            }
            output += source.len();
        }
        buffer
    };
    root.buffer = hoisted;
    if !retain_children {
        root.submobjects.clear();
    }
    Ok(())
}

/// Install a native brace and retain the analytic tip's point index. Point
/// identity, rather than a frozen coordinate, makes `get_tip()` live after
/// ordinary Mobject transforms.
fn install_brace_tree<'py>(
    slf: &Bound<'py, BridgeMobject>,
    factory: &Bound<'py, PyAny>,
    brace: fmn_library::Brace,
) -> PyResult<(Bound<'py, PyList>, usize)> {
    let tip = brace.tip();
    let built = brace.build();
    let tip_index = built
        .points()
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| {
            let distance = |point: &&[f64; 3]| {
                let dx = point[0] - tip[0];
                let dy = point[1] - tip[1];
                let dz = point[2] - tip[2];
                dx * dx + dy * dy + dz * dz
            };
            distance(left).total_cmp(&distance(right))
        })
        .map_or(0, |(index, _)| index);
    Ok((install_native_tree(slf, factory, built)?, tip_index))
}

/// Install a built native family on a constructing proxy: the root's own
/// records replace `slf`'s nursery; descendants become factory shells.
fn install_native_tree<'py>(
    slf: &Bound<'py, BridgeMobject>,
    factory: &Bound<'py, PyAny>,
    tree: impl Into<Mobject>,
) -> PyResult<Bound<'py, PyList>> {
    let mut tree = tree.into();
    let children = std::mem::take(&mut tree.submobjects);
    {
        let mut cell = slf.borrow_mut();
        if cell.engine.is_some() {
            return Err(PyRuntimeError::new_err(
                "a native builder may only construct a detached mobject",
            ));
        }
        cell.nursery = Some(Nursery::new(tree));
        cell.initialized = true;
    }
    native_shell_specs(slf.py(), factory, children)
}

/// The engine-backed camera-frame state (fm-d3gt): a thin proxy over
/// Lumen's [`fmn_scene::studio_bridge::CameraFrame`], the ONE implementation of the
/// Reference's euler/orientation/shape/fov semantics (fm-0gy).
///
/// The bootstrap's `CameraFrame(Mobject)` owns one of these as its
/// authoritative state; every camera method delegates here, so orientation,
/// center, shape, and field of view round-trip exactly (D5, state-real).
/// This value is also the renderer-binding seam: final native PNG capture
/// hands the same `fmn_scene::studio_bridge::CameraFrame` to Lumen's `Camera`
/// unchanged.
#[pyclass(unsendable, name = "_CameraFrameCore")]
struct PyCameraFrameCore {
    frame: fmn_scene::studio_bridge::CameraFrame,
}

/// Lumen's capture-camera configuration behind the public Python `Camera`.
///
/// The portal keeps Python's mutable `CameraFrame` object as the live scene
/// identity, while this value makes construction and capture configuration go
/// through Lumen's validated builder instead of a schema-only Python shell.
#[pyclass(unsendable, name = "_CameraCore")]
struct PyCameraCore {
    camera: Camera,
}

fn camera_error(error: fmn_scene::studio_bridge::CameraError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

#[pymethods]
impl PyCameraFrameCore {
    #[new]
    fn py_new(
        frame_shape: [f64; 2],
        center_point: [f64; 3],
        fovy: f64,
        euler_axes: &str,
    ) -> PyResult<Self> {
        Ok(Self {
            frame: fmn_scene::studio_bridge::CameraFrame::new(
                frame_shape,
                center_point,
                fovy,
                euler_axes,
            )
            .map_err(camera_error)?,
        })
    }

    fn __copy__(&self) -> Self {
        Self {
            frame: self.frame.clone(),
        }
    }

    fn __deepcopy__(&self, _memo: &Bound<'_, PyAny>) -> Self {
        Self {
            frame: self.frame.clone(),
        }
    }

    fn center(&self) -> [f64; 3] {
        self.frame.center()
    }

    fn set_center(&mut self, center: [f64; 3]) -> PyResult<()> {
        self.frame.set_center(center).map_err(camera_error)?;
        Ok(())
    }

    fn shape(&self) -> (f64, f64) {
        let [width, height] = self.frame.shape();
        (width, height)
    }

    fn set_shape(&mut self, shape: [f64; 2]) -> PyResult<()> {
        self.frame.set_shape(shape).map_err(camera_error)?;
        Ok(())
    }

    fn aspect_ratio(&self) -> f64 {
        self.frame.aspect_ratio()
    }

    fn scale(&self) -> f64 {
        self.frame.scale()
    }

    fn orientation(&self) -> [f64; 4] {
        self.frame.orientation()
    }

    fn set_orientation(&mut self, orientation: [f64; 4]) -> PyResult<()> {
        self.frame
            .set_orientation(orientation)
            .map_err(camera_error)?;
        Ok(())
    }

    fn make_orientation_default(&mut self) {
        self.frame.make_orientation_default();
    }

    // Reference-verbatim pymethod name; the Rust naming lint does not apply.
    #[allow(clippy::wrong_self_convention)]
    fn to_default_state(&mut self) {
        self.frame.to_default_state();
    }

    fn euler_axes(&self) -> String {
        self.frame.euler_axes().to_owned()
    }

    fn set_euler_axes(&mut self, seq: &str) -> PyResult<()> {
        self.frame.set_euler_axes(seq).map_err(camera_error)?;
        Ok(())
    }

    fn euler_angles(&self) -> [f64; 3] {
        self.frame.euler_angles()
    }

    fn set_euler_angles(
        &mut self,
        theta: Option<f64>,
        phi: Option<f64>,
        gamma: Option<f64>,
    ) -> PyResult<()> {
        self.frame
            .set_euler_angles(theta, phi, gamma)
            .map_err(camera_error)?;
        Ok(())
    }

    fn increment_euler_angles(&mut self, dtheta: f64, dphi: f64, dgamma: f64) -> PyResult<()> {
        self.frame
            .increment_euler_angles(dtheta, dphi, dgamma)
            .map_err(camera_error)?;
        Ok(())
    }

    fn rotate(&mut self, angle: f64, axis: [f64; 3]) -> PyResult<()> {
        self.frame.rotate(angle, axis).map_err(camera_error)?;
        Ok(())
    }

    fn field_of_view(&self) -> f64 {
        self.frame.field_of_view()
    }

    fn set_field_of_view(&mut self, fovy: f64) -> PyResult<()> {
        self.frame.set_field_of_view(fovy).map_err(camera_error)?;
        Ok(())
    }

    fn focal_distance(&self) -> f64 {
        self.frame.focal_distance()
    }

    fn set_focal_distance(&mut self, focal_distance: f64) -> PyResult<()> {
        self.frame
            .set_focal_distance(focal_distance)
            .map_err(camera_error)?;
        Ok(())
    }

    fn view_matrix(&self) -> [[f64; 4]; 4] {
        self.frame.view_matrix()
    }

    // The `to_*`/`from_*` names mirror the Reference's Python API verbatim;
    // Rust's self-convention lint does not apply to a pymethod surface.
    #[allow(clippy::wrong_self_convention)]
    fn to_fixed_frame_point(&self, point: [f64; 3], relative: bool) -> [f64; 3] {
        self.frame.to_fixed_frame_point(point, relative)
    }

    #[allow(clippy::wrong_self_convention)]
    fn from_fixed_frame_point(&self, point: [f64; 3], relative: bool) -> [f64; 3] {
        self.frame.from_fixed_frame_point(point, relative)
    }

    fn implied_camera_location(&self) -> [f64; 3] {
        self.frame.implied_camera_location()
    }
}

#[pymethods]
impl PyCameraCore {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn py_new(
        frame: PyRef<'_, PyCameraFrameCore>,
        resolution: [u32; 2],
        fps: u32,
        background_rgb: [f64; 3],
        background_opacity: f64,
        max_allowable_norm: f64,
        light_source_position: [f64; 3],
        samples: u8,
    ) -> PyResult<Self> {
        let background = fmn_core::color::Srgb {
            r: background_rgb[0],
            g: background_rgb[1],
            b: background_rgb[2],
        }
        .to_linear(background_opacity);
        let camera = Camera::new(CameraConfig {
            resolution: (resolution[0], resolution[1]),
            fps,
            background,
            max_allowable_norm,
            samples,
            light_source_position,
            frame: frame.frame.clone(),
        })
        .map_err(camera_error)?;
        Ok(Self { camera })
    }

    fn pixel_shape(&self) -> (u32, u32) {
        self.camera.pixel_shape()
    }

    fn set_pixel_shape(&mut self, width: u32, height: u32) -> PyResult<()> {
        self.camera
            .set_pixel_shape(width, height)
            .map_err(camera_error)?;
        Ok(())
    }

    fn fps(&self) -> u32 {
        self.camera.fps()
    }

    fn max_allowable_norm(&self) -> f64 {
        self.camera.max_allowable_norm()
    }

    fn samples(&self) -> u8 {
        self.camera.samples()
    }

    fn light_source_position(&self) -> [f64; 3] {
        self.camera.light_source_position()
    }

    fn frame_shape(&self) -> (f64, f64) {
        let shape = self.camera.frame().shape();
        (shape[0], shape[1])
    }
}

fn execute_bootstrap(py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    // Direct ExtensionFileLoader users do not install the module in
    // sys.modules until after create_module returns, but our bootstrap must
    // assemble child packages during create_module. Pass the actual module
    // explicitly, then remove the temporary self-reference.
    module.add("_FMN_MODULE", module)?;
    module.add("_API_SCHEMA_TSV", include_str!("../../../API_SCHEMA.tsv"))?;
    module.add("_API_OVERLAY_TSV", include_str!("../../../API_OVERLAY.tsv"))?;
    let source = CString::new(include_str!("../python/manimlib_bootstrap.py"))
        .expect("embedded bootstrap contains no NUL");
    let globals = module.dict();
    let result = py.run(source.as_c_str(), Some(&globals), Some(&globals));
    module.delattr("_FMN_MODULE")?;
    result?;
    let semantics = CString::new(include_str!("../python/manimlib/_animation_semantics.py"))
        .expect("embedded animation semantics contain no NUL");
    install_animation_semantics(py, module, semantics.as_c_str())
}

fn install_animation_semantics(
    py: Python<'_>,
    module: &Bound<'_, PyModule>,
    source: &CStr,
) -> PyResult<()> {
    // Use a private globals dictionary, so a direct ExtensionFileLoader and
    // a wheel import execute identical code without importing the surrounding
    // Python package or publishing another module in sys.modules.
    let globals = PyDict::new(py);
    globals.set_item("__name__", "manimlib._animation_semantics")?;
    py.run(source, Some(&globals), Some(&globals))?;
    let install = globals.get_item("install")?.ok_or_else(|| {
        pyo3::exceptions::PyImportError::new_err("missing production animation semantics installer")
    })?;
    install.call1((module,))?;
    Ok(())
}

/// The shipped `Mobject.get_family()` walk: the Reference's path-wise
/// preorder, where a shared descendant appears once per path that reaches it
/// and a genuine cycle raises. Each step is the bootstrap loop's own
/// operation in its order (`isinstance(member, _BridgeMobject)`, the cycle
/// check, `list(member.submobjects)`), so authored `submobjects` properties
/// and list subclasses run exactly as before; only the interpreter overhead
/// goes (fm-5wq.31: `has_updaters` walks large animation copies every frame).
/// No Stage is borrowed, so an authored getter may reenter freely.
#[pyfunction]
fn _path_family<'py>(root: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyList>> {
    let py = root.py();
    let bridge = py.get_type::<BridgeMobject>();
    let list = py.get_type::<PyList>();
    let submobjects = pyo3::intern!(py, "submobjects");
    let mut family = Vec::new();
    let mut visiting = HashSet::new();
    let mut stack = vec![(true, root.clone())];
    while let Some((entering, mobject)) = stack.pop() {
        // Every marked object is held by `family`, so its address is its id.
        let marker = mobject.as_ptr() as usize;
        if !entering {
            visiting.remove(&marker);
            continue;
        }
        if !mobject.is_instance(&bridge)? {
            return Err(PyTypeError::new_err(
                "submobjects must be Mobject instances",
            ));
        }
        if visiting.contains(&marker) {
            return Err(FamilyCycleError::new_err(
                "submobjects would create a family cycle",
            ));
        }
        family.push(mobject.clone());
        let children = list
            .call1((mobject.getattr(submobjects)?,))?
            .cast_into::<PyList>()?;
        if !children.is_empty() {
            visiting.insert(marker);
            stack.push((false, mobject));
            stack.extend(children.iter().rev().map(|child| (true, child)));
        }
    }
    PyList::new(py, family)
}

/// Identity keys (object addresses, `Mob` handles) for the binding's
/// per-call maps and the proxy registry.
type ByAddress = fmn_mobject::IdBuildHasher;

/// The per-member loop of `Scene._fmn_requires_public_scene_update`
/// (updater_dispatch.py): whether any member reachable from `pending` needs
/// the public update path. `helpers` holds that scan's Python decisions,
/// `(Mobject, hook_names, child_protocols, object.__getattribute__,
/// plain_instance_dict, member_changed, submobjects_kept, container_changed,
/// generic_getattr)`. Each is called at the same point, and cached per class
/// for this scan, exactly as the Python loop did it. Own dictionaries are
/// read through `object.__getattribute__`, or through plain `getattr` for a
/// class whose lookup `generic_getattr` shows is object's own (the same C
/// lookup), so no authored getter runs. Only the walk and its bookkeeping
/// moved (fm-5wq.31: about 3 us per member per frame).
#[pyfunction]
fn _public_update_scan<'py>(
    pending: Vec<Bound<'py, PyAny>>,
    helpers: &Bound<'py, PyTuple>,
) -> PyResult<bool> {
    type Helpers<'py> = (
        Bound<'py, PyAny>,
        Bound<'py, PyTuple>,
        Bound<'py, PyDict>,
        Bound<'py, PyAny>,
        Bound<'py, PyAny>,
        Bound<'py, PyAny>,
        Bound<'py, PyAny>,
        Bound<'py, PyAny>,
        Bound<'py, PyAny>,
    );
    let py = helpers.py();
    let (
        mobject,
        hook_names,
        child_protocols,
        instance_dict,
        plain,
        member_changed,
        submobjects_kept,
        container_changed,
        generic_getattr,
    ): Helpers<'py> = helpers.extract()?;
    let submobjects = pyo3::intern!(py, "submobjects");
    let dict_name = pyo3::intern!(py, "__dict__");
    // `try: own = _instance_dict(obj, "__dict__") except AttributeError: None`
    let own_dict =
        |obj: &Bound<'py, PyAny>, generic: bool| -> PyResult<Option<Bound<'py, PyDict>>> {
            let own = if generic {
                obj.getattr(dict_name)
            } else {
                instance_dict.call1((obj, dict_name))
            };
            match own {
                Ok(own) if own.is_none() => Ok(None),
                Ok(own) => Ok(Some(own.cast_into::<PyDict>()?)),
                Err(error) if error.is_instance_of::<pyo3::exceptions::PyAttributeError>(py) => {
                    Ok(None)
                }
                Err(error) => Err(error),
            }
        };
    // `not dict.keys(own).isdisjoint(names)` iterates `names` and asks
    // PyDict_Contains of each, stopping at the first hit. The names are
    // read once per scan; a methods table is iterated in place (PyDict_Next
    // is its iteration order), so no per-member iterator is allocated.
    let hook_names: Vec<Bound<'py, PyAny>> = hook_names.iter().collect();
    let holds_any = |own: &Bound<'py, PyDict>, names: &[Bound<'py, PyAny>]| -> PyResult<bool> {
        for name in names {
            if own.contains(name)? {
                return Ok(true);
            }
        }
        Ok(false)
    };
    let holds_any_key = |own: &Bound<'py, PyDict>, table: &Bound<'py, PyDict>| -> PyResult<bool> {
        for (name, _) in table.iter() {
            if own.contains(name)? {
                return Ok(true);
            }
        }
        Ok(false)
    };
    // Per-class answers for this scan, keyed by type; `classes` keeps each
    // keyed type alive, and `visited` each member, so no address is reused.
    let mut classes: Vec<Bound<'py, pyo3::types::PyType>> = Vec::new();
    let mut cached = |cache: &mut HashMap<usize, bool, ByAddress>,
                      cls: &Bound<'py, pyo3::types::PyType>,
                      decide: &dyn Fn() -> PyResult<bool>|
     -> PyResult<bool> {
        let key = cls.as_ptr() as usize;
        if let Some(&answer) = cache.get(&key) {
            return Ok(answer);
        }
        let answer = decide()?;
        cache.insert(key, answer);
        classes.push(cls.clone());
        Ok(answer)
    };
    let mut visible = HashMap::default();
    let mut generic = HashMap::default();
    let mut class_changed = HashMap::default();
    let mut kept = HashMap::default();
    let mut container = HashMap::default();
    let mut pending = pending;
    let mut seen: HashSet<usize, ByAddress> = HashSet::default();
    let mut visited = Vec::new();
    while let Some(member) = pending.pop() {
        if !seen.insert(member.as_ptr() as usize) {
            continue;
        }
        if !member.is_instance(&mobject)? {
            return Ok(true);
        }
        let cls = member.get_type();
        let own = if cached(&mut visible, &cls, &|| plain.call1((&cls,))?.is_truthy())? {
            let generic = cached(&mut generic, &cls, &|| {
                generic_getattr.call1((&cls,))?.is_truthy()
            })?;
            own_dict(&member, generic)?
        } else {
            None
        };
        let changed = match own {
            Some(own) if holds_any(&own, &hook_names)? => {
                member_changed.call1((&member,))?.is_truthy()?
            }
            _ => cached(&mut class_changed, &cls, &|| {
                member_changed.call1((&member,))?.is_truthy()
            })?,
        };
        if changed
            || !cached(&mut kept, &cls, &|| {
                submobjects_kept.call1((&cls,))?.is_truthy()
            })?
        {
            return Ok(true);
        }
        let children = member.getattr(submobjects)?;
        let kind = children.get_type();
        let Some(methods) = child_protocols.get_item(&kind)? else {
            return Ok(true);
        };
        let methods = methods.cast_into::<PyDict>()?;
        let own = if cached(&mut visible, &kind, &|| plain.call1((&kind,))?.is_truthy())? {
            let generic = cached(&mut generic, &kind, &|| {
                generic_getattr.call1((&kind,))?.is_truthy()
            })?;
            own_dict(&children, generic)?
        } else {
            None
        };
        let changed = match own {
            Some(own) if holds_any_key(&own, &methods)? => container_changed
                .call1((&children, &methods))?
                .is_truthy()?,
            _ => cached(&mut container, &kind, &|| {
                container_changed.call1((&children, &methods))?.is_truthy()
            })?,
        };
        if changed {
            return Ok(true);
        }
        for child in children.try_iter()? {
            pending.push(child?);
        }
        visited.push(member);
    }
    Ok(false)
}

/// `any(member.updaters for member in family)`, the shipped
/// `Mobject.has_updaters` scan, in order and short-circuiting, so an authored
/// `updaters` property or `__bool__` runs exactly as before (fm-5wq.31).
#[pyfunction]
fn _any_updaters(family: &Bound<'_, PyAny>) -> PyResult<bool> {
    let updaters = pyo3::intern!(family.py(), "updaters");
    for member in family.try_iter()? {
        if member?.getattr(updaters)?.is_truthy()? {
            return Ok(true);
        }
    }
    Ok(false)
}

#[pyfunction]
fn _composition_intervals(run_times: Vec<f64>, lag_ratio: f64) -> Vec<(f64, f64)> {
    fmn_anim::composition::build_timings(&run_times, lag_ratio)
        .into_iter()
        .map(|interval| (interval.start, interval.end))
        .collect()
}

/// `manimlib.config` directories from the one typed native resolution: the
/// bundled defaults, then an optional `custom_config.yml` document (the
/// Reference's cwd layer). The bootstrap derives the per-subdir paths with
/// Reference `update_directory_config` semantics.
#[pyfunction]
#[pyo3(signature = (custom_config = None))]
fn _resolved_directories<'py>(
    py: Python<'py>,
    custom_config: Option<&str>,
) -> PyResult<Bound<'py, PyDict>> {
    let layers: Vec<fmn_config::config::Layer<'_>> = custom_config
        .map(|text| fmn_config::config::Layer {
            name: "custom_config.yml",
            text,
        })
        .into_iter()
        .collect();
    let resolved = fmn_config::Config::resolve(&layers, None)
        .map_err(|error| PyValueError::new_err(error.to_string()))?;
    let directories = &resolved.config.directories;
    let out = PyDict::new(py);
    out.set_item("mirror_module_path", directories.mirror_module_path)?;
    out.set_item("base", &directories.base)?;
    let subdirs = PyDict::new(py);
    for (key, value) in &directories.subdirs {
        subdirs.set_item(key, value)?;
    }
    out.set_item("subdirs", subdirs)?;
    out.set_item("cache", &directories.cache)?;
    out.set_item(
        "removed_mirror_prefix",
        directories.removed_mirror_prefix.as_deref(),
    )?;
    Ok(out)
}

/// The Reference's `manim_config.camera` layer from the same typed native
/// resolution standalone `fmn` reads: the bundled defaults (1920x1080, 30 fps,
/// `#333333`), then an optional `custom_config.yml`. The portal's `Scene`
/// merges it first, ahead of `default_camera_config` and `camera_config`,
/// as the Reference's `Scene.__init__` does.
#[pyfunction]
#[pyo3(signature = (custom_config = None))]
fn _resolved_camera<'py>(
    py: Python<'py>,
    custom_config: Option<&str>,
) -> PyResult<Bound<'py, PyDict>> {
    let layers: Vec<fmn_config::config::Layer<'_>> = custom_config
        .map(|text| fmn_config::config::Layer {
            name: "custom_config.yml",
            text,
        })
        .into_iter()
        .collect();
    let resolved = fmn_config::Config::resolve(&layers, None)
        .map_err(|error| PyValueError::new_err(error.to_string()))?;
    let camera = &resolved.config.camera;
    let out = PyDict::new(py);
    out.set_item("resolution", camera.resolution)?;
    out.set_item("fps", camera.fps)?;
    out.set_item("background_color", &camera.background_color)?;
    out.set_item("background_opacity", camera.background_opacity)?;
    Ok(out)
}

// CPython's `subtype_traverse` skips visiting an instance's heap type when
// the nearest non-Python base is itself a heap type with a traverse, trusting
// that base to visit `Py_TYPE(self)`: since CPython 3.9, heap types visit
// their type. PyO3 0.29's synthesized traverse for the two subclassable,
// `dict` pyclasses never does, so a cycle through a Python subclass's class
// object is invisible to the collector and leaks. The case that surfaced it
// (fm-2mvb) is a scene-local `class Demo(Scene)` whose `construct` closes over
// mobjects bound to that scene: instance -> class -> construct -> closure ->
// mobject -> `_scene` -> instance. These wrappers visit the type first and
// then run PyO3's own traverse. Neither class extends another pyclass, so
// PyO3's super-traverse still finds nothing to chain to.
static BRIDGE_TRAVERSE: std::sync::OnceLock<ffi::traverseproc> = std::sync::OnceLock::new();
static SCENE_TRAVERSE: std::sync::OnceLock<ffi::traverseproc> = std::sync::OnceLock::new();

/// # Safety
/// Called only by the collector through a wrapped `tp_traverse` slot, with a
/// live object of the wrapped class and the collector's own visit callback.
unsafe fn traverse_heap_type_then(
    object: *mut ffi::PyObject,
    visit: ffi::visitproc,
    arg: *mut std::ffi::c_void,
    original: Option<&ffi::traverseproc>,
) -> std::ffi::c_int {
    // SAFETY: the collector passes a live object, whose type pointer is valid
    // and cannot change while traversal runs.
    let ty = unsafe { ffi::Py_TYPE(object) };
    // SAFETY: `ty` is a valid type object (above).
    if unsafe { ffi::PyType_HasFeature(ty, ffi::Py_TPFLAGS_HEAPTYPE) } != 0 {
        // SAFETY: the instance holds a strong reference to its heap type
        // (PyO3 releases it in tp_dealloc), which is exactly what we report.
        let visited = unsafe { visit(ty.cast(), arg) };
        if visited != 0 {
            return visited;
        }
    }
    match original {
        // SAFETY: PyO3's own traverse for this class, with the collector's
        // arguments.
        Some(traverse) => unsafe { traverse(object, visit, arg) },
        None => 0,
    }
}

unsafe extern "C" fn bridge_traverse(
    object: *mut ffi::PyObject,
    visit: ffi::visitproc,
    arg: *mut std::ffi::c_void,
) -> std::ffi::c_int {
    // SAFETY: installed only as `_BridgeMobject`'s tp_traverse.
    unsafe { traverse_heap_type_then(object, visit, arg, BRIDGE_TRAVERSE.get()) }
}

unsafe extern "C" fn scene_traverse(
    object: *mut ffi::PyObject,
    visit: ffi::visitproc,
    arg: *mut std::ffi::c_void,
) -> std::ffi::c_int {
    // SAFETY: installed only as `_SceneCore`'s tp_traverse.
    unsafe { traverse_heap_type_then(object, visit, arg, SCENE_TRAVERSE.get()) }
}

fn visit_heap_type_in_traverse(
    ty: &Bound<'_, pyo3::types::PyType>,
    original_slot: &std::sync::OnceLock<ffi::traverseproc>,
    wrapper: ffi::traverseproc,
) -> PyResult<()> {
    let raw = ty.as_type_ptr();
    // SAFETY: ADR-0015 keeps abi3 off, so pyo3-ffi's PyTypeObject layout is
    // the interpreter ABI (method_cache reads tp_version_tag the same way).
    // The type is fully initialized and the GIL is held.
    let current = unsafe { (*raw).tp_traverse };
    if current.map(|traverse| traverse as usize) == Some(wrapper as usize) {
        // The module can initialize again over the same process-wide type.
        return Ok(());
    }
    let original = current.ok_or_else(|| {
        PyRuntimeError::new_err("a subclassable dict pyclass has no tp_traverse slot")
    })?;
    let _ = original_slot.set(original);
    // SAFETY: as above. Replacing a slot of our own heap type and announcing
    // it with PyType_Modified is the documented way to change a type slot.
    unsafe {
        (*raw).tp_traverse = Some(wrapper);
        ffi::PyType_Modified(raw);
    }
    Ok(())
}

fn populate_manimlib(py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<BridgeMobject>()?;
    module.add_class::<PyScene>()?;
    visit_heap_type_in_traverse(
        &py.get_type::<BridgeMobject>(),
        &BRIDGE_TRAVERSE,
        bridge_traverse,
    )?;
    visit_heap_type_in_traverse(&py.get_type::<PyScene>(), &SCENE_TRAVERSE, scene_traverse)?;
    module.add_class::<portal_studio::Recording>()?;
    module.add_class::<portal_studio::Host>()?;
    module.add_class::<PyRecordView>()?;
    module.add_class::<PyGilProbe>()?;
    module.add_class::<PyCameraFrameCore>()?;
    module.add_class::<PyCameraCore>()?;
    module.add_class::<portal_readback::CameraCapture>()?;
    module.add_class::<PyFieldProbe>()?;
    module.add_class::<ladder::PyBatchedUpdater>()?;
    module.add_class::<ladder::PyArrayUpdater>()?;
    module.add_class::<ladder::PyNativeUpdater>()?;
    module.add_function(wrap_pyfunction!(
        crossing::_crossing_stats_snapshot,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(crossing::_crossing_stats_reset, module)?)?;
    module.add_function(wrap_pyfunction!(method_cache::_method_cache_stats, module)?)?;
    module.add_function(wrap_pyfunction!(method_cache::_method_cache_reset, module)?)?;
    module.add_function(wrap_pyfunction!(method_cache::_type_version_tags, module)?)?;
    module.add_function(wrap_pyfunction!(report::_crossing_report, module)?)?;
    module.add_function(wrap_pyfunction!(_path_family, module)?)?;
    module.add_function(wrap_pyfunction!(_any_updaters, module)?)?;
    module.add_function(wrap_pyfunction!(_public_update_scan, module)?)?;
    module.add_function(wrap_pyfunction!(_composition_intervals, module)?)?;
    module.add_function(wrap_pyfunction!(_resolved_directories, module)?)?;
    module.add_function(wrap_pyfunction!(_resolved_camera, module)?)?;
    module.add_function(wrap_pyfunction!(
        portal_assignment::_linear_sum_assignment,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        portal_streamlines::_stream_line_samples,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        portal_streamlines::_stream_line_widths,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        portal_provenance::_portal_publish_manifest,
        module
    )?)?;
    module.add("_StaleHandleError", py.get_type::<StaleHandleError>())?;
    module.add("_ForeignStageError", py.get_type::<ForeignStageError>())?;
    module.add("_FamilyCycleError", py.get_type::<FamilyCycleError>())?;
    module.add("_CapabilityError", py.get_type::<CapabilityError>())?;
    module.add("_TexError", py.get_type::<TexError>())?;
    module.add("_EndScene", py.get_type::<EndScene>())?;
    module.add("__engine__", "FrankenManim")?;
    module.add(
        "__thread_policy__",
        "scene and mobject proxies are confined to their creating scene-worker thread",
    )?;
    portal_typography::install(module)?;
    portal_network::install(module)?;
    portal_texture::install(module)?;
    execute_bootstrap(py, module)?;
    portal_recording::install(module)?;
    // Packaging identity is owned by Cargo, not a second hand-maintained
    // Python version string.  W11's wheel and console entry point both read
    // these sentinels after the schema bootstrap has assembled `manimlib`.
    // `PyModule::add` records names in PyO3's generated `__all__`.  The
    // Reference intentionally has no `__all__`, so packaging metadata is set
    // as an ordinary attribute after the bootstrap has removed that helper.
    module.setattr("__version__", env!("CARGO_PKG_VERSION"))?;
    module.setattr("__distribution__", "franken-manim")?;
    module.setattr("__franken_manim__", true)?;
    module.setattr("__abi_policy__", "cpython-3.13-full-abi")?;
    // The accompanying pure-Python package is part of the portal. Direct
    // ExtensionFileLoader and embedded users must see the same playback,
    // recovery, output and interaction semantics as the installed wheel.
    py.import("fmn_python.initialization")?
        .getattr("initialize")?
        .call1((module,))?;
    Ok(())
}

/// Initialize the direct extension module used by the embedded acceptance
/// suite, by developers loading the Cargo cdylib without a wheel, and as the
/// wheel package's private `manimlib.manimlib` native member.
#[pymodule(gil_used = true)]
fn manimlib(py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    populate_manimlib(py, module)
}

/// Serialize the Python-embedding acceptance suites: they share one
/// process-global interpreter and one `sys.modules["manimlib"]` slot, so
/// concurrent module construction races. Poison-tolerant: a panicked suite
/// must not wedge the others.
pub(crate) fn python_embedding_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|error| error.into_inner())
}

/// Run one embedded-Python suite with explicit owner-thread teardown.
///
/// PyO3's `unsendable` guard intentionally refuses to destroy a proxy on a
/// thread other than the one which created it. Merely serializing the Rust
/// test functions is therefore insufficient: the test harness may run the
/// next suite on another OS thread while the previous suite still has
/// module cycles in `sys.modules`. Keep the lock through module removal and
/// cyclic GC, capture every unraisable destructor error, and require the
/// temporary root module to become unreachable before releasing the lock.
#[cfg(any(test, feature = "gauntlet"))]
pub(crate) fn with_python_test_module<T>(
    suite: &'static str,
    body: impl for<'py> FnOnce(Python<'py>, &Bound<'py, PyModule>, &Bound<'py, PyDict>) -> T,
) -> T {
    let _lock = python_embedding_lock();
    Python::initialize();
    Python::attach(|py| {
        let sys = py.import("sys").expect("import sys");
        let gc = py.import("gc").expect("import gc");
        let weakref = py.import("weakref").expect("import weakref");
        let modules = sys.getattr("modules").expect("sys.modules");
        let module_names = || -> PyResult<HashSet<String>> {
            modules
                .call_method0("keys")?
                .try_iter()?
                .map(|item| item.and_then(|name| name.extract::<String>()))
                .collect()
        };
        let before = module_names().expect("snapshot sys.modules");
        assert!(
            before.iter().all(|name| !name.starts_with("manimlib")),
            "{suite}: a prior Python suite leaked manimlib modules"
        );
        let videos_ref = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../scripts/videos_ref")
            .canonicalize()
            .ok();

        let hook_globals = PyDict::new(py);
        hook_globals
            .set_item(
                "_fmn_source_package",
                concat!(env!("CARGO_MANIFEST_DIR"), "/python"),
            )
            .expect("embedding source package path");
        hook_globals
            .set_item("_fmn_unraisable", PyList::empty(py))
            .expect("install unraisable capture list");
        let hook_source = CString::new(
            r#"import sys as _fmn_sys
_fmn_previous_path = _fmn_sys.path[:]
_fmn_sys.path.insert(0, _fmn_source_package)
_fmn_old_unraisablehook = _fmn_sys.unraisablehook
def _fmn_capture_unraisable(event):
    _fmn_unraisable.append(
        f'{type(event.exc_value).__name__}: {event.exc_value}'
    )
_fmn_sys.unraisablehook = _fmn_capture_unraisable
"#,
        )
        .expect("unraisable hook source contains no NUL");
        py.run(
            hook_source.as_c_str(),
            Some(&hook_globals),
            Some(&hook_globals),
        )
        .expect("install unraisable hook");

        let module = PyModule::new(py, "manimlib").expect("create test module");
        modules
            .set_item("manimlib", &module)
            .expect("install manimlib");
        let module_weakref = weakref
            .getattr("ref")
            .and_then(|constructor| constructor.call1((&module,)))
            .expect("weak-reference the temporary manimlib module");
        let suite_globals = PyDict::new(py);
        suite_globals
            .set_item("__name__", format!("__fmn_{suite}_tests__"))
            .expect("set suite module name");

        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            manimlib(py, &module).expect("initialize manimlib");
            body(py, &module, &suite_globals)
        }));
        py.run(
            c"_fmn_sys.path[:] = _fmn_previous_path",
            Some(&hook_globals),
            None,
        )
        .expect("restore embedding host module search path");
        let lifetime_globals = PyDict::new(py);
        lifetime_globals.set_item("gc", &gc).expect("GC observer");
        lifetime_globals
            .set_item("weakref", &weakref)
            .expect("weak observer");
        lifetime_globals
            .set_item("Bridge", py.get_type::<BridgeMobject>())
            .expect("proxy type");
        py.run(
            c"proxies = [\n    (weakref.ref(obj), type(obj).__name__)\n    for obj in gc.get_objects()\n    if isinstance(obj, Bridge)\n]",
            Some(&lifetime_globals),
            None,
        )
        .expect("observe native proxy lifetimes without retaining them");
        // Cached classes can own native proxies through class attributes or
        // callback closures. Drop those strong owners with this worker's GIL,
        // before Rust thread-local destruction can defer them to another thread.
        method_cache::clear_for_worker_teardown(py);
        // Python callbacks retain their globals, while suite globals retain
        // PyO3 instances and their callbacks. Some of those extension types
        // are intentionally outside Python's cyclic-GC graph, so the
        // embedding host must break the cycle explicitly on the owner.
        suite_globals.clear();

        let after = module_names().expect("snapshot suite-added modules");
        for name in after.difference(&before) {
            let is_manimlib = name.starts_with("manimlib");
            let is_corpus_module = videos_ref.as_ref().is_some_and(|root| {
                modules
                    .get_item(name)
                    .and_then(|module| module.getattr("__file__"))
                    .and_then(|path| path.extract::<String>())
                    .is_ok_and(|path| std::path::Path::new(&path).starts_with(root))
            });
            if is_manimlib || is_corpus_module {
                if let Ok(value) = modules.get_item(name)
                    && let Ok(owned_module) = value.cast_into::<PyModule>()
                {
                    owned_module.dict().clear();
                }
                let removal = modules.del_item(name);
                assert!(
                    removal.is_ok(),
                    "{suite}: remove module {name}: {:?}",
                    removal.err()
                );
            }
        }
        // PyO3 functions added with `module.add_function` keep the extension
        // module as their `__self__`. CPython does not collect that builtin
        // function <-> module-dict cycle by itself, so explicitly clear the
        // temporary module exactly as an embedding host would during worker
        // teardown. This also releases suite globals before the owner thread
        // can change.
        module.dict().clear();
        drop(module);
        assert_eq!(
            method_cache::stats().entries,
            0,
            "cache must be empty before GC"
        );
        gc.call_method0("collect")
            .expect("collect suite-owned Python cycles");
        // Scene class/global cycles can keep native Points alive through the
        // first collection. The console failure scenario reproduced this
        // ownership pattern before the dispatcher's state became arrays.
        // Keep the weak observers below: a second pass is not a waiver for
        // any proxy that still survives its owner thread.
        gc.call_method0("collect")
            .expect("collect remaining scene class/global cycles");
        py.run(
            c"survivors = [\n    (name, [type(owner).__name__ for owner in gc.get_referrers(ref())])\n    for ref, name in proxies\n    if ref() is not None\n]",
            Some(&lifetime_globals),
            None,
        )
        .expect("inspect retained native proxy owners");
        let survivors: Vec<(String, Vec<String>)> = lifetime_globals
            .get_item("survivors")
            .expect("surviving proxy lookup")
            .expect("surviving proxy observation")
            .extract()
            .expect("native proxy lifetime observations");

        let old_hook = hook_globals
            .get_item("_fmn_old_unraisablehook")
            .expect("lookup prior unraisable hook")
            .expect("prior unraisable hook exists");
        sys.setattr("unraisablehook", old_hook)
            .expect("restore prior unraisable hook");
        let unraisable: Vec<String> = hook_globals
            .get_item("_fmn_unraisable")
            .expect("lookup unraisable capture")
            .expect("unraisable capture exists")
            .extract()
            .expect("extract unraisable errors");
        assert!(
            unraisable.is_empty(),
            "{suite}: Python teardown emitted unraisable errors: {unraisable:?}"
        );
        assert!(
            survivors.is_empty(),
            "{suite}: native proxies survived owner-thread teardown: {survivors:?}"
        );
        let surviving_module = module_weakref.call0().expect("read module weak reference");
        let referrer_types = if surviving_module.is_none() {
            Vec::new()
        } else {
            gc.call_method1("get_referrers", (&surviving_module,))
                .expect("inspect leaked module referrers")
                .try_iter()
                .expect("iterate leaked module referrers")
                .map(|item| {
                    item.and_then(|value| {
                        let t = value.get_type().name()?.to_string();
                        let r = value.repr()?.extract::<String>()?;
                        Ok(format!("{t}: {r}"))
                    })
                    .unwrap_or_else(|error| format!("<unreadable: {error}>"))
                })
                .collect()
        };
        assert!(
            surviving_module.is_none(),
            "{suite}: temporary manimlib module survived owner-thread teardown; \
             referrer types: {referrer_types:?}"
        );
        assert!(
            module_names()
                .expect("verify restored sys.modules")
                .iter()
                .all(|name| !name.starts_with("manimlib")),
            "{suite}: embedded Python suite left a manimlib module installed"
        );

        match outcome {
            Ok(value) => value,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    })
}

/// Production captured-preview seam, available only to the Gauntlet.
#[cfg(feature = "gauntlet")]
pub use portal_studio::{PortalStudioGauntletReport, run_portal_gauntlet_studio_capture};

/// Structured result from the feature-gated in-process Gauntlet portal row.
#[cfg(feature = "gauntlet")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortalGauntletReport {
    /// Published PNG-sequence directory.
    pub path: PathBuf,
    /// Number of lifecycle captures published.
    pub frame_count: u64,
    /// Total PNG bytes in the generation.
    pub bytes: u64,
    /// Canonical ordered PNG-tree digest.
    pub digest: String,
    /// Journal-ready Lumen engine identity.
    pub engine: String,
    /// Fixed render team width.
    pub threads: usize,
}

/// Drive one real Python `Scene` through its production Lumen/Reel route.
///
/// This adapter exists only for the Gauntlet feature used by
/// `fmn-conformance`; the installed wheel exposes the same route through the
/// `fmn-python` console rather than a Rust test hook.
#[cfg(feature = "gauntlet")]
pub fn run_portal_gauntlet_png_sequence(
    destination: &std::path::Path,
    seed: u64,
) -> Result<PortalGauntletReport, String> {
    run_portal_gauntlet_png(destination, seed, false)
}

/// Drive one real Python `Scene` to a single final-state PNG through Lumen/Reel.
#[cfg(feature = "gauntlet")]
pub fn run_portal_gauntlet_png_still(
    destination: &std::path::Path,
    seed: u64,
) -> Result<PortalGauntletReport, String> {
    run_portal_gauntlet_png(destination, seed, true)
}

/// Run the same decoded GIF/y4m/WAV acceptance as the native binary and clean wheel.
/// `FMN_REQUIRE_FFMPEG=1` requires real encoder acceptance in addition to the
/// always-run absence refusal.
#[cfg(feature = "gauntlet")]
pub fn run_portal_gauntlet_native_outputs(
    destination: &std::path::Path,
) -> Result<PortalOutputGauntletReport, String> {
    with_python_test_module("native output Gauntlet", |py, _module, globals| {
        globals
            .set_item("_fmn_output_root", destination.to_string_lossy().as_ref())
            .map_err(|error| error.to_string())?;
        let source = CString::new(include_str!("../tests/native_outputs.py"))
            .expect("native output suite contains no NUL");
        py.run(source.as_c_str(), Some(globals), Some(globals))
            .inspect_err(|error| error.print(py))
            .map_err(|error| error.to_string())?;
        let report = globals
            .get_item("native_output_report")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "native output suite emitted no report".to_owned())?;
        let read = |name| -> PyResult<u64> { report.get_item(name)?.extract() };
        Ok(PortalOutputGauntletReport {
            frames: read("frames").map_err(|error| error.to_string())?,
            formats: read("formats").map_err(|error| error.to_string())?,
            publication_failures: read("publication_failures")
                .map_err(|error| error.to_string())?,
            thread_replays: read("thread_replays").map_err(|error| error.to_string())?,
            sample_frames: read("sample_frames").map_err(|error| error.to_string())?,
            video_formats: read("video_formats").map_err(|error| error.to_string())?,
            video_frames: read("video_frames").map_err(|error| error.to_string())?,
            video_capability_refusals: read("video_capability_refusals")
                .map_err(|error| error.to_string())?,
        })
    })
}

/// Exercise negotiated output through both programmatic and console owners.
/// The shared acceptance source independently decodes real encoder output;
/// `FMN_REQUIRE_FFMPEG=1` makes encoder availability a hard prerequisite.
#[cfg(feature = "gauntlet")]
pub fn run_portal_gauntlet_video_options() -> Result<(), String> {
    with_python_test_module("video options Gauntlet", |py, _module, globals| {
        let source = CString::new(include_str!("../tests/video_options.py"))
            .expect("video options suite contains no NUL");
        py.run(source.as_c_str(), Some(globals), Some(globals))
            .inspect_err(|error| error.print(py))
            .map_err(|error| error.to_string())
    })
}

/// What [`run_portal_gauntlet_planar_route`] observed (fm-sq8.11).
#[cfg(feature = "gauntlet")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortalPlanarRouteGauntletReport {
    /// The planar scene's PNG-sequence directory (standard mode, one thread:
    /// certified output needs a file-backed extension this process lacks).
    pub portal_sequence: PathBuf,
    /// The same scene exported as a planar FMTL/1 bundle for native replay.
    pub bundle: PathBuf,
    /// Frames the planar scene published.
    pub frame_count: u64,
    /// `(planar, camera, first camera reason)` for the planar scene.
    pub planar_routes: (u64, u64, Option<String>),
    /// The same for a scene holding a surface.
    pub surface_routes: (u64, u64, Option<String>),
    /// The planar render's `(teams used, teams planned, threads per team,
    /// frames in flight)`.
    pub planar_plan: (u64, u64, u64, u64),
}

/// One semantic-oracle reading from the portal suite (fm-5wq.46).
#[cfg(feature = "gauntlet")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortalSemanticReading {
    /// `portal_scene` or `portal_camera_readback`.
    pub route: String,
    /// `rendered` or `planted_vertical_flip`.
    pub frame: String,
    /// The oracle, e.g. `orientation.triangle`.
    pub oracle: String,
    /// What it measured.
    pub measured: String,
    /// Whether the frame satisfied it.
    pub pass: bool,
}

/// Run `tests/semantic_witness.py`: the semantic witness through the
/// portal's scene render and camera readback, each frame and its planted
/// vertical mirror read by the semantic sanity oracles. The suite itself
/// asserts the verdicts; the readings come back for the e2e log.
///
/// # Errors
/// The suite's own failure, as text.
#[cfg(feature = "gauntlet")]
pub fn run_portal_gauntlet_semantic_witness() -> Result<Vec<PortalSemanticReading>, String> {
    with_python_test_module("semantic witness Gauntlet", |py, _module, globals| {
        let source = CString::new(include_str!("../tests/semantic_witness.py"))
            .expect("semantic witness suite contains no NUL");
        py.run(source.as_c_str(), Some(globals), Some(globals))
            .inspect_err(|error| error.print(py))
            .map_err(|error| error.to_string())?;
        let report = globals
            .get_item("semantic_witness_report")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "semantic witness suite emitted no report".to_owned())?;
        let readings: Vec<(String, String, String, String, bool)> = report
            .get_item("readings")
            .and_then(|readings| readings.extract())
            .map_err(|error| error.to_string())?;
        Ok(readings
            .into_iter()
            .map(
                |(route, frame, oracle, measured, pass)| PortalSemanticReading {
                    route,
                    frame,
                    oracle,
                    measured,
                    pass,
                },
            )
            .collect())
    })
}

/// Execute `tests/hoeffding_portal.py` unedited (two segments of the Hoeffding's
/// D explainer, as ordinary `manimlib` scene code) and render `RingHook` and
/// `ShuffleLiveD` to PNG sequences under `directory` through
/// `fmn_python.render_scene`, one render thread. Returns each scene's name and
/// published frame count (fm-longform-native-e2e-hoeffding-l59d).
///
/// # Errors
/// The scene file's or a render's failure, as text.
#[cfg(feature = "gauntlet")]
pub fn run_portal_gauntlet_hoeffding(
    directory: &std::path::Path,
    resolution: (u32, u32),
    fps: u32,
) -> Result<Vec<(String, u64)>, String> {
    let directory = directory
        .to_str()
        .ok_or_else(|| "Gauntlet Hoeffding directory is not UTF-8".to_owned())?
        .to_owned();
    with_python_test_module("Hoeffding portal Gauntlet", |py, _module, globals| {
        globals
            .set_item("_fmn_hoeffding_dir", &directory)
            .map_err(|error| error.to_string())?;
        for (name, value) in [
            ("_fmn_width", resolution.0),
            ("_fmn_height", resolution.1),
            ("_fmn_fps", fps),
        ] {
            globals
                .set_item(name, value)
                .map_err(|error| error.to_string())?;
        }
        let scenes = CString::new(include_str!("../tests/hoeffding_portal.py"))
            .expect("the Hoeffding portal scenes contain no NUL");
        py.run(scenes.as_c_str(), Some(globals), Some(globals))
            .inspect_err(|error| error.print(py))
            .map_err(|error| error.to_string())?;
        py.run(
            cr#"import pathlib
from fmn_python import render_scene

hoeffding_portal_report = []
for _scene in (RingHook, ShuffleLiveD):
    _receipt = render_scene(
        _scene, pathlib.Path(_fmn_hoeffding_dir) / _scene.__name__, format="png_sequence",
        resolution=(_fmn_width, _fmn_height), fps=_fmn_fps, threads=1)
    hoeffding_portal_report.append((_scene.__name__, int(_receipt.frame_count)))
"#,
            Some(globals),
            Some(globals),
        )
        .inspect_err(|error| error.print(py))
        .map_err(|error| error.to_string())?;
        globals
            .get_item("hoeffding_portal_report")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "the Hoeffding portal driver emitted no report".to_owned())?
            .extract::<Vec<(String, u64)>>()
            .map_err(|error| error.to_string())
    })
}

/// Render one pure-2D Python scene through the portal, export the same scene
/// as FMTL/1, and render a surface scene; report the routes taken.
#[cfg(feature = "gauntlet")]
pub fn run_portal_gauntlet_planar_route(
    directory: &std::path::Path,
) -> Result<PortalPlanarRouteGauntletReport, String> {
    with_python_test_module("planar route Gauntlet", |py, _module, globals| {
        globals
            .set_item("_fmn_dir", directory.to_string_lossy().as_ref())
            .map_err(|error| error.to_string())?;
        let source = CString::new(
            r#"import os

import numpy as np
from manimlib import BLUE, YELLOW, Circle, ParametricSurface, Scene, Square, Transform, linear
from fmn_python import export_bundle


class _Flat(Scene):
    def construct(self):
        square = Square(side_length=2, fill_opacity=0.6, stroke_width=6).set_color(BLUE)
        self.add(square)
        self.play(Transform(square, Circle(radius=1).set_stroke(YELLOW, 4)),
                  run_time=0.5, rate_func=linear)


class _Curved(Scene):
    def construct(self):
        self.add(ParametricSurface(lambda u, v: np.array([u, v, u * v]),
                                   u_range=(-1, 1), v_range=(-1, 1)))
        self.wait(0.25)


flat = _Flat()
flat_result = flat.render(os.path.join(_fmn_dir, "portal"), format="png_sequence",
                          resolution=(64, 36), fps=8, threads=1)
_fmn_flat_routes = tuple(flat._render_routes)
_fmn_flat_plan = tuple(flat._render_plan)
export_bundle(_Flat, os.path.join(_fmn_dir, "flat.fmtl"), resolution=(64, 36), fps=8)
curved = _Curved()
curved.render(os.path.join(_fmn_dir, "curved.y4m"), format="y4m",
              resolution=(64, 36), fps=8, threads=1)
_fmn_curved_routes = tuple(curved._render_routes)
_fmn_flat_frames = int(flat_result.frame_count)
"#,
        )
        .expect("planar route scene contains no NUL");
        py.run(source.as_c_str(), Some(globals), Some(globals))
            .inspect_err(|error| error.print(py))
            .map_err(|error| error.to_string())?;
        let get = |name: &str| {
            globals
                .get_item(name)
                .map_err(|error| error.to_string())?
                .ok_or_else(|| format!("planar route scene emitted no {name}"))
        };
        let planar_routes = get("_fmn_flat_routes")?
            .extract()
            .map_err(|error: PyErr| error.to_string())?;
        let surface_routes = get("_fmn_curved_routes")?
            .extract()
            .map_err(|error: PyErr| error.to_string())?;
        let frame_count = get("_fmn_flat_frames")?
            .extract()
            .map_err(|error: PyErr| error.to_string())?;
        let planar_plan = get("_fmn_flat_plan")?
            .extract()
            .map_err(|error: PyErr| error.to_string())?;
        Ok(PortalPlanarRouteGauntletReport {
            portal_sequence: directory.join("portal"),
            bundle: directory.join("flat.fmtl"),
            frame_count,
            planar_routes,
            surface_routes,
            planar_plan,
        })
    })
}

/// What [`run_portal_gauntlet_reference_defaults`] observed.
#[cfg(feature = "gauntlet")]
#[derive(Debug)]
pub struct PortalDefaultsGauntletReport {
    /// The receipt's frame rate for a scene with no camera configuration.
    pub fps: u64,
    /// Frames the receipt reports.
    pub frame_count: u64,
    /// Width of a `Brace` around a detached two-member `VGroup`.
    pub brace_width: f64,
    /// That `VGroup`'s own width.
    pub target_width: f64,
    /// A bare `Scene().camera_config`: resolution, fps, background, opacity.
    pub scene_camera: ((u32, u32), u32, String, f64),
    /// `Tex(r"\frac{1}{2}").get_height()` at the default font size.
    pub display_fraction_height: f64,
    /// The README quick example's formula: `(width, height)`.
    pub readme_formula: (f64, f64),
}

/// Reference-default parity through the production portal route
/// (fm-5wq.44, fm-5wq.43). A scene with no camera configuration renders a
/// y4m to `destination` at the pinned config's 30 fps on `#333333`, and a
/// brace around a detached composite spans the composite; the caller decodes
/// the artifact independently.
#[cfg(feature = "gauntlet")]
pub fn run_portal_gauntlet_reference_defaults(
    destination: &std::path::Path,
) -> Result<PortalDefaultsGauntletReport, String> {
    with_python_test_module("reference defaults Gauntlet", |py, _module, globals| {
        globals
            .set_item("_fmn_destination", destination.to_string_lossy().as_ref())
            .map_err(|error| error.to_string())?;
        let source = CString::new(
            r#"from manimlib import Brace, Circle, DOWN, RIGHT, Scene, Square, Tex, VGroup
from fmn_python import render_scene


class _ReferenceDefaults(Scene):
    def construct(self):
        self.wait(0.5)


receipt = render_scene(_ReferenceDefaults, _fmn_destination, format="y4m",
                       resolution=(32, 18), threads=1)
group = VGroup(Square(), Circle().shift(3 * RIGHT))
brace = Brace(group, DOWN)
_fmn_report = (int(receipt.fps), int(receipt.frame_count),
               float(brace.get_width()), float(group.get_width()))
_camera = Scene().camera_config
_fmn_camera = (tuple(int(n) for n in _camera["resolution"]), int(_camera["fps"]),
               str(_camera["background_color"]), float(_camera["background_opacity"]))
_fmn_display_fraction_height = float(Tex(r"\frac{1}{2}").get_height())
_readme = Tex(r"\int_0^\infty e^{-x^2}\,dx = \frac{\sqrt{\pi}}{2}")
_fmn_readme_formula = (float(_readme.get_width()), float(_readme.get_height()))
"#,
        )
        .expect("reference defaults scene contains no NUL");
        py.run(source.as_c_str(), Some(globals), Some(globals))
            .inspect_err(|error| error.print(py))
            .map_err(|error| error.to_string())?;
        let (fps, frame_count, brace_width, target_width): (u64, u64, f64, f64) = globals
            .get_item("_fmn_report")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "reference defaults scene emitted no report".to_owned())?
            .extract()
            .map_err(|error: PyErr| error.to_string())?;
        let scene_camera: ((u32, u32), u32, String, f64) = globals
            .get_item("_fmn_camera")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "reference defaults scene emitted no camera config".to_owned())?
            .extract()
            .map_err(|error: PyErr| error.to_string())?;
        let display_fraction_height: f64 = globals
            .get_item("_fmn_display_fraction_height")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "reference defaults scene emitted no Tex height".to_owned())?
            .extract()
            .map_err(|error: PyErr| error.to_string())?;
        let readme_formula: (f64, f64) = globals
            .get_item("_fmn_readme_formula")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "reference defaults scene emitted no README formula box".to_owned())?
            .extract()
            .map_err(|error: PyErr| error.to_string())?;
        Ok(PortalDefaultsGauntletReport {
            fps,
            frame_count,
            brace_width,
            target_width,
            scene_camera,
            display_fraction_height,
            readme_formula,
        })
    })
}

/// Observations from independently decoded portal artifacts and planted failures.
#[cfg(feature = "gauntlet")]
#[derive(Debug)]
pub struct PortalOutputGauntletReport {
    /// Frames decoded from each native motion artifact.
    pub frames: u64,
    /// Native formats exercised.
    pub formats: u64,
    /// Native publication failures that preserved the destination.
    pub publication_failures: u64,
    /// Native artifacts replayed across thread counts.
    pub thread_replays: u64,
    /// Stereo sample frames decoded from the native WAV.
    pub sample_frames: u64,
    /// Encoded video formats exercised (zero without ffmpeg).
    pub video_formats: u64,
    /// Frames decoded across the motion video artifacts.
    pub video_frames: u64,
    /// Forced missing-encoder refusals, including on qualified hosts.
    pub video_capability_refusals: u64,
}

/// Exercise live sampled wireframes and an independently authored PNG oracle.
#[cfg(feature = "gauntlet")]
pub fn run_portal_gauntlet_surface_mesh(
    destination: &std::path::Path,
    seed: u64,
) -> Result<(Vec<u8>, u64), String> {
    let destination = destination
        .to_str()
        .ok_or_else(|| "surface mesh destination is not UTF-8".to_owned())?;
    with_python_test_module("surface mesh Gauntlet", |py, _module, globals| {
        let source = CString::new(include_str!("../tests/surface_mesh.py"))
            .expect("surface mesh suite contains no NUL");
        py.run(source.as_c_str(), Some(globals), Some(globals))
            .inspect_err(|error| error.print(py))
            .map_err(|error| error.to_string())?;
        globals
            .get_item("render_surface_mesh")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "surface mesh renderer is absent".to_owned())?
            .call1((destination, seed))
            .inspect_err(|error| error.print(py))
            .and_then(|value| value.extract::<(Vec<u8>, u64)>())
            .map_err(|error| error.to_string())
    })
}

/// Exercise the shared Animation/Transform lifecycle through native PNG output.
#[cfg(feature = "gauntlet")]
pub fn run_portal_gauntlet_animation_lifecycle(
    destination: &std::path::Path,
    seed: u64,
) -> Result<PortalGauntletReport, String> {
    let destination = destination
        .to_str()
        .ok_or_else(|| "animation lifecycle destination is not UTF-8".to_owned())?;
    with_python_test_module("animation lifecycle", |py, _module, globals| {
        let source = CString::new(include_str!("../tests/animation_semantics.py"))
            .expect("animation suite contains no NUL");
        py.run(source.as_c_str(), Some(globals), Some(globals))
            .inspect_err(|error| error.print(py))
            .map_err(|error| error.to_string())?;
        let render = globals
            .get_item("render_animation_lifecycle")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "animation lifecycle renderer is absent".to_owned())?;
        let report = render
            .call1((destination, seed))
            .inspect_err(|error| error.print(py))
            .and_then(|value| value.extract::<(String, u64, u64, String, String, usize)>())
            .map_err(|error| error.to_string())?;
        Ok(PortalGauntletReport {
            path: PathBuf::from(report.0),
            frame_count: report.1,
            bytes: report.2,
            digest: report.3,
            engine: report.4,
            threads: report.5,
        })
    })
}

#[cfg(feature = "gauntlet")]
fn run_portal_gauntlet_png(
    destination: &std::path::Path,
    seed: u64,
    single_frame: bool,
) -> Result<PortalGauntletReport, String> {
    let destination = destination
        .to_str()
        .ok_or_else(|| "Gauntlet portal destination is not UTF-8".to_owned())?
        .to_owned();
    with_python_test_module("portal Gauntlet", |py, _module, globals| {
        globals
            .set_item("_fmn_destination", &destination)
            .map_err(|error| error.to_string())?;
        globals
            .set_item("_fmn_seed", seed)
            .map_err(|error| error.to_string())?;
        globals
            .set_item("_fmn_single_frame", single_frame)
            .map_err(|error| error.to_string())?;
        let source = CString::new(
            r#"from manimlib import AnnularSector, Arrow, Axes, BLUE_C, BLUE_E, BraceLabel, BraceText, BulletedList, Checkmark, Circle, Cone, Cross, CubicBezier, CurvedDoubleArrow, DashedLine, DashedVMobject, Disk3D, Dot, Dodecahedron, Elbow, Exmark, FullScreenFadeRectangle, FullScreenRectangle, FunctionGraph, GREY_C, GrowArrow, GrowFromCenter, GrowFromEdge, GrowFromPoint, ImplicitFunction, Line, Line3D, Matrix, ParametricSurface, Polygon, Polyline, Prismify, RED, Rectangle, Rotate, Rotating, RoundedRectangle, Scene, ScreenRectangle, Square3D, StrokeArrow, TangentLine, TimeVaryingVectorField, Title, Torus, Underline, VCube, VGroup3D, VMobject, VPrism, Vector, VectorField, linear

class _GauntletPortalScene(Scene):
    # NumPy's compatibility RandomState accepts only 32-bit scalar seeds.
    # Keep the Gauntlet's full u64 for the native Scene/RngRoot below while
    # deriving the source-scene seed explicitly at this Python-only boundary.
    random_seed = _fmn_seed & 0xFFFF_FFFF

    def construct(self):
        frame_fade = FullScreenFadeRectangle(fill_opacity=0.04)
        frame_fill = FullScreenRectangle(
            height=0.45,
            fill_color=BLUE_E,
            fill_opacity=0.35,
        ).shift((0.0, 1.65, 0.0))
        frame_border = ScreenRectangle(
            height=3.5,
            stroke_color=GREY_C,
            stroke_width=1.0,
        )
        cubic = CubicBezier(
            (-2.6, 1.45, 0.0),
            (-2.2, 2.0, 0.0),
            (-1.4, 2.0, 0.0),
            (-1.0, 1.45, 0.0),
            stroke_color=BLUE_C,
            stroke_width=2.0,
        )
        elbow = Elbow(
            width=0.45,
            angle=0.35,
            stroke_color=RED,
            stroke_width=2.0,
        ).shift((1.9, 1.25, 0.0))
        native_path = VMobject(stroke_width=3.0)
        native_path.start_new_path((-2.0, -1.4, 0.0))
        native_path.add_cubic_bezier_curve_to(
            (-1.5, -0.6, 0.0),
            (1.5, -0.6, 0.0),
            (2.0, -1.4, 0.0),
        )
        native_path.add_arc_to((2.5, -0.9, 0.0), 0.65)
        native_path.add_subpath((
            (-0.6, -1.6, 0.0),
            (0.0, -1.1, 0.0),
            (0.6, -1.6, 0.0),
        )).close_path()
        native_path.make_jagged(recurse=False)
        portal_ops = VMobject(stroke_width=2.0).set_points((
            (-1.8, 0.0, 0.0),
            (-1.2, 0.0, 0.0),
            (-1.2, 0.6, 0.0),
        ))
        portal_ops.append_vectorized_mobject(
            VMobject(stroke_width=2.5).set_points_as_corners((
                (-0.8, 0.0, 0.0),
                (-0.2, 0.4, 0.0),
            ))
        )
        alignment_peer = VMobject().set_points_as_corners((
            (-1.8, 0.0, 0.0),
            (-1.4, 0.2, 0.0),
            (-1.0, 0.0, 0.0),
            (-0.6, 0.2, 0.0),
            (-0.2, 0.0, 0.0),
        ))
        portal_ops.align_points(alignment_peer)
        portal_ops.subdivide_sharp_curves(1.0, recurse=False)
        smooth_ops = VMobject(stroke_width=2.0).set_points_smoothly((
            (0.2, 0.0, 0.0),
            (0.8, 0.5, 0.0),
            (1.4, 0.0, 0.0),
        ))
        smooth_ops.subdivide_curves_by_condition(
            lambda b0, b1, b2: 1 if b1[1] > b0[1] else 0,
            recurse=False,
        )
        smooth_ops.subdivide_intersections(recurse=False, n_subdivisions=1)
        surface_source = ParametricSurface(
            lambda u, v: (u, v, 0.0),
            u_range=(-0.8, 0.8),
            v_range=(-0.5, 0.5),
            resolution=(9, 7),
        ).shift((2.0, 1.2, 0.0))
        surface_partial = surface_source.copy()
        surface_partial.pointwise_become_partial(
            surface_source, 0.2, 0.8, axis=0
        )
        matcher_target = Circle(radius=0.35).shift((2.35, -1.1, 0.0))
        matcher_cross = Cross(matcher_target, stroke_width=[0, 4, 0])
        matcher_underline = Underline(
            matcher_target, buff=0.08, stroke_width=[0, 2, 2, 0]
        )
        checkmark = Checkmark(font_size=24).shift((2.5, 0.35, 0.0))
        exmark = Exmark(font_size=24).shift((1.8, 0.35, 0.0))
        solid_torus = Torus(
            r1=0.35,
            r2=0.12,
            resolution=(9, 7),
            color=BLUE_C,
        ).shift((-2.55, 0.75, 0.0))
        solid_cone = Cone(
            height=0.65,
            radius=0.22,
            resolution=(9, 5),
            color=RED,
        ).shift((-2.0, 0.65, 0.0))
        solid_line = Line3D(
            (-2.7, 0.15, 0.0),
            (-1.8, 0.15, 0.0),
            width=0.08,
            resolution=(7, 5),
            color=BLUE_E,
        )
        solid_disk = Disk3D(
            radius=0.22,
            resolution=(3, 9),
            color=RED,
        ).shift((-1.45, 0.7, 0.0))
        solid_square = Square3D(
            side_length=0.4,
            resolution=(3, 3),
            color=BLUE_C,
        ).shift((-0.95, 0.7, 0.0))
        vector_solids = VGroup3D(
            VCube(
                side_length=0.35,
                fill_color=BLUE_C,
                stroke_width=0.5,
            ).shift((-0.45, 0.7, 0.0)),
            VPrism(
                width=0.45,
                height=0.3,
                depth=0.2,
                fill_color=RED,
            ).shift((0.05, 0.7, 0.0)),
        )
        vector_dodecahedron = Dodecahedron(
            fill_color=BLUE_E,
            stroke_color=BLUE_C,
            stroke_width=0.5,
        ).scale(0.12).shift((0.55, 0.7, 0.0))
        vector_prismify = Prismify(
            Polygon(
                (-0.2, -0.15, 0.0),
                (0.2, -0.15, 0.0),
                (0.0, 0.2, 0.0),
                fill_color=RED,
                fill_opacity=0.6,
                stroke_width=0.5,
            ),
            depth=0.15,
        ).shift((1.05, 0.7, 0.0))
        native_list = BulletedList(
            "Native text",
            "Deterministic layout",
            buff=0.12,
            font_size=16,
            color=BLUE_C,
        ).scale(0.22).shift((2.4, -1.65, 0.0))
        native_list.fade_all_but(0, opacity=0.4, scale_factor=0.8)
        native_title = Title(
            "Franken",
            "Manim",
            font_size=20,
            match_underline_width_to_text=True,
            underline_style=dict(stroke_width=1.5, stroke_color=RED),
            color=BLUE_E,
        ).scale(0.25).shift((0.0, -3.0, 0.0))
        native_matrix = Matrix(
            [["x", "1"], ["0", "y"]],
            h_buff=0.25,
            v_buff=0.2,
            element_config=dict(font_size=16, color=BLUE_C),
        ).scale(0.22).shift((-2.45, -1.75, 0.0))
        native_brace_label = BraceLabel(
            Circle(radius=0.28).shift((1.4, -1.7, 0.0)),
            "r",
            label_scale=0.35,
            label_buff=0.04,
            color=RED,
        ).scale(0.55)
        native_brace_text = BraceText(
            Circle(radius=0.24).shift((0.75, -1.7, 0.0)),
            "text",
            label_scale=0.25,
            label_buff=0.04,
            color=BLUE_C,
        ).scale(0.5)
        native_function_graph = FunctionGraph(
            lambda x: 0.3 * x * x - 0.2,
            x_range=(-0.7, 0.7, 0.1),
            color=BLUE_C,
            use_smoothing=False,
            stroke_width=1.5,
        ).shift((-0.1, 1.25, 0.0))
        native_implicit = ImplicitFunction(
            lambda x, y: x * x + y * y - 0.16,
            x_range=(-0.6, 0.6),
            y_range=(-0.6, 0.6),
            min_depth=2,
            max_quads=128,
            stroke_color=RED,
            stroke_width=1.5,
        ).shift((0.95, 1.25, 0.0))
        native_line = Line(
            (-1.15, 1.65, 0.0),
            (1.15, 1.65, 0.0),
            stroke_color=RED,
            stroke_width=1.5,
        )
        native_line.set_path_arc(-0.45).set_length(1.8).add_tip(
            length=0.18,
            width=0.14,
        )
        native_arrow = Arrow(
            (-2.35, 2.35, 0.0),
            (-1.35, 2.35, 0.0),
            buff=0.04,
            thickness=2.5,
            tip_width_ratio=4.0,
            max_tip_length_to_length_ratio=0.35,
            max_width_to_length_ratio=0.08,
            fill_color=BLUE_C,
        ).scale(0.9)
        native_arrow.set_thickness(2.75).put_start_and_end_on(
            (-2.3, 2.35, 0.0),
            (-1.4, 2.35, 0.0),
        )
        native_vector = Vector(
            (0.65, 0.2, 0.0),
            thickness=2.25,
            max_tip_length_to_length_ratio=0.4,
            max_width_to_length_ratio=0.09,
            fill_color=RED,
        ).shift((1.55, 2.25, 0.0))
        primitive_dot = Dot(
            (-3.0, 2.45, 0.0),
            radius=0.08,
            fill_color=RED,
        )
        primitive_rect = Rectangle(
            width=0.4,
            height=0.25,
            stroke_color=BLUE_C,
            stroke_width=1.0,
        ).surround(primitive_dot, buff=0.06)
        primitive_round = RoundedRectangle(
            width=0.45,
            height=0.28,
            corner_radius=0.07,
            stroke_color=RED,
            stroke_width=1.0,
        ).shift((3.0, 2.45, 0.0))
        primitive_polyline = Polyline(
            (2.7, 2.1, 0.0),
            (3.0, 2.3, 0.0),
            (3.3, 2.1, 0.0),
            stroke_color=BLUE_C,
            stroke_width=1.5,
        )
        native_dashed_line = DashedLine(
            (-3.25, 2.05, 0.0),
            (-2.55, 2.05, 0.0),
            dash_length=0.08,
            positive_space_ratio=0.6,
            stroke_color=BLUE_C,
            stroke_width=1.5,
        )
        native_stroke_arrow = StrokeArrow(
            (-3.25, 1.78, 0.0),
            (-2.55, 1.78, 0.0),
            buff=0.02,
            tip_width_ratio=3.0,
            stroke_color=RED,
            stroke_width=1.5,
        )
        tangent_source = Circle(radius=0.18).shift((2.25, 2.05, 0.0))
        native_tangent_line = TangentLine(
            tangent_source,
            0.125,
            length=0.55,
            stroke_color=RED,
            stroke_width=1.5,
        )
        field_axes = Axes(
            x_range=(-1.0, 1.0, 1.0),
            y_range=(-1.0, 1.0, 1.0),
            width=0.7,
            height=0.7,
        ).shift((-0.7, 2.0, 0.0))
        native_field = VectorField(
            lambda coords: coords * 0.0 + (0.35, 0.1),
            field_axes,
            sample_coords=((-1.0, 0.0), (0.0, 0.0), (1.0, 0.0)),
            max_vect_len=0.3,
            color=BLUE_C,
            stroke_width=1.5,
        )
        time_field_axes = Axes(
            x_range=(-1.0, 1.0, 1.0),
            y_range=(-1.0, 1.0, 1.0),
            width=0.7,
            height=0.7,
        ).shift((0.7, 2.0, 0.0))
        native_time_field = TimeVaryingVectorField(
            lambda coords, time: coords * 0.0 + (0.2 + time, -0.1),
            time_field_axes,
            sample_coords=((-1.0, 0.0), (0.0, 0.0), (1.0, 0.0)),
            max_vect_len=0.3,
            color=RED,
            stroke_width=1.5,
        )
        self.add(
            frame_fade,
            frame_fill,
            frame_border,
            cubic,
            elbow,
            Circle(radius=0.9),
            CurvedDoubleArrow((-1.5, -0.5, 0), (1.5, -0.5, 0)),
            AnnularSector(inner_radius=0.25, outer_radius=0.6).shift((0, 1, 0)),
            DashedVMobject(Circle(radius=0.45), num_dashes=6).shift((-1.25, 1, 0)),
            native_path,
            portal_ops,
            smooth_ops,
            surface_partial,
            matcher_cross,
            matcher_underline,
            checkmark,
            exmark,
            solid_torus,
            solid_cone,
            solid_line,
            solid_disk,
            solid_square,
            vector_solids,
            vector_dodecahedron,
            vector_prismify,
            native_list,
            native_title,
            native_matrix,
            native_brace_label,
            native_brace_text,
            native_function_graph,
            native_implicit,
            native_line,
            native_arrow,
            native_vector,
            primitive_dot,
            primitive_rect,
            primitive_round,
            primitive_polyline,
            native_dashed_line,
            native_stroke_arrow,
            tangent_source,
            native_tangent_line,
            native_field,
            native_time_field,
        )
        # All four growing classes share Choreo's native start-prep
        # Transform.  This production play proves the authored Python MRO and
        # constructor anchors reach real intermediate Lumen frames, including
        # point-color and edge/arrow specializations, rather than stopping at
        # import-compatible shells.
        self.play(
            GrowFromPoint(
                native_stroke_arrow,
                (-3.0, 1.55, 0.0),
                point_color=BLUE_C,
            ),
            GrowFromEdge(primitive_round, (-1.0, 0.0, 0.0)),
            GrowFromCenter(native_tangent_line),
            GrowArrow(native_arrow),
            # The public Animation -> Rotating -> Rotate hierarchy reaches
            # Choreo's absolute-pose rotation in the same real Lumen/Reel
            # frame stream.  Distinct pivots exercise both rotation routes.
            Rotate(primitive_polyline, angle=0.15),
            Rotating(
                primitive_rect,
                angle=-0.1,
                about_point=(0.0, 0.0, 0.0),
            ),
            run_time=1 / 30,
            rate_func=linear,
        )
        self.wait(1 / 30)

_fmn_scene = _GauntletPortalScene()
_fmn_begin = (
    _fmn_scene._begin_png
    if _fmn_single_frame
    else _fmn_scene._begin_png_sequence
)
_fmn_begin(_fmn_destination, 96, 54, 30, 1, _fmn_seed)
try:
    _fmn_scene.run()
    _fmn_report = _fmn_scene._finish_render(
        _fmn_scene.frame._core,
        _fmn_scene.camera.light_source.get_center(),
    )
except Exception:
    _fmn_scene._abort_render()
    raise
"#,
        )
        .expect("Gauntlet source contains no NUL");
        py.run(source.as_c_str(), Some(globals), Some(globals))
            .map_err(|error| error.to_string())?;
        let report = globals
            .get_item("_fmn_report")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "Gauntlet portal script returned no report".to_owned())?
            .extract::<(String, u64, u64, String, String, usize)>()
            .map_err(|error| error.to_string())?;
        Ok(PortalGauntletReport {
            path: PathBuf::from(report.0),
            frame_count: report.1,
            bytes: report.2,
            digest: report.3,
            engine: report.4,
            threads: report.5,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallible_native_graph_sampling() {
        crate::with_python_test_module("fallible graph sampling", |py, _module, globals| {
            let code = CString::new(include_str!("../tests/native_graph_sampling.py")).unwrap();
            py.run(code.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .unwrap();
            globals
                .get_item("run_native_graph_sampling")
                .unwrap()
                .unwrap()
                .call0()
                .inspect_err(|error| error.print(py))
                .unwrap();
        });
    }

    #[test]
    fn production_native_output_acceptance_suite() {
        crate::with_python_test_module("native output acceptance", |py, _module, globals| {
            let source = CString::new(include_str!("../tests/native_outputs.py"))
                .expect("native output suite contains no NUL");
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("decoded native GIF/y4m/WAV acceptance suite");
        });
    }

    #[test]
    fn production_camera_runtime_acceptance_suite() {
        crate::with_python_test_module("camera runtime acceptance", |py, module, globals| {
            assert_eq!(
                module
                    .getattr("_FMN_PORTAL_RUNTIME_STATE")
                    .and_then(|state| state.extract::<String>())
                    .expect("complete production initialization"),
                "ready"
            );
            for (name, text) in [
                (
                    "camera_motion.py",
                    include_str!("../tests/camera_motion.py"),
                ),
                (
                    "camera_execution.py",
                    include_str!("../tests/camera_execution.py"),
                ),
                (
                    "updater_family.py",
                    include_str!("../tests/updater_family.py"),
                ),
            ] {
                globals.set_item("__file__", name).expect("suite filename");
                let source = CString::new(text).expect("camera suite contains no NUL");
                py.run(source.as_c_str(), Some(globals), Some(globals))
                    .inspect_err(|error| error.print(py))
                    .expect("native camera and updater family acceptance");
            }
        });
    }

    #[test]
    fn production_persistent_runtime_acceptance_suite() {
        crate::with_python_test_module("persistent runtime recovery", |py, _module, globals| {
            for (name, text) in [
                (
                    "animation_updaters.py",
                    include_str!("../tests/animation_updaters.py"),
                ),
                (
                    "persistent_recovery.py",
                    include_str!("../tests/persistent_recovery.py"),
                ),
            ] {
                globals.set_item("__file__", name).expect("suite filename");
                let source = CString::new(text).expect("persistent suite contains no NUL");
                py.run(source.as_c_str(), Some(globals), Some(globals))
                    .inspect_err(|error| error.print(py))
                    .expect("native persistent lifecycle and failure ownership");
            }
        });
    }

    #[test]
    fn production_drawing_runtime_acceptance_suite() {
        crate::with_python_test_module("native drawing runtime", |py, _module, globals| {
            for (name, text) in [
                (
                    "creation_semantics.py",
                    include_str!("../tests/creation_semantics.py"),
                ),
                (
                    "drawing_runtime.py",
                    include_str!("../tests/drawing_runtime.py"),
                ),
            ] {
                globals.set_item("__file__", name).expect("suite filename");
                let source = CString::new(text).expect("drawing suite contains no NUL");
                py.run(source.as_c_str(), Some(globals), Some(globals))
                    .inspect_err(|error| error.print(py))
                    .expect("native creation phases and failure ownership");
            }
        });
    }

    #[test]
    fn production_portal_runtime_acceptance_suite() {
        crate::with_python_test_module("complete portal runtime", |py, _module, globals| {
            for (name, text) in [
                (
                    "portal_initialization.py",
                    include_str!("../tests/portal_initialization.py"),
                ),
                (
                    "fading_semantics.py",
                    include_str!("../tests/fading_semantics.py"),
                ),
                (
                    "fading_family.py",
                    include_str!("../tests/fading_family.py"),
                ),
            ] {
                globals.set_item("__file__", name).expect("suite filename");
                let source = CString::new(text).expect("portal suite contains no NUL");
                py.run(source.as_c_str(), Some(globals), Some(globals))
                    .inspect_err(|error| error.print(py))
                    .expect("identical production initialization and exact fade restoration");
            }
        });
    }

    /// fm-5wq.15: the committed effective dispatch table (which layer
    /// implements each Reference method, through how many installer wraps)
    /// is what the embedded portal actually builds. A change to that
    /// layering must change docs/api/portal_dispatch.tsv in the same diff.
    #[test]
    fn portal_dispatch_table_matches_the_embedded_portal() {
        crate::with_python_test_module("portal dispatch table", |py, module, globals| {
            globals
                .set_item("__name__", "portal_dispatch_table")
                .expect("generator module name");
            let source = CString::new(include_str!("../../../scripts/portal_dispatch_table.py"))
                .expect("generator contains no NUL");
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("dispatch table generator loads");
            let generated: String = globals
                .get_item("table")
                .expect("generator globals")
                .expect("table function")
                .call1((module,))
                .and_then(|table| table.extract())
                .inspect_err(|error| error.print(py))
                .expect("dispatch table generates");
            let committed = include_str!("../../../docs/api/portal_dispatch.tsv");
            if let Some((line, (built, recorded))) = generated
                .lines()
                .zip(committed.lines())
                .enumerate()
                .find(|(_, (built, recorded))| built != recorded)
            {
                panic!(
                    "docs/api/portal_dispatch.tsv drifted at line {}:\n  embedded portal: {built}\n  committed:       {recorded}\nregenerate with `python scripts/portal_dispatch_table.py > docs/api/portal_dispatch.tsv` against a wheel of this tree",
                    line + 1
                );
            }
            assert_eq!(
                generated.lines().count(),
                committed.lines().count(),
                "docs/api/portal_dispatch.tsv has a different number of rows than the embedded portal"
            );
        });
    }

    #[test]
    fn production_animation_semantics_acceptance_suite() {
        crate::with_python_test_module("animation acceptance", |py, module, globals| {
            let begin = module
                .getattr("Animation")
                .and_then(|class| class.getattr("begin"))
                .expect("production Animation.begin");
            let semantics =
                CString::new(include_str!("../python/manimlib/_animation_semantics.py"))
                    .expect("embedded animation semantics contain no NUL");
            install_animation_semantics(py, module, semantics.as_c_str())
                .expect("repeated installation is harmless");
            let installed_begin = module
                .getattr("Animation")
                .and_then(|class| class.getattr("begin"))
                .expect("Animation.begin after repeated installation");
            assert!(
                begin.is(&installed_begin),
                "repeated initialization preserves existing method identities"
            );
            let error = install_animation_semantics(py, module, c"pass")
                .expect_err("an absent installer must fail initialization");
            assert!(error.is_instance_of::<pyo3::exceptions::PyImportError>(py));
            assert!(
                error
                    .to_string()
                    .contains("missing production animation semantics installer")
            );
            globals
                .set_item(
                    "__file__",
                    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/animation_semantics.py"),
                )
                .expect("set animation suite source path");
            let source = CString::new(include_str!("../tests/animation_semantics.py"))
                .expect("animation suite contains no NUL");
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("native Animation/Transform acceptance suite");
            let failed_source = globals
                .get_item("failed_source")
                .expect("read mixed-callback abort witness")
                .expect("mixed-callback abort witness ran");
            let failed_source = failed_source.cast::<BridgeMobject>().expect("native proxy");
            with_stage(failed_source, |stage, mob| {
                let entry = stage
                    .get(mob)
                    .expect("aborted animation retains its source");
                assert!(
                    entry.buffer.locked_keys().is_empty(),
                    "callback abort releases actual native record locks"
                );
                assert!(!stage.is_updating_suspended(mob));
            })
            .expect("read native abort state");
        });
    }

    #[test]
    fn production_callback_cleanup_acceptance_suite() {
        crate::with_python_test_module("callback cleanup acceptance", |py, _module, globals| {
            let source = CString::new(include_str!("../tests/callback_cleanup.py"))
                .expect("callback cleanup suite contains no NUL");
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("load native callback cleanup acceptance");
            let checks = globals
                .get_item("verify_callback_cleanup")
                .expect("find callback cleanup acceptance")
                .expect("callback cleanup acceptance is defined")
                .call0()
                .inspect_err(|error| error.print(py))
                .and_then(|value| value.extract::<u64>())
                .expect("native callback cleanup acceptance");
            assert_eq!(checks, 4);
        });
    }

    #[test]
    fn production_surface_mesh_acceptance_suite() {
        crate::with_python_test_module("surface mesh acceptance", |py, _module, globals| {
            let source = CString::new(include_str!("../tests/surface_mesh.py"))
                .expect("surface mesh suite contains no NUL");
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("load native surface mesh acceptance");
            let checks = globals
                .get_item("verify_surface_mesh")
                .expect("find surface mesh acceptance")
                .expect("surface mesh acceptance is defined")
                .call0()
                .inspect_err(|error| error.print(py))
                .and_then(|value| value.extract::<u64>())
                .expect("native surface mesh acceptance");
            assert_eq!(checks, 12);
        });
    }

    #[test]
    fn production_bridge_acceptance_suite() {
        crate::with_python_test_module("bridge acceptance", |py, _module, globals| {
            globals
                .set_item(
                    "__file__",
                    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/bridge.py"),
                )
                .expect("set bridge suite source path");
            globals
                .set_item("_expected_package_version", env!("CARGO_PKG_VERSION"))
                .expect("set bridge suite package version");
            let source = CString::new(include_str!("../tests/bridge.py"))
                .expect("test source contains no NUL");
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .expect("Python bridge acceptance suite");

            // fm-7if keeps affine motion out of object-space records until the
            // authoritative Python data surface demands synchronization. An
            // already exported zero-copy array must still observe engine
            // writes, and a later scalar read must materialize a pending
            // placement before exposing the buffer.
            let parent = globals
                .get_item("parent")
                .expect("globals lookup")
                .expect("bridge suite defines parent");
            let proxy = parent.cast::<BridgeMobject>().expect("parent proxy");
            let (engine, mob) = bound_parts(&proxy.borrow()).expect("bound parent");
            let data = parent.getattr("data").expect("live NumPy data");
            let before: Vec<f32> = data
                .get_item("point")
                .expect("point field")
                .get_item(0)
                .expect("first point")
                .call_method0("tolist")
                .expect("point list")
                .extract()
                .expect("f32 point");
            engine.borrow_mut().stage_mut().shift(mob, [2.0, -1.0, 0.0]);
            let viewed: Vec<f32> = data
                .get_item("point")
                .expect("point field")
                .get_item(0)
                .expect("first point")
                .call_method0("tolist")
                .expect("point list")
                .extract()
                .expect("f32 point");
            #[allow(clippy::cast_possible_truncation)]
            let expected_viewed = [
                (f64::from(before[0]) + 2.0) as f32,
                (f64::from(before[1]) - 1.0) as f32,
                before[2],
            ];
            assert_eq!(viewed, expected_viewed);
            assert!(
                engine
                    .borrow()
                    .stage()
                    .placement(mob)
                    .expect("live")
                    .is_identity(),
                "an attached view receives affine writes in-place"
            );
            drop(data);

            engine.borrow_mut().stage_mut().shift(mob, [1.0, 0.0, 0.0]);
            assert!(
                !engine
                    .borrow()
                    .stage()
                    .placement(mob)
                    .expect("live")
                    .is_identity(),
                "without a view, motion stays in the placement channel"
            );
            let read: Vec<f32> = parent
                .call_method1("get_field", ("point", 0))
                .expect("world-space field read")
                .extract()
                .expect("f32 point");
            #[allow(clippy::cast_possible_truncation)]
            let expected_read = [(f64::from(viewed[0]) + 1.0) as f32, viewed[1], viewed[2]];
            assert_eq!(read, expected_read);
            assert!(
                engine
                    .borrow()
                    .stage()
                    .placement(mob)
                    .expect("live")
                    .is_identity(),
                "an API read synchronizes placement back to RecordBuffer"
            );
        });
    }

    #[test]
    fn python_suite_cycles_are_collected_before_the_owner_thread_changes() {
        fn run_on_fresh_thread(suite: &'static str) -> std::thread::ThreadId {
            std::thread::spawn(move || {
                let owner = std::thread::current().id();
                let retained_type = crate::with_python_test_module(suite, |py, module, _globals| {
                    let instance = module
                        .getattr("Mobject")
                        .and_then(|class| class.call0())
                        .expect("construct teardown probe");
                    instance
                        .setattr("_teardown_cycle", &instance)
                        .expect("make an owned Python cycle");
                    module
                        .setattr("_teardown_probe", instance)
                        .expect("retain the probe from the temporary module");
                    let source = CString::new(
                        "class CachedOwner(Mobject):\n    retained = Mobject()\n    def cached_probe(self):\n        return 7\n",
                    ).expect("cache probe source");
                    py.run(source.as_c_str(), Some(&module.dict()), Some(&module.dict()))
                        .expect("construct cached class with a native-owned attribute");
                    let class = module.getattr("CachedOwner").expect("cached probe class");
                    let instance = class.call0().expect("cached probe instance");
                    let value: u32 = crate::method_cache::call_cached0(&instance, "cached_probe")
                        .expect("populate the actual native method cache")
                        .extract().expect("probe return value");
                    assert_eq!(value, 7);
                    assert!(crate::method_cache::stats().entries > 0);
                    py.import("weakref").expect("weakref")
                        .getattr("ref").expect("weakref.ref")
                        .call1((class,)).expect("retain only a weak type observer")
                        .unbind()
                });
                Python::attach(|py| {
                    assert!(retained_type.bind(py).call0().expect("read type lifetime").is_none(),
                        "worker teardown retained a cached type and its native-owned attributes");
                    assert_eq!(crate::method_cache::stats().entries, 0);
                });
                owner
            })
            .join()
            .expect("Python teardown probe thread")
        }

        let first = run_on_fresh_thread("first owner-thread teardown probe");
        let second = run_on_fresh_thread("second owner-thread teardown probe");
        assert_ne!(
            first, second,
            "the regression requires distinct test owners"
        );
    }

    #[test]
    fn cross_thread_unsendable_access_is_a_typed_refusal() {
        const CHILD_ENV: &str = "FMN_UNSENDABLE_PROBE_CHILD";
        if std::env::var_os(CHILD_ENV).is_none() {
            // PyO3 implements its `unsendable` refusal by catching a Rust
            // panic and translating it into `PanicException`. The default
            // panic hook necessarily writes the caught panic to stderr.
            // Contain that intentional negative control in a child copy of
            // this test binary so the ordinary all-target gate remains
            // stderr-clean without installing a process-global hook which
            // could hide an unrelated concurrent panic.
            let output = std::process::Command::new(
                std::env::current_exe().expect("locate current test binary"),
            )
            .args([
                "--exact",
                "tests::cross_thread_unsendable_access_is_a_typed_refusal",
                "--nocapture",
            ])
            .env(CHILD_ENV, "1")
            .output()
            .expect("run isolated unsendable negative control");
            assert!(
                output.status.success(),
                "isolated unsendable negative control failed\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains("unsendable, but sent to another thread"),
                "the isolated control did not exercise PyO3's thread guard: {stderr}"
            );
            return;
        }

        crate::with_python_test_module(
            "isolated unsendable negative control",
            |py, module, _globals| {
                let object = module
                    .getattr("Mobject")
                    .and_then(|class| class.call0())
                    .expect("construct thread-confined probe")
                    .unbind();
                let (object, error) = py.detach(|| {
                    std::thread::spawn(move || {
                        Python::attach(|py| {
                            let locals = PyDict::new(py);
                            locals
                                .set_item("_fmn_probe", object.bind(py))
                                .expect("install foreign-thread probe");
                            let source = CString::new(
                                "try:\n\
                                 \x20\x20\x20\x20_fmn_probe.n_records()\n\
                                 except BaseException as _fmn_error:\n\
                                 \x20\x20\x20\x20_fmn_error_text = str(_fmn_error)\n\
                                 else:\n\
                                 \x20\x20\x20\x20raise AssertionError('foreign-thread access succeeded')\n",
                            )
                            .expect("foreign-thread probe source contains no NUL");
                            py.run(source.as_c_str(), Some(&locals), Some(&locals))
                                .expect("catch the PyO3 refusal inside Python");
                            let error = locals
                                .get_item("_fmn_error_text")
                                .expect("lookup foreign-thread refusal")
                                .expect("foreign-thread refusal was recorded")
                                .extract::<String>()
                                .expect("extract foreign-thread refusal");
                            locals.clear();
                            (object, error)
                        })
                    })
                    .join()
                    .expect("foreign-thread refusal probe")
                });
                assert!(
                    error.contains("unsendable, but sent to another thread"),
                    "unexpected cross-thread refusal: {error}"
                );
                drop(object);
            },
        );
    }
}

#[cfg(test)]
mod subset_reveal_acceptance {
    #[test]
    fn production_subset_reveal_acceptance() {
        crate::with_python_test_module("native subset reveals", |py, _module, globals| {
            let source = std::ffi::CString::new(include_str!("../tests/subset_reveal.py"))
                .expect("subset reveal source contains no NUL");
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("real native subset selection, lifecycle and output");
        });
    }
}
