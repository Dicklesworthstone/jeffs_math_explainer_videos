//! Pixel formats and negotiated frame layouts (§14.1, §14.3).
//!
//! The format set is exactly the negotiated-sink vocabulary of D-23:
//! `Rgba8`/`Bgra8` for alpha and compatibility sinks, `Rgba16F` for
//! linear-light intermediates where quality demands, `Nv12` for
//! ordinary 8-bit video, `P010` for 10-bit output — and canonical RGBA
//! (that is, `Rgba8` produced by the certified transfer kernel) for
//! certified artifacts.
//!
//! Row-stride discipline: sinks negotiate stride, internal buffers may
//! pad. A [`FrameLayout`] is the negotiated result — per-plane strides
//! and offsets over one contiguous allocation — validated once, so the
//! kernels and pools downstream never re-derive geometry.

use crate::FrameError;

/// The pixel formats fmn-frame owns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelFormat {
    /// 8-bit RGBA, one plane, 4 bytes per pixel.
    Rgba8,
    /// 8-bit BGRA (compatibility sinks), one plane, 4 bytes per pixel.
    Bgra8,
    /// Linear-light binary16 RGBA, one plane, 8 bytes per pixel
    /// (channel samples are raw binary16 bits, little-endian).
    Rgba16F,
    /// 4:2:0 Y′CbCr, two planes: full-res luma, interleaved half-res
    /// CbCr. 8-bit samples.
    Nv12,
    /// 4:2:0 Y′CbCr, two planes, 10-bit samples MSB-aligned in
    /// little-endian `u16`s (low 6 bits zero).
    P010,
}

impl PixelFormat {
    /// Number of planes in this format.
    #[must_use]
    pub const fn plane_count(self) -> usize {
        match self {
            Self::Rgba8 | Self::Bgra8 | Self::Rgba16F => 1,
            Self::Nv12 | Self::P010 => 2,
        }
    }

    /// Whether 4:2:0 subsampling requires even frame dimensions.
    #[must_use]
    pub const fn requires_even_dimensions(self) -> bool {
        matches!(self, Self::Nv12 | Self::P010)
    }

    /// The sample size in bytes — every plane stride must be a multiple
    /// of this (a torn `u16` sample is not a negotiable stride).
    #[must_use]
    pub const fn sample_size(self) -> usize {
        match self {
            Self::Rgba8 | Self::Bgra8 | Self::Nv12 => 1,
            Self::Rgba16F | Self::P010 => 2,
        }
    }

    /// Rows in `plane` for a frame `height` rows tall.
    #[must_use]
    pub const fn plane_rows(self, height: u32, plane: usize) -> u32 {
        match (self, plane) {
            (Self::Nv12 | Self::P010, 1) => height / 2,
            _ => height,
        }
    }

    /// The minimum (payload) row width of `plane` in bytes.
    ///
    /// Returns `None` on arithmetic overflow.
    #[must_use]
    pub fn min_row_bytes(self, width: u32, plane: usize) -> Option<usize> {
        let w = width as usize;
        match (self, plane) {
            (Self::Rgba8 | Self::Bgra8, 0) => w.checked_mul(4),
            (Self::Rgba16F, 0) => w.checked_mul(8),
            // NV12 luma is w bytes; its chroma row is w/2 CbCr pairs of
            // 2 bytes = w bytes again.
            (Self::Nv12, 0 | 1) => Some(w),
            // P010 doubles both for 16-bit containers.
            (Self::P010, 0 | 1) => w.checked_mul(2),
            _ => None,
        }
    }
}

/// Y′CbCr quantization range semantics (§14.1: range is part of the
/// format contract, never a guess).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorRange {
    /// Limited ("video", "MPEG") range: 8-bit Y′ ∈ [16, 235],
    /// chroma ∈ [16, 240]; 10-bit Y′ ∈ [64, 940], chroma ∈ [64, 960].
    Limited,
    /// Full ("PC", "JPEG") range: the whole code space.
    Full,
}

/// 4:2:0 chroma sample siting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChromaSiting {
    /// Horizontally co-sited with the left luma column, vertically
    /// interstitial — the BT.709 / H.264 default (`chroma_loc` "left").
    /// The kernel averages the two left-column pixels of each 2×2 quad.
    Left,
    /// Interstitial in both axes ("center", JPEG style). The kernel box-
    /// averages the full 2×2 quad.
    Center,
}

/// A validated, negotiated frame geometry: format, dimensions, and
/// per-plane strides/offsets over one contiguous allocation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FrameLayout {
    format: PixelFormat,
    width: u32,
    height: u32,
    strides: [usize; 2],
    offsets: [usize; 2],
    total_bytes: usize,
}

impl FrameLayout {
    /// A layout with minimal (unpadded) strides.
    pub fn tight(format: PixelFormat, width: u32, height: u32) -> Result<Self, FrameError> {
        Self::with_row_alignment(format, width, height, 1)
    }

    /// A layout whose row strides are padded up to `alignment` bytes
    /// (must be a power of two) — the negotiation path for sinks that
    /// require aligned rows.
    pub fn with_row_alignment(
        format: PixelFormat,
        width: u32,
        height: u32,
        alignment: usize,
    ) -> Result<Self, FrameError> {
        if !alignment.is_power_of_two() {
            return Err(FrameError::BadAlignment { alignment });
        }
        let mut strides = [0usize; 2];
        for (plane, stride) in strides.iter_mut().enumerate().take(format.plane_count()) {
            let min = format
                .min_row_bytes(width, plane)
                .ok_or(FrameError::TooLarge)?;
            *stride =
                min.checked_add(alignment - 1).ok_or(FrameError::TooLarge)? & !(alignment - 1);
        }
        Self::with_strides(format, width, height, &strides[..format.plane_count()])
    }

    /// A layout with explicitly negotiated per-plane strides.
    ///
    /// Each stride must cover the payload row and be a multiple of the
    /// format's sample size.
    pub fn with_strides(
        format: PixelFormat,
        width: u32,
        height: u32,
        strides: &[usize],
    ) -> Result<Self, FrameError> {
        if width == 0 || height == 0 {
            return Err(FrameError::ZeroDimension);
        }
        if format.requires_even_dimensions()
            && (!width.is_multiple_of(2) || !height.is_multiple_of(2))
        {
            return Err(FrameError::OddDimensions {
                format,
                width,
                height,
            });
        }
        let planes = format.plane_count();
        if strides.len() != planes {
            return Err(FrameError::WrongStrideCount {
                expected: planes,
                got: strides.len(),
            });
        }
        let mut fixed = [0usize; 2];
        let mut offsets = [0usize; 2];
        let mut total = 0usize;
        for (plane, &stride) in strides.iter().enumerate() {
            let min = format
                .min_row_bytes(width, plane)
                .ok_or(FrameError::TooLarge)?;
            if stride < min {
                return Err(FrameError::StrideTooSmall { plane, stride, min });
            }
            if stride % format.sample_size() != 0 {
                return Err(FrameError::StrideMisaligned {
                    plane,
                    stride,
                    sample_size: format.sample_size(),
                });
            }
            fixed[plane] = stride;
            offsets[plane] = total;
            let plane_bytes = stride
                .checked_mul(format.plane_rows(height, plane) as usize)
                .ok_or(FrameError::TooLarge)?;
            total = total.checked_add(plane_bytes).ok_or(FrameError::TooLarge)?;
        }
        Ok(Self {
            format,
            width,
            height,
            strides: fixed,
            offsets,
            total_bytes: total,
        })
    }

    /// The pixel format.
    #[must_use]
    pub const fn format(&self) -> PixelFormat {
        self.format
    }

    /// Frame width in pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Frame height in pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// The row stride of `plane` in bytes.
    #[must_use]
    pub const fn stride(&self, plane: usize) -> usize {
        self.strides[plane]
    }

    /// Byte offset of `plane` inside the frame allocation.
    #[must_use]
    pub const fn plane_offset(&self, plane: usize) -> usize {
        self.offsets[plane]
    }

    /// Total bytes of `plane`.
    #[must_use]
    pub fn plane_bytes(&self, plane: usize) -> usize {
        self.strides[plane] * self.format.plane_rows(self.height, plane) as usize
    }

    /// Total bytes of the frame allocation.
    #[must_use]
    pub const fn total_bytes(&self) -> usize {
        self.total_bytes
    }
}
