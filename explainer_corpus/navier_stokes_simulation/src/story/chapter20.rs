//! Chapter 20, "Back to the wake" (navier-full-script03.md cues 20_1 to 20_9 and the hold 20_h1;
//! BronzeJaguar's closing source navier-closing-scene02-fs03.md, lines 255 to 292).
//!
//! The chapter returns to the opening's tunnel and question card at the opening's scale, names the
//! standard test's exact shape (DFG 2D-2), and lays out the measurement plan as a table: three
//! grids, each with cells half as wide, at our smallest step (the spatial rungs), then the finest
//! grid at two longer steps (the timestep ladder). Our entries come only from a `wake-ledger01`
//! file (`wake_ledger`), which takes numbers only from compatible, terminal, status-OK runs;
//! every other entry is drawn as "?", and nothing absent is drawn as zero. The grey reference
//! column holds FEATFLOW 2014's published values (another program's calculations, not
//! measurements and not our results), recomputed by `tools/twin20_01.py` from the retained
//! archive. The prediction 20_h1 asks how many swings per second St ≈ 0.3 gives; 20_7 answers
//! 3.02 per second and ticks a metronome at that rate: a clock, not a flow.
//!
//! No wake is drawn. The tunnel stays a static outline with its teaching grid, as in the opening;
//! the only arrows are the prescribed ST96 inflow profile, a boundary condition. A measured wake
//! field is W2's, a release prerequisite that this chapter does not draw (the ledger keeps W2
//! pending). The slot line 20_v1 is not voiced; the render log says whether a compare filled it.
//!
//! Voice: the kit narrator. Each cue is `say`d before its pictures, which wait on its sentence
//! starts (`until`). The prediction 20_h1, after 20_6, holds a five-second countdown while the
//! answer ("3.02") and the metronome are not yet drawn.

use explainer_kit::{Facts, Kit, Palette, Timed, hex, v};
use fmn::animation::indicate;
use fmn::prelude::*;

use crate::wake_ledger::{self, Compare, Ledger, Run};

/// Chapter 20's cues in spoken order and the number of sentences in each (the narrator's gates).
/// Sentences end at `.`, `?` or `!` before a space, as in the kit; the check recounts them.
pub const CUES: [(&str, usize); 9] = [
    ("20_1", 3),
    ("20_2", 4),
    ("20_3", 3),
    ("20_4", 4),
    ("20_5", 3),
    ("20_6", 3),
    ("20_7", 3),
    ("20_8", 3),
    ("20_9", 3),
];

/// Script HOLD 20_h1: the prediction countdown, in whole seconds.
pub const PREDICTION_SECONDS: u32 = 5;
/// 20_h1's question and answers, as the closing's holds table has them.
const QUESTION: &str = "St ≈ 0.3, Ū = 1 m/s, D = 0.1 m: swings per second?";
const OPTIONS: [&str; 3] = ["about 0.3", "about 3", "about 30"];
/// The givens of 20_h1, as numbers: St ≈ 0.3, Ū = 1 m/s, D = 0.1 m.
const HOLD_STROUHAL: f64 = 0.3;
/// 17_11's formula card.
const FORMULA: &str = "St = D f / Ū";
const FORMULA_NOTE: &str = "shown in ch. 17";
const CLOCK_NOTE: &str = "a clock, not a flow";

/// The opening's question card, and the closing's checklist under it (20_9).
const QUESTION_CARD: &str = "Does halving the cells change the answer?";
const CHECKLIST: [&str; 5] = [
    "halve the cells",
    "halve the steps",
    "check walls, inlet, outlet",
    "compare with published answers to the same test",
    "write it down",
];
const TEST_LABEL: &str = "standard test 2D-2, Re 100";
const STREAKLINE: &str = "dye: history (streakline, ch. 2)";

/// The table: run columns (the `wake_ledger` roles), their planned rungs (h the coarsest cell
/// width, k our smallest step), a legend, the row labels in `wake_ledger::QUANTITIES` order, and
/// the definitions of "middle" and "swing size".
const COLUMNS: [&str; 5] = ["grid 1", "grid 2", "grid 3", "step 2", "step 3"];
const RUNGS: [&str; 5] = ["h, k", "h/2, k", "h/4, k", "h/4, longer", "h/4, longest"];
const LEGEND: &str = "h: cell width, k: smallest step";
const ROW_LABELS: [&str; 14] = [
    "Strouhal number",
    "drag: peak",
    "drag: middle of the swing",
    "drag: swing size",
    "lift: peak",
    "lift: middle of the swing",
    "lift: swing size",
    "drag curve vs reference",
    "lift curve vs reference",
    "ΔP, front − back",
    "fluid cells",
    "time steps",
    "time on the clock",
    "memory",
];
const FOOTNOTE: &str =
    "middle of the swing: (max + min)/2, FEATFLOW's 'mean', not a time average; swing size: max − min";
/// What a pending entry shows.
const PENDING: &str = "?";
const CLOSE_PENDING: &str = "close enough: ?";
const CLOSE_DECLARED: &str = "close enough";
const REFERENCE_TITLE: [&str; 2] = ["reference", "FEATFLOW 2014"];
const REFERENCE_LABEL: &str =
    "FEATFLOW 2014: another program's reference calculations; not measurements, not our results";
/// The reference column's rows (indices into `wake_ledger::QUANTITIES`): St, drag peak, lift peak, ΔP.
const REFERENCE_ROWS: [usize; 4] = [0, 1, 4, 9];

/// Chapter 16's table (16_7), every entry "?" (FUTURE-SIM A2).
const A2_TITLE: &str = "chapter 16's cylinder comparison: uniform against adaptive, at the declared accuracy";
const A2_COLUMNS: [&str; 3] = ["coarse", "fine", "adaptive"];
const A2_ROWS: [&str; 7] = [
    "error in the measured answer",
    "cells, fluid and total",
    "time steps",
    "time on the clock",
    "memory",
    "the pressure puzzle's share",
    "imbalance kick at each change of grid",
];

/// 20_5's number line.
const LIFT_AXIS: &str = "lift peak";
const BAND_LABEL: &str = "1996: estimated interval";
const DOT_LABEL: &str = "2014: later reference data";

/// The Precisely lower thirds, each the full script's text split into lines. 20_2's runs over four
/// pages (20_2 sentences 1 and 3, then 20_3 sentences 0 and 1). On screen, the page text's
/// `$t_0$` and `$C_L$` are written t₀ and C_L. Two places differ from the script's wording: page A
/// names Schaefer and Turek (1996) before the first ST96, and 20_4 points to chapter 17, not its cue.
const PRECISELY_20_2A: [&str; 2] = [
    "As §2.4 of Schaefer and Turek (1996, hereafter ST96) asks: 'three successively coarsened \
     meshes … with a finest time",
    "discretization', and 'two successively coarsened time discretizations … together with the \
     finest mesh'.",
];
const PRECISELY_20_2B: [&str; 4] = [
    "ST96 concluding remark 2: 'by chance, \"maximum values\" may be obtained with good accuracy \
     even without capturing",
    "the general pattern of the flow at all'. So each force is also compared over one swing, \
     as FEATFLOW lists it: the middle",
    "of the swing, (max + min)/2, which FEATFLOW calls the mean ('mean=(max+min)/2'), \
     not a time average, which would be",
    "reported in a column of its own; and the swing size, max − min ('amp=max-min'), \
     the whole swing, not half of it.",
];
const PRECISELY_20_2C: [&str; 4] = [
    "The curves' shapes are compared with every cycle starting at a lift minimum, \
     as FEATFLOW's do ('A cycle starts at a",
    "time instant t₀ where the lift coefficient C_L is smallest'; its curves 'are shifted in time \
     such that t₀ matched'), so",
    "phase means only when the drag peaks relative to the lift. Time stays physical, \
     so each run keeps its own period, and",
    "ΔP keeps ST96's own timing, half a period after the lift's peak.",
];
const PRECISELY_20_2D: [&str; 3] = [
    "Shapes matched on a time axis divided by each run's own period would say nothing \
     about the frequency, which the Strouhal",
    "number checks. Storage alone, at 40 cells per diameter (880 × 164 cells, four 32-bit fields), \
     is 2.31 MB per saved state,",
    "about 8.3 GB per minute at 60 states per second (planning arithmetic).",
];
const PRECISELY_20_4: [&str; 3] = [
    "FEATFLOW 2014 reference data (Q2/P1disc, level 6), one cycle in t = 9–10 s. \
     Peak times interpolated. ΔP ≈ 2.485 at t0 + T/2",
    "with t0 at max cL (ST96). These values belong to this confined channel. \
     Agreeing with them checks our setup and program,",
    "not our equations (ch. 17).",
];
const PRECISELY_20_5: [&str; 2] = [
    "ST96 Table 4's last rows, labelled 'lower bound' and 'upper bound', \
     are 'estimated intervals for the \"exact\" results'",
    "suggested by the authors. FEATFLOW's interpolated peak, 0.98658, \
     lies below the earlier lower bound of 0.99.",
];
const PRECISELY_20_7: [&str; 2] = [
    "0.3018 × 1 m/s / 0.1 m = 3.018 per second; 1/f = 0.3313 s (FEATFLOW). \
     This belongs to the confined channel",
    "at Re 100. A cylinder in open air swings at a different rate, which this film does not give.",
];

// ------------------------------------------------------------------------------------ numbers

/// FEATFLOW 2014's 2D-2 reference (Q2/P1disc, level 6, k = 1/400, one cycle in t = 9 to 10 s), as
/// root's verified analysis `wake-analyzer-root01/verified02/reference.json` gives them (its
/// `table_row` and `analysis.measurements.period`). tools/twin20_01.py reads the retained archive's
/// two log files, checks them against that analysis, and recomputes these values independently.
pub const REF_STROUHAL: f64 = 0.301_841_005_749_285_35;
pub const REF_DRAG_PEAK: f64 = 3.227_392_727_692_691_5;
pub const REF_LIFT_PEAK: f64 = 0.986_582_983_304_548_4;
pub const REF_DELTA_P: f64 = 2.485_198_446_314_071_5;
pub const REF_PERIOD: f64 = 0.331_300_247_796_887_6;

/// ST96 Table 4 (2D-2), its last two rows, "lower bound" and "upper bound": cDmax, cLmax, St, ΔP.
pub const ST96_LOWER: [f64; 4] = [3.22, 0.99, 0.295, 2.46];
pub const ST96_UPPER: [f64; 4] = [3.24, 1.01, 0.305, 2.5];

/// ST96's 2D-2 case: channel, cylinder, peak inflow speed Um, kinematic viscosity, diameter.
pub const BENCH: TunnelShape = TunnelShape {
    length: 2.2,
    height: 0.41,
    cylinder: [0.2, 0.2, 0.05],
};
const PEAK_INFLOW: f64 = 1.5;
const VISCOSITY: f64 = 1e-3;
const DIAMETER: f64 = 0.1;
/// The planning arithmetic of 20_2's Precisely, in millimetres and whole cells: 40 cells per
/// diameter, four 32-bit fields per cell, 60 saved states per second.
const PLAN_CELLS_PER_DIAMETER: u64 = 40;
const PLAN_MM: [u64; 3] = [2200, 410, 100];
const PLAN_FIELDS: u64 = 4;
const PLAN_STATES_PER_SECOND: u64 = 60;

/// The inflow U(0, y) = 4 Um y (H − y) / H² (ST96), in m/s.
fn inflow(y: f64) -> f64 {
    4.0 * PEAK_INFLOW * y * (BENCH.height - y) / (BENCH.height * BENCH.height)
}

/// The mean inflow speed Ū: Simpson's rule over the channel height, exact for the parabola.
fn mean_inflow() -> f64 {
    let h = BENCH.height;
    (inflow(0.0) + 4.0 * inflow(h / 2.0) + inflow(h)) / 6.0
}

/// f = St Ū / D, swings per second.
fn swings_per_second(strouhal: f64) -> f64 {
    strouhal * mean_inflow() / DIAMETER
}

/// Bytes per saved state at the plan's resolution, and its cell counts.
fn plan_storage() -> (u64, [u64; 2]) {
    let [length, height, diameter] = PLAN_MM;
    let cells = [
        length * PLAN_CELLS_PER_DIAMETER / diameter,
        height * PLAN_CELLS_PER_DIAMETER / diameter,
    ];
    (cells[0] * cells[1] * PLAN_FIELDS * 4, cells)
}

/// A value as Python's `%g` prints it (six significant digits), the oracle file's format.
pub fn g6(x: f64) -> String {
    if x == 0.0 {
        return "0".to_owned();
    }
    let trim = |s: &str| -> String {
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_owned()
        } else {
            s.to_owned()
        }
    };
    let sci = format!("{x:.5e}");
    let Some((mantissa, exponent)) = sci.split_once('e') else {
        return sci;
    };
    let Ok(exponent) = exponent.parse::<i32>() else {
        return sci;
    };
    if (-4..6).contains(&exponent) {
        let places = (5 - exponent) as usize;
        trim(&format!("{x:.places$}"))
    } else {
        let sign = if exponent < 0 { '-' } else { '+' };
        format!("{}e{sign}{:02}", trim(mantissa), exponent.abs())
    }
}

fn py_bool(b: bool) -> &'static str {
    if b { "True" } else { "False" }
}

/// Every number-bearing label the chapter draws, computed from the constants above.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    pub ref_strouhal: String,
    pub ref_drag: String,
    pub ref_lift: String,
    pub ref_delta_p: String,
    pub interval: [String; 2],
    pub ticks: [String; 5],
    pub dot: String,
    pub answer: String,
    pub every: String,
    pub length: String,
    pub height: String,
    pub cylinder: String,
    pub inflow: String,
}

pub fn labels() -> Labels {
    let d = wake_ledger::decimals;
    let f = swings_per_second(REF_STROUHAL);
    let [cx, cy, radius] = BENCH.cylinder;
    Labels {
        ref_strouhal: d(REF_STROUHAL, 3),
        ref_drag: d(REF_DRAG_PEAK, 2),
        ref_lift: d(REF_LIFT_PEAK, 3),
        ref_delta_p: d(REF_DELTA_P, 3),
        interval: [d(ST96_LOWER[1], 2), d(ST96_UPPER[1], 2)],
        ticks: AXIS_TICKS.map(|t| d(t, 2)),
        dot: d(REF_LIFT_PEAK, 3),
        answer: format!("f = St × Ū / D ≈ {} per second", d(f, 2)),
        every: format!("one swing every {} s", d(1.0 / f, 2)),
        length: format!("{} m", g6(BENCH.length)),
        height: format!("{} m", g6(BENCH.height)),
        cylinder: format!("D = {} m at ({} m, {} m)", g6(2.0 * radius), g6(cx), g6(cy)),
        inflow: format!(
            "prescribed inflow: peak {} m/s, average {} m/s",
            g6(PEAK_INFLOW),
            g6(mean_inflow())
        ),
    }
}

/// The chapter's fact table (the render log prints it).
pub fn facts() -> Result<Facts, String> {
    let mut facts = Facts::new();
    register(&mut facts)?;
    facts.self_check()?;
    Ok(facts)
}

/// Add chapter 20's numbers to a film's registry. The keys the closing's oracle prints
/// (navier-closing-scene02/runs/oracle-keys03.txt) are pinned to its printed values; the `c20.`
/// keys pin what is drawn to the cue that says it.
pub fn register(facts: &mut Facts) -> Result<(), String> {
    let l = labels();
    let (bytes, cells) = plan_storage();
    let per_minute = bytes as f64 * (PLAN_STATES_PER_SECOND * 60) as f64 / 1e9;
    let oracle = "navier-closing-scene02 runs/oracle-keys03.txt";
    let in_interval = |k: usize, x: f64| ST96_LOWER[k] <= x && x <= ST96_UPPER[k];
    facts
        .pin("dfg.re", g6(mean_inflow() * DIAMETER / VISCOSITY), "100", oracle)?
        .pin("dfg.inlet.mean", g6(mean_inflow()), "1", oracle)?
        .pin("dfg.inlet.peak", g6(PEAK_INFLOW), "1.5", oracle)?
        .pin("dfg.hold.arith", g6(swings_per_second(HOLD_STROUHAL)), "3", oracle)?
        .pin(
            "dfg.swings.per.s",
            g6(swings_per_second(REF_STROUHAL)),
            "3.01841",
            oracle,
        )?
        .pin(
            "dfg.swing.period",
            g6(1.0 / swings_per_second(REF_STROUHAL)),
            "0.3313",
            oracle,
        )?
        .pin("ref.drag.round", g6(REF_DRAG_PEAK), "3.22739", oracle)?
        .pin("ref.lift.round", g6(REF_LIFT_PEAK), "0.986583", oracle)?
        .pin("ref.st.round", g6(REF_STROUHAL), "0.301841", oracle)?
        .pin(
            "ref.st.in.1996.range",
            py_bool(in_interval(2, REF_STROUHAL)),
            "True",
            oracle,
        )?
        .pin(
            "ref.lift.below.1996.range",
            py_bool(REF_LIFT_PEAK < ST96_LOWER[1]),
            "True",
            oracle,
        )?
        .pin("data.bytes.per.state", g6(bytes as f64), "2.30912e+06", oracle)?
        .pin("data.gb.per.minute", g6(per_minute), "8.31283", oracle)?
        .pin(
            "c20.ref.period",
            g6(REF_PERIOD),
            "0.3313",
            "20_7 Precisely: 1/f = 0.3313 s",
        )?
        .pin(
            "c20.ref.st.shown",
            &l.ref_strouhal,
            "0.302",
            "20_4: about zero point three zero two",
        )?
        .pin(
            "c20.ref.drag.shown",
            &l.ref_drag,
            "3.23",
            "20_4: about three point two three",
        )?
        .pin(
            "c20.ref.lift.shown",
            &l.ref_lift,
            "0.987",
            "20_4: about zero point nine eight seven",
        )?
        .pin(
            "c20.ref.dp.shown",
            &l.ref_delta_p,
            "2.485",
            "20_4 Precisely: ΔP ≈ 2.485",
        )?
        .pin("c20.lift.dot.shown", &l.dot, "0.987", "20_5: just below that range")?
        .pin(
            "c20.interval.shown",
            l.interval.join(" to "),
            "0.99 to 1.01",
            "20_5: zero point nine nine to one point zero one",
        )?
        .pin(
            "c20.answer.shown",
            &l.answer,
            "f = St × Ū / D ≈ 3.02 per second",
            "20_7: about three point oh two swings per second",
        )?
        .pin(
            "c20.every.shown",
            &l.every,
            "one swing every 0.33 s",
            "20_7: one every third of a second",
        )?
        .pin("c20.length.shown", &l.length, "2.2 m", "ST96 2D-2 channel length")?
        .pin("c20.height.shown", &l.height, "0.41 m", "ST96 2D-2 channel height")?
        .pin(
            "c20.cylinder.shown",
            &l.cylinder,
            "D = 0.1 m at (0.2 m, 0.2 m)",
            "ST96 2D-2 cylinder",
        )?
        .pin(
            "c20.inflow.shown",
            &l.inflow,
            "prescribed inflow: peak 1.5 m/s, average 1 m/s",
            "ST96: Um = 1.5 m/s, Ū = 2 U(0, H/2)/3 = 1 m/s",
        )?
        .pin(
            "c20.plan.cells",
            format!("{} × {}", cells[0], cells[1]),
            "880 × 164",
            "20_2 Precisely",
        )?
        .pin(
            "c20.plan.mb.shown",
            wake_ledger::decimals(bytes as f64 / 1e6, 2),
            "2.31",
            "20_2 Precisely: 2.31 MB",
        )?
        .pin(
            "c20.plan.gb.shown",
            wake_ledger::decimals(per_minute, 1),
            "8.3",
            "20_2 Precisely: about 8.3 GB",
        )?
        .add("c20.prediction.seconds", PREDICTION_SECONDS)?;
    Ok(())
}

// ------------------------------------------------------------------------------------ layout

type Place = ([f64; 2], [f64; 2], f64);

/// Pacing, in seconds of scene time. No number on screen depends on these.
pub mod pacing {
    pub const FADE: f64 = 0.5;
    pub const MORPH: f64 = 1.1;
    pub const PULSE: f64 = 0.8;
    pub const BEAT: f64 = 0.8;
    /// 20_7: the metronome keeps swinging this long after the line's last sentence starts.
    pub const METRONOME_AFTER: f64 = 3.0;
}
use pacing::{BEAT, FADE, METRONOME_AFTER, MORPH, PULSE};

// Layout, in scene units (frame 16 × 9; the opening's safe area is x ∈ [−7.1, 7.1], y ∈ [−4, 4]).
/// 20_1 and 20_9: the opening's tunnel and its question card, where the opening and chapter 09 leave them.
const TUNNEL_WIDTH: f64 = 11.5;
const TUNNEL_CENTER: [f64; 2] = [0.0, 1.15];
const CARD_FULL: Place = ([0.0, -1.25], [8.2, 0.8], 30.0);
/// 20_1's labels of the exact shape.
const CYLINDER_AT: [f64; 2] = [-4.0, 2.55];
const TEST_FULL_AT: [f64; 2] = [2.6, 2.55];
const HEIGHT_AT: [f64; 2] = [6.45, 1.15];
const LENGTH_Y: f64 = -0.05;
const LENGTH_AT: [f64; 2] = [0.0, -0.32];
const INFLOW_AT: [f64; 2] = [-5.75, -0.32];
/// Inflow arrows: scene units per m/s at the opening's width (the peak arrow stays clear of the cylinder).
const ARROW_PER_SPEED: f64 = 0.4;
const INFLOW_ARROWS: usize = 9;
/// 20_2 to 20_8: the same tunnel as a strip at the top left, the card at the top right.
const STRIP_WIDTH: f64 = 6.2;
const STRIP_CENTER: [f64; 2] = [-3.9, 3.35];
const CARD_STRIP: Place = ([3.6, 3.62], [6.6, 0.56], 21.0);
const TEST_STRIP_AT: [f64; 2] = [3.6, 3.0];
/// 20_8's corner label: right-aligned under the strip, then under the full tunnel.
const STREAK_STRIP_AT: [f64; 2] = [-0.8, 2.56];
const STREAK_FULL_AT: [f64; 2] = [5.75, -0.32];
/// The table: label column, run columns, the close-enough column and the reference column.
const LABEL_X: f64 = -7.05;
const COLUMN_X: [f64; 7] = [-2.95, -1.53, -0.11, 1.31, 2.73, 4.15, 5.57];
const CLOSE: usize = 5;
const REFERENCE: usize = 6;
const HEADER_Y: f64 = 2.2;
const NOTE_Y: f64 = 1.9;
const ROW_Y0: f64 = 1.55;
const ROW_PITCH: f64 = 0.29;
const FOOTNOTE_Y: f64 = -2.62;
/// The lower third: the reference label and Precisely lines.
const LOWER_Y: [f64; 4] = [-2.98, -3.31, -3.64, -3.97];
/// 20_3, 20_5 to 20_7: a card over the run and close-enough columns (their cells fade out under it).
const OVERLAY_AT: [f64; 2] = [0.6, -0.35];
const OVERLAY_SIZE: [f64; 2] = [8.6, 3.0];
/// 20_5's number line: lift peak 0.98 to 1.02 across AXIS_X, at AXIS_Y.
const AXIS_TICKS: [f64; 5] = [0.98, 0.99, 1.0, 1.01, 1.02];
const AXIS_X: [f64; 2] = [-2.9, 4.1];
const AXIS_Y: f64 = -0.55;
/// 20_6 and 20_7 inside the card.
const GIVENS_AT: [f64; 2] = [0.6, 0.8];
const FORMULA_AT: [f64; 2] = [-1.8, -0.1];
const OPTIONS_X: f64 = 2.6;
const OPTIONS_Y: [f64; 3] = [0.2, -0.2, -0.6];
const COUNTDOWN_AT: [f64; 2] = [0.6, -1.4];
const PIVOT: [f64; 2] = [3.1, -1.55];
const ARM: f64 = 1.3;
/// The metronome swings ±25°.
const SWING: f64 = 25.0 * std::f64::consts::PI / 180.0;
const EVERY_AT: [f64; 2] = [-3.3, -0.95];
const CLOCK_AT: [f64; 2] = [-3.3, -1.38];
/// 20_9's checklist, left-aligned under the full card.
const CHECKLIST_X: f64 = -3.9;
const CHECKLIST_Y0: f64 = -2.05;
const CHECKLIST_PITCH: f64 = 0.42;

fn row_y(quantity: usize) -> f64 {
    ROW_Y0 - ROW_PITCH * quantity as f64
}

fn axis_x(lift: f64) -> f64 {
    AXIS_X[0] + (AXIS_X[1] - AXIS_X[0]) * (lift - AXIS_TICKS[0]) / (AXIS_TICKS[4] - AXIS_TICKS[0])
}

fn card_fill() -> Srgb {
    hex("#182433")
}

/// Published reference values are reference grey (the closing's colour table).
fn reference_grey() -> Srgb {
    Palette::dim()
}

fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

/// A rough text width: about half an em per character.
fn text_width(words: &str, size: f64) -> f64 {
    words.chars().count() as f64 * size * 0.0052
}

/// The opening's tunnel in metres (story.rs passes the retained snapshot's geometry, as for chapter 05).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TunnelShape {
    pub length: f64,
    pub height: f64,
    /// Centre x, centre y, radius.
    pub cylinder: [f64; 3],
}

pub struct Chapter20 {
    pub kit: Kit,
    pub tunnel: Option<TunnelShape>,
    /// Our entries: `Ledger::pending` (every entry "?") or a parsed `wake-ledger01` file.
    pub ledger: Ledger,
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

/// A rounded card centred at `at`: fill colour and opacity, edge colour and width.
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

/// The scene point of (x, y) in metres for a tunnel drawn `width` wide about `center`.
fn tunnel_at(g: TunnelShape, width: f64, center: [f64; 2], x: f64, y: f64) -> [f64; 2] {
    let scale = width / g.length;
    [
        center[0] + scale * (x - g.length / 2.0),
        center[1] + scale * (y - g.height / 2.0),
    ]
}

/// The static tunnel and its teaching grid, drawn as chapter 09's `tunnel` draws them: the wall
/// box, the cylinder, two row lines and the column lines one cell (H/3) apart about the middle.
fn tunnel_drawing(stage: &mut Stage<'_>, g: TunnelShape, width: f64, center: [f64; 2]) -> fmn::Result<Vec<Mob>> {
    let scale = width / g.length;
    let at = |x: f64, y: f64| tunnel_at(g, width, center, x, y);
    let [cx, cy, radius] = g.cylinder;
    let mut out = vec![
        rect(
            stage,
            center,
            [width, scale * g.height],
            (hex("#17283C"), 1.0),
            (Palette::dim(), 1.7),
        )?,
        stage.arena_mut().add(
            Circle::new()
                .radius(scale * radius)
                .style(
                    Style::default()
                        .fill(hex("#0A121F"), 1.0)
                        .stroke(Palette::ink(), 1.6, 1.0),
                )
                .build()
                .moved_to(p2(at(cx, cy))),
        ),
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

/// The ST96 inflow at the inlet: INFLOW_ARROWS arrows, all at the mean speed (`profile` false) or
/// at U(0, y) (true), and the profile's curve through the tips. A boundary condition, not a flow
/// result.
fn inflow_drawing(stage: &mut Stage<'_>, width: f64, center: [f64; 2], profile: bool) -> fmn::Result<(Vec<Mob>, Mob)> {
    let scale = ARROW_PER_SPEED * width / TUNNEL_WIDTH;
    let thickness = 1.4 * width / TUNNEL_WIDTH;
    let h = BENCH.height;
    let mut arrows = Vec::new();
    for j in 0..INFLOW_ARROWS {
        let y = h * (j as f64 + 0.5) / INFLOW_ARROWS as f64;
        let from = tunnel_at(BENCH, width, center, 0.0, y);
        let speed = if profile { inflow(y) } else { mean_inflow() };
        arrows.push(arrow(
            stage,
            from,
            [from[0] + scale * speed, from[1]],
            Palette::blue(),
            thickness,
        )?);
    }
    let tips: Vec<[f64; 2]> = (0..=40)
        .map(|k| {
            let y = h * f64::from(k) / 40.0;
            let at = tunnel_at(BENCH, width, center, 0.0, y);
            [at[0] + scale * if profile { inflow(y) } else { mean_inflow() }, at[1]]
        })
        .collect();
    Ok((
        arrows,
        curve(stage, &tips, Palette::blue(), 1.6 * width / TUNNEL_WIDTH, 0.8),
    ))
}

// ------------------------------------------------------------------------------------ staging

/// The table's drawn pieces.
struct Table {
    titles: Vec<Mob>,
    rungs: Vec<Mob>,
    legend: Mob,
    labels: Vec<Mob>,
    /// cells[role][quantity]
    cells: Vec<Vec<Mob>>,
    footnote: Mob,
    target: Vec<Mob>,
    tolerances: Vec<Mob>,
    reference_title: Vec<Mob>,
    reference: Vec<Mob>,
}

impl Table {
    fn all_cells(&self) -> Vec<Mob> {
        self.cells.iter().flatten().copied().collect()
    }

    fn pending(&self, ledger: &Ledger) -> Vec<Mob> {
        let mut out = Vec::new();
        for (role, column) in self.cells.iter().enumerate() {
            for (q, &m) in column.iter().enumerate() {
                if ledger.cell(role, q).is_none() {
                    out.push(m);
                }
            }
        }
        out
    }

    /// What the overlay card covers: every run cell and every tolerance.
    fn covered(&self) -> Vec<Mob> {
        let mut out = self.all_cells();
        out.extend(&self.tolerances);
        out
    }
}

/// The chapter's state: what is on screen, and a pending batch of animations.
struct Ch20<'a> {
    k: &'a Kit,
    on: Vec<Mob>,
    pending: Vec<Box<dyn Animation>>,
}

impl Ch20<'_> {
    // Text through the kit, so the layout lint and the glyph check see every string.
    fn text(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: [f64; 2]) -> fmn::Result<Mob> {
        self.k.text(stage, s, size, color, p2(at))
    }

    fn text_left(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: [f64; 2]) -> fmn::Result<Mob> {
        self.k.text_left(stage, s, size, color, p2(at))
    }

    fn text_right(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: [f64; 2]) -> fmn::Result<Mob> {
        let m = self.k.text(stage, s, size, color, p2(at))?;
        stage.move_to(m, p2(at), RIGHT);
        Ok(m)
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

    /// Each of `from` (on screen) takes the shape and paint of its partner in `to` (detached).
    fn morph(&mut self, stage: &mut Stage<'_>, from: &[Mob], to: &[Mob], secs: f64) -> fmn::Result<()> {
        if from.len() != to.len() {
            return Err(fail(format!(
                "chapter 20: morph of {} pieces into {}",
                from.len(),
                to.len()
            )));
        }
        for (&a, &b) in from.iter().zip(to) {
            if self.on.contains(&a) {
                self.pending.push(stage.prepare(Transform::new(a, b).rt(secs))?);
            }
        }
        Ok(())
    }

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

    /// Finish what is queued, then start line `id`.
    fn say(&mut self, stage: &mut Stage<'_>, id: &str) -> fmn::Result<()> {
        if !CUES.iter().any(|c| c.0 == id) {
            return Err(fail(format!("chapter 20 has no cue {id}")));
        }
        self.play(stage)?;
        println!("Chapter 20 cue {id} at {:.6} s", stage.scene().time().to_f64());
        self.k.say(stage, id)
    }

    /// Finish what is queued, then wait for sentence `j` of line `id` (nothing if it has begun).
    fn until(&mut self, stage: &mut Stage<'_>, id: &str, j: usize) -> fmn::Result<()> {
        let sentences = CUES.iter().find(|c| c.0 == id).map_or(0, |c| c.1);
        if j >= sentences {
            return Err(fail(format!("chapter 20 line {id} has no sentence {j}")));
        }
        self.play(stage)?;
        self.k.until_sentence(stage, id, j)
    }

    /// Finish what is queued, let the line end (plus the kit's breath), before a wipe.
    fn hold(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        self.play(stage)?;
        self.k.hold(stage)
    }

    /// The opening's question card, drawn as chapter 09 draws it: an open dim frame (corner 0.12,
    /// stroke 2.0) and ink words.
    fn question(&self, stage: &mut Stage<'_>, place: Place) -> fmn::Result<[Mob; 2]> {
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

    /// Lines of a lower third, the first at LOWER_Y[first].
    fn lower(&self, stage: &mut Stage<'_>, lines: &[&str], first: usize) -> fmn::Result<Vec<Mob>> {
        lines
            .iter()
            .enumerate()
            .map(|(i, words)| self.text(stage, words, 17.0, Palette::dim(), [0.0, LOWER_Y[first + i]]))
            .collect()
    }

    /// Change the lower-third without laying two sets of type over each other.
    fn swap(&mut self, stage: &mut Stage<'_>, old: &[Mob], new: &[Mob]) -> fmn::Result<()> {
        self.cut(stage, old);
        self.appear(stage, new, FADE)
    }

    /// The table, built detached: our entries from the ledger ("?" where pending), the declared
    /// tolerances ("?" until root declares them), and the reference column.
    fn table(&self, stage: &mut Stage<'_>, ledger: &Ledger, l: &Labels) -> fmn::Result<Table> {
        let (ink, dim) = (Palette::ink(), Palette::dim());
        let mut titles = Vec::new();
        let mut rungs = Vec::new();
        let mut cells = Vec::new();
        for (role, title) in COLUMNS.iter().enumerate() {
            let x = COLUMN_X[role];
            titles.push(self.text(stage, title, 19.0, ink, [x, HEADER_Y])?);
            let rung = match ledger.run(role) {
                Run::Measured(m) => format!("lvl {}, {} s", m.level, g6(m.dt)),
                Run::Pending(_) => RUNGS[role].to_owned(),
            };
            rungs.push(self.text(stage, &rung, 15.0, dim, [x, NOTE_Y])?);
            let mut column = Vec::new();
            for q in 0..wake_ledger::QUANTITIES.len() {
                let words = ledger
                    .cell(role, q)
                    .map_or_else(|| PENDING.to_owned(), |x| wake_ledger::shown(q, x));
                column.push(self.text(stage, &words, 18.0, ink, [x, row_y(q)])?);
            }
            cells.push(column);
        }
        let legend = self.text_left(stage, LEGEND, 15.0, dim, [LABEL_X, NOTE_Y])?;
        let mut labels = Vec::new();
        for (q, words) in ROW_LABELS.iter().enumerate() {
            labels.push(self.text_left(stage, words, 18.0, ink, [LABEL_X, row_y(q)])?);
        }
        let footnote = self.text_left(stage, FOOTNOTE, 16.0, dim, [LABEL_X, FOOTNOTE_Y])?;
        let declared = wake_ledger::TOLERANCES
            .iter()
            .any(|&(_, q)| ledger.close_enough(q).is_some());
        let words = if declared { CLOSE_DECLARED } else { CLOSE_PENDING };
        let at = [COLUMN_X[CLOSE], HEADER_Y];
        let target = vec![
            rounded(
                stage,
                at,
                [text_width(words, 17.0) + 0.3, 0.42],
                (card_fill(), 1.0),
                (ink, 1.4),
            )?,
            self.text(stage, words, 17.0, ink, at)?,
        ];
        let mut tolerances = Vec::new();
        for &(_, q) in &wake_ledger::TOLERANCES {
            let words = ledger
                .close_enough(q)
                .map_or_else(|| PENDING.to_owned(), wake_ledger::shown_tolerance);
            tolerances.push(self.text(stage, &words, 18.0, ink, [COLUMN_X[CLOSE], row_y(q)])?);
        }
        let grey = reference_grey();
        let x = COLUMN_X[REFERENCE];
        let reference_title = vec![
            self.text(stage, REFERENCE_TITLE[0], 19.0, grey, [x, HEADER_Y])?,
            self.text(stage, REFERENCE_TITLE[1], 15.0, grey, [x, NOTE_Y])?,
        ];
        let shown = [&l.ref_strouhal, &l.ref_drag, &l.ref_lift, &l.ref_delta_p];
        let mut reference = Vec::new();
        for (&q, words) in REFERENCE_ROWS.iter().zip(shown) {
            reference.push(self.text(stage, words, 18.0, grey, [x, row_y(q)])?);
        }
        Ok(Table {
            titles,
            rungs,
            legend,
            labels,
            cells,
            footnote,
            target,
            tolerances,
            reference_title,
            reference,
        })
    }

    /// Chapter 16's table (16_7) on the overlay card, every entry "?".
    fn a2_table(&self, stage: &mut Stage<'_>) -> fmn::Result<(Vec<Mob>, Vec<Mob>)> {
        let (ink, dim) = (Palette::ink(), Palette::dim());
        let mut frame = vec![
            rounded(stage, OVERLAY_AT, OVERLAY_SIZE, (card_fill(), 1.0), (dim, 1.4))?,
            self.text(stage, A2_TITLE, 19.0, ink, [OVERLAY_AT[0], 0.85])?,
        ];
        let xs = [1.9, 3.0, 4.1];
        for (x, words) in xs.iter().zip(A2_COLUMNS) {
            frame.push(self.text(stage, words, 17.0, ink, [*x, 0.5])?);
        }
        let mut marks = Vec::new();
        for (i, words) in A2_ROWS.iter().enumerate() {
            let y = 0.18 - 0.27 * i as f64;
            frame.push(self.text_left(stage, words, 16.0, dim, [-3.35, y])?);
            for x in xs {
                marks.push(self.text(stage, PENDING, 17.0, ink, [x, y])?);
            }
        }
        Ok((frame, marks))
    }

    /// HOLD 20_h1: the line ends plus a breath (`kit.hold`); a marked five-second countdown. The
    /// card (givens, formula, choices) is already up; nothing of 20_7's answer is drawn before
    /// this returns.
    fn predict(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        self.play(stage)?;
        self.k.hold(stage)?;
        self.k.mark(stage, "predict");
        for second in (1..=PREDICTION_SECONDS).rev() {
            let clock = self.text(stage, &second.to_string(), 30.0, Palette::hero(), COUNTDOWN_AT)?;
            self.put(stage, &[clock])?;
            stage.wait(1.0)?;
            self.cut(stage, &[clock]);
        }
        Ok(())
    }

    /// 20_7's metronome: an arm and bob on a base, swinging from the left mark to the right and
    /// back, once every `period` seconds, `swings` times, with a tick at each swing's start. Each
    /// half swing ends at its exact time on the scene clock, so frame rounding never accumulates.
    fn metronome(&mut self, stage: &mut Stage<'_>, period: f64, swings: u32) -> fmn::Result<Vec<Mob>> {
        let (ink, dim, faint) = (Palette::ink(), Palette::dim(), Palette::faint());
        let tip = |angle: f64| [PIVOT[0] - ARM * angle.sin(), PIVOT[1] + ARM * angle.cos()];
        let bob_at = |angle: f64| [PIVOT[0] - 0.8 * ARM * angle.sin(), PIVOT[1] + 0.8 * ARM * angle.cos()];
        let mut parts = vec![
            line(stage, [PIVOT[0] - 0.4, PIVOT[1]], [PIVOT[0] + 0.4, PIVOT[1]], dim, 2.0)?,
            disc(stage, PIVOT, 0.05, dim),
        ];
        for side in [SWING, -SWING] {
            let [x, y] = tip(side);
            let [dx, dy] = [x - PIVOT[0], y - PIVOT[1]];
            parts.push(line(
                stage,
                [x + 0.08 * dx, y + 0.08 * dy],
                [x + 0.2 * dx, y + 0.2 * dy],
                faint,
                2.0,
            )?);
        }
        let arm = line(stage, PIVOT, tip(SWING), ink, 2.4)?;
        let bob = disc(stage, bob_at(SWING), 0.08, ink);
        let [x, y] = tip(SWING);
        let tick = disc(
            stage,
            [x + 0.3 * (x - PIVOT[0]), y + 0.3 * (y - PIVOT[1])],
            0.06,
            Palette::hero(),
        );
        parts.extend([arm, bob]);
        self.appear(stage, &parts, FADE)?;
        self.play(stage)?;
        let start = stage.scene().time().to_f64();
        for k in 0..swings {
            for half in 0..2u32 {
                if half == 0 {
                    self.put(stage, &[tick])?;
                }
                let end = start + period * (f64::from(k) + 0.5 * f64::from(half + 1));
                let now = stage.scene().time().to_f64();
                let secs = (end - now).max(1e-3);
                let angle = if half == 0 { -2.0 * SWING } else { 2.0 * SWING };
                for m in [arm, bob] {
                    let turn = rotate(m, angle).with_about_point(p2(PIVOT)).rt(secs);
                    self.pending.push(stage.prepare(turn)?);
                }
                self.play(stage)?;
                if half == 0 {
                    self.cut(stage, &[tick]);
                }
            }
        }
        let elapsed = stage.scene().time().to_f64() - start;
        println!(
            "Chapter 20 metronome: {swings} swings in {elapsed:.6} s of scene time \
             ({:.6} s each; period {period:.6} s)",
            elapsed / f64::from(swings)
        );
        Ok(parts)
    }
}

/// The ledger's state, for the render log: each run, the compare, the slot, W2.
fn log_ledger(ledger: &Ledger) {
    println!("Chapter 20 entries from: {}", ledger.source);
    for (role, name) in wake_ledger::ROLES.iter().enumerate() {
        match ledger.run(role) {
            Run::Measured(m) => println!(
                "Chapter 20 {name}: measured, level {}, dt {} s, analysis {}, receipt {}",
                m.level, m.dt, m.analysis_sha256, m.receipt_sha256
            ),
            Run::Pending(why) => println!("Chapter 20 {name}: pending ({why})"),
        }
    }
    let compare = match &ledger.compare {
        Compare::NotRun => "not run".to_owned(),
        Compare::Refused(why) => format!("refused ({why})"),
        Compare::Withheld(why) => format!("ran, slot withheld ({why})"),
        Compare::Filled(why) => format!("ran, slot filled ({why})"),
    };
    println!("Chapter 20 compare: {compare}");
    match &ledger.slot {
        Some(slot) => println!(
            "Chapter 20 SLOT 20_v1 filled but not voiced (root wires and records it): {:?}",
            slot.values
        ),
        None => println!("Chapter 20 SLOT 20_v1 not filled; nothing of it is drawn or voiced"),
    }
    println!("Chapter 20 W2 pending: {}", ledger.w2);
}

impl SceneConstruct for Chapter20 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        // The refusal comes before the first frame.
        let tunnel = self.tunnel.ok_or_else(|| {
            fail("chapter 20 opens on the tunnel: render it with --tunnel and the retained startup snapshots".into())
        })?;
        for fact in facts().map_err(fail)?.print().lines() {
            println!("Chapter 20 fact {fact}");
        }
        log_ledger(&self.ledger);
        let ledger = &self.ledger;
        let l = labels();
        let (ink, dim, hero) = (Palette::ink(), Palette::dim(), Palette::hero());
        let mut s = Ch20 {
            k: &self.kit,
            on: Vec::new(),
            pending: Vec::new(),
        };

        // 20_1: back to the opening's tunnel and its question card; then the standard test's shape.
        s.say(stage, "20_1")?;
        let drawing = tunnel_drawing(stage, tunnel, TUNNEL_WIDTH, TUNNEL_CENTER)?;
        let question = s.question(stage, CARD_FULL)?;
        s.appear(stage, &drawing, FADE)?;
        s.appear(stage, &question, FADE)?;
        s.until(stage, "20_1", 1)?;
        s.pulse(stage, &question, None)?;
        s.until(stage, "20_1", 2)?;
        if tunnel == BENCH {
            println!("Chapter 20: the retained tunnel already has 2D-2's shape, so 20_1 labels it and adds the inflow");
        } else {
            let exact = tunnel_drawing(stage, BENCH, TUNNEL_WIDTH, TUNNEL_CENTER)?;
            if exact.len() == drawing.len() {
                s.morph(stage, &drawing, &exact, MORPH)?;
            } else {
                return Err(fail(format!(
                    "chapter 20: the tunnel {tunnel:?} cannot morph into 2D-2's grid"
                )));
            }
        }
        let (uniform, _) = inflow_drawing(stage, TUNNEL_WIDTH, TUNNEL_CENTER, false)?;
        let (bent, profile) = inflow_drawing(stage, TUNNEL_WIDTH, TUNNEL_CENTER, true)?;
        s.appear(stage, &uniform, FADE)?;
        s.play(stage)?;
        s.morph(stage, &uniform, &bent, MORPH)?;
        s.draw(stage, &[profile], MORPH)?;
        let left = tunnel_at(BENCH, TUNNEL_WIDTH, TUNNEL_CENTER, 0.0, 0.0)[0];
        let right = tunnel_at(BENCH, TUNNEL_WIDTH, TUNNEL_CENTER, BENCH.length, 0.0)[0];
        let mut shape = vec![
            line(stage, [left, LENGTH_Y], [right, LENGTH_Y], dim, 1.4)?,
            line(stage, [left, LENGTH_Y - 0.08], [left, LENGTH_Y + 0.08], dim, 1.4)?,
            line(stage, [right, LENGTH_Y - 0.08], [right, LENGTH_Y + 0.08], dim, 1.4)?,
            s.text(stage, &l.length, 22.0, ink, LENGTH_AT)?,
            s.text(stage, &l.height, 22.0, ink, HEIGHT_AT)?,
            s.text(stage, &l.cylinder, 22.0, ink, CYLINDER_AT)?,
            s.text_left(stage, &l.inflow, 20.0, dim, INFLOW_AT)?,
        ];
        let test = s.text(stage, TEST_LABEL, 24.0, ink, TEST_FULL_AT)?;
        shape.push(test);
        s.appear(stage, &shape, FADE)?;
        s.play(stage)?;

        // 20_2: the tunnel and card move to the top as a strip; the plan becomes a table.
        s.say(stage, "20_2")?;
        let labels_only = shape[..shape.len() - 1].to_vec();
        s.vanish(stage, &labels_only, FADE)?;
        s.play(stage)?;
        let strip = tunnel_drawing(stage, BENCH, STRIP_WIDTH, STRIP_CENTER)?;
        let (strip_arrows, strip_profile) = inflow_drawing(stage, STRIP_WIDTH, STRIP_CENTER, true)?;
        let card_strip = s.question(stage, CARD_STRIP)?;
        let test_strip = s.text(stage, TEST_LABEL, 18.0, dim, TEST_STRIP_AT)?;
        s.morph(stage, &drawing, &strip, MORPH)?;
        s.morph(stage, &uniform, &strip_arrows, MORPH)?;
        s.morph(stage, &[profile], &[strip_profile], MORPH)?;
        s.morph(stage, &question, &card_strip, MORPH)?;
        s.morph(stage, &[test], &[test_strip], MORPH)?;
        let t = s.table(stage, ledger, &l)?;
        s.until(stage, "20_2", 1)?;
        s.appear(stage, &t.titles[..3], FADE)?;
        s.appear(stage, &t.rungs[..3], FADE)?;
        s.appear(stage, &[t.legend], FADE)?;
        let page_a = s.lower(stage, &PRECISELY_20_2A, 0)?;
        s.appear(stage, &page_a, FADE)?;
        s.until(stage, "20_2", 2)?;
        s.appear(stage, &t.titles[3..], FADE)?;
        s.appear(stage, &t.rungs[3..], FADE)?;
        s.until(stage, "20_2", 3)?;
        s.appear(stage, &t.labels, FADE)?;
        s.appear(stage, &t.all_cells(), FADE)?;
        s.appear(stage, &[t.footnote], FADE)?;
        let page_b = s.lower(stage, &PRECISELY_20_2B, 0)?;
        s.swap(stage, &page_a, &page_b)?;
        s.play(stage)?;
        println!(
            "Chapter 20 table: {} of 70 run entries measured, {} pending (\"?\")",
            70 - t.pending(ledger).len(),
            t.pending(ledger).len()
        );

        // 20_3: close enough is declared first; chapter 16's table slides in, every entry "?".
        s.say(stage, "20_3")?;
        s.appear(stage, &t.target, FADE)?;
        s.appear(stage, &t.tolerances, FADE)?;
        let page_c = s.lower(stage, &PRECISELY_20_2C, 0)?;
        s.play(stage)?;
        s.cut(stage, &page_b);
        s.put(stage, &page_c)?;
        s.pulse(stage, &t.target[1..], None)?;
        s.until(stage, "20_3", 1)?;
        let waiting = t.pending(ledger);
        s.pulse(stage, &waiting, None)?;
        s.play(stage)?;
        let (a2, a2_marks) = s.a2_table(stage)?;
        s.vanish(stage, &t.covered(), FADE)?;
        s.appear(stage, &a2, FADE)?;
        s.appear(stage, &a2_marks, FADE)?;
        let page_d = s.lower(stage, &PRECISELY_20_2D, 0)?;
        s.swap(stage, &page_c, &page_d)?;
        s.until(stage, "20_3", 2)?;
        s.pulse(stage, &a2_marks, None)?;
        s.hold(stage)?;
        s.vanish(stage, &a2, FADE)?;
        s.vanish(stage, &a2_marks, FADE)?;
        s.vanish(stage, &page_d, FADE)?;
        s.appear(stage, &t.covered(), FADE)?;

        // 20_4: the reference column fills, grey and labelled as another program's calculations.
        s.say(stage, "20_4")?;
        let label = s.lower(stage, &[REFERENCE_LABEL], 0)?;
        let precisely_4 = s.lower(stage, &PRECISELY_20_4, 1)?;
        s.appear(stage, &t.reference_title, FADE)?;
        s.appear(stage, &label, FADE)?;
        s.appear(stage, &precisely_4, FADE)?;
        s.until(stage, "20_4", 1)?;
        s.appear(stage, &[t.reference[1]], FADE)?;
        s.until(stage, "20_4", 2)?;
        s.appear(stage, &[t.reference[2]], FADE)?;
        s.until(stage, "20_4", 3)?;
        s.appear(stage, &[t.reference[0], t.reference[3]], FADE)?;

        // 20_5: the 1996 estimated interval as a grey band, the 2014 value just below it.
        s.say(stage, "20_5")?;
        s.pulse(stage, &t.reference, None)?;
        s.until(stage, "20_5", 1)?;
        let precisely_5 = s.lower(stage, &PRECISELY_20_5, 1)?;
        s.swap(stage, &precisely_4, &precisely_5)?;
        s.vanish(stage, &t.covered(), FADE)?;
        let overlay = rounded(stage, OVERLAY_AT, OVERLAY_SIZE, (card_fill(), 1.0), (dim, 1.4))?;
        let grey = reference_grey();
        let mut axis = vec![
            s.text(stage, LIFT_AXIS, 22.0, ink, [OVERLAY_AT[0], 0.85])?,
            line(stage, [AXIS_X[0], AXIS_Y], [AXIS_X[1], AXIS_Y], dim, 1.6)?,
        ];
        for (&t_at, words) in AXIS_TICKS.iter().zip(&l.ticks) {
            let x = axis_x(t_at);
            axis.push(line(stage, [x, AXIS_Y - 0.07], [x, AXIS_Y + 0.07], dim, 1.6)?);
            axis.push(s.text(stage, words, 17.0, dim, [x, AXIS_Y - 0.3])?);
        }
        let (lo, hi) = (axis_x(ST96_LOWER[1]), axis_x(ST96_UPPER[1]));
        let band = vec![
            rect(
                stage,
                [(lo + hi) / 2.0, AXIS_Y],
                [hi - lo, 0.3],
                (grey, 0.35),
                (grey, 1.2),
            )?,
            s.text(stage, BAND_LABEL, 20.0, grey, [(lo + hi) / 2.0, AXIS_Y + 0.5])?,
        ];
        s.appear(stage, &[overlay], FADE)?;
        s.appear(stage, &axis, FADE)?;
        s.appear(stage, &band, FADE)?;
        s.until(stage, "20_5", 2)?;
        let x = axis_x(REF_LIFT_PEAK);
        let dot = vec![
            disc(stage, [x, AXIS_Y], 0.09, grey),
            s.text(stage, &l.dot, 18.0, grey, [x, AXIS_Y + 0.35])?,
            s.text_left(stage, DOT_LABEL, 18.0, grey, [AXIS_X[0], AXIS_Y - 0.75])?,
        ];
        s.appear(stage, &dot, FADE)?;

        // 20_6: the prediction card, then HOLD 20_h1.
        s.say(stage, "20_6")?;
        s.vanish(stage, &axis, FADE)?;
        s.vanish(stage, &band, FADE)?;
        s.vanish(stage, &dot, FADE)?;
        s.vanish(stage, &precisely_5, FADE)?;
        s.until(stage, "20_6", 1)?;
        let givens = s.text(stage, QUESTION, 22.0, ink, GIVENS_AT)?;
        let formula = vec![
            rounded(stage, FORMULA_AT, [2.7, 0.62], (card_fill(), 1.0), (ink, 1.4))?,
            s.text(stage, FORMULA, 24.0, ink, FORMULA_AT)?,
            s.text(stage, FORMULA_NOTE, 16.0, dim, [FORMULA_AT[0], FORMULA_AT[1] - 0.55])?,
        ];
        s.appear(stage, &[givens], FADE)?;
        s.appear(stage, &formula, FADE)?;
        s.until(stage, "20_6", 2)?;
        let mut choices = Vec::new();
        for (words, y) in OPTIONS.iter().zip(OPTIONS_Y) {
            choices.push(s.text(stage, words, 22.0, dim, [OPTIONS_X, y])?);
        }
        s.appear(stage, &choices, FADE)?;
        s.predict(stage)?;

        // 20_7: about three; f = St Ū / D ≈ 3.02 per second, and a metronome at that rate.
        s.say(stage, "20_7")?;
        s.pulse(stage, &[choices[1]], Some([1.0, 0.847, 0.4]))?;
        s.until(stage, "20_7", 1)?;
        s.vanish(stage, &[givens, choices[0], choices[2]], FADE)?;
        s.vanish(stage, &[choices[1]], FADE)?;
        let answer = s.text(stage, &l.answer, 24.0, hero, GIVENS_AT)?;
        let notes = vec![
            s.text_left(stage, &l.every, 20.0, ink, EVERY_AT)?,
            s.text_left(stage, CLOCK_NOTE, 20.0, dim, CLOCK_AT)?,
        ];
        let precisely_7 = s.lower(stage, &PRECISELY_20_7, 1)?;
        s.appear(stage, &[answer], FADE)?;
        s.appear(stage, &notes, FADE)?;
        s.appear(stage, &precisely_7, FADE)?;
        let period = 1.0 / swings_per_second(REF_STROUHAL);
        let starts = s.k.sentence_starts("20_7");
        let swings = match (starts.get(1), starts.get(2)) {
            (Some(a), Some(b)) => ((b - a + METRONOME_AFTER) / period).ceil().clamp(6.0, 40.0) as u32,
            _ => 12,
        };
        let clock = s.metronome(stage, period, swings)?;
        s.until(stage, "20_7", 2)?;

        // 20_8: the corner label for every tunnel frame from here on.
        s.say(stage, "20_8")?;
        let mut card = vec![answer, overlay];
        card.extend(&formula);
        card.extend(&notes);
        card.extend(&clock);
        s.vanish(stage, &card, FADE)?;
        s.vanish(stage, &precisely_7, FADE)?;
        s.appear(stage, &t.covered(), FADE)?;
        s.until(stage, "20_8", 1)?;
        let streak = s.text_right(stage, STREAKLINE, 17.0, dim, STREAK_STRIP_AT)?;
        s.appear(stage, &[streak], FADE)?;
        s.play(stage)?;
        s.pulse(stage, &[streak], None)?;
        s.until(stage, "20_8", 2)?;
        s.pulse(stage, &[streak], None)?;

        // 20_9: the table leaves; the tunnel and card return to the opening's places; the checklist.
        s.say(stage, "20_9")?;
        let mut table = t.titles.clone();
        table.extend(&t.rungs);
        table.push(t.legend);
        table.extend(&t.labels);
        table.extend(t.all_cells());
        table.push(t.footnote);
        table.extend(&t.target);
        table.extend(&t.tolerances);
        table.extend(&t.reference_title);
        table.extend(&t.reference);
        table.extend(&label);
        s.vanish(stage, &table, FADE)?;
        s.play(stage)?;
        let full = tunnel_drawing(stage, BENCH, TUNNEL_WIDTH, TUNNEL_CENTER)?;
        let (full_arrows, full_profile) = inflow_drawing(stage, TUNNEL_WIDTH, TUNNEL_CENTER, true)?;
        let card_full = s.question(stage, CARD_FULL)?;
        let test_full = s.text(stage, TEST_LABEL, 24.0, ink, TEST_FULL_AT)?;
        let streak_full = s.text_right(stage, STREAKLINE, 20.0, dim, STREAK_FULL_AT)?;
        s.morph(stage, &drawing, &full, MORPH)?;
        s.morph(stage, &uniform, &full_arrows, MORPH)?;
        s.morph(stage, &[profile], &[full_profile], MORPH)?;
        s.morph(stage, &question, &card_full, MORPH)?;
        s.morph(stage, &[test], &[test_full], MORPH)?;
        s.morph(stage, &[streak], &[streak_full], MORPH)?;
        s.until(stage, "20_9", 1)?;
        s.pulse(stage, &question, None)?;
        s.until(stage, "20_9", 2)?;
        for (i, words) in CHECKLIST.iter().enumerate() {
            let y = CHECKLIST_Y0 - CHECKLIST_PITCH * i as f64;
            let item = vec![
                rect(stage, [CHECKLIST_X + 0.12, y], [0.22, 0.22], (ink, 0.0), (ink, 1.4))?,
                s.text_left(stage, words, 24.0, ink, [CHECKLIST_X + 0.45, y])?,
            ];
            s.appear(stage, &item, FADE)?;
            s.play(stage)?;
            stage.wait(BEAT)?;
        }
        s.hold(stage)?;
        println!("Chapter 20 ends on the opening's tunnel, its question card and the checklist; no wake is drawn");
        s.k.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cues_run_in_order() {
        let ids: Vec<&str> = CUES.iter().map(|&(id, _)| id).collect();
        let expected: Vec<String> = (1..=9).map(|k| format!("20_{k}")).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        assert!(CUES.iter().all(|&(_, sentences)| sentences > 0));
    }

    #[test]
    fn labels_match_the_script() {
        let l = labels();
        assert_eq!(l.ref_strouhal, "0.302");
        assert_eq!(l.ref_drag, "3.23");
        assert_eq!(l.ref_lift, "0.987");
        assert_eq!(l.ref_delta_p, "2.485");
        assert_eq!(l.interval, ["0.99", "1.01"]);
        assert_eq!(l.ticks, ["0.98", "0.99", "1.00", "1.01", "1.02"]);
        assert_eq!(l.answer, "f = St × Ū / D ≈ 3.02 per second");
        assert_eq!(l.every, "one swing every 0.33 s");
        assert_eq!(l.cylinder, "D = 0.1 m at (0.2 m, 0.2 m)");
        assert_eq!(l.inflow, "prescribed inflow: peak 1.5 m/s, average 1 m/s");
    }

    #[test]
    fn facts_pin_the_oracle() {
        let f = facts().expect("every pin holds");
        assert_eq!(f.get("dfg.swings.per.s"), Some("3.01841"));
        assert_eq!(f.get("data.bytes.per.state"), Some("2.30912e+06"));
        assert_eq!(f.get("ref.lift.below.1996.range"), Some("True"));
    }

    #[test]
    fn g6_prints_like_python() {
        assert_eq!(g6(2_309_120.0), "2.30912e+06");
        assert_eq!(g6(0.000_012_5), "1.25e-05");
        assert_eq!(g6(100.000_000_000_000_01), "100");
        assert_eq!(g6(0.331_300_247_796_887_6), "0.3313");
        assert_eq!(g6(-3.5), "-3.5");
    }

    #[test]
    fn the_reference_is_the_spoken_one() {
        // 20_5: the 2014 lift peak lies just below the 1996 interval; St lies inside its own.
        const { assert!(REF_LIFT_PEAK < ST96_LOWER[1] && ST96_LOWER[1] - REF_LIFT_PEAK < 0.01) };
        const { assert!(ST96_LOWER[2] <= REF_STROUHAL && REF_STROUHAL <= ST96_UPPER[2]) };
        assert!((REF_PERIOD * swings_per_second(REF_STROUHAL) - 1.0).abs() < 1e-6);
        assert!((mean_inflow() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn without_a_ledger_every_entry_is_pending() {
        let ledger = Ledger::pending("no ledger given");
        for role in 0..5 {
            for q in 0..14 {
                assert_eq!(ledger.cell(role, q), None);
            }
        }
    }

    #[test]
    fn layout_keeps_its_parts_apart() {
        // Run columns and the close-enough column sit under the overlay card; the reference does not.
        let half = (COLUMN_X[1] - COLUMN_X[0]) / 2.0;
        let card = [
            OVERLAY_AT[0] - OVERLAY_SIZE[0] / 2.0,
            OVERLAY_AT[0] + OVERLAY_SIZE[0] / 2.0,
        ];
        assert!(card[0] <= COLUMN_X[0] - half + 0.01 && COLUMN_X[CLOSE] + half - 0.01 <= card[1]);
        assert!(COLUMN_X[REFERENCE] - text_width("0.987", 18.0) / 2.0 > card[1]);
        // The longest row label ends before the first column.
        let widest = ROW_LABELS.iter().map(|w| text_width(w, 18.0)).fold(0.0, f64::max);
        assert!(LABEL_X + widest < COLUMN_X[0] - half);
        // Rows end above the footnote and the footnote above the lower third.
        assert!(row_y(13) - 0.15 > FOOTNOTE_Y + 0.1 && FOOTNOTE_Y - 0.1 > LOWER_Y[0] + 0.1);
        // The strip clears the table's titles; the peak inflow arrow clears the cylinder.
        let strip_bottom = STRIP_CENTER[1] - STRIP_WIDTH / BENCH.length * BENCH.height / 2.0;
        assert!(strip_bottom > STREAK_STRIP_AT[1] + 0.1 && STREAK_STRIP_AT[1] - 0.1 > HEADER_Y + 0.1);
        let scale = TUNNEL_WIDTH / BENCH.length;
        let cylinder_left = tunnel_at(
            BENCH,
            TUNNEL_WIDTH,
            TUNNEL_CENTER,
            BENCH.cylinder[0] - BENCH.cylinder[2],
            0.0,
        )[0];
        let inlet = tunnel_at(BENCH, TUNNEL_WIDTH, TUNNEL_CENTER, 0.0, 0.0)[0];
        assert!(inlet + ARROW_PER_SPEED * PEAK_INFLOW < cylinder_left && scale > 5.0);
        // Every lower-third line fits the safe width.
        for page in [
            &PRECISELY_20_2A[..],
            &PRECISELY_20_2B,
            &PRECISELY_20_2C,
            &PRECISELY_20_2D,
        ] {
            assert!(
                page.iter().all(|w| text_width(w, 17.0) < 14.2),
                "a 20_2 page line is too wide"
            );
        }
        for page in [
            &PRECISELY_20_4[..],
            &PRECISELY_20_5,
            &PRECISELY_20_7,
            &[REFERENCE_LABEL],
        ] {
            assert!(
                page.iter().all(|w| text_width(w, 17.0) < 14.2),
                "a lower-third line is too wide"
            );
        }
    }
}
