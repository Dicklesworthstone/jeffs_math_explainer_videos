//! Chapter 15, "The seam between coarse and fine" (navier-full-script03.md cues 15_1 to 15_12 and the
//! hold 15_h1; BronzeJaguar's storyboard navier-adaptive-scene03.md, lines 134 to 180).
//!
//! The chapter opens on chapter 14's last picture, rebuilt from the same numbers: the big cell, the
//! split cell beside it, the white seam and the question strip. The big cell keeps that place and
//! scale to the end. Three rules follow. A seam keeps amounts only when the big face carries what the
//! small faces carry (15_1 to 15_5). A merge or a split keeps the dye total (15_6 and 15_7, in the side
//! panel). And a split cell's four children balance only after a pressure repair, which needs the
//! outside faces to add up (15_8 to 15_10, solved exactly by `four_cell`). Every number in those beats
//! is a toy value in box units, pinned to BronzeJaguar's oracle `ad01` (oracle-keys02.txt), and the
//! picture labels it "toy numbers".
//!
//! 15_12 is measured. Its panel draws the retained transfer04 run `transfer09` (one adaptive start-up
//! of the tunnel with 14 changes of grid): the largest cell imbalance just before each change, just
//! after it and after the run's own pressure repair, and the drag coefficient before and after. Those
//! values are the run's `transfers.csv` fields, embedded below as the printed strings. The chapter
//! ends on the seam (the big cell whole, the two small cells, their flows) beside that panel.
//!
//! Voice: the kit narrator. Each cue is `say`d before its pictures, which wait on its sentence starts
//! (`until`). The prediction 15_h1, after 15_3, holds a five-second countdown while the red "−½ per
//! step" badge and every reflux arrow are not yet drawn. A scratch-voice draft times the gates from
//! estimated line lengths, a narrated render from the takes' own sentence starts.

use std::cmp::Ordering;

use explainer_kit::{Facts, Kit, Palette, Timed, hex, v};
use fmn::animation::indicate;
use fmn::prelude::*;

use crate::box_view;
use crate::four_cell::{self, CELLS, Q, V4};

/// Chapter 15's cues in spoken order and the number of sentences in each (the narrator's gates).
/// Sentences end at `.`, `?` or `!` before a space, as in the kit; the check recounts them.
pub const CUES: [(&str, usize); 12] = [
    ("15_1", 4),
    ("15_2", 4),
    ("15_3", 3),
    ("15_4", 4),
    ("15_5", 3),
    ("15_6", 4),
    ("15_7", 3),
    ("15_8", 4),
    ("15_9", 2),
    ("15_10", 4),
    ("15_11", 4),
    ("15_12", 2),
];

/// Script HOLD 15_h1: the prediction countdown, in whole seconds.
pub const PREDICTION_SECONDS: u32 = 5;
/// 15_h1's question and answers, as the storyboard's holds table has them.
pub const QUESTION: &str = "The big cell loses 2½, the small cells gain 2: where does the extra half go?";
/// The same question as drawn, so that ½ is a typeset fraction.
const QUESTION_TEX: [&str; 2] = [
    r"\text{The big cell loses }2\tfrac{1}{2}\text{, the small cells gain }2\text{:}",
    r"\text{where does the extra half go?}",
];
pub const OPTIONS: [&str; 3] = ["into the small cells", "back to the big cell", "nowhere"];

// Chapter 14's last picture (chapter14.rs QUESTION_CARD, CARD_STRIP, SEAM_BIG, SCHEMATIC, SEAM and the
// 14_13 drawing); the check compares these with White's source.
const QUESTION_CARD: &str = "Does halving the cells change the answer?";
const CARD_STRIP: ([f64; 2], [f64; 2], f64) = ([3.65, 3.55], [6.4, 0.56], 21.0);
const SEAM_BIG: [f64; 4] = [-5.0, -2.6, -1.4, 1.0];
const SCHEMATIC: &str = "schematic";
const SEAM: &str = "seam";

const TOY_NOTE: &str = "toy numbers: small side h, big side 2h";
const MERGE_HEAD: &str = "dye levels (toy numbers)";
const MERGE_TAGS: [&str; 3] = ["four small cells", "merged: average", "copy one child"];
const SPLIT_TAGS: [&str; 3] = ["neighbour", "parent", "neighbour"];
const RULE_15_4: &str = "kept only if big face = small faces' total";
const LEDGER07_HEAD: &str = "ch. 07: one face, two cells";
const LEDGER07_NOTE: &str = "same face, opposite signs";
const PUZZLE_NOTE: &str = "ch. 09's pressure puzzle, on four cells";
const TRIES: [&str; 2] = ["inside: the straight-line guess", "inside: every face 0"];
const IN_WORD: &str = "1 in";
/// 15_11's cards, as the full script has them; the last is drawn with a typeset ≠.
pub const CARDS: [&str; 5] = [
    "seam: faces agree",
    "transfer: amounts kept",
    "balance: in = out",
    "geometry: wall shape (ch. 17)",
    "multigrid ≠ adaptive grid",
];
const CARD_LAST_TEX: &str = r"\text{multigrid} \neq \text{adaptive grid}";

/// Chapter 07's 07_5 ledger: the right face of its cell carries 3 (chapter07.rs `CELL[1]`).
const LEDGER07_FLOW: i64 = 3;

/// The Precisely lower thirds, each the full script's text split into lines (the check rejoins them).
const PRECISELY_15_4: [&str; 2] = [
    "AMReX documentation: 'the area-weighted fluxes from the fine grid advance do not in general match the underlying flux",
    "from the coarse grid face, resulting in a lack of global conservation.'",
];
const PRECISELY_15_5: [&str; 3] = [
    "Basilisk's restriction sets a coarse face to the mean of the two fine faces under it. AMReX: 'The remedy is to modify the solution",
    "in the coarse cells immediately next to the coarse-fine interface to account for the mismatch stored in the flux register.' Codes",
    "whose small cells take several shorter steps per big step must compare 'the area and time-weighted fine grid fluxes'.",
];
const PRECISELY_15_6: [&str; 2] = [
    "Basilisk's restriction makes the parent the average of its children (multigrid-common.h line 47). AMReX: 'This is",
    "simply corrected by overwriting covered coarse cells to be the average of the overlying fine cells.'",
];
const PRECISELY_15_7: [&str; 3] = [
    "Slope fill: child = parent ± a slope term, opposite in each pair (Basilisk multigrid-common.h line 231), so the total is kept for",
    "any slope. A smooth fill need not preserve that total. Keeping the total does not keep values in range:",
    "17/4 is above the parent and both neighbours. A limiter that flattens the slope at a peak gives 4 and 4.",
];
const PRECISELY_15_9: [&str; 4] = [
    "L p = d on the four children with the outer faces fixed and pA = 0: p = (0, −1/8, 1/8, 1/4). Basilisk does a local",
    "projection like this when it refines face velocities on trees, citing Popinet (2009), §3.3 (not read here). With",
    "embedded solids it uses plain face refinement instead. AMReX's FaceDivFree interpolation 'ensures the divergence of",
    "the fine ghost cells matches the value of the divergence of the underlying coarse cell.'",
];
const PRECISELY_15_12: [&str; 2] = [
    "With embedded solids, Basilisk refines face velocities with plain refine_face and uses an approximate projection. So",
    "balance after regridding has to be measured, not assumed.",
];

/// The Basilisk lines the Precisely thirds rest on, as BronzeJaguar's reader `ad02` found them in the
/// pinned tarball (basilisk-20261025.tar.gz): fact key, file and line, text.
pub const BASILISK: [(&str, &str, &str); 7] = [
    (
        "bas.restrict.average",
        "basilisk/src/grid/multigrid-common.h:47",
        "s[] = sum/(1 << dimension);",
    ),
    (
        "bas.face.average",
        "basilisk/src/grid/multigrid-common.h:69",
        "v.x[] = (fine(v.x,0,0) + fine(v.x,0,1))/2.;",
    ),
    (
        "bas.linear.fill",
        "basilisk/src/grid/multigrid-common.h:231",
        "s[] += child.x*g.x*cm[-child.x]/cmc;",
    ),
    (
        "bas.solenoidal.comment",
        "basilisk/src/navier-stokes/centered.h:158",
        "On trees, refinement of the face-centered velocity field needs to",
    ),
    (
        "bas.solenoidal.set",
        "basilisk/src/navier-stokes/centered.h:162",
        "uf.x.refine = refine_face_solenoidal;",
    ),
    (
        "bas.solenoidal.embed",
        "basilisk/src/navier-stokes/centered.h:169",
        "uf.x.refine = refine_face;",
    ),
    (
        "bas.solenoidal.local",
        "basilisk/src/grid/tree-common.h:579",
        "// local projection, see section 3.3 of Popinet, JCP, 2009",
    ),
];

/// The AMReX documentation sentences quoted on screen, as `ad02` recorded them (oracle-keys02.txt).
pub const AMREX: [(&str, &str); 6] = [
    (
        "amrex.flux.mismatch",
        "the area-weighted fluxes from the fine grid advance do not in general match the underlying flux from the coarse grid face, resulting in a lack of global conservation.",
    ),
    (
        "amrex.subcycling",
        "the coarse grid flux must be compared to the area and time-weighted fine grid fluxes.",
    ),
    (
        "amrex.conservation.rule",
        "Conservation only happens if the sum of the (area and time-weighted) fine fluxes equals the coarse flux, which in general is not true.",
    ),
    (
        "amrex.reflux.remedy",
        "The remedy is to modify the solution in the coarse cells immediately next to the coarse-fine interface to account for the mismatch stored in the flux register",
    ),
    (
        "amrex.average.down",
        "This is simply corrected by overwriting covered coarse cells to be the average of the overlying fine cells.",
    ),
    (
        "amrex.facedivfree",
        "ensures the divergence of the fine ghost cells matches the value of the divergence of the underlying coarse cell.",
    ),
];

// ------------------------------------------------------------------------------ the measured run

/// The run 15_12 draws: navier-solver-transfer04, case transfer09 (transfer09-verification.json:
/// steady inflow, adaptive, finest level 9, 0.25 s; summary.json: 14 regrids, no failed solve).
pub const RUN: &str = "transfer09";
pub const RUN_LEVEL: u32 = 9;
pub const RUN_SECONDS: f64 = 0.25;
pub const TRANSFERS_SHA256: &str = "9e094c7b1576201b";

/// transfer09/transfers.csv, one row per change of grid, eight of its nineteen fields exactly as
/// printed (`%.17g`): t, raw_div_before, raw_div_after, raw_div_corrected, drag_before, drag_after,
/// lift_before and lift_after. raw_div is the largest |net outflow| of a wet cell per unit area and
/// second (tunnel.c `flow_diagnostics`); drag and lift are force coefficients (`force_coefficients`).
/// "before" is read before `adapt_wavelet`, "after" after it, and "corrected" after the run's
/// `project_transfer` pressure solve over the whole grid.
pub const CHANGES: [[&str; 8]; 14] = [
    [
        "0.0065074343434343438",
        "7.0083038226920191e-07",
        "1.9309012565181964",
        "5.861673493992164e-07",
        "20.524756058155734",
        "20.524756058174312",
        "-0.014806628724132074",
        "-0.014806628728032546",
    ],
    [
        "0.019523355555555559",
        "5.0475947917932862e-07",
        "2.7780470918769811",
        "4.9792644970742037e-07",
        "13.621095007292935",
        "13.621095007292935",
        "-0.0053304797460921082",
        "-0.0053304797460921082",
    ],
    [
        "0.035416666666666666",
        "4.6809180628826219e-07",
        "2.0202979714704075",
        "4.6573668876653998e-07",
        "10.001895756076213",
        "10.001895756076213",
        "-0.0022304672996903705",
        "-0.0022304672996903705",
    ],
    [
        "0.052272727272727276",
        "4.2978631417667224e-07",
        "2.2595363971288633",
        "3.6346492997768877e-07",
        "8.2367598831758642",
        "8.2367598831758642",
        "-0.0011357656384433854",
        "-0.0011357656384433854",
    ],
    [
        "0.070454545454545464",
        "3.7949629133615659e-07",
        "2.8296749763719924",
        "3.8725694826173464e-07",
        "7.231815741285927",
        "7.231815741285927",
        "-0.0010493528747933479",
        "-0.0010493528747933479",
    ],
    [
        "0.088636363636363652",
        "4.1839736258930671e-07",
        "3.0750565879533278",
        "3.946699481116589e-07",
        "6.6218671982585038",
        "6.6218671982585038",
        "-0.0043378843694402104",
        "-0.0043378843694402104",
    ],
    [
        "0.10681818181818183",
        "3.8627683534436796e-07",
        "3.9082022958882452",
        "3.9677775460048801e-07",
        "6.2183915506766967",
        "6.2183915506766967",
        "-0.0030130449303114545",
        "-0.0030130449303114545",
    ],
    [
        "0.125",
        "3.787749523753714e-07",
        "3.9513222329127795",
        "4.2733905595236406e-07",
        "5.9327518584409393",
        "5.9327518584409393",
        "-0.0021648553312256249",
        "-0.0021648553312256249",
    ],
    [
        "0.14318181818181819",
        "3.4938543135021316e-07",
        "3.8645353895814596",
        "3.8576385606561281e-07",
        "5.7205663531559034",
        "5.7205663531559034",
        "0.00028983018437391174",
        "0.00028983018437391174",
    ],
    [
        "0.16136363636363635",
        "4.3689513153088999e-07",
        "0.67601986532051928",
        "4.0466269768359054e-07",
        "5.5641266868863237",
        "5.5628835216591783",
        "0.0018170055341699385",
        "0.0018197386374383009",
    ],
    [
        "0.17954545454545451",
        "3.9171650142305614e-07",
        "0.67799970403343002",
        "3.9459500871043205e-07",
        "5.454719502380132",
        "5.4517530523494839",
        "0.0028229457111840746",
        "0.002824976787173054",
    ],
    [
        "0.1977272727272727",
        "3.7180648213008816e-07",
        "0.68077491588117511",
        "3.7753691056967756e-07",
        "5.3651105444226204",
        "5.3651105444226204",
        "0.0029188293970480402",
        "0.0029188293970480402",
    ],
    [
        "0.21590909090909088",
        "3.2191707546175286e-07",
        "0.68467101670046659",
        "3.6273501495264705e-07",
        "5.2990415455997617",
        "5.2989790452589594",
        "0.0029823529749516417",
        "0.0029824719064717397",
    ],
    [
        "0.23409090909090907",
        "2.7197238872727181e-07",
        "0.68853406068497347",
        "3.9362507029011558e-07",
        "5.2537778761868275",
        "5.2537778761868275",
        "0.0031675029778215676",
        "0.0031675029778215676",
    ],
];

const T: usize = 0;
const BEFORE: usize = 1;
const AFTER: usize = 2;
const REPAIRED: usize = 3;
const DRAG_BEFORE: usize = 4;
const DRAG_AFTER: usize = 5;
const LIFT_BEFORE: usize = 6;
const LIFT_AFTER: usize = 7;

const MEASURED_HEAD: &str = "measured: one adaptive run of the tunnel";
const MEASURED_SUB: &str = "run transfer09: 0.25 s of start-up, level 9, 14 grid changes";
const KICK_AXIS: &str = "max |net flux| / cell area (1/s)";
const DRAG_AXIS: &str = "drag coefficient on the cylinder (start-up)";

// ----------------------------------------------------------------------------------- layout

const SIDE_X: f64 = 3.65;
const PRECISELY_Y: f64 = -3.3;
const PREDICT_AT: [f64; 2] = [3.65, 0.2];
const PREDICT_SIZE: [f64; 2] = [6.4, 2.75];
/// Scene units of arrow per unit of flow, and the shortest arrow drawn (only 1/8 needs it).
const PER_FLOW: f64 = 0.8;
const MIN_ARROW: f64 = 0.25;
const FLOW_SIZE: f64 = 24.0;
const SMALL_SIZE: f64 = 22.0;
/// The y of the seam's midpoint (the seam itself is chapter 14's `SEAM_BIG` right edge, x = −2.6).
const MID_Y: f64 = -0.2;
/// The two small cells' centres (lower, upper); their faces on the seam carry 3/2 and 1/2.
const SMALL: [[f64; 2]; 2] = [[-2.0, -0.8], [-2.0, 0.4]];
/// Where the small faces' labels sit: below the lower arrow, above the upper one.
const SMALL_LABEL_AT: [[f64; 2]; 2] = [[-2.0, -1.08], [-2.0, 0.68]];
/// The big cell's own view of its right face, and the consistent face of cure 1: an arrow on the big
/// cell's side that stops just short of the seam (the midpoint tick stays visible).
const OWN_Y: f64 = -0.2;
const OWN_END: f64 = -2.66;
const OWN_LABEL_Y: f64 = 0.08;
/// Cure 2's reflux arrow: from the seam back into the big cell.
const REFLUX_Y: f64 = -0.62;
const REFLUX_LABEL_AT: [f64; 2] = [-3.85, -0.62];
/// The leak badge, under the seam's lower end.
const LEAK_AT: [f64; 2] = [-2.6, -1.85];
const LEAK_SIZE: [f64; 2] = [2.2, 0.46];
const TAG_AT: [f64; 2] = [-7.0, 2.3];
const TOY_AT: [f64; 2] = [-7.0, 1.9];
/// The split box's children A, B, C, D (four_cell::CELL_AT order), each 1 × 1 in box units.
const KIDS: [[f64; 2]; 4] = [[-4.4, -0.8], [-3.2, -0.8], [-4.4, 0.4], [-3.2, 0.4]];
/// Inside faces AB, CD, AC, BD (four_cell::FACES order): midpoint and positive direction.
const FACE_MID: [[f64; 2]; 4] = [[-3.8, -0.8], [-3.8, 0.4], [-4.4, -0.2], [-3.2, -0.2]];
const FACE_DIR: [[f64; 2]; 4] = [[1.0, 0.0], [1.0, 0.0], [0.0, 1.0], [0.0, 1.0]];
/// Inside faces' value labels, beside each face's line and never on it (box_view's rule): AB and CD
/// sit in their "to" cells, B and D, clear of the A|B and C|D line at x = −3.8.
const INNER_LABEL_AT: [[f64; 2]; 4] = [[-3.5, -1.12], [-3.5, 0.72], [-4.1, -0.45], [-3.5, -0.45]];
const LETTER_AT: [[f64; 2]; 4] = [[-4.75, -1.12], [-2.85, -1.12], [-4.75, 0.72], [-2.85, 0.72]];
/// The pressures, in each child's corner nearest the middle line but clear of the AC and BD arrows.
const PRESSURE_AT: [[f64; 2]; 4] = [[-4.72, -0.55], [-2.88, -0.55], [-4.72, 0.15], [-2.88, 0.15]];
const BADGE_AT: [[f64; 2]; 4] = [[-4.4, -1.8], [-3.2, -1.8], [-4.4, 1.4], [-3.2, 1.4]];
const BADGE_SIZE: [f64; 2] = [0.66, 0.42];
/// The left face's inflow into A and C: arrows end on the big cell's left edge.
const INFLOW_START: f64 = -5.8;
const INFLOW_LABEL_X: f64 = -6.25;
/// The "seam" label moves up while the C and D badges sit above the box (15_8 to 15_10).
const SEAM_LABEL_LOW: [f64; 2] = [-2.6, 1.35];
const SEAM_LABEL_HIGH: [f64; 2] = [-2.6, 1.85];
/// Side readouts of 15_2 to 15_5.
const READOUT_Y: [f64; 5] = [2.7, 2.2, 1.55, 0.9, 0.35];
const LEDGER07_AT: [f64; 2] = [3.65, 0.75];
/// 15_6: the four children, the merged cell and the copied ghost, in the side panel.
const MERGE_X: [f64; 3] = [1.75, 4.0, 5.95];
const MERGE_Y: f64 = 1.6;
const MERGE_SIDE: f64 = 1.4;
const TOTAL_Y: f64 = 0.6;
const TAG_Y: f64 = 2.55;
const HEAD_Y: f64 = 3.0;
/// 15_7: the parent between its neighbours.
const SPLIT_X: [f64; 3] = [1.95, 3.65, 5.35];
const SPLIT_Y: f64 = 1.55;
const SPLIT_SIDE: f64 = 1.5;
/// 15_9 to 15_11 side lines.
const SIDE_Y: [f64; 4] = [2.6, 2.05, 1.45, 0.85];
const SUM_BOX_AT: [f64; 2] = [3.65, 1.5];
const SUM_BOX_SIZE: [f64; 2] = [3.6, 0.62];
const TRY_Y: f64 = 0.7;
const CARD_Y: [f64; 5] = [2.75, 2.13, 1.51, 0.89, 0.27];
const CARD_SIZE: [f64; 2] = [6.0, 0.5];
/// 15_12's panel: one time axis for both plots, t = 0 to 0.25 s.
const AXIS_X: [f64; 2] = [0.9, 6.7];
const KICK_BASE: f64 = 0.55;
const KICK_SCALE: f64 = 0.38;
const KICK_LABEL_Y: f64 = 2.3;
const BAR_WIDTH: f64 = 0.09;
const MEASURED_Y: [f64; 2] = [2.98, 2.66];
const KICK_READOUT_Y: [f64; 3] = [0.18, -0.14, -0.46];
const DRAG_LABEL_Y: f64 = -0.92;
const DRAG_BASE: f64 = -2.8;
const DRAG_SCALE: f64 = 0.07;
const DRAG_READOUT_AT: [[f64; 2]; 2] = [[5.0, -1.45], [5.0, -1.8]];
const TIME_LABEL_Y: f64 = -3.0;

const FADE: f64 = 0.5;
const SWAP: f64 = 0.6;
const PULSE: f64 = 0.8;
const BEAT: f64 = 0.4;

fn card_fill() -> Srgb {
    hex("#182433")
}

/// White, the teaching highlight (the seam).
fn glow() -> Srgb {
    hex("#FFFFFF")
}

/// Rose, the dye role of chapters 11 to 13.
fn dye() -> Srgb {
    hex("#F49BBE")
}

/// Pressure above zero orange (box_view), below zero teal, zero dim.
fn pressure_color(p: Q) -> Srgb {
    match p.cmp(&Q::ZERO) {
        Ordering::Greater => box_view::push_color(),
        Ordering::Less => Palette::teal(),
        Ordering::Equal => Palette::dim(),
    }
}

const WHITE_RGB: [f32; 3] = [1.0, 1.0, 1.0];
const RED_RGB: [f32; 3] = [0.988, 0.384, 0.333];
const BLUE_RGB: [f32; 3] = [0.345, 0.769, 0.867];
const GREEN_RGB: [f32; 3] = [0.514, 0.757, 0.404];

fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

pub struct Chapter15 {
    pub kit: Kit,
}

// ------------------------------------------------------------------------------- toy numbers

/// Lengths in box units: a small face, a big face.
const FINE_FACE: i64 = 1;
const COARSE_FACE: i64 = 2;
/// 15_6: the four children's dye levels in reading order (upper left, upper right, lower left, lower
/// right), each child of area 1.
const DYE: [i64; 4] = [1, 3, 2, 6];
/// 15_7: left neighbour, parent, right neighbour.
const ROW: [i64; 3] = [0, 4, 2];

/// Flows to the right through the seam's small faces each step: lower, upper.
pub fn fine_faces() -> [Q; 2] {
    [Q::new(3, 2), Q::new(1, 2)]
}

pub fn fine_total() -> Q {
    let [lower, upper] = fine_faces();
    lower + upper
}

/// What the big cell computes for its own right face from its coarser view (15_3).
pub fn coarse_own() -> Q {
    Q::new(5, 2)
}

/// Cure 1: the big face's speed is the mean of the small faces' speeds.
pub fn consistent_speed() -> Q {
    let [lower, upper] = fine_faces();
    let fine = Q::int(FINE_FACE);
    (lower / fine + upper / fine) / Q::int(2)
}

pub fn consistent_flow() -> Q {
    consistent_speed() * Q::int(COARSE_FACE)
}

/// What disappears at the seam each step when the big cell keeps its own view.
pub fn leak() -> Q {
    coarse_own() - fine_total()
}

/// Cure 2: the big cell's loss after the difference is paid back.
pub fn refluxed_loss() -> Q {
    coarse_own() - leak()
}

pub fn merge_total_before() -> Q {
    DYE.iter().fold(Q::ZERO, |s, &d| s + Q::int(d))
}

/// The merged cell's level: the children's average.
pub fn merge_parent() -> Q {
    merge_total_before() / Q::int(4)
}

pub fn merge_total_after() -> Q {
    merge_parent() * Q::int(4)
}

/// Copying one child (the largest, 6) over the whole area instead.
pub fn merge_copy_total() -> Q {
    Q::int(DYE.iter().copied().max().unwrap_or(0)) * Q::int(4)
}

/// Slope fill: each child is the parent plus or minus a quarter of the centred slope (child centres
/// sit a quarter of the parent's width from its middle). Left child, right child.
pub fn slope_children() -> [Q; 2] {
    let [left, parent, right] = ROW.map(Q::int);
    let slope = (right - left) / Q::int(2);
    let step = slope / Q::int(4);
    [parent - step, parent + step]
}

/// Four children of area 1: two of each value.
pub fn slope_total() -> Q {
    let [l, r] = slope_children();
    Q::int(2) * (l + r)
}

pub fn parent_total() -> Q {
    Q::int(ROW[1]) * Q::int(4)
}

/// The minmod-limited slope: zero at a peak, so the children equal the parent.
pub fn limited_children() -> [Q; 2] {
    let [left, parent, right] = ROW.map(Q::int);
    let (a, b) = (parent - left, right - parent);
    let slope = if a * b <= Q::ZERO {
        Q::ZERO
    } else if a.abs() < b.abs() {
        a
    } else {
        b
    };
    let step = slope / Q::int(4);
    [parent - step, parent + step]
}

/// The split box's outside faces, as each child's net outflow through them (A, B, C, D): the left
/// face's inflow, the seam's agreed total, splits evenly into A and C; B and D send the small faces'
/// flows on.
pub fn outer() -> V4 {
    let [lower, upper] = fine_faces();
    let half = fine_total() / Q::int(2);
    [-half, lower, -half, upper]
}

/// 15_10: the seam disagrees, so B's outer face carries the big cell's 5/2 less D's 1/2.
pub fn outer_bad() -> V4 {
    let [_, upper] = fine_faces();
    let mut o = outer();
    o[1] = coarse_own() - upper;
    o
}

/// The straight-line guess: the left and right faces both move air at speed 1 (2 across a face 2
/// long), so the line between them gives AB = CD = 1 and nothing up or down.
pub fn inner_line() -> V4 {
    let speed = |flow: Q| flow / Q::int(COARSE_FACE);
    let mid = (speed(fine_total()) + speed(consistent_flow())) / Q::int(2);
    let mid = mid * Q::int(FINE_FACE);
    [mid, mid, Q::ZERO, Q::ZERO]
}

/// Each child's net outflow: outside faces plus inside faces.
pub fn imbalance(outside: &V4, inside: &V4) -> V4 {
    four_cell::add(outside, &four_cell::div(inside))
}

/// L p = d with d = −div u* and pA = 0 (chapter 09's sign convention).
pub fn pressure() -> V4 {
    let d = four_cell::neg(&imbalance(&outer(), &inner_line()));
    four_cell::solve_pinned(&four_cell::matrix(), &d, four_cell::PIN)
}

/// u = u* − grad p on the inside faces.
pub fn inner_fixed() -> V4 {
    four_cell::sub(&inner_line(), &four_cell::grad(&pressure()))
}

/// 15_10's two tries with the bad seam: the straight-line guess, then every inside face 0.
pub fn tries() -> [V4; 2] {
    [inner_line(), [Q::ZERO; 4]]
}

/// The drawn length of a flow arrow: proportional, with a floor so 1/8 still shows.
fn drawn(value: Q) -> f64 {
    (value.abs().to_f64() * PER_FLOW).max(MIN_ARROW)
}

/// `+\tfrac{1}{2}` for a positive value, as the badges print imbalance.
fn signed(q: Q) -> String {
    if q > Q::ZERO { format!("+{}", q.tex()) } else { q.tex() }
}

/// `a + b - c = s` for the outside faces, in A, B, C, D order.
fn sum_tex(values: &V4) -> String {
    let mut out = values[0].tex();
    for &q in &values[1..] {
        if q < Q::ZERO {
            out.push_str(&format!(" - {}", q.abs().tex()));
        } else {
            out.push_str(&format!(" + {}", q.tex()));
        }
    }
    let total = four_cell::sum(values);
    format!("{out} = {}", total.tex())
}

/// `(a, b, c, d)` with each rational as `four_cell::Q` prints it, the oracle's tuple form.
fn tuple(values: impl IntoIterator<Item = Q>) -> String {
    let parts: Vec<String> = values.into_iter().map(|q| q.to_string()).collect();
    format!("({})", parts.join(", "))
}

// ------------------------------------------------------------------------------- measured numbers

fn field(row: usize, col: usize) -> f64 {
    CHANGES[row][col].parse::<f64>().unwrap_or(f64::NAN)
}

/// The printed field of column `col` with the largest (or smallest) value.
fn extreme(col: usize, largest: bool) -> &'static str {
    let mut best = 0;
    for row in 1..CHANGES.len() {
        let better = if largest {
            field(row, col) > field(best, col)
        } else {
            field(row, col) < field(best, col)
        };
        if better {
            best = row;
        }
    }
    CHANGES[best][col]
}

/// The row whose drag coefficient moved most at its change of grid.
fn largest_drag_jump() -> usize {
    let jump = |r: usize| (field(r, DRAG_AFTER) - field(r, DRAG_BEFORE)).abs();
    (1..CHANGES.len()).fold(0, |best, r| if jump(r) > jump(best) { r } else { best })
}

fn drag_jump(row: usize) -> f64 {
    (field(row, DRAG_AFTER) - field(row, DRAG_BEFORE)).abs()
}

fn lift_jump_max() -> f64 {
    (0..CHANGES.len())
        .map(|r| (field(r, LIFT_AFTER) - field(r, LIFT_BEFORE)).abs())
        .fold(0.0, f64::max)
}

/// Rows whose drag coefficient printed the same before and after the change.
fn drag_unchanged() -> usize {
    CHANGES.iter().filter(|r| r[DRAG_BEFORE] == r[DRAG_AFTER]).count()
}

/// `a to b`, both scaled by 10^`power` and rounded to `digits` places.
fn range(col: usize, power: i32, digits: usize) -> String {
    let scale = 10f64.powi(power);
    let lo = field_named(extreme(col, false)) * scale;
    let hi = field_named(extreme(col, true)) * scale;
    format!("{lo:.digits$} to {hi:.digits$}")
}

fn field_named(s: &str) -> f64 {
    s.parse::<f64>().unwrap_or(f64::NAN)
}

/// The same range for Tex, with the word upright and spaced.
fn tex_range(col: usize, power: i32, digits: usize) -> String {
    range(col, power, digits).replace(" to ", r"\text{ to }")
}

// ------------------------------------------------------------------------------------ labels

/// Every number-bearing label the chapter draws.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    pub small_sum: String,
    pub big_view: String,
    pub leak: String,
    pub mended: String,
    pub cure_one: String,
    pub cure_two: String,
    pub merge_total: String,
    pub merged_total: String,
    pub copy_total: String,
    pub split_children: String,
    pub split_parent: String,
    pub pressures: String,
    pub outside_good: String,
    pub outside_bad: String,
    pub bad_sum: String,
    pub before: String,
    pub after: String,
    pub repaired: String,
    pub drag_same: String,
    pub drag_jump: String,
}

pub fn labels() -> Labels {
    let [lower, upper] = fine_faces();
    let [l, r] = slope_children();
    let p = pressure();
    let jump = largest_drag_jump();
    Labels {
        small_sum: format!(
            r"\text{{small faces: }}{} + {} = {}",
            lower.tex(),
            upper.tex(),
            fine_total().tex()
        ),
        big_view: format!(r"\text{{big cell's own view: }}{}", coarse_own().tex()),
        leak: format!(r"{}\ \text{{lost per step}}", leak().tex()),
        mended: format!(r"{}\ \text{{lost per step}}", (refluxed_loss() - fine_total()).tex()),
        cure_one: format!(
            r"\text{{cure 1: mean speed }}{} \times \text{{length }}{} = {}",
            consistent_speed().tex(),
            COARSE_FACE,
            consistent_flow().tex()
        ),
        cure_two: format!(
            r"\text{{cure 2: reflux, pay }}{}\text{{ back to the big cell}}",
            leak().tex()
        ),
        merge_total: format!(
            "{} = {}",
            DYE.map(|d| d.to_string()).join(" + "),
            merge_total_before().tex()
        ),
        merged_total: format!(r"{} \times 4 = {}", merge_parent().tex(), merge_total_after().tex()),
        copy_total: format!(
            r"{} \times 4 = {}",
            DYE.iter().copied().max().unwrap_or(0),
            merge_copy_total().tex()
        ),
        split_children: format!(
            r"\text{{children: }}{l} + {r} + {l} + {r} = {}",
            slope_total().tex(),
            l = l.tex(),
            r = r.tex()
        ),
        split_parent: format!(r"\text{{parent: }}{} \times 4 = {}", ROW[1], parent_total().tex()),
        pressures: format!(
            r"p = ({},\ {},\ {},\ {})",
            p[0].tex(),
            p[1].tex(),
            p[2].tex(),
            p[3].tex()
        ),
        outside_good: format!(r"\text{{outside faces: }}{}", sum_tex(&outer())),
        outside_bad: format!(r"\text{{outside faces: }}{}", sum_tex(&outer_bad())),
        bad_sum: format!("A + B + C + D = {}", four_cell::sum(&outer_bad()).tex()),
        before: format!(
            r"\text{{just before each change: }}{} \times 10^{{-7}}",
            tex_range(BEFORE, 7, 1)
        ),
        after: format!(r"\text{{just after it: }}{}", tex_range(AFTER, 0, 2)),
        repaired: format!(
            r"\text{{after the run's own pressure repair: }}{} \times 10^{{-7}}",
            tex_range(REPAIRED, 7, 1)
        ),
        drag_same: format!(
            r"\text{{unchanged at {} of {} changes}}",
            drag_unchanged(),
            CHANGES.len()
        ),
        drag_jump: format!(
            r"\text{{largest jump }}{:.3}\text{{ of }}{:.2}",
            drag_jump(jump),
            field(jump, DRAG_BEFORE)
        ),
    }
}

// ------------------------------------------------------------------------------------- facts

/// The chapter's numbers alone, checked (a preview's `facts`).
pub fn facts() -> Result<Facts, String> {
    let mut facts = Facts::new();
    register(&mut facts)?;
    facts.self_check()?;
    Ok(facts)
}

/// Add chapter 15's numbers to a film's registry. The toy values are pinned to the exact value
/// BronzeJaguar's oracle `ad01` printed (navier-adaptive-scene02/runs/oracle-keys02.txt); the measured
/// values to transfer09's printed fields and the two receipts.
pub fn register(facts: &mut Facts) -> Result<(), String> {
    let ad01 = "ad01-run01 (navier-adaptive-scene02 oracle-keys02.txt)";
    let csv = "navier-solver-transfer04/transfer09/transfers.csv 9e094c7b";
    let run = "navier-solver-transfer04/transfer09-verification.json 99a2341d";
    let shown = "navier-full-script03.md chapter 15 picture";
    let [lower, upper] = fine_faces();
    let [l, r] = slope_children();
    let [cl, cr] = limited_children();
    let guess = inner_line();
    let fixed = inner_fixed();
    let p = pressure();
    let before = imbalance(&outer(), &guess);
    let after = imbalance(&outer(), &fixed);
    let zero = |v: &V4| v.iter().all(|q| q.is_zero());
    let solves = four_cell::apply(&four_cell::matrix(), &p) == four_cell::neg(&before) && p[four_cell::PIN].is_zero();
    let bad_sums_hold = tries()
        .iter()
        .chain([&fixed])
        .all(|inside| four_cell::sum(&imbalance(&outer_bad(), inside)) == leak());
    let jump = largest_drag_jump();
    let l15 = labels();
    facts
        .pin("seam.fine.bottom", lower, "3/2", ad01)?
        .pin("seam.fine.top", upper, "1/2", ad01)?
        .pin("seam.fine.total", fine_total(), "2", ad01)?
        .pin("seam.coarse.velocity.consistent", consistent_speed(), "1", ad01)?
        .pin("seam.consistent.flow", consistent_flow() == fine_total(), "true", ad01)?
        .pin("seam.coarse.own.flow", coarse_own(), "5/2", ad01)?
        .pin("seam.leak.per.step", leak(), "1/2", ad01)?
        .pin("seam.reflux.fixes", refluxed_loss() == fine_total(), "true", ad01)?
        .pin("merge.parent", merge_parent(), "3", ad01)?
        .pin("merge.total.before", merge_total_before(), "12", ad01)?
        .pin("merge.total.after", merge_total_after(), "12", ad01)?
        .pin("merge.copy.total", merge_copy_total(), "24", ad01)?
        .pin("split.slope.left", l, "15/4", ad01)?
        .pin("split.slope.right", r, "17/4", ad01)?
        .pin("split.slope.total", slope_total(), "16", ad01)?
        .pin("split.parent.total", parent_total(), "16", ad01)?
        .pin("split.fill.left", l, "15/4", ad01)?
        .pin("split.fill.right", r, "17/4", ad01)?
        .pin(
            "split.slope.overshoot",
            ROW.iter().all(|&x| r > Q::int(x)),
            "true",
            ad01,
        )?
        .pin("split.limited.child", cl, "4", ad01)?
        .pin(
            "split.limited.flat",
            cl == cr && Q::int(2) * (cl + cr) == parent_total(),
            "true",
            ad01,
        )?
        .pin(
            "split.coarse.balanced",
            four_cell::sum(&outer()).is_zero(),
            "true",
            ad01,
        )?
        .pin("split.inner.line", tuple(guess), "(1, 1, 0, 0)", ad01)?
        .pin("split.imbalance.line", tuple(before), "(0, 1/2, 0, -1/2)", ad01)?
        .pin(
            "split.imbalance.size",
            before.iter().map(|q| q.abs()).max().unwrap_or(Q::ZERO),
            "1/2",
            ad01,
        )?
        .pin("split.box.solves", solves, "true", ad01)?
        .pin("split.p", tuple(p), "(0, -1/8, 1/8, 1/4)", ad01)?
        .pin("split.inner.fixed", tuple(fixed), "(9/8, 7/8, -1/8, -3/8)", ad01)?
        .pin("split.fixed.AB", fixed[0], "9/8", ad01)?
        .pin("split.fixed.CD", fixed[1], "7/8", ad01)?
        .pin("split.fixed.AC", fixed[2], "-1/8", ad01)?
        .pin("split.fixed.BD", fixed[3], "-3/8", ad01)?
        .pin("split.fixed.balanced", zero(&after), "true", ad01)?
        .pin(
            "split.loop.kept",
            four_cell::loop_sum(&guess).is_zero() && four_cell::loop_sum(&fixed).is_zero(),
            "true",
            ad01,
        )?
        .pin("split.seam.bad.sum", four_cell::sum(&outer_bad()), "1/2", ad01)?
        .pin(
            "split.seam.bad.unsolvable",
            bad_sums_hold && !leak().is_zero(),
            "true",
            ad01,
        )?
        .pin(
            "ch15.ledger07.flow",
            LEDGER07_FLOW,
            "3",
            "src/story/chapter07.rs CELL[1] (07_5: Left cell +3, Right cell −3)",
        )?
        .pin("transfer09.level", RUN_LEVEL, "9", run)?
        .pin("transfer09.duration", RUN_SECONDS, "0.25", run)?
        .pin(
            "transfer09.changes",
            CHANGES.len(),
            "14",
            "navier-solver-transfer04/transfer09/summary.json 748b8780 regrids",
        )?
        .pin("transfer09.kick.max", extreme(AFTER, true), "3.9513222329127795", csv)?
        .pin("transfer09.kick.min", extreme(AFTER, false), "0.67601986532051928", csv)?
        .pin(
            "transfer09.before.max",
            extreme(BEFORE, true),
            "7.0083038226920191e-07",
            csv,
        )?
        .pin(
            "transfer09.before.min",
            extreme(BEFORE, false),
            "2.7197238872727181e-07",
            csv,
        )?
        .pin(
            "transfer09.repaired.max",
            extreme(REPAIRED, true),
            "5.861673493992164e-07",
            csv,
        )?
        .pin(
            "transfer09.repaired.min",
            extreme(REPAIRED, false),
            "3.6273501495264705e-07",
            csv,
        )?
        .pin(
            "transfer09.repair.lowers.all",
            (0..CHANGES.len()).all(|r| field(r, REPAIRED) < field(r, AFTER)),
            "true",
            csv,
        )?
        .pin("transfer09.drag.same", drag_unchanged(), "10", csv)?
        .pin(
            "transfer09.drag.jump.max",
            drag_jump(jump),
            "0.0029664500306481045",
            csv,
        )?
        .pin("transfer09.drag.jump.t", CHANGES[jump][T], "0.17954545454545451", csv)?
        .pin(
            "transfer09.lift.jump.max",
            lift_jump_max(),
            "0.0000027331032683623833",
            csv,
        )?
        .pin("transfer09.before.shown", range(BEFORE, 7, 1), "2.7 to 7.0", shown)?
        .pin("transfer09.after.shown", range(AFTER, 0, 2), "0.68 to 3.95", shown)?
        .pin("transfer09.repaired.shown", range(REPAIRED, 7, 1), "3.6 to 5.9", shown)?
        .pin(
            "transfer09.drag.jump.shown",
            &l15.drag_jump,
            r"\text{largest jump }0.003\text{ of }5.45",
            shown,
        )?
        .add("ch15.prediction.seconds", PREDICTION_SECONDS)?;
    for (key, source, text) in BASILISK {
        facts.add(key, format!("{source} {text}"))?;
    }
    for (key, quote) in AMREX {
        facts.add(key, quote)?;
    }
    Ok(())
}

// ----------------------------------------------------------------------------------- geometry

fn text_width(words: &str, size: f64) -> f64 {
    words.chars().count() as f64 * size * 0.0052
}

fn kick_x(t: f64) -> f64 {
    AXIS_X[0] + (AXIS_X[1] - AXIS_X[0]) * t / RUN_SECONDS
}

fn drag_y(drag: f64) -> f64 {
    DRAG_BASE + DRAG_SCALE * drag
}

/// The precisely lines' first baseline: four-line thirds start higher.
fn precisely_top(lines: usize) -> f64 {
    if lines > 3 { PRECISELY_Y + 0.3 } else { PRECISELY_Y }
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

/// The same with rounded corners, for cards and badges.
fn rounded(
    stage: &mut Stage<'_>,
    at: [f64; 2],
    size: [f64; 2],
    corner: f64,
    fill: (Srgb, f64),
    edge: (Srgb, f64),
) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Rectangle::new()
            .width(size[0])
            .height(size[1])
            .corner_radius(corner)
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

/// A flow arrow along x from `start` at height `y`; negative values point left. None for zero.
fn flow_x(stage: &mut Stage<'_>, start: f64, y: f64, value: Q) -> fmn::Result<Option<Mob>> {
    if value.is_zero() {
        return Ok(None);
    }
    let end = if value > Q::ZERO {
        start + drawn(value)
    } else {
        start - drawn(value)
    };
    Ok(Some(arrow(
        stage,
        [start, y],
        [end, y],
        box_view::flow_color(),
        box_view::FLOW_THICKNESS,
    )?))
}

/// An arrow on the big cell's side that ends just short of the seam, for `value` leaving it.
fn flow_to_seam(stage: &mut Stage<'_>, value: Q) -> fmn::Result<Mob> {
    arrow(
        stage,
        [OWN_END - drawn(value), OWN_Y],
        [OWN_END, OWN_Y],
        box_view::flow_color(),
        box_view::FLOW_THICKNESS,
    )
}

/// Inside face `f`'s arrow, centred on the face along its positive direction; None for zero.
fn inner_arrow(stage: &mut Stage<'_>, f: usize, value: Q) -> fmn::Result<Option<Mob>> {
    if value.is_zero() {
        return Ok(None);
    }
    let ([x, y], [dx, dy]) = (FACE_MID[f], FACE_DIR[f]);
    let half = drawn(value) / 2.0 * if value > Q::ZERO { 1.0 } else { -1.0 };
    Ok(Some(arrow(
        stage,
        [x - dx * half, y - dy * half],
        [x + dx * half, y + dy * half],
        box_view::flow_color(),
        box_view::FLOW_THICKNESS,
    )?))
}

/// A cross through a square of side `side` at `at`.
fn cross(stage: &mut Stage<'_>, at: [f64; 2], side: f64) -> fmn::Result<[Mob; 2]> {
    let h = side / 2.0 - 0.12;
    let red = Palette::red();
    Ok([
        line(stage, [at[0] - h, at[1] - h], [at[0] + h, at[1] + h], red, 4.0)?,
        line(stage, [at[0] - h, at[1] + h], [at[0] + h, at[1] - h], red, 4.0)?,
    ])
}

/// Rose fill strength for a dye level (level 6 at about half strength, so ink labels stay legible).
fn dye_opacity(level: Q) -> f64 {
    0.1 + 0.07 * level.to_f64()
}

// ------------------------------------------------------------------------------------- the scene

/// The chapter's state: what is on screen, and a pending batch of animations.
struct Ch15<'a> {
    k: &'a Kit,
    on: Vec<Mob>,
    pending: Vec<Box<dyn Animation>>,
}

impl Ch15<'_> {
    // Text through the kit, so the layout lint and the glyph check see every string.
    fn text(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: [f64; 2]) -> fmn::Result<Mob> {
        self.k.text(stage, s, size, color, p2(at))
    }

    fn text_left(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: [f64; 2]) -> fmn::Result<Mob> {
        self.k.text_left(stage, s, size, color, p2(at))
    }

    fn tex(&self, stage: &mut Stage<'_>, src: &str, size: f64, color: Srgb, at: [f64; 2]) -> fmn::Result<Mob> {
        self.k.tex(stage, src, size, color, &[], p2(at))
    }

    fn tex_left(&self, stage: &mut Stage<'_>, src: &str, size: f64, color: Srgb, at: [f64; 2]) -> fmn::Result<Mob> {
        self.k.tex_left(stage, src, size, color, &[], p2(at))
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

    /// One picture fades into another in the same batch.
    fn swap(&mut self, stage: &mut Stage<'_>, old: &[Mob], new: &[Mob]) -> fmn::Result<()> {
        self.vanish(stage, old, SWAP)?;
        self.appear(stage, new, SWAP)
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
            return Err(fail(format!("chapter 15 has no cue {id}")));
        }
        self.play(stage)?;
        println!("Chapter 15 cue {id} at {:.6} s", stage.scene().time().to_f64());
        self.k.say(stage, id)
    }

    /// Finish what is queued, then wait for sentence `j` of line `id` (nothing if it has begun).
    fn until(&mut self, stage: &mut Stage<'_>, id: &str, j: usize) -> fmn::Result<()> {
        let sentences = CUES.iter().find(|c| c.0 == id).map_or(0, |c| c.1);
        if j >= sentences {
            return Err(fail(format!("chapter 15 line {id} has no sentence {j}")));
        }
        self.play(stage)?;
        self.k.until_sentence(stage, id, j)
    }

    /// A card: rounded frame and centred words (text, or Tex when `tex` is set).
    fn card(&self, stage: &mut Stage<'_>, at: [f64; 2], words: &str, tex: bool, edge: Srgb) -> fmn::Result<[Mob; 2]> {
        let frame = rounded(stage, at, CARD_SIZE, 0.1, (card_fill(), 1.0), (edge, 1.6))?;
        let label = if tex {
            self.tex(stage, words, SMALL_SIZE, Palette::ink(), at)?
        } else {
            self.text(stage, words, SMALL_SIZE, Palette::ink(), at)?
        };
        Ok([frame, label])
    }

    /// Chapter 14's question card as its strip: an open dim frame (corner 0.12, stroke 2.0), ink words.
    fn question_strip(&self, stage: &mut Stage<'_>) -> fmn::Result<[Mob; 2]> {
        let (at, size, font) = CARD_STRIP;
        let frame = rounded(stage, at, size, 0.12, (Palette::ink(), 0.0), (Palette::dim(), 2.0))?;
        Ok([frame, self.text(stage, QUESTION_CARD, font, Palette::ink(), at)?])
    }

    /// Chapter 14's small outlined tag ("schematic").
    fn chip(&self, stage: &mut Stage<'_>, at: [f64; 2], words: &str, color: Srgb) -> fmn::Result<[Mob; 2]> {
        let width = text_width(words, 18.0) + 0.4;
        Ok([
            rounded(stage, at, [width, 0.4], 0.1, (card_fill(), 0.0), (color, 1.2))?,
            self.text(stage, words, 18.0, color, at)?,
        ])
    }

    /// A lower third with the exact form of a simplification, one text line per entry.
    fn precisely(&self, stage: &mut Stage<'_>, lines: &[&str]) -> fmn::Result<Vec<Mob>> {
        let top = precisely_top(lines.len());
        lines
            .iter()
            .enumerate()
            .map(|(i, words)| self.text(stage, words, 19.0, Palette::dim(), [0.0, top - 0.36 * i as f64]))
            .collect()
    }

    /// An imbalance badge: an open rounded frame and the signed value, red unless zero.
    fn badge(
        &self,
        stage: &mut Stage<'_>,
        at: [f64; 2],
        size: [f64; 2],
        words: &str,
        value: Q,
    ) -> fmn::Result<[Mob; 2]> {
        let color = box_view::badge_color(value);
        Ok([
            rounded(stage, at, size, 0.07, (card_fill(), 0.0), (color, 2.2))?,
            self.tex(stage, words, SMALL_SIZE, color, at)?,
        ])
    }

    /// The four children's badges for imbalances `d`.
    fn badges(&self, stage: &mut Stage<'_>, d: &V4) -> fmn::Result<Vec<Mob>> {
        let mut out = Vec::new();
        for (i, &q) in d.iter().enumerate() {
            out.extend(self.badge(stage, BADGE_AT[i], BADGE_SIZE, &signed(q), q)?);
        }
        Ok(out)
    }

    /// The inside faces' arrows and value labels for `inside`.
    fn inside(&self, stage: &mut Stage<'_>, inside: &V4) -> fmn::Result<Vec<Mob>> {
        let mut out = Vec::new();
        for (f, &q) in inside.iter().enumerate() {
            out.extend(inner_arrow(stage, f, q)?);
            let size = if f < 2 { FLOW_SIZE } else { 20.0 };
            out.push(self.tex(stage, &q.tex(), size, box_view::flow_color(), INNER_LABEL_AT[f])?);
        }
        Ok(out)
    }

    /// HOLD 15_h1: the card asks; the line ends plus a breath (`kit.hold`); a marked five-second
    /// countdown; the card leaves. Nothing of 15_4's answer exists before this returns.
    fn predict(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let (ink, dim) = (Palette::ink(), Palette::dim());
        let mut asked = vec![rounded(
            stage,
            PREDICT_AT,
            PREDICT_SIZE,
            0.1,
            (card_fill(), 1.0),
            (dim, 1.4),
        )?];
        for (i, src) in QUESTION_TEX.iter().enumerate() {
            asked.push(self.tex(
                stage,
                src,
                24.0,
                ink,
                [PREDICT_AT[0], PREDICT_AT[1] + 0.95 - 0.4 * i as f64],
            )?);
        }
        for (i, words) in OPTIONS.iter().enumerate() {
            asked.push(self.text(
                stage,
                words,
                21.0,
                dim,
                [PREDICT_AT[0], PREDICT_AT[1] + 0.05 - 0.36 * i as f64],
            )?);
        }
        self.appear(stage, &asked, FADE)?;
        self.play(stage)?;
        self.k.hold(stage)?;
        self.k.mark(stage, "predict");
        println!(
            "Chapter 15 hold 15_h1 at {:.6} s: {QUESTION}",
            stage.scene().time().to_f64()
        );
        for second in (1..=PREDICTION_SECONDS).rev() {
            let clock = self.text(
                stage,
                &second.to_string(),
                30.0,
                Palette::hero(),
                [PREDICT_AT[0], PREDICT_AT[1] - 1.05],
            )?;
            self.put(stage, &[clock])?;
            stage.wait(1.0)?;
            self.cut(stage, &[clock]);
        }
        self.vanish(stage, &asked, FADE)?;
        self.play(stage)
    }

    /// Everything on screen except `keep`.
    fn all_but(&self, keep: &[Mob]) -> Vec<Mob> {
        self.on.iter().copied().filter(|m| !keep.contains(m)).collect()
    }
}

impl SceneConstruct for Chapter15 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        // The render log carries the fact table, so each picture's numbers can be read against it.
        for fact in facts().map_err(fail)?.print().lines() {
            println!("Chapter 15 fact {fact}");
        }
        let l = labels();
        let (ink, dim, faint, blue) = (Palette::ink(), Palette::dim(), Palette::faint(), box_view::flow_color());
        let (red, green) = (Palette::red(), Palette::green());
        let [x0, x1, y0, y1] = SEAM_BIG;
        let side = x1 - x0;
        let big_at = [(x0 + x1) / 2.0, (y0 + y1) / 2.0];
        let neighbour_at = [x1 + side / 2.0, (y0 + y1) / 2.0];
        let mut s = Ch15 {
            k: &self.kit,
            on: Vec::new(),
            pending: Vec::new(),
        };

        // Frame 0: chapter 14's last picture, object for object (its 14_13 code and constants).
        let strip = s.question_strip(stage)?;
        let big = rect(stage, big_at, [side, y1 - y0], (ink, 0.0), (ink, 1.8))?;
        let neighbour = rect(stage, neighbour_at, [side, side], (ink, 0.0), (ink, 1.8))?;
        let h = side / 2.0;
        let quarter = [
            line(
                stage,
                [neighbour_at[0] - h, neighbour_at[1]],
                [neighbour_at[0] + h, neighbour_at[1]],
                ink,
                1.6,
            )?,
            line(
                stage,
                [neighbour_at[0], neighbour_at[1] - h],
                [neighbour_at[0], neighbour_at[1] + h],
                ink,
                1.6,
            )?,
        ];
        let chip = s.chip(stage, [x1, y0 - 0.55], SCHEMATIC, dim)?;
        let edge = line(stage, [x1, y0], [x1, y1], glow(), 4.5)?;
        let seam_label = s.text(stage, SEAM, 24.0, glow(), SEAM_LABEL_LOW)?;
        s.put(stage, &strip)?;
        s.put(stage, &[big, neighbour])?;
        s.put(stage, &quarter)?;
        s.put(stage, &chip)?;
        s.put(stage, &[edge, seam_label])?;

        // 15_1: the seam; the big cell; the two small cells beside it (the rest of the split cell
        // dims); the shared face splits at its midpoint; the box-units tag and the toy-numbers note.
        s.say(stage, "15_1")?;
        s.pulse(stage, &[edge], Some(WHITE_RGB))?;
        s.until(stage, "15_1", 1)?;
        s.pulse(stage, &[big], None)?;
        s.until(stage, "15_1", 2)?;
        let faint_neighbour = rect(stage, neighbour_at, [side, side], (ink, 0.0), (faint, 1.8))?;
        let faint_quarter = [
            line(
                stage,
                [neighbour_at[0] - h, neighbour_at[1]],
                [neighbour_at[0] + h, neighbour_at[1]],
                faint,
                1.6,
            )?,
            line(
                stage,
                [neighbour_at[0], neighbour_at[1] - h],
                [neighbour_at[0], neighbour_at[1] + h],
                faint,
                1.6,
            )?,
        ];
        s.morph(stage, neighbour, faint_neighbour, SWAP)?;
        s.morph(stage, quarter[0], faint_quarter[0], SWAP)?;
        s.morph(stage, quarter[1], faint_quarter[1], SWAP)?;
        let small_cells = [
            rect(stage, [SMALL[0][0], MID_Y], [side / 2.0, side], (ink, 0.0), (ink, 1.8))?,
            line(stage, [x1, MID_Y], [x1 + side / 2.0, MID_Y], ink, 1.6)?,
        ];
        s.draw(stage, &small_cells, SWAP)?;
        s.until(stage, "15_1", 3)?;
        let halves = [
            line(stage, [x1, y0], [x1, MID_Y], glow(), 4.5)?,
            line(stage, [x1, MID_Y], [x1, y1], glow(), 4.5)?,
        ];
        s.cut(stage, &[edge]);
        s.put(stage, &halves)?;
        let tick = line(stage, [x1 - 0.15, MID_Y], [x1 + 0.15, MID_Y], glow(), 3.0)?;
        s.appear(stage, &[tick], FADE)?;
        s.vanish(stage, &chip, FADE)?;
        let tag = s.tex_left(stage, &box_view::tag_tex(), box_view::TAG_SIZE, dim, TAG_AT)?;
        let toy = s.text_left(stage, TOY_NOTE, 18.0, dim, TOY_AT)?;
        s.appear(stage, &[tag, toy], FADE)?;
        s.play(stage)?;
        s.pulse(stage, &[halves[0]], Some(WHITE_RGB))?;
        s.play(stage)?;
        s.pulse(stage, &[halves[1]], Some(WHITE_RGB))?;

        // 15_2: the small faces carry 3/2 and 1/2; the small cells receive 2.
        s.say(stage, "15_2")?;
        let [lower, upper] = fine_faces();
        s.until(stage, "15_2", 1)?;
        let lower_arrow = flow_x(stage, x1, SMALL[0][1], lower)?.ok_or_else(|| fail("3/2 draws".into()))?;
        let mut lower_label = s.tex(stage, &lower.tex(), FLOW_SIZE, blue, SMALL_LABEL_AT[0])?;
        s.appear(stage, &[lower_arrow, lower_label], FADE)?;
        s.until(stage, "15_2", 2)?;
        let upper_arrow = flow_x(stage, x1, SMALL[1][1], upper)?.ok_or_else(|| fail("1/2 draws".into()))?;
        let mut upper_label = s.tex(stage, &upper.tex(), FLOW_SIZE, blue, SMALL_LABEL_AT[1])?;
        s.appear(stage, &[upper_arrow, upper_label], FADE)?;
        s.until(stage, "15_2", 3)?;
        let small_sum = s.tex(stage, &l.small_sum, 26.0, ink, [SIDE_X, READOUT_Y[0]])?;
        s.appear(stage, &[small_sum], FADE)?;

        // 15_3: the big cell's own view, 5/2 leaving its side; 2 arriving; then the prediction.
        s.say(stage, "15_3")?;
        let own = flow_to_seam(stage, coarse_own())?;
        let own_label = s.tex(
            stage,
            &coarse_own().tex(),
            FLOW_SIZE,
            blue,
            [OWN_END - drawn(coarse_own()) / 2.0, OWN_LABEL_Y],
        )?;
        let big_view = s.tex(stage, &l.big_view, 26.0, ink, [SIDE_X, READOUT_Y[1]])?;
        s.appear(stage, &[own, own_label, big_view], FADE)?;
        s.until(stage, "15_3", 1)?;
        s.pulse(stage, &[own, big_view], Some(BLUE_RGB))?;
        s.play(stage)?;
        s.pulse(stage, &[lower_arrow, upper_arrow, small_sum], Some(BLUE_RGB))?;
        s.until(stage, "15_3", 2)?;
        s.predict(stage)?;

        // 15_4: the answer. The half is lost every step: a red badge on the seam pulses once a step.
        s.say(stage, "15_4")?;
        let leak_badge = s.badge(stage, LEAK_AT, LEAK_SIZE, &l.leak, leak())?;
        s.appear(stage, &leak_badge, FADE)?;
        s.until(stage, "15_4", 1)?;
        // Three pulses, one a step, back to back (2.4 s: the sentence is short).
        for _ in 0..3 {
            s.pulse(stage, &leak_badge, Some(RED_RGB))?;
            s.play(stage)?;
        }
        s.until(stage, "15_4", 2)?;
        // Chapter 07's 07_5 ledger in miniature: one face, the same flow out of one cell and into the next.
        let [lx, ly] = LEDGER07_AT;
        let flow07 = Q::int(LEDGER07_FLOW);
        let ledger07 = vec![
            rounded(stage, LEDGER07_AT, [3.0, 2.3], 0.1, (card_fill(), 1.0), (dim, 1.2))?,
            s.text(stage, LEDGER07_HEAD, 18.0, dim, [lx, ly + 0.9])?,
            rect(stage, [lx - 0.4, ly + 0.25], [0.8, 0.8], (ink, 0.0), (ink, 1.6))?,
            rect(stage, [lx + 0.4, ly + 0.25], [0.8, 0.8], (ink, 0.0), (ink, 1.6))?,
            arrow(stage, [lx - 0.25, ly + 0.25], [lx + 0.25, ly + 0.25], blue, 3.0)?,
            s.tex(stage, &signed(flow07), FLOW_SIZE, blue, [lx - 0.4, ly - 0.4])?,
            s.tex(stage, &signed(-flow07), FLOW_SIZE, blue, [lx + 0.4, ly - 0.4])?,
            s.text(stage, LEDGER07_NOTE, 18.0, dim, [lx, ly - 0.85])?,
        ];
        s.appear(stage, &ledger07, FADE)?;
        s.until(stage, "15_4", 3)?;
        s.vanish(stage, &ledger07, FADE)?;
        // The rule takes the miniature's place once it has gone (its header sat at the rule's height).
        s.play(stage)?;
        let rule = s.text(stage, RULE_15_4, SMALL_SIZE, ink, [SIDE_X, READOUT_Y[2]])?;
        s.appear(stage, &[rule], FADE)?;
        let mut precisely = s.precisely(stage, &PRECISELY_15_4)?;
        s.appear(stage, &precisely, FADE)?;

        // 15_5: cure 1 builds the big face from the small ones (2); cure 2 refluxes the 5/2 state.
        s.say(stage, "15_5")?;
        s.vanish(stage, &precisely, FADE)?;
        s.until(stage, "15_5", 1)?;
        let built = flow_to_seam(stage, consistent_flow())?;
        let built_label = s.tex(
            stage,
            &consistent_flow().tex(),
            FLOW_SIZE,
            blue,
            [OWN_END - drawn(consistent_flow()) / 2.0, OWN_LABEL_Y],
        )?;
        let cure_one = s.tex(stage, &l.cure_one, SMALL_SIZE, ink, [SIDE_X, READOUT_Y[3]])?;
        let no_leak = s.badge(stage, LEAK_AT, LEAK_SIZE, &l.mended, Q::ZERO)?;
        s.pulse(stage, &[lower_arrow, upper_arrow], Some(BLUE_RGB))?;
        s.play(stage)?;
        s.swap(stage, &[own, own_label], &[built, built_label])?;
        s.swap(stage, &leak_badge, &no_leak)?;
        s.appear(stage, &[cure_one], FADE)?;
        s.until(stage, "15_5", 2)?;
        // Back to the 5/2 state, then the difference is paid back to the big cell.
        let own_again = flow_to_seam(stage, coarse_own())?;
        let own_again_label = s.tex(
            stage,
            &coarse_own().tex(),
            FLOW_SIZE,
            blue,
            [OWN_END - drawn(coarse_own()) / 2.0, OWN_LABEL_Y],
        )?;
        let leak_again = s.badge(stage, LEAK_AT, LEAK_SIZE, &l.leak, leak())?;
        s.swap(stage, &[built, built_label], &[own_again, own_again_label])?;
        s.swap(stage, &no_leak, &leak_again)?;
        s.play(stage)?;
        let reflux = flow_x(stage, x1, REFLUX_Y, -leak())?.ok_or_else(|| fail("the reflux arrow draws".into()))?;
        let reflux_label = s.tex(
            stage,
            &format!(r"\text{{reflux }}{}", signed(leak())),
            SMALL_SIZE,
            blue,
            REFLUX_LABEL_AT,
        )?;
        let cure_two = s.tex(stage, &l.cure_two, SMALL_SIZE, ink, [SIDE_X, READOUT_Y[4]])?;
        s.appear(stage, &[reflux, reflux_label, cure_two], FADE)?;
        s.play(stage)?;
        let mended = s.badge(stage, LEAK_AT, LEAK_SIZE, &l.mended, Q::ZERO)?;
        s.swap(stage, &leak_again, &mended)?;
        s.play(stage)?;
        s.pulse(stage, &mended, Some(GREEN_RGB))?;
        precisely = s.precisely(stage, &PRECISELY_15_5)?;
        s.appear(stage, &precisely, FADE)?;

        // 15_6: merging four dye cells keeps the total; copying one child makes dye.
        s.say(stage, "15_6")?;
        let mut leaving = vec![own_again, own_again_label, reflux, reflux_label];
        leaving.extend(mended);
        leaving.extend([small_sum, big_view, rule, cure_one, cure_two]);
        leaving.extend(precisely.iter().copied());
        s.vanish(stage, &leaving, FADE)?;
        let head = s.text(stage, MERGE_HEAD, 20.0, dim, [SIDE_X, HEAD_Y])?;
        s.appear(stage, &[head], FADE)?;
        s.until(stage, "15_6", 1)?;
        let child = MERGE_SIDE / 2.0;
        let mut children = Vec::new();
        let mut child_labels = Vec::new();
        for (k, &level) in DYE.iter().enumerate() {
            let (col, row) = (k % 2, k / 2);
            let at = [
                MERGE_X[0] - child / 2.0 + child * col as f64,
                MERGE_Y + child / 2.0 - child * row as f64,
            ];
            children.push(rect(
                stage,
                at,
                [child, child],
                (dye(), dye_opacity(Q::int(level))),
                (ink, 1.4),
            )?);
            child_labels.push(s.text(stage, &level.to_string(), FLOW_SIZE, ink, at)?);
        }
        let merge_tags: Vec<Mob> = MERGE_TAGS
            .iter()
            .zip(MERGE_X)
            .map(|(words, x)| s.text(stage, words, 17.0, dim, [x, TAG_Y]))
            .collect::<fmn::Result<_>>()?;
        let total_before = s.tex(stage, &l.merge_total, FLOW_SIZE, ink, [MERGE_X[0], TOTAL_Y])?;
        s.appear(stage, &children, FADE)?;
        s.appear(stage, &child_labels, FADE)?;
        s.appear(stage, &[merge_tags[0], total_before], FADE)?;
        s.until(stage, "15_6", 2)?;
        let merged_at = [MERGE_X[1], MERGE_Y];
        let merged = rect(
            stage,
            merged_at,
            [MERGE_SIDE, MERGE_SIDE],
            (dye(), dye_opacity(merge_parent())),
            (ink, 1.6),
        )?;
        let merged_label = s.tex(stage, &merge_parent().tex(), FLOW_SIZE, ink, merged_at)?;
        let into = arrow(
            stage,
            [MERGE_X[0] + MERGE_SIDE / 2.0 + 0.12, MERGE_Y],
            [MERGE_X[1] - MERGE_SIDE / 2.0 - 0.12, MERGE_Y],
            dim,
            2.0,
        )?;
        let total_after = s.tex(stage, &l.merged_total, FLOW_SIZE, ink, [MERGE_X[1], TOTAL_Y])?;
        s.appear(stage, &[into, merged, merged_label, merge_tags[1], total_after], FADE)?;
        s.until(stage, "15_6", 3)?;
        let ghost_at = [MERGE_X[2], MERGE_Y];
        let mut ghost = vec![
            rect(
                stage,
                ghost_at,
                [MERGE_SIDE, MERGE_SIDE],
                (dye(), dye_opacity(Q::int(6)) * 0.5),
                (dim, 1.4),
            )?,
            s.text(stage, "6", FLOW_SIZE, dim, ghost_at)?,
            merge_tags[2],
            s.tex(stage, &l.copy_total, FLOW_SIZE, red, [MERGE_X[2], TOTAL_Y])?,
        ];
        s.appear(stage, &ghost, FADE)?;
        s.play(stage)?;
        let crossed = cross(stage, ghost_at, MERGE_SIDE)?;
        s.draw(stage, &crossed, SWAP)?;
        ghost.extend(crossed);
        precisely = s.precisely(stage, &PRECISELY_15_6)?;
        s.appear(stage, &precisely, FADE)?;

        // 15_7: a split keeps the total: 15/4 and 17/4 in each row, 16 = 16.
        s.say(stage, "15_7")?;
        let mut leaving = children;
        leaving.extend(child_labels);
        leaving.extend([
            merge_tags[0],
            merge_tags[1],
            total_before,
            into,
            merged,
            merged_label,
            total_after,
        ]);
        leaving.extend(ghost);
        leaving.extend(precisely.iter().copied());
        s.vanish(stage, &leaving, FADE)?;
        let mut row_cells = Vec::new();
        let mut row_labels = Vec::new();
        let mut row_tags = Vec::new();
        for (k, &level) in ROW.iter().enumerate() {
            let at = [SPLIT_X[k], SPLIT_Y];
            let edge_color = if k == 1 { ink } else { dim };
            row_cells.push(rect(
                stage,
                at,
                [SPLIT_SIDE, SPLIT_SIDE],
                (dye(), dye_opacity(Q::int(level))),
                (edge_color, 1.6),
            )?);
            row_labels.push(s.text(stage, &level.to_string(), 26.0, ink, at)?);
            row_tags.push(s.text(stage, SPLIT_TAGS[k], 17.0, dim, [SPLIT_X[k], TAG_Y])?);
        }
        s.appear(stage, &row_cells, FADE)?;
        s.appear(stage, &row_labels, FADE)?;
        s.appear(stage, &row_tags, FADE)?;
        s.until(stage, "15_7", 1)?;
        let [left_child, right_child] = slope_children();
        let quarter_side = SPLIT_SIDE / 2.0;
        let mut kids = Vec::new();
        let mut kid_labels = Vec::new();
        for row in 0..2 {
            for (col, value) in [left_child, right_child].into_iter().enumerate() {
                let at = [
                    SPLIT_X[1] - quarter_side / 2.0 + quarter_side * col as f64,
                    SPLIT_Y - quarter_side / 2.0 + quarter_side * row as f64,
                ];
                kids.push(rect(
                    stage,
                    at,
                    [quarter_side, quarter_side],
                    (dye(), dye_opacity(value)),
                    (ink, 1.2),
                )?);
                kid_labels.push(s.tex(stage, &value.tex(), SMALL_SIZE, ink, at)?);
            }
        }
        s.vanish(stage, &[row_labels[1]], SWAP)?;
        s.appear(stage, &kids, SWAP)?;
        s.appear(stage, &kid_labels, SWAP)?;
        s.until(stage, "15_7", 2)?;
        let totals = [
            s.tex(stage, &l.split_children, SMALL_SIZE, ink, [SIDE_X, 0.4])?,
            s.tex(stage, &l.split_parent, SMALL_SIZE, ink, [SIDE_X, -0.1])?,
        ];
        s.appear(stage, &totals, FADE)?;
        precisely = s.precisely(stage, &PRECISELY_15_7)?;
        s.appear(stage, &precisely, FADE)?;

        // 15_8: the big cell splits into A, B, C and D; the outside faces keep their flows; the
        // straight-line inside guess leaves B losing a half and D gaining a half.
        s.say(stage, "15_8")?;
        let mut leaving = row_cells;
        leaving.extend(row_labels);
        leaving.extend(row_tags);
        leaving.extend(kids);
        leaving.extend(kid_labels);
        leaving.extend(totals);
        leaving.push(head);
        leaving.extend(precisely.iter().copied());
        s.vanish(stage, &leaving, FADE)?;
        s.until(stage, "15_8", 1)?;
        let split_lines = [
            line(stage, [x0, MID_Y], [x1, MID_Y], ink, 1.6)?,
            line(stage, [big_at[0], y0], [big_at[0], y1], ink, 1.6)?,
        ];
        s.draw(stage, &split_lines, SWAP)?;
        let letters: Vec<Mob> = CELLS
            .iter()
            .zip(LETTER_AT)
            .map(|(name, at)| s.text(stage, name, 26.0, ink, at))
            .collect::<fmn::Result<_>>()?;
        s.appear(stage, &letters, FADE)?;
        let o = outer();
        let mut inflows = Vec::new();
        for i in [0, 2] {
            inflows.extend(flow_x(stage, INFLOW_START, KIDS[i][1], -o[i])?);
            inflows.push(s.text(stage, IN_WORD, SMALL_SIZE, blue, [INFLOW_LABEL_X, KIDS[i][1]])?);
        }
        s.appear(stage, &inflows, FADE)?;
        let lower_out = s.tex(
            stage,
            &format!(r"{}\text{{ out}}", lower.tex()),
            FLOW_SIZE,
            blue,
            SMALL_LABEL_AT[0],
        )?;
        let upper_out = s.tex(
            stage,
            &format!(r"{}\text{{ out}}", upper.tex()),
            FLOW_SIZE,
            blue,
            SMALL_LABEL_AT[1],
        )?;
        s.swap(stage, &[lower_label, upper_label], &[lower_out, upper_out])?;
        lower_label = lower_out;
        upper_label = upper_out;
        let seam_high = s.text(stage, SEAM, 24.0, glow(), SEAM_LABEL_HIGH)?;
        s.morph(stage, seam_label, seam_high, SWAP)?;
        s.until(stage, "15_8", 2)?;
        let unknown: Vec<Mob> = INNER_LABEL_AT
            .iter()
            .map(|&at| s.text(stage, "?", FLOW_SIZE, dim, at))
            .collect::<fmn::Result<_>>()?;
        s.appear(stage, &unknown, FADE)?;
        s.play(stage)?;
        s.pulse(stage, &split_lines, None)?;
        s.until(stage, "15_8", 3)?;
        let mut inner = s.inside(stage, &inner_line())?;
        let mut badges = s.badges(stage, &imbalance(&o, &inner_line()))?;
        s.swap(stage, &unknown, &inner)?;
        s.appear(stage, &badges, FADE)?;
        s.play(stage)?;
        s.pulse(stage, &[badges[2], badges[3], badges[6], badges[7]], Some(RED_RGB))?;

        // 15_9: chapter 09's pressure puzzle on the four children; the push balances all four.
        s.say(stage, "15_9")?;
        let mut pressure_marks = Vec::new();
        for (&pi, at) in pressure().iter().zip(PRESSURE_AT) {
            pressure_marks.push(s.tex(stage, &signed(pi), SMALL_SIZE, pressure_color(pi), at)?);
        }
        let mut puzzle = vec![
            s.text(stage, PUZZLE_NOTE, 20.0, dim, [SIDE_X, SIDE_Y[0]])?,
            s.tex(stage, r"L\,p = d,\quad p_A = 0", 26.0, ink, [SIDE_X, SIDE_Y[1]])?,
        ];
        s.appear(stage, &pressure_marks, FADE)?;
        s.appear(stage, &puzzle, FADE)?;
        s.play(stage)?;
        let solved = s.tex(stage, &l.pressures, 26.0, ink, [SIDE_X, SIDE_Y[2]])?;
        s.appear(stage, &[solved], FADE)?;
        puzzle.push(solved);
        s.until(stage, "15_9", 1)?;
        let fixed_inner = s.inside(stage, &inner_fixed())?;
        let fixed_badges = s.badges(stage, &imbalance(&o, &inner_fixed()))?;
        let push = s.tex(stage, r"u = u^{*} - \nabla p", 26.0, ink, [SIDE_X, SIDE_Y[3]])?;
        s.swap(stage, &inner, &fixed_inner)?;
        s.swap(stage, &badges, &fixed_badges)?;
        s.appear(stage, &[push], FADE)?;
        puzzle.push(push);
        inner = fixed_inner;
        badges = fixed_badges;
        s.play(stage)?;
        s.pulse(stage, &badges, Some(GREEN_RGB))?;
        precisely = s.precisely(stage, &PRECISELY_15_9)?;
        s.appear(stage, &precisely, FADE)?;

        // 15_10: the repair needs the outside faces to add up. With a disagreeing seam the four
        // imbalances add to a half whatever the inside faces carry.
        s.say(stage, "15_10")?;
        s.vanish(stage, &precisely, FADE)?;
        s.vanish(stage, &puzzle, FADE)?;
        let outside = s.tex(stage, &l.outside_good, 26.0, ink, [SIDE_X, SIDE_Y[0]])?;
        s.appear(stage, &[outside], FADE)?;
        s.play(stage)?;
        let outer_arrows: Vec<Mob> = inflows.iter().copied().chain([lower_arrow, upper_arrow]).collect();
        s.pulse(stage, &outer_arrows, Some(BLUE_RGB))?;
        s.until(stage, "15_10", 1)?;
        let bad = outer_bad();
        let bad_arrow = flow_x(stage, x1, SMALL[0][1], bad[1])?.ok_or_else(|| fail("B's 2 draws".into()))?;
        let bad_label = s.tex(
            stage,
            &format!(r"{}\text{{ out}}", bad[1].tex()),
            FLOW_SIZE,
            blue,
            SMALL_LABEL_AT[0],
        )?;
        let outside_bad = s.tex(stage, &l.outside_bad, 26.0, red, [SIDE_X, SIDE_Y[0]])?;
        let bad_badges = s.badges(stage, &imbalance(&bad, &inner_fixed()))?;
        s.swap(stage, &[lower_arrow, lower_label], &[bad_arrow, bad_label])?;
        s.swap(stage, &[outside], &[outside_bad])?;
        s.swap(stage, &badges, &bad_badges)?;
        badges = bad_badges;
        s.until(stage, "15_10", 2)?;
        let sum_box = [
            rounded(stage, SUM_BOX_AT, SUM_BOX_SIZE, 0.1, (card_fill(), 1.0), (red, 2.2))?,
            s.tex(stage, &l.bad_sum, 26.0, red, SUM_BOX_AT)?,
        ];
        s.appear(stage, &sum_box, FADE)?;
        s.until(stage, "15_10", 3)?;
        let mut try_note: Option<Mob> = None;
        for (k, inside) in tries().iter().enumerate() {
            let next_inner = s.inside(stage, inside)?;
            let next_badges = s.badges(stage, &imbalance(&bad, inside))?;
            let note = s.text(stage, TRIES[k], 20.0, dim, [SIDE_X, TRY_Y])?;
            s.swap(stage, &inner, &next_inner)?;
            s.swap(stage, &badges, &next_badges)?;
            match try_note {
                Some(old) => s.swap(stage, &[old], &[note])?,
                None => s.appear(stage, &[note], SWAP)?,
            }
            try_note = Some(note);
            inner = next_inner;
            badges = next_badges;
            s.play(stage)?;
            s.pulse(stage, &sum_box, Some(RED_RGB))?;
            s.pause(stage, BEAT)?;
        }

        // 15_11: the seam returns as it was (the big cell whole, the agreed small flows); five cards.
        s.say(stage, "15_11")?;
        let mut leaving = split_lines.to_vec();
        leaving.extend(letters);
        leaving.extend(inflows);
        leaving.extend(inner);
        leaving.extend(badges);
        leaving.extend(pressure_marks);
        leaving.extend([outside_bad]);
        leaving.extend(sum_box);
        leaving.extend(try_note);
        s.vanish(stage, &leaving, FADE)?;
        let lower_again = flow_x(stage, x1, SMALL[0][1], lower)?.ok_or_else(|| fail("3/2 draws".into()))?;
        let lower_plain = s.tex(stage, &lower.tex(), FLOW_SIZE, blue, SMALL_LABEL_AT[0])?;
        let upper_plain = s.tex(stage, &upper.tex(), FLOW_SIZE, blue, SMALL_LABEL_AT[1])?;
        s.swap(
            stage,
            &[bad_arrow, bad_label, upper_label],
            &[lower_again, lower_plain, upper_plain],
        )?;
        let seam_low = s.text(stage, SEAM, 24.0, glow(), SEAM_LABEL_LOW)?;
        s.morph(stage, seam_label, seam_low, SWAP)?;
        s.until(stage, "15_11", 1)?;
        let mut cards = Vec::new();
        for (k, (&y, words)) in CARD_Y.iter().zip(CARDS).take(3).enumerate() {
            cards.extend(s.card(stage, [SIDE_X, y], words, false, ink)?);
            s.appear(stage, &cards[2 * k..], FADE)?;
            s.pause(stage, BEAT)?;
        }
        s.until(stage, "15_11", 2)?;
        cards.extend(s.card(stage, [SIDE_X, CARD_Y[3]], CARDS[3], false, dim)?);
        s.appear(stage, &cards[6..], FADE)?;
        s.until(stage, "15_11", 3)?;
        cards.extend(s.card(stage, [SIDE_X, CARD_Y[4]], CARD_LAST_TEX, true, dim)?);
        s.appear(stage, &cards[8..], FADE)?;

        // 15_12: measured. One adaptive tunnel run's 14 changes of grid: the kick just after each
        // change, then the before values, the run's own repair and the drag before and after.
        s.say(stage, "15_12")?;
        println!("Chapter 15 measured panel: navier-solver-transfer04 {RUN} transfers.csv {TRANSFERS_SHA256}");
        s.vanish(stage, &cards, FADE)?;
        let panel_head = vec![
            s.text(stage, MEASURED_HEAD, 20.0, ink, [SIDE_X, MEASURED_Y[0]])?,
            s.text(stage, MEASURED_SUB, 16.0, dim, [SIDE_X, MEASURED_Y[1]])?,
            s.text_left(stage, KICK_AXIS, 17.0, dim, [0.6, KICK_LABEL_Y])?,
            line(stage, [AXIS_X[0], KICK_BASE], [AXIS_X[1], KICK_BASE], dim, 1.3)?,
            s.text(stage, "0", 15.0, dim, [AXIS_X[0] - 0.22, KICK_BASE])?,
            s.text(stage, "3", 15.0, dim, [AXIS_X[0] - 0.22, KICK_BASE + 3.0 * KICK_SCALE])?,
            line(
                stage,
                [AXIS_X[0] - 0.06, KICK_BASE + 3.0 * KICK_SCALE],
                [AXIS_X[0], KICK_BASE + 3.0 * KICK_SCALE],
                dim,
                1.3,
            )?,
        ];
        s.appear(stage, &panel_head, FADE)?;
        s.play(stage)?;
        // Each bar rises from the axis to the run's raw value just after its change.
        for row in 0..CHANGES.len() {
            let x = kick_x(field(row, T));
            let tall = field(row, AFTER) * KICK_SCALE;
            let start = rect(stage, [x, KICK_BASE + 0.005], [BAR_WIDTH, 0.01], (red, 1.0), (red, 0.0))?;
            let full = rect(
                stage,
                [x, KICK_BASE + tall / 2.0],
                [BAR_WIDTH, tall],
                (red, 1.0),
                (red, 0.0),
            )?;
            s.put(stage, &[start])?;
            s.morph(stage, start, full, 0.9)?;
        }
        let after_readout = s.tex(stage, &l.after, 18.0, red, [SIDE_X, KICK_READOUT_Y[1]])?;
        s.appear(stage, &[after_readout], FADE)?;
        s.until(stage, "15_12", 1)?;
        let mut before_marks = Vec::new();
        let mut repaired_marks = Vec::new();
        let mut drag_marks = Vec::new();
        for row in 0..CHANGES.len() {
            let x = kick_x(field(row, T));
            before_marks.push(ring(stage, [x, KICK_BASE], 0.05, dim, 1.4));
            repaired_marks.push(disc(stage, [x, KICK_BASE], 0.035, green));
            drag_marks.push(ring(stage, [x, drag_y(field(row, DRAG_BEFORE))], 0.06, dim, 1.4));
            drag_marks.push(disc(stage, [x, drag_y(field(row, DRAG_AFTER))], 0.035, ink));
        }
        let before_readout = s.tex(stage, &l.before, 18.0, dim, [SIDE_X, KICK_READOUT_Y[0]])?;
        s.appear(stage, &before_marks, FADE)?;
        s.appear(stage, &[before_readout], FADE)?;
        let drag_frame = vec![
            s.text_left(stage, DRAG_AXIS, 17.0, dim, [0.6, DRAG_LABEL_Y])?,
            line(stage, [AXIS_X[0], DRAG_BASE], [AXIS_X[1], DRAG_BASE], dim, 1.3)?,
            s.text(stage, "0", 15.0, dim, [AXIS_X[0] - 0.22, DRAG_BASE])?,
            s.text(stage, "20", 15.0, dim, [AXIS_X[0] - 0.26, drag_y(20.0)])?,
            line(
                stage,
                [AXIS_X[0] - 0.06, drag_y(20.0)],
                [AXIS_X[0], drag_y(20.0)],
                dim,
                1.3,
            )?,
            s.text(stage, "0 s", 15.0, dim, [AXIS_X[0], TIME_LABEL_Y])?,
            s.text(stage, "0.25 s", 15.0, dim, [AXIS_X[1], TIME_LABEL_Y])?,
        ];
        s.appear(stage, &drag_frame, FADE)?;
        s.appear(stage, &drag_marks, FADE)?;
        s.play(stage)?;
        let repaired_readout = s.tex(stage, &l.repaired, 18.0, green, [SIDE_X, KICK_READOUT_Y[2]])?;
        let drag_readouts = [
            s.tex(stage, &l.drag_same, 17.0, ink, DRAG_READOUT_AT[0])?,
            s.tex(stage, &l.drag_jump, 17.0, ink, DRAG_READOUT_AT[1])?,
        ];
        s.appear(stage, &repaired_marks, FADE)?;
        s.appear(stage, &[repaired_readout], FADE)?;
        s.appear(stage, &drag_readouts, FADE)?;
        precisely = s.precisely(stage, &PRECISELY_15_12)?;
        s.appear(stage, &precisely, FADE)?;
        s.play(stage)?;
        let on_screen = s.all_but(&[]).len();
        println!(
            "Chapter 15 ends on the seam (big cell whole, small faces 3/2 and 1/2) beside the measured transfer09 panel: {on_screen} objects"
        );
        s.k.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(n: i64, d: i64) -> Q {
        Q::new(n, d)
    }

    #[test]
    fn cues_run_in_order() {
        let ids: Vec<&str> = CUES.iter().map(|&(id, _)| id).collect();
        let expected: Vec<String> = (1..=12).map(|k| format!("15_{k}")).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        assert!(CUES.iter().all(|&(_, sentences)| sentences > 0));
    }

    #[test]
    fn the_seam_loses_a_half_and_both_cures_stop_it() {
        assert_eq!(fine_total(), Q::int(2));
        assert_eq!(coarse_own(), q(5, 2));
        assert_eq!(leak(), q(1, 2));
        assert_eq!(consistent_speed(), Q::ONE);
        assert_eq!(consistent_flow(), fine_total());
        assert_eq!(refluxed_loss(), fine_total());
    }

    #[test]
    fn merging_and_splitting_keep_the_dye() {
        assert_eq!(merge_total_before(), Q::int(12));
        assert_eq!(merge_parent(), Q::int(3));
        assert_eq!(merge_total_after(), merge_total_before());
        assert_eq!(merge_copy_total(), Q::int(24));
        assert_eq!(slope_children(), [q(15, 4), q(17, 4)]);
        assert_eq!(slope_total(), parent_total());
        assert_eq!(limited_children(), [Q::int(4), Q::int(4)]);
        // The slope fill keeps the total for any neighbours: the two offsets cancel.
        let [l, r] = slope_children();
        assert_eq!(l + r, Q::int(2) * Q::int(ROW[1]));
    }

    #[test]
    fn the_pressure_repair_balances_every_child() {
        let o = outer();
        assert_eq!(o, [Q::int(-1), q(3, 2), Q::int(-1), q(1, 2)]);
        assert!(four_cell::sum(&o).is_zero());
        assert_eq!(inner_line(), [Q::ONE, Q::ONE, Q::ZERO, Q::ZERO]);
        assert_eq!(imbalance(&o, &inner_line()), [Q::ZERO, q(1, 2), Q::ZERO, q(-1, 2)]);
        assert_eq!(pressure(), [Q::ZERO, q(-1, 8), q(1, 8), q(1, 4)]);
        assert_eq!(inner_fixed(), [q(9, 8), q(7, 8), q(-1, 8), q(-3, 8)]);
        assert!(imbalance(&o, &inner_fixed()).iter().all(|d| d.is_zero()));
        assert!(four_cell::loop_sum(&inner_fixed()).is_zero());
    }

    #[test]
    fn a_disagreeing_seam_cannot_be_fixed_inside() {
        let bad = outer_bad();
        assert_eq!(bad[1], Q::int(2));
        let values = [Q::int(-1), Q::ZERO, q(1, 2), Q::int(2)];
        for a in values {
            for b in values {
                for c in values {
                    for d in values {
                        let inside = [a, b, c, d];
                        assert_eq!(four_cell::sum(&imbalance(&bad, &inside)), q(1, 2));
                    }
                }
            }
        }
        let [line_try, zero_try] = tries();
        assert_eq!(imbalance(&bad, &line_try), [Q::ZERO, Q::ONE, Q::ZERO, q(-1, 2)]);
        assert_eq!(imbalance(&bad, &zero_try), [Q::int(-1), Q::int(2), Q::int(-1), q(1, 2)]);
    }

    #[test]
    fn facts_pin_the_oracles_and_the_run() {
        let f = facts().expect("every pin holds");
        assert_eq!(f.get("split.p"), Some("(0, -1/8, 1/8, 1/4)"));
        assert_eq!(f.get("transfer09.drag.same"), Some("10"));
        assert_eq!(
            f.get("bas.solenoidal.embed"),
            Some("basilisk/src/navier-stokes/centered.h:169 uf.x.refine = refine_face;")
        );
        // 56 pinned or added numbers, seven Basilisk lines and six AMReX sentences.
        assert_eq!(f.len(), 69);
    }

    #[test]
    fn labels_match_the_storyboard() {
        let l = labels();
        assert_eq!(l.small_sum, r"\text{small faces: }\tfrac{3}{2} + \tfrac{1}{2} = 2");
        assert_eq!(l.big_view, r"\text{big cell's own view: }\tfrac{5}{2}");
        assert_eq!(l.leak, r"\tfrac{1}{2}\ \text{lost per step}");
        assert_eq!(l.mended, r"0\ \text{lost per step}");
        assert_eq!(l.cure_one, r"\text{cure 1: mean speed }1 \times \text{length }2 = 2");
        assert_eq!(
            l.cure_two,
            r"\text{cure 2: reflux, pay }\tfrac{1}{2}\text{ back to the big cell}"
        );
        assert_eq!(l.merge_total, "1 + 3 + 2 + 6 = 12");
        assert_eq!(l.merged_total, r"3 \times 4 = 12");
        assert_eq!(l.copy_total, r"6 \times 4 = 24");
        assert_eq!(
            l.split_children,
            r"\text{children: }\tfrac{15}{4} + \tfrac{17}{4} + \tfrac{15}{4} + \tfrac{17}{4} = 16"
        );
        assert_eq!(l.split_parent, r"\text{parent: }4 \times 4 = 16");
        assert_eq!(l.pressures, r"p = (0,\ -\tfrac{1}{8},\ \tfrac{1}{8},\ \tfrac{1}{4})");
        assert_eq!(
            l.outside_good,
            r"\text{outside faces: }-1 + \tfrac{3}{2} - 1 + \tfrac{1}{2} = 0"
        );
        assert_eq!(
            l.outside_bad,
            r"\text{outside faces: }-1 + 2 - 1 + \tfrac{1}{2} = \tfrac{1}{2}"
        );
        assert_eq!(l.bad_sum, r"A + B + C + D = \tfrac{1}{2}");
        assert_eq!(
            l.before,
            r"\text{just before each change: }2.7\text{ to }7.0 \times 10^{-7}"
        );
        assert_eq!(l.after, r"\text{just after it: }0.68\text{ to }3.95");
        assert_eq!(
            l.repaired,
            r"\text{after the run's own pressure repair: }3.6\text{ to }5.9 \times 10^{-7}"
        );
        assert_eq!(l.drag_same, r"\text{unchanged at 10 of 14 changes}");
        assert_eq!(l.drag_jump, r"\text{largest jump }0.003\text{ of }5.45");
        assert_eq!(signed(q(1, 2)), r"+\tfrac{1}{2}");
        assert_eq!(signed(q(-1, 8)), r"-\tfrac{1}{8}");
    }

    #[test]
    fn the_measured_rows_are_ordered_and_the_repair_lowers_every_kick() {
        for row in 0..CHANGES.len() {
            for col in 0..8 {
                assert!(field(row, col).is_finite(), "row {row} col {col}");
            }
            assert!(field(row, REPAIRED) < field(row, AFTER));
            assert!(field(row, BEFORE) < 1e-6 && field(row, REPAIRED) < 1e-6);
            if row > 0 {
                assert!(field(row, T) > field(row - 1, T));
            }
            assert!(field(row, T) > 0.0 && field(row, T) < RUN_SECONDS);
        }
        assert_eq!(largest_drag_jump(), 10);
        assert!(lift_jump_max() < 3e-6);
    }

    #[test]
    fn only_an_eighth_needs_the_drawn_minimum() {
        let mut flows: Vec<Q> = fine_faces().to_vec();
        flows.extend([coarse_own(), consistent_flow(), leak()]);
        flows.extend(outer());
        flows.extend(outer_bad());
        flows.extend(inner_line());
        flows.extend(inner_fixed());
        for inside in tries() {
            flows.extend(inside);
        }
        for value in flows.into_iter().filter(|v| !v.is_zero()) {
            let floored = value.abs().to_f64() * PER_FLOW < MIN_ARROW;
            assert_eq!(floored, value.abs() == q(1, 8), "{value}");
        }
    }

    #[test]
    fn the_picture_fits_its_windows() {
        // Main window x ∈ [−7.1, 0.2]; side panel x ∈ [0.4, 6.95]; frame y ∈ [−4.5, 4.5].
        let in_main = |x: f64| (-7.1..=0.2).contains(&x);
        let in_side = |x: f64| (0.4..=6.95).contains(&x);
        // Inflow arrows and their labels.
        assert!(in_main(INFLOW_LABEL_X - text_width(IN_WORD, SMALL_SIZE) / 2.0));
        const { assert!(INFLOW_START + PER_FLOW <= SEAM_BIG[0] + 1e-12) };
        // B's longest arrow (2 in 15_10) stays in the main window and clear of the side panel.
        assert!(in_main(SEAM_BIG[1] + drawn(Q::int(2))));
        // The big cell's own 5/2 arrow stays inside the big cell.
        assert!(OWN_END - drawn(coarse_own()) > SEAM_BIG[0]);
        // Inside arrows sit on the faces between the children they join, and stay inside them.
        for (f, &(a, b)) in four_cell::FACES.iter().enumerate() {
            let mid = [(KIDS[a][0] + KIDS[b][0]) / 2.0, (KIDS[a][1] + KIDS[b][1]) / 2.0];
            assert!((mid[0] - FACE_MID[f][0]).abs() < 1e-12 && (mid[1] - FACE_MID[f][1]).abs() < 1e-12);
            let dir = [KIDS[b][0] - KIDS[a][0], KIDS[b][1] - KIDS[a][1]];
            assert!((dir[0] / 1.2 - FACE_DIR[f][0]).abs() < 1e-12 && (dir[1] / 1.2 - FACE_DIR[f][1]).abs() < 1e-12);
            assert!(drawn(inner_fixed()[f]) / 2.0 < 0.6);
        }
        // Side-panel words fit the panel.
        for words in [
            MEASURED_HEAD,
            MEASURED_SUB,
            KICK_AXIS,
            DRAG_AXIS,
            RULE_15_4,
            PUZZLE_NOTE,
            MERGE_HEAD,
        ] {
            assert!(text_width(words, 20.0) < 6.4, "{words}");
        }
        for words in &CARDS[..4] {
            assert!(text_width(words, SMALL_SIZE) < CARD_SIZE[0] - 0.3, "{words}");
        }
        // The toy note ends left of the raised "seam" label (15_8 to 15_10).
        assert!(TOY_AT[0] + text_width(TOY_NOTE, 18.0) < SEAM_LABEL_HIGH[0] - text_width(SEAM, 24.0) / 2.0 - 0.3);
        assert!(text_width(QUESTION, 24.0) / 2.0 < PREDICT_SIZE[0]);
        for words in OPTIONS {
            assert!(text_width(words, 21.0) < PREDICT_SIZE[0]);
        }
        // The predict card sits under the two readouts and above the precisely lines.
        const { assert!(PREDICT_AT[1] + PREDICT_SIZE[1] / 2.0 < READOUT_Y[1] - 0.2) };
        const { assert!(PREDICT_AT[1] - PREDICT_SIZE[1] / 2.0 > PRECISELY_Y + 0.3) };
        // The merge and split squares stay in the side panel.
        assert!(in_side(MERGE_X[0] - MERGE_SIDE / 2.0) && in_side(MERGE_X[2] + MERGE_SIDE / 2.0));
        assert!(in_side(SPLIT_X[0] - SPLIT_SIDE / 2.0) && in_side(SPLIT_X[2] + SPLIT_SIDE / 2.0));
        const { assert!(MERGE_X[1] - MERGE_X[0] > MERGE_SIDE && MERGE_X[2] - MERGE_X[1] > MERGE_SIDE) };
        // Cards stack below the question strip without touching.
        const { assert!(CARD_Y[0] + CARD_SIZE[1] / 2.0 < CARD_STRIP.0[1] - CARD_STRIP.1[1] / 2.0) };
        for w in CARD_Y.windows(2) {
            assert!(w[0] - w[1] > CARD_SIZE[1] + 0.05);
        }
        // Precisely lines fit the frame width and stay above its bottom edge.
        for lines in [
            &PRECISELY_15_4[..],
            &PRECISELY_15_5,
            &PRECISELY_15_6,
            &PRECISELY_15_7,
            &PRECISELY_15_9,
            &PRECISELY_15_12,
        ] {
            for words in lines {
                assert!(text_width(words, 19.0) < 14.2, "{words}");
            }
            assert!(precisely_top(lines.len()) - 0.36 * (lines.len() - 1) as f64 > -4.3);
        }
        // 15_12: bars under the axis label, the drag points between the drag axis and its label,
        // and the drag readouts clear of every point to their left.
        let tallest = field_named(extreme(AFTER, true)) * KICK_SCALE;
        assert!(KICK_BASE + tallest < KICK_LABEL_Y - 0.15);
        for row in 0..CHANGES.len() {
            for col in [DRAG_BEFORE, DRAG_AFTER] {
                let y = drag_y(field(row, col));
                assert!(y > DRAG_BASE && y < DRAG_LABEL_Y - 0.25);
                let x = kick_x(field(row, T));
                let readout_left = DRAG_READOUT_AT[0][0] - 1.8;
                assert!(x < readout_left || y < DRAG_READOUT_AT[1][1] - 0.3, "row {row}");
            }
            assert!(in_side(kick_x(field(row, T))));
        }
        const { assert!(KICK_READOUT_Y[2] - 0.15 > DRAG_LABEL_Y + 0.15) };
    }

    #[test]
    fn quotes_agree_with_the_precisely_lines() {
        let joined = |lines: &[&str]| lines.join(" ");
        let mismatch = AMREX
            .iter()
            .find(|a| a.0 == "amrex.flux.mismatch")
            .map(|a| a.1)
            .unwrap_or("");
        assert!(joined(&PRECISELY_15_4).contains(mismatch));
        let remedy = AMREX
            .iter()
            .find(|a| a.0 == "amrex.reflux.remedy")
            .map(|a| a.1)
            .unwrap_or("");
        assert!(joined(&PRECISELY_15_5).contains(remedy));
        let average = AMREX
            .iter()
            .find(|a| a.0 == "amrex.average.down")
            .map(|a| a.1)
            .unwrap_or("");
        assert!(joined(&PRECISELY_15_6).contains(average));
        let divfree = AMREX
            .iter()
            .find(|a| a.0 == "amrex.facedivfree")
            .map(|a| a.1)
            .unwrap_or("");
        assert!(joined(&PRECISELY_15_9).contains(divfree));
        assert!(joined(&PRECISELY_15_6).contains("multigrid-common.h line 47") && BASILISK[0].1.ends_with(":47"));
        assert!(joined(&PRECISELY_15_7).contains("multigrid-common.h line 231") && BASILISK[2].1.ends_with(":231"));
        assert!(joined(&PRECISELY_15_12).contains("refine_face") && BASILISK[5].2.contains("refine_face;"));
        assert_eq!(
            QUESTION_TEX
                .join(" ")
                .replace(r"\text{", "")
                .replace('}', "")
                .replace(r"2\tfrac{1{2", "2½"),
            "The big cell loses 2½, the small cells gain 2: where does the extra half go?"
        );
    }
}
