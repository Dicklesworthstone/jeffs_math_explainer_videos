//! Chapter 02, the shrinking region: chapter 01's last picture, then a zoom
//! window on the last tenth of [0, 1]. The strip where the curve is above
//! one half, x_n < x ≤ 1, holds x = 1 (02_1); at x = 1 the curve and the
//! limit are both 1, so the gap there is zero (02_2); the gap is at least one
//! half on x_n ≤ x < 1, from the crossing to just short of 1 (02_3). The
//! strip narrows as the power grows to 1000 (02_4), narrower than one pixel
//! of the final picture from n = 898; the prediction (02_5); stretching the
//! window ten, a hundred, a thousand times shows it still there (02_6), and
//! power and stretch grown together show the width shrinking like ln 2/n
//! without ever vanishing (02_7); the moving point stays inside it with its
//! gap of one half (02_8); a picture cannot settle this, a guarantee for
//! every point can (02_9).
//!
//! The zoom window is drawn at its true stretch, labelled; nothing is
//! widened to make it visible. Every number drawn comes from the facts
//! registry; every shape from the model.

use explainer_kit::props::precisely;
use explainer_kit::{BACKGROUND, Facts, Frame2, Kit, Palette, Timed, hex, rect, thousands, v};
use fmn::prelude::*;

use super::draw::{
    BRIGHT_W, Bright, END_R, GHOST_W, LENS_H, LENS_W, PLOT, Plot, curve_color, curve_label, fact, fade_in_all,
    fade_in_lagged, fade_out_all, fail, flat, gap_color, ghost, line_style, move_to, moving, say, step, think_pause,
};
use super::limit_that_jumps::{
    card_fix_first, card_move_with, fixed_back, gap_note, gap_span, half_line, witness_at, witness_formula,
    witness_head, witness_row,
};
use crate::facts::facts;
use crate::model::{MORE_POWERS, POWERS, Ratio, ZOOMS, above_half, gap_at_least_half, half_height};

pub struct TheShrinkingRegion {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const STRETCHED: &str = "stretched ×";
const SIDEWAYS: &str = "sideways";
const HEIGHTS: &str = "heights ×";
const ABOVE_LEGEND: &str = r"\text{curve above } \tfrac{1}{2}:\ x_n < x \le 1";
const GAP_LEGEND: &str = r"\text{gap at least } \tfrac{1}{2}:\ x_n \le x < 1";
const AT_ONE: &str = "at x = 1: curve";
const LIMIT: &str = "limit";
const GAP: &str = "gap";
const WIDTH: &str = "width";
const PIXELS_A: &str = "the plot is";
const PIXELS_B: &str = "pixels wide in the 4K picture;";
const PIXELS_C: &str = "from n =";
const PIXELS_D: &str = "on, the strip is narrower than one pixel";
const FILLS: &str = "the strip fills";
const OF_WINDOW: &str = "of the window";
const EVERY_POINT: &str = "a guarantee for every x at once";
const WHO_FIRST: &str = "who chooses first: how close, which power, which point?";

// what the pictures wait for, each in the line the test below holds it to
const STRIP_AT: &str = "That's a strip";
const BELONGS_AT: &str = "x equals one belongs to it";
const BOTH_AT: &str = "both at height one";
const STARTS_AT: &str = "a strip that starts";
const HUNDRED_AT: &str = "a hundred times";
const THOUSAND_AT: &str = "a thousand times";
const RESTART_AT: &str = "Reset the window to x to the tenth";
const RAISE_AT: &str = "Raise the power tenfold";
const AGAIN_AT: &str = "then again";
const SHRINKS_AT: &str = "Its width shrinks";
const INSIDE_AT: &str = "always inside it";
const NEED_AT: &str = "What we need";

// --- layout (scene units; the frame is 14.2 by 8; the plot is draw::PLOT)

/// The zoom window's panel sits right of the plot (its size is draw's
/// `LENS_W` and `LENS_H`: the plot's width, a third of its height).
const LENS_CENTER: Vec3 = [3.7, 1.9, 0.0];
/// Above the panel: its two ends' x, then the window's label.
const ENDS_Y: f64 = LENS_CENTER[1] + LENS_H / 2.0 + 0.22;
const ZOOM_Y: f64 = LENS_CENTER[1] + LENS_H / 2.0 + 0.6;
/// Under the panel: the two strips (curve above 1/2, gap at least 1/2).
const ABOVE_Y: f64 = LENS_CENTER[1] - LENS_H / 2.0 - 0.28;
const GAP_Y: f64 = LENS_CENTER[1] - LENS_H / 2.0 - 0.54;
const STRIP_H: f64 = 0.1;
const MARK_R: f64 = 0.07;
/// The legends under the strips, left-aligned with the panel.
const LEGEND_Y: [f64; 2] = [-0.02, -0.38];
/// The readouts under the legends, left-aligned.
const READ_X: f64 = 0.6;
const READ_Y: [f64; 3] = [-0.95, -1.3, -1.65];
/// The think ring, right of the legends.
const RING_AT: Vec3 = [5.9, -0.2, 0.0];
const RING_R: f64 = 0.45;
/// The main plot's gap strip, just under its x axis.
const MAIN_STRIP_DROP: f64 = 0.13;
const MAIN_STRIP_H: f64 = 0.05;
/// The arrow that points at that strip once it is narrower than a pixel:
/// its tip this far right of x = 1, its shaft this long. It sits at the
/// strip's height, between the limit's hollow mark above and the axis
/// numeral below; a ring there touched both (review 3881).
const POINTER_GAP: f64 = 0.14;
const POINTER_LEN: f64 = 0.2;
const POINTER_HEAD: f64 = 0.05;

// everything sits inside the frame and above a three-line lower-third
// (its top at -2.42); the strips sit under the panel and above the legends
const _: () = assert!(LENS_CENTER[0] - LENS_W / 2.0 > 0.4 && LENS_CENTER[0] + LENS_W / 2.0 < 6.9);
const _: () = assert!(ZOOM_Y + 0.2 < 4.0);
const _: () = assert!(GAP_Y - STRIP_H / 2.0 - MARK_R > LEGEND_Y[0] + 0.2);
const _: () = assert!(READ_Y[2] - 0.2 > -2.42 && READ_Y[0] < LEGEND_Y[1] - 0.3);
const _: () = assert!(RING_AT[0] + RING_R < 7.1 && RING_AT[1] - RING_R - 0.45 > READ_Y[1]);
// the arrow's tip sits right of the hollow mark at (1, 0) and of the numeral
// 1 under it (narrower than the mark), with room for both strokes, and its
// head stays under the mark
const _: () = assert!(POINTER_GAP > END_R + 0.06 && MAIN_STRIP_DROP - POINTER_HEAD > END_R);

/// The zoom window at a stretch of m: [1 − 1/m, 1] across the panel.
fn lens(m: u32) -> Plot {
    Plot {
        frame: Frame2 {
            center: LENS_CENTER,
            width: LENS_W,
            height: LENS_H,
            x: (1.0 - 1.0 / f64::from(m), 1.0),
            y: (0.0, 1.0),
        },
    }
}

/// A strip from x to 1 at height y (scene units) under `plot`, detached:
/// as wide as the plot draws [x, 1], however thin that is.
fn strip(stage: &mut Stage<'_>, plot: &Plot, x: f64, y: f64, h: f64, color: Srgb) -> fmn::Result<Mob> {
    let (left, right) = (plot.p(x, 0.0)[0], plot.p(1.0, 0.0)[0]);
    let r = rect(v(left, y - h / 2.0), right - left, h, flat(color, 0.9))?;
    Ok(stage.arena_mut().add(r))
}

/// An end mark of a strip, detached: filled when the end belongs to the
/// strip, hollow when it does not.
fn end_mark(stage: &mut Stage<'_>, at: Vec3, color: Srgb, belongs: bool) -> Mob {
    let style = if belongs {
        flat(color, 1.0)
    } else {
        Style::default().fill(hex(BACKGROUND), 1.0).stroke(color, 3.0, 1.0)
    };
    let c = Circle::new().radius(MARK_R).arc_center(at).style(style).build();
    stage.arena_mut().add(c)
}

/// A ring that glows around `at` and fades: a highlight that leaves nothing.
fn glow(kit: &Kit, stage: &mut Stage<'_>, at: Vec3, r: f64) -> fmn::Result<()> {
    let ring = Circle::new()
        .radius(r)
        .arc_center(at)
        .style(line_style(Palette::hero(), 4.0))
        .build();
    let ring = stage.arena_mut().add(ring);
    kit.allow_crossing(ring);
    fade_in_all(stage, &[ring], 0.3)?;
    fade_out_all(stage, &[ring], 0.6)
}

/// What the zoom window shows of the n-th curve at a stretch of m: the
/// curve, the moving point's gap bar and dot, and the two strips; their four
/// end marks when the strip is wide enough on screen to tell its ends apart;
/// the window's label and its left end's x.
struct View {
    shapes: [Mob; 5],
    marks: Option<[Mob; 4]>,
    labels: [Mob; 2],
}

impl View {
    fn build(kit: &Kit, stage: &mut Stage<'_>, f: &Facts, n: u32, m: u32) -> fmn::Result<Self> {
        let plot = lens(m);
        let x = half_height(n).map_err(fail)?;
        let curve = plot.curve_window(stage, n, curve_color(), BRIGHT_W)?;
        let (floor, top) = gap_span(x, n)?;
        let bar = plot.segment(stage, (x, top), (x, floor), gap_color(), 6.0)?;
        let dot = plot.dot(stage, x, top, moving());
        let above = strip(stage, &plot, x, ABOVE_Y, STRIP_H, moving())?;
        let gaps = strip(stage, &plot, x, GAP_Y, STRIP_H, gap_color())?;
        let (left, right) = (plot.p(x, 0.0)[0], plot.p(1.0, 0.0)[0]);
        // whether x = 1 belongs to each strip is the model's exact answer
        // there (the curve is 1, above 1/2; the gap is 0, below it); at x_n
        // the curve and the gap are both exactly 1/2 by x_n's definition, so
        // x_n is not above 1/2 and its gap is at least 1/2
        let one = Ratio::point(1, 1).map_err(fail)?;
        let (one_above, one_gap) = (above_half(one, n), gap_at_least_half(one, n));
        let marks = (right - left >= 6.0 * MARK_R).then(|| {
            [
                end_mark(stage, v(left, ABOVE_Y), moving(), false),
                end_mark(stage, v(right, ABOVE_Y), moving(), one_above),
                end_mark(stage, v(left, GAP_Y), gap_color(), true),
                end_mark(stage, v(right, GAP_Y), gap_color(), one_gap),
            ]
        });
        let heights = fact(f, "C2.lens.heights")?;
        let zoom = kit.text(
            stage,
            &format!("n = {n}, {STRETCHED}{m} {SIDEWAYS}; {HEIGHTS}{heights}"),
            22.0,
            Palette::dim(),
            v(LENS_CENTER[0], ZOOM_Y),
        )?;
        let window = fact(f, &format!("C2.window.x{m}"))?;
        let end = kit.text(stage, &window, 22.0, Palette::dim(), v(plot.frame.left(), ENDS_Y))?;
        Ok(Self {
            shapes: [curve, bar, dot, above, gaps],
            marks,
            labels: [zoom, end],
        })
    }

    /// Prepared animations that turn this view into `next` over `rt`
    /// seconds: shapes morph, marks morph or come and go, labels swap.
    fn morph_to(self, stage: &mut Stage<'_>, next: &Self, rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for (a, b) in self.shapes.into_iter().zip(next.shapes) {
            anims.push(stage.prepare(fmn::animation::replacement_transform(a, b).rt(rt))?);
        }
        match (self.marks, next.marks) {
            (Some(a), Some(b)) => {
                for (a, b) in a.into_iter().zip(b) {
                    anims.push(stage.prepare(fmn::animation::replacement_transform(a, b).rt(rt))?);
                }
            }
            (Some(a), None) => {
                for m in a {
                    let out = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
                    anims.push(stage.prepare(out.rt(rt * 0.5))?);
                }
            }
            (None, Some(b)) => {
                for m in b {
                    let inn = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
                    anims.push(stage.prepare(inn.rt(rt))?);
                }
            }
            (None, None) => {}
        }
        for m in self.labels {
            let out = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(out.rt(rt * 0.5))?);
        }
        for m in next.labels {
            let inn = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(inn.rt(rt))?);
        }
        Ok(anims)
    }
}

/// The magnifier on the main plot for a stretch of m, detached: the box
/// around [1 − 1/m, 1] at its true width, and two lines to the panel.
fn magnifier(stage: &mut Stage<'_>, m: u32) -> fmn::Result<[Mob; 3]> {
    let lo = 1.0 - 1.0 / f64::from(m);
    let (bl, tr, br) = (PLOT.p(lo, 0.0), PLOT.p(1.0, 1.0), PLOT.p(1.0, 0.0));
    let panel = lens(m).frame;
    let frame = rect(bl, tr[0] - bl[0], tr[1] - bl[1], line_style(Palette::dim(), 2.0))?;
    let upper = Line::new(tr, v(panel.left(), panel.top()))
        .style(line_style(Palette::faint(), 1.5))
        .build()?;
    let lower = Line::new(br, v(panel.left(), panel.bottom()))
        .style(line_style(Palette::faint(), 1.5))
        .build()?;
    Ok([
        stage.arena_mut().add(frame),
        stage.arena_mut().add(upper),
        stage.arena_mut().add(lower),
    ])
}

/// Prepared morphs from one magnifier to another, `rt` seconds.
fn magnify(stage: &mut Stage<'_>, from: [Mob; 3], to: [Mob; 3], rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for (a, b) in from.into_iter().zip(to) {
        anims.push(stage.prepare(fmn::animation::replacement_transform(a, b).rt(rt))?);
    }
    Ok(anims)
}

/// The main plot's gap strip for the n-th curve, just under the x axis,
/// detached: at its true width, below one pixel from n = 898 on.
fn main_strip(stage: &mut Stage<'_>, n: u32) -> fmn::Result<Mob> {
    let x = half_height(n).map_err(fail)?;
    let y = PLOT.frame.bottom() - MAIN_STRIP_DROP;
    strip(stage, &PLOT, x, y, MAIN_STRIP_H, gap_color())
}

/// A short arrow pointing left at the main plot's strip from just right of
/// x = 1, at the strip's height, detached: where the strip is once it is
/// narrower than a pixel, not how wide it is.
fn pointer(stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let end = PLOT.p(1.0, 0.0);
    let y = PLOT.frame.bottom() - MAIN_STRIP_DROP;
    let tip = v(end[0] + POINTER_GAP, y);
    let ends = [
        v(tip[0] + POINTER_LEN, y),
        v(tip[0] + POINTER_HEAD * 1.4, y + POINTER_HEAD),
        v(tip[0] + POINTER_HEAD * 1.4, y - POINTER_HEAD),
    ];
    let mut lines = Vec::new();
    for e in ends {
        lines.push(Line::new(e, tip).style(line_style(Palette::hero(), 3.0)).build()?);
    }
    Ok(stage.arena_mut().add(v_group(lines)))
}

/// The readout of the n-th strip's width, detached.
fn width_readout(kit: &Kit, stage: &mut Stage<'_>, f: &Facts, n: u32) -> fmn::Result<Mob> {
    let w = fact(f, &format!("C2.width.n{n}.6dp"))?;
    kit.text_left(
        stage,
        &format!("n = {n}: {WIDTH} {w}"),
        24.0,
        gap_color(),
        v(READ_X, READ_Y[0]),
    )
}

/// Chapter 01's last picture as it rests, rebuilt with chapter 01's own
/// builders: the plot with every curve, x⁵⁰ bright, the half line, the moving
/// point on x⁵⁰ with its gap bar, the table, its notes, the fixed point and
/// the two cards (`panel`, which 02_1 clears).
struct Opening {
    bright: Bright,
    dot: Mob,
    bar: Mob,
    panel: Vec<Mob>,
}

fn opening(kit: &Kit, stage: &mut Stage<'_>, f: &Facts) -> fmn::Result<Opening> {
    let last = POWERS[POWERS.len() - 1];
    let mut scene = PLOT.axes(kit, stage)?;
    scene.extend(PLOT.limit(stage)?);
    for &n in &POWERS {
        scene.push(PLOT.curve(stage, n, ghost(), GHOST_W)?);
    }
    let curve = PLOT.curve(stage, last, curve_color(), BRIGHT_W)?;
    let label = curve_label(kit, stage, last)?;
    scene.extend([curve, label]);
    scene.extend(half_line(kit, stage)?);
    let mut panel = witness_head(kit, stage)?.to_vec();
    for i in 0..POWERS.len() {
        panel.extend(witness_row(kit, stage, f, i)?);
    }
    panel.push(gap_note(kit, stage, f)?);
    panel.push(witness_formula(kit, stage)?);
    panel.extend(fixed_back(stage, last)?);
    panel.extend(card_fix_first(kit, stage, f)?);
    panel.extend(card_move_with(kit, stage, f)?);
    let [dot, bar] = witness_at(stage, last)?;
    for &m in scene.iter().chain(&panel).chain(&[bar, dot]) {
        stage.add_to_scene(m)?;
    }
    Ok(Opening {
        bright: Bright::new(curve, label, last),
        dot,
        bar,
        panel,
    })
}

impl SceneConstruct for TheShrinkingRegion {
    fn name(&self) -> &str {
        "02_the_shrinking_region"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let f = facts().map_err(fail)?;
        let Opening {
            bright: mut b,
            dot,
            bar,
            panel,
        } = opening(&kit, stage, &f)?;

        // 02_1: the panel clears; the zoom window on the last tenth; the
        // strip where the curve is above one half, and x = 1 inside it
        let line = say(&kit, stage, "02_1")?;
        fade_out_all(stage, &panel, 0.6)?;
        let mut zoom = ZOOMS[0];
        // what the window shows at every stretch: its border, the half line,
        // the limit (flat, the hollow mark at x = 1, the point (1, 1)) and its
        // right end, x = 1
        let window = lens(zoom);
        let border = stage.arena_mut().add(window.frame.border(Palette::dim())?);
        let half_mark = window.dashed(stage, (window.frame.x.0, 0.5), (1.0, 0.5), moving())?;
        let [flat_part, hollow, point] = window.limit(stage)?;
        let right_end = kit.text(
            stage,
            &fact(&f, "C1.domain.right")?,
            22.0,
            Palette::dim(),
            v(window.p(1.0, 0.0)[0], ENDS_Y),
        )?;
        let fixed_parts = [border, half_mark, flat_part, hollow, point, right_end];
        let mut view = View::build(&kit, stage, &f, b.n, zoom)?;
        let mut glass = magnifier(stage, zoom)?;
        let mut ins = glass.to_vec();
        ins.extend(fixed_parts);
        ins.extend(&view.shapes[..3]);
        ins.extend(view.labels);
        fade_in_all(stage, &ins, 0.8)?;
        line.wait_for(stage, STRIP_AT)?;
        let above_legend = kit.tex_left(
            stage,
            ABOVE_LEGEND,
            28.0,
            moving(),
            &[],
            v(LENS_CENTER[0] - LENS_W / 2.0, LEGEND_Y[0]),
        )?;
        let mut strip_ins = vec![view.shapes[3], above_legend];
        if let Some(marks) = view.marks {
            strip_ins.extend(&marks[..2]);
        }
        fade_in_all(stage, &strip_ins, 0.6)?;
        line.wait_for(stage, BELONGS_AT)?;
        glow(&kit, stage, v(window.p(1.0, 0.0)[0], ABOVE_Y), 0.18)?;

        // 02_2: at x = 1 the curve and the limit are both 1; the gap is zero
        let line = say(&kit, stage, "02_2")?;
        line.wait_for(stage, BOTH_AT)?;
        glow(&kit, stage, window.p(1.0, 1.0), 0.2)?;
        let one = fact(&f, "C1.endpoint.height")?;
        let at_one = format!(
            "{AT_ONE} {one}, {LIMIT} {}, {GAP} {}",
            fact(&f, "C1.limit.at_one")?,
            fact(&f, "C2.gap_at_one")?
        );
        let at_one = kit.text_left(
            stage,
            &at_one,
            20.0,
            Palette::ink(),
            v(window.frame.left() + 0.15, window.frame.top() - 0.25),
        )?;
        fade_in_all(stage, &[at_one], 0.5)?;

        // 02_3: the gap is at least one half from the crossing to just short
        // of 1; the same strip on the main plot, under its axis
        let line = say(&kit, stage, "02_3")?;
        line.wait_for(stage, STARTS_AT)?;
        let gap_legend = kit.tex_left(
            stage,
            GAP_LEGEND,
            28.0,
            gap_color(),
            &[],
            v(LENS_CENTER[0] - LENS_W / 2.0, LEGEND_Y[1]),
        )?;
        let mut on_main = main_strip(stage, b.n)?;
        let mut strip_ins = vec![view.shapes[4], gap_legend, on_main];
        if let Some(marks) = view.marks {
            strip_ins.extend(&marks[2..]);
        }
        fade_in_all(stage, &strip_ins, 0.6)?;

        // 02_4: the power grows; the strip narrows on both plots
        say(&kit, stage, "02_4")?;
        fade_out_all(stage, &[at_one], 0.3)?;
        let mut readout = width_readout(&kit, stage, &f, b.n)?;
        fade_in_all(stage, &[readout], 0.3)?;
        for &n in &MORE_POWERS {
            let x = half_height(n).map_err(fail)?;
            let (floor, top) = gap_span(x, n)?;
            let next = View::build(&kit, stage, &f, n, zoom)?;
            let next_strip = main_strip(stage, n)?;
            let next_readout = width_readout(&kit, stage, &f, n)?;
            let mut with = view.morph_to(stage, &next, 0.6)?;
            with.push(stage.prepare(move_to(dot, PLOT.p(x, top), 0.6)?)?);
            with.push(stage.prepare(move_to(bar, PLOT.p(x, f64::midpoint(floor, top)), 0.6)?)?);
            with.push(stage.prepare(fmn::animation::replacement_transform(on_main, next_strip).rt(0.6))?);
            let out = fade_out(stage.arena_mut(), readout, ORIGIN, 1.0)?;
            with.push(stage.prepare(out.rt(0.3))?);
            let inn = fade_in(stage.arena_mut(), next_readout, ORIGIN, 1.0)?;
            with.push(stage.prepare(inn.rt(0.6))?);
            step(&kit, stage, &mut b, n, 0.6, with)?;
            stage.scene_mut().bring_to_front(&[bar, dot])?;
            view = next;
            on_main = next_strip;
            readout = next_readout;
        }
        // narrower than a pixel of the final picture: mark where, not how wide
        let px = fact(&f, "C2.pixels.plot_4k")?;
        let from = fact(&f, "C2.subpixel_from")?;
        let px_whole: u32 = px.parse().map_err(|_| fail(format!("C2.pixels.plot_4k is {px}")))?;
        let notes = [
            kit.text_left(
                stage,
                &format!("{PIXELS_A} {} {PIXELS_B}", thousands(px_whole)),
                20.0,
                Palette::dim(),
                v(READ_X, READ_Y[1]),
            )?,
            kit.text_left(
                stage,
                &format!("{PIXELS_C} {from} {PIXELS_D}"),
                20.0,
                Palette::dim(),
                v(READ_X, READ_Y[2]),
            )?,
        ];
        let here = pointer(stage)?;
        fade_in_lagged(stage, &[here, notes[0], notes[1]], 1.0, 0.3)?;

        // 02_5: the prediction; the answer stays hidden while the ring drains
        say(&kit, stage, "02_5")?;
        let ask = kit.text(stage, "?", 60.0, Palette::hero(), RING_AT)?;
        fade_in_all(stage, &[ask], 0.3)?;
        think_pause(&kit, stage, RING_AT, RING_R)?;

        // 02_6: stretch the window ten, a hundred, a thousand times
        let line = say(&kit, stage, "02_6")?;
        fade_out_all(stage, &[ask, here], 0.3)?;
        for (&m, at) in ZOOMS[1..].iter().zip([HUNDRED_AT, THOUSAND_AT]) {
            line.wait_for(stage, at)?;
            let next = View::build(&kit, stage, &f, b.n, m)?;
            let next_glass = magnifier(stage, m)?;
            let mut anims = view.morph_to(stage, &next, 0.8)?;
            anims.extend(magnify(stage, glass, next_glass, 0.8)?);
            stage.play_prepared(anims)?;
            view = next;
            glass = next_glass;
            zoom = m;
        }

        // 02_7: power and stretch grown together look almost the same; the
        // width shrinks like ln 2/n and never vanishes
        let line = say(&kit, stage, "02_7")?;
        let mut leaving = glass.to_vec();
        leaving.extend([readout, notes[0], notes[1]]);
        fade_out_all(stage, &leaving, 0.4)?;
        // only the window resets, to n = 10 at ×10 (the main plot stays at
        // x^1000); then the power and the stretch go up tenfold, then again
        let mut rows = Vec::new();
        for (i, (&m, at)) in ZOOMS.iter().zip([RESTART_AT, RAISE_AT, AGAIN_AT]).enumerate() {
            line.wait_for(stage, at)?;
            let next = View::build(&kit, stage, &f, m, m)?;
            let share = fact(&f, &format!("C2.share.n{m}.3dp"))?;
            let row = kit.text_left(
                stage,
                &format!("n = {m}, {STRETCHED}{m}: {FILLS} {share} {OF_WINDOW}"),
                20.0,
                Palette::ink(),
                v(READ_X, READ_Y[i]),
            )?;
            let mut anims = view.morph_to(stage, &next, 0.7)?;
            let inn = fade_in(stage.arena_mut(), row, ORIGIN, 1.0)?;
            anims.push(stage.prepare(inn.rt(0.5))?);
            stage.play_prepared(anims)?;
            view = next;
            rows.push(row);
        }
        line.wait_for(stage, SHRINKS_AT)?;
        // short enough to read in 02_7's last sentence: the exact width, that
        // it is never zero, and the limit (the two-sided bounds behind it are
        // in model.rs, oracle.py and the README, not on screen)
        let exact = format!(
            "w(n) = 1 − 2^(−1/n) > 0; n·w(n) → ln 2 ≈ {}. This is a limit, not a finite equality.",
            fact(&f, "C2.ln2.4dp")?
        );
        let lower = precisely(&kit, stage, &exact)?;
        fade_in_all(stage, &[lower], 0.6)?;

        // 02_8: the moving point stays inside the gap strip, at its left end
        let line = say(&kit, stage, "02_8")?;
        fade_out_all(stage, &[lower], 0.4)?;
        line.wait_for(stage, INSIDE_AT)?;
        let x = half_height(b.n).map_err(fail)?;
        let plot = lens(zoom);
        glow(&kit, stage, plot.p(x, 0.5), 0.2)?;
        glow(&kit, stage, v(plot.p(x, 0.0)[0], GAP_Y), 0.15)?;

        // 02_9: a picture cannot settle it; a guarantee for every point can
        let line = say(&kit, stage, "02_9")?;
        line.wait_for(stage, NEED_AT)?;
        let summary = [
            kit.text_left(stage, EVERY_POINT, 26.0, Palette::ink(), v(READ_X, READ_Y[0]))?,
            kit.text_left(stage, WHO_FIRST, 22.0, Palette::dim(), v(READ_X, READ_Y[1] - 0.05))?,
        ];
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &rows {
            let out = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(out.rt(0.4))?);
        }
        stage.play_prepared(anims)?;
        fade_in_lagged(stage, &summary, 1.0, 0.4)?;
        kit.hold(stage)?;
        kit.music_breath(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every phrase a picture waits for is said once, in the line that
    /// waits for it, and the stretches in the order the window takes them.
    #[test]
    fn the_pictures_wait_on_words_the_script_says() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let text = |id: &str| script.text(id).unwrap();
        let waits = [
            ("02_1", STRIP_AT),
            ("02_1", BELONGS_AT),
            ("02_2", BOTH_AT),
            ("02_3", STARTS_AT),
            ("02_6", HUNDRED_AT),
            ("02_6", THOUSAND_AT),
            ("02_7", RESTART_AT),
            ("02_7", RAISE_AT),
            ("02_7", AGAIN_AT),
            ("02_7", SHRINKS_AT),
            ("02_8", INSIDE_AT),
            ("02_9", NEED_AT),
        ];
        for (id, phrase) in waits {
            assert_eq!(text(id).matches(phrase).count(), 1, "{id} says {phrase:?} once");
        }
        let six = text("02_6");
        assert!(six.find(HUNDRED_AT).unwrap() < six.find(THOUSAND_AT).unwrap());
        // 02_7 opens by resetting the window, naming it, before it raises
        // anything: the window leaves n = 1000 for n = 10 on those words, and
        // the second tenfold step lands on "then again"
        let seven = text("02_7");
        let at = |p: &str| seven.find(p).unwrap();
        assert_eq!(at(RESTART_AT), 0);
        assert!(at(RESTART_AT) < at(RAISE_AT) && at(RAISE_AT) < at(AGAIN_AT) && at(AGAIN_AT) < at(SHRINKS_AT));
    }

    /// What the window's label says is what it draws. It is as wide as the
    /// plot, so [1 − 1/m, 1] across it is that stretch of x drawn m times
    /// wider than on the plot; its heights are drawn at the labelled share of
    /// the plot's (`C2.lens.heights`).
    #[test]
    fn the_labels_match_the_geometry() {
        let f = facts().unwrap();
        let heights = f.get("C2.lens.heights").unwrap();
        let (num, den) = heights.split_once('/').unwrap();
        let share = num.parse::<f64>().unwrap() / den.parse::<f64>().unwrap();
        for (i, m) in ZOOMS.into_iter().enumerate() {
            assert_eq!(f.get(&format!("C2.zoom.{i}")), Some(m.to_string().as_str()));
            let window = lens(m).frame;
            let on_plot = PLOT.frame.width * (window.x.1 - window.x.0);
            assert!((window.width / on_plot - f64::from(m)).abs() < 1e-9, "×{m} is not ×{m}");
            assert!(
                (window.height / PLOT.frame.height - share).abs() < 1e-12,
                "heights ×{heights}"
            );
        }
        assert_eq!(f.get("C2.restart_power"), Some(ZOOMS[0].to_string().as_str()));
    }

    /// The window shows [1 − 1/m, 1] at each stretch m, so the strip for
    /// n = m fills m·(1 − x_m) of it, about the same share each time and
    /// below ln 2; the opening's strip (n = 50 at ×10) is wide enough to show
    /// its ends, n = 1000 at ×10 is not, and at ×1000 its ends return.
    #[test]
    fn the_window_is_true_to_scale() {
        for m in ZOOMS {
            let w = lens(m);
            assert!((w.frame.x.1 - w.frame.x.0 - 1.0 / f64::from(m)).abs() < 1e-12);
            let x = half_height(m).unwrap();
            let share = (w.p(1.0, 0.0)[0] - w.p(x, 0.0)[0]) / LENS_W;
            assert!((share - f64::from(m) * crate::model::width(m).unwrap()).abs() < 1e-9);
            assert!(share > 0.66 && share < std::f64::consts::LN_2, "m = {m}: {share}");
        }
        let ends = |plot: &Plot, n: u32| {
            let x = half_height(n).unwrap();
            plot.p(1.0, 0.0)[0] - plot.p(x, 0.0)[0] >= 6.0 * MARK_R
        };
        let last = MORE_POWERS[MORE_POWERS.len() - 1];
        assert!(ends(&lens(ZOOMS[0]), POWERS[POWERS.len() - 1]));
        assert!(!ends(&lens(ZOOMS[0]), last));
        assert!(ends(&lens(ZOOMS[ZOOMS.len() - 1]), last));
    }

    /// The magnifier's box on the main plot is the window at its true
    /// width: at ×1000 it spans about 1.3 pixels of the 4K picture.
    #[test]
    fn the_magnifier_is_true_width() {
        for m in ZOOMS {
            let w = PLOT.p(1.0, 0.0)[0] - PLOT.p(1.0 - 1.0 / f64::from(m), 0.0)[0];
            assert!((w - PLOT.frame.width / f64::from(m)).abs() < 1e-9, "m = {m}");
        }
        let px_per_unit = f64::from(crate::facts::FINAL_PIXELS_ACROSS) / fmn::core::constants::FRAME_WIDTH;
        let narrowest = PLOT.frame.width / f64::from(ZOOMS[ZOOMS.len() - 1]) * px_per_unit;
        assert!(narrowest > 1.0 && narrowest < 2.0, "{narrowest} pixels");
    }
}
