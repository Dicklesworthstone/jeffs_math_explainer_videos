//! The standard-only Metal annex executor.
//!
//! This module is the production descendant of G0-8 and G0-8b. It derives a
//! packed, single-typed device layouts from the same prepared [`FrameJob`] and
//! [`ThreeDJob`] the CPU engines consume, dispatches only through frankentorch's
//! safe generic gateway, and keeps its output surfaces alive across frames. The
//! semantic front-end, painter-ordered CSR command runs, fill-before-stroke
//! rule, camera clipping, and affine preparation therefore have one authority.
//!
//! The annex never participates in `certified`. Its public preview composition
//! root reports whether Metal actually produced a frame or whether the declared
//! fast-CPU fallback did; CPU bytes are never published under a Metal identity.

use std::fmt;
use std::time::{Duration, Instant};

use fmn_frame::{ChromaSiting, ColorRange, FrameBuffer, FrameError, FrameLayout, PixelFormat};
use fmn_hash::{Digest, Schema, Writer, sha256};
use fmn_mobject::JointType;
use ft_kernel_metal::Error as GatewayError;
use ft_kernel_metal::compute::{Gateway, Grid, MathMode, Pipeline, SharedBuffer};
use ft_kernel_metal::presentation::{NativePresenter, PresentationConfig};
pub use ft_kernel_metal::presentation::{
    PresentOutcome, PresentationError, PresentationPipelineInfo, PresentationState,
};

use crate::engine::{AaPolicy, Draw, EngineIdentity, FrameConfig, FrameJob, FrameJobError};
use crate::fill::MonoTable;
use crate::plan::RenderPlan;
use crate::three_d::{CompiledPrimitive, Shader, ThreeDJob};
use crate::{Binning, Segment};

const KERNEL_SOURCE: &str = include_str!("shaders/metal.metal");
const RASTER_KERNEL: &str = "fmn_render_frame";
const THREE_D_KERNEL: &str = "fmn_render_three_d";
const RGBA8_KERNEL: &str = "fmn_rgba16f_to_rgba8";
const NV12_KERNEL: &str = "fmn_rgba16f_to_nv12";
const P010_KERNEL: &str = "fmn_rgba16f_to_p010";
const SEGMENT_ARC_INTERVALS: usize = 16;
const SEGMENT_ARC_VALUES: usize = SEGMENT_ARC_INTERVALS + 1;
const SEGMENT_STRIDE: usize = 8 + SEGMENT_ARC_VALUES + 1;
const PIECE_STRIDE: usize = 6;
const JOIN_STRIDE: usize = 13;
const STATION_STRIDE: usize = 5;
const DRAW_U32_STRIDE: usize = 10;
const DRAW_F32_STRIDE: usize = 8;
const STYLE_STRIDE: usize = 20;
const THREE_D_VERTEX_STRIDE: usize = 17;
const THREE_D_TRIANGLE_STRIDE: usize = THREE_D_VERTEX_STRIDE * 3;
const THREE_D_DRAW_U32_STRIDE: usize = 4;
const THREE_D_DRAW_F32_STRIDE: usize = 24;
const STATUS_COMPLETE: u32 = 0x464d_4e4d;
const TRANSFER_TILE: usize = 16;
const TRANSFER_TABLE_ENTRIES: usize = 1 << 16;
const TRANSFER_TABLE_WORDS_PER_PLANE: usize = TRANSFER_TABLE_ENTRIES / 4;
const TRANSFER_TABLE_WORDS: usize = TRANSFER_TABLE_WORDS_PER_PLANE * 2;

const DRAW_FILL: u32 = 1 << 0;
const DRAW_STROKE: u32 = 1 << 1;
const STROKE_BEHIND: u32 = 1 << 2;
const FLAT_FILL: u32 = 1 << 3;
const MULTI_CONTOUR_GRADIENT: u32 = 1 << 4;
const THREE_D_SHADER_GOURAUD: u32 = 1;
const THREE_D_SHADER_DOT: u32 = 2;

/// Canonical schema for the annex-specific half of C7.
pub const METAL_BACKEND_SCHEMA: Schema = Schema::new(*b"FMNM", 1, 0, 1);

/// Canonical schema for native Studio presentation provenance.
pub const NATIVE_PREVIEW_BACKEND_SCHEMA: Schema = Schema::new(*b"FMNP", 1, 0, 1);

/// Version-1 maximum linear-channel error for Metal versus certified.
///
/// The production three-frame corpus measures `0.1376953125` at the inner
/// border of one curved fill. This is the already-characterized `f32`
/// nearest-point conditioning residual from G0-8, not fill-area drift; the
/// blocking budget leaves roughly five percent headroom.
pub const METAL_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR: f64 = 0.145;

/// Version-1 RMS linear-channel error for Metal versus certified.
///
/// The production corpus maximum is `0.0020294770459577255`.
pub const METAL_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR: f64 = 0.002_14;

/// Version-1 minimum global sRGB-luma SSIM for Metal versus certified.
///
/// The production corpus minimum is `0.9998831140399521`; the bound applies
/// five percent headroom to perceptual distortion (`1 - SSIM`).
pub const METAL_VISUAL_BUDGET_V1_MIN_SSIM: f64 = 0.999_87;

/// Version-1 maximum linear-channel error for prepared 3D triangles.
///
/// This separately gates fixed-grid Gouraud surfaces and true/glow dots because
/// their perspective interpolation and radial falloff do not exercise the 2D
/// curve-distance residual measured above. The production corpus measures one
/// binary16 step (`0.00048828125`); the bound leaves five percent headroom.
pub const METAL_THREE_D_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR: f64 = 0.000_513;

/// Version-1 RMS linear-channel error for prepared 3D triangles.
///
/// The production corpus measures `0.0000013473872790351067`.
pub const METAL_THREE_D_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR: f64 = 0.000_001_42;

/// Version-1 minimum global sRGB-luma SSIM for prepared 3D triangles.
///
/// The production corpus measures `1.0`; the smoke-alarm threshold leaves one
/// part per million for cross-device standard-mode arithmetic.
pub const METAL_THREE_D_VISUAL_BUDGET_V1_MIN_SSIM: f64 = 0.999_999;

/// Maximum encoded-code error for the GPU RGBA16F-to-RGBA8 transfer.
///
/// Both implementations index the same authoritative Reel transfer tables by
/// raw binary16 bits, so this is exact rather than a visual-equivalence budget.
pub const METAL_RGBA8_TRANSFER_V1_MAX_CODE_ERROR: u8 = 0;

/// A Metal dispatch could not truthfully produce the requested frame.
#[derive(Debug)]
pub enum MetalError {
    /// The backend-neutral front-end rejected stale or unsupported inputs.
    FrameJob(FrameJobError),
    /// The requested frame layout was invalid.
    Frame(FrameError),
    /// The sanctioned frankentorch gateway refused or failed the operation.
    Gateway(GatewayError),
    /// A host-side count or byte calculation overflowed.
    SizeOverflow(&'static str),
    /// The selected fine tile exceeds the compiled pipeline's occupancy limit.
    ThreadgroupTooLarge {
        /// Pipeline entry point.
        kernel: &'static str,
        /// Requested threads.
        requested: usize,
        /// Introspected pipeline maximum.
        maximum: usize,
    },
    /// The gateway returned success but not every dispatched group completed.
    ///
    /// The pinned gateway's synchronous dispatch does not expose Metal command
    /// buffer status. Every kernel therefore writes a fresh per-group sentinel,
    /// and the wrapper fails closed when a runtime device error leaves any
    /// sentinel unwritten.
    IncompleteDispatch {
        /// Pipeline entry point.
        kernel: &'static str,
        /// Groups whose sentinel was present.
        completed: usize,
        /// Groups dispatched.
        expected: usize,
    },
    /// A backend-specific invariant drifted from its shader mirror.
    Layout(&'static str),
    /// Full record stroke profiles require the CPU renderer until the Metal
    /// annex carries an equivalent station table. Never flatten them silently.
    UnsupportedStrokeProfile,
    /// Full fill boundary colors currently require the analytic CPU field.
    UnsupportedFillProfile,
    /// The prepared 3D painter sequence contains a primitive not yet mapped by
    /// this annex pipeline.
    UnsupportedThreeDPrimitive {
        /// Painter-sequence index.
        draw: usize,
        /// Stable primitive description.
        primitive: &'static str,
    },
}

impl MetalError {
    fn unavailable(&self) -> bool {
        matches!(self, Self::Gateway(GatewayError::Unavailable))
    }

    fn permits_preview_fallback(&self) -> bool {
        matches!(
            self,
            Self::Gateway(_)
                | Self::ThreadgroupTooLarge { .. }
                | Self::IncompleteDispatch { .. }
                | Self::Layout(_)
        )
    }
}

impl fmt::Display for MetalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FrameJob(error) => error.fmt(f),
            Self::Frame(error) => error.fmt(f),
            Self::Gateway(error) => error.fmt(f),
            Self::SizeOverflow(what) => write!(f, "{what} exceeds this platform"),
            Self::ThreadgroupTooLarge {
                kernel,
                requested,
                maximum,
            } => write!(
                f,
                "{kernel} needs {requested} threads per threadgroup; the pipeline permits {maximum}"
            ),
            Self::IncompleteDispatch {
                kernel,
                completed,
                expected,
            } => write!(
                f,
                "{kernel} completed {completed} of {expected} threadgroups"
            ),
            Self::Layout(message) => write!(f, "Metal derived-layout mismatch: {message}"),
            Self::UnsupportedFillProfile => {
                f.write_str("Metal does not support retained fill profiles")
            }
            Self::UnsupportedStrokeProfile => {
                f.write_str("Metal does not support retained stroke profiles")
            }
            Self::UnsupportedThreeDPrimitive { draw, primitive } => write!(
                f,
                "Metal 3D draw {draw} uses unsupported prepared primitive {primitive}"
            ),
        }
    }
}

impl std::error::Error for MetalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::FrameJob(error) => Some(error),
            Self::Frame(error) => Some(error),
            Self::Gateway(error) => Some(error),
            Self::SizeOverflow(_)
            | Self::ThreadgroupTooLarge { .. }
            | Self::IncompleteDispatch { .. }
            | Self::Layout(_)
            | Self::UnsupportedThreeDPrimitive { .. }
            | Self::UnsupportedStrokeProfile
            | Self::UnsupportedFillProfile => None,
        }
    }
}

impl From<FrameJobError> for MetalError {
    fn from(error: FrameJobError) -> Self {
        Self::FrameJob(error)
    }
}

impl From<FrameError> for MetalError {
    fn from(error: FrameError) -> Self {
        Self::Frame(error)
    }
}

impl From<GatewayError> for MetalError {
    fn from(error: GatewayError) -> Self {
        Self::Gateway(error)
    }
}

/// One production annex dispatch's PG-A and provenance record.
#[derive(Debug, Clone)]
pub struct MetalReport {
    /// Engine identity carried by the frame.
    pub identity: EngineIdentity,
    /// Metal device name.
    pub device: String,
    /// Whether the selected device reports unified memory.
    pub unified_memory: bool,
    /// Safe math is permanent for the first production annex.
    pub math_mode: &'static str,
    /// Hash of the exact embedded MSL source.
    pub kernel_digest: Digest,
    /// Hash of the exact binary16-to-byte tables resident on the device.
    pub transfer_table_digest: Digest,
    /// Fine-tile threads used by the raster pipeline.
    pub threads_per_threadgroup: usize,
    /// Raster pipeline occupancy ceiling.
    pub max_threads_per_threadgroup: usize,
    /// Raster pipeline SIMD execution width.
    pub thread_execution_width: usize,
    /// Bytes materialized into new input buffers for this frame.
    pub upload_bytes: usize,
    /// Frame-pixel bytes copied to the host for the requested output.
    ///
    /// Status sentinels are control-plane validation, not frame pixels. Native
    /// presentation reports zero here because its lifetime-held RGBA8 surface
    /// remains GPU-visible through the drawable handoff.
    pub readback_bytes: usize,
    /// Whether the renderer reused its lifetime-held raw surface.
    pub raw_surface_reused: bool,
    /// Whether a converted output surface was reused.
    pub output_surface_reused: bool,
    /// Host-observed prepare/upload/dispatch/output-handoff wall time.
    pub elapsed: Duration,
    /// Format copied back to the host.
    pub output_format: PixelFormat,
    /// Quantization range for a planar YUV output.
    pub color_range: Option<ColorRange>,
    /// Chroma siting for a planar YUV output.
    pub chroma_siting: Option<ChromaSiting>,
}

impl MetalReport {
    /// Stable annex backend identity for C7.
    ///
    /// Timing and per-frame byte counts are observations, not identity, and are
    /// deliberately excluded.
    #[must_use]
    pub fn backend_journal(&self) -> Vec<u8> {
        let mut writer = Writer::new(METAL_BACKEND_SCHEMA);
        writer.put_str(self.identity.engine.name());
        writer.put_u32(self.identity.renderer_version);
        writer.put_str(&self.device);
        writer.put_bool(self.unified_memory);
        writer.put_str(self.math_mode);
        writer.put_bytes(self.kernel_digest.as_bytes());
        writer.put_u64(self.threads_per_threadgroup as u64);
        writer.put_u64(self.max_threads_per_threadgroup as u64);
        writer.put_u64(self.thread_execution_width as u64);
        writer.put_str(format_name(self.output_format));
        writer.put_bytes(self.transfer_table_digest.as_bytes());
        writer.put_str(color_range_name(self.color_range));
        writer.put_str(chroma_siting_name(self.chroma_siting));
        writer
            .finish()
            .expect("the fixed-size Metal backend identity fits the schema limits")
    }

    /// Digest of [`MetalReport::backend_journal`].
    #[must_use]
    pub fn backend_digest(&self) -> Digest {
        sha256(&self.backend_journal())
    }

    /// Measured whole-call frame rate, when the timer had nonzero resolution.
    #[must_use]
    pub fn frames_per_second(&self) -> Option<f64> {
        let seconds = self.elapsed.as_secs_f64();
        (seconds > 0.0).then_some(1.0 / seconds)
    }
}

/// A native Studio frame could not render or reach its drawable.
#[derive(Debug)]
pub enum NativePreviewError {
    /// Lumen could not produce the lifetime-held RGBA8 surface.
    Render(MetalError),
    /// Frankentorch could not create, update, or present the native surface.
    Presentation(PresentationError),
}

impl NativePreviewError {
    /// Declared CPU-stream fallback, when this failure is backend-local.
    ///
    /// Caller input, frame-layout, and size failures remain errors. An
    /// unsupported 3D primitive is a frame-local fallback; other admitted
    /// backend and presentation failures demote the native route.
    #[must_use]
    pub fn stream_fallback(&self) -> Option<PreviewFallback> {
        match self {
            Self::Render(MetalError::Gateway(GatewayError::Unavailable))
            | Self::Presentation(PresentationError::Unavailable) => {
                Some(PreviewFallback::Unavailable)
            }
            Self::Render(
                error @ (MetalError::UnsupportedThreeDPrimitive { .. }
                | MetalError::UnsupportedStrokeProfile
                | MetalError::UnsupportedFillProfile),
            ) => Some(PreviewFallback::Unsupported(error.to_string())),
            Self::Render(error) if error.permits_preview_fallback() => {
                Some(PreviewFallback::BackendFailure(error.to_string()))
            }
            Self::Presentation(
                error @ (PresentationError::WrongThread
                | PresentationError::Window(_)
                | PresentationError::Pipeline(_)
                | PresentationError::Closed
                | PresentationError::CommandBuffer(_)),
            ) => Some(PreviewFallback::BackendFailure(error.to_string())),
            Self::Render(_)
            | Self::Presentation(
                PresentationError::InvalidDimensions { .. }
                | PresentationError::InvalidStride { .. }
                | PresentationError::SizeOverflow
                | PresentationError::BufferTooSmall { .. },
            ) => None,
        }
    }

    /// Declared fallback during native-route construction.
    ///
    /// A missing Metal device and presentation-surface failures may select the
    /// CPU stream. Renderer shader or pipeline construction failures remain
    /// product errors instead of being hidden by fallback.
    #[must_use]
    pub fn construction_stream_fallback(&self) -> Option<PreviewFallback> {
        match self {
            Self::Render(MetalError::Gateway(GatewayError::Unavailable))
            | Self::Presentation(_) => self.stream_fallback(),
            Self::Render(_) => None,
        }
    }

    /// Whether this is a per-frame capability gap rather than a dead backend.
    #[must_use]
    pub fn is_frame_local_unsupported(&self) -> bool {
        matches!(
            self,
            Self::Render(
                MetalError::UnsupportedThreeDPrimitive { .. }
                    | MetalError::UnsupportedStrokeProfile
                    | MetalError::UnsupportedFillProfile
            )
        )
    }
}

impl fmt::Display for NativePreviewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Render(error) => error.fmt(f),
            Self::Presentation(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for NativePreviewError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Render(error) => Some(error),
            Self::Presentation(error) => Some(error),
        }
    }
}

impl From<MetalError> for NativePreviewError {
    fn from(error: MetalError) -> Self {
        Self::Render(error)
    }
}

impl From<PresentationError> for NativePreviewError {
    fn from(error: PresentationError) -> Self {
        Self::Presentation(error)
    }
}

/// PG-A evidence for one native Studio presentation attempt.
#[derive(Debug, Clone)]
pub struct NativePreviewReport {
    /// Lumen render and transfer facts; `readback_bytes` is always zero.
    pub metal: MetalReport,
    /// Occupancy facts for frankentorch's drawable transfer pipeline.
    pub presentation: PresentationPipelineInfo,
    /// Whether a drawable was presented or temporarily unavailable.
    pub outcome: PresentOutcome,
}

impl NativePreviewReport {
    /// Stable combined renderer/presentation backend identity.
    #[must_use]
    pub fn backend_journal(&self) -> Vec<u8> {
        let mut writer = Writer::new(NATIVE_PREVIEW_BACKEND_SCHEMA);
        writer.put_bytes(&self.metal.backend_journal());
        writer.put_str("native-cametal-layer");
        writer.put_u64(self.presentation.threads_per_threadgroup[0] as u64);
        writer.put_u64(self.presentation.threads_per_threadgroup[1] as u64);
        writer.put_u64(self.presentation.max_threads_per_threadgroup as u64);
        writer.put_u64(self.presentation.thread_execution_width as u64);
        writer
            .finish()
            .expect("the fixed-size native preview identity fits the schema limits")
    }

    /// Digest of [`NativePreviewReport::backend_journal`].
    #[must_use]
    pub fn backend_digest(&self) -> Digest {
        sha256(&self.backend_journal())
    }

    /// Frame-pixel bytes copied back to host memory.
    #[must_use]
    pub fn frame_pixel_readback_bytes(&self) -> usize {
        self.metal.readback_bytes
    }
}

/// Why Studio preview used the CPU route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewFallback {
    /// This target or machine has no usable Metal device.
    Unavailable,
    /// The current prepared frame uses an annex feature that has not landed.
    Unsupported(String),
    /// A device existed but a later annex operation failed.
    BackendFailure(String),
}

/// Which engine actually produced a preview frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewRoute {
    /// Metal produced the pixels.
    Metal,
    /// Fast CPU produced the pixels for the recorded reason.
    FastCpu(PreviewFallback),
}

/// One Studio-ready RGBA8 frame with truthful execution provenance.
#[derive(Debug)]
pub struct PreviewFrame {
    /// Tight, top-row-first RGBA8 pixels.
    pub frame: FrameBuffer,
    /// Actual execution route.
    pub route: PreviewRoute,
    /// PG-A record when Metal ran.
    pub metal: Option<MetalReport>,
}

impl PreviewFrame {
    /// The engine identity that truthfully produced these bytes.
    #[must_use]
    pub fn identity(&self) -> EngineIdentity {
        match self.route {
            PreviewRoute::Metal => EngineIdentity::metal(),
            PreviewRoute::FastCpu(_) => EngineIdentity::fast(),
        }
    }
}

/// Studio's renderer-owned selection and fallback boundary.
pub enum PreviewRenderer {
    /// Supported Metal device and compiled pipelines.
    Metal(Box<MetalRenderer>),
    /// Declared CPU fallback.
    FastCpu(PreviewFallback),
}

impl fmt::Debug for PreviewRenderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Metal(renderer) => f.debug_tuple("Metal").field(renderer).finish(),
            Self::FastCpu(reason) => f.debug_tuple("FastCpu").field(reason).finish(),
        }
    }
}

impl PreviewRenderer {
    /// Prefer Metal and select the CPU fallback only when Metal is unavailable.
    ///
    /// A shader or pipeline error on a machine that claims Metal is a product
    /// defect and is returned, not mislabeled as hardware absence.
    pub fn new() -> Result<Self, MetalError> {
        match MetalRenderer::new() {
            Ok(renderer) => Ok(Self::Metal(Box::new(renderer))),
            Err(error) if error.unavailable() => Ok(Self::FastCpu(PreviewFallback::Unavailable)),
            Err(error) => Err(error),
        }
    }

    /// Render a Studio-ready RGBA8 frame.
    ///
    /// A runtime backend failure demotes this renderer to fast CPU and returns
    /// the CPU frame without turning a live preview dark. The returned route
    /// and identity make that fallback observable.
    pub fn render(
        &mut self,
        plan: &RenderPlan,
        mono: &MonoTable,
        binning: &Binning,
        config: FrameConfig,
        cpu_threads: usize,
    ) -> Result<PreviewFrame, MetalError> {
        let mut transient = false;
        let reason = match self {
            Self::Metal(renderer) => match renderer.render_rgba8(plan, mono, binning, config) {
                Ok((frame, report)) => {
                    return Ok(PreviewFrame {
                        frame,
                        route: PreviewRoute::Metal,
                        metal: Some(report),
                    });
                }
                Err(
                    error @ (MetalError::UnsupportedStrokeProfile
                    | MetalError::UnsupportedFillProfile),
                ) => {
                    transient = true;
                    PreviewFallback::Unsupported(error.to_string())
                }
                Err(error) if error.permits_preview_fallback() => {
                    PreviewFallback::BackendFailure(error.to_string())
                }
                Err(error) => return Err(error),
            },
            Self::FastCpu(reason) => reason.clone(),
        };
        if !transient {
            *self = Self::FastCpu(reason.clone());
        }
        let frame = render_cpu_rgba8(plan, mono, binning, config, cpu_threads)?;
        Ok(PreviewFrame {
            frame,
            route: PreviewRoute::FastCpu(reason),
            metal: None,
        })
    }

    /// Render a prepared 3D Studio frame through the same truthful annex/fallback
    /// boundary.
    ///
    /// An unsupported prepared primitive falls back for this frame only; it does
    /// not poison a healthy Metal device. A runtime backend failure retains the
    /// existing permanent demotion policy.
    pub fn render_three_d(
        &mut self,
        job: &ThreeDJob<'_>,
        cpu_threads: usize,
    ) -> Result<PreviewFrame, MetalError> {
        let mut transient = None;
        let reason = match self {
            Self::Metal(renderer) => match renderer.render_three_d_rgba8(job) {
                Ok((frame, report)) => {
                    return Ok(PreviewFrame {
                        frame,
                        route: PreviewRoute::Metal,
                        metal: Some(report),
                    });
                }
                Err(
                    error @ (MetalError::UnsupportedThreeDPrimitive { .. }
                    | MetalError::UnsupportedStrokeProfile
                    | MetalError::UnsupportedFillProfile),
                ) => {
                    let reason = PreviewFallback::Unsupported(error.to_string());
                    transient = Some(reason.clone());
                    reason
                }
                Err(error) if error.permits_preview_fallback() => {
                    PreviewFallback::BackendFailure(error.to_string())
                }
                Err(error) => return Err(error),
            },
            Self::FastCpu(reason) => reason.clone(),
        };
        if transient.is_none() {
            *self = Self::FastCpu(reason.clone());
        }
        let frame = render_cpu_three_d_rgba8(job, cpu_threads)?;
        Ok(PreviewFrame {
            frame,
            route: PreviewRoute::FastCpu(reason),
            metal: None,
        })
    }
}

/// Lifetime-held Metal pipelines and output surfaces.
pub struct MetalRenderer {
    gateway: Gateway,
    raster: Pipeline,
    three_d: Pipeline,
    rgba8: Pipeline,
    nv12: Pipeline,
    p010: Pipeline,
    transfer_table: SharedBuffer,
    raw_surface: Option<SurfaceSlot>,
    output_surface: Option<OutputSurfaceSlot>,
    device: String,
    unified_memory: bool,
    kernel_digest: Digest,
    transfer_table_digest: Digest,
}

impl fmt::Debug for MetalRenderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MetalRenderer")
            .field("device", &self.device)
            .field("unified_memory", &self.unified_memory)
            .field(
                "raw_surface_bytes",
                &self.raw_surface.as_ref().map(|s| s.bytes),
            )
            .field(
                "output_surface_bytes",
                &self
                    .output_surface
                    .as_ref()
                    .map(|surface| surface.layout.total_bytes()),
            )
            .finish_non_exhaustive()
    }
}

impl MetalRenderer {
    /// Whether the current target exposes a Metal gateway.
    #[must_use]
    pub fn is_available() -> bool {
        Gateway::open().is_ok()
    }

    /// Open the device and compile the embedded safe-math pipelines once.
    pub fn new() -> Result<Self, MetalError> {
        let gateway = Gateway::open()?;
        let library = gateway.library_with(KERNEL_SOURCE, MathMode::Safe)?;
        let raster = library.pipeline(RASTER_KERNEL)?;
        let three_d = library.pipeline(THREE_D_KERNEL)?;
        let rgba8 = library.pipeline(RGBA8_KERNEL)?;
        let nv12 = library.pipeline(NV12_KERNEL)?;
        let p010 = library.pipeline(P010_KERNEL)?;
        let (transfer_words, transfer_table_digest) = build_transfer_table();
        let transfer_table = gateway.buffer_u32(&transfer_words)?;
        Ok(Self {
            gateway,
            raster,
            three_d,
            rgba8,
            nv12,
            p010,
            transfer_table,
            raw_surface: None,
            output_surface: None,
            device: gateway.device_name(),
            unified_memory: gateway.has_unified_memory(),
            kernel_digest: sha256(KERNEL_SOURCE.as_bytes()),
            transfer_table_digest,
        })
    }

    /// Render the annex's linear-light Rgba16F comparison surface.
    pub fn render_raw(
        &mut self,
        plan: &RenderPlan,
        mono: &MonoTable,
        binning: &Binning,
        config: FrameConfig,
    ) -> Result<(FrameBuffer, MetalReport), MetalError> {
        let started = Instant::now();
        let job = FrameJob::for_metal(plan, mono, binning, config)?;
        let flat = FlatFrame::derive(&job)?;
        let raw_layout = config.layout()?;
        let (raw_reused, upload_bytes) = self.dispatch_raster(&flat, raw_layout.total_bytes())?;
        let mut frame = FrameBuffer::new(raw_layout);
        self.raw_surface
            .as_ref()
            .ok_or(MetalError::Layout("raw surface disappeared"))?
            .buffer
            .read_u8(frame.as_bytes_mut())?;
        let report = self.report(
            &self.raster,
            flat.tile().saturating_mul(flat.tile()),
            upload_bytes,
            frame.as_bytes().len(),
            raw_reused,
            false,
            started.elapsed(),
            PixelFormat::Rgba16F,
            None,
            None,
        );
        Ok((frame, report))
    }

    /// Render and transfer directly to Studio's tight RGBA8 payload.
    ///
    /// The linear half surface remains device-resident between raster and
    /// transfer; only the four-byte preview surface crosses to the host.
    pub fn render_rgba8(
        &mut self,
        plan: &RenderPlan,
        mono: &MonoTable,
        binning: &Binning,
        config: FrameConfig,
    ) -> Result<(FrameBuffer, MetalReport), MetalError> {
        let started = Instant::now();
        let dispatch = self.dispatch_rgba8_surface(plan, mono, binning, config)?;
        let frame = self.read_rgba8_surface()?;
        let report = self.report(
            &self.raster,
            dispatch.threads_per_threadgroup,
            dispatch.upload_bytes,
            frame.as_bytes().len(),
            dispatch.raw_surface_reused,
            dispatch.output_surface_reused,
            started.elapsed(),
            PixelFormat::Rgba8,
            None,
            None,
        );
        Ok((frame, report))
    }

    /// Render a prepared 3D painter sequence into its linear-light comparison
    /// surface.
    ///
    /// The first annex tranche accepts camera-clipped true/glow dots and
    /// untextured Gouraud surfaces. Both consume the exact triangle IR already
    /// prepared for [`ThreeDJob`]'s CPU executor; retained vectors and textured
    /// surfaces return a typed refusal so a preview owner can select its declared
    /// CPU fallback without mislabeling the bytes.
    pub fn render_three_d_raw(
        &mut self,
        job: &ThreeDJob<'_>,
    ) -> Result<(FrameBuffer, MetalReport), MetalError> {
        let started = Instant::now();
        let flat = FlatThreeDFrame::derive(job)?;
        let raw_layout = job.layout()?;
        let (raw_reused, upload_bytes) = self.dispatch_three_d(&flat, raw_layout.total_bytes())?;
        let mut frame = FrameBuffer::new(raw_layout);
        self.raw_surface
            .as_ref()
            .ok_or(MetalError::Layout("raw surface disappeared"))?
            .buffer
            .read_u8(frame.as_bytes_mut())?;
        let report = self.report(
            &self.three_d,
            flat.tile().saturating_mul(flat.tile()),
            upload_bytes,
            frame.as_bytes().len(),
            raw_reused,
            false,
            started.elapsed(),
            PixelFormat::Rgba16F,
            None,
            None,
        );
        Ok((frame, report))
    }

    /// Render a prepared dot/surface 3D frame and transfer it on-device to the
    /// tight RGBA8 Studio payload.
    pub fn render_three_d_rgba8(
        &mut self,
        job: &ThreeDJob<'_>,
    ) -> Result<(FrameBuffer, MetalReport), MetalError> {
        let started = Instant::now();
        let dispatch = self.dispatch_three_d_rgba8_surface(job)?;
        let frame = self.read_rgba8_surface()?;
        let report = self.report(
            &self.three_d,
            dispatch.threads_per_threadgroup,
            dispatch.upload_bytes,
            frame.as_bytes().len(),
            dispatch.raw_surface_reused,
            dispatch.output_surface_reused,
            started.elapsed(),
            PixelFormat::Rgba8,
            None,
            None,
        );
        Ok((frame, report))
    }

    fn dispatch_rgba8_surface(
        &mut self,
        plan: &RenderPlan,
        mono: &MonoTable,
        binning: &Binning,
        config: FrameConfig,
    ) -> Result<Rgba8SurfaceDispatch, MetalError> {
        let job = FrameJob::for_metal(plan, mono, binning, config)?;
        let flat = FlatFrame::derive(&job)?;
        let raw_layout = config.layout()?;
        let (raw_surface_reused, upload_bytes) =
            self.dispatch_raster(&flat, raw_layout.total_bytes())?;
        let (output_surface_reused, transfer_upload_bytes) =
            self.transfer_rgba8_surface(config.viewport.width, config.viewport.height)?;
        Ok(Rgba8SurfaceDispatch {
            width: config.viewport.width,
            height: config.viewport.height,
            threads_per_threadgroup: flat.tile().saturating_mul(flat.tile()),
            upload_bytes: upload_bytes
                .checked_add(transfer_upload_bytes)
                .ok_or(MetalError::SizeOverflow("frame upload count"))?,
            raw_surface_reused,
            output_surface_reused,
        })
    }

    fn dispatch_three_d_rgba8_surface(
        &mut self,
        job: &ThreeDJob<'_>,
    ) -> Result<Rgba8SurfaceDispatch, MetalError> {
        let flat = FlatThreeDFrame::derive(job)?;
        let layout = job.layout()?;
        let (raw_surface_reused, upload_bytes) =
            self.dispatch_three_d(&flat, layout.total_bytes())?;
        let (output_surface_reused, transfer_upload_bytes) =
            self.transfer_rgba8_surface(layout.width(), layout.height())?;
        Ok(Rgba8SurfaceDispatch {
            width: layout.width(),
            height: layout.height(),
            threads_per_threadgroup: flat.tile().saturating_mul(flat.tile()),
            upload_bytes: upload_bytes
                .checked_add(transfer_upload_bytes)
                .ok_or(MetalError::SizeOverflow("3D frame upload count"))?,
            raw_surface_reused,
            output_surface_reused,
        })
    }

    /// Render and convert on-device to a negotiated NV12 layout.
    ///
    /// The layout's per-plane strides are honored exactly. Only the compact
    /// Y′CbCr allocation crosses to the host; the linear RGBA16F surface stays
    /// device-resident.
    #[allow(clippy::too_many_arguments)]
    pub fn render_nv12(
        &mut self,
        plan: &RenderPlan,
        mono: &MonoTable,
        binning: &Binning,
        config: FrameConfig,
        output_layout: FrameLayout,
        range: ColorRange,
        siting: ChromaSiting,
    ) -> Result<(FrameBuffer, MetalReport), MetalError> {
        self.render_yuv420(
            plan,
            mono,
            binning,
            config,
            output_layout,
            PixelFormat::Nv12,
            range,
            siting,
        )
    }

    /// Render and convert on-device to a negotiated P010 layout.
    ///
    /// Reel defines P010 as limited-range only. Full range is therefore a
    /// typed refusal before any raster work is submitted.
    #[allow(clippy::too_many_arguments)]
    pub fn render_p010(
        &mut self,
        plan: &RenderPlan,
        mono: &MonoTable,
        binning: &Binning,
        config: FrameConfig,
        output_layout: FrameLayout,
        range: ColorRange,
        siting: ChromaSiting,
    ) -> Result<(FrameBuffer, MetalReport), MetalError> {
        self.render_yuv420(
            plan,
            mono,
            binning,
            config,
            output_layout,
            PixelFormat::P010,
            range,
            siting,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn render_yuv420(
        &mut self,
        plan: &RenderPlan,
        mono: &MonoTable,
        binning: &Binning,
        config: FrameConfig,
        output_layout: FrameLayout,
        expected_format: PixelFormat,
        range: ColorRange,
        siting: ChromaSiting,
    ) -> Result<(FrameBuffer, MetalReport), MetalError> {
        validate_yuv_output(&output_layout, config, expected_format, range)?;

        let started = Instant::now();
        let job = FrameJob::for_metal(plan, mono, binning, config)?;
        let flat = FlatFrame::derive(&job)?;
        let raw_layout = config.layout()?;
        let (raw_reused, upload_bytes) = self.dispatch_raster(&flat, raw_layout.total_bytes())?;
        let output_reused =
            ensure_output_surface(self.gateway, &mut self.output_surface, &output_layout)?;

        let sample_size = output_layout.format().sample_size();
        let transfer_params = [
            output_layout.width(),
            output_layout.height(),
            checked_u32(output_layout.stride(0) / sample_size, "YUV luma stride")?,
            checked_u32(output_layout.stride(1) / sample_size, "YUV chroma stride")?,
            checked_u32(
                output_layout.plane_offset(1) / sample_size,
                "YUV chroma offset",
            )?,
            color_range_code(range),
            chroma_siting_code(siting),
        ];
        let params = self.gateway.buffer_u32(&transfer_params)?;
        let quad_width = output_layout.width() as usize / 2;
        let quad_height = output_layout.height() as usize / 2;
        let groups_x = quad_width.div_ceil(TRANSFER_TILE);
        let groups_y = quad_height.div_ceil(TRANSFER_TILE);
        let group_count = groups_x
            .checked_mul(groups_y)
            .ok_or(MetalError::SizeOverflow("YUV transfer group count"))?;
        let (pipeline, kernel) = match expected_format {
            PixelFormat::Nv12 => (&self.nv12, NV12_KERNEL),
            PixelFormat::P010 => (&self.p010, P010_KERNEL),
            _ => return Err(MetalError::Layout("non-YUV transfer pipeline selected")),
        };
        validate_threadgroup(pipeline, kernel, TRANSFER_TILE, TRANSFER_TILE)?;
        let status =
            self.gateway
                .buffer_zeroed(checked_bytes(group_count, 4, "YUV transfer status")?)?;
        self.gateway.dispatch(
            pipeline,
            &[
                &params,
                &self
                    .raw_surface
                    .as_ref()
                    .ok_or(MetalError::Layout("raw surface disappeared"))?
                    .buffer,
                &self.transfer_table,
                &self
                    .output_surface
                    .as_ref()
                    .ok_or(MetalError::Layout("YUV surface disappeared"))?
                    .buffer,
                &status,
            ],
            Grid::grid_2d(groups_x, groups_y, TRANSFER_TILE, TRANSFER_TILE),
        )?;
        verify_status(&status, group_count, kernel)?;

        let mut frame = FrameBuffer::new(output_layout);
        self.output_surface
            .as_ref()
            .ok_or(MetalError::Layout("YUV surface disappeared"))?
            .buffer
            .read_u8(frame.as_bytes_mut())?;
        let report = self.report(
            &self.raster,
            flat.tile().saturating_mul(flat.tile()),
            upload_bytes
                .checked_add(std::mem::size_of_val(&transfer_params))
                .ok_or(MetalError::SizeOverflow("frame upload count"))?,
            frame.as_bytes().len(),
            raw_reused,
            output_reused,
            started.elapsed(),
            expected_format,
            Some(range),
            Some(siting),
        );
        Ok((frame, report))
    }

    fn dispatch_raster(
        &mut self,
        flat: &FlatFrame,
        raw_bytes: usize,
    ) -> Result<(bool, usize), MetalError> {
        let raw_reused = ensure_surface(self.gateway, &mut self.raw_surface, raw_bytes)?;
        let params_u32 = self.gateway.buffer_u32(&flat.params_u32)?;
        let params_f32 = self.gateway.buffer_f32(&flat.params_f32)?;
        let segments = self.gateway.buffer_f32(nonempty_f32(&flat.segments))?;
        let pieces = self.gateway.buffer_f32(nonempty_f32(&flat.pieces))?;
        let joins = self.gateway.buffer_f32(nonempty_f32(&flat.joins))?;
        let stations = self.gateway.buffer_f32(nonempty_f32(&flat.stations))?;
        let draw_u32 = self.gateway.buffer_u32(nonempty_u32(&flat.draw_u32))?;
        let draw_f32 = self.gateway.buffer_f32(nonempty_f32(&flat.draw_f32))?;
        let styles = self.gateway.buffer_f32(nonempty_f32(&flat.styles))?;
        let tile_offsets = self.gateway.buffer_u32(&flat.tile_offsets)?;
        let tile_draws = self.gateway.buffer_u32(nonempty_u32(&flat.tile_draws))?;
        let tile_flags = self.gateway.buffer_u32(nonempty_u32(&flat.tile_flags))?;
        let group_count = flat
            .cols()
            .checked_mul(flat.rows())
            .ok_or(MetalError::SizeOverflow("raster group count"))?;
        let status = self
            .gateway
            .buffer_zeroed(checked_bytes(group_count, 4, "raster status")?)?;

        let tile = flat.tile();
        validate_threadgroup(&self.raster, RASTER_KERNEL, tile, tile)?;
        self.gateway.dispatch(
            &self.raster,
            &[
                &params_u32,
                &params_f32,
                &segments,
                &pieces,
                &joins,
                &stations,
                &draw_u32,
                &draw_f32,
                &styles,
                &tile_offsets,
                &tile_draws,
                &tile_flags,
                &self
                    .raw_surface
                    .as_ref()
                    .ok_or(MetalError::Layout("raw surface disappeared"))?
                    .buffer,
                &status,
            ],
            Grid::grid_2d(flat.cols(), flat.rows(), tile, tile),
        )?;
        verify_status(&status, group_count, RASTER_KERNEL)?;
        Ok((raw_reused, flat.upload_bytes()?))
    }

    fn dispatch_three_d(
        &mut self,
        flat: &FlatThreeDFrame,
        raw_bytes: usize,
    ) -> Result<(bool, usize), MetalError> {
        let raw_reused = ensure_surface(self.gateway, &mut self.raw_surface, raw_bytes)?;
        let params_u32 = self.gateway.buffer_u32(&flat.params_u32)?;
        let params_f32 = self.gateway.buffer_f32(&flat.params_f32)?;
        let triangles = self.gateway.buffer_f32(nonempty_f32(&flat.triangles))?;
        let draw_u32 = self.gateway.buffer_u32(nonempty_u32(&flat.draw_u32))?;
        let draw_f32 = self.gateway.buffer_f32(nonempty_f32(&flat.draw_f32))?;
        let tile_offsets = self.gateway.buffer_u32(&flat.tile_offsets)?;
        let tile_draws = self.gateway.buffer_u32(nonempty_u32(&flat.tile_draws))?;
        let group_count = flat
            .cols()
            .checked_mul(flat.rows())
            .ok_or(MetalError::SizeOverflow("3D raster group count"))?;
        let status =
            self.gateway
                .buffer_zeroed(checked_bytes(group_count, 4, "3D raster status")?)?;

        let tile = flat.tile();
        validate_threadgroup(&self.three_d, THREE_D_KERNEL, tile, tile)?;
        self.gateway.dispatch(
            &self.three_d,
            &[
                &params_u32,
                &params_f32,
                &triangles,
                &draw_u32,
                &draw_f32,
                &tile_offsets,
                &tile_draws,
                &self
                    .raw_surface
                    .as_ref()
                    .ok_or(MetalError::Layout("raw surface disappeared"))?
                    .buffer,
                &status,
            ],
            Grid::grid_2d(flat.cols(), flat.rows(), tile, tile),
        )?;
        verify_status(&status, group_count, THREE_D_KERNEL)?;
        Ok((raw_reused, flat.upload_bytes()?))
    }

    fn transfer_rgba8_surface(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<(bool, usize), MetalError> {
        let output_layout = FrameLayout::tight(PixelFormat::Rgba8, width, height)?;
        let output_reused =
            ensure_output_surface(self.gateway, &mut self.output_surface, &output_layout)?;
        let transfer_params = [
            width,
            height,
            u32::try_from(output_layout.stride(0))
                .map_err(|_| MetalError::SizeOverflow("RGBA8 stride"))?,
        ];
        let params = self.gateway.buffer_u32(&transfer_params)?;
        let groups_x = (width as usize).div_ceil(TRANSFER_TILE);
        let groups_y = (height as usize).div_ceil(TRANSFER_TILE);
        let group_count = groups_x
            .checked_mul(groups_y)
            .ok_or(MetalError::SizeOverflow("transfer group count"))?;
        validate_threadgroup(&self.rgba8, RGBA8_KERNEL, TRANSFER_TILE, TRANSFER_TILE)?;
        let status =
            self.gateway
                .buffer_zeroed(checked_bytes(group_count, 4, "transfer status")?)?;
        self.gateway.dispatch(
            &self.rgba8,
            &[
                &params,
                &self
                    .raw_surface
                    .as_ref()
                    .ok_or(MetalError::Layout("raw surface disappeared"))?
                    .buffer,
                &self.transfer_table,
                &self
                    .output_surface
                    .as_ref()
                    .ok_or(MetalError::Layout("RGBA8 surface disappeared"))?
                    .buffer,
                &status,
            ],
            Grid::grid_2d(groups_x, groups_y, TRANSFER_TILE, TRANSFER_TILE),
        )?;
        verify_status(&status, group_count, RGBA8_KERNEL)?;

        Ok((output_reused, std::mem::size_of_val(&transfer_params)))
    }

    fn read_rgba8_surface(&self) -> Result<FrameBuffer, MetalError> {
        let surface = self
            .output_surface
            .as_ref()
            .ok_or(MetalError::Layout("RGBA8 surface disappeared"))?;
        let mut frame = FrameBuffer::new(surface.layout.clone());
        surface.buffer.read_u8(frame.as_bytes_mut())?;
        Ok(frame)
    }

    #[allow(clippy::too_many_arguments)]
    fn report(
        &self,
        pipeline: &Pipeline,
        threads_per_threadgroup: usize,
        upload_bytes: usize,
        readback_bytes: usize,
        raw_surface_reused: bool,
        output_surface_reused: bool,
        elapsed: Duration,
        output_format: PixelFormat,
        color_range: Option<ColorRange>,
        chroma_siting: Option<ChromaSiting>,
    ) -> MetalReport {
        MetalReport {
            identity: EngineIdentity::metal(),
            device: self.device.clone(),
            unified_memory: self.unified_memory,
            math_mode: "safe",
            kernel_digest: self.kernel_digest,
            transfer_table_digest: self.transfer_table_digest,
            threads_per_threadgroup,
            max_threads_per_threadgroup: pipeline.max_threads_per_threadgroup(),
            thread_execution_width: pipeline.thread_execution_width(),
            upload_bytes,
            readback_bytes,
            raw_surface_reused,
            output_surface_reused,
            elapsed,
            output_format,
            color_range,
            chroma_siting,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Rgba8SurfaceDispatch {
    width: u32,
    height: u32,
    threads_per_threadgroup: usize,
    upload_bytes: usize,
    raw_surface_reused: bool,
    output_surface_reused: bool,
}

#[derive(Debug, Clone, Copy)]
enum NativeRenderPipeline {
    TwoD,
    ThreeD,
}

/// Lumen's main-thread native Metal preview surface.
///
/// This owner couples the renderer's lifetime-held RGBA8 [`SharedBuffer`] to
/// frankentorch's `CAMetalLayer` presenter. No frame-pixel readback occurs on
/// this path. Browser multipart-PNG and terminal protocols are separate,
/// CPU-visible stream consumers and are selected by `fmn-studio` only when
/// native presentation is unavailable or fails.
pub struct NativePreviewRenderer {
    renderer: MetalRenderer,
    presenter: NativePresenter,
}

impl fmt::Debug for NativePreviewRenderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativePreviewRenderer")
            .field("renderer", &self.renderer)
            .field("presentation", &self.presenter.pipeline_info())
            .field("closed", &self.presenter.is_closed())
            .finish()
    }
}

impl NativePreviewRenderer {
    /// Open Lumen and a native `CAMetalLayer` preview window.
    ///
    /// Frankentorch enforces the process-main-thread requirement and retains
    /// every AppKit, Objective-C, drawable, and command-buffer lifetime.
    pub fn new(
        width: u32,
        height: u32,
        title: impl Into<String>,
    ) -> Result<Self, NativePreviewError> {
        let config = PresentationConfig::new(width, height, title);
        config.validate()?;
        let renderer = MetalRenderer::new()?;
        let presenter = NativePresenter::open(&renderer.gateway, config)?;
        Ok(Self {
            renderer,
            presenter,
        })
    }

    /// Fixed occupancy facts for the native drawable transfer pipeline.
    #[must_use]
    pub fn presentation_pipeline_info(&self) -> PresentationPipelineInfo {
        self.presenter.pipeline_info()
    }

    /// Whether the native window has completed teardown.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.presenter.is_closed()
    }

    /// Drain pending AppKit events and report the native surface state.
    pub fn poll_events(&mut self) -> Result<PresentationState, NativePreviewError> {
        self.presenter.poll_events().map_err(Into::into)
    }

    /// Render and present a 2D Studio frame without copying frame pixels back
    /// into host-owned memory.
    pub fn render(
        &mut self,
        plan: &RenderPlan,
        mono: &MonoTable,
        binning: &Binning,
        config: FrameConfig,
    ) -> Result<NativePreviewReport, NativePreviewError> {
        let started = Instant::now();
        let dispatch = self
            .renderer
            .dispatch_rgba8_surface(plan, mono, binning, config)?;
        self.present_dispatch(dispatch, NativeRenderPipeline::TwoD, started)
    }

    /// Render and present a prepared 3D Studio frame without frame-pixel
    /// readback.
    pub fn render_three_d(
        &mut self,
        job: &ThreeDJob<'_>,
    ) -> Result<NativePreviewReport, NativePreviewError> {
        let started = Instant::now();
        let dispatch = self.renderer.dispatch_three_d_rgba8_surface(job)?;
        self.present_dispatch(dispatch, NativeRenderPipeline::ThreeD, started)
    }

    /// Idempotently close the native preview surface.
    pub fn close(&mut self) -> Result<(), NativePreviewError> {
        self.presenter.close().map_err(Into::into)
    }

    fn present_dispatch(
        &mut self,
        dispatch: Rgba8SurfaceDispatch,
        pipeline: NativeRenderPipeline,
        started: Instant,
    ) -> Result<NativePreviewReport, NativePreviewError> {
        let surface = self
            .renderer
            .output_surface
            .as_ref()
            .ok_or(MetalError::Layout("RGBA8 surface disappeared"))?;
        if surface.layout.width() != dispatch.width || surface.layout.height() != dispatch.height {
            return Err(MetalError::Layout("RGBA8 presentation dimensions drifted").into());
        }
        let outcome = self.presenter.present_rgba8(
            &surface.buffer,
            dispatch.width,
            dispatch.height,
            surface.layout.stride(0),
        )?;
        let upload_bytes = dispatch
            .upload_bytes
            .checked_add(std::mem::size_of::<[u32; 3]>())
            .ok_or(MetalError::SizeOverflow("native presentation upload count"))?;
        let render_pipeline = match pipeline {
            NativeRenderPipeline::TwoD => &self.renderer.raster,
            NativeRenderPipeline::ThreeD => &self.renderer.three_d,
        };
        let metal = self.renderer.report(
            render_pipeline,
            dispatch.threads_per_threadgroup,
            upload_bytes,
            0,
            dispatch.raw_surface_reused,
            dispatch.output_surface_reused,
            started.elapsed(),
            PixelFormat::Rgba8,
            None,
            None,
        );
        Ok(NativePreviewReport {
            metal,
            presentation: self.presenter.pipeline_info(),
            outcome,
        })
    }
}

struct SurfaceSlot {
    bytes: usize,
    buffer: SharedBuffer,
}

struct OutputSurfaceSlot {
    layout: FrameLayout,
    buffer: SharedBuffer,
}

/// Return whether the existing lifetime-held surface was reused.
fn ensure_surface(
    gateway: Gateway,
    slot: &mut Option<SurfaceSlot>,
    bytes: usize,
) -> Result<bool, MetalError> {
    if slot.as_ref().is_some_and(|surface| surface.bytes == bytes) {
        return Ok(true);
    }
    let buffer = gateway.buffer_zeroed(bytes)?;
    *slot = Some(SurfaceSlot { bytes, buffer });
    Ok(false)
}

/// Return whether the exact negotiated layout's lifetime-held surface was
/// reused. Byte length alone is insufficient: equal-size layouts can put
/// padding at different offsets, and transfer kernels intentionally leave
/// padding untouched.
fn ensure_output_surface(
    gateway: Gateway,
    slot: &mut Option<OutputSurfaceSlot>,
    layout: &FrameLayout,
) -> Result<bool, MetalError> {
    if slot
        .as_ref()
        .is_some_and(|surface| surface.layout == *layout)
    {
        return Ok(true);
    }
    let buffer = gateway.buffer_zeroed(layout.total_bytes())?;
    *slot = Some(OutputSurfaceSlot {
        layout: layout.clone(),
        buffer,
    });
    Ok(false)
}

fn validate_yuv_output(
    layout: &FrameLayout,
    config: FrameConfig,
    expected_format: PixelFormat,
    range: ColorRange,
) -> Result<(), MetalError> {
    if expected_format == PixelFormat::P010 && range != ColorRange::Limited {
        return Err(FrameError::UnsupportedConversion("P010 output is limited-range only").into());
    }
    let expected = match expected_format {
        PixelFormat::Nv12 => "Nv12 destination",
        PixelFormat::P010 => "P010 destination",
        _ => return Err(MetalError::Layout("non-YUV output format selected")),
    };
    if layout.format() != expected_format {
        return Err(FrameError::FormatMismatch {
            expected,
            got: layout.format(),
        }
        .into());
    }
    if layout.width() != config.viewport.width || layout.height() != config.viewport.height {
        return Err(FrameError::DimensionMismatch.into());
    }
    Ok(())
}

fn build_transfer_table() -> (Vec<u32>, Digest) {
    let tables = fmn_frame::transfer::tables();
    let mut bytes = Vec::with_capacity(TRANSFER_TABLE_ENTRIES * 2);
    for bits in 0..=u16::MAX {
        bytes.push(tables.srgb8_from_f16(bits));
    }
    for bits in 0..=u16::MAX {
        bytes.push(tables.linear8_from_f16(bits));
    }
    let digest = sha256(&bytes);
    let (chunks, remainder) = bytes.as_chunks::<4>();
    debug_assert!(remainder.is_empty());
    let words = chunks
        .iter()
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect::<Vec<_>>();
    debug_assert_eq!(words.len(), TRANSFER_TABLE_WORDS);
    (words, digest)
}

fn checked_u32(value: usize, name: &'static str) -> Result<u32, MetalError> {
    u32::try_from(value).map_err(|_| MetalError::SizeOverflow(name))
}

const fn color_range_code(range: ColorRange) -> u32 {
    match range {
        ColorRange::Limited => 0,
        ColorRange::Full => 1,
    }
}

const fn chroma_siting_code(siting: ChromaSiting) -> u32 {
    match siting {
        ChromaSiting::Left => 0,
        ChromaSiting::Center => 1,
    }
}

fn validate_threadgroup(
    pipeline: &Pipeline,
    kernel: &'static str,
    x: usize,
    y: usize,
) -> Result<(), MetalError> {
    let requested = x
        .checked_mul(y)
        .ok_or(MetalError::SizeOverflow("threadgroup size"))?;
    let maximum = pipeline.max_threads_per_threadgroup();
    if requested > maximum {
        return Err(MetalError::ThreadgroupTooLarge {
            kernel,
            requested,
            maximum,
        });
    }
    Ok(())
}

fn verify_status(
    status: &SharedBuffer,
    groups: usize,
    kernel: &'static str,
) -> Result<(), MetalError> {
    let mut words = vec![0u32; groups];
    status.read_u32(&mut words)?;
    let completed = words
        .iter()
        .filter(|&&word| word == STATUS_COMPLETE)
        .count();
    if completed != groups {
        return Err(MetalError::IncompleteDispatch {
            kernel,
            completed,
            expected: groups,
        });
    }
    Ok(())
}

fn render_cpu_rgba8(
    plan: &RenderPlan,
    mono: &MonoTable,
    binning: &Binning,
    config: FrameConfig,
    threads: usize,
) -> Result<FrameBuffer, MetalError> {
    let job = FrameJob::with_identity(plan, mono, binning, config, EngineIdentity::fast())?;
    let raw = job.render(threads)?;
    let layout = FrameLayout::tight(
        PixelFormat::Rgba8,
        config.viewport.width,
        config.viewport.height,
    )?;
    let mut output = FrameBuffer::new(layout);
    fmn_frame::convert::rgba16f_to_rgba8(&raw, &mut output)?;
    Ok(output)
}

fn render_cpu_three_d_rgba8(
    job: &ThreeDJob<'_>,
    threads: usize,
) -> Result<FrameBuffer, MetalError> {
    let raw = job.render(threads)?;
    let layout = FrameLayout::tight(
        PixelFormat::Rgba8,
        raw.layout().width(),
        raw.layout().height(),
    )?;
    let mut output = FrameBuffer::new(layout);
    fmn_frame::convert::rgba16f_to_rgba8(&raw, &mut output)?;
    Ok(output)
}

#[derive(Debug)]
struct FlatFrame {
    params_u32: Vec<u32>,
    params_f32: Vec<f32>,
    segments: Vec<f32>,
    pieces: Vec<f32>,
    joins: Vec<f32>,
    stations: Vec<f32>,
    draw_u32: Vec<u32>,
    draw_f32: Vec<f32>,
    styles: Vec<f32>,
    tile_offsets: Vec<u32>,
    tile_draws: Vec<u32>,
    tile_flags: Vec<u32>,
}

impl FlatFrame {
    fn derive(job: &FrameJob<'_>) -> Result<Self, MetalError> {
        let config = job.frame_config();
        let binning = job.frame_binning();
        let tile = binning.tiling().fine_tile.max(1);
        let cols = config.viewport.width.div_ceil(tile);
        let rows = config.viewport.height.div_ceil(tile);
        if binning.tile_count()
            != (cols as usize)
                .checked_mul(rows as usize)
                .ok_or(MetalError::SizeOverflow("tile count"))?
        {
            return Err(MetalError::Layout("binning grid does not match the frame"));
        }
        if binning.offsets().len() != binning.tile_count() + 1
            || binning.draws().len() != binning.flags().len()
        {
            return Err(MetalError::Layout("CSR arrays are not parallel"));
        }

        let samples = match config.aa {
            AaPolicy::Adaptive => 1,
            AaPolicy::Ssaa2x => 2,
            AaPolicy::Ssaa4x => 4,
        };
        let mut flat = Self {
            params_u32: vec![
                config.viewport.width,
                config.viewport.height,
                tile,
                cols,
                samples,
                u32::try_from(job.prepared_draws().len())
                    .map_err(|_| MetalError::SizeOverflow("draw count"))?,
            ],
            params_f32: vec![
                config.background.r as f32,
                config.background.g as f32,
                config.background.b as f32,
                config.background.a as f32,
            ],
            segments: Vec::new(),
            pieces: Vec::new(),
            joins: Vec::new(),
            stations: Vec::new(),
            draw_u32: Vec::with_capacity(job.prepared_draws().len() * DRAW_U32_STRIDE),
            draw_f32: Vec::with_capacity(job.prepared_draws().len() * DRAW_F32_STRIDE),
            styles: Vec::with_capacity(job.prepared_draws().len() * STYLE_STRIDE),
            tile_offsets: binning.offsets().to_vec(),
            tile_draws: binning.draws().to_vec(),
            tile_flags: binning.flags().to_vec(),
        };

        for draw in job.prepared_draws() {
            match draw {
                Some(draw) => flat.push_draw(job, draw)?,
                None => flat.push_empty_draw(),
            }
        }
        flat.validate()?;
        Ok(flat)
    }

    fn push_draw(&mut self, job: &FrameJob<'_>, draw: &Draw) -> Result<(), MetalError> {
        if draw.style.fill_profile.is_some() {
            return Err(MetalError::UnsupportedFillProfile);
        }
        if draw.style.stroke_profile.is_some() {
            return Err(MetalError::UnsupportedStrokeProfile);
        }
        let config = job.frame_config();
        let first_segment = scalar_row(&self.segments, SEGMENT_STRIDE, "segment index")?;
        for segment in job.segments_of(draw) {
            push_segment(
                &mut self.segments,
                segment,
                config,
                draw.translate,
                draw.straight_segments,
            );
        }
        let segment_count = scalar_row(&self.segments, SEGMENT_STRIDE, "segment count")?
            .checked_sub(first_segment)
            .ok_or(MetalError::Layout("segment range reversed"))?;

        let first_piece = scalar_row(&self.pieces, PIECE_STRIDE, "piece index")?;
        for piece in job.pieces_of(draw) {
            self.pieces.extend_from_slice(&[
                (piece.p0[0] + draw.translate[0]) as f32,
                (piece.p0[1] + draw.translate[1]) as f32,
                (piece.p1[0] + draw.translate[0]) as f32,
                (piece.p1[1] + draw.translate[1]) as f32,
                (piece.p2[0] + draw.translate[0]) as f32,
                (piece.p2[1] + draw.translate[1]) as f32,
            ]);
        }
        let piece_count = scalar_row(&self.pieces, PIECE_STRIDE, "piece count")?
            .checked_sub(first_piece)
            .ok_or(MetalError::Layout("piece range reversed"))?;

        let first_join = scalar_row(&self.joins, JOIN_STRIDE, "join index")?;
        for join in job.joins_of(draw) {
            self.joins.extend_from_slice(&[
                join.anchor[0] as f32,
                join.anchor[1] as f32,
                join.t_in[0] as f32,
                join.t_in[1] as f32,
                join.t_out[0] as f32,
                join.t_out[1] as f32,
                join.half_width as f32,
                join.bisector[0] as f32,
                join.bisector[1] as f32,
                join.n_in[0] as f32,
                join.n_in[1] as f32,
                join.n_out[0] as f32,
                join.n_out[1] as f32,
            ]);
        }
        let join_count = scalar_row(&self.joins, JOIN_STRIDE, "join count")?
            .checked_sub(first_join)
            .ok_or(MetalError::Layout("join range reversed"))?;

        let first_station = scalar_row(&self.stations, STATION_STRIDE, "station index")?;
        let mut multi_contour_gradient = false;
        if let Some(field) = job.field_of(draw) {
            let (points, params, next, edge_params) = field.stations();
            if points.len() != params.len() {
                return Err(MetalError::Layout("gradient stations are not parallel"));
            }
            multi_contour_gradient = field.has_explicit_contours();
            if multi_contour_gradient
                && (points.len() != next.len() || points.len() != edge_params.len())
            {
                return Err(MetalError::Layout(
                    "gradient contour edges are not parallel",
                ));
            }
            for (offset, (point, &param)) in points.iter().zip(params).enumerate() {
                let local_next = if multi_contour_gradient {
                    next[offset]
                } else if offset + 1 < points.len() {
                    offset + 1
                } else {
                    0
                };
                if local_next >= points.len() {
                    return Err(MetalError::Layout("gradient contour edge escapes draw"));
                }
                let local_next = u32::try_from(local_next)
                    .map_err(|_| MetalError::SizeOverflow("gradient next station"))?;
                let global_next = first_station
                    .checked_add(local_next)
                    .ok_or(MetalError::SizeOverflow("gradient next station"))?;
                let edge_param = if multi_contour_gradient {
                    edge_params[offset]
                } else if offset + 1 < params.len() {
                    params[offset + 1]
                } else {
                    1.0
                };
                self.stations.extend_from_slice(&[
                    (point[0] + draw.translate[0]) as f32,
                    (point[1] + draw.translate[1]) as f32,
                    param as f32,
                    f32::from_bits(global_next),
                    edge_param as f32,
                ]);
            }
        }
        let station_count = scalar_row(&self.stations, STATION_STRIDE, "station count")?
            .checked_sub(first_station)
            .ok_or(MetalError::Layout("station range reversed"))?;

        let mut flags = 0;
        if draw.draws_fill {
            flags |= DRAW_FILL;
        }
        if draw.draws_stroke {
            flags |= DRAW_STROKE;
        }
        if draw.style.stroke_behind {
            flags |= STROKE_BEHIND;
        }
        if draw.flat_fill.is_some() {
            flags |= FLAT_FILL;
        }
        if multi_contour_gradient {
            flags |= MULTI_CONTOUR_GRADIENT;
        }
        let joint = match draw.style.joint_type {
            JointType::Auto | JointType::NoJoint => 0,
            JointType::Bevel => 1,
            JointType::Miter => 2,
        };
        self.draw_u32.extend_from_slice(&[
            first_segment,
            segment_count,
            first_piece,
            piece_count,
            first_join,
            join_count,
            first_station,
            station_count,
            flags,
            joint,
        ]);
        let stroke_slab = draw.stroke.map_or([0.0; 4], |stroke| stroke.slab);
        self.draw_f32.extend(
            draw.fill_slab
                .into_iter()
                .chain(stroke_slab)
                .map(|value| value as f32),
        );
        self.styles.extend_from_slice(&[
            draw.style.fill_rgba[0],
            draw.style.fill_rgba[1],
            draw.style.fill_rgba[2],
            draw.style.fill_rgba[3],
            draw.style.fill_rgba_end[0],
            draw.style.fill_rgba_end[1],
            draw.style.fill_rgba_end[2],
            draw.style.fill_rgba_end[3],
            draw.style.stroke_rgba[0],
            draw.style.stroke_rgba[1],
            draw.style.stroke_rgba[2],
            draw.style.stroke_rgba[3],
            draw.style.stroke_rgba_end[0],
            draw.style.stroke_rgba_end[1],
            draw.style.stroke_rgba_end[2],
            draw.style.stroke_rgba_end[3],
            crate::stroke::width_px(draw.style.stroke_width, config.map) as f32,
            crate::stroke::width_px(draw.style.stroke_width_end, config.map) as f32,
            crate::fill::border_width_px(draw.style.fill_border_width, config.map) as f32,
            draw.style.anti_alias_width,
        ]);
        Ok(())
    }

    fn push_empty_draw(&mut self) {
        self.draw_u32
            .extend(std::iter::repeat_n(0, DRAW_U32_STRIDE));
        self.draw_f32
            .extend(std::iter::repeat_n(0.0, DRAW_F32_STRIDE));
        self.styles.extend(std::iter::repeat_n(0.0, STYLE_STRIDE));
    }

    fn validate(&self) -> Result<(), MetalError> {
        for (slice, stride, name) in [
            (self.segments.as_slice(), SEGMENT_STRIDE, "segments"),
            (self.pieces.as_slice(), PIECE_STRIDE, "pieces"),
            (self.joins.as_slice(), JOIN_STRIDE, "joins"),
            (self.stations.as_slice(), STATION_STRIDE, "stations"),
            (self.draw_f32.as_slice(), DRAW_F32_STRIDE, "draw_f32"),
            (self.styles.as_slice(), STYLE_STRIDE, "styles"),
        ] {
            if !slice.len().is_multiple_of(stride) {
                return Err(MetalError::Layout(name));
            }
        }
        if !self.draw_u32.len().is_multiple_of(DRAW_U32_STRIDE) {
            return Err(MetalError::Layout("draw_u32"));
        }
        let draws = self.draw_u32.len() / DRAW_U32_STRIDE;
        if self.draw_f32.len() / DRAW_F32_STRIDE != draws
            || self.styles.len() / STYLE_STRIDE != draws
            || self.params_u32.get(5).copied() != u32::try_from(draws).ok()
        {
            return Err(MetalError::Layout("draw tables are not parallel"));
        }
        Ok(())
    }

    fn tile(&self) -> usize {
        self.params_u32[2] as usize
    }

    fn cols(&self) -> usize {
        self.params_u32[3] as usize
    }

    fn rows(&self) -> usize {
        (self.params_u32[1] as usize).div_ceil(self.tile())
    }

    fn upload_bytes(&self) -> Result<usize, MetalError> {
        let scalars = [
            self.params_u32.len(),
            self.params_f32.len(),
            self.segments.len(),
            self.pieces.len(),
            self.joins.len(),
            self.stations.len(),
            self.draw_u32.len(),
            self.draw_f32.len(),
            self.styles.len(),
            self.tile_offsets.len(),
            self.tile_draws.len(),
            self.tile_flags.len(),
        ]
        .into_iter()
        .try_fold(0usize, |sum, len| sum.checked_add(len))
        .ok_or(MetalError::SizeOverflow("frame upload scalar count"))?;
        checked_bytes(scalars, 4, "frame upload bytes")
    }
}

#[derive(Debug)]
struct FlatThreeDFrame {
    params_u32: Vec<u32>,
    params_f32: Vec<f32>,
    triangles: Vec<f32>,
    draw_u32: Vec<u32>,
    draw_f32: Vec<f32>,
    tile_offsets: Vec<u32>,
    tile_draws: Vec<u32>,
}

impl FlatThreeDFrame {
    fn derive(job: &ThreeDJob<'_>) -> Result<Self, MetalError> {
        let camera = job.camera();
        let width = camera.pixel_width();
        let height = camera.pixel_height();
        let tile = job.tiling().fine_tile.max(1);
        let cols = width.div_ceil(tile);
        let sample_grid = job.sample_grid();
        if !matches!(sample_grid, 1 | 2 | 4) {
            return Err(MetalError::Layout("unsupported 3D sample grid"));
        }
        let draw_count = u32::try_from(job.prepared_draws().len())
            .map_err(|_| MetalError::SizeOverflow("3D draw count"))?;
        let background = camera.background();
        let mut flat = Self {
            params_u32: vec![width, height, tile, cols, sample_grid, draw_count],
            params_f32: vec![
                background.r as f32,
                background.g as f32,
                background.b as f32,
                background.a as f32,
            ],
            triangles: Vec::new(),
            draw_u32: Vec::with_capacity(job.prepared_draws().len() * THREE_D_DRAW_U32_STRIDE),
            draw_f32: Vec::with_capacity(job.prepared_draws().len() * THREE_D_DRAW_F32_STRIDE),
            tile_offsets: Vec::new(),
            tile_draws: Vec::new(),
        };
        let mut draw_bounds = Vec::with_capacity(job.prepared_draws().len());

        for (draw_index, draw) in job.prepared_draws().iter().enumerate() {
            let CompiledPrimitive::Triangles { triangles, shader } = &draw.primitive else {
                return Err(MetalError::UnsupportedThreeDPrimitive {
                    draw: draw_index,
                    primitive: "retained-vector",
                });
            };
            let shader_code = match *shader {
                Shader::Gouraud => THREE_D_SHADER_GOURAUD,
                Shader::Dot { .. } => THREE_D_SHADER_DOT,
                Shader::Texture { .. } => {
                    return Err(MetalError::UnsupportedThreeDPrimitive {
                        draw: draw_index,
                        primitive: "textured-surface",
                    });
                }
            };

            let first_triangle = scalar_row(
                &flat.triangles,
                THREE_D_TRIANGLE_STRIDE,
                "3D triangle index",
            )?;
            for triangle in triangles {
                push_three_d_triangle(&mut flat.triangles, triangle);
            }
            let triangle_count = scalar_row(
                &flat.triangles,
                THREE_D_TRIANGLE_STRIDE,
                "3D triangle count",
            )?
            .checked_sub(first_triangle)
            .ok_or(MetalError::Layout("3D triangle range reversed"))?;
            flat.draw_u32.extend_from_slice(&[
                first_triangle,
                triangle_count,
                shader_code,
                u32::from(draw.depth_test),
            ]);
            flat.draw_f32
                .extend_from_slice(&three_d_shader_slab(*shader));
            draw_bounds.push(draw.bounds);
        }

        (flat.tile_offsets, flat.tile_draws) =
            stable_three_d_bins(&draw_bounds, width, height, tile)?;
        flat.validate()?;
        Ok(flat)
    }

    fn validate(&self) -> Result<(), MetalError> {
        if self.params_u32.len() != 6 || self.params_f32.len() != 4 {
            return Err(MetalError::Layout("3D frame parameters"));
        }
        if !self.triangles.len().is_multiple_of(THREE_D_TRIANGLE_STRIDE)
            || !self.draw_u32.len().is_multiple_of(THREE_D_DRAW_U32_STRIDE)
            || !self.draw_f32.len().is_multiple_of(THREE_D_DRAW_F32_STRIDE)
        {
            return Err(MetalError::Layout("3D scalar-table stride"));
        }
        if self
            .params_f32
            .iter()
            .chain(&self.triangles)
            .chain(&self.draw_f32)
            .any(|value| !value.is_finite())
        {
            return Err(MetalError::Layout(
                "3D f32 packet contains a non-finite value",
            ));
        }
        let draws = self.draw_u32.len() / THREE_D_DRAW_U32_STRIDE;
        if self.draw_f32.len() / THREE_D_DRAW_F32_STRIDE != draws
            || self.params_u32.get(5).copied() != u32::try_from(draws).ok()
        {
            return Err(MetalError::Layout("3D draw tables are not parallel"));
        }
        let tile_count = self
            .cols()
            .checked_mul(self.rows())
            .ok_or(MetalError::SizeOverflow("3D tile count"))?;
        if self.tile_offsets.len() != tile_count + 1
            || self.tile_offsets.first().copied() != Some(0)
            || self.tile_offsets.last().copied() != u32::try_from(self.tile_draws.len()).ok()
            || self.tile_offsets.windows(2).any(|pair| pair[0] > pair[1])
            || self.tile_draws.iter().any(|&draw| draw as usize >= draws)
        {
            return Err(MetalError::Layout("3D stable tile CSR"));
        }
        Ok(())
    }

    fn tile(&self) -> usize {
        self.params_u32[2] as usize
    }

    fn cols(&self) -> usize {
        self.params_u32[3] as usize
    }

    fn rows(&self) -> usize {
        (self.params_u32[1] as usize).div_ceil(self.tile())
    }

    fn upload_bytes(&self) -> Result<usize, MetalError> {
        let scalars = [
            self.params_u32.len(),
            self.params_f32.len(),
            self.triangles.len(),
            self.draw_u32.len(),
            self.draw_f32.len(),
            self.tile_offsets.len(),
            self.tile_draws.len(),
        ]
        .into_iter()
        .try_fold(0usize, |sum, len| sum.checked_add(len))
        .ok_or(MetalError::SizeOverflow("3D upload scalar count"))?;
        checked_bytes(scalars, 4, "3D upload bytes")
    }
}

fn push_three_d_triangle(out: &mut Vec<f32>, triangle: &crate::three_d::RasterTriangle) {
    for vertex in triangle.vertices {
        out.extend_from_slice(&[
            vertex.screen[0] as f32,
            vertex.screen[1] as f32,
            vertex.inverse_w as f32,
            vertex.ndc_z as f32,
            vertex.attributes.world[0] as f32,
            vertex.attributes.world[1] as f32,
            vertex.attributes.world[2] as f32,
            vertex.attributes.normal[0] as f32,
            vertex.attributes.normal[1] as f32,
            vertex.attributes.normal[2] as f32,
            vertex.attributes.color[0] as f32,
            vertex.attributes.color[1] as f32,
            vertex.attributes.color[2] as f32,
            vertex.attributes.color[3] as f32,
            vertex.attributes.uv[0] as f32,
            vertex.attributes.uv[1] as f32,
            vertex.attributes.opacity as f32,
        ]);
    }
}

fn three_d_shader_slab(shader: Shader<'_>) -> [f32; THREE_D_DRAW_F32_STRIDE] {
    let mut slab = [0.0; THREE_D_DRAW_F32_STRIDE];
    if let Shader::Dot {
        center,
        radius,
        color,
        glow_factor,
        shading,
        to_camera,
        scaled_aa_width,
        light_position,
        camera_position,
    } = shader
    {
        slab[0..3].copy_from_slice(&center.map(|value| value as f32));
        slab[3] = radius as f32;
        slab[4..8].copy_from_slice(&[
            color.r as f32,
            color.g as f32,
            color.b as f32,
            color.a as f32,
        ]);
        slab[8] = glow_factor as f32;
        slab[9..12].copy_from_slice(&shading.map(|value| value as f32));
        slab[12..15].copy_from_slice(&to_camera.map(|value| value as f32));
        slab[15] = scaled_aa_width as f32;
        slab[16..19].copy_from_slice(&light_position.map(|value| value as f32));
        slab[19..22].copy_from_slice(&camera_position.map(|value| value as f32));
    }
    slab
}

fn stable_three_d_bins(
    bounds: &[Option<[f64; 4]>],
    width: u32,
    height: u32,
    tile: u32,
) -> Result<(Vec<u32>, Vec<u32>), MetalError> {
    let cols = width.div_ceil(tile) as usize;
    let rows = height.div_ceil(tile) as usize;
    let tile_count = cols
        .checked_mul(rows)
        .ok_or(MetalError::SizeOverflow("3D tile count"))?;
    let mut counts = vec![0usize; tile_count];
    for &draw_bounds in bounds {
        visit_three_d_tiles(draw_bounds, width, height, tile, |index| {
            counts[index] = counts[index]
                .checked_add(1)
                .ok_or(MetalError::SizeOverflow("3D tile command count"))?;
            Ok(())
        })?;
    }

    let mut tile_offsets = Vec::with_capacity(tile_count + 1);
    tile_offsets.push(0);
    let mut total = 0usize;
    for &count in &counts {
        total = total
            .checked_add(count)
            .ok_or(MetalError::SizeOverflow("3D tile command total"))?;
        tile_offsets.push(
            u32::try_from(total).map_err(|_| MetalError::SizeOverflow("3D tile command offset"))?,
        );
    }
    let mut tile_draws = vec![0u32; total];
    let mut cursors = tile_offsets[..tile_count]
        .iter()
        .map(|&offset| offset as usize)
        .collect::<Vec<_>>();
    for (draw, &draw_bounds) in bounds.iter().enumerate() {
        let draw = u32::try_from(draw).map_err(|_| MetalError::SizeOverflow("3D draw index"))?;
        visit_three_d_tiles(draw_bounds, width, height, tile, |index| {
            let cursor = cursors[index];
            tile_draws[cursor] = draw;
            cursors[index] = cursor + 1;
            Ok(())
        })?;
    }
    Ok((tile_offsets, tile_draws))
}

fn visit_three_d_tiles(
    bounds: Option<[f64; 4]>,
    width: u32,
    height: u32,
    tile: u32,
    mut visit: impl FnMut(usize) -> Result<(), MetalError>,
) -> Result<(), MetalError> {
    let Some(bounds) = bounds else {
        return Ok(());
    };
    if bounds.iter().any(|value| !value.is_finite()) {
        return Err(MetalError::Layout("non-finite 3D draw bounds"));
    }
    let cols = width.div_ceil(tile) as usize;
    let rows = height.div_ceil(tile) as usize;
    let candidate = |minimum: f64, maximum: f64, cells: usize| {
        let scale = f64::from(tile);
        let lo = (minimum / scale).floor() - 1.0;
        let hi = (maximum / scale).ceil() + 1.0;
        let clamp = |value: f64| {
            if value <= 0.0 {
                0
            } else if value >= cells as f64 {
                cells
            } else {
                value as usize
            }
        };
        clamp(lo)..clamp(hi)
    };
    let xs = candidate(bounds[0], bounds[2], cols);
    let ys = candidate(bounds[1], bounds[3], rows);
    for y in ys {
        for x in xs.clone() {
            let x0 = (x as u32).saturating_mul(tile);
            let y0 = (y as u32).saturating_mul(tile);
            let rectangle = [
                x0,
                y0,
                x0.saturating_add(tile).min(width),
                y0.saturating_add(tile).min(height),
            ];
            if bounds[2] >= f64::from(rectangle[0])
                && bounds[3] >= f64::from(rectangle[1])
                && bounds[0] <= f64::from(rectangle[2])
                && bounds[1] <= f64::from(rectangle[3])
            {
                visit(y * cols + x)?;
            }
        }
    }
    Ok(())
}

fn push_segment(
    out: &mut Vec<f32>,
    segment: &Segment,
    config: FrameConfig,
    translate: [f64; 2],
    straight_hint: bool,
) {
    let screen = |point: [f64; 3]| {
        let q = config.map.to_pixel(point[0], point[1]);
        [q[0] + translate[0], q[1] + translate[1]]
    };
    let p0 = screen(segment.p0);
    let p1 = screen(segment.p1);
    let p2 = screen(segment.p2);
    out.extend_from_slice(&[
        p0[0] as f32,
        p0[1] as f32,
        p1[0] as f32,
        p1[1] as f32,
        p2[0] as f32,
        p2[1] as f32,
        segment.s0 as f32,
        segment.s1 as f32,
    ]);

    let total = fmn_geom::arclength::quadratic_arc_length(segment.p0, segment.p1, segment.p2);
    for index in 0..=SEGMENT_ARC_INTERVALS {
        let t = index as f64 / SEGMENT_ARC_INTERVALS as f64;
        let fraction = if total > 0.0 {
            let partial =
                fmn_geom::bezier::partial_quadratic(&[segment.p0, segment.p1, segment.p2], 0.0, t);
            (fmn_geom::arclength::quadratic_arc_length(partial[0], partial[1], partial[2]) / total)
                .clamp(0.0, 1.0)
        } else {
            0.0
        };
        out.push(fraction as f32);
    }
    out.push(
        if crate::stroke::line_approximation_admitted(segment, config.map, straight_hint) {
            1.0
        } else {
            0.0
        },
    );
}

fn scalar_row(slice: &[f32], stride: usize, name: &'static str) -> Result<u32, MetalError> {
    if !slice.len().is_multiple_of(stride) {
        return Err(MetalError::Layout(name));
    }
    u32::try_from(slice.len() / stride).map_err(|_| MetalError::SizeOverflow(name))
}

fn checked_bytes(count: usize, width: usize, name: &'static str) -> Result<usize, MetalError> {
    count
        .checked_mul(width)
        .ok_or(MetalError::SizeOverflow(name))
}

fn nonempty_f32(values: &[f32]) -> &[f32] {
    if values.is_empty() { &[0.0] } else { values }
}

fn nonempty_u32(values: &[u32]) -> &[u32] {
    if values.is_empty() { &[0] } else { values }
}

const fn format_name(format: PixelFormat) -> &'static str {
    match format {
        PixelFormat::Rgba8 => "rgba8",
        PixelFormat::Bgra8 => "bgra8",
        PixelFormat::Rgba16F => "rgba16f",
        PixelFormat::Nv12 => "nv12",
        PixelFormat::P010 => "p010",
    }
}

const fn color_range_name(range: Option<ColorRange>) -> &'static str {
    match range {
        Some(ColorRange::Limited) => "limited",
        Some(ColorRange::Full) => "full",
        None => "none",
    }
}

const fn chroma_siting_name(siting: Option<ChromaSiting>) -> &'static str {
    match siting {
        Some(ChromaSiting::Left) => "left",
        Some(ChromaSiting::Center) => "center",
        None => "none",
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_os = "macos"))]
    use fmn_core::color::LinearRgba;
    use fmn_core::color::Srgb;
    #[cfg(not(target_os = "macos"))]
    use fmn_mobject::Stage;

    use super::*;
    #[cfg(not(target_os = "macos"))]
    use crate::bin::{ScreenMap, Tiling, Viewport};
    use crate::camera::{Camera, CameraConfig};
    use crate::three_d::{SurfaceDraw, SurfaceMesh, SurfaceVertex, ThreeDDraw, TrueDotDraw};

    #[test]
    fn the_shader_mirrors_every_host_stride_and_entry_point() {
        for (name, value) in [
            ("SEGMENT_STRIDE", SEGMENT_STRIDE),
            ("SEGMENT_ARC_INTERVALS", SEGMENT_ARC_INTERVALS),
            ("PIECE_STRIDE", PIECE_STRIDE),
            ("JOIN_STRIDE", JOIN_STRIDE),
            ("STATION_STRIDE", STATION_STRIDE),
            ("DRAW_U32_STRIDE", DRAW_U32_STRIDE),
            ("DRAW_F32_STRIDE", DRAW_F32_STRIDE),
            ("STYLE_STRIDE", STYLE_STRIDE),
            ("THREE_D_VERTEX_STRIDE", THREE_D_VERTEX_STRIDE),
            ("THREE_D_TRIANGLE_STRIDE", THREE_D_TRIANGLE_STRIDE),
            ("THREE_D_DRAW_U32_STRIDE", THREE_D_DRAW_U32_STRIDE),
            ("THREE_D_DRAW_F32_STRIDE", THREE_D_DRAW_F32_STRIDE),
            (
                "TRANSFER_TABLE_WORDS_PER_PLANE",
                TRANSFER_TABLE_WORDS_PER_PLANE,
            ),
        ] {
            assert!(
                KERNEL_SOURCE.contains(&format!("#define {name} {value}")),
                "shader is missing the mirrored `{name}` stride"
            );
        }
        for kernel in [
            RASTER_KERNEL,
            THREE_D_KERNEL,
            RGBA8_KERNEL,
            NV12_KERNEL,
            P010_KERNEL,
        ] {
            assert!(
                KERNEL_SOURCE.contains(&format!("kernel void {kernel}(")),
                "shader is missing `{kernel}`"
            );
        }
    }

    #[test]
    fn three_d_packet_keeps_prepared_triangles_and_stable_painter_bins() {
        let black = Srgb::from_rgb8(0, 0, 0).to_linear(1.0);
        let white = Srgb::from_rgb8(255, 255, 255).to_linear(0.8);
        let camera = Camera::new(CameraConfig {
            resolution: (64, 48),
            samples: 2,
            background: black,
            ..CameraConfig::default()
        })
        .expect("camera");
        let mesh = SurfaceMesh::new(
            vec![
                SurfaceVertex::colored([-2.0, -1.0, 0.0], [0.0, 0.0, 1.0], white),
                SurfaceVertex::colored([2.0, -1.0, 0.0], [0.0, 0.0, 1.0], white),
                SurfaceVertex::colored([0.0, 1.5, 0.0], [0.0, 0.0, 1.0], white),
            ],
            vec![0, 1, 2],
        )
        .expect("surface");
        let draws = [
            ThreeDDraw::Surface(SurfaceDraw::new(&mesh)),
            ThreeDDraw::TrueDot(TrueDotDraw::glow([-0.5, 0.0, 1.0], 0.7, white)),
            ThreeDDraw::TrueDot(TrueDotDraw::new([0.5, 0.0, 1.2], 0.4, white)),
        ];
        let job = ThreeDJob::new(
            &camera,
            &draws,
            crate::bin::Tiling {
                macro_tile: 32,
                fine_tile: 8,
            },
        )
        .expect("prepared 3D job");
        let flat = FlatThreeDFrame::derive(&job).expect("Metal packet");

        assert_eq!(flat.params_u32, [64, 48, 8, 8, 2, 3]);
        assert_eq!(
            flat.triangles.len() / THREE_D_TRIANGLE_STRIDE,
            5,
            "one surface triangle plus two billboard triangles per dot"
        );
        assert_eq!(
            flat.draw_u32
                .as_chunks::<THREE_D_DRAW_U32_STRIDE>()
                .0
                .iter()
                .map(|draw| draw[2])
                .collect::<Vec<_>>(),
            [
                THREE_D_SHADER_GOURAUD,
                THREE_D_SHADER_DOT,
                THREE_D_SHADER_DOT
            ]
        );
        assert!(
            flat.tile_offsets.windows(2).all(|range| {
                flat.tile_draws[range[0] as usize..range[1] as usize]
                    .windows(2)
                    .all(|pair| pair[0] <= pair[1])
            }),
            "stable scatter must never invert painter order"
        );
        assert!(flat.upload_bytes().expect("bounded packet") > 0);
    }

    #[test]
    fn backend_identity_excludes_measurement_noise() {
        let base = MetalReport {
            identity: EngineIdentity::metal(),
            device: "fixture".to_owned(),
            unified_memory: true,
            math_mode: "safe",
            kernel_digest: sha256(b"kernel"),
            transfer_table_digest: sha256(b"transfer table"),
            threads_per_threadgroup: 256,
            max_threads_per_threadgroup: 1024,
            thread_execution_width: 32,
            upload_bytes: 10,
            readback_bytes: 20,
            raw_surface_reused: false,
            output_surface_reused: false,
            elapsed: Duration::from_millis(2),
            output_format: PixelFormat::Rgba8,
            color_range: None,
            chroma_siting: None,
        };
        let mut observed = base.clone();
        observed.upload_bytes = 999;
        observed.elapsed = Duration::from_secs(3);
        observed.raw_surface_reused = true;
        assert_eq!(base.backend_digest(), observed.backend_digest());
        observed.device.push_str("-other");
        assert_ne!(base.backend_digest(), observed.backend_digest());
        let mut yuv = base.clone();
        yuv.output_format = PixelFormat::Nv12;
        yuv.color_range = Some(ColorRange::Limited);
        yuv.chroma_siting = Some(ChromaSiting::Left);
        assert_ne!(base.backend_digest(), yuv.backend_digest());
    }

    #[test]
    fn native_preview_validates_dimensions_before_platform_probe() {
        let error =
            NativePreviewRenderer::new(0, 16, "invalid").expect_err("zero width is rejected");
        assert!(matches!(
            error,
            NativePreviewError::Presentation(PresentationError::InvalidDimensions {
                width: 0,
                height: 16
            })
        ));
    }

    #[test]
    fn packed_transfer_table_is_the_authoritative_reel_table() {
        let (words, digest) = build_transfer_table();
        assert_eq!(words.len(), TRANSFER_TABLE_WORDS);
        let tables = fmn_frame::transfer::tables();
        let unpack = |plane: usize, bits: u16| {
            let index = bits as usize;
            let word = words[plane * TRANSFER_TABLE_WORDS_PER_PLANE + index / 4];
            ((word >> ((index % 4) * 8)) & 0xff) as u8
        };
        for bits in [0, 1, 0x3555, 0x3c00, 0x7c00, 0x7e00, u16::MAX] {
            assert_eq!(unpack(0, bits), tables.srgb8_from_f16(bits));
            assert_eq!(unpack(1, bits), tables.linear8_from_f16(bits));
        }
        let rebuilt_bytes = words
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect::<Vec<_>>();
        assert_eq!(digest, sha256(&rebuilt_bytes));
    }

    #[test]
    fn preview_fallback_is_reserved_for_backend_failures() {
        assert!(
            MetalError::IncompleteDispatch {
                kernel: RASTER_KERNEL,
                completed: 0,
                expected: 1,
            }
            .permits_preview_fallback()
        );
        assert!(
            MetalError::ThreadgroupTooLarge {
                kernel: RASTER_KERNEL,
                requested: 256,
                maximum: 128,
            }
            .permits_preview_fallback()
        );
        assert!(!MetalError::SizeOverflow("fixture").permits_preview_fallback());
        assert!(
            !MetalError::UnsupportedThreeDPrimitive {
                draw: 0,
                primitive: "fixture",
            }
            .permits_preview_fallback(),
            "frame-local capability gaps must not demote a healthy device"
        );
        assert_eq!(
            NativePreviewError::Presentation(PresentationError::Unavailable).stream_fallback(),
            Some(PreviewFallback::Unavailable)
        );
        assert!(matches!(
            NativePreviewError::Presentation(PresentationError::WrongThread).stream_fallback(),
            Some(PreviewFallback::BackendFailure(_))
        ));
        assert!(
            NativePreviewError::Presentation(PresentationError::InvalidDimensions {
                width: 0,
                height: 1,
            })
            .stream_fallback()
            .is_none(),
            "caller layout errors must not be mislabeled as backend fallback"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn unsupported_three_d_packet_falls_back_without_demoting_metal() {
        use crate::texture::{Texture, TextureEncoding};

        if !MetalRenderer::is_available() {
            return;
        }
        let camera = Camera::new(CameraConfig {
            resolution: (16, 16),
            samples: 1,
            background: Srgb::from_rgb8(0, 0, 0).to_linear(1.0),
            ..CameraConfig::default()
        })
        .expect("camera");
        let texture = Texture::from_rgba8(1, 1, &[255, 128, 32, 255], TextureEncoding::Linear)
            .expect("texture");
        let normal = [0.0, 0.0, 1.0];
        let mesh = SurfaceMesh::from_uv_grid(
            vec![
                SurfaceVertex::textured([-4.0, 3.0, 0.0], normal, [0.0, 0.0], 1.0),
                SurfaceVertex::textured([-4.0, -3.0, 0.0], normal, [0.0, 1.0], 1.0),
                SurfaceVertex::textured([4.0, 3.0, 0.0], normal, [1.0, 0.0], 1.0),
                SurfaceVertex::textured([4.0, -3.0, 0.0], normal, [1.0, 1.0], 1.0),
            ],
            (2, 2),
        )
        .expect("quad");
        let draws = [ThreeDDraw::Surface(SurfaceDraw::image(&mesh, &texture))];
        let job = ThreeDJob::new(&camera, &draws, crate::bin::Tiling::default()).expect("job");
        let expected = render_cpu_three_d_rgba8(&job, 1).expect("CPU frame");
        let mut renderer = PreviewRenderer::new().expect("Metal preview renderer");
        let preview = renderer
            .render_three_d(&job, 1)
            .expect("unsupported texture uses CPU for this frame");

        assert!(matches!(
            preview.route,
            PreviewRoute::FastCpu(PreviewFallback::Unsupported(_))
        ));
        assert_eq!(preview.frame.as_bytes(), expected.as_bytes());
        assert!(
            matches!(renderer, PreviewRenderer::Metal(_)),
            "a frame-local capability gap must not demote the device"
        );
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn yuv_output_validation_refuses_semantic_mismatches_before_dispatch() {
        let config = FrameConfig::new(
            Viewport {
                width: 16,
                height: 16,
            },
            ScreenMap {
                scale: 1.0,
                origin: [8.0, 8.0],
                y_up: false,
            },
            LinearRgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        );
        let p010 = FrameLayout::tight(PixelFormat::P010, 16, 16).expect("valid P010 layout");
        assert!(matches!(
            validate_yuv_output(&p010, config, PixelFormat::P010, ColorRange::Full),
            Err(MetalError::Frame(FrameError::UnsupportedConversion(
                "P010 output is limited-range only"
            )))
        ));

        let rgba = FrameLayout::tight(PixelFormat::Rgba8, 16, 16).expect("valid RGBA layout");
        assert!(matches!(
            validate_yuv_output(&rgba, config, PixelFormat::Nv12, ColorRange::Limited),
            Err(MetalError::Frame(FrameError::FormatMismatch {
                expected: "Nv12 destination",
                got: PixelFormat::Rgba8,
            }))
        ));

        let other_size = FrameLayout::tight(PixelFormat::Nv12, 18, 16).expect("valid NV12 layout");
        assert!(matches!(
            validate_yuv_output(&other_size, config, PixelFormat::Nv12, ColorRange::Limited,),
            Err(MetalError::Frame(FrameError::DimensionMismatch))
        ));
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn unavailable_metal_renders_through_the_truthful_cpu_preview_route() {
        let mut renderer = PreviewRenderer::new().expect("unavailability is a supported state");
        assert!(matches!(
            renderer,
            PreviewRenderer::FastCpu(PreviewFallback::Unavailable)
        ));

        let stage = Stage::new();
        let config = FrameConfig::new(
            Viewport {
                width: 16,
                height: 16,
            },
            ScreenMap {
                scale: 1.0,
                origin: [8.0, 8.0],
                y_up: false,
            },
            LinearRgba {
                r: 0.04,
                g: 0.05,
                b: 0.06,
                a: 1.0,
            },
        );
        let mut plan = RenderPlan::new();
        plan.sync(&stage, 0).expect("valid Metal fixture");
        let mono = MonoTable::build(&plan, config.map).expect("bounded Metal monotone table");
        let binning = Binning::build(&plan, config.viewport, Tiling::default(), config.map)
            .expect("bounded test binning");
        let expected =
            render_cpu_rgba8(&plan, &mono, &binning, config, 1).expect("CPU comparison frame");
        let preview = renderer
            .render(&plan, &mono, &binning, config, 1)
            .expect("CPU fallback renders a preview frame");

        assert_eq!(
            preview.route,
            PreviewRoute::FastCpu(PreviewFallback::Unavailable)
        );
        assert_eq!(preview.identity(), EngineIdentity::fast());
        assert!(preview.metal.is_none());
        assert_eq!(preview.frame.as_bytes(), expected.as_bytes());

        let camera = Camera::new(CameraConfig {
            resolution: (16, 16),
            samples: 1,
            background: config.background,
            ..CameraConfig::default()
        })
        .expect("3D camera");
        let dot = TrueDotDraw::glow(
            [0.0, 0.0, 0.0],
            0.5,
            Srgb::from_rgb8(255, 128, 32).to_linear(0.8),
        );
        let draws = [ThreeDDraw::TrueDot(dot)];
        let job = ThreeDJob::new(&camera, &draws, Tiling::default()).expect("3D job");
        let expected = render_cpu_three_d_rgba8(&job, 1).expect("3D CPU comparison frame");
        let preview = renderer
            .render_three_d(&job, 1)
            .expect("CPU fallback renders a 3D preview frame");
        assert_eq!(
            preview.route,
            PreviewRoute::FastCpu(PreviewFallback::Unavailable)
        );
        assert_eq!(preview.frame.as_bytes(), expected.as_bytes());
    }
}
