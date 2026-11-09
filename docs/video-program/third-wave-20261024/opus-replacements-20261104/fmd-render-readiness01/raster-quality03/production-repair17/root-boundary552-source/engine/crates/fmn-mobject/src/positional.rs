//! The structural/positional surface (§8.4, §1.1): bounding boxes with
//! automatic invalidation, the critical-point getters, and the full positional
//! API — all as [`Stage`] methods that recurse over a handle's family, exactly
//! as the Reference's `Mobject` methods recurse over `get_family()`.
//!
//! # Bounding boxes and invalidation
//!
//! Every positional operation is defined against the bounding box (§8.4), so the
//! box must be both correct and cheap. Ours is **lazily recomputed and keyed by
//! a subtree signature**: [`Stage::get_bounding_box`] folds the signature of
//! every family member's point-field revision and identity; if it matches the
//! cache the stored box is returned, otherwise the box is recomputed from the
//! `point` records in f64. Because a point write bumps the RecordBuffer's field
//! revision and a structural change alters the family list, the box invalidates
//! **automatically through any channel** — a raw `write` on the record buffer, a
//! positional transform, or an `attach`/`detach` — with no explicit dirty-flag
//! bookkeeping to get wrong. This is the §8.4 dirty-flag requirement expressed
//! as the D-20 lazy-revisioned-mirror pattern already used for render mirrors.
//!
//! Native affine operations normally let this cache recompute — min/max commute
//! with those maps and negative scales then produce the mathematically correct
//! ordering. The Python portal also has a narrow caller-installed cache seam
//! for the Reference's public `works_on_bounding_box=True` branch, where an
//! arbitrary Python function transforms the stored `[min, mid, max]` rows
//! directly. That value remains guarded by the same subtree signature rather
//! than becoming an untracked second source of geometry truth.
//!
//! # Numeric doctrine
//!
//! Points are stored as f32 records but all positional math is done in f64
//! (`Semantic`, §6.1); parity against the Reference is checked at loose f32
//! tolerance (§16.4).

use fmn_core::constants::{DOWN, FRAME_X_RADIUS, FRAME_Y_RADIUS, IN, LEFT, ORIGIN, OUT, RIGHT, UP};
use fmn_core::types::Vec3;
use fmn_geom::{Mat3, QuadPath, space_ops};

use crate::StageError;
use crate::bbox::{BoundingBox, BoxAccum};
use crate::stage::{Mob, Stage};
use crate::uniforms::Uniforms;
use crate::{Placement, RecordBuffer};

/// A positional target: another mobject (positioned by its critical point) or a
/// literal point. Methods that accept "a mobject or a point" take
/// `impl Into<PosTarget>`, so both `stage.move_to(a, b, ORIGIN)` and
/// `stage.move_to(a, [1.0, 0.0, 0.0], ORIGIN)` read naturally.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PosTarget {
    /// Align against another mobject's bounding box.
    Mob(Mob),
    /// Align against a fixed point.
    Point(Vec3),
}

impl From<Mob> for PosTarget {
    fn from(m: Mob) -> Self {
        Self::Mob(m)
    }
}

impl From<Vec3> for PosTarget {
    fn from(p: Vec3) -> Self {
        Self::Point(p)
    }
}

/// Prepare all auxiliary 3D point fields before resetting retained placement.
/// A normal-control point is a point, not a direction: translating only `point`
/// changes their difference and therefore changes surface lighting. UVs, colors,
/// and other non-pointlike fields must never receive the geometric map.
fn mapped_auxiliary_points(
    buffer: &RecordBuffer,
    placement: Placement,
    map: impl Fn(Vec3) -> Vec3,
) -> Vec<(String, Vec<f32>)> {
    let mut columns: Vec<(String, Vec<f32>)> = Vec::new();
    for key in buffer.schema().pointlike_keys() {
        if key == "point" || columns.iter().any(|(name, _)| name == key) {
            continue;
        }
        if !buffer
            .schema()
            .fields()
            .iter()
            .any(|field| field.name == *key && field.width == 3)
        {
            continue;
        }
        if let Some(values) = buffer.read_column(key) {
            #[allow(clippy::cast_possible_truncation)]
            let mapped = values
                .as_chunks::<3>()
                .0
                .iter()
                .flat_map(|point| {
                    map(placement.apply_point([
                        f64::from(point[0]),
                        f64::from(point[1]),
                        f64::from(point[2]),
                    ]))
                    .map(|value| value as f32)
                })
                .collect();
            columns.push((key.clone(), mapped));
        }
    }
    columns
}

// --- small f64 vector helpers (kept local; the geometry kernel's Vec3 math is
// pub(crate)-internal to fmn-geom and not a dependency here) ---

#[inline]
fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn scaled(a: Vec3, s: f64) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn neg(a: Vec3) -> Vec3 {
    [-a[0], -a[1], -a[2]]
}

#[inline]
fn hadamard(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2]]
}

/// NumPy `sign`: `-0.0` and `+0.0` both map to `0.0` (unlike `f64::signum`).
#[inline]
fn np_sign(a: Vec3) -> Vec3 {
    let s = |x: f64| {
        if x > 0.0 {
            1.0
        } else if x < 0.0 {
            -1.0
        } else {
            0.0
        }
    };
    [s(a[0]), s(a[1]), s(a[2])]
}

#[inline]
fn np_abs(a: Vec3) -> Vec3 {
    [a[0].abs(), a[1].abs(), a[2].abs()]
}

/// The Reference's per-scalar minimum scale factor (`scale`'s `min_scale_factor`).
const MIN_SCALE_FACTOR: f64 = 1e-8;

impl Stage {
    // ---------------------------------------------------------- bounding box

    /// A subtree signature: any point write, placement edit, resize, or
    /// structural change alters it, which is what drives cache invalidation.
    fn bbox_signature(&self, mob: Mob) -> u64 {
        // FNV-1a over the family. In-memory only; never serialized, so no
        // cross-platform determinism obligation (that is fmn-hash's job).
        const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
        const PRIME: u64 = 0x0000_0100_0000_01b3;
        let mut h = OFFSET;
        let mut mix = |v: u64| h = (h ^ v).wrapping_mul(PRIME);
        for m in self.family(mob) {
            mix(m.bits());
            if let Some(e) = self.get(m) {
                mix(e.buffer.field_revision("point").unwrap_or(0));
                mix(e.placement_revision());
                mix(e.buffer.len() as u64);
            }
        }
        h
    }

    /// Recompute the box from the family's `point` records (in f64).
    /// A point-free family has a degenerate box at its retained anchor; an
    /// empty parent or sibling must not contribute an artificial origin to a
    /// family that does contain geometry.
    fn compute_bounding_box(&self, mob: Mob) -> BoundingBox {
        let mut acc = BoxAccum::new();
        let mut has_points = false;
        for m in self.family(mob) {
            if let Some(e) = self.get(m)
                && let Some(col) = e.buffer.read_column("point")
            {
                let placement = e.placement();
                let (tris, _rem) = col.as_chunks::<3>();
                for tri in tris {
                    has_points = true;
                    acc.push(placement.apply_point([
                        f64::from(tri[0]),
                        f64::from(tri[1]),
                        f64::from(tri[2]),
                    ]));
                }
            }
        }
        if !has_points && let Some(entry) = self.get(mob) {
            acc.push(entry.placement().apply_point(ORIGIN));
        }
        acc.finish()
    }

    fn bbox_has_writable_point_view(&self, mob: Mob) -> bool {
        self.family(mob).into_iter().any(|member| {
            self.get(member)
                .is_some_and(|entry| entry.buffer.writable_view_affects("point"))
        })
    }

    /// The bounding box of `mob`'s whole family (§8.4). Lazily recomputed; see
    /// the module docs for the invalidation model.
    #[must_use]
    pub fn get_bounding_box(&self, mob: Mob) -> BoundingBox {
        let sig = self.bbox_signature(mob);
        let Some(e) = self.get(mob) else {
            return BoundingBox::ZERO;
        };
        let forced = self.bbox_has_writable_point_view(mob);
        let cached = if forced {
            None
        } else {
            let cache = e.bbox_cell().borrow();
            (cache.signature == Some(sig)).then_some(cache.value) // ubs:ignore -- geometry revision key, not a secret
        };
        if let Some(v) = cached {
            return v;
        }
        let value = self.compute_bounding_box(mob);
        let mut c = e.bbox_cell().borrow_mut();
        c.signature = Some(sig);
        c.value = value;
        c.materializations += 1;
        c.caller_installed = false;
        value
    }

    /// How many times `mob`'s box has actually been recomputed — the laziness
    /// test hook (mirrors `MirrorSet::materializations`).
    #[must_use]
    pub fn bbox_materializations(&self, mob: Mob) -> u64 {
        self.get(mob)
            .map_or(0, |e| e.bbox_cell().borrow().materializations)
    }

    /// Install a caller-computed Reference bounding box under the entry's
    /// current subtree signature.
    ///
    /// Most engine operations deliberately let the lazy cache recompute from
    /// points. The Python compatibility portal uses this narrow seam for
    /// `apply_points_function(..., works_on_bounding_box=True)`, whose public
    /// contract transforms the three cached `[min, mid, max]` rows with the
    /// user callable instead. Any later point, placement, or family mutation
    /// changes the signature and invalidates this value automatically.
    pub fn install_bounding_box_cache(&self, mob: Mob, value: BoundingBox) -> &Self {
        let signature = self.bbox_signature(mob);
        if let Some(entry) = self.get(mob) {
            let mut cache = entry.bbox_cell().borrow_mut();
            cache.signature = Some(signature);
            cache.value = value;
            cache.caller_installed = true;
        }
        self
    }

    /// Return a still-current caller-installed Reference bounding box.
    ///
    /// This distinguishes the compatibility portal's deliberately transformed
    /// `[min, mid, max]` rows from an ordinary lazy materialization without
    /// weakening normal signature-based invalidation.
    #[must_use]
    pub fn installed_bounding_box_cache(&self, mob: Mob) -> Option<BoundingBox> {
        if self.bbox_has_writable_point_view(mob) {
            return None;
        }
        let signature = self.bbox_signature(mob);
        let entry = self.get(mob)?;
        let cache = entry.bbox_cell().borrow();
        (cache.caller_installed && cache.signature == Some(signature)).then_some(cache.value) // ubs:ignore -- geometry revision key, not a secret
    }

    // ---------------------------------------------------- critical-point getters

    /// The critical point of `mob`'s box in a direction (Reference
    /// `get_bounding_box_point`).
    #[must_use]
    pub fn get_bounding_box_point(&self, mob: Mob, dir: Vec3) -> Vec3 {
        self.get_bounding_box(mob).point(dir)
    }

    /// The center of `mob`'s box.
    #[must_use]
    pub fn get_center(&self, mob: Mob) -> Vec3 {
        self.get_bounding_box(mob).center()
    }

    /// Alias of [`get_bounding_box_point`](Self::get_bounding_box_point).
    #[must_use]
    pub fn get_edge_center(&self, mob: Mob, dir: Vec3) -> Vec3 {
        self.get_bounding_box_point(mob, dir)
    }

    /// Alias of [`get_bounding_box_point`](Self::get_bounding_box_point).
    #[must_use]
    pub fn get_corner(&self, mob: Mob, dir: Vec3) -> Vec3 {
        self.get_bounding_box_point(mob, dir)
    }

    /// Left / right / top / bottom / zenith / nadir edge centers.
    #[must_use]
    pub fn get_left(&self, mob: Mob) -> Vec3 {
        self.get_edge_center(mob, LEFT)
    }
    /// See [`get_left`](Self::get_left).
    #[must_use]
    pub fn get_right(&self, mob: Mob) -> Vec3 {
        self.get_edge_center(mob, RIGHT)
    }
    /// See [`get_left`](Self::get_left).
    #[must_use]
    pub fn get_top(&self, mob: Mob) -> Vec3 {
        self.get_edge_center(mob, UP)
    }
    /// See [`get_left`](Self::get_left).
    #[must_use]
    pub fn get_bottom(&self, mob: Mob) -> Vec3 {
        self.get_edge_center(mob, DOWN)
    }
    /// See [`get_left`](Self::get_left).
    #[must_use]
    pub fn get_zenith(&self, mob: Mob) -> Vec3 {
        self.get_edge_center(mob, OUT)
    }
    /// See [`get_left`](Self::get_left).
    #[must_use]
    pub fn get_nadir(&self, mob: Mob) -> Vec3 {
        self.get_edge_center(mob, IN)
    }

    /// The coordinate along `dim` at the box's critical point in `dir`.
    #[must_use]
    pub fn get_coord(&self, mob: Mob, dim: usize, dir: Vec3) -> f64 {
        self.get_bounding_box_point(mob, dir)[dim]
    }
    /// x / y / z at the box center (`get_coord` with `ORIGIN`).
    #[must_use]
    pub fn get_x(&self, mob: Mob) -> f64 {
        self.get_coord(mob, 0, ORIGIN)
    }
    /// See [`get_x`](Self::get_x).
    #[must_use]
    pub fn get_y(&self, mob: Mob) -> f64 {
        self.get_coord(mob, 1, ORIGIN)
    }
    /// See [`get_x`](Self::get_x).
    #[must_use]
    pub fn get_z(&self, mob: Mob) -> f64 {
        self.get_coord(mob, 2, ORIGIN)
    }

    /// Extent of `mob` along an axis.
    #[must_use]
    pub fn length_over_dim(&self, mob: Mob, dim: usize) -> f64 {
        self.get_bounding_box(mob).length_over_dim(dim)
    }
    /// Width (x extent).
    #[must_use]
    pub fn get_width(&self, mob: Mob) -> f64 {
        self.length_over_dim(mob, 0)
    }
    /// Height (y extent).
    #[must_use]
    pub fn get_height(&self, mob: Mob) -> f64 {
        self.length_over_dim(mob, 1)
    }
    /// Depth (z extent).
    #[must_use]
    pub fn get_depth(&self, mob: Mob) -> f64 {
        self.length_over_dim(mob, 2)
    }

    /// The first `point` record (Reference `get_start`), or `None` if empty.
    #[must_use]
    pub fn get_start(&self, mob: Mob) -> Option<Vec3> {
        let e = self.get(mob)?;
        let v = e.buffer.read(0, "point")?;
        Some(
            e.placement()
                .apply_point([f64::from(v[0]), f64::from(v[1]), f64::from(v[2])]),
        )
    }

    /// The last `point` record (Reference `get_end`), or `None` if empty.
    #[must_use]
    pub fn get_end(&self, mob: Mob) -> Option<Vec3> {
        let e = self.get(mob)?;
        let n = e.buffer.len();
        let v = e.buffer.read(n.checked_sub(1)?, "point")?;
        Some(
            e.placement()
                .apply_point([f64::from(v[0]), f64::from(v[1]), f64::from(v[2])]),
        )
    }

    /// Reference `get_start_and_end`. `None` when the entry has no points
    /// (the Reference's `throw_error_if_no_points`).
    #[must_use]
    pub fn get_start_and_end(&self, mob: Mob) -> Option<(Vec3, Vec3)> {
        Some((self.get_start(mob)?, self.get_end(mob)?))
    }

    /// This entry's object-space `point` records in f64 (§6.1).
    ///
    /// Lumen consumes this surface for artifacts keyed on Geometry alone.
    /// Scene-facing geometry queries use [`Stage::get_points`], which applies
    /// the independent object→world placement.
    #[must_use]
    pub fn get_object_points(&self, mob: Mob) -> Option<Vec<Vec3>> {
        let column = self.get(mob)?.buffer.read_column("point")?;
        Some(
            column
                .as_chunks::<3>()
                .0
                .iter()
                .map(|c| [f64::from(c[0]), f64::from(c[1]), f64::from(c[2])])
                .collect(),
        )
    }

    /// Reference `get_points`: this entry's own world-space points in f64,
    /// not recursing into the family.
    #[must_use]
    pub fn get_points(&self, mob: Mob) -> Option<Vec<Vec3>> {
        let entry = self.get(mob)?;
        let placement = entry.placement();
        self.get_object_points(mob).map(|points| {
            points
                .into_iter()
                .map(|point| placement.apply_point(point))
                .collect()
        })
    }

    /// Reference `set_points`: replace this entry with the supplied world-space
    /// points, reset its placement to identity, resize its records
    /// (order-preserving, so the surrounding style columns survive), and write
    /// the run.
    ///
    /// On a vmobject-shaped record the `joint_angle` column is refreshed
    /// from the new path in the same call — the Reference recomputes it
    /// lazily behind `needs_new_joint_angles`, and doing it here keeps the
    /// column true to the points through every channel that can write them.
    /// A point run that violates the shared-anchor invariant is a typed
    /// error, not a silently malformed path.
    ///
    /// Like the Reference's `resize_points`, this refreshes the bounding box
    /// of `mob` and its ancestors: a point-free ancestor returns to the
    /// origin (fm-fc53).
    pub fn set_points(&mut self, mob: Mob, points: &[Vec3]) -> Result<(), StageError> {
        self.write_points(mob, points)?;
        self.refresh_bounding_box(mob, false, true);
        Ok(())
    }

    /// [`Stage::set_points`] without the Reference's bounding-box refresh:
    /// the representation path (placement baking, pointwise maps whose
    /// caller refreshes once afterwards).
    fn write_points(&mut self, mob: Mob, points: &[Vec3]) -> Result<(), StageError> {
        let angles: Option<Vec<f64>> = if self.is_vmobject_schema(mob)? {
            let path = QuadPath::from_points(points.to_vec()).map_err(StageError::Geometry)?;
            Some(path.joint_angles())
        } else {
            None
        };
        let entry = self.get_mut(mob).ok_or(StageError::StaleHandle)?;
        entry
            .buffer
            .resize_preserving_order(points.len())
            .map_err(StageError::Record)?;
        // Resizing preserves the order of every column. Materialize the other
        // pointlike fields at that SAME cardinality before discarding placement;
        // a primary-point edit must not strand normal seeds in object space.
        if !entry.placement().is_identity() {
            let columns = mapped_auxiliary_points(&entry.buffer, entry.placement(), |p| p);
            for (key, values) in columns {
                entry.buffer.write_range(&key, 0, &values);
            }
        }
        entry.set_placement(Placement::IDENTITY);
        #[allow(clippy::cast_possible_truncation)]
        let flat: Vec<f32> = points
            .iter()
            .flat_map(|p| p.iter().map(|v| *v as f32))
            .collect();
        entry.buffer.write_range("point", 0, &flat);
        if let Some(angles) = angles {
            #[allow(clippy::cast_possible_truncation)]
            let flat: Vec<f32> = angles.iter().map(|a| *a as f32).collect();
            entry.buffer.write_range("joint_angle", 0, &flat);
        }
        Ok(())
    }

    /// Whether this entry's records are vmobject-shaped: the `joint_angle`
    /// field is the schema marker (present in `RecordSchema::vmobject`,
    /// absent from the base layout).
    pub(crate) fn is_vmobject_schema(&self, mob: Mob) -> Result<bool, StageError> {
        Ok(self
            .try_get(mob)?
            .buffer
            .schema()
            .offset("joint_angle")
            .is_some())
    }

    /// Reference `put_start_and_end_on` (mobject.py:1299): scale, rotate in
    /// the xy plane, tilt out of it, and shift, so the family's first point
    /// lands on `start` and its last on `end`.
    ///
    /// The Reference raises "Cannot position endpoints of closed loop" when
    /// the current endpoints coincide; that is
    /// [`StageError::DegenerateEndpoints`] here, because the caller
    /// (`Line`, the tip algebra) has a real decision to make about it —
    /// `Line::put_start_and_end_on` overrides exactly this case.
    pub fn put_start_and_end_on(
        &mut self,
        mob: Mob,
        start: Vec3,
        end: Vec3,
    ) -> Result<(), StageError> {
        let (curr_start, curr_end) = self
            .get_start_and_end(mob)
            .ok_or(StageError::DegenerateEndpoints)?;
        let curr_vect = sub(curr_end, curr_start);
        if curr_vect == [0.0; 3] {
            return Err(StageError::DegenerateEndpoints);
        }
        let target_vect = sub(end, start);
        self.scale_about(
            mob,
            space_ops::get_norm(target_vect) / space_ops::get_norm(curr_vect),
            Some(curr_start),
            None,
        );
        self.rotate(
            mob,
            space_ops::angle_of_vector(target_vect) - space_ops::angle_of_vector(curr_vect),
            OUT,
            None,
            None,
        );
        // Out-of-plane tilt, about the axis perpendicular to the target in
        // the xy plane — the Reference's own choice of axis, degenerate
        // axis and all (a purely vertical target rotates about zero, i.e.
        // not at all, which is what it wants).
        let curr_xy = space_ops::get_norm([curr_vect[0], curr_vect[1], 0.0]);
        let target_xy = space_ops::get_norm([target_vect[0], target_vect[1], 0.0]);
        self.rotate(
            mob,
            space_ops::angle_of_vector([curr_xy, curr_vect[2], 0.0])
                - space_ops::angle_of_vector([target_xy, target_vect[2], 0.0]),
            [-target_vect[1], target_vect[0], 0.0],
            None,
            None,
        );
        let now_start = self.get_start(mob).ok_or(StageError::DegenerateEndpoints)?;
        self.shift(mob, sub(start, now_start));
        Ok(())
    }

    // ----------------------------------------------------- the transform core

    /// This entry's object→world placement.
    #[must_use]
    pub fn placement(&self, mob: Mob) -> Option<Placement> {
        self.get(mob).map(|entry| entry.placement())
    }

    /// This entry's independent placement revision.
    #[must_use]
    pub fn placement_revision(&self, mob: Mob) -> Option<u64> {
        self.get(mob).map(|entry| entry.placement_revision())
    }

    /// Replace one entry's placement, advancing only its placement revision
    /// when the affine coefficients changed bit-for-bit.
    pub fn set_placement(
        &mut self,
        mob: Mob,
        placement: Placement,
    ) -> Result<&mut Self, StageError> {
        self.get_mut(mob)
            .ok_or(StageError::StaleHandle)?
            .set_placement(placement);
        Ok(self)
    }

    /// Bake every declared 3D pointlike field into the authoritative records
    /// and reset placement to identity, including surface normal-control points.
    ///
    /// Arbitrary pointwise maps need this boundary because their result is not
    /// generally representable by one affine map. Affine positional operations
    /// never call it.
    pub fn bake_placement(&mut self, mob: Mob) -> Result<bool, StageError> {
        let entry = self.get(mob).ok_or(StageError::StaleHandle)?;
        let placement = entry.placement();
        // An empty record has nowhere to store a baked anchor. Keep its
        // placement, including when a Python points/data view is requested.
        if placement.is_identity() || entry.buffer.is_empty() {
            return Ok(false);
        }
        let Some(points) = self.get_points(mob) else {
            // Custom records can contain pointlike fields without a primary
            // `point`. Those fields still own world-space values on readback.
            let entry = self.get_mut(mob).ok_or(StageError::StaleHandle)?;
            let columns = mapped_auxiliary_points(&entry.buffer, placement, |p| p);
            for (key, values) in columns {
                entry.buffer.write_range(&key, 0, &values);
            }
            entry.set_placement(Placement::IDENTITY);
            return Ok(true);
        };
        // A representation change, not a Reference point edit: no refresh.
        self.write_points(mob, &points)?;
        Ok(true)
    }

    /// [`Stage::bake_placement`] over a whole family.
    pub fn bake_family_placements(&mut self, mob: Mob) -> Result<(), StageError> {
        let family = self.family(mob);
        if family.is_empty() {
            return Err(StageError::StaleHandle);
        }
        for member in family {
            self.bake_placement(member)?;
        }
        Ok(())
    }

    /// Resolve the pivot the Reference's `apply_points_function` uses: an
    /// explicit `about_point` wins; else `about_edge` (if given) resolves to the
    /// box's critical point there; else there is no pivot (transform in place).
    fn resolve_pivot(
        &self,
        mob: Mob,
        about_point: Option<Vec3>,
        about_edge: Option<Vec3>,
    ) -> Option<Vec3> {
        match (about_point, about_edge) {
            (Some(p), _) => Some(p),
            (None, Some(e)) => Some(self.get_bounding_box_point(mob, e)),
            (None, None) => None,
        }
    }

    /// Apply `f` to every point of `mob`'s family, optionally about `pivot`
    /// (`f(p - pivot) + pivot`). The bounding box invalidates automatically via
    /// the record revision bump.
    fn transform_points<F: Fn(Vec3) -> Vec3>(&mut self, mob: Mob, pivot: Option<Vec3>, f: F) {
        for member in self.family(mob) {
            let _ = self.map_world_pointlikes(member, |point| match pivot {
                Some(pv) => add(f(sub(point, pv)), pv),
                None => f(point),
            });
        }
    }

    /// Apply one map to the complete record geometry without an intermediate
    /// f32 bake. Retain the existing primary-point arithmetic and joint refresh,
    /// and publish auxiliary columns through the same live-view generation.
    fn map_world_pointlikes(
        &mut self,
        mob: Mob,
        map: impl Fn(Vec3) -> Vec3,
    ) -> Result<(), StageError> {
        let points = self
            .get_points(mob)
            .map(|points| points.into_iter().map(&map).collect::<Vec<_>>());
        let entry = self.get(mob).ok_or(StageError::StaleHandle)?;
        let columns = mapped_auxiliary_points(&entry.buffer, entry.placement(), &map);
        if let Some(points) = points {
            self.write_points(mob, &points)?;
        } else {
            self.set_placement(mob, Placement::IDENTITY)?;
        }
        let entry = self.get_mut(mob).ok_or(StageError::StaleHandle)?;
        for (key, values) in columns {
            entry.buffer.write_range(&key, 0, &values);
        }
        Ok(())
    }

    fn apply_affine_many(&mut self, mobs: &[Mob], affine: Placement) -> &mut Self {
        let mut members = Vec::new();
        for &mob in mobs {
            for member in self.family(mob) {
                if !members.contains(&member) {
                    members.push(member);
                }
            }
        }
        for member in members {
            let has_live_view = self.get(member).is_some_and(|entry| {
                !entry.buffer.is_empty() && entry.buffer.live_view_count() > 0
            });
            if has_live_view {
                let _ = self.map_world_pointlikes(member, |point| affine.apply_point(point));
                continue;
            }
            if let Some(entry) = self.get_mut(member) {
                entry.set_placement(affine.compose(entry.placement()));
            }
        }
        self
    }

    // ------------------------------------------- point-free positions (fm-fc53)
    //
    // The Reference keeps one cached `[min, mid, max]` box per mobject
    // (mobject.py:338). A point-less family's box is that cache: the
    // box-preserving maps (`shift`, `scale`, `stretch`, and everything built
    // on them) transform it in place, and a refresh recomputes it, which for
    // a family without points is the origin (mobject.py:344). Here an entry
    // without records keeps that position as its placement's translation, so
    // the refresh is a placement reset. An entry WITH records is never
    // touched: its box is always recomputed from its points.

    /// Every transitive parent of `mob`, each once.
    fn ancestors(&self, mob: Mob) -> Vec<Mob> {
        let mut out: Vec<Mob> = Vec::new();
        let mut pending: Vec<Mob> = self
            .get(mob)
            .map(|entry| entry.parents().to_vec())
            .unwrap_or_default();
        while let Some(current) = pending.pop() {
            if out.contains(&current) {
                continue;
            }
            if let Some(entry) = self.get(current) {
                pending.extend_from_slice(entry.parents());
                out.push(current);
            }
        }
        out
    }

    /// Return a record-less entry to the origin; entries with records keep
    /// their placement.
    fn reset_point_free_anchor(&mut self, mob: Mob) {
        if let Some(entry) = self.get_mut(mob)
            && entry.buffer.is_empty()
        {
            entry.set_placement(Placement::IDENTITY);
        }
    }

    /// Reference `refresh_bounding_box(recurse_down, recurse_up)`
    /// (mobject.py:362): recompute the box of `mob` (and its whole family
    /// with `recurse_down`, and every ancestor with `recurse_up`).
    ///
    /// Boxes over points are recomputed lazily anyway, so the observable
    /// effect is on point-free families: their retained position returns to
    /// the origin, where the Reference's recomputed empty box is.
    pub fn refresh_bounding_box(
        &mut self,
        mob: Mob,
        recurse_down: bool,
        recurse_up: bool,
    ) -> &mut Self {
        let mut members = if recurse_down {
            self.family(mob)
        } else {
            vec![mob]
        };
        if recurse_up {
            members.extend(self.ancestors(mob));
        }
        for member in members {
            self.reset_point_free_anchor(member);
        }
        self
    }

    /// The Reference's `for parent in self.parents: parent.refresh_bounding_box()`
    /// after a box-preserving map (mobject.py:306).
    fn refresh_ancestor_boxes(&mut self, mob: Mob) -> &mut Self {
        for ancestor in self.ancestors(mob) {
            self.reset_point_free_anchor(ancestor);
        }
        self
    }

    /// Place a record-less entry's degenerate box at `point`: the Reference's
    /// cached box after `apply_points_function(..., works_on_bounding_box=True)`
    /// maps it through an arbitrary function. Returns `false`, changing
    /// nothing, for an entry with records.
    ///
    /// # Errors
    /// [`StageError::StaleHandle`].
    pub fn set_point_free_anchor(&mut self, mob: Mob, point: Vec3) -> Result<bool, StageError> {
        let entry = self.get_mut(mob).ok_or(StageError::StaleHandle)?;
        if !entry.buffer.is_empty() {
            return Ok(false);
        }
        entry.set_placement(Placement::from_translation(point));
        Ok(true)
    }

    /// Compose one world-space affine operation over every family member.
    ///
    /// `affine` acts after each member's current placement. Object-space points
    /// and their field revisions stay untouched unless a live record view pins
    /// the authoritative buffer. In that case the operation bakes through the
    /// existing generation so V3's engine-write visibility remains true for
    /// zero-copy NumPy views.
    ///
    /// A general map is the Reference's `apply_matrix`, which recomputes
    /// every box afterwards: point-free family members and ancestors return
    /// to the origin ([`Stage::refresh_bounding_box`]).
    pub fn apply_affine(&mut self, mob: Mob, affine: Placement) -> &mut Self {
        self.apply_affine_many(&[mob], affine);
        self.refresh_bounding_box(mob, true, true)
    }

    /// The Reference's `apply_points_function` made public: apply `f` to
    /// every family point about the resolved pivot (an explicit
    /// `about_point` wins; else the box point at `about_edge`; both `None`
    /// transforms in place). `Mobject.apply_function`'s default is
    /// `about_point = ORIGIN` — pass `Some(ORIGIN)` for that surface.
    /// Boxes are recomputed afterwards, as in the Reference
    /// (`works_on_bounding_box=False`): point-free members and ancestors
    /// return to the origin.
    pub fn apply_points_function(
        &mut self,
        mob: Mob,
        f: impl Fn(Vec3) -> Vec3,
        about_point: Option<Vec3>,
        about_edge: Option<Vec3>,
    ) -> &mut Self {
        let pivot = self.resolve_pivot(mob, about_point, about_edge);
        self.transform_points(mob, pivot, f);
        self.refresh_bounding_box(mob, true, true)
    }

    /// Reference `rotate` (mobject.py): rotate the family about `axis` by
    /// `angle`, pivoting on `about_point` (or the box point at
    /// `about_edge`; both `None` defaults to the box center, the
    /// Reference's `about_edge = ORIGIN`). Rotation is not a box-preserving
    /// map, so the Reference recomputes every box: a point-free family
    /// returns to the origin (fm-fc53).
    pub fn rotate(
        &mut self,
        mob: Mob,
        angle: f64,
        axis: Vec3,
        about_point: Option<Vec3>,
        about_edge: Option<Vec3>,
    ) -> &mut Self {
        let m = fmn_geom::rotation_matrix(angle, axis);
        let edge = if about_point.is_none() && about_edge.is_none() {
            Some(ORIGIN)
        } else {
            about_edge
        };
        let pivot = self.resolve_pivot(mob, about_point, edge);
        let affine = pivot.map_or_else(|| Placement::new(m, [0.0; 3]), |p| Placement::about(m, p));
        self.apply_affine(mob, affine);
        self
    }

    /// Reference `match_points` (mobject.py:311): resize `mob`'s records
    /// to `source`'s count (order-preserving) and copy every pointlike
    /// column. Applies member-for-member is the *caller's* concern — this
    /// is the single-entry rule, exactly the Reference's.
    pub fn match_points(&mut self, mob: Mob, source: Mob) {
        let (len, placement, columns): (usize, Placement, Vec<(String, Vec<f32>)>) =
            match self.get(source) {
                Some(entry) => (
                    entry.buffer.len(),
                    entry.placement(),
                    entry
                        .buffer
                        .schema()
                        .pointlike_keys()
                        .iter()
                        .filter_map(|k| entry.buffer.read_column(k).map(|c| (k.clone(), c)))
                        .collect(),
                ),
                None => return,
            };
        let Some(entry) = self.get_mut(mob) else {
            return;
        };
        if entry.buffer.len() != len {
            // An impossible target size (fm-vek.2) degrades to the same
            // silent no-op as the writes below, exactly like the other
            // infallible plumbing paths.
            let _ = entry.buffer.resize_preserving_order(len);
        }
        for (field, column) in columns {
            if entry.buffer.read_column(&field).as_deref() != Some(column.as_slice()) {
                entry.buffer.write_range(&field, 0, &column);
            }
        }
        entry.set_placement(placement);
        // The Reference resizes through `resize_points`, which refreshes this
        // box and every ancestor's: matching an empty source leaves no
        // position behind (fm-fc53).
        self.refresh_bounding_box(mob, false, true);
    }

    // ---------------------------------------------------------- positional API

    /// Shift every point by `vector`. A point-free family keeps its position
    /// and moves with it; its point-free ancestors' boxes are refreshed, as
    /// the Reference's `apply_points_function` refreshes `self.parents`.
    pub fn shift(&mut self, mob: Mob, vector: Vec3) -> &mut Self {
        self.apply_affine_many(&[mob], Placement::from_translation(vector));
        self.refresh_ancestor_boxes(mob)
    }

    /// Shift the union of several families, visiting shared members once.
    ///
    /// This is the aggregate `Group(*mobjects).shift(...)` rule used by
    /// interactive multi-selection: overlapping selections and diamond-shared
    /// descendants must not receive the same translation more than once.
    pub fn shift_many(&mut self, mobs: &[Mob], vector: Vec3) -> &mut Self {
        self.apply_affine_many(mobs, Placement::from_translation(vector))
    }

    /// Scale about the box center (Reference default `about_edge=ORIGIN`).
    pub fn scale(&mut self, mob: Mob, factor: f64) -> &mut Self {
        self.scale_about(mob, factor, None, Some(ORIGIN))
    }

    /// Scale about an explicit `about_point` or `about_edge` (the Reference's
    /// full `scale` surface). `factor` is clamped to `MIN_SCALE_FACTOR`.
    pub fn scale_about(
        &mut self,
        mob: Mob,
        factor: f64,
        about_point: Option<Vec3>,
        about_edge: Option<Vec3>,
    ) -> &mut Self {
        let factor = factor.max(MIN_SCALE_FACTOR);
        let pivot = self.resolve_pivot(mob, about_point, about_edge);
        let linear = [[factor, 0.0, 0.0], [0.0, factor, 0.0], [0.0, 0.0, factor]];
        let affine = pivot.map_or_else(
            || Placement::new(linear, [0.0; 3]),
            |p| Placement::about(linear, p),
        );
        // Box-preserving (`works_on_bounding_box=True`): a point-free family
        // scales its position about the pivot; only ancestors refresh.
        self.apply_affine_many(&[mob], affine);
        self.refresh_ancestor_boxes(mob)
    }

    /// Uniformly scale the union of several families about one shared point.
    ///
    /// Family members reachable through multiple roots are transformed once.
    pub fn scale_many_about_point(
        &mut self,
        mobs: &[Mob],
        factor: f64,
        about_point: Vec3,
    ) -> &mut Self {
        let factor = factor.max(MIN_SCALE_FACTOR);
        let linear = [[factor, 0.0, 0.0], [0.0, factor, 0.0], [0.0, 0.0, factor]];
        self.apply_affine_many(mobs, Placement::about(linear, about_point))
    }

    /// Stretch along one axis about the box center.
    pub fn stretch(&mut self, mob: Mob, factor: f64, dim: usize) -> &mut Self {
        self.stretch_about(mob, factor, dim, None, Some(ORIGIN))
    }

    /// Stretch along one axis about an explicit pivot.
    pub fn stretch_about(
        &mut self,
        mob: Mob,
        factor: f64,
        dim: usize,
        about_point: Option<Vec3>,
        about_edge: Option<Vec3>,
    ) -> &mut Self {
        let pivot = self.resolve_pivot(mob, about_point, about_edge);
        let mut linear: Mat3 = Placement::IDENTITY.linear();
        linear[dim][dim] = factor;
        let affine = pivot.map_or_else(
            || Placement::new(linear, [0.0; 3]),
            |p| Placement::about(linear, p),
        );
        // Box-preserving, like `scale_about`.
        self.apply_affine_many(&[mob], affine);
        self.refresh_ancestor_boxes(mob)
    }

    /// Stretch the union of several families along one axis about one point.
    ///
    /// Family members reachable through multiple roots are transformed once.
    pub fn stretch_many_about_point(
        &mut self,
        mobs: &[Mob],
        factor: f64,
        dim: usize,
        about_point: Vec3,
    ) -> &mut Self {
        let mut linear: Mat3 = Placement::IDENTITY.linear();
        linear[dim][dim] = factor;
        self.apply_affine_many(mobs, Placement::about(linear, about_point))
    }

    /// Move the box center to the origin.
    pub fn center(&mut self, mob: Mob) -> &mut Self {
        let c = self.get_center(mob);
        self.shift(mob, neg(c))
    }

    /// Align a side/corner to the frame border with `buff` (Reference
    /// `align_on_border`, the engine of `to_edge`/`to_corner`).
    fn align_on_border(&mut self, mob: Mob, dir: Vec3, buff: f64) -> &mut Self {
        let radius = [FRAME_X_RADIUS, FRAME_Y_RADIUS, 0.0];
        let s = np_sign(dir);
        let target = hadamard(s, radius);
        let point_to_align = self.get_bounding_box_point(mob, dir);
        let mut shift_val = sub(sub(target, point_to_align), scaled(dir, buff));
        // Zero out axes the direction does not touch.
        shift_val = hadamard(shift_val, np_abs(s));
        self.shift(mob, shift_val)
    }

    /// Move `mob` against the frame edge `edge` with buffer `buff`.
    pub fn to_edge(&mut self, mob: Mob, edge: Vec3, buff: f64) -> &mut Self {
        self.align_on_border(mob, edge, buff)
    }

    /// Move `mob` into the frame corner `corner` with buffer `buff`.
    pub fn to_corner(&mut self, mob: Mob, corner: Vec3, buff: f64) -> &mut Self {
        self.align_on_border(mob, corner, buff)
    }

    /// Reference `shift_onto_screen(buff=…)`: for each of `UP`, `DOWN`,
    /// `LEFT`, `RIGHT` in that order, an edge centre beyond the frame's
    /// half-extent less `buff` is pulled back with [`Stage::to_edge`].
    pub fn shift_onto_screen(&mut self, mob: Mob, buff: f64) -> &mut Self {
        for (vect, radius) in [
            (UP, FRAME_Y_RADIUS),
            (DOWN, FRAME_Y_RADIUS),
            (LEFT, FRAME_X_RADIUS),
            (RIGHT, FRAME_X_RADIUS),
        ] {
            let edge = self.get_bounding_box_point(mob, vect);
            if space_ops::dot(edge, vect) > radius - buff {
                self.to_edge(mob, vect, buff);
            }
        }
        self
    }

    /// Position `mob` next to a target, on the `direction` side, `buff` away,
    /// aligning along `aligned_edge` (Reference `next_to`, `coor_mask = 1`).
    pub fn next_to(
        &mut self,
        mob: Mob,
        target: impl Into<PosTarget>,
        direction: Vec3,
        buff: f64,
        aligned_edge: Vec3,
    ) -> &mut Self {
        let target_point = match target.into() {
            PosTarget::Mob(m) => self.get_bounding_box_point(m, add(aligned_edge, direction)),
            PosTarget::Point(p) => p,
        };
        let point_to_align = self.get_bounding_box_point(mob, sub(aligned_edge, direction));
        self.shift(
            mob,
            add(sub(target_point, point_to_align), scaled(direction, buff)),
        )
    }

    /// Move `mob` so its critical point at `aligned_edge` coincides with the
    /// target's (Reference `move_to`, `coor_mask = 1`).
    pub fn move_to(
        &mut self,
        mob: Mob,
        target: impl Into<PosTarget>,
        aligned_edge: Vec3,
    ) -> &mut Self {
        let target_point = match target.into() {
            PosTarget::Mob(m) => self.get_bounding_box_point(m, aligned_edge),
            PosTarget::Point(p) => p,
        };
        let point_to_align = self.get_bounding_box_point(mob, aligned_edge);
        self.shift(mob, sub(target_point, point_to_align))
    }

    /// Align `mob` to a target along the nonzero components of `direction`
    /// (Reference `align_to`).
    pub fn align_to(
        &mut self,
        mob: Mob,
        target: impl Into<PosTarget>,
        direction: Vec3,
    ) -> &mut Self {
        let point = match target.into() {
            PosTarget::Mob(m) => self.get_bounding_box_point(m, direction),
            PosTarget::Point(p) => p,
        };
        for dim in 0..3 {
            if direction[dim] != 0.0 {
                self.set_coord(mob, point[dim], dim, direction);
            }
        }
        self
    }

    /// Set the coordinate along `dim` (at the box's critical point in `dir`) to
    /// `value` by shifting (Reference `set_coord`).
    pub fn set_coord(&mut self, mob: Mob, value: f64, dim: usize, dir: Vec3) -> &mut Self {
        let curr = self.get_coord(mob, dim, dir);
        let mut shift_vect = [0.0; 3];
        shift_vect[dim] = value - curr;
        self.shift(mob, shift_vect)
    }

    /// Set the x/y/z of the box center.
    pub fn set_x(&mut self, mob: Mob, x: f64) -> &mut Self {
        self.set_coord(mob, x, 0, ORIGIN)
    }
    /// See [`set_x`](Self::set_x).
    pub fn set_y(&mut self, mob: Mob, y: f64) -> &mut Self {
        self.set_coord(mob, y, 1, ORIGIN)
    }
    /// See [`set_x`](Self::set_x).
    pub fn set_z(&mut self, mob: Mob, z: f64) -> &mut Self {
        self.set_coord(mob, z, 2, ORIGIN)
    }

    /// Rescale so the extent along `dim` becomes `length`, by scaling (uniform)
    /// or stretching (single-axis). No-op if the current extent is zero.
    pub fn rescale_to_fit(
        &mut self,
        mob: Mob,
        length: f64,
        dim: usize,
        stretch: bool,
    ) -> &mut Self {
        let old = self.length_over_dim(mob, dim);
        if old == 0.0 {
            return self;
        }
        if stretch {
            self.stretch(mob, length / old, dim)
        } else {
            self.scale(mob, length / old)
        }
    }

    /// Set the width (uniform scale unless `stretch`).
    pub fn set_width(&mut self, mob: Mob, width: f64, stretch: bool) -> &mut Self {
        self.rescale_to_fit(mob, width, 0, stretch)
    }
    /// Set the height (uniform scale unless `stretch`).
    pub fn set_height(&mut self, mob: Mob, height: f64, stretch: bool) -> &mut Self {
        self.rescale_to_fit(mob, height, 1, stretch)
    }
    /// Set the depth (uniform scale unless `stretch`).
    pub fn set_depth(&mut self, mob: Mob, depth: f64, stretch: bool) -> &mut Self {
        self.rescale_to_fit(mob, depth, 2, stretch)
    }
    /// Stretch (single-axis) to fit a width.
    pub fn stretch_to_fit_width(&mut self, mob: Mob, width: f64) -> &mut Self {
        self.rescale_to_fit(mob, width, 0, true)
    }
    /// Stretch (single-axis) to fit a height.
    pub fn stretch_to_fit_height(&mut self, mob: Mob, height: f64) -> &mut Self {
        self.rescale_to_fit(mob, height, 1, true)
    }
    /// Stretch (single-axis) to fit a depth.
    pub fn stretch_to_fit_depth(&mut self, mob: Mob, depth: f64) -> &mut Self {
        self.rescale_to_fit(mob, depth, 2, true)
    }

    /// Match another mobject's extent along `dim` (uniform scale).
    pub fn match_dim_size(&mut self, mob: Mob, other: Mob, dim: usize) -> &mut Self {
        let length = self.length_over_dim(other, dim);
        self.rescale_to_fit(mob, length, dim, false)
    }
    /// Match another mobject's width.
    pub fn match_width(&mut self, mob: Mob, other: Mob) -> &mut Self {
        self.match_dim_size(mob, other, 0)
    }
    /// Match another mobject's height.
    pub fn match_height(&mut self, mob: Mob, other: Mob) -> &mut Self {
        self.match_dim_size(mob, other, 1)
    }
    /// Match another mobject's depth.
    pub fn match_depth(&mut self, mob: Mob, other: Mob) -> &mut Self {
        self.match_dim_size(mob, other, 2)
    }
    /// Match another mobject's coordinate along `dim` (at critical point `dir`).
    pub fn match_coord(&mut self, mob: Mob, other: Mob, dim: usize, dir: Vec3) -> &mut Self {
        let coord = self.get_coord(other, dim, dir);
        self.set_coord(mob, coord, dim, dir)
    }
    /// Match another mobject's x.
    pub fn match_x(&mut self, mob: Mob, other: Mob) -> &mut Self {
        self.match_coord(mob, other, 0, ORIGIN)
    }
    /// Match another mobject's y.
    pub fn match_y(&mut self, mob: Mob, other: Mob) -> &mut Self {
        self.match_coord(mob, other, 1, ORIGIN)
    }
    /// Match another mobject's z.
    pub fn match_z(&mut self, mob: Mob, other: Mob) -> &mut Self {
        self.match_coord(mob, other, 2, ORIGIN)
    }

    /// Lay out `mob`'s direct submobjects in a line, each `next_to` the last
    /// along `direction` with buffer `buff`; recenter the group if `center`
    /// (Reference `arrange`).
    pub fn arrange(&mut self, mob: Mob, direction: Vec3, buff: f64, center: bool) -> &mut Self {
        let subs = self
            .get(mob)
            .map(|e| e.submobjects().to_vec())
            .unwrap_or_default();
        for pair in subs.windows(2) {
            self.next_to(pair[1], PosTarget::Mob(pair[0]), direction, buff, ORIGIN);
        }
        if center {
            self.center(mob);
        }
        self
    }

    /// Lay out `mob`'s direct submobjects in a grid of `n_rows × n_cols`, each
    /// cell `x_unit`/`y_unit` apart (the widest/tallest submobject plus the
    /// buffer), filling rows first, then recentering (Reference
    /// `arrange_in_grid`, the common path).
    pub fn arrange_in_grid(
        &mut self,
        mob: Mob,
        n_rows: usize,
        n_cols: usize,
        h_buff: f64,
        v_buff: f64,
        aligned_edge: Vec3,
    ) -> &mut Self {
        let subs = self
            .get(mob)
            .map(|e| e.submobjects().to_vec())
            .unwrap_or_default();
        if subs.is_empty() || n_cols == 0 {
            return self;
        }
        let max_w = subs
            .iter()
            .map(|&s| self.get_width(s))
            .fold(0.0_f64, f64::max);
        let max_h = subs
            .iter()
            .map(|&s| self.get_height(s))
            .fold(0.0_f64, f64::max);
        let x_unit = h_buff + max_w;
        let y_unit = v_buff + max_h;
        let _ = n_rows; // rows are implied by fill-rows-first over n_cols
        for (index, &s) in subs.iter().enumerate() {
            let x = (index % n_cols) as f64;
            let y = (index / n_cols) as f64;
            self.move_to(s, ORIGIN, aligned_edge);
            self.shift(s, add(scaled(RIGHT, x * x_unit), scaled(DOWN, y * y_unit)));
        }
        self.center(mob)
    }

    // ------------------------------------------------------------- uniforms

    /// Read `mob`'s uniform inventory.
    #[must_use]
    pub fn uniforms(&self, mob: Mob) -> Option<&Uniforms> {
        self.get(mob).map(crate::stage::Entry::uniforms)
    }

    /// Mutable access to `mob`'s uniform inventory (scene code writes
    /// `mobject.uniforms` directly — the future Python bridge surface).
    pub fn uniforms_mut(&mut self, mob: Mob) -> Option<&mut Uniforms> {
        self.get_mut(mob).map(crate::stage::Entry::uniforms_mut)
    }

    /// Apply a mutation to the uniform inventory of `mob` (and its family if
    /// `recurse`), matching the Reference's family-recursing `set_uniform`.
    fn edit_uniforms(&mut self, mob: Mob, recurse: bool, f: impl Fn(&mut Uniforms)) -> &mut Self {
        let targets = if recurse { self.family(mob) } else { vec![mob] };
        for m in targets {
            if let Some(e) = self.get_mut(m) {
                f(e.uniforms_mut());
            }
        }
        self
    }

    /// Set the anti-alias width across the family (Reference
    /// `set_anti_alias_width`).
    pub fn set_anti_alias_width(&mut self, mob: Mob, width: f64, recurse: bool) -> &mut Self {
        self.edit_uniforms(mob, recurse, |u| u.anti_alias_width = width)
    }
    /// Set the `[reflectiveness, gloss, shadow]` shading triple.
    pub fn set_shading(&mut self, mob: Mob, shading: Vec3, recurse: bool) -> &mut Self {
        self.edit_uniforms(mob, recurse, |u| u.shading = shading)
    }
    /// Set the flat-stroke flag.
    pub fn set_flat_stroke(&mut self, mob: Mob, flat: bool, recurse: bool) -> &mut Self {
        self.edit_uniforms(mob, recurse, |u| u.flat_stroke = flat)
    }
    /// Set whether the stroke scales with zoom.
    pub fn set_scale_stroke_with_zoom(
        &mut self,
        mob: Mob,
        value: bool,
        recurse: bool,
    ) -> &mut Self {
        self.edit_uniforms(mob, recurse, |u| u.scale_stroke_with_zoom = value)
    }
    /// Set whether the stroke draws behind the fill.
    pub fn set_stroke_behind(&mut self, mob: Mob, behind: bool, recurse: bool) -> &mut Self {
        self.edit_uniforms(mob, recurse, |u| u.stroke_behind = behind)
    }
    /// Set the `is_fixed_in_frame` float mix (kept camera model).
    pub fn set_fixed_in_frame(&mut self, mob: Mob, value: f64, recurse: bool) -> &mut Self {
        self.edit_uniforms(mob, recurse, |u| u.is_fixed_in_frame = value)
    }
    /// C-7: accepted no-op. Records the flag across the family but changes no
    /// rendered bits.
    pub fn use_winding_fill(&mut self, mob: Mob, value: bool, recurse: bool) -> &mut Self {
        self.edit_uniforms(mob, recurse, |u| {
            u.use_winding_fill(value);
        })
    }

    /// C-2 / BN-07: read the **correct** uniform (not `flat_stroke`, as the
    /// Reference's buggy `get_scale_stroke_with_zoom` does).
    #[must_use]
    pub fn get_scale_stroke_with_zoom(&self, mob: Mob) -> bool {
        self.get(mob)
            .is_some_and(|e| e.uniforms().get_scale_stroke_with_zoom())
    }
}
