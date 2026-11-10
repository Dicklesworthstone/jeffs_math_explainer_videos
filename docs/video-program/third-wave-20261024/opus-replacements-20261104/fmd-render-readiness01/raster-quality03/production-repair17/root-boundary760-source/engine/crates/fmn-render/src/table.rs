//! The compiled render IR's tables (§10.8), and the interning that makes them
//! smaller than the scene.
//!
//! > Between Marionette's authoritative state and the engines sits a compiled,
//! > backend-neutral render IR — `PathTable`, `SegmentTable`, `StyleTable`,
//! > `InstanceTable`, `ImageTable`, and per-tile command lists — synchronized
//! > *lazily* from the RecordBuffer under §8.2's mirror rule, with
//! > backend-specific layouts derived from it.
//!
//! ## Backend-neutral means what it says
//!
//! Everything here is host structs in the semantic width (`f64`, §6.1). The flat
//! single-typed arrays a GPU wants are a **derived layout**, one derivation per
//! backend, not this representation reshaped — G0-8's finding F1, confirmed
//! against a real Metal pipeline before this table existed. Two consequences the
//! spike measured and this module honours:
//!
//! - **F1**: a device buffer binding is a typed pointer, so a derived layout is
//!   one flat array per table, each array a single scalar type.
//! - **F12**: fills and strokes read *different* style fields, so the device
//!   style row is per-stage. The interned table and its dedup index are shared;
//!   the layout is not.
//!
//! ## Interning is a semantics, not an optimization
//!
//! [`StyleTable`] dedups **by exact bits**, matching
//! `fmn_mobject::order::BatchKey`. §8.5 makes batching semantics — how many draw
//! calls a scene takes is observable — so a table that merged `0.1 + 0.2` with
//! `0.3` would quietly change a number the Parity Ledger has to explain.
//!
//! [`ShapeTable`] is the same mechanism applied to geometry, and it is where
//! §10.8's "glyphs and repeated shapes are interned" lives: an outline compiles
//! once, content-addressed, and each occurrence becomes an [`Instance`] carrying
//! a transform, a style and an order. For text-heavy mathematical scenes that is
//! where the duplication is — the same `x` glyph in a formula is one compiled
//! path and N instances — and the same mechanism serves dots, arrow tips,
//! repeated graph nodes and copied decorations.

use crate::hint::Hint;
use crate::texture::{SamplerPolicy, Texture, TextureEncoding, TextureWrap};
use fmn_cache::CacheKey;
use fmn_core::types::{Vec3, canonicalize_f64};
use fmn_geom::arclength::{ArcLengthTable, quadratic_arc_length};
use fmn_geom::quadpath::QuadPath;
use fmn_hash::{Digest, Schema, Sha256};
use fmn_mobject::{
    BoundingBox, ImageColorSpace, ImagePixelFormat, ImageResource, ImageSampler, ImageWrap,
    JointType, Mob, Placement, Uniforms,
};
use std::collections::HashMap;
use std::sync::Arc;

/// A retained IR table could not preserve its `u32` index contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableError {
    /// A shape builder returned a row under a different content identity.
    ShapeIdentityMismatch {
        /// Identity requested from the deduplication index.
        requested: Digest,
        /// Identity carried by the built shape row.
        built: Digest,
    },
    /// Adding a row would make the table wider than its `u32` indices encode.
    IndexCapacityExceeded {
        /// The table or range being indexed.
        resource: &'static str,
        /// Rows or the exclusive segment end requested.
        requested: u64,
    },
    /// A preflighted retained table could not reserve its pending rows.
    AllocationFailed {
        /// The table being reserved.
        resource: &'static str,
        /// Total rows requested after the reservation.
        requested: u64,
    },
    /// A validated Marionette image could not become a Lumen texture.
    ImageTextureInvalid,
}

impl std::fmt::Display for TableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ShapeIdentityMismatch { requested, built } => write!(
                f,
                "retained shape builder returned identity {built} for requested identity {requested}"
            ),
            Self::IndexCapacityExceeded {
                resource,
                requested,
            } => write!(
                f,
                "retained {resource} needs index capacity {requested}, exceeding u32"
            ),
            Self::AllocationFailed {
                resource,
                requested,
            } => write!(
                f,
                "could not reserve {requested} rows for retained {resource}"
            ),
            Self::ImageTextureInvalid => {
                f.write_str("validated image resource could not become a Lumen texture")
            }
        }
    }
}

impl std::error::Error for TableError {}

fn rows_after_push(resource: &'static str, len: usize) -> Result<(u32, u64), TableError> {
    let requested = u64::try_from(len)
        .unwrap_or(u64::MAX)
        .checked_add(1)
        .ok_or(TableError::IndexCapacityExceeded {
            resource,
            requested: u64::MAX,
        })?;
    if requested > u64::from(u32::MAX) {
        return Err(TableError::IndexCapacityExceeded {
            resource,
            requested,
        });
    }
    let index = u32::try_from(len).map_err(|_| TableError::IndexCapacityExceeded {
        resource,
        requested,
    })?;
    Ok((index, requested))
}

pub(crate) fn check_row_count(resource: &'static str, count: usize) -> Result<(), TableError> {
    let requested = u64::try_from(count).unwrap_or(u64::MAX);
    if requested > u64::from(u32::MAX) {
        return Err(TableError::IndexCapacityExceeded {
            resource,
            requested,
        });
    }
    Ok(())
}

fn allocation_failed(resource: &'static str, current: usize, additional: usize) -> TableError {
    TableError::AllocationFailed {
        resource,
        requested: current
            .checked_add(additional)
            .and_then(|value| u64::try_from(value).ok())
            .unwrap_or(u64::MAX),
    }
}

/// One quadratic segment of a compiled path, in **object space**.
///
/// §10.8 says compiled paths retain "screen-independent coefficients", and this
/// is why: a camera move must not rebuild curve geometry. The screen-space `f32`
/// form a kernel reads is a derived layout keyed on
/// `(Geometry, Transform, CameraProjection)`; this is keyed on `Geometry` alone.
///
/// The normalized arc-length span `(s0, s1)` rides along because §10.3
/// interpolates stroke width and colour by arc length, and §7.3's arc-length
/// layer is transcendental-dense and branchy — the wrong shape for a per-pixel
/// kernel and the right shape for the host (G0-8 finding F3). The span is a
/// function of object-space geometry alone, so it survives a restyle,
/// translation, and a structurally proven similarity. A general affine
/// placement derives transformed segments once per frame and reparameterizes
/// their spans: non-uniform scale and shear change relative arc lengths even
/// though the retained coefficients remain reusable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    /// Start anchor.
    pub p0: Vec3,
    /// Control handle.
    pub p1: Vec3,
    /// End anchor.
    pub p2: Vec3,
    /// Normalized arc length at `t = 0`, over the whole path.
    pub s0: f64,
    /// Normalized arc length at `t = 1`.
    pub s1: f64,
}

/// Recompute normalized path arc-length spans after transforming coefficients.
///
/// The retained [`Segment`] table is object-space geometry. Rotation and
/// uniform scale preserve its normalized spans mathematically, but a general
/// affine placement does not: stretch and shear change the relative lengths of
/// differently oriented curves. Frame-local transformed segments therefore
/// pass through this once before stroke ramps or gradient stations consume
/// `s0/s1`.
pub(crate) fn reparameterize_arc_length(segments: &mut [Segment]) {
    let mut total = 0.0;
    for segment in &mut *segments {
        let length = quadratic_arc_length(segment.p0, segment.p1, segment.p2);
        segment.s0 = 0.0;
        segment.s1 = length;
        total += length;
    }
    if total <= 0.0 || !total.is_finite() {
        for segment in segments {
            segment.s0 = 0.0;
            segment.s1 = 0.0;
        }
        return;
    }
    let mut accumulated = 0.0;
    for segment in segments {
        let length = segment.s1;
        segment.s0 = accumulated / total;
        accumulated += length;
        segment.s1 = accumulated / total;
    }
}

/// Whether a placement is an exactly recognizable similarity whose normalized
/// arc-length spans can keep borrowing the object-space values.
///
/// This is deliberately narrower than a numerical `AᵀA = s²I` test. A
/// floating-point tolerance could admit a slightly anisotropic stretch, while
/// exact Gram products could still misclassify after rounding. We therefore
/// accept only a finite non-zero signed permutation with one equal-magnitude
/// coefficient in every row and column. That covers uniform scale, reflection,
/// and exact axis permutations; dense rotations conservatively reparameterize
/// until placement provenance can prove their construction.
pub(crate) fn retains_normalized_arc_length(placement: Placement) -> bool {
    let mut used_columns = [false; 3];
    let mut scale_bits = None;

    for row in placement.linear() {
        let mut occupied_column = None;
        for (column, coefficient) in row.into_iter().enumerate() {
            if coefficient == 0.0 {
                continue;
            }
            if !coefficient.is_finite() || occupied_column.is_some() || used_columns[column] {
                return false;
            }
            let magnitude = coefficient.abs();
            match scale_bits {
                None => scale_bits = Some(magnitude.to_bits()),
                Some(expected) if expected == magnitude.to_bits() => {}
                Some(_) => return false,
            }
            occupied_column = Some(column);
        }
        let Some(column) = occupied_column else {
            return false;
        };
        used_columns[column] = true;
    }

    scale_bits.is_some()
}

/// The style a compiled path draws under.
///
/// **Retained stroke profiles and per-path constants.** The Reference stores `stroke_rgba`,
/// `stroke_width`, `fill_rgba` and `fill_border_width` *per point*, which is how
/// a gradient along a path is expressed, and this row retains an immutable true-arc station table when those columns
/// vary. Manually constructed styles can still use the legacy two-end ramp. The analytic fill
/// field and true-distance stroke kernels consume these endpoints with their
/// shared arc-length parameterization. This table owns the *interning*, dedup
/// index, and per-stage derivation; camera projection remains a derived
/// [`crate::three_d::ThreeDJob`] concern and never rewrites this object-space
/// row.
#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    /// Stroke colour at arc-length 0, linear light, straight alpha.
    pub stroke_rgba: [f32; 4],
    /// Stroke colour at arc-length 1.
    pub stroke_rgba_end: [f32; 4],
    /// Stroke width at arc-length 0, object units.
    pub stroke_width: f32,
    /// Stroke width at arc-length 1.
    pub stroke_width_end: f32,
    /// Optional immutable per-record stroke paint; overrides the endpoint ramp.
    pub stroke_profile: Option<std::sync::Arc<crate::StrokeProfile>>,
    /// Fill colour at the ramp's start.
    pub fill_rgba: [f32; 4],
    /// Fill colour at the ramp's end.
    pub fill_rgba_end: [f32; 4],
    /// Optional immutable per-record boundary fill; overrides the endpoint ramp.
    pub fill_profile: Option<std::sync::Arc<crate::FillProfile>>,
    /// §10.2's principled inner border stroke.
    pub fill_border_width: f32,
    /// The AA band, from the object's uniforms.
    pub anti_alias_width: f32,
    /// Stroke joint style.
    pub joint_type: JointType,
    /// Float mix between the world camera and fixed-frame projection.
    pub is_fixed_in_frame: f64,
    /// `(reflectiveness, gloss, shadow)` for vector-compatible shading.
    pub shading: Vec3,
    /// Four world-space user clip planes.
    pub clip_planes: [[f64; 4]; 4],
    /// Construct the stroke in the path's world plane rather than billboarded
    /// toward the camera.
    pub flat_stroke: bool,
    /// Keep stroke width in world units while the camera frame zooms.
    pub scale_stroke_with_zoom: bool,
    /// Whether the stroke draws behind the fill (§8.5 batch key).
    pub stroke_behind: bool,
    /// Opt in to the painter-sequence's per-fragment depth test/write.
    pub depth_test: bool,
}

impl Style {
    /// The bit pattern this style interns under.
    ///
    /// Bitwise, matching `BatchKey`. The array width is asserted by
    /// construction: adding a field without widening it fails every test here
    /// rather than silently merging two distinct styles.
    pub(crate) fn bits(&self) -> Vec<u64> {
        let mut b = vec![0u64; 45];
        let scalars = [
            self.stroke_width,
            self.stroke_width_end,
            self.fill_border_width,
            self.anti_alias_width,
        ];
        for (i, v) in self
            .stroke_rgba
            .iter()
            .chain(self.stroke_rgba_end.iter())
            .chain(self.fill_rgba.iter())
            .chain(self.fill_rgba_end.iter())
            .chain(scalars.iter())
            .enumerate()
        {
            b[i] = u64::from(v.to_bits());
        }
        b[20] = self.joint_type.to_code().to_bits();
        b[21] = u64::from(self.stroke_behind);
        b[22] = self.is_fixed_in_frame.to_bits();
        for (slot, value) in b[23..26].iter_mut().zip(self.shading) {
            *slot = value.to_bits();
        }
        for (slot, value) in b[26..42].iter_mut().zip(self.clip_planes.iter().flatten()) {
            *slot = value.to_bits();
        }
        b[42] = u64::from(self.flat_stroke);
        b[43] = u64::from(self.scale_stroke_with_zoom);
        b[44] = u64::from(self.depth_test);
        if let Some(profile) = &self.stroke_profile {
            b.push(profile.knots().len() as u64);
            for knot in profile.knots() {
                b.push(knot.s.to_bits());
                b.push(u64::from(knot.width.to_bits()));
                b.extend(knot.rgba.map(|value| u64::from(value.to_bits())));
            }
        }
        if let Some(profile) = &self.fill_profile {
            b.push(u64::MAX);
            b.push(profile.knots().len() as u64);
            for knot in profile.knots() {
                b.push(knot.s.to_bits());
                b.extend(knot.rgba.map(|value| u64::from(value.to_bits())));
            }
        }
        b
    }

    /// The per-object constants a `Uniforms` contributes.
    #[must_use]
    pub fn with_uniforms(mut self, u: &Uniforms) -> Style {
        self.anti_alias_width = u.anti_alias_width as f32;
        self.joint_type = u.joint_type;
        self.is_fixed_in_frame = u.is_fixed_in_frame;
        self.shading = u.shading;
        self.clip_planes = u.clip_planes;
        self.flat_stroke = u.flat_stroke;
        self.scale_stroke_with_zoom = u.scale_stroke_with_zoom;
        self.stroke_behind = u.stroke_behind;
        self.depth_test = u.depth_test;
        self
    }
}

impl Default for Style {
    fn default() -> Self {
        let u = Uniforms::default();
        Style {
            stroke_rgba: [0.0; 4],
            stroke_rgba_end: [0.0; 4],
            stroke_width: 0.0,
            stroke_width_end: 0.0,
            stroke_profile: None,
            fill_rgba: [0.0; 4],
            fill_rgba_end: [0.0; 4],
            fill_profile: None,
            fill_border_width: 0.0,
            anti_alias_width: u.anti_alias_width as f32,
            joint_type: u.joint_type,
            is_fixed_in_frame: u.is_fixed_in_frame,
            shading: u.shading,
            clip_planes: u.clip_planes,
            flat_stroke: u.flat_stroke,
            scale_stroke_with_zoom: u.scale_stroke_with_zoom,
            stroke_behind: u.stroke_behind,
            depth_test: u.depth_test,
        }
    }
}

/// Deduplicated styles, and the index that made them unique.
#[derive(Debug, Clone, Default)]
pub struct StyleTable {
    rows: Vec<Style>,
    index: HashMap<Vec<u64>, u32>,
    profile_knots: usize,
}

impl StyleTable {
    /// Intern `style`, returning its row.
    ///
    /// # Errors
    /// Returns [`TableError::IndexCapacityExceeded`] before mutation when one
    /// more row would exceed the table's `u32` index width, or
    /// [`TableError::AllocationFailed`] when its row or dedup index cannot
    /// reserve that row.
    pub fn intern(&mut self, style: Style) -> Result<u32, TableError> {
        let key = style.bits();
        if let Some(&i) = self.index.get(&key) {
            return Ok(i);
        }
        let (i, _) = rows_after_push("style rows", self.rows.len())?;
        self.rows
            .try_reserve(1)
            .map_err(|_| allocation_failed("style rows", self.rows.len(), 1))?;
        self.index
            .try_reserve(1)
            .map_err(|_| allocation_failed("style index", self.index.len(), 1))?;
        let knots = style.profile_knots();
        self.profile_knots =
            self.profile_knots
                .checked_add(knots)
                .ok_or(TableError::IndexCapacityExceeded {
                    resource: "profile knots",
                    requested: u64::MAX,
                })?;
        self.rows.push(style);
        self.index.insert(key, i);
        Ok(i)
    }

    pub(crate) fn index_of(&self, style: &Style) -> Option<u32> {
        self.index.get(&style.bits()).copied()
    }

    pub(crate) fn reserve_additional(&mut self, additional: usize) -> Result<(), TableError> {
        let total =
            self.rows
                .len()
                .checked_add(additional)
                .ok_or(TableError::IndexCapacityExceeded {
                    resource: "style rows",
                    requested: u64::MAX,
                })?;
        check_row_count("style rows", total)?;
        self.rows
            .try_reserve_exact(additional)
            .map_err(|_| allocation_failed("style rows", self.rows.len(), additional))?;
        self.index
            .try_reserve(additional)
            .map_err(|_| allocation_failed("style index", self.index.len(), additional))?;
        Ok(())
    }

    pub(crate) fn insert_prepared(&mut self, index: u32, style: Style) {
        debug_assert_eq!(usize::try_from(index).ok(), Some(self.rows.len()));
        let key = style.bits();
        debug_assert!(!self.index.contains_key(&key));
        self.profile_knots = self
            .profile_knots
            .checked_add(style.profile_knots())
            .expect("profile knot admission precedes publication");
        self.rows.push(style);
        self.index.insert(key, index);
    }

    /// Total retained profile knots, counted per interned style row.
    #[must_use]
    pub fn profile_knots(&self) -> usize {
        self.profile_knots
    }

    /// The interned rows, in insertion order.
    #[must_use]
    pub fn rows(&self) -> &[Style] {
        &self.rows
    }

    /// One row.
    #[must_use]
    pub fn get(&self, index: u32) -> Option<&Style> {
        self.rows.get(usize::try_from(index).ok()?)
    }

    /// How many distinct styles the scene uses.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Is the table empty?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Forget every row.
    pub fn clear(&mut self) {
        self.rows.clear();
        self.index.clear();
        self.profile_knots = 0;
    }
}

/// Complete semantic identity of one retained image row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageDescriptor {
    /// Width in texels.
    pub width: u32,
    /// Height in texels.
    pub height: u32,
    /// Byte layout.
    pub format: ImagePixelFormat,
    /// Transfer function applied before filtering.
    pub color_space: ImageColorSpace,
    /// Per-axis addressing policy.
    pub sampler: ImageSampler,
    /// Raw-pixel content address, represented by the governed cache key type.
    pub content: CacheKey,
}

/// One decoded, linearized row in the retained `ImageTable`.
#[derive(Debug, Clone)]
pub struct Image {
    descriptor: ImageDescriptor,
    texture: Arc<Texture>,
    sampler: SamplerPolicy,
}

impl Image {
    /// Semantic descriptor used for exact interning.
    #[must_use]
    pub const fn descriptor(&self) -> ImageDescriptor {
        self.descriptor
    }

    /// Premultiplied-linear texture consumed by the rasterizer.
    #[must_use]
    pub fn texture(&self) -> &Texture {
        self.texture.as_ref()
    }

    /// Lumen's executable form of the durable sampler policy.
    #[must_use]
    pub const fn sampler(&self) -> SamplerPolicy {
        self.sampler
    }
}

/// Exact-descriptor interning for immutable image resources.
#[derive(Debug, Clone, Default)]
pub struct ImageTable {
    rows: Vec<Image>,
    index: HashMap<ImageDescriptor, u32>,
    textures: HashMap<ImageDescriptor, Arc<Texture>>,
}

impl ImageTable {
    /// Intern a Marionette resource, decoding transfer exactly once per unique
    /// descriptor and raw-pixel content address.
    ///
    /// # Errors
    /// Returns a typed table refusal without publishing a partial row.
    pub fn intern(&mut self, resource: &ImageResource) -> Result<u32, TableError> {
        let descriptor = ImageDescriptor {
            width: resource.width(),
            height: resource.height(),
            format: resource.pixel_format(),
            color_space: resource.color_space(),
            sampler: resource.sampler(),
            content: CacheKey::from_digest(resource.content_digest()),
        };
        if let Some(&index) = self.index.get(&descriptor) {
            return Ok(index);
        }
        let (index, _) = rows_after_push("image rows", self.rows.len())?;
        self.rows
            .try_reserve(1)
            .map_err(|_| allocation_failed("image rows", self.rows.len(), 1))?;
        self.index
            .try_reserve(1)
            .map_err(|_| allocation_failed("image index", self.index.len(), 1))?;

        let encoding = match resource.color_space() {
            ImageColorSpace::Srgb => TextureEncoding::Srgb,
            ImageColorSpace::Linear => TextureEncoding::Linear,
            ImageColorSpace::Gamma(gamma) => TextureEncoding::Gamma(gamma),
        };
        let texture = if let Some(texture) = self.textures.get(&descriptor) {
            Arc::clone(texture)
        } else {
            self.textures
                .try_reserve(1)
                .map_err(|_| allocation_failed("image texture index", self.textures.len(), 1))?;
            let texture = Arc::new(
                Texture::from_rgba8(
                    resource.width(),
                    resource.height(),
                    resource.pixels(),
                    encoding,
                )
                .map_err(|error| match error {
                    crate::texture::TextureError::AllocationFailed { pixels } => {
                        TableError::AllocationFailed {
                            resource: "image texels",
                            requested: pixels,
                        }
                    }
                    _ => TableError::ImageTextureInvalid,
                })?,
            );
            self.textures.insert(descriptor, Arc::clone(&texture));
            texture
        };
        let convert_wrap = |wrap| match wrap {
            ImageWrap::Repeat => TextureWrap::Repeat,
            ImageWrap::ClampToEdge => TextureWrap::ClampToEdge,
            ImageWrap::MirroredRepeat => TextureWrap::MirroredRepeat,
        };
        let sampler = SamplerPolicy {
            wrap_u: convert_wrap(resource.sampler().wrap_u),
            wrap_v: convert_wrap(resource.sampler().wrap_v),
        };
        self.rows.push(Image {
            descriptor,
            texture,
            sampler,
        });
        self.index.insert(descriptor, index);
        Ok(index)
    }

    /// All rows in stable insertion order.
    #[must_use]
    pub fn rows(&self) -> &[Image] {
        &self.rows
    }

    /// One interned image row.
    #[must_use]
    pub fn get(&self, index: u32) -> Option<&Image> {
        self.rows.get(usize::try_from(index).ok()?)
    }

    /// Number of unique image descriptors and payloads.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Whether no image has been interned.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Prepare a transactional frame view while retaining decoded textures.
    ///
    /// # Errors
    /// Returns a typed refusal when the bounded texture index cannot be cloned
    /// without allocation failure. The published table remains unchanged.
    pub(crate) fn prepare_frame(&self) -> Result<Self, TableError> {
        let mut textures = HashMap::new();
        textures
            .try_reserve(self.textures.len())
            .map_err(|_| allocation_failed("image texture index", 0, self.textures.len()))?;
        textures.extend(
            self.textures
                .iter()
                .map(|(&descriptor, texture)| (descriptor, Arc::clone(texture))),
        );
        Ok(Self {
            rows: Vec::new(),
            index: HashMap::new(),
            textures,
        })
    }

    /// Drop decoded textures not referenced by this prepared frame.
    pub(crate) fn retain_current(&mut self) {
        self.textures
            .retain(|descriptor, _| self.index.contains_key(descriptor));
    }
}

/// A compiled outline: geometry that many instances share.
///
/// The unit §10.8 interns. Its identity is the **content digest of the object-
/// space points**, not the mobject that produced it, which is exactly what makes
/// two copies of one glyph — or two `Dot`s, or a hundred arrow tips — collapse
/// to one compiled path.
///
/// **Geometry here is shape-local**: translated so the outline's first anchor
/// sits at the origin, which is the same normalization [`shape_digest`] hashes
/// under. That is not tidiness, it is what makes interning *correct*. Store the
/// points where the first mobject happened to be and every later instance
/// sharing the outline inherits that position — the interning still dedups, and
/// the picture is wrong. An instance's [`Instance::placement`] is the only thing
/// that says where the outline goes.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    /// The content address of the object-space points.
    pub digest: Digest,
    /// The segment range this shape owns in [`crate::plan::RenderPlan::segments`]
    /// — §10.8's `SegmentTable`, which is one flat table for the whole scene.
    pub first_segment: u32,
    /// How many segments.
    pub segment_count: u32,
    /// Object-space bounds — the control-polygon hull, which contains the curve.
    pub bounds: BoundingBox,
    /// The kernel route (§10.8's primitive hint).
    pub hint: Hint,
    /// Total object-space arc length, and the per-curve table §7.3 owns.
    pub arc_length: ArcLengthTable,
    /// Segment indices, **relative to `first_segment`**, at which a subpath
    /// begins. Always starts with `0` for a shape that has any segments.
    ///
    /// fmn-geom marks a subpath break with a null curve, and [`compile_shape`]
    /// drops those — a null curve is not a drawn piece. Dropping them without
    /// recording *where* they were loses the only thing they carry, and §10.2's
    /// fill is the consumer that needs it: a fill closes each subpath, so it has
    /// to know where one ends and the next begins. Recovering the boundaries
    /// from anchor continuity (`p2 != p0`) is almost right, and silently wrong
    /// exactly when one subpath begins where the previous one ended — a letter
    /// `o` whose counter is drawn starting from the point the outer ring closed
    /// at would fill as one ring with a chord through it.
    pub subpath_starts: Vec<u32>,
}

/// One occurrence of a [`Shape`]: transform, style, clip, order.
///
/// §10.8's word is "instance", and the fields are its list. An instance is
/// deliberately small — an index, a placement and two more indices — because the
/// entire point of interning is that this is what a repeated glyph costs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Instance {
    /// Which compiled shape.
    pub shape: u32,
    /// Which style row.
    pub style: u32,
    /// The mobject this occurrence came from, for the inspector and the journal.
    pub mob: Mob,
    /// Object→world placement.
    pub placement: Placement,
    /// Painter order: position in the draw sequence.
    pub order: u32,
    /// A fold of this occurrence's seven revision axes at sync time.
    ///
    /// Carried on the instance rather than looked up because the tile cache's
    /// key (§10.8) is per *tile*, and a tile knows its commands rather than
    /// their mobjects. One `u64` per occurrence is the cheapest thing that makes
    /// the key complete.
    pub revisions: u64,
    /// Whether a **writable live view** exists on this occurrence's buffer.
    ///
    /// §8.2: "while a *writable* live view exists, the affected object … is
    /// conservatively invalidated each frame — a live view never silently
    /// receives weaker semantics to buy speed." A view can mutate points with no
    /// Stage method called and therefore no revision bumped, so this is not
    /// foldable into `revisions`: it *poisons* the tile key instead, and the
    /// difference is that a poisoned tile can never hit however many frames pass.
    pub volatile: bool,
    /// Whether writable point storage makes the compiled primitive hint unsafe.
    ///
    /// This is narrower than [`Instance::volatile`]: a field-scoped colour view
    /// poisons cached pixels but cannot invalidate a geometry hint. Binning and
    /// the frame compiler use this bit to take their general, fail-closed paths.
    pub hint_unsafe: bool,
}

/// Interned outlines and their occurrences.
///
/// The `InstanceTable` of §10.8, plus the shape table it indexes into. They live
/// in one type because an instance without its shape is not a thing, and because
/// the dedup statistic — N occurrences, M compiled shapes — is the number this
/// mechanism exists to move and should be readable in one place.
#[derive(Debug, Clone, Default)]
pub struct ShapeTable {
    shapes: Vec<Shape>,
    by_digest: HashMap<Digest, u32>,
    instances: Vec<Instance>,
}

impl ShapeTable {
    /// Intern an outline by its content digest.
    ///
    /// `build` runs only on a miss — which is the whole mechanism: compiling a
    /// glyph outline means decoding it, splitting it, and measuring its arc
    /// length, and doing that once per occurrence is what §10.8 calls "the bulk
    /// of geometry duplication".
    ///
    /// # Errors
    /// Returns [`TableError::IndexCapacityExceeded`] before running `build` or
    /// mutating the table when one more row would exceed its `u32` index width,
    /// [`TableError::AllocationFailed`] when its row or digest index cannot
    /// reserve that row, or [`TableError::ShapeIdentityMismatch`] when the
    /// builder returns a shape carrying a different digest. Every refusal
    /// leaves the semantic table rows and indices unchanged.
    pub fn intern_shape(
        &mut self,
        digest: Digest,
        build: impl FnOnce() -> Shape,
    ) -> Result<u32, TableError> {
        if let Some(&i) = self.by_digest.get(&digest) {
            return Ok(i);
        }
        let (i, _) = rows_after_push("shape rows", self.shapes.len())?;
        self.shapes
            .try_reserve(1)
            .map_err(|_| allocation_failed("shape rows", self.shapes.len(), 1))?;
        self.by_digest
            .try_reserve(1)
            .map_err(|_| allocation_failed("shape index", self.by_digest.len(), 1))?;
        let shape = build();
        if shape.digest != digest {
            return Err(TableError::ShapeIdentityMismatch {
                requested: digest,
                built: shape.digest,
            });
        }
        self.shapes.push(shape);
        self.by_digest.insert(digest, i);
        Ok(i)
    }

    /// Record an occurrence, returning its index.
    ///
    /// # Errors
    /// Returns [`TableError::IndexCapacityExceeded`] before mutation when one
    /// more occurrence would exceed the instance table's `u32` index width, or
    /// [`TableError::AllocationFailed`] when the occurrence cannot be reserved.
    pub fn push_instance(&mut self, instance: Instance) -> Result<u32, TableError> {
        let (index, _) = rows_after_push("instance rows", self.instances.len())?;
        self.instances
            .try_reserve(1)
            .map_err(|_| allocation_failed("instance rows", self.instances.len(), 1))?;
        self.instances.push(instance);
        Ok(index)
    }

    pub(crate) fn shape_index(&self, digest: Digest) -> Option<u32> {
        self.by_digest.get(&digest).copied()
    }

    pub(crate) fn reserve_shapes(&mut self, shapes: usize) -> Result<(), TableError> {
        let total_shapes =
            self.shapes
                .len()
                .checked_add(shapes)
                .ok_or(TableError::IndexCapacityExceeded {
                    resource: "shape rows",
                    requested: u64::MAX,
                })?;
        check_row_count("shape rows", total_shapes)?;
        self.shapes
            .try_reserve_exact(shapes)
            .map_err(|_| allocation_failed("shape rows", self.shapes.len(), shapes))?;
        self.by_digest
            .try_reserve(shapes)
            .map_err(|_| allocation_failed("shape index", self.by_digest.len(), shapes))?;
        Ok(())
    }

    pub(crate) fn insert_prepared(&mut self, index: u32, shape: Shape) {
        debug_assert_eq!(usize::try_from(index).ok(), Some(self.shapes.len()));
        debug_assert!(!self.by_digest.contains_key(&shape.digest));
        self.by_digest.insert(shape.digest, index);
        self.shapes.push(shape);
    }

    pub(crate) fn swap_instances(&mut self, instances: &mut Vec<Instance>) {
        std::mem::swap(&mut self.instances, instances);
    }

    /// The compiled shapes.
    #[must_use]
    pub fn shapes(&self) -> &[Shape] {
        &self.shapes
    }

    /// The occurrences, in painter order.
    #[must_use]
    pub fn instances(&self) -> &[Instance] {
        &self.instances
    }

    /// One shape.
    #[must_use]
    pub fn shape(&self, index: u32) -> Option<&Shape> {
        self.shapes.get(usize::try_from(index).ok()?)
    }

    /// How many occurrences share one compiled shape, on average.
    ///
    /// The number §10.8's interning exists to raise, and the one an IR golden
    /// snapshot should carry: it is `1.0` for a scene of distinct paths and `N`
    /// for `N` copies of a glyph.
    #[must_use]
    pub fn instances_per_shape(&self) -> f64 {
        if self.shapes.is_empty() {
            return 0.0;
        }
        self.instances.len() as f64 / self.shapes.len() as f64
    }

    /// Forget the occurrences, keeping the compiled shapes.
    ///
    /// The per-frame half of the retained plan: painter order is a property of
    /// the frame, so the instance list is rebuilt every sync while the outlines
    /// it indexes persist. Clearing both would make interning worthless.
    pub fn clear_instances(&mut self) {
        self.instances.clear();
    }

    /// Forget everything.
    pub fn clear(&mut self) {
        self.shapes.clear();
        self.by_digest.clear();
        self.instances.clear();
    }
}

const SHAPE_SCHEMA: Schema = Schema::new(*b"FMNR", 1, 1, 0);

/// Hash the fixed FMNR framing around one already-canonical coordinate stream.
///
/// The callback must write exactly `point_count * 3` encoded `f64` values. It
/// stays private so the only production caller derives both the count and the
/// values from the same slice. Keeping the framing separate also lets the
/// over-envelope regression feed repeated encoded blocks without allocating an
/// enormous outline.
fn framed_shape_digest(point_count: u64, write_coordinates: impl FnOnce(&mut Sha256)) -> Digest {
    let coordinate_bytes = point_count.saturating_mul(3 * 8);
    let payload_len = 8u64.saturating_add(coordinate_bytes);

    let mut document = Sha256::new();
    document.update(&SHAPE_SCHEMA.magic);
    document.update(&SHAPE_SCHEMA.id.to_le_bytes());
    document.update(&SHAPE_SCHEMA.major.to_le_bytes());
    document.update(&SHAPE_SCHEMA.minor.to_le_bytes());
    document.update(&0u16.to_le_bytes()); // flags
    document.update(&0u16.to_le_bytes()); // reserved
    document.update(&payload_len.to_le_bytes());
    document.update(&point_count.to_le_bytes());
    write_coordinates(&mut document);

    let checksum = document.clone().finalize();
    document.update(checksum.as_bytes());
    document.finalize()
}

/// The content address of an outline: object-space points, canonically hashed.
///
/// Uses `fmn-hash`'s versioned envelope rather than an ad-hoc byte dump, so the
/// digest is stable across platforms and versions — floats are canonicalized at
/// the boundary (`-0.0 → +0.0`, one canonical NaN), which matters here because
/// two glyphs that differ only in the sign of a zero are the same outline and
/// must intern together.
///
/// **Position is excluded**: the points are hashed relative to the first anchor,
/// so two copies of one glyph at different places share a compiled shape and
/// differ only in their [`Instance::placement`]. That is the entire mechanism
/// — without it, interning would find nothing in a real formula.
#[must_use]
pub fn shape_digest(points: &[Vec3]) -> Digest {
    let origin = points.first().copied().unwrap_or([0.0; 3]);
    let point_count = u64::try_from(points.len()).unwrap_or(u64::MAX);
    framed_shape_digest(point_count, |document| {
        for point in points {
            for coordinate in 0..3 {
                let relative = canonicalize_f64(point[coordinate] - origin[coordinate]);
                document.update(&relative.to_bits().to_le_bytes());
            }
        }
    })
}

/// Build a [`Shape`] from an object-space path.
///
/// The compile step §10.8 wants to happen once per *outline* rather than once
/// per occurrence: segments, hull bounds, the arc-length table, and the hint.
///
/// # Errors
/// Returns [`TableError::IndexCapacityExceeded`] when the shape's segment or
/// subpath rows, or its range in the caller's flat segment table, cannot be
/// represented by the retained IR's `u32` indices.
pub fn compile_shape(
    digest: Digest,
    path: &QuadPath,
    hint: Hint,
    first_segment: u32,
) -> Result<(Shape, Vec<Segment>), TableError> {
    let arc_length = ArcLengthTable::for_path(path);
    let lengths = arc_length.curve_lengths();

    // A subpath break is **a handle sitting on top of the previous anchor**,
    // which is fmn-geom's convention and the Reference's — not a zero-length
    // curve. The two coincide only when the next subpath happens to begin where
    // the last one ended, so testing length finds an annulus whose rings are
    // concentric and misses every glyph counter. Asking `QuadPath` rather than
    // re-deriving the predicate also keeps the Reference's 1e-4 disambiguation
    // tolerance in the one place that owns it.
    let breaks: std::collections::HashSet<usize> = path
        .subpath_end_indices()
        .into_iter()
        // The last entry is the path's own end, not a break; it is the only one
        // that can be an odd index, and it never names a curve start.
        .filter(|&i| i.is_multiple_of(2) && i + 2 < path.points().len())
        .collect();

    // The ramp normalizes over the length actually drawn. A dropped break has
    // real length, and leaving it in the denominator would stop every stroke's
    // width and colour ramp short of its endpoint. Summing the kept curves in
    // curve order is bit-identical to `ArcLengthTable::total()` for any path
    // with nothing to drop, which is every single-subpath shape.
    let mut total = 0.0f64;
    for i in 0..path.num_curves() {
        let len = lengths.get(i).copied().unwrap_or(0.0);
        if len > 0.0 && !breaks.contains(&(2 * i)) {
            total += len;
        }
    }

    let mut segments = Vec::with_capacity(path.num_curves());
    let mut subpath_starts: Vec<u32> = Vec::new();
    let mut opens_subpath = true;
    let mut acc = 0.0f64;
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];

    for i in 0..path.num_curves() {
        let Some([p0, p1, p2]) = path.nth_curve_points(i) else {
            continue;
        };
        let len = lengths.get(i).copied().unwrap_or(0.0);
        if breaks.contains(&(2 * i)) {
            // Not a drawn piece: dropping it is what keeps a stroked annulus
            // from growing a radial spoke between its two rings.
            opens_subpath = true;
            continue;
        }
        if len <= 0.0 {
            // A true null curve (`p0 == p1 == p2`) draws nothing and, per
            // `subpath_end_indices`' own disambiguation, does not open a
            // subpath either — a run of them is padding, not a boundary.
            continue;
        }
        if opens_subpath {
            let start =
                u32::try_from(segments.len()).map_err(|_| TableError::IndexCapacityExceeded {
                    resource: "shape subpath rows",
                    requested: u64::try_from(segments.len())
                        .unwrap_or(u64::MAX)
                        .saturating_add(1),
                })?;
            subpath_starts.push(start);
            opens_subpath = false;
        }
        let s0 = if total > 0.0 { acc / total } else { 0.0 };
        acc += len;
        let s1 = if total > 0.0 { acc / total } else { 0.0 };
        for p in [p0, p1, p2] {
            for k in 0..3 {
                min[k] = min[k].min(p[k]);
                max[k] = max[k].max(p[k]);
            }
        }
        segments.push(Segment { p0, p1, p2, s0, s1 });
    }

    let bounds = if segments.is_empty() {
        BoundingBox::ZERO
    } else {
        BoundingBox {
            min,
            mid: [
                0.5 * (min[0] + max[0]),
                0.5 * (min[1] + max[1]),
                0.5 * (min[2] + max[2]),
            ],
            max,
        }
    };

    let count = u32::try_from(segments.len()).map_err(|_| TableError::IndexCapacityExceeded {
        resource: "shape segment rows",
        requested: u64::try_from(segments.len()).unwrap_or(u64::MAX),
    })?;
    let requested = u64::from(first_segment) + u64::from(count);
    if requested > u64::from(u32::MAX) {
        return Err(TableError::IndexCapacityExceeded {
            resource: "segment range",
            requested,
        });
    }
    Ok((
        Shape {
            digest,
            first_segment,
            segment_count: count,
            bounds,
            hint,
            arc_length,
            subpath_starts,
        },
        segments,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_core::constants::TAU;
    use fmn_hash::{Limits, Writer, sha256};

    fn style_with(width: f32) -> Style {
        Style {
            stroke_width: width,
            ..Style::default()
        }
    }

    fn tri() -> QuadPath {
        let mut p = QuadPath::default();
        p.start_new_path([0.0, 0.0, 0.0]);
        p.add_line_to([2.0, 0.0, 0.0], false).unwrap();
        p.add_line_to([1.0, 1.0, 0.0], false).unwrap();
        p.add_line_to([0.0, 0.0, 0.0], false).unwrap();
        p
    }

    fn legacy_shape_digest(points: &[Vec3]) -> Digest {
        let origin = points.first().copied().unwrap_or([0.0; 3]);
        let mut writer = Writer::new(SHAPE_SCHEMA);
        writer.put_u64(points.len() as u64);
        for point in points {
            for coordinate in 0..3 {
                writer.put_f64(point[coordinate] - origin[coordinate]);
            }
        }
        sha256(&writer.finish().expect("small shape identity document"))
    }

    #[test]
    fn image_table_deduplicates_complete_descriptors_not_pixels_alone() {
        let pixels = vec![10, 20, 30, 255];
        let srgb = ImageResource::rgba8(
            1,
            1,
            pixels.clone(),
            ImageColorSpace::Srgb,
            ImageSampler::default(),
        )
        .unwrap();
        let linear = ImageResource::rgba8(
            1,
            1,
            pixels.clone(),
            ImageColorSpace::Linear,
            ImageSampler::default(),
        )
        .unwrap();
        let clamped = ImageResource::rgba8(
            1,
            1,
            pixels,
            ImageColorSpace::Srgb,
            ImageSampler {
                wrap_u: ImageWrap::ClampToEdge,
                wrap_v: ImageWrap::Repeat,
            },
        )
        .unwrap();

        let mut table = ImageTable::default();
        let first = table.intern(&srgb).unwrap();
        assert_eq!(table.intern(&srgb).unwrap(), first);
        let linear_index = table.intern(&linear).unwrap();
        let clamped_index = table.intern(&clamped).unwrap();
        assert_ne!(first, linear_index, "transfer function is semantic");
        assert_ne!(first, clamped_index, "sampler policy is semantic");
        assert_eq!(table.len(), 3);
        assert_eq!(
            table.get(first).unwrap().descriptor().content,
            table.get(linear_index).unwrap().descriptor().content,
            "the raw payload keeps one fmn-cache content address"
        );
    }

    #[test]
    fn streaming_shape_identity_preserves_in_envelope_document_digests() {
        let fixtures = [
            Vec::new(),
            vec![[1.0, -0.0, f64::NAN]],
            tri().points().to_vec(),
        ];
        for points in fixtures {
            assert_eq!(shape_digest(&points), legacy_shape_digest(&points));
        }
    }

    #[test]
    fn over_envelope_shape_identity_never_uses_the_empty_digest_fallback() {
        const HEADER_BYTES: u64 = 24;
        const CHECKSUM_BYTES: u64 = 32;
        const COUNT_BYTES: u64 = 8;
        const BYTES_PER_POINT: u64 = 3 * 8;
        const BLOCK_BYTES: usize = 1 << 20;

        let max_total = u64::try_from(Limits::DEFAULT.max_total).expect("limit fits u64");
        let framed_fixed = HEADER_BYTES + CHECKSUM_BYTES + COUNT_BYTES;
        let point_count = (max_total - framed_fixed) / BYTES_PER_POINT + 1;
        let coordinate_bytes = point_count * BYTES_PER_POINT;
        assert!(framed_fixed + coordinate_bytes > max_total);

        let zeroes = vec![0u8; BLOCK_BYTES];
        let digest = framed_shape_digest(point_count, |document| {
            let mut remaining = coordinate_bytes;
            while remaining >= BLOCK_BYTES as u64 {
                document.update(&zeroes);
                remaining -= BLOCK_BYTES as u64;
            }
            let tail = usize::try_from(remaining).expect("tail is smaller than one block");
            document.update(&zeroes[..tail]);
        });
        assert_ne!(
            digest,
            sha256(&[]),
            "the old over-limit fallback collapsed every such outline here"
        );
    }

    #[test]
    fn table_width_checks_refuse_wrap_without_large_allocations() {
        let requested = u64::from(u32::MAX) + 1;
        assert_eq!(
            rows_after_push(
                "style rows",
                usize::try_from(u32::MAX).expect("usize represents u32 on supported targets"),
            ),
            Err(TableError::IndexCapacityExceeded {
                resource: "style rows",
                requested,
            })
        );

        let path = tri();
        assert_eq!(
            compile_shape(
                shape_digest(path.points()),
                &path,
                Hint::General,
                u32::MAX - 2,
            )
            .expect_err("the exclusive segment end exceeds u32"),
            TableError::IndexCapacityExceeded {
                resource: "segment range",
                requested,
            }
        );
    }

    #[test]
    fn shape_interning_refuses_mismatched_builder_identity_atomically() {
        let path = tri();
        let built_digest = shape_digest(path.points());
        let requested_digest = sha256(b"a different retained shape");
        let (shape, _) = compile_shape(built_digest, &path, Hint::General, 0)
            .expect("fixture fits retained table widths");
        let mut table = ShapeTable::default();

        assert_eq!(
            table
                .intern_shape(requested_digest, || shape.clone())
                .expect_err("the index identity must match the built shape"),
            TableError::ShapeIdentityMismatch {
                requested: requested_digest,
                built: built_digest,
            }
        );
        assert!(table.shapes().is_empty());
        assert!(table.instances().is_empty());
        assert_eq!(table.shape_index(requested_digest), None);
        assert_eq!(table.shape_index(built_digest), None);

        let index = table
            .intern_shape(built_digest, || shape.clone())
            .expect("the corrected identity occupies the first row");
        assert_eq!(index, 0);
        let mut rebuilt = false;
        assert_eq!(
            table
                .intern_shape(built_digest, || {
                    rebuilt = true;
                    shape
                })
                .expect("the corrected identity deduplicates"),
            index
        );
        assert!(!rebuilt);
        assert_eq!(table.shapes().len(), 1);
    }

    #[test]
    fn normalized_arc_length_similarity_admission_is_exact_and_conservative() {
        let admitted = [
            Placement::IDENTITY,
            Placement::from_translation([17.0, -31.0, 5.0]),
            Placement::new(
                [[2.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 2.0]],
                [0.0; 3],
            ),
            Placement::new(
                [[0.0, -2.0, 0.0], [2.0, 0.0, 0.0], [0.0, 0.0, -2.0]],
                [1.0, 2.0, 3.0],
            ),
        ];
        for placement in admitted {
            assert!(
                retains_normalized_arc_length(placement),
                "exact signed-axis similarity was rejected: {placement:?}"
            );
        }

        let rejected = [
            Placement::new([[0.0; 3]; 3], [0.0; 3]),
            Placement::new(
                [[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 2.0]],
                [0.0; 3],
            ),
            Placement::new(
                [[1.0, 0.25, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                [0.0; 3],
            ),
            // This 3-4-5 matrix is mathematically a dense rotation. Without
            // construction provenance it intentionally takes the oracle path.
            Placement::new(
                [[0.8, -0.6, 0.0], [0.6, 0.8, 0.0], [0.0, 0.0, 1.0]],
                [0.0; 3],
            ),
            Placement::new(
                [
                    [f64::INFINITY, 0.0, 0.0],
                    [0.0, f64::INFINITY, 0.0],
                    [0.0, 0.0, f64::INFINITY],
                ],
                [0.0; 3],
            ),
        ];
        for placement in rejected {
            assert!(
                !retains_normalized_arc_length(placement),
                "non-proven similarity was admitted: {placement:?}"
            );
        }
    }

    #[test]
    fn styles_intern_by_bits_and_never_merge_distinct_ones() {
        let mut t = StyleTable::default();
        let a = t.intern(style_with(2.0)).expect("one style row");
        let b = t.intern(style_with(2.0)).expect("deduplicated style row");
        assert_eq!(a, b);
        assert_eq!(t.len(), 1);
        // One ulp apart is two styles: §8.5 makes batching semantics, so a
        // table that merged these would change how many draw calls a scene
        // takes. `2.0 + f32::EPSILON` would NOT do — EPSILON is the ulp at 1.0,
        // and at 2.0 it rounds straight back, which is how this test first
        // passed against a key that could not tell them apart.
        let c = t
            .intern(style_with(f32::from_bits(2.0f32.to_bits() + 1)))
            .expect("second style row");
        assert_ne!(c, a);
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn every_style_field_participates_in_the_key() {
        // The failure this guards is a field added to Style and forgotten in
        // `bits`, which silently merges two distinct styles forever after.
        let base = Style::default();
        let variants: Vec<Style> = vec![
            Style {
                stroke_rgba: [1.0, 0.0, 0.0, 1.0],
                ..base.clone()
            },
            Style {
                stroke_rgba_end: [1.0, 0.0, 0.0, 1.0],
                ..base.clone()
            },
            Style {
                stroke_width: 1.0,
                ..base.clone()
            },
            Style {
                stroke_width_end: 1.0,
                ..base.clone()
            },
            Style {
                fill_rgba: [0.0, 1.0, 0.0, 1.0],
                ..base.clone()
            },
            Style {
                fill_rgba_end: [0.0, 1.0, 0.0, 1.0],
                ..base.clone()
            },
            Style {
                fill_border_width: 1.0,
                ..base.clone()
            },
            Style {
                anti_alias_width: 2.0,
                ..base.clone()
            },
            Style {
                joint_type: JointType::Miter,
                ..base.clone()
            },
            Style {
                is_fixed_in_frame: 0.5,
                ..base.clone()
            },
            Style {
                shading: [0.1, 0.2, 0.3],
                ..base.clone()
            },
            Style {
                clip_planes: [[1.0, 0.0, 0.0, -1.0], [0.0; 4], [0.0; 4], [0.0; 4]],
                ..base.clone()
            },
            Style {
                flat_stroke: true,
                ..base.clone()
            },
            Style {
                scale_stroke_with_zoom: true,
                ..base.clone()
            },
            Style {
                stroke_behind: true,
                ..base.clone()
            },
            Style {
                depth_test: true,
                ..base.clone()
            },
        ];

        let mut t = StyleTable::default();
        let baseline = t.intern(base).expect("baseline style row");
        for (i, v) in variants.iter().enumerate() {
            assert_ne!(
                t.intern(v.clone()).expect("variant style row"),
                baseline,
                "variant {i} interned as the default style — a field is missing from the key"
            );
        }
        assert_eq!(t.len(), variants.len() + 1);
    }

    #[test]
    fn uniforms_reach_the_style_row() {
        let u = Uniforms {
            is_fixed_in_frame: 0.25,
            shading: [0.1, 0.2, 0.3],
            clip_planes: [[1.0, 0.0, 0.0, -2.0], [0.0; 4], [0.0; 4], [0.0; 4]],
            anti_alias_width: 2.5,
            joint_type: JointType::Bevel,
            flat_stroke: true,
            scale_stroke_with_zoom: true,
            stroke_behind: true,
            depth_test: true,
            ..Uniforms::default()
        };
        let s = Style::default().with_uniforms(&u);
        assert_eq!(s.is_fixed_in_frame, 0.25);
        assert_eq!(s.shading, [0.1, 0.2, 0.3]);
        assert_eq!(
            s.clip_planes,
            [[1.0, 0.0, 0.0, -2.0], [0.0; 4], [0.0; 4], [0.0; 4]]
        );
        assert_eq!(s.anti_alias_width, 2.5);
        assert_eq!(s.joint_type, JointType::Bevel);
        assert!(s.flat_stroke);
        assert!(s.scale_stroke_with_zoom);
        assert!(s.stroke_behind);
        assert!(s.depth_test);
    }

    #[test]
    fn one_outline_at_two_places_is_one_compiled_shape() {
        // The instancing claim, at its smallest: N copies of a glyph must
        // produce 1 compiled path and N instances.
        let points: Vec<Vec3> = tri().points().to_vec();
        let shifted: Vec<Vec3> = points
            .iter()
            .map(|p| [p[0] + 100.0, p[1] - 7.5, p[2]])
            .collect();
        assert_eq!(
            shape_digest(&points),
            shape_digest(&shifted),
            "position must not be part of an outline's identity"
        );

        // A real handle, because Instance carries one and a synthetic Mob is
        // not constructible from outside Marionette — which is the point of a
        // generational handle.
        let mut stage = fmn_mobject::Stage::new();
        let mob = stage.add(fmn_mobject::Mobject::new());

        let mut table = ShapeTable::default();
        let mut compiles = 0;
        for (i, pts) in [points.clone(), shifted.clone(), points.clone()]
            .into_iter()
            .enumerate()
        {
            let digest = shape_digest(&pts);
            let shape = table
                .intern_shape(digest, || {
                    compiles += 1;
                    let path = QuadPath::from_points(pts.clone()).expect("valid path");
                    compile_shape(digest, &path, Hint::Line, 0)
                        .expect("fixture fits retained table widths")
                        .0
                })
                .expect("shape row");
            table
                .push_instance(Instance {
                    shape,
                    style: 0,
                    mob,
                    placement: Placement::from_translation([pts[0][0], pts[0][1], pts[0][2]]),
                    order: u32::try_from(i).expect("small fixture"),
                    revisions: 0,
                    volatile: false,
                    hint_unsafe: false,
                })
                .expect("instance row");
        }
        assert_eq!(compiles, 1, "the outline must compile once");
        assert_eq!(table.shapes().len(), 1);
        assert_eq!(table.instances().len(), 3);
        assert!((table.instances_per_shape() - 3.0).abs() < 1e-12);
    }

    #[test]
    fn a_different_outline_is_a_different_shape() {
        let a: Vec<Vec3> = tri().points().to_vec();
        let mut b = a.clone();
        b[2] = [2.0, 0.5, 0.0];
        assert_ne!(shape_digest(&a), shape_digest(&b));
    }

    #[test]
    fn a_signed_zero_does_not_split_an_outline() {
        // fmn-hash canonicalizes floats at the boundary. Two glyphs differing
        // only in the sign of a zero are one outline, and interning them
        // separately would silently halve the mechanism's value on real text.
        let a: Vec<Vec3> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let b: Vec<Vec3> = vec![[0.0, -0.0, 0.0], [1.0, -0.0, 0.0], [2.0, -0.0, 0.0]];
        assert_eq!(shape_digest(&a), shape_digest(&b));
    }

    #[test]
    fn a_subpath_that_begins_elsewhere_is_still_a_subpath() {
        // fmn-geom signals a subpath break with **a handle sitting on top of the
        // previous anchor** (`QuadPath::subpath_end_indices`, and the Reference's
        // own convention). A break to a *different* location therefore has
        // nonzero length, so testing for a zero-length curve finds only the
        // special case where the next subpath happens to start where the last
        // one ended.
        //
        // Two consequences, both visible: an annulus or a glyph counter compiles
        // as ONE subpath, which §10.2 says makes it "fill as one ring with a
        // chord through it"; and the break itself survives into the segment list,
        // so a stroked annulus grows a radial spoke from its outer ring to its
        // inner one.
        let outer = QuadPath::try_arc(0.0, TAU, 1.0, [0.0, 0.0, 0.0], Some(8)).expect("valid arc");
        let inner = QuadPath::try_arc(0.0, -TAU, 0.4, [0.0, 0.0, 0.0], Some(8)).expect("valid arc");
        let mut ring = QuadPath::new();
        ring.add_subpath(outer.points()).expect("outer");
        ring.add_subpath(inner.points()).expect("inner");

        // fmn-geom itself sees two subpaths; the compiler must agree with it.
        assert_eq!(ring.subpaths().len(), 2, "the fixture is not a ring");

        let (shape, segments) = compile_shape(shape_digest(ring.points()), &ring, Hint::General, 0)
            .expect("fixture fits retained table widths");
        assert_eq!(
            shape.subpath_starts.len(),
            2,
            "the compiled shape lost a subpath boundary: {:?}",
            shape.subpath_starts
        );
        // And the break is not a drawn piece.
        assert_eq!(
            segments.len(),
            16,
            "the subpath break survived as a drawn segment"
        );
        for s in &segments {
            assert!(
                s.p0 != s.p1 || s.p1 == s.p2,
                "a break curve (handle on the anchor) is in the segment list"
            );
        }
    }

    #[test]
    fn compiling_a_shape_partitions_arc_length_and_hulls_the_curve() {
        let path = tri();
        let digest = shape_digest(path.points());
        let (shape, segments) = compile_shape(digest, &path, Hint::Line, 7)
            .expect("fixture fits retained table widths");
        assert_eq!(shape.first_segment, 7);
        assert_eq!(
            shape.segment_count,
            u32::try_from(segments.len()).expect("small fixture")
        );
        assert_eq!(segments.len(), 3);

        // The spans partition [0, 1] in order.
        assert_eq!(segments[0].s0, 0.0);
        assert!((segments[segments.len() - 1].s1 - 1.0).abs() < 1e-12);
        for w in segments.windows(2) {
            assert_eq!(w[0].s1, w[1].s0);
        }
        // Hull bounds contain every control point, hence the curve.
        assert_eq!(shape.bounds.min, [0.0, 0.0, 0.0]);
        assert_eq!(shape.bounds.max, [2.0, 1.0, 0.0]);
        assert_eq!(shape.bounds.mid, [1.0, 0.5, 0.0]);
        // And the arc length is the shared §7.3 artifact, not a private one:
        // the triangle's perimeter, 2 + 2*sqrt(2).
        let want = 2.0 + 2.0 * std::f64::consts::SQRT_2;
        assert!(
            (shape.arc_length.total() - want).abs() < 1e-9,
            "{} vs {want}",
            shape.arc_length.total()
        );
    }

    #[test]
    fn a_path_with_no_curves_compiles_to_nothing_rather_than_a_degenerate_shape() {
        let mut p = QuadPath::default();
        p.start_new_path([1.0, 1.0, 0.0]);
        let (shape, segments) = compile_shape(shape_digest(p.points()), &p, Hint::General, 0)
            .expect("fixture fits retained table widths");
        assert!(segments.is_empty());
        assert_eq!(shape.segment_count, 0);
        assert_eq!(shape.bounds, BoundingBox::ZERO);
    }
}
