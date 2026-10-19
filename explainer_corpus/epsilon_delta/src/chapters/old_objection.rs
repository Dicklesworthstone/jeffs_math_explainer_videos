//! Chapter 03, what the old objection was about: chapter 02's last picture,
//! then George Berkeley's objection in The Analyst (1734, §§XIII–XIV) to a
//! step in an argument for the rate at which a power changes (03_1). The
//! step on our own modern example, the square at x = 1/2: move x by h, not
//! zero; the height changes by h + h²; divide by h and the chord's slope is
//! 1 + h (03_2). The old argument then lets h vanish and keeps 1; Berkeley's
//! words, verbatim, with their section, page and edition (03_3), kept up
//! through the prediction's question (03_4): at h = 0 the two points are
//! one and the quotient is 0/0, no slope at all (03_5). The answer 1 is
//! still right, reached with h never zero: the slopes at h = ±1/2, ±1/4,
//! ±1/10 close in on 1 around a hollow mark at h = 0 (03_6); every h with
//! 0 < |h| < 1/10 gives a slope within 1/10 of 1, and so for any tolerance,
//! with the exact statement on a card that stays to the end (03_7); this
//! never needs the questioned step, the order of choosing comes next, and
//! whether limits exist is a separate question (03_8).
//!
//! Every number drawn comes from the facts registry; every shape from the
//! model, which refuses the slope at h = 0 instead of evaluating it.

use explainer_kit::layout::TEXT_MARGIN;
use explainer_kit::props::precisely;
use explainer_kit::{BACKGROUND, Facts, Frame2, Kit, Palette, Timed, hex, rect, v};
use fmn::prelude::*;

use super::draw::{
    BRIGHT_W, END_R, NUMBER_DROP, PLOT, Plot, curve_color, curve_label, fact, fade_in_all, fade_in_lagged,
    fade_out_all, fail, fails, fixed, flat, limit_color, line_style, move_to, moving, say, slope_color, think_pause,
    wait_until,
};
use super::shrinking_region::{Resting, glow, resting};
use crate::facts::facts;
use crate::model::{BASE, Frac, NUDGES, SQUARE, TOLERANCES, chord_limit, chord_slope, clip_unit, power};

pub struct WhatTheOldObjectionWasAbout {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const AUTHOR: &str = "George Berkeley, The Analyst";
const SECTIONS: &str = "§§XIII–XIV, pp.";
const OF_A_POWER: &str = ": the fluxion of any power";
const MODERN: &str = r"\text{our modern example: } y = x";
const AT_X: &str = r"\text{ at } x = ";
/// Berkeley's sentence on the step (§XIII), verbatim in Wilkins's edition,
/// two phrases left out where the ellipses stand.
const QUOTE: [&str; 3] = [
    "“the former Supposition that the Increments",
    "were something … is destroyed, and yet a Consequence",
    "of that Supposition … is retained.”",
];
const QUOTE_FROM: &str = "Berkeley, The Analyst, §XIII, p.";
const EDITION: [&str; 2] = ["(Wilkins's", "edition)"];
const NO_SLOPE: &str = "every line through the point fits: no slope";
const SLOPE_OF: &str = "slope of the chord, against h";
const TOLERANCE: &str = "tolerance";
const NEXT: &str = "next: who picks the tolerance, and when?";

// what the pictures wait for, each in the line the test below holds it to
const BERKELEY_AT: &str = "George Berkeley";
const SQUARE_AT: &str = "for a square";
const MOVE_AT: &str = "Move x from one half";
const HEIGHT_AT: &str = "The height changes";
const DIVIDE_AT: &str = "Divide by h";
const VANISH_AT: &str = "lets h vanish";
const OBJECTED_AT: &str = "Berkeley objected";
const ZERO_AT: &str = "zero over zero";
const SHRINK_AT: &str = "Shrink h";
const CLOSE_AT: &str = "close in on one";
const WINDOW_AT: &str = "h within one tenth of zero";
const BAND_AT: &str = "the slope is within one tenth";
const ANY_AT: &str = "Any tolerance";
const LIMIT_AT: &str = "the limit is one";
const NEVER_AT: &str = "never needs";
const NEXT_AT: &str = "Who picks the tolerance";

// --- layout (scene units; the frame is 14.2 by 8; the plot is draw::PLOT)

/// The right column's left edge, and its rows: the author, the sections,
/// the modern example, the three steps, Berkeley's sentence and its page.
pub(super) const RIGHT_X: f64 = 0.6;
const AUTHOR_Y: f64 = 3.4;
const SECTIONS_Y: f64 = 2.95;
const MODERN_Y: f64 = 2.35;
const STEP_Y: [f64; 3] = [1.75, 0.85, -0.15];
const QUOTE_Y: [f64; 3] = [-0.75, -1.1, -1.45];
const QUOTE_FROM_Y: f64 = -1.85;
/// 03_5's 0/0 sits a little below the third step's row (it is a fraction),
/// its note under it.
const ZERO_Y: f64 = -0.25;
const NO_SLOPE_Y: f64 = -1.0;
/// The think ring, under the two steps that stay.
const RING_AT: Vec3 = [3.75, -0.75, 0.0];
const RING_R: f64 = 0.45;
/// The slope panel: h from −0.6 to 0.6 across, slopes from 0.25 to 1.75 up,
/// so the graph of 1 + h over |h| ≤ 1/2 stays inside it.
pub(super) const SLOPES: Plot = Plot {
    frame: Frame2 {
        center: [3.75, 0.95, 0.0],
        width: 4.8,
        height: 3.0,
        x: (-0.6, 0.6),
        y: (0.25, 1.75),
    },
};
/// Above the panel: the quotient's formula, then the panel's title. The
/// formula's top stays title-safe (draft06 measured it 0.21 from the top at
/// 3.45, about 0.34 above its centre).
const FORMULA_Y: f64 = 3.25;
const TITLE_Y: f64 = 2.62;
/// Under the panel: the readout (03_6) and the limit (03_7) on one row,
/// what comes next (03_8) under it.
pub(super) const READ_Y: f64 = -1.55;
pub(super) const NEXT_Y: f64 = -2.15;
/// The window's label sits inside the panel, this far over its foot, at
/// h = 0; the tolerance's label in the panel's empty top left (data units).
const WINDOW_LABEL_UP: f64 = 0.3;
const BAND_LABEL: (f64, f64) = (-0.57, 1.33);
/// The chord's stroke; how long the far point takes for each hop.
const CHORD_W: f64 = 4.0;
const HOP_S: f64 = 0.45;
/// 03_5's slide of the far point onto the base point: along the curve
/// through these increments, at an even speed (drawn, never printed).
const SLIDE: [f64; 5] = [0.4, 0.3, 0.2, 0.1, 0.0];
const SLIDE_S: f64 = 1.2;
/// The fan of 03_5: lines through the base point at these slopes, each one
/// as good as another when the quotient is 0/0 (drawn, never printed).
const FAN: [f64; 7] = [-0.5, 0.0, 0.5, 1.0, 1.5, 2.0, 3.0];
/// How long 03_5's 0/0, its fan and its note stay fully shown from the
/// moment the last of them is in, before they clear: past 03_5's end into
/// 03_6's first sentence, which says the answer is reached without h ever
/// being zero (draft07 showed them only ~1.3 s; review 4017). The note alone
/// wants its characters at [`READ_CPS`]; the 0/0 and the fan get the rest.
const ZERO_READ_S: f64 = 4.0;
/// Characters a second a viewer reads a short note at.
const READ_CPS: f64 = 15.0;

// the panel sits right of the plot and inside the frame; its formula (0.45
// either side of its centre, a fraction) stays title-safe under the top
// edge; its formula, title, readout row and last line clear it, each other
// and a three-line lower-third (its top at -2.42), and the panel's numbers
// under its foot
const _: () = assert!(SLOPES.frame.center[0] - SLOPES.frame.width / 2.0 > 1.0);
const _: () = assert!(SLOPES.frame.center[0] + SLOPES.frame.width / 2.0 + 0.6 < 7.1);
const _: () = assert!(FORMULA_Y + 0.45 < 4.0 - TEXT_MARGIN && FORMULA_Y - 0.45 > TITLE_Y + 0.15);
const _: () = assert!(TITLE_Y - 0.15 > SLOPES.frame.center[1] + SLOPES.frame.height / 2.0);
const _: () = assert!(SLOPES.frame.center[1] - SLOPES.frame.height / 2.0 - NUMBER_DROP - 0.2 > READ_Y + 0.3);
const _: () = assert!(READ_Y - 0.3 > NEXT_Y + 0.15 && NEXT_Y - 0.15 > -2.42);
// the right column's rows do not touch (the second step and 03_5's 0/0 are
// fractions, about 0.9 tall), Berkeley's page sits above a lower-third, and
// the ring clears the steps that stay
const _: () = assert!(AUTHOR_Y - SECTIONS_Y > 0.4 && SECTIONS_Y - MODERN_Y > 0.4 && MODERN_Y - STEP_Y[0] > 0.45);
const _: () = assert!(STEP_Y[0] - STEP_Y[1] > 0.7 && STEP_Y[1] - ZERO_Y > 0.95 && STEP_Y[2] - QUOTE_Y[0] > 0.45);
const _: () = assert!(QUOTE_FROM_Y - 0.15 > -2.42 && ZERO_Y - 0.5 > NO_SLOPE_Y + 0.12);
const _: () = assert!(RING_AT[1] + RING_R < STEP_Y[1] - 0.45 && RING_AT[1] - RING_R - 0.45 > -2.42);
// the tolerance's label sits inside the panel, well above the graph 1 + h
// at its own h
const _: () = assert!(BAND_LABEL.0 > SLOPES.frame.x.0 && BAND_LABEL.1 < SLOPES.frame.y.1);
const _: () = assert!(BAND_LABEL.1 > 1.0 + BAND_LABEL.0 + 0.5);
// 03_5's note is readable in its hold, with a second to spare for the 0/0
const _: () = assert!(ZERO_READ_S >= NO_SLOPE.len() as f64 / READ_CPS + 1.0);

/// A fact's fraction ("1/2", "-1/4", "1") as TeX: \tfrac{1}{2}, -\tfrac{1}{4}, 1.
pub(super) fn tex_frac(s: &str) -> String {
    let (sign, body) = s.strip_prefix('-').map_or(("", s), |b| ("-", b));
    match body.split_once('/') {
        Some((n, d)) => format!(r"{sign}\tfrac{{{n}}}{{{d}}}"),
        None => format!("{sign}{body}"),
    }
}

/// A fact's fraction as text, with a true minus sign: "−1/4".
pub(super) fn shown(s: &str) -> String {
    s.replace('-', "−")
}

/// The base point (1/2, 1/4), in data units.
fn base() -> (f64, f64) {
    let x = BASE.f64();
    (x, x * x)
}

/// The far point (1/2 + h, (1/2 + h)²), in data units.
fn far(h: f64) -> fmn::Result<(f64, f64)> {
    let x = BASE.f64() + h;
    Ok((x, power(x, SQUARE).map_err(fail)?))
}

/// The line through the base point with slope s across the plot, detached.
fn line_through(stage: &mut Stage<'_>, s: f64, color: Srgb, w: f64) -> fmn::Result<Mob> {
    let (x0, y0) = base();
    let [a, b] = clip_unit(x0, y0, s).map_err(fail)?;
    PLOT.segment(stage, a, b, color, w)
}

/// The fan's line at slope s, dashed, detached.
fn fan_line(stage: &mut Stage<'_>, s: f64) -> fmn::Result<Mob> {
    let (x0, y0) = base();
    let [a, b] = clip_unit(x0, y0, s).map_err(fail)?;
    PLOT.dashed(stage, a, b, fails())
}

/// The chord from the base point to the far point at increment h, across the
/// plot, detached; h = 0 has no chord, and the model refuses it.
fn chord(stage: &mut Stage<'_>, h: Frac) -> fmn::Result<Mob> {
    let s = chord_slope(BASE, h).map_err(fail)?;
    line_through(stage, s.f64(), slope_color(), CHORD_W)
}

/// The far point at increment h, detached.
fn far_dot(stage: &mut Stage<'_>, h: f64) -> fmn::Result<Mob> {
    let (x, y) = far(h)?;
    Ok(PLOT.dot(stage, x, y, moving()))
}

/// The readout of the slope at the increment `NUDGES[i]`, detached.
fn readout(kit: &Kit, stage: &mut Stage<'_>, f: &Facts, i: usize) -> fmn::Result<Mob> {
    let h = shown(&fact(f, &format!("C3.h.{i}"))?);
    let s = fact(f, &format!("C3.slope.{i}"))?;
    kit.text_left(
        stage,
        &format!("h = {h}: slope {s}"),
        24.0,
        Palette::ink(),
        v(RIGHT_X, READ_Y),
    )
}

/// The slope panel's axes, detached: h along the foot with −1/2, 0 and 1/2
/// under it, the slopes up the left side with the limit's 1 beside it, and
/// h at the end of its axis.
fn panel_axes(kit: &Kit, stage: &mut Stage<'_>, f: &Facts) -> fmn::Result<Vec<Mob>> {
    let p = SLOPES.frame;
    let dim = Palette::dim();
    let (left, right, bottom, top) = (p.left(), p.right(), p.bottom(), p.top());
    let mut out = Vec::new();
    for (a, b) in [
        (v(left - 0.1, bottom), v(right + 0.15, bottom)),
        (v(left, bottom - 0.1), v(left, top + 0.15)),
    ] {
        let l = Line::new(a, b).style(line_style(dim, 2.5)).build()?;
        out.push(stage.arena_mut().add(l));
    }
    let zero = Frac::whole(0);
    for (h, key) in [(NUDGES[3], "C3.h.3"), (zero, "C3.h.to"), (NUDGES[0], "C3.h.0")] {
        let x = SLOPES.p(h.f64(), p.y.0)[0];
        let tick = Line::new(v(x, bottom - 0.08), v(x, bottom + 0.08))
            .style(line_style(dim, 2.5))
            .build()?;
        out.push(stage.arena_mut().add(tick));
        out.push(kit.tex(
            stage,
            &tex_frac(&fact(f, key)?),
            26.0,
            dim,
            &[],
            v(x, bottom - NUMBER_DROP),
        )?);
    }
    let one = SLOPES.p(p.x.0, chord_limit(BASE).map_err(fail)?.f64());
    out.push(kit.text(stage, &fact(f, "C3.limit")?, 24.0, dim, v(left - 0.25, one[1]))?);
    out.push(kit.tex(stage, "h", 30.0, dim, &[], v(right + 0.4, bottom))?);
    Ok(out)
}

/// The panel's picture of where the slopes go, detached: the limit's level
/// (gold, dashed, across the panel) and the hollow mark at h = 0, where the
/// slope has no value.
fn panel_limit(stage: &mut Stage<'_>) -> fmn::Result<[Mob; 2]> {
    let (lo, hi) = SLOPES.frame.x;
    let level = chord_limit(BASE).map_err(fail)?.f64();
    let line = SLOPES.dashed(stage, (lo, level), (hi, level), limit_color())?;
    let hole = Circle::new()
        .radius(END_R)
        .arc_center(SLOPES.p(0.0, level))
        .style(
            Style::default()
                .fill(hex(BACKGROUND), 1.0)
                .stroke(slope_color(), 3.5, 1.0),
        )
        .build();
    Ok([line, stage.arena_mut().add(hole)])
}

/// The graph of the slope against h over 0 < |h| ≤ 1/2, detached: two pieces
/// that stop at the hollow mark's edge, since h = 0 is not in it.
fn panel_graph(stage: &mut Stage<'_>) -> fmn::Result<[Mob; 2]> {
    let p = SLOPES.frame;
    let wide = NUDGES[0];
    let (lo, hi) = (wide.negated().map_err(fail)?, wide);
    let at = |h: Frac| -> fmn::Result<(f64, f64)> { Ok((h.f64(), chord_slope(BASE, h).map_err(fail)?.f64())) };
    // the slope is 1 + h, so a step of r in h moves (r, r) in data units:
    // the mark's edge is END_R away on screen
    let (sx, sy) = (p.width / (p.x.1 - p.x.0), p.height / (p.y.1 - p.y.0));
    let r = END_R / sx.hypot(sy);
    let level = chord_limit(BASE).map_err(fail)?.f64();
    let left = SLOPES.segment(stage, at(lo)?, (-r, level - r), slope_color(), CHORD_W)?;
    let right = SLOPES.segment(stage, (r, level + r), at(hi)?, slope_color(), CHORD_W)?;
    Ok([left, right])
}

/// The tolerance band around the limit and the window of h around 0 for the
/// tolerance t, detached: the band across the panel, the window up it.
fn band_and_window(stage: &mut Stage<'_>, t: f64) -> fmn::Result<[Mob; 2]> {
    let p = SLOPES.frame;
    let level = chord_limit(BASE).map_err(fail)?.f64();
    let (y0, y1) = (SLOPES.p(0.0, level - t)[1], SLOPES.p(0.0, level + t)[1]);
    let (x0, x1) = (SLOPES.p(-t, level)[0], SLOPES.p(t, level)[0]);
    let band = rect(v(p.left(), y0), p.width, y1 - y0, flat(limit_color(), 0.22))?;
    let window = rect(v(x0, p.bottom()), x1 - x0, p.height, flat(moving(), 0.22))?;
    Ok([stage.arena_mut().add(band), stage.arena_mut().add(window)])
}

/// The labels of the tolerance `TOLERANCES[i]` and of its window, detached.
fn tolerance_labels(kit: &Kit, stage: &mut Stage<'_>, f: &Facts, i: usize) -> fmn::Result<[Mob; 2]> {
    let t = fact(f, &format!("C3.eps.{i}"))?;
    let d = fact(f, &format!("C3.delta.{i}"))?;
    let zero = fact(f, "C3.h.to")?;
    let band = kit.text_left(
        stage,
        &format!("{TOLERANCE} {t}"),
        22.0,
        limit_color(),
        SLOPES.p(BAND_LABEL.0, BAND_LABEL.1),
    )?;
    let foot = SLOPES.p(0.0, SLOPES.frame.y.0);
    let window = kit.text(
        stage,
        &format!("{zero} < |h| < {d}"),
        22.0,
        moving(),
        v(foot[0], foot[1] + WINDOW_LABEL_UP),
    )?;
    Ok([band, window])
}

/// A move of `mob` to `to` at an even speed over `rt` seconds, so a chain
/// of them glides without stopping.
fn glide(stage: &mut Stage<'_>, mob: Mob, to: Vec3, rt: f64) -> fmn::Result<()> {
    let anim = mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            rate_func: Some(fmn::core::rate::linear),
            ..AnimateArgs::default()
        })?
        .move_to(to, ORIGIN)?;
    stage.play(anim)?;
    Ok(())
}

/// The quotient's formula above the panel (03_6 on), from facts.
fn formula_tex(f: &Facts) -> fmn::Result<String> {
    let hx = tex_frac(&fact(f, "C3.base")?);
    let square = fact(f, "C3.power")?;
    let quotient = fact(f, "C3.quotient")?;
    let zero = fact(f, "C3.h.to")?;
    Ok(format!(
        r"Q(h) = \frac{{\left({hx} + h\right)^{{{square}}} - \left({hx}\right)^{{{square}}}}}{{h}} = {quotient} \quad (h \neq {zero})"
    ))
}

/// The limit under the panel (03_7 on), from facts.
fn limit_tex(f: &Facts) -> fmn::Result<String> {
    let zero = fact(f, "C3.h.to")?;
    let target = fact(f, "C3.limit")?;
    Ok(format!(r"\lim_{{h \to {zero}}} Q(h) = {target}"))
}

/// The exact statement on the precisely card (03_7 to the end), from facts.
fn exact_text(f: &Facts) -> fmn::Result<String> {
    let zero = fact(f, "C3.h.to")?;
    let half = fact(f, "C3.base")?;
    let square = fact(f, "C3.power")?;
    let quotient = fact(f, "C3.quotient")?;
    let target = fact(f, "C3.limit")?;
    let at_zero = fact(f, "C3.at_zero")?;
    Ok(format!(
        "for every h ≠ {zero}, (({half} + h)^{square} − {})/h = {quotient}, so for every tolerance t > {zero}, \
         {zero} < |h| < t puts it within t of {target}. At h = {zero} it is {at_zero}, which the limit never uses.",
        fact(f, "C3.base.height")?,
    ))
}

/// This chapter's last picture, for the chapter after it, split by what
/// that chapter does with each part.
pub(super) struct Final {
    /// The base point (its guide, its 1/2 and its dot), the chord and far
    /// point at h = −1/10, and the gold line of slope 1. (The plot's axes,
    /// the square and its label stay in the scene and are not returned.)
    pub on_plot: Vec<Mob>,
    /// The slope panel's axes and its limit's dashed level: they stay.
    pub panel: Vec<Mob>,
    /// The hollow mark at h = 0 and the graph's two pieces.
    pub hole: Mob,
    pub graph: [Mob; 2],
    /// The band and window at 1/20, and their labels.
    pub band: Mob,
    pub window: Mob,
    pub labels: [Mob; 2],
    /// What the chapter after clears at once: the formula, the panel's title,
    /// the six hop dots, the limit, the "next:" line and the precisely card.
    pub clear: Vec<Mob>,
}

/// This chapter's last picture, rebuilt with its own builders in the state
/// 03_8 leaves it, back to front as 03_6 to 03_8 stack it: the plot (axes,
/// the square, its label; the base point's guide and 1/2, the chord and the
/// gold line, then the base point and the far point on top); the panel (its
/// axes, the formula and title, the window and band at 1/20 with their
/// labels, then the graph, the dashed level, the hollow mark and the dots on
/// top); the precisely card, the limit and the "next:" line.
pub(super) fn final_picture(kit: &Kit, stage: &mut Stage<'_>, f: &Facts) -> fmn::Result<Final> {
    let mut plot = PLOT.axes(kit, stage)?;
    plot.push(PLOT.curve(stage, SQUARE, curve_color(), BRIGHT_W)?);
    plot.push(curve_label(kit, stage, SQUARE)?);
    let (x0, y0) = base();
    let guide = PLOT.dashed(stage, (x0, 0.0), (x0, y0), fixed())?;
    let foot = PLOT.p(x0, 0.0);
    let half_mark = kit.tex(
        stage,
        &tex_frac(&fact(f, "C3.base")?),
        26.0,
        fixed(),
        &[],
        v(foot[0], foot[1] - NUMBER_DROP),
    )?;
    let last = NUDGES[NUDGES.len() - 1];
    let chord_at = chord(stage, last)?;
    let tangent = line_through(stage, chord_limit(BASE).map_err(fail)?.f64(), limit_color(), CHORD_W)?;
    let point = PLOT.dot(stage, x0, y0, fixed());
    let far_point = far_dot(stage, last.f64())?;
    let on_plot = vec![guide, half_mark, chord_at, tangent, point, far_point];

    let mut panel = panel_axes(kit, stage, f)?;
    let formula = kit.tex(
        stage,
        &formula_tex(f)?,
        28.0,
        Palette::ink(),
        &[],
        v(SLOPES.frame.center[0], FORMULA_Y),
    )?;
    let title = kit.text(
        stage,
        SLOPE_OF,
        22.0,
        Palette::dim(),
        v(SLOPES.frame.center[0], TITLE_Y),
    )?;
    let at = TOLERANCES.len() - 1;
    let [band, window] = band_and_window(stage, TOLERANCES[at].f64())?;
    let labels = tolerance_labels(kit, stage, f, at)?;
    let graph = panel_graph(stage)?;
    let [level, hole] = panel_limit(stage)?;
    panel.push(level);
    let mut dots = Vec::new();
    for &h in &NUDGES {
        dots.push(SLOPES.dot(stage, h.f64(), chord_slope(BASE, h).map_err(fail)?.f64(), slope_color()));
    }
    let card = precisely(kit, stage, &exact_text(f)?)?;
    let limit_line = kit.tex_left(stage, &limit_tex(f)?, 30.0, limit_color(), &[], v(RIGHT_X, READ_Y))?;
    let next = kit.text_left(stage, NEXT, 22.0, Palette::dim(), v(RIGHT_X, NEXT_Y))?;

    let mut order = plot;
    order.extend(&on_plot);
    order.extend(&panel[..panel.len() - 1]);
    order.extend([
        formula, title, window, labels[1], band, labels[0], graph[0], graph[1], level, hole,
    ]);
    order.extend(&dots);
    order.extend([card, limit_line, next]);
    for &m in &order {
        stage.add_to_scene(m)?;
    }
    let mut clear = vec![formula, title];
    clear.extend(dots);
    clear.extend([card, limit_line, next]);
    Ok(Final {
        on_plot,
        panel,
        hole,
        graph,
        band,
        window,
        labels,
        clear,
    })
}

/// Prepared crossfades from `from` to `to`, `rt` seconds.
pub(super) fn swap(stage: &mut Stage<'_>, from: &[Mob], to: &[Mob], rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in from {
        let out = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(out.rt(rt * 0.5))?);
    }
    for &m in to {
        let inn = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(inn.rt(rt))?);
    }
    Ok(anims)
}

impl SceneConstruct for WhatTheOldObjectionWasAbout {
    fn name(&self) -> &str {
        "03_what_the_old_objection_was_about"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let f = facts().map_err(fail)?;
        let Resting {
            bright: b,
            rest,
            summary,
        } = resting(&kit, stage, &f)?;
        let year = fact(&f, "C3.year")?;
        let half = fact(&f, "C3.base")?;
        let square = fact(&f, "C3.power")?;

        // 03_1: the plot clears to its axes and the bright curve; Berkeley's
        // book and the sections read
        let line = say(&kit, stage, "03_1")?;
        let mut leaving = rest;
        leaving.push(summary[1]);
        fade_out_all(stage, &leaving, 0.8)?;
        line.wait_for(stage, BERKELEY_AT)?;
        let head = [
            kit.text_left(
                stage,
                &format!("{AUTHOR} ({year})"),
                30.0,
                Palette::ink(),
                v(RIGHT_X, AUTHOR_Y),
            )?,
            kit.text_left(
                stage,
                &format!(
                    "{SECTIONS} {}–{}{OF_A_POWER}",
                    fact(&f, "C3.page.first")?,
                    fact(&f, "C3.page.last")?
                ),
                22.0,
                Palette::dim(),
                v(RIGHT_X, SECTIONS_Y),
            )?,
        ];
        let anims = swap(stage, &[summary[0]], &head, 0.8)?;
        stage.play_prepared(anims)?;

        // 03_2: the square at x = 1/2, ours and modern; the far point at
        // h = 1/2, the change h + h², the division by h ≠ 0 and the slope 1 + h
        let line = say(&kit, stage, "03_2")?;
        line.wait_for(stage, SQUARE_AT)?;
        let curve = PLOT.curve(stage, SQUARE, curve_color(), BRIGHT_W)?;
        let label = curve_label(&kit, stage, SQUARE)?;
        let modern = kit.tex_left(
            stage,
            &format!("{MODERN}^{{{square}}}{AT_X}{}", tex_frac(&half)),
            28.0,
            Palette::ink(),
            &[],
            v(RIGHT_X, MODERN_Y),
        )?;
        let morph = fmn::animation::replacement_transform(b.curve, curve).rt(0.8);
        let mut anims = vec![stage.prepare(morph)?];
        anims.extend(swap(stage, &[b.label], &[label, modern], 0.8)?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, MOVE_AT)?;
        let (x0, y0) = base();
        let guide = PLOT.dashed(stage, (x0, 0.0), (x0, y0), fixed())?;
        let foot = PLOT.p(x0, 0.0);
        let half_mark = kit.tex(
            stage,
            &tex_frac(&half),
            26.0,
            fixed(),
            &[],
            v(foot[0], foot[1] - NUMBER_DROP),
        )?;
        let point = PLOT.dot(stage, x0, y0, fixed());
        fade_in_all(stage, &[guide, half_mark, point], 0.5)?;
        let first = NUDGES[0];
        let (x1, y1) = far(first.f64())?;
        let mut far_point = far_dot(stage, first.f64())?;
        let run = PLOT.segment(stage, (x0, y0), (x1, y0), moving(), 4.0)?;
        let run_mid = PLOT.p(f64::midpoint(x0, x1), y0);
        let run_label = kit.tex(stage, "h", 30.0, moving(), &[], v(run_mid[0], run_mid[1] - 0.25))?;
        fade_in_lagged(stage, &[far_point, run, run_label], 0.9, 0.3)?;
        line.wait_for(stage, HEIGHT_AT)?;
        let change = fact(&f, "C3.change")?;
        let rise = PLOT.segment(stage, (x1, y0), (x1, y1), curve_color(), 4.0)?;
        let rise_mid = PLOT.p(x1, f64::midpoint(y0, y1));
        let rise_label = kit.tex_left(
            stage,
            &change,
            28.0,
            curve_color(),
            &[],
            v(rise_mid[0] + 0.15, rise_mid[1]),
        )?;
        let hx = tex_frac(&half);
        let step1 = kit.tex_left(
            stage,
            &format!(r"\left({hx} + h\right)^{{{square}}} - \left({hx}\right)^{{{square}}} = {change}"),
            30.0,
            Palette::ink(),
            &[],
            v(RIGHT_X, STEP_Y[0]),
        )?;
        fade_in_lagged(stage, &[rise, rise_label, step1], 0.9, 0.3)?;
        line.wait_for(stage, DIVIDE_AT)?;
        let quotient = fact(&f, "C3.quotient")?;
        let zero = fact(&f, "C3.h.to")?;
        let mut chord_now = chord(stage, first)?;
        let step2 = kit.tex_left(
            stage,
            &format!(r"\frac{{{change}}}{{h}} = {quotient}"),
            30.0,
            Palette::ink(),
            &[],
            v(RIGHT_X, STEP_Y[1]),
        )?;
        let step2_right = RIGHT_X + stage.get_width(step2) + 0.35;
        let nonzero = kit.tex_left(
            stage,
            &format!(r"(h \neq {zero})"),
            30.0,
            moving(),
            &[],
            v(step2_right, STEP_Y[1]),
        )?;
        fade_in_lagged(stage, &[chord_now, step2, nonzero], 0.9, 0.3)?;
        stage.scene_mut().bring_to_front(&[point, far_point])?;

        // 03_3: the old argument lets h vanish and keeps 1; Berkeley's
        // sentence, verbatim, with its section, page and edition (the book
        // and its year head the column)
        let line = say(&kit, stage, "03_3")?;
        line.wait_for(stage, VANISH_AT)?;
        let target = fact(&f, "C3.limit")?;
        let step3 = kit.tex_left(
            stage,
            &format!(r"\text{{then }} h = {zero} \text{{, so }} {quotient} = {target}"),
            30.0,
            fails(),
            &[],
            v(RIGHT_X, STEP_Y[2]),
        )?;
        fade_in_all(stage, &[step3], 0.6)?;
        line.wait_for(stage, OBJECTED_AT)?;
        let mut quote = Vec::new();
        for (text, y) in QUOTE.iter().zip(QUOTE_Y) {
            quote.push(kit.text_left(stage, text, 20.0, Palette::ink(), v(RIGHT_X, y))?);
        }
        quote.push(kit.text_left(
            stage,
            &format!(
                "{QUOTE_FROM} {} {} {} {}",
                fact(&f, "C3.quote.page")?,
                EDITION[0],
                fact(&f, "C3.edition")?,
                EDITION[1]
            ),
            18.0,
            Palette::dim(),
            v(RIGHT_X, QUOTE_FROM_Y),
        )?);
        fade_in_lagged(stage, &quote, 1.2, 0.3)?;

        // 03_4: the prediction. Berkeley's sentence stays up while the
        // question is asked, then gives way to the ring, which drains with
        // the answer hidden
        say(&kit, stage, "03_4")?;
        kit.hold(stage)?;
        let mut clearing = quote;
        clearing.push(step3);
        let ask = kit.text(stage, "?", 60.0, Palette::hero(), RING_AT)?;
        let anims = swap(stage, &clearing, &[ask], 0.5)?;
        stage.play_prepared(anims)?;
        think_pause(&kit, stage, RING_AT, RING_R)?;

        // 03_5: at h = 0 the far point is the base point; the quotient is
        // 0/0 and every line through the point fits
        let line = say(&kit, stage, "03_5")?;
        fade_out_all(stage, &[ask, chord_now, run, run_label, rise, rise_label], 0.4)?;
        // the far point slides along the curve onto the base point
        for h in SLIDE {
            let (x, y) = far(h)?;
            glide(stage, far_point, PLOT.p(x, y), SLIDE_S / SLIDE.len() as f64)?;
        }
        line.wait_for(stage, ZERO_AT)?;
        let mut fan = Vec::new();
        for s in FAN {
            fan.push(fan_line(stage, s)?);
        }
        let at_zero = fact(&f, "C3.at_zero")?;
        let (zero_num, zero_den) = at_zero
            .split_once('/')
            .ok_or_else(|| fail(format!("C3.at_zero is {at_zero}")))?;
        let zero_step = kit.tex_left(
            stage,
            &format!(
                r"h = {zero}:\quad \frac{{\left({hx} + {zero}\right)^{{{square}}} - \left({hx}\right)^{{{square}}}}}{{{zero}}} = \frac{{{zero_num}}}{{{zero_den}}}"
            ),
            30.0,
            fails(),
            &[],
            v(RIGHT_X, ZERO_Y),
        )?;
        let no_slope = kit.text_left(stage, NO_SLOPE, 22.0, Palette::dim(), v(RIGHT_X, NO_SLOPE_Y))?;
        let mut ins = fan.clone();
        ins.extend([zero_step, no_slope]);
        fade_in_lagged(stage, &ins, 1.2, 0.15)?;
        stage.scene_mut().bring_to_front(&[point, far_point])?;
        let zero_shown = stage.scene().time().to_f64();

        // 03_6: the answer, with h never zero; the 0/0 stays into its first
        // sentence until read, then the panel of slopes against h and the
        // hollow mark at h = 0; the far point hops through ±1/2, ±1/4, ±1/10,
        // each slope landing on the panel; the graph and the line of slope 1
        let line = say(&kit, stage, "03_6")?;
        wait_until(stage, zero_shown + ZERO_READ_S)?;
        let mut clearing = fan;
        clearing.extend([
            zero_step, no_slope, far_point, step1, step2, nonzero, modern, head[0], head[1],
        ]);
        fade_out_all(stage, &clearing, 0.5)?;
        let mut panel = panel_axes(&kit, stage, &f)?;
        let formula = kit.tex(
            stage,
            &formula_tex(&f)?,
            28.0,
            Palette::ink(),
            &[],
            v(SLOPES.frame.center[0], FORMULA_Y),
        )?;
        let title = kit.text(
            stage,
            SLOPE_OF,
            22.0,
            Palette::dim(),
            v(SLOPES.frame.center[0], TITLE_Y),
        )?;
        let [level, hole] = panel_limit(stage)?;
        panel.extend([formula, title, level, hole]);
        fade_in_all(stage, &panel, 0.8)?;
        line.wait_for(stage, SHRINK_AT)?;
        far_point = far_dot(stage, first.f64())?;
        chord_now = chord(stage, first)?;
        let mut dots = vec![SLOPES.dot(
            stage,
            first.f64(),
            chord_slope(BASE, first).map_err(fail)?.f64(),
            slope_color(),
        )];
        let mut shown_now = readout(&kit, stage, &f, 0)?;
        fade_in_all(stage, &[chord_now, far_point, dots[0], shown_now], 0.4)?;
        stage.scene_mut().bring_to_front(&[point, far_point])?;
        for (i, &h) in NUDGES.iter().enumerate().skip(1) {
            let (x, y) = far(h.f64())?;
            let next_chord = chord(stage, h)?;
            let dot = SLOPES.dot(stage, h.f64(), chord_slope(BASE, h).map_err(fail)?.f64(), slope_color());
            let next_shown = readout(&kit, stage, &f, i)?;
            let mut anims = vec![
                stage.prepare(fmn::animation::replacement_transform(chord_now, next_chord).rt(HOP_S))?,
                stage.prepare(move_to(far_point, PLOT.p(x, y), HOP_S)?)?,
            ];
            anims.extend(swap(stage, &[shown_now], &[dot, next_shown], HOP_S)?);
            stage.play_prepared(anims)?;
            stage.scene_mut().bring_to_front(&[point, far_point])?;
            chord_now = next_chord;
            shown_now = next_shown;
            dots.push(dot);
        }
        line.wait_for(stage, CLOSE_AT)?;
        let graph = panel_graph(stage)?;
        let tangent = line_through(stage, chord_limit(BASE).map_err(fail)?.f64(), limit_color(), CHORD_W)?;
        let anims = swap(stage, &[shown_now], &[graph[0], graph[1], tangent], 0.8)?;
        stage.play_prepared(anims)?;
        stage.scene_mut().bring_to_front(&[hole])?;
        stage.scene_mut().bring_to_front(&dots)?;
        stage.scene_mut().bring_to_front(&[point, far_point])?;

        // 03_7: the window 0 < |h| < 1/10 and the band within 1/10 of 1; any
        // tolerance the same way (1/20 drawn), with the exact statement on a
        // card that comes in as they narrow and stays to the end; the limit
        let line = say(&kit, stage, "03_7")?;
        line.wait_for(stage, WINDOW_AT)?;
        let [mut band, mut window] = band_and_window(stage, TOLERANCES[0].f64())?;
        let [mut band_label, mut window_label] = tolerance_labels(&kit, stage, &f, 0)?;
        fade_in_all(stage, &[window, window_label], 0.5)?;
        line.wait_for(stage, BAND_AT)?;
        fade_in_all(stage, &[band, band_label], 0.5)?;
        let mut front = vec![graph[0], graph[1], level, hole];
        front.extend(&dots);
        stage.scene_mut().bring_to_front(&front)?;
        line.wait_for(stage, ANY_AT)?;
        for (i, &t) in TOLERANCES.iter().enumerate().skip(1) {
            let [next_band, next_window] = band_and_window(stage, t.f64())?;
            let [next_band_label, next_window_label] = tolerance_labels(&kit, stage, &f, i)?;
            let mut anims = vec![
                stage.prepare(fmn::animation::replacement_transform(band, next_band).rt(0.7))?,
                stage.prepare(fmn::animation::replacement_transform(window, next_window).rt(0.7))?,
            ];
            anims.extend(swap(
                stage,
                &[band_label, window_label],
                &[next_band_label, next_window_label],
                0.7,
            )?);
            stage.play_prepared(anims)?;
            stage.scene_mut().bring_to_front(&front)?;
            band = next_band;
            window = next_window;
            band_label = next_band_label;
            window_label = next_window_label;
        }
        // the exact statement comes in as the window narrows and stands to
        // the chapter's end, through 03_8: the slope is exactly 1 + h for
        // every h ≠ 0, every tolerance has its window, and h = 0 gives 0/0
        let lower = precisely(&kit, stage, &exact_text(&f)?)?;
        fade_in_all(stage, &[lower], 0.5)?;
        line.wait_for(stage, LIMIT_AT)?;
        let limit_line = kit.tex_left(stage, &limit_tex(&f)?, 30.0, limit_color(), &[], v(RIGHT_X, READ_Y))?;
        fade_in_all(stage, &[limit_line], 0.6)?;

        // 03_8: the limit never needs the questioned step; who picks the
        // tolerance comes next; whether limits exist is another question. The
        // exact card stays.
        let line = say(&kit, stage, "03_8")?;
        line.wait_for(stage, NEVER_AT)?;
        glow(&kit, stage, SLOPES.p(0.0, chord_limit(BASE).map_err(fail)?.f64()), 0.2)?;
        line.wait_for(stage, NEXT_AT)?;
        let next = kit.text_left(stage, NEXT, 22.0, Palette::dim(), v(RIGHT_X, NEXT_Y))?;
        fade_in_all(stage, &[next], 0.6)?;
        kit.hold(stage)?;
        kit.music_breath(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every phrase a picture waits for is said once, in the line that
    /// waits for it, in the order the pictures take them.
    #[test]
    fn the_pictures_wait_on_words_the_script_says() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let text = |id: &str| script.text(id).unwrap();
        let waits = [
            ("03_1", BERKELEY_AT),
            ("03_2", SQUARE_AT),
            ("03_2", MOVE_AT),
            ("03_2", HEIGHT_AT),
            ("03_2", DIVIDE_AT),
            ("03_3", VANISH_AT),
            ("03_3", OBJECTED_AT),
            ("03_5", ZERO_AT),
            ("03_6", SHRINK_AT),
            ("03_6", CLOSE_AT),
            ("03_7", WINDOW_AT),
            ("03_7", BAND_AT),
            ("03_7", ANY_AT),
            ("03_7", LIMIT_AT),
            ("03_8", NEVER_AT),
            ("03_8", NEXT_AT),
        ];
        for (id, phrase) in waits {
            assert_eq!(text(id).matches(phrase).count(), 1, "{id} says {phrase:?} once");
        }
        for pair in waits.windows(2).filter(|w| w[0].0 == w[1].0) {
            let line = text(pair[0].0);
            assert!(line.find(pair[0].1) < line.find(pair[1].1), "{pair:?}");
        }
    }

    /// The modern example is named as ours before any step is shown, h is
    /// nonzero where it is divided by, and the question is asked before its
    /// answer, 0/0, is given.
    #[test]
    fn the_modern_step_keeps_h_nonzero() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let text = |id: &str| script.text(id).unwrap();
        let two = text("03_2");
        let at = |p: &str| two.find(p).unwrap();
        assert!(at("in modern notation") < at(MOVE_AT));
        assert!(at("not zero") < at(DIVIDE_AT));
        assert!(text("03_4").ends_with('?') && text("03_5").starts_with("None."));
        assert!(text("03_6").contains("without h ever being zero"));
    }

    /// The panel shows every increment drawn, and its graph keeps 1/2 + h in
    /// the plot; the window at each tolerance is inside the panel and wider
    /// than the hollow mark, so it reads around it.
    #[test]
    fn the_panel_holds_what_it_draws() {
        let p = SLOPES.frame;
        for h in NUDGES {
            let s = chord_slope(BASE, h).unwrap().f64();
            assert!(p.x.0 < h.f64() && h.f64() < p.x.1 && p.y.0 < s && s < p.y.1, "h = {h}");
        }
        let sx = p.width / (p.x.1 - p.x.0);
        let sy = p.height / (p.y.1 - p.y.0);
        for t in TOLERANCES {
            assert!(
                2.0 * t.f64() * sx > 4.0 * END_R && 2.0 * t.f64() * sy > 2.0 * END_R,
                "t = {t}"
            );
            assert!(t.f64() < p.x.1 && 1.0 + t.f64() < p.y.1);
        }
        // the tolerance's label sits above the widest band drawn
        let (_, ly) = BAND_LABEL;
        assert!(ly > chord_limit(BASE).unwrap().f64() + TOLERANCES[0].f64() + 0.1);
        assert_eq!(tex_frac("-1/4"), r"-\tfrac{1}{4}");
        assert_eq!(tex_frac("0"), "0");
        assert_eq!(shown("-1/10"), "−1/10");
    }
}
