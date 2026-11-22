//! Chapter 19, "Two numerical languages": cues 19_1 to 19_10 and the 19_h1 hold, native and exact.
//! No lattice Boltzmann output of any kind is drawn. The stars, the 5 × 5 patch and its one step,
//! the density bar and the momentum chain are exact arithmetic on nine illustrative amounts
//! ([`super::shapes19`]); the Taylor–Green pattern and the shear layer are drawn from their
//! formulas; the one measured value is RubyRaven's retained prototype ratio, printed as its run
//! printed it; the car is chapter 18's own outline, with no flow.
//!
//! RubyRaven `ruby-lattice01` for root's `navier_stokes_simulation` crate (Agent Mail 36272). Cue
//! text and pictures: `navier-full-script03.md` chapter 19 (sha256 31a83a2b…). The measured ratio:
//! independent01 `runs/lbm_tg01-run03.txt` (sha256 cc48d55c…). `tools/check_lattice01.py`
//! recomputes every number in Fractions and binds each quoted value to its pinned file.
//!
//! Every picture says what kind of thing it is:
//! - exact (the method's own rules and definitions, in lattice units): the stream and collide
//!   rules, ρ = Σ f_i and ρu = Σ f_i c_i, ν = (τ − 1/2)/3, p = ρ/3, Ma = U/c_s;
//! - illustration: the nine amounts (in 36ths), the patch, τ = 2 in the patch, the exaggerated
//!   densities, the slider's speeds and the illustrative scale ½ Ma², the swirls, the car;
//! - measured: "measured rate / predicted = 1.00006", the prototype's printed ratio at 128 × 128,
//!   labelled as a diagnostic prototype, not the film's solver.
//!
//! The pictures, in the order the lines ask for them:
//! - 19_1: one cell's nine-arrow star, "lattice Boltzmann (D2Q9)"; chapter 7's staggered cell
//!   beside it (one pressure, velocity across each face); the nine amounts as numbers;
//! - 19_2: the numbers of the cell standing still, toward a side, toward a corner, boxed in turn;
//!   the cell becomes the middle of a 5 × 5 patch; every moving amount hops one cell (stream);
//!   the faint balanced mix appears in the four cells that changed, and their stars ease halfway
//!   to it (collide, τ = 2);
//! - 19_3: the same nine amounts stack into a density bar, then add tip to tail into one momentum
//!   arrow; ρ = Σ f_i, ρu = Σ f_i c_i and the cell's velocity;
//! - 19_4: ν = (τ − 1/2)/3; the prototype's τ and ν; its two labels; the Taylor–Green pattern from
//!   its formula, shrinking along the dashed predicted curve; then the measured ratio;
//! - 19_5: "no pressure puzzle (ch. 9)"; two neighbouring cells, denser (orange) and less dense
//!   (teal), "exaggerated", p = ρ/3; at twice the speed, four times the difference;
//! - 19_6: the question card; 19_h1: the 5 s countdown (the slider and the squeeze readout do not
//!   exist yet);
//! - 19_7: the answer; chapter 17's channel, a speed slider and the squeeze meter, readouts only at
//!   each stop (0.05, 0.1, 0.025 cells per step); Ma = U/c_s; a uniform stream, "Δρ = 0"; near the
//!   cylinder, Δρ > 0;
//! - 19_8: the projection card and the lattice Boltzmann card, both arrows into one test card;
//! - 19_9: a shear layer from its formula with schematic small swirls; the coarse grid greys them
//!   out; the violet eddy viscosity, strongest where the shear is; the model's label and formula;
//! - 19_10: a neutral card with chapter 18's car outline on a lattice, no flow, "flat
//!   illustration, not a car measurement"; the reference animation credited in the description.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::f64::consts::{FRAC_PI_2, TAU};
use std::rc::Rc;

use super::shapes18::{Pose, car};
use super::shapes19::{self as sh, d2q9, hero, mach, patch, shear, squeeze_cells, star, taylor_green};
use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Palette, Timed, hex};
use fmn::prelude::*;

// ------------------------------------------------------------------ layout

/// The title, at chapter 17's place and size.
pub const TITLE: &str = "Two numerical languages";
pub const TITLE_AT: (f64, f64) = (-7.4, 3.95);
pub const TITLE_PT: f64 = 40.0;
pub const BODY_PT: f64 = 28.0;
pub const SMALL_PT: f64 = 24.0;
pub const FINE_PT: f64 = 22.0;
/// The side panel's centre line (the panel spans x 0.4 to 6.95).
pub const PANEL_X: f64 = 3.7;

/// 19_1 and 19_3: one cell, 2.4 units wide; its star is drawn in cell widths.
pub const HERO: Pose = Pose {
    origin: (-3.45, 0.2),
    scale: 2.4,
};
pub const HERO_NAME: &str = "lattice Boltzmann (D2Q9)";
pub const HERO_NAME_AT: (f64, f64) = (-3.45, 1.85);
/// Chapter 7's staggered cell (`story/chapter07.rs`: CELL_WIDTH 1.38, gold pressure dot of radius
/// 0.089, blue face arrows of thickness 1.55 from 0.2 before the face to 0.22 past it).
pub const MAC_AT: (f64, f64) = (1.6, 0.2);
pub const MAC_CELL: f64 = 1.38;
pub const MAC_NAME: &str = "our method (ch. 7)";
pub const MAC_NOTE: &str = "a pressure; velocity across each face";
pub const MAC_TEXT_Y: [f64; 2] = [1.35, -0.95];
/// The nine amounts as numbers, each where its direction points.
pub const GRID_AT: (f64, f64) = (5.4, 0.2);
pub const GRID_STEP: f64 = 0.62;
pub const GRID_NAME: &str = "the nine amounts";
pub const GRID_TEXT_Y: [f64; 2] = [1.35, -0.95];

/// 19_2.
pub const SAYS: [&str; 3] = ["standing still", "toward a side", "toward a corner"];
/// Which amounts each of [`SAYS`] picks out: standing still, the four sides, the four corners.
pub const SAYS_PICKS: [&[usize]; 3] = [&[0], &[1, 2, 3, 4], &[5, 6, 7, 8]];
pub const SAYS_Y: f64 = -1.45;
pub const BOX_SIZE: (f64, f64) = (0.52, 0.42);
pub const PATCH: Pose = Pose {
    origin: (-3.45, 0.0),
    scale: 1.2,
};
pub const PATCH_TAG: &str = "illustration: every other cell holds still fluid's balanced mix";
pub const PATCH_TAG_AT: (f64, f64) = (-3.45, -3.75);
pub const STREAM_NAME: &str = "stream: each moving amount hops one cell";
pub const STREAM_TEX: &str = r"f_i(x + c_i,\ t + 1) = f_i^{*}(x,\ t)";
pub const COLLIDE_NAME: &str = "collide: relax toward the balanced mix";
pub const COLLIDE_TEX: &str = r"f_i^{*} = f_i - (f_i - f_i^{\mathrm{eq}})/\tau";
pub const GHOST_NOTE: &str = "faint stars: the balanced mix, same density, same momentum";
pub const BALANCED_NOTE: &str = "(the other cells already hold theirs)";
pub const RULES_TAG: &str = "exact: the method's update rules (BGK)";
/// Stream name and rule, collide name and rule, τ, the two notes, the tag.
pub const RULES_Y: [f64; 8] = [2.75, 2.25, 1.35, 0.85, 0.2, -0.65, -1.1, -2.0];
/// Background stars are drawn fainter than the one cell we follow.
pub const BACKGROUND_OPACITY: f64 = 0.45;

/// 19_3.
pub const DENSITY_NAME: &str = "density: add the nine";
pub const MOMENTUM_NAME: &str = "momentum: weight each by its direction";
/// Density name, its sum, momentum name, its x and y parts.
pub const SIDE_Y3: [f64; 5] = [2.75, 2.3, 1.45, 1.0, 0.6];
pub const AMOUNTS_TAG_AT: (f64, f64) = (-3.45, -1.45);
pub const BAR_X: f64 = -0.9;
pub const BAR_BASE: f64 = -1.0;
/// Scene units per unit of density.
pub const BAR_SCALE: f64 = 2.2;
pub const BAR_TICK: f64 = 0.08;
pub const BAR_NAME: &str = "density";
pub const BAR_NAME_AT: (f64, f64) = (-0.9, -1.45);
pub const CHAIN_AT: (f64, f64) = (1.5, -1.7);
/// The chain is the star's arrows at twice their size, tip to tail in this order: north,
/// north-east, east, south-east, south, south-west, west, north-west.
pub const CHAIN_SCALE: f64 = 2.0;
pub const CHAIN_ORDER: [usize; 8] = [2, 5, 1, 8, 4, 7, 3, 6];
pub const MOMENTUM_LABEL: &str = "momentum";
pub const MOMENTUM_LABEL_AT: (f64, f64) = (2.15, -2.35);
pub const DENSITY_TEX: &str = r"\rho = \sum_i f_i";
pub const MOMENTUM_TEX: &str = r"\rho u = \sum_i f_i\, c_i";
pub const DEFS_AT: [(f64, f64); 2] = [(5.7, -0.55), (5.7, -1.3)];
pub const DEFS_TAG: &str = "exact: the method's definitions";
pub const VELOCITY_Y: f64 = -2.9;
pub const DEFS_TAG_Y: f64 = -3.35;

/// 19_4: one period of the Taylor–Green pattern in a 4.6-unit square.
pub const TG: Pose = Pose {
    origin: (-3.6, 0.0),
    scale: 4.6,
};
pub const TG_NAME: &str = "Taylor–Green pattern, drawn from its formula";
pub const TG_NAME_AT: (f64, f64) = (-3.6, 2.7);
/// The arrow of unit speed, scene units.
pub const TG_ARROW: f64 = 0.5;
pub const TG_HEAD: f64 = 0.12;
pub const NU_TEX: &str = r"\nu = (\tau - 1/2)/3";
pub const NU_TAG: &str = "exact, in lattice units";
/// ν's rule, its tag, the prototype's τ and ν, the measured ratio, its percentage.
pub const SIDE_Y4: [f64; 5] = [3.0, 2.5, 2.0, 1.35, 0.9];
/// The amplitude graph: x0, x1, y0, y1; [`GRAPH_S`] e-foldings across; amplitude 1 at
/// [`GRAPH_TOP`] of the height.
pub const GRAPH: [f64; 4] = [1.2, 6.6, -2.2, 0.4];
pub const GRAPH_S: f64 = 2.0;
pub const GRAPH_TOP: f64 = 0.9;
pub const DASHES: usize = 16;
pub const DECAY_TEX: &str = r"\text{predicted: } e^{-2\nu k^{2} t}";
pub const DECAY_TEX_AT: (f64, f64) = (4.8, -0.4);
pub const AMPLITUDE: &str = "amplitude";
pub const AMPLITUDE_AT: (f64, f64) = (1.2, 0.7);
pub const TIME: &str = "time";
pub const TIME_AT: (f64, f64) = (6.6, -2.5);
pub const XR_LABEL: &str = "lattice Boltzmann, weakly compressible; low Mach";
pub const PROTO_AT: (f64, f64) = (0.0, -2.95);
pub const XR_AT: (f64, f64) = (0.0, -3.4);

/// 19_5: two neighbouring cells.
pub const CELLS: [Pose; 2] = [
    Pose {
        origin: (-4.6, 0.0),
        scale: 2.2,
    },
    Pose {
        origin: (-2.4, 0.0),
        scale: 2.2,
    },
];
pub const NO_PUZZLE: &str = "no pressure puzzle (ch. 9)";
pub const NO_PUZZLE_AT: (f64, f64) = (-3.5, 2.95);
pub const EXAGGERATED: &str = "exaggerated";
pub const EXAGGERATED_AT: (f64, f64) = (-3.5, -1.45);
pub const DENSITY_LABEL_Y: f64 = -1.95;
pub const PRESSURE_TEX: &str = r"p = \rho/3";
pub const DP_TEX: &str = r"\Delta p = \Delta\rho/3";
pub const PRESSURE_AT: [(f64, f64); 2] = [(-4.6, -2.6), (-2.4, -2.6)];
pub const UNITS_TEX: &str = r"\text{exact, in lattice units: } c_s^{2} = 1/3";
pub const UNITS_AT: (f64, f64) = (-3.5, -3.2);
pub const FLOW_AT: (f64, f64) = (-5.6, 1.65);
pub const FLOW_LEN: f64 = 1.0;
pub const FASTER_AT: (f64, f64) = (-3.5, 2.35);
/// Tint opacity per unit of density difference.
pub const TINT_PER: f64 = 1.5;

/// 19_6 and 19_h1.
pub const Q_CARD_AT: (f64, f64) = (3.9, -0.6);
pub const Q_CARD_SIZE: (f64, f64) = (7.3, 3.2);
pub const Q_LABEL: &str = "your prediction";
pub const QUESTION: &str = "Which flows does lattice Boltzmann handle most like ours?";
pub const QUESTION_LINES: [&str; 2] = ["Which flows does lattice Boltzmann", "handle most like ours?"];
pub const CHOICES: [&str; 3] = ["slow", "fast", "no difference"];
/// Slow (19_7).
pub const ANSWER: usize = 0;
pub const PAUSE_NOTE: &str = "pause for more time";
pub const HOLD_SECONDS: f64 = 5.0;
pub const RING_R: f64 = 0.3;
pub const CHOICE_STEP: f64 = 0.38;

/// 19_7: chapter 17's channel as 17_8 placed it (metres from the inlet's lower corner, at 17_7's
/// scale, lifted by `CH_LIFT`): walls and cylinder only.
pub const CH_SCALE: f64 = 6.0;
pub const CH_ORIGIN: (f64, f64) = (-6.6, -1.5);
pub const CH_LIFT: f64 = 1.2;
pub mod channel {
    pub const LENGTH: f64 = 2.2;
    pub const HEIGHT: f64 = 0.41;
    pub const DIAMETER: f64 = 0.1;
    pub const CENTRE: (f64, f64) = (0.2, 0.2);
}
pub const CHANNEL_NAME: &str = "the benchmark channel (ch. 17)";
pub const CHANNEL_NAME_AT: (f64, f64) = (-6.6, 3.0);
pub const SLIDER_X: (f64, f64) = (-6.3, -1.3);
pub const SLIDER_Y: f64 = -1.25;
pub const SLIDER_NAME: &str = "speed U, in cells per step";
pub const SLIDER_NAME_AT: (f64, f64) = (-3.8, -0.75);
pub const TICK_Y: f64 = -1.6;
pub const KNOB_R: f64 = 0.12;
/// Speed, Mach number, the Mach rule, its tag.
pub const READ_AT: [(f64, f64); 4] = [(-3.8, -2.1), (-3.8, -2.55), (-3.8, -3.1), (-3.8, -3.55)];
pub const MA_TEX: &str = r"\mathrm{Ma} = U/c_s,\quad c_s = 1/\sqrt{3}";
pub const MA_TAG: &str = "exact, in lattice units";
pub const METER_X: (f64, f64) = (1.0, 7.0);
pub const METER_Y: f64 = -1.25;
pub const METER_H: f64 = 0.24;
pub const METER_TEX: &str =
    r"\text{density difference near the cylinder (illustrative scale }\frac{1}{2}\,\mathrm{Ma}^{2}\text{)}";
pub const METER_NAME_AT: (f64, f64) = (4.0, -0.72);
pub const SQUEEZE_AT: (f64, f64) = (4.0, -1.75);
pub const NOTE7_AT: (f64, f64) = (4.0, -2.2);
pub const INSET_AT: (f64, f64) = (4.0, -3.1);
pub const INSET_SIZE: (f64, f64) = (6.6, 1.0);
pub const INSET_ARROW_X: (f64, f64) = (1.1, 2.1);
pub const INSET_ARROW_Y: [f64; 3] = [-2.8, -3.1, -3.4];
pub const UNIFORM_NAME: &str = "uniform stream, no cylinder";
pub const UNIFORM_AT: (f64, f64) = (2.5, -2.85);
pub const ZERO_TEX: &str = r"\Delta\rho = 0";
pub const ZERO_AT: (f64, f64) = (2.5, -3.3);
pub const ANY_SPEED: &str = "at any speed (exact)";
pub const ANY_SPEED_AT: (f64, f64) = (3.85, -3.3);
pub const NEAR_TEX: &str = r"\text{near the cylinder: } \Delta\rho > 0\ \text{at every speed } U > 0";
pub const NEAR_AT: (f64, f64) = (-1.7, 0.93);

/// 19_8.
pub const CARD8_AT: [(f64, f64); 2] = [(-3.85, 1.25), (3.85, 1.25)];
pub const CARD8_SIZE: (f64, f64) = (7.3, 1.8);
pub const PROJ_TEXT: &str = "projection: no squeeze, down to the leftover imbalance";
pub const PROJ_NOTE: &str = "leftover imbalance: chapter 10's residual";
pub const LBM_TEX: &str = r"\text{lattice Boltzmann: squeezes, about }\mathrm{Ma}^{2}";
pub const LBM_NOTE: &str = "weakly compressible; low Mach";
/// Each card's main line and note.
pub const CARD8_Y: [f64; 2] = [1.5, 0.95];
pub const BOTH: &str = "two approximations, wrong in different ways";
pub const BOTH_AT: (f64, f64) = (0.0, -0.25);
pub const BENCH_AT: (f64, f64) = (0.0, -1.85);
pub const BENCH_SIZE: (f64, f64) = (7.0, 1.6);
pub const BENCH_NAME: &str = "both must pass the same tests";
pub const GATE_CHECKS: [&str; 2] = ["convergence: ?", "benchmark: ?"];
pub const BENCH_Y: [f64; 2] = [-1.45, -2.15];
pub const BENCH_X: [f64; 2] = [-1.6, 1.6];
pub const FEED: [[(f64, f64); 2]; 2] = [[(-3.85, 0.25), (-2.2, -0.95)], [(3.85, 0.25), (2.2, -0.95)]];

/// 19_9: the coarse grid's centre (its cells are [`shear::CELL`]).
pub const SHEAR_AT: (f64, f64) = (-2.9, 0.35);
/// The arrow of unit speed, scene units; rows slower than [`SHEAR_MIN`] draw no arrow.
pub const SHEAR_ARROW: f64 = 0.8;
pub const SHEAR_MIN: f64 = 0.05;
pub const SHEAR_NAME: &str = "resolved flow: a shear layer, drawn from its formula";
pub const SWIRLS_TAG: &str = "small swirls: schematic";
pub const COARSE_NOTE: &str = "a coarse grid: too coarse for the small swirls";
pub const TINT_NOTE: &str = "extra viscosity where the shear is strongest";
pub const NU_T_TEX: &str = r"\nu_t = (C_{\mathrm{Smag}}\,\Delta)^{2}\,|S|";
pub const NU_T_NOTE_TEX: &str = r"\text{standard form; } C_{\mathrm{Smag}}\text{ is the model's constant}";
pub const SIDE_X9: f64 = 4.55;
/// Shear name, swirls tag, coarse note, tint note, the formula, its note.
pub const SIDE_Y9: [f64; 6] = [2.75, 2.3, 1.5, 1.05, 0.2, -0.35];
pub const MODEL_LABEL: &str = "eddy-viscosity model (for example Smagorinsky): a model, not part of Navier-Stokes";
pub const MODEL_LABEL_AT: (f64, f64) = (0.0, -2.85);
pub const DETAIL_NOTE: &str = "average effect only; the swirls' detail is not restored";
pub const DETAIL_AT: (f64, f64) = (0.0, -3.35);

/// 19_10.
pub const CAR_CARD_AT: (f64, f64) = (0.0, -0.2);
pub const CAR_CARD_SIZE: (f64, f64) = (9.4, 4.4);
/// Chapter 18's outline, at chapter 18's scale.
pub const CAR_POSE: Pose = Pose {
    origin: (-2.925, -0.9),
    scale: 1.3,
};
pub const LATTICE_STEP: f64 = 0.4;
/// The lattice behind the car: x0, x1, y0, y1.
pub const LATTICE: [f64; 4] = [-4.4, 4.4, -1.3, 1.3];
pub const CARD_NOTE: &str = "our own diagram: a car-like outline on a lattice, no flow";
pub const CARD_NOTE_AT: (f64, f64) = (0.0, 1.6);
/// Chapter 18's permanent label (18_8).
pub const PERMANENT: &str = "flat illustration, not a car measurement";
pub const PERMANENT_AT: (f64, f64) = (0.0, -1.75);
pub const CREDIT: &str = "the reference animation is credited in the description, not shown";
pub const CREDIT_AT: (f64, f64) = (0.0, -3.0);

// ------------------------------------------------------------------ style

const WALL_W: f64 = 4.0;
const DIM_W: f64 = 2.0;
const CIRCLE_W: f64 = 2.0;
const STAR_W: f64 = 3.0;
const GHOST_W: f64 = 1.5;
const GRID_W: f64 = 1.5;
const FLOW_W: f64 = 3.5;
const FLOW_T: f64 = 3.0;
const MAC_T: f64 = 1.55;
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
const HOP_T: f64 = 1.0;
const COLLIDE_T: f64 = 1.5;
const BAR_T: f64 = 1.0;
const CHAIN_T: f64 = 1.5;
const DECAY_T: f64 = 4.0;
const EASE_T: f64 = 1.0;
const SLIDE_T: f64 = 1.0;

/// Labels closer than this (scene units; 24 px at 4K) count as touching.
const LABEL_GAP: f64 = 0.1;

fn solid_grey() -> Srgb {
    hex("#3B4352")
}

fn tyre() -> Srgb {
    hex("#232A36")
}

fn card_fill() -> Srgb {
    hex("#141C29")
}

/// Higher pressure, denser (chapter 4's push colour, `box_view::push_color`).
fn push_orange() -> Srgb {
    hex("#FF7A47")
}

/// The eddy viscosity (chapter 4's violet).
fn violet() -> Srgb {
    hex("#B48CFF")
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
    Style::default().fill(fill, 1.0).stroke(Palette::ink(), 2.5, 1.0)
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

fn tex_left(kit: &Kit, stage: &mut Stage<'_>, src: &str, size: f64, color: Srgb, at: (f64, f64)) -> fmn::Result<Mob> {
    let m = tex(kit, stage, src, size, color, at)?;
    stage.move_to(m, vp(at), LEFT);
    Ok(m)
}

fn rect(at: (f64, f64), size: (f64, f64), corner: f64, style: Style) -> fmn::Result<VMobject> {
    Ok(Rectangle::new()
        .width(size.0)
        .height(size.1)
        .corner_radius(corner)
        .style(style)
        .build()?
        .moved_to(vp(at)))
}

fn card(stage: &mut Stage<'_>, at: (f64, f64), size: (f64, f64)) -> fmn::Result<Mob> {
    let style = Style::default()
        .fill(card_fill(), 0.96)
        .stroke(Palette::dim(), 2.5, 1.0);
    Ok(add(stage, rect(at, size, 0.18, style)?, Z_TEXT - 1))
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

/// The built points of a path, for [`morph`].
fn built(points: Vec<Vec3>) -> Vec<Vec3> {
    polyline(points).build().points().to_vec()
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

fn smooth(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
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

/// Where `phrase` begins in sentence `j` of line `id` (of [`SCRIPT19`]), as a share of that
/// sentence's characters.
pub fn phrase_share(id: &str, j: usize, phrase: &str) -> Option<f64> {
    let line = SCRIPT19
        .lines()
        .find_map(|row| row.strip_prefix(id)?.strip_prefix('\t'))?;
    let sentence = *sentences(line).get(j)?;
    let at = sentence.find(phrase)?;
    Some(sentence[..at].chars().count() as f64 / sentence.chars().count() as f64)
}

/// Wait until `phrase` of sentence `j` of line `id` is about to be said: the sentence's start
/// (the narration's own, from the take's samples once recorded) plus the phrase's share of the
/// sentence's text times the time to the next sentence, or, in the line's last sentence, to the
/// line's end. The share is an estimate by text length, and the pause after the sentence is
/// counted in, so a picture lands on its words or a little after. `t0` is the scene time the line
/// began, read right after `kit.say`. Without a narrator it waits for nothing.
fn until_phrase(kit: &Kit, stage: &mut Stage<'_>, t0: f64, id: &str, j: usize, phrase: &str) -> fmn::Result<()> {
    let share =
        phrase_share(id, j, phrase).ok_or_else(|| scene_error(format!("{id} sentence {j} does not say {phrase:?}")))?;
    let starts = kit.sentence_starts(id);
    let end = starts
        .get(j + 1)
        .copied()
        .or_else(|| kit.narrator.as_ref().and_then(|n| n.length(id)));
    let (Some(&a), Some(b)) = (starts.get(j), end) else {
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

/// One mobject's points, eased from `from` to `to` (both built with the same number of points).
pub struct Morph {
    pub mob: Mob,
    pub from: Vec<Vec3>,
    pub to: Vec<Vec3>,
}

/// Ease every [`Morph`] together over `seconds`, one frame at a time.
fn morph(stage: &mut Stage<'_>, moves: &[Morph], seconds: f64) -> fmn::Result<()> {
    if let Some(m) = moves.iter().find(|m| m.from.len() != m.to.len()) {
        return Err(scene_error(format!("morph: {} points to {}", m.from.len(), m.to.len())));
    }
    let fps = stage.scene().time().fps();
    let frames = frames_covering(seconds, fps).max(1);
    for k in 1..=frames {
        let s = smooth(k as f64 / frames as f64);
        for m in moves {
            let points: Vec<Vec3> = m
                .from
                .iter()
                .zip(&m.to)
                .map(|(a, b)| v(a[0] + s * (b[0] - a[0]), a[1] + s * (b[1] - a[1])))
                .collect();
            let _ = stage.set_points(m.mob, &points);
        }
        wait_frames(stage, 1)?;
    }
    Ok(())
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

    /// Record that a fade the caller plays takes `m` off screen.
    fn leaving(&mut self, stage: &mut Stage<'_>, m: Mob) {
        if let Some(k) = self.0.iter().position(|&s| s == m) {
            self.0.remove(k);
            stage.clear_updaters(m, true);
        }
    }

    /// Record that a fade the caller plays brings `m` on screen.
    fn arriving(&mut self, m: Mob) {
        if !self.0.contains(&m) {
            self.0.push(m);
        }
    }
}

// ----------------------------------------------------------------- stars

/// Amount `i`'s arrow in the cell at `pose` (centre and width), as a point run.
fn star_arrow(pose: &Pose, i: usize, a: f64) -> Vec<Vec3> {
    let [tail, tip] = star::arrow(i, a);
    let l = (tip.0 - tail.0).hypot(tip.1 - tail.1);
    headed(&[pose.at(tail), pose.at(tip)], pose.scale * star::head(l))
}

/// The resting amount's disc in the cell at `pose`, as built points.
fn rest_points(pose: &Pose, a: f64) -> Vec<Vec3> {
    Circle::new()
        .radius(pose.scale * star::dot_radius(a))
        .arc_center(vp(pose.origin))
        .build()
        .points()
        .to_vec()
}

fn arrow_mob(stage: &mut Stage<'_>, pose: &Pose, i: usize, a: f64, color: Srgb, opacity: f64, width: f64) -> Mob {
    let line = polyline(star_arrow(pose, i, a))
        .style(line_style(color, width, opacity))
        .build();
    add(stage, line, Z_MARK)
}

fn rest_mob(stage: &mut Stage<'_>, pose: &Pose, a: f64, opacity: f64) -> Mob {
    let disc = Circle::new()
        .radius(pose.scale * star::dot_radius(a))
        .arc_center(vp(pose.origin))
        .style(fill_style(Palette::blue(), opacity))
        .build();
    add(stage, disc, Z_MARK)
}

/// One cell's star: its square, its resting disc and its eight arrows (amount i at index i − 1).
pub struct StarMobs {
    pub square: Mob,
    pub rest: Mob,
    pub arrows: Vec<Mob>,
}

impl StarMobs {
    pub fn all(&self) -> Vec<Mob> {
        [vec![self.square, self.rest], self.arrows.clone()].concat()
    }
}

fn build_star(stage: &mut Stage<'_>, pose: &Pose, f: &d2q9::Pops, opacity: f64) -> fmn::Result<StarMobs> {
    let square = add(
        stage,
        rect(
            pose.origin,
            (pose.scale, pose.scale),
            0.0,
            line_style(Palette::dim(), DIM_W, 1.0),
        )?,
        Z_LINE,
    );
    let rest = rest_mob(stage, pose, sh::to_f64(f[0]), opacity);
    let arrows = (1..9)
        .map(|i| arrow_mob(stage, pose, i, sh::to_f64(f[i]), Palette::blue(), opacity, STAR_W))
        .collect();
    Ok(StarMobs { square, rest, arrows })
}

/// The cell `(x, y)` of the 19_2 patch.
fn cell_pose(x: i64, y: i64) -> Pose {
    Pose {
        origin: (
            PATCH.origin.0 + PATCH.scale * x as f64,
            PATCH.origin.1 + PATCH.scale * y as f64,
        ),
        scale: PATCH.scale,
    }
}

fn inside_window(x: i64, y: i64) -> bool {
    x.abs() <= patch::HALF && y.abs() <= patch::HALF
}

/// The 19_2 patch: grid lines, and every window cell's amounts (the middle cell's are the hero
/// star's, carried in).
struct Patch {
    lines: Mob,
    arrows: BTreeMap<(i64, i64, usize), Mob>,
    rests: BTreeMap<(i64, i64), Mob>,
    background: Vec<Mob>,
}

fn build_patch(stage: &mut Stage<'_>, hero_star: &StarMobs) -> Patch {
    let (h, c) = (patch::HALF as f64 + 0.5, PATCH.scale);
    let (ox, oy) = PATCH.origin;
    let style = line_style(Palette::dim(), GRID_W, 1.0);
    let mut lines = Vec::new();
    for k in 0..=(2 * patch::HALF + 1) {
        let d = (k as f64 - h) * c;
        lines.push(seg(v(ox + d, oy - h * c), v(ox + d, oy + h * c), style));
        lines.push(seg(v(ox - h * c, oy + d), v(ox + h * c, oy + d), style));
    }
    let lines = group(stage, lines, Z_LINE);
    let (mut arrows, mut rests, mut background) = (BTreeMap::new(), BTreeMap::new(), vec![lines]);
    for (x, y) in patch::window() {
        if (x, y) == (0, 0) {
            for (k, &m) in hero_star.arrows.iter().enumerate() {
                arrows.insert((0, 0, k + 1), m);
            }
            rests.insert((0, 0), hero_star.rest);
            continue;
        }
        let pose = cell_pose(x, y);
        let f = patch::before(x, y);
        let rest = rest_mob(stage, &pose, sh::to_f64(f[0]), BACKGROUND_OPACITY);
        rests.insert((x, y), rest);
        background.push(rest);
        for (i, &a) in f.iter().enumerate().skip(1) {
            let m = arrow_mob(
                stage,
                &pose,
                i,
                sh::to_f64(a),
                Palette::blue(),
                BACKGROUND_OPACITY,
                STAR_W,
            );
            arrows.insert((x, y, i), m);
            background.push(m);
        }
    }
    Patch {
        lines,
        arrows,
        rests,
        background,
    }
}

/// Chapter 7's staggered cell: its outline, the gold pressure dot at its centre, a blue arrow
/// across each face.
fn build_mac(stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let (x, y) = MAC_AT;
    let half = MAC_CELL / 2.0;
    let rim = Style::default()
        .stroke(Palette::dim(), 2.0, 1.0)
        .fill(Palette::dim(), 0.08);
    let mut mobs = vec![add(stage, rect(MAC_AT, (MAC_CELL, MAC_CELL), 0.0, rim)?, Z_LINE)];
    let gold = Palette::gold();
    let d = Dot::new()
        .point(vp(MAC_AT))
        .radius(0.089)
        .style(Style::default().fill(gold, 1.0).stroke(gold, 0.0, 1.0))
        .build();
    mobs.push(add(stage, d, Z_MARK));
    let blue = Palette::blue();
    for d in [-half, half] {
        mobs.push(arrow(stage, v(x + d - 0.2, y), v(x + d + 0.22, y), blue, MAC_T)?);
        mobs.push(arrow(stage, v(x, y + d - 0.2), v(x, y + d + 0.22), blue, MAC_T)?);
    }
    Ok(mobs)
}

/// Amount i's number in the 19_1 grid: where its direction points.
fn grid_at(i: usize) -> (f64, f64) {
    let (cx, cy) = d2q9::C[i];
    (GRID_AT.0 + GRID_STEP * cx as f64, GRID_AT.1 + GRID_STEP * cy as f64)
}

/// 19_3's bar: amount i's segment, in the order rest, east, north, west, south, the corners,
/// with a small tick at its top, as a five-point run like a star arrow's.
fn bar_segment(f: &d2q9::Pops, i: usize) -> Vec<Vec3> {
    let below: f64 = f[..i].iter().map(|&a| sh::to_f64(a)).sum();
    let (y0, y1) = (
        BAR_BASE + BAR_SCALE * below,
        BAR_BASE + BAR_SCALE * (below + sh::to_f64(f[i])),
    );
    vec![
        v(BAR_X, y0),
        v(BAR_X, y1),
        v(BAR_X - BAR_TICK, y1),
        v(BAR_X, y1),
        v(BAR_X + BAR_TICK, y1),
    ]
}

/// One link of 19_3's chain: the amount, its arrow's tail and its tip.
type Link = (usize, (f64, f64), (f64, f64));

/// 19_3's chain: each arrow's tail and tip, tip to tail in [`CHAIN_ORDER`], at
/// [`CHAIN_SCALE`] times the star's size.
fn chain(f: &d2q9::Pops) -> Vec<Link> {
    let k = CHAIN_SCALE * HERO.scale;
    let mut at = CHAIN_AT;
    CHAIN_ORDER
        .iter()
        .map(|&i| {
            let (dx, dy) = star::vector(i, sh::to_f64(f[i]));
            let tip = (at.0 + k * dx, at.1 + k * dy);
            let out = (i, at, tip);
            at = tip;
            out
        })
        .collect()
}

// ----------------------------------------------------------------- the card

/// The 19_h1 prediction card.
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
        if QUESTION_LINES.join(" ") != QUESTION {
            return Err(scene_error(
                "the card's lines are not the hold row's question".to_owned(),
            ));
        }
        let mut parts = vec![card(stage, Q_CARD_AT, Q_CARD_SIZE)?];
        parts.push(text_left(kit, stage, Q_LABEL, FINE_PT, dim, (left + 0.35, top - 0.27))?);
        for (k, line) in QUESTION_LINES.iter().enumerate() {
            parts.push(text_left(
                kit,
                stage,
                line,
                BODY_PT,
                ink,
                (left + 0.35, top - 0.7 - 0.4 * k as f64),
            )?);
        }
        let x = left + 0.45;
        let mut markers = Vec::new();
        let mut chosen = None;
        for (k, choice) in CHOICES.iter().enumerate() {
            let y = top - 1.65 - CHOICE_STEP * k as f64;
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

/// One arrow of the 19_4 pattern: its mobject, its centre and its velocity at amplitude 1.
type FieldArrow = (Mob, (f64, f64), (f64, f64));

/// 19_4's decay, one frame at a time once `from` is set: the pattern's arrows shrink by the
/// predicted amplitude e^(−s), s rising to [`GRAPH_S`] over [`DECAY_T`], and the dot rides the
/// dashed curve. Nothing here is simulated.
struct Decay {
    t: f64,
    from: Option<f64>,
    arrows: Vec<FieldArrow>,
    dot: Mob,
    at: (f64, f64),
}

/// A Taylor–Green arrow centred at `p`, velocity `w` (scene units), as a point run.
fn tg_arrow(p: (f64, f64), w: (f64, f64)) -> Vec<Vec3> {
    let l = w.0.hypot(w.1);
    headed(
        &[(p.0 - w.0 / 2.0, p.1 - w.1 / 2.0), (p.0 + w.0 / 2.0, p.1 + w.1 / 2.0)],
        (0.4 * l).min(TG_HEAD),
    )
}

/// The graph's point at s e-foldings, amplitude `a`.
fn graph_at(s: f64, a: f64) -> (f64, f64) {
    let [x0, x1, y0, y1] = GRAPH;
    (x0 + (x1 - x0) * s / GRAPH_S, y0 + (y1 - y0) * GRAPH_TOP * a)
}

impl Decay {
    fn step(&mut self, s: &mut fmn::mobject::Stage, dt: f64) {
        self.t += dt;
        let Some(t0) = self.from else {
            return;
        };
        let e = GRAPH_S * ((self.t - t0) / DECAY_T).clamp(0.0, 1.0);
        let a = taylor_green::amplitude(e);
        for &(m, p, w) in &self.arrows {
            let _ = s.set_points(m, polyline(tg_arrow(p, (a * w.0, a * w.1))).build().points());
        }
        let to = graph_at(e, a);
        s.shift(self.dot, v(to.0 - self.at.0, to.1 - self.at.1));
        self.at = to;
    }
}

// ----------------------------------------------------------------- the chapter

/// Chapter 19: what is on screen, and every object a later cue changes.
pub struct Lattice19 {
    pub screen: Screen,
    pub title: Mob,
    hero: Option<StarMobs>,
    numbers: Vec<Mob>,
    side: Vec<Mob>,
    driver: Option<Mob>,
}

impl Lattice19 {
    pub fn build(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let title = text_left(kit, stage, TITLE, TITLE_PT, Palette::ink(), TITLE_AT)?;
        Ok(Self {
            screen: Screen::default(),
            title,
            hero: None,
            numbers: Vec::new(),
            side: Vec::new(),
            driver: None,
        })
    }

    /// Stop the motion, if any, and fade everything but the title and `keep`.
    fn clear(&mut self, stage: &mut Stage<'_>, keep: &[Mob]) -> fmn::Result<()> {
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
        self.screen.hide(stage, &out, MOVE)
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

    // ---------------------------------------------------------------- 19_1

    pub fn cue_19_1(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "19_1")?;
        let t0 = stage.scene().time().to_f64();
        let (ink, dim) = (Palette::ink(), Palette::dim());
        self.screen.show(stage, &[self.title], FADE)?;
        let star = build_star(stage, &HERO, &hero::pops(), 1.0)?;
        let name = text(kit, stage, HERO_NAME, BODY_PT, ink, HERO_NAME_AT)?;
        self.screen.show(stage, &[star.all(), vec![name]].concat(), FADE)?;
        // Instead of velocity and pressure (chapter 7's cell), nine numbers.
        kit.until_sentence(stage, "19_1", 1)?;
        let mut mac = build_mac(stage)?;
        mac.push(text(kit, stage, MAC_NAME, SMALL_PT, ink, (MAC_AT.0, MAC_TEXT_Y[0]))?);
        mac.push(text(kit, stage, MAC_NOTE, FINE_PT, dim, (MAC_AT.0, MAC_TEXT_Y[1]))?);
        self.screen.show(stage, &mac, FADE)?;
        until_phrase(kit, stage, t0, "19_1", 1, "each cell keeps nine numbers")?;
        let mut numbers = Vec::new();
        for (i, a) in hero::AMOUNTS.iter().enumerate() {
            numbers.push(text(kit, stage, &a.to_string(), SMALL_PT, ink, grid_at(i))?);
        }
        let grid_labels = vec![
            text(kit, stage, GRID_NAME, SMALL_PT, ink, (GRID_AT.0, GRID_TEXT_Y[0]))?,
            text(
                kit,
                stage,
                &hero::amounts_tag(),
                FINE_PT,
                dim,
                (GRID_AT.0, GRID_TEXT_Y[1]),
            )?,
        ];
        self.screen
            .show(stage, &[numbers.clone(), grid_labels.clone()].concat(), FADE)?;
        let labels = [vec![name, mac[mac.len() - 2], mac[mac.len() - 1]], grid_labels.clone()].concat();
        refuse_touching(stage, &labels, "19_1")?;
        self.side = [vec![name], mac, grid_labels].concat();
        self.numbers = numbers;
        self.hero = Some(star);
        Ok(())
    }

    // ---------------------------------------------------------------- 19_2

    /// Box the numbers that [`SAYS`] `k` picks out, and name them, in place of the last ones.
    fn pick(&mut self, kit: &Kit, stage: &mut Stage<'_>, k: usize, last: &mut Vec<Mob>) -> fmn::Result<()> {
        let mut into = Vec::new();
        for &i in SAYS_PICKS[k] {
            let style = line_style(Palette::hero(), 2.5, 1.0);
            into.push(add(stage, rect(grid_at(i), BOX_SIZE, 0.08, style)?, Z_MARK));
        }
        into.push(text(
            kit,
            stage,
            SAYS[k],
            SMALL_PT,
            Palette::hero(),
            (GRID_AT.0, SAYS_Y),
        )?);
        let out = std::mem::take(last);
        self.screen.swap(stage, &out, &into, QUICK)?;
        *last = into;
        Ok(())
    }

    pub fn cue_19_2(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "19_2")?;
        let t0 = stage.scene().time().to_f64();
        let (ink, dim) = (Palette::ink(), Palette::dim());
        let mut picked = Vec::new();
        until_phrase(kit, stage, t0, "19_2", 0, "standing still")?;
        self.pick(kit, stage, 0, &mut picked)?;
        until_phrase(kit, stage, t0, "19_2", 0, "toward a side")?;
        self.pick(kit, stage, 1, &mut picked)?;
        until_phrase(kit, stage, t0, "19_2", 0, "toward a corner")?;
        self.pick(kit, stage, 2, &mut picked)?;
        // The cell becomes the middle of a 5 × 5 patch of still fluid.
        kit.until_sentence(stage, "19_2", 1)?;
        let hero_star = self
            .hero
            .take()
            .ok_or_else(|| scene_error("19_2 needs 19_1's cell".to_owned()))?;
        let out = [
            std::mem::take(&mut self.side),
            std::mem::take(&mut self.numbers),
            picked,
        ]
        .concat();
        let carries: Vec<Carry> = hero_star
            .all()
            .into_iter()
            .map(|mob| Carry {
                mob,
                from: HERO,
                to: PATCH,
            })
            .collect();
        self.screen.play(stage, &out, &[], &[], &carries, ORIGIN, MOVE)?;
        let mut cells = build_patch(stage, &hero_star);
        let tag = text(kit, stage, PATCH_TAG, FINE_PT, dim, PATCH_TAG_AT)?;
        self.screen
            .show(stage, &[cells.background.clone(), vec![tag]].concat(), FADE)?;
        // the patch's grid lines now cover the cell's own square
        self.screen.take(stage, hero_star.square);
        let rules = vec![
            text(kit, stage, STREAM_NAME, SMALL_PT, ink, (PANEL_X, RULES_Y[0]))?,
            tex(kit, stage, STREAM_TEX, SMALL_PT, ink, (PANEL_X, RULES_Y[1]))?,
            text(kit, stage, RULES_TAG, FINE_PT, dim, (PANEL_X, RULES_Y[7]))?,
        ];
        self.screen.show(stage, &rules, FADE)?;
        // Stream: every moving amount hops one cell along its own direction.
        until_phrase(kit, stage, t0, "19_2", 1, "hops exactly one cell in its own direction")?;
        self.stream(stage, &mut cells)?;
        // Collide: the four cells that changed relax halfway to their balanced mix.
        kit.until_sentence(stage, "19_2", 2)?;
        let changed = patch::changed();
        let mut ghosts = Vec::new();
        for &(x, y) in &changed {
            let pose = cell_pose(x, y);
            let e = d2q9::balanced(&patch::after_stream(x, y));
            let ring_r = pose.scale * star::dot_radius(sh::to_f64(e[0]));
            ghosts.push(ring(stage, vp(pose.origin), ring_r, ink, GHOST_W));
            for (i, &a) in e.iter().enumerate().skip(1) {
                ghosts.push(arrow_mob(stage, &pose, i, sh::to_f64(a), ink, 0.5, GHOST_W));
            }
        }
        for &g in &ghosts {
            stage.arena_mut().set_z_index(g, Z_LINE + 1, true);
        }
        let notes = vec![
            text(kit, stage, COLLIDE_NAME, SMALL_PT, ink, (PANEL_X, RULES_Y[2]))?,
            tex(kit, stage, COLLIDE_TEX, SMALL_PT, ink, (PANEL_X, RULES_Y[3]))?,
            tex(kit, stage, &patch::tau_tex(), FINE_PT, dim, (PANEL_X, RULES_Y[4]))?,
            text(kit, stage, GHOST_NOTE, FINE_PT, dim, (PANEL_X, RULES_Y[5]))?,
            text(kit, stage, BALANCED_NOTE, FINE_PT, dim, (PANEL_X, RULES_Y[6]))?,
        ];
        self.screen.show(stage, &[ghosts, notes.clone()].concat(), FADE)?;
        until_phrase(kit, stage, t0, "19_2", 2, "relax toward a balanced mix")?;
        let mut moves = Vec::new();
        for &(x, y) in &changed {
            let pose = cell_pose(x, y);
            let (a, b) = (patch::after_stream(x, y), patch::after_collide(x, y));
            let rest = cells.rests[&(x, y)];
            moves.push(Morph {
                mob: rest,
                from: rest_points(&pose, sh::to_f64(a[0])),
                to: rest_points(&pose, sh::to_f64(b[0])),
            });
            for i in 1..9 {
                moves.push(Morph {
                    mob: cells.arrows[&(x, y, i)],
                    from: built(star_arrow(&pose, i, sh::to_f64(a[i]))),
                    to: built(star_arrow(&pose, i, sh::to_f64(b[i]))),
                });
            }
        }
        morph(stage, &moves, COLLIDE_T)?;
        let _ = cells.lines;
        refuse_touching(stage, &[rules, notes, vec![tag]].concat(), "19_2")
    }

    /// One stream step of the patch, in one play: amounts that stay in the window slide one
    /// cell; amounts that leave fade out as they slide; amounts arriving from the still fluid
    /// around the window slide in as they fade in.
    fn stream(&mut self, stage: &mut Stage<'_>, cells: &mut Patch) -> fmn::Result<()> {
        let c = PATCH.scale;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        let mut moved = BTreeMap::new();
        for ((x, y, i), m) in std::mem::take(&mut cells.arrows) {
            let (cx, cy) = d2q9::C[i];
            let by = v(c * cx as f64, c * cy as f64);
            if inside_window(x + cx, y + cy) {
                let a = m
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(HOP_T),
                        ..AnimateArgs::default()
                    })?
                    .shift(by)?;
                anims.push(stage.prepare(a)?);
                moved.insert((x + cx, y + cy, i), m);
            } else {
                self.screen.leaving(stage, m);
                let a = fade_out(stage.arena_mut(), m, by, 1.0)?;
                anims.push(stage.prepare(a.rt(HOP_T))?);
            }
        }
        for (x, y) in patch::window() {
            let f = patch::after_stream(x, y);
            for (i, &(cx, cy)) in d2q9::C.iter().enumerate().skip(1) {
                if inside_window(x - cx, y - cy) {
                    continue;
                }
                let m = arrow_mob(
                    stage,
                    &cell_pose(x, y),
                    i,
                    sh::to_f64(f[i]),
                    Palette::blue(),
                    BACKGROUND_OPACITY,
                    STAR_W,
                );
                self.screen.arriving(m);
                let a = fade_in(stage.arena_mut(), m, v(c * cx as f64, c * cy as f64), 1.0)?;
                anims.push(stage.prepare(a.rt(HOP_T))?);
                moved.insert((x, y, i), m);
            }
        }
        stage.play_prepared(anims)?;
        cells.arrows = moved;
        Ok(())
    }

    // ---------------------------------------------------------------- 19_3

    pub fn cue_19_3(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "19_3")?;
        let t0 = stage.scene().time().to_f64();
        let (ink, dim, blue, gold) = (Palette::ink(), Palette::dim(), Palette::blue(), Palette::hero());
        self.clear(stage, &[])?;
        // The same nine amounts as 19_1's cell.
        let f = hero::pops();
        let star = build_star(stage, &HERO, &f, 1.0)?;
        let tag = text(kit, stage, &hero::amounts_tag(), FINE_PT, dim, AMOUNTS_TAG_AT)?;
        self.screen.show(stage, &[star.all(), vec![tag]].concat(), FADE)?;
        // Density: copies of the nine stack into one bar.
        until_phrase(kit, stage, t0, "19_3", 0, "and you get its density")?;
        let (cx, cy) = HERO.origin;
        let r = HERO.scale * star::dot_radius(sh::to_f64(f[0]));
        let rest_from = vec![
            v(cx, cy - r),
            v(cx, cy + r),
            v(cx - 0.02, cy + r),
            v(cx, cy + r),
            v(cx + 0.02, cy + r),
        ];
        let mut moves = Vec::new();
        for i in 0..9 {
            let from = if i == 0 {
                rest_from.clone()
            } else {
                star_arrow(&HERO, i, sh::to_f64(f[i]))
            };
            let m = add(
                stage,
                polyline(from.clone()).style(line_style(blue, STAR_W, 1.0)).build(),
                Z_MARK + 1,
            );
            self.screen.put(stage, m)?;
            moves.push(Morph {
                mob: m,
                from: built(from),
                to: built(bar_segment(&f, i)),
            });
        }
        morph(stage, &moves, BAR_T)?;
        let density = vec![
            text(kit, stage, DENSITY_NAME, SMALL_PT, ink, (PANEL_X, SIDE_Y3[0]))?,
            text(kit, stage, &hero::sum_label(), SMALL_PT, blue, (PANEL_X, SIDE_Y3[1]))?,
            text(kit, stage, BAR_NAME, FINE_PT, dim, BAR_NAME_AT)?,
        ];
        self.screen.show(stage, &density, FADE)?;
        // Momentum: copies of the eight moving amounts, twice their size, tip to tail.
        kit.until_sentence(stage, "19_3", 1)?;
        let name = text(kit, stage, MOMENTUM_NAME, SMALL_PT, ink, (PANEL_X, SIDE_Y3[2]))?;
        self.screen.show(stage, &[name], FADE)?;
        until_phrase(kit, stage, t0, "19_3", 1, "and you get its momentum")?;
        let mut moves = Vec::new();
        let k = CHAIN_SCALE * HERO.scale;
        for (i, tail, tip) in chain(&f) {
            let from = star_arrow(&HERO, i, sh::to_f64(f[i]));
            let m = add(
                stage,
                polyline(from.clone()).style(line_style(blue, STAR_W, 1.0)).build(),
                Z_MARK + 1,
            );
            self.screen.put(stage, m)?;
            let l = (tip.0 - tail.0).hypot(tip.1 - tail.1) / k;
            moves.push(Morph {
                mob: m,
                from: built(from),
                to: built(headed(&[tail, tip], k * star::head(l))),
            });
        }
        morph(stage, &moves, CHAIN_T)?;
        let (mx, my) = d2q9::momentum(&f);
        let a = star::ARROW * k;
        let end = (CHAIN_AT.0 + a * sh::to_f64(mx), CHAIN_AT.1 + a * sh::to_f64(my));
        let total = vec![
            arrow(stage, vp(CHAIN_AT), vp(end), gold, FLOW_T)?,
            text(kit, stage, MOMENTUM_LABEL, FINE_PT, gold, MOMENTUM_LABEL_AT)?,
            text(kit, stage, &hero::axis_label(0), FINE_PT, ink, (PANEL_X, SIDE_Y3[3]))?,
            text(kit, stage, &hero::axis_label(1), FINE_PT, ink, (PANEL_X, SIDE_Y3[4]))?,
        ];
        self.screen.show(stage, &total, FADE)?;
        // The same kind of flow: density and velocity, from different numbers.
        kit.until_sentence(stage, "19_3", 2)?;
        let defs = vec![
            tex(kit, stage, DENSITY_TEX, SMALL_PT, ink, DEFS_AT[0])?,
            tex(kit, stage, MOMENTUM_TEX, SMALL_PT, ink, DEFS_AT[1])?,
            text(kit, stage, &hero::velocity_label(), FINE_PT, ink, (PANEL_X, VELOCITY_Y))?,
            text(kit, stage, DEFS_TAG, FINE_PT, dim, (PANEL_X, DEFS_TAG_Y))?,
        ];
        self.screen.show(stage, &defs, FADE)?;
        let labels = [vec![tag, name], density, total[1..].to_vec(), defs].concat();
        refuse_touching(stage, &labels, "19_3")
    }

    // ---------------------------------------------------------------- 19_4

    pub fn cue_19_4(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "19_4")?;
        let t0 = stage.scene().time().to_f64();
        let (ink, dim, faint, blue, gold) = (
            Palette::ink(),
            Palette::dim(),
            Palette::faint(),
            Palette::blue(),
            Palette::hero(),
        );
        self.clear(stage, &[])?;
        let rule = vec![
            tex(kit, stage, NU_TEX, SMALL_PT, ink, (PANEL_X, SIDE_Y4[0]))?,
            text(kit, stage, NU_TAG, FINE_PT, dim, (PANEL_X, SIDE_Y4[1]))?,
        ];
        self.screen.show(stage, &rule, FADE)?;
        // The prototype: its own τ and ν, and what it is.
        kit.until_sentence(stage, "19_4", 1)?;
        let proto_nu = tex(kit, stage, &taylor_green::nu_tex(), FINE_PT, dim, (PANEL_X, SIDE_Y4[2]))?;
        self.screen.show(stage, &[proto_nu], FADE)?;
        until_phrase(kit, stage, t0, "19_4", 1, "separate from the solver this film builds")?;
        let names = vec![
            text(kit, stage, &taylor_green::proto_label(), FINE_PT, ink, PROTO_AT)?,
            text(kit, stage, XR_LABEL, FINE_PT, dim, XR_AT)?,
        ];
        self.screen.show(stage, &names, FADE)?;
        // The pattern from its formula, fading along the predicted curve.
        kit.until_sentence(stage, "19_4", 2)?;
        let side = TG.scale;
        let mut field = vec![add(
            stage,
            rect(TG.origin, (side, side), 0.0, line_style(dim, DIM_W, 1.0))?,
            Z_LINE,
        )];
        let mut arrows = Vec::new();
        let n = taylor_green::SAMPLES;
        for my in 0..n {
            for mx in 0..n {
                let (xr, yr) = (taylor_green::sample(mx), taylor_green::sample(my));
                let p = TG.at((xr / TAU - 0.5, yr / TAU - 0.5));
                let (u, w) = taylor_green::velocity(xr, yr);
                let vel = (TG_ARROW * u, TG_ARROW * w);
                let m = add(
                    stage,
                    polyline(tg_arrow(p, vel)).style(line_style(blue, STAR_W, 1.0)).build(),
                    Z_MARK,
                );
                field.push(m);
                arrows.push((m, p, vel));
            }
        }
        field.push(text(kit, stage, TG_NAME, FINE_PT, ink, TG_NAME_AT)?);
        let [x0, x1, y0, y1] = GRAPH;
        let axes = group(
            stage,
            vec![
                seg(v(x0, y0), v(x1, y0), line_style(dim, DIM_W, 1.0)),
                seg(v(x0, y0), v(x0, y1), line_style(dim, DIM_W, 1.0)),
            ],
            Z_LINE,
        );
        let steps = 2 * DASHES;
        let curve: Vec<(f64, f64)> = (0..=steps)
            .map(|k| {
                let s = GRAPH_S * k as f64 / steps as f64;
                graph_at(s, taylor_green::amplitude(s))
            })
            .collect();
        let dashes = (0..DASHES)
            .map(|k| {
                polyline(vec![vp(curve[2 * k]), vp(curve[2 * k + 1])])
                    .style(line_style(ink, DIM_W, 0.8))
                    .build()
            })
            .collect();
        let start = graph_at(0.0, 1.0);
        let marker = dot(stage, vp(start), 0.08, gold);
        let graph = vec![
            axes,
            group(stage, dashes, Z_MARK),
            marker,
            tex(kit, stage, DECAY_TEX, FINE_PT, ink, DECAY_TEX_AT)?,
            text_left(kit, stage, AMPLITUDE, FINE_PT, dim, AMPLITUDE_AT)?,
            text_right(kit, stage, TIME, FINE_PT, dim, TIME_AT)?,
        ];
        self.screen
            .show(stage, &[field.clone(), graph.clone()].concat(), FADE)?;
        let _ = faint;
        let state = Rc::new(RefCell::new(Decay {
            t: 0.0,
            from: None,
            arrows,
            dot: marker,
            at: start,
        }));
        self.drive(stage, Rc::clone(&state), Decay::step)?;
        state.borrow_mut().from = Some(0.0);
        // The measured ratio, as the prototype's run printed it.
        kit.until_sentence(stage, "19_4", 3)?;
        let readout = vec![
            text(
                kit,
                stage,
                &taylor_green::ratio_label(),
                SMALL_PT,
                gold,
                (PANEL_X, SIDE_Y4[3]),
            )?,
            text(
                kit,
                stage,
                &taylor_green::within_label(),
                FINE_PT,
                gold,
                (PANEL_X, SIDE_Y4[4]),
            )?,
        ];
        self.screen.show(stage, &readout, FADE)?;
        let labels = [
            rule,
            vec![proto_nu],
            names,
            vec![field[field.len() - 1]],
            graph[3..].to_vec(),
            readout,
        ]
        .concat();
        refuse_touching(stage, &labels, "19_4")
    }

    // ---------------------------------------------------------------- 19_5

    pub fn cue_19_5(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "19_5")?;
        let t0 = stage.scene().time().to_f64();
        let (ink, dim, teal) = (Palette::ink(), Palette::dim(), Palette::teal());
        self.clear(stage, &[])?;
        kit.until_sentence(stage, "19_5", 1)?;
        let no_puzzle = text(kit, stage, NO_PUZZLE, SMALL_PT, ink, NO_PUZZLE_AT)?;
        self.screen.show(stage, &[no_puzzle], FADE)?;
        // Pressure is a third of the density: two neighbouring cells, one squeezed.
        kit.until_sentence(stage, "19_5", 2)?;
        let tints_at = |stage: &mut Stage<'_>, k: usize| -> fmn::Result<Vec<Mob>> {
            let d = sh::to_f64(squeeze_cells::SPLIT[k]);
            let mut out = Vec::new();
            for (pose, color) in CELLS.iter().zip([push_orange(), teal]) {
                let fill = fill_style(color, TINT_PER * d);
                out.push(add(
                    stage,
                    rect(pose.origin, (pose.scale, pose.scale), 0.0, fill)?,
                    Z_FACE,
                ));
            }
            Ok(out)
        };
        let tints = tints_at(stage, 0)?;
        let (hi, lo) = squeeze_cells::densities(0);
        let mut stars = Vec::new();
        for (pose, rho) in CELLS.iter().zip([hi, lo]) {
            stars.push(build_star(
                stage,
                pose,
                &d2q9::equilibrium(rho, (sh::int(0), sh::int(0))),
                1.0,
            )?);
        }
        let first = vec![
            text(kit, stage, EXAGGERATED, FINE_PT, dim, EXAGGERATED_AT)?,
            tex(kit, stage, PRESSURE_TEX, SMALL_PT, ink, PRESSURE_AT[0])?,
        ];
        let drawn: Vec<Mob> = stars.iter().flat_map(StarMobs::all).collect();
        self.screen
            .show(stage, &[tints.clone(), drawn, first.clone()].concat(), FADE)?;
        until_phrase(
            kit,
            stage,
            t0,
            "19_5",
            2,
            "so pressure differences come from squeezing the fluid a little",
        )?;
        let label_pair = |stage: &mut Stage<'_>, k: usize| -> fmn::Result<Vec<Mob>> {
            let (hi, lo) = squeeze_cells::densities(k);
            Ok(vec![
                tex(
                    kit,
                    stage,
                    &squeeze_cells::density_tex(hi),
                    SMALL_PT,
                    push_orange(),
                    (CELLS[0].origin.0, DENSITY_LABEL_Y),
                )?,
                tex(
                    kit,
                    stage,
                    &squeeze_cells::density_tex(lo),
                    SMALL_PT,
                    teal,
                    (CELLS[1].origin.0, DENSITY_LABEL_Y),
                )?,
            ])
        };
        let densities = label_pair(stage, 0)?;
        let rules = vec![
            tex(kit, stage, DP_TEX, SMALL_PT, ink, PRESSURE_AT[1])?,
            tex(kit, stage, UNITS_TEX, FINE_PT, dim, UNITS_AT)?,
        ];
        self.screen
            .show(stage, &[densities.clone(), rules.clone()].concat(), FADE)?;
        // The same pattern, twice as fast: four times the difference.
        kit.until_sentence(stage, "19_5", 3)?;
        let (fx, fy) = FLOW_AT;
        let flow_at = |len: f64| headed(&[(fx, fy), (fx + len, fy)], HEAD);
        let flow = add(
            stage,
            polyline(flow_at(FLOW_LEN)).style(line_style(ink, FLOW_W, 1.0)).build(),
            Z_MARK,
        );
        let label_at = |len: f64| (fx + len + 0.25, fy + 0.26);
        let speed = text_left(
            kit,
            stage,
            &squeeze_cells::speed_label(1),
            SMALL_PT,
            ink,
            label_at(FLOW_LEN),
        )?;
        self.screen.show(stage, &[flow, speed], FADE)?;
        until_phrase(kit, stage, t0, "19_5", 3, "run faster")?;
        let up = squeeze_cells::SPEED_UP as f64;
        let (hi2, lo2) = squeeze_cells::densities(1);
        let mut moves = vec![Morph {
            mob: flow,
            from: built(flow_at(FLOW_LEN)),
            to: built(flow_at(up * FLOW_LEN)),
        }];
        for ((pose, s), (a, b)) in CELLS.iter().zip(&stars).zip([(hi, hi2), (lo, lo2)]) {
            let (fa, fb) = (
                d2q9::equilibrium(a, (sh::int(0), sh::int(0))),
                d2q9::equilibrium(b, (sh::int(0), sh::int(0))),
            );
            moves.push(Morph {
                mob: s.rest,
                from: rest_points(pose, sh::to_f64(fa[0])),
                to: rest_points(pose, sh::to_f64(fb[0])),
            });
            for i in 1..9 {
                moves.push(Morph {
                    mob: s.arrows[i - 1],
                    from: built(star_arrow(pose, i, sh::to_f64(fa[i]))),
                    to: built(star_arrow(pose, i, sh::to_f64(fb[i]))),
                });
            }
        }
        morph(stage, &moves, EASE_T)?;
        let tints2 = tints_at(stage, 1)?;
        let densities2 = label_pair(stage, 1)?;
        let speed2 = text_left(
            kit,
            stage,
            &squeeze_cells::speed_label(squeeze_cells::SPEED_UP),
            SMALL_PT,
            ink,
            label_at(up * FLOW_LEN),
        )?;
        let faster = text(kit, stage, &squeeze_cells::ratio_label(), FINE_PT, ink, FASTER_AT)?;
        let out = [tints, densities, vec![speed]].concat();
        let into = [tints2, densities2.clone(), vec![speed2, faster]].concat();
        self.screen.swap(stage, &out, &into, FADE)?;
        let labels = [vec![no_puzzle, faster, speed2], first, densities2, rules].concat();
        refuse_touching(stage, &labels, "19_5")
    }

    // ---------------------------------------------------------------- 19_6

    pub fn cue_19_6(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Prediction> {
        kit.say(stage, "19_6")?;
        let card = Prediction::build(kit, stage)?;
        let shown = [card.parts.clone(), card.markers.clone(), vec![card.track, card.note]].concat();
        self.screen.show(stage, &shown, FADE)?;
        Ok(card)
    }

    // ---------------------------------------------------------------- 19_h1

    /// The question and 19_5's cells stay; the countdown runs 5 s on the ring and digits;
    /// nothing else moves, and the slider and the squeeze readout do not exist yet.
    pub fn hold_19_h1(&mut self, kit: &Kit, stage: &mut Stage<'_>, card: &Prediction) -> fmn::Result<()> {
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

    // ---------------------------------------------------------------- 19_7

    /// The slider's knob and the meter's bar from speed `from` to `to`, one frame at a time.
    fn slide(&mut self, stage: &mut Stage<'_>, knob: Mob, bar: Mob, from: sh::Q, to: sh::Q) -> fmn::Result<()> {
        let fps = stage.scene().time().fps();
        let frames = frames_covering(SLIDE_T, fps).max(1);
        let (u0, u1) = (sh::to_f64(from), sh::to_f64(to));
        let mut x = slider_x(u0);
        for k in 1..=frames {
            let u = u0 + (u1 - u0) * smooth(k as f64 / frames as f64);
            stage.shift(knob, v(slider_x(u) - x, 0.0));
            x = slider_x(u);
            let _ = stage.set_points(bar, Polygon::new(meter_corners(u)).build().points());
            wait_frames(stage, 1)?;
        }
        Ok(())
    }

    /// The readouts at a stop, shown together: the speed, the Mach number, the squeeze and, away
    /// from the example speed, how the stop compares with it.
    fn readouts(&mut self, kit: &Kit, stage: &mut Stage<'_>, u: sh::Q) -> fmn::Result<Vec<Mob>> {
        let (ink, orange) = (Palette::ink(), push_orange());
        let mut r = vec![
            text(kit, stage, &mach::speed_label(u), SMALL_PT, ink, READ_AT[0])?,
            text(kit, stage, &mach::ma_label(u), SMALL_PT, ink, READ_AT[1])?,
            tex(kit, stage, &mach::squeeze_tex(u), SMALL_PT, orange, SQUEEZE_AT)?,
        ];
        if u != mach::EXAMPLE {
            r.push(text(kit, stage, &mach::compare_label(u), FINE_PT, ink, NOTE7_AT)?);
        }
        self.screen.show(stage, &r, FADE)?;
        Ok(r)
    }

    pub fn cue_19_7(&mut self, kit: &Kit, stage: &mut Stage<'_>, card: Prediction) -> fmn::Result<()> {
        kit.say(stage, "19_7")?;
        let t0 = stage.scene().time().to_f64();
        let (ink, dim, faint, blue) = (Palette::ink(), Palette::dim(), Palette::faint(), Palette::blue());
        self.screen.show(stage, &[card.chosen], QUICK)?;
        pause(stage, QUICK)?;
        // The answer's picture: the channel, a speed slider and the squeeze meter.
        let (channel, channel_name) = build_channel(kit, stage)?;
        let (track, knob, ticks) = build_slider(kit, stage)?;
        let slider_name = text(kit, stage, SLIDER_NAME, FINE_PT, dim, SLIDER_NAME_AT)?;
        let meter_track = add(
            stage,
            rect(
                ((METER_X.0 + METER_X.1) / 2.0, METER_Y),
                (METER_X.1 - METER_X.0, METER_H),
                0.0,
                line_style(faint, DIM_W, 1.0),
            )?,
            Z_LINE,
        );
        let bar = add(
            stage,
            Polygon::new(meter_corners(0.0))
                .style(fill_style(push_orange(), 1.0))
                .build(),
            Z_MARK,
        );
        let meter_name = tex(kit, stage, METER_TEX, FINE_PT, ink, METER_NAME_AT)?;
        let out: Vec<Mob> = self.screen.on().iter().copied().filter(|&m| m != self.title).collect();
        let into = [
            channel,
            vec![channel_name, track, knob, slider_name, meter_track, bar, meter_name],
            ticks.clone(),
        ]
        .concat();
        self.screen.swap(stage, &out, &into, FADE)?;
        // The example speed, then twice it: four times the squeeze.
        kit.until_sentence(stage, "19_7", 1)?;
        self.slide(stage, knob, bar, (0, 1), mach::EXAMPLE)?;
        let first = self.readouts(kit, stage, mach::EXAMPLE)?;
        until_phrase(
            kit,
            stage,
            t0,
            "19_7",
            1,
            "the squeeze grows with the square of the speed",
        )?;
        self.screen.hide(stage, &first, QUICK)?;
        self.slide(stage, knob, bar, mach::EXAMPLE, mach::FASTER)?;
        let second = self.readouts(kit, stage, mach::FASTER)?;
        until_phrase(
            kit,
            stage,
            t0,
            "19_7",
            1,
            "measured against the lattice's own speed of sound",
        )?;
        let rule = vec![
            tex(kit, stage, MA_TEX, SMALL_PT, ink, READ_AT[2])?,
            text(kit, stage, MA_TAG, FINE_PT, dim, READ_AT[3])?,
        ];
        self.screen.show(stage, &rule, FADE)?;
        // Slower: half the example speed, a quarter of its squeeze.
        kit.until_sentence(stage, "19_7", 2)?;
        self.screen.hide(stage, &second, QUICK)?;
        self.slide(stage, knob, bar, mach::FASTER, mach::SLOWER)?;
        let third = self.readouts(kit, stage, mach::SLOWER)?;
        // Only a flow with no pressure differences needs no squeeze.
        until_phrase(kit, stage, t0, "19_7", 2, "wherever the pressure differs")?;
        let mut inset = vec![card_at(stage, INSET_AT, INSET_SIZE)?];
        for &y in &INSET_ARROW_Y {
            inset.push(arrow(
                stage,
                v(INSET_ARROW_X.0, y),
                v(INSET_ARROW_X.1, y),
                blue,
                FLOW_T,
            )?);
        }
        inset.push(text_left(kit, stage, UNIFORM_NAME, FINE_PT, ink, UNIFORM_AT)?);
        inset.push(tex_left(kit, stage, ZERO_TEX, SMALL_PT, Palette::teal(), ZERO_AT)?);
        inset.push(text_left(kit, stage, ANY_SPEED, FINE_PT, dim, ANY_SPEED_AT)?);
        self.screen.show(stage, &inset, FADE)?;
        until_phrase(kit, stage, t0, "19_7", 2, "it never becomes exactly unsqueezable")?;
        let near = tex(kit, stage, NEAR_TEX, FINE_PT, push_orange(), NEAR_AT)?;
        self.screen.show(stage, &[near], FADE)?;
        let labels = [
            vec![channel_name, slider_name, meter_name, near],
            third,
            rule,
            inset[4..].to_vec(),
        ]
        .concat();
        refuse_touching(stage, &labels, "19_7")
    }

    // ---------------------------------------------------------------- 19_8

    pub fn cue_19_8(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "19_8")?;
        let t0 = stage.scene().time().to_f64();
        let (ink, dim) = (Palette::ink(), Palette::dim());
        self.clear(stage, &[])?;
        let lbm = vec![
            card(stage, CARD8_AT[1], CARD8_SIZE)?,
            tex(kit, stage, LBM_TEX, SMALL_PT, ink, (CARD8_AT[1].0, CARD8_Y[0]))?,
            text(kit, stage, LBM_NOTE, FINE_PT, dim, (CARD8_AT[1].0, CARD8_Y[1]))?,
        ];
        self.screen.show(stage, &lbm, FADE)?;
        // Our method: no squeeze at all, down to the leftover imbalance.
        kit.until_sentence(stage, "19_8", 1)?;
        let projection = vec![
            card(stage, CARD8_AT[0], CARD8_SIZE)?,
            text(kit, stage, PROJ_TEXT, SMALL_PT, ink, (CARD8_AT[0].0, CARD8_Y[0]))?,
        ];
        self.screen.show(stage, &projection, FADE)?;
        until_phrase(
            kit,
            stage,
            t0,
            "19_8",
            2,
            "down to the leftover imbalance we met in chapter ten",
        )?;
        let note = text(kit, stage, PROJ_NOTE, FINE_PT, dim, (CARD8_AT[0].0, CARD8_Y[1]))?;
        self.screen.show(stage, &[note], FADE)?;
        // Two approximations, and one set of tests for both.
        until_phrase(kit, stage, t0, "19_8", 3, "wrong in different ways")?;
        let both = text(kit, stage, BOTH, FINE_PT, ink, BOTH_AT)?;
        self.screen.show(stage, &[both], FADE)?;
        until_phrase(kit, stage, t0, "19_8", 3, "both must pass the same tests")?;
        let mut bench = vec![card(stage, BENCH_AT, BENCH_SIZE)?];
        for feed in &FEED {
            bench.push(path_arrow(stage, feed, dim, 1.0));
        }
        bench.push(text(kit, stage, BENCH_NAME, SMALL_PT, ink, (BENCH_AT.0, BENCH_Y[0]))?);
        for (k, check) in GATE_CHECKS.iter().enumerate() {
            bench.push(text(
                kit,
                stage,
                check,
                SMALL_PT,
                check_color(k),
                (BENCH_X[k], BENCH_Y[1]),
            )?);
        }
        self.screen.show(stage, &bench, FADE)?;
        let labels = [
            lbm[1..].to_vec(),
            projection[1..].to_vec(),
            vec![note, both],
            bench[3..].to_vec(),
        ]
        .concat();
        refuse_touching(stage, &labels, "19_8")
    }

    // ---------------------------------------------------------------- 19_9

    pub fn cue_19_9(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "19_9")?;
        let t0 = stage.scene().time().to_f64();
        let (ink, dim, faint, blue) = (Palette::ink(), Palette::dim(), Palette::faint(), Palette::blue());
        self.clear(stage, &[])?;
        // A shear layer from its formula, with schematic small swirls along it.
        let (ox, oy) = SHEAR_AT;
        let mut flow = Vec::new();
        for r in 0..shear::ROWS {
            let y = shear::row_y(r);
            let u = shear::speed(y);
            if u.abs() < SHEAR_MIN {
                continue;
            }
            for c in 0..shear::COLS {
                let x = shear::col_x(c);
                let half = SHEAR_ARROW * u / 2.0;
                flow.push(path_arrow(
                    stage,
                    &[(ox + x - half, oy + y), (ox + x + half, oy + y)],
                    blue,
                    1.0,
                ));
            }
        }
        let swirls: Vec<Mob> = shear::SWIRLS
            .iter()
            .map(|&(x, y)| {
                let points = swirl_points((ox + x, oy + y), shear::SWIRL_R);
                add(
                    stage,
                    polyline(headed(&points, 0.09))
                        .style(line_style(ink, STAR_W, 0.9))
                        .build(),
                    Z_MARK,
                )
            })
            .collect();
        let names = vec![
            text(kit, stage, SHEAR_NAME, FINE_PT, ink, (SIDE_X9, SIDE_Y9[0]))?,
            text(kit, stage, SWIRLS_TAG, FINE_PT, dim, (SIDE_X9, SIDE_Y9[1]))?,
        ];
        self.screen
            .show(stage, &[flow, swirls.clone(), names.clone()].concat(), FADE)?;
        // Too coarse for them: the grid, then the swirls grey out.
        kit.until_sentence(stage, "19_9", 1)?;
        let (w, h) = (shear::COLS as f64 * shear::CELL.0, shear::ROWS as f64 * shear::CELL.1);
        let style = line_style(dim, DIM_W, 1.0);
        let mut lines = Vec::new();
        for c in 0..=shear::COLS {
            let x = ox - w / 2.0 + shear::CELL.0 * c as f64;
            lines.push(seg(v(x, oy - h / 2.0), v(x, oy + h / 2.0), style));
        }
        for r in 0..=shear::ROWS {
            let y = oy - h / 2.0 + shear::CELL.1 * r as f64;
            lines.push(seg(v(ox - w / 2.0, y), v(ox + w / 2.0, y), style));
        }
        let grid = group(stage, lines, Z_LINE);
        let coarse = text(kit, stage, COARSE_NOTE, FINE_PT, ink, (SIDE_X9, SIDE_Y9[2]))?;
        self.screen.show(stage, &[grid, coarse], FADE)?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &swirls {
            let a = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(FADE),
                    ..AnimateArgs::default()
                })?
                .set_stroke(Some(faint), None, Some(0.9))?;
            anims.push(stage.prepare(a)?);
        }
        stage.play_prepared(anims)?;
        // Extra viscosity where the shear is strongest: one violet tint per row.
        until_phrase(
            kit,
            stage,
            t0,
            "19_9",
            1,
            "they add extra viscosity where the flow shears hardest",
        )?;
        let mut tint = Vec::new();
        for r in 0..shear::ROWS {
            let at = (ox, oy + shear::row_y(r));
            let fill = fill_style(violet(), shear::tint(r));
            tint.push(add(stage, rect(at, (w, shear::CELL.1), 0.0, fill)?, Z_FACE));
        }
        tint.push(text(kit, stage, TINT_NOTE, FINE_PT, violet(), (SIDE_X9, SIDE_Y9[3]))?);
        self.screen.show(stage, &tint, FADE)?;
        // A model, not part of Navier–Stokes.
        kit.until_sentence(stage, "19_9", 2)?;
        let model = text(kit, stage, MODEL_LABEL, FINE_PT, ink, MODEL_LABEL_AT)?;
        self.screen.show(stage, &[model], FADE)?;
        until_phrase(
            kit,
            stage,
            t0,
            "19_9",
            2,
            "an extra assumption on top of the Navier-Stokes equations",
        )?;
        let formula = vec![
            tex(kit, stage, NU_T_TEX, SMALL_PT, violet(), (SIDE_X9, SIDE_Y9[4]))?,
            tex(kit, stage, NU_T_NOTE_TEX, FINE_PT, dim, (SIDE_X9, SIDE_Y9[5]))?,
        ];
        self.screen.show(stage, &formula, FADE)?;
        until_phrase(kit, stage, t0, "19_9", 3, "but it doesn't bring back their detail")?;
        let detail = text(kit, stage, DETAIL_NOTE, FINE_PT, dim, DETAIL_AT)?;
        self.screen.show(stage, &[detail], FADE)?;
        let labels = [names, vec![coarse, tint[tint.len() - 1], model, detail], formula].concat();
        refuse_touching(stage, &labels, "19_9")
    }

    // ---------------------------------------------------------------- 19_10

    pub fn cue_19_10(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "19_10")?;
        let t0 = stage.scene().time().to_f64();
        let (ink, dim, faint) = (Palette::ink(), Palette::dim(), Palette::faint());
        self.clear(stage, &[])?;
        // A neutral card: our own outline on a lattice, no flow.
        let mut shown = vec![card(stage, CAR_CARD_AT, CAR_CARD_SIZE)?];
        let [x0, x1, y0, y1] = LATTICE;
        let style = line_style(faint, GRID_W, 1.0);
        let mut lines = Vec::new();
        let (nx, ny) = (
            ((x1 - x0) / LATTICE_STEP).round() as usize,
            ((y1 - y0) / LATTICE_STEP).floor() as usize,
        );
        for k in 0..=nx {
            let x = x0 + LATTICE_STEP * k as f64;
            lines.push(seg(v(x, y0), v(x, y1), style));
        }
        for k in 0..=ny {
            let y = y0 + LATTICE_STEP * k as f64;
            lines.push(seg(v(x0, y), v(x1, y), style));
        }
        shown.push(group(stage, lines, Z_TEXT));
        let car = build_outline(stage, &CAR_POSE)?;
        stage.arena_mut().set_z_index(car, Z_TEXT + 1, true);
        shown.push(car);
        let note = text(kit, stage, CARD_NOTE, FINE_PT, dim, CARD_NOTE_AT)?;
        let permanent = text(kit, stage, PERMANENT, SMALL_PT, ink, PERMANENT_AT)?;
        shown.extend([note, permanent]);
        self.screen.show(stage, &shown, FADE)?;
        until_phrase(kit, stage, t0, "19_10", 1, "still not be a measurement of a real car")?;
        let credit = text(kit, stage, CREDIT, FINE_PT, dim, CREDIT_AT)?;
        self.screen.show(stage, &[credit], FADE)?;
        refuse_touching(stage, &[note, permanent, credit], "19_10")
    }
}

/// A card above the picture (19_7's inset).
fn card_at(stage: &mut Stage<'_>, at: (f64, f64), size: (f64, f64)) -> fmn::Result<Mob> {
    let m = card(stage, at, size)?;
    stage.arena_mut().set_z_index(m, Z_MARK - 1, true);
    Ok(m)
}

/// The slider's x for speed `u` (cells per step).
fn slider_x(u: f64) -> f64 {
    SLIDER_X.0 + (SLIDER_X.1 - SLIDER_X.0) * u / sh::to_f64(mach::TOP)
}

/// The meter's bar at speed `u`: its length is the illustrative squeeze ½Ma², full at
/// [`mach::TOP`].
fn meter_corners(u: f64) -> Vec<Vec3> {
    let share = (u / sh::to_f64(mach::TOP)).powi(2);
    let x = METER_X.0 + ((METER_X.1 - METER_X.0) * share).max(0.004);
    let (b, t) = (METER_Y - METER_H / 2.0, METER_Y + METER_H / 2.0);
    vec![v(METER_X.0, b), v(x, b), v(x, t), v(METER_X.0, t)]
}

/// A small swirl: most of a circle, clockwise (the shear layer's own sense of spin, since its
/// speed grows upward), as a point run.
fn swirl_points(c: (f64, f64), r: f64) -> Vec<(f64, f64)> {
    let n = 20;
    (0..=n)
        .map(|k| {
            let a = FRAC_PI_2 - 0.85 * TAU * k as f64 / n as f64;
            (c.0 + r * a.cos(), c.1 + r * a.sin())
        })
        .collect()
}

/// Chapter 17's channel, walls and cylinder, at 17_8's placing, and its name.
fn build_channel(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<(Vec<Mob>, Mob)> {
    let (l, h, d) = (channel::LENGTH, channel::HEIGHT, channel::DIAMETER);
    let (cx, cy) = channel::CENTRE;
    let (ink, dim) = (Palette::ink(), Palette::dim());
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
    let name = text_left(kit, stage, CHANNEL_NAME, FINE_PT, dim, CHANNEL_NAME_AT)?;
    Ok((vec![walls, cylinder], name))
}

/// The slider: its track, its knob at speed 0, and the ticks with their speeds.
fn build_slider(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<(Mob, Mob, Vec<Mob>)> {
    let dim = Palette::dim();
    let track = add(
        stage,
        seg(
            v(SLIDER_X.0, SLIDER_Y),
            v(SLIDER_X.1, SLIDER_Y),
            line_style(dim, WALL_W, 1.0),
        ),
        Z_LINE,
    );
    let knob = dot(stage, v(slider_x(0.0), SLIDER_Y), KNOB_R, Palette::hero());
    let mut ticks = Vec::new();
    for u in mach::TICKS {
        let x = slider_x(sh::to_f64(u));
        ticks.push(add(
            stage,
            seg(v(x, SLIDER_Y - 0.1), v(x, SLIDER_Y + 0.1), line_style(dim, DIM_W, 1.0)),
            Z_LINE,
        ));
        ticks.push(text(kit, stage, &mach::tick_label(u), FINE_PT, dim, (x, TICK_Y))?);
    }
    Ok((track, knob, ticks))
}

/// Chapter 18's outline at `pose`: the body, filled like chapter 17's cylinder, and its two wheels.
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
    Ok(group(stage, parts, Z_TEXT))
}

// --------------------------------------------------------------- the cues

/// Cues 19_1 to 19_10 and the 19_h1 hold, in order, from a clear screen. Returns the chapter
/// with 19_10's picture on screen.
pub fn chapter19(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Lattice19> {
    let mut ch = Lattice19::build(kit, stage)?;
    ch.cue_19_1(kit, stage)?;
    ch.cue_19_2(kit, stage)?;
    ch.cue_19_3(kit, stage)?;
    ch.cue_19_4(kit, stage)?;
    ch.cue_19_5(kit, stage)?;
    let card = ch.cue_19_6(kit, stage)?;
    ch.hold_19_h1(kit, stage, &card)?;
    ch.cue_19_7(kit, stage, card)?;
    ch.cue_19_8(kit, stage)?;
    ch.cue_19_9(kit, stage)?;
    ch.cue_19_10(kit, stage)?;
    Ok(ch)
}

/// The chapter as root's crate registers its chapters (`story.rs`: `make: |kit|
/// Box::new(Chapter19 { kit })`).
pub struct Chapter19 {
    pub kit: Kit,
}

impl SceneConstruct for Chapter19 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        chapter19(&self.kit, stage)?;
        self.kit.finish(stage)
    }
}

/// Every string the chapter sets as text, and every TeX source, for a glyph probe.
#[cfg(test)]
pub fn glyph_strings() -> (Vec<String>, Vec<String>) {
    let mut text: Vec<String> = [
        TITLE,
        HERO_NAME,
        MAC_NAME,
        MAC_NOTE,
        GRID_NAME,
        PATCH_TAG,
        STREAM_NAME,
        COLLIDE_NAME,
        GHOST_NOTE,
        BALANCED_NOTE,
        RULES_TAG,
        DENSITY_NAME,
        MOMENTUM_NAME,
        BAR_NAME,
        MOMENTUM_LABEL,
        DEFS_TAG,
        TG_NAME,
        NU_TAG,
        AMPLITUDE,
        TIME,
        XR_LABEL,
        NO_PUZZLE,
        EXAGGERATED,
        Q_LABEL,
        PAUSE_NOTE,
        CHANNEL_NAME,
        SLIDER_NAME,
        MA_TAG,
        UNIFORM_NAME,
        ANY_SPEED,
        PROJ_TEXT,
        PROJ_NOTE,
        LBM_NOTE,
        BOTH,
        BENCH_NAME,
        SHEAR_NAME,
        SWIRLS_TAG,
        COARSE_NOTE,
        TINT_NOTE,
        MODEL_LABEL,
        DETAIL_NOTE,
        CARD_NOTE,
        PERMANENT,
        CREDIT,
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    text.extend(
        SAYS.iter()
            .chain(&QUESTION_LINES)
            .chain(&CHOICES)
            .chain(&GATE_CHECKS)
            .map(|s| s.to_string()),
    );
    text.extend((1..=HOLD_SECONDS as u32).map(|s| s.to_string()));
    text.extend(hero::AMOUNTS.iter().map(|a| a.to_string()));
    text.extend([
        hero::sum_label(),
        hero::axis_label(0),
        hero::axis_label(1),
        hero::velocity_label(),
        hero::amounts_tag(),
        mach::compare_label(mach::FASTER),
        mach::compare_label(mach::SLOWER),
        taylor_green::proto_label(),
        taylor_green::ratio_label(),
        taylor_green::within_label(),
        squeeze_cells::speed_label(1),
        squeeze_cells::speed_label(squeeze_cells::SPEED_UP),
        squeeze_cells::ratio_label(),
    ]);
    for u in [mach::EXAMPLE, mach::FASTER, mach::SLOWER] {
        text.extend([mach::speed_label(u), mach::ma_label(u)]);
    }
    text.extend(mach::TICKS.map(mach::tick_label));
    let mut tex: Vec<String> = [
        STREAM_TEX,
        COLLIDE_TEX,
        DENSITY_TEX,
        MOMENTUM_TEX,
        NU_TEX,
        DECAY_TEX,
        PRESSURE_TEX,
        DP_TEX,
        UNITS_TEX,
        MA_TEX,
        METER_TEX,
        ZERO_TEX,
        NEAR_TEX,
        LBM_TEX,
        NU_T_TEX,
        NU_T_NOTE_TEX,
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    tex.extend([patch::tau_tex(), taylor_green::nu_tex()]);
    for k in 0..2 {
        let (hi, lo) = squeeze_cells::densities(k);
        tex.extend([squeeze_cells::density_tex(hi), squeeze_cells::density_tex(lo)]);
    }
    tex.extend([mach::EXAMPLE, mach::FASTER, mach::SLOWER].map(mach::squeeze_tex));
    (text, tex)
}

/// The spoken lines, verbatim from `navier-full-script03.md` chapter 19 (`tools/check_lattice01.py`
/// compares them byte for byte), as `script.tsv` rows. 19_h1 is a hold, not a line.
pub const SCRIPT19: &str = concat!(
    "19_1\tThere's a quite different way to build a fluid simulator, called lattice Boltzmann. Instead of velocity and pressure, each cell keeps nine numbers.\n",
    "19_2\tEach one says how much fluid in the cell is moving one way, standing still, toward a side, or toward a corner. Every step, each moving amount hops exactly one cell in its own direction. Then, inside each cell, the nine amounts relax toward a balanced mix.\n",
    "19_3\tAdd up a cell's nine amounts, and you get its density. Weight each by its direction, and you get its momentum. So this method reaches the same kind of flow, through different numbers.\n",
    "19_4\tHow quickly the amounts relax sets the viscosity. We checked this with a small lattice Boltzmann program, separate from the solver this film builds. A decaying pattern of swirls ran on a grid of one hundred twenty-eight cells each way. It faded at the predicted rate, to within six thousandths of a percent.\n",
    "19_5\tHere's the catch. There's no pressure puzzle. Pressure is simply a third of the density, in the lattice's units, so pressure differences come from squeezing the fluid a little. And the same flow pattern, run faster, comes with bigger pressure differences.\n",
    "19_6\tSo which flows will lattice Boltzmann handle most like our unsqueezable fluid, slow ones or fast ones?\n",
    "19_7\tSlow ones. For the same flow pattern, the squeeze grows with the square of the speed, measured against the lattice's own speed of sound. So the slower the flow, the less it squeezes, but wherever the pressure differs, as it does around our cylinder, it never becomes exactly unsqueezable.\n",
    "19_8\tOur method works the other way round. Its model fluid can't be squeezed at all. Each step, the pressure puzzle removes the squeeze, down to the leftover imbalance we met in chapter ten. These are two approximations of the same slow flow, wrong in different ways, and both must pass the same tests.\n",
    "19_9\tOne more ingredient appears in some programs. When the grid is too coarse for the smallest swirls, they add extra viscosity where the flow shears hardest, to stand in for what's missing. That's a model, an extra assumption on top of the Navier-Stokes equations. It guesses the missing swirls' average effect, but it doesn't bring back their detail.\n",
    "19_10\tSo a striking flat animation of a car, built this way, is an illustration. It can be honest and beautiful, and still not be a measurement of a real car.\n",
);

#[cfg(test)]
mod tests {
    use super::*;
    use explainer_kit::{Narrator, Script};

    const IDS: [&str; 10] = [
        "19_1", "19_2", "19_3", "19_4", "19_5", "19_6", "19_7", "19_8", "19_9", "19_10",
    ];

    fn kit() -> Kit {
        let script = Script::parse(SCRIPT19).unwrap();
        Kit::new().unwrap().with_narrator(Narrator::estimate(&script))
    }

    #[test]
    fn script_has_the_ten_lines_in_order() {
        let script = Script::parse(SCRIPT19).unwrap();
        let ids: Vec<&str> = script.lines().iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(ids, IDS);
        let kit = kit();
        let counts: Vec<usize> = ids.iter().map(|id| kit.sentence_starts(id).len()).collect();
        assert_eq!(counts, [2, 3, 3, 4, 4, 1, 3, 4, 4, 2]);
        for row in SCRIPT19.lines() {
            let (id, line) = row.split_once('\t').unwrap();
            assert_eq!(sentences(line).len(), kit.sentence_starts(id).len(), "{id}");
        }
    }

    #[test]
    fn phrases_sit_in_their_sentences() {
        // rustfmt sets a long call one argument per line: read the source with its whitespace
        // collapsed.
        let src = include_str!("lattice19_01.rs")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .replace("( ", "(");
        let mut gates = 0;
        for call in src.split("until_phrase(kit, stage, t0, \"").skip(1) {
            let (id, rest) = call.split_once('"').unwrap();
            let (j, arg) = rest[2..].split_once(", ").unwrap();
            let j: usize = j.parse().unwrap();
            let phrase = arg.strip_prefix('"').unwrap().split('"').next().unwrap();
            assert!(phrase_share(id, j, phrase).is_some(), "{id} {j} {phrase}");
            gates += 1;
        }
        assert_eq!(gates, 22);
    }

    #[test]
    fn card_and_labels_come_from_the_data() {
        assert_eq!(QUESTION_LINES.join(" "), QUESTION);
        assert_eq!(CHOICES[ANSWER], "slow");
        let (text, tex) = glyph_strings();
        assert!(text.contains(&"measured rate / predicted = 1.00006".to_owned()));
        assert!(text.contains(&"within 0.006%".to_owned()));
        assert!(text.contains(&"diagnostic prototype (numpy), not the film's solver; 128 × 128".to_owned()));
        assert!(tex.contains(&r"\Delta\rho/\rho \approx 0.004".to_owned()));
        assert_eq!(text.iter().filter(|s| s.contains("measured")).count(), 1);
        assert!(text.iter().chain(&tex).all(|s| !s.to_lowercase().contains("validat")));
    }

    #[test]
    fn layout_stays_in_frame() {
        for (at, size) in [
            (Q_CARD_AT, Q_CARD_SIZE),
            (CARD8_AT[0], CARD8_SIZE),
            (CARD8_AT[1], CARD8_SIZE),
            (BENCH_AT, BENCH_SIZE),
            (INSET_AT, INSET_SIZE),
            (CAR_CARD_AT, CAR_CARD_SIZE),
        ] {
            assert!(at.0 - size.0 / 2.0 >= -7.6 && at.0 + size.0 / 2.0 <= 7.6);
            assert!(at.1 - size.1 / 2.0 >= -4.4 && at.1 + size.1 / 2.0 <= 3.6);
        }
        // the patch, with the amounts sliding in from one cell beyond it
        // the amounts sliding in start in the inner half of the ring of cells around the window
        let reach = (patch::HALF as f64 + 1.0) * PATCH.scale;
        assert!(PATCH.origin.0 - reach >= -7.6 && PATCH.origin.1 + reach <= 3.7);
        // the meter at full speed, and the knob's whole travel
        assert!((meter_corners(sh::to_f64(mach::TOP))[1][0] - METER_X.1).abs() < 1e-12);
        assert!((slider_x(sh::to_f64(mach::TOP)) - SLIDER_X.1).abs() < 1e-12);
        // the hold's ring sits inside its card, clear of the label row
        let (rx, ry) = Prediction::ring_at();
        assert!(
            rx + RING_R < Q_CARD_AT.0 + Q_CARD_SIZE.0 / 2.0 && ry + RING_R < Q_CARD_AT.1 + Q_CARD_SIZE.1 / 2.0 - 0.4
        );
        // the car on its card
        for p in car::outline() {
            let (x, y) = CAR_POSE.at(p);
            assert!(x.abs() < CAR_CARD_SIZE.0 / 2.0 - 0.3 && y > CAR_CARD_AT.1 - CAR_CARD_SIZE.1 / 2.0);
        }
    }

    #[test]
    fn the_chain_ends_at_the_momentum() {
        let f = hero::pops();
        let links = chain(&f);
        let end = links[links.len() - 1].2;
        let (mx, my) = d2q9::momentum(&f);
        let k = star::ARROW * CHAIN_SCALE * HERO.scale;
        assert!((end.0 - CHAIN_AT.0 - k * sh::to_f64(mx)).abs() < 1e-12);
        assert!((end.1 - CHAIN_AT.1 - k * sh::to_f64(my)).abs() < 1e-12);
        let top = bar_segment(&f, 8)[1][1];
        assert!((top - BAR_BASE - BAR_SCALE * sh::to_f64(d2q9::density(&f))).abs() < 1e-12);
    }
}
