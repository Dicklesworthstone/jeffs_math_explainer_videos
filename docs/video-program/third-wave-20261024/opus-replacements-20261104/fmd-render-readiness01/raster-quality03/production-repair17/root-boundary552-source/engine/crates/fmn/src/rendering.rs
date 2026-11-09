//! Native scene-to-artifact composition over Proscenium, Lumen, and Reel.
//!
//! [`render`] runs an ordinary [`SceneConstruct`] and publishes PNG frames, a
//! GIF, or a Y4M stream without CPython or ffmpeg, or an MP4/MOV video through
//! the one governed ffmpeg boundary (D2) with the scene's sound cues mixed
//! natively and muxed. The existing scene clock, retained renderer, bounded
//! frame pipeline, and ordered emitter remain authoritative; this module does
//! not implement a second animation loop.
//!
//! Output is published only after successful scene execution and sink
//! finalization. Errors (and unwinding scene panics) cancel and join the output
//! worker before returning. Existing destinations are never overwritten.
//!
//! `Certified` selects the certified CPU and canonical PNG encoding. A render
//! report is an artifact receipt, **not** a certified input-closure manifest;
//! caller-owned external assets and callbacks still need closure attestation.
//! Encoded video is outside the certified artifact set by construction, so a
//! certified video request is refused rather than mislabelled.

use std::fmt;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fmn_anim::FramePacket;
use fmn_codec::{CompressionLevel, Y4mColorspace};
use fmn_config::Config;
use fmn_config::config::{DeterminismMode, Engine, ThreadPolicy};
use fmn_core::color::{HexParseError, Srgb};
use fmn_frame::{FrameError, FrameLayout, PixelFormat};
use fmn_output::{
    ArtifactDigest, AudioDecodeError, AudioDecodeLimits, AudioDecoder, Boundary, BoundaryError,
    ColorDescription, Container, EmitterConfig, EmitterError, EmitterFailure, EncoderCapabilities,
    EncoderChoice, FfmpegArtifactReport, FfmpegSink, FfmpegSinkConfig, FfmpegSoundtrack,
    FfmpegTool, GifSink, GifSinkConfig, InvocationReport, MixReport, MixerConfig,
    NativeArtifactKind, NegotiationError, OrderedEmitter, PngSink, PngSinkConfig, PngTarget,
    ReceiptError, SinkAdapterError, SinkLimits, SinkReceipt, SoundError, VideoJob, WireFormat,
    Y4mSink, Y4mSinkConfig, frames_to_samples,
};
use fmn_platform::clock::StdClock;
use fmn_platform::fs::{FileSystem, FsError, StdFs};
use fmn_platform::process::{FfmpegLocator, FfmpegLocatorError, ProcessRunner};
use fmn_platform::topology::HardwareTopology;
use fmn_render::{
    Camera, CameraError, EngineIdentity, FrameConfig, RetainedFrameRendererConfig,
    RetainedFrameRendererError, ScreenMap, Tiling, Viewport,
};
use fmn_runtime::{
    ExecutionEngine, FrameStreamError, OutputPixelFormat, PipelineStats, PlanError, PlanRequest,
    RenderIntent, SurfaceSpec,
};
use fmn_scene::{CaptureReason, IntegrationError, RuntimeConfig, SceneRunReport, SceneSink};
use fmn_tex::{TexSession, TypesetSessionReport};

use crate::SceneConstruct;

mod camera_animation;
mod compiled;
mod ffmpeg_limits;
mod pipeline;
mod soundtrack;
pub use camera_animation::{render_camera, render_camera_with_fs};
pub use compiled::{render_bundle, render_bundle_with_fs};
pub use ffmpeg_limits::{
    DEFAULT_RENDER_FFMPEG_TIMEOUT, FfmpegLimitsReport, admit_render_ffmpeg_limits,
    default_render_ffmpeg_limits,
};
pub use pipeline::{NativeFrameError, NativeFramePipeline};
pub use soundtrack::{PreparedSceneSoundtrack, SoundtrackInput, SoundtrackTimeline};

pub use fmn_output::negotiate::{EncoderPreset, EncoderTune, VideoQuality};
pub use fmn_output::{EmitterReport, JobLimits, NativeArtifactReport};
pub use fmn_render::{CameraConfig, CameraFrame, RenderPlanLimits};
pub use fmn_runtime::ExecutionPlan;

/// Read optional `file_writer` video controls after configuration layering.
///
/// Both native front doors use the same typed encoder catalogs. This performs
/// no I/O; the resulting policy still passes through `VideoJob::with_quality`
/// for encoder/container admission. Absent keys preserve encoder defaults.
pub fn configured_video_quality(
    writer: &fmn_config::config::FileWriterConfig,
) -> Result<VideoQuality, fmn_output::NegotiationError> {
    let crf = writer
        .video_crf
        .map(|value| {
            u8::try_from(value).ok().filter(|value| *value <= 51).ok_or(
                fmn_output::NegotiationError("file_writer.video_crf must lie in 0..=51"),
            )
        })
        .transpose()?;
    let preset = writer
        .video_preset
        .as_deref()
        .map(str::parse::<EncoderPreset>)
        .transpose()?;
    let tune = writer
        .video_tune
        .as_deref()
        .map(str::parse::<EncoderTune>)
        .transpose()?;
    Ok(VideoQuality {
        crf,
        preset,
        tune,
        bitrate: writer.video_bitrate,
    })
}

/// Output formats. The output path is a directory for PNG sequences and a
/// file otherwise. No format silently substitutes for another.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderFormat {
    /// `frame_000000.png`, ... and a completion manifest, atomically published
    /// as one new directory generation.
    PngSequence,
    /// Native animated GIF, looping indefinitely.
    Gif,
    /// Native planar 4:2:0 Y4M. Width and height must both be even.
    Y4m,
    /// MP4 video through the governed ffmpeg boundary (D2), encoded with
    /// `file_writer.video_codec` over `file_writer.pixel_format` (4:2:0, so
    /// width and height must be even). Scene sound cues are mixed natively
    /// and muxed. Requires an [`FfmpegCapability`]; `standard` mode only.
    Mp4,
    /// QuickTime MOV through the same boundary. A translucent camera
    /// background selects alpha-preserving `qtrle` over an RGBA wire;
    /// otherwise it is the ordinary 4:2:0 encode in a MOV container.
    Mov,
}

impl RenderFormat {
    /// Whether this format is encoded by ffmpeg rather than a native codec.
    #[must_use]
    pub const fn is_video(self) -> bool {
        matches!(self, Self::Mp4 | Self::Mov)
    }
}

/// The process capability behind video formats: the one external tool (D2).
///
/// It is never ambient in [`render_with_fs`] or the other explicit-capability
/// entry points; [`render`] supplies `FfmpegCapability::host()` only when the
/// crate's `ffmpeg` feature is enabled and the caller left
/// [`RenderOptions::ffmpeg`] unset. Its absence is a capability error that
/// names the native alternatives, never a silent format substitution.
#[derive(Clone)]
pub struct FfmpegCapability {
    /// The exact-image process mechanism. The only program it ever launches
    /// for this entry point is the located ffmpeg.
    pub runner: Arc<dyn ProcessRunner>,
    /// Resolves `file_writer.ffmpeg_bin` to a canonical executable.
    pub locator: Arc<dyn FfmpegLocator>,
    /// Canonical parent of the boundary's private session and job directories.
    pub workdir_root: PathBuf,
}

impl FfmpegCapability {
    /// The host's process runner and ffmpeg locator (`PATH` snapshotted now),
    /// with private work directories under the system temporary directory.
    #[cfg(feature = "ffmpeg")]
    #[must_use]
    pub fn host() -> Self {
        Self {
            runner: Arc::new(fmn_platform::process::StdProcessRunner),
            locator: Arc::new(fmn_platform::process::StdFfmpegLocator::from_host_path()),
            workdir_root: std::env::temp_dir(),
        }
    }
}

impl fmt::Debug for FfmpegCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FfmpegCapability")
            .field("runner", &self.runner.mechanism())
            .field("workdir_root", &self.workdir_root)
            .finish_non_exhaustive()
    }
}

/// The natively mixed soundtrack muxed into a video artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SoundtrackReport {
    /// Cues mixed, in insertion order.
    pub cues_mixed: usize,
    /// Interleaved PCM frames (one sample per channel).
    pub sample_frames: u64,
    /// Output sample rate.
    pub sample_rate: u32,
    /// Output channels.
    pub channels: u16,
    /// Samples that exceeded `[-1, 1]` before the defined clamp.
    pub clipped_samples: u64,
}

/// The published artifact of one render.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderArtifact {
    /// Requested format.
    pub format: RenderFormat,
    /// Published file or immutable sequence-directory root.
    pub path: PathBuf,
    /// Frames represented by the artifact.
    pub frame_count: u64,
    /// Total published bytes.
    pub bytes: u64,
    /// Raw-file digest, or the canonical ordered tree digest for a PNG sequence.
    pub digest: ArtifactDigest,
    /// Video-boundary invocations in execution order (encode, then optional
    /// audio mux), each naming the hashed tool and exact argv. Media decode
    /// may overlap the live encoder and is listed separately in
    /// [`Self::audio_decodes`]. Empty for native codecs.
    pub ffmpeg: Vec<InvocationReport>,
    /// Governed media-decode invocations in cue decode order. Native PCM WAV
    /// adds none. These may overlap the video encode, so the two lists do not
    /// imply a single sequential execution order.
    pub audio_decodes: Vec<InvocationReport>,
    /// Effective resource bounds used for this video's encode, mux, and media
    /// decode. Native artifacts carry no ffmpeg policy. See
    /// [`FfmpegLimitsReport::to_json`] for manifest embedding.
    pub ffmpeg_limits: Option<FfmpegLimitsReport>,
    /// Admitted video-quality overrides; `None` for native codecs. An empty
    /// policy means encoder defaults, not an inferred CRF or measured quality.
    /// The exact emitted flags also remain in the ffmpeg invocation provenance.
    pub video_quality: Option<VideoQuality>,
    /// The soundtrack muxed into a video artifact; `None` when the scene
    /// authored no sound or the format carries no audio.
    pub soundtrack: Option<SoundtrackReport>,
}

impl RenderArtifact {
    fn native(format: RenderFormat, report: NativeArtifactReport) -> Result<Self, RenderError> {
        let expected = match format {
            RenderFormat::PngSequence => NativeArtifactKind::PngSequence,
            RenderFormat::Gif => NativeArtifactKind::Gif,
            RenderFormat::Y4m => NativeArtifactKind::Y4m,
            RenderFormat::Mp4 | RenderFormat::Mov => {
                return Err(RenderError::InvalidOptions(
                    "a video format produced a native artifact receipt",
                ));
            }
        };
        if report.kind != expected {
            return Err(RenderError::InvalidOptions(
                "native sink published a different artifact kind",
            ));
        }
        Ok(Self {
            format,
            path: report.path,
            frame_count: report.frame_count,
            bytes: report.bytes,
            digest: report.digest,
            ffmpeg: Vec::new(),
            audio_decodes: Vec::new(),
            ffmpeg_limits: None,
            video_quality: None,
            soundtrack: None,
        })
    }

    fn video(
        format: RenderFormat,
        report: FfmpegArtifactReport,
        soundtrack: Option<SoundtrackReport>,
    ) -> Self {
        Self {
            format,
            path: report.boundary.destination,
            frame_count: report.frame_count,
            bytes: report.boundary.artifact_bytes,
            digest: report.boundary.artifact_digest,
            ffmpeg: report.boundary.invocations,
            audio_decodes: Vec::new(),
            ffmpeg_limits: None,
            video_quality: None,
            soundtrack,
        }
    }
}

/// Configuration for the native rendering front door.
///
/// [`Self::new`] resolves the bundled configuration without reading ambient
/// files. Callers may replace `config` with their own resolved configuration.
/// Camera resolution/background/fps, frame height, scene timing, seed, AA and
/// CPU thread policy are honored. `file_writer` settings do not override the
/// explicit `format` or `output` below. Accelerator and video-encoder requests
/// are not silently accepted by this CPU/native-codec entry point.
#[derive(Clone, Debug)]
pub struct RenderOptions {
    /// The destination; publication is no-clobber.
    pub output: PathBuf,
    /// Explicit native output format.
    pub format: RenderFormat,
    /// Scene and renderer settings, using the existing configuration schema.
    pub config: Config,
    /// Explicit perspective camera for surfaces, dot clouds, textured images,
    /// and vectors in one painter sequence. `None` keeps the cached 2D route.
    /// Use [`Self::camera_config`] to match the export's viewport and background.
    /// Camera captures use the certified CPU implementation even in standard
    /// mode; `CameraConfig::samples` controls their adaptive edge ceiling.
    pub camera: Option<CameraConfig>,
    /// Maximum number of captured frames (checked before rasterization).
    pub max_frames: u64,
    /// Maximum input stream and encoded artifact size, independently enforced.
    pub max_output_bytes: u64,
    /// Budget for planned frame storage and sink resident buffers, not for the
    /// caller's scene graph or geometry. The sink also enforces its own bound.
    pub max_resident_bytes: u64,
    /// Retained geometry and paint the scene may hold, checked on every captured
    /// frame. The default fits ordinary scenes; one that needs more declares it
    /// here, and frame bits do not depend on it. [`Self::max_resident_bytes`] does
    /// not cover scene geometry, so the caller answers for the memory admitted.
    /// The monotone-table and binning budgets still apply and refuse by name.
    pub plan_limits: RenderPlanLimits,
    /// Upper bound on the output ring and frozen frame-job count; the scheduler
    /// may lower it. Does not change frame sampling or pixels.
    pub frames_in_flight: usize,
    /// Attach the resolved persistent typeset cache when a scene first needs
    /// TeX. Storage refusals fall back to native layout and appear in the report.
    /// False keeps the bounded memory front without any persistent-cache I/O.
    pub typeset_cache: bool,
    /// Ceiling for declared preflight workers, additionally bounded to 64 and
    /// available parallelism. This is independent of frame-render thread policy.
    pub typeset_preflight_workers: NonZeroUsize,
    /// The ffmpeg process capability for [`RenderFormat::Mp4`] and
    /// [`RenderFormat::Mov`]. Native formats never use it. When unset,
    /// [`render`] supplies `FfmpegCapability::host()` if the `ffmpeg` feature
    /// is enabled; every other entry point refuses video by name.
    pub ffmpeg: Option<FfmpegCapability>,
    /// Explicit bounds for each render-time ffmpeg invocation. The default
    /// timeout is 24 hours because encoding spans scene construction and frame
    /// production; log/artifact defaults remain 1 MiB per stream and 8 GiB.
    /// Set a smaller timeout for service workloads or a larger one for long
    /// offline jobs. Zero/unrepresentable bounds are refused, not unlimited.
    /// The artifact cap is also bounded by [`Self::max_output_bytes`]. Native
    /// formats ignore this policy; tool probes retain their own short bounds.
    pub ffmpeg_limits: JobLimits,
    /// Opt-in CRF, preset, tune, or bitrate for opaque MP4/MOV. The default
    /// inherits optional `config.file_writer.video_*` controls; absent config
    /// keys preserve existing encoder behavior. Explicit fields override the
    /// corresponding configured fields, and selecting CRF or bitrate replaces
    /// the other configured rate-control mode. Settings
    /// incompatible with the selected codec, alpha MOV, or native formats are
    /// refused before tool discovery rather than dropped. Never changes raw
    /// frame rendering, frame sampling, or certified artifact semantics.
    pub video_quality: VideoQuality,
}

impl RenderOptions {
    /// PNG-sequence export using the bundled defaults and explicit budgets.
    ///
    /// # Errors
    /// Returns a configuration error if the bundled defaults cannot be resolved.
    pub fn new(output: impl Into<PathBuf>) -> Result<Self, fmn_config::ConfigError> {
        Ok(Self {
            output: output.into(),
            format: RenderFormat::PngSequence,
            config: Config::resolve(&[], None)?.config,
            camera: None,
            max_frames: 1_000_000,
            max_output_bytes: 64 * 1024 * 1024 * 1024,
            max_resident_bytes: 512 * 1024 * 1024,
            plan_limits: RenderPlanLimits::default(),
            frames_in_flight: 2,
            typeset_cache: true,
            typeset_preflight_workers: crate::typesetting::DEFAULT_PREFLIGHT_WORKERS,
            ffmpeg: None,
            ffmpeg_limits: ffmpeg_limits::default_job_limits(),
            video_quality: VideoQuality::default(),
        })
    }

    /// The same defaults with an explicit output format.
    ///
    /// # Errors
    /// Returns a configuration error if the bundled defaults cannot be resolved.
    pub fn with_format(
        output: impl Into<PathBuf>,
        format: RenderFormat,
    ) -> Result<Self, fmn_config::ConfigError> {
        let mut options = Self::new(output)?;
        options.format = format;
        Ok(options)
    }

    /// Resolve configuration and explicit video overrides without tool access.
    /// Encoder/container compatibility is checked by the ordinary video job.
    pub fn effective_video_quality(&self) -> Result<VideoQuality, RenderError> {
        let mut writer = self.config.file_writer.clone();
        let explicit = self.video_quality;
        if explicit.crf.is_some() || explicit.bitrate.is_some() {
            // An explicit rate mode replaces the configured one. Preserve
            // two explicit modes so ordinary negotiation rejects the conflict.
            writer.video_crf = explicit.crf.map(u32::from);
            writer.video_bitrate = explicit.bitrate;
        }
        if let Some(preset) = explicit.preset {
            writer.video_preset = Some(preset.as_str().to_owned());
        }
        if let Some(tune) = explicit.tune {
            writer.video_tune = Some(tune.as_str().to_owned());
        }
        configured_video_quality(&writer).map_err(RenderError::Negotiation)
    }

    /// Build a perspective camera matching the current export configuration.
    ///
    /// Customize its frame orientation, center, field of view, or light, then
    /// assign it to [`Self::camera`]. Configure resolution/fps/background before
    /// calling this helper; a later mismatch is rejected rather than silently
    /// resizing or recoloring output. This is a fixed camera for the render;
    /// ordinary scene animations still move the captured mobjects.
    ///
    /// # Errors
    /// Refuses invalid frame geometry or a malformed background color.
    pub fn camera_config(&self) -> Result<CameraConfig, RenderError> {
        let (width, height) = self.config.camera.resolution;
        if width == 0 || height == 0 {
            return Err(RenderError::InvalidOptions(
                "camera resolution must be nonzero",
            ));
        }
        let frame_height = self.config.sizes.frame_height;
        let mut frame = CameraFrame::default();
        frame
            .set_shape([
                frame_height * f64::from(width) / f64::from(height),
                frame_height,
            ])
            .map_err(RenderError::Camera)?;
        Ok(CameraConfig {
            resolution: (width, height),
            fps: self.config.camera.fps,
            background: Srgb::from_hex(&self.config.camera.background_color)
                .map_err(RenderError::Color)?
                .to_linear(self.config.camera.background_opacity),
            frame,
            ..CameraConfig::default()
        })
    }

    fn typesetting_session(&self, fs: Arc<dyn FileSystem>) -> TexSession {
        if self.typeset_cache {
            TexSession::with_cache(&self.config, fs, Arc::new(StdClock::new()))
        } else {
            TexSession::memory(self.config.tex.template.clone())
        }
    }
}

/// Successful execution and publication, including the actual CPU plan.
#[derive(Debug, Clone)]
pub struct RenderReport {
    /// Exact scene time and play count from Proscenium.
    pub scene: SceneRunReport,
    /// Published destination, content digest, byte count, frame count, and
    /// for video the ffmpeg provenance and muxed soundtrack.
    pub artifact: RenderArtifact,
    /// Final output queue and sink counters after the worker has joined.
    pub emission: EmitterReport,
    /// The scheduler decision used for this render.
    pub execution_plan: ExecutionPlan,
    /// Final joined frame-pipeline counters for affine and camera captures.
    pub frame_pipeline: Option<PipelineStats>,
    /// Actual typesetting/cache work, separate from certified artifact identity.
    /// Code-free bundle replay has no typesetting session and reports zero work.
    pub typesetting: TypesetSessionReport,
}

/// Typed failure of native scene rendering; underlying sources are retained.
#[derive(Debug)]
pub enum RenderError {
    /// A compiled FMTL input refused schema, engine or snapshot validation.
    Bundle(fmn_scene::timeline_bundle::BundleReadError),
    /// Invalid bounds or a configuration combination this entry point cannot use.
    InvalidOptions(&'static str),
    /// A requested backend is not available through this entry point.
    Capability(&'static str),
    /// The configured background color is not valid hexadecimal RGB.
    Color(HexParseError),
    /// Invalid camera pose, geometry, or lighting.
    Camera(CameraError),
    /// Invalid runtime configuration or a scene-authored error.
    Scene(crate::Error),
    /// Topology/scheduling request was refused.
    Plan(PlanError),
    /// Frame layout or conversion failed.
    Frame(FrameError),
    /// The retained renderer refused a frame.
    Renderer(RetainedFrameRendererError),
    /// Worker-side pipeline failure, preserving stage and final counters.
    Pipeline(FrameStreamError<NativeFrameError>),
    /// Native sink creation failed.
    Sink(SinkAdapterError),
    /// A frame could not enter the bounded output queue.
    Emitter(EmitterError),
    /// Output drain/finalization failed; includes final delivery counters.
    Drain(EmitterFailure),
    /// A finalized sink did not supply its artifact receipt.
    Receipt(ReceiptError),
    /// The configured ffmpeg executable could not be located (D2). Native
    /// PNG-sequence, GIF and Y4M outputs need no external tool.
    FfmpegUnavailable(FfmpegLocatorError),
    /// The ffmpeg boundary refused its tool, probe, or job.
    Ffmpeg(BoundaryError),
    /// The installed ffmpeg does not offer the negotiated encoder.
    EncoderUnavailable(String),
    /// The requested video profile cannot be negotiated.
    Negotiation(NegotiationError),
    /// A sound cue's asset could not be read.
    SoundAsset {
        /// The authored `add_sound` path.
        path: PathBuf,
        /// The filesystem refusal.
        error: FsError,
    },
    /// A sound cue's asset could not be decoded.
    SoundDecode {
        /// The authored `add_sound` path.
        path: PathBuf,
        /// The decoder refusal.
        error: AudioDecodeError,
    },
    /// The native mixer refused a cue or its timeline budget.
    Sound(SoundError),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOptions(message) | Self::Capability(message) => f.write_str(message),
            Self::Bundle(error) => error.fmt(f),
            Self::Color(error) => error.fmt(f),
            Self::Camera(error) => error.fmt(f),
            Self::Scene(error) => error.fmt(f),
            Self::Plan(error) => error.fmt(f),
            Self::Frame(error) => error.fmt(f),
            Self::Renderer(RetainedFrameRendererError::CameraRequired { program }) => write!(
                f,
                "{program:?} content needs the camera route: set RenderOptions::camera \
                 (for example `options.camera = Some(options.camera_config()?)`) or render \
                 with fmn::render_camera"
            ),
            Self::Renderer(error) => error.fmt(f),
            Self::Pipeline(error) => error.fmt(f),
            Self::Sink(error) => error.fmt(f),
            Self::Emitter(error) => error.fmt(f),
            Self::Drain(error) => error.fmt(f),
            Self::Receipt(error) => error.fmt(f),
            Self::FfmpegUnavailable(error) => write!(
                f,
                "ffmpeg is unavailable: {error}; {}",
                fmn_output::NATIVE_ALTERNATIVE
            ),
            Self::Ffmpeg(error) => write!(f, "ffmpeg boundary: {error}"),
            Self::EncoderUnavailable(encoder) => write!(
                f,
                "installed ffmpeg does not offer encoder {encoder:?}; {}",
                fmn_output::NATIVE_ALTERNATIVE
            ),
            Self::Negotiation(error) => error.fmt(f),
            Self::SoundAsset { path, error } => {
                write!(f, "sound cue {}: {error}", path.display())
            }
            Self::SoundDecode { path, error } => {
                write!(f, "sound cue {}: {error}", path.display())
            }
            Self::Sound(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for RenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidOptions(_) | Self::Capability(_) => None,
            Self::Bundle(error) => Some(error),
            Self::Color(error) => Some(error),
            Self::Camera(error) => Some(error),
            Self::Scene(error) => Some(error),
            Self::Plan(error) => Some(error),
            Self::Frame(error) => Some(error),
            Self::Renderer(error) => Some(error),
            Self::Pipeline(error) => Some(error),
            Self::Sink(error) => Some(error),
            Self::Emitter(error) => Some(error),
            Self::Drain(error) => Some(error),
            Self::Receipt(error) => Some(error),
            Self::FfmpegUnavailable(error) => Some(error),
            Self::Ffmpeg(error) => Some(error),
            Self::EncoderUnavailable(_) => None,
            Self::Negotiation(error) => Some(error),
            Self::SoundAsset { error, .. } => Some(error),
            Self::SoundDecode { error, .. } => Some(error),
            Self::Sound(error) => Some(error),
        }
    }
}

/// Render one native scene and atomically publish the requested artifact.
///
/// Static scenes with no explicit capture receive one final-state frame.
/// Animated scenes retain exactly the runtime's capture samples; no alpha-zero
/// or extra terminal frame is inserted. Sound cues (`Scene::add_sound`) are
/// mixed natively and muxed into MP4/MOV video; image and Y4M outputs carry no
/// audio.
///
/// Video formats use [`RenderOptions::ffmpeg`] or, when it is unset and the
/// `ffmpeg` feature is enabled, the host's ffmpeg capability.
///
/// # Errors
/// Returns [`RenderError`] without publishing a successful partial artifact.
pub fn render<P: SceneConstruct + ?Sized>(
    program: &mut P,
    #[cfg_attr(not(feature = "ffmpeg"), allow(unused_mut))] mut options: RenderOptions,
) -> Result<RenderReport, RenderError> {
    #[cfg(feature = "ffmpeg")]
    if options.format.is_video() && options.ffmpeg.is_none() {
        options.ffmpeg = Some(FfmpegCapability::host());
    }
    render_with_fs(program, options, Arc::new(StdFs))
}

/// The same rendering path with an explicit filesystem capability, useful for
/// sandboxed hosts and tests. No asset fetcher or process runner is implicit:
/// video requires an explicit [`RenderOptions::ffmpeg`].
///
/// # Errors
/// Returns [`RenderError`] from scene execution, rasterization or publication.
pub fn render_with_fs<P: SceneConstruct + ?Sized>(
    program: &mut P,
    options: RenderOptions,
    fs: Arc<dyn FileSystem>,
) -> Result<RenderReport, RenderError> {
    let runtime = RuntimeConfig::from_config(&options.config);
    // Reject bad timing before constructing a sink or touching its destination.
    if runtime.fps == 0 {
        return Err(RenderError::InvalidOptions("fps must be nonzero"));
    }
    if !runtime.default_wait_time.is_finite() || runtime.default_wait_time < 0.0 {
        return Err(RenderError::InvalidOptions(
            "default_wait_time must be finite and non-negative",
        ));
    }
    let seed = options.config.determinism.seed;
    let typesetting = options.typesetting_session(Arc::clone(&fs));
    let preflight_workers = options.typeset_preflight_workers;
    let mut sink = RenderSink::new(options, fs)?;
    let completed = crate::run_scene_with_typesetting(
        program,
        runtime,
        seed,
        &mut sink,
        &typesetting,
        preflight_workers,
    );
    sink.surface_failure()?;
    let completed = completed.map_err(RenderError::Scene)?;
    if sink.next_sequence == 0 {
        sink.render_stage(completed.scene().stage())?;
    }
    let finished = sink.finish(Some(completed.scene()))?;
    Ok(RenderReport {
        scene: *completed.report(),
        artifact: finished.artifact,
        emission: finished.emission,
        execution_plan: sink.plan.clone(),
        frame_pipeline: finished.frame_pipeline,
        typesetting: completed.typesetting_report().clone(),
    })
}

/// The joined pipeline, drained output and published artifact of one render.
struct FinishedRender {
    frame_pipeline: Option<PipelineStats>,
    emission: EmitterReport,
    artifact: RenderArtifact,
}

/// Where the sink's publication receipt comes from.
enum RenderReceipt {
    Native(SinkReceipt<NativeArtifactReport>),
    Video(SinkReceipt<FfmpegArtifactReport>),
}

/// The live ffmpeg session of a video render: the deferred soundtrack slot
/// and the same governed capability for decoding non-WAV sound cues.
struct VideoSession {
    soundtrack: Option<FfmpegSoundtrack>,
    tool: FfmpegTool,
    runner: Arc<dyn ProcessRunner>,
    workdir_root: PathBuf,
    fs: Arc<dyn FileSystem>,
    job_limits: JobLimits,
    max_input_bytes: u64,
    video_quality: VideoQuality,
    timeline: SoundtrackTimeline,
    audio_decodes: Vec<InvocationReport>,
}

struct RenderSink {
    pipeline: Option<NativeFramePipeline>,
    emitter: Option<OrderedEmitter>,
    receipt: RenderReceipt,
    format: RenderFormat,
    video: Option<VideoSession>,
    plan: ExecutionPlan,
    next_sequence: u64,
    max_frames: u64,
    failure: Option<RenderError>,
}

impl RenderSink {
    fn new(options: RenderOptions, fs: Arc<dyn FileSystem>) -> Result<Self, RenderError> {
        let config = &options.config;
        if config.render.engine != Engine::Cpu {
            return Err(RenderError::Capability(
                "native render() supports render.engine=cpu; no accelerator is silently substituted",
            ));
        }
        if !config.sizes.frame_height.is_finite() || config.sizes.frame_height <= 0.0 {
            return Err(RenderError::InvalidOptions(
                "frame height must be finite and positive",
            ));
        }
        if !config.camera.background_opacity.is_finite()
            || !(0.0..=1.0).contains(&config.camera.background_opacity)
        {
            return Err(RenderError::InvalidOptions(
                "background opacity must be in [0, 1]",
            ));
        }
        if options.frames_in_flight == 0 {
            return Err(RenderError::InvalidOptions(
                "frames_in_flight must be nonzero",
            ));
        }
        if !options.format.is_video() && !options.video_quality.is_default() {
            return Err(RenderError::InvalidOptions(
                "video_quality requires an opaque MP4/MOV output",
            ));
        }
        let limits = SinkLimits::new(
            options.max_frames,
            options.max_resident_bytes,
            options.max_output_bytes,
            options.max_output_bytes,
        )
        .map_err(RenderError::Sink)?;
        let (width, height) = config.camera.resolution;
        let certified = config.determinism.mode == DeterminismMode::Certified;
        // Negotiate before touching topology, the destination or a process.
        let video_job = if options.format.is_video() {
            if certified {
                return Err(RenderError::Capability(
                    "ffmpeg video is outside the certified artifact set; use RenderFormat::PngSequence or RenderFormat::Y4m with certified determinism",
                ));
            }
            // Invalid profiles have the same refusal with or without a host
            // ffmpeg capability. Admission itself needs no process access.
            let job = negotiate_video(options.format, config, options.camera.as_ref())?
                .with_quality(options.effective_video_quality()?)
                .map_err(RenderError::Negotiation)?;
            if options.ffmpeg.is_none() {
                return Err(RenderError::Capability(
                    "video output requires RenderOptions::ffmpeg (or render() with the `ffmpeg` feature); native PNG-sequence, GIF and Y4M outputs need no external tool",
                ));
            }
            Some(job)
        } else {
            None
        };
        let job_limits = options
            .format
            .is_video()
            .then(|| options.effective_ffmpeg_limits())
            .transpose()?;
        let pixel_format = match (&video_job, options.format) {
            (Some(job), _) => job.wire.frame_format(),
            (None, RenderFormat::Y4m) => PixelFormat::Nv12,
            (None, _) => PixelFormat::Rgba8,
        };
        let layout = FrameLayout::tight(pixel_format, width, height).map_err(RenderError::Frame)?;
        let logical = std::thread::available_parallelism()
            .ok()
            .and_then(|count| u32::try_from(count.get()).ok())
            .unwrap_or(1);
        let topology = if cfg!(target_os = "linux") {
            HardwareTopology::detect_linux(fs.as_ref())
                .unwrap_or_else(|_| HardwareTopology::fallback(logical))
        } else {
            HardwareTopology::fallback(logical)
        };
        // Output conversion writes straight into the reserved ring slot; no
        // per-stage intermediate surface needs planning.
        let surface = SurfaceSpec::lumen(width, height);
        let output_format = match pixel_format {
            PixelFormat::Bgra8 => OutputPixelFormat::Bgra8,
            PixelFormat::Nv12 => OutputPixelFormat::Nv12,
            PixelFormat::P010 => OutputPixelFormat::P010,
            PixelFormat::Rgba8 | PixelFormat::Rgba16F => OutputPixelFormat::Rgba8,
        };
        let mut request = if certified {
            PlanRequest::certified(RenderIntent::Offline, surface, output_format)
        } else {
            PlanRequest::standard(RenderIntent::Offline, surface, output_format)
        }
        .with_max_frames_in_flight(options.frames_in_flight);
        // ThreeDJob's camera path has one certified CPU implementation. Do not
        // report a fast/annex engine that did not actually produce the pixels.
        if options.camera.is_some() {
            request = request.with_engine(ExecutionEngine::CertifiedCpu);
        }
        if let ThreadPolicy::Fixed(threads) = config.render.threads {
            let threads = usize::try_from(threads)
                .map_err(|_| RenderError::InvalidOptions("thread count exceeds this target"))?;
            request = request.with_max_cpu_threads(threads);
        }
        let plan = ExecutionPlan::derive(request, &topology, None).map_err(RenderError::Plan)?;
        // Account for retained pixels separately from in-flight raw/output
        // surfaces. Each affine render team retains its own tile cache; camera
        // preparation retains one raw frame. Check before allocating anything.
        let retained_frames = if options.camera.is_some() {
            // Live-camera preparation plus the largest compiled-camera mode:
            // one retained raw preparation frame on each active render team.
            plan.render_teams
                .len()
                .checked_add(1)
                .ok_or(RenderError::InvalidOptions(
                    "retained frame count overflowed",
                ))?
        } else {
            plan.render_teams.len()
        };
        let retained_bytes = u64::try_from(retained_frames).ok().and_then(|count| {
            u64::from(width)
                .checked_mul(u64::from(height))?
                .checked_mul(8)?
                .checked_mul(count)
        });
        let planned = u64::try_from(plan.estimated_in_flight_bytes)
            .ok()
            .and_then(|bytes| bytes.checked_add(retained_bytes?))
            .and_then(|bytes| bytes.checked_add(64 * 1024 * 1024))
            .ok_or(RenderError::InvalidOptions(
                "render memory budget overflowed",
            ))?;
        if planned > options.max_resident_bytes {
            return Err(RenderError::InvalidOptions(
                "render plan exceeds max_resident_bytes",
            ));
        }
        let background = Srgb::from_hex(&config.camera.background_color)
            .map_err(RenderError::Color)?
            .to_linear(config.camera.background_opacity);
        let camera = options
            .camera
            .map(|camera| {
                if camera.resolution != (width, height)
                    || camera.fps != config.camera.fps
                    || camera.background != background
                {
                    return Err(RenderError::InvalidOptions(
                        "camera resolution, fps, and background must match the export configuration",
                    ));
                }
                Camera::new(camera).map_err(RenderError::Camera)
            })
            .transpose()?;
        let frame = FrameConfig::new(
            Viewport { width, height },
            ScreenMap {
                scale: f64::from(height) / config.sizes.frame_height,
                origin: [f64::from(width) / 2.0, f64::from(height) / 2.0],
                y_up: true,
            },
            background,
        )
        .with_aa_policy(config.render.aa);
        let renderer_config = RetainedFrameRendererConfig {
            frame,
            tiling: Tiling {
                macro_tile: plan.macro_tile,
                fine_tile: plan.fine_tile,
            },
            engine: if plan.engine == ExecutionEngine::CertifiedCpu {
                EngineIdentity::certified()
            } else {
                EngineIdentity::fast()
            },
            threads: plan
                .render_teams
                .first()
                .map_or(1, fmn_runtime::TeamPlan::threads),
        };
        let mut video = None;
        let (binding, receipt) = match options.format {
            RenderFormat::PngSequence => {
                let (binding, receipt) = PngSink::new(
                    fs,
                    PngSinkConfig {
                        target: PngTarget::Sequence {
                            directory: options.output,
                            stem: "frame".to_owned(),
                            digits: 6,
                        },
                        width,
                        height,
                        first_sequence: 0,
                        compression: if certified {
                            CompressionLevel::Best
                        } else {
                            CompressionLevel::Default
                        },
                        threads: plan.encoder_threads().max(1),
                        limits,
                        profile: None,
                    },
                )
                .map_err(RenderError::Sink)?
                .with_no_clobber()
                .into_binding("png-sequence");
                (binding, RenderReceipt::Native(receipt))
            }
            RenderFormat::Gif => {
                let (binding, receipt) = GifSink::new(
                    fs,
                    GifSinkConfig {
                        destination: options.output,
                        width,
                        height,
                        fps: (config.camera.fps, 1),
                        loop_forever: true,
                        first_sequence: 0,
                        limits,
                        profile: None,
                    },
                )
                .map_err(RenderError::Sink)?
                .with_no_clobber()
                .into_binding("gif");
                (binding, RenderReceipt::Native(receipt))
            }
            RenderFormat::Y4m => {
                let (binding, receipt) = Y4mSink::new(
                    fs,
                    Y4mSinkConfig {
                        destination: options.output,
                        width,
                        height,
                        fps: (config.camera.fps, 1),
                        colorspace: Y4mColorspace::C420Mpeg2,
                        first_sequence: 0,
                        limits,
                        profile: None,
                    },
                )
                .map_err(RenderError::Sink)?
                .with_no_clobber()
                .into_binding("y4m");
                (binding, RenderReceipt::Native(receipt))
            }
            RenderFormat::Mp4 | RenderFormat::Mov => {
                let job = video_job.ok_or(RenderError::InvalidOptions(
                    "video format reached the sink without a negotiated job",
                ))?;
                let video_quality = job.quality().map_err(RenderError::Negotiation)?;
                let job_limits = job_limits.ok_or(RenderError::InvalidOptions(
                    "video format reached the sink without admitted ffmpeg limits",
                ))?;
                let capability = options.ffmpeg.ok_or(RenderError::Capability(
                    "video output requires an ffmpeg capability",
                ))?;
                let encoder = job
                    .resolved_encoder()
                    .map_err(RenderError::Negotiation)?
                    .ok_or(RenderError::InvalidOptions("video requires an encoder"))?;
                let executable = capability
                    .locator
                    .locate_ffmpeg(Path::new(&config.file_writer.ffmpeg_bin))
                    .map_err(RenderError::FfmpegUnavailable)?;
                let tool = FfmpegTool::resolve(
                    executable,
                    capability.runner.as_ref(),
                    &capability.workdir_root,
                )
                .map_err(RenderError::Ffmpeg)?;
                let capabilities = EncoderCapabilities::probe(&tool, capability.runner.as_ref())
                    .map_err(RenderError::Ffmpeg)?;
                if !capabilities.offers(&encoder) {
                    return Err(RenderError::EncoderUnavailable(encoder));
                }
                let (sink, soundtrack) = FfmpegSink::new(
                    Arc::clone(&capability.runner),
                    FfmpegSinkConfig {
                        tool: tool.clone(),
                        capabilities,
                        job,
                        audio: None,
                        destination: options.output,
                        workdir_root: capability.workdir_root.clone(),
                        job_limits: job_limits.clone(),
                        first_sequence: 0,
                        limits,
                        profile: None,
                    },
                )
                .map_err(RenderError::Sink)?
                .with_no_clobber()
                .with_deferred_soundtrack()
                .map_err(RenderError::Sink)?;
                video = Some(VideoSession {
                    soundtrack: Some(soundtrack),
                    tool,
                    runner: capability.runner,
                    workdir_root: capability.workdir_root,
                    fs: Arc::clone(&fs),
                    job_limits,
                    max_input_bytes: options.max_output_bytes,
                    video_quality,
                    timeline: SoundtrackTimeline::default(),
                    audio_decodes: Vec::new(),
                });
                let (binding, receipt) = sink.into_binding("ffmpeg-video");
                (binding, RenderReceipt::Video(receipt))
            }
        };
        let emitter = OrderedEmitter::new(
            EmitterConfig::new(layout, plan.frames_in_flight, 0).map_err(RenderError::Emitter)?,
            vec![binding],
        )
        .map_err(RenderError::Emitter)?;
        let pipeline = match NativeFramePipeline::with_plan_limits(
            plan.clone(),
            renderer_config,
            camera,
            emitter.handle(),
            options.plan_limits,
        ) {
            Ok(pipeline) => pipeline,
            Err(error) => {
                emitter.cancel();
                let _ = emitter.finish();
                return Err(error);
            }
        };
        Ok(Self {
            pipeline: Some(pipeline),
            emitter: Some(emitter),
            receipt,
            format: options.format,
            video,
            plan,
            next_sequence: 0,
            max_frames: options.max_frames,
            failure: None,
        })
    }

    fn render_stage(&mut self, stage: &fmn_mobject::Stage) -> Result<(), RenderError> {
        if self.next_sequence >= self.max_frames {
            return Err(RenderError::InvalidOptions("scene exceeded max_frames"));
        }
        self.pipeline
            .as_mut()
            .ok_or(RenderError::Pipeline(FrameStreamError::Closed))?
            .capture(stage, self.next_sequence)?;
        self.next_sequence += 1;
        Ok(())
    }

    /// Return a failure recorded during capture, after joining the workers
    /// whose closed stream may be hiding the original stage error.
    fn surface_failure(&mut self) -> Result<(), RenderError> {
        let Some(error) = self.failure.take() else {
            return Ok(());
        };
        // A closed admission stream can hide the original worker failure.
        // Wake output waiters before joining raster/conversion workers.
        if matches!(&error, RenderError::Pipeline(_)) {
            if let Some(emitter) = &self.emitter {
                emitter.cancel();
            }
            if let Some(pipeline) = self.pipeline.take() {
                pipeline.finish()?;
            }
        }
        Err(error)
    }

    /// Join every frame stage, supply a video's soundtrack, drain the output
    /// and take the publication receipt — in that order. `scene` provides the
    /// sound cues; compiled replay has none.
    fn finish(&mut self, scene: Option<&fmn_scene::Scene>) -> Result<FinishedRender, RenderError> {
        // Joining all raster/conversion work is a prerequisite to publication.
        let frame_pipeline = self
            .pipeline
            .take()
            .map(NativeFramePipeline::finish)
            .transpose()?;
        let soundtrack = match self.video.as_mut() {
            Some(video) => {
                let mix = match scene {
                    Some(scene) => video.mix(
                        scene,
                        self.next_sequence,
                        self.plan.output_team.threads().max(1),
                    )?,
                    None => None,
                };
                let report = mix.as_ref().map(|mix| SoundtrackReport {
                    cues_mixed: mix.cues_mixed,
                    sample_frames: (mix.audio.samples.len()
                        / usize::from(mix.audio.channels.max(1)))
                        as u64,
                    sample_rate: mix.audio.sample_rate,
                    channels: mix.audio.channels,
                    clipped_samples: mix.clipped_samples,
                });
                video
                    .soundtrack
                    .take()
                    .ok_or(RenderError::InvalidOptions(
                        "video soundtrack was already supplied",
                    ))?
                    .finish(mix)
                    .map_err(RenderError::Sink)?;
                report
            }
            None => None,
        };
        let emission = self
            .emitter
            .take()
            .ok_or(RenderError::InvalidOptions(
                "render emitter was already finalized",
            ))?
            .finish()
            .map_err(RenderError::Drain)?;
        let mut artifact = match &self.receipt {
            RenderReceipt::Native(receipt) => {
                RenderArtifact::native(self.format, receipt.take().map_err(RenderError::Receipt)?)?
            }
            RenderReceipt::Video(receipt) => RenderArtifact::video(
                self.format,
                receipt.take().map_err(RenderError::Receipt)?,
                soundtrack,
            ),
        };
        artifact.ffmpeg_limits = self
            .video
            .as_ref()
            .map(|video| FfmpegLimitsReport::new(&video.job_limits, video.max_input_bytes));
        artifact.video_quality = self.video.as_ref().map(|video| video.video_quality);
        if let Some(video) = self.video.as_mut() {
            artifact.audio_decodes = std::mem::take(&mut video.audio_decodes);
        }
        Ok(FinishedRender {
            frame_pipeline,
            emission,
            artifact,
        })
    }
}

impl VideoSession {
    /// Mix the scene's sound cues on the exact rational timeline, spanning at
    /// least the published picture. `None` when the scene authored no sound.
    fn mix(
        &mut self,
        scene: &fmn_scene::Scene,
        frames_published: u64,
        threads: usize,
    ) -> Result<Option<MixReport>, RenderError> {
        let requests = scene.sound_requests();
        if requests.is_empty() {
            return Ok(None);
        }
        let config = MixerConfig::default();
        let time = scene.time();
        let picture_frames = self.timeline.output_frame(time.frames())?.max(
            i64::try_from(frames_published)
                .map_err(|_| RenderError::InvalidOptions("published frame count exceeds i64"))?,
        );
        let timeline = frames_to_samples(picture_frames, time.fps(), config.sample_rate)
            .map_err(RenderError::Sound)?;
        let timeline = u64::try_from(timeline)
            .map_err(|_| RenderError::InvalidOptions("soundtrack timeline is negative"))?;
        // Non-WAV cues transcode through the same governed capability that
        // encodes the picture; native PCM WAV never reaches a process.
        let boundary = Boundary::new(
            self.tool.clone(),
            Arc::clone(&self.runner),
            self.job_limits.clone(),
            self.workdir_root.clone(),
        )
        .map_err(RenderError::Ffmpeg)?;
        let mut decoder = AudioDecoder::with_boundary(AudioDecodeLimits::default(), boundary)
            .map_err(|error| RenderError::SoundDecode {
                path: PathBuf::new(),
                error,
            })?;
        let prepared = PreparedSceneSoundtrack::new(
            self.fs.as_ref(),
            requests,
            &mut decoder,
            config,
            Some(&self.timeline),
        )?
        .with_timeline_frames(timeline);
        let mixed = prepared.mix(threads)?;
        self.audio_decodes = prepared.invocations();
        Ok(Some(mixed))
    }
}

/// Negotiate the ffmpeg job from the export configuration, exactly as the
/// CLI and portal do: no ffmpeg filters, an alpha wire only where alpha is
/// meaningful, and every incompatible combination refused before any process
/// or destination is touched.
fn negotiate_video(
    format: RenderFormat,
    config: &Config,
    camera: Option<&CameraConfig>,
) -> Result<VideoJob, RenderError> {
    // Any represented non-identity value is semantically observable; an
    // epsilon check would silently discard a requested color transform.
    if config.file_writer.saturation != 1.0 || config.file_writer.gamma != 1.0 {
        return Err(RenderError::Capability(
            "non-default file_writer saturation/gamma require a native color-transform stage; ffmpeg filters are forbidden",
        ));
    }
    let (width, height) = config.camera.resolution;
    let transparent = format == RenderFormat::Mov
        && camera.map_or(config.camera.background_opacity < 1.0, |camera| {
            camera.background.a < 1.0
        });
    if format == RenderFormat::Mp4 && config.camera.background_opacity < 1.0 {
        return Err(RenderError::InvalidOptions(
            "a translucent background requires alpha-preserving RenderFormat::Mov",
        ));
    }
    let wire = match config
        .file_writer
        .pixel_format
        .to_ascii_lowercase()
        .as_str()
    {
        "yuv420p" if transparent => WireFormat::Rgba8,
        "rgba" | "rgba8" => WireFormat::Rgba8,
        "bgra" | "bgra8" => WireFormat::Bgra8,
        "nv12" | "yuv420p" => WireFormat::Nv12,
        "p010" | "p010le" | "yuv420p10le" => WireFormat::P010,
        _ => {
            return Err(RenderError::InvalidOptions(
                "unsupported file_writer.pixel_format; choose rgba, bgra, nv12/yuv420p, or p010le",
            ));
        }
    };
    if transparent && !wire.has_alpha() {
        return Err(RenderError::InvalidOptions(
            "transparent video requires an rgba or bgra file_writer.pixel_format",
        ));
    }
    let codec = config.file_writer.video_codec.trim();
    if codec.is_empty()
        || !codec
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(RenderError::InvalidOptions(
            "file_writer.video_codec must be a nonempty encoder name (letters, digits, underscores)",
        ));
    }
    let encoder = if codec.eq_ignore_ascii_case("auto") || (transparent && codec == "libx264") {
        EncoderChoice::Auto
    } else {
        EncoderChoice::Named(codec.to_owned())
    };
    let job = VideoJob {
        width,
        height,
        fps: (config.camera.fps, 1),
        wire,
        color: if wire.has_alpha() {
            ColorDescription::srgb_full()
        } else {
            ColorDescription::video_bt709()
        },
        container: match (format, transparent) {
            (_, true) => Container::MovTransparent,
            (RenderFormat::Mov, false) => Container::Mov,
            _ => Container::Mp4,
        },
        encoder,
        crf: None,
    };
    let encoder = job.resolved_encoder().map_err(RenderError::Negotiation)?;
    if transparent && encoder.as_deref() != Some("qtrle") {
        return Err(RenderError::InvalidOptions(
            "transparent video currently requires the qtrle encoder",
        ));
    }
    // Both ordinary container paths encode 4:2:0, even over an RGBA wire.
    if !transparent && (!width.is_multiple_of(2) || !height.is_multiple_of(2)) {
        return Err(RenderError::InvalidOptions(
            "ordinary video requires even width and height for 4:2:0 output",
        ));
    }
    Ok(job)
}

impl SceneSink for RenderSink {
    fn event(&mut self, event: fmn_scene::LifecycleEvent) -> Result<(), IntegrationError> {
        if let Some(error) = &self.failure {
            return Err(IntegrationError::new("native-render", error.to_string()));
        }
        if let Some(video) = self.video.as_mut() {
            video
                .timeline
                .observe(event.time.frames(), event.skipping)
                .map_err(|error| {
                    let message = error.to_string();
                    self.failure = Some(error);
                    IntegrationError::new("native-render", message)
                })?;
        }
        Ok(())
    }

    fn capture(
        &mut self,
        _reason: CaptureReason,
        packet: FramePacket,
    ) -> Result<(), IntegrationError> {
        if let Some(error) = &self.failure {
            return Err(IntegrationError::new("native-render", error.to_string()));
        }
        let stage = packet.materialize_stage();
        self.render_stage(&stage).map_err(|error| {
            let message = error.to_string();
            self.failure = Some(error);
            IntegrationError::new("native-render", message)
        })
    }
}

impl Drop for RenderSink {
    fn drop(&mut self) {
        // Wake blocked output operations BEFORE waiting for raster workers.
        if let Some(emitter) = &self.emitter {
            emitter.cancel();
        }
        drop(self.pipeline.take());
        // Join output abort cleanup before returning, including on scene panic.
        if let Some(emitter) = self.emitter.take() {
            let _ = emitter.finish();
        }
    }
}
