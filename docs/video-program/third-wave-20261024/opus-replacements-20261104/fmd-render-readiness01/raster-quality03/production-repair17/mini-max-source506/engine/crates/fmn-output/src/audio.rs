//! Native-first sound-asset decoding over the one governed media boundary.
//!
//! PCM WAV never locates or starts ffmpeg. Other explicitly recognized audio
//! containers require an opt-in decoder capability. The external decoder
//! preserves the source sample rate and channel count; Reel's native mixer
//! still owns resampling, placement, gain, ducking, and final quantization.
//! Decoded PCM has an independent content identity and carries the actual
//! decoder invocation. This is not a certification of compressed-file decode.

use std::path::PathBuf;
use std::sync::Arc;

use fmn_codec::{WavAudio, WavError, WavLimits, decode_wav};
use fmn_hash::{Digest, Sha256, sha256};
#[cfg(not(all(not(target_arch = "wasm32"), feature = "exact-process")))]
use fmn_platform::process::NoProcessRunner as HostProcessRunner;
#[cfg(all(not(target_arch = "wasm32"), feature = "exact-process"))]
use fmn_platform::process::StdProcessRunner as HostProcessRunner;
use fmn_platform::process::{FfmpegLocator, StdFfmpegLocator};

use crate::{Boundary, BoundaryError, FfmpegTool, InvocationReport, JobLimits};

/// The allowlisted, byte-recognized containers; never a caller-supplied format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioInputFormat {
    Wav,
    Flac,
    MpegAudio,
    AacAdts,
    Ogg,
    IsoMedia,
    Aiff,
}

impl AudioInputFormat {
    /// The exact ffmpeg demuxer, also used in decode provenance.
    #[must_use]
    pub const fn demuxer(self) -> &'static str {
        match self {
            Self::Wav => "wav",
            Self::Flac => "flac",
            Self::MpegAudio => "mp3",
            Self::AacAdts => "aac",
            Self::Ogg => "ogg",
            Self::IsoMedia => "mov",
            Self::Aiff => "aiff",
        }
    }

    fn recognize(bytes: &[u8]) -> Option<Self> {
        if bytes.len() >= 12 {
            if matches!(&bytes[..4], b"RIFF" | b"RF64") && &bytes[8..12] == b"WAVE" {
                return Some(Self::Wav);
            }
            if &bytes[..4] == b"FORM" && matches!(&bytes[8..12], b"AIFF" | b"AIFC") {
                return Some(Self::Aiff);
            }
            if matches!(&bytes[4..8], b"ftyp" | b"moov" | b"mdat" | b"wide") {
                let size = u32::from_be_bytes(bytes[..4].try_into().ok()?);
                if size == 1 || (size >= 8 && u64::from(size) <= bytes.len() as u64) {
                    return Some(Self::IsoMedia);
                }
            }
        }
        if bytes.starts_with(b"fLaC") {
            return Some(Self::Flac);
        }
        if bytes.starts_with(b"OggS") {
            return Some(Self::Ogg);
        }
        if bytes.len() >= 10
            && bytes.starts_with(b"ID3")
            && (2..=4).contains(&bytes[3])
            && bytes[6..10].iter().all(|b| b & 0x80 == 0)
        {
            return Some(Self::MpegAudio);
        }
        if bytes.len() >= 4 && bytes[0] == 0xff {
            if bytes[1] & 0xf6 == 0xf0 {
                return Some(Self::AacAdts);
            }
            // MPEG audio: sync, non-reserved version/layer, bitrate and rate.
            if bytes[1] & 0xe0 == 0xe0
                && bytes[1] & 0x18 != 0x08
                && bytes[1] & 0x06 != 0
                && bytes[2] & 0xf0 != 0xf0
                && bytes[2] & 0x0c != 0x0c
            {
                return Some(Self::MpegAudio);
            }
        }
        None
    }
}

/// Bounds apply to both native decode and compressed expansion, before allocation.
#[derive(Clone, Debug)]
pub struct AudioDecodeLimits {
    pub max_input_bytes: u64,
    /// Interleaved samples (frames times channels), not frames.
    pub max_samples: u64,
}

impl Default for AudioDecodeLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 64 * 1024 * 1024,
            max_samples: WavLimits::default().max_samples,
        }
    }
}

impl AudioDecodeLimits {
    pub(crate) fn decoded_byte_limit(&self) -> Result<u64, AudioDecodeError> {
        if self.max_input_bytes == 0 || self.max_samples == 0 {
            return Err(AudioDecodeError::InvalidLimits);
        }
        // Float32 PCM plus bounded RIFF framing. Refuse a cap that cannot be
        // represented by the owned RIFF decoder or by this host's address space.
        self.max_samples
            .checked_mul(4)
            .and_then(|n| n.checked_add(65_536))
            .filter(|n| *n < u64::from(u32::MAX) && usize::try_from(*n).is_ok())
            .ok_or(AudioDecodeError::InvalidLimits)
    }
}

#[derive(Debug)]
pub enum AudioDecodeError {
    InvalidLimits,
    InputOversized { bytes: u64, max: u64 },
    UnsupportedInput,
    TranscoderRequired { format: AudioInputFormat },
    Capability(String),
    Native(WavError),
    InvalidPcm(&'static str),
    Boundary(BoundaryError),
}

impl std::fmt::Display for AudioDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLimits => f.write_str("audio decode limits must be nonzero and fit native RIFF/address-space bounds"),
            Self::InputOversized { bytes, max } => write!(f, "audio input of {bytes} bytes exceeds the {max}-byte budget"),
            Self::UnsupportedInput => f.write_str("unrecognized audio container; use PCM WAV, FLAC, MP3, ADTS AAC, Ogg, M4A/MP4, or AIFF; playlists and remote references are not audio assets"),
            Self::TranscoderRequired { format } => write!(f, "{} audio requires the optional governed ffmpeg decoder; supply PCM WAV for native-only or certified rendering", format.demuxer()),
            Self::Capability(detail) => write!(f, "audio decode capability unavailable: {detail}; supply PCM WAV to avoid ffmpeg"),
            Self::Native(error) => write!(f, "native audio decode: {error}"),
            Self::InvalidPcm(detail) => write!(f, "decoded audio: {detail}"),
            Self::Boundary(error) => write!(f, "audio media boundary: {error}"),
        }
    }
}

impl std::error::Error for AudioDecodeError {}
impl From<BoundaryError> for AudioDecodeError {
    fn from(error: BoundaryError) -> Self {
        Self::Boundary(error)
    }
}

/// Exact input identity, normalized PCM identity, and optional external work.
#[derive(Clone, Debug)]
pub struct AudioDecodeReport {
    pub format: AudioInputFormat,
    pub source_digest: Digest,
    /// SHA-256 of the versioned, little-endian normalized PCM stream; excludes
    /// container metadata and original sample packing. Negative zero is canonical.
    pub pcm_digest: Digest,
    pub invocation: Option<InvocationReport>,
}

#[derive(Clone, Debug)]
pub struct DecodedAudio {
    pub audio: WavAudio,
    pub report: AudioDecodeReport,
}

pub(crate) fn decoded_audio(
    source: &[u8],
    audio: WavAudio,
    format: AudioInputFormat,
    invocation: Option<InvocationReport>,
) -> Result<DecodedAudio, AudioDecodeError> {
    if !matches!(audio.channels, 1 | 2) {
        return Err(AudioDecodeError::InvalidPcm(
            "only mono and stereo layouts are supported; implicit downmix is forbidden",
        ));
    }
    if audio.sample_rate == 0
        || !audio
            .samples
            .len()
            .is_multiple_of(usize::from(audio.channels))
    {
        return Err(AudioDecodeError::InvalidPcm(
            "sample rate must be nonzero and samples must contain complete channel frames",
        ));
    }
    let mut hash = Sha256::new();
    hash.update(b"fmn-normalized-pcm-v1\0");
    hash.update(&audio.channels.to_le_bytes());
    hash.update(&audio.sample_rate.to_le_bytes());
    hash.update(&(audio.samples.len() as u64).to_le_bytes());
    for &sample in &audio.samples {
        if !sample.is_finite() {
            return Err(AudioDecodeError::InvalidPcm("non-finite PCM sample"));
        }
        hash.update(&(if sample == 0.0 { 0.0_f32 } else { sample }).to_le_bytes());
    }
    Ok(DecodedAudio {
        report: AudioDecodeReport {
            format,
            source_digest: sha256(source),
            pcm_digest: hash.finalize(),
            invocation,
        },
        audio,
    })
}

/// Reusable, lazy host decoder. `None` is an explicit native-only policy.
/// Construction and PCM WAV decoding never discover, probe, or start ffmpeg.
/// A resolved decoder is retained for the generation, so later cues cannot
/// silently select a different executable after a PATH or source change.
pub struct AudioDecoder {
    limits: AudioDecodeLimits,
    ffmpeg_bin: Option<PathBuf>,
    boundary: Option<Boundary>,
    locator: StdFfmpegLocator,
    workdir_root: Option<PathBuf>,
}

impl AudioDecoder {
    pub fn new(
        limits: AudioDecodeLimits,
        ffmpeg_bin: Option<PathBuf>,
    ) -> Result<Self, AudioDecodeError> {
        limits.decoded_byte_limit()?;
        if ffmpeg_bin
            .as_ref()
            .is_some_and(|p| p.as_os_str().is_empty())
        {
            return Err(AudioDecodeError::Capability("empty ffmpeg path".into()));
        }
        if cfg!(target_arch = "wasm32") && ffmpeg_bin.is_some() {
            return Err(AudioDecodeError::Capability(
                "host audio transcoding is unavailable on wasm32".into(),
            ));
        }
        let ffmpeg_bin = ffmpeg_bin
            .map(|path| {
                if path.is_relative() && path.components().count() > 1 {
                    std::env::current_dir()
                        .map(|cwd| cwd.join(path))
                        .map_err(|e| AudioDecodeError::Capability(e.to_string()))
                } else {
                    Ok(path)
                }
            })
            .transpose()?;
        let (locator, workdir_root) = if ffmpeg_bin.is_some() {
            (
                StdFfmpegLocator::from_host_path(),
                Some(std::env::temp_dir()),
            )
        } else {
            (StdFfmpegLocator::default(), None)
        };
        Ok(Self {
            limits,
            ffmpeg_bin,
            boundary: None,
            locator,
            workdir_root,
        })
    }

    /// Inject an already-governed boundary (e.g. a host's capability runner).
    pub fn with_boundary(
        limits: AudioDecodeLimits,
        boundary: Boundary,
    ) -> Result<Self, AudioDecodeError> {
        limits.decoded_byte_limit()?;
        Ok(Self {
            limits,
            ffmpeg_bin: None,
            boundary: Some(boundary),
            locator: StdFfmpegLocator::default(),
            workdir_root: None,
        })
    }

    pub fn decode(&mut self, bytes: &[u8]) -> Result<DecodedAudio, AudioDecodeError> {
        if bytes.len() as u64 > self.limits.max_input_bytes {
            return Err(AudioDecodeError::InputOversized {
                bytes: bytes.len() as u64,
                max: self.limits.max_input_bytes,
            });
        }
        match decode_wav(
            bytes,
            &WavLimits {
                max_samples: self.limits.max_samples,
            },
        ) {
            Ok(audio) => return decoded_audio(bytes, audio, AudioInputFormat::Wav, None),
            Err(
                WavError::NotWav
                | WavError::UnsupportedFormat { .. }
                | WavError::UnsupportedDepth { .. },
            ) => {}
            // A damaged PCM file or exceeded native budget is never "repaired"
            // by a more permissive external parser.
            Err(error) => return Err(AudioDecodeError::Native(error)),
        }
        let format =
            AudioInputFormat::recognize(bytes).ok_or(AudioDecodeError::UnsupportedInput)?;
        if self.boundary.is_none() {
            let path = self
                .ffmpeg_bin
                .as_ref()
                .ok_or(AudioDecodeError::TranscoderRequired { format })?;
            // The native-codec facade deliberately disables exact-process.
            // Do not enable a process substrate just to decode WAV, or inspect
            // ambient PATH when this build cannot honor a transcode request.
            // An explicitly injected boundary remains usable without the host
            // runner, and native WAV returned above never reaches this check.
            if !cfg!(all(not(target_arch = "wasm32"), feature = "exact-process")) {
                return Err(AudioDecodeError::Capability(
                    "host transcoding requires a native exact-process build or an injected boundary"
                        .into(),
                ));
            }
            let runner = Arc::new(HostProcessRunner);
            let executable = self
                .locator
                .locate_ffmpeg(path)
                .map_err(|e| AudioDecodeError::Capability(e.to_string()))?;
            let root = self
                .workdir_root
                .clone()
                .ok_or(AudioDecodeError::TranscoderRequired { format })?;
            let tool = FfmpegTool::resolve(executable, runner.as_ref(), &root)?;
            let limits = JobLimits {
                max_artifact_bytes: self.limits.decoded_byte_limit()?,
                ..JobLimits::default()
            };
            self.boundary = Some(Boundary::new(tool, runner, limits, root)?);
        }
        self.boundary
            .as_ref()
            .ok_or(AudioDecodeError::TranscoderRequired { format })?
            .decode_audio_bytes(bytes, format, &self.limits)
    }
}
