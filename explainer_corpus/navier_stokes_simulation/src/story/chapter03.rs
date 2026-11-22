//! Chapter 03, "Follow a parcel": the narrowing channel, cues 03_1 to 03_12
//! and HOLD 03_h1, native and exact.
//!
//! LavenderPike `lavender-forces01` for root's `navier_stokes_simulation`
//! crate (Mail 36229: "complete 03_9–03_12 with the source's local/material
//! derivative comparison"). Revision 02 of `parcel-scene01`'s
//! `follow_parcel01.rs` (06f6108d…, Mail 36224, kept unchanged in its own
//! folder): cues 03_1 to 03_8 draw as before, except that the prediction card
//! takes its words from a [`CardSpec`] and 03_8 no longer holds at its end.
//! Cue text and pictures: `navier-full-script03.md` chapter 03 (sha256
//! 31a83a2b…), copied there from `navier-opening-script03.md` lines 193 to 232
//! (10760e12…). Fixture and facts: section CH of
//! `navier-opening-script01/tools/facts01.py` (23f955eb…) and its run
//! `navier-opening-script02/inputs/facts01-run02.txt` (08e3870a…).
//!
//! 03_9 to 03_11 write the material derivative in the side panel: the two
//! parts in words, Du/Dt = ∂u/∂t + u ∂u/∂x, the vector form with (u·∇)u, the
//! moving part underlined gold, named "advection" and set beside a struck-out
//! "force". 03_12's tunnel picture (the thumbnail grows, the gap beside the
//! cylinder outlined white) is supplied by [`TunnelGap`].
//! After 03_12's line the chapter clears to the walls alone, the picture
//! chapter 04 opens on (`parcel_forces01.rs`).
//!
//! The channel is the steady field u = 2 + x, v = −y (metres, metres per
//! second) for 0 ≤ x ≤ 2, between the walls y = ±4/(2 + x), which are
//! streamlines. Nothing changes at any fixed point: the field has no time in
//! it. The parcel released on the centre line at the entrance follows
//! x(t) = 2(eᵗ − 1). Its speed 2eᵗ climbs from 2 to 4 m/s and so does its
//! acceleration u ∂u/∂x = 2eᵗ. It reaches the exit at t = ln 2 ≈ 0.69 s.
//! Every arrow, the parcel, its trail and every readout are computed from
//! these formulas at every frame; nothing is drawn by hand.
//!
//! The channel keeps one set of coordinates from 03_2 to 05_6 (opening03,
//! persistent objects): [`SCALE`] and [`ORIGIN_AT`] are that set, through
//! [`to_scene`].
//!
//! This module is self-contained (no peer module, not even chapter 02's): its
//! prediction card repeats the countdown card's design. Chapter 04's module
//! borrows its coordinates, helpers and card from here, so the two chapters
//! share one channel and one card.

use std::cell::Cell;
use std::f64::consts::{FRAC_PI_2, LN_2, TAU};

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Palette, Timed, children, hex};
use fmn::prelude::*;

// ---------------------------------------------------------------- the flow

/// The channel's length, metres (fact ch.length).
pub const LENGTH: f64 = 2.0;

/// The velocity at (x, y), metres per second. Steady: no time in it.
pub fn velocity(x: f64, y: f64) -> (f64, f64) {
    (2.0 + x, -y)
}

/// The speed at (x, y), metres per second.
pub fn speed(x: f64, y: f64) -> f64 {
    (2.0 + x).hypot(y)
}

/// The upper wall's height at `x`; the lower wall is its mirror. Both are
/// streamlines, y (2 + x) = ±4.
pub fn wall(x: f64) -> f64 {
    4.0 / (2.0 + x)
}

/// How fast u grows along x, per second (m/s per metre; fact ch.dudx).
pub const DUDX: f64 = 1.0;
/// How fast the velocity changes at any fixed point (fact ch.dudt.fixed).
pub const DUDT_FIXED: f64 = 0.0;
/// Seconds for the centre-line parcel to cross: x(t) = 2(eᵗ − 1) reaches
/// 2 m at t = ln 2 (fact ch.time.cross, 0.69).
pub const CROSS: f64 = LN_2;

/// The centre-line parcel's distance from the entrance after `t` seconds.
#[cfg_attr(not(test), allow(dead_code))]
pub fn parcel_x(t: f64) -> f64 {
    2.0 * (t.exp() - 1.0)
}

/// The same path by the share `s` of the crossing (t = s ln 2): exactly 0 m
/// at s = 0 and exactly 2 m at s = 1.
pub fn parcel_at(s: f64) -> f64 {
    2.0 * (s.exp2() - 1.0)
}

/// The parcel's speed at share `s`: 2 m/s at the entrance, 4 at the exit.
pub fn parcel_speed(s: f64) -> f64 {
    2.0 * s.exp2()
}

/// The parcel's acceleration at share `s`, u ∂u/∂x on the centre line:
/// 2 m/s² at the entrance, 4 at the exit (facts ch.accel.in, ch.accel.out).
pub fn parcel_accel(s: f64) -> f64 {
    parcel_speed(s) * DUDX
}

/// The share of the crossing shown on frame `k` of an `n`-frame run: 0 at
/// the entrance, exactly 1 on the last frame.
pub fn run_share(k: u64, n: u64) -> f64 {
    k as f64 / n as f64
}

// ------------------------------------------------------------------ layout

/// Scene units per metre, fixed from 03_2 to 05_6.
pub const SCALE: f64 = 1.2;
/// Where the entrance's centre, channel (0, 0), sits: the channel spans x
/// −4.65 to −2.25 and y −2.4 to 2.4, centred in the main window (x −7.1 to
/// 0.2) like chapter 02's window.
pub const ORIGIN_AT: (f64, f64) = (-4.65, 0.0);
/// Polyline samples along each wall.
pub const WALL_SAMPLES: usize = 120;
/// The arrows: four columns, and four rows on the streamlines through 0.45
/// and 0.8 of the local half-width, above and below. The centre line is kept
/// clear for the speed labels, the pin and the parcel.
pub const ARROW_COLUMNS: [f64; 4] = [0.25, 0.75, 1.25, 1.75];
pub const ARROW_ROWS: [f64; 4] = [0.8, 0.45, -0.45, -0.8];
/// Arrow length per unit of speed: 0.13 scene units per m/s, so every length
/// is proportional to the speed it shows (0.31 to 0.50 units here).
pub const ARROW_PER_SPEED: f64 = 0.13;
/// Thin enough that no arrow's shaft or head is capped by its length, so all
/// sixteen share one shaft width.
pub const ARROW_THICKNESS: f64 = 2.0;
/// The fixed gauge pin, halfway along the centre line.
pub const PIN: (f64, f64) = (1.0, 0.0);

/// The 03_2 dimension marks: width lines this far outside each mouth, with
/// end ticks and their labels; the length brace this far below the entrance's
/// lower corner, its label ending above y −3.15 (the caption slot's top,
/// −3.245, less 0.1).
pub const DIM_GAP: f64 = 0.35;
pub const DIM_TICK: f64 = 0.08;
pub const DIM_LABEL_GAP: f64 = 0.15;
pub const BRACE_GAP: f64 = 0.1;
pub const BRACE_LABEL_GAP: f64 = 0.06;
/// The centre-line speed labels: 2 m/s outside the entrance and 4 m/s
/// outside the exit, on the centre line; 3 m/s above the pin, clear of the
/// pin's white 03_8 ring.
pub const END_LABEL_GAP: f64 = 0.25;
pub const MID_LABEL_LIFT: f64 = 0.25;
/// The crossing time, outside the exit under the 4 m/s label.
pub const CROSSING_Y: f64 = -0.4;
/// The white teaching rings of 03_8, round the pin and the parcel.
pub const HIGHLIGHT_R: f64 = 0.17;

pub const TITLE: &str = "Follow a parcel";
pub const TITLE_AT: (f64, f64) = (-6.72, 3.4);
/// Side-panel readout rows: labels end at `LABEL_RIGHT`, live numbers start
/// at `NUM_X`, units start at `UNIT_X`, all on one baseline per row.
pub const LABEL_RIGHT: f64 = 3.8;
pub const NUM_X: f64 = 4.0;
pub const UNIT_X: f64 = 4.95;
/// 03_3: the running clock and the pin's gauge.
pub const CLOCK_BASE: f64 = 1.3;
pub const GAUGE_BASE: f64 = 0.65;
pub const GAUGE_LABEL: &str = "speed at the pin";
/// 03_4: the parcel's givens.
pub const GIVEN_AT: (f64, f64) = (0.9, 1.45);
/// 03_6: the product, in two rows, from sentences 2 and 3.
pub const PRODUCT: [&str; 2] = [r"2\,\mathrm{m/s}\times 1\ \mathrm{per\ s}", r"=2\,\mathrm{m/s^{2}}"];
pub const PRODUCT_AT: [(f64, f64); 2] = [(0.9, 0.45), (0.9, -0.25)];
/// 03_7: the parcel's speed, its acceleration and the flow time.
pub const SPEED_BASE: f64 = -1.0;
pub const ACCEL_BASE: f64 = -1.65;
pub const RUN_CLOCK_BASE: f64 = -2.3;
pub const RUN_NOTE_AT: (f64, f64) = (0.9, -2.75);
pub const ACCEL_LABEL: &str = "speeding up";
pub const ACCEL_UNIT: &str = "m/s per s";
/// 03_8: the split panel.
pub const SPLIT_X: [f64; 2] = [2.15, 5.2];
pub const SPLIT_HEADS: [&str; 2] = ["at a fixed spot", "riding along"];
pub const SPLIT_UNIT: &str = "m/s per second";
pub const DIVIDER_X: f64 = 3.65;
pub const SPLIT_LOW: f64 = -1.0;
pub const SPLIT_HIGH: f64 = 1.3;
pub const HEAD_Y: f64 = 0.85;
pub const VALUE_Y: f64 = 0.1;
pub const SPLIT_UNIT_Y: f64 = -0.6;
/// The white box round the riding-along column (03_8, sentence 4).
pub const SPLIT_BOX: (f64, f64, f64, f64) = (3.9, 6.5, -0.95, 1.25);
/// The prediction card (03_5 to 03_6).
pub const CARD_AT: (f64, f64) = (3.65, -0.85);
pub const CARD_SIZE: (f64, f64) = (5.6, 3.0);
pub const QUESTION: [&str; 2] = ["How fast is the parcel", "speeding up as it enters?"];
pub const QUESTION_DY: [f64; 2] = [1.05, 0.6];
pub const CHOICES: [&str; 3] = ["0", "1", "2"];
pub const CHOICE_DX: f64 = 1.4;
pub const CHOICE_DY: f64 = -0.15;
pub const CARD_UNIT: &str = "m/s per second";
pub const CARD_UNIT_DY: f64 = -0.6;
/// The answer, "2" (03_6).
pub const ANSWER: usize = 2;
pub const PAUSE_NOTE: &str = "pause for more time";
pub const RING_R: f64 = 0.36;
/// 03_h1's card as one spec ([`PredictCard::build`]).
pub const CARD03: CardSpec = CardSpec {
    at: CARD_AT,
    size: CARD_SIZE,
    text_pt: CARD_PT,
    question: &QUESTION,
    question_dy: &QUESTION_DY,
    choices: &CHOICES,
    choice_dx: CHOICE_DX,
    choice_dy: CHOICE_DY,
    unit: Some((CARD_UNIT, CARD_UNIT_DY)),
    answer: ANSWER,
    answer_box: (0.75, 0.62),
    ring_lift: 0.55,
};
/// 03_9: the two parts in words, side by side, a plus between them; centred
/// columns, lines `PART_STEP` apart from `PART_TOP` down.
pub const PART_X: [f64; 2] = [1.95, 5.25];
pub const PART_NOW: [&str; 2] = ["the flow changing", "where you are"];
pub const PART_MOVING: [&str; 3] = ["your speed, times", "how fast the flow", "changes along your way"];
pub const PART_TOP: f64 = 1.45;
pub const PART_STEP: f64 = 0.4;
pub const PART_PLUS_X: f64 = 3.45;
pub const PART_PT: f64 = 26.0;
/// 03_10: the material derivative in one dimension, then in vector form,
/// each centred at its point. The moving part of each is underlined gold
/// `UNDERLINE_GAP` under its lowest glyph.
pub const FORMS: [&str; 2] = [
    r"Du/Dt=\partial u/\partial t+u\,\partial u/\partial x",
    r"Du/Dt=\partial u/\partial t+(u\cdot\nabla)u",
];
pub const MOVING: [&str; 2] = [r"u\,\partial u/\partial x", r"(u\cdot\nabla)u"];
pub const FORM_AT: [(f64, f64); 2] = [(3.4, -0.2), (3.4, -1.1)];
pub const UNDERLINE_GAP: f64 = 0.08;
pub const UNDERLINE_W: f64 = 4.0;
/// 03_11: "advection" with its top `ADVECTION_GAP` under the vector form's
/// underline, centred on it; "force", struck out red, `NOT_FORCE_GAP` to its
/// right.
pub const ADVECTION: &str = "advection";
pub const ADVECTION_GAP: f64 = 0.22;
pub const NOT_FORCE: &str = "force";
pub const NOT_FORCE_GAP: f64 = 0.3;
pub const STRIKE_W: f64 = 3.0;

pub const TITLE_PT: f64 = 42.0;
pub const READOUT_PT: f64 = 34.0;
pub const CARD_PT: f64 = 34.0;
pub const FORMULA_PT: f64 = 44.0;
pub const VALUE_PT: f64 = 52.0;
pub const DIM_PT: f64 = 30.0;
pub const GIVEN_PT: f64 = 30.0;
pub const HEAD_PT: f64 = 30.0;
pub const LABEL_PT: f64 = 28.0;
pub const FINE_PT: f64 = 24.0;

pub const WALL_W: f64 = 4.0;
const DIM_W: f64 = 2.5;
const PIN_W: f64 = 3.0;
const PIN_R: f64 = 0.1;
const PIN_DOT: f64 = 0.025;
const PIN_STEM: f64 = 0.28;
const PIN_BASE: f64 = 0.16;
pub const PARCEL_R: f64 = 0.09;
const TRAIL_W: f64 = 5.0;
pub const HIGHLIGHT_W: f64 = 3.0;

// Draw order: arrows under the walls under the marks, the trail, the parcel,
// text, and the teaching rings on top.
pub const Z_FIELD: i32 = 1;
pub const Z_WALLS: i32 = 2;
pub const Z_MARKS: i32 = 3;
pub const Z_TRAIL: i32 = 4;
pub const Z_PARCEL: i32 = 5;
pub const Z_TEXT: i32 = 6;
pub const Z_HIGHLIGHT: i32 = 7;

// Timing, seconds. Every value is exact in binary and a whole number of
// frames at 60 fps (fmn rounds a segment up on its exact f64); at 30 fps the
// 0.25 and 0.75 s steps round up a frame. The run and the countdown are frame
// loops, exact at any rate.
/// Film seconds for the 0.69 s crossing (03_7): slow motion.
pub const RUN_SECONDS: f64 = 4.0;
/// The 03_h1 prediction countdown.
pub const HOLD_SECONDS: f64 = 5.0;
pub const OPEN: f64 = 0.75;
pub const FADE: f64 = 0.5;
pub const QUICK: f64 = 0.25;
const ARROWS_IN: f64 = 0.75;
const WALLS_IN: f64 = 1.0;

/// The teaching highlight: a white outline.
pub fn white() -> Srgb {
    hex("#FFFFFF")
}

pub fn card_fill() -> Srgb {
    hex("#141C29")
}

// ------------------------------------------------------- words from facts

/// "2 m/s", "3 m/s", "4 m/s": the centre-line speed at `x`, from the field.
pub fn speed_label(x: f64) -> String {
    format!("{} m/s", speed(x, 0.0))
}

/// "4 m" at the entrance, "2 m" at the exit: the width between the walls.
pub fn width_label(x: f64) -> String {
    format!("{} m", 2.0 * wall(x))
}

/// "2 m": the length brace.
pub fn length_label() -> String {
    format!("{LENGTH} m")
}

/// "in 0.69 s": the crossing time at the exit.
pub fn crossing_label() -> String {
    format!("in {CROSS:.2} s")
}

/// "speed 2 m/s; +1 m/s per meter" (03_4).
pub fn given_line() -> String {
    format!("speed {} m/s; +{DUDX} m/s per meter", speed(0.0, 0.0))
}

/// "slow motion: 0.69 s of flow in 4 s of film" (03_7).
pub fn run_note() -> String {
    format!("slow motion: {CROSS:.2} s of flow in {RUN_SECONDS} s of film")
}

/// "0" at a fixed spot; "2, then 4" riding along (03_8).
pub fn split_value(side: usize) -> String {
    if side == 0 {
        format!("{DUDT_FIXED}")
    } else {
        format!("{}, then {}", parcel_accel(0.0), parcel_accel(1.0))
    }
}

// ----------------------------------------------------------------- helpers

pub fn v(x: f64, y: f64) -> Vec3 {
    [x, y, 0.0]
}

/// Channel metres (the entrance's centre at the origin) to scene units, at
/// the one scale.
pub fn to_scene(x: f64, y: f64) -> Vec3 {
    v(ORIGIN_AT.0 + SCALE * x, ORIGIN_AT.1 + SCALE * y)
}

/// The wall above (`sign` 1) or below (−1), entrance to exit.
pub fn wall_points(sign: f64) -> Vec<(f64, f64)> {
    (0..=WALL_SAMPLES)
        .map(|i| {
            let x = LENGTH * i as f64 / WALL_SAMPLES as f64;
            (x, sign * wall(x))
        })
        .collect()
}

/// The two walls, upper then lower, detached: the one drawing chapters 03 and
/// 04 share, so the join between them is exact.
pub fn build_walls(stage: &mut Stage<'_>) -> Vec<Mob> {
    [1.0, -1.0]
        .into_iter()
        .map(|sign| {
            let points: Vec<Vec3> = wall_points(sign).iter().map(|&(x, y)| to_scene(x, y)).collect();
            add(
                stage,
                polyline(points).style(line_style(Palette::ink(), WALL_W, 1.0)).build(),
                Z_WALLS,
            )
        })
        .collect()
}

/// The arrows' sample points, channel metres: top row first, entrance to
/// exit.
pub fn arrow_points() -> Vec<(f64, f64)> {
    let mut out = Vec::with_capacity(ARROW_ROWS.len() * ARROW_COLUMNS.len());
    for &share in &ARROW_ROWS {
        for &x in &ARROW_COLUMNS {
            out.push((x, share * wall(x)));
        }
    }
    out
}

/// The arrow centred on sample point `p`, along the flow there, its length
/// proportional to the speed.
pub fn arrow_at(p: (f64, f64)) -> Arrow {
    let (u, w) = velocity(p.0, p.1);
    let c = to_scene(p.0, p.1);
    let h = ARROW_PER_SPEED / 2.0;
    Arrow::new(v(c[0] - h * u, c[1] - h * w), v(c[0] + h * u, c[1] + h * w))
        .buff(0.0)
        .thickness(ARROW_THICKNESS)
        .color(Palette::blue())
}

pub fn line_style(color: Srgb, width: f64, opacity: f64) -> Style {
    Style::default().fill(color, 0.0).stroke(color, width, opacity)
}

/// A detached mobject at draw layer `z`.
pub fn add(stage: &mut Stage<'_>, object: VMobject, z: i32) -> Mob {
    let m = stage.arena_mut().add(object);
    stage.set_z_index(m, z, true);
    m
}

pub fn segment(a: Vec3, b: Vec3, width: f64) -> VMobject {
    polyline(vec![a, b])
        .style(line_style(Palette::ink(), width, 1.0))
        .build()
}

pub fn ring(at: Vec3, r: f64, color: Srgb, width: f64) -> VMobject {
    Circle::new()
        .radius(r)
        .arc_center(at)
        .style(line_style(color, width, 1.0))
        .build()
}

/// What is left of a ring of radius `r` at `c`, drained clockwise from the
/// top (the corpus's countdown ring).
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

/// Detached text centred at `at`, then moved so its `edge` is at `at`.
pub fn text_at(
    kit: &Kit,
    stage: &mut Stage<'_>,
    s: &str,
    size: f64,
    color: Srgb,
    at: Vec3,
    edge: Vec3,
) -> fmn::Result<Mob> {
    let m = kit.text(stage, s, size, color, at)?;
    stage.move_to(m, at, edge);
    stage.set_z_index(m, Z_TEXT, true);
    Ok(m)
}

/// Ink text with its baseline at `base` and its `edge` (LEFT or RIGHT) at
/// `x`, so a label, a live number and a unit share one baseline whatever
/// their letters' depths. The baseline is read from an unshown copy with a
/// flat-bottomed "2" appended: the first glyph's height above that "2" is
/// where it sits above the baseline (as `inspection_paradox` aligns pieces to
/// an unshown whole).
pub fn baseline_text(
    kit: &Kit,
    stage: &mut Stage<'_>,
    s: &str,
    size: f64,
    x: f64,
    base: f64,
    edge: Vec3,
) -> fmn::Result<Mob> {
    let ink = Palette::ink();
    let probe = kit.text(stage, &format!("{s}2"), size, ink, ORIGIN)?;
    let glyphs = children(stage, probe);
    let lift = match (glyphs.first(), glyphs.last()) {
        (Some(&first), Some(&two)) if glyphs.len() >= 2 => stage.get_bottom(first)[1] - stage.get_bottom(two)[1],
        _ => 0.0,
    };
    let m = kit.text(stage, s, size, ink, ORIGIN)?;
    let head = children(stage, m).first().copied().unwrap_or(m);
    stage.move_to(m, v(x, base), edge);
    let dy = base + lift - stage.get_bottom(head)[1];
    stage.shift(m, v(0.0, dy));
    stage.set_z_index(m, Z_TEXT, true);
    Ok(m)
}

/// A scene error with `message`, as the crate's `main.rs` makes them.
pub fn fail(message: &str) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.to_owned().into_boxed_str(),
    )))
}

/// Left, right and bottom of the glyphs that typeset `term`, its first
/// occurrence in the source of Tex `m`, from that Tex's span map: entry `i` is
/// child `i`, and a glyph belongs to the term when its source span starts
/// inside the term's bytes (the engine's own rule for `OldTex` groups).
pub fn term_extent(
    stage: &Stage<'_>,
    m: Mob,
    spans: &fmn::library::SpanMapData,
    term: &str,
) -> fmn::Result<(f64, f64, f64)> {
    let start = spans
        .source
        .find(term)
        .ok_or_else(|| fail("a term is not in its Tex"))?;
    let end = start + term.len();
    let glyphs = children(stage, m);
    if glyphs.len() != spans.entries.len() {
        return Err(fail("a Tex's children do not match its span map"));
    }
    let mut extent: Option<(f64, f64, f64)> = None;
    for (entry, &glyph) in spans.entries.iter().zip(&glyphs) {
        if (start..end).contains(&entry.start) {
            let (l, r, b) = (
                stage.get_left(glyph)[0],
                stage.get_right(glyph)[0],
                stage.get_bottom(glyph)[1],
            );
            extent = Some(extent.map_or((l, r, b), |(l0, r0, b0)| (l0.min(l), r0.max(r), b0.min(b))));
        }
    }
    extent.ok_or_else(|| fail("a term draws no glyph"))
}

/// The image records in `m`'s family, `m` included (chapter 04's pressure
/// plate). The engine's fades write only the alpha of colour fields, which a
/// raster quad does not have, so a plate gets its own fade beside them.
fn image_records(stage: &Stage<'_>, m: Mob) -> Vec<Mob> {
    stage
        .family(m)
        .into_iter()
        .filter(|&k| stage.get(k).is_some_and(|e| e.image_resource().is_some()))
        .collect()
}

/// A plate's own fade (production's `plate_fade`, candidate 264's draw.rs):
/// played in the same `play` as the engine fade of `owner`, listed after it,
/// with the same run time and rate, so it adds no frame and moves no cue.
/// `rising` goes 0 to 1; a fade out ends with the lane back at 1 as the
/// remover takes the plate off the scene.
fn plate_fade(owner: Mob, images: Vec<Mob>, rising: bool) -> fmn::animation::UpdateFromFunc {
    fmn::animation::UpdateFromFunc::new_alpha(owner, move |s: &mut fmn::mobject::Stage, _: Mob, a: f64| {
        let shown = (if rising { a } else { 1.0 - a }).clamp(0.0, 1.0);
        for &k in &images {
            s.set_scalar_lanes(k, "opacity", shown, false);
        }
    })
    .with_config(AnimConfig {
        name: "PlateFade".to_owned(),
        final_alpha_value: if rising { 1.0 } else { 0.0 },
        ..AnimConfig::default()
    })
}

/// Fade `out` away and `into` in, together, over `rt` seconds, image plates
/// included. Updaters on `out` are cleared first, so live readouts fade
/// instead of redrawing.
pub fn swap(stage: &mut Stage<'_>, out: &[Mob], into: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for (&m, rising) in out.iter().map(|m| (m, false)).chain(into.iter().map(|m| (m, true))) {
        if rising {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(rt))?);
        } else {
            stage.clear_updaters(m, true);
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(rt))?);
        }
        let images = image_records(stage, m);
        if !images.is_empty() {
            anims.push(stage.prepare(plate_fade(m, images, rising).rt(rt))?);
        }
    }
    if !anims.is_empty() {
        stage.play_prepared(anims)?;
    }
    Ok(())
}

pub fn fade_in_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    swap(stage, &[], mobs, rt)
}

pub fn fade_out_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    swap(stage, mobs, &[], rt)
}

/// Draw `mobs` along their paths together, over `rt` seconds.
pub fn create_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        anims.push(stage.prepare(show_creation(m).rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Frames of film for `seconds` at the scene's rate.
pub fn film_frames(stage: &Stage<'_>, seconds: f64) -> u64 {
    frames_covering(seconds, stage.scene().time().fps())
}

// ------------------------------------------------------------ the channel

/// Every object of the channel and its side panel, built once at 03_1 and
/// never rescaled. One tracker, [`Channel::run`], drives the parcel, its
/// trail and the 03_7 readouts.
pub struct Channel {
    /// The share of the crossing on show, 0 (entrance) to 1 (exit).
    pub run: Mob,
    pub title: Option<Mob>,
    pub walls: Vec<Mob>,
    /// The 03_2 width lines, ticks and labels, the brace and its label.
    pub dims: Vec<Mob>,
    pub arrows: Vec<Mob>,
    pub pin: Vec<Mob>,
    /// The centre-line speeds, entrance, pin, exit: 2, 3 and 4 m/s.
    pub labels: Vec<Mob>,
    pub parcel: Mob,
    pub trail: Mob,
    pub crossing: Mob,
    /// 03_3: "t =", "s", the gauge's label and unit; then the live clock and
    /// gauge once shown.
    pub gauge: Vec<Mob>,
    pub given: Mob,
    pub product: Vec<Mob>,
    /// 03_7: labels, units and the slow-motion note; then the live speed,
    /// acceleration and flow time once shown.
    pub readouts: Vec<Mob>,
    pub divider: Mob,
    pub heads: [Mob; 2],
    pub values: [Mob; 2],
    pub units: [Mob; 2],
    pub split_box: Mob,
    /// White rings round the pin and round the parcel at the exit.
    pub highlights: [Mob; 2],
    /// The parcel's updater anchor, taken off the scene when the chapter
    /// clears.
    mover: Mob,
}

impl Channel {
    /// Build everything detached, with the parcel's updater.
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, title: bool) -> fmn::Result<Self> {
        let run = stage.add_value_tracker(0.0);
        let ink = Palette::ink();
        let title = if title {
            Some(kit.text_left(stage, TITLE, TITLE_PT, ink, v(TITLE_AT.0, TITLE_AT.1))?)
        } else {
            None
        };

        let walls = build_walls(stage);

        let (entrance, exit) = (to_scene(0.0, 0.0)[0], to_scene(LENGTH, 0.0)[0]);
        let mut dims = Vec::new();
        for (x, at, label_x, edge) in [
            (0.0, entrance - DIM_GAP, entrance - DIM_GAP - DIM_LABEL_GAP, RIGHT),
            (LENGTH, exit + DIM_GAP, exit + DIM_GAP + DIM_LABEL_GAP, LEFT),
        ] {
            let (lo, hi) = (to_scene(x, -wall(x))[1], to_scene(x, wall(x))[1]);
            dims.push(add(stage, segment(v(at, lo), v(at, hi), DIM_W), Z_MARKS));
            for y in [lo, hi] {
                dims.push(add(
                    stage,
                    segment(v(at - DIM_TICK, y), v(at + DIM_TICK, y), DIM_W),
                    Z_MARKS,
                ));
            }
            dims.push(text_at(
                kit,
                stage,
                &width_label(x),
                DIM_PT,
                ink,
                v(label_x, 0.0),
                edge,
            )?);
        }
        let below = to_scene(0.0, -wall(0.0))[1] - BRACE_GAP;
        let span = polyline(vec![v(entrance, below), v(exit, below)]).build();
        let brace = Brace::around(&span, DOWN).buff(0.0).color(ink);
        let tip = brace.tip();
        dims.push(add(stage, brace.build(), Z_MARKS));
        dims.push(text_at(
            kit,
            stage,
            &length_label(),
            DIM_PT,
            ink,
            v(tip[0], tip[1] - BRACE_LABEL_GAP),
            UP,
        )?);

        let mut arrows = Vec::with_capacity(ARROW_ROWS.len() * ARROW_COLUMNS.len());
        for p in arrow_points() {
            arrows.push(add(stage, arrow_at(p).build()?, Z_FIELD));
        }

        let pc = to_scene(PIN.0, PIN.1);
        let foot = pc[1] - PIN_R - PIN_STEM;
        let pin = vec![
            add(stage, ring(pc, PIN_R, ink, PIN_W), Z_MARKS),
            add(stage, Dot::new().point(pc).radius(PIN_DOT).color(ink).build(), Z_MARKS),
            add(stage, segment(v(pc[0], pc[1] - PIN_R), v(pc[0], foot), PIN_W), Z_MARKS),
            add(
                stage,
                segment(v(pc[0] - PIN_BASE / 2.0, foot), v(pc[0] + PIN_BASE / 2.0, foot), PIN_W),
                Z_MARKS,
            ),
        ];

        let blue = Palette::blue();
        let labels = vec![
            text_at(
                kit,
                stage,
                &speed_label(0.0),
                LABEL_PT,
                blue,
                v(entrance - END_LABEL_GAP, 0.0),
                RIGHT,
            )?,
            text_at(
                kit,
                stage,
                &speed_label(PIN.0),
                LABEL_PT,
                blue,
                v(pc[0], pc[1] + MID_LABEL_LIFT),
                DOWN,
            )?,
            text_at(
                kit,
                stage,
                &speed_label(LENGTH),
                LABEL_PT,
                blue,
                v(exit + END_LABEL_GAP, 0.0),
                LEFT,
            )?,
        ];

        let gold = Palette::gold();
        let start = to_scene(0.0, 0.0);
        let parcel = add(
            stage,
            Dot::new().point(start).radius(PARCEL_R).color(gold).build(),
            Z_PARCEL,
        );
        let trail = add(
            stage,
            polyline(vec![start, to_scene(LENGTH, 0.0)])
                .style(line_style(gold, TRAIL_W, 0.0))
                .build(),
            Z_TRAIL,
        );
        let crossing = text_at(
            kit,
            stage,
            &crossing_label(),
            LABEL_PT,
            gold,
            v(exit + END_LABEL_GAP, CROSSING_Y),
            LEFT,
        )?;

        let gauge = vec![
            baseline_text(kit, stage, "t =", READOUT_PT, LABEL_RIGHT, CLOCK_BASE, RIGHT)?,
            baseline_text(kit, stage, "s", READOUT_PT, UNIT_X, CLOCK_BASE, LEFT)?,
            baseline_text(kit, stage, GAUGE_LABEL, READOUT_PT, LABEL_RIGHT, GAUGE_BASE, RIGHT)?,
            baseline_text(kit, stage, "m/s", READOUT_PT, UNIT_X, GAUGE_BASE, LEFT)?,
        ];
        let given = kit.text_left(stage, &given_line(), GIVEN_PT, ink, v(GIVEN_AT.0, GIVEN_AT.1))?;
        stage.set_z_index(given, Z_TEXT, true);
        let mut product = Vec::with_capacity(PRODUCT.len());
        for (src, at) in PRODUCT.iter().zip(PRODUCT_AT) {
            let m = kit.tex_left(stage, src, FORMULA_PT, ink, &[], v(at.0, at.1))?;
            stage.set_z_index(m, Z_TEXT, true);
            product.push(m);
        }
        let mut readouts = vec![
            baseline_text(kit, stage, "speed", READOUT_PT, LABEL_RIGHT, SPEED_BASE, RIGHT)?,
            baseline_text(kit, stage, "m/s", READOUT_PT, UNIT_X, SPEED_BASE, LEFT)?,
            baseline_text(kit, stage, ACCEL_LABEL, READOUT_PT, LABEL_RIGHT, ACCEL_BASE, RIGHT)?,
            baseline_text(kit, stage, ACCEL_UNIT, READOUT_PT, UNIT_X, ACCEL_BASE, LEFT)?,
            baseline_text(kit, stage, "t =", READOUT_PT, LABEL_RIGHT, RUN_CLOCK_BASE, RIGHT)?,
            baseline_text(kit, stage, "s", READOUT_PT, UNIT_X, RUN_CLOCK_BASE, LEFT)?,
        ];
        let note = kit.text_left(
            stage,
            &run_note(),
            FINE_PT,
            Palette::dim(),
            v(RUN_NOTE_AT.0, RUN_NOTE_AT.1),
        )?;
        stage.set_z_index(note, Z_TEXT, true);
        readouts.push(note);

        let divider = add(
            stage,
            polyline(vec![v(DIVIDER_X, SPLIT_LOW), v(DIVIDER_X, SPLIT_HIGH)])
                .style(line_style(Palette::faint(), 2.0, 1.0))
                .build(),
            Z_TEXT,
        );
        let heads = [
            text_at(kit, stage, SPLIT_HEADS[0], HEAD_PT, ink, v(SPLIT_X[0], HEAD_Y), ORIGIN)?,
            text_at(kit, stage, SPLIT_HEADS[1], HEAD_PT, ink, v(SPLIT_X[1], HEAD_Y), ORIGIN)?,
        ];
        let values = [
            text_at(
                kit,
                stage,
                &split_value(0),
                VALUE_PT,
                blue,
                v(SPLIT_X[0], VALUE_Y),
                ORIGIN,
            )?,
            text_at(
                kit,
                stage,
                &split_value(1),
                VALUE_PT,
                gold,
                v(SPLIT_X[1], VALUE_Y),
                ORIGIN,
            )?,
        ];
        let dim = Palette::dim();
        let units = [
            text_at(
                kit,
                stage,
                SPLIT_UNIT,
                FINE_PT,
                dim,
                v(SPLIT_X[0], SPLIT_UNIT_Y),
                ORIGIN,
            )?,
            text_at(
                kit,
                stage,
                SPLIT_UNIT,
                FINE_PT,
                dim,
                v(SPLIT_X[1], SPLIT_UNIT_Y),
                ORIGIN,
            )?,
        ];
        let (bx0, bx1, by0, by1) = SPLIT_BOX;
        let split_box = Rectangle::new()
            .width(bx1 - bx0)
            .height(by1 - by0)
            .corner_radius(0.12)
            .style(line_style(white(), HIGHLIGHT_W, 1.0))
            .build()?
            .moved_to(v((bx0 + bx1) / 2.0, (by0 + by1) / 2.0));
        let split_box = add(stage, split_box, Z_HIGHLIGHT);
        let highlights = [
            add(stage, ring(pc, HIGHLIGHT_R, white(), HIGHLIGHT_W), Z_HIGHLIGHT),
            add(
                stage,
                ring(to_scene(LENGTH, 0.0), HIGHLIGHT_R, white(), HIGHLIGHT_W),
                Z_HIGHLIGHT,
            ),
        ];

        // One updater: the parcel and its trail follow the exact path
        // whenever the share of the crossing moves. It stays at 0 (the
        // parcel at rest at the entrance) until 03_7.
        let mover = stage.arena_mut().add(VMobject::new());
        let mut last = f64::NAN;
        stage.add_updater(
            mover,
            move |s, _| {
                let share = s.tracker_value(run).unwrap_or(0.0);
                if share == last {
                    return;
                }
                let at = to_scene(parcel_at(share), 0.0);
                s.move_to(parcel, at, ORIGIN);
                let drawn = share > 0.0;
                if drawn {
                    let _ = s.set_points(trail, polyline(vec![start, at]).build().points());
                }
                s.set_stroke(trail, None, None, Some(if drawn { 1.0 } else { 0.0 }), None, false);
                last = share;
            },
            true,
        )?;
        stage.add_to_scene(mover)?;

        Ok(Self {
            run,
            title,
            walls,
            dims,
            arrows,
            pin,
            labels,
            parcel,
            trail,
            crossing,
            gauge,
            given,
            product,
            readouts,
            divider,
            heads,
            values,
            units,
            split_box,
            highlights,
            mover,
        })
    }

    /// 03_3: the pin fades in; its gauge and a clock pop in beside it. The
    /// clock counts real seconds from now (scene time); the gauge reads the
    /// field's speed at the pin on every frame. The field has no time in it,
    /// so the gauge cannot change while the clock runs.
    fn gauge_in(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        fade_in_all(stage, &self.pin, QUICK)?;
        let started = Cell::new(f64::NAN);
        let clock = kit.live_value(
            stage,
            move |s| {
                if started.get().is_nan() {
                    started.set(s.time());
                }
                s.time() - started.get()
            },
            1,
            false,
            READOUT_PT,
            Palette::ink(),
            v(NUM_X, CLOCK_BASE),
            DL,
        )?;
        let gauge = kit.live_value(
            stage,
            |_| speed(PIN.0, PIN.1),
            1,
            false,
            READOUT_PT,
            Palette::blue(),
            v(NUM_X, GAUGE_BASE),
            DL,
        )?;
        for &m in &self.gauge {
            stage.add_to_scene(m)?;
        }
        self.gauge.extend([clock, gauge]);
        Ok(())
    }

    /// 03_7: the parcel's speed (blue), its acceleration (gold) and the flow
    /// time, live from the share of the crossing, with the slow-motion note.
    fn readouts_in(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let run = self.run;
        let share = move |s: &fmn::mobject::Stage| s.tracker_value(run).unwrap_or(0.0);
        let speed_now = kit.live_value(
            stage,
            move |s| parcel_speed(share(s)),
            2,
            false,
            READOUT_PT,
            Palette::blue(),
            v(NUM_X, SPEED_BASE),
            DL,
        )?;
        let accel_now = kit.live_value(
            stage,
            move |s| parcel_accel(share(s)),
            2,
            false,
            READOUT_PT,
            Palette::gold(),
            v(NUM_X, ACCEL_BASE),
            DL,
        )?;
        let clock_now = kit.live_value(
            stage,
            move |s| CROSS * share(s),
            2,
            false,
            READOUT_PT,
            Palette::ink(),
            v(NUM_X, RUN_CLOCK_BASE),
            DL,
        )?;
        for &m in &self.readouts {
            stage.add_to_scene(m)?;
        }
        self.readouts.extend([speed_now, accel_now, clock_now]);
        Ok(())
    }

    /// The crossing, 0.69 s of flow in [`RUN_SECONDS`] of film, set frame by
    /// frame from [`run_share`]: the last frame is exactly the exit.
    pub fn run_through(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let n = film_frames(stage, RUN_SECONDS);
        for k in 1..=n {
            stage.set_tracker_value(self.run, run_share(k, n))?;
            wait_frames(stage, 1)?;
        }
        Ok(())
    }

    /// The split panel and its rings go (03_9).
    pub fn clear_panel(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        let mut panel = vec![self.divider, self.split_box];
        panel.extend(self.heads);
        panel.extend(self.values);
        panel.extend(self.units);
        panel.extend(self.highlights);
        let shown: Vec<Mob> = {
            let on = stage.scene().mobjects();
            panel.into_iter().filter(|m| on.contains(m)).collect()
        };
        fade_out_all(stage, &shown, rt)
    }

    /// Everything of the chapter's but the walls.
    fn all_but_walls(&self) -> Vec<Mob> {
        let mut all = vec![
            self.parcel,
            self.trail,
            self.crossing,
            self.given,
            self.divider,
            self.split_box,
        ];
        all.extend(self.title);
        for group in [
            &self.dims,
            &self.arrows,
            &self.pin,
            &self.labels,
            &self.gauge,
            &self.product,
            &self.readouts,
        ] {
            all.extend(group);
        }
        all.extend(self.heads);
        all.extend(self.values);
        all.extend(self.units);
        all.extend(self.highlights);
        all
    }

    /// Whatever of `mobs` is on screen fades out.
    fn fade_shown(stage: &mut Stage<'_>, mobs: Vec<Mob>, rt: f64) -> fmn::Result<()> {
        let shown: Vec<Mob> = {
            let on = stage.scene().mobjects();
            mobs.into_iter().filter(|m| on.contains(m)).collect()
        };
        fade_out_all(stage, &shown, rt)
    }

    /// The chapter's close: everything goes but the walls (and with it
    /// `extra`, the side panel's 03_9 to 03_11 objects), the picture chapter
    /// 04 opens on.
    pub fn leave_to_walls(&self, stage: &mut Stage<'_>, extra: &[Mob], rt: f64) -> fmn::Result<()> {
        stage.remove_from_scene(self.mover);
        let mut all = self.all_but_walls();
        all.extend(extra);
        Self::fade_shown(stage, all, rt)
    }

    /// Everything of the chapter's, walls too, for an exit root may prefer.
    #[allow(dead_code)]
    pub fn leave(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        stage.remove_from_scene(self.mover);
        let mut all = self.all_but_walls();
        all.extend(&self.walls);
        Self::fade_shown(stage, all, rt)
    }
}

// ------------------------------------------------- 03_9 to 03_11: Du/Dt

/// The material derivative in the side panel: its two parts in words, the
/// one-dimensional and vector forms with their moving parts underlined gold,
/// "advection" under the vector form's, and a struck-out "force" beside it.
pub struct MaterialPanel {
    /// The words for "changing where you are", then for "your speed, times
    /// how fast the flow changes along your way".
    pub parts: [Vec<Mob>; 2],
    pub plus: Mob,
    /// Du/Dt = ∂u/∂t + u ∂u/∂x; Du/Dt = ∂u/∂t + (u·∇)u.
    pub forms: [Mob; 2],
    pub underlines: [Mob; 2],
    pub advection: Mob,
    /// "force" and the red line through it.
    pub not_force: [Mob; 2],
}

impl MaterialPanel {
    /// Build everything detached, placed from the typeset forms' own glyphs.
    pub fn build(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let ink = Palette::ink();
        let gold = Palette::gold();
        let mut parts = [Vec::new(), Vec::new()];
        for (k, lines) in [&PART_NOW[..], &PART_MOVING[..]].into_iter().enumerate() {
            for (i, line) in lines.iter().enumerate() {
                let at = v(PART_X[k], PART_TOP - PART_STEP * i as f64);
                parts[k].push(text_at(kit, stage, line, PART_PT, ink, at, ORIGIN)?);
            }
        }
        let plus = text_at(
            kit,
            stage,
            "+",
            HEAD_PT,
            ink,
            v(PART_PLUS_X, PART_TOP - PART_STEP),
            ORIGIN,
        )?;

        let mut forms = Vec::with_capacity(FORMS.len());
        let mut underlines = Vec::with_capacity(FORMS.len());
        let mut extents = Vec::with_capacity(FORMS.len());
        for (src, (term, at)) in FORMS.iter().zip(MOVING.iter().zip(FORM_AT)) {
            let (m, spans) = kit.tex_spans(stage, src, FORMULA_PT, ink, &[], v(at.0, at.1))?;
            stage.set_z_index(m, Z_TEXT, true);
            let (x0, x1, bottom) = term_extent(stage, m, &spans, term)?;
            let y = bottom - UNDERLINE_GAP;
            underlines.push(add(
                stage,
                polyline(vec![v(x0, y), v(x1, y)])
                    .style(line_style(gold, UNDERLINE_W, 1.0))
                    .build(),
                Z_TEXT,
            ));
            forms.push(m);
            extents.push((x0, x1, bottom));
        }
        let (x0, x1, bottom) = extents[1];
        let advection = text_at(
            kit,
            stage,
            ADVECTION,
            LABEL_PT,
            gold,
            v((x0 + x1) / 2.0, bottom - UNDERLINE_GAP - ADVECTION_GAP),
            UP,
        )?;
        let middle = stage.get_center(advection)[1];
        let word = text_at(
            kit,
            stage,
            NOT_FORCE,
            LABEL_PT,
            Palette::dim(),
            v(stage.get_right(advection)[0] + NOT_FORCE_GAP, middle),
            LEFT,
        )?;
        let (l, r) = (stage.get_left(word)[0], stage.get_right(word)[0]);
        let strike = add(
            stage,
            polyline(vec![v(l - 0.05, middle), v(r + 0.05, middle)])
                .style(line_style(Palette::red(), STRIKE_W, 1.0))
                .build(),
            Z_HIGHLIGHT,
        );
        Ok(Self {
            parts,
            plus,
            forms: [forms[0], forms[1]],
            underlines: [underlines[0], underlines[1]],
            advection,
            not_force: [word, strike],
        })
    }

    /// Every object of the panel's.
    pub fn all(&self) -> Vec<Mob> {
        let mut all: Vec<Mob> = self.parts.iter().flatten().copied().collect();
        all.push(self.plus);
        all.extend(self.forms);
        all.extend(self.underlines);
        all.push(self.advection);
        all.extend(self.not_force);
        all
    }
}

/// Root's tunnel half of 03_12: the thumbnail grows briefly and the gap
/// beside the cylinder is outlined white, with no simulated parcels. The
/// hook runs right after the line starts; the channel and the side panel stay
/// as they are while it runs.
pub trait TunnelGap {
    fn gap_beside_cylinder(&mut self, _kit: &Kit, _stage: &mut Stage<'_>) -> fmn::Result<()> {
        Ok(())
    }
}

/// No tunnel: the channel alone (previews and tests).
#[cfg(test)]
pub struct NoTunnel;

#[cfg(test)]
impl TunnelGap for NoTunnel {}

// ------------------------------------------------------ the prediction card

/// A prediction card's words and layout: its centre and size, text size,
/// question lines (offsets from the centre), choices (centred on the card,
/// `choice_dx` apart), an optional fine-print unit under them, the answer's
/// index and its green box, and the countdown ring's height above the card's
/// bottom.
#[derive(Clone, Copy, Debug)]
pub struct CardSpec {
    pub at: (f64, f64),
    pub size: (f64, f64),
    pub text_pt: f64,
    pub question: &'static [&'static str],
    pub question_dy: &'static [f64],
    pub choices: &'static [&'static str],
    pub choice_dx: f64,
    pub choice_dy: f64,
    pub unit: Option<(&'static str, f64)>,
    pub answer: usize,
    pub answer_box: (f64, f64),
    pub ring_lift: f64,
}

impl CardSpec {
    /// The centre of choice `i`.
    pub fn choice_at(&self, i: usize) -> (f64, f64) {
        let middle = (self.choices.len() as f64 - 1.0) / 2.0;
        (
            self.at.0 + self.choice_dx * (i as f64 - middle),
            self.at.1 + self.choice_dy,
        )
    }

    /// The centre of the countdown ring, lower right.
    pub fn ring_at(&self) -> (f64, f64) {
        (
            self.at.0 + self.size.0 / 2.0 - 0.75,
            self.at.1 - self.size.1 / 2.0 + self.ring_lift,
        )
    }
}

/// A question with its choices (03_h1: "How fast is the parcel speeding up
/// as it enters?" with 0, 1 and 2 m/s per second), "pause for more time" and
/// a ring that drains over [`HOLD_SECONDS`] with the seconds left in its
/// middle.
pub struct PredictCard {
    pub back: Mob,
    pub question: Vec<Mob>,
    pub choices: Vec<Mob>,
    pub unit: Option<Mob>,
    pub note: Mob,
    pub track: Mob,
    pub ring: Mob,
    pub digits: Vec<Mob>,
    pub answer: Mob,
    answer_at: usize,
    left: Mob,
    anchor: Mob,
}

impl PredictCard {
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, spec: &CardSpec) -> fmn::Result<Self> {
        let (cx, cy) = spec.at;
        let (w, h) = spec.size;
        let ink = Palette::ink();
        let back = Rectangle::new()
            .width(w)
            .height(h)
            .corner_radius(0.18)
            .style(
                Style::default()
                    .fill(card_fill(), 0.96)
                    .stroke(Palette::dim(), 2.5, 1.0),
            )
            .build()?
            .moved_to(v(cx, cy));
        let back = add(stage, back, Z_TEXT);
        let mut question = Vec::with_capacity(spec.question.len());
        for (line, dy) in spec.question.iter().zip(spec.question_dy) {
            question.push(text_at(kit, stage, line, spec.text_pt, ink, v(cx, cy + dy), ORIGIN)?);
        }
        let mut choices = Vec::with_capacity(spec.choices.len());
        for (i, choice) in spec.choices.iter().enumerate() {
            let (x, y) = spec.choice_at(i);
            choices.push(text_at(kit, stage, choice, spec.text_pt, ink, v(x, y), ORIGIN)?);
        }
        let unit = match spec.unit {
            Some((words, dy)) => Some(text_at(
                kit,
                stage,
                words,
                FINE_PT,
                Palette::dim(),
                v(cx, cy + dy),
                ORIGIN,
            )?),
            None => None,
        };
        let ring_at = spec.ring_at();
        let note = kit.text_left(
            stage,
            PAUSE_NOTE,
            FINE_PT,
            Palette::dim(),
            v(cx - w / 2.0 + 0.35, ring_at.1),
        )?;
        stage.set_z_index(note, Z_TEXT, true);
        let track = add(
            stage,
            ring(v(ring_at.0, ring_at.1), RING_R, Palette::faint(), 6.0),
            Z_TEXT,
        );
        let drain = add(
            stage,
            polyline(ring_points(ring_at, RING_R, 1.0))
                .style(line_style(ink, 7.0, 1.0))
                .build(),
            Z_TEXT,
        );
        let seconds = HOLD_SECONDS as usize;
        let mut digits = Vec::with_capacity(seconds);
        for second in 1..=seconds {
            let d = text_at(
                kit,
                stage,
                &second.to_string(),
                CARD_PT,
                ink,
                v(ring_at.0, ring_at.1),
                ORIGIN,
            )?;
            stage.set_opacity(d, 0.0, true);
            digits.push(d);
        }
        let (ax, ay) = spec.choice_at(spec.answer);
        let answer = Rectangle::new()
            .width(spec.answer_box.0)
            .height(spec.answer_box.1)
            .corner_radius(0.1)
            .style(line_style(Palette::green(), 3.0, 1.0))
            .build()?
            .moved_to(v(ax, ay));
        let answer = add(stage, answer, Z_TEXT);

        let left = stage.add_value_tracker(HOLD_SECONDS);
        let anchor = stage.arena_mut().add(VMobject::new());
        let shown = digits.clone();
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s, _| {
                let l = s.tracker_value(left).unwrap_or(HOLD_SECONDS);
                if l == last {
                    return;
                }
                let _ = s.set_points(
                    drain,
                    polyline(ring_points(ring_at, RING_R, l / HOLD_SECONDS))
                        .build()
                        .points(),
                );
                let current = l.ceil() as usize;
                for (k, &d) in shown.iter().enumerate() {
                    s.set_opacity(d, if l > 0.0 && current == k + 1 { 1.0 } else { 0.0 }, true);
                }
                last = l;
            },
            true,
        )?;
        Ok(Self {
            back,
            question,
            choices,
            unit,
            note,
            track,
            ring: drain,
            digits,
            answer,
            answer_at: spec.answer,
            left,
            anchor,
        })
    }

    /// The card, its question, choices and unit, the note and the ring's
    /// track.
    pub fn show(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let mut shown = vec![self.back];
        shown.extend(&self.question);
        shown.extend(&self.choices);
        shown.extend(self.unit);
        shown.extend([self.note, self.track]);
        fade_in_all(stage, &shown, FADE)
    }

    /// The countdown, frame by frame: "5" for the first second, "1" for the
    /// last, nothing at zero; then the ring, its track and the note go.
    pub fn count(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add_to_scene(self.anchor)?;
        stage.add_to_scene(self.ring)?;
        for &d in &self.digits {
            stage.add_to_scene(d)?;
        }
        let n = film_frames(stage, HOLD_SECONDS);
        for k in 1..=n {
            stage.set_tracker_value(self.left, HOLD_SECONDS * (n - k) as f64 / n as f64)?;
            wait_frames(stage, 1)?;
        }
        stage.remove_from_scene(self.anchor);
        for &d in &self.digits {
            stage.remove_from_scene(d);
        }
        fade_out_all(stage, &[self.ring, self.track, self.note], QUICK)
    }

    /// The reveal ("Two.", "Neither."): the answer is marked green.
    pub fn answer(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.set_color(self.choices[self.answer_at], Palette::green(), None, true);
        fade_in_all(stage, &[self.answer], QUICK)
    }

    pub fn leave(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let mut out = vec![self.back, self.answer];
        out.extend(self.unit);
        out.extend(&self.question);
        out.extend(&self.choices);
        fade_out_all(stage, &out, FADE)
    }
}

// --------------------------------------------------------------- the cues

/// 03_1 "Take one small bit of air, a parcel, and follow it.": a gold dot
/// appears at the spot that will be the entrance's centre (and the title, if
/// asked for).
pub fn cue_03_1(kit: &Kit, stage: &mut Stage<'_>, title: bool) -> fmn::Result<Channel> {
    kit.say(stage, "03_1")?;
    let channel = Channel::build(kit, stage, title)?;
    let mut opening = vec![channel.parcel];
    opening.extend(channel.title);
    fade_in_all(stage, &opening, OPEN)?;
    Ok(channel)
}

/// 03_2 "…Air moves through a channel that narrows…": with sentence 1 the
/// walls draw around the dot, then the widths 4 m and 2 m and the 2 m brace.
pub fn cue_03_2(kit: &Kit, stage: &mut Stage<'_>, channel: &Channel) -> fmn::Result<()> {
    kit.say(stage, "03_2")?;
    kit.until_sentence(stage, "03_2", 1)?;
    create_all(stage, &channel.walls, WALLS_IN)?;
    fade_in_all(stage, &channel.dims, FADE)
}

/// 03_3 "This flow never changes. …": the marks and the resting dot go as
/// the blue arrows fill the channel; with sentence 1 the pin, its gauge
/// (3.0 m/s) and a running clock; with sentence 2 the centre-line speeds,
/// entrance to exit.
pub fn cue_03_3(kit: &Kit, stage: &mut Stage<'_>, channel: &mut Channel) -> fmn::Result<()> {
    kit.say(stage, "03_3")?;
    let mut out = channel.dims.clone();
    out.push(channel.parcel);
    swap(stage, &out, &channel.arrows, ARROWS_IN)?;
    kit.until_sentence(stage, "03_3", 1)?;
    channel.gauge_in(kit, stage)?;
    kit.until_sentence(stage, "03_3", 2)?;
    for &label in &channel.labels {
        fade_in_all(stage, &[label], FADE)?;
    }
    Ok(())
}

/// 03_4 "Release a parcel at the entrance, on the center line. …": the clock
/// and gauge go and the parcel waits at the entrance; with sentence 1 the
/// givens line.
pub fn cue_03_4(kit: &Kit, stage: &mut Stage<'_>, channel: &Channel) -> fmn::Result<()> {
    kit.say(stage, "03_4")?;
    swap(stage, &channel.gauge, &[channel.parcel], FADE)?;
    kit.until_sentence(stage, "03_4", 1)?;
    fade_in_all(stage, &[channel.given], FADE)
}

/// 03_5 "…So how fast is the parcel speeding up as it enters? …": the card
/// comes with sentence 1. The parcel stays at rest.
pub fn cue_03_5(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<PredictCard> {
    kit.say(stage, "03_5")?;
    let card = PredictCard::build(kit, stage, &CARD03)?;
    kit.until_sentence(stage, "03_5", 1)?;
    card.show(stage)?;
    Ok(card)
}

/// HOLD 03_h1: the question finishes, the moment is marked for the score,
/// and the 5 s countdown runs. No acceleration readout, no product and no
/// parcel motion.
pub fn hold_03_h1(kit: &Kit, stage: &mut Stage<'_>, card: &PredictCard) -> fmn::Result<()> {
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    card.count(stage)
}

/// 03_6 "Two. …": the answer; the card goes with sentence 1; "2 m/s × 1 per
/// s" with sentence 2 ("Two times one."); "= 2 m/s²" with sentence 3.
pub fn cue_03_6(kit: &Kit, stage: &mut Stage<'_>, channel: &Channel, card: &PredictCard) -> fmn::Result<()> {
    kit.say(stage, "03_6")?;
    card.answer(stage)?;
    kit.until_sentence(stage, "03_6", 1)?;
    card.leave(stage)?;
    kit.until_sentence(stage, "03_6", 2)?;
    fade_in_all(stage, &channel.product[..1], FADE)?;
    kit.until_sentence(stage, "03_6", 3)?;
    fade_in_all(stage, &channel.product[1..], FADE)
}

/// 03_7 "At the exit it's moving at four, …": the readouts come and the
/// parcel runs through, its gold trail behind it, speed and acceleration
/// climbing from 2 to 4 together; with sentence 1 the crossing time at the
/// exit.
pub fn cue_03_7(kit: &Kit, stage: &mut Stage<'_>, channel: &mut Channel) -> fmn::Result<()> {
    kit.say(stage, "03_7")?;
    channel.readouts_in(kit, stage)?;
    stage.add_to_scene(channel.trail)?;
    channel.run_through(stage)?;
    kit.until_sentence(stage, "03_7", 1)?;
    fade_in_all(stage, &[channel.crossing], FADE)
}

/// 03_8 "So there are two different questions. …": the panel clears to a
/// divider. Sentence 1: "at a fixed spot", the pin ringed white. Sentence 2
/// ("Here, not at all."): 0. Sentence 3: "riding along", the parcel ringed
/// instead, then "2, then 4". Sentence 4 (Newton's law): the riding-along
/// column boxed white. The panel stays until 03_9's line starts.
pub fn cue_03_8(kit: &Kit, stage: &mut Stage<'_>, channel: &Channel) -> fmn::Result<()> {
    kit.say(stage, "03_8")?;
    let mut out = vec![channel.given];
    out.extend(&channel.product);
    out.extend(&channel.readouts);
    swap(stage, &out, &[channel.divider], FADE)?;
    kit.until_sentence(stage, "03_8", 1)?;
    fade_in_all(stage, &[channel.heads[0], channel.highlights[0]], FADE)?;
    kit.until_sentence(stage, "03_8", 2)?;
    fade_in_all(stage, &[channel.values[0], channel.units[0]], FADE)?;
    kit.until_sentence(stage, "03_8", 3)?;
    swap(
        stage,
        &[channel.highlights[0]],
        &[channel.heads[1], channel.highlights[1]],
        FADE,
    )?;
    fade_in_all(stage, &[channel.values[1], channel.units[1]], FADE)?;
    kit.until_sentence(stage, "03_8", 4)?;
    swap(stage, &[channel.highlights[1]], &[channel.split_box], FADE)
}

/// 03_9 "Riding along, the change has two parts. …": the split panel and its
/// box go. Sentence 1 ("The flow might be changing where you are."): the
/// first part in words. Sentence 2 ("And you're moving to where the flow is
/// different…"): a plus and the second part, side by side with the first.
pub fn cue_03_9(kit: &Kit, stage: &mut Stage<'_>, channel: &Channel) -> fmn::Result<MaterialPanel> {
    kit.say(stage, "03_9")?;
    let panel = MaterialPanel::build(kit, stage)?;
    channel.clear_panel(stage, FADE)?;
    kit.until_sentence(stage, "03_9", 1)?;
    fade_in_all(stage, &panel.parts[0], FADE)?;
    kit.until_sentence(stage, "03_9", 2)?;
    let mut second = vec![panel.plus];
    second.extend(&panel.parts[1]);
    fade_in_all(stage, &second, FADE)?;
    Ok(panel)
}

/// 03_10 "Together, they make the material derivative. …": Du/Dt = ∂u/∂t +
/// u ∂u/∂x under the words, its moving part underlined gold; with sentence 1
/// the vector form, Du/Dt = ∂u/∂t + (u·∇)u, its (u·∇)u underlined gold.
pub fn cue_03_10(kit: &Kit, stage: &mut Stage<'_>, panel: &MaterialPanel) -> fmn::Result<()> {
    kit.say(stage, "03_10")?;
    fade_in_all(stage, &panel.forms[..1], FADE)?;
    create_all(stage, &panel.underlines[..1], FADE)?;
    kit.until_sentence(stage, "03_10", 1)?;
    fade_in_all(stage, &panel.forms[1..], FADE)?;
    create_all(stage, &panel.underlines[1..], FADE)
}

/// 03_11 "That second part is called advection. …": "advection" under the
/// gold term. Sentence 1 ("It is not a force."): "force" beside it, struck
/// out. Sentence 2 ("Something is pushing this parcel faster…"): the parcel
/// at the exit ringed white.
pub fn cue_03_11(kit: &Kit, stage: &mut Stage<'_>, channel: &Channel, panel: &MaterialPanel) -> fmn::Result<()> {
    kit.say(stage, "03_11")?;
    fade_in_all(stage, &[panel.advection], FADE)?;
    kit.until_sentence(stage, "03_11", 1)?;
    fade_in_all(stage, &panel.not_force[..1], QUICK)?;
    create_all(stage, &panel.not_force[1..], QUICK)?;
    kit.until_sentence(stage, "03_11", 2)?;
    fade_in_all(stage, &[channel.highlights[1]], FADE)
}

/// 03_12 "In our tunnel, the gap beside the cylinder narrows too. …": root's
/// tunnel picture runs while the channel and panel stay. After the line and
/// its breath, everything but the walls goes: the picture chapter 04 opens
/// on.
pub fn cue_03_12(
    kit: &Kit,
    stage: &mut Stage<'_>,
    channel: &Channel,
    panel: &MaterialPanel,
    tunnel: &mut dyn TunnelGap,
) -> fmn::Result<()> {
    kit.say(stage, "03_12")?;
    tunnel.gap_beside_cylinder(kit, stage)?;
    kit.hold(stage)?;
    channel.leave_to_walls(stage, &panel.all(), FADE)
}

/// Cues 03_1 to 03_12 and the 03_h1 hold, in order. Returns the channel, its
/// walls alone on screen.
pub fn chapter03(kit: &Kit, stage: &mut Stage<'_>, tunnel: &mut dyn TunnelGap, title: bool) -> fmn::Result<Channel> {
    let mut channel = cue_03_1(kit, stage, title)?;
    cue_03_2(kit, stage, &channel)?;
    cue_03_3(kit, stage, &mut channel)?;
    cue_03_4(kit, stage, &channel)?;
    let card = cue_03_5(kit, stage)?;
    hold_03_h1(kit, stage, &card)?;
    cue_03_6(kit, stage, &channel, &card)?;
    cue_03_7(kit, stage, &mut channel)?;
    cue_03_8(kit, stage, &channel)?;
    let panel = cue_03_9(kit, stage, &channel)?;
    cue_03_10(kit, stage, &panel)?;
    cue_03_11(kit, stage, &channel, &panel)?;
    cue_03_12(kit, stage, &channel, &panel, tunnel)?;
    Ok(channel)
}

/// The channel alone as a scene (no tunnel).
#[cfg(test)]
pub struct FollowParcelScene {
    pub kit: Kit,
    pub title: bool,
}

#[cfg(test)]
impl SceneConstruct for FollowParcelScene {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        chapter03(&self.kit, stage, &mut NoTunnel, self.title).map(|_| ())
    }
}

/// The spoken lines this scene says, verbatim from `navier-full-script03.md`
/// (`check_forces01.py` compares them byte for byte), as `script.tsv` rows.
/// 03_h1 is a hold, not a line.
#[cfg(test)]
pub const SCRIPT03: &str = concat!(
    "03_1\tTake one small bit of air, a parcel, and follow it.\n",
    "03_2\tHere's a simpler flow we can write down exactly. Air moves through a channel that narrows from four meters wide to two, over a length of two meters.\n",
    "03_3\tThis flow never changes. Stand at any point, and the arrow there stays the same forever. Down the middle, the speed is two meters per second at the entrance, three halfway along, and four at the exit.\n",
    "03_4\tRelease a parcel at the entrance, on the center line. It's moving at two meters per second, and for every meter it travels, the flow around it gets one meter per second faster.\n",
    "03_5\tNothing changes at any fixed point. So how fast is the parcel speeding up as it enters? Zero, one, or two meters per second, every second?\n",
    "03_6\tTwo. In a tiny moment, it moves a tiny distance at two meters per second, and each meter of that distance adds one meter per second of speed. Two times one. Its speed rises at two meters per second, every second.\n",
    "03_7\tAt the exit it's moving at four, through the same climb, so there it's speeding up at four. It crosses the whole channel in about seven tenths of a second.\n",
    "03_8\tSo there are two different questions. Standing still, how fast does the velocity change at this spot? Here, not at all. Riding along, how fast does your velocity change? That's the one Newton's law needs, because forces act on the parcel, not on a spot.\n",
    "03_9\tRiding along, the change has two parts. The flow might be changing where you are. And you're moving to where the flow is different, which counts as your speed, times how fast the flow changes along your way.\n",
    "03_10\tTogether, they make the material derivative. It's the change you'd feel riding with the fluid.\n",
    "03_11\tThat second part is called advection. It is not a force. Something is pushing this parcel faster, and we'll find it next. Advection is the bookkeeping that connects what we see at fixed points to what each parcel feels.\n",
    "03_12\tIn our tunnel, the gap beside the cylinder narrows too. Air passing through it changes speed, even if the flow there never changes at all.\n",
);

#[cfg(test)]
mod tests {
    use super::*;
    use explainer_kit::{Narrator, Script};

    const EXACT: f64 = 1e-12;

    /// The CH facts of facts01-run02.txt, from the module's own formulas.
    #[test]
    fn the_channel_gives_the_facts() {
        assert_eq!(2.0 * wall(0.0), 4.0, "ch.width.in");
        assert_eq!(2.0 * wall(LENGTH), 2.0, "ch.width.out");
        let walls = wall_points(1.0);
        assert_eq!((walls[0].0, walls[WALL_SAMPLES].0), (0.0, 2.0), "ch.length");
        assert_eq!(
            [speed(0.0, 0.0), speed(1.0, 0.0), speed(2.0, 0.0)],
            [2.0, 3.0, 4.0],
            "ch.speed.in/mid/out"
        );
        for x in [0.0, 0.5, 1.0] {
            assert_eq!(velocity(x + 1.0, 0.0).0 - velocity(x, 0.0).0, DUDX, "ch.dudx at {x}");
            // the flow across every slice is 8 m²/s per metre of depth
            assert!(
                (velocity(x, 0.0).0 * 2.0 * wall(x) - 8.0).abs() < EXACT,
                "ch.flux.slice at {x}"
            );
            // the walls are streamlines: the flow runs along them
            let (u, w) = velocity(x, wall(x));
            assert!((w / u + 4.0 / (2.0 + x).powi(2)).abs() < EXACT, "wall slope at {x}");
        }
        assert_eq!([parcel_accel(0.0), parcel_accel(1.0)], [2.0, 4.0], "ch.accel.in/out");
        assert_eq!(split_value(0), "0", "ch.dudt.fixed");
        assert_eq!(format!("{CROSS:.2}"), "0.69", "ch.time.cross");
    }

    /// The path solves the flow: dx/dt = u(x, 0) all the way, both ends exact.
    #[test]
    fn the_parcel_path_solves_the_flow() {
        assert_eq!(parcel_at(0.0), 0.0);
        assert_eq!(parcel_at(1.0), LENGTH);
        assert_eq!([parcel_speed(0.0), parcel_speed(1.0)], [2.0, 4.0]);
        for k in 0..=64 {
            let s = f64::from(k) / 64.0;
            let t = s * CROSS;
            assert!((parcel_x(t) - parcel_at(s)).abs() < EXACT, "the two forms at s {s}");
            assert!((parcel_speed(s) - velocity(parcel_at(s), 0.0).0).abs() < EXACT);
            let h = 1e-5;
            let rate = (parcel_x(t + h) - parcel_x(t - h)) / (2.0 * h);
            assert!((rate - velocity(parcel_x(t), 0.0).0).abs() < 1e-8, "dx/dt at t {t}");
            let climb = (parcel_speed((t + h) / CROSS) - parcel_speed((t - h) / CROSS)) / (2.0 * h);
            assert!((climb - parcel_accel(s)).abs() < 1e-8, "dv/dt at t {t}");
        }
    }

    /// The run's first and last frames show the entrance and exactly the exit
    /// at every rate.
    #[test]
    fn the_run_ends_exactly_at_the_exit() {
        for fps in [10_u32, 30, 60] {
            let n = frames_covering(RUN_SECONDS, fps);
            assert_eq!(n, u64::from(4 * fps));
            assert_eq!(run_share(n, n), 1.0);
            assert_eq!(parcel_at(run_share(n, n)), LENGTH, "{fps} fps");
            assert!(parcel_at(run_share(1, n)) > 0.0);
            assert_eq!(format!("{:.2}", CROSS * run_share(n, n)), "0.69");
        }
    }

    /// The words on screen come from the facts.
    #[test]
    fn the_labels_say_the_facts() {
        assert_eq!(
            [speed_label(0.0), speed_label(PIN.0), speed_label(LENGTH)],
            ["2 m/s", "3 m/s", "4 m/s"]
        );
        assert_eq!([width_label(0.0), width_label(LENGTH)], ["4 m", "2 m"]);
        assert_eq!(length_label(), "2 m");
        assert_eq!(crossing_label(), "in 0.69 s");
        assert_eq!(given_line(), "speed 2 m/s; +1 m/s per meter");
        assert_eq!(run_note(), "slow motion: 0.69 s of flow in 4 s of film");
        assert_eq!([split_value(0), split_value(1)], ["0", "2, then 4"]);
    }

    /// Every arrow stays inside the walls and clear of the centre line, and
    /// no two arrows come within 0.1 scene units of each other along their
    /// centre lines (the nearest pair is tip to tail along a row, 0.147).
    #[test]
    fn the_arrows_stay_inside_the_channel() {
        let points = arrow_points();
        let along = |p: (f64, f64)| -> Vec<(f64, f64)> {
            let (u, w) = velocity(p.0, p.1);
            let half = ARROW_PER_SPEED / 2.0 / SCALE;
            (0..=20)
                .map(|i| {
                    let t = f64::from(i) / 10.0 - 1.0;
                    (p.0 + t * half * u, p.1 + t * half * w)
                })
                .collect()
        };
        for &p in &points {
            for (ex, ey) in along(p) {
                assert!((0.0..=LENGTH).contains(&ex), "arrow at {p:?} leaves the ends");
                assert!(ey.abs() < wall(ex) - 0.1, "arrow at {p:?} reaches a wall");
                assert!(ey.abs() > 0.2, "arrow at {p:?} nears the centre line");
            }
        }
        for (i, &a) in points.iter().enumerate() {
            for &b in &points[i + 1..] {
                let near = along(a)
                    .iter()
                    .flat_map(|&p| along(b).into_iter().map(move |q| (p.0 - q.0).hypot(p.1 - q.1)))
                    .fold(f64::INFINITY, f64::min);
                assert!(near * SCALE > 0.1, "{a:?} and {b:?} come within {near} m");
            }
        }
    }

    /// Every sentence a cue waits for exists in its line.
    #[test]
    fn every_sentence_gate_exists() {
        let script = Script::parse(SCRIPT03).unwrap();
        let narrator = Narrator::estimate(&script);
        for (id, j) in [
            ("03_2", 1),
            ("03_3", 1),
            ("03_3", 2),
            ("03_4", 1),
            ("03_5", 1),
            ("03_6", 1),
            ("03_6", 2),
            ("03_6", 3),
            ("03_7", 1),
            ("03_8", 1),
            ("03_8", 2),
            ("03_8", 3),
            ("03_8", 4),
            ("03_9", 1),
            ("03_9", 2),
            ("03_10", 1),
            ("03_11", 1),
            ("03_11", 2),
        ] {
            assert!(narrator.sentence_starts(id).len() > j, "{id} sentence {j}");
        }
    }

    /// Each form holds its moving part once, and the card's answer is "2".
    #[test]
    fn the_forms_and_card_say_the_script() {
        for (form, moving) in FORMS.iter().zip(MOVING) {
            assert_eq!(form.matches(moving).count(), 1, "{moving} in {form}");
            assert!(form.ends_with(moving), "the moving part is the second term of {form}");
        }
        assert_eq!(CARD03.choices[CARD03.answer], "2");
        assert_eq!(CARD03.choice_at(1), (CARD_AT.0, CARD_AT.1 + CHOICE_DY));
        assert_eq!(CARD03.question.len(), CARD03.question_dy.len());
    }

    /// The whole chapter runs headless on the scratch voice, at 10 fps.
    #[test]
    fn the_chapter_runs_on_the_scratch_voice() {
        let script = Script::parse(SCRIPT03).unwrap();
        let kit = Kit::new().unwrap().with_narrator(Narrator::estimate(&script));
        let mut scene = FollowParcelScene { kit, title: true };
        let config = fmn::scene::RuntimeConfig {
            fps: 10,
            ..fmn::scene::RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut fmn::scene::NullSceneSink).expect("chapter 03 runs");
    }
}
