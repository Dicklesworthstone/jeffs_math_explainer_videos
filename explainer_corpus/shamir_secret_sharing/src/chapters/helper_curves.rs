//! Chapter 14, helper curves: how three numbers can give the secret, on
//! paper first. The opening trio, trustees 1, 2 and 3 with their cards 8, 3
//! and 1, opened the door in chapter 04, and how was left for later; they
//! know three points of the dealer's curve on the dial, but not its spins
//! (14_1). On paper, where nothing wraps, their heights would be 19, 47 and
//! 89, three points pin one parabola, and its height at position 0 is the
//! secret (14_2). Each trustee gets a helper: 1 at its own position and 0
//! at the other two, drawn first only as those three dots on three small
//! boards that share the paper plot's positions (14_3). Scaled by the
//! heights and added, the helpers make 19 at position 1, because there the
//! other two are 0, and 47 and 89 at positions 2 and 3: a column outline
//! moves across the boards as each sum is written (14_4). The viewer is asked
//! what recipe is 0 at positions 2 and 3 (14_5): (position − 2) × (position
//! − 3), which is 2 at position 1, so halved (14_6); the other two helpers
//! the same way, divided by −1 and by 2 (14_7). The weighted sum, drawn on
//! the plot, lies on the dealer's parabola all the way, because only one
//! parabola passes through three points (14_8).
//!
//! The helpers are `model::helper_at` over `model::helper_at_own`; every
//! value the boards and the working show is checked against it, and the
//! scene stops unless each helper is 1 at its own trustee and 0 at the
//! others, the weighted sum gives the three heights, and the drawn sum is
//! the dealer's parabola at every sample. [`Setting`] builds the pieces that
//! stay, so chapter 15 opens on exactly what this chapter ends on.
//! Emphasis never restyles or scales typeset text: rings, outlines and
//! strokes of their own point at it, and only shapes (dots, curves) change
//! color or swell.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, Timed, fade_all, tween, v};
use fmn::prelude::*;

use super::a_ten_mark_dial::text_after;
use super::board::{Board, Grid, ring};
use super::draw::{
    DIAL_R, DIAL_Y, DOOR_X, Rail, Spoken, Vault, card, disc, fade_in_all, fail, pause, read_by, recolor, recolor_all,
    recolorings, say, stroke, swaps, think, wait_until,
};
use super::envelope::{PAD, dealt, door_glow, frame_halos, outline};
use super::two_keys::{glow, glowing, leaving, leaving_all_but, moving, swell_all, text_halo};
use super::two_shares::{disc_ring, text_right};
use crate::model::{self, Dial, MARKS, OPENING_TRIO, Ratio, Recipe, SECRET, TRUSTEES};
use crate::palette;

pub struct HelperCurves {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model)
const BOARD_TITLE: &str = "on the dial: points only, no curve";
const SPINS_UNKNOWN: &str = "spins: ? and ?";
pub const PAPER_TITLE: &str = "on paper: ordinary numbers";
const HEIGHT_TITLE: &str = "height";
const POSITION_TITLE: &str = "position";
pub const UNKNOWN: &str = "?";
const HALVING: &str = "on paper, halving is fine";

// ------------------------------------------------------------------ layout

/// The trio's dial board, right (14_1): chapter 09's, and the spins' note
/// under its position numbers.
const DIAL_BOARD: Grid = Grid {
    x0: 1.5,
    y0: -0.15,
    dx: 0.85,
    dy: 0.27,
    marks: MARKS,
    last: 5,
};
const SPINS_X: f64 = 3.65;
const SPINS_Y: f64 = -0.95;
const SPINS_SIZE: f64 = 22.0;
pub const POINT_RING: f64 = 0.17;
/// The paper plot, left: position 0's place and one step, height 0's place
/// and one unit, the tallest labelled height and the step between labels,
/// the last position, and how far the drawn curves run past it.
pub const PAPER_X0: f64 = -5.6;
pub const PAPER_DX: f64 = 0.95;
pub const PAPER_Y0: f64 = -0.55;
pub const PAPER_DY: f64 = 0.03;
pub const PAPER_TOP: i64 = 100;
const PAPER_STEP: usize = 20;
pub const LAST: u32 = 3;
pub const CURVE_END: f64 = 3.2;
const PAPER_RUN_ON: f64 = 0.4;
const PAPER_LABEL_SIZE: f64 = 18.0;
const PAPER_LABEL_DX: f64 = 0.36;
const PAPER_POSITION_DY: f64 = 0.25;
pub const PAPER_TITLE_Y: f64 = 3.45;
const PAPER_TITLE_SIZE: f64 = 22.0;
const AXIS_TITLE_GAP: f64 = 0.25;
const AXIS_W: f64 = 2.5;
const TICK: f64 = 0.08;
pub const POINT_R: f64 = 0.08;
const POINT_LABEL_SIZE: f64 = 22.0;
const POINT_LABEL_DX: f64 = -0.3;
const POINT_LABEL_DY: f64 = 0.26;
/// The "?" (later the secret) by position 0, up and right of where the
/// curve meets the side, clear of the height labels.
pub const UNKNOWN_DX: f64 = 0.25;
pub const UNKNOWN_DY: f64 = 0.35;
pub const UNKNOWN_SIZE: f64 = 32.0;
/// The dealer's parabola is drawn wider than the sum, so where the two lie
/// on each other the parabola shows round it.
const PARABOLA_W: f64 = 7.0;
const SUM_W: f64 = 3.5;
pub const CURVE_SAMPLES: usize = 96;
/// The helper boards, right, one over another: position 0's place across
/// and one step (each board's columns under the same numbers), each board's
/// 0 line, one unit of height, and how far its side runs each way.
pub const HELPER_X0: f64 = 1.3;
pub const HELPER_DX: f64 = 0.8;
pub const HELPER_Y: [f64; 3] = [2.85, 1.35, -0.15];
pub const HELPER_DY: f64 = 0.17;
pub const HELPER_SPAN: f64 = 3.0;
const HELPER_RUN_ON: f64 = 0.35;
const HELPER_W: f64 = 3.0;
/// Each board's name, right-aligned left of its side; the positions over
/// the top board, "position" left of them. The side has a tick at 1 and no
/// label: the dots' own labels give the scale, and chapter 15 writes the
/// weights there.
pub const NAME_GAP: f64 = 0.35;
pub const NAME_DY: f64 = 0.22;
pub const NAME_SIZE: f64 = 18.0;
pub const HEAD_Y: f64 = 3.62;
pub const HEAD_SIZE: f64 = 16.0;
/// A helper's dots and their labels, up and to the left.
pub const DOT_R: f64 = 0.07;
pub const DOT_LABEL_SIZE: f64 = 16.0;
pub const DOT_LABEL_DX: f64 = -0.2;
pub const DOT_LABEL_DY: f64 = 0.17;
pub const DOT_RING: f64 = 0.15;
/// The position-zero outline leaves chapter 15's exterior weights clear.
pub const COLUMN_W: f64 = 0.4;
/// At trustee positions, enclose each dot and its left-offset zero/one label.
const CELL_COLUMN_W: f64 = 0.7;
/// The scale tags right of the boards.
pub const TAG_X: f64 = 4.35;
pub const TAG_SIZE: f64 = 22.0;
/// The working under the boards, its lines left-aligned, clear of the
/// rail's last frame, all over the lower third's band.
pub const WORK_X: f64 = -1.2;
pub const WORK_Y: [f64; 4] = [-1.45, -1.9, -2.35, -2.8];
pub const WORK_SIZE: f64 = 20.0;
const NOTE_SIZE: f64 = 16.0;
/// The think ring, lower right (14_5).
const THINK_X: f64 = 5.6;
const THINK_Y: f64 = -2.2;
/// The hold after the card is read.
const HOLD_AT_END: f64 = 0.6;

// -------------------------------------------------------------------- data

/// Trustee `i`'s helper at `x` before it is scaled: (x − each other
/// trustee's position), multiplied together, in ordinary numbers.
pub fn product_at(xs: &[u32], i: usize, x: f64) -> f64 {
    xs.iter()
        .enumerate()
        .filter(|&(j, _)| j != i)
        .map(|(_, &xj)| x - f64::from(xj))
        .product()
}

/// Trustee `i`'s helper at `x`: the product over its value at the
/// trustee's own position, so it is 1 there.
pub fn helper_value(xs: &[u32], i: usize, x: f64) -> f64 {
    product_at(xs, i, x) / model::helper_at_own(xs, i) as f64
}

/// The weighted sum at `x`: each trustee's height times its helper.
pub fn weighted_sum_at(trio: &[(u32, i64)], x: f64) -> f64 {
    let xs: Vec<u32> = trio.iter().map(|p| p.0).collect();
    trio.iter()
        .enumerate()
        .map(|(i, &(_, h))| h as f64 * helper_value(&xs, i, x))
        .sum()
}

/// Trustee `i`'s helper at whole position `at`, exactly.
pub fn helper_exact(xs: &[u32], i: usize, at: i64) -> Ratio {
    Ratio::new(model::helper_at(xs, i, at), model::helper_at_own(xs, i))
}

/// What each helper is at each trustee's position, row by row; an error
/// unless each is 1 at its own trustee and 0 at the others.
pub fn at_trio(xs: &[u32]) -> Result<Vec<Vec<i64>>, String> {
    let rows: Vec<Vec<i64>> = (0..xs.len())
        .map(|i| {
            xs.iter()
                .map(|&x| helper_exact(xs, i, i64::from(x)).whole().unwrap_or(i64::MIN))
                .collect()
        })
        .collect();
    for (i, row) in rows.iter().enumerate() {
        for (j, &value) in row.iter().enumerate() {
            if value != i64::from(i == j) {
                return Err(format!("helper {i} is {value} at trustee {}", xs[j]));
            }
        }
    }
    Ok(rows)
}

/// The weighted sum at whole position `at`, exactly.
pub fn sum_exact(trio: &[(u32, i64)], at: i64) -> Ratio {
    let xs: Vec<u32> = trio.iter().map(|p| p.0).collect();
    trio.iter()
        .enumerate()
        .map(|(i, &(_, h))| helper_exact(&xs, i, at).times(h))
        .fold(Ratio::int(0), Ratio::plus)
}

/// The positions where trustee `i`'s helper must be 0: the other trustees'.
pub fn zeros(xs: &[u32], i: usize) -> Vec<u32> {
    xs.iter()
        .enumerate()
        .filter(|&(j, _)| j != i)
        .map(|(_, &x)| x)
        .collect()
}

/// Each factor of trustee `i`'s helper at `at`: `at` less each other
/// trustee's position.
pub fn factors_at(xs: &[u32], i: usize, at: i64) -> Vec<i64> {
    zeros(xs, i).into_iter().map(|x| at - i64::from(x)).collect()
}

/// `f` from `from` to `to`, sampled evenly.
pub fn sampled(f: impl Fn(f64) -> f64, from: f64, to: f64) -> Vec<(f64, f64)> {
    (0..=CURVE_SAMPLES)
        .map(|k| {
            let x = from + (to - from) * k as f64 / CURVE_SAMPLES as f64;
            (x, f(x))
        })
        .collect()
}

/// The runs of `points` with heights from `lo` to `hi`, each cut where it
/// crosses out, the crossing found on the straight piece between samples.
pub fn clipped(points: &[(f64, f64)], lo: f64, hi: f64) -> Vec<Vec<(f64, f64)>> {
    let inside = |h: f64| (lo..=hi).contains(&h);
    let cross = |a: (f64, f64), b: (f64, f64)| {
        let edge = if a.1.max(b.1) > hi { hi } else { lo };
        let t = (edge - a.1) / (b.1 - a.1);
        (a.0 + (b.0 - a.0) * t, edge)
    };
    let mut runs: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut run: Vec<(f64, f64)> = Vec::new();
    for (k, &p) in points.iter().enumerate() {
        match (k.checked_sub(1).map(|j| points[j]), inside(p.1)) {
            (Some(q), true) if !inside(q.1) => {
                run.push(cross(q, p));
                run.push(p);
            }
            (Some(q), false) if inside(q.1) => {
                run.push(cross(q, p));
                runs.push(std::mem::take(&mut run));
            }
            (_, true) => run.push(p),
            _ => {}
        }
    }
    if run.len() > 1 {
        runs.push(run);
    }
    runs.retain(|r| r.len() > 1);
    runs
}

/// A signed whole number as written, the minus sign typeset: "−3".
pub fn signed(n: i64) -> String {
    if n < 0 {
        format!("\u{2212}{}", -n)
    } else {
        n.to_string()
    }
}

/// The same, bracketed when negative, to stand after "×": "(−1)".
pub fn bracketed(n: i64) -> String {
    if n < 0 {
        format!("({})", signed(n))
    } else {
        n.to_string()
    }
}

/// "trustee 1's helper".
pub fn helper_name(t: u32) -> String {
    format!("trustee {t}'s helper")
}

/// One factor at `at`, written out: "(position − 2)", or "(1 − 2)".
fn factor(at: &str, x: u32) -> String {
    format!("({at} \u{2212} {x})")
}

/// Trustee `i`'s product of factors: "(position − 2) × (position − 3)".
pub fn product_text(xs: &[u32], i: usize, at: &str) -> String {
    let parts: Vec<String> = zeros(xs, i).into_iter().map(|x| factor(at, x)).collect();
    parts.join(" \u{d7} ")
}

/// "trustee 1's helper: (position − 2) × (position − 3)".
pub fn recipe_line(xs: &[u32], i: usize) -> String {
    format!("{}: {}", helper_name(xs[i]), product_text(xs, i, "position"))
}

/// What dividing by `own` is written as: "÷ 2", "÷ (−1)".
pub fn divide_text(own: i64) -> String {
    format!("\u{f7} {}", bracketed(own))
}

/// 14_6's working at the trustee's own position, with the factors' values,
/// and the product after it: "at 1: (1 − 2) × (1 − 3) = (−1) × (−2)", "= 2".
pub fn own_working(xs: &[u32], i: usize) -> [String; 2] {
    let x = xs[i];
    let values: Vec<String> = factors_at(xs, i, i64::from(x)).into_iter().map(bracketed).collect();
    [
        format!(
            "at {x}: {} = {}",
            product_text(xs, i, &x.to_string()),
            values.join(" \u{d7} ")
        ),
        format!("= {}", signed(model::helper_at_own(xs, i))),
    ]
}

/// 14_7's shorter working: "at 2: (2 − 1) × (2 − 3) = −1".
pub fn own_line(xs: &[u32], i: usize) -> String {
    let x = xs[i];
    format!(
        "at {x}: {} = {}",
        product_text(xs, i, &x.to_string()),
        signed(model::helper_at_own(xs, i))
    )
}

/// 14_4's sum at trustee `j`'s position: each height times each helper
/// there, and the total: "at 1: 19 × 1 + 47 × 0 + 89 × 0 = 19".
pub fn sum_line(trio: &[(u32, i64)], j: usize) -> Result<String, String> {
    let xs: Vec<u32> = trio.iter().map(|p| p.0).collect();
    let rows = at_trio(&xs)?;
    let terms: Vec<String> = trio
        .iter()
        .zip(&rows)
        .map(|(&(_, h), row)| format!("{h} \u{d7} {}", row[j]))
        .collect();
    let total = sum_exact(trio, i64::from(xs[j]))
        .whole()
        .ok_or("a sum that is not whole")?;
    Ok(format!("at {}: {} = {total}", xs[j], terms.join(" + ")))
}

/// A helper's scale: "× 19".
pub fn scale_tag(h: i64) -> String {
    format!("\u{d7} {h}")
}

/// 14_8's precisely card: one parabola through three points, so the
/// weighted sum is the dealer's curve.
pub fn card_text(trio: &[(u32, i64)]) -> String {
    let terms: Vec<String> = trio
        .iter()
        .enumerate()
        .map(|(i, &(_, h))| format!("{h} \u{d7} helper {}", i + 1))
        .collect();
    format!(
        "three points pin one parabola (chapter 09), so {} is the dealer's curve; at 0 it is the secret",
        terms.join(" + ")
    )
}

// ---------------------------------------------------------------- drawing

/// The scene point at position `x`, height `h`, on the paper plot.
pub fn paper_at(x: f64, h: f64) -> Vec3 {
    v(PAPER_X0 + PAPER_DX * x, PAPER_Y0 + PAPER_DY * h)
}

/// The scene point at position `x`, height `h`, on helper board `b`.
pub fn on_board(b: usize, x: f64, h: f64) -> Vec3 {
    v(HELPER_X0 + HELPER_DX * x, HELPER_Y[b] + HELPER_DY * h)
}

/// Where a dot's label sits on board `b`.
pub fn dot_label_at(b: usize, x: f64, h: f64) -> Vec3 {
    let p = on_board(b, x, h);
    v(p[0] + DOT_LABEL_DX, p[1] + DOT_LABEL_DY)
}

/// The centre of the column outline at position `x`, down all three
/// boards, and its height.
pub fn column_at(x: u32) -> (Vec3, f64) {
    let top = HELPER_Y[0] + HELPER_DY * HELPER_SPAN;
    let bottom = HELPER_Y[HELPER_Y.len() - 1] - HELPER_DY * HELPER_SPAN;
    (
        v(HELPER_X0 + HELPER_DX * f64::from(x), (top + bottom) / 2.0),
        top - bottom + 2.0 * PAD,
    )
}

/// An outline down position `x`'s column, in `color`. Detached.
pub fn column_outline(stage: &mut Stage<'_>, x: u32, color: Srgb) -> fmn::Result<Mob> {
    let (at, h) = column_at(x);
    outline(stage, at, COLUMN_W, h, color)
}

/// A stroke through `points` on the paper plot. Detached.
fn on_paper(stage: &mut Stage<'_>, points: &[(f64, f64)], color: Srgb, w: f64) -> Mob {
    stroke(stage, points.iter().map(|&(x, h)| paper_at(x, h)).collect(), color, w)
}

/// A stroke through `points` on board `b`. Detached.
pub fn on_board_stroke(stage: &mut Stage<'_>, b: usize, points: &[(f64, f64)], color: Srgb) -> Mob {
    stroke(
        stage,
        points.iter().map(|&(x, h)| on_board(b, x, h)).collect(),
        color,
        HELPER_W,
    )
}

/// The paper plot's axes: a tick and a label at every 20th height and
/// under every position, the axes' names and the title. Detached.
fn paper_axes(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let origin = paper_at(0.0, 0.0);
    let right = paper_at(f64::from(LAST), 0.0)[0] + PAPER_RUN_ON;
    let top = paper_at(0.0, PAPER_TOP as f64)[1];
    let mut parts = vec![
        stroke(stage, vec![origin, v(right, PAPER_Y0)], Palette::dim(), AXIS_W),
        stroke(stage, vec![origin, v(PAPER_X0, top)], Palette::dim(), AXIS_W),
    ];
    for h in (0..=PAPER_TOP).step_by(PAPER_STEP) {
        let p = paper_at(0.0, h as f64);
        if h > 0 {
            parts.push(stroke(stage, vec![p, v(p[0] - TICK, p[1])], Palette::dim(), AXIS_W));
        }
        parts.push(kit.text(
            stage,
            &h.to_string(),
            PAPER_LABEL_SIZE,
            Palette::dim(),
            v(PAPER_X0 - PAPER_LABEL_DX, p[1]),
        )?);
    }
    for x in 0..=LAST {
        let p = paper_at(f64::from(x), 0.0);
        if x > 0 {
            parts.push(stroke(stage, vec![p, v(p[0], p[1] - TICK)], Palette::dim(), AXIS_W));
        }
        parts.push(kit.text(
            stage,
            &x.to_string(),
            PAPER_LABEL_SIZE,
            Palette::dim(),
            v(p[0], PAPER_Y0 - PAPER_POSITION_DY),
        )?);
    }
    parts.push(kit.text(
        stage,
        HEIGHT_TITLE,
        PAPER_LABEL_SIZE,
        Palette::dim(),
        v(PAPER_X0, top + AXIS_TITLE_GAP),
    )?);
    parts.push(kit.text_left(
        stage,
        POSITION_TITLE,
        PAPER_LABEL_SIZE,
        Palette::dim(),
        v(right + AXIS_TITLE_GAP / 2.0, PAPER_Y0),
    )?);
    parts.push(kit.text_left(
        stage,
        PAPER_TITLE,
        PAPER_TITLE_SIZE,
        Palette::dim(),
        v(PAPER_X0 - PAPER_LABEL_DX, PAPER_TITLE_Y),
    )?);
    Ok(parts)
}

/// Board `b`'s axes, its side's tick at 1 and its name. Detached.
fn board_axes(kit: &Kit, stage: &mut Stage<'_>, b: usize, t: u32) -> fmn::Result<Vec<Mob>> {
    let right = on_board(b, f64::from(LAST), 0.0)[0] + HELPER_RUN_ON;
    let y = HELPER_Y[b];
    let mut parts = vec![
        stroke(stage, vec![on_board(b, 0.0, 0.0), v(right, y)], Palette::dim(), AXIS_W),
        stroke(
            stage,
            vec![on_board(b, 0.0, -HELPER_SPAN), on_board(b, 0.0, HELPER_SPAN)],
            Palette::dim(),
            AXIS_W,
        ),
    ];
    for x in 1..=LAST {
        let p = on_board(b, f64::from(x), 0.0);
        parts.push(stroke(stage, vec![p, v(p[0], p[1] - TICK)], Palette::dim(), AXIS_W));
    }
    let one = on_board(b, 0.0, 1.0);
    parts.push(stroke(
        stage,
        vec![one, v(one[0] - TICK, one[1])],
        Palette::dim(),
        AXIS_W,
    ));
    parts.push(text_right(
        kit,
        stage,
        &helper_name(t),
        NAME_SIZE,
        Palette::dim(),
        v(HELPER_X0 - NAME_GAP, y - NAME_DY),
    )?);
    Ok(parts)
}

/// The pieces that stay from 14_2 to 15_3, built once, detached, so
/// chapter 15 opens on exactly what chapter 14 ends on.
pub struct Setting {
    /// The paper plot's axes, names and title.
    pub plot: Vec<Mob>,
    /// The trio's points on paper, each a dot and its height's label.
    pub points: Vec<[Mob; 2]>,
    /// The "?" by position 0.
    pub unknown: Mob,
    /// The dealer's parabola, and the weighted sum drawn over it.
    pub parabola: Mob,
    pub sum: Mob,
    /// Each helper board's axes and name, and the positions over them.
    pub boards: Vec<Vec<Mob>>,
    pub heads: Vec<Mob>,
    /// Each board's dot at each trustee's position, with its label.
    pub dots: Vec<Vec<[Mob; 2]>>,
    /// Each helper's curve, scaled to 1 at its own trustee.
    pub curves: Vec<Mob>,
    /// The scale tags, "× 19" and on.
    pub tags: Vec<Mob>,
}

impl Setting {
    /// The pieces for the trio `trio` (position, height on paper), the
    /// dealer's parabola `recipe`'s, in `parabola_color`.
    pub fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        trio: &[(u32, i64)],
        recipe: &Recipe,
        parabola_color: Srgb,
    ) -> fmn::Result<Self> {
        let xs: Vec<u32> = trio.iter().map(|p| p.0).collect();
        let rows = at_trio(&xs).map_err(fail)?;
        let plot = paper_axes(kit, stage)?;
        let mut points = Vec::with_capacity(trio.len());
        for &(x, h) in trio {
            let p = paper_at(f64::from(x), h as f64);
            let dot = disc(stage, palette::trio(), p, POINT_R);
            let label = kit.text(
                stage,
                &h.to_string(),
                POINT_LABEL_SIZE,
                palette::trio(),
                v(p[0] + POINT_LABEL_DX, p[1] + POINT_LABEL_DY),
            )?;
            points.push([dot, label]);
        }
        let meets = paper_at(0.0, f64::from(recipe.secret));
        let unknown = kit.text(
            stage,
            UNKNOWN,
            UNKNOWN_SIZE,
            palette::secret(),
            v(meets[0] + UNKNOWN_DX, meets[1] + UNKNOWN_DY),
        )?;
        let dealt_curve = sampled(|x| dealer_height(recipe, x), 0.0, CURVE_END);
        let parabola = on_paper(stage, &dealt_curve, parabola_color, PARABOLA_W);
        let sum_curve = sampled(|x| weighted_sum_at(trio, x), 0.0, CURVE_END);
        let sum = on_paper(stage, &sum_curve, palette::trio(), SUM_W);
        let mut boards = Vec::with_capacity(xs.len());
        for (b, &t) in xs.iter().enumerate() {
            boards.push(board_axes(kit, stage, b, t)?);
        }
        let mut heads = Vec::with_capacity(LAST as usize + 2);
        for x in 0..=LAST {
            heads.push(kit.text(
                stage,
                &x.to_string(),
                HEAD_SIZE,
                Palette::dim(),
                v(on_board(0, f64::from(x), 0.0)[0], HEAD_Y),
            )?);
        }
        heads.push(text_right(
            kit,
            stage,
            POSITION_TITLE,
            HEAD_SIZE,
            Palette::dim(),
            v(HELPER_X0 - NAME_GAP, HEAD_Y),
        )?);
        let mut dots = Vec::with_capacity(xs.len());
        for (b, row) in rows.iter().enumerate() {
            let mut on = Vec::with_capacity(row.len());
            for (&x, &value) in xs.iter().zip(row) {
                let (xf, hf) = (f64::from(x), value as f64);
                let color = if value == 0 { Palette::dim() } else { palette::trio() };
                let dot = disc(stage, color, on_board(b, xf, hf), DOT_R);
                let label = kit.text(
                    stage,
                    &value.to_string(),
                    DOT_LABEL_SIZE,
                    color,
                    dot_label_at(b, xf, hf),
                )?;
                on.push([dot, label]);
            }
            dots.push(on);
        }
        let mut curves = Vec::with_capacity(xs.len());
        for b in 0..xs.len() {
            let samples = sampled(|x| helper_value(&xs, b, x), 0.0, CURVE_END);
            curves.push(on_board_stroke(stage, b, &samples, palette::trio()));
        }
        let mut tags = Vec::with_capacity(xs.len());
        for (b, &(_, h)) in trio.iter().enumerate() {
            tags.push(kit.text_left(stage, &scale_tag(h), TAG_SIZE, palette::trio(), v(TAG_X, HELPER_Y[b]))?);
        }
        Ok(Self {
            plot,
            points,
            unknown,
            parabola,
            sum,
            boards,
            heads,
            dots,
            curves,
            tags,
        })
    }

    /// The plot's points' dots.
    pub fn point_dots(&self) -> Vec<Mob> {
        self.points.iter().map(|p| p[0]).collect()
    }

    /// Everything, in the order chapter 15 brings it in.
    pub fn all(&self) -> Vec<Mob> {
        let mut all = self.plot.clone();
        all.extend([self.parabola, self.sum]);
        all.extend(self.points.iter().flatten());
        all.push(self.unknown);
        all.extend(self.boards.iter().flatten());
        all.extend(&self.heads);
        all.extend(&self.curves);
        all.extend(self.dots.iter().flatten().flatten());
        all.extend(&self.tags);
        all
    }
}

/// The dealer's height on paper at any position, for drawing.
pub fn dealer_height(recipe: &Recipe, x: f64) -> f64 {
    recipe
        .coefficients()
        .iter()
        .rev()
        .fold(0.0, |acc, &c| acc * x + f64::from(c))
}

/// The trio's points on paper: each position with its height.
pub fn paper_trio(recipe: &Recipe) -> Vec<(u32, i64)> {
    OPENING_TRIO.iter().map(|&x| (x, recipe.height(x))).collect()
}

/// The scene stops unless the helpers do their jobs: each 1 at its own
/// trustee and 0 at the others, the weighted sum giving each height and, at
/// position 0, the secret, and the drawn sum lying on the dealer's
/// parabola at every sample.
pub fn check_helpers(trio: &[(u32, i64)], recipe: &Recipe) -> Result<(), String> {
    let xs: Vec<u32> = trio.iter().map(|p| p.0).collect();
    at_trio(&xs)?;
    for &(x, h) in trio {
        if sum_exact(trio, i64::from(x)) != Ratio::int(h) {
            return Err(format!("the sum misses ({x}, {h})"));
        }
    }
    if sum_exact(trio, 0) != Ratio::int(i64::from(recipe.secret)) {
        return Err("the sum's height at 0 is not the secret".into());
    }
    for (x, h) in sampled(|x| weighted_sum_at(trio, x), 0.0, CURVE_END) {
        if (h - dealer_height(recipe, x)).abs() > 1e-9 {
            return Err(format!("the sum leaves the parabola at {x}"));
        }
        if !(0.0..=PAPER_TOP as f64).contains(&h) {
            return Err(format!("the sum leaves the plot at {x}"));
        }
    }
    for b in 0..xs.len() {
        for (x, h) in sampled(|x| helper_value(&xs, b, x), 0.0, CURVE_END) {
            if h.abs() > HELPER_SPAN + 1e-9 {
                return Err(format!("helper {b} leaves its board at {x}"));
            }
        }
    }
    Ok(())
}

impl SceneConstruct for HelperCurves {
    fn name(&self) -> &str {
        "14_helper_curves"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let dial = Dial::vault();
        let recipe = Recipe::example();
        let shares = dealt(dial, &OPENING_TRIO);
        let trio = paper_trio(&recipe);
        check_helpers(&trio, &recipe).map_err(fail)?;
        let xs: Vec<u32> = trio.iter().map(|p| p.0).collect();
        let sums: Vec<String> = (0..trio.len())
            .map(|j| sum_line(&trio, j))
            .collect::<Result<_, _>>()
            .map_err(fail)?;
        let card_words = card_text(&trio);
        // the dealer's parabola in ink, as chapter 09 drew it on paper
        let setting = Setting::build(&kit, stage, &trio, &recipe, Palette::ink())?;
        let others: Vec<u32> = TRUSTEES.iter().copied().filter(|t| !xs.contains(t)).collect();

        // the opening: the rail, the trio teal with their cards, the others
        // grey; the vault, shut
        let rail = Rail::build(&kit, stage, &TRUSTEES)?;
        rail.reveal(stage, 0.6)?;
        let mut cards = Vec::with_capacity(shares.len());
        for &(t, y) in &shares {
            cards.push(card(&kit, stage, t, &y.to_string(), palette::share())?);
        }
        let vault = Vault::build(&kit, stage, dial)?;
        let mut anims = swaps(stage, &[], &cards, 0.6)?;
        anims.extend(recolorings(stage, &rail.tint(&xs, palette::trio()), 0.6)?);
        anims.extend(recolorings(stage, &rail.tint(&others, palette::ghost()), 0.6)?);
        stage.play_prepared(anims)?;
        vault.show(stage, 0.6)?;

        // 14_1: the door from chapter 04, opened by three numbers; three
        // points on the dial, no spins
        let spoken = say(&kit, stage, "14_1")?;
        spoken.wait_for(stage, "brought their eight, three and one")?;
        let halos = frame_halos(stage, &xs, palette::trio())?;
        glow(stage, &halos, 0.9)?;
        spoken.wait_for(stage, "and the door opened")?;
        vault.turn(stage, SECRET, 0.7)?;
        recolor_all(stage, &[(vault.door, palette::open())], 0.3)?;
        door_glow(stage, &vault, palette::open(), 0.6)?;
        spoken.wait_for(stage, "How they did it was left for later")?;
        let anims = vec![
            stage.prepare(recolor(vault.door, palette::ghost(), 0.6)?)?,
            stage.prepare(tween(vault.needle, 0.0, 0.6)?)?,
        ];
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "They know three points")?;
        let board = Board::build(&kit, stage, DIAL_BOARD, BOARD_TITLE, true)?;
        board.show(stage, 0.6)?;
        let marked: Vec<Mob> = shares
            .iter()
            .map(|&(x, y)| board.point(stage, x, y, palette::trio()))
            .collect();
        let rings: Vec<Mob> = shares
            .iter()
            .map(|&(x, y)| {
                ring(
                    stage,
                    DIAL_BOARD.at(f64::from(x), f64::from(y)),
                    POINT_RING,
                    palette::trio(),
                )
            })
            .collect();
        let mut anims = swaps(stage, &[], &marked, 0.6)?;
        anims.extend(glowing(stage, &rings, 1.0)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "but not its spins")?;
        let spins = kit.text(stage, SPINS_UNKNOWN, SPINS_SIZE, palette::spin(), v(SPINS_X, SPINS_Y))?;
        fade_in_all(stage, &[spins], 0.4)?;
        spoken.wait_for(stage, "So how can three numbers give five")?;
        let o = ring(stage, v(DOOR_X, DIAL_Y), DIAL_R + 0.12, palette::secret());
        glow(stage, &[o], 1.0)?;
        spoken.wait_end(stage)?;

        // 14_2: on paper, the trio's heights; one parabola through them,
        // meeting the side at the secret
        let spoken = say(&kit, stage, "14_2")?;
        let mut keep: Vec<Mob> = rail.parts().to_vec();
        keep.extend(&cards);
        let anims = leaving_all_but(stage, &keep, ORIGIN, 0.5)?;
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "go back to paper")?;
        fade_in_all(stage, &setting.plot, 0.7)?;
        for (i, phrase) in ["nineteen", "forty-seven", "eighty-nine"].into_iter().enumerate() {
            spoken.wait_for(stage, phrase)?;
            let r = disc_ring(stage, xs[i], palette::trio());
            let mut anims = swaps(stage, &[], &setting.points[i], 0.5)?;
            anims.extend(glowing(stage, &[r], 0.9)?);
            stage.play_prepared(anims)?;
        }
        spoken.wait_for(stage, "Three points pin a parabola down")?;
        let a = stage.prepare(show_creation(setting.parabola).rt(1.2))?;
        stage.play_prepared(vec![a])?;
        spoken.wait_for(stage, "its height at zero is the secret")?;
        fade_in_all(stage, &[setting.unknown], 0.4)?;
        spoken.wait_end(stage)?;

        // 14_3: three boards, one helper each, only their jobs at the three
        // positions
        let spoken = say(&kit, stage, "14_3")?;
        spoken.wait_for(stage, "Give each trustee a helper curve")?;
        let mut parts: Vec<Mob> = setting.boards.iter().flatten().copied().collect();
        parts.extend(&setting.heads);
        fade_in_all(stage, &parts, 0.7)?;
        spoken.wait_for(stage, "Trustee one's helper is one at position one")?;
        fade_in_all(stage, &setting.dots[0][0], 0.4)?;
        spoken.wait_for(stage, "and zero at positions two and three")?;
        let zero_dots: Vec<Mob> = setting.dots[0][1..].iter().flatten().copied().collect();
        fade_in_all(stage, &zero_dots, 0.4)?;
        spoken.wait_for(stage, "Trustee two's is one at position two")?;
        let row: Vec<Mob> = setting.dots[1].iter().flatten().copied().collect();
        fade_in_all(stage, &row, 0.4)?;
        spoken.wait_for(stage, "trustee three's is one at position three")?;
        let row: Vec<Mob> = setting.dots[2].iter().flatten().copied().collect();
        fade_in_all(stage, &row, 0.4)?;
        spoken.wait_for(stage, "answers to its own trustee only")?;
        let own: Vec<Mob> = (0..xs.len())
            .map(|b| ring(stage, on_board(b, f64::from(xs[b]), 1.0), DOT_RING, palette::trio()))
            .collect();
        glow(stage, &own, 1.1)?;
        spoken.wait_end(stage)?;

        // 14_4: scaled by the heights and added; position by position, only
        // that trustee's helper counts
        let spoken = say(&kit, stage, "14_4")?;
        for (b, phrase) in [
            "nineteen times the first",
            "forty-seven times the second",
            "eighty-nine times the third",
        ]
        .into_iter()
        .enumerate()
        {
            spoken.wait_for(stage, phrase)?;
            fade_in_all(stage, &[setting.tags[b]], 0.4)?;
        }
        let mut sum_mobs = Vec::with_capacity(sums.len());
        let mut column = None;
        for (j, cue) in ["At position one", "At position two", "and at three"]
            .into_iter()
            .enumerate()
        {
            spoken.wait_for(stage, cue)?;
            match column {
                None => {
                    let (at, height) = column_at(xs[j]);
                    let o = outline(stage, at, CELL_COLUMN_W, height, palette::trio())?;
                    fade_in_all(stage, &[o], 0.4)?;
                    column = Some(o);
                }
                Some(o) => {
                    let to = column_at(xs[j]).0;
                    let a = moving(stage, o, to, 0.5)?;
                    stage.play_prepared(vec![a])?;
                }
            }
            if j == 0 {
                spoken.wait_for(stage, "zero times anything is zero")?;
                let rings: Vec<Mob> = (0..xs.len())
                    .filter(|&b| b != j)
                    .map(|b| ring(stage, on_board(b, f64::from(xs[j]), 0.0), DOT_RING, Palette::dim()))
                    .collect();
                glow(stage, &rings, 0.9)?;
                spoken.wait_for(stage, "so the sum there is nineteen")?;
            }
            let line = kit.text_left(stage, &sums[j], WORK_SIZE, Palette::ink(), v(WORK_X, WORK_Y[j]))?;
            let r = ring(
                stage,
                paper_at(f64::from(xs[j]), trio[j].1 as f64),
                POINT_RING,
                palette::trio(),
            );
            let mut anims = swaps(stage, &[], &[line], 0.5)?;
            anims.extend(glowing(stage, &[r], 1.0)?);
            stage.play_prepared(anims)?;
            sum_mobs.push(line);
        }
        let column = column.ok_or_else(|| fail("no column outline".into()))?;
        spoken.wait_for(stage, "passes through all three points")?;
        swell_all(stage, &setting.point_dots(), palette::trio(), 0.8)?;
        spoken.wait_end(stage)?;

        // 14_5: what recipe is 0 at positions 2 and 3?
        let spoken = say(&kit, stage, "14_5")?;
        let mut gone = sum_mobs.clone();
        gone.push(column);
        let mut anims = leaving(stage, &gone, ORIGIN, 0.5)?;
        let dimmed: Vec<(Mob, Srgb)> = setting.dots[1..]
            .iter()
            .flatten()
            .map(|d| (d[0], palette::ghost()))
            .collect();
        anims.extend(recolorings(stage, &dimmed, 0.5)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "What simple recipe")?;
        let first_zeros = zeros(&xs, 0);
        let rings: Vec<Mob> = first_zeros
            .iter()
            .map(|&x| ring(stage, on_board(0, f64::from(x), 0.0), DOT_RING, Palette::dim()))
            .collect();
        glow(stage, &rings, 1.0)?;
        spoken.wait_end(stage)?;
        let question = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 14_6: (position − 2) × (position − 3), 2 at position 1, halved
        let spoken = say(&kit, stage, "14_6")?;
        let anims = leaving(stage, &[question], ORIGIN, 0.3)?;
        stage.play_prepared(anims)?;
        let mut work = Vec::new();
        let (first, rest) = helper_steps(&kit, stage, &spoken, &xs, 0, &setting)?;
        work.extend(first);
        work.extend(rest);
        spoken.wait_for(stage, "On paper, halving is no trouble")?;
        let note = kit.text_left(stage, HALVING, NOTE_SIZE, Palette::dim(), v(WORK_X, WORK_Y[2]))?;
        fade_in_all(stage, &[note], 0.4)?;
        work.push(note);
        spoken.wait_end(stage)?;

        // 14_7: the other two helpers, the same steps
        let spoken = say(&kit, stage, "14_7")?;
        let mut anims = leaving(stage, &work, ORIGIN, 0.5)?;
        let lit: Vec<(Mob, Srgb)> = (1..xs.len())
            .flat_map(|b| {
                setting.dots[b]
                    .iter()
                    .enumerate()
                    .map(move |(j, d)| (d[0], if j == b { palette::trio() } else { Palette::dim() }))
            })
            .collect();
        anims.extend(recolorings(stage, &lit, 0.5)?);
        stage.play_prepared(anims)?;
        let mut work = Vec::new();
        for b in 1..xs.len() {
            let (first, rest) = helper_steps(&kit, stage, &spoken, &xs, b, &setting)?;
            work.extend(first);
            work.extend(rest);
        }
        spoken.wait_end(stage)?;

        // 14_8: the weighted sum on paper lies on the dealer's parabola
        let spoken = say(&kit, stage, "14_8")?;
        let anims = leaving(stage, &work, ORIGIN, 0.5)?;
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "Adding scaled helpers keeps that form")?;
        let a = stage.prepare(show_creation(setting.sum).rt(1.4))?;
        stage.play_prepared(vec![a])?;
        spoken.wait_for(stage, "passes through the dealer's three points")?;
        let rings: Vec<Mob> = trio
            .iter()
            .map(|&(x, h)| ring(stage, paper_at(f64::from(x), h as f64), POINT_RING, palette::trio()))
            .collect();
        glow(stage, &rings, 1.0)?;
        // the two curves become one: the parabola takes the sum's color
        spoken.wait_for(stage, "Only one curve of that form can do that")?;
        recolor_all(stage, &[(setting.parabola, palette::trio())], 0.6)?;
        spoken.wait_for(stage, "Its height at zero is the secret")?;
        let meets = ring(
            stage,
            paper_at(0.0, f64::from(recipe.secret)),
            POINT_RING,
            palette::secret(),
        );
        let o = text_halo(stage, setting.unknown, palette::secret())?;
        glow(stage, &[meets, o], 1.0)?;
        let exact = precisely(&kit, stage, &card_words)?;
        let anims = swaps(stage, &[], &[exact], 0.5)?;
        stage.play_prepared(anims)?;
        let read = read_by(stage, &card_words);
        spoken.wait_end(stage)?;
        wait_until(stage, read)?;
        pause(stage, HOLD_AT_END)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

/// The phrases 14_6 and 14_7 wait for as trustee `i`'s helper is built:
/// its product, the product at its own position, and the scaling.
pub fn step_cues(i: usize) -> [&'static str; 4] {
    match i {
        0 => [
            "Position minus two, times position minus three",
            "At position one it's minus one times minus two",
            "which is two,",
            "so halve it",
        ],
        1 => [
            "Trustee two's is position minus one, times position minus three",
            "which is minus one at position two",
            "which is minus one at position two",
            "so divide by minus one",
        ],
        _ => [
            "Trustee three's is position minus one, times position minus two",
            "which is two at position three",
            "which is two at position three",
            "so halve it",
        ],
    }
}

/// Trustee `i`'s helper built on its board as `spoken` says it: the
/// product's line and its curve, the product at the trustee's own
/// position with a ring there, then the scaling, the curve giving way to
/// the helper's. Returns the lines written, first the product's.
fn helper_steps(
    kit: &Kit,
    stage: &mut Stage<'_>,
    spoken: &Spoken,
    xs: &[u32],
    i: usize,
    setting: &Setting,
) -> fmn::Result<(Vec<Mob>, Vec<Mob>)> {
    let cues = step_cues(i);
    // trustee 1's lines are the first two; 2's and 3's take two each
    let row = if i == 0 { 0 } else { 2 * (i - 1) };
    let own = model::helper_at_own(xs, i);
    spoken.wait_for(stage, cues[0])?;
    let recipe = kit.text_left(
        stage,
        &recipe_line(xs, i),
        WORK_SIZE,
        Palette::ink(),
        v(WORK_X, WORK_Y[row]),
    )?;
    fade_in_all(stage, &[recipe], 0.5)?;
    if i == 0 {
        spoken.wait_for(stage, "Each factor is zero at its own position")?;
        let rings: Vec<Mob> = zeros(xs, i)
            .iter()
            .map(|&x| ring(stage, on_board(i, f64::from(x), 0.0), DOT_RING, palette::trio()))
            .collect();
        glow(stage, &rings, 0.9)?;
        spoken.wait_for(stage, "so the product is zero at both")?;
    }
    let raw = sampled(|x| product_at(xs, i, x), 0.0, CURVE_END);
    let runs = clipped(&raw, -HELPER_SPAN, HELPER_SPAN);
    let product: Vec<Mob> = runs
        .iter()
        .map(|r| on_board_stroke(stage, i, r, Palette::ink()))
        .collect();
    let mut drawn = Vec::with_capacity(product.len());
    for &m in &product {
        drawn.push(stage.prepare(show_creation(m).rt(1.0))?);
    }
    stage.play_prepared(drawn)?;
    spoken.wait_for(stage, cues[1])?;
    let at_own = if i == 0 {
        own_working(xs, i)[0].clone()
    } else {
        own_line(xs, i)
    };
    let working = kit.text_left(stage, &at_own, WORK_SIZE, Palette::ink(), v(WORK_X, WORK_Y[row + 1]))?;
    fade_in_all(stage, &[working], 0.5)?;
    let mut lines = vec![working];
    spoken.wait_for(stage, cues[2])?;
    let r = ring(
        stage,
        on_board(i, f64::from(xs[i]), own as f64),
        DOT_RING,
        Palette::ink(),
    );
    let mut anims = glowing(stage, &[r], 1.0)?;
    if i == 0 {
        let value = text_after(kit, stage, working, &own_working(xs, i)[1], WORK_SIZE, Palette::ink())?;
        anims.extend(swaps(stage, &[], &[value], 0.4)?);
        lines.push(value);
    }
    stage.play_prepared(anims)?;
    spoken.wait_for(stage, cues[3])?;
    let divide = text_after(kit, stage, recipe, &divide_text(own), WORK_SIZE, Palette::ink())?;
    let mut anims = swaps(stage, &product, &[setting.curves[i]], 0.8)?;
    anims.extend(swaps(stage, &[], &[divide], 0.5)?);
    stage.play_prepared(anims)?;
    lines.push(divide);
    Ok((vec![recipe], lines))
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::super::draw::{FRAME_H, FRAME_W, FRAME_Y, RAIL_DX, RAIL_X0, THINK_R};
    use super::*;
    use crate::facts;
    use crate::model::csv;

    /// The production narration: registering chapter 14 brings its lines
    /// into script.tsv.
    const SCRIPT: &str = crate::SCRIPT;

    /// Generous text sizes, as the other chapters' tests.
    fn text_w(text: &str, size: f64) -> f64 {
        0.0055 * size * text.chars().count() as f64
    }

    fn text_h(size: f64) -> f64 {
        0.0065 * size
    }

    /// Trustee `i`'s helper written out with fractions of whole numbers,
    /// apart from the model: the product of (at − other) over the product
    /// of (own − other), as (top, bottom).
    fn by_hand(xs: &[i64], i: usize, at: i64) -> (i64, i64) {
        let (mut top, mut bottom) = (1, 1);
        for (j, &x) in xs.iter().enumerate() {
            if j != i {
                top *= at - x;
                bottom *= xs[i] - x;
            }
        }
        (top, bottom)
    }

    /// Every phrase the scene waits for is in its line of the script, once,
    /// and they come in the order they are said.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(SCRIPT).unwrap();
        let mut phrases = vec![
            ("14_1", "brought their eight, three and one"),
            ("14_1", "and the door opened"),
            ("14_1", "How they did it was left for later"),
            ("14_1", "They know three points"),
            ("14_1", "but not its spins"),
            ("14_1", "So how can three numbers give five"),
            ("14_2", "go back to paper"),
            ("14_2", "nineteen"),
            ("14_2", "forty-seven"),
            ("14_2", "eighty-nine"),
            ("14_2", "Three points pin a parabola down"),
            ("14_2", "its height at zero is the secret"),
            ("14_3", "Give each trustee a helper curve"),
            ("14_3", "Trustee one's helper is one at position one"),
            ("14_3", "and zero at positions two and three"),
            ("14_3", "Trustee two's is one at position two"),
            ("14_3", "trustee three's is one at position three"),
            ("14_3", "answers to its own trustee only"),
            ("14_4", "nineteen times the first"),
            ("14_4", "forty-seven times the second"),
            ("14_4", "eighty-nine times the third"),
            ("14_4", "At position one"),
            ("14_4", "zero times anything is zero"),
            ("14_4", "so the sum there is nineteen"),
            ("14_4", "At position two"),
            ("14_4", "and at three"),
            ("14_4", "passes through all three points"),
            ("14_5", "What simple recipe"),
        ];
        // 14_6 and 14_7 wait for each helper's steps in turn
        let c = step_cues(0);
        phrases.extend([
            ("14_6", c[0]),
            ("14_6", "Each factor is zero at its own position"),
            ("14_6", "so the product is zero at both"),
            ("14_6", c[1]),
            ("14_6", c[2]),
            ("14_6", c[3]),
            ("14_6", "On paper, halving is no trouble"),
        ]);
        for i in 1..3 {
            let c = step_cues(i);
            // the product's value and the ring there share one phrase
            assert_eq!(c[1], c[2]);
            phrases.extend([("14_7", c[0]), ("14_7", c[1]), ("14_7", c[3])]);
        }
        phrases.extend([
            ("14_8", "Adding scaled helpers keeps that form"),
            ("14_8", "passes through the dealer's three points"),
            ("14_8", "Only one curve of that form can do that"),
            ("14_8", "Its height at zero is the secret"),
        ]);
        let mut last: Option<(&str, usize)> = None;
        for (id, phrase) in phrases {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert_eq!(text.matches(phrase).count(), 1, "{id}: {phrase:?} in {text}");
            let at = text.find(phrase).unwrap();
            if let Some((prev_id, prev_at)) = last {
                assert!(
                    prev_id != id || prev_at < at,
                    "{id}: {phrase:?} is waited for out of order"
                );
            }
            last = Some((id, at));
        }
        // the question ends 14_5, the think time after it
        assert!(script.text("14_5").unwrap().ends_with("two and three?"));
    }

    /// What the scene draws and writes is the facts': the trio's cards and
    /// heights, each helper at the three positions, its factors and its
    /// value at its own trustee, and the sums.
    #[test]
    fn the_pictures_are_the_facts() {
        let f = facts::facts().unwrap();
        let get = |k: &str| f.get(k).unwrap_or_else(|| panic!("no fact {k}")).to_owned();
        let recipe = Recipe::example();
        let trio = paper_trio(&recipe);
        let xs: Vec<u32> = trio.iter().map(|p| p.0).collect();
        assert_eq!(get("S-F1.opening_trio"), csv(&xs));
        let shares: Vec<u32> = dealt(Dial::vault(), &OPENING_TRIO).iter().map(|p| p.1).collect();
        assert_eq!(get("S-F1.opening_shares"), csv(&shares));
        let heights: Vec<i64> = trio.iter().map(|p| p.1).collect();
        assert_eq!(get("S-F9.paper.heights"), csv(&heights));
        let rows = at_trio(&xs).unwrap();
        let own: Vec<i64> = (0..xs.len()).map(|i| model::helper_at_own(&xs, i)).collect();
        for (i, row) in rows.iter().enumerate() {
            let n = i + 1;
            assert_eq!(get(&format!("S-F9.helper.{n}.position")), xs[i].to_string());
            assert_eq!(get(&format!("S-F9.helper.{n}.at_trio")), csv(row));
            assert_eq!(get(&format!("S-F9.helper.{n}.zeros")), csv(&zeros(&xs, i)));
            assert_eq!(
                get(&format!("S-F9.helper.{n}.factors_at_own")),
                csv(&factors_at(&xs, i, i64::from(xs[i])))
            );
            assert_eq!(get(&format!("S-F9.helper.{n}.own")), own[i].to_string());
            assert_eq!(get(&format!("S-F9.helper.{n}.scaled")), csv(&[heights[i], n as i64]));
            // the sum at this trustee's position is its height
            assert_eq!(
                get(&format!("S-F9.sum_at.{n}")),
                sum_exact(&trio, i64::from(xs[i])).to_string()
            );
        }
        assert_eq!(get("S-F9.helper_at_own"), csv(&own));
        assert_eq!(get("S-F9.helper.on"), "1");
        assert_eq!(get("S-F9.helper.off"), "0");
        assert_eq!(get("S-F9.helper.others"), (xs.len() - 1).to_string());
        // at position 1, the helpers that are 0 there (14_4)
        let zero_there: Vec<usize> = (0..xs.len()).filter(|&i| rows[i][0] == 0).map(|i| i + 1).collect();
        assert_eq!(get("S-F9.sum_at.1.zero_helpers"), csv(&zero_there));
        assert_eq!(get("S-F1.secret"), sum_exact(&trio, 0).to_string());
        // the written lines, as the scene builds them
        assert_eq!(
            sum_line(&trio, 0).unwrap(),
            "at 1: 19 \u{d7} 1 + 47 \u{d7} 0 + 89 \u{d7} 0 = 19"
        );
        assert_eq!(
            sum_line(&trio, 2).unwrap(),
            "at 3: 19 \u{d7} 0 + 47 \u{d7} 0 + 89 \u{d7} 1 = 89"
        );
        assert_eq!(
            recipe_line(&xs, 0),
            "trustee 1's helper: (position \u{2212} 2) \u{d7} (position \u{2212} 3)"
        );
        assert_eq!(
            own_working(&xs, 0),
            [
                "at 1: (1 \u{2212} 2) \u{d7} (1 \u{2212} 3) = (\u{2212}1) \u{d7} (\u{2212}2)".to_owned(),
                "= 2".to_owned()
            ]
        );
        assert_eq!(
            own_line(&xs, 1),
            "at 2: (2 \u{2212} 1) \u{d7} (2 \u{2212} 3) = \u{2212}1"
        );
        assert_eq!(own_line(&xs, 2), "at 3: (3 \u{2212} 1) \u{d7} (3 \u{2212} 2) = 2");
        assert_eq!(divide_text(own[0]), "\u{f7} 2");
        assert_eq!(divide_text(own[1]), "\u{f7} (\u{2212}1)");
        assert_eq!(scale_tag(19), "\u{d7} 19");
    }

    /// The helpers counted by hand: each is 1 at its own trustee and 0 at
    /// the others, the weighted sum gives 19, 47, 89 and, at 0, 5, and the
    /// controls fail as they should: a helper left unhalved is 2 at its
    /// trustee, two trustees at one position have no helpers, and the
    /// weights on another trio give another curve.
    #[test]
    fn the_helpers_are_counted_by_hand() {
        let recipe = Recipe::example();
        let trio = paper_trio(&recipe);
        let xs: Vec<i64> = trio.iter().map(|p| i64::from(p.0)).collect();
        for i in 0..3 {
            for (j, &x) in xs.iter().enumerate() {
                let (top, bottom) = by_hand(&xs, i, x);
                assert_eq!(top % bottom, 0);
                assert_eq!(top / bottom, i64::from(i == j), "helper {i} at {x}");
            }
        }
        for &(x, h) in &trio {
            let total: i64 = (0..3)
                .map(|i| {
                    let (top, bottom) = by_hand(&xs, i, i64::from(x));
                    trio[i].1 * top / bottom
                })
                .sum();
            assert_eq!(total, h);
        }
        // at 0 the helpers are 3, −3 and 1 and the sum is the secret
        let at0: Vec<(i64, i64)> = (0..3).map(|i| by_hand(&xs, i, 0)).collect();
        assert_eq!(at0, vec![(6, 2), (3, -1), (2, 2)]);
        let paper: i64 = (0..3).map(|i| trio[i].1 * at0[i].0 / at0[i].1).sum();
        assert_eq!(paper, i64::from(SECRET));
        check_helpers(&trio, &recipe).unwrap();
        // controls: the product alone is 2 at trustee 1, not 1
        assert!((product_at(&OPENING_TRIO, 0, 1.0) - 2.0).abs() < 1e-12);
        // a wrong height breaks the parabola check
        let mut wrong = trio.clone();
        wrong[1].1 += 1;
        assert!(check_helpers(&wrong, &recipe).is_err());
        // another trio's weighted sum is the same parabola but another
        // helper set: 2, 4, 5 have weights 10/3, −5, 8/3, not whole
        let other: Vec<(u32, i64)> = [2, 4, 5].iter().map(|&x| (x, recipe.height(x))).collect();
        assert_eq!(sum_exact(&other, 0), Ratio::int(i64::from(SECRET)));
        assert_eq!(helper_exact(&[2, 4, 5], 0, 0), Ratio::new(10, 3));
    }

    /// The clip keeps what is inside, ends where a piece crosses out, and
    /// leaves the helpers whole: only trustee 1's product leaves its board,
    /// climbing past 3 before position 0.7.
    #[test]
    fn the_clip_cuts_where_the_curve_leaves() {
        let raw = sampled(|x| product_at(&OPENING_TRIO, 0, x), 0.0, CURVE_END);
        let runs = clipped(&raw, -HELPER_SPAN, HELPER_SPAN);
        assert_eq!(runs.len(), 1);
        let (x0, h0) = runs[0][0];
        // (x − 2)(x − 3) = 3 at x = (5 − √13) / 2
        assert!((x0 - (5.0 - 13f64.sqrt()) / 2.0).abs() < 1e-3, "{x0}");
        assert!((h0 - HELPER_SPAN).abs() < 1e-12);
        assert!(runs[0].iter().all(|&(_, h)| h.abs() <= HELPER_SPAN + 1e-12));
        for i in 1..3 {
            let raw = sampled(|x| product_at(&OPENING_TRIO, i, x), 0.0, CURVE_END);
            let runs = clipped(&raw, -HELPER_SPAN, HELPER_SPAN);
            assert_eq!(runs.len(), 1);
            assert_eq!(runs[0].len(), raw.len());
        }
        for i in 0..3 {
            let s = sampled(|x| helper_value(&OPENING_TRIO, i, x), 0.0, CURVE_END);
            assert_eq!(clipped(&s, -HELPER_SPAN, HELPER_SPAN)[0].len(), s.len());
        }
        // a curve that comes in, leaves below and comes back gives two runs
        let wave = sampled(|x| 4.0 * (x * 2.0).cos(), 0.0, CURVE_END);
        assert_eq!(clipped(&wave, -3.0, 3.0).len(), 2);
    }

    /// The card stays two lines at every wrap the lower third narrows to.
    #[test]
    fn the_card_is_two_lines() {
        let text = card_text(&paper_trio(&Recipe::example()));
        for width in (72..=92).rev().step_by(4) {
            let lines = explainer_kit::animatic::wrap(&text, width, 3);
            assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
        }
    }

    /// The pieces keep apart, inside the frame: the paper plot left over
    /// the rail's frames, the helper boards right with their names between
    /// them and the plot, the tags right of the boards, the working under
    /// the boards right of the rail and over the lower third's band, and
    /// the think ring right of the working.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        let edge = FRAME_X_RADIUS - 0.25;
        let top = FRAME_Y_RADIUS - 0.25;
        let frames_top = FRAME_Y + FRAME_H / 2.0;
        let rail_right = RAIL_X0 + RAIL_DX * 4.0 + FRAME_W / 2.0;
        // the plot: labels inside, positions over the frames, the title
        // under the top, the curve's top under the title
        const {
            assert!(PAPER_Y0 - PAPER_POSITION_DY - PAPER_LABEL_SIZE * 0.0065 / 2.0 > FRAME_Y + FRAME_H / 2.0 + 0.2);
            assert!(PAPER_TITLE_Y + PAPER_TITLE_SIZE * 0.0065 / 2.0 < FRAME_Y_RADIUS - 0.25);
        }
        assert!(PAPER_X0 - PAPER_LABEL_DX - text_w("100", PAPER_LABEL_SIZE) / 2.0 > -edge);
        let height_title = paper_at(0.0, PAPER_TOP as f64)[1] + AXIS_TITLE_GAP + text_h(PAPER_LABEL_SIZE) / 2.0;
        assert!(height_title < PAPER_TITLE_Y - text_h(PAPER_TITLE_SIZE) / 2.0 - 0.1);
        let label_top = paper_at(3.0, 89.0)[1] + POINT_LABEL_DY + text_h(POINT_LABEL_SIZE) / 2.0;
        assert!(
            label_top < PAPER_TITLE_Y - text_h(PAPER_TITLE_SIZE) / 2.0 - 0.1,
            "{label_top}"
        );
        let position_right = paper_at(f64::from(LAST), 0.0)[0]
            + PAPER_RUN_ON
            + AXIS_TITLE_GAP / 2.0
            + text_w(POSITION_TITLE, PAPER_LABEL_SIZE);
        // the boards' names between the plot and the boards
        let name_left = HELPER_X0 - NAME_GAP - text_w(&helper_name(3), NAME_SIZE);
        assert!(position_right < name_left - 0.3, "{position_right} {name_left}");
        assert!(paper_at(CURVE_END, 0.0)[0] < name_left - 0.3);
        // the "?" clear of the side's labels and of the curve
        let q = paper_at(0.0, f64::from(SECRET));
        assert!(q[0] + UNKNOWN_DX - text_w(UNKNOWN, UNKNOWN_SIZE) / 2.0 > PAPER_X0 + 0.05);
        let curve_there = paper_at(0.0, dealer_height(&Recipe::example(), UNKNOWN_DX / PAPER_DX))[1];
        assert!(q[1] + UNKNOWN_DY - text_h(UNKNOWN_SIZE) / 2.0 > curve_there + 0.05);
        // the boards: apart, the top one's positions under the frame's top
        for b in 0..2 {
            assert!(HELPER_Y[b] - HELPER_DY * HELPER_SPAN > HELPER_Y[b + 1] + HELPER_DY * HELPER_SPAN + 0.3);
        }
        assert!(HEAD_Y - text_h(HEAD_SIZE) / 2.0 > HELPER_Y[0] + HELPER_DY * HELPER_SPAN + 0.1);
        assert!(HEAD_Y + text_h(HEAD_SIZE) / 2.0 < top);
        // a dot's label stays on its board
        let label_top = dot_label_at(0, 1.0, 1.0)[1] + text_h(DOT_LABEL_SIZE) / 2.0;
        assert!(label_top < HEAD_Y - text_h(HEAD_SIZE) / 2.0, "{label_top}");
        // the tags right of the boards, inside the frame
        let board_right = on_board(0, f64::from(LAST), 0.0)[0] + HELPER_RUN_ON;
        assert!(TAG_X > board_right + 0.2);
        let widest_tag = text_w("\u{2212}3 \u{d7} 47 = \u{2212}141", TAG_SIZE);
        assert!(TAG_X + widest_tag < edge, "{}", TAG_X + widest_tag);
        // the working: right of the rail, under the bottom board, over the
        // lower third (a band at most 2.0 high, two lines 1.02)
        let board_bottom = HELPER_Y[2] - HELPER_DY * HELPER_SPAN;
        assert!(WORK_Y[0] + text_h(WORK_SIZE) / 2.0 < board_bottom - 0.4);
        assert!(WORK_X > rail_right + 0.4, "{rail_right}");
        let band_top = -FRAME_Y_RADIUS + 0.2 + 1.02;
        let xs = OPENING_TRIO;
        let widest = [
            recipe_line(&xs, 0) + " " + &divide_text(2),
            own_working(&xs, 0)[0].clone() + " = 2",
            recipe_line(&xs, 1) + " " + &divide_text(-1),
            sum_line(&paper_trio(&Recipe::example()), 0).unwrap(),
        ]
        .iter()
        .map(|s| text_w(s, WORK_SIZE))
        .fold(0.0, f64::max);
        assert!(WORK_X + widest < edge, "{}", WORK_X + widest);
        // the 14_4 sums and 14_7's four lines end over the band's top, and
        // the think ring sits right of the sums
        assert!(WORK_Y[3] - text_h(WORK_SIZE) / 2.0 > -FRAME_Y_RADIUS + 0.25);
        assert!(WORK_Y[1] - text_h(WORK_SIZE) / 2.0 > band_top + 0.1);
        let sums_right = WORK_X + text_w(&sum_line(&paper_trio(&Recipe::example()), 0).unwrap(), WORK_SIZE);
        assert!(THINK_X - THINK_R > sums_right + 0.3, "{sums_right}");
        assert!(THINK_X + text_w(super::super::draw::PAUSE_NOTE, 18.0) / 2.0 < edge);
        assert!(THINK_Y - THINK_R - 0.3 - text_h(18.0) / 2.0 > -top);
        // the dial board right of the vault, under the frame's top
        assert!(DIAL_BOARD.left() - 0.32 - 0.1 > DOOR_X + super::super::draw::DOOR_W / 2.0 + 0.3);
        assert!(DIAL_BOARD.right() < edge);
        assert!(DIAL_BOARD.title_at()[1] + 0.13 < top);
        assert!(SPINS_Y + text_h(SPINS_SIZE) / 2.0 < DIAL_BOARD.position_label_at(0)[1] - 0.1);
        assert!(SPINS_Y - text_h(SPINS_SIZE) / 2.0 > frames_top + 0.1);
    }
}
