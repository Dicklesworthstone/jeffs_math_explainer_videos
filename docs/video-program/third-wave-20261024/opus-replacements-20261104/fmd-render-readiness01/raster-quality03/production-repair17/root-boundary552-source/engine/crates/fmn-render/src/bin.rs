//! Two-level binning, per-tile command lists, and painter-order-safe occlusion
//! pruning (§10.8, §10.5).
//!
//! > Binning is **two-level**: object bounds → ~128×128 macrotiles →
//! > engine-sized fine tiles → stable per-tile command runs, sizes tuned per
//! > platform by the execution plan (§17.4), never by semantics. Command lists
//! > keep stable draw indices; transparent runs stay in exact painter order;
//! > annex binning uses deterministic count/prefix/scatter or a stable key sort,
//! > never unordered atomic appends.
//!
//! ## Order-stable by construction, not by discipline
//!
//! The determinism obligation (§10.5) is that binning output is identical at any
//! thread count. That is met here structurally rather than by locking: the
//! algorithm is a **count → prefix-sum → scatter** over instances visited in
//! painter order, so a tile's run ascends in instance index whatever partition
//! produced it, and [`Binning::build_partitioned`] takes the partition count as a
//! parameter precisely so the property is testable without threads. A partition
//! is a *slice of the instance list*, and each partition's contribution to each
//! tile is placed at an offset computed from the partitions before it — so no
//! result depends on which partition finished first.
//!
//! G0-8 proved the same CSR shape end-to-end on Metal, where the property
//! matters more: the kernel only *reads* `draws[offsets[t]..offsets[t+1]]` in
//! index order, so painter order is structural and there is no ordering decision
//! at dispatch time to get wrong.
//!
//! ## Why two levels
//!
//! One level makes every instance test every fine tile its bounds touch. A
//! 1080p frame at 16-px tiles is 8 160 tiles, and a full-screen background tests
//! all of them. The macrotile pass cuts that by the square of the ratio: an
//! instance is tested against ~128-px macrotiles first, and the fine pass only
//! considers instances the macrotile already accepted.
//!
//! The sizes come from §17.4's `ExecutionPlan` — which does not exist yet
//! (`fmn-runtime` is a skeleton), so [`Tiling`] carries defaults and takes them
//! as parameters. That is the right shape regardless: §17.4 is explicit that
//! tile dimensions are a *scheduling* choice, and the size-sweep test here is
//! what makes "tuning changes speed, never semantics" a checked claim rather
//! than an intention.
//!
//! ## Pruning has a proof obligation per skip
//!
//! The bead's rule is absolute: **when in doubt, draw.** A command is skipped
//! only when a later command in the same tile provably covers the whole tile
//! opaquely, which means all three of: the tile is geometrically inside the
//! later shape, the later shape's fill is opaque at both ramp ends, and the
//! coverage there is exactly `1` rather than an accumulation that lands near it
//! (G0-8b's finding F13 — without that last clause the pruned and unpruned
//! frames differ in the last bit of a channel and "provably unchanged" is
//! false). The interior classification and the pruning are therefore one
//! mechanism, and they ship together.

use crate::hint::Hint;
use crate::plan::{PlanIdentity, RenderPlan};
use crate::table::{Instance, Shape};
use fmn_core::types::Vec3;
use fmn_mobject::Placement;

/// The command touches the tile's edge: coverage must be evaluated.
pub const CLASS_PARTIAL: u32 = 0;
/// The tile lies wholly inside the shape: coverage is `1` everywhere in it.
///
/// §10.4's "fully covered" tile class. Two things follow, and the second is the
/// one that is easy to miss: the per-pixel evaluation disappears for that
/// command, **and** its coverage becomes exactly `1` instead of an accumulation
/// landing within an ulp of it — which is the precondition that makes occlusion
/// pruning bit-exact (G0-8b/F13).
pub const CLASS_INTERIOR: u32 = 1;

/// The object→screen mapping a frame is binned under.
///
/// Deliberately a uniform scale plus an origin rather than a camera. §10.4's
/// camera, its projection conventions and `is_fixed_in_frame` are not in the
/// tree, and inventing a `Projection` here would create a second camera to
/// reconcile later — the failure this crate already avoided for the transform
/// revision channel (fm-7if).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenMap {
    /// Object units per pixel, inverted: pixels per object unit.
    pub scale: f64,
    /// Where object-space `[0, 0]` lands, in pixels.
    pub origin: [f64; 2],
    /// Object-space +Y points **up** the frame. Frames are top-row-first
    /// (D-23: no vflip anywhere), so every scene front door — manim's +Y-up
    /// plane — sets this. `false` is a raw y-down object space (pixel-aligned
    /// rasterization fixtures); a scene rendered that way is vertically
    /// mirrored, which is what fm-sq8.9 fixed. Journaled with the map.
    pub y_up: bool,
}

impl Default for ScreenMap {
    fn default() -> Self {
        ScreenMap {
            scale: 1.0,
            origin: [0.0, 0.0],
            y_up: false,
        }
    }
}

impl ScreenMap {
    /// Map a **shape-local** point through its instance placement to pixels.
    ///
    /// Both arguments are required, and that is the point: compiled outlines are
    /// shape-local so that instancing can share them, so nothing downstream may
    /// read a shape's coordinates without the affine map that places it.
    #[must_use]
    pub fn place(&self, p: Vec3, placement: Placement) -> [f64; 2] {
        let world = placement.apply_point(p);
        self.to_pixel(world[0], world[1])
    }

    /// Pixels per object unit along +Y: negative when +Y points up a
    /// top-row-first frame.
    #[must_use]
    pub fn y_scale(&self) -> f64 {
        if self.y_up { -self.scale } else { self.scale }
    }

    /// A manim-oriented map: +Y up, object `[0, 0]` at `origin`.
    #[must_use]
    pub fn y_up(scale: f64, origin: [f64; 2]) -> Self {
        ScreenMap {
            scale,
            origin,
            y_up: true,
        }
    }

    /// Object space → pixels. This, [`Self::delta_to_pixel`] and
    /// [`Self::to_object`] are the one definition of the mapping; renderer
    /// stages must not re-derive it (inlined copies once ignored the frame's
    /// orientation and rendered every 2D scene upside down).
    #[must_use]
    pub fn to_pixel(&self, x: f64, y: f64) -> [f64; 2] {
        [
            self.origin[0] + x * self.scale,
            self.origin[1] + y * self.y_scale(),
        ]
    }

    /// An object-space translation expressed in pixels.
    #[must_use]
    pub fn delta_to_pixel(&self, dx: f64, dy: f64) -> [f64; 2] {
        [dx * self.scale, dy * self.y_scale()]
    }

    /// Pixels → object space, the inverse of [`Self::to_pixel`] after
    /// removing a pixel-space `translate` (see [`Self::delta_to_pixel`]).
    #[must_use]
    pub fn to_object(&self, px: [f64; 2], translate: [f64; 2]) -> [f64; 2] {
        [
            (px[0] - self.origin[0] - translate[0]) / self.scale,
            (px[1] - self.origin[1] - translate[1]) / self.y_scale(),
        ]
    }
}

/// §10.8's two-level tiling.
///
/// Both sizes are scheduling choices (§17.4), never semantics — which the
/// size-sweep test in this module checks rather than assumes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tiling {
    /// Macrotile edge in pixels. §10.8 says "~128×128".
    pub macro_tile: u32,
    /// Fine tile edge in pixels — the engine's tile, and on the annex the
    /// threadgroup (G0-8 finding F4).
    pub fine_tile: u32,
}

impl Default for Tiling {
    fn default() -> Self {
        Tiling {
            macro_tile: 128,
            fine_tile: 16,
        }
    }
}

/// The pixel rectangle a frame covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Viewport {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

/// Hard bounds for the temporary and retained tables built by [`Binning`].
///
/// The defaults comfortably cover 8K frames at the standard 16-pixel tile
/// size and the certified 96-worker profile, while refusing scheduling choices
/// whose table products would dominate the frame itself. Callers with an
/// intentionally larger, provisioned workload can pass explicit limits through
/// [`Binning::build_with_limits`] or
/// [`Binning::build_partitioned_with_limits`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BinningLimits {
    /// Maximum fine or coarse tiles in one grid.
    pub max_tiles: u64,
    /// Maximum painter-ordered instances in one plan.
    pub max_instances: u64,
    /// Maximum fine or coarse instance-to-tile references.
    pub max_tile_references: u64,
    /// Maximum cells across the partition-by-fine-tile count table.
    pub max_partition_cells: u64,
    /// Maximum conservatively estimated bytes across all binning tables.
    pub max_working_bytes: u64,
}

impl Default for BinningLimits {
    fn default() -> Self {
        Self {
            max_tiles: 1 << 20,
            max_instances: 1 << 20,
            max_tile_references: 1 << 24,
            max_partition_cells: 1 << 24,
            max_working_bytes: 256 * 1024 * 1024,
        }
    }
}

/// A uniform grid over the viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Grid {
    tile: u32,
    cols: u32,
    rows: u32,
    count: usize,
}

impl Grid {
    fn new(
        viewport: Viewport,
        tile: u32,
        level: &'static str,
        limits: BinningLimits,
    ) -> Result<Grid, BinningError> {
        let tile = tile.max(1);
        let cols = viewport.width.div_ceil(tile);
        let rows = viewport.height.div_ceil(tile);
        let count = u64::from(cols)
            .checked_mul(u64::from(rows))
            .ok_or(BinningError::CountOverflow { resource: level })?;
        check_limit(level, count, limits.max_tiles)?;
        check_limit(level, count, u64::from(u32::MAX))?;
        let count = usize::try_from(count).map_err(|_| BinningError::AddressSpace {
            resource: level,
            requested: count,
        })?;
        Ok(Grid {
            tile,
            cols,
            rows,
            count,
        })
    }

    fn count(&self) -> usize {
        self.count
    }

    /// The inclusive tile-index span an AABB touches, clamped to the grid.
    ///
    /// Returns `None` for a box entirely off-grid, which must produce no command
    /// at all rather than a clamped one — the difference between "not visible"
    /// and "drawn at the edge".
    fn span(&self, aabb: [f64; 4]) -> Option<(u32, u32, u32, u32)> {
        if self.cols == 0 || self.rows == 0 {
            return None;
        }
        let w = f64::from(self.cols) * f64::from(self.tile);
        let h = f64::from(self.rows) * f64::from(self.tile);
        if aabb[2] < 0.0 || aabb[3] < 0.0 || aabb[0] > w || aabb[1] > h {
            return None;
        }
        let t = self.tile as f64;
        let lo = |v: f64, n: u32| ((v / t).floor().max(0.0) as u32).min(n.saturating_sub(1));
        let hi = |v: f64, n: u32| ((v / t).floor().max(0.0) as u32).min(n.saturating_sub(1));
        Some((
            lo(aabb[0], self.cols),
            lo(aabb[1], self.rows),
            hi(aabb[2], self.cols),
            hi(aabb[3], self.rows),
        ))
    }

    fn rect(&self, index: usize, viewport: Viewport) -> [f64; 4] {
        let tx = (index as u32 % self.cols) * self.tile;
        let ty = (index as u32 / self.cols) * self.tile;
        [
            f64::from(tx),
            f64::from(ty),
            f64::from(tx.saturating_add(self.tile).min(viewport.width)),
            f64::from(ty.saturating_add(self.tile).min(viewport.height)),
        ]
    }
}

/// What a pruning pass removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PruneReport {
    /// Commands before pruning.
    pub before: usize,
    /// Commands after.
    pub after: usize,
    /// Tiles in which at least one command was dropped.
    pub tiles_touched: usize,
}

impl PruneReport {
    /// The fraction of commands removed.
    #[must_use]
    pub fn removed_fraction(&self) -> f64 {
        if self.before == 0 {
            return 0.0;
        }
        (self.before - self.after) as f64 / self.before as f64
    }
}

/// A binning build or pruning pass could not preserve its resource or identity
/// contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinningError {
    /// A declared count exceeded its explicit workload limit.
    LimitExceeded {
        /// The table or work axis being bounded.
        resource: &'static str,
        /// Count or bytes requested.
        requested: u64,
        /// Configured maximum.
        limit: u64,
    },
    /// Arithmetic for a table or work count overflowed before allocation.
    CountOverflow {
        /// The table or work axis whose count overflowed.
        resource: &'static str,
    },
    /// A representable wire count does not fit this target's address space.
    AddressSpace {
        /// The table or work axis being converted.
        resource: &'static str,
        /// Elements requested.
        requested: u64,
    },
    /// The allocator refused a preflighted table reservation.
    AllocationFailed {
        /// The table being reserved.
        resource: &'static str,
        /// Elements requested.
        requested: u64,
    },
    /// The painter-ordered plan no longer matches the one whose command lists
    /// this binning stores.
    PlanMismatch,
}

impl std::fmt::Display for BinningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LimitExceeded {
                resource,
                requested,
                limit,
            } => write!(
                f,
                "binning {resource} needs {requested}, exceeding the limit {limit}"
            ),
            Self::CountOverflow { resource } => {
                write!(f, "binning {resource} count overflowed")
            }
            Self::AddressSpace {
                resource,
                requested,
            } => write!(
                f,
                "binning {resource} count {requested} exceeds this target address space"
            ),
            Self::AllocationFailed {
                resource,
                requested,
            } => write!(
                f,
                "could not reserve {requested} elements for binning {resource}"
            ),
            Self::PlanMismatch => f.write_str("binning was derived from a different render plan"),
        }
    }
}

impl std::error::Error for BinningError {}

fn check_limit(resource: &'static str, requested: u64, limit: u64) -> Result<(), BinningError> {
    if requested > limit {
        return Err(BinningError::LimitExceeded {
            resource,
            requested,
            limit,
        });
    }
    Ok(())
}

fn checked_usize(resource: &'static str, requested: u64) -> Result<usize, BinningError> {
    usize::try_from(requested).map_err(|_| BinningError::AddressSpace {
        resource,
        requested,
    })
}

fn try_with_capacity<T>(resource: &'static str, requested: usize) -> Result<Vec<T>, BinningError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(requested)
        .map_err(|_| BinningError::AllocationFailed {
            resource,
            requested: u64::try_from(requested).unwrap_or(u64::MAX),
        })?;
    Ok(values)
}

fn try_filled<T: Clone>(
    resource: &'static str,
    requested: usize,
    value: T,
) -> Result<Vec<T>, BinningError> {
    let mut values = try_with_capacity(resource, requested)?;
    values.resize(requested, value);
    Ok(values)
}

fn checked_product(resource: &'static str, left: u64, right: u64) -> Result<u64, BinningError> {
    left.checked_mul(right)
        .ok_or(BinningError::CountOverflow { resource })
}

fn checked_sum(resource: &'static str, left: u64, right: u64) -> Result<u64, BinningError> {
    left.checked_add(right)
        .ok_or(BinningError::CountOverflow { resource })
}

fn table_bytes<T>(resource: &'static str, count: u64) -> Result<u64, BinningError> {
    checked_product(
        resource,
        count,
        u64::try_from(std::mem::size_of::<T>()).unwrap_or(u64::MAX),
    )
}

fn add_table_bytes<T>(
    total: &mut u64,
    resource: &'static str,
    count: u64,
) -> Result<(), BinningError> {
    *total = checked_sum(resource, *total, table_bytes::<T>(resource, count)?)?;
    Ok(())
}

fn span_cells(span: (u32, u32, u32, u32)) -> Result<u64, BinningError> {
    let (x0, y0, x1, y1) = span;
    let cols = u64::from(x1 - x0) + 1;
    let rows = u64::from(y1 - y0) + 1;
    checked_product("tile references", cols, rows)
}

/// Per-fine-tile command lists in CSR form, with §10.4's class per command.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Binning {
    tiling: Tiling,
    viewport: Viewport,
    map: ScreenMap,
    plan: PlanIdentity,
    fine: Grid,
    /// `fine.count() + 1` entries; tile `i` owns `draws[offsets[i]..offsets[i+1]]`.
    offsets: Vec<u32>,
    /// Instance indices, **ascending within a tile** — which is painter order,
    /// because the instance list is painter order.
    draws: Vec<u32>,
    /// One class word per entry of `draws`.
    ///
    /// §10.8's command runs carry "stable draw indices"; G0-8b's F13 is that
    /// they want a flag word beside each index, because interior-vs-edge is a
    /// property of a *(shape, tile)* pair rather than of a tile.
    flags: Vec<u32>,
    /// Macrotiles that held at least one instance — the work the fine pass
    /// skipped, recorded because a two-level scheme that never prunes anything
    /// is just a slower one-level scheme.
    macro_hits: usize,
    macro_count: usize,
}

impl Binning {
    /// Bin a synchronized plan, serially.
    ///
    /// # Errors
    /// [`BinningError`] if any derived table exceeds the default resource
    /// limits or cannot be reserved.
    pub fn build(
        plan: &RenderPlan,
        viewport: Viewport,
        tiling: Tiling,
        map: ScreenMap,
    ) -> Result<Binning, BinningError> {
        Binning::build_with_limits(plan, viewport, tiling, map, BinningLimits::default())
    }

    /// Bin a synchronized plan under explicit resource limits.
    ///
    /// # Errors
    /// [`BinningError`] if a grid, membership table, partition table, working
    /// set, or allocator reservation exceeds `limits`.
    pub fn build_with_limits(
        plan: &RenderPlan,
        viewport: Viewport,
        tiling: Tiling,
        map: ScreenMap,
        limits: BinningLimits,
    ) -> Result<Binning, BinningError> {
        Binning::build_partitioned_with_limits(plan, viewport, tiling, map, 1, limits)
    }

    /// Bin a synchronized plan as if across `partitions` workers.
    ///
    /// The partition count is the §10.5 determinism obligation made testable
    /// without threads: a partition is a contiguous slice of the painter-ordered
    /// instance list, each partition's per-tile counts are summed **in partition
    /// order**, and each instance is scattered to a slot derived from that sum.
    /// Nothing depends on which partition would have finished first, so the
    /// output is byte-identical at every partition count — including counts that
    /// do not divide the work evenly.
    pub fn build_partitioned(
        plan: &RenderPlan,
        viewport: Viewport,
        tiling: Tiling,
        map: ScreenMap,
        partitions: usize,
    ) -> Result<Binning, BinningError> {
        Binning::build_partitioned_with_limits(
            plan,
            viewport,
            tiling,
            map,
            partitions,
            BinningLimits::default(),
        )
    }

    /// Partitioned binning under explicit resource limits.
    ///
    /// Empty partitions do not affect output, so a requested partition count
    /// above the number of instances is normalized away before table sizing.
    ///
    /// # Errors
    /// [`BinningError`] before any proportional allocation or tile traversal
    /// whose declared work exceeds `limits`.
    pub fn build_partitioned_with_limits(
        plan: &RenderPlan,
        viewport: Viewport,
        tiling: Tiling,
        map: ScreenMap,
        partitions: usize,
        limits: BinningLimits,
    ) -> Result<Binning, BinningError> {
        let fine = Grid::new(viewport, tiling.fine_tile, "fine tiles", limits)?;
        let coarse = Grid::new(
            viewport,
            tiling.macro_tile.max(tiling.fine_tile),
            "macro tiles",
            limits,
        )?;
        let instances = plan.shapes().instances();
        let instance_count = u64::try_from(instances.len()).unwrap_or(u64::MAX);
        check_limit("instances", instance_count, limits.max_instances)?;
        check_limit("instances", instance_count, u64::from(u32::MAX))?;
        let partitions = partitions.max(1).min(instances.len().max(1));
        let partition_count = u64::try_from(partitions).unwrap_or(u64::MAX);
        let fine_count = u64::try_from(fine.count()).unwrap_or(u64::MAX);
        let partition_cells = checked_product("partition cells", partition_count, fine_count)?;
        check_limit(
            "partition cells",
            partition_cells,
            limits.max_partition_cells,
        )?;

        let mut initial_bytes = 0u64;
        add_table_bytes::<Option<[f64; 4]>>(&mut initial_bytes, "screen bounds", instance_count)?;
        add_table_bytes::<usize>(
            &mut initial_bytes,
            "macrotile counts",
            u64::try_from(coarse.count()).unwrap_or(u64::MAX),
        )?;
        check_limit("working bytes", initial_bytes, limits.max_working_bytes)?;

        // Screen AABBs, once per instance. Object bounds are the control-polygon
        // hull, which contains the curve, so this is conservative by
        // construction rather than by a fudge factor.
        let mut boxes = try_with_capacity("screen bounds", instances.len())?;
        boxes.extend(instances.iter().map(|inst| screen_aabb(plan, inst, map)));

        // ---- level one: macrotiles. Nothing is emitted from this pass; it
        // exists so the fine pass sees a short list per region.
        let mut macro_counts = try_filled("macrotile counts", coarse.count(), 0usize)?;
        let mut macro_reference_count = 0u64;
        for aabb in &boxes {
            let Some(aabb) = aabb else { continue };
            let Some((x0, y0, x1, y1)) = coarse.span(*aabb) else {
                continue;
            };
            macro_reference_count = checked_sum(
                "macrotile references",
                macro_reference_count,
                span_cells((x0, y0, x1, y1))?,
            )?;
            check_limit(
                "macrotile references",
                macro_reference_count,
                limits.max_tile_references,
            )?;
            for ty in y0..=y1 {
                for tx in x0..=x1 {
                    let index = ty as usize * coarse.cols as usize + tx as usize;
                    macro_counts[index] =
                        macro_counts[index]
                            .checked_add(1)
                            .ok_or(BinningError::CountOverflow {
                                resource: "macrotile member count",
                            })?;
                }
            }
        }
        let macro_hits = macro_counts.iter().filter(|&&count| count != 0).count();

        let mut preliminary_bytes = initial_bytes;
        add_table_bytes::<Vec<u32>>(
            &mut preliminary_bytes,
            "macrotile rows",
            u64::try_from(coarse.count()).unwrap_or(u64::MAX),
        )?;
        add_table_bytes::<u32>(
            &mut preliminary_bytes,
            "macrotile references",
            macro_reference_count,
        )?;
        check_limit("working bytes", preliminary_bytes, limits.max_working_bytes)?;

        let mut macro_members = try_filled("macrotile rows", coarse.count(), Vec::<u32>::new())?;
        for (members, &count) in macro_members.iter_mut().zip(&macro_counts) {
            members
                .try_reserve_exact(count)
                .map_err(|_| BinningError::AllocationFailed {
                    resource: "macrotile references",
                    requested: u64::try_from(count).unwrap_or(u64::MAX),
                })?;
        }
        for (i, aabb) in boxes.iter().enumerate() {
            let Some(aabb) = aabb else { continue };
            let Some((x0, y0, x1, y1)) = coarse.span(*aabb) else {
                continue;
            };
            let instance = u32::try_from(i).map_err(|_| BinningError::AddressSpace {
                resource: "instance index",
                requested: u64::try_from(i).unwrap_or(u64::MAX),
            })?;
            for ty in y0..=y1 {
                for tx in x0..=x1 {
                    let index = ty as usize * coarse.cols as usize + tx as usize;
                    macro_members[index].push(instance);
                }
            }
        }

        // Which fine tiles each instance actually lands in — derived through the
        // macrotiles, so an instance is only tested against fine tiles inside
        // macrotiles that already accepted it.
        let mut counting_bytes = preliminary_bytes;
        add_table_bytes::<usize>(&mut counting_bytes, "instance tile counts", instance_count)?;
        check_limit("working bytes", counting_bytes, limits.max_working_bytes)?;
        let mut fine_counts = try_filled("instance tile counts", instances.len(), 0usize)?;
        let mut fine_reference_count = 0u64;
        for (m, members) in macro_members.iter().enumerate() {
            if members.is_empty() {
                continue;
            }
            let mrect = coarse.rect(m, viewport);
            let fx0 = (mrect[0] / f64::from(fine.tile)).floor() as u32;
            let fy0 = (mrect[1] / f64::from(fine.tile)).floor() as u32;
            let fx1 = (((mrect[2] - 1.0).max(mrect[0]) / f64::from(fine.tile)).floor() as u32)
                .min(fine.cols.saturating_sub(1));
            let fy1 = (((mrect[3] - 1.0).max(mrect[1]) / f64::from(fine.tile)).floor() as u32)
                .min(fine.rows.saturating_sub(1));
            for &i in members {
                let Some(aabb) = boxes[i as usize] else {
                    continue;
                };
                let Some((x0, y0, x1, y1)) = fine.span(aabb) else {
                    continue;
                };
                let span = (x0.max(fx0), y0.max(fy0), x1.min(fx1), y1.min(fy1));
                if span.0 > span.2 || span.1 > span.3 {
                    continue;
                }
                let cells = span_cells(span)?;
                fine_reference_count =
                    checked_sum("fine tile references", fine_reference_count, cells)?;
                check_limit(
                    "fine tile references",
                    fine_reference_count,
                    limits.max_tile_references,
                )?;
                let index = i as usize;
                fine_counts[index] = fine_counts[index]
                    .checked_add(checked_usize("fine tile references", cells)?)
                    .ok_or(BinningError::CountOverflow {
                        resource: "instance tile count",
                    })?;
            }
        }
        check_limit(
            "fine tile references",
            fine_reference_count,
            u64::from(u32::MAX),
        )?;

        let mut working_bytes = counting_bytes;
        add_table_bytes::<Vec<u32>>(&mut working_bytes, "instance tile rows", instance_count)?;
        add_table_bytes::<u32>(
            &mut working_bytes,
            "fine tile references",
            fine_reference_count,
        )?;
        add_table_bytes::<(usize, usize)>(&mut working_bytes, "partition ranges", partition_count)?;
        add_table_bytes::<Vec<u32>>(&mut working_bytes, "partition count rows", partition_count)?;
        add_table_bytes::<u32>(&mut working_bytes, "partition count cells", partition_cells)?;
        add_table_bytes::<Vec<u32>>(&mut working_bytes, "partition cursor rows", partition_count)?;
        add_table_bytes::<u32>(
            &mut working_bytes,
            "partition cursor cells",
            partition_cells,
        )?;
        add_table_bytes::<u32>(
            &mut working_bytes,
            "tile offsets",
            checked_sum("tile offsets", fine_count, 1)?,
        )?;
        add_table_bytes::<u32>(&mut working_bytes, "draw indices", fine_reference_count)?;
        add_table_bytes::<u32>(&mut working_bytes, "class flags", fine_reference_count)?;
        check_limit("working bytes", working_bytes, limits.max_working_bytes)?;

        let mut per_instance =
            try_filled("instance tile rows", instances.len(), Vec::<u32>::new())?;
        for (tiles, &count) in per_instance.iter_mut().zip(&fine_counts) {
            tiles
                .try_reserve_exact(count)
                .map_err(|_| BinningError::AllocationFailed {
                    resource: "fine tile references",
                    requested: u64::try_from(count).unwrap_or(u64::MAX),
                })?;
        }
        for (m, members) in macro_members.iter().enumerate() {
            if members.is_empty() {
                continue;
            }
            let mrect = coarse.rect(m, viewport);
            let fx0 = (mrect[0] / f64::from(fine.tile)).floor() as u32;
            let fy0 = (mrect[1] / f64::from(fine.tile)).floor() as u32;
            let fx1 = (((mrect[2] - 1.0).max(mrect[0]) / f64::from(fine.tile)).floor() as u32)
                .min(fine.cols.saturating_sub(1));
            let fy1 = (((mrect[3] - 1.0).max(mrect[1]) / f64::from(fine.tile)).floor() as u32)
                .min(fine.rows.saturating_sub(1));
            for &i in members {
                let Some(aabb) = boxes[i as usize] else {
                    continue;
                };
                let Some((x0, y0, x1, y1)) = fine.span(aabb) else {
                    continue;
                };
                for ty in y0.max(fy0)..=y1.min(fy1) {
                    for tx in x0.max(fx0)..=x1.min(fx1) {
                        let tile = u64::from(ty) * u64::from(fine.cols) + u64::from(tx);
                        per_instance[i as usize].push(u32::try_from(tile).map_err(|_| {
                            BinningError::AddressSpace {
                                resource: "fine tile index",
                                requested: tile,
                            }
                        })?);
                    }
                }
            }
        }

        // ---- count → prefix → scatter, partitioned.
        let n = fine.count();
        let chunk = instances.len().div_ceil(partitions).max(1);
        let mut ranges = try_with_capacity("partition ranges", partitions)?;
        for p in 0..partitions {
            let lo = (p * chunk).min(instances.len());
            let hi = ((p + 1) * chunk).min(instances.len());
            ranges.push((lo, hi));
        }

        // counts[p][t]: how many commands partition p contributes to tile t.
        let mut counts = try_with_capacity("partition count rows", ranges.len())?;
        for _ in &ranges {
            counts.push(try_filled("partition count cells", n, 0u32)?);
        }
        for (p, &(lo, hi)) in ranges.iter().enumerate() {
            for tiles in per_instance.iter().take(hi).skip(lo) {
                for &t in tiles {
                    counts[p][t as usize] = counts[p][t as usize].checked_add(1).ok_or(
                        BinningError::CountOverflow {
                            resource: "commands per partition tile",
                        },
                    )?;
                }
            }
        }

        // Offsets: tiles in index order, and within a tile the partitions in
        // partition order. That second clause is what keeps a tile's run in
        // painter order at any partition count.
        let offset_count = n.checked_add(1).ok_or(BinningError::CountOverflow {
            resource: "tile offsets",
        })?;
        let mut offsets = try_with_capacity("tile offsets", offset_count)?;
        let mut cursors = try_with_capacity("partition cursor rows", ranges.len())?;
        for _ in &ranges {
            cursors.push(try_filled("partition cursor cells", n, 0u32)?);
        }
        let mut running = 0u32;
        offsets.push(0);
        for t in 0..n {
            for p in 0..ranges.len() {
                cursors[p][t] = running;
                running = running
                    .checked_add(counts[p][t])
                    .ok_or(BinningError::CountOverflow {
                        resource: "CSR command count",
                    })?;
            }
            offsets.push(running);
        }

        let draw_count = usize::try_from(running).map_err(|_| BinningError::AddressSpace {
            resource: "draw indices",
            requested: u64::from(running),
        })?;
        let mut draws = try_filled("draw indices", draw_count, 0u32)?;
        for (p, &(lo, hi)) in ranges.iter().enumerate() {
            for (i, tiles) in per_instance.iter().enumerate().take(hi).skip(lo) {
                for &t in tiles {
                    let slot = cursors[p][t as usize] as usize;
                    draws[slot] = u32::try_from(i).map_err(|_| BinningError::AddressSpace {
                        resource: "instance index",
                        requested: u64::try_from(i).unwrap_or(u64::MAX),
                    })?;
                    cursors[p][t as usize] = cursors[p][t as usize].checked_add(1).ok_or(
                        BinningError::CountOverflow {
                            resource: "partition cursor",
                        },
                    )?;
                }
            }
        }

        // ---- §10.4's class, per command.
        let mut flags = try_filled("class flags", draws.len(), CLASS_PARTIAL)?;
        for t in 0..n {
            let rect = fine.rect(t, viewport);
            for k in offsets[t] as usize..offsets[t + 1] as usize {
                let inst = &instances[draws[k] as usize];
                if covers_tile(plan, inst, map, rect) {
                    flags[k] = CLASS_INTERIOR;
                }
            }
        }

        Ok(Binning {
            tiling,
            viewport,
            map,
            plan: plan.identity(),
            fine,
            offsets,
            draws,
            flags,
            macro_hits,
            macro_count: coarse.count(),
        })
    }

    /// The tiling this binning used.
    #[must_use]
    pub fn tiling(&self) -> Tiling {
        self.tiling
    }

    /// The exact viewport this binning covered.
    #[must_use]
    pub(crate) fn viewport(&self) -> Viewport {
        self.viewport
    }

    /// The exact object-to-screen mapping used to classify and scatter draws.
    #[must_use]
    pub(crate) fn map(&self) -> ScreenMap {
        self.map
    }

    /// Whether every command index and class word still names the same
    /// painter-ordered draw.
    #[must_use]
    pub(crate) fn matches_plan(&self, plan: &RenderPlan) -> bool {
        self.plan == plan.identity()
    }

    /// Fine tiles in the grid.
    #[must_use]
    pub fn tile_count(&self) -> usize {
        self.fine.count()
    }

    /// The CSR offsets.
    #[must_use]
    pub fn offsets(&self) -> &[u32] {
        &self.offsets
    }

    /// The CSR instance indices.
    #[must_use]
    pub fn draws(&self) -> &[u32] {
        &self.draws
    }

    /// The per-command class words.
    #[must_use]
    pub fn flags(&self) -> &[u32] {
        &self.flags
    }

    /// One tile's command run, in painter order.
    ///
    /// Returns `None` when `index` is outside the fine grid.
    #[must_use]
    pub fn tile(&self, index: usize) -> Option<&[u32]> {
        self.draws.get(self.tile_span(index)?)
    }

    /// One tile's class words, parallel to [`Binning::tile`].
    ///
    /// Returns `None` when `index` is outside the fine grid.
    #[must_use]
    pub fn tile_flags(&self, index: usize) -> Option<&[u32]> {
        self.flags.get(self.tile_span(index)?)
    }

    /// The fine tile containing an in-viewport pixel.
    ///
    /// Returns `None` for an empty viewport or coordinates on or beyond its
    /// half-open right and lower edges. Checking the coordinates before folding
    /// them into a linear index prevents an out-of-range x coordinate from
    /// aliasing a valid tile in the next row.
    #[must_use]
    pub fn tile_of(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.viewport.width || y >= self.viewport.height {
            return None;
        }
        let tx = x / self.fine.tile;
        let ty = y / self.fine.tile;
        let index = u64::from(ty)
            .checked_mul(u64::from(self.fine.cols))?
            .checked_add(u64::from(tx))?;
        let index = usize::try_from(index).ok()?;
        (index < self.fine.count()).then_some(index)
    }

    fn tile_span(&self, index: usize) -> Option<std::ops::Range<usize>> {
        let next = index.checked_add(1)?;
        let start = usize::try_from(*self.offsets.get(index)?).ok()?;
        let end = usize::try_from(*self.offsets.get(next)?).ok()?;
        (start <= end).then_some(start..end)
    }

    /// How many macrotiles held anything, out of how many there are.
    ///
    /// The two-level scheme's own justification: a pass that accepts every
    /// macrotile is a slower one-level pass, and this is the number that says so.
    #[must_use]
    pub fn macro_occupancy(&self) -> (usize, usize) {
        (self.macro_hits, self.macro_count)
    }

    /// Drop commands that are provably invisible, keeping painter order.
    ///
    /// Walks each tile's run forward, remembers the **last** command that
    /// provably paints the whole tile opaque, and drops everything before it.
    /// The proof obligation per skip is [`covers_tile`] plus an opaque fill at
    /// both ramp ends plus the [`CLASS_INTERIOR`] classification that makes the
    /// covering command's coverage exactly `1`. Anything failing a clause is
    /// simply drawn.
    ///
    /// # Errors
    /// [`BinningError::PlanMismatch`] if `plan` is not the synchronized plan
    /// from which these command lists were built. The binning is left unchanged
    /// on error.
    pub fn prune_occluded(&mut self, plan: &RenderPlan) -> Result<PruneReport, BinningError> {
        if !self.matches_plan(plan) {
            return Err(BinningError::PlanMismatch);
        }
        let instances = plan.shapes().instances();
        let mut report = PruneReport {
            before: self.draws.len(),
            after: 0,
            tiles_touched: 0,
        };
        let mut offsets = try_with_capacity("pruned tile offsets", self.offsets.len())?;
        let mut draws = try_with_capacity("pruned draw indices", self.draws.len())?;
        let mut flags = try_with_capacity("pruned class flags", self.flags.len())?;
        offsets.push(0u32);

        for t in 0..self.fine.count() {
            let lo = self.offsets[t] as usize;
            let hi = self.offsets[t + 1] as usize;
            let mut start = lo;
            for k in lo..hi {
                if self.flags[k] == CLASS_INTERIOR
                    && is_opaque_fill(plan, &instances[self.draws[k] as usize])
                {
                    start = k;
                }
            }
            if start > lo {
                report.tiles_touched += 1;
            }
            draws.extend_from_slice(&self.draws[start..hi]);
            flags.extend_from_slice(&self.flags[start..hi]);
            offsets.push(
                u32::try_from(draws.len()).map_err(|_| BinningError::AddressSpace {
                    resource: "pruned draw indices",
                    requested: u64::try_from(draws.len()).unwrap_or(u64::MAX),
                })?,
            );
        }

        report.after = draws.len();
        self.offsets = offsets;
        self.draws = draws;
        self.flags = flags;
        Ok(report)
    }
}

/// One instance's conservative screen-space AABB.
///
/// Two things beyond placing the hull, and both are load-bearing.
///
/// **The per-draw width expansion (§10.3).** "Conservative slabs are the
/// retained control-polygon hull *plus a per-draw width expansion*, and the
/// expansion is deliberately not retained — width is a *style* property, so a
/// retained slab would be invalidated by a restyle." Without it a stroke is
/// binned to the tiles its *centreline* hull touches, and everything the
/// half-width and the antialiasing band reach beyond that is simply never
/// listed: a stroke whose outline ends on a tile boundary loses `w/2 + aa`
/// pixels of edge, cleanly and silently. It cost nothing until an engine
/// existed to draw the missing band, which is why this arrives with fm-ig3
/// rather than with the binner.
///
/// **Normalization.** A negative `ScreenMap::scale` — the natural way to spell
/// a y-flip — maps `bounds.min` above `bounds.max`, and an inverted AABB makes
/// [`Grid::span`]'s `lo..=hi` loops yield nothing at all, so every instance
/// vanishes from the frame. Taking componentwise min/max costs two comparisons
/// and removes a silent-drop.
fn screen_aabb(plan: &RenderPlan, inst: &Instance, map: ScreenMap) -> Option<[f64; 4]> {
    let shape = plan.shapes().shape(inst.shape)?;
    if shape.segment_count == 0 {
        return None;
    }
    // An affine image of an axis-aligned box reaches each screen-axis extremum
    // at a corner. Check all eight because a 3D linear part may mix z into x/y.
    let b = shape.bounds;
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    for x in [b.min[0], b.max[0]] {
        for y in [b.min[1], b.max[1]] {
            for z in [b.min[2], b.max[2]] {
                let point = map.place([x, y, z], inst.placement);
                for axis in 0..2 {
                    lo[axis] = lo[axis].min(point[axis]);
                    hi[axis] = hi[axis].max(point[axis]);
                }
            }
        }
    }
    let pad = stroke_expansion(plan, inst, map);
    Some([lo[0] - pad, lo[1] - pad, hi[0] + pad, hi[1] + pad])
}

/// How far beyond its outline a draw's stroke can reach, in screen pixels.
///
/// The widest geometric reach plus the antialiasing band — the same pad
/// [`crate::stroke::segment_slab`] applies per segment, computed here once per
/// instance. This is one half-width for round/bevel joins and up to
/// [`crate::stroke::MITER_LIMIT`] half-widths for a miter. Zero for a style that
/// strokes nothing, so a pure fill is binned to exactly its own hull: the
/// analytic fill's coverage *is* its antialiasing, and it is exactly zero
/// outside the path.
fn stroke_expansion(plan: &RenderPlan, inst: &Instance, map: ScreenMap) -> f64 {
    let Some(style) = plan.styles().get(inst.style) else {
        return 0.0;
    };
    if style.maximum_stroke_width() <= 0.0 {
        return 0.0;
    }
    crate::stroke::max_stroke_reach_px(style, map)
}

/// Does this instance's shape contain the whole tile?
///
/// Conservative in the safe direction: a `false` costs a classification and a
/// pruning opportunity, and there is no input for which a `true` is wrong. Only
/// the hints whose containment is a closed form answer `true`; everything else
/// draws.
#[must_use]
pub fn covers_tile(plan: &RenderPlan, inst: &Instance, map: ScreenMap, rect: [f64; 4]) -> bool {
    let Some(shape) = plan.shapes().shape(inst.shape) else {
        return false;
    };
    // A margin so that the containment holds for the whole pixel cell and not
    // merely for the tile's corner coordinates. Tiles are pixel-aligned, so one
    // pixel is enough; the extra is against the predicate's own edge cases
    // rather than the geometry's.
    const MARGIN: f64 = 1.0;
    let grown = [
        rect[0] - MARGIN,
        rect[1] - MARGIN,
        rect[2] + MARGIN,
        rect[3] + MARGIN,
    ];
    // A writable point view can change the outline without an engine callback.
    // The retained row may still carry a hint interned before that view existed,
    // so only the general predicate is admissible for its lifetime.
    if inst.hint_unsafe || !inst.placement.is_translation() {
        return covers_convex(plan, shape, map, inst.placement, grown);
    }
    match shape.hint {
        Hint::Rect {
            center,
            width,
            height,
        } => {
            let c = map.place(center, inst.placement);
            let hw = 0.5 * width * map.scale;
            let hh = 0.5 * height * map.scale;
            grown[0] >= c[0] - hw
                && grown[2] <= c[0] + hw
                && grown[1] >= c[1] - hh
                && grown[3] <= c[1] + hh
        }
        Hint::RoundedRect {
            center,
            width,
            height,
            corner_radius,
        } => {
            // Conservatively the inscribed rectangle, shrunk by the corner
            // radius on every side: any point inside that is inside the rounded
            // rectangle, whatever the corners do.
            let c = map.place(center, inst.placement);
            let hw = (0.5 * width - corner_radius).max(0.0) * map.scale;
            let hh = (0.5 * height - corner_radius).max(0.0) * map.scale;
            grown[0] >= c[0] - hw
                && grown[2] <= c[0] + hw
                && grown[1] >= c[1] - hh
                && grown[3] <= c[1] + hh
        }
        Hint::Circle { center, radius } | Hint::Dot { center, radius } => {
            let c = map.place(center, inst.placement);
            let r = radius * map.scale;
            // A rectangle is inside a disc iff its farthest corner is.
            let dx = (c[0] - grown[0]).abs().max((c[0] - grown[2]).abs());
            let dy = (c[1] - grown[1]).abs().max((c[1] - grown[3]).abs());
            dx * dx + dy * dy <= r * r
        }
        Hint::Polyline { closed: true } | Hint::General => {
            covers_convex(plan, shape, map, inst.placement, grown)
        }
        _ => false,
    }
}

/// The general case: a closed convex outline containing a rectangle.
///
/// Sound because a quadratic Bézier lies within its control polygon's hull and
/// bulges toward its handle, so a convex, consistently-turning control polygon
/// bounds a convex region — and four corners inside a convex region put the
/// whole rectangle inside it. An arc kernel would answer this exactly; this is
/// the fallback for outlines that carry no such tag.
fn covers_convex(
    plan: &RenderPlan,
    shape: &Shape,
    map: ScreenMap,
    placement: Placement,
    rect: [f64; 4],
) -> bool {
    let first = shape.first_segment as usize;
    let count = shape.segment_count as usize;
    if count < 2 {
        return false;
    }
    let segs = &plan.segments()[first..first + count];
    let close = |a: Vec3, b: Vec3| {
        (a[0] - b[0]).abs() <= 1e-9 && (a[1] - b[1]).abs() <= 1e-9 && (a[2] - b[2]).abs() <= 1e-9
    };
    if !close(segs[count - 1].p2, segs[0].p0) {
        return false;
    }

    let poly: Vec<[f64; 2]> = segs
        .iter()
        .flat_map(|s| [map.place(s.p0, placement), map.place(s.p1, placement)])
        .collect();
    let n = poly.len();
    let mut sign = 0i32;
    for i in 0..n {
        let (a, b, c) = (poly[i], poly[(i + 1) % n], poly[(i + 2) % n]);
        let cross = (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0]);
        let s = if cross > 1e-9 {
            1
        } else if cross < -1e-9 {
            -1
        } else {
            0
        };
        if s != 0 {
            if sign == 0 {
                sign = s;
            } else if sign != s {
                return false;
            }
        }
    }
    if sign == 0 {
        return false;
    }

    // Every corner strictly inside every edge's half-plane.
    for &(x, y) in &[
        (rect[0], rect[1]),
        (rect[2], rect[1]),
        (rect[0], rect[3]),
        (rect[2], rect[3]),
    ] {
        for i in 0..n {
            let (a, b) = (poly[i], poly[(i + 1) % n]);
            let cross = (b[0] - a[0]) * (y - a[1]) - (b[1] - a[1]) * (x - a[0]);
            if (cross * f64::from(sign)) < 0.0 {
                return false;
            }
        }
    }
    true
}

/// Does this instance composite as pure source wherever it has full coverage?
fn is_opaque_fill(plan: &RenderPlan, inst: &Instance) -> bool {
    plan.styles()
        .get(inst.style)
        .is_some_and(|s| s.has_opaque_fill())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::RenderPlan;
    use fmn_mobject::{JointType, Mobject, RecordBuffer, RecordSchema, ShapeTag, Stage};

    #[test]
    fn the_screen_map_is_one_invertible_definition_in_either_orientation() {
        for y_up in [true, false] {
            let map = ScreenMap {
                scale: 67.5,
                origin: [960.0, 540.0],
                y_up,
            };
            // +Y is up a top-row-first frame exactly when y_up.
            let above = map.to_pixel(0.0, 1.0);
            assert_eq!(above[1] < map.origin[1], y_up);
            assert_eq!(map.to_pixel(2.0, 0.0), [960.0 + 135.0, 540.0]);
            // A placement's translation and a point move by the same amount.
            let (p, d) = ([0.75, -1.5], [0.25, 2.0]);
            let moved = map.to_pixel(p[0] + d[0], p[1] + d[1]);
            let base = map.to_pixel(p[0], p[1]);
            let delta = map.delta_to_pixel(d[0], d[1]);
            assert_eq!(moved, [base[0] + delta[0], base[1] + delta[1]]);
            // The inverse undoes both, translate included.
            let translate = [3.0, -7.0];
            let pixel = [base[0] + translate[0], base[1] + translate[1]];
            assert_eq!(map.to_object(pixel, translate), p);
            // `place` is the same map.
            assert_eq!(map.place([p[0], p[1], 0.0], Placement::default()), base);
        }
        assert_eq!(
            ScreenMap::y_up(2.0, [1.0, 1.0]),
            ScreenMap {
                scale: 2.0,
                origin: [1.0, 1.0],
                y_up: true
            }
        );
    }

    fn rect_mobject(cx: f64, cy: f64, w: f64, h: f64, fill_alpha: f32) -> Mobject {
        let (hw, hh) = (0.5 * w, 0.5 * h);
        let pts: Vec<[f64; 3]> = vec![
            [cx - hw, cy - hh, 0.0],
            [cx, cy - hh, 0.0],
            [cx + hw, cy - hh, 0.0],
            [cx + hw, cy, 0.0],
            [cx + hw, cy + hh, 0.0],
            [cx, cy + hh, 0.0],
            [cx - hw, cy + hh, 0.0],
            [cx - hw, cy, 0.0],
            [cx - hw, cy - hh, 0.0],
        ];
        let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), pts.len()).unwrap();
        for (i, p) in pts.iter().enumerate() {
            buffer.write(i, "point", &[p[0] as f32, p[1] as f32, p[2] as f32]);
            buffer.write(i, "fill_rgba", &[0.2, 0.4, 0.6, fill_alpha]);
            buffer.write(i, "stroke_rgba", &[1.0, 1.0, 1.0, 1.0]);
        }
        Mobject::from_buffer(buffer)
    }

    /// A scene of axis-aligned rectangles, each tagged truthfully.
    fn scene(rects: &[(f64, f64, f64, f64, f32)]) -> Stage {
        let mut stage = Stage::new();
        for &(cx, cy, w, h, a) in rects {
            let mob = stage.add(rect_mobject(cx, cy, w, h, a));
            stage.set_shape(
                mob,
                ShapeTag::Rect {
                    center: [cx, cy, 0.0],
                    width: w,
                    height: h,
                },
            );
            stage.add_to_scene(mob).expect("live");
        }
        stage
    }

    fn synced(stage: &Stage) -> RenderPlan {
        let mut plan = RenderPlan::new();
        plan.sync(stage, 0).expect("valid binning fixture");
        plan
    }

    fn viewport() -> Viewport {
        Viewport {
            width: 256,
            height: 256,
        }
    }

    fn tile_at(binning: &Binning, x: u32, y: u32) -> usize {
        binning
            .tile_of(x, y)
            .expect("test coordinate is inside the viewport")
    }

    fn tile_run(binning: &Binning, index: usize) -> &[u32] {
        binning.tile(index).expect("test tile index is in range")
    }

    fn tile_class_run(binning: &Binning, index: usize) -> &[u32] {
        binning
            .tile_flags(index)
            .expect("test tile index is in range")
    }

    /// A stroked horizontal line whose hull ends exactly on a tile boundary.
    ///
    /// Purpose-built for the expansion test: the *centreline* stops at the
    /// boundary, so a slab that is only the hull lists no tile beyond it while
    /// the stroke's half-width and AA band plainly reach into the next one.
    fn stroked_line(x0: f64, y: f64, x1: f64, width: f32) -> Mobject {
        let pts = [[x0, y, 0.0], [0.5 * (x0 + x1), y, 0.0], [x1, y, 0.0]];
        let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), pts.len()).unwrap();
        for (i, p) in pts.iter().enumerate() {
            buffer.write(i, "point", &[p[0] as f32, p[1] as f32, p[2] as f32]);
            buffer.write(i, "stroke_rgba", &[1.0, 1.0, 1.0, 1.0]);
            buffer.write(i, "stroke_width", &[width]);
        }
        Mobject::from_buffer(buffer)
    }

    fn shallow_stroked_v(width: f32) -> Mobject {
        let points = [
            [40.0, 104.0, 0.0],
            [52.0, 84.0, 0.0],
            [64.0, 64.0, 0.0],
            [76.0, 84.0, 0.0],
            [88.0, 104.0, 0.0],
        ];
        let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len()).unwrap();
        for (i, point) in points.iter().enumerate() {
            buffer.write(
                i,
                "point",
                &[point[0] as f32, point[1] as f32, point[2] as f32],
            );
            buffer.write(i, "stroke_rgba", &[1.0, 1.0, 1.0, 1.0]);
            buffer.write(i, "stroke_width", &[width]);
        }
        Mobject::from_buffer(buffer)
    }

    #[test]
    fn binning_limits_accept_exact_boundaries_and_refuse_one_over() {
        let plan = synced(&scene(&[(16.0, 16.0, 32.0, 32.0, 1.0)]));
        let viewport = Viewport {
            width: 32,
            height: 32,
        };
        let tiling = Tiling {
            macro_tile: 32,
            fine_tile: 16,
        };
        let mut limits = BinningLimits {
            max_tiles: 4,
            max_instances: 1,
            max_tile_references: 4,
            max_partition_cells: 4,
            max_working_bytes: BinningLimits::default().max_working_bytes,
        };
        let exact =
            Binning::build_with_limits(&plan, viewport, tiling, ScreenMap::default(), limits)
                .expect("exact limits");
        assert_eq!(exact.tile_count(), 4);
        assert_eq!(exact.draws().len(), 4);

        limits.max_tiles = 3;
        assert_eq!(
            Binning::build_with_limits(&plan, viewport, tiling, ScreenMap::default(), limits,),
            Err(BinningError::LimitExceeded {
                resource: "fine tiles",
                requested: 4,
                limit: 3,
            })
        );
        limits.max_tiles = 4;
        limits.max_tile_references = 3;
        assert_eq!(
            Binning::build_with_limits(&plan, viewport, tiling, ScreenMap::default(), limits,),
            Err(BinningError::LimitExceeded {
                resource: "fine tile references",
                requested: 4,
                limit: 3,
            })
        );
    }

    #[test]
    fn each_count_axis_has_a_typed_preallocation_refusal() {
        let viewport = Viewport {
            width: 32,
            height: 32,
        };
        let tiling = Tiling {
            macro_tile: 32,
            fine_tile: 16,
        };
        let single = synced(&scene(&[(16.0, 16.0, 16.0, 16.0, 1.0)]));
        let mut limits = BinningLimits {
            max_instances: 0,
            ..BinningLimits::default()
        };
        assert_eq!(
            Binning::build_with_limits(&single, viewport, tiling, ScreenMap::default(), limits,),
            Err(BinningError::LimitExceeded {
                resource: "instances",
                requested: 1,
                limit: 0,
            })
        );

        limits.max_instances = BinningLimits::default().max_instances;
        limits.max_partition_cells = 3;
        assert_eq!(
            Binning::build_partitioned_with_limits(
                &single,
                viewport,
                tiling,
                ScreenMap::default(),
                96,
                limits,
            ),
            Err(BinningError::LimitExceeded {
                resource: "partition cells",
                requested: 4,
                limit: 3,
            })
        );

        let pair = synced(&scene(&[
            (8.0, 8.0, 8.0, 8.0, 1.0),
            (24.0, 24.0, 8.0, 8.0, 1.0),
        ]));
        limits.max_partition_cells = BinningLimits::default().max_partition_cells;
        limits.max_tile_references = 1;
        assert_eq!(
            Binning::build_with_limits(&pair, viewport, tiling, ScreenMap::default(), limits,),
            Err(BinningError::LimitExceeded {
                resource: "macrotile references",
                requested: 2,
                limit: 1,
            })
        );
    }

    #[test]
    fn default_limits_admit_the_certified_8k_partition_grid() -> Result<(), BinningError> {
        let limits = BinningLimits::default();
        let fine = Grid::new(
            Viewport {
                width: 7680,
                height: 4320,
            },
            Tiling::default().fine_tile,
            "fine tiles",
            limits,
        )?;
        assert_eq!(fine.count(), 129_600);
        let partition_cells = checked_product(
            "partition cells",
            u64::try_from(fine.count()).unwrap_or(u64::MAX),
            96,
        )?;
        assert_eq!(partition_cells, 12_441_600);
        assert!(partition_cells <= limits.max_partition_cells);
        let count_and_cursor_bytes = checked_product(
            "partition table bytes",
            table_bytes::<u32>("partition count cells", partition_cells)?,
            2,
        )?;
        assert!(count_and_cursor_bytes < limits.max_working_bytes);
        Ok(())
    }

    fn required_working_bytes(
        result: Result<Binning, BinningError>,
        limit: u64,
        phase: &str,
    ) -> Result<u64, String> {
        match result {
            Err(BinningError::LimitExceeded {
                resource: "working bytes",
                requested,
                limit: actual_limit,
            }) if actual_limit == limit => Ok(requested),
            other => Err(format!("unexpected {phase}-budget result: {other:?}")),
        }
    }

    #[test]
    fn binning_working_byte_limit_is_checked_before_each_table_phase() -> Result<(), String> {
        let plan = synced(&scene(&[(16.0, 16.0, 32.0, 32.0, 1.0)]));
        let viewport = Viewport {
            width: 32,
            height: 32,
        };
        let tiling = Tiling {
            macro_tile: 32,
            fine_tile: 16,
        };
        let mut limits = BinningLimits {
            max_working_bytes: 0,
            ..BinningLimits::default()
        };
        let initial = required_working_bytes(
            Binning::build_with_limits(&plan, viewport, tiling, ScreenMap::default(), limits),
            0,
            "initial",
        )?;
        limits.max_working_bytes = initial;
        let macro_tables = required_working_bytes(
            Binning::build_with_limits(&plan, viewport, tiling, ScreenMap::default(), limits),
            initial,
            "macrotile",
        )?;
        assert!(macro_tables > initial);
        limits.max_working_bytes = macro_tables;
        let fine_counts = required_working_bytes(
            Binning::build_with_limits(&plan, viewport, tiling, ScreenMap::default(), limits),
            macro_tables,
            "fine-count",
        )?;
        assert!(fine_counts > macro_tables);
        limits.max_working_bytes = fine_counts;
        let full = required_working_bytes(
            Binning::build_with_limits(&plan, viewport, tiling, ScreenMap::default(), limits),
            fine_counts,
            "full",
        )?;
        assert!(full > fine_counts);
        limits.max_working_bytes = full;
        assert!(
            Binning::build_with_limits(&plan, viewport, tiling, ScreenMap::default(), limits,)
                .is_ok()
        );
        limits.max_working_bytes = full - 1;
        assert!(matches!(
            Binning::build_with_limits(
                &plan,
                viewport,
                tiling,
                ScreenMap::default(),
                limits,
            ),
            Err(BinningError::LimitExceeded {
                resource: "working bytes",
                requested,
                limit,
            }) if requested == full && limit + 1 == full
        ));
        Ok(())
    }

    #[test]
    fn maximal_viewport_is_a_typed_preallocation_refusal() {
        assert!(matches!(
            Binning::build(
                &RenderPlan::new(),
                Viewport {
                    width: u32::MAX,
                    height: u32::MAX,
                },
                Tiling {
                    macro_tile: 1,
                    fine_tile: 1,
                },
                ScreenMap::default(),
            ),
            Err(BinningError::LimitExceeded {
                resource: "fine tiles",
                ..
            })
        ));
    }

    #[test]
    fn off_viewport_coordinates_do_not_alias_valid_tiles() {
        let viewport = viewport();
        let binning = Binning::build(
            &RenderPlan::new(),
            viewport,
            Tiling::default(),
            ScreenMap::default(),
        )
        .expect("bounded empty-scene binning");

        assert_eq!(binning.tile_of(0, Tiling::default().fine_tile), Some(16),);
        assert_eq!(binning.tile_of(viewport.width, 0), None);
        assert_eq!(binning.tile_of(0, viewport.height), None);
        assert_eq!(binning.tile_of(u32::MAX, u32::MAX), None);

        let empty = Binning::build(
            &RenderPlan::new(),
            Viewport::default(),
            Tiling::default(),
            ScreenMap::default(),
        )
        .expect("zero-area viewport has an empty grid");
        assert_eq!(empty.tile_count(), 0);
        assert_eq!(empty.tile_of(0, 0), None);
    }

    #[test]
    fn out_of_range_tile_runs_are_checked() {
        let binning = Binning::build(
            &RenderPlan::new(),
            viewport(),
            Tiling::default(),
            ScreenMap::default(),
        )
        .expect("bounded empty-scene binning");
        let boundary = binning.tile_count();
        assert_eq!(binning.tile(boundary), None);
        assert_eq!(binning.tile_flags(boundary), None);
        assert_eq!(binning.tile(usize::MAX), None);
        assert_eq!(binning.tile_flags(usize::MAX), None);

        let empty = Binning::build(
            &RenderPlan::new(),
            Viewport::default(),
            Tiling::default(),
            ScreenMap::default(),
        )
        .expect("zero-area viewport has an empty grid");
        assert_eq!(empty.tile(0), None);
        assert_eq!(empty.tile_flags(0), None);
    }

    #[test]
    fn a_stroke_is_binned_to_the_tiles_its_width_reaches() {
        // §10.3's per-draw width expansion, as a test rather than a sentence.
        // The line's hull is the single row y = 64, which lands on a tile
        // boundary; a 400-unit stroke is 4 px wide, so with the AA band the
        // stroke covers y from about 60.5 to 67.5 and must be listed in the tile
        // row above the boundary as well as the one below it.
        //
        // Without the expansion this test finds one tile row instead of two, and
        // the engine draws a stroke with its top edge sliced off.
        let mut stage = Stage::new();
        let mob = stage.add(stroked_line(40.0, 64.0, 200.0, 400.0));
        stage.add_to_scene(mob).expect("live");
        let plan = synced(&stage);
        let tiling = Tiling {
            macro_tile: 128,
            fine_tile: 16,
        };
        let b = Binning::build(&plan, viewport(), tiling, ScreenMap::default())
            .expect("bounded test binning");

        let listed = |x: u32, y: u32| !tile_run(&b, tile_at(&b, x, y)).is_empty();
        assert!(listed(64, 64), "the centreline's own tile row is missing");
        assert!(
            listed(64, 63),
            "the tile row above the boundary is unlisted, so the stroke's upper \
             half-width and AA band would never be drawn"
        );
    }

    #[test]
    fn a_miter_is_binned_to_the_tiles_its_tip_reaches() {
        // The V's apex is at y=64 and its admitted miter reaches into the
        // y=32..48 tile. A half-width-only expansion stops in y=48..64 and
        // silently clips the tip before the stroke kernel can see it.
        let mut stage = Stage::new();
        let mob = stage.add(shallow_stroked_v(2000.0));
        stage.uniforms_mut(mob).expect("live").joint_type = JointType::Miter;
        stage.add_to_scene(mob).expect("live");
        let plan = synced(&stage);
        let tiling = Tiling {
            macro_tile: 128,
            fine_tile: 16,
        };
        let b = Binning::build(&plan, viewport(), tiling, ScreenMap::default())
            .expect("bounded test binning");
        assert!(
            !tile_run(&b, tile_at(&b, 64, 47)).is_empty(),
            "the miter-tip tile is absent"
        );
    }

    #[test]
    fn a_pure_fill_is_binned_to_exactly_its_own_hull() {
        // The other half of the expansion rule: the analytic fill's coverage IS
        // its antialiasing and is exactly zero outside the path, so a fill with
        // no stroke must not pay for a band it cannot reach into.
        let stage = scene(&[(64.0, 64.0, 32.0, 32.0, 1.0)]);
        let plan = synced(&stage);
        let tiling = Tiling {
            macro_tile: 128,
            fine_tile: 16,
        };
        let b = Binning::build(&plan, viewport(), tiling, ScreenMap::default())
            .expect("bounded test binning");
        // The rect spans x,y in [48, 80]: tiles 3..=5 on each axis and no more.
        assert!(!tile_run(&b, tile_at(&b, 48, 48)).is_empty());
        assert!(
            tile_run(&b, tile_at(&b, 32, 64)).is_empty(),
            "a fill was padded into a tile its coverage cannot reach"
        );
    }

    #[test]
    fn a_negative_scale_still_bins_every_instance() {
        // A negative `ScreenMap::scale` is the natural spelling of a y-flip, and
        // it maps `bounds.min` above `bounds.max`. An un-normalized AABB makes
        // `Grid::span`'s `lo..=hi` loops empty, so every instance silently
        // vanishes — a blank frame with no error anywhere.
        let stage = scene(&[(-64.0, -64.0, 40.0, 40.0, 1.0)]);
        let plan = synced(&stage);
        let flipped = ScreenMap {
            scale: -1.0,
            origin: [0.0, 0.0],
            y_up: false,
        };
        let b = Binning::build(&plan, viewport(), Tiling::default(), flipped)
            .expect("bounded test binning");
        assert!(
            !b.draws().is_empty(),
            "a y-flip binned the whole scene out of existence"
        );
        assert!(!tile_run(&b, tile_at(&b, 64, 64)).is_empty());
    }

    #[test]
    fn a_tile_run_is_ascending_which_is_painter_order() {
        let stage = scene(&[
            (60.0, 60.0, 100.0, 100.0, 1.0),
            (80.0, 80.0, 100.0, 100.0, 1.0),
            (100.0, 100.0, 100.0, 100.0, 1.0),
        ]);
        let plan = synced(&stage);
        let b = Binning::build(&plan, viewport(), Tiling::default(), ScreenMap::default())
            .expect("bounded test binning");
        assert!(!b.draws().is_empty());
        for t in 0..b.tile_count() {
            let run = tile_run(&b, t);
            assert!(
                run.windows(2).all(|w| w[0] < w[1]),
                "tile {t} out of painter order: {run:?}"
            );
        }
    }

    #[test]
    fn binning_is_identical_at_every_partition_count() {
        // §10.5's determinism obligation, made testable without threads: a
        // partition is a slice of the painter-ordered instance list, and no
        // result may depend on which slice would have finished first. The odd
        // counts matter — they are the ones that do not divide the work evenly.
        let stage = scene(&[
            (40.0, 40.0, 60.0, 60.0, 1.0),
            (90.0, 50.0, 80.0, 40.0, 0.5),
            (150.0, 150.0, 120.0, 90.0, 1.0),
            (30.0, 200.0, 50.0, 50.0, 0.25),
            (200.0, 60.0, 70.0, 70.0, 1.0),
        ]);
        let plan = synced(&stage);
        let serial = Binning::build_partitioned(
            &plan,
            viewport(),
            Tiling::default(),
            ScreenMap::default(),
            1,
        )
        .expect("bounded serial test binning");
        for partitions in [1, 2, 3, 4, 5, 7, 16, 64] {
            let parallel = Binning::build_partitioned(
                &plan,
                viewport(),
                Tiling::default(),
                ScreenMap::default(),
                partitions,
            )
            .expect("bounded partitioned test binning");
            assert_eq!(
                parallel.offsets(),
                serial.offsets(),
                "offsets differ at {partitions} partitions"
            );
            assert_eq!(
                parallel.draws(),
                serial.draws(),
                "draws differ at {partitions} partitions"
            );
            assert_eq!(
                parallel.flags(),
                serial.flags(),
                "classes differ at {partitions} partitions"
            );
        }
    }

    /// The ordered commands that can actually reach one pixel.
    ///
    /// Binning is *conservative*: a tile lists every instance whose bounds meet
    /// the tile, so a coarser tile legitimately lists commands that miss the
    /// pixel. The invariant tiling owes is therefore not "the same list" — it is
    /// that the list always **contains** every command that can reach the pixel,
    /// in painter order, and that filtering to those gives the same answer at
    /// any tile size. (The first version of this test compared raw lists and
    /// failed for exactly the right reason.)
    fn relevant_at(b: &Binning, plan: &RenderPlan, map: ScreenMap, x: u32, y: u32) -> Vec<u32> {
        let px = f64::from(x) + 0.5;
        let py = f64::from(y) + 0.5;
        let instances = plan.shapes().instances();
        tile_run(b, tile_at(b, x, y))
            .iter()
            .copied()
            .filter(|&d| {
                screen_aabb(plan, &instances[d as usize], map)
                    .is_some_and(|a| px >= a[0] && px <= a[2] && py >= a[1] && py <= a[3])
            })
            .collect()
    }

    /// The same thing computed without any binning at all — the ground truth a
    /// binner must reproduce.
    fn relevant_unbinned(plan: &RenderPlan, map: ScreenMap, x: u32, y: u32) -> Vec<u32> {
        let px = f64::from(x) + 0.5;
        let py = f64::from(y) + 0.5;
        plan.shapes()
            .instances()
            .iter()
            .enumerate()
            .filter(|(_, inst)| {
                screen_aabb(plan, inst, map)
                    .is_some_and(|a| px >= a[0] && px <= a[2] && py >= a[1] && py <= a[3])
            })
            .map(|(i, _)| i as u32)
            .collect()
    }

    #[test]
    fn tile_size_is_a_scheduling_choice_and_changes_no_command() {
        // §17.4: "tile and macrotile dimensions… never by semantics". Every
        // tiling must deliver every relevant command to every pixel, in painter
        // order, and agree with the unbinned ground truth.
        let stage = scene(&[
            (50.0, 50.0, 90.0, 90.0, 1.0),
            (120.0, 90.0, 110.0, 60.0, 0.5),
            (180.0, 180.0, 100.0, 100.0, 1.0),
        ]);
        let plan = synced(&stage);
        let map = ScreenMap::default();
        for tiling in [
            Tiling {
                macro_tile: 128,
                fine_tile: 16,
            },
            Tiling {
                macro_tile: 128,
                fine_tile: 8,
            },
            Tiling {
                macro_tile: 64,
                fine_tile: 32,
            },
            Tiling {
                macro_tile: 256,
                fine_tile: 4,
            },
            Tiling {
                macro_tile: 32,
                fine_tile: 32,
            },
        ] {
            let b = Binning::build(&plan, viewport(), tiling, map).expect("bounded test binning");
            for y in (0..256).step_by(7) {
                for x in (0..256).step_by(5) {
                    assert_eq!(
                        relevant_at(&b, &plan, map, x, y),
                        relevant_unbinned(&plan, map, x, y),
                        "pixel ({x},{y}) under {tiling:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_macrotile_pass_actually_prunes() {
        // A two-level scheme whose first level accepts everything is a slower
        // one-level scheme. One small object in a large viewport must leave most
        // macrotiles empty.
        let stage = scene(&[(20.0, 20.0, 20.0, 20.0, 1.0)]);
        let plan = synced(&stage);
        let b = Binning::build(&plan, viewport(), Tiling::default(), ScreenMap::default())
            .expect("bounded test binning");
        let (hits, total) = b.macro_occupancy();
        assert!(total > 1, "the viewport must span several macrotiles");
        assert!(
            hits < total,
            "the macrotile pass pruned nothing: {hits}/{total}"
        );
    }

    #[test]
    fn an_off_screen_object_produces_no_command() {
        // The difference between "not visible" and "drawn at the edge": a
        // clamped span would put a command in the border tiles.
        let stage = scene(&[(-500.0, -500.0, 40.0, 40.0, 1.0)]);
        let plan = synced(&stage);
        let b = Binning::build(&plan, viewport(), Tiling::default(), ScreenMap::default())
            .expect("bounded test binning");
        assert!(b.draws().is_empty(), "{:?}", b.draws());
    }

    #[test]
    fn a_tile_inside_an_opaque_rectangle_is_classified_interior() {
        let stage = scene(&[(128.0, 128.0, 200.0, 200.0, 1.0)]);
        let plan = synced(&stage);
        let b = Binning::build(&plan, viewport(), Tiling::default(), ScreenMap::default())
            .expect("bounded test binning");
        // The centre tile is deep inside; a corner tile is not.
        let centre = tile_at(&b, 128, 128);
        assert_eq!(tile_class_run(&b, centre), &[CLASS_INTERIOR]);
        let corner = tile_at(&b, 30, 30);
        assert!(
            tile_class_run(&b, corner)
                .iter()
                .all(|f| *f == CLASS_PARTIAL),
            "the edge tile cannot be interior"
        );
    }

    #[test]
    fn pruning_drops_the_hidden_layer_and_keeps_the_visible_suffix() {
        // The equivalence the bead asks for, stated the way it can be checked
        // before an engine exists: for every pixel, the *visible suffix* of the
        // command list — everything from the last opaque full-cover onward — is
        // unchanged. Nothing before it can reach a pixel, whatever the shading.
        let stage = scene(&[
            (128.0, 128.0, 60.0, 60.0, 1.0),
            (128.0, 128.0, 220.0, 220.0, 1.0),
            (128.0, 128.0, 40.0, 40.0, 0.5),
        ]);
        let plan = synced(&stage);
        let before = Binning::build(&plan, viewport(), Tiling::default(), ScreenMap::default())
            .expect("bounded test binning");
        let mut after = before.clone();
        let report = after.prune_occluded(&plan).expect("matching plan");

        assert!(report.removed_fraction() > 0.0, "{report:?}");
        for y in (0..256).step_by(3) {
            for x in (0..256).step_by(3) {
                let a = visible_suffix(&before, &plan, x, y);
                let b = visible_suffix(&after, &plan, x, y);
                assert_eq!(a, b, "pixel ({x},{y}) sees a different visible suffix");
            }
        }
    }

    /// Everything from the last opaque full-tile cover onward.
    fn visible_suffix(b: &Binning, plan: &RenderPlan, x: u32, y: u32) -> Vec<u32> {
        let t = tile_at(b, x, y);
        let run = tile_run(b, t);
        let flags = tile_class_run(b, t);
        let instances = plan.shapes().instances();
        let mut start = 0;
        for (k, &d) in run.iter().enumerate() {
            if flags[k] == CLASS_INTERIOR && is_opaque_fill(plan, &instances[d as usize]) {
                start = k;
            }
        }
        run[start..].to_vec()
    }

    #[test]
    fn a_translucent_cover_prunes_nothing() {
        // The adversarial case the rule exists for: a stack under something that
        // blends. Every command still reaches the compositor.
        let stage = scene(&[
            (128.0, 128.0, 60.0, 60.0, 1.0),
            (128.0, 128.0, 220.0, 220.0, 0.5),
        ]);
        let plan = synced(&stage);
        let mut b = Binning::build(&plan, viewport(), Tiling::default(), ScreenMap::default())
            .expect("bounded test binning");
        let report = b.prune_occluded(&plan).expect("matching plan");
        assert_eq!(report.before, report.after, "{report:?}");
    }

    #[test]
    fn pruning_refuses_a_stale_plan_without_mutating_the_binning() {
        let source = scene(&[(40.0, 40.0, 60.0, 60.0, 1.0)]);
        let source_plan = synced(&source);
        let mut binning = Binning::build(
            &source_plan,
            viewport(),
            Tiling::default(),
            ScreenMap::default(),
        )
        .expect("bounded test binning");
        let before = binning.clone();

        // Same table and instance cardinalities, different placement. Index-only
        // validation would accept this and prune using the wrong rectangle.
        let current = scene(&[(200.0, 200.0, 60.0, 60.0, 1.0)]);
        let current_plan = synced(&current);
        assert_eq!(
            binning.prune_occluded(&current_plan),
            Err(BinningError::PlanMismatch)
        );
        assert_eq!(binning, before, "a failed prune must be transactional");
    }

    #[test]
    fn an_opaque_cover_that_does_not_reach_a_tile_prunes_only_where_it_does() {
        // Painter-order safety at the boundary: the cover is opaque and full in
        // the tiles it fills, and irrelevant in the tiles it misses.
        let stage = scene(&[
            (40.0, 40.0, 60.0, 60.0, 1.0),
            (200.0, 200.0, 60.0, 60.0, 1.0),
            (200.0, 200.0, 100.0, 100.0, 1.0),
        ]);
        let plan = synced(&stage);
        let before = Binning::build(&plan, viewport(), Tiling::default(), ScreenMap::default())
            .expect("bounded test binning");
        let mut after = before.clone();
        after.prune_occluded(&plan).expect("matching plan");

        // The lone rectangle at (40,40) is instance 0 and nothing covers it.
        let lone = tile_at(&before, 40, 40);
        assert!(tile_run(&before, lone).contains(&0));
        assert!(
            tile_run(&after, lone).contains(&0),
            "an uncovered draw was dropped"
        );
        // Under the big cover, the small one is gone.
        let covered = tile_at(&before, 200, 200);
        assert!(tile_run(&before, covered).contains(&1));
        assert!(!tile_run(&after, covered).contains(&1));
    }

    #[test]
    fn pruning_never_drops_the_covering_command_itself() {
        let stage = scene(&[
            (128.0, 128.0, 40.0, 40.0, 1.0),
            (128.0, 128.0, 200.0, 200.0, 1.0),
        ]);
        let plan = synced(&stage);
        let mut b = Binning::build(&plan, viewport(), Tiling::default(), ScreenMap::default())
            .expect("bounded test binning");
        b.prune_occluded(&plan).expect("matching plan");
        let centre = tile_at(&b, 128, 128);
        assert_eq!(
            tile_run(&b, centre),
            &[1],
            "only the cover survives, and it does"
        );
    }

    #[test]
    fn pruning_is_identical_at_every_partition_count() {
        // Pruning consumes the classification, so it inherits the determinism
        // obligation and has to be checked under it.
        let stage = scene(&[
            (128.0, 128.0, 60.0, 60.0, 1.0),
            (60.0, 60.0, 40.0, 40.0, 1.0),
            (128.0, 128.0, 240.0, 240.0, 1.0),
            (100.0, 100.0, 30.0, 30.0, 0.5),
        ]);
        let plan = synced(&stage);
        let mut reference = Binning::build_partitioned(
            &plan,
            viewport(),
            Tiling::default(),
            ScreenMap::default(),
            1,
        )
        .expect("bounded reference test binning");
        reference
            .prune_occluded(&plan)
            .expect("matching reference plan");
        for partitions in [2, 3, 8, 32] {
            let mut other = Binning::build_partitioned(
                &plan,
                viewport(),
                Tiling::default(),
                ScreenMap::default(),
                partitions,
            )
            .expect("bounded partitioned test binning");
            other.prune_occluded(&plan).expect("matching plan");
            assert_eq!(other.offsets(), reference.offsets());
            assert_eq!(other.draws(), reference.draws());
            assert_eq!(other.flags(), reference.flags());
        }
    }

    #[test]
    fn a_concave_outline_never_claims_to_cover_a_tile() {
        // The convexity clause is load-bearing: an L covers some of its bounding
        // tiles and not others, and the corner test alone cannot tell which.
        let vertices: Vec<[f64; 3]> = [
            [20.0, 20.0],
            [120.0, 20.0],
            [220.0, 20.0],
            [220.0, 70.0],
            [220.0, 120.0],
            [120.0, 120.0],
            [120.0, 170.0],
            [120.0, 220.0],
            [70.0, 220.0],
            [20.0, 220.0],
            [20.0, 120.0],
            [20.0, 20.0],
        ]
        .iter()
        .map(|p| [p[0], p[1], 0.0])
        .collect();
        let mut l = Vec::with_capacity(2 * vertices.len() - 1);
        l.push(vertices[0]);
        for pair in vertices.windows(2) {
            l.push([
                0.5 * (pair[0][0] + pair[1][0]),
                0.5 * (pair[0][1] + pair[1][1]),
                0.5 * (pair[0][2] + pair[1][2]),
            ]);
            l.push(pair[1]);
        }
        let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), l.len()).unwrap();
        for (i, p) in l.iter().enumerate() {
            buffer.write(i, "point", &[p[0] as f32, p[1] as f32, p[2] as f32]);
            buffer.write(i, "fill_rgba", &[0.2, 0.4, 0.6, 1.0]);
        }
        let mut stage = Stage::new();
        let mob = stage.add(Mobject::from_buffer(buffer));
        stage.add_to_scene(mob).expect("live");
        let plan = synced(&stage);
        let b = Binning::build(&plan, viewport(), Tiling::default(), ScreenMap::default())
            .expect("bounded test binning");
        assert!(
            !b.flags().is_empty(),
            "the concave fixture must reach tiles"
        );
        assert!(
            b.flags().iter().all(|f| *f == CLASS_PARTIAL),
            "a concave outline claimed to cover a tile"
        );
    }

    #[test]
    fn a_circle_covers_a_tile_only_when_its_farthest_corner_is_inside() {
        let mut stage = Stage::new();
        let mob = stage.add(rect_mobject(128.0, 128.0, 200.0, 200.0, 1.0));
        stage.set_shape(
            mob,
            ShapeTag::Circle {
                center: [128.0, 128.0, 0.0],
                radius: 100.0,
            },
        );
        stage.add_to_scene(mob).expect("live");
        let plan = synced(&stage);
        let b = Binning::build(&plan, viewport(), Tiling::default(), ScreenMap::default())
            .expect("bounded test binning");
        // Dead centre: inside. On the diagonal near the rim: not.
        assert_eq!(tile_class_run(&b, tile_at(&b, 128, 128)), &[CLASS_INTERIOR]);
        let rim = tile_at(&b, 128 + 96, 128);
        assert!(tile_class_run(&b, rim).iter().all(|f| *f == CLASS_PARTIAL));
    }

    #[test]
    fn two_instances_of_one_outline_bin_at_their_own_positions() {
        // The regression test for a real bug this bead surfaced. `shape_digest`
        // excludes position — that is what makes interning find anything — but
        // compiled outlines were being stored at the first mobject's ABSOLUTE
        // coordinates. Two 60x60 rectangles at (40,40) and (200,200) therefore
        // shared a shape whose bounds were the first one's, so the second binned
        // on top of the first and an opaque cover pruned a draw that was
        // nowhere near it. Outlines are shape-local now, and the instance
        // placement puts each occurrence in world space.
        let stage = scene(&[
            (40.0, 40.0, 60.0, 60.0, 1.0),
            (200.0, 200.0, 60.0, 60.0, 1.0),
        ]);
        let plan = synced(&stage);
        assert_eq!(
            plan.shapes().shapes().len(),
            1,
            "the two rectangles are one outline — the interning must still fire"
        );

        let b = Binning::build(&plan, viewport(), Tiling::default(), ScreenMap::default())
            .expect("bounded test binning");
        let near = tile_at(&b, 40, 40);
        let far = tile_at(&b, 200, 200);
        assert_eq!(tile_run(&b, near), &[0], "instance 0 belongs at (40,40)");
        assert_eq!(tile_run(&b, far), &[1], "instance 1 belongs at (200,200)");
        assert!(
            !tile_run(&b, near).contains(&1) && !tile_run(&b, far).contains(&0),
            "neither may bin at the other's position"
        );
    }

    #[test]
    fn an_empty_plan_bins_to_an_empty_but_valid_csr() {
        let plan = RenderPlan::new();
        let b = Binning::build(&plan, viewport(), Tiling::default(), ScreenMap::default())
            .expect("bounded test binning");
        assert_eq!(b.offsets().len(), b.tile_count() + 1);
        assert!(b.draws().is_empty());
        assert_eq!(*b.offsets().last().expect("non-empty"), 0);
    }

    #[test]
    fn the_csr_is_well_formed() {
        let stage = scene(&[
            (60.0, 60.0, 100.0, 100.0, 1.0),
            (180.0, 180.0, 90.0, 90.0, 0.5),
        ]);
        let plan = synced(&stage);
        let b = Binning::build(&plan, viewport(), Tiling::default(), ScreenMap::default())
            .expect("bounded test binning");
        assert_eq!(b.offsets().len(), b.tile_count() + 1);
        assert_eq!(
            *b.offsets().last().expect("non-empty") as usize,
            b.draws().len()
        );
        assert_eq!(b.flags().len(), b.draws().len());
        assert!(b.offsets().windows(2).all(|w| w[0] <= w[1]));
    }
}
