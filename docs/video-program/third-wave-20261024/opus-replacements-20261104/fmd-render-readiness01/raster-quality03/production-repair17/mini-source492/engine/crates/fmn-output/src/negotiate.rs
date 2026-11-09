//! The negotiation model of the ffmpeg boundary (§14.3, D-23) — pure
//! data and deterministic argv construction, no processes anywhere.
//!
//! Negotiation replaces the Reference's fixed `rawvideo/rgba → vflip →
//! eq` pipe. fmn-frame renders in output orientation and applies the
//! transfer natively, so **no argv builder in this module can emit
//! `vflip` or an `eq` filter** — there is no code path for either, and
//! the contract suite asserts their absence over every builder.
//!
//! The negotiated dimensions: wire pixel format (RGBA8/BGRA8 for alpha
//! and compatibility, NV12 for ordinary 8-bit video, P010 for 10-bit),
//! color description (primaries, transfer, range), frame rate as an
//! exact rational, container, and encoder. The arithmetic behind NV12:
//! a 3840×2160 RGBA8 frame is 33,177,600 bytes against NV12's
//! 12,441,600 — 2.67× less pipe payload before any copies.

use std::path::Path;

use fmn_frame::{ColorRange, PixelFormat};

mod quality;
pub use quality::{EncoderPreset, EncoderTune, VideoQuality};

/// The pixel formats that travel down the pipe to ffmpeg.
///
/// This is deliberately narrower than [`PixelFormat`]: `Rgba16F` is a
/// renderer intermediate, never a wire format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WireFormat {
    /// 8-bit RGBA — alpha and compatibility sinks.
    Rgba8,
    /// 8-bit BGRA — compatibility sinks that want it.
    Bgra8,
    /// 4:2:0 8-bit — ordinary video.
    Nv12,
    /// 4:2:0 10-bit — 10-bit/HDR-capable output.
    P010,
}

impl WireFormat {
    /// The ffmpeg `-pix_fmt` name.
    #[must_use]
    pub const fn ffmpeg_pix_fmt(self) -> &'static str {
        match self {
            Self::Rgba8 => "rgba",
            Self::Bgra8 => "bgra",
            Self::Nv12 => "nv12",
            Self::P010 => "p010le",
        }
    }

    /// The fmn-frame format this wire format carries.
    #[must_use]
    pub const fn frame_format(self) -> PixelFormat {
        match self {
            Self::Rgba8 => PixelFormat::Rgba8,
            Self::Bgra8 => PixelFormat::Bgra8,
            Self::Nv12 => PixelFormat::Nv12,
            Self::P010 => PixelFormat::P010,
        }
    }

    /// Bytes of one tightly-packed frame on the wire.
    #[must_use]
    pub const fn frame_bytes(self, width: u32, height: u32) -> usize {
        let px = width as usize * height as usize;
        match self {
            Self::Rgba8 | Self::Bgra8 => px * 4,
            // Luma + interleaved half-res chroma.
            Self::Nv12 => px + px / 2,
            Self::P010 => (px + px / 2) * 2,
        }
    }

    /// Whether the wire carries an alpha channel.
    #[must_use]
    pub const fn has_alpha(self) -> bool {
        matches!(self, Self::Rgba8 | Self::Bgra8)
    }
}

/// Color primaries. BT.709 is the only negotiated space today; the enum
/// exists so a future BT.2020 lane is a variant, not a rewrite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Primaries {
    /// ITU-R BT.709 primaries.
    Bt709,
}

/// Transfer characteristic declared to the sink. fmn-frame applied the
/// transfer once, natively; this flag only *describes* the bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Transfer {
    /// IEC 61966-2-1 (sRGB) — the canonical RGBA path.
    Srgb,
    /// ITU-R BT.709 — the conventional video declaration.
    Bt709,
}

/// The complete color description of the wire bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorDescription {
    /// Color primaries.
    pub primaries: Primaries,
    /// Transfer characteristic.
    pub transfer: Transfer,
    /// Quantization range.
    pub range: ColorRange,
}

impl ColorDescription {
    /// The ordinary 8-bit video description: BT.709, limited range.
    #[must_use]
    pub const fn video_bt709() -> Self {
        Self {
            primaries: Primaries::Bt709,
            transfer: Transfer::Bt709,
            range: ColorRange::Limited,
        }
    }

    /// The RGBA compatibility description: sRGB transfer, full range.
    #[must_use]
    pub const fn srgb_full() -> Self {
        Self {
            primaries: Primaries::Bt709,
            transfer: Transfer::Srgb,
            range: ColorRange::Full,
        }
    }
}

/// Encoder selection. `Auto` resolves to the documented software
/// default for the container; hardware encoders are always an explicit,
/// named request (FFMPEG_PROTOCOL.md §5) so acceleration can never be a
/// silent substitution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncoderChoice {
    /// The container's software default (`libx264`, or `qtrle` for
    /// transparent MOV).
    Auto,
    /// A named encoder (software or hardware), validated against the
    /// installed ffmpeg's capabilities before any spawn.
    Named(String),
    /// An explicit encoder and its typed quality policy. Construct through
    /// [`VideoJob::with_quality`] to resolve `Auto` before binding the policy.
    /// Every boundary use revalidates it; direct enum construction cannot
    /// bypass encoder-specific refusals. This carries settings without adding
    /// required fields to existing `VideoJob` struct literals.
    Configured {
        /// The actual ffmpeg encoder name, not an argv fragment.
        name: String,
        /// Opt-in compressed-video controls.
        quality: VideoQuality,
    },
}

/// The negotiated container/mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Container {
    /// MP4 (`+faststart`), yuv output.
    Mp4,
    /// Opaque QuickTime MOV.
    Mov,
    /// Transparent MOV (`qtrle`, argb) — requires an alpha wire format.
    MovTransparent,
    /// The retained ffmpeg GIF mode (the native GIF codec needs no
    /// ffmpeg at all; this mode exists for parity).
    Gif,
}

impl Container {
    /// File extension for intermediate artifacts.
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mov | Self::MovTransparent => "mov",
            Self::Gif => "gif",
        }
    }

    const fn default_encoder(self) -> Option<&'static str> {
        match self {
            Self::Mp4 | Self::Mov => Some("libx264"),
            Self::MovTransparent => Some("qtrle"),
            // GIF is a muxer-level mode; no `-c:v` is emitted.
            Self::Gif => None,
        }
    }
}

/// One negotiated video-encode job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoJob {
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
    /// Exact rational frame rate (numerator, denominator).
    pub fps: (u32, u32),
    /// The wire pixel format.
    pub wire: WireFormat,
    /// The color description of the wire bytes.
    pub color: ColorDescription,
    /// The container/mode.
    pub container: Container,
    /// Encoder selection.
    pub encoder: EncoderChoice,
    /// Constant-rate-factor quality, only meaningful for the software
    /// x264/x265 encoders. [`Self::with_quality`] also accepts it; conflicting
    /// legacy/configured values are refused instead of silently overwritten.
    pub crf: Option<u8>,
}

/// A refused negotiation, named.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegotiationError(pub &'static str);

impl std::fmt::Display for NegotiationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "refused negotiation: {}", self.0)
    }
}

impl std::error::Error for NegotiationError {}

impl VideoJob {
    /// Resolve the encoder name this job will use (`Auto` → the
    /// container's software default).
    ///
    /// # Errors
    /// [`NegotiationError`] when the job's dimensions contradict each
    /// other (alpha container on an opaque wire, zero dimensions or frame
    /// rate) or quality controls contradict the encoder/container/rate mode.
    pub fn resolved_encoder(&self) -> Result<Option<String>, NegotiationError> {
        if self.width == 0 || self.height == 0 {
            return Err(NegotiationError("zero frame dimensions"));
        }
        if self.fps.0 == 0 || self.fps.1 == 0 {
            return Err(NegotiationError("zero frame rate"));
        }
        if self.container == Container::MovTransparent && !self.wire.has_alpha() {
            return Err(NegotiationError(
                "transparent MOV requires an alpha wire format (RGBA8/BGRA8)",
            ));
        }
        let encoder = match &self.encoder {
            EncoderChoice::Auto => self.container.default_encoder().map(str::to_owned),
            EncoderChoice::Named(name) | EncoderChoice::Configured { name, .. } => {
                if self.container == Container::Gif {
                    return Err(NegotiationError(
                        "GIF mode is muxer-level; it takes no encoder",
                    ));
                }
                Some(name.clone())
            }
        };
        self.merged_quality()?
            .validate(encoder.as_deref(), self.container)?;
        Ok(encoder)
    }
}

fn push(argv: &mut Vec<String>, items: &[&str]) {
    argv.extend(items.iter().map(|s| (*s).to_string()));
}

const fn range_name(range: ColorRange) -> &'static str {
    match range {
        ColorRange::Limited => "tv",
        ColorRange::Full => "pc",
    }
}

const fn transfer_name(transfer: Transfer) -> &'static str {
    match transfer {
        Transfer::Srgb => "iec61966-2-1",
        Transfer::Bt709 => "bt709",
    }
}

/// The common prefix of every boundary invocation.
fn base_argv() -> Vec<String> {
    vec!["-hide_banner".into(), "-loglevel".into(), "error".into()]
}

/// Build the encode argv: rawvideo frames on stdin → `out`.
///
/// # Errors
/// [`NegotiationError`] per [`VideoJob::resolved_encoder`].
pub fn encode_argv(job: &VideoJob, out: &Path) -> Result<Vec<String>, NegotiationError> {
    let encoder = job.resolved_encoder()?;
    let quality = job.merged_quality()?;
    let mut argv = base_argv();
    // Input: tightly-packed frames, output orientation, on stdin.
    push(&mut argv, &["-f", "rawvideo"]);
    push(&mut argv, &["-pix_fmt", job.wire.ffmpeg_pix_fmt()]);
    argv.push("-video_size".into());
    argv.push(format!("{}x{}", job.width, job.height));
    argv.push("-framerate".into());
    argv.push(format!("{}/{}", job.fps.0, job.fps.1));
    push(&mut argv, &["-i", "-"]);

    // Output. Only this pure negotiation layer emits encoder controls.
    if let Some(encoder) = &encoder {
        push(&mut argv, &["-c:v", encoder]);
    }
    if let Some(crf) = quality.crf {
        argv.push("-crf".into());
        argv.push(crf.to_string());
    }
    if let Some(preset) = quality.preset {
        push(&mut argv, &["-preset", preset.as_str()]);
    }
    if let Some(tune) = quality.tune {
        push(&mut argv, &["-tune", tune.as_str()]);
    }
    if let Some(bitrate) = quality.bitrate {
        argv.push("-b:v".into());
        argv.push(bitrate.to_string());
    }
    match job.container {
        Container::Mp4 | Container::Mov => {
            // yuv output planes; 10-bit in stays 10-bit out.
            let out_fmt = if job.wire == WireFormat::P010 {
                "yuv420p10le"
            } else {
                "yuv420p"
            };
            push(&mut argv, &["-pix_fmt", out_fmt]);
        }
        Container::MovTransparent => push(&mut argv, &["-pix_fmt", "argb"]),
        Container::Gif => push(&mut argv, &["-f", "gif"]),
    }
    if job.container == Container::Mp4 {
        push(&mut argv, &["-movflags", "+faststart"]);
    }
    push(&mut argv, &["-color_primaries", "bt709"]);
    push(
        &mut argv,
        &["-color_trc", transfer_name(job.color.transfer)],
    );
    push(&mut argv, &["-colorspace", "bt709"]);
    push(&mut argv, &["-color_range", range_name(job.color.range)]);
    argv.push("-y".into());
    argv.push(out.display().to_string());
    Ok(argv)
}

/// Stage 2 of the two-stage audio mux: copy the already-encoded video
/// stream (never re-encode — `-c:v copy` is the contract) and encode
/// the audio to AAC.
#[must_use]
pub fn mux_argv(video: &Path, audio: &Path, out: &Path) -> Vec<String> {
    let mut argv = base_argv();
    push(&mut argv, &["-i"]);
    argv.push(video.display().to_string());
    push(&mut argv, &["-i"]);
    argv.push(audio.display().to_string());
    push(&mut argv, &["-c:v", "copy", "-c:a", "aac"]);
    push(&mut argv, &["-map", "0:v:0", "-map", "1:a:0"]);
    argv.push("-y".into());
    argv.push(out.display().to_string());
    argv
}

/// Concatenate partial movie files (the Reference's insert-file /
/// partial-movie mechanism) with stream copy.
#[must_use]
pub fn concat_argv(list_file: &Path, out: &Path) -> Vec<String> {
    let mut argv = base_argv();
    push(&mut argv, &["-f", "concat", "-safe", "0", "-i"]);
    argv.push(list_file.display().to_string());
    push(&mut argv, &["-c", "copy"]);
    argv.push("-y".into());
    argv.push(out.display().to_string());
    argv
}

/// Media-transcode capability: decode any audio ffmpeg reads into the
/// engine's native PCM WAV.
#[must_use]
pub fn transcode_audio_argv(input: &Path, out_wav: &Path) -> Vec<String> {
    let mut argv = base_argv();
    push(&mut argv, &["-i"]);
    argv.push(input.display().to_string());
    push(&mut argv, &["-vn", "-acodec", "pcm_s16le", "-f", "wav"]);
    argv.push("-y".into());
    argv.push(out_wav.display().to_string());
    argv
}

/// Media-transcode capability: decode an exotic image into PNG for the
/// native decoder.
#[must_use]
pub fn transcode_image_argv(input: &Path, out_png: &Path) -> Vec<String> {
    let mut argv = base_argv();
    push(&mut argv, &["-i"]);
    argv.push(input.display().to_string());
    push(
        &mut argv,
        &["-frames:v", "1", "-c:v", "png", "-f", "image2"],
    );
    argv.push("-y".into());
    argv.push(out_png.display().to_string());
    argv
}
