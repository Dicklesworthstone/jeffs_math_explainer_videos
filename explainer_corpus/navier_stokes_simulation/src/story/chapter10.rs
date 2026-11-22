//! Chapter 10, "Small fixes and coarse grids" (navier-full-script03.md, cues 10_1 to 10_35, holds 10_h1 to
//! 10_h3).
//!
//! An eight-cell tube with a solid wall at the left and a tank held at p = 0 at the right. A guess for the
//! pressure correction stands on the dotted "right answer" line as bars, one scene unit per pressure unit,
//! with the imbalance badges (div u_new = −r) underneath. The chapter sweeps the guess with damped Jacobi,
//! graphs the largest error and residual, pairs the cells, solves the four-cell coarse puzzle, adds the
//! straight-line correction, sweeps again, and shows why the sweeps come first and why a small residual is
//! not a small error. Every bar, badge, graph point, coarse value and readout is computed by
//! `tube::TubeFacts` from the tube's rules and checked by `TubeFacts::check` before the first frame; its
//! fact rows are pinned to tube_oracle01's independent values. Nothing on screen is typed.
//!
//! The chapter opens on chapter 09's last frame, rebuilt at t = 0: the full tunnel of the retained startup
//! snapshot (`--tunnel DIR`, bound in `story.rs`) drawn by chapter 09's own `tunnel`, and the open
//! question card as chapter 09 builds it. It ends on the four step cards with "carry" lit, the tunnel
//! thumbnail and the docked question card. The script's Precisely lower thirds are not drawn, as in
//! chapters 08 and 09.
//!
//! Voice: the kit narrator, as in chapters 07 to 09 and 11. Each cue is `say`d before its pictures, and
//! pictures wait on its sentence starts (`until`). The three holds follow chapter 08's: the question card
//! is up by the line's last sentence, then `hold`, the "predict" mark and a countdown. The next line must
//! be the hold's answer (`say` refuses any other) and the card fades only once it has begun, so nothing
//! answers it early.

use explainer_kit::{BACKGROUND, Facts, Frame2, Kit, Palette, Timed, hex, rect, thousands};
use fmn::animation::{grow_arrow, indicate};
use fmn::library::TransformMatchingTex;
use fmn::prelude::*;

use super::chapter08::PREDICTION_SECONDS;
use super::chapter09::tunnel;
use crate::box_view::{self, BOX, CARD_WORD_SIZE, CARD_Y, CARDS, CORRECT, LETTER_SIZE, TAG_SIZE, WALL_SIZE, card_x, p};
use crate::error;
use crate::field::TunnelGeometry;
use crate::four_cell::{CELLS, FACES};
use crate::tube::{self, M, N, POST, PRE, R, TubeFacts, decimals};

/// Chapter 10's cues in spoken order and the number of sentences in each (the narrator's gates).
/// Sentences end at `.`, `?` or `!` before a space, as in the kit; the check recounts them.
pub const CUES: [(&str, usize); 35] = [
    ("10_1", 3),
    ("10_2", 3),
    ("10_3", 4),
    ("10_4", 3),
    ("10_5", 4),
    ("10_6", 3),
    ("10_7", 3),
    ("10_8", 3),
    ("10_9", 3),
    ("10_10", 3),
    ("10_11", 3),
    ("10_12", 4),
    ("10_13", 3),
    ("10_14", 3),
    ("10_15", 2),
    ("10_16", 4),
    ("10_17", 3),
    ("10_18", 3),
    ("10_19", 3),
    ("10_20", 2),
    ("10_21", 3),
    ("10_22", 2),
    ("10_23", 3),
    ("10_24", 1),
    ("10_25", 3),
    ("10_26", 4),
    ("10_27", 2),
    ("10_28", 4),
    ("10_29", 4),
    ("10_30", 2),
    ("10_31", 3),
    ("10_32", 4),
    ("10_33", 3),
    ("10_34", 3),
    ("10_35", 2),
];
/// The script's chapter heading, shown at the start as chapters 08 and 09 show theirs.
const TITLE: &str = "Small fixes and coarse grids";
/// The open question card (chapter 09's 09_12 card, docked under the thumbnail here).
const QUESTION: &str = "Does halving the cells change the answer?";

/// A prediction hold (the script's holds table): the line it follows, the line that answers it, the card.
struct Hold {
    id: &'static str,
    after: &'static str,
    answer: &'static str,
    question: &'static str,
    choices: [&'static str; 3],
}

const HOLDS: [Hold; 3] = [
    Hold {
        id: "10_h1",
        after: "10_7",
        answer: "10_8",
        question: "What does one sweep do to the zigzag?",
        choices: ["flip it", "halve it", "erase it"],
    },
    Hold {
        id: "10_h2",
        after: "10_21",
        answer: "10_22",
        question: "How big is the largest error\nafter the correction?",
        choices: ["about half", "about a tenth", "about two hundredths"],
    },
    Hold {
        id: "10_h3",
        after: "10_25",
        answer: "10_26",
        question: "Skip the sweeps: does the error\nshrink, stay or grow?",
        choices: ["shrink", "stay about the same", "grow"],
    },
];

/// The 10_13 citation lower third, as the script gives it.
const CITATION: &str = "Brandt, Mathematics of Computation, 1977, p. 338";

/// Basilisk `src/poisson.h` as the scene's reader tools/mg04.py quotes it (scene02 oracle-keys02.txt,
/// pinned file 82269e92): key, the quoted meaning, the source line.
const BASILISK: [(&str, &str, u32); 13] = [
    ("bas.residual.sign", "correction equation L(da) = b - L(a~)", 20),
    ("bas.cycle.restrict", "residual restricted to all levels first", 43),
    ("bas.cycle.coarsest", "coarsest level starts from zero", 53),
    (
        "bas.cycle.interp",
        "coarser correction bilinearly interpolated as the next guess",
        63,
    ),
    ("bas.nitermax", "100", 104),
    ("bas.nitermin", "1", 104),
    ("bas.tolerance", "1e-3", 105),
    (
        "bas.loop",
        "continue while i < NITERMAX and (i < NITERMIN or resa > tolerance)",
        180,
    ),
    ("bas.warning", "warns when the tolerance is not reached", 218),
    (
        "bas.relax.default",
        "default reuses values as soon as they are updated",
        278,
    ),
    ("bas.jacobi.weight", "2/3 (only when JACOBI is defined)", 336),
    (
        "bas.residual.max",
        "residual reported as the largest |res| over cells",
        370,
    ),
    (
        "bas.tolerance.meaning",
        "maximum relative change in volume of a cell per timestep",
        506,
    ),
];
/// Basilisk's loop condition, line 180, verbatim (the 10_31 source card).
const LOOP_CODE: &str = "s.i < NITERMAX && (s.i < NITERMIN || s.resa > tolerance);";

/// The two picture scales the script states, pinned to its words: the zoomed bars of 10_23 and 10_24
/// ("× 25, labelled") and the face arrows of 10_29 ("2.5 scene units per unit of flow").
const ZOOM: f64 = 25.0;
const FLOW_SCALE: f64 = 2.5;
const SCALES: [(&str, f64, &str, &str); 2] = [
    ("ch10.pic.zoom", ZOOM, "25", "script 10_23 picture"),
    ("ch10.pic.flow.scale", FLOW_SCALE, "2.5", "script 10_29 picture"),
];

/// `TubeFacts::fact_rows`, in order, each pinned to tube_oracle01's value and to the script line it supports.
const PINS: [(&str, &str, &str); 118] = [
    ("tube.cells", "8", "script 10_2 facts"),
    ("tube.diag", "1, 2, 2, 2, 2, 2, 2, 2", "script 10_2 facts"),
    (
        "tube.diag.is.matrix.diagonal",
        "true",
        "scene04 Conventions; tube_oracle01",
    ),
    ("tube.det", "1", "script 10_2 facts"),
    ("tube.no.free.constant", "true", "script 10_2 facts"),
    ("tube.symmetric", "true", "scene04 Conventions; tube_oracle01"),
    ("tube.identity", "true", "script 10_2 facts"),
    ("tube.truth", "0, 0, 0, 0, 0, 0, 0, 0", "script 10_3 facts"),
    (
        "tube.guess",
        "865/578, 271/578, 817/578, 191/578, 705/578, 47/578, 529/578, -161/578",
        "script 10_3 facts",
    ),
    (
        "tube.dome",
        "288/289, 280/289, 264/289, 240/289, 208/289, 168/289, 120/289, 64/289",
        "script 10_4 facts",
    ),
    (
        "tube.zigzag",
        "1/2, -1/2, 1/2, -1/2, 1/2, -1/2, 1/2, -1/2",
        "script 10_4 facts",
    ),
    ("tube.zigzag.size", "1/2", "script 10_4 facts"),
    ("tube.guess.err", "865/578", "script 10_4, 10_12, 10_26 facts"),
    ("ch10.show.wall", "0", "script 10_2 picture"),
    ("ch10.show.tank", "p = 0", "script 10_2 picture"),
    ("ch10.show.truth", "0", "script 10_3 picture"),
    ("ch10.show.truth.everywhere", "true", "script 10_3 picture"),
    ("ch10.show.weight", "1/2", "script 10_5 precisely"),
    ("zig.sweep1", "0, 0, 0, 0, 0, 0, 0, -1/8", "script 10_8 facts"),
    ("zig.sweep1.interior.zero", "true", "script 10_8 facts"),
    ("zig.sweep1.last", "1/8", "script 10_8 facts"),
    ("ch10.show.zig.last", "1/8", "script 10_8 picture"),
    (
        "zig.full.sweep1",
        "-1/2, 1/2, -1/2, 1/2, -1/2, 1/2, -1/2, 1/4",
        "script 10_9 facts",
    ),
    ("zig.full.flips", "true", "scene04 Conventions; tube_oracle01"),
    (
        "fix.full.sweep1",
        "-1/2, 1/2, -1/2, 1/2, -1/2, 1/2, -1/2, 1/4",
        "scene04 Conventions; tube_oracle01",
    ),
    ("fix.full.cells1to7.opposite", "true", "script 10_9 facts"),
    ("fix.full.cell8", "-1/2 -> 1/4", "script 10_9 facts"),
    ("fix.full.all.cross", "true", "script 10_9 facts"),
    (
        "fix.full.sweep2",
        "1/2, -1/2, 1/2, -1/2, 1/2, -1/2, 3/8, -1/4",
        "scene04 Conventions; tube_oracle01",
    ),
    ("fix.full.back.exact", "true", "script 10_9 facts"),
    ("fix.full.back.side", "true", "script 10_9 facts"),
    ("dome.move.interior.same", "true", "scene04 Conventions; tube_oracle01"),
    ("dome.move.interior", "2/289", "script 10_10 facts"),
    ("dome.err", "288/289", "script 10_10, 10_11 facts"),
    ("dome.resid", "8/289", "script 10_11 facts"),
    ("ch10.show.dome.move", "0.007", "script 10_10 picture"),
    ("ch10.show.dome.err", "0.997", "script 10_10 picture"),
    ("fix.dome.r.signed", "-8/289", "script 10_11 facts"),
    ("fix.dome.r.same", "true", "scene04 Conventions; tube_oracle01"),
    ("fix.dome.badge", "8/289", "script 10_11 facts"),
    (
        "fix.dome.moves",
        "-4/289, -2/289, -2/289, -2/289, -2/289, -2/289, -2/289, -2/289",
        "script 10_11 facts",
    ),
    ("ch10.show.dome.badge", "0.03", "script 10_11 picture"),
    ("mix.err.sweep1", "284/289", "script 10_12 facts"),
    ("mix.err.sweep3", "1113/1156", "script 10_29 facts"),
    ("mix.err.sweep20", "16322869695327/19859928776704", "script 10_12 facts"),
    ("mix.resid.sweep0", "586/289", "script 10_12 facts"),
    ("mix.resid.sweep1", "265/1156", "script 10_12 facts"),
    ("mix.resid.sweep3", "793/18496", "script 10_29 facts"),
    ("mix.resid.sweep20", "305044445913/9929964388352", "script 10_12 facts"),
    (
        "mix.err.curve",
        "1.497, 0.983, 0.972, 0.963, 0.954, 0.945, 0.936, 0.928, 0.919, 0.911, 0.903, 0.895, 0.886, 0.878, 0.870, 0.862, 0.854, 0.846, 0.838, 0.830, 0.822",
        "script 10_12 facts",
    ),
    (
        "mix.resid.curve",
        "2.028, 0.229, 0.076, 0.043, 0.035, 0.033, 0.033, 0.033, 0.033, 0.033, 0.033, 0.033, 0.033, 0.032, 0.032, 0.032, 0.032, 0.031, 0.031, 0.031, 0.031",
        "script 10_12 facts",
    ),
    ("fix.sweep1.linear", "true", "script 10_12 facts"),
    (
        "fix.sweep1.zigzag.left",
        "0, 0, 0, 0, 0, 0, 0, -1/8",
        "script 10_12 facts",
    ),
    ("fix.sweep1.cell8", "207/2312", "script 10_12 facts"),
    ("fix.sweep1.cell8.parts", "true", "scene04 Conventions; tube_oracle01"),
    ("fix.sweep1.err", "284/289", "scene04 Conventions; tube_oracle01"),
    (
        "ch10.show.sweep1.cell8",
        "0.0895 = 0.2145 − 0.125",
        "script 10_12 picture",
    ),
    ("ch10.show.sweep1.cell8.zigzag.negative", "true", "script 10_12 picture"),
    ("double.n8", "73", "script 10_14 facts"),
    ("double.n16", "296", "script 10_14 facts"),
    ("double.n32", "1192", "script 10_14 facts"),
    ("double.ratio1", "296/73", "script 10_14 facts"),
    ("double.ratio2", "149/37", "script 10_14 facts"),
    ("ch10.show.double.ratio", "4", "script 10_14 picture"),
    ("ch10.show.double.ratio.both", "true", "script 10_14 picture"),
    ("cycle.pre.err", "1113/1156", "script 10_16, 10_22 facts"),
    ("cycle.before.is.sweep3", "true", "scene04 Conventions; tube_oracle01"),
    (
        "cycle.restricted",
        "-121/2312, -135/2312, -2899/36992, 107/4624",
        "script 10_18 facts",
    ),
    (
        "coarse.rows",
        "1/2 -1/2 0 0; -1/2 1 -1/2 0; 0 -1/2 1 -1/2; 0 0 -1/2 7/6",
        "script 10_19 facts",
    ),
    (
        "cycle.coarse",
        "-70525/73984, -3693/4352, -46397/73984, -18417/73984",
        "script 10_19 facts",
    ),
    (
        "cycle.correction",
        "-70525/73984, -68589/73984, -64717/73984, -58685/73984, -50493/73984, -19701/36992, -6353/18496, -6139/36992",
        "script 10_20 facts",
    ),
    ("cycle.mid.err", "373/18496", "script 10_22 facts"),
    ("cycle.mid.resid", "373/18496", "script 10_23 facts"),
    ("cycle.end.err", "21015/1183744", "script 10_23 facts"),
    ("cycle.end.resid", "2471/591872", "script 10_23 facts"),
    ("cycle2.end.err", "817625/2424307712", "script 10_24 facts"),
    (
        "cycle2.end.resid",
        "174519/1212153856",
        "scene04 Conventions; tube_oracle01",
    ),
    ("smooth.only.match", "418", "script 10_23 facts"),
    (
        "fix.coarse.badges",
        "121/2312, 135/2312, 2899/36992, -107/4624",
        "script 10_18 facts",
    ),
    (
        "fix.coarse.badges.are.minus.R",
        "true",
        "scene04 Conventions; tube_oracle01",
    ),
    (
        "fix.coarse.rhs",
        "-121/2312, -135/2312, -2899/36992, 107/4624",
        "script 10_19 facts",
    ),
    (
        "fix.coarse.v",
        "-70525/73984, -3693/4352, -46397/73984, -18417/73984",
        "script 10_19 facts",
    ),
    ("fix.coarse.v.negative", "true", "script 10_19 facts"),
    (
        "fix.correction.negative",
        "-70525/73984, -68589/73984, -64717/73984, -58685/73984, -50493/73984, -19701/36992, -6353/18496, -6139/36992",
        "script 10_20 facts",
    ),
    (
        "fix.correction.all.negative",
        "true",
        "scene04 Conventions; tube_oracle01",
    ),
    ("fix.correction.rises", "true", "script 10_20 facts"),
    ("fix.correction.flat.wall", "true", "script 10_20 facts"),
    ("fix.correction.zero.at.tank", "true", "script 10_20 facts"),
    (
        "fix.correction.matches.rule",
        "true",
        "scene04 Conventions; tube_oracle01",
    ),
    ("fix.add.v.err", "373/18496", "script 10_22 facts"),
    ("fix.add.w.err", "141757/73984", "scene04 Conventions; tube_oracle01"),
    ("fix.pre.err", "1113/1156", "script 10_22 facts"),
    ("fix.bars.drop", "true", "script 10_22 facts"),
    (
        "ch10.show.coarse.badges",
        "0.052, 0.058, 0.078, −0.023",
        "script 10_18 picture",
    ),
    (
        "ch10.show.coarse.rhs",
        "−0.052, −0.058, −0.078, +0.023",
        "script 10_19 picture",
    ),
    (
        "ch10.show.coarse.v",
        "−0.95, −0.85, −0.63, −0.25",
        "script 10_19 picture",
    ),
    ("ch10.show.mid.err", "0.02", "script 10_22 picture"),
    ("ch10.show.alone", "418", "script 10_23 picture"),
    ("ch10.show.round", "3 + 2", "script 10_23 picture"),
    ("ch10.show.round2.err", "0.0003", "script 10_24 picture"),
    (
        "ch10.pic.sweep3.badges",
        "21/1156, 79/2312, 35/1156, 65/2312, 1313/36992, 793/18496, 159/18496, -587/18496",
        "script 10_16 picture state; tube_oracle01",
    ),
    (
        "ch10.pic.cycle.mid",
        "707/73984, 1299/73984, 1299/73984, 1219/73984, 1219/73984, 373/18496, 373/18496, 0",
        "script 10_22 picture state; tube_oracle01",
    ),
    (
        "ch10.pic.cycle.end",
        "1077/73984, 573/36992, 1237/73984, 20177/1183744, 20949/1183744, 21015/1183744, 16139/1183744, 1865/295936",
        "script 10_23 picture state; tube_oracle01",
    ),
    (
        "ch10.pic.cycle2.before",
        "586289/37879808, 1193899/75759616, 310493/18939904, 321033/18939904, 639121/37879808, 581117/37879808, 222079/18939904, 477425/75759616",
        "script 10_24 picture state; tube_oracle01",
    ),
    (
        "ch10.pic.cycle2.mid",
        "-827/8912896, -9745/75759616, -9745/75759616, 22647/151519232, 22647/151519232, 70439/151519232, 70439/151519232, 0",
        "script 10_24 picture state; tube_oracle01",
    ),
    (
        "ch10.pic.cycle2.end",
        "-139627/1212153856, -14587/142606336, -23931/606076928, 199459/2424307712, 559175/2424307712, 817625/2424307712, 727037/2424307712, 352195/2424307712",
        "script 10_24 picture state; tube_oracle01",
    ),
    ("skip.err", "29/4", "script 10_26 facts"),
    (
        "skip.restricted",
        "273/289, -16/289, -16/289, -321/578",
        "scene04 Conventions; tube_oracle01",
    ),
    (
        "ch10.pic.skip.mid",
        "29/4, 23/4, 23/4, 15/4, 15/4, 7/4, 7/4, 0",
        "script 10_26 picture state; tube_oracle01",
    ),
    ("skip.zigzag.pairs", "1, 0, 0, -1/2", "script 10_26 facts"),
    ("skip.zigzag.middle.cancel", "true", "script 10_26 facts"),
    ("skip.zigzag.ends.not", "true", "script 10_26 facts"),
    ("ch10.show.skip.err", "7.25", "script 10_26 picture"),
    (
        "ch10.show.skip.coarse.badges",
        "−0.94, 0.06, 0.06, 0.56",
        "script 10_26 picture",
    ),
    (
        "flow.sweep3",
        "21/1156, 121/2312, 191/2312, 32/289, 5409/36992, 6995/36992, 7313/36992, 6139/36992",
        "script 10_29, 10_30 facts",
    ),
    ("flow.sweep3.max", "7313/36992", "script 10_29 facts"),
    ("flow.running.sum", "true", "script 10_30 facts"),
    (
        "ch10.pic.flow.longest.face",
        "7",
        "script 10_29 picture state; tube_oracle01",
    ),
];

fn basilisk(key: &str) -> fmn::Result<&'static str> {
    BASILISK
        .iter()
        .find(|(k, _, _)| *k == key)
        .map(|(_, value, _)| *value)
        .ok_or_else(|| error(format!("chapter 10 quotes no Basilisk row {key}")))
}

/// Chapter 10's numbers alone, checked (a preview's `facts`).
pub fn facts() -> Result<Facts, String> {
    let mut facts = Facts::new();
    register(&mut facts)?;
    facts.self_check()?;
    Ok(facts)
}

/// Add chapter 10's numbers to a film's registry: every `TubeFacts::fact_rows` row, pinned as `PINS`
/// says, the two picture scales, and the Basilisk quotations with their source lines.
pub fn register(facts: &mut Facts) -> Result<(), String> {
    let f = TubeFacts::solve();
    f.check()?;
    let rows = f.fact_rows();
    let keys: Vec<&str> = rows.iter().map(|(key, _)| key.as_str()).collect();
    if keys != PINS.map(|(key, _, _)| key) {
        return Err(format!(
            "chapter 10 pins {:?}, but tube prints {keys:?}",
            PINS.map(|(key, _, _)| key)
        ));
    }
    for ((key, value), (_, expected, source)) in rows.iter().zip(PINS) {
        facts.pin(key, value, expected, source)?;
    }
    for (key, value, expected, source) in SCALES {
        facts.pin(key, value, expected, source)?;
    }
    for (key, value, line) in BASILISK {
        facts.pin(
            key,
            value,
            value,
            &format!("Basilisk src/poisson.h line {line}, quoted by scene02 mg04"),
        )?;
    }
    Ok(())
}

pub struct Chapter10 {
    pub kit: Kit,
    /// The tunnel the chapter opens on; `None` when the render was not given the retained snapshots.
    pub tunnel: Option<TunnelGeometry>,
}

/// The step cards chapter 10 names besides `CORRECT`: "carry" (10_35) and "smooth" (10_6).
const CARRY: usize = 0;
const SMOOTH: usize = 1;
/// Cell 4, the script's example cell in 10_5 and 10_10 (0-based).
const CELL_FOUR: usize = 3;

// Layout, scene units: the camera frame is 16 × 9 (x within ±8, y within ±4.5). The main window is
// x ∈ [−7.1, 0.2] and the side panel x ∈ [0.4, 6.95] (OP3); the tube, bars, badges and units tag keep one
// place and one scale from 10_2 to 10_34 (navier-multigrid-scene04 Layout). Widths are estimates (about
// CMR em × font size × 0.01).
const CELL: f64 = 0.75;
const TUBE_LEFT: f64 = -6.6;
const TUBE_TOP: f64 = -0.3;
const TUBE_BOTTOM: f64 = -1.05;
const TUBE_Y: f64 = (TUBE_TOP + TUBE_BOTTOM) / 2.0;
const ZERO_Y: f64 = 0.35;
const BAR_W: f64 = 0.45;
/// Bars above this are clipped with a break mark and their value printed (10_26); the scale never changes.
const CLIP_Y: f64 = 2.3;
const BADGE_Y: f64 = -1.45;
const BADGE_W: f64 = 0.68;
const BADGE_H: f64 = 0.38;
const BADGE_SIZE: f64 = 20.0;
const BRACKET_Y: f64 = -2.05;
const COARSE_Y: f64 = -2.45;
const COARSE_W: f64 = 1.2;
const COARSE_SIZE: f64 = 22.0;
/// Half the spacing of a pair's two copied terms under its bracket (10_18). At 20 pt the widest pair, 0.009 and
/// −0.032, keeps about 0.17 between the two and stays inside the bracket ends (pair centre ± 0.67).
const PAIR_TERM_DX: f64 = 0.35;
const RUN_Y: f64 = -1.98;
const RUN_NOTE_Y: f64 = -2.45;
const TAG_Y: f64 = 2.9;
const NOTE_Y: f64 = 2.45;
const TANK_RIGHT: f64 = 0.15;
const TANK_TOP: f64 = 0.45;
const TANK_BOTTOM: f64 = -1.8;
const TANK_WORD_Y: f64 = -1.3;
const TANK_P_Y: f64 = -1.6;
const ANSWER_AT: [f64; 2] = [-0.6, 1.95];
const NOTE_SIZE: f64 = 22.0;
const CAPTION_Y: f64 = -3.4;
const FLOW_THICKNESS: f64 = 3.0;
// Side panel.
const PANEL_X: f64 = 3.7;
const READOUT_Y: f64 = 0.9;
const READOUT_SIZE: f64 = 30.0;
const ROW_LEFT: f64 = 0.7;
const HUMP_ROW_Y: f64 = 0.55;
const ZIG_ROW_Y: f64 = -1.45;
const GRAPH: Frame2 = Frame2 {
    center: [3.7, -0.5, 0.0],
    width: 5.8,
    height: 4.2,
    x: (0.0, tube::GRAPH_SWEEPS as f64),
    y: (0.0, 2.1),
};
const GRAPH_LABEL_AT: f64 = 15.5;
const GRAPH_LABEL_GAP: f64 = 0.3;
const BAND_PAD: f64 = 0.1;
const HALF_LEFT: f64 = 0.9;
const HALF_LEN: f64 = 5.0;
const HALF_Y: [f64; tube::DOUBLING.len()] = [0.8, -0.55, -1.9];
const HALF_SCALE: f64 = 0.7;
const HALF_TEXT: f64 = 0.36;
const RATIO_X: f64 = 6.35;
/// Steps of the 10_23 "sweeps alone" count-up (a picture rate, not a fact; it ends on `alone`).
const ALONE_TICKS: usize = 20;
const MXC: [f64; M] = [0.95, 1.55, 2.15, 2.75];
const MYC: [f64; M] = [1.0, 0.45, -0.1, -0.65];
const VX: f64 = 3.75;
const EQX: f64 = 4.55;
const RX: f64 = 5.45;
const MATRIX_SIZE: f64 = 28.0;
const CORRECTION: Frame2 = Frame2 {
    center: [3.7, -1.95, 0.0],
    width: 5.4,
    height: 1.3,
    x: (0.0, N as f64 + 0.5),
    y: (-1.0, 0.0),
};
const ZOOM_ZERO_Y: f64 = -0.9;
const ALONE_Y: f64 = -1.75;
const ROUND_Y: f64 = -2.3;
const ASK_Y: f64 = 0.2;
const ASK_W: f64 = 6.2;
const ASK_H: f64 = 2.3;
const CHOICE_SIZE: f64 = 22.0;
const CHOICE_GAP: f64 = 0.5;
const STEPS_Y: f64 = 0.4;
const SOURCE_Y: f64 = -1.2;
const LADDER_Y: [f64; 4] = [1.2, 0.35, -0.5, -1.35];
const LADDER_LEFT: f64 = 1.6;
const LADDER_LEN: f64 = 3.6;
// The thumbnail and the docked question card (OP3 layout: thumbnail x ∈ [3.9, 6.9], y ∈ [2.75, 3.75]).
const THUMB: [f64; 4] = [3.9, 6.9, 2.75, 3.75];
const DOCK: [f64; 2] = [5.4, 2.225];
const DOCK_W: f64 = 3.0;
const DOCK_H: f64 = 0.85;
const DOCK_SIZE: f64 = 24.0;
// The four-cell box's return in 10_1: chapter 09's box, scaled about its centre and moved left.
const SMALL_BOX: f64 = 0.4;
const SMALL_BOX_AT: [f64; 2] = [-5.4, -0.7];
// Chapter 09's last frame: its TUNNEL_WIDTH and TUNNEL_CENTER_Y, and its 09_12 question card.
const TUNNEL_WIDTH: f64 = 11.5;
const TUNNEL_CENTER_Y: f64 = 1.15;
const CARD_09: [f64; 2] = [0.0, -1.25];
const CARD_09_W: f64 = 8.2;
const CARD_09_H: f64 = 0.8;
const CARD_09_SIZE: f64 = 30.0;

fn cx(i: usize) -> f64 {
    TUBE_LEFT + CELL * (i as f64 + 0.5)
}

fn fx(j: usize) -> f64 {
    TUBE_LEFT + CELL * j as f64
}

fn pair_x(c: usize) -> f64 {
    (cx(2 * c) + cx(2 * c + 1)) / 2.0
}

fn tank_mid() -> f64 {
    (fx(N) + TANK_RIGHT) / 2.0
}

/// OP3's pressure roles: orange above zero, teal below.
fn pressure_color(value: f64) -> Srgb {
    if value < 0.0 {
        Palette::teal()
    } else {
        box_view::push_color()
    }
}

/// Cell `i`'s open neighbours in `values` (index `N` is the tank, at its pressure; the wall is none)
/// and their average, which one sweep moves the cell halfway toward.
fn neighbours(values: &[R], i: usize) -> (Vec<(usize, R)>, R) {
    let mut near = Vec::new();
    if i > 0 {
        near.push((i - 1, values[i - 1]));
    }
    near.push((i + 1, values.get(i + 1).copied().unwrap_or_else(tube::tank)));
    let sum = near.iter().fold(R::ZERO, |a, &(_, b)| a + b);
    let average = sum / R::int(near.len() as i128);
    (near, average)
}

/// One row of bars: its zero line, scene units per pressure unit, bar width and clip height.
#[derive(Clone, Copy)]
struct BarRow {
    zero: f64,
    scale: f64,
    width: f64,
    clip: f64,
}

const MAIN: BarRow = BarRow {
    zero: ZERO_Y,
    scale: 1.0,
    width: BAR_W,
    clip: CLIP_Y,
};

/// A bar for `value` standing on the row's zero line at `x` (below it when negative); a ghost is an
/// outline only.
fn bar_at(stage: &mut Stage<'_>, row: BarRow, x: f64, value: f64, ghost: bool) -> fmn::Result<Mob> {
    let size = (value * row.scale).abs();
    let (bottom, height) = if value < 0.0 {
        (row.zero - size, size)
    } else {
        (row.zero, size.min(row.clip - row.zero))
    };
    let color = pressure_color(value);
    let style = if ghost {
        Style::default().fill(color, 0.0).stroke(Palette::ink(), 1.6, 1.0)
    } else {
        Style::default().fill(color, 0.85).stroke(color, 1.0, 1.0)
    };
    Ok(stage
        .arena_mut()
        .add(rect(p(x - row.width / 2.0, bottom), row.width, height, style)?))
}

fn dashed(stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb, width: f64) -> fmn::Result<Mob> {
    let d = DashedLine::new(a, b)
        .dash_length(0.07)
        .style(Style::default().stroke(color, width, 1.0))
        .build()?;
    Ok(stage.arena_mut().add(d))
}

fn dot(stage: &mut Stage<'_>, at: Vec3, color: Srgb, radius: f64) -> Mob {
    stage.arena_mut().add(
        Dot::new()
            .point(at)
            .radius(radius)
            .style(Style::default().fill(color, 1.0).stroke(color, 0.0, 1.0))
            .build(),
    )
}

/// A plain open rectangle (outlines, the tubes of 10_14 and 10_33, check boxes, the source card).
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

/// The question card's frame as chapter 09 builds it in 09_12, at any place and size.
fn question_card(stage: &mut Stage<'_>, at: [f64; 2], w: f64, h: f64) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Rectangle::new()
            .width(w)
            .height(h)
            .corner_radius(0.12)
            .style(
                Style::default()
                    .fill(Palette::ink(), 0.0)
                    .stroke(Palette::dim(), 2.0, 1.0),
            )
            .build()?
            .moved_to(p(at[0], at[1])),
    ))
}

/// The thumbnail's width and centre: the tunnel's own shape, as large as the thumbnail box allows.
fn thumbnail(g: TunnelGeometry) -> (f64, [f64; 2]) {
    let [x0, x1, y0, y1] = THUMB;
    let width = (x1 - x0).min((y1 - y0) * g.length / g.height);
    (width, [(x0 + x1) / 2.0, (y0 + y1) / 2.0])
}

/// The chapter's state: what is on screen, a pending batch, the tube's bars and badges, and the line
/// being spoken with the answer line a finished countdown waits for.
struct Ch10<'a> {
    k: &'a Kit,
    on: Vec<Mob>,
    pending: Vec<Box<dyn Animation>>,
    bars: Vec<Mob>,
    badges: Vec<[Mob; 2]>,
    line: &'static str,
    answer_due: Option<&'static str>,
}

impl Ch10<'_> {
    // Text and math through the kit, so the layout lint and the glyph check see every string.
    fn text(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        self.k.text(stage, s, size, color, at)
    }

    fn text_left(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        self.k.text_left(stage, s, size, color, at)
    }

    fn mono(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        self.k.mono(stage, s, size, color, at)
    }

    fn tex(&self, stage: &mut Stage<'_>, src: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        self.k.tex(stage, src, size, color, &[], at)
    }

    fn tex_left(&self, stage: &mut Stage<'_>, src: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        self.k.tex_left(stage, src, size, color, &[], at)
    }

    fn tex_spans(
        &self,
        stage: &mut Stage<'_>,
        src: &str,
        size: f64,
        color: Srgb,
        at: Vec3,
    ) -> fmn::Result<(Mob, fmn::library::SpanMapData)> {
        self.k.tex_spans(stage, src, size, color, &[], at)
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

    /// Remove at once (no fade): objects replaced in place, the countdown digits, a finished card.
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

    /// Tex to Tex by matching glyphs; the target replaces the source on screen.
    fn matching(
        &mut self,
        stage: &mut Stage<'_>,
        from: (Mob, &fmn::library::SpanMapData),
        to: (Mob, &fmn::library::SpanMapData),
        secs: f64,
    ) -> fmn::Result<Mob> {
        let a = TransformMatchingTex::new(stage.arena_mut(), from.0, to.0, from.1, to.1)?;
        self.pending.push(stage.prepare(a.rt(secs))?);
        self.on.retain(|&x| x != from.0);
        self.on.push(to.0);
        Ok(to.0)
    }

    /// Move an on-screen mobject by `by`.
    fn slide(&mut self, stage: &mut Stage<'_>, m: Mob, by: [f64; 2], secs: f64) -> fmn::Result<()> {
        if self.on.contains(&m) {
            let a = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(secs),
                    ..AnimateArgs::default()
                })?
                .shift(p(by[0], by[1]))?;
            self.pending.push(stage.prepare(a)?);
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

    fn refill(&mut self, stage: &mut Stage<'_>, m: Mob, color: Srgb, opacity: f64, secs: f64) -> fmn::Result<()> {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(secs),
                ..AnimateArgs::default()
            })?
            .set_fill(Some(color), Some(opacity))?;
        self.pending.push(stage.prepare(a)?);
        Ok(())
    }

    fn play(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        if !self.pending.is_empty() {
            stage.play_prepared(std::mem::take(&mut self.pending))?;
        }
        Ok(())
    }

    fn line(&self, stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb, width: f64) -> fmn::Result<Mob> {
        box_view::line(stage, a, b, color, width)
    }

    /// Finish what is queued, then start line `id` (the kit waits for the previous line first). After a
    /// countdown only the hold's answer may follow.
    fn say(&mut self, stage: &mut Stage<'_>, id: &'static str) -> fmn::Result<()> {
        if !CUES.iter().any(|c| c.0 == id) {
            return Err(error(format!("chapter 10 has no cue {id}")));
        }
        if let Some(due) = self.answer_due.take()
            && due != id
        {
            return Err(error(format!("chapter 10 answers its hold with {due}, not {id}")));
        }
        self.play(stage)?;
        println!("Chapter 10 cue {id} at {:.6} s", stage.scene().time().to_f64());
        self.line = id;
        self.k.say(stage, id)
    }

    /// Finish what is queued, then wait for sentence `j` of line `id` (nothing if it has begun).
    fn until(&mut self, stage: &mut Stage<'_>, id: &str, j: usize) -> fmn::Result<()> {
        let sentences = CUES.iter().find(|c| c.0 == id).map_or(0, |c| c.1);
        if j >= sentences {
            return Err(error(format!("chapter 10 line {id} has no sentence {j}")));
        }
        self.play(stage)?;
        self.k.until_sentence(stage, id, j)
    }

    // ---------- the tube's bars and badges ----------

    /// Every bar takes the height of its cell's value (queued).
    fn set_bars(&mut self, stage: &mut Stage<'_>, values: &[R], secs: f64) -> fmn::Result<()> {
        for (i, &value) in values.iter().enumerate() {
            let target = bar_at(stage, MAIN, cx(i), value.to_f64(), false)?;
            self.morph(stage, self.bars[i], target, secs)?;
        }
        Ok(())
    }

    /// Outline bars for `values`, detached.
    fn ghosts(&self, stage: &mut Stage<'_>, values: &[R]) -> fmn::Result<Vec<Mob>> {
        values
            .iter()
            .enumerate()
            .map(|(i, &value)| bar_at(stage, MAIN, cx(i), value.to_f64(), true))
            .collect()
    }

    fn move_ghosts(&mut self, stage: &mut Stage<'_>, ghosts: &[Mob], values: &[R], secs: f64) -> fmn::Result<()> {
        for (i, (&g, &value)) in ghosts.iter().zip(values).enumerate() {
            let target = bar_at(stage, MAIN, cx(i), value.to_f64(), true)?;
            self.morph(stage, g, target, secs)?;
        }
        Ok(())
    }

    /// A badge frame and its value at `at`: red, or green for an exact zero.
    fn badge(
        &self,
        stage: &mut Stage<'_>,
        at: Vec3,
        value: R,
        digits: usize,
        size: f64,
        width: f64,
    ) -> fmn::Result<[Mob; 2]> {
        let color = if value.is_zero() {
            Palette::green()
        } else {
            Palette::red()
        };
        let frame = box_view::frame(stage, at, width, BADGE_H, color, 2.0)?;
        let words = self.text(stage, &decimals(value, digits), size, color, at)?;
        Ok([frame, words])
    }

    /// The badge row shows `values` (the cells' imbalances, −r) to `digits` decimals; `secs` 0 swaps at once.
    fn set_badges(&mut self, stage: &mut Stage<'_>, values: &[R], digits: usize, secs: f64) -> fmn::Result<()> {
        let mut fresh = Vec::with_capacity(N);
        for (i, &value) in values.iter().enumerate() {
            fresh.push(self.badge(stage, p(cx(i), BADGE_Y), value, digits, BADGE_SIZE, BADGE_W)?);
        }
        let old = self.badge_mobs();
        let new: Vec<Mob> = fresh.iter().flatten().copied().collect();
        if secs > 0.0 {
            self.vanish(stage, &old, secs)?;
            self.appear(stage, &new, secs)?;
        } else {
            self.cut(stage, &old);
            self.put(stage, &new)?;
        }
        self.badges = fresh;
        Ok(())
    }

    fn clear_badges(&mut self, stage: &mut Stage<'_>, secs: f64) -> fmn::Result<()> {
        let old = self.badge_mobs();
        self.vanish(stage, &old, secs)?;
        self.badges.clear();
        Ok(())
    }

    fn badge_mobs(&self) -> Vec<Mob> {
        self.badges.iter().flatten().copied().collect()
    }

    // ---------- the prediction holds ----------

    /// The hold's card in the side panel with its question; the line must be the one the hold follows.
    fn ask(&mut self, stage: &mut Stage<'_>, hold: &Hold) -> fmn::Result<Vec<Mob>> {
        if self.line != hold.after {
            return Err(error(format!(
                "chapter 10 asks {} during {}, not {}",
                hold.id, self.line, hold.after
            )));
        }
        let card = stage.arena_mut().add(
            Rectangle::new()
                .width(ASK_W)
                .height(ASK_H)
                .corner_radius(0.12)
                .style(
                    Style::default()
                        .fill(hex("#182433"), 1.0)
                        .stroke(Palette::dim(), 1.4, 1.0),
                )
                .build()?
                .moved_to(p(PANEL_X, ASK_Y)),
        );
        let question = self.text(stage, hold.question, 26.0, Palette::ink(), p(PANEL_X, ASK_Y + 0.5))?;
        let shown = vec![card, question];
        self.appear(stage, &shown, 0.5)?;
        Ok(shown)
    }

    /// The three choices in one row across the card, one after another (as chapter 08 offers its own).
    fn offer(&mut self, stage: &mut Stage<'_>, hold: &Hold, card: &mut Vec<Mob>) -> fmn::Result<()> {
        let y = ASK_Y - 0.3;
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
            "Chapter 10 hold {} after {} at {:.6} s",
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
                p(PANEL_X, ASK_Y - 0.8),
            )?;
            self.put(stage, &[clock])?;
            stage.wait(1.0)?;
            self.cut(stage, &[clock]);
        }
        self.answer_due = Some(hold.answer);
        Ok(())
    }
}

/// The four-cell box of chapters 07 to 09 (outline, face lines, letters), small at the left.
fn small_box(s: &Ch10<'_>, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let mut parts = vec![box_view::outline(stage)?];
    for g in 0..FACES.len() {
        parts.push(box_view::dashed_face(stage, g)?);
    }
    for (i, name) in CELLS.iter().enumerate() {
        parts.push(s.text(stage, name, LETTER_SIZE, Palette::dim(), box_view::letter_at(i))?);
    }
    for &m in &parts {
        stage
            .arena_mut()
            .scale_about(m, SMALL_BOX, Some(p(BOX[0], BOX[1])), None);
        stage
            .arena_mut()
            .shift(m, p(SMALL_BOX_AT[0] - BOX[0], SMALL_BOX_AT[1] - BOX[1]));
    }
    Ok(parts)
}

/// The step cards, the lit one in the hero colour.
fn step_cards(s: &Ch10<'_>, stage: &mut Stage<'_>, lit: usize) -> fmn::Result<Vec<[Mob; 2]>> {
    (0..CARDS.len())
        .map(|k| {
            let (card, rim) = box_view::card(stage, k, k == lit)?;
            let word = s.text(stage, CARDS[k], CARD_WORD_SIZE, rim, p(card_x(k), CARD_Y))?;
            Ok([card, word])
        })
        .collect()
}

impl SceneConstruct for Chapter10 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        // Both refusals come before the first frame.
        let geometry = self.tunnel.ok_or_else(|| {
            error("chapter 10 opens on the tunnel: render it with --tunnel and the retained startup snapshots".into())
        })?;
        let f = TubeFacts::solve();
        f.check().map_err(error)?;
        let shown = facts().map_err(error)?;
        // The render log carries the fact table, so each picture's numbers can be read against it.
        for line in shown.print().lines() {
            println!("Chapter 10 fact {line}");
        }
        let fact = |key: &str| {
            shown
                .get(key)
                .map(str::to_string)
                .ok_or_else(|| error(format!("chapter 10 has no fact {key}")))
        };
        let ink = Palette::ink();
        let dim = Palette::dim();
        let hero = Palette::hero();
        let red = Palette::red();
        let teal = Palette::teal();
        let orange = box_view::push_color();
        let w = tube::weight();
        // Texts the pictures compute from the tube must read exactly as their pinned facts.
        let agree = |key: &str, drawn: String| -> fmn::Result<()> {
            let pinned = fact(key)?;
            if drawn == pinned {
                Ok(())
            } else {
                Err(error(format!("chapter 10 draws {drawn:?} where {key} is {pinned:?}")))
            }
        };
        let coarse3 = tube::restrict(&TubeFacts::badges(&f.history[PRE]));
        let listed: Vec<String> = coarse3.iter().map(|&v| decimals(v, 3)).collect();
        agree("ch10.show.coarse.badges", listed.join(", "))?;
        let hump_badges = TubeFacts::badges(&f.hump);
        for &v in &hump_badges {
            agree("ch10.show.dome.badge", decimals(v, 2))?;
        }
        agree("ch10.show.dome.err", decimals(tube::largest(&f.hump), 3))?;
        agree(
            "ch10.show.dome.move",
            decimals(f.hump[CELL_FOUR] - f.hump_sweep[CELL_FOUR], 3),
        )?;
        agree("ch10.show.mid.err", decimals(tube::largest(&f.one.mid), 2))?;
        agree("ch10.show.round2.err", decimals(tube::largest(&f.two.end), 4))?;
        agree("ch10.show.skip.err", decimals(tube::largest(&f.skip.mid), 2))?;
        agree("ch10.show.alone", f.alone.to_string())?;
        agree("ch10.pic.flow.longest.face", f.longest_face().to_string())?;
        let mut s = Ch10 {
            k: &self.kit,
            on: Vec::new(),
            pending: Vec::new(),
            bars: Vec::new(),
            badges: Vec::new(),
            line: "",
            answer_due: None,
        };

        // Chapter 09's last frame at t = 0: the full tunnel, by chapter 09's `tunnel`, and the open
        // question card as chapter 09 builds it.
        let full = tunnel(stage, geometry, TUNNEL_WIDTH, [0.0, TUNNEL_CENTER_Y])?;
        let card_frame = question_card(stage, CARD_09, CARD_09_W, CARD_09_H)?;
        let card_words = s.text(stage, QUESTION, CARD_09_SIZE, ink, p(CARD_09[0], CARD_09[1]))?;
        s.put(stage, &full)?;
        s.put(stage, &[card_frame, card_words])?;
        let dock = [card_frame, card_words];

        // 10_1: the tunnel shrinks to its thumbnail and the card docks under it; the title, then the step
        // cards with "correct" lit; the four-cell box returns small at the left, then fades.
        s.say(stage, "10_1")?;
        let (thumb_width, thumb_center) = thumbnail(geometry);
        let thumb = tunnel(stage, geometry, thumb_width, thumb_center)?;
        for (&from, &to) in full.iter().zip(&thumb) {
            s.morph(stage, from, to, 1.2)?;
        }
        let dock_frame = question_card(stage, DOCK, DOCK_W, DOCK_H)?;
        let dock_words = s.text(
            stage,
            &QUESTION.replacen("cells ", "cells\n", 1),
            DOCK_SIZE,
            ink,
            p(DOCK[0], DOCK[1]),
        )?;
        s.morph(stage, card_frame, dock_frame, 1.2)?;
        s.morph(stage, card_words, dock_words, 1.2)?;
        let title = s.text(stage, TITLE, 45.0, ink, p(0.0, CARD_Y))?;
        s.appear(stage, &[title], 0.6)?;
        s.play(stage)?;
        stage.wait(0.5)?;
        s.vanish(stage, &[title], 0.4)?;
        s.play(stage)?;
        let cards = step_cards(&s, stage, CORRECT)?;
        let card_mobs: Vec<Mob> = cards.iter().flatten().copied().collect();
        s.appear(stage, &card_mobs, 0.5)?;
        s.until(stage, "10_1", 1)?;
        let four = small_box(&s, stage)?;
        s.appear(stage, &four, 0.5)?;
        s.until(stage, "10_1", 2)?;
        s.vanish(stage, &four, 0.7)?;

        // 10_2: the tube draws cell by cell; the thick wall gets its 0; the tank, p = 0; the units tag.
        s.say(stage, "10_2")?;
        s.vanish(stage, &card_mobs, 0.5)?;
        s.play(stage)?;
        for i in 0..N {
            let (x0, x1) = (fx(i), fx(i + 1));
            let mut cell = vec![
                s.line(stage, p(x0, TUBE_TOP), p(x1, TUBE_TOP), ink, 2.0)?,
                s.line(stage, p(x0, TUBE_BOTTOM), p(x1, TUBE_BOTTOM), ink, 2.0)?,
            ];
            if i > 0 {
                cell.push(s.line(stage, p(x0, TUBE_TOP), p(x0, TUBE_BOTTOM), dim, 1.4)?);
            }
            s.appear(stage, &cell, 0.16)?;
            s.play(stage)?;
        }
        s.until(stage, "10_2", 1)?;
        let wall = s.line(
            stage,
            p(TUBE_LEFT, TUBE_TOP + 0.08),
            p(TUBE_LEFT, TUBE_BOTTOM - 0.08),
            ink,
            6.0,
        )?;
        let wall_flow = s.text(
            stage,
            &fact("ch10.show.wall")?,
            WALL_SIZE,
            dim,
            p(TUBE_LEFT - 0.25, TUBE_Y),
        )?;
        s.appear(stage, &[wall, wall_flow], 0.4)?;
        s.until(stage, "10_2", 2)?;
        let tank_x = fx(N);
        let tank = vec![
            s.line(stage, p(tank_x, TANK_TOP), p(TANK_RIGHT, TANK_TOP), dim, 2.0)?,
            s.line(stage, p(TANK_RIGHT, TANK_TOP), p(TANK_RIGHT, TANK_BOTTOM), dim, 2.0)?,
            s.line(stage, p(TANK_RIGHT, TANK_BOTTOM), p(tank_x, TANK_BOTTOM), dim, 2.0)?,
            s.line(stage, p(tank_x, TANK_TOP), p(tank_x, TUBE_TOP), dim, 2.0)?,
            s.line(stage, p(tank_x, TUBE_BOTTOM), p(tank_x, TANK_BOTTOM), dim, 2.0)?,
            s.text(stage, "tank", NOTE_SIZE, dim, p(tank_mid(), TANK_WORD_Y))?,
            s.tex(stage, &fact("ch10.show.tank")?, NOTE_SIZE, dim, p(tank_mid(), TANK_P_Y))?,
        ];
        let tag = s.tex_left(stage, &box_view::tag_tex(), TAG_SIZE, dim, p(TUBE_LEFT, TAG_Y))?;
        s.appear(stage, &tank, 0.5)?;
        s.appear(stage, &[tag], 0.5)?;

        // 10_3: the dotted right answer; the guess rises as bars, orange above and teal below; all error.
        s.say(stage, "10_3")?;
        let zero_line = dashed(stage, p(TUBE_LEFT, ZERO_Y), p(fx(N), ZERO_Y), ink, 1.45)?;
        let answer_words = |truth: &str| format!("right answer: {truth}");
        let answer = s.text(
            stage,
            &answer_words(&fact("ch10.show.truth")?),
            NOTE_SIZE,
            ink,
            p(ANSWER_AT[0], ANSWER_AT[1]),
        )?;
        s.appear(stage, &[zero_line, answer], 0.5)?;
        s.until(stage, "10_3", 2)?;
        s.pulse(stage, &[zero_line, answer], 0.8)?;
        s.until(stage, "10_3", 3)?;
        s.bars = (0..N)
            .map(|i| bar_at(stage, MAIN, cx(i), 0.0, false))
            .collect::<fmn::Result<Vec<Mob>>>()?;
        let bars = s.bars.clone();
        s.put(stage, &bars)?;
        s.set_bars(stage, &f.guess, 1.0)?;
        let gap_note = s.text_left(stage, "error = gap to the line", NOTE_SIZE, ink, p(TUBE_LEFT, NOTE_Y))?;
        s.appear(stage, &[gap_note], 0.5)?;

        // 10_4: the guess splits into the hump and the zigzag, side by side at the same scale, then
        // recombines.
        s.say(stage, "10_4")?;
        let hump_row = BarRow {
            zero: HUMP_ROW_Y,
            ..MAIN
        };
        let zig_row = BarRow {
            zero: ZIG_ROW_Y,
            ..MAIN
        };
        let row_mid = ROW_LEFT + CELL * N as f64 / 2.0;
        let shift_to = |row: BarRow| [ROW_LEFT - TUBE_LEFT, row.zero - MAIN.zero];
        let parts = |stage: &mut Stage<'_>, values: &[R]| -> fmn::Result<Vec<Mob>> {
            values
                .iter()
                .enumerate()
                .map(|(i, v)| bar_at(stage, MAIN, cx(i), v.to_f64(), false))
                .collect()
        };
        let hump_bars = parts(stage, &f.hump)?;
        let zig_bars = parts(stage, &f.zigzag)?;
        s.until(stage, "10_4", 1)?;
        s.vanish(stage, &[gap_note], 0.3)?;
        s.appear(stage, &hump_bars, 0.3)?;
        s.play(stage)?;
        for &m in &hump_bars {
            s.slide(stage, m, shift_to(hump_row), 1.0)?;
        }
        let hump_word = s.text(stage, "hump", NOTE_SIZE, dim, p(row_mid, HUMP_ROW_Y - 0.3))?;
        s.appear(stage, &[hump_word], 0.4)?;
        s.until(stage, "10_4", 2)?;
        s.appear(stage, &zig_bars, 0.3)?;
        s.play(stage)?;
        for &m in &zig_bars {
            s.slide(stage, m, shift_to(zig_row), 1.0)?;
        }
        let zig_word = s.tex(
            stage,
            &format!("\\text{{zigzag, }}\\pm {}", tube::largest(&f.zigzag).tex()),
            NOTE_SIZE,
            dim,
            p(row_mid, ZIG_ROW_Y - 0.75),
        )?;
        s.appear(stage, &[zig_word], 0.4)?;
        s.play(stage)?;
        stage.wait(1.2)?;
        let [hx, hy] = shift_to(hump_row);
        let [zx, zy] = shift_to(zig_row);
        for &m in &hump_bars {
            s.slide(stage, m, [-hx, -hy], 0.9)?;
        }
        for &m in &zig_bars {
            s.slide(stage, m, [-zx, -zy], 0.9)?;
        }
        s.vanish(stage, &[hump_word, zig_word], 0.5)?;
        s.play(stage)?;
        s.vanish(stage, &hump_bars, 0.3)?;
        s.vanish(stage, &zig_bars, 0.3)?;

        // 10_5: one cell's fix: cell 4 outlined, its neighbours' values ticked, their average ticked
        // white, an arrow halfway there; then cell 8 (the tank's 0) and cell 1 (only cell 2); then all
        // eight together, as outlines over the guess.
        s.say(stage, "10_5")?;
        let swept = tube::sweep(&f.guess, w);
        let demo = |s: &Ch10<'_>, stage: &mut Stage<'_>, i: usize| -> fmn::Result<(Mob, Vec<Mob>)> {
            let outline = plain_rect(stage, p(cx(i), TUBE_Y), CELL, TUBE_TOP - TUBE_BOTTOM, ink, 3.0)?;
            let (near, average) = neighbours(&f.guess, i);
            let mut marks = Vec::new();
            for (j, value) in near {
                let x = if j < N { cx(j) } else { tank_mid() };
                let y = ZERO_Y + value.to_f64();
                marks.push(s.line(stage, p(x - 0.3, y), p(x + 0.3, y), dim, 2.4)?);
            }
            let y = ZERO_Y + average.to_f64();
            marks.push(s.line(stage, p(cx(i) - 0.32, y), p(cx(i) + 0.32, y), ink, 3.0)?);
            let (from, to) = (ZERO_Y + f.guess[i].to_f64(), ZERO_Y + swept[i].to_f64());
            marks.push(box_view::arrow(stage, p(cx(i), from), p(cx(i), to), ink, 3.0)?);
            Ok((outline, marks))
        };
        let (outline, marks) = demo(&s, stage, CELL_FOUR)?;
        s.appear(stage, &[outline], 0.4)?;
        s.until(stage, "10_5", 1)?;
        s.appear(stage, &marks, 0.5)?;
        s.until(stage, "10_5", 2)?;
        s.vanish(stage, &[outline], 0.3)?;
        s.vanish(stage, &marks, 0.3)?;
        for i in [N - 1, 0] {
            let (outline, marks) = demo(&s, stage, i)?;
            s.appear(stage, &[outline], 0.4)?;
            s.appear(stage, &marks, 0.4)?;
            s.play(stage)?;
            stage.wait(0.8)?;
            s.vanish(stage, &[outline], 0.3)?;
            s.vanish(stage, &marks, 0.3)?;
        }
        s.until(stage, "10_5", 3)?;
        let ghosts = s.ghosts(stage, &f.guess)?;
        s.appear(stage, &ghosts, 0.3)?;
        s.play(stage)?;
        s.move_ghosts(stage, &ghosts, &swept, 0.9)?;
        s.play(stage)?;
        stage.wait(0.6)?;
        s.vanish(stage, &ghosts, 0.5)?;

        // 10_6: smoothing smooths the error in our guess, not the air; the smooth card flickers in the
        // card row with a grey note.
        s.say(stage, "10_6")?;
        let smoothing = s.text(
            stage,
            "smoothing: the error in our guess",
            NOTE_SIZE,
            ink,
            p((TUBE_LEFT + fx(N)) / 2.0, BADGE_Y),
        )?;
        s.appear(stage, &[smoothing], 0.5)?;
        s.until(stage, "10_6", 2)?;
        let (smooth_card, rim) = box_view::card(stage, SMOOTH, false)?;
        let smooth_word = s.text(stage, CARDS[SMOOTH], CARD_WORD_SIZE, rim, p(card_x(SMOOTH), CARD_Y))?;
        let viscosity = s.text_left(
            stage,
            "viscosity: the air",
            NOTE_SIZE,
            dim,
            p(card_x(SMOOTH) + 0.9, CARD_Y),
        )?;
        let smooth = [smooth_card, smooth_word, viscosity];
        s.appear(stage, &smooth, 0.4)?;
        s.play(stage)?;
        for _ in 0..2 {
            s.pulse(stage, &smooth[..2], 0.5)?;
            s.play(stage)?;
        }
        stage.wait(0.6)?;
        s.vanish(stage, &smooth, 0.5)?;

        // 10_7: the hump fades and only the zigzag remains; the first prediction.
        s.say(stage, "10_7")?;
        s.vanish(stage, &[smoothing], 0.3)?;
        s.set_bars(stage, &f.zigzag, 0.8)?;
        s.until(stage, "10_7", 1)?;
        let mut card = s.ask(stage, &HOLDS[0])?;
        s.until(stage, "10_7", 2)?;
        s.offer(stage, &HOLDS[0], &mut card)?;
        s.countdown(stage, &HOLDS[0])?;

        // 10_8: one sweep erases it: cells 1 to 7 land on the line and cell 8 keeps an eighth.
        s.say(stage, "10_8")?;
        s.vanish(stage, &card, 0.4)?;
        s.set_bars(stage, &f.zig_sweep, 0.9)?;
        s.until(stage, "10_8", 2)?;
        let eighth = s.text(
            stage,
            &fact("ch10.show.zig.last")?,
            NOTE_SIZE,
            ink,
            p(cx(N - 1), ZERO_Y - 0.35),
        )?;
        s.appear(stage, &[eighth], 0.4)?;

        // 10_9: a ghost replay at full jump: every bar swaps sides, then swaps back (cells 1 to 6 exactly).
        s.say(stage, "10_9")?;
        let jumps = s.ghosts(stage, &f.zigzag)?;
        s.appear(stage, &jumps, 0.4)?;
        s.until(stage, "10_9", 1)?;
        s.move_ghosts(stage, &jumps, &f.full[0], 0.9)?;
        s.until(stage, "10_9", 2)?;
        s.move_ghosts(stage, &jumps, &f.full[1], 0.9)?;
        s.play(stage)?;
        stage.wait(0.8)?;
        s.vanish(stage, &jumps, 0.5)?;

        // 10_10: the hump returns. One sweep runs as outlines over its solid bars, as in 10_5; a magnifier
        // on cell 4 reads its tiny move, and a separate readout gives the largest error of the solid bars,
        // the whole grid's (the hump's own, nearly one), so the readout and the bars agree.
        s.say(stage, "10_10")?;
        s.vanish(stage, &[eighth], 0.3)?;
        s.set_bars(stage, &f.hump, 0.9)?;
        s.until(stage, "10_10", 2)?;
        let sweep_ghosts = s.ghosts(stage, &f.hump)?;
        s.appear(stage, &sweep_ghosts, 0.3)?;
        s.play(stage)?;
        s.move_ghosts(stage, &sweep_ghosts, &f.hump_sweep, 0.8)?;
        s.play(stage)?;
        let top = ZERO_Y + f.hump_sweep[CELL_FOUR].to_f64();
        let magnifier = stage.arena_mut().add(
            Circle::new()
                .radius(0.32)
                .style(Style::default().fill(ink, 0.0).stroke(ink, 2.4, 1.0))
                .build()
                .moved_to(p(cx(CELL_FOUR), top)),
        );
        let moved = s.text(
            stage,
            &format!("move {}", fact("ch10.show.dome.move")?),
            NOTE_SIZE,
            ink,
            p(cx(CELL_FOUR), top + 0.57),
        )?;
        let largest = s.text(
            stage,
            &format!("largest error {}", fact("ch10.show.dome.err")?),
            READOUT_SIZE,
            ink,
            p(PANEL_X, READOUT_Y),
        )?;
        s.appear(stage, &[magnifier, moved], 0.4)?;
        s.appear(stage, &[largest], 0.4)?;

        // 10_11: the red badge row under the tube, every badge 0.03, beside the tall bars: the hump, whose
        // residual the badges are. The sweep's outlines and the magnifier go; the readout stays.
        s.say(stage, "10_11")?;
        s.vanish(stage, &[magnifier, moved], 0.3)?;
        s.vanish(stage, &sweep_ghosts, 0.3)?;
        s.set_badges(stage, &hump_badges, 2, 0.6)?;
        s.until(stage, "10_11", 1)?;
        let row = s.badge_mobs();
        s.pulse(stage, &row, 0.8)?;
        s.until(stage, "10_11", 2)?;
        s.pulse(stage, &s.bars.clone(), 0.8)?;

        // 10_12: sweeps on the whole guess; the side-panel graph draws point by point with the bars, a
        // solid orange line for the largest error and a dashed red one for the largest residual.
        s.say(stage, "10_12")?;
        s.vanish(stage, &[largest], 0.4)?;
        s.set_bars(stage, &f.guess, 0.8)?;
        s.set_badges(stage, &TubeFacts::badges(&f.guess), 2, 0.5)?;
        let g = GRAPH;
        let mut graph = vec![
            s.line(stage, p(g.left(), g.bottom()), p(g.right(), g.bottom()), dim, 1.6)?,
            s.line(stage, p(g.left(), g.bottom()), p(g.left(), g.top()), dim, 1.6)?,
        ];
        // Sweep ticks at the start, the middle and the end of the graph's window; value ticks at its whole
        // numbers above zero (zero is the corner the sweeps' 0 already labels).
        for k in (0..=tube::GRAPH_SWEEPS).step_by(tube::GRAPH_SWEEPS / 2) {
            let at = g.p(k as f64, 0.0);
            graph.push(s.text(stage, &k.to_string(), 20.0, dim, p(at[0], g.bottom() - 0.25))?);
        }
        for v in 1..=(g.y.1.floor() as u32) {
            let at = g.p(0.0, f64::from(v));
            graph.push(s.text(stage, &v.to_string(), 20.0, dim, p(g.left() - 0.22, at[1]))?);
        }
        let axis_at = g.p(GRAPH_LABEL_AT, 0.0);
        graph.push(s.text(stage, "sweeps", 20.0, dim, p(axis_at[0], g.bottom() - 0.25))?);
        let errors: Vec<f64> = f.errors.iter().map(|x| x.to_f64()).collect();
        let residuals: Vec<f64> = f.residuals.iter().map(|x| x.to_f64()).collect();
        graph.push(dot(stage, g.p(0.0, errors[0]), orange, 0.045));
        graph.push(dot(stage, g.p(0.0, residuals[0]), red, 0.045));
        s.appear(stage, &graph, 0.5)?;
        let step = |s: &mut Ch10<'_>, stage: &mut Stage<'_>, graph: &mut Vec<Mob>, k: usize| -> fmn::Result<()> {
            s.set_bars(stage, &f.history[k], 0.2)?;
            s.set_badges(stage, &TubeFacts::badges(&f.history[k]), 2, 0.0)?;
            let (x0, x1) = ((k - 1) as f64, k as f64);
            let new = vec![
                s.line(stage, g.p(x0, errors[k - 1]), g.p(x1, errors[k]), orange, 3.0)?,
                dashed(stage, g.p(x0, residuals[k - 1]), g.p(x1, residuals[k]), red, 2.5)?,
                dot(stage, g.p(x1, errors[k]), orange, 0.045),
                dot(stage, g.p(x1, residuals[k]), red, 0.045),
            ];
            s.appear(stage, &new, 0.2)?;
            s.play(stage)?;
            graph.extend(new);
            Ok(())
        };
        s.until(stage, "10_12", 1)?;
        step(&mut s, stage, &mut graph, 1)?;
        let cell8 = ZERO_Y + f.history[1][N - 1].to_f64();
        let tick = s.line(stage, p(cx(N - 1) - 0.3, cell8), p(cx(N - 1) + 0.3, cell8), ink, 3.0)?;
        let parts8 = s.text(
            stage,
            &fact("ch10.show.sweep1.cell8")?,
            20.0,
            ink,
            p(cx(N - 1) - 0.6, 1.5),
        )?;
        s.appear(stage, &[tick, parts8], 0.4)?;
        s.until(stage, "10_12", 2)?;
        s.vanish(stage, &[tick, parts8], 0.4)?;
        for k in 2..=tube::GRAPH_SWEEPS / 2 {
            step(&mut s, stage, &mut graph, k)?;
        }
        s.until(stage, "10_12", 3)?;
        for k in tube::GRAPH_SWEEPS / 2 + 1..=tube::GRAPH_SWEEPS {
            step(&mut s, stage, &mut graph, k)?;
        }
        let last = tube::GRAPH_SWEEPS;
        let err_label_at = g.p(GRAPH_LABEL_AT, errors[last]);
        let res_label_at = g.p(GRAPH_LABEL_AT, residuals[last]);
        let line_labels = vec![
            s.text(
                stage,
                "largest error",
                22.0,
                orange,
                p(err_label_at[0], err_label_at[1] - GRAPH_LABEL_GAP),
            )?,
            s.text(
                stage,
                "largest residual",
                22.0,
                red,
                p(res_label_at[0], res_label_at[1] + GRAPH_LABEL_GAP),
            )?,
        ];
        s.appear(stage, &line_labels, 0.4)?;
        graph.extend(line_labels);

        // 10_13: fast at first, then stuck: the flat part of both lines outlined white; Brandt cited.
        s.say(stage, "10_13")?;
        let band = |stage: &mut Stage<'_>, values: &[f64], from: usize| -> fmn::Result<Mob> {
            let tail = &values[from..];
            let lo = tail.iter().fold(f64::INFINITY, |m, &x| m.min(x));
            let hi = tail.iter().fold(f64::NEG_INFINITY, |m, &x| m.max(x));
            let (a, b) = (g.p(from as f64, lo), g.p(last as f64, hi));
            plain_rect(
                stage,
                p((a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0),
                b[0] - a[0] + 2.0 * BAND_PAD,
                b[1] - a[1] + 2.0 * BAND_PAD,
                ink,
                2.0,
            )
        };
        let outlines = vec![band(stage, &errors, 1)?, band(stage, &residuals, PRE)?];
        s.appear(stage, &outlines, 0.5)?;
        s.until(stage, "10_13", 1)?;
        let citation = s.text(stage, CITATION, 24.0, dim, p(0.0, CAPTION_Y))?;
        s.appear(stage, &[citation], 0.5)?;

        // 10_14: three tubes of one length (8, 16 and 32 cells), each halving its own hump with a sweep
        // counter; "about × 4" between them.
        s.say(stage, "10_14")?;
        s.vanish(stage, &graph, 0.5)?;
        s.vanish(stage, &outlines, 0.5)?;
        s.vanish(stage, &[citation], 0.5)?;
        s.until(stage, "10_14", 1)?;
        let half_row = |y: f64, n: usize| BarRow {
            zero: y + 0.12,
            scale: HALF_SCALE,
            width: 0.8 * HALF_LEN / n as f64,
            clip: y + 0.12 + HALF_SCALE,
        };
        let half_x = |n: usize, i: usize| HALF_LEFT + HALF_LEN * (i as f64 + 0.5) / n as f64;
        let count_at = |row: usize| p(HALF_LEFT + HALF_LEN - 0.6, HALF_Y[row] - HALF_TEXT);
        let row_bars = |stage: &mut Stage<'_>, row: usize, cells: usize, state: &[f64]| -> fmn::Result<Vec<Mob>> {
            let bars_row = half_row(HALF_Y[row], cells);
            state
                .iter()
                .enumerate()
                .map(|(i, &v)| bar_at(stage, bars_row, half_x(cells, i), v, false))
                .collect()
        };
        let mut halving = Vec::new();
        let mut halving_bars: Vec<Vec<Mob>> = Vec::new();
        let mut counters = Vec::new();
        for (row, h) in f.halvings.iter().enumerate() {
            let y = HALF_Y[row];
            halving.push(plain_rect(
                stage,
                p(HALF_LEFT + HALF_LEN / 2.0, y),
                HALF_LEN,
                0.24,
                ink,
                1.6,
            )?);
            let start = h.start.iter().fold(0.0_f64, |m, x| m.max(x.abs()));
            let half = half_row(y, h.cells).zero + HALF_SCALE * start / 2.0;
            halving.push(dashed(
                stage,
                p(HALF_LEFT, half),
                p(HALF_LEFT + HALF_LEN, half),
                ink,
                1.2,
            )?);
            halving.push(s.text_left(
                stage,
                &format!("{} cells", h.cells),
                20.0,
                dim,
                p(HALF_LEFT, y - HALF_TEXT),
            )?);
            halving_bars.push(row_bars(stage, row, h.cells, &h.samples[0].1)?);
            let (k, _) = &h.samples[0];
            counters.push(s.text(
                stage,
                &format!("{} sweeps", thousands(*k as u64)),
                20.0,
                ink,
                count_at(row),
            )?);
        }
        let first_bars: Vec<Mob> = halving_bars.iter().flatten().copied().collect();
        s.appear(stage, &halving, 0.5)?;
        s.appear(stage, &first_bars, 0.5)?;
        s.appear(stage, &counters, 0.5)?;
        s.play(stage)?;
        for j in 1..=tube::HALVING_SAMPLES {
            for (row, h) in f.halvings.iter().enumerate() {
                let (k, state) = &h.samples[j];
                let fresh = row_bars(stage, row, h.cells, state)?;
                let old = std::mem::replace(&mut halving_bars[row], fresh.clone());
                s.cut(stage, &old);
                s.put(stage, &fresh)?;
                let count = s.text(
                    stage,
                    &format!("{} sweeps", thousands(*k as u64)),
                    20.0,
                    ink,
                    count_at(row),
                )?;
                s.cut(stage, &[counters[row]]);
                s.put(stage, &[count])?;
                counters[row] = count;
            }
            stage.wait(0.1)?;
        }
        s.until(stage, "10_14", 2)?;
        let ratio = fact("ch10.show.double.ratio")?;
        let mut ratios = Vec::new();
        for row in 0..f.halvings.len() - 1 {
            let y = (HALF_Y[row] + HALF_Y[row + 1]) / 2.0 - HALF_TEXT;
            ratios.push(s.text(stage, &format!("about \u{00d7} {ratio}"), 22.0, hero, p(RATIO_X, y))?);
        }
        s.appear(stage, &ratios, 0.5)?;

        // 10_15: bad news for the question card, which glows once.
        s.say(stage, "10_15")?;
        s.pulse(stage, &dock, 1.0)?;

        // 10_16: back to the eight-cell tube after three sweeps, a smooth slope; brackets pair the cells.
        s.say(stage, "10_16")?;
        let mut halving_all = halving.clone();
        halving_all.extend(halving_bars.iter().flatten().copied());
        halving_all.extend(counters.iter().copied());
        halving_all.extend(ratios.iter().copied());
        s.vanish(stage, &halving_all, 0.5)?;
        s.until(stage, "10_16", 1)?;
        s.set_bars(stage, &f.history[PRE], 0.8)?;
        s.set_badges(stage, &TubeFacts::badges(&f.history[PRE]), 3, 0.5)?;
        s.until(stage, "10_16", 3)?;
        let mut brackets = Vec::new();
        for c in 0..M {
            let (a, b) = (fx(2 * c) + 0.08, fx(2 * c + 2) - 0.08);
            brackets.push(s.line(stage, p(a, BRACKET_Y), p(b, BRACKET_Y), ink, 1.6)?);
            brackets.push(s.line(stage, p(a, BRACKET_Y), p(a, BRACKET_Y + 0.12), ink, 1.6)?);
            brackets.push(s.line(stage, p(b, BRACKET_Y), p(b, BRACKET_Y + 0.12), ink, 1.6)?);
        }
        s.appear(stage, &brackets, 0.6)?;

        // 10_17: the error solves the same kind of puzzle, with the residuals on the right side.
        s.say(stage, "10_17")?;
        let (lpd, lpd_spans) = s.tex_spans(stage, "L\\,p = d", 44.0, ink, p(PANEL_X, 1.0))?;
        s.appear(stage, &[lpd], 0.5)?;
        s.until(stage, "10_17", 2)?;
        let (ler, ler_spans) = s.tex_spans(stage, "L\\,e = r", 44.0, ink, p(PANEL_X, 1.0))?;
        let equation = s.matching(stage, (lpd, &lpd_spans), (ler, &ler_spans), 1.0)?;
        let meaning = vec![
            s.text(stage, "e: error, unknown", 24.0, orange, p(PANEL_X, 0.2))?,
            s.text(stage, "r: residual, measured", 24.0, red, p(PANEL_X, -0.3))?,
        ];
        s.appear(stage, &meaning, 0.5)?;

        // 10_18: each pair's two badge values slide down side by side under their bracket, apart so both terms
        // stay readable; the shared face flashes plus in one and minus in the other, then cancels; the pair's
        // coarse badge (their sum) then replaces the two terms.
        s.say(stage, "10_18")?;
        let badges3 = TubeFacts::badges(&f.history[PRE]);
        let coarse_badges = &coarse3;
        let mut copies = Vec::new();
        for (i, &value) in badges3.iter().enumerate() {
            let copy = s.text(stage, &decimals(value, 3), BADGE_SIZE, red, p(cx(i), BADGE_Y))?;
            s.put(stage, &[copy])?;
            let side = if i % 2 == 0 { -1.0 } else { 1.0 };
            let dx = pair_x(i / 2) + side * PAIR_TERM_DX - cx(i);
            s.slide(stage, copy, [dx, COARSE_Y - BADGE_Y], 0.8)?;
            copies.push(copy);
        }
        s.until(stage, "10_18", 1)?;
        let mut faces = Vec::new();
        for c in 0..M {
            let x = fx(2 * c + 1);
            faces.push(s.text(stage, "+", 24.0, red, p(x - 0.17, TUBE_Y))?);
            faces.push(s.text(stage, "\u{2212}", 24.0, red, p(x + 0.17, TUBE_Y))?);
        }
        s.appear(stage, &faces, 0.4)?;
        s.play(stage)?;
        stage.wait(0.8)?;
        s.vanish(stage, &faces, 0.5)?;
        s.until(stage, "10_18", 2)?;
        s.vanish(stage, &copies, 0.3)?;
        let mut coarse = Vec::new();
        for (c, &value) in coarse_badges.iter().enumerate() {
            coarse.push(s.badge(stage, p(pair_x(c), COARSE_Y), value, 3, COARSE_SIZE, COARSE_W)?);
        }
        let coarse_mobs: Vec<Mob> = coarse.iter().flatten().copied().collect();
        s.appear(stage, &coarse_mobs, 0.4)?;

        // 10_19: the 4 × 4 coarse puzzle; the coarse badges move into its right side and flip sign under
        // "residual = − imbalance"; then its four solved values, all negative.
        s.say(stage, "10_19")?;
        let mut side = vec![equation];
        side.extend(meaning);
        s.vanish(stage, &side, 0.4)?;
        let mut matrix = Vec::new();
        for (i, row) in f.coarse_rows.iter().enumerate() {
            for (j, value) in row.iter().enumerate() {
                matrix.push(s.tex(stage, &value.tex(), MATRIX_SIZE, ink, p(MXC[j], MYC[i]))?);
            }
        }
        let mut unknowns = Vec::new();
        for (i, &y) in MYC.iter().enumerate() {
            unknowns.push(s.tex(stage, &format!("v_{{{}}}", i + 1), MATRIX_SIZE, teal, p(VX, y))?);
        }
        let (top, bottom) = (MYC[0] + 0.32, MYC[M - 1] - 0.32);
        for (x, opens_right) in [
            (MXC[0] - 0.33, true),
            (MXC[M - 1] + 0.33, false),
            (VX - 0.45, true),
            (VX + 0.45, false),
            (RX - 0.5, true),
            (RX + 0.5, false),
        ] {
            let tick = if opens_right { 0.12 } else { -0.12 };
            matrix.push(s.line(stage, p(x, top), p(x, bottom), dim, 2.0)?);
            matrix.push(s.line(stage, p(x, top), p(x + tick, top), dim, 2.0)?);
            matrix.push(s.line(stage, p(x, bottom), p(x + tick, bottom), dim, 2.0)?);
        }
        matrix.push(s.tex(stage, "=", MATRIX_SIZE, ink, p(EQX, (top + bottom) / 2.0))?);
        s.appear(stage, &matrix, 0.5)?;
        s.appear(stage, &unknowns, 0.5)?;
        s.until(stage, "10_19", 1)?;
        let flip = s.text(stage, "residual = \u{2212} imbalance", 20.0, ink, p(RX, 1.55))?;
        s.appear(stage, &[flip], 0.4)?;
        s.play(stage)?;
        let rhs_text = fact("ch10.show.coarse.rhs")?;
        let mut rhs = Vec::new();
        for (c, words) in rhs_text.split(", ").enumerate() {
            let copy = s.text(
                stage,
                &decimals(coarse_badges[c], 3),
                COARSE_SIZE,
                red,
                p(pair_x(c), COARSE_Y),
            )?;
            s.put(stage, &[copy])?;
            let target = s.text(stage, words, 26.0, red, p(RX, MYC[c]))?;
            s.morph(stage, copy, target, 1.0)?;
            rhs.push(copy);
        }
        s.until(stage, "10_19", 2)?;
        let solved_text = fact("ch10.show.coarse.v")?;
        for (i, words) in solved_text.split(", ").enumerate() {
            let target = s.text(stage, words, 26.0, teal, p(VX, MYC[i]))?;
            s.morph(stage, unknowns[i], target, 0.8)?;
        }
        matrix.extend(unknowns.iter().copied());
        matrix.extend(rhs);
        matrix.push(flip);

        // 10_20: in the side panel, the coarse values as teal dots below the zero line, joined by straight
        // segments, flat from the wall and reaching zero at the tank's centre; eight ticks read off the
        // fine values. The tube's bars do not change yet.
        s.say(stage, "10_20")?;
        let cg = CORRECTION;
        let at = |x: R, y: R| cg.p(x.to_f64(), y.to_f64());
        let mut plot = vec![
            dashed(stage, at(R::ZERO, R::ZERO), at(tube::tank_x(N), R::ZERO), ink, 1.45)?,
            s.text(stage, "wall", 20.0, dim, p(cg.left(), cg.bottom() - 0.25))?,
            s.text(stage, "tank", 20.0, dim, p(cg.right(), cg.bottom() - 0.25))?,
        ];
        let one = &f.one;
        let centres = tube::coarse_centres();
        let mut knots = vec![(R::ZERO, one.coarse[0])];
        knots.extend(centres.iter().copied().zip(one.coarse.iter().copied()));
        knots.push((tube::tank_x(N), tube::tank()));
        for pair in knots.windows(2) {
            plot.push(s.line(stage, at(pair[0].0, pair[0].1), at(pair[1].0, pair[1].1), teal, 3.0)?);
        }
        for (&x, &v) in centres.iter().zip(&one.coarse) {
            plot.push(dot(stage, at(x, v), teal, 0.07));
        }
        s.appear(stage, &plot, 0.6)?;
        s.until(stage, "10_20", 1)?;
        let mut stems = Vec::new();
        for (i, &c) in one.correction.iter().enumerate() {
            let x = R::new(2 * i as i128 + 1, 2);
            stems.push(s.line(stage, at(x, R::ZERO), at(x, c), teal, 1.6)?);
            stems.push(dot(stage, at(x, c), ink, 0.045));
        }
        s.appear(stage, &stems, 0.6)?;
        plot.extend(stems.iter().copied());

        // 10_21: before the correction the largest error is almost one; how big after? The bars are
        // unchanged.
        s.say(stage, "10_21")?;
        s.vanish(stage, &matrix, 0.4)?;
        s.until(stage, "10_21", 1)?;
        let mut card = s.ask(stage, &HOLDS[1])?;
        s.until(stage, "10_21", 2)?;
        s.offer(stage, &HOLDS[1], &mut card)?;
        s.countdown(stage, &HOLDS[1])?;

        // 10_22: about two hundredths: each tick's value is added to its bar, and every bar drops to
        // nearly the line.
        s.say(stage, "10_22")?;
        s.vanish(stage, &card, 0.4)?;
        s.pulse(stage, &stems, 0.6)?;
        s.play(stage)?;
        s.set_bars(stage, &one.mid, 1.0)?;
        s.set_badges(stage, &TubeFacts::badges(&one.mid), 3, 0.5)?;
        // The 10_18 coarse sums describe the residual before this correction: they and their brackets leave
        // with it, inside its 1.0 s group (no clock change). 10_26 and 10_27 bring the brackets back.
        s.vanish(stage, &coarse_mobs, 0.4)?;
        s.vanish(stage, &brackets, 0.4)?;
        let mut readout = s.text(
            stage,
            &format!("largest error {}", fact("ch10.show.mid.err")?),
            READOUT_SIZE,
            ink,
            p(PANEL_X, READOUT_Y),
        )?;
        s.appear(stage, &[readout], 0.5)?;

        // 10_23: a zoomed bar view (× 25, labelled) shows two more sweeps smoothing the kinks; a counter
        // "sweeps alone" ticks up beside the round's recipe.
        s.say(stage, "10_23")?;
        s.vanish(stage, &plot, 0.4)?;
        let zoom_row = BarRow {
            zero: ZOOM_ZERO_Y,
            scale: ZOOM,
            width: BAR_W,
            clip: READOUT_Y - 0.3,
        };
        let zoom_bars = |stage: &mut Stage<'_>, values: &[R]| -> fmn::Result<Vec<Mob>> {
            values
                .iter()
                .enumerate()
                .map(|(i, v)| bar_at(stage, zoom_row, ROW_LEFT + CELL * (i as f64 + 0.5), v.to_f64(), false))
                .collect()
        };
        let zoomed = zoom_bars(stage, &one.mid)?;
        let zoom_frame = vec![
            dashed(
                stage,
                p(ROW_LEFT, ZOOM_ZERO_Y),
                p(ROW_LEFT + CELL * N as f64, ZOOM_ZERO_Y),
                ink,
                1.45,
            )?,
            s.text_left(
                stage,
                &format!("\u{00d7} {}", fact("ch10.pic.zoom")?),
                24.0,
                ink,
                p(ROW_LEFT, ZOOM_ZERO_Y + 0.75),
            )?,
        ];
        s.appear(stage, &zoom_frame, 0.5)?;
        s.appear(stage, &zoomed, 0.5)?;
        s.play(stage)?;
        let sweep_on = |s: &mut Ch10<'_>, stage: &mut Stage<'_>, state: &[R], badges: bool| -> fmn::Result<()> {
            s.set_bars(stage, state, 0.6)?;
            let fresh = zoom_bars(stage, state)?;
            for (&from, &to) in zoomed.iter().zip(&fresh) {
                s.morph(stage, from, to, 0.6)?;
            }
            if badges {
                s.set_badges(stage, &TubeFacts::badges(state), 3, 0.0)?;
            }
            s.play(stage)
        };
        for k in 1..=POST {
            sweep_on(&mut s, stage, &tube::sweeps(&one.mid, k, w), true)?;
        }
        s.until(stage, "10_23", 1)?;
        let row = s.badge_mobs();
        s.pulse(stage, &row, 0.8)?;
        s.until(stage, "10_23", 2)?;
        let round = s.text(
            stage,
            &format!("this round: {} sweeps + one four-cell solve", fact("ch10.show.round")?),
            22.0,
            dim,
            p(PANEL_X, ROUND_Y),
        )?;
        let alone_words = |n: usize| format!("sweeps alone: {n}");
        let mut alone = s.text(stage, &alone_words(0), 26.0, ink, p(PANEL_X, ALONE_Y))?;
        s.appear(stage, &[round, alone], 0.4)?;
        s.play(stage)?;
        for j in 1..=ALONE_TICKS {
            let count = s.text(
                stage,
                &alone_words(f.alone * j / ALONE_TICKS),
                26.0,
                ink,
                p(PANEL_X, ALONE_Y),
            )?;
            s.cut(stage, &[alone]);
            s.put(stage, &[count])?;
            alone = count;
            stage.wait(0.08)?;
        }

        // 10_24: a second round flattens the zoomed bars onto the line and the readout drops. The script's
        // picture names no badges here, so the badge row rests until the reset.
        s.say(stage, "10_24")?;
        s.clear_badges(stage, 0.4)?;
        for state in [&f.two.before, &f.two.mid, &f.two.end] {
            sweep_on(&mut s, stage, state, false)?;
        }
        let second = s.text(
            stage,
            &format!("largest error {}", fact("ch10.show.round2.err")?),
            READOUT_SIZE,
            ink,
            p(PANEL_X, READOUT_Y),
        )?;
        s.vanish(stage, &[readout], 0.4)?;
        s.appear(stage, &[second], 0.4)?;
        readout = second;

        // 10_25: skip the sweeps? Reset to the original guess; the third prediction.
        s.say(stage, "10_25")?;
        let mut panel = zoom_frame.clone();
        panel.extend(zoomed.iter().copied());
        panel.extend([readout, round, alone]);
        s.vanish(stage, &panel, 0.5)?;
        s.set_bars(stage, &f.guess, 0.8)?;
        s.set_badges(stage, &TubeFacts::badges(&f.guess), 2, 0.5)?;
        s.until(stage, "10_25", 1)?;
        let mut card = s.ask(stage, &HOLDS[2])?;
        s.until(stage, "10_25", 2)?;
        s.offer(stage, &HOLDS[2], &mut card)?;
        s.countdown(stage, &HOLDS[2])?;

        // 10_26: it grows: the bars shoot past the window top, clipped with break marks, and the tallest is
        // labelled with the largest error; the coarse grid saw the zigzag's leftovers at the ends, and the
        // first and last coarse badges are outlined white.
        s.say(stage, "10_26")?;
        s.vanish(stage, &card, 0.4)?;
        s.clear_badges(stage, 0.4)?;
        s.set_bars(stage, &f.skip.mid, 1.0)?;
        let mut clipped = Vec::new();
        for (i, &value) in f.skip.mid.iter().enumerate() {
            if value.to_f64() > CLIP_Y - ZERO_Y {
                let (a, b) = (p(cx(i) - 0.3, CLIP_Y - 0.2), p(cx(i) + 0.3, CLIP_Y - 0.08));
                clipped.push(s.line(stage, a, b, hex(BACKGROUND), 6.0)?);
            }
        }
        let peak = tube::largest(&f.skip.mid);
        let tallest = f
            .skip
            .mid
            .iter()
            .position(|x| x.abs() == peak)
            .ok_or_else(|| error("chapter 10's skipped round has no largest bar".into()))?;
        let grown = fact("ch10.show.skip.err")?;
        clipped.push(s.text(stage, &grown, NOTE_SIZE, ink, p(cx(tallest), CLIP_Y + 0.22))?);
        s.appear(stage, &clipped, 0.4)?;
        s.until(stage, "10_26", 1)?;
        let skip_text = fact("ch10.show.skip.coarse.badges")?;
        let skip_badges = tube::neg(&f.skip.coarse_rhs);
        let mut skip_coarse = Vec::new();
        for (c, words) in skip_text.split(", ").enumerate() {
            let color = if skip_badges[c].is_zero() {
                Palette::green()
            } else {
                red
            };
            skip_coarse.push(box_view::frame(
                stage,
                p(pair_x(c), COARSE_Y),
                COARSE_W,
                BADGE_H,
                color,
                2.0,
            )?);
            skip_coarse.push(s.text(stage, words, COARSE_SIZE, color, p(pair_x(c), COARSE_Y))?);
        }
        s.appear(stage, &brackets, 0.4)?;
        s.appear(stage, &skip_coarse, 0.4)?;
        s.until(stage, "10_26", 2)?;
        let middle: Vec<Mob> = brackets[3..3 * (M - 1)].to_vec();
        s.pulse(stage, &middle, 0.8)?;
        s.until(stage, "10_26", 3)?;
        let ends = vec![
            plain_rect(stage, p(pair_x(0), COARSE_Y), COARSE_W + 0.14, BADGE_H + 0.14, ink, 2.6)?,
            plain_rect(
                stage,
                p(pair_x(M - 1), COARSE_Y),
                COARSE_W + 0.14,
                BADGE_H + 0.14,
                ink,
                2.6,
            )?,
        ];
        s.appear(stage, &ends, 0.4)?;

        // 10_27: the correct order replays in miniature: sweep, pair, coarse, lines, sweep.
        s.say(stage, "10_27")?;
        let mut gone = clipped.clone();
        gone.extend(skip_coarse.iter().copied());
        gone.extend(ends.iter().copied());
        s.vanish(stage, &gone, 0.4)?;
        s.set_bars(stage, &f.guess, 0.5)?;
        let words = ["sweep", "pair", "coarse", "lines", "sweep"];
        let step_x = |k: usize| PANEL_X + 1.3 * (k as f64 - (words.len() - 1) as f64 / 2.0);
        let mut steps = Vec::new();
        for (k, word) in words.iter().enumerate() {
            steps.push(s.text(stage, word, 24.0, dim, p(step_x(k), STEPS_Y))?);
        }
        for k in 0..words.len() - 1 {
            steps.push(box_view::arrow(
                stage,
                p(step_x(k) + 0.45, STEPS_Y),
                p(step_x(k + 1) - 0.45, STEPS_Y),
                dim,
                1.4,
            )?);
        }
        s.appear(stage, &steps, 0.4)?;
        s.play(stage)?;
        // A step lights by turning its own dim word to the hero colour in place (same text, size and position,
        // so only the paint changes) in the 0.2 s the fade-in of a second word over it took.
        let light = |s: &mut Ch10<'_>, stage: &mut Stage<'_>, steps: &[Mob], k: usize| -> fmn::Result<()> {
            let word = s.text(stage, words[k], 24.0, hero, p(step_x(k), STEPS_Y))?;
            s.morph(stage, steps[k], word, 0.2)
        };
        light(&mut s, stage, &steps, 0)?;
        for sweep in &f.history[1..=PRE] {
            s.set_bars(stage, sweep, 0.3)?;
            s.play(stage)?;
        }
        s.set_badges(stage, &TubeFacts::badges(&f.history[PRE]), 3, 0.3)?;
        s.until(stage, "10_27", 1)?;
        light(&mut s, stage, &steps, 1)?;
        s.appear(stage, &brackets, 0.3)?;
        s.pulse(stage, &brackets, 0.5)?;
        s.play(stage)?;
        light(&mut s, stage, &steps, 2)?;
        s.appear(stage, &coarse_mobs, 0.3)?;
        s.play(stage)?;
        light(&mut s, stage, &steps, 3)?;
        s.set_bars(stage, &one.mid, 0.6)?;
        s.play(stage)?;
        light(&mut s, stage, &steps, 4)?;
        s.set_bars(stage, &one.end, 0.5)?;
        s.set_badges(stage, &TubeFacts::badges(&one.end), 3, 0.4)?;
        // The replayed coarse sums belong to the badges before the lines; they fade with the final sweep,
        // inside its 0.5 s group, so they never sit under the near-zero badges as if current.
        s.vanish(stage, &coarse_mobs, 0.4)?;

        // 10_28: the catch: back at the guess, "right answer: 0" turns to "?" and the bars grey out; only
        // the red badges stay bright.
        s.say(stage, "10_28")?;
        let mut replay = steps.clone();
        replay.extend(coarse_mobs.iter().copied());
        replay.extend(brackets.iter().copied());
        s.vanish(stage, &replay, 0.5)?;
        s.set_bars(stage, &f.guess, 0.6)?;
        s.set_badges(stage, &TubeFacts::badges(&f.guess), 2, 0.4)?;
        s.until(stage, "10_28", 2)?;
        let unknown = s.text(stage, &answer_words("?"), NOTE_SIZE, ink, p(ANSWER_AT[0], ANSWER_AT[1]))?;
        s.morph(stage, answer, unknown, 0.6)?;
        for m in s.bars.clone() {
            s.refill(stage, m, dim, 0.5, 0.6)?;
        }
        s.until(stage, "10_28", 3)?;
        let row = s.badge_mobs();
        s.pulse(stage, &row, 0.8)?;

        // 10_29: the three-sweep state: small red badges, tall bars, and blue arrows on faces 1 to 8 at
        // 2.5 scene units per unit of flow, longest on face 7.
        s.say(stage, "10_29")?;
        let known = s.text(
            stage,
            &answer_words(&fact("ch10.show.truth")?),
            NOTE_SIZE,
            ink,
            p(ANSWER_AT[0], ANSWER_AT[1]),
        )?;
        s.morph(stage, answer, known, 0.5)?;
        s.set_bars(stage, &f.history[PRE], 0.8)?;
        s.set_badges(stage, &TubeFacts::badges(&f.history[PRE]), 3, 0.5)?;
        s.until(stage, "10_29", 1)?;
        let row = s.badge_mobs();
        s.pulse(stage, &row, 0.8)?;
        s.until(stage, "10_29", 2)?;
        s.pulse(stage, &s.bars.clone(), 0.8)?;
        s.until(stage, "10_29", 3)?;
        let mut flow_arrows = Vec::new();
        for (j, &u) in f.flow3.iter().enumerate() {
            let (x, half) = (fx(j + 1), FLOW_SCALE * u.to_f64() / 2.0);
            let arrow = box_view::arrow(
                stage,
                p(x - half, TUBE_Y),
                p(x + half, TUBE_Y),
                box_view::flow_color(),
                FLOW_THICKNESS,
            )?;
            s.grow(stage, arrow, 0.6)?;
            flow_arrows.push(arrow);
        }

        // 10_30: the badges stack cell by cell into a running total that matches each face arrow in turn.
        s.say(stage, "10_30")?;
        let note = s.text(
            stage,
            "face flow = sum of the badges from the wall",
            NOTE_SIZE,
            dim,
            p((TUBE_LEFT + fx(N)) / 2.0, RUN_NOTE_Y),
        )?;
        s.appear(stage, &[note], 0.4)?;
        s.until(stage, "10_30", 1)?;
        let mut totals = vec![note];
        for (j, &sum) in f.running_badges().iter().enumerate() {
            let words = s.text(
                stage,
                &decimals(sum, 3),
                BADGE_SIZE,
                box_view::flow_color(),
                p(fx(j + 1), RUN_Y),
            )?;
            let [frame, value] = s.badges[j];
            s.pulse(stage, &[frame, value, flow_arrows[j]], 0.4)?;
            s.appear(stage, &[words], 0.3)?;
            s.play(stage)?;
            totals.push(words);
        }

        // 10_31: a source card: Basilisk's defaults and its loop condition, as quoted.
        s.say(stage, "10_31")?;
        s.vanish(stage, &totals, 0.4)?;
        s.vanish(stage, &flow_arrows, 0.4)?;
        s.until(stage, "10_31", 1)?;
        let (first_half, second_half) = LOOP_CODE
            .split_once("&& ")
            .ok_or_else(|| error(format!("chapter 10 cannot break the loop line {LOOP_CODE:?}")))?;
        let source = vec![
            plain_rect(stage, p(PANEL_X, SOURCE_Y), ASK_W, 2.2, dim, 1.6)?,
            s.text(stage, "Basilisk src/poisson.h", 24.0, ink, p(PANEL_X, SOURCE_Y + 0.8))?,
            s.mono(
                stage,
                &format!(
                    "NITERMAX = {}, TOLERANCE = {}",
                    basilisk("bas.nitermax")?,
                    basilisk("bas.tolerance")?
                ),
                22.0,
                ink,
                p(PANEL_X, SOURCE_Y + 0.25),
            )?,
            s.mono(stage, &format!("{first_half}&&"), 20.0, dim, p(PANEL_X, SOURCE_Y - 0.3))?,
            s.mono(stage, second_half, 20.0, dim, p(PANEL_X, SOURCE_Y - 0.7))?,
        ];
        s.appear(stage, &source, 0.6)?;
        s.until(stage, "10_31", 2)?;
        s.pulse(stage, &source[2..3], 0.8)?;

        // 10_32: two check boxes under the question card; the card glows.
        s.say(stage, "10_32")?;
        let check_row = |s: &Ch10<'_>, stage: &mut Stage<'_>, y: f64, words: &str| -> fmn::Result<Vec<Mob>> {
            Ok(vec![
                plain_rect(stage, p(1.0, y), 0.28, 0.28, ink, 2.0)?,
                s.text_left(stage, words, 22.0, ink, p(1.3, y))?,
            ])
        };
        s.until(stage, "10_32", 2)?;
        let mut checks = check_row(&s, stage, 1.35, "did the solver leave only small imbalances?")?;
        s.appear(stage, &checks, 0.4)?;
        s.until(stage, "10_32", 3)?;
        let same = check_row(&s, stage, 0.9, "same picture when tightened or refined?")?;
        s.appear(stage, &same, 0.4)?;
        s.pulse(stage, &dock, 1.0)?;
        checks.extend(same);

        // 10_33: a ladder of tubes (8, 4, 2 and 1 cells): sums of residuals go down, straight-line
        // corrections come up; "multigrid".
        s.say(stage, "10_33")?;
        s.vanish(stage, &source, 0.4)?;
        s.vanish(stage, &checks, 0.4)?;
        s.until(stage, "10_33", 1)?;
        let mut ladder = Vec::new();
        let mut levels: Vec<usize> = Vec::new();
        for (k, &y) in LADDER_Y.iter().enumerate() {
            let cells = N >> k;
            let width = LADDER_LEN / cells as f64;
            for i in 0..cells {
                ladder.push(plain_rect(
                    stage,
                    p(LADDER_LEFT + width * (i as f64 + 0.5), y),
                    width,
                    0.36,
                    ink,
                    1.6,
                )?);
            }
            levels.push(cells);
            if let Some(&next) = LADDER_Y.get(k + 1) {
                ladder.push(box_view::arrow(stage, p(1.2, y - 0.22), p(1.2, next + 0.22), red, 2.0)?);
                let up_x = LADDER_LEFT + LADDER_LEN + 0.4;
                ladder.push(box_view::arrow(
                    stage,
                    p(up_x, next + 0.22),
                    p(up_x, y - 0.22),
                    teal,
                    2.0,
                )?);
            }
        }
        let mid_y = (LADDER_Y[0] + LADDER_Y[1]) / 2.0;
        ladder.push(s.text(stage, "sums", 20.0, red, p(0.75, mid_y))?);
        ladder.push(s.text(stage, "lines", 20.0, teal, p(LADDER_LEFT + LADDER_LEN + 0.95, mid_y))?);
        s.appear(stage, &ladder, 0.8)?;
        s.until(stage, "10_33", 2)?;
        let multigrid = s.text(stage, "multigrid", 34.0, hero, p(LADDER_LEFT + LADDER_LEN / 2.0, -2.1))?;
        s.appear(stage, &[multigrid], 0.5)?;

        // 10_34: each level's zigzag-sized mistake flashes in turn; the tunnel thumbnail's teaching grid
        // stays unchanged.
        s.say(stage, "10_34")?;
        for (&cells, &y) in levels.iter().zip(&LADDER_Y) {
            let width = LADDER_LEN / cells as f64;
            let mut zigzag = Vec::new();
            for k in 0..2 * cells {
                let (x0, x1) = (
                    LADDER_LEFT + width * k as f64 / 2.0,
                    LADDER_LEFT + width * (k + 1) as f64 / 2.0,
                );
                let (y0, y1) = if k % 2 == 0 {
                    (y - 0.12, y + 0.12)
                } else {
                    (y + 0.12, y - 0.12)
                };
                zigzag.push(s.line(stage, p(x0, y0), p(x1, y1), hero, 2.6)?);
            }
            s.appear(stage, &zigzag, 0.25)?;
            s.play(stage)?;
            s.vanish(stage, &zigzag, 0.35)?;
            s.play(stage)?;
        }
        s.until(stage, "10_34", 2)?;
        s.pulse(stage, &full[2..], 1.0)?;

        // 10_35: the four cards return; "correct" dims and "carry" glows; the tube fades.
        s.say(stage, "10_35")?;
        let keep: Vec<Mob> = full.iter().copied().chain(dock).collect();
        let mut everything = s.on.clone();
        everything.retain(|m| !keep.contains(m));
        s.vanish(stage, &everything, 0.8)?;
        let cards = step_cards(&s, stage, CORRECT)?;
        let card_mobs: Vec<Mob> = cards.iter().flatten().copied().collect();
        s.appear(stage, &card_mobs, 0.5)?;
        s.until(stage, "10_35", 1)?;
        let carry = step_cards(&s, stage, CARRY)?;
        for (now, next) in cards.iter().zip(&carry) {
            for (&from, &to) in now.iter().zip(next) {
                s.morph(stage, from, to, 0.6)?;
            }
        }
        s.play(stage)?;
        s.pulse(stage, &cards[CARRY], 0.8)?;
        s.play(stage)?;
        println!("Chapter 10 ends on the four step cards with carry lit, the thumbnail and the question card");
        s.k.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cues_holds_and_pins_line_up() {
        let ids: Vec<&str> = CUES.iter().map(|c| c.0).collect();
        for hold in &HOLDS {
            let after = ids
                .iter()
                .position(|&id| id == hold.after)
                .expect("the hold follows a cue");
            assert_eq!(
                ids.get(after + 1),
                Some(&hold.answer),
                "{} is answered by the next line",
                hold.id
            );
        }
        let f = TubeFacts::solve();
        let rows = f.fact_rows();
        assert_eq!(rows.len(), PINS.len());
        for ((key, value), (pin, expected, _)) in rows.iter().zip(PINS) {
            assert_eq!((key.as_str(), value.as_str()), (pin, expected));
        }
        assert!(facts().is_ok());
    }

    #[test]
    fn neighbours_average_is_the_sweep_target() {
        let f = TubeFacts::solve();
        let swept = tube::sweep(&f.guess, tube::weight());
        for i in 0..N {
            let (_, average) = neighbours(&f.guess, i);
            assert_eq!(
                swept[i],
                f.guess[i] + tube::weight() * (average - f.guess[i]),
                "cell {}",
                i + 1
            );
        }
    }
}
