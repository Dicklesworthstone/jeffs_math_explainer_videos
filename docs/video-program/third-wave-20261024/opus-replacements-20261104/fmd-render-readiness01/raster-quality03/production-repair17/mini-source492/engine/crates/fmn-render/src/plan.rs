//! The retained render plan: lazy synchronization from Marionette (§10.8, §8.2).
//!
//! > … synchronized *lazily* from the RecordBuffer under §8.2's mirror rule …
//!
//! ## What "lazily" has to mean to be worth anything
//!
//! Not "recompute a bit less". The plan holds compiled outlines across frames
//! and, for a renderable whose declared axes have not moved, **never reads its
//! points at all**. That is the difference between a cache and a retained plan:
//! a cache still has to look at the data to discover it is unchanged, and
//! looking at the data is most of the cost — reading a glyph's points, hashing
//! them, splitting curves, measuring arc length.
//!
//! So the sync loop's inner question is a comparison of seven integers
//! ([`crate::revision`]), and everything expensive sits behind it. The cheap
//! things that genuinely must happen every frame — the painter-order instance
//! list — are rebuilt every frame, because an instance is an index, an affine
//! placement and two more indices, and pretending otherwise would buy nothing
//! and cost correctness.
//!
//! ## The observable
//!
//! [`SyncStats`] exists because "untouched resources never recompile" is a claim
//! about work *not* done, which is untestable unless the work counts itself.
//! `MirrorSet::materializations` and `CachedArcLength::rebuilds` are the same
//! idea one layer down, and the same reason.

use crate::hint::Hint;
use crate::revision::{Axis, Dependency, Revisions};
use crate::table::{
    ImageTable, Instance, Segment, Shape, ShapeTable, Style, StyleTable, TableError,
    check_row_count, compile_shape, shape_digest,
};
use fmn_core::types::Vec3;
use fmn_geom::{GeomError, quadpath::QuadPath};
use fmn_hash::{Digest, Sha256};
use fmn_mobject::{Mob, Placement, ProgramKind, RecordBuffer, Stage};
use std::collections::{HashMap, HashSet};

/// The axes a compiled outline depends on.
///
/// Geometry because it *is* the geometry, topology because a point count or a
/// family change reshapes it. Deliberately **not** style, order or camera: a
/// recolour, a `z_index` bump and a pan must all leave a compiled outline alone,
/// which is the §10.8 promise this constant makes checkable.
pub const SHAPE_AXES: [Axis; 2] = [Axis::Topology, Axis::Geometry];

/// The axes a style row depends on.
pub const STYLE_AXES: [Axis; 1] = [Axis::Style];

// Profile stations follow true arc lengths after an affine placement.
const PROFILE_STYLE_AXES: [Axis; 4] =
    [Axis::Topology, Axis::Geometry, Axis::Transform, Axis::Style];

/// Record fields the compiled outline actually observes today.
///
/// Keep this consumption list exact: a field-scoped view of user metadata must
/// not turn into object-wide recompilation merely because both share a record.
const SHAPE_VIEW_FIELDS: [&str; 1] = ["point"];

/// Record fields [`read_style`] observes.
const STYLE_VIEW_FIELDS: [&str; 4] = [
    "stroke_rgba",
    "stroke_width",
    "fill_rgba",
    "fill_border_width",
];

/// Admission limits for one retained-plan synchronization.
///
/// The instance default matches [`crate::bin::BinningLimits`]. The hard retained
/// geometry ceiling admits sixteen maximum-size Chisel paths, while the smaller
/// unreachable-row budgets reclaim mutation history at a named safe point long
/// before that ceiling. Callers with a deliberately larger, provisioned scene
/// can pass explicit limits through [`RenderPlan::sync_with_limits`], or declare
/// them on a renderer or as `fmn::RenderOptions::plan_limits`; the `u32` table
/// width remains absolute.
///
/// Compaction reclaims only rows already unreachable when a sync begins. A frame
/// that repaints every row straight after a compaction therefore holds the old
/// and new rows together: a scene that repaints needs twice its live knots, not
/// once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderPlanLimits {
    /// Maximum painter-ordered instances produced by one sync.
    pub max_instances: u64,
    /// Maximum input curves materialized for geometry rebuilds in one sync.
    pub max_input_curves: u64,
    /// Maximum segments retained across every interned outline.
    pub max_retained_segments: u64,
    /// Maximum distinct retained outlines.
    pub max_retained_shapes: u64,
    /// Maximum distinct retained style rows.
    pub max_retained_styles: u64,
    /// Maximum per-record stroke knots retained across distinct styles.
    pub max_retained_profile_knots: u64,
    /// Maximum compiled segment rows no live retained entry references.
    ///
    /// Exceeding this budget schedules a deterministic rebuild at the next sync
    /// safe point. The hard retained-row limits above remain absolute.
    pub max_unreachable_segments: u64,
    /// Maximum compiled shape rows no live retained entry references.
    pub max_unreachable_shapes: u64,
    /// Maximum style rows no live retained entry references.
    pub max_unreachable_styles: u64,
}

impl Default for RenderPlanLimits {
    fn default() -> Self {
        Self {
            max_instances: 1 << 20,
            max_input_curves: 1 << 20,
            max_retained_segments: 1 << 20,
            max_retained_shapes: 1 << 20,
            max_retained_styles: 1 << 20,
            max_retained_profile_knots: 1 << 20,
            max_unreachable_segments: 1 << 16,
            max_unreachable_shapes: 1 << 12,
            max_unreachable_styles: 1 << 12,
        }
    }
}

/// A named generation of one retained plan's compacted table layout.
///
/// Ordinary synchronization preserves the epoch and every live shape/style
/// index. A deterministic reclamation rebuild advances it exactly once, making
/// that exceptional index transition observable to traces and performance
/// evidence without weakening the content identities on derived artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RenderPlanEpoch(u64);

impl RenderPlanEpoch {
    /// The monotone generation number.
    #[must_use]
    pub fn get(self) -> u64 {
        self.0
    }
}

/// Why a retained-plan safe point rebuilt its interned tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionReason {
    /// Unreachable historical rows exceeded a declared retention budget.
    UnreachableBudget,
    /// Append-only admission hit a hard retained-row or index boundary while
    /// reclaimable historical rows existed.
    AdmissionBoundary,
}

/// One named retained-table epoch transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompactionStats {
    /// Layout generation that was replaced.
    pub from_epoch: RenderPlanEpoch,
    /// Layout generation installed by the safe point.
    pub to_epoch: RenderPlanEpoch,
    /// Trigger that selected the exceptional rebuild.
    pub reason: CompactionReason,
    /// Shape rows retained before the rebuild.
    pub shapes_before: usize,
    /// Shape rows retained after the rebuild.
    pub shapes_after: usize,
    /// Segment rows retained before the rebuild.
    pub segments_before: usize,
    /// Segment rows retained after the rebuild.
    pub segments_after: usize,
    /// Style rows retained before the rebuild.
    pub styles_before: usize,
    /// Style rows retained after the rebuild.
    pub styles_after: usize,
}

/// A retained-plan synchronization could not represent scene geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncError {
    /// One renderable's point run violated Chisel's shared-anchor layout.
    InvalidGeometry {
        /// The renderable whose geometry was rejected.
        mob: Mob,
        /// Chisel's precise representation error.
        source: GeomError,
    },
    /// A varying record stroke could not be represented safely.
    InvalidStrokeProfile {
        /// Renderable whose profile was rejected.
        mob: Mob,
        /// The invalid station or paint contract.
        source: crate::StrokeProfileError,
    },
    /// A varying fill record could not be retained safely.
    InvalidFillProfile {
        /// Renderable whose profile was rejected.
        mob: Mob,
        /// Precise station or paint contract.
        source: crate::FillProfileError,
    },
    /// A declared retained-plan count exceeded its caller-selected ceiling.
    LimitExceeded {
        /// The table or work axis being bounded.
        resource: &'static str,
        /// Rows or input curves requested.
        requested: u64,
        /// Configured maximum.
        limit: u64,
    },
    /// A temporary synchronization table could not reserve its bounded input.
    AllocationFailed {
        /// The temporary table being reserved.
        resource: &'static str,
        /// Elements requested.
        requested: u64,
    },
    /// A retained table could not preserve its index or allocation contract.
    Table(TableError),
    /// The retained layout generation could not advance without aliasing an old
    /// epoch.
    EpochExhausted,
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidGeometry { mob, source } => {
                write!(f, "cannot compile geometry for {mob:?}: {source}")
            }
            Self::InvalidFillProfile { mob, source } => {
                write!(f, "cannot compile fill profile for {mob:?}: {source}")
            }
            Self::InvalidStrokeProfile { mob, source } => {
                write!(f, "cannot compile stroke profile for {mob:?}: {source}")
            }
            Self::LimitExceeded {
                resource,
                requested,
                limit,
            } => write!(
                f,
                "retained plan {resource} needs {requested}, exceeding the limit {limit}"
            ),
            Self::AllocationFailed {
                resource,
                requested,
            } => write!(
                f,
                "could not reserve {requested} elements for retained plan {resource}"
            ),
            Self::Table(source) => write!(f, "could not build retained plan: {source}"),
            Self::EpochExhausted => {
                f.write_str("retained plan epoch exhausted its u64 generation space")
            }
        }
    }
}

impl std::error::Error for SyncError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidGeometry { source, .. } => Some(source),
            Self::Table(source) => Some(source),
            Self::InvalidStrokeProfile { source, .. } => Some(source),
            Self::InvalidFillProfile { source, .. } => Some(source),
            Self::LimitExceeded { .. } | Self::AllocationFailed { .. } | Self::EpochExhausted => {
                None
            }
        }
    }
}

impl From<TableError> for SyncError {
    fn from(source: TableError) -> Self {
        Self::Table(source)
    }
}

fn check_limit(resource: &'static str, requested: u64, limit: u64) -> Result<(), SyncError> {
    if requested > limit {
        return Err(SyncError::LimitExceeded {
            resource,
            requested,
            limit,
        });
    }
    Ok(())
}

fn count_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn checked_add_count(
    resource: &'static str,
    left: usize,
    right: usize,
) -> Result<usize, SyncError> {
    left.checked_add(right).ok_or({
        SyncError::Table(TableError::IndexCapacityExceeded {
            resource,
            requested: u64::MAX,
        })
    })
}

fn prepared_index(
    resource: &'static str,
    current: usize,
    pending: usize,
) -> Result<u32, SyncError> {
    let index = checked_add_count(resource, current, pending)?;
    let rows = index.checked_add(1).ok_or({
        SyncError::Table(TableError::IndexCapacityExceeded {
            resource,
            requested: u64::MAX,
        })
    })?;
    check_row_count(resource, rows)?;
    u32::try_from(index).map_err(|_| {
        SyncError::Table(TableError::IndexCapacityExceeded {
            resource,
            requested: count_u64(rows),
        })
    })
}

fn try_reserve_exact<T>(
    values: &mut Vec<T>,
    resource: &'static str,
    additional: usize,
) -> Result<(), SyncError> {
    values
        .try_reserve_exact(additional)
        .map_err(|_| SyncError::AllocationFailed {
            resource,
            requested: count_u64(additional),
        })
}

fn writable_view_affects_any(buffer: &RecordBuffer, fields: &[&str]) -> bool {
    fields
        .iter()
        .any(|field| buffer.writable_view_affects(field))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Reachability {
    unreachable_segments: u64,
    unreachable_shapes: u64,
    unreachable_styles: u64,
}

impl Reachability {
    fn exceeds(self, limits: RenderPlanLimits) -> bool {
        self.unreachable_segments > limits.max_unreachable_segments
            || self.unreachable_shapes > limits.max_unreachable_shapes
            || self.unreachable_styles > limits.max_unreachable_styles
    }

    fn can_relieve(self, error: &SyncError) -> bool {
        let resource = match error {
            SyncError::LimitExceeded { resource, .. }
            | SyncError::AllocationFailed { resource, .. }
            | SyncError::Table(TableError::IndexCapacityExceeded { resource, .. })
            | SyncError::Table(TableError::AllocationFailed { resource, .. }) => *resource,
            SyncError::Table(TableError::ShapeIdentityMismatch { .. })
            | SyncError::Table(TableError::ImageTextureInvalid)
            | SyncError::InvalidGeometry { .. }
            | SyncError::InvalidStrokeProfile { .. }
            | SyncError::InvalidFillProfile { .. }
            | SyncError::EpochExhausted => return false,
        };
        match resource {
            "retained segments" | "segment rows" => self.unreachable_segments > 0,
            "retained shapes" | "shape rows" | "shape index" => self.unreachable_shapes > 0,
            "retained styles" | "style rows" | "style index" | "retained profile knots" => {
                self.unreachable_styles > 0
            }
            _ => false,
        }
    }
}

fn prepare_marks(
    marks: &mut Vec<bool>,
    len: usize,
    resource: &'static str,
) -> Result<(), SyncError> {
    if len > marks.len() {
        let additional = len - marks.len();
        marks
            .try_reserve_exact(additional)
            .map_err(|_| SyncError::AllocationFailed {
                resource,
                requested: count_u64(len),
            })?;
    }
    marks.resize(len, false);
    marks.fill(false);
    Ok(())
}

/// What one [`RenderPlan::sync`] did, and — more usefully — did not do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SyncStats {
    /// Renderables the draw plan presented.
    pub visited: usize,
    /// Outlines compiled from points this sync.
    pub shapes_compiled: usize,
    /// Renderables whose outline was reused without their points being read.
    pub shapes_reused: usize,
    /// Outlines that interned onto an existing compiled shape — the instancing
    /// win, counted separately from a reuse because it is a different mechanism:
    /// this one is two *different* mobjects sharing one outline.
    pub shapes_shared: usize,
    /// Style rows recomputed this sync.
    pub styles_rebuilt: usize,
    /// Retained entries dropped because their mobject left the draw plan.
    pub dropped: usize,
    /// The exceptional table-layout transition performed at this safe point.
    pub compaction: Option<CompactionStats>,
}

/// One renderable's retained compilation.
#[derive(Debug, Clone, Copy)]
struct Retained {
    shape_dep: Dependency,
    style_dep: Dependency,
    shape: u32,
    style: u32,
    /// The object-space first-anchor normalization, kept so a reuse needs no
    /// point read. The entry placement composes with it per frame.
    local_origin: Vec3,
}

/// The compiled, backend-neutral render IR for a scene, retained across frames.
#[derive(Debug, Clone)]
pub struct RenderPlan {
    segments: Vec<Segment>,
    styles: StyleTable,
    shapes: ShapeTable,
    images: ImageTable,
    retained: HashMap<Mob, Retained>,
    scratch_instances: Vec<Instance>,
    scratch_retained: HashMap<Mob, Retained>,
    scratch_seen: HashSet<Mob>,
    scratch_live_shapes: Vec<bool>,
    scratch_live_styles: Vec<bool>,
    stats: SyncStats,
    epoch: RenderPlanEpoch,
    geometry_key: GeometryIdentity,
    plan_key: PlanIdentity,
}

/// Collision-resistant identity of the compiled geometry tables.
///
/// This is deliberately separate from [`PlanIdentity`]: a recolour or painter
/// reorder does not invalidate geometry-only artifacts such as
/// [`crate::fill::MonoTable`]. The zero value is reserved for an unbuilt
/// artifact and is never emitted by the SHA-256 construction below.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct GeometryIdentity(Digest);

impl Default for GeometryIdentity {
    fn default() -> Self {
        Self(Digest::from_bytes([0; 32]))
    }
}

/// Collision-resistant identity of every pixel-deciding row in a render plan.
///
/// Unlike an allocation address or a local generation counter, a content
/// identity lets independently compiled but identical plans share derived
/// artifacts safely, while reordered or stale instance lists cannot alias.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PlanIdentity(Digest);

impl Default for PlanIdentity {
    fn default() -> Self {
        Self(Digest::from_bytes([0; 32]))
    }
}

impl Default for RenderPlan {
    fn default() -> Self {
        let mut plan = Self {
            segments: Vec::new(),
            styles: StyleTable::default(),
            shapes: ShapeTable::default(),
            images: ImageTable::default(),
            retained: HashMap::new(),
            scratch_instances: Vec::new(),
            scratch_retained: HashMap::new(),
            scratch_seen: HashSet::new(),
            scratch_live_shapes: Vec::new(),
            scratch_live_styles: Vec::new(),
            stats: SyncStats::default(),
            epoch: RenderPlanEpoch::default(),
            geometry_key: GeometryIdentity::default(),
            plan_key: PlanIdentity::default(),
        };
        plan.refresh_identities();
        plan
    }
}

/// Allocation-free canonical input to the in-tree SHA-256 implementation.
///
/// These identities are process-local guards rather than durable documents, so
/// they do not ride the size-limited serialization envelope. They preserve
/// exact float bits (matching `StyleTable`'s equality) and domain-separate every
/// sequence, giving the same collision resistance as the content-addressed
/// cache without making large scenes fail merely because an integrity check was
/// requested.
struct IdentityHasher(Sha256);

impl IdentityHasher {
    fn new(domain: &[u8]) -> Self {
        let mut hash = Sha256::new();
        hash.update(domain);
        Self(hash)
    }

    fn bytes(&mut self, value: &[u8]) {
        self.0.update(value);
    }

    fn bool(&mut self, value: bool) {
        self.bytes(&[u8::from(value)]);
    }

    fn u32(&mut self, value: u32) {
        self.bytes(&value.to_le_bytes());
    }

    fn u64(&mut self, value: u64) {
        self.bytes(&value.to_le_bytes());
    }

    fn f32(&mut self, value: f32) {
        self.u32(value.to_bits());
    }

    fn f64(&mut self, value: f64) {
        self.u64(value.to_bits());
    }

    fn digest(&mut self, value: Digest) {
        self.bytes(value.as_bytes());
    }

    fn finish(self) -> Digest {
        self.0.finalize()
    }
}

impl RenderPlan {
    /// An empty plan.
    #[must_use]
    pub fn new() -> RenderPlan {
        RenderPlan::default()
    }

    /// The current retained-table layout generation.
    #[must_use]
    pub fn epoch(&self) -> RenderPlanEpoch {
        self.epoch
    }

    /// Content-addressed image rows retained across camera renders.
    #[must_use]
    pub const fn images(&self) -> &ImageTable {
        &self.images
    }

    pub(crate) fn prepare_image_frame(&self) -> Result<ImageTable, SyncError> {
        self.images.prepare_frame().map_err(SyncError::from)
    }

    pub(crate) fn commit_image_frame(&mut self, mut images: ImageTable) {
        images.retain_current();
        self.images = images;
    }

    fn reachability(&mut self) -> Result<Reachability, SyncError> {
        prepare_marks(
            &mut self.scratch_live_shapes,
            self.shapes.shapes().len(),
            "shape reachability",
        )?;
        prepare_marks(
            &mut self.scratch_live_styles,
            self.styles.len(),
            "style reachability",
        )?;

        for retained in self.retained.values() {
            if let Some(live) = usize::try_from(retained.shape)
                .ok()
                .and_then(|index| self.scratch_live_shapes.get_mut(index))
            {
                *live = true;
            }
            if let Some(live) = usize::try_from(retained.style)
                .ok()
                .and_then(|index| self.scratch_live_styles.get_mut(index))
            {
                *live = true;
            }
        }

        let live_shapes = self
            .scratch_live_shapes
            .iter()
            .filter(|&&live| live)
            .count();
        let live_styles = self
            .scratch_live_styles
            .iter()
            .filter(|&&live| live)
            .count();
        let live_segments = self
            .scratch_live_shapes
            .iter()
            .zip(self.shapes.shapes())
            .filter(|(live, _)| **live)
            .fold(0u64, |total, (_, shape)| {
                total.saturating_add(u64::from(shape.segment_count))
            });

        Ok(Reachability {
            unreachable_segments: count_u64(self.segments.len()).saturating_sub(live_segments),
            unreachable_shapes: count_u64(self.shapes.shapes().len())
                .saturating_sub(count_u64(live_shapes)),
            unreachable_styles: count_u64(self.styles.len()).saturating_sub(count_u64(live_styles)),
        })
    }

    fn rebuild_at_safe_point(
        &mut self,
        stage: &Stage,
        camera: u64,
        limits: RenderPlanLimits,
        reason: CompactionReason,
    ) -> Result<SyncStats, SyncError> {
        let from_epoch = self.epoch;
        let to_epoch = RenderPlanEpoch(
            from_epoch
                .0
                .checked_add(1)
                .ok_or(SyncError::EpochExhausted)?,
        );
        let shapes_before = self.shapes.shapes().len();
        let segments_before = self.segments.len();
        let styles_before = self.styles.len();

        let mut replacement = RenderPlan::new();
        replacement.epoch = to_epoch;
        let mut stats = replacement.sync_append_only(stage, camera, limits)?;
        stats.dropped = self
            .retained
            .keys()
            .filter(|mob| !replacement.retained.contains_key(mob))
            .count();
        stats.compaction = Some(CompactionStats {
            from_epoch,
            to_epoch,
            reason,
            shapes_before,
            shapes_after: replacement.shapes.shapes().len(),
            segments_before,
            segments_after: replacement.segments.len(),
            styles_before,
            styles_after: replacement.styles.len(),
        });
        replacement.stats = stats;
        *self = replacement;
        Ok(stats)
    }

    /// Synchronize against `stage`, at camera revision `camera`.
    ///
    /// Uses [`RenderPlanLimits::default`]. Call [`RenderPlan::sync_with_limits`]
    /// for an explicitly provisioned larger or smaller workload.
    ///
    /// # Errors
    /// Returns [`SyncError`] before mutating the retained plan when geometry is
    /// malformed or the synchronization cannot satisfy its resource contract.
    pub fn sync(&mut self, stage: &Stage, camera: u64) -> Result<SyncStats, SyncError> {
        self.sync_with_limits(stage, camera, RenderPlanLimits::default())
    }

    /// Synchronize with explicit retained-table admission limits.
    ///
    /// Preparation is transactional: changed geometry is compiled and every
    /// prospective table index is checked in temporary storage. Ordinary syncs
    /// append rows and preserve indices within the current [`RenderPlanEpoch`].
    /// When unreachable history exceeds its budget, a fresh compact plan is
    /// prepared separately and replaces this one only after that full rebuild
    /// succeeds.
    ///
    /// # Errors
    /// Returns [`SyncError`] without changing any observable plan state.
    pub fn sync_with_limits(
        &mut self,
        stage: &Stage,
        camera: u64,
        limits: RenderPlanLimits,
    ) -> Result<SyncStats, SyncError> {
        let reachability = self.reachability()?;
        if reachability.exceeds(limits) {
            return self.rebuild_at_safe_point(
                stage,
                camera,
                limits,
                CompactionReason::UnreachableBudget,
            );
        }

        match self.sync_append_only(stage, camera, limits) {
            Err(error) if reachability.can_relieve(&error) => self.rebuild_at_safe_point(
                stage,
                camera,
                limits,
                CompactionReason::AdmissionBoundary,
            ),
            result => result,
        }
    }

    fn sync_append_only(
        &mut self,
        stage: &Stage,
        camera: u64,
        limits: RenderPlanLimits,
    ) -> Result<SyncStats, SyncError> {
        let plan = stage.draw_plan();
        let vector_items = plan
            .items()
            .iter()
            .filter(|item| item.key.program == ProgramKind::Vector)
            .count();
        let draw_items = count_u64(vector_items);
        check_limit("instances", draw_items, limits.max_instances)?;
        check_row_count("instance rows", plan.items().len())?;
        check_limit(
            "retained segments",
            count_u64(self.segments.len()),
            limits.max_retained_segments,
        )?;
        check_limit(
            "retained shapes",
            count_u64(self.shapes.shapes().len()),
            limits.max_retained_shapes,
        )?;
        check_limit(
            "retained styles",
            count_u64(self.styles.len()),
            limits.max_retained_styles,
        )?;

        check_limit(
            "retained profile knots",
            count_u64(self.styles.profile_knots()),
            limits.max_retained_profile_knots,
        )?;

        // Chisel's whole-run invariant is available from record metadata. Check
        // it before materializing any changed point column so one malformed
        // renderable cannot make earlier valid outlines pay compilation work
        // before the atomic refusal.
        for item in plan.items() {
            if item.key.program != ProgramKind::Vector {
                continue;
            }
            let mob = item.mob;
            let Some(entry) = stage.get(mob) else {
                continue;
            };
            if entry.buffer.schema().field_width("point") != Some(3) {
                continue;
            }
            let len = entry.buffer.len();
            if len != 0 && len.is_multiple_of(2) {
                return Err(SyncError::InvalidGeometry {
                    mob,
                    source: GeomError::EvenPointCount { len },
                });
            }
        }

        let mut instances = std::mem::take(&mut self.scratch_instances);
        instances.clear();
        try_reserve_exact(&mut instances, "prepared instances", vector_items)?;
        let mut seen = std::mem::take(&mut self.scratch_seen);
        seen.clear();
        seen.try_reserve(vector_items)
            .map_err(|_| SyncError::AllocationFailed {
                resource: "live mobjects",
                requested: draw_items,
            })?;
        let mut next_retained = std::mem::take(&mut self.scratch_retained);
        next_retained.clear();
        next_retained
            .try_reserve(vector_items)
            .map_err(|_| SyncError::AllocationFailed {
                resource: "retained entries",
                requested: draw_items,
            })?;

        let mut pending_shapes: Vec<(u32, Shape, Vec<Segment>)> = Vec::new();
        let mut pending_shape_indices: HashMap<Digest, u32> = HashMap::new();
        let mut pending_styles: Vec<(u32, Style)> = Vec::new();
        let mut pending_style_indices: HashMap<Vec<u64>, u32> = HashMap::new();
        let mut pending_segments = 0usize;
        let mut pending_profile_knots = 0usize;
        let mut input_curves = 0u64;
        let mut stats = SyncStats::default();

        for (order, item) in plan.items().iter().enumerate() {
            if item.key.program != ProgramKind::Vector {
                continue;
            }
            let mob = item.mob;
            let Some(now) = Revisions::read(stage, mob).map(|r| r.with_camera(camera)) else {
                continue;
            };
            stats.visited += 1;
            seen.insert(mob);

            let previous = next_retained
                .get(&mob)
                .copied()
                .or_else(|| self.retained.get(&mob).copied());
            if let Some(retained) = previous {
                next_retained.entry(mob).or_insert(retained);
            }

            let Some(entry_placement) = stage.placement(mob) else {
                continue;
            };
            let entry = stage.get(mob);
            let (hint_unsafe, style_unsafe) = entry.map_or((false, false), |entry| {
                (
                    writable_view_affects_any(&entry.buffer, &SHAPE_VIEW_FIELDS),
                    writable_view_affects_any(&entry.buffer, &STYLE_VIEW_FIELDS),
                )
            });

            let (shape, local_origin) = match &previous {
                Some(retained) if !hint_unsafe && !retained.shape_dep.is_stale(&now) => {
                    stats.shapes_reused += 1;
                    (retained.shape, retained.local_origin)
                }
                _ => {
                    let counted_from_schema = entry.and_then(|entry| {
                        (entry.buffer.schema().field_width("point") == Some(3))
                            .then_some(entry.buffer.len())
                    });
                    if let Some(point_count) = counted_from_schema {
                        let curves = count_u64(point_count.saturating_sub(1) / 2);
                        input_curves =
                            input_curves
                                .checked_add(curves)
                                .ok_or(SyncError::LimitExceeded {
                                    resource: "input curves",
                                    requested: u64::MAX,
                                    limit: limits.max_input_curves,
                                })?;
                        check_limit("input curves", input_curves, limits.max_input_curves)?;
                    }
                    let Some(object_points) = stage.get_object_points(mob) else {
                        continue;
                    };
                    if object_points.is_empty() {
                        continue;
                    }
                    let local_origin = object_points[0];
                    let mut local = Vec::new();
                    try_reserve_exact(&mut local, "shape-local points", object_points.len())?;
                    local.extend(object_points.iter().map(|point| {
                        [
                            point[0] - local_origin[0],
                            point[1] - local_origin[1],
                            point[2] - local_origin[2],
                        ]
                    }));
                    let path = QuadPath::from_points(local)
                        .map_err(|source| SyncError::InvalidGeometry { mob, source })?;
                    if counted_from_schema.is_none() {
                        input_curves = input_curves
                            .checked_add(count_u64(path.num_curves()))
                            .ok_or(SyncError::LimitExceeded {
                                resource: "input curves",
                                requested: u64::MAX,
                                limit: limits.max_input_curves,
                            })?;
                        check_limit("input curves", input_curves, limits.max_input_curves)?;
                    }

                    let digest = shape_digest(&object_points);
                    let shape = if let Some(index) = self.shapes.shape_index(digest) {
                        stats.shapes_shared += 1;
                        index
                    } else if let Some(&index) = pending_shape_indices.get(&digest) {
                        stats.shapes_shared += 1;
                        index
                    } else {
                        let pending_shape_count =
                            checked_add_count("shape rows", pending_shapes.len(), 1)?;
                        let retained_shape_count = checked_add_count(
                            "shape rows",
                            self.shapes.shapes().len(),
                            pending_shape_count,
                        )?;
                        check_limit(
                            "retained shapes",
                            count_u64(retained_shape_count),
                            limits.max_retained_shapes,
                        )?;
                        let index = prepared_index(
                            "shape rows",
                            self.shapes.shapes().len(),
                            pending_shapes.len(),
                        )?;
                        let first_segment = checked_add_count(
                            "segment rows",
                            self.segments.len(),
                            pending_segments,
                        )?;
                        let first_segment = u32::try_from(first_segment).map_err(|_| {
                            SyncError::Table(TableError::IndexCapacityExceeded {
                                resource: "segment rows",
                                requested: count_u64(first_segment).saturating_add(1),
                            })
                        })?;
                        let hint = Hint::of(stage, mob).translated([
                            -local_origin[0],
                            -local_origin[1],
                            -local_origin[2],
                        ]);
                        let (shape, segments) = compile_shape(digest, &path, hint, first_segment)?;
                        let retained_segment_count = checked_add_count(
                            "segment rows",
                            self.segments.len(),
                            checked_add_count("segment rows", pending_segments, segments.len())?,
                        )?;
                        check_limit(
                            "retained segments",
                            count_u64(retained_segment_count),
                            limits.max_retained_segments,
                        )?;
                        check_row_count("segment rows", retained_segment_count)?;

                        pending_shapes
                            .try_reserve(1)
                            .map_err(|_| SyncError::AllocationFailed {
                                resource: "pending shapes",
                                requested: count_u64(pending_shapes.len()).saturating_add(1),
                            })?;
                        pending_shape_indices.try_reserve(1).map_err(|_| {
                            SyncError::AllocationFailed {
                                resource: "pending shape index",
                                requested: count_u64(pending_shape_indices.len()).saturating_add(1),
                            }
                        })?;
                        pending_segments = pending_segments.checked_add(segments.len()).ok_or(
                            SyncError::Table(TableError::IndexCapacityExceeded {
                                resource: "segment rows",
                                requested: u64::MAX,
                            }),
                        )?;
                        pending_shape_indices.insert(digest, index);
                        pending_shapes.push((index, shape, segments));
                        stats.shapes_compiled += 1;
                        index
                    };
                    (shape, local_origin)
                }
            };

            // The dependency describes authored paint, not just a currently
            // measurable profile. Collapse removes stations, not dependency on
            // geometry. Reuse this per-object metadata without reading paint on
            // unchanged frames, including when style rows are shared.
            let previous_profiled =
                previous.is_some_and(|old| old.style_dep.axes().contains(&Axis::Geometry));
            let style_unsafe = style_unsafe || (previous_profiled && hint_unsafe);
            let (style, profiled) = match &previous {
                Some(retained) if !style_unsafe && !retained.style_dep.is_stale(&now) => {
                    (retained.style, previous_profiled)
                }
                _ => {
                    stats.styles_rebuilt += 1;
                    let has_varying_paint = crate::stroke_profile::has_varying_paint(stage, mob)
                        || crate::fill_profile::has_varying_paint(stage, mob);
                    let mut row = read_style(stage, mob);
                    row.stroke_profile =
                        crate::stroke_profile::from_records(stage, mob, decode_rgba)?
                            .map(std::sync::Arc::new);
                    row.fill_profile = crate::fill_profile::from_records(stage, mob, decode_rgba)?
                        .map(std::sync::Arc::new);
                    let key = row.bits();
                    let style = if let Some(index) = self.styles.index_of(&row) {
                        index
                    } else if let Some(&index) = pending_style_indices.get(&key) {
                        index
                    } else {
                        let pending_style_count =
                            checked_add_count("style rows", pending_styles.len(), 1)?;
                        let retained_style_count = checked_add_count(
                            "style rows",
                            self.styles.len(),
                            pending_style_count,
                        )?;
                        check_limit(
                            "retained styles",
                            count_u64(retained_style_count),
                            limits.max_retained_styles,
                        )?;
                        let index =
                            prepared_index("style rows", self.styles.len(), pending_styles.len())?;
                        pending_styles
                            .try_reserve(1)
                            .map_err(|_| SyncError::AllocationFailed {
                                resource: "pending styles",
                                requested: count_u64(pending_styles.len()).saturating_add(1),
                            })?;
                        pending_style_indices.try_reserve(1).map_err(|_| {
                            SyncError::AllocationFailed {
                                resource: "pending style index",
                                requested: count_u64(pending_style_indices.len()).saturating_add(1),
                            }
                        })?;
                        let knots = row.profile_knots();
                        pending_profile_knots =
                            checked_add_count("profile knots", pending_profile_knots, knots)?;
                        let retained_knots = checked_add_count(
                            "profile knots",
                            self.styles.profile_knots(),
                            pending_profile_knots,
                        )?;
                        check_limit(
                            "retained profile knots",
                            count_u64(retained_knots),
                            limits.max_retained_profile_knots,
                        )?;
                        pending_style_indices.insert(key, index);
                        pending_styles.push((index, row));
                        index
                    };
                    (style, has_varying_paint)
                }
            };

            let order = u32::try_from(order).map_err(|_| {
                SyncError::Table(TableError::IndexCapacityExceeded {
                    resource: "painter order",
                    requested: draw_items,
                })
            })?;
            let volatile = hint_unsafe || style_unsafe;
            let placement = entry_placement.compose(Placement::from_translation(local_origin));
            instances.push(Instance {
                shape,
                style,
                mob,
                placement,
                order,
                revisions: now.fold(),
                volatile,
                hint_unsafe,
            });

            next_retained.insert(
                mob,
                Retained {
                    shape_dep: Dependency::new(now, &SHAPE_AXES),
                    style_dep: Dependency::new(
                        now,
                        if profiled {
                            &PROFILE_STYLE_AXES
                        } else {
                            &STYLE_AXES
                        },
                    ),
                    shape,
                    style,
                    local_origin,
                },
            );
        }

        stats.dropped = self
            .retained
            .keys()
            .filter(|mob| !seen.contains(mob))
            .count();

        self.segments
            .try_reserve_exact(pending_segments)
            .map_err(|_| SyncError::AllocationFailed {
                resource: "segment rows",
                requested: count_u64(self.segments.len())
                    .saturating_add(count_u64(pending_segments)),
            })?;
        self.styles.reserve_additional(pending_styles.len())?;
        self.shapes.reserve_shapes(pending_shapes.len())?;

        for (index, style) in pending_styles {
            self.styles.insert_prepared(index, style);
        }
        for (index, shape, mut segments) in pending_shapes {
            debug_assert_eq!(
                usize::try_from(shape.first_segment).ok(),
                Some(self.segments.len())
            );
            self.segments.append(&mut segments);
            self.shapes.insert_prepared(index, shape);
        }
        self.shapes.swap_instances(&mut instances);
        instances.clear();
        self.scratch_instances = instances;
        std::mem::swap(&mut self.retained, &mut next_retained);
        next_retained.clear();
        self.scratch_retained = next_retained;
        seen.clear();
        self.scratch_seen = seen;
        self.stats = stats;

        if stats.shapes_compiled > 0 {
            self.geometry_key = self.compute_geometry_identity();
        }
        self.plan_key = self.compute_identity(self.geometry_key);
        Ok(stats)
    }

    /// The segment table.
    #[must_use]
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// The interned style table.
    #[must_use]
    pub fn styles(&self) -> &StyleTable {
        &self.styles
    }

    /// The interned shapes and their painter-ordered instances.
    #[must_use]
    pub fn shapes(&self) -> &ShapeTable {
        &self.shapes
    }

    /// What the last [`sync`] did.
    ///
    /// [`sync`]: RenderPlan::sync
    #[must_use]
    pub fn stats(&self) -> SyncStats {
        self.stats
    }

    /// Identity of the shape-indexed geometry consumed by derived fill tables.
    pub(crate) fn geometry_identity(&self) -> GeometryIdentity {
        self.geometry_key
    }

    /// Identity of geometry, styles, and the painter-ordered instance list.
    pub(crate) fn identity(&self) -> PlanIdentity {
        self.plan_key
    }

    /// Refresh both cached identities once after a synchronization.
    fn refresh_identities(&mut self) {
        let geometry = self.compute_geometry_identity();
        let plan = self.compute_identity(geometry);
        self.geometry_key = geometry;
        self.plan_key = plan;
    }

    fn compute_geometry_identity(&self) -> GeometryIdentity {
        let mut hash = IdentityHasher::new(b"fmn-render/geometry-identity/v1");

        hash.u64(self.segments.len() as u64);
        for segment in &self.segments {
            for point in [segment.p0, segment.p1, segment.p2] {
                for component in point {
                    hash.f64(component);
                }
            }
            hash.f64(segment.s0);
            hash.f64(segment.s1);
        }

        let shapes = self.shapes.shapes();
        hash.u64(shapes.len() as u64);
        for shape in shapes {
            hash.digest(shape.digest);
            hash.u32(shape.first_segment);
            hash.u32(shape.segment_count);
            for point in [shape.bounds.min, shape.bounds.mid, shape.bounds.max] {
                for component in point {
                    hash.f64(component);
                }
            }
            hash_hint(&mut hash, shape.hint);
            hash.u64(shape.arc_length.curve_lengths().len() as u64);
            for &length in shape.arc_length.curve_lengths() {
                hash.f64(length);
            }
            hash.u64(shape.subpath_starts.len() as u64);
            for &start in &shape.subpath_starts {
                hash.u32(start);
            }
        }

        GeometryIdentity(hash.finish())
    }

    fn compute_identity(&self, geometry: GeometryIdentity) -> PlanIdentity {
        let mut hash = IdentityHasher::new(b"fmn-render/plan-identity/v1");
        hash.digest(geometry.0);

        let instances = self.shapes.instances();
        hash.u64(instances.len() as u64);
        for instance in instances {
            hash.u32(instance.shape);
            hash.u32(instance.style);
            match self.styles.get(instance.style) {
                Some(style) => {
                    hash.bool(true);
                    for &component in style
                        .stroke_rgba
                        .iter()
                        .chain(style.stroke_rgba_end.iter())
                        .chain(style.fill_rgba.iter())
                        .chain(style.fill_rgba_end.iter())
                    {
                        hash.f32(component);
                    }
                    hash.f32(style.stroke_width);
                    hash.f32(style.stroke_width_end);
                    hash.f32(style.fill_border_width);
                    hash.f32(style.anti_alias_width);
                    hash.f64(style.joint_type.to_code());
                    hash.f64(style.is_fixed_in_frame);
                    for component in style.shading {
                        hash.f64(component);
                    }
                    for component in style.clip_planes.iter().flatten() {
                        hash.f64(*component);
                    }
                    hash.bool(style.flat_stroke);
                    hash.bool(style.scale_stroke_with_zoom);
                    hash.bool(style.stroke_behind);
                    hash.bool(style.depth_test);
                }
                None => hash.bool(false),
            }
            for component in instance.placement.coefficients() {
                hash.f64(component);
            }
            hash.u32(instance.order);
            // A point view disables hint-derived tile classification. Binning
            // built on the other side of this transition is therefore stale
            // even when the point bytes themselves have not moved.
            hash.bool(instance.hint_unsafe);
        }

        // Preserve the legacy identity domain for endpoint-only plans.
        let profiles: Vec<_> = instances
            .iter()
            .enumerate()
            .filter_map(|(i, instance)| {
                self.styles
                    .get(instance.style)?
                    .stroke_profile
                    .as_ref()
                    .map(|p| (i, p))
            })
            .collect();
        if !profiles.is_empty() {
            hash.bytes(b"fmn-render/stroke-profiles/v1");
            hash.u64(profiles.len() as u64);
            for (index, profile) in profiles {
                hash.u64(index as u64);
                hash.u64(profile.knots().len() as u64);
                for knot in profile.knots() {
                    hash.f64(knot.s);
                    hash.f32(knot.width);
                    for component in knot.rgba {
                        hash.f32(component);
                    }
                }
            }
        }
        let fills: Vec<_> = instances
            .iter()
            .enumerate()
            .filter_map(|(i, instance)| {
                self.styles
                    .get(instance.style)?
                    .fill_profile
                    .as_ref()
                    .map(|p| (i, p))
            })
            .collect();
        if !fills.is_empty() {
            hash.bytes(b"fmn-render/fill-profiles/v1");
            hash.u64(fills.len() as u64);
            for (index, profile) in fills {
                hash.u64(index as u64);
                hash.u64(profile.knots().len() as u64);
                for knot in profile.knots() {
                    hash.f64(knot.s);
                    for component in knot.rgba {
                        hash.f32(component);
                    }
                }
            }
        }
        PlanIdentity(hash.finish())
    }
}

/// Feed a primitive hint's full payload, not merely its diagnostic name.
///
/// The parameters participate in specialized fill and containment kernels, so
/// hashing only the enum discriminant would let two differently placed circles
/// or rectangles share a binning that was built for the other one.
fn hash_hint(hash: &mut IdentityHasher, hint: Hint) {
    fn point(hash: &mut IdentityHasher, tag: u8, center: [f64; 3]) {
        hash.bytes(&[tag]);
        for component in center {
            hash.f64(component);
        }
    }

    match hint {
        Hint::General => hash.bytes(&[0]),
        Hint::Line => hash.bytes(&[1]),
        Hint::Polyline { closed } => {
            hash.bytes(&[2]);
            hash.bool(closed);
        }
        Hint::Arc {
            center,
            radius,
            start_angle,
            angle,
        } => {
            point(hash, 3, center);
            hash.f64(radius);
            hash.f64(start_angle);
            hash.f64(angle);
        }
        Hint::Circle { center, radius } => {
            point(hash, 4, center);
            hash.f64(radius);
        }
        Hint::Dot { center, radius } => {
            point(hash, 5, center);
            hash.f64(radius);
        }
        Hint::Rect {
            center,
            width,
            height,
        } => {
            point(hash, 6, center);
            hash.f64(width);
            hash.f64(height);
        }
        Hint::RoundedRect {
            center,
            width,
            height,
            corner_radius,
        } => {
            point(hash, 7, center);
            hash.f64(width);
            hash.f64(height);
            hash.f64(corner_radius);
        }
    }
}

/// Decode one sRGB-encoded colour record into the linear light the IR stores.
///
/// **This is BN-04's "render boundary", and it had no implementation.** The
/// record buffer holds what manim holds — sRGB-encoded components, because
/// `mobject.data` is API surface — while [`Style`] documents linear light and
/// every consumer already assumes it: `fill_rgba_at` and `stroke_rgba_at` both
/// say a ramp "happens where colour interpolation is defined, not in an encoded
/// space". Between the writer and those readers the decode simply did not exist,
/// so a mid-tone would have rendered at its encoded value — visibly wrong, and
/// wrong in the direction that looks like a lighting choice rather than a bug.
///
/// Alpha does not decode. It is a coverage fraction, not a light intensity, and
/// gamma-encoding it is the mistake `fmn_frame::transfer` names on the way out.
///
/// The decode routes through [`fmn_frame::transfer::srgb_decode`] rather than
/// `fmn_core::color::srgb_eotf`: the two compute the same function, but the
/// former rides `fmn_dmath::pow` and the latter rides `std::powf`, and ADR-0010's
/// first binding property is that fmn-dmath owns every transcendental on the
/// certified path. It is decoded **once per interned style row**, not per pixel.
fn decode_rgba(rgba: [f32; 4]) -> [f32; 4] {
    let linearize_srgb = fmn_frame::transfer::srgb_decode;
    [
        linearize_srgb(f64::from(rgba[0])) as f32,
        linearize_srgb(f64::from(rgba[1])) as f32,
        linearize_srgb(f64::from(rgba[2])) as f32,
        rgba[3],
    ]
}

/// Read one renderable's style row out of its record buffer.
///
/// The Reference stores these per point; this reads the ramp's endpoints, which
/// is the subset [`Style`] documents. A point-less mobject yields the default
/// row rather than an error — a group with no geometry of its own is normal, and
/// it simply contributes no instance.
fn read_style(stage: &Stage, mob: Mob) -> Style {
    let mut row = Style::default();
    let Some(entry) = stage.get(mob) else {
        return row;
    };
    let buffer = &entry.buffer;
    let n = buffer.len();
    if n == 0 {
        return row.with_uniforms(entry.uniforms());
    }
    let last = n - 1;
    let rgba = |index: usize, field: &str| -> [f32; 4] {
        buffer
            .read(index, field)
            .and_then(|v| <[f32; 4]>::try_from(v.as_slice()).ok())
            .unwrap_or([0.0; 4])
    };
    let scalar = |index: usize, field: &str| -> f32 {
        buffer
            .read(index, field)
            .and_then(|v| v.first().copied())
            .unwrap_or(0.0)
    };
    row.stroke_rgba = decode_rgba(rgba(0, "stroke_rgba"));
    row.stroke_rgba_end = decode_rgba(rgba(last, "stroke_rgba"));
    row.fill_rgba = decode_rgba(rgba(0, "fill_rgba"));
    row.fill_rgba_end = decode_rgba(rgba(last, "fill_rgba"));
    row.stroke_width = scalar(0, "stroke_width");
    row.stroke_width_end = scalar(last, "stroke_width");
    row.fill_border_width = scalar(0, "fill_border_width");
    row.with_uniforms(entry.uniforms())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Binning, FrameConfig, FrameJob, FrameJobError, MonoTable, OutputTransform, ScreenMap,
        TileCache, TileWork, Tiling, Viewport, frame_digest,
    };
    use fmn_core::color::LinearRgba;
    use fmn_mobject::{Mobject, RecordBuffer, RecordSchema};

    fn vmobject(points: &[[f64; 3]]) -> Mobject {
        let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len()).unwrap();
        for (i, p) in points.iter().enumerate() {
            buffer.write(i, "point", &[p[0] as f32, p[1] as f32, p[2] as f32]);
            buffer.write(i, "stroke_rgba", &[1.0, 1.0, 1.0, 1.0]);
            buffer.write(i, "stroke_width", &[2.0]);
        }
        Mobject::from_buffer(buffer)
    }

    fn tri_at(dx: f64, dy: f64) -> Mobject {
        vmobject(&[
            [dx, dy, 0.0],
            [dx + 1.0, dy, 0.0],
            [dx + 2.0, dy, 0.0],
            [dx + 2.0, dy + 0.5, 0.0],
            [dx + 2.0, dy + 1.0, 0.0],
        ])
    }

    fn staged(n: usize) -> (Stage, Vec<Mob>) {
        let mut stage = Stage::new();
        let mut mobs = Vec::new();
        for i in 0..n {
            let mob = stage.add(tri_at(i as f64 * 10.0, 0.0));
            stage.add_to_scene(mob).expect("live");
            mobs.push(mob);
        }
        (stage, mobs)
    }

    fn sync_valid(plan: &mut RenderPlan, stage: &Stage, camera: u64) -> SyncStats {
        plan.sync(stage, camera).expect("valid test scene")
    }

    fn compacting_limits() -> RenderPlanLimits {
        RenderPlanLimits {
            max_retained_segments: 64,
            max_retained_shapes: 32,
            max_retained_styles: 32,
            max_unreachable_segments: 0,
            max_unreachable_shapes: 0,
            max_unreachable_styles: 0,
            ..RenderPlanLimits::default()
        }
    }

    fn render_fixture() -> (Viewport, ScreenMap, FrameConfig) {
        let viewport = Viewport {
            width: 48,
            height: 32,
        };
        let map = ScreenMap {
            scale: 8.0,
            origin: [8.0, 8.0],
            y_up: false,
        };
        let config = FrameConfig::new(
            viewport,
            map,
            LinearRgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        );
        (viewport, map, config)
    }

    #[test]
    fn a_first_sync_compiles_and_a_second_reads_no_points() {
        // The claim the whole module exists for. Not "recompiles less" —
        // *nothing* is compiled on the second pass, and the points are never
        // touched.
        let (stage, _) = staged(3);
        let mut plan = RenderPlan::new();

        let first = sync_valid(&mut plan, &stage, 0);
        assert_eq!(first.visited, 3);
        assert_eq!(first.shapes_compiled, 1, "three copies of one outline");
        assert_eq!(first.shapes_shared, 2);
        assert_eq!(first.shapes_reused, 0);

        let second = sync_valid(&mut plan, &stage, 0);
        assert_eq!(second.visited, 3);
        assert_eq!(second.shapes_compiled, 0);
        assert_eq!(second.shapes_shared, 0);
        assert_eq!(second.shapes_reused, 3);
        assert_eq!(second.styles_rebuilt, 0);
    }

    #[test]
    fn malformed_shared_anchor_geometry_is_named_and_atomic() {
        let (mut stage, _) = staged(1);
        let mut plan = RenderPlan::new();
        sync_valid(&mut plan, &stage, 0);

        let before_segments = plan.segments().to_vec();
        let before_styles = plan.styles().rows().to_vec();
        let before_shapes = plan.shapes().shapes().to_vec();
        let before_instances = plan.shapes().instances().to_vec();
        let before_stats = plan.stats();
        let before_geometry = plan.geometry_identity();
        let before_identity = plan.identity();
        let before_retained = plan.retained.len();

        let malformed = stage.add(vmobject(&[[20.0, 0.0, 0.0], [21.0, 0.0, 0.0]]));
        stage.add_to_scene(malformed).expect("live");
        assert_eq!(
            plan.sync(&stage, 0)
                .expect_err("even point run must be refused"),
            SyncError::InvalidGeometry {
                mob: malformed,
                source: fmn_geom::GeomError::EvenPointCount { len: 2 },
            }
        );

        assert_eq!(plan.segments(), before_segments);
        assert_eq!(plan.styles().rows(), before_styles);
        assert_eq!(plan.shapes().shapes(), before_shapes);
        assert_eq!(plan.shapes().instances(), before_instances);
        assert_eq!(plan.stats(), before_stats);
        assert_eq!(plan.geometry_identity(), before_geometry);
        assert_eq!(plan.identity(), before_identity);
        assert_eq!(plan.retained.len(), before_retained);
    }

    #[test]
    fn a_bare_mobjects_even_point_run_is_data_not_geometry() {
        // R-13 (fm-5wq.37): `Group(...).set_points(30 points)` in corpus
        // DirectMGFInterpretation. The Reference's bare Mobject draws nothing;
        // its points must not reach the shared-anchor validator.
        let mut stage = Stage::new();
        let child = stage.add(tri_at(0.0, 0.0));
        let points: Vec<[f64; 3]> = (0..30).map(|i| [f64::from(i), 1.0, 0.0]).collect();
        let group = stage.add(Mobject::from_points(&points));
        stage.attach(group, child).expect("attach");
        stage.add_to_scene(group).expect("live");
        let mut plan = RenderPlan::new();
        sync_valid(&mut plan, &stage, 0);
        assert_eq!(
            plan.shapes().instances().len(),
            1,
            "only the vector child draws"
        );
    }

    #[test]
    fn retained_limits_count_deduplicated_rows_not_draw_items() {
        let (stage, _) = staged(8);
        let mut plan = RenderPlan::new();
        let limits = RenderPlanLimits {
            max_instances: 8,
            max_input_curves: 16,
            max_retained_segments: 2,
            max_retained_shapes: 1,
            max_retained_styles: 1,
            ..RenderPlanLimits::default()
        };

        let stats = plan
            .sync_with_limits(&stage, 0, limits)
            .expect("eight copies fit one retained shape and style row");

        assert_eq!(stats.shapes_compiled, 1);
        assert_eq!(stats.shapes_shared, 7);
        assert_eq!(plan.segments().len(), 2);
        assert_eq!(plan.shapes().shapes().len(), 1);
        assert_eq!(plan.styles().len(), 1);
        assert_eq!(plan.shapes().instances().len(), 8);
    }

    #[test]
    fn retained_limit_refusal_is_named_and_atomic() {
        let (mut stage, mobs) = staged(1);
        let mut plan = RenderPlan::new();
        sync_valid(&mut plan, &stage, 0);

        let before_segments = plan.segments().to_vec();
        let before_styles = plan.styles().rows().to_vec();
        let before_shapes = plan.shapes().shapes().to_vec();
        let before_instances = plan.shapes().instances().to_vec();
        let before_stats = plan.stats();
        let before_geometry = plan.geometry_identity();
        let before_identity = plan.identity();
        let before_retained = plan.retained.len();

        stage
            .get_mut(mobs[0])
            .expect("live")
            .buffer
            .write(2, "point", &[500.0, 500.0, 0.0]);
        let limits = RenderPlanLimits {
            max_retained_shapes: 1,
            ..RenderPlanLimits::default()
        };

        assert_eq!(
            plan.sync_with_limits(&stage, 0, limits)
                .expect_err("a second retained shape exceeds the limit"),
            SyncError::LimitExceeded {
                resource: "retained shapes",
                requested: 2,
                limit: 1,
            }
        );

        assert_eq!(plan.segments(), before_segments);
        assert_eq!(plan.styles().rows(), before_styles);
        assert_eq!(plan.shapes().shapes(), before_shapes);
        assert_eq!(plan.shapes().instances(), before_instances);
        assert_eq!(plan.stats(), before_stats);
        assert_eq!(plan.geometry_identity(), before_geometry);
        assert_eq!(plan.identity(), before_identity);
        assert_eq!(plan.retained.len(), before_retained);
    }

    #[test]
    fn each_sync_work_axis_has_a_named_limit() {
        let (two, _) = staged(2);
        let mut plan = RenderPlan::new();
        assert_eq!(
            plan.sync_with_limits(
                &two,
                0,
                RenderPlanLimits {
                    max_instances: 1,
                    ..RenderPlanLimits::default()
                },
            )
            .expect_err("two draw items exceed one instance"),
            SyncError::LimitExceeded {
                resource: "instances",
                requested: 2,
                limit: 1,
            }
        );
        assert!(plan.shapes().instances().is_empty());
        assert!(plan.shapes().shapes().is_empty());
        assert!(plan.segments().is_empty());

        let (one, _) = staged(1);
        assert_eq!(
            plan.sync_with_limits(
                &one,
                0,
                RenderPlanLimits {
                    max_input_curves: 1,
                    ..RenderPlanLimits::default()
                },
            )
            .expect_err("the two-curve outline exceeds one input curve"),
            SyncError::LimitExceeded {
                resource: "input curves",
                requested: 2,
                limit: 1,
            }
        );
        assert!(plan.shapes().shapes().is_empty());
        assert!(plan.segments().is_empty());

        assert_eq!(
            plan.sync_with_limits(
                &one,
                0,
                RenderPlanLimits {
                    max_retained_segments: 1,
                    ..RenderPlanLimits::default()
                },
            )
            .expect_err("the two compiled segments exceed one retained row"),
            SyncError::LimitExceeded {
                resource: "retained segments",
                requested: 2,
                limit: 1,
            }
        );
        assert!(plan.shapes().shapes().is_empty());
        assert!(plan.segments().is_empty());
    }

    #[test]
    fn retained_style_limit_refusal_preserves_the_old_style_and_identity() {
        let (mut stage, mobs) = staged(1);
        let mut plan = RenderPlan::new();
        sync_valid(&mut plan, &stage, 0);
        let before_styles = plan.styles().rows().to_vec();
        let before_instances = plan.shapes().instances().to_vec();
        let before_stats = plan.stats();
        let before_identity = plan.identity();

        stage
            .get_mut(mobs[0])
            .expect("live")
            .buffer
            .write(0, "stroke_rgba", &[1.0, 0.0, 0.0, 1.0]);
        assert_eq!(
            plan.sync_with_limits(
                &stage,
                0,
                RenderPlanLimits {
                    max_retained_styles: 1,
                    ..RenderPlanLimits::default()
                },
            )
            .expect_err("a second retained style exceeds the limit"),
            SyncError::LimitExceeded {
                resource: "retained styles",
                requested: 2,
                limit: 1,
            }
        );

        assert_eq!(plan.styles().rows(), before_styles);
        assert_eq!(plan.shapes().instances(), before_instances);
        assert_eq!(plan.stats(), before_stats);
        assert_eq!(plan.identity(), before_identity);
    }

    #[test]
    fn a_lower_limit_does_not_grandfather_existing_retained_rows() {
        let (stage, _) = staged(1);
        let mut plan = RenderPlan::new();
        sync_valid(&mut plan, &stage, 0);
        let before_instances = plan.shapes().instances().to_vec();
        let before_stats = plan.stats();
        let before_identity = plan.identity();

        assert_eq!(
            plan.sync_with_limits(
                &stage,
                0,
                RenderPlanLimits {
                    max_retained_segments: 1,
                    ..RenderPlanLimits::default()
                },
            )
            .expect_err("the existing two-row segment table exceeds the new limit"),
            SyncError::LimitExceeded {
                resource: "retained segments",
                requested: 2,
                limit: 1,
            }
        );
        assert_eq!(plan.shapes().instances(), before_instances);
        assert_eq!(plan.stats(), before_stats);
        assert_eq!(plan.identity(), before_identity);
    }

    #[test]
    fn unreachable_budgets_plateau_and_wait_frames_keep_indices_and_cache_hits() {
        let (mut stage, mobs) = staged(1);
        let mob = mobs[0];
        let limits = compacting_limits();
        let mut plan = RenderPlan::new();
        plan.sync_with_limits(&stage, 0, limits)
            .expect("initial retained plan");

        // Warm the reachability marks and every transactional sync scratch table.
        let warm = plan
            .sync_with_limits(&stage, 0, limits)
            .expect("warm wait frame");
        assert!(warm.compaction.is_none());
        let stable_shape = plan.shapes().instances()[0].shape;
        let stable_style = plan.shapes().instances()[0].style;
        let stable_epoch = plan.epoch();
        let scratch_capacities = (
            plan.scratch_instances.capacity(),
            plan.scratch_retained.capacity(),
            plan.scratch_seen.capacity(),
            plan.scratch_live_shapes.capacity(),
            plan.scratch_live_styles.capacity(),
        );
        for _ in 0..16 {
            let stats = plan
                .sync_with_limits(&stage, 0, limits)
                .expect("unchanged wait frame");
            assert!(stats.compaction.is_none());
            assert_eq!(stats.shapes_reused, 1);
            assert_eq!(plan.epoch(), stable_epoch);
            assert_eq!(plan.shapes().instances()[0].shape, stable_shape);
            assert_eq!(plan.shapes().instances()[0].style, stable_style);
        }
        assert_eq!(
            (
                plan.scratch_instances.capacity(),
                plan.scratch_retained.capacity(),
                plan.scratch_seen.capacity(),
                plan.scratch_live_shapes.capacity(),
                plan.scratch_live_styles.capacity(),
            ),
            scratch_capacities,
            "steady wait sync grew a retained scratch allocation"
        );

        let mut max_shapes = plan.shapes().shapes().len();
        let mut max_segments = plan.segments().len();
        let mut max_styles = plan.styles().len();
        let mut transitions = 0usize;
        for generation in 1..=64 {
            let entry = stage.get_mut(mob).expect("live");
            entry
                .buffer
                .write(2, "point", &[2.0 + generation as f32 / 128.0, 0.25, 0.0]);
            entry.buffer.write(
                0,
                "stroke_rgba",
                &[generation as f32 / 128.0, 0.25, 0.75, 1.0],
            );
            let stats = plan
                .sync_with_limits(&stage, 0, limits)
                .expect("bounded mutation sync");
            if let Some(compaction) = stats.compaction {
                transitions += 1;
                assert_eq!(compaction.from_epoch.get() + 1, compaction.to_epoch.get());
                assert_eq!(plan.epoch(), compaction.to_epoch);
                assert_eq!(compaction.reason, CompactionReason::UnreachableBudget);
            }
            max_shapes = max_shapes.max(plan.shapes().shapes().len());
            max_segments = max_segments.max(plan.segments().len());
            max_styles = max_styles.max(plan.styles().len());
        }
        assert!(transitions > 0, "the adversarial corpus never reclaimed");
        assert!(
            max_shapes <= 2,
            "shape history did not plateau: {max_shapes}"
        );
        assert!(
            max_segments <= 4,
            "segment history did not plateau: {max_segments}"
        );
        assert!(
            max_styles <= 2,
            "style history did not plateau: {max_styles}"
        );

        // A final safe point may collect the last mutation's one-frame history;
        // the next wait must be wholly stable and cacheable.
        plan.sync_with_limits(&stage, 0, limits)
            .expect("settling safe point");
        plan.sync_with_limits(&stage, 0, limits)
            .expect("post-compaction wait");
        let final_shape = plan.shapes().instances()[0].shape;
        let final_style = plan.shapes().instances()[0].style;
        let final_epoch = plan.epoch();
        let (viewport, map, _) = render_fixture();
        let output = OutputTransform {
            viewport,
            map,
            pixel_format: 0,
        };
        let binning =
            Binning::build(&plan, viewport, Tiling::default(), map).expect("bounded cache fixture");
        let mut cache = TileCache::new();
        cache.begin_frame();
        for tile in 0..binning.tile_count() {
            if let TileWork::Rasterize(key) = cache
                .plan_tile(&binning, &plan, tile, 0, output)
                .expect("in-range tile")
            {
                cache.store(tile, key, tile);
            }
        }
        assert!(cache.stats().misses > 0, "fixture populated no cached tile");

        let wait = plan
            .sync_with_limits(&stage, 0, limits)
            .expect("cache-hit wait frame");
        assert!(wait.compaction.is_none());
        assert_eq!(plan.epoch(), final_epoch);
        assert_eq!(plan.shapes().instances()[0].shape, final_shape);
        assert_eq!(plan.shapes().instances()[0].style, final_style);
        let wait_binning =
            Binning::build(&plan, viewport, Tiling::default(), map).expect("bounded wait binning");
        cache.begin_frame();
        for tile in 0..wait_binning.tile_count() {
            let _ = cache
                .plan_tile(&wait_binning, &plan, tile, 0, output)
                .expect("in-range tile");
        }
        assert!(
            cache.stats().hits > 0,
            "wait frame produced no tile-cache hit"
        );
        assert_eq!(cache.stats().misses, 0);
        assert_eq!(cache.stats().poisoned, 0);
    }

    #[test]
    fn a_hard_boundary_compacts_when_rebuilding_the_live_state_fits() {
        let (mut stage, mobs) = staged(1);
        let mut plan = RenderPlan::new();
        let limits = RenderPlanLimits {
            max_retained_segments: 4,
            max_retained_shapes: 2,
            max_retained_styles: 2,
            ..RenderPlanLimits::default()
        };
        plan.sync_with_limits(&stage, 0, limits)
            .expect("one live outline fits");

        let entry = stage.get_mut(mobs[0]).expect("live");
        entry.buffer.write(2, "point", &[2.25, 0.25, 0.0]);
        entry
            .buffer
            .write(0, "stroke_rgba", &[0.25, 0.5, 0.75, 1.0]);
        let first = plan
            .sync_with_limits(&stage, 0, limits)
            .expect("one historical row remains admitted");
        assert!(first.compaction.is_none());
        assert_eq!(plan.shapes().shapes().len(), 2);
        assert_eq!(plan.styles().len(), 2);

        let entry = stage.get_mut(mobs[0]).expect("live");
        entry.buffer.write(2, "point", &[2.5, 0.5, 0.0]);
        entry
            .buffer
            .write(0, "stroke_rgba", &[0.5, 0.25, 0.75, 1.0]);
        let stats = plan
            .sync_with_limits(&stage, 0, limits)
            .expect("safe-point rebuild fits the same hard boundary");
        let compaction = stats.compaction.expect("boundary must name its epoch");

        assert_eq!(compaction.reason, CompactionReason::AdmissionBoundary);
        assert_eq!(compaction.from_epoch, RenderPlanEpoch(0));
        assert_eq!(compaction.to_epoch, RenderPlanEpoch(1));
        assert_eq!(plan.epoch(), RenderPlanEpoch(1));
        assert_eq!(plan.shapes().shapes().len(), 1);
        assert_eq!(plan.segments().len(), 2);
        assert_eq!(plan.styles().len(), 1);
    }

    #[test]
    fn a_failed_budget_compaction_leaves_every_observable_unchanged() {
        let (mut stage, mobs) = staged(1);
        let mob = mobs[0];
        let limits = compacting_limits();
        let mut plan = RenderPlan::new();
        plan.sync_with_limits(&stage, 0, limits)
            .expect("initial retained plan");
        stage
            .get_mut(mob)
            .expect("live")
            .buffer
            .write(2, "point", &[2.5, 0.25, 0.0]);
        plan.sync_with_limits(&stage, 0, limits)
            .expect("one frame of admitted history");
        assert_eq!(plan.shapes().shapes().len(), 2);

        let before_segments = plan.segments().to_vec();
        let before_styles = plan.styles().rows().to_vec();
        let before_shapes = plan.shapes().shapes().to_vec();
        let before_instances = plan.shapes().instances().to_vec();
        let before_stats = plan.stats();
        let before_geometry = plan.geometry_identity();
        let before_identity = plan.identity();
        let before_epoch = plan.epoch();
        stage
            .get_mut(mob)
            .expect("live")
            .buffer
            .resize(2)
            .expect("bounded resize");

        assert_eq!(
            plan.sync_with_limits(&stage, 0, limits)
                .expect_err("malformed replacement must not install"),
            SyncError::InvalidGeometry {
                mob,
                source: GeomError::EvenPointCount { len: 2 },
            }
        );
        assert_eq!(plan.segments(), before_segments);
        assert_eq!(plan.styles().rows(), before_styles);
        assert_eq!(plan.shapes().shapes(), before_shapes);
        assert_eq!(plan.shapes().instances(), before_instances);
        assert_eq!(plan.stats(), before_stats);
        assert_eq!(plan.geometry_identity(), before_geometry);
        assert_eq!(plan.identity(), before_identity);
        assert_eq!(plan.epoch(), before_epoch);
    }

    #[test]
    fn compaction_refuses_stale_artifacts_and_preserves_frame_bits_at_all_thread_counts() {
        let (mut stage, mobs) = staged(1);
        let mob = mobs[0];
        let append_limits = RenderPlanLimits {
            max_unreachable_segments: u64::MAX,
            max_unreachable_shapes: u64::MAX,
            max_unreachable_styles: u64::MAX,
            ..RenderPlanLimits::default()
        };
        let compact_limits = compacting_limits();
        let mut append_only = RenderPlan::new();
        let mut compacting = RenderPlan::new();
        append_only
            .sync_with_limits(&stage, 0, append_limits)
            .expect("append-only seed");
        compacting
            .sync_with_limits(&stage, 0, compact_limits)
            .expect("compacting seed");

        stage
            .get_mut(mob)
            .expect("live")
            .buffer
            .write(2, "point", &[2.25, 0.25, 0.0]);
        append_only
            .sync_with_limits(&stage, 0, append_limits)
            .expect("append-only first mutation");
        compacting
            .sync_with_limits(&stage, 0, compact_limits)
            .expect("compacting first mutation");
        let (viewport, map, config) = render_fixture();
        let stale_mono = MonoTable::build(&compacting, map).expect("bounded stale monotone table");
        let stale_binning = Binning::build(&compacting, viewport, Tiling::default(), map)
            .expect("bounded stale binning");

        stage
            .get_mut(mob)
            .expect("live")
            .buffer
            .write(2, "point", &[2.5, 0.5, 0.0]);
        append_only
            .sync_with_limits(&stage, 0, append_limits)
            .expect("append-only second mutation");
        let stats = compacting
            .sync_with_limits(&stage, 0, compact_limits)
            .expect("budget-triggered compaction");
        assert_eq!(
            stats.compaction.expect("named compaction").reason,
            CompactionReason::UnreachableBudget
        );

        let append_mono =
            MonoTable::build(&append_only, map).expect("bounded append monotone table");
        let append_binning = Binning::build(&append_only, viewport, Tiling::default(), map)
            .expect("bounded append binning");
        let compact_mono =
            MonoTable::build(&compacting, map).expect("bounded compact monotone table");
        let compact_binning = Binning::build(&compacting, viewport, Tiling::default(), map)
            .expect("bounded compact binning");

        assert!(!stale_mono.matches_plan(&compacting));
        assert!(!stale_binning.matches_plan(&compacting));
        assert_eq!(
            FrameJob::new(&compacting, &stale_mono, &compact_binning, config)
                .expect_err("stale geometry must be refused"),
            FrameJobError::MonoPlanMismatch
        );
        assert_eq!(
            FrameJob::new(&compacting, &compact_mono, &stale_binning, config)
                .expect_err("stale command indices must be refused"),
            FrameJobError::BinningPlanMismatch
        );

        let append_frame = FrameJob::new(&append_only, &append_mono, &append_binning, config)
            .expect("coherent append-only job")
            .render(1)
            .expect("append-only frame");
        let expected = frame_digest(&append_frame).expect("canonical append digest");
        for threads in [1, 4, 16] {
            let frame = FrameJob::new(&compacting, &compact_mono, &compact_binning, config)
                .expect("coherent compacted job")
                .render(threads)
                .expect("compacted frame");
            assert_eq!(
                frame_digest(&frame).expect("canonical compact digest"),
                expected,
                "compaction changed frame bits at {threads} threads"
            );
        }
    }

    #[test]
    fn writable_views_refresh_exactly_the_render_fields_they_expose() {
        let (mut stage, mobs) = staged(1);
        let mob = mobs[0];
        let mut plan = RenderPlan::new();
        sync_valid(&mut plan, &stage, 0);

        let point_view = stage
            .get_mut(mob)
            .expect("live")
            .buffer
            .export_field_view("point", true)
            .expect("point field");
        for _ in 0..2 {
            let stats = sync_valid(&mut plan, &stage, 0);
            assert_eq!(stats.shapes_reused, 0);
            assert_eq!(
                stats.shapes_shared, 1,
                "unchanged bytes may re-intern, but must be observed"
            );
            assert_eq!(stats.styles_rebuilt, 0);
            let instance = plan.shapes().instances()[0];
            assert!(instance.volatile);
            assert!(instance.hint_unsafe);
        }
        drop(point_view);

        // Detaching the writable point view conservatively advances its field
        // revision, so the final state is observed once more before ordinary
        // reuse resumes.
        let detached = sync_valid(&mut plan, &stage, 0);
        assert_eq!(detached.shapes_reused, 0);
        assert_eq!(detached.shapes_shared, 1);
        assert!(!plan.shapes().instances()[0].hint_unsafe);
        assert_eq!(sync_valid(&mut plan, &stage, 0).shapes_reused, 1);

        let style_view = stage
            .get_mut(mob)
            .expect("live")
            .buffer
            .export_field_view("fill_rgba", true)
            .expect("fill field");
        for _ in 0..2 {
            let stats = sync_valid(&mut plan, &stage, 0);
            assert_eq!(stats.shapes_reused, 1);
            assert_eq!(stats.styles_rebuilt, 1);
            let instance = plan.shapes().instances()[0];
            assert!(instance.volatile);
            assert!(
                !instance.hint_unsafe,
                "a colour view cannot invalidate geometry"
            );
        }
        drop(style_view);

        let whole_view = stage.get_mut(mob).expect("live").buffer.export_view(true);
        let stats = sync_valid(&mut plan, &stage, 0);
        assert_eq!(stats.shapes_reused, 0);
        assert_eq!(stats.shapes_shared, 1);
        assert_eq!(stats.styles_rebuilt, 1);
        assert!(plan.shapes().instances()[0].hint_unsafe);
        drop(whole_view);
    }

    #[test]
    fn a_writable_custom_field_view_does_not_invalidate_render_state() {
        let schema = RecordSchema::new(
            &[
                ("point", 3),
                ("stroke_rgba", 4),
                ("stroke_width", 1),
                ("joint_angle", 1),
                ("fill_rgba", 4),
                ("base_normal", 3),
                ("fill_border_width", 1),
                ("user_metadata", 1),
            ],
            &["point"],
            &["point"],
        )
        .unwrap();
        let points = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 0.5, 0.0],
            [2.0, 1.0, 0.0],
        ];
        let mut buffer = RecordBuffer::new(schema, points.len()).unwrap();
        for (i, point) in points.iter().enumerate() {
            buffer.write(i, "point", point);
            buffer.write(i, "stroke_rgba", &[1.0; 4]);
            buffer.write(i, "stroke_width", &[2.0]);
        }
        let mut stage = Stage::new();
        let mob = stage.add(Mobject::from_buffer(buffer));
        stage.add_to_scene(mob).expect("live");
        let mut plan = RenderPlan::new();
        sync_valid(&mut plan, &stage, 0);

        let metadata_view = stage
            .get_mut(mob)
            .expect("live")
            .buffer
            .export_field_view("user_metadata", true)
            .expect("custom field");
        let stats = sync_valid(&mut plan, &stage, 0);
        assert_eq!(stats.shapes_reused, 1);
        assert_eq!(stats.styles_rebuilt, 0);
        assert!(!plan.shapes().instances()[0].volatile);
        drop(metadata_view);
    }

    #[test]
    fn n_copies_of_one_outline_are_one_compiled_path_and_n_instances() {
        // §10.8's instancing dedup, stated as the acceptance criterion does.
        let (stage, _) = staged(8);
        let mut plan = RenderPlan::new();
        sync_valid(&mut plan, &stage, 0);
        assert_eq!(plan.shapes().shapes().len(), 1);
        assert_eq!(plan.shapes().instances().len(), 8);
        assert!((plan.shapes().instances_per_shape() - 8.0).abs() < 1e-12);
        // And the segment table holds one outline's worth, not eight.
        assert_eq!(plan.segments().len(), 2);
    }

    #[test]
    fn a_recolour_rebuilds_the_style_and_not_the_geometry() {
        // The §10.8 headline: "a color change does not regenerate curve
        // coefficients".
        let (mut stage, mobs) = staged(2);
        let mut plan = RenderPlan::new();
        sync_valid(&mut plan, &stage, 0);

        stage
            .get_mut(mobs[0])
            .expect("live")
            .buffer
            .write(0, "stroke_rgba", &[1.0, 0.0, 0.0, 1.0]);

        let after = sync_valid(&mut plan, &stage, 0);
        assert_eq!(after.shapes_compiled, 0, "no outline may recompile");
        assert_eq!(after.shapes_reused, 2);
        assert_eq!(after.styles_rebuilt, 1, "exactly the one that changed");
        assert_eq!(plan.styles().len(), 2, "and it interned as a new row");
    }

    #[test]
    fn a_camera_move_rebuilds_nothing() {
        // "a camera move does not re-decode font outlines" — object-space
        // geometry cannot depend on the camera, so the axis must not reach it.
        let (stage, _) = staged(4);
        let mut plan = RenderPlan::new();
        sync_valid(&mut plan, &stage, 0);
        let after = sync_valid(&mut plan, &stage, 99);
        assert_eq!(after.shapes_compiled, 0);
        assert_eq!(after.styles_rebuilt, 0);
        assert_eq!(after.shapes_reused, 4);
    }

    #[test]
    fn affine_placement_reuses_the_object_space_outline() {
        let (mut stage, mobs) = staged(1);
        let mob = mobs[0];
        let point_revision = stage.get(mob).unwrap().buffer.field_revision("point");
        let mut plan = RenderPlan::new();
        sync_valid(&mut plan, &stage, 0);
        let geometry = plan.geometry_identity();

        stage.shift(mob, [7.0, -3.0, 0.5]);
        let translated = sync_valid(&mut plan, &stage, 0);
        assert_eq!(translated.shapes_reused, 1);
        assert_eq!(translated.shapes_compiled, 0);
        assert_eq!(translated.shapes_shared, 0);
        assert_eq!(plan.geometry_identity(), geometry);
        assert_eq!(
            stage.get(mob).unwrap().buffer.field_revision("point"),
            point_revision
        );
        assert_eq!(
            plan.shapes().instances()[0].placement.translation(),
            [7.0, -3.0, 0.5]
        );

        stage.rotate(
            mob,
            std::f64::consts::FRAC_PI_2,
            [0.0, 0.0, 1.0],
            Some([7.0, -3.0, 0.5]),
            None,
        );
        let rotated = sync_valid(&mut plan, &stage, 0);
        assert_eq!(
            rotated.shapes_reused, 1,
            "linear placement is screen-derived state, not a reshape"
        );
        assert_eq!(rotated.shapes_compiled, 0);
        assert_eq!(plan.geometry_identity(), geometry);
    }

    #[test]
    fn moving_one_point_recompiles_exactly_one_outline() {
        let (mut stage, mobs) = staged(3);
        let mut plan = RenderPlan::new();
        sync_valid(&mut plan, &stage, 0);

        stage
            .get_mut(mobs[1])
            .expect("live")
            .buffer
            .write(2, "point", &[500.0, 500.0, 0.0]);

        let after = sync_valid(&mut plan, &stage, 0);
        assert_eq!(after.shapes_reused, 2, "the untouched two are reused");
        assert_eq!(
            after.shapes_compiled + after.shapes_shared,
            1,
            "and exactly one is recompiled"
        );
        assert_eq!(
            plan.shapes().shapes().len(),
            2,
            "a second outline exists now"
        );
    }

    #[test]
    fn a_z_index_change_reuses_every_outline_but_reorders_the_instances() {
        // Order is its own axis: the geometry cannot care, and the instance list
        // must.
        let (mut stage, mobs) = staged(3);
        let mut plan = RenderPlan::new();
        sync_valid(&mut plan, &stage, 0);
        let before: Vec<Mob> = plan.shapes().instances().iter().map(|i| i.mob).collect();

        stage.set_z_index(mobs[0], 10, false);
        stage.add_to_scene(mobs[0]).expect("live");

        let after = sync_valid(&mut plan, &stage, 0);
        assert_eq!(after.shapes_compiled, 0);
        assert_eq!(after.shapes_reused, 3);
        let now: Vec<Mob> = plan.shapes().instances().iter().map(|i| i.mob).collect();
        assert_ne!(before, now, "raising z_index must move it in painter order");
        assert_eq!(now.last(), Some(&mobs[0]), "and put it on top");
    }

    #[test]
    fn leaving_the_scene_drops_the_retained_entry() {
        let (mut stage, mobs) = staged(3);
        let mut plan = RenderPlan::new();
        sync_valid(&mut plan, &stage, 0);

        stage.remove_from_scene(mobs[2]);
        let after = sync_valid(&mut plan, &stage, 0);
        assert_eq!(after.visited, 2);
        assert_eq!(after.dropped, 1);
        assert_eq!(plan.shapes().instances().len(), 2);
    }

    #[test]
    fn instances_carry_painter_order_and_the_placement_that_positions_them() {
        let (stage, mobs) = staged(3);
        let mut plan = RenderPlan::new();
        sync_valid(&mut plan, &stage, 0);
        let instances = plan.shapes().instances();
        assert_eq!(instances.len(), 3);
        for (i, inst) in instances.iter().enumerate() {
            assert_eq!(inst.order, u32::try_from(i).expect("small fixture"));
            assert_eq!(inst.mob, mobs[i]);
            // The outline is shared, so the placement is what distinguishes the
            // three — which is the whole reason position is excluded from the
            // shape digest.
            assert!((inst.placement.translation()[0] - i as f64 * 10.0).abs() < 1e-9);
        }
        assert_eq!(instances[0].shape, instances[2].shape);
    }

    #[test]
    fn interned_indices_are_stable_across_syncs() {
        // The property the tile cache's key rests on (`crate::cache`): within a
        // retained-plan epoch, shape and style indices come from append-only
        // interning, so adding an object cannot renumber existing rows. A named
        // safe-point compaction may advance the epoch and invalidate derived
        // artifacts; ordinary syncs never do.
        let (mut stage, mobs) = staged(2);
        let mut plan = RenderPlan::new();
        sync_valid(&mut plan, &stage, 0);
        let shapes: Vec<u32> = plan.shapes().instances().iter().map(|i| i.shape).collect();
        let styles: Vec<u32> = plan.shapes().instances().iter().map(|i| i.style).collect();

        // A newcomer *behind* the others, so it lands first in the draw sequence
        // and shifts every instance index.
        let newcomer = stage.add(vmobject(&[
            [0.0, 0.0, 0.0],
            [3.0, 1.0, 0.0],
            [6.0, 0.0, 0.0],
        ]));
        stage.set_z_index(newcomer, -5, false);
        stage.add_to_scene(newcomer).expect("live");
        sync_valid(&mut plan, &stage, 0);

        let instances = plan.shapes().instances();
        assert_eq!(instances.len(), 3);
        assert_eq!(instances[0].mob, newcomer, "the newcomer draws first");
        for (k, mob) in mobs.iter().enumerate() {
            let inst = instances
                .iter()
                .find(|i| i.mob == *mob)
                .expect("still in the scene");
            assert_eq!(inst.shape, shapes[k], "shape index moved for {mob:?}");
            assert_eq!(inst.style, styles[k], "style index moved for {mob:?}");
        }
    }

    #[test]
    fn a_point_less_group_contributes_no_instance() {
        // A `Group` with children but no geometry of its own is normal, and it
        // must not appear as a degenerate compiled path.
        let mut stage = Stage::new();
        let group = stage.add(Mobject::new());
        stage.add_to_scene(group).expect("live");
        let mut plan = RenderPlan::new();
        let stats = sync_valid(&mut plan, &stage, 0);
        assert_eq!(stats.shapes_compiled, 0);
        assert_eq!(plan.shapes().instances().len(), 0);
    }
}
