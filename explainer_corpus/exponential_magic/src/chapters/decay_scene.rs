//! Native chapter 08: a removal rule, the curve it forces, and the assumptions it needs.
//! Source candidate with the review 713 repairs: not compiled, rendered or listened to.
//!
//! Each cue has one function, in film order, and each begins with its own `say`. The graph drawn in 08_1 stays until
//! the chapter ends. Curves, bars, arrows and readouts are drawn from `applications_model` through the functions
//! below, so a picture cannot disagree with the number beside it. Amounts are chosen relative units, not measurements.
use crate::applications_model as m;
use crate::phrase::until_phrase;
use explainer_kit::{Facts, Kit, Palette, Timed, add, ink_style, rect, stroke_style, tween, v};
use fmn::core::rate::linear;
use fmn::prelude::*;

pub struct DecayLaw {
    pub kit: Kit,
}

const TITLE_Y: f64 = 3.4;
const PLOT_LEFT: f64 = -6.0;
const PLOT_WIDTH: f64 = 6.6;
const PLOT_BOTTOM: f64 = -2.2;
const PLOT_HEIGHT: f64 = 4.4;
const TIME_MAX: f64 = 4.0;
const START: f64 = 100.0;
const SECOND_START: f64 = 50.0;
/// The rate constant that makes one unit of the time axis exactly one half-life.
const K: f64 = std::f64::consts::LN_2;
/// The opening curve is `second_order(START, OPENING_K, t)`: `k A0 = 1`, the curve 08_7 names.
const OPENING_K: f64 = 0.01;
const BAR_ONE: f64 = 1.45;
const BAR_TWO: f64 = 2.45;
const BAR_WIDTH: f64 = 0.62;
const BAR_READ_Y: f64 = 2.62;
/// The centre of the text column to the right of the bars.
const COL_X: f64 = 5.0;
/// Long strokes are cut into pieces of this many segments (kit rule for strokes under moving objects).
const PIECE: usize = 12;
const STEPS_PER_UNIT: f64 = 24.0;
const GOLD_SAMPLES: [f64; 4] = [0.0, 0.35, 0.7, 1.05];
const GOLD_RUN: f64 = 0.35;
const BLUE_SAMPLES: [f64; 4] = [0.0, 1.0, 2.0, 3.0];
const BLUE_RUN: f64 = 0.5;
/// Where the rate law is read while its equation appears: the arrow is still long there.
const PROBE_T: f64 = 0.8;
/// How far the matching points travel in 08_4, well short of the concealed endpoint.
const MATCH_T: f64 = 0.45;
const LATER_START: f64 = 1.5;
const LATER_SPAN: f64 = 2.5;
const LATER_POINTS: usize = 60;
/// The moment of the temperature change in 08_7.
const WARMED_AT: f64 = 1.5;
const RIGHT_LEFT: f64 = 1.7;
const RIGHT_WIDTH: f64 = 4.8;
const RIGHT_BOTTOM: f64 = -2.2;
const RIGHT_HEIGHT: f64 = 2.3;
const MOLECULES: usize = 20;
const MOLECULE_RUN: f64 = 3.6;
/// The multiplicative golden-ratio sequence gives irregular, reproducible waiting-time quantiles.
const GOLDEN: f64 = 0.618_033_988_749_894_9;

fn interval() -> f64 {
    m::half_life(K)
}
fn first_order(initial: f64, t: f64) -> f64 {
    initial * m::remaining(K, t)
}
/// A fixed removal whose rate equals the proportional rule's rate at the start; it stops at zero.
fn fixed_removal(t: f64) -> f64 {
    (START - K * START * t).max(0.0)
}
fn empty_time() -> f64 {
    1.0 / K
}
/// The fraction left under the second-order law, for the chosen `k = 1`.
fn second_order_fraction(initial: f64, t: f64) -> f64 {
    m::second_order(initial, 1.0, t) / initial
}
/// The first-order curve when `k` doubles at `WARMED_AT`.
fn warmed(t: f64) -> f64 {
    if t <= WARMED_AT {
        first_order(START, t)
    } else {
        first_order(START, WARMED_AT) * m::remaining(2.0 * K, t - WARMED_AT)
    }
}
/// One reproducible waiting time per molecule, sorted; `remaining(K, t)` of each is its quantile.
fn molecule_times() -> Vec<f64> {
    let mut times: Vec<f64> = (1..=MOLECULES)
        .map(|i| {
            let share = (i as f64 * GOLDEN).fract();
            -(1.0 - share).ln() / K
        })
        .collect();
    times.sort_by(f64::total_cmp);
    times
}
fn molecules_left(times: &[f64], t: f64) -> usize {
    times.iter().filter(|&&at| at > t).count()
}
/// An amount as the film writes it: whole numbers bare, otherwise one decimal.
fn amount_label(amount: f64) -> String {
    let rounded = (amount * 10.0).round() / 10.0;
    if (rounded - rounded.round()).abs() < 1e-9 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.1}")
    }
}
fn step_label(from: f64, to: f64) -> String {
    format!("{} → {}", amount_label(from), amount_label(to))
}
/// The fraction left after `n` half-lives, as TeX, from the model.
fn halves_tex(n: i32) -> String {
    let denominator = (1.0 / m::remaining(K, f64::from(n) * interval())).round() as u64;
    format!(r"\tfrac{{1}}{{{denominator}}}")
}
fn halves_text(n: i32) -> String {
    let denominator = (1.0 / m::remaining(K, f64::from(n) * interval())).round() as u64;
    if denominator == 1 {
        "1".to_owned()
    } else {
        format!("1/{denominator}")
    }
}
fn warmed_left_label() -> String {
    format!("{:.2}", m::remaining(2.0 * K, interval()))
}
fn second_order_label(initial: f64) -> String {
    format!("{:.2}", second_order_fraction(initial, 1.0))
}

/// Every number the chapter shows, plus the stage04 oracle's chemistry rows.
pub fn append_facts(f: &mut Facts) -> Result<(), String> {
    for n in 0..9 {
        f.add(
            &format!("chem_half_{n}"),
            format!("{:.10}", first_order(START, f64::from(n) * interval())),
        )?;
        f.add(
            &format!("chem_second_order_{n}"),
            format!("{:.10}", second_order_fraction(1.0, f64::from(n))),
        )?;
    }
    f.add("decay.interval", format!("{:.10}", interval()))?;
    f.add("decay.fixed.empty_time", format!("{:.10}", empty_time()))?;
    for n in 0..3 {
        let (from, to) = (first_order(START, f64::from(n)), first_order(START, f64::from(n + 1)));
        f.add(&format!("decay.step.{n}"), step_label(from, to))?;
    }
    f.add(
        "decay.second.step",
        step_label(SECOND_START, first_order(SECOND_START, interval())),
    )?;
    f.add("decay.warmed.left_after_old_half_life", warmed_left_label())?;
    f.add("decay.second_order.left_from_1", second_order_label(1.0))?;
    f.add("decay.second_order.left_from_half", second_order_label(0.5))?;
    let times = molecule_times();
    f.add("decay.molecules", MOLECULES)?;
    f.add("decay.molecules.left_at_end", molecules_left(&times, MOLECULE_RUN))?;
    Ok(())
}

fn to_scene(t: f64, amount: f64) -> Vec3 {
    v(
        PLOT_LEFT + PLOT_WIDTH * t / TIME_MAX,
        PLOT_BOTTOM + PLOT_HEIGHT * amount / START,
    )
}
fn right_scene(t: f64, fraction: f64) -> Vec3 {
    v(
        RIGHT_LEFT + RIGHT_WIDTH * t / TIME_MAX,
        RIGHT_BOTTOM + RIGHT_HEIGHT * fraction,
    )
}

fn stroke(a: Vec3, b: Vec3, color: Srgb, width: f64) -> VMobject {
    Line::new(a, b)
        .style(stroke_style(color, width))
        .build()
        .expect("a line has two distinct ends")
}
fn spot(at: Vec3, color: Srgb) -> VMobject {
    Circle::new().radius(0.09).style(ink_style(color)).build().moved_to(at)
}
fn arrow(a: Vec3, b: Vec3, color: Srgb) -> VMobject {
    Arrow::new(a, b)
        .buff(0.0)
        .thickness(3.5)
        .style(Style::default().fill(color, 1.0).stroke(color, 3.5, 1.0))
        .build()
        .expect("an arrow has two distinct ends")
}
fn curve_points(f: impl Fn(f64) -> Vec3, from: f64, to: f64) -> Vec<Vec3> {
    let steps = (((to - from) * STEPS_PER_UNIT).ceil() as usize).max(1);
    (0..=steps)
        .map(|i| f(from + (to - from) * i as f64 / steps as f64))
        .collect()
}
fn short_strokes(points: &[Vec3], color: Srgb, width: f64) -> VMobject {
    let mut parts = Vec::new();
    let mut start = 0;
    while start + 1 < points.len() {
        let end = (start + PIECE).min(points.len() - 1);
        parts.push(
            polyline(points[start..=end].to_vec())
                .style(stroke_style(color, width))
                .build(),
        );
        start = end;
    }
    v_group(parts)
}
/// `f` drawn from `from` up to `to`; nothing yet when the two agree.
fn drawn_curve(f: impl Fn(f64) -> Vec3, from: f64, to: f64, color: Srgb, width: f64) -> VMobject {
    if to - from < 1e-6 {
        return VMobject::new();
    }
    short_strokes(&curve_points(f, from, to), color, width)
}
/// A bar standing on the graph's zero line and drawn on the graph's amount scale.
fn column(x: f64, amount: f64, color: Srgb) -> VMobject {
    let height = (PLOT_HEIGHT * amount / START).max(0.004);
    rect(v(x - BAR_WIDTH / 2.0, PLOT_BOTTOM), BAR_WIDTH, height, ink_style(color)).expect("positive extents")
}
/// The tangent at (t, amount) followed for `run` of time at this removal `rate`, stopped at zero amount.
fn slope_ends(t: f64, amount: f64, rate: f64, run: f64) -> Option<(Vec3, Vec3)> {
    let run = if rate > 0.0 { run.min(amount / rate) } else { run };
    (run > 0.01).then(|| (to_scene(t, amount), to_scene(t + run, amount - rate * run)))
}
fn slope_arrow(t: f64, amount: f64, rate: f64, run: f64, lift: Vec3, color: Srgb) -> VMobject {
    match slope_ends(t, amount, rate, run) {
        Some((a, b)) => arrow(add(a, lift), add(b, lift), color),
        None => VMobject::new(),
    }
}

fn opening_curve(t: f64) -> VMobject {
    drawn_curve(
        |x| to_scene(x, m::second_order(START, OPENING_K, x)),
        0.0,
        t,
        Palette::dim(),
        3.0,
    )
}
fn fixed_curve(t: f64) -> VMobject {
    drawn_curve(|x| to_scene(x, fixed_removal(x)), 0.0, t, Palette::gold(), 3.5)
}
fn fixed_spot(t: f64) -> VMobject {
    spot(to_scene(t, fixed_removal(t)), Palette::gold())
}
fn fixed_column(t: f64) -> VMobject {
    column(BAR_TWO, fixed_removal(t), Palette::gold())
}
/// Parallel tangents left behind as the fixed rule runs, lifted off its line so they can be seen.
fn fixed_tangents(t: f64) -> VMobject {
    v_group(GOLD_SAMPLES.into_iter().filter(|&at| at <= t).map(|at| {
        slope_arrow(
            at,
            fixed_removal(at),
            K * START,
            GOLD_RUN,
            v(0.14, 0.2),
            Palette::gold(),
        )
    }))
}
fn model_curve(t: f64) -> VMobject {
    drawn_curve(|x| to_scene(x, first_order(START, x)), 0.0, t, Palette::blue(), 3.5)
}
fn model_spot(t: f64) -> VMobject {
    spot(to_scene(t, first_order(START, t)), Palette::blue())
}
fn model_column(t: f64) -> VMobject {
    column(BAR_ONE, first_order(START, t), Palette::blue())
}
/// How far the moving slope arrow follows its tangent: never past the end of the time axis.
fn model_run(t: f64) -> f64 {
    BLUE_RUN.min(TIME_MAX - t)
}
fn model_slope(t: f64) -> VMobject {
    let amount = first_order(START, t);
    slope_arrow(t, amount, K * amount, model_run(t), ORIGIN, Palette::blue())
}
/// The slopes left behind at each half-life, shorter and flatter each time.
fn model_tangents(t: f64) -> VMobject {
    v_group(BLUE_SAMPLES.into_iter().filter(|&at| at <= t).map(|at| {
        let amount = first_order(START, at);
        slope_arrow(at, amount, K * amount, BLUE_RUN, ORIGIN, Palette::dim())
    }))
}
fn first_interval_trace(t: f64) -> VMobject {
    drawn_curve(|x| to_scene(x, first_order(START, x)), 0.0, t, Palette::hero(), 5.0)
}
fn second_curve(t: f64) -> VMobject {
    drawn_curve(
        |x| to_scene(x, first_order(SECOND_START, x)),
        0.0,
        t,
        Palette::teal(),
        3.5,
    )
}
fn second_spot(t: f64) -> VMobject {
    spot(to_scene(t, first_order(SECOND_START, t)), Palette::teal())
}
fn second_column(t: f64) -> VMobject {
    column(BAR_TWO, first_order(SECOND_START, t), Palette::teal())
}
fn second_slope(t: f64) -> VMobject {
    let amount = first_order(SECOND_START, t);
    slope_arrow(t, amount, K * amount, BLUE_RUN, ORIGIN, Palette::teal())
}
fn warmed_curve(t: f64) -> VMobject {
    drawn_curve(|x| to_scene(x, warmed(x)), WARMED_AT, t, Palette::red(), 3.5)
}
fn single_stroke(points: Vec<Vec3>, color: Srgb, width: f64) -> VMobject {
    polyline(points).style(stroke_style(color, width)).build()
}
/// The first interval of `height` at time `s`, in one stroke with a fixed point count, so it can morph.
fn first_interval(height: impl Fn(f64) -> f64, color: Srgb) -> VMobject {
    let points = (0..=LATER_POINTS)
        .map(|i| {
            let x = interval() * i as f64 / LATER_POINTS as f64;
            to_scene(x, height(x))
        })
        .collect();
    single_stroke(points, color, 5.0)
}
/// The later segment, starting at time `at`, with its amount taken from `amount(s)` over `LATER_SPAN`.
fn later_segment(at: f64, amount: impl Fn(f64) -> f64) -> VMobject {
    let points = (0..=LATER_POINTS)
        .map(|i| {
            let s = LATER_SPAN * i as f64 / LATER_POINTS as f64;
            to_scene(at + s, amount(s))
        })
        .collect();
    single_stroke(points, Palette::hero(), 5.0)
}

fn mob(s: &mut Stage<'_>, shape: VMobject) -> Mob {
    s.arena_mut().add(shape)
}
fn show(s: &mut Stage<'_>, mobs: &[Mob], seconds: f64) -> fmn::Result<()> {
    let mut moves: Vec<Box<dyn Animation>> = Vec::new();
    for &one in mobs {
        let a = fade_in(s.arena_mut(), one, ORIGIN, 1.0)?;
        moves.push(s.prepare(a.rt(seconds))?);
    }
    s.play_prepared(moves)?;
    Ok(())
}
/// Fade these out. Updaters go first, so a live object fades instead of being redrawn at full opacity.
fn clear(s: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut moves: Vec<Box<dyn Animation>> = Vec::new();
    for &one in mobs {
        s.clear_updaters(one, true);
        let a = fade_out(s.arena_mut(), one, ORIGIN, 1.0)?;
        moves.push(s.prepare(a.rt(0.35))?);
    }
    s.play_prepared(moves)?;
    Ok(())
}
fn morph(s: &mut Stage<'_>, pairs: &[(Mob, Mob)], seconds: f64) -> fmn::Result<()> {
    let mut moves: Vec<Box<dyn Animation>> = Vec::new();
    for &(a, b) in pairs {
        moves.push(s.prepare(Transform::new(a, b).rt(seconds))?);
    }
    s.play_prepared(moves)?;
    Ok(())
}
fn shift(s: &mut Stage<'_>, mobs: &[Mob], dx: f64, seconds: f64) -> fmn::Result<()> {
    let mut moves: Vec<Box<dyn Animation>> = Vec::new();
    for &one in mobs {
        let a = one
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(seconds),
                ..AnimateArgs::default()
            })?
            .shift(v(dx, 0.0))?;
        moves.push(s.prepare(a)?);
    }
    s.play_prepared(moves)?;
    Ok(())
}
/// Model time advances at a constant speed: an eased clock would misstate the process.
fn run(s: &mut Stage<'_>, clock: Mob, to: f64, seconds: f64) -> fmn::Result<()> {
    let a = clock
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(seconds),
            rate_func: Some(linear),
            ..AnimateArgs::default()
        })?
        .set_value(to)?;
    let a = s.prepare(a)?;
    s.play_prepared(vec![a])?;
    Ok(())
}
fn rewind(s: &mut Stage<'_>, clock: Mob, to: f64, seconds: f64) -> fmn::Result<()> {
    let a = tween(clock, to, seconds)?;
    let a = s.prepare(a)?;
    s.play_prepared(vec![a])?;
    Ok(())
}
/// An object redrawn from `clock` every frame.
fn live(s: &mut Stage<'_>, clock: Mob, start: f64, draw: impl Fn(f64) -> VMobject + 'static) -> fmn::Result<Mob> {
    let shown = s.always_redraw(move |a| {
        let t = a.tracker_value(clock).unwrap_or(start);
        a.add(draw(t))
    });
    s.add_to_scene(shown)?;
    Ok(shown)
}
/// As `live`, but a still twin fades in first, so the object does not pop on. Never for an object empty at `start`.
fn enter(s: &mut Stage<'_>, clock: Mob, start: f64, draw: impl Fn(f64) -> VMobject + 'static) -> fmn::Result<Mob> {
    let twin = mob(s, draw(start));
    show(s, &[twin], 0.3)?;
    let shown = live(s, clock, start, draw)?;
    s.remove_from_scene(twin);
    Ok(shown)
}
/// A finished live trace becomes a still one in the same frame, so it stops costing a redraw.
fn settle(s: &mut Stage<'_>, drawing: Mob, still: Mob) -> fmn::Result<()> {
    s.clear_updaters(drawing, true);
    s.remove_from_scene(drawing);
    s.add_to_scene(still)?;
    Ok(())
}
fn live_amount(
    s: &mut Stage<'_>,
    k: &Kit,
    clock: Mob,
    amount: impl Fn(f64) -> f64 + 'static,
    color: Srgb,
    x: f64,
) -> fmn::Result<Mob> {
    k.live_value(
        s,
        move |a| amount(a.tracker_value(clock).unwrap_or(0.0)),
        1,
        false,
        25.0,
        color,
        v(x, BAR_READ_Y),
        ORIGIN,
    )
}
fn lines(s: &mut Stage<'_>, k: &Kit, rows: &[&str], size: f64, color: Srgb, y: f64) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        out.push(k.text(s, row, size, color, v(COL_X, y - 0.48 * i as f64))?);
    }
    Ok(out)
}
fn retitle(s: &mut Stage<'_>, k: &Kit, title: Mob, text: &str) -> fmn::Result<()> {
    let new = k.text(s, text, 34.0, Palette::ink(), v(0.0, TITLE_Y))?;
    morph(s, &[(title, new)], 0.5)
}
/// One interval wide and the full height of the plot, starting at time `t`.
fn band_shape(t: f64) -> fmn::Result<VMobject> {
    let width = PLOT_WIDTH * interval() / TIME_MAX;
    let style = Style::default()
        .fill(Palette::hero(), 0.14)
        .stroke(Palette::hero(), 0.0, 0.0);
    rect(to_scene(t, 0.0), width, PLOT_HEIGHT, style)
}
/// Where the label for the interval starting at `t` sits: inside the band, above the curve.
fn band_label_at(t: f64) -> Vec3 {
    to_scene(t + 0.62 * interval(), 88.0)
}

/// What stays on screen for the whole chapter.
struct Frame {
    title: Mob,
    y_name: Mob,
    x_name: Mob,
}

/// The rule cards and the fixed rule's objects, which leave together in 08_3.
struct FirstPass {
    leaving: Vec<Mob>,
}

/// The blue model sample: one clock drives all of it.
struct Sample {
    clock: Mob,
    curve: Mob,
    dot: Mob,
    arrow: Mob,
    ghosts: Mob,
    bar: Mob,
    read: Mob,
}

/// The tick labels and interval band built in 08_4.
struct Marks {
    y_labels: Vec<Mob>,
    band: Mob,
    first_label: Mob,
    highlight: Mob,
}

/// The second sample: a teal dot and bar on the blue sample's clock, and no trace until 08_5.
struct Second {
    dot: Mob,
    bar: Mob,
    read: Mob,
    rows: Vec<Mob>,
}

/// 08_1: a curve with no named cause, then the two instructions that could produce a fall.
fn opening(s: &mut Stage<'_>, k: &Kit) -> fmn::Result<(Frame, Vec<Mob>)> {
    k.say(s, "08_1")?;
    let title = k.text(
        s,
        "A falling curve does not name its cause",
        34.0,
        Palette::ink(),
        v(0.0, TITLE_Y),
    )?;
    let corner = to_scene(0.0, 0.0);
    let x_end = to_scene(TIME_MAX, 0.0);
    let y_name = k.text(s, "amount (relative units)", 22.0, Palette::dim(), v(-4.75, 2.62))?;
    let x_name = k.text(s, "time", 22.0, Palette::dim(), v(x_end[0] - 0.35, PLOT_BOTTOM - 0.72))?;
    let x_axis = stroke(corner, x_end, Palette::dim(), 2.0);
    let y_axis = stroke(corner, to_scene(0.0, START), Palette::dim(), 2.0);
    let x_axis = mob(s, x_axis);
    let y_axis = mob(s, y_axis);
    show(s, &[title, x_axis, y_axis, y_name, x_name], 0.5)?;
    // Draw the fall itself first: the viewer sees a shape before any explanation of it.
    let sketch = s.add_value_tracker(0.0);
    let drawing = live(s, sketch, 0.0, opening_curve)?;
    run(s, sketch, TIME_MAX, 3.0)?;
    let opening_still = mob(s, opening_curve(TIME_MAX));
    settle(s, drawing, opening_still)?;
    k.until_sentence(s, "08_1", 1)?;
    let ask = k.text(s, "What does the process do?", 28.0, Palette::hero(), v(COL_X, 2.0))?;
    show(s, &[ask], 0.4)?;
    k.until_sentence(s, "08_1", 2)?;
    let amount_rule = lines(
        s,
        k,
        &["Remove the same", "amount each second"],
        27.0,
        Palette::gold(),
        1.1,
    )?;
    show(s, &amount_rule, 0.4)?;
    until_phrase(k, s, "08_1", "or the same fraction")?;
    let fraction_rule = lines(
        s,
        k,
        &["Remove the same", "fraction of what remains"],
        27.0,
        Palette::blue(),
        -0.2,
    )?;
    show(s, &fraction_rule, 0.4)?;
    k.until_sentence(s, "08_1", 4)?;
    clear(s, &[opening_still, ask])?;
    let start = to_scene(0.0, START);
    let start_dot = mob(s, spot(start, Palette::ink()));
    let start_name = k.text(s, "same start", 24.0, Palette::ink(), add(start, v(1.0, 0.0)))?;
    show(s, &[start_dot, start_name], 0.4)?;
    let mut leaving = amount_rule;
    leaving.extend(fraction_rule);
    leaving.extend([start_dot, start_name]);
    Ok((Frame { title, y_name, x_name }, leaving))
}

/// 08_2: the fixed rule runs to zero and stops; then the proportional rule, with its slope shrinking.
fn two_rules(s: &mut Stage<'_>, k: &Kit, cards: Vec<Mob>) -> fmn::Result<(FirstPass, Sample)> {
    k.say(s, "08_2")?;
    let fixed = s.add_value_tracker(0.0);
    let gold_bar = enter(s, fixed, 0.0, fixed_column)?;
    let gold_read = live_amount(s, k, fixed, fixed_removal, Palette::gold(), BAR_TWO)?;
    let gold_ticks = live(s, fixed, 0.0, fixed_tangents)?;
    let gold_line = live(s, fixed, 0.0, fixed_curve)?;
    let gold_dot = live(s, fixed, 0.0, fixed_spot)?;
    // The picture must not go below zero: this clock stops where the rule stops.
    let end = empty_time();
    run(s, fixed, end, 5.2)?;
    let gold_still = mob(s, fixed_curve(end));
    settle(s, gold_line, gold_still)?;
    s.add_to_scene(gold_dot)?;
    let zero = to_scene(end, 0.0);
    let stops = k.text(s, "stops at zero", 22.0, Palette::gold(), add(zero, v(1.0, 0.3)))?;
    show(s, &[stops], 0.3)?;
    let mut leaving = cards;
    leaving.extend([gold_bar, gold_read, gold_ticks, gold_still, gold_dot, stops]);
    k.until_sentence(s, "08_2", 2)?;
    let clock = s.add_value_tracker(0.0);
    let bar = enter(s, clock, 0.0, model_column)?;
    let read = live_amount(s, k, clock, |t| first_order(START, t), Palette::blue(), BAR_ONE)?;
    let ghosts = live(s, clock, 0.0, model_tangents)?;
    let drawing = live(s, clock, 0.0, model_curve)?;
    let dot = live(s, clock, 0.0, model_spot)?;
    let arrow = enter(s, clock, 0.0, model_slope)?;
    k.until_sentence(s, "08_2", 3)?;
    run(s, clock, interval(), 2.2)?;
    k.until_sentence(s, "08_2", 4)?;
    run(s, clock, 3.0 * interval(), 2.4)?;
    k.until_sentence(s, "08_2", 5)?;
    let note = lines(s, k, &["The amount", "sets the slope"], 27.0, Palette::blue(), -1.6)?;
    show(s, &note, 0.4)?;
    run(s, clock, TIME_MAX, 1.4)?;
    let curve = mob(s, model_curve(TIME_MAX));
    settle(s, drawing, curve)?;
    s.add_to_scene(dot)?;
    s.add_to_scene(arrow)?;
    leaving.extend(note);
    Ok((
        FirstPass { leaving },
        Sample {
            clock,
            curve,
            dot,
            arrow,
            ghosts,
            bar,
            read,
        },
    ))
}

/// 08_3: the bar and slope arrow stay in place while the equation is written beside them.
fn name_the_law(s: &mut Stage<'_>, k: &Kit, f: &Frame, first: FirstPass, b: &Sample) -> fmn::Result<Vec<Mob>> {
    k.say(s, "08_3")?;
    let mut leaving = first.leaving;
    leaving.push(b.ghosts);
    clear(s, &leaving)?;
    retitle(s, k, f.title, "Removal proportional to the amount")?;
    rewind(s, b.clock, PROBE_T, 0.9)?;
    let names = [("A", Palette::blue()), ("k", Palette::hero())];
    let law = k.tex(s, r"A'(t)=-k\,A(t)", 40.0, Palette::ink(), &names, v(COL_X, 2.0))?;
    let on_bar = k.tex(s, "A", 30.0, Palette::blue(), &[], v(BAR_ONE, PLOT_BOTTOM - 0.38))?;
    let probe = first_order(START, PROBE_T);
    let on_arrow = k.tex(
        s,
        "A'",
        28.0,
        Palette::blue(),
        &[],
        add(to_scene(PROBE_T + 0.32, probe), v(0.32, 0.22)),
    )?;
    show(s, &[law, on_bar, on_arrow], 0.4)?;
    k.until_sentence(s, "08_3", 1)?;
    let terms = lines(
        s,
        k,
        &["minus: removal", "k is positive and stays constant"],
        24.0,
        Palette::dim(),
        1.25,
    )?;
    show(s, &terms, 0.4)?;
    k.until_sentence(s, "08_3", 2)?;
    let solved = k.tex(
        s,
        r"\text{already solved: } y'=c\,y,\ \ c=-k",
        26.0,
        Palette::dim(),
        &[],
        v(COL_X, 0.2),
    )?;
    show(s, &[solved], 0.4)?;
    k.until_sentence(s, "08_3", 3)?;
    let solution = k.tex(s, r"A(t)=A_0\,e^{-kt}", 42.0, Palette::ink(), &names, v(COL_X, -0.65))?;
    show(s, &[solution], 0.4)?;
    clear(s, &[on_arrow])?;
    // The point, bar and arrow keep moving under the formula: it describes this object, not a new one.
    run(s, b.clock, 2.4, 3.2)?;
    k.until_sentence(s, "08_3", 4)?;
    let order = k.text(s, "a first-order rate law", 28.0, Palette::green(), v(COL_X, -1.5))?;
    show(s, &[order], 0.4)?;
    k.until_sentence(s, "08_3", 5)?;
    let caveat = lines(
        s,
        k,
        &["Established from the reaction", "and its conditions, not assumed"],
        23.0,
        Palette::dim(),
        -2.2,
    )?;
    show(s, &caveat, 0.4)?;
    let mut cards = terms;
    cards.extend([law, on_bar, solved, solution, order]);
    cards.extend(caveat);
    Ok(cards)
}

/// 08_4: starts of 100 and 50 share an origin; the second sample's endpoint stays concealed through the hold.
fn second_sample(s: &mut Stage<'_>, k: &Kit, f: &Frame, b: &Sample, cards: &[Mob]) -> fmn::Result<(Marks, Second)> {
    k.say(s, "08_4")?;
    let mut leaving = cards.to_vec();
    leaving.push(b.arrow);
    clear(s, &leaving)?;
    retitle(s, k, f.title, "Predict it without solving again")?;
    rewind(s, b.clock, 0.0, 0.8)?;
    let intervals = k.text(
        s,
        "time (intervals)",
        22.0,
        Palette::dim(),
        v(to_scene(TIME_MAX, 0.0)[0] - 0.35, PLOT_BOTTOM - 0.72),
    )?;
    morph(s, &[(f.x_name, intervals)], 0.4)?;
    let mut scale = Vec::new();
    let mut y_labels = Vec::new();
    for n in 0..=3 {
        let amount = first_order(START, f64::from(n) * interval());
        y_labels.push(k.text(
            s,
            &amount_label(amount),
            20.0,
            Palette::dim(),
            v(PLOT_LEFT - 0.36, to_scene(0.0, amount)[1]),
        )?);
    }
    y_labels.push(k.text(s, "0", 20.0, Palette::dim(), v(PLOT_LEFT - 0.36, PLOT_BOTTOM))?);
    for n in 1..=4 {
        let x = to_scene(f64::from(n) * interval(), 0.0)[0];
        let tick = stroke(v(x, PLOT_BOTTOM), v(x, PLOT_BOTTOM - 0.1), Palette::dim(), 2.0);
        scale.push(mob(s, tick));
        scale.push(k.text(s, &n.to_string(), 20.0, Palette::dim(), v(x, PLOT_BOTTOM - 0.33))?);
    }
    // 25 and 12.5 wait for their reveals in 08_5: a printed 25 directly below the question's 50 would be the answer.
    let mut marks_in = scale;
    marks_in.extend([y_labels[0], y_labels[1], y_labels[4]]);
    show(s, &marks_in, 0.5)?;
    k.until_sentence(s, "08_4", 1)?;
    let band = band_shape(0.0)?;
    let band = mob(s, band);
    show(s, &[band], 0.4)?;
    s.add_to_scene(b.curve)?;
    s.add_to_scene(b.dot)?;
    let drawing = live(s, b.clock, 0.0, first_interval_trace)?;
    run(s, b.clock, interval(), 2.0)?;
    let highlight = mob(s, first_interval_trace(interval()));
    settle(s, drawing, highlight)?;
    s.add_to_scene(b.dot)?;
    let first_label = k.text(
        s,
        &step_label(START, first_order(START, interval())),
        24.0,
        Palette::hero(),
        band_label_at(0.0),
    )?;
    show(s, &[first_label], 0.3)?;
    k.until_sentence(s, "08_4", 2)?;
    rewind(s, b.clock, 0.0, 0.6)?;
    let bar = enter(s, b.clock, 0.0, second_column)?;
    let read = live_amount(
        s,
        k,
        b.clock,
        |t| first_order(SECOND_START, t),
        Palette::teal(),
        BAR_TWO,
    )?;
    let dot = enter(s, b.clock, 0.0, second_spot)?;
    let start_note = lines(s, k, &["Second sample:", "same conditions"], 26.0, Palette::teal(), 1.6)?;
    show(s, &start_note, 0.4)?;
    k.until_sentence(s, "08_4", 3)?;
    let blue_slope = enter(s, b.clock, 0.0, model_slope)?;
    let teal_slope = enter(s, b.clock, 0.0, second_slope)?;
    let rate_note = lines(
        s,
        k,
        &["half the amount,", "half the removal rate"],
        26.0,
        Palette::teal(),
        0.5,
    )?;
    show(s, &rate_note, 0.4)?;
    // The pair moves together only part of the way, and the teal point leaves no trace.
    run(s, b.clock, MATCH_T, 1.4)?;
    rewind(s, b.clock, 0.0, 0.7)?;
    k.until_sentence(s, "08_4", 4)?;
    clear(s, &[blue_slope, teal_slope])?;
    let later = k.text(s, "One interval later:", 26.0, Palette::teal(), v(COL_X, -0.7))?;
    let question = k.text(
        s,
        &format!("{} → ?", amount_label(SECOND_START)),
        40.0,
        Palette::teal(),
        v(COL_X, -1.35),
    )?;
    show(s, &[later, question], 0.4)?;
    k.hold(s)?;
    k.mark(s, "predict");
    k.music_breath(s, 4.0)?;
    let mut rows = start_note;
    rows.extend(rate_note);
    rows.extend([later, question]);
    Ok((
        Marks {
            y_labels,
            band,
            first_label,
            highlight,
        },
        Second { dot, bar, read, rows },
    ))
}

/// 08_5: the reveal, the half-sized copy, then equal intervals that multiply rather than subtract.
fn repeated_halving(
    s: &mut Stage<'_>,
    k: &Kit,
    f: &Frame,
    b: &Sample,
    marks: &Marks,
    second: Second,
) -> fmn::Result<Vec<Mob>> {
    k.say(s, "08_5")?;
    clear(s, &second.rows)?;
    let drawing = live(s, b.clock, 0.0, second_curve)?;
    run(s, b.clock, interval(), 1.0)?;
    let second_still = mob(s, second_curve(interval()));
    settle(s, drawing, second_still)?;
    s.add_to_scene(second.dot)?;
    s.add_to_scene(b.dot)?;
    let second_label = k.text(
        s,
        &step_label(SECOND_START, first_order(SECOND_START, interval())),
        38.0,
        Palette::teal(),
        v(COL_X, 1.4),
    )?;
    show(s, &[second_label, marks.y_labels[2]], 0.3)?;
    k.until_sentence(s, "08_5", 1)?;
    retitle(s, k, f.title, "The same interval keeps halving")?;
    let copy = first_interval(|x| first_order(START, x), Palette::hero());
    let copy = mob(s, copy);
    show(s, &[copy], 0.2)?;
    let half_sized = first_interval(|x| first_order(SECOND_START, x), Palette::hero());
    let half_sized = mob(s, half_sized);
    morph(s, &[(copy, half_sized)], 1.3)?;
    let copy_note = k.text(s, "a half-sized copy", 26.0, Palette::hero(), v(COL_X, 0.6))?;
    show(s, &[copy_note], 0.3)?;
    k.until_sentence(s, "08_5", 2)?;
    clear(s, &[second.dot, second.bar, second.read, copy])?;
    let step = PLOT_WIDTH * interval() / TIME_MAX;
    shift(s, &[marks.band], step, 0.7)?;
    run(s, b.clock, 2.0 * interval(), 1.3)?;
    let second_step = k.text(
        s,
        &step_label(first_order(START, interval()), first_order(START, 2.0 * interval())),
        24.0,
        Palette::hero(),
        band_label_at(interval()),
    )?;
    show(s, &[second_step], 0.3)?;
    until_phrase(k, s, "08_5", "then twenty-five")?;
    shift(s, &[marks.band], step, 0.7)?;
    run(s, b.clock, 3.0 * interval(), 1.3)?;
    let third_step = k.text(
        s,
        &step_label(
            first_order(START, 2.0 * interval()),
            first_order(START, 3.0 * interval()),
        ),
        24.0,
        Palette::hero(),
        band_label_at(2.0 * interval()),
    )?;
    show(s, &[third_step, marks.y_labels[3]], 0.3)?;
    k.until_sentence(s, "08_5", 3)?;
    let half = amount_label(SECOND_START);
    let subtraction = k.tex(
        s,
        &format!("{}-{half}-{half}-{half}", amount_label(START)),
        30.0,
        Palette::dim(),
        &[],
        v(COL_X, -0.3),
    )?;
    let strike = stroke(v(COL_X - 1.25, -0.3), v(COL_X + 1.25, -0.3), Palette::red(), 3.5);
    let strike = mob(s, strike);
    k.allow_crossing(strike);
    show(s, &[subtraction, strike], 0.4)?;
    k.until_sentence(s, "08_5", 4)?;
    let halving = halves_tex(1);
    let mut factors = Vec::new();
    for n in 0..3 {
        let x = to_scene((f64::from(n) + 0.5) * interval(), 0.0)[0];
        factors.push(k.tex(
            s,
            &format!(r"\times {halving}"),
            30.0,
            Palette::hero(),
            &[],
            v(x, PLOT_BOTTOM - 0.82),
        )?);
    }
    show(s, &factors, 0.4)?;
    k.until_sentence(s, "08_5", 5)?;
    let name_at = to_scene(2.5 * interval(), 58.0);
    let half_life = k.text(s, "half-life", 30.0, Palette::hero(), name_at)?;
    let scope = k.text(s, "constant k", 22.0, Palette::hero(), add(name_at, v(0.0, -0.42)))?;
    show(s, &[half_life, scope], 0.4)?;
    let mut leaving = vec![
        second_still,
        second_label,
        copy_note,
        second_step,
        third_step,
        subtraction,
        strike,
        half_life,
        scope,
    ];
    leaving.extend(factors);
    Ok(leaving)
}

/// 08_6: the same plot read as a fraction. A later segment moves, then rescales, and only then lands on the curve.
fn restart_the_clock(
    s: &mut Stage<'_>,
    k: &Kit,
    f: &Frame,
    b: &Sample,
    marks: &Marks,
    leaving: &[Mob],
) -> fmn::Result<Vec<Mob>> {
    k.say(s, "08_6")?;
    let mut going = leaving.to_vec();
    going.extend([marks.band, marks.first_label, marks.highlight, b.dot, b.bar, b.read]);
    clear(s, &going)?;
    retitle(s, k, f.title, "The clock does not care when it restarts")?;
    let mut relabels = Vec::new();
    for (n, &old) in marks.y_labels[..4].iter().enumerate() {
        let n = n as i32;
        let y = to_scene(0.0, first_order(START, f64::from(n) * interval()))[1];
        let new = k.text(s, &halves_text(n), 20.0, Palette::dim(), v(PLOT_LEFT - 0.36, y))?;
        relabels.push((old, new));
    }
    let new_name = k.text(s, "fraction remaining", 22.0, Palette::dim(), v(-4.95, 2.62))?;
    relabels.push((f.y_name, new_name));
    morph(s, &relabels, 0.6)?;
    k.until_sentence(s, "08_6", 1)?;
    let product = k.tex(
        s,
        r"e^{-k(s+t)}=e^{-ks}\,e^{-kt}",
        32.0,
        Palette::ink(),
        &[],
        v(COL_X - 1.3, 1.9),
    )?;
    let example = k.tex(
        s,
        &format!(r"{0}\times {0}={1}", halves_tex(1), halves_tex(2)),
        34.0,
        Palette::hero(),
        &[],
        v(COL_X - 1.3, 1.1),
    )?;
    show(s, &[product, example], 0.4)?;
    k.until_sentence(s, "08_6", 2)?;
    let rule_stays = lines(
        s,
        k,
        &["The amount changes.", "The rule stays."],
        26.0,
        Palette::dim(),
        0.3,
    )?;
    show(s, &rule_stays, 0.4)?;
    k.until_sentence(s, "08_6", 3)?;
    let at_later = first_order(START, LATER_START);
    let piece = later_segment(LATER_START, |x| first_order(START, LATER_START + x));
    let piece = mob(s, piece);
    show(s, &[piece], 0.4)?;
    // First the time shift alone, then the division: the two operations are separate on screen.
    let moved = later_segment(0.0, |x| first_order(START, LATER_START + x));
    let moved = mob(s, moved);
    morph(s, &[(piece, moved)], 1.3)?;
    let divide = k.text(
        s,
        "divide by its starting amount",
        24.0,
        Palette::hero(),
        v(COL_X - 1.0, -0.9),
    )?;
    show(s, &[divide], 0.3)?;
    let rescaled = later_segment(0.0, move |x| first_order(START, LATER_START + x) / at_later * START);
    let rescaled = mob(s, rescaled);
    morph(s, &[(piece, rescaled)], 1.5)?;
    k.until_sentence(s, "08_6", 4)?;
    let lands = k.text(
        s,
        "It lands on the original curve",
        26.0,
        Palette::hero(),
        v(COL_X - 1.0, -1.5),
    )?;
    show(s, &[lands], 0.4)?;
    let mut out = vec![product, example, piece, divide, lands];
    out.extend(rule_stays);
    Ok(out)
}

/// 08_7: one assumption at a time. First the constant changes; then the law itself, with the derivation visible.
fn assumptions(s: &mut Stage<'_>, k: &Kit, f: &Frame, leaving: &[Mob]) -> fmn::Result<Vec<Mob>> {
    k.say(s, "08_7")?;
    clear(s, leaving)?;
    retitle(s, k, f.title, "Change one assumption at a time")?;
    k.until_sentence(s, "08_7", 1)?;
    let change_x = to_scene(WARMED_AT, 0.0)[0];
    let change_line = DashedLine::new(v(change_x, PLOT_BOTTOM), v(change_x, 2.0))
        .dash_length(0.14)
        .style(stroke_style(Palette::red(), 2.5))
        .build()?;
    let change_line = mob(s, change_line);
    let changed = k.text(s, "warmer: k doubles", 24.0, Palette::red(), v(change_x + 1.25, 1.95))?;
    show(s, &[change_line, changed], 0.4)?;
    let warm = s.add_value_tracker(WARMED_AT);
    let drawing = live(s, warm, WARMED_AT, warmed_curve)?;
    run(s, warm, WARMED_AT + interval(), 1.8)?;
    let warm_still = mob(s, warmed_curve(WARMED_AT + interval()));
    settle(s, drawing, warm_still)?;
    let old = k.text(
        s,
        &format!("One old half-life now multiplies by {}", warmed_left_label()),
        24.0,
        Palette::red(),
        v(-1.1, 1.25),
    )?;
    show(s, &[old], 0.4)?;
    k.until_sentence(s, "08_7", 2)?;
    let centre = RIGHT_LEFT + RIGHT_WIDTH / 2.0;
    let derive = k.tex(
        s,
        r"A'=-kA^2\ \Rightarrow\ \left(\tfrac{1}{A}\right)'=k",
        30.0,
        Palette::ink(),
        &[],
        v(centre, 2.35),
    )?;
    let solved = k.tex(
        s,
        r"A(t)=\frac{A_0}{1+kA_0t}",
        32.0,
        Palette::ink(),
        &[],
        v(centre, 1.5),
    )?;
    let units = k.text(s, "This k has different units", 22.0, Palette::dim(), v(centre, 0.72))?;
    show(s, &[derive, solved, units], 0.4)?;
    let corner = right_scene(0.0, 0.0);
    let x_axis = stroke(corner, right_scene(TIME_MAX, 0.0), Palette::dim(), 2.0);
    let y_axis = stroke(corner, right_scene(0.0, 1.0), Palette::dim(), 2.0);
    let x_axis = mob(s, x_axis);
    let y_axis = mob(s, y_axis);
    let right_name = k.text(
        s,
        "fraction left",
        20.0,
        Palette::dim(),
        add(right_scene(0.0, 1.0), v(0.75, 0.2)),
    )?;
    let right_time = k.text(
        s,
        "time",
        20.0,
        Palette::dim(),
        add(right_scene(TIME_MAX, 0.0), v(-0.3, -0.3)),
    )?;
    let from_one = drawn_curve(
        |x| right_scene(x, second_order_fraction(1.0, x)),
        0.0,
        TIME_MAX,
        Palette::blue(),
        3.5,
    );
    let from_one = mob(s, from_one);
    show(s, &[x_axis, y_axis, right_name, right_time, from_one], 0.4)?;
    until_phrase(k, s, "08_7", "and the rescaled curves")?;
    let from_half = drawn_curve(
        |x| right_scene(x, second_order_fraction(0.5, x)),
        0.0,
        TIME_MAX,
        Palette::teal(),
        3.5,
    );
    let from_half = mob(s, from_half);
    let name_one = k.text(
        s,
        "started at 1",
        21.0,
        Palette::blue(),
        add(
            right_scene(TIME_MAX, second_order_fraction(1.0, TIME_MAX)),
            v(-0.75, -0.26),
        ),
    )?;
    let name_half = k.text(
        s,
        "started at 1/2",
        21.0,
        Palette::teal(),
        add(
            right_scene(TIME_MAX, second_order_fraction(0.5, TIME_MAX)),
            v(-0.85, 0.42),
        ),
    )?;
    show(s, &[from_half, name_one, name_half], 0.4)?;
    k.until_sentence(s, "08_7", 3)?;
    retitle(s, k, f.title, "A precise rule forces the shape")?;
    let echo = k.text(
        s,
        "the opening curve had this law",
        24.0,
        Palette::dim(),
        v(centre, -2.95),
    )?;
    show(s, &[echo], 0.4)?;
    Ok(vec![
        change_line,
        changed,
        warm_still,
        old,
        derive,
        solved,
        units,
        x_axis,
        y_axis,
        right_name,
        right_time,
        from_one,
        from_half,
        name_one,
        name_half,
        echo,
    ])
}

/// 08_8: the smooth curve and bar beside twenty molecules that do not change in step.
fn macroscopic(s: &mut Stage<'_>, k: &Kit, f: &Frame, b: &Sample, marks: &Marks, leaving: &[Mob]) -> fmn::Result<()> {
    k.say(s, "08_8")?;
    clear(s, leaving)?;
    retitle(s, k, f.title, "The smooth curve is a macroscopic model")?;
    // The bar, its readout and the step trace are amounts, so the axis goes back to reading amounts.
    let mut relabels = Vec::new();
    for (n, &old) in marks.y_labels[..4].iter().enumerate() {
        let amount = first_order(START, n as f64 * interval());
        let y = to_scene(0.0, amount)[1];
        let new = k.text(s, &amount_label(amount), 20.0, Palette::dim(), v(PLOT_LEFT - 0.36, y))?;
        relabels.push((old, new));
    }
    let new_name = k.text(s, "amount (relative units)", 22.0, Palette::dim(), v(-4.75, 2.62))?;
    relabels.push((f.y_name, new_name));
    morph(s, &relabels, 0.6)?;
    rewind(s, b.clock, 0.0, 0.5)?;
    let bar = enter(s, b.clock, 0.0, model_column)?;
    let read = live_amount(s, k, b.clock, |t| first_order(START, t), Palette::blue(), BAR_ONE)?;
    let dot = enter(s, b.clock, 0.0, model_spot)?;
    k.until_sentence(s, "08_8", 1)?;
    let times = molecule_times();
    let field_times = times.clone();
    let field = enter(s, b.clock, 0.0, move |t| {
        v_group(field_times.iter().enumerate().map(|(i, &at)| {
            let (row, col) = (i / 5, i % 5);
            let at_point = v(3.6 + 0.62 * col as f64, 1.35 - 0.62 * row as f64);
            spot(at_point, if at > t { Palette::blue() } else { Palette::faint() })
        }))
    })?;
    let step_times = times;
    let steps = live(s, b.clock, 0.0, move |t| {
        let mut points = vec![to_scene(0.0, START)];
        let mut amount = START;
        for &at in step_times.iter().take_while(|&&at| at <= t) {
            points.push(to_scene(at, amount));
            amount -= START / MOLECULES as f64;
            points.push(to_scene(at, amount));
        }
        points.push(to_scene(t, amount));
        if t > 1e-6 {
            short_strokes(&points, Palette::teal(), 2.5)
        } else {
            VMobject::new()
        }
    })?;
    let illustrative = k.text(s, "one illustrative run", 22.0, Palette::teal(), v(COL_X - 0.2, -1.5))?;
    show(s, &[illustrative], 0.3)?;
    run(s, b.clock, MOLECULE_RUN, 5.5)?;
    k.until_sentence(s, "08_8", 2)?;
    let meaning = lines(
        s,
        k,
        &["The bar is an amount,", "not a forecast of each molecule"],
        24.0,
        Palette::ink(),
        -2.1,
    )?;
    show(s, &meaning, 0.4)?;
    k.until_sentence(s, "08_8", 3)?;
    until_phrase(k, s, "08_8", "where the quantity grows")?;
    let mut going = vec![field, steps, illustrative, bar, read, dot];
    going.extend(meaning);
    clear(s, &going)?;
    let next = k.text(
        s,
        "Next: a quantity rising toward a ceiling",
        30.0,
        Palette::green(),
        v(2.3, 0.6),
    )?;
    show(s, &[next], 0.4)?;
    Ok(())
}

impl SceneConstruct for DecayLaw {
    fn name(&self) -> &str {
        "08_decay_law"
    }
    fn construct(&mut self, s: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let (frame, cards) = opening(s, &k)?;
        let (first, sample) = two_rules(s, &k, cards)?;
        let law_cards = name_the_law(s, &k, &frame, first, &sample)?;
        let (marks, second) = second_sample(s, &k, &frame, &sample, &law_cards)?;
        let leaving = repeated_halving(s, &k, &frame, &sample, &marks, second)?;
        let leaving = restart_the_clock(s, &k, &frame, &sample, &marks, &leaving)?;
        let leaving = assumptions(s, &k, &frame, &leaving)?;
        macroscopic(s, &k, &frame, &sample, &marks, &leaving)?;
        k.finish(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The scene's code, without this test module.
    fn scene() -> &'static str {
        include_str!("decay_scene.rs")
            .split("\n#[cfg(test)]\nmod tests {")
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
    fn every_cue_is_said_once_and_in_film_order() {
        for n in 1..=8 {
            assert_eq!(
                scene().matches(&format!(r#"k.say(s, "08_{n}")?;"#)).count(),
                1,
                "08_{n}"
            );
        }
        let cues: Vec<String> = (1..=8).map(|n| format!(r#"k.say(s, "08_{n}")?;"#)).collect();
        assert_in_order(&cues.iter().map(String::as_str).collect::<Vec<_>>());
    }
    #[test]
    fn the_endpoint_stays_concealed_through_the_authored_hold() {
        assert_in_order(&[
            "fn second_sample(",
            "run(s, b.clock, MATCH_T, 1.4)?;",
            "rewind(s, b.clock, 0.0, 0.7)?;",
            "k.hold(s)?;",
            r#"k.mark(s, "predict");"#,
            "k.music_breath(s, 4.0)?;",
            r#"k.say(s, "08_5")?;"#,
            "live(s, b.clock, 0.0, second_curve)?",
        ]);
        assert!(MATCH_T < interval());
        let hold = scene().find("k.music_breath").expect("the hold exists");
        assert!(
            !scene()[..hold].contains("second_curve)?"),
            "no second trace before the hold"
        );
    }
    #[test]
    fn the_answer_stays_off_the_amount_axis_until_its_reveal() {
        assert!(!scene().contains("marks_in.extend(&y_labels)"));
        assert_in_order(&[
            "marks_in.extend([y_labels[0], y_labels[1], y_labels[4]]);",
            r#"k.mark(s, "predict");"#,
            r#"k.say(s, "08_5")?;"#,
            "run(s, b.clock, interval(), 1.0)?;",
            "show(s, &[second_label, marks.y_labels[2]], 0.3)?;",
            "show(s, &[third_step, marks.y_labels[3]], 0.3)?;",
        ]);
        assert_eq!(amount_label(first_order(START, 2.0 * interval())), "25");
        assert_eq!(amount_label(first_order(START, 3.0 * interval())), "12.5");
    }
    #[test]
    fn the_moving_slope_arrow_stops_at_the_end_of_the_time_axis() {
        let end_x = to_scene(TIME_MAX, 0.0)[0];
        for i in 0..=400 {
            let t = TIME_MAX * f64::from(i) / 400.0;
            let amount = first_order(START, t);
            if let Some((_, tip)) = slope_ends(t, amount, K * amount, model_run(t)) {
                assert!(tip[0] <= end_x + 1e-9, "t = {t}");
            }
        }
        let last = first_order(START, TIME_MAX);
        assert!(slope_ends(TIME_MAX, last, K * last, model_run(TIME_MAX)).is_none());
    }
    #[test]
    fn the_title_follows_the_teaching_stage() {
        assert_in_order(&[
            r#"k.say(s, "08_3")?;"#,
            r#"retitle(s, k, f.title, "Removal proportional to the amount")?;"#,
            r#"k.say(s, "08_4")?;"#,
            r#"retitle(s, k, f.title, "Predict it without solving again")?;"#,
            r#"k.say(s, "08_5")?;"#,
            r#"k.until_sentence(s, "08_5", 1)?;"#,
            r#"retitle(s, k, f.title, "The same interval keeps halving")?;"#,
            r#"k.say(s, "08_6")?;"#,
            r#"retitle(s, k, f.title, "The clock does not care when it restarts")?;"#,
        ]);
    }
    #[test]
    fn the_warmed_curve_says_what_its_quarter_is() {
        assert!(scene().contains("One old half-life now multiplies by {}"));
        assert!(!scene().contains("now leaves"));
    }
    #[test]
    fn the_right_panel_names_its_time_axis() {
        assert_in_order(&[
            r#"k.say(s, "08_7")?;"#,
            "right_scene(TIME_MAX, 0.0)",
            r#""time""#,
            "show(s, &[x_axis, y_axis, right_name, right_time, from_one], 0.4)?;",
        ]);
    }
    #[test]
    fn the_amount_axis_returns_for_the_bar_in_08_8() {
        assert_in_order(&[
            "fn macroscopic(",
            r#"k.say(s, "08_8")?;"#,
            "marks.y_labels[..4]",
            "amount_label(amount)",
            r#""amount (relative units)""#,
            "relabels.push((f.y_name, new_name));",
            "morph(s, &relabels, 0.6)?;",
            r#"k.until_sentence(s, "08_8", 1)?;"#,
        ]);
    }
    #[test]
    fn the_later_segment_moves_before_it_rescales() {
        assert_in_order(&[
            "later_segment(0.0, |x| first_order(START, LATER_START + x))",
            "morph(s, &[(piece, moved)], 1.3)?;",
            "/ at_later * START",
            "morph(s, &[(piece, rescaled)], 1.5)?;",
        ]);
    }
    #[test]
    fn the_time_axis_is_one_half_life_per_unit() {
        assert!((interval() - 1.0).abs() < 1e-15);
        assert_eq!(amount_label(first_order(START, 3.0 * interval())), "12.5");
        assert_eq!(step_label(100.0, 50.0), "100 → 50");
        assert_eq!(halves_text(2), "1/4");
        assert_eq!(halves_tex(1), r"\tfrac{1}{2}");
    }
    #[test]
    fn the_fixed_rule_never_goes_negative_and_starts_on_the_same_slope() {
        assert!(fixed_removal(empty_time()).abs() < 1e-9);
        assert_eq!(fixed_removal(10.0), 0.0);
        assert!((fixed_removal(1e-6) - first_order(START, 1e-6)).abs() < 1e-8);
        assert!(GOLD_SAMPLES[3] + GOLD_RUN < empty_time());
    }
    #[test]
    fn a_slope_arrow_is_clipped_at_zero() {
        let (_, end) = slope_ends(1.4, fixed_removal(1.4), K * START, GOLD_RUN).expect("still above zero");
        assert!(end[1] >= PLOT_BOTTOM - 1e-12);
        assert!(slope_ends(empty_time(), 0.0, K * START, GOLD_RUN).is_none());
    }
    #[test]
    fn the_rescaled_later_segment_is_the_original_curve() {
        for i in 0..=LATER_POINTS {
            let x = LATER_SPAN * i as f64 / LATER_POINTS as f64;
            let rescaled = first_order(START, LATER_START + x) / first_order(START, LATER_START) * START;
            assert!((rescaled - first_order(START, x)).abs() < 1e-11);
        }
    }
    #[test]
    fn rescaled_second_order_curves_do_not_coincide() {
        assert_eq!(second_order_label(1.0), "0.50");
        assert_eq!(second_order_label(0.5), "0.67");
        assert_eq!(warmed_left_label(), "0.25");
        assert!((m::second_order(START, OPENING_K, 2.0) / START - second_order_fraction(1.0, 2.0)).abs() < 1e-12);
    }
    #[test]
    fn the_warmed_curve_is_continuous_at_the_change() {
        assert_eq!(warmed(WARMED_AT), first_order(START, WARMED_AT));
        assert!((warmed(WARMED_AT + 1e-9) - warmed(WARMED_AT)).abs() < 1e-6);
    }
    #[test]
    fn the_molecules_do_not_change_in_step() {
        let times = molecule_times();
        assert_eq!(times.len(), MOLECULES);
        assert!(times.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(molecules_left(&times, 0.0), MOLECULES);
        for (i, &at) in times.iter().enumerate() {
            assert!((0.0..1.0).contains(&m::remaining(K, at)), "molecule {i}");
        }
        assert!(molecules_left(&times, MOLECULE_RUN) > 0);
    }
    #[test]
    fn facts_register_without_a_duplicate_key() {
        let mut f = Facts::new();
        append_facts(&mut f).expect("no duplicate keys");
        assert_eq!(f.get("decay.step.2"), Some("25 → 12.5"));
        assert_eq!(f.get("chem_half_1"), Some("50.0000000000"));
    }
}
