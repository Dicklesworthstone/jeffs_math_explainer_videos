//! Chapter 17, "Are we solving the right problem?": cues 17_8 to 17_15 and the 17_h2 hold,
//! native and exact. Nothing is simulated, and none of the numbers is ours.
//!
//! RubyRaven `ruby-boundary-tail01` for root's `navier_stokes_simulation` crate (Agent Mail
//! 36228). It continues [`super::boundary17_01`] from the end of 17_7, where
//! the ST96 channel is on screen. Cue text and pictures: `navier-full-script03.md` chapter 17
//! (sha256 31a83a2b…). Numbers: [`super::benchmark01`], ST96 as printed, held exactly;
//! `tools/check_tail01.py` recomputes them from the ST96 text in Fractions.
//!
//! The pictures, in the order the lines ask for them:
//! - 17_8: 17_7's dimensions go and the channel rises; "no slip" on both walls and beside the
//!   cylinder; blue inflow arrows at k H / 8 with the parabola through their tips and "peak
//!   1.5 m/s"; a dashed line at the mean, "average 1 m/s";
//! - 17_9: the fluid tag and "Re = Ū D / ν = 1 × 0.1 / 0.001 = 100", named "Reynolds number";
//!   then the similarity card and "slow air (ch. 5)";
//! - 17_10: the outlet glows, "outlet: each group's choice"; a stand-in for chapter 13's ledger
//!   slides in and gains the row "outlet rule: ?";
//! - 17_11: four cards; drag and lift arrows on the cylinder; then St = D f / Ū; then the
//!   front and back points of ΔP;
//! - 17_12: a Strouhal line headed "method 7a, 1996 (not our solver)"; its three grids, 0.2968,
//!   0.2994 and 0.3003, either side of a dashed 0.300; the changes "+0.0026" and "+0.0009";
//!   then its finest grid at coarser time steps, 0.2881, 0.2973 and 0.3003;
//! - 17_13: "convergence"; the measured band, 0.284 to 0.290; the gap "0.0133 ≈ 15 × 0.0009";
//!   the 10-step dot ringed, with its caution; the not-to-scale "longer run" schematic; the
//!   question card;
//! - 17_h2: the 5 s countdown on the card;
//! - 17_14: the answer; the longer-inlet run's 0.289 lands in the band; the ST96 §4 quote;
//! - 17_15: the cards "convergence", "benchmark" and "validation"; the ledger returns and gains
//!   their three columns, every entry "?" until our runs exist.
//!
//! Published results are labelled where they appear and none enters our ledger. Root owns
//! chapter 18 onward; the three cards and the ledger stay on screen at the end of 17_15.

use std::f64::consts::{FRAC_PI_2, TAU};

use super::benchmark01::{self as bm, GRIDS, LINE, LONGER_INLET, MEASURED, STEPS, Tenk, inlet};
use super::boundary17_01::{
    BODY_PT, CH_ORIGIN, CH_SCALE, Chapter17, FINE_PT, HOLD_SECONDS, PAUSE_NOTE, RING_R, SMALL_PT, chapter17,
};
use super::staircase01::channel;
use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Palette, Timed, hex};
use fmn::prelude::*;

// ------------------------------------------------------------------ layout

/// From 17_8 the channel sits this much higher than in 17_7, freeing the band below it.
pub const CH_LIFT: f64 = 1.2;

/// 17_8: inflow arrows, scene units per m/s; they start at heights k H / 8, k = 1 to 7.
pub const INLET_SCALE: f64 = 0.45;
pub const INLET_ARROWS: usize = 7;
pub const NO_SLIP: &str = "no slip";
/// Above the top wall, below the bottom wall (centres), and right of the cylinder (left edge).
pub const NO_SLIP_AT: [(f64, f64); 3] = [(0.0, 2.45), (0.0, -0.6), (-4.85, 0.9)];
/// Left edges: the peak readout, and the average just above the wall, over its dashed line.
pub const PEAK_AT: (f64, f64) = (-6.6, 3.0);
pub const AVERAGE_AT: (f64, f64) = (-6.6, 2.55);

/// 17_9.
pub const RE_CARD_AT: (f64, f64) = (-3.7, -1.9);
pub const RE_CARD_SIZE: (f64, f64) = (7.4, 1.7);
pub const RE_NAME: &str = "Reynolds number";
/// Name, formula, fluid tag.
pub const RE_Y: [f64; 3] = [-1.32, -1.87, -2.42];
pub const RE_PT: f64 = 34.0;
pub const FLUID_PT: f64 = 26.0;
pub const SIMILAR: [&str; 2] = ["same shape, boundaries, start and Re", "→ same pattern, scaled"];
pub const SIMILAR_AT: (f64, f64) = (3.9, -1.9);
pub const SIMILAR_SIZE: (f64, f64) = (7.0, 1.7);
pub const SIMILAR_Y: [f64; 2] = [-1.62, -2.18];
pub const SLOW_AIR: &str = "slow air (ch. 5)";
pub const SLOW_AIR_AT: (f64, f64) = (3.9, -3.1);

/// 17_10: the outlet tag's right edge, inside the channel at mid-height.
pub const OUTLET: &str = "outlet: each group's choice";
pub const OUTLET_AT: (f64, f64) = (6.35, 0.93);

/// The ledger: a stand-in for chapter 13's (13_2: what we changed, what we measured, what that
/// comparison can tell us; blank cells read "not measured"), with rows from 13_3, 13_4 and
/// 13_8. Root replaces it with chapter 13's own ledger once that chapter is built.
pub const LEDGER_TITLE: &str = "ledger (chapter 13)";
pub const LEDGER_HEADS: [&str; 3] = ["what we changed", "what we measured", "what it can tell us"];
pub const LEDGER_ROWS: [&str; 4] = ["solver tolerance", "time step", "cells and step together", "rounding"];
pub const NOT_MEASURED: &str = "not measured";
pub const OUTLET_ROW: &str = "outlet rule: ?";
pub const LEDGER_CHECKS: [&str; 3] = ["convergence", "benchmark", "validation"];
pub const OPEN: &str = "?";
pub const OURS: &str = "our runs: each new entry stays ? until we run them";
pub const LEDGER_LEFT: f64 = -7.55;
pub const LEDGER_TOP: f64 = -0.9;
pub const LEDGER_BOTTOM: f64 = -4.25;
pub const LEDGER_LABEL_W: f64 = 3.2;
pub const LEDGER_COL_W: f64 = 2.35;
pub const LEDGER_ROW_STEP: f64 = 0.42;
/// The ledger slides in from this far to the right.
pub const LEDGER_SLIDE: f64 = 2.0;

/// 17_11. Card 3's second line is [`ST_TEX`].
pub const FORCE_CARDS: [&str; 4] = [
    "drag coefficient (peak)",
    "lift coefficient (peak)",
    "Strouhal number",
    "ΔP, front − back",
];
pub const FORCE_NOTES: [&str; 4] = ["along the flow", "across the flow", "", "pressure difference"];
pub const ST_TEX: &str = r"\mathrm{St} = D f / \bar{U}";
pub const ST_PT: f64 = 30.0;
pub const FORCE_CARD_AT: [(f64, f64); 4] = [(2.525, -1.725), (5.975, -1.725), (2.525, -3.475), (5.975, -3.475)];
pub const FORCE_CARD_SIZE: (f64, f64) = (3.25, 1.55);
pub const DRAG_LEN: f64 = 0.85;
pub const LIFT_LEN: f64 = 0.7;

/// 17_12: the Strouhal line ([`LINE`]) and its two rows.
pub const AXIS_Y: f64 = -0.95;
pub const AXIS_NAME: &str = "St";
/// Right edge.
pub const AXIS_NAME_AT: (f64, f64) = (-6.2, -0.95);
pub const TICK_STEP: i64 = 50;
pub const TICK: f64 = 0.08;
pub const TICK_LABEL_Y: f64 = -1.3;
pub const HEADER: &str = "method 7a, 1996 (not our solver)";
pub const HEADER_AT: (f64, f64) = (-7.4, 3.22);
pub const HEADER_SOURCE: &str = "Schaefer and Turek (1996), Table 4";
pub const HEADER_SOURCE_AT: (f64, f64) = (-3.15, 3.22);
pub const ROW_Y: [f64; 2] = [2.6, 1.15];
pub const ROW_LABEL: [&str; 2] = ["three grids, each with ×4 unknowns", "finest grid, coarser time steps"];
pub const ROW_LABEL_X: f64 = -7.4;
/// Dot labels and row sub-labels sit this far below their row.
pub const LABEL_DROP: f64 = 0.35;
/// Each dot label: −1 ends at its dot, 0 is centred on it, 1 starts at it.
pub const ROW_ANCHORS: [[i32; 3]; 2] = [[0, -1, 1], [0, 0, 1]];
pub const DOT_R: f64 = 0.07;
pub const ARROW_BUFF: f64 = 0.13;
/// Row 1's change labels sit this far above it.
pub const CHANGE_LIFT: f64 = 0.35;
pub const GUIDE_TOP: f64 = 2.75;

/// 17_13.
pub const CONVERGENCE: &str = "convergence";
pub const CONVERGENCE_AT: (f64, f64) = (3.93, 3.35);
pub const BAND_HALF: f64 = 0.15;
pub const BAND_LABEL_Y: [f64; 2] = [-1.72, -2.06];
pub const BAND_SOURCE: &str = "same 1996 report, §4";
pub const GAP_Y: f64 = -0.3;
pub const CAUTION: [&str; 2] = [
    "least time-resolved short-inlet computation:",
    "agreement here may be accidental",
];
pub const CAUTION_AT: [(f64, f64); 2] = [(-3.3, 1.9), (-3.3, 1.57)];
pub const CAUTION_R: f64 = 0.14;
/// The schematic, not to scale: wall heights, and x of the experiment's inlet, the computed
/// inlet and the outlet; the cylinder (x, y, r).
pub const SCHEMA_WALLS: [f64; 2] = [-3.0, -3.75];
pub const SCHEMA_X: [f64; 3] = [-7.2, -4.0, -0.6];
pub const SCHEMA_CYLINDER: (f64, f64, f64) = (-3.55, -3.375, 0.15);
pub const EXPERIMENT: &str = "experiment: longer run before the cylinder";
pub const EXPERIMENT_AT: (f64, f64) = (-7.2, -2.68);
pub const COMPUTED: &str = "computed";
pub const COMPUTED_AT: (f64, f64) = (-2.05, -3.375);
pub const NOT_TO_SCALE: &str = "not to scale";
pub const NOT_TO_SCALE_AT: (f64, f64) = (-7.2, -4.05);
pub const Q_CARD_AT: (f64, f64) = (4.0, -2.85);
pub const Q_CARD_SIZE: (f64, f64) = (7.1, 2.4);
pub const Q_LABEL: &str = "your prediction";
pub const QUESTION2: [&str; 2] = ["Settled near 0.3003, measured 0.287 ± 0.003:", "what closes the gap?"];
pub const CHOICES2: [&str; 3] = ["more cells", "finer time steps", "a longer run before the cylinder"];
/// Index into [`CHOICES2`].
pub const ANSWER2: usize = 2;
pub const CHOICE_STEP: f64 = 0.35;

/// 17_14.
pub const LONG_LABEL_DY: f64 = 0.4;
pub const LONG_NOTE: [&str; 2] = [
    "0.289: same report, inlet moved farther upstream",
    "(the paper gives neither its grid nor its time step)",
];
pub const LONG_NOTE_AT: [(f64, f64); 2] = [(0.6, -1.85), (0.6, -2.17)];
pub const STANDARD: &str = "standard channel";
pub const STANDARD_AT: (f64, f64) = (5.15, -0.3);
pub const QUOTE17: [&str; 3] = [
    "“owing to the short distance between the inlet and the",
    "cylinder for the computations, the flow conditions",
    "in front of the cylinder are slightly different.”",
];
pub const QUOTE17_SOURCE: &str = "Schaefer and Turek (1996), §4";
pub const QUOTE_CARD_AT: (f64, f64) = (4.0, -3.32);
pub const QUOTE_CARD_SIZE: (f64, f64) = (7.1, 1.56);
/// Three quote lines, then the source.
pub const QUOTE_Y: [f64; 4] = [-2.8, -3.1, -3.4, -3.8];

/// 17_15: each card's head, then its question in two lines.
pub const CHECKS: [[&str; 3]; 3] = [
    ["convergence:", "did we solve our", "equations well?"],
    ["benchmark:", "do we agree with careful", "answers to the same problem?"],
    ["validation:", "does the problem we chose", "match a real measurement?"],
];
pub const CHECK_AT: [(f64, f64); 3] = [(-5.05, 1.75), (0.0, 1.75), (5.05, 1.75)];
pub const CHECK_SIZE: (f64, f64) = (4.85, 1.9);
pub const CHECK_Y: [f64; 3] = [2.3, 1.75, 1.35];

// ------------------------------------------------------------------ style

const WALL_W: f64 = 4.0;
const CURVE_W: f64 = 3.0;
const INLET_T: f64 = 2.5;
const FORCE_T: f64 = 4.0;
const CHANGE_T: f64 = 2.5;
const AXIS_W: f64 = 2.5;
const TICK_W: f64 = 2.0;
const GUIDE_W: f64 = 1.5;
const RULE_W: f64 = 1.5;
const BAND_W: f64 = 2.0;
const SCHEMA_W: f64 = 3.0;
const MARK_W: f64 = 3.0;

const Z_LINE: i32 = 2;
const Z_MARK: i32 = 5;
const Z_TEXT: i32 = 8;

// Every duration is a multiple of 1/4 s: exact in binary, whole frames at 30 and 60 fps.
const FADE: f64 = 0.5;
const QUICK: f64 = 0.25;
const DRAW: f64 = 1.0;
const MOVE: f64 = 0.75;

/// Labels closer than this (scene units; 24 px at 4K) count as touching.
const LABEL_GAP: f64 = 0.1;

fn solid_grey() -> Srgb {
    hex("#3B4352")
}

fn card_fill() -> Srgb {
    hex("#141C29")
}

fn band_white() -> Srgb {
    hex("#FFFFFF")
}

/// 17_15's card colours, also their ledger columns'.
fn check_color(k: usize) -> Srgb {
    [Palette::green(), Palette::blue(), Palette::gold()][k]
}

// ----------------------------------------------------------------- helpers

fn v(x: f64, y: f64) -> Vec3 {
    [x, y, 0.0]
}

/// A point of the channel, metres from the inlet's lower corner, in scene units: 17_7's
/// scale, lifted by [`CH_LIFT`].
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
    let m = text(kit, stage, s, size, color, at)?;
    stage.move_to(m, v(at.0, at.1), LEFT);
    Ok(m)
}

fn text_right(kit: &Kit, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: (f64, f64)) -> fmn::Result<Mob> {
    let m = text(kit, stage, s, size, color, at)?;
    stage.move_to(m, v(at.0, at.1), RIGHT);
    Ok(m)
}

/// Text anchored as [`ROW_ANCHORS`] says: −1 ends at `at`, 0 is centred on it, 1 starts at it.
fn text_anchored(
    kit: &Kit,
    stage: &mut Stage<'_>,
    s: &str,
    color: Srgb,
    at: (f64, f64),
    anchor: i32,
) -> fmn::Result<Mob> {
    match anchor {
        -1 => text_right(kit, stage, s, SMALL_PT, color, at),
        1 => text_left(kit, stage, s, SMALL_PT, color, at),
        _ => text(kit, stage, s, SMALL_PT, color, at),
    }
}

fn tex(kit: &Kit, stage: &mut Stage<'_>, src: &str, size: f64, color: Srgb, at: (f64, f64)) -> fmn::Result<Mob> {
    let m = kit.tex(stage, src, size, color, &[], v(at.0, at.1))?;
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

/// The dashes of a straight line from `a` to `b`, starting and ending with a dash, dashes and
/// gaps about `dash` long.
fn dashes(a: Vec3, b: Vec3, dash: f64, style: Style) -> Vec<VMobject> {
    let len = (b[0] - a[0]).hypot(b[1] - a[1]);
    let count = ((len / dash + 1.0) / 2.0).round().max(1.0) as usize;
    let piece = len / (2 * count - 1) as f64;
    let at = |t: f64| v(a[0] + (b[0] - a[0]) * t / len, a[1] + (b[1] - a[1]) * t / len);
    (0..count)
        .map(|k| seg(at(2.0 * k as f64 * piece), at((2 * k + 1) as f64 * piece), style))
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

/// Refuse the scene if any two of `mobs` come within [`LABEL_GAP`] of each other.
fn refuse_touching(stage: &Stage<'_>, mobs: &[Mob], what: &str) -> fmn::Result<()> {
    let b = boxes(stage, mobs);
    for i in 0..b.len() {
        for j in i + 1..b.len() {
            let apart_x = b[i][1] + LABEL_GAP <= b[j][0] || b[j][1] + LABEL_GAP <= b[i][0];
            let apart_y = b[i][3] + LABEL_GAP <= b[j][2] || b[j][3] + LABEL_GAP <= b[i][2];
            if !(apart_x || apart_y) {
                return Err(scene_error(format!("{what}: labels {i} and {j} touch")));
            }
        }
    }
    Ok(())
}

/// Refuse the scene if the vertical line at `x` passes within [`LABEL_GAP`] of any of `mobs`.
fn refuse_crossing(stage: &Stage<'_>, mobs: &[Mob], x: f64, what: &str) -> fmn::Result<()> {
    for (k, b) in boxes(stage, mobs).iter().enumerate() {
        if b[0] - LABEL_GAP < x && x < b[1] + LABEL_GAP {
            return Err(scene_error(format!("{what}: label {k} meets the line at x = {x}")));
        }
    }
    Ok(())
}

// ----------------------------------------------------------------- phrases

/// 17_12's second sentence, phrase by phrase: one dot each.
pub const ROW1_SAID: [&str; 3] = [
    "Near point two nine seven",
    "then just under point three",
    "then just over it",
];

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

/// Where `phrase` begins in sentence `j` of line `id` (of [`SCRIPT17_TAIL`]), as a share of
/// that sentence's characters.
pub fn phrase_share(id: &str, j: usize, phrase: &str) -> Option<f64> {
    let line = SCRIPT17_TAIL
        .lines()
        .find_map(|row| row.strip_prefix(id)?.strip_prefix('\t'))?;
    let sentence = *sentences(line).get(j)?;
    let at = sentence.find(phrase)?;
    Some(sentence[..at].chars().count() as f64 / sentence.chars().count() as f64)
}

/// Wait until `phrase` of sentence `j` of line `id` is about to be said, so a readout lands
/// with its words rather than ahead of them: the sentence's start (the narration's own, from
/// the take's samples once recorded) plus the phrase's share of the sentence's text times the
/// time to the next sentence. The share is an estimate by text length, and the pause after
/// the sentence is counted in, so a readout lands on its words or a little after. `t0` is the
/// scene time the line began, read right after `kit.say`. Without a narrator, or in a line's
/// last sentence, it waits for the sentence's start.
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

// ----------------------------------------------------------------- the screen

/// Mobjects on screen. Every fade goes through it, so a hide never re-roots a mobject that is
/// already gone (the double-fade flash) and a show never fades in a mobject twice. It takes
/// over from 17_1–17_7's tracker, whose list is private, at the known end state of 17_7.
#[derive(Default)]
pub struct Screen(Vec<Mob>);

impl Screen {
    pub fn adopt(mobs: impl IntoIterator<Item = Mob>) -> Self {
        Self(mobs.into_iter().collect())
    }

    pub fn on(&self) -> &[Mob] {
        &self.0
    }

    /// Fade `out` away and `into` in together, each of `into` arriving by `slide` (zero for a
    /// plain fade), skipping any already in the wanted state.
    fn fades(&mut self, stage: &mut Stage<'_>, out: &[Mob], into: &[Mob], slide: Vec3, rt: f64) -> fmn::Result<()> {
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
        if !anims.is_empty() {
            stage.play_prepared(anims)?;
        }
        Ok(())
    }

    pub fn swap(&mut self, stage: &mut Stage<'_>, out: &[Mob], into: &[Mob], rt: f64) -> fmn::Result<()> {
        self.fades(stage, out, into, ORIGIN, rt)
    }

    pub fn show(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
        self.fades(stage, &[], mobs, ORIGIN, rt)
    }

    pub fn hide(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
        self.fades(stage, mobs, &[], ORIGIN, rt)
    }

    /// Fade `mobs` in as they slide by `slide` into place.
    pub fn slide_in(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], slide: Vec3, rt: f64) -> fmn::Result<()> {
        self.fades(stage, &[], mobs, slide, rt)
    }

    /// Move the on-screen `mobs` by `by`, together.
    pub fn shift(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], by: Vec3, rt: f64) -> fmn::Result<()> {
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in mobs {
            if self.0.contains(&m) {
                let a = m
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(rt),
                        ..AnimateArgs::default()
                    })?
                    .shift(by)?;
                anims.push(stage.prepare(a)?);
            }
        }
        if !anims.is_empty() {
            stage.play_prepared(anims)?;
        }
        Ok(())
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

// ----------------------------------------------------------------- the ledger

/// The ledger stand-in: built once, shown in 17_10 with two columns beside the labels, and in
/// 17_15 with three more.
pub struct Ledger {
    pub narrow: Mob,
    pub wide: Mob,
    pub title: Mob,
    pub ours: Mob,
    pub heads: Vec<Mob>,
    pub rule: Mob,
    pub wide_rule: Mob,
    /// Labels and cells of the chapter 13 rows.
    pub rows: Vec<Mob>,
    /// The 17_10 row.
    pub outlet: Vec<Mob>,
    /// The 17_15 columns: each head, then its cells.
    pub columns: Vec<Vec<Mob>>,
}

impl Ledger {
    /// Centre of data column `k` (0 and 1 from chapter 13, 2 to 4 from 17_15).
    pub fn col_x(k: usize) -> f64 {
        LEDGER_LEFT + LEDGER_LABEL_W + LEDGER_COL_W * (k as f64 + 0.5)
    }

    pub fn row_y(k: usize) -> f64 {
        LEDGER_TOP - 1.27 - LEDGER_ROW_STEP * k as f64
    }

    pub fn right(columns: usize) -> f64 {
        LEDGER_LEFT + LEDGER_LABEL_W + LEDGER_COL_W * columns as f64
    }

    fn back(stage: &mut Stage<'_>, columns: usize) -> fmn::Result<Mob> {
        let right = Self::right(columns);
        card(
            stage,
            (0.5 * (LEDGER_LEFT + right), 0.5 * (LEDGER_TOP + LEDGER_BOTTOM)),
            (right - LEDGER_LEFT, LEDGER_TOP - LEDGER_BOTTOM),
        )
    }

    fn rule(stage: &mut Stage<'_>, columns: usize) -> Mob {
        let y = LEDGER_TOP - 0.95;
        let line = seg(
            v(LEDGER_LEFT + 0.15, y),
            v(Self::right(columns) - 0.15, y),
            line_style(Palette::faint(), RULE_W, 1.0),
        );
        add(stage, line, Z_TEXT)
    }

    fn row(kit: &Kit, stage: &mut Stage<'_>, k: usize, label: &str, color: Srgb) -> fmn::Result<Vec<Mob>> {
        let y = Self::row_y(k);
        let mut out = vec![text_left(kit, stage, label, FINE_PT, color, (LEDGER_LEFT + 0.25, y))?];
        for c in 0..2 {
            out.push(text(
                kit,
                stage,
                NOT_MEASURED,
                FINE_PT,
                Palette::dim(),
                (Self::col_x(c), y),
            )?);
        }
        Ok(out)
    }

    pub fn build(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let (ink, dim) = (Palette::ink(), Palette::dim());
        let narrow = Self::back(stage, 2)?;
        let wide = Self::back(stage, 5)?;
        let x_label = LEDGER_LEFT + 0.25;
        let title_y = LEDGER_TOP - 0.3;
        let title = text_left(kit, stage, LEDGER_TITLE, FINE_PT, dim, (x_label, title_y))?;
        let ours = text_right(kit, stage, OURS, FINE_PT, dim, (Self::right(5) - 0.25, title_y))?;
        let heads_y = LEDGER_TOP - 0.72;
        let mut heads = vec![text_left(
            kit,
            stage,
            LEDGER_HEADS[0],
            FINE_PT,
            ink,
            (x_label, heads_y),
        )?];
        for (c, head) in LEDGER_HEADS[1..].iter().enumerate() {
            heads.push(text(kit, stage, head, FINE_PT, ink, (Self::col_x(c), heads_y))?);
        }
        let rule = Self::rule(stage, 2);
        let wide_rule = Self::rule(stage, 5);
        let mut rows = Vec::new();
        for (k, label) in LEDGER_ROWS.iter().enumerate() {
            rows.extend(Self::row(kit, stage, k, label, ink)?);
        }
        let outlet = Self::row(kit, stage, LEDGER_ROWS.len(), OUTLET_ROW, Palette::hero())?;
        let mut columns = Vec::new();
        for (c, name) in LEDGER_CHECKS.iter().enumerate() {
            let x = Self::col_x(c + 2);
            let mut column = vec![text(kit, stage, name, FINE_PT, check_color(c), (x, heads_y))?];
            for r in 0..=LEDGER_ROWS.len() {
                column.push(text(kit, stage, OPEN, FINE_PT, dim, (x, Self::row_y(r)))?);
            }
            columns.push(column);
        }
        Ok(Self {
            narrow,
            wide,
            title,
            ours,
            heads,
            rule,
            wide_rule,
            rows,
            outlet,
            columns,
        })
    }

    /// The ledger as chapter 13 left it: card, title, heads, rule and rows.
    pub fn base(&self) -> Vec<Mob> {
        [
            vec![self.narrow, self.title],
            self.heads.clone(),
            vec![self.rule],
            self.rows.clone(),
        ]
        .concat()
    }
}

// ----------------------------------------------------------------- the card

/// The 17_h2 prediction card.
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
    pub fn build(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let (cx, cy) = Q_CARD_AT;
        let (w, h) = Q_CARD_SIZE;
        let (left, right, top) = (cx - w / 2.0, cx + w / 2.0, cy + h / 2.0);
        let (ink, dim) = (Palette::ink(), Palette::dim());
        let mut parts = vec![card(stage, Q_CARD_AT, Q_CARD_SIZE)?];
        parts.push(text_left(kit, stage, Q_LABEL, FINE_PT, dim, (left + 0.35, top - 0.27))?);
        parts.push(text_left(
            kit,
            stage,
            QUESTION2[0],
            BODY_PT,
            ink,
            (left + 0.35, top - 0.67),
        )?);
        parts.push(text_left(
            kit,
            stage,
            QUESTION2[1],
            BODY_PT,
            ink,
            (left + 0.35, top - 1.02),
        )?);
        let x = left + 0.45;
        let mut markers = Vec::new();
        let mut chosen = None;
        for (k, choice) in CHOICES2.iter().enumerate() {
            let y = top - 1.42 - CHOICE_STEP * k as f64;
            parts.push(text_left(kit, stage, choice, BODY_PT, ink, (x + 0.3, y))?);
            markers.push(ring(stage, v(x, y), 0.1, dim, 2.5));
            if k == ANSWER2 {
                chosen = Some(dot(stage, v(x, y), 0.1, Palette::green()));
            }
        }
        let note = text_right(kit, stage, PAUSE_NOTE, FINE_PT, dim, (right - 0.3, top - 0.27))?;
        let ring_at = (right - 0.7, top - 1.42 - CHOICE_STEP);
        let track = ring(stage, v(ring_at.0, ring_at.1), RING_R, Palette::faint(), 6.0);
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
            chosen: chosen.ok_or_else(|| scene_error("ANSWER2 is not a choice".to_owned()))?,
            ring: countdown,
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

// ----------------------------------------------------------------- the tail

/// Chapter 17 from 17_8: what is on screen, and every group of objects a later cue changes.
pub struct Tail {
    pub screen: Screen,
    pub title: Option<Mob>,
    pub walls: Mob,
    pub cylinder: Mob,
    dims: Vec<Mob>,
    dress: Vec<Mob>,
    re_cards: Vec<Mob>,
    ledger: Option<Ledger>,
    line: Vec<Mob>,
    schematic: Vec<Mob>,
    longer: Option<Mob>,
}

impl Tail {
    /// Take over from 17_7. At its end the title (when drawn) and the channel are on screen:
    /// walls, cylinder, then 13 dimension lines and labels (`cue_17_7`'s `channel`).
    pub fn adopt(ch: Chapter17) -> fmn::Result<Self> {
        if ch.channel.len() != 15 {
            return Err(scene_error(format!(
                "17_7 left {} channel objects, not 15",
                ch.channel.len()
            )));
        }
        let screen = Screen::adopt(ch.title.into_iter().chain(ch.channel.iter().copied()));
        Ok(Self {
            screen,
            title: ch.title,
            walls: ch.channel[0],
            cylinder: ch.channel[1],
            dims: ch.channel[2..].to_vec(),
            dress: Vec::new(),
            re_cards: Vec::new(),
            ledger: None,
            line: Vec::new(),
            schematic: Vec::new(),
            longer: None,
        })
    }

    /// Everything but the title goes.
    fn clear(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let out: Vec<Mob> = self
            .screen
            .on()
            .iter()
            .copied()
            .filter(|&m| Some(m) != self.title)
            .collect();
        self.screen.hide(stage, &out, FADE)
    }

    // ---------------------------------------------------------------- 17_8

    pub fn cue_17_8(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "17_8")?;
        let t0 = stage.scene().time().to_f64();
        let dims = std::mem::take(&mut self.dims);
        self.screen.hide(stage, &dims, FADE)?;
        self.screen
            .shift(stage, &[self.walls, self.cylinder], v(0.0, CH_LIFT), MOVE)?;
        let teal = Palette::teal();
        let tags = vec![
            text(kit, stage, NO_SLIP, SMALL_PT, teal, NO_SLIP_AT[0])?,
            text(kit, stage, NO_SLIP, SMALL_PT, teal, NO_SLIP_AT[1])?,
            text_left(kit, stage, NO_SLIP, SMALL_PT, teal, NO_SLIP_AT[2])?,
        ];
        self.screen.show(stage, &tags, FADE)?;
        kit.until_sentence(stage, "17_8", 1)?;
        // The inflow, U(0, y) = 4 Um y (H − y) / H²: arrows at k H / 8, the parabola through
        // their tips.
        let blue = Palette::blue();
        let tip = |s: f64| {
            let p = ch_pt(0.0, s * inlet::HEIGHT);
            v(p[0] + INLET_SCALE * inlet::speed_at_share(s), p[1])
        };
        let mut arrows = Vec::new();
        for k in 1..=INLET_ARROWS {
            let s = k as f64 / (INLET_ARROWS + 1) as f64;
            arrows.push(arrow(stage, ch_pt(0.0, s * inlet::HEIGHT), tip(s), blue, INLET_T)?);
        }
        let curve: Vec<Vec3> = (0..=64).map(|m| tip(m as f64 / 64.0)).collect();
        let parabola = add(
            stage,
            polyline(curve).style(line_style(blue, CURVE_W, 1.0)).build(),
            Z_MARK,
        );
        self.screen.show(stage, &arrows, FADE)?;
        self.screen.create(stage, &[parabola], DRAW)?;
        until_phrase(kit, stage, t0, "17_8", 1, "at one and a half meters per second")?;
        let peak = text_left(kit, stage, &bm::peak_label(), BODY_PT, blue, PEAK_AT)?;
        self.screen.show(stage, &[peak], FADE)?;
        kit.until_sentence(stage, "17_8", 2)?;
        // The average, 1 m/s: a dashed line from the bottom wall up to its readout. The mean
        // over the drawn profile must be the paper's 2 Um / 3.
        if inlet::mean() != inlet::paper_mean() {
            return Err(scene_error("the inflow's mean is not ST96's 2 Um / 3".to_owned()));
        }
        let (mean_n, mean_d) = inlet::mean();
        let x = ch_pt(0.0, 0.0)[0] + INLET_SCALE * mean_n as f64 / mean_d as f64;
        let ink = Palette::ink();
        let mean = group(
            stage,
            dashes(
                v(x, ch_pt(0.0, 0.0)[1]),
                v(x, ch_pt(0.0, inlet::HEIGHT)[1] + 0.12),
                0.1,
                line_style(ink, MARK_W, 1.0),
            ),
            Z_MARK,
        );
        let average = text_left(kit, stage, &bm::average_label(), BODY_PT, ink, AVERAGE_AT)?;
        self.screen.show(stage, &[mean, average], FADE)?;
        self.dress = [tags, arrows, vec![parabola, peak, mean, average]].concat();
        Ok(())
    }

    // ---------------------------------------------------------------- 17_9

    pub fn cue_17_9(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "17_9")?;
        let t0 = stage.scene().time().to_f64();
        let (ink, dim) = (Palette::ink(), Palette::dim());
        let back = card(stage, RE_CARD_AT, RE_CARD_SIZE)?;
        let fluid = tex(kit, stage, &bm::fluid_tex(), FLUID_PT, dim, (RE_CARD_AT.0, RE_Y[2]))?;
        self.screen.show(stage, &[back, fluid], FADE)?;
        until_phrase(kit, stage, t0, "17_9", 0, "one number, average speed")?;
        let formula = tex(kit, stage, &bm::re_tex(), RE_PT, ink, (RE_CARD_AT.0, RE_Y[1]))?;
        self.screen.show(stage, &[formula], FADE)?;
        kit.until_sentence(stage, "17_9", 1)?;
        let name = text(kit, stage, RE_NAME, SMALL_PT, dim, (RE_CARD_AT.0, RE_Y[0]))?;
        self.screen.show(stage, &[name], FADE)?;
        kit.until_sentence(stage, "17_9", 2)?;
        let similar = vec![
            card(stage, SIMILAR_AT, SIMILAR_SIZE)?,
            text(kit, stage, SIMILAR[0], BODY_PT, ink, (SIMILAR_AT.0, SIMILAR_Y[0]))?,
            text(kit, stage, SIMILAR[1], BODY_PT, ink, (SIMILAR_AT.0, SIMILAR_Y[1]))?,
        ];
        self.screen.show(stage, &similar, FADE)?;
        kit.until_sentence(stage, "17_9", 3)?;
        let slow = text(kit, stage, SLOW_AIR, SMALL_PT, dim, SLOW_AIR_AT)?;
        self.screen.show(stage, &[slow], FADE)?;
        self.re_cards = [vec![back, fluid, formula, name], similar, vec![slow]].concat();
        Ok(())
    }

    // --------------------------------------------------------------- 17_10

    pub fn cue_17_10(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "17_10")?;
        let hero = Palette::hero();
        let (l, h) = (channel::LENGTH, channel::HEIGHT);
        let outlet = add(
            stage,
            seg(ch_pt(l, 0.0), ch_pt(l, h), line_style(hero, WALL_W, 1.0)),
            Z_MARK,
        );
        let tag = text_right(kit, stage, OUTLET, SMALL_PT, hero, OUTLET_AT)?;
        self.screen.show(stage, &[outlet, tag], FADE)?;
        self.dress.extend([outlet, tag]);
        kit.until_sentence(stage, "17_10", 1)?;
        // The 17_9 cards make way for chapter 13's ledger, which gains a row.
        let cards = std::mem::take(&mut self.re_cards);
        self.screen.hide(stage, &cards, FADE)?;
        let ledger = Ledger::build(kit, stage)?;
        self.screen
            .slide_in(stage, &ledger.base(), v(-LEDGER_SLIDE, 0.0), MOVE)?;
        self.screen.show(stage, &ledger.outlet, FADE)?;
        self.ledger = Some(ledger);
        Ok(())
    }

    // --------------------------------------------------------------- 17_11

    pub fn cue_17_11(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "17_11")?;
        // 17_8's inflow and tags go (the outlet stays); four empty cards.
        let keep_outlet = self.dress.split_off(self.dress.len() - 2);
        let dress = std::mem::replace(&mut self.dress, keep_outlet);
        let backs: Vec<Mob> = FORCE_CARD_AT
            .iter()
            .map(|&at| card(stage, at, FORCE_CARD_SIZE))
            .collect::<fmn::Result<_>>()?;
        self.screen.swap(stage, &dress, &backs, FADE)?;
        self.dress.extend(&backs);
        let colors = [Palette::gold(), Palette::teal(), Palette::blue(), Palette::hero()];
        let mut contents = Vec::new();
        for (k, &(cx, cy)) in FORCE_CARD_AT.iter().enumerate() {
            let head = text(kit, stage, FORCE_CARDS[k], SMALL_PT, colors[k], (cx, cy + 0.3))?;
            let note = if k == 2 {
                tex(kit, stage, ST_TEX, ST_PT, Palette::ink(), (cx, cy - 0.3))?
            } else {
                text(kit, stage, FORCE_NOTES[k], FINE_PT, Palette::dim(), (cx, cy - 0.3))?
            };
            contents.push([head, note]);
        }
        kit.until_sentence(stage, "17_11", 1)?;
        // Peak forces: along the flow (drag) and across it (lift).
        let c = ch_pt(channel::CENTRE.0, channel::CENTRE.1);
        let r = CH_SCALE * channel::DIAMETER / 2.0;
        let drag = arrow(
            stage,
            v(c[0] + r + 0.15, c[1]),
            v(c[0] + r + 0.15 + DRAG_LEN, c[1]),
            colors[0],
            FORCE_T,
        )?;
        let lift = arrow(
            stage,
            v(c[0], c[1] + r + 0.08),
            v(c[0], c[1] + r + 0.08 + LIFT_LEN),
            colors[1],
            FORCE_T,
        )?;
        let shown = [vec![drag, lift], contents[0].to_vec(), contents[1].to_vec()].concat();
        self.screen.show(stage, &shown, FADE)?;
        kit.until_sentence(stage, "17_11", 2)?;
        self.screen.show(stage, &contents[2], FADE)?;
        kit.until_sentence(stage, "17_11", 3)?;
        // ΔP: the points in front of and behind the cylinder (ST96 §2.2).
        let mut points = Vec::new();
        for &(x, y) in &bm::PRESSURE_POINTS {
            let off = (x - channel::CENTRE.0).hypot(y - channel::CENTRE.1) - channel::DIAMETER / 2.0;
            if off.abs() > 1e-12 {
                return Err(scene_error(format!(
                    "pressure point ({x}, {y}) is {off} m off the cylinder"
                )));
            }
            points.push(dot(stage, ch_pt(x, y), DOT_R, colors[3]));
        }
        self.screen
            .show(stage, &[points.clone(), contents[3].to_vec()].concat(), FADE)?;
        self.dress.extend([drag, lift]);
        self.dress.extend(points);
        self.dress.extend(contents.iter().flatten());
        Ok(())
    }

    // --------------------------------------------------------------- 17_12

    /// One row of dots on the Strouhal line, with their value labels.
    fn row(
        &mut self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        k: usize,
        runs: &[bm::Run; 3],
    ) -> fmn::Result<(Vec<Mob>, Vec<Mob>, Vec<Mob>)> {
        let y = ROW_Y[k];
        let ink = Palette::ink();
        let (mut dots, mut labels, mut arrows) = (Vec::new(), Vec::new(), Vec::new());
        for (j, run) in runs.iter().enumerate() {
            let x = LINE.x(run.st);
            dots.push(dot(stage, v(x, y), DOT_R, ink));
            labels.push(text_anchored(
                kit,
                stage,
                &run.st.four(),
                ink,
                (x, y - LABEL_DROP),
                ROW_ANCHORS[k][j],
            )?);
        }
        let color = if k == 0 { ink } else { Palette::dim() };
        for w in runs.windows(2) {
            let (a, b) = (LINE.x(w[0].st), LINE.x(w[1].st));
            arrows.push(arrow(
                stage,
                v(a + ARROW_BUFF, y),
                v(b - ARROW_BUFF, y),
                color,
                CHANGE_T,
            )?);
        }
        Ok((dots, labels, arrows))
    }

    pub fn cue_17_12(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "17_12")?;
        let t0 = stage.scene().time().to_f64();
        // What the row labels and the line claim: four times the unknowns each time, and every
        // value of these rows and the band inside the drawn line (0.289 waits for 17_14).
        if bm::unknowns_ratios() != [Some(4), Some(4)] {
            return Err(scene_error(
                "row 1 says each grid has four times the unknowns".to_owned(),
            ));
        }
        let (lo, hi) = bm::band();
        for value in GRIDS.iter().chain(&STEPS).map(|r| r.st).chain([lo, hi]) {
            if !LINE.contains(value) {
                return Err(scene_error(format!("{} is off the Strouhal line", value.four())));
            }
        }
        self.clear(stage)?;
        self.dress.clear();
        let (ink, dim) = (Palette::ink(), Palette::dim());
        // The line, its ticks, and what is on it.
        let axis = add(
            stage,
            seg(v(LINE.x0, AXIS_Y), v(LINE.x1, AXIS_Y), line_style(dim, AXIS_W, 1.0)),
            Z_LINE,
        );
        let mut ticks = Vec::new();
        let mut tick_labels = Vec::new();
        for t in LINE.ticks(TICK_STEP) {
            let x = LINE.x(t);
            ticks.push(seg(
                v(x, AXIS_Y - TICK),
                v(x, AXIS_Y + TICK),
                line_style(dim, TICK_W, 1.0),
            ));
            let label = t
                .three()
                .ok_or_else(|| scene_error(format!("tick {t:?} has four places")))?;
            tick_labels.push(text(kit, stage, &label, FINE_PT, dim, (x, TICK_LABEL_Y))?);
        }
        let ticks = group(stage, ticks, Z_LINE);
        let name = text_right(kit, stage, AXIS_NAME, BODY_PT, ink, AXIS_NAME_AT)?;
        let header = text_left(kit, stage, HEADER, SMALL_PT, ink, HEADER_AT)?;
        let source = text_left(kit, stage, HEADER_SOURCE, FINE_PT, dim, HEADER_SOURCE_AT)?;
        let row_label = [
            text_left(kit, stage, ROW_LABEL[0], SMALL_PT, ink, (ROW_LABEL_X, ROW_Y[0]))?,
            text_left(
                kit,
                stage,
                &bm::grids_label(),
                FINE_PT,
                dim,
                (ROW_LABEL_X, ROW_Y[0] - LABEL_DROP),
            )?,
        ];
        let first = [vec![axis, ticks, name, header, source], tick_labels, row_label.to_vec()].concat();
        self.screen.show(stage, &first, FADE)?;
        kit.until_sentence(stage, "17_12", 1)?;
        // Near 0.297, then just under 0.3, then just over it.
        let (dots, labels, arrows) = self.row(kit, stage, 0, &GRIDS)?;
        let x3 = LINE.x(Tenk(3000));
        let guide = group(
            stage,
            dashes(
                v(x3, AXIS_Y + TICK + 0.05),
                v(x3, GUIDE_TOP),
                0.08,
                line_style(dim, GUIDE_W, 1.0),
            ),
            Z_LINE,
        );
        refuse_touching(stage, &labels, "row 1")?;
        refuse_crossing(stage, &labels, x3, "row 1 and the 0.3 guide")?;
        for (j, (&d, &l)) in dots.iter().zip(&labels).enumerate() {
            until_phrase(kit, stage, t0, "17_12", 1, ROW1_SAID[j])?;
            let mut now = vec![d, l];
            if j == 1 {
                now.push(guide);
            }
            self.screen.show(stage, &now, FADE)?;
        }
        kit.until_sentence(stage, "17_12", 2)?;
        // The changes shrink: the exact differences of the dots they join.
        let mut changes = Vec::new();
        for (w, c) in GRIDS.windows(2).zip(bm::changes(&GRIDS)) {
            let mid = 0.5 * (LINE.x(w[0].st) + LINE.x(w[1].st));
            changes.push(text(
                kit,
                stage,
                &c.signed(),
                SMALL_PT,
                ink,
                (mid, ROW_Y[0] + CHANGE_LIFT),
            )?);
        }
        refuse_touching(stage, &changes, "row 1 changes")?;
        self.screen.create(stage, &arrows, QUICK)?;
        self.screen.show(stage, &changes, FADE)?;
        kit.until_sentence(stage, "17_12", 3)?;
        // The finest grid at coarser time steps: each finer step came out higher.
        let (dots2, labels2, arrows2) = self.row(kit, stage, 1, &STEPS)?;
        refuse_touching(stage, &labels2, "row 2")?;
        refuse_crossing(stage, &labels2, x3, "row 2 and the 0.3 guide")?;
        let row2_label = [
            text_left(kit, stage, ROW_LABEL[1], SMALL_PT, ink, (ROW_LABEL_X, ROW_Y[1]))?,
            text_left(
                kit,
                stage,
                &bm::steps_label(),
                FINE_PT,
                dim,
                (ROW_LABEL_X, ROW_Y[1] - LABEL_DROP),
            )?,
        ];
        self.screen.show(
            stage,
            &[row2_label.to_vec(), dots2.clone(), labels2.clone()].concat(),
            FADE,
        )?;
        self.screen.create(stage, &arrows2, QUICK)?;
        self.line = [
            first,
            dots,
            labels,
            arrows,
            changes,
            vec![guide],
            row2_label.to_vec(),
            dots2,
            labels2,
            arrows2,
        ]
        .concat();
        Ok(())
    }

    // --------------------------------------------------------------- 17_13

    pub fn cue_17_13(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Prediction> {
        kit.say(stage, "17_13")?;
        let t0 = stage.scene().time().to_f64();
        let (ink, dim, hero, gold) = (Palette::ink(), Palette::dim(), Palette::hero(), Palette::gold());
        let settling = text(kit, stage, CONVERGENCE, SMALL_PT, Palette::green(), CONVERGENCE_AT)?;
        self.screen.show(stage, &[settling], FADE)?;
        until_phrase(kit, stage, t0, "17_13", 1, "came out at zero point two eight seven")?;
        // The measurement, 0.287 ± 0.003: a white band on the same line.
        let (lo, hi) = bm::band();
        let band = Rectangle::new()
            .width(LINE.x(hi) - LINE.x(lo))
            .height(2.0 * BAND_HALF)
            .style(
                Style::default()
                    .fill(band_white(), 0.3)
                    .stroke(band_white(), BAND_W, 1.0),
            )
            .build()?
            .moved_to(v(0.5 * (LINE.x(lo) + LINE.x(hi)), AXIS_Y));
        let band = add(stage, band, Z_LINE + 1);
        let xm = LINE.x(MEASURED);
        let band_text = [
            text(kit, stage, &bm::band_label(), SMALL_PT, ink, (xm, BAND_LABEL_Y[0]))?,
            text(kit, stage, BAND_SOURCE, FINE_PT, dim, (xm, BAND_LABEL_Y[1]))?,
        ];
        self.screen
            .show(stage, &[vec![band], band_text.to_vec()].concat(), FADE)?;
        // The gap, from the finest grid's 0.3003 down to 0.287, against the last grid change.
        let xf = LINE.x(GRIDS[2].st);
        let gap_line = group(
            stage,
            vec![
                seg(v(xm, GAP_Y), v(xf, GAP_Y), line_style(hero, MARK_W, 1.0)),
                seg(v(xm, GAP_Y), v(xm, AXIS_Y + BAND_HALF), line_style(hero, MARK_W, 1.0)),
                seg(v(xf, GAP_Y + 0.12), v(xf, GAP_Y - 0.12), line_style(hero, MARK_W, 1.0)),
            ],
            Z_MARK,
        );
        let gap_text = text(
            kit,
            stage,
            &bm::gap_label(),
            SMALL_PT,
            hero,
            (0.5 * (xm + xf), GAP_Y + 0.25),
        )?;
        self.screen.show(stage, &[gap_line, gap_text], FADE)?;
        // The 10-step dot inside the band: the short-inlet computation, matched by chance.
        if !bm::inside_band(STEPS[0].st) {
            return Err(scene_error(
                "the 10-step value is outside the band; drop the caution".to_owned(),
            ));
        }
        let x10 = LINE.x(STEPS[0].st);
        let circled = ring(stage, v(x10, ROW_Y[1]), CAUTION_R, gold, MARK_W);
        let leader = add(
            stage,
            seg(
                v(x10, CAUTION_AT[1].1 - 0.13),
                v(x10, ROW_Y[1] + CAUTION_R + 0.03),
                line_style(gold, 2.0, 1.0),
            ),
            Z_MARK,
        );
        let caution = [
            text_left(kit, stage, CAUTION[0], FINE_PT, gold, CAUTION_AT[0])?,
            text_left(kit, stage, CAUTION[1], FINE_PT, gold, CAUTION_AT[1])?,
        ];
        self.screen
            .show(stage, &[vec![circled, leader], caution.to_vec()].concat(), FADE)?;
        kit.until_sentence(stage, "17_13", 2)?;
        // The experiment's channel ran longer before the cylinder: a schematic, not to scale.
        let [top, bottom] = SCHEMA_WALLS;
        let [x_exp, x_in, x_out] = SCHEMA_X;
        let computed = group(
            stage,
            vec![
                seg(v(x_in, top), v(x_out, top), line_style(ink, SCHEMA_W, 1.0)),
                seg(v(x_in, bottom), v(x_out, bottom), line_style(ink, SCHEMA_W, 1.0)),
                seg(v(x_in, bottom), v(x_in, top), line_style(dim, 2.0, 1.0)),
                seg(v(x_out, bottom), v(x_out, top), line_style(dim, 2.0, 1.0)),
            ],
            Z_LINE,
        );
        let (sx, sy, sr) = SCHEMA_CYLINDER;
        let cylinder = Circle::new()
            .radius(sr)
            .arc_center(v(sx, sy))
            .style(Style::default().fill(solid_grey(), 1.0).stroke(ink, 2.0, 1.0))
            .build();
        let cylinder = add(stage, cylinder, Z_LINE);
        let dashed_style = line_style(dim, SCHEMA_W, 1.0);
        let extension = [
            dashes(v(x_exp, top), v(x_in, top), 0.12, dashed_style),
            dashes(v(x_exp, bottom), v(x_in, bottom), 0.12, dashed_style),
            dashes(v(x_exp, bottom), v(x_exp, top), 0.1, line_style(dim, 2.0, 1.0)),
        ]
        .concat();
        let extension = group(stage, extension, Z_LINE);
        let labels = [
            text_left(kit, stage, EXPERIMENT, SMALL_PT, ink, EXPERIMENT_AT)?,
            text(kit, stage, COMPUTED, FINE_PT, dim, COMPUTED_AT)?,
            text_left(kit, stage, NOT_TO_SCALE, FINE_PT, dim, NOT_TO_SCALE_AT)?,
        ];
        let teal = Palette::teal();
        let longer = group(
            stage,
            vec![
                seg(v(x_exp, top), v(x_in, top), line_style(teal, SCHEMA_W, 1.0)),
                seg(v(x_exp, bottom), v(x_in, bottom), line_style(teal, SCHEMA_W, 1.0)),
            ],
            Z_LINE + 1,
        );
        self.schematic = [vec![computed, cylinder, extension], labels.to_vec()].concat();
        self.screen.show(stage, &self.schematic.clone(), FADE)?;
        self.longer = Some(longer);
        kit.until_sentence(stage, "17_13", 3)?;
        let card = Prediction::build(kit, stage)?;
        let shown: Vec<Mob> = card
            .parts
            .iter()
            .chain(&card.markers)
            .copied()
            .chain([card.note, card.track])
            .collect();
        self.screen.show(stage, &shown, FADE)?;
        self.line.extend([settling, band, gap_line, gap_text, circled, leader]);
        self.line.extend(band_text);
        self.line.extend(caution);
        Ok(card)
    }

    // --------------------------------------------------------------- 17_h2

    /// The question, the evidence and the schematic stay; the countdown runs 5 s on the ring
    /// and digits; nothing else moves, and the 0.289 dot does not exist yet.
    pub fn hold_17_h2(&mut self, kit: &Kit, stage: &mut Stage<'_>, card: &Prediction) -> fmn::Result<()> {
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

    // --------------------------------------------------------------- 17_14

    pub fn cue_17_14(&mut self, kit: &Kit, stage: &mut Stage<'_>, card: Prediction) -> fmn::Result<()> {
        kit.say(stage, "17_14")?;
        let t0 = stage.scene().time().to_f64();
        let teal = Palette::teal();
        let longer = self
            .longer
            .take()
            .ok_or_else(|| scene_error("17_14 before 17_13".to_owned()))?;
        self.screen.show(stage, &[card.chosen], QUICK)?;
        self.screen.show(stage, &[longer], FADE)?;
        self.schematic.push(longer);
        until_phrase(kit, stage, t0, "17_14", 1, "gave zero point two eight nine")?;
        // 0.289 lands in the band; the standard channel's 0.3003 stays outside it.
        if !bm::inside_band(LONGER_INLET) || bm::inside_band(GRIDS[2].st) {
            return Err(scene_error(
                "0.289 must lie inside the band and 0.3003 outside".to_owned(),
            ));
        }
        let x = LINE.x(LONGER_INLET);
        let value = LONGER_INLET
            .three()
            .ok_or_else(|| scene_error("0.289 has three places".to_owned()))?;
        let landed = vec![
            dot(stage, v(x, AXIS_Y), 1.3 * DOT_R, teal),
            text(kit, stage, &value, SMALL_PT, teal, (x, AXIS_Y + LONG_LABEL_DY))?,
            text_left(kit, stage, LONG_NOTE[0], FINE_PT, teal, LONG_NOTE_AT[0])?,
            text_left(kit, stage, LONG_NOTE[1], FINE_PT, Palette::dim(), LONG_NOTE_AT[1])?,
            text_left(kit, stage, STANDARD, FINE_PT, Palette::dim(), STANDARD_AT)?,
        ];
        // The notes sit where the card's label and question were: card out, then the result in.
        // 15 + 15 frames, as long as one 0.5 s swap, so no later frame moves.
        self.screen.hide(stage, &card.all(), QUICK)?;
        self.screen.show(stage, &landed, QUICK)?;
        kit.until_sentence(stage, "17_14", 2)?;
        // Why: the authors' words.
        let (cx, _) = QUOTE_CARD_AT;
        let mut quote = vec![card_at(stage)?];
        for (k, line) in QUOTE17.iter().enumerate() {
            quote.push(text(kit, stage, line, FINE_PT, Palette::ink(), (cx, QUOTE_Y[k]))?);
        }
        quote.push(text(
            kit,
            stage,
            QUOTE17_SOURCE,
            FINE_PT,
            Palette::dim(),
            (cx, QUOTE_Y[3]),
        )?);
        self.screen.show(stage, &quote, FADE)?;
        self.line.extend(landed);
        self.line.extend(quote);
        Ok(())
    }

    // --------------------------------------------------------------- 17_15

    pub fn cue_17_15(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "17_15")?;
        self.clear(stage)?;
        self.line.clear();
        self.schematic.clear();
        let ledger = self
            .ledger
            .take()
            .ok_or_else(|| scene_error("17_15 before 17_10".to_owned()))?;
        let back = [ledger.base(), ledger.outlet.clone()].concat();
        self.screen.slide_in(stage, &back, v(-LEDGER_SLIDE, 0.0), MOVE)?;
        let ink = Palette::ink();
        for (k, check) in CHECKS.iter().enumerate() {
            kit.until_sentence(stage, "17_15", k + 1)?;
            let (cx, _) = CHECK_AT[k];
            let card = vec![
                card(stage, CHECK_AT[k], CHECK_SIZE)?,
                text(kit, stage, check[0], BODY_PT, check_color(k), (cx, CHECK_Y[0]))?,
                text(kit, stage, check[1], SMALL_PT, ink, (cx, CHECK_Y[1]))?,
                text(kit, stage, check[2], SMALL_PT, ink, (cx, CHECK_Y[2]))?,
            ];
            self.screen.show(stage, &card, FADE)?;
            // The ledger gains the check's column; every entry for our runs is "?".
            let mut into = ledger.columns[k].clone();
            let mut out = Vec::new();
            if k == 0 {
                out.extend([ledger.narrow, ledger.rule]);
                into.extend([ledger.wide, ledger.wide_rule, ledger.ours]);
            }
            self.screen.swap(stage, &out, &into, FADE)?;
        }
        self.ledger = Some(ledger);
        kit.hold(stage)
    }
}

fn card_at(stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    card(stage, QUOTE_CARD_AT, QUOTE_CARD_SIZE)
}

// --------------------------------------------------------------- the cues

/// Cues 17_8 to 17_15 and the 17_h2 hold, in order, from 17_7's end state. Returns the
/// tail, with the three check cards and the ledger on screen for root's chapter 18.
pub fn chapter17_tail(kit: &Kit, stage: &mut Stage<'_>, ch: Chapter17) -> fmn::Result<Tail> {
    let mut t = Tail::adopt(ch)?;
    t.cue_17_8(kit, stage)?;
    t.cue_17_9(kit, stage)?;
    t.cue_17_10(kit, stage)?;
    t.cue_17_11(kit, stage)?;
    t.cue_17_12(kit, stage)?;
    let card = t.cue_17_13(kit, stage)?;
    t.hold_17_h2(kit, stage, &card)?;
    t.cue_17_14(kit, stage, card)?;
    t.cue_17_15(kit, stage)?;
    Ok(t)
}

/// The complete boundary and benchmark chapter.
pub struct TailScene {
    pub kit: Kit,
}

impl SceneConstruct for TailScene {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let ch = chapter17(&self.kit, stage, true)?;
        let mut tail = chapter17_tail(&self.kit, stage, ch)?;
        let all = tail.screen.on().to_vec();
        tail.screen.hide(stage, &all, FADE)?;
        self.kit.finish(stage)
    }
}

/// The spoken lines this part says, verbatim from `navier-full-script03.md` chapter 17
/// (`tools/check_tail01.py` compares them byte for byte), as `script.tsv` rows. 17_h2 is a
/// hold, not a line.
pub const SCRIPT17_TAIL: &str = concat!(
    "17_8\tAt the walls and on the cylinder, the fluid doesn't slip, as in chapter six. At the inlet it arrives in a curved profile, still at the walls and fastest in the middle, at one and a half meters per second. On average, that's one meter per second.\n",
    "17_9\tThe fluid isn't air, and with its viscosity, one number, average speed times diameter divided by viscosity, comes out at exactly one hundred. That's the Reynolds number, and the larger it is, the weaker viscosity's smoothing is, compared with the flow carrying itself along. In our equations, the same shape, boundaries and start, at the same Reynolds number, give the same pattern, just scaled in size and speed. And slow air follows those equations closely.\n",
    "17_10\tAt the outlet, the test lets each group choose its own rule. As we said in chapter six, that rule is a choice, not nature's, so it goes in the ledger with everything else.\n",
    "17_11\tEvery group reported the same few numbers. Two are peak forces, along the flow and across it, called drag and lift coefficients. One is how often the wake swings, the Strouhal number. And one is a pressure difference across the cylinder.\n",
    "17_12\tHere's one group's Strouhal number from the nineteen ninety-six report, on three grids, each with four times the unknowns of the last. Near point two nine seven, then just under point three, then just over it. The changes shrink, from about two and a half thousandths to under one thousandth. And the finer the time steps, the higher it came out.\n",
    "17_13\tThat's convergence, the answer settling down as we refine. Measurements were made for this test too, and the Strouhal number came out at zero point two eight seven, give or take three thousandths. But the experiment's channel ran longer before the cylinder than the computed one did. What would close the gap?\n",
    "17_14\tGive the flow a longer run before the cylinder. With the inlet moved farther upstream, a computation in the same report gave zero point two eight nine, inside the measured range. The authors explain that, with the inlet so close, the flow arriving at the cylinder is slightly different.\n",
    "17_15\tSo there are three different checks. Convergence asks whether the answer stops changing as we refine, and it tests how well we solved our equations. Benchmarking, agreeing with careful calculations of the very same problem, catches mistakes in our setup and our program, but it can't test the equations themselves. Validation asks whether it matches a measurement of the real setup, and only that tests whether our equations, shape and boundaries describe the real flow.\n",
);

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::boundary17_01::{SCRIPT17, TITLE_AT};
    use explainer_kit::{Narrator, Script};

    #[test]
    fn script_has_the_eight_lines_in_order() {
        let script = Script::parse(&format!("{SCRIPT17}{SCRIPT17_TAIL}")).unwrap();
        let ids: Vec<&str> = script.lines().iter().map(|(id, _)| id.as_str()).skip(7).collect();
        assert_eq!(
            ids,
            ["17_8", "17_9", "17_10", "17_11", "17_12", "17_13", "17_14", "17_15"]
        );
        let kit = Kit::new().unwrap().with_narrator(Narrator::estimate(&script));
        let counts: Vec<usize> = ids.iter().map(|id| kit.sentence_starts(id).len()).collect();
        assert_eq!(counts, [3, 4, 2, 4, 4, 4, 3, 4]);
    }

    #[test]
    fn phrases_sit_in_their_sentences() {
        let script = Script::parse(&format!("{SCRIPT17}{SCRIPT17_TAIL}")).unwrap();
        let kit = Kit::new().unwrap().with_narrator(Narrator::estimate(&script));
        for row in SCRIPT17_TAIL.lines() {
            let (id, line) = row.split_once('\t').unwrap();
            assert_eq!(sentences(line).len(), kit.sentence_starts(id).len(), "{id}");
        }
        let src = include_str!("tail17_01.rs");
        let mut gates = 0;
        for call in src.split("until_phrase(kit, stage, t0, \"").skip(1) {
            let (id, rest) = call.split_once('"').unwrap();
            let (j, arg) = rest[2..].split_once(", ").unwrap();
            let j: usize = j.parse().unwrap();
            let phrases: Vec<&str> = match arg.strip_prefix('"') {
                Some(p) => vec![p.split('"').next().unwrap()],
                None => ROW1_SAID.to_vec(),
            };
            for p in phrases {
                assert!(phrase_share(id, j, p).is_some(), "{id} {j} {p}");
                gates += 1;
            }
        }
        assert_eq!(gates, 7);
        assert!(phrase_share("17_8", 1, "at one and a half meters per second").unwrap() > 0.5);
        assert_eq!(phrase_share("17_12", 1, ROW1_SAID[0]), Some(0.0));
    }

    #[test]
    fn card_text_comes_from_the_data() {
        assert!(QUESTION2[0].contains(&GRIDS[2].st.four()));
        assert!(QUESTION2[0].contains(&bm::band_label()["measured: ".len()..]));
        assert!(LONG_NOTE[0].starts_with(&LONGER_INLET.three().unwrap()));
        assert_eq!(CHOICES2[ANSWER2], "a longer run before the cylinder");
        assert!(FORCE_NOTES[2].is_empty(), "card 3's second line is ST_TEX");
    }

    #[test]
    fn layout_stays_in_frame() {
        let top = ch_pt(0.0, channel::HEIGHT)[1];
        assert!(PEAK_AT.1 + 0.3 < TITLE_AT.1 - 0.2 && AVERAGE_AT.1 - 0.2 > top);
        assert!(ch_pt(0.0, 0.0)[1] - 0.5 > RE_CARD_AT.1 + RE_CARD_SIZE.1 / 2.0);
        assert!(Ledger::right(5) <= 7.6 && LEDGER_BOTTOM >= -4.4 && LEDGER_LEFT >= -7.6);
        assert!(Ledger::right(2) < FORCE_CARD_AT[0].0 - FORCE_CARD_SIZE.0 / 2.0);
        for (at, size) in [
            (Q_CARD_AT, Q_CARD_SIZE),
            (QUOTE_CARD_AT, QUOTE_CARD_SIZE),
            (RE_CARD_AT, RE_CARD_SIZE),
            (SIMILAR_AT, SIMILAR_SIZE),
        ] {
            assert!(at.0 - size.0 / 2.0 >= -7.6 && at.0 + size.0 / 2.0 <= 7.6);
            assert!(at.1 - size.1 / 2.0 >= -4.4 && at.1 + size.1 / 2.0 <= 4.0);
        }
        assert!(Q_CARD_AT.1 + Q_CARD_SIZE.1 / 2.0 < TICK_LABEL_Y - 0.2);
        let inlet_tip = ch_pt(0.0, channel::CENTRE.1)[0] + INLET_SCALE * 1.5;
        let cylinder_front = ch_pt(channel::CENTRE.0 - channel::DIAMETER / 2.0, 0.0)[0];
        assert!(
            cylinder_front - inlet_tip > 0.2,
            "the peak arrow stops short of the cylinder"
        );
    }

    #[test]
    fn ledger_columns_fit() {
        assert_eq!(Ledger::col_x(0), LEDGER_LEFT + LEDGER_LABEL_W + LEDGER_COL_W / 2.0);
        assert!(Ledger::row_y(LEDGER_ROWS.len()) - 0.3 > LEDGER_BOTTOM);
        assert_eq!(LEDGER_CHECKS.len() + 2, 5);
    }
}
