//! The graphing family (§12.2): `ParametricCurve`, `FunctionGraph`, and
//! `ImplicitFunction` — the curves the Atlas's coordinate systems draw.
//!
//! Ported from `manimlib/mobject/functions.py` @ `6199a00d`. The sampling
//! semantics are the Reference's, exactly:
//!
//! * **Subpath layout.** Discontinuities inside `(t_min, t_max)` split the
//!   parameter range at `jump ∓ epsilon`; the sorted boundary list
//!   `[t_min, t_max, jumps ∓ ε…]` is walked in consecutive pairs, each pair
//!   producing one subpath of corner-joined samples.
//! * **Sampling.** Within a pair `(t1, t2)` the samples are
//!   `t1 + k·step` for every `k ≥ 0` with `t1 + k·step < t2` — numpy
//!   `arange` semantics, multiplication form, strict `<` — followed by an
//!   explicit final sample at `t2`.
//! * **Smoothing.** `use_smoothing` maps onto [`QuadPath::make_smooth`]:
//!   `approx = true` for `ParametricCurve` (the Reference's
//!   `make_smooth(approx=True)` — handle recomputation, point count
//!   unchanged), `approx = false` for `ImplicitFunction` (the Reference's
//!   bare `make_smooth()` — the true spline solve). No divergence.
//!
//! Two deliberate, documented divergences:
//!
//! * **Non-positive `step`.** numpy's `arange` raises on `step = 0` and
//!   returns empty for a step pointing away from `t2`; an unconditional
//!   `k·step < t2` loop would hang on the former. Here a finite
//!   non-positive step samples only the explicit `t2` endpoint of each
//!   pair. Non-finite work controls are typed [`SamplingError`]s.
//! * **Owned closures.** `t_func`/`function` are owned `dyn Fn` values, so
//!   the builders implement [`std::fmt::Debug`] by hand (configuration
//!   only) and do not derive `Clone`/`PartialEq` the way the closure-free
//!   builders do.
//!
//! The Reference's `bind_graph_to_func` updater is **not** ported: updater
//! wiring is Proscenium's (W9), and a built [`VMobject`] here is plain
//! detached geometry. The `get_graph` glue lives in
//! [`graph_parametric`], which the Axes-family builders (coords.rs /
//! planes.rs) call with their `c2p`.

use std::fmt;

use fmn_core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS, YELLOW};
use fmn_core::types::Vec3;
use fmn_geom::{GeomError, IsolineConfig, IsolineError, QuadPath, try_plot_isoline};
use fmn_mobject::uniforms::JointType;

use crate::style::Style;
use crate::vmobject::VMobject;

/// The Reference's `ParametricCurve(t_range=(0, 1, 0.1))`.
pub const DEFAULT_T_RANGE: [f64; 3] = [0.0, 1.0, 0.1];
/// The Reference's `ParametricCurve(epsilon=1e-8)`.
pub const DEFAULT_EPSILON: f64 = 1e-8;
/// The Reference's `FunctionGraph(x_range=(-8, 8, 0.25))`.
pub const DEFAULT_X_RANGE: [f64; 3] = [-8.0, 8.0, 0.25];
/// The Reference's `ImplicitFunction(min_depth=5)`.
pub const DEFAULT_MIN_DEPTH: u32 = 5;
/// The Reference's `ImplicitFunction(max_quads=1500)`.
pub const DEFAULT_MAX_QUADS: usize = 1500;
/// Default upper bound for values produced by one Atlas sampling
/// operation.
///
/// The bound is deliberately generous for ordinary scene geometry while
/// still making tiny steps and enormous finite ranges refuse before any
/// proportional allocation or iteration. Callers with a deliberate
/// larger corpus can opt into a different [`SamplingBudget`].
pub const DEFAULT_MAX_SAMPLES: usize = 65_536;

/// The explicit resource contract shared by Atlas range sampling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SamplingBudget {
    max_samples: usize,
}

impl SamplingBudget {
    /// The default Atlas sampling budget.
    pub const DEFAULT: Self = Self::new(DEFAULT_MAX_SAMPLES);

    /// A budget allowing at most `max_samples` produced values.
    #[must_use]
    pub const fn new(max_samples: usize) -> Self {
        Self { max_samples }
    }

    /// The maximum number of produced values.
    #[must_use]
    pub const fn max_samples(self) -> usize {
        self.max_samples
    }

    pub(crate) fn ensure_total(
        self,
        context: &'static str,
        samples: usize,
    ) -> Result<(), SamplingError> {
        if samples > self.max_samples {
            return Err(SamplingError::LimitExceeded {
                context,
                max_samples: self.max_samples,
            });
        }
        Ok(())
    }

    fn sampled_count(
        self,
        context: &'static str,
        start: f64,
        stop: f64,
        step: f64,
        include_stop: bool,
    ) -> Result<usize, SamplingError> {
        ensure_finite(context, "start", start)?;
        ensure_finite(context, "stop", stop)?;
        ensure_finite(context, "step", step)?;

        let terminal = usize::from(include_stop);
        self.ensure_total(context, terminal)?;
        if step <= 0.0 || start >= stop {
            return Ok(terminal);
        }

        let span = stop - start;
        let quotient = span / step;
        if !span.is_finite() || !quotient.is_finite() {
            return Err(SamplingError::LimitExceeded {
                context,
                max_samples: self.max_samples,
            });
        }

        let arange_count = quotient.ceil();
        let allowed = self.max_samples - terminal;
        if arange_count > allowed as f64 {
            return Err(SamplingError::LimitExceeded {
                context,
                max_samples: self.max_samples,
            });
        }
        let arange_count = arange_count as usize;
        arange_count
            .checked_add(terminal)
            .ok_or(SamplingError::CapacityOverflow { context })
    }
}

impl Default for SamplingBudget {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// A sampling request refused before proportional work begins.
#[derive(Debug, Clone, PartialEq)]
pub enum SamplingError {
    /// A range component that controls work is not finite.
    NonFinite {
        /// The sampling surface being built.
        context: &'static str,
        /// The rejected parameter.
        parameter: &'static str,
        /// The rejected value.
        value: f64,
    },
    /// The request would produce more values than its declared budget.
    LimitExceeded {
        /// The sampling surface being built.
        context: &'static str,
        /// The active limit.
        max_samples: usize,
    },
    /// Count arithmetic cannot be represented by the host.
    CapacityOverflow {
        /// The sampling surface being built.
        context: &'static str,
    },
    /// Reserving the validated bounded output failed.
    AllocationFailed {
        /// The sampling surface being built.
        context: &'static str,
        /// The validated number of values requested.
        samples: usize,
    },
}

impl fmt::Display for SamplingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite {
                context,
                parameter,
                value,
            } => write!(f, "{context} requires finite {parameter}, got {value}"),
            Self::LimitExceeded {
                context,
                max_samples,
            } => write!(
                f,
                "{context} exceeds the declared {max_samples}-sample budget"
            ),
            Self::CapacityOverflow { context } => {
                write!(f, "{context} sample count exceeds the host capacity")
            }
            Self::AllocationFailed { context, samples } => {
                write!(f, "{context} could not reserve {samples} sampled values")
            }
        }
    }
}

impl std::error::Error for SamplingError {}

fn ensure_finite(
    context: &'static str,
    parameter: &'static str,
    value: f64,
) -> Result<(), SamplingError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(SamplingError::NonFinite {
            context,
            parameter,
            value,
        })
    }
}

pub(crate) fn sampled_values(
    context: &'static str,
    start: f64,
    stop: f64,
    step: f64,
    include_stop: bool,
    budget: SamplingBudget,
) -> Result<Vec<f64>, SamplingError> {
    let count = budget.sampled_count(context, start, stop, step, include_stop)?;
    let terminal = usize::from(include_stop);
    let arange_count = count - terminal;
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| SamplingError::AllocationFailed {
            context,
            samples: count,
        })?;
    for k in 0..arange_count {
        values.push(k as f64 * step + start);
    }
    if include_stop {
        values.push(stop);
    }
    Ok(values)
}

/// A graphing failure: bounded sampling, isoline extraction, or the path
/// kernel's true spline solve.
#[derive(Debug, Clone, PartialEq)]
pub enum GraphError<E = std::convert::Infallible> {
    /// The original callback error. No later sample is evaluated.
    Callback(E),
    /// A sampled or smoothed point cannot be stored in finite native records.
    InvalidPoint {
        /// Zero-based sample index (or path-record index after smoothing).
        index: usize,
        /// The offending coordinates.
        point: Vec3,
    },
    /// Atlas sampling refused a non-finite or over-budget request.
    Sampling(SamplingError),
    /// [`fmn_geom::plot_isoline`] rejected the domain or depth.
    Isoline(IsolineError<E>),
    /// The path kernel rejected a construction (the true smoothing solve).
    Geom(GeomError),
}

impl<E: fmt::Display> fmt::Display for GraphError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Callback(error) => write!(f, "graph callback failed: {error}"),
            Self::InvalidPoint { index, point } => write!(
                f,
                "graph point {index} must be finite and f32-representable: {point:?}"
            ),
            Self::Sampling(e) => write!(f, "graph sampling failed: {e}"),
            Self::Isoline(e) => write!(f, "isoline extraction failed: {e}"),
            Self::Geom(e) => write!(f, "path construction failed: {e}"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for GraphError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Callback(error) => Some(error),
            Self::InvalidPoint { .. } => None,
            Self::Sampling(error) => Some(error),
            Self::Isoline(error) => Some(error),
            Self::Geom(error) => Some(error),
        }
    }
}

impl<E> From<SamplingError> for GraphError<E> {
    fn from(e: SamplingError) -> Self {
        Self::Sampling(e)
    }
}

impl<E> From<IsolineError<E>> for GraphError<E> {
    fn from(e: IsolineError<E>) -> Self {
        match e {
            IsolineError::Callback(error) => Self::Callback(error),
            other => Self::Isoline(other),
        }
    }
}

impl<E> From<GeomError> for GraphError<E> {
    fn from(e: GeomError) -> Self {
        Self::Geom(e)
    }
}

/// The Reference's pair sampling: `t1 + k·step` while `< t2`, then an
/// explicit `t2` — `[*np.arange(t1, t2, step), t2]`.
///
/// A finite non-positive step yields only `t2` (see the module docs).
fn sample_pair(
    t1: f64,
    t2: f64,
    step: f64,
    budget: SamplingBudget,
) -> Result<Vec<f64>, SamplingError> {
    sampled_values("parametric curve", t1, t2, step, true, budget)
}

/// `ParametricCurve(t_func, t_range, epsilon, discontinuities,
/// use_smoothing)`: a parametric path sampled per segment and joined as
/// corners, optionally approx-smoothed.
///
/// `t_func` is the owned counterpart of the Reference's callable;
/// [`ParametricCurve::point_from_function`] is its
/// `get_point_from_function`.
pub struct ParametricCurve {
    t_func: Box<dyn Fn(f64) -> Vec3>,
    t_range: [f64; 3],
    epsilon: f64,
    discontinuities: Vec<f64>,
    use_smoothing: bool,
    sampling_budget: SamplingBudget,
    style: Style,
}

impl fmt::Debug for ParametricCurve {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParametricCurve")
            .field("t_range", &self.t_range)
            .field("epsilon", &self.epsilon)
            .field("discontinuities", &self.discontinuities)
            .field("use_smoothing", &self.use_smoothing)
            .field("sampling_budget", &self.sampling_budget)
            .field("style", &self.style)
            .finish_non_exhaustive()
    }
}

impl ParametricCurve {
    /// A curve over `t_func` with the Reference's defaults:
    /// `t_range = (0, 1, 0.1)`, `epsilon = 1e-8`, no discontinuities,
    /// smoothing on.
    #[must_use]
    pub fn new(t_func: impl Fn(f64) -> Vec3 + 'static) -> Self {
        Self {
            t_func: Box::new(t_func),
            t_range: DEFAULT_T_RANGE,
            epsilon: DEFAULT_EPSILON,
            discontinuities: Vec::new(),
            use_smoothing: true,
            sampling_budget: SamplingBudget::default(),
            style: Style::default(),
        }
    }

    /// `(t_min, t_max, step)`.
    #[must_use]
    pub fn t_range(mut self, t_range: [f64; 3]) -> Self {
        self.t_range = t_range;
        self
    }

    /// Half the excluded width around each discontinuity.
    #[must_use]
    pub fn epsilon(mut self, epsilon: f64) -> Self {
        self.epsilon = epsilon;
        self
    }

    /// Parameter values where the function jumps; only those strictly
    /// inside `(t_min, t_max)` split the path.
    #[must_use]
    pub fn discontinuities(mut self, discontinuities: impl Into<Vec<f64>>) -> Self {
        self.discontinuities = discontinuities.into();
        self
    }

    /// Whether to approx-smooth the corner path (Reference default true).
    #[must_use]
    pub fn use_smoothing(mut self, use_smoothing: bool) -> Self {
        self.use_smoothing = use_smoothing;
        self
    }

    /// Bound the total parameter samples produced by [`Self::build`].
    #[must_use]
    pub fn sampling_budget(mut self, budget: SamplingBudget) -> Self {
        self.sampling_budget = budget;
        self
    }

    /// Set stroke and fill colour.
    #[must_use]
    pub fn color(mut self, color: fmn_core::color::Srgb) -> Self {
        self.style = self.style.color(color);
        self
    }

    /// Replace the style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// The parameter function (`get_t_func`).
    pub fn t_func(&self) -> &dyn Fn(f64) -> Vec3 {
        &*self.t_func
    }

    /// The configured `(t_min, t_max, step)`.
    #[must_use]
    pub fn t_range_value(&self) -> [f64; 3] {
        self.t_range
    }

    /// The configured discontinuities.
    #[must_use]
    pub fn discontinuities_list(&self) -> &[f64] {
        &self.discontinuities
    }

    /// `get_point_from_function`.
    #[must_use]
    pub fn point_from_function(&self, t: f64) -> Vec3 {
        (self.t_func)(t)
    }

    /// `init_points`: sample each continuity segment as corners.
    ///
    /// Delegates to [`ParametricCurveSpec::try_sample`] with the owned function.
    /// The same admitted ranges, point validation and path kernel serve both
    /// infallible and fallible callers; no sentinel geometry is constructed.
    ///
    /// # Errors
    /// [`GraphError::Sampling`] if the range controls are non-finite or
    /// the requested boundary/sample work exceeds the configured budget;
    /// [`GraphError::InvalidPoint`] for unrepresentable samples or handles.
    pub fn build(self) -> Result<VMobject, GraphError> {
        ParametricCurveSpec {
            t_range: self.t_range,
            epsilon: self.epsilon,
            discontinuities: self.discontinuities,
            use_smoothing: self.use_smoothing,
            sampling_budget: self.sampling_budget,
            style: self.style,
        }
        .try_sample(|t| Ok((self.t_func)(t)))
    }
}

/// Sampling controls independent of callback storage. Both owned graph builders
/// and borrowed, fallible host callbacks use this one path-construction owner.
#[derive(Debug, Clone, PartialEq)]
pub struct ParametricCurveSpec {
    /// Start, stop and increment, including the explicit stop sample.
    pub t_range: [f64; 3],
    /// Half the excluded width around each discontinuity.
    pub epsilon: f64,
    /// Interior discontinuities, sorted together with the domain endpoints.
    pub discontinuities: Vec<f64>,
    /// Apply the existing approximate smoothing after successful sampling.
    pub use_smoothing: bool,
    /// Aggregate boundary and sample admission budget.
    pub sampling_budget: SamplingBudget,
    /// Style of the completed path.
    pub style: Style,
}

impl Default for ParametricCurveSpec {
    fn default() -> Self {
        Self {
            t_range: DEFAULT_T_RANGE,
            epsilon: DEFAULT_EPSILON,
            discontinuities: Vec::new(),
            use_smoothing: true,
            sampling_budget: SamplingBudget::DEFAULT,
            style: Style::default(),
        }
    }
}

fn finite_graph_point<E>(point: Vec3, index: usize) -> Result<Vec3, GraphError<E>> {
    if point
        .iter()
        .all(|value| value.is_finite() && value.abs() <= f64::from(f32::MAX))
    {
        Ok(point)
    } else {
        Err(GraphError::InvalidPoint { index, point })
    }
}

impl ParametricCurveSpec {
    /// Construct a detached curve from a borrowed, stateful, fallible callback.
    /// Sampling order and smoothing are identical to [`ParametricCurve::build`].
    /// No partial path is returned, and no sample follows the first callback error.
    ///
    /// # Errors
    /// Admission, invalid point, path-construction, or original callback error.
    pub fn try_sample<E>(
        &self,
        mut function: impl FnMut(f64) -> Result<Vec3, E>,
    ) -> Result<VMobject, GraphError<E>> {
        let [t_min, t_max, step] = self.t_range;
        ensure_finite("parametric curve", "t_min", t_min)?;
        ensure_finite("parametric curve", "t_max", t_max)?;
        ensure_finite("parametric curve", "step", step)?;
        ensure_finite("parametric curve", "epsilon", self.epsilon)?;

        self.sampling_budget.ensure_total(
            "parametric-curve discontinuities",
            self.discontinuities.len(),
        )?;
        let mut in_range_discontinuities = 0usize;
        for &jump in &self.discontinuities {
            ensure_finite("parametric curve", "discontinuity", jump)?;
            if jump > t_min && jump < t_max {
                let before = jump - self.epsilon;
                let after = jump + self.epsilon;
                ensure_finite("parametric curve", "discontinuity minus epsilon", before)?;
                ensure_finite("parametric curve", "discontinuity plus epsilon", after)?;
                in_range_discontinuities = in_range_discontinuities.checked_add(1).ok_or(
                    SamplingError::CapacityOverflow {
                        context: "parametric curve boundaries",
                    },
                )?;
            }
        }
        let boundary_count = in_range_discontinuities
            .checked_mul(2)
            .and_then(|count| count.checked_add(2))
            .ok_or(SamplingError::CapacityOverflow {
                context: "parametric curve boundaries",
            })?;
        self.sampling_budget
            .ensure_total("parametric curve boundaries", boundary_count)?;

        let mut boundary_times = Vec::new();
        boundary_times
            .try_reserve_exact(boundary_count)
            .map_err(|_| SamplingError::AllocationFailed {
                context: "parametric curve boundaries",
                samples: boundary_count,
            })?;
        boundary_times.extend([t_min, t_max]);
        for &jump in &self.discontinuities {
            if jump > t_min && jump < t_max {
                let before = jump - self.epsilon;
                let after = jump + self.epsilon;
                boundary_times.push(before);
                boundary_times.push(after);
            }
        }
        boundary_times.sort_by(f64::total_cmp);

        let mut total_samples = 0usize;
        for pair in boundary_times.windows(2).step_by(2) {
            let count = self.sampling_budget.sampled_count(
                "parametric curve",
                pair[0],
                pair[1],
                step,
                true,
            )?;
            total_samples =
                total_samples
                    .checked_add(count)
                    .ok_or(SamplingError::CapacityOverflow {
                        context: "parametric curve",
                    })?;
            self.sampling_budget
                .ensure_total("parametric curve", total_samples)?;
        }

        let mut path = QuadPath::new();
        let mut sample_index = 0;
        for pair in boundary_times.windows(2).step_by(2) {
            let (t1, t2) = (pair[0], pair[1]);
            let ts = sample_pair(t1, t2, step, self.sampling_budget)?;
            let mut points = Vec::new();
            points
                .try_reserve_exact(ts.len())
                .map_err(|_| SamplingError::AllocationFailed {
                    context: "parametric curve points",
                    samples: ts.len(),
                })?;
            for t in ts {
                let point = function(t).map_err(GraphError::Callback)?;
                points.push(finite_graph_point(point, sample_index)?);
                sample_index += 1;
            }
            path.start_new_path(points[0]);
            path.add_points_as_corners(&points[1..])?;
        }
        if self.use_smoothing {
            path.make_smooth(true)?;
        }
        if !path.has_points() {
            let point = function(t_min).map_err(GraphError::Callback)?;
            path.set_points(vec![finite_graph_point(point, sample_index)?])?;
        }
        for (index, &point) in path.points().iter().enumerate() {
            finite_graph_point::<E>(point, index)?;
        }
        Ok(VMobject::from_path(&path).with_style(self.style))
    }
}

/// `FunctionGraph(function, x_range=(-8, 8, 0.25), color=YELLOW)`: the
/// graph of `y = f(x)` in plain scene coordinates — a [`ParametricCurve`]
/// of `t ↦ (t, f(t), 0)`, no axes required.
pub struct FunctionGraph {
    function: Box<dyn Fn(f64) -> f64>,
    x_range: [f64; 3],
    epsilon: f64,
    discontinuities: Vec<f64>,
    use_smoothing: bool,
    sampling_budget: SamplingBudget,
    style: Style,
}

impl fmt::Debug for FunctionGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FunctionGraph")
            .field("x_range", &self.x_range)
            .field("epsilon", &self.epsilon)
            .field("discontinuities", &self.discontinuities)
            .field("use_smoothing", &self.use_smoothing)
            .field("sampling_budget", &self.sampling_budget)
            .field("style", &self.style)
            .finish_non_exhaustive()
    }
}

impl FunctionGraph {
    /// The graph of `function` over the Reference's default x-range,
    /// stroked `YELLOW`.
    #[must_use]
    pub fn new(function: impl Fn(f64) -> f64 + 'static) -> Self {
        Self {
            function: Box::new(function),
            x_range: DEFAULT_X_RANGE,
            epsilon: DEFAULT_EPSILON,
            discontinuities: Vec::new(),
            use_smoothing: true,
            sampling_budget: SamplingBudget::default(),
            style: Style::default().color(YELLOW),
        }
    }

    /// `(x_min, x_max, step)` — for a bare graph the third component is
    /// the sample step itself (the Reference passes `x_range` straight
    /// through as the parametric `t_range`).
    #[must_use]
    pub fn x_range(mut self, x_range: [f64; 3]) -> Self {
        self.x_range = x_range;
        self
    }

    /// Half the excluded width around each discontinuity.
    #[must_use]
    pub fn epsilon(mut self, epsilon: f64) -> Self {
        self.epsilon = epsilon;
        self
    }

    /// X values where the function jumps.
    #[must_use]
    pub fn discontinuities(mut self, discontinuities: impl Into<Vec<f64>>) -> Self {
        self.discontinuities = discontinuities.into();
        self
    }

    /// Whether to approx-smooth the corner path (Reference default true).
    #[must_use]
    pub fn use_smoothing(mut self, use_smoothing: bool) -> Self {
        self.use_smoothing = use_smoothing;
        self
    }

    /// Bound the parameter samples produced by [`Self::build`].
    #[must_use]
    pub fn sampling_budget(mut self, budget: SamplingBudget) -> Self {
        self.sampling_budget = budget;
        self
    }

    /// Set stroke and fill colour (Reference default `YELLOW`).
    #[must_use]
    pub fn color(mut self, color: fmn_core::color::Srgb) -> Self {
        self.style = self.style.color(color);
        self
    }

    /// Replace the style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// The graphed function — the Reference's `underlying_function`,
    /// surfaced by its `get_function`.
    pub fn underlying_function(&self) -> &dyn Fn(f64) -> f64 {
        &*self.function
    }

    /// The configured `(x_min, x_max, step)` (`get_x_range`).
    #[must_use]
    pub fn x_range_value(&self) -> [f64; 3] {
        self.x_range
    }

    /// Build the parametric curve of `t ↦ (t, f(t), 0)`.
    pub fn build(self) -> Result<VMobject, GraphError> {
        let function = self.function;
        ParametricCurve::new(move |t| [t, function(t), 0.0])
            .t_range(self.x_range)
            .epsilon(self.epsilon)
            .discontinuities(self.discontinuities)
            .use_smoothing(self.use_smoothing)
            .sampling_budget(self.sampling_budget)
            .style(self.style)
            .build()
    }
}

/// `ImplicitFunction(func, x_range, y_range, min_depth, max_quads,
/// use_smoothing, joint_type)`: the level set `func(x, y) = 0` over a
/// rectangle, extracted by [`fmn_geom::plot_isoline`] and drawn as
/// corner-joined subpaths, one per returned curve.
///
/// The Reference's defaults: the frame rectangle
/// `(-FRAME_X_RADIUS..FRAME_X_RADIUS) × (-FRAME_Y_RADIUS..FRAME_Y_RADIUS)`,
/// `min_depth = 5`, `max_quads = 1500`, no smoothing, `no_joint`. Empty
/// curves are dropped, as the Reference's `if curve != []` filter does.
pub struct ImplicitFunction {
    func: Box<dyn Fn(f64, f64) -> f64>,
    x_range: [f64; 2],
    y_range: [f64; 2],
    min_depth: u32,
    max_quads: usize,
    use_smoothing: bool,
    joint_type: JointType,
    style: Style,
}

impl fmt::Debug for ImplicitFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImplicitFunction")
            .field("x_range", &self.x_range)
            .field("y_range", &self.y_range)
            .field("min_depth", &self.min_depth)
            .field("max_quads", &self.max_quads)
            .field("use_smoothing", &self.use_smoothing)
            .field("joint_type", &self.joint_type)
            .field("style", &self.style)
            .finish_non_exhaustive()
    }
}

impl ImplicitFunction {
    /// The zero set of `func` over the frame rectangle, with the
    /// Reference's extraction budget.
    #[must_use]
    pub fn new(func: impl Fn(f64, f64) -> f64 + 'static) -> Self {
        Self {
            func: Box::new(func),
            x_range: [-FRAME_X_RADIUS, FRAME_X_RADIUS],
            y_range: [-FRAME_Y_RADIUS, FRAME_Y_RADIUS],
            min_depth: DEFAULT_MIN_DEPTH,
            max_quads: DEFAULT_MAX_QUADS,
            use_smoothing: false,
            joint_type: JointType::NoJoint,
            style: Style::default(),
        }
    }

    /// `(x_min, x_max)` of the extraction rectangle.
    #[must_use]
    pub fn x_range(mut self, x_range: [f64; 2]) -> Self {
        self.x_range = x_range;
        self
    }

    /// `(y_min, y_max)` of the extraction rectangle.
    #[must_use]
    pub fn y_range(mut self, y_range: [f64; 2]) -> Self {
        self.y_range = y_range;
        self
    }

    /// Minimum quadtree subdivision depth (takes precedence over
    /// `max_quads`, exactly the Reference's rule).
    #[must_use]
    pub fn min_depth(mut self, min_depth: u32) -> Self {
        self.min_depth = min_depth;
        self
    }

    /// The leaf budget.
    #[must_use]
    pub fn max_quads(mut self, max_quads: usize) -> Self {
        self.max_quads = max_quads;
        self
    }

    /// Whether to run the true spline solve over the extracted corners
    /// (Reference default false).
    #[must_use]
    pub fn use_smoothing(mut self, use_smoothing: bool) -> Self {
        self.use_smoothing = use_smoothing;
        self
    }

    /// The stroke joint uniform (Reference default `no_joint`).
    #[must_use]
    pub fn joint_type(mut self, joint_type: JointType) -> Self {
        self.joint_type = joint_type;
        self
    }

    /// Set stroke and fill colour.
    #[must_use]
    pub fn color(mut self, color: fmn_core::color::Srgb) -> Self {
        self.style = self.style.color(color);
        self
    }

    /// Replace the style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// The scalar field whose zero set is drawn.
    pub fn func(&self) -> &dyn Fn(f64, f64) -> f64 {
        &*self.func
    }

    /// Extract the level set and lay it out as corner subpaths.
    ///
    /// # Errors
    /// [`GraphError::Isoline`] when the rectangle is empty or inverted or
    /// the depth overflows the budget arithmetic; [`GraphError::Geom`]
    /// when `use_smoothing` runs the true spline solve and it fails.
    pub fn build(self) -> Result<VMobject, GraphError> {
        ImplicitFunctionSpec {
            x_range: self.x_range,
            y_range: self.y_range,
            config: IsolineConfig {
                min_depth: self.min_depth,
                max_quads: self.max_quads,
                ..IsolineConfig::default()
            },
            use_smoothing: self.use_smoothing,
            joint_type: self.joint_type,
            style: self.style,
        }
        .try_sample(|x, y| Ok((self.func)(x, y)))
    }
}

/// Isoline controls independent of callback ownership. Native and host callers
/// share Chisel's fallible contourer and Atlas's one path materializer.
#[derive(Debug, Clone, PartialEq)]
pub struct ImplicitFunctionSpec {
    /// Horizontal domain bounds.
    pub x_range: [f64; 2],
    /// Vertical domain bounds.
    pub y_range: [f64; 2],
    /// Depth, leaf budget, level and tolerance for native contour extraction.
    pub config: IsolineConfig,
    /// Apply the existing true-spline smoothing to the extracted contours.
    pub use_smoothing: bool,
    /// Stroke joint policy.
    pub joint_type: JointType,
    /// Style of the completed path.
    pub style: Style,
}

impl Default for ImplicitFunctionSpec {
    fn default() -> Self {
        Self {
            x_range: [-FRAME_X_RADIUS, FRAME_X_RADIUS],
            y_range: [-FRAME_Y_RADIUS, FRAME_Y_RADIUS],
            config: IsolineConfig::default(),
            use_smoothing: false,
            joint_type: JointType::NoJoint,
            style: Style::default(),
        }
    }
}

impl ImplicitFunctionSpec {
    /// Extract and materialize a fallible field. NaN/infinity values retain the
    /// native undefined-region meaning; an `Err` instead aborts immediately.
    ///
    /// # Errors
    /// Native isoline admission, the original field error, invalid record points,
    /// or the true-spline solve. No partial mobject is returned on any failure.
    pub fn try_sample<E>(
        &self,
        function: impl FnMut(f64, f64) -> Result<f64, E>,
    ) -> Result<VMobject, GraphError<E>> {
        let curves = try_plot_isoline(
            function,
            [self.x_range[0], self.y_range[0]],
            [self.x_range[1], self.y_range[1]],
            &self.config,
        )?;
        let mut path = QuadPath::new();
        for curve in &curves {
            if curve.is_empty() {
                continue;
            }
            let first = curve[0];
            path.start_new_path([first[0], first[1], 0.0]);
            let corners: Vec<Vec3> = curve[1..].iter().map(|p| [p[0], p[1], 0.0]).collect();
            let _ = path.add_points_as_corners(&corners);
        }
        if self.use_smoothing {
            path.make_smooth(false)?;
        }
        for (index, &point) in path.points().iter().enumerate() {
            finite_graph_point::<E>(point, index)?;
        }
        Ok(VMobject::from_path(&path)
            .with_style(self.style)
            .with_joint_type(self.joint_type))
    }
}

/// The `get_graph` glue for the Axes family (coords.rs / planes.rs):
/// `CoordinateSystem.get_graph` without the coordinate system.
///
/// Ports the Reference exactly: the x-range's third component is the
/// *tick* step, so the parametric sample step is
/// `x_range[2] / num_sampled_per_tick`, and the curve is
/// `t ↦ c2p(t, function(t))`. The intended wiring is a one-line method on
/// each concrete axes struct:
///
/// ```text
/// fn get_graph(&self, f: impl Fn(f64) -> f64 + 'static) -> ParametricCurve {
///     graph_parametric(f, |cs| self.c2p(cs), self.x_range,
///                      self.num_sampled_graph_points_per_tick())
/// }
/// ```
///
/// The Reference also stashes `underlying_function`/`x_range` on the graph
/// and offers `bind=True` updater wiring; the binding half is W9's and is
/// deliberately not here.
#[must_use]
pub fn graph_parametric(
    function: impl Fn(f64) -> f64 + 'static,
    c2p: impl Fn(&[f64]) -> Vec3 + 'static,
    x_range: [f64; 3],
    num_sampled_per_tick: f64,
    sampling_budget: SamplingBudget,
) -> ParametricCurve {
    let step = x_range[2] / num_sampled_per_tick;
    ParametricCurve::new(move |t| c2p(&[t, function(t)]))
        .t_range([x_range[0], x_range[1], step])
        .sampling_budget(sampling_budget)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_core::constants::PI;
    use fmn_geom::plot_isoline_with_stats;
    use std::cell::Cell;
    use std::rc::Rc;

    const EPS: f64 = DEFAULT_EPSILON;

    fn built_path(vmob: &VMobject) -> QuadPath {
        vmob.path().expect("builders emit the shared-anchor layout")
    }

    fn anchors_of(path: &QuadPath) -> Vec<Vec3> {
        path.anchors()
    }

    // ---- ParametricCurve sampling fixtures ----------------------------

    #[test]
    fn t_range_sampling_is_exact() {
        // (0, 1, 0.1): arange gives k = 0..=9 (10·0.1 = 1.0 is not < 1),
        // plus the explicit t2 — 11 samples, one subpath.
        let vmob = ParametricCurve::new(|t| [t, 2.0 * t, 0.0])
            .use_smoothing(false)
            .build()
            .expect("default sampling is bounded");
        let path = built_path(&vmob);
        let subpaths = path.subpaths();
        assert_eq!(subpaths.len(), 1);
        let anchors = anchors_of(&path);
        assert_eq!(anchors.len(), 11);
        assert_eq!(path.num_points(), 2 * 11 - 1);
        assert_eq!(anchors[0], [0.0, 0.0, 0.0]);
        // The last sample is the explicit t2, not an arange value.
        assert_eq!(anchors[10], [1.0, 2.0, 0.0]);
        // Multiplication-form arange: sample k is exactly t1 + k·step.
        for (k, a) in anchors.iter().enumerate().take(10) {
            let t = k as f64 * 0.1;
            assert_eq!(*a, [t, 2.0 * t, 0.0]);
        }
    }

    #[test]
    fn last_point_is_t_func_at_t_max() {
        let vmob = ParametricCurve::new(|t| [t, t * t, 1.0])
            .t_range([0.25, 0.75, 0.1])
            .use_smoothing(false)
            .build()
            .expect("fixture sampling is bounded");
        let anchors = anchors_of(&built_path(&vmob));
        assert_eq!(anchors.last(), Some(&[0.75, 0.5625, 1.0]));
        assert_eq!(anchors[0], [0.25, 0.0625, 1.0]);
    }

    #[test]
    fn discontinuities_outside_the_range_are_ignored() {
        let vmob = ParametricCurve::new(|t| [t, 0.0, 0.0])
            .t_range([-3.0, 3.0, 0.1])
            .discontinuities(vec![-5.0, -3.0, 3.0, 10.0])
            .use_smoothing(false)
            .build()
            .expect("fixture sampling is bounded");
        assert_eq!(built_path(&vmob).subpaths().len(), 1);
    }

    #[test]
    fn sampling_budget_is_exact_and_checked_before_function_calls() {
        // Four arange values plus the explicit endpoint fit exactly.
        let calls = Rc::new(Cell::new(0));
        let observed = Rc::clone(&calls);
        let vmob = ParametricCurve::new(move |t| {
            observed.set(observed.get() + 1);
            [t, 0.0, 0.0]
        })
        .t_range([0.0, 1.0, 0.25])
        .use_smoothing(false)
        .sampling_budget(SamplingBudget::new(5))
        .build()
        .expect("five samples fit exactly");
        assert_eq!(built_path(&vmob).anchors().len(), 5);
        assert_eq!(calls.get(), 5);

        let refused_calls = Rc::new(Cell::new(0));
        let observed = Rc::clone(&refused_calls);
        let error = ParametricCurve::new(move |t| {
            observed.set(observed.get() + 1);
            [t, 0.0, 0.0]
        })
        .t_range([0.0, 1.0, 0.25])
        .sampling_budget(SamplingBudget::new(4))
        .build()
        .expect_err("five samples exceed a four-sample budget");
        assert!(matches!(
            error,
            GraphError::Sampling(SamplingError::LimitExceeded {
                context: "parametric curve",
                max_samples: 4
            })
        ));
        assert_eq!(
            refused_calls.get(),
            0,
            "the parameter function must not run before budget validation"
        );
    }

    #[test]
    fn sampling_rejects_non_finite_and_tiny_step_inputs() {
        for (range, parameter) in [
            ([f64::NAN, 1.0, 0.1], "t_min"),
            ([0.0, f64::INFINITY, 0.1], "t_max"),
            ([0.0, 1.0, f64::NAN], "step"),
        ] {
            let error = ParametricCurve::new(|t| [t, 0.0, 0.0])
                .t_range(range)
                .build()
                .expect_err("non-finite work controls must be refused");
            assert!(matches!(
                error,
                GraphError::Sampling(SamplingError::NonFinite {
                    parameter: got,
                    ..
                }) if got == parameter
            ));
        }

        let error = ParametricCurve::new(|t| [t, 0.0, 0.0])
            .t_range([0.0, 1.0, f64::MIN_POSITIVE])
            .sampling_budget(SamplingBudget::new(32))
            .build()
            .expect_err("a tiny finite step must be refused before iteration");
        assert!(matches!(
            error,
            GraphError::Sampling(SamplingError::LimitExceeded {
                max_samples: 32,
                ..
            })
        ));
    }

    #[test]
    fn discontinuity_work_is_bounded_before_expansion() {
        let error = ParametricCurve::new(|t| [t, 0.0, 0.0])
            .discontinuities(vec![-10.0; 5])
            .sampling_budget(SamplingBudget::new(4))
            .build()
            .expect_err("even ignored discontinuities are bounded input work");
        assert!(matches!(
            error,
            GraphError::Sampling(SamplingError::LimitExceeded {
                context: "parametric-curve discontinuities",
                max_samples: 4
            })
        ));
    }

    #[test]
    fn discontinuity_boundaries_are_preflighted_before_evaluation() {
        let calls = Rc::new(Cell::new(0));
        let observed = Rc::clone(&calls);
        let error = ParametricCurve::new(move |t| {
            observed.set(observed.get() + 1);
            [t, 0.0, 0.0]
        })
        .t_range([0.0, 1.0, 1.0])
        .discontinuities(vec![0.5])
        .sampling_budget(SamplingBudget::new(3))
        .build()
        .expect_err("four expanded boundaries exceed a three-sample budget");
        assert!(matches!(
            error,
            GraphError::Sampling(SamplingError::LimitExceeded {
                context: "parametric curve boundaries",
                max_samples: 3
            })
        ));
        assert_eq!(
            calls.get(),
            0,
            "the parameter function must not run before boundary validation"
        );
    }

    // ---- Discontinuity fixtures (the bead's headline) -----------------

    #[test]
    fn reciprocal_breaks_at_zero() {
        // 1/x on (-3, 3) with the jump declared at 0: two subpaths, the
        // gap straddling the singularity at ±epsilon.
        let vmob = ParametricCurve::new(|t| [t, 1.0 / t, 0.0])
            .t_range([-3.0, 3.0, 0.1])
            .discontinuities(vec![0.0])
            .use_smoothing(false)
            .build()
            .expect("fixture sampling is bounded");
        let path = built_path(&vmob);
        let subpaths = path.subpaths();
        assert_eq!(subpaths.len(), 2);

        let left: Vec<Vec3> = subpaths[0].iter().step_by(2).copied().collect();
        let right: Vec<Vec3> = subpaths[1].iter().step_by(2).copied().collect();
        // arange(-3, -ε, 0.1): k = 0..=29 (k·0.1 − 3 = 0.0 is not < −ε),
        // plus the explicit −ε → 31 anchors per side.
        assert_eq!(left.len(), 31);
        assert_eq!(right.len(), 31);

        // No sample lands in the excluded (−ε, +ε) band…
        for a in left.iter().chain(right.iter()) {
            assert!(a[0].abs() >= EPS, "sample inside the jump band: {a:?}");
        }
        // …and the two subpaths' facing ends sit exactly at ±ε, so the gap
        // straddles 0.
        assert_eq!(left.last().expect("nonempty")[0], -EPS);
        assert_eq!(right[0][0], EPS);
        assert!(left.iter().all(|a| a[0] < 0.0));
        assert!(right.iter().all(|a| a[0] > 0.0));
        // The left end approaches the pole from below (1/x → −∞).
        assert!(left.last().expect("nonempty")[1] < 0.0);
        assert!(right[0][1] > 0.0);
    }

    #[test]
    fn tangent_breaks_at_pi_over_2() {
        let vmob = ParametricCurve::new(|t| [t, t.tan(), 0.0])
            .t_range([1.0, 2.0, 0.05])
            .discontinuities(vec![PI / 2.0])
            .use_smoothing(false)
            .build()
            .expect("fixture sampling is bounded");
        let path = built_path(&vmob);
        let subpaths = path.subpaths();
        assert_eq!(subpaths.len(), 2);
        let left: Vec<Vec3> = subpaths[0].iter().step_by(2).copied().collect();
        let right: Vec<Vec3> = subpaths[1].iter().step_by(2).copied().collect();
        assert_eq!(left.last().expect("nonempty")[0], PI / 2.0 - EPS);
        assert_eq!(right[0][0], PI / 2.0 + EPS);
        // tan blows up +∞ on the left of π/2 and −∞ on the right.
        assert!(left.last().expect("nonempty")[1] > 0.0);
        assert!(right[0][1] < 0.0);
    }

    #[test]
    fn step_function_breaks_at_the_jump() {
        let vmob = ParametricCurve::new(|t| [t, if t < 0.5 { 0.0 } else { 1.0 }, 0.0])
            .t_range([0.0, 1.0, 0.1])
            .discontinuities(vec![0.5])
            .use_smoothing(false)
            .build()
            .expect("fixture sampling is bounded");
        let path = built_path(&vmob);
        let subpaths = path.subpaths();
        assert_eq!(subpaths.len(), 2);
        let left: Vec<Vec3> = subpaths[0].iter().step_by(2).copied().collect();
        let right: Vec<Vec3> = subpaths[1].iter().step_by(2).copied().collect();
        assert!(left.iter().all(|a| a[1] == 0.0));
        assert!(right.iter().all(|a| a[1] == 1.0));
        assert_eq!(left.last().expect("nonempty")[0], 0.5 - EPS);
        assert_eq!(right[0][0], 0.5 + EPS);
    }

    // ---- Smoothing ----------------------------------------------------

    #[test]
    fn no_smoothing_keeps_corner_handles() {
        let t_func = |t: f64| [t, t * t, 0.0];
        let plain = ParametricCurve::new(t_func)
            .t_range([0.0, 1.0, 0.2])
            .use_smoothing(false)
            .build()
            .expect("fixture sampling is bounded");
        let smoothed = ParametricCurve::new(t_func)
            .t_range([0.0, 1.0, 0.2])
            .use_smoothing(true)
            .build()
            .expect("fixture sampling is bounded");
        let plain_path = built_path(&plain);
        let smoothed_path = built_path(&smoothed);
        // Approx smoothing keeps the point count; only the handles move.
        assert_eq!(plain_path.num_points(), smoothed_path.num_points());

        // Unsmoothed: every handle is the exact midpoint of its anchors.
        let points = plain_path.points();
        for w in points.windows(3).step_by(2) {
            let mid = [
                0.5 * (w[0][0] + w[2][0]),
                0.5 * (w[0][1] + w[2][1]),
                0.5 * (w[0][2] + w[2][2]),
            ];
            assert_eq!(w[1], mid);
        }
        // Smoothed: at least one handle leaves the midpoint (a parabola is
        // not a polyline).
        let sp = smoothed_path.points();
        let any_off_midpoint = sp.windows(3).step_by(2).any(|w| {
            let mid = [
                0.5 * (w[0][0] + w[2][0]),
                0.5 * (w[0][1] + w[2][1]),
                0.5 * (w[0][2] + w[2][2]),
            ];
            w[1] != mid
        });
        assert!(any_off_midpoint);
        // Anchors are untouched by approx smoothing.
        assert_eq!(plain_path.anchors(), smoothed_path.anchors());
    }

    // ---- FunctionGraph -------------------------------------------------

    #[test]
    fn function_graph_defaults_match_the_reference() {
        let graph = FunctionGraph::new(|x| x * x);
        assert_eq!(graph.x_range_value(), [-8.0, 8.0, 0.25]);
        assert_eq!(graph.underlying_function()(3.0), 9.0);
        let vmob = graph.build().expect("default sampling is bounded");
        assert_eq!(vmob.style().stroke_color, YELLOW);
        let path = built_path(&vmob);
        // arange(-8, 8, 0.25) is 64 samples; plus t2 → 65 anchors.
        let anchors = anchors_of(&path);
        assert_eq!(anchors.len(), 65);
        assert_eq!(anchors[0], [-8.0, 64.0, 0.0]);
        assert_eq!(anchors[64], [8.0, 64.0, 0.0]);
    }

    #[test]
    fn function_graph_samples_t_to_t_f_t_0() {
        let vmob = FunctionGraph::new(f64::sin)
            .x_range([0.0, PI, 0.1])
            .use_smoothing(false)
            .build()
            .expect("fixture sampling is bounded");
        let anchors = anchors_of(&built_path(&vmob));
        for a in &anchors {
            assert_eq!(a[1], a[0].sin());
            assert_eq!(a[2], 0.0);
        }
    }

    // ---- ImplicitFunction ----------------------------------------------

    fn circle(x: f64, y: f64) -> f64 {
        (x * x + y * y) - 1.0
    }

    #[test]
    fn implicit_circle_is_accurate() {
        let vmob = ImplicitFunction::new(circle)
            .x_range([-1.5, 1.5])
            .y_range([-1.5, 1.5])
            .build()
            .expect("valid domain");
        let path = built_path(&vmob);
        let anchors = anchors_of(&path);
        assert!(anchors.len() > 16, "a circle needs a real curve");
        for a in &anchors {
            let r = (a[0] * a[0] + a[1] * a[1]).sqrt();
            assert!((r - 1.0).abs() < 1e-2, "anchor {a:?} off the circle");
            assert_eq!(a[2], 0.0);
        }
    }

    #[test]
    fn implicit_config_maps_through_to_the_extractor() {
        // The budget contract, tested against plot_isoline directly (the
        // module's seam): min_depth takes precedence over max_quads, and
        // ImplicitFunction's knobs land in IsolineConfig unchanged.
        let tight = IsolineConfig {
            min_depth: 1,
            max_quads: 64,
            ..IsolineConfig::default()
        };
        let (curves, stats) = plot_isoline_with_stats(circle, [-1.5, -1.5], [1.5, 1.5], &tight)
            .expect("valid domain");
        assert!(stats.leaves <= 64, "budget binds: {stats:?}");
        assert!(stats.evaluations > 0);

        // min_depth precedence: 4^4 forced leaves ignore max_quads = 1.
        let deep = IsolineConfig {
            min_depth: 4,
            max_quads: 1,
            ..IsolineConfig::default()
        };
        let (_, stats) =
            plot_isoline_with_stats(circle, [-1.5, -1.5], [1.5, 1.5], &deep).expect("valid domain");
        assert_eq!(stats.leaves, 256);

        // Passthrough: ImplicitFunction with the same knobs lays out
        // exactly the curves the extractor returns — Σ(2Lᵢ) − 1 points in
        // shared-anchor layout with break markers.
        let vmob = ImplicitFunction::new(circle)
            .x_range([-1.5, 1.5])
            .y_range([-1.5, 1.5])
            .min_depth(1)
            .max_quads(64)
            .build()
            .expect("valid domain");
        let total: usize = curves.iter().map(std::vec::Vec::len).sum();
        assert!(total > 0);
        assert_eq!(vmob.points().len(), 2 * total - 1);
    }

    #[test]
    fn implicit_empty_level_set_produces_no_points() {
        let vmob = ImplicitFunction::new(|x, y| x * x + y * y + 1.0)
            .x_range([-1.0, 1.0])
            .y_range([-1.0, 1.0])
            .build()
            .expect("valid domain");
        assert!(vmob.points().is_empty());
    }

    #[test]
    fn implicit_nan_region_is_absent_not_fatal() {
        // A field undefined on a vertical strip: curves appear only where
        // the field is defined, and nothing panics.
        let vmob = ImplicitFunction::new(|x, y| if x.abs() < 0.25 { f64::NAN } else { y - 0.5 })
            .x_range([-2.0, 2.0])
            .y_range([-2.0, 2.0])
            .build()
            .expect("valid domain");
        let anchors = anchors_of(&built_path(&vmob));
        assert!(!anchors.is_empty(), "the defined region has a zero set");
        for a in &anchors {
            assert!(a[0].is_finite() && a[1].is_finite());
            assert!(a[0].abs() >= 0.25, "curve inside the NaN strip: {a:?}");
            assert!((a[1] - 0.5).abs() < 1e-2, "off the y = 0.5 line: {a:?}");
        }
    }

    #[test]
    fn implicit_defaults_match_the_reference() {
        let f = ImplicitFunction::new(circle);
        assert_eq!(f.x_range, [-FRAME_X_RADIUS, FRAME_X_RADIUS]);
        assert_eq!(f.y_range, [-FRAME_Y_RADIUS, FRAME_Y_RADIUS]);
        assert_eq!(f.min_depth, 5);
        assert_eq!(f.max_quads, 1500);
        assert!(!f.use_smoothing);
        assert_eq!(f.joint_type, JointType::NoJoint);
    }

    // ---- graph_parametric (the get_graph seam) --------------------------

    #[test]
    fn graph_parametric_divides_the_step_and_maps_through_c2p() {
        // x_range tick step 0.5 with 5 samples per tick → sample step 0.1,
        // so arange(0, 2, 0.1) + t2 gives 21 anchors, each mapped through
        // c2p(t, f(t)).
        let curve = graph_parametric(
            |x| x,
            |cs| [2.0 * cs[0], 3.0 * cs[1], 0.0],
            [0.0, 2.0, 0.5],
            5.0,
            SamplingBudget::default(),
        );
        assert_eq!(curve.t_range_value(), [0.0, 2.0, 0.1]);
        let vmob = curve
            .use_smoothing(false)
            .build()
            .expect("fixture sampling is bounded");
        let anchors = anchors_of(&built_path(&vmob));
        assert_eq!(anchors.len(), 21);
        assert_eq!(anchors[0], [0.0, 0.0, 0.0]);
        assert_eq!(anchors[20], [4.0, 6.0, 0.0]);
        for (k, a) in anchors.iter().enumerate().take(20) {
            let t = k as f64 * 0.1;
            assert_eq!(*a, [2.0 * t, 3.0 * t, 0.0]);
        }
    }
}
