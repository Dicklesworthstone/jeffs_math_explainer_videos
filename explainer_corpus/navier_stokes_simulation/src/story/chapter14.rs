//! Chapter 14, "Spend cells where they matter" (navier-full-script03.md cues 14_1 to 14_13 and the
//! hold 14_h1; BronzeJaguar's storyboard navier-adaptive-scene03.md, lines 84 to 132).
//!
//! The chapter asks where an adaptive grid should spend its cells, and shows that "split where the
//! air spins" is the wrong test with a counterexample that has no spin at all: the ideal flow past a
//! cylinder. Its streamlines are exact curves, and every number on screen is a closed form from
//! `adapt_row`, evaluated in exact rationals. Nothing is simulated and no adaptive tunnel grid is
//! drawn (FUTURE-SIM A1), so the tunnel's adaptive cell count stays an open readout. Splitting and
//! merging are drawn on one schematic cell (14_2, 14_13) and on the row (14_8, 14_9); multigrid's
//! whole coarse copies appear once, inside a "pressure solve" frame, as a different idea.
//!
//! Voice: the kit narrator. Each cue is `say`d before its pictures, which wait on its sentence
//! starts (`until`). The prediction 14_h1, after 14_6, holds a five-second countdown while the miss
//! bars, their readouts and the 4/9 and 3/4 labels are not yet drawn. A scratch-voice draft times the
//! gates from estimated line lengths, a narrated render from the takes' own sentence starts.

use explainer_kit::{BACKGROUND, Facts, Kit, Palette, Timed, hex, v};
use fmn::animation::indicate;
use fmn::prelude::*;

use crate::adapt_row::{self, Cell};
use crate::four_cell::Q;

/// Chapter 14's cues in spoken order and the number of sentences in each (the narrator's gates).
/// Sentences end at `.`, `?` or `!` before a space, as in the kit; the check recounts them.
pub const CUES: [(&str, usize); 13] = [
    ("14_1", 3),
    ("14_2", 3),
    ("14_3", 3),
    ("14_4", 4),
    ("14_5", 3),
    ("14_6", 2),
    ("14_7", 4),
    ("14_8", 4),
    ("14_9", 3),
    ("14_10", 4),
    ("14_11", 4),
    ("14_12", 4),
    ("14_13", 2),
];

/// Script HOLD 14_h1: the prediction countdown, in whole seconds.
pub const PREDICTION_SECONDS: u32 = 5;
/// 14_h1's question and answers, as the full script's holds table has them.
const QUESTION: [&str; 2] = [
    "No spin anywhere: is a coarse cell",
    "equally good everywhere along the line?",
];
const OPTIONS: [&str; 3] = ["yes", "no, worse near the cylinder", "no, worse far out"];

/// The opening's question card, held as a strip from 14_2 to the end.
const QUESTION_CARD: &str = "Does halving the cells change the answer?";
const RULE: &str = "split where the air spins";
const STEPS_NOTE: &str = "smaller steps too (ch. 12)";
const ADAPTIVE: &str = "adaptive grid";
const OPEN_READOUT: &str = "readout: cells ?";
const MULTIGRID_NOTE: &str = "multigrid: inside the pressure solve (ch. 10)";
const SCHEMATIC: &str = "schematic";
const ANALYTIC_FLOW: &str = "analytic: an exact formula, nothing simulated";
const ANALYTIC_ROW: &str = "analytic speeds, geometric cells, nothing simulated";
const CELL_WIDTH: &str = "2 radii";
const SPEED_AXIS: &str = "speed";
const LEGEND: [&str; 3] = [
    "speed at a cell end",
    "straight-line guess",
    "exact speed in the middle",
];
const MISS: &str = "miss";
const UNIFORM_TITLE: &str = "uniform strain: u = −x, v = y";
const SLOPE: &str = "slope keeps changing";
const CARDS_14_11: [&str; 3] = ["thin layers", "wall shape (ch. 17)", "the quantity we want"];
const NOT_ENOUGH: &str = "not enough alone";
const SEAM: &str = "seam";

/// 14_12's source card as the storyboard quotes it, one part per sentence gate.
const CARD_14_12: [&str; 3] = [
    "Basilisk adapt_wavelet:",
    "refine if e > emax;",
    "coarsen if e ≤ emax/1.5",
];
/// Basilisk's cylinder example, `examples/karman.c` line 146 without its semicolon.
const KARMAN_CALL: &str = "adapt_wavelet ({cs,u,f}, {1e-2,3e-2,3e-2,3e-2}, maxlevel, 4)";
const KARMAN_NOTE: &str = "its cylinder example, examples/karman.c line 146";

/// The Basilisk lines 14_9 and 14_12 rest on, as BronzeJaguar's reader `ad02` found them in the
/// pinned tarball (basilisk-20261025.tar.gz): fact key, file and line, text.
pub const BASILISK: [(&str, &str, &str); 7] = [
    (
        "bas.adapt.signature",
        "basilisk/src/grid/tree-common.h:173",
        "astats adapt_wavelet (scalar * slist,",
    ),
    (
        "bas.adapt.predict",
        "basilisk/src/grid/tree-common.h:242",
        "s.prolongation (point, s);",
    ),
    (
        "bas.adapt.miss",
        "basilisk/src/grid/tree-common.h:246",
        "double e = fabs(*b - s[]);",
    ),
    (
        "bas.adapt.refine",
        "basilisk/src/grid/tree-common.h:247",
        "if (e > emax && level < maxlevel) {",
    ),
    (
        "bas.adapt.coarsen",
        "basilisk/src/grid/tree-common.h:251",
        "else if ((e <= emax/1.5 || level > maxlevel) &&",
    ),
    (
        "bas.default.prolong",
        "basilisk/src/grid/multigrid-common.h:261",
        "s.prolongation = refine_bilinear;",
    ),
    (
        "bas.karman.adapt",
        "basilisk/src/examples/karman.c:146",
        "adapt_wavelet ({cs,u,f}, {1e-2,3e-2,3e-2,3e-2}, maxlevel, 4);",
    ),
];

/// The Precisely lower thirds, each the full script's text split into lines.
const PRECISELY_14_4: [&str; 3] = [
    "Potential flow, radius 1, stream speed 1: u = 1 − (x² − y²)/r⁴, v = −2xy/r⁴.",
    "Spin and divergence are zero everywhere, and nothing crosses the circle.",
    "Unlike the real flow, it slips along the wall and has no wake and no drag.",
];
const PRECISELY_14_5: [&str; 2] = [
    "This measures how well a grid can describe a known flow, not the error a solver makes.",
    "Real indicators are stand-ins for the error.",
];
const PRECISELY_14_10: [&str; 3] = [
    "The miss equals (h²/8)|u_xx| at some point of the cell (h is the cell width). On this line u_xx = −6/x⁴.",
    "From the near cell to the third, strain falls 27-fold, u_xx at the middles 81-fold and the miss about 126-fold.",
    "A uniform strain (u = −x, v = y) has no spin and no miss.",
];
const PRECISELY_14_11: [&str; 3] = [
    "The ideal flow slips along the wall; a real one sticks to it, which makes the thin layer.",
    "Vorticity measures spin. A straight-line guess can miss even where spin is zero.",
    "A useful refinement test also depends on the wall and the quantity we need.",
];
const PRECISELY_14_12: [&str; 3] = [
    "Basilisk predicts from the parent cell with the field's prolongation (bilinear by default),",
    "not from point values at cell ends as our row does. The example's tolerances are in its own units.",
    "cs: the fraction of each cell that is fluid (0 inside the cylinder); \
     u: velocity (two components); f: a passive tracer.",
];

/// Pacing, in seconds of scene time. No number on screen depends on these.
pub mod pacing {
    pub const FADE: f64 = 0.5;
    pub const MORPH: f64 = 0.9;
    /// 14_4: the streamlines draw from left to right.
    pub const DRAW: f64 = 2.4;
    pub const SPLIT: f64 = 0.6;
    pub const RISE: f64 = 0.9;
    pub const PULSE: f64 = 0.8;
    pub const BEAT: f64 = 0.4;
    /// 14_9: hold the two failed bars until the split clause in the retained take.
    pub const SECOND_SPLIT_HOLD: f64 = 2.6;
    /// 14_10: the two squares of air deform.
    pub const SQUEEZE: f64 = 1.6;
}
use pacing::{BEAT, DRAW, FADE, MORPH, PULSE, RISE, SPLIT, SQUEEZE};

// Layout, in scene units (frame 16 × 9; the opening's safe area is x ∈ [−7.1, 7.1], y ∈ [−4, 4], with
// the main window left of x = 0.2 and the side panel right of x = 0.4). From scene03's proposals.
/// 14_1: the full tunnel and the question card under it, where chapter 09 ends.
const TUNNEL_WIDTH: f64 = 11.5;
const TUNNEL_CENTER: [f64; 2] = [0.0, 1.15];
/// The question card exactly as chapter 09 leaves it (frame 8.2 × 0.8 at (0, −1.25), words 30),
/// then held as a strip at the side panel's top.
const CARD_FULL: ([f64; 2], [f64; 2], f64) = ([0.0, -1.25], [8.2, 0.8], 30.0);
const CARD_STRIP: ([f64; 2], [f64; 2], f64) = ([3.65, 3.55], [6.4, 0.56], 21.0);
/// The main window's right edge.
const MAIN_RIGHT: f64 = 0.2;
/// 14_2: the schematic cell and the multigrid stack.
const SCHEMATIC_AT: [f64; 2] = [-3.6, 0.3];
const SCHEMATIC_SIDE: f64 = 2.6;
const STACK_Y: f64 = -0.25;
const STACK_X: [f64; 3] = [1.85, 3.65, 5.45];
const STACK_SIDE: f64 = 1.2;
const STACK_CELLS: [u32; 3] = [8, 4, 2];
/// The rule card, big in 14_3 and then held in the side panel to 14_11.
const RULE_BIG: ([f64; 2], [f64; 2], f64) = ([-3.45, 0.4], [6.4, 0.95], 32.0);
const RULE_SMALL: ([f64; 2], [f64; 2], f64) = ([3.65, 2.75], [6.0, 0.56], 22.0);
const SIDE_X: f64 = 3.65;
/// 14_4: the ideal flow, cylinder radius FLOW_SCALE, streamlines over |x| ≤ FLOW_REACH radii.
const FLOW_CENTER: [f64; 2] = [-3.45, 0.2];
const FLOW_SCALE: f64 = 0.9;
const FLOW_REACH: f64 = 4.0;
const STREAMS: [f64; 5] = [0.18, 0.5, 0.9, 1.4, 2.0];
/// The speed readouts sit between the streamlines c = 0.9 and c = 1.4, above and below the shoulders.
const SHOULDER_LABEL: f64 = 1.73;
/// 14_5 to 14_10: the row, ROW_SCALE scene units per radius, the cylinder's centre at ROW_CENTER.
const ROW_SCALE: f64 = 0.48;
const ROW_CENTER: [f64; 2] = [-0.4, 0.2];
/// The speed graph above the row: u = 0 at GRAPH_ZERO, u = 1 at GRAPH_ZERO + GRAPH_HEIGHT.
const GRAPH_ZERO: f64 = 0.95;
const GRAPH_HEIGHT: f64 = 1.6;
const GRAPH_LEFT: f64 = -6.75;
const GRAPH_RIGHT: f64 = -0.75;
/// Speed arrows, centred on the cell ends: scene units per stream speed.
const ARROW_PER_SPEED: f64 = 0.8;
/// Miss bars stand on BAR_BASE, BAR_SCALE scene units per unit of miss (0.02 of miss per scene
/// unit), clipped at BAR_CLIP with a break mark; the tolerance line is BAR_SCALE × 0.01 above.
const BAR_BASE: f64 = -2.3;
const BAR_SCALE: f64 = 50.0;
const BAR_CLIP: f64 = 1.5;
const LEGEND_Y: [f64; 3] = [-0.8, -1.2, -1.6];
const PREDICT_AT: [f64; 2] = [3.65, 0.9];
const PREDICT_SIZE: [f64; 2] = [6.4, 2.75];
/// 14_10: the squares of air deform as the local strain would over FLOW_TIME (radius over stream
/// speed), drawn as e^(∓strain × time); an illustration of the rate, not a tracked parcel.
const SQUARE: f64 = 0.36;
const FLOW_TIME: f64 = 2.0;
const INSET_AT: [f64; 2] = [3.65, 0.25];
const INSET_SIZE: [f64; 2] = [6.0, 3.6];
const PRECISELY_Y: f64 = -3.3;
/// 14_13: chapter 15's seam, big cell x ∈ [−5.0, −2.6], y ∈ [−1.4, 1.0] (scene03 layout).
const SEAM_BIG: [f64; 4] = [-5.0, -2.6, -1.4, 1.0];
/// A split square has 2 × 2 children.
const CHILDREN: usize = 4;

/// The indicator miss: magenta, a role used for nothing else (scene03's colour table, root J4).
fn miss_color() -> Srgb {
    hex("#E84FD0")
}

fn card_fill() -> Srgb {
    hex("#182433")
}

/// White, the opening's teaching highlight.
fn glow() -> Srgb {
    hex("#FFFFFF")
}

fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

/// The opening's tunnel in metres (story.rs passes the retained snapshot's geometry, as for chapter 05).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TunnelShape {
    pub length: f64,
    pub height: f64,
    /// Centre x, centre y, radius.
    pub cylinder: [f64; 3],
}

pub struct Chapter14 {
    pub kit: Kit,
    pub tunnel: Option<TunnelShape>,
}

// ------------------------------------------------------------------------------------ numbers

/// Every number-bearing label the chapter draws, built from `adapt_row`'s exact values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    pub four_cells: String,
    pub shoulders: Vec<String>,
    pub spin_points: Vec<String>,
    pub spin_cells: Vec<String>,
    pub third: String,
    pub near: String,
    pub guess: String,
    pub exact: String,
    pub worse: String,
    pub tolerance: String,
    pub second: String,
    pub cells_first: String,
    pub spin_rule: String,
    pub cells_second: String,
    pub strain_near: String,
    pub strain_third: String,
    pub uniform_spin: String,
    pub uniform_strain: String,
    pub uniform_miss: String,
}

pub fn labels() -> Labels {
    let row = adapt_row::coarse_row();
    let [first, second] = adapt_row::rounds();
    let ratios = adapt_row::near_against_third();
    let uniform = adapt_row::uniform_gradient(Q::ZERO, Q::ZERO);
    let uniform_miss = row.iter().map(|&c| adapt_row::uniform_miss(c)).max().unwrap_or(Q::ZERO);
    Labels {
        four_cells: format!("× {CHILDREN} cells"),
        shoulders: adapt_row::shoulders()
            .iter()
            .map(|&(x, y)| format!("{} ×", adapt_row::velocity(x, y).0))
            .collect(),
        spin_points: adapt_row::marked_points()
            .iter()
            .map(|&(x, y)| format!("spin {}", adapt_row::spin(x, y)))
            .collect(),
        spin_cells: row
            .iter()
            .map(|c| format!("spin {}", adapt_row::spin(c.middle(), Q::ZERO)))
            .collect(),
        third: adapt_row::decimal(row[2].miss(), 3),
        near: adapt_row::decimal(row[0].miss(), 2),
        guess: row[0].guess().to_string(),
        exact: row[0].exact().to_string(),
        worse: format!("about {} × worse near the cylinder", adapt_row::decimal(ratios.miss, 0)),
        tolerance: format!("tolerance {}", adapt_row::decimal(adapt_row::tolerance(), 2)),
        second: adapt_row::decimal(row[1].miss(), 3),
        cells_first: format!(
            "cells {} → {} (halve all: {})",
            first.cells.len(),
            first.after.len(),
            adapt_row::halve_all(&row).len()
        ),
        spin_rule: format!("spin rule: {}", adapt_row::spin_rule(&row).len()),
        cells_second: format!(
            "cells {} (halve all twice: {})",
            second.after.len(),
            adapt_row::halve_all(&adapt_row::halve_all(&row)).len()
        ),
        strain_near: format!("strain {}", adapt_row::strain(row[0].middle()).abs()),
        strain_third: format!("strain {}", adapt_row::strain(row[2].middle()).abs()),
        uniform_spin: format!("spin {}", uniform[2] - uniform[1]),
        uniform_strain: format!("strain {}", uniform[0].abs()),
        uniform_miss: format!("miss {uniform_miss}"),
    }
}

/// `(a, b, c)` with each rational as `four_cell::Q` prints it, the oracle's tuple form.
fn tuple(values: impl IntoIterator<Item = Q>) -> String {
    let parts: Vec<String> = values.into_iter().map(|q| q.to_string()).collect();
    format!("({})", parts.join(", "))
}

/// The chapter's numbers alone, checked (a preview's `facts`).
pub fn facts() -> Result<Facts, String> {
    let mut facts = Facts::new();
    register(&mut facts)?;
    facts.self_check()?;
    Ok(facts)
}

/// Add chapter 14's numbers to a film's registry. Each is pinned to the exact value BronzeJaguar's
/// oracles printed (navier-adaptive-scene02/runs: ad01-run01, ad03-run01 and ad04-run02, joined in
/// oracle-keys02.txt); the readouts the chapter rounds are pinned to the storyboard's strings.
pub fn register(facts: &mut Facts) -> Result<(), String> {
    let row = adapt_row::coarse_row();
    let [first, second] = adapt_row::rounds();
    let ratios = adapt_row::near_against_third();
    let marked = adapt_row::marked_points();
    let uniform = adapt_row::uniform_gradient(Q::ZERO, Q::ZERO);
    let all_zero = |points: &[(Q, Q)], f: fn(Q, Q) -> Q| points.iter().all(|&(x, y)| f(x, y).is_zero());
    let mut line_points: Vec<(Q, Q)> = adapt_row::row_points().iter().map(|&x| (x, Q::ZERO)).collect();
    line_points.extend(row.iter().map(|c| (c.middle(), Q::ZERO)));
    let shown = labels();
    let ad01 = "ad01-run01 (navier-adaptive-scene02)";
    let ad03 = "ad03-run01 (navier-adaptive-scene02)";
    let ad04 = "ad04-run02 (navier-adaptive-scene02)";
    let story = "navier-full-script03.md chapter 14 picture";
    facts
        .pin("ideal.spin.zero", all_zero(&marked, adapt_row::spin), "true", ad01)?
        .pin("ideal.div.zero", all_zero(&marked, adapt_row::divergence), "true", ad01)?
        .pin(
            "ideal.wall.no.flow",
            all_zero(&adapt_row::wall_points(), adapt_row::wall_flow),
            "true",
            ad01,
        )?
        .pin(
            "ideal.speed.shoulder",
            adapt_row::velocity(Q::ZERO, Q::ONE).0,
            "2",
            ad01,
        )?
        .pin(
            "row.spin.zero.offline",
            all_zero(&adapt_row::offline_points(), adapt_row::spin),
            "true",
            ad03,
        )?
        .pin(
            "row.points",
            tuple(adapt_row::row_points()),
            "(-1, -3, -5, -7, -9, -11, -13)",
            ad03,
        )?
        .pin(
            "row.u",
            tuple(adapt_row::row_points().map(adapt_row::speed)),
            "(0, 8/9, 24/25, 48/49, 80/81, 120/121, 168/169)",
            ad03,
        )?
        .pin("ideal.u.wall", adapt_row::speed(Q::int(-1)), "0", ad01)?
        .pin("ideal.u.near.far.end", adapt_row::speed(row[0].outer), "8/9", ad01)?
        .pin("ideal.spin.near", adapt_row::spin(row[0].middle(), Q::ZERO), "0", ad01)?
        .pin("ideal.spin.far", adapt_row::spin(row[2].middle(), Q::ZERO), "0", ad01)?
        .pin("row.spin.zero", all_zero(&line_points, adapt_row::spin), "true", ad03)?
        .pin("row.miss.third", row[2].miss(), "107/44100", ad03)?
        .pin("ideal.far.miss", row[2].miss(), "107/44100", ad01)?
        .pin("row.miss.near", row[0].miss(), "11/36", ad03)?
        .pin("ideal.near.miss", row[0].miss(), "11/36", ad01)?
        .pin("ideal.near.guess", row[0].guess(), "4/9", ad01)?
        .pin("ideal.u.near.mid", row[0].exact(), "3/4", ad01)?
        .pin("ideal.miss.ratio", ratios.miss, "13475/107", ad01)?
        .pin(
            "row.miss.decreasing",
            row.windows(2).all(|w| w[0].miss() > w[1].miss()),
            "true",
            ad03,
        )?
        .pin(
            "row.misses",
            tuple(row.map(Cell::miss)),
            "(11/36, 47/3600, 107/44100, 191/254016, 299/980100, 431/2944656)",
            ad03,
        )?
        .pin("row.tol", adapt_row::tolerance(), "1/100", ad03)?
        .pin("row.miss.second", row[1].miss(), "47/3600", ad03)?
        .pin(
            "row.split.nearest.two",
            first.split == [true, true, false, false, false, false],
            "true",
            ad03,
        )?
        .pin("row.cells.before", first.cells.len(), "6", ad03)?
        .pin("row.cells.after", first.after.len(), "8", ad03)?
        .pin("row.cells.all.halved", adapt_row::halve_all(&row).len(), "12", ad03)?
        .pin(
            "row.spin.rule.splits.none",
            adapt_row::spin_rule(&row).len() == row.len(),
            "true",
            ad03,
        )?
        .pin(
            "row2.misses",
            tuple(first.made().into_iter().map(Cell::miss)),
            "(13/72, 37/1800, 73/14112, 121/64800)",
            ad03,
        )?
        .pin(
            "row2.split.wall.two",
            adapt_row::test_row(&first.made(), adapt_row::tolerance()).split == [true, true, false, false],
            "true",
            ad03,
        )?
        .pin("row2.cells.after", second.after.len(), "10", ad03)?
        .pin(
            "row2.cells.all.halved.twice",
            adapt_row::halve_all(&adapt_row::halve_all(&row)).len(),
            "24",
            ad03,
        )?
        .pin("row.strain.near", adapt_row::strain(row[0].middle()).abs(), "1/4", ad03)?
        .pin(
            "row.strain.third",
            adapt_row::strain(row[2].middle()).abs(),
            "1/108",
            ad03,
        )?
        .pin("ideal.strain.near", adapt_row::strain(row[0].middle()), "-1/4", ad01)?
        .pin("ideal.strain.far", adapt_row::strain(row[2].middle()), "-1/108", ad01)?
        .pin(
            "curv.miss.bounded.all",
            row.iter().all(|c| {
                let (lo, hi) = c.miss_bounds();
                lo <= c.miss() && c.miss() <= hi
            }),
            "true",
            ad04,
        )?
        .pin(
            "curv.near.mid",
            adapt_row::curvature(row[0].middle()).abs(),
            "3/8",
            ad04,
        )?
        .pin(
            "curv.third.mid",
            adapt_row::curvature(row[2].middle()).abs(),
            "1/216",
            ad04,
        )?
        .pin("curv.ratio.strain", ratios.strain, "27", ad04)?
        .pin("curv.ratio.curvature", ratios.curvature, "81", ad04)?
        .pin("curv.ratio.miss", ratios.miss, "13475/107", ad04)?
        .pin(
            "curv.fastest.near",
            adapt_row::row_points()
                .iter()
                .all(|&x| adapt_row::curvature(Q::int(-1)).abs() >= adapt_row::curvature(x).abs()),
            "true",
            ad04,
        )?
        .pin("ustrain.spin", uniform[2] - uniform[1], "0", ad04)?
        .pin("ustrain.div", uniform[0] + uniform[3], "0", ad04)?
        .pin("ustrain.strain", uniform[0].abs(), "1", ad04)?
        .pin(
            "ustrain.miss.zero",
            row.iter().all(|&c| adapt_row::uniform_miss(c).is_zero()),
            "true",
            ad04,
        )?
        .pin(
            "ustrain.linear.check",
            adapt_row::row_points()
                .iter()
                .all(|&x| adapt_row::uniform_gradient(x, Q::ZERO)[0] == -Q::ONE),
            "true",
            ad04,
        )?
        .pin("row.miss.near.shown", &shown.near, "0.31", story)?
        .pin("row.miss.second.shown", &shown.second, "0.013", story)?
        .pin("row.miss.third.shown", &shown.third, "0.002", story)?
        .pin(
            "row.tol.shown",
            &shown.tolerance,
            "tolerance 0.01",
            "Agent Mail 36273: the 0.01 threshold drawn",
        )?
        .pin(
            "ideal.miss.ratio.shown",
            adapt_row::decimal(ratios.miss, 0),
            "126",
            "14_10 Precisely: about 126-fold",
        )?
        .add("row.prediction.seconds", PREDICTION_SECONDS)?;
    for (key, source, text) in BASILISK {
        facts.add(key, format!("{source} {text}"))?;
    }
    Ok(())
}

// ----------------------------------------------------------------------------------- geometry

fn row_x(x: Q) -> f64 {
    ROW_CENTER[0] + ROW_SCALE * x.to_f64()
}

fn graph_y(u: Q) -> f64 {
    GRAPH_ZERO + GRAPH_HEIGHT * u.to_f64()
}

/// Where the speed at x sits on the graph.
fn graph_point(x: Q) -> [f64; 2] {
    [row_x(x), graph_y(adapt_row::speed(x))]
}

/// The flow view's scene point for (x, y) in radii.
fn flow_at(x: f64, y: f64) -> [f64; 2] {
    [FLOW_CENTER[0] + FLOW_SCALE * x, FLOW_CENTER[1] + FLOW_SCALE * y]
}

/// The row view's scene point for (x, y) in radii.
fn row_at(x: f64, y: f64) -> [f64; 2] {
    [ROW_CENTER[0] + ROW_SCALE * x, ROW_CENTER[1] + ROW_SCALE * y]
}

/// A miss bar's height in scene units, and whether it is clipped.
fn bar_height(miss: Q) -> (f64, bool) {
    let h = BAR_SCALE * miss.to_f64();
    if h > BAR_CLIP { (BAR_CLIP, true) } else { (h, false) }
}

/// The tolerance line's height above the bars' base.
fn tolerance_height() -> f64 {
    BAR_SCALE * adapt_row::tolerance().to_f64()
}

/// A rough text width: about half an em per character.
fn text_width(words: &str, size: f64) -> f64 {
    words.chars().count() as f64 * size * 0.0052
}

/// The line number in a `file:line` pin.
fn pin_line(pin: &str) -> &str {
    pin.rsplit(':').next().unwrap_or(pin)
}

// ----------------------------------------------------------------------------------- builders

fn p2(at: [f64; 2]) -> Vec3 {
    v(at[0], at[1])
}

fn line(stage: &mut Stage<'_>, a: [f64; 2], b: [f64; 2], color: Srgb, width: f64) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Line::new(p2(a), p2(b))
            .style(Style::default().stroke(color, width, 1.0))
            .build()?,
    ))
}

fn dashed(stage: &mut Stage<'_>, a: [f64; 2], b: [f64; 2], color: Srgb, width: f64) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        DashedLine::new(p2(a), p2(b))
            .dash_length(0.07)
            .style(Style::default().stroke(color, width, 1.0))
            .build()?,
    ))
}

/// A sharp rectangle centred at `at`: fill colour and opacity, edge colour and width (0 for none).
fn rect(stage: &mut Stage<'_>, at: [f64; 2], size: [f64; 2], fill: (Srgb, f64), edge: (Srgb, f64)) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Rectangle::new()
            .width(size[0])
            .height(size[1])
            .style(Style::default().fill(fill.0, fill.1).stroke(edge.0, edge.1, 1.0))
            .build()?
            .moved_to(p2(at)),
    ))
}

/// The same with rounded corners, for cards.
fn rounded(
    stage: &mut Stage<'_>,
    at: [f64; 2],
    size: [f64; 2],
    fill: (Srgb, f64),
    edge: (Srgb, f64),
) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Rectangle::new()
            .width(size[0])
            .height(size[1])
            .corner_radius(0.1)
            .style(Style::default().fill(fill.0, fill.1).stroke(edge.0, edge.1, 1.0))
            .build()?
            .moved_to(p2(at)),
    ))
}

fn ring(stage: &mut Stage<'_>, at: [f64; 2], radius: f64, color: Srgb, width: f64) -> Mob {
    stage.arena_mut().add(
        Circle::new()
            .radius(radius)
            .style(Style::default().fill(color, 0.0).stroke(color, width, 1.0))
            .build()
            .moved_to(p2(at)),
    )
}

fn disc(stage: &mut Stage<'_>, at: [f64; 2], radius: f64, color: Srgb) -> Mob {
    stage.arena_mut().add(
        Dot::new()
            .point(p2(at))
            .radius(radius)
            .style(Style::default().fill(color, 1.0).stroke(color, 0.0, 1.0))
            .build(),
    )
}

fn arrow(stage: &mut Stage<'_>, from: [f64; 2], to: [f64; 2], color: Srgb, thickness: f64) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Arrow::new(p2(from), p2(to))
            .buff(0.0)
            .thickness(thickness)
            .max_tip_length_to_length_ratio(0.45)
            .color(color)
            .build()?,
    ))
}

fn curve(stage: &mut Stage<'_>, points: &[[f64; 2]], color: Srgb, width: f64, opacity: f64) -> Mob {
    stage.arena_mut().add(
        polyline(points.iter().map(|&p| p2(p)))
            .style(Style::default().fill(color, 0.0).stroke(color, width, opacity))
            .build(),
    )
}

/// A cylinder: solid fill and a light-ink outline, as the tunnel draws it.
fn cylinder(stage: &mut Stage<'_>, at: [f64; 2], radius: f64) -> Mob {
    stage.arena_mut().add(
        Circle::new()
            .radius(radius)
            .style(
                Style::default()
                    .fill(hex("#0A121F"), 1.0)
                    .stroke(Palette::ink(), 1.6, 1.0),
            )
            .build()
            .moved_to(p2(at)),
    )
}

/// The exact streamlines ψ = ±c, each left to right, nearest the axis first: across the flow view,
/// or (`row`) at the row's scale out to the main window's right edge, where 14_5 rescales them.
fn streamlines(stage: &mut Stage<'_>, row: bool) -> Vec<Mob> {
    let reach = if row {
        (MAIN_RIGHT - ROW_CENTER[0]) / ROW_SCALE
    } else {
        FLOW_REACH
    };
    let mut out = Vec::new();
    for c in STREAMS {
        for c in [c, -c] {
            let points: Vec<[f64; 2]> = adapt_row::streamline(c, reach, 241)
                .iter()
                .map(|&[x, y]| if row { row_at(x, y) } else { flow_at(x, y) })
                .collect();
            out.push(curve(stage, &points, Palette::ink(), 1.4, 0.55));
        }
    }
    out
}

/// The dividing streamline's two halves on the axis, in front of and behind the cylinder: across
/// the flow view, or in the row view from the graph's left end to the main window's right edge.
fn axis(stage: &mut Stage<'_>, row: bool) -> fmn::Result<Vec<Mob>> {
    let at = |x: f64| if row { row_at(x, 0.0) } else { flow_at(x, 0.0) };
    let (front, behind) = if row {
        (
            (GRAPH_LEFT - ROW_CENTER[0]) / ROW_SCALE,
            (MAIN_RIGHT - ROW_CENTER[0]) / ROW_SCALE,
        )
    } else {
        (-FLOW_REACH, FLOW_REACH)
    };
    Ok(vec![
        line(stage, at(front), at(-1.0), Palette::dim(), 1.4)?,
        line(stage, at(1.0), at(behind), Palette::dim(), 1.4)?,
    ])
}

/// The static tunnel and its teaching grid, drawn as chapter 09's `tunnel` draws them.
fn tunnel_drawing(stage: &mut Stage<'_>, g: TunnelShape, width: f64, center: [f64; 2]) -> fmn::Result<Vec<Mob>> {
    let scale = width / g.length;
    let at = |x: f64, y: f64| {
        [
            center[0] + scale * (x - g.length / 2.0),
            center[1] + scale * (y - g.height / 2.0),
        ]
    };
    let [cx, cy, radius] = g.cylinder;
    let mut out = vec![
        rect(
            stage,
            center,
            [width, scale * g.height],
            (hex("#17283C"), 1.0),
            (Palette::dim(), 1.7),
        )?,
        cylinder(stage, at(cx, cy), scale * radius),
    ];
    let cell = g.height / 3.0;
    for row in 1..3 {
        let y = f64::from(row) * cell;
        out.push(line(stage, at(0.0, y), at(g.length, y), Palette::faint(), 1.1)?);
    }
    let middle = g.length / 2.0;
    let half_columns = (middle / cell).floor() as i32;
    for column in -half_columns..=half_columns {
        let x = middle + f64::from(column) * cell;
        out.push(line(stage, at(x, 0.0), at(x, g.height), Palette::faint(), 1.1)?);
    }
    Ok(out)
}

/// The teaching-grid cell 14_1 splits: in the middle row, the first cell wholly behind the
/// cylinder. Its two split lines and its centre, in scene units.
fn grid_split(
    stage: &mut Stage<'_>,
    g: TunnelShape,
    width: f64,
    center: [f64; 2],
) -> fmn::Result<(Vec<Mob>, [f64; 2])> {
    let scale = width / g.length;
    let cell = g.height / 3.0;
    let middle = g.length / 2.0;
    let [cx, _, radius] = g.cylinder;
    let left = middle + ((cx + radius - middle) / cell).ceil() * cell;
    if left + cell > middle + (middle / cell).floor() * cell {
        return Err(fail(format!(
            "chapter 14: no whole grid cell behind the cylinder in {g:?}"
        )));
    }
    let c = [
        center[0] + scale * (left + cell / 2.0 - g.length / 2.0),
        center[1] + scale * (1.5 * cell - g.height / 2.0),
    ];
    let half = scale * cell / 2.0;
    Ok((
        vec![
            line(stage, [c[0] - half, c[1]], [c[0] + half, c[1]], Palette::ink(), 1.6)?,
            line(stage, [c[0], c[1] - half], [c[0], c[1] + half], Palette::ink(), 1.6)?,
        ],
        c,
    ))
}

/// The two lines that split a square of side `side` centred at `at` into its four children.
fn quarter_lines(stage: &mut Stage<'_>, at: [f64; 2], side: f64, color: Srgb, width: f64) -> fmn::Result<Vec<Mob>> {
    let h = side / 2.0;
    Ok(vec![
        line(stage, [at[0] - h, at[1]], [at[0] + h, at[1]], color, width)?,
        line(stage, [at[0], at[1] - h], [at[0], at[1] + h], color, width)?,
    ])
}

/// One of multigrid's whole grids: a square with `n` × `n` cells.
fn whole_grid(stage: &mut Stage<'_>, at: [f64; 2], side: f64, n: u32) -> fmn::Result<Vec<Mob>> {
    let mut out = vec![rect(
        stage,
        at,
        [side, side],
        (Palette::faint(), 0.35),
        (Palette::dim(), 1.4),
    )?];
    let h = side / 2.0;
    for k in 1..n {
        let s = -h + side * f64::from(k) / f64::from(n);
        out.push(line(
            stage,
            [at[0] + s, at[1] - h],
            [at[0] + s, at[1] + h],
            Palette::dim(),
            0.9,
        )?);
        out.push(line(
            stage,
            [at[0] - h, at[1] + s],
            [at[0] + h, at[1] + s],
            Palette::dim(),
            0.9,
        )?);
    }
    Ok(out)
}

/// A miss bar on the row (with its break mark when clipped), and the flat bar it rises from.
fn miss_bar(stage: &mut Stage<'_>, cell: Cell) -> fmn::Result<(Mob, Mob, Vec<Mob>)> {
    let x = row_x(cell.middle());
    let width = 0.5 * ROW_SCALE * cell.width().to_f64();
    let (h, clipped) = bar_height(cell.miss());
    let style = (miss_color(), 0.85);
    let flat = rect(stage, [x, BAR_BASE + 0.001], [width, 0.002], style, (miss_color(), 0.0))?;
    let full = rect(stage, [x, BAR_BASE + h / 2.0], [width, h], style, (miss_color(), 0.0))?;
    let mut mark = Vec::new();
    if clipped {
        let (y, w) = (BAR_BASE + h - 0.3, width / 2.0 + 0.06);
        for dy in [0.0, 0.1] {
            mark.push(line(
                stage,
                [x - w, y + dy - 0.05],
                [x + w, y + dy + 0.05],
                hex(BACKGROUND),
                4.0,
            )?);
        }
    }
    Ok((flat, full, mark))
}

// ------------------------------------------------------------------------------------ staging

/// The row's drawn pieces, keyed by what they show.
#[derive(Default)]
struct Row {
    cells: Vec<Mob>,
    dividers: Vec<Mob>,
    arrows: Vec<Mob>,
    dots: Vec<(Q, Mob)>,
    dashes: Vec<(Cell, Mob)>,
    rings: Vec<(Q, Mob)>,
    spins: Vec<(Cell, Mob)>,
    bars: Vec<(Cell, Vec<Mob>)>,
}

fn take<K: PartialEq + Copy, T>(list: &mut Vec<(K, T)>, key: K) -> Option<T> {
    let i = list.iter().position(|(k, _)| *k == key)?;
    Some(list.remove(i).1)
}

/// The chapter's state: what is on screen, and a pending batch of animations.
struct Ch14<'a> {
    k: &'a Kit,
    on: Vec<Mob>,
    pending: Vec<Box<dyn Animation>>,
}

impl Ch14<'_> {
    // Text through the kit, so the layout lint and the glyph check see every string.
    fn text(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: [f64; 2]) -> fmn::Result<Mob> {
        self.k.text(stage, s, size, color, p2(at))
    }

    fn text_left(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: [f64; 2]) -> fmn::Result<Mob> {
        self.k.text_left(stage, s, size, color, p2(at))
    }

    fn mono(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: [f64; 2]) -> fmn::Result<Mob> {
        self.k.mono(stage, s, size, color, p2(at))
    }

    /// Put on screen at once.
    fn put(&mut self, stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
        for &m in mobs {
            if !self.on.contains(&m) {
                stage.add_to_scene(m)?;
                self.on.push(m);
            }
        }
        Ok(())
    }

    /// Remove at once (no fade).
    fn cut(&mut self, stage: &mut Stage<'_>, mobs: &[Mob]) {
        let gone: Vec<Mob> = mobs.iter().copied().filter(|m| self.on.contains(m)).collect();
        stage.scene_mut().remove(&gone);
        self.on.retain(|m| !gone.contains(m));
    }

    fn appear(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], secs: f64) -> fmn::Result<()> {
        for &m in mobs {
            if !self.on.contains(&m) {
                let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(secs);
                self.pending.push(stage.prepare(a)?);
                self.on.push(m);
            }
        }
        Ok(())
    }

    /// Fade out what is on screen; anything already gone is skipped (no second fade).
    fn vanish(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], secs: f64) -> fmn::Result<()> {
        for &m in mobs {
            if self.on.contains(&m) {
                let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(secs);
                self.pending.push(stage.prepare(a)?);
                self.on.retain(|&x| x != m);
            }
        }
        Ok(())
    }

    /// `from` (on screen) takes `to`'s shape and paint; `to` stays detached.
    fn morph(&mut self, stage: &mut Stage<'_>, from: Mob, to: Mob, secs: f64) -> fmn::Result<()> {
        if self.on.contains(&from) {
            self.pending.push(stage.prepare(Transform::new(from, to).rt(secs))?);
        }
        Ok(())
    }

    /// Draw along their paths.
    fn draw(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], secs: f64) -> fmn::Result<()> {
        for &m in mobs {
            if !self.on.contains(&m) {
                self.pending.push(stage.prepare(show_creation(m).rt(secs))?);
                self.on.push(m);
            }
        }
        Ok(())
    }

    fn pulse(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], color: Option<[f32; 3]>) -> fmn::Result<()> {
        for &m in mobs {
            if self.on.contains(&m) {
                let a = indicate(stage.arena_mut(), m, 1.15, color)?.rt(PULSE);
                self.pending.push(stage.prepare(a)?);
            }
        }
        Ok(())
    }

    fn play(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        if !self.pending.is_empty() {
            stage.play_prepared(std::mem::take(&mut self.pending))?;
        }
        Ok(())
    }

    /// Finish what is queued, then hold the picture.
    fn pause(&mut self, stage: &mut Stage<'_>, secs: f64) -> fmn::Result<()> {
        self.play(stage)?;
        stage.wait(secs)?;
        Ok(())
    }

    /// Finish what is queued, then start line `id`.
    fn say(&mut self, stage: &mut Stage<'_>, id: &str) -> fmn::Result<()> {
        if !CUES.iter().any(|c| c.0 == id) {
            return Err(fail(format!("chapter 14 has no cue {id}")));
        }
        self.play(stage)?;
        println!("Chapter 14 cue {id} at {:.6} s", stage.scene().time().to_f64());
        self.k.say(stage, id)
    }

    /// Finish what is queued, then wait for sentence `j` of line `id` (nothing if it has begun).
    fn until(&mut self, stage: &mut Stage<'_>, id: &str, j: usize) -> fmn::Result<()> {
        let sentences = CUES.iter().find(|c| c.0 == id).map_or(0, |c| c.1);
        if j >= sentences {
            return Err(fail(format!("chapter 14 line {id} has no sentence {j}")));
        }
        self.play(stage)?;
        self.k.until_sentence(stage, id, j)
    }

    /// A card: rounded frame and centred words.
    fn card(
        &self,
        stage: &mut Stage<'_>,
        place: ([f64; 2], [f64; 2], f64),
        words: &str,
        edge: Srgb,
    ) -> fmn::Result<[Mob; 2]> {
        let (at, size, font) = place;
        Ok([
            rounded(stage, at, size, (card_fill(), 1.0), (edge, 1.6))?,
            self.text(stage, words, font, Palette::ink(), at)?,
        ])
    }

    /// The opening's question card, drawn as chapter 09 draws it: an open dim frame (corner 0.12,
    /// stroke 2.0) and ink words.
    fn question(&self, stage: &mut Stage<'_>, place: ([f64; 2], [f64; 2], f64)) -> fmn::Result<[Mob; 2]> {
        let (at, size, font) = place;
        let frame = stage.arena_mut().add(
            Rectangle::new()
                .width(size[0])
                .height(size[1])
                .corner_radius(0.12)
                .style(
                    Style::default()
                        .fill(Palette::ink(), 0.0)
                        .stroke(Palette::dim(), 2.0, 1.0),
                )
                .build()?
                .moved_to(p2(at)),
        );
        Ok([frame, self.text(stage, QUESTION_CARD, font, Palette::ink(), at)?])
    }

    /// A small outlined tag ("schematic", "analytic …", "not enough alone").
    fn chip(&self, stage: &mut Stage<'_>, at: [f64; 2], words: &str, color: Srgb) -> fmn::Result<Vec<Mob>> {
        let width = text_width(words, 18.0) + 0.4;
        Ok(vec![
            rounded(stage, at, [width, 0.4], (card_fill(), 0.0), (color, 1.2))?,
            self.text(stage, words, 18.0, color, at)?,
        ])
    }

    /// A lower third with the exact form of a simplification, one text line per entry.
    fn precisely(&self, stage: &mut Stage<'_>, lines: &[&str]) -> fmn::Result<Vec<Mob>> {
        lines
            .iter()
            .enumerate()
            .map(|(i, words)| self.text(stage, words, 19.0, Palette::dim(), [0.0, PRECISELY_Y - 0.36 * i as f64]))
            .collect()
    }

    /// HOLD 14_h1: the card asks; the line ends plus a breath (`kit.hold`); a marked five-second
    /// countdown; the card leaves. The caller draws nothing of 14_7's answer before this returns.
    fn predict(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let (ink, dim) = (Palette::ink(), Palette::dim());
        let mut asked = vec![rounded(
            stage,
            PREDICT_AT,
            PREDICT_SIZE,
            (card_fill(), 1.0),
            (dim, 1.4),
        )?];
        for (i, words) in QUESTION.iter().enumerate() {
            asked.push(self.text(stage, words, 24.0, ink, [PREDICT_AT[0], 1.85 - 0.4 * i as f64])?);
        }
        for (i, words) in OPTIONS.iter().enumerate() {
            asked.push(self.text(stage, words, 21.0, dim, [PREDICT_AT[0], 0.95 - 0.36 * i as f64])?);
        }
        self.appear(stage, &asked, FADE)?;
        self.play(stage)?;
        self.k.hold(stage)?;
        self.k.mark(stage, "predict");
        for second in (1..=PREDICTION_SECONDS).rev() {
            let clock = self.text(
                stage,
                &second.to_string(),
                30.0,
                Palette::hero(),
                [PREDICT_AT[0], -0.15],
            )?;
            self.put(stage, &[clock])?;
            stage.wait(1.0)?;
            self.cut(stage, &[clock]);
        }
        self.vanish(stage, &asked, FADE)?;
        self.play(stage)
    }

    // ------------------------------------------------------------------------------- the row

    fn graph_dot(&self, stage: &mut Stage<'_>, x: Q) -> Mob {
        disc(stage, graph_point(x), 0.045, Palette::blue())
    }

    fn guess_dash(&self, stage: &mut Stage<'_>, cell: Cell) -> fmn::Result<Mob> {
        dashed(
            stage,
            graph_point(cell.outer),
            graph_point(cell.inner),
            Palette::dim(),
            1.6,
        )
    }

    fn middle_ring(&self, stage: &mut Stage<'_>, x: Q) -> Mob {
        ring(stage, graph_point(x), 0.065, Palette::ink(), 1.6)
    }

    /// Bars rise in `cells`; clipped ones get their break mark once they stand.
    fn rise(&mut self, stage: &mut Stage<'_>, row: &mut Row, cells: &[Cell]) -> fmn::Result<()> {
        let mut marks = Vec::new();
        for &cell in cells {
            let (flat, full, mark) = miss_bar(stage, cell)?;
            self.put(stage, &[flat])?;
            self.morph(stage, flat, full, RISE)?;
            let mut parts = vec![flat];
            parts.extend(&mark);
            marks.extend(mark);
            row.bars.push((cell, parts));
        }
        self.play(stage)?;
        self.appear(stage, &marks, BEAT)
    }

    /// Split `cells` in two: their bars, labels and straight lines leave, a divider is drawn at
    /// each middle, the exact middle value there becomes a known end, and the halves get lines.
    fn split(&mut self, stage: &mut Stage<'_>, row: &mut Row, cells: &[Cell]) -> fmn::Result<()> {
        for &cell in cells {
            let mut gone = take(&mut row.bars, cell).unwrap_or_default();
            gone.extend(take(&mut row.spins, cell));
            gone.extend(take(&mut row.dashes, cell));
            self.vanish(stage, &gone, SPLIT)?;
            let m = cell.middle();
            let x = row_x(m);
            let top = ROW_CENTER[1] + ROW_SCALE;
            let divider = line(stage, [x, top - 2.0 * ROW_SCALE], [x, top], Palette::ink(), 1.3)?;
            self.draw(stage, &[divider], SPLIT)?;
            row.dividers.push(divider);
            let dot = self.graph_dot(stage, m);
            match take(&mut row.rings, m) {
                Some(r) => {
                    self.morph(stage, r, dot, SPLIT)?;
                    row.dots.push((m, r));
                }
                None => {
                    self.appear(stage, &[dot], SPLIT)?;
                    row.dots.push((m, dot));
                }
            }
            for half in cell.halves() {
                let dash = self.guess_dash(stage, half)?;
                self.appear(stage, &[dash], SPLIT)?;
                row.dashes.push((half, dash));
            }
        }
        self.play(stage)
    }

    /// Back to the six coarse cells (14_10): everything the rounds added leaves, and the two
    /// middles that became ends are exact-value rings again.
    fn restore_coarse(&mut self, stage: &mut Stage<'_>, row: &mut Row) -> fmn::Result<()> {
        let coarse = adapt_row::coarse_row();
        let ends = adapt_row::row_points();
        let mut gone: Vec<Mob> = row.bars.drain(..).flat_map(|(_, parts)| parts).collect();
        gone.extend(row.spins.drain(..).map(|(_, m)| m));
        gone.append(&mut row.dividers);
        let (keep, extra): (Vec<_>, Vec<_>) = row.dashes.drain(..).partition(|(c, _)| coarse.contains(c));
        gone.extend(extra.into_iter().map(|(_, m)| m));
        row.dashes = keep;
        let (keep, extra): (Vec<_>, Vec<_>) = row
            .rings
            .drain(..)
            .partition(|(x, _)| coarse.iter().any(|c| c.middle() == *x));
        gone.extend(extra.into_iter().map(|(_, m)| m));
        row.rings = keep;
        let (keep, extra): (Vec<_>, Vec<_>) = row.dots.drain(..).partition(|(x, _)| ends.contains(x));
        row.dots = keep;
        for (x, dot) in extra {
            if coarse.iter().any(|c| c.middle() == x) {
                let back = self.middle_ring(stage, x);
                self.morph(stage, dot, back, MORPH)?;
                row.rings.push((x, dot));
            } else {
                gone.push(dot);
            }
        }
        self.vanish(stage, &gone, FADE)?;
        for cell in coarse {
            if !row.dashes.iter().any(|(c, _)| *c == cell) {
                let dash = self.guess_dash(stage, cell)?;
                self.appear(stage, &[dash], FADE)?;
                row.dashes.push((cell, dash));
            }
        }
        self.play(stage)
    }

    /// Everything on screen except `keep`.
    fn all_but(&self, keep: &[Mob]) -> Vec<Mob> {
        self.on.iter().copied().filter(|m| !keep.contains(m)).collect()
    }
}

impl SceneConstruct for Chapter14 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        // Both refusals come before the first frame.
        let tunnel = self.tunnel.ok_or_else(|| {
            fail("chapter 14 opens on the tunnel: render it with --tunnel and the retained startup snapshots".into())
        })?;
        // The render log carries the fact table, so each picture's numbers can be read against it.
        for fact in facts().map_err(fail)?.print().lines() {
            println!("Chapter 14 fact {fact}");
        }
        let l = labels();
        let row_cells = adapt_row::coarse_row();
        let [first, second] = adapt_row::rounds();
        let (ink, dim, blue, hero) = (Palette::ink(), Palette::dim(), Palette::blue(), Palette::hero());
        let mut s = Ch14 {
            k: &self.kit,
            on: Vec::new(),
            pending: Vec::new(),
        };

        // 14_1: back to the question card under the tunnel; one teaching-grid cell splits into four.
        s.say(stage, "14_1")?;
        let drawing = tunnel_drawing(stage, tunnel, TUNNEL_WIDTH, TUNNEL_CENTER)?;
        let question = s.question(stage, CARD_FULL)?;
        s.appear(stage, &drawing, FADE)?;
        s.appear(stage, &question, FADE)?;
        s.play(stage)?;
        s.pulse(stage, &question, None)?;
        s.until(stage, "14_1", 1)?;
        let (quarter, at) = grid_split(stage, tunnel, TUNNEL_WIDTH, TUNNEL_CENTER)?;
        s.draw(stage, &quarter, SPLIT)?;
        let times_four = s.text(stage, &l.four_cells, 30.0, hero, [at[0], 2.62])?;
        let steps = s.text(stage, STEPS_NOTE, 20.0, dim, [at[0], 3.02])?;
        s.appear(stage, &[times_four, steps], FADE)?;

        // 14_2: an adaptive grid splits and merges one schematic cell; multigrid is another idea.
        s.say(stage, "14_2")?;
        let mut leaving = drawing;
        leaving.extend(quarter);
        leaving.extend([times_four, steps]);
        s.vanish(stage, &leaving, FADE)?;
        let strip = s.question(stage, CARD_STRIP)?;
        s.morph(stage, question[0], strip[0], MORPH)?;
        s.morph(stage, question[1], strip[1], MORPH)?;
        let keep = [question[0], question[1]];
        let big = rect(stage, SCHEMATIC_AT, [SCHEMATIC_SIDE; 2], (ink, 0.0), (ink, 1.8))?;
        let schematic = s.chip(
            stage,
            [SCHEMATIC_AT[0], SCHEMATIC_AT[1] - 0.5 * SCHEMATIC_SIDE - 0.45],
            SCHEMATIC,
            dim,
        )?;
        s.appear(stage, &[big], FADE)?;
        s.appear(stage, &schematic, FADE)?;
        s.pause(stage, BEAT)?;
        let children = quarter_lines(stage, SCHEMATIC_AT, SCHEMATIC_SIDE, ink, 1.6)?;
        s.draw(stage, &children, SPLIT)?;
        s.pause(stage, BEAT)?;
        let corner = [
            SCHEMATIC_AT[0] - SCHEMATIC_SIDE / 4.0,
            SCHEMATIC_AT[1] - SCHEMATIC_SIDE / 4.0,
        ];
        let grandchildren = quarter_lines(stage, corner, SCHEMATIC_SIDE / 2.0, ink, 1.3)?;
        s.draw(stage, &grandchildren, SPLIT)?;
        s.pause(stage, 2.0 * BEAT)?;
        s.vanish(stage, &grandchildren, SPLIT)?;
        s.until(stage, "14_2", 1)?;
        let adaptive = s.text(stage, ADAPTIVE, 30.0, ink, [SCHEMATIC_AT[0], 2.2])?;
        let open = s.text(stage, OPEN_READOUT, 24.0, ink, [SIDE_X, 2.1])?;
        s.appear(stage, &[adaptive, open], FADE)?;
        s.until(stage, "14_2", 2)?;
        let mut stack = vec![rounded(
            stage,
            [SIDE_X, STACK_Y - 0.1],
            [6.0, 2.0],
            (card_fill(), 0.0),
            (dim, 1.2),
        )?];
        for (&x, &n) in STACK_X.iter().zip(&STACK_CELLS) {
            stack.extend(whole_grid(stage, [x, STACK_Y], STACK_SIDE, n)?);
        }
        stack.push(s.text(stage, MULTIGRID_NOTE, 22.0, dim, [SIDE_X, STACK_Y - 1.4])?);
        s.appear(stage, &stack, FADE)?;

        // 14_3: the natural first rule.
        s.say(stage, "14_3")?;
        let mut leaving = vec![big, adaptive, open];
        leaving.extend(&schematic);
        leaving.extend(&children);
        leaving.extend(&stack);
        s.vanish(stage, &leaving, FADE)?;
        s.until(stage, "14_3", 1)?;
        let rule = s.card(stage, RULE_BIG, RULE, Palette::gold())?;
        s.appear(stage, &rule, FADE)?;

        // 14_4: the ideal flow, exact, with no spin anywhere.
        s.say(stage, "14_4")?;
        let rule_small = s.card(stage, RULE_SMALL, RULE, Palette::gold())?;
        s.morph(stage, rule[0], rule_small[0], MORPH)?;
        s.morph(stage, rule[1], rule_small[1], MORPH)?;
        let keep = [keep[0], keep[1], rule[0], rule[1]];
        let body = cylinder(stage, FLOW_CENTER, FLOW_SCALE);
        let flow_axis = axis(stage, false)?;
        s.appear(stage, &[body], FADE)?;
        s.appear(stage, &flow_axis, FADE)?;
        s.play(stage)?;
        let streams = streamlines(stage, false);
        s.draw(stage, &streams, DRAW)?;
        let analytic = s.chip(stage, [FLOW_CENTER[0], -2.85], ANALYTIC_FLOW, dim)?;
        s.appear(stage, &analytic, FADE)?;
        s.until(stage, "14_4", 1)?;
        let mut speeds = Vec::new();
        for (words, sign) in l.shoulders.iter().zip([1.0, -1.0]) {
            speeds.push(s.text(stage, words, 24.0, ink, flow_at(0.0, sign * SHOULDER_LABEL))?);
        }
        s.appear(stage, &speeds, FADE)?;
        s.until(stage, "14_4", 2)?;
        let mut marks = Vec::new();
        for (&(x, y), words) in adapt_row::marked_points().iter().zip(&l.spin_points) {
            let at = flow_at(x.to_f64(), y.to_f64());
            marks.push(disc(stage, at, 0.05, ink));
            marks.push(s.text(stage, words, 17.0, ink, [at[0] + 0.42, at[1]])?);
        }
        s.appear(stage, &marks, FADE)?;
        let precisely = s.precisely(stage, &PRECISELY_14_4)?;
        s.appear(stage, &precisely, FADE)?;
        s.until(stage, "14_4", 3)?;
        // "Curving paths": the two streamlines closest over and under the cylinder curve the most.
        s.pulse(stage, &streams[..2], None)?;

        // 14_5: the flow rescales to the row (its streamlines then leave, clear of the speed graph);
        // six coarse cells line the centre line.
        s.say(stage, "14_5")?;
        let mut leaving = speeds;
        leaving.extend(&marks);
        leaving.extend(&precisely);
        s.vanish(stage, &leaving, FADE)?;
        s.play(stage)?;
        let body_row = cylinder(stage, ROW_CENTER, ROW_SCALE);
        s.morph(stage, body, body_row, MORPH)?;
        let streams_row = streamlines(stage, true);
        for (&from, &to) in streams.iter().zip(&streams_row) {
            s.morph(stage, from, to, MORPH)?;
        }
        let axis_row = axis(stage, true)?;
        for (&from, &to) in flow_axis.iter().zip(&axis_row) {
            s.morph(stage, from, to, MORPH)?;
        }
        let analytic_row = s.chip(stage, [FLOW_CENTER[0], -2.85], ANALYTIC_ROW, dim)?;
        s.play(stage)?;
        s.cut(stage, &analytic);
        s.put(stage, &analytic_row)?;
        s.vanish(stage, &streams, FADE)?;
        let mut row = Row::default();
        for cell in row_cells {
            let side = ROW_SCALE * cell.width().to_f64();
            row.cells.push(rect(
                stage,
                [row_x(cell.middle()), ROW_CENTER[1]],
                [side, side],
                (ink, 0.0),
                (ink, 1.6),
            )?);
        }
        s.appear(stage, &row.cells, FADE)?;
        let far = row_cells[5];
        let (a, b, y) = (row_x(far.outer), row_x(far.inner), ROW_CENTER[1] - ROW_SCALE - 0.12);
        let width_label = vec![
            line(stage, [a, y], [b, y], dim, 1.4)?,
            line(stage, [a, y], [a, y + 0.1], dim, 1.4)?,
            line(stage, [b, y], [b, y + 0.1], dim, 1.4)?,
            s.text(stage, CELL_WIDTH, 19.0, dim, [row_x(far.middle()), y - 0.22])?,
        ];
        s.appear(stage, &width_label, FADE)?;
        s.until(stage, "14_5", 1)?;
        for x in adapt_row::row_points() {
            let length = ARROW_PER_SPEED * adapt_row::speed(x).to_f64();
            let c = [row_x(x), ROW_CENTER[1]];
            let m = if length > 0.0 {
                arrow(
                    stage,
                    [c[0] - length / 2.0, c[1]],
                    [c[0] + length / 2.0, c[1]],
                    blue,
                    2.0,
                )?
            } else {
                disc(stage, c, 0.06, blue)
            };
            row.arrows.push(m);
        }
        s.appear(stage, &row.arrows, FADE)?;
        let unit = Q::ONE;
        let graph = vec![
            line(stage, [GRAPH_LEFT, GRAPH_ZERO], [GRAPH_RIGHT, GRAPH_ZERO], dim, 1.3)?,
            dashed(
                stage,
                [GRAPH_LEFT, graph_y(unit)],
                [GRAPH_RIGHT, graph_y(unit)],
                Palette::faint(),
                1.3,
            )?,
            s.text_left(stage, SPEED_AXIS, 20.0, dim, [GRAPH_LEFT - 0.3, graph_y(unit) + 0.3])?,
            s.text(stage, &unit.to_string(), 18.0, dim, [GRAPH_LEFT - 0.2, graph_y(unit)])?,
            s.text(stage, &Q::ZERO.to_string(), 18.0, dim, [GRAPH_LEFT - 0.2, GRAPH_ZERO])?,
        ];
        s.appear(stage, &graph, FADE)?;
        for x in adapt_row::row_points() {
            let dot = s.graph_dot(stage, x);
            row.dots.push((x, dot));
        }
        for cell in row_cells {
            let dash = s.guess_dash(stage, cell)?;
            row.dashes.push((cell, dash));
        }
        let shown: Vec<Mob> = row
            .dots
            .iter()
            .map(|d| d.1)
            .chain(row.dashes.iter().map(|d| d.1))
            .collect();
        s.appear(stage, &shown, FADE)?;
        s.until(stage, "14_5", 2)?;
        for cell in row_cells {
            let r = s.middle_ring(stage, cell.middle());
            row.rings.push((cell.middle(), r));
        }
        let rings: Vec<Mob> = row.rings.iter().map(|r| r.1).collect();
        s.appear(stage, &rings, FADE)?;
        let mut legend = vec![
            disc(stage, [1.25, LEGEND_Y[0]], 0.045, blue),
            dashed(stage, [1.0, LEGEND_Y[1]], [1.5, LEGEND_Y[1]], dim, 1.6)?,
            ring(stage, [1.25, LEGEND_Y[2]], 0.065, ink, 1.6),
        ];
        for (words, &y) in LEGEND.iter().zip(&LEGEND_Y) {
            legend.push(s.text_left(stage, words, 20.0, dim, [1.75, y])?);
        }
        s.appear(stage, &legend, FADE)?;
        let precisely = s.precisely(stage, &PRECISELY_14_5)?;
        s.appear(stage, &precisely, FADE)?;

        // 14_6: no spin in any cell; then the prediction.
        s.say(stage, "14_6")?;
        s.vanish(stage, &precisely, FADE)?;
        for (&cell, words) in row_cells.iter().zip(&l.spin_cells) {
            let m = s.text(stage, words, 17.0, ink, [row_x(cell.middle()), ROW_CENTER[1] + 0.27])?;
            row.spins.push((cell, m));
        }
        let spins: Vec<Mob> = row.spins.iter().map(|m| m.1).collect();
        s.appear(stage, &spins, FADE)?;
        s.until(stage, "14_6", 1)?;
        s.predict(stage)?;

        // 14_7: the answer. Magenta miss bars rise in all six cells.
        s.say(stage, "14_7")?;
        let mut bars_frame = vec![
            line(stage, [GRAPH_LEFT, BAR_BASE], [GRAPH_RIGHT, BAR_BASE], dim, 1.3)?,
            s.text_left(stage, MISS, 20.0, miss_color(), [-7.05, BAR_BASE + 0.15])?,
        ];
        s.appear(stage, &bars_frame, FADE)?;
        s.rise(stage, &mut row, &row_cells)?;
        s.until(stage, "14_7", 1)?;
        let third = row_cells[2];
        let (h3, _) = bar_height(third.miss());
        let third_readout = s.text(
            stage,
            &l.third,
            20.0,
            ink,
            [row_x(third.middle()), BAR_BASE + h3 + 0.22],
        )?;
        s.appear(stage, &[third_readout], FADE)?;
        if let Some((_, parts)) = row.bars.iter().find(|(c, _)| *c == third) {
            let parts = parts.clone();
            s.pulse(stage, &parts, None)?;
        }
        s.until(stage, "14_7", 2)?;
        let near = row_cells[0];
        let (hn, _) = bar_height(near.miss());
        let xn = row_x(near.middle());
        let near_readout = s.text(stage, &l.near, 20.0, ink, [xn, BAR_BASE + hn + 0.22])?;
        let gap = line(
            stage,
            [xn, graph_y(near.guess())],
            [xn, graph_y(near.exact())],
            miss_color(),
            2.4,
        )?;
        let guess = s.text(stage, &l.guess, 20.0, ink, [xn - 0.3, graph_y(near.guess()) - 0.16])?;
        let exact = s.text(stage, &l.exact, 20.0, ink, [xn + 0.3, graph_y(near.exact()) + 0.07])?;
        let answer = [near_readout, gap, guess, exact];
        s.appear(stage, &answer, FADE)?;
        s.until(stage, "14_7", 3)?;
        let worse = s.text(stage, &l.worse, 22.0, ink, [SIDE_X, 0.65])?;
        s.appear(stage, &[worse], FADE)?;

        // 14_8: the tolerance, 0.01, drawn in the cell test; the two nearest cells split.
        s.say(stage, "14_8")?;
        s.vanish(stage, &[worse], FADE)?;
        let tol_y = BAR_BASE + tolerance_height();
        let threshold = vec![
            dashed(stage, [GRAPH_LEFT, tol_y], [GRAPH_RIGHT, tol_y], miss_color(), 2.0)?,
            s.text_left(stage, &l.tolerance, 20.0, miss_color(), [GRAPH_RIGHT + 0.15, tol_y])?,
        ];
        s.appear(stage, &threshold, FADE)?;
        s.until(stage, "14_8", 1)?;
        let second_cell = row_cells[1];
        let (h2, _) = bar_height(second_cell.miss());
        let second_readout = s.text(
            stage,
            &l.second,
            20.0,
            ink,
            [row_x(second_cell.middle()), BAR_BASE + h2 + 0.22],
        )?;
        s.appear(stage, &[second_readout], FADE)?;
        let crossing: Vec<Mob> = row
            .bars
            .iter()
            .filter(|(c, _)| first.splits().contains(c))
            .flat_map(|(_, parts)| parts.clone())
            .collect();
        s.pulse(stage, &crossing, None)?;
        s.pause(stage, BEAT)?;
        s.vanish(stage, &answer, SPLIT)?;
        s.vanish(stage, &[second_readout, third_readout], SPLIT)?;
        s.split(stage, &mut row, &first.splits())?;
        s.until(stage, "14_8", 2)?;
        let cells_readout = s.text(stage, &l.cells_first, 24.0, ink, [SIDE_X, 1.65])?;
        s.appear(stage, &[cells_readout], FADE)?;
        s.until(stage, "14_8", 3)?;
        let spin_readout = s.text(stage, &l.spin_rule, 24.0, dim, [SIDE_X, 1.15])?;
        s.appear(stage, &[spin_readout], FADE)?;

        // 14_9: the test runs again on the new cells; the two at the wall split again.
        s.say(stage, "14_9")?;
        let made = first.made();
        let mut new_rings = Vec::new();
        for cell in &made {
            let r = s.middle_ring(stage, cell.middle());
            row.rings.push((cell.middle(), r));
            new_rings.push(r);
        }
        s.appear(stage, &new_rings, FADE)?;
        s.rise(stage, &mut row, &made)?;
        s.until(stage, "14_9", 1)?;
        let again: Vec<Mob> = row
            .bars
            .iter()
            .filter(|(c, _)| second.splits().contains(c))
            .flat_map(|(_, parts)| parts.clone())
            .collect();
        s.pulse(stage, &again, None)?;
        s.pause(stage, pacing::SECOND_SPLIT_HOLD)?;
        s.split(stage, &mut row, &second.splits())?;
        // Show the measured errors that remain on the finest cells without moving the cue gates.
        for cell in second.made() {
            let ring = s.middle_ring(stage, cell.middle());
            s.put(stage, &[ring])?;
            row.rings.push((cell.middle(), ring));
            let (_, bar, mut marks) = miss_bar(stage, cell)?;
            marks.insert(0, bar);
            s.put(stage, &marks)?;
            row.bars.push((cell, marks));
        }
        let cells_again = s.text(stage, &l.cells_second, 24.0, ink, [SIDE_X, 1.65])?;
        s.morph(stage, cells_readout, cells_again, MORPH)?;
        s.until(stage, "14_9", 2)?;
        let above = second.after.iter().filter(|c| c.miss() > adapt_row::tolerance()).count();
        let limit = format!("At the size limit, {above} cells still fail");
        let smallest = s.text(stage, &limit, 22.0, miss_color(), [SIDE_X, 0.65])?;
        s.appear(stage, &[smallest], FADE)?;

        // 14_10: what the miss follows. Strain near and far, a uniform strain with no miss, and the
        // slope that keeps changing.
        s.say(stage, "14_10")?;
        let mut leaving = vec![cells_readout, spin_readout, smallest];
        leaving.append(&mut bars_frame);
        leaving.extend(&threshold);
        leaving.extend(&legend);
        leaving.extend(&width_label);
        s.vanish(stage, &leaving, FADE)?;
        s.restore_coarse(stage, &mut row)?;
        s.until(stage, "14_10", 1)?;
        let mut squares = Vec::new();
        let mut squeezed = Vec::new();
        let mut strain_words = Vec::new();
        for (cell, words) in [(row_cells[0], &l.strain_near), (row_cells[2], &l.strain_third)] {
            let c = [row_x(cell.middle()), ROW_CENTER[1]];
            let rate = adapt_row::strain(cell.middle()).abs().to_f64();
            let gold = Palette::gold();
            squares.push(rect(stage, c, [SQUARE, SQUARE], (gold, 0.18), (gold, 2.0))?);
            let along = SQUARE * (-rate * FLOW_TIME).exp();
            let across = SQUARE * (rate * FLOW_TIME).exp();
            squeezed.push(rect(stage, c, [along, across], (gold, 0.18), (gold, 2.0))?);
            strain_words.push(s.text(stage, words, 20.0, ink, [c[0], ROW_CENTER[1] - ROW_SCALE - 0.3])?);
        }
        s.appear(stage, &squares, FADE)?;
        s.appear(stage, &strain_words, FADE)?;
        s.play(stage)?;
        for (&from, &to) in squares.iter().zip(&squeezed) {
            s.morph(stage, from, to, SQUEEZE)?;
        }
        s.until(stage, "14_10", 2)?;
        let mut inset = vec![
            rounded(stage, INSET_AT, INSET_SIZE, (card_fill(), 1.0), (dim, 1.2))?,
            s.text(stage, UNIFORM_TITLE, 20.0, ink, [INSET_AT[0], INSET_AT[1] + 1.55])?,
        ];
        // Straight arrows on a grid: (u, v) = (−x, y) at x, y ∈ {−1, −½, 0, ½, 1}, inset units.
        let field = [INSET_AT[0] - 1.0, INSET_AT[1] + 0.55];
        for i in -2..=2 {
            for j in -1..=1 {
                let (x, y) = (Q::new(i, 2), Q::new(j, 2));
                let (u, w) = adapt_row::uniform_velocity(x, y);
                let c = [field[0] + 0.55 * x.to_f64() * 2.0, field[1] + 0.42 * y.to_f64() * 2.0];
                let (du, dw) = (0.32 * u.to_f64(), 0.32 * w.to_f64());
                inset.push(if u.is_zero() && w.is_zero() {
                    disc(stage, c, 0.04, blue)
                } else {
                    arrow(
                        stage,
                        [c[0] - du / 2.0, c[1] - dw / 2.0],
                        [c[0] + du / 2.0, c[1] + dw / 2.0],
                        blue,
                        1.6,
                    )?
                });
            }
        }
        // Its speed graph along y = 0, u = −x over x ∈ [−1, 1], with three straight-line cells on it.
        let graph_at = |x: Q| -> [f64; 2] {
            let u = adapt_row::uniform_velocity(x, Q::ZERO).0;
            [field[0] + 1.1 * x.to_f64(), INSET_AT[1] - 0.95 + 0.45 * u.to_f64()]
        };
        inset.push(line(stage, graph_at(-Q::ONE), graph_at(Q::ONE), blue, 2.0)?);
        let thirds = [-Q::ONE, Q::new(-1, 3), Q::new(1, 3), Q::ONE];
        for w in thirds.windows(2) {
            let cell = Cell {
                outer: w[0],
                inner: w[1],
            };
            inset.push(dashed(stage, graph_at(cell.outer), graph_at(cell.inner), ink, 1.6)?);
            inset.push(ring(stage, graph_at(cell.middle()), 0.06, ink, 1.6));
        }
        for (words, y) in [
            (&l.uniform_spin, 0.95),
            (&l.uniform_strain, 0.45),
            (&l.uniform_miss, -0.95),
        ] {
            inset.push(s.text(stage, words, 22.0, ink, [INSET_AT[0] + 1.95, INSET_AT[1] + y])?);
        }
        s.appear(stage, &inset, FADE)?;
        s.until(stage, "14_10", 3)?;
        let samples: Vec<[f64; 2]> = (0..=48)
            .map(|i| {
                let x = -3.0 + 2.0 * f64::from(i) / 48.0;
                [
                    ROW_CENTER[0] + ROW_SCALE * x,
                    GRAPH_ZERO + GRAPH_HEIGHT * adapt_row::speed_f64(x),
                ]
            })
            .collect();
        let bend = curve(stage, &samples, blue, 2.4, 1.0);
        s.draw(stage, &[bend], MORPH)?;
        let slope = s.text(
            stage,
            SLOPE,
            20.0,
            ink,
            [row_x(near.middle()) - 0.6, graph_y(Q::ONE) + 0.32],
        )?;
        s.appear(stage, &[slope], FADE)?;
        let precisely = s.precisely(stage, &PRECISELY_14_10)?;
        s.appear(stage, &precisely, FADE)?;

        // 14_11: what real flows add; spin is a clue, not a measure of error.
        s.say(stage, "14_11")?;
        let mut leaving = squares;
        leaving.extend(&strain_words);
        leaving.extend(&inset);
        leaving.extend([bend, slope]);
        leaving.extend(&precisely);
        s.vanish(stage, &leaving, FADE / 2.0)?;
        s.play(stage)?;
        let mut cards = Vec::new();
        for (j, words) in CARDS_14_11.iter().enumerate() {
            if j > 0 {
                s.until(stage, "14_11", j)?;
            }
            let [frame, text] = s.card(stage, ([SIDE_X, 1.5 - 0.7 * j as f64], [4.8, 0.55], 22.0), words, dim)?;
            let fade = if j == 0 { FADE / 2.0 } else { FADE };
            s.appear(stage, &[frame, text], fade)?;
            cards.extend([frame, text]);
            if j == 0 {
                let precisely = s.precisely(stage, &PRECISELY_14_11)?;
                s.appear(stage, &precisely, fade)?;
                cards.extend(precisely);
            }
        }
        s.until(stage, "14_11", 3)?;
        let tag = s.chip(stage, [SIDE_X, 2.2], NOT_ENOUGH, dim)?;
        s.appear(stage, &tag, FADE)?;

        // 14_12: what Basilisk tests, as quoted from its source.
        s.say(stage, "14_12")?;
        let leaving = s.all_but(&keep[..2]);
        s.vanish(stage, &leaving, FADE)?;
        let source_card = [-0.3, 0.45];
        let mut basilisk = vec![
            rounded(stage, source_card, [12.8, 3.6], (card_fill(), 1.0), (dim, 1.2))?,
            s.text(stage, CARD_14_12[0], 30.0, ink, [source_card[0], 1.8])?,
        ];
        s.appear(stage, &basilisk, FADE)?;
        for (j, words) in CARD_14_12.iter().enumerate().skip(1) {
            s.until(stage, "14_12", j)?;
            let part = s.text(stage, words, 28.0, ink, [source_card[0], 1.8 - 0.62 * j as f64])?;
            s.appear(stage, &[part], FADE)?;
            basilisk.push(part);
        }
        s.until(stage, "14_12", 3)?;
        let pins = format!(
            "Basilisk source, pinned tarball: grid/tree-common.h lines {} to {}, grid/multigrid-common.h line {}",
            pin_line(BASILISK[0].1),
            pin_line(BASILISK[4].1),
            pin_line(BASILISK[5].1)
        );
        let example = vec![
            s.text(stage, KARMAN_NOTE, 18.0, dim, [source_card[0], -0.1])?,
            s.mono(stage, KARMAN_CALL, 22.0, ink, [source_card[0], -0.55])?,
            s.text(stage, &pins, 16.0, dim, [source_card[0], -1.05])?,
        ];
        s.appear(stage, &example, FADE)?;
        basilisk.extend(example);
        let precisely = s.precisely(stage, &PRECISELY_14_12)?;
        s.appear(stage, &precisely, FADE)?;
        basilisk.extend(precisely);

        // 14_13: every split leaves a seam (chapter 15's big cell, in chapter 15's place).
        s.say(stage, "14_13")?;
        s.vanish(stage, &basilisk, FADE)?;
        let [x0, x1, y0, y1] = SEAM_BIG;
        let side = x1 - x0;
        let mut seam = vec![rect(
            stage,
            [(x0 + x1) / 2.0, (y0 + y1) / 2.0],
            [side, y1 - y0],
            (ink, 0.0),
            (ink, 1.8),
        )?];
        let neighbour = [x1 + side / 2.0, (y0 + y1) / 2.0];
        seam.push(rect(stage, neighbour, [side, side], (ink, 0.0), (ink, 1.8))?);
        seam.extend(quarter_lines(stage, neighbour, side, ink, 1.6)?);
        seam.extend(s.chip(stage, [x1, y0 - 0.55], SCHEMATIC, dim)?);
        s.appear(stage, &seam, FADE)?;
        let edge = line(stage, [x1, y0], [x1, y1], glow(), 4.5)?;
        let named = s.text(stage, SEAM, 24.0, glow(), [x1, y1 + 0.35])?;
        s.appear(stage, &[edge, named], FADE)?;
        s.play(stage)?;
        s.pulse(stage, &[edge], Some([1.0, 1.0, 1.0]))?;
        s.play(stage)?;
        println!("Chapter 14 ends on chapter 15's seam: the big cell, the split cell beside it and the glowing edge");
        s.k.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cues_run_in_order() {
        let ids: Vec<&str> = CUES.iter().map(|&(id, _)| id).collect();
        let expected: Vec<String> = (1..=13).map(|k| format!("14_{k}")).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        assert!(CUES.iter().all(|&(_, sentences)| sentences > 0));
    }

    #[test]
    fn labels_match_the_storyboard() {
        let l = labels();
        assert_eq!(l.four_cells, "× 4 cells");
        assert_eq!(l.shoulders, ["2 ×", "2 ×"]);
        assert!(l.spin_points.len() == 6 && l.spin_points.iter().all(|w| w == "spin 0"));
        assert!(l.spin_cells.len() == 6 && l.spin_cells.iter().all(|w| w == "spin 0"));
        assert_eq!(l.third, "0.002");
        assert_eq!(l.near, "0.31");
        assert_eq!(l.guess, "4/9");
        assert_eq!(l.exact, "3/4");
        assert_eq!(l.worse, "about 126 × worse near the cylinder");
        assert_eq!(l.tolerance, "tolerance 0.01");
        assert_eq!(l.second, "0.013");
        assert_eq!(l.cells_first, "cells 6 → 8 (halve all: 12)");
        assert_eq!(l.spin_rule, "spin rule: 6");
        assert_eq!(l.cells_second, "cells 10 (halve all twice: 24)");
        assert_eq!(l.strain_near, "strain 1/4");
        assert_eq!(l.strain_third, "strain 1/108");
        assert_eq!(l.uniform_spin, "spin 0");
        assert_eq!(l.uniform_strain, "strain 1");
        assert_eq!(l.uniform_miss, "miss 0");
    }

    #[test]
    fn facts_pin_the_oracles() {
        let f = facts().expect("every pin holds");
        assert_eq!(f.get("row.cells.after"), Some("8"));
        assert_eq!(f.get("row2.cells.after"), Some("10"));
        assert_eq!(f.get("row.miss.near.shown"), Some("0.31"));
        assert_eq!(
            f.get("bas.adapt.refine"),
            Some("basilisk/src/grid/tree-common.h:247 if (e > emax && level < maxlevel) {")
        );
        // 54 pinned or added numbers and the seven Basilisk lines.
        assert_eq!(f.len(), 61);
    }

    #[test]
    fn the_tolerance_line_separates_the_bars_that_split() {
        let tol = tolerance_height();
        assert!((tol - 0.5).abs() < 1e-12);
        let [first, second] = adapt_row::rounds();
        for round in [&first, &second] {
            for (cell, &split) in round.cells.iter().zip(&round.split) {
                let (h, clipped) = bar_height(cell.miss());
                assert_eq!(h > tol, split, "{cell:?}");
                assert_eq!(clipped, BAR_SCALE * cell.miss().to_f64() > BAR_CLIP);
            }
        }
        // The bars that split stand clear of the line; the rest stay clearly below it.
        let row = adapt_row::coarse_row();
        assert!(bar_height(row[1].miss()).0 - tol > 0.15);
        assert!(tol - bar_height(row[2].miss()).0 > 0.35);
        // The clipped bar's readout clears the cells above it (checked when the tests compile).
        const { assert!(BAR_BASE + BAR_CLIP + 0.22 + 0.12 < ROW_CENTER[1] - ROW_SCALE) };
    }

    #[test]
    fn the_row_and_the_flow_fit_their_windows() {
        let ends = adapt_row::row_points();
        let (left, right) = (row_x(ends[6]), row_x(ends[0]));
        assert!(left >= -7.1 && right <= 0.2);
        assert!(
            (row_x(ends[0]) - (ROW_CENTER[0] - ROW_SCALE)).abs() < 1e-12,
            "the row ends at the cylinder's front"
        );
        for c in STREAMS {
            for &[x, y] in &adapt_row::streamline(c, FLOW_REACH, 241) {
                let at = flow_at(x, y);
                assert!((-7.1..=0.2).contains(&at[0]) && (-2.6..=2.8).contains(&at[1]));
            }
        }
        for (x, y) in adapt_row::marked_points() {
            let at = flow_at(x.to_f64(), y.to_f64());
            assert!((-7.1..=0.2).contains(&at[0]) && (-2.6..=2.8).contains(&at[1]));
            assert!(at[0] + 0.42 + text_width("spin 0", 17.0) / 2.0 <= 0.2);
        }
        // The arrows on the coarse row do not touch: each is shorter than a cell.
        const { assert!(ARROW_PER_SPEED < 2.0 * ROW_SCALE) };
        // Side-panel text fits the panel.
        let l = labels();
        for words in [
            l.cells_first.as_str(),
            &l.cells_second,
            &l.worse,
            MULTIGRID_NOTE,
            OPEN_READOUT,
        ] {
            assert!(text_width(words, 24.0) < 6.4, "{words}");
        }
        for words in PRECISELY_14_4
            .iter()
            .chain(&PRECISELY_14_10)
            .chain(&PRECISELY_14_11)
            .chain(&PRECISELY_14_12)
        {
            assert!(text_width(words, 19.0) < 14.2, "{words}");
        }
    }

    #[test]
    fn the_split_cell_lies_behind_the_cylinder() {
        let benchmark = TunnelShape {
            length: 2.2,
            height: 0.41,
            cylinder: [0.2, 0.2, 0.05],
        };
        let cell = benchmark.height / 3.0;
        let middle = benchmark.length / 2.0;
        let left = middle + ((0.25 - middle) / cell).ceil() * cell;
        assert!(left >= 0.25 && left - cell < 0.25);
        assert!(left + cell <= benchmark.length);
    }

    #[test]
    fn quotes_agree_with_the_basilisk_lines() {
        let karman = BASILISK.iter().find(|b| b.0 == "bas.karman.adapt").map(|b| b.2);
        assert_eq!(karman, Some(format!("{KARMAN_CALL};").as_str()));
        let coarsen = BASILISK
            .iter()
            .find(|b| b.0 == "bas.adapt.coarsen")
            .map(|b| b.2)
            .unwrap_or("");
        assert!(coarsen.contains("e <= emax/1.5") && CARD_14_12[2].ends_with("e ≤ emax/1.5"));
        let refine = BASILISK
            .iter()
            .find(|b| b.0 == "bas.adapt.refine")
            .map(|b| b.2)
            .unwrap_or("");
        assert!(refine.contains("e > emax") && CARD_14_12[1].contains("e > emax"));
        assert_eq!(pin_line(BASILISK[0].1), "173");
        assert_eq!(KARMAN_NOTE.rsplit(' ').next(), Some(pin_line(BASILISK[6].1)));
        assert_eq!(
            QUESTION.join(" "),
            "No spin anywhere: is a coarse cell equally good everywhere along the line?"
        );
        assert_eq!(
            OPTIONS.join("; "),
            "yes; no, worse near the cylinder; no, worse far out"
        );
    }
}
