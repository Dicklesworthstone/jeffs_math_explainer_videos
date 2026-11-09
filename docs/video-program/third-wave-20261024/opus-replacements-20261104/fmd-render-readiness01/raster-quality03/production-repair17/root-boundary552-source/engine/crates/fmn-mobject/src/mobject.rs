//! Detached mobject values: plain data before any stage exists.
//!
//! The §15.1 boundary ratified by G0-1: mobjects are constructed and
//! composed as ordinary values (builders in higher crates convert via
//! `Into<Mobject>`), and `Stage::add` moves them — with their detached
//! children — into the arena.
//!
//! A detached mobject carries everything `Stage::add` needs to reconstruct
//! the entry: the record data, the per-object [`Uniforms`], and the
//! semantic [`ShapeTag`] its constructor built (§10.8). Those last two
//! matter because a builder decides them — `Dot` is a dot, `Circle` sets
//! its own stroke colour — and a value that could not carry them would
//! force every library class to be constructed in two halves, one before
//! `add` and one after.

use fmn_core::types::Vec3;
use fmn_hash::{Digest, sha256};
use std::sync::Arc;

use crate::record::{RecordBuffer, RecordSchema};
use crate::shape::ShapeTag;
use crate::uniforms::Uniforms;

/// The renderer program and constructor metadata an arena entry requires.
///
/// This is distinct from [`ShapeTag`]. A shape tag is an optional vector
/// coverage hint whose geometric payload may become stale after a point
/// write; a render primitive is durable semantic identity. A sampled surface
/// remains a surface after its points are animated, and its UV resolution is
/// required to reconstruct the fixed triangle topology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderPrimitive {
    /// Filled/stroked quadratic-vector records.
    #[default]
    Vector,
    /// A fixed UV-grid surface. The product must equal the record count.
    SurfaceGrid {
        /// Sampled points along the u and v axes.
        resolution: (usize, usize),
    },
    /// Explicit triangle soup: every three records form one triangle.
    TriangleMesh,
    /// Camera-facing dots, one per record.
    DotCloud,
    /// A six-record textured quad backed by an [`ImageResource`].
    ImageQuad,
}

/// The storage layout of one durable image resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImagePixelFormat {
    /// Four tightly packed bytes per texel, in red-green-blue-alpha order.
    Rgba8,
}

/// Transfer function applied before Lumen filters an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageColorSpace {
    /// IEC 61966-2-1 sRGB.
    Srgb,
    /// Already linear-light samples.
    Linear,
    /// PNG-style gamma, stored as gamma multiplied by 100,000.
    Gamma(u32),
}

/// How an image coordinate outside `[0, 1]` addresses texels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageWrap {
    /// Repeat every integer interval.
    Repeat,
    /// Pin to the outer texel.
    ClampToEdge,
    /// Repeat while reflecting every other interval.
    MirroredRepeat,
}

/// Durable per-axis image sampling policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageSampler {
    /// Horizontal addressing.
    pub wrap_u: ImageWrap,
    /// Vertical addressing.
    pub wrap_v: ImageWrap,
}

impl Default for ImageSampler {
    fn default() -> Self {
        Self {
            wrap_u: ImageWrap::Repeat,
            wrap_v: ImageWrap::Repeat,
        }
    }
}

/// Why decoded image bytes could not satisfy the durable resource contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageResourceError {
    /// Width and height must both be non-zero and their RGBA8 product must fit.
    InvalidDimensions,
    /// A dark-side resource cannot itself contain another texture pair.
    NestedTexturePair,
    /// A material has one addressing policy shared by its light/dark images.
    SamplerMismatch,
    /// The byte count was not exactly `width * height * 4`.
    InvalidByteLength {
        /// Exact byte count implied by the dimensions.
        expected: usize,
        /// Byte count supplied by the caller.
        actual: usize,
    },
}

impl std::fmt::Display for ImageResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::InvalidDimensions => {
                f.write_str("image dimensions must be non-zero and fit the RGBA8 layout")
            }
            Self::NestedTexturePair => f.write_str("a dark image cannot contain a texture pair"),
            Self::SamplerMismatch => f.write_str("light and dark images must use the same sampler"),
            Self::InvalidByteLength { expected, actual } => write!(
                f,
                "RGBA8 image needs exactly {expected} bytes, received {actual}"
            ),
        }
    }
}

impl std::error::Error for ImageResourceError {}

/// Immutable, content-addressed raster bytes carried by Marionette state.
///
/// Pixel storage is shared by copies and snapshots. Mutating an image means
/// replacing this value through `Stage`, which advances the image revision
/// axis without exposing a writable byte channel that could bypass the
/// retained renderer's revision discipline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageResource {
    width: u32,
    height: u32,
    pixels: Arc<[u8]>,
    color_space: ImageColorSpace,
    sampler: ImageSampler,
    content_digest: Digest,
    // A single-level pair keeps both immutable resources in the same Stage
    // revision/snapshot axis. The primary pixel digest remains a pixel address;
    // the canonical snapshot commits to both complete descriptors and payloads.
    dark_image: Option<Arc<ImageResource>>,
}

impl ImageResource {
    /// Validate and take ownership of tightly packed, top-row-first RGBA8.
    pub fn rgba8(
        width: u32,
        height: u32,
        pixels: Vec<u8>,
        color_space: ImageColorSpace,
        sampler: ImageSampler,
    ) -> Result<Self, ImageResourceError> {
        let expected = u64::from(width)
            .checked_mul(u64::from(height))
            .and_then(|count| count.checked_mul(4))
            .and_then(|count| usize::try_from(count).ok())
            .filter(|_| width != 0 && height != 0)
            .ok_or(ImageResourceError::InvalidDimensions)?;
        if pixels.len() != expected {
            return Err(ImageResourceError::InvalidByteLength {
                expected,
                actual: pixels.len(),
            });
        }
        let content_digest = sha256(&pixels);
        Ok(Self {
            width,
            height,
            pixels: Arc::from(pixels),
            color_space,
            sampler,
            content_digest,
            dark_image: None,
        })
    }

    /// Attach or replace the dark-side texture for a sampled surface or mesh.
    ///
    /// Dimensions and transfer encodings may differ. Each image is decoded and
    /// filtered independently by Lumen, then the existing light-facing shader
    /// crossfades the samples. Copies and snapshots share both pixel buffers.
    ///
    /// # Errors
    /// Rejects nested pairs and conflicting sampler policies rather than
    /// silently discarding a third image or an authored addressing policy.
    pub fn with_dark_image(mut self, dark: Self) -> Result<Self, ImageResourceError> {
        if dark.dark_image.is_some() {
            return Err(ImageResourceError::NestedTexturePair);
        }
        if self.sampler != dark.sampler {
            return Err(ImageResourceError::SamplerMismatch);
        }
        self.dark_image = Some(Arc::new(dark));
        Ok(self)
    }

    /// Optional dark-side image. This resource never contains another pair.
    #[must_use]
    pub fn dark_image(&self) -> Option<&Self> {
        self.dark_image.as_deref()
    }

    /// Return the primary image alone, preserving its original pixel storage.
    #[must_use]
    pub fn without_dark_image(mut self) -> Self {
        self.dark_image = None;
        self
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

    /// Fixed storage format.
    #[must_use]
    pub const fn pixel_format(&self) -> ImagePixelFormat {
        ImagePixelFormat::Rgba8
    }

    /// Transfer function applied before filtering.
    #[must_use]
    pub const fn color_space(&self) -> ImageColorSpace {
        self.color_space
    }

    /// Addressing policy on both axes.
    #[must_use]
    pub const fn sampler(&self) -> ImageSampler {
        self.sampler
    }

    /// Raw RGBA8 bytes, row zero first.
    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// SHA-256 of the raw pixel bytes, suitable for `fmn-cache` addressing.
    #[must_use]
    pub const fn content_digest(&self) -> Digest {
        self.content_digest
    }
}

/// A detached mobject: record data, per-object uniforms, the semantic
/// shape tag, and detached children.
pub struct Mobject {
    /// The per-object record data.
    pub buffer: RecordBuffer,
    /// The per-object uniform inventory (§8.4) this mobject will enter the
    /// arena with.
    pub uniforms: Uniforms,
    /// The semantic shape (§10.8) the constructor built, stamped against
    /// these points when the mobject enters the arena.
    pub shape: ShapeTag,
    /// Durable renderer identity and topology metadata.
    pub render_primitive: RenderPrimitive,
    /// Optional immutable raster resource for [`RenderPrimitive::ImageQuad`].
    pub image: Option<ImageResource>,
    /// The Reference's `z_index` (§8.5): the scene list's sort key. Zero is
    /// the Reference's default; a builder that means to sit above or below
    /// its siblings sets it here rather than after `add`.
    pub z_index: i32,
    /// Children still outside any arena; `Stage::add` recurses over these.
    pub submobjects: Vec<Mobject>,
    /// The Reference's `_data_defaults` row for a point-free mobject
    /// (§6.1): the constructor style a later resize seeds from, flattened
    /// as `(lane name, first-row values)` pairs. `None` unless the source
    /// builder carried an explicit style onto zero points.
    pub defaults_row: Option<Vec<(String, Vec<f32>)>>,
}

impl Default for Mobject {
    fn default() -> Self {
        Self::new()
    }
}

impl Mobject {
    /// An empty mobject (no records, no children).
    #[must_use]
    pub fn new() -> Self {
        Self::from_buffer(
            RecordBuffer::new(RecordSchema::mobject(), 0)
                .expect("an empty mobject schema buffer cannot overflow"),
        )
    }

    /// A mobject over an already-built record buffer, with default
    /// uniforms, no shape, and no children.
    #[must_use]
    pub fn from_buffer(buffer: RecordBuffer) -> Self {
        Self {
            buffer,
            uniforms: Uniforms::default(),
            submobjects: Vec::new(),
            defaults_row: None,
            shape: ShapeTag::General,
            render_primitive: RenderPrimitive::Vector,
            image: None,
            z_index: 0,
        }
    }

    /// A mobject whose `point` records are the given points (semantic f64
    /// in, record f32 stored, per §6.1). Its schema is the Reference's base
    /// `Mobject` dtype, so it carries no paint and never draws
    /// ([`Entry::paints`](crate::stage::Entry::paints)).
    #[must_use]
    pub fn from_points(points: &[Vec3]) -> Self {
        Self::points_over(RecordSchema::mobject(), points)
    }

    /// A vector mobject over the given points: the Reference's `VMobject`
    /// dtype, paint lanes zero (transparent, width 0) until styled.
    #[must_use]
    pub fn vector_from_points(points: &[Vec3]) -> Self {
        Self::points_over(RecordSchema::vmobject(), points)
    }

    fn points_over(schema: RecordSchema, points: &[Vec3]) -> Self {
        // A slice of 24-byte Vec3s holds at most isize::MAX/24 records, so
        // seventeen f32 lanes per record always fit one allocation.
        let mut buffer = RecordBuffer::new(schema, points.len())
            .expect("record sizing bounded by the input slice");
        for (i, p) in points.iter().enumerate() {
            buffer.write(i, "point", &[p[0] as f32, p[1] as f32, p[2] as f32]);
        }
        Self::from_buffer(buffer)
    }

    /// Group composition while detached.
    #[must_use]
    pub fn group(children: Vec<Mobject>) -> Self {
        let mut out = Self::new();
        out.submobjects = children;
        out
    }

    /// Attach the semantic shape tag (builder style).
    #[must_use]
    pub fn with_shape(mut self, shape: ShapeTag) -> Self {
        self.shape = shape;
        self
    }

    /// Attach durable renderer identity/topology metadata.
    #[must_use]
    pub fn with_render_primitive(mut self, render_primitive: RenderPrimitive) -> Self {
        self.render_primitive = render_primitive;
        self
    }

    /// Attach immutable image bytes and select the textured-quad renderer.
    #[must_use]
    pub fn with_image_resource(mut self, image: ImageResource) -> Self {
        self.image = Some(image);
        self.render_primitive = RenderPrimitive::ImageQuad;
        self
    }

    /// Attach the uniform inventory (builder style).
    #[must_use]
    pub fn with_uniforms(mut self, uniforms: Uniforms) -> Self {
        self.uniforms = uniforms;
        self
    }

    /// Set the scene-list sort key (builder style, §8.5).
    #[must_use]
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }
}
