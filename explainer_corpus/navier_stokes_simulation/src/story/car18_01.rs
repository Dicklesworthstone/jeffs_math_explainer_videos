//! Chapter 18, "Why a car is harder": cues 18_1 to 18_8 and the 18_h1 hold, native and exact.
//! Nothing here is a flow field: every picture is a static or kinematic schematic of our own
//! design, defined in [`super::shapes18`], and labelled as one.
//!
//! RubyRaven `ruby-car01` for root's `navier_stokes_simulation` crate (Agent Mail 36250). Cue text
//! and pictures: `navier-full-script03.md` chapter 18 (sha256 31a83a2b…). Numbers: ST96 §2.1–2.2
//! (the test), independent01 `TREATMENT.md` line 271 (the car), ST96 §6 (the quote);
//! `tools/check_car01.py` recomputes and quotes them from those files.
//!
//! The pictures, in the order the lines ask for them:
//! - 18_1: chapter 17's channel as 17_8–17_10 left it (walls, cylinder, "no slip", the blue
//!   inflow, "outlet: each group's choice") with chapter 13's "2D model" tag; below it our own
//!   car-like outline, "illustrative shape, no flow"; then "drag: ?";
//! - 18_2: the outline moves aside; the third direction, both ways, as a faint arrow through it,
//!   "the direction we left out"; the question card;
//! - 18_h1: the 5 s countdown; the extrusion does not exist yet;
//! - 18_3: the answer; the outline extrudes both ways into a ridge that fades out at its ends,
//!   "a car-shaped wall that never ends"; arrows over and under it, at the slice and once behind
//!   it, "2D model:
//!   w = 0, ∂/∂z = 0"; then a block car of finite width with arrows over, under and around;
//! - 18_4: the outline on a road, its wheels turning as it rolls in ("seen from the road",
//!   "car: +U"); then the car's frame: the car stands, the road slides back ("road: −U") and the
//!   wind arrives ("air: −U"), the wheels still turning at the same rate, "frame change only: no
//!   flow computed"; "ground treatment ?",
//!   "wheels ?";
//! - 18_5: a vortex tube stretched to twice its length at constant volume and circulation:
//!   thinner (÷√2) and spinning twice as fast; a flat swirl drifting with its spin unchanged,
//!   "(ω·∇)u = ω_z ∂u/∂z = 0", "no viscosity, no twisting force";
//! - 18_6: a slice through a schematic 3D flow, arrows crossing it ("w ≠ 0") and changing along
//!   it ("∂u/∂z ≠ 0"); the ST96 §6 paragraph, verbatim;
//! - 18_7: a logarithmic bar, "test: Re 100" and "car: Re ≈ 9,000,000", "× 90,000", with both
//!   formulas;
//! - 18_8: the channel again, "convergence: ?" and "benchmark: ?" before the car; the outline's
//!   permanent label, "flat illustration, not a car measurement"; "no drag or downforce read
//!   from it".

use std::cell::RefCell;
use std::f64::consts::{FRAC_PI_2, TAU};
use std::rc::Rc;

use super::shapes18::{self as sh, Pose, block, car, log_bar, ridge, slice, tube};
use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Palette, Timed, hex};
use fmn::prelude::*;

// ------------------------------------------------------------------ layout

/// The title, at chapter 17's place and size.
pub const TITLE: &str = "Why a car is harder";
pub const TITLE_AT: (f64, f64) = (-7.4, 3.95);
pub const TITLE_PT: f64 = 40.0;
pub const BODY_PT: f64 = 28.0;
pub const SMALL_PT: f64 = 24.0;
pub const FINE_PT: f64 = 22.0;

/// Chapter 17's channel as 17_8 placed it: metres from the inlet's lower corner, at 17_7's
/// scale, lifted by `CH_LIFT`.
pub const CH_SCALE: f64 = 6.0;
pub const CH_ORIGIN: (f64, f64) = (-6.6, -1.5);
pub const CH_LIFT: f64 = 1.2;
pub mod channel {
    pub const LENGTH: f64 = 2.2;
    pub const HEIGHT: f64 = 0.41;
    pub const DIAMETER: f64 = 0.1;
    pub const CENTRE: (f64, f64) = (0.2, 0.2);
    /// ST96 §2.2: the inflow's peak, m/s.
    pub const PEAK: f64 = 1.5;
}
/// 17_8's inflow: scene units per m/s, arrows at k H / 8.
pub const INLET_SCALE: f64 = 0.45;
pub const INLET_ARROWS: usize = 7;
pub const NO_SLIP: &str = "no slip";
pub const NO_SLIP_AT: [(f64, f64); 3] = [(0.0, 2.45), (0.0, -0.6), (-4.85, 0.9)];
pub const OUTLET: &str = "outlet: each group's choice";
pub const OUTLET_AT: (f64, f64) = (6.35, 0.93);
/// Chapter 13's label for the model (13_7).
pub const MODEL: &str = "2D model";
pub const MODEL_AT: (f64, f64) = (1.4, 0.93);
pub const CHANNEL_NAME: &str = "the benchmark channel (ch. 17)";
pub const CHANNEL_NAME_AT: (f64, f64) = (-6.6, 3.0);

/// Where the outline stands: 18_1 and 18_8 (under the channel), 18_2–18_3 (left), 18_4 (on the
/// road, once it has driven in by `DRIVE` scene units).
pub const POSE_A: Pose = Pose {
    origin: (-2.925, -3.8),
    scale: 1.3,
};
pub const POSE_B: Pose = Pose {
    origin: (-5.3, -1.0),
    scale: 1.0,
};
pub const POSE_C: Pose = Pose {
    origin: (-2.925, -1.6),
    scale: 1.3,
};
pub const DRIVE: f64 = 3.0;
/// 18_3's block car.
pub const BLOCK_POSE: Pose = Pose {
    origin: (1.7, -1.0),
    scale: 1.0,
};

/// 18_1 and 18_8.
pub const SHAPE_NOTE: &str = "illustrative shape, no flow";
pub const PERMANENT: &str = "flat illustration, not a car measurement";
pub const NOTE_AT: (f64, f64) = (0.0, -4.1);
pub const DRAG: &str = "drag: ?";
pub const NOT_READ: &str = "no drag or downforce read from it";
pub const DRAG_AT: (f64, f64) = (3.4, -2.9);

/// 18_2: the third direction through the outline's side (metres, car frame), each way.
pub const AXIS_AT: (f64, f64, f64) = (2.25, 0.8, 0.0);
pub const AXIS_REACH: f64 = 2.5;
pub const LEFT_OUT: &str = "the direction we left out";
pub const LEFT_OUT_AT: (f64, f64) = (-3.0, 1.25);

/// 18_h1.
pub const Q_CARD_AT: (f64, f64) = (3.9, -0.6);
pub const Q_CARD_SIZE: (f64, f64) = (7.3, 3.0);
pub const Q_LABEL: &str = "your prediction";
pub const QUESTION: &str = "What does a flat car stand for?";
pub const CHOICES: [&str; 3] = [
    "a car",
    "a thin slice of a car",
    "a car-shaped wall that goes on sideways forever",
];
/// The endless wall (18_3).
pub const ANSWER: usize = 2;
pub const PAUSE_NOTE: &str = "pause for more time";
pub const HOLD_SECONDS: f64 = 5.0;
pub const RING_R: f64 = 0.3;
pub const CHOICE_STEP: f64 = 0.38;

/// 18_3.
pub const RIDGE_NAME: &str = "a car-shaped wall that never ends";
pub const RIDGE_NAME_AT: (f64, f64) = (-3.3, 2.15);
pub const OVER: &str = "over";
pub const UNDER: &str = "under";
pub const AROUND: &str = "around its sides";
/// Air passes the car nose to tail (car frame), metres: over the roof and under the wheels.
pub const OVER_PATH: [(f64, f64); 3] = [(5.0, 1.25), (2.25, 1.95), (-0.5, 1.25)];
pub const UNDER_Y: f64 = -0.2;
pub const PATH_X: (f64, f64) = (5.0, -0.5);
/// The faint copy of the ridge's arrows behind the slice, at this depth (metres).
pub const ECHO_Z: [f64; 1] = [1.8];
pub const MODEL_TEX: &str = r"\text{2D model: } w = 0,\ \partial/\partial z = 0";
pub const MODEL_TEX_AT: (f64, f64) = (-3.3, -2.55);
pub const BLOCK_NAME: &str = "a real car: finite width";
pub const BLOCK_NAME_AT: (f64, f64) = (4.6, -2.55);
pub const SCHEMATIC: &str = "schematic arrows, not measured flow";
pub const SCHEMATIC_AT: (f64, f64) = (0.0, -3.4);
/// The block car's arrows (car frame, metres): over at mid-width and under at the near edge;
/// around the near side is [`block::around`].
pub const BLOCK_OVER: [(f64, f64, f64); 3] = [(5.0, 1.55, 0.9), (2.25, 2.0, 0.9), (-0.5, 1.55, 0.9)];
/// The block car's labels: over, under, around its sides.
pub const LABEL_AT: [(f64, f64); 3] = [(4.35, 1.5), (3.6, -1.55), (5.95, -1.95)];

/// 18_4.
/// The road's dashes run past both frame edges (x = ±8), so a dash wraps unseen.
pub const ROAD_X: (f64, f64) = (-8.4, 8.4);
pub const DASH: f64 = 0.3;
pub const DASH_PERIOD: f64 = 0.6;
pub const SPEED_MAX: f64 = 2.4;
pub const GROUND_NAME: &str = "seen from the road";
pub const CAR_NAME: &str = "seen from the car";
pub const FRAME_NAME_AT: (f64, f64) = (-7.4, 3.1);
/// The car's arrow above the roof, and the air's ahead of the nose (car frame, metres).
pub const CAR_ARROW: [(f64, f64); 2] = [(1.6, 1.75), (2.9, 1.75)];
pub const CAR_ARROW_LABEL: (f64, f64) = (2.25, 2.12);
pub const AIR_X: (f64, f64) = (6.8, 5.3);
pub const AIR_Y: [f64; 3] = [0.45, 0.95, 1.45];
pub const AIR_LABEL_AT: (f64, f64) = (4.94, 0.85);
pub const FRAME_TAG: &str = "frame change only: no flow computed";
pub const FRAME_TAG_AT: (f64, f64) = (0.0, -2.6);
pub const ROAD_ROW_Y: f64 = -2.05;
pub const GROUND_Q: &str = "ground treatment ?";
pub const WHEELS_Q: &str = "wheels ?";
pub const ROAD_ROW_X: [f64; 3] = [-4.6, 0.0, 4.6];
pub const SPOKES: usize = 3;

/// 18_5.
pub const TUBE_AT: (f64, f64) = (-3.9, 0.3);
pub const TUBE_L0: f64 = 3.4;
pub const TUBE_R0: f64 = 0.9;
pub const SPIN0: f64 = 1.2;
pub const STRETCH_T: f64 = 3.0;
pub const SPIN_ARC: f64 = 1.1;
pub const BANDS: [f64; 3] = [-0.25, 0.0, 0.25];
pub const TUBE_NAME: &str = "3D: a swirl stretched along its axis";
pub const THINNER: &str = "thinner, spinning faster";
pub const TUBE_TAG: &str = "same volume, same circulation";
pub const SWIRLS_TAG: &str = "schematic swirls, not measured flow";
pub const SWIRLS_TAG_AT: (f64, f64) = (0.0, -3.45);
pub const TUBE_TEXT_Y: [f64; 3] = [2.3, -1.55, -2.0];
pub const SWIRL_AT: (f64, f64) = (2.0, 0.3);
pub const SWIRL_R: f64 = 0.85;
pub const DRIFT: f64 = 2.6;
pub const DRIFT_T: f64 = 3.0;
pub const FLAT_NAME: &str = "flat world: no direction to stretch along";
pub const STRETCH_TEX: &str = r"(\omega\cdot\nabla)\,u = \omega_z\,\partial u/\partial z = 0";
pub const READOUT_TEX: &str = r"\omega_z = \omega_0";
pub const KEEPS: &str = "no viscosity, no twisting force";
pub const TWIST_NOTE: &str = "(like chapter 11's spin-up force)";
pub const FLAT_X: f64 = 3.9;
pub const FLAT_TEXT_Y: [f64; 4] = [2.3, -1.5, -2.3, -2.75];
pub const READOUT_DY: f64 = 1.25;

/// 18_6: the slice's lower-left corner (z = 0).
pub const SLICE_AT: (f64, f64) = (-6.6, -1.8);
pub const SLICE_NAME: &str = "a slice";
pub const CROSS_TEX: &str = r"w \neq 0";
pub const ALONG_TEX: &str = r"\partial u/\partial z \neq 0";
pub const REAL_TAG: &str = "real 3D flow: schematic arrows, not measured";
pub const FLAT_RULE_TEX: &str = r"\text{flat equations: } w = 0,\ \partial/\partial z = 0";
/// "a slice" above the box, the two blue readouts in the column right of it, the tags below.
pub const SLICE_TEXT_AT: [(f64, f64); 5] = [(-7.4, 2.3), (-0.5, 0.6), (-0.5, -0.9), (-4.3, -2.85), (-4.3, -3.4)];
/// ST96 §6, verbatim: the paragraph's two sentences, as two parts.
pub const QUOTE_A: [&str; 2] = [
    "Even in the laminar case, the chosen nonstationary 3D",
    "problems showed to be harder than expected.",
];
pub const QUOTE_B: [&str; 4] = [
    "In particular, it was apparently not possible to achieve",
    "reliable reference solutions for the test cases 3D-2Q and",
    "3D-2Z. Hence the benchmark has to be considered as still",
    "open and everybody is invited to try again.",
];
pub const QUOTE_SOURCE: &str = "Schaefer and Turek (1996), §6";
pub const QUOTE_NOTE: &str = "3D-2Q, 3D-2Z: their time-periodic 3D tests (Tables 8, 9)";
pub const QUOTE_CARD_AT: (f64, f64) = (3.95, 0.15);
pub const QUOTE_CARD_SIZE: (f64, f64) = (7.2, 3.8);
pub const QUOTE_TOP: f64 = 1.6;
pub const QUOTE_STEP: f64 = 0.36;

/// 18_7.
pub const BAR_Y: f64 = 0.6;
pub const TICK: f64 = 0.12;
pub const MINOR: f64 = 0.07;
pub const TICK_LABEL_Y: f64 = 0.15;
pub const AXIS_NAME: &str = "Reynolds number, log scale";
pub const AXIS_NAME_AT: (f64, f64) = (-7.4, -0.45);
pub const DOT_LABEL_Y: f64 = 1.05;
pub const CAR_LABEL_RIGHT: f64 = 7.4;
pub const ARC_Y: (f64, f64) = (1.4, 2.5);
pub const RATIO_AT: (f64, f64) = (0.95, 2.85);
pub const FORMULA_CARD_AT: (f64, f64) = (0.0, -2.3);
pub const FORMULA_CARD_SIZE: (f64, f64) = (14.6, 2.8);
pub const FORMULA_Y: [f64; 4] = [-1.4, -2.1, -2.8, -3.25];
pub const AIR_NOTE: &str = "air viscosity: a textbook room-temperature value, not measured here";

/// 18_8.
pub const GATE: &str = "first, the cylinder's test:";
pub const GATE_CHECKS: [&str; 2] = ["convergence: ?", "benchmark: ?"];
pub const GATE_AT: [(f64, f64); 3] = [(-6.6, 2.65), (-3.2, 2.65), (-1.2, 2.65)];
pub const THEN: &str = "then";
pub const THEN_ARROW: [(f64, f64); 2] = [(0.0, -0.55), (0.0, -1.6)];
pub const THEN_AT: (f64, f64) = (0.3, -1.08);

// ------------------------------------------------------------------ style

const WALL_W: f64 = 4.0;
const DIM_W: f64 = 2.0;
const CIRCLE_W: f64 = 2.0;
const CURVE_W: f64 = 3.0;
const INLET_T: f64 = 2.5;
const OUTLINE_W: f64 = 2.5;
const RIDGE_W: f64 = 2.0;
const FLOW_W: f64 = 3.5;
const FLOW_T: f64 = 3.0;
const SPOKE_W: f64 = 3.0;
const TUBE_W: f64 = 3.0;
const AXIS_W: f64 = 2.5;
const TICK_W: f64 = 2.0;
const HEAD: f64 = 0.18;

const Z_FACE: i32 = 1;
const Z_LINE: i32 = 2;
const Z_SOLID: i32 = 3;
const Z_MARK: i32 = 5;
const Z_TEXT: i32 = 8;

// Every duration is a multiple of 1/4 s: exact in binary, whole frames at 30 and 60 fps.
const FADE: f64 = 0.5;
const QUICK: f64 = 0.25;
const MOVE: f64 = 0.75;
const LOOK: f64 = 0.5;
const STEP_T: f64 = 0.25;

/// Labels closer than this (scene units; 24 px at 4K) count as touching.
const LABEL_GAP: f64 = 0.1;

fn solid_grey() -> Srgb {
    hex("#3B4352")
}

fn top_grey() -> Srgb {
    hex("#4A5466")
}

fn end_grey() -> Srgb {
    hex("#323947")
}

fn tyre() -> Srgb {
    hex("#232A36")
}

fn card_fill() -> Srgb {
    hex("#141C29")
}

/// 17_15's colours for convergence and benchmark.
fn check_color(k: usize) -> Srgb {
    [Palette::green(), Palette::blue()][k]
}

// ----------------------------------------------------------------- helpers

fn v(x: f64, y: f64) -> Vec3 {
    [x, y, 0.0]
}

fn vp(p: (f64, f64)) -> Vec3 {
    v(p.0, p.1)
}

/// A point of the channel, metres from the inlet's lower corner (17_8's placing).
pub fn ch_pt(x: f64, y: f64) -> Vec3 {
    v(CH_ORIGIN.0 + CH_SCALE * x, CH_ORIGIN.1 + CH_LIFT + CH_SCALE * y)
}

fn scene_error(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

fn line_style(color: Srgb, width: f64, opacity: f64) -> Style {
    Style::default().fill(color, 0.0).stroke(color, width, opacity)
}

fn fill_style(color: Srgb, opacity: f64) -> Style {
    Style::default().fill(color, opacity).stroke(color, 0.0, opacity)
}

fn solid_style(fill: Srgb) -> Style {
    Style::default().fill(fill, 1.0).stroke(Palette::ink(), OUTLINE_W, 1.0)
}

fn seg(a: Vec3, b: Vec3, style: Style) -> VMobject {
    polyline(vec![a, b]).style(style).build()
}

fn add(stage: &mut Stage<'_>, object: VMobject, z: i32) -> Mob {
    let m = stage.arena_mut().add(object);
    stage.arena_mut().set_z_index(m, z, true);
    m
}

fn group(stage: &mut Stage<'_>, children: Vec<VMobject>, z: i32) -> Mob {
    add(stage, VMobject::new().with_children(children), z)
}

fn text(kit: &Kit, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: (f64, f64)) -> fmn::Result<Mob> {
    let m = kit.text(stage, s, size, color, vp(at))?;
    stage.arena_mut().set_z_index(m, Z_TEXT, true);
    Ok(m)
}

fn text_left(kit: &Kit, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: (f64, f64)) -> fmn::Result<Mob> {
    let m = text(kit, stage, s, size, color, at)?;
    stage.move_to(m, vp(at), LEFT);
    Ok(m)
}

fn text_right(kit: &Kit, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: (f64, f64)) -> fmn::Result<Mob> {
    let m = text(kit, stage, s, size, color, at)?;
    stage.move_to(m, vp(at), RIGHT);
    Ok(m)
}

fn tex(kit: &Kit, stage: &mut Stage<'_>, src: &str, size: f64, color: Srgb, at: (f64, f64)) -> fmn::Result<Mob> {
    let m = kit.tex(stage, src, size, color, &[], vp(at))?;
    stage.arena_mut().set_z_index(m, Z_TEXT, true);
    Ok(m)
}

fn card(stage: &mut Stage<'_>, at: (f64, f64), size: (f64, f64)) -> fmn::Result<Mob> {
    let back = Rectangle::new()
        .width(size.0)
        .height(size.1)
        .corner_radius(0.18)
        .style(
            Style::default()
                .fill(card_fill(), 0.96)
                .stroke(Palette::dim(), 2.5, 1.0),
        )
        .build()?
        .moved_to(vp(at));
    Ok(add(stage, back, Z_TEXT - 1))
}

fn arrow(stage: &mut Stage<'_>, from: Vec3, to: Vec3, color: Srgb, thickness: f64) -> fmn::Result<Mob> {
    let a = Arrow::new(from, to)
        .buff(0.0)
        .thickness(thickness)
        .style(fill_style(color, 1.0))
        .build()?;
    Ok(add(stage, a, Z_MARK))
}

fn dot(stage: &mut Stage<'_>, at: Vec3, r: f64, color: Srgb) -> Mob {
    let d = Dot::new().point(at).radius(r).style(fill_style(color, 1.0)).build();
    add(stage, d, Z_MARK + 1)
}

fn ring(stage: &mut Stage<'_>, at: Vec3, r: f64, color: Srgb, width: f64) -> Mob {
    let c = Circle::new()
        .radius(r)
        .arc_center(at)
        .style(line_style(color, width, 1.0))
        .build();
    add(stage, c, Z_MARK + 1)
}

/// A stroked path ending in an open head ("V") along its last direction, as one point run:
/// the path, then back from the tip along one side of the head, to the tip, along the other.
fn headed(points: &[(f64, f64)], size: f64) -> Vec<Vec3> {
    let n = points.len();
    let (a, b) = (points[n - 2], points[n - 1]);
    let len = (b.0 - a.0).hypot(b.1 - a.1).max(1e-9);
    let (ux, uy) = ((b.0 - a.0) / len, (b.1 - a.1) / len);
    let side = |s: f64| {
        let (c, sn) = (0.45f64.cos(), s * 0.45f64.sin());
        (b.0 - size * (c * ux - sn * uy), b.1 - size * (sn * ux + c * uy))
    };
    let mut out: Vec<Vec3> = points.iter().map(|&p| vp(p)).collect();
    out.extend([vp(side(1.0)), vp(b), vp(side(-1.0))]);
    out
}

fn path_arrow(stage: &mut Stage<'_>, points: &[(f64, f64)], color: Srgb, opacity: f64) -> Mob {
    let line = polyline(headed(points, HEAD))
        .style(line_style(color, FLOW_W, opacity))
        .build();
    add(stage, line, Z_MARK)
}

/// A parabola through three points' x and the middle one's height (`n` spans).
fn bow(a: (f64, f64), peak: (f64, f64), b: (f64, f64), n: usize) -> Vec<(f64, f64)> {
    (0..=n)
        .map(|k| {
            let t = k as f64 / n as f64;
            let base = a.1 + (b.1 - a.1) * t;
            (
                a.0 + (b.0 - a.0) * t,
                base + (peak.1 - (a.1 + b.1) / 2.0) * 4.0 * t * (1.0 - t),
            )
        })
        .collect()
}

/// What is left of a countdown ring, drained clockwise from the top (as 17_h1's).
fn ring_points(c: (f64, f64), r: f64, share: f64) -> Vec<Vec3> {
    let share = share.clamp(0.002, 1.0);
    let steps = ((64.0 * share).ceil() as usize).max(1);
    (0..=steps)
        .map(|k| {
            let a = FRAC_PI_2 + TAU * share * k as f64 / steps as f64;
            v(c.0 + r * a.cos(), c.1 + r * a.sin())
        })
        .collect()
}

/// Each mobject's box: x0, x1, y0, y1.
fn boxes(stage: &Stage<'_>, mobs: &[Mob]) -> Vec<[f64; 4]> {
    mobs.iter()
        .map(|&m| {
            let c = stage.get_center(m);
            let (w, h) = (stage.get_width(m) / 2.0, stage.get_height(m) / 2.0);
            [c[0] - w, c[0] + w, c[1] - h, c[1] + h]
        })
        .collect()
}

/// Refuse the scene if any two of `mobs` come within [`LABEL_GAP`] of each other, or any
/// leaves the frame's 0.25 margin.
fn refuse_touching(stage: &Stage<'_>, mobs: &[Mob], what: &str) -> fmn::Result<()> {
    let b = boxes(stage, mobs);
    for (i, bi) in b.iter().enumerate() {
        if bi[0] < -7.75 || bi[1] > 7.75 || bi[2] < -4.25 || bi[3] > 4.25 {
            return Err(scene_error(format!("{what}: label {i} leaves the frame")));
        }
        for (j, bj) in b.iter().enumerate().skip(i + 1) {
            let apart_x = bi[1] + LABEL_GAP <= bj[0] || bj[1] + LABEL_GAP <= bi[0];
            let apart_y = bi[3] + LABEL_GAP <= bj[2] || bj[3] + LABEL_GAP <= bi[2];
            if !(apart_x || apart_y) {
                return Err(scene_error(format!("{what}: labels {i} and {j} touch")));
            }
        }
    }
    Ok(())
}

// ----------------------------------------------------------------- phrases

/// The sentences of a spoken line: each ends at ".", "?" or "!" before a space or the line's
/// end (the kit's rule; a test compares the counts).
pub fn sentences(line: &str) -> Vec<&str> {
    let bytes = line.as_bytes();
    let (mut out, mut start) = (Vec::new(), 0);
    for (k, &b) in bytes.iter().enumerate() {
        if matches!(b, b'.' | b'?' | b'!') && (k + 1 == bytes.len() || bytes[k + 1] == b' ') {
            out.push(line[start..=k].trim());
            start = k + 1;
        }
    }
    if !line[start..].trim().is_empty() {
        out.push(line[start..].trim());
    }
    out
}

/// Where `phrase` begins in sentence `j` of line `id` (of [`SCRIPT18`]), as a share of that
/// sentence's characters.
pub fn phrase_share(id: &str, j: usize, phrase: &str) -> Option<f64> {
    let line = SCRIPT18
        .lines()
        .find_map(|row| row.strip_prefix(id)?.strip_prefix('\t'))?;
    let sentence = *sentences(line).get(j)?;
    let at = sentence.find(phrase)?;
    Some(sentence[..at].chars().count() as f64 / sentence.chars().count() as f64)
}

/// Wait until `phrase` of sentence `j` of line `id` is about to be said: the sentence's start
/// (the narration's own, from the take's samples once recorded) plus the phrase's share of the
/// sentence's text times the time to the next sentence. The share is an estimate by text
/// length, and the pause after the sentence is counted in, so a picture lands on its words or a
/// little after. `t0` is the scene time the line began, read right after `kit.say`. Without a
/// narrator, or in a line's last sentence, it waits for the sentence's start.
fn until_phrase(kit: &Kit, stage: &mut Stage<'_>, t0: f64, id: &str, j: usize, phrase: &str) -> fmn::Result<()> {
    let share =
        phrase_share(id, j, phrase).ok_or_else(|| scene_error(format!("{id} sentence {j} does not say {phrase:?}")))?;
    let starts = kit.sentence_starts(id);
    let (Some(&a), Some(&b)) = (starts.get(j), starts.get(j + 1)) else {
        return kit.until_sentence(stage, id, j);
    };
    let at = t0 + a + share * (b - a);
    let (now, fps) = (stage.scene().time().to_f64(), stage.scene().time().fps());
    if at > now {
        wait_frames(stage, frames_covering(at - now, fps))?;
    }
    Ok(())
}

fn pause(stage: &mut Stage<'_>, seconds: f64) -> fmn::Result<()> {
    let fps = stage.scene().time().fps();
    wait_frames(stage, frames_covering(seconds, fps))
}

// ----------------------------------------------------------------- the screen

/// Mobjects on screen. Every fade goes through it, so a hide never re-roots a mobject that is
/// already gone (the double-fade flash) and a show never fades in a mobject twice.
#[derive(Default)]
pub struct Screen(Vec<Mob>);

/// One mobject carried from one pose to another (scaled about the first's origin, then
/// shifted), as part of a [`Screen::play`].
pub struct Carry {
    pub mob: Mob,
    pub from: Pose,
    pub to: Pose,
}

impl Screen {
    pub fn on(&self) -> &[Mob] {
        &self.0
    }

    /// In one play: fade `out` away, fade `into` in (sliding by `slide`), draw `drawn` along
    /// its paths and carry each of `carries`; mobjects already in the wanted state are skipped.
    #[allow(clippy::too_many_arguments)]
    fn play(
        &mut self,
        stage: &mut Stage<'_>,
        out: &[Mob],
        into: &[Mob],
        drawn: &[Mob],
        carries: &[Carry],
        slide: Vec3,
        rt: f64,
    ) -> fmn::Result<()> {
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in out {
            if let Some(k) = self.0.iter().position(|&s| s == m) {
                self.0.remove(k);
                stage.clear_updaters(m, true);
                let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
                anims.push(stage.prepare(a.rt(rt))?);
            }
        }
        for &m in into {
            if !self.0.contains(&m) {
                self.0.push(m);
                let a = fade_in(stage.arena_mut(), m, slide, 1.0)?;
                anims.push(stage.prepare(a.rt(rt))?);
            }
        }
        for &m in drawn {
            if !self.0.contains(&m) {
                self.0.push(m);
                anims.push(stage.prepare(show_creation(m).rt(rt))?);
            }
        }
        for c in carries {
            let (f, about, by) = c.from.carry(&c.to);
            if self.0.contains(&c.mob) {
                let a = c
                    .mob
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(rt),
                        ..AnimateArgs::default()
                    })?
                    .scale_about(f, Some(vp(about)), None)?
                    .shift(vp(by))?;
                anims.push(stage.prepare(a)?);
            } else {
                stage.scale_about(c.mob, f, Some(vp(about)), None);
                stage.shift(c.mob, vp(by));
            }
        }
        if !anims.is_empty() {
            stage.play_prepared(anims)?;
        }
        Ok(())
    }

    pub fn swap(&mut self, stage: &mut Stage<'_>, out: &[Mob], into: &[Mob], rt: f64) -> fmn::Result<()> {
        self.play(stage, out, into, &[], &[], ORIGIN, rt)
    }

    pub fn show(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
        self.play(stage, &[], mobs, &[], &[], ORIGIN, rt)
    }

    pub fn hide(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
        self.play(stage, mobs, &[], &[], &[], ORIGIN, rt)
    }

    /// Put `m` on screen at once.
    pub fn put(&mut self, stage: &mut Stage<'_>, m: Mob) -> fmn::Result<()> {
        if !self.0.contains(&m) {
            self.0.push(m);
            stage.add_to_scene(m)?;
        }
        Ok(())
    }

    /// Take `m` off screen at once.
    pub fn take(&mut self, stage: &mut Stage<'_>, m: Mob) {
        if let Some(k) = self.0.iter().position(|&s| s == m) {
            self.0.remove(k);
            stage.clear_updaters(m, true);
            stage.remove_from_scene(m);
        }
    }
}

// ----------------------------------------------------------------- shapes on screen

/// Chapter 17's channel: walls (top, bottom, inlet, outlet) and the cylinder, as 17_7 drew them,
/// at 17_8's placing; then 17_8–17_10's boundary dress and chapter 13's model tag.
fn build_channel(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<(Vec<Mob>, Vec<Mob>)> {
    let (l, h, d) = (channel::LENGTH, channel::HEIGHT, channel::DIAMETER);
    let (cx, cy) = channel::CENTRE;
    let (ink, dim, blue, teal, hero) = (
        Palette::ink(),
        Palette::dim(),
        Palette::blue(),
        Palette::teal(),
        Palette::hero(),
    );
    let walls = group(
        stage,
        vec![
            seg(ch_pt(0.0, 0.0), ch_pt(l, 0.0), line_style(ink, WALL_W, 1.0)),
            seg(ch_pt(0.0, h), ch_pt(l, h), line_style(ink, WALL_W, 1.0)),
            seg(ch_pt(0.0, 0.0), ch_pt(0.0, h), line_style(dim, DIM_W, 1.0)),
            seg(ch_pt(l, 0.0), ch_pt(l, h), line_style(dim, DIM_W, 1.0)),
        ],
        Z_SOLID,
    );
    let cylinder = add(
        stage,
        Circle::new()
            .radius(CH_SCALE * d / 2.0)
            .arc_center(ch_pt(cx, cy))
            .style(Style::default().fill(solid_grey(), 1.0).stroke(ink, CIRCLE_W, 1.0))
            .build(),
        Z_SOLID,
    );
    let mut dress = vec![
        text(kit, stage, NO_SLIP, SMALL_PT, teal, NO_SLIP_AT[0])?,
        text(kit, stage, NO_SLIP, SMALL_PT, teal, NO_SLIP_AT[1])?,
        text_left(kit, stage, NO_SLIP, SMALL_PT, teal, NO_SLIP_AT[2])?,
    ];
    // 17_8's inflow, U(0, y) = 4 Um y (H − y) / H²: arrows at k H / 8, the parabola through
    // their tips.
    let tip = |s: f64| {
        let p = ch_pt(0.0, s * h);
        v(p[0] + INLET_SCALE * 4.0 * channel::PEAK * s * (1.0 - s), p[1])
    };
    for k in 1..=INLET_ARROWS {
        let s = k as f64 / (INLET_ARROWS + 1) as f64;
        dress.push(arrow(stage, ch_pt(0.0, s * h), tip(s), blue, INLET_T)?);
    }
    let curve: Vec<Vec3> = (0..=64).map(|m| tip(m as f64 / 64.0)).collect();
    dress.push(add(
        stage,
        polyline(curve).style(line_style(blue, CURVE_W, 1.0)).build(),
        Z_MARK,
    ));
    dress.push(add(
        stage,
        seg(ch_pt(l, 0.0), ch_pt(l, h), line_style(hero, WALL_W, 1.0)),
        Z_MARK,
    ));
    dress.push(text_right(kit, stage, OUTLET, SMALL_PT, hero, OUTLET_AT)?);
    dress.push(text(kit, stage, MODEL, SMALL_PT, dim, MODEL_AT)?);
    dress.push(text_left(kit, stage, CHANNEL_NAME, FINE_PT, dim, CHANNEL_NAME_AT)?);
    Ok((vec![walls, cylinder], dress))
}

/// Our outline at `pose`: the body, filled like chapter 17's cylinder, and its two wheels.
fn build_outline(stage: &mut Stage<'_>, pose: &Pose) -> fmn::Result<Mob> {
    let body = Polygon::new(car::outline().into_iter().map(|p| vp(pose.at(p))))
        .style(solid_style(solid_grey()))
        .build();
    let mut parts = vec![body];
    for c in car::wheels() {
        parts.push(
            Circle::new()
                .radius(pose.scale * car::wheel_radius())
                .arc_center(vp(pose.at(c)))
                .style(solid_style(tyre()))
                .build(),
        );
    }
    Ok(group(stage, parts, Z_SOLID))
}

/// One wheel's spokes at `pose`.
fn spokes(stage: &mut Stage<'_>, pose: &Pose, c: (f64, f64)) -> Mob {
    let centre = pose.at(c);
    let r = 0.8 * pose.scale * car::wheel_radius();
    let parts = (0..SPOKES)
        .map(|k| {
            let a = FRAC_PI_2 + TAU * k as f64 / SPOKES as f64;
            seg(
                vp(centre),
                v(centre.0 + r * a.cos(), centre.1 + r * a.sin()),
                line_style(Palette::ink(), SPOKE_W, 1.0),
            )
        })
        .collect();
    group(stage, parts, Z_SOLID + 1)
}

/// The outline at depth z (metres) in the cabinet projection: stroke only.
fn outline_copy(stage: &mut Stage<'_>, pose: &Pose, z: f64, opacity: f64) -> Mob {
    let copy = Polygon::new(car::outline().into_iter().map(|(x, y)| vp(pose.at3((x, y, z)))))
        .style(line_style(Palette::ink(), RIDGE_W, opacity))
        .build();
    add(stage, copy, if z < 0.0 { Z_MARK - 1 } else { Z_LINE })
}

/// 18_3's extrusion, step by step outward: at step k the two copies at ±k·STEP and the lines
/// from (k − 1)·STEP to k·STEP through the ridge points, both ways, fading.
fn build_ridge(stage: &mut Stage<'_>, pose: &Pose) -> Vec<Vec<Mob>> {
    let pts = car::ridge_points();
    (1..=ridge::COPIES)
        .map(|k| {
            let (z0, z1) = ((k - 1) as f64 * ridge::STEP, k as f64 * ridge::STEP);
            let mut step = Vec::new();
            for sign in [-1.0, 1.0] {
                step.push(outline_copy(stage, pose, sign * z1, 0.8 * ridge::fade(z1)));
                let style = line_style(Palette::ink(), RIDGE_W, 0.8 * ridge::fade((z0 + z1) / 2.0));
                let lines = pts
                    .iter()
                    .map(|&(x, y)| seg(vp(pose.at3((x, y, sign * z0))), vp(pose.at3((x, y, sign * z1))), style))
                    .collect();
                step.push(group(stage, lines, if sign < 0.0 { Z_MARK - 1 } else { Z_LINE }));
            }
            step
        })
        .collect()
}

/// The ridge's arrows at depth z: over the roof and under the wheels, nose to tail.
fn ridge_arrows(stage: &mut Stage<'_>, pose: &Pose, z: f64, opacity: f64) -> [Mob; 2] {
    let at = |p: (f64, f64)| pose.at3((p.0, p.1, z));
    let over: Vec<(f64, f64)> = bow(OVER_PATH[0], OVER_PATH[1], OVER_PATH[2], 24)
        .into_iter()
        .map(at)
        .collect();
    let under = [at((PATH_X.0, UNDER_Y)), at((PATH_X.1, UNDER_Y))];
    let blue = Palette::blue();
    let arrows = [
        path_arrow(stage, &over, blue, opacity),
        path_arrow(stage, &under, blue, opacity),
    ];
    if z > 0.0 {
        // Behind the slice: under the opaque outline, which hides them where they pass behind it.
        for m in arrows {
            stage.arena_mut().set_z_index(m, Z_LINE, true);
        }
    }
    arrows
}

/// 18_3's real car: boxes of finite width (near side, top and nose end of the body, then of the
/// cabin), its near wheels, and its arrows over, under and around the near side.
fn build_block(stage: &mut Stage<'_>, pose: &Pose) -> (Vec<Mob>, [Mob; 3]) {
    let fills = [solid_grey(), top_grey(), end_grey()];
    let mut parts = Vec::new();
    for (z, b) in [(Z_FACE, block::BODY), (Z_FACE + 1, block::CABIN)] {
        for (face, &fill) in block::faces(b).iter().zip(&fills) {
            let poly = Polygon::new(face.iter().map(|&p| vp(pose.at3(p))))
                .style(solid_style(fill))
                .build();
            parts.push(add(stage, poly, z));
        }
    }
    for c in car::wheels() {
        let w = Circle::new()
            .radius(pose.scale * car::wheel_radius())
            .arc_center(vp(pose.at(c)))
            .style(solid_style(tyre()))
            .build();
        parts.push(add(stage, w, Z_SOLID));
    }
    let blue = Palette::blue();
    let over: Vec<(f64, f64)> = bow(
        (BLOCK_OVER[0].0, BLOCK_OVER[0].1),
        (BLOCK_OVER[1].0, BLOCK_OVER[1].1),
        (BLOCK_OVER[2].0, BLOCK_OVER[2].1),
        24,
    )
    .into_iter()
    .map(|(x, y)| pose.at3((x, y, BLOCK_OVER[0].2)))
    .collect();
    let under = [pose.at3((PATH_X.0, UNDER_Y, 0.0)), pose.at3((PATH_X.1, UNDER_Y, 0.0))];
    let around: Vec<(f64, f64)> = block::around().into_iter().map(|p| pose.at3(p)).collect();
    let arrows = [
        path_arrow(stage, &over, blue, 1.0),
        path_arrow(stage, &under, blue, 1.0),
        path_arrow(stage, &around, blue, 1.0),
    ];
    (parts, arrows)
}

// ----------------------------------------------------------------- the card

/// The 18_h1 prediction card.
pub struct Prediction {
    pub parts: Vec<Mob>,
    pub markers: Vec<Mob>,
    pub chosen: Mob,
    pub ring: Mob,
    pub track: Mob,
    pub note: Mob,
    pub digits: Vec<Mob>,
    ring_at: (f64, f64),
}

impl Prediction {
    pub fn ring_at() -> (f64, f64) {
        let (cx, cy) = Q_CARD_AT;
        (cx + Q_CARD_SIZE.0 / 2.0 - 0.6, cy + Q_CARD_SIZE.1 / 2.0 - 0.85)
    }

    pub fn build(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let (cx, cy) = Q_CARD_AT;
        let (w, h) = Q_CARD_SIZE;
        let (left, right, top) = (cx - w / 2.0, cx + w / 2.0, cy + h / 2.0);
        let (ink, dim) = (Palette::ink(), Palette::dim());
        let mut parts = vec![card(stage, Q_CARD_AT, Q_CARD_SIZE)?];
        parts.push(text_left(kit, stage, Q_LABEL, FINE_PT, dim, (left + 0.35, top - 0.27))?);
        parts.push(text_left(kit, stage, QUESTION, BODY_PT, ink, (left + 0.35, top - 0.7))?);
        let x = left + 0.45;
        let mut markers = Vec::new();
        let mut chosen = None;
        for (k, choice) in CHOICES.iter().enumerate() {
            let y = top - 1.2 - CHOICE_STEP * k as f64;
            parts.push(text_left(kit, stage, choice, SMALL_PT, ink, (x + 0.3, y))?);
            markers.push(ring(stage, v(x, y), 0.1, dim, 2.5));
            if k == ANSWER {
                chosen = Some(dot(stage, v(x, y), 0.1, Palette::green()));
            }
        }
        let note = text_right(kit, stage, PAUSE_NOTE, FINE_PT, dim, (right - 0.3, top - 0.27))?;
        let ring_at = Self::ring_at();
        let track = ring(stage, vp(ring_at), RING_R, Palette::faint(), 6.0);
        let countdown = polyline(ring_points(ring_at, RING_R, 1.0))
            .style(line_style(ink, 7.0, 1.0))
            .build();
        let countdown = add(stage, countdown, Z_TEXT + 1);
        let mut digits = Vec::new();
        for second in 1..=HOLD_SECONDS as u32 {
            digits.push(text(kit, stage, &second.to_string(), BODY_PT, ink, ring_at)?);
        }
        Ok(Self {
            parts,
            markers,
            chosen: chosen.ok_or_else(|| scene_error("ANSWER is not a choice".to_owned()))?,
            ring: countdown,
            track,
            note,
            digits,
            ring_at,
        })
    }
}

// ----------------------------------------------------------------- motion

/// 18_4's motion, one frame at a time: until the car has driven `reach`, the car (outline,
/// spokes and its arrow) moves at `speed` on a still road; after, the car stands and the road's
/// dashes slide back at the same speed. The wheels turn at speed / radius throughout.
struct Drive {
    speed: f64,
    moved: f64,
    reach: f64,
    car: Vec<Mob>,
    spokes: Vec<Mob>,
    hubs: Vec<(f64, f64)>,
    radius: f64,
    dashes: Vec<Mob>,
    dash_x: Vec<f64>,
}

impl Drive {
    fn step(&mut self, s: &mut fmn::mobject::Stage, dt: f64) {
        let car_step = (self.reach - self.moved).clamp(0.0, self.speed * dt);
        if car_step > 0.0 {
            for &m in self.car.iter().chain(&self.spokes) {
                s.shift(m, v(car_step, 0.0));
            }
            for h in &mut self.hubs {
                h.0 += car_step;
            }
            self.moved += car_step;
        }
        let road_step = self.speed * dt - car_step;
        if road_step > 0.0 {
            let span = DASH_PERIOD * self.dashes.len() as f64;
            for (&m, x) in self.dashes.iter().zip(&mut self.dash_x) {
                let mut dx = -road_step;
                if *x + dx + DASH < ROAD_X.0 {
                    dx += span;
                }
                *x += dx;
                s.shift(m, v(dx, 0.0));
            }
        }
        let turn = -sh::wheel_rate(self.speed, self.radius) * dt;
        for (&m, &h) in self.spokes.iter().zip(&self.hubs) {
            s.rotate(m, turn, OUT, Some(vp(h)), None);
        }
    }
}

/// 18_5's motion: the tube's stretch (from `stretch_from`, over [`STRETCH_T`]) and spin, the flat
/// swirl's spin and drift (from `drift_from`, [`DRIFT`] over [`DRIFT_T`]).
struct Spin {
    t: f64,
    /// The flat swirl turns only once it has faded in (a fade would undo its arrows' updates).
    swirl_on: bool,
    stretch_from: Option<f64>,
    drift_from: Option<f64>,
    tube_phase: f64,
    swirl_phase: f64,
    drifted: f64,
    /// Top and bottom lines, the two ends, the bands, then the spin arrows.
    tube: Vec<Mob>,
    /// The circle and the readout move; the arrows are redrawn.
    swirl_moving: Vec<Mob>,
    swirl_arrows: Vec<Mob>,
}

/// The tube's lines at length `l` (before the view's cos 30°) and spin phase.
fn tube_lines(l: f64, phase: f64) -> Vec<Vec<Vec3>> {
    let r = tube::radius(TUBE_R0, TUBE_L0, l);
    let half = l * tube::cos_view() / 2.0;
    let (cx, cy) = TUBE_AT;
    let to3 = |pts: Vec<(f64, f64)>| pts.into_iter().map(vp).collect::<Vec<Vec3>>();
    let mut out = vec![
        vec![v(cx - half, cy + r), v(cx + half, cy + r)],
        vec![v(cx - half, cy - r), v(cx + half, cy - r)],
        to3(tube::section((cx - half, cy), r, 0.0, TAU, 48)),
        to3(tube::section((cx + half, cy), r, 0.0, TAU, 48)),
    ];
    for f in BANDS {
        out.push(to3(tube::section((cx + 2.0 * half * f, cy), r, 0.0, TAU, 48)));
    }
    for k in 0..3 {
        let a = phase + TAU * k as f64 / 3.0;
        out.push(headed(&tube::section((cx + half, cy), r, a, a + SPIN_ARC, 12), HEAD));
    }
    out
}

fn swirl_arrows(centre: (f64, f64), phase: f64) -> Vec<Vec<Vec3>> {
    (0..3)
        .map(|k| {
            let a = phase + TAU * k as f64 / 3.0;
            headed(&tube::circle_arc(centre, SWIRL_R, a, a + SPIN_ARC, 12), HEAD)
        })
        .collect()
}

impl Spin {
    fn length(&self) -> f64 {
        let s = self
            .stretch_from
            .map_or(0.0, |t0| tube::smoothstep((self.t - t0) / STRETCH_T));
        TUBE_L0 * (1.0 + s)
    }

    fn step(&mut self, s: &mut fmn::mobject::Stage, dt: f64) {
        self.t += dt;
        let l = self.length();
        self.tube_phase += tube::spin(SPIN0, TUBE_L0, l) * dt;
        for (&m, pts) in self.tube.iter().zip(tube_lines(l, self.tube_phase)) {
            let _ = s.set_points(m, polyline(pts).build().points());
        }
        if !self.swirl_on {
            return;
        }
        self.swirl_phase += SPIN0 * dt;
        if let Some(t0) = self.drift_from {
            let want = DRIFT * ((self.t - t0) / DRIFT_T).clamp(0.0, 1.0);
            let dx = want - self.drifted;
            if dx > 0.0 {
                for &m in &self.swirl_moving {
                    s.shift(m, v(dx, 0.0));
                }
                self.drifted = want;
            }
        }
        let centre = (SWIRL_AT.0 + self.drifted, SWIRL_AT.1);
        for (&m, pts) in self.swirl_arrows.iter().zip(swirl_arrows(centre, self.swirl_phase)) {
            let _ = s.set_points(m, polyline(pts).build().points());
        }
    }
}

// ----------------------------------------------------------------- the chapter

/// Chapter 18: what is on screen, and every object a later cue changes.
pub struct Car18 {
    pub screen: Screen,
    pub title: Mob,
    channel: Vec<Mob>,
    dress: Vec<Mob>,
    pub outline: Mob,
    pose: Pose,
    driver: Option<Mob>,
}

impl Car18 {
    pub fn build(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let title = text_left(kit, stage, TITLE, TITLE_PT, Palette::ink(), TITLE_AT)?;
        let (channel, dress) = build_channel(kit, stage)?;
        let outline = build_outline(stage, &POSE_A)?;
        Ok(Self {
            screen: Screen::default(),
            title,
            channel,
            dress,
            outline,
            pose: POSE_A,
            driver: None,
        })
    }

    /// Stop the motion, if any, and fade everything but the title and `keep`.
    fn clear(&mut self, stage: &mut Stage<'_>, keep: &[Mob], carry: Option<Pose>) -> fmn::Result<()> {
        if let Some(d) = self.driver.take() {
            self.screen.take(stage, d);
        }
        let out: Vec<Mob> = self
            .screen
            .on()
            .iter()
            .copied()
            .filter(|&m| m != self.title && !keep.contains(&m))
            .collect();
        let carries: Vec<Carry> = carry
            .map(|to| Carry {
                mob: self.outline,
                from: self.pose,
                to,
            })
            .into_iter()
            .collect();
        self.screen.play(stage, &out, &[], &[], &carries, ORIGIN, MOVE)?;
        if let Some(to) = carry {
            self.pose = to;
        }
        Ok(())
    }

    /// An empty mobject in the scene whose updater runs `step` every frame.
    fn drive<S: 'static>(
        &mut self,
        stage: &mut Stage<'_>,
        state: Rc<RefCell<S>>,
        step: fn(&mut S, &mut fmn::mobject::Stage, f64),
    ) -> fmn::Result<()> {
        let driver = stage.arena_mut().add(VMobject::new());
        self.screen.put(stage, driver)?;
        stage.add_dt_updater(
            driver,
            move |s, _, dt| {
                if dt > 0.0 {
                    step(&mut state.borrow_mut(), s, dt);
                }
            },
            false,
        )?;
        self.driver = Some(driver);
        Ok(())
    }

    // ---------------------------------------------------------------- 18_1

    pub fn cue_18_1(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "18_1")?;
        self.screen.show(stage, &[self.title], FADE)?;
        let shown = [self.channel.clone(), self.dress.clone()].concat();
        self.screen.show(stage, &shown, FADE)?;
        let note = text(kit, stage, SHAPE_NOTE, SMALL_PT, Palette::dim(), NOTE_AT)?;
        self.screen.show(stage, &[self.outline, note], FADE)?;
        kit.until_sentence(stage, "18_1", 1)?;
        let drag = text_left(kit, stage, DRAG, SMALL_PT, Palette::ink(), DRAG_AT)?;
        self.screen.show(stage, &[drag], FADE)
    }

    // ---------------------------------------------------------------- 18_2

    pub fn cue_18_2(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Prediction> {
        kit.say(stage, "18_2")?;
        let t0 = stage.scene().time().to_f64();
        self.clear(stage, &[self.outline], Some(POSE_B))?;
        until_phrase(kit, stage, t0, "18_2", 0, "the direction we left out")?;
        let (x, y, z) = AXIS_AT;
        let mid = vp(POSE_B.at3(AXIS_AT));
        let dim = Palette::dim();
        let axis = [
            arrow(stage, mid, vp(POSE_B.at3((x, y, z - AXIS_REACH))), dim, 2.0)?,
            arrow(stage, mid, vp(POSE_B.at3((x, y, z + AXIS_REACH))), dim, 2.0)?,
            text(kit, stage, LEFT_OUT, SMALL_PT, dim, LEFT_OUT_AT)?,
        ];
        self.screen.show(stage, &axis, FADE)?;
        kit.until_sentence(stage, "18_2", 1)?;
        let card = Prediction::build(kit, stage)?;
        let shown = [card.parts.clone(), card.markers.clone(), vec![card.track, card.note]].concat();
        self.screen.show(stage, &shown, FADE)?;
        Ok(card)
    }

    // ---------------------------------------------------------------- 18_h1

    /// The question and the outline stay; the countdown runs 5 s on the ring and digits;
    /// nothing else moves, and the extrusion does not exist yet.
    pub fn hold_18_h1(&mut self, kit: &Kit, stage: &mut Stage<'_>, card: &Prediction) -> fmn::Result<()> {
        kit.hold(stage)?;
        kit.mark(stage, "predict");
        self.screen.put(stage, card.ring)?;
        let fps = stage.scene().time().fps();
        let frames = frames_covering(HOLD_SECONDS, fps);
        let mut showing: Option<Mob> = None;
        for k in 0..frames {
            let left = HOLD_SECONDS * (frames - k) as f64 / frames as f64;
            let digit = card.digits[(left.ceil() as usize).clamp(1, card.digits.len()) - 1];
            if showing != Some(digit) {
                if let Some(old) = showing {
                    self.screen.take(stage, old);
                }
                self.screen.put(stage, digit)?;
                showing = Some(digit);
            }
            let _ = stage.set_points(
                card.ring,
                polyline(ring_points(card.ring_at, RING_R, left / HOLD_SECONDS))
                    .build()
                    .points(),
            );
            wait_frames(stage, 1)?;
        }
        if let Some(old) = showing {
            self.screen.take(stage, old);
        }
        self.screen.take(stage, card.ring);
        self.screen.hide(stage, &[card.track, card.note], QUICK)
    }

    // ---------------------------------------------------------------- 18_3

    pub fn cue_18_3(&mut self, kit: &Kit, stage: &mut Stage<'_>, card: Prediction) -> fmn::Result<()> {
        kit.say(stage, "18_3")?;
        let t0 = stage.scene().time().to_f64();
        self.screen.show(stage, &[card.chosen], QUICK)?;
        pause(stage, LOOK)?;
        // The answer: the outline extrudes both ways, step by step, fading toward the ends.
        let steps = build_ridge(stage, &POSE_B);
        let name = text(kit, stage, RIDGE_NAME, BODY_PT, Palette::ink(), RIDGE_NAME_AT)?;
        let out: Vec<Mob> = self
            .screen
            .on()
            .iter()
            .copied()
            .filter(|&m| m != self.title && m != self.outline)
            .collect();
        self.screen.swap(stage, &out, &steps[0], STEP_T)?;
        for step in &steps[1..] {
            self.screen.show(stage, step, STEP_T)?;
        }
        self.screen.show(stage, &[name], FADE)?;
        kit.until_sentence(stage, "18_3", 1)?;
        // Over and under, at the slice and the same one depth behind it.
        let mut arrows = ridge_arrows(stage, &POSE_B, 0.0, 1.0).to_vec();
        for z in ECHO_Z {
            arrows.extend(ridge_arrows(stage, &POSE_B, z, 0.35));
        }
        self.screen.show(stage, &arrows, FADE)?;
        until_phrase(kit, stage, t0, "18_3", 1, "never around its sides")?;
        let model = tex(kit, stage, MODEL_TEX, SMALL_PT, Palette::dim(), MODEL_TEX_AT)?;
        self.screen.show(stage, &[model], FADE)?;
        kit.until_sentence(stage, "18_3", 2)?;
        // A real car has width: over, under and around its sides.
        let (parts, around) = build_block(stage, &BLOCK_POSE);
        let ink = Palette::ink();
        let block_labels = vec![
            text(kit, stage, OVER, SMALL_PT, ink, LABEL_AT[0])?,
            text(kit, stage, UNDER, SMALL_PT, ink, LABEL_AT[1])?,
            text(kit, stage, AROUND, SMALL_PT, ink, LABEL_AT[2])?,
            text(kit, stage, BLOCK_NAME, BODY_PT, ink, BLOCK_NAME_AT)?,
            text(kit, stage, SCHEMATIC, FINE_PT, Palette::dim(), SCHEMATIC_AT)?,
        ];
        let all_labels = [vec![name, model], block_labels.clone()].concat();
        refuse_touching(stage, &all_labels, "18_3")?;
        self.screen
            .show(stage, &[parts, around.to_vec(), block_labels].concat(), FADE)
    }

    // ---------------------------------------------------------------- 18_4

    pub fn cue_18_4(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "18_4")?;
        let t0 = stage.scene().time().to_f64();
        let start = Pose {
            origin: (POSE_C.origin.0 - DRIVE, POSE_C.origin.1),
            scale: POSE_C.scale,
        };
        self.clear(stage, &[self.outline], Some(start))?;
        // The road (still), the wheels' spokes, and the ground frame's names.
        let y = POSE_C.origin.1;
        let n = ((ROAD_X.1 - ROAD_X.0) / DASH_PERIOD).ceil() as usize + 1;
        let dash_x: Vec<f64> = (0..n).map(|k| ROAD_X.0 + DASH_PERIOD * k as f64).collect();
        let road = Palette::dim();
        let dashes: Vec<Mob> = dash_x
            .iter()
            .map(|&x| {
                add(
                    stage,
                    seg(v(x, y), v(x + DASH, y), line_style(road, WALL_W, 1.0)),
                    Z_LINE,
                )
            })
            .collect();
        let wheels = car::wheels();
        let spokes: Vec<Mob> = wheels.iter().map(|&c| spokes(stage, &start, c)).collect();
        let (ink, blue, hero) = (Palette::ink(), Palette::blue(), Palette::hero());
        let car_label = sh::frame_label("car", false).ok_or_else(|| scene_error("no car".to_owned()))?;
        let car_arrow = arrow(
            stage,
            vp(start.at(CAR_ARROW[0])),
            vp(start.at(CAR_ARROW[1])),
            ink,
            FLOW_T,
        )?;
        let car_text = text(kit, stage, &car_label, SMALL_PT, ink, start.at(CAR_ARROW_LABEL))?;
        let ground = text_left(kit, stage, GROUND_NAME, BODY_PT, Palette::dim(), FRAME_NAME_AT)?;
        let shown = [dashes.clone(), spokes.clone(), vec![car_arrow, car_text, ground]].concat();
        self.screen.show(stage, &shown, FADE)?;
        // It rolls in, arriving as the car's frame begins.
        let starts = kit.sentence_starts("18_4");
        let now = stage.scene().time().to_f64();
        let left = starts.get(1).map_or(2.0, |&s| t0 + s - now);
        let speed = DRIVE / left.max(DRIVE / SPEED_MAX);
        let state = Rc::new(RefCell::new(Drive {
            speed,
            moved: 0.0,
            reach: DRIVE,
            car: vec![self.outline, car_arrow, car_text],
            spokes,
            hubs: wheels.iter().map(|&c| start.at(c)).collect(),
            radius: start.scale * car::wheel_radius(),
            dashes,
            dash_x,
        }));
        self.drive(stage, state, Drive::step)?;
        self.pose = POSE_C;
        kit.until_sentence(stage, "18_4", 1)?;
        // The car has arrived (one frame's margin) before its arrow and label fade.
        let fps = stage.scene().time().fps();
        let driven = stage.scene().time().to_f64() - now;
        pause(stage, (DRIVE / speed - driven).max(0.0) + 1.0 / f64::from(fps))?;
        // Seen from the car: subtract its velocity from everything.
        let road_label = sh::frame_label("road", true).ok_or_else(|| scene_error("no road".to_owned()))?;
        let air_label = sh::frame_label("air", true).ok_or_else(|| scene_error("no air".to_owned()))?;
        let mut into = vec![
            text_left(kit, stage, CAR_NAME, BODY_PT, Palette::dim(), FRAME_NAME_AT)?,
            text(kit, stage, &road_label, SMALL_PT, hero, (ROAD_ROW_X[2], ROAD_ROW_Y))?,
            text(kit, stage, &air_label, SMALL_PT, blue, AIR_LABEL_AT)?,
            text(kit, stage, FRAME_TAG, FINE_PT, Palette::dim(), FRAME_TAG_AT)?,
        ];
        for &h in &AIR_Y {
            into.push(arrow(
                stage,
                vp(POSE_C.at((AIR_X.0, h))),
                vp(POSE_C.at((AIR_X.1, h))),
                blue,
                FLOW_T,
            )?);
        }
        self.screen.swap(stage, &[car_arrow, car_text, ground], &into, FADE)?;
        kit.until_sentence(stage, "18_4", 2)?;
        let questions = vec![
            text(kit, stage, GROUND_Q, SMALL_PT, hero, (ROAD_ROW_X[0], ROAD_ROW_Y))?,
            text(kit, stage, WHEELS_Q, SMALL_PT, hero, (ROAD_ROW_X[1], ROAD_ROW_Y))?,
        ];
        let row = [questions.clone(), vec![into[1], into[2], into[3]]].concat();
        refuse_touching(stage, &row, "18_4 road row")?;
        self.screen.show(stage, &questions, FADE)
    }

    // ---------------------------------------------------------------- 18_5

    pub fn cue_18_5(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "18_5")?;
        let t0 = stage.scene().time().to_f64();
        self.clear(stage, &[], None)?;
        // A schematic tube, spinning; then stretched to twice its length.
        let (ink, dim, blue) = (Palette::ink(), Palette::dim(), Palette::blue());
        let mut tube = Vec::new();
        for (k, pts) in tube_lines(TUBE_L0, 0.0).into_iter().enumerate() {
            let (color, width, opacity) = match k {
                0..=3 => (ink, TUBE_W, 1.0),
                4..=6 => (ink, TUBE_W * 0.6, 0.35),
                _ => (blue, FLOW_W, 1.0),
            };
            tube.push(add(
                stage,
                polyline(pts).style(line_style(color, width, opacity)).build(),
                Z_MARK,
            ));
        }
        let (cx, _) = TUBE_AT;
        let tube_name = text(kit, stage, TUBE_NAME, SMALL_PT, ink, (cx, TUBE_TEXT_Y[0]))?;
        let tube_tag = text(kit, stage, TUBE_TAG, FINE_PT, dim, (cx, TUBE_TEXT_Y[2]))?;
        let swirls_tag = text(kit, stage, SWIRLS_TAG, FINE_PT, dim, SWIRLS_TAG_AT)?;
        let shown = [tube.clone(), vec![tube_name, tube_tag, swirls_tag]].concat();
        self.screen.show(stage, &shown, FADE)?;
        let circle = add(
            stage,
            Circle::new()
                .radius(SWIRL_R)
                .arc_center(vp(SWIRL_AT))
                .style(line_style(ink, TUBE_W, 1.0))
                .build(),
            Z_MARK,
        );
        let readout = tex(
            kit,
            stage,
            READOUT_TEX,
            SMALL_PT,
            ink,
            (SWIRL_AT.0, SWIRL_AT.1 + READOUT_DY),
        )?;
        let arrows: Vec<Mob> = swirl_arrows(SWIRL_AT, 0.0)
            .into_iter()
            .map(|pts| {
                add(
                    stage,
                    polyline(pts).style(line_style(blue, FLOW_W, 1.0)).build(),
                    Z_MARK,
                )
            })
            .collect();
        let state = Rc::new(RefCell::new(Spin {
            t: 0.0,
            swirl_on: false,
            stretch_from: None,
            drift_from: None,
            tube_phase: 0.0,
            swirl_phase: 0.0,
            drifted: 0.0,
            tube,
            swirl_moving: vec![circle, readout],
            swirl_arrows: arrows.clone(),
        }));
        self.drive(stage, Rc::clone(&state), Spin::step)?;
        until_phrase(kit, stage, t0, "18_5", 1, "can be stretched along its own axis")?;
        let now = state.borrow().t;
        state.borrow_mut().stretch_from = Some(now);
        until_phrase(kit, stage, t0, "18_5", 1, "making it thinner and spinning it faster")?;
        let thinner = text(kit, stage, THINNER, SMALL_PT, ink, (cx, TUBE_TEXT_Y[1]))?;
        self.screen.show(stage, &[thinner], FADE)?;
        // Flat: nothing to stretch along.
        kit.until_sentence(stage, "18_5", 2)?;
        let flat = vec![
            text(kit, stage, FLAT_NAME, SMALL_PT, ink, (FLAT_X, FLAT_TEXT_Y[0]))?,
            tex(kit, stage, STRETCH_TEX, SMALL_PT, ink, (FLAT_X, FLAT_TEXT_Y[1]))?,
        ];
        self.screen
            .show(stage, &[vec![circle, readout], arrows, flat.clone()].concat(), FADE)?;
        state.borrow_mut().swirl_on = true;
        kit.until_sentence(stage, "18_5", 3)?;
        let keeps = vec![
            text(kit, stage, KEEPS, SMALL_PT, Palette::hero(), (FLAT_X, FLAT_TEXT_Y[2]))?,
            text(kit, stage, TWIST_NOTE, FINE_PT, dim, (FLAT_X, FLAT_TEXT_Y[3]))?,
        ];
        self.screen.show(stage, &keeps, FADE)?;
        let now = state.borrow().t;
        state.borrow_mut().drift_from = Some(now);
        let labels = [vec![tube_name, tube_tag, swirls_tag, thinner], flat, keeps].concat();
        refuse_touching(stage, &labels, "18_5")
    }

    // ---------------------------------------------------------------- 18_6

    pub fn cue_18_6(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "18_6")?;
        let t0 = stage.scene().time().to_f64();
        self.clear(stage, &[], None)?;
        let at = |p: slice::P3| {
            let (dx, dy) = sh::oblique(p.2);
            v(SLICE_AT.0 + p.0 + dx, SLICE_AT.1 + p.1 + dy)
        };
        let (teal, faint, dim, blue, ink) = (
            Palette::teal(),
            Palette::faint(),
            Palette::dim(),
            Palette::blue(),
            Palette::ink(),
        );
        let (w, h, d) = (slice::W, slice::H, slice::DEPTH);
        let corners = |z: f64| [at((0.0, 0.0, z)), at((w, 0.0, z)), at((w, h, z)), at((0.0, h, z))];
        let mut region = Vec::new();
        for z in [-d, d] {
            region.push(add(
                stage,
                Polygon::new(corners(z)).style(line_style(faint, RIDGE_W, 1.0)).build(),
                Z_LINE,
            ));
        }
        let edges = (0..4)
            .map(|k| seg(corners(-d)[k], corners(d)[k], line_style(faint, RIDGE_W, 1.0)))
            .collect();
        region.push(group(stage, edges, Z_LINE));
        let plane = Polygon::new(corners(0.0))
            .style(Style::default().fill(teal, 0.12).stroke(teal, RIDGE_W, 1.0))
            .build();
        region.push(add(stage, plane, Z_LINE + 1));
        let names = [
            text_left(kit, stage, SLICE_NAME, SMALL_PT, teal, SLICE_TEXT_AT[0])?,
            text(kit, stage, REAL_TAG, FINE_PT, dim, SLICE_TEXT_AT[3])?,
        ];
        self.screen.show(stage, &[region, names.to_vec()].concat(), FADE)?;
        // Air crossing the slice: solid in front of it, faint behind, a dot where it pierces.
        kit.until_sentence(stage, "18_6", 1)?;
        let xy = |p: Vec3| (p[0], p[1]);
        let mut crossing = Vec::new();
        for &(a, b) in &slice::CROSSING {
            let p = slice::pierce(a, b).ok_or_else(|| scene_error("an arrow misses the slice".to_owned()))?;
            crossing.push(add(stage, seg(at(a), at(p), line_style(blue, FLOW_W, 1.0)), Z_MARK));
            crossing.push(path_arrow(stage, &[xy(at(p)), xy(at(b))], blue, 0.45));
            crossing.push(dot(stage, at(p), 0.07, teal));
        }
        let cross_tex = tex(kit, stage, CROSS_TEX, SMALL_PT, blue, SLICE_TEXT_AT[1])?;
        self.screen.show(stage, &[crossing, vec![cross_tex]].concat(), FADE)?;
        until_phrase(kit, stage, t0, "18_6", 1, "changing along it")?;
        let along: Vec<Mob> = slice::ALONG
            .iter()
            .map(|&(a, b)| path_arrow(stage, &[xy(at(a)), xy(at(b))], blue, if a.2 < 0.0 { 1.0 } else { 0.45 }))
            .collect();
        let along_tex = tex(kit, stage, ALONG_TEX, SMALL_PT, blue, SLICE_TEXT_AT[2])?;
        self.screen.show(stage, &[along, vec![along_tex]].concat(), FADE)?;
        until_phrase(kit, stage, t0, "18_6", 1, "which the flat equations rule out")?;
        let rule = tex(kit, stage, FLAT_RULE_TEX, SMALL_PT, dim, SLICE_TEXT_AT[4])?;
        self.screen.show(stage, &[rule], FADE)?;
        // The organizers' own words.
        kit.until_sentence(stage, "18_6", 2)?;
        let (qx, _) = QUOTE_CARD_AT;
        let left = qx - QUOTE_CARD_SIZE.0 / 2.0 + 0.35;
        let mut part_a = vec![card(stage, QUOTE_CARD_AT, QUOTE_CARD_SIZE)?];
        for (k, line) in QUOTE_A.iter().enumerate() {
            part_a.push(text_left(
                kit,
                stage,
                line,
                FINE_PT,
                ink,
                (left, QUOTE_TOP - QUOTE_STEP * k as f64),
            )?);
        }
        let below_a = QUOTE_TOP - QUOTE_STEP * (QUOTE_A.len() as f64 + 0.5);
        part_a.push(text_left(
            kit,
            stage,
            QUOTE_SOURCE,
            FINE_PT,
            dim,
            (left, below_a - QUOTE_STEP * 4.5),
        )?);
        self.screen.show(stage, &part_a, FADE)?;
        kit.until_sentence(stage, "18_6", 3)?;
        let mut part_b = Vec::new();
        for (k, line) in QUOTE_B.iter().enumerate() {
            part_b.push(text_left(
                kit,
                stage,
                line,
                FINE_PT,
                ink,
                (left, below_a - QUOTE_STEP * k as f64),
            )?);
        }
        part_b.push(text_left(
            kit,
            stage,
            QUOTE_NOTE,
            FINE_PT,
            dim,
            (left, below_a - QUOTE_STEP * 5.5),
        )?);
        self.screen.show(stage, &part_b, FADE)?;
        let labels = [names.to_vec(), vec![cross_tex, along_tex, rule]].concat();
        refuse_touching(stage, &labels, "18_6")?;
        refuse_touching(stage, &[part_a[1..].to_vec(), part_b].concat(), "18_6 quote")
    }

    // ---------------------------------------------------------------- 18_7

    pub fn cue_18_7(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "18_7")?;
        let t0 = stage.scene().time().to_f64();
        self.clear(stage, &[], None)?;
        let (ink, dim, faint, blue, hero) = (
            Palette::ink(),
            Palette::dim(),
            Palette::faint(),
            Palette::blue(),
            Palette::hero(),
        );
        let x = log_bar::x;
        let mut bar = vec![add(
            stage,
            seg(
                v(log_bar::LEFT, BAR_Y),
                v(log_bar::right(), BAR_Y),
                line_style(ink, AXIS_W, 1.0),
            ),
            Z_LINE,
        )];
        let mut ticks = Vec::new();
        let mut minor = Vec::new();
        for k in log_bar::FIRST..=log_bar::LAST {
            let at = x(10f64.powi(k));
            ticks.push(seg(
                v(at, BAR_Y - TICK),
                v(at, BAR_Y + TICK),
                line_style(ink, TICK_W, 1.0),
            ));
            bar.push(tex(kit, stage, &sh::decade_tex(k), SMALL_PT, dim, (at, TICK_LABEL_Y))?);
            if k < log_bar::LAST {
                for m in 2..10 {
                    let at = x(f64::from(m) * 10f64.powi(k));
                    minor.push(seg(
                        v(at, BAR_Y - MINOR),
                        v(at, BAR_Y + MINOR),
                        line_style(faint, TICK_W, 1.0),
                    ));
                }
            }
        }
        bar.push(group(stage, ticks, Z_LINE));
        bar.push(group(stage, minor, Z_LINE));
        bar.push(text_left(kit, stage, AXIS_NAME, FINE_PT, dim, AXIS_NAME_AT)?);
        self.screen.show(stage, &bar, FADE)?;
        // The test, as chapter 17 computed it.
        kit.until_sentence(stage, "18_7", 1)?;
        let test_re = sh::test_flow::reynolds();
        let test_x = x(test_re.0 as f64 / test_re.1 as f64);
        let test = vec![
            dot(stage, v(test_x, BAR_Y), 0.1, blue),
            text(kit, stage, &sh::test_label(), SMALL_PT, blue, (test_x, DOT_LABEL_Y))?,
            card(stage, FORMULA_CARD_AT, FORMULA_CARD_SIZE)?,
            tex(kit, stage, &sh::test_re_tex(), SMALL_PT, blue, (0.0, FORMULA_Y[0]))?,
        ];
        self.screen.show(stage, &test, FADE)?;
        // The car: its numbers as they are said, then its place, then the ratio.
        kit.until_sentence(stage, "18_7", 2)?;
        let car_formula = vec![
            tex(kit, stage, &sh::car_re_tex(), SMALL_PT, hero, (0.0, FORMULA_Y[1]))?,
            text(kit, stage, AIR_NOTE, FINE_PT, dim, (0.0, FORMULA_Y[2]))?,
            text(kit, stage, &sh::mph_label(), FINE_PT, dim, (0.0, FORMULA_Y[3]))?,
        ];
        self.screen.show(stage, &car_formula, FADE)?;
        until_phrase(kit, stage, t0, "18_7", 2, "is near nine million")?;
        let car_re = sh::car_flow::reynolds();
        let car_x = x(car_re.0 as f64 / car_re.1 as f64);
        let car_mark = vec![
            dot(stage, v(car_x, BAR_Y), 0.1, hero),
            text_right(
                kit,
                stage,
                &sh::car_label(),
                SMALL_PT,
                hero,
                (CAR_LABEL_RIGHT, DOT_LABEL_Y),
            )?,
        ];
        self.screen.show(stage, &car_mark, FADE)?;
        until_phrase(kit, stage, t0, "18_7", 3, "about ninety thousand times")?;
        let arc = bow(
            (test_x, ARC_Y.0),
            ((test_x + car_x) / 2.0, ARC_Y.1),
            (car_x, ARC_Y.0),
            32,
        );
        let ratio = vec![
            path_arrow(stage, &arc, hero, 1.0),
            text(kit, stage, &sh::ratio_label(), BODY_PT, hero, RATIO_AT)?,
        ];
        self.screen.show(stage, &ratio, FADE)?;
        refuse_touching(stage, &[test[1], car_mark[1], ratio[1], bar[bar.len() - 1]], "18_7")
    }

    // ---------------------------------------------------------------- 18_8

    pub fn cue_18_8(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "18_8")?;
        let t0 = stage.scene().time().to_f64();
        self.clear(stage, &[], Some(POSE_A))?;
        // The cylinder's test first: still open; then the car shape, labelled.
        let gate = vec![
            text_left(kit, stage, GATE, SMALL_PT, Palette::ink(), GATE_AT[0])?,
            text_left(kit, stage, GATE_CHECKS[0], SMALL_PT, check_color(0), GATE_AT[1])?,
            text_left(kit, stage, GATE_CHECKS[1], SMALL_PT, check_color(1), GATE_AT[2])?,
        ];
        let then = vec![
            arrow(stage, vp(THEN_ARROW[0]), vp(THEN_ARROW[1]), Palette::dim(), 2.5)?,
            text_left(kit, stage, THEN, SMALL_PT, Palette::dim(), THEN_AT)?,
        ];
        let shown = [self.channel.clone(), gate.clone()].concat();
        self.screen.show(stage, &shown, FADE)?;
        self.screen.show(stage, &[then, vec![self.outline]].concat(), FADE)?;
        until_phrase(kit, stage, t0, "18_8", 0, "it's labeled as a flat illustration")?;
        let permanent = text(kit, stage, PERMANENT, SMALL_PT, Palette::ink(), NOTE_AT)?;
        self.screen.show(stage, &[permanent], FADE)?;
        kit.until_sentence(stage, "18_8", 1)?;
        let not_read = text_left(kit, stage, NOT_READ, SMALL_PT, Palette::dim(), DRAG_AT)?;
        self.screen.show(stage, &[not_read], FADE)?;
        refuse_touching(stage, &[gate, vec![permanent, not_read]].concat(), "18_8")
    }
}

// --------------------------------------------------------------- the cues

/// Cues 18_1 to 18_8 and the 18_h1 hold, in order, from a clear screen. Returns the chapter
/// with 18_8's picture on screen.
pub fn chapter18(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Car18> {
    let mut ch = Car18::build(kit, stage)?;
    ch.cue_18_1(kit, stage)?;
    let card = ch.cue_18_2(kit, stage)?;
    ch.hold_18_h1(kit, stage, &card)?;
    ch.cue_18_3(kit, stage, card)?;
    ch.cue_18_4(kit, stage)?;
    ch.cue_18_5(kit, stage)?;
    ch.cue_18_6(kit, stage)?;
    ch.cue_18_7(kit, stage)?;
    ch.cue_18_8(kit, stage)?;
    Ok(ch)
}

/// The chapter as root's crate registers its chapters (`story.rs`: `make: |kit|
/// Box::new(Chapter18 { kit })`).
pub struct Chapter18 {
    pub kit: Kit,
}

impl SceneConstruct for Chapter18 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        chapter18(&self.kit, stage)?;
        self.kit.finish(stage)
    }
}

/// Every string the chapter sets as text, and every TeX source, for a glyph probe.
#[cfg(test)]
pub fn glyph_strings() -> (Vec<String>, Vec<String>) {
    let mut text: Vec<String> = [
        TITLE,
        NO_SLIP,
        OUTLET,
        MODEL,
        CHANNEL_NAME,
        SHAPE_NOTE,
        PERMANENT,
        FRAME_TAG,
        SWIRLS_TAG,
        DRAG,
        NOT_READ,
        LEFT_OUT,
        Q_LABEL,
        QUESTION,
        PAUSE_NOTE,
        RIDGE_NAME,
        OVER,
        UNDER,
        AROUND,
        BLOCK_NAME,
        SCHEMATIC,
        GROUND_NAME,
        CAR_NAME,
        GROUND_Q,
        WHEELS_Q,
        TUBE_NAME,
        THINNER,
        TUBE_TAG,
        FLAT_NAME,
        KEEPS,
        TWIST_NOTE,
        SLICE_NAME,
        REAL_TAG,
        QUOTE_SOURCE,
        QUOTE_NOTE,
        AXIS_NAME,
        AIR_NOTE,
        GATE,
        THEN,
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    text.extend(
        CHOICES
            .iter()
            .chain(&QUOTE_A)
            .chain(&QUOTE_B)
            .chain(&GATE_CHECKS)
            .map(|s| s.to_string()),
    );
    text.extend((1..=HOLD_SECONDS as u32).map(|s| s.to_string()));
    text.extend(
        ["car", "air", "road"]
            .iter()
            .filter_map(|n| sh::frame_label(n, *n != "car")),
    );
    text.extend([sh::test_label(), sh::car_label(), sh::ratio_label(), sh::mph_label()]);
    let mut tex: Vec<String> = [MODEL_TEX, STRETCH_TEX, READOUT_TEX, CROSS_TEX, ALONG_TEX, FLAT_RULE_TEX]
        .iter()
        .map(|s| s.to_string())
        .collect();
    tex.extend((log_bar::FIRST..=log_bar::LAST).map(sh::decade_tex));
    tex.extend([sh::test_re_tex(), sh::car_re_tex()]);
    (text, tex)
}

/// The spoken lines, verbatim from `navier-full-script03.md` chapter 18 (`tools/check_car01.py`
/// compares them byte for byte), as `script.tsv` rows. 18_h1 is a hold, not a line.
pub const SCRIPT18: &str = concat!(
    "18_1\tA cylinder is a long way from a car. So why not put a car shape in our tunnel and read off its drag?\n",
    "18_2\tRemember that our world is flat, and nothing in it changes in the direction we left out. So what does a flat car really stand for?\n",
    "18_3\tA car-shaped wall that never ends. Air can only go over it or under it, never around its sides. Around a real car, air does all three.\n",
    "18_4\tUnderneath, a real car rolls along a road. Seen from the car, the road slides backward at the car's own speed, and the wheels turn. A flat picture has to make choices about both.\n",
    "18_5\tThen there's the spin. In three dimensions, a swirl can be stretched along its own axis, making it thinner and spinning it faster. In our flat world there's no direction to stretch it along. So without viscosity, or an added force that twists it, each parcel just keeps its spin as it's carried.\n",
    "18_6\tSo a flat answer isn't simply a slice of a real one. A slice through a real flow can have air crossing it and changing along it, which the flat equations rule out. Even the nineteen ninety-six organizers found their changing three-dimensional tests harder than expected. For two of them, they couldn't settle reliable answers, and they left the test open.\n",
    "18_7\tAnd the numbers change scale. Our test runs at a Reynolds number of one hundred. A real car, four and a half meters long, at thirty meters per second in air, is near nine million. That's about ninety thousand times the Reynolds number in our test.\n",
    "18_8\tSo if a car shape appears in our tunnel, it comes only after the cylinder passes its test, and it's labeled as a flat illustration. We won't read a real car's drag or downforce from it.\n",
);

#[cfg(test)]
mod tests {
    use super::*;
    use explainer_kit::{Narrator, Script};

    fn kit() -> Kit {
        let script = Script::parse(SCRIPT18).unwrap();
        Kit::new().unwrap().with_narrator(Narrator::estimate(&script))
    }

    #[test]
    fn script_has_the_eight_lines_in_order() {
        let script = Script::parse(SCRIPT18).unwrap();
        let ids: Vec<&str> = script.lines().iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(ids, ["18_1", "18_2", "18_3", "18_4", "18_5", "18_6", "18_7", "18_8"]);
        let kit = kit();
        let counts: Vec<usize> = ids.iter().map(|id| kit.sentence_starts(id).len()).collect();
        assert_eq!(counts, [2, 2, 3, 3, 4, 4, 4, 2]);
        for row in SCRIPT18.lines() {
            let (id, line) = row.split_once('\t').unwrap();
            assert_eq!(sentences(line).len(), kit.sentence_starts(id).len(), "{id}");
        }
    }

    #[test]
    fn phrases_sit_in_their_sentences() {
        let src = include_str!("car18_01.rs");
        let mut gates = 0;
        for call in src.split("until_phrase(kit, stage, t0, \"").skip(1) {
            let (id, rest) = call.split_once('"').unwrap();
            let (j, arg) = rest[2..].split_once(", ").unwrap();
            let j: usize = j.parse().unwrap();
            let phrase = arg.strip_prefix('"').unwrap().split('"').next().unwrap();
            assert!(phrase_share(id, j, phrase).is_some(), "{id} {j} {phrase}");
            gates += 1;
        }
        assert_eq!(gates, 9);
    }

    #[test]
    fn card_and_labels_come_from_the_data() {
        assert_eq!(CHOICES[ANSWER], "a car-shaped wall that goes on sideways forever");
        assert!(QUOTE_A.concat().starts_with("Even in the laminar case"));
        let (text, tex) = glyph_strings();
        assert!(text.contains(&"road: −U".to_owned()) && text.contains(&"air: −U".to_owned()));
        assert!(text.contains(&"car: +U".to_owned()));
        assert!(text.iter().chain(&tex).all(|s| !s.to_lowercase().contains("validat")));
        assert_eq!(tex.len(), 6 + 7 + 2);
    }

    #[test]
    fn layout_stays_in_frame() {
        for (at, size) in [
            (Q_CARD_AT, Q_CARD_SIZE),
            (QUOTE_CARD_AT, QUOTE_CARD_SIZE),
            (FORMULA_CARD_AT, FORMULA_CARD_SIZE),
        ] {
            assert!(at.0 - size.0 / 2.0 >= -7.6 && at.0 + size.0 / 2.0 <= 7.6);
            assert!(at.1 - size.1 / 2.0 >= -4.4 && at.1 + size.1 / 2.0 <= 3.6);
        }
        // the outline in each pose, and the ridge's ends, stay in the frame
        let o = car::outline();
        for pose in [POSE_A, POSE_B, POSE_C, BLOCK_POSE] {
            for &p in &o {
                let (x, y) = pose.at(p);
                assert!((-7.6..=7.6).contains(&x) && (-4.4..=3.6).contains(&y));
            }
        }
        for z in [-ridge::reach(), ridge::reach()] {
            for &(x, y) in &o {
                let (sx, sy) = POSE_B.at3((x, y, z));
                assert!((-7.6..=7.6).contains(&sx) && (-4.4..=3.6).contains(&sy));
            }
        }
        // the car drives in from inside the frame, and the road's dashes wrap off screen
        assert!(ROAD_X.0 < -8.0 - DASH && ROAD_X.1 > 8.0);
        assert!(POSE_C.origin.0 - DRIVE >= -7.6);
        // the tube at full stretch
        let half = 2.0 * TUBE_L0 * tube::cos_view() / 2.0;
        assert!(TUBE_AT.0 - half - TUBE_R0 * tube::SIN_VIEW >= -7.6 && TUBE_AT.0 + half < SWIRL_AT.0 - SWIRL_R);
        assert!(SWIRL_AT.0 + DRIFT + SWIRL_R <= 7.6);
        // the hold's ring sits inside its card, clear of the label row
        let (rx, ry) = Prediction::ring_at();
        assert!(
            rx + RING_R < Q_CARD_AT.0 + Q_CARD_SIZE.0 / 2.0 && ry + RING_R < Q_CARD_AT.1 + Q_CARD_SIZE.1 / 2.0 - 0.4
        );
    }
}
