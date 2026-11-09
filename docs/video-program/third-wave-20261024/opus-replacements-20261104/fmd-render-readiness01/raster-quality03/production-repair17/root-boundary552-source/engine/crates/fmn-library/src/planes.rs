//! The plane family (§12.2): [`ThreeDAxes`], [`NumberPlane`], and
//! [`ComplexPlane`] — the coordinate systems built on the `coords.rs`
//! axis machinery.
//!
//! Ported from `manimlib/mobject/coordinate_systems.py` @ `6199a00d`
//! (`ThreeDAxes`, `NumberPlane`, `ComplexPlane`). The axes themselves are
//! [`crate::coords::NumberLine`]s and the 2D scaffolding is
//! [`crate::coords::Axes`]; this module adds the third axis, the
//! background/faded line families, and complex-number labelling.
//!
//! Reference-fidelity notes:
//!
//! * **`ThreeDAxes` ranges are its own.** The Reference's `ThreeDAxes`
//!   defaults are `x_range=(-6, 6, 1)`, `y_range=(-5, 5, 1)`,
//!   `z_range=(-4, 4, 1)` — *not* the `Axes` defaults. The z-axis is the
//!   x-axis construction rotated `-π/2` about `UP` through the origin (so
//!   `+z` points `OUT` of the screen) and then by
//!   `angle_of_vector(z_normal)` about `OUT` (default `z_normal = DOWN`,
//!   which reorients the tick marks but cannot move the axis itself).
//! * **The z-axis ignores the top-level `unit_size`.** The Reference
//!   injects `unit_size` into a *local copy* of `axis_config` inside
//!   `Axes.__init__`; the z-axis merge reads the caller's original dict.
//!   `ThreeDAxes::unit_size(2.0)` therefore scales x and y but leaves the
//!   z unit at 1.0 (use [`AxisConfig::unit_size`] inside
//!   `z_axis_config` to scale it). This is the Reference's exact
//!   behaviour, kept deliberately.
//! * **`NumberPlane`'s line families use numpy `arange` semantics.** The
//!   positions are `x_min + i·(x_step/(1+ratio))` for every `i` below
//!   `ceil((x_max + step - x_min)/step)` — the overshoot case included, so
//!   with `x_range=(-2, 2, 1)` and `faded_line_ratio=2` the last faded
//!   line sits at `7/3 > x_max`, exactly as the Reference draws it. A
//!   position whose absolute value is below `1e-8` is skipped (the axis
//!   itself lives there). Every `(1+ratio)`-th line by *index* is a
//!   background line; the rest are faded.
//! * **`faded_line_style.stroke_color` inherits.** When unset it falls
//!   back to the background stroke color (the Reference's
//!   `init_background_lines` defaulting).
//! * **`ComplexPlane` labels are axis number mobjects**, one per default
//!   coordinate value — the Reference does *not* typeset combined `a+bi`
//!   strings. Each complex value `z` becomes a real-axis
//!   [`DecimalNumber`][crate::numbers::DecimalNumber] of `z.re` when
//!   `|z.im| <= |z.re|`, otherwise an
//!   imaginary-axis `DecimalNumber` of `z.im` carrying the unit `i`
//!   (with the Reference's `±1 → ±i` digit-drop inside
//!   `NumberLine::number_mobject_with_unit`).
//!
//! Two small, documented divergences:
//!
//! * **`y_unit_size` is correct.** The Reference's
//!   `NumberPlane.get_y_unit_size` returns the *x*-axis unit size (an
//!   upstream bug, invisible whenever the unit sizes agree). Ours
//!   returns the y-axis unit size.
//! * **`prepare_for_nonlinear_transform` is not ported.** It exists to
//!   feed the Reference's `apply_function` animation; nonlinear
//!   transforms are the animation tier's (W9), and a built [`VMobject`]
//!   here is plain detached geometry.
//!
//! Built charts support owning placement and native graph/surface construction.
//! Their coordinate maps move with every axis, grid line, and attached label.

mod placement;

use fmn_core::color::Srgb;
use fmn_core::constants::{
    BLUE_D, DEFAULT_MOBJECT_COLOR, DL, DOWN, ORIGIN, OUT, PI, SMALL_BUFF, UP,
};
use fmn_core::types::Vec3;
use fmn_geom::space_ops::angle_of_vector;
use fmn_text::FontBook;

use crate::coords::{Axes, AxisConfig, CoordinateSystem, CoordsError, NumberLine, create_axis};
use crate::graphs::{
    ParametricCurve, SamplingBudget, SamplingError, graph_parametric, sampled_values,
};
use crate::line::{Arrow, Line};
use crate::vmobject::{VMobject, v_group};

/// The Reference's `ThreeDAxes(x_range=(-6.0, 6.0, 1.0))`.
pub const THREE_D_X_RANGE: [f64; 3] = [-6.0, 6.0, 1.0];
/// The Reference's `ThreeDAxes(y_range=(-5.0, 5.0, 1.0))`.
pub const THREE_D_Y_RANGE: [f64; 3] = [-5.0, 5.0, 1.0];
/// The Reference's `ThreeDAxes(z_range=(-4.0, 4.0, 1.0))`.
pub const THREE_D_Z_RANGE: [f64; 3] = [-4.0, 4.0, 1.0];

/// The Reference's `NumberPlane(x_range=(-8.0, 8.0, 1.0))`.
pub const PLANE_X_RANGE: [f64; 3] = [-8.0, 8.0, 1.0];
/// The Reference's `NumberPlane(y_range=(-4.0, 4.0, 1.0))`.
pub const PLANE_Y_RANGE: [f64; 3] = [-4.0, 4.0, 1.0];

/// The Reference's `NumberPlane(faded_line_ratio=4)`.
pub const DEFAULT_FADED_LINE_RATIO: usize = 4;

/// Positions with `|x| < EPSILON` get no background line: the axis
/// itself is drawn there (the Reference's `abs(x) < 1e-8` skip).
const SKIP_EPSILON: f64 = 1e-8;

/// The stroke triple of a `NumberPlane` line family.
///
/// The Reference passes these as style dicts (`background_line_style =
/// dict(stroke_color=BLUE_D, stroke_width=2, stroke_opacity=1)`); only
/// the stroke is ever set, the fill stays the `Line`'s own.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineFamilyStyle {
    /// `stroke_color`.
    pub stroke_color: Srgb,
    /// `stroke_width`.
    pub stroke_width: f64,
    /// `stroke_opacity`.
    pub stroke_opacity: f64,
}

impl Default for LineFamilyStyle {
    /// The Reference's `background_line_style` default.
    fn default() -> Self {
        Self {
            stroke_color: BLUE_D,
            stroke_width: 2.0,
            stroke_opacity: 1.0,
        }
    }
}

/// The faded family's style: the Reference's `faded_line_style =
/// dict(stroke_width=1, stroke_opacity=0.25)` plus an optional color
/// override. `None` inherits the background stroke color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FadedLineStyle {
    /// `stroke_color` — `None` inherits `background_line_style`'s.
    pub stroke_color: Option<Srgb>,
    /// `stroke_width`.
    pub stroke_width: f64,
    /// `stroke_opacity`.
    pub stroke_opacity: f64,
}

impl Default for FadedLineStyle {
    /// The Reference's `faded_line_style` default.
    fn default() -> Self {
        Self {
            stroke_color: None,
            stroke_width: 1.0,
            stroke_opacity: 0.25,
        }
    }
}

/// The Reference's `NumberPlane.default_axis_config`: white 2-width
/// stroke, no ticks, no tip, number labels `DL` at `SMALL_BUFF`. Only the
/// stroke is white; the fill keeps `NumberLine`'s `DEFAULT_LIGHT_COLOR`
/// (fm-78b6).
fn number_plane_axis_config() -> AxisConfig {
    AxisConfig {
        stroke_color: Some(DEFAULT_MOBJECT_COLOR),
        stroke_width: Some(2.0),
        include_ticks: Some(false),
        include_tip: Some(false),
        line_to_number_buff: Some(SMALL_BUFF),
        line_to_number_direction: Some(DL),
        ..AxisConfig::default()
    }
}

/// The Reference's `NumberPlane.default_y_axis_config`
/// (`line_to_number_direction=DL` — overriding the `Axes` default of
/// `LEFT`).
fn number_plane_y_axis_config() -> AxisConfig {
    AxisConfig {
        line_to_number_direction: Some(DL),
        ..AxisConfig::default()
    }
}

/// numpy `arange(start, stop, step)` — `start + i·step` for
/// `i < ceil((stop - start)/step)`, the multiplication form, so the
/// floating-point sequence (including endpoint overshoot) matches the
/// Reference bit for bit.
///
/// A finite non-positive `step` yields nothing; numpy raises there, and
/// an unconditional index loop would hang (the same divergence
/// `graphs.rs` documents for its sampler). Non-finite controls are typed
/// [`SamplingError`]s.
fn arange(
    start: f64,
    stop: f64,
    step: f64,
    budget: SamplingBudget,
) -> Result<Vec<f64>, SamplingError> {
    sampled_values("number-plane lines", start, stop, step, false, budget)
}

/// The Reference's `get_lines_parallel_to_axis`: copies of the line
/// spanning `axis1`, shifted to every dense tick position along `axis2`,
/// split into background (every `(1 + ratio)`-th by index) and faded.
fn lines_parallel_to_axis(
    axis1: &NumberLine,
    axis2: &NumberLine,
    faded_line_ratio: usize,
    sampling_budget: SamplingBudget,
) -> Result<(Vec<VMobject>, Vec<VMobject>), SamplingError> {
    let dense_freq = faded_line_ratio
        .checked_add(1)
        .ok_or(SamplingError::CapacityOverflow {
            context: "number-plane faded-line ratio",
        })?;
    let step = axis2.x_step() / dense_freq as f64;
    let template = Line::new(axis1.start_point(), axis1.end_point())
        .build()
        .expect("number-plane grid templates are straight segments");
    let origin = axis2.n2p(0.0);
    let mut background = Vec::new();
    let mut faded = Vec::new();
    for (i, x) in arange(axis2.x_min(), axis2.x_max() + step, step, sampling_budget)?
        .into_iter()
        .enumerate()
    {
        if x.abs() < SKIP_EPSILON {
            continue;
        }
        let target = axis2.n2p(x);
        let line = template.clone().shifted([
            target[0] - origin[0],
            target[1] - origin[1],
            target[2] - origin[2],
        ]);
        if i % dense_freq == 0 {
            background.push(line);
        } else {
            faded.push(line);
        }
    }
    Ok((background, faded))
}

/// `ThreeDAxes` — three [`NumberLine`] axes with `+z` pointing `OUT` of
/// the screen (§12.2, Reference `coordinate_systems.py:ThreeDAxes`).
///
/// The builder holds the constructor configuration; [`ThreeDAxes::build`]
/// assembles the axes (x/y through [`Axes`], z through
/// [`create_axis`] plus the two Reference rotations) and the flat
/// `[x_axis, y_axis, z_axis]` mobject family. [`CoordinateSystem`]
/// methods read the built axes, so call `build` first.
#[derive(Debug, Clone)]
pub struct ThreeDAxes {
    x_range: [f64; 3],
    y_range: [f64; 3],
    z_range: [f64; 3],
    axis_config: AxisConfig,
    x_axis_config: AxisConfig,
    y_axis_config: AxisConfig,
    z_axis_config: AxisConfig,
    z_normal: Vec3,
    height: Option<f64>,
    width: Option<f64>,
    depth: Option<f64>,
    unit_size: f64,
    num_sampled_graph_points_per_tick: f64,
    sampling_budget: SamplingBudget,
    built: Option<ThreeDBuilt>,
}

/// The built state: the composed 2D axes, the rotated z-axis, and the
/// assembled family mobject.
#[derive(Debug, Clone)]
struct ThreeDBuilt {
    axes: Axes,
    z_axis: NumberLine,
    vmob: VMobject,
}

impl Default for ThreeDAxes {
    fn default() -> Self {
        Self::new()
    }
}

impl ThreeDAxes {
    /// The Reference defaults: `x_range=(-6, 6, 1)`,
    /// `y_range=(-5, 5, 1)`, `z_range=(-4, 4, 1)`, `z_normal=DOWN`,
    /// `unit_size=1`, no explicit height/width/depth.
    #[must_use]
    pub fn new() -> Self {
        Self {
            x_range: THREE_D_X_RANGE,
            y_range: THREE_D_Y_RANGE,
            z_range: THREE_D_Z_RANGE,
            axis_config: AxisConfig::default(),
            x_axis_config: AxisConfig::default(),
            y_axis_config: AxisConfig::default(),
            z_axis_config: AxisConfig::default(),
            z_normal: DOWN,
            height: None,
            width: None,
            depth: None,
            unit_size: 1.0,
            num_sampled_graph_points_per_tick: 5.0,
            sampling_budget: SamplingBudget::default(),
            built: None,
        }
    }

    /// `x_range=`.
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

    /// `z_range=`.
    #[must_use]
    pub fn z_range(mut self, range: [f64; 3]) -> Self {
        self.z_range = range;
        self
    }

    /// `axis_config=` — applied to all three axes (under each axis's own
    /// config, the Reference's `merge_dicts_recursively` order). Note the
    /// z-axis never sees the top-level `unit_size` (module docs).
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

    /// `z_axis_config=`.
    #[must_use]
    pub fn z_axis_config(mut self, config: AxisConfig) -> Self {
        self.z_axis_config = config;
        self
    }

    /// `z_normal=` — the direction the z-axis's tick side faces after
    /// the second rotation. Does not move the axis itself.
    #[must_use]
    pub fn z_normal(mut self, normal: Vec3) -> Self {
        self.z_normal = normal;
        self
    }

    /// `height=` (the y-axis length).
    #[must_use]
    pub fn height(mut self, height: f64) -> Self {
        self.height = Some(height);
        self
    }

    /// `width=` (the x-axis length).
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.width = Some(width);
        self
    }

    /// `depth=` (the z-axis length).
    #[must_use]
    pub fn depth(mut self, depth: f64) -> Self {
        self.depth = Some(depth);
        self
    }

    /// `unit_size=` — scales the x and y axes only (module docs).
    #[must_use]
    pub fn unit_size(mut self, unit_size: f64) -> Self {
        self.unit_size = unit_size;
        self
    }

    /// `num_sampled_graph_points_per_tick=`.
    #[must_use]
    pub fn num_sampled_graph_points_per_tick(mut self, n: f64) -> Self {
        self.num_sampled_graph_points_per_tick = n;
        self
    }

    /// Bound tick and graph sampling across all three axes.
    #[must_use]
    pub fn sampling_budget(mut self, budget: SamplingBudget) -> Self {
        self.sampling_budget = budget;
        self
    }

    /// Assemble the axes. The x/y pair is built by [`Axes`]; the z-axis
    /// is [`create_axis`] output rotated `-π/2` about `UP` through the
    /// origin (putting `+z` along `OUT`) and then by
    /// `angle_of_vector(z_normal)` about `OUT`, and finally shifted onto
    /// the shared origin — the Reference's `__init__` verbatim.
    pub fn build(mut self, book: &FontBook) -> Result<Self, CoordsError> {
        let mut axes_builder = Axes::new()
            .x_range(self.x_range)
            .y_range(self.y_range)
            .axis_config(self.axis_config.clone())
            .x_axis_config(self.x_axis_config.clone())
            .y_axis_config(self.y_axis_config.clone())
            .unit_size(self.unit_size)
            .sampling_budget(self.sampling_budget);
        if let Some(height) = self.height {
            axes_builder = axes_builder.height(height);
        }
        if let Some(width) = self.width {
            axes_builder = axes_builder.width(width);
        }
        let axes = axes_builder.build(book)?;

        // merge_dicts_recursively(default_axis_config,
        //                         default_z_axis_config,
        //                         axis_config, z_axis_config) — both
        // ThreeDAxes defaults are empty, so this is the caller's
        // axis_config under z_axis_config. The top-level unit_size is
        // deliberately absent (module docs).
        let z_config = self.axis_config.clone().merge(self.z_axis_config.clone());
        let z_axis = create_axis(self.z_range, z_config, self.depth)
            .rotated_about(-PI / 2.0, UP, ORIGIN)
            .rotated_about(angle_of_vector(self.z_normal), OUT, ORIGIN)
            .shifted(axes.x_axis().n2p(0.0))
            .sampling_budget(self.sampling_budget)
            .build()?;

        // The Reference's self.add(*self.axes) then self.add(z_axis):
        // a flat [x_axis, y_axis, z_axis] family.
        let vmob = axes.vmob().clone().with_child(z_axis.vmob().clone());
        self.built = Some(ThreeDBuilt { axes, z_axis, vmob });
        Ok(self)
    }

    fn built(&self) -> &ThreeDBuilt {
        self.built
            .as_ref()
            .expect("ThreeDAxes::build must run before geometry access")
    }

    /// The x-axis (Reference `get_x_axis`).
    #[must_use]
    pub fn x_axis(&self) -> &NumberLine {
        self.built().axes.x_axis()
    }

    /// The y-axis (Reference `get_y_axis`).
    #[must_use]
    pub fn y_axis(&self) -> &NumberLine {
        self.built().axes.y_axis()
    }

    /// The z-axis (Reference `get_z_axis`).
    #[must_use]
    pub fn z_axis(&self) -> &NumberLine {
        &self.built().z_axis
    }

    /// The assembled `[x_axis, y_axis, z_axis]` family.
    #[must_use]
    pub fn vmob(&self) -> &VMobject {
        &self.built().vmob
    }

    /// Consume into the assembled family.
    #[must_use]
    pub fn into_vmob(self) -> VMobject {
        self.built
            .expect("ThreeDAxes::build must run before into_vmob")
            .vmob
    }
}

impl CoordinateSystem for ThreeDAxes {
    fn c2p(&self, coords: &[f64]) -> Vec3 {
        let built = self.built();
        let origin = built.axes.x_axis().n2p(0.0);
        let axes = [built.axes.x_axis(), built.axes.y_axis(), &built.z_axis];
        let mut point = origin;
        for (axis, &coord) in axes.iter().zip(coords.iter()) {
            let q = axis.n2p(coord);
            point = [
                point[0] + q[0] - origin[0],
                point[1] + q[1] - origin[1],
                point[2] + q[2] - origin[2],
            ];
        }
        point
    }

    fn p2c(&self, point: Vec3) -> [f64; 3] {
        let built = self.built();
        [
            built.axes.x_axis().p2n(point),
            built.axes.y_axis().p2n(point),
            built.z_axis.p2n(point),
        ]
    }

    fn all_ranges(&self) -> Vec<[f64; 3]> {
        vec![self.x_range, self.y_range, self.z_range]
    }

    fn num_sampled_graph_points_per_tick(&self) -> f64 {
        self.num_sampled_graph_points_per_tick
    }

    fn graph_sampling_budget(&self) -> SamplingBudget {
        self.sampling_budget
    }

    fn dimension(&self) -> usize {
        3
    }
}

impl ThreeDAxes {
    pub(crate) fn coordinate_axes(&self) -> Option<[&NumberLine; 3]> {
        let built = self.built.as_ref()?;
        Some([built.axes.x_axis(), built.axes.y_axis(), &built.z_axis])
    }
}

/// `NumberPlane` — a 2D [`Axes`] with a full grid of background lines
/// behind it (§12.2, Reference `coordinate_systems.py:NumberPlane`).
///
/// The built family is `[faded_lines, background_lines, x_axis,
/// y_axis]` — the Reference's `add_to_back(faded_lines,
/// background_lines)` order. Each line family is itself a `v_group` of
/// individual [`Line`] mobjects, so structural fixtures can count and
/// locate every line.
#[derive(Debug, Clone)]
pub struct NumberPlane {
    x_range: [f64; 3],
    y_range: [f64; 3],
    axis_config: AxisConfig,
    x_axis_config: AxisConfig,
    y_axis_config: AxisConfig,
    height: Option<f64>,
    width: Option<f64>,
    unit_size: f64,
    background_line_style: LineFamilyStyle,
    faded_line_style: FadedLineStyle,
    faded_line_ratio: usize,
    make_smooth_after_applying_functions: bool,
    num_sampled_graph_points_per_tick: f64,
    sampling_budget: SamplingBudget,
    built: Option<PlaneBuilt>,
}

/// The built state: axes, both line families, and the assembled mobject.
#[derive(Debug, Clone)]
struct PlaneBuilt {
    axes: Axes,
    background_lines: VMobject,
    faded_lines: VMobject,
    vmob: VMobject,
}

impl Default for NumberPlane {
    fn default() -> Self {
        Self::new()
    }
}

impl NumberPlane {
    /// The Reference defaults: `x_range=(-8, 8, 1)`,
    /// `y_range=(-4, 4, 1)`, BLUE_D background lines at width 2, faded
    /// lines at width 1 / opacity 0.25, `faded_line_ratio=4`,
    /// `unit_size=1`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            x_range: PLANE_X_RANGE,
            y_range: PLANE_Y_RANGE,
            axis_config: AxisConfig::default(),
            x_axis_config: AxisConfig::default(),
            y_axis_config: AxisConfig::default(),
            height: None,
            width: None,
            unit_size: 1.0,
            background_line_style: LineFamilyStyle::default(),
            faded_line_style: FadedLineStyle::default(),
            faded_line_ratio: DEFAULT_FADED_LINE_RATIO,
            make_smooth_after_applying_functions: true,
            num_sampled_graph_points_per_tick: 5.0,
            sampling_budget: SamplingBudget::default(),
            built: None,
        }
    }

    /// `x_range=`.
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

    /// `axis_config=` — merged *over* the `NumberPlane` axis defaults
    /// (module docs), under `x/y_axis_config`.
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

    /// `height=` (the y-axis length).
    #[must_use]
    pub fn height(mut self, height: f64) -> Self {
        self.height = Some(height);
        self
    }

    /// `width=` (the x-axis length).
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.width = Some(width);
        self
    }

    /// `unit_size=`.
    #[must_use]
    pub fn unit_size(mut self, unit_size: f64) -> Self {
        self.unit_size = unit_size;
        self
    }

    /// `background_line_style=`.
    #[must_use]
    pub fn background_line_style(mut self, style: LineFamilyStyle) -> Self {
        self.background_line_style = style;
        self
    }

    /// `faded_line_style=`.
    #[must_use]
    pub fn faded_line_style(mut self, style: FadedLineStyle) -> Self {
        self.faded_line_style = style;
        self
    }

    /// `faded_line_ratio=` — every `(1 + ratio)`-th line by index is a
    /// background line; the rest are faded.
    #[must_use]
    pub fn faded_line_ratio(mut self, ratio: usize) -> Self {
        self.faded_line_ratio = ratio;
        self
    }

    /// `make_smooth_after_applying_functions=` — carried for the
    /// animation tier; no effect on the built geometry.
    #[must_use]
    pub fn make_smooth_after_applying_functions(mut self, on: bool) -> Self {
        self.make_smooth_after_applying_functions = on;
        self
    }

    /// `num_sampled_graph_points_per_tick=`.
    #[must_use]
    pub fn num_sampled_graph_points_per_tick(mut self, n: f64) -> Self {
        self.num_sampled_graph_points_per_tick = n;
        self
    }

    /// Bound axis ticks, grid lines, rectangles, and graph samples.
    #[must_use]
    pub fn sampling_budget(mut self, budget: SamplingBudget) -> Self {
        self.sampling_budget = budget;
        self
    }

    /// Assemble the axes and both line families (the Reference's
    /// `__init__` + `init_background_lines`).
    pub fn build(mut self, book: &FontBook) -> Result<Self, CoordsError> {
        // NumberPlane's default_axis_config sits below the caller's
        // axis_config; its default_y_axis_config (DL) below the
        // caller's y_axis_config — the Reference's merge order.
        let axis_config = number_plane_axis_config().merge(self.axis_config.clone());
        let y_axis_config = number_plane_y_axis_config().merge(self.y_axis_config.clone());
        let mut axes_builder = Axes::new()
            .x_range(self.x_range)
            .y_range(self.y_range)
            .axis_config(axis_config)
            .x_axis_config(self.x_axis_config.clone())
            .y_axis_config(y_axis_config)
            .unit_size(self.unit_size)
            .sampling_budget(self.sampling_budget);
        if let Some(height) = self.height {
            axes_builder = axes_builder.height(height);
        }
        if let Some(width) = self.width {
            axes_builder = axes_builder.width(width);
        }
        let axes = axes_builder.build(book)?;

        let (x_bg, x_faded) = lines_parallel_to_axis(
            axes.x_axis(),
            axes.y_axis(),
            self.faded_line_ratio,
            self.sampling_budget,
        )?;
        let (y_bg, y_faded) = lines_parallel_to_axis(
            axes.y_axis(),
            axes.x_axis(),
            self.faded_line_ratio,
            self.sampling_budget,
        )?;

        // lines1 = (*x_lines1, *y_lines1), lines2 = (*x_lines2, *y_lines2).
        let bg = self.background_line_style;
        let background_lines = v_group(x_bg.into_iter().chain(y_bg))
            .map_style_deep(|s| s.stroke(bg.stroke_color, bg.stroke_width, bg.stroke_opacity));
        let faded_color = self
            .faded_line_style
            .stroke_color
            .unwrap_or(bg.stroke_color);
        let faded = self.faded_line_style;
        let faded_lines = v_group(x_faded.into_iter().chain(y_faded))
            .map_style_deep(|s| s.stroke(faded_color, faded.stroke_width, faded.stroke_opacity));

        // add_to_back(faded_lines, background_lines) behind the axes.
        let vmob = VMobject::new().with_children(vec![
            faded_lines.clone(),
            background_lines.clone(),
            axes.x_axis().vmob().clone(),
            axes.y_axis().vmob().clone(),
        ]);
        self.built = Some(PlaneBuilt {
            axes,
            background_lines,
            faded_lines,
            vmob,
        });
        Ok(self)
    }

    fn built(&self) -> &PlaneBuilt {
        self.built
            .as_ref()
            .expect("NumberPlane::build must run before geometry access")
    }

    /// The x-axis (Reference `get_x_axis`).
    #[must_use]
    pub fn x_axis(&self) -> &NumberLine {
        self.built().axes.x_axis()
    }

    /// The y-axis (Reference `get_y_axis`).
    #[must_use]
    pub fn y_axis(&self) -> &NumberLine {
        self.built().axes.y_axis()
    }

    /// The background-line family: all x-parallel lines first, then the
    /// y-parallel ones (the Reference's `VGroup(*x_lines1, *y_lines1)`).
    #[must_use]
    pub fn background_lines(&self) -> &VMobject {
        &self.built().background_lines
    }

    /// The faded-line family (same ordering as [`Self::background_lines`]).
    #[must_use]
    pub fn faded_lines(&self) -> &VMobject {
        &self.built().faded_lines
    }

    /// The x-axis unit size (Reference `get_x_unit_size`).
    #[must_use]
    pub fn x_unit_size(&self) -> f64 {
        self.x_axis().effective_unit_size()
    }

    /// The y-axis unit size. The Reference's `get_y_unit_size` returns
    /// the *x*-axis unit size (an upstream bug); ours returns the
    /// y-axis's (module docs).
    #[must_use]
    pub fn y_unit_size(&self) -> f64 {
        self.y_axis().effective_unit_size()
    }

    /// The assembled `[faded, background, x_axis, y_axis]` family.
    #[must_use]
    pub fn vmob(&self) -> &VMobject {
        &self.built().vmob
    }

    /// Consume into the assembled family.
    #[must_use]
    pub fn into_vmob(self) -> VMobject {
        self.built
            .expect("NumberPlane::build must run before into_vmob")
            .vmob
    }

    /// The Reference's `get_vector`: an [`Arrow`] from the origin to
    /// `c2p(coords)` with `buff=0`.
    #[must_use]
    pub fn get_vector(&self, coords: &[f64]) -> VMobject {
        Arrow::new(self.c2p(&[0.0, 0.0]), self.c2p(coords))
            .buff(0.0)
            .build()
            .expect("coordinate-system vectors are uncurved arrows")
    }

    /// The Reference's `get_graph`: sample `function` over `x_range`
    /// (default: the plane's own) at
    /// `num_sampled_graph_points_per_tick` points per tick, drawn
    /// through this plane's `c2p`. Discontinuity handling lives in
    /// [`ParametricCurve`]'s setters.
    #[must_use]
    pub fn get_graph(
        &self,
        function: impl Fn(f64) -> f64 + 'static,
        x_range: Option<[f64; 3]>,
    ) -> ParametricCurve {
        let x_axis = self.x_axis().clone();
        let y_axis = self.y_axis().clone();
        let c2p = move |coords: &[f64]| {
            let origin = x_axis.n2p(0.0);
            let mut point = origin;
            for (axis, &coord) in [&x_axis, &y_axis].into_iter().zip(coords.iter()) {
                let q = axis.n2p(coord);
                point = [
                    point[0] + q[0] - origin[0],
                    point[1] + q[1] - origin[1],
                    point[2] + q[2] - origin[2],
                ];
            }
            point
        };
        graph_parametric(
            function,
            c2p,
            x_range.unwrap_or(self.x_range),
            self.num_sampled_graph_points_per_tick,
            self.sampling_budget,
        )
    }
}

impl CoordinateSystem for NumberPlane {
    fn c2p(&self, coords: &[f64]) -> Vec3 {
        self.built().axes.c2p(coords)
    }

    fn p2c(&self, point: Vec3) -> [f64; 3] {
        self.built().axes.p2c(point)
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

impl NumberPlane {
    pub(crate) fn coordinate_axes(&self) -> Option<[&NumberLine; 2]> {
        let built = self.built.as_ref()?;
        Some([built.axes.x_axis(), built.axes.y_axis()])
    }
}

/// `ComplexPlane` — a [`NumberPlane`] whose coordinate labels are
/// complex numbers (§12.2, Reference `coordinate_systems.py:ComplexPlane`).
///
/// The plane itself is exactly a `NumberPlane`; this wrapper adds the
/// `n2p`/`p2n` complex-point shorthands and the label machinery. Labels
/// are axis number mobjects (module docs): real values ride the x-axis,
/// imaginary values ride the y-axis with the unit `i`.
///
/// [`DecimalNumber`]: crate::numbers::DecimalNumber
#[derive(Debug, Clone)]
pub struct ComplexPlane {
    plane: NumberPlane,
    coordinate_labels: Vec<VMobject>,
}

impl Default for ComplexPlane {
    fn default() -> Self {
        Self::new()
    }
}

impl ComplexPlane {
    /// The `NumberPlane` defaults (the Reference adds no constructor
    /// overrides of its own).
    #[must_use]
    pub fn new() -> Self {
        Self {
            plane: NumberPlane::new(),
            coordinate_labels: Vec::new(),
        }
    }

    /// `x_range=`.
    #[must_use]
    pub fn x_range(mut self, range: [f64; 3]) -> Self {
        self.plane = self.plane.x_range(range);
        self
    }

    /// `y_range=`.
    #[must_use]
    pub fn y_range(mut self, range: [f64; 3]) -> Self {
        self.plane = self.plane.y_range(range);
        self
    }

    /// `axis_config=`.
    #[must_use]
    pub fn axis_config(mut self, config: AxisConfig) -> Self {
        self.plane = self.plane.axis_config(config);
        self
    }

    /// `x_axis_config=`.
    #[must_use]
    pub fn x_axis_config(mut self, config: AxisConfig) -> Self {
        self.plane = self.plane.x_axis_config(config);
        self
    }

    /// `y_axis_config=`.
    #[must_use]
    pub fn y_axis_config(mut self, config: AxisConfig) -> Self {
        self.plane = self.plane.y_axis_config(config);
        self
    }

    /// `height=`.
    #[must_use]
    pub fn height(mut self, height: f64) -> Self {
        self.plane = self.plane.height(height);
        self
    }

    /// `width=`.
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.plane = self.plane.width(width);
        self
    }

    /// `unit_size=`.
    #[must_use]
    pub fn unit_size(mut self, unit_size: f64) -> Self {
        self.plane = self.plane.unit_size(unit_size);
        self
    }

    /// `background_line_style=`.
    #[must_use]
    pub fn background_line_style(mut self, style: LineFamilyStyle) -> Self {
        self.plane = self.plane.background_line_style(style);
        self
    }

    /// `faded_line_style=`.
    #[must_use]
    pub fn faded_line_style(mut self, style: FadedLineStyle) -> Self {
        self.plane = self.plane.faded_line_style(style);
        self
    }

    /// `faded_line_ratio=`.
    #[must_use]
    pub fn faded_line_ratio(mut self, ratio: usize) -> Self {
        self.plane = self.plane.faded_line_ratio(ratio);
        self
    }

    /// `num_sampled_graph_points_per_tick=`.
    #[must_use]
    pub fn num_sampled_graph_points_per_tick(mut self, n: f64) -> Self {
        self.plane = self.plane.num_sampled_graph_points_per_tick(n);
        self
    }

    /// Bound every sampling surface delegated to the underlying plane.
    #[must_use]
    pub fn sampling_budget(mut self, budget: SamplingBudget) -> Self {
        self.plane = self.plane.sampling_budget(budget);
        self
    }

    /// Assemble the underlying plane.
    pub fn build(mut self, book: &FontBook) -> Result<Self, CoordsError> {
        self.plane = self.plane.build(book)?;
        self.coordinate_labels = Vec::new();
        Ok(self)
    }

    /// `n2p`: the point of the complex number `(re, im)`.
    #[must_use]
    pub fn n2p(&self, z: [f64; 2]) -> Vec3 {
        self.plane.c2p(&z)
    }

    /// `p2n`: the complex number `(re, im)` of a point.
    #[must_use]
    pub fn p2n(&self, point: Vec3) -> [f64; 2] {
        let coords = self.plane.p2c(point);
        [coords[0], coords[1]]
    }

    /// The x-axis.
    #[must_use]
    pub fn x_axis(&self) -> &NumberLine {
        self.plane.x_axis()
    }

    /// The y-axis.
    #[must_use]
    pub fn y_axis(&self) -> &NumberLine {
        self.plane.y_axis()
    }

    /// The assembled plane family (labels appended as a final child
    /// once [`Self::add_coordinate_labels`] has run).
    #[must_use]
    pub fn vmob(&self) -> &VMobject {
        self.plane.vmob()
    }

    /// Consume into the assembled family.
    #[must_use]
    pub fn into_vmob(self) -> VMobject {
        self.plane.into_vmob()
    }

    /// The labels [`Self::add_coordinate_labels`] built, in
    /// [`Self::default_coordinate_values`] order.
    #[must_use]
    pub fn coordinate_labels(&self) -> &[VMobject] {
        &self.coordinate_labels
    }

    /// The Reference's `get_default_coordinate_values`: the x tick
    /// range (as reals) followed by the nonzero y tick range (as pure
    /// imaginaries), each skipping its first entry. The Reference's
    /// `skip_first` parameter is accepted but never used — the first
    /// tick is always dropped — so it is not part of this signature.
    pub fn default_coordinate_values(&self) -> Result<Vec<[f64; 2]>, SamplingError> {
        let x_ticks = self.x_axis().tick_range()?;
        let y_ticks = self.y_axis().tick_range()?;
        let x_count = x_ticks.len().saturating_sub(1);
        let y_count = y_ticks.iter().skip(1).filter(|y| **y != 0.0).count();
        let total = x_count
            .checked_add(y_count)
            .ok_or(SamplingError::CapacityOverflow {
                context: "complex-plane coordinate labels",
            })?;
        self.plane
            .sampling_budget
            .ensure_total("complex-plane coordinate labels", total)?;

        let mut values = Vec::new();
        values
            .try_reserve_exact(total)
            .map_err(|_| SamplingError::AllocationFailed {
                context: "complex-plane coordinate labels",
                samples: total,
            })?;
        values.extend(x_ticks.into_iter().skip(1).map(|x| [x, 0.0]));
        values.extend(
            y_ticks
                .into_iter()
                .skip(1)
                .filter(|y| *y != 0.0)
                .map(|y| [0.0, y]),
        );
        Ok(values)
    }

    /// The Reference's `add_coordinate_labels` over the default values.
    pub fn add_coordinate_labels(&mut self, book: &FontBook) -> Result<(), CoordsError> {
        let numbers = self.default_coordinate_values()?;
        self.add_labels(&numbers, book)
    }

    /// `add_coordinate_labels(numbers=...)`: explicit values as
    /// `(re, im)` pairs.
    pub fn add_coordinate_labels_for(
        &mut self,
        numbers: &[[f64; 2]],
        book: &FontBook,
    ) -> Result<(), CoordsError> {
        self.plane
            .sampling_budget
            .ensure_total("complex-plane labels", numbers.len())?;
        self.add_labels(numbers, book)
    }

    /// One number mobject per value: imaginary-dominant values go to
    /// the y-axis with the unit `i`, the rest to the x-axis — the
    /// Reference's `abs(z.imag) > abs(z.real)` split. The labels are
    /// appended to the plane as a single trailing group.
    fn add_labels(&mut self, numbers: &[[f64; 2]], book: &FontBook) -> Result<(), CoordsError> {
        let mut labels = Vec::with_capacity(numbers.len());
        for z in numbers {
            let (re, im) = (z[0], z[1]);
            let label = if im.abs() > re.abs() {
                self.y_axis().number_mobject_with_unit(im, 1.0, "i", book)?
            } else {
                self.x_axis().number_mobject(re, book)?
            };
            labels.push(label);
        }
        let built = self
            .plane
            .built
            .as_mut()
            .expect("ComplexPlane::build must run before add_coordinate_labels");
        built.vmob = core::mem::take(&mut built.vmob).with_child(v_group(labels.clone()));
        self.coordinate_labels = labels;
        Ok(())
    }

    /// The Reference's `get_vector`, delegated.
    #[must_use]
    pub fn get_vector(&self, coords: &[f64]) -> VMobject {
        self.plane.get_vector(coords)
    }

    /// The Reference's `get_graph`, delegated.
    #[must_use]
    pub fn get_graph(
        &self,
        function: impl Fn(f64) -> f64 + 'static,
        x_range: Option<[f64; 3]>,
    ) -> ParametricCurve {
        self.plane.get_graph(function, x_range)
    }
}

impl CoordinateSystem for ComplexPlane {
    fn c2p(&self, coords: &[f64]) -> Vec3 {
        self.plane.c2p(coords)
    }

    fn p2c(&self, point: Vec3) -> [f64; 3] {
        self.plane.p2c(point)
    }

    fn all_ranges(&self) -> Vec<[f64; 3]> {
        self.plane.all_ranges()
    }

    fn num_sampled_graph_points_per_tick(&self) -> f64 {
        CoordinateSystem::num_sampled_graph_points_per_tick(&self.plane)
    }

    fn graph_sampling_budget(&self) -> SamplingBudget {
        self.plane.graph_sampling_budget()
    }
}

impl ComplexPlane {
    pub(crate) fn coordinate_axes(&self) -> Option<[&NumberLine; 2]> {
        self.plane.coordinate_axes()
    }
}

/// Like [`Axes`], a built plane is added to a stage directly.
impl From<NumberPlane> for fmn_mobject::Mobject {
    fn from(plane: NumberPlane) -> Self {
        plane.into_vmob().into()
    }
}

impl From<ComplexPlane> for fmn_mobject::Mobject {
    fn from(plane: ComplexPlane) -> Self {
        plane.into_vmob().into()
    }
}

impl From<ThreeDAxes> for fmn_mobject::Mobject {
    fn from(axes: ThreeDAxes) -> Self {
        axes.into_vmob().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::numbers::DecimalNumber;
    use fmn_core::constants::{GREEN, LEFT, RED, RIGHT, UR, WHITE};

    fn book() -> FontBook {
        FontBook::bundled().expect("bundled faces parse")
    }

    fn assert_point_near(actual: Vec3, expected: Vec3, tol: f64, what: &str) {
        for dim in 0..3 {
            assert!(
                (actual[dim] - expected[dim]).abs() <= tol,
                "{what}: dim {dim} expected {} got {}",
                expected[dim],
                actual[dim]
            );
        }
    }

    /// The center of a line's bounding box: its midpoint.
    fn line_center(line: &VMobject) -> Vec3 {
        line.center_point()
    }

    // ---- ThreeDAxes ------------------------------------------------

    #[test]
    fn three_d_axes_default_ranges_and_dimension() {
        let axes = ThreeDAxes::new().build(&book()).expect("build");
        assert_eq!(
            axes.all_ranges(),
            vec![THREE_D_X_RANGE, THREE_D_Y_RANGE, THREE_D_Z_RANGE]
        );
        assert_eq!(axes.dimension(), 3);
    }

    #[test]
    fn three_d_axes_z_axis_points_out() {
        let axes = ThreeDAxes::new().build(&book()).expect("build");
        // The two Reference rotations take the horizontal z NumberLine
        // onto the OUT axis: c2p z ↦ [0, 0, z].
        assert_point_near(axes.c2p(&[0.0, 0.0, 1.0]), OUT, 1e-9, "c2p(+z)");
        assert_point_near(
            axes.c2p(&[0.0, 0.0, -2.0]),
            [0.0, 0.0, -2.0],
            1e-9,
            "c2p(-2z)",
        );
        // x and y are untouched by the z machinery.
        assert_point_near(axes.c2p(&[1.0, 0.0, 0.0]), RIGHT, 1e-9, "c2p(+x)");
        assert_point_near(axes.c2p(&[0.0, 1.0, 0.0]), UP, 1e-9, "c2p(+y)");
        // The z NumberLine spans z_min..z_max along OUT.
        assert_point_near(
            axes.z_axis().start_point(),
            [0.0, 0.0, -4.0],
            1e-9,
            "z start",
        );
        assert_point_near(axes.z_axis().end_point(), [0.0, 0.0, 4.0], 1e-9, "z end");
    }

    #[test]
    fn three_d_axes_z_normal_only_reorients_ticks() {
        // z_normal rotates about the z-axis itself: the axis line cannot
        // move, whatever the normal.
        for normal in [DOWN, LEFT, [1.0, 1.0, 0.0]] {
            let axes = ThreeDAxes::new()
                .z_normal(normal)
                .build(&book())
                .expect("build");
            assert_point_near(axes.c2p(&[0.0, 0.0, 2.0]), [0.0, 0.0, 2.0], 1e-9, "z fixed");
        }
    }

    #[test]
    fn three_d_axes_round_trip() {
        let axes = ThreeDAxes::new().build(&book()).expect("build");
        for x in [-5.5, -1.0, 0.0, 2.25, 6.0] {
            for y in [-4.0, 0.5, 5.0] {
                for z in [-3.75, 0.0, 4.0] {
                    let coords = [x, y, z];
                    let back = axes.p2c(axes.c2p(&coords));
                    for dim in 0..3 {
                        assert!(
                            (back[dim] - coords[dim]).abs() < 1e-9,
                            "round trip {coords:?}: dim {dim} got {}",
                            back[dim]
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn three_d_axes_round_trip_custom_ranges_and_unit_size() {
        let axes = ThreeDAxes::new()
            .x_range([-2.0, 4.0, 0.5])
            .y_range([1.0, 3.0, 0.25])
            .z_range([0.0, 2.0, 0.5])
            .unit_size(0.7)
            .build(&book())
            .expect("build");
        // The z-axis ignores unit_size (Reference quirk): +1z is one
        // full manim unit OUT, while +1x is 0.7 RIGHT of the origin.
        let origin = axes.origin();
        let pz = axes.c2p(&[1.0, 1.0, 1.0]);
        assert!(
            (pz[2] - origin[2] - 1.0).abs() < 1e-9,
            "z unit is 1: {pz:?}"
        );
        assert!(
            (pz[0] - origin[0] - 0.7).abs() < 1e-9,
            "x unit is 0.7: {pz:?}"
        );
        for coords in [[-1.5, 1.25, 0.5], [3.0, 2.0, 1.75], [0.0, 3.0, 0.0]] {
            let back = axes.p2c(axes.c2p(&coords));
            for dim in 0..3 {
                assert!((back[dim] - coords[dim]).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn three_d_axes_depth_sets_z_length() {
        // depth=4 over a range of span 8: z unit size is 0.5.
        let axes = ThreeDAxes::new().depth(4.0).build(&book()).expect("build");
        let origin = axes.origin();
        let pz = axes.c2p(&[0.0, 0.0, 1.0]);
        assert!((pz[2] - origin[2] - 0.5).abs() < 1e-9, "z unit 0.5: {pz:?}");
        assert_point_near(axes.z_axis().end_point(), [0.0, 0.0, 2.0], 1e-9, "z end");
    }

    #[test]
    fn three_d_axes_family_is_flat() {
        let axes = ThreeDAxes::new().build(&book()).expect("build");
        // [x_axis, y_axis, z_axis], the Reference's add order.
        assert_eq!(axes.vmob().children().len(), 3);
    }

    // ---- NumberPlane: line families --------------------------------

    #[test]
    fn number_plane_default_line_counts() {
        let plane = NumberPlane::new().build(&book()).expect("build");
        // numpy-verified fixtures: x-parallel (8 bg / 32 faded at
        // y=-4..4), y-parallel (16 bg / 64 faded at x=-8..8).
        let background = plane.background_lines().children();
        let faded = plane.faded_lines().children();
        assert_eq!(background.len(), 24);
        assert_eq!(faded.len(), 96);
        // x-parallel lines come first in each family.
        for (i, line) in background.iter().enumerate() {
            let points = line.points();
            let start = points.first().expect("a line has points");
            let end = points.last().expect("a line has points");
            if i < 8 {
                assert!((start[1] - end[1]).abs() < 1e-9, "x-parallel bg line {i}");
            } else {
                assert!((start[0] - end[0]).abs() < 1e-9, "y-parallel bg line {i}");
            }
        }
        // The family sits behind the axes in the assembled vmob.
        assert_eq!(plane.vmob().children().len(), 4);
    }

    #[test]
    fn number_plane_small_fixture_positions_and_split() {
        // x_range=y_range=(-2, 2, 1), faded_line_ratio=2: step = 1/3,
        // inputs -2 + i/3 for i in 0..14, skip i=6 (x=0), background at
        // i % 3 == 0 — numpy-verified: background at -2, -1, 1, 2; faded
        // at -5/3, -4/3, -2/3, -1/3, 1/3, 2/3, 4/3, 5/3, 7/3 (the 7/3
        // overshoot past x_max is the Reference's arange semantics).
        let plane = NumberPlane::new()
            .x_range([-2.0, 2.0, 1.0])
            .y_range([-2.0, 2.0, 1.0])
            .faded_line_ratio(2)
            .build(&book())
            .expect("build");
        let background = plane.background_lines().children();
        let faded = plane.faded_lines().children();
        assert_eq!(background.len(), 8, "4 per orientation");
        assert_eq!(faded.len(), 18, "9 per orientation");

        // x-parallel family members are horizontal lines at these
        // heights, spanning the x-axis's full extent.
        let expected_bg_heights = [-2.0, -1.0, 1.0, 2.0];
        for (line, &y) in background.iter().zip(&expected_bg_heights).take(4) {
            let c = line_center(line);
            assert!((c[1] - y).abs() < 1e-9, "bg line height {y} got {c:?}");
            assert!((c[0]).abs() < 1e-9, "bg line centered in x: {c:?}");
            assert!(
                (line.length_over_dim(0) - 4.0).abs() < 1e-9,
                "bg line spans x"
            );
        }
        let expected_faded_heights = [
            -5.0 / 3.0,
            -4.0 / 3.0,
            -2.0 / 3.0,
            -1.0 / 3.0,
            1.0 / 3.0,
            2.0 / 3.0,
            4.0 / 3.0,
            5.0 / 3.0,
            7.0 / 3.0,
        ];
        for (line, &y) in faded.iter().zip(&expected_faded_heights).take(9) {
            let c = line_center(line);
            assert!((c[1] - y).abs() < 1e-9, "faded line height {y} got {c:?}");
        }
        // y-parallel members mirror them (constant-x lines).
        let y_bg = &background[4..];
        for (line, &x) in y_bg.iter().zip(&expected_bg_heights) {
            let c = line_center(line);
            assert!((c[0] - x).abs() < 1e-9, "bg line x {x} got {c:?}");
            assert!(
                (line.length_over_dim(1) - 4.0).abs() < 1e-9,
                "bg line spans y"
            );
        }
        // No line at 0: the skip covers both families.
        for line in background.iter().chain(faded) {
            let c = line_center(line);
            assert!(
                c[0].abs() > 1e-6 || c[1].abs() > 1e-6,
                "no line at the axis"
            );
        }
    }

    #[test]
    fn number_plane_family_styles() {
        let plane = NumberPlane::new().build(&book()).expect("build");
        for line in plane.background_lines().children() {
            let style = line.style();
            assert_eq!(style.stroke_color, BLUE_D);
            assert!((style.stroke_width - 2.0).abs() < 1e-12);
            assert!((style.stroke_opacity - 1.0).abs() < 1e-12);
        }
        for line in plane.faded_lines().children() {
            let style = line.style();
            // The Reference defaults the faded stroke color to the
            // background's.
            assert_eq!(style.stroke_color, BLUE_D);
            assert!((style.stroke_width - 1.0).abs() < 1e-12);
            assert!((style.stroke_opacity - 0.25).abs() < 1e-12);
        }
    }

    #[test]
    fn number_plane_axes_stroke_white_and_keep_the_number_line_fill() {
        // The Reference's NumberPlane.default_axis_config sets only
        // stroke_color, so each axis keeps NumberLine's DEFAULT_LIGHT_COLOR
        // fill (fm-78b6: get_fill_color() is #BBBBBB on the Reference).
        let plane = NumberPlane::new().build(&book()).expect("build");
        for axis in [plane.x_axis(), plane.y_axis()] {
            let style = axis.vmob().style();
            assert_eq!(style.stroke_color, WHITE);
            assert_eq!(style.fill_color, fmn_core::constants::DEFAULT_LIGHT_COLOR);
        }
    }

    #[test]
    fn number_plane_faded_color_inherits_and_overrides() {
        let plane = NumberPlane::new()
            .background_line_style(LineFamilyStyle {
                stroke_color: RED,
                ..LineFamilyStyle::default()
            })
            .build(&book())
            .expect("build");
        assert_eq!(
            plane.faded_lines().children()[0].style().stroke_color,
            RED,
            "faded inherits the background color"
        );
        let plane = NumberPlane::new()
            .background_line_style(LineFamilyStyle {
                stroke_color: RED,
                ..LineFamilyStyle::default()
            })
            .faded_line_style(FadedLineStyle {
                stroke_color: Some(GREEN),
                ..FadedLineStyle::default()
            })
            .build(&book())
            .expect("build");
        assert_eq!(
            plane.faded_lines().children()[0].style().stroke_color,
            GREEN,
            "explicit faded color wins"
        );
        assert_eq!(
            plane.background_lines().children()[0].style().stroke_color,
            RED
        );
    }

    #[test]
    fn number_plane_axes_are_bare_white_lines() {
        let plane = NumberPlane::new().build(&book()).expect("build");
        // The NumberPlane axis config: no ticks, no tip, no numbers —
        // each axis is its line and nothing else.
        assert!(plane.x_axis().vmob().children().is_empty());
        assert!(plane.y_axis().vmob().children().is_empty());
        let style = plane.x_axis().vmob().style();
        assert_eq!(style.stroke_color, WHITE);
        assert!((style.stroke_width - 2.0).abs() < 1e-12);
    }

    #[test]
    fn number_plane_c2p_p2c_round_trip() {
        for plane in [
            NumberPlane::new().build(&book()).expect("build"),
            NumberPlane::new()
                .x_range([-2.0, 4.0, 0.5])
                .y_range([1.0, 3.0, 0.25])
                .unit_size(0.5)
                .build(&book())
                .expect("build"),
        ] {
            for x in [-7.5, -0.25, 0.0, 3.5] {
                for y in [-3.0, 0.0, 2.75] {
                    let coords = [x, y];
                    let back = plane.p2c(plane.c2p(&coords));
                    assert!((back[0] - x).abs() < 1e-9, "x round trip: {back:?}");
                    assert!((back[1] - y).abs() < 1e-9, "y round trip: {back:?}");
                }
            }
        }
    }

    #[test]
    fn number_plane_get_vector_spans_origin_to_coords() {
        let plane = NumberPlane::new().build(&book()).expect("build");
        let vector = plane.get_vector(&[2.0, 1.0]);
        // The tip lands exactly on c2p(coords)…
        let tip_reached = vector
            .points()
            .iter()
            .any(|p| (p[0] - 2.0).abs() < 1e-9 && (p[1] - 1.0).abs() < 1e-9);
        assert!(tip_reached, "tip at c2p(2, 1)");
        // …and the shaft starts at the origin (its corners sit half a
        // thickness off-axis, so the extent is not exactly ORIGIN).
        let (_, max) = vector.extent().expect("vector has extent");
        assert!((max[0] - 2.0).abs() < 1e-9, "extent reaches x=2: {max:?}");
        let near_origin = vector
            .points()
            .iter()
            .any(|p| p[0].abs() < 0.05 && p[1].abs() < 0.05);
        assert!(near_origin, "shaft at the origin");
    }

    #[test]
    fn number_plane_unit_sizes() {
        let plane = NumberPlane::new().width(6.0).build(&book()).expect("build");
        // width=6 over a span of 16: x unit 0.375; y untouched at 1.
        assert!((plane.x_unit_size() - 0.375).abs() < 1e-9);
        assert!((plane.y_unit_size() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn arange_matches_numpy() {
        // The fixture generator: numpy gives
        // arange(-2, 2 + 1/3, 1/3) == -2 + i/3 for i in 0..14.
        let values = arange(-2.0, 2.0 + 1.0 / 3.0, 1.0 / 3.0, SamplingBudget::default())
            .expect("fixture sampling is bounded");
        assert_eq!(values.len(), 14);
        for (i, v) in values.iter().enumerate() {
            assert!((v - (-2.0 + i as f64 / 3.0)).abs() < 1e-12);
        }
        // arange(-4, 4.2, 0.2): 41 values (0.2 quotient undershoots 41).
        assert_eq!(
            arange(-4.0, 4.0 + 0.2, 0.2, SamplingBudget::default())
                .expect("fixture sampling is bounded")
                .len(),
            41
        );
        // A non-positive step yields nothing (numpy raises there).
        assert!(
            arange(0.0, 1.0, 0.0, SamplingBudget::default())
                .expect("zero step has no samples")
                .is_empty()
        );
        assert!(
            arange(0.0, 1.0, -0.5, SamplingBudget::default())
                .expect("backward step has no samples")
                .is_empty()
        );

        let error = arange(-2.0, 2.0 + 1.0 / 3.0, 1.0 / 3.0, SamplingBudget::new(13))
            .expect_err("the same fourteen-value range exceeds a thirteen-sample budget");
        assert!(matches!(
            error,
            SamplingError::LimitExceeded {
                context: "number-plane lines",
                max_samples: 13
            }
        ));
    }

    #[test]
    fn number_plane_refuses_ratio_overflow_and_dense_ranges() {
        let overflow = NumberPlane::new()
            .faded_line_ratio(usize::MAX)
            .build(&book())
            .expect_err("the dense-frequency addition must not wrap");
        assert!(matches!(
            overflow,
            CoordsError::Sampling(SamplingError::CapacityOverflow {
                context: "number-plane faded-line ratio"
            })
        ));

        let dense = NumberPlane::new()
            .x_range([0.0, 1.0, f64::MIN_POSITIVE])
            .y_range([0.0, 1.0, 1.0])
            .sampling_budget(SamplingBudget::new(32))
            .build(&book())
            .expect_err("a tiny finite grid step must be refused");
        assert!(matches!(
            dense,
            CoordsError::Sampling(SamplingError::LimitExceeded {
                context: "number-plane lines",
                max_samples: 32
            })
        ));
    }

    // ---- ComplexPlane -----------------------------------------------

    #[test]
    fn complex_plane_n2p_p2n_round_trip() {
        let plane = ComplexPlane::new().build(&book()).expect("build");
        for z in [[-3.5, 2.25], [0.0, 0.0], [7.9, -3.9]] {
            let back = plane.p2n(plane.n2p(z));
            assert!((back[0] - z[0]).abs() < 1e-9);
            assert!((back[1] - z[1]).abs() < 1e-9);
        }
    }

    #[test]
    fn complex_plane_default_coordinate_values() {
        let plane = ComplexPlane::new().build(&book()).expect("build");
        let values = plane
            .default_coordinate_values()
            .expect("default ticks are bounded");
        // tick ranges -8..=8 and -4..=4, first entry dropped, y's zero
        // dropped: 16 reals then 7 pure imaginaries (numpy-verified).
        assert_eq!(values.len(), 23);
        for (i, v) in values.iter().take(16).enumerate() {
            let expected = (i as f64) - 7.0;
            assert_eq!(*v, [expected, 0.0], "real value {i}");
        }
        let expected_imag = [-3.0, -2.0, -1.0, 1.0, 2.0, 3.0, 4.0];
        for (v, &im) in values[16..].iter().zip(&expected_imag) {
            assert_eq!(*v, [0.0, im]);
        }
    }

    #[test]
    fn complex_plane_combined_coordinate_values_obey_one_budget() {
        let plane = ComplexPlane::new()
            .faded_line_ratio(0)
            .sampling_budget(SamplingBudget::new(17))
            .build(&book())
            .expect("each individual default range fits seventeen samples");
        let error = plane
            .default_coordinate_values()
            .expect_err("the combined twenty-three labels exceed the shared budget");
        assert!(matches!(
            error,
            SamplingError::LimitExceeded {
                context: "complex-plane coordinate labels",
                max_samples: 17
            }
        ));
    }

    #[test]
    fn complex_plane_label_glyph_sequences() {
        let book = book();
        let mut plane = ComplexPlane::new().build(&book).expect("build");
        plane.add_coordinate_labels(&book).expect("labels");
        let labels = plane.coordinate_labels();
        assert_eq!(labels.len(), 23);
        let values = plane
            .default_coordinate_values()
            .expect("default ticks are bounded");

        for (label, z) in labels.iter().zip(&values) {
            let (re, im) = (z[0], z[1]);
            if im.abs() > re.abs() {
                // Imaginary path: DecimalNumber(im, ndp=0, unit="i") with
                // the Reference's |v|==1 digit drop.
                let expected_children = if im.abs() == 1.0 {
                    if im < 0.0 { 2 } else { 1 } // "–i" / "i"
                } else if im < 0.0 {
                    3 // dash + digits + unit
                } else {
                    2 // digits + unit
                };
                assert_eq!(label.children().len(), expected_children, "label for {im}i");
                // Positioned DL of the axis point at SMALL_BUFF. For
                // |im| == 1 the Reference drops the digit and then
                // `move_to`s the pre-drop center, so the UR corner is no
                // longer at the anchor — assert that center instead.
                let axis_point = plane.y_axis().n2p(im);
                if im.abs() == 1.0 {
                    let oracle = DecimalNumber::new(im)
                        .num_decimal_places(0)
                        .font_size(36.0)
                        .unit("i")
                        .build(&book)
                        .expect("oracle builds")
                        .into_vmob()
                        .next_to_point(axis_point, DL, SMALL_BUFF, ORIGIN);
                    assert_point_near(
                        label.center_point(),
                        oracle.center_point(),
                        1e-9,
                        "±i label keeps the pre-drop center",
                    );
                } else {
                    let ur = label.bbox_point(UR).expect("label extent");
                    assert_point_near(
                        ur,
                        [
                            axis_point[0] + DL[0] * SMALL_BUFF,
                            axis_point[1] + DL[1] * SMALL_BUFF,
                            axis_point[2],
                        ],
                        1e-9,
                        "imag label position",
                    );
                }
            } else {
                // Real path: plain DecimalNumber(re, ndp=0).
                let digits = format!("{}", re.abs() as i64).len();
                let expected_children = digits + usize::from(re < 0.0);
                assert_eq!(label.children().len(), expected_children, "label for {re}");
                let axis_point = plane.x_axis().n2p(re);
                let ur = label.bbox_point(UR).expect("label extent");
                assert_point_near(
                    ur,
                    [
                        axis_point[0] + DL[0] * SMALL_BUFF,
                        axis_point[1] + DL[1] * SMALL_BUFF,
                        axis_point[2],
                    ],
                    1e-9,
                    "real label position",
                );
            }
        }
    }

    #[test]
    fn complex_plane_labels_match_decimal_number_oracle() {
        // Independent oracle: a real-axis label is exactly a
        // DecimalNumber(value, ndp=0, font_size=36) placed next to the
        // axis point, DL at SMALL_BUFF.
        let book = book();
        let mut plane = ComplexPlane::new().build(&book).expect("build");
        plane
            .add_coordinate_labels_for(&[[2.0, 0.0]], &book)
            .expect("labels");
        let label = &plane.coordinate_labels()[0];
        let oracle = DecimalNumber::new(2.0)
            .num_decimal_places(0)
            .font_size(36.0)
            .build(&book)
            .expect("oracle builds")
            .into_vmob()
            .next_to_point(plane.x_axis().n2p(2.0), DL, SMALL_BUFF, ORIGIN);
        assert_eq!(*label, oracle, "real label is a DecimalNumber at DL");
    }

    #[test]
    fn complex_plane_zero_labels_real_axis() {
        // |0i| > |0| is false: zero lands on the real axis — the
        // Reference's split at equality.
        let book = book();
        let mut plane = ComplexPlane::new().build(&book).expect("build");
        plane
            .add_coordinate_labels_for(&[[0.0, 0.0]], &book)
            .expect("labels");
        let label = &plane.coordinate_labels()[0];
        assert_eq!(label.children().len(), 1, "a lone \"0\"");
        let ur = label.bbox_point(UR).expect("label extent");
        let axis_point = plane.x_axis().n2p(0.0);
        assert_point_near(
            ur,
            [
                axis_point[0] + DL[0] * SMALL_BUFF,
                axis_point[1] + DL[1] * SMALL_BUFF,
                axis_point[2],
            ],
            1e-9,
            "zero label on the real axis",
        );
    }

    #[test]
    fn complex_plane_imaginary_labels_use_unit_i() {
        // The unit path: 2i keeps digit and unit, i drops the digit.
        let book = book();
        let mut plane = ComplexPlane::new().build(&book).expect("build");
        plane
            .add_coordinate_labels_for(&[[0.0, 2.0], [0.0, 1.0], [0.0, -1.0]], &book)
            .expect("labels");
        let labels = plane.coordinate_labels();
        assert_eq!(labels[0].children().len(), 2, "\"2i\": digit + unit");
        assert_eq!(labels[1].children().len(), 1, "\"i\": unit alone");
        assert_eq!(labels[2].children().len(), 2, "\"–i\": dash + unit");
    }

    #[test]
    fn complex_plane_labels_appended_as_trailing_group() {
        let book = book();
        let mut plane = ComplexPlane::new().build(&book).expect("build");
        plane.add_coordinate_labels(&book).expect("labels");
        let children = plane.vmob().children();
        // [faded, background, x_axis, y_axis, coordinate_labels].
        assert_eq!(children.len(), 5);
        assert_eq!(children[4].children().len(), 23);
    }
}
