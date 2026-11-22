//! Chapter 17, "Are we solving the right problem?": cues 17_1 to 17_7 and the 17_h1 hold,
//! native and exact. Nothing is simulated.
//!
//! RubyRaven `ruby-boundary01` for root's `navier_stokes_simulation` crate (Agent Mail 36212).
//! Cue text and pictures: `navier-full-script03.md` chapter 17 (sha256 31a83a2b…), whose
//! chapters 17 to 20 are BronzeJaguar's root-canonical closing02 with root's exact edits.
//! Geometry: [`super::staircase01`], which reproduces the reviewed cl01 facts
//! (`navier-closing-scene02/runs/cl01-run01.txt`) in its own tests, and
//! `tools/check_boundary01.py`, which recomputes them independently in exact fractions.
//!
//! The pictures, in the order the lines ask for them:
//! - 17_1: the circle over an 8 × 8 block, its centre on a grid corner (a white cross), each
//!   cell's middle a dot, the 52 solid cells grey, then the gold staircase and the rule card;
//! - 17_2: the 16- and 32-cell staircases in turn, then a magnifier on the 32-cell corner with
//!   the largest gap (0.643 cell widths), labelled "≤ 0.71 cell widths";
//! - 17_3: a white dot walks the circle ("≈ 3.14 D"), the staircase glows, the prediction card;
//! - 17_h1: the 5 s countdown on the card;
//! - 17_4: the answer, then each straight piece of the 8-cell outline slides to its side of the
//!   1 × 1 box and the pieces form the square; the same slide at 16, 32, 64 and 128 cells, with
//!   "4 D" each time; then axis normals on the staircase and radial normals on the circle;
//! - 17_5: the 1996 source card, a schematic grid shaped to the wall, then the 8-cell grid with
//!   each cut cell shaded by its exact solid share, each crossed face showing its open part,
//!   and the karman.c card;
//! - 17_6: the typed circle dashed, D/40 inside the true one; its staircases at 32, 64 and 128
//!   cells, with a magnifier on the 32-cell corner nearest the true wall;
//! - 17_7: the ST96 channel to scale with its dimensions, the 1 cm difference marked.
//!
//! Root owns 16_10's hand-off into 17_1 and everything from 17_8. The channel stays on screen
//! at the end of 17_7 for 17_8.

use std::f64::consts::{FRAC_PI_2, PI, TAU};

use super::staircase01::{
    CutCell, LEVELS, Staircase, TRUE_RADIUS, TYPED_RADIUS, WRONG_LEVELS, Window, arc_points, arc_runs, channel,
    clip_axis_segment, cut_cells, cut_faces, rightmost_lowest,
};
use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Palette, Timed, hex};
use fmn::prelude::*;

// ------------------------------------------------------------------ layout

/// The circle's centre on screen and one diameter in scene units, fixed from 17_1 to 17_6.
/// The 1 × 1 box spans x −6.2 to −0.6 and y −3.0 to 2.6 (frame: x ±8, y ±4.5).
pub const FIG: (f64, f64) = (-3.4, -0.2);
pub const FIG_D: f64 = 5.6;
/// The right-hand panel's centre line.
pub const PANEL_X: f64 = 4.1;

pub const TITLE: &str = "Are we solving the right problem?";
pub const TITLE_AT: (f64, f64) = (-7.4, 3.95);

/// 17_1's rule card (fact stair.n8 picture), two lines.
pub const RULE: [&str; 2] = ["centre on a grid corner;", "solid if its middle is inside"];
pub const RULE_AT: (f64, f64) = (PANEL_X, 2.75);
pub const RULE_SIZE: (f64, f64) = (6.4, 1.2);
/// The level readout: cells per diameter, then solid cells.
pub const LEVEL_AT: [(f64, f64); 2] = [(PANEL_X, 1.85), (PANEL_X, 1.45)];

/// The magnifier's inset, scene units: centre and half side.
pub const INSET_AT: (f64, f64) = (3.3, -1.1);
pub const INSET_HALF: f64 = 1.3;
/// The lens: a window 1.5 cells on each side of the corner, at 32 cells per diameter (exact
/// in binary: 3/64 D).
pub const LENS_HALF_CELLS: f64 = 1.5;
pub const INSET_LABEL_AT: [(f64, f64); 2] = [(INSET_AT.0, -2.85), (INSET_AT.0, -3.3)];
/// 17_2's labels: the bound from the script (h/√2 ≈ 0.7071 h), and the largest gap of the
/// 32-cell staircase, computed ([`gap_here`]: 0.643, cl01 stair.n32).
pub const GAP_BOUND: &str = "≤ 0.71 cell widths";

/// 17_3's readouts, below the card.
pub const TRIP_AT: [(f64, f64); 2] = [(PANEL_X, -3.0), (PANEL_X, -3.55)];
pub const TRIP: [&str; 2] = ["around the circle: ≈ 3.14 D", "around the staircase: ?"];

/// The 17_h1 card (holds table): label, question, setup, choices.
pub const CARD_AT: (f64, f64) = (PANEL_X, -0.6);
pub const CARD_SIZE: (f64, f64) = (6.6, 3.6);
pub const CARD_LABEL: &str = "your prediction";
pub const QUESTION: [&str; 2] = ["Halve the cells: what happens", "to the staircase outline's length?"];
pub const SETUP: &str = "centre on a grid corner; cells halved each time";
pub const CHOICES: [&str; 3] = ["gets closer to 3.14 diameters", "stays the same", "keeps growing"];
/// The answer, "stays the same" (17_4).
pub const ANSWER: usize = 1;
pub const PAUSE_NOTE: &str = "pause for more time";
pub const HOLD_SECONDS: f64 = 5.0;
pub const RING_R: f64 = 0.3;

/// 17_4's rows, one per level, and the comparison and normals legend.
pub const ROWS_TOP: f64 = 1.85;
pub const ROW_STEP: f64 = 0.5;
pub const RATIO: [&str; 2] = ["4 D against π D ≈ 3.14 D", "4/π ≈ 1.27: about a quarter too long"];
pub const RATIO_Y: [f64; 2] = [-0.9, -1.35];
pub const NORMALS: [&str; 2] = ["staircase: up, down, left or right", "real wall: every direction"];
pub const NORMALS_Y: [f64; 2] = [-2.2, -2.65];

/// 17_5's source card (ST96 conclusion 5, fact st.concl.curved), wrapped by hand.
pub const QUOTE: [&str; 4] = [
    "\"The use of only first order upwinding",
    "(or crude approximation of curved boundaries)",
    "does not lead to satisfactory accuracy even",
    "on very fine meshes (several million unknowns in 2D).\"",
];
/// "Schäfer" is spelt "Schaefer" on screen: no film here has drawn "ä" (glyph probe pending).
pub const QUOTE_SOURCE: &str = "Schaefer and Turek (1996), conclusion 5";
pub const QUOTE_AT: (f64, f64) = (PANEL_X, 1.6);
pub const QUOTE_SIZE: (f64, f64) = (6.8, 2.9);
/// The schematic grid shaped to the wall (not to scale, not a real grid; fact st.concl.contour).
pub const FITTED_AT: (f64, f64) = (PANEL_X, -1.6);
pub const FITTED_RINGS: [f64; 5] = [0.4, 0.55, 0.75, 1.0, 1.3];
pub const FITTED_SPOKES: usize = 24;
pub const FITTED_LABEL: [&str; 2] = [
    "grid shaped to the wall (schematic)",
    "conclusion 3: contour adapted grids",
];
pub const FITTED_LABEL_Y: [f64; 2] = [-3.25, -3.65];
/// The fractional-cell picture's card (fact bas.karman.solid) and legend.
pub const KARMAN: [&str; 2] = ["Basilisk karman.c:", "solid (cs, fs, …)"];
/// The script's "Precisely" for 17_5: "cs is the fluid fraction of each cell."
pub const KARMAN_NOTE: &str = "cs: the fluid fraction of each cell";
pub const KARMAN_Y: [f64; 3] = [2.2, 1.7, 1.1];
pub const CUT_LEGEND: [&str; 2] = [
    "grey: each crossed cell's solid share",
    "teal: the open part of each crossed face",
];
pub const CUT_LEGEND_Y: [f64; 2] = [-0.2, -0.65];

/// 17_6's readouts (facts wrong.radius.gap, wrong.stays.inside).
pub const WRONG: [&str; 2] = ["typed diameter −5%", "typed circle to true wall: D/40, at every grid"];
pub const WRONG_Y: [f64; 2] = [2.6, 2.1];
pub const WRONG_LEVEL_Y: f64 = 1.45;
pub const NEAREST: &str = "nearest corner: still inside the true wall";

/// 17_7's channel: metres to scene units, and the inlet's lower corner on screen. The channel
/// spans x −6.6 to 6.6 and y −1.5 to 0.96.
pub const CH_SCALE: f64 = 6.0;
pub const CH_ORIGIN: (f64, f64) = (-6.6, -1.5);
pub const CH_LENGTH_LABEL: &str = "2.2 m";
pub const CH_HEIGHT_LABEL: &str = "0.41 m";
pub const CH_GAP_LABELS: [&str; 3] = ["0.15 m", "D 0.1 m", "0.16 m"];
pub const CH_DIFFERENCE: &str = "1 cm more above than below";
/// Where that label starts, right of the gap labels inside the channel, with a leader to the
/// mark.
pub const CH_DIFFERENCE_AT: (f64, f64) = (-3.0, 0.62);

pub const TITLE_PT: f64 = 40.0;
pub const CARD_PT: f64 = 30.0;
pub const BODY_PT: f64 = 28.0;
pub const SMALL_PT: f64 = 24.0;
pub const FINE_PT: f64 = 22.0;

// Stroke widths (fmn units: 0.01 scene units each, round caps).
// At 4K (240 px per unit), 1.0 is 2.4 px. The 32-cell typed staircase comes within 0.0059 D
// (7.9 px) of the true circle; widths 1.5 and 2.0 leave 3.7 px clear (check_boundary01.py).
const GRID_W: f64 = 1.5;
const CIRCLE_W: f64 = 2.0;
const OUTLINE_W: [f64; 5] = [5.0, 4.0, 3.0, 2.0, 1.5];
const TYPED_OUTLINE_W: f64 = 1.5;
const HALO_W: f64 = 16.0;
const LENS_W: f64 = 2.0;
const MARK_W: f64 = 4.0;
const DIM_W: f64 = 2.0;
const WALL_W: f64 = 4.0;

// Draw order.
const Z_GRID: i32 = 0;
const Z_FILL: i32 = 1;
const Z_DOTS: i32 = 2;
const Z_CIRCLE: i32 = 3;
const Z_OUTLINE: i32 = 4;
const Z_MARK: i32 = 5;
const Z_INSET: i32 = 6;
const Z_TEXT: i32 = 8;

// Timing, seconds; each value is exact in binary or a whole number of frames at 30 and 60 fps.
const FADE: f64 = 0.5;
const QUICK: f64 = 0.25;
const DRAW: f64 = 1.25;
const WALK: f64 = 3.0;
const SLIDE_FIRST: f64 = 1.5;
const SLIDE_AGAIN: f64 = 0.75;
const LOOK: f64 = 1.25;

/// The background, as the crate's main.rs sets it (lenses are opaque so the figure under
/// them stays hidden).
pub const BACKGROUND: &str = "#0A121F";

fn solid_grey() -> Srgb {
    hex("#3B4352")
}

fn grid_ink() -> Srgb {
    hex("#2A3546")
}

fn card_fill() -> Srgb {
    hex("#141C29")
}

fn white() -> Srgb {
    hex("#FFFFFF")
}

// ----------------------------------------------------------------- helpers

fn v(x: f64, y: f64) -> Vec3 {
    [x, y, 0.0]
}

/// Diameters (the circle's centre at the origin) to scene units.
pub fn fig(p: (f64, f64)) -> Vec3 {
    v(FIG.0 + FIG_D * p.0, FIG.1 + FIG_D * p.1)
}

/// A staircase corner in cell units to scene units.
fn cell_pt(s: &Staircase, (x, y): (i64, i64)) -> Vec3 {
    fig((x as f64 / s.n as f64, y as f64 / s.n as f64))
}

fn scene_error(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

/// The point of the circle of radius `r` on the ray from the centre through `p`.
fn towards_circle(p: (f64, f64), r: f64) -> (f64, f64) {
    let d = p.0.hypot(p.1);
    (p.0 * r / d, p.1 * r / d)
}

fn line_style(color: Srgb, width: f64, opacity: f64) -> Style {
    Style::default().fill(color, 0.0).stroke(color, width, opacity)
}

fn fill_style(color: Srgb, opacity: f64) -> Style {
    Style::default().fill(color, opacity).stroke(color, 0.0, opacity)
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
    let m = kit.text(stage, s, size, color, v(at.0, at.1))?;
    stage.arena_mut().set_z_index(m, Z_TEXT, true);
    Ok(m)
}

fn text_left(kit: &Kit, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: (f64, f64)) -> fmn::Result<Mob> {
    let m = kit.text_left(stage, s, size, color, v(at.0, at.1))?;
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
        .moved_to(v(at.0, at.1));
    Ok(add(stage, back, Z_TEXT - 1))
}

/// What is left of a countdown ring, drained clockwise from the top.
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

/// Mobjects on screen. Every fade goes through it, so a hide never re-roots a mobject that is
/// already gone (the double-fade flash) and a show never fades in a mobject twice.
#[derive(Default)]
pub struct Shown(Vec<Mob>);

impl Shown {
    /// Fade `out` away and `into` in together, skipping any already in the wanted state.
    pub fn swap(&mut self, stage: &mut Stage<'_>, out: &[Mob], into: &[Mob], rt: f64) -> fmn::Result<()> {
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
                let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
                anims.push(stage.prepare(a.rt(rt))?);
            }
        }
        if !anims.is_empty() {
            stage.play_prepared(anims)?;
        }
        Ok(())
    }

    pub fn show(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
        self.swap(stage, &[], mobs, rt)
    }

    pub fn hide(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
        self.swap(stage, mobs, &[], rt)
    }

    /// Draw `mobs` along their paths together.
    pub fn create(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in mobs {
            if !self.0.contains(&m) {
                self.0.push(m);
                anims.push(stage.prepare(show_creation(m).rt(rt))?);
            }
        }
        if !anims.is_empty() {
            stage.play_prepared(anims)?;
        }
        Ok(())
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
            stage.remove_from_scene(m);
        }
    }
}

// --------------------------------------------------------- the staircases

/// One staircase as drawn: its grey cells, its gold outline, and optionally its grid and the
/// cell middles.
pub struct Drawn {
    pub stair: Staircase,
    pub fill: Mob,
    pub outline: Mob,
    pub grid: Option<Mob>,
    pub dots: Option<Mob>,
}

impl Drawn {
    pub fn build(stage: &mut Stage<'_>, stair: Staircase, width: f64, grid: bool, dots: bool) -> fmn::Result<Self> {
        let corners: Vec<Vec3> = stair.polygon().iter().map(|&p| cell_pt(&stair, p)).collect();
        let fill = add(
            stage,
            Polygon::new(corners.clone())
                .style(fill_style(solid_grey(), 1.0))
                .build(),
            Z_FILL,
        );
        let outline = add(
            stage,
            Polygon::new(corners)
                .style(line_style(Palette::gold(), width, 1.0))
                .build(),
            Z_OUTLINE,
        );
        let grid = grid.then(|| box_grid(stage, stair.n));
        let dots = dots.then(|| middles(stage, &stair));
        Ok(Self {
            stair,
            fill,
            outline,
            grid,
            dots,
        })
    }

    /// Everything this drawing puts on screen, grid first.
    pub fn all(&self) -> Vec<Mob> {
        self.grid
            .iter()
            .copied()
            .chain([self.fill])
            .chain(self.dots)
            .chain([self.outline])
            .collect()
    }
}

/// The cell lines of the 1 × 1 box at `n` cells per diameter.
fn box_grid(stage: &mut Stage<'_>, n: i64) -> Mob {
    let style = line_style(grid_ink(), GRID_W, 1.0);
    let mut lines = Vec::new();
    for k in 0..=n {
        let t = -0.5 + k as f64 / n as f64;
        lines.push(seg(fig((t, -0.5)), fig((t, 0.5)), style));
        lines.push(seg(fig((-0.5, t)), fig((0.5, t)), style));
    }
    group(stage, lines, Z_GRID)
}

/// A dot at every cell middle of the box: ink inside the circle (solid), dim outside.
fn middles(stage: &mut Stage<'_>, s: &Staircase) -> Mob {
    let n = s.n;
    let mut dots = Vec::new();
    for i in -n / 2..n / 2 {
        for j in -n / 2..n / 2 {
            let p = fig(((2 * i + 1) as f64 / (2 * n) as f64, (2 * j + 1) as f64 / (2 * n) as f64));
            let color = if s.solid(i, j) { Palette::ink() } else { Palette::dim() };
            dots.push(Dot::new().point(p).radius(0.04).style(fill_style(color, 1.0)).build());
        }
    }
    group(stage, dots, Z_DOTS)
}

/// The outline's straight pieces, each its own mobject so 17_4 can slide it.
fn pieces(stage: &mut Stage<'_>, s: &Staircase, width: f64) -> Vec<(Mob, (f64, f64))> {
    s.edges()
        .iter()
        .map(|e| {
            let m = add(
                stage,
                seg(
                    cell_pt(s, e.a),
                    cell_pt(s, e.b),
                    line_style(Palette::gold(), width, 1.0),
                ),
                Z_OUTLINE,
            );
            let (dx, dy) = s.slide(e);
            (m, (FIG_D * dx as f64 / s.n as f64, FIG_D * dy as f64 / s.n as f64))
        })
        .collect()
}

fn outline_width(n: i64) -> f64 {
    LEVELS.iter().position(|&l| l == n).map(|k| OUTLINE_W[k]).unwrap_or(1.5)
}

/// The text of each 17_4 row, from the outline's measured length: "8 cells per diameter: 4 D".
/// Refuses a length that is not a whole number of diameters.
pub fn row_label(s: &Staircase) -> fmn::Result<String> {
    let (cells, n) = (s.perimeter_cells(), s.n);
    if cells % n != 0 {
        return Err(scene_error(format!(
            "{n} cells per diameter: outline {cells} cell widths, not whole diameters"
        )));
    }
    Ok(format!("{n} cells per diameter: {} D", cells / n))
}

/// 17_2's second label, from the 32-cell staircase's largest gap.
pub fn gap_here(s: &Staircase) -> String {
    format!("largest gap here: {:.2} cell widths", s.largest_gap().0 * s.n as f64)
}

/// 17_1 and 17_2's readout: cells per diameter, then the solid cells (cl01 stair.n*).
pub fn level_labels(s: &Staircase) -> [String; 2] {
    [
        format!("{} cells per diameter", s.n),
        format!("{} solid cells", explainer_kit::thousands(s.cells() as u64)),
    ]
}

// --------------------------------------------------------- the magnifier

/// A lens on the figure and its enlarged inset: the figure's window `w`, drawn again at
/// [`INSET_AT`] with the staircase's cells, grid, outline and the circles that cross it.
pub struct Magnifier {
    pub group: Mob,
    pub marker: Mob,
}

impl Magnifier {
    /// `typed`: also draw the typed circle (dashed). The marker runs from `from` to `to`
    /// (diameters), both inside the window.
    pub fn build(
        stage: &mut Stage<'_>,
        s: &Staircase,
        w: Window,
        typed: bool,
        from: (f64, f64),
        to: (f64, f64),
    ) -> fmn::Result<Self> {
        let k = INSET_HALF / w.half;
        let map = |p: (f64, f64)| v(INSET_AT.0 + (p.0 - w.cx) * k, INSET_AT.1 + (p.1 - w.cy) * k);
        let (xl, xh, yl, yh) = (w.cx - w.half, w.cx + w.half, w.cy - w.half, w.cy + w.half);
        let n = s.n;
        let h = 1.0 / n as f64;
        let mut parts = Vec::new();
        // The inset's opaque back and frame.
        let corners = [(xl, yl), (xh, yl), (xh, yh), (xl, yh)];
        parts.push(
            Polygon::new(corners.map(map))
                .style(
                    Style::default()
                        .fill(hex(BACKGROUND), 1.0)
                        .stroke(Palette::dim(), LENS_W, 1.0),
                )
                .build(),
        );
        // Solid cells, clipped to the window.
        let (i0, i1) = ((xl * n as f64).floor() as i64, (xh * n as f64).ceil() as i64);
        let (j0, j1) = ((yl * n as f64).floor() as i64, (yh * n as f64).ceil() as i64);
        for i in i0..i1 {
            for j in j0..j1 {
                if !s.solid(i, j) {
                    continue;
                }
                let (a0, a1) = ((i as f64 * h).max(xl), ((i + 1) as f64 * h).min(xh));
                let (b0, b1) = ((j as f64 * h).max(yl), ((j + 1) as f64 * h).min(yh));
                if a0 < a1 && b0 < b1 {
                    let rect = [(a0, b0), (a1, b0), (a1, b1), (a0, b1)];
                    parts.push(Polygon::new(rect.map(map)).style(fill_style(solid_grey(), 1.0)).build());
                }
            }
        }
        // Cell lines inside the window.
        let grid = line_style(grid_ink(), GRID_W, 1.0);
        for i in i0..=i1 {
            let x = i as f64 * h;
            if let Some((a, b)) = clip_axis_segment((x, yl), (x, yh), &w) {
                parts.push(seg(map(a), map(b), grid));
            }
        }
        for j in j0..=j1 {
            let y = j as f64 * h;
            if let Some((a, b)) = clip_axis_segment((xl, y), (xh, y), &w) {
                parts.push(seg(map(a), map(b), grid));
            }
        }
        // The true circle, and the typed one dashed.
        for (a0, a1) in arc_runs(0.5, &w) {
            let pts: Vec<Vec3> = arc_points(0.5, a0, a1, 0.002).into_iter().map(map).collect();
            parts.push(polyline(pts).style(line_style(Palette::ink(), CIRCLE_W, 0.8)).build());
        }
        if typed {
            let r = staircase01_radius(TYPED_RADIUS);
            for (a0, a1) in arc_runs(r, &w) {
                let pts: Vec<Vec3> = arc_points(r, a0, a1, 0.002).into_iter().map(map).collect();
                parts.push(dashed(
                    &polyline(pts).style(line_style(white(), CIRCLE_W, 1.0)).build(),
                    0.16,
                )?);
            }
        }
        // The staircase outline, clipped.
        for e in s.edges() {
            let (a, b) = (
                (e.a.0 as f64 * h, e.a.1 as f64 * h),
                (e.b.0 as f64 * h, e.b.1 as f64 * h),
            );
            if let Some((a, b)) = clip_axis_segment(a, b, &w) {
                parts.push(seg(map(a), map(b), line_style(Palette::gold(), OUTLINE_W[0], 1.0)));
            }
        }
        // The lens on the figure, and the two links to the inset (the inset lies to its right).
        let lens = corners.map(fig);
        parts.push(
            Polygon::new(lens)
                .style(line_style(Palette::dim(), LENS_W, 1.0))
                .build(),
        );
        let inset = corners.map(map);
        parts.push(seg(lens[1], inset[0], line_style(Palette::dim(), LENS_W * 0.75, 0.8)));
        parts.push(seg(lens[2], inset[3], line_style(Palette::dim(), LENS_W * 0.75, 0.8)));
        let group = group(stage, parts, Z_INSET);
        let marker = group_marker(stage, map(from), map(to));
        Ok(Self { group, marker })
    }
}

fn staircase01_radius((p, q): (i64, i64)) -> f64 {
    p as f64 / q as f64
}

/// A gap marker: a segment with a dot at each end, in the highlight colour.
fn group_marker(stage: &mut Stage<'_>, a: Vec3, b: Vec3) -> Mob {
    let c = Palette::hero();
    let parts = vec![
        seg(a, b, line_style(c, MARK_W, 1.0)),
        Dot::new().point(a).radius(0.06).style(fill_style(c, 1.0)).build(),
        Dot::new().point(b).radius(0.06).style(fill_style(c, 1.0)).build(),
    ];
    group(stage, parts, Z_INSET + 1)
}

/// `source` cut into dashes about `dash` scene units long, half of each period drawn.
fn dashed(source: &VMobject, dash: f64) -> fmn::Result<VMobject> {
    let pts = source.points();
    let length: f64 = pts
        .windows(2)
        .map(|p| ((p[1][0] - p[0][0]).powi(2) + (p[1][1] - p[0][1]).powi(2)).sqrt())
        .sum();
    let count = ((length / (2.0 * dash)).round() as usize).max(1);
    Ok(dashed_vmobject(source, count, 0.5, 0.0)?)
}

/// The lens window: `cells` cell widths on each side of corner `p` (diameters), moved just
/// enough to stay inside the 1 × 1 box (the corner stays inside the window).
fn window_at(s: &Staircase, p: (f64, f64), cells: f64) -> Window {
    let half = cells / s.n as f64;
    let inside = |c: f64| c.clamp(-0.5 + half, 0.5 - half);
    Window {
        cx: inside(p.0),
        cy: inside(p.1),
        half,
    }
}

/// Points closer than this, D, count as the same extreme (ties from the circle's symmetry).
const TIE: f64 = 1e-12;

/// 17_2's lens corner: of the tied largest gaps, the one nearest the inset (right, then low).
pub fn gap_corner(s: &Staircase) -> (f64, f64) {
    rightmost_lowest(&s.largest_gap_points(TIE)).expect("a staircase has an outline")
}

/// 17_6's lens corner: of the typed staircase's tied farthest corners, the one nearest the inset.
pub fn nearest_wall_corner(s: &Staircase) -> (f64, f64) {
    rightmost_lowest(&s.farthest_points(TIE)).expect("a staircase has an outline")
}

// ------------------------------------------------------------- the chapter

/// Every object chapter 17 keeps between its cues.
pub struct Chapter17 {
    pub shown: Shown,
    pub title: Option<Mob>,
    pub circle: Mob,
    pub cross: Mob,
    pub rule: Vec<Mob>,
    pub level: Vec<Mob>,
    pub drawn: Option<Drawn>,
    pub magnifier: Vec<Mob>,
    pub trip: Vec<Mob>,
    pub walker: Vec<Mob>,
    pub halo: Option<Mob>,
    pub rows: Vec<Mob>,
    pub square: Vec<Mob>,
    pub ratio: Vec<Mob>,
    pub normals: Vec<Mob>,
    pub source: Vec<Mob>,
    pub fitted: Vec<Mob>,
    pub cut: Vec<Mob>,
    pub typed: Vec<Mob>,
    pub channel: Vec<Mob>,
}

/// The 17_h1 prediction card.
pub struct Card {
    pub parts: Vec<Mob>,
    pub markers: Vec<Mob>,
    pub chosen: Mob,
    pub ring: Mob,
    pub track: Mob,
    pub note: Mob,
    pub digits: Vec<Mob>,
    ring_at: (f64, f64),
}

impl Card {
    pub fn build(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let (cx, cy) = CARD_AT;
        let (w, h) = CARD_SIZE;
        let top = cy + h / 2.0;
        let ink = Palette::ink();
        let mut parts = vec![card(stage, CARD_AT, CARD_SIZE)?];
        parts.push(text(
            kit,
            stage,
            CARD_LABEL,
            SMALL_PT,
            Palette::dim(),
            (cx, top - 0.35),
        )?);
        parts.push(text(kit, stage, QUESTION[0], CARD_PT, ink, (cx, top - 0.85))?);
        parts.push(text(kit, stage, QUESTION[1], CARD_PT, ink, (cx, top - 1.25))?);
        parts.push(text(kit, stage, SETUP, FINE_PT, Palette::dim(), (cx, top - 1.7))?);
        let left = cx - w / 2.0 + 0.5;
        let mut markers = Vec::new();
        let mut chosen = None;
        for (k, choice) in CHOICES.iter().enumerate() {
            let y = top - 2.18 - 0.42 * k as f64;
            parts.push(text_left(kit, stage, choice, BODY_PT, ink, (left + 0.3, y))?);
            let ring = Circle::new()
                .radius(0.1)
                .arc_center(v(left, y))
                .style(line_style(Palette::dim(), 2.5, 1.0))
                .build();
            markers.push(add(stage, ring, Z_TEXT));
            if k == ANSWER {
                let dot = Dot::new()
                    .point(v(left, y))
                    .radius(0.1)
                    .style(fill_style(Palette::green(), 1.0))
                    .build();
                chosen = Some(add(stage, dot, Z_TEXT + 1));
            }
        }
        let ring_at = (cx + w / 2.0 - 0.6, cy - h / 2.0 + 0.75);
        let note = text(
            kit,
            stage,
            PAUSE_NOTE,
            FINE_PT,
            Palette::dim(),
            (ring_at.0 - 1.6, cy - h / 2.0 + 0.3),
        )?;
        let track = add(
            stage,
            Circle::new()
                .radius(RING_R)
                .arc_center(v(ring_at.0, ring_at.1))
                .style(line_style(Palette::faint(), 6.0, 1.0))
                .build(),
            Z_TEXT,
        );
        let ring = add(
            stage,
            polyline(ring_points(ring_at, RING_R, 1.0))
                .style(line_style(ink, 7.0, 1.0))
                .build(),
            Z_TEXT + 1,
        );
        let mut digits = Vec::new();
        for second in 1..=HOLD_SECONDS as u32 {
            digits.push(text(kit, stage, &second.to_string(), BODY_PT, ink, ring_at)?);
        }
        Ok(Self {
            parts,
            markers,
            chosen: chosen.expect("ANSWER indexes CHOICES"),
            ring,
            track,
            note,
            digits,
            ring_at,
        })
    }

    pub fn all(&self) -> Vec<Mob> {
        self.parts
            .iter()
            .chain(&self.markers)
            .copied()
            .chain([self.chosen])
            .collect()
    }
}

impl Chapter17 {
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, title: bool) -> fmn::Result<Self> {
        let title = if title {
            Some(text_left(kit, stage, TITLE, TITLE_PT, Palette::ink(), TITLE_AT)?)
        } else {
            None
        };
        let circle = add(
            stage,
            Circle::new()
                .radius(FIG_D / 2.0)
                .arc_center(fig((0.0, 0.0)))
                .style(line_style(Palette::ink(), CIRCLE_W, 0.8))
                .build(),
            Z_CIRCLE,
        );
        let c = fig((0.0, 0.0));
        let arm = 0.14;
        let cross = group(
            stage,
            vec![
                seg(v(c[0] - arm, c[1]), v(c[0] + arm, c[1]), line_style(white(), 3.0, 1.0)),
                seg(v(c[0], c[1] - arm), v(c[0], c[1] + arm), line_style(white(), 3.0, 1.0)),
            ],
            Z_MARK,
        );
        Ok(Self {
            shown: Shown::default(),
            title,
            circle,
            cross,
            rule: Vec::new(),
            level: Vec::new(),
            drawn: None,
            magnifier: Vec::new(),
            trip: Vec::new(),
            walker: Vec::new(),
            halo: None,
            rows: Vec::new(),
            square: Vec::new(),
            ratio: Vec::new(),
            normals: Vec::new(),
            source: Vec::new(),
            fitted: Vec::new(),
            cut: Vec::new(),
            typed: Vec::new(),
            channel: Vec::new(),
        })
    }

    /// Replace the staircase on screen (and the level readout) with `next`.
    fn change_level(
        &mut self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        next: Drawn,
        readout: bool,
        rt: f64,
    ) -> fmn::Result<()> {
        let out: Vec<Mob> = self
            .drawn
            .as_ref()
            .map(Drawn::all)
            .unwrap_or_default()
            .into_iter()
            .chain(self.level.clone())
            .collect();
        let mut into = next.all();
        let mut level = Vec::new();
        if readout {
            let [a, b] = level_labels(&next.stair);
            level.push(text(kit, stage, &a, BODY_PT, Palette::ink(), LEVEL_AT[0])?);
            level.push(text(kit, stage, &b, SMALL_PT, Palette::dim(), LEVEL_AT[1])?);
            into.extend(&level);
        }
        self.shown.swap(stage, &out, &into, rt)?;
        self.drawn = Some(next);
        self.level = level;
        Ok(())
    }

    // ---------------------------------------------------------------- 17_1

    pub fn cue_17_1(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "17_1")?;
        let mut first: Vec<Mob> = self.title.into_iter().collect();
        first.push(self.circle);
        self.shown.show(stage, &first, FADE)?;
        kit.until_sentence(stage, "17_1", 1)?;
        let eight = Drawn::build(stage, Staircase::new(8, TRUE_RADIUS), OUTLINE_W[0], true, true)?;
        let grid = eight.grid.expect("built with a grid");
        let dots = eight.dots.expect("built with dots");
        let [rule_card, l0, l1] = [
            card(stage, RULE_AT, RULE_SIZE)?,
            text(
                kit,
                stage,
                RULE[0],
                BODY_PT,
                Palette::ink(),
                (RULE_AT.0, RULE_AT.1 + 0.22),
            )?,
            text(
                kit,
                stage,
                RULE[1],
                BODY_PT,
                Palette::ink(),
                (RULE_AT.0, RULE_AT.1 - 0.22),
            )?,
        ];
        self.rule = vec![rule_card, l0, l1];
        self.shown.show(stage, &[grid, self.cross], FADE)?;
        self.shown.show(stage, &[dots], FADE)?;
        self.shown.show(stage, &self.rule.clone(), FADE)?;
        self.shown.show(stage, &[eight.fill], DRAW)?;
        kit.until_sentence(stage, "17_1", 2)?;
        self.shown.create(stage, &[eight.outline], DRAW)?;
        let [a, b] = level_labels(&eight.stair);
        self.level = vec![
            text(kit, stage, &a, BODY_PT, Palette::ink(), LEVEL_AT[0])?,
            text(kit, stage, &b, SMALL_PT, Palette::dim(), LEVEL_AT[1])?,
        ];
        self.shown.show(stage, &self.level.clone(), FADE)?;
        self.drawn = Some(eight);
        Ok(())
    }

    // ---------------------------------------------------------------- 17_2

    pub fn cue_17_2(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "17_2")?;
        let sixteen = Drawn::build(stage, Staircase::new(16, TRUE_RADIUS), OUTLINE_W[1], true, false)?;
        self.change_level(kit, stage, sixteen, true, FADE)?;
        kit.until_sentence(stage, "17_2", 1)?;
        let thirty_two = Drawn::build(stage, Staircase::new(32, TRUE_RADIUS), OUTLINE_W[2], true, false)?;
        self.change_level(kit, stage, thirty_two, true, FADE)?;
        let s = &self.drawn.as_ref().expect("the 32-cell staircase").stair;
        let corner = gap_corner(s);
        let towards = towards_circle(corner, 0.5);
        let mag = Magnifier::build(stage, s, window_at(s, corner, LENS_HALF_CELLS), false, corner, towards)?;
        let here = gap_here(s);
        let labels = [
            text(kit, stage, GAP_BOUND, BODY_PT, Palette::hero(), INSET_LABEL_AT[0])?,
            text(kit, stage, &here, FINE_PT, Palette::dim(), INSET_LABEL_AT[1])?,
        ];
        self.magnifier = vec![mag.group, mag.marker, labels[0], labels[1]];
        self.shown.show(stage, &[mag.group], FADE)?;
        self.shown.show(stage, &[mag.marker, labels[0], labels[1]], FADE)?;
        Ok(())
    }

    // ---------------------------------------------------------------- 17_3

    pub fn cue_17_3(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Card> {
        kit.say(stage, "17_3")?;
        let lens = std::mem::take(&mut self.magnifier);
        let rule = std::mem::take(&mut self.rule);
        self.shown.hide(stage, &[lens, rule].concat(), FADE)?;
        kit.until_sentence(stage, "17_3", 1)?;
        // The walk: once around from the bottom, counterclockwise.
        let start = -FRAC_PI_2;
        let on = |a: f64| fig((0.5 * a.cos(), 0.5 * a.sin()));
        let dot = add(
            stage,
            Dot::new()
                .point(on(start))
                .radius(0.09)
                .style(fill_style(white(), 1.0))
                .build(),
            Z_MARK,
        );
        let trail = add(
            stage,
            polyline(vec![on(start), on(start)])
                .style(line_style(white(), CIRCLE_W + 1.0, 1.0))
                .build(),
            Z_MARK,
        );
        let around = text(kit, stage, TRIP[0], BODY_PT, Palette::ink(), TRIP_AT[0])?;
        self.shown.show(stage, &[around], QUICK)?;
        self.shown.put(stage, trail)?;
        self.shown.put(stage, dot)?;
        let frames = frames_covering(WALK, stage.scene().time().fps());
        for k in 1..=frames {
            let a = start + TAU * k as f64 / frames as f64;
            let steps = ((k as f64 / frames as f64) * 240.0).ceil().max(1.0) as usize;
            let pts: Vec<Vec3> = (0..=steps)
                .map(|m| on(start + (a - start) * m as f64 / steps as f64))
                .collect();
            let _ = stage.set_points(trail, polyline(pts).build().points());
            stage.move_to(dot, on(a), ORIGIN);
            wait_frames(stage, 1)?;
        }
        self.walker = vec![dot, trail];
        kit.until_sentence(stage, "17_3", 2)?;
        let s = &self.drawn.as_ref().expect("the 32-cell staircase").stair;
        let corners: Vec<Vec3> = s.polygon().iter().map(|&p| cell_pt(s, p)).collect();
        let halo = add(
            stage,
            Polygon::new(corners)
                .style(line_style(Palette::gold(), HALO_W, 0.3))
                .build(),
            Z_OUTLINE - 1,
        );
        let stair_trip = text(kit, stage, TRIP[1], BODY_PT, Palette::gold(), TRIP_AT[1])?;
        self.trip = vec![around, stair_trip];
        self.shown
            .swap(stage, &self.walker.clone(), &[halo, stair_trip], FADE)?;
        self.halo = Some(halo);
        let card = Card::build(kit, stage)?;
        let shown: Vec<Mob> = card
            .parts
            .iter()
            .chain(&card.markers)
            .copied()
            .chain([card.note, card.track])
            .collect();
        self.shown.show(stage, &shown, FADE)?;
        Ok(card)
    }

    // --------------------------------------------------------------- 17_h1

    /// The question stays; the countdown runs 5 s on the ring and digits; nothing else moves.
    pub fn hold_17_h1(&mut self, kit: &Kit, stage: &mut Stage<'_>, card: &Card) -> fmn::Result<()> {
        kit.hold(stage)?;
        kit.mark(stage, "predict");
        self.shown.put(stage, card.ring)?;
        let fps = stage.scene().time().fps();
        let frames = frames_covering(HOLD_SECONDS, fps);
        let mut showing: Option<Mob> = None;
        for k in 0..frames {
            let left = HOLD_SECONDS * (frames - k) as f64 / frames as f64;
            let digit = card.digits[(left.ceil() as usize).clamp(1, card.digits.len()) - 1];
            if showing != Some(digit) {
                if let Some(old) = showing {
                    self.shown.take(stage, old);
                }
                self.shown.put(stage, digit)?;
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
            self.shown.take(stage, old);
        }
        self.shown.take(stage, card.ring);
        self.shown.hide(stage, &[card.track, card.note], QUICK)
    }

    // ---------------------------------------------------------------- 17_4

    pub fn cue_17_4(&mut self, kit: &Kit, stage: &mut Stage<'_>, card: Card) -> fmn::Result<()> {
        kit.say(stage, "17_4")?;
        self.shown.show(stage, &[card.chosen], QUICK)?;
        kit.until_sentence(stage, "17_4", 1)?;
        // Clear the card, the readouts and the 32-cell drawing; bring in the 8-cell pieces.
        let mut out = card.all();
        out.append(&mut self.trip);
        out.extend(self.halo.take());
        out.append(&mut self.level);
        out.extend(self.drawn.take().map(|d| d.all()).unwrap_or_default());
        out.push(self.cross);
        let eight = Staircase::new(8, TRUE_RADIUS);
        let mut moving = pieces(stage, &eight, OUTLINE_W[0]);
        let into: Vec<Mob> = moving.iter().map(|p| p.0).collect();
        self.shown.swap(stage, &out, &into, FADE)?;
        for (k, &n) in LEVELS.iter().enumerate() {
            if k > 0 {
                let stair = Staircase::new(n, TRUE_RADIUS);
                let next = pieces(stage, &stair, outline_width(n));
                let into: Vec<Mob> = next.iter().map(|p| p.0).collect();
                let out: Vec<Mob> = moving.iter().map(|p| p.0).collect();
                self.shown.swap(stage, &out, &into, QUICK)?;
                moving = next;
            }
            let rt = if k == 0 { SLIDE_FIRST } else { SLIDE_AGAIN };
            let mut anims: Vec<Box<dyn Animation>> = Vec::new();
            for &(m, (dx, dy)) in &moving {
                if dx != 0.0 || dy != 0.0 {
                    let a = m
                        .animate()
                        .set_anim_args(AnimateArgs {
                            run_time: Some(rt),
                            ..AnimateArgs::default()
                        })?
                        .shift(v(dx, dy))?;
                    anims.push(stage.prepare(a)?);
                }
            }
            stage.play_prepared(anims)?;
            let label = row_label(&Staircase::new(n, TRUE_RADIUS))?;
            let row = text(
                kit,
                stage,
                &label,
                BODY_PT,
                Palette::ink(),
                (PANEL_X, ROWS_TOP - ROW_STEP * k as f64),
            )?;
            self.rows.push(row);
            self.shown.show(stage, &[row], QUICK)?;
        }
        self.square = moving.iter().map(|p| p.0).collect();
        kit.until_sentence(stage, "17_4", 2)?;
        // Back to the 8-cell staircase, unmoved, for the normals.
        let eight = Drawn::build(stage, eight, OUTLINE_W[0], false, false)?;
        let ratio = [
            text(kit, stage, RATIO[0], BODY_PT, Palette::ink(), (PANEL_X, RATIO_Y[0]))?,
            text(kit, stage, RATIO[1], BODY_PT, Palette::ink(), (PANEL_X, RATIO_Y[1]))?,
        ];
        self.ratio = ratio.to_vec();
        let square = std::mem::take(&mut self.square);
        let mut into = vec![eight.fill, eight.outline];
        into.extend(ratio);
        self.shown.swap(stage, &square, &into, FADE)?;
        // Normals: axis arrows on the staircase's left half, radial arrows on the circle's right.
        let mut arrows = Vec::new();
        let gold = Palette::gold();
        for e in eight.stair.edges() {
            let mid = ((e.a.0 + e.b.0) as f64 / 16.0, (e.a.1 + e.b.1) as f64 / 16.0);
            if mid.0 >= 0.0 {
                continue;
            }
            let (nx, ny) = e.normal();
            let from = fig(mid);
            let to = v(from[0] + 0.42 * nx as f64, from[1] + 0.42 * ny as f64);
            arrows.push(arrow(stage, from, to, gold)?);
        }
        let blue = Palette::blue();
        for k in 0..8 {
            let a = -FRAC_PI_2 + PI * (k as f64 + 0.5) / 8.0;
            let from = fig((0.5 * a.cos(), 0.5 * a.sin()));
            let to = v(from[0] + 0.42 * a.cos(), from[1] + 0.42 * a.sin());
            arrows.push(arrow(stage, from, to, blue)?);
        }
        let legend = [
            text(kit, stage, NORMALS[0], SMALL_PT, gold, (PANEL_X, NORMALS_Y[0]))?,
            text(kit, stage, NORMALS[1], SMALL_PT, blue, (PANEL_X, NORMALS_Y[1]))?,
        ];
        self.shown.show(stage, &arrows, FADE)?;
        self.shown.show(stage, &legend, FADE)?;
        self.normals = [arrows, legend.to_vec()].concat();
        self.drawn = Some(eight);
        Ok(())
    }

    // ---------------------------------------------------------------- 17_5

    pub fn cue_17_5(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "17_5")?;
        let out: Vec<Mob> = [
            std::mem::take(&mut self.normals),
            std::mem::take(&mut self.ratio),
            std::mem::take(&mut self.rows),
        ]
        .concat();
        self.shown.hide(stage, &out, FADE)?;
        kit.until_sentence(stage, "17_5", 1)?;
        let (cx, cy) = QUOTE_AT;
        let top = cy + QUOTE_SIZE.1 / 2.0;
        let mut source = vec![card(stage, QUOTE_AT, QUOTE_SIZE)?];
        for (k, line) in QUOTE.iter().enumerate() {
            source.push(text(
                kit,
                stage,
                line,
                FINE_PT,
                Palette::ink(),
                (cx, top - 0.5 - 0.45 * k as f64),
            )?);
        }
        source.push(text(
            kit,
            stage,
            QUOTE_SOURCE,
            FINE_PT,
            Palette::dim(),
            (cx, top - 2.45),
        )?);
        self.shown.show(stage, &source, FADE)?;
        self.source = source;
        kit.until_sentence(stage, "17_5", 2)?;
        let c = FITTED_AT;
        let mut lines = Vec::new();
        for (k, &r) in FITTED_RINGS.iter().enumerate() {
            let (color, width) = if k == 0 {
                (Palette::ink(), CIRCLE_W)
            } else {
                (Palette::dim(), GRID_W)
            };
            lines.push(
                Circle::new()
                    .radius(r)
                    .arc_center(v(c.0, c.1))
                    .style(line_style(color, width, 1.0))
                    .build(),
            );
        }
        let (r0, r1) = (FITTED_RINGS[0], FITTED_RINGS[FITTED_RINGS.len() - 1]);
        for k in 0..FITTED_SPOKES {
            let a = TAU * k as f64 / FITTED_SPOKES as f64;
            lines.push(seg(
                v(c.0 + r0 * a.cos(), c.1 + r0 * a.sin()),
                v(c.0 + r1 * a.cos(), c.1 + r1 * a.sin()),
                line_style(Palette::dim(), GRID_W, 1.0),
            ));
        }
        let fitted = vec![
            group(stage, lines, Z_GRID),
            text(
                kit,
                stage,
                FITTED_LABEL[0],
                SMALL_PT,
                Palette::ink(),
                (PANEL_X, FITTED_LABEL_Y[0]),
            )?,
            text(
                kit,
                stage,
                FITTED_LABEL[1],
                FINE_PT,
                Palette::dim(),
                (PANEL_X, FITTED_LABEL_Y[1]),
            )?,
        ];
        self.shown.show(stage, &fitted, FADE)?;
        self.fitted = fitted;
        kit.until_sentence(stage, "17_5", 3)?;
        // Back on the square grid: cut cells shaded by their solid share, crossed faces open.
        let mut out = std::mem::take(&mut self.source);
        out.append(&mut self.fitted);
        out.extend(self.drawn.take().map(|d| d.all()).unwrap_or_default());
        let n = 8;
        let grid = box_grid(stage, n);
        let cells = shaded_cells(stage, n);
        let faces = crossed_faces(stage, n);
        let note = [
            text(kit, stage, KARMAN[0], BODY_PT, Palette::ink(), (PANEL_X, KARMAN_Y[0]))?,
            text(kit, stage, KARMAN[1], BODY_PT, Palette::ink(), (PANEL_X, KARMAN_Y[1]))?,
            text(
                kit,
                stage,
                KARMAN_NOTE,
                SMALL_PT,
                Palette::dim(),
                (PANEL_X, KARMAN_Y[2]),
            )?,
            text(
                kit,
                stage,
                CUT_LEGEND[0],
                SMALL_PT,
                solid_label(),
                (PANEL_X, CUT_LEGEND_Y[0]),
            )?,
            text(
                kit,
                stage,
                CUT_LEGEND[1],
                SMALL_PT,
                Palette::teal(),
                (PANEL_X, CUT_LEGEND_Y[1]),
            )?,
        ];
        self.shown.swap(stage, &out, &[grid, cells], FADE)?;
        self.shown.show(stage, &[faces], FADE)?;
        self.shown.show(stage, &note, FADE)?;
        self.cut = [vec![grid, cells, faces], note.to_vec()].concat();
        Ok(())
    }

    // ---------------------------------------------------------------- 17_6

    pub fn cue_17_6(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "17_6")?;
        let cut = std::mem::take(&mut self.cut);
        self.shown.hide(stage, &cut, FADE)?;
        kit.until_sentence(stage, "17_6", 1)?;
        let r = staircase01_radius(TYPED_RADIUS) * FIG_D;
        let typed_circle = Circle::new()
            .radius(r)
            .arc_center(fig((0.0, 0.0)))
            .style(line_style(white(), CIRCLE_W, 1.0))
            .build();
        let typed_circle = add(stage, dashed(&typed_circle, 0.16)?, Z_CIRCLE);
        let readouts = [
            text(kit, stage, WRONG[0], BODY_PT, Palette::red(), (PANEL_X, WRONG_Y[0]))?,
            text(kit, stage, WRONG[1], SMALL_PT, Palette::ink(), (PANEL_X, WRONG_Y[1]))?,
        ];
        self.shown.show(stage, &[typed_circle], FADE)?;
        self.shown.show(stage, &readouts, FADE)?;
        self.typed = [vec![typed_circle], readouts.to_vec()].concat();
        kit.until_sentence(stage, "17_6", 2)?;
        for n in WRONG_LEVELS {
            let stair = Staircase::new(n, TYPED_RADIUS);
            // The picture says the staircase never reaches the true wall: refuse if it does.
            if !stair.strictly_inside(TRUE_RADIUS) {
                return Err(scene_error(format!(
                    "{n} cells: the typed staircase reaches the true wall"
                )));
            }
            let next = Drawn::build(stage, stair, TYPED_OUTLINE_W, false, false)?;
            let label = text(
                kit,
                stage,
                &format!("{n} cells per diameter"),
                BODY_PT,
                Palette::ink(),
                (PANEL_X, WRONG_LEVEL_Y),
            )?;
            let mut out: Vec<Mob> = self.drawn.take().map(|d| d.all()).unwrap_or_default();
            out.append(&mut self.level);
            out.append(&mut self.magnifier);
            let mut into = next.all();
            into.push(label);
            self.shown.swap(stage, &out, &into, FADE)?;
            self.level = vec![label];
            if n == WRONG_LEVELS[0] {
                let corner = nearest_wall_corner(&next.stair);
                let wall = towards_circle(corner, 0.5);
                let mag = Magnifier::build(
                    stage,
                    &next.stair,
                    window_at(&next.stair, corner, LENS_HALF_CELLS),
                    true,
                    corner,
                    wall,
                )?;
                let label = text(kit, stage, NEAREST, SMALL_PT, Palette::hero(), INSET_LABEL_AT[0])?;
                self.magnifier = vec![mag.group, mag.marker, label];
                self.shown.show(stage, &self.magnifier.clone(), FADE)?;
                stage.wait(LOOK)?;
            }
            self.drawn = Some(next);
            stage.wait(LOOK)?;
        }
        Ok(())
    }

    // ---------------------------------------------------------------- 17_7

    pub fn cue_17_7(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "17_7")?;
        // Everything but the title goes.
        let out: Vec<Mob> = self
            .shown
            .0
            .iter()
            .copied()
            .filter(|&m| Some(m) != self.title)
            .collect();
        self.shown.hide(stage, &out, FADE)?;
        self.drawn = None;
        self.level.clear();
        self.magnifier.clear();
        self.typed.clear();
        let m = |x: f64, y: f64| v(CH_ORIGIN.0 + CH_SCALE * x, CH_ORIGIN.1 + CH_SCALE * y);
        let (l, h, d) = (channel::LENGTH, channel::HEIGHT, channel::DIAMETER);
        let (cx, cy) = channel::CENTRE;
        let ink = Palette::ink();
        let walls = group(
            stage,
            vec![
                seg(m(0.0, 0.0), m(l, 0.0), line_style(ink, WALL_W, 1.0)),
                seg(m(0.0, h), m(l, h), line_style(ink, WALL_W, 1.0)),
                seg(m(0.0, 0.0), m(0.0, h), line_style(Palette::dim(), DIM_W, 1.0)),
                seg(m(l, 0.0), m(l, h), line_style(Palette::dim(), DIM_W, 1.0)),
            ],
            Z_OUTLINE,
        );
        let cylinder = add(
            stage,
            Circle::new()
                .radius(CH_SCALE * d / 2.0)
                .arc_center(m(cx, cy))
                .style(Style::default().fill(solid_grey(), 1.0).stroke(ink, CIRCLE_W, 1.0))
                .build(),
            Z_OUTLINE,
        );
        self.shown.show(stage, &[walls, cylinder], FADE)?;
        kit.until_sentence(stage, "17_7", 1)?;
        let dim = |stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb| -> Mob {
            let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
            let len = (dx * dx + dy * dy).sqrt();
            let (tx, ty) = (-dy / len * 0.08, dx / len * 0.08);
            group(
                stage,
                vec![
                    seg(a, b, line_style(color, DIM_W, 1.0)),
                    seg(
                        v(a[0] - tx, a[1] - ty),
                        v(a[0] + tx, a[1] + ty),
                        line_style(color, DIM_W, 1.0),
                    ),
                    seg(
                        v(b[0] - tx, b[1] - ty),
                        v(b[0] + tx, b[1] + ty),
                        line_style(color, DIM_W, 1.0),
                    ),
                ],
                Z_MARK,
            )
        };
        let top = CH_ORIGIN.1 + CH_SCALE * h;
        let length = dim(
            stage,
            v(CH_ORIGIN.0, top + 0.45),
            v(CH_ORIGIN.0 + CH_SCALE * l, top + 0.45),
            ink,
        );
        let length_label = text(kit, stage, CH_LENGTH_LABEL, BODY_PT, ink, (0.0, top + 0.8))?;
        let hx = CH_ORIGIN.0 + CH_SCALE * (l - 0.12);
        let height = dim(stage, v(hx, CH_ORIGIN.1), v(hx, top), ink);
        let height_label = text_left(
            kit,
            stage,
            CH_HEIGHT_LABEL,
            BODY_PT,
            ink,
            (hx - 1.15, CH_ORIGIN.1 + CH_SCALE * h / 2.0),
        )?;
        self.shown
            .show(stage, &[length, length_label, height, height_label], FADE)?;
        kit.until_sentence(stage, "17_7", 2)?;
        let x = CH_ORIGIN.0 + CH_SCALE * (cx + 0.125);
        let ys = [0.0, cy - d / 2.0, cy + d / 2.0, h].map(|y| CH_ORIGIN.1 + CH_SCALE * y);
        let mut gaps = Vec::new();
        for k in 0..3 {
            gaps.push(dim(
                stage,
                v(x, ys[k]),
                v(x, ys[k + 1]),
                if k == 1 { ink } else { Palette::teal() },
            ));
        }
        let ext = group(
            stage,
            vec![
                seg(
                    m(cx, cy - d / 2.0),
                    v(x, ys[1]),
                    line_style(Palette::dim(), GRID_W, 1.0),
                ),
                seg(
                    m(cx, cy + d / 2.0),
                    v(x, ys[2]),
                    line_style(Palette::dim(), GRID_W, 1.0),
                ),
            ],
            Z_GRID,
        );
        for (k, label) in CH_GAP_LABELS.iter().enumerate() {
            let y = 0.5 * (ys[k] + ys[k + 1]);
            gaps.push(text_left(
                kit,
                stage,
                label,
                SMALL_PT,
                if k == 1 { ink } else { Palette::teal() },
                (x + 0.2, y),
            )?);
        }
        gaps.push(ext);
        self.shown.show(stage, &gaps, FADE)?;
        kit.until_sentence(stage, "17_7", 3)?;
        // The 1 cm: the top of the upper gap, above the height of the lower one.
        let excess = channel::gap_above() - channel::gap_below();
        if (excess - 0.01).abs() > 1e-12 {
            return Err(scene_error(format!("the gaps differ by {excess} m, not 1 cm")));
        }
        let mark_from = ys[3] - CH_SCALE * excess;
        let hero = Palette::hero();
        let mid = 0.5 * (mark_from + ys[3]);
        let extra = group(
            stage,
            vec![
                seg(v(x, mark_from), v(x, ys[3]), line_style(hero, MARK_W * 2.0, 1.0)),
                seg(
                    v(x - 0.12, mark_from),
                    v(x + 0.12, mark_from),
                    line_style(hero, DIM_W, 1.0),
                ),
                seg(
                    v(x + 0.15, mid),
                    v(CH_DIFFERENCE_AT.0 - 0.1, CH_DIFFERENCE_AT.1),
                    line_style(hero, DIM_W, 1.0),
                ),
            ],
            Z_MARK + 1,
        );
        let extra_label = text_left(kit, stage, CH_DIFFERENCE, SMALL_PT, hero, CH_DIFFERENCE_AT)?;
        self.shown.show(stage, &[extra, extra_label], FADE)?;
        self.channel = [
            vec![walls, cylinder, length, length_label, height, height_label],
            gaps,
            vec![extra, extra_label],
        ]
        .concat();
        kit.hold(stage)
    }
}

fn solid_label() -> Srgb {
    hex("#9AA6B8")
}

fn arrow(stage: &mut Stage<'_>, from: Vec3, to: Vec3, color: Srgb) -> fmn::Result<Mob> {
    let a = Arrow::new(from, to)
        .buff(0.0)
        .thickness(3.0)
        .style(fill_style(color, 1.0))
        .build()?;
    Ok(add(stage, a, Z_MARK))
}

/// The 8 × 8 box's cells: fully solid cells grey, crossed cells grey at their solid share
/// (exact areas, [`cut_cells`]), fluid cells empty.
fn shaded_cells(stage: &mut Stage<'_>, n: i64) -> Mob {
    let h = 1.0 / n as f64;
    let cut: Vec<CutCell> = cut_cells(n);
    let mut parts = Vec::new();
    for i in -n / 2..n / 2 {
        for j in -n / 2..n / 2 {
            let far = |lo: i64| lo.abs().max((lo + 1).abs());
            let inside = 4 * (far(i).pow(2) + far(j).pow(2)) <= n * n;
            let share = if inside {
                1.0
            } else if let Some(c) = cut.iter().find(|c| c.i == i && c.j == j) {
                1.0 - c.fluid
            } else {
                continue;
            };
            let (x0, y0) = (i as f64 * h, j as f64 * h);
            let rect = [(x0, y0), (x0 + h, y0), (x0 + h, y0 + h), (x0, y0 + h)].map(fig);
            parts.push(Polygon::new(rect).style(fill_style(solid_grey(), share)).build());
        }
    }
    group(stage, parts, Z_FILL)
}

/// Every face the true wall crosses at `n` cells: the open (fluid) part teal, the solid part
/// dim ([`cut_faces`]).
fn crossed_faces(stage: &mut Stage<'_>, n: i64) -> Mob {
    let h = 1.0 / n as f64;
    let mut parts = Vec::new();
    for f in cut_faces(n) {
        debug_assert!(f.open > 0.0 && f.open < 1.0, "a crossed face is partly open: {f:?}");
        let (a, b) = (
            (f.a.0 as f64 * h, f.a.1 as f64 * h),
            (f.b.0 as f64 * h, f.b.1 as f64 * h),
        );
        let (s0, s1) = f.solid;
        // the open part is the face minus the solid interval; it lies at one end
        let open = if (s0.0 - a.0).abs() + (s0.1 - a.1).abs() < 1e-12 {
            (s1, b)
        } else {
            (a, s0)
        };
        parts.push(seg(fig(s0), fig(s1), line_style(Palette::dim(), MARK_W, 1.0)));
        parts.push(seg(
            fig(open.0),
            fig(open.1),
            line_style(Palette::teal(), MARK_W + 1.0, 1.0),
        ));
    }
    group(stage, parts, Z_MARK)
}

// --------------------------------------------------------------- the cues

/// Cues 17_1 to 17_7 and the 17_h1 hold, in order. Returns the chapter's objects, with the
/// channel still on screen for root's 17_8.
pub fn chapter17(kit: &Kit, stage: &mut Stage<'_>, title: bool) -> fmn::Result<Chapter17> {
    let mut ch = Chapter17::build(kit, stage, title)?;
    ch.cue_17_1(kit, stage)?;
    ch.cue_17_2(kit, stage)?;
    let card = ch.cue_17_3(kit, stage)?;
    ch.hold_17_h1(kit, stage, &card)?;
    ch.cue_17_4(kit, stage, card)?;
    ch.cue_17_5(kit, stage)?;
    ch.cue_17_6(kit, stage)?;
    ch.cue_17_7(kit, stage)?;
    Ok(ch)
}

/// The spoken lines this scene says, verbatim from `navier-full-script03.md` chapter 17
/// (`tools/check_boundary01.py` compares them byte for byte), as `script.tsv` rows. 17_h1 is a
/// hold, not a line.
#[cfg(test)]
pub const SCRIPT17: &str = concat!(
    "17_1\tBack to the cylinder's outline. Put the circle's center where four square cells meet, and call each cell solid or fluid, depending on whether its middle lies inside the circle. That simplest drawing turns the circle into a staircase.\n",
    "17_2\tHalve the cells, and the steps shrink. Every point of the staircase stays within about seven tenths of a cell width of the true circle, so halving the cells halves that limit.\n",
    "17_3\tBut one thing doesn't improve. Walk around the circle, and the trip is about three point one four diameters. What happens to the trip around the staircase as the cells shrink?\n",
    "17_4\tIt stays at exactly four diameters, at every size. Slide each step outward, and the pieces line up into a square around the circle. That's about a quarter too long, and every piece faces straight up, down, left or right, while the real wall turns smoothly through every direction.\n",
    "17_5\tDoes that matter? The nineteen ninety-six comparison from chapter sixteen concluded that, for drag and lift, crude curved walls aren't accurate enough, even on very fine grids. Its most accurate solutions used grids shaped to fit the wall. On a square grid, one fix is to tell each cell how much of it is fluid, and where the real wall crosses it.\n",
    "17_6\tNow a worse mistake, one that refining can't see. Suppose the diameter we typed in is five percent too small. Each halving hugs that wrong circle more closely, and if the method converges, the answers settle down onto the answer for the wrong cylinder.\n",
    "17_7\tSo it pays to solve a problem that's pinned down exactly, like the standard test from chapter sixteen. Its channel is two point two meters long and forty-one centimeters tall. Its cylinder is ten centimeters across, with fifteen centimeters of gap below it and sixteen above. Even that one centimeter of difference is part of the test.\n",
);

#[cfg(test)]
mod tests {
    use super::*;
    use explainer_kit::{Narrator, Script};

    #[test]
    fn script_has_the_seven_lines_in_order() {
        let script = Script::parse(SCRIPT17).unwrap();
        let ids: Vec<&str> = script.lines().iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(ids, ["17_1", "17_2", "17_3", "17_4", "17_5", "17_6", "17_7"]);
        let kit = Kit::new().unwrap().with_narrator(Narrator::estimate(&script));
        assert_eq!(kit.sentence_starts("17_5").len(), 4);
        assert_eq!(kit.sentence_starts("17_7").len(), 4);
    }

    #[test]
    fn layout_stays_in_frame() {
        let (x0, x1) = (FIG.0 - FIG_D / 2.0, FIG.0 + FIG_D / 2.0);
        let (y0, y1) = (FIG.1 - FIG_D / 2.0, FIG.1 + FIG_D / 2.0);
        assert!(x0 > -8.0 + 0.4 && x1 < 0.0 && y0 > -4.5 + 0.4 && y1 < TITLE_AT.1 - 0.4);
        assert!(INSET_AT.0 - INSET_HALF > x1 + 1.0 && INSET_AT.0 + INSET_HALF < 7.6);
        let ch_right = CH_ORIGIN.0 + CH_SCALE * channel::LENGTH;
        assert!(CH_ORIGIN.0 > -7.6 && ch_right < 7.6);
    }

    #[test]
    fn lens_windows_are_binary_exact_and_inside_the_box() {
        let s = Staircase::new(32, TRUE_RADIUS);
        let corner = gap_corner(&s);
        assert_eq!(corner, (0.4375, -0.28125));
        let w = window_at(&s, corner, LENS_HALF_CELLS);
        assert_eq!(w.half * 64.0, 3.0);
        assert!(w.cx.abs() + w.half <= 0.5 && w.cy.abs() + w.half <= 0.5);
        assert!(w.contains(corner) && w.contains(towards_circle(corner, 0.5)));
        let t = Staircase::new(32, TYPED_RADIUS);
        let corner = nearest_wall_corner(&t);
        assert_eq!(corner, (0.46875, -0.15625));
        assert!(corner.0.hypot(corner.1) < 0.5);
        let w = window_at(&t, corner, LENS_HALF_CELLS);
        assert_eq!((w.cx, w.cy), (0.453125, -0.15625));
        assert!(w.cx.abs() + w.half <= 0.5 && w.cy.abs() + w.half <= 0.5);
        assert!(w.contains(corner) && w.contains(towards_circle(corner, 0.5)));
    }

    #[test]
    fn labels_come_from_the_geometry() {
        for n in LEVELS {
            assert_eq!(
                row_label(&Staircase::new(n, TRUE_RADIUS)).unwrap(),
                format!("{n} cells per diameter: 4 D")
            );
        }
        assert_eq!(
            gap_here(&Staircase::new(32, TRUE_RADIUS)),
            "largest gap here: 0.64 cell widths"
        );
    }
}
