//! Chapter 16, "Was the saving real?" (navier-full-script03.md, cues 16_1 to 16_10, hold 16_h1).
//!
//! The chapter opens on chapter 14's cell-budget readout, read from the film's registry (chapter 14's own
//! pinned `row2.*` facts), and dims it under "less work?". A strip of mixed cells shows one shared time step
//! set by the shortest crossing time (16_2). Three strips with matching smallest cells ask the prediction
//! 16_h1 (16_3); three clocks answer it (16_4). Cost tags attach to the adaptive strip (16_5). Run cards
//! and an open target card frame a fair comparison (16_6), and the side panel holds two tables (16_7): the
//! periodic shear comparison that has been run, filled from its receipts, and the cylinder comparison that
//! has not, every entry "?". 16_8 draws the standard test's channel inside the square a tree grid starts
//! from. 16_9 says what the shear test measured, line by line as the narrator says it: both errors inside
//! the target, the fluid-cell, stored-cell, clock and memory costs on separate lines, the tighter pressure
//! solve, and what the box cannot test; then the cylinder's open entries pulse. With the first line, a note
//! above it names the errors' denominator (the whole flow, drift included), and the measured table's title
//! band gives the same peaks against the shear alone. 16_10 turns the cylinder into a staircase of cells.
//!
//! Every number on screen is a pinned fact. The teaching strips are exact fractions (`four_cell::Q`); the
//! measured rows are computed from `RUNS`, a field-for-field copy of navier-solver-shear03/comparison.json
//! (sha256 1efa9b22…) that the package check compares with the file. `PINS` holds cost_oracle01's rows
//! verbatim, and `construct` compares every drawn string with its pin before the first frame. The two
//! shear-alone peaks are not in the registry: `SHEAR_ALONE_PEAKS` copies them from bronze-shear-relative01.
//!
//! Voice: the kit narrator, as in chapters 07 to 10. Each cue is `say`d before its pictures, and pictures
//! wait on its sentence starts (`until`). The hold follows chapter 08's: the card is up by the line's last
//! sentence, then `hold`, the "predict" mark and a countdown; the next line must be the answer (`say` refuses
//! any other), and nothing that answers it (the clocks, their counts, the viscosity tag, the measured step
//! counts) exists before that line begins.

use explainer_kit::{BACKGROUND, Facts, Kit, Palette, Timed, hex, rect, thousands};
use fmn::animation::{grow_arrow, indicate};
use fmn::prelude::*;

use super::chapter08::PREDICTION_SECONDS;
use crate::box_view::{self, p};
use crate::error;
use crate::four_cell::Q;

/// Chapter 16's cues in spoken order and the number of sentences in each (the narrator's gates).
/// Sentences end at `.`, `?` or `!` before a space, as in the kit; the check recounts them.
pub const CUES: [(&str, usize); 10] = [
    ("16_1", 4),
    ("16_2", 4),
    ("16_3", 4),
    ("16_4", 4),
    ("16_5", 4),
    ("16_6", 3),
    ("16_7", 4),
    ("16_8", 4),
    ("16_9", 6),
    ("16_10", 2),
];
/// The script's chapter heading, shown at the start as chapters 08 to 10 show theirs.
const TITLE: &str = "Was the saving real?";

/// A prediction hold (the script's holds table): the line it follows, the line that answers it, the card.
struct Hold {
    id: &'static str,
    after: &'static str,
    answer: &'static str,
    question: &'static str,
    choices: [&'static str; 3],
}

const HOLD: Hold = Hold {
    id: "16_h1",
    after: "16_3",
    answer: "16_4",
    question: "Same smallest cells, and the fastest air in them:\nwhich takes more time steps?",
    choices: ["uniform fine", "adaptive", "the same"],
};

// ------------------------------------------------------------------------------------------ numbers

/// The 16_2 strip: cell widths and the two air speeds (scene02 ad04's big and smallest cells among them).
const STRIP_WIDTHS: [(i64, i64); 5] = [(1, 1), (1, 1), (1, 2), (1, 4), (1, 4)];
const STRIP_FIRST: [(i64, i64); 5] = [(4, 1), (1, 1), (1, 1), (1, 2), (1, 2)];
const STRIP_SECOND: [(i64, i64); 5] = [(1, 1), (1, 1), (1, 1), (1, 2), (4, 1)];
/// The 16_3 runs: coarse, fine (one width, many cells) and adaptive, each covering one length.
const COARSE_WIDTHS: [(i64, i64); 4] = [(1, 1), (1, 1), (1, 1), (1, 1)];
const FINE_WIDTH: (i64, i64) = (1, 4);
const FINE_CELLS: usize = 16;
const ADAPTIVE_WIDTHS: [(i64, i64); 7] = [(1, 1), (1, 1), (1, 2), (1, 4), (1, 4), (1, 2), (1, 2)];

/// Schäfer and Turek (1996), quoted by scene02's reader (oracle-keys02 dfg.length, dfg.height.diameter).
const DFG_LENGTH: &str = "2.2";
const DFG_HEIGHT: &str = "0.41";
const DFG_DIAMETER: &str = "0.1";
/// The year of ST96 in scene03's source table.
const DFG_YEAR: u32 = 1996;
/// The cylinder's centre, 0.2 from the inlet and 0.2 above the bottom wall (scene03 native notes, ST96 Fig. 1).
const DFG_CENTRE: &str = "0.2";

/// One measured run, field for field as navier-solver-shear03/comparison.json records it.
struct Run {
    id: &'static str,
    steps: u32,
    cells: u32,
    allocated: u32,
    leaf_steps: u64,
    allocated_steps: u64,
    regrids: u32,
    peak_rel_l2: f64,
    peak_cfl: f64,
    peak_volume_defect: f64,
    rss_bytes: u64,
    wall_seconds: f64,
    dt_cap: f64,
    pressure_budget: f64,
    within: bool,
}

/// navier-solver-shear03/comparison.json (sha256 1efa9b2225685d63…), its seven runs in the plan's order.
static RUNS: [Run; 7] = [
    Run {
        id: "coarse",
        steps: 185,
        cells: 256,
        allocated: 365,
        leaf_steps: 47360,
        allocated_steps: 67525,
        regrids: 0,
        peak_rel_l2: 0.05973625229340614,
        peak_cfl: 0.020659123409365397,
        peak_volume_defect: 4.0371746350006114e-18,
        rss_bytes: 4145152,
        wall_seconds: 0.06,
        dt_cap: 0.0025,
        pressure_budget: 1e-09,
        within: false,
    },
    Run {
        id: "fine_a",
        steps: 185,
        cells: 16384,
        allocated: 21869,
        leaf_steps: 3031040,
        allocated_steps: 4045765,
        regrids: 0,
        peak_rel_l2: 0.0010274908967359754,
        peak_cfl: 0.18701682542772122,
        peak_volume_defect: 7.202875333215767e-12,
        rss_bytes: 26181632,
        wall_seconds: 1.07,
        dt_cap: 0.0025,
        pressure_budget: 1e-09,
        within: true,
    },
    Run {
        id: "adaptive_a",
        steps: 185,
        cells: 6304,
        allocated: 9325,
        leaf_steps: 1067168,
        allocated_steps: 1593029,
        regrids: 25,
        peak_rel_l2: 0.0012902148172008296,
        peak_cfl: 0.1870168243290688,
        peak_volume_defect: 6.500723192072069e-12,
        rss_bytes: 13565952,
        wall_seconds: 0.5,
        dt_cap: 0.0025,
        pressure_budget: 1e-09,
        within: true,
    },
    Run {
        id: "adaptive_b",
        steps: 185,
        cells: 6304,
        allocated: 9325,
        leaf_steps: 1067168,
        allocated_steps: 1593029,
        regrids: 25,
        peak_rel_l2: 0.0012902148172008296,
        peak_cfl: 0.1870168243290688,
        peak_volume_defect: 6.500723192072069e-12,
        rss_bytes: 15073280,
        wall_seconds: 0.5,
        dt_cap: 0.0025,
        pressure_budget: 1e-09,
        within: true,
    },
    Run {
        id: "fine_b",
        steps: 185,
        cells: 16384,
        allocated: 21869,
        leaf_steps: 3031040,
        allocated_steps: 4045765,
        regrids: 0,
        peak_rel_l2: 0.0010274908967359754,
        peak_cfl: 0.18701682542772122,
        peak_volume_defect: 7.202875333215767e-12,
        rss_bytes: 21659648,
        wall_seconds: 1.05,
        dt_cap: 0.0025,
        pressure_budget: 1e-09,
        within: true,
    },
    Run {
        id: "adaptive_timestep",
        steps: 342,
        cells: 6304,
        allocated: 9325,
        leaf_steps: 1978560,
        allocated_steps: 2952606,
        regrids: 27,
        peak_rel_l2: 0.0014078499915997967,
        peak_cfl: 0.09872159013592302,
        peak_volume_defect: 8.771117165473827e-13,
        rss_bytes: 16121856,
        wall_seconds: 0.94,
        dt_cap: 0.00125,
        pressure_budget: 1e-09,
        within: true,
    },
    Run {
        id: "adaptive_pressure",
        steps: 185,
        cells: 6304,
        allocated: 9325,
        leaf_steps: 1067168,
        allocated_steps: 1593029,
        regrids: 25,
        peak_rel_l2: 0.0012902154347677133,
        peak_cfl: 0.18701682401847708,
        peak_volume_defect: 7.322627352964061e-14,
        rss_bytes: 13467648,
        wall_seconds: 0.6,
        dt_cap: 0.0025,
        pressure_budget: 1e-11,
        within: true,
    },
];
/// The declared error target (shear-plan01.json, written before the runs; comparison.json "target").
const TARGET: f64 = 0.005;
/// The declared target's second part: the largest volume defect a step may leave (the same files).
const TARGET_VOLUME: f64 = 1.01e-9;
/// The solver's CFL setting (navier-solver-shear03/tunnel.c, `event init`: `CFL = .6;`).
const CFL_LIMIT: f64 = 0.6;
/// Fluid leaves at t = 0.4 in the two replays (navier-shear-replay01, frame-0000185.flow, every leaf with a
/// positive fluid fraction). The coarse run has no replay; it is the same periodic source with no solid.
const REPLAY_FLUID: [(&str, u32); 2] = [("fine", 16384), ("adaptive", 6304)];
/// The three runs a table column shows, with their repeats.
const SHOWN: [(&str, &[&str]); 3] = [
    ("coarse", &["coarse"]),
    ("fine", &["fine_a", "fine_b"]),
    ("adaptive", &["adaptive_a", "adaptive_b"]),
];

/// cost_oracle01's rows (runs/cost_oracle01-run02.txt), verbatim and in order, with their sources.
const PINS: [(&str, &str, &str); 66] = [
    (
        "ch16.strip.widths",
        "1, 1, 1/2, 1/4, 1/4",
        "cost_oracle01: the 16_2 strip",
    ),
    (
        "ch16.strip.speeds.first",
        "4, 1, 1, 1/2, 1/2",
        "cost_oracle01: the 16_2 strip",
    ),
    (
        "ch16.strip.times.first",
        "1/4, 1, 1/2, 1/2, 1/2",
        "cost_oracle01: width ÷ speed",
    ),
    ("ch16.strip.shortest.first", "1/4", "cost_oracle01: width ÷ speed"),
    ("ch16.strip.shortest.first.cell", "1", "cost_oracle01: width ÷ speed"),
    (
        "dt.min.crossing.big.cell",
        "true",
        "cost_oracle01; scene02 oracle-keys02 (ad04)",
    ),
    (
        "ch16.strip.speeds.second",
        "1, 1, 1, 1/2, 4",
        "cost_oracle01: the 16_2 strip",
    ),
    (
        "ch16.strip.times.second",
        "1, 1, 1/2, 1/2, 1/16",
        "cost_oracle01: width ÷ speed",
    ),
    ("ch16.strip.shortest.second", "1/16", "cost_oracle01: width ÷ speed"),
    ("ch16.strip.shortest.second.cell", "5", "cost_oracle01: width ÷ speed"),
    ("ch16.runs.length", "4", "cost_oracle01: the 16_3 strips"),
    (
        "ch16.runs.coarse.widths",
        "1, 1, 1, 1",
        "cost_oracle01: the 16_3 strips",
    ),
    ("ch16.runs.fine.width", "1/4", "cost_oracle01: the 16_3 strips"),
    ("ch16.runs.fine.cells", "16", "cost_oracle01: the 16_3 strips"),
    (
        "ch16.runs.adaptive.widths",
        "1, 1, 1/2, 1/4, 1/4, 1/2, 1/2",
        "cost_oracle01: the 16_3 strips",
    ),
    (
        "dt.ratio.quarter.cells",
        "1/4",
        "cost_oracle01; scene02 oracle-keys02 (ad03)",
    ),
    (
        "dt.fine.vs.coarse",
        "1/4",
        "cost_oracle01; scene02 oracle-keys02 (ad04)",
    ),
    ("dt.steps.ratio", "4", "cost_oracle01; scene02 oracle-keys02 (ad03)"),
    (
        "dt.steps.adaptive.vs.fine",
        "1",
        "cost_oracle01; scene02 oracle-keys02 (ad04)",
    ),
    ("ch16.clock.coarse", "1", "cost_oracle01: steps per coarse step"),
    ("ch16.clock.fine", "4", "cost_oracle01: steps per coarse step"),
    ("ch16.clock.adaptive", "4", "cost_oracle01: steps per coarse step"),
    (
        "dt.diff.fine.vs.coarse",
        "1/16",
        "cost_oracle01; scene02 oracle-keys02 (ad04)",
    ),
    (
        "dt.diff.adaptive.vs.fine",
        "1",
        "cost_oracle01; scene02 oracle-keys02 (ad04)",
    ),
    ("ch16.clock.viscous.fine", "16", "cost_oracle01: explicit viscous steps"),
    (
        "ch16.clock.viscous.adaptive",
        "16",
        "cost_oracle01: explicit viscous steps",
    ),
    ("chan.length", "2.2", "ST96 line 134 via oracle-keys02 dfg.length"),
    (
        "chan.height",
        "0.41",
        "ST96 line 142 via oracle-keys02 dfg.height.diameter",
    ),
    (
        "chan.diameter",
        "0.1",
        "ST96 line 142 via oracle-keys02 dfg.height.diameter",
    ),
    ("chan.year", "1996", "scene03 source table ST96"),
    ("chan.aspect", "220/41", "cost_oracle01; scene02 oracle-keys02 (ad04)"),
    (
        "chan.outside.square",
        "179/220",
        "cost_oracle01; scene02 oracle-keys02 (ad04)",
    ),
    ("ch16.show.outside", "4/5", "cost_oracle01: the nearest fifth"),
    ("shear.target", "0.5%", "shear-plan01.json target; cost_oracle01"),
    ("shear.error.coarse", "5.97%", "comparison.json coarse; cost_oracle01"),
    (
        "shear.error.fine",
        "0.103%",
        "comparison.json fine_a = fine_b; cost_oracle01",
    ),
    (
        "shear.error.adaptive",
        "0.129%",
        "comparison.json adaptive_a = adaptive_b; cost_oracle01",
    ),
    (
        "shear.pass",
        "false, true, true",
        "comparison.json within_error_target; cost_oracle01",
    ),
    (
        "shear.target.volume",
        "1.01e-9",
        "shear-plan01.json target; cost_oracle01",
    ),
    (
        "shear.volume.worst",
        "7.2e-12",
        "comparison.json peak_volume_defect; cost_oracle01",
    ),
    (
        "shear.cells.coarse",
        "256 / 256",
        "comparison.json cells; periodic solver; cost_oracle01",
    ),
    (
        "shear.cells.fine",
        "16,384 / 16,384",
        "comparison.json cells; replay fine frame 185; cost_oracle01",
    ),
    (
        "shear.cells.adaptive",
        "6,304 / 6,304",
        "comparison.json cells; replay adaptive frame 185; cost_oracle01",
    ),
    ("shear.steps", "185", "comparison.json steps; cost_oracle01"),
    ("shear.cap", "0.0025", "comparison.json dt_cap; cost_oracle01"),
    ("shear.cfl.peak", "0.19", "comparison.json peak_cfl; cost_oracle01"),
    ("shear.cfl.limit", "0.6", "tunnel.c CFL; cost_oracle01"),
    (
        "shear.half.cap.steps",
        "342",
        "comparison.json adaptive_timestep; cost_oracle01",
    ),
    (
        "shear.half.cap.error",
        "0.141%",
        "comparison.json adaptive_timestep; cost_oracle01",
    ),
    (
        "shear.clock.coarse",
        "0.06 s",
        "comparison.json process_wall_seconds; cost_oracle01",
    ),
    (
        "shear.clock.fine",
        "1.05 to 1.07 s",
        "comparison.json process_wall_seconds; cost_oracle01",
    ),
    (
        "shear.clock.adaptive",
        "0.50 s",
        "comparison.json process_wall_seconds; cost_oracle01",
    ),
    (
        "shear.memory.coarse",
        "4.1 MB",
        "comparison.json process_rss_bytes; cost_oracle01",
    ),
    (
        "shear.memory.fine",
        "21.7 to 26.2 MB",
        "comparison.json process_rss_bytes; cost_oracle01",
    ),
    (
        "shear.memory.adaptive",
        "13.6 to 15.1 MB",
        "comparison.json process_rss_bytes; cost_oracle01",
    ),
    ("shear.regrids", "25", "comparison.json regrids; cost_oracle01"),
    (
        "shear.stored.fine",
        "21,869 for 16,384",
        "comparison.json allocated_cells; cost_oracle01",
    ),
    (
        "shear.stored.adaptive",
        "9,325 for 6,304",
        "comparison.json allocated_cells; cost_oracle01",
    ),
    (
        "shear.share.leaf",
        "35%",
        "comparison.json adaptive_to_fixed_leaf_step_ratio; cost_oracle01",
    ),
    (
        "shear.share.stored",
        "39%",
        "comparison.json adaptive_to_fixed_allocated_step_ratio; cost_oracle01",
    ),
    (
        "shear.share.wall",
        "47%",
        "comparison.json adaptive_to_fixed_process_wall_ratio; cost_oracle01",
    ),
    (
        "shear.share.memory",
        "60%",
        "comparison.json adaptive_to_fixed_process_rss_ratio; cost_oracle01",
    ),
    (
        "shear.fewer.leaf",
        "64.8%",
        "comparison.json adaptive_to_fixed_leaf_step_ratio; cost_oracle01",
    ),
    (
        "shear.pressure.tighter",
        "100",
        "comparison.json pressure_budget; cost_oracle01",
    ),
    (
        "shear.pressure.tight.error",
        "0.129%",
        "comparison.json adaptive_pressure; cost_oracle01",
    ),
    (
        "shear.tolerance",
        "1e-9",
        "comparison.json pressure_budget; cost_oracle01",
    ),
];

fn q((num, den): (i64, i64)) -> Q {
    Q::new(num, den)
}

fn qs(pairs: &[(i64, i64)]) -> Vec<Q> {
    pairs.iter().copied().map(q).collect()
}

fn list(values: &[Q]) -> String {
    values.iter().map(Q::to_string).collect::<Vec<_>>().join(", ")
}

/// The one smallest value and its 0-based place; two equal smallest values are refused.
fn shortest(values: &[Q]) -> Result<(usize, Q), String> {
    let least = *values.iter().min().ok_or("chapter 16 strip has no cells")?;
    let places: Vec<usize> = (0..values.len()).filter(|&i| values[i] == least).collect();
    match places[..] {
        [one] => Ok((one, least)),
        _ => Err(format!("chapter 16 strip has {} shortest cells", places.len())),
    }
}

fn crossing(widths: &[Q], speeds: &[Q]) -> Vec<Q> {
    widths.iter().zip(speeds).map(|(&w, &u)| w / u).collect()
}

fn fine_widths() -> Vec<Q> {
    vec![q(FINE_WIDTH); FINE_CELLS]
}

/// "2.2" as the exact fraction 11/5.
fn parse_decimal(text: &str) -> Result<Q, String> {
    let (whole, part) = text
        .split_once('.')
        .ok_or_else(|| format!("{text} has no decimal point"))?;
    let digits: i64 = format!("{whole}{part}")
        .parse()
        .map_err(|_| format!("{text} is not a decimal"))?;
    let places = u32::try_from(part.len()).map_err(|_| format!("{text} is too long"))?;
    Ok(Q::new(digits, 10_i64.pow(places)))
}

/// A short terminating fraction printed as a decimal: 41/100 is "0.41".
fn decimal(value: Q) -> Result<String, String> {
    for places in 0..=6_u32 {
        if let Ok(n) = (value * Q::int(10_i64.pow(places))).to_string().parse::<i64>() {
            let unit = 10_i64.pow(places);
            return Ok(if places == 0 {
                n.to_string()
            } else {
                format!("{}.{:0width$}", n / unit, n % unit, width = places as usize)
            });
        }
    }
    Err(format!("{value} has no short decimal"))
}

fn channel() -> Result<[Q; 4], String> {
    Ok([
        parse_decimal(DFG_LENGTH)?,
        parse_decimal(DFG_HEIGHT)?,
        parse_decimal(DFG_DIAMETER)?,
        parse_decimal(DFG_CENTRE)?,
    ])
}

fn run(id: &str) -> Result<&'static Run, String> {
    RUNS.iter()
        .find(|r| r.id == id)
        .ok_or_else(|| format!("chapter 16 has no measured run {id}"))
}

fn runs_of(mesh: &str) -> Result<Vec<&'static Run>, String> {
    let (_, ids) = SHOWN
        .iter()
        .find(|(name, _)| *name == mesh)
        .ok_or_else(|| format!("chapter 16 shows no {mesh} column"))?;
    ids.iter().copied().map(run).collect()
}

fn mean(runs: &[&Run], field: fn(&Run) -> f64) -> f64 {
    runs.iter().map(|r| field(r)).sum::<f64>() / runs.len() as f64
}

/// Three significant figures of a fraction as a percentage: 0.0010275 is "0.103%".
fn sig3_percent(x: f64) -> String {
    let value = 100.0 * x;
    let places = (2 - value.log10().floor() as i32).max(0) as usize;
    format!("{value:.places$}%")
}

/// One value, or the low and high of repeats, to `places` decimals with a unit.
fn span(values: &[f64], places: usize, unit: &str) -> String {
    let low = values.iter().copied().fold(f64::INFINITY, f64::min);
    let high = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let (a, b) = (format!("{low:.places$}"), format!("{high:.places$}"));
    if a == b {
        format!("{a} {unit}")
    } else {
        format!("{a} to {b} {unit}")
    }
}

/// Every fact the chapter draws or speaks, computed, in cost_oracle01's order.
fn fact_rows() -> Result<Vec<(String, String)>, String> {
    let mut rows: Vec<(String, String)> = Vec::new();
    let mut add = |key: &str, value: String| rows.push((key.to_string(), value));

    let widths = qs(&STRIP_WIDTHS);
    let first = crossing(&widths, &qs(&STRIP_FIRST));
    let second = crossing(&widths, &qs(&STRIP_SECOND));
    let (first_cell, first_time) = shortest(&first)?;
    let (second_cell, second_time) = shortest(&second)?;
    let biggest = *widths.iter().max().ok_or("chapter 16 strip has no cells")?;
    add("ch16.strip.widths", list(&widths));
    add("ch16.strip.speeds.first", list(&qs(&STRIP_FIRST)));
    add("ch16.strip.times.first", list(&first));
    add("ch16.strip.shortest.first", first_time.to_string());
    add("ch16.strip.shortest.first.cell", (first_cell + 1).to_string());
    add("dt.min.crossing.big.cell", (widths[first_cell] == biggest).to_string());
    add("ch16.strip.speeds.second", list(&qs(&STRIP_SECOND)));
    add("ch16.strip.times.second", list(&second));
    add("ch16.strip.shortest.second", second_time.to_string());
    add("ch16.strip.shortest.second.cell", (second_cell + 1).to_string());

    let coarse = qs(&COARSE_WIDTHS);
    let fine = fine_widths();
    let adaptive = qs(&ADAPTIVE_WIDTHS);
    let total = |cells: &[Q]| cells.iter().fold(Q::ZERO, |a, &b| a + b);
    if total(&coarse) != total(&fine) || total(&fine) != total(&adaptive) {
        return Err("chapter 16's three strips cover different lengths".into());
    }
    let least = least_width;
    if least(&adaptive)? != least(&fine)? {
        return Err("chapter 16's adaptive strip does not reach the fine width".into());
    }
    add("ch16.runs.length", total(&coarse).to_string());
    add("ch16.runs.coarse.widths", list(&coarse));
    add("ch16.runs.fine.width", q(FINE_WIDTH).to_string());
    add("ch16.runs.fine.cells", fine.len().to_string());
    add("ch16.runs.adaptive.widths", list(&adaptive));
    let steps = [least(&coarse)?, least(&fine)?, least(&adaptive)?];
    add("dt.ratio.quarter.cells", (steps[1] / steps[0]).to_string());
    add("dt.fine.vs.coarse", (steps[1] / steps[0]).to_string());
    let clocks = steps.map(|s| steps[0] / s);
    add("dt.steps.ratio", clocks[1].to_string());
    add("dt.steps.adaptive.vs.fine", (clocks[2] / clocks[1]).to_string());
    for (name, clock) in ["coarse", "fine", "adaptive"].iter().zip(clocks) {
        add(&format!("ch16.clock.{name}"), clock.to_string());
    }
    let viscous = steps.map(|s| s * s);
    add("dt.diff.fine.vs.coarse", (viscous[1] / viscous[0]).to_string());
    add("dt.diff.adaptive.vs.fine", (viscous[2] / viscous[1]).to_string());
    add("ch16.clock.viscous.fine", (viscous[0] / viscous[1]).to_string());
    add("ch16.clock.viscous.adaptive", (viscous[0] / viscous[2]).to_string());

    let [length, height, diameter, _] = channel()?;
    add("chan.length", decimal(length)?);
    add("chan.height", decimal(height)?);
    add("chan.diameter", decimal(diameter)?);
    add("chan.year", DFG_YEAR.to_string());
    add("chan.aspect", (length / height).to_string());
    let outside = Q::ONE - height / length;
    add("chan.outside.square", outside.to_string());
    add(
        "ch16.show.outside",
        Q::new((outside * Q::int(5)).to_f64().round() as i64, 5).to_string(),
    );

    let columns = [runs_of("coarse")?, runs_of("fine")?, runs_of("adaptive")?];
    add("shear.target", format!("{:.1}%", 100.0 * TARGET));
    for (name, runs) in ["coarse", "fine", "adaptive"].iter().zip(&columns) {
        add(&format!("shear.error.{name}"), sig3_percent(runs[0].peak_rel_l2));
    }
    let mut passes = Vec::new();
    for runs in &columns {
        let pass = runs[0].peak_rel_l2 <= TARGET;
        if runs
            .iter()
            .any(|r| r.within != pass || r.peak_rel_l2 != runs[0].peak_rel_l2)
        {
            return Err(format!(
                "chapter 16: {} disagrees with its error target or repeat",
                runs[0].id
            ));
        }
        passes.push(pass.to_string());
    }
    add("shear.pass", passes.join(", "));
    if RUNS.iter().any(|r| r.peak_volume_defect > TARGET_VOLUME) {
        return Err("chapter 16: a measured run misses the declared volume target".into());
    }
    add("shear.target.volume", format!("{TARGET_VOLUME:e}"));
    let worst = columns
        .iter()
        .flatten()
        .map(|r| r.peak_volume_defect)
        .fold(0.0, f64::max);
    add("shear.volume.worst", format!("{worst:.1e}"));
    for (name, runs) in ["coarse", "fine", "adaptive"].iter().zip(&columns) {
        let all = runs[0].cells;
        let fluid = match REPLAY_FLUID.iter().find(|(mesh, _)| mesh == name) {
            Some(&(_, wet)) => wet,
            // No replay of the coarse run: its source has no solid, so every leaf is fluid.
            None if *name == "coarse" => all,
            None => return Err(format!("chapter 16 has no replay fluid count for the {name} run")),
        };
        add(
            &format!("shear.cells.{name}"),
            format!("{} / {}", thousands(fluid), thousands(all)),
        );
    }
    let every: Vec<&Run> = columns.iter().flatten().copied().collect();
    if every
        .iter()
        .any(|r| r.steps != every[0].steps || r.dt_cap != every[0].dt_cap)
    {
        return Err("chapter 16's three runs differ in steps or step cap".into());
    }
    add("shear.steps", every[0].steps.to_string());
    add("shear.cap", every[0].dt_cap.to_string());
    let peak = every.iter().map(|r| r.peak_cfl).fold(0.0, f64::max);
    add("shear.cfl.peak", format!("{peak:.2}"));
    add("shear.cfl.limit", CFL_LIMIT.to_string());
    let half = run("adaptive_timestep")?;
    if half.dt_cap * 2.0 != columns[2][0].dt_cap {
        return Err("chapter 16's time-step control is not half the cap".into());
    }
    add("shear.half.cap.steps", half.steps.to_string());
    add("shear.half.cap.error", sig3_percent(half.peak_rel_l2));
    for (name, runs) in ["coarse", "fine", "adaptive"].iter().zip(&columns) {
        let seconds: Vec<f64> = runs.iter().map(|r| r.wall_seconds).collect();
        add(&format!("shear.clock.{name}"), span(&seconds, 2, "s"));
    }
    for (name, runs) in ["coarse", "fine", "adaptive"].iter().zip(&columns) {
        let memory: Vec<f64> = runs.iter().map(|r| r.rss_bytes as f64 / 1e6).collect();
        add(&format!("shear.memory.{name}"), span(&memory, 1, "MB"));
    }
    if columns[0].iter().chain(&columns[1]).any(|r| r.regrids != 0) {
        return Err("chapter 16's uniform runs regrid".into());
    }
    add("shear.regrids", columns[2][0].regrids.to_string());
    for (name, runs) in [("fine", &columns[1]), ("adaptive", &columns[2])] {
        add(
            &format!("shear.stored.{name}"),
            format!("{} for {}", thousands(runs[0].allocated), thousands(runs[0].cells)),
        );
    }
    let share = |field: fn(&Run) -> f64| mean(&columns[2], field) / mean(&columns[1], field);
    let leaf = share(|r| r.leaf_steps as f64);
    add("shear.share.leaf", format!("{:.0}%", 100.0 * leaf));
    add(
        "shear.share.stored",
        format!("{:.0}%", 100.0 * share(|r| r.allocated_steps as f64)),
    );
    add("shear.share.wall", format!("{:.0}%", 100.0 * share(|r| r.wall_seconds)));
    add(
        "shear.share.memory",
        format!("{:.0}%", 100.0 * share(|r| r.rss_bytes as f64)),
    );
    add("shear.fewer.leaf", format!("{:.1}%", 100.0 * (1.0 - leaf)));
    let tight = run("adaptive_pressure")?;
    let budget = columns[2][0].pressure_budget;
    add(
        "shear.pressure.tighter",
        format!("{:.0}", budget / tight.pressure_budget),
    );
    add("shear.pressure.tight.error", sig3_percent(tight.peak_rel_l2));
    add("shear.tolerance", format!("{budget:e}"));
    Ok(rows)
}

/// Chapter 16's numbers alone, checked (a preview's `facts`).
pub fn facts() -> Result<Facts, String> {
    let mut facts = Facts::new();
    register(&mut facts)?;
    facts.self_check()?;
    Ok(facts)
}

/// Add chapter 16's numbers to a film's registry, each pinned as `PINS` says.
pub fn register(facts: &mut Facts) -> Result<(), String> {
    let rows = fact_rows()?;
    let keys: Vec<&str> = rows.iter().map(|(key, _)| key.as_str()).collect();
    if keys != PINS.map(|(key, _, _)| key) {
        return Err(format!(
            "chapter 16 pins {:?}, but computes {keys:?}",
            PINS.map(|(key, _, _)| key)
        ));
    }
    for ((key, value), (_, expected, source)) in rows.iter().zip(PINS) {
        facts.pin(key, value, expected, source)?;
    }
    Ok(())
}

pub struct Chapter16 {
    pub kit: Kit,
}

// Layout, scene units: the camera frame is 16 × 9 (x within ±8, y within ±4.5). The main window is
// x ∈ [−7.1, 0.2] and the side panel x ∈ [0.4, 6.95] (OP3).
const TITLE_Y: f64 = 3.45;
/// Chapter 14's readout as 14_9 leaves it (white-adapt-decision01 chapter14.rs: SIDE_X 3.65, y 1.65, 24).
const BUDGET_AT: [f64; 2] = [3.65, 1.65];
const BUDGET_SIZE: f64 = 24.0;
const LESS_WORK_Y: f64 = 1.1;
const PANEL_X: f64 = 3.7;
const NOTE_SIZE: f64 = 18.0;
const LABEL_SIZE: f64 = 22.0;

/// 16_2: one strip, two scene units per unit of width; arrows above, crossing times below.
const STRIP_LEFT: f64 = -6.8;
const STRIP_UNIT: f64 = 2.0;
const STRIP_Y: f64 = 0.6;
const STRIP_H: f64 = 0.9;
const ARROW_Y: f64 = 1.35;
const ARROW_PER_SPEED: f64 = 0.35;
const ARROW_INSET: f64 = 0.08;
const TIME_Y: f64 = -0.2;
const STRIP_HEADER_Y: f64 = 2.4;
const CLOCK_AT: [f64; 2] = [2.0, 0.6];
const CLOCK_R: f64 = 0.45;
const CLOCK_RULE_AT: [f64; 2] = [4.75, 0.6];
const CLOCK_NOTE_Y: f64 = -0.2;
const TICK: f64 = std::f64::consts::FRAC_PI_3;

/// 16_3 to 16_5: three strips of one length, 1.35 scene units per unit.
const RUNS_LEFT: f64 = -6.9;
const RUNS_UNIT: f64 = 1.35;
const RUNS_Y: [f64; 3] = [1.9, 0.7, -0.5];
const RUNS_H: f64 = 0.55;
const RUNS_NAMES: [&str; 3] = ["uniform coarse", "uniform fine", "adaptive"];
/// 16_3's change of picture, each way: 16_2's picture fades out, then the runs fade in where it stood. The two
/// plays take the frames of the one 0.5 s crossfade they replace (fmn plays ceil(seconds × fps) frames: 15 + 15
/// of 30 at the film's 60 fps), so every later frame is where it was.
const SWAP_SECONDS: f64 = 0.25;
/// 16_3's condition card, the script's four items: two with sentence 0, then one each with sentences 1 and 2.
const CONDITIONS: [&str; 4] = [
    "same flow",
    "same simulated time",
    "speed sets one shared step",
    "fastest air in the smallest cells",
];
const CARD_AT: [f64; 2] = [3.7, 2.55];
const CARD_W: f64 = 6.2;
const CARD_H: f64 = 1.6;
const CARD_GAP: f64 = 0.34;
const ASK_Y: f64 = -0.3;
const ASK_W: f64 = 6.2;
const ASK_H: f64 = 2.5;
const QUESTION_SIZE: f64 = 24.0;
const CHOICE_SIZE: f64 = 22.0;
const CHOICE_GAP: f64 = 0.5;
const CLOCKS_X: f64 = -0.95;
const COUNT_X: f64 = -0.25;
const SMALL_CLOCK_R: f64 = 0.24;
const STEADY_Y: f64 = -1.3;
const EXPLICIT_AT: [f64; 2] = [-3.6, -1.8];
/// The measured block in the side panel (16_4, 16_5), one line every `MEASURED_GAP`.
const MEASURED_TOP: f64 = 1.35;
const MEASURED_GAP: f64 = 0.32;
const TAGS: [&str; 3] = ["pressure", "regrid", "bookkeeping"];
const TAG_NOTES: [&str; 3] = ["whole box, every step", "split, move values", "which cell is where"];
const TAG_X: [f64; 3] = [-6.0, -4.2, -2.4];
const TAG_Y: f64 = -1.45;
const TAG_W: f64 = 1.65;
const TAG_H: f64 = 0.45;
const TAG_NOTE_Y: f64 = -1.95;

/// 16_6: run cards and the open target card in the main window; the shear's matched settings at the side.
const RUN_CARD_X: [f64; 3] = [-6.0, -3.45, -0.9];
const RUN_CARD_Y: f64 = 1.25;
const RUN_CARD_W: f64 = 2.1;
const RUN_CARD_H: f64 = 1.9;
const RUN_SETTINGS: [&str; 4] = ["same computer", "same physics", "same walls", "same solver tolerance"];
const TARGET_AT: [f64; 2] = [-3.45, -1.0];
const TARGET_W: f64 = 6.6;
const TARGET_H: f64 = 0.9;
const GROUP_Y: f64 = 2.6;
/// 16_6's side panel: the shear run's matched settings, then its declared target, one line every `SIDE_GAP`.
const MATCHED_TOP: f64 = 2.6;
const DECLARED_TOP: f64 = 1.0;
const SIDE_GAP: f64 = 0.35;
const SIDE_SIZE: f64 = 19.0;

/// 16_7: the two tables in the side panel, label column at the left, three value columns.
const TABLE_LEFT: f64 = 0.55;
const TABLE_COLUMNS: [f64; 3] = [3.2, 4.65, 6.1];
const TABLE_FRAME: [f64; 2] = [0.45, 6.9];
const ROW_GAP: f64 = 0.32;
const SHEAR_TITLE_Y: f64 = 3.6;
const CYLINDER_TITLE_Y: f64 = 0.35;
const HEAD_DROP: f64 = 0.37;
const ROW_DROP: f64 = 0.72;
const TABLE_SIZE: f64 = 17.0;
const TITLE_SIZE: f64 = 21.0;
const ROW_LABELS: [&str; 7] = [
    "error (target {})",
    "cells, fluid / all",
    "time steps",
    "clock time",
    "memory",
    "pressure share",
    "regrid kick",
];
/// The rows each sentence of 16_7 names: 1 (error, cells, steps), 2 (clock, memory, pressure), 3 (kick).
const ROWS_BY_SENTENCE: [(usize, std::ops::Range<usize>); 3] = [(1, 0..3), (2, 3..6), (3, 6..7)];

/// 16_8: the standard test's channel at the bottom of the 2.2 square, 4.2 scene units a side.
const SQUARE_LEFT: f64 = -5.6;
const SQUARE_BOTTOM: f64 = -0.9;
const SQUARE_SIDE: f64 = 4.2;
const HATCH_GAP: f64 = 0.3;
const GRID_COLUMNS: usize = 22;
const GRID_ROWS: usize = 4;
/// 16_9: the verdict, one line per sentence, centred in the square's place above the channel.
const VERDICT_Y: [f64; 5] = [2.45, 2.03, 1.61, 1.19, 0.77];
const VERDICT_X: f64 = -3.45;
/// 16_9: the declared error divides by the whole exact velocity, whose steady drift (0.3 and 0.35) is about
/// 96% of its square at t = 0.4. One verdict step above the first line, the note says so.
const WHOLE_FLOW_NOTE: &str = "errors relative to the whole flow, steady drift included";
const WHOLE_FLOW_Y: f64 = 2.87;
/// The same peak errors divided by the shear alone (the exact flow less its drift), fine_a then adaptive_a,
/// both at t = 0.400 s: bronze-shear-relative01 (README 017cd3b7…, runs/evaluate01.txt 34fd1e3b…), and
/// white-shear-variation01 agrees (0.51745% and 0.64975%). They carry no target: the 0.5% belongs to the
/// whole-flow row. They sit right-aligned in the measured table's title band, as far inside the frame's
/// right edge as the row labels are inside its left.
const SHEAR_ALONE_PEAKS: [(&str, f64); 2] = [("fine", 0.5175), ("adaptive", 0.6498)];
const SHEAR_ALONE_RIGHT: f64 = TABLE_FRAME[1] - (TABLE_LEFT - TABLE_FRAME[0]);

/// 16_10: the cylinder, magnified, and its staircase of cells.
const BIG_AT: [f64; 2] = [-3.5, 1.4];
const BIG_R: f64 = 1.3;
const STAIR_CELLS: i32 = 5;

/// The chapter's state: what is on screen, a pending batch, the line being spoken and the answer due.
struct Ch16<'a> {
    k: &'a Kit,
    on: Vec<Mob>,
    pending: Vec<Box<dyn Animation>>,
    line: &'static str,
    answer_due: Option<&'static str>,
}

impl Ch16<'_> {
    // Text through the kit, so the layout lint and the glyph check see every string.
    fn text(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        self.k.text(stage, s, size, color, at)
    }

    fn text_left(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        self.k.text_left(stage, s, size, color, at)
    }

    fn put(&mut self, stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
        for &m in mobs {
            if !self.on.contains(&m) {
                stage.add_to_scene(m)?;
                self.on.push(m);
            }
        }
        Ok(())
    }

    /// Remove at once (no fade): the countdown digits and anything replaced in place.
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

    /// Fade out everything on screen except `keep`.
    fn clear_except(&mut self, stage: &mut Stage<'_>, keep: &[Mob], secs: f64) -> fmn::Result<()> {
        let gone: Vec<Mob> = self.on.iter().copied().filter(|m| !keep.contains(m)).collect();
        self.vanish(stage, &gone, secs)
    }

    /// `from` (on screen) takes `to`'s shape and paint; `to` stays detached.
    fn morph(&mut self, stage: &mut Stage<'_>, from: Mob, to: Mob, secs: f64) -> fmn::Result<()> {
        if self.on.contains(&from) {
            self.pending.push(stage.prepare(Transform::new(from, to).rt(secs))?);
        }
        Ok(())
    }

    fn grow(&mut self, stage: &mut Stage<'_>, arrow: Mob, secs: f64) -> fmn::Result<()> {
        if !self.on.contains(&arrow) {
            let a = grow_arrow(stage.arena_mut(), arrow)?.rt(secs);
            self.pending.push(stage.prepare(a)?);
            self.on.push(arrow);
        }
        Ok(())
    }

    fn pulse(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], secs: f64) -> fmn::Result<()> {
        for &m in mobs {
            if self.on.contains(&m) {
                let a = indicate(stage.arena_mut(), m, 1.15, None)?.rt(secs);
                self.pending.push(stage.prepare(a)?);
            }
        }
        Ok(())
    }

    fn refill(&mut self, stage: &mut Stage<'_>, m: Mob, color: Srgb, secs: f64) -> fmn::Result<()> {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(secs),
                ..AnimateArgs::default()
            })?
            .set_fill(Some(color), Some(1.0))?;
        self.pending.push(stage.prepare(a)?);
        Ok(())
    }

    fn play(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        if !self.pending.is_empty() {
            stage.play_prepared(std::mem::take(&mut self.pending))?;
        }
        Ok(())
    }

    /// Finish what is queued, then start line `id` (the kit waits for the previous line first). After the
    /// countdown only the hold's answer may follow.
    fn say(&mut self, stage: &mut Stage<'_>, id: &'static str) -> fmn::Result<()> {
        if !CUES.iter().any(|c| c.0 == id) {
            return Err(error(format!("chapter 16 has no cue {id}")));
        }
        if let Some(due) = self.answer_due.take()
            && due != id
        {
            return Err(error(format!("chapter 16 answers its hold with {due}, not {id}")));
        }
        self.play(stage)?;
        println!("Chapter 16 cue {id} at {:.6} s", stage.scene().time().to_f64());
        self.line = id;
        self.k.say(stage, id)
    }

    /// Finish what is queued, then wait for sentence `j` of line `id` (nothing if it has begun).
    fn until(&mut self, stage: &mut Stage<'_>, id: &str, j: usize) -> fmn::Result<()> {
        let sentences = CUES.iter().find(|c| c.0 == id).map_or(0, |c| c.1);
        if j >= sentences {
            return Err(error(format!("chapter 16 line {id} has no sentence {j}")));
        }
        self.play(stage)?;
        self.k.until_sentence(stage, id, j)
    }

    /// The hold's card in the side panel with its question; the line must be the one the hold follows.
    fn ask(&mut self, stage: &mut Stage<'_>, hold: &Hold) -> fmn::Result<Vec<Mob>> {
        if self.line != hold.after {
            return Err(error(format!(
                "chapter 16 asks {} during {}, not {}",
                hold.id, self.line, hold.after
            )));
        }
        let frame = card(stage, [PANEL_X, ASK_Y], ASK_W, ASK_H)?;
        let question = self.text(
            stage,
            hold.question,
            QUESTION_SIZE,
            Palette::ink(),
            p(PANEL_X, ASK_Y + 0.6),
        )?;
        let shown = vec![frame, question];
        self.appear(stage, &shown, 0.5)?;
        Ok(shown)
    }

    /// The three choices in one row across the card, one after another (as chapter 08 offers its own).
    fn offer(&mut self, stage: &mut Stage<'_>, hold: &Hold, card: &mut Vec<Mob>) -> fmn::Result<()> {
        let y = ASK_Y - 0.35;
        let mut choices = Vec::new();
        for words in hold.choices {
            choices.push(self.text(stage, words, CHOICE_SIZE, Palette::dim(), p(PANEL_X, y))?);
        }
        let widths: Vec<f64> = choices.iter().map(|&m| stage.get_width(m)).collect();
        let total = widths.iter().sum::<f64>() + CHOICE_GAP * (widths.len() - 1) as f64;
        let mut left = PANEL_X - total / 2.0;
        for (&m, &width) in choices.iter().zip(&widths) {
            stage.move_to(m, p(left + width / 2.0, y), ORIGIN);
            left += width + CHOICE_GAP;
            self.appear(stage, &[m], 0.35)?;
            self.play(stage)?;
            stage.wait(0.15)?;
        }
        card.extend(choices);
        Ok(())
    }

    /// Chapter 08's hold: the line finishes, the "predict" mark, whole seconds counting down (added and
    /// removed). The card stays until the answer line has begun, and that line must be the hold's answer.
    fn countdown(&mut self, stage: &mut Stage<'_>, hold: &Hold) -> fmn::Result<()> {
        self.play(stage)?;
        self.k.hold(stage)?;
        self.k.mark(stage, "predict");
        println!(
            "Chapter 16 hold {} after {} at {:.6} s",
            hold.id,
            hold.after,
            stage.scene().time().to_f64()
        );
        for second in (1..=PREDICTION_SECONDS).rev() {
            let clock = self.text(
                stage,
                &second.to_string(),
                34.0,
                Palette::hero(),
                p(PANEL_X, ASK_Y - 0.85),
            )?;
            self.put(stage, &[clock])?;
            stage.wait(1.0)?;
            self.cut(stage, &[clock]);
        }
        self.answer_due = Some(hold.answer);
        Ok(())
    }
}

fn plain_rect(stage: &mut Stage<'_>, at: Vec3, w: f64, h: f64, rim: Srgb, width: f64) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Rectangle::new()
            .width(w)
            .height(h)
            .style(Style::default().fill(rim, 0.0).stroke(rim, width, 1.0))
            .build()?
            .moved_to(at),
    ))
}

fn card(stage: &mut Stage<'_>, at: [f64; 2], w: f64, h: f64) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Rectangle::new()
            .width(w)
            .height(h)
            .corner_radius(0.12)
            .style(
                Style::default()
                    .fill(hex("#182433"), 1.0)
                    .stroke(Palette::dim(), 1.4, 1.0),
            )
            .build()?
            .moved_to(p(at[0], at[1])),
    ))
}

fn circle(stage: &mut Stage<'_>, at: [f64; 2], radius: f64, color: Srgb, width: f64) -> Mob {
    stage.arena_mut().add(
        Circle::new()
            .radius(radius)
            .style(Style::default().fill(color, 0.0).stroke(color, width, 1.0))
            .build()
            .moved_to(p(at[0], at[1])),
    )
}

/// Cells of `widths` side by side from `left`, `unit` scene units per unit of width: the outlines and the
/// left and right edge of each.
fn strip(
    stage: &mut Stage<'_>,
    widths: &[Q],
    left: f64,
    unit: f64,
    y: f64,
    h: f64,
) -> fmn::Result<(Vec<Mob>, Vec<[f64; 2]>)> {
    let mut x = left;
    let mut cells = Vec::with_capacity(widths.len());
    let mut edges = Vec::with_capacity(widths.len());
    for w in widths {
        let width = w.to_f64() * unit;
        cells.push(plain_rect(stage, p(x + width / 2.0, y), width, h, Palette::ink(), 1.6)?);
        edges.push([x, x + width]);
        x += width;
    }
    Ok((cells, edges))
}

/// The air in each cell of the 16_2 strip: a blue arrow from the cell's left edge, `ARROW_PER_SPEED` long
/// per unit of speed.
fn speed_arrows(stage: &mut Stage<'_>, edges: &[[f64; 2]], speeds: &[Q]) -> fmn::Result<Vec<Mob>> {
    edges
        .iter()
        .zip(speeds)
        .map(|(edge, u)| {
            let start = edge[0] + ARROW_INSET;
            box_view::arrow(
                stage,
                p(start, ARROW_Y),
                p(start + u.to_f64() * ARROW_PER_SPEED, ARROW_Y),
                box_view::flow_color(),
                3.0,
            )
        })
        .collect()
}

/// A clock hand from `at`, `turns` ticks clockwise from twelve.
fn hand(stage: &mut Stage<'_>, at: [f64; 2], radius: f64, turns: f64) -> fmn::Result<Mob> {
    let angle = turns * TICK;
    let tip = p(at[0] + 0.85 * radius * angle.sin(), at[1] + 0.85 * radius * angle.cos());
    box_view::line(stage, p(at[0], at[1]), tip, Palette::ink(), 3.0)
}

/// Diagonal hatch lines filling the rectangle [x0, x1] × [y0, y1], `HATCH_GAP` apart.
fn hatch(stage: &mut Stage<'_>, [x0, x1]: [f64; 2], [y0, y1]: [f64; 2]) -> fmn::Result<Vec<Mob>> {
    let mut lines = Vec::new();
    let mut c = x0 - (y1 - y0) + HATCH_GAP;
    while c < x1 {
        // The line x − y = c − y0 clipped to the rectangle.
        let (mut a, mut b) = ([c, y0], [c + (y1 - y0), y1]);
        if a[0] < x0 {
            a = [x0, y0 + (x0 - c)];
        }
        if b[0] > x1 {
            b = [x1, y1 - (b[0] - x1)];
        }
        lines.push(box_view::line(
            stage,
            p(a[0], a[1]),
            p(b[0], b[1]),
            Palette::faint(),
            1.2,
        )?);
        c += HATCH_GAP;
    }
    Ok(lines)
}

impl SceneConstruct for Chapter16 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        // Every refusal comes before the first frame.
        let replay = super::shear_display::retained()?;
        let shown = facts().map_err(error)?;
        let film = super::facts().map_err(error)?;
        let budget = |key: &str| {
            film.get(key).map(str::to_string).ok_or_else(|| {
                error(format!(
                    "chapter 16 opens on chapter 14's readout: the film registers no {key} (wire chapter 14 first)"
                ))
            })
        };
        let readout = format!(
            "cells {} (halve all twice: {})",
            budget("row2.cells.after")?,
            budget("row2.cells.all.halved.twice")?
        );
        // The render log carries the fact table, so each picture's numbers can be read against it.
        for line in shown.print().lines() {
            println!("Chapter 16 fact {line}");
        }
        let fact = |key: &str| {
            shown
                .get(key)
                .map(str::to_string)
                .ok_or_else(|| error(format!("chapter 16 has no fact {key}")))
        };
        // Texts the pictures compute from the strips must read exactly as their pinned facts.
        let agree = |key: &str, drawn: String| -> fmn::Result<()> {
            let pinned = fact(key)?;
            if drawn == pinned {
                Ok(())
            } else {
                Err(error(format!("chapter 16 draws {drawn:?} where {key} is {pinned:?}")))
            }
        };
        let widths = qs(&STRIP_WIDTHS);
        let (first_speeds, second_speeds) = (qs(&STRIP_FIRST), qs(&STRIP_SECOND));
        let first = crossing(&widths, &first_speeds);
        let second = crossing(&widths, &second_speeds);
        agree("ch16.strip.times.first", list(&first))?;
        agree("ch16.strip.times.second", list(&second))?;
        let (first_cell, _) = shortest(&first).map_err(error)?;
        let (second_cell, _) = shortest(&second).map_err(error)?;
        let strips = [qs(&COARSE_WIDTHS), fine_widths(), qs(&ADAPTIVE_WIDTHS)];
        agree("ch16.runs.adaptive.widths", list(&strips[2]))?;
        let [length, height, diameter, centre] = channel().map_err(error)?;
        agree("chan.outside.square", (Q::ONE - height / length).to_string())?;

        let ink = Palette::ink();
        let dim = Palette::dim();
        let hero = Palette::hero();
        let red = Palette::red();
        let violet = Palette::purple();
        let mut s = Ch16 {
            k: &self.kit,
            on: Vec::new(),
            pending: Vec::new(),
            line: "",
            answer_due: None,
        };

        // ---------------------------------------------------------------- 16_1 the honest question
        s.say(stage, "16_1")?;
        let title = s.text(stage, TITLE, 45.0, ink, p(0.0, TITLE_Y))?;
        s.appear(stage, &[title], 0.6)?;
        let held = s.text(stage, &readout, BUDGET_SIZE, ink, p(BUDGET_AT[0], BUDGET_AT[1]))?;
        s.put(stage, &[held])?;
        s.play(stage)?;
        stage.wait(0.5)?;
        s.vanish(stage, &[title], 0.4)?;
        s.until(stage, "16_1", 2)?;
        s.pulse(stage, &[held], 0.8)?;
        s.until(stage, "16_1", 3)?;
        s.refill(stage, held, dim, 0.5)?;
        let less = s.text(stage, "less work?", 26.0, hero, p(BUDGET_AT[0], LESS_WORK_Y))?;
        s.appear(stage, &[less], 0.5)?;

        // ---------------------------------------------------------------- 16_2 one shared step
        s.say(stage, "16_2")?;
        s.vanish(stage, &[held, less], 0.5)?;
        let header = s.text(
            stage,
            "crossing time = width ÷ speed",
            24.0,
            ink,
            p(-3.8, STRIP_HEADER_Y),
        )?;
        let (cells, edges) = strip(stage, &widths, STRIP_LEFT, STRIP_UNIT, STRIP_Y, STRIP_H)?;
        s.appear(stage, &[header], 0.5)?;
        s.appear(stage, &cells, 0.5)?;
        s.play(stage)?;
        let arrows = speed_arrows(stage, &edges, &first_speeds)?;
        for &a in &arrows {
            s.grow(stage, a, 0.6)?;
        }
        let time_label = |s: &Ch16<'_>, stage: &mut Stage<'_>, times: &[Q]| -> fmn::Result<Vec<Mob>> {
            edges
                .iter()
                .zip(times)
                .map(|(e, t)| s.text(stage, &t.to_string(), LABEL_SIZE, ink, p((e[0] + e[1]) / 2.0, TIME_Y)))
                .collect()
        };
        let times = time_label(&s, stage, &first)?;
        s.appear(stage, &times, 0.5)?;
        s.until(stage, "16_2", 1)?;
        let face = circle(stage, CLOCK_AT, CLOCK_R, ink, 2.4);
        let needle = hand(stage, CLOCK_AT, CLOCK_R, 0.0)?;
        let one_clock = s.text(
            stage,
            "one step for every cell",
            NOTE_SIZE,
            dim,
            p(CLOCK_AT[0], CLOCK_NOTE_Y),
        )?;
        s.appear(stage, &[face, needle, one_clock], 0.5)?;
        s.until(stage, "16_2", 2)?;
        let outline_at = |cell: usize| {
            let [x0, x1] = edges[cell];
            ((x0 + x1) / 2.0, x1 - x0)
        };
        let (mid, wide) = outline_at(first_cell);
        let outline = plain_rect(stage, p(mid, STRIP_Y), wide, STRIP_H, ink, 4.0)?;
        let rule = s.text(
            stage,
            &format!("step = CFL × {}", fact("ch16.strip.shortest.first")?),
            26.0,
            ink,
            p(CLOCK_RULE_AT[0], CLOCK_RULE_AT[1]),
        )?;
        s.appear(stage, &[outline, rule], 0.5)?;
        s.play(stage)?;
        for turn in 1..=2 {
            let next = hand(stage, CLOCK_AT, CLOCK_R, f64::from(turn))?;
            s.morph(stage, needle, next, 0.4)?;
            s.play(stage)?;
        }
        s.until(stage, "16_2", 3)?;
        let faster = speed_arrows(stage, &edges, &second_speeds)?;
        for (&from, &to) in arrows.iter().zip(&faster) {
            s.morph(stage, from, to, 0.7)?;
        }
        let times_after = time_label(&s, stage, &second)?;
        s.vanish(stage, &times, 0.35)?;
        s.appear(stage, &times_after, 0.35)?;
        let (mid, wide) = outline_at(second_cell);
        let moved = plain_rect(stage, p(mid, STRIP_Y), wide, STRIP_H, ink, 4.0)?;
        s.morph(stage, outline, moved, 0.7)?;
        let rule_after = s.text(
            stage,
            &format!("step = CFL × {}", fact("ch16.strip.shortest.second")?),
            26.0,
            ink,
            p(CLOCK_RULE_AT[0], CLOCK_RULE_AT[1]),
        )?;
        s.morph(stage, rule, rule_after, 0.6)?;
        s.play(stage)?;
        // Short steps: the same clock now ticks four times as often.
        for turn in 3..=10 {
            let next = hand(stage, CLOCK_AT, CLOCK_R, f64::from(turn))?;
            s.morph(stage, needle, next, 0.12)?;
            s.play(stage)?;
        }

        // ---------------------------------------------------------------- 16_3 three runs, the hold
        s.say(stage, "16_3")?;
        // The rows take the old picture's place, so it leaves before they arrive: no frame shows both.
        s.clear_except(stage, &[], SWAP_SECONDS)?;
        s.play(stage)?;
        let mut rows = Vec::new();
        let mut runs_edges = Vec::new();
        for ((widths, &y), name) in strips.iter().zip(&RUNS_Y).zip(RUNS_NAMES) {
            let (mut cells, edges) = strip(stage, widths, RUNS_LEFT, RUNS_UNIT, y, RUNS_H)?;
            let label = s.text_left(stage, name, LABEL_SIZE, dim, p(RUNS_LEFT, y + RUNS_H / 2.0 + 0.25))?;
            cells.push(label);
            s.appear(stage, &cells, SWAP_SECONDS)?;
            rows.push(cells);
            runs_edges.push(edges);
        }
        let conditions = card(stage, CARD_AT, CARD_W, CARD_H)?;
        let condition_lines: Vec<Mob> = CONDITIONS
            .iter()
            .enumerate()
            .map(|(i, words)| {
                s.text(
                    stage,
                    words,
                    LABEL_SIZE,
                    ink,
                    p(CARD_AT[0], CARD_AT[1] + 1.5 * CARD_GAP - CARD_GAP * i as f64),
                )
            })
            .collect::<fmn::Result<_>>()?;
        s.appear(
            stage,
            &[conditions, condition_lines[0], condition_lines[1]],
            SWAP_SECONDS,
        )?;
        s.until(stage, "16_3", 1)?;
        s.appear(stage, &[condition_lines[2]], 0.4)?;
        s.until(stage, "16_3", 2)?;
        // The adaptive run's smallest cells, the same stretch of the flow in every run, and the fastest air
        // through it. The fine run's cells there are the same width; the coarse run has none that small.
        let smallest = least_width(&strips[2]).map_err(error)?;
        let reach: Vec<[f64; 2]> = runs_edges[2]
            .iter()
            .zip(&strips[2])
            .filter(|(_, w)| **w == smallest)
            .map(|(e, _)| *e)
            .collect();
        let (Some(start), Some(end)) = (reach.first(), reach.last()) else {
            return Err(error("chapter 16's adaptive strip has no smallest cell".into()));
        };
        let (from, to) = (start[0], end[1]);
        let mut marks = Vec::new();
        let mut gusts = Vec::new();
        for (edges, (&y, widths)) in runs_edges.iter().zip(RUNS_Y.iter().zip(&strips)) {
            for (e, w) in edges.iter().zip(widths) {
                if *w == smallest && e[0] >= from - 1e-9 && e[1] <= to + 1e-9 {
                    marks.push(
                        stage.arena_mut().add(rect(
                            p(e[0], y - RUNS_H / 2.0),
                            e[1] - e[0],
                            RUNS_H,
                            Style::default()
                                .fill(box_view::flow_color(), 0.25)
                                .stroke(ink, 0.0, 1.0),
                        )?),
                    );
                }
            }
            gusts.push(box_view::arrow(
                stage,
                p(from - 0.3, y + RUNS_H / 2.0 + 0.12),
                p(to + 0.3, y + RUNS_H / 2.0 + 0.12),
                box_view::flow_color(),
                3.0,
            )?);
        }
        s.appear(stage, &marks, 0.4)?;
        for &g in &gusts {
            s.grow(stage, g, 0.6)?;
        }
        s.appear(stage, &[condition_lines[3]], 0.4)?;
        s.until(stage, "16_3", 3)?;
        let mut ask = s.ask(stage, &HOLD)?;
        s.play(stage)?;
        s.offer(stage, &HOLD, &mut ask)?;
        s.countdown(stage, &HOLD)?;

        // ---------------------------------------------------------------- 16_4 the answer, measured steps
        s.say(stage, "16_4")?;
        s.vanish(stage, &ask, 0.4)?;
        let counts = [
            fact("ch16.clock.coarse")?,
            fact("ch16.clock.fine")?,
            fact("ch16.clock.adaptive")?,
        ];
        let faces: Vec<Mob> = RUNS_Y
            .iter()
            .map(|&y| circle(stage, [CLOCKS_X, y], SMALL_CLOCK_R, ink, 2.0))
            .collect();
        s.appear(stage, &faces, 0.4)?;
        s.play(stage)?;
        // Each clock counts its run's steps over one stretch of simulated time: the fine and adaptive
        // clocks tick through theirs while the coarse run takes its one step, and all land together.
        let totals: Vec<u32> = counts
            .iter()
            .map(|c| {
                c.parse::<u32>()
                    .map_err(|_| error(format!("chapter 16 counts {c:?} steps")))
            })
            .collect::<fmn::Result<_>>()?;
        let longest = totals
            .iter()
            .copied()
            .max()
            .ok_or_else(|| error("chapter 16 has no clocks".into()))?;
        let mut count_mobs: Vec<Option<Mob>> = vec![None; RUNS_Y.len()];
        for tick in 1..=longest {
            for (row, &total) in totals.iter().enumerate() {
                // A run with fewer steps shows each of them when the same simulated time has passed.
                if u64::from(tick) * u64::from(total) % u64::from(longest) != 0 {
                    continue;
                }
                let reached = u64::from(tick) * u64::from(total) / u64::from(longest);
                let digit = s.text(stage, &reached.to_string(), 30.0, hero, p(COUNT_X, RUNS_Y[row]))?;
                if let Some(old) = count_mobs[row] {
                    s.cut(stage, &[old]);
                }
                s.put(stage, &[digit])?;
                count_mobs[row] = Some(digit);
            }
            stage.wait(0.35)?;
        }
        for (shown_count, count) in count_mobs.iter().zip(&counts) {
            if shown_count.is_none() {
                return Err(error(format!("chapter 16's clock never reached {count}")));
            }
        }
        let steady = s.text(
            stage,
            "steady stepping: no cap, start-up not counted",
            NOTE_SIZE,
            dim,
            p(-3.75, STEADY_Y),
        )?;
        s.appear(stage, &[steady], 0.4)?;
        s.until(stage, "16_4", 1)?;
        let measured_lines = [
            (
                format!(
                    "measured, a periodic shear flow: {} steps in every run",
                    fact("shear.steps")?
                ),
                ink,
            ),
            (
                format!("a step cap of {} set them, not the air:", fact("shear.cap")?),
                dim,
            ),
            (
                format!(
                    "at most {} of a cell per step (limit {})",
                    fact("shear.cfl.peak")?,
                    fact("shear.cfl.limit")?
                ),
                dim,
            ),
            (
                format!(
                    "half the cap: {} steps, error {}",
                    fact("shear.half.cap.steps")?,
                    fact("shear.half.cap.error")?
                ),
                dim,
            ),
        ];
        let mut measured = Vec::new();
        for (i, (words, color)) in measured_lines.iter().enumerate() {
            let at = p(PANEL_X, MEASURED_TOP - MEASURED_GAP * i as f64);
            measured.push(s.text(stage, words, NOTE_SIZE, *color, at)?);
        }
        s.appear(stage, &measured, 0.5)?;
        s.until(stage, "16_4", 2)?;
        let explicit = s.text(
            stage,
            "explicit step",
            LABEL_SIZE,
            violet,
            p(EXPLICIT_AT[0], EXPLICIT_AT[1]),
        )?;
        let tag_w = stage.get_width(explicit) + 0.4;
        let explicit_frame = box_view::frame(stage, p(EXPLICIT_AT[0], EXPLICIT_AT[1]), tag_w, 0.45, violet, 2.0)?;
        s.appear(stage, &[explicit, explicit_frame], 0.5)?;
        s.until(stage, "16_4", 3)?;
        for (row, key) in [(1, "ch16.clock.viscous.fine"), (2, "ch16.clock.viscous.adaptive")] {
            let sixteen = s.text(stage, &fact(key)?, 30.0, violet, p(COUNT_X, RUNS_Y[row]))?;
            if let Some(count) = count_mobs[row] {
                s.morph(stage, count, sixteen, 0.6)?;
            }
        }

        // ---------------------------------------------------------------- 16_5 other costs
        s.say(stage, "16_5")?;
        let mut stay: Vec<Mob> = rows.iter().flatten().copied().collect();
        stay.extend(&measured);
        s.clear_except(stage, &stay, 0.5)?;
        let mut tags = Vec::new();
        for ((&x, word), note) in TAG_X.iter().zip(TAGS).zip(TAG_NOTES) {
            let frame = box_view::frame(stage, p(x, TAG_Y), TAG_W, TAG_H, ink, 1.6)?;
            let word = s.text(stage, word, LABEL_SIZE, ink, p(x, TAG_Y))?;
            let tie = box_view::line(
                stage,
                p(x, TAG_Y + TAG_H / 2.0),
                p(x, RUNS_Y[2] - RUNS_H / 2.0),
                dim,
                1.4,
            )?;
            let note = s.text(stage, note, NOTE_SIZE - 1.0, dim, p(x, TAG_NOTE_Y))?;
            tags.push([frame, word, tie, note]);
        }
        // The measured counterpart of each cost, under the step counts (the third takes two lines).
        let cost_lines = [
            vec!["pressure's share of the time: not measured".to_string()],
            vec![format!("adaptive run: {} regrids", fact("shear.regrids")?)],
            vec![
                "cells kept, leaves and their parents:".to_string(),
                format!(
                    "fine {}; adaptive {}",
                    fact("shear.stored.fine")?,
                    fact("shear.stored.adaptive")?
                ),
            ],
        ];
        let mut line_no = measured_lines.len() + 1;
        let mut costs = Vec::new();
        for lines in &cost_lines {
            let mut group = Vec::new();
            for words in lines {
                let at = p(PANEL_X, MEASURED_TOP - MEASURED_GAP * line_no as f64);
                group.push(s.text(stage, words, NOTE_SIZE - 1.0, dim, at)?);
                line_no += 1;
            }
            costs.push(group);
        }
        for (k, (tag, cost)) in tags.iter().zip(&costs).enumerate() {
            s.until(stage, "16_5", k + 1)?;
            s.appear(stage, tag, 0.45)?;
            s.appear(stage, cost, 0.45)?;
        }

        // ---------------------------------------------------------------- 16_6 a fair comparison
        s.say(stage, "16_6")?;
        s.clear_except(stage, &[], 0.5)?;
        let group = s.text(
            stage,
            "the cylinder comparison, still to run",
            20.0,
            dim,
            p(-3.45, GROUP_Y),
        )?;
        s.appear(stage, &[group], 0.5)?;
        s.until(stage, "16_6", 1)?;
        let mut run_cards = Vec::new();
        for (&x, name) in RUN_CARD_X.iter().zip(RUNS_NAMES) {
            let frame = card(stage, [x, RUN_CARD_Y], RUN_CARD_W, RUN_CARD_H)?;
            let title = s.text(stage, name, LABEL_SIZE, ink, p(x, RUN_CARD_Y + 0.6))?;
            run_cards.extend([frame, title]);
            for (i, setting) in RUN_SETTINGS.iter().enumerate() {
                let at = p(x, RUN_CARD_Y + 0.2 - 0.3 * i as f64);
                run_cards.push(s.text(stage, setting, NOTE_SIZE - 2.0, dim, at)?);
            }
        }
        s.appear(stage, &run_cards, 0.5)?;
        let matched = [
            ("the shear comparison, already run:".to_string(), ink),
            ("same host, same exact flow, periodic box".to_string(), dim),
            (
                format!(
                    "pressure tolerance {}, step cap {}",
                    fact("shear.tolerance")?,
                    fact("shear.cap")?
                ),
                dim,
            ),
            (
                format!(
                    "{}× tighter tolerance: adaptive error still {}",
                    fact("shear.pressure.tighter")?,
                    fact("shear.pressure.tight.error")?
                ),
                dim,
            ),
        ];
        let mut side = Vec::new();
        for (i, (words, color)) in matched.iter().enumerate() {
            let at = p(PANEL_X, MATCHED_TOP - SIDE_GAP * i as f64);
            side.push(s.text(stage, words, SIDE_SIZE, *color, at)?);
        }
        s.appear(stage, &side, 0.5)?;
        s.until(stage, "16_6", 2)?;
        let target_card = card(stage, TARGET_AT, TARGET_W, TARGET_H)?;
        let answer = s.text(
            stage,
            "measured answer: ?",
            LABEL_SIZE,
            ink,
            p(TARGET_AT[0] - TARGET_W / 4.0, TARGET_AT[1]),
        )?;
        let close = s.text(
            stage,
            "close enough: ?",
            LABEL_SIZE,
            ink,
            p(TARGET_AT[0] + TARGET_W / 4.0, TARGET_AT[1]),
        )?;
        s.appear(stage, &[target_card, answer, close], 0.5)?;
        let declared = [
            ("declared before its runs:".to_string(), ink),
            (
                "worst sampled relative velocity error vs the exact flow".to_string(),
                dim,
            ),
            (format!("close enough: at most {}", fact("shear.target")?), dim),
            (
                format!(
                    "volume defect per step at most {}: worst {}",
                    fact("shear.target.volume")?,
                    fact("shear.volume.worst")?
                ),
                dim,
            ),
        ];
        let mut target_side = Vec::new();
        for (i, (words, color)) in declared.iter().enumerate() {
            let at = p(PANEL_X, DECLARED_TOP - SIDE_GAP * i as f64);
            target_side.push(s.text(stage, words, SIDE_SIZE, *color, at)?);
        }
        s.appear(stage, &target_side, 0.5)?;

        // ---------------------------------------------------- 16_7 the saved fields and their measurements
        s.say(stage, "16_7")?;
        s.clear_except(stage, &[], 0.4)?;
        s.play(stage)?;
        let target = fact("shear.target")?;
        let mut row_names: Vec<String> = ROW_LABELS.iter().map(|label| label.replace("{}", &target)).collect();
        let shear_values = shear_table(&fact)?;
        let shear_table = table(&s, stage, SHEAR_TITLE_Y, "periodic shear: measured", &row_names)?;
        row_names[0] = "error target pending".to_owned();
        let cylinder_table = table(
            &s,
            stage,
            CYLINDER_TITLE_Y,
            "cylinder: matched comparison pending",
            &row_names,
        )?;
        let mut tables = [shear_table, cylinder_table];
        for t in &tables {
            s.appear(stage, &t.frame, 0.5)?;
            for row in &t.values {
                s.appear(stage, row, 0.5)?;
            }
        }
        s.play(stage)?;
        // At 30 and 60 fps each of the 17 saved observations holds for 11/30 seconds.
        // This shows the same physical times before the second spoken sentence.
        let replay_clock = super::shear_display::ShearClock {
            dwell: 0.36,
            ..super::shear_display::ShearClock::DEFAULT
        };
        let shear = super::shear_display::play(&self.kit, stage, replay, replay_clock)?;
        for (sentence, range) in ROWS_BY_SENTENCE {
            s.until(stage, "16_7", sentence)?;
            for r in range {
                for t in &tables {
                    s.refill(stage, t.labels[r], ink, 0.4)?;
                }
                for (c, (words, misses)) in shear_values[r].iter().enumerate() {
                    let color = if *misses { red } else { ink };
                    let at = p(TABLE_COLUMNS[c], row_y(SHEAR_TITLE_Y, r));
                    let value = s.text(stage, words, TABLE_SIZE, color, at)?;
                    s.cut(stage, &[tables[0].values[r][c]]);
                    s.appear(stage, &[value], 0.4)?;
                    tables[0].values[r][c] = value;
                }
            }
        }

        // ---------------------------------------------------------------- 16_8 the box
        s.say(stage, "16_8")?;
        shear.leave(stage, 0.5)?;
        let keep: Vec<Mob> = tables.iter().flat_map(Table::all).collect();
        s.clear_except(stage, &keep, 0.5)?;
        let scale = SQUARE_SIDE / length.to_f64();
        let at = |x: Q, y: Q| p(SQUARE_LEFT + scale * x.to_f64(), SQUARE_BOTTOM + scale * y.to_f64());
        let top = SQUARE_BOTTOM + scale * height.to_f64();
        let channel_box = plain_rect(
            stage,
            p(SQUARE_LEFT + SQUARE_SIDE / 2.0, (SQUARE_BOTTOM + top) / 2.0),
            SQUARE_SIDE,
            top - SQUARE_BOTTOM,
            ink,
            2.4,
        )?;
        let cylinder = stage.arena_mut().add(
            Circle::new()
                .radius(scale * diameter.to_f64() / 2.0)
                .style(Style::default().fill(hex(BACKGROUND), 1.0).stroke(ink, 2.0, 1.0))
                .build()
                .moved_to(at(centre, centre)),
        );
        s.appear(stage, &[channel_box, cylinder], 0.6)?;
        s.until(stage, "16_8", 1)?;
        let length_label = s.text(
            stage,
            &fact("chan.length")?,
            LABEL_SIZE,
            ink,
            p(SQUARE_LEFT + SQUARE_SIDE / 2.0, SQUARE_BOTTOM - 0.25),
        )?;
        let height_label = s.text(
            stage,
            &fact("chan.height")?,
            LABEL_SIZE,
            ink,
            p(SQUARE_LEFT - 0.35, (SQUARE_BOTTOM + top) / 2.0),
        )?;
        let test = s.text(
            stage,
            &format!("standard test ({})", fact("chan.year")?),
            LABEL_SIZE,
            dim,
            p(SQUARE_LEFT + SQUARE_SIDE / 2.0, SQUARE_BOTTOM - 0.6),
        )?;
        s.appear(stage, &[length_label, height_label, test], 0.5)?;
        s.until(stage, "16_8", 2)?;
        let square = plain_rect(
            stage,
            p(SQUARE_LEFT + SQUARE_SIDE / 2.0, SQUARE_BOTTOM + SQUARE_SIDE / 2.0),
            SQUARE_SIDE,
            SQUARE_SIDE,
            dim,
            2.0,
        )?;
        s.appear(stage, &[square], 0.6)?;
        s.play(stage)?;
        let hatching = hatch(
            stage,
            [SQUARE_LEFT, SQUARE_LEFT + SQUARE_SIDE],
            [top, SQUARE_BOTTOM + SQUARE_SIDE],
        )?;
        s.appear(stage, &hatching, 0.6)?;
        let plate_y = (top + SQUARE_BOTTOM + SQUARE_SIDE) / 2.0;
        let outside = s.text(
            stage,
            &format!("≈ {} outside", fact("ch16.show.outside")?),
            28.0,
            ink,
            p(SQUARE_LEFT + SQUARE_SIDE / 2.0, plate_y),
        )?;
        let plate = card(
            stage,
            [SQUARE_LEFT + SQUARE_SIDE / 2.0, plate_y],
            stage.get_width(outside) + 0.4,
            0.6,
        )?;
        s.appear(stage, &[plate, outside], 0.5)?;
        s.until(stage, "16_8", 3)?;
        let mut grid = Vec::new();
        let cell_w = SQUARE_SIDE / GRID_COLUMNS as f64;
        let cell_h = (top - SQUARE_BOTTOM) / GRID_ROWS as f64;
        for i in 1..GRID_COLUMNS {
            let x = SQUARE_LEFT + cell_w * i as f64;
            grid.push(box_view::line(stage, p(x, SQUARE_BOTTOM), p(x, top), dim, 1.0)?);
        }
        for j in 1..GRID_ROWS {
            let y = SQUARE_BOTTOM + cell_h * j as f64;
            grid.push(box_view::line(
                stage,
                p(SQUARE_LEFT, y),
                p(SQUARE_LEFT + SQUARE_SIDE, y),
                dim,
                1.0,
            )?);
        }
        let fair = s.text(
            stage,
            "fair baseline: a uniform grid over the channel alone",
            NOTE_SIZE,
            dim,
            p(SQUARE_LEFT + SQUARE_SIDE / 2.0, SQUARE_BOTTOM - 0.95),
        )?;
        s.appear(stage, &grid, 0.6)?;
        s.appear(stage, &[fair], 0.5)?;

        // ---------------------------------------------------------------- 16_9 what the shear test says
        s.say(stage, "16_9")?;
        // The square has made its point: the verdict takes its place above the channel, whose cylinder 16_10
        // lights up. Each line waits for the sentence that says it; the costs stay on separate lines.
        let mut square_parts = vec![square, plate, outside, fair];
        square_parts.extend(&hatching);
        square_parts.extend(&grid);
        s.vanish(stage, &square_parts, 0.4)?;
        let verdict = [
            format!(
                "shear test: fine {}, adaptive {}, both inside the {} target",
                fact("shear.error.fine")?,
                fact("shear.error.adaptive")?,
                fact("shear.target")?
            ),
            format!(
                "fluid-cell updates: adaptive {} of fine, {} fewer; allocated-cell updates {}",
                fact("shear.share.leaf")?,
                fact("shear.fewer.leaf")?,
                fact("shear.share.stored")?
            ),
            format!(
                "clock on this computer: adaptive {}, fine {} ({}); memory {}",
                fact("shear.clock.adaptive")?,
                fact("shear.clock.fine")?,
                fact("shear.share.wall")?,
                fact("shear.share.memory")?
            ),
            format!(
                "pressure imbalance asked {}× smaller: adaptive error still {}",
                fact("shear.pressure.tighter")?,
                fact("shear.pressure.tight.error")?
            ),
            "rounding: not tested; walls and wake: not in this box".to_string(),
        ];
        // The denominator note and the shear-alone readout fade in with the first line, in its play. The
        // readout belongs to the measured table and stays with it.
        let whole_flow = s.text(stage, WHOLE_FLOW_NOTE, NOTE_SIZE, dim, p(VERDICT_X, WHOLE_FLOW_Y))?;
        let band = p(SHEAR_ALONE_RIGHT, SHEAR_TITLE_Y);
        let shear_alone = s.text(stage, &shear_alone_note(), TABLE_SIZE, dim, band)?;
        stage.move_to(shear_alone, band, RIGHT);
        tables[0].frame.push(shear_alone);
        for (k, (words, &y)) in verdict.iter().zip(&VERDICT_Y).enumerate() {
            if k > 0 {
                s.until(stage, "16_9", k)?;
            }
            let line = s.text(stage, words, NOTE_SIZE, ink, p(VERDICT_X, y))?;
            if k == 0 {
                s.appear(stage, &[line, whole_flow, shear_alone], 0.5)?;
            } else {
                s.appear(stage, &[line], 0.5)?;
            }
        }
        s.until(stage, "16_9", VERDICT_Y.len())?;
        let open: Vec<Mob> = tables[1].values.iter().flatten().copied().collect();
        s.pulse(stage, &open, 0.8)?;

        // ---------------------------------------------------------------- 16_10 the cylinder's shape
        s.say(stage, "16_10")?;
        // The channel's cylinder lights up in place, then grows into the middle of the main window.
        let small = stage.arena_mut().add(
            Circle::new()
                .radius(scale * diameter.to_f64() / 2.0)
                .style(Style::default().fill(ink, 0.0).stroke(ink, 4.0, 1.0))
                .build()
                .moved_to(at(centre, centre)),
        );
        s.put(stage, &[small])?;
        let stays: Vec<Mob> = tables.iter().flat_map(Table::all).chain([small]).collect();
        s.clear_except(stage, &stays, 0.5)?;
        s.play(stage)?;
        let glow = circle(stage, BIG_AT, BIG_R, ink, 4.0);
        s.morph(stage, small, glow, 0.8)?;
        s.play(stage)?;
        s.pulse(stage, &[small], 0.6)?;
        s.play(stage)?;
        let mut stairs = Vec::new();
        let cell = BIG_R / f64::from(STAIR_CELLS);
        for i in -STAIR_CELLS..STAIR_CELLS {
            for j in -STAIR_CELLS..STAIR_CELLS {
                let (cx, cy) = ((f64::from(i) + 0.5) * cell, (f64::from(j) + 0.5) * cell);
                if cx.hypot(cy) < BIG_R {
                    stairs.push(stage.arena_mut().add(rect(
                        p(BIG_AT[0] + cx - cell / 2.0, BIG_AT[1] + cy - cell / 2.0),
                        cell,
                        cell,
                        Style::default().fill(ink, 0.25).stroke(dim, 1.0, 1.0),
                    )?));
                }
            }
        }
        s.appear(stage, &stairs, 0.8)?;
        s.until(stage, "16_10", 1)?;
        s.vanish(stage, &[small], 0.6)?;
        s.play(stage)?;
        println!("Chapter 16 ends on the two tables and the cylinder's staircase of cells");
        s.k.finish(stage)
    }
}

/// The smallest width in a strip.
fn least_width(widths: &[Q]) -> Result<Q, String> {
    widths
        .iter()
        .copied()
        .min()
        .ok_or_else(|| "chapter 16 strip has no cells".to_string())
}

/// The y of table row `r` under a title at `title_y`.
fn row_y(title_y: f64, r: usize) -> f64 {
    title_y - ROW_DROP - ROW_GAP * r as f64
}

/// One side-panel table: its frame, title and column heads; the row labels (faint until named); and a "?"
/// in every value cell.
struct Table {
    frame: Vec<Mob>,
    labels: Vec<Mob>,
    values: Vec<Vec<Mob>>,
}

impl Table {
    fn all(&self) -> Vec<Mob> {
        let mut all = self.frame.clone();
        all.extend(self.values.iter().flatten());
        all
    }
}

fn table(s: &Ch16<'_>, stage: &mut Stage<'_>, title_y: f64, title: &str, row_names: &[String]) -> fmn::Result<Table> {
    let bottom = row_y(title_y, row_names.len() - 1) - 0.22;
    let top = title_y + 0.22;
    let outline = plain_rect(
        stage,
        p((TABLE_FRAME[0] + TABLE_FRAME[1]) / 2.0, (top + bottom) / 2.0),
        TABLE_FRAME[1] - TABLE_FRAME[0],
        top - bottom,
        Palette::dim(),
        1.4,
    )?;
    let mut frame = vec![
        outline,
        s.text_left(stage, title, TITLE_SIZE, Palette::ink(), p(TABLE_LEFT, title_y))?,
    ];
    for (&x, name) in TABLE_COLUMNS.iter().zip(["coarse", "fine", "adaptive"]) {
        frame.push(s.text(stage, name, TABLE_SIZE, Palette::dim(), p(x, title_y - HEAD_DROP))?);
    }
    let mut labels = Vec::new();
    let mut values = Vec::new();
    for (r, name) in row_names.iter().enumerate() {
        let y = row_y(title_y, r);
        let label = s.text_left(stage, name, TABLE_SIZE, Palette::faint(), p(TABLE_LEFT, y))?;
        frame.push(label);
        labels.push(label);
        let open = if r == 1 { "? / ?" } else { "?" };
        values.push(
            TABLE_COLUMNS
                .iter()
                .map(|&x| s.text(stage, open, TABLE_SIZE, Palette::dim(), p(x, y)))
                .collect::<fmn::Result<Vec<Mob>>>()?,
        );
    }
    Ok(Table { frame, labels, values })
}

/// The measured table's title-band readout: the peak errors against the shear alone, to two decimals.
fn shear_alone_note() -> String {
    let [(fine, fine_peak), (adaptive, adaptive_peak)] = SHEAR_ALONE_PEAKS;
    format!("vs the shear alone: {fine} {fine_peak:.2}%, {adaptive} {adaptive_peak:.2}%")
}

/// The measured table's cells, row by row (coarse, fine, adaptive), each with "misses the error target".
fn shear_table(fact: &dyn Fn(&str) -> fmn::Result<String>) -> fmn::Result<Vec<[(String, bool); 3]>> {
    let pass: Vec<bool> = fact("shear.pass")?.split(", ").map(|v| v == "true").collect();
    let per = |prefix: &str| -> fmn::Result<[(String, bool); 3]> {
        Ok([
            (fact(&format!("{prefix}.coarse"))?, false),
            (fact(&format!("{prefix}.fine"))?, false),
            (fact(&format!("{prefix}.adaptive"))?, false),
        ])
    };
    let mut errors = per("shear.error")?;
    for (cell, ok) in errors.iter_mut().zip(&pass) {
        cell.1 = !ok;
    }
    let steps = fact("shear.steps")?;
    let unmeasured = || ("not timed".to_string(), false);
    let none = || ("no regrids".to_string(), false);
    Ok(vec![
        errors,
        per("shear.cells")?,
        [(steps.clone(), false), (steps.clone(), false), (steps, false)],
        per("shear.clock")?,
        per("shear.memory")?,
        [unmeasured(), unmeasured(), unmeasured()],
        [
            none(),
            none(),
            (format!("{} regrids; ?", fact("shear.regrids")?), false),
        ],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cues_hold_and_pins_line_up() {
        let ids: Vec<&str> = CUES.iter().map(|c| c.0).collect();
        let after = ids
            .iter()
            .position(|&id| id == HOLD.after)
            .expect("the hold follows a cue");
        assert_eq!(ids.get(after + 1), Some(&HOLD.answer));
        let rows = fact_rows().expect("chapter 16's rows compute");
        assert_eq!(rows.len(), PINS.len());
        for ((key, value), (pin, expected, _)) in rows.iter().zip(PINS) {
            assert_eq!((key.as_str(), value.as_str()), (pin, expected));
        }
        assert!(facts().is_ok());
    }

    #[test]
    fn cell_savings_are_not_clock_savings() {
        // The check compares these with comparison.json's own ratios; here, the cell saving must not read
        // as the clock or memory saving.
        let rows = fact_rows().expect("chapter 16's rows compute");
        let value = |key: &str| {
            rows.iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
                .expect("chapter 16 pins it")
        };
        assert_ne!(value("shear.share.leaf"), value("shear.share.wall"));
        assert_ne!(value("shear.share.leaf"), value("shear.share.memory"));
        let leaf = mean(&runs_of("adaptive").unwrap(), |r| r.leaf_steps as f64)
            / mean(&runs_of("fine").unwrap(), |r| r.leaf_steps as f64);
        assert!(leaf < 1.0);
    }

    #[test]
    fn the_shear_alone_readout_matches_its_receipts() {
        assert_eq!(shear_alone_note(), "vs the shear alone: fine 0.52%, adaptive 0.65%");
        // Both peaks fall at t = 0.400 s, where one factor, the whole flow's size over the shear's (5.036 on
        // both grids, bronze-shear-relative01), turns each run's whole-flow peak into its shear-alone peak.
        // A swapped or mistyped value breaks the shared factor.
        let factors = SHEAR_ALONE_PEAKS.map(|(mesh, peak)| {
            let whole = runs_of(mesh).expect("chapter 16 shows it")[0].peak_rel_l2;
            peak / 100.0 / whole
        });
        for factor in factors {
            assert!((5.03..5.04).contains(&factor), "{factor}");
        }
        assert!((factors[0] - factors[1]).abs() < 0.002);
    }

    #[test]
    fn the_16_3_change_of_picture_keeps_the_crossfade_frames() {
        // A play lasts ceil(seconds × fps) frames (fmn-anim clock.rs); 0.25 and 0.5 are exact binary fractions.
        let frames = |seconds: f64| (seconds * 60.0).ceil() as u32;
        assert_eq!(2 * frames(SWAP_SECONDS), frames(0.5));
    }
}
