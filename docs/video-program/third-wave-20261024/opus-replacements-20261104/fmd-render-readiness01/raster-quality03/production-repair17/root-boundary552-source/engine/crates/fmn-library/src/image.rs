//! `ImageMobject` (§12.4, fm-2u6): a raster image as a mobject, decoded by
//! fmn-codec (Appendix A `types/image_mobject.py`).
//!
//! The Reference's surface: a file becomes a mobject whose two triangles
//! carry the image as a texture, sized `height` scene units tall and
//! `height · (pw/ph)` wide, with `point_to_rgb` sampling back into the
//! raster. Here the input is bytes (PNG or JPEG, sniffed by magic number)
//! rather than a path — the library tier has no filesystem — and decoding
//! is fmn-codec's owned, budget-checked path (W8).
//!
//! # Decoding and resampling
//!
//! Decoding **normalizes sample format, never geometry**: any PNG color
//! type/bit depth becomes tightly packed RGBA8, JPEG YCbCr becomes RGBA8
//! with EXIF orientation applied, and row 0 is the image's top row
//! (fmn-codec's D-23). Pixel dimensions are preserved exactly — there is
//! no geometric resampling anywhere in the pipeline; scene-space size
//! comes from `height` and the pixel aspect ratio.
//!
//! The detached record dtype is the Reference's exactly — `(point,
//! im_coords, opacity)`, six points, two triangles — and conversion carries
//! the decoded pixels into Marionette's durable, content-addressed image
//! resource. Lumen interns repeated resources and consumes these same UVs.

use fmn_codec::png::ColorIntent;
use fmn_codec::{JpegError, JpegLimits, PngError, PngLimits, decode_jpeg, decode_png};
use fmn_core::constants::{DL, DR, UL, UR};
use fmn_core::types::Vec3;
use fmn_mobject::record::{RecordBuffer, RecordSchema};
use fmn_mobject::{ImageColorSpace, ImageResource, ImageResourceError, ImageSampler, Mobject};

/// The Reference's default `height` for `ImageMobject` (scene units).
pub const DEFAULT_IMAGE_HEIGHT: f64 = 4.0;

/// Why byte input could not become an [`ImageMobject`].
#[derive(Debug, Clone, PartialEq)]
pub enum ImageError {
    /// Neither a PNG signature nor a JPEG SOI marker led the input.
    UnknownFormat,
    /// Image quads accept one raster, not a light/dark surface material.
    PairedResource,
    /// The PNG decoder refused the input.
    Png(PngError),
    /// The JPEG decoder refused the input.
    Jpeg(JpegError),
    /// An already-decoded raster violated the durable RGBA8 layout.
    Resource(ImageResourceError),
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownFormat => write!(
                f,
                "not a recognized image: input leads with neither the PNG \
                 signature nor a JPEG SOI marker"
            ),
            Self::PairedResource => f.write_str("image quads require an unpaired image resource"),
            Self::Png(e) => write!(f, "PNG decode refused the input: {e}"),
            Self::Jpeg(e) => write!(f, "JPEG decode refused the input: {e}"),
            Self::Resource(e) => write!(f, "decoded image layout is invalid: {e}"),
        }
    }
}

impl std::error::Error for ImageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Png(error) => Some(error),
            Self::Jpeg(error) => Some(error),
            Self::Resource(error) => Some(error),
            Self::UnknownFormat | Self::PairedResource => None,
        }
    }
}

const PNG_MAGIC: &[u8] = b"\x89PNG\r\n\x1a\n";
const JPEG_MAGIC: &[u8] = b"\xff\xd8\xff";

/// `ImageMobject`: a decoded raster plus its scene-space quad
/// (`types/image_mobject.py`).
#[derive(Debug, Clone, PartialEq)]
pub struct ImageMobject {
    resource: ImageResource,
    height: f64,
    opacity: f64,
    z_index: i32,
}

impl ImageMobject {
    /// Decode PNG or JPEG bytes (sniffed by magic number) under the
    /// default decode budgets.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ImageError> {
        Self::from_bytes_with_limits(bytes, &PngLimits::default(), &JpegLimits::default())
    }

    /// Decode PNG or JPEG bytes (sniffed by magic number) under explicit
    /// budgets — the one untrusted-input entry, so every front door refuses
    /// an unrecognized format with the same [`ImageError::UnknownFormat`].
    pub fn from_bytes_with_limits(
        bytes: &[u8],
        png_limits: &PngLimits,
        jpeg_limits: &JpegLimits,
    ) -> Result<Self, ImageError> {
        if bytes.starts_with(PNG_MAGIC) {
            Self::from_png_with_limits(bytes, png_limits)
        } else if bytes.starts_with(JPEG_MAGIC) {
            Self::from_jpeg_with_limits(bytes, jpeg_limits)
        } else {
            Err(ImageError::UnknownFormat)
        }
    }

    /// Decode PNG bytes under the default budgets.
    pub fn from_png(bytes: &[u8]) -> Result<Self, ImageError> {
        Self::from_png_with_limits(bytes, &PngLimits::default())
    }

    /// Decode JPEG bytes under the default budgets.
    pub fn from_jpeg(bytes: &[u8]) -> Result<Self, ImageError> {
        Self::from_jpeg_with_limits(bytes, &JpegLimits::default())
    }

    /// Decode PNG bytes under explicit budgets (the untrusted-input path).
    pub fn from_png_with_limits(bytes: &[u8], limits: &PngLimits) -> Result<Self, ImageError> {
        let decoded = decode_png(bytes, limits).map_err(ImageError::Png)?;
        // The decoder normalizes sample layout, not transfer. Carry its color
        // intent into Marionette so Lumen applies the transfer exactly once,
        // matching direct Texture::decode and preserving checkpoint identity.
        let color_space = match decoded.intent {
            ColorIntent::Gamma { gamma_100000 } => ImageColorSpace::Gamma(gamma_100000),
            ColorIntent::Srgb { .. } | ColorIntent::AssumedSrgb => ImageColorSpace::Srgb,
        };
        let resource = ImageResource::rgba8(
            decoded.width,
            decoded.height,
            decoded.rgba,
            color_space,
            ImageSampler::default(),
        )
        .map_err(ImageError::Resource)?;
        Self::from_resource(resource)
    }

    /// Decode JPEG bytes under explicit budgets (the untrusted-input path).
    pub fn from_jpeg_with_limits(bytes: &[u8], limits: &JpegLimits) -> Result<Self, ImageError> {
        let decoded = decode_jpeg(bytes, limits).map_err(ImageError::Jpeg)?;
        Self::from_rgba8(decoded.width, decoded.height, decoded.rgba)
    }

    /// Wrap an already-decoded RGBA8 raster (row 0 the top row). A buffer
    /// that is not exactly `width × height × 4` bytes is a programming
    /// error in the caller. It is still validated here so invalid state can
    /// never enter a snapshot or a retained table.
    pub fn from_rgba8(width: u32, height: u32, rgba: Vec<u8>) -> Result<Self, ImageError> {
        let resource = ImageResource::rgba8(
            width,
            height,
            rgba,
            ImageColorSpace::Srgb,
            ImageSampler::default(),
        )
        .map_err(ImageError::Resource)?;
        Self::from_resource(resource)
    }

    /// Reuse an immutable, validated raster without copying or decoding pixels.
    /// The resource's transfer and sampler metadata stay authoritative.
    ///
    /// # Errors
    /// A light/dark pair is a surface material, not an image-quad resource.
    pub fn from_resource(resource: ImageResource) -> Result<Self, ImageError> {
        if resource.dark_image().is_some() {
            return Err(ImageError::PairedResource);
        }
        Ok(Self {
            resource,
            height: DEFAULT_IMAGE_HEIGHT,
            opacity: 1.0,
            z_index: 0,
        })
    }

    /// The scene-space height (`height=`; default 4.0).
    #[must_use]
    pub fn with_height(mut self, height: f64) -> Self {
        self.height = height;
        self
    }

    /// The quad's opacity (`opacity=`).
    #[must_use]
    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }

    /// The scene-list sort key (§8.5).
    #[must_use]
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }

    /// Raster width in pixels.
    #[must_use]
    pub fn pixel_width(&self) -> u32 {
        self.resource.width()
    }

    /// Raster height in pixels.
    #[must_use]
    pub fn pixel_height(&self) -> u32 {
        self.resource.height()
    }

    /// The decoded raster: RGBA8, `width × height × 4` bytes, row 0 top.
    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        self.resource.pixels()
    }

    /// The scene-space height.
    #[must_use]
    pub fn height(&self) -> f64 {
        self.height
    }

    /// The scene-space width: `height · (pw / ph)` — the Reference's
    /// `set_width(2·aspect, stretch)` followed by `set_height(height)`.
    /// A zero-height raster (a decoder cannot produce one) would give 0.
    #[must_use]
    pub fn scene_width(&self) -> f64 {
        let aspect = f64::from(self.resource.width()) / f64::from(self.resource.height());
        self.height * aspect
    }

    /// The quad's six corners in scene space, centered on the origin:
    /// `[UL, DL, UR, DR, UR, DL]` scaled to `scene_width × height`
    /// (the Reference's `init_data`/`init_points`).
    #[must_use]
    pub fn quad_points(&self) -> [Vec3; 6] {
        let hx = self.scene_width() / 2.0;
        let hy = self.height / 2.0;
        let scale = |[x, y, z]: Vec3| [x * hx, y * hy, z];
        [
            scale(UL),
            scale(DL),
            scale(UR),
            scale(DR),
            scale(UR),
            scale(DL),
        ]
    }

    /// `point_to_rgb`: sample the raster under a scene-space point.
    /// Returns `None` outside the quad — the Reference raises, and its
    /// guard (`not x_ok and y_ok`) additionally misses two of the four
    /// outside cases; `Option` names the intent.
    #[must_use]
    pub fn point_to_rgb(&self, point: Vec3) -> Option<[f64; 3]> {
        let hx = self.scene_width() / 2.0;
        let hy = self.height / 2.0;
        if hx <= 0.0 || hy <= 0.0 {
            return None;
        }
        let x_alpha = (point[0] + hx) / (2.0 * hx);
        let y_alpha = (point[1] + hy) / (2.0 * hy);
        if !(0.0..=1.0).contains(&x_alpha) || !(0.0..=1.0).contains(&y_alpha) {
            return None;
        }
        // The Reference's `int((pw - 1) * x_alpha)` on its PIL raster, whose
        // row 0 is the top — v = 1 - y_alpha because y grows upward here.
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let px = (f64::from(self.resource.width() - 1) * x_alpha) as usize;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let py = (f64::from(self.resource.height() - 1) * (1.0 - y_alpha)) as usize;
        let offset = (py * self.resource.width() as usize + px) * 4;
        let pixel = self.resource.pixels().get(offset..offset + 3)?;
        Some([
            f64::from(pixel[0]) / 255.0,
            f64::from(pixel[1]) / 255.0,
            f64::from(pixel[2]) / 255.0,
        ])
    }
}

impl From<ImageMobject> for Mobject {
    fn from(image: ImageMobject) -> Self {
        // The Reference's ImageMobject dtype, field for field:
        // [('point', f32, 3), ('im_coords', f32, 2), ('opacity', f32, 1)].
        let schema = RecordSchema::new(
            &[("point", 3), ("im_coords", 2), ("opacity", 1)],
            &["point"],
            &["point"],
        )
        .expect("the image record schema is six lanes");
        let mut buffer = RecordBuffer::new(schema, 6).expect("six image records cannot overflow");
        #[allow(clippy::cast_possible_truncation)]
        let flat_points: Vec<f32> = image
            .quad_points()
            .iter()
            .flat_map(|p| p.iter().map(|v| *v as f32))
            .collect();
        buffer.write_range("point", 0, &flat_points);
        // v = 0 at the top row, matching the decoded raster's orientation.
        let im_coords: Vec<f32> =
            [0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0].to_vec();
        buffer.write_range("im_coords", 0, &im_coords);
        #[allow(clippy::cast_possible_truncation)]
        let opacity: Vec<f32> = vec![image.opacity as f32; 6];
        buffer.write_range("opacity", 0, &opacity);
        Mobject::from_buffer(buffer)
            .with_z_index(image.z_index)
            .with_image_resource(image.resource)
    }
}

// ------------------------------------------------------------------ tests

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_codec::{CompressionLevel, encode_rgba8};
    use fmn_core::constants::ORIGIN;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    /// A 4×3 synthetic raster: pixel (x, y) is (x·40, y·80, x·40+y·10, 255).
    fn synthetic_png() -> (Vec<u8>, Vec<u8>) {
        let mut rgba = Vec::with_capacity(4 * 3 * 4);
        for y in 0..3u8 {
            for x in 0..4u8 {
                rgba.extend_from_slice(&[x * 40, y * 80, x * 40 + y * 10, 255]);
            }
        }
        let png = encode_rgba8(4, 3, &rgba, CompressionLevel::Fast);
        (png, rgba)
    }

    fn intent_png(gamma: Option<u32>, srgb: bool) -> Vec<u8> {
        let encoded = encode_rgba8(1, 1, &[128, 128, 128, 255], CompressionLevel::Fast);
        let mut result = PNG_MAGIC.to_vec();
        let mut cursor = PNG_MAGIC.len();
        while cursor < encoded.len() {
            let size = u32::from_be_bytes(encoded[cursor..cursor + 4].try_into().unwrap()) as usize;
            let kind = &encoded[cursor + 4..cursor + 8];
            if kind == b"gAMA" {
                if let Some(value) = gamma {
                    let mut chunk = kind.to_vec();
                    chunk.extend_from_slice(&value.to_be_bytes());
                    result.extend_from_slice(&4u32.to_be_bytes());
                    result.extend_from_slice(&chunk);
                    result.extend_from_slice(&fmn_codec::checksum::crc32(&chunk).to_be_bytes());
                }
            } else if kind != b"sRGB" || srgb {
                result.extend_from_slice(&encoded[cursor..cursor + size + 12]);
            }
            cursor += size + 12;
        }
        result
    }

    #[test]
    fn png_gamma_metadata_survives_native_resource_construction() {
        for gamma in [100_000, 50_000, 45_455] {
            let image = ImageMobject::from_png(&intent_png(Some(gamma), false)).unwrap();
            assert_eq!(image.pixels(), &[128, 128, 128, 255]);
            let object: Mobject = image.into();
            assert_eq!(
                object.image.as_ref().unwrap().color_space(),
                ImageColorSpace::Gamma(gamma)
            );
        }
    }

    #[test]
    fn png_srgb_precedence_and_untagged_default_stay_srgb() {
        for (gamma, srgb) in [(Some(100_000), true), (None, true), (None, false)] {
            let object: Mobject = ImageMobject::from_png(&intent_png(gamma, srgb))
                .unwrap()
                .into();
            let image = object.image.as_ref().unwrap();
            assert_eq!(image.color_space(), ImageColorSpace::Srgb);
            assert_eq!(image.pixels(), &[128, 128, 128, 255]);
        }
    }

    #[test]
    fn prepared_resource_reuses_pixels_and_refuses_surface_pairs() {
        let resource = ImageResource::rgba8(
            2,
            1,
            vec![255; 8],
            ImageColorSpace::Srgb,
            ImageSampler::default(),
        )
        .unwrap();
        let pointer = resource.pixels().as_ptr();
        let object: Mobject = ImageMobject::from_resource(resource.clone())
            .unwrap()
            .into();
        assert_eq!(object.image.as_ref().unwrap().pixels().as_ptr(), pointer);
        let pair = resource.clone().with_dark_image(resource).unwrap();
        assert_eq!(
            ImageMobject::from_resource(pair),
            Err(ImageError::PairedResource)
        );
    }

    #[test]
    fn synthetic_png_round_trips_pixel_dims() {
        let (png, rgba) = synthetic_png();
        let image = ImageMobject::from_bytes(&png).expect("decodes");
        assert_eq!(image.pixel_width(), 4);
        assert_eq!(image.pixel_height(), 3);
        assert_eq!(
            image.pixels(),
            rgba.as_slice(),
            "decode normalizes to RGBA8"
        );
    }

    #[test]
    fn scene_geometry_uses_the_pixel_aspect() {
        let (png, _) = synthetic_png();
        let image = ImageMobject::from_png(&png).expect("decodes");
        // Default height 4.0; width = 4 * 4/3.
        assert!(close(image.height(), DEFAULT_IMAGE_HEIGHT));
        assert!(close(image.scene_width(), 4.0 * 4.0 / 3.0));
        let points = image.quad_points();
        // UL then DL: the first triangle's left edge spans the full height.
        let hx = (4.0 * 4.0 / 3.0) / 2.0;
        assert!(close(points[0][0], -hx) && close(points[0][1], 2.0));
        assert!(close(points[1][0], -hx) && close(points[1][1], -2.0));
        assert!(close(points[2][0], hx) && close(points[2][1], 2.0));
    }

    #[test]
    fn point_to_rgb_samples_the_raster() {
        let (png, _) = synthetic_png();
        let image = ImageMobject::from_png(&png).expect("decodes");
        // Upper-left corner: pixel (0, 0) = (0, 0, 0).
        let hx = image.scene_width() / 2.0;
        let rgb = image.point_to_rgb([-hx, 2.0, 0.0]).expect("inside");
        assert!(close(rgb[0], 0.0) && close(rgb[1], 0.0));
        // Lower-right corner: pixel (3, 2) = (120, 160, 140).
        let rgb = image.point_to_rgb([hx, -2.0, 0.0]).expect("inside");
        assert!(close(rgb[0], 120.0 / 255.0));
        assert!(close(rgb[1], 160.0 / 255.0));
        // Outside the quad is None, on either axis (the Reference's guard
        // misses half of these).
        assert!(image.point_to_rgb([hx + 0.01, 0.0, 0.0]).is_none());
        assert!(image.point_to_rgb([0.0, 2.01, 0.0]).is_none());
        assert!(image.point_to_rgb([0.0, -2.01, 0.0]).is_none());
        assert!(image.point_to_rgb(ORIGIN).is_some());
    }

    #[test]
    fn unknown_and_malformed_inputs_are_typed_errors() {
        assert_eq!(
            ImageMobject::from_bytes(b"not an image at all"),
            Err(ImageError::UnknownFormat)
        );
        // PNG magic but garbage after: the PNG decoder's named refusal.
        let mut bogus = PNG_MAGIC.to_vec();
        bogus.extend_from_slice(b"\x00\x00\x00garbage");
        assert!(matches!(
            ImageMobject::from_bytes(&bogus),
            Err(ImageError::Png(_))
        ));
        // JPEG magic but truncated: the JPEG decoder's named refusal.
        assert!(matches!(
            ImageMobject::from_bytes(b"\xff\xd8\xff"),
            Err(ImageError::Jpeg(_))
        ));
        // The budgeted entry sniffs the same way and honours the caller's
        // pixel budget: the 4×3 synthetic PNG decodes at 12 pixels and is
        // refused at 11.
        let (png, _) = synthetic_png();
        let jpeg_limits = JpegLimits::default();
        let at = |max_pixels| PngLimits {
            max_pixels,
            ..PngLimits::default()
        };
        assert!(ImageMobject::from_bytes_with_limits(&png, &at(12), &jpeg_limits).is_ok());
        assert!(matches!(
            ImageMobject::from_bytes_with_limits(&png, &at(11), &jpeg_limits),
            Err(ImageError::Png(_))
        ));
        assert_eq!(
            ImageMobject::from_bytes_with_limits(b"class A: pass\n", &at(12), &jpeg_limits),
            Err(ImageError::UnknownFormat)
        );
        assert!(matches!(
            ImageMobject::from_rgba8(2, 2, vec![0; 4]),
            Err(ImageError::Resource(
                ImageResourceError::InvalidByteLength {
                    expected: 16,
                    actual: 4
                }
            ))
        ));
    }

    #[test]
    fn records_carry_the_reference_dtype() {
        let (png, rgba) = synthetic_png();
        let mob = Mobject::from(
            ImageMobject::from_png(&png)
                .expect("decodes")
                .with_opacity(0.5),
        );
        assert_eq!(mob.buffer.len(), 6);
        let names: Vec<&str> = mob
            .buffer
            .schema()
            .fields()
            .iter()
            .map(|f| f.name.as_str())
            .collect();
        assert_eq!(names, ["point", "im_coords", "opacity"]);
        assert_eq!(
            mob.buffer.read(0, "im_coords").expect("field"),
            vec![0.0, 0.0]
        );
        assert_eq!(
            mob.buffer.read(3, "im_coords").expect("field"),
            vec![1.0, 1.0]
        );
        assert_eq!(mob.buffer.read(5, "opacity").expect("field"), vec![0.5]);
        assert_eq!(
            mob.render_primitive,
            fmn_mobject::RenderPrimitive::ImageQuad
        );
        let resource = mob
            .image
            .as_ref()
            .expect("decoded pixels travel with mobject");
        assert_eq!(resource.width(), 4);
        assert_eq!(resource.height(), 3);
        assert_eq!(resource.pixels(), rgba.as_slice());
    }
}
