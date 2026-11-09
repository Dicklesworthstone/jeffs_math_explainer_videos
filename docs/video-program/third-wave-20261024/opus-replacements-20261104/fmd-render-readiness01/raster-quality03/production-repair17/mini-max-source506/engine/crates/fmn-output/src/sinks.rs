//! Production adapters from [`crate::FrameSink`] to Reel artifacts.
//!
//! The codecs and ffmpeg boundary deliberately own byte production, while
//! this module owns composition concerns that neither lower layer can infer:
//! ordered sequence validation, stride removal, explicit resource budgets,
//! cancellation-safe finalization, atomic native publication, and completion
//! receipts for the CLI/provenance layer.
//!
//! Every adapter streams into an unpublished staging destination as frames arrive. The
//! emitter first prepares every sink, then grants one publication phase
//! linearized against cancellation. GIF/y4m and ffmpeg retain no whole-render
//! payload. PNG sequences publish as one immutable no-clobber directory
//! generation whose completion marker is written last, so partial runs, stale
//! tails, and concurrent runs cannot mix.

use crate::{
    Boundary, BoundaryError, BoundaryReport, DitherPolicy, EncoderCapabilities, FfmpegTool,
    FrameSink, JobLimits, MixReport, PreparedFfmpegArtifact, SinkBinding, SinkFailure, SinkWrite,
    StreamingEncode, VideoJob,
};
use fmn_codec::png::encode_rgba8_segmented_parallel;
use fmn_codec::y4m::{append_y4m_frame_nv12, y4m_header};
use fmn_codec::{CompressionLevel, GifStreamEncoder, SampleFormat, Y4mColorspace};
use fmn_frame::{FrameBuffer, FrameLayout, PixelFormat};
use fmn_hash::{Digest, Sha256, sha256};
use fmn_platform::clock::Clock;
use fmn_platform::fs::{
    AtomicDirectoryWriter, AtomicFileWriter, FileSystem, PreparedAtomicDirectory,
    PreparedAtomicFile,
};
use fmn_platform::process::{ProcessCancellation, ProcessRunner, ProcessStdinLimits};
use fmn_platform::profile::{
    ProfileLane, ProfileLaneRole, ProfilePath, ProfilePhase, ProfileRecorder, ProfileSpan,
};
use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// Explicit bounds for a production sink.
///
/// `max_resident_bytes` bounds one frame/scratch unit retained by the adapter.
/// `max_stream_bytes` caps cumulative accepted input; it is a logical resource
/// guard, not permission to retain that input. `max_artifact_bytes` caps the
/// complete native artifact or ffmpeg result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SinkLimits {
    max_frames: u64,
    max_resident_bytes: u64,
    max_stream_bytes: u64,
    max_artifact_bytes: u64,
    exact_frames: Option<u64>,
}

impl SinkLimits {
    /// Construct nonzero resource limits.
    ///
    /// # Errors
    ///
    /// Returns [`SinkAdapterError::InvalidConfig`] when any bound is zero.
    pub fn new(
        max_frames: u64,
        max_resident_bytes: u64,
        max_stream_bytes: u64,
        max_artifact_bytes: u64,
    ) -> Result<Self, SinkAdapterError> {
        if max_frames == 0 {
            return Err(SinkAdapterError::InvalidConfig(
                "sink max_frames must be nonzero",
            ));
        }
        if max_resident_bytes == 0 {
            return Err(SinkAdapterError::InvalidConfig(
                "sink max_resident_bytes must be nonzero",
            ));
        }
        if max_stream_bytes == 0 {
            return Err(SinkAdapterError::InvalidConfig(
                "sink max_stream_bytes must be nonzero",
            ));
        }
        if max_artifact_bytes == 0 {
            return Err(SinkAdapterError::InvalidConfig(
                "sink max_artifact_bytes must be nonzero",
            ));
        }
        Ok(Self {
            max_frames,
            max_resident_bytes,
            max_stream_bytes,
            max_artifact_bytes,
            exact_frames: None,
        })
    }

    /// Require successful finalization to contain exactly `frames` frames.
    ///
    /// This closes the otherwise undetectable truncated-stream case when the
    /// composition root knows the render plan's terminal frame count.
    ///
    /// # Errors
    ///
    /// Returns [`SinkAdapterError::InvalidConfig`] for zero or for a count
    /// above [`Self::max_frames`].
    pub fn requiring_exact_frames(mut self, frames: u64) -> Result<Self, SinkAdapterError> {
        if frames == 0 {
            return Err(SinkAdapterError::InvalidConfig(
                "sink exact frame count must be nonzero",
            ));
        }
        if frames > self.max_frames {
            return Err(SinkAdapterError::InvalidConfig(
                "sink exact frame count exceeds max_frames",
            ));
        }
        self.exact_frames = Some(frames);
        Ok(self)
    }

    /// Maximum accepted frame count.
    #[must_use]
    pub const fn max_frames(self) -> u64 {
        self.max_frames
    }

    /// Maximum resident adapter payload.
    #[must_use]
    pub const fn max_resident_bytes(self) -> u64 {
        self.max_resident_bytes
    }

    /// Maximum cumulative input bytes.
    #[must_use]
    pub const fn max_stream_bytes(self) -> u64 {
        self.max_stream_bytes
    }

    /// Maximum complete artifact bytes.
    #[must_use]
    pub const fn max_artifact_bytes(self) -> u64 {
        self.max_artifact_bytes
    }

    /// Required terminal frame count, when the composition root knows it.
    #[must_use]
    pub const fn exact_frames(self) -> Option<u64> {
        self.exact_frames
    }
}

/// Optional output-stage profiling capability.
///
/// A composition root hands the same clock and recorder to the runtime
/// pipeline and these sinks, keeping `emit`/`encode` spans in one
/// monotonic epoch. Omit this value for the zero-cost disabled path.
#[derive(Clone)]
pub struct OutputProfile {
    clock: Arc<dyn Clock>,
    recorder: ProfileRecorder,
    path: ProfilePath,
}

impl OutputProfile {
    /// Bind the run's clock, recorder, and scene/play path.
    #[must_use]
    pub fn new(clock: Arc<dyn Clock>, recorder: ProfileRecorder, path: ProfilePath) -> Self {
        Self {
            clock,
            recorder,
            path,
        }
    }

    /// An encode span on encoder lane `lane`, for a frame-concurrent
    /// sequence encoder (fm-hyqz).
    fn encode_span(&self, frame: u64, lane: u16) -> Option<ProfileSpan<'_>> {
        self.recorder.span(
            self.clock.as_ref(),
            self.path.with_frame(frame),
            ProfilePhase::Encode,
            ProfileLane::new(ProfileLaneRole::Encoder, lane),
        )
    }

    fn span(&self, frame: Option<u64>, phase: ProfilePhase) -> Option<ProfileSpan<'_>> {
        let path = frame.map_or(self.path, |sequence| self.path.with_frame(sequence));
        let lane = if phase == ProfilePhase::Encode {
            ProfileLane::new(ProfileLaneRole::Encoder, 0)
        } else {
            ProfileLane::output()
        };
        self.recorder.span(self.clock.as_ref(), path, phase, lane)
    }
}

impl fmt::Debug for OutputProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OutputProfile")
            .field("recorder", &self.recorder)
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

/// Typed construction, validation, codec, or publication failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SinkAdapterError {
    /// A configuration value cannot describe a production sink.
    InvalidConfig(&'static str),
    /// The configured frame layout cannot exist.
    InvalidGeometry {
        /// Frame-layer diagnostic.
        detail: String,
    },
    /// A frame arrived outside the contiguous configured sequence.
    UnexpectedSequence {
        /// Required next sequence.
        expected: u64,
        /// Supplied sequence.
        actual: u64,
    },
    /// The terminal sequence has no successor.
    SequenceExhausted {
        /// Refused sequence.
        sequence: u64,
    },
    /// More frames were offered than the declared budget.
    FrameLimitExceeded {
        /// Attempted frame count.
        attempted: u64,
        /// Configured maximum.
        max: u64,
    },
    /// Finalization observed a count other than the declared exact count.
    FrameCountMismatch {
        /// Required terminal frame count.
        expected: u64,
        /// Buffered terminal frame count.
        actual: u64,
    },
    /// One resident frame/scratch unit would exceed its declared budget.
    ResidentBytesExceeded {
        /// Attempted resident bytes.
        attempted: u64,
        /// Configured maximum.
        max: u64,
    },
    /// Cumulative input would exceed its declared logical stream budget.
    StreamBytesExceeded {
        /// Attempted cumulative bytes.
        attempted: u64,
        /// Configured maximum.
        max: u64,
    },
    /// Encoded output would exceed its declared budget.
    ArtifactBytesExceeded {
        /// Attempted artifact bytes.
        attempted: u64,
        /// Configured maximum.
        max: u64,
    },
    /// The frame's format or dimensions do not match the sink.
    FrameMismatch {
        /// Required pixel format.
        expected_format: PixelFormat,
        /// Required width.
        expected_width: u32,
        /// Required height.
        expected_height: u32,
        /// Supplied pixel format.
        got_format: PixelFormat,
        /// Supplied width.
        got_width: u32,
        /// Supplied height.
        got_height: u32,
    },
    /// A completed artifact requires at least one frame.
    EmptyStream {
        /// Artifact kind.
        kind: &'static str,
    },
    /// A native codec refused the current frame or stream record.
    Codec {
        /// Codec identity.
        codec: &'static str,
        /// Codec diagnostic.
        detail: String,
    },
    /// Atomic native publication failed.
    Publish {
        /// Intended destination.
        path: PathBuf,
        /// Filesystem diagnostic.
        detail: String,
    },
    /// The negotiated ffmpeg boundary refused the job.
    Ffmpeg {
        /// Boundary diagnostic.
        detail: String,
    },
    /// Cooperative cancellation stopped private sink work.
    Cancelled,
    /// The sink lifecycle was invoked twice.
    AlreadyFinalized,
}

impl fmt::Display for SinkAdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(detail) => write!(f, "output sink configuration: {detail}"),
            Self::InvalidGeometry { detail } => write!(f, "output sink geometry: {detail}"),
            Self::UnexpectedSequence { expected, actual } => {
                write!(f, "output sink expected frame {expected}, got {actual}")
            }
            Self::SequenceExhausted { sequence } => {
                write!(f, "output sink frame {sequence} has no successor")
            }
            Self::FrameLimitExceeded { attempted, max } => {
                write!(f, "output sink frame count {attempted} exceeds limit {max}")
            }
            Self::FrameCountMismatch { expected, actual } => write!(
                f,
                "output sink expected exactly {expected} frames, finalized with {actual}"
            ),
            Self::ResidentBytesExceeded { attempted, max } => write!(
                f,
                "output sink resident payload {attempted} bytes exceeds limit {max}"
            ),
            Self::StreamBytesExceeded { attempted, max } => write!(
                f,
                "output sink cumulative input {attempted} bytes exceeds limit {max}"
            ),
            Self::ArtifactBytesExceeded { attempted, max } => write!(
                f,
                "output sink artifact set {attempted} bytes exceeds limit {max}"
            ),
            Self::FrameMismatch {
                expected_format,
                expected_width,
                expected_height,
                got_format,
                got_width,
                got_height,
            } => write!(
                f,
                "output sink needs {expected_format:?} {expected_width}x{expected_height}, \
                 got {got_format:?} {got_width}x{got_height}"
            ),
            Self::EmptyStream { kind } => {
                write!(f, "cannot publish an empty {kind} stream")
            }
            Self::Codec { codec, detail } => write!(f, "{codec} encode failed: {detail}"),
            Self::Publish { path, detail } => {
                write!(f, "publish {} atomically: {detail}", path.display())
            }
            Self::Ffmpeg { detail } => write!(f, "ffmpeg sink: {detail}"),
            Self::Cancelled => f.write_str("output sink was cancelled"),
            Self::AlreadyFinalized => f.write_str("output sink was already finalized"),
        }
    }
}

impl std::error::Error for SinkAdapterError {}

impl SinkAdapterError {
    /// Stable machine-readable category retained through [`SinkFailure`].
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidConfig(_) => "sink.invalid_config",
            Self::InvalidGeometry { .. } => "sink.invalid_geometry",
            Self::UnexpectedSequence { .. } => "sink.unexpected_sequence",
            Self::SequenceExhausted { .. } => "sink.sequence_exhausted",
            Self::FrameLimitExceeded { .. } => "sink.frame_limit",
            Self::FrameCountMismatch { .. } => "sink.frame_count_mismatch",
            Self::ResidentBytesExceeded { .. } => "sink.resident_limit",
            Self::StreamBytesExceeded { .. } => "sink.stream_limit",
            Self::ArtifactBytesExceeded { .. } => "sink.artifact_limit",
            Self::FrameMismatch { .. } => "sink.frame_mismatch",
            Self::EmptyStream { .. } => "sink.empty_stream",
            Self::Codec { .. } => "sink.codec",
            Self::Publish { .. } => "sink.publish",
            Self::Ffmpeg { .. } => "sink.ffmpeg",
            Self::Cancelled => "sink.cancelled",
            Self::AlreadyFinalized => "sink.already_finalized",
        }
    }
}

/// The result of attempting to consume a completion receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiptError {
    /// The emitter has not finalized the sink yet.
    Pending,
    /// Terminal cancellation/failure aborted publication.
    Aborted,
    /// Finalization failed.
    Failed(SinkFailure),
    /// The successful report was already taken.
    AlreadyTaken,
}

impl fmt::Display for ReceiptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => f.write_str("output sink completion is pending"),
            Self::Aborted => f.write_str("output sink was aborted"),
            Self::Failed(failure) => write!(f, "output sink failed: {failure}"),
            Self::AlreadyTaken => f.write_str("output sink completion was already taken"),
        }
    }
}

impl std::error::Error for ReceiptError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Failed(failure) => Some(failure),
            _ => None,
        }
    }
}

enum ReceiptState<T> {
    Pending,
    Published(T),
    Aborted,
    Failed(SinkFailure),
    Taken,
}

/// A cloneable handle through which composition retrieves a sink's report.
pub struct SinkReceipt<T> {
    inner: Arc<Mutex<ReceiptState<T>>>,
}

impl<T> SinkReceipt<T> {
    fn pending() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ReceiptState::Pending)),
        }
    }

    /// Take the completed report exactly once.
    ///
    /// # Errors
    ///
    /// [`ReceiptError`] describes a pending, aborted, failed, or already
    /// consumed lifecycle.
    pub fn take(&self) -> Result<T, ReceiptError> {
        let mut state = lock(&self.inner);
        let current = std::mem::replace(&mut *state, ReceiptState::Taken);
        match current {
            ReceiptState::Published(report) => Ok(report),
            ReceiptState::Pending => {
                *state = ReceiptState::Pending;
                Err(ReceiptError::Pending)
            }
            ReceiptState::Aborted => {
                *state = ReceiptState::Aborted;
                Err(ReceiptError::Aborted)
            }
            ReceiptState::Failed(failure) => {
                *state = ReceiptState::Failed(failure.clone());
                Err(ReceiptError::Failed(failure))
            }
            ReceiptState::Taken => Err(ReceiptError::AlreadyTaken),
        }
    }

    fn publish(&self, report: T) {
        let mut state = lock(&self.inner);
        if matches!(*state, ReceiptState::Pending) {
            *state = ReceiptState::Published(report);
        }
    }

    fn fail(&self, failure: SinkFailure) {
        let mut state = lock(&self.inner);
        if matches!(*state, ReceiptState::Pending) {
            *state = ReceiptState::Failed(failure);
        }
    }

    fn abort(&self) {
        let mut state = lock(&self.inner);
        if matches!(*state, ReceiptState::Pending) {
            *state = ReceiptState::Aborted;
        }
    }
}

impl<T> Clone for SinkReceipt<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> fmt::Debug for SinkReceipt<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = match &*lock(&self.inner) {
            ReceiptState::Pending => "pending",
            ReceiptState::Published(_) => "published",
            ReceiptState::Aborted => "aborted",
            ReceiptState::Failed(_) => "failed",
            ReceiptState::Taken => "taken",
        };
        f.debug_struct("SinkReceipt")
            .field("status", &status)
            .finish()
    }
}

/// Native artifact identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeArtifactKind {
    /// One canonical PNG.
    Png,
    /// A canonical PNG sequence.
    PngSequence,
    /// Native animated GIF.
    Gif,
    /// Native YUV4MPEG2 stream.
    Y4m,
    /// Native WAV soundtrack.
    Wav,
    /// Native SVG still.
    Svg,
}

/// Successful native frame-artifact publication.
///
/// A PNG sequence is one immutable directory generation: its reserved
/// completion marker is the publication point after every child is durable.
/// The report retains only that root, never one metadata allocation per frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeArtifactReport {
    /// Published format.
    pub kind: NativeArtifactKind,
    /// Successfully published file or immutable sequence-directory root.
    pub path: PathBuf,
    /// Frames represented by the artifact set.
    pub frame_count: u64,
    /// Total published bytes.
    pub bytes: u64,
    /// Raw-file digest, or the canonical ordered tree digest for a PNG sequence.
    pub digest: Digest,
}

/// Successful negotiated ffmpeg publication.
#[derive(Debug)]
pub struct FfmpegArtifactReport {
    /// Boundary provenance and captured stderr.
    pub boundary: BoundaryReport,
    /// Frames handed to ffmpeg.
    pub frame_count: u64,
    /// Tightly packed input bytes handed to ffmpeg.
    pub input_bytes: u64,
}

/// Successful native WAV publication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WavPublicationReport {
    /// Published destination.
    pub path: PathBuf,
    /// Encoded bytes.
    pub bytes: u64,
    /// SHA-256 of the exact published WAV bytes.
    pub digest: Digest,
    /// Interleaved PCM frame count (one sample per channel).
    pub sample_frames: u64,
    /// Mix clipping evidence retained in the report.
    pub clipped_samples: u64,
    /// Ordered cues represented by the mix.
    pub cues_mixed: usize,
}

/// Destination policy for canonical PNG output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PngTarget {
    /// Exactly one frame at this path.
    Single(PathBuf),
    /// One immutable directory generation, claimed no-clobber and published
    /// by a completion marker after every
    /// `{stem}_{sequence:0digits}.png` child is prepared.
    Sequence {
        /// Final generation directory. It must not already exist.
        directory: PathBuf,
        /// Conservative cross-platform safe leaf stem.
        stem: String,
        /// Minimum decimal sequence width.
        digits: usize,
    },
}

/// Canonical PNG sink configuration.
#[derive(Debug, Clone)]
pub struct PngSinkConfig {
    /// Destination policy.
    pub target: PngTarget,
    /// Frame width.
    pub width: u32,
    /// Frame height.
    pub height: u32,
    /// First accepted sequence.
    pub first_sequence: u64,
    /// Deterministic codec effort.
    pub compression: CompressionLevel,
    /// Encoder workers. A sequence encodes up to this many frames at once
    /// (bounded by `max_resident_bytes`), giving each frame the remainder as
    /// fixed-segment workers; a single PNG fans its own segments. Canonical
    /// bytes are invariant at every count.
    pub threads: usize,
    /// Explicit bounds.
    pub limits: SinkLimits,
    /// Optional output profiling.
    pub profile: Option<OutputProfile>,
}

impl PngSinkConfig {
    /// How many sequence frames this configuration encodes at once
    /// (fm-hyqz): up to `threads`, while every in-flight frame's raw and
    /// encoded bytes fit `max_resident_bytes`. One for a single PNG.
    #[must_use]
    pub fn sequence_frame_concurrency(&self) -> usize {
        let frame_bytes = u64::from(self.width)
            .saturating_mul(u64::from(self.height))
            .saturating_mul(4);
        png_frame_concurrency(self, frame_bytes)
    }
}

/// Cancellation-safe canonical PNG / PNG-sequence adapter.
pub struct PngSink {
    fs: Arc<dyn FileSystem>,
    config: PngSinkConfig,
    expected_layout: FrameLayout,
    state: FrameState,
    scratch: Vec<u8>,
    single_encoded: Option<Vec<u8>>,
    directory_writer: Option<Box<dyn AtomicDirectoryWriter>>,
    prepared_file: Option<Box<dyn PreparedAtomicFile>>,
    prepared_directory: Option<Box<dyn PreparedAtomicDirectory>>,
    artifact_bytes: u64,
    artifact_hasher: Sha256,
    receipt: SinkReceipt<NativeArtifactReport>,
    no_clobber: bool,
    /// Frame-concurrent sequence encoding, started at the first frame when
    /// more than one frame may be in flight (fm-hyqz).
    encoders: Option<PngEncodePool>,
}

impl PngSink {
    /// Validate and construct a PNG sink.
    ///
    /// # Errors
    ///
    /// Invalid geometry, destination, stem, digits, or worker count.
    pub fn new(fs: Arc<dyn FileSystem>, config: PngSinkConfig) -> Result<Self, SinkAdapterError> {
        validate_png_target(&config.target)?;
        if config.threads == 0 {
            return Err(SinkAdapterError::InvalidConfig(
                "PNG worker count must be nonzero",
            ));
        }
        if matches!(&config.target, PngTarget::Single(_))
            && config.limits.exact_frames.is_some_and(|frames| frames != 1)
        {
            return Err(SinkAdapterError::InvalidConfig(
                "single PNG exact frame count must be one",
            ));
        }
        let expected_layout = tight_layout(PixelFormat::Rgba8, config.width, config.height)?;
        let state = FrameState::new(config.first_sequence, config.limits);
        state.check_resident(byte_len_from_usize(expected_layout.total_bytes())?)?;
        let scratch_bytes = expected_layout.total_bytes();
        let mut artifact_hasher = Sha256::new();
        if matches!(&config.target, PngTarget::Sequence { .. }) {
            artifact_hasher.update(b"fmn-png-sequence-tree/v1\0");
        }
        Ok(Self {
            fs,
            config,
            expected_layout,
            state,
            scratch: Vec::with_capacity(scratch_bytes),
            single_encoded: None,
            directory_writer: None,
            prepared_file: None,
            prepared_directory: None,
            artifact_bytes: 0,
            artifact_hasher,
            receipt: SinkReceipt::pending(),
            no_clobber: false,
            encoders: None,
        })
    }

    /// Require atomic create-only publication of a single PNG. Sequences
    /// already use immutable create-only directory generations.
    #[must_use]
    pub fn with_no_clobber(mut self) -> Self {
        self.no_clobber = true;
        self
    }

    /// Completion report handle to retain before moving this sink into a binding.
    #[must_use]
    pub fn receipt(&self) -> SinkReceipt<NativeArtifactReport> {
        self.receipt.clone()
    }

    /// Bind this durable adapter reliably and retain its completion receipt.
    #[must_use]
    pub fn into_binding(
        self,
        name: impl Into<String>,
    ) -> (SinkBinding, SinkReceipt<NativeArtifactReport>) {
        let receipt = self.receipt();
        (SinkBinding::reliable(name, self), receipt)
    }

    fn write(&mut self, sequence: u64, frame: &FrameBuffer) -> Result<(), SinkAdapterError> {
        if matches!(&self.config.target, PngTarget::Single(_)) && self.state.frame_count != 0 {
            return Err(SinkAdapterError::FrameLimitExceeded {
                attempted: 2,
                max: 1,
            });
        }
        let frame_bytes = validate_frame(frame, &self.expected_layout)?;
        self.state.check(sequence, frame_bytes)?;
        // The span holds its own handle so the sink stays mutable under it.
        let profile = self.config.profile.clone();
        let _span = profile
            .as_ref()
            .and_then(|profile| profile.span(Some(sequence), ProfilePhase::Emit));
        if self.encoders.is_none() {
            let concurrency = png_frame_concurrency(&self.config, frame_bytes);
            if concurrency > 1 {
                let segment_threads = (self.config.threads / concurrency).max(1);
                self.encoders =
                    PngEncodePool::start(&self.config, concurrency, segment_threads, sequence);
            }
        }
        if self.encoders.is_some() {
            return self.submit_sequence_frame(sequence, frame, frame_bytes);
        }
        self.scratch.clear();
        append_tight(frame, &self.expected_layout, &mut self.scratch);
        let encoded = {
            let _encode = self
                .config
                .profile
                .as_ref()
                .and_then(|profile| profile.span(Some(sequence), ProfilePhase::Encode));
            encode_rgba8_segmented_parallel(
                self.config.width,
                self.config.height,
                &self.scratch,
                self.config.compression,
                self.config.threads,
            )
        };
        let artifact_bytes = self.admit_encoded(frame_bytes, &encoded)?;
        if matches!(self.config.target, PngTarget::Single(_)) {
            self.artifact_hasher.update(&encoded);
            self.single_encoded = Some(encoded);
        } else {
            self.publish_sequence_frame(sequence, &encoded)?;
        }
        self.artifact_bytes = artifact_bytes;
        self.state.commit_frame(frame_bytes);
        Ok(())
    }

    /// The resident and artifact budgets for one encoded frame; returns the
    /// artifact total after it.
    fn admit_encoded(&self, frame_bytes: u64, encoded: &[u8]) -> Result<u64, SinkAdapterError> {
        let encoded_bytes = byte_len(encoded)?;
        let resident = frame_bytes.checked_add(encoded_bytes).ok_or(
            SinkAdapterError::ResidentBytesExceeded {
                attempted: u64::MAX,
                max: self.config.limits.max_resident_bytes,
            },
        )?;
        self.state.check_resident(resident)?;
        let artifact_bytes = self.artifact_bytes.checked_add(encoded_bytes).ok_or(
            SinkAdapterError::ArtifactBytesExceeded {
                attempted: u64::MAX,
                max: self.config.limits.max_artifact_bytes,
            },
        )?;
        enforce_artifact_limit(artifact_bytes, self.config.limits.max_artifact_bytes)?;
        Ok(artifact_bytes)
    }

    /// Hash and stage one sequence file, in sequence order.
    fn publish_sequence_frame(
        &mut self,
        sequence: u64,
        encoded: &[u8],
    ) -> Result<(), SinkAdapterError> {
        let PngTarget::Sequence {
            directory,
            stem,
            digits,
        } = &self.config.target
        else {
            return Err(SinkAdapterError::InvalidConfig(
                "only a PNG sequence publishes frame files",
            ));
        };
        if self.directory_writer.is_none() {
            self.directory_writer = Some(
                Arc::clone(&self.fs)
                    .begin_atomic_directory(directory)
                    .map_err(|error| publish_error(directory, error))?,
            );
        }
        let encoded_bytes = byte_len(encoded)?;
        let leaf = png_leaf(stem, *digits, sequence);
        self.artifact_hasher
            .update(&u64::try_from(leaf.len()).unwrap_or(u64::MAX).to_le_bytes());
        self.artifact_hasher.update(leaf.as_bytes());
        self.artifact_hasher.update(&encoded_bytes.to_le_bytes());
        self.artifact_hasher.update(encoded);
        self.directory_writer
            .as_mut()
            .ok_or(SinkAdapterError::AlreadyFinalized)?
            .write_file(Path::new(&leaf), encoded)
            .map_err(|error| publish_error(directory, error))
    }

    /// Hand one accepted sequence frame to the encoder pool, waiting for a
    /// slot when every slot is busy, then commit whatever has completed in
    /// order.
    fn submit_sequence_frame(
        &mut self,
        sequence: u64,
        frame: &FrameBuffer,
        frame_bytes: u64,
    ) -> Result<(), SinkAdapterError> {
        let pool = self
            .encoders
            .as_mut()
            .ok_or(SinkAdapterError::AlreadyFinalized)?;
        let mut rgba = pool.spare.pop().unwrap_or_default();
        rgba.clear();
        append_tight(frame, &self.expected_layout, &mut rgba);
        while self
            .encoders
            .as_ref()
            .is_some_and(|pool| pool.in_flight >= pool.capacity)
        {
            self.receive_encoded(true)?;
            self.commit_ready()?;
        }
        let pool = self
            .encoders
            .as_mut()
            .ok_or(SinkAdapterError::AlreadyFinalized)?;
        pool.submit(sequence, rgba)?;
        self.state.commit_frame(frame_bytes);
        while self.receive_encoded(false)? {}
        self.commit_ready()
    }

    /// Move one finished frame from the pool into its ready set. Returns
    /// false when nothing was waiting and `block` is false.
    fn receive_encoded(&mut self, block: bool) -> Result<bool, SinkAdapterError> {
        let pool = self
            .encoders
            .as_mut()
            .ok_or(SinkAdapterError::AlreadyFinalized)?;
        pool.receive(block)
    }

    /// Commit every frame whose predecessors are committed.
    fn commit_ready(&mut self) -> Result<(), SinkAdapterError> {
        let frame_bytes = byte_len_from_usize(self.expected_layout.total_bytes())?;
        loop {
            let Some(pool) = self.encoders.as_mut() else {
                return Ok(());
            };
            let sequence = pool.next_commit;
            let Some(encoded) = pool.ready.remove(&sequence) else {
                return Ok(());
            };
            pool.next_commit += 1;
            pool.in_flight -= 1;
            let artifact_bytes = self.admit_encoded(frame_bytes, &encoded)?;
            self.publish_sequence_frame(sequence, &encoded)?;
            self.artifact_bytes = artifact_bytes;
        }
    }

    /// Drain the pool in order and stop its workers.
    fn drain_encoders(&mut self) -> Result<(), SinkAdapterError> {
        while self
            .encoders
            .as_ref()
            .is_some_and(|pool| pool.in_flight > 0)
        {
            self.receive_encoded(true)?;
            self.commit_ready()?;
        }
        self.encoders = None;
        Ok(())
    }

    fn prepare(&mut self) -> Result<(), SinkAdapterError> {
        self.drain_encoders()?;
        self.state.prepare("PNG")?;
        match &self.config.target {
            PngTarget::Single(path) => {
                let encoded = self
                    .single_encoded
                    .take()
                    .ok_or(SinkAdapterError::AlreadyFinalized)?;
                let mut writer = Arc::clone(&self.fs)
                    .begin_atomic_file(path)
                    .map_err(|error| publish_error(path, error))?;
                writer
                    .write(&encoded)
                    .map_err(|error| publish_error(path, error))?;
                self.prepared_file = Some(
                    writer
                        .prepare()
                        .map_err(|error| publish_error(path, error))?,
                );
            }
            PngTarget::Sequence { directory, .. } => {
                let writer = self
                    .directory_writer
                    .take()
                    .ok_or(SinkAdapterError::AlreadyFinalized)?;
                self.prepared_directory = Some(
                    writer
                        .prepare()
                        .map_err(|error| publish_error(directory, error))?,
                );
            }
        }
        Ok(())
    }

    fn commit(&mut self) -> Result<NativeArtifactReport, SinkAdapterError> {
        self.state.grant_commit()?;
        match &self.config.target {
            PngTarget::Single(path) => commit_file(
                self.prepared_file
                    .take()
                    .ok_or(SinkAdapterError::AlreadyFinalized)?,
                self.no_clobber,
            )
            .map_err(|error| publish_error(path, error))?,
            PngTarget::Sequence { directory, .. } => self
                .prepared_directory
                .take()
                .ok_or(SinkAdapterError::AlreadyFinalized)?
                .commit()
                .map_err(|error| publish_error(directory, error))?,
        }
        let (kind, path) = match &self.config.target {
            PngTarget::Single(path) => (NativeArtifactKind::Png, path.clone()),
            PngTarget::Sequence { directory, .. } => {
                (NativeArtifactKind::PngSequence, directory.clone())
            }
        };
        let mut artifact_hasher = self.artifact_hasher.clone();
        if matches!(&self.config.target, PngTarget::Sequence { .. }) {
            artifact_hasher.update(&self.state.frame_count.to_le_bytes());
        }
        Ok(NativeArtifactReport {
            kind,
            path,
            frame_count: self.state.frame_count,
            bytes: self.artifact_bytes,
            digest: artifact_hasher.finalize(),
        })
    }

    fn abort_inner(&mut self) {
        self.state.abort();
        self.encoders = None;
        self.single_encoded = None;
        self.directory_writer = None;
        self.prepared_file = None;
        self.prepared_directory = None;
    }
}

impl FrameSink for PngSink {
    fn write_frame(
        &mut self,
        sequence: u64,
        frame: &FrameBuffer,
    ) -> Result<SinkWrite, SinkFailure> {
        let result = self.write(sequence, frame);
        if result.is_err() {
            self.abort_inner();
        }
        sink_write(&self.receipt, result)
    }

    fn finish(&mut self) -> Result<(), SinkFailure> {
        self.prepare_finish()?;
        self.commit_finish()
    }

    fn prepare_finish(&mut self) -> Result<(), SinkFailure> {
        let result = self.prepare();
        if result.is_err() {
            self.abort_inner();
        }
        sink_prepare(&self.receipt, result)
    }

    fn commit_finish(&mut self) -> Result<(), SinkFailure> {
        let result = self.commit();
        if result.is_err() {
            self.abort_inner();
        }
        sink_finish(&self.receipt, result)
    }

    fn abort(&mut self) {
        self.abort_inner();
        self.receipt.abort();
    }
}

impl Drop for PngSink {
    fn drop(&mut self) {
        self.abort();
    }
}

/// How many sequence frames a PNG sink may encode at once (fm-hyqz): up to
/// `threads`, while every in-flight frame's raw and encoded bytes fit
/// `max_resident_bytes` together. One for a single PNG and on wasm32.
fn png_frame_concurrency(config: &PngSinkConfig, frame_bytes: u64) -> usize {
    if cfg!(target_arch = "wasm32") || !matches!(config.target, PngTarget::Sequence { .. }) {
        return 1;
    }
    let by_memory = config.limits.max_resident_bytes / frame_bytes.saturating_mul(2).max(1);
    config
        .threads
        .min(usize::try_from(by_memory).unwrap_or(usize::MAX))
        .max(1)
}

/// Frame-concurrent PNG-sequence encoding (fm-hyqz).
///
/// Each job is one whole frame encoded by the same canonical function as
/// the serial path, so the bytes cannot depend on the pool. Finished frames
/// are committed strictly in sequence order, which keeps the serial path's
/// artifact digest, file set and first-error order. Dropping the pool closes
/// its queue and joins the workers; a worker finishes at most its current
/// frame.
struct PngEncodePool {
    jobs: Option<SyncSender<PngEncodeJob>>,
    done: Receiver<PngEncoded>,
    workers: Vec<std::thread::JoinHandle<()>>,
    /// Most frames in flight at once.
    capacity: usize,
    /// Sequence of the oldest frame not yet committed.
    next_commit: u64,
    /// Frames submitted and not yet committed.
    in_flight: usize,
    /// Encoded frames waiting for their predecessors.
    ready: BTreeMap<u64, Vec<u8>>,
    /// Raw-frame buffers the workers handed back, for reuse.
    spare: Vec<Vec<u8>>,
}

struct PngEncodeJob {
    sequence: u64,
    rgba: Vec<u8>,
}

struct PngEncoded {
    sequence: u64,
    rgba: Vec<u8>,
    encoded: Result<Vec<u8>, &'static str>,
}

impl PngEncodePool {
    /// Start `capacity` workers, each fanning its frame over
    /// `segment_threads` fixed DEFLATE segments. `None` when a worker cannot
    /// start; the sink then keeps its serial path.
    fn start(
        config: &PngSinkConfig,
        capacity: usize,
        segment_threads: usize,
        first_sequence: u64,
    ) -> Option<Self> {
        let (jobs, job_rx) = mpsc::sync_channel::<PngEncodeJob>(capacity);
        let job_rx = Arc::new(Mutex::new(job_rx));
        let (done_tx, done) = mpsc::channel::<PngEncoded>();
        let mut pool = Self {
            jobs: Some(jobs),
            done,
            workers: Vec::with_capacity(capacity),
            capacity,
            next_commit: first_sequence,
            in_flight: 0,
            ready: BTreeMap::new(),
            spare: Vec::new(),
        };
        for worker in 0..capacity {
            let job_rx = Arc::clone(&job_rx);
            let done_tx = done_tx.clone();
            let (width, height, level) = (config.width, config.height, config.compression);
            let profile = config.profile.clone();
            let lane = u16::try_from(worker).unwrap_or(u16::MAX);
            let spawned = std::thread::Builder::new()
                .name(format!("fmn-png-encode-{worker}"))
                .spawn(move || {
                    loop {
                        let job = job_rx.lock().unwrap_or_else(PoisonError::into_inner).recv();
                        let Ok(PngEncodeJob { sequence, rgba }) = job else {
                            break;
                        };
                        let encoded = {
                            let _span = profile
                                .as_ref()
                                .and_then(|profile| profile.encode_span(sequence, lane));
                            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                encode_rgba8_segmented_parallel(
                                    width,
                                    height,
                                    &rgba,
                                    level,
                                    segment_threads,
                                )
                            }))
                            .map_err(|_| "a PNG encoder worker panicked")
                        };
                        let finished = PngEncoded {
                            sequence,
                            rgba,
                            encoded,
                        };
                        if done_tx.send(finished).is_err() {
                            break;
                        }
                    }
                });
            match spawned {
                Ok(handle) => pool.workers.push(handle),
                // Drop joins the workers already started.
                Err(_) => return None,
            }
        }
        Some(pool)
    }

    fn submit(&mut self, sequence: u64, rgba: Vec<u8>) -> Result<(), SinkAdapterError> {
        self.jobs
            .as_ref()
            .ok_or(SinkAdapterError::AlreadyFinalized)?
            .send(PngEncodeJob { sequence, rgba })
            .map_err(|_| SinkAdapterError::Codec {
                codec: "PNG",
                detail: "the encoder pool stopped".to_owned(),
            })?;
        self.in_flight += 1;
        Ok(())
    }

    fn receive(&mut self, block: bool) -> Result<bool, SinkAdapterError> {
        let finished = if block {
            match self.done.recv() {
                Ok(finished) => finished,
                Err(_) => {
                    return Err(SinkAdapterError::Codec {
                        codec: "PNG",
                        detail: "the encoder pool stopped".to_owned(),
                    });
                }
            }
        } else {
            match self.done.try_recv() {
                Ok(finished) => finished,
                Err(_) => return Ok(false),
            }
        };
        self.spare.push(finished.rgba);
        let encoded = finished.encoded.map_err(|detail| SinkAdapterError::Codec {
            codec: "PNG",
            detail: detail.to_owned(),
        })?;
        self.ready.insert(finished.sequence, encoded);
        Ok(true)
    }
}

impl Drop for PngEncodePool {
    fn drop(&mut self) {
        self.jobs = None;
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

/// Native GIF sink configuration.
#[derive(Debug, Clone)]
pub struct GifSinkConfig {
    /// Destination path.
    pub destination: PathBuf,
    /// Frame width.
    pub width: u32,
    /// Frame height.
    pub height: u32,
    /// Exact rational frame rate.
    pub fps: (u32, u32),
    /// Emit the NETSCAPE forever-loop extension.
    pub loop_forever: bool,
    /// First accepted sequence.
    pub first_sequence: u64,
    /// Explicit bounds.
    pub limits: SinkLimits,
    /// Optional output profiling.
    pub profile: Option<OutputProfile>,
}

/// Cancellation-safe native GIF adapter.
pub struct GifSink {
    fs: Arc<dyn FileSystem>,
    config: GifSinkConfig,
    expected_layout: FrameLayout,
    state: FrameState,
    encoder: GifStreamEncoder,
    scratch: Vec<u8>,
    writer: Option<Box<dyn AtomicFileWriter>>,
    prepared: Option<Box<dyn PreparedAtomicFile>>,
    artifact_bytes: u64,
    artifact_hasher: Sha256,
    receipt: SinkReceipt<NativeArtifactReport>,
    no_clobber: bool,
}

impl GifSink {
    /// Validate and construct a native GIF sink.
    pub fn new(fs: Arc<dyn FileSystem>, config: GifSinkConfig) -> Result<Self, SinkAdapterError> {
        validate_destination(&config.destination)?;
        validate_fps(config.fps)?;
        if config.width > u32::from(u16::MAX) || config.height > u32::from(u16::MAX) {
            return Err(SinkAdapterError::InvalidConfig(
                "GIF dimensions exceed the 16-bit format limit",
            ));
        }
        let expected_layout = tight_layout(PixelFormat::Rgba8, config.width, config.height)?;
        let state = FrameState::new(config.first_sequence, config.limits);
        state.check_resident(byte_len_from_usize(expected_layout.total_bytes())?)?;
        let scratch_bytes = expected_layout.total_bytes();
        let encoder =
            GifStreamEncoder::new(config.width, config.height, config.fps, config.loop_forever)
                .map_err(|error| SinkAdapterError::Codec {
                    codec: "GIF",
                    detail: error.to_string(),
                })?;
        Ok(Self {
            fs,
            config,
            expected_layout,
            state,
            encoder,
            scratch: Vec::with_capacity(scratch_bytes),
            writer: None,
            prepared: None,
            artifact_bytes: 0,
            artifact_hasher: Sha256::new(),
            receipt: SinkReceipt::pending(),
            no_clobber: false,
        })
    }

    /// Refuse an occupied destination atomically when the complete GIF commits.
    #[must_use]
    pub fn with_no_clobber(mut self) -> Self {
        self.no_clobber = true;
        self
    }

    /// Completion report handle.
    #[must_use]
    pub fn receipt(&self) -> SinkReceipt<NativeArtifactReport> {
        self.receipt.clone()
    }

    /// Bind this durable adapter reliably and retain its completion receipt.
    #[must_use]
    pub fn into_binding(
        self,
        name: impl Into<String>,
    ) -> (SinkBinding, SinkReceipt<NativeArtifactReport>) {
        let receipt = self.receipt();
        (SinkBinding::reliable(name, self), receipt)
    }

    fn write(&mut self, sequence: u64, frame: &FrameBuffer) -> Result<(), SinkAdapterError> {
        let frame_bytes = validate_frame(frame, &self.expected_layout)?;
        self.state.check(sequence, frame_bytes)?;
        let _span = self
            .config
            .profile
            .as_ref()
            .and_then(|profile| profile.span(Some(sequence), ProfilePhase::Emit));
        self.scratch.clear();
        append_tight(frame, &self.expected_layout, &mut self.scratch);
        if self.writer.is_none() {
            let header = self.encoder.header();
            let header_bytes = byte_len(&header)?;
            enforce_artifact_limit(header_bytes, self.config.limits.max_artifact_bytes)?;
            let mut writer = Arc::clone(&self.fs)
                .begin_atomic_file(&self.config.destination)
                .map_err(|error| publish_error(&self.config.destination, error))?;
            writer
                .write(&header)
                .map_err(|error| publish_error(&self.config.destination, error))?;
            self.artifact_hasher.update(&header);
            self.artifact_bytes = header_bytes;
            self.writer = Some(writer);
        }
        let encoded = {
            let _encode = self
                .config
                .profile
                .as_ref()
                .and_then(|profile| profile.span(Some(sequence), ProfilePhase::Encode));
            self.encoder
                .encode_frame(&self.scratch)
                .map_err(|error| SinkAdapterError::Codec {
                    codec: "GIF",
                    detail: error.to_string(),
                })?
        };
        let encoded_bytes = byte_len(&encoded)?;
        let resident = frame_bytes.checked_add(encoded_bytes).ok_or(
            SinkAdapterError::ResidentBytesExceeded {
                attempted: u64::MAX,
                max: self.config.limits.max_resident_bytes,
            },
        )?;
        self.state.check_resident(resident)?;
        let artifact_bytes = self.artifact_bytes.checked_add(encoded_bytes).ok_or(
            SinkAdapterError::ArtifactBytesExceeded {
                attempted: u64::MAX,
                max: self.config.limits.max_artifact_bytes,
            },
        )?;
        enforce_artifact_limit(artifact_bytes, self.config.limits.max_artifact_bytes)?;
        self.writer
            .as_mut()
            .ok_or(SinkAdapterError::AlreadyFinalized)?
            .write(&encoded)
            .map_err(|error| publish_error(&self.config.destination, error))?;
        self.artifact_hasher.update(&encoded);
        self.artifact_bytes = artifact_bytes;
        self.state.commit_frame(frame_bytes);
        Ok(())
    }

    fn prepare(&mut self) -> Result<(), SinkAdapterError> {
        self.state.prepare("GIF")?;
        let artifact_bytes =
            self.artifact_bytes
                .checked_add(1)
                .ok_or(SinkAdapterError::ArtifactBytesExceeded {
                    attempted: u64::MAX,
                    max: self.config.limits.max_artifact_bytes,
                })?;
        enforce_artifact_limit(artifact_bytes, self.config.limits.max_artifact_bytes)?;
        let mut writer = self
            .writer
            .take()
            .ok_or(SinkAdapterError::AlreadyFinalized)?;
        let trailer = GifStreamEncoder::trailer();
        writer
            .write(&trailer)
            .map_err(|error| publish_error(&self.config.destination, error))?;
        self.artifact_hasher.update(&trailer);
        self.prepared = Some(
            writer
                .prepare()
                .map_err(|error| publish_error(&self.config.destination, error))?,
        );
        self.artifact_bytes = artifact_bytes;
        Ok(())
    }

    fn commit(&mut self) -> Result<NativeArtifactReport, SinkAdapterError> {
        self.state.grant_commit()?;
        commit_file(
            self.prepared
                .take()
                .ok_or(SinkAdapterError::AlreadyFinalized)?,
            self.no_clobber,
        )
        .map_err(|error| publish_error(&self.config.destination, error))?;
        Ok(NativeArtifactReport {
            kind: NativeArtifactKind::Gif,
            path: self.config.destination.clone(),
            frame_count: self.state.frame_count,
            bytes: self.artifact_bytes,
            digest: self.artifact_hasher.clone().finalize(),
        })
    }

    fn abort_inner(&mut self) {
        self.state.abort();
        self.writer = None;
        self.prepared = None;
    }
}

impl FrameSink for GifSink {
    fn write_frame(
        &mut self,
        sequence: u64,
        frame: &FrameBuffer,
    ) -> Result<SinkWrite, SinkFailure> {
        let result = self.write(sequence, frame);
        if result.is_err() {
            self.abort_inner();
        }
        sink_write(&self.receipt, result)
    }

    fn finish(&mut self) -> Result<(), SinkFailure> {
        self.prepare_finish()?;
        self.commit_finish()
    }

    fn prepare_finish(&mut self) -> Result<(), SinkFailure> {
        let result = self.prepare();
        if result.is_err() {
            self.abort_inner();
        }
        sink_prepare(&self.receipt, result)
    }

    fn commit_finish(&mut self) -> Result<(), SinkFailure> {
        let result = self.commit();
        if result.is_err() {
            self.abort_inner();
        }
        sink_finish(&self.receipt, result)
    }

    fn abort(&mut self) {
        self.abort_inner();
        self.receipt.abort();
    }
}

impl Drop for GifSink {
    fn drop(&mut self) {
        self.abort();
    }
}

/// Native y4m sink configuration.
#[derive(Debug, Clone)]
pub struct Y4mSinkConfig {
    /// Destination path.
    pub destination: PathBuf,
    /// Frame width.
    pub width: u32,
    /// Frame height.
    pub height: u32,
    /// Exact rational frame rate.
    pub fps: (u32, u32),
    /// Chroma siting/container tag.
    pub colorspace: Y4mColorspace,
    /// First accepted sequence.
    pub first_sequence: u64,
    /// Explicit bounds.
    pub limits: SinkLimits,
    /// Optional output profiling.
    pub profile: Option<OutputProfile>,
}

/// Cancellation-safe native y4m adapter.
pub struct Y4mSink {
    fs: Arc<dyn FileSystem>,
    config: Y4mSinkConfig,
    expected_layout: FrameLayout,
    state: FrameState,
    scratch: Vec<u8>,
    header: Vec<u8>,
    writer: Option<Box<dyn AtomicFileWriter>>,
    prepared: Option<Box<dyn PreparedAtomicFile>>,
    artifact_bytes: u64,
    artifact_hasher: Sha256,
    receipt: SinkReceipt<NativeArtifactReport>,
    no_clobber: bool,
}

impl Y4mSink {
    /// Validate and construct a native y4m sink.
    pub fn new(fs: Arc<dyn FileSystem>, config: Y4mSinkConfig) -> Result<Self, SinkAdapterError> {
        validate_destination(&config.destination)?;
        validate_fps(config.fps)?;
        let expected_layout = tight_layout(PixelFormat::Nv12, config.width, config.height)?;
        let header = y4m_header(config.width, config.height, config.fps, config.colorspace);
        let header_bytes = byte_len(&header)?;
        enforce_artifact_limit(header_bytes, config.limits.max_artifact_bytes)?;
        let state = FrameState::new(config.first_sequence, config.limits);
        let record_capacity =
            expected_layout
                .total_bytes()
                .checked_add(6)
                .ok_or(SinkAdapterError::InvalidConfig(
                    "y4m frame record size overflowed",
                ))?;
        state.check_resident(byte_len_from_usize(record_capacity)?)?;
        Ok(Self {
            fs,
            config,
            expected_layout,
            state,
            scratch: Vec::with_capacity(record_capacity),
            header,
            writer: None,
            prepared: None,
            artifact_bytes: 0,
            artifact_hasher: Sha256::new(),
            receipt: SinkReceipt::pending(),
            no_clobber: false,
        })
    }

    /// Refuse an occupied destination atomically when the complete stream commits.
    #[must_use]
    pub fn with_no_clobber(mut self) -> Self {
        self.no_clobber = true;
        self
    }

    /// Completion report handle.
    #[must_use]
    pub fn receipt(&self) -> SinkReceipt<NativeArtifactReport> {
        self.receipt.clone()
    }

    /// Bind this durable adapter reliably and retain its completion receipt.
    #[must_use]
    pub fn into_binding(
        self,
        name: impl Into<String>,
    ) -> (SinkBinding, SinkReceipt<NativeArtifactReport>) {
        let receipt = self.receipt();
        (SinkBinding::reliable(name, self), receipt)
    }

    fn write(&mut self, sequence: u64, frame: &FrameBuffer) -> Result<(), SinkAdapterError> {
        let payload = validate_frame(frame, &self.expected_layout)?;
        self.state.check(sequence, payload)?;
        let _span = self
            .config
            .profile
            .as_ref()
            .and_then(|profile| profile.span(Some(sequence), ProfilePhase::Emit));
        if self.writer.is_none() {
            let mut writer = Arc::clone(&self.fs)
                .begin_atomic_file(&self.config.destination)
                .map_err(|error| publish_error(&self.config.destination, error))?;
            writer
                .write(&self.header)
                .map_err(|error| publish_error(&self.config.destination, error))?;
            self.artifact_hasher.update(&self.header);
            self.artifact_bytes = byte_len(&self.header)?;
            self.writer = Some(writer);
        }
        self.scratch.clear();
        {
            let _encode = self
                .config
                .profile
                .as_ref()
                .and_then(|profile| profile.span(Some(sequence), ProfilePhase::Encode));
            append_y4m_frame_nv12(
                &mut self.scratch,
                self.config.width,
                self.config.height,
                frame,
            )
            .map_err(|error| SinkAdapterError::Codec {
                codec: "y4m",
                detail: error.to_string(),
            })?;
        }
        let record_bytes = byte_len(&self.scratch)?;
        self.state.check_resident(record_bytes)?;
        let attempted_artifact = self.artifact_bytes.checked_add(record_bytes).ok_or(
            SinkAdapterError::ArtifactBytesExceeded {
                attempted: u64::MAX,
                max: self.config.limits.max_artifact_bytes,
            },
        )?;
        enforce_artifact_limit(attempted_artifact, self.config.limits.max_artifact_bytes)?;
        self.writer
            .as_mut()
            .ok_or(SinkAdapterError::AlreadyFinalized)?
            .write(&self.scratch)
            .map_err(|error| publish_error(&self.config.destination, error))?;
        self.artifact_hasher.update(&self.scratch);
        self.artifact_bytes = attempted_artifact;
        self.state.commit_frame(payload);
        Ok(())
    }

    fn prepare(&mut self) -> Result<(), SinkAdapterError> {
        self.state.prepare("y4m")?;
        let writer = self
            .writer
            .take()
            .ok_or(SinkAdapterError::AlreadyFinalized)?;
        self.prepared = Some(
            writer
                .prepare()
                .map_err(|error| publish_error(&self.config.destination, error))?,
        );
        Ok(())
    }

    fn commit(&mut self) -> Result<NativeArtifactReport, SinkAdapterError> {
        self.state.grant_commit()?;
        commit_file(
            self.prepared
                .take()
                .ok_or(SinkAdapterError::AlreadyFinalized)?,
            self.no_clobber,
        )
        .map_err(|error| publish_error(&self.config.destination, error))?;
        Ok(NativeArtifactReport {
            kind: NativeArtifactKind::Y4m,
            path: self.config.destination.clone(),
            frame_count: self.state.frame_count,
            bytes: self.artifact_bytes,
            digest: self.artifact_hasher.clone().finalize(),
        })
    }

    fn abort_inner(&mut self) {
        self.state.abort();
        self.writer = None;
        self.prepared = None;
    }
}

impl FrameSink for Y4mSink {
    fn write_frame(
        &mut self,
        sequence: u64,
        frame: &FrameBuffer,
    ) -> Result<SinkWrite, SinkFailure> {
        let result = self.write(sequence, frame);
        if result.is_err() {
            self.abort_inner();
        }
        sink_write(&self.receipt, result)
    }

    fn finish(&mut self) -> Result<(), SinkFailure> {
        self.prepare_finish()?;
        self.commit_finish()
    }

    fn prepare_finish(&mut self) -> Result<(), SinkFailure> {
        let result = self.prepare();
        if result.is_err() {
            self.abort_inner();
        }
        sink_prepare(&self.receipt, result)
    }

    fn commit_finish(&mut self) -> Result<(), SinkFailure> {
        let result = self.commit();
        if result.is_err() {
            self.abort_inner();
        }
        sink_finish(&self.receipt, result)
    }

    fn abort(&mut self) {
        self.abort_inner();
        self.receipt.abort();
    }
}

impl Drop for Y4mSink {
    fn drop(&mut self) {
        self.abort();
    }
}

/// Negotiated ffmpeg sink configuration.
pub struct FfmpegSinkConfig {
    /// Already-resolved/fingerprinted tool. Executable discovery is a separate
    /// platform capability; this adapter never performs ambient `PATH` lookup.
    pub tool: FfmpegTool,
    /// Probed encoder inventory.
    pub capabilities: EncoderCapabilities,
    /// Negotiated video job.
    pub job: VideoJob,
    /// Optional native WAV for the boundary's two-stage mux.
    pub audio: Option<PathBuf>,
    /// Published destination.
    pub destination: PathBuf,
    /// Canonical parent used when resolving `tool`; the boundary's owned
    /// private session and exclusive job directories live below it. A
    /// different canonical parent is refused.
    pub workdir_root: PathBuf,
    /// Timeout/log/workdir policy. Its artifact limit is tightened to the
    /// smaller of this value and [`SinkLimits::max_artifact_bytes`].
    pub job_limits: JobLimits,
    /// First accepted sequence.
    pub first_sequence: u64,
    /// Explicit payload/artifact bounds.
    pub limits: SinkLimits,
    /// Optional output profiling.
    pub profile: Option<OutputProfile>,
}

impl fmt::Debug for FfmpegSinkConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FfmpegSinkConfig")
            .field("tool", &self.tool)
            .field("capabilities", &self.capabilities)
            .field("job", &self.job)
            .field("audio", &self.audio)
            .field("destination", &self.destination)
            .field("workdir_root", &self.workdir_root)
            .field("job_limits", &self.job_limits)
            .field("first_sequence", &self.first_sequence)
            .field("limits", &self.limits)
            .field("profile", &self.profile)
            .finish()
    }
}

/// Cancellation-safe adapter to the negotiated ffmpeg boundary.
pub struct FfmpegSink {
    runner: Arc<dyn ProcessRunner>,
    config: FfmpegSinkConfig,
    expected_layout: FrameLayout,
    state: FrameState,
    cancellation: ProcessCancellation,
    stream: Option<StreamingEncode>,
    prepared: Option<PreparedFfmpegArtifact>,
    input_bytes: u64,
    receipt: SinkReceipt<FfmpegArtifactReport>,
    deferred_soundtrack: Option<Receiver<Option<MixReport>>>,
    no_clobber: bool,
}

/// One-use soundtrack completion for a live video sink. Scene construction
/// must explicitly supply its final mix (or no cues) before emitter finish.
/// Dropping this handle never authorizes silent publication.
pub struct FfmpegSoundtrack {
    sender: SyncSender<Option<MixReport>>,
}

impl FfmpegSoundtrack {
    /// Supply the completed scene soundtrack without waiting for video drain.
    ///
    /// # Errors
    /// Returns an error if the sink has already failed or been cancelled.
    pub fn finish(self, mix: Option<MixReport>) -> Result<(), SinkAdapterError> {
        self.sender
            .send(mix)
            .map_err(|_| SinkAdapterError::Cancelled)
    }
}

impl FfmpegSink {
    /// Validate and construct a negotiated ffmpeg sink.
    pub fn new(
        runner: Arc<dyn ProcessRunner>,
        mut config: FfmpegSinkConfig,
    ) -> Result<Self, SinkAdapterError> {
        validate_destination(&config.destination)?;
        validate_destination(&config.workdir_root)?;
        if config
            .audio
            .as_ref()
            .is_some_and(|path| !path.is_absolute())
        {
            return Err(SinkAdapterError::InvalidConfig(
                "ffmpeg audio path must be absolute",
            ));
        }
        let encoder = config
            .job
            .resolved_encoder()
            .map_err(|error| SinkAdapterError::Ffmpeg {
                detail: error.to_string(),
            })?;
        if let Some(encoder) = encoder
            && !config.capabilities.offers(&encoder)
        {
            return Err(SinkAdapterError::Ffmpeg {
                detail: format!("installed ffmpeg does not offer encoder {encoder:?}"),
            });
        }
        let expected_layout = tight_layout(
            config.job.wire.frame_format(),
            config.job.width,
            config.job.height,
        )?;
        config.job_limits.max_artifact_bytes = config
            .job_limits
            .max_artifact_bytes
            .min(config.limits.max_artifact_bytes);
        let state = FrameState::new(config.first_sequence, config.limits);
        state.check_resident(byte_len_from_usize(expected_layout.total_bytes())?)?;
        Ok(Self {
            runner,
            config,
            expected_layout,
            state,
            cancellation: ProcessCancellation::new(),
            stream: None,
            prepared: None,
            input_bytes: 0,
            receipt: SinkReceipt::pending(),
            deferred_soundtrack: None,
            no_clobber: false,
        })
    }

    /// Publish the verified video only if the destination remains absent.
    /// The final create-only operation, not an existence precheck, arbitrates
    /// concurrent renders after encoding and optional audio muxing finish.
    #[must_use]
    pub fn with_no_clobber(mut self) -> Self {
        self.no_clobber = true;
        self
    }

    /// Connect a soundtrack whose cues become known while frames are emitted.
    /// The completion handle is bounded to one message and must be resolved
    /// before [`OrderedEmitter::finish`](crate::OrderedEmitter::finish).
    ///
    /// # Errors
    /// Refuses a second audio source or a sink that has already accepted frames.
    pub fn with_deferred_soundtrack(
        mut self,
    ) -> Result<(Self, FfmpegSoundtrack), SinkAdapterError> {
        if self.config.audio.is_some()
            || self.deferred_soundtrack.is_some()
            || self.stream.is_some()
            || self.state.lifecycle != AdapterLifecycle::Accepting
        {
            return Err(SinkAdapterError::InvalidConfig(
                "deferred soundtrack requires a fresh sink without an audio source",
            ));
        }
        let (sender, receiver) = mpsc::sync_channel(1);
        self.deferred_soundtrack = Some(receiver);
        Ok((self, FfmpegSoundtrack { sender }))
    }

    /// Completion report handle carrying every boundary invocation.
    ///
    /// Each invocation identifies the canonical configured tool and the
    /// rehashed private executable copy that the process mechanism ran.
    #[must_use]
    pub fn receipt(&self) -> SinkReceipt<FfmpegArtifactReport> {
        self.receipt.clone()
    }

    /// Bind this durable adapter reliably and retain its provenance receipt.
    #[must_use]
    pub fn into_binding(
        self,
        name: impl Into<String>,
    ) -> (SinkBinding, SinkReceipt<FfmpegArtifactReport>) {
        let receipt = self.receipt();
        (SinkBinding::reliable(name, self), receipt)
    }

    fn write(&mut self, sequence: u64, frame: &FrameBuffer) -> Result<(), SinkAdapterError> {
        let frame_bytes = validate_frame(frame, &self.expected_layout)?;
        self.state.check(sequence, frame_bytes)?;
        let _span = self
            .config
            .profile
            .as_ref()
            .and_then(|profile| profile.span(Some(sequence), ProfilePhase::Emit));
        if self.stream.is_none() {
            let boundary = Boundary::new(
                self.config.tool.clone(),
                Arc::clone(&self.runner),
                self.config.job_limits.clone(),
                self.config.workdir_root.clone(),
            )
            .map_err(boundary_error)?;
            self.stream = Some(
                boundary
                    .start_encode(
                        &self.config.job,
                        &self.config.capabilities,
                        &self.config.destination,
                        self.config.audio.as_deref(),
                        self.cancellation.clone(),
                        ProcessStdinLimits::new(
                            self.config.limits.max_resident_bytes,
                            self.config.limits.max_stream_bytes,
                        ),
                    )
                    .map_err(boundary_error)?,
            );
        }
        let _feed = self
            .config
            .profile
            .as_ref()
            .and_then(|profile| profile.span(Some(sequence), ProfilePhase::FfmpegFeed));
        let feed = feed_tight(
            self.stream
                .as_mut()
                .ok_or(SinkAdapterError::AlreadyFinalized)?,
            frame,
            &self.expected_layout,
        );
        if let Err(error) = feed {
            if self.cancellation.is_cancelled() {
                return Err(SinkAdapterError::Cancelled);
            }
            return Err(error);
        }
        self.input_bytes = self.input_bytes.checked_add(frame_bytes).ok_or(
            SinkAdapterError::StreamBytesExceeded {
                attempted: u64::MAX,
                max: self.config.limits.max_stream_bytes,
            },
        )?;
        self.state.commit_frame(frame_bytes);
        Ok(())
    }

    fn prepare(&mut self) -> Result<(), SinkAdapterError> {
        self.state.prepare("ffmpeg video")?;
        let _span = self
            .config
            .profile
            .as_ref()
            .and_then(|profile| profile.span(None, ProfilePhase::Encode));
        let stream = self
            .stream
            .take()
            .ok_or(SinkAdapterError::AlreadyFinalized)?;
        let mix = self
            .deferred_soundtrack
            .take()
            .map(|receiver| {
                receiver.try_recv().map_err(|_| {
                    SinkAdapterError::InvalidConfig(
                        "scene soundtrack was not finalized before video publication",
                    )
                })
            })
            .transpose()?
            .flatten();
        self.prepared = Some(if let Some(mix) = mix {
            preflight_wav_artifact(
                SampleFormat::S16,
                DitherPolicy::None,
                mix.audio.samples.len(),
                self.config.job_limits.max_artifact_bytes,
            )?;
            let wav = mix
                .wav_bytes(SampleFormat::S16, DitherPolicy::None)
                .map_err(|error| SinkAdapterError::Codec {
                    codec: "WAV",
                    detail: error.to_string(),
                })?;
            stream.prepare_with_wav(&wav).map_err(boundary_error)?
        } else {
            stream.prepare().map_err(boundary_error)?
        });
        Ok(())
    }

    fn commit(&mut self) -> Result<FfmpegArtifactReport, SinkAdapterError> {
        self.state.grant_commit()?;
        let prepared = self
            .prepared
            .take()
            .ok_or(SinkAdapterError::AlreadyFinalized)?;
        let report = if self.no_clobber {
            prepared.commit_new()
        } else {
            prepared.commit()
        }
        .map_err(boundary_error)?;
        Ok(FfmpegArtifactReport {
            boundary: report,
            frame_count: self.state.frame_count,
            input_bytes: self.input_bytes,
        })
    }

    fn abort_inner(&mut self) {
        self.state.abort();
        self.stream = None;
        self.prepared = None;
        self.deferred_soundtrack = None;
    }
}

impl FrameSink for FfmpegSink {
    fn set_cancellation(&mut self, cancellation: ProcessCancellation) {
        if self.stream.is_none() && self.prepared.is_none() {
            self.cancellation = cancellation;
        }
    }

    fn write_frame(
        &mut self,
        sequence: u64,
        frame: &FrameBuffer,
    ) -> Result<SinkWrite, SinkFailure> {
        let result = self.write(sequence, frame);
        if result.is_err() {
            self.abort_inner();
        }
        sink_write(&self.receipt, result)
    }

    fn finish(&mut self) -> Result<(), SinkFailure> {
        self.prepare_finish()?;
        self.commit_finish()
    }

    fn prepare_finish(&mut self) -> Result<(), SinkFailure> {
        let result = self.prepare();
        if result.is_err() {
            self.abort_inner();
        }
        sink_prepare(&self.receipt, result)
    }

    fn commit_finish(&mut self) -> Result<(), SinkFailure> {
        let result = self.commit();
        if result.is_err() {
            self.abort_inner();
        }
        sink_finish(&self.receipt, result)
    }

    fn abort(&mut self) {
        self.abort_inner();
        self.receipt.abort();
    }
}

impl Drop for FfmpegSink {
    fn drop(&mut self) {
        self.abort();
    }
}

/// Native WAV publication configuration.
#[derive(Debug, Clone)]
pub struct WavPublicationConfig {
    /// Destination path.
    pub destination: PathBuf,
    /// Certified `S16` or lossless-intermediate `F32`.
    pub format: SampleFormat,
    /// Defined reduction policy.
    pub dither: DitherPolicy,
    /// Complete artifact bound.
    pub max_artifact_bytes: u64,
    /// Optional output profiling.
    pub profile: Option<OutputProfile>,
}

/// Encode and atomically publish a completed native mix as WAV.
///
/// WAV is deliberately not a [`FrameSink`]: audio is finalized from
/// [`MixReport`] on its own sample timeline, then optionally handed to the
/// ffmpeg sink for the boundary's two-stage mux.
pub fn publish_wav(
    fs: &dyn FileSystem,
    config: &WavPublicationConfig,
    mix: &MixReport,
) -> Result<WavPublicationReport, SinkAdapterError> {
    publish_wav_with_policy(fs, config, mix, false)
}

/// Publish a complete native WAV without replacing an existing destination.
/// Uses the same codec, budgets and receipt as [`publish_wav`], with the
/// filesystem's atomic create-new operation as its publication point.
///
/// # Errors
/// Invalid configuration, codec/budget errors, or an occupied destination.
pub fn publish_wav_new(
    fs: &dyn FileSystem,
    config: &WavPublicationConfig,
    mix: &MixReport,
) -> Result<WavPublicationReport, SinkAdapterError> {
    publish_wav_with_policy(fs, config, mix, true)
}

fn publish_wav_with_policy(
    fs: &dyn FileSystem,
    config: &WavPublicationConfig,
    mix: &MixReport,
    no_clobber: bool,
) -> Result<WavPublicationReport, SinkAdapterError> {
    validate_destination(&config.destination)?;
    if config.max_artifact_bytes == 0 {
        return Err(SinkAdapterError::InvalidConfig(
            "WAV max_artifact_bytes must be nonzero",
        ));
    }
    preflight_wav_artifact(
        config.format,
        config.dither,
        mix.audio.samples.len(),
        config.max_artifact_bytes,
    )?;
    let _span = config
        .profile
        .as_ref()
        .and_then(|profile| profile.span(None, ProfilePhase::Encode));
    let encoded = mix
        .wav_bytes(config.format, config.dither)
        .map_err(|error| SinkAdapterError::Codec {
            codec: "WAV",
            detail: error.to_string(),
        })?;
    let bytes = byte_len(&encoded)?;
    enforce_artifact_limit(bytes, config.max_artifact_bytes)?;
    if no_clobber {
        if !fs
            .create_new(&config.destination, &encoded)
            .map_err(|error| publish_error(&config.destination, error))?
        {
            return Err(publish_error(
                &config.destination,
                "WAV destination already exists",
            ));
        }
    } else {
        publish_atomic(fs, &config.destination, &encoded)?;
    }
    let channels = u64::from(mix.audio.channels);
    let sample_count = u64::try_from(mix.audio.samples.len())
        .map_err(|_| SinkAdapterError::InvalidConfig("WAV sample count is not representable"))?;
    Ok(WavPublicationReport {
        path: config.destination.clone(),
        bytes,
        digest: sha256(&encoded),
        sample_frames: sample_count / channels,
        clipped_samples: mix.clipped_samples,
        cues_mixed: mix.cues_mixed,
    })
}
/// Native SVG publication configuration.
#[derive(Debug, Clone)]
pub struct SvgPublicationConfig {
    /// Destination path.
    pub destination: PathBuf,
    /// Complete artifact bound.
    pub max_artifact_bytes: u64,
    /// Optional output profiling.
    pub profile: Option<OutputProfile>,
}

/// Successful native SVG publication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SvgPublicationReport {
    /// Published destination.
    pub path: PathBuf,
    /// Encoded bytes.
    pub bytes: u64,
    /// SHA-256 of the published bytes.
    pub digest: Digest,
}

/// Encode-validate and atomically publish one complete native SVG document.
///
/// SVG, like WAV, is deliberately not a [`FrameSink`]: a still is a single
/// finished document, not an ordered frame stream. The sink receives the
/// complete emitted bytes from the composition root, so the Reel boundary
/// stays bytes-only; geometry ownership remains in the geometry crates. The
/// publication is bounded by `max_artifact_bytes` and is one atomic rename,
/// so cancellation between staging and publication never exposes a torn
/// artifact.
///
/// # Errors
/// An invalid destination, a zero budget, or document bytes beyond the
/// declared artifact budget ([`SinkAdapterError::ArtifactBytesExceeded`]).
pub fn publish_svg(
    fs: &dyn FileSystem,
    config: &SvgPublicationConfig,
    document: &[u8],
) -> Result<SvgPublicationReport, SinkAdapterError> {
    validate_destination(&config.destination)?;
    if config.max_artifact_bytes == 0 {
        return Err(SinkAdapterError::InvalidConfig(
            "SVG max_artifact_bytes must be nonzero",
        ));
    }
    let _span = config
        .profile
        .as_ref()
        .and_then(|profile| profile.span(None, ProfilePhase::Encode));
    let bytes = byte_len(document)?;
    enforce_artifact_limit(bytes, config.max_artifact_bytes)?;
    publish_atomic(fs, &config.destination, document)?;
    Ok(SvgPublicationReport {
        path: config.destination.clone(),
        bytes,
        digest: sha256(document),
    })
}

/// Publish a complete SVG only if no destination exists, including a symlink.
///
/// Like [`publish_svg`], this is bounded and atomic, but the filesystem's
/// create-new capability arbitrates competing publishers at the final commit.
/// An existence precheck alone cannot establish this no-clobber contract.
///
/// # Errors
/// An existing destination, invalid budget, oversized document or filesystem
/// publication error. Existing bytes remain untouched on every failure.
pub fn publish_svg_new(
    fs: &dyn FileSystem,
    config: &SvgPublicationConfig,
    document: &[u8],
) -> Result<SvgPublicationReport, SinkAdapterError> {
    validate_destination(&config.destination)?;
    if config.max_artifact_bytes == 0 {
        return Err(SinkAdapterError::InvalidConfig(
            "SVG max_artifact_bytes must be nonzero",
        ));
    }
    let bytes = byte_len(document)?;
    enforce_artifact_limit(bytes, config.max_artifact_bytes)?;
    let _span = config
        .profile
        .as_ref()
        .and_then(|profile| profile.span(None, ProfilePhase::Encode));
    if !fs
        .create_new(&config.destination, document)
        .map_err(|error| publish_error(&config.destination, error))?
    {
        return Err(publish_error(
            &config.destination,
            "SVG destination already exists",
        ));
    }
    Ok(SvgPublicationReport {
        path: config.destination.clone(),
        bytes,
        digest: sha256(document),
    })
}

fn commit_file(
    prepared: Box<dyn PreparedAtomicFile>,
    no_clobber: bool,
) -> Result<(), fmn_platform::fs::FsError> {
    if no_clobber {
        prepared.commit_new()
    } else {
        prepared.commit()
    }
}

struct FrameState {
    expected_sequence: u64,
    frame_count: u64,
    stream_bytes: u64,
    limits: SinkLimits,
    lifecycle: AdapterLifecycle,
}

impl FrameState {
    fn new(first_sequence: u64, limits: SinkLimits) -> Self {
        Self {
            expected_sequence: first_sequence,
            frame_count: 0,
            stream_bytes: 0,
            limits,
            lifecycle: AdapterLifecycle::Accepting,
        }
    }

    fn check(&self, sequence: u64, frame_bytes: u64) -> Result<(), SinkAdapterError> {
        if self.lifecycle != AdapterLifecycle::Accepting {
            return Err(SinkAdapterError::AlreadyFinalized);
        }
        if sequence != self.expected_sequence {
            return Err(SinkAdapterError::UnexpectedSequence {
                expected: self.expected_sequence,
                actual: sequence,
            });
        }
        if sequence == u64::MAX {
            return Err(SinkAdapterError::SequenceExhausted { sequence });
        }
        let attempted_frames =
            self.frame_count
                .checked_add(1)
                .ok_or(SinkAdapterError::FrameLimitExceeded {
                    attempted: u64::MAX,
                    max: self.limits.max_frames,
                })?;
        let accepted_frames = self.limits.exact_frames.unwrap_or(self.limits.max_frames);
        if attempted_frames > accepted_frames {
            return Err(SinkAdapterError::FrameLimitExceeded {
                attempted: attempted_frames,
                max: accepted_frames,
            });
        }
        self.check_resident(frame_bytes)?;
        let attempted_bytes = self.stream_bytes.checked_add(frame_bytes).ok_or(
            SinkAdapterError::StreamBytesExceeded {
                attempted: u64::MAX,
                max: self.limits.max_stream_bytes,
            },
        )?;
        if attempted_bytes > self.limits.max_stream_bytes {
            return Err(SinkAdapterError::StreamBytesExceeded {
                attempted: attempted_bytes,
                max: self.limits.max_stream_bytes,
            });
        }
        Ok(())
    }

    fn check_resident(&self, bytes: u64) -> Result<(), SinkAdapterError> {
        if bytes > self.limits.max_resident_bytes {
            return Err(SinkAdapterError::ResidentBytesExceeded {
                attempted: bytes,
                max: self.limits.max_resident_bytes,
            });
        }
        Ok(())
    }

    fn commit_frame(&mut self, frame_bytes: u64) {
        self.expected_sequence += 1;
        self.frame_count += 1;
        self.stream_bytes += frame_bytes;
    }

    fn prepare(&mut self, kind: &'static str) -> Result<(), SinkAdapterError> {
        if self.lifecycle != AdapterLifecycle::Accepting {
            return Err(SinkAdapterError::AlreadyFinalized);
        }
        if self.frame_count == 0 {
            return Err(SinkAdapterError::EmptyStream { kind });
        }
        if let Some(expected) = self.limits.exact_frames
            && self.frame_count != expected
        {
            return Err(SinkAdapterError::FrameCountMismatch {
                expected,
                actual: self.frame_count,
            });
        }
        self.lifecycle = AdapterLifecycle::Prepared;
        Ok(())
    }

    fn grant_commit(&mut self) -> Result<(), SinkAdapterError> {
        if self.lifecycle != AdapterLifecycle::Prepared {
            return Err(SinkAdapterError::AlreadyFinalized);
        }
        self.lifecycle = AdapterLifecycle::Finalized;
        Ok(())
    }

    fn abort(&mut self) {
        self.lifecycle = AdapterLifecycle::Finalized;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AdapterLifecycle {
    Accepting,
    Prepared,
    Finalized,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn tight_layout(
    format: PixelFormat,
    width: u32,
    height: u32,
) -> Result<FrameLayout, SinkAdapterError> {
    FrameLayout::tight(format, width, height).map_err(|error| SinkAdapterError::InvalidGeometry {
        detail: error.to_string(),
    })
}

fn validate_fps(fps: (u32, u32)) -> Result<(), SinkAdapterError> {
    if fps.0 == 0 || fps.1 == 0 {
        return Err(SinkAdapterError::InvalidConfig(
            "output frame rate must be nonzero",
        ));
    }
    Ok(())
}

fn validate_destination(path: &Path) -> Result<(), SinkAdapterError> {
    if path.as_os_str().is_empty() {
        return Err(SinkAdapterError::InvalidConfig(
            "output destination must not be empty",
        ));
    }
    Ok(())
}

fn validate_png_target(target: &PngTarget) -> Result<(), SinkAdapterError> {
    match target {
        PngTarget::Single(path) => validate_destination(path),
        PngTarget::Sequence {
            directory,
            stem,
            digits,
        } => {
            validate_destination(directory)?;
            let mut bytes = stem.bytes();
            let first_is_safe = bytes
                .next()
                .is_some_and(|byte| byte.is_ascii_alphanumeric());
            let rest_is_safe =
                bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'));
            let upper = stem.to_ascii_uppercase();
            let windows_reserved = matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                || upper.strip_prefix("COM").is_some_and(|number| {
                    matches!(number, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
                })
                || upper.strip_prefix("LPT").is_some_and(|number| {
                    matches!(number, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
                });
            if !first_is_safe || !rest_is_safe || stem.len() > 128 || windows_reserved {
                return Err(SinkAdapterError::InvalidConfig(
                    "PNG sequence stem must be 1..=128 ASCII alphanumeric/_/- bytes, start alphanumeric, and not be a Windows device name",
                ));
            }
            if !(1..=20).contains(digits) {
                return Err(SinkAdapterError::InvalidConfig(
                    "PNG sequence digits must be in 1..=20",
                ));
            }
            Ok(())
        }
    }
}

fn validate_frame(frame: &FrameBuffer, expected: &FrameLayout) -> Result<u64, SinkAdapterError> {
    let got = frame.layout();
    if got.format() != expected.format()
        || got.width() != expected.width()
        || got.height() != expected.height()
    {
        return Err(SinkAdapterError::FrameMismatch {
            expected_format: expected.format(),
            expected_width: expected.width(),
            expected_height: expected.height(),
            got_format: got.format(),
            got_width: got.width(),
            got_height: got.height(),
        });
    }
    byte_len_from_usize(expected.total_bytes())
}

fn append_tight(frame: &FrameBuffer, expected: &FrameLayout, out: &mut Vec<u8>) {
    let layout = frame.layout();
    for plane in 0..expected.format().plane_count() {
        let rows = expected.format().plane_rows(expected.height(), plane) as usize;
        let row_bytes = expected.stride(plane);
        let stride = layout.stride(plane);
        let source = frame.plane(plane);
        for row in 0..rows {
            let start = row * stride;
            out.extend_from_slice(&source[start..start + row_bytes]);
        }
    }
}

fn feed_tight(
    stream: &mut StreamingEncode,
    frame: &FrameBuffer,
    expected: &FrameLayout,
) -> Result<(), SinkAdapterError> {
    let layout = frame.layout();
    for plane in 0..expected.format().plane_count() {
        let rows = expected.format().plane_rows(expected.height(), plane) as usize;
        let row_bytes = expected.stride(plane);
        let stride = layout.stride(plane);
        let source = frame.plane(plane);
        for row in 0..rows {
            let start = row * stride;
            stream
                .write_stdin(&source[start..start + row_bytes])
                .map_err(boundary_error)?;
        }
    }
    Ok(())
}

fn png_leaf(stem: &str, digits: usize, sequence: u64) -> String {
    format!("{stem}_{sequence:0width$}.png", width = digits)
}

fn preflight_wav_artifact(
    format: SampleFormat,
    dither: DitherPolicy,
    sample_count: usize,
    max_artifact_bytes: u64,
) -> Result<(), SinkAdapterError> {
    let bytes_per_sample = match (format, dither) {
        (SampleFormat::S16, _) => 2u64,
        (SampleFormat::F32, DitherPolicy::None) => 4u64,
        // Let `MixReport::wav_bytes` return its precise semantic refusal for
        // unsupported formats or dither on floating-point output.
        _ => return Ok(()),
    };
    let samples = u64::try_from(sample_count)
        .map_err(|_| SinkAdapterError::InvalidConfig("WAV sample count is not representable"))?;
    let data_bytes = samples
        .checked_mul(bytes_per_sample)
        .ok_or(SinkAdapterError::Codec {
            codec: "WAV",
            detail: "encoded sample data exceeds the RIFF size domain".to_string(),
        })?;
    if data_bytes > u64::from(u32::MAX) - 36 {
        return Err(SinkAdapterError::Codec {
            codec: "WAV",
            detail: "encoded sample data exceeds the RIFF size domain".to_string(),
        });
    }
    let artifact_bytes = data_bytes.checked_add(44).ok_or(SinkAdapterError::Codec {
        codec: "WAV",
        detail: "encoded artifact size overflowed".to_string(),
    })?;
    enforce_artifact_limit(artifact_bytes, max_artifact_bytes)
}

fn enforce_artifact_limit(attempted: u64, max: u64) -> Result<(), SinkAdapterError> {
    if attempted > max {
        return Err(SinkAdapterError::ArtifactBytesExceeded { attempted, max });
    }
    Ok(())
}

fn publish_atomic(fs: &dyn FileSystem, path: &Path, bytes: &[u8]) -> Result<(), SinkAdapterError> {
    fs.write_atomic(path, bytes)
        .map_err(|error| SinkAdapterError::Publish {
            path: path.to_path_buf(),
            detail: error.to_string(),
        })
}

fn publish_error(path: &Path, error: impl fmt::Display) -> SinkAdapterError {
    SinkAdapterError::Publish {
        path: path.to_path_buf(),
        detail: error.to_string(),
    }
}

fn boundary_error(error: BoundaryError) -> SinkAdapterError {
    if matches!(&error, BoundaryError::Cancelled) {
        SinkAdapterError::Cancelled
    } else {
        SinkAdapterError::Ffmpeg {
            detail: error.to_string(),
        }
    }
}

fn byte_len(bytes: &[u8]) -> Result<u64, SinkAdapterError> {
    byte_len_from_usize(bytes.len())
}

fn byte_len_from_usize(bytes: usize) -> Result<u64, SinkAdapterError> {
    u64::try_from(bytes)
        .map_err(|_| SinkAdapterError::InvalidConfig("byte count is not representable as u64"))
}

fn sink_write<T>(
    receipt: &SinkReceipt<T>,
    result: Result<(), SinkAdapterError>,
) -> Result<SinkWrite, SinkFailure> {
    match result {
        Ok(()) => Ok(SinkWrite::Consumed),
        Err(error) => {
            let failure = adapter_failure(&error);
            if matches!(error, SinkAdapterError::Cancelled) {
                receipt.abort();
            } else {
                receipt.fail(failure.clone());
            }
            Err(failure)
        }
    }
}

fn sink_prepare<T>(
    receipt: &SinkReceipt<T>,
    result: Result<(), SinkAdapterError>,
) -> Result<(), SinkFailure> {
    match result {
        Ok(()) => Ok(()),
        Err(error) => {
            let failure = adapter_failure(&error);
            if matches!(error, SinkAdapterError::Cancelled) {
                receipt.abort();
            } else {
                receipt.fail(failure.clone());
            }
            Err(failure)
        }
    }
}

fn sink_finish<T>(
    receipt: &SinkReceipt<T>,
    result: Result<T, SinkAdapterError>,
) -> Result<(), SinkFailure> {
    match result {
        Ok(report) => {
            receipt.publish(report);
            Ok(())
        }
        Err(error) => {
            let failure = adapter_failure(&error);
            if matches!(error, SinkAdapterError::Cancelled) {
                receipt.abort();
            } else {
                receipt.fail(failure.clone());
            }
            Err(failure)
        }
    }
}

fn adapter_failure(error: &SinkAdapterError) -> SinkFailure {
    SinkFailure::with_code(error.code(), error.to_string())
}
