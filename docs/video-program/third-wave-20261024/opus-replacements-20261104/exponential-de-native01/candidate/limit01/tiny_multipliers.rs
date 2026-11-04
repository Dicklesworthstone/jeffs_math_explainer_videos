//! Native staged small-multiplier chapters. Type-checked at the pin; never rendered.
//!
//! Two chapters share these pictures. 04 earns the exact coefficient, and 05 proves the limit from it. The boundary is
//! the teaching's own, and 05 needs it: one chapter would pass the kit's raw-output limit at 4K60.
//!
//! Each cue has one function, in film order, and each function begins with its own `say`.
//! The rate chapter's four steps become a product, the product expands into choices, and the same green ceilings
//! the series chapter measured carry the limit, at both signs of the input.
use crate::limit_model::{
    AIM, BAR_COUNTS, BAR_TOP, CUT, EARLY_CUT, EDGE_COUNTS, EDGE_INPUT, FILL_COUNTS, FILL_FAR, FILL_NEAR, FILL_TOP,
    GRAPH_LOW, GRAPH_SAMPLES, HEAD_COUNTS, STRIP_CELLS, SWEEP_CELLS, TABLE_COUNTS, WALK_COUNTS, WARNING_COLUMN,
    WARNING_COUNTS, chain, choices, column_label, distance, e_value, edge_base, edge_value, endpoint, gap, graph_x,
    head_gaps, rate_one_label, strip_coefficient, sweep_label, tail_print, term, total_label, up_label, walk, ways,
};
use crate::phrase::until_phrase;
use crate::product_model::e_label;
use crate::rate_model::proportional_steps;
use crate::series_model::{RADIUS, ZOOM, coefficient, term_size};
use explainer_kit::{Kit, Palette, Timed, ink_style, lagged, rect, stroke_style, tween, v};
use fmn::prelude::*;

pub struct TinyMultipliers {
    pub kit: Kit,
}
pub struct WholeSum {
    pub kit: Kit,
}

const TITLE: &str = "Why the steps reach E";
const WHOLE_TITLE: &str = "Does the whole sum arrive?";
const BIGGER: &str = "bigger input: a later cut, more cells";
const TITLE_Y: f64 = 3.35;
/// The left column of the first two cues, beside the rate chapter's panel.
const LEFT_X: f64 = -3.2;

// 04_1 and 04_2: the same panel, at the same place and scale, as the right-hand drawing in the rate chapter.
const STEP_LEFT: f64 = 0.5;
const STEP_WIDTH: f64 = 4.5;
const STEP_BOTTOM: f64 = -1.8;
const STEP_HEIGHT: f64 = 3.1;
const STEP_TOP: f64 = 3.0;
const STRIP_BOTTOM: f64 = -2.75;
const STRIP_HEIGHT: f64 = 0.5;

const ADD_RISE: &str = r"h+\frac14h";
const AS_MULTIPLY: &str = r"=\left(1+\frac14\right)h=\frac54h";
const FOUR_STEPS: &str = r"1\cdot\frac54\cdot\frac54\cdot\frac54\cdot\frac54=\left(\frac54\right)^4";
const RATE_RULE: &str = r"y'=xy";
const CELL_MULTIPLIER: &str = r"1+\frac{x}{n}";
const EACH_CELL: &str = r"\text{each cell}\ \times\left(1+\frac{x}{n}\right)";
const POWER: &str = r"\left(1+\frac{x}{n}\right)^n";
const RATE_ONE: &str = r"x=1:\quad\left(1+\frac1n\right)^n";

// 04_3 and 05_8: one graph of E, wide enough for both signs.
const GRAPH_LEFT: f64 = -5.4;
const GRAPH_X_UNIT: f64 = 2.0;
const GRAPH_BASE: f64 = -3.0;
const GRAPH_Y_UNIT: f64 = 1.1;
const GRAPH_Y_TOP: f64 = 5.0;
const GRAPH_RIGHT_X: f64 = 5.35;
const NOT_PROOF: &str = "Near is still not a proof";
const WHY_LIMIT: &str = "Why should that limit be our series?";

// 04_4: the sixteen choices, in five columns.
const OFFER: &str = r"\text{each cell: keep}\ 1,\ \text{or take}\ \frac{x}{n}";
const COLUMN_X0: f64 = -4.4;
const COLUMN_PITCH: f64 = 2.2;
const ROW_LABEL_X: f64 = -6.05;
const POWER_Y: f64 = 1.45;
const MINI_TOP: f64 = 1.05;
const MINI_PITCH: f64 = 0.38;
const MINI_CELL: f64 = 0.28;
const MINI_HEIGHT: f64 = 0.26;
const WAYS_Y: f64 = -1.45;
const COEFFICIENT_Y: f64 = -2.0;
const SERIES_Y: f64 = -2.55;
const STRIP_POWERS: [&str; 5] = ["1", "x", "x^2", "x^3", "x^4"];
const WORKED: &str = r"x^2:\quad 6\cdot\frac{x}{4}\cdot\frac{x}{4}=\frac{6}{16}x^2";

// 04_5 and 04_6: the same count with letters, and with the worked numbers beside it.
const LETTER_X: f64 = -2.6;
const NUMBER_X: f64 = 3.9;
const PICK_NAME: &str = r"\text{pick}\ k\ \text{cells out of}\ n";
const PICK_CELL: f64 = 0.42;
const PICK_HEIGHT: f64 = 0.32;
const PICK_Y: f64 = 2.3;
const PICKED_CELLS: [usize; 2] = [0, 2];
const PICKS: &str = r"n(n-1)(n-2)\cdots(n-k+1)";
const PICKS_WORKED: &str = r"n=4,\ k=2:\quad 4\cdot 3=12";
const RECOUNT: &str = r"\text{each set counted}\ k!\ \text{times}";
const RECOUNT_WORKED: &str = r"2!=2";
const CHOOSE: &str = r"\binom{n}{k}=\frac{n(n-1)\cdots(n-k+1)}{k!}";
const CHOOSE_WORKED: &str = r"\frac{4\cdot 3}{2!}=6";
const TERM_SHARED: &str =
    r"\binom{n}{k}\left(\frac{x}{n}\right)^k=\frac{x^k}{k!}\cdot\frac{n}{n}\cdot\frac{n-1}{n}\cdots\frac{n-k+1}{n}";
const TERM_CHAIN: &str =
    r"=\frac{x^k}{k!}\cdot 1\cdot\left(1-\frac1n\right)\left(1-\frac2n\right)\cdots\left(1-\frac{k-1}{n}\right)";
const CHAIN_WORKED: &str = r"n=4,\ k=2:\quad\frac{x^2}{2!}\cdot 1\cdot\frac34=\frac{6}{16}x^2";
const CHAIN_ZERO: &str = r"k>n:\quad\text{the factor}\ 1-\frac{n}{n}=0";

// 05_1 and 05_2: every column of the chain, far past the ones that have settled.
const CHAIN_NAME: &str = r"\text{chain}=1\cdot\left(1-\frac1n\right)\cdots\left(1-\frac{k-1}{n}\right)";
const FULL: &str = "full: the series coefficient";
const COLUMN_NAME: &str = "column k";
const FILL_LEFT: f64 = -5.4;
const FILL_PITCH: f64 = 0.108;
const FILL_WIDTH: f64 = 0.08;
const FILL_BASE: f64 = -2.2;
const FILL_UNIT: f64 = 3.2;
const READ_Y: f64 = 2.45;
const MUST_ARRIVE: &str = "Must the whole sum arrive?";
const NOT_ENOUGH: &str = "Not enough by itself";
const WARNING_SUM: &str = r"n\ \text{terms, each}\ \frac1n";
const TOWER_X: f64 = 6.3;
const OUTWARD: &str = "the weight moves out";
const TOTAL_ONE: &str = "total: 1";

// 05_3 to 05_6: the series chapter's bars and ceilings, with room below for the signs that flip.
const BAR_X0: f64 = -5.7;
const BAR_PITCH: f64 = 0.6;
const BAR_WIDTH: f64 = 0.42;
const BAR_BASE: f64 = -0.6;
const BAR_UNIT: f64 = 1.3;
const BAR_INDEX_Y: f64 = -0.85;
const BAR_INDEX_LOW_Y: f64 = -2.85;
/// Above the tallest ceiling, so the input and the count stay clear when odd bars point down.
const NAME_Y: f64 = 2.1;
const MIN_BAR: f64 = 0.004;
const CARD_X: f64 = 3.75;
const BOUNDED: &str = r"\left|\binom{n}{k}\left(\frac{x}{n}\right)^k\right|\leq\frac{|x|^k}{k!}";
const CHAIN_RANGE: &str = r"k\leq n:\quad 0\leq\text{chain}\leq 1";
const CHAIN_EMPTY: &str = r"k>n:\quad\text{chain}=0";
const NO_CEILING: &str = "The warning sum has no such ceilings";
const HARMONIC: &str = r"1+\frac12+\frac13+\frac14+\cdots\ \text{has no limit}";
const DISTANCE: &str = r"\left|E(x)-\left(1+\frac{x}{n}\right)^n\right|";
const GAPS: &str = r"\leq\sum_k\frac{|x|^k}{k!}\left(1-\text{chain}\right)";
const CUT_HEIGHT: f64 = 2.0;
const INSET_X0: f64 = 1.55;
const INSET_PITCH: f64 = 0.72;
const INSET_WIDTH: f64 = 0.5;
const INSET_BASE: f64 = -3.2;
const SIGNS: &str = r"\text{signs}:\ +,\ -,\ +,\ -,\ \ldots";
const SAME_SIZE: &str = r"\left|\text{term at}\ {-x}\right|=\left|\text{term at}\ x\right|";

// 05_7: the edge table.
const CELLS_X: f64 = -5.0;
const BASE_X: f64 = -3.0;
const VALUE_X: f64 = -0.6;
const NOTE_X: f64 = 4.1;
const ROW_Y0: f64 = 1.75;
const ROW_PITCH: f64 = 0.48;
const NOT_POSITIVE: &str = "multiplier not positive:";
const NO_DECAY: &str = "no decay reading";
const POSITIVE: &str = "multiplier positive: decay";
const SAME_ALGEBRA: &str = "Same chain, same ceilings, for every n";
const NO_CLIMB: &str = "No steady climb needed, no logarithm used";

const LIMIT: &str = r"\lim_{n\to\infty}\left(1+\frac{x}{n}\right)^n=E(x)";
const EVERY_REAL: &str = "for every real x";
const SPECIAL_CASE: &str = r"x=1:\quad\lim_{n\to\infty}\left(1+\frac1n\right)^n=e";
const TIME_SWAP: &str = r"\text{rate}\ x\ \text{for time}\ 1\ =\ \text{rate}\ 1\ \text{for time}\ x";
const SECOND_DEBT: &str = "Why the stepped approximations reach E";

fn step_point(t: f64, y: f64) -> Vec3 {
    v(STEP_LEFT + STEP_WIDTH * t, STEP_BOTTOM + STEP_HEIGHT * y / STEP_TOP)
}
fn graph_point(x: f64, y: f64) -> Vec3 {
    v(
        GRAPH_LEFT + GRAPH_X_UNIT * (x - GRAPH_LOW),
        GRAPH_BASE + GRAPH_Y_UNIT * y,
    )
}
fn column_x(k: usize) -> f64 {
    COLUMN_X0 + COLUMN_PITCH * k as f64
}
fn fill_x(k: usize) -> f64 {
    FILL_LEFT + FILL_PITCH * k as f64
}
fn bar_x(k: usize) -> f64 {
    BAR_X0 + BAR_PITCH * k as f64
}
fn inset_x(k: usize) -> f64 {
    INSET_X0 + INSET_PITCH * (k - EARLY_CUT - 1) as f64
}
fn inset_unit() -> f64 {
    BAR_UNIT * f64::from(ZOOM)
}
/// The line between column `cut` and the next.
fn cut_x(cut: usize) -> f64 {
    bar_x(cut) + BAR_PITCH / 2.0
}

fn reveal(stage: &mut Stage<'_>, mob: Mob) -> fmn::Result<()> {
    let a = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
    stage.play(a.rt(0.35))?;
    Ok(())
}
fn reveal_all(stage: &mut Stage<'_>, mobs: &[Mob], seconds: f64) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        animations.push(stage.prepare(a.rt(seconds))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}
/// Fade these out. Updaters go first, so a live bar fades instead of being redrawn at full opacity.
fn clear(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        animations.push(stage.prepare(a.rt(0.3))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}
fn line(stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb, width: f64) -> fmn::Result<Mob> {
    Ok(stage
        .arena_mut()
        .add(Line::new(a, b).style(stroke_style(color, width)).build()?))
}
fn dot(stage: &mut Stage<'_>, at: Vec3, color: Srgb) -> Mob {
    let marker = Circle::new().radius(0.09).style(ink_style(color)).build().moved_to(at);
    stage.arena_mut().add(marker)
}
fn path(stage: &mut Stage<'_>, points: Vec<Vec3>, color: Srgb, width: f64) -> Mob {
    stage
        .arena_mut()
        .add(polyline(points).style(stroke_style(color, width)).build())
}
fn glide(stage: &mut Stage<'_>, mob: Mob, to: Vec3, seconds: f64) -> fmn::Result<Box<dyn Animation>> {
    let a = mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(seconds),
            ..AnimateArgs::default()
        })?
        .move_to(to, ORIGIN)?;
    stage.prepare(a)
}
fn recolor(stage: &mut Stage<'_>, mob: Mob, color: Srgb, seconds: f64) -> fmn::Result<Box<dyn Animation>> {
    let a = mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(seconds),
            ..AnimateArgs::default()
        })?
        .set_color_with_opacity(color, 1.0)?;
    stage.prepare(a)
}
/// set_stroke, not set_color: a recolour also sets fill alpha and would fill the open path.
fn dim_path(stage: &mut Stage<'_>, mob: Mob, seconds: f64) -> fmn::Result<Box<dyn Animation>> {
    let a = mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(seconds),
            ..AnimateArgs::default()
        })?
        .set_stroke(Some(Palette::dim()), None, None)?;
    stage.prepare(a)
}
fn flash(stage: &mut Stage<'_>, mobs: &[Mob], scale: f64, seconds: f64) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fmn::animation::indicate(stage.arena_mut(), m, scale, None)?;
        animations.push(stage.prepare(a.rt(seconds))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}
/// Old words out and new words in: never morph a text whose words stay.
fn swap(stage: &mut Stage<'_>, old: Mob, new: Mob, seconds: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let out = fade_out(stage.arena_mut(), old, ORIGIN, 1.0)?;
    let into = fade_in(stage.arena_mut(), new, ORIGIN, 1.0)?;
    Ok(vec![stage.prepare(out.rt(seconds))?, stage.prepare(into.rt(seconds))?])
}
/// A filled bar standing on its base. Below the smallest height it can draw, it stays a thin line.
fn bar(stage: &mut Stage<'_>, x: f64, base: f64, width: f64, height: f64, color: Srgb) -> fmn::Result<Mob> {
    let r = rect(v(x - width / 2.0, base), width, height.max(MIN_BAR), ink_style(color))?;
    Ok(stage.arena_mut().add(r))
}
/// An outlined ceiling standing on the same base.
fn ceiling(stage: &mut Stage<'_>, x: f64, base: f64, width: f64, height: f64, color: Srgb) -> fmn::Result<Mob> {
    let r = rect(
        v(x - width / 2.0, base),
        width,
        height.max(MIN_BAR),
        stroke_style(color, 2.0),
    )?;
    Ok(stage.arena_mut().add(r))
}
/// Morph each handle into its new shape, on the same clock. The handles stay the live objects.
fn morph_all(stage: &mut Stage<'_>, moves: &[(Mob, Mob)], seconds: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &(from, to) in moves {
        animations.push(stage.prepare(Transform::new(from, to).rt(seconds))?);
    }
    Ok(animations)
}

/// The rate chapter's proportional drawing, at n steps, from its own model.
fn step_walk(stage: &mut Stage<'_>, n: u32, color: Srgb) -> Mob {
    let points: Vec<Vec3> = proportional_steps(n).iter().map(|&(t, y)| step_point(t, y)).collect();
    path(stage, points, color, 3.0)
}
/// One unit of time, cut into n cells. Past the point where a cell can be drawn, it is a solid band.
fn strip(stage: &mut Stage<'_>, n: u32) -> fmn::Result<Vec<Mob>> {
    if n > 64 {
        let band = rect(
            v(STEP_LEFT, STRIP_BOTTOM),
            STEP_WIDTH,
            STRIP_HEIGHT,
            ink_style(Palette::dim()),
        )?;
        return Ok(vec![stage.arena_mut().add(band)]);
    }
    let width = STEP_WIDTH / f64::from(n);
    let mut cells = Vec::new();
    for i in 0..n {
        let cell = rect(
            v(STEP_LEFT + width * f64::from(i), STRIP_BOTTOM),
            width,
            STRIP_HEIGHT,
            stroke_style(Palette::dim(), 1.5),
        )?;
        cells.push(stage.arena_mut().add(cell));
    }
    Ok(cells)
}
fn cell_center(n: u32, i: u32) -> Vec3 {
    let width = STEP_WIDTH / f64::from(n);
    v(
        STEP_LEFT + width * (f64::from(i) + 0.5),
        STRIP_BOTTOM + STRIP_HEIGHT / 2.0,
    )
}

/// What 04_1 leaves on screen for 04_2.
struct StepPanel {
    axes: Vec<Mob>,
    walk: Mob,
    cells: Vec<Mob>,
    cell_labels: Vec<Mob>,
    working: Vec<Mob>,
    end_dot: Mob,
    end_label: Mob,
}

/// 04_1: the drawing rule, read again. A step adds a fraction of the height, so it multiplies the height.
fn steps_multiply(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<StepPanel> {
    k.say(stage, "04_1")?;
    let title = k.text(stage, TITLE, 35.0, Palette::ink(), v(0.0, TITLE_Y))?;
    let mut axes = vec![
        line(stage, step_point(0.0, 0.0), step_point(1.0, 0.0), Palette::dim(), 2.0)?,
        line(
            stage,
            step_point(0.0, 0.0),
            step_point(0.0, STEP_TOP),
            Palette::dim(),
            2.0,
        )?,
    ];
    for y in 0..=3 {
        axes.push(k.text(
            stage,
            &y.to_string(),
            19.0,
            Palette::dim(),
            v(STEP_LEFT - 0.25, step_point(0.0, f64::from(y))[1]),
        )?);
    }
    axes.push(dot(stage, step_point(0.0, 1.0), Palette::ink()));
    let walk4 = step_walk(stage, 4, Palette::blue());
    let mut shown = axes.clone();
    shown.extend([title, walk4]);
    reveal_all(stage, &shown, 0.5)?;

    // One step, taken apart: the height it read, its width, and the rise they make.
    k.until_sentence(stage, "04_1", 1)?;
    let steps = proportional_steps(4);
    let ((t1, h), (t2, top)) = (steps[1], steps[2]);
    let mut working = vec![
        line(stage, step_point(t1, 0.0), step_point(t1, h), Palette::blue(), 2.5)?,
        k.tex(
            stage,
            "h",
            26.0,
            Palette::blue(),
            &[],
            v(step_point(t1, 0.0)[0] - 0.2, step_point(t1, h / 2.0)[1]),
        )?,
        line(stage, step_point(t1, h), step_point(t2, h), Palette::dim(), 2.0)?,
        k.tex(
            stage,
            r"\text{width}\ \frac14",
            21.0,
            Palette::dim(),
            &[],
            v(step_point((t1 + t2) / 2.0, h)[0], step_point(t1, h)[1] - 0.32),
        )?,
        line(stage, step_point(t2, h), step_point(t2, top), Palette::gold(), 4.0)?,
        k.tex(
            stage,
            r"\frac14h",
            24.0,
            Palette::gold(),
            &[],
            v(step_point(t2, h)[0] + 0.5, step_point(t2, h)[1] - 0.1),
        )?,
        k.tex(stage, ADD_RISE, 30.0, Palette::ink(), &[], v(LEFT_X, 1.4))?,
    ];
    reveal_all(stage, &working, 0.4)?;

    k.until_sentence(stage, "04_1", 2)?;
    let multiply = k.tex(stage, AS_MULTIPLY, 30.0, Palette::gold(), &[], v(LEFT_X, 0.5))?;
    reveal(stage, multiply)?;
    working.push(multiply);

    // Every cell does the same thing, and a dot walks the steps as each cell lights.
    k.until_sentence(stage, "04_1", 3)?;
    let cells = strip(stage, 4)?;
    let mut cell_labels = Vec::new();
    for i in 0..4 {
        cell_labels.push(k.tex(stage, r"\times\frac54", 22.0, Palette::gold(), &[], cell_center(4, i))?);
    }
    let mut row = cells.clone();
    row.extend(cell_labels.iter().copied());
    reveal_all(stage, &row, 0.35)?;
    let hopper = dot(stage, step_point(0.0, 1.0), Palette::gold());
    reveal(stage, hopper)?;
    for (i, &(t, y)) in steps.iter().enumerate().skip(1) {
        let lit = fmn::animation::indicate(stage.arena_mut(), cell_labels[i - 1], 1.2, None)?;
        let hop = vec![
            glide(stage, hopper, step_point(t, y), 0.35)?,
            stage.prepare(lit.rt(0.35))?,
        ];
        stage.play_prepared(hop)?;
    }

    // Four multipliers, and the number the rate chapter printed.
    k.until_sentence(stage, "04_1", 4)?;
    let product = k.tex(stage, FOUR_STEPS, 28.0, Palette::gold(), &[], v(LEFT_X, -0.55))?;
    let (t_end, y_end) = steps[4];
    let end_label = k.text(
        stage,
        &rate_one_label(4),
        23.0,
        Palette::gold(),
        v(step_point(t_end, y_end)[0] + 0.5, step_point(t_end, y_end)[1]),
    )?;
    let value = k.text(
        stage,
        &format!("= {}", rate_one_label(4)),
        28.0,
        Palette::gold(),
        v(LEFT_X, -1.3),
    )?;
    reveal_all(stage, &[product, end_label, value], 0.35)?;
    working.extend([product, value]);
    k.hold(stage)?;
    Ok(StepPanel {
        axes,
        walk: walk4,
        cells,
        cell_labels,
        working,
        end_dot: hopper,
        end_label,
    })
}

/// 04_2: the strip is one unit of time, the rate becomes x, and the rate-one endpoints fill a table on their way to e.
fn rate_and_cells(stage: &mut Stage<'_>, k: &Kit, panel: &mut StepPanel) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "04_2")?;
    clear(stage, &panel.working)?;
    let unit = k.text(
        stage,
        "one unit of time",
        23.0,
        Palette::dim(),
        v(STEP_LEFT + STEP_WIDTH / 2.0, STRIP_BOTTOM - 0.3),
    )?;
    let mut animations = lit_cells(stage, &panel.cells)?;
    let shown = fade_in(stage.arena_mut(), unit, ORIGIN, 1.0)?;
    animations.push(stage.prepare(shown.rt(0.6))?);
    stage.play_prepared(animations)?;
    let mut card = vec![unit];

    k.until_sentence(stage, "04_2", 1)?;
    let rule = k.tex(stage, RATE_RULE, 30.0, Palette::blue(), &[], v(LEFT_X, 2.1))?;
    reveal(stage, rule)?;
    card.push(rule);

    // The same cells, now in letters.
    k.until_sentence(stage, "04_2", 2)?;
    let each = k.tex(stage, EACH_CELL, 28.0, Palette::gold(), &[], v(LEFT_X, 1.25))?;
    let mut change = Vec::new();
    let mut letters = Vec::new();
    for (i, &old) in panel.cell_labels.iter().enumerate() {
        let new = k.tex(
            stage,
            CELL_MULTIPLIER,
            20.0,
            Palette::gold(),
            &[],
            cell_center(4, i as u32),
        )?;
        change.extend(swap(stage, old, new, 0.4)?);
        letters.push(new);
    }
    let shown = fade_in(stage.arena_mut(), each, ORIGIN, 1.0)?;
    change.push(stage.prepare(shown.rt(0.4))?);
    stage.play_prepared(change)?;
    panel.cell_labels = letters;
    card.push(each);

    k.until_sentence(stage, "04_2", 3)?;
    let power = k.tex(stage, POWER, 34.0, Palette::hero(), &[], v(LEFT_X, 0.35))?;
    reveal(stage, power)?;
    card.push(power);

    k.until_sentence(stage, "04_2", 4)?;
    let rate_one = k.tex(stage, RATE_ONE, 28.0, Palette::ink(), &[], v(LEFT_X, -0.55))?;
    let first = table_row(stage, k, TABLE_COUNTS[0], 0)?;
    reveal_all(stage, &[rate_one, first], 0.35)?;
    card.extend([rate_one, first]);

    // More cells: finer strip, finer drawing, higher endpoint. The old drawing greys, as it did in the rate chapter.
    k.until_sentence(stage, "04_2", 5)?;
    clear(stage, &panel.cell_labels)?;
    for (row, n) in TABLE_COUNTS.iter().copied().enumerate().skip(1) {
        let fresh = step_walk(stage, n, Palette::blue());
        let new_cells = strip(stage, n)?;
        let shown_row = table_row(stage, k, n, row)?;
        let steps = proportional_steps(n);
        let (t, y) = steps[n as usize];
        let new_label = k.text(
            stage,
            &rate_one_label(n),
            23.0,
            Palette::gold(),
            v(step_point(t, y)[0] + 0.5, step_point(t, y)[1]),
        )?;
        let mut moves = vec![
            dim_path(stage, panel.walk, 0.45)?,
            glide(stage, panel.end_dot, step_point(t, y), 0.45)?,
        ];
        moves.extend(swap(stage, panel.end_label, new_label, 0.45)?);
        for &m in &panel.cells {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            moves.push(stage.prepare(a.rt(0.45))?);
        }
        for m in new_cells.iter().copied().chain([fresh, shown_row]) {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            moves.push(stage.prepare(a.rt(0.45))?);
        }
        stage.play_prepared(moves)?;
        card.push(panel.walk);
        panel.walk = fresh;
        panel.cells = new_cells;
        panel.end_label = new_label;
        card.push(shown_row);
    }
    let limit = k.text(
        stage,
        &format!("e = {}", e_label()),
        25.0,
        Palette::hero(),
        v(LEFT_X, table_y(TABLE_COUNTS.len()) - 0.05),
    )?;
    reveal(stage, limit)?;
    card.push(limit);
    k.hold(stage)?;
    Ok(card)
}
/// The cells take the gold of their multipliers. set_stroke, not indicate: an outline must stay an outline.
fn lit_cells(stage: &mut Stage<'_>, cells: &[Mob]) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &m in cells {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.6),
                ..AnimateArgs::default()
            })?
            .set_stroke(Some(Palette::gold()), None, None)?;
        animations.push(stage.prepare(a)?);
    }
    Ok(animations)
}
fn table_y(row: usize) -> f64 {
    -1.1 - 0.36 * row as f64
}
fn table_row(stage: &mut Stage<'_>, k: &Kit, n: u32, row: usize) -> fmn::Result<Mob> {
    k.text(
        stage,
        &format!("{n} cells: {}", rate_one_label(n)),
        23.0,
        Palette::ink(),
        v(LEFT_X, table_y(row)),
    )
}

/// The fixed axes and the series curve, shared by 04_3 and 05_8.
struct Graph {
    axes: Vec<Mob>,
    curve: Mob,
    name: Mob,
}
fn graph(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<Graph> {
    let mut axes = vec![
        line(
            stage,
            graph_point(GRAPH_LOW, 0.0),
            graph_point(RADIUS, 0.0),
            Palette::dim(),
            2.0,
        )?,
        line(
            stage,
            graph_point(GRAPH_LOW, 0.0),
            graph_point(GRAPH_LOW, GRAPH_Y_TOP),
            Palette::dim(),
            2.0,
        )?,
        line(
            stage,
            graph_point(0.0, 0.0),
            graph_point(0.0, GRAPH_Y_TOP),
            Palette::faint(),
            1.5,
        )?,
        dot(stage, graph_point(0.0, 1.0), Palette::ink()),
    ];
    for x in [-3, -2, -1, 0, 1] {
        axes.push(k.tex(
            stage,
            &x.to_string(),
            21.0,
            Palette::dim(),
            &[],
            v(graph_point(f64::from(x), 0.0)[0], GRAPH_BASE - 0.28),
        )?);
    }
    for y in 1..=4 {
        axes.push(k.text(
            stage,
            &y.to_string(),
            21.0,
            Palette::dim(),
            v(GRAPH_LEFT - 0.25, graph_point(0.0, f64::from(y))[1]),
        )?);
    }
    let points: Vec<Vec3> = (0..=GRAPH_SAMPLES)
        .map(|j| graph_point(graph_x(j), e_value(graph_x(j))))
        .collect();
    let curve = path(stage, points, Palette::blue(), 3.0);
    let name = k.tex(stage, "E(x)", 27.0, Palette::blue(), &[], v(2.55, 1.75))?;
    Ok(Graph { axes, curve, name })
}

/// 04_3: the endpoints on the graph of the series, at both signs, and the question that picture cannot answer.
fn endpoints_on_the_graph(stage: &mut Stage<'_>, k: &Kit, panel: &StepPanel, card: &[Mob]) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "04_3")?;
    let mut leaving = card.to_vec();
    leaving.extend(panel.axes.iter().copied());
    leaving.extend(panel.cells.iter().copied());
    leaving.extend([panel.walk, panel.end_dot, panel.end_label]);
    clear(stage, &leaving)?;
    let drawn = graph(stage, k)?;
    let cells_note = k.text(
        stage,
        &format!("{SWEEP_CELLS} cells"),
        25.0,
        Palette::gold(),
        v(GRAPH_RIGHT_X, 2.3),
    )?;
    let mut shown = drawn.axes.clone();
    shown.extend([drawn.curve, drawn.name, cells_note]);
    reveal_all(stage, &shown, 0.4)?;
    // Live from here: the endpoint at the rate the tracker holds, starting at the table's rate of one.
    let input = stage.add_value_tracker(1.0);
    let marker = stage.always_redraw(move |s| {
        let x = s.tracker_value(input).unwrap_or(1.0);
        let d = Circle::new()
            .radius(0.09)
            .style(ink_style(Palette::gold()))
            .build()
            .moved_to(graph_point(x, endpoint(x, SWEEP_CELLS)));
        s.add(d)
    });
    stage.add_to_scene(marker)?;

    // The dot follows the curve out to the right, then all the way across zero to the left.
    k.until_sentence(stage, "04_3", 1)?;
    let out = tween(input, RADIUS, 0.7)?;
    let out = stage.prepare(out)?;
    stage.play_prepared(vec![out])?;
    let back = tween(input, -RADIUS, 2.4)?;
    let back = stage.prepare(back)?;
    stage.play_prepared(vec![back])?;
    let mut readings = vec![cells_note, marker];
    let mut rows = Vec::new();
    for (i, x) in [RADIUS, -RADIUS].into_iter().enumerate() {
        let (end, series) = sweep_label(x);
        let top = 1.65 - 1.25 * i as f64;
        rows.push(k.tex(
            stage,
            &format!("x={x}"),
            25.0,
            Palette::dim(),
            &[],
            v(GRAPH_RIGHT_X, top),
        )?);
        rows.push(k.text(
            stage,
            &format!("endpoint {end}"),
            23.0,
            Palette::gold(),
            v(GRAPH_RIGHT_X, top - 0.4),
        )?);
        rows.push(k.text(
            stage,
            &format!("series {series}"),
            23.0,
            Palette::blue(),
            v(GRAPH_RIGHT_X, top - 0.75),
        )?);
    }
    reveal_all(stage, &rows, 0.35)?;
    readings.extend(rows);

    k.until_sentence(stage, "04_3", 2)?;
    let not_proof = k.text(stage, NOT_PROOF, 27.0, Palette::hero(), v(-2.6, 1.85))?;
    reveal(stage, not_proof)?;

    k.until_sentence(stage, "04_3", 3)?;
    let why = k.text(stage, WHY_LIMIT, 27.0, Palette::hero(), v(-2.6, 1.85))?;
    let change = swap(stage, not_proof, why, 0.35)?;
    stage.play_prepared(change)?;

    k.until_sentence(stage, "04_3", 4)?;
    let power = k.tex(stage, POWER, 34.0, Palette::gold(), &[], v(-2.6, 0.95))?;
    reveal(stage, power)?;
    k.hold(stage)?;
    let mut screen = drawn.axes;
    screen.extend([drawn.curve, drawn.name, why, power]);
    screen.extend(readings);
    Ok(screen)
}

/// Four small cells, the taken ones lit, at the head of row `row` of a column.
fn mini_strip(stage: &mut Stage<'_>, column: f64, row: usize, lit: &[bool; STRIP_CELLS]) -> fmn::Result<Vec<Mob>> {
    let left = column - MINI_CELL * STRIP_CELLS as f64 / 2.0;
    let y = MINI_TOP - MINI_PITCH * row as f64 - MINI_HEIGHT;
    let mut cells = Vec::new();
    for (i, &taken) in lit.iter().enumerate() {
        let style = if taken {
            ink_style(Palette::gold())
        } else {
            stroke_style(Palette::dim(), 1.5)
        };
        let cell = rect(v(left + MINI_CELL * i as f64, y), MINI_CELL, MINI_HEIGHT, style)?;
        cells.push(stage.arena_mut().add(cell));
    }
    Ok(cells)
}

/// 04_4: multiplying out is choosing. The worked strip's sixteen choices sort into Pascal's row.
fn choices_sort(stage: &mut Stage<'_>, k: &Kit, screen: &[Mob]) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "04_4")?;
    clear(stage, screen)?;
    let offer = k.tex(stage, OFFER, 27.0, Palette::gold(), &[], v(0.0, 2.45))?;
    reveal(stage, offer)?;
    let mut card = vec![offer];

    // Every way of choosing, one small strip each.
    k.until_sentence(stage, "04_4", 2)?;
    let mut all = Vec::new();
    let mut columns: Vec<Vec<Mob>> = Vec::new();
    for count in 0..=STRIP_CELLS {
        let mut column = Vec::new();
        for (row, lit) in choices(count).iter().enumerate() {
            column.extend(mini_strip(stage, column_x(count), row, lit)?);
        }
        for &m in &column {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            all.push(stage.prepare(a)?);
        }
        columns.push(column);
    }
    let group = lagged(stage, all, 0.03)?;
    stage.play(group.rt(1.2))?;
    card.extend(columns.into_iter().flatten());

    // Sorted by how many cells took x: the power each column makes, and how many ways reach it.
    k.until_sentence(stage, "04_4", 3)?;
    let mut labels = vec![
        k.text(stage, "power", 21.0, Palette::dim(), v(ROW_LABEL_X, POWER_Y))?,
        k.text(stage, "ways", 21.0, Palette::dim(), v(ROW_LABEL_X, WAYS_Y))?,
    ];
    for (count, power) in STRIP_POWERS.iter().enumerate() {
        labels.push(k.tex(stage, power, 26.0, Palette::ink(), &[], v(column_x(count), POWER_Y))?);
        labels.push(k.text(
            stage,
            &ways(count).to_string(),
            25.0,
            Palette::ink(),
            v(column_x(count), WAYS_Y),
        )?);
    }
    reveal_all(stage, &labels, 0.35)?;
    card.extend(labels);

    k.until_sentence(stage, "04_4", 4)?;
    let highlight = rect(
        v(column_x(2) - 0.75, WAYS_Y - 0.25),
        1.5,
        POWER_Y + 0.25 - WAYS_Y + 0.25,
        stroke_style(Palette::yellow(), 3.0),
    )?;
    let frame = stage.arena_mut().add(highlight);
    reveal(stage, frame)?;
    card.push(frame);

    // Six ways, each worth a quarter times a quarter, and the series' own coefficient beneath for comparison.
    k.until_sentence(stage, "04_4", 5)?;
    let mut rows = vec![
        k.text(
            stage,
            "coefficient",
            21.0,
            Palette::dim(),
            v(ROW_LABEL_X, COEFFICIENT_Y),
        )?,
        k.text(stage, "series", 21.0, Palette::dim(), v(ROW_LABEL_X, SERIES_Y))?,
        k.tex(stage, WORKED, 28.0, Palette::gold(), &[], v(0.0, -3.3))?,
    ];
    for count in 0..=STRIP_CELLS {
        let color = if count == 2 { Palette::gold() } else { Palette::ink() };
        rows.push(k.text(
            stage,
            &strip_coefficient(count),
            25.0,
            color,
            v(column_x(count), COEFFICIENT_Y),
        )?);
        rows.push(k.text(
            stage,
            &coefficient(count),
            25.0,
            Palette::dim(),
            v(column_x(count), SERIES_Y),
        )?);
    }
    reveal_all(stage, &rows, 0.4)?;
    card.extend(rows);
    k.hold(stage)?;
    Ok(card)
}

/// Four cells at `center`, the picked ones gold, each with the order it was picked in.
fn picked_strip(stage: &mut Stage<'_>, k: &Kit, center: f64, order: [&str; 2]) -> fmn::Result<(Vec<Mob>, Vec<Mob>)> {
    let left = center - PICK_CELL * STRIP_CELLS as f64 / 2.0;
    let mut empty = Vec::new();
    let mut picks = Vec::new();
    for cell in 0..STRIP_CELLS {
        let at = v(left + PICK_CELL * cell as f64, PICK_Y);
        let drawn = rect(at, PICK_CELL, PICK_HEIGHT, stroke_style(Palette::dim(), 1.5))?;
        empty.push(stage.arena_mut().add(drawn));
    }
    for (name, cell) in order.into_iter().zip(PICKED_CELLS) {
        let at = v(left + PICK_CELL * cell as f64, PICK_Y);
        let taken = rect(at, PICK_CELL, PICK_HEIGHT, ink_style(Palette::gold()))?;
        picks.push(stage.arena_mut().add(taken));
        picks.push(k.text(
            stage,
            name,
            21.0,
            Palette::ink(),
            v(at[0] + PICK_CELL / 2.0, PICK_Y + PICK_HEIGHT + 0.2),
        )?);
    }
    Ok((empty, picks))
}

/// 04_5: the same count with letters, the worked numbers beside it, and the factorial earned by double counting.
fn count_in_general(stage: &mut Stage<'_>, k: &Kit, card: &[Mob]) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "04_5")?;
    clear(stage, card)?;
    let name = k.tex(stage, PICK_NAME, 30.0, Palette::ink(), &[], v(LETTER_X, 2.5))?;
    let (first_empty, first_picks) = picked_strip(stage, k, NUMBER_X - 1.3, ["1", "2"])?;
    let mut opening = first_empty.clone();
    opening.push(name);
    reveal_all(stage, &opening, 0.4)?;
    let mut screen = opening;

    // One cell, then another, on the worked strip.
    k.until_sentence(stage, "04_5", 1)?;
    reveal_all(stage, &first_picks[..2], 0.3)?;
    reveal_all(stage, &first_picks[2..], 0.3)?;
    screen.extend(first_picks);

    k.until_sentence(stage, "04_5", 2)?;
    let picks = k.tex(stage, PICKS, 30.0, Palette::ink(), &[], v(LETTER_X, 1.6))?;
    let picks_worked = k.tex(stage, PICKS_WORKED, 26.0, Palette::dim(), &[], v(NUMBER_X, 1.6))?;
    reveal_all(stage, &[picks, picks_worked], 0.4)?;
    screen.extend([picks, picks_worked]);

    // The same two cells, picked the other way round: one set, counted twice.
    k.until_sentence(stage, "04_5", 3)?;
    let (second_empty, second_picks) = picked_strip(stage, k, NUMBER_X + 1.3, ["2", "1"])?;
    let recount = k.tex(stage, RECOUNT, 28.0, Palette::ink(), &[], v(LETTER_X, 0.6))?;
    let recount_worked = k.tex(stage, RECOUNT_WORKED, 26.0, Palette::dim(), &[], v(NUMBER_X, 0.6))?;
    let mut orders = second_empty;
    orders.extend(second_picks);
    orders.extend([recount, recount_worked]);
    reveal_all(stage, &orders, 0.4)?;
    screen.extend(orders);
    let choose = k.tex(stage, CHOOSE, 32.0, Palette::gold(), &[], v(LETTER_X, -1.05))?;
    let choose_worked = k.tex(stage, CHOOSE_WORKED, 28.0, Palette::gold(), &[], v(NUMBER_X, -1.05))?;
    reveal_all(stage, &[choose, choose_worked], 0.4)?;
    screen.extend([choose, choose_worked]);

    k.until_sentence(stage, "04_5", 4)?;
    flash(stage, &[choose], 1.08, 0.9)?;
    k.hold(stage)?;
    Ok(screen)
}

/// 04_6: the factors of n share out, and the coefficient appears as one over k factorial times the chain.
fn chain_appears(stage: &mut Stage<'_>, k: &Kit, screen: &[Mob]) -> fmn::Result<()> {
    k.say(stage, "04_6")?;
    clear(stage, screen)?;
    let shared = k.tex(stage, TERM_SHARED, 30.0, Palette::ink(), &[], v(0.0, 1.9))?;
    reveal(stage, shared)?;

    k.until_sentence(stage, "04_6", 1)?;
    let exact = k.tex(stage, TERM_CHAIN, 30.0, Palette::gold(), &[], v(0.0, 0.55))?;
    reveal(stage, exact)?;

    k.until_sentence(stage, "04_6", 2)?;
    flash(stage, &[exact], 1.06, 0.8)?;

    // The recurring example: the same six sixteenths, read through the chain.
    k.until_sentence(stage, "04_6", 3)?;
    let worked = k.tex(stage, CHAIN_WORKED, 28.0, Palette::ink(), &[], v(0.0, -0.95))?;
    reveal(stage, worked)?;

    k.until_sentence(stage, "04_6", 4)?;
    let zero = k.tex(stage, CHAIN_ZERO, 28.0, Palette::dim(), &[], v(0.0, -2.25))?;
    reveal(stage, zero)?;
    k.hold(stage)
}

fn fill_bar(stage: &mut Stage<'_>, n: u32, k: usize) -> fmn::Result<Mob> {
    bar(
        stage,
        fill_x(k),
        FILL_BASE,
        FILL_WIDTH,
        chain(n, k) * FILL_UNIT,
        Palette::blue(),
    )
}
/// The n and the two column readings, as they stand for one n.
fn fill_readings(stage: &mut Stage<'_>, k: &Kit, n: u32) -> fmn::Result<[Mob; 3]> {
    Ok([
        k.text(stage, &format!("n = {n}"), 27.0, Palette::gold(), v(1.4, READ_Y))?,
        k.text(
            stage,
            &format!("column {FILL_NEAR}: {}", column_label(n, FILL_NEAR)),
            23.0,
            Palette::ink(),
            v(4.6, READ_Y + 0.2),
        )?,
        k.text(
            stage,
            &format!("column {FILL_FAR}: {}", column_label(n, FILL_FAR)),
            23.0,
            Palette::dim(),
            v(4.6, READ_Y - 0.2),
        )?,
    ])
}
/// One step of n: every bar to its new height, on the same clock as the readings.
fn fill_step(
    stage: &mut Stage<'_>,
    k: &Kit,
    bars: &[Mob],
    readings: &mut [Mob; 3],
    n: u32,
    seconds: f64,
) -> fmn::Result<()> {
    let mut targets = Vec::new();
    for (i, &b) in bars.iter().enumerate() {
        targets.push((b, fill_bar(stage, n, i)?));
    }
    let new = fill_readings(stage, k, n)?;
    let mut moves = morph_all(stage, &targets, seconds)?;
    for (old, fresh) in readings.iter().zip(new) {
        moves.extend(swap(stage, *old, fresh, seconds)?);
    }
    stage.play_prepared(moves)?;
    *readings = new;
    Ok(())
}

/// What 05_1 leaves for 05_2.
struct FillPanel {
    /// Labels and axis that 05_2 keeps, because its bars stand on the same base.
    base: Vec<Mob>,
    /// What leaves when the warning sum replaces the chains.
    chains: Vec<Mob>,
}

/// 05_1: each column fills as n grows, and the far columns are still empty. The answer to the question waits.
fn columns_fill(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<FillPanel> {
    k.say(stage, "05_1")?;
    let title = k.text(stage, WHOLE_TITLE, 35.0, Palette::ink(), v(0.0, TITLE_Y))?;
    let name = k.tex(stage, CHAIN_NAME, 26.0, Palette::ink(), &[], v(-3.3, READ_Y))?;
    let axis = line(
        stage,
        v(fill_x(0) - 0.1, FILL_BASE),
        v(fill_x(FILL_TOP) + 0.1, FILL_BASE),
        Palette::dim(),
        2.0,
    )?;
    let mut base = vec![
        axis,
        k.text(stage, COLUMN_NAME, 21.0, Palette::dim(), v(0.0, FILL_BASE - 0.62))?,
    ];
    for tick in (0..=FILL_TOP).step_by(25) {
        base.push(k.text(
            stage,
            &tick.to_string(),
            21.0,
            Palette::dim(),
            v(fill_x(tick), FILL_BASE - 0.28),
        )?);
    }
    let mut bars = Vec::new();
    for i in 0..=FILL_TOP {
        bars.push(fill_bar(stage, FILL_COUNTS[0], i)?);
    }
    let mut readings = fill_readings(stage, k, FILL_COUNTS[0])?;
    let mut opening = base.clone();
    opening.extend(bars.iter().copied());
    opening.extend(readings);
    opening.extend([title, name]);
    reveal_all(stage, &opening, 0.4)?;

    // A column's bar is its chain, and the ceiling is the series' own coefficient.
    k.until_sentence(stage, "05_1", 1)?;
    let top = line(
        stage,
        v(fill_x(0) - 0.1, FILL_BASE + FILL_UNIT),
        v(fill_x(FILL_TOP) + 0.1, FILL_BASE + FILL_UNIT),
        Palette::green(),
        2.0,
    )?;
    let full = k.text(stage, FULL, 23.0, Palette::green(), v(2.4, FILL_BASE + FILL_UNIT + 0.3))?;
    reveal_all(stage, &[top, full], 0.35)?;

    k.until_sentence(stage, "05_1", 2)?;
    fill_step(stage, k, &bars, &mut readings, FILL_COUNTS[1], 0.6)?;
    fill_step(stage, k, &bars, &mut readings, FILL_COUNTS[2], 0.6)?;

    k.until_sentence(stage, "05_1", 3)?;
    fill_step(stage, k, &bars, &mut readings, FILL_COUNTS[3], 0.6)?;

    k.until_sentence(stage, "05_1", 4)?;
    fill_step(stage, k, &bars, &mut readings, FILL_COUNTS[4], 0.6)?;
    flash(stage, &[readings[1], readings[2]], 1.1, 0.8)?;

    k.until_sentence(stage, "05_1", 5)?;
    let question = k.text(stage, MUST_ARRIVE, 27.0, Palette::hero(), v(0.0, -3.5))?;
    reveal(stage, question)?;
    k.hold(stage)?;
    k.mark(stage, "predict");
    k.music_breath(stage, 4.0)?;
    clear(stage, &[question])?;
    let mut chains = bars;
    chains.extend(readings);
    chains.extend([name, top, full]);
    Ok(FillPanel { base, chains })
}

/// The warning sum's bars, for n terms, in columns 1 to n of the same axis. The column being watched stands out.
fn warning_bars(stage: &mut Stage<'_>, n: u32) -> fmn::Result<Vec<Mob>> {
    let mut bars = Vec::new();
    for column in 1..=n as usize {
        let color = if column == WARNING_COLUMN {
            Palette::hero()
        } else {
            Palette::gold()
        };
        bars.push(bar(
            stage,
            fill_x(column),
            FILL_BASE,
            FILL_WIDTH,
            FILL_UNIT / f64::from(n),
            color,
        )?);
    }
    Ok(bars)
}
/// One step of the warning sum. Old columns shrink and new columns grow out of the base, on one clock.
fn warning_step(stage: &mut Stage<'_>, k: &Kit, bars: &mut Vec<Mob>, reading: &mut Mob, n: u32) -> fmn::Result<()> {
    let targets = warning_bars(stage, n)?;
    let new_reading = warning_reading(stage, k, n)?;
    let mut moves = swap(stage, *reading, new_reading, 0.6)?;
    for (i, &target) in targets.iter().enumerate() {
        if let Some(&old) = bars.get(i) {
            moves.push(stage.prepare(Transform::new(old, target).rt(0.6))?);
        } else {
            let grow = fmn::animation::grow_from_edge(stage.arena_mut(), target, DOWN, None)?;
            moves.push(stage.prepare(grow.rt(0.6))?);
            bars.push(target);
        }
    }
    stage.play_prepared(moves)?;
    *reading = new_reading;
    Ok(())
}
fn warning_reading(stage: &mut Stage<'_>, k: &Kit, n: u32) -> fmn::Result<Mob> {
    k.tex(
        stage,
        &format!(r"n={n}:\quad\text{{column}}\ {WARNING_COLUMN}\ =\frac{{1}}{{{n}}}"),
        26.0,
        Palette::gold(),
        &[],
        v(2.6, READ_Y),
    )
}

/// 05_2: the answer is no. A sum whose fixed terms all vanish while its total stays at one.
fn warning_sum(stage: &mut Stage<'_>, k: &Kit, panel: FillPanel) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "05_2")?;
    let verdict = k.text(stage, NOT_ENOUGH, 27.0, Palette::hero(), v(0.0, 1.55))?;
    reveal(stage, verdict)?;

    k.until_sentence(stage, "05_2", 1)?;
    let mut leaving = panel.chains;
    leaving.push(verdict);
    clear(stage, &leaving)?;

    k.until_sentence(stage, "05_2", 2)?;
    let mut bars = warning_bars(stage, WARNING_COUNTS[0])?;
    let name = k.tex(stage, WARNING_SUM, 28.0, Palette::gold(), &[], v(-3.3, READ_Y))?;
    let mut reading = warning_reading(stage, k, WARNING_COUNTS[0])?;
    let mut shown = bars.clone();
    shown.extend([name, reading]);
    reveal_all(stage, &shown, 0.4)?;

    k.until_sentence(stage, "05_2", 3)?;
    warning_step(stage, k, &mut bars, &mut reading, WARNING_COUNTS[1])?;
    warning_step(stage, k, &mut bars, &mut reading, WARNING_COUNTS[2])?;

    // The tower is every bar stacked: one full unit, whatever n is.
    k.until_sentence(stage, "05_2", 4)?;
    let tower = bar(stage, TOWER_X, FILL_BASE, 0.5, FILL_UNIT, Palette::hero())?;
    let total = k.text(
        stage,
        TOTAL_ONE,
        23.0,
        Palette::hero(),
        v(TOWER_X - 0.1, FILL_BASE + FILL_UNIT + 0.3),
    )?;
    reveal_all(stage, &[tower, total], 0.4)?;

    k.until_sentence(stage, "05_2", 5)?;
    warning_step(stage, k, &mut bars, &mut reading, WARNING_COUNTS[3])?;
    let outward = k.text(stage, OUTWARD, 23.0, Palette::hero(), v(1.0, FILL_BASE + 1.0))?;
    reveal(stage, outward)?;
    k.hold(stage)?;
    let mut screen = panel.base;
    screen.extend(bars);
    screen.extend([name, reading, tower, total, outward]);
    Ok(screen)
}

/// Bar k of the expanded product at the chapter's radius, standing on the chart's base.
fn term_bar(stage: &mut Stage<'_>, n: u32, i: usize) -> fmn::Result<Mob> {
    bar(
        stage,
        bar_x(i),
        BAR_BASE,
        BAR_WIDTH,
        term(RADIUS, n, i).abs() * BAR_UNIT,
        Palette::blue(),
    )
}
/// The empty space under ceiling k.
fn gap_block(stage: &mut Stage<'_>, n: u32, i: usize) -> fmn::Result<Mob> {
    bar(
        stage,
        bar_x(i),
        BAR_BASE + term(RADIUS, n, i).abs() * BAR_UNIT,
        BAR_WIDTH,
        gap(RADIUS, n, i) * BAR_UNIT,
        Palette::hero(),
    )
}
fn n_label(stage: &mut Stage<'_>, k: &Kit, n: u32) -> fmn::Result<Mob> {
    k.text(stage, &format!("n = {n}"), 25.0, Palette::gold(), v(-3.0, NAME_Y))
}

/// What 05_3 leaves for 05_4 and 05_5.
struct Ceilings {
    bars: Vec<Mob>,
    lids: Vec<Mob>,
    indices: Vec<Mob>,
    axis: Mob,
    input: Mob,
    count: Mob,
    card: Vec<Mob>,
}

/// 05_3: the chain stays in range, so no bar passes the series' own ceiling, and n never moves those ceilings.
fn ceilings_hold(stage: &mut Stage<'_>, k: &Kit, screen: &[Mob]) -> fmn::Result<Ceilings> {
    k.say(stage, "05_3")?;
    clear(stage, screen)?;
    let axis = line(
        stage,
        v(bar_x(0) - 0.3, BAR_BASE),
        v(bar_x(BAR_TOP) + 0.3, BAR_BASE),
        Palette::dim(),
        2.0,
    )?;
    let mut lids = Vec::new();
    let mut indices = Vec::new();
    for i in 0..=BAR_TOP {
        lids.push(ceiling(
            stage,
            bar_x(i),
            BAR_BASE,
            BAR_WIDTH,
            term_size(RADIUS, i) * BAR_UNIT,
            Palette::green(),
        )?);
        indices.push(k.text(stage, &i.to_string(), 21.0, Palette::dim(), v(bar_x(i), BAR_INDEX_Y))?);
    }
    let input = k.tex(
        stage,
        &format!("x={RADIUS}"),
        25.0,
        Palette::gold(),
        &[],
        v(-4.6, NAME_Y),
    )?;
    let mut opening = lids.clone();
    opening.extend(indices.iter().copied());
    opening.extend([axis, input]);
    reveal_all(stage, &opening, 0.4)?;

    k.until_sentence(stage, "05_3", 1)?;
    let range = k.tex(stage, CHAIN_RANGE, 27.0, Palette::ink(), &[], v(CARD_X, 1.6))?;
    reveal(stage, range)?;
    until_phrase(k, stage, "05_3", "and after that")?;
    let empty = k.tex(stage, CHAIN_EMPTY, 25.0, Palette::dim(), &[], v(CARD_X, 1.0))?;
    reveal(stage, empty)?;

    // The bars rise out of the base, each one inside its ceiling.
    k.until_sentence(stage, "05_3", 2)?;
    let bound = k.tex(stage, BOUNDED, 28.0, Palette::green(), &[], v(CARD_X, 2.4))?;
    let mut bars = Vec::new();
    let mut grows: Vec<Box<dyn Animation>> = Vec::new();
    for i in 0..=BAR_TOP {
        let b = term_bar(stage, BAR_COUNTS[0], i)?;
        let grow = fmn::animation::grow_from_edge(stage.arena_mut(), b, DOWN, None)?;
        grows.push(stage.prepare(grow)?);
        bars.push(b);
    }
    let group = lagged(stage, grows, 0.08)?;
    stage.play(group.rt(0.9))?;
    let mut count = n_label(stage, k, BAR_COUNTS[0])?;
    reveal_all(stage, &[bound, count], 0.35)?;
    // Bars were added last; the ceilings go back on top, so each lid stays whole.
    for &lid in &lids {
        stage.add_to_scene(lid)?;
    }

    // n climbs. The lids do not move, and nothing passes them.
    k.until_sentence(stage, "05_3", 3)?;
    for &n in &BAR_COUNTS[1..] {
        let mut targets = Vec::new();
        for (i, &b) in bars.iter().enumerate() {
            targets.push((b, term_bar(stage, n, i)?));
        }
        let new_count = n_label(stage, k, n)?;
        let mut moves = morph_all(stage, &targets, 0.5)?;
        moves.extend(swap(stage, count, new_count, 0.5)?);
        stage.play_prepared(moves)?;
        count = new_count;
    }

    k.until_sentence(stage, "05_3", 4)?;
    let no_ceiling = k.text(stage, NO_CEILING, 25.0, Palette::gold(), v(CARD_X, 0.1))?;
    reveal(stage, no_ceiling)?;

    k.until_sentence(stage, "05_3", 5)?;
    let harmonic = k.tex(stage, HARMONIC, 26.0, Palette::gold(), &[], v(CARD_X, -0.65))?;
    reveal(stage, harmonic)?;
    k.hold(stage)?;
    Ok(Ceilings {
        bars,
        lids,
        indices,
        axis,
        input,
        count,
        card: vec![bound, range, empty, no_ceiling, harmonic],
    })
}

/// One zoomed bar of the tail, at the series chapter's enlargement.
fn inset_bar(stage: &mut Stage<'_>, n: u32, i: usize) -> fmn::Result<Mob> {
    bar(
        stage,
        inset_x(i),
        INSET_BASE,
        INSET_WIDTH,
        term(RADIUS, n, i).abs() * inset_unit(),
        Palette::hero(),
    )
}

/// What 05_4 leaves for 05_5.
struct Cut {
    gaps: Vec<Mob>,
    inset_bars: Vec<Mob>,
    line: Mob,
    name: Mob,
    tail: Mob,
    card: Vec<Mob>,
}

/// 05_4: the distance is at most the gaps. The aim is set and the cut is chosen before n moves.
fn gaps_and_cut(stage: &mut Stage<'_>, k: &Kit, panel: &mut Ceilings) -> fmn::Result<Cut> {
    k.say(stage, "05_4")?;
    clear(stage, &panel.card)?;
    // Back to a small n, so there are gaps to look at.
    let start = BAR_COUNTS[1];
    let mut targets = Vec::new();
    for (i, &b) in panel.bars.iter().enumerate() {
        targets.push((b, term_bar(stage, start, i)?));
    }
    let new_count = n_label(stage, k, start)?;
    let distance_tex = k.tex(stage, DISTANCE, 28.0, Palette::ink(), &[], v(CARD_X, 2.4))?;
    let mut moves = morph_all(stage, &targets, 0.5)?;
    moves.extend(swap(stage, panel.count, new_count, 0.5)?);
    let shown = fade_in(stage.arena_mut(), distance_tex, ORIGIN, 1.0)?;
    moves.push(stage.prepare(shown.rt(0.5))?);
    stage.play_prepared(moves)?;
    panel.count = new_count;
    let mut card = vec![distance_tex];

    // Each column's empty space turns solid. Together they are the bound.
    k.until_sentence(stage, "05_4", 1)?;
    let mut gaps = Vec::new();
    for i in 0..=BAR_TOP {
        gaps.push(gap_block(stage, start, i)?);
    }
    let gaps_tex = k.tex(stage, GAPS, 26.0, Palette::hero(), &[], v(CARD_X, 1.55))?;
    let mut shown_gaps = gaps.clone();
    shown_gaps.push(gaps_tex);
    reveal_all(stage, &shown_gaps, 0.4)?;
    card.push(gaps_tex);

    k.until_sentence(stage, "05_4", 2)?;
    let aim = k.text(
        stage,
        &format!("aim: distance below {AIM}"),
        23.0,
        Palette::ink(),
        v(CARD_X, 0.85),
    )?;
    reveal(stage, aim)?;
    card.push(aim);

    // The cut comes before any change of n, and the first one tried is not good enough for the aim.
    k.until_sentence(stage, "05_4", 3)?;
    let line = stage.arena_mut().add(
        DashedLine::new(
            v(cut_x(EARLY_CUT), BAR_BASE - 0.1),
            v(cut_x(EARLY_CUT), BAR_BASE + CUT_HEIGHT),
        )
        .dash_length(0.12)
        .style(stroke_style(Palette::ink(), 2.0))
        .build()?,
    );
    k.allow_crossing(line);
    let name = k.text(
        stage,
        "cut",
        23.0,
        Palette::ink(),
        v(cut_x(EARLY_CUT), BAR_BASE + CUT_HEIGHT + 0.2),
    )?;
    reveal_all(stage, &[line, name], 0.35)?;

    // Past the cut, the zoom from the series chapter: the same ceilings, with this chapter's bars inside them.
    k.until_sentence(stage, "05_4", 4)?;
    let mut inset = Vec::new();
    let mut inset_bars = Vec::new();
    for i in EARLY_CUT + 1..=BAR_TOP {
        let b = inset_bar(stage, start, i)?;
        inset_bars.push(b);
        inset.push(b);
        inset.push(ceiling(
            stage,
            inset_x(i),
            INSET_BASE,
            INSET_WIDTH,
            term_size(RADIUS, i) * inset_unit(),
            Palette::green(),
        )?);
        inset.push(k.text(
            stage,
            &i.to_string(),
            21.0,
            Palette::dim(),
            v(inset_x(i), INSET_BASE - 0.25),
        )?);
    }
    inset.push(line_between(
        stage,
        inset_x(EARLY_CUT + 1) - 0.4,
        inset_x(BAR_TOP) + 0.4,
    )?);
    inset.push(k.text(
        stage,
        &format!("zoom ×{ZOOM}"),
        23.0,
        Palette::dim(),
        v(6.05, INSET_BASE + 0.9),
    )?);
    let tail = k.text(
        stage,
        &format!("tail after {EARLY_CUT} ≤ {}: too big", tail_print(EARLY_CUT)),
        23.0,
        Palette::red(),
        v(CARD_X, 0.45),
    )?;
    let mut shown_inset = inset.clone();
    shown_inset.push(tail);
    reveal_all(stage, &shown_inset, 0.4)?;
    card.extend(inset);
    k.hold(stage)?;
    Ok(Cut {
        gaps,
        inset_bars,
        line,
        name,
        tail,
        card,
    })
}
fn line_between(stage: &mut Stage<'_>, from: f64, to: f64) -> fmn::Result<Mob> {
    line(stage, v(from, INSET_BASE), v(to, INSET_BASE), Palette::dim(), 2.0)
}

/// 05_5: a later cut, and only now does n move. The tail's number stays put while the head closes.
fn tail_then_head(stage: &mut Stage<'_>, k: &Kit, panel: &mut Ceilings, cut: Cut) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "05_5")?;
    let mut card = cut.card;
    card.extend(cut.gaps.iter().copied());
    card.extend([cut.line, cut.name]);

    until_phrase(k, stage, "05_5", "so push")?;
    let late = k.text(
        stage,
        &format!("tail after {CUT} ≤ {}", tail_print(CUT)),
        23.0,
        Palette::green(),
        v(CARD_X, 0.45),
    )?;
    let inset_cut = stage.arena_mut().add(
        DashedLine::new(
            v(inset_x(CUT) + INSET_PITCH / 2.0, INSET_BASE - 0.1),
            v(inset_x(CUT) + INSET_PITCH / 2.0, INSET_BASE + 1.45),
        )
        .dash_length(0.12)
        .style(stroke_style(Palette::ink(), 2.0))
        .build()?,
    );
    k.allow_crossing(inset_cut);
    let mut moves = swap(stage, cut.tail, late, 0.5)?;
    moves.push(glide(
        stage,
        cut.line,
        v(cut_x(CUT), BAR_BASE + (CUT_HEIGHT - 0.1) / 2.0),
        0.5,
    )?);
    moves.push(glide(stage, cut.name, v(cut_x(CUT), BAR_BASE + CUT_HEIGHT + 0.2), 0.5)?);
    let shown = fade_in(stage.arena_mut(), inset_cut, ORIGIN, 1.0)?;
    moves.push(stage.prepare(shown.rt(0.5))?);
    stage.play_prepared(moves)?;
    card.extend([late, inset_cut]);

    // Each count closes the gaps before the cut. Those columns' zoomed bars rise to their ceilings.
    k.until_sentence(stage, "05_5", 1)?;
    let mut rows = Vec::new();
    for (row, &n) in HEAD_COUNTS.iter().enumerate() {
        let mut targets = Vec::new();
        for (i, &b) in panel.bars.iter().enumerate() {
            targets.push((b, term_bar(stage, n, i)?));
        }
        for (i, &g) in cut.gaps.iter().enumerate() {
            targets.push((g, gap_block(stage, n, i)?));
        }
        for (j, i) in (EARLY_CUT + 1..=BAR_TOP).enumerate() {
            targets.push((cut.inset_bars[j], inset_bar(stage, n, i)?));
        }
        let new_count = n_label(stage, k, n)?;
        let fits = n == HEAD_COUNTS[HEAD_COUNTS.len() - 1];
        let shown_row = k.text(
            stage,
            &format!("{n} cells: head ≤ {}", up_label(head_gaps(RADIUS, n, CUT))),
            23.0,
            if fits { Palette::green() } else { Palette::dim() },
            v(CARD_X, 0.05 - 0.36 * row as f64),
        )?;
        let mut moves = morph_all(stage, &targets, 0.5)?;
        moves.extend(swap(stage, panel.count, new_count, 0.5)?);
        let shown = fade_in(stage.arena_mut(), shown_row, ORIGIN, 1.0)?;
        moves.push(stage.prepare(shown.rt(0.5))?);
        stage.play_prepared(moves)?;
        panel.count = new_count;
        rows.push(shown_row);
    }
    card.extend(rows.iter().copied());

    // The order, on the two rows it names.
    k.until_sentence(stage, "05_5", 2)?;
    flash(stage, &[late], 1.1, 0.5)?;
    until_phrase(k, stage, "05_5", "then make")?;
    flash(stage, &[rows[rows.len() - 1]], 1.1, 0.5)?;

    k.until_sentence(stage, "05_5", 3)?;
    let last = HEAD_COUNTS[HEAD_COUNTS.len() - 1];
    let total = k.text(
        stage,
        &format!("total ≤ {} < {AIM}", total_label(last)),
        25.0,
        Palette::green(),
        v(CARD_X, -1.15),
    )?;
    reveal(stage, total)?;
    card.push(total);
    k.hold(stage)?;
    Ok(card)
}

/// Bar k of the expanded product at the input the tracker holds, pointing up or down with its sign.
fn live_term_bar(stage: &mut Stage<'_>, input: Mob, n: u32, i: usize) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let x = s.tracker_value(input).unwrap_or(RADIUS);
        let value = term(x, n, i);
        let height = (value.abs() * BAR_UNIT).max(MIN_BAR);
        let r = Rectangle::new()
            .width(BAR_WIDTH)
            .height(height)
            .style(ink_style(Palette::blue()))
            .build()
            .expect("positive extents");
        s.add(r.moved_to(v(bar_x(i), BAR_BASE + value.signum() * height / 2.0)))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// 05_6: the same picture at minus three halves. Odd bars point down, and their sizes do not change.
fn negative_input(stage: &mut Stage<'_>, k: &Kit, panel: &Ceilings, leaving: &[Mob]) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "05_6")?;
    clear(stage, leaving)?;
    // Back to a count where the gaps can be seen. The indices step down now, before any bar can cross them.
    let shown_n = BAR_COUNTS[1];
    let mut targets = Vec::new();
    for (i, &b) in panel.bars.iter().enumerate() {
        targets.push((b, term_bar(stage, shown_n, i)?));
    }
    let new_count = n_label(stage, k, shown_n)?;
    let mut moves = morph_all(stage, &targets, 0.5)?;
    moves.extend(swap(stage, panel.count, new_count, 0.5)?);
    for (i, &index) in panel.indices.iter().enumerate() {
        moves.push(glide(stage, index, v(bar_x(i), BAR_INDEX_LOW_Y), 0.5)?);
    }
    stage.play_prepared(moves)?;

    // Live from here: at the tracker's starting value each live bar is exactly the bar it replaces.
    k.until_sentence(stage, "05_6", 1)?;
    let tracker = stage.add_value_tracker(RADIUS);
    let mut live = Vec::new();
    for (i, &old) in panel.bars.iter().enumerate() {
        live.push(live_term_bar(stage, tracker, shown_n, i)?);
        stage.remove_from_scene(old);
    }
    for &lid in &panel.lids {
        stage.add_to_scene(lid)?;
    }
    let name = k.tex(stage, "x=", 25.0, Palette::gold(), &[], v(-4.9, NAME_Y))?;
    let reading = k.live_number(
        stage,
        tracker,
        1,
        false,
        25.0,
        Palette::gold(),
        v(-4.15, NAME_Y),
        ORIGIN,
    )?;
    let out = fade_out(stage.arena_mut(), panel.input, ORIGIN, 1.0)?;
    let into = fade_in(stage.arena_mut(), name, ORIGIN, 1.0)?;
    let relabel = vec![stage.prepare(out.rt(0.25))?, stage.prepare(into.rt(0.25))?];
    stage.play_prepared(relabel)?;
    let sweep = tween(tracker, -RADIUS, 1.5)?;
    let sweep = stage.prepare(sweep)?;
    stage.play_prepared(vec![sweep])?;

    // The odd columns get their ceilings below the axis.
    k.until_sentence(stage, "05_6", 2)?;
    let mut below = Vec::new();
    let mut steps: Vec<Box<dyn Animation>> = Vec::new();
    for i in (1..=BAR_TOP).step_by(2) {
        let height = (term_size(RADIUS, i) * BAR_UNIT).max(MIN_BAR);
        let lid = ceiling(stage, bar_x(i), BAR_BASE - height, BAR_WIDTH, height, Palette::green())?;
        let a = fade_in(stage.arena_mut(), lid, ORIGIN, 1.0)?;
        steps.push(stage.prepare(a.rt(0.5))?);
        below.push(lid);
    }
    let signs = k.tex(stage, SIGNS, 26.0, Palette::ink(), &[], v(CARD_X, 1.6))?;
    let shown_signs = fade_in(stage.arena_mut(), signs, ORIGIN, 1.0)?;
    steps.push(stage.prepare(shown_signs.rt(0.5))?);
    stage.play_prepared(steps)?;

    k.until_sentence(stage, "05_6", 3)?;
    let same = k.tex(stage, SAME_SIZE, 26.0, Palette::green(), &[], v(CARD_X, 0.75))?;
    reveal(stage, same)?;

    // The same bound holds, and the true distances sit under it at both signs.
    k.until_sentence(stage, "05_6", 4)?;
    let last = HEAD_COUNTS[HEAD_COUNTS.len() - 1];
    let total = k.text(
        stage,
        &format!("{last} cells, total ≤ {}", total_label(last)),
        23.0,
        Palette::green(),
        v(CARD_X, -0.1),
    )?;
    let plus = k.tex(
        stage,
        &format!(r"x={RADIUS}:\ \text{{distance}}\ {:.5}", distance(RADIUS, last)),
        25.0,
        Palette::ink(),
        &[],
        v(CARD_X, -0.6),
    )?;
    let minus = k.tex(
        stage,
        &format!(r"x={}:\ \text{{distance}}\ {:.5}", -RADIUS, distance(-RADIUS, last)),
        25.0,
        Palette::ink(),
        &[],
        v(CARD_X, -1.1),
    )?;
    reveal_all(stage, &[total, plus, minus], 0.35)?;
    k.hold(stage)?;
    let mut screen = live;
    screen.extend(below);
    screen.extend(panel.lids.iter().copied());
    screen.extend(panel.indices.iter().copied());
    screen.extend([panel.axis, new_count, name, reading, signs, same, total, plus, minus]);
    Ok(screen)
}

/// 05_7: minus three, cell by cell. Every kind of base appears, and the values do not climb steadily.
fn edge_table(stage: &mut Stage<'_>, k: &Kit, screen: &[Mob]) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "05_7")?;
    clear(stage, screen)?;
    let mut card = vec![
        k.tex(
            stage,
            &format!("x={EDGE_INPUT}"),
            30.0,
            Palette::gold(),
            &[],
            v(CELLS_X, 2.55),
        )?,
        k.text(stage, "cells", 23.0, Palette::dim(), v(CELLS_X, 2.2))?,
        k.tex(stage, CELL_MULTIPLIER, 23.0, Palette::dim(), &[], v(BASE_X, 2.3))?,
        k.tex(stage, POWER, 23.0, Palette::dim(), &[], v(VALUE_X, 2.3))?,
    ];
    reveal_all(stage, &card, 0.35)?;
    let row_y = |row: usize| ROW_Y0 - ROW_PITCH * row as f64;
    let mut rows: Vec<[Mob; 3]> = Vec::new();
    for (row, &n) in EDGE_COUNTS.iter().enumerate() {
        let color = if row < 3 { Palette::red() } else { Palette::ink() };
        rows.push([
            k.text(stage, &n.to_string(), 25.0, color, v(CELLS_X, row_y(row)))?,
            k.tex(stage, &edge_base(n), 25.0, color, &[], v(BASE_X, row_y(row)))?,
            k.tex(stage, &edge_value(n), 25.0, color, &[], v(VALUE_X, row_y(row)))?,
        ]);
    }
    k.until_sentence(stage, "05_7", 1)?;
    reveal_all(stage, &rows[0], 0.35)?;
    k.until_sentence(stage, "05_7", 2)?;
    reveal_all(stage, &rows[1], 0.35)?;
    until_phrase(k, stage, "05_7", "and three")?;
    reveal_all(stage, &rows[2], 0.35)?;

    // Almost nothing at four cells, then the climb. E of minus three stands under the column it is approached by.
    k.until_sentence(stage, "05_7", 3)?;
    reveal_all(stage, &rows[3], 0.3)?;
    for row in &rows[4..] {
        reveal_all(stage, row, 0.3)?;
    }
    let series = k.tex(
        stage,
        &format!(r"E({EDGE_INPUT})={:.4}", e_value(EDGE_INPUT)),
        27.0,
        Palette::blue(),
        &[],
        v(VALUE_X - 0.6, row_y(EDGE_COUNTS.len()) - 0.05),
    )?;
    reveal(stage, series)?;
    card.push(series);
    card.extend(rows.iter().flatten().copied());

    // The physical reading stops at the rows whose multiplier is not positive. The algebra does not.
    k.until_sentence(stage, "05_7", 4)?;
    let notes = [
        line(
            stage,
            v(NOTE_X - 2.4, row_y(0) + 0.2),
            v(NOTE_X - 2.4, row_y(2) - 0.2),
            Palette::red(),
            2.5,
        )?,
        k.text(stage, NOT_POSITIVE, 23.0, Palette::red(), v(NOTE_X, row_y(1) + 0.2))?,
        k.text(stage, NO_DECAY, 23.0, Palette::red(), v(NOTE_X, row_y(1) - 0.2))?,
        line(
            stage,
            v(NOTE_X - 2.4, row_y(3) + 0.2),
            v(NOTE_X - 2.4, row_y(6) - 0.2),
            Palette::green(),
            2.5,
        )?,
        k.text(stage, POSITIVE, 23.0, Palette::green(), v(NOTE_X, row_y(4) - 0.2))?,
    ];
    reveal_all(stage, &notes, 0.4)?;
    card.extend(notes);

    k.until_sentence(stage, "05_7", 5)?;
    let same = k.text(stage, SAME_ALGEBRA, 25.0, Palette::gold(), v(0.0, -2.75))?;
    reveal(stage, same)?;
    card.push(same);

    k.until_sentence(stage, "05_7", 6)?;
    let no_climb = k.text(stage, NO_CLIMB, 25.0, Palette::gold(), v(0.0, -3.3))?;
    reveal(stage, no_climb)?;
    card.push(no_climb);
    k.hold(stage)?;
    Ok(card)
}

/// One stepped walk on the closing graph, from the start out to input x.
fn graph_walk(stage: &mut Stage<'_>, x: f64, n: u32, color: Srgb) -> Mob {
    let points: Vec<Vec3> = walk(x, n).iter().map(|&(a, y)| graph_point(a, y)).collect();
    path(stage, points, color, 2.5)
}

/// 05_8: the opening drawing in its rigorous form. Stepped walks in both directions, on the curve they reach.
fn payoff(stage: &mut Stage<'_>, k: &Kit, card: &[Mob]) -> fmn::Result<()> {
    k.say(stage, "05_8")?;
    clear(stage, card)?;
    let drawn = graph(stage, k)?;
    let limit = k.tex(stage, LIMIT, 32.0, Palette::hero(), &[], v(-2.6, 1.8))?;
    let every = k.text(stage, EVERY_REAL, 25.0, Palette::hero(), v(-2.6, 1.15))?;
    let mut shown = drawn.axes.clone();
    shown.extend([drawn.curve, drawn.name, limit, every]);
    reveal_all(stage, &shown, 0.4)?;

    k.until_sentence(stage, "05_8", 1)?;
    let later = k.text(stage, BIGGER, 23.0, Palette::dim(), v(-2.6, 0.6))?;
    reveal(stage, later)?;

    k.until_sentence(stage, "05_8", 2)?;
    let at_one = dot(stage, graph_point(1.0, e_value(1.0)), Palette::gold());
    let special = k.tex(stage, SPECIAL_CASE, 27.0, Palette::gold(), &[], v(-2.6, -0.1))?;
    let number = k.text(
        stage,
        &format!("e = {}", e_label()),
        23.0,
        Palette::gold(),
        v(graph_point(1.0, 0.0)[0] + 1.25, graph_point(0.0, e_value(1.0))[1] - 0.1),
    )?;
    reveal_all(stage, &[at_one, special, number], 0.35)?;

    // The same product, read the other way: this is why a drawing at rate one reaches every input.
    k.until_sentence(stage, "05_8", 3)?;
    let swapped = k.tex(
        stage,
        TIME_SWAP,
        25.0,
        Palette::ink(),
        &[],
        v(GRAPH_RIGHT_X - 0.7, 2.85),
    )?;
    reveal(stage, swapped)?;

    // Few steps fall short and more steps close in, on each side of the start.
    k.until_sentence(stage, "05_8", 4)?;
    let coarse = [
        graph_walk(stage, RADIUS, WALK_COUNTS[0], Palette::gold()),
        graph_walk(stage, GRAPH_LOW, WALK_COUNTS[0], Palette::gold()),
    ];
    reveal_all(stage, &coarse, 0.5)?;
    let fine = [
        graph_walk(stage, RADIUS, WALK_COUNTS[1], Palette::hero()),
        graph_walk(stage, GRAPH_LOW, WALK_COUNTS[1], Palette::hero()),
    ];
    let mut change = vec![dim_path(stage, coarse[0], 0.5)?, dim_path(stage, coarse[1], 0.5)?];
    for m in fine {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        change.push(stage.prepare(a.rt(0.5))?);
    }
    stage.play_prepared(change)?;

    // The debt, in the words the series chapter wrote it in, now paid.
    k.until_sentence(stage, "05_8", 5)?;
    let debt = k.text(stage, SECOND_DEBT, 25.0, Palette::gold(), v(0.0, -3.62))?;
    reveal(stage, debt)?;
    let paid = recolor(stage, debt, Palette::green(), 0.5)?;
    stage.play_prepared(vec![paid])?;
    Ok(())
}

impl SceneConstruct for TinyMultipliers {
    fn name(&self) -> &str {
        "04_tiny_multipliers"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let mut panel = steps_multiply(stage, &k)?;
        let card = rate_and_cells(stage, &k, &mut panel)?;
        let screen = endpoints_on_the_graph(stage, &k, &panel, &card)?;
        let strips = choices_sort(stage, &k, &screen)?;
        let count = count_in_general(stage, &k, &strips)?;
        chain_appears(stage, &k, &count)?;
        k.finish(stage)
    }
}

impl SceneConstruct for WholeSum {
    fn name(&self) -> &str {
        "05_whole_sum"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let fill = columns_fill(stage, &k)?;
        let warning = warning_sum(stage, &k, fill)?;
        let mut ceilings = ceilings_hold(stage, &k, &warning)?;
        let cut = gaps_and_cut(stage, &k, &mut ceilings)?;
        let proof = tail_then_head(stage, &k, &mut ceilings, cut)?;
        let negative = negative_input(stage, &k, &ceilings, &proof)?;
        let table = edge_table(stage, &k, &negative)?;
        payoff(stage, &k, &table)?;
        k.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// The scene's code, without this test module.
    fn scene() -> &'static str {
        include_str!("tiny_multipliers.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("split yields a first part")
    }
    fn assert_in_order(needles: &[&str]) {
        let mut from = 0;
        for &needle in needles {
            let at = scene()[from..]
                .find(needle)
                .unwrap_or_else(|| panic!("{needle} missing or out of order"));
            from += at + needle.len();
        }
    }
    #[test]
    fn the_cues_run_in_film_order_each_from_its_own_function() {
        assert_in_order(&[
            r#"k.say(stage, "04_1")?;"#,
            r#"k.say(stage, "04_2")?;"#,
            r#"k.say(stage, "04_3")?;"#,
            r#"k.say(stage, "04_4")?;"#,
            r#"k.say(stage, "04_5")?;"#,
            r#"k.say(stage, "04_6")?;"#,
            r#"k.say(stage, "05_1")?;"#,
            r#"k.say(stage, "05_2")?;"#,
            r#"k.say(stage, "05_3")?;"#,
            r#"k.say(stage, "05_4")?;"#,
            r#"k.say(stage, "05_5")?;"#,
            r#"k.say(stage, "05_6")?;"#,
            r#"k.say(stage, "05_7")?;"#,
            r#"k.say(stage, "05_8")?;"#,
        ]);
    }
    /// A cue's words, from the script the phrase hook reads at run time.
    fn spoken(cue: &str) -> &'static str {
        let prefix = format!("{cue}\t");
        crate::phrase::SCRIPT
            .lines()
            .find_map(|l| l.strip_prefix(prefix.as_str()))
            .unwrap_or_else(|| panic!("{cue} is not in the script"))
    }
    #[test]
    fn every_phrase_hook_is_in_its_own_line() {
        for (cue, phrase) in [
            ("05_3", "and after that"),
            ("05_5", "so push"),
            ("05_5", "then make"),
            ("05_7", "and three"),
        ] {
            assert!(spoken(cue).contains(phrase), "{cue}: {phrase}");
            assert!(scene().contains(&format!(r#"until_phrase(k, stage, "{cue}", "{phrase}")?;"#)));
        }
    }
    #[test]
    fn the_two_chapters_split_where_the_teaching_turns() {
        let first = scene().split("impl SceneConstruct for TinyMultipliers").nth(1).unwrap();
        let (first, second) = first.split_once("impl SceneConstruct for WholeSum").unwrap();
        assert!(first.contains("04_tiny_multipliers") && first.contains("chain_appears(stage"));
        assert!(!first.contains("columns_fill"), "the limit waits for its own chapter");
        assert!(second.contains("05_whole_sum") && second.contains("columns_fill(stage"));
        assert!(second.contains("payoff(stage"));
        assert!(!second.contains("chain_appears"), "the formula is earned once");
    }
    #[test]
    fn the_think_gap_is_authored_and_the_answer_waits() {
        assert_in_order(&[
            "let question = k.text(stage, MUST_ARRIVE",
            "reveal(stage, question)?;",
            "k.hold(stage)?;",
            r#"k.mark(stage, "predict");"#,
            "k.music_breath(stage, 4.0)?;",
            "clear(stage, &[question])?;",
            r#"k.say(stage, "05_2")?;"#,
            "k.text(stage, NOT_ENOUGH",
        ]);
    }
    #[test]
    fn the_multiplier_is_earned_before_the_power_is_written() {
        assert_in_order(&[
            r#"until_sentence(stage, "04_1", 1)"#,
            "ADD_RISE",
            r#"until_sentence(stage, "04_1", 2)"#,
            "AS_MULTIPLY",
            r#"until_sentence(stage, "04_1", 3)"#,
            r#"until_sentence(stage, "04_1", 4)"#,
            "FOUR_STEPS",
            r#"k.say(stage, "04_2")?;"#,
            r#"until_sentence(stage, "04_2", 3)"#,
            "k.tex(stage, POWER",
        ]);
    }
    #[test]
    fn the_series_comparison_waits_for_the_counted_coefficient() {
        assert_in_order(&[
            r#"until_sentence(stage, "04_4", 2)"#,
            "mini_strip(stage",
            r#"until_sentence(stage, "04_4", 3)"#,
            "ways(count)",
            r#"until_sentence(stage, "04_4", 5)"#,
            "WORKED",
            "strip_coefficient(count)",
            "coefficient(count)",
        ]);
    }
    #[test]
    fn the_factorial_is_earned_by_the_double_count_before_the_chain_appears() {
        assert_in_order(&[
            r#"until_sentence(stage, "04_5", 2)"#,
            "k.tex(stage, PICKS,",
            r#"until_sentence(stage, "04_5", 3)"#,
            r#"["2", "1"]"#,
            "k.tex(stage, RECOUNT,",
            "k.tex(stage, CHOOSE,",
            r#"k.say(stage, "04_6")?;"#,
            "k.tex(stage, TERM_SHARED",
            r#"until_sentence(stage, "04_6", 1)"#,
            "k.tex(stage, TERM_CHAIN",
            r#"until_sentence(stage, "04_6", 3)"#,
            "k.tex(stage, CHAIN_WORKED",
            r#"until_sentence(stage, "04_6", 4)"#,
            "k.tex(stage, CHAIN_ZERO",
        ]);
    }
    #[test]
    fn the_warning_is_seen_before_the_ceilings_rescue_it() {
        assert_in_order(&[
            r#"k.say(stage, "05_2")?;"#,
            "warning_bars(stage, WARNING_COUNTS[0])",
            "TOTAL_ONE",
            r#"k.say(stage, "05_3")?;"#,
            "k.tex(stage, CHAIN_RANGE",
            r#"until_phrase(k, stage, "05_3", "and after that")?;"#,
            "k.tex(stage, CHAIN_EMPTY",
            "k.tex(stage, BOUNDED",
            "k.text(stage, NO_CEILING",
            "k.tex(stage, HARMONIC",
        ]);
    }
    #[test]
    fn the_cut_is_chosen_before_n_moves() {
        assert_in_order(&[
            r#"k.say(stage, "05_4")?;"#,
            "k.tex(stage, GAPS",
            r#"until_sentence(stage, "05_4", 2)"#,
            "aim: distance below",
            r#"until_sentence(stage, "05_4", 3)"#,
            "DashedLine::new",
            r#"until_sentence(stage, "05_4", 4)"#,
            "tail_print(EARLY_CUT)",
            r#"k.say(stage, "05_5")?;"#,
            r#"until_phrase(k, stage, "05_5", "so push")?;"#,
            "tail_print(CUT)",
            r#"until_sentence(stage, "05_5", 1)"#,
            "for (row, &n) in HEAD_COUNTS.iter().enumerate()",
            "head_gaps(RADIUS, n, CUT)",
            r#"until_phrase(k, stage, "05_5", "then make")?;"#,
            r#"until_sentence(stage, "05_5", 3)"#,
            "total_label(last)",
        ]);
        // Between the first picture of the cut and the sentence that closes the head, n never moves.
        let cut_first = scene().split("fn tail_then_head(").next().unwrap();
        let after_cut = cut_first.split("DashedLine::new").nth(1).unwrap();
        assert!(!after_cut.contains("term_bar("), "n moved before the cut was settled");
    }
    #[test]
    fn the_negative_input_uses_the_same_ceilings_and_is_measured() {
        assert_in_order(&[
            r#"k.say(stage, "05_6")?;"#,
            "BAR_INDEX_LOW_Y",
            r#"until_sentence(stage, "05_6", 1)"#,
            "tween(tracker, -RADIUS, 1.5)",
            r#"until_sentence(stage, "05_6", 2)"#,
            "step_by(2)",
            r#"until_sentence(stage, "05_6", 3)"#,
            "SAME_SIZE",
            r#"until_sentence(stage, "05_6", 4)"#,
            "distance(-RADIUS, last)",
        ]);
    }
    #[test]
    fn minus_three_prints_every_kind_of_base() {
        assert_in_order(&[
            r#"k.say(stage, "05_7")?;"#,
            "edge_base(n)",
            r#"until_sentence(stage, "05_7", 1)"#,
            "rows[0]",
            r#"until_sentence(stage, "05_7", 2)"#,
            "rows[1]",
            r#"until_phrase(k, stage, "05_7", "and three")?;"#,
            "rows[2]",
            r#"until_sentence(stage, "05_7", 4)"#,
            "NOT_POSITIVE",
            r#"until_sentence(stage, "05_7", 6)"#,
            "NO_CLIMB",
        ]);
    }
    #[test]
    fn the_debt_is_paid_last_in_its_own_words() {
        assert_eq!(SECOND_DEBT, "Why the stepped approximations reach E");
        assert_in_order(&[
            r#"k.say(stage, "05_8")?;"#,
            "k.tex(stage, LIMIT",
            r#"until_sentence(stage, "05_8", 1)"#,
            "k.text(stage, BIGGER",
            r#"until_sentence(stage, "05_8", 2)"#,
            "SPECIAL_CASE",
            r#"until_sentence(stage, "05_8", 3)"#,
            "TIME_SWAP",
            r#"until_sentence(stage, "05_8", 4)"#,
            "graph_walk(stage, GRAPH_LOW",
            r#"until_sentence(stage, "05_8", 5)"#,
            "k.text(stage, SECOND_DEBT",
            "Palette::green()",
            "k.finish(stage)",
        ]);
    }
    #[test]
    fn e_is_named_as_a_special_case_not_the_general_formula() {
        assert!(LIMIT.contains(r"\frac{x}{n}") && LIMIT.contains("E(x)"));
        assert!(SPECIAL_CASE.starts_with("x=1"));
    }
    #[test]
    fn outlines_and_open_paths_are_never_recoloured_with_a_fill() {
        // indicate and set_color fill what they recolour at the pin, so they may only reach text.
        assert_in_order(&["fn lit_cells(", ".set_stroke(Some(Palette::gold()), None, None)?"]);
        assert_in_order(&["fn dim_path(", ".set_stroke(Some(Palette::dim()), None, None)?"]);
        let lit = scene()
            .split("fn lit_cells(")
            .nth(1)
            .unwrap()
            .split("fn table_y(")
            .next()
            .unwrap();
        assert!(!lit.contains("indicate"));
    }
    #[test]
    fn live_bars_lose_their_updaters_before_they_fade() {
        assert_in_order(&["fn clear(", "stage.clear_updaters(m, true);", "fade_out("]);
    }
}
