//! Native chapter 09: charging a capacitor, and the quantity that actually decays.
//! Source candidate with the review 713 repairs: not compiled, rendered or listened to.
//!
//! Each cue has one function, in film order, and each begins with its own `say`. One clock drives the capacitor's
//! charge, the current arrow, the voltage column and every trace, all through `applications_model`, so the linked
//! readouts cannot drift apart. The component values are chosen model values, not measurements of a device.
use crate::applications_model as m;
use explainer_kit::{Facts, Kit, Palette, Timed, add, ink_style, rect, stroke_style, thousands, tween, v};
use fmn::core::rate::linear;
use fmn::prelude::*;

pub struct ChargingGap {
    pub kit: Kit,
}

const TITLE_Y: f64 = 3.4;
const GRAPH_LEFT: f64 = 0.75;
const GRAPH_WIDTH: f64 = 6.0;
const GRAPH_BOTTOM: f64 = -2.2;
const GRAPH_HEIGHT: f64 = 4.4;
const TIME_MAX: f64 = 6.0;
const VOLT_MAX: f64 = 13.0;
/// The voltage column shares the graph's vertical scale, directly left of it.
const COLUMN_LEFT: f64 = -0.45;
const COLUMN_WIDTH: f64 = 0.45;
const LOOP_LEFT: f64 = -6.2;
const LOOP_RIGHT: f64 = -2.2;
const LOOP_TOP: f64 = 2.4;
const LOOP_BOTTOM: f64 = 0.2;
const BATTERY_LONG: f64 = 1.42;
const BATTERY_SHORT: f64 = 1.18;
const PLATE_TOP: f64 = 1.52;
const PLATE_BOTTOM: f64 = 1.08;
const RESISTOR_LEFT: f64 = -4.9;
const RESISTOR_RIGHT: f64 = -3.5;
const RESISTOR_TEETH: usize = 6;
const ARROW_X: f64 = -4.2;
const ARROW_Y: f64 = -0.13;
const ARROW_FULL: f64 = 2.6;
const CHARGES: usize = 8;
const CHARGE_LEFT: f64 = -2.59;
const CHARGE_STEP: f64 = 0.11;
/// Under the circuit: equation rows, then the current panel, the table and the settling notes.
const FORMULA_X: f64 = -5.0;
const TAG_X: f64 = -2.4;
const NOTE_X: f64 = -3.85;
const ROWS: [f64; 3] = [-1.05, -1.95, -2.85];
const MINI_LEFT: f64 = -6.0;
const MINI_WIDTH: f64 = 4.4;
const MINI_BOTTOM: f64 = -3.0;
const MINI_HEIGHT: f64 = 1.8;
/// The free area of the graph, between the rising voltage and the falling gap.
const FREE_X: f64 = 4.8;
const READ_COLUMNS: [f64; 3] = [1.7, 3.7, 5.7];
const READ_NAME_Y: f64 = 2.95;
const READ_VALUE_Y: f64 = 2.5;
const PIECE: usize = 12;
const STEPS_PER_SECOND: f64 = 24.0;
/// An explicitly chosen instrument resolution for 09_8, in volts. It is not a property of the model.
const RESOLUTION_V: f64 = 0.05;
/// The window of the main graph that the 09_8 inset magnifies.
const ZOOM_START_S: f64 = 4.0;
const ZOOM_END_S: f64 = TIME_MAX;
const ZOOM_LOW_V: f64 = 11.7;
const ZOOM_HIGH_V: f64 = 12.02;
const ZOOM_LEFT: f64 = 3.2;
const ZOOM_WIDTH: f64 = 3.5;
const ZOOM_BOTTOM: f64 = -1.35;
const ZOOM_HEIGHT: f64 = 2.5;

fn battery() -> f64 {
    m::DEMO.battery_v
}
fn state(t: f64) -> m::State {
    m::DEMO.state(t)
}
fn tau() -> f64 {
    state(0.0).time_constant_s
}
/// The demo circuit with its resistance multiplied by `factor`.
fn with_resistance(factor: f64) -> m::Circuit {
    m::Circuit {
        resistance_ohm: factor * m::DEMO.resistance_ohm,
        ..m::DEMO
    }
}
fn with_capacitance(factor: f64) -> m::Circuit {
    m::Circuit {
        capacitance_f: factor * m::DEMO.capacitance_f,
        ..m::DEMO
    }
}
/// When the gap first falls to the chosen resolution, from the model's own exponential.
fn settle_time() -> f64 {
    tau() * (battery() / RESOLUTION_V).ln()
}
fn milliamps(c: m::Circuit, t: f64) -> f64 {
    c.state(t).current_a * 1e3
}
fn time_label(c: m::Circuit) -> String {
    format!("{:.0} s", c.state(0.0).time_constant_s)
}
fn current_label(c: m::Circuit) -> String {
    format!("{:.2} mA", milliamps(c, 0.0))
}
fn ohms_label(c: m::Circuit) -> String {
    format!("{} ohms", thousands(c.resistance_ohm.round() as u64))
}
fn farads_label(c: m::Circuit) -> String {
    format!("{:.0} microfarads", c.capacitance_f * 1e6)
}
fn battery_label() -> String {
    format!("{:.0} V", battery())
}
fn percent_label(fraction: f64) -> String {
    format!("{:.1}%", 100.0 * fraction)
}
fn gap_left_label() -> String {
    percent_label(state(tau()).gap_v / battery())
}
fn filled_label() -> String {
    percent_label(state(tau()).voltage_v / battery())
}
fn one_tau_volts_label() -> String {
    format!("{:.2} V", state(tau()).voltage_v)
}
fn resolution_label() -> String {
    format!("{RESOLUTION_V:.2} V")
}
fn settle_label() -> String {
    format!("{:.2} s", settle_time())
}
fn final_gap_label() -> String {
    format!("{:.3} V", state(ZOOM_END_S).gap_v)
}
fn seconds_label(t: f64) -> String {
    format!("{t:.0} s")
}
fn window_low_label() -> String {
    format!("{ZOOM_LOW_V:.1} V")
}
fn rc_tex() -> String {
    format!(
        r"RC=\text{{{}}}\times\text{{{}}}=\text{{{:.0} second}}",
        ohms_label(m::DEMO),
        farads_label(m::DEMO),
        tau()
    )
}

/// Every number the chapter shows, plus the stage04 oracle's circuit rows.
pub fn append_facts(f: &mut Facts) -> Result<(), String> {
    f.add("rc_tau_s", format!("{:.10}", tau()))?;
    for n in 0..=100_u32 {
        let now = state(f64::from(n) / 10.0);
        f.add(&format!("rc_{n:03}_voltage_v"), format!("{:.10}", now.voltage_v))?;
        f.add(&format!("rc_{n:03}_gap_v"), format!("{:.10}", now.gap_v))?;
        f.add(&format!("rc_{n:03}_current_a"), format!("{:.10}", now.current_a))?;
        f.add(&format!("rc_{n:03}_charge_c"), format!("{:.10}", now.charge_c))?;
    }
    f.add("circuit.display.battery", battery_label())?;
    f.add("circuit.display.ohms", ohms_label(m::DEMO))?;
    f.add("circuit.display.ohms_doubled", ohms_label(with_resistance(2.0)))?;
    f.add("circuit.display.farads", farads_label(m::DEMO))?;
    f.add("circuit.display.farads_doubled", farads_label(with_capacitance(2.0)))?;
    f.add("circuit.display.gap_left", gap_left_label())?;
    f.add("circuit.display.filled", filled_label())?;
    f.add("circuit.display.one_tau_volts", one_tau_volts_label())?;
    f.add("circuit.display.tau", time_label(m::DEMO))?;
    f.add("circuit.display.tau_r_doubled", time_label(with_resistance(2.0)))?;
    f.add("circuit.display.tau_c_doubled", time_label(with_capacitance(2.0)))?;
    f.add("circuit.display.start_current", current_label(m::DEMO))?;
    f.add(
        "circuit.display.start_current_r_doubled",
        current_label(with_resistance(2.0)),
    )?;
    f.add(
        "circuit.display.start_current_c_doubled",
        current_label(with_capacitance(2.0)),
    )?;
    f.add("circuit.display.resolution", resolution_label())?;
    f.add("circuit.display.settle_time", settle_label())?;
    f.add("circuit.display.final_gap", final_gap_label())?;
    f.add("circuit.display.window_low", window_low_label())?;
    f.add("circuit.display.window_start", seconds_label(ZOOM_START_S))?;
    f.add("circuit.display.window_end", seconds_label(ZOOM_END_S))?;
    Ok(())
}

fn to_graph(t: f64, volts: f64) -> Vec3 {
    v(
        GRAPH_LEFT + GRAPH_WIDTH * t / TIME_MAX,
        GRAPH_BOTTOM + GRAPH_HEIGHT * volts / VOLT_MAX,
    )
}
fn to_mini(t: f64, current_ma: f64) -> Vec3 {
    v(
        MINI_LEFT + MINI_WIDTH * t / TIME_MAX,
        MINI_BOTTOM + MINI_HEIGHT * current_ma / milliamps(m::DEMO, 0.0),
    )
}
fn to_zoom(t: f64, volts: f64) -> Vec3 {
    v(
        ZOOM_LEFT + ZOOM_WIDTH * (t - ZOOM_START_S) / (ZOOM_END_S - ZOOM_START_S),
        ZOOM_BOTTOM + ZOOM_HEIGHT * (volts - ZOOM_LOW_V) / (ZOOM_HIGH_V - ZOOM_LOW_V),
    )
}

fn stroke(a: Vec3, b: Vec3, color: Srgb, width: f64) -> VMobject {
    Line::new(a, b)
        .style(stroke_style(color, width))
        .build()
        .expect("a line has two distinct ends")
}
fn spot(at: Vec3, radius: f64, color: Srgb) -> VMobject {
    Circle::new()
        .radius(radius)
        .style(ink_style(color))
        .build()
        .moved_to(at)
}
fn arrow(a: Vec3, b: Vec3, color: Srgb) -> VMobject {
    Arrow::new(a, b)
        .buff(0.0)
        .thickness(4.0)
        .style(Style::default().fill(color, 1.0).stroke(color, 4.0, 1.0))
        .build()
        .expect("an arrow has two distinct ends")
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
    let steps = (((to - from) * STEPS_PER_SECOND).ceil() as usize).max(1);
    let points: Vec<Vec3> = (0..=steps)
        .map(|i| f(from + (to - from) * i as f64 / steps as f64))
        .collect();
    short_strokes(&points, color, width)
}
fn voltage_curve(t: f64) -> VMobject {
    drawn_curve(|x| to_graph(x, state(x).voltage_v), 0.0, t, Palette::blue(), 3.5)
}
fn gap_curve(t: f64) -> VMobject {
    drawn_curve(|x| to_graph(x, state(x).gap_v), 0.0, t, Palette::hero(), 3.5)
}
fn current_curve(t: f64) -> VMobject {
    drawn_curve(|x| to_mini(x, milliamps(m::DEMO, x)), 0.0, t, Palette::green(), 3.0)
}
fn slower_voltage_curve(t: f64) -> VMobject {
    drawn_curve(
        |x| to_graph(x, with_resistance(2.0).state(x).voltage_v),
        0.0,
        t,
        Palette::teal(),
        3.5,
    )
}
fn slower_current_curve(t: f64) -> VMobject {
    drawn_curve(
        |x| to_mini(x, milliamps(with_resistance(2.0), x)),
        0.0,
        t,
        Palette::teal(),
        3.0,
    )
}
/// The slower trace with its times divided by `squeeze`, as one stroke with a fixed point count, so it can morph.
fn slower_voltage_stroke(squeeze: f64) -> VMobject {
    let points: Vec<Vec3> = (0..=120)
        .map(|i| {
            let x = TIME_MAX * f64::from(i) / 120.0;
            to_graph(x / squeeze, with_resistance(2.0).state(x).voltage_v)
        })
        .collect();
    polyline(points).style(stroke_style(Palette::teal(), 3.5)).build()
}
fn voltage_spot(t: f64) -> VMobject {
    spot(to_graph(t, state(t).voltage_v), 0.09, Palette::blue())
}
fn gap_spot(t: f64) -> VMobject {
    spot(to_graph(t, state(t).gap_v), 0.09, Palette::hero())
}
fn current_spot(t: f64) -> VMobject {
    spot(to_mini(t, milliamps(m::DEMO, t)), 0.08, Palette::green())
}
fn zoom_curve(t: f64) -> VMobject {
    drawn_curve(
        |x| to_zoom(x, state(x).voltage_v),
        ZOOM_START_S,
        t,
        Palette::blue(),
        3.0,
    )
}
fn zoom_spot(t: f64) -> VMobject {
    spot(to_zoom(t, state(t).voltage_v), 0.08, Palette::blue())
}
/// The gap drawn on the graph at this moment: from the capacitor's voltage up to the battery's.
fn gap_segment(t: f64) -> VMobject {
    let now = state(t);
    if now.gap_v < 0.02 {
        return VMobject::new();
    }
    stroke(to_graph(t, now.voltage_v), to_graph(t, battery()), Palette::hero(), 4.0)
}
fn voltage_column(t: f64) -> VMobject {
    let height = GRAPH_HEIGHT * state(t).voltage_v / VOLT_MAX;
    rect(
        v(COLUMN_LEFT, GRAPH_BOTTOM),
        COLUMN_WIDTH,
        height,
        ink_style(Palette::blue()),
    )
    .expect("rect clamps its sizes")
}
/// The remaining gap on the same column and the same scale, resting on top of the capacitor's voltage.
fn gap_column(t: f64) -> VMobject {
    let now = state(t);
    let bottom = to_graph(0.0, now.voltage_v)[1];
    let height = GRAPH_HEIGHT * now.gap_v / VOLT_MAX;
    let style = Style::default()
        .fill(Palette::hero(), 0.75)
        .stroke(Palette::hero(), 0.0, 0.0);
    rect(v(COLUMN_LEFT, bottom), COLUMN_WIDTH, height, style).expect("rect clamps its sizes")
}
/// Separated charge inside the plates: one pair of marks per eighth of the battery voltage.
fn charges(t: f64) -> VMobject {
    let count = (CHARGES as f64 * state(t).voltage_v / battery()).round() as usize;
    let mut marks = Vec::new();
    for i in 0..count {
        let x = CHARGE_LEFT + CHARGE_STEP * i as f64;
        marks.push(spot(v(x, PLATE_TOP - 0.09), 0.04, Palette::gold()));
        marks.push(spot(v(x, PLATE_BOTTOM + 0.09), 0.04, Palette::teal()));
    }
    v_group(marks)
}
/// The arrow on the bottom wire. Current leaves the battery's long plate, so on this wire it runs right to left.
fn current_arrow(t: f64, factor: f64) -> VMobject {
    let length = ARROW_FULL * milliamps(with_resistance(factor), t) / milliamps(m::DEMO, 0.0);
    if length < 0.08 {
        return VMobject::new();
    }
    arrow(
        v(ARROW_X + length / 2.0, ARROW_Y),
        v(ARROW_X - length / 2.0, ARROW_Y),
        Palette::green(),
    )
}
fn resistor_shape(color: Srgb) -> VMobject {
    let span = RESISTOR_RIGHT - RESISTOR_LEFT;
    let mut points = vec![v(RESISTOR_LEFT, LOOP_TOP)];
    for i in 0..RESISTOR_TEETH {
        let x = RESISTOR_LEFT + span * (i as f64 + 0.5) / RESISTOR_TEETH as f64;
        let up = if i.is_multiple_of(2) { 0.18 } else { -0.18 };
        points.push(v(x, LOOP_TOP + up));
    }
    points.push(v(RESISTOR_RIGHT, LOOP_TOP));
    polyline(points).style(stroke_style(color, 3.0)).build()
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
fn retitle(s: &mut Stage<'_>, k: &Kit, title: Mob, text: &str) -> fmn::Result<()> {
    let new = k.text(s, text, 34.0, Palette::ink(), v(0.0, TITLE_Y))?;
    morph(s, &[(title, new)], 0.5)
}
/// A formula and its plain-words tag, on one row under the circuit.
fn row(s: &mut Stage<'_>, k: &Kit, tex: &str, tag: &str, y: f64, color: Srgb) -> fmn::Result<[Mob; 2]> {
    let formula = k.tex(s, tex, 30.0, Palette::ink(), &[], v(FORMULA_X, y))?;
    let words = k.text(s, tag, 22.0, color, v(TAG_X, y))?;
    Ok([formula, words])
}
/// Recolour a stroked object without touching its fill.
fn tint(s: &mut Stage<'_>, one: Mob, color: Srgb, seconds: f64) -> fmn::Result<Box<dyn Animation>> {
    let a = one
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(seconds),
            ..AnimateArgs::default()
        })?
        .set_stroke(Some(color), None, None)?;
    s.prepare(a)
}
fn dashed(a: Vec3, b: Vec3, color: Srgb, width: f64, dash: f64) -> fmn::Result<VMobject> {
    Ok(DashedLine::new(a, b)
        .dash_length(dash)
        .style(stroke_style(color, width))
        .build()?)
}

/// What the chapter keeps on screen and the clocks that drive it.
struct Board {
    title: Mob,
    clock: Mob,
    /// The multiple of the resistance that the arrow and the current readout use.
    factor: Mob,
    resistor: Mob,
}

/// The objects of 09_2 that later cues still move or remove.
struct GapView {
    segment: Mob,
    voltage_dot: Mob,
    x_name: Mob,
    formulas: Vec<Mob>,
}

/// The objects of 09_3 that later cues still move or remove.
struct Graph {
    segment: Mob,
    voltage_dot: Mob,
    gap_dot: Mob,
    x_name: Mob,
    formulas: Vec<Mob>,
}

/// The labels of the chosen values, built in 09_4.
struct Values {
    resistance: Mob,
    capacitance: Mob,
}

/// The current panel, built in 09_5.
struct Readouts {
    panel: Vec<Mob>,
    current_dot: Mob,
    /// Hidden while the resistance label changes ahead of the prediction: until 09_7 it would be the old circuit's.
    current_value: Mob,
    current_unit: Mob,
}

/// The current arrow reads both the clock and the resistance multiple. A still twin fades in first.
fn enter_current(s: &mut Stage<'_>, clock: Mob, factor: Mob, start: f64) -> fmn::Result<()> {
    let twin = mob(s, current_arrow(start, 1.0));
    show(s, &[twin], 0.3)?;
    let shown = s.always_redraw(move |a| {
        let t = a.tracker_value(clock).unwrap_or(start);
        let multiple = a.tracker_value(factor).unwrap_or(1.0);
        a.add(current_arrow(t, multiple))
    });
    s.add_to_scene(shown)?;
    s.remove_from_scene(twin);
    Ok(())
}

/// The current readout, for whichever resistance multiple the board's factor holds.
fn current_readout(s: &mut Stage<'_>, k: &Kit, b: &Board) -> fmn::Result<Mob> {
    let (clock, factor) = (b.clock, b.factor);
    k.live_value(
        s,
        move |a| {
            let t = a.tracker_value(clock).unwrap_or(0.0);
            milliamps(with_resistance(a.tracker_value(factor).unwrap_or(1.0)), t)
        },
        2,
        false,
        28.0,
        Palette::green(),
        v(READ_COLUMNS[2] + 0.35, READ_VALUE_Y),
        RIGHT,
    )
}
fn current_unit(s: &mut Stage<'_>, k: &Kit) -> fmn::Result<Mob> {
    k.text_left(s, "mA", 24.0, Palette::green(), v(READ_COLUMNS[2] + 0.45, READ_VALUE_Y))
}

/// 09_1: real circuit symbols first, with no equation. One clock charges the plates and shrinks the arrow.
fn the_circuit(s: &mut Stage<'_>, k: &Kit) -> fmn::Result<Board> {
    k.say(s, "09_1")?;
    let title = k.text(
        s,
        "A battery that charges a capacitor",
        34.0,
        Palette::ink(),
        v(0.0, TITLE_Y),
    )?;
    show(s, &[title], 0.4)?;
    let long_plate = stroke(
        v(LOOP_LEFT - 0.4, BATTERY_LONG),
        v(LOOP_LEFT + 0.4, BATTERY_LONG),
        Palette::ink(),
        4.0,
    );
    let short_plate = stroke(
        v(LOOP_LEFT - 0.2, BATTERY_SHORT),
        v(LOOP_LEFT + 0.2, BATTERY_SHORT),
        Palette::ink(),
        4.0,
    );
    let long_plate = mob(s, long_plate);
    let short_plate = mob(s, short_plate);
    let battery_name = k.text(s, "battery", 24.0, Palette::ink(), v(LOOP_LEFT + 1.05, 1.3))?;
    let plus = k.tex(
        s,
        "+",
        26.0,
        Palette::ink(),
        &[],
        v(LOOP_LEFT + 0.55, BATTERY_LONG + 0.2),
    )?;
    show(s, &[long_plate, short_plate, battery_name, plus], 0.4)?;
    let resistor = resistor_shape(Palette::ink());
    let resistor = mob(s, resistor);
    let resistor_name = k.text(s, "resistor", 24.0, Palette::ink(), v(ARROW_X, LOOP_TOP + 0.42))?;
    show(s, &[resistor, resistor_name], 0.4)?;
    let upper = stroke(
        v(LOOP_RIGHT - 0.45, PLATE_TOP),
        v(LOOP_RIGHT + 0.45, PLATE_TOP),
        Palette::ink(),
        4.0,
    );
    let lower = stroke(
        v(LOOP_RIGHT - 0.45, PLATE_BOTTOM),
        v(LOOP_RIGHT + 0.45, PLATE_BOTTOM),
        Palette::ink(),
        4.0,
    );
    let upper = mob(s, upper);
    let lower = mob(s, lower);
    let capacitor_name = k.text(s, "capacitor", 24.0, Palette::ink(), v(LOOP_RIGHT - 1.05, 1.3))?;
    show(s, &[upper, lower, capacitor_name], 0.4)?;
    let wire_ends = [
        (v(LOOP_LEFT, LOOP_BOTTOM), v(LOOP_LEFT, BATTERY_SHORT)),
        (v(LOOP_LEFT, BATTERY_LONG), v(LOOP_LEFT, LOOP_TOP)),
        (v(LOOP_LEFT, LOOP_TOP), v(RESISTOR_LEFT, LOOP_TOP)),
        (v(RESISTOR_RIGHT, LOOP_TOP), v(LOOP_RIGHT, LOOP_TOP)),
        (v(LOOP_RIGHT, LOOP_TOP), v(LOOP_RIGHT, PLATE_TOP)),
        (v(LOOP_RIGHT, PLATE_BOTTOM), v(LOOP_RIGHT, LOOP_BOTTOM)),
        (v(LOOP_RIGHT, LOOP_BOTTOM), v(LOOP_LEFT, LOOP_BOTTOM)),
    ];
    let mut wires = Vec::new();
    for (a, b) in wire_ends {
        let wire = stroke(a, b, Palette::dim(), 2.5);
        wires.push(mob(s, wire));
    }
    show(s, &wires, 0.4)?;
    let clock = s.add_value_tracker(0.0);
    let factor = s.add_value_tracker(1.0);
    live(s, clock, 0.0, charges)?;
    k.until_sentence(s, "09_1", 1)?;
    let separated = k.text(s, "separated charge", 22.0, Palette::gold(), v(-0.85, 1.3))?;
    show(s, &[separated], 0.3)?;
    run(s, clock, 1.2 * tau(), 2.2)?;
    k.until_sentence(s, "09_1", 2)?;
    enter_current(s, clock, factor, 1.2 * tau())?;
    let current_name = k.text(s, "current", 22.0, Palette::green(), v(ARROW_X, -0.46))?;
    show(s, &[current_name], 0.3)?;
    k.until_sentence(s, "09_1", 3)?;
    rewind(s, clock, 0.0, 0.6)?;
    run(s, clock, 4.0 * tau(), 3.6)?;
    k.until_sentence(s, "09_1", 4)?;
    let not_tired = k.text(
        s,
        "same battery voltage",
        22.0,
        Palette::dim(),
        v(LOOP_LEFT + 1.55, 0.72),
    )?;
    show(s, &[not_tired], 0.3)?;
    k.until_sentence(s, "09_1", 5)?;
    let shrinking = k.text(
        s,
        "The difference that drives it is shrinking",
        25.0,
        Palette::hero(),
        v(-4.05, -1.0),
    )?;
    show(s, &[shrinking], 0.4)?;
    k.hold(s)?;
    clear(s, &[separated, not_tired, shrinking])?;
    Ok(Board {
        title,
        clock,
        factor,
        resistor,
    })
}

/// 09_2: the battery level, the capacitor voltage, and the gap between them on one vertical scale.
fn the_gap(s: &mut Stage<'_>, k: &Kit, b: &Board) -> fmn::Result<GapView> {
    k.say(s, "09_2")?;
    retitle(s, k, b.title, "What drives the current is a gap")?;
    rewind(s, b.clock, 0.0, 0.7)?;
    let corner = to_graph(0.0, 0.0);
    let x_end = to_graph(TIME_MAX, 0.0);
    let x_axis = stroke(corner, x_end, Palette::dim(), 2.0);
    let y_axis = stroke(corner, to_graph(0.0, VOLT_MAX), Palette::dim(), 2.0);
    // Keep the shared voltage level, with a gap for the graph's twelve-volt tick label.
    let level_y = to_graph(0.0, battery())[1];
    let level_column = stroke(v(COLUMN_LEFT - 0.1, level_y), v(GRAPH_LEFT - 0.60, level_y), Palette::gold(), 3.0);
    let level = stroke(v(GRAPH_LEFT - 0.05, level_y), to_graph(TIME_MAX, battery()), Palette::gold(), 3.0);
    let level_column = mob(s, level_column);
    let (x_axis, y_axis, level) = (mob(s, x_axis), mob(s, y_axis), mob(s, level));
    let level_name = k.text(
        s,
        "battery voltage",
        24.0,
        Palette::gold(),
        add(to_graph(TIME_MAX, battery()), v(-1.1, 0.3)),
    )?;
    let x_name = k.text(s, "time", 22.0, Palette::dim(), v(5.75, GRAPH_BOTTOM - 0.72))?;
    show(s, &[x_axis, y_axis, level_column, level, level_name, x_name], 0.5)?;
    k.until_sentence(s, "09_2", 1)?;
    live(s, b.clock, 0.0, voltage_column)?;
    let voltage_dot = enter(s, b.clock, 0.0, voltage_spot)?;
    let voltage_name = k.tex(
        s,
        "V",
        30.0,
        Palette::blue(),
        &[],
        v(COLUMN_LEFT + COLUMN_WIDTH / 2.0, GRAPH_BOTTOM - 0.38),
    )?;
    show(s, &[voltage_name], 0.3)?;
    k.until_sentence(s, "09_2", 2)?;
    enter(s, b.clock, 0.0, gap_column)?;
    let segment = enter(s, b.clock, 0.0, gap_segment)?;
    let gap_name = k.text(s, "gap", 26.0, Palette::hero(), v(COLUMN_LEFT - 0.42, 1.35))?;
    show(s, &[gap_name], 0.3)?;
    s.add_to_scene(voltage_dot)?;
    k.until_sentence(s, "09_2", 3)?;
    let ohm = row(
        s,
        k,
        r"i=\frac{V_B-V}{R}",
        "gap over resistance",
        ROWS[0],
        Palette::hero(),
    )?;
    show(s, &ohm, 0.4)?;
    k.until_sentence(s, "09_2", 4)?;
    let capacitor = row(s, k, r"i=C\,V'", "capacitance times rate", ROWS[1], Palette::blue())?;
    show(s, &capacitor, 0.4)?;
    k.until_sentence(s, "09_2", 5)?;
    let joined = row(
        s,
        k,
        r"V'=\frac{V_B-V}{RC}",
        "proportional to the gap",
        ROWS[2],
        Palette::hero(),
    )?;
    show(s, &joined, 0.4)?;
    // The gap visibly shrinks while the equation that says so is on screen.
    run(s, b.clock, 0.8 * tau(), 2.4)?;
    let mut formulas = ohm.to_vec();
    formulas.extend(capacitor);
    formulas.extend(joined);
    Ok(GapView {
        segment,
        voltage_dot,
        x_name,
        formulas,
    })
}

/// 09_3: the gap, kept on screen, gets its own decay equation. Voltage and gap are traced from one state.
fn change_what_we_measure(s: &mut Stage<'_>, k: &Kit, b: &Board, g: GapView) -> fmn::Result<Graph> {
    k.say(s, "09_3")?;
    clear(s, &g.formulas)?;
    let rate = k.tex(
        s,
        r"V'=\frac{V_B-V}{RC}",
        30.0,
        Palette::ink(),
        &[],
        v(FORMULA_X, ROWS[0]),
    )?;
    let not_simple = k.text(
        s,
        "V' also depends on the battery",
        22.0,
        Palette::dim(),
        v(TAG_X, ROWS[0]),
    )?;
    show(s, &[rate, not_simple], 0.4)?;
    k.until_sentence(s, "09_3", 1)?;
    let names_gap = k.tex(s, r"g=V_B-V", 30.0, Palette::hero(), &[], v(FORMULA_X, ROWS[1]))?;
    let the_gap_tag = k.text(
        s,
        "the gap, measured directly",
        22.0,
        Palette::hero(),
        v(TAG_X, ROWS[1]),
    )?;
    show(s, &[names_gap, the_gap_tag], 0.4)?;
    k.until_sentence(s, "09_3", 2)?;
    let before = state(0.8 * tau()).voltage_v;
    let after = state(1.1 * tau()).voltage_v;
    let start_y = to_graph(0.0, before)[1];
    let end_y = to_graph(0.0, after)[1];
    let rise = stroke(
        v(COLUMN_LEFT - 0.14, start_y),
        v(COLUMN_LEFT - 0.14, end_y),
        Palette::blue(),
        5.0,
    );
    let fall = stroke(
        v(COLUMN_LEFT + COLUMN_WIDTH + 0.14, start_y),
        v(COLUMN_LEFT + COLUMN_WIDTH + 0.14, end_y),
        Palette::hero(),
        5.0,
    );
    let (rise, fall) = (mob(s, rise), mob(s, fall));
    show(s, &[rise, fall], 0.3)?;
    run(s, b.clock, 1.1 * tau(), 1.4)?;
    let same_amount = k.text(
        s,
        "up by the same amount the gap falls",
        22.0,
        Palette::dim(),
        v(TAG_X, ROWS[2]),
    )?;
    show(s, &[same_amount], 0.3)?;
    k.until_sentence(s, "09_3", 3)?;
    clear(s, &[not_simple, the_gap_tag, same_amount, rise, fall])?;
    let decay = k.tex(
        s,
        r"g'=-V'=-\frac{g}{RC}",
        30.0,
        Palette::hero(),
        &[],
        v(FORMULA_X + 0.7, ROWS[2]),
    )?;
    show(s, &[decay], 0.4)?;
    k.until_sentence(s, "09_3", 4)?;
    let same_law = k.text(
        s,
        "the decay law, for a new quantity",
        22.0,
        Palette::green(),
        v(TAG_X, ROWS[1]),
    )?;
    show(s, &[same_law], 0.4)?;
    k.until_sentence(s, "09_3", 5)?;
    clear(s, &[rate, names_gap, same_law])?;
    let solved = k.tex(
        s,
        r"g(t)=V_B\,e^{-t/RC}",
        30.0,
        Palette::hero(),
        &[],
        v(FORMULA_X + 0.3, ROWS[0]),
    )?;
    let voltage = k.tex(
        s,
        r"V(t)=V_B-g(t)",
        30.0,
        Palette::blue(),
        &[],
        v(FORMULA_X + 0.3, ROWS[1] + 0.05),
    )?;
    let sum_name = k.tex(s, r"V+g=", 28.0, Palette::gold(), &[], v(-0.78, READ_VALUE_Y))?;
    let sum_unit = k.text_left(s, "V", 24.0, Palette::gold(), v(0.63, READ_VALUE_Y))?;
    show(s, &[solved, voltage, sum_name, sum_unit], 0.4)?;
    // The fixed sum is read from the model state, so it stays fixed only if voltage and gap really are complementary.
    let clock = b.clock;
    k.live_value(
        s,
        move |a| {
            let now = state(a.tracker_value(clock).unwrap_or(0.0));
            now.voltage_v + now.gap_v
        },
        2,
        false,
        28.0,
        Palette::gold(),
        v(0.55, READ_VALUE_Y),
        RIGHT,
    )?;
    rewind(s, b.clock, 0.0, 0.5)?;
    // Both traces and both dots read the same clock through the same model state.
    let voltage_drawing = live(s, b.clock, 0.0, voltage_curve)?;
    let gap_drawing = live(s, b.clock, 0.0, gap_curve)?;
    let gap_dot = enter(s, b.clock, 0.0, gap_spot)?;
    s.add_to_scene(g.voltage_dot)?;
    run(s, b.clock, TIME_MAX, 3.6)?;
    let voltage_trace = mob(s, voltage_curve(TIME_MAX));
    settle(s, voltage_drawing, voltage_trace)?;
    let gap_trace = mob(s, gap_curve(TIME_MAX));
    settle(s, gap_drawing, gap_trace)?;
    s.add_to_scene(g.voltage_dot)?;
    s.add_to_scene(gap_dot)?;
    Ok(Graph {
        segment: g.segment,
        voltage_dot: g.voltage_dot,
        gap_dot,
        x_name: g.x_name,
        formulas: vec![decay, solved, voltage],
    })
}

/// 09_4: chosen values and units replace the bare symbols, and the product sets the clock.
fn choose_values(s: &mut Stage<'_>, k: &Kit, b: &Board, g: &Graph) -> fmn::Result<(Values, Vec<Mob>)> {
    k.say(s, "09_4")?;
    clear(s, &g.formulas)?;
    retitle(s, k, b.title, "Choose values: one time constant")?;
    let battery_value = k.text(s, &battery_label(), 26.0, Palette::gold(), v(LOOP_LEFT + 1.05, 0.72))?;
    let resistance = k.text(s, &ohms_label(m::DEMO), 24.0, Palette::ink(), v(ARROW_X, 1.95))?;
    let capacitance = k.text(
        s,
        &farads_label(m::DEMO),
        22.0,
        Palette::ink(),
        v(LOOP_RIGHT - 1.25, 0.72),
    )?;
    show(s, &[battery_value, resistance, capacitance], 0.4)?;
    let mut ticks = Vec::new();
    for second in 1..=(TIME_MAX as u32) {
        let x = to_graph(f64::from(second), 0.0)[0];
        let tick = stroke(v(x, GRAPH_BOTTOM), v(x, GRAPH_BOTTOM - 0.1), Palette::dim(), 2.0);
        ticks.push(mob(s, tick));
        ticks.push(k.text(s, &second.to_string(), 20.0, Palette::dim(), v(x, GRAPH_BOTTOM - 0.33))?);
    }
    for volts in [0_u32, 4, 8, 12] {
        let y = to_graph(0.0, f64::from(volts))[1];
        ticks.push(k.text(s, &volts.to_string(), 20.0, Palette::dim(), v(GRAPH_LEFT - 0.32, y))?);
    }
    let volts = k.text(
        s,
        "volts",
        20.0,
        Palette::dim(),
        v(GRAPH_LEFT + 0.42, to_graph(0.0, VOLT_MAX)[1]),
    )?;
    ticks.push(volts);
    show(s, &ticks, 0.5)?;
    let seconds = k.text(s, "time (seconds)", 22.0, Palette::dim(), v(5.75, GRAPH_BOTTOM - 0.72))?;
    morph(s, &[(g.x_name, seconds)], 0.4)?;
    k.until_sentence(s, "09_4", 1)?;
    let chosen = k.text(
        s,
        "chosen model values, not a measured device",
        23.0,
        Palette::dim(),
        v(NOTE_X, ROWS[0]),
    )?;
    show(s, &[chosen], 0.4)?;
    k.until_sentence(s, "09_4", 2)?;
    let product = k.tex(s, &rc_tex(), 26.0, Palette::ink(), &[], v(NOTE_X, ROWS[1]))?;
    show(s, &[product], 0.4)?;
    k.until_sentence(s, "09_4", 3)?;
    rewind(s, b.clock, tau(), 1.0)?;
    let x = to_graph(tau(), 0.0)[0];
    let mark = dashed(
        v(x, GRAPH_BOTTOM),
        v(x, to_graph(0.0, battery())[1]),
        Palette::dim(),
        2.5,
        0.14,
    )?;
    let mark = mob(s, mark);
    let one_over_e = k.tex(
        s,
        r"g(RC)=\frac{1}{e}\,g(0)",
        30.0,
        Palette::hero(),
        &[],
        v(NOTE_X, ROWS[2]),
    )?;
    show(s, &[mark, one_over_e], 0.4)?;
    let values = Values {
        resistance,
        capacitance,
    };
    Ok((values, vec![chosen, product, one_over_e, mark]))
}

/// 09_5: the numbers at one time constant, then one moving clock behind three readouts and three curves.
fn three_readouts(s: &mut Stage<'_>, k: &Kit, b: &Board, cards: &[Mob]) -> fmn::Result<Readouts> {
    k.say(s, "09_5")?;
    let remains = k.text(
        s,
        &format!("{} of the gap remains", gap_left_label()),
        24.0,
        Palette::hero(),
        v(FREE_X, 0.85),
    )?;
    let filled = k.text(
        s,
        &format!("{} filled", filled_label()),
        24.0,
        Palette::blue(),
        v(FREE_X, 0.3),
    )?;
    show(s, &[remains, filled], 0.4)?;
    let mut names = Vec::new();
    for (x, name, color) in [
        (READ_COLUMNS[0], "voltage", Palette::blue()),
        (READ_COLUMNS[1], "gap", Palette::hero()),
        (READ_COLUMNS[2], "current", Palette::green()),
    ] {
        names.push(k.text(s, name, 22.0, color, v(x, READ_NAME_Y))?);
    }
    show(s, &names, 0.3)?;
    let clock = b.clock;
    k.live_value(
        s,
        move |a| state(a.tracker_value(clock).unwrap_or(0.0)).voltage_v,
        2,
        false,
        28.0,
        Palette::blue(),
        v(READ_COLUMNS[0] + 0.35, READ_VALUE_Y),
        RIGHT,
    )?;
    let volt_unit = k.text_left(s, "V", 24.0, Palette::blue(), v(READ_COLUMNS[0] + 0.45, READ_VALUE_Y))?;
    k.live_value(
        s,
        move |a| state(a.tracker_value(clock).unwrap_or(0.0)).gap_v,
        2,
        false,
        28.0,
        Palette::hero(),
        v(READ_COLUMNS[1] + 0.35, READ_VALUE_Y),
        RIGHT,
    )?;
    let gap_unit = k.text_left(s, "V", 24.0, Palette::hero(), v(READ_COLUMNS[1] + 0.45, READ_VALUE_Y))?;
    let current_value = current_readout(s, k, b)?;
    let amp_unit = current_unit(s, k)?;
    show(s, &[volt_unit, gap_unit, amp_unit], 0.3)?;
    k.until_sentence(s, "09_5", 1)?;
    let read_volts = k.text(
        s,
        &format!("about {}", one_tau_volts_label()),
        28.0,
        Palette::blue(),
        v(FREE_X, -0.3),
    )?;
    show(s, &[read_volts], 0.3)?;
    k.until_sentence(s, "09_5", 2)?;
    let mut going = cards.to_vec();
    going.extend([remains, filled, read_volts]);
    clear(s, &going)?;
    let corner = to_mini(0.0, 0.0);
    let start_current = milliamps(m::DEMO, 0.0);
    let x_axis = stroke(corner, to_mini(TIME_MAX, 0.0), Palette::dim(), 2.0);
    let y_axis = stroke(corner, to_mini(0.0, start_current), Palette::dim(), 2.0);
    let (x_axis, y_axis) = (mob(s, x_axis), mob(s, y_axis));
    let panel_name = k.text(
        s,
        "current",
        20.0,
        Palette::green(),
        add(to_mini(0.0, start_current), v(0.5, 0.18)),
    )?;
    show(s, &[x_axis, y_axis, panel_name], 0.3)?;
    rewind(s, clock, 0.0, 0.6)?;
    let drawing = live(s, clock, 0.0, current_curve)?;
    let current_dot = enter(s, clock, 0.0, current_spot)?;
    run(s, clock, 4.0 * tau(), 4.0)?;
    k.until_sentence(s, "09_5", 3)?;
    run(s, clock, TIME_MAX, 1.2)?;
    let current_still = mob(s, current_curve(TIME_MAX));
    settle(s, drawing, current_still)?;
    s.add_to_scene(current_dot)?;
    k.until_sentence(s, "09_5", 4)?;
    let one_state = k.text(
        s,
        "one moving state, three curves",
        26.0,
        Palette::ink(),
        v(FREE_X, 0.2),
    )?;
    show(s, &[one_state], 0.4)?;
    k.hold(s)?;
    clear(s, &[one_state])?;
    Ok(Readouts {
        panel: vec![x_axis, y_axis, panel_name, current_still],
        current_dot,
        current_value,
        current_unit: amp_unit,
    })
}

/// 09_6: double the resistance and ask for two predictions. Nothing about the slower circuit is drawn yet.
fn predict_doubling(
    s: &mut Stage<'_>,
    k: &Kit,
    b: &Board,
    g: &Graph,
    values: &Values,
    read: &Readouts,
) -> fmn::Result<Vec<Mob>> {
    k.say(s, "09_6")?;
    // The number goes before the resistance label changes, so 1.20 mA is never shown beside 20,000 ohms.
    clear(
        s,
        &[
            g.voltage_dot,
            g.gap_dot,
            g.segment,
            read.current_dot,
            read.current_value,
            read.current_unit,
        ],
    )?;
    rewind(s, b.clock, 0.0, 0.7)?;
    retitle(s, k, b.title, "Double the resistance: predict")?;
    let doubled = k.text(
        s,
        &ohms_label(with_resistance(2.0)),
        24.0,
        Palette::teal(),
        v(ARROW_X, 1.95),
    )?;
    let recolor = tint(s, b.resistor, Palette::teal(), 0.5)?;
    let relabel = s.prepare(Transform::new(values.resistance, doubled).rt(0.5))?;
    s.play_prepared(vec![recolor, relabel])?;
    k.until_sentence(s, "09_6", 2)?;
    let current_question = k.text(
        s,
        "Starting current: up or down?",
        24.0,
        Palette::teal(),
        v(FREE_X, 0.85),
    )?;
    show(s, &[current_question], 0.3)?;
    k.until_sentence(s, "09_6", 3)?;
    let clock_question = k.text(
        s,
        "Same fraction: more time or less?",
        24.0,
        Palette::teal(),
        v(FREE_X, 0.3),
    )?;
    show(s, &[clock_question], 0.3)?;
    k.hold(s)?;
    k.mark(s, "predict");
    k.music_breath(s, 4.0)?;
    Ok(vec![current_question, clock_question])
}

/// 09_7: the answer, the rescaled clock, and a separate change of capacitance.
fn change_the_clock(
    s: &mut Stage<'_>,
    k: &Kit,
    b: &Board,
    values: &Values,
    read: &Readouts,
    questions: &[Mob],
) -> fmn::Result<Vec<Mob>> {
    k.say(s, "09_7")?;
    clear(s, questions)?;
    rewind(s, b.factor, 2.0, 1.0)?;
    // The readout returns with the answer: the doubled resistance at time zero.
    current_readout(s, k, b)?;
    let returned_unit = current_unit(s, k)?;
    show(s, &[returned_unit], 0.3)?;
    let slow = s.add_value_tracker(0.0);
    let voltage_drawing = live(s, slow, 0.0, slower_voltage_curve)?;
    let current_drawing = live(s, slow, 0.0, slower_current_curve)?;
    run(s, slow, TIME_MAX, 2.6)?;
    let slower_still = mob(s, slower_voltage_stroke(1.0));
    settle(s, voltage_drawing, slower_still)?;
    let slower_current = mob(s, slower_current_curve(TIME_MAX));
    settle(s, current_drawing, slower_current)?;
    let fast_x = to_graph(tau(), 0.0)[0];
    let slow_x = to_graph(with_resistance(2.0).state(0.0).time_constant_s, 0.0)[0];
    let level_y = to_graph(0.0, state(tau()).voltage_v)[1];
    let same_fraction = dashed(v(GRAPH_LEFT, level_y), v(slow_x, level_y), Palette::dim(), 2.0, 0.12)?;
    let to_fast = dashed(v(fast_x, GRAPH_BOTTOM), v(fast_x, level_y), Palette::blue(), 2.5, 0.12)?;
    let to_slow = dashed(v(slow_x, GRAPH_BOTTOM), v(slow_x, level_y), Palette::teal(), 2.5, 0.12)?;
    let (same_fraction, to_fast, to_slow) = (mob(s, same_fraction), mob(s, to_fast), mob(s, to_slow));
    show(s, &[same_fraction, to_fast, to_slow], 0.4)?;
    k.until_sentence(s, "09_7", 1)?;
    let in_current = k.tex(
        s,
        r"i(0)=\frac{V_B}{R}",
        30.0,
        Palette::ink(),
        &[],
        v(FREE_X - 0.9, 0.2),
    )?;
    let in_clock = k.tex(s, r"\tau=RC", 30.0, Palette::ink(), &[], v(FREE_X + 1.1, 0.2))?;
    show(s, &[in_current, in_clock], 0.4)?;
    k.until_sentence(s, "09_7", 2)?;
    let squeezed = slower_voltage_stroke(2.0);
    let squeezed = mob(s, squeezed);
    let factor_name = k.text(s, "divide its times by 2", 24.0, Palette::teal(), v(FREE_X, -0.45))?;
    show(s, &[factor_name], 0.3)?;
    morph(s, &[(slower_still, squeezed)], 1.6)?;
    k.until_sentence(s, "09_7", 3)?;
    let not_shape = k.text(
        s,
        "The clock changed, not the shape",
        25.0,
        Palette::teal(),
        v(FREE_X, -1.05),
    )?;
    show(s, &[not_shape], 0.4)?;
    k.until_sentence(s, "09_7", 4)?;
    let mut going = read.panel.clone();
    going.extend([
        slower_current,
        in_current,
        in_clock,
        factor_name,
        not_shape,
        same_fraction,
        to_fast,
        to_slow,
    ]);
    clear(s, &going)?;
    // One change at a time: the resistance returns before the capacitance changes.
    let original = k.text(s, &ohms_label(m::DEMO), 24.0, Palette::ink(), v(ARROW_X, 1.95))?;
    let larger = k.text(
        s,
        &farads_label(with_capacitance(2.0)),
        22.0,
        Palette::teal(),
        v(LOOP_RIGHT - 1.25, 0.72),
    )?;
    let back = tween(b.factor, 1.0, 0.8)?;
    let moves = vec![
        s.prepare(back)?,
        tint(s, b.resistor, Palette::ink(), 0.8)?,
        s.prepare(Transform::new(values.resistance, original).rt(0.8))?,
        s.prepare(Transform::new(values.capacitance, larger).rt(0.8))?,
    ];
    s.play_prepared(moves)?;
    let table_rows = [
        ("start", time_label(m::DEMO), current_label(m::DEMO), Palette::ink()),
        (
            "R doubled",
            time_label(with_resistance(2.0)),
            current_label(with_resistance(2.0)),
            Palette::teal(),
        ),
        (
            "C doubled",
            time_label(with_capacitance(2.0)),
            current_label(with_capacitance(2.0)),
            Palette::teal(),
        ),
    ];
    let mut table = vec![
        k.tex(s, "RC", 24.0, Palette::dim(), &[], v(-3.75, -1.2))?,
        k.text(s, "starting current", 22.0, Palette::dim(), v(-1.9, -1.2))?,
    ];
    for (i, (name, clock_text, current_text, color)) in table_rows.iter().enumerate() {
        let y = -1.75 - 0.55 * i as f64;
        table.push(k.text(s, name, 24.0, *color, v(-5.4, y))?);
        table.push(k.text(s, clock_text, 24.0, *color, v(-3.75, y))?);
        table.push(k.text(s, current_text, 24.0, *color, v(-1.9, y))?);
    }
    show(s, &table, 0.4)?;
    table.push(slower_still);
    Ok(table)
}

/// 09_8: the model's assumptions, then the last of the gap, magnified, against a chosen instrument resolution.
fn practical_settling(s: &mut Stage<'_>, k: &Kit, b: &Board, values: &Values, table: &[Mob]) -> fmn::Result<()> {
    k.say(s, "09_8")?;
    clear(s, table)?;
    retitle(s, k, b.title, "An ideal model, and a real instrument")?;
    // Back to the demo capacitor: every settling number below is for this value, not the 09_7 doubling.
    let restored = k.text(
        s,
        &farads_label(m::DEMO),
        22.0,
        Palette::ink(),
        v(LOOP_RIGHT - 1.25, 0.72),
    )?;
    morph(s, &[(values.capacitance, restored)], 0.5)?;
    let mut assumptions = Vec::new();
    for (i, text) in [
        "constant battery voltage",
        "linear resistor",
        "fixed capacitance",
        "no leakage or other parts",
    ]
    .iter()
    .enumerate()
    {
        assumptions.push(k.text(s, text, 22.0, Palette::dim(), v(FREE_X, 1.0 - 0.46 * i as f64))?);
    }
    show(s, &assumptions, 0.4)?;
    k.until_sentence(s, "09_8", 1)?;
    clear(s, &assumptions)?;
    // The box on the main graph marks exactly the window the inset magnifies, and the leads tie the two together.
    let box_low = to_graph(ZOOM_START_S, ZOOM_LOW_V);
    let box_high = to_graph(ZOOM_END_S, ZOOM_HIGH_V);
    let corner = to_zoom(ZOOM_START_S, ZOOM_LOW_V);
    let far = to_zoom(ZOOM_END_S, ZOOM_HIGH_V);
    let locator = rect(
        box_low,
        box_high[0] - box_low[0],
        box_high[1] - box_low[1],
        stroke_style(Palette::hero(), 2.5),
    )?;
    let inset = rect(corner, ZOOM_WIDTH, ZOOM_HEIGHT, stroke_style(Palette::dim(), 2.0))?;
    let left_lead = stroke(box_low, v(corner[0], far[1]), Palette::dim(), 1.5);
    let right_lead = stroke(v(box_high[0], box_low[1]), far, Palette::dim(), 1.5);
    let ceiling_y = to_zoom(ZOOM_START_S, battery())[1];
    let ceiling = stroke(
        v(ZOOM_LEFT, ceiling_y),
        v(ZOOM_LEFT + ZOOM_WIDTH, ceiling_y),
        Palette::gold(),
        3.0,
    );
    let (locator, inset, left_lead, right_lead, ceiling) = (
        mob(s, locator),
        mob(s, inset),
        mob(s, left_lead),
        mob(s, right_lead),
        mob(s, ceiling),
    );
    let ceiling_name = k.text(
        s,
        &battery_label(),
        20.0,
        Palette::gold(),
        v(ZOOM_LEFT - 0.34, ceiling_y),
    )?;
    let low_name = k.text_left(
        s,
        &window_low_label(),
        18.0,
        Palette::dim(),
        v(ZOOM_LEFT + 0.08, ZOOM_BOTTOM + 0.16),
    )?;
    let start_name = k.text(
        s,
        &seconds_label(ZOOM_START_S),
        20.0,
        Palette::dim(),
        v(ZOOM_LEFT, ZOOM_BOTTOM - 0.2),
    )?;
    let end_name = k.text(
        s,
        &seconds_label(ZOOM_END_S),
        20.0,
        Palette::dim(),
        v(ZOOM_LEFT + ZOOM_WIDTH - 0.05, ZOOM_BOTTOM - 0.2),
    )?;
    show(
        s,
        &[
            locator,
            inset,
            left_lead,
            right_lead,
            ceiling,
            ceiling_name,
            low_name,
            start_name,
            end_name,
        ],
        0.4,
    )?;
    // The chapter's one clock: the column, the arrow and the readouts move with the inset.
    rewind(s, b.clock, ZOOM_START_S, 0.6)?;
    let drawing = live(s, b.clock, ZOOM_START_S, zoom_curve)?;
    let dot = enter(s, b.clock, ZOOM_START_S, zoom_spot)?;
    run(s, b.clock, ZOOM_END_S, 3.2)?;
    let curve = mob(s, zoom_curve(ZOOM_END_S));
    settle(s, drawing, curve)?;
    s.add_to_scene(dot)?;
    k.until_sentence(s, "09_8", 2)?;
    let band_bottom = to_zoom(ZOOM_START_S, battery() - RESOLUTION_V)[1];
    let band_style = Style::default()
        .fill(Palette::green(), 0.3)
        .stroke(Palette::green(), 0.0, 0.0);
    let band = rect(
        v(ZOOM_LEFT, band_bottom),
        ZOOM_WIDTH,
        ceiling_y - band_bottom,
        band_style,
    )?;
    let band = mob(s, band);
    let band_name = k.text(
        s,
        &format!("chosen instrument resolution: {}", resolution_label()),
        24.0,
        Palette::green(),
        v(NOTE_X, -1.05),
    )?;
    show(s, &[band, band_name], 0.4)?;
    s.add_to_scene(ceiling)?;
    s.add_to_scene(curve)?;
    s.add_to_scene(dot)?;
    k.until_sentence(s, "09_8", 3)?;
    let settle_x = to_zoom(settle_time(), ZOOM_LOW_V)[0];
    let settle_mark = dashed(
        v(settle_x, ZOOM_BOTTOM),
        v(settle_x, ceiling_y),
        Palette::green(),
        2.5,
        0.1,
    )?;
    let settle_mark = mob(s, settle_mark);
    let settle_name = k.text(
        s,
        &format!("this instrument settles at {}", settle_label()),
        24.0,
        Palette::green(),
        v(NOTE_X, -1.65),
    )?;
    let not_zero = k.text(
        s,
        &format!(
            "the model's gap at {} is still {}",
            seconds_label(ZOOM_END_S),
            final_gap_label()
        ),
        22.0,
        Palette::dim(),
        v(NOTE_X, -2.25),
    )?;
    show(s, &[settle_mark, settle_name, not_zero], 0.4)?;
    k.until_sentence(s, "09_8", 4)?;
    let question = k.text(
        s,
        "How small must the gap be for this job?",
        28.0,
        Palette::hero(),
        v(NOTE_X, -3.0),
    )?;
    show(s, &[question], 0.4)?;
    Ok(())
}

impl SceneConstruct for ChargingGap {
    fn name(&self) -> &str {
        "09_charging_gap"
    }
    fn construct(&mut self, s: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let board = the_circuit(s, &k)?;
        let gap_view = the_gap(s, &k, &board)?;
        let graph = change_what_we_measure(s, &k, &board, gap_view)?;
        let (values, cards) = choose_values(s, &k, &board, &graph)?;
        let read = three_readouts(s, &k, &board, &cards)?;
        let questions = predict_doubling(s, &k, &board, &graph, &values, &read)?;
        let table = change_the_clock(s, &k, &board, &values, &read, &questions)?;
        practical_settling(s, &k, &board, &values, &table)?;
        k.finish(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The scene's code, without this test module.
    fn scene() -> &'static str {
        include_str!("circuit_scene.rs")
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
        let cues: Vec<String> = (1..=8).map(|n| format!(r#"k.say(s, "09_{n}")?;"#)).collect();
        for cue in &cues {
            assert_eq!(scene().matches(cue.as_str()).count(), 1, "{cue}");
        }
        assert_in_order(&cues.iter().map(String::as_str).collect::<Vec<_>>());
    }
    #[test]
    fn symbols_come_before_equations() {
        assert_in_order(&[
            r#""battery""#,
            r#""resistor""#,
            r#""capacitor""#,
            r#"k.say(s, "09_2")?;"#,
            r"i=\frac{V_B-V}{R}",
        ]);
    }
    #[test]
    fn the_slower_trace_stays_hidden_through_the_authored_hold() {
        assert_in_order(&[
            r#"k.say(s, "09_6")?;"#,
            "k.hold(s)?;",
            r#"k.mark(s, "predict");"#,
            "k.music_breath(s, 4.0)?;",
            r#"k.say(s, "09_7")?;"#,
            "live(s, slow, 0.0, slower_voltage_curve)?",
        ]);
        let hold = scene().find("k.music_breath").expect("the hold exists");
        assert!(
            !scene()[..hold].contains("slower_voltage_curve)?"),
            "no slower trace before the hold"
        );
    }
    #[test]
    fn the_current_readout_is_gone_while_the_changed_resistance_is_on_screen() {
        assert_in_order(&[
            r#"k.say(s, "09_6")?;"#,
            "read.current_value,",
            "read.current_unit,",
            "Transform::new(values.resistance, doubled)",
            r#"k.mark(s, "predict");"#,
            r#"k.say(s, "09_7")?;"#,
            "rewind(s, b.factor, 2.0, 1.0)?;",
            "current_readout(s, k, b)?;",
            "show(s, &[returned_unit], 0.3)?;",
        ]);
        assert_eq!(scene().matches("current_readout(s, k, b)?").count(), 2);
    }
    #[test]
    fn the_capacitor_label_returns_before_the_settling_numbers() {
        assert_in_order(&[
            "fn practical_settling(",
            r#"k.say(s, "09_8")?;"#,
            "&farads_label(m::DEMO),",
            "morph(s, &[(values.capacitance, restored)], 0.5)?;",
            r#"k.until_sentence(s, "09_8", 1)?;"#,
            "settle_label()",
            "final_gap_label()",
        ]);
        // The printed numbers belong to the demo capacitor; at 200 microfarads they would be these.
        let larger = with_capacitance(2.0);
        let larger_settle = larger.state(0.0).time_constant_s * (battery() / RESOLUTION_V).ln();
        assert_eq!(format!("{larger_settle:.2} s"), "10.96 s");
        assert_eq!(format!("{:.2} V", larger.state(ZOOM_END_S).gap_v), "0.60 V");
    }
    #[test]
    fn the_settling_inset_runs_on_the_chapters_one_clock() {
        assert!(!scene().contains("zoom_clock"));
        assert_in_order(&[
            r#"k.say(s, "09_8")?;"#,
            "clear(s, &assumptions)?;",
            "rewind(s, b.clock, ZOOM_START_S, 0.6)?;",
            "live(s, b.clock, ZOOM_START_S, zoom_curve)?",
            "enter(s, b.clock, ZOOM_START_S, zoom_spot)?",
            "run(s, b.clock, ZOOM_END_S, 3.2)?;",
            r#"k.until_sentence(s, "09_8", 2)?;"#,
        ]);
        let end = state(ZOOM_END_S);
        assert_eq!(format!("{:.2}", end.voltage_v), "11.97");
        assert_eq!(format!("{:.2}", end.gap_v), "0.03");
    }
    #[test]
    fn the_decay_tag_leaves_before_the_voltage_formula_takes_its_row() {
        assert_in_order(&[
            r#"k.until_sentence(s, "09_3", 5)?;"#,
            "clear(s, &[rate, names_gap, same_law])?;",
            r"V(t)=V_B-g(t)",
            "formulas: vec![decay, solved, voltage],",
        ]);
    }
    #[test]
    fn the_scene_uses_the_scripted_rounded_values() {
        assert_eq!(gap_left_label(), "36.8%");
        assert_eq!(filled_label(), "63.2%");
        assert_eq!(one_tau_volts_label(), "7.59 V");
        assert_eq!(ohms_label(m::DEMO), "10,000 ohms");
        assert_eq!(farads_label(m::DEMO), "100 microfarads");
        assert_eq!(battery_label(), "12 V");
        assert_eq!(time_label(m::DEMO), "1 s");
    }
    #[test]
    fn doubling_r_and_doubling_c_are_kept_apart() {
        assert_eq!(current_label(m::DEMO), "1.20 mA");
        assert_eq!(current_label(with_resistance(2.0)), "0.60 mA");
        assert_eq!(current_label(with_capacitance(2.0)), "1.20 mA");
        assert_eq!(time_label(with_resistance(2.0)), "2 s");
        assert_eq!(time_label(with_capacitance(2.0)), "2 s");
    }
    #[test]
    fn dividing_the_slower_times_by_two_lands_on_the_demo_curve() {
        for i in 0..=120 {
            let x = TIME_MAX * f64::from(i) / 120.0;
            let slow = with_resistance(2.0).state(x).voltage_v;
            assert!((slow - state(x / 2.0).voltage_v).abs() < 1e-12);
        }
    }
    #[test]
    fn the_gap_and_the_voltage_always_fill_the_column() {
        for i in 0..=60 {
            let now = state(TIME_MAX * f64::from(i) / 60.0);
            assert!((now.voltage_v + now.gap_v - battery()).abs() < 1e-12);
        }
    }
    #[test]
    fn the_settling_time_is_where_the_model_gap_meets_the_chosen_resolution() {
        assert!((state(settle_time()).gap_v - RESOLUTION_V).abs() < 1e-12);
        assert!(
            state(ZOOM_END_S).gap_v > 0.0,
            "the model's gap is never zero at a finite time"
        );
        assert!(ZOOM_START_S < settle_time() && settle_time() < ZOOM_END_S);
        assert!(state(ZOOM_START_S).voltage_v > ZOOM_LOW_V);
        assert!(state(ZOOM_END_S).voltage_v < battery());
        assert!(ZOOM_HIGH_V > battery());
        assert_eq!(settle_label(), "5.48 s");
        assert_eq!(final_gap_label(), "0.030 V");
    }
    #[test]
    fn the_arrow_points_the_way_current_flows_on_the_bottom_wire() {
        // The long plate is on top, so current goes round clockwise and leftward along the bottom wire.
        const _: () = assert!(BATTERY_LONG > BATTERY_SHORT);
        assert_in_order(&[
            "v(ARROW_X + length / 2.0, ARROW_Y)",
            "v(ARROW_X - length / 2.0, ARROW_Y)",
        ]);
    }
    #[test]
    fn a_vanishing_arrow_is_empty_rather_than_degenerate() {
        assert!(milliamps(m::DEMO, 60.0) * ARROW_FULL / milliamps(m::DEMO, 0.0) < 0.08);
    }
    #[test]
    fn facts_register_without_a_duplicate_key() {
        let mut f = Facts::new();
        append_facts(&mut f).expect("no duplicate keys");
        assert_eq!(f.get("circuit.display.one_tau_volts"), Some("7.59 V"));
        assert_eq!(f.get("rc_010_gap_v"), Some("4.4145532941"));
    }
}
