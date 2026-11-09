//! The coordinate-system plane (§12.2): the [`CoordinateSystem`] trait,
//! [`NumberLine`], [`Axes`], [`UnitInterval`], and [`Slider`].
//!
//! Ported from `manimlib/mobject/number_line.py` and the `Axes` half of
//! `manimlib/mobject/coordinate_systems.py` @ `6199a00d`. Constructor
//! defaults and layout semantics are the Reference's exactly:
//!
//! * **`n2p`/`p2n` are endpoint interpolations.** `number_to_point` reads
//!   the line's own first and last points and extrapolates linearly;
//!   `point_to_number` projects onto the line through them (the
//!   Reference's `fdiv` — a zero-length line maps everything to
//!   infinity). Ours keep the endpoints as tracked state so the same
//!   formulas work pre-build, and [`NumberLine::shifted`] /
//!   [`NumberLine::rotated_about`] carry the state through transforms
//!   the way the Reference's point mutation does.
//! * **Tick range** is `arange(x_min, x_max + x_step, x_step)` filtered to
//!   `<= x_max` (`x_max + x_step` becomes `x_max` when `include_tip`), and
//!   a tick at a `big_tick_numbers` value (numpy `isclose`) is
//!   `longer_tick_multiple` times taller. A tick is a `Line(size*DOWN,
//!   size*UP)` — total length `2 * size` — rotated to the line's angle
//!   and moved to `n2p(x)`.
//! * **Number labels** are native [`DecimalNumber`]s (BN-08), positioned
//!   `next_to(n2p(x), direction, buff)` with the Reference's minus-sign
//!   re-centering (a negative label whose direction is purely vertical
//!   shifts left by half the dash glyph, so the digit — not the sign —
//!   sits under the tick).
//! * **`Axes` builds two number lines** through the config merge
//!   `default_axis < default_x/y_axis < axis_config < {unit_size} <
//!   x/y_axis_config`, shifts each by `-n2p(0)`, rotates the y-axis 90°
//!   about the origin, and centres the group. `c2p` is the Reference
//!   formula verbatim: `origin + Σ (axis.n2p(c) − origin)`.
//!
//! Deliberate divergences, all documented at their sites:
//!
//! * **`tick_offset` and `show_signed_area` are stored but unused** — the
//!   Reference stores both and never reads them (`number_line.py:39,61`,
//!   `coordinate_systems.py:375`). Kept for ctor parity.
//! * **The graph-taking calculus helpers take the function directly.**
//!   The Reference reads `graph.underlying_function`; a built [`VMobject`]
//!   here is plain geometry, so `input_to_graph_point` and friends take
//!   `&dyn Fn(f64) -> f64` (the `get_graph` closure) instead. The
//!   binary-search fallback for function-less graphs is not ported.
//! * **`Slider` is a static snapshot.** The Reference wires the tip and
//!   the decimal label to a `ValueTracker` with updaters (W9/Proscenium
//!   territory); ours assembles the group at the initial value. Its TeX
//!   label is a native `Text` + [`DecimalNumber`] composition (BN-08).
//! * **Invalid `input_sample_type` is a typed error** ([`CoordsError`]),
//!   where the Reference raises a bare `Exception`.

mod live;
pub(crate) mod placement;

pub use live::{CoordinateFrame, CoordinateFrameError, LiveCoordinateSystem};

use fmn_core::color::{Srgb, color_gradient};
use fmn_core::constants::{
    BLACK, BLUE, DEFAULT_LIGHT_COLOR, DEFAULT_MOBJECT_COLOR, DEFAULT_MOBJECT_TO_EDGE_BUFF, DL,
    DOWN, DR, FRAME_X_RADIUS, FRAME_Y_RADIUS, GREEN, GREY_A, LEFT, MED_SMALL_BUFF, ORIGIN, OUT, PI,
    RED, RIGHT, SMALL_BUFF, UP, YELLOW,
};
use fmn_core::types::Vec3;
use fmn_geom::{QuadPath, space_ops};
use fmn_tex::TexEngine;
use fmn_text::FontBook;

use crate::graphs::{SamplingBudget, SamplingError, sampled_values};
use crate::line::{DashedLine, Line};
use crate::numbers::DecimalNumber;
use crate::pointcloud::DotCloud;
use crate::poly::{ArrowTip, Polygon};
use crate::style::Style;
use crate::tex::{Tex, TexMobjectError};
use crate::text::{Text, TextMobjectError, text_style};
use crate::tip::{TipEnd, attach_tip};
use crate::vmobject::{DashError, VMobject, v_group};

/// The Reference's `coordinate_systems.EPSILON` — the tangent secant step.
pub const EPSILON: f64 = 1e-8;

/// The Reference's `DEFAULT_X_RANGE`.
pub const DEFAULT_X_RANGE: [f64; 3] = [-8.0, 8.0, 1.0];
/// The Reference's `DEFAULT_Y_RANGE`.
pub const DEFAULT_Y_RANGE: [f64; 3] = [-4.0, 4.0, 1.0];

/// The Reference's `decimal_number_config.font_size` for standalone
/// `get_number_mobject` labels.
pub const DEFAULT_NUMBER_FONT_SIZE: f64 = 36.0;
/// The Reference's `add_numbers(font_size=24)` override.
pub const ADD_NUMBERS_FONT_SIZE: f64 = 24.0;

/// `CoordinateSystem` (the Reference's ABC): the bidirectional map between
/// abstract coordinates and scene points.
///
/// The trait shape is the W7 batch contract; `Axes` (here) and the
/// planes family (planes.rs) implement it on their built values.
pub trait CoordinateSystem {
    /// `coords_to_point`: map coordinates to a scene point.
    fn c2p(&self, coords: &[f64]) -> Vec3;
    /// `point_to_coords`: map a scene point back to coordinates
    /// (zero-padded to three components).
    fn p2c(&self, point: Vec3) -> [f64; 3];
    /// `get_all_ranges`: `(min, max, tick_step)` per axis.
    fn all_ranges(&self) -> Vec<[f64; 3]>;
    /// `num_sampled_graph_points_per_tick` (Reference default 5).
    fn num_sampled_graph_points_per_tick(&self) -> f64 {
        5.0
    }
    /// Resource limit carried into graphs sampled from this coordinate system.
    fn graph_sampling_budget(&self) -> SamplingBudget {
        SamplingBudget::default()
    }
    /// The coordinate dimension (2 for `Axes`, 3 for `ThreeDAxes`).
    fn dimension(&self) -> usize {
        2
    }
    /// `get_origin`: `c2p(0, …, 0)`.
    fn origin(&self) -> Vec3 {
        self.c2p(&vec![0.0; self.dimension()])
    }
}

/// A coordinate-system failure that the Reference raises as a bare
/// `Exception`; here it is typed.
#[derive(Debug)]
pub enum CoordsError {
    /// `get_riemann_rectangles(input_sample_type=…)` outside
    /// `"left" | "right" | "center"`.
    InvalidSampleType(String),
    /// Atlas range sampling refused before proportional work began.
    Sampling(SamplingError),
    /// A native number or label failed to typeset.
    Text(TextMobjectError),
    /// A dashed projection line exceeded or violated the dash contract.
    Dash(DashError),
    /// An axis or graph label failed to typeset.
    Tex(TexMobjectError),
    /// A Riemann range that decreases, has a non-positive step, or whose
    /// step cannot advance the bound (BN-20).
    InvalidRiemannRange(&'static str),
    /// A live native chart no longer describes a finite affine frame.
    LiveFrame(CoordinateFrameError),
}

impl std::fmt::Display for CoordsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSampleType(t) => {
                write!(
                    f,
                    "invalid input sample type {t:?} (expected left|right|center)"
                )
            }
            Self::Sampling(e) => write!(f, "coordinate sampling failed: {e}"),
            Self::Text(e) => write!(f, "coordinate label failed: {e}"),
            Self::Dash(e) => write!(f, "coordinate dash construction failed: {e}"),
            Self::Tex(e) => write!(f, "coordinate label failed to typeset: {e}"),
            Self::InvalidRiemannRange(reason) => write!(f, "invalid Riemann range: {reason}"),
            Self::LiveFrame(e) => write!(f, "live coordinate frame failed: {e}"),
        }
    }
}

impl std::error::Error for CoordsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Sampling(e) => Some(e),
            Self::Text(e) => Some(e),
            Self::Dash(e) => Some(e),
            Self::Tex(e) => Some(e),
            Self::LiveFrame(e) => Some(e),
            Self::InvalidSampleType(_) | Self::InvalidRiemannRange(_) => None,
        }
    }
}

impl From<SamplingError> for CoordsError {
    fn from(e: SamplingError) -> Self {
        Self::Sampling(e)
    }
}

impl From<TextMobjectError> for CoordsError {
    fn from(e: TextMobjectError) -> Self {
        Self::Text(e)
    }
}

impl From<DashError> for CoordsError {
    fn from(e: DashError) -> Self {
        Self::Dash(e)
    }
}

impl From<TexMobjectError> for CoordsError {
    fn from(e: TexMobjectError) -> Self {
        Self::Tex(e)
    }
}

impl From<CoordinateFrameError> for CoordsError {
    fn from(e: CoordinateFrameError) -> Self {
        Self::LiveFrame(e)
    }
}

// --- small Vec3 helpers (matching the sibling modules' local style) ----

fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: Vec3, s: f64) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn neg(a: Vec3) -> Vec3 {
    [-a[0], -a[1], -a[2]]
}

/// The Reference's `fdiv`: division with a defined answer at zero.
fn fdiv(a: f64, b: f64) -> f64 {
    if b == 0.0 { f64::INFINITY } else { a / b }
}

/// numpy `arange(start, stop, step)`: `start + k·step` while `< stop`.
///
/// A finite non-positive step yields nothing (numpy raises on 0 and
/// returns empty when the step points away from `stop`; both read as
/// "no ticks", which is the only sane detached-builder behaviour).
/// Non-finite controls are typed [`SamplingError`]s.
fn arange(
    context: &'static str,
    start: f64,
    stop: f64,
    step: f64,
    budget: SamplingBudget,
) -> Result<Vec<f64>, SamplingError> {
    sampled_values(context, start, stop, step, false, budget)
}

/// numpy `isclose` with its defaults (`rtol=1e-5, atol=1e-8`).
fn isclose(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-8 + 1e-5 * b.abs()
}

/// Membership under [`isclose`] for finite, total-order-sorted candidates.
fn sorted_contains_close(values: &[f64], target: f64) -> bool {
    let index = values.partition_point(|&value| value < target && !isclose(value, target));
    values
        .get(index)
        .is_some_and(|&value| isclose(value, target))
}

/// The Reference's `inverse_interpolate`.
fn inverse_interpolate(x0: f64, x1: f64, x: f64) -> f64 {
    (x - x0) / (x1 - x0)
}

/// BN-20's Riemann bins over `[start, stop]`: `start + k·step` for
/// `k < count`, then `stop`, where `count = max(1, ⌈(stop − start)/step⌉)`
/// with the ratio stepped down one ulp first, so a range that is an exact
/// multiple of `step` up to rounding never grows a sliver bin. The portal's
/// `fmn_python.calculus._riemann_grid` computes the same bins.
fn riemann_bins(
    start: f64,
    stop: f64,
    step: f64,
    budget: SamplingBudget,
) -> Result<Vec<(f64, f64)>, CoordsError> {
    const CONTEXT: &str = "Riemann rectangles";
    for (parameter, value) in [("start", start), ("stop", stop), ("dx", step)] {
        if !value.is_finite() {
            return Err(SamplingError::NonFinite {
                context: CONTEXT,
                parameter,
                value,
            }
            .into());
        }
    }
    if stop < start {
        return Err(CoordsError::InvalidRiemannRange("the range decreases"));
    }
    if step <= 0.0 {
        return Err(CoordsError::InvalidRiemannRange("dx must be positive"));
    }
    if start == stop {
        return Ok(Vec::new());
    }
    let ratio = (stop - start) / step;
    let limit = budget.max_samples();
    let exceeded = || {
        CoordsError::from(SamplingError::LimitExceeded {
            context: CONTEXT,
            max_samples: limit,
        })
    };
    if !ratio.is_finite() || ratio > limit as f64 {
        return Err(exceeded());
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let count = (ratio.next_down().ceil() as usize).max(1);
    // `count` bins have `count + 1` bounds; the budget bounds those values.
    if count >= limit {
        return Err(exceeded());
    }
    let mut boundaries: Vec<f64> = (0..count).map(|k| start + k as f64 * step).collect();
    boundaries.push(stop);
    let bins: Vec<(f64, f64)> = boundaries.windows(2).map(|w| (w[0], w[1])).collect();
    if bins.iter().any(|&(left, right)| right <= left) {
        return Err(CoordsError::InvalidRiemannRange(
            "dx is too small to advance the range",
        ));
    }
    Ok(bins)
}

// -----------------------------------------------------------------------
// NumberLine
// -----------------------------------------------------------------------

/// `NumberLine(x_range, unit_size, …)` (Appendix A `mobject/number_line`).
///
/// A value type in the library idiom: by-value config setters, then
/// [`build`](NumberLine::build) (geometry only — no font access needed)
/// or [`build_numbered`](NumberLine::build_numbered) (which also typesets
/// `include_numbers` labels against a [`FontBook`]). `n2p`/`p2n` and the
/// range getters work before and after building, and stay correct through
/// [`shifted`](NumberLine::shifted) / [`rotated_about`](NumberLine::rotated_about).
#[derive(Debug, Clone)]
pub struct NumberLine {
    // --- the Reference's constructor surface ---------------------------
    x_range: [f64; 3],
    style: Style,
    unit_size: f64,
    width: Option<f64>,
    include_ticks: bool,
    tick_size: f64,
    longer_tick_multiple: f64,
    tick_offset: f64,
    big_tick_spacing: Option<f64>,
    big_tick_numbers: Vec<f64>,
    include_numbers: bool,
    line_to_number_direction: Vec3,
    line_to_number_buff: f64,
    include_tip: bool,
    tip_size: f64,
    num_decimal_places: usize,
    number_font_size: f64,
    numbers_font_size: f64,
    numbers_to_exclude: Option<Vec<f64>>,
    sampling_budget: SamplingBudget,
    // --- geometry state --------------------------------------------------
    /// First point of the line (`get_points()[0]`); kept in sync by the
    /// setters and the transform methods so `n2p` is exact at all times.
    start: Vec3,
    /// Last point of the line (`get_points()[-1]`).
    end: Vec3,
    // --- built state -----------------------------------------------------
    vmob: VMobject,
    /// The labels `add_numbers` built, in `x_values` order (excluding
    /// filtered) — the glyph-sequence introspection surface.
    numbers: Vec<DecimalNumber>,
}

impl Default for NumberLine {
    fn default() -> Self {
        Self::new(DEFAULT_X_RANGE)
    }
}

impl NumberLine {
    /// The Reference defaults: `unit_size=1`, ticks on at `0.1` with
    /// `1.5×` longer big ticks, no numbers, `line_to_number_direction=DOWN`,
    /// `buff=MED_SMALL_BUFF`, no tip, `num_decimal_places=0`,
    /// `font_size=36`, `color=DEFAULT_LIGHT_COLOR` (fill and stroke, as the
    /// Reference's `Line(color=...)` resolves it) and a 2-width stroke.
    #[must_use]
    pub fn new(x_range: [f64; 3]) -> Self {
        let mut line = Self {
            x_range,
            style: Style::default().color(DEFAULT_LIGHT_COLOR).stroke(
                DEFAULT_LIGHT_COLOR,
                2.0,
                1.0,
            ),
            unit_size: 1.0,
            width: None,
            include_ticks: true,
            tick_size: 0.1,
            longer_tick_multiple: 1.5,
            tick_offset: 0.0,
            big_tick_spacing: None,
            big_tick_numbers: Vec::new(),
            include_numbers: false,
            line_to_number_direction: DOWN,
            line_to_number_buff: MED_SMALL_BUFF,
            include_tip: false,
            tip_size: 0.25,
            num_decimal_places: 0,
            number_font_size: DEFAULT_NUMBER_FONT_SIZE,
            numbers_font_size: ADD_NUMBERS_FONT_SIZE,
            numbers_to_exclude: None,
            sampling_budget: SamplingBudget::default(),
            start: ORIGIN,
            end: ORIGIN,
            vmob: VMobject::new(),
            numbers: Vec::new(),
        };
        line.sync_endpoints();
        line
    }

    /// The two-element `RangeSpecifier` form: `(min, max)` with step 1.
    #[must_use]
    pub fn with_min_max(x_min: f64, x_max: f64) -> Self {
        Self::new([x_min, x_max, 1.0])
    }

    /// Recompute the canonical endpoints: the horizontal line through the
    /// range midpoint, centred (the Reference's `scale` + `center`).
    fn sync_endpoints(&mut self) {
        let mid = 0.5 * (self.x_range[0] + self.x_range[1]);
        let unit = self.effective_unit_size();
        self.start = [(self.x_range[0] - mid) * unit, 0.0, 0.0];
        self.end = [(self.x_range[1] - mid) * unit, 0.0, 0.0];
    }

    // --- setters (the Reference's keyword surface) ---------------------

    /// `unit_size=`: one number-line unit in scene units.
    ///
    /// Recomputes the canonical endpoints; call before transforming.
    #[must_use]
    pub fn unit_size(mut self, unit_size: f64) -> Self {
        self.unit_size = unit_size;
        self.width = None;
        self.sync_endpoints();
        self
    }

    /// `width=`: total width, overriding `unit_size`
    /// (`width / (x_max − x_min)` becomes the effective unit size).
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.width = Some(width);
        self.sync_endpoints();
        self
    }

    /// `include_ticks=` (default true).
    #[must_use]
    pub fn include_ticks(mut self, on: bool) -> Self {
        self.include_ticks = on;
        self
    }

    /// `tick_size=` (default 0.1 — a tick spans `±size` about the line).
    #[must_use]
    pub fn tick_size(mut self, size: f64) -> Self {
        self.tick_size = size;
        self
    }

    /// `longer_tick_multiple=` (default 1.5).
    #[must_use]
    pub fn longer_tick_multiple(mut self, multiple: f64) -> Self {
        self.longer_tick_multiple = multiple;
        self
    }

    /// `tick_offset=`. Stored for ctor parity; the Reference stores it
    /// and never reads it (`number_line.py:61`).
    #[must_use]
    pub fn tick_offset(mut self, offset: f64) -> Self {
        self.tick_offset = offset;
        self
    }

    /// `big_tick_spacing=`: big ticks at `arange(x_min, x_max + spacing,
    /// spacing)`.
    #[must_use]
    pub fn big_tick_spacing(mut self, spacing: f64) -> Self {
        self.big_tick_spacing = Some(spacing);
        self
    }

    /// `big_tick_numbers=`: the explicit big-tick list.
    #[must_use]
    pub fn big_tick_numbers(mut self, numbers: impl Into<Vec<f64>>) -> Self {
        self.big_tick_numbers = numbers.into();
        self.big_tick_spacing = None;
        self
    }

    /// `include_numbers=`: labels at every tick on
    /// [`build_numbered`](Self::build_numbered).
    #[must_use]
    pub fn include_numbers(mut self, on: bool) -> Self {
        self.include_numbers = on;
        self
    }

    /// `line_to_number_direction=` (default `DOWN`).
    #[must_use]
    pub fn line_to_number_direction(mut self, direction: Vec3) -> Self {
        self.line_to_number_direction = direction;
        self
    }

    /// `line_to_number_buff=` (default `MED_SMALL_BUFF`).
    #[must_use]
    pub fn line_to_number_buff(mut self, buff: f64) -> Self {
        self.line_to_number_buff = buff;
        self
    }

    /// The configured `line_to_number_direction`.
    #[must_use]
    pub fn line_to_number_direction_value(&self) -> Vec3 {
        self.line_to_number_direction
    }

    /// `include_tip=`: an arrow tip at the line's end.
    #[must_use]
    pub fn include_tip(mut self, on: bool) -> Self {
        self.include_tip = on;
        self
    }

    /// `tip_config=(width, length)` — the Reference sets both to 0.25.
    #[must_use]
    pub fn tip_size(mut self, size: f64) -> Self {
        self.tip_size = size;
        self
    }

    /// `decimal_number_config.num_decimal_places=` (default 0).
    #[must_use]
    pub fn num_decimal_places(mut self, ndp: usize) -> Self {
        self.num_decimal_places = ndp;
        self
    }

    /// `decimal_number_config.font_size=` for standalone
    /// [`number_mobject`](Self::number_mobject) labels (default 36).
    #[must_use]
    pub fn number_font_size(mut self, font_size: f64) -> Self {
        self.number_font_size = font_size;
        self
    }

    /// The `add_numbers(font_size=24)` override.
    #[must_use]
    pub fn numbers_font_size(mut self, font_size: f64) -> Self {
        self.numbers_font_size = font_size;
        self
    }

    /// `numbers_to_exclude=`.
    #[must_use]
    pub fn numbers_to_exclude(mut self, values: impl Into<Vec<f64>>) -> Self {
        self.numbers_to_exclude = Some(values.into());
        self
    }

    /// Bound every tick or default-label range produced by this line.
    #[must_use]
    pub fn sampling_budget(mut self, budget: SamplingBudget) -> Self {
        self.sampling_budget = budget;
        self
    }

    /// `color=`: stroke and fill together.
    #[must_use]
    pub fn color(mut self, color: Srgb) -> Self {
        self.style = self.style.color(color);
        self
    }

    /// `stroke_width=` (default 2).
    #[must_use]
    pub fn stroke_width(mut self, width: f64) -> Self {
        self.style = self.style.stroke_width(width);
        self
    }

    /// `stroke_opacity=`: the Reference's VMobject style keyword, which the
    /// line's ticks match.
    #[must_use]
    pub fn stroke_opacity(mut self, opacity: f64) -> Self {
        self.style.stroke_opacity = opacity;
        self
    }

    /// `stroke_color=`: the stroke alone, which the line's ticks match.
    /// `VMobject.__init__` resolves `stroke_color or color`, so this wins
    /// over [`NumberLine::color`] for the stroke whichever is set first.
    #[must_use]
    pub fn stroke_color(mut self, color: Srgb) -> Self {
        self.style.stroke_color = color;
        self
    }

    /// Replace the line's style wholesale.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    // --- getters ---------------------------------------------------------

    /// `x_min`.
    #[must_use]
    pub fn x_min(&self) -> f64 {
        self.x_range[0]
    }

    /// `x_max`.
    #[must_use]
    pub fn x_max(&self) -> f64 {
        self.x_range[1]
    }

    /// `x_step`.
    #[must_use]
    pub fn x_step(&self) -> f64 {
        self.x_range[2]
    }

    /// The full `(min, max, step)` range.
    #[must_use]
    pub fn x_range(&self) -> [f64; 3] {
        self.x_range
    }

    /// `get_unit_size`: the effective unit size — `width/(x_max − x_min)`
    /// when a width was given, else `unit_size`.
    #[must_use]
    pub fn effective_unit_size(&self) -> f64 {
        match self.width {
            Some(w) => w / (self.x_range[1] - self.x_range[0]),
            None => self.unit_size,
        }
    }

    /// The first point of the line.
    #[must_use]
    pub fn start_point(&self) -> Vec3 {
        self.start
    }

    /// The last point of the line.
    #[must_use]
    pub fn end_point(&self) -> Vec3 {
        self.end
    }

    /// The line's angle (`get_angle`).
    #[must_use]
    pub fn line_angle(&self) -> f64 {
        space_ops::angle_of_vector(sub(self.end, self.start))
    }

    /// `get_tick_range`: `arange(x_min, x_max + x_step, x_step)` — the end
    /// is `x_max` instead when a tip replaces the last tick — filtered to
    /// `<= x_max`.
    pub fn tick_range(&self) -> Result<Vec<f64>, SamplingError> {
        let end = if self.include_tip {
            self.x_range[1]
        } else {
            self.x_range[1] + self.x_range[2]
        };
        Ok(arange(
            "number-line ticks",
            self.x_range[0],
            end,
            self.x_range[2],
            self.sampling_budget,
        )?
        .into_iter()
        .filter(|&x| x <= self.x_range[1])
        .collect())
    }

    /// The resolved big-tick list (`big_tick_spacing` wins when set).
    pub fn resolved_big_tick_numbers(&self) -> Result<Vec<f64>, SamplingError> {
        match self.big_tick_spacing {
            Some(spacing) => arange(
                "number-line big ticks",
                self.x_range[0],
                self.x_range[1] + spacing,
                spacing,
                self.sampling_budget,
            ),
            None => {
                self.sampling_budget
                    .ensure_total("number-line big ticks", self.big_tick_numbers.len())?;
                Ok(self.big_tick_numbers.clone())
            }
        }
    }

    /// `number_to_point`: extrapolate along the tracked endpoints.
    #[must_use]
    pub fn n2p(&self, number: f64) -> Vec3 {
        let alpha = (number - self.x_range[0]) / (self.x_range[1] - self.x_range[0]);
        add(self.start, scale(sub(self.end, self.start), alpha))
    }

    /// `point_to_number`: project onto the line, then read the number.
    #[must_use]
    pub fn p2n(&self, point: Vec3) -> f64 {
        let vect = sub(self.end, self.start);
        let proportion = fdiv(
            space_ops::dot(sub(point, self.start), vect),
            space_ops::dot(vect, vect),
        );
        self.x_range[0] + (self.x_range[1] - self.x_range[0]) * proportion
    }

    /// `get_projection`: the projection of a point onto the line through
    /// the segment's ends.
    #[must_use]
    pub fn projection(&self, point: Vec3) -> Vec3 {
        let vect = sub(self.end, self.start);
        let t = fdiv(
            space_ops::dot(sub(point, self.start), vect),
            space_ops::dot(vect, vect),
        );
        add(self.start, scale(vect, t))
    }

    /// The built family.
    #[must_use]
    pub fn vmob(&self) -> &VMobject {
        &self.vmob
    }

    /// Consume into the family (for `Stage::add`).
    #[must_use]
    pub fn into_vmob(self) -> VMobject {
        self.vmob
    }

    /// The labels [`add_numbers`](Self::add_numbers) built, in order.
    #[must_use]
    pub fn numbers(&self) -> &[DecimalNumber] {
        &self.numbers
    }

    // --- transforms (carry the n2p state, exactly like point mutation) --

    /// Shift the whole line — geometry, endpoints, and labels.
    #[must_use]
    pub fn shifted(mut self, offset: Vec3) -> Self {
        self.vmob = core::mem::take(&mut self.vmob).shifted(offset);
        self.start = add(self.start, offset);
        self.end = add(self.end, offset);
        self
    }

    /// Rotate about a pivot and axis (Reference `rotate`).
    #[must_use]
    pub fn rotated_about(mut self, angle: f64, axis: Vec3, about: Vec3) -> Self {
        self.vmob = core::mem::take(&mut self.vmob).rotated_about(angle, axis, about);
        let m = fmn_geom::rotation_matrix(angle, axis);
        let rot = |p: Vec3| {
            let v = sub(p, about);
            [
                about[0] + m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
                about[1] + m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
                about[2] + m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
            ]
        };
        self.start = rot(self.start);
        self.end = rot(self.end);
        self
    }

    // --- building --------------------------------------------------------

    /// One tick: `Line(size*DOWN, size*UP)`, rotated to the line's angle,
    /// moved to `n2p(x)`, styled like the line (`match_style`).
    fn tick_vmob(&self, x: f64, size: f64) -> VMobject {
        let tick = Line::new(scale(DOWN, size), scale(UP, size))
            .style(self.style)
            .build()
            .expect("a NumberLine tick is always a straight segment");
        let angle = self.line_angle();
        let center = tick.center_point();
        tick.rotated_about(angle, OUT, center).moved_to(self.n2p(x))
    }

    /// Build the line, tip, and ticks. No font access: number labels need
    /// [`build_numbered`](Self::build_numbered) or
    /// [`add_numbers`](Self::add_numbers) instead.
    pub fn build(mut self) -> Result<Self, SamplingError> {
        let mut vmob = Line::new(self.start, self.end)
            .style(self.style)
            .build()
            .expect("a NumberLine axis is always a straight segment");
        if self.include_tip {
            // The Reference's tip_config (width=length=0.25), stroked like
            // the line (`self.tip.set_stroke(self.stroke_color, …)`).
            let tip = ArrowTip::new()
                .width(self.tip_size)
                .length(self.tip_size)
                .style(Style::default().fill(DEFAULT_MOBJECT_COLOR, 1.0).stroke(
                    self.style.stroke_color,
                    self.style.stroke_width,
                    1.0,
                ));
            vmob = attach_tip(vmob, tip, TipEnd::End);
        }
        if self.include_ticks {
            let mut big = self.resolved_big_tick_numbers()?;
            // Non-finite candidates never match a finite sampled tick under
            // `isclose`; remove them so one binary search replaces the
            // previous potentially quadratic scan.
            big.retain(|value| value.is_finite());
            big.sort_by(f64::total_cmp);
            let ticks: Vec<VMobject> = self
                .tick_range()?
                .into_iter()
                .map(|x| {
                    let mut size = self.tick_size;
                    if sorted_contains_close(&big, x) {
                        size *= self.longer_tick_multiple;
                    }
                    self.tick_vmob(x, size)
                })
                .collect();
            vmob = vmob.with_child(v_group(ticks));
        }
        self.vmob = vmob;
        Ok(self)
    }

    /// Build, then add `include_numbers` labels
    /// (`add_numbers(excluding=numbers_to_exclude)`), as the Reference's
    /// constructor does.
    ///
    /// # Errors
    /// [`CoordsError`] from bounded tick sampling or label typesetting.
    pub fn build_numbered(self, book: &FontBook) -> Result<Self, CoordsError> {
        let include = self.include_numbers;
        let excluding = self.numbers_to_exclude.clone();
        let mut built = self.build()?;
        if include {
            built.add_numbers(book, None, excluding.as_deref())?;
        }
        Ok(built)
    }

    /// `get_number_mobject`'s core: build the [`DecimalNumber`] for `x`
    /// and position it. Returns the built label (for glyph introspection)
    /// and its positioned geometry.
    fn build_number(
        &self,
        x: f64,
        font_size: f64,
        book: &FontBook,
    ) -> Result<(DecimalNumber, VMobject), TextMobjectError> {
        let dec = DecimalNumber::new(x)
            .num_decimal_places(self.num_decimal_places)
            .font_size(font_size)
            .build(book)?;
        let mut vmob = dec.vmob().clone().next_to_point(
            self.n2p(x),
            self.line_to_number_direction,
            self.line_to_number_buff,
            ORIGIN,
        );
        if x < 0.0 && self.line_to_number_direction[0] == 0.0 {
            // Align without the minus sign: shift left by half the dash.
            let dash_width = dec
                .vmob()
                .children()
                .first()
                .map_or(0.0, |c| c.length_over_dim(0));
            vmob = vmob.shifted([-dash_width / 2.0, 0.0, 0.0]);
        }
        Ok((dec, vmob))
    }

    /// `get_number_mobject`: the positioned label geometry for `x`,
    /// using the line's `line_to_number_direction`/`_buff` and
    /// `decimal_number_config` (font size 36 by default).
    ///
    /// # Errors
    /// [`TextMobjectError`] from typesetting.
    pub fn number_mobject(&self, x: f64, book: &FontBook) -> Result<VMobject, TextMobjectError> {
        Ok(self.build_number(x, self.number_font_size, book)?.1)
    }

    /// `get_number_mobject(x, unit=…, unit_tex=…)`: the unit-label path —
    /// `DecimalNumber(x / unit, unit=unit_tex)`, and when `|x| == unit`
    /// the lone `1` digit is dropped (and for `x < 0` the dash re-seats
    /// beside the unit), exactly the Reference's special case.
    ///
    /// # Errors
    /// [`TextMobjectError`] from typesetting.
    pub fn number_mobject_with_unit(
        &self,
        x: f64,
        unit: f64,
        unit_tex: &str,
        book: &FontBook,
    ) -> Result<VMobject, TextMobjectError> {
        let mut dec = DecimalNumber::new(x / unit)
            .num_decimal_places(self.num_decimal_places)
            .font_size(self.number_font_size);
        if !unit_tex.is_empty() {
            dec = dec.unit(unit_tex);
        }
        let dec = dec.build(book)?;
        let mut vmob = dec.vmob().clone().next_to_point(
            self.n2p(x),
            self.line_to_number_direction,
            self.line_to_number_buff,
            ORIGIN,
        );
        if x < 0.0 && self.line_to_number_direction[0] == 0.0 {
            let dash_width = dec
                .vmob()
                .children()
                .first()
                .map_or(0.0, |c| c.length_over_dim(0));
            vmob = vmob.shifted([-dash_width / 2.0, 0.0, 0.0]);
        }
        if x.abs() == unit && !unit_tex.is_empty() {
            let center = vmob.center_point();
            let mut kids = vmob.children().to_vec();
            if x > 0.0 {
                // Drop the lone "1"; the unit stays.
                if !kids.is_empty() {
                    kids.remove(0);
                }
            } else if kids.len() >= 2 {
                // [dash, "1", unit…] → drop "1", dash rides LEFT of the unit.
                let dash = kids[0].clone();
                let dash_buff = dash.length_over_dim(0) / 4.0;
                kids.remove(1);
                let unit_child = kids[1].clone();
                kids[0] = dash.next_to(&unit_child, LEFT, dash_buff, ORIGIN);
            }
            vmob = VMobject::new()
                .with_style(vmob.style())
                .with_children(kids)
                .moved_to(center);
        }
        Ok(vmob)
    }

    /// `add_numbers`: build and attach labels at `x_values` (default: the
    /// tick range), skipping `excluding` (default:
    /// `numbers_to_exclude`), at the `add_numbers` font size (24).
    ///
    /// The labels are appended to the family as one group child, and the
    /// built [`DecimalNumber`]s are kept (in order) on
    /// [`numbers`](Self::numbers) for introspection.
    ///
    /// # Errors
    /// [`CoordsError`] from bounded default-range sampling or typesetting.
    pub fn add_numbers(
        &mut self,
        book: &FontBook,
        x_values: Option<&[f64]>,
        excluding: Option<&[f64]>,
    ) -> Result<(), CoordsError> {
        let values: Vec<f64> = match x_values {
            Some(v) => {
                self.sampling_budget
                    .ensure_total("number-line labels", v.len())?;
                v.to_vec()
            }
            None => self.tick_range()?,
        };
        let excluding = excluding.or(self.numbers_to_exclude.as_deref());
        let mut numbers = Vec::new();
        let mut vmobs = Vec::new();
        for x in values {
            if excluding.is_some_and(|ex| ex.contains(&x)) {
                continue;
            }
            let (dec, vmob) = self.build_number(x, self.numbers_font_size, book)?;
            numbers.push(dec);
            vmobs.push(vmob);
        }
        self.vmob = core::mem::take(&mut self.vmob).with_child(v_group(vmobs));
        self.numbers = numbers;
        Ok(())
    }
}

// -----------------------------------------------------------------------
// AxisConfig + create_axis
// -----------------------------------------------------------------------

/// The per-axis configuration of [`Axes`], as an Option-field record
/// standing in for the Reference's `dict` kwargs: `merge` is
/// `merge_dicts_recursively` restricted to this surface (later `Some`
/// wins), and [`AxisConfig::apply`] pushes the result onto a
/// [`NumberLine`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AxisConfig {
    /// `color=`.
    pub color: Option<Srgb>,
    /// `stroke_color=`: overrides `color` for the stroke
    /// (`stroke_color or color`), applied after it.
    pub stroke_color: Option<Srgb>,
    /// `stroke_width=`.
    pub stroke_width: Option<f64>,
    /// `stroke_opacity=`.
    pub stroke_opacity: Option<f64>,
    /// `unit_size=`.
    pub unit_size: Option<f64>,
    /// `include_ticks=`.
    pub include_ticks: Option<bool>,
    /// `tick_size=`.
    pub tick_size: Option<f64>,
    /// `longer_tick_multiple=`.
    pub longer_tick_multiple: Option<f64>,
    /// `tick_offset=` (stored parity — see [`NumberLine::tick_offset`]).
    pub tick_offset: Option<f64>,
    /// `big_tick_spacing=`.
    pub big_tick_spacing: Option<f64>,
    /// `include_numbers=`.
    pub include_numbers: Option<bool>,
    /// `line_to_number_direction=`.
    pub line_to_number_direction: Option<Vec3>,
    /// `line_to_number_buff=`.
    pub line_to_number_buff: Option<f64>,
    /// `include_tip=`.
    pub include_tip: Option<bool>,
    /// `decimal_number_config.num_decimal_places=`.
    pub num_decimal_places: Option<usize>,
    /// `decimal_number_config.font_size=`.
    pub number_font_size: Option<f64>,
    /// `numbers_to_exclude=`.
    pub numbers_to_exclude: Option<Vec<f64>>,
}

impl AxisConfig {
    /// `merge_dicts_recursively(self, over)`: every `Some` in `over` wins.
    #[must_use]
    pub fn merge(self, over: Self) -> Self {
        Self {
            color: over.color.or(self.color),
            stroke_color: over.stroke_color.or(self.stroke_color),
            stroke_width: over.stroke_width.or(self.stroke_width),
            stroke_opacity: over.stroke_opacity.or(self.stroke_opacity),
            unit_size: over.unit_size.or(self.unit_size),
            include_ticks: over.include_ticks.or(self.include_ticks),
            tick_size: over.tick_size.or(self.tick_size),
            longer_tick_multiple: over.longer_tick_multiple.or(self.longer_tick_multiple),
            tick_offset: over.tick_offset.or(self.tick_offset),
            big_tick_spacing: over.big_tick_spacing.or(self.big_tick_spacing),
            include_numbers: over.include_numbers.or(self.include_numbers),
            line_to_number_direction: over
                .line_to_number_direction
                .or(self.line_to_number_direction),
            line_to_number_buff: over.line_to_number_buff.or(self.line_to_number_buff),
            include_tip: over.include_tip.or(self.include_tip),
            num_decimal_places: over.num_decimal_places.or(self.num_decimal_places),
            number_font_size: over.number_font_size.or(self.number_font_size),
            numbers_to_exclude: over.numbers_to_exclude.or(self.numbers_to_exclude),
        }
    }

    /// Push the configured fields onto a [`NumberLine`].
    #[must_use]
    pub fn apply(self, mut line: NumberLine) -> NumberLine {
        if let Some(v) = self.color {
            line = line.color(v);
        }
        if let Some(v) = self.stroke_color {
            line = line.stroke_color(v);
        }
        if let Some(v) = self.stroke_width {
            line = line.stroke_width(v);
        }
        if let Some(v) = self.stroke_opacity {
            line = line.stroke_opacity(v);
        }
        if let Some(v) = self.unit_size {
            line = line.unit_size(v);
        }
        if let Some(v) = self.include_ticks {
            line = line.include_ticks(v);
        }
        if let Some(v) = self.tick_size {
            line = line.tick_size(v);
        }
        if let Some(v) = self.longer_tick_multiple {
            line = line.longer_tick_multiple(v);
        }
        if let Some(v) = self.tick_offset {
            line = line.tick_offset(v);
        }
        if let Some(v) = self.big_tick_spacing {
            line = line.big_tick_spacing(v);
        }
        if let Some(v) = self.include_numbers {
            line = line.include_numbers(v);
        }
        if let Some(v) = self.line_to_number_direction {
            line = line.line_to_number_direction(v);
        }
        if let Some(v) = self.line_to_number_buff {
            line = line.line_to_number_buff(v);
        }
        if let Some(v) = self.include_tip {
            line = line.include_tip(v);
        }
        if let Some(v) = self.num_decimal_places {
            line = line.num_decimal_places(v);
        }
        if let Some(v) = self.number_font_size {
            line = line.number_font_size(v);
        }
        if let Some(v) = self.numbers_to_exclude {
            line = line.numbers_to_exclude(v);
        }
        line
    }
}

/// The Reference's `Axes.create_axis`, as a free function so the planes
/// family can build a z-axis the same way: a [`NumberLine`] over `range`
/// with `config` applied and `length` as its width, shifted by `-n2p(0)`.
///
/// The line is returned unbuilt — `n2p`/`p2n` are already exact; call
/// [`NumberLine::build`] (no font access needed) for geometry.
#[must_use]
pub fn create_axis(range: [f64; 3], config: AxisConfig, length: Option<f64>) -> NumberLine {
    let mut line = config.apply(NumberLine::new(range));
    if let Some(l) = length {
        line = line.width(l);
    }
    let shift = neg(line.n2p(0.0));
    line.shifted(shift)
}

// -----------------------------------------------------------------------
// Axes
// -----------------------------------------------------------------------

/// `Axes(x_range, y_range, axis_config, …)` (Appendix A
/// `mobject/coordinate_systems`).
///
/// Two [`NumberLine`]s built through the config merge, the y-axis rotated
/// 90° about the origin, the group centred — the Reference's constructor
/// verbatim. The built value implements [`CoordinateSystem`], and the
/// calculus helpers (`get_riemann_rectangles`, `get_area_under_graph`,
/// `get_tangent_line`, …) mirror the Reference's, taking the graphed
/// function directly (see the module docs).
#[derive(Debug, Clone)]
pub struct Axes {
    x_range: [f64; 3],
    y_range: [f64; 3],
    axis_config: AxisConfig,
    x_axis_config: AxisConfig,
    y_axis_config: AxisConfig,
    height: Option<f64>,
    width: Option<f64>,
    unit_size: f64,
    num_sampled_graph_points_per_tick: f64,
    sampling_budget: SamplingBudget,
    // --- built state -----------------------------------------------------
    x_axis: NumberLine,
    y_axis: NumberLine,
    vmob: VMobject,
}

impl Default for Axes {
    fn default() -> Self {
        Self::new()
    }
}

impl Axes {
    /// The Reference defaults: `x_range=(-8, 8, 1)`, `y_range=(-4, 4, 1)`,
    /// `unit_size=1`, no explicit height/width, 5 sampled graph points
    /// per tick.
    #[must_use]
    pub fn new() -> Self {
        Self {
            x_range: DEFAULT_X_RANGE,
            y_range: DEFAULT_Y_RANGE,
            axis_config: AxisConfig::default(),
            x_axis_config: AxisConfig::default(),
            y_axis_config: AxisConfig::default(),
            height: None,
            width: None,
            unit_size: 1.0,
            num_sampled_graph_points_per_tick: 5.0,
            sampling_budget: SamplingBudget::default(),
            x_axis: NumberLine::new(DEFAULT_X_RANGE),
            y_axis: NumberLine::new(DEFAULT_Y_RANGE),
            vmob: VMobject::new(),
        }
    }

    /// `x_range=` (tick step in the third component).
    #[must_use]
    pub fn x_range(mut self, range: [f64; 3]) -> Self {
        self.x_range = range;
        self
    }

    /// `y_range=`.
    #[must_use]
    pub fn y_range(mut self, range: [f64; 3]) -> Self {
        self.y_range = range;
        self
    }

    /// `axis_config=` — merged over the defaults, under the per-axis
    /// configs (the Reference's `merge_dicts_recursively` order).
    #[must_use]
    pub fn axis_config(mut self, config: AxisConfig) -> Self {
        self.axis_config = config;
        self
    }

    /// `x_axis_config=`.
    #[must_use]
    pub fn x_axis_config(mut self, config: AxisConfig) -> Self {
        self.x_axis_config = config;
        self
    }

    /// `y_axis_config=`.
    #[must_use]
    pub fn y_axis_config(mut self, config: AxisConfig) -> Self {
        self.y_axis_config = config;
        self
    }

    /// `height=` (the y-axis width, in the Reference's terms).
    #[must_use]
    pub fn height(mut self, height: f64) -> Self {
        self.height = Some(height);
        self
    }

    /// `width=` (the x-axis width).
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.width = Some(width);
        self
    }

    /// `unit_size=` — injected between `axis_config` and the per-axis
    /// configs, exactly the Reference's `dict(**axis_config,
    /// unit_size=unit_size)`.
    #[must_use]
    pub fn unit_size(mut self, unit_size: f64) -> Self {
        self.unit_size = unit_size;
        self
    }

    /// `num_sampled_graph_points_per_tick=` (Reference default 5).
    #[must_use]
    pub fn num_sampled_graph_points_per_tick(mut self, n: f64) -> Self {
        self.num_sampled_graph_points_per_tick = n;
        self
    }

    /// Bound ticks, default labels, rectangles, and graph samples.
    #[must_use]
    pub fn sampling_budget(mut self, budget: SamplingBudget) -> Self {
        self.sampling_budget = budget;
        self
    }

    /// The merged x-axis config:
    /// `default_axis < axis_config < {unit_size} < x_axis_config`.
    fn merged_x_config(&self) -> AxisConfig {
        AxisConfig::default()
            .merge(self.axis_config.clone())
            .merge(AxisConfig {
                unit_size: Some(self.unit_size),
                ..AxisConfig::default()
            })
            .merge(self.x_axis_config.clone())
    }

    /// The merged y-axis config, with the Reference's
    /// `default_y_axis_config = dict(line_to_number_direction=LEFT)`.
    fn merged_y_config(&self) -> AxisConfig {
        AxisConfig {
            line_to_number_direction: Some(LEFT),
            ..AxisConfig::default()
        }
        .merge(self.axis_config.clone())
        .merge(AxisConfig {
            unit_size: Some(self.unit_size),
            ..AxisConfig::default()
        })
        .merge(self.y_axis_config.clone())
    }

    /// Build both axes and centre the group.
    ///
    /// # Errors
    /// [`CoordsError`] from bounded tick sampling or number typesetting.
    pub fn build(mut self, book: &FontBook) -> Result<Self, CoordsError> {
        let x_axis = create_axis(self.x_range, self.merged_x_config(), self.width)
            .sampling_budget(self.sampling_budget)
            .build_numbered(book)?;
        let y_axis = create_axis(self.y_range, self.merged_y_config(), self.height)
            .sampling_budget(self.sampling_budget)
            .build_numbered(book)?
            .rotated_about(PI / 2.0, OUT, ORIGIN);
        let group = v_group([x_axis.vmob().clone(), y_axis.vmob().clone()]);
        let shift = neg(group.center_point());
        self.x_axis = x_axis.shifted(shift);
        self.y_axis = y_axis.shifted(shift);
        self.vmob = group.shifted(shift);
        Ok(self)
    }

    /// The x-axis.
    #[must_use]
    pub fn x_axis(&self) -> &NumberLine {
        &self.x_axis
    }

    /// The y-axis.
    #[must_use]
    pub fn y_axis(&self) -> &NumberLine {
        &self.y_axis
    }

    /// The built family (`[x_axis, y_axis]`).
    #[must_use]
    pub fn vmob(&self) -> &VMobject {
        &self.vmob
    }

    /// Consume into the family (for `Stage::add`).
    #[must_use]
    pub fn into_vmob(self) -> VMobject {
        self.vmob
    }

    /// `get_axis(index)`.
    #[must_use]
    pub fn axis(&self, index: usize) -> &NumberLine {
        if index == 0 {
            &self.x_axis
        } else {
            &self.y_axis
        }
    }

    // --- labels ------------------------------------------------------------

    /// `add_coordinate_labels`: numbers on both axes (`excluding=[0]` by
    /// default; values default to each axis's tick range). `font_size`
    /// overrides the [`DecimalNumber`] size used by both native number
    /// shelves. The labels land on the axes' [`NumberLine::numbers`] and
    /// the family is re-synced.
    ///
    /// # Errors
    /// [`CoordsError`] from bounded default-range sampling or typesetting.
    pub fn add_coordinate_labels(
        &mut self,
        book: &FontBook,
        x_values: Option<&[f64]>,
        y_values: Option<&[f64]>,
        excluding: Option<&[f64]>,
        font_size: Option<f64>,
    ) -> Result<(), CoordsError> {
        let excluding = excluding.unwrap_or(&[0.0]);
        if let Some(font_size) = font_size {
            self.x_axis.numbers_font_size = font_size;
            self.y_axis.numbers_font_size = font_size;
        }
        self.x_axis.add_numbers(book, x_values, Some(excluding))?;
        self.y_axis.add_numbers(book, y_values, Some(excluding))?;
        self.vmob = v_group([self.x_axis.vmob().clone(), self.y_axis.vmob().clone()]);
        Ok(())
    }

    // --- graphing ------------------------------------------------------------

    /// `get_graph`: the graph of `function` over this axes' x-range as a
    /// [`crate::graphs::ParametricCurve`] — the tick step becomes the
    /// sample step via `num_sampled_graph_points_per_tick`.
    #[must_use]
    pub fn get_graph(
        &self,
        function: impl Fn(f64) -> f64 + 'static,
    ) -> crate::graphs::ParametricCurve {
        let x_axis = self.x_axis.clone();
        let y_axis = self.y_axis.clone();
        crate::graphs::graph_parametric(
            function,
            move |coords| c2p_with_axes(&x_axis, &y_axis, coords),
            self.x_range,
            self.num_sampled_graph_points_per_tick,
            self.sampling_budget,
        )
    }

    /// `input_to_graph_point` for a graph with a known underlying
    /// function: `c2p(x, f(x))`. (The Reference's binary-search fallback
    /// for function-less graphs is not ported; see the module docs.)
    #[must_use]
    pub fn input_to_graph_point(&self, x: f64, function: &dyn Fn(f64) -> f64) -> Vec3 {
        self.c2p(&[x, function(x)])
    }

    /// `i2gp` alias.
    #[must_use]
    pub fn i2gp(&self, x: f64, function: &dyn Fn(f64) -> f64) -> Vec3 {
        self.input_to_graph_point(x, function)
    }

    // --- lines to points -----------------------------------------------------

    /// `get_line_from_axis_to_point(index, point, line_func=DashedLine,
    /// color=GREY_A, stroke_width=2)`.
    ///
    /// # Errors
    ///
    /// Returns [`CoordsError::Dash`] when the projected line cannot satisfy
    /// the bounded dashed-path contract.
    pub fn get_line_from_axis_to_point(
        &self,
        index: usize,
        point: Vec3,
    ) -> Result<VMobject, CoordsError> {
        let axis = self.axis(index);
        Ok(DashedLine::new(axis.projection(point), point)
            .style(Style::default().stroke(GREY_A, 2.0, 1.0))
            .build()?)
    }

    /// `get_v_line`: from the x-axis to the point.
    ///
    /// # Errors
    ///
    /// Returns [`CoordsError::Dash`] when the line cannot satisfy the bounded
    /// dashed-path contract.
    pub fn get_v_line(&self, point: Vec3) -> Result<VMobject, CoordsError> {
        self.get_line_from_axis_to_point(0, point)
    }

    /// `get_h_line`: from the y-axis to the point.
    ///
    /// # Errors
    ///
    /// Returns [`CoordsError::Dash`] when the line cannot satisfy the bounded
    /// dashed-path contract.
    pub fn get_h_line(&self, point: Vec3) -> Result<VMobject, CoordsError> {
        self.get_line_from_axis_to_point(1, point)
    }

    /// `get_v_line_to_graph(x, graph)`: [`get_v_line`](Self::get_v_line)
    /// to the graph point over `x`.
    ///
    /// # Errors
    /// As [`get_v_line`](Self::get_v_line).
    pub fn get_v_line_to_graph(
        &self,
        x: f64,
        function: &dyn Fn(f64) -> f64,
    ) -> Result<VMobject, CoordsError> {
        self.get_v_line(self.input_to_graph_point(x, function))
    }

    /// `get_h_line_to_graph(x, graph)`: [`get_h_line`](Self::get_h_line)
    /// to the graph point over `x`.
    ///
    /// # Errors
    /// As [`get_h_line`](Self::get_h_line).
    pub fn get_h_line_to_graph(
        &self,
        x: f64,
        function: &dyn Fn(f64) -> f64,
    ) -> Result<VMobject, CoordsError> {
        self.get_h_line(self.input_to_graph_point(x, function))
    }

    /// `get_parametric_curve(function, **kwargs)`: a [`ParametricCurve`]
    /// through `c2p` of `function(t)`'s first two coordinates, at the
    /// curve's own defaults (`t_range = (0, 1, 0.1)`), to be configured
    /// further before `build`.
    ///
    /// [`ParametricCurve`]: crate::graphs::ParametricCurve
    #[must_use]
    pub fn get_parametric_curve(
        &self,
        function: impl Fn(f64) -> Vec3 + 'static,
    ) -> crate::graphs::ParametricCurve {
        let x_axis = self.x_axis.clone();
        let y_axis = self.y_axis.clone();
        crate::graphs::ParametricCurve::new(move |t| {
            let coords = function(t);
            c2p_with_axes(&x_axis, &y_axis, &coords[..2])
        })
        .sampling_budget(self.sampling_budget)
    }

    /// `get_scatterplot(x_values, y_values, **dot_config)`: a [`DotCloud`]
    /// at `c2p(x, y)` for each pair (extra values on the longer side are
    /// ignored, as numpy's pairwise `c2p` would refuse them).
    #[must_use]
    pub fn get_scatterplot(&self, x_values: &[f64], y_values: &[f64]) -> DotCloud {
        DotCloud::new(
            x_values
                .iter()
                .zip(y_values)
                .map(|(&x, &y)| self.c2p(&[x, y])),
        )
    }

    // --- labels ---------------------------------------------------------------

    /// `get_axis_label(label_tex, axis, edge, direction, buff, ensure_on_screen)`:
    /// a `Tex` placed `buff` away from the axis's `edge` centre, on the
    /// `direction` side, then (when asked) shifted onto the default frame.
    ///
    /// # Errors
    /// [`CoordsError::Tex`] when the label fails to typeset.
    #[allow(clippy::too_many_arguments)]
    pub fn get_axis_label(
        &self,
        engine: &TexEngine,
        label_tex: &str,
        axis: usize,
        edge: Vec3,
        direction: Vec3,
        buff: f64,
        ensure_on_screen: bool,
    ) -> Result<VMobject, CoordsError> {
        let label = Tex::new(label_tex).build(engine)?.vmob.moved_to(ORIGIN);
        Ok(self.place_axis_label(label, axis, edge, direction, buff, ensure_on_screen))
    }

    /// The placement half of [`get_axis_label`](Self::get_axis_label), for
    /// a label built some other way (a `Text`, a styled `Tex`, a group).
    #[must_use]
    pub fn place_axis_label(
        &self,
        label: VMobject,
        axis: usize,
        edge: Vec3,
        direction: Vec3,
        buff: f64,
        ensure_on_screen: bool,
    ) -> VMobject {
        let axis = self.axis(axis).vmob();
        let anchor = axis.bbox_point(edge).unwrap_or(ORIGIN);
        let label = label.next_to_point(anchor, direction, buff, ORIGIN);
        if ensure_on_screen {
            label.shifted_onto_screen(MED_SMALL_BUFF)
        } else {
            label
        }
    }

    /// `get_x_axis_label(label_tex, edge=RIGHT, direction=DL)`.
    ///
    /// # Errors
    /// As [`get_axis_label`](Self::get_axis_label).
    pub fn get_x_axis_label(
        &self,
        engine: &TexEngine,
        label_tex: &str,
    ) -> Result<VMobject, CoordsError> {
        self.get_axis_label(engine, label_tex, 0, RIGHT, DL, MED_SMALL_BUFF, false)
    }

    /// `get_y_axis_label(label_tex, edge=UP, direction=DR)`.
    ///
    /// # Errors
    /// As [`get_axis_label`](Self::get_axis_label).
    pub fn get_y_axis_label(
        &self,
        engine: &TexEngine,
        label_tex: &str,
    ) -> Result<VMobject, CoordsError> {
        self.get_axis_label(engine, label_tex, 1, UP, DR, MED_SMALL_BUFF, false)
    }

    /// `get_axis_labels(x_label_tex="x", y_label_tex="y")`: both labels as
    /// one group, x first.
    ///
    /// # Errors
    /// As [`get_axis_label`](Self::get_axis_label).
    pub fn get_axis_labels(
        &self,
        engine: &TexEngine,
        x_label_tex: &str,
        y_label_tex: &str,
    ) -> Result<VMobject, CoordsError> {
        Ok(v_group([
            self.get_x_axis_label(engine, x_label_tex)?,
            self.get_y_axis_label(engine, y_label_tex)?,
        ]))
    }

    /// `get_graph_label(graph, label, x=None, buff=MED_SMALL_BUFF, color=None)`
    /// for the graph of `function` (drawn as `graph`).
    ///
    /// With no `x`, the Reference searches `arange(*x_range)` from the right
    /// for the first graph point whose `|x|` and `|y|` leave room for the
    /// label inside the default frame, falling back to the range's end. The
    /// label sits `buff` off the graph along the upward tangent normal and is
    /// then shifted onto the frame.
    ///
    /// Colour: with no `color`, the label takes the graph's colour (its fill
    /// colour if it has a visible fill, else its stroke colour — the
    /// Reference's `match_color`). A given `color` is applied; the Reference
    /// accepts the parameter but never uses it.
    ///
    /// # Errors
    /// [`CoordsError::Sampling`] when the default search range exceeds the
    /// axes' sampling budget.
    pub fn get_graph_label(
        &self,
        function: &dyn Fn(f64) -> f64,
        graph: &VMobject,
        label: VMobject,
        x: Option<f64>,
        buff: f64,
        color: Option<Srgb>,
    ) -> Result<VMobject, CoordsError> {
        let color = color.unwrap_or_else(|| {
            let style = graph.style();
            if style.fill_opacity > 0.0 {
                style.fill_color
            } else {
                style.stroke_color
            }
        });
        let label = label.map_style_deep(|style| style.color(color));
        let x = match x {
            Some(x) => x,
            None => {
                let max_y = FRAME_Y_RADIUS - label.length_over_dim(1);
                let max_x = FRAME_X_RADIUS - label.length_over_dim(0);
                let candidates = arange(
                    "get_graph_label",
                    self.x_range[0],
                    self.x_range[1],
                    self.x_range[2],
                    self.sampling_budget,
                )?;
                candidates
                    .into_iter()
                    .rev()
                    .find(|&x0| {
                        let point = self.input_to_graph_point(x0, function);
                        point[0].abs() < max_x && point[1].abs() < max_y
                    })
                    .unwrap_or(self.x_range[1])
            }
        };
        let point = self.input_to_graph_point(x, function);
        let angle = self.angle_of_tangent(x, function);
        let mut normal = space_ops::rotate_vector(RIGHT, angle + PI / 2.0, OUT);
        if normal[1] < 0.0 {
            normal = [-normal[0], -normal[1], -normal[2]];
        }
        Ok(label
            .next_to_point(point, normal, buff, ORIGIN)
            .shifted_onto_screen(DEFAULT_MOBJECT_TO_EDGE_BUFF))
    }

    /// [`get_graph_label`](Self::get_graph_label) with a `Tex` label.
    ///
    /// # Errors
    /// [`CoordsError::Tex`] when the label fails to typeset, else as
    /// [`get_graph_label`](Self::get_graph_label).
    pub fn get_graph_label_tex(
        &self,
        engine: &TexEngine,
        function: &dyn Fn(f64) -> f64,
        graph: &VMobject,
        label_tex: &str,
        x: Option<f64>,
        color: Option<Srgb>,
    ) -> Result<VMobject, CoordsError> {
        let label = Tex::new(label_tex).build(engine)?.vmob.moved_to(ORIGIN);
        self.get_graph_label(function, graph, label, x, MED_SMALL_BUFF, color)
    }

    // --- calculus -------------------------------------------------------------

    /// `angle_of_tangent(x, graph, dx=EPSILON)`: the angle of the secant
    /// from `x` to `x + dx` on the graph of `function`.
    #[must_use]
    pub fn angle_of_tangent(&self, x: f64, function: &dyn Fn(f64) -> f64) -> f64 {
        self.angle_of_tangent_dx(x, function, EPSILON)
    }

    /// The `dx`-explicit form of [`angle_of_tangent`](Self::angle_of_tangent).
    #[must_use]
    pub fn angle_of_tangent_dx(&self, x: f64, function: &dyn Fn(f64) -> f64, dx: f64) -> f64 {
        let p0 = self.input_to_graph_point(x, function);
        let p1 = self.input_to_graph_point(x + dx, function);
        space_ops::angle_of_vector(sub(p1, p0))
    }

    /// `slope_of_tangent`: `tan(angle_of_tangent(x))`.
    #[must_use]
    pub fn slope_of_tangent(&self, x: f64, function: &dyn Fn(f64) -> f64) -> f64 {
        fmn_dmath::tan(self.angle_of_tangent(x, function))
    }

    /// `get_tangent_line(x, graph, length=5)`: a `Line(LEFT, RIGHT)`
    /// resized to `length`, rotated to the tangent angle, centred on the
    /// graph point.
    #[must_use]
    pub fn get_tangent_line(&self, x: f64, function: &dyn Fn(f64) -> f64, length: f64) -> VMobject {
        let line = Line::new(LEFT, RIGHT)
            .build()
            .expect("the tangent template is always a straight segment")
            .with_width(length, false);
        let angle = self.angle_of_tangent(x, function);
        let center = line.center_point();
        line.rotated_about(angle, OUT, center)
            .moved_to(self.input_to_graph_point(x, function))
    }

    /// `get_riemann_rectangles` with the Reference's default styling
    /// (stroke 1 BLACK, fill opacity 1, BLUE→GREEN gradient, RED
    /// negatives, stroke behind).
    ///
    /// # Errors
    /// [`CoordsError::InvalidSampleType`] for a sample type outside
    /// `left | right | center`, or [`CoordsError::Sampling`] when the
    /// requested rectangle range exceeds the configured budget.
    pub fn get_riemann_rectangles(
        &self,
        function: &dyn Fn(f64) -> f64,
        x_range: Option<[f64; 2]>,
        dx: Option<f64>,
        input_sample_type: &str,
    ) -> Result<VMobject, CoordsError> {
        self.riemann_rectangles_styled(
            function,
            x_range,
            dx,
            input_sample_type,
            &RiemannConfig::default(),
        )
    }

    /// The styled form of [`get_riemann_rectangles`](Self::get_riemann_rectangles),
    /// under BN-20 (`docs/behavior_notes/BN-20-calculus-geometry.md`):
    ///
    /// * **The integration interval is authoritative.** Bins start every
    ///   `dx` from the lower bound; a final partial bin ends exactly at the
    ///   upper bound (the Reference extends the range by `dx` and can
    ///   overshoot it). A step count within one unit in the last place of an
    ///   integer is that integer, so no sliver bin appears; a zero-length
    ///   interval is an empty group.
    /// * **Rectangles are built in the axes' chart**: the corners are
    ///   `c2p(x0, 0)`, `c2p(x1, 0)`, `c2p(x1, h)`, `c2p(x0, h)` with
    ///   `h = function(sample)`, so rotated or sheared axes keep their bins
    ///   (the Reference measures width and sign in world space).
    /// * `show_signed_area` decides whether a negative bin (`h < 0`) takes
    ///   `negative_color`; zero-height bins are nonnegative.
    ///
    /// The sample is the bin's left end, right end or midpoint by
    /// `input_sample_type`; colours run the `colors` gradient across bins.
    ///
    /// # Errors
    /// [`CoordsError::InvalidSampleType`] for a sample type outside
    /// `left | right | center`; [`CoordsError::Sampling`] for a non-finite
    /// bound or step, or more bins than the configured budget;
    /// [`CoordsError::InvalidRiemannRange`] for a decreasing range, a
    /// non-positive step, or a step too small to advance the bound.
    pub fn riemann_rectangles_styled(
        &self,
        function: &dyn Fn(f64) -> f64,
        x_range: Option<[f64; 2]>,
        dx: Option<f64>,
        input_sample_type: &str,
        config: &RiemannConfig,
    ) -> Result<VMobject, CoordsError> {
        if !matches!(input_sample_type, "left" | "right" | "center") {
            return Err(CoordsError::InvalidSampleType(input_sample_type.to_owned()));
        }
        let [start, stop] = x_range.unwrap_or([self.x_range[0], self.x_range[1]]);
        let step = dx.unwrap_or(self.x_range[2]);
        let bins = riemann_bins(start, stop, step, self.sampling_budget)?;
        let gradient = if config.colors.len() >= 2 {
            color_gradient(&config.colors, bins.len())
        } else {
            vec![config.colors.first().copied().unwrap_or(BLUE); bins.len()]
        };
        let mut rects = Vec::with_capacity(bins.len());
        for (i, &(x0, x1)) in bins.iter().enumerate() {
            let sample = match input_sample_type {
                "right" => x1,
                "center" => x0 + (x1 - x0) / 2.0,
                _ => x0,
            };
            let height = function(sample);
            let negative = height < 0.0;
            let fill = if negative && config.show_signed_area {
                config.negative_color
            } else {
                gradient.get(i).copied().unwrap_or(GREEN)
            };
            let style = Style::default()
                .stroke(config.stroke_color, config.stroke_width, 1.0)
                .fill(fill, config.fill_opacity);
            let corners = [
                self.c2p(&[x0, 0.0]),
                self.c2p(&[x1, 0.0]),
                self.c2p(&[x1, height]),
                self.c2p(&[x0, height]),
            ];
            let rect = Polygon::new(corners)
                .style(style)
                .build()
                .with_stroke_behind(config.stroke_behind);
            rects.push(rect);
        }
        Ok(v_group(rects))
    }

    /// `get_area_under_graph(graph, x_range, fill_color=BLUE,
    /// fill_opacity=0.5)`.
    ///
    /// `graph` is a built curve (e.g. from [`get_graph`](Self::get_graph))
    /// and `graph_x_range` the x-range it was sampled over — the pair the
    /// Reference keeps on the mobject itself. The curve is cut between
    /// the alpha bounds `inverse_interpolate(graph_x_range, x)` (curve-index
    /// space, as the Reference's `pointwise_become_partial`), then closed
    /// through `c2p(x1, 0)` and `c2p(x0, 0)` back to its start.
    #[must_use]
    pub fn get_area_under_graph(
        &self,
        graph: &VMobject,
        graph_x_range: [f64; 2],
        x_range: Option<[f64; 2]>,
    ) -> VMobject {
        self.area_under_graph_styled(graph, graph_x_range, x_range, BLUE, 0.5)
    }

    /// The styled form of [`get_area_under_graph`](Self::get_area_under_graph).
    #[must_use]
    pub fn area_under_graph_styled(
        &self,
        graph: &VMobject,
        graph_x_range: [f64; 2],
        x_range: Option<[f64; 2]>,
        fill_color: Srgb,
        fill_opacity: f64,
    ) -> VMobject {
        let style = Style::default()
            .stroke_width(0.0)
            .fill(fill_color, fill_opacity);
        let xr = x_range.unwrap_or_else(|| match (graph.points().first(), graph.points().last()) {
            (Some(&s), Some(&e)) => [self.x_axis.p2n(s), self.x_axis.p2n(e)],
            _ => [0.0, 0.0],
        });
        let a0 = inverse_interpolate(graph_x_range[0], graph_x_range[1], xr[0]);
        let a1 = inverse_interpolate(graph_x_range[0], graph_x_range[1], xr[1]);
        let Some((points, _, _)) = QuadPath::partial_points(graph.points(), a0, a1) else {
            return VMobject::new().with_style(style);
        };
        let Ok(mut path) = QuadPath::from_points(points) else {
            return VMobject::new().with_style(style);
        };
        let start = path.points().first().copied();
        let _ = path.add_line_to(self.c2p(&[xr[1], 0.0]), true);
        let _ = path.add_line_to(self.c2p(&[xr[0], 0.0]), true);
        if let Some(s) = start {
            let _ = path.add_line_to(s, true);
        }
        VMobject::from_path(&path).with_style(style)
    }
}

impl CoordinateSystem for Axes {
    fn c2p(&self, coords: &[f64]) -> Vec3 {
        c2p_with_axes(&self.x_axis, &self.y_axis, coords)
    }

    fn p2c(&self, point: Vec3) -> [f64; 3] {
        [self.x_axis.p2n(point), self.y_axis.p2n(point), 0.0]
    }

    fn all_ranges(&self) -> Vec<[f64; 3]> {
        vec![self.x_range, self.y_range]
    }

    fn num_sampled_graph_points_per_tick(&self) -> f64 {
        self.num_sampled_graph_points_per_tick
    }

    fn graph_sampling_budget(&self) -> SamplingBudget {
        self.sampling_budget
    }
}

impl From<Axes> for fmn_mobject::Mobject {
    fn from(axes: Axes) -> Self {
        axes.into_vmob().into()
    }
}

impl From<NumberLine> for fmn_mobject::Mobject {
    fn from(line: NumberLine) -> Self {
        line.into_vmob().into()
    }
}

/// The Reference's `coords_to_point`:
/// `origin + Σ (axis.n2p(coord) − origin)`, with `origin = x_axis.n2p(0)`.
fn c2p_with_axes(x_axis: &NumberLine, y_axis: &NumberLine, coords: &[f64]) -> Vec3 {
    let origin = x_axis.n2p(0.0);
    let mut result = origin;
    if let Some(&x) = coords.first() {
        result = add(result, sub(x_axis.n2p(x), origin));
    }
    if let Some(&y) = coords.get(1) {
        result = add(result, sub(y_axis.n2p(y), origin));
    }
    result
}

/// `get_riemann_rectangles`' style surface, with the Reference's defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct RiemannConfig {
    /// `stroke_width=1`.
    pub stroke_width: f64,
    /// `stroke_color=BLACK`.
    pub stroke_color: Srgb,
    /// `fill_opacity=1`.
    pub fill_opacity: f64,
    /// `colors=(BLUE, GREEN)` — the gradient across the rectangles.
    pub colors: Vec<Srgb>,
    /// `negative_color=RED`.
    pub negative_color: Srgb,
    /// `stroke_background=True`.
    pub stroke_behind: bool,
    /// `show_signed_area=True`: negative bins take `negative_color`; with
    /// `false` they keep their gradient colour (BN-20 — the Reference
    /// accepts the parameter and never reads it).
    pub show_signed_area: bool,
}

impl Default for RiemannConfig {
    fn default() -> Self {
        Self {
            stroke_width: 1.0,
            stroke_color: BLACK,
            fill_opacity: 1.0,
            colors: vec![BLUE, GREEN],
            negative_color: RED,
            stroke_behind: true,
            show_signed_area: true,
        }
    }
}

// -----------------------------------------------------------------------
// UnitInterval + Slider
// -----------------------------------------------------------------------

/// `UnitInterval(NumberLine)`: the `(0, 1, 0.1)` preset — `unit_size=10`,
/// big ticks at `[0, 1]`, one decimal place. A constructor surface only:
/// it produces a configured [`NumberLine`].
pub struct UnitInterval;

impl UnitInterval {
    /// The Reference's preset constructor. (Returns the configured
    /// [`NumberLine`] — the preset is a factory, not a wrapper.)
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new() -> NumberLine {
        NumberLine::new([0.0, 1.0, 0.1])
            .unit_size(10.0)
            .big_tick_numbers(vec![0.0, 1.0])
            .num_decimal_places(1)
    }
}

/// `Slider(value_tracker, …)` (`number_line.py`) — a number line with a
/// tip riding it and a value label, assembled at an initial value.
///
/// The Reference wires tip and label to a `ValueTracker` via updaters;
/// updater wiring is Proscenium's (W9), so this is the static snapshot at
/// `value`. The TeX label (`"x = 0.00"`) is a native `Text` prefix +
/// [`DecimalNumber`] composition (BN-08) — the Reference colours the
/// variable name `arrow_color`, and so does the prefix here.
#[derive(Debug, Clone)]
pub struct Slider {
    value: f64,
    x_range: [f64; 2],
    var_name: Option<String>,
    width: f64,
    #[allow(dead_code)] // stored parity: the Reference accepts `unit_size` and never reads it
    unit_size: f64,
    arrow_width: f64,
    arrow_length: f64,
    arrow_color: Srgb,
    font_size: f64,
    label_buff: f64,
    num_decimal_places: usize,
    tick_size: f64,
    angle: f64,
    label_direction: Option<Vec3>,
    add_tick_labels: bool,
    tick_label_font_size: f64,
    sampling_budget: SamplingBudget,
}

impl Slider {
    /// The Reference defaults: `x_range=(-5, 5)`, `width=3`, 0.15×0.15
    /// YELLOW tip, `font_size=24`, `label_buff=SMALL_BUFF`, two decimal
    /// places, `tick_size=0.05`, tick labels on at size 16.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self {
            value,
            x_range: [-5.0, 5.0],
            var_name: None,
            width: 3.0,
            unit_size: 1.0,
            arrow_width: 0.15,
            arrow_length: 0.15,
            arrow_color: YELLOW,
            font_size: 24.0,
            label_buff: SMALL_BUFF,
            num_decimal_places: 2,
            tick_size: 0.05,
            angle: 0.0,
            label_direction: None,
            add_tick_labels: true,
            tick_label_font_size: 16.0,
            sampling_budget: SamplingBudget::default(),
        }
    }

    /// `x_range=` (a `(min, max)` pair; the step stays 1).
    #[must_use]
    pub fn x_range(mut self, range: [f64; 2]) -> Self {
        self.x_range = range;
        self
    }

    /// `var_name=` — the `"x = "` label prefix.
    #[must_use]
    pub fn var_name(mut self, name: &str) -> Self {
        self.var_name = Some(name.to_owned());
        self
    }

    /// `width=` (default 3).
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// `arrow_width=` / `arrow_length=` (default 0.15).
    #[must_use]
    pub fn arrow_size(mut self, width: f64, length: f64) -> Self {
        self.arrow_width = width;
        self.arrow_length = length;
        self
    }

    /// `arrow_color=` (default `YELLOW`).
    #[must_use]
    pub fn arrow_color(mut self, color: Srgb) -> Self {
        self.arrow_color = color;
        self
    }

    /// `font_size=` for the value label (default 24).
    #[must_use]
    pub fn font_size(mut self, font_size: f64) -> Self {
        self.font_size = font_size;
        self
    }

    /// `label_buff=` (default `SMALL_BUFF`).
    #[must_use]
    pub fn label_buff(mut self, buff: f64) -> Self {
        self.label_buff = buff;
        self
    }

    /// `num_decimal_places=` (default 2).
    #[must_use]
    pub fn num_decimal_places(mut self, ndp: usize) -> Self {
        self.num_decimal_places = ndp;
        self
    }

    /// `tick_size=` (default 0.05).
    #[must_use]
    pub fn tick_size(mut self, size: f64) -> Self {
        self.tick_size = size;
        self
    }

    /// `angle=` — the whole line's rotation (default 0).
    #[must_use]
    pub fn angle(mut self, angle: f64) -> Self {
        self.angle = angle;
        self
    }

    /// `label_direction=` (default `round(rotate_vector(UP, angle), 2)`).
    #[must_use]
    pub fn label_direction(mut self, direction: Vec3) -> Self {
        self.label_direction = Some(direction);
        self
    }

    /// `add_tick_labels=` (default true).
    #[must_use]
    pub fn add_tick_labels(mut self, on: bool) -> Self {
        self.add_tick_labels = on;
        self
    }

    /// `tick_label_font_size=` (default 16).
    #[must_use]
    pub fn tick_label_font_size(mut self, size: f64) -> Self {
        self.tick_label_font_size = size;
        self
    }

    /// Bound the line's tick and default-label sampling.
    #[must_use]
    pub fn sampling_budget(mut self, budget: SamplingBudget) -> Self {
        self.sampling_budget = budget;
        self
    }

    /// The resolved label direction (`np.round(rotate_vector(UP, angle), 2)`).
    fn resolved_label_direction(&self) -> Vec3 {
        self.label_direction.unwrap_or_else(|| {
            let d = space_ops::rotate_vector(UP, self.angle, OUT);
            [
                (d[0] * 100.0).round() / 100.0,
                (d[1] * 100.0).round() / 100.0,
                (d[2] * 100.0).round() / 100.0,
            ]
        })
    }

    /// Assemble the group `[number_line, tip, label]` at the initial
    /// value, stroke behind throughout (the Reference's closing
    /// `set_stroke(behind=True)`).
    ///
    /// # Errors
    /// [`CoordsError`] from bounded sampling or typesetting.
    pub fn build(&self, book: &FontBook) -> Result<VMobject, CoordsError> {
        let label_direction = self.resolved_label_direction();

        // The number line, rotated by `angle`, with tick labels on the
        // side opposite the label (`direction=-label_direction`,
        // `buff=2*tick_size`).
        let line = NumberLine::with_min_max(self.x_range[0], self.x_range[1])
            .width(self.width)
            .tick_size(self.tick_size)
            .line_to_number_direction(neg(label_direction))
            .line_to_number_buff(2.0 * self.tick_size)
            .numbers_font_size(self.tick_label_font_size)
            .include_numbers(self.add_tick_labels)
            .sampling_budget(self.sampling_budget)
            .build_numbered(book)?
            .rotated_about(self.angle, OUT, ORIGIN);

        // The tip at the tracked value, back edge on the line.
        let tip = ArrowTip::new()
            .width(self.arrow_width)
            .length(self.arrow_length)
            .color(self.arrow_color)
            .angle(-PI + space_ops::angle_of_vector(label_direction))
            .build()
            .moved_to_aligned(line.n2p(self.value), neg(label_direction));

        // The label: native prefix + decimal, beside the tip.
        let decimal = DecimalNumber::new(self.value)
            .num_decimal_places(self.num_decimal_places)
            .font_size(self.font_size)
            .build(book)?;
        let label = match &self.var_name {
            Some(name) => {
                let prefix = Text::new(&format!("{name} ="))
                    .font_size(self.font_size)
                    .style(text_style().color(self.arrow_color))
                    .build(book)?;
                let value = decimal.vmob().clone().next_to(
                    &prefix.vmob,
                    RIGHT,
                    0.1 * self.font_size / 48.0,
                    ORIGIN,
                );
                v_group([prefix.vmob, value])
            }
            None => decimal.vmob().clone(),
        };
        let label = label.next_to(&tip, label_direction, self.label_buff, ORIGIN);

        let group = v_group([line.into_vmob(), tip, label]);
        Ok(stroke_behind_deep(group))
    }
}

/// The Reference's `set_stroke(behind=True)`: `stroke_behind` on the
/// whole family, recursively.
fn stroke_behind_deep(vmob: VMobject) -> VMobject {
    vmob.with_stroke_behind(true)
        .map_children(stroke_behind_deep)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn book() -> FontBook {
        FontBook::bundled().expect("bundled faces parse")
    }

    fn assert_vec3_near(actual: Vec3, expected: Vec3, tol: f64, what: &str) {
        for k in 0..3 {
            assert!(
                (actual[k] - expected[k]).abs() < tol,
                "{what}: component {k}: {} vs {} (tol {tol})",
                actual[k],
                expected[k]
            );
        }
    }

    use fmn_core::constants::{UL, UR};

    fn tex_engine() -> TexEngine {
        TexEngine::new("fmd-math/pack/default", None).expect("bundled math pack")
    }

    fn unit_square(side: f64) -> VMobject {
        crate::poly::Square::new().side_length(side).build()
    }

    #[test]
    fn axis_labels_sit_at_the_reference_corners() {
        let axes = Axes::new().build(&book()).expect("build axes");
        let engine = tex_engine();
        // x: the label's UR corner sits `buff` down-left of the x-axis's
        // right edge centre (`next_to(edge_center, DL, buff)`).
        let x_label = axes.get_x_axis_label(&engine, "x").expect("x label");
        let right = axes.x_axis().vmob().bbox_point(RIGHT).expect("x axis");
        assert_vec3_near(
            x_label.bbox_point(UR).expect("label"),
            [
                right[0] - MED_SMALL_BUFF,
                right[1] - MED_SMALL_BUFF,
                right[2],
            ],
            1e-9,
            "x label corner",
        );
        // y: the label's UL corner sits `buff` down-right of the top.
        let y_label = axes.get_y_axis_label(&engine, "y").expect("y label");
        let top = axes.y_axis().vmob().bbox_point(UP).expect("y axis");
        assert_vec3_near(
            y_label.bbox_point(UL).expect("label"),
            [top[0] + MED_SMALL_BUFF, top[1] - MED_SMALL_BUFF, top[2]],
            1e-9,
            "y label corner",
        );
        let both = axes.get_axis_labels(&engine, "x", "y").expect("labels");
        assert_eq!(both.children().len(), 2);
        assert_eq!(both.children()[0].extent(), x_label.extent());
        assert_eq!(both.children()[1].extent(), y_label.extent());
        // ensure_on_screen pulls a label that would cross the frame back in.
        let placed = axes.place_axis_label(unit_square(0.5), 0, RIGHT, RIGHT, 0.0, true);
        let edge = placed.bbox_point(RIGHT).expect("placed")[0];
        assert!(
            (edge - (FRAME_X_RADIUS - MED_SMALL_BUFF)).abs() < 1e-9,
            "{edge}"
        );
    }

    #[test]
    fn graph_label_follows_the_upward_normal_then_the_frame() {
        let axes = Axes::new().build(&book()).expect("build axes");
        let f = |x: f64| 0.5 * x;
        let graph = axes.get_graph(f).color(BLUE).build().expect("graph");
        let label = unit_square(0.5);
        // Explicit x on the line y = x/2: the normal is the slope's upward
        // perpendicular, and the label's lower-right corner (its side facing
        // the graph) sits `buff` along it.
        let placed = axes
            .get_graph_label(&f, &graph, label.clone(), Some(1.0), MED_SMALL_BUFF, None)
            .expect("label");
        let angle = 0.5f64.atan();
        let normal = [-angle.sin(), angle.cos(), 0.0];
        let point = axes.c2p(&[1.0, 0.5]);
        assert_vec3_near(
            placed.bbox_point(DR).expect("placed"),
            [
                point[0] + normal[0] * MED_SMALL_BUFF,
                point[1] + normal[1] * MED_SMALL_BUFF,
                0.0,
            ],
            1e-6,
            "explicit x",
        );
        assert_eq!(placed.style().fill_color, BLUE, "matches the graph stroke");
        // No x: from the right, 7 leaves no room for a 0.5-wide label
        // (|7| >= 7.11 - 0.5), 6 does; the label then crosses the top
        // (3.72 > 4 - 0.5) and `shift_onto_screen` brings its top to 3.5.
        let searched = axes
            .get_graph_label(&f, &graph, label, None, MED_SMALL_BUFF, Some(RED))
            .expect("label");
        let point = axes.c2p(&[6.0, 3.0]);
        let corner = searched.bbox_point(DR).expect("searched");
        assert!((corner[0] - (point[0] + normal[0] * MED_SMALL_BUFF)).abs() < 1e-6);
        let top = searched.bbox_point(UP).expect("searched")[1];
        assert!((top - (FRAME_Y_RADIUS - DEFAULT_MOBJECT_TO_EDGE_BUFF)).abs() < 1e-9);
        assert_eq!(
            searched.style().fill_color,
            RED,
            "an explicit colour is applied"
        );
    }

    #[test]
    fn graph_lines_scatterplots_and_parametric_curves_go_through_c2p() {
        let axes = Axes::new().build(&book()).expect("build axes");
        let f = |x: f64| x * x / 4.0;
        let v = axes.get_v_line_to_graph(2.0, &f).expect("v line");
        let expected = axes.get_v_line(axes.c2p(&[2.0, 1.0])).expect("v line");
        assert_eq!(v.extent(), expected.extent());
        let h = axes.get_h_line_to_graph(-2.0, &f).expect("h line");
        let expected = axes.get_h_line(axes.c2p(&[-2.0, 1.0])).expect("h line");
        assert_eq!(h.extent(), expected.extent());

        let cloud = axes.get_scatterplot(&[0.0, 1.0, 2.0], &[1.0, -1.0]);
        let mobject: fmn_mobject::Mobject = cloud.into();
        let points: Vec<Vec3> = mobject
            .buffer
            .read_column("point")
            .expect("points")
            .as_chunks::<3>()
            .0
            .iter()
            .map(|p| [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])])
            .collect();
        assert_eq!(points.len(), 2, "pairs, not the longer side");
        assert_vec3_near(points[1], axes.c2p(&[1.0, -1.0]), 1e-6, "second dot");

        let curve = axes
            .get_parametric_curve(|t| [t.cos(), t.sin(), 0.0])
            .build()
            .expect("curve");
        let points = curve.points();
        assert_vec3_near(points[0], axes.c2p(&[1.0, 0.0]), 1e-9, "start");
        assert_vec3_near(
            *points.last().expect("end"),
            axes.c2p(&[1.0f64.cos(), 1.0f64.sin()]),
            1e-9,
            "end",
        );
    }

    /// A deterministic splitmix64 — property tests without a rand dep.
    struct Lcg(u64);

    impl Lcg {
        fn next_f64(&mut self) -> f64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^= z >> 31;
            ((z >> 11) as f64) / ((1u64 << 53) as f64)
        }

        fn range(&mut self, lo: f64, hi: f64) -> f64 {
            lo + (hi - lo) * self.next_f64()
        }
    }

    // --- NumberLine geometry --------------------------------------------

    #[test]
    fn n2p_p2n_are_exact_inverses_across_ranges_and_scales() {
        let cases: [NumberLine; 4] = [
            NumberLine::new([-8.0, 8.0, 1.0]),
            NumberLine::new([-3.0, 5.0, 0.5]),
            NumberLine::new([2.0, 9.0, 1.0]).unit_size(0.5),
            NumberLine::new([0.0, 1.0, 0.1]).width(6.0),
        ];
        let mut rng = Lcg(0xC0_0F_FE);
        for line in &cases {
            for _ in 0..100 {
                let x = rng.range(line.x_min() - 2.0, line.x_max() + 2.0);
                let back = line.p2n(line.n2p(x));
                assert!(
                    (back - x).abs() < 1e-12,
                    "range {:?}: n2p→p2n drifted to {back} from {x}",
                    line.x_range()
                );
            }
        }
    }

    #[test]
    fn width_overrides_unit_size() {
        let line = NumberLine::new([0.0, 5.0, 1.0]).width(10.0);
        assert!((line.effective_unit_size() - 2.0).abs() < 1e-15);
        assert_vec3_near(line.n2p(5.0), [5.0, 0.0, 0.0], 1e-12, "n2p(5)");
        assert_vec3_near(line.n2p(0.0), [-5.0, 0.0, 0.0], 1e-12, "n2p(0)");
    }

    #[test]
    fn stroke_color_wins_over_color_for_the_stroke_in_either_order() {
        // VMobject.__init__: `stroke_color or color` for the stroke, `color`
        // for the fill, however the axis config dicts merged.
        let red = Srgb {
            r: 1.0,
            g: 0.0,
            b: 0.0,
        };
        let blue = Srgb {
            r: 0.0,
            g: 0.0,
            b: 1.0,
        };
        let color = AxisConfig {
            color: Some(red),
            ..AxisConfig::default()
        };
        let stroke = AxisConfig {
            stroke_color: Some(blue),
            ..AxisConfig::default()
        };
        for config in [
            color.clone().merge(stroke.clone()),
            stroke.clone().merge(color.clone()),
        ] {
            let line = config.apply(NumberLine::new([-2.0, 2.0, 1.0]));
            assert_eq!(line.style.stroke_color, blue);
            assert_eq!(line.style.fill_color, red);
        }
        let only_color = color.apply(NumberLine::new([-2.0, 2.0, 1.0]));
        assert_eq!(only_color.style.stroke_color, red);
    }

    #[test]
    fn tick_positions_and_longer_ticks_match_the_reference() {
        // Ticks at every integer of [-2, 2]; big ticks at 0 and ±2.
        let line = NumberLine::new([-2.0, 2.0, 1.0])
            // Explicit inputs need not be sorted; non-finite candidates
            // never match a finite tick under numpy `isclose`.
            .big_tick_numbers(vec![f64::INFINITY, 2.0, -2.0, f64::NAN, 0.0])
            .build()
            .expect("fixture ticks are bounded");
        let ticks = &line.vmob().children()[0];
        assert_eq!(ticks.children().len(), 5);
        for (i, tick) in ticks.children().iter().enumerate() {
            let x = i as f64 - 2.0;
            let expected_size = if x == 0.0 || x.abs() == 2.0 {
                0.1 * 1.5
            } else {
                0.1
            };
            assert_vec3_near(tick.center_point(), line.n2p(x), 1e-12, "tick centre");
            assert!(
                (tick.length_over_dim(1) - 2.0 * expected_size).abs() < 1e-12,
                "tick at {x}: height {} vs {}",
                tick.length_over_dim(1),
                2.0 * expected_size
            );
            // A tick is a vertical segment through its centre.
            assert!(
                tick.length_over_dim(0) < 1e-12,
                "tick at {x} is not vertical"
            );
        }
    }

    #[test]
    fn sorted_isclose_membership_matches_the_reference_scan() {
        let candidates = vec![
            f64::NAN,
            f64::INFINITY,
            -f64::MAX,
            -1.0,
            -0.0,
            0.999_98,
            1.000_005,
            f64::MAX,
            f64::NEG_INFINITY,
        ];
        let mut sorted: Vec<_> = candidates
            .iter()
            .copied()
            .filter(|value| value.is_finite())
            .collect();
        sorted.sort_by(f64::total_cmp);
        for target in [-f64::MAX, -1.0, 0.0, 1.0, f64::MAX] {
            let expected = candidates
                .iter()
                .copied()
                .any(|value| isclose(value, target));
            assert_eq!(
                sorted_contains_close(&sorted, target),
                expected,
                "membership drifted at {target}"
            );
        }
    }

    #[test]
    fn tick_range_respects_include_tip() {
        let line = NumberLine::new([-2.0, 2.0, 1.0]);
        assert_eq!(
            line.tick_range().expect("fixture ticks are bounded"),
            vec![-2.0, -1.0, 0.0, 1.0, 2.0]
        );
        let tipped = line.include_tip(true);
        assert_eq!(
            tipped.tick_range().expect("fixture ticks are bounded"),
            vec![-2.0, -1.0, 0.0, 1.0]
        );
    }

    #[test]
    fn number_line_sampling_budget_has_an_exact_boundary() {
        let exact = NumberLine::new([0.0, 4.0, 1.0])
            .sampling_budget(SamplingBudget::new(5))
            .tick_range()
            .expect("five tick values fit exactly");
        assert_eq!(exact, vec![0.0, 1.0, 2.0, 3.0, 4.0]);

        let error = NumberLine::new([0.0, 4.0, 1.0])
            .sampling_budget(SamplingBudget::new(4))
            .tick_range()
            .expect_err("five values exceed a four-sample budget");
        assert!(matches!(
            error,
            SamplingError::LimitExceeded {
                context: "number-line ticks",
                max_samples: 4
            }
        ));
    }

    #[test]
    fn number_line_refuses_hostile_ranges_before_building_ticks() {
        let tiny_step = NumberLine::new([0.0, 1.0, f64::MIN_POSITIVE])
            .sampling_budget(SamplingBudget::new(32))
            .build()
            .expect_err("tiny finite steps must be bounded");
        assert!(matches!(
            tiny_step,
            SamplingError::LimitExceeded {
                max_samples: 32,
                ..
            }
        ));

        let non_finite = NumberLine::new([0.0, f64::INFINITY, 1.0])
            .tick_range()
            .expect_err("non-finite endpoints must be refused");
        assert!(matches!(
            non_finite,
            SamplingError::NonFinite {
                parameter: "stop",
                ..
            }
        ));
    }

    // --- NumberLine labels (glyph sequences) ------------------------------

    fn num_strings(line: &NumberLine) -> Vec<String> {
        line.numbers()
            .iter()
            .map(|d| d.num_string().to_owned())
            .collect()
    }

    #[test]
    fn add_numbers_glyph_sequence_excluding_zero() {
        let mut line = NumberLine::new([-3.0, 3.0, 1.0])
            .build()
            .expect("fixture ticks are bounded");
        line.add_numbers(&book(), None, Some(&[0.0]))
            .expect("typeset labels");
        // The Reference's minus glyph is U+2013 EN DASH.
        assert_eq!(
            num_strings(&line),
            vec!["\u{2013}3", "\u{2013}2", "\u{2013}1", "1", "2", "3"]
        );
        // Labels sit below the line, one per kept tick, in order.
        let group = line.vmob().children().last().expect("numbers group");
        assert_eq!(group.children().len(), 6);
        let kept = [-3.0, -2.0, -1.0, 1.0, 2.0, 3.0];
        for (child, &x) in group.children().iter().zip(&kept) {
            let anchor = line.n2p(x);
            let center = child.center_point();
            assert!(
                center[1] < anchor[1] - MED_SMALL_BUFF + 1e-9,
                "label at {x} is not below the line"
            );
            assert!(
                center[1] > anchor[1] - MED_SMALL_BUFF - 0.5,
                "label at {x} drifted too far below"
            );
            assert!(
                (center[0] - anchor[0]).abs() < 0.25,
                "label at {x}: center {} too far from tick {}",
                center[0],
                anchor[0]
            );
        }
    }

    #[test]
    fn add_numbers_glyph_sequences_across_ranges_and_scales() {
        // Fractional range at a large scale, one decimal place.
        let mut fine = NumberLine::new([0.0, 1.0, 0.25])
            .unit_size(6.0)
            .num_decimal_places(1)
            .build()
            .expect("fixture ticks are bounded");
        fine.add_numbers(&book(), None, None).expect("typeset");
        assert_eq!(num_strings(&fine), vec!["0.0", "0.2", "0.5", "0.8", "1.0"]);

        // Coarse range at a small scale, integer formatting.
        let mut coarse = NumberLine::new([-10.0, 10.0, 5.0])
            .unit_size(0.5)
            .build()
            .expect("fixture ticks are bounded");
        coarse.add_numbers(&book(), None, None).expect("typeset");
        assert_eq!(
            num_strings(&coarse),
            vec!["\u{2013}10", "\u{2013}5", "0", "5", "10"]
        );
        // Positioned at the scaled n2p.
        let group = coarse.vmob().children().last().expect("numbers group");
        for (child, &x) in group.children().iter().zip(&[-10.0, -5.0, 0.0, 5.0, 10.0]) {
            let anchor = coarse.n2p(x);
            assert!(
                (child.center_point()[0] - anchor[0]).abs() < 0.3,
                "coarse label at {x} misplaced"
            );
        }
    }

    #[test]
    fn number_mobject_with_unit_drops_the_lone_one() {
        let line = NumberLine::new([-3.0, 3.0, 1.0]);
        let book = book();
        // x == unit: "1i" becomes just "i".
        let pos = line
            .number_mobject_with_unit(1.0, 1.0, "i", &book)
            .expect("typeset");
        assert_eq!(pos.children().len(), 1, "the lone 1 must be dropped");
        // x == -unit: "–1i" becomes "–i" (dash re-seated beside the unit).
        let neg = line
            .number_mobject_with_unit(-1.0, 1.0, "i", &book)
            .expect("typeset");
        assert_eq!(neg.children().len(), 2, "expected dash + unit");
        // x != ±unit: untouched ("2i" → digit + unit).
        let plain = line
            .number_mobject_with_unit(2.0, 1.0, "i", &book)
            .expect("typeset");
        assert_eq!(plain.children().len(), 2, "expected digit + unit");
    }

    // --- Axes --------------------------------------------------------------

    #[test]
    fn default_axes_map_coordinates_to_themselves() {
        let axes = Axes::new().build(&book()).expect("build axes");
        assert_vec3_near(axes.c2p(&[3.0, 2.0]), [3.0, 2.0, 0.0], 1e-12, "c2p(3,2)");
        assert_vec3_near(axes.origin(), ORIGIN, 1e-12, "origin");
        assert_vec3_near(axes.c2p(&[-8.0, -4.0]), [-8.0, -4.0, 0.0], 1e-12, "corner");
        assert_eq!(axes.all_ranges(), vec![DEFAULT_X_RANGE, DEFAULT_Y_RANGE]);
    }

    #[test]
    fn c2p_p2c_round_trip_property() {
        let configs = [
            Axes::new(),
            Axes::new()
                .x_range([-3.0, 5.0, 1.0])
                .y_range([2.0, 9.0, 0.5]),
            Axes::new()
                .x_range([1.0, 4.0, 0.5])
                .y_range([-7.0, -2.0, 1.0])
                .unit_size(2.0),
            Axes::new().width(10.0).height(3.0),
        ];
        let mut rng = Lcg(0xBE_EF);
        for axes in &configs {
            let axes = axes.clone().build(&book()).expect("build axes");
            let [xr, yr] = axes.all_ranges()[..] else {
                continue;
            };
            for _ in 0..150 {
                let x = rng.range(xr[0], xr[1]);
                let y = rng.range(yr[0], yr[1]);
                let back = axes.p2c(axes.c2p(&[x, y]));
                assert!(
                    (back[0] - x).abs() < 1e-12 && (back[1] - y).abs() < 1e-12,
                    "ranges {xr:?}/{yr:?}: ({x}, {y}) round-tripped to {back:?}"
                );
            }
        }
    }

    #[test]
    fn y_axis_is_the_x_axis_rotated_ninety_degrees() {
        let axes = Axes::new().build(&book()).expect("build axes");
        // A rotated horizontal line is vertical, centred at the origin.
        assert_vec3_near(
            axes.y_axis().start_point(),
            [0.0, -4.0, 0.0],
            1e-9,
            "y start",
        );
        assert_vec3_near(axes.y_axis().end_point(), [0.0, 4.0, 0.0], 1e-9, "y end");
        assert_vec3_near(axes.y_axis().n2p(2.0), [0.0, 2.0, 0.0], 1e-9, "y n2p(2)");
        // Both axes agree on where 0 lives.
        assert_vec3_near(
            axes.x_axis().n2p(0.0),
            axes.y_axis().n2p(0.0),
            1e-12,
            "shared origin",
        );
        // The y-axis numbers direction default is LEFT (default_y_axis_config).
        assert_eq!(axes.y_axis().line_to_number_direction_value(), LEFT);
    }

    #[test]
    fn shifted_origins_keep_both_axes_in_agreement() {
        let axes = Axes::new()
            .x_range([-2.0, 6.0, 1.0])
            .y_range([1.0, 5.0, 1.0])
            .build(&book())
            .expect("build axes");
        // n2p(0) coincides across axes even off-centre…
        assert_vec3_near(
            axes.x_axis().n2p(0.0),
            axes.y_axis().n2p(0.0),
            1e-9,
            "origin match",
        );
        // …and c2p(0, 0) is exactly that shared point.
        assert_vec3_near(
            axes.c2p(&[0.0, 0.0]),
            axes.x_axis().n2p(0.0),
            1e-12,
            "c2p(0,0)",
        );
    }

    #[test]
    fn add_coordinate_labels_glyph_sequences() {
        let mut axes = Axes::new()
            .x_range([-2.0, 2.0, 1.0])
            .y_range([-1.0, 1.0, 1.0])
            .build(&book())
            .expect("build axes");
        axes.add_coordinate_labels(&book(), None, None, None, None)
            .expect("typeset labels");
        let xs: Vec<&str> = axes
            .x_axis()
            .numbers()
            .iter()
            .map(DecimalNumber::num_string)
            .collect();
        let ys: Vec<&str> = axes
            .y_axis()
            .numbers()
            .iter()
            .map(DecimalNumber::num_string)
            .collect();
        assert_eq!(
            xs,
            vec!["\u{2013}2", "\u{2013}1", "1", "2"],
            "0 is excluded"
        );
        assert_eq!(ys, vec!["\u{2013}1", "1"], "0 is excluded");
    }

    // --- lines to points ------------------------------------------------------

    #[test]
    fn v_and_h_lines_project_onto_the_axes() {
        let axes = Axes::new().build(&book()).expect("build axes");
        let point = [3.0, 2.0, 0.0];
        let v = axes.get_v_line(point).expect("valid vertical dash line");
        let h = axes.get_h_line(point).expect("valid horizontal dash line");
        // Each starts at the projection on its axis and runs to the point.
        // A DashedLine's own point run is empty (the dashes are children),
        // so measure the family extent; the last dash stops short of the
        // point, so only the near end is exact.
        let (v_min, v_max) = v.extent().expect("v line extent");
        let (h_min, h_max) = h.extent().expect("h line extent");
        assert!(
            (v_min[0] - 3.0).abs() < 1e-9 && (v_max[0] - 3.0).abs() < 1e-9,
            "v line is vertical at x=3"
        );
        assert!((v_min[1] - 0.0).abs() < 1e-9, "v line starts on the x-axis");
        assert!(v_max[1] > 1.8, "v line reaches the point");
        assert!(
            (h_min[1] - 2.0).abs() < 1e-9 && (h_max[1] - 2.0).abs() < 1e-9,
            "h line is horizontal at y=2"
        );
        assert!((h_min[0] - 0.0).abs() < 1e-9, "h line starts on the y-axis");
        assert!(h_max[0] > 2.8, "h line reaches the point");
        assert_eq!(v.style().stroke_color, GREY_A);
        assert!((v.style().stroke_width - 2.0).abs() < 1e-15);
    }

    // --- calculus ----------------------------------------------------------------

    #[test]
    fn riemann_left_on_identity() {
        let axes = Axes::new().build(&book()).expect("build axes");
        let rects = axes
            .get_riemann_rectangles(&|x| x, Some([-2.0, 2.0]), Some(1.0), "left")
            .expect("valid sample type");
        assert_eq!(rects.children().len(), 4);
        let expected_heights = [2.0, 1.0, 0.0, 1.0];
        let gradient = color_gradient(&[BLUE, GREEN], 4);
        for (i, rect) in rects.children().iter().enumerate() {
            let x0 = i as f64 - 2.0;
            assert!(
                (rect.length_over_dim(1) - expected_heights[i]).abs() < 1e-9,
                "rect {i}: height {} vs {}",
                rect.length_over_dim(1),
                expected_heights[i]
            );
            assert!(
                (rect.length_over_dim(0) - 1.0).abs() < 1e-12,
                "rect {i}: width {} vs 1",
                rect.length_over_dim(0)
            );
            // Positive rects sit on the axis; negative ones hang below.
            let anchor = axes.c2p(&[x0, 0.0]);
            if i >= 2 {
                assert_vec3_near(
                    rect.bbox_point(DL).expect("bbox"),
                    anchor,
                    1e-9,
                    "DL anchor",
                );
                // BN-20: a zero-height bin is nonnegative and keeps its
                // gradient colour; the tail is GREEN.
                assert_eq!(rect.style().fill_color, gradient[i], "gradient colour");
            } else {
                assert_vec3_near(
                    rect.bbox_point(UL).expect("bbox"),
                    anchor,
                    1e-9,
                    "UL anchor",
                );
                assert_eq!(rect.style().fill_color, RED, "negative rects are RED");
            }
            assert_eq!(rect.style().stroke_color, BLACK);
            assert!((rect.style().stroke_width - 1.0).abs() < 1e-15);
            assert!((rect.style().fill_opacity - 1.0).abs() < 1e-15);
        }
    }

    #[test]
    fn riemann_center_on_shifted_parabola_marks_negative_regions_red() {
        let axes = Axes::new().build(&book()).expect("build axes");
        let rects = axes
            .get_riemann_rectangles(&|x| x * x - 1.0, Some([0.0, 3.0]), Some(1.0), "center")
            .expect("valid sample type");
        assert_eq!(rects.children().len(), 3);
        // Centers 0.5, 1.5, 2.5 → f = -0.75, 1.25, 5.25.
        let expected_heights = [0.75, 1.25, 5.25];
        for (i, rect) in rects.children().iter().enumerate() {
            assert!(
                (rect.length_over_dim(1) - expected_heights[i]).abs() < 1e-9,
                "rect {i}: height {} vs {}",
                rect.length_over_dim(1),
                expected_heights[i]
            );
        }
        assert_eq!(rects.children()[0].style().fill_color, RED, "dipped region");
        assert_eq!(
            rects.children()[1].style().fill_color,
            color_gradient(&[BLUE, GREEN], 3)[1]
        );
        assert_eq!(rects.children()[2].style().fill_color, GREEN);
    }

    #[test]
    fn riemann_rejects_an_unknown_sample_type() {
        let axes = Axes::new().build(&book()).expect("build axes");
        let err = axes
            .get_riemann_rectangles(&|x| x, None, None, "middle")
            .expect_err("invalid sample type must fail");
        assert!(matches!(
            err,
            CoordsError::InvalidSampleType(ref sample) if sample == "middle"
        ));
    }

    #[test]
    fn riemann_defaults_use_the_axes_x_range_and_step() {
        let axes = Axes::new()
            .x_range([-1.0, 2.0, 0.5])
            .build(&book())
            .expect("build axes");
        let rects = axes
            .get_riemann_rectangles(&|_| 1.0, None, None, "right")
            .expect("valid sample type");
        // arange(-1, 2 + 0.5, 0.5) → 7 bounds → 6 rects of width 0.5.
        assert_eq!(rects.children().len(), 6);
        assert!((rects.children()[0].length_over_dim(0) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn axis_stroke_opacity_reaches_the_line_and_its_ticks() {
        // The Reference passes axis_config style keywords to NumberLine
        // (a Line), whose ticks match its style.
        let axes = Axes::new()
            .axis_config(AxisConfig {
                stroke_opacity: Some(0.25),
                ..AxisConfig::default()
            })
            .build(&book())
            .expect("build axes");
        for axis in [axes.x_axis(), axes.y_axis()] {
            let line = axis.vmob();
            assert!((line.style().stroke_opacity - 0.25).abs() < 1e-15);
            let ticks = line.children().last().expect("ticks group");
            assert!(!ticks.children().is_empty());
            for tick in ticks.children() {
                assert!((tick.style().stroke_opacity - 0.25).abs() < 1e-15);
            }
        }
        let default = Axes::new().build(&book()).expect("build axes");
        assert!((default.x_axis().vmob().style().stroke_opacity - 1.0).abs() < 1e-15);
    }

    #[test]
    fn riemann_bins_follow_bn20() {
        let axes = Axes::new().build(&book()).expect("build axes");
        let f = |x: f64| x.sin() + 1.0;
        // The interval is authoritative: 12 full bins, then [3, π].
        let rects = axes
            .get_riemann_rectangles(&f, Some([0.0, PI]), Some(0.25), "left")
            .expect("bins");
        assert_eq!(rects.children().len(), 13);
        let last = &rects.children()[12];
        assert_vec3_near(
            last.bbox_point(DR).expect("last bin"),
            axes.c2p(&[PI, 0.0]),
            1e-9,
            "the final partial bin ends at the bound",
        );
        assert!((last.length_over_dim(0) - (PI - 3.0)).abs() < 1e-9);
        // A range that is a multiple of dx only up to rounding grows no
        // sliver bin: 0.3 / 0.1 is 2.9999999999999996.
        let exact = axes
            .get_riemann_rectangles(&f, Some([0.0, 0.3]), Some(0.1), "right")
            .expect("bins");
        assert_eq!(exact.children().len(), 3);
        // A zero-length interval is empty; refusals are typed.
        let empty = axes
            .get_riemann_rectangles(&f, Some([1.0, 1.0]), Some(0.5), "left")
            .expect("empty");
        assert!(empty.children().is_empty());
        assert!(matches!(
            axes.get_riemann_rectangles(&f, Some([1.0, 0.0]), Some(0.5), "left"),
            Err(CoordsError::InvalidRiemannRange(_))
        ));
        assert!(matches!(
            axes.get_riemann_rectangles(&f, Some([0.0, 1.0]), Some(0.0), "left"),
            Err(CoordsError::InvalidRiemannRange(_))
        ));
        assert!(matches!(
            axes.get_riemann_rectangles(&f, Some([0.0, f64::NAN]), Some(0.5), "left"),
            Err(CoordsError::Sampling(SamplingError::NonFinite { .. }))
        ));
        // show_signed_area=false keeps the gradient on negative bins.
        let config = RiemannConfig {
            show_signed_area: false,
            ..RiemannConfig::default()
        };
        let unsigned = axes
            .riemann_rectangles_styled(&|x| x, Some([-2.0, 2.0]), Some(1.0), "left", &config)
            .expect("bins");
        let gradient = color_gradient(&[BLUE, GREEN], 4);
        for (rect, colour) in unsigned.children().iter().zip(&gradient) {
            assert_eq!(rect.style().fill_color, *colour);
        }
    }

    #[test]
    fn riemann_sampling_is_preflighted_before_function_evaluation() {
        let axes = Axes::new()
            .x_range([0.0, 1.0, 0.25])
            .axis_config(AxisConfig {
                include_ticks: Some(false),
                ..AxisConfig::default()
            })
            .sampling_budget(SamplingBudget::new(4))
            .build(&book())
            .expect("tick-free axes do not sample during build");
        let calls = Cell::new(0);
        let function = |x| {
            calls.set(calls.get() + 1);
            x
        };
        let error = axes
            .get_riemann_rectangles(&function, None, None, "left")
            .expect_err("five bounds exceed a four-sample budget");
        assert!(matches!(
            error,
            CoordsError::Sampling(SamplingError::LimitExceeded {
                context: "Riemann rectangles",
                max_samples: 4
            })
        ));
        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn area_under_graph_closes_through_the_axis() {
        let axes = Axes::new().build(&book()).expect("build axes");
        let graph = axes
            .get_graph(|x| x * x)
            .use_smoothing(false)
            .build()
            .expect("default graph sampling is bounded");
        let area = axes.get_area_under_graph(&graph, [-8.0, 8.0], Some([-1.0, 2.0]));
        let points = area.points();
        assert!(points.len() > 6, "a partial curve plus the closing lines");
        // The Reference appends c2p(x1, 0), c2p(x0, 0), and the start.
        let first = points[0];
        let last = *points.last().expect("non-empty");
        assert_vec3_near(last, first, 1e-9, "closed back to start");
        assert_vec3_near(first, [-1.0, 1.0, 0.0], 1e-6, "starts on the curve");
        let has = |target: Vec3| {
            points
                .iter()
                .any(|p| (p[0] - target[0]).abs() < 1e-9 && (p[1] - target[1]).abs() < 1e-9)
        };
        assert!(has([2.0, 0.0, 0.0]), "passes through c2p(2, 0)");
        assert!(has([-1.0, 0.0, 0.0]), "passes through c2p(-1, 0)");
        assert!((area.style().stroke_width - 0.0).abs() < 1e-15);
        assert!((area.style().fill_opacity - 0.5).abs() < 1e-15);
        assert_eq!(area.style().fill_color, BLUE);
    }

    #[test]
    fn tangent_angle_slope_and_line_on_known_curves() {
        let axes = Axes::new().build(&book()).expect("build axes");
        // f(x) = x: 45° on unit axes, slope 1.
        let angle = axes.angle_of_tangent(2.0, &|x| x);
        assert!((angle - PI / 4.0).abs() < 1e-6, "angle {angle} vs π/4");
        assert!((axes.slope_of_tangent(2.0, &|x| x) - 1.0).abs() < 1e-6);
        // f(x) = x² at x = 0: horizontal tangent.
        assert!(axes.angle_of_tangent(0.0, &|x| x * x).abs() < 1e-6);
        // f(x) = x² at x = 1: slope 2.
        assert!((axes.slope_of_tangent(1.0, &|x| x * x) - 2.0).abs() < 1e-5);
        // The tangent line: length 5, centred on the graph point.
        let line = axes.get_tangent_line(2.0, &|x| x, 5.0);
        let points = line.points();
        let (first, last) = (points[0], points[points.len() - 1]);
        let span = space_ops::get_dist(first, last);
        assert!((span - 5.0).abs() < 1e-9, "tangent span {span} vs 5");
        assert_vec3_near(line.center_point(), [2.0, 2.0, 0.0], 1e-9, "tangent centre");
    }

    // --- UnitInterval + Slider -----------------------------------------------------

    #[test]
    fn unit_interval_preset() {
        let line = UnitInterval::new();
        assert_vec3_near(line.n2p(0.0), [-5.0, 0.0, 0.0], 1e-12, "n2p(0)");
        assert_vec3_near(line.n2p(0.5), ORIGIN, 1e-12, "n2p(0.5)");
        assert_vec3_near(line.n2p(1.0), [5.0, 0.0, 0.0], 1e-12, "n2p(1)");
        assert_eq!(
            line.tick_range()
                .expect("unit-interval ticks are bounded")
                .len(),
            11
        );
        let built = line.build().expect("unit-interval ticks are bounded");
        let ticks = &built.vmob().children()[0];
        assert_eq!(ticks.children().len(), 11);
        // Big ticks at 0 and 1 (the ends): 1.5× taller.
        for (i, tick) in ticks.children().iter().enumerate() {
            let expected = if i == 0 || i == 10 { 0.3 } else { 0.2 };
            assert!(
                (tick.length_over_dim(1) - expected).abs() < 1e-12,
                "tick {i}: height {} vs {expected}",
                tick.length_over_dim(1)
            );
        }
    }

    #[test]
    fn slider_assembles_line_tip_and_label() {
        let slider = Slider::new(0.5).var_name("x");
        let vmob = slider.build(&book()).expect("build slider");
        assert_eq!(vmob.children().len(), 3, "[number_line, tip, label]");
        // The line: x_range (-5, 5) at width 3 → unit 0.3; n2p(0.5) = [0.15, 0, 0].
        // (The family bbox is wider — tick labels stick out past the ends —
        // so measure the base line's own points.)
        let line = &vmob.children()[0];
        let (first, last) = (
            line.points().first().copied().expect("line start"),
            line.points().last().copied().expect("line end"),
        );
        assert!((last[0] - first[0] - 3.0).abs() < 1e-9, "line width");
        // The tip's bottom edge (label_direction = UP → aligned DOWN) sits
        // on the line at the value.
        let tip = &vmob.children()[1];
        let bottom = tip.bbox_point(DOWN).expect("tip bbox");
        assert_vec3_near(bottom, [0.15, 0.0, 0.0], 1e-9, "tip seat");
        // The label exists above the tip with glyph children.
        let label = &vmob.children()[2];
        assert!(!label.children().is_empty(), "label has content");
        assert!(
            label.center_point()[1] > tip.center_point()[1],
            "label above tip"
        );
    }

    #[test]
    fn slider_tick_labels_use_the_small_font() {
        let slider = Slider::new(-2.0).add_tick_labels(true);
        let vmob = slider.build(&book()).expect("build slider");
        let line = &vmob.children()[0];
        // [ticks group, numbers group] under the base line.
        assert_eq!(line.children().len(), 2, "ticks + numbers");
        let numbers = &line.children()[1];
        assert_eq!(numbers.children().len(), 11, "every integer of [-5, 5]");
    }
}
