//! Kitty-graphics and sixel terminal preview adapters.
//!
//! Both encoders write through an ordinary [`Write`] capability; they never
//! inspect ambient terminal state or spawn a helper. Kitty receives a
//! deterministic PNG in bounded base64 chunks. Sixel uses a fixed 6×6×6 RGB
//! cube and transparent pixels as unpainted background, so identical RGBA8
//! bytes produce identical terminal bytes.

use std::fmt;
use std::io::Write;

use fmn_codec::{CompressionLevel, PngError, PngLimits, decode_png, encode_rgba8};

/// Terminal image protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalProtocol {
    /// Kitty graphics protocol, PNG payload.
    Kitty,
    /// DEC sixel with a fixed 216-color cube.
    Sixel,
}

/// Terminal encoding budgets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TuiLimits {
    /// Maximum source pixels.
    pub max_pixels: usize,
    /// Maximum terminal escape-sequence bytes.
    pub max_encoded_bytes: usize,
    /// Maximum base64 characters in one Kitty application command.
    pub kitty_chunk_bytes: usize,
}

impl Default for TuiLimits {
    fn default() -> Self {
        Self {
            max_pixels: 3840 * 2160,
            max_encoded_bytes: 128 * 1024 * 1024,
            kitty_chunk_bytes: 4096,
        }
    }
}

impl TuiLimits {
    fn validate(self) -> Result<Self, TuiError> {
        if self.max_pixels == 0 || self.max_encoded_bytes == 0 || self.kitty_chunk_bytes < 4 {
            Err(TuiError::InvalidLimits)
        } else {
            Ok(self)
        }
    }
}

/// Bounded terminal preview encoder.
#[derive(Clone, Copy, Debug)]
pub struct TerminalPreview {
    protocol: TerminalProtocol,
    limits: TuiLimits,
}

impl TerminalPreview {
    /// Construct an encoder.
    pub fn new(protocol: TerminalProtocol, limits: TuiLimits) -> Result<Self, TuiError> {
        Ok(Self {
            protocol,
            limits: limits.validate()?,
        })
    }

    /// Selected terminal protocol.
    #[must_use]
    pub const fn protocol(&self) -> TerminalProtocol {
        self.protocol
    }

    /// Encode and write a tight top-row-first RGBA8 frame.
    pub fn write_rgba8(
        &self,
        writer: &mut impl Write,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Result<(), TuiError> {
        validate_rgba(width, height, rgba, self.limits)?;
        match self.protocol {
            TerminalProtocol::Kitty => {
                let png = encode_rgba8(width, height, rgba, CompressionLevel::Fast);
                write_kitty_png(writer, &png, self.limits)?;
            }
            TerminalProtocol::Sixel => {
                write_sixel(writer, width, height, rgba, self.limits)?;
            }
        }
        writer.flush()?;
        Ok(())
    }

    /// Write already-encoded PNG through Kitty.
    pub fn write_png(&self, writer: &mut impl Write, png: &[u8]) -> Result<(), TuiError> {
        if self.protocol != TerminalProtocol::Kitty {
            return Err(TuiError::PngRequiresKitty);
        }
        if !png.starts_with(b"\x89PNG\r\n\x1a\n") {
            return Err(TuiError::InvalidPngSignature);
        }
        kitty_encoded_size(png.len(), self.limits)?;
        let max_pixels = u64::try_from(self.limits.max_pixels).unwrap_or(u64::MAX);
        decode_png(
            png,
            &PngLimits {
                max_pixels,
                ..PngLimits::default()
            },
        )?;
        write_kitty_png(writer, png, self.limits)?;
        writer.flush()?;
        Ok(())
    }
}

fn validate_rgba(width: u32, height: u32, rgba: &[u8], limits: TuiLimits) -> Result<(), TuiError> {
    if width == 0 || height == 0 {
        return Err(TuiError::ZeroDimensions);
    }
    let pixels = usize::try_from(width)
        .ok()
        .and_then(|width| {
            usize::try_from(height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .ok_or(TuiError::DimensionOverflow)?;
    if pixels > limits.max_pixels {
        return Err(TuiError::PixelLimit {
            limit: limits.max_pixels,
            needed: pixels,
        });
    }
    let expected = pixels.checked_mul(4).ok_or(TuiError::DimensionOverflow)?;
    if rgba.len() != expected {
        return Err(TuiError::RgbaLength {
            expected,
            found: rgba.len(),
        });
    }
    Ok(())
}

fn write_kitty_png(writer: &mut impl Write, png: &[u8], limits: TuiLimits) -> Result<(), TuiError> {
    const BASE64_WRITE_BUFFER_BYTES: usize = 4096;

    let (chunk_bytes, chunks, _) = kitty_encoded_size(png.len(), limits)?;
    let raw_chunk_bytes = (chunk_bytes / 4)
        .checked_mul(3)
        .ok_or(TuiError::EncodedSizeOverflow)?;
    let mut encoded = [0u8; BASE64_WRITE_BUFFER_BYTES];
    debug_assert_eq!(png.chunks(raw_chunk_bytes).len(), chunks);
    for (index, chunk) in png.chunks(raw_chunk_bytes).enumerate() {
        writer.write_all(b"\x1b_G")?;
        if index == 0 {
            writer.write_all(b"a=T,f=100,t=d,q=2,")?;
        }
        let more = index + 1 < chunks;
        writer.write_all(if more { b"m=1;" } else { b"m=0;" })?;
        let mut encoded_len = 0;
        for block in chunk.chunks(3) {
            if encoded_len == encoded.len() {
                writer.write_all(&encoded)?;
                encoded_len = 0;
            }
            encoded[encoded_len..encoded_len + 4].copy_from_slice(&base64_block(block));
            encoded_len += 4;
        }
        if encoded_len != 0 {
            writer.write_all(&encoded[..encoded_len])?;
        }
        writer.write_all(b"\x1b\\")?;
    }
    Ok(())
}

fn kitty_encoded_size(
    png_len: usize,
    limits: TuiLimits,
) -> Result<(usize, usize, usize), TuiError> {
    const FIRST_CHUNK_FRAMING_BYTES: usize =
        b"\x1b_G".len() + b"a=T,f=100,t=d,q=2,".len() + b"m=0;".len() + b"\x1b\\".len();
    const FOLLOWING_CHUNK_FRAMING_BYTES: usize = b"\x1b_G".len() + b"m=0;".len() + b"\x1b\\".len();

    let chunk_bytes = limits.kitty_chunk_bytes & !3;
    let base64_bytes = png_len
        .div_ceil(3)
        .checked_mul(4)
        .ok_or(TuiError::EncodedSizeOverflow)?;
    let chunks = base64_bytes.div_ceil(chunk_bytes);
    let framing_bytes = if chunks == 0 {
        0
    } else {
        (chunks - 1)
            .checked_mul(FOLLOWING_CHUNK_FRAMING_BYTES)
            .and_then(|following| following.checked_add(FIRST_CHUNK_FRAMING_BYTES))
            .ok_or(TuiError::EncodedSizeOverflow)?
    };
    let encoded_size = base64_bytes
        .checked_add(framing_bytes)
        .ok_or(TuiError::EncodedSizeOverflow)?;
    if encoded_size > limits.max_encoded_bytes {
        return Err(TuiError::EncodedLimit {
            limit: limits.max_encoded_bytes,
            needed: encoded_size,
        });
    }
    Ok((chunk_bytes, chunks, encoded_size))
}

fn write_sixel(
    writer: &mut impl Write,
    width: u32,
    height: u32,
    rgba: &[u8],
    limits: TuiLimits,
) -> Result<(), TuiError> {
    const PIXEL_WRITE_BUFFER_BYTES: usize = 4096;

    let (width_usize, height_usize, used) = sixel_preflight(width, height, rgba, limits)?;
    writer.write_all(b"\x1bP0;0;0q")?;
    write!(writer, "\"1;1;{width};{height}")?;
    for (index, present) in used.into_iter().enumerate() {
        if !present {
            continue;
        }
        let red = (index / 36) * 20;
        let green = ((index / 6) % 6) * 20;
        let blue = (index % 6) * 20;
        write!(writer, "#{index};2;{red};{green};{blue}")?;
    }

    let mut pixels = [0u8; PIXEL_WRITE_BUFFER_BYTES];
    for band_y in (0..height_usize).step_by(6) {
        let band_palette = band_palette(rgba, width_usize, height_usize, band_y);
        let mut any_color = false;
        for (color, present) in band_palette.into_iter().enumerate() {
            if !present {
                continue;
            }
            if any_color {
                writer.write_all(b"$")?;
            }
            any_color = true;
            write!(writer, "#{color}")?;
            let mut pixel_len = 0;
            for x in 0..width_usize {
                let mut bits = 0u8;
                for bit in 0..6 {
                    let y = band_y + bit;
                    if y >= height_usize {
                        break;
                    }
                    let at = (y * width_usize + x) * 4;
                    let pixel = &rgba[at..at + 4];
                    if pixel[3] >= 128
                        && usize::from(cube_index(pixel[0], pixel[1], pixel[2])) == color
                    {
                        bits |= 1 << bit;
                    }
                }
                pixels[pixel_len] = 63 + bits;
                pixel_len += 1;
                if pixel_len == pixels.len() {
                    writer.write_all(&pixels)?;
                    pixel_len = 0;
                }
            }
            if pixel_len != 0 {
                writer.write_all(&pixels[..pixel_len])?;
            }
        }
        writer.write_all(b"-")?;
    }
    writer.write_all(b"\x1b\\")?;
    Ok(())
}

fn sixel_preflight(
    width: u32,
    height: u32,
    rgba: &[u8],
    limits: TuiLimits,
) -> Result<(usize, usize, [bool; 216]), TuiError> {
    let width_usize = usize::try_from(width).map_err(|_| TuiError::DimensionOverflow)?;
    let height_usize = usize::try_from(height).map_err(|_| TuiError::DimensionOverflow)?;
    let mut used = [false; 216];
    for pixel in rgba.as_chunks::<4>().0 {
        if pixel[3] >= 128 {
            used[usize::from(cube_index(pixel[0], pixel[1], pixel[2]))] = true;
        }
    }

    let mut encoded_size = b"\x1bP0;0;0q".len();
    add_sixel_size(
        &mut encoded_size,
        b"\"1;1;".len() + decimal_digits(width_usize) + 1 + decimal_digits(height_usize),
    )?;
    for (index, present) in used.iter().copied().enumerate() {
        if !present {
            continue;
        }
        let red = (index / 36) * 20;
        let green = ((index / 6) % 6) * 20;
        let blue = (index % 6) * 20;
        add_sixel_size(
            &mut encoded_size,
            1 + decimal_digits(index)
                + b";2;".len()
                + decimal_digits(red)
                + 1
                + decimal_digits(green)
                + 1
                + decimal_digits(blue),
        )?;
    }

    for band_y in (0..height_usize).step_by(6) {
        let band_palette = band_palette(rgba, width_usize, height_usize, band_y);
        let mut any_color = false;
        for (color, present) in band_palette.into_iter().enumerate() {
            if !present {
                continue;
            }
            if any_color {
                add_sixel_size(&mut encoded_size, 1)?;
            }
            any_color = true;
            add_sixel_size(&mut encoded_size, 1 + decimal_digits(color) + width_usize)?;
        }
        add_sixel_size(&mut encoded_size, 1)?;
    }
    add_sixel_size(&mut encoded_size, b"\x1b\\".len())?;
    if encoded_size > limits.max_encoded_bytes {
        return Err(TuiError::EncodedLimit {
            limit: limits.max_encoded_bytes,
            needed: encoded_size,
        });
    }
    Ok((width_usize, height_usize, used))
}

fn band_palette(rgba: &[u8], width: usize, height: usize, band_y: usize) -> [bool; 216] {
    let mut palette = [false; 216];
    for y in band_y..band_y.saturating_add(6).min(height) {
        for x in 0..width {
            let at = (y * width + x) * 4;
            let pixel = &rgba[at..at + 4];
            if pixel[3] >= 128 {
                palette[usize::from(cube_index(pixel[0], pixel[1], pixel[2]))] = true;
            }
        }
    }
    palette
}

const fn cube_index(red: u8, green: u8, blue: u8) -> u8 {
    let red = red / 51;
    let green = green / 51;
    let blue = blue / 51;
    red * 36 + green * 6 + blue
}

fn add_sixel_size(total: &mut usize, additional: usize) -> Result<(), TuiError> {
    *total = total
        .checked_add(additional)
        .ok_or(TuiError::EncodedSizeOverflow)?;
    Ok(())
}

fn decimal_digits(mut value: usize) -> usize {
    let mut digits = 1;
    while value >= 10 {
        value /= 10;
        digits += 1;
    }
    digits
}

fn base64_block(bytes: &[u8]) -> [u8; 4] {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    debug_assert!((1..=3).contains(&bytes.len()));
    let first = bytes[0];
    let second = bytes.get(1).copied().unwrap_or(0);
    let third = bytes.get(2).copied().unwrap_or(0);
    [
        ALPHABET[usize::from(first >> 2)],
        ALPHABET[usize::from(((first & 0x03) << 4) | (second >> 4))],
        if bytes.len() > 1 {
            ALPHABET[usize::from(((second & 0x0f) << 2) | (third >> 6))]
        } else {
            b'='
        },
        if bytes.len() > 2 {
            ALPHABET[usize::from(third & 0x3f)]
        } else {
            b'='
        },
    ]
}

/// Terminal adapter failure.
#[derive(Debug)]
pub enum TuiError {
    /// I/O failure.
    Io(std::io::Error),
    /// At least one ceiling was zero or unusable.
    InvalidLimits,
    /// Frame width or height was zero.
    ZeroDimensions,
    /// Dimension arithmetic overflowed.
    DimensionOverflow,
    /// Source frame exceeded the pixel ceiling.
    PixelLimit {
        /// Ceiling.
        limit: usize,
        /// Requested pixels.
        needed: usize,
    },
    /// RGBA byte count did not match the dimensions.
    RgbaLength {
        /// Required bytes.
        expected: usize,
        /// Supplied bytes.
        found: usize,
    },
    /// Encoded-size arithmetic overflowed.
    EncodedSizeOverflow,
    /// Terminal output exceeded its ceiling.
    EncodedLimit {
        /// Ceiling.
        limit: usize,
        /// Bytes required so far.
        needed: usize,
    },
    /// A PNG was passed to a sixel encoder.
    PngRequiresKitty,
    /// Input did not begin with the PNG signature.
    InvalidPngSignature,
    /// The owned PNG decoder rejected the container or its pixel budget.
    InvalidPng(PngError),
}

impl fmt::Display for TuiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "terminal preview I/O: {error}"),
            Self::InvalidLimits => f.write_str("terminal preview limits must be nonzero"),
            Self::ZeroDimensions => f.write_str("terminal preview dimensions must be nonzero"),
            Self::DimensionOverflow => f.write_str("terminal preview dimensions overflow usize"),
            Self::PixelLimit { limit, needed } => {
                write!(
                    f,
                    "terminal frame has {needed} pixels, over the {limit}-pixel limit"
                )
            }
            Self::RgbaLength { expected, found } => {
                write!(
                    f,
                    "terminal RGBA frame has {found} bytes, expected {expected}"
                )
            }
            Self::EncodedSizeOverflow => f.write_str("terminal output size overflow"),
            Self::EncodedLimit { limit, needed } => {
                write!(
                    f,
                    "terminal output needs {needed} bytes, over the {limit}-byte limit"
                )
            }
            Self::PngRequiresKitty => f.write_str("pre-encoded PNG output requires Kitty graphics"),
            Self::InvalidPngSignature => f.write_str("terminal PNG has an invalid signature"),
            Self::InvalidPng(error) => write!(f, "terminal PNG is invalid: {error}"),
        }
    }
}

impl std::error::Error for TuiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::InvalidPng(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for TuiError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<PngError> for TuiError {
    fn from(error: PngError) -> Self {
        Self::InvalidPng(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct BoundedWriteObserver {
        bytes: Vec<u8>,
        max_write: usize,
        largest_write: usize,
    }

    impl Write for BoundedWriteObserver {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.largest_write = self.largest_write.max(bytes.len());
            if bytes.len() > self.max_write {
                return Err(std::io::Error::other("aggregate terminal write"));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn base64_vectors_are_canonical() {
        assert_eq!(base64_block(b"f"), *b"Zg==");
        assert_eq!(base64_block(b"fo"), *b"Zm8=");
        assert_eq!(base64_block(b"foo"), *b"Zm9v");

        let limits = TuiLimits {
            kitty_chunk_bytes: 4,
            ..TuiLimits::default()
        };
        let mut framed = Vec::new();
        write_kitty_png(&mut framed, b"foobar", limits).unwrap();
        assert_eq!(
            framed,
            b"\x1b_Ga=T,f=100,t=d,q=2,m=1;Zm9v\x1b\\\x1b_Gm=0;YmFy\x1b\\"
        );
    }

    #[test]
    fn kitty_streams_png_as_bounded_writes() {
        let png = encode_rgba8(
            2,
            1,
            &[255, 0, 0, 255, 0, 255, 0, 255],
            CompressionLevel::Fast,
        );
        let limits = TuiLimits {
            kitty_chunk_bytes: 8,
            ..TuiLimits::default()
        };
        let preview = TerminalPreview::new(TerminalProtocol::Kitty, limits).unwrap();
        let mut expected = Vec::new();
        preview.write_png(&mut expected, &png).unwrap();

        let mut observed = BoundedWriteObserver {
            max_write: 32,
            ..BoundedWriteObserver::default()
        };
        preview.write_png(&mut observed, &png).unwrap();

        assert_eq!(observed.bytes, expected);
        assert!(observed.largest_write <= observed.max_write);
        assert!(expected.len() > observed.max_write);
    }

    #[test]
    fn kitty_size_preflight_rejects_arithmetic_overflow() {
        let limits = TuiLimits {
            max_encoded_bytes: usize::MAX,
            kitty_chunk_bytes: 4,
            ..TuiLimits::default()
        };
        assert!(matches!(
            kitty_encoded_size(usize::MAX, limits),
            Err(TuiError::EncodedSizeOverflow)
        ));
    }

    #[test]
    fn kitty_and_sixel_have_complete_escape_boundaries() {
        let rgba = [255, 0, 0, 255, 0, 255, 0, 255];
        let mut kitty = Vec::new();
        TerminalPreview::new(TerminalProtocol::Kitty, TuiLimits::default())
            .unwrap()
            .write_rgba8(&mut kitty, 2, 1, &rgba)
            .unwrap();
        assert!(kitty.starts_with(b"\x1b_Ga=T,f=100,t=d,q=2,"));
        assert!(kitty.ends_with(b"\x1b\\"));

        let mut sixel = Vec::new();
        TerminalPreview::new(TerminalProtocol::Sixel, TuiLimits::default())
            .unwrap()
            .write_rgba8(&mut sixel, 2, 1, &rgba)
            .unwrap();
        assert!(sixel.starts_with(b"\x1bP0;0;0q"));
        assert!(sixel.ends_with(b"\x1b\\"));
    }

    #[test]
    fn sixel_palette_discovery_is_band_local_and_byte_stable() {
        let mut rgba = Vec::new();
        for row in 0..12 {
            let pixel = if row < 6 {
                [255, 0, 0, 255]
            } else {
                [0, 255, 0, 255]
            };
            rgba.extend_from_slice(&pixel);
        }

        let first = band_palette(&rgba, 1, 12, 0);
        let second = band_palette(&rgba, 1, 12, 6);
        assert_eq!(first.into_iter().filter(|present| *present).count(), 1);
        assert!(first[180]);
        assert_eq!(second.into_iter().filter(|present| *present).count(), 1);
        assert!(second[30]);

        let mut sixel = Vec::new();
        TerminalPreview::new(TerminalProtocol::Sixel, TuiLimits::default())
            .unwrap()
            .write_rgba8(&mut sixel, 1, 12, &rgba)
            .unwrap();
        assert_eq!(
            sixel,
            b"\x1bP0;0;0q\"1;1;1;12#30;2;0;100;0#180;2;100;0;0#180~-#30~-\x1b\\"
        );
    }

    #[test]
    fn sixel_streams_as_bounded_writes_after_exact_preflight() {
        let rgba = [255, 0, 0, 255].repeat(64);
        let generous = TerminalPreview::new(TerminalProtocol::Sixel, TuiLimits::default()).unwrap();
        let mut expected = Vec::new();
        generous.write_rgba8(&mut expected, 64, 1, &rgba).unwrap();

        let exact = TerminalPreview::new(
            TerminalProtocol::Sixel,
            TuiLimits {
                max_pixels: 64,
                max_encoded_bytes: expected.len(),
                ..TuiLimits::default()
            },
        )
        .unwrap();
        let mut observed = BoundedWriteObserver {
            max_write: 64,
            ..BoundedWriteObserver::default()
        };
        exact.write_rgba8(&mut observed, 64, 1, &rgba).unwrap();
        assert_eq!(observed.bytes, expected);
        assert!(observed.largest_write <= observed.max_write);
        assert!(expected.len() > observed.max_write);

        let one_byte_short = TerminalPreview::new(
            TerminalProtocol::Sixel,
            TuiLimits {
                max_pixels: 64,
                max_encoded_bytes: expected.len() - 1,
                ..TuiLimits::default()
            },
        )
        .unwrap();
        let mut refused = BoundedWriteObserver {
            max_write: 64,
            ..BoundedWriteObserver::default()
        };
        assert!(matches!(
            one_byte_short.write_rgba8(&mut refused, 64, 1, &rgba),
            Err(TuiError::EncodedLimit { limit, needed })
                if limit == expected.len() - 1 && needed == expected.len()
        ));
        assert!(refused.bytes.is_empty());
    }
}
