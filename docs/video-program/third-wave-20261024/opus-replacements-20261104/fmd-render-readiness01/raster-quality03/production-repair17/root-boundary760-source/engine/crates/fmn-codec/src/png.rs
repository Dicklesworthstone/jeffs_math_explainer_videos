//! Owned PNG codec (§14.2): decode across the real ecosystem's full
//! matrix, encode for stills / sequences / the Studio stream.
//!
//! # Decode
//!
//! Grayscale ± alpha, truecolor ± alpha, indexed (+`tRNS`), bit depths
//! 1/2/4/8/16, Adam7 interlacing, all five filters — normalized to
//! **canonical RGBA8**. 16-bit samples quantize by rounding
//! (`(v·255 + 32767) / 65535`); sub-byte grayscale scales exactly
//! (×255, ×85, ×17).
//!
//! ## The gamma/sRGB chunk policy (defined precedence, documented)
//!
//! Pixel bytes are never resampled by the decoder. Color intent is
//! *reported* on the decoded image with this precedence: an `sRGB`
//! chunk wins over `gAMA`; absent both, samples are assumed
//! sRGB-encoded (the ecosystem's de-facto default). Consumers that
//! need linear light apply the transfer exactly once, in fmn-frame —
//! never here, never twice.
//!
//! ## The untrusted-input posture (§16.5, R14)
//!
//! Dimension and pixel-count limits are checked at `IHDR`; the exact
//! decompressed size is computed from the header geometry and declared
//! to the inflater **before** any decompression, so a bomb is refused
//! at the declared bound, not discovered at allocation. Chunk counts
//! are bounded, CRCs are verified, IDAT runs must be consecutive, and
//! unknown *critical* chunks are typed refusals.
//!
//! # Encode
//!
//! Canonical RGBA8 in, deterministic bytes out: per-row
//! minimum-sum-of-absolute-differences filter selection with fixed tie
//! order, the owned deterministic DEFLATE, and a fixed chunk sequence
//! (`IHDR`, `sRGB`, `gAMA`, `IDAT`, `IEND`) — the same image and level
//! produce the same file on every platform (self-goldens depend on it).

use crate::checksum::crc32;
use crate::deflate::{CompressionLevel, zlib_compress};
use crate::inflate::{InflateError, zlib_decompress};
use std::simd::cmp::SimdPartialOrd;
use std::simd::num::{SimdInt, SimdUint};
use std::simd::{Select, Simd, Swizzle};

/// Typed refusals of the PNG decoder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PngError {
    /// The signature bytes are not PNG's.
    NotPng,
    /// The stream ended mid-chunk.
    Truncated,
    /// A chunk CRC-32 check failed.
    ChunkCrc,
    /// More chunks than the declared budget.
    TooManyChunks {
        /// The configured chunk budget.
        limit: usize,
    },
    /// The image exceeds the declared pixel budget.
    TooLarge {
        /// The configured pixel budget.
        max_pixels: u64,
    },
    /// A malformed or contradictory IHDR field.
    BadIhdr(&'static str),
    /// A malformed PLTE chunk, or one missing where required.
    BadPalette(&'static str),
    /// A malformed tRNS chunk for the image's color type.
    BadTrns(&'static str),
    /// IDAT chunks must be consecutive; data was missing or scattered.
    BadIdat(&'static str),
    /// A chunk after IEND, or a malformed IEND.
    BadIend,
    /// An unknown chunk marked critical — skipping it would silently
    /// misrender, so it is refused by name.
    UnknownCritical([u8; 4]),
    /// A scanline filter byte outside 0..=4.
    BadFilter(u8),
    /// The pixel stream failed to decompress.
    Inflate(InflateError),
    /// The decompressed stream is not exactly the geometry's size.
    WrongDataSize {
        /// Bytes the header geometry requires.
        expected: usize,
        /// Bytes the stream actually inflated to.
        got: usize,
    },
}

impl std::fmt::Display for PngError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotPng => write!(f, "not a PNG (bad signature)"),
            Self::Truncated => write!(f, "png stream truncated mid-chunk"),
            Self::ChunkCrc => write!(f, "png chunk crc mismatch"),
            Self::TooManyChunks { limit } => {
                write!(f, "png exceeds the {limit}-chunk budget")
            }
            Self::TooLarge { max_pixels } => {
                write!(f, "png exceeds the {max_pixels}-pixel budget")
            }
            Self::BadIhdr(what) => write!(f, "malformed IHDR: {what}"),
            Self::BadPalette(what) => write!(f, "malformed palette: {what}"),
            Self::BadTrns(what) => write!(f, "malformed tRNS: {what}"),
            Self::BadIdat(what) => write!(f, "malformed IDAT run: {what}"),
            Self::BadIend => write!(f, "malformed or misplaced IEND"),
            Self::UnknownCritical(name) => {
                write!(f, "unknown critical chunk {:?}", name.map(|b| b as char))
            }
            Self::BadFilter(t) => write!(f, "scanline filter {t} outside 0..=4"),
            Self::Inflate(e) => write!(f, "pixel stream: {e}"),
            Self::WrongDataSize { expected, got } => write!(
                f,
                "decompressed pixel stream is {got} bytes, geometry requires {expected}"
            ),
        }
    }
}

impl std::error::Error for PngError {}

impl From<InflateError> for PngError {
    fn from(e: InflateError) -> Self {
        Self::Inflate(e)
    }
}

/// Decode resource budgets, declared before any work happens.
#[derive(Debug, Clone)]
pub struct PngLimits {
    /// Maximum `width × height` in pixels.
    pub max_pixels: u64,
    /// Maximum chunk count.
    pub max_chunks: usize,
}

impl Default for PngLimits {
    /// 268 megapixels (a 16384² frame) and 4096 chunks — far above any
    /// real asset, far below a bomb.
    fn default() -> Self {
        Self {
            max_pixels: 1 << 28,
            max_chunks: 4096,
        }
    }
}

/// The reported color intent (see the module docs for the precedence).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorIntent {
    /// An `sRGB` chunk was present (rendering intent kept verbatim).
    Srgb {
        /// The declared rendering intent byte.
        intent: u8,
    },
    /// Only a `gAMA` chunk was present; value is gamma × 100000.
    Gamma {
        /// Encoded gamma × 100000, as stored.
        gamma_100000: u32,
    },
    /// Neither chunk: samples are assumed sRGB-encoded.
    AssumedSrgb,
}

/// A decoded PNG, normalized to canonical RGBA8 (tight rows, output
/// orientation — row 0 is the top row, D-23).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedPng {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width × height × 4` bytes, RGBA, row-major.
    pub rgba: Vec<u8>,
    /// The source's color type byte (0/2/3/4/6), for provenance.
    pub source_color_type: u8,
    /// The source's bit depth (1/2/4/8/16), for provenance.
    pub source_bit_depth: u8,
    /// The reported color intent.
    pub intent: ColorIntent,
}

const SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

/// Adam7 pass geometry: x origin, y origin, x step, y step.
const ADAM7: [(u32, u32, u32, u32); 7] = [
    (0, 0, 8, 8),
    (4, 0, 8, 8),
    (0, 4, 4, 8),
    (2, 0, 4, 4),
    (0, 2, 2, 4),
    (1, 0, 2, 2),
    (0, 1, 1, 2),
];

const fn channels(color_type: u8) -> u32 {
    match color_type {
        0 | 3 => 1,
        4 => 2,
        2 => 3,
        _ => 4,
    }
}

/// Bytes per scanline for `width` pixels (excluding the filter byte).
const fn row_bytes(width: u32, bits_per_pixel: u32) -> usize {
    ((width as u64 * bits_per_pixel as u64).div_ceil(8)) as usize
}

/// Filter-reconstruction step distance in whole bytes.
const fn filter_bpp(bits_per_pixel: u32) -> usize {
    bits_per_pixel.div_ceil(8) as usize
}

struct Ihdr {
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: u8,
    interlaced: bool,
}

impl Ihdr {
    fn parse(data: &[u8]) -> Result<Self, PngError> {
        if data.len() != 13 {
            return Err(PngError::BadIhdr("length"));
        }
        let width = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let height = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let (bit_depth, color_type) = (data[8], data[9]);
        if width == 0 || height == 0 || width > 0x7fff_ffff || height > 0x7fff_ffff {
            return Err(PngError::BadIhdr("dimensions"));
        }
        let depth_ok = match color_type {
            0 => matches!(bit_depth, 1 | 2 | 4 | 8 | 16),
            3 => matches!(bit_depth, 1 | 2 | 4 | 8),
            2 | 4 | 6 => matches!(bit_depth, 8 | 16),
            _ => return Err(PngError::BadIhdr("color type")),
        };
        if !depth_ok {
            return Err(PngError::BadIhdr("bit depth for color type"));
        }
        if data[10] != 0 {
            return Err(PngError::BadIhdr("compression method"));
        }
        if data[11] != 0 {
            return Err(PngError::BadIhdr("filter method"));
        }
        let interlaced = match data[12] {
            0 => false,
            1 => true,
            _ => return Err(PngError::BadIhdr("interlace method")),
        };
        Ok(Self {
            width,
            height,
            bit_depth,
            color_type,
            interlaced,
        })
    }

    const fn bits_per_pixel(&self) -> u32 {
        channels(self.color_type) * self.bit_depth as u32
    }

    /// The pass list actually present in the stream: `(x0, y0, xstep,
    /// ystep, pass_width, pass_height)`, empty passes excluded.
    fn passes(&self) -> Vec<(u32, u32, u32, u32, u32, u32)> {
        if self.interlaced {
            ADAM7
                .iter()
                .map(|&(x0, y0, xs, ys)| {
                    let w = self.width.saturating_sub(x0).div_ceil(xs);
                    let h = self.height.saturating_sub(y0).div_ceil(ys);
                    (x0, y0, xs, ys, w, h)
                })
                .filter(|&(.., w, h)| w > 0 && h > 0)
                .collect()
        } else {
            vec![(0, 0, 1, 1, self.width, self.height)]
        }
    }

    /// Exact byte size of the filtered pixel stream — the inflate
    /// budget, computed from geometry alone.
    fn raw_stream_size(&self) -> usize {
        let bpp = self.bits_per_pixel();
        self.passes()
            .iter()
            .map(|&(.., w, h)| h as usize * (1 + row_bytes(w, bpp)))
            .sum()
    }
}

/// Undo one scanline's filter in place. `prev` is the reconstructed
/// previous scanline of the same pass (empty for the first row).
fn unfilter_row(filter: u8, row: &mut [u8], prev: &[u8], bpp: usize) -> Result<(), PngError> {
    match filter {
        0 => {}
        1 => unfilter_sub(row, bpp),
        2 => {
            if !prev.is_empty() {
                for (byte, &up) in row.iter_mut().zip(prev) {
                    *byte = byte.wrapping_add(up);
                }
            }
        }
        3 => {
            // Average's reconstructed-left dependency is nonlinear. A
            // four-channel std::simd trial regressed both throughput and
            // latency, so keep the boundary-split scalar recurrence.
            let left_free = bpp.min(row.len());
            if prev.is_empty() {
                for i in left_free..row.len() {
                    row[i] = row[i].wrapping_add(row[i - bpp] / 2);
                }
            } else {
                for (byte, &up) in row[..left_free].iter_mut().zip(&prev[..left_free]) {
                    *byte = byte.wrapping_add(up / 2);
                }
                for i in left_free..row.len() {
                    let average = (u16::from(row[i - bpp]) + u16::from(prev[i])) / 2;
                    row[i] = row[i].wrapping_add(average as u8);
                }
            }
        }
        4 => {
            if prev.is_empty() {
                unfilter_sub(row, bpp);
            } else {
                // RGB8's padded fourth lane pays off only once AVX2 is
                // selected for the whole artifact; the portable tier keeps
                // the scalar recurrence.
                #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
                if bpp == 3 {
                    unfilter_paeth_3(row, prev);
                    return Ok(());
                }
                match bpp {
                    4 => unfilter_paeth_4(row, prev),
                    6 => unfilter_paeth_6(row, prev),
                    8 => unfilter_paeth_8(row, prev),
                    _ => {
                        let left_free = bpp.min(row.len());
                        for (byte, &up) in row[..left_free].iter_mut().zip(&prev[..left_free]) {
                            *byte = byte.wrapping_add(up);
                        }
                        for i in left_free..row.len() {
                            let predictor = paeth_predictor(row[i - bpp], prev[i], prev[i - bpp]);
                            row[i] = row[i].wrapping_add(predictor);
                        }
                    }
                }
            }
        }
        t => return Err(PngError::BadFilter(t)),
    }
    Ok(())
}

// Sub is a bpp-strided inclusive prefix sum modulo 256. Sixteen bytes
// keep every supported power-of-two bpp aligned at chunk boundaries;
// the repeated terminal channels are the exact carry into the next
// Hillis-Steele scan. RGB8/RGB16 (bpp 3/6) retain the scalar recurrence.
type U8x16 = Simd<u8, 16>;
type U8x4 = Simd<u8, 4>;
type U8x8 = Simd<u8, 8>;

struct RepeatTail1;
impl Swizzle<16> for RepeatTail1 {
    const INDEX: [usize; 16] = [15; 16];
}

struct RepeatTail2;
impl Swizzle<16> for RepeatTail2 {
    const INDEX: [usize; 16] = [
        14, 15, 14, 15, 14, 15, 14, 15, 14, 15, 14, 15, 14, 15, 14, 15,
    ];
}

struct RepeatTail4;
impl Swizzle<16> for RepeatTail4 {
    const INDEX: [usize; 16] = [
        12, 13, 14, 15, 12, 13, 14, 15, 12, 13, 14, 15, 12, 13, 14, 15,
    ];
}

struct RepeatTail8;
impl Swizzle<16> for RepeatTail8 {
    const INDEX: [usize; 16] = [8, 9, 10, 11, 12, 13, 14, 15, 8, 9, 10, 11, 12, 13, 14, 15];
}

macro_rules! sub_unfilter_kernel {
    ($name:ident, $bpp:literal, $repeat:ident, $($shift:literal),+) => {
        fn $name(row: &mut [u8]) {
            let complete = row.len() / 16 * 16;
            let mut carry = U8x16::splat(0);
            for chunk in row[..complete].as_chunks_mut::<16>().0 {
                let mut values = U8x16::from_array(*chunk);
                $(
                    values += values.shift_elements_right::<$shift>(0);
                )+
                values += carry;
                carry = $repeat::swizzle(values);
                *chunk = values.to_array();
            }
            for i in complete.max($bpp)..row.len() {
                row[i] = row[i].wrapping_add(row[i - $bpp]);
            }
        }
    };
}

sub_unfilter_kernel!(unfilter_sub_1, 1, RepeatTail1, 1, 2, 4, 8);
sub_unfilter_kernel!(unfilter_sub_2, 2, RepeatTail2, 2, 4, 8);
sub_unfilter_kernel!(unfilter_sub_4, 4, RepeatTail4, 4, 8);
sub_unfilter_kernel!(unfilter_sub_8, 8, RepeatTail8, 8);

fn unfilter_sub(row: &mut [u8], bpp: usize) {
    match bpp {
        1 => unfilter_sub_1(row),
        2 => unfilter_sub_2(row),
        4 => unfilter_sub_4(row),
        8 => unfilter_sub_8(row),
        _ => {
            for i in bpp..row.len() {
                row[i] = row[i].wrapping_add(row[i - bpp]);
            }
        }
    }
}

macro_rules! paeth_unfilter_kernel {
    ($name:ident, $vector:ident, $predictor:ident, $bpp:literal) => {
        fn $name(row: &mut [u8], prev: &[u8]) {
            const BPP: usize = $bpp;
            let left_free = BPP.min(row.len());
            for (byte, &up) in row[..left_free].iter_mut().zip(&prev[..left_free]) {
                *byte = byte.wrapping_add(up);
            }

            let mut i = left_free;
            while i + BPP <= row.len() {
                let filtered = $vector::from_slice(&row[i..]);
                let left = $vector::from_slice(&row[i - BPP..]);
                let up = $vector::from_slice(&prev[i..]);
                let upper_left = $vector::from_slice(&prev[i - BPP..]);
                let predictor = $predictor(left, up, upper_left);
                (filtered + predictor).copy_to_slice(&mut row[i..i + BPP]);
                i += BPP;
            }
            for at in i..row.len() {
                let predictor = paeth_predictor(row[at - BPP], prev[at], prev[at - BPP]);
                row[at] = row[at].wrapping_add(predictor);
            }
        }
    };
}

// Paeth is nonlinear along a channel but independent between channels.
// Vectorize one whole pixel, preserving the reconstructed-left
// dependency between pixels. Two lanes measured slower and stay scalar;
// 4/6/8-byte pixels are profitable in both governed tiers.
paeth_unfilter_kernel!(unfilter_paeth_4, U8x4, paeth_predictor_4, 4);
paeth_unfilter_kernel!(unfilter_paeth_8, U8x8, paeth_predictor_8, 8);

#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
fn unfilter_paeth_3(row: &mut [u8], prev: &[u8]) {
    const BPP: usize = 3;
    let left_free = BPP.min(row.len());
    for (byte, &up) in row[..left_free].iter_mut().zip(&prev[..left_free]) {
        *byte = byte.wrapping_add(up);
    }

    let mut i = left_free;
    while i + BPP <= row.len() {
        let filtered = U8x4::from_array([row[i], row[i + 1], row[i + 2], 0]);
        let left = U8x4::from_array([row[i - BPP], row[i - BPP + 1], row[i - BPP + 2], 0]);
        let up = U8x4::from_array([prev[i], prev[i + 1], prev[i + 2], 0]);
        let upper_left = U8x4::from_array([prev[i - BPP], prev[i - BPP + 1], prev[i - BPP + 2], 0]);
        let decoded = (filtered + paeth_predictor_4(left, up, upper_left)).to_array();
        row[i..i + BPP].copy_from_slice(&decoded[..BPP]);
        i += BPP;
    }
    for at in i..row.len() {
        let predictor = paeth_predictor(row[at - BPP], prev[at], prev[at - BPP]);
        row[at] = row[at].wrapping_add(predictor);
    }
}

fn unfilter_paeth_6(row: &mut [u8], prev: &[u8]) {
    const BPP: usize = 6;
    let left_free = BPP.min(row.len());
    for (byte, &up) in row[..left_free].iter_mut().zip(&prev[..left_free]) {
        *byte = byte.wrapping_add(up);
    }

    let mut i = left_free;
    while i + BPP <= row.len() {
        let filtered = load_six(&row[i..]);
        let left = load_six(&row[i - BPP..]);
        let up = load_six(&prev[i..]);
        let upper_left = load_six(&prev[i - BPP..]);
        let decoded = (filtered + paeth_predictor_8(left, up, upper_left)).to_array();
        row[i..i + BPP].copy_from_slice(&decoded[..BPP]);
        i += BPP;
    }
    for at in i..row.len() {
        let predictor = paeth_predictor(row[at - BPP], prev[at], prev[at - BPP]);
        row[at] = row[at].wrapping_add(predictor);
    }
}

#[inline]
fn load_six(bytes: &[u8]) -> U8x8 {
    U8x8::from_array([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], 0, 0,
    ])
}

macro_rules! paeth_vector_predictor {
    ($name:ident, $vector:ident) => {
        #[inline]
        fn $name(a: $vector, b: $vector, c: $vector) -> $vector {
            let (a, b, c) = (a.cast::<i16>(), b.cast::<i16>(), c.cast::<i16>());
            let p = a + b - c;
            let (pa, pb, pc) = ((p - a).abs(), (p - b).abs(), (p - c).abs());
            let take_a = pa.simd_le(pb) & pa.simd_le(pc);
            let take_b = pb.simd_le(pc);
            take_a.select(a, take_b.select(b, c)).cast()
        }
    };
}

paeth_vector_predictor!(paeth_predictor_4, U8x4);
paeth_vector_predictor!(paeth_predictor_8, U8x8);

#[inline]
fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let (a, b, c) = (i32::from(a), i32::from(b), i32::from(c));
    let p = a + b - c;
    let (pa, pb, pc) = ((p - a).abs(), (p - b).abs(), (p - c).abs());
    if pa <= pb && pa <= pc {
        a as u8
    } else if pb <= pc {
        b as u8
    } else {
        c as u8
    }
}

/// Read sample `index` from a reconstructed scanline. Samples are
/// channel-interleaved; `index` counts samples, not pixels. Sub-byte
/// samples pack big-endian within a byte (leftmost pixel high).
fn read_sample(row: &[u8], index: usize, bit_depth: u8) -> u16 {
    match bit_depth {
        16 => u16::from_be_bytes([row[index * 2], row[index * 2 + 1]]),
        8 => u16::from(row[index]),
        d => {
            let per_byte = usize::from(8 / d);
            let byte = row[index / per_byte];
            let shift = 8 - ((index % per_byte) + 1) * usize::from(d);
            u16::from(byte >> shift) & ((1u16 << d) - 1)
        }
    }
}

/// Scale a source sample to 8 bits: exact expansion for sub-byte
/// depths, identity for 8, rounded quantization for 16.
fn scale_sample(v: u16, bit_depth: u8) -> u8 {
    match bit_depth {
        1 => (v * 255) as u8,
        2 => (v * 85) as u8,
        4 => (v * 17) as u8,
        8 => v as u8,
        _ => ((u32::from(v) * 255 + 32767) / 65535) as u8,
    }
}

/// The tRNS payload, interpreted per color type.
enum Transparency {
    None,
    /// Color type 3: per-palette-index alpha.
    Palette(Vec<u8>),
    /// Color type 0: the fully transparent gray sample (source depth).
    Gray(u16),
    /// Color type 2: the fully transparent RGB sample (source depth).
    Rgb([u16; 3]),
}

/// Decode a PNG to canonical RGBA8 under the given budgets.
// ubs:ignore — PNG decoding, not JWT decoding or validation.
pub fn decode(data: &[u8], limits: &PngLimits) -> Result<DecodedPng, PngError> {
    // ubs:ignore — Fixed public PNG magic bytes, not a secret or token.
    if data.len() < 8 || data[..8] != SIGNATURE {
        return Err(PngError::NotPng);
    }

    let mut cursor = 8usize;
    let mut chunk_count = 0usize;
    let mut ihdr: Option<Ihdr> = None;
    let mut palette: Option<Vec<[u8; 3]>> = None;
    let mut transparency = Transparency::None;
    let mut srgb: Option<u8> = None;
    let mut gama: Option<u32> = None;
    let mut idat: Vec<u8> = Vec::new();
    let mut idat_started = false;
    let mut idat_done = false;
    let mut iend = false;

    while cursor < data.len() {
        if iend {
            return Err(PngError::BadIend);
        }
        chunk_count += 1;
        if chunk_count > limits.max_chunks {
            return Err(PngError::TooManyChunks {
                limit: limits.max_chunks,
            });
        }
        if data.len() - cursor < 12 {
            return Err(PngError::Truncated);
        }
        let len = u32::from_be_bytes([
            data[cursor],
            data[cursor + 1],
            data[cursor + 2],
            data[cursor + 3],
        ]) as usize;
        if len > 0x7fff_ffff || data.len() - cursor - 12 < len {
            return Err(PngError::Truncated);
        }
        let name: [u8; 4] = [
            data[cursor + 4],
            data[cursor + 5],
            data[cursor + 6],
            data[cursor + 7],
        ];
        let body = &data[cursor + 8..cursor + 8 + len];
        let stored_crc = u32::from_be_bytes([
            data[cursor + 8 + len],
            data[cursor + 9 + len],
            data[cursor + 10 + len],
            data[cursor + 11 + len],
        ]);
        if crc32(&data[cursor + 4..cursor + 8 + len]) != stored_crc {
            return Err(PngError::ChunkCrc);
        }
        cursor += 12 + len;

        // IDAT chunks must form one consecutive run.
        if idat_started && !idat_done && &name != b"IDAT" {
            idat_done = true;
        }

        match &name {
            b"IHDR" => {
                if ihdr.is_some() || chunk_count != 1 {
                    return Err(PngError::BadIhdr("IHDR must be the first, only header"));
                }
                let parsed = Ihdr::parse(body)?;
                let pixels = u64::from(parsed.width) * u64::from(parsed.height);
                if pixels > limits.max_pixels {
                    return Err(PngError::TooLarge {
                        max_pixels: limits.max_pixels,
                    });
                }
                ihdr = Some(parsed);
            }
            b"PLTE" => {
                let header = ihdr.as_ref().ok_or(PngError::BadIhdr("missing"))?;
                if len == 0 || !len.is_multiple_of(3) || len / 3 > 256 {
                    return Err(PngError::BadPalette("entry count"));
                }
                if palette.is_some() || idat_started {
                    return Err(PngError::BadPalette("duplicate or late PLTE"));
                }
                if matches!(header.color_type, 0 | 4) {
                    return Err(PngError::BadPalette("PLTE forbidden for grayscale"));
                }
                palette = Some(body.as_chunks::<3>().0.to_vec());
            }
            b"tRNS" => {
                let header = ihdr.as_ref().ok_or(PngError::BadIhdr("missing"))?;
                if idat_started {
                    return Err(PngError::BadTrns("tRNS after IDAT"));
                }
                transparency = match header.color_type {
                    3 => {
                        let entries = palette
                            .as_ref()
                            .ok_or(PngError::BadTrns("tRNS before PLTE"))?
                            .len();
                        if len > entries {
                            return Err(PngError::BadTrns("more entries than palette"));
                        }
                        Transparency::Palette(body.to_vec())
                    }
                    0 => {
                        if len != 2 {
                            return Err(PngError::BadTrns("grayscale tRNS length"));
                        }
                        Transparency::Gray(u16::from_be_bytes([body[0], body[1]]))
                    }
                    2 => {
                        if len != 6 {
                            return Err(PngError::BadTrns("rgb tRNS length"));
                        }
                        Transparency::Rgb([
                            u16::from_be_bytes([body[0], body[1]]),
                            u16::from_be_bytes([body[2], body[3]]),
                            u16::from_be_bytes([body[4], body[5]]),
                        ])
                    }
                    _ => return Err(PngError::BadTrns("tRNS with an alpha color type")),
                };
            }
            b"sRGB" => {
                if len != 1 {
                    return Err(PngError::BadIhdr("sRGB length"));
                }
                srgb = Some(body[0]);
            }
            b"gAMA" => {
                if len != 4 {
                    return Err(PngError::BadIhdr("gAMA length"));
                }
                gama = Some(u32::from_be_bytes([body[0], body[1], body[2], body[3]]));
            }
            b"IDAT" => {
                if ihdr.is_none() {
                    return Err(PngError::BadIdat("IDAT before IHDR"));
                }
                if idat_done {
                    return Err(PngError::BadIdat("IDAT run is not consecutive"));
                }
                idat_started = true;
                idat.extend_from_slice(body);
            }
            b"IEND" => {
                if len != 0 {
                    return Err(PngError::BadIend);
                }
                iend = true;
            }
            _ => {
                // Ancillary (lowercase first letter) chunks are skipped;
                // unknown critical chunks are refused.
                if name[0] & 0x20 == 0 {
                    return Err(PngError::UnknownCritical(name));
                }
            }
        }
    }
    if !iend {
        return Err(PngError::Truncated);
    }
    let header = ihdr.ok_or(PngError::BadIhdr("missing"))?;
    if !idat_started {
        return Err(PngError::BadIdat("no IDAT"));
    }
    if header.color_type == 3 && palette.is_none() {
        return Err(PngError::BadPalette("indexed image without PLTE"));
    }

    // The budget is exact and declared before inflation begins.
    let expected = header.raw_stream_size();
    let raw = zlib_decompress(&idat, expected)?;
    if raw.len() != expected {
        return Err(PngError::WrongDataSize {
            expected,
            got: raw.len(),
        });
    }

    let intent = match (srgb, gama) {
        (Some(i), _) => ColorIntent::Srgb { intent: i },
        (None, Some(g)) => ColorIntent::Gamma { gamma_100000: g },
        (None, None) => ColorIntent::AssumedSrgb,
    };

    let mut rgba = vec![0u8; header.width as usize * header.height as usize * 4];
    let bpp_bits = header.bits_per_pixel();
    let bpp = filter_bpp(bpp_bits);
    let depth = header.bit_depth;
    let ch = channels(header.color_type) as usize;
    let palette = palette.unwrap_or_default();

    let mut offset = 0usize;
    for (x0, y0, xs, ys, pass_w, pass_h) in header.passes() {
        let line = row_bytes(pass_w, bpp_bits);
        let mut prev: Vec<u8> = Vec::new();
        for py in 0..pass_h {
            let filter = raw[offset];
            let mut row = raw[offset + 1..offset + 1 + line].to_vec();
            offset += 1 + line;
            unfilter_row(filter, &mut row, &prev, bpp)?;

            let out_y = y0 + py * ys;
            for px in 0..pass_w {
                let out_x = x0 + px * xs;
                let at = (out_y as usize * header.width as usize + out_x as usize) * 4;
                let base = px as usize * ch;
                let pixel: [u8; 4] = match header.color_type {
                    0 => {
                        let v = read_sample(&row, base, depth);
                        let g = scale_sample(v, depth);
                        let alpha = match transparency {
                            Transparency::Gray(t) if t == v => 0,
                            _ => 255,
                        };
                        [g, g, g, alpha]
                    }
                    2 => {
                        let r = read_sample(&row, base, depth);
                        let g = read_sample(&row, base + 1, depth);
                        let b = read_sample(&row, base + 2, depth);
                        let alpha = match transparency {
                            Transparency::Rgb(t) if t == [r, g, b] => 0,
                            _ => 255,
                        };
                        [
                            scale_sample(r, depth),
                            scale_sample(g, depth),
                            scale_sample(b, depth),
                            alpha,
                        ]
                    }
                    3 => {
                        let index = usize::from(read_sample(&row, base, depth));
                        let entry = palette
                            .get(index)
                            .ok_or(PngError::BadPalette("index out of range"))?;
                        let alpha = match &transparency {
                            Transparency::Palette(a) => a.get(index).copied().unwrap_or(255),
                            _ => 255,
                        };
                        [entry[0], entry[1], entry[2], alpha]
                    }
                    4 => {
                        let g = scale_sample(read_sample(&row, base, depth), depth);
                        let a = scale_sample(read_sample(&row, base + 1, depth), depth);
                        [g, g, g, a]
                    }
                    _ => [
                        scale_sample(read_sample(&row, base, depth), depth),
                        scale_sample(read_sample(&row, base + 1, depth), depth),
                        scale_sample(read_sample(&row, base + 2, depth), depth),
                        scale_sample(read_sample(&row, base + 3, depth), depth),
                    ],
                };
                rgba[at..at + 4].copy_from_slice(&pixel);
            }
            prev = row;
        }
    }

    Ok(DecodedPng {
        width: header.width,
        height: header.height,
        rgba,
        source_color_type: header.color_type,
        source_bit_depth: header.bit_depth,
        intent,
    })
}

/// Apply filter `filter` to `row` (with `prev` as the prior raw row)
/// into `out`.
fn apply_filter(filter: u8, row: &[u8], prev: &[u8], bpp: usize, out: &mut [u8]) {
    debug_assert_eq!(out.len(), row.len());
    let left_free = bpp.min(row.len());
    match filter {
        0 => out.copy_from_slice(row),
        1 => {
            out[..left_free].copy_from_slice(&row[..left_free]);
            for ((filtered, &byte), &left) in out[left_free..]
                .iter_mut()
                .zip(&row[left_free..])
                .zip(&row[..row.len() - left_free])
            {
                *filtered = byte.wrapping_sub(left);
            }
        }
        2 => {
            if prev.is_empty() {
                out.copy_from_slice(row);
            } else {
                for ((filtered, &byte), &up) in out.iter_mut().zip(row).zip(prev) {
                    *filtered = byte.wrapping_sub(up);
                }
            }
        }
        3 => {
            if prev.is_empty() {
                out[..left_free].copy_from_slice(&row[..left_free]);
                for ((filtered, &byte), &left) in out[left_free..]
                    .iter_mut()
                    .zip(&row[left_free..])
                    .zip(&row[..row.len() - left_free])
                {
                    *filtered = byte.wrapping_sub(left / 2);
                }
            } else {
                for ((filtered, &byte), &up) in out[..left_free]
                    .iter_mut()
                    .zip(&row[..left_free])
                    .zip(&prev[..left_free])
                {
                    *filtered = byte.wrapping_sub(up / 2);
                }
                for (((filtered, &byte), &left), &up) in out[left_free..]
                    .iter_mut()
                    .zip(&row[left_free..])
                    .zip(&row[..row.len() - left_free])
                    .zip(&prev[left_free..])
                {
                    *filtered = byte.wrapping_sub(((u16::from(left) + u16::from(up)) / 2) as u8);
                }
            }
        }
        4 => {
            if prev.is_empty() {
                out[..left_free].copy_from_slice(&row[..left_free]);
                for ((filtered, &byte), &left) in out[left_free..]
                    .iter_mut()
                    .zip(&row[left_free..])
                    .zip(&row[..row.len() - left_free])
                {
                    *filtered = byte.wrapping_sub(left);
                }
            } else {
                for ((filtered, &byte), &up) in out[..left_free]
                    .iter_mut()
                    .zip(&row[..left_free])
                    .zip(&prev[..left_free])
                {
                    *filtered = byte.wrapping_sub(up);
                }
                for ((((filtered, &byte), &left), &up), &upper_left) in out[left_free..]
                    .iter_mut()
                    .zip(&row[left_free..])
                    .zip(&row[..row.len() - left_free])
                    .zip(&prev[left_free..])
                    .zip(&prev[..prev.len() - left_free])
                {
                    *filtered = byte.wrapping_sub(paeth_predictor(left, up, upper_left));
                }
            }
        }
        _ => unreachable!("encoder filter id is fixed to 0..=4"),
    }
}

fn write_chunk(out: &mut Vec<u8>, name: &[u8; 4], body: &[u8]) {
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    out.extend_from_slice(name);
    out.extend_from_slice(body);
    let mut tagged = Vec::with_capacity(4 + body.len());
    tagged.extend_from_slice(name);
    tagged.extend_from_slice(body);
    out.extend_from_slice(&crc32(&tagged).to_be_bytes());
}

/// Encode canonical RGBA8 (`width × height × 4` bytes, tight rows,
/// output orientation) as a deterministic PNG.
///
/// # Panics
///
/// Panics if `rgba.len() != width * height * 4` — that is a caller
/// bug, not an input condition.
#[must_use]
pub fn encode_rgba8(width: u32, height: u32, rgba: &[u8], level: CompressionLevel) -> Vec<u8> {
    let filtered = filtered_stream(width, height, rgba);
    assemble_png(width, height, &zlib_compress(&filtered, level))
}

/// Min-SAD per-row filter selection into the filtered byte stream.
fn filtered_stream(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    assert_eq!(
        rgba.len(),
        width as usize * height as usize * 4,
        "rgba buffer does not match dimensions"
    );
    let line = width as usize * 4;
    let mut filtered = Vec::with_capacity((line + 1) * height as usize);
    let mut best = vec![0u8; line];
    let mut candidate = vec![0u8; line];
    for y in 0..height as usize {
        let row = &rgba[y * line..(y + 1) * line];
        let prev = if y == 0 {
            &[][..]
        } else {
            &rgba[(y - 1) * line..y * line]
        };
        // Minimum sum of absolute differences, fixed tie order 0..=4.
        let mut best_filter = 0u8;
        let mut best_score = u64::MAX;
        for filter in 0..=4u8 {
            apply_filter(filter, row, prev, 4, &mut candidate);
            let score: u64 = candidate
                .iter()
                .map(|&b| u64::from((b as i8).unsigned_abs()))
                .sum();
            if score < best_score {
                best_score = score;
                best_filter = filter;
                std::mem::swap(&mut best, &mut candidate);
            }
        }
        filtered.push(best_filter);
        filtered.extend_from_slice(&best);
    }
    filtered
}

fn assemble_png(width: u32, height: u32, idat: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&SIGNATURE);
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // 8-bit RGBA, no interlace
    write_chunk(&mut out, b"IHDR", &ihdr);
    write_chunk(&mut out, b"sRGB", &[0]); // perceptual
    write_chunk(&mut out, b"gAMA", &45455u32.to_be_bytes());
    write_chunk(&mut out, b"IDAT", idat);
    write_chunk(&mut out, b"IEND", &[]);
    out
}

/// The fixed DEFLATE segment size of the canonical sequence form —
/// boundaries are a function of content length alone, never of thread
/// count (§14.2's load-bearing determinism property).
const SEQUENCE_SEGMENT_BYTES: usize = 1 << 18;

/// Encode one frame in the canonical **segmented** form: the filtered
/// stream is compressed as fixed-boundary DEFLATE segments (the
/// W8CODEC interlock), so the byte-identical file can be produced
/// serially or by parallel workers compressing segments independently.
///
/// # Panics
/// As [`encode_rgba8`].
#[must_use]
pub fn encode_rgba8_segmented(
    width: u32,
    height: u32,
    rgba: &[u8],
    level: CompressionLevel,
) -> Vec<u8> {
    encode_rgba8_segmented_parallel(width, height, rgba, level, 1)
}

/// Thread fan-out clamp (W5 wasm tier 1, fm-l97).
///
/// Segmented encode output is byte-identical at every worker count by
/// construction (fixed segment boundaries, concatenation in index order), so
/// on `wasm32-unknown-unknown` — where `std::thread::scope`'s spawn panics,
/// no thread existing to spawn — any request collapses to the serial loop
/// with no effect on the bytes. Native builds honor the request unchanged.
#[inline]
const fn effective_workers(requested: usize) -> usize {
    if cfg!(target_arch = "wasm32") {
        1
    } else {
        requested
    }
}

/// Encode one canonical segmented frame, fanning its fixed DEFLATE segments
/// across `threads` workers.
///
/// Segment boundaries and concatenation order depend only on filtered byte
/// length, so output is byte-identical at every nonzero worker count. This is
/// the bounded-streaming sequence path: callers need retain only one frame,
/// rather than batching a whole render to obtain parallel encode.
///
/// # Panics
/// As [`encode_rgba8`].
#[must_use]
pub fn encode_rgba8_segmented_parallel(
    width: u32,
    height: u32,
    rgba: &[u8],
    level: CompressionLevel,
    threads: usize,
) -> Vec<u8> {
    let filtered = filtered_stream(width, height, rgba);
    let mut idat = crate::deflate::zlib_header(level).to_vec();
    let count = filtered.len().div_ceil(SEQUENCE_SEGMENT_BYTES).max(1);
    let workers = effective_workers(threads).max(1).min(count);
    if workers == 1 {
        for i in 0..count {
            let start = i * SEQUENCE_SEGMENT_BYTES;
            let end = (start + SEQUENCE_SEGMENT_BYTES).min(filtered.len());
            idat.extend_from_slice(&crate::deflate::deflate_segment(
                &filtered[..start],
                &filtered[start..end],
                level,
                i + 1 == count,
            ));
        }
    } else {
        let mut segments = vec![Vec::new(); count];
        std::thread::scope(|scope| {
            let mut handles = Vec::with_capacity(workers);
            for worker in 0..workers {
                let filtered = &filtered;
                handles.push(scope.spawn(move || {
                    let mut produced = Vec::new();
                    let mut index = worker;
                    while index < count {
                        let start = index * SEQUENCE_SEGMENT_BYTES;
                        let end = (start + SEQUENCE_SEGMENT_BYTES).min(filtered.len());
                        produced.push((
                            index,
                            crate::deflate::deflate_segment(
                                &filtered[..start],
                                &filtered[start..end],
                                level,
                                index + 1 == count,
                            ),
                        ));
                        index += workers;
                    }
                    produced
                }));
            }
            for handle in handles {
                for (index, bytes) in handle.join().expect("png segment worker panicked") {
                    segments[index] = bytes;
                }
            }
        });
        for segment in segments {
            idat.extend_from_slice(&segment);
        }
    }
    idat.extend_from_slice(&crate::checksum::adler32(&filtered).to_be_bytes());
    assemble_png(width, height, &idat)
}

/// Encode a frame sequence in the canonical segmented form, frames
/// fanned across `threads` workers — **bit-identical output at any
/// thread count** (PG-5 covers these bytes under `certified`).
///
/// # Panics
/// As [`encode_rgba8`], per frame.
#[must_use]
pub fn encode_png_sequence(
    width: u32,
    height: u32,
    frames: &[&[u8]],
    level: CompressionLevel,
    threads: usize,
) -> Vec<Vec<u8>> {
    let workers = effective_workers(threads).max(1).min(frames.len().max(1));
    if workers <= 1 {
        return frames
            .iter()
            .map(|frame| encode_rgba8_segmented(width, height, frame, level))
            .collect();
    }
    let mut out: Vec<Vec<u8>> = vec![Vec::new(); frames.len()];
    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(workers);
        for worker in 0..workers {
            handles.push(scope.spawn(move || {
                let mut produced = Vec::new();
                let mut index = worker;
                while index < frames.len() {
                    produced.push((
                        index,
                        encode_rgba8_segmented(width, height, frames[index], level),
                    ));
                    index += workers;
                }
                produced
            }));
        }
        for handle in handles {
            for (index, bytes) in handle.join().expect("png sequence worker panicked") {
                out[index] = bytes;
            }
        }
    });
    out
}

#[cfg(test)]
mod filter_tests {
    extern crate test;

    use super::{apply_filter, paeth_predictor, paeth_predictor_4, unfilter_row};
    use std::hint::black_box;
    use test::Bencher;

    const FILTERS: [u8; 5] = [0, 1, 2, 3, 4];
    const BPP_VALUES: [usize; 6] = [1, 2, 3, 4, 6, 8];
    const ROW_LENGTHS: [usize; 17] = [
        1, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17, 31, 32, 33, 255, 256, 257,
    ];

    fn seeded_bytes(len: usize, mut state: u32) -> Vec<u8> {
        (0..len)
            .map(|_| {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                (state >> 24) as u8
            })
            .collect()
    }

    fn scalar_filter(filter: u8, row: &[u8], prev: &[u8], bpp: usize) -> Vec<u8> {
        (0..row.len())
            .map(|i| {
                let left = if i >= bpp { row[i - bpp] } else { 0 };
                let up = if prev.is_empty() { 0 } else { prev[i] };
                match filter {
                    0 => row[i],
                    1 => row[i].wrapping_sub(left),
                    2 => row[i].wrapping_sub(up),
                    3 => row[i].wrapping_sub(((u16::from(left) + u16::from(up)) / 2) as u8),
                    4 => {
                        let upper_left = if i >= bpp && !prev.is_empty() {
                            prev[i - bpp]
                        } else {
                            0
                        };
                        row[i].wrapping_sub(paeth_predictor(left, up, upper_left))
                    }
                    _ => unreachable!("test filter id is fixed to 0..=4"),
                }
            })
            .collect()
    }

    #[test]
    fn every_filter_round_trips_the_bpp_and_width_matrix() {
        for len in ROW_LENGTHS {
            let row = seeded_bytes(len, len as u32 ^ 0x9e37_79b9);
            let previous = seeded_bytes(len, len as u32 ^ 0x243f_6a88);
            for bpp in BPP_VALUES {
                for filter in FILTERS {
                    for prev in [&[][..], previous.as_slice()] {
                        let mut filtered = vec![0u8; len];
                        apply_filter(filter, &row, prev, bpp, &mut filtered);
                        assert_eq!(
                            filtered,
                            scalar_filter(filter, &row, prev, bpp),
                            "encoded filter={filter}, bpp={bpp}, len={len}, prev={}",
                            !prev.is_empty()
                        );
                        unfilter_row(filter, &mut filtered, prev, bpp).unwrap();
                        assert_eq!(
                            filtered,
                            row,
                            "filter={filter}, bpp={bpp}, len={len}, prev={}",
                            !prev.is_empty()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn four_lane_paeth_is_exhaustive_over_every_byte_triple() {
        for a_base in (0u16..=255).step_by(4) {
            let a = [
                a_base as u8,
                (a_base + 1) as u8,
                (a_base + 2) as u8,
                (a_base + 3) as u8,
            ];
            for b in 0u8..=255 {
                for c in 0u8..=255 {
                    let got = paeth_predictor_4(a.into(), [b, b, b, b].into(), [c, c, c, c].into())
                        .to_array();
                    for lane in 0..4 {
                        assert_eq!(
                            got[lane],
                            paeth_predictor(a[lane], b, c),
                            "a={}, b={b}, c={c}",
                            a[lane]
                        );
                    }
                }
            }
        }
    }

    fn benchmark_apply(bench: &mut Bencher, filter: u8, len: usize) {
        let row = seeded_bytes(len, 0x9e37_79b9);
        let previous = seeded_bytes(len, 0x243f_6a88);
        let mut out = vec![0u8; len];
        bench.bytes = len as u64;
        bench.iter(|| {
            apply_filter(
                filter,
                black_box(&row),
                black_box(&previous),
                4,
                black_box(&mut out),
            );
            black_box(&out);
        });
    }

    fn benchmark_unfilter(bench: &mut Bencher, filter: u8, len: usize) {
        benchmark_unfilter_bpp(bench, filter, len, 4);
    }

    fn benchmark_unfilter_bpp(bench: &mut Bencher, filter: u8, len: usize, bpp: usize) {
        let mut row = seeded_bytes(len, 0x1319_8a2e);
        let previous = seeded_bytes(len, 0x243f_6a88);
        bench.bytes = len as u64;
        bench.iter(|| {
            unfilter_row(filter, black_box(&mut row), black_box(&previous), bpp)
                .expect("benchmark filter id is valid");
            black_box(&row);
        });
    }

    macro_rules! filter_bench {
        ($name:ident, $runner:ident, $filter:expr, $len:expr) => {
            #[bench]
            fn $name(bench: &mut Bencher) {
                $runner(bench, $filter, $len);
            }
        };
    }

    macro_rules! filter_bpp_bench {
        ($name:ident, $filter:expr, $bpp:expr) => {
            #[bench]
            fn $name(bench: &mut Bencher) {
                benchmark_unfilter_bpp(bench, $filter, 65_536, $bpp);
            }
        };
    }

    filter_bench!(apply_f0_0016, benchmark_apply, 0, 16);
    filter_bench!(apply_f0_0064, benchmark_apply, 0, 64);
    filter_bench!(apply_f0_0256, benchmark_apply, 0, 256);
    filter_bench!(apply_f0_4096, benchmark_apply, 0, 4_096);
    filter_bench!(apply_f0_65536, benchmark_apply, 0, 65_536);
    filter_bench!(apply_f1_0016, benchmark_apply, 1, 16);
    filter_bench!(apply_f1_0064, benchmark_apply, 1, 64);
    filter_bench!(apply_f1_0256, benchmark_apply, 1, 256);
    filter_bench!(apply_f1_4096, benchmark_apply, 1, 4_096);
    filter_bench!(apply_f1_65536, benchmark_apply, 1, 65_536);
    filter_bench!(apply_f2_0016, benchmark_apply, 2, 16);
    filter_bench!(apply_f2_0064, benchmark_apply, 2, 64);
    filter_bench!(apply_f2_0256, benchmark_apply, 2, 256);
    filter_bench!(apply_f2_4096, benchmark_apply, 2, 4_096);
    filter_bench!(apply_f2_65536, benchmark_apply, 2, 65_536);
    filter_bench!(apply_f3_0016, benchmark_apply, 3, 16);
    filter_bench!(apply_f3_0064, benchmark_apply, 3, 64);
    filter_bench!(apply_f3_0256, benchmark_apply, 3, 256);
    filter_bench!(apply_f3_4096, benchmark_apply, 3, 4_096);
    filter_bench!(apply_f3_65536, benchmark_apply, 3, 65_536);
    filter_bench!(apply_f4_0016, benchmark_apply, 4, 16);
    filter_bench!(apply_f4_0064, benchmark_apply, 4, 64);
    filter_bench!(apply_f4_0256, benchmark_apply, 4, 256);
    filter_bench!(apply_f4_4096, benchmark_apply, 4, 4_096);
    filter_bench!(apply_f4_65536, benchmark_apply, 4, 65_536);

    filter_bench!(unfilter_f0_0016, benchmark_unfilter, 0, 16);
    filter_bench!(unfilter_f0_0064, benchmark_unfilter, 0, 64);
    filter_bench!(unfilter_f0_0256, benchmark_unfilter, 0, 256);
    filter_bench!(unfilter_f0_4096, benchmark_unfilter, 0, 4_096);
    filter_bench!(unfilter_f0_65536, benchmark_unfilter, 0, 65_536);
    filter_bench!(unfilter_f1_0016, benchmark_unfilter, 1, 16);
    filter_bench!(unfilter_f1_0064, benchmark_unfilter, 1, 64);
    filter_bench!(unfilter_f1_0256, benchmark_unfilter, 1, 256);
    filter_bench!(unfilter_f1_4096, benchmark_unfilter, 1, 4_096);
    filter_bench!(unfilter_f1_65536, benchmark_unfilter, 1, 65_536);
    filter_bench!(unfilter_f2_0016, benchmark_unfilter, 2, 16);
    filter_bench!(unfilter_f2_0064, benchmark_unfilter, 2, 64);
    filter_bench!(unfilter_f2_0256, benchmark_unfilter, 2, 256);
    filter_bench!(unfilter_f2_4096, benchmark_unfilter, 2, 4_096);
    filter_bench!(unfilter_f2_65536, benchmark_unfilter, 2, 65_536);
    filter_bench!(unfilter_f3_0016, benchmark_unfilter, 3, 16);
    filter_bench!(unfilter_f3_0064, benchmark_unfilter, 3, 64);
    filter_bench!(unfilter_f3_0256, benchmark_unfilter, 3, 256);
    filter_bench!(unfilter_f3_4096, benchmark_unfilter, 3, 4_096);
    filter_bench!(unfilter_f3_65536, benchmark_unfilter, 3, 65_536);
    filter_bench!(unfilter_f4_0016, benchmark_unfilter, 4, 16);
    filter_bench!(unfilter_f4_0064, benchmark_unfilter, 4, 64);
    filter_bench!(unfilter_f4_0256, benchmark_unfilter, 4, 256);
    filter_bench!(unfilter_f4_4096, benchmark_unfilter, 4, 4_096);
    filter_bench!(unfilter_f4_65536, benchmark_unfilter, 4, 65_536);

    filter_bpp_bench!(unfilter_sub_bpp1_65536, 1, 1);
    filter_bpp_bench!(unfilter_sub_bpp2_65536, 1, 2);
    filter_bpp_bench!(unfilter_sub_bpp3_65536, 1, 3);
    filter_bpp_bench!(unfilter_sub_bpp6_65536, 1, 6);
    filter_bpp_bench!(unfilter_sub_bpp8_65536, 1, 8);
    filter_bpp_bench!(unfilter_paeth_bpp1_65536, 4, 1);
    filter_bpp_bench!(unfilter_paeth_bpp2_65536, 4, 2);
    filter_bpp_bench!(unfilter_paeth_bpp3_65536, 4, 3);
    filter_bpp_bench!(unfilter_paeth_bpp6_65536, 4, 6);
    filter_bpp_bench!(unfilter_paeth_bpp8_65536, 4, 8);
}
