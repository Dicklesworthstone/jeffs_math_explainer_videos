//! Sink orchestration, the negotiated ffmpeg boundary, and sound (§14).
//!
//! Landed so far (fm-wj3): the **negotiated ffmpeg boundary v2** —
//! FFMPEG_PROTOCOL.md's implementation. [`negotiate`] is the pure
//! negotiation model and deterministic argv construction (no `vflip`,
//! no `eq`, structurally); [`ffmpeg`] is the sandboxed execution layer
//! over fmn-platform's process capability: opaque fingerprinted tool
//! resolution proven through a private executable copy, an identity-checked
//! private session with exclusive job directories, environment allowlisting,
//! artifact verification, and atomic publication. ffmpeg is optional — its
//! absence (or inability to run as a relocation-safe private copy) is a
//! capability error naming the native alternative.
//!
//! [`emitter`] is the bounded, preallocated, frame-index-ordered handoff
//! between Lumen/runtime workers and every sink. [`sound`] is the native
//! WAV/PCM path: sample-exact placement, a named deterministic resampler,
//! ordered gain/duck mixing, explicit clipping evidence, and certified WAV
//! bytes. [`sinks`] supplies the production, bounded adapters that compose
//! those pieces into cancellation-safe native and negotiated artifacts.
#![cfg_attr(
    any(
        test,
        all(
            target_arch = "x86_64",
            target_feature = "avx2",
            target_feature = "bmi2",
            target_feature = "fma"
        ),
        all(
            target_arch = "x86_64",
            target_feature = "avx512f",
            target_feature = "avx512bw",
            target_feature = "avx512dq",
            target_feature = "avx512vl"
        ),
        all(target_arch = "aarch64", target_feature = "neon")
    ),
    feature(portable_simd)
)]
#![cfg_attr(test, feature(test))]
#![cfg_attr(windows, feature(windows_by_handle))]
#![forbid(unsafe_code)]

pub mod audio;
pub mod emitter;
pub mod ffmpeg;
pub mod negotiate;
pub mod provenance;
pub mod sinks;
pub mod sound;
mod sound_edit;

pub use fmn_hash::Digest as ArtifactDigest;

pub use audio::{
    AudioDecodeError, AudioDecodeLimits, AudioDecodeReport, AudioDecoder, AudioInputFormat,
    DecodedAudio,
};

pub use emitter::{
    CancelOutcome, EmitterConfig, EmitterError, EmitterFailure, EmitterHandle, EmitterReport,
    EmitterStats, FrameReservation, FrameSink, OrderedEmitter, SinkBinding, SinkFailure, SinkMode,
    SinkReport, SinkWrite,
};
pub use ffmpeg::{
    Boundary, BoundaryError, BoundaryReport, EncoderCapabilities, FfmpegTool, HARDWARE_ENCODERS,
    InvocationReport, JobLimits, NATIVE_ALTERNATIVE, PreparedFfmpegArtifact, Provenance,
    StreamingEncode,
};
pub use negotiate::{
    ColorDescription, Container, EncoderChoice, NegotiationError, Primaries, Transfer, VideoJob,
    WireFormat,
};
pub use provenance::{
    CLOSURE_SCHEMA, ClosureItem, ManifestComparison, ManifestError, ManifestIdentity, ManifestMode,
    ManifestOutput, PLATFORM_ITEM_PREFIX, PROVENANCE_SCHEMA, ProvenanceManifest, SEMANTIC_SCHEMA,
    StructuralField, certified_build_refusal,
};
pub use sinks::{
    FfmpegArtifactReport, FfmpegSink, FfmpegSinkConfig, FfmpegSoundtrack, GifSink, GifSinkConfig,
    NativeArtifactKind, NativeArtifactReport, OutputProfile, PngSink, PngSinkConfig, PngTarget,
    ReceiptError, SinkAdapterError, SinkLimits, SinkReceipt, SvgPublicationConfig,
    SvgPublicationReport, WavPublicationConfig, WavPublicationReport, Y4mSink, Y4mSinkConfig,
    publish_svg, publish_svg_new, publish_wav, publish_wav_new,
};
pub use sound::{
    COMPILED_MIX_LANES, COMPILED_MIX_TIER, DitherPolicy, MixKernel, MixReport, MixerConfig,
    RESAMPLER_NAME, SoundCue, SoundError, SoundMixer, frames_to_samples,
};
