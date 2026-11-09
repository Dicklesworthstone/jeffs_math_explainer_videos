//! `QuadPath`: the shared-anchor quadratic-Bézier path model (§1.2, §7.1).
//!
//! `points = [a0, h0, a1, h1, a2, …]` — length odd when nonempty, curve *i*
//! is `points[2i..2i+3]`, and a subpath break is a null curve whose handle
//! sits exactly on its anchor. This layout is **API surface**: curve counts,
//! partial reveals, alignment, and user code indexing `points` all depend on
//! it, so every mutator here preserves it formally and the exact fixtures in
//! `tests/` lock it down. Semantics are ported from `VMobject`
//! (`3b1b/manim` @ `6199a00d`), computed in f64 per §6.1.
//!
//! The proportion/length layer (`point_from_proportion`, `get_arc_length`) is
//! fm-xci — true arc length under the original names. Cubic inputs route
//! through the one error-bounded converter in [`crate::cubic`] (fm-6cf).

use crate::GeomError;
use crate::bezier;
use crate::cubic;
use crate::scalar;
use crate::smoothing;
use crate::space_ops;
use crate::vec;
use fmn_core::constants::{DEG, OUT, PI, TAU};
use fmn_core::types::Vec3;

/// The Reference's `VMobject.tolerance_for_point_equality`.
pub const DEFAULT_TOLERANCE_FOR_POINT_EQUALITY: f64 = 1e-8;

/// Maximum total quadratic count after one public subdivision or insertion
/// operation.
///
/// The shared-anchor representation needs `2 * curves + 1` points, so this
/// cap bounds the output to 131,073 [`Vec3`] values (about 3 MiB of point
/// payload) before any output allocation or mutation occurs.
pub const MAX_SUBDIVIDED_CURVES: usize = 65_536;

/// The Reference's subpath-end disambiguation tolerance (its own comment:
/// "TODO, this is too unsystematic" — kept exactly, because subpath
/// decomposition is API behavior).
const SUBPATH_END_ATOL: f64 = 1e-4;

/// The angle below which `add_arc_to` degrades to a line.
const ARC_ANGLE_THRESHOLD: f64 = 1e-3;

const NO_CURVE: usize = usize::MAX;

fn curve_priority(norm: f64, index: usize) -> f64 {
    if norm.is_nan() {
        // `np.argmax` starts with index zero. A NaN there therefore keeps
        // winning because every later `value > NaN` comparison is false;
        // later NaNs are ignored. Map those two cases to an ordinary total
        // priority so the tournament below preserves that exact behavior.
        if index == 0 {
            f64::INFINITY
        } else {
            f64::NEG_INFINITY
        }
    } else {
        norm
    }
}

fn preferred_curve(norms: &[f64], left: usize, right: usize) -> usize {
    if left == NO_CURVE {
        return right;
    }
    if right == NO_CURVE {
        return left;
    }
    if curve_priority(norms[right], right) > curve_priority(norms[left], left) {
        right
    } else {
        // First index wins ties, matching `np.argmax` and the old linear
        // scan.
        left
    }
}

fn curve_winner_tree(norms: &[f64]) -> Result<(Vec<usize>, usize), GeomError> {
    let leaf_count =
        norms
            .len()
            .checked_next_power_of_two()
            .ok_or(GeomError::SubdivisionBudgetExceeded {
                requested: usize::MAX,
                max: MAX_SUBDIVIDED_CURVES,
            })?;
    let tree_len = leaf_count
        .checked_mul(2)
        .ok_or(GeomError::SubdivisionBudgetExceeded {
            requested: usize::MAX,
            max: MAX_SUBDIVIDED_CURVES,
        })?;
    let mut tree = vec![NO_CURVE; tree_len];
    for index in 0..norms.len() {
        tree[leaf_count + index] = index;
    }
    for node in (1..leaf_count).rev() {
        tree[node] = preferred_curve(norms, tree[2 * node], tree[2 * node + 1]);
    }
    Ok((tree, leaf_count))
}

fn refresh_curve_winner(tree: &mut [usize], leaf_count: usize, index: usize, norms: &[f64]) {
    let mut node = leaf_count + index;
    while node > 1 {
        node /= 2;
        tree[node] = preferred_curve(norms, tree[2 * node], tree[2 * node + 1]);
    }
}

/// Anchor modes for [`QuadPath::change_anchor_mode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorMode {
    /// Handles at segment midpoints: straight polyline corners.
    Jagged,
    /// The local parabola-through-neighbors handle rule (no solver).
    ApproxSmooth,
    /// The spline solve: a smooth quadratic path through the anchors
    /// (may change the number of points).
    TrueSmooth,
}

/// A joined quadratic-Bézier path over the shared-anchor layout.
#[derive(Debug, Clone, PartialEq)]
pub struct QuadPath {
    points: Vec<Vec3>,
    tolerance_for_point_equality: f64,
    /// `VMobject.long_lines`: lines split into two quadratics instead of one.
    long_lines: bool,
}

impl Default for QuadPath {
    fn default() -> Self {
        Self::new()
    }
}

impl QuadPath {
    #[must_use]
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            tolerance_for_point_equality: DEFAULT_TOLERANCE_FOR_POINT_EQUALITY,
            long_lines: false,
        }
    }

    /// Build a path from raw shared-anchor points (length must be 0 or odd).
    pub fn from_points(points: Vec<Vec3>) -> Result<Self, GeomError> {
        let mut path = Self::new();
        path.set_points(points)?;
        Ok(path)
    }

    /// The arc primitive, under the one arc-density rule (BN-09): the
    /// unit-arc points scaled by `radius`, rotated by `start_angle`, and
    /// shifted to `arc_center`. `n_components` defaults to
    /// [`bezier::arc_n_components`].
    ///
    /// # Errors
    /// The arc-component contract's typed refusals (fm-4tb.1):
    /// [`GeomError::NonFiniteArcAngle`], [`GeomError::ZeroArcComponents`],
    /// [`GeomError::ArcComponentOverflow`], and
    /// [`GeomError::ArcComponentsAboveBudget`].
    pub fn try_arc(
        start_angle: f64,
        angle: f64,
        radius: f64,
        arc_center: Vec3,
        n_components: Option<usize>,
    ) -> Result<Self, GeomError> {
        let n = match n_components {
            Some(n) => n,
            None => bezier::arc_n_components(angle)?,
        };
        let rot = space_ops::rotation_about_z(start_angle);
        let points = bezier::quadratic_points_for_arc(angle, n)?
            .into_iter()
            .map(|p| {
                let rotated = [
                    rot[0][0] * p[0] + rot[0][1] * p[1],
                    rot[1][0] * p[0] + rot[1][1] * p[1],
                    p[2],
                ];
                vec::add(vec::scale(rotated, radius), arc_center)
            })
            .collect();
        Ok(Self {
            points,
            ..Self::new()
        })
    }

    // ------------------------------------------------------------- storage

    #[must_use]
    pub fn points(&self) -> &[Vec3] {
        &self.points
    }

    #[must_use]
    pub fn num_points(&self) -> usize {
        self.points.len()
    }

    #[must_use]
    pub fn has_points(&self) -> bool {
        !self.points.is_empty()
    }

    #[must_use]
    pub fn tolerance_for_point_equality(&self) -> f64 {
        self.tolerance_for_point_equality
    }

    pub fn set_tolerance_for_point_equality(&mut self, tolerance: f64) -> &mut Self {
        self.tolerance_for_point_equality = tolerance;
        self
    }

    pub fn set_long_lines(&mut self, long_lines: bool) -> &mut Self {
        self.long_lines = long_lines;
        self
    }

    /// Accept the Reference's quality-lowering compatibility knob.
    ///
    /// FrankenManim always keeps the configured error bound, so this setting
    /// cannot switch to the Reference's unbounded shallow-cubic shortcut
    /// (BN-13). The one converter emits one quadratic for effectively
    /// quadratic input when it can preserve both the guarantee and
    /// shared-anchor encoding.
    pub fn set_use_simple_quadratic_approx(&mut self, _value: bool) -> &mut Self {
        self
    }

    /// Replace the points wholesale. The shared-anchor invariant is checked:
    /// the length must be 0 or odd.
    pub fn set_points(&mut self, points: Vec<Vec3>) -> Result<&mut Self, GeomError> {
        if !points.is_empty() && points.len().is_multiple_of(2) {
            return Err(GeomError::EvenPointCount { len: points.len() });
        }
        self.points = points;
        Ok(self)
    }

    /// Append points. Preserving oddness means appending an even count.
    pub fn append_points(&mut self, points: &[Vec3]) -> Result<&mut Self, GeomError> {
        if !points.len().is_multiple_of(2) {
            return Err(GeomError::EvenPointCount {
                len: self.points.len() + points.len(),
            });
        }
        if self.points.is_empty() && !points.is_empty() {
            return Err(GeomError::EmptyPath);
        }
        self.points.extend_from_slice(points);
        Ok(self)
    }

    pub fn clear_points(&mut self) -> &mut Self {
        self.points.clear();
        self
    }

    /// `set_anchors_and_handles`: interleave `anchors` and `handles`
    /// (`anchors.len() == handles.len() + 1`; empty anchors clears).
    pub fn set_anchors_and_handles(
        &mut self,
        anchors: &[Vec3],
        handles: &[Vec3],
    ) -> Result<&mut Self, GeomError> {
        if anchors.is_empty() {
            self.clear_points();
            return Ok(self);
        }
        if anchors.len() != handles.len() + 1 {
            return Err(GeomError::MismatchedAnchorsAndHandles {
                anchors: anchors.len(),
                handles: handles.len(),
            });
        }
        let mut points = Vec::with_capacity(2 * anchors.len() - 1);
        for i in 0..handles.len() {
            points.push(anchors[i]);
            points.push(handles[i]);
        }
        points.push(anchors[anchors.len() - 1]);
        self.points = points;
        Ok(self)
    }

    // ------------------------------------------------------- path building

    /// `start_new_path`: path ends are signaled by a handle sitting directly
    /// on top of the previous anchor.
    pub fn start_new_path(&mut self, point: Vec3) -> &mut Self {
        if let Some(&last) = self.points.last() {
            self.points.push(last);
            self.points.push(point);
        } else {
            self.points.push(point);
        }
        self
    }

    /// `add_line_to`. With `allow_null_line = false` a segment to a
    /// coincident point is silently skipped.
    pub fn add_line_to(
        &mut self,
        point: Vec3,
        allow_null_line: bool,
    ) -> Result<&mut Self, GeomError> {
        let last = self.last_point().ok_or(GeomError::EmptyPath)?;
        if !allow_null_line && self.consider_points_equal(last, point) {
            return Ok(self);
        }
        let n_alphas = if self.long_lines { 5 } else { 3 };
        let alphas = bezier::linspace(0.0, 1.0, n_alphas);
        for &alpha in &alphas[1..] {
            self.points.push(vec::lerp(last, point, alpha));
        }
        Ok(self)
    }

    /// `add_quadratic_bezier_curve_to`. A handle coincident with the last
    /// anchor would read as a subpath break, so it is nudged to the segment
    /// midpoint, exactly as the Reference does.
    pub fn add_quadratic_bezier_curve_to(
        &mut self,
        handle: Vec3,
        anchor: Vec3,
        allow_null_curve: bool,
    ) -> Result<&mut Self, GeomError> {
        let last = self.last_point().ok_or(GeomError::EmptyPath)?;
        if !allow_null_curve && self.consider_points_equal(last, anchor) {
            return Ok(self);
        }
        let handle = if self.consider_points_equal(handle, last) {
            space_ops::midpoint(handle, anchor)
        } else {
            handle
        };
        self.points.push(handle);
        self.points.push(anchor);
        Ok(self)
    }

    /// `add_cubic_bezier_curve_to`: reduce the cubic at the G0-2 canonical
    /// 1080p tolerance and append it.
    pub fn add_cubic_bezier_curve_to(
        &mut self,
        handle1: Vec3,
        handle2: Vec3,
        anchor: Vec3,
    ) -> Result<&mut Self, GeomError> {
        self.add_cubic_bezier_curve_to_with_tolerance(
            handle1,
            handle2,
            anchor,
            cubic::DEFAULT_TOLERANCE_SCENE,
        )
    }

    /// Reduce and append a cubic under an explicit scene-unit `tolerance`.
    ///
    /// This is the configuration seam for importers and non-default render
    /// scales. The emitted point run is the exact chain whose Bernstein bound
    /// was admitted by [`cubic::cubic_to_quadratics`].
    pub fn add_cubic_bezier_curve_to_with_tolerance(
        &mut self,
        handle1: Vec3,
        handle2: Vec3,
        anchor: Vec3,
        tolerance: f64,
    ) -> Result<&mut Self, GeomError> {
        let last = self.last_point().ok_or(GeomError::EmptyPath)?;
        let quad_approx = cubic::cubic_to_quadratics(last, handle1, handle2, anchor, tolerance)?;
        self.points.extend_from_slice(&quad_approx[1..]);
        Ok(self)
    }

    /// `add_smooth_curve_to`: continue with the reflection of the last
    /// handle, or a plain line when a new subpath was just started.
    pub fn add_smooth_curve_to(&mut self, point: Vec3) -> Result<&mut Self, GeomError> {
        if self.has_new_path_started() {
            self.add_line_to(point, true)
        } else {
            let handle = self
                .reflection_of_last_handle()
                .ok_or(GeomError::EmptyPath)?;
            self.add_quadratic_bezier_curve_to(handle, point, true)
        }
    }

    /// `add_smooth_cubic_curve_to`.
    pub fn add_smooth_cubic_curve_to(
        &mut self,
        handle: Vec3,
        point: Vec3,
    ) -> Result<&mut Self, GeomError> {
        if self.points.is_empty() {
            return Err(GeomError::EmptyPath);
        }
        let new_handle = if self.num_points() == 1 {
            handle
        } else {
            // Reflection is well-defined here: at least two points exist.
            let n = self.points.len();
            vec::sub(vec::scale(self.points[n - 1], 2.0), self.points[n - 2])
        };
        self.add_cubic_bezier_curve_to(new_handle, handle, point)
    }

    /// `add_arc_to`: an arc subtending `angle` from the current end to
    /// `point`. `n_components` defaults to the one arc-density rule (BN-09);
    /// the Reference's `ceil(8·|θ|/TAU)` here was one of the three retired
    /// conventions.
    ///
    /// # Errors
    /// [`GeomError::EmptyPath`] without a current endpoint, or the typed
    /// arc-component refusal for the requested angle and count.
    pub fn add_arc_to(
        &mut self,
        point: Vec3,
        angle: f64,
        n_components: Option<usize>,
    ) -> Result<&mut Self, GeomError> {
        self.add_arc_to_with_threshold(point, angle, n_components, ARC_ANGLE_THRESHOLD)
    }

    /// `add_arc_to` with the caller's public straight-line cutoff.
    ///
    /// The Python surface exposes this as its `threshold` argument. Arc
    /// density remains governed by BN-09; the threshold changes only whether
    /// a sufficiently small angle takes the ordinary line path.
    pub fn add_arc_to_with_threshold(
        &mut self,
        point: Vec3,
        angle: f64,
        n_components: Option<usize>,
        threshold: f64,
    ) -> Result<&mut Self, GeomError> {
        let last = self.last_point().ok_or(GeomError::EmptyPath)?;
        match n_components {
            Some(n) => {
                let _point_count = bezier::arc_point_count(angle, n)?;
            }
            None if !angle.is_finite() => return Err(GeomError::NonFiniteArcAngle),
            None => {}
        }
        if angle.abs() < threshold {
            return self.add_line_to(point, true);
        }
        let n = match n_components {
            Some(n) => n,
            None => bezier::arc_n_components(angle)?,
        };
        let mut arc_points = bezier::quadratic_points_for_arc(angle, n)?;
        let target_vect = vec::sub(point, last);
        let curr_vect = vec::sub(arc_points[arc_points.len() - 1], arc_points[0]);
        let rot = space_ops::rotation_between_vectors(curr_vect, target_vect);
        let scale_factor = space_ops::get_norm(target_vect) / space_ops::get_norm(curr_vect);
        for p in arc_points.iter_mut() {
            // Reference: `arc_points @ R.T` = R · p per point.
            *p = vec::scale(vec::mul_point_mat(*p, &vec::transpose(&rot)), scale_factor);
        }
        let offset = vec::sub(last, arc_points[0]);
        for p in arc_points.iter_mut() {
            *p = vec::add(*p, offset);
        }
        self.points.extend_from_slice(&arc_points[1..]);
        Ok(self)
    }

    /// `add_points_as_corners`.
    pub fn add_points_as_corners(&mut self, points: &[Vec3]) -> Result<&mut Self, GeomError> {
        for &p in points {
            self.add_line_to(p, true)?;
        }
        Ok(self)
    }

    /// `set_points_as_corners`: anchors with midpoint handles.
    pub fn set_points_as_corners(&mut self, points: &[Vec3]) -> Result<&mut Self, GeomError> {
        let handles: Vec<Vec3> = points
            .windows(2)
            .map(|w| space_ops::midpoint(w[0], w[1]))
            .collect();
        self.set_anchors_and_handles(points, &handles)
    }

    /// `set_points_smoothly`.
    pub fn set_points_smoothly(
        &mut self,
        points: &[Vec3],
        approx: bool,
    ) -> Result<&mut Self, GeomError> {
        self.set_points_as_corners(points)?;
        self.make_smooth(approx)?;
        Ok(self)
    }

    /// `add_subpath`: splice a shared-anchor point run in as a new subpath
    /// (or a continuation when it starts at the current end).
    pub fn add_subpath(&mut self, points: &[Vec3]) -> Result<&mut Self, GeomError> {
        if points.len().is_multiple_of(2) && !points.is_empty() {
            return Err(GeomError::EvenPointCount { len: points.len() });
        }
        if points.is_empty() {
            return Ok(self);
        }
        if !self.has_points() {
            self.points = points.to_vec();
            return Ok(self);
        }
        if !self.consider_points_equal(points[0], self.points[self.points.len() - 1]) {
            self.start_new_path(points[0]);
        }
        self.points.extend_from_slice(&points[1..]);
        Ok(self)
    }

    // ------------------------------------------------------------- queries

    /// `consider_points_equal`: strict per-component comparison against the
    /// path's tolerance.
    #[must_use]
    pub fn consider_points_equal(&self, p0: Vec3, p1: Vec3) -> bool {
        (0..3).all(|i| (p1[i] - p0[i]).abs() < self.tolerance_for_point_equality)
    }

    /// `has_new_path_started`: a one-point path, or a trailing break marker.
    #[must_use]
    pub fn has_new_path_started(&self) -> bool {
        match self.points.len() {
            0 => false,
            1 => true,
            n => self.consider_points_equal(self.points[n - 3], self.points[n - 2]),
        }
    }

    #[must_use]
    pub fn last_point(&self) -> Option<Vec3> {
        self.points.last().copied()
    }

    /// `get_reflection_of_last_handle`: `2·points[-1] − points[-2]`.
    #[must_use]
    pub fn reflection_of_last_handle(&self) -> Option<Vec3> {
        let n = self.points.len();
        if n < 2 {
            return None;
        }
        Some(vec::sub(
            vec::scale(self.points[n - 1], 2.0),
            self.points[n - 2],
        ))
    }

    /// `close_path`: join the end of the last subpath back to its start.
    pub fn close_path(&mut self, smooth: bool) -> Result<&mut Self, GeomError> {
        if self.is_closed() {
            return Ok(self);
        }
        let start = self.last_subpath_start().ok_or(GeomError::EmptyPath)?;
        if smooth {
            self.add_smooth_curve_to(start)
        } else {
            self.add_line_to(start, true)
        }
    }

    fn last_subpath_start(&self) -> Option<Vec3> {
        if self.points.is_empty() {
            return None;
        }
        let ends = self.subpath_end_indices();
        let index = if ends.len() == 1 {
            0
        } else {
            ends[ends.len() - 2] + 2
        };
        Some(self.points[index])
    }

    /// `is_closed`: whether the last subpath's end returns to its start
    /// (within the point-equality tolerance). An empty path is not closed.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        match self.last_subpath_start() {
            Some(start) => self.consider_points_equal(start, self.points[self.points.len() - 1]),
            None => false,
        }
    }

    #[must_use]
    pub fn num_curves(&self) -> usize {
        self.points.len() / 2
    }

    /// Curve *i* = `points[2i..2i+3]`.
    #[must_use]
    pub fn nth_curve_points(&self, n: usize) -> Option<[Vec3; 3]> {
        if n >= self.num_curves() {
            return None;
        }
        Some([
            self.points[2 * n],
            self.points[2 * n + 1],
            self.points[2 * n + 2],
        ])
    }

    /// Evaluate curve `n` at parameter `t`.
    #[must_use]
    pub fn nth_curve_point(&self, n: usize, t: f64) -> Option<Vec3> {
        self.nth_curve_points(n)
            .map(|[p0, p1, p2]| bezier::quadratic_point(p0, p1, p2, t))
    }

    /// Iterator over the curve triples.
    pub fn bezier_tuples(&self) -> impl Iterator<Item = [Vec3; 3]> + '_ {
        (0..self.num_curves()).map(|i| {
            [
                self.points[2 * i],
                self.points[2 * i + 1],
                self.points[2 * i + 2],
            ]
        })
    }

    #[must_use]
    pub fn anchors(&self) -> Vec<Vec3> {
        self.points.iter().step_by(2).copied().collect()
    }

    #[must_use]
    pub fn start_anchors(&self) -> Vec<Vec3> {
        if self.points.is_empty() {
            return Vec::new();
        }
        self.points[..self.points.len() - 1]
            .iter()
            .step_by(2)
            .copied()
            .collect()
    }

    #[must_use]
    pub fn end_anchors(&self) -> Vec<Vec3> {
        if self.points.len() < 3 {
            return Vec::new();
        }
        self.points[2..].iter().step_by(2).copied().collect()
    }

    /// `get_subpath_end_indices`: anchor indices ending each subpath. An
    /// anchor ends a subpath when its following handle sits exactly on top
    /// of it *and* the following anchor is genuinely distinct (beyond the
    /// Reference's fixed 1e-4), disambiguating breaks from runs of null
    /// curves. The final point index is always an end.
    #[must_use]
    pub fn subpath_end_indices(&self) -> Vec<usize> {
        if self.points.is_empty() {
            return Vec::new();
        }
        let mut ends = Vec::new();
        for i in 0..self.num_curves() {
            let a0 = self.points[2 * i];
            let h = self.points[2 * i + 1];
            let a1 = self.points[2 * i + 2];
            let is_break = a0 == h && (0..3).any(|k| (h[k] - a1[k]).abs() > SUBPATH_END_ATOL);
            if is_break {
                ends.push(2 * i);
            }
        }
        ends.push(self.points.len() - 1);
        ends
    }

    /// The subpaths as point slices (each in shared-anchor layout).
    #[must_use]
    pub fn subpaths(&self) -> Vec<&[Vec3]> {
        if self.points.is_empty() {
            return Vec::new();
        }
        let ends = self.subpath_end_indices();
        let mut out = Vec::with_capacity(ends.len());
        let mut start = 0usize;
        for &end in &ends {
            out.push(&self.points[start..=end]);
            start = end + 2;
        }
        out
    }

    /// `get_points_without_null_curves`.
    #[must_use]
    pub fn points_without_null_curves(&self, atol: f64) -> Vec<Vec3> {
        if self.points.is_empty() {
            return Vec::new();
        }
        let mut out = vec![self.points[0]];
        for tup in self.bezier_tuples() {
            if space_ops::get_norm(vec::sub(tup[1], tup[0])) > atol
                || space_ops::get_norm(vec::sub(tup[2], tup[0])) > atol
            {
                out.push(tup[1]);
                out.push(tup[2]);
            }
        }
        out
    }

    // -------------------------------------------------------- subdivision

    /// `subdivide_curves_by_condition`: split each curve into `f(curve) + 1`
    /// equal-parameter pieces.
    ///
    /// The callback is evaluated exactly once per input curve during a
    /// preflight. Per-curve arithmetic, the aggregate piece count, and the
    /// final shared-anchor point count must all fit [`MAX_SUBDIVIDED_CURVES`]
    /// before output allocation or path mutation.
    pub fn subdivide_curves_by_condition(
        &mut self,
        tuple_to_subdivisions: impl Fn([Vec3; 3]) -> usize,
    ) -> Result<&mut Self, GeomError> {
        if !self.has_points() {
            return Ok(self);
        }
        let tuples: Vec<[Vec3; 3]> = self.bezier_tuples().collect();
        let mut pieces_per_curve = Vec::with_capacity(tuples.len());
        let mut total_curves = 0usize;
        for &tup in &tuples {
            let subdivisions = tuple_to_subdivisions(tup);
            let pieces =
                subdivisions
                    .checked_add(1)
                    .ok_or(GeomError::SubdivisionBudgetExceeded {
                        requested: usize::MAX,
                        max: MAX_SUBDIVIDED_CURVES,
                    })?;
            total_curves =
                total_curves
                    .checked_add(pieces)
                    .ok_or(GeomError::SubdivisionBudgetExceeded {
                        requested: usize::MAX,
                        max: MAX_SUBDIVIDED_CURVES,
                    })?;
            if total_curves > MAX_SUBDIVIDED_CURVES {
                return Err(GeomError::SubdivisionBudgetExceeded {
                    requested: total_curves,
                    max: MAX_SUBDIVIDED_CURVES,
                });
            }
            pieces_per_curve.push(pieces);
        }

        let point_count = total_curves
            .checked_mul(2)
            .and_then(|count| count.checked_add(1))
            .ok_or(GeomError::SubdivisionBudgetExceeded {
                requested: usize::MAX,
                max: MAX_SUBDIVIDED_CURVES,
            })?;
        let mut new_points = Vec::with_capacity(point_count);
        new_points.push(self.points[0]);
        for (tup, pieces) in tuples.into_iter().zip(pieces_per_curve) {
            if pieces > 1 {
                let alphas = bezier::linspace(0.0, 1.0, pieces + 1);
                for pair in alphas.windows(2) {
                    let sub = bezier::partial_quadratic(&tup, pair[0], pair[1]);
                    new_points.push(sub[1]);
                    new_points.push(sub[2]);
                }
            } else {
                new_points.push(tup[1]);
                new_points.push(tup[2]);
            }
        }
        debug_assert_eq!(new_points.len(), point_count);
        self.points = new_points;
        Ok(self)
    }

    /// `subdivide_sharp_curves`, with a positive finite threshold.
    pub fn subdivide_sharp_curves(&mut self, angle_threshold: f64) -> Result<&mut Self, GeomError> {
        if !angle_threshold.is_finite() || angle_threshold <= 0.0 {
            return Err(GeomError::InvalidSubdivisionThreshold);
        }
        self.subdivide_curves_by_condition(|[b0, b1, b2]| {
            let angle = space_ops::angle_between_vectors(vec::sub(b1, b0), vec::sub(b2, b1));
            (angle / angle_threshold) as usize
        })
    }

    /// `insert_n_curves_to_point_list`: distribute `n` extra curves over the
    /// longest curves (null curves never split), preserving the traced shape.
    ///
    /// The shared-anchor input is validated and the final curve and point
    /// counts are checked against [`MAX_SUBDIVIDED_CURVES`] before selection
    /// work or output allocation. The winner tournament preserves the
    /// Reference's first-index tie rule while reducing selection from
    /// `O(n * input_curves)` to `O(input_curves + n log input_curves)`.
    ///
    /// # Errors
    ///
    /// Returns [`GeomError::EvenPointCount`] for a malformed shared-anchor
    /// run or [`GeomError::SubdivisionBudgetExceeded`] when count arithmetic
    /// overflows or the requested output exceeds the public curve budget.
    pub fn insert_n_curves_to_point_list(
        n: usize,
        points: &[Vec3],
        tolerance: f64,
    ) -> Result<Vec<Vec3>, GeomError> {
        if points.is_empty() {
            return Ok(Vec::new());
        }
        if points.len().is_multiple_of(2) {
            return Err(GeomError::EvenPointCount { len: points.len() });
        }

        let existing_curves = (points.len() - 1) / 2;
        let output_curves =
            existing_curves
                .checked_add(n)
                .ok_or(GeomError::SubdivisionBudgetExceeded {
                    requested: usize::MAX,
                    max: MAX_SUBDIVIDED_CURVES,
                })?;
        if output_curves > MAX_SUBDIVIDED_CURVES {
            return Err(GeomError::SubdivisionBudgetExceeded {
                requested: output_curves,
                max: MAX_SUBDIVIDED_CURVES,
            });
        }
        let point_count = output_curves
            .checked_mul(2)
            .and_then(|count| count.checked_add(1))
            .ok_or(GeomError::SubdivisionBudgetExceeded {
                requested: usize::MAX,
                max: MAX_SUBDIVIDED_CURVES,
            })?;

        if points.len() == 1 {
            return Ok(vec![points[0]; point_count]);
        }
        if n == 0 {
            return Ok(points.to_vec());
        }
        let tuples: Vec<[Vec3; 3]> = (0..points.len().saturating_sub(1) / 2)
            .map(|i| [points[2 * i], points[2 * i + 1], points[2 * i + 2]])
            .collect();
        let mut norms: Vec<f64> = tuples
            .iter()
            .map(|tup| {
                if space_ops::get_norm(vec::sub(tup[1], tup[0])) < tolerance {
                    0.0
                } else {
                    space_ops::get_norm(vec::sub(tup[2], tup[0]))
                }
            })
            .collect();
        let mut ipc = vec![0usize; tuples.len()];
        let (mut winners, winner_leaves) = curve_winner_tree(&norms)?;
        for _ in 0..n {
            let index = winners[1];
            let next_count =
                ipc[index]
                    .checked_add(1)
                    .ok_or(GeomError::SubdivisionBudgetExceeded {
                        requested: usize::MAX,
                        max: MAX_SUBDIVIDED_CURVES,
                    })?;
            let divisor =
                next_count
                    .checked_add(1)
                    .ok_or(GeomError::SubdivisionBudgetExceeded {
                        requested: usize::MAX,
                        max: MAX_SUBDIVIDED_CURVES,
                    })?;
            ipc[index] = next_count;
            norms[index] *= next_count as f64 / divisor as f64;
            refresh_curve_winner(&mut winners, winner_leaves, index, &norms);
        }
        let mut new_points = Vec::with_capacity(point_count);
        new_points.push(points[0]);
        for (tup, &n_inserts) in tuples.iter().zip(ipc.iter()) {
            let alphas = bezier::linspace(0.0, 1.0, n_inserts + 2);
            for pair in alphas.windows(2) {
                let sub = bezier::partial_quadratic(tup, pair[0], pair[1]);
                new_points.push(sub[1]);
                new_points.push(sub[2]);
            }
        }
        Ok(new_points)
    }

    /// `integer_interpolate` (utils/bezier.py): the integer sample index
    /// along `[start, end)` at `alpha`, plus the fractional residue toward
    /// the next index. `alpha >= 1` pins to `(end - 1, 1.0)`; `alpha <= 0`
    /// to `(start, 0.0)` — the Reference's exact clamping.
    ///
    /// # Errors
    /// Delegates the authoritative range and finite-alpha validation to
    /// [`bezier::integer_interpolate`].
    pub fn integer_interpolate(
        start: i64,
        end: i64,
        alpha: f64,
    ) -> Result<(i64, f64), bezier::IntegerInterpolationError> {
        bezier::integer_interpolate(start, end, alpha)
    }

    /// The pure-geometry core of `pointwise_become_partial`
    /// (vectorized_mobject.py:1050): restrict a shared-anchor run to the
    /// proportion window `[a, b]` **by curve index** (uniform over curves,
    /// not arc length — that is the Reference's rule; true-proportion
    /// reveals compose this with the arclength layer's alpha remap).
    ///
    /// The partial run keeps the original point count: points before the
    /// window collapse onto the window's first point, points after onto its
    /// last, and the boundary curves are [`bezier::partial_quadratic`]
    /// restrictions. Returns `(new_points, i1, i4)` where `[..i1]` and
    /// `[i4..]` are the collapsed flanks (the Reference zeroes
    /// `joint_angle` there); `None` when the run has no curves (the
    /// Reference mutates a discarded copy — a deliberate no-op).
    #[must_use]
    pub fn partial_points(points: &[Vec3], a: f64, b: f64) -> Option<(Vec<Vec3>, usize, usize)> {
        // get_num_curves() is get_num_points() // 2 — identical to
        // (len − 1) / 2 on every valid shared-anchor run (odd length);
        // the latter also stays in-bounds on a malformed even run, where
        // the Reference would raise IndexError.
        let num_curves = points.len().saturating_sub(1) / 2;
        if num_curves == 0 {
            return None;
        }
        let end = i64::try_from(num_curves).ok()?;
        let (lower_index, lower_residue) = Self::integer_interpolate(0, end, a).ok()?;
        let (upper_index, upper_residue) = Self::integer_interpolate(0, end, b).ok()?;
        #[allow(clippy::cast_sign_loss)]
        let (lower_index, upper_index) = (lower_index as usize, upper_index as usize);
        let i1 = 2 * lower_index;
        let i2 = 2 * lower_index + 3;
        let i3 = 2 * upper_index;
        let i4 = 2 * upper_index + 3;

        let mut new_points = points.to_vec();
        if lower_index == upper_index {
            let tup = bezier::partial_quadratic(
                &[points[i1], points[i1 + 1], points[i1 + 2]],
                lower_residue,
                upper_residue,
            );
            for p in &mut new_points[..i1] {
                *p = tup[0];
            }
            new_points[i1..i4].copy_from_slice(&tup);
            for p in &mut new_points[i4..] {
                *p = tup[2];
            }
        } else {
            let low_tup = bezier::partial_quadratic(
                &[points[i1], points[i1 + 1], points[i1 + 2]],
                lower_residue,
                1.0,
            );
            let high_tup = bezier::partial_quadratic(
                &[points[i3], points[i3 + 1], points[i3 + 2]],
                0.0,
                upper_residue,
            );
            for p in &mut new_points[..i1] {
                *p = low_tup[0];
            }
            new_points[i1..i2].copy_from_slice(&low_tup);
            // i2..i3 keeps the middle section exactly.
            new_points[i3..i4].copy_from_slice(&high_tup);
            for p in &mut new_points[i4..] {
                *p = high_tup[2];
            }
        }
        Some((new_points, i1, i4))
    }

    /// `insert_n_curves`, with checked output sizing and failure atomicity.
    pub fn insert_n_curves(&mut self, n: usize) -> Result<&mut Self, GeomError> {
        if self.num_curves() > 0 {
            let points = Self::insert_n_curves_to_point_list(
                n,
                &self.points,
                self.tolerance_for_point_equality,
            )?;
            self.points = points;
        }
        Ok(self)
    }

    // ------------------------------------------------------- anchor modes

    /// `is_smooth`: all anchor-joint angles under `angle_tol`
    /// (the Reference's default is 1°).
    #[must_use]
    pub fn is_smooth(&self, angle_tol: f64) -> bool {
        self.joint_angles()
            .iter()
            .step_by(2)
            .all(|a| a.abs() < angle_tol)
    }

    /// `change_anchor_mode`: recompute every subpath's handles under the
    /// given mode, preserving the break markers, with the Reference's exact
    /// post-fixes for handles that land on an anchor.
    pub fn change_anchor_mode(&mut self, mode: AnchorMode) -> Result<&mut Self, GeomError> {
        if self.points.is_empty() {
            return Ok(self);
        }
        let subpaths: Vec<Vec<Vec3>> = self.subpaths().into_iter().map(|s| s.to_vec()).collect();
        // Build transactionally: true smoothing can refuse a tolerance or
        // resource budget, and that refusal must not erase the live path.
        let mut rebuilt = Self {
            points: Vec::with_capacity(self.points.len()),
            tolerance_for_point_equality: self.tolerance_for_point_equality,
            long_lines: self.long_lines,
        };
        for subpath in subpaths {
            let anchors: Vec<Vec3> = subpath.iter().step_by(2).copied().collect();
            let mut new_subpath = subpath.clone();
            match mode {
                AnchorMode::Jagged => {
                    for (slot, pair) in new_subpath
                        .iter_mut()
                        .skip(1)
                        .step_by(2)
                        .zip(anchors.windows(2))
                    {
                        *slot = space_ops::midpoint(pair[0], pair[1]);
                    }
                }
                AnchorMode::ApproxSmooth => {
                    let handles = smoothing::approx_smooth_quadratic_handles(&anchors);
                    for (slot, handle) in new_subpath.iter_mut().skip(1).step_by(2).zip(handles) {
                        *slot = handle;
                    }
                }
                AnchorMode::TrueSmooth => {
                    new_subpath =
                        smoothing::smooth_quadratic_path(&anchors, cubic::DEFAULT_TOLERANCE_SCENE)?;
                }
            }
            // Shift any handle that ended up exactly on top of the previous
            // anchor (a false break marker), then any that landed on the
            // following anchor (a degenerate handle).
            for i in (1..new_subpath.len()).step_by(2) {
                if new_subpath[i - 1] == new_subpath[i] {
                    new_subpath[i] = space_ops::midpoint(new_subpath[i - 1], new_subpath[i + 1]);
                }
                if new_subpath[i] == new_subpath[i + 1] {
                    new_subpath[i] = space_ops::midpoint(new_subpath[i - 1], new_subpath[i + 1]);
                }
            }
            rebuilt.add_subpath(&new_subpath)?;
        }
        self.points = rebuilt.points;
        Ok(self)
    }

    /// `make_smooth`: `approx = true` keeps the point count (approx handles),
    /// `false` runs the true spline solve. Already-smooth paths are left
    /// untouched, matching the Reference.
    pub fn make_smooth(&mut self, approx: bool) -> Result<&mut Self, GeomError> {
        if self.is_smooth(1.0 * DEG) {
            return Ok(self);
        }
        let mode = if approx {
            AnchorMode::ApproxSmooth
        } else {
            AnchorMode::TrueSmooth
        };
        self.change_anchor_mode(mode)
    }

    /// `make_jagged`.
    pub fn make_jagged(&mut self) -> Result<&mut Self, GeomError> {
        self.change_anchor_mode(AnchorMode::Jagged)
    }

    // ------------------------------------------------ normals and joints

    /// `get_area_vector`: the right-hand-rule area vector of the anchor
    /// polygon(s).
    #[must_use]
    pub fn area_vector(&self) -> Vec3 {
        if !self.has_points() {
            return [0.0; 3];
        }
        let mut area = [0.0; 3];
        for subpath in self.subpaths() {
            let anchors: Vec<Vec3> = subpath.iter().step_by(2).copied().collect();
            if anchors.is_empty() {
                continue;
            }
            for i in 0..anchors.len() {
                let p0 = anchors[i];
                let p1 = anchors[(i + 1) % anchors.len()];
                area[0] += 0.5 * (p0[1] + p1[1]) * (p1[2] - p0[2]);
                area[1] += 0.5 * (p0[2] + p1[2]) * (p1[0] - p0[0]);
                area[2] += 0.5 * (p0[0] + p1[0]) * (p1[1] - p0[1]);
            }
        }
        area
    }

    /// `get_unit_normal`.
    #[must_use]
    pub fn unit_normal(&self) -> Vec3 {
        if self.num_points() < 3 {
            return OUT;
        }
        let area_vect = self.area_vector();
        let area = space_ops::get_norm(area_vect);
        if area > 0.0 {
            vec::scale(area_vect, 1.0 / area)
        } else {
            space_ops::get_unit_normal(
                vec::sub(self.points[1], self.points[0]),
                vec::sub(self.points[2], self.points[1]),
                1e-6,
            )
        }
    }

    /// `get_joint_angles`: per-point signed turn angles between the tangent
    /// into and out of each vertex, in the plane of the path's unit normal.
    /// Closed subpaths join their seam tangents; open ends get zero turn.
    ///
    /// Computed fresh on every call; revisioned caching arrives with the
    /// render mirrors (§8.2 / §10.8), never here.
    #[must_use]
    pub fn joint_angles(&self) -> Vec<f64> {
        let n = self.points.len();
        if n < 3 {
            return vec![0.0; n];
        }
        let rot = space_ops::rotation_between_vectors(OUT, self.unit_normal());
        let points: Vec<Vec3> = self
            .points
            .iter()
            .map(|p| vec::mul_point_mat(*p, &rot))
            .collect();

        // Tangent vectors into and out of each vertex.
        let mut v_in = vec![[0.0f64; 3]; n];
        let mut v_out = vec![[0.0f64; 3]; n];
        for i in 0..(n - 1) / 2 {
            let a0_to_h = vec::sub(points[2 * i + 1], points[2 * i]);
            let h_to_a1 = vec::sub(points[2 * i + 2], points[2 * i + 1]);
            v_in[2 * i + 1] = a0_to_h;
            v_in[2 * i + 2] = h_to_a1;
            v_out[2 * i] = a0_to_h;
            v_out[2 * i + 1] = h_to_a1;
        }

        // Join up closed loops; mark unclosed path ends as straight-through.
        let ends = self.subpath_end_indices();
        let mut start = 0usize;
        for &end in &ends {
            if start != end {
                if points[start] == points[end] {
                    v_in[start] = v_out[end - 1];
                    v_out[end] = v_in[start + 1];
                } else {
                    v_in[start] = v_out[start];
                    v_out[end] = v_in[end];
                }
            }
            start = end + 2;
        }

        (0..n)
            .map(|i| {
                let angle_in = scalar::atan2(v_in[i][1], v_in[i][0]);
                let angle_out = scalar::atan2(v_out[i][1], v_out[i][0]);
                let mut diff = angle_out - angle_in;
                if diff < -PI {
                    diff += TAU;
                }
                if diff > PI {
                    diff -= TAU;
                }
                diff
            })
            .collect()
    }

    /// `reverse_points`: reverse the trace direction, repositioning break
    /// markers so subpath decomposition survives, exactly as the Reference
    /// does.
    pub fn reverse_points(&mut self) -> &mut Self {
        if !self.has_points() {
            return self;
        }
        let ends = self.subpath_end_indices();
        for &end in &ends[..ends.len().saturating_sub(1)] {
            self.points[end + 1] = self.points[end + 2];
        }
        self.points.reverse();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn right_angle_curve() -> QuadPath {
        QuadPath::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]]).unwrap()
    }

    #[test]
    fn subdivision_preflight_enforces_exact_and_overflow_boundaries_atomically() {
        let mut exact = right_angle_curve();
        let calls = Cell::new(0usize);
        exact
            .subdivide_curves_by_condition(|_| {
                calls.set(calls.get() + 1);
                MAX_SUBDIVIDED_CURVES - 1
            })
            .unwrap();
        assert_eq!(calls.get(), 1);
        assert_eq!(exact.num_curves(), MAX_SUBDIVIDED_CURVES);
        assert_eq!(exact.num_points(), 2 * MAX_SUBDIVIDED_CURVES + 1);

        let original = right_angle_curve();
        let mut one_over = original.clone();
        assert_eq!(
            one_over
                .subdivide_curves_by_condition(|_| MAX_SUBDIVIDED_CURVES)
                .unwrap_err(),
            GeomError::SubdivisionBudgetExceeded {
                requested: MAX_SUBDIVIDED_CURVES + 1,
                max: MAX_SUBDIVIDED_CURVES,
            }
        );
        assert_eq!(one_over, original);

        let mut arithmetic_overflow = original.clone();
        assert_eq!(
            arithmetic_overflow
                .subdivide_curves_by_condition(|_| usize::MAX)
                .unwrap_err(),
            GeomError::SubdivisionBudgetExceeded {
                requested: usize::MAX,
                max: MAX_SUBDIVIDED_CURVES,
            }
        );
        assert_eq!(arithmetic_overflow, original);
    }

    #[test]
    fn subdivision_preflight_bounds_aggregate_work_and_calls_once_per_curve() {
        let mut path = QuadPath::new();
        path.set_points_as_corners(&[[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [2.0, 0.0, 0.0]])
            .unwrap();
        let original = path.clone();
        let calls = Cell::new(0usize);
        let first_curve_subdivisions = MAX_SUBDIVIDED_CURVES - 1;
        assert_eq!(
            path.subdivide_curves_by_condition(|_| {
                let call = calls.get();
                calls.set(call + 1);
                if call == 0 {
                    first_curve_subdivisions
                } else {
                    0
                }
            })
            .unwrap_err(),
            GeomError::SubdivisionBudgetExceeded {
                requested: MAX_SUBDIVIDED_CURVES + 1,
                max: MAX_SUBDIVIDED_CURVES,
            }
        );
        assert_eq!(calls.get(), 2);
        assert_eq!(path, original);
    }

    #[test]
    fn sharp_subdivision_rejects_invalid_thresholds_without_mutation() {
        let original = right_angle_curve();
        for threshold in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut path = original.clone();
            assert_eq!(
                path.subdivide_sharp_curves(threshold).unwrap_err(),
                GeomError::InvalidSubdivisionThreshold
            );
            assert_eq!(path, original);
        }

        let mut too_fine = original.clone();
        assert_eq!(
            too_fine
                .subdivide_sharp_curves(f64::MIN_POSITIVE)
                .unwrap_err(),
            GeomError::SubdivisionBudgetExceeded {
                requested: usize::MAX,
                max: MAX_SUBDIVIDED_CURVES,
            }
        );
        assert_eq!(too_fine, original);
    }

    #[test]
    fn curve_insertion_preflight_enforces_layout_budget_and_atomicity() {
        let point = [3.0, 4.0, 0.0];
        let exact = QuadPath::insert_n_curves_to_point_list(
            MAX_SUBDIVIDED_CURVES,
            &[point],
            DEFAULT_TOLERANCE_FOR_POINT_EQUALITY,
        )
        .unwrap();
        assert_eq!(exact.len(), 2 * MAX_SUBDIVIDED_CURVES + 1);
        assert!(exact.iter().all(|candidate| *candidate == point));

        assert_eq!(
            QuadPath::insert_n_curves_to_point_list(
                MAX_SUBDIVIDED_CURVES + 1,
                &[point],
                DEFAULT_TOLERANCE_FOR_POINT_EQUALITY,
            )
            .unwrap_err(),
            GeomError::SubdivisionBudgetExceeded {
                requested: MAX_SUBDIVIDED_CURVES + 1,
                max: MAX_SUBDIVIDED_CURVES,
            }
        );
        assert_eq!(
            QuadPath::insert_n_curves_to_point_list(
                usize::MAX,
                &[point],
                DEFAULT_TOLERANCE_FOR_POINT_EQUALITY,
            )
            .unwrap_err(),
            GeomError::SubdivisionBudgetExceeded {
                requested: usize::MAX,
                max: MAX_SUBDIVIDED_CURVES,
            }
        );
        assert_eq!(
            QuadPath::insert_n_curves_to_point_list(
                0,
                &[point, point],
                DEFAULT_TOLERANCE_FOR_POINT_EQUALITY,
            )
            .unwrap_err(),
            GeomError::EvenPointCount { len: 2 }
        );
        assert!(
            QuadPath::insert_n_curves_to_point_list(
                usize::MAX,
                &[],
                DEFAULT_TOLERANCE_FOR_POINT_EQUALITY,
            )
            .unwrap()
            .is_empty()
        );

        let original = right_angle_curve();
        let mut path = original.clone();
        assert_eq!(
            path.insert_n_curves(MAX_SUBDIVIDED_CURVES).unwrap_err(),
            GeomError::SubdivisionBudgetExceeded {
                requested: MAX_SUBDIVIDED_CURVES + 1,
                max: MAX_SUBDIVIDED_CURVES,
            }
        );
        assert_eq!(path, original);
    }

    #[test]
    fn curve_insertion_tournament_preserves_first_max_distribution() {
        let points = [
            [0.0, 0.0, 0.0],
            [0.5, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [5.0, 0.0, 0.0],
        ];
        let inserted = QuadPath::insert_n_curves_to_point_list(
            4,
            &points,
            DEFAULT_TOLERANCE_FOR_POINT_EQUALITY,
        )
        .unwrap();
        let anchors: Vec<f64> = inserted.iter().step_by(2).map(|point| point[0]).collect();
        assert_eq!(anchors, vec![0.0, 0.5, 1.0, 2.0, 3.0, 4.0, 5.0]);

        let (tree, _) = curve_winner_tree(&[1.0, f64::NAN, 100.0]).unwrap();
        assert_eq!(tree[1], 2, "later NaN is ignored like np.argmax");
        let (tree, _) = curve_winner_tree(&[f64::NAN, 100.0]).unwrap();
        assert_eq!(tree[1], 0, "leading NaN remains the first winner");
        let (tree, _) = curve_winner_tree(&[2.0, 2.0, 2.0]).unwrap();
        assert_eq!(tree[1], 0, "first index wins equal priorities");
    }

    #[test]
    fn arc_constructor_uses_bn09_density() {
        let quarter = QuadPath::try_arc(0.0, TAU / 4.0, 1.0, [0.0; 3], None).expect("valid arc");
        assert_eq!(quarter.num_curves(), 4);
        let full = QuadPath::try_arc(0.0, TAU, 2.0, [1.0, 0.0, 0.0], None).expect("valid arc");
        assert_eq!(full.num_curves(), 16);
        // Anchors sit on the circle of radius 2 about (1, 0, 0).
        for anchor in full.anchors() {
            let r = space_ops::get_norm(vec::sub(anchor, [1.0, 0.0, 0.0]));
            assert!((r - 2.0).abs() < 1e-12);
        }
    }

    #[test]
    fn arc_public_surfaces_refuse_invalid_components_before_mutation() {
        assert_eq!(
            QuadPath::try_arc(0.0, TAU / 4.0, 1.0, [0.0; 3], Some(0)),
            Err(GeomError::ZeroArcComponents)
        );
        assert_eq!(
            QuadPath::try_arc(0.0, f64::NAN, 1.0, [0.0; 3], None),
            Err(GeomError::NonFiniteArcAngle)
        );
        assert_eq!(
            QuadPath::try_arc(0.0, TAU / 4.0, 1.0, [0.0; 3], Some(usize::MAX)),
            Err(GeomError::ArcComponentOverflow { count: usize::MAX })
        );

        let mut path = QuadPath::new();
        path.start_new_path([0.0; 3]);
        let original = path.clone();
        assert!(matches!(
            path.add_arc_to([1.0, 0.0, 0.0], 0.0, Some(0)),
            Err(GeomError::ZeroArcComponents)
        ));
        assert_eq!(path, original, "a tiny-angle refusal must be atomic");
        assert!(matches!(
            path.add_arc_to([1.0, 0.0, 0.0], f64::INFINITY, None),
            Err(GeomError::NonFiniteArcAngle)
        ));
        assert_eq!(path, original, "a non-finite refusal must be atomic");
    }

    #[test]
    fn add_arc_to_honors_the_callers_strict_line_threshold() {
        let mut as_line = QuadPath::new();
        as_line.start_new_path([0.0; 3]);
        as_line
            .add_arc_to_with_threshold([1.0, 0.0, 0.0], 5e-4, Some(1), 1e-3)
            .unwrap();
        assert_eq!(as_line.points()[1], [0.5, 0.0, 0.0]);

        let mut as_arc = QuadPath::new();
        as_arc.start_new_path([0.0; 3]);
        as_arc
            .add_arc_to_with_threshold([1.0, 0.0, 0.0], 5e-4, Some(1), 1e-4)
            .unwrap();
        assert_ne!(as_arc.points()[1], [0.5, 0.0, 0.0]);

        let mut exact_boundary = QuadPath::new();
        exact_boundary.start_new_path([0.0; 3]);
        exact_boundary
            .add_arc_to_with_threshold([1.0, 0.0, 0.0], 5e-4, Some(1), 5e-4)
            .unwrap();
        assert_eq!(
            exact_boundary.points(),
            as_arc.points(),
            "the public threshold comparison is strict"
        );
    }

    #[test]
    fn joint_angles_zero_for_straight_line() {
        let mut path = QuadPath::new();
        path.start_new_path([0.0; 3]);
        path.add_line_to([1.0, 0.0, 0.0], true).unwrap();
        path.add_line_to([2.0, 0.0, 0.0], true).unwrap();
        for angle in path.joint_angles() {
            assert!(angle.abs() < 1e-12);
        }
    }
}
