//! Marionette's persistence layer (§8.7, fm-879): arena snapshots as
//! durable bytes in fmn-hash's canonical container (§6.7), one mechanism
//! backing four consumers — SceneState (§13.1), Studio undo, replay-
//! journal barriers (§13.4), and the Gauntlet's geometry-snapshot
//! self-goldens (§16.3).
//!
//! Format guarantees, exactly the §6.7 policy:
//! - **Versioned schema ids** ([`SNAPSHOT_SCHEMA`] `FMNA/1` v1.8,
//!   [`SCENE_STATE_SCHEMA`] `FMNA/2` v2.0; the self-golden suites hold `FMNS`):
//!   additive-minor / breaking-major from day one — snapshots persist in
//!   caches and repro bundles.
//! - **Deterministic bytes**: canonical field order (schema order for
//!   record columns, slot order for the arena, no map anywhere), float
//!   canonicalization at the write boundary (`-0 → +0`, one NaN — the
//!   [`fmn_hash::Writer`] rule), so snapshot-hash equality is meaningful
//!   for the replay journal and the self-goldens.
//! - **Corruption detection and size limits on read**: the container's
//!   trailing SHA-256 and [`fmn_hash::Limits`], enforced by
//!   [`fmn_hash::Reader::open`] before any payload is touched, then the
//!   decoded-allocation budget ([`SnapshotLimits`], fm-vek.7): every
//!   fixed-width count is preflighted against both the remaining encoded
//!   bytes and the aggregate decoded-bytes ceiling before the destination
//!   storage it names is reserved.
//!
//! The honesty clause, stated where it binds (§8.7, §13.4): **updater
//! callables and native executable attributes never serialize.** A durable snapshot records each updater's
//! identity — `(UpdaterId, kind)` — and nothing else; decode returns that
//! manifest ([`UpdaterManifest`]) alongside a [`Snapshot`] whose entries
//! carry no callables. Re-binding callables (and invalidating a barrier
//! when a callback's version hash changed) is the replay journal's job
//! (fm-y7u), which consumes these identities. A consequence worth knowing:
//! re-encoding a decoded snapshot of an updater-bearing stage yields
//! different bytes while the callables are absent; rebinding every manifest
//! identity through [`Stage::restore_updater_bindings`] restores the original
//! canonical bytes. Byte-level re-open determinism holds directly for
//! callable-free states.
//! Native attributes registered with `add_dt_updater_with_state` must likewise
//! be reconstructed by replay or explicitly restored with `set_updater_state`;
//! the manifest alone does not describe an executable history.
//!
//! Handles serialize as `(slot index, generation)` — the stage id is a
//! process-local mint, re-bound at decode against the target stage
//! ([`Snapshot::from_bytes`] takes the stage whose id decoded handles
//! adopt), so a repro bundle restores into any fresh arena.

use fmn_core::rng::Pcg64Dxsm;
use fmn_hash::{Digest, Limits, Reader, Schema, SerialError, UnknownPolicy, Writer, sha256};

use fmn_core::types::Vec3;

use crate::record::{RecordBuffer, RecordSchema};
use crate::shape::{ShapeSlot, ShapeTag};
use crate::stage::{Mob, Snapshot, SnapshotEntry, Stage, UpdaterFn, UpdaterId};
use crate::uniforms::{JointType, Uniforms};
use crate::{ImageColorSpace, ImageResource, ImageSampler, ImageWrap, Placement, RenderPrimitive};

/// The arena-snapshot document: magic `FMNA`, schema id 1, version 1.8.
///
/// Minor history: 1.1 appended the shape tag; 1.2 appended `z_index`; 1.3
/// preserves the monotonic updater-id cursor, including identities removed
/// before the snapshot; 1.4 appended the object→world placement table
/// (fm-7if); 1.5 appended durable renderer primitive/topology identity;
/// 1.6 appended immutable image descriptors and RGBA8 bytes; **1.7 appends
/// the per-buffer `_data_defaults` phantom record** (fm-laky): one
/// presence bool and, when set, exactly one stride of f32 lanes. **1.8 appends
/// durable dark-side image resources**, retaining the full 1.7 payload prefix.
pub const SNAPSHOT_SCHEMA: Schema = Schema::new(*b"FMNA", 1, 1, 8);

/// The scene-state envelope: magic `FMNA`, schema id 2, version 2.0.
///
/// Version 2 replaces the lossy floating scene time with the exact rational
/// clock identity `(frames_elapsed, fps)`. It still embeds a
/// [`SNAPSHOT_SCHEMA`] document as its final field.
pub const SCENE_STATE_SCHEMA: Schema = Schema::new(*b"FMNA", 2, 2, 0);

/// Errors from snapshot decode.
#[derive(Debug, Clone, PartialEq)]
pub enum PersistError {
    /// The container refused the bytes (magic/schema/version/checksum/
    /// size/EOF — every variant named by [`SerialError`]).
    Serial(SerialError),
    /// The payload parsed but violates the document's own invariants.
    Malformed(&'static str),
    /// A shape-tag discriminant this build does not know. A newer writer
    /// is the likely cause, and guessing would fabricate geometry.
    UnknownShapeTag(u8),
    /// The decoded-allocation budget (fm-vek.7, [`SnapshotLimits`]) refused
    /// a count *before* any reservation: the document asked for more
    /// destination or transient storage than the aggregate limit allows.
    AllocationLimit {
        /// The aggregate decoded-bytes ceiling in force.
        limit: usize,
        /// Aggregate bytes the decode would have charged.
        needed: usize,
        /// Which destination/transient structure tripped the budget.
        what: &'static str,
    },
}

impl std::fmt::Display for PersistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serial(e) => write!(f, "snapshot container refused: {e}"),
            Self::Malformed(what) => write!(f, "snapshot payload malformed: {what}"),
            Self::UnknownShapeTag(code) => {
                write!(f, "snapshot carries unknown shape-tag discriminant {code}")
            }
            Self::AllocationLimit {
                limit,
                needed,
                what,
            } => {
                write!(
                    f,
                    "snapshot decode budget exceeded by {what}: \
                     {needed} bytes charged against a {limit}-byte limit"
                )
            }
        }
    }
}

impl std::error::Error for PersistError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Serial(e) => Some(e),
            Self::Malformed(_) | Self::UnknownShapeTag(_) | Self::AllocationLimit { .. } => None,
        }
    }
}

impl From<SerialError> for PersistError {
    fn from(e: SerialError) -> Self {
        Self::Serial(e)
    }
}

/// Decode-time allocation budget for [`Snapshot::from_bytes`] (fm-vek.7).
///
/// The encoded container is already capped by [`Limits::DEFAULT`]
/// (256 MiB), but decoding *amplifies*: a five-byte empty-slot record
/// becomes a full slot-table entry, and every fixed-width count names
/// destination storage the decoder is asked to materialize. This budget
/// charges each destination and transient allocation against one explicit
/// aggregate ceiling *before* the reservation happens, so a hostile or
/// corrupt count is a typed refusal ([`PersistError::AllocationLimit`]),
/// never a multi-GiB speculative allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotLimits {
    /// Maximum aggregate bytes of decoded destination + transient storage
    /// one snapshot decode may charge.
    pub max_total_decoded_bytes: usize,
}

impl SnapshotLimits {
    /// The default budget: four times the encoded container cap. Record
    /// payloads (the dominant term in any real scene) decode at
    /// essentially 1:1 against their encoding, so 4x leaves every
    /// runtime-reachable arena — including large sparse ones — far under
    /// the ceiling, while a `u32::MAX` slot count or a five-byte-per-slot
    /// empty-slot bomb trips it orders of magnitude before the allocation
    /// it named.
    pub const DEFAULT: Self = Self {
        max_total_decoded_bytes: 4 * Limits::DEFAULT.max_total,
    };
}

impl Default for SnapshotLimits {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The running account of decoded destination + transient bytes: every
/// reservation charges here *before* it happens.
struct DecodeBudget {
    limit: usize,
    charged: usize,
}

impl DecodeBudget {
    fn new(limit: usize) -> Self {
        Self { limit, charged: 0 }
    }

    fn charge(&mut self, bytes: usize, what: &'static str) -> Result<(), PersistError> {
        let charged = self.charged.saturating_add(bytes);
        if charged > self.limit {
            return Err(PersistError::AllocationLimit {
                limit: self.limit,
                needed: charged,
                what,
            });
        }
        self.charged = charged;
        Ok(())
    }

    /// Reserve exactly `additional` elements, the budget already charged;
    /// an allocator refusal surfaces as the same typed budget error rather
    /// than an abort.
    fn reserve<T>(
        &self,
        vec: &mut Vec<T>,
        additional: usize,
        what: &'static str,
    ) -> Result<(), PersistError> {
        vec.try_reserve_exact(additional)
            .map_err(|_| PersistError::AllocationLimit {
                limit: self.limit,
                needed: self.charged,
                what,
            })
    }
}

/// Preflight one fixed-width count against *both* feasibility channels
/// (fm-vek.7): the encoded bytes it would take to actually carry that
/// many items, and the decoded-storage budget for the destination vector.
/// Runs before any reservation or iteration over the count.
fn preflight_count(
    r: &Reader<'_>,
    budget: &mut DecodeBudget,
    count: usize,
    encoded_each: usize,
    decoded_each: usize,
    what: &'static str,
) -> Result<(), PersistError> {
    let need = count.saturating_mul(encoded_each);
    if need > r.remaining() {
        return Err(PersistError::Serial(SerialError::UnexpectedEof {
            need,
            remaining: r.remaining(),
        }));
    }
    budget.charge(count.saturating_mul(decoded_each), what)
}

/// An updater's serializable kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdaterKindTag {
    /// `f(stage, mob)`.
    NonDt,
    /// `f(stage, mob, dt)`.
    Dt,
}

/// The per-slot updater identities a durable snapshot records — the
/// §13.4 vocabulary the replay journal validates against when re-binding
/// callables. Slots appear in arena order; slots without updaters are
/// omitted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UpdaterManifest {
    /// `(slot index, [(updater id, kind)])` in arena order.
    pub entries: Vec<(u32, Vec<(u64, UpdaterKindTag)>)>,
}

/// One updater identity resolved from durable slot coordinates into a live
/// Stage handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdaterIdentity {
    /// Mobject whose updater list contained this identity.
    pub mob: Mob,
    /// Original removal token.
    pub id: UpdaterId,
    /// Callable shape the replay registry must provide.
    pub kind: UpdaterKindTag,
}

impl UpdaterManifest {
    /// Resolve durable slot identities against the Stage used for decode.
    ///
    /// Results retain arena order and per-mobject updater execution order,
    /// ready for a replay owner's callback registry. The returned
    /// [`UpdaterId`] values are the original removal identities, not newly
    /// allocated substitutes.
    ///
    /// # Errors
    /// [`PersistError::Malformed`] if a manifest names a slot that is not live
    /// in `stage`, is not in canonical arena order, or carries a reserved or
    /// duplicate identity.
    pub fn identities(&self, stage: &Stage) -> Result<Vec<UpdaterIdentity>, PersistError> {
        let mut identities = Vec::new();
        let mut previous_slot = None;
        for (slot, updaters) in &self.entries {
            if previous_slot.is_some_and(|previous| previous >= *slot) {
                return Err(PersistError::Malformed(
                    "updater manifest slots are not strictly increasing",
                ));
            }
            if updaters.is_empty() {
                return Err(PersistError::Malformed(
                    "updater manifest contains an empty slot",
                ));
            }
            let mob = stage.mob_at_slot(*slot).ok_or(PersistError::Malformed(
                "updater manifest target is not live",
            ))?;
            let mut seen = Vec::with_capacity(updaters.len());
            for &(raw, kind) in updaters {
                if raw == 0 {
                    return Err(PersistError::Malformed(
                        "updater manifest carries the reserved zero identity",
                    ));
                }
                if seen.contains(&raw) {
                    return Err(PersistError::Malformed(
                        "updater manifest repeats an identity on one mobject",
                    ));
                }
                seen.push(raw);
                identities.push(UpdaterIdentity {
                    mob,
                    id: UpdaterId::from_raw(raw),
                    kind,
                });
            }
            previous_slot = Some(*slot);
        }
        Ok(identities)
    }
}

/// What [`Snapshot::from_bytes`] yields: the restorable snapshot (no
/// callables) plus the updater identities that were attached when the
/// bytes were written.
pub struct DecodedSnapshot {
    /// The arena state; feed it to [`Stage::restore`].
    pub snapshot: Snapshot,
    /// The updater identities (see the module docs' honesty clause).
    pub updaters: UpdaterManifest,
}

// ------------------------------------------------------------- encoding

fn put_mob(w: &mut Writer, mob: Mob) {
    let (index, generation) = mob.parts();
    w.put_u32(index).put_u32(generation);
}

fn put_mob_opt(w: &mut Writer, mob: Option<Mob>) {
    match mob {
        Some(m) => {
            w.put_bool(true);
            put_mob(w, m);
        }
        None => {
            w.put_bool(false);
        }
    }
}

fn count_u16(value: usize) -> Result<u16, SerialError> {
    u16::try_from(value).map_err(|_| SerialError::SizeLimit {
        limit: usize::from(u16::MAX),
        needed: value,
    })
}

fn count_u32(value: usize) -> Result<u32, SerialError> {
    u32::try_from(value).map_err(|_| SerialError::SizeLimit {
        limit: usize::try_from(u32::MAX).unwrap_or(usize::MAX),
        needed: value,
    })
}

fn count_u64(value: usize) -> Result<u64, SerialError> {
    u64::try_from(value).map_err(|_| SerialError::SizeLimit {
        limit: usize::try_from(u64::MAX).unwrap_or(usize::MAX),
        needed: value,
    })
}

fn put_render_primitive(w: &mut Writer, primitive: RenderPrimitive) -> Result<(), SerialError> {
    match primitive {
        RenderPrimitive::Vector => {
            w.put_u8(0);
        }
        RenderPrimitive::SurfaceGrid { resolution } => {
            w.put_u8(1)
                .put_u64(count_u64(resolution.0)?)
                .put_u64(count_u64(resolution.1)?);
        }
        RenderPrimitive::TriangleMesh => {
            w.put_u8(2);
        }
        RenderPrimitive::DotCloud => {
            w.put_u8(3);
        }
        RenderPrimitive::ImageQuad => {
            w.put_u8(4);
        }
    }
    Ok(())
}

fn get_render_primitive(r: &mut Reader<'_>) -> Result<RenderPrimitive, PersistError> {
    match r.get_u8()? {
        0 => Ok(RenderPrimitive::Vector),
        1 => Ok(RenderPrimitive::SurfaceGrid {
            resolution: (
                usize::try_from(r.get_u64()?)
                    .map_err(|_| PersistError::Malformed("surface u resolution overflows"))?,
                usize::try_from(r.get_u64()?)
                    .map_err(|_| PersistError::Malformed("surface v resolution overflows"))?,
            ),
        }),
        2 => Ok(RenderPrimitive::TriangleMesh),
        3 => Ok(RenderPrimitive::DotCloud),
        4 => Ok(RenderPrimitive::ImageQuad),
        _ => Err(PersistError::Malformed("unknown render primitive")),
    }
}

fn put_image_resource(w: &mut Writer, image: &ImageResource) {
    w.put_u32(image.width()).put_u32(image.height());
    match image.color_space() {
        ImageColorSpace::Srgb => {
            w.put_u8(0);
        }
        ImageColorSpace::Linear => {
            w.put_u8(1);
        }
        ImageColorSpace::Gamma(gamma) => {
            w.put_u8(2).put_u32(gamma);
        }
    }
    let put_wrap = |w: &mut Writer, wrap: ImageWrap| {
        w.put_u8(match wrap {
            ImageWrap::Repeat => 0,
            ImageWrap::ClampToEdge => 1,
            ImageWrap::MirroredRepeat => 2,
        });
    };
    put_wrap(w, image.sampler().wrap_u);
    put_wrap(w, image.sampler().wrap_v);
    w.put_digest(&image.content_digest())
        .put_bytes(image.pixels());
}

fn get_image_resource(
    r: &mut Reader<'_>,
    budget: &mut DecodeBudget,
) -> Result<ImageResource, PersistError> {
    let width = r.get_u32()?;
    let height = r.get_u32()?;
    let color_space = match r.get_u8()? {
        0 => ImageColorSpace::Srgb,
        1 => ImageColorSpace::Linear,
        2 => ImageColorSpace::Gamma(r.get_u32()?),
        _ => return Err(PersistError::Malformed("unknown image color space")),
    };
    let get_wrap = |r: &mut Reader<'_>| -> Result<ImageWrap, PersistError> {
        match r.get_u8()? {
            0 => Ok(ImageWrap::Repeat),
            1 => Ok(ImageWrap::ClampToEdge),
            2 => Ok(ImageWrap::MirroredRepeat),
            _ => Err(PersistError::Malformed("unknown image wrap policy")),
        }
    };
    let sampler = ImageSampler {
        wrap_u: get_wrap(r)?,
        wrap_v: get_wrap(r)?,
    };
    let claimed_digest = r.get_digest()?;
    let encoded = r.get_bytes()?;
    budget.charge(encoded.len().saturating_mul(2), "image pixel resource")?;
    let mut pixels = Vec::new();
    budget.reserve(&mut pixels, encoded.len(), "image pixel resource")?;
    pixels.extend_from_slice(encoded);
    let image = ImageResource::rgba8(width, height, pixels, color_space, sampler)
        .map_err(|_| PersistError::Malformed("invalid image resource layout"))?;
    if image.content_digest() != claimed_digest {
        return Err(PersistError::Malformed("image content digest mismatch"));
    }
    Ok(image)
}

fn put_buffer(w: &mut Writer, buffer: &RecordBuffer) -> Result<(), SerialError> {
    let schema = buffer.schema();
    let fields = schema.fields();
    w.put_u16(count_u16(fields.len())?);
    for field in fields {
        w.put_str(&field.name).put_u16(count_u16(field.width)?);
    }
    let put_names = |w: &mut Writer, names: &[String]| -> Result<(), SerialError> {
        w.put_u16(count_u16(names.len())?);
        for name in names {
            w.put_str(name);
        }
        Ok(())
    };
    put_names(w, schema.aligned_keys())?;
    put_names(w, schema.pointlike_keys())?;
    w.put_u32(count_u32(buffer.len())?);
    for field in fields {
        if let Some(column) = buffer.read_column(&field.name) {
            for value in column {
                w.put_f32(value);
            }
        }
    }
    let locked = buffer.locked_keys();
    w.put_u16(count_u16(locked.len())?);
    for name in locked {
        w.put_str(name);
    }
    // Minor 1.7: the `_data_defaults` phantom record. Appended, never
    // interleaved — a pre-1.7 reader of this stream refuses under Strict
    // (NewerMinor), and this reader defaults older streams to no row.
    match buffer.defaults_row() {
        Some(row) => {
            w.put_bool(true);
            for &lane in row {
                w.put_f32(lane);
            }
        }
        None => {
            w.put_bool(false);
        }
    }
    Ok(())
}

fn put_uniforms(w: &mut Writer, u: &Uniforms) {
    w.put_f64(u.is_fixed_in_frame);
    for v in u.shading {
        w.put_f64(v);
    }
    for plane in &u.clip_planes {
        for &v in plane {
            w.put_f64(v);
        }
    }
    w.put_f64(u.anti_alias_width);
    w.put_u8(match u.joint_type {
        JointType::NoJoint => 0,
        JointType::Auto => 1,
        JointType::Bevel => 2,
        JointType::Miter => 3,
    });
    w.put_bool(u.flat_stroke)
        .put_bool(u.scale_stroke_with_zoom)
        .put_bool(u.stroke_behind)
        .put_bool(u.depth_test)
        .put_bool(u.use_winding_fill);
}

fn put_entry(w: &mut Writer, entry: &SnapshotEntry) -> Result<(), SerialError> {
    put_buffer(w, &entry.buffer)?;
    w.put_u32(count_u32(entry.submobjects.len())?);
    for &m in &entry.submobjects {
        put_mob(w, m);
    }
    w.put_u32(count_u32(entry.parents.len())?);
    for &m in &entry.parents {
        put_mob(w, m);
    }
    // Updaters: identity + kind only — the honesty clause.
    w.put_u32(count_u32(entry.updaters.len())?);
    for slot in &entry.updaters {
        w.put_u64(slot.id.raw());
        w.put_u8(match slot.func {
            UpdaterFn::NonDt(_) => 0,
            UpdaterFn::Dt(_) => 1,
        });
    }
    w.put_bool(entry.updating_suspended)
        .put_bool(entry.is_animating);
    match entry.tracker {
        Some(t) => {
            w.put_bool(true);
            w.put_u8(match t.kind {
                crate::dynamics::TrackerKind::Plain => 0,
                crate::dynamics::TrackerKind::Exponential => 1,
                crate::dynamics::TrackerKind::Complex => 2,
            });
            w.put_f64(t.lanes[0]).put_f64(t.lanes[1]);
        }
        None => {
            w.put_bool(false);
        }
    }
    put_mob_opt(w, entry.target);
    put_mob_opt(w, entry.saved_state);
    w.put_u64(count_u64(entry.pins)?)
        .put_bool(entry.pending_delete);
    put_uniforms(w, &entry.uniforms);
    put_shape(w, &entry.shape)?;
    // Minor 1.2 appends the §8.5 scene-sort key. Appended, never
    // interleaved: a 1.1 reader stops at the shape tag and is still
    // correct, and a 1.2 reader of a 1.1 stream defaults it to zero (the
    // Reference's default) — the §6.7 additive-minor rule.
    w.put_i32(entry.z_index);
    Ok(())
}

/// The semantic shape tag (§10.8), added in schema minor 1.1.
///
/// The tag carries durable class configuration — `Line`'s `path_arc`
/// outlives every transform — so dropping it on a round trip would change
/// meaning, not just performance. Encoded as a discriminant plus its
/// payload, followed by whether its geometry hint is live (`0` for live,
/// `u64::MAX` for absent). The process-local RecordBuffer revision number is
/// deliberately not durable semantic state; decode rebinds a live marker to
/// the reconstructed buffer's current `point` revision.
fn put_shape(w: &mut Writer, slot: &ShapeSlot) -> Result<(), SerialError> {
    let put_point = |w: &mut Writer, p: Vec3| {
        w.put_f64(p[0]).put_f64(p[1]).put_f64(p[2]);
    };
    match slot.tag {
        ShapeTag::General => {
            w.put_u8(0);
        }
        ShapeTag::Line {
            start,
            end,
            path_arc,
            buff,
        } => {
            w.put_u8(1);
            put_point(w, start);
            put_point(w, end);
            w.put_f64(path_arc).put_f64(buff);
        }
        ShapeTag::Polyline { vertices, closed } => {
            w.put_u8(2);
            w.put_u64(count_u64(vertices)?).put_bool(closed);
        }
        ShapeTag::Arc {
            center,
            radius,
            start_angle,
            angle,
        } => {
            w.put_u8(3);
            put_point(w, center);
            w.put_f64(radius).put_f64(start_angle).put_f64(angle);
        }
        ShapeTag::Circle { center, radius } => {
            w.put_u8(4);
            put_point(w, center);
            w.put_f64(radius);
        }
        ShapeTag::Dot { center, radius } => {
            w.put_u8(5);
            put_point(w, center);
            w.put_f64(radius);
        }
        ShapeTag::Rect {
            center,
            width,
            height,
        } => {
            w.put_u8(6);
            put_point(w, center);
            w.put_f64(width).put_f64(height);
        }
        ShapeTag::RoundedRect {
            center,
            width,
            height,
            corner_radius,
        } => {
            w.put_u8(7);
            put_point(w, center);
            w.put_f64(width).put_f64(height).put_f64(corner_radius);
        }
    }
    w.put_u64(if slot.point_revision.is_some() {
        0
    } else {
        u64::MAX
    });
    Ok(())
}

fn get_shape(r: &mut Reader<'_>) -> Result<ShapeSlot, PersistError> {
    let point = |r: &mut Reader<'_>| -> Result<Vec3, PersistError> {
        Ok([r.get_f64()?, r.get_f64()?, r.get_f64()?])
    };
    let tag = match r.get_u8()? {
        0 => ShapeTag::General,
        1 => ShapeTag::Line {
            start: point(r)?,
            end: point(r)?,
            path_arc: r.get_f64()?,
            buff: r.get_f64()?,
        },
        2 => ShapeTag::Polyline {
            vertices: usize::try_from(r.get_u64()?)
                .map_err(|_| PersistError::Malformed("polyline vertex count overflows"))?,
            closed: r.get_bool()?,
        },
        3 => ShapeTag::Arc {
            center: point(r)?,
            radius: r.get_f64()?,
            start_angle: r.get_f64()?,
            angle: r.get_f64()?,
        },
        4 => ShapeTag::Circle {
            center: point(r)?,
            radius: r.get_f64()?,
        },
        5 => ShapeTag::Dot {
            center: point(r)?,
            radius: r.get_f64()?,
        },
        6 => ShapeTag::Rect {
            center: point(r)?,
            width: r.get_f64()?,
            height: r.get_f64()?,
        },
        7 => ShapeTag::RoundedRect {
            center: point(r)?,
            width: r.get_f64()?,
            height: r.get_f64()?,
            corner_radius: r.get_f64()?,
        },
        other => return Err(PersistError::UnknownShapeTag(other)),
    };
    let revision = r.get_u64()?;
    Ok(ShapeSlot {
        tag,
        point_revision: (revision != u64::MAX).then_some(revision),
    })
}

fn preflight_record_payload(
    r: &Reader<'_>,
    len: usize,
    stride: usize,
    budget: &mut DecodeBudget,
) -> Result<(), PersistError> {
    let needed = len
        .checked_mul(stride)
        .and_then(|lanes| lanes.checked_mul(std::mem::size_of::<f32>()))
        .ok_or(PersistError::Serial(SerialError::SizeLimit {
            limit: Limits::DEFAULT.max_total,
            needed: usize::MAX,
        }))?;
    if needed > Limits::DEFAULT.max_total {
        return Err(PersistError::Serial(SerialError::SizeLimit {
            limit: Limits::DEFAULT.max_total,
            needed,
        }));
    }
    if needed > r.remaining() {
        return Err(PersistError::Serial(SerialError::UnexpectedEof {
            need: needed,
            remaining: r.remaining(),
        }));
    }
    // The destination RecordBuffer's cells decode at 1:1 against the
    // payload just proved feasible; charge them before construction.
    budget.charge(needed, "record buffer cells")
}

fn live_snapshot_entry(slots: &[(u32, Option<SnapshotEntry>)], mob: Mob) -> Option<&SnapshotEntry> {
    let (index, generation) = mob.parts();
    let (stored_generation, entry) = slots.get(index as usize)?;
    if *stored_generation != generation {
        return None;
    }
    entry.as_ref()
}

fn links_are_unique(links: &[Mob]) -> bool {
    let mut identities: Vec<u64> = links.iter().map(|mob| mob.bits()).collect();
    identities.sort_unstable();
    identities.windows(2).all(|pair| pair[0] != pair[1])
}

/// Validate the arena invariants that every public [`Stage`] mutation
/// preserves before the decoded state can reach [`Stage::restore`].
///
/// This pass is deliberately iterative: persisted family depth is input, not
/// a reason to consume the process stack. Its auxiliary storage is linear in
/// the already-decoded slot and edge inventories.
fn validate_decoded_arena(
    snapshot: &Snapshot,
    budget: &mut DecodeBudget,
) -> Result<(), PersistError> {
    // The validation pass's transient storage (the free-slot bitmap, the
    // indegree table, the traversal stack) is linear in the slot count;
    // charge it like every other destination structure (fm-vek.7).
    budget.charge(
        snapshot
            .slots
            .len()
            .saturating_mul(1 + std::mem::size_of::<u32>() + std::mem::size_of::<usize>()),
        "arena validation indexes",
    )?;
    let mut free_seen = vec![false; snapshot.slots.len()];
    for &raw_index in &snapshot.free {
        let index = raw_index as usize;
        let Some((_, entry)) = snapshot.slots.get(index) else {
            return Err(PersistError::Malformed("free slot index is out of range"));
        };
        if entry.is_some() {
            return Err(PersistError::Malformed(
                "free slot ledger names a live slot",
            ));
        }
        if std::mem::replace(&mut free_seen[index], true) {
            return Err(PersistError::Malformed("free slot ledger repeats an index"));
        }
    }
    if snapshot
        .slots
        .iter()
        .enumerate()
        .any(|(index, (_, entry))| entry.is_none() && !free_seen[index])
    {
        return Err(PersistError::Malformed(
            "free slot ledger omits an empty slot",
        ));
    }

    if snapshot
        .roots
        .iter()
        .any(|&root| live_snapshot_entry(&snapshot.slots, root).is_none())
    {
        return Err(PersistError::Malformed("scene root is not live"));
    }

    // Once liveness has proved each handle's generation, slot-index pairs
    // uniquely identify edges. Keeping the lookup in `(u32, u32)` form uses
    // exactly the same eight bytes as one serialized handle rather than
    // doubling auxiliary storage with two `u64` identities.
    let mut reverse_edges: Vec<(u32, u32)> = Vec::new();
    let mut child_edge_count = 0_usize;
    let mut live_count = 0_usize;
    for (slot_index, (_, entry)) in snapshot.slots.iter().enumerate() {
        let Some(entry) = entry else {
            continue;
        };
        live_count += 1;
        if entry.pending_delete && entry.pins == 0 {
            return Err(PersistError::Malformed("pending delete has no pins"));
        }
        if !links_are_unique(&entry.submobjects) || !links_are_unique(&entry.parents) {
            return Err(PersistError::Malformed("family edge is duplicated"));
        }
        if entry
            .submobjects
            .iter()
            .chain(&entry.parents)
            .any(|&mob| live_snapshot_entry(&snapshot.slots, mob).is_none())
        {
            return Err(PersistError::Malformed("family link is not live"));
        }
        // The reverse-edge index is linear in the parent-link inventory;
        // charge and reserve it fallibly before extending (fm-vek.7).
        budget.charge(
            entry
                .parents
                .len()
                .saturating_mul(std::mem::size_of::<(u32, u32)>()),
            "family reverse-edge index",
        )?;
        budget.reserve(
            &mut reverse_edges,
            entry.parents.len(),
            "family reverse-edge index",
        )?;
        child_edge_count = child_edge_count
            .checked_add(entry.submobjects.len())
            .ok_or(PersistError::Malformed("family edge count overflows"))?;
        let slot_index = u32::try_from(slot_index)
            .map_err(|_| PersistError::Malformed("arena slot count exceeds format"))?;
        reverse_edges.extend(
            entry
                .parents
                .iter()
                .map(|parent| (parent.parts().0, slot_index)),
        );
    }
    if child_edge_count != reverse_edges.len() {
        return Err(PersistError::Malformed(
            "family parent-child links are asymmetric",
        ));
    }
    reverse_edges.sort_unstable();
    for (slot_index, (_, entry)) in snapshot.slots.iter().enumerate() {
        let Some(entry) = entry else {
            continue;
        };
        let slot_index = u32::try_from(slot_index)
            .map_err(|_| PersistError::Malformed("arena slot count exceeds format"))?;
        if entry.submobjects.iter().any(|child| {
            reverse_edges
                .binary_search(&(slot_index, child.parts().0))
                .is_err()
        }) {
            return Err(PersistError::Malformed(
                "family parent-child links are asymmetric",
            ));
        }
    }

    let mut indegrees = vec![0_u32; snapshot.slots.len()];
    let mut ready = Vec::new();
    for (index, (_, entry)) in snapshot.slots.iter().enumerate() {
        let Some(entry) = entry else {
            continue;
        };
        indegrees[index] = u32::try_from(entry.parents.len())
            .map_err(|_| PersistError::Malformed("family parent count exceeds format"))?;
        if indegrees[index] == 0 {
            ready.push(index);
        }
    }
    let mut visited = 0_usize;
    while let Some(index) = ready.pop() {
        visited += 1;
        let entry = snapshot.slots[index]
            .1
            .as_ref()
            .ok_or(PersistError::Malformed(
                "family traversal reached a free slot",
            ))?;
        for child in &entry.submobjects {
            let (child_index, _) = child.parts();
            let degree = &mut indegrees[child_index as usize];
            *degree = degree.checked_sub(1).ok_or(PersistError::Malformed(
                "family parent-child links are asymmetric",
            ))?;
            if *degree == 0 {
                ready.push(child_index as usize);
            }
        }
    }
    if visited != live_count {
        return Err(PersistError::Malformed("family graph contains a cycle"));
    }
    Ok(())
}

impl Snapshot {
    /// Serialize into the versioned canonical container.
    ///
    /// # Errors
    /// [`SerialError::SizeLimit`] when the state exceeds
    /// [`Limits::DEFAULT`] or one of the format's fixed-width count fields.
    pub fn to_bytes(&self) -> Result<Vec<u8>, SerialError> {
        let mut w = Writer::new(SNAPSHOT_SCHEMA);
        w.put_u32(count_u32(self.slots.len())?);
        for (generation, entry) in &self.slots {
            w.put_u32(*generation);
            match entry {
                Some(e) => {
                    w.put_bool(true);
                    put_entry(&mut w, e)?;
                }
                None => {
                    w.put_bool(false);
                }
            }
        }
        w.put_u32(count_u32(self.free.len())?);
        for &index in &self.free {
            w.put_u32(index);
        }
        w.put_u32(count_u32(self.roots.len())?);
        for &root in &self.roots {
            put_mob(&mut w, root);
        }
        w.put_u64(self.next_updater_id);
        // Minor 1.4: appended after the complete 1.3 payload so an older
        // compatible reader can stop before it. Runtime revision counters are
        // intentionally absent: canonical bytes describe state, not the edit
        // sequence that produced it.
        for (_, entry) in &self.slots {
            match entry {
                Some(entry) => {
                    w.put_bool(true);
                    for coefficient in entry.placement.coefficients() {
                        w.put_f64(coefficient);
                    }
                }
                None => {
                    w.put_bool(false);
                }
            }
        }
        // Minor 1.5: renderer program/topology identity is semantic state,
        // not a cache hint. Keep it in a liveness-parallel table so the 1.4
        // payload remains an exact prefix.
        for (_, entry) in &self.slots {
            match entry {
                Some(entry) => {
                    w.put_bool(true);
                    put_render_primitive(&mut w, entry.render_primitive)?;
                }
                None => {
                    w.put_bool(false);
                }
            }
        }
        // Minor 1.6: immutable image resources are semantic state. Their
        // descriptor and content digest precede the bytes so corruption and
        // descriptor drift are refused during decode.
        for (_, entry) in &self.slots {
            match entry {
                Some(entry) => {
                    w.put_bool(true);
                    match &entry.image {
                        Some(image) => {
                            w.put_bool(true);
                            put_image_resource(&mut w, image);
                        }
                        None => {
                            w.put_bool(false);
                        }
                    }
                }
                None => {
                    w.put_bool(false);
                }
            }
        }
        // Minor 1.8: a dark texture is part of the same immutable material,
        // not an ambient file path or a renderer-only cache. Preserve the old
        // primary-image table verbatim and append a liveness-parallel table.
        for (_, entry) in &self.slots {
            match entry {
                Some(entry) => {
                    w.put_bool(true);
                    let dark = entry.image.as_ref().and_then(ImageResource::dark_image);
                    w.put_bool(dark.is_some());
                    if let Some(dark) = dark {
                        put_image_resource(&mut w, dark);
                    }
                }
                None => {
                    w.put_bool(false);
                }
            }
        }
        w.finish()
    }

    /// The snapshot's content address: SHA-256 of its canonical bytes —
    /// what the replay journal compares at barriers.
    ///
    /// # Errors
    /// As [`Snapshot::to_bytes`].
    pub fn content_hash(&self) -> Result<Digest, SerialError> {
        Ok(sha256(&self.to_bytes()?))
    }

    /// Decode a durable snapshot, re-binding every handle to `stage`'s
    /// mint. Feed the result to [`Stage::restore`] on that same stage.
    ///
    /// # Errors
    /// [`PersistError::Serial`] (container), [`PersistError::Malformed`]
    /// (payload invariants), [`PersistError::AllocationLimit`] (the
    /// [`SnapshotLimits::DEFAULT`] decoded-allocation budget).
    pub fn from_bytes(bytes: &[u8], stage: &Stage) -> Result<DecodedSnapshot, PersistError> {
        Self::from_bytes_with_limits(bytes, stage, SnapshotLimits::DEFAULT)
    }

    /// [`Snapshot::from_bytes`] under an explicit decoded-allocation
    /// budget (fm-vek.7): every fixed-width count is preflighted against
    /// both the remaining encoded bytes and `limits` before the
    /// destination storage it names is reserved.
    ///
    /// # Errors
    /// As [`Snapshot::from_bytes`].
    pub fn from_bytes_with_limits(
        bytes: &[u8],
        stage: &Stage,
        limits: SnapshotLimits,
    ) -> Result<DecodedSnapshot, PersistError> {
        let stage_id = stage.stage_id();
        let mut budget = DecodeBudget::new(limits.max_total_decoded_bytes);
        let mut r = Reader::open(
            bytes,
            SNAPSHOT_SCHEMA,
            Limits::DEFAULT,
            UnknownPolicy::Strict,
        )?;
        let get_mob = |r: &mut Reader<'_>| -> Result<Mob, PersistError> {
            let index = r.get_u32()?;
            let generation = r.get_u32()?;
            Ok(Mob::from_parts(stage_id, index, generation))
        };
        let get_mob_opt = |r: &mut Reader<'_>| -> Result<Option<Mob>, PersistError> {
            if r.get_bool()? {
                let index = r.get_u32()?;
                let generation = r.get_u32()?;
                Ok(Some(Mob::from_parts(stage_id, index, generation)))
            } else {
                Ok(None)
            }
        };

        let slot_count = r.get_u32()? as usize;
        // Each slot costs at least five encoded bytes (generation + live
        // flag); a count the payload cannot carry is refused here, before
        // the slot table it names is reserved (fm-vek.7).
        preflight_count(
            &r,
            &mut budget,
            slot_count,
            5,
            std::mem::size_of::<(u32, Option<SnapshotEntry>)>(),
            "arena slot table",
        )?;
        let mut slots: Vec<(u32, Option<SnapshotEntry>)> = Vec::new();
        budget.reserve(&mut slots, slot_count, "arena slot table")?;
        let mut manifest = UpdaterManifest::default();
        for slot_index in 0..slot_count {
            let generation = r.get_u32()?;
            let entry = if r.get_bool()? {
                // --- buffer
                let n_fields = r.get_u16()? as usize;
                // At least ten encoded bytes each (u64 name-length prefix
                // + u16 width).
                preflight_count(
                    &r,
                    &mut budget,
                    n_fields,
                    10,
                    std::mem::size_of::<(String, usize)>(),
                    "record field table",
                )?;
                let mut fields: Vec<(String, usize)> = Vec::new();
                budget.reserve(&mut fields, n_fields, "record field table")?;
                for _ in 0..n_fields {
                    let name = r.get_str()?.to_owned();
                    budget.charge(name.len(), "record field names")?;
                    let width = r.get_u16()? as usize;
                    if width == 0 {
                        return Err(PersistError::Malformed("zero-width record field"));
                    }
                    fields.push((name, width));
                }
                let get_names = |r: &mut Reader<'_>,
                                 budget: &mut DecodeBudget,
                                 what: &'static str|
                 -> Result<Vec<String>, PersistError> {
                    let n = r.get_u16()? as usize;
                    // u64 length prefix per name.
                    preflight_count(r, budget, n, 8, std::mem::size_of::<String>(), what)?;
                    let mut names = Vec::new();
                    budget.reserve(&mut names, n, what)?;
                    for _ in 0..n {
                        let name = r.get_str()?.to_owned();
                        budget.charge(name.len(), what)?;
                        names.push(name);
                    }
                    Ok(names)
                };
                let aligned = get_names(&mut r, &mut budget, "aligned key names")?;
                let pointlike = get_names(&mut r, &mut budget, "pointlike key names")?;
                let len = r.get_u32()? as usize;
                let field_refs: Vec<(&str, usize)> =
                    fields.iter().map(|(n, w)| (n.as_str(), *w)).collect();
                let aligned_refs: Vec<&str> = aligned.iter().map(String::as_str).collect();
                let pointlike_refs: Vec<&str> = pointlike.iter().map(String::as_str).collect();
                let schema = RecordSchema::new(&field_refs, &aligned_refs, &pointlike_refs)
                    .map_err(|_| PersistError::Malformed("record schema stride overflows usize"))?;
                let stride = schema.stride();
                preflight_record_payload(&r, len, stride, &mut budget)?;
                let mut buffer = RecordBuffer::new(schema, len)
                    .map_err(|_| PersistError::Malformed("record buffer sizing overflows usize"))?;
                for (name, width) in &fields {
                    // `len * stride` lanes were just proved to fit the
                    // container cap, so this per-field product cannot wrap.
                    let lanes = len * width;
                    budget.charge(
                        lanes.saturating_mul(std::mem::size_of::<f32>()),
                        "record column staging",
                    )?;
                    let mut column = Vec::new();
                    budget.reserve(&mut column, lanes, "record column staging")?;
                    for _ in 0..lanes {
                        column.push(r.get_f32()?);
                    }
                    buffer.write_range(name, 0, &column);
                }
                let locked = get_names(&mut r, &mut budget, "locked key names")?;
                if !locked.is_empty() {
                    buffer.lock_data(locked.iter().map(String::as_str));
                }
                // Minor 1.7: the `_data_defaults` phantom record. Older
                // streams have no section; the reader defaults to no row,
                // exactly as the 1.1/1.2 branches do for their fields.
                if r.version().1 >= 7 && r.get_bool()? {
                    preflight_record_payload(&r, 1, stride, &mut budget)?;
                    budget.charge(
                        stride.saturating_mul(std::mem::size_of::<f32>()),
                        "record defaults staging",
                    )?;
                    let mut row = Vec::with_capacity(stride);
                    for _ in 0..stride {
                        row.push(r.get_f32()?);
                    }
                    buffer.restore_defaults_row(&row);
                }
                // --- graph + state
                let n_sub = r.get_u32()? as usize;
                preflight_count(
                    &r,
                    &mut budget,
                    n_sub,
                    8,
                    std::mem::size_of::<Mob>(),
                    "family submobject links",
                )?;
                let mut submobjects = Vec::new();
                budget.reserve(&mut submobjects, n_sub, "family submobject links")?;
                for _ in 0..n_sub {
                    submobjects.push(get_mob(&mut r)?);
                }
                let n_par = r.get_u32()? as usize;
                preflight_count(
                    &r,
                    &mut budget,
                    n_par,
                    8,
                    std::mem::size_of::<Mob>(),
                    "family parent links",
                )?;
                let mut parents = Vec::new();
                budget.reserve(&mut parents, n_par, "family parent links")?;
                for _ in 0..n_par {
                    parents.push(get_mob(&mut r)?);
                }
                let n_upd = r.get_u32()? as usize;
                // u64 identity + u8 kind per updater on the wire.
                preflight_count(
                    &r,
                    &mut budget,
                    n_upd,
                    9,
                    std::mem::size_of::<(u64, UpdaterKindTag)>(),
                    "updater identities",
                )?;
                let mut ids = Vec::new();
                budget.reserve(&mut ids, n_upd, "updater identities")?;
                for _ in 0..n_upd {
                    let id = r.get_u64()?;
                    if id == 0 {
                        return Err(PersistError::Malformed(
                            "updater manifest carries the reserved zero identity",
                        ));
                    }
                    if ids.iter().any(|(existing, _)| *existing == id) {
                        return Err(PersistError::Malformed(
                            "updater manifest repeats an identity on one mobject",
                        ));
                    }
                    let kind = match r.get_u8()? {
                        0 => UpdaterKindTag::NonDt,
                        1 => UpdaterKindTag::Dt,
                        _ => return Err(PersistError::Malformed("unknown updater kind")),
                    };
                    ids.push((id, kind));
                }
                if !ids.is_empty() {
                    #[allow(clippy::cast_possible_truncation)]
                    manifest.entries.push((slot_index as u32, ids));
                }
                let updating_suspended = r.get_bool()?;
                let is_animating = r.get_bool()?;
                let tracker = if r.get_bool()? {
                    let kind = match r.get_u8()? {
                        0 => crate::dynamics::TrackerKind::Plain,
                        1 => crate::dynamics::TrackerKind::Exponential,
                        2 => crate::dynamics::TrackerKind::Complex,
                        _ => return Err(PersistError::Malformed("unknown tracker kind")),
                    };
                    let lanes = [r.get_f64()?, r.get_f64()?];
                    Some(crate::dynamics::Tracker { kind, lanes })
                } else {
                    None
                };
                let target = get_mob_opt(&mut r)?;
                let saved_state = get_mob_opt(&mut r)?;
                let pins = usize::try_from(r.get_u64()?)
                    .map_err(|_| PersistError::Malformed("pin count overflows"))?;
                let pending_delete = r.get_bool()?;
                // --- uniforms (field order is the schema)
                let is_fixed_in_frame = r.get_f64()?;
                let mut shading = [0.0; 3];
                for lane in &mut shading {
                    *lane = r.get_f64()?;
                }
                let mut clip_planes = [[0.0; 4]; 4];
                for plane in &mut clip_planes {
                    for slot in plane {
                        *slot = r.get_f64()?;
                    }
                }
                let anti_alias_width = r.get_f64()?;
                let joint_type = JointType::from_code(f64::from(r.get_u8()?));
                let uniforms = Uniforms {
                    is_fixed_in_frame,
                    shading,
                    clip_planes,
                    anti_alias_width,
                    joint_type,
                    flat_stroke: r.get_bool()?,
                    scale_stroke_with_zoom: r.get_bool()?,
                    stroke_behind: r.get_bool()?,
                    depth_test: r.get_bool()?,
                    use_winding_fill: r.get_bool()?,
                };
                // Schema minor 1.1 appended the shape tag; a 1.0 stream
                // simply has no shape, which is what General means.
                let mut shape = if r.version().1 >= 1 {
                    get_shape(&mut r)?
                } else {
                    ShapeSlot::default()
                };
                if shape.point_revision.is_some() {
                    shape.point_revision = buffer.field_revision("point");
                }
                // Minor 1.2 appended the scene-sort key (§8.5).
                let z_index = if r.version().1 >= 2 { r.get_i32()? } else { 0 };
                Some(SnapshotEntry {
                    buffer,
                    placement: Placement::IDENTITY,
                    placement_revision: 0,
                    submobjects,
                    parents,
                    updaters: Vec::new(),       // callables never serialize
                    updater_states: Vec::new(), // replay owns executable state
                    updating_suspended,
                    is_animating,
                    tracker,
                    target,
                    saved_state,
                    pins,
                    pending_delete,
                    uniforms,
                    z_index,
                    shape,
                    render_primitive: RenderPrimitive::Vector,
                    image: None,
                    image_revision: 0,
                })
            } else {
                None
            };
            slots.push((generation, entry));
        }
        let n_free = r.get_u32()? as usize;
        preflight_count(
            &r,
            &mut budget,
            n_free,
            4,
            std::mem::size_of::<u32>(),
            "free slot ledger",
        )?;
        let mut free = Vec::new();
        budget.reserve(&mut free, n_free, "free slot ledger")?;
        for _ in 0..n_free {
            free.push(r.get_u32()?);
        }
        let n_roots = r.get_u32()? as usize;
        preflight_count(
            &r,
            &mut budget,
            n_roots,
            8,
            std::mem::size_of::<Mob>(),
            "scene root table",
        )?;
        let mut roots = Vec::new();
        budget.reserve(&mut roots, n_roots, "scene root table")?;
        for _ in 0..n_roots {
            roots.push(get_mob(&mut r)?);
        }
        let max_updater_id = manifest
            .entries
            .iter()
            .flat_map(|(_, updaters)| updaters.iter().map(|(id, _)| *id))
            .max();
        // Minor 1.3 made the cursor part of durable state. Deriving from the
        // active manifest for older streams is the only compatible choice,
        // but 1.3 also preserves ids of updaters removed before the barrier.
        let next_updater_id = if r.version().1 >= 3 {
            let next = r.get_u64()?;
            if next == 0 || max_updater_id.is_some_and(|id| id >= next) {
                return Err(PersistError::Malformed(
                    "updater id cursor does not follow active identities",
                ));
            }
            next
        } else {
            max_updater_id
                .unwrap_or(0)
                .checked_add(1)
                .ok_or(PersistError::Malformed("updater id space is exhausted"))?
                .max(1)
        };
        if r.version().1 >= 4 {
            for entry in &mut slots {
                let present = r.get_bool()?;
                match (present, &mut entry.1) {
                    (false, None) => {}
                    (true, Some(entry)) => {
                        let coefficients = [
                            r.get_f64()?,
                            r.get_f64()?,
                            r.get_f64()?,
                            r.get_f64()?,
                            r.get_f64()?,
                            r.get_f64()?,
                            r.get_f64()?,
                            r.get_f64()?,
                            r.get_f64()?,
                            r.get_f64()?,
                            r.get_f64()?,
                            r.get_f64()?,
                        ];
                        entry.placement = Placement::new(
                            [
                                [coefficients[0], coefficients[1], coefficients[2]],
                                [coefficients[4], coefficients[5], coefficients[6]],
                                [coefficients[8], coefficients[9], coefficients[10]],
                            ],
                            [coefficients[3], coefficients[7], coefficients[11]],
                        );
                    }
                    _ => {
                        return Err(PersistError::Malformed(
                            "placement table does not match arena liveness",
                        ));
                    }
                }
            }
        }
        if r.version().1 >= 5 {
            for entry in &mut slots {
                let present = r.get_bool()?;
                match (present, &mut entry.1) {
                    (false, None) => {}
                    (true, Some(entry)) => {
                        entry.render_primitive = get_render_primitive(&mut r)?;
                    }
                    _ => {
                        return Err(PersistError::Malformed(
                            "render-primitive table does not match arena liveness",
                        ));
                    }
                }
            }
        }
        if r.version().1 >= 6 {
            for entry in &mut slots {
                let present = r.get_bool()?;
                match (present, &mut entry.1) {
                    (false, None) => {}
                    (true, Some(entry)) => {
                        if r.get_bool()? {
                            entry.image = Some(get_image_resource(&mut r, &mut budget)?);
                            entry.image_revision = 1;
                        }
                    }
                    _ => {
                        return Err(PersistError::Malformed(
                            "image-resource table does not match arena liveness",
                        ));
                    }
                }
            }
        }
        if r.version().1 >= 8 {
            for (_, entry) in &mut slots {
                let present = r.get_bool()?;
                match (present, entry) {
                    (false, None) => {}
                    (true, Some(entry)) => {
                        if r.get_bool()? {
                            budget.charge(
                                std::mem::size_of::<ImageResource>(),
                                "dark image descriptor",
                            )?;
                            let dark = get_image_resource(&mut r, &mut budget)?;
                            let light = entry.image.take().ok_or(PersistError::Malformed(
                                "dark image requires a primary image resource",
                            ))?;
                            entry.image = Some(light.with_dark_image(dark).map_err(|_| {
                                PersistError::Malformed("invalid light/dark image pair")
                            })?);
                        }
                    }
                    _ => {
                        return Err(PersistError::Malformed(
                            "dark-image table does not match arena liveness",
                        ));
                    }
                }
            }
        }
        r.finish()?;
        let snapshot = Snapshot {
            stage_id,
            next_updater_id,
            slots,
            free,
            roots,
        };
        validate_decoded_arena(&snapshot, &mut budget)?;
        Ok(DecodedSnapshot {
            snapshot,
            updaters: manifest,
        })
    }
}

impl Stage {
    /// [`Stage::snapshot`] straight to canonical bytes.
    ///
    /// # Errors
    /// As [`Snapshot::to_bytes`].
    pub fn snapshot_bytes(&self) -> Result<Vec<u8>, SerialError> {
        self.snapshot().to_bytes()
    }
}

// ----------------------------------------------------------- SceneState

/// The §13.1 scene-scope state: exact rational-clock position, play count,
/// the one RNG's state words (fmn-core's export surface), and the arena
/// snapshot. The scene runtime (fm-5xm) captures and re-applies it; this
/// layer owns the bytes.
pub struct SceneState {
    /// Exact number of elapsed frames at capture.
    pub frames_elapsed: i64,
    /// Exact frame grid at capture.
    pub fps: u32,
    /// Completed `play()` count at capture.
    pub play_count: u64,
    /// `Pcg64Dxsm::state()`: `((state_hi, state_lo), (inc_hi, inc_lo))`.
    pub rng_state: ([u64; 2], [u64; 2]),
    /// The arena.
    pub snapshot: Snapshot,
}

/// A decoded scene state: the fields plus the snapshot's updater manifest.
pub struct DecodedSceneState {
    /// Exact number of elapsed frames at capture.
    pub frames_elapsed: i64,
    /// Exact frame grid at capture.
    pub fps: u32,
    /// Completed `play()` count at capture.
    pub play_count: u64,
    /// The RNG state words; [`DecodedSceneState::rng`] rebuilds the
    /// generator.
    pub rng_state: ([u64; 2], [u64; 2]),
    /// The arena, handles re-bound to the decoding stage.
    pub snapshot: Snapshot,
    /// The updater identities recorded at capture.
    pub updaters: UpdaterManifest,
}

impl DecodedSceneState {
    /// Rebuild the generator exactly where it was.
    #[must_use]
    pub fn rng(&self) -> Pcg64Dxsm {
        let (state, inc) = self.rng_state;
        Pcg64Dxsm::restore(state, inc)
    }
}

impl SceneState {
    /// Capture the scene-scope state from a stage, its exact clock position,
    /// a play counter, and the RNG.
    #[must_use]
    pub fn capture(
        stage: &Stage,
        frames_elapsed: i64,
        fps: u32,
        play_count: u64,
        rng: &Pcg64Dxsm,
    ) -> Self {
        Self {
            frames_elapsed,
            fps,
            play_count,
            rng_state: rng.state(),
            snapshot: stage.snapshot(),
        }
    }

    /// Serialize the envelope (exact clock fields, play count, RNG words,
    /// then the nested snapshot document as a length-prefixed field).
    ///
    /// # Errors
    /// As [`Snapshot::to_bytes`].
    pub fn to_bytes(&self) -> Result<Vec<u8>, SerialError> {
        let snapshot_bytes = self.snapshot.to_bytes()?;
        let mut w = Writer::new(SCENE_STATE_SCHEMA);
        w.put_i64(self.frames_elapsed)
            .put_u32(self.fps)
            .put_u64(self.play_count);
        let (state, inc) = self.rng_state;
        w.put_u64(state[0])
            .put_u64(state[1])
            .put_u64(inc[0])
            .put_u64(inc[1]);
        w.put_bytes(&snapshot_bytes);
        w.finish()
    }

    /// Decode an envelope, re-binding snapshot handles to `stage`.
    ///
    /// # Errors
    /// As [`Snapshot::from_bytes`].
    pub fn from_bytes(bytes: &[u8], stage: &Stage) -> Result<DecodedSceneState, PersistError> {
        let mut r = Reader::open(
            bytes,
            SCENE_STATE_SCHEMA,
            Limits::DEFAULT,
            UnknownPolicy::Strict,
        )?;
        let frames_elapsed = r.get_i64()?;
        let fps = r.get_u32()?;
        let play_count = r.get_u64()?;
        let state = [r.get_u64()?, r.get_u64()?];
        let inc = [r.get_u64()?, r.get_u64()?];
        // The nested snapshot document is borrowed straight out of the
        // envelope — no transit clone (fm-vek.7); its own decode re-opens
        // it as a container and budgets what it materializes.
        let snapshot_bytes = r.get_bytes()?;
        r.finish()?;
        let decoded = Snapshot::from_bytes(snapshot_bytes, stage)?;
        Ok(DecodedSceneState {
            frames_elapsed,
            fps,
            play_count,
            rng_state: (state, inc),
            snapshot: decoded.snapshot,
            updaters: decoded.updaters,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Mobject;

    fn entry_mut(snapshot: &mut Snapshot, mob: Mob) -> &mut SnapshotEntry {
        let (index, generation) = mob.parts();
        let (stored_generation, entry) = &mut snapshot.slots[index as usize];
        assert_eq!(*stored_generation, generation);
        entry.as_mut().expect("test handle must name a live entry")
    }

    fn assert_malformed(snapshot: &Snapshot, stage: &Stage, expected: &'static str) {
        let bytes = snapshot.to_bytes().unwrap();
        let error = Snapshot::from_bytes(&bytes, stage)
            .map(|_| ())
            .expect_err("the malformed canonical snapshot must be refused");
        assert_eq!(error, PersistError::Malformed(expected));
    }

    #[test]
    fn decode_refuses_impossible_lifetime_and_free_slot_ledgers() {
        let mut stage = Stage::new();
        let mob = stage.add(Mobject::new());

        let mut pending_without_pins = stage.snapshot();
        entry_mut(&mut pending_without_pins, mob).pending_delete = true;
        assert_malformed(&pending_without_pins, &stage, "pending delete has no pins");

        let mut out_of_range = stage.snapshot();
        out_of_range
            .free
            .push(u32::try_from(out_of_range.slots.len()).unwrap());
        assert_malformed(&out_of_range, &stage, "free slot index is out of range");

        let mut live_slot = stage.snapshot();
        live_slot.free.push(0);
        assert_malformed(&live_slot, &stage, "free slot ledger names a live slot");

        let free_mob = stage.add(Mobject::new());
        stage.delete(free_mob).unwrap();
        let mut duplicate = stage.snapshot();
        duplicate.free.push(duplicate.free[0]);
        assert_malformed(&duplicate, &stage, "free slot ledger repeats an index");

        let mut missing = stage.snapshot();
        missing.free.clear();
        assert_malformed(&missing, &stage, "free slot ledger omits an empty slot");
    }

    #[test]
    fn decode_refuses_impossible_roots_and_family_graphs() {
        let mut stage = Stage::new();
        let parent = stage.add(Mobject::new());
        let child = stage.add(Mobject::new());

        let mut stale_root = stage.snapshot();
        let (index, generation) = parent.parts();
        stale_root.roots.push(Mob::from_parts(
            stale_root.stage_id,
            index,
            generation.wrapping_add(1),
        ));
        assert_malformed(&stale_root, &stage, "scene root is not live");

        let mut stale_edge = stage.snapshot();
        let (index, generation) = child.parts();
        let stale_child = Mob::from_parts(stale_edge.stage_id, index, generation.wrapping_add(1));
        entry_mut(&mut stale_edge, parent)
            .submobjects
            .push(stale_child);
        assert_malformed(&stale_edge, &stage, "family link is not live");

        let mut duplicate_edge = stage.snapshot();
        entry_mut(&mut duplicate_edge, parent)
            .submobjects
            .extend([child, child]);
        entry_mut(&mut duplicate_edge, child).parents.push(parent);
        assert_malformed(&duplicate_edge, &stage, "family edge is duplicated");

        let mut asymmetric = stage.snapshot();
        entry_mut(&mut asymmetric, parent).submobjects.push(child);
        assert_malformed(
            &asymmetric,
            &stage,
            "family parent-child links are asymmetric",
        );

        let mut cyclic = stage.snapshot();
        entry_mut(&mut cyclic, parent).submobjects.push(child);
        entry_mut(&mut cyclic, parent).parents.push(child);
        entry_mut(&mut cyclic, child).submobjects.push(parent);
        entry_mut(&mut cyclic, child).parents.push(parent);
        assert_malformed(&cyclic, &stage, "family graph contains a cycle");
    }

    #[test]
    fn decode_accepts_runtime_reachable_lifetime_and_graph_states() {
        let mut stage = Stage::new();
        let root = stage.add(Mobject::new());
        let left = stage.add(Mobject::new());
        let right = stage.add(Mobject::new());
        let shared = stage.add(Mobject::new());
        stage.attach(root, left).unwrap();
        stage.attach(root, right).unwrap();
        stage.attach(left, shared).unwrap();
        stage.attach(right, shared).unwrap();
        stage.add_many_to_scene(&[root, root]).unwrap();

        stage.pin(left).unwrap();
        stage.delete(left).unwrap();

        stage.generate_target(right).unwrap();
        let target = stage.target(right).unwrap();
        stage.delete(target).unwrap();
        stage.save_state(right).unwrap();
        let saved = stage.saved_state(right).unwrap();
        stage.delete(saved).unwrap();

        let bytes = stage.snapshot_bytes().unwrap();
        let decoded = Snapshot::from_bytes(&bytes, &stage).unwrap();
        let mut snapshot = decoded.snapshot;
        assert_eq!(snapshot.roots, [root, root]);
        assert_eq!(snapshot.to_bytes().unwrap(), bytes);
        let left_entry = entry_mut(&mut snapshot, left);
        assert_eq!(left_entry.pins, 1);
        assert!(left_entry.pending_delete);
    }

    #[test]
    fn decode_validates_a_deep_family_without_recursion() {
        const DEPTH: usize = 4_096;

        let mut stage = Stage::new();
        let handles: Vec<Mob> = (0..DEPTH).map(|_| stage.add(Mobject::new())).collect();
        let mut snapshot = stage.snapshot();
        for edge in handles.windows(2) {
            entry_mut(&mut snapshot, edge[0]).submobjects.push(edge[1]);
            entry_mut(&mut snapshot, edge[1]).parents.push(edge[0]);
        }

        let bytes = snapshot.to_bytes().unwrap();
        let decoded = Snapshot::from_bytes(&bytes, &stage).unwrap();
        assert_eq!(decoded.snapshot.slots.len(), DEPTH);
    }
    #[test]
    fn defaults_row_round_trips_and_is_adopted_on_growth() {
        let mut stage = Stage::new();
        let mob = stage.add(Mobject::new());
        let field = stage.get(mob).unwrap().buffer.schema().fields()[0]
            .name
            .clone();
        let width = stage
            .get(mob)
            .unwrap()
            .buffer
            .schema()
            .field_width(&field)
            .expect("first schema field has a width");
        let values: Vec<f32> = (0..width).map(|i| 1.0 + 0.5 * i as f32).collect();
        stage
            .get_mut(mob)
            .unwrap()
            .buffer
            .write_default_record(&field, &values);
        assert!(stage.get(mob).unwrap().buffer.has_defaults());

        let bytes = stage.snapshot().to_bytes().unwrap();
        let mut decoded = Snapshot::from_bytes(&bytes, &stage).unwrap();
        let restored = entry_mut(&mut decoded.snapshot, mob);
        assert!(restored.buffer.has_defaults());
        let row = restored.buffer.defaults_row().unwrap();
        assert_eq!(&row[..values.len()], &values[..]);
        assert!(row[values.len()..].iter().all(|lane| *lane == 0.0));

        // The whole point of the row: growth from zero adopts it.
        let mut grown = restored.buffer.deep_clone();
        grown.resize(2).unwrap();
        let expected: Vec<f32> = values
            .iter()
            .copied()
            .chain(values.iter().copied())
            .collect();
        assert_eq!(grown.read_column(&field).unwrap(), expected);
    }

    #[test]
    fn buffers_without_defaults_round_trip_without_the_section() {
        let mut stage = Stage::new();
        let empty = stage.add(Mobject::new());
        let filled = stage.add(Mobject::new());
        let field = stage.get(filled).unwrap().buffer.schema().fields()[0]
            .name
            .clone();
        let width = stage
            .get(filled)
            .unwrap()
            .buffer
            .schema()
            .field_width(&field)
            .expect("first schema field has a width");
        stage
            .get_mut(filled)
            .unwrap()
            .buffer
            .resize(2)
            .expect("growth within allocation limits");
        let column = vec![0.25; 2 * width];
        stage
            .get_mut(filled)
            .unwrap()
            .buffer
            .write_range(&field, 0, &column);

        assert!(!stage.get(empty).unwrap().buffer.has_defaults());
        assert!(!stage.get(filled).unwrap().buffer.has_defaults());

        let bytes = stage.snapshot().to_bytes().unwrap();
        let mut decoded = Snapshot::from_bytes(&bytes, &stage).unwrap();
        assert!(
            !entry_mut(&mut decoded.snapshot, empty)
                .buffer
                .has_defaults()
        );
        assert!(
            !entry_mut(&mut decoded.snapshot, filled)
                .buffer
                .has_defaults()
        );
    }
}
