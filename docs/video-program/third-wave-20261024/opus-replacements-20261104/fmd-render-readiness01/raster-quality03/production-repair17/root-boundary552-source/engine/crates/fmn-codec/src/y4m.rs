//! Owned YUV4MPEG2 (y4m) container (§14.2) — the native, no-ffmpeg
//! uncompressed video output and the certified raw-frame stream's
//! container option.
//!
//! y4m is a header line plus `FRAME` records of planar YUV data. The
//! writer emits planar I420 (from fmn-frame NV12 buffers, chroma
//! deinterleaved) in the two 4:2:0 chroma sitings `Y4mColorspace`
//! defines; full-resolution C444 output is future work. The exact
//! rational frame rate travels verbatim in the `F` parameter — no
//! float ever touches the timing. A reader lives here too, primarily
//! as the conformance oracle for the writer.

use fmn_frame::{FrameBuffer, PixelFormat};

/// Typed refusals of the y4m codec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Y4mError {
    /// Not a YUV4MPEG2 stream.
    NotY4m,
    /// The stream ended mid-structure.
    Truncated,
    /// A malformed header or frame parameter.
    BadHeader(&'static str),
    /// The writer requires an NV12 source buffer for C420 output.
    WrongSource {
        /// The format that was supplied.
        got: PixelFormat,
    },
    /// Frame geometry does not match the stream header.
    GeometryMismatch,
}

impl std::fmt::Display for Y4mError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotY4m => write!(f, "not a YUV4MPEG2 stream"),
            Self::Truncated => write!(f, "y4m stream truncated"),
            Self::BadHeader(what) => write!(f, "malformed y4m header: {what}"),
            Self::WrongSource { got } => {
                write!(f, "y4m C420 writer needs an Nv12 source, got {got:?}")
            }
            Self::GeometryMismatch => write!(f, "frame geometry differs from stream header"),
        }
    }
}

impl std::error::Error for Y4mError {}

/// The colorspaces the writer emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Y4mColorspace {
    /// 4:2:0 with MPEG-2 (left-sited) chroma — the ordinary video form.
    C420Mpeg2,
    /// 4:2:0 with JPEG (center-sited) chroma.
    C420Jpeg,
}

impl Y4mColorspace {
    const fn tag(self) -> &'static str {
        match self {
            Self::C420Mpeg2 => "C420mpeg2",
            Self::C420Jpeg => "C420jpeg",
        }
    }
}

/// Encode the complete y4m stream header.
#[must_use]
pub fn y4m_header(width: u32, height: u32, fps: (u32, u32), colorspace: Y4mColorspace) -> Vec<u8> {
    format!(
        "YUV4MPEG2 W{width} H{height} F{}:{} Ip A1:1 {}\n",
        fps.0,
        fps.1,
        colorspace.tag()
    )
    .into_bytes()
}

/// Append one NV12 frame record to a reusable buffer.
///
/// The caller may clear and reuse `out` for each frame, keeping memory
/// duration-independent. Stride padding is dropped and NV12 chroma is
/// deinterleaved into y4m's planar I420 order.
///
/// # Errors
/// [`Y4mError::WrongSource`] / [`Y4mError::GeometryMismatch`].
pub fn append_y4m_frame_nv12(
    out: &mut Vec<u8>,
    width: u32,
    height: u32,
    frame: &FrameBuffer,
) -> Result<(), Y4mError> {
    let layout = frame.layout();
    if layout.format() != PixelFormat::Nv12 {
        return Err(Y4mError::WrongSource {
            got: layout.format(),
        });
    }
    if layout.width() != width || layout.height() != height {
        return Err(Y4mError::GeometryMismatch);
    }
    let width = width as usize;
    let height = height as usize;
    out.extend_from_slice(b"FRAME\n");
    let luma = frame.plane(0);
    let luma_stride = layout.stride(0);
    for row in 0..height {
        out.extend_from_slice(&luma[row * luma_stride..row * luma_stride + width]);
    }
    let chroma = frame.plane(1);
    let chroma_stride = layout.stride(1);
    for row in 0..height / 2 {
        let row = &chroma[row * chroma_stride..row * chroma_stride + width];
        for pair in row.as_chunks::<2>().0 {
            out.push(pair[0]);
        }
    }
    for row in 0..height / 2 {
        let row = &chroma[row * chroma_stride..row * chroma_stride + width];
        for pair in row.as_chunks::<2>().0 {
            out.push(pair[1]);
        }
    }
    Ok(())
}

/// A y4m stream writer accumulating into bytes.
#[derive(Debug)]
pub struct Y4mWriter {
    width: u32,
    height: u32,
    out: Vec<u8>,
}

impl Y4mWriter {
    /// Begin a stream: the header carries the exact rational rate.
    #[must_use]
    pub fn new(width: u32, height: u32, fps: (u32, u32), colorspace: Y4mColorspace) -> Self {
        let out = y4m_header(width, height, fps, colorspace);
        Self { width, height, out }
    }

    /// Append one frame from an NV12 buffer (chroma deinterleaved to
    /// the planar I420 layout y4m requires). Stride padding is dropped;
    /// only payload bytes reach the stream.
    ///
    /// # Errors
    /// [`Y4mError::WrongSource`] / [`Y4mError::GeometryMismatch`].
    pub fn write_frame_nv12(&mut self, frame: &FrameBuffer) -> Result<(), Y4mError> {
        append_y4m_frame_nv12(&mut self.out, self.width, self.height, frame)
    }

    /// Finish the stream.
    #[must_use]
    pub fn finish(self) -> Vec<u8> {
        self.out
    }
}

/// A parsed y4m stream (the writer's conformance oracle).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedY4m {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// The exact rational rate from the `F` parameter.
    pub fps: (u32, u32),
    /// The colorspace tag, verbatim.
    pub colorspace: String,
    /// Each frame's planar payload (Y, then Cb, then Cr).
    pub frames: Vec<Vec<u8>>,
}

/// Parse a y4m stream.
///
/// # Errors
/// Every refusal in [`Y4mError`].
pub fn decode_y4m(data: &[u8]) -> Result<DecodedY4m, Y4mError> {
    let header_end = data
        .iter()
        .position(|&b| b == b'\n')
        .ok_or(Y4mError::Truncated)?;
    let header = std::str::from_utf8(&data[..header_end]).map_err(|_| Y4mError::NotY4m)?;
    let mut parts = header.split_ascii_whitespace();
    if parts.next() != Some("YUV4MPEG2") {
        return Err(Y4mError::NotY4m);
    }
    let mut width = 0u32;
    let mut height = 0u32;
    let mut fps = (0u32, 0u32);
    let mut colorspace = "C420".to_string();
    for param in parts {
        let (key, value) = param.split_at(1);
        match key {
            "W" => width = value.parse().map_err(|_| Y4mError::BadHeader("W"))?,
            "H" => height = value.parse().map_err(|_| Y4mError::BadHeader("H"))?,
            "F" => {
                let (num, den) = value.split_once(':').ok_or(Y4mError::BadHeader("F"))?;
                fps = (
                    num.parse().map_err(|_| Y4mError::BadHeader("F num"))?,
                    den.parse().map_err(|_| Y4mError::BadHeader("F den"))?,
                );
            }
            "C" => colorspace = param.to_string(),
            _ => {}
        }
    }
    if width == 0 || height == 0 || fps.0 == 0 || fps.1 == 0 {
        return Err(Y4mError::BadHeader("missing W/H/F"));
    }
    let frame_bytes = width as usize * height as usize * 3 / 2;

    let mut frames = Vec::new();
    let mut at = header_end + 1;
    while at < data.len() {
        let line_end = data[at..]
            .iter()
            .position(|&b| b == b'\n')
            .ok_or(Y4mError::Truncated)?;
        if !data[at..].starts_with(b"FRAME") {
            return Err(Y4mError::BadHeader("expected FRAME"));
        }
        at += line_end + 1;
        let payload = data.get(at..at + frame_bytes).ok_or(Y4mError::Truncated)?;
        frames.push(payload.to_vec());
        at += frame_bytes;
    }
    Ok(DecodedY4m {
        width,
        height,
        fps,
        colorspace,
        frames,
    })
}
