//! Native texture decode and sampling policy (§10.6, fm-0gy).
//!
//! The policy is intentionally executable rather than prose around a platform
//! sampler:
//!
//! - PNG and JPEG decode only through `fmn-codec`;
//! - row zero and `v = 0` are the **top** row (D-23: no hidden v-flip);
//! - RGB is decoded to linear light before filtering;
//! - bilinear filtering happens on premultiplied linear RGBA, preventing
//!   transparent texel colour from bleeding into a visible edge;
//! - wrap is explicit per axis;
//! - non-native formats return a typed ffmpeg-capability refusal.

use fmn_codec::png::ColorIntent;
use fmn_codec::{JpegError, JpegLimits, PngError, PngLimits, decode_jpeg, decode_png};
use fmn_core::color::LinearRgba;
use fmn_hash::{Digest, Schema, Sha256, sha256};

/// Texture-source identity document.
const TEXTURE_SCHEMA: Schema = Schema::new(*b"FMNT", 1, 1, 0);

/// Hash caller-owned RGBA bytes as the canonical `FMNT` document without
/// materializing a second copy of the pixel payload.
///
/// This is byte-for-byte the framing emitted by [`fmn_hash::Writer`]: fixed
/// header, positional payload, inner document checksum, then the outer content
/// digest. Streaming matters because `Texture::from_rgba8` admits valid images
/// larger than the serializer's general-purpose field limit. Falling back to a
/// raw-pixel hash at that boundary would drop dimensions and transfer encoding
/// from the texture identity.
fn rgba_identity<'a>(
    width: u32,
    height: u32,
    encoding: TextureEncoding,
    rgba_len: usize,
    chunks: impl IntoIterator<Item = &'a [u8]>,
) -> Result<Digest, TextureError> {
    checked_rgba_layout(width, height, rgba_len)?;
    let rgba_len_u64 = u64::try_from(rgba_len).map_err(|_| TextureError::InvalidDimensions)?;
    let (encoding_tag, gamma) = match encoding {
        TextureEncoding::Srgb => (0u32, None),
        TextureEncoding::Linear => (1, None),
        TextureEncoding::Gamma(gamma) => (2, Some(gamma)),
    };
    let metadata_len = 4u64 + 4 + 4 + u64::from(gamma.is_some()) * 4 + 8;
    let payload_len = metadata_len
        .checked_add(rgba_len_u64)
        .ok_or(TextureError::InvalidDimensions)?;

    let mut document = Sha256::new();
    document.update(&TEXTURE_SCHEMA.magic);
    document.update(&TEXTURE_SCHEMA.id.to_le_bytes());
    document.update(&TEXTURE_SCHEMA.major.to_le_bytes());
    document.update(&TEXTURE_SCHEMA.minor.to_le_bytes());
    document.update(&0u16.to_le_bytes()); // flags
    document.update(&0u16.to_le_bytes()); // reserved
    document.update(&payload_len.to_le_bytes());
    document.update(&width.to_le_bytes());
    document.update(&height.to_le_bytes());
    document.update(&encoding_tag.to_le_bytes());
    if let Some(gamma) = gamma {
        document.update(&gamma.to_le_bytes());
    }
    document.update(&rgba_len_u64.to_le_bytes());

    let mut observed = 0usize;
    for chunk in chunks {
        observed = observed
            .checked_add(chunk.len())
            .ok_or(TextureError::InvalidDimensions)?;
        if observed > rgba_len {
            return Err(TextureError::InvalidDimensions);
        }
        document.update(chunk);
    }
    if observed != rgba_len {
        return Err(TextureError::InvalidDimensions);
    }

    let checksum = document.clone().finalize();
    document.update(checksum.as_bytes());
    Ok(document.finalize())
}

/// Where decoded rows and normalized UV coordinates begin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureOrientation {
    /// Row zero is top and `v = 0` samples that row.
    TopLeft,
}

/// FrankenManim has one output-oriented texture convention.
pub const TEXTURE_ORIENTATION: TextureOrientation = TextureOrientation::TopLeft;

/// How a normalized coordinate outside `[0, 1]` addresses texels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureWrap {
    /// Repeat every integer interval, matching the Reference texture default.
    Repeat,
    /// Pin to the outer texel.
    ClampToEdge,
    /// Repeat while reflecting every other interval.
    MirroredRepeat,
}

/// Per-axis texture sampling policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SamplerPolicy {
    /// Horizontal addressing.
    pub wrap_u: TextureWrap,
    /// Vertical addressing.
    pub wrap_v: TextureWrap,
}

impl Default for SamplerPolicy {
    fn default() -> Self {
        Self {
            wrap_u: TextureWrap::Repeat,
            wrap_v: TextureWrap::Repeat,
        }
    }
}

/// Color transfer declared or assumed for input samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureEncoding {
    /// IEC sRGB transfer.
    Srgb,
    /// Already-linear samples.
    Linear,
    /// PNG `gAMA`, stored as gamma × 100000.
    Gamma(u32),
}

/// Native decoder and provenance path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureSource {
    /// PNG with an explicit sRGB chunk and its rendering-intent byte.
    PngSrgb(u8),
    /// PNG with only a `gAMA` chunk.
    PngGamma(u32),
    /// PNG with neither transfer chunk; project policy assumes sRGB.
    PngAssumedSrgb,
    /// JPEG; decoded RGB is assumed sRGB.
    Jpeg {
        /// Whether SOF2 progressive coding was used.
        progressive: bool,
        /// EXIF orientation already applied by fmn-codec.
        orientation: u8,
    },
    /// Caller-provided RGBA bytes.
    Rgba8(TextureEncoding),
}

/// Resource budgets declared before decoding untrusted texture input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureLimits {
    /// Maximum decoded pixels.
    pub max_pixels: u64,
    /// Maximum PNG chunk count.
    pub max_png_chunks: usize,
}

impl Default for TextureLimits {
    fn default() -> Self {
        let png = PngLimits::default();
        Self {
            max_pixels: png.max_pixels,
            max_png_chunks: png.max_chunks,
        }
    }
}

/// Why an image could not become a native texture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextureError {
    /// Native PNG refusal.
    Png(PngError),
    /// Native JPEG refusal.
    Jpeg(JpegError),
    /// Dimensions and RGBA payload length disagree.
    RgbaLength {
        /// Bytes required by the dimensions.
        expected: usize,
        /// Bytes supplied.
        got: usize,
    },
    /// Dimensions are empty or overflow addressable storage.
    InvalidDimensions,
    /// Decoded texel storage could not be reserved.
    AllocationFailed {
        /// Linear-premultiplied texels requested.
        pixels: u64,
    },
    /// The format is outside the native PNG/JPEG set.
    RequiresFfmpeg {
        /// Lowercase extension or `unknown`.
        format: String,
    },
}

impl std::fmt::Display for TextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Png(error) => write!(f, "png texture decode failed: {error}"),
            Self::Jpeg(error) => write!(f, "jpeg texture decode failed: {error}"),
            Self::RgbaLength { expected, got } => {
                write!(
                    f,
                    "rgba texture has {got} bytes, dimensions require {expected}"
                )
            }
            Self::InvalidDimensions => {
                f.write_str("texture dimensions must be nonzero and addressable")
            }
            Self::AllocationFailed { pixels } => {
                write!(f, "could not reserve {pixels} decoded texture texels")
            }
            Self::RequiresFfmpeg { format } => write!(
                f,
                "{format} texture requires the optional ffmpeg image-transcode capability; \
                 convert to PNG/JPEG or enable ffmpeg"
            ),
        }
    }
}

impl std::error::Error for TextureError {}

impl From<PngError> for TextureError {
    fn from(error: PngError) -> Self {
        Self::Png(error)
    }
}

impl From<JpegError> for TextureError {
    fn from(error: JpegError) -> Self {
        Self::Jpeg(error)
    }
}

fn checked_rgba_layout(width: u32, height: u32, rgba_len: usize) -> Result<usize, TextureError> {
    let pixels = u64::from(width)
        .checked_mul(u64::from(height))
        .ok_or(TextureError::InvalidDimensions)?;
    if pixels == 0 {
        return Err(TextureError::InvalidDimensions);
    }
    let expected_u64 = pixels
        .checked_mul(4)
        .ok_or(TextureError::InvalidDimensions)?;
    let expected = usize::try_from(expected_u64).map_err(|_| TextureError::InvalidDimensions)?;
    if rgba_len != expected {
        return Err(TextureError::RgbaLength {
            expected,
            got: rgba_len,
        });
    }
    usize::try_from(pixels).map_err(|_| TextureError::InvalidDimensions)
}

fn allocate_texels(pixels: usize) -> Result<Vec<[f32; 4]>, TextureError> {
    let mut texels = Vec::new();
    texels
        .try_reserve_exact(pixels)
        .map_err(|_| TextureError::AllocationFailed {
            pixels: u64::try_from(pixels).unwrap_or(u64::MAX),
        })?;
    Ok(texels)
}

/// Immutable, output-oriented, premultiplied-linear texture.
#[derive(Debug, Clone, PartialEq)]
pub struct Texture {
    width: u32,
    height: u32,
    /// Premultiplied linear RGBA. Filtering this representation makes alpha
    /// edges associative with the compositor.
    texels: Vec<[f32; 4]>,
    source: TextureSource,
    digest: Digest,
}

impl Texture {
    /// Decode PNG/JPEG by magic. Other formats name the ffmpeg capability.
    pub fn decode(name: &str, bytes: &[u8], limits: TextureLimits) -> Result<Self, TextureError> {
        if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            let decoded = decode_png(
                bytes,
                &PngLimits {
                    max_pixels: limits.max_pixels,
                    max_chunks: limits.max_png_chunks,
                },
            )?;
            let (encoding, source) = match decoded.intent {
                ColorIntent::Srgb { intent } => {
                    (TextureEncoding::Srgb, TextureSource::PngSrgb(intent))
                }
                ColorIntent::Gamma { gamma_100000 } => (
                    TextureEncoding::Gamma(gamma_100000),
                    TextureSource::PngGamma(gamma_100000),
                ),
                ColorIntent::AssumedSrgb => (TextureEncoding::Srgb, TextureSource::PngAssumedSrgb),
            };
            return Self::from_parts(
                decoded.width,
                decoded.height,
                &decoded.rgba,
                encoding,
                source,
                sha256(bytes),
            );
        }
        if bytes.starts_with(&[0xff, 0xd8]) {
            let decoded = decode_jpeg(
                bytes,
                &JpegLimits {
                    max_pixels: limits.max_pixels,
                },
            )?;
            return Self::from_parts(
                decoded.width,
                decoded.height,
                &decoded.rgba,
                TextureEncoding::Srgb,
                TextureSource::Jpeg {
                    progressive: decoded.progressive,
                    orientation: decoded.orientation,
                },
                sha256(bytes),
            );
        }
        Err(TextureError::RequiresFfmpeg {
            format: extension(name),
        })
    }

    /// Build from tight, top-row-first RGBA8.
    pub fn from_rgba8(
        width: u32,
        height: u32,
        rgba: &[u8],
        encoding: TextureEncoding,
    ) -> Result<Self, TextureError> {
        let digest = rgba_identity(width, height, encoding, rgba.len(), std::iter::once(rgba))?;
        Self::from_parts(
            width,
            height,
            rgba,
            encoding,
            TextureSource::Rgba8(encoding),
            digest,
        )
    }

    fn from_parts(
        width: u32,
        height: u32,
        rgba: &[u8],
        encoding: TextureEncoding,
        source: TextureSource,
        digest: Digest,
    ) -> Result<Self, TextureError> {
        let pixels = checked_rgba_layout(width, height, rgba.len())?;
        let mut texels = allocate_texels(pixels)?;
        for pixel in rgba.as_chunks::<4>().0 {
            let alpha = f64::from(pixel[3]) / 255.0;
            let decode = |byte: u8| decode_channel(f64::from(byte) / 255.0, encoding);
            texels.push([
                (decode(pixel[0]) * alpha) as f32,
                (decode(pixel[1]) * alpha) as f32,
                (decode(pixel[2]) * alpha) as f32,
                alpha as f32,
            ]);
        }
        Ok(Self {
            width,
            height,
            texels,
            source,
            digest,
        })
    }

    /// Width in texels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Height in texels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Decoder/color-intent provenance.
    #[must_use]
    pub const fn source(&self) -> TextureSource {
        self.source
    }

    /// Content address of the encoded input plus dimensions/transfer metadata.
    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }

    /// The one orientation convention.
    #[must_use]
    pub const fn orientation(&self) -> TextureOrientation {
        TEXTURE_ORIENTATION
    }

    /// Read one exact texel as straight linear RGBA.
    #[must_use]
    pub fn texel(&self, x: u32, y: u32) -> Option<LinearRgba> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(unpremultiply(self.texels[(y * self.width + x) as usize]))
    }

    /// Bilinear sample at normalized, top-left-origin UV.
    ///
    /// Coordinates name texture edges, like a GPU sampler: the centre of
    /// texel `(x, y)` is `((x + 0.5) / width, (y + 0.5) / height)`.
    #[must_use]
    pub fn sample(&self, uv: [f64; 2], policy: SamplerPolicy) -> LinearRgba {
        if !uv.iter().all(|component| component.is_finite()) {
            return LinearRgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            };
        }
        let x = uv[0] * f64::from(self.width) - 0.5;
        let y = uv[1] * f64::from(self.height) - 0.5;
        let x0 = floor_to_i64(x);
        let y0 = floor_to_i64(y);
        let fx = x - x0 as f64;
        let fy = y - y0 as f64;
        let x1 = x0.saturating_add(1);
        let y1 = y0.saturating_add(1);
        let a = self.wrapped_texel(x0, y0, policy);
        let b = self.wrapped_texel(x1, y0, policy);
        let c = self.wrapped_texel(x0, y1, policy);
        let d = self.wrapped_texel(x1, y1, policy);
        let top = lerp4(a, b, fx);
        let bottom = lerp4(c, d, fx);
        unpremultiply(lerp4(top, bottom, fy))
    }

    fn wrapped_texel(&self, x: i64, y: i64, policy: SamplerPolicy) -> [f32; 4] {
        let x = wrap_index(x, self.width, policy.wrap_u);
        let y = wrap_index(y, self.height, policy.wrap_v);
        self.texels[(y * self.width + x) as usize]
    }
}

fn extension(name: &str) -> String {
    let extension = std::path::Path::new(name)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase);
    extension.unwrap_or_else(|| "unknown".to_owned())
}

fn decode_channel(encoded: f64, encoding: TextureEncoding) -> f64 {
    match encoding {
        TextureEncoding::Srgb => fmn_frame::transfer::srgb_decode(encoded),
        TextureEncoding::Linear => encoded,
        TextureEncoding::Gamma(gamma_100000) => {
            if gamma_100000 == 0 {
                0.0
            } else {
                fmn_dmath::pow(encoded, 100_000.0 / f64::from(gamma_100000))
            }
        }
    }
}

fn floor_to_i64(value: f64) -> i64 {
    let floor = value.floor();
    if floor <= i64::MIN as f64 {
        i64::MIN
    } else if floor >= i64::MAX as f64 {
        i64::MAX
    } else {
        floor as i64
    }
}

fn wrap_index(index: i64, size: u32, wrap: TextureWrap) -> u32 {
    let size = i64::from(size);
    match wrap {
        TextureWrap::ClampToEdge => index.clamp(0, size - 1) as u32,
        TextureWrap::Repeat => index.rem_euclid(size) as u32,
        TextureWrap::MirroredRepeat => {
            let period = size.saturating_mul(2);
            let value = index.rem_euclid(period);
            if value < size {
                value as u32
            } else {
                (period - 1 - value) as u32
            }
        }
    }
}

fn lerp4(a: [f32; 4], b: [f32; 4], alpha: f64) -> [f32; 4] {
    let alpha = alpha as f32;
    let mut result = [0.0f32; 4];
    for index in 0..4 {
        result[index] = a[index] + (b[index] - a[index]) * alpha;
    }
    result
}

fn unpremultiply(value: [f32; 4]) -> LinearRgba {
    let alpha = f64::from(value[3]);
    if alpha <= 0.0 {
        return LinearRgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };
    }
    LinearRgba {
        r: f64::from(value[0]) / alpha,
        g: f64::from(value[1]) / alpha,
        b: f64::from(value[2]) / alpha,
        a: alpha,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_hash::{Limits, Writer};

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() <= 2e-6
    }

    fn policy(wrap: TextureWrap) -> SamplerPolicy {
        SamplerPolicy {
            wrap_u: wrap,
            wrap_v: wrap,
        }
    }

    fn matrix() -> Texture {
        // Top row: red, green. Bottom row: blue, white.
        Texture::from_rgba8(
            2,
            2,
            &[
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
            ],
            TextureEncoding::Linear,
        )
        .expect("valid matrix")
    }

    fn legacy_in_envelope_identity(
        width: u32,
        height: u32,
        rgba: &[u8],
        encoding: TextureEncoding,
    ) -> Digest {
        let mut writer = Writer::new(TEXTURE_SCHEMA);
        writer.put_u32(width);
        writer.put_u32(height);
        match encoding {
            TextureEncoding::Srgb => {
                writer.put_u32(0);
            }
            TextureEncoding::Linear => {
                writer.put_u32(1);
            }
            TextureEncoding::Gamma(gamma) => {
                writer.put_u32(2);
                writer.put_u32(gamma);
            }
        }
        writer.put_bytes(rgba);
        sha256(&writer.finish().expect("small identity document"))
    }

    #[test]
    fn streaming_rgba_identity_preserves_in_envelope_document_digests() {
        let rgba = [0, 1, 2, 3, 4, 5, 6, 7];
        for encoding in [
            TextureEncoding::Srgb,
            TextureEncoding::Linear,
            TextureEncoding::Gamma(45_455),
        ] {
            let texture = Texture::from_rgba8(2, 1, &rgba, encoding).expect("valid texture");
            assert_eq!(
                texture.digest(),
                legacy_in_envelope_identity(2, 1, &rgba, encoding)
            );
        }
    }

    #[test]
    fn over_envelope_rgba_identity_still_binds_transfer_semantics() {
        const CHUNK_BYTES: usize = 1 << 20;
        const CHUNKS: usize = 65;
        const WIDTH: u32 = 4096;
        const HEIGHT: u32 = 4160;
        let block = vec![0xa5; CHUNK_BYTES];
        let rgba_len = CHUNK_BYTES * CHUNKS;
        assert!(rgba_len > Limits::DEFAULT.max_field);
        assert_eq!(
            u64::try_from(rgba_len).expect("fixture length fits"),
            u64::from(WIDTH) * u64::from(HEIGHT) * 4
        );

        let digest = |encoding| {
            rgba_identity(
                WIDTH,
                HEIGHT,
                encoding,
                rgba_len,
                (0..CHUNKS).map(|_| block.as_slice()),
            )
            .expect("streamed identity")
        };
        assert_ne!(
            digest(TextureEncoding::Linear),
            digest(TextureEncoding::Srgb),
            "the old over-limit raw-pixel fallback erased this distinction"
        );
    }

    #[test]
    fn invalid_rgba_layout_is_refused_before_identity_consumes_chunks() {
        let consumed = std::cell::Cell::new(0usize);
        let chunks = std::iter::once_with(|| {
            consumed.set(consumed.get() + 1);
            &[][..]
        });
        assert_eq!(
            rgba_identity(2, 2, TextureEncoding::Srgb, 4, chunks),
            Err(TextureError::RgbaLength {
                expected: 16,
                got: 4,
            })
        );
        assert_eq!(consumed.get(), 0);
    }

    #[test]
    fn texel_capacity_overflow_is_a_typed_allocation_refusal() {
        let pixels = u64::try_from(usize::MAX).unwrap_or(u64::MAX);
        assert_eq!(
            allocate_texels(usize::MAX),
            Err(TextureError::AllocationFailed { pixels })
        );
    }

    #[test]
    fn native_decode_uses_fmn_codec_and_keeps_output_orientation() {
        let png = include_bytes!("../../fmn-codec/tests/fixtures/png/rgba8.png");
        let texture =
            Texture::decode("asset.PNG", png, TextureLimits::default()).expect("native PNG decode");
        assert_eq!(texture.orientation(), TextureOrientation::TopLeft);
        assert!(matches!(
            texture.source(),
            TextureSource::PngAssumedSrgb | TextureSource::PngSrgb(_) | TextureSource::PngGamma(_)
        ));

        let jpeg = include_bytes!("../../fmn-codec/tests/fixtures/jpeg/orient6_le.jpg");
        let oriented = Texture::decode("portrait.jpeg", jpeg, TextureLimits::default())
            .expect("native JPEG decode");
        assert!(matches!(
            oriented.source(),
            TextureSource::Jpeg { orientation: 6, .. }
        ));
        assert!(oriented.width() > 0 && oriented.height() > 0);
    }

    #[test]
    fn exotic_formats_name_the_optional_capability_and_alternative() {
        let error = Texture::decode("photo.webp", b"RIFF....WEBP", TextureLimits::default())
            .expect_err("webp is not native");
        assert_eq!(
            error,
            TextureError::RequiresFfmpeg {
                format: "webp".to_owned()
            }
        );
        let text = error.to_string();
        assert!(text.contains("ffmpeg"));
        assert!(text.contains("PNG/JPEG"));
    }

    #[test]
    fn v_zero_is_the_top_row_and_bilinear_centres_are_exact() {
        let texture = matrix();
        let red = texture.sample([0.25, 0.25], policy(TextureWrap::ClampToEdge));
        let blue = texture.sample([0.25, 0.75], policy(TextureWrap::ClampToEdge));
        assert!(close(red.r, 1.0) && close(red.g, 0.0) && close(red.b, 0.0));
        assert!(close(blue.r, 0.0) && close(blue.g, 0.0) && close(blue.b, 1.0));
    }

    #[test]
    fn wrap_policy_matrix_is_explicit() {
        let texture = matrix();
        let repeated = texture.sample([1.25, 0.25], policy(TextureWrap::Repeat));
        assert!(close(repeated.r, 1.0) && close(repeated.g, 0.0));

        let clamped = texture.sample([1.25, 0.25], policy(TextureWrap::ClampToEdge));
        assert!(close(clamped.r, 0.0) && close(clamped.g, 1.0));

        let mirrored = texture.sample([1.25, 0.25], policy(TextureWrap::MirroredRepeat));
        assert!(close(mirrored.r, 0.0) && close(mirrored.g, 1.0));
    }

    #[test]
    fn filtering_is_linear_light_and_premultiplied_alpha() {
        let texture = Texture::from_rgba8(
            2,
            1,
            &[255, 0, 0, 0, 0, 0, 255, 255],
            TextureEncoding::Linear,
        )
        .expect("valid");
        let sample = texture.sample([0.5, 0.5], policy(TextureWrap::ClampToEdge));
        assert!(close(sample.a, 0.5));
        assert!(close(sample.r, 0.0), "transparent red must not bleed");
        assert!(close(sample.b, 1.0));
    }

    #[test]
    fn gamma_and_srgb_decode_before_filtering() {
        let linear =
            Texture::from_rgba8(1, 1, &[128, 128, 128, 255], TextureEncoding::Gamma(100_000))
                .expect("linear gamma");
        let gamma =
            Texture::from_rgba8(1, 1, &[128, 128, 128, 255], TextureEncoding::Gamma(50_000))
                .expect("gamma half");
        let a = linear.texel(0, 0).expect("texel").r;
        let b = gamma.texel(0, 0).expect("texel").r;
        assert!(close(a, 128.0 / 255.0));
        assert!(close(b, (128.0_f64 / 255.0).powi(2)));
    }

    #[test]
    fn dimensions_are_checked_before_allocation() {
        assert!(matches!(
            Texture::from_rgba8(2, 2, &[0; 4], TextureEncoding::Srgb),
            Err(TextureError::RgbaLength {
                expected: 16,
                got: 4
            })
        ));
        assert_eq!(
            Texture::from_rgba8(0, 1, &[], TextureEncoding::Srgb),
            Err(TextureError::InvalidDimensions)
        );
    }
}
