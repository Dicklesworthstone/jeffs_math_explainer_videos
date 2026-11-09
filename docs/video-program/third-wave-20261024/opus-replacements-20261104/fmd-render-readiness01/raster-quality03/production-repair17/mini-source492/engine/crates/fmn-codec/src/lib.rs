//! Native image/media codec primitives (§14.2) — PNG, JPEG decode,
//! and the owned DEFLATE that everything compressed rides on.
//!
//! The governed closure (D1) admits no compression or image crates:
//! every byte parsed or produced here is owned code. Decoders are
//! **untrusted-input parsers** under §16.5/R14 — resource budgets are
//! declared before any decompression or allocation, refusals are typed,
//! and no input can make them hang or overallocate.
//!
//! - [`checksum`] — CRC-32 (PNG) and Adler-32 (zlib), owned.
//! - [`inflate`] — DEFLATE/zlib decompression with a hard output
//!   budget (the decompression-bomb refusal).
//! - [`deflate`] — deterministic DEFLATE/zlib compression with
//!   byte-aligned segment boundaries, the interlock W8CODEC2's
//!   deterministic parallel PNG encode composes over.
#![feature(portable_simd)]
#![cfg_attr(test, feature(test))]
#![forbid(unsafe_code)]

pub mod checksum;
pub mod deflate;
pub mod gif;
pub mod inflate;
pub mod jpeg;
pub mod png;
pub mod wav;
pub mod y4m;

pub use deflate::{CompressionLevel, deflate as deflate_bytes, deflate_segment, zlib_compress};
pub use gif::{GifError, GifStreamEncoder, encode_gif};
pub use inflate::{InflateError, inflate as inflate_bytes, zlib_decompress};
pub use jpeg::{DecodedJpeg, JpegError, JpegLimits, decode as decode_jpeg};
pub use png::{
    DecodedPng, PngError, PngLimits, decode as decode_png, encode_png_sequence, encode_rgba8,
    encode_rgba8_segmented,
};
pub use wav::{SampleFormat, WavAudio, WavError, WavLimits, decode_wav, encode_wav};
pub use y4m::{
    DecodedY4m, Y4mColorspace, Y4mError, Y4mWriter, append_y4m_frame_nv12, decode_y4m, y4m_header,
};
