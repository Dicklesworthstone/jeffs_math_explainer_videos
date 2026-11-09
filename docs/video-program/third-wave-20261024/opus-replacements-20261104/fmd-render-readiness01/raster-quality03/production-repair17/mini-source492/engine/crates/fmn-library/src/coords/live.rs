//! Coordinate maps read from live native axes, including affine animation.

use fmn_geom::space_ops::{cross, dot};
use fmn_mobject::{CopyMap, Mob, Stage};

use super::{Axes, CoordinateSystem, CoordsError, NumberLine, Vec3, add, scale, sub};
use crate::graphs::{
    GraphError, ParametricCurve, ParametricCurveSpec, SamplingBudget, graph_parametric,
};
use crate::planes::{ComplexPlane, NumberPlane, ThreeDAxes};
use crate::solids::{ParametricSurface, Surface, SurfaceSampleError, SurfaceSpec};
use crate::vmobject::VMobject;

/// Why a native chart cannot supply an affine coordinate map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoordinateFrameError {
    /// The detached chart must be built before its numeric axes can be bound.
    UnbuiltChart,
    /// A root or axis handle was deleted, or belongs to another Stage.
    StaleHandle(Mob),
    /// The supplied root does not contain an axis in the chart's normal layout.
    MissingAxis {
        /// Numeric axis index (x = 0, y = 1, z = 2).
        axis: usize,
        /// Expected child slot in the chart root.
        child_index: usize,
    },
    /// A previously bound axis was replaced, removed, or moved to another slot.
    AxisChanged {
        /// Numeric axis index.
        axis: usize,
    },
    /// Axis geometry or its numeric calibration cannot describe a finite line.
    InvalidAxis {
        /// Numeric axis index.
        axis: usize,
        /// The failed geometry or calibration condition.
        reason: &'static str,
    },
    /// The axes no longer meet at one numeric origin.
    DisconnectedOrigins {
        /// Numeric axis index whose origin differs from the x-axis origin.
        axis: usize,
    },
    /// The axes are collapsed, parallel, or numerically singular.
    DegenerateBasis,
    /// A live conversion received or produced non-finite coordinates.
    NonFiniteCoordinates,
    /// The requested construction needs a different coordinate dimension.
    DimensionMismatch {
        /// Required number of coordinate axes.
        expected: usize,
        /// Number of axes in the bound frame.
        actual: usize,
    },
}

impl std::fmt::Display for CoordinateFrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnbuiltChart => f.write_str("build the coordinate system before binding it"),
            Self::StaleHandle(mob) => write!(f, "stale or foreign coordinate handle {mob:?}"),
            Self::MissingAxis { axis, child_index } => {
                write!(
                    f,
                    "axis {axis} is missing from chart child slot {child_index}"
                )
            }
            Self::AxisChanged { axis } => {
                write!(
                    f,
                    "axis {axis} was replaced or reordered; bind the new chart family"
                )
            }
            Self::InvalidAxis { axis, reason } => write!(f, "invalid axis {axis}: {reason}"),
            Self::DisconnectedOrigins { axis } => {
                write!(f, "axis {axis} no longer shares the chart origin")
            }
            Self::DegenerateBasis => f.write_str("coordinate axes have no stable affine inverse"),
            Self::NonFiniteCoordinates => f.write_str("coordinate conversion is not finite"),
            Self::DimensionMismatch { expected, actual } => {
                write!(
                    f,
                    "construction requires {expected} coordinate axes, got {actual}"
                )
            }
        }
    }
}

impl std::error::Error for CoordinateFrameError {}

#[derive(Debug, Clone)]
struct AxisBinding {
    mob: Mob,
    child_index: usize,
    // Numeric coordinates of the shaft's endpoints, not necessarily min/max:
    // attaching an arrow tip trims the shaft but preserves NumberLine's map.
    shaft_range: [f64; 2],
}

/// A chart whose coordinate conversions follow its registered Stage geometry.
///
/// Create it with [`Axes::bind`], [`ThreeDAxes::bind`], [`NumberPlane::bind`], or
/// [`ComplexPlane::bind`]. Each conversion reads the current world-space axes,
/// so translations, rotations, nonuniform scales, shears, and affine Transform
/// samples update the map without mutating the original detached builder.
///
/// Cloning a binding keeps the same Stage-scoped handles. Use [`Self::rebind`]
/// for a copied chart. A binding refuses replaced/reordered axes and nonlinear
/// axis geometry; it never silently follows an unrelated replacement handle.
/// Removing a chart from scene membership does not invalidate its live handles.
///
/// To build several objects in one consistent frame, take one [`Self::snapshot`].
/// In a native updater, take a new snapshot on each tick before building a graph
/// or moving a label. The resulting [`CoordinateFrame`] owns no Stage borrow.
#[derive(Debug, Clone)]
pub struct LiveCoordinateSystem {
    root: Mob,
    axes: Vec<AxisBinding>,
    ranges: Vec<[f64; 3]>,
    samples_per_tick: f64,
    sampling_budget: SamplingBudget,
}

impl LiveCoordinateSystem {
    fn bind_axes<const N: usize>(
        stage: &Stage,
        root: Mob,
        chart: &impl CoordinateSystem,
        child_indices: [usize; N],
        number_lines: [&NumberLine; N],
    ) -> Result<Self, CoordsError> {
        let children = stage
            .get(root)
            .ok_or(CoordinateFrameError::StaleHandle(root))?
            .submobjects();
        let mut axes = Vec::with_capacity(N);
        for (axis, (&child_index, line)) in child_indices.iter().zip(number_lines).enumerate() {
            let mob = *children
                .get(child_index)
                .ok_or(CoordinateFrameError::MissingAxis { axis, child_index })?;
            let invalid = |reason| CoordinateFrameError::InvalidAxis { axis, reason };
            let points = line.vmob().points();
            let (&start, &end) = points
                .first()
                .zip(points.last())
                .ok_or(CoordinateFrameError::UnbuiltChart)?;
            let shaft_range = [line.p2n(start), line.p2n(end)];
            let span = shaft_range[1] - shaft_range[0];
            if !line.x_range().iter().all(|n| n.is_finite())
                || !shaft_range.iter().all(|n| n.is_finite())
                || !span.is_finite()
                || span == 0.0
            {
                return Err(invalid("numeric shaft range must be finite and nonzero").into());
            }
            axes.push(AxisBinding {
                mob,
                child_index,
                shaft_range,
            });
        }
        let binding = Self {
            root,
            axes,
            ranges: chart.all_ranges(),
            samples_per_tick: chart.num_sampled_graph_points_per_tick(),
            sampling_budget: chart.graph_sampling_budget(),
        };
        binding.snapshot(stage)?;
        Ok(binding)
    }

    /// The registered chart family root.
    #[must_use]
    pub fn root(&self) -> Mob {
        self.root
    }

    /// A bound axis handle (x = 0, y = 1, z = 2).
    #[must_use]
    pub fn axis(&self, index: usize) -> Option<Mob> {
        self.axes.get(index).map(|axis| axis.mob)
    }

    /// Remap family-internal handles when copying native owned updater state.
    ///
    /// Pass this method as the remapper when registering a binding with
    /// `Stage::add_dt_updater_with_state`, or call it from a larger state's
    /// remapper. When the chart root is outside the copied family, the whole
    /// binding remains shared, even if one axis alone was copied. A copied
    /// chart root brings all its axes with it; the next snapshot validates
    /// that the remapped root still owns those axes.
    pub fn remap_handles(&mut self, map: &CopyMap) {
        let Some(root) = map.get(self.root) else {
            return;
        };
        self.root = root;
        for axis in &mut self.axes {
            axis.mob = map.get(axis.mob).unwrap_or(axis.mob);
        }
    }

    /// Bind the same numeric ranges and shaft calibration to another chart root.
    ///
    /// This supports both `Stage::copy_family` and copies into a different Stage.
    /// The new root must have the same axis child layout as the original chart.
    /// Validation finishes before a new binding is returned.
    pub fn rebind(&self, stage: &Stage, root: Mob) -> Result<Self, CoordsError> {
        let children = stage
            .get(root)
            .ok_or(CoordinateFrameError::StaleHandle(root))?
            .submobjects();
        let mut rebound = self.clone();
        rebound.root = root;
        for (axis, binding) in rebound.axes.iter_mut().enumerate() {
            binding.mob =
                *children
                    .get(binding.child_index)
                    .ok_or(CoordinateFrameError::MissingAxis {
                        axis,
                        child_index: binding.child_index,
                    })?;
        }
        rebound.snapshot(stage)?;
        Ok(rebound)
    }

    /// Freeze the current world-space coordinate map without retaining a Stage.
    ///
    /// The inverse uses a dual basis, so nonorthogonal axes round-trip correctly.
    /// For a 2D chart embedded in 3D, `p2c` projects onto its plane. Native record
    /// rounding is tolerated when checking straight axes and their common origin.
    /// Deleted handles, altered axis layout, non-finite/curved geometry, separated
    /// origins, and singular frames return [`CoordsError::LiveFrame`].
    pub fn snapshot(&self, stage: &Stage) -> Result<CoordinateFrame, CoordsError> {
        let children = stage
            .get(self.root)
            .ok_or(CoordinateFrameError::StaleHandle(self.root))?
            .submobjects();
        let mut origin = [0.0; 3];
        let mut basis = [[0.0; 3]; 3];
        let mut origin_tolerance = 0.0;
        for (axis, binding) in self.axes.iter().enumerate() {
            let entry = stage
                .get(binding.mob)
                .ok_or(CoordinateFrameError::StaleHandle(binding.mob))?;
            if children.get(binding.child_index) != Some(&binding.mob) {
                return Err(CoordinateFrameError::AxisChanged { axis }.into());
            }
            let invalid = |reason| CoordinateFrameError::InvalidAxis { axis, reason };
            let schema = entry.buffer.schema();
            // Check widths before calling get_start_and_end, which reads three
            // point lanes. A custom record layout must never trigger an index panic.
            if schema.field_width("point") != Some(3)
                || schema.field_width("joint_angle") != Some(1)
            {
                return Err(invalid("expected a VMobject with three point lanes").into());
            }
            if entry.buffer.len() < 2 {
                return Err(invalid("axis shaft has fewer than two points").into());
            }
            let (start, end) = stage
                .get_start_and_end(binding.mob)
                .ok_or_else(|| invalid("axis shaft has no endpoints"))?;
            let span = sub(end, start);
            let span_scale = max_abs(span);
            if !finite(start) || !finite(end) || !finite(span) {
                return Err(invalid("axis endpoints are not finite").into());
            }
            if span_scale == 0.0 {
                return Err(CoordinateFrameError::DegenerateBasis.into());
            }
            let translation = entry.placement().translation();
            let linear_size = max_abs(sub(start, translation)).max(max_abs(sub(end, translation)));
            let world_size = max_abs(start).max(max_abs(end));
            let tolerance =
                16.0 * f64::from(f32::EPSILON) * linear_size + 32.0 * f64::EPSILON * world_size;
            let direction = divide(span, span_scale);
            let points = stage
                .get_points(binding.mob)
                .ok_or_else(|| invalid("axis has no point field"))?;
            for point in points {
                if !finite(point) {
                    return Err(invalid("axis points are not finite").into());
                }
                let offset = divide(sub(point, start), span_scale);
                if !finite(offset)
                    || max_abs(cross(offset, direction)) > 4.0 * tolerance / span_scale
                {
                    return Err(invalid("axis shaft is no longer straight").into());
                }
            }
            basis[axis] = divide(span, binding.shaft_range[1] - binding.shaft_range[0]);
            let axis_origin = sub(start, scale(basis[axis], binding.shaft_range[0]));
            if !finite(basis[axis]) || !finite(axis_origin) {
                return Err(invalid("axis calibration produces non-finite coordinates").into());
            }
            // Extrapolating the shaft to numeric zero magnifies endpoint rounding.
            let tolerance = tolerance
                * (1.0
                    + (binding.shaft_range[0] / (binding.shaft_range[1] - binding.shaft_range[0]))
                        .abs());
            if axis == 0 {
                origin = axis_origin;
                origin_tolerance = tolerance;
            } else if max_abs(sub(axis_origin, origin)) > origin_tolerance + tolerance {
                return Err(CoordinateFrameError::DisconnectedOrigins { axis }.into());
            }
        }
        let dual = dual_basis(basis, self.axes.len())?;
        Ok(CoordinateFrame {
            origin,
            basis,
            dual,
            ranges: self.ranges.clone(),
            samples_per_tick: self.samples_per_tick,
            sampling_budget: self.sampling_budget,
        })
    }

    /// Convert through the current Stage frame. Missing components are zero.
    pub fn c2p(&self, stage: &Stage, coords: &[f64]) -> Result<Vec3, CoordsError> {
        if !coords.iter().take(self.axes.len()).all(|n| n.is_finite()) {
            return Err(CoordinateFrameError::NonFiniteCoordinates.into());
        }
        finite_result(self.snapshot(stage)?.c2p(coords))
    }

    /// Invert the current Stage frame, zero-padding a 2D result.
    pub fn p2c(&self, stage: &Stage, point: Vec3) -> Result<[f64; 3], CoordsError> {
        if !finite(point) {
            return Err(CoordinateFrameError::NonFiniteCoordinates.into());
        }
        finite_result(self.snapshot(stage)?.p2c(point))
    }

    /// Build a graph from one snapshot of the current Stage axes.
    ///
    /// Taking another graph after movement samples the new frame; an already
    /// returned graph builder keeps the frame in which it was created.
    pub fn get_graph(
        &self,
        stage: &Stage,
        function: impl Fn(f64) -> f64 + 'static,
    ) -> Result<ParametricCurve, CoordsError> {
        Ok(self.snapshot(stage)?.get_graph(function))
    }

    /// Build `(x(t), y(t), z(t))` from one snapshot of the current Stage axes.
    ///
    /// The curve inherits this chart's curve sampling budget. Parameter range,
    /// discontinuities, and smoothing remain configurable on the returned builder.
    pub fn get_parametric_curve(
        &self,
        stage: &Stage,
        function: impl Fn(f64) -> Vec3 + 'static,
    ) -> Result<ParametricCurve, CoordsError> {
        Ok(self.snapshot(stage)?.get_parametric_curve(function))
    }

    /// Build `z = function(x, y)` over the current 3D chart's x/y ranges.
    ///
    /// A 2D binding returns a typed dimension error. Surface sampling uses
    /// ParametricSurface's independent grid budget and configurable resolution.
    pub fn get_surface(
        &self,
        stage: &Stage,
        function: impl Fn(f64, f64) -> f64 + 'static,
    ) -> Result<ParametricSurface, CoordsError> {
        self.snapshot(stage)?.get_surface(function)
    }

    /// Build a parameterized surface from one current affine chart snapshot.
    ///
    /// Vertices and derivative probes use the same snapshot, even if the Stage
    /// moves before the returned builder is sampled. For a borrowed or fallible
    /// callback, use `snapshot(stage)?.try_parametric_surface(...)` instead.
    pub fn get_parametric_surface(
        &self,
        stage: &Stage,
        function: impl Fn(f64, f64) -> Vec3 + 'static,
    ) -> Result<ParametricSurface, CoordsError> {
        Ok(self.snapshot(stage)?.get_parametric_surface(function))
    }
}

/// An immutable affine chart, suitable for graph, surface, and field builders.
///
/// Obtained from [`LiveCoordinateSystem::snapshot`]. Later Stage mutations,
/// copies, and deletions cannot change it. Its inverse uses the full affine
/// basis; it does not assume that axes are perpendicular or equally scaled.
/// Like other [`CoordinateSystem`] implementations, direct trait conversions
/// accept arbitrary floating-point inputs. The live conversion methods also
/// reject non-finite input and output.
#[derive(Debug, Clone)]
pub struct CoordinateFrame {
    origin: Vec3,
    basis: [Vec3; 3],
    dual: [Vec3; 3],
    ranges: Vec<[f64; 3]>,
    samples_per_tick: f64,
    sampling_budget: SamplingBudget,
}

impl CoordinateSystem for CoordinateFrame {
    fn c2p(&self, coords: &[f64]) -> Vec3 {
        let mut point = self.origin;
        for (&coord, basis) in coords.iter().zip(&self.basis[..self.dimension()]) {
            point = add(point, scale(*basis, coord));
        }
        point
    }

    fn p2c(&self, point: Vec3) -> [f64; 3] {
        let offset = sub(point, self.origin);
        std::array::from_fn(|axis| {
            if axis < self.dimension() {
                dot(offset, self.dual[axis])
            } else {
                0.0
            }
        })
    }

    fn all_ranges(&self) -> Vec<[f64; 3]> {
        self.ranges.clone()
    }

    fn num_sampled_graph_points_per_tick(&self) -> f64 {
        self.samples_per_tick
    }

    fn graph_sampling_budget(&self) -> SamplingBudget {
        self.sampling_budget
    }

    fn dimension(&self) -> usize {
        self.ranges.len()
    }

    fn origin(&self) -> Vec3 {
        self.origin
    }
}

impl CoordinateFrame {
    /// Sample `y = function(x)` in this frozen chart using its original range,
    /// samples per tick, and resource budget. A 3D chart uses numeric z = 0.
    #[must_use]
    pub fn get_graph(&self, function: impl Fn(f64) -> f64 + 'static) -> ParametricCurve {
        let frame = self.clone();
        graph_parametric(
            function,
            move |coords| frame.c2p(coords),
            self.ranges[0],
            self.samples_per_tick,
            self.sampling_budget,
        )
    }

    /// Build a curve from chart coordinates `(x(t), y(t), z(t))`.
    ///
    /// Like ThreeDAxes' helper, this keeps ParametricCurve's default parameter
    /// range and inherits the chart's curve sampling budget. All ordinary curve
    /// builder overrides remain available. A 2D frame uses the first two
    /// returned coordinates and embeds the curve in its plane.
    #[must_use]
    pub fn get_parametric_curve(
        &self,
        function: impl Fn(f64) -> Vec3 + 'static,
    ) -> ParametricCurve {
        let frame = self.clone();
        ParametricCurve::new(move |t| frame.c2p(&function(t))).sampling_budget(self.sampling_budget)
    }

    /// Sample a borrowed, fallible curve callback in this frozen chart.
    ///
    /// The explicit specification owns sampling controls and its budget. The
    /// shared native sampler admits work before invoking the callback, preserves
    /// the first callback error, and refuses non-finite mapped record points.
    pub fn try_parametric_curve<E>(
        &self,
        mut function: impl FnMut(f64) -> Result<Vec3, E>,
        spec: &ParametricCurveSpec,
    ) -> Result<VMobject, GraphError<E>> {
        spec.try_sample(|t| function(t).map(|point| self.c2p(&point)))
    }

    /// Build a height surface `z = function(x, y)` over this chart's x/y ranges.
    ///
    /// The returned builder uses ParametricSurface's independent grid budget;
    /// range, resolution, color, opacity, and shading overrides are unchanged.
    /// A 2D frame returns [`CoordinateFrameError::DimensionMismatch`] rather than
    /// discarding the height coordinate.
    pub fn get_surface(
        &self,
        function: impl Fn(f64, f64) -> f64 + 'static,
    ) -> Result<ParametricSurface, CoordsError> {
        if self.dimension() != 3 {
            return Err(CoordinateFrameError::DimensionMismatch {
                expected: 3,
                actual: self.dimension(),
            }
            .into());
        }
        Ok(self
            .get_parametric_surface(move |x, y| [x, y, function(x, y)])
            .u_range(self.ranges[0][0], self.ranges[0][1])
            .v_range(self.ranges[1][0], self.ranges[1][1]))
    }

    /// Build a parameterized surface in this frozen coordinate frame.
    ///
    /// Both parameter ranges default to `(0, 1)`. The native surface sampler
    /// maps vertices and derivative probes through c2p before deriving normals,
    /// so lighting follows nonuniform scale, shear, and reflection. A 2D frame
    /// produces a surface embedded in its plane using the first two coordinates.
    #[must_use]
    pub fn get_parametric_surface(
        &self,
        function: impl Fn(f64, f64) -> Vec3 + 'static,
    ) -> ParametricSurface {
        let frame = self.clone();
        ParametricSurface::new(move |u, v| frame.c2p(&function(u, v)))
    }

    /// Sample a borrowed, fallible surface callback in this frozen chart.
    ///
    /// The explicit native specification and grid budget are authoritative.
    /// Admission, callback errors, and non-finite/f32-unrepresentable mapped
    /// samples use the existing [`SurfaceSampleError`] contract; no partial
    /// surface is returned. Derivative probes also pass through this frame.
    pub fn try_parametric_surface<E>(
        &self,
        mut function: impl FnMut(f64, f64) -> Result<Vec3, E>,
        spec: &SurfaceSpec,
        budget: SamplingBudget,
    ) -> Result<Surface, SurfaceSampleError<E>> {
        spec.try_sample_with_budget(|u, v| function(u, v).map(|point| self.c2p(&point)), budget)
    }

    /// The world-space graph point `c2p(x, function(x))`.
    #[must_use]
    pub fn input_to_graph_point(&self, x: f64, function: &dyn Fn(f64) -> f64) -> Vec3 {
        self.c2p(&[x, function(x)])
    }
}

impl Axes {
    /// Bind this built chart's numeric map to its registered Stage family.
    /// The root must have the same `[x_axis, y_axis]` layout as `self.vmob()`.
    pub fn bind(&self, stage: &Stage, root: Mob) -> Result<LiveCoordinateSystem, CoordsError> {
        if self.vmob.children().is_empty() {
            return Err(CoordinateFrameError::UnbuiltChart.into());
        }
        LiveCoordinateSystem::bind_axes(stage, root, self, [0, 1], [&self.x_axis, &self.y_axis])
    }
}

impl ThreeDAxes {
    /// Bind this built chart to a registered `[x_axis, y_axis, z_axis]` family.
    pub fn bind(&self, stage: &Stage, root: Mob) -> Result<LiveCoordinateSystem, CoordsError> {
        let axes = self
            .coordinate_axes()
            .ok_or(CoordinateFrameError::UnbuiltChart)?;
        LiveCoordinateSystem::bind_axes(stage, root, self, [0, 1, 2], axes)
    }
}

impl NumberPlane {
    /// Bind the numeric axes in a registered plane, after its two grid groups.
    pub fn bind(&self, stage: &Stage, root: Mob) -> Result<LiveCoordinateSystem, CoordsError> {
        let axes = self
            .coordinate_axes()
            .ok_or(CoordinateFrameError::UnbuiltChart)?;
        LiveCoordinateSystem::bind_axes(stage, root, self, [2, 3], axes)
    }
}

impl ComplexPlane {
    /// Bind a registered complex plane, preserving its real/imaginary ranges.
    pub fn bind(&self, stage: &Stage, root: Mob) -> Result<LiveCoordinateSystem, CoordsError> {
        let axes = self
            .coordinate_axes()
            .ok_or(CoordinateFrameError::UnbuiltChart)?;
        LiveCoordinateSystem::bind_axes(stage, root, self, [2, 3], axes)
    }
}

fn finite(point: Vec3) -> bool {
    point.iter().all(|n| n.is_finite())
}

fn finite_result(point: Vec3) -> Result<Vec3, CoordsError> {
    if finite(point) {
        Ok(point)
    } else {
        Err(CoordinateFrameError::NonFiniteCoordinates.into())
    }
}

fn max_abs(point: Vec3) -> f64 {
    point[0].abs().max(point[1].abs()).max(point[2].abs())
}

fn divide(point: Vec3, divisor: f64) -> Vec3 {
    // Componentwise division avoids an overflowing reciprocal for tiny axes.
    point.map(|value| value / divisor)
}

fn dual_basis(basis: [Vec3; 3], dimension: usize) -> Result<[Vec3; 3], CoordinateFrameError> {
    // Normalize each axis independently before taking cross products. This
    // admits large/small and strongly nonuniform scales without squaring them.
    let mut normalized = [[0.0; 3]; 3];
    let mut scales = [0.0; 3];
    for axis in 0..dimension {
        scales[axis] = max_abs(basis[axis]);
        if !scales[axis].is_finite() || scales[axis] == 0.0 {
            return Err(CoordinateFrameError::DegenerateBasis);
        }
        normalized[axis] = divide(basis[axis], scales[axis]);
    }
    let [x, y, z] = normalized;
    let normal = cross(x, y);
    let singular_tolerance = 64.0 * f64::EPSILON;
    let mut dual = [[0.0; 3]; 3];
    if dimension == 2 {
        let denominator = dot(normal, normal);
        if denominator <= singular_tolerance * singular_tolerance {
            return Err(CoordinateFrameError::DegenerateBasis);
        }
        dual[0] = divide(divide(cross(y, normal), denominator), scales[0]);
        dual[1] = divide(divide(cross(normal, x), denominator), scales[1]);
    } else {
        let determinant = dot(normal, z);
        if determinant.abs() <= singular_tolerance {
            return Err(CoordinateFrameError::DegenerateBasis);
        }
        dual[0] = divide(divide(cross(y, z), determinant), scales[0]);
        dual[1] = divide(divide(cross(z, x), determinant), scales[1]);
        dual[2] = divide(divide(normal, determinant), scales[2]);
    }
    if dual.iter().all(|&vector| finite(vector)) {
        Ok(dual)
    } else {
        Err(CoordinateFrameError::DegenerateBasis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AxisConfig, FontBook, Text};
    use fmn_anim::{AnimConfig, Animation, RateFunc, Transform};
    use fmn_mobject::{Mobject, Placement, RecordBuffer, RecordSchema};

    fn axes(tipped: bool) -> Axes {
        Axes::new()
            .x_range([-2.0, 4.0, 1.0])
            .y_range([-1.0, 3.0, 1.0])
            .width(6.0)
            .height(3.0)
            .axis_config(AxisConfig {
                include_tip: Some(tipped),
                include_ticks: Some(false),
                ..AxisConfig::default()
            })
            .build(&FontBook::bundled().unwrap())
            .unwrap()
    }

    fn near(actual: Vec3, expected: Vec3) {
        let tolerance = 3e-6 * (1.0 + max_abs(expected));
        for axis in 0..3 {
            assert!(
                (actual[axis] - expected[axis]).abs() <= tolerance,
                "{actual:?} differs from {expected:?}, tolerance {tolerance}"
            );
        }
    }

    fn live_error(result: Result<CoordinateFrame, CoordsError>) -> CoordinateFrameError {
        match result {
            Err(CoordsError::LiveFrame(error)) => error,
            other => panic!("expected a typed live-frame error, got {other:?}"),
        }
    }

    fn three_dimensional_chart() -> ThreeDAxes {
        ThreeDAxes::new()
            .x_range([-2.0, 3.0, 1.0])
            .y_range([-1.0, 2.0, 1.0])
            .z_range([-3.0, 1.0, 1.0])
            .width(4.0)
            .height(6.0)
            .depth(8.0)
            .axis_config(AxisConfig {
                include_tip: Some(true),
                include_ticks: Some(false),
                ..AxisConfig::default()
            })
            .build(&FontBook::bundled().unwrap())
            .unwrap()
    }

    fn surface_affine() -> Placement {
        // A reflection with shear and unequal scales exercises normal mapping.
        Placement::new(
            [[1.25, 0.3, -0.4], [-0.2, 0.9, 0.6], [0.5, -0.7, -1.1]],
            [2.0, -1.0, 0.5],
        )
    }

    #[test]
    fn live_parametric_curves_sample_three_axes_and_freeze_their_creation_frame() {
        let chart = three_dimensional_chart();
        let mut stage = Stage::new();
        let root = stage.add(chart.clone());
        let live = chart.bind(&stage, root).unwrap();
        let deferred = live
            .get_parametric_curve(&stage, |t| [t, t * t, 1.0 - t])
            .unwrap();
        let affine = surface_affine();
        stage.apply_affine(root, affine);
        let curve = live
            .get_parametric_curve(&stage, |t| [t, t * t, 1.0 - t])
            .unwrap()
            .t_range([-1.0, 1.0, 0.5])
            .use_smoothing(false)
            .build()
            .unwrap();
        let anchors = curve.path().unwrap().anchors();
        assert_eq!(anchors.len(), 5);
        for (point, t) in anchors.into_iter().zip([-1.0, -0.5, 0.0, 0.5, 1.0]) {
            near(point, affine.apply_point(chart.c2p(&[t, t * t, 1.0 - t])));
        }
        // The unmodified parameter domain is 0..1, independent of x_range.
        let original = deferred.build().unwrap();
        near(original.points()[0], chart.c2p(&[0.0, 0.0, 1.0]));
        near(
            *original.points().last().unwrap(),
            chart.c2p(&[1.0, 1.0, 0.0]),
        );
    }

    #[test]
    fn live_height_surfaces_use_chart_ranges_and_affine_derivative_normals() {
        // Surface grids keep their own budget: the twelve vertices below exceed
        // the chart's four-sample curve budget and are still admitted.
        let chart = three_dimensional_chart().sampling_budget(SamplingBudget::new(4));
        let mut stage = Stage::new();
        let root = stage.add(chart.clone());
        let live = chart.bind(&stage, root).unwrap();
        let affine = surface_affine();
        stage.apply_affine(root, affine);
        let surface = live
            .get_surface(&stage, |x, y| 2.0 * x - 3.0 * y + 0.5)
            .unwrap()
            .resolution(3, 4)
            .build();
        assert_eq!(surface.u_range(), (-2.0, 3.0));
        assert_eq!(surface.v_range(), (-1.0, 2.0));
        assert_eq!(surface.resolution(), (3, 4));
        assert_eq!(surface.triangle_indices().len(), 36);
        for (i, x) in [-2.0, 0.5, 3.0].into_iter().enumerate() {
            for (j, y) in [-1.0, 0.0, 1.0, 2.0].into_iter().enumerate() {
                near(
                    surface.points()[i * 4 + j],
                    affine.apply_point(chart.c2p(&[x, y, 2.0 * x - 3.0 * y + 0.5])),
                );
            }
        }
        let dx = affine.apply_vector(sub(chart.c2p(&[1.0, 0.0, 2.0]), chart.origin()));
        let dy = affine.apply_vector(sub(chart.c2p(&[0.0, 1.0, -3.0]), chart.origin()));
        let expected_normal = fmn_geom::space_ops::normalize(cross(dx, dy));
        for normal in surface.unit_normals() {
            near(normal, expected_normal);
        }
    }

    #[test]
    fn live_parametric_surfaces_keep_uv_controls_and_snapshot_positions() {
        let chart = three_dimensional_chart();
        let mut stage = Stage::new();
        let root = stage.add(chart.clone());
        let live = chart.bind(&stage, root).unwrap();
        let affine = surface_affine();
        stage.apply_affine(root, affine);
        let frame = live.snapshot(&stage).unwrap();
        let pending = live
            .get_parametric_surface(&stage, |u, v| [u + v, u - v, u * v])
            .unwrap();
        stage.shift(root, [7.0, 0.0, -2.0]);
        let surface = pending.resolution(2, 2).build();
        assert_eq!(surface.u_range(), (0.0, 1.0));
        assert_eq!(surface.v_range(), (0.0, 1.0));
        for (point, coordinates) in surface.points().iter().zip([
            [0.0, 0.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [2.0, 0.0, 1.0],
        ]) {
            near(*point, affine.apply_point(chart.c2p(&coordinates)));
        }
        let explicit = frame
            .get_parametric_surface(|u, v| [u + v, u - v, u * v])
            .u_range(-1.0, 1.0)
            .v_range(2.0, 3.0)
            .resolution(2, 2)
            .build();
        for (i, u) in [-1.0, 1.0].into_iter().enumerate() {
            for (j, v) in [2.0, 3.0].into_iter().enumerate() {
                near(
                    explicit.points()[i * 2 + j],
                    affine.apply_point(chart.c2p(&[u + v, u - v, u * v])),
                );
            }
        }
    }

    #[test]
    fn live_height_surface_requires_three_dimensions_but_planar_surfaces_are_supported() {
        let chart = axes(false);
        let mut stage = Stage::new();
        let root = stage.add(chart.clone());
        let live = chart.bind(&stage, root).unwrap();
        stage.apply_affine(root, surface_affine());
        let frame = live.snapshot(&stage).unwrap();
        for result in [
            live.get_surface(&stage, |x, y| x + y),
            frame.get_surface(|x, y| x + y),
        ] {
            assert!(matches!(
                result,
                Err(CoordsError::LiveFrame(
                    CoordinateFrameError::DimensionMismatch {
                        expected: 3,
                        actual: 2,
                    }
                ))
            ));
        }
        let planar = frame
            .get_parametric_surface(|u, v| [u, v, 99.0])
            .resolution(2, 2)
            .build();
        for (point, coordinates) in
            planar
                .points()
                .iter()
                .zip([[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]])
        {
            near(*point, frame.c2p(&coordinates));
        }
    }

    #[test]
    fn live_parametric_curve_budget_and_fallible_sampling_preserve_native_errors() {
        use std::cell::Cell;
        use std::rc::Rc;
        let chart = three_dimensional_chart().sampling_budget(SamplingBudget::new(4));
        let mut stage = Stage::new();
        let root = stage.add(chart.clone());
        let live = chart.bind(&stage, root).unwrap();
        stage.apply_affine(root, surface_affine());
        let calls = Rc::new(Cell::new(0));
        let sample_calls = calls.clone();
        let denied = live
            .get_parametric_curve(&stage, move |t| {
                sample_calls.set(sample_calls.get() + 1);
                [t, t * t, -t]
            })
            .unwrap()
            .build();
        assert!(matches!(
            denied,
            Err(GraphError::Sampling(crate::SamplingError::LimitExceeded {
                max_samples: 4,
                ..
            }))
        ));
        assert_eq!(calls.get(), 0);
        let frame = live.snapshot(&stage).unwrap();
        let spec = ParametricCurveSpec {
            t_range: [0.0, 1.0, 0.25],
            use_smoothing: false,
            sampling_budget: SamplingBudget::new(5),
            ..ParametricCurveSpec::default()
        };
        let mut sampled = Vec::new();
        let failed = frame.try_parametric_curve(
            |t| {
                sampled.push(t);
                if t == 0.5 {
                    Err("curve sample failed")
                } else {
                    Ok([t, t * t, -t])
                }
            },
            &spec,
        );
        assert!(matches!(
            failed,
            Err(GraphError::Callback("curve sample failed"))
        ));
        assert_eq!(sampled, vec![0.0, 0.25, 0.5]);
        let curve = frame
            .try_parametric_curve(|t| Ok::<_, ()>([t, t * t, -t]), &spec)
            .unwrap();
        near(
            *curve.points().last().unwrap(),
            surface_affine().apply_point(chart.c2p(&[1.0, 1.0, -1.0])),
        );
        assert!(matches!(
            frame.try_parametric_curve(|_| Ok::<_, ()>([1e40, 0.0, 0.0]), &spec),
            Err(GraphError::InvalidPoint { index: 0, .. })
        ));
    }

    #[test]
    fn live_fallible_surfaces_admit_before_callbacks_and_stop_at_failed_derivative_probes() {
        let chart = three_dimensional_chart();
        let mut stage = Stage::new();
        let root = stage.add(chart.clone());
        let live = chart.bind(&stage, root).unwrap();
        stage.apply_affine(root, surface_affine());
        let frame = live.snapshot(&stage).unwrap();
        let spec = SurfaceSpec {
            u_range: (-2.0, 1.0),
            v_range: (2.0, 3.0),
            resolution: (3, 4),
            ..SurfaceSpec::default()
        };
        let mut sampled = Vec::new();
        let denied = frame.try_parametric_surface(
            |u, v| {
                sampled.push((u, v));
                Ok::<_, &'static str>([u, v, u + v])
            },
            &spec,
            SamplingBudget::new(11),
        );
        assert!(matches!(denied, Err(SurfaceSampleError::Budget(_))));
        assert!(sampled.is_empty());
        let failed = frame.try_parametric_surface(
            |u, v| {
                sampled.push((u, v));
                if sampled.len() == 2 {
                    Err("u derivative failed")
                } else {
                    Ok([u, v, u + v])
                }
            },
            &spec,
            SamplingBudget::new(12),
        );
        assert!(matches!(
            failed,
            Err(SurfaceSampleError::Callback("u derivative failed"))
        ));
        assert_eq!(sampled, vec![(-2.0, 2.0), (-2.0 + spec.epsilon, 2.0)]);
        let surface = frame
            .try_parametric_surface(
                |u, v| Ok::<_, ()>([u, v, u + v]),
                &spec,
                SamplingBudget::new(12),
            )
            .unwrap();
        assert_eq!(surface.points().len(), 12);
        near(
            surface.points()[0],
            surface_affine().apply_point(chart.c2p(&[-2.0, 2.0, 0.0])),
        );
        assert!(matches!(
            frame.try_parametric_surface(
                |_, _| Ok::<_, ()>([1e40, 0.0, 0.0]),
                &spec,
                SamplingBudget::new(12),
            ),
            Err(SurfaceSampleError::InvalidSample {
                index: 0,
                probe: "point"
            })
        ));
    }

    #[test]
    fn live_tipped_axes_preserve_numeric_endpoints_before_and_after_affine_motion() {
        for tipped in [false, true] {
            let chart = axes(tipped);
            let mut stage = Stage::new();
            let root = stage.add(chart.clone());
            let live = chart.bind(&stage, root).unwrap();
            let affine = Placement::new(
                [[1.25, 0.7, -0.2], [-0.4, 0.8, 0.6], [0.3, -0.5, 1.1]],
                [3.0, -2.0, 1.5],
            );
            for coordinate in [[0.0, 0.0], [-2.0, 3.0], [4.0, -1.0], [7.0, 5.0]] {
                near(
                    live.c2p(&stage, &coordinate).unwrap(),
                    chart.c2p(&coordinate),
                );
            }
            stage.apply_affine(root, affine);
            // Binding after movement must use the same numeric calibration.
            let later_binding = chart.bind(&stage, root).unwrap();
            let frame = live.snapshot(&stage).unwrap();
            for x in -4..=5 {
                for y in -3..=4 {
                    let coordinate = [f64::from(x) * 0.7, f64::from(y) * 0.4];
                    let expected = affine.apply_point(chart.c2p(&coordinate));
                    let point = live.c2p(&stage, &coordinate).unwrap();
                    near(point, expected);
                    near(later_binding.c2p(&stage, &coordinate).unwrap(), expected);
                    near(frame.p2c(point), [coordinate[0], coordinate[1], 0.0]);
                }
            }
            // The inverse of an embedded 2D chart projects off-plane points.
            let normal = cross(
                sub(frame.c2p(&[1.0, 0.0]), frame.origin()),
                sub(frame.c2p(&[0.0, 1.0]), frame.origin()),
            );
            let point = add(frame.c2p(&[1.2, -0.7]), scale(normal, 4.0));
            near(live.p2c(&stage, point).unwrap(), [1.2, -0.7, 0.0]);
        }
    }

    #[test]
    fn live_extrapolated_origins_and_tipped_shafts_agree_when_ranges_exclude_zero() {
        let chart = Axes::new()
            .x_range([2.0, 7.0, 1.0])
            .y_range([-6.0, -2.0, 1.0])
            .width(8.0)
            .height(3.0)
            .axis_config(AxisConfig {
                include_tip: Some(true),
                ..AxisConfig::default()
            })
            .build(&FontBook::bundled().unwrap())
            .unwrap();
        let mut stage = Stage::new();
        let root = stage.add(chart.clone());
        let live = chart.bind(&stage, root).unwrap();
        let affine = Placement::new(
            [[0.3, -1.4, 0.0], [0.8, 0.2, 0.0], [0.5, 0.1, 1.0]],
            [-4.0, 1.2, 0.5],
        );
        stage.apply_affine(root, affine);
        for coordinate in [[0.0, 0.0], [2.0, -6.0], [7.0, -2.0], [-3.0, 4.0]] {
            let point = live.c2p(&stage, &coordinate).unwrap();
            near(point, affine.apply_point(chart.c2p(&coordinate)));
            near(
                live.p2c(&stage, point).unwrap(),
                [coordinate[0], coordinate[1], 0.0],
            );
        }
    }

    #[test]
    fn live_three_dimensional_frames_invert_nonorthogonal_affine_axes() {
        let chart = ThreeDAxes::new()
            .x_range([-2.0, 3.0, 1.0])
            .y_range([-1.0, 2.0, 1.0])
            .z_range([-3.0, 1.0, 1.0])
            .axis_config(AxisConfig {
                include_tip: Some(true),
                ..AxisConfig::default()
            })
            .build(&FontBook::bundled().unwrap())
            .unwrap();
        let mut stage = Stage::new();
        let root = stage.add(chart.clone());
        let live = chart.bind(&stage, root).unwrap();
        let affine = Placement::new(
            [[1.2, 0.7, -0.4], [-0.3, 0.8, 0.2], [0.6, -0.2, -1.5]],
            [2.0, -1.5, 0.7],
        );
        stage.apply_affine(root, affine);
        let frame = live.snapshot(&stage).unwrap();
        assert_eq!(frame.dimension(), 3);
        for coordinate in [[0.0, 0.0, 0.0], [1.2, -0.5, 0.7], [-3.0, 4.0, 2.0]] {
            let point = live.c2p(&stage, &coordinate).unwrap();
            near(point, affine.apply_point(chart.c2p(&coordinate)));
            near(frame.p2c(point), coordinate);
        }
    }

    #[test]
    fn live_plane_bindings_select_axes_after_grids_and_preserve_complex_labels() {
        let book = FontBook::bundled().unwrap();
        let plane = NumberPlane::new()
            .x_range([-2.0, 3.0, 1.0])
            .y_range([-1.0, 2.0, 1.0])
            .build(&book)
            .unwrap();
        let mut complex = ComplexPlane::new()
            .x_range([-2.0, 3.0, 1.0])
            .y_range([-1.0, 2.0, 1.0])
            .build(&book)
            .unwrap();
        complex
            .add_coordinate_labels_for(&[[1.0, 0.0], [0.0, 1.0]], &book)
            .unwrap();
        let mut stage = Stage::new();
        let plane_root = stage.add(plane.clone());
        let complex_root = stage.add(complex.clone());
        let plain_live = plane.bind(&stage, plane_root).unwrap();
        let complex_live = complex.bind(&stage, complex_root).unwrap();
        for (root, live) in [(plane_root, plain_live), (complex_root, complex_live)] {
            let children = stage.get(root).unwrap().submobjects();
            assert_eq!(live.axis(0), Some(children[2]));
            assert_eq!(live.axis(1), Some(children[3]));
            let before = live.c2p(&stage, &[1.2, -0.5]).unwrap();
            stage.shift(root, [-2.0, 1.0, 0.7]);
            near(
                live.c2p(&stage, &[1.2, -0.5]).unwrap(),
                add(before, [-2.0, 1.0, 0.7]),
            );
        }
        assert_eq!(stage.get(complex_root).unwrap().submobjects().len(), 5);
    }

    #[test]
    fn live_copy_rebinding_and_snapshots_keep_distinct_chart_frames() {
        let chart = axes(true);
        let mut stage = Stage::new();
        let root = stage.add(chart.clone());
        let live = chart.bind(&stage, root).unwrap();
        let frame = live.snapshot(&stage).unwrap();
        let coordinate = [1.5, -0.5];
        let original = frame.c2p(&coordinate);
        let native_binding = stage
            .add_dt_updater_with_state(
                root,
                live.clone(),
                LiveCoordinateSystem::remap_handles,
                |_, _, _, _| {},
                false,
            )
            .unwrap();
        let axis_binding = stage
            .add_dt_updater_with_state(
                live.axis(0).unwrap(),
                live.clone(),
                LiveCoordinateSystem::remap_handles,
                |_, _, _, _| {},
                false,
            )
            .unwrap();
        let copied_map = stage.copy_family_mapped(root).unwrap();
        let copied_root = copied_map.root();
        let copied = live.rebind(&stage, copied_root).unwrap();
        let mut remapped = live.clone();
        remapped.remap_handles(&copied_map);
        assert_eq!(remapped.root(), copied_root);
        assert_eq!(remapped.axis(0), copied.axis(0));
        assert_ne!(copied.axis(0), live.axis(0));
        stage.shift(copied_root, [4.0, 1.0, 0.0]);
        near(
            copied.c2p(&stage, &coordinate).unwrap(),
            add(original, [4.0, 1.0, 0.0]),
        );
        near(
            remapped.c2p(&stage, &coordinate).unwrap(),
            add(original, [4.0, 1.0, 0.0]),
        );
        let owned = stage
            .updater_state::<LiveCoordinateSystem>(copied_root, native_binding)
            .unwrap();
        assert_eq!(owned.root(), copied_root);
        near(
            owned.c2p(&stage, &coordinate).unwrap(),
            add(original, [4.0, 1.0, 0.0]),
        );
        near(live.c2p(&stage, &coordinate).unwrap(), original);
        // Copying one axis preserves its reference to the whole external chart.
        let copied_axis = stage.copy_family(live.axis(0).unwrap()).unwrap();
        let external = stage
            .updater_state::<LiveCoordinateSystem>(copied_axis, axis_binding)
            .unwrap();
        assert_eq!(external.root(), root);
        assert_eq!(external.axis(0), live.axis(0));
        near(external.c2p(&stage, &coordinate).unwrap(), original);
        let mut destination = Stage::new();
        let imported_root = stage.copy_into(copied_root, &mut destination).unwrap();
        let imported = copied.rebind(&destination, imported_root).unwrap();
        near(
            imported.c2p(&destination, &coordinate).unwrap(),
            copied.c2p(&stage, &coordinate).unwrap(),
        );
        assert!(matches!(
            copied.snapshot(&destination),
            Err(CoordsError::LiveFrame(CoordinateFrameError::StaleHandle(_)))
        ));
        stage.shift(root, [-3.0, 0.0, 0.0]);
        near(frame.c2p(&coordinate), original);
        stage.delete(root).unwrap();
        near(frame.c2p(&coordinate), original);
        near(
            copied.c2p(&stage, &coordinate).unwrap(),
            add(original, [4.0, 1.0, 0.0]),
        );
    }

    #[test]
    fn live_transform_samples_move_graphs_and_labels_in_the_current_frame() {
        let chart = axes(true);
        let mut stage = Stage::new();
        let root = stage.add(chart.clone());
        let live = chart.bind(&stage, root).unwrap();
        let frozen = live.snapshot(&stage).unwrap();
        let deferred_graph = frozen.get_graph(|x| 0.5 * x + 1.0);
        let target = stage.copy_family(root).unwrap();
        let delta = [3.0, -2.0, 0.5];
        stage.shift(target, delta);
        let mut animation = Transform::new(root, target).with_config(AnimConfig {
            rate_func: RateFunc::linear(),
            ..AnimConfig::default()
        });
        animation.begin(&mut stage).unwrap();
        let label = Text::new("P")
            .build(&FontBook::bundled().unwrap())
            .unwrap()
            .vmob;
        for alpha in [0.0, 0.25, 0.75, 1.0] {
            animation.interpolate(&mut stage, alpha);
            let frame = live.snapshot(&stage).unwrap();
            let graph = frame.get_graph(|x| 0.5 * x + 1.0).build().unwrap();
            near(
                *graph.points().first().unwrap(),
                add(frozen.c2p(&[-2.0, 0.0]), scale(delta, alpha)),
            );
            near(
                *graph.points().last().unwrap(),
                add(frozen.c2p(&[4.0, 3.0]), scale(delta, alpha)),
            );
            let point = frame.input_to_graph_point(1.0, &|x| 0.5 * x + 1.0);
            near(
                label.clone().moved_to(point).center_point(),
                add(frozen.c2p(&[1.0, 1.5]), scale(delta, alpha)),
            );
            near(frame.p2c(point), [1.0, 1.5, 0.0]);
        }
        animation.finish(&mut stage);
        let old_graph = deferred_graph.build().unwrap();
        near(
            *old_graph.points().first().unwrap(),
            frozen.c2p(&[-2.0, 0.0]),
        );
    }

    #[test]
    fn live_graphs_can_redraw_from_native_updaters_after_axis_motion() {
        let chart = axes(true);
        let mut stage = Stage::new();
        let root = stage.add(chart.clone());
        let live = chart.bind(&stage, root).unwrap();
        let graph_binding = live.clone();
        let graph = stage.always_redraw(move |stage| {
            let curve = graph_binding
                .get_graph(stage, |x| x + 1.0)
                .unwrap()
                .build()
                .unwrap();
            stage.add(curve)
        });
        stage.add_many_to_scene(&[root, graph]).unwrap();
        stage.shift(root, [2.0, 1.0, 0.0]);
        stage.update(0.1);
        // always_redraw keeps a stable empty container around its new content.
        let curve = stage.get(graph).unwrap().submobjects()[0];
        near(
            stage.get_start(curve).unwrap(),
            live.c2p(&stage, &[-2.0, -1.0]).unwrap(),
        );
        near(
            stage.get_end(curve).unwrap(),
            live.c2p(&stage, &[4.0, 5.0]).unwrap(),
        );
    }

    #[test]
    fn live_graph_snapshot_preserves_the_chart_sampling_resource_contract() {
        let chart = axes(false).sampling_budget(SamplingBudget::new(8));
        let mut stage = Stage::new();
        let root = stage.add(chart.clone());
        let frame = chart.bind(&stage, root).unwrap().snapshot(&stage).unwrap();
        assert_eq!(frame.graph_sampling_budget(), SamplingBudget::new(8));
        assert_eq!(frame.all_ranges(), chart.all_ranges());
        assert_eq!(frame.num_sampled_graph_points_per_tick(), 5.0);
        assert!(matches!(
            frame.get_graph(|x| x).build(),
            Err(crate::graphs::GraphError::Sampling(
                crate::graphs::SamplingError::LimitExceeded { max_samples: 8, .. }
            ))
        ));
        let book = FontBook::bundled().unwrap();
        let plane = NumberPlane::new()
            .build(&book)
            .unwrap()
            .sampling_budget(SamplingBudget::new(9));
        let plane_root = stage.add(plane.clone());
        assert_eq!(
            plane
                .bind(&stage, plane_root)
                .unwrap()
                .snapshot(&stage)
                .unwrap()
                .graph_sampling_budget(),
            SamplingBudget::new(9)
        );
        let complex = ComplexPlane::new()
            .build(&book)
            .unwrap()
            .sampling_budget(SamplingBudget::new(10));
        let complex_root = stage.add(complex.clone());
        assert_eq!(
            complex
                .bind(&stage, complex_root)
                .unwrap()
                .snapshot(&stage)
                .unwrap()
                .graph_sampling_budget(),
            SamplingBudget::new(10)
        );
        let three = ThreeDAxes::new()
            .build(&book)
            .unwrap()
            .sampling_budget(SamplingBudget::new(11));
        let three_root = stage.add(three.clone());
        assert_eq!(
            three
                .bind(&stage, three_root)
                .unwrap()
                .snapshot(&stage)
                .unwrap()
                .graph_sampling_budget(),
            SamplingBudget::new(11)
        );
    }

    #[test]
    fn live_invalid_frames_fail_with_typed_errors_before_geometry_access() {
        let chart = axes(false);
        let mut stage = Stage::new();
        let root = stage.add(chart.clone());
        let live = chart.bind(&stage, root).unwrap();
        let x = live.axis(0).unwrap();
        let y = live.axis(1).unwrap();
        let baseline = stage.snapshot();

        let mut changed = baseline.materialize();
        changed.replace_children(root, &[y, x]).unwrap();
        assert_eq!(
            live_error(live.snapshot(&changed)),
            CoordinateFrameError::AxisChanged { axis: 0 }
        );

        let mut changed = baseline.materialize();
        changed.delete(x).unwrap();
        assert_eq!(
            live_error(live.snapshot(&changed)),
            CoordinateFrameError::StaleHandle(x)
        );

        let mut changed = baseline.materialize();
        changed.shift(y, [0.5, 0.0, 0.0]);
        assert_eq!(
            live_error(live.snapshot(&changed)),
            CoordinateFrameError::DisconnectedOrigins { axis: 1 }
        );

        let mut changed = baseline.materialize();
        // Stage::scale clamps to MIN_SCALE_FACTOR; an explicitly singular
        // affine map exercises the collapsed-frame refusal itself.
        changed.apply_affine(root, Placement::new([[0.0; 3]; 3], [0.0; 3]));
        assert_eq!(
            live_error(live.snapshot(&changed)),
            CoordinateFrameError::DegenerateBasis
        );

        let mut changed = baseline.materialize();
        let schema =
            RecordSchema::new(&[("point", 1), ("joint_angle", 1)], &["point"], &["point"]).unwrap();
        changed.get_mut(x).unwrap().buffer = RecordBuffer::new(schema, 3).unwrap();
        assert!(matches!(
            live_error(live.snapshot(&changed)),
            CoordinateFrameError::InvalidAxis { axis: 0, .. }
        ));

        let mut changed = baseline.materialize();
        let mut points = changed.get_points(x).unwrap();
        points[1][1] += 0.5;
        changed.set_points(x, &points).unwrap();
        assert_eq!(
            live_error(live.snapshot(&changed)),
            CoordinateFrameError::InvalidAxis {
                axis: 0,
                reason: "axis shaft is no longer straight"
            }
        );

        let mut changed = baseline.materialize();
        changed
            .set_placement(x, Placement::from_translation([f64::NAN, 0.0, 0.0]))
            .unwrap();
        assert!(matches!(
            live_error(live.snapshot(&changed)),
            CoordinateFrameError::InvalidAxis { axis: 0, .. }
        ));
        assert!(matches!(
            live.c2p(&stage, &[f64::NAN, 1.0]),
            Err(CoordsError::LiveFrame(
                CoordinateFrameError::NonFiniteCoordinates
            ))
        ));
        assert!(matches!(
            live.p2c(&stage, [0.0, f64::INFINITY, 0.0]),
            Err(CoordsError::LiveFrame(
                CoordinateFrameError::NonFiniteCoordinates
            ))
        ));

        let empty = stage.add(Mobject::new());
        assert!(matches!(
            chart.bind(&stage, empty),
            Err(CoordsError::LiveFrame(
                CoordinateFrameError::MissingAxis { .. }
            ))
        ));
        assert!(matches!(
            Axes::new().bind(&stage, root),
            Err(CoordsError::LiveFrame(CoordinateFrameError::UnbuiltChart))
        ));
        assert!(matches!(
            ThreeDAxes::new().bind(&stage, root),
            Err(CoordsError::LiveFrame(CoordinateFrameError::UnbuiltChart))
        ));
        assert!(matches!(
            NumberPlane::new().bind(&stage, root),
            Err(CoordsError::LiveFrame(CoordinateFrameError::UnbuiltChart))
        ));
        assert!(matches!(
            ComplexPlane::new().bind(&stage, root),
            Err(CoordsError::LiveFrame(CoordinateFrameError::UnbuiltChart))
        ));
    }

    #[test]
    fn live_normalized_inverse_handles_extreme_nonuniform_scales_and_refuses_singular_axes() {
        let chart = axes(false);
        let mut stage = Stage::new();
        let root = stage.add(chart.clone());
        let live = chart.bind(&stage, root).unwrap();
        stage.apply_affine(
            root,
            Placement::new(
                [[1e120, 0.0, 0.0], [0.0, 1e-120, 0.0], [0.0, 0.0, 1.0]],
                [0.0; 3],
            ),
        );
        let point = live.c2p(&stage, &[0.7, -0.8]).unwrap();
        near(live.p2c(&stage, point).unwrap(), [0.7, -0.8, 0.0]);

        let mut singular = Stage::new();
        let root = singular.add(chart.clone());
        let live = chart.bind(&singular, root).unwrap();
        singular.apply_affine(
            root,
            Placement::new(
                [[1.0, 2.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
                [0.0; 3],
            ),
        );
        assert_eq!(
            live_error(live.snapshot(&singular)),
            CoordinateFrameError::DegenerateBasis
        );
    }
}
