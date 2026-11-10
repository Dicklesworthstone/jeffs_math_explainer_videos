//! Family and point alignment — the Transform mechanism's data plane
//! (§9.4, fm-cye), ported from the pinned Reference exactly:
//!
//! - `align_data_and_family` runs **`align_family` first, then
//!   `align_data`** (mobject.py:1741 — the order is semantics).
//! - `align_family` reconciles submobject *counts* recursively via
//!   `add_n_more_submobjects` (mobject.py:1757/1777): a childless mobject
//!   grows single-point copies of itself at its center; a mobject with
//!   children pads by emitting each child once plus invisible
//!   (opacity-zero) copies, distributed by the Reference's
//!   `repeat_indices = arange(target) * curr // target` rule.
//! - `align_points` dispatches by record schema: plain records
//!   null-align by `resize_preserving_order` (mobject.py:1751); vmobject
//!   records (marked by a `joint_angle` field) run the bezier-aware
//!   algorithm (vectorized_mobject.py:964): subpaths split, sorted
//!   descending by polyline length, missing subpaths synthesized by
//!   folding the largest back on itself, per-pair curve counts equalized
//!   by greedy longest-curve insertion, subpath breaks re-marked with a
//!   repeated anchor, and joint angles refreshed.
//!
//! `push_self_into_submobjects` exists in the Reference but has no call
//! site in the pinned tree; it is deliberately not ported.

use fmn_core::types::Vec3;
use fmn_geom::{DEFAULT_TOLERANCE_FOR_POINT_EQUALITY, QuadPath, bezier};

use crate::StageError;
use crate::stage::{Mob, Stage};

/// Maximum direct-child fan-out that family alignment may produce at one
/// node. Alignment recursively copies child families, so the public operation
/// needs a deterministic ceiling before it begins cloning or allocating.
pub const MAX_ALIGNED_SUBMOBJECTS: usize = 65_536;

fn checked_aligned_submobject_count(
    current: usize,
    additional: usize,
) -> Result<usize, StageError> {
    let requested =
        current
            .checked_add(additional)
            .ok_or(StageError::SubmobjectBudgetExceeded {
                requested: usize::MAX,
                max: MAX_ALIGNED_SUBMOBJECTS,
            })?;
    if requested > MAX_ALIGNED_SUBMOBJECTS {
        return Err(StageError::SubmobjectBudgetExceeded {
            requested,
            max: MAX_ALIGNED_SUBMOBJECTS,
        });
    }
    Ok(requested)
}

/// Euclidean distance between two points (local helper; fmn-geom's vector
/// utilities are crate-private).
fn dist(a: Vec3, b: Vec3) -> f64 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// Polyline length over the raw shared-anchor run — the Reference's
/// subpath sort key sums consecutive point distances, handles included.
fn polyline_length(points: &[Vec3]) -> f64 {
    points.windows(2).map(|w| dist(w[0], w[1])).sum()
}

fn is_vmobject_schema(stage: &Stage, mob: Mob) -> Result<bool, StageError> {
    stage.is_vmobject_schema(mob)
}

fn read_points(stage: &Stage, mob: Mob) -> Result<Vec<Vec3>, StageError> {
    stage.get_points(mob).ok_or(StageError::StaleHandle)
}

fn refresh_joint_angles(stage: &mut Stage, mob: Mob) -> Result<(), StageError> {
    let points = stage
        .get_object_points(mob)
        .ok_or(StageError::StaleHandle)?;
    if points.is_empty() {
        return Ok(());
    }
    let angles = QuadPath::from_points(points)
        .map_err(StageError::Geometry)?
        .joint_angles();
    #[allow(clippy::cast_possible_truncation)]
    let flat: Vec<f32> = angles.iter().map(|angle| *angle as f32).collect();
    let entry = stage.get_mut(mob).ok_or(StageError::StaleHandle)?;
    if entry.buffer.read_column("joint_angle").as_deref() != Some(flat.as_slice()) {
        entry.buffer.write_range("joint_angle", 0, &flat);
    }
    Ok(())
}

/// Resize the whole record run preserving order, then write the new point
/// run and (for vmobject records) refreshed joint angles.
fn write_points(stage: &mut Stage, mob: Mob, points: &[Vec3]) -> Result<(), StageError> {
    stage.set_points(mob, points)
}

impl Stage {
    /// Reference `is_aligned_with` (mobject.py:1731): equal record counts,
    /// equal submobject counts, recursively.
    #[must_use]
    pub fn is_aligned_with(&self, a: Mob, b: Mob) -> bool {
        let (Some(ea), Some(eb)) = (self.get(a), self.get(b)) else {
            return false;
        };
        ea.render_primitive() == eb.render_primitive()
            && ea.buffer.schema() == eb.buffer.schema()
            && ea.buffer.len() == eb.buffer.len()
            && ea.submobjects().len() == eb.submobjects().len()
            && ea
                .submobjects()
                .to_vec()
                .iter()
                .zip(eb.submobjects().to_vec().iter())
                .all(|(&sa, &sb)| self.is_aligned_with(sa, sb))
    }

    /// Reference `align_family` (mobject.py:1757): pad the smaller side's
    /// submobject count with `add_n_more_submobjects`, then recurse over
    /// the zipped children.
    ///
    /// # Errors
    /// [`StageError::StaleHandle`].
    pub fn align_family(&mut self, a: Mob, b: Mob) -> Result<(), StageError> {
        let n1 = self.try_get(a)?.submobjects().len();
        let n2 = self.try_get(b)?.submobjects().len();
        if n1 != n2 {
            self.add_n_more_submobjects(a, n2.saturating_sub(n1))?;
            self.add_n_more_submobjects(b, n1.saturating_sub(n2))?;
        }
        let ca = self.try_get(a)?.submobjects().to_vec();
        let cb = self.try_get(b)?.submobjects().to_vec();
        for (&sa, &sb) in ca.iter().zip(cb.iter()) {
            self.align_family(sa, sb)?;
        }
        Ok(())
    }

    /// Reference `add_n_more_submobjects` (mobject.py:1777). Childless:
    /// `n` single-point copies of `mob` at its center. Otherwise each
    /// existing child is kept and padded with invisible (opacity-zero)
    /// copies of itself, counts distributed by
    /// `repeat_indices = arange(curr + n) * curr // (curr + n)`.
    ///
    /// # Errors
    /// [`StageError::StaleHandle`] or
    /// [`StageError::SubmobjectBudgetExceeded`].
    pub fn add_n_more_submobjects(&mut self, mob: Mob, n: usize) -> Result<(), StageError> {
        if n == 0 {
            return Ok(());
        }
        let children = self.try_get(mob)?.submobjects().to_vec();
        let curr = children.len();
        let target = checked_aligned_submobject_count(curr, n)?;
        if curr == 0 {
            let center = self.get_center(mob);
            for _ in 0..target {
                let copy = self.copy_family(mob)?;
                write_points(self, copy, &[center])?;
                self.attach(mob, copy).expect("fresh leaf copy is acyclic");
            }
            return Ok(());
        }
        let mut split_factors = vec![0usize; curr];
        for i in 0..target {
            split_factors[i * curr / target] += 1;
        }
        let mut new_children: Vec<Mob> = Vec::with_capacity(target);
        for (&child, &sf) in children.iter().zip(split_factors.iter()) {
            new_children.push(child);
            for _ in 1..sf {
                let ghost = self.copy_family(child)?;
                self.set_family_opacity_zero(ghost);
                new_children.push(ghost);
            }
        }
        for &child in &children {
            self.detach(mob, child);
        }
        for &child in &new_children {
            self.attach(mob, child).expect("padding copies are acyclic");
        }
        Ok(())
    }

    /// Reference `set_opacity`: write the alpha lane of every `*rgba` field
    /// across the whole family.
    #[allow(clippy::cast_possible_truncation)]
    pub fn set_family_opacity(&mut self, mob: Mob, opacity: f64) {
        let opacity = opacity as f32;
        for member in self.family(mob) {
            let Some(entry) = self.get_mut(member) else {
                continue;
            };
            let fields: Vec<String> = entry
                .buffer
                .schema()
                .fields()
                .iter()
                .filter(|f| f.name.ends_with("rgba"))
                .map(|f| f.name.clone())
                .collect();
            for field in fields {
                if let Some(mut column) = entry.buffer.read_column(&field) {
                    for alpha in column.iter_mut().skip(3).step_by(4) {
                        *alpha = opacity;
                    }
                    entry.buffer.write_range(&field, 0, &column);
                }
            }
        }
    }

    /// The Reference's colour write on one 4-lane column (`fill_rgba`,
    /// `stroke_rgba`, or a base mobject's `rgba`) across `mob`, or its whole
    /// family when `recurse`: replace the rgb and/or alpha lanes, leaving
    /// the other untouched. Members whose schema lacks `field` are skipped.
    pub fn set_color_lanes(
        &mut self,
        mob: Mob,
        field: &str,
        rgb: Option<[f64; 3]>,
        alpha: Option<f64>,
        recurse: bool,
    ) {
        let members = if recurse { self.family(mob) } else { vec![mob] };
        for member in members {
            if let Some(entry) = self.get_mut(member) {
                entry.buffer.write_color_lanes(field, rgb, alpha);
            }
        }
    }

    /// One scalar written across a 1-lane column (`stroke_width`,
    /// `fill_border_width`) of `mob`, or of its whole family when `recurse`.
    pub fn set_scalar_lanes(&mut self, mob: Mob, field: &str, value: f64, recurse: bool) {
        let members = if recurse { self.family(mob) } else { vec![mob] };
        for member in members {
            if let Some(entry) = self.get_mut(member) {
                entry.buffer.write_scalar_lanes(field, value);
            }
        }
    }

    /// Reference `invisible_copy`'s `set_opacity(0)`. Public because the
    /// fade mechanism family uses the exact same record mutation.
    pub fn set_family_opacity_zero(&mut self, mob: Mob) {
        self.set_family_opacity(mob, 0.0);
    }

    /// Reference `align_data` (mobject.py:1746): zip the two families and
    /// align each pair's points. (Run [`Stage::align_family`] first —
    /// `align_data_and_family` does — so the zip covers both sides.)
    ///
    /// # Errors
    /// [`StageError::StaleHandle`], [`StageError::SchemaMismatch`] on a
    /// vmobject/base mixed pair, [`StageError::Geometry`] on a malformed
    /// point run.
    pub fn align_data(&mut self, a: Mob, b: Mob) -> Result<(), StageError> {
        let fa = self.family(a);
        let fb = self.family(b);
        for (&ma, &mb) in fa.iter().zip(fb.iter()) {
            self.align_points(ma, mb)?;
        }
        Ok(())
    }

    /// Reference `align_data_and_family` (mobject.py:1741): family first,
    /// then data — the order is semantics.
    ///
    /// # Errors
    /// As [`Stage::align_family`] and [`Stage::align_data`].
    pub fn align_data_and_family(&mut self, a: Mob, b: Mob) -> Result<(), StageError> {
        self.align_family(a, b)?;
        self.align_data(a, b)
    }

    /// Reference `align_points`: base records null-align by proportional
    /// resize (mobject.py:1751); vmobject pairs run the bezier-aware
    /// subpath algorithm (vectorized_mobject.py:964). A mixed pair is a
    /// typed refusal — the Reference would crash on one.
    ///
    /// # Errors
    /// [`StageError::StaleHandle`], [`StageError::SchemaMismatch`],
    /// [`StageError::Geometry`].
    pub fn align_points(&mut self, a: Mob, b: Mob) -> Result<(), StageError> {
        self.align_points_with_tolerance(a, b, DEFAULT_TOLERANCE_FOR_POINT_EQUALITY)
    }

    /// Reference `VMobject.align_points` under the receiver's live
    /// `tolerance_for_point_equality`. The ordinary native animation path
    /// uses [`Stage::align_points`] and its governed default; portals whose
    /// public object can override the tolerance use this exact seam rather
    /// than reimplementing subpath alignment outside Marionette.
    ///
    /// # Errors
    /// As [`Stage::align_points`].
    pub fn align_points_with_tolerance(
        &mut self,
        a: Mob,
        b: Mob,
        tolerance: f64,
    ) -> Result<(), StageError> {
        if matches!(
            self.try_get(a)?.render_primitive(),
            crate::RenderPrimitive::SurfaceGrid { .. }
        ) || matches!(
            self.try_get(b)?.render_primitive(),
            crate::RenderPrimitive::SurfaceGrid { .. }
        ) {
            return self.align_surface_points(a, b);
        }
        match (is_vmobject_schema(self, a)?, is_vmobject_schema(self, b)?) {
            (true, true) => self.align_points_vmobject(a, b, tolerance),
            (false, false) => {
                let la = self.try_get(a)?.buffer.len();
                let lb = self.try_get(b)?.buffer.len();
                let max_len = la.max(lb);
                self.get_mut(a)
                    .expect("checked live")
                    .buffer
                    .resize_preserving_order(max_len)
                    .map_err(StageError::Record)?;
                self.get_mut(b)
                    .expect("checked live")
                    .buffer
                    .resize_preserving_order(max_len)
                    .map_err(StageError::Record)?;
                Ok(())
            }
            _ => Err(StageError::SchemaMismatch),
        }
    }

    /// Reference `pointwise_become_partial` (vectorized_mobject.py:1050):
    /// make `mob`'s points the restriction of `source`'s to the proportion
    /// window `[a, b]` (by curve index), copying `source`'s joint angles
    /// and zeroing them over the collapsed flanks. The partial-reveal
    /// mechanism family (ShowCreation, Write, the passing flashes — §9.4)
    /// drives every frame through this one operation.
    ///
    /// An empty `source` is a no-op (nothing to restrict); a childless
    /// point run with no curves keeps `mob`'s points untouched, exactly as
    /// the Reference (its zeroed array is written to a discarded copy).
    ///
    /// # Errors
    /// [`StageError::StaleHandle`]; [`StageError::SchemaMismatch`] when
    /// either side of a non-empty pair is not vmobject-shaped (the
    /// Reference asserts `isinstance(vmobject, VMobject)`).
    pub fn pointwise_become_partial(
        &mut self,
        mob: Mob,
        source: Mob,
        a: f64,
        b: f64,
    ) -> Result<(), StageError> {
        let src_len = self.try_get(source)?.buffer.len();
        if src_len == 0 {
            // Nothing to restrict; the Reference would copy an empty
            // joint-angle column and leave the points loop degenerate.
            return Ok(());
        }
        if !is_vmobject_schema(self, mob)? || !is_vmobject_schema(self, source)? {
            return Err(StageError::SchemaMismatch);
        }
        let src_points = self
            .try_get(source)?
            .buffer
            .read_column("point")
            .ok_or(StageError::SchemaMismatch)?;
        let src_angles = self
            .try_get(source)?
            .buffer
            .read_column("joint_angle")
            .ok_or(StageError::SchemaMismatch)?;
        let source_placement = self.try_get(source)?.placement();
        {
            let entry = self.get_mut(mob).ok_or(StageError::StaleHandle)?;
            entry.set_placement(source_placement);
            if entry.buffer.len() != src_len {
                entry
                    .buffer
                    .resize_preserving_order(src_len)
                    .map_err(StageError::Record)?;
            }
            // `self.data["joint_angle"] = vmobject.data["joint_angle"]`
            // happens before the full-range short-circuit, so it lands in
            // both branches.
            entry.buffer.write_range("joint_angle", 0, &src_angles);
        }
        if a <= 0.0 && b >= 1.0 {
            let entry = self.get_mut(mob).ok_or(StageError::StaleHandle)?;
            entry.buffer.write_range("point", 0, &src_points);
            return Ok(());
        }
        let pts: Vec<Vec3> = src_points
            .as_chunks::<3>()
            .0
            .iter()
            .map(|c| [f64::from(c[0]), f64::from(c[1]), f64::from(c[2])])
            .collect();
        let Some((new_points, i1, i4)) = QuadPath::partial_points(&pts, a, b) else {
            return Ok(()); // no curves — the Reference's discarded-copy no-op
        };
        let entry = self.get_mut(mob).ok_or(StageError::StaleHandle)?;
        #[allow(clippy::cast_possible_truncation)]
        let flat: Vec<f32> = new_points
            .iter()
            .flat_map(|p| p.iter().map(|v| *v as f32))
            .collect();
        entry.buffer.write_range("point", 0, &flat);
        // joint_angle[:i1] = 0; joint_angle[i4:] = 0.
        let mut angles = src_angles;
        for angle in &mut angles[..i1.min(src_len)] {
            *angle = 0.0;
        }
        for angle in &mut angles[i4.min(src_len)..] {
            *angle = 0.0;
        }
        entry.buffer.write_range("joint_angle", 0, &angles);
        Ok(())
    }

    /// Reference `Surface.pointwise_become_partial` (surface.py:176):
    /// collapse the sampled UV grid outside `[a, b]` along `axis`, keeping
    /// the grid shape and every non-point record unchanged.
    ///
    /// Surface resolution is constructor data rather than a raster hint, so
    /// the caller supplies it explicitly. This keeps ordinary surfaces on
    /// [`crate::ShapeTag::General`] while giving Choreo one state-real
    /// partial-reveal operation shared by every front door.
    ///
    /// # Errors
    /// [`StageError::StaleHandle`]; [`StageError::SchemaMismatch`] when the
    /// records are not matching ordinary Surface schemas, the resolution
    /// does not describe the record count, or `axis` is not a revealable UV
    /// dimension.
    pub fn surface_pointwise_become_partial(
        &mut self,
        mob: Mob,
        source: Mob,
        resolution: (usize, usize),
        axis: usize,
        a: f64,
        b: f64,
    ) -> Result<(), StageError> {
        let source_entry = self.try_get(source)?;
        let source_schema = source_entry.buffer.schema().clone();
        let source_len = source_entry.buffer.len();
        let source_placement = source_entry.placement();
        let source_points = source_entry
            .buffer
            .read_column("point")
            .ok_or(StageError::SchemaMismatch)?;
        let source_normals = source_entry
            .buffer
            .read_column("d_normal_point")
            .ok_or(StageError::SchemaMismatch)?;

        let (nu, nv) = resolution;
        let axis_len = if axis == 0 { nu } else { nv };
        let fields = source_schema.fields();
        let ordinary_surface_schema = fields.len() == 3
            && fields[0].name == "point"
            && fields[0].width == 3
            && fields[1].name == "d_normal_point"
            && fields[1].width == 3
            && fields[2].name == "rgba"
            && fields[2].width == 4;
        if axis > 1
            || resolution.0.checked_mul(resolution.1) != Some(source_len)
            || (source_len != 0 && axis_len < 2)
            || !ordinary_surface_schema
        {
            return Err(StageError::SchemaMismatch);
        }
        let destination = self.try_get(mob)?;
        if destination.buffer.schema() != &source_schema {
            return Err(StageError::SchemaMismatch);
        }
        if source_len == 0 {
            return Ok(());
        }
        {
            let entry = self.get_mut(mob).ok_or(StageError::StaleHandle)?;
            entry.set_placement(source_placement);
            if entry.buffer.len() != source_len {
                entry
                    .buffer
                    .resize_preserving_order(source_len)
                    .map_err(StageError::Record)?;
            }
        }
        if a <= 0.0 && b >= 1.0 {
            let entry = self.get_mut(mob).ok_or(StageError::StaleHandle)?;
            entry.buffer.write_range("point", 0, &source_points);
            entry
                .buffer
                .write_range("d_normal_point", 0, &source_normals);
            return Ok(());
        }

        let mut points: Vec<Vec3> = source_points
            .as_chunks::<3>()
            .0
            .iter()
            .map(|row| [f64::from(row[0]), f64::from(row[1]), f64::from(row[2])])
            .collect();
        let max_index = i64::try_from(axis_len - 1).map_err(|_| StageError::SchemaMismatch)?;
        let (lower_index, lower_residue) = if max_index <= 0 {
            (0, 0.0)
        } else {
            bezier::integer_interpolate(0, max_index, a).map_err(|_| StageError::SchemaMismatch)?
        };
        let (upper_index, upper_residue) = if max_index <= 0 {
            (0, 0.0)
        } else {
            bezier::integer_interpolate(0, max_index, b).map_err(|_| StageError::SchemaMismatch)?
        };
        let lower_index = usize::try_from(lower_index).map_err(|_| StageError::SchemaMismatch)?;
        let upper_index = usize::try_from(upper_index).map_err(|_| StageError::SchemaMismatch)?;
        let lerp = |p: Vec3, q: Vec3, alpha: f64| {
            [
                (1.0 - alpha) * p[0] + alpha * q[0],
                (1.0 - alpha) * p[1] + alpha * q[1],
                (1.0 - alpha) * p[2] + alpha * q[2],
            ]
        };
        if axis == 0 {
            for v in 0..nv {
                let lower = lerp(
                    points[lower_index * nv + v],
                    points[(lower_index + 1) * nv + v],
                    lower_residue,
                );
                for u in 0..lower_index {
                    points[u * nv + v] = lower;
                }
                let upper = lerp(
                    points[upper_index * nv + v],
                    points[(upper_index + 1) * nv + v],
                    upper_residue,
                );
                for u in upper_index + 1..nu {
                    points[u * nv + v] = upper;
                }
            }
        } else {
            for u in 0..nu {
                let row = u * nv;
                let lower = lerp(
                    points[row + lower_index],
                    points[row + lower_index + 1],
                    lower_residue,
                );
                for v in 0..lower_index {
                    points[row + v] = lower;
                }
                let upper = lerp(
                    points[row + upper_index],
                    points[row + upper_index + 1],
                    upper_residue,
                );
                for v in upper_index + 1..nv {
                    points[row + v] = upper;
                }
            }
        }

        #[allow(clippy::cast_possible_truncation)]
        let flat: Vec<f32> = points
            .iter()
            .flat_map(|point| point.iter().map(|value| *value as f32))
            .collect();
        self.get_mut(mob)
            .ok_or(StageError::StaleHandle)?
            .buffer
            .write_range("point", 0, &flat);
        Ok(())
    }

    /// `point_from_proportion` under the original name, constant-speed by
    /// true arc length (BN-03): the point `alpha` of the way along `mob`'s
    /// path measured by arc length, via the W2 inverse-arclength layer.
    /// The Reference's `quick_point_from_proportion` (equal-curve-length
    /// approximation) is deliberately not the rule `MoveAlongPath` rides —
    /// that is the Behavior-Noted improvement.
    ///
    /// # Errors
    /// [`StageError::StaleHandle`]; [`StageError::SchemaMismatch`] on a
    /// pointless mobject; [`StageError::Geometry`] on a malformed run.
    pub fn point_from_proportion(&self, mob: Mob, alpha: f64) -> Result<Vec3, StageError> {
        let points = read_points(self, mob)?;
        if points.is_empty() {
            return Err(StageError::SchemaMismatch);
        }
        let path = QuadPath::from_points(points).map_err(StageError::Geometry)?;
        path.point_from_proportion(alpha)
            .ok_or(StageError::SchemaMismatch)
    }

    /// Reference `make_approximately_smooth` over the family: every
    /// vmobject member's handles are recomputed for smooth joins, point
    /// counts kept. The `SmoothedVectorizedHomotopy` hook
    /// (`apply_points_function(..., make_smooth=True)`).
    ///
    /// # Errors
    /// [`StageError::StaleHandle`], [`StageError::Geometry`].
    pub fn make_family_smooth(&mut self, mob: Mob) -> Result<(), StageError> {
        for member in self.family(mob) {
            if !is_vmobject_schema(self, member)? {
                continue;
            }
            let points = read_points(self, member)?;
            if points.len() < 3 {
                continue;
            }
            let mut path = QuadPath::from_points(points).map_err(StageError::Geometry)?;
            path.make_smooth(true).map_err(StageError::Geometry)?;
            let smoothed = path.points().to_vec();
            write_points(self, member, &smoothed)?;
        }
        Ok(())
    }

    /// Reference `VMobject.reverse_points(recurse)` over
    /// `Mobject.reverse_points`: re-mark subpath-break handles and invert odd
    /// `base_normal` rows for the selected VMobject family, then reverse
    /// **every** record row in the complete family wholesale. Colors, widths,
    /// and angles travel with their points exactly as `data[::-1]` does.
    ///
    /// The split is intentional. The Reference's VMobject pre-pass honors
    /// `recurse`, while its base-class reversal always traverses the complete
    /// family. Keeping both phases explicit preserves even the observable
    /// `recurse=false` edge rather than silently normalizing it.
    ///
    /// # Errors
    /// [`StageError::StaleHandle`], [`StageError::Geometry`].
    pub fn reverse_family_points(&mut self, mob: Mob) -> Result<(), StageError> {
        self.reverse_family_points_with_scope(mob, true)
    }

    /// The same family-row reversal with an explicit VMobject pre-repair
    /// scope, used by the Python `recurse` surface.
    pub fn reverse_family_points_with_scope(
        &mut self,
        mob: Mob,
        repair_recurse: bool,
    ) -> Result<(), StageError> {
        let family = self.family(mob);
        let repair_members = if repair_recurse {
            family.clone()
        } else {
            vec![mob]
        };
        for member in repair_members {
            let len = self.try_get(member)?.buffer.len();
            if len == 0 {
                continue;
            }
            if is_vmobject_schema(self, member)? {
                let mut points = self
                    .get_object_points(member)
                    .ok_or(StageError::StaleHandle)?;
                let path = QuadPath::from_points(points.clone()).map_err(StageError::Geometry)?;
                let end_indices = path.subpath_end_indices();
                for &e in &end_indices[..end_indices.len().saturating_sub(1)] {
                    if e + 2 < points.len() {
                        points[e + 1] = points[e + 2];
                    }
                }
                let entry = self.get_mut(member).ok_or(StageError::StaleHandle)?;
                #[allow(clippy::cast_possible_truncation)]
                let flat: Vec<f32> = points
                    .iter()
                    .flat_map(|p| p.iter().map(|v| *v as f32))
                    .collect();
                entry.buffer.write_range("point", 0, &flat);
                if let Some(mut normals) = entry.buffer.read_column("base_normal") {
                    for row in normals.as_chunks_mut::<3>().0.iter_mut().skip(1).step_by(2) {
                        for lane in row {
                            *lane = -*lane;
                        }
                    }
                    entry.buffer.write_range("base_normal", 0, &normals);
                }
            }
        }
        for member in family {
            let len = self.try_get(member)?.buffer.len();
            if len == 0 {
                continue;
            }
            // Mobject.reverse_points: data[::-1] for every family member.
            let entry = self.get_mut(member).ok_or(StageError::StaleHandle)?;
            let fields: Vec<(String, usize)> = entry
                .buffer
                .schema()
                .fields()
                .iter()
                .map(|f| (f.name.clone(), f.width))
                .collect();
            for (field, width) in fields {
                if let Some(column) = entry.buffer.read_column(&field) {
                    let reversed: Vec<f32> = column
                        .chunks_exact(width)
                        .rev()
                        .flatten()
                        .copied()
                        .collect();
                    entry.buffer.write_range(&field, 0, &reversed);
                }
            }
        }
        Ok(())
    }

    /// Reference `refresh_joint_angles` (vectorized_mobject.py:1159) made
    /// eager: recompute every vmobject family member's joint-angle column
    /// from its current points (the Reference marks lazily; our data plane
    /// keeps the column current — same observable state). Non-vmobject and
    /// empty members are skipped.
    ///
    /// # Errors
    /// [`StageError::StaleHandle`], [`StageError::Geometry`] on a
    /// malformed point run.
    pub fn refresh_family_joint_angles(&mut self, mob: Mob) -> Result<(), StageError> {
        for member in self.family(mob) {
            if !is_vmobject_schema(self, member)? {
                continue;
            }
            refresh_joint_angles(self, member)?;
        }
        Ok(())
    }

    /// vectorized_mobject.py:964, step for step.
    fn align_points_vmobject(&mut self, a: Mob, b: Mob, tolerance: f64) -> Result<(), StageError> {
        let mut pa = read_points(self, a)?;
        let mut pb = read_points(self, b)?;
        if pa.len() == pb.len() {
            // Equal counts need no point write. Joint angles are object-space
            // geometry, so refreshing them must not bake an independent
            // placement back into the point buffer (fm-7if).
            refresh_joint_angles(self, a)?;
            refresh_joint_angles(self, b)?;
            return Ok(());
        }
        // No points → one point at the center (start_new_path(get_center())).
        if pa.is_empty() {
            pa = vec![self.get_center(a)];
        }
        if pb.is_empty() {
            pb = vec![self.get_center(b)];
        }

        let path_a = QuadPath::from_points(pa).map_err(StageError::Geometry)?;
        let path_b = QuadPath::from_points(pb).map_err(StageError::Geometry)?;
        let mut subpaths1: Vec<Vec<Vec3>> = path_a
            .subpaths()
            .into_iter()
            .map(<[Vec3]>::to_vec)
            .collect();
        let mut subpaths2: Vec<Vec<Vec3>> = path_b
            .subpaths()
            .into_iter()
            .map(<[Vec3]>::to_vec)
            .collect();
        for subpaths in [&mut subpaths1, &mut subpaths2] {
            let mut keyed: Vec<(f64, Vec<Vec3>)> = subpaths
                .drain(..)
                .map(|sp| (polyline_length(&sp), sp))
                .collect();
            // Descending by length; stable, like Python's list.sort.
            keyed.sort_by(|x, y| y.0.partial_cmp(&x.0).unwrap_or(std::cmp::Ordering::Equal));
            *subpaths = keyed.into_iter().map(|(_, sp)| sp).collect();
        }
        let n_subpaths = subpaths1.len().max(subpaths2.len());

        // Missing subpaths fold the largest back on itself:
        // vstack([sp0[:-1], sp0[::-1]]) — a degenerate zero-area run.
        let get_nth = |list: &[Vec<Vec3>], n: usize| -> Vec<Vec3> {
            if n >= list.len() {
                let sp0 = &list[0];
                let mut folded = sp0[..sp0.len() - 1].to_vec();
                folded.extend(sp0.iter().rev().copied());
                folded
            } else {
                list[n].clone()
            }
        };

        let mut new_points1: Vec<Vec3> = Vec::new();
        let mut new_points2: Vec<Vec3> = Vec::new();
        for n in 0..n_subpaths {
            let sp1 = get_nth(&subpaths1, n);
            let sp2 = get_nth(&subpaths2, n);
            let diff1 = sp2.len().saturating_sub(sp1.len()) / 2;
            let diff2 = sp1.len().saturating_sub(sp2.len()) / 2;
            let sp1 = QuadPath::insert_n_curves_to_point_list(diff1, &sp1, tolerance)
                .map_err(StageError::Geometry)?;
            let sp2 = QuadPath::insert_n_curves_to_point_list(diff2, &sp2, tolerance)
                .map_err(StageError::Geometry)?;
            if n > 0 {
                // Intermediate anchor marking the subpath break.
                new_points1.push(*new_points1.last().expect("prior subpath emitted"));
                new_points2.push(*new_points2.last().expect("prior subpath emitted"));
            }
            new_points1.extend(sp1);
            new_points2.extend(sp2);
        }

        write_points(self, a, &new_points1)?;
        write_points(self, b, &new_points2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Mobject, RecordBuffer, RecordSchema};

    fn surface(stage: &mut Stage) -> Mob {
        let schema = RecordSchema::new(
            &[("point", 3), ("d_normal_point", 3), ("rgba", 4)],
            &["point"],
            &["point", "d_normal_point"],
        )
        .expect("surface schema");
        let mut buffer = RecordBuffer::new(schema, 9).expect("3x3 surface");
        let mut points = Vec::new();
        let mut normals = Vec::new();
        for u in [0.0_f32, 1.0, 2.0] {
            for v in [0.0_f32, 1.0, 2.0] {
                points.extend_from_slice(&[u, v, 0.0]);
                normals.extend_from_slice(&[u, v, 1.0]);
            }
        }
        buffer.write_range("point", 0, &points);
        buffer.write_range("d_normal_point", 0, &normals);
        buffer.write_range("rgba", 0, &[1.0; 36]);
        stage.add(Mobject::from_buffer(buffer))
    }

    #[test]
    fn polyline_length_sums_consecutive_gaps() {
        let pts = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 2.0, 0.0]];
        assert!((polyline_length(&pts) - 3.0).abs() < 1e-12);
    }

    #[test]
    fn aligned_submobject_budget_accepts_the_exact_boundary() {
        assert_eq!(MAX_ALIGNED_SUBMOBJECTS, 65_536);
        assert_eq!(
            checked_aligned_submobject_count(0, MAX_ALIGNED_SUBMOBJECTS),
            Ok(MAX_ALIGNED_SUBMOBJECTS)
        );
        assert_eq!(
            checked_aligned_submobject_count(MAX_ALIGNED_SUBMOBJECTS - 1, 1),
            Ok(MAX_ALIGNED_SUBMOBJECTS)
        );
    }

    #[test]
    fn surface_partial_reveal_collapses_uv_flanks_and_restores_pointlikes() {
        let mut stage = Stage::new();
        let source = surface(&mut stage);
        let destination = stage.copy_family(source).expect("surface copy");

        stage
            .surface_pointwise_become_partial(destination, source, (3, 3), 1, 0.5, 0.5)
            .expect("middle v slice");
        let points = stage.get_points(destination).expect("surface points");
        assert_eq!(
            points,
            vec![
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, 1.0, 0.0],
                [2.0, 1.0, 0.0],
                [2.0, 1.0, 0.0],
            ]
        );
        stage
            .get_mut(destination)
            .expect("destination")
            .buffer
            .write_range("d_normal_point", 0, &[0.0; 27]);
        stage
            .surface_pointwise_become_partial(destination, source, (3, 3), 1, 0.0, 1.0)
            .expect("full surface");
        assert_eq!(
            stage
                .get(destination)
                .expect("destination")
                .buffer
                .read_column("d_normal_point"),
            stage
                .get(source)
                .expect("source")
                .buffer
                .read_column("d_normal_point")
        );
    }

    #[test]
    fn surface_partial_reveal_refuses_false_grid_metadata() {
        let mut stage = Stage::new();
        let source = surface(&mut stage);
        let destination = stage.copy_family(source).expect("surface copy");
        assert_eq!(
            stage.surface_pointwise_become_partial(destination, source, (2, 4), 1, 0.0, 0.5,),
            Err(StageError::SchemaMismatch)
        );
    }
}
