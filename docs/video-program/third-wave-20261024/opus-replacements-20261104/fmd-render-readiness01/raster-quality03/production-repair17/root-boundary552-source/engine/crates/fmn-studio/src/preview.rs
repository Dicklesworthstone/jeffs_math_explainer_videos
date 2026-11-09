//! Studio's native-presentation and CPU-stream composition boundary.
//!
//! A native `CAMetalLayer` can consume Lumen's lifetime-held RGBA8 Metal
//! surface without frame-pixel readback. Browser multipart-PNG and kitty/sixel
//! previews cannot: they require CPU-visible bytes. This module keeps those
//! routes distinct and reports which one actually served each frame.

use std::fmt;

use fmn_render::metal::{
    MetalError, NativePreviewError, NativePreviewRenderer, NativePreviewReport,
    PresentationPipelineInfo, PresentationState, PreviewFallback, PreviewFrame, PreviewRenderer,
};
use fmn_render::{Binning, EngineIdentity, FrameConfig, MonoTable, RenderPlan, ThreeDJob};

/// Construction policy for Studio's preferred native preview route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioPreviewConfig {
    /// Initial native drawable width.
    pub width: u32,
    /// Initial native drawable height.
    pub height: u32,
    /// Native window title.
    pub title: String,
    /// Threads assigned by the caller's standard-mode execution plan when the
    /// CPU stream fallback runs.
    pub cpu_threads: usize,
}

impl StudioPreviewConfig {
    /// Construct an explicit Studio preview policy.
    pub fn new(width: u32, height: u32, title: impl Into<String>, cpu_threads: usize) -> Self {
        Self {
            width,
            height,
            title: title.into(),
            cpu_threads,
        }
    }
}

impl Default for StudioPreviewConfig {
    fn default() -> Self {
        Self::new(1280, 720, "FrankenManim Studio", 1)
    }
}

/// Route currently selected by the Studio preview owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioPreviewRoute {
    /// Native Metal rendering into a `CAMetalLayer`, with no frame-pixel
    /// readback.
    NativeMetal,
    /// CPU-visible RGBA8 bytes for multipart-PNG or a terminal protocol.
    CpuStream(PreviewFallback),
}

/// Observable result of one Studio preview frame.
#[derive(Debug)]
pub enum StudioPreviewOutput {
    /// Native presentation completed or was temporarily occluded.
    Native(NativePreviewReport),
    /// A CPU-visible frame is ready for browser or terminal publication.
    Stream(PreviewFrame),
}

impl StudioPreviewOutput {
    /// Engine that truthfully produced this output.
    #[must_use]
    pub fn identity(&self) -> EngineIdentity {
        match self {
            Self::Native(report) => report.metal.identity,
            Self::Stream(frame) => frame.identity(),
        }
    }

    /// Consume a CPU-visible stream output, or return `None` for native
    /// presentation.
    #[must_use]
    pub fn into_stream(self) -> Option<PreviewFrame> {
        match self {
            Self::Native(_) => None,
            Self::Stream(frame) => Some(frame),
        }
    }
}

/// Studio preview construction or frame failure.
#[derive(Debug)]
pub enum StudioPreviewError {
    /// CPU fallback requires at least one execution thread.
    InvalidCpuThreads,
    /// The native renderer or presenter rejected a non-fallback condition.
    Native(NativePreviewError),
    /// The declared CPU stream could not produce its frame.
    Stream(MetalError),
}

impl fmt::Display for StudioPreviewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCpuThreads => {
                write!(f, "Studio CPU preview requires at least one thread")
            }
            Self::Native(error) => error.fmt(f),
            Self::Stream(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for StudioPreviewError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidCpuThreads => None,
            Self::Native(error) => Some(error),
            Self::Stream(error) => Some(error),
        }
    }
}

impl From<NativePreviewError> for StudioPreviewError {
    fn from(error: NativePreviewError) -> Self {
        Self::Native(error)
    }
}

enum StudioPreviewBackend {
    Native(Box<NativePreviewRenderer>),
    CpuStream {
        renderer: PreviewRenderer,
        reason: PreviewFallback,
    },
}

impl fmt::Debug for StudioPreviewBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Native(renderer) => f.debug_tuple("Native").field(renderer).finish(),
            Self::CpuStream { renderer, reason } => f
                .debug_struct("CpuStream")
                .field("renderer", renderer)
                .field("reason", reason)
                .finish(),
        }
    }
}

/// Concrete Studio preview owner.
///
/// Supported main-thread Apple sessions start on native Metal. Target
/// unavailability and presentation construction failures select the tested
/// fast-CPU stream. A later native backend or presentation failure permanently
/// demotes the owner before returning the CPU frame for that same call.
#[derive(Debug)]
pub struct StudioPreviewRenderer {
    backend: StudioPreviewBackend,
    cpu_threads: usize,
}

impl StudioPreviewRenderer {
    /// Prefer native Metal presentation, with an explicit CPU-stream fallback.
    pub fn new(config: StudioPreviewConfig) -> Result<Self, StudioPreviewError> {
        if config.cpu_threads == 0 {
            return Err(StudioPreviewError::InvalidCpuThreads);
        }
        let backend = match NativePreviewRenderer::new(config.width, config.height, config.title) {
            Ok(renderer) => StudioPreviewBackend::Native(Box::new(renderer)),
            Err(error) => {
                let Some(reason) = error.construction_stream_fallback() else {
                    return Err(error.into());
                };
                Self::cpu_backend(reason)
            }
        };
        Ok(Self {
            backend,
            cpu_threads: config.cpu_threads,
        })
    }

    /// Currently selected route.
    #[must_use]
    pub fn route(&self) -> StudioPreviewRoute {
        match &self.backend {
            StudioPreviewBackend::Native(_) => StudioPreviewRoute::NativeMetal,
            StudioPreviewBackend::CpuStream { reason, .. } => {
                StudioPreviewRoute::CpuStream(reason.clone())
            }
        }
    }

    /// Native drawable-pipeline occupancy, when the native route is active.
    #[must_use]
    pub fn presentation_pipeline_info(&self) -> Option<PresentationPipelineInfo> {
        match &self.backend {
            StudioPreviewBackend::Native(renderer) => Some(renderer.presentation_pipeline_info()),
            StudioPreviewBackend::CpuStream { .. } => None,
        }
    }

    /// Drain native events. CPU stream owners have no native event queue.
    pub fn poll_events(&mut self) -> Result<Option<PresentationState>, StudioPreviewError> {
        match &mut self.backend {
            StudioPreviewBackend::Native(renderer) => {
                renderer.poll_events().map(Some).map_err(Into::into)
            }
            StudioPreviewBackend::CpuStream { .. } => Ok(None),
        }
    }

    /// Render one 2D preview through the currently selected route.
    pub fn render(
        &mut self,
        plan: &RenderPlan,
        mono: &MonoTable,
        binning: &Binning,
        config: FrameConfig,
    ) -> Result<StudioPreviewOutput, StudioPreviewError> {
        let fallback = match &mut self.backend {
            StudioPreviewBackend::Native(renderer) => {
                match renderer.render(plan, mono, binning, config) {
                    Ok(report) => return Ok(StudioPreviewOutput::Native(report)),
                    Err(error) => {
                        let Some(reason) = error.stream_fallback() else {
                            return Err(error.into());
                        };
                        reason
                    }
                }
            }
            StudioPreviewBackend::CpuStream { renderer, .. } => {
                return renderer
                    .render(plan, mono, binning, config, self.cpu_threads)
                    .map(StudioPreviewOutput::Stream)
                    .map_err(StudioPreviewError::Stream);
            }
        };
        self.backend = Self::cpu_backend(fallback);
        self.render_cpu(plan, mono, binning, config)
    }

    /// Render one prepared 3D preview through the selected route.
    ///
    /// An unsupported prepared primitive uses the CPU stream for this frame
    /// only. Backend and presentation failures permanently demote the owner.
    pub fn render_three_d(
        &mut self,
        job: &ThreeDJob<'_>,
    ) -> Result<StudioPreviewOutput, StudioPreviewError> {
        let (fallback, transient) = match &mut self.backend {
            StudioPreviewBackend::Native(renderer) => match renderer.render_three_d(job) {
                Ok(report) => return Ok(StudioPreviewOutput::Native(report)),
                Err(error) => {
                    let Some(reason) = error.stream_fallback() else {
                        return Err(error.into());
                    };
                    (reason, error.is_frame_local_unsupported())
                }
            },
            StudioPreviewBackend::CpuStream { renderer, .. } => {
                return renderer
                    .render_three_d(job, self.cpu_threads)
                    .map(StudioPreviewOutput::Stream)
                    .map_err(StudioPreviewError::Stream);
            }
        };
        if transient {
            let mut renderer = PreviewRenderer::FastCpu(fallback);
            return renderer
                .render_three_d(job, self.cpu_threads)
                .map(StudioPreviewOutput::Stream)
                .map_err(StudioPreviewError::Stream);
        }
        self.backend = Self::cpu_backend(fallback);
        self.render_cpu_three_d(job)
    }

    /// Idempotently close an active native preview surface.
    pub fn close(&mut self) -> Result<(), StudioPreviewError> {
        match &mut self.backend {
            StudioPreviewBackend::Native(renderer) => renderer.close().map_err(Into::into),
            StudioPreviewBackend::CpuStream { .. } => Ok(()),
        }
    }

    fn cpu_backend(reason: PreviewFallback) -> StudioPreviewBackend {
        StudioPreviewBackend::CpuStream {
            renderer: PreviewRenderer::FastCpu(reason.clone()),
            reason,
        }
    }

    fn render_cpu(
        &mut self,
        plan: &RenderPlan,
        mono: &MonoTable,
        binning: &Binning,
        config: FrameConfig,
    ) -> Result<StudioPreviewOutput, StudioPreviewError> {
        let StudioPreviewBackend::CpuStream { renderer, .. } = &mut self.backend else {
            return Err(StudioPreviewError::Native(NativePreviewError::Render(
                MetalError::Layout("Studio CPU fallback was not installed"),
            )));
        };
        renderer
            .render(plan, mono, binning, config, self.cpu_threads)
            .map(StudioPreviewOutput::Stream)
            .map_err(StudioPreviewError::Stream)
    }

    fn render_cpu_three_d(
        &mut self,
        job: &ThreeDJob<'_>,
    ) -> Result<StudioPreviewOutput, StudioPreviewError> {
        let StudioPreviewBackend::CpuStream { renderer, .. } = &mut self.backend else {
            return Err(StudioPreviewError::Native(NativePreviewError::Render(
                MetalError::Layout("Studio CPU fallback was not installed"),
            )));
        };
        renderer
            .render_three_d(job, self.cpu_threads)
            .map(StudioPreviewOutput::Stream)
            .map_err(StudioPreviewError::Stream)
    }
}
