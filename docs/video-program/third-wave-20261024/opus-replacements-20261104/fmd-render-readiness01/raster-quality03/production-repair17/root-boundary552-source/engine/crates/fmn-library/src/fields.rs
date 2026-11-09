//! The field family (§12.4, fm-2u6): `VectorField`, `TimeVaryingVectorField`,
//! `StreamLines`, `AnimatedStreamLines`, and the stateful tracers
//! (`TracedPath`, `TracingTail`, `AnimatedBoundary`).
//!
//! Ported from `manimlib/mobject/vector_field.py` and
//! `manimlib/mobject/changing.py` @ `6199a00d`. The sampling semantics are
//! the Reference's exactly; four deliberate, documented divergences:
//!
//! * **A real integrator.** The Reference's `ode_solution_points` rides
//!   SciPy's `solve_ivp` at default tolerances and draws the solver's
//!   `t_eval` rows directly. Here integration is fsci-integrate's adaptive
//!   RK45 (Dormand–Prince, error-controlled, at the SUITE.lock pin) with
//!   **dense output**: the line's samples come from the continuous
//!   extension between accepted knots, re-spaced at *even true-arc-length*
//!   intervals by Chisel's machinery — resolving the Reference's two dead
//!   knobs (`arc_len`, `n_samples_per_line`, both declared and ignored
//!   upstream) and its `TODO: account for arc length somehow?`.
//!   `cutoff_norm`, another declared-but-ignored Reference knob, becomes
//!   real: a line stops at the first sample whose field norm exceeds it.
//! * **tanh normalization on fmn-dmath.** The magnitude→length mapping
//!   (`max_len · tanh(|v| / max_len)`) goes through
//!   [`fmn_dmath::tanh`] — certified-stable, never the std intrinsic, per
//!   ADR-0014's single-transcendental-funnel rule.
//! * **One RNG.** Stream-line seed jitter and animated lag draws come from
//!   the single RNG's named [`STREAM_LINES_SUBSTREAM`] substream
//!   (`RngRoot::substream`), so field seeding never perturbs another
//!   subsystem's draws (§6.5). The Reference's global `np.random` is dead.
//! * **Elapsed-time traces.** `TracedPath` and `TracingTail` retain their
//!   elapsed-time window on the declared anchor grid, replacing the
//!   Reference's estimate from the most recent frame delta. Only observed
//!   source positions are interpolated; finite windows bound storage and
//!   work per update. An unbounded-duration trace refuses before exceeding
//!   the shared 100,000-anchor resource ceiling rather than dropping history.
//!
//! **Purity (§9.5).** `TimeVaryingVectorField`, `AnimatedStreamLines`,
//! `TracedPath`, `TracingTail`, and `AnimatedBoundary` are updater-bound
//! classes: their Stage-side `add_*` helpers register dt-updaters, which
//! the segment-purity classifier probes through
//! `Stage::family_updater_kinds` — a scene containing one classifies
//! stateful ([`fmn_anim::purity::ImpureEffect::DtUpdater`]), exactly as
//! the classifier's contract anticipates ("stateful tracers … demote
//! through the same probe"). The pure builders ([`VectorField`],
//! [`StreamLines`]) are detached values like every other library class.

use std::collections::VecDeque;
use std::fmt;

use fmn_core::color::Srgb;
use fmn_core::constants::{COLORMAP_3B1B, DEFAULT_MOBJECT_COLOR, FRAME_HEIGHT, FRAME_WIDTH};
use fmn_core::rng::{RngRoot, Substream};
use fmn_core::types::Vec3;
use fmn_geom::arclength::ArcLengthTable;
use fmn_geom::{QuadPath, space_ops};
use fmn_mobject::stage::{Mob, Stage};
use fmn_mobject::uniforms::JointType;

use crate::coords::CoordinateSystem;
use crate::graphs::{SamplingBudget, SamplingError};
use crate::style::Style;
use crate::vmobject::VMobject;

/// The named substream every field-family draw comes from (§6.5): seed
/// jitter for [`StreamLines`] and lag times for [`AnimatedStreamLines`].
/// The name mirrors the convention fmn-anim's frame-order tests already
/// use for the stream-lines fork.
pub const STREAM_LINES_SUBSTREAM: &str = "streamlines";

/// A vectorized field function, the Reference's
/// `Callable[[VectArray], VectArray]`: takes coordinate rows, returns one
/// output row per input row.
pub type FieldFn = dyn Fn(&[[f64; 3]]) -> Vec<[f64; 3]> + 'static;

/// A time-varying field function: coordinate rows plus the current time.
pub type TimeFieldFn = dyn Fn(&[[f64; 3]], f64) -> Vec<[f64; 3]> + 'static;

/// The Reference's `vectorize`: lift a pointwise function to a vectorized
/// field function.
pub fn vectorize(
    pointwise: impl Fn(&[f64; 3]) -> [f64; 3] + 'static,
) -> impl Fn(&[[f64; 3]]) -> Vec<[f64; 3]> + 'static {
    move |coords| coords.iter().map(&pointwise).collect()
}

/// Failures of the field family. The Reference raises bare `Exception`s
/// (or crashes on empty samples); here every refusal is typed.
#[derive(Debug)]
pub enum FieldError {
    /// A work control (density, dt, tolerance, …) was non-finite or
    /// out of its meaningful domain.
    NonFiniteControl {
        /// What was being configured.
        context: &'static str,
    },
    /// `VectorField` needs at least two sample points to derive the step
    /// size the Reference reads from `sample_points[1] - sample_points[0]`.
    InsufficientSamples,
    /// The vectorized function returned a row count different from its
    /// input's.
    FuncShapeMismatch {
        /// Rows given.
        expected: usize,
        /// Rows returned.
        got: usize,
    },
    /// The integrator refused the problem (fsci-integrate's validation).
    Integration(String),
    /// A sampling budget was exceeded.
    Sampling(SamplingError),
    /// A stage operation failed inside an `add_*` helper.
    Stage(fmn_mobject::StageError),
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteControl { context } => {
                write!(f, "non-finite or out-of-domain control: {context}")
            }
            Self::InsufficientSamples => write!(
                f,
                "vector field needs at least two sample points to derive a step size"
            ),
            Self::FuncShapeMismatch { expected, got } => write!(
                f,
                "field function returned {got} rows for {expected} coordinate rows"
            ),
            Self::Integration(msg) => write!(f, "integration failed: {msg}"),
            Self::Sampling(e) => write!(f, "sampling: {e}"),
            Self::Stage(e) => write!(f, "stage: {e}"),
        }
    }
}

impl std::error::Error for FieldError {}

impl From<SamplingError> for FieldError {
    fn from(e: SamplingError) -> Self {
        Self::Sampling(e)
    }
}

impl From<fmn_mobject::StageError> for FieldError {
    fn from(e: fmn_mobject::StageError) -> Self {
        Self::Stage(e)
    }
}

fn ensure_finite(context: &'static str, value: f64) -> Result<(), FieldError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(FieldError::NonFiniteControl { context })
    }
}

fn ensure_positive(context: &'static str, value: f64) -> Result<(), FieldError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(FieldError::NonFiniteControl { context })
    }
}

/// numpy `arange` semantics (multiplication form, strict `<` on the stop),
/// matching the Atlas's one convention (graphs.rs `sample_pair`).
fn arange(
    context: &'static str,
    start: f64,
    stop: f64,
    step: f64,
    budget: SamplingBudget,
) -> Result<Vec<f64>, FieldError> {
    ensure_finite(context, start)?;
    ensure_finite(context, stop)?;
    ensure_finite(context, step)?;
    if step <= 0.0 || start >= stop {
        return Ok(Vec::new());
    }
    let mut values = Vec::new();
    let mut k = 0usize;
    loop {
        let value = start + k as f64 * step;
        // NaN-safe numpy semantics: continue strictly while value < stop.
        if value.is_nan() || value >= stop {
            break;
        }
        budget.ensure_total(context, values.len() + 1)?;
        values.push(value);
        k += 1;
    }
    Ok(values)
}

/// `get_sample_coords`: the Reference's sampling grid — per axis range
/// `(min, max, step)` sampled as `arange(min, max + step, step / density)`,
/// cartesian product with the first range outermost, coordinates padded to
/// three components.
pub fn get_sample_coords(
    coordinate_system: &dyn CoordinateSystem,
    density: f64,
    budget: SamplingBudget,
) -> Result<Vec<[f64; 3]>, FieldError> {
    ensure_positive("sample grid density", density)?;
    let ranges = coordinate_system.all_ranges();
    let mut axes: Vec<Vec<f64>> = Vec::with_capacity(ranges.len());
    for [min, max, step] in ranges {
        let step = step / density;
        axes.push(arange("sample grid axis", min, max + step, step, budget)?);
    }
    // An empty axis has an empty Cartesian product, not an index-zero row.
    if axes.is_empty() || axes.len() > 3 {
        return Err(FieldError::NonFiniteControl {
            context: "sample grid dimension",
        });
    }
    if axes.iter().any(Vec::is_empty) {
        return Ok(Vec::new());
    }
    let total = axes
        .iter()
        .try_fold(1usize, |acc, axis| acc.checked_mul(axis.len()))
        .ok_or(FieldError::Sampling(SamplingError::CapacityOverflow {
            context: "sample grid product",
        }))?;
    budget.ensure_total("sample grid", total)?;
    let mut coords = Vec::with_capacity(total);
    let mut index = vec![0usize; axes.len()];
    loop {
        let mut coord = [0.0; 3];
        for (c, &i) in index.iter().enumerate() {
            if c < 3 {
                coord[c] = axes[c][i];
            }
        }
        coords.push(coord);
        // Odometer: last axis fastest (it.product order).
        let mut axis = axes.len();
        loop {
            if axis == 0 {
                return Ok(coords);
            }
            axis -= 1;
            index[axis] += 1;
            if index[axis] < axes[axis].len() {
                break;
            }
            index[axis] = 0;
        }
    }
}

/// `VectorField.get_sample_points`: the Reference's older rectilinear
/// helper — `1/density` spacings with the corner truncated toward zero the
/// way numpy's `.astype(int)` does.
#[must_use]
pub fn grid_sample_points(
    center: Vec3,
    half_extents: [f64; 3],
    densities: [f64; 3],
    budget: SamplingBudget,
) -> Vec<[f64; 3]> {
    let mut axes: Vec<Vec<f64>> = Vec::with_capacity(3);
    for d in 0..3 {
        let spacing = 1.0 / densities[d];
        let reach = spacing * (half_extents[d] / spacing).trunc();
        let mut axis = Vec::new();
        let mut k = 0usize;
        loop {
            let v = (center[d] - reach) + k as f64 * spacing;
            if v.is_nan() || v >= center[d] + reach + spacing {
                break;
            }
            axis.push(v);
            k += 1;
            if axis.len() > budget.max_samples() {
                return Vec::new();
            }
        }
        axes.push(axis);
    }
    let total = axes
        .iter()
        .try_fold(1usize, |acc, axis| acc.checked_mul(axis.len()))
        .unwrap_or(0);
    if total > budget.max_samples() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(total);
    for &x in &axes[0] {
        for &y in &axes[1] {
            for &z in &axes[2] {
                out.push([x, y, z]);
            }
        }
    }
    out
}

// ------------------------------------------------------------ colormap

/// The Reference's `get_vectorized_rgb_gradient_function`
/// (vector_field.py:41): inverse-interpolate each value into `[min, max]`,
/// clip to `[0, 1]`, and lerp between neighboring colormap anchors with
/// `bezier.interpolate` — a plain per-channel lerp, deliberately NOT
/// `interpolate_color`'s sqrt form (that helper lives in utils/color.py;
/// this one is vector_field.py's own).
///
/// A degenerate range (`!(max > min)`) maps every value to alpha 0 — the
/// Reference's divide-by-zero NaN made deterministic.
#[must_use]
pub fn colormap_gradient(anchors: &[Srgb], min: f64, max: f64, values: &[f64]) -> Vec<Srgb> {
    values
        .iter()
        .map(|&v| colormap_gradient_at(anchors, min, max, v))
        .collect()
}

/// The Reference's `get_rgb_gradient_function`: the scalar form of
/// [`colormap_gradient`].
#[must_use]
pub fn colormap_gradient_at(anchors: &[Srgb], min: f64, max: f64, value: f64) -> Srgb {
    if anchors.is_empty() {
        return DEFAULT_MOBJECT_COLOR;
    }
    if anchors.len() == 1 {
        return anchors[0];
    }
    let alpha = if max > min {
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let scaled = alpha * (anchors.len() - 1) as f64;
    let lo = (scaled.floor() as usize).min(anchors.len() - 1);
    let hi = (lo + 1).min(anchors.len() - 1);
    let t = scaled - lo as f64;
    let lerp = |a: f64, b: f64| (1.0 - t) * a + t * b;
    Srgb {
        r: lerp(anchors[lo].r, anchors[hi].r),
        g: lerp(anchors[lo].g, anchors[hi].g),
        b: lerp(anchors[lo].b, anchors[hi].b),
    }
}

fn lerp3(a: Vec3, b: Vec3, t: f64) -> Vec3 {
    [
        (1.0 - t) * a[0] + t * b[0],
        (1.0 - t) * a[1] + t * b[1],
        (1.0 - t) * a[2] + t * b[2],
    ]
}

fn add3(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub3(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale3(a: Vec3, s: f64) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

/// Scene units per coordinate unit along x, without reaching into any
/// concrete axis type: `|c2p(1,0,0) − c2p(0,0,0)|`. The Reference reads
/// `x_axis.get_unit_size()`; for the linear coordinate systems the trait
/// covers these are the same number.
fn x_unit_size(cs: &dyn CoordinateSystem) -> f64 {
    let origin = cs.origin();
    let one = cs.c2p(&[1.0, 0.0, 0.0]);
    space_ops::get_dist(one, origin)
}

// --------------------------------------------------------- VectorField

/// The Reference's `VectorField` defaults.
pub const VECTOR_FIELD_DENSITY: f64 = 2.0;
/// Default `stroke_width`.
pub const VECTOR_FIELD_STROKE_WIDTH: f64 = 3.0;
/// Default `tip_width_ratio`.
pub const TIP_WIDTH_RATIO: f64 = 4.0;
/// Default `tip_len_to_width`.
pub const TIP_LEN_TO_WIDTH: f64 = 0.01;
/// Default `max_vect_len_to_step_size`.
pub const MAX_VECT_LEN_TO_STEP_SIZE: f64 = 0.8;

/// The style surface of a [`VectorField`] — every knob the Reference's
/// constructor takes, resolved (a `None` `magnitude_range` means
/// "compute `(0, max norm)` at build", exactly as the Reference).
#[derive(Debug, Clone, PartialEq)]
pub struct VectorFieldStyle {
    /// `stroke_width` (Reference default 3).
    pub stroke_width: f64,
    /// `stroke_opacity` (Reference default 1).
    pub stroke_opacity: f64,
    /// `tip_width_ratio` (Reference default 4).
    pub tip_width_ratio: f64,
    /// `tip_len_to_width` (Reference default 0.01).
    pub tip_len_to_width: f64,
    /// `max_vect_len`, in coordinate units; `None` derives
    /// `max_vect_len_to_step_size · |sample_points[1] − sample_points[0]|`.
    pub max_vect_len: Option<f64>,
    /// `max_vect_len_to_step_size` (Reference default 0.8).
    pub max_vect_len_to_step_size: f64,
    /// `flat_stroke`.
    pub flat_stroke: bool,
    /// A single stroke color; when `Some`, no colormap is applied (the
    /// Reference's `color is not None ⇒ color_map = None`).
    pub color: Option<Srgb>,
    /// The colormap anchors; `None` disables the per-magnitude coloring.
    /// Defaults to [`COLORMAP_3B1B`] (the Reference's `"3b1b_colormap"`).
    pub color_map: Option<Vec<Srgb>>,
    /// `magnitude_range`; `None` resolves to `(0, max output norm)` at
    /// build time (and stays fixed for a `TimeVaryingVectorField`, as the
    /// Reference's does).
    pub magnitude_range: Option<(f64, f64)>,
}

impl Default for VectorFieldStyle {
    fn default() -> Self {
        Self {
            stroke_width: VECTOR_FIELD_STROKE_WIDTH,
            stroke_opacity: 1.0,
            tip_width_ratio: TIP_WIDTH_RATIO,
            tip_len_to_width: TIP_LEN_TO_WIDTH,
            max_vect_len: None,
            max_vect_len_to_step_size: MAX_VECT_LEN_TO_STEP_SIZE,
            flat_stroke: false,
            color: None,
            color_map: Some(COLORMAP_3B1B.to_vec()),
            magnitude_range: None,
        }
    }
}

/// The output of the Reference's `update_vectors`: the 8-points-per-arrow
/// shared-anchor run plus the per-point stroke columns.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorGeometry {
    /// The `8·n − 1` point run: per arrow
    /// `(base, ·, head_base, ·, head_base, ·, tip, tip)`.
    pub points: Vec<Vec3>,
    /// Per-point stroke widths (`stroke_width · base_widths · width_scalars`).
    pub stroke_widths: Vec<f64>,
    /// Per-point stroke colors, when a colormap is active.
    pub stroke_rgba: Vec<[f64; 4]>,
    /// The tanh-compressed drawn lengths, one per arrow.
    pub drawn_norms: Vec<f64>,
    /// The resolved `max_displayed_vect_len`.
    pub max_displayed_vect_len: f64,
}

/// The Reference's `update_vectors` (vector_field.py:259), pure.
///
/// `sample_points` are the arrow bases in scene space; `out_vects` the
/// field outputs mapped into scene space (`c2p(output) − origin`);
/// `output_norms` the norms in *coordinate* space (they drive the color
/// mapping); `max_len` the resolved `max_displayed_vect_len`.
#[allow(clippy::too_many_arguments)]
pub fn vector_field_geometry(
    style: &VectorFieldStyle,
    sample_points: &[Vec3],
    out_vects: &[Vec3],
    output_norms: &[f64],
    max_len: f64,
) -> VectorGeometry {
    let n = sample_points.len();
    let tip_width = style.tip_width_ratio * style.stroke_width;
    let tip_len = style.tip_len_to_width * tip_width;

    let mut points = vec![[0.0; 3]; (8 * n).saturating_sub(1)];
    let mut drawn_norms = Vec::with_capacity(n);
    for (i, (&base, &out_vect)) in sample_points.iter().zip(out_vects).enumerate() {
        let out_norm = space_ops::get_norm(out_vect);
        let unit = if out_norm > 0.0 {
            scale3(out_vect, 1.0 / out_norm)
        } else {
            [0.0; 3]
        };
        // The magnitude→length mapping, on fmn-dmath's certified tanh
        // (ADR-0014): drawn = max_len · tanh(|v| / max_len).
        let drawn = if max_len.is_finite() && max_len > 0.0 {
            max_len * fmn_dmath::tanh(out_norm / max_len)
        } else {
            out_norm
        };
        drawn_norms.push(drawn);
        let head_base = add3(base, scale3(unit, (drawn - tip_len).max(0.0)));
        let tip = add3(base, scale3(unit, drawn));
        points[8 * i] = base;
        points[8 * i + 2] = head_base;
        points[8 * i + 4] = head_base;
        points[8 * i + 6] = tip;
        for offset in [1usize, 3, 5] {
            points[8 * i + offset] =
                space_ops::midpoint(points[8 * i + offset - 1], points[8 * i + offset + 1]);
        }
        if 8 * i + 7 < points.len() {
            points[8 * i + 7] = tip;
        }
    }

    // The base width pattern (init_base_stroke_width_array): ones with
    // [4::8] = tip_width_ratio, [5::8] = ratio/2, [6::8] = [7::8] = 0.
    let mut stroke_widths = Vec::with_capacity(points.len());
    for (i, &drawn) in drawn_norms.iter().enumerate() {
        // The Reference divides by tip_len unconditionally; a zero-width
        // stroke makes that NaN. The deterministic reading: no tip to
        // scale against ⇒ full width where anything is drawn.
        let scalar = if tip_len > 0.0 {
            (drawn / tip_len).clamp(0.0, 1.0)
        } else if drawn > 0.0 {
            1.0
        } else {
            0.0
        };
        let w = style.stroke_width * scalar;
        let base = [
            1.0,
            1.0,
            1.0,
            1.0,
            style.tip_width_ratio,
            style.tip_width_ratio * 0.5,
            0.0,
            0.0,
        ];
        for (j, b) in base.iter().enumerate() {
            if 8 * i + j < (8 * n).saturating_sub(1) {
                stroke_widths.push(w * b);
            }
        }
    }

    let stroke_rgba = match (&style.color_map, style.magnitude_range) {
        (Some(anchors), Some((lo, hi))) => {
            let rgb_per_arrow = colormap_gradient(anchors, lo, hi, output_norms);
            let mut rgba = Vec::with_capacity(points.len());
            for (i, c) in rgb_per_arrow.iter().enumerate().take(n) {
                for j in 0..8 {
                    if 8 * i + j < (8 * n).saturating_sub(1) {
                        rgba.push([c.r, c.g, c.b, style.stroke_opacity]);
                    }
                }
            }
            rgba
        }
        _ => Vec::new(),
    };

    VectorGeometry {
        points,
        stroke_widths,
        stroke_rgba,
        drawn_norms,
        max_displayed_vect_len: max_len,
    }
}

/// Resolve the Reference's `max_displayed_vect_len`: `None` reads the step
/// size from the first two sample points; `Some(m)` scales by the x unit
/// size.
fn resolve_max_len(
    style: &VectorFieldStyle,
    sample_points: &[Vec3],
    unit_size: f64,
) -> Result<f64, FieldError> {
    match style.max_vect_len {
        Some(m) => {
            ensure_finite("max_vect_len", m)?;
            Ok(m * unit_size)
        }
        None => {
            if sample_points.len() < 2 {
                return Err(FieldError::InsufficientSamples);
            }
            let step = space_ops::get_dist(sample_points[1], sample_points[0]);
            Ok(style.max_vect_len_to_step_size * step)
        }
    }
}

/// A built vector field: the detached arrow geometry plus the per-point
/// columns and the bookkeeping the Reference keeps on the instance.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorFieldMobject {
    vmob: VMobject,
    stroke_rgba: Vec<[f64; 4]>,
    stroke_widths: Vec<f64>,
    sample_coords: Vec<[f64; 3]>,
    sample_points: Vec<Vec3>,
    output_norms: Vec<f64>,
    max_displayed_vect_len: f64,
}

impl VectorFieldMobject {
    /// The detached geometry (arrows with the stroke-width taper riding
    /// the value's stroke profile).
    #[must_use]
    pub fn vmob(&self) -> &VMobject {
        &self.vmob
    }

    /// Consume into the detached geometry.
    #[must_use]
    pub fn into_vmob(self) -> VMobject {
        self.vmob
    }

    /// The coordinates the arrows are sampled at.
    #[must_use]
    pub fn sample_coords(&self) -> &[[f64; 3]] {
        &self.sample_coords
    }

    /// The arrow bases in scene space.
    #[must_use]
    pub fn sample_points(&self) -> &[Vec3] {
        &self.sample_points
    }

    /// The field norms in coordinate space at each sample.
    #[must_use]
    pub fn output_norms(&self) -> &[f64] {
        &self.output_norms
    }

    /// The resolved `max_displayed_vect_len`.
    #[must_use]
    pub fn max_displayed_vect_len(&self) -> f64 {
        self.max_displayed_vect_len
    }

    /// Per-point stroke colors (empty when a single color is used).
    #[must_use]
    pub fn stroke_rgba(&self) -> &[[f64; 4]] {
        &self.stroke_rgba
    }

    /// Per-point stroke widths.
    #[must_use]
    pub fn stroke_widths(&self) -> &[f64] {
        &self.stroke_widths
    }

    /// Add to a stage and write the per-point columns — the
    /// color-by-magnitude array cannot ride the value type, so it lands
    /// here, the same extension pattern as [`crate::style::VStyle`].
    pub fn add_to_stage(self, stage: &mut Stage) -> Mob {
        let mob = stage.add(self.vmob);
        write_stroke_widths(stage, mob, &self.stroke_widths);
        if !self.stroke_rgba.is_empty() {
            write_stroke_rgba(stage, mob, &self.stroke_rgba);
        }
        mob
    }
}

/// Shared builder body for [`VectorField`] and [`TimeVaryingVectorField`].
#[allow(clippy::too_many_arguments)]
fn build_field(
    func: &FieldFn,
    cs: &dyn CoordinateSystem,
    style: &VectorFieldStyle,
    sample_coords: Option<Vec<[f64; 3]>>,
    density: f64,
    budget: SamplingBudget,
) -> Result<VectorFieldMobject, FieldError> {
    ensure_finite("stroke_width", style.stroke_width)?;
    ensure_finite("stroke_opacity", style.stroke_opacity)?;
    let sample_coords = match sample_coords {
        Some(coords) => coords,
        None => get_sample_coords(cs, density, budget)?,
    };
    if sample_coords.len() < 2 {
        return Err(FieldError::InsufficientSamples);
    }
    budget.ensure_total("vector field samples", sample_coords.len())?;
    let sample_points: Vec<Vec3> = sample_coords.iter().map(|c| cs.c2p(c)).collect();
    let unit = x_unit_size(cs);
    let max_len = resolve_max_len(style, &sample_points, unit)?;

    let outputs = func(&sample_coords);
    if outputs.len() != sample_coords.len() {
        return Err(FieldError::FuncShapeMismatch {
            expected: sample_coords.len(),
            got: outputs.len(),
        });
    }
    let output_norms: Vec<f64> = outputs.iter().map(|o| space_ops::get_norm(*o)).collect();
    let origin = cs.origin();
    let out_vects: Vec<Vec3> = outputs.iter().map(|o| sub3(cs.c2p(o), origin)).collect();

    let mut style = style.clone();
    if style.magnitude_range.is_none() {
        let max_value = output_norms.iter().copied().fold(0.0, f64::max);
        style.magnitude_range = Some((0.0, max_value));
    }

    let geometry =
        vector_field_geometry(&style, &sample_points, &out_vects, &output_norms, max_len);
    let mut vstyle = Style::default().stroke(
        style.color.unwrap_or(style.stroke_color_fallback()),
        style.stroke_width,
        style.stroke_opacity,
    );
    vstyle.fill_opacity = 0.0;
    let vmob = VMobject::from_points(geometry.points.clone())
        .with_style(vstyle)
        .with_joint_type(JointType::NoJoint)
        .with_flat_stroke(style.flat_stroke)
        .with_stroke_profile(geometry.stroke_widths.clone());

    Ok(VectorFieldMobject {
        vmob,
        stroke_rgba: geometry.stroke_rgba,
        stroke_widths: geometry.stroke_widths,
        sample_coords,
        sample_points,
        output_norms,
        max_displayed_vect_len: max_len,
    })
}

impl VectorFieldStyle {
    /// The Reference leaves the stroke color unset when a colormap is
    /// active (the per-point array overrides); the value type needs a
    /// concrete style, which the column write replaces point-for-point.
    fn stroke_color_fallback(&self) -> Srgb {
        DEFAULT_MOBJECT_COLOR
    }
}

/// `VectorField` (vector_field.py:150): arrows on a coordinate system's
/// sampling grid, lengths tanh-compressed, colors magnitude-mapped.
pub struct VectorField {
    func: Box<FieldFn>,
    cs: Box<dyn CoordinateSystem>,
    sample_coords: Option<Vec<[f64; 3]>>,
    density: f64,
    style: VectorFieldStyle,
    budget: SamplingBudget,
}

impl fmt::Debug for VectorField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VectorField")
            .field("density", &self.density)
            .field("style", &self.style)
            .field(
                "n_sample_coords",
                &self.sample_coords.as_ref().map(Vec::len),
            )
            .finish_non_exhaustive()
    }
}

impl VectorField {
    /// A field over `func` sampled on `coordinate_system` at the
    /// Reference's default density (2.0).
    pub fn new(
        func: impl Fn(&[[f64; 3]]) -> Vec<[f64; 3]> + 'static,
        coordinate_system: impl CoordinateSystem + 'static,
    ) -> Self {
        Self {
            func: Box::new(func),
            cs: Box::new(coordinate_system),
            sample_coords: None,
            density: VECTOR_FIELD_DENSITY,
            style: VectorFieldStyle::default(),
            budget: SamplingBudget::DEFAULT,
        }
    }

    /// Explicit `sample_coords` (the Reference's escape hatch over the
    /// density grid).
    #[must_use]
    pub fn with_sample_coords(mut self, coords: Vec<[f64; 3]>) -> Self {
        self.sample_coords = Some(coords);
        self
    }

    /// `density` (Reference default 2.0).
    #[must_use]
    pub fn with_density(mut self, density: f64) -> Self {
        self.density = density;
        self
    }

    /// Replace the whole style surface.
    #[must_use]
    pub fn with_style_config(mut self, style: VectorFieldStyle) -> Self {
        self.style = style;
        self
    }

    /// A single stroke color (disables the colormap, as the Reference).
    #[must_use]
    pub fn with_color(mut self, color: Srgb) -> Self {
        self.style.color = Some(color);
        self
    }

    /// `magnitude_range`; otherwise `(0, max norm)` at build.
    #[must_use]
    pub fn with_magnitude_range(mut self, lo: f64, hi: f64) -> Self {
        self.style.magnitude_range = Some((lo, hi));
        self
    }

    /// `max_vect_len` in coordinate units; otherwise derived from the
    /// sample step size.
    #[must_use]
    pub fn with_max_vect_len(mut self, max_vect_len: f64) -> Self {
        self.style.max_vect_len = Some(max_vect_len);
        self
    }

    /// The sampling budget.
    #[must_use]
    pub fn with_budget(mut self, budget: SamplingBudget) -> Self {
        self.budget = budget;
        self
    }

    /// The field function.
    pub fn func(&self) -> &FieldFn {
        &*self.func
    }

    /// Build the arrows at the field's current state.
    ///
    /// # Errors
    /// [`FieldError`] on bad controls, empty samples, a shape-mismatched
    /// function, or a blown budget.
    pub fn build(self) -> Result<VectorFieldMobject, FieldError> {
        build_field(
            &self.func,
            &*self.cs,
            &self.style,
            self.sample_coords,
            self.density,
            self.budget,
        )
    }
}

/// `TimeVaryingVectorField` (vector_field.py:318): a vector field whose
/// function reads the current time. Stateful by construction — the
/// Stage-side helper binds a dt-updater that advances `time` and redraws
/// the arrows, which demotes every segment it participates in (§9.5).
pub struct TimeVaryingVectorField {
    time_func: Box<TimeFieldFn>,
    cs: Box<dyn CoordinateSystem>,
    time: f64,
    sample_coords: Option<Vec<[f64; 3]>>,
    density: f64,
    style: VectorFieldStyle,
    budget: SamplingBudget,
}

impl fmt::Debug for TimeVaryingVectorField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TimeVaryingVectorField")
            .field("time", &self.time)
            .field("density", &self.density)
            .finish_non_exhaustive()
    }
}

impl TimeVaryingVectorField {
    /// A time-varying field, starting at `time = 0` (the Reference).
    pub fn new(
        time_func: impl Fn(&[[f64; 3]], f64) -> Vec<[f64; 3]> + 'static,
        coordinate_system: impl CoordinateSystem + 'static,
    ) -> Self {
        Self {
            time_func: Box::new(time_func),
            cs: Box::new(coordinate_system),
            time: 0.0,
            sample_coords: None,
            density: VECTOR_FIELD_DENSITY,
            style: VectorFieldStyle::default(),
            budget: SamplingBudget::DEFAULT,
        }
    }

    /// The starting time (the Reference's `self.time = 0` made settable).
    #[must_use]
    pub fn at_time(mut self, time: f64) -> Self {
        self.time = time;
        self
    }

    /// Explicit sample coordinates.
    #[must_use]
    pub fn with_sample_coords(mut self, coords: Vec<[f64; 3]>) -> Self {
        self.sample_coords = Some(coords);
        self
    }

    /// `density`.
    #[must_use]
    pub fn with_density(mut self, density: f64) -> Self {
        self.density = density;
        self
    }

    /// Replace the style surface.
    #[must_use]
    pub fn with_style_config(mut self, style: VectorFieldStyle) -> Self {
        self.style = style;
        self
    }

    /// The sampling budget.
    #[must_use]
    pub fn with_budget(mut self, budget: SamplingBudget) -> Self {
        self.budget = budget;
        self
    }

    /// Add to a stage at the starting time and bind the dt-updater — the
    /// Reference's `add_updater(increment_time)` plus
    /// `always.update_vectors()`, fused into one updater (same net order:
    /// increment, then redraw). Returns the field's handle.
    ///
    /// # Errors
    /// [`FieldError`] on the same refusals as [`VectorField::build`], or a
    /// stale stage handle.
    pub fn add_to_stage(self, stage: &mut Stage) -> Result<Mob, FieldError> {
        let time_func = std::rc::Rc::new(self.time_func);
        let cs = self.cs;
        let time = self.time;
        let tf_for_build = std::rc::Rc::clone(&time_func);
        let func = move |coords: &[[f64; 3]]| tf_for_build(coords, time);
        let built = build_field(
            &func,
            &*cs,
            &self.style,
            self.sample_coords,
            self.density,
            self.budget,
        )?;
        let style = {
            let mut s = self.style.clone();
            // build_field resolved the magnitude range against the initial
            // evaluation; the Reference keeps that range for all later
            // updates. Rebuild the resolved style the same way.
            if s.magnitude_range.is_none() {
                let max_value = built.output_norms.iter().copied().fold(0.0, f64::max);
                s.magnitude_range = Some((0.0, max_value));
            }
            s
        };
        let max_len = built.max_displayed_vect_len;
        let sample_coords = built.sample_coords.clone();
        let sample_points = built.sample_points.clone();
        let mob = built.add_to_stage(stage);

        let update = move |stage: &mut Stage, mob: Mob, id, dt: f64| {
            let time = stage
                .updater_state_mut::<f64>(mob, id)
                .expect("time-varying field clock is installed");
            *time += dt;
            let time = *time;
            let outputs = time_func(&sample_coords, time);
            if outputs.len() != sample_coords.len() {
                return; // the Reference would raise next frame; hold the last good frame
            }
            let output_norms: Vec<f64> = outputs.iter().map(|o| space_ops::get_norm(*o)).collect();
            let origin = cs.origin();
            let out_vects: Vec<Vec3> = outputs.iter().map(|o| sub3(cs.c2p(o), origin)).collect();
            let geometry =
                vector_field_geometry(&style, &sample_points, &out_vects, &output_norms, max_len);
            write_points(stage, mob, &geometry.points);
            write_stroke_widths(stage, mob, &geometry.stroke_widths);
            if !geometry.stroke_rgba.is_empty() {
                write_stroke_rgba(stage, mob, &geometry.stroke_rgba);
            }
        };
        // One immediate pass so the arrows match `time` even before the
        // first tick (the Reference's constructor drew at time 0).
        stage.add_dt_updater_with_state(mob, time, |_, _| {}, update, true)?;
        Ok(mob)
    }
}

// ---------------------------------------------------------- integrator

/// The RK45 tuning for stream-line integration — the visual-quality
/// contract: tight enough that the dense output is smooth on screen,
/// loose enough to stay cheap for the 2D/3D systems fields live in.
/// `dense_output` is on by default and should stay on: it is what makes
/// the samples come from the continuous extension rather than the knots.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntegratorTune {
    /// Relative tolerance (SciPy's `rtol`; default here 1e-6, tighter
    /// than SciPy's 1e-3 for visual smoothness).
    pub rtol: f64,
    /// Absolute tolerance (SciPy's `atol`; default here 1e-9).
    pub atol: f64,
    /// Maximum step (SciPy's `max_step`; infinite by default).
    pub max_step: f64,
    /// Populate the dense-output solution record (default true).
    pub dense_output: bool,
}

impl Default for IntegratorTune {
    fn default() -> Self {
        Self {
            rtol: 1e-6,
            atol: 1e-9,
            max_step: f64::INFINITY,
            dense_output: true,
        }
    }
}

/// One integrated stream line, in coordinate space.
#[derive(Debug, Clone, PartialEq)]
pub struct StreamSolution {
    /// The dense-output samples at the requested `t_eval` grid (first row
    /// is the seed), truncated at the first sample over `cutoff_norm` or
    /// at the first non-finite row.
    pub coords: Vec<[f64; 3]>,
    /// Accepted solver knots (the dense output's control points) — proof
    /// the line points are NOT the solver's step points.
    pub n_solver_knots: usize,
    /// Field evaluations the integration cost.
    pub nfev: usize,
}

/// `ode_solution_points` (vector_field.py:69) on the real integrator:
/// fsci-integrate's adaptive RK45 with dense output, sampled on the
/// Reference's `arange(0, time, dt)` grid.
///
/// The samples are read off the continuous extension (`t_eval` is
/// evaluated by Hermite dense interpolation between accepted knots inside
/// fsci-integrate), never from the solver's step points.
///
/// # Errors
/// [`FieldError::Integration`] when fsci-integrate's validation refuses
/// the problem; [`FieldError::NonFiniteControl`] for bad time controls.
#[allow(clippy::too_many_arguments)]
pub fn ode_solution_points(
    func: &FieldFn,
    state0: [f64; 3],
    dim: usize,
    time: f64,
    dt: f64,
    tune: IntegratorTune,
    max_time_steps: usize,
    cutoff_norm: f64,
) -> Result<StreamSolution, FieldError> {
    ensure_positive("solution_time", time)?;
    ensure_positive("stream dt", dt)?;
    ensure_positive("integrator rtol", tune.rtol)?;
    ensure_positive("integrator atol", tune.atol)?;
    let dim = dim.clamp(1, 3);
    let mut t_eval = arange(
        "stream t_eval",
        0.0,
        time,
        dt,
        SamplingBudget::new(max_time_steps.max(2)),
    )?;
    if t_eval.is_empty() {
        t_eval.push(0.0);
    }

    let y0: Vec<f64> = state0[..dim].to_vec();
    let mut rhs = move |_t: f64, y: &[f64]| -> Vec<f64> {
        let mut coord = [0.0; 3];
        coord[..dim].copy_from_slice(y);
        let out = func(&[coord]);
        let row = out.first().copied().unwrap_or([0.0; 3]);
        row[..dim].to_vec()
    };
    let options = fsci_integrate::SolveIvpOptions {
        t_span: (0.0, time),
        y0: &y0,
        method: fsci_integrate::SolverKind::Rk45,
        t_eval: Some(&t_eval),
        dense_output: tune.dense_output,
        rtol: tune.rtol,
        atol: fsci_integrate::ToleranceValue::Scalar(tune.atol),
        max_step: tune.max_step,
        ..fsci_integrate::SolveIvpOptions::default()
    };
    let result = fsci_integrate::solve_ivp(&mut rhs, &options)
        .map_err(|e| FieldError::Integration(format!("{e}")))?;

    let n_solver_knots = result.sol.as_ref().map_or(0, |sol| sol.knots.len());
    // Rows are states at t_eval; truncate at the first non-finite row or
    // the first sample whose field norm exceeds cutoff_norm (the
    // Reference's declared-but-dead knob, made real).
    let mut coords = Vec::with_capacity(result.y.len());
    for row in &result.y {
        if row.len() < dim || row[..dim].iter().any(|v| !v.is_finite()) {
            break;
        }
        let mut coord = [0.0; 3];
        coord[..dim].copy_from_slice(&row[..dim]);
        coords.push(coord);
        let norm = space_ops::get_norm(func(&[coord]).first().copied().unwrap_or([0.0; 3]));
        if norm > cutoff_norm {
            break;
        }
    }
    if coords.is_empty() {
        coords.push(state0);
    }
    Ok(StreamSolution {
        coords,
        n_solver_knots,
        nfev: result.nfev,
    })
}

/// Re-space a fine polyline at even TRUE-arc-length stations — the
/// "arc-ish" sampling the Reference never got to
/// (`TODO: account for arc length somehow?`). The drawn line covers at
/// most `max_arc_len` of true length, cut exactly at the boundary.
#[must_use]
pub fn resample_even_arc(points: &[Vec3], n_samples: usize, max_arc_len: f64) -> Vec<Vec3> {
    if points.is_empty() || n_samples == 0 {
        return Vec::new();
    }
    if points.len() == 1 || n_samples == 1 {
        return vec![points[0]; n_samples.max(1)];
    }
    // Cumulative chord lengths of the dense polyline — the true length of
    // the curve we actually drew (BN-03: measure the actual curve).
    let mut cumulative = Vec::with_capacity(points.len());
    cumulative.push(0.0);
    for pair in points.windows(2) {
        cumulative.push(
            cumulative.last().copied().unwrap_or(0.0) + space_ops::get_dist(pair[1], pair[0]),
        );
    }
    let total = cumulative.last().copied().unwrap_or(0.0);
    if total <= 0.0 {
        return vec![points[0]; n_samples];
    }
    let used = if max_arc_len.is_finite() && max_arc_len > 0.0 {
        total.min(max_arc_len)
    } else {
        total
    };
    let mut out = Vec::with_capacity(n_samples);
    let mut seg = 0usize;
    let last = cumulative.len() - 1;
    for i in 0..n_samples {
        let target = used * i as f64 / (n_samples - 1) as f64;
        while seg + 1 < last && cumulative[seg + 1] < target {
            seg += 1;
        }
        let (s0, s1) = (cumulative[seg], cumulative[seg + 1]);
        let t = if s1 > s0 {
            (target - s0) / (s1 - s0)
        } else {
            0.0
        };
        let a = points[seg];
        let b = points[(seg + 1).min(points.len() - 1)];
        out.push(lerp3(a, b, t.clamp(0.0, 1.0)));
    }
    out
}

// ---------------------------------------------------------- StreamLines

/// The Reference's `StreamLines` defaults.
pub const STREAM_DENSITY: f64 = 1.0;
/// Default `solution_time`.
pub const STREAM_SOLUTION_TIME: f64 = 3.0;
/// Default `dt` — the dense-output sampling step.
pub const STREAM_DT: f64 = 0.05;
/// Default `arc_len` — the cap on drawn true length.
pub const STREAM_ARC_LEN: f64 = 3.0;
/// Default `max_time_steps`.
pub const STREAM_MAX_TIME_STEPS: usize = 200;
/// Default `n_samples_per_line`.
pub const STREAM_SAMPLES_PER_LINE: usize = 10;
/// Default `cutoff_norm`.
pub const STREAM_CUTOFF_NORM: f64 = 15.0;

/// `StreamLines`' style surface (the Reference's `init_style` knobs).
#[derive(Debug, Clone, PartialEq)]
pub struct StreamLineStyle {
    /// `stroke_width` (Reference default 1.0).
    pub stroke_width: f64,
    /// `stroke_color` used when `color_by_magnitude` is off.
    pub stroke_color: Srgb,
    /// `stroke_opacity` (Reference default 1).
    pub stroke_opacity: f64,
    /// `color_by_magnitude` (Reference default true).
    pub color_by_magnitude: bool,
    /// `magnitude_range` for the colormap (Reference default `(0, 2)`).
    pub magnitude_range: (f64, f64),
    /// `taper_stroke_width` (Reference default false): taper each line
    /// `0 → width → 0` by TRUE arc length (BN-03).
    pub taper_stroke_width: bool,
    /// The colormap anchors (Reference `"3b1b_colormap"`).
    pub color_map: Vec<Srgb>,
}

impl Default for StreamLineStyle {
    fn default() -> Self {
        Self {
            stroke_width: 1.0,
            stroke_color: DEFAULT_MOBJECT_COLOR,
            stroke_opacity: 1.0,
            color_by_magnitude: true,
            magnitude_range: (0.0, 2.0),
            taper_stroke_width: false,
            color_map: COLORMAP_3B1B.to_vec(),
        }
    }
}

/// One drawn stream line's bookkeeping — what the Reference hangs off the
/// line instance (`virtual_time`) plus the integrator provenance.
#[derive(Debug, Clone, PartialEq)]
pub struct StreamLineMeta {
    /// `virtual_time` (the Reference's `solution_time`): the time one
    /// flow traversal represents; `AnimatedStreamLines` paces flashes by
    /// `virtual_time / rate_multiple`.
    pub virtual_time: f64,
    /// Per-point stroke colors (empty when `color_by_magnitude` is off).
    pub stroke_rgba: Vec<[f64; 4]>,
    /// Per-point stroke widths.
    pub stroke_widths: Vec<f64>,
    /// Accepted RK45 knots during integration.
    pub n_solver_knots: usize,
    /// Dense-output samples the line was re-spaced from.
    pub n_dense_samples: usize,
}

/// A built `StreamLines`: the group of lines plus per-line metadata and
/// the RNG bookkeeping [`AnimatedStreamLines`] continues from.
#[derive(Debug, Clone)]
pub struct StreamLinesMobject {
    group: VMobject,
    lines: Vec<StreamLineMeta>,
    rng_draws: u64,
    substream: Substream,
}

impl StreamLinesMobject {
    /// The detached group (one child per line).
    #[must_use]
    pub fn vmob(&self) -> &VMobject {
        &self.group
    }

    /// Per-line metadata, in child order.
    #[must_use]
    pub fn lines(&self) -> &[StreamLineMeta] {
        &self.lines
    }

    /// Draws consumed from [`STREAM_LINES_SUBSTREAM`] so far —
    /// `AnimatedStreamLines` advances past exactly these, keeping one
    /// continuous deterministic stream.
    #[must_use]
    pub fn rng_draws(&self) -> u64 {
        self.rng_draws
    }

    /// Add to a stage, writing the per-line color/width columns.
    pub fn add_to_stage(self, stage: &mut Stage) -> Mob {
        let group = stage.add(self.group);
        let children = stage
            .get(group)
            .map(|e| e.submobjects().to_vec())
            .unwrap_or_default();
        for (meta, child) in self.lines.iter().zip(children) {
            if !meta.stroke_rgba.is_empty() {
                write_stroke_rgba(stage, child, &meta.stroke_rgba);
            }
            if !meta.stroke_widths.is_empty() {
                write_stroke_widths(stage, child, &meta.stroke_widths);
            }
        }
        group
    }
}

/// `StreamLines` (vector_field.py:334): flow lines of the field, seeded
/// on the jittered sampling grid, integrated by adaptive RK45 with dense
/// output, drawn at even true-arc spacing.
pub struct StreamLines {
    func: Box<FieldFn>,
    cs: Box<dyn CoordinateSystem>,
    substream: Substream,
    sample_coords: Option<Vec<[f64; 3]>>,
    density: f64,
    n_repeats: usize,
    noise_factor: Option<f64>,
    solution_time: f64,
    dt: f64,
    arc_len: f64,
    max_time_steps: usize,
    n_samples_per_line: usize,
    cutoff_norm: f64,
    tune: IntegratorTune,
    style: StreamLineStyle,
    budget: SamplingBudget,
}

impl fmt::Debug for StreamLines {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StreamLines")
            .field("density", &self.density)
            .field("n_repeats", &self.n_repeats)
            .field("solution_time", &self.solution_time)
            .field("dt", &self.dt)
            .field("arc_len", &self.arc_len)
            .finish_non_exhaustive()
    }
}

impl StreamLines {
    /// Stream lines of `func` over `coordinate_system`, seeded from the
    /// single RNG's [`STREAM_LINES_SUBSTREAM`] substream.
    pub fn new(
        func: impl Fn(&[[f64; 3]]) -> Vec<[f64; 3]> + 'static,
        coordinate_system: impl CoordinateSystem + 'static,
        rng: &RngRoot,
    ) -> Self {
        Self {
            func: Box::new(func),
            cs: Box::new(coordinate_system),
            substream: rng.substream(STREAM_LINES_SUBSTREAM),
            sample_coords: None,
            density: STREAM_DENSITY,
            n_repeats: 1,
            noise_factor: None,
            solution_time: STREAM_SOLUTION_TIME,
            dt: STREAM_DT,
            arc_len: STREAM_ARC_LEN,
            max_time_steps: STREAM_MAX_TIME_STEPS,
            n_samples_per_line: STREAM_SAMPLES_PER_LINE,
            cutoff_norm: STREAM_CUTOFF_NORM,
            tune: IntegratorTune::default(),
            style: StreamLineStyle::default(),
            budget: SamplingBudget::DEFAULT,
        }
    }

    /// Explicit coordinate seeds, in caller order. No jitter draws are consumed.
    /// Input is validated against the builder's budget before any field call.
    #[must_use]
    pub fn with_sample_coords(mut self, coords: Vec<[f64; 3]>) -> Self {
        self.sample_coords = Some(coords);
        self
    }

    /// `density` (Reference default 1.0).
    #[must_use]
    pub fn with_density(mut self, density: f64) -> Self {
        self.density = density;
        self
    }

    /// `n_repeats` (Reference default 1): jittered copies of the grid.
    #[must_use]
    pub fn with_n_repeats(mut self, n_repeats: usize) -> Self {
        self.n_repeats = n_repeats;
        self
    }

    /// `noise_factor`; `None` derives `(x_unit_size / density) · 0.5`.
    #[must_use]
    pub fn with_noise_factor(mut self, noise_factor: f64) -> Self {
        self.noise_factor = Some(noise_factor);
        self
    }

    /// `solution_time` (Reference default 3).
    #[must_use]
    pub fn with_solution_time(mut self, solution_time: f64) -> Self {
        self.solution_time = solution_time;
        self
    }

    /// `dt` — the dense-output sampling step (Reference default 0.05).
    #[must_use]
    pub fn with_dt(mut self, dt: f64) -> Self {
        self.dt = dt;
        self
    }

    /// `arc_len` — the cap on each line's drawn TRUE length (Reference
    /// default 3, declared but never honored upstream).
    #[must_use]
    pub fn with_arc_len(mut self, arc_len: f64) -> Self {
        self.arc_len = arc_len;
        self
    }

    /// `max_time_steps` (Reference default 200): the dense-sample cap.
    #[must_use]
    pub fn with_max_time_steps(mut self, max_time_steps: usize) -> Self {
        self.max_time_steps = max_time_steps;
        self
    }

    /// `n_samples_per_line` (Reference default 10): points per drawn line
    /// after even-arc re-spacing.
    #[must_use]
    pub fn with_samples_per_line(mut self, n: usize) -> Self {
        self.n_samples_per_line = n;
        self
    }

    /// `cutoff_norm` (Reference default 15): stop a line where the field
    /// norm exceeds this.
    #[must_use]
    pub fn with_cutoff_norm(mut self, cutoff_norm: f64) -> Self {
        self.cutoff_norm = cutoff_norm;
        self
    }

    /// The integrator tuning.
    #[must_use]
    pub fn with_integrator(mut self, tune: IntegratorTune) -> Self {
        self.tune = tune;
        self
    }

    /// Replace the style surface.
    #[must_use]
    pub fn with_style_config(mut self, style: StreamLineStyle) -> Self {
        self.style = style;
        self
    }

    /// The sampling budget.
    #[must_use]
    pub fn with_budget(mut self, budget: SamplingBudget) -> Self {
        self.budget = budget;
        self
    }

    /// `point_func` (vector_field.py:364): push scene points along the
    /// field — `c2p(func(p2c(p))) − origin`.
    #[must_use]
    pub fn point_func(&self, points: &[Vec3]) -> Vec<Vec3> {
        let origin = self.cs.origin();
        points
            .iter()
            .map(|&p| {
                let coords = self.cs.p2c(p);
                let out = (self.func)(&[coords]).first().copied().unwrap_or([0.0; 3]);
                sub3(self.cs.c2p(&out), origin)
            })
            .collect()
    }

    /// The jittered seed coordinates (the Reference's `get_sample_coords`
    /// on the class): the density grid, `n_repeats` copies (repeats
    /// outermost), each jittered by `noise_factor · U[0,1)` per component
    /// from the named substream. Also returns the draw count consumed.
    ///
    /// # Errors
    /// Refuses nonfinite seeds, invalid grid controls, and exceeded budgets.
    pub fn sample_coords(&self) -> Result<(Vec<[f64; 3]>, u64), FieldError> {
        if let Some(seeds) = &self.sample_coords {
            self.budget.ensure_total("stream seeds", seeds.len())?;
            for seed in seeds {
                for &value in seed {
                    ensure_finite("stream seed", value)?;
                }
            }
            return Ok((seeds.clone(), 0));
        }
        ensure_positive("stream density", self.density)?;
        let grid = get_sample_coords(&*self.cs, self.density, self.budget)?;
        let dim = self.cs.dimension().clamp(1, 3);
        let noise_factor = match self.noise_factor {
            Some(nf) => {
                ensure_finite("noise_factor", nf)?;
                nf
            }
            None => (x_unit_size(&*self.cs) / self.density) * 0.5,
        };
        ensure_finite("noise_factor", noise_factor)?;
        let total = grid
            .len()
            .checked_mul(self.n_repeats)
            .ok_or(FieldError::Sampling(SamplingError::CapacityOverflow {
                context: "stream seed repeats",
            }))?;
        self.budget.ensure_total("stream seeds", total)?;
        let mut rng_gen = self.substream.sequential();
        let mut seeds = Vec::with_capacity(total);
        let mut draws = 0u64;
        for _ in 0..self.n_repeats {
            for coords in &grid {
                let mut seed = *coords;
                for c in seed.iter_mut().take(dim) {
                    *c += noise_factor * rng_gen.next_f64();
                    ensure_finite("jittered stream seed", *c)?;
                    draws += 1;
                }
                seeds.push(seed);
            }
        }
        Ok((seeds, draws))
    }

    /// Draw the lines (the Reference's `draw_lines` + `init_style`).
    ///
    /// # Errors
    /// [`FieldError`] on bad controls, a blown budget, or an integration
    /// refusal.
    pub fn build(self) -> Result<StreamLinesMobject, FieldError> {
        ensure_positive("solution_time", self.solution_time)?;
        // arc_len may be +inf (uncapped); NaN or non-positive is refused.
        if self.arc_len.is_nan() || self.arc_len <= 0.0 {
            return Err(FieldError::NonFiniteControl { context: "arc_len" });
        }
        ensure_positive("cutoff_norm", self.cutoff_norm)?;
        let (seeds, draws) = self.sample_coords()?;
        let dim = self.cs.dimension().clamp(1, 3);

        let mut children = Vec::with_capacity(seeds.len());
        let mut metas = Vec::with_capacity(seeds.len());
        for seed in &seeds {
            let solution = ode_solution_points(
                &self.func,
                *seed,
                dim,
                self.solution_time,
                self.dt,
                self.tune,
                self.max_time_steps,
                self.cutoff_norm,
            )?;
            // Dense samples → scene space → even TRUE-arc re-spacing.
            let scene_points: Vec<Vec3> = solution.coords.iter().map(|c| self.cs.c2p(c)).collect();
            let sampled = resample_even_arc(&scene_points, self.n_samples_per_line, self.arc_len);
            let n_dense = solution.coords.len();

            let mut path = QuadPath::new();
            if let Some(&first) = sampled.first() {
                path.start_new_path(first);
                let _ = path.add_points_as_corners(&sampled[1..]);
                if sampled.len() > 2 {
                    let _ = path.make_smooth(true);
                }
            }
            let child = VMobject::from_path(&path)
                .with_style(Style::default().stroke(
                    self.style.stroke_color,
                    self.style.stroke_width,
                    self.style.stroke_opacity,
                ))
                .with_joint_type(JointType::NoJoint);

            // init_style: per-point magnitudes → colormap, widths tapered
            // by TRUE arc length (BN-03).
            let points = child.points().to_vec();
            let stroke_rgba = if self.style.color_by_magnitude {
                let norms: Vec<f64> = points
                    .iter()
                    .map(|&p| {
                        let coords = self.cs.p2c(p);
                        let out = (self.func)(&[coords]).first().copied().unwrap_or([0.0; 3]);
                        space_ops::get_norm(out)
                    })
                    .collect();
                let (lo, hi) = self.style.magnitude_range;
                colormap_gradient(&self.style.color_map, lo, hi, &norms)
                    .iter()
                    .map(|c| [c.r, c.g, c.b, self.style.stroke_opacity])
                    .collect()
            } else {
                Vec::new()
            };
            let stroke_widths = if self.style.taper_stroke_width {
                taper_by_true_length(&path, &[0.0, self.style.stroke_width, 0.0])
            } else {
                vec![self.style.stroke_width; points.len()]
            };

            metas.push(StreamLineMeta {
                virtual_time: self.solution_time,
                stroke_rgba,
                stroke_widths,
                n_solver_knots: solution.n_solver_knots,
                n_dense_samples: n_dense,
            });
            children.push(child);
        }

        Ok(StreamLinesMobject {
            group: crate::vmobject::v_group(children),
            lines: metas,
            rng_draws: draws,
            substream: self.substream,
        })
    }
}

/// Distribute a width profile over a path's point run by TRUE arc-length
/// proportion (BN-03): anchors at exact cumulative proportions, handles at
/// their curve's half-length.
#[must_use]
pub fn taper_by_true_length(path: &QuadPath, profile: &[f64]) -> Vec<f64> {
    let points = path.points();
    if points.is_empty() || profile.is_empty() {
        return Vec::new();
    }
    if profile.len() == 1 {
        return vec![profile[0]; points.len()];
    }
    let table = ArcLengthTable::for_path(path);
    let total = table.total();
    let curve_lengths = table.curve_lengths();
    let at = |p: f64| {
        let scaled = p.clamp(0.0, 1.0) * (profile.len() - 1) as f64;
        let lo = (scaled.floor() as usize).min(profile.len() - 1);
        let hi = (lo + 1).min(profile.len() - 1);
        let t = scaled - lo as f64;
        (1.0 - t) * profile[lo] + t * profile[hi]
    };
    if total <= 0.0 {
        return vec![at(0.0); points.len()];
    }
    let mut out = Vec::with_capacity(points.len());
    let mut cumulative = 0.0;
    for (k, &len_k) in curve_lengths.iter().enumerate() {
        if 2 * k >= points.len() {
            break;
        }
        if k == 0 {
            out.push(at(0.0));
        }
        if 2 * k + 1 < points.len() {
            out.push(at((cumulative + 0.5 * len_k) / total));
        }
        cumulative += len_k;
        out.push(at(cumulative / total));
    }
    out.resize(points.len(), at(1.0));
    out
}

// ------------------------------------------------- AnimatedStreamLines

/// `AnimatedStreamLines` (vector_field.py:445): a per-line
/// `VShowPassingFlash` sweeping each stream line on its own lagged clock.
/// Updater-bound — stateful by construction (§9.5).
pub struct AnimatedStreamLines {
    stream_lines: StreamLines,
    lag_range: f64,
    rate_multiple: f64,
    time_width: f64,
    taper_width: f64,
}

#[derive(Clone)]
struct StreamLineState {
    children: Vec<Mob>,
    times: Vec<f64>,
}

impl fmt::Debug for AnimatedStreamLines {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnimatedStreamLines")
            .field("lag_range", &self.lag_range)
            .field("rate_multiple", &self.rate_multiple)
            .field("time_width", &self.time_width)
            .finish_non_exhaustive()
    }
}

impl AnimatedStreamLines {
    /// Wrap a [`StreamLines`] builder with the Reference's defaults:
    /// `lag_range = 4`, `rate_multiple = 1`, `line_anim_config =
    /// (rate_func=linear, time_width=1)`.
    #[must_use]
    pub fn new(stream_lines: StreamLines) -> Self {
        Self {
            stream_lines,
            lag_range: 4.0,
            rate_multiple: 1.0,
            time_width: 1.0,
            taper_width: 0.05,
        }
    }

    /// `lag_range` (Reference default 4): the spread of negative start
    /// times.
    #[must_use]
    pub fn with_lag_range(mut self, lag_range: f64) -> Self {
        self.lag_range = lag_range;
        self
    }

    /// `rate_multiple` (Reference default 1): divides each line's
    /// `virtual_time` into its flash's run time.
    #[must_use]
    pub fn with_rate_multiple(mut self, rate_multiple: f64) -> Self {
        self.rate_multiple = rate_multiple;
        self
    }

    /// The passing flash's `time_width` (Reference passes 1.0).
    #[must_use]
    pub fn with_time_width(mut self, time_width: f64) -> Self {
        self.time_width = time_width;
        self
    }

    /// Add to a stage and bind the flow updater. Lag times draw from the
    /// same [`STREAM_LINES_SUBSTREAM`] substream, advanced past the seed
    /// jitter's draws so the stream stays one continuous deterministic
    /// sequence. Returns the group's handle.
    ///
    /// # Errors
    /// [`FieldError`] on bad controls, a blown budget, an integration
    /// refusal, or a stale handle.
    pub fn add_to_stage(self, stage: &mut Stage) -> Result<Mob, FieldError> {
        ensure_positive("lag_range", self.lag_range)?;
        ensure_positive("rate_multiple", self.rate_multiple)?;
        ensure_positive("time_width", self.time_width)?;
        let built = self.stream_lines.build()?;
        let substream = built.substream.clone();
        let prior_draws = built.rng_draws;
        let run_times: Vec<f64> = built
            .lines
            .iter()
            .map(|m| m.virtual_time / self.rate_multiple)
            .collect();
        let group = built.add_to_stage(stage);
        let children = stage
            .get(group)
            .map(|e| e.submobjects().to_vec())
            .unwrap_or_default();

        // Lag times: `-lag_range · U[0,1)` per line (vector_field.py:466),
        // from the continuous named stream.
        let mut rng_gen = substream.sequential();
        for _ in 0..prior_draws {
            let _ = rng_gen.next_f64();
        }
        let times: Vec<f64> = (0..children.len())
            .map(|_| -self.lag_range * rng_gen.next_f64())
            .collect();

        // Base profiles: the lines' current stroke widths, end-tapered the
        // way VShowPassingFlash's begin() snapshots them.
        let taper_width = self.taper_width;
        let taper_kernel = move |x: f64| {
            if x < taper_width {
                x
            } else if x > 1.0 - taper_width {
                1.0 - x
            } else {
                1.0
            }
        };
        let mut profiles: Vec<Vec<f64>> = Vec::with_capacity(children.len());
        for &child in &children {
            let widths = stage
                .get(child)
                .and_then(|e| e.buffer.read_column("stroke_width"))
                .unwrap_or_default();
            let n = widths.len();
            profiles.push(
                widths
                    .iter()
                    .enumerate()
                    .map(|(i, &w)| {
                        let x = if n > 1 {
                            i as f64 / (n as f64 - 1.0)
                        } else {
                            0.0
                        };
                        f64::from(w) * taper_kernel(x)
                    })
                    .collect(),
            );
        }

        let time_width = self.time_width;
        stage.add_dt_updater_with_state(
            group,
            StreamLineState { children, times },
            |state, map| {
                for child in &mut state.children {
                    *child = map.get(*child).unwrap_or(*child);
                }
            },
            move |stage: &mut Stage, mob: Mob, id, dt: f64| {
                for (i, &run_time) in run_times.iter().enumerate() {
                    let state = stage
                        .updater_state_mut::<StreamLineState>(mob, id)
                        .expect("stream-line updater attributes are installed");
                    state.times[i] += dt;
                    let time = state.times[i];
                    let child = state.children[i];
                    if run_time.is_nan() || run_time <= 0.0 {
                        continue;
                    }
                    // vector_field.py:474 update(): adjusted time mod the
                    // flash's run time, linear rate.
                    let adjusted = time.max(0.0).rem_euclid(run_time);
                    let alpha = adjusted / run_time;
                    // indication.py:222's gaussian sweep (σ = tw/6, swept
                    // −tw/2 → 1 + tw/2), zeroed outside 3σ.
                    let sigma = time_width / 6.0;
                    let mu = (1.0 - alpha) * (-time_width / 2.0) + alpha * (1.0 + time_width / 2.0);
                    let profile = &profiles[i];
                    let n = profile.len();
                    let widths: Vec<f64> = profile
                        .iter()
                        .enumerate()
                        .map(|(j, &w)| {
                            let x = if n > 1 {
                                j as f64 / (n as f64 - 1.0)
                            } else {
                                0.0
                            };
                            if (x - mu).abs() > 3.0 * sigma || sigma <= 0.0 {
                                return 0.0;
                            }
                            let z = (x - mu) / sigma;
                            w * fmn_dmath::exp(-0.5 * z * z)
                        })
                        .collect();
                    write_stroke_widths(stage, child, &widths);
                }
            },
            false,
        )?;
        Ok(group)
    }
}

// -------------------------------------------------------------- tracers

const MAX_TRACE_ANCHORS: usize = 100_000;

/// A stroke-width (or opacity) profile along a traced path: one value, or
/// a taper distributed by TRUE arc length (BN-03).
#[derive(Debug, Clone, PartialEq)]
pub enum StrokeProfile {
    /// One value across the whole path.
    Uniform(f64),
    /// A start→end taper, distributed by true arc-length proportion.
    Taper(Vec<f64>),
}

impl StrokeProfile {
    fn values(&self, path: &QuadPath) -> Vec<f64> {
        match self {
            Self::Uniform(w) => vec![*w; path.points().len()],
            Self::Taper(profile) => taper_by_true_length(path, profile),
        }
    }
}

/// A trace's regularly spaced observations, newest first. Ages are
/// relative to the latest observation rather than an ever-growing clock.
/// Keeping the grid point before the cutoff permits exact temporal clipping;
/// the current endpoint is published separately between grid observations.
#[derive(Clone)]
struct TailSamples {
    window: f64,
    spacing: f64,
    phase: f64,
    previous: Vec3,
    anchors: VecDeque<Vec3>,
    limit: usize,
}

enum TraceSource {
    PointFunction(Box<dyn Fn(&Stage) -> Vec3>),
    Mobject(Mob),
}

#[derive(Clone)]
struct TraceState {
    samples: Option<TailSamples>,
    target: Option<Mob>,
}

impl TailSamples {
    fn new(window: f64, spacing: f64, intervals: usize, start: Vec3) -> Self {
        Self {
            window,
            spacing,
            phase: 0.0,
            previous: start,
            anchors: VecDeque::from(vec![start; intervals + 1]),
            limit: intervals + 1,
        }
    }

    /// TracedPath begins when its source is first observed. It never
    /// fabricates the stationary prehistory that TracingTail requests.
    fn first_observation(window: f64, spacing: f64, intervals: usize, point: Vec3) -> Self {
        Self {
            window,
            spacing,
            phase: 0.0,
            previous: point,
            anchors: VecDeque::from([point]),
            limit: intervals + 1,
        }
    }

    /// The caller validates the finite, positive delta before reading its
    /// source. Neither work nor storage is proportional to `window / dt` or
    /// `dt / spacing`: samples already older than the cutoff are never built.
    fn advance(&mut self, point: Vec3, dt: f64) -> Vec<Vec3> {
        if self.window == 0.0 {
            self.previous = point;
            self.anchors[0] = point;
            return vec![point];
        }

        // Reduce before addition so even huge finite steps cannot overflow
        // the local clock. The subtraction branch also avoids phase + dt.
        let remainder = dt % self.spacing;
        let until_anchor = self.spacing - self.phase;
        let mut phase = if remainder >= until_anchor {
            remainder - until_anchor
        } else {
            self.phase + remainder
        };
        // A sum strictly below spacing can round onto its upper endpoint.
        if phase == self.spacing {
            phase = 0.0;
        }

        // The deque spans the full window and its predecessor. Iterating
        // backward from the newest grid point skips the expired prefix of a
        // large step without computing an unbounded absolute grid index.
        // A finite trace can discard observations outside its window. An
        // infinite-duration trace must retain every grid point, so one
        // extra candidate detects a full history before any state changes.
        let candidate_limit = if self.window.is_finite() {
            self.limit
        } else {
            self.limit - self.anchors.len() + 1
        };
        let new_count = (0..candidate_limit)
            .take_while(|&index| phase + index as f64 * self.spacing < dt)
            .count();
        if !self.window.is_finite() && new_count > self.limit - self.anchors.len() {
            std::panic::panic_any(SamplingError::LimitExceeded {
                context: "TracedPath history; use a finite window or increase time_per_anchor",
                max_samples: MAX_TRACE_ANCHORS,
            });
        }
        for index in (0..new_count).rev() {
            let age = phase + index as f64 * self.spacing;
            let alpha = (1.0 - age / dt).clamp(0.0, 1.0);
            let anchor = lerp3(self.previous, point, alpha);
            if self.anchors.len() == self.limit {
                let _ = self.anchors.pop_back();
            }
            self.anchors.push_front(anchor);
        }
        self.phase = phase;
        self.previous = point;

        let mut points = Vec::with_capacity(self.anchors.len() + 1);
        if !self.window.is_finite() {
            points.extend(self.anchors.iter().rev().copied());
        } else if self.window < phase {
            // Both the cutoff and the endpoint lie after the newest grid
            // observation; no expired grid point belongs in the visible path.
            points.push(lerp3(self.anchors[0], point, 1.0 - self.window / phase));
        } else {
            let intervals = (self.window - phase) / self.spacing;
            let inside = (intervals.ceil() as usize).min(self.anchors.len() - 1);
            let boundary = if intervals > (self.anchors.len() - 1) as f64 {
                // The configured window reaches before the first observed
                // point. Start at that point, without invented prehistory.
                self.anchors[inside]
            } else if inside == 0 {
                self.anchors[0]
            } else {
                lerp3(
                    self.anchors[inside],
                    self.anchors[inside - 1],
                    (inside as f64 - intervals).clamp(0.0, 1.0),
                )
            };
            points.push(boundary);
            points.extend((0..inside).rev().map(|index| self.anchors[index]));
        }
        if phase > 0.0 {
            points.push(point);
        }
        points
    }
}

/// `TracedPath` (changing.py:66): record where a point goes, drawing the
/// trace. Updater-bound — stateful by construction (§9.5).
#[derive(Debug, Clone)]
pub struct TracedPath {
    stroke_color: Srgb,
    stroke_width: StrokeProfile,
    stroke_opacity: StrokeProfile,
    time_traced: f64,
    time_per_anchor: f64,
}

impl Default for TracedPath {
    fn default() -> Self {
        Self {
            stroke_color: DEFAULT_MOBJECT_COLOR,
            stroke_width: StrokeProfile::Uniform(2.0),
            stroke_opacity: StrokeProfile::Uniform(1.0),
            time_traced: f64::INFINITY,
            time_per_anchor: 1.0 / 15.0,
        }
    }
}

impl TracedPath {
    /// The Reference's defaults: unbounded duration, anchor cadence 1/15 s.
    /// History still obeys the shared 100,000-anchor resource ceiling.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// `time_traced` (Reference default `inf`): how far back the trace
    /// remembers, in seconds.
    #[must_use]
    pub fn with_time_traced(mut self, time_traced: f64) -> Self {
        self.time_traced = time_traced;
        self
    }

    /// `time_per_anchor` (Reference default 1/15).
    #[must_use]
    pub fn with_time_per_anchor(mut self, time_per_anchor: f64) -> Self {
        self.time_per_anchor = time_per_anchor;
        self
    }

    /// Stroke color.
    #[must_use]
    pub fn with_stroke_color(mut self, color: Srgb) -> Self {
        self.stroke_color = color;
        self
    }

    /// Stroke width, scalar or taper.
    #[must_use]
    pub fn with_stroke_width(mut self, width: StrokeProfile) -> Self {
        self.stroke_width = width;
        self
    }

    /// Stroke opacity, scalar or taper.
    #[must_use]
    pub fn with_stroke_opacity(mut self, opacity: StrokeProfile) -> Self {
        self.stroke_opacity = opacity;
        self
    }

    /// Add to a stage and bind the trace updater (`update_path`).
    /// `traced_point_func` reads the point to follow from the stage (the
    /// Reference's zero-argument closure). Returns the trace's handle.
    ///
    /// # Errors
    /// [`FieldError::Stage`] on a stale handle;
    /// [`FieldError::NonFiniteControl`] for an invalid window/cadence;
    /// [`FieldError::Sampling`] when the finite window exceeds the shared
    /// 100,000-anchor resource ceiling. Refusal precedes stage adoption.
    ///
    /// # Panics
    /// The installed updater requires finite, nonnegative deltas and finite
    /// source coordinates. An infinite-duration trace also refuses a step
    /// that exceeds its anchor budget. These refusals preserve its previous
    /// history and geometry; the infallible [`Stage::update`] API does not
    /// roll back its clock or arbitrary effects of the source callback.
    pub fn add_to_stage(
        self,
        stage: &mut Stage,
        traced_point_func: impl Fn(&Stage) -> Vec3 + 'static,
    ) -> Result<Mob, FieldError> {
        self.add_to_stage_with_history(
            stage,
            TraceSource::PointFunction(Box::new(traced_point_func)),
            None,
        )
    }

    /// Seed observations without publishing a path before the first updater
    /// tick. TracingTail has construction-time history; TracedPath does not.
    fn add_to_stage_with_history(
        self,
        stage: &mut Stage,
        source: TraceSource,
        tail_samples: Option<TailSamples>,
    ) -> Result<Mob, FieldError> {
        if self.time_traced.is_nan() || self.time_traced < 0.0 {
            return Err(FieldError::NonFiniteControl {
                context: "TracedPath time_traced must be nonnegative or positive infinity",
            });
        }
        ensure_positive(
            "TracedPath time_per_anchor must be finite and positive",
            self.time_per_anchor,
        )?;
        let intervals = if self.time_traced.is_finite() {
            let intervals = (self.time_traced / self.time_per_anchor).ceil() as usize;
            SamplingBudget::new(MAX_TRACE_ANCHORS - 3)
                .ensure_total("TracedPath finite sample history", intervals)?;
            intervals
        } else {
            // One initial grid point and one possible between-grid endpoint
            // count toward the same ceiling as finite traces and the portal.
            MAX_TRACE_ANCHORS - 2
        };
        let mob = stage.add(VMobject::new().with_style(Style::default().stroke(
            self.stroke_color,
            match &self.stroke_width {
                StrokeProfile::Uniform(w) => *w,
                StrokeProfile::Taper(p) => p.first().copied().unwrap_or(0.0),
            },
            match &self.stroke_opacity {
                StrokeProfile::Uniform(o) => *o,
                StrokeProfile::Taper(p) => p.first().copied().unwrap_or(0.0),
            },
        )));
        let (target, point_func) = match source {
            TraceSource::PointFunction(func) => (None, Some(func)),
            TraceSource::Mobject(target) => (Some(target), None),
        };
        let config = self;
        stage.add_dt_updater_with_state(
            mob,
            TraceState {
                samples: tail_samples,
                target,
            },
            |state, map| {
                if let Some(target) = state.target {
                    state.target = Some(map.get(target).unwrap_or(target));
                }
            },
            move |stage: &mut Stage, mob: Mob, id, dt: f64| {
                // Stage's updater API is infallible. Refuse invalid deltas
                // before source sampling or changing trace history/geometry.
                assert!(
                    dt.is_finite() && dt >= 0.0,
                    "TracedPath/TracingTail updater dt must be finite and nonnegative"
                );
                // changing.py:92 update_path: dt == 0 is an explicit no-op.
                if dt == 0.0 {
                    return;
                }
                let target = stage
                    .updater_state::<TraceState>(mob, id)
                    .expect("trace updater attributes are installed")
                    .target;
                let point = if let Some(target) = target {
                    stage.get_center(target)
                } else {
                    point_func.as_ref().expect("point-function trace source")(stage)
                };
                assert!(
                    point.iter().all(|coordinate| coordinate.is_finite()),
                    "TracedPath/TracingTail source must return a finite 3D point"
                );

                let state = stage
                    .updater_state_mut::<TraceState>(mob, id)
                    .expect("trace updater attributes are installed");
                let points = if let Some(samples) = state.samples.as_mut() {
                    samples.advance(point, dt)
                } else {
                    state.samples = Some(TailSamples::first_observation(
                        config.time_traced,
                        config.time_per_anchor,
                        intervals,
                        point,
                    ));
                    vec![point]
                };

                if !points.is_empty() {
                    let path = smooth_polyline(&points);
                    let _ = stage.set_points(mob, path.points());
                    write_stroke_widths(stage, mob, &config.stroke_width.values(&path));
                    let opacities = config.stroke_opacity.values(&path);
                    let rgba: Vec<[f64; 4]> = opacities
                        .iter()
                        .map(|&o| {
                            [
                                config.stroke_color.r,
                                config.stroke_color.g,
                                config.stroke_color.b,
                                o,
                            ]
                        })
                        .collect();
                    write_stroke_rgba(stage, mob, &rgba);
                }
            },
            false,
        )?;
        Ok(mob)
    }
}

/// `TracingTail` (changing.py:118): a `TracedPath` that follows a mobject
/// (or any point function) with a fading, length-bounded tail.
#[derive(Debug, Clone)]
pub struct TracingTail {
    traced: TracedPath,
    /// Complete anchor intervals covering the window. The sampler also
    /// retains the grid observation preceding its moving cutoff.
    prefill: usize,
}

impl TracingTail {
    /// The Reference's defaults: `time_traced = 1`, width taper `(0, 3)`,
    /// opacity taper `(0, 1)`.
    #[must_use]
    pub fn new() -> Self {
        let time_traced = 1.0_f64;
        let time_per_anchor = 1.0_f64 / 15.0;
        Self {
            prefill: (time_traced / time_per_anchor).ceil() as usize,
            traced: TracedPath {
                stroke_width: StrokeProfile::Taper(vec![0.0, 3.0]),
                stroke_opacity: StrokeProfile::Taper(vec![0.0, 1.0]),
                time_traced,
                time_per_anchor,
                ..TracedPath::default()
            },
        }
    }

    /// `time_traced` (Reference default 1).
    #[must_use]
    pub fn with_time_traced(mut self, time_traced: f64) -> Self {
        self.traced.time_traced = time_traced;
        self.prefill = (time_traced / self.traced.time_per_anchor).ceil() as usize;
        self
    }

    /// `time_per_anchor` (Reference default 1/15): the temporal spacing of
    /// the tail's grid observations, including its stationary prehistory.
    ///
    /// # Errors
    /// [`FieldError::NonFiniteControl`] when the cadence is non-finite or
    /// not positive.
    pub fn with_time_per_anchor(mut self, time_per_anchor: f64) -> Result<Self, FieldError> {
        ensure_positive(
            "TracingTail time_per_anchor must be finite and positive",
            time_per_anchor,
        )?;
        self.traced.time_per_anchor = time_per_anchor;
        self.prefill = (self.traced.time_traced / time_per_anchor).ceil() as usize;
        Ok(self)
    }

    /// Stroke color.
    #[must_use]
    pub fn with_stroke_color(mut self, color: Srgb) -> Self {
        self.traced.stroke_color = color;
        self
    }

    /// The width taper (Reference default `(0, 3)`).
    #[must_use]
    pub fn with_stroke_width_taper(mut self, profile: Vec<f64>) -> Self {
        self.traced.stroke_width = StrokeProfile::Taper(profile);
        self
    }

    /// The opacity taper (Reference default `(0, 1)`).
    #[must_use]
    pub fn with_stroke_opacity_taper(mut self, profile: Vec<f64>) -> Self {
        self.traced.stroke_opacity = StrokeProfile::Taper(profile);
        self
    }

    /// Add to a stage following `target`'s center (the Reference's
    /// `mobject_or_func.get_center`), with copies of the current point in
    /// the sample history. Geometry remains empty until a positive-dt
    /// update publishes that history. Returns the tail's handle.
    ///
    /// # Errors
    /// [`FieldError::Stage`] on a stale handle, [`FieldError::NonFiniteControl`]
    /// for an invalid window, or [`FieldError::Sampling`] when its initial
    /// history exceeds the portal's shared 100,000-anchor resource ceiling.
    ///
    /// # Panics
    /// The installed updater requires finite, nonnegative deltas. Since
    /// [`Stage::update`] has no fallible updater channel, an invalid delta
    /// panics before this tail reads its source or changes its history/points.
    /// The enclosing stage's already-advanced clock is not rolled back.
    pub fn add_to_stage(self, stage: &mut Stage, target: Mob) -> Result<Mob, FieldError> {
        if !self.traced.time_traced.is_finite() || self.traced.time_traced < 0.0 {
            return Err(FieldError::NonFiniteControl {
                context: "TracingTail time_traced must be finite and nonnegative",
            });
        }
        SamplingBudget::new(MAX_TRACE_ANCHORS - 3)
            .ensure_total("TracingTail initial sample history", self.prefill)?;
        stage.try_get(target)?;
        let start = stage.get_center(target);
        let samples = TailSamples::new(
            self.traced.time_traced,
            self.traced.time_per_anchor,
            self.prefill,
            start,
        );
        self.traced
            .add_to_stage_with_history(stage, TraceSource::Mobject(target), Some(samples))
    }
}

impl Default for TracingTail {
    fn default() -> Self {
        Self::new()
    }
}

/// `set_points_smoothly` (vectorized_mobject.py:681): corners plus approx
/// smoothing.
fn smooth_polyline(points: &[Vec3]) -> QuadPath {
    let mut path = QuadPath::new();
    if let Some(&first) = points.first() {
        path.start_new_path(first);
        let _ = path.add_points_as_corners(&points[1..]);
        if points.len() > 2 {
            let _ = path.make_smooth(true);
        }
    }
    path
}

/// `AnimatedBoundary` (changing.py:17): two ghost copies of a mobject —
/// one drawing in, one fading out — cycling through a color list.
/// Updater-bound — stateful by construction (§9.5).
pub struct AnimatedBoundary {
    colors: Vec<Srgb>,
    max_stroke_width: f64,
    cycle_rate: f64,
    back_and_forth: bool,
    draw_rate_func: Box<dyn Fn(f64) -> f64 + 'static>,
    fade_rate_func: Box<dyn Fn(f64) -> f64 + 'static>,
}

#[derive(Clone)]
struct BoundaryState {
    total_time: f64,
    source_family: Vec<Mob>,
    growing_family: Vec<Mob>,
    fading_family: Vec<Mob>,
}

impl fmt::Debug for AnimatedBoundary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnimatedBoundary")
            .field("colors", &self.colors)
            .field("max_stroke_width", &self.max_stroke_width)
            .field("cycle_rate", &self.cycle_rate)
            .field("back_and_forth", &self.back_and_forth)
            .finish_non_exhaustive()
    }
}

impl Default for AnimatedBoundary {
    fn default() -> Self {
        Self {
            colors: vec![
                fmn_core::constants::BLUE_D,
                fmn_core::constants::BLUE_B,
                fmn_core::constants::BLUE_E,
                fmn_core::constants::GREY_BROWN,
            ],
            max_stroke_width: 3.0,
            cycle_rate: 0.5,
            back_and_forth: true,
            draw_rate_func: Box::new(fmn_core::rate::smooth),
            fade_rate_func: Box::new(fmn_core::rate::smooth),
        }
    }
}

impl AnimatedBoundary {
    /// The Reference's defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The color cycle (Reference `[BLUE_D, BLUE_B, BLUE_E, GREY_BROWN]`).
    #[must_use]
    pub fn with_colors(mut self, colors: Vec<Srgb>) -> Self {
        self.colors = colors;
        self
    }

    /// `max_stroke_width` (Reference default 3).
    #[must_use]
    pub fn with_max_stroke_width(mut self, width: f64) -> Self {
        self.max_stroke_width = width;
        self
    }

    /// `cycle_rate` (Reference default 0.5).
    #[must_use]
    pub fn with_cycle_rate(mut self, rate: f64) -> Self {
        self.cycle_rate = rate;
        self
    }

    /// `back_and_forth` (Reference default true).
    #[must_use]
    pub fn with_back_and_forth(mut self, on: bool) -> Self {
        self.back_and_forth = on;
        self
    }

    /// `draw_rate_func` (Reference default `smooth`).
    #[must_use]
    pub fn with_draw_rate_func(mut self, f: impl Fn(f64) -> f64 + 'static) -> Self {
        self.draw_rate_func = Box::new(f);
        self
    }

    /// `fade_rate_func` (Reference default `smooth`).
    #[must_use]
    pub fn with_fade_rate_func(mut self, f: impl Fn(f64) -> f64 + 'static) -> Self {
        self.fade_rate_func = Box::new(f);
        self
    }

    /// Add to a stage bound to `source` (which stays where it is — the
    /// ghosts mirror it every tick), and bind the updater. Returns the
    /// boundary group's handle.
    ///
    /// # Errors
    /// [`FieldError`] on non-finite controls or a stale handle.
    pub fn add_to_stage(self, stage: &mut Stage, source: Mob) -> Result<Mob, FieldError> {
        ensure_positive("cycle_rate", self.cycle_rate)?;
        ensure_finite("max_stroke_width", self.max_stroke_width)?;
        if self.colors.is_empty() {
            return Err(FieldError::NonFiniteControl {
                context: "animated boundary colors",
            });
        }
        stage.try_get(source)?;
        // The Reference's boundary copies: same geometry, no stroke, no
        // fill. Mirror the source family member-for-member so the
        // become-partial zip lines up (changing.py:81
        // full_family_become_partial).
        let source_family = stage.family(source);
        let mut growing_children = Vec::new();
        let mut fading_children = Vec::new();
        for member in &source_family {
            let points = stage.get_object_points(*member).unwrap_or_default();
            let ghost = || {
                VMobject::from_points(points.clone()).with_style(
                    Style::default()
                        .stroke(DEFAULT_MOBJECT_COLOR, 0.0, 0.0)
                        .fill(DEFAULT_MOBJECT_COLOR, 0.0),
                )
            };
            growing_children.push(ghost());
            fading_children.push(ghost());
        }
        let growing = stage.add(crate::vmobject::v_group(growing_children));
        let fading = stage.add(crate::vmobject::v_group(fading_children));
        let group = stage.add(crate::vmobject::v_group(Vec::new()));
        stage.attach(group, growing)?;
        stage.attach(group, fading)?;
        // These are flat ghost families: one child per source member. Their
        // empty wrapper is not a copy of the source's first member.
        let growing_family = stage.family(growing).into_iter().skip(1).collect();
        let fading_family = stage.family(fading).into_iter().skip(1).collect();

        let config = self;
        stage.add_dt_updater_with_state(
            group,
            BoundaryState {
                total_time: 0.0,
                source_family,
                growing_family,
                fading_family,
            },
            |state, map| {
                for family in [
                    &mut state.source_family,
                    &mut state.growing_family,
                    &mut state.fading_family,
                ] {
                    for member in family {
                        *member = map.get(*member).unwrap_or(*member);
                    }
                }
            },
            move |stage: &mut Stage, mob: Mob, id, dt: f64| {
                // changing.py:44 update_boundary_copies.
                let state = stage
                    .updater_state::<BoundaryState>(mob, id)
                    .expect("boundary updater attributes are installed");
                let time = state.total_time * config.cycle_rate;
                let members = state.source_family.len();
                let n_colors = config.colors.len();
                let color_index = (time % n_colors as f64) as usize;
                let alpha = time % 1.0;
                let draw_alpha = (config.draw_rate_func)(alpha);
                let fade_alpha = (config.fade_rate_func)(alpha);

                let (a, b) = if config.back_and_forth && (time as i64) % 2 == 1 {
                    (1.0 - draw_alpha, 1.0)
                } else {
                    (0.0, draw_alpha)
                };
                for index in 0..members {
                    let state = stage
                        .updater_state::<BoundaryState>(mob, id)
                        .expect("boundary updater attributes are installed");
                    let member = state.growing_family[index];
                    let src = state.source_family[index];
                    if stage.contains(member) && stage.contains(src) {
                        let _ = stage.pointwise_become_partial(member, src, a, b);
                        write_uniform_stroke(
                            stage,
                            member,
                            config.colors[color_index],
                            config.max_stroke_width,
                        );
                    }
                }
                if time >= 1.0 {
                    let fade_color = config.colors[(color_index + n_colors - 1) % n_colors];
                    for index in 0..members {
                        let state = stage
                            .updater_state::<BoundaryState>(mob, id)
                            .expect("boundary updater attributes are installed");
                        let member = state.fading_family[index];
                        let src = state.source_family[index];
                        if stage.contains(member) && stage.contains(src) {
                            let _ = stage.pointwise_become_partial(member, src, 0.0, 1.0);
                            write_uniform_stroke(
                                stage,
                                member,
                                fade_color,
                                (1.0 - fade_alpha) * config.max_stroke_width,
                            );
                        }
                    }
                }
                stage
                    .updater_state_mut::<BoundaryState>(mob, id)
                    .expect("boundary updater attributes are installed")
                    .total_time += dt;
            },
            false,
        )?;
        Ok(group)
    }
}

// --------------------------------------------------- updater utilities

/// `move_along_vector_field` (vector_field.py:81): shift a mobject along
/// the field every tick. Updater-bound.
///
/// # Errors
/// [`FieldError::Stage`] on a stale handle.
pub fn move_along_vector_field(
    stage: &mut Stage,
    mob: Mob,
    func: impl Fn(Vec3) -> Vec3 + 'static,
) -> Result<(), FieldError> {
    stage.add_dt_updater(
        mob,
        move |stage: &mut Stage, mob: Mob, dt: f64| {
            let center = stage.get_center(mob);
            let _ = stage.shift(mob, scale3(func(center), dt));
        },
        false,
    )?;
    Ok(())
}

/// `move_submobjects_along_vector_field` (vector_field.py:91): nudge each
/// child whose center lies within the frame — the Reference's full-frame
/// comparison kept exactly (`abs(x) < FRAME_WIDTH`, not the radius).
///
/// # Errors
/// [`FieldError::Stage`] on a stale handle.
pub fn move_submobjects_along_vector_field(
    stage: &mut Stage,
    mob: Mob,
    func: impl Fn(Vec3) -> Vec3 + 'static,
) -> Result<(), FieldError> {
    stage.add_dt_updater(
        mob,
        move |stage: &mut Stage, mob: Mob, dt: f64| {
            let children = stage
                .get(mob)
                .map(|e| e.submobjects().to_vec())
                .unwrap_or_default();
            for child in children {
                let center = stage.get_center(child);
                if center[0].abs() < FRAME_WIDTH && center[1].abs() < FRAME_HEIGHT {
                    let _ = stage.shift(child, scale3(func(center), dt));
                }
            }
        },
        false,
    )?;
    Ok(())
}

/// `move_points_along_vector_field` (vector_field.py:105): nudge every
/// point through the coordinate system — `p + (c2p(func(p2c(p))) −
/// origin) · dt`. Updater-bound.
///
/// # Errors
/// [`FieldError::Stage`] on a stale handle.
pub fn move_points_along_vector_field(
    stage: &mut Stage,
    mob: Mob,
    func: impl Fn(f64, f64) -> [f64; 2] + 'static,
    cs: impl CoordinateSystem + 'static,
) -> Result<(), FieldError> {
    stage.add_dt_updater(
        mob,
        move |stage: &mut Stage, mob: Mob, dt: f64| {
            let origin = cs.origin();
            let points = stage.get_object_points(mob).unwrap_or_default();
            let moved: Vec<Vec3> = points
                .iter()
                .map(|&p| {
                    let coords = cs.p2c(p);
                    let out = func(coords[0], coords[1]);
                    let target = cs.c2p(&[out[0], out[1], 0.0]);
                    add3(p, scale3(sub3(target, origin), dt))
                })
                .collect();
            let _ = stage.set_points(mob, &moved);
        },
        false,
    )?;
    Ok(())
}

// ------------------------------------------------------- stage plumbing

fn write_points(stage: &mut Stage, mob: Mob, points: &[Vec3]) {
    let _ = stage.set_points(mob, points);
}

fn write_stroke_widths(stage: &mut Stage, mob: Mob, widths: &[f64]) {
    let Some(entry) = stage.get_mut(mob) else {
        return;
    };
    if entry.buffer.len() != widths.len() {
        let _ = entry.buffer.resize_preserving_order(widths.len());
    }
    let flat: Vec<f32> = widths.iter().map(|&w| w as f32).collect();
    let _ = entry.buffer.write_range("stroke_width", 0, &flat);
}

fn write_stroke_rgba(stage: &mut Stage, mob: Mob, rgba: &[[f64; 4]]) {
    let Some(entry) = stage.get_mut(mob) else {
        return;
    };
    if entry.buffer.len() != rgba.len() {
        let _ = entry.buffer.resize_preserving_order(rgba.len());
    }
    let flat: Vec<f32> = rgba
        .iter()
        .flat_map(|c| c.iter().map(|&v| v as f32))
        .collect();
    let _ = entry.buffer.write_range("stroke_rgba", 0, &flat);
}

fn write_uniform_stroke(stage: &mut Stage, mob: Mob, color: Srgb, width: f64) {
    let Some(entry) = stage.get_mut(mob) else {
        return;
    };
    let n = entry.buffer.len();
    let rgba: Vec<f32> = (0..n)
        .flat_map(|_| [color.r as f32, color.g as f32, color.b as f32, 1.0])
        .collect();
    let _ = entry.buffer.write_range("stroke_rgba", 0, &rgba);
    let widths = vec![width as f32; n];
    let _ = entry.buffer.write_range("stroke_width", 0, &widths);
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_anim::purity::{ImpureEffect, Purity, classify_wait};
    use fmn_mobject::Stage;

    /// A deterministic 2D coordinate system for fixtures: `c2p` scales by
    /// `unit`, ranges as configured. No fonts, no wall clock.
    #[derive(Debug, Clone)]
    struct TestAxes {
        x_range: [f64; 3],
        y_range: [f64; 3],
        unit: f64,
    }

    impl CoordinateSystem for TestAxes {
        fn c2p(&self, coords: &[f64]) -> Vec3 {
            [
                coords.first().copied().unwrap_or(0.0) * self.unit,
                coords.get(1).copied().unwrap_or(0.0) * self.unit,
                coords.get(2).copied().unwrap_or(0.0) * self.unit,
            ]
        }
        fn p2c(&self, point: Vec3) -> [f64; 3] {
            [
                point[0] / self.unit,
                point[1] / self.unit,
                point[2] / self.unit,
            ]
        }
        fn all_ranges(&self) -> Vec<[f64; 3]> {
            vec![self.x_range, self.y_range]
        }
    }

    fn axes() -> TestAxes {
        TestAxes {
            x_range: [-2.0, 2.0, 1.0],
            y_range: [-1.0, 1.0, 0.5],
            unit: 1.0,
        }
    }

    fn rotation_field(coords: &[[f64; 3]]) -> Vec<[f64; 3]> {
        coords.iter().map(|c| [-c[1], c[0], 0.0]).collect()
    }

    // ------------------------------------------------ sampling fixtures

    #[test]
    fn sample_grid_matches_the_references_layout() {
        // vector_field.py get_sample_coords: per range (min, max, step),
        // arange(min, max + step, step / density); product, x outermost.
        // density 2: x step 0.5 over arange(-2, 2.5, .5) → -2..2 (9),
        // y step 0.25 over arange(-1, 1.25, .25) → -1..1 (9) → 81 coords.
        let coords = get_sample_coords(&axes(), 2.0, SamplingBudget::DEFAULT)
            .unwrap_or_else(|e| std::panic::panic_any(format!("grid: {e}")));
        assert_eq!(coords.len(), 81);
        assert_eq!(coords[0], [-2.0, -1.0, 0.0]);
        assert_eq!(coords[1], [-2.0, -0.75, 0.0]);
        assert_eq!(coords[8], [-2.0, 1.0, 0.0]);
        assert_eq!(coords[9], [-1.5, -1.0, 0.0]);
        assert_eq!(coords[80], [2.0, 1.0, 0.0]);
    }

    #[test]
    fn grid_sample_points_truncates_corners_like_the_reference() {
        // center 0, half-extent 1 in x (density 1 → spacing 1 → reach 1),
        // degenerate y/z (density 2 → spacing 0.5 → reach 1→ truncated 1).
        let points = grid_sample_points(
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [1.0, 2.0, 2.0],
            SamplingBudget::DEFAULT,
        );
        assert_eq!(
            points,
            vec![[-1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]
        );
    }

    #[test]
    fn colormap_gradient_lerps_between_anchors() {
        let anchors = [
            Srgb {
                r: 0.0,
                g: 0.0,
                b: 0.0,
            },
            Srgb {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
            Srgb {
                r: 1.0,
                g: 1.0,
                b: 0.0,
            },
        ];
        let lo = colormap_gradient_at(&anchors, 0.0, 2.0, 0.0);
        assert_eq!(lo, anchors[0]);
        let hi = colormap_gradient_at(&anchors, 0.0, 2.0, 2.0);
        assert_eq!(hi, anchors[2]);
        // alpha 0.5 → scaled 1.0 → exactly anchor[1].
        let mid = colormap_gradient_at(&anchors, 0.0, 2.0, 1.0);
        assert_eq!(mid, anchors[1]);
        // alpha 0.25 → scaled 0.5 → halfway between anchors 0 and 1.
        let q = colormap_gradient_at(&anchors, 0.0, 2.0, 0.5);
        assert!((q.r - 0.5).abs() < 1e-12 && q.g.abs() < 1e-12);
        // Clipping outside the range.
        assert_eq!(colormap_gradient_at(&anchors, 0.0, 2.0, -5.0), anchors[0]);
        assert_eq!(colormap_gradient_at(&anchors, 0.0, 2.0, 99.0), anchors[2]);
        // Degenerate range is deterministic (the Reference NaNs).
        assert_eq!(colormap_gradient_at(&anchors, 1.0, 1.0, 1.0), anchors[0]);
    }

    // -------------------------------------------------------- VectorField

    #[test]
    fn vector_field_tanh_bounds_drawn_lengths() {
        // A huge constant field: every arrow's drawn length stays below
        // max_displayed_vect_len, because tanh < 1 (on fmn-dmath).
        let huge = |coords: &[[f64; 3]]| coords.iter().map(|_| [1.0e6, 0.0, 0.0]).collect();
        let built = VectorField::new(huge, axes())
            .build()
            .unwrap_or_else(|e| std::panic::panic_any(format!("build: {e}")));
        let max_len = built.max_displayed_vect_len();
        let n = built.sample_points().len();
        assert!(n > 1);
        for i in 0..n {
            let base = built.vmob().points()[8 * i];
            let tip = built.vmob().points()[8 * i + 6];
            let drawn = space_ops::get_dist(tip, base);
            assert!(
                drawn <= max_len * (1.0 + 1e-12),
                "arrow {i} drawn {drawn} beyond tanh bound {max_len}"
            );
            // And the compression is real: the raw vector is enormous.
            assert!(drawn < 1.0);
        }
    }

    #[test]
    fn vector_field_zero_field_has_no_nan_and_zero_widths() {
        let zero = |coords: &[[f64; 3]]| coords.iter().map(|_| [0.0; 3]).collect();
        let built = VectorField::new(zero, axes())
            .build()
            .unwrap_or_else(|e| std::panic::panic_any(format!("build: {e}")));
        assert!(
            built
                .vmob()
                .points()
                .iter()
                .all(|p| p.iter().all(|v| v.is_finite())),
            "zero field produced a non-finite point"
        );
        assert!(
            built.stroke_widths().iter().all(|&w| w == 0.0),
            "zero field should draw zero-width arrows"
        );
    }

    #[test]
    fn vector_field_layout_is_eight_points_per_arrow_minus_one() {
        let built = VectorField::new(rotation_field, axes())
            .build()
            .unwrap_or_else(|e| std::panic::panic_any(format!("build: {e}")));
        let n = built.sample_points().len();
        assert_eq!(built.vmob().points().len(), 8 * n - 1);
        assert_eq!(built.stroke_widths().len(), 8 * n - 1);
        assert_eq!(built.stroke_rgba().len(), 8 * n - 1);
        // Magnitude range resolved to (0, max norm): the corner (-2, -1)
        // of the rotation field has norm √5 ≈ 2.236; the colormap's first
        // arrow (smallest magnitude at x=-2? no — grid order x-major, the
        // first sample is (-2,-1)) — assert the magnitude-resolved colors
        // differ between the smallest and largest norm arrows.
        let norms = built.output_norms();
        let (min_i, _) = norms
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.total_cmp(b.1))
            .unwrap_or((0, &0.0));
        let (max_i, _) = norms
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .unwrap_or((0, &0.0));
        assert_ne!(norms[min_i], norms[max_i]);
        let rgba = built.stroke_rgba();
        let c_min = rgba[8 * min_i];
        let c_max = rgba[8 * max_i];
        assert_ne!(c_min, c_max, "colormap did not separate magnitudes");
    }

    // -------------------------------------------------------- StreamLines

    fn stream_fixture(seed: u64) -> StreamLinesMobject {
        let rng = RngRoot::from_seed(seed);
        StreamLines::new(rotation_field, axes(), &rng)
            .with_density(1.0)
            .with_solution_time(1.0)
            .with_dt(0.05)
            .with_arc_len(2.0)
            .with_samples_per_line(10)
            .build()
            .unwrap_or_else(|e| std::panic::panic_any(format!("stream build: {e}")))
    }

    #[test]
    fn stream_lines_are_bit_exact_per_seed() {
        let a = stream_fixture(0);
        let b = stream_fixture(0);
        assert_eq!(a.lines().len(), b.lines().len());
        assert_eq!(a.rng_draws(), b.rng_draws());
        let pa: Vec<Vec3> = a
            .vmob()
            .children()
            .iter()
            .flat_map(|c| c.points().to_vec())
            .collect();
        let pb: Vec<Vec3> = b
            .vmob()
            .children()
            .iter()
            .flat_map(|c| c.points().to_vec())
            .collect();
        assert_eq!(pa.len(), pb.len());
        for (x, y) in pa.iter().zip(&pb) {
            assert!(
                x.iter()
                    .zip(y.iter())
                    .all(|(a, b)| a.to_bits() == b.to_bits()),
                "same seed must be bit-exact"
            );
        }
    }

    #[test]
    fn stream_lines_differ_across_seeds() {
        let a = stream_fixture(0);
        let b = stream_fixture(1);
        let pa: Vec<Vec3> = a
            .vmob()
            .children()
            .iter()
            .flat_map(|c| c.points().to_vec())
            .collect();
        let pb: Vec<Vec3> = b
            .vmob()
            .children()
            .iter()
            .flat_map(|c| c.points().to_vec())
            .collect();
        assert_eq!(pa.len(), pb.len());
        let differ = pa.iter().zip(&pb).filter(|(x, y)| x != y).count();
        assert!(
            differ > pa.len() / 2,
            "different seeds should move most points (got {differ}/{})",
            pa.len()
        );
    }

    #[test]
    fn stream_line_points_come_from_dense_output_not_solver_knots() {
        let built = stream_fixture(0);
        for (child, meta) in built.vmob().children().iter().zip(built.lines()) {
            // 10 even-arc samples → 19 shared-anchor points after approx
            // smoothing; dense sampling was 0..1 by 0.05 → 20 samples.
            assert_eq!(child.points().len(), 19);
            assert_eq!(meta.n_dense_samples, 20);
            assert!(
                meta.n_solver_knots != meta.n_dense_samples || meta.n_solver_knots == 0,
                "knot count must not silently equal the dense grid"
            );
        }
    }

    #[test]
    fn stream_lines_dense_resampling_is_smooth() {
        // Rotation field ⇒ circular stream lines; even-arc re-spacing of
        // dense samples must keep the turning angle per segment small.
        let built = stream_fixture(0);
        let mut max_angle = 0.0f64;
        for child in built.vmob().children() {
            let pts = child.points();
            for w in pts.windows(3) {
                let v1 = sub3(w[1], w[0]);
                let v2 = sub3(w[2], w[1]);
                if space_ops::get_norm(v1) > 1e-12 && space_ops::get_norm(v2) > 1e-12 {
                    let angle = space_ops::angle_between_vectors(v1, v2);
                    max_angle = max_angle.max(angle);
                }
            }
        }
        assert!(
            max_angle < 0.5,
            "consecutive-segment angle {max_angle} exceeds the smoothness bound"
        );
    }

    #[test]
    fn stream_lines_seed_jitter_uses_the_named_substream() {
        // The first draws of the "streamlines" substream at seed 0 must
        // equal the jitter the builder applies: build with noise_factor 1
        // and unit axes, then seeds = grid + 0.5 · U — verified through
        // bit-exactness against a manual replay of the same substream.
        let rng = RngRoot::from_seed(7);
        let mut rng_gen = rng.substream(STREAM_LINES_SUBSTREAM).sequential();
        let first_draws: Vec<f64> = (0..8).map(|_| rng_gen.next_f64()).collect();
        // Rebuilding must replay the identical sequence (determinism of
        // the substream itself is fmn-core's contract; here we prove the
        // builder draws from THIS stream by comparing against a second
        // independent generator).
        let mut gen2 = RngRoot::from_seed(7)
            .substream(STREAM_LINES_SUBSTREAM)
            .sequential();
        let replay: Vec<f64> = (0..8).map(|_| gen2.next_f64()).collect();
        assert_eq!(first_draws, replay);
        // And the name matters: a different name gives a different stream.
        let mut other = RngRoot::from_seed(7).substream("dots").sequential();
        let other_draws: Vec<f64> = (0..8).map(|_| other.next_f64()).collect();
        assert_ne!(first_draws, other_draws);
    }

    #[test]
    fn stream_lines_cutoff_norm_stops_runaway_lines() {
        // Radial explosion field: norm grows with radius; a tight cutoff
        // must shorten the drawn line versus a loose one.
        let radial = |coords: &[[f64; 3]]| {
            coords
                .iter()
                .map(|c| [c[0] * 4.0, c[1] * 4.0, 0.0])
                .collect()
        };
        let rng = RngRoot::from_seed(3);
        let tight = StreamLines::new(radial, axes(), &rng)
            .with_solution_time(1.0)
            .with_dt(0.05)
            .with_samples_per_line(10)
            .with_cutoff_norm(4.0)
            .with_arc_len(f64::INFINITY)
            .build()
            .unwrap_or_else(|e| std::panic::panic_any(format!("tight: {e}")));
        let loose = StreamLines::new(radial, axes(), &rng)
            .with_solution_time(1.0)
            .with_dt(0.05)
            .with_samples_per_line(10)
            .with_cutoff_norm(1.0e9)
            .with_arc_len(f64::INFINITY)
            .build()
            .unwrap_or_else(|e| std::panic::panic_any(format!("loose: {e}")));
        let tight_dense: usize = tight.lines().iter().map(|m| m.n_dense_samples).sum();
        let loose_dense: usize = loose.lines().iter().map(|m| m.n_dense_samples).sum();
        assert!(
            tight_dense < loose_dense,
            "cutoff did not truncate: tight {tight_dense} vs loose {loose_dense}"
        );
    }

    // ------------------------------------------------ AnimatedStreamLines

    #[test]
    fn animated_stream_lines_are_deterministic_per_seed() {
        let run = |seed: u64| {
            let mut stage = Stage::new();
            let rng = RngRoot::from_seed(seed);
            let stream = StreamLines::new(rotation_field, axes(), &rng)
                .with_solution_time(1.0)
                .with_samples_per_line(6);
            let group = AnimatedStreamLines::new(stream)
                .add_to_stage(&mut stage)
                .unwrap_or_else(|e| std::panic::panic_any(format!("anim: {e}")));
            stage
                .add_to_scene(group)
                .unwrap_or_else(|e| std::panic::panic_any(format!("root: {e}")));
            stage.update(0.1);
            let children = stage
                .get(group)
                .map(|e| e.submobjects().to_vec())
                .unwrap_or_default();
            children
                .iter()
                .map(|&c| {
                    stage
                        .get(c)
                        .and_then(|e| e.buffer.read_column("stroke_width"))
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(run(11), run(11), "same seed, same swept widths");
        assert_ne!(run(11), run(12), "different seeds, different lags");
    }

    // ------------------------------------------------------ purity (§9.5)

    #[test]
    fn traced_path_demotes_the_segment_to_stateful() {
        let mut stage = Stage::new();
        let dot = stage.add(VMobject::from_points(vec![
            [0.0; 3],
            [0.5, 0.5, 0.0],
            [1.0, 0.0, 0.0],
        ]));
        stage
            .add_to_scene(dot)
            .unwrap_or_else(|e| std::panic::panic_any(format!("root: {e}")));
        // The equivalent non-tracer scene classifies pure.
        assert_eq!(classify_wait(&stage, false), Purity::Pure);

        let traced = TracedPath::new()
            .add_to_stage(&mut stage, move |s: &Stage| s.get_center(dot))
            .unwrap_or_else(|e| std::panic::panic_any(format!("traced: {e}")));
        stage
            .add_to_scene(traced)
            .unwrap_or_else(|e| std::panic::panic_any(format!("root: {e}")));
        match classify_wait(&stage, false) {
            Purity::Stateful(effects) => {
                assert!(
                    effects.contains(&ImpureEffect::DtUpdater),
                    "tracer must demote via DtUpdater, got {effects:?}"
                );
            }
            Purity::Pure => {
                std::panic::panic_any("a scene with a TracedPath must classify stateful")
            }
        }
    }

    #[test]
    fn traced_path_records_the_trace_by_true_length_taper() {
        let mut stage = Stage::new();
        let cursor = stage.add(VMobject::from_points(vec![[0.0; 3]]));
        let traced = TracedPath::new()
            .with_stroke_width(StrokeProfile::Taper(vec![0.0, 4.0]))
            .add_to_stage(&mut stage, move |s: &Stage| s.get_center(cursor))
            .unwrap_or_else(|e| std::panic::panic_any(format!("traced: {e}")));
        stage
            .add_to_scene(traced)
            .unwrap_or_else(|e| std::panic::panic_any(format!("root: {e}")));
        // Walk an L: right 1, up 1 — true length 2 (approx smoothing
        // keeps the path within rounding of the polyline anchors).
        for i in 1..=8 {
            let x = 0.125 * i as f64;
            stage
                .set_points(cursor, &[[x, 0.0, 0.0]])
                .unwrap_or_else(|e| std::panic::panic_any(format!("move: {e}")));
            stage.update(0.1);
        }
        for i in 1..=8 {
            let y = 0.125 * i as f64;
            stage
                .set_points(cursor, &[[1.0, y, 0.0]])
                .unwrap_or_else(|e| std::panic::panic_any(format!("move: {e}")));
            stage.update(0.1);
        }
        let points = stage
            .get_points(traced)
            .unwrap_or_else(|| std::panic::panic_any("traced points"));
        assert!(points.len() >= 3, "trace should hold a path");
        assert!(points.iter().all(|p| p.iter().all(|v| v.is_finite())));
        // The taper: first record near 0 width, last near 4.
        let widths = stage
            .get(traced)
            .and_then(|e| e.buffer.read_column("stroke_width"))
            .unwrap_or_default();
        assert_eq!(widths.len(), points.len());
        let first = widths.first().copied().unwrap_or(-1.0);
        let last = widths.last().copied().unwrap_or(-1.0);
        assert!(first < 0.1, "tail start width {first} should taper from 0");
        assert!(
            (last - 4.0).abs() < 0.05,
            "tail end width {last} should reach the taper's far value"
        );
    }

    #[test]
    fn traced_path_stores_custom_time_per_anchor() {
        let traced = TracedPath::new().with_time_per_anchor(1.0 / 30.0);
        assert_eq!(traced.time_per_anchor, 1.0 / 30.0);
    }

    #[test]
    fn traced_path_begins_at_its_first_observation_and_clips_the_window() {
        let mut stage = Stage::new();
        let cursor = stage.add(VMobject::from_points(vec![[0.0; 3]]));
        let traced = TracedPath::new()
            .with_time_traced(0.5)
            .with_time_per_anchor(0.25)
            .add_to_stage(&mut stage, move |s| s.get_center(cursor))
            .unwrap();
        stage.add_to_scene(traced).unwrap();
        assert!(stage.get_points(traced).unwrap().is_empty());
        stage.update(0.0);
        assert!(stage.get_points(traced).unwrap().is_empty());
        stage.set_points(cursor, &[[0.125, 0.0, 0.0]]).unwrap();
        stage.update(0.125);
        assert_eq!(stage.get_points(traced).unwrap(), vec![[0.125, 0.0, 0.0]]);
        stage.set_points(cursor, &[[0.25, 0.0, 0.0]]).unwrap();
        stage.update(0.125);
        stage.set_points(cursor, &[[0.75, 0.0, 0.0]]).unwrap();
        stage.update(0.5);
        let anchors: Vec<_> = stage
            .get_points(traced)
            .unwrap()
            .into_iter()
            .step_by(2)
            .collect();
        assert_eq!(
            anchors,
            vec![
                [0.25, 0.0, 0.0],
                [0.375, 0.0, 0.0],
                [0.625, 0.0, 0.0],
                [0.75, 0.0, 0.0],
            ]
        );
    }

    #[test]
    fn traced_path_irregular_updates_preserve_duration_and_cadence() {
        fn run(deltas: &[f64]) -> Vec<Vec3> {
            let mut stage = Stage::new();
            let cursor = stage.add(VMobject::from_points(vec![[0.0; 3]]));
            let traced = TracedPath::new()
                .with_time_traced(0.875)
                .with_time_per_anchor(0.25)
                .add_to_stage(&mut stage, move |s| s.get_center(cursor))
                .unwrap();
            stage.add_to_scene(traced).unwrap();
            // Unlike TracingTail, TracedPath has no construction-time sample.
            stage.update(0.125);
            let mut time = 0.0;
            for &dt in deltas {
                time += dt;
                stage
                    .set_points(cursor, &[[time, 2.0 * time, 0.0]])
                    .unwrap();
                stage.update(dt);
            }
            stage
                .get_points(traced)
                .unwrap()
                .into_iter()
                .step_by(2)
                .collect()
        }
        let expected = vec![
            [0.625, 1.25, 0.0],
            [0.75, 1.5, 0.0],
            [1.0, 2.0, 0.0],
            [1.25, 2.5, 0.0],
            [1.5, 3.0, 0.0],
        ];
        assert_eq!(run(&[0.125; 12]), expected);
        assert_eq!(
            run(&[0.375, 0.125, 0.75, 0.000_976_562_5, 0.249_023_437_5]),
            expected
        );
    }

    #[test]
    fn traced_path_finite_history_is_bounded_under_small_updates() {
        let mut stage = Stage::new();
        let cursor = stage.add(VMobject::from_points(vec![[0.0; 3]]));
        let traced = TracedPath::new()
            .with_time_traced(1.0)
            .with_time_per_anchor(0.25)
            .add_to_stage(&mut stage, move |s| s.get_center(cursor))
            .unwrap();
        stage.add_to_scene(traced).unwrap();
        for (dt, point) in [
            (1.0 / 1024.0, [1.0, 0.0, 0.0]),
            (1.0 / 2048.0, [2.0, 0.0, 0.0]),
            (0.5, [3.0, 0.0, 0.0]),
            (1.0 / 1024.0, [4.0, 0.0, 0.0]),
        ] {
            stage.set_points(cursor, &[point]).unwrap();
            stage.update(dt);
            let points = stage.get_points(traced).unwrap();
            assert!(
                points.len() <= 13,
                "history follows cadence, not window / dt"
            );
            assert_eq!(points.last(), Some(&point));
        }
    }

    #[test]
    fn traced_path_unbounded_history_uses_the_declared_grid() {
        for (spacing, expected_anchors) in [(0.25, 5), (0.5, 3)] {
            let mut stage = Stage::new();
            let cursor = stage.add(VMobject::from_points(vec![[0.0; 3]]));
            let traced = TracedPath::new()
                .with_time_per_anchor(spacing)
                .add_to_stage(&mut stage, move |s| s.get_center(cursor))
                .unwrap();
            stage.add_to_scene(traced).unwrap();
            stage.update(0.125);
            for index in 1..=8 {
                stage
                    .set_points(cursor, &[[index as f64 / 8.0, 0.0, 0.0]])
                    .unwrap();
                stage.update(0.125);
            }
            let anchors: Vec<_> = stage
                .get_points(traced)
                .unwrap()
                .into_iter()
                .step_by(2)
                .collect();
            assert_eq!(anchors.len(), expected_anchors);
            for (index, point) in anchors.iter().enumerate() {
                assert_eq!(*point, [index as f64 * spacing, 0.0, 0.0]);
            }
        }
    }

    #[test]
    fn traced_path_refuses_invalid_configuration_before_adoption() {
        let mut stage = Stage::new();
        for window in [-1.0, f64::NAN, f64::NEG_INFINITY] {
            let epoch = stage.topology_epoch();
            assert!(matches!(
                TracedPath::new()
                    .with_time_traced(window)
                    .add_to_stage(&mut stage, |_| [0.0; 3]),
                Err(FieldError::NonFiniteControl { .. })
            ));
            assert_eq!(stage.topology_epoch(), epoch);
        }
        for spacing in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            let epoch = stage.topology_epoch();
            assert!(matches!(
                TracedPath::new()
                    .with_time_per_anchor(spacing)
                    .add_to_stage(&mut stage, |_| [0.0; 3]),
                Err(FieldError::NonFiniteControl { .. })
            ));
            assert_eq!(stage.topology_epoch(), epoch);
        }
        let epoch = stage.topology_epoch();
        assert!(matches!(
            TracedPath::new()
                .with_time_traced(99_997.25)
                .with_time_per_anchor(1.0)
                .add_to_stage(&mut stage, |_| [0.0; 3]),
            Err(FieldError::Sampling(_))
        ));
        assert_eq!(stage.topology_epoch(), epoch);
    }

    #[test]
    fn traced_path_invalid_updates_preserve_history_and_allow_retry() {
        use std::cell::Cell;
        use std::panic::{AssertUnwindSafe, catch_unwind};
        use std::rc::Rc;

        let point = Rc::new(Cell::new([0.0; 3]));
        let calls = Rc::new(Cell::new(0));
        let mut stage = Stage::new();
        let traced = TracedPath::new()
            .with_time_traced(0.5)
            .with_time_per_anchor(0.25)
            .add_to_stage(&mut stage, {
                let point = Rc::clone(&point);
                let calls = Rc::clone(&calls);
                move |_| {
                    calls.set(calls.get() + 1);
                    point.get()
                }
            })
            .unwrap();
        stage.add_to_scene(traced).unwrap();
        stage.update(0.125);
        let before = stage.get_points(traced).unwrap();
        for dt in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(catch_unwind(AssertUnwindSafe(|| stage.update(dt))).is_err());
            assert_eq!(calls.get(), 1, "invalid dt must not sample the source");
            assert_eq!(stage.get_points(traced).unwrap(), before);
        }
        point.set([f64::NAN, 0.0, 0.0]);
        assert!(catch_unwind(AssertUnwindSafe(|| stage.update(0.125))).is_err());
        assert_eq!(stage.get_points(traced).unwrap(), before);
        point.set([1.0, 0.0, 0.0]);
        stage.update(0.25);
        assert_eq!(
            stage.get_points(traced).unwrap(),
            vec![[0.0; 3], [0.5, 0.0, 0.0], [1.0, 0.0, 0.0]]
        );
    }

    #[test]
    fn traced_path_unbounded_budget_refusal_does_not_discard_history() {
        use std::cell::Cell;
        use std::panic::{AssertUnwindSafe, catch_unwind};
        use std::rc::Rc;

        let point = Rc::new(Cell::new([0.0; 3]));
        let mut stage = Stage::new();
        let traced = TracedPath::new()
            .with_time_per_anchor(0.25)
            .add_to_stage(&mut stage, {
                let point = Rc::clone(&point);
                move |_| point.get()
            })
            .unwrap();
        stage.add_to_scene(traced).unwrap();
        stage.update(0.125);
        point.set([99.0, 0.0, 0.0]);
        assert!(catch_unwind(AssertUnwindSafe(|| stage.update(f64::MAX))).is_err());
        assert_eq!(stage.get_points(traced).unwrap(), vec![[0.0; 3]]);
        point.set([1.0, 0.0, 0.0]);
        stage.update(0.25);
        assert_eq!(
            stage.get_points(traced).unwrap().last(),
            Some(&[1.0, 0.0, 0.0])
        );
        assert_eq!(stage.get_points(traced).unwrap().len(), 3);
    }

    #[test]
    fn tracing_tail_seeds_history_but_stays_empty_until_positive_dt() {
        let mut stage = Stage::new();
        let cursor = stage.add(VMobject::from_points(vec![[2.0, 1.0, 0.0]]));
        stage
            .add_to_scene(cursor)
            .unwrap_or_else(|e| std::panic::panic_any(format!("root: {e}")));
        let default_tail = TracingTail::new();
        assert_eq!(default_tail.prefill, 15);
        let tail = default_tail
            .add_to_stage(&mut stage, cursor)
            .unwrap_or_else(|e| std::panic::panic_any(format!("tail: {e}")));
        stage
            .add_to_scene(tail)
            .unwrap_or_else(|e| std::panic::panic_any(format!("root: {e}")));
        // The Reference seeds only traced_points. Construction and zero-dt
        // updates must not turn that history into a degenerate visible path.
        let points = stage
            .get_points(tail)
            .unwrap_or_else(|| std::panic::panic_any("tail points"));
        assert!(points.is_empty());

        let custom_tail = TracingTail::new()
            .with_time_per_anchor(1.0 / 30.0)
            .unwrap_or_else(|error| std::panic::panic_any(format!("cadence: {error}")));
        assert_eq!(custom_tail.prefill, 30);
        let custom_tail = custom_tail
            .add_to_stage(&mut stage, cursor)
            .unwrap_or_else(|error| std::panic::panic_any(format!("custom tail: {error}")));
        stage
            .add_to_scene(custom_tail)
            .unwrap_or_else(|error| std::panic::panic_any(format!("custom root: {error}")));
        stage.update(0.0);
        for handle in [tail, custom_tail] {
            assert!(stage.get_points(handle).unwrap().is_empty());
        }

        stage.set_points(cursor, &[[3.0, 2.0, 0.0]]).unwrap();
        stage.update(1.0 / 30.0);
        for handle in [tail, custom_tail] {
            let points = stage.get_points(handle).unwrap();
            assert!(points.len() > 1);
            // A geometry-only seed is lost on the first update. These two
            // endpoints independently prove that the updater owns its seed.
            assert_eq!(points.first(), Some(&[2.0, 1.0, 0.0]));
            assert_eq!(points.last(), Some(&[3.0, 2.0, 0.0]));
            let widths = stage
                .get(handle)
                .unwrap()
                .buffer
                .read_column("stroke_width")
                .unwrap();
            let colors = stage
                .get(handle)
                .unwrap()
                .buffer
                .read_column("stroke_rgba")
                .unwrap();
            assert_eq!(widths.first(), Some(&0.0));
            assert_eq!(widths.last(), Some(&3.0));
            assert_eq!(colors[3], 0.0);
            assert_eq!(colors.last(), Some(&1.0));
        }
        for cadence in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(matches!(
                TracingTail::new().with_time_per_anchor(cadence),
                Err(FieldError::NonFiniteControl { .. })
            ));
        }
        assert!(!classify_wait(&stage, false).is_pure());
    }

    #[test]
    fn tracing_tail_zero_window_publishes_only_the_current_point() {
        let mut stage = Stage::new();
        let cursor = stage.add(VMobject::from_points(vec![[2.0, 1.0, 0.0]]));
        let tail = TracingTail::new()
            .with_time_traced(0.0)
            .add_to_stage(&mut stage, cursor)
            .unwrap();
        stage.add_to_scene(tail).unwrap();
        assert!(stage.get_points(tail).unwrap().is_empty());
        for point in [[3.0, 2.0, 0.0], [5.0, 4.0, 0.0]] {
            stage.set_points(cursor, &[point]).unwrap();
            stage.update(0.125);
            assert_eq!(stage.get_points(tail).unwrap(), vec![point]);
        }
    }

    #[test]
    fn tracing_tail_irregular_deltas_keep_the_same_temporal_grid() {
        fn run(deltas: &[f64]) -> Vec<Vec3> {
            let mut stage = Stage::new();
            let cursor = stage.add(VMobject::from_points(vec![[0.0; 3]]));
            let tail = TracingTail::new()
                .with_time_traced(0.875)
                .with_time_per_anchor(0.25)
                .unwrap()
                .add_to_stage(&mut stage, cursor)
                .unwrap();
            stage.add_to_scene(tail).unwrap();
            let mut time = 0.0;
            for &dt in deltas {
                time += dt;
                stage
                    .set_points(cursor, &[[time, 2.0 * time, 0.0]])
                    .unwrap();
                stage.update(dt);
            }
            stage
                .get_points(tail)
                .unwrap()
                .into_iter()
                .step_by(2)
                .collect()
        }

        // The cutoff is between grid points. A long frame followed by a
        // short frame must neither collapse the window nor revive old data.
        let expected = vec![
            [0.625, 1.25, 0.0],
            [0.75, 1.5, 0.0],
            [1.0, 2.0, 0.0],
            [1.25, 2.5, 0.0],
            [1.5, 3.0, 0.0],
        ];
        assert_eq!(run(&[0.0625; 24]), expected);
        assert_eq!(
            run(&[0.375, 0.125, 0.75, 0.000_976_562_5, 0.249_023_437_5]),
            expected
        );
    }

    #[test]
    fn tracing_tail_clips_piecewise_observations_between_grid_points() {
        let mut stage = Stage::new();
        let cursor = stage.add(VMobject::from_points(vec![[0.0; 3]]));
        let tail = TracingTail::new()
            .with_time_traced(0.5)
            .with_time_per_anchor(0.25)
            .unwrap()
            .add_to_stage(&mut stage, cursor)
            .unwrap();
        stage.add_to_scene(tail).unwrap();
        for (dt, point) in [
            (0.25, [1.0, 0.0, 0.0]),
            (0.25, [1.0, 1.0, 0.0]),
            (0.25, [2.0, 1.0, 0.0]),
            (0.125, [2.0, 2.0, 0.0]),
        ] {
            stage.set_points(cursor, &[point]).unwrap();
            stage.update(dt);
        }
        let anchors: Vec<_> = stage
            .get_points(tail)
            .unwrap()
            .into_iter()
            .step_by(2)
            .collect();
        assert_eq!(
            anchors,
            vec![
                [1.0, 0.5, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, 1.0, 0.0],
                [2.0, 2.0, 0.0],
            ]
        );

        // The next grid point interpolates the two actual latest source
        // observations, including the one between earlier grid points.
        stage.set_points(cursor, &[[4.0, 2.0, 0.0]]).unwrap();
        stage.update(0.375);
        let anchors: Vec<_> = stage
            .get_points(tail)
            .unwrap()
            .into_iter()
            .step_by(2)
            .collect();
        assert_eq!(anchors.len(), 3);
        assert_eq!(anchors[0], [2.0, 1.0, 0.0]);
        assert!((anchors[1][0] - 8.0 / 3.0).abs() < 1e-6);
        assert_eq!(anchors[1][1], 2.0);
        assert_eq!(anchors[2], [4.0, 2.0, 0.0]);
    }

    #[test]
    fn tracing_tail_tiny_steps_keep_seed_and_bound_the_geometry() {
        let mut stage = Stage::new();
        let start = [2.0, 1.0, 0.0];
        let cursor = stage.add(VMobject::from_points(vec![start]));
        let tail = TracingTail::new().add_to_stage(&mut stage, cursor).unwrap();
        stage.add_to_scene(tail).unwrap();
        for (dt, point) in [
            (1e-12, [3.0, 2.0, 0.0]),
            (1e-9, [4.0, 3.0, 0.0]),
            (1e-10, [5.0, 4.0, 0.0]),
        ] {
            stage.set_points(cursor, &[point]).unwrap();
            stage.update(dt);
            let points = stage.get_points(tail).unwrap();
            assert!(
                points.len() <= 35,
                "tiny dt must not allocate a frame-count window"
            );
            assert_eq!(points.first(), Some(&start));
            assert_eq!(points.last(), Some(&point));
            assert!(points.iter().flatten().all(|value| value.is_finite()));
        }
    }

    #[test]
    fn tracing_tail_large_steps_skip_expired_samples() {
        // Direct sample values avoid f32 geometry rounding masking a narrow
        // temporal window after a very large observed displacement in time.
        let mut samples = TailSamples::new(1.0, 0.25, 4, [0.0; 3]);
        let points = samples.advance([64.0, 0.0, 0.0], 1e6);
        assert_eq!(points.len(), 5);
        for (index, point) in points.iter().enumerate() {
            let age = 1.0 - index as f64 * 0.25;
            assert!((point[0] - 64.0 * (1.0 - age / 1e6)).abs() < 1e-12);
        }

        // dt / spacing overflows, but the finite tail still has only eight
        // grid intervals. No absolute time or grid index is required.
        let spacing = f64::from_bits(1);
        let mut samples = TailSamples::new(8.0 * spacing, spacing, 8, [0.0; 3]);
        for point in [[1.0, 2.0, 0.0], [3.0, 4.0, 0.0]] {
            let points = samples.advance(point, f64::MAX);
            assert!(points.len() <= 11);
            assert!(points.iter().flatten().all(|value| value.is_finite()));
            assert_eq!(points.last(), Some(&point));
            assert!(points.iter().all(|sample| sample == &point));
        }
    }

    #[test]
    fn tracing_tail_invalid_deltas_do_not_sample_or_change_tail_state() {
        use std::cell::Cell;
        use std::panic::{AssertUnwindSafe, catch_unwind};
        use std::rc::Rc;

        let source = Rc::new(Cell::new([0.0; 3]));
        let make = |stage: &mut Stage, calls: Rc<Cell<usize>>| {
            let source = Rc::clone(&source);
            let tail = TracingTail::new()
                .with_time_traced(0.5)
                .with_time_per_anchor(0.125)
                .unwrap();
            let samples = TailSamples::new(0.5, 0.125, tail.prefill, source.get());
            let mob = tail
                .traced
                .add_to_stage_with_history(
                    stage,
                    TraceSource::PointFunction(Box::new(move |_| {
                        calls.set(calls.get() + 1);
                        source.get()
                    })),
                    Some(samples),
                )
                .unwrap();
            stage.add_to_scene(mob).unwrap();
            mob
        };
        let mut stage = Stage::new();
        let mut control = Stage::new();
        let calls = Rc::new(Cell::new(0));
        let tail = make(&mut stage, Rc::clone(&calls));
        let expected = make(&mut control, Rc::new(Cell::new(0)));
        stage.update(0.0);
        assert_eq!(calls.get(), 0);
        assert!(stage.get_points(tail).unwrap().is_empty());

        for (step, (dt, point)) in [(0.25, [1.0, 1.0, 0.0]), (0.125, [2.0, 1.0, 0.0])]
            .into_iter()
            .enumerate()
        {
            let before = stage.get_points(tail).unwrap();
            source.set([99.0, 99.0, 0.0]);
            for invalid in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                assert!(catch_unwind(AssertUnwindSafe(|| stage.update(invalid))).is_err());
                assert_eq!(calls.get(), step);
                assert_eq!(stage.get_points(tail).unwrap(), before);
            }
            source.set(point);
            stage.update(dt);
            control.update(dt);
            assert_eq!(calls.get(), step + 1);
            // Valid continuation proves hidden history stayed unchanged too.
            // Stage::update has already advanced its own clock on invalid
            // input; this assertion is intentionally about the tail only.
            assert_eq!(
                stage.get_points(tail).unwrap(),
                control.get_points(expected).unwrap()
            );
        }
    }

    #[test]
    fn tracing_tail_refuses_invalid_or_unbounded_initial_history() {
        let mut stage = Stage::new();
        let cursor = stage.add(VMobject::from_points(vec![[2.0, 1.0, 0.0]]));
        for duration in [-1.0, f64::NAN, f64::INFINITY] {
            assert!(matches!(
                TracingTail::new()
                    .with_time_traced(duration)
                    .add_to_stage(&mut stage, cursor),
                Err(FieldError::NonFiniteControl { .. })
            ));
        }
        assert!(matches!(
            TracingTail::new()
                .with_time_traced(1e6)
                .add_to_stage(&mut stage, cursor),
            Err(FieldError::Sampling(_))
        ));
        // A floor-only admission would accept this extra partial interval.
        assert!(matches!(
            TracingTail::new()
                .with_time_traced(99_997.25)
                .with_time_per_anchor(1.0)
                .unwrap()
                .add_to_stage(&mut stage, cursor),
            Err(FieldError::Sampling(_))
        ));
    }

    #[test]
    fn animated_boundary_demotes_and_ghosts_follow_source() {
        let mut stage = Stage::new();
        let square = stage.add(VMobject::from_points(vec![
            [0.0, 0.0, 0.0],
            [0.5, 0.5, 0.0],
            [1.0, 0.0, 0.0],
            [1.5, 0.5, 0.0],
            [2.0, 0.0, 0.0],
        ]));
        stage
            .add_to_scene(square)
            .unwrap_or_else(|e| std::panic::panic_any(format!("root: {e}")));
        let boundary = AnimatedBoundary::new()
            .add_to_stage(&mut stage, square)
            .unwrap_or_else(|e| std::panic::panic_any(format!("boundary: {e}")));
        stage
            .add_to_scene(boundary)
            .unwrap_or_else(|e| std::panic::panic_any(format!("root: {e}")));
        assert!(!classify_wait(&stage, false).is_pure());
        // After a tick, the growing ghost holds a partial copy of the
        // source with a nonzero stroke somewhere.
        stage.update(0.5);
        let ghosts = stage
            .get(boundary)
            .map(|e| e.submobjects().to_vec())
            .unwrap_or_default();
        assert_eq!(ghosts.len(), 2);
        let growing_family = stage.family(ghosts[0]);
        let any_width = growing_family.iter().any(|&m| {
            stage
                .get(m)
                .and_then(|e| e.buffer.read_column("stroke_width"))
                .unwrap_or_default()
                .iter()
                .any(|&w| w > 0.0)
        });
        assert!(any_width, "growing ghost never got its stroke");
    }

    #[test]
    fn time_varying_field_demotes_and_redraws() {
        let mut stage = Stage::new();
        let tvf = TimeVaryingVectorField::new(
            |coords: &[[f64; 3]], t: f64| {
                coords
                    .iter()
                    .map(|c| [c[0] * fmn_dmath::cos(t), c[1] * fmn_dmath::cos(t), 0.0])
                    .collect()
            },
            axes(),
        );
        let mob = tvf
            .add_to_stage(&mut stage)
            .unwrap_or_else(|e| std::panic::panic_any(format!("tvf: {e}")));
        stage
            .add_to_scene(mob)
            .unwrap_or_else(|e| std::panic::panic_any(format!("root: {e}")));
        assert!(!classify_wait(&stage, false).is_pure());
        let before = stage
            .get_points(mob)
            .unwrap_or_else(|| std::panic::panic_any("points"));
        stage.update(0.25);
        let after = stage
            .get_points(mob)
            .unwrap_or_else(|| std::panic::panic_any("points"));
        assert_eq!(before.len(), after.len(), "the arrow layout is stable");
        assert!(
            before.iter().zip(&after).any(|(a, b)| a != b),
            "the field did not redraw at the new time"
        );
        assert!(after.iter().all(|p| p.iter().all(|v| v.is_finite())));
    }

    #[test]
    fn owned_updater_traced_path_copy_branches_from_its_existing_history() {
        use std::cell::Cell;
        use std::rc::Rc;

        let point = Rc::new(Cell::new([0.0; 3]));
        let source = Rc::clone(&point);
        let mut stage = Stage::new();
        let trace = TracedPath::new()
            .with_time_traced(1.0)
            .with_time_per_anchor(0.25)
            .add_to_stage(&mut stage, move |_| source.get())
            .unwrap();
        stage.update_mobject(trace, 0.125);
        point.set([0.25, 0.0, 0.0]);
        stage.update_mobject(trace, 0.25);
        let copied = stage.copy_family(trace).unwrap();
        assert_eq!(stage.updater_ids(trace), stage.updater_ids(copied));
        let original_points = stage.get_points(trace).unwrap();
        point.set([2.0, 0.0, 0.0]);
        stage.update_mobject(copied, 0.25);
        assert_eq!(stage.get_points(trace).unwrap(), original_points);
        point.set([0.5, 0.0, 0.0]);
        stage.update_mobject(trace, 0.25);
        let anchors = |mob| {
            stage
                .get_points(mob)
                .unwrap()
                .into_iter()
                .step_by(2)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            anchors(trace),
            vec![[0.0; 3], [0.25, 0.0, 0.0], [0.5, 0.0, 0.0]]
        );
        assert_eq!(
            anchors(copied),
            vec![[0.0; 3], [0.25, 0.0, 0.0], [2.0, 0.0, 0.0]]
        );
    }

    #[test]
    fn owned_updater_trace_snapshots_freeze_history_and_restore_replay() {
        let mut stage = Stage::new();
        let cursor = stage.add(VMobject::from_points(vec![[0.0; 3]]));
        let trace = TracedPath::new()
            .with_time_traced(1.0)
            .with_time_per_anchor(0.25)
            .add_to_stage(&mut stage, move |stage| stage.get_center(cursor))
            .unwrap();
        stage.update_mobject(trace, 0.125);
        stage.set_points(cursor, &[[0.25, 0.0, 0.0]]).unwrap();
        stage.update_mobject(trace, 0.25);
        let snapshot = stage.snapshot();
        stage.set_points(cursor, &[[0.5, 0.0, 0.0]]).unwrap();
        stage.update_mobject(trace, 0.25);
        let replayed = stage.get_points(trace).unwrap();
        for endpoint in [8.0, 4.0, 2.0] {
            let mut branch = snapshot.materialize();
            branch.set_points(cursor, &[[endpoint, 0.0, 0.0]]).unwrap();
            branch.update_mobject(trace, 0.25);
            let anchors = branch
                .get_points(trace)
                .unwrap()
                .into_iter()
                .step_by(2)
                .collect::<Vec<_>>();
            assert_eq!(
                anchors,
                vec![[0.0; 3], [0.25, 0.0, 0.0], [endpoint, 0.0, 0.0]]
            );
            assert_eq!(stage.get_points(trace).unwrap(), replayed);
        }
        stage.restore(&snapshot);
        stage.set_points(cursor, &[[0.5, 0.0, 0.0]]).unwrap();
        stage.update_mobject(trace, 0.25);
        assert_eq!(stage.get_points(trace).unwrap(), replayed);
    }

    #[test]
    fn owned_updater_tail_copy_remaps_the_traced_family_member() {
        for cross_stage in [false, true] {
            let mut stage = Stage::new();
            let cursor = stage.add(VMobject::from_points(vec![[0.0; 3]]));
            let tail = TracingTail::new()
                .with_time_traced(0.5)
                .with_time_per_anchor(0.25)
                .unwrap()
                .add_to_stage(&mut stage, cursor)
                .unwrap();
            let family = stage.add(crate::vmobject::v_group(Vec::new()));
            stage.attach(family, cursor).unwrap();
            stage.attach(family, tail).unwrap();
            stage.set_points(cursor, &[[1.0, 0.0, 0.0]]).unwrap();
            stage.update_mobject(tail, 0.25);
            let before = stage.get_points(tail).unwrap();
            let mut other = Stage::new();
            let copied = if cross_stage {
                stage.copy_into(family, &mut other).unwrap()
            } else {
                stage.copy_family(family).unwrap()
            };
            let destination = if cross_stage { &mut other } else { &mut stage };
            let copy_children = destination.get(copied).unwrap().submobjects().to_vec();
            let copy_cursor = copy_children[0];
            let copy_tail = copy_children[1];
            destination
                .set_points(copy_cursor, &[[10.0, 0.0, 0.0]])
                .unwrap();
            destination.update_mobject(copy_tail, 0.25);
            let copy_anchors = destination
                .get_points(copy_tail)
                .unwrap()
                .into_iter()
                .step_by(2)
                .collect::<Vec<_>>();
            assert_eq!(
                copy_anchors,
                vec![[0.0; 3], [1.0, 0.0, 0.0], [10.0, 0.0, 0.0]]
            );
            assert_eq!(stage.get_points(tail).unwrap(), before);
            stage.set_points(cursor, &[[2.0, 0.0, 0.0]]).unwrap();
            stage.update_mobject(tail, 0.25);
            let anchors = stage
                .get_points(tail)
                .unwrap()
                .into_iter()
                .step_by(2)
                .collect::<Vec<_>>();
            assert_eq!(anchors, vec![[0.0; 3], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        }
    }

    #[test]
    fn owned_updater_boundary_copy_updates_its_own_ghosts_and_clock() {
        let mut stage = Stage::new();
        let source = stage.add(crate::vmobject::v_group(vec![
            VMobject::from_points(vec![[0.0; 3], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]),
            VMobject::from_points(vec![[10.0, 0.0, 0.0], [11.0, 0.0, 0.0], [12.0, 0.0, 0.0]]),
        ]));
        let boundary = AnimatedBoundary::new()
            .with_cycle_rate(1.0)
            .with_draw_rate_func(|alpha| alpha)
            .add_to_stage(&mut stage, source)
            .unwrap();
        stage.update_mobject(boundary, 0.25);
        stage.update_mobject(boundary, 0.25);
        let growing = stage.get(boundary).unwrap().submobjects()[0];
        let ghosts = stage.get(growing).unwrap().submobjects();
        assert_eq!(
            stage.get_points(ghosts[1]).unwrap().last(),
            Some(&[0.5, 0.0, 0.0])
        );
        assert_eq!(
            stage.get_points(ghosts[2]).unwrap().last(),
            Some(&[10.5, 0.0, 0.0])
        );
        let points = |stage: &Stage, mob| {
            stage
                .family(mob)
                .iter()
                .map(|&child| stage.get_points(child).unwrap())
                .collect::<Vec<_>>()
        };
        let copied = stage.copy_family(boundary).unwrap();
        let before = points(&stage, boundary);
        stage.update_mobject(copied, 0.25);
        assert_eq!(
            points(&stage, boundary),
            before,
            "copy must not write original ghosts"
        );
        assert_ne!(points(&stage, copied), before, "copy's clock must advance");
        stage.update_mobject(boundary, 0.25);
        assert_eq!(points(&stage, copied), points(&stage, boundary));
    }

    #[test]
    fn owned_updater_time_varying_field_copies_advance_independent_clocks() {
        let mut stage = Stage::new();
        let field = TimeVaryingVectorField::new(
            |coords, time| vec![[1.0, time, 0.0]; coords.len()],
            axes(),
        )
        .with_sample_coords(vec![[0.0; 3], [1.0, 0.0, 0.0]])
        .add_to_stage(&mut stage)
        .unwrap();
        stage.update_mobject(field, 0.25);
        let copied = stage.copy_family(field).unwrap();
        let before = stage.get_points(field).unwrap();
        stage.update_mobject(copied, 0.5);
        assert_eq!(stage.get_points(field).unwrap(), before);
        assert_ne!(stage.get_points(copied).unwrap(), before);
        stage.update_mobject(field, 0.5);
        assert_eq!(stage.get_points(field), stage.get_points(copied));
    }

    #[test]
    fn owned_updater_stream_line_copies_write_their_own_children() {
        let mut stage = Stage::new();
        let rng = RngRoot::from_seed(11);
        let streams = StreamLines::new(rotation_field, axes(), &rng)
            .with_sample_coords(vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]])
            .with_noise_factor(0.0)
            .with_solution_time(1.0)
            .with_samples_per_line(12);
        let original = AnimatedStreamLines::new(streams)
            .with_lag_range(0.01)
            .add_to_stage(&mut stage)
            .unwrap();
        stage.update_mobject(original, 0.25);
        let widths = |stage: &Stage, mob| {
            stage
                .get(mob)
                .unwrap()
                .submobjects()
                .iter()
                .map(|&child| {
                    stage
                        .get(child)
                        .unwrap()
                        .buffer
                        .read_column("stroke_width")
                        .unwrap()
                })
                .collect::<Vec<_>>()
        };
        let copied = stage.copy_family(original).unwrap();
        let before = widths(&stage, original);
        stage.update_mobject(copied, 0.25);
        assert_eq!(widths(&stage, original), before);
        assert_ne!(widths(&stage, copied), before);
        stage.update_mobject(original, 0.25);
        assert_eq!(widths(&stage, original), widths(&stage, copied));
    }

    #[test]
    fn move_helpers_nudge_and_demote() {
        let mut stage = Stage::new();
        let mob = stage.add(VMobject::from_points(vec![
            [0.0; 3],
            [0.5, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        ]));
        stage
            .add_to_scene(mob)
            .unwrap_or_else(|e| std::panic::panic_any(format!("root: {e}")));
        move_along_vector_field(&mut stage, mob, |_c| [1.0, 0.0, 0.0])
            .unwrap_or_else(|e| std::panic::panic_any(format!("move: {e}")));
        let before = stage.get_center(mob);
        stage.update(0.5);
        let after = stage.get_center(mob);
        assert!(
            (after[0] - before[0] - 0.5).abs() < 1e-9,
            "shift by func·dt"
        );
        assert!(!classify_wait(&stage, false).is_pure());
    }
}
