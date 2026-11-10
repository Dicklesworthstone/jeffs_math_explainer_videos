//! Chapter 10: a rate that changes with the clock. The tiny multipliers of chapter 04 and the uniqueness test of
//! chapter 06 carry over with one change: the rate is read at each moment. Author source, not built or rendered.
use super::ending_draw::{
    dot, fade_in_all, fade_in_lagged, fade_out_all, flat, line_of_math, line_style, live, path, ring, run, seg, swap,
    title, value,
};
use crate::changing_rate_model as m;
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

pub struct ChangingRates {
    pub kit: Kit,
}

const AXIS_X: f64 = -6.1;
const T_UNIT: f64 = 2.55;
const RATE_ZERO: f64 = 1.7;
const RATE_UNIT: f64 = 0.82;
const AMOUNT_ZERO: f64 = -3.05;
const AMOUNT_UNIT: f64 = 1.55;
const CARD_X: f64 = 3.6;
/// The empty band between the panels.
const MULTIPLIER_ROW: f64 = 0.32;
const LIVE_SAMPLES: f64 = 120.0;
/// Seconds of the live sweep; the voice carries the argument while it runs.
const SWEEP: f64 = 6.0;
/// Think time after the 10_2 question, in seconds. Review 721 M10 proposed three to four seconds for this pair. The
/// value is authored here and still has to be set against Frederick Surrey takes in audio-review.
const THINK_10_2: f64 = 4.0;
/// The pond of 10_1, where the panels stand later.
const POND: (f64, f64) = (-3.3, 0.0);
const POND_RADIUS: f64 = 1.75;
/// Algae in the pond, as offsets from its centre: the first `ALGAE_AT_START` at the start, all of them near the peak,
/// and the first `ALGAE_WHILE_DYING` while the colder water thins them. That last count is still above the start, so
/// the pond does not show the ending that 10_2 asks the viewer to predict.
const ALGAE: [(f64, f64); 7] = [
    (-0.55, 0.45),
    (0.5, 0.65),
    (0.15, -0.5),
    (-0.85, -0.35),
    (0.85, -0.15),
    (-0.15, 1.0),
    (-0.45, -0.95),
];
const ALGAE_AT_START: usize = 4;
const ALGAE_WHILE_DYING: usize = 5;
const ALGAE_RADIUS: f64 = 0.12;
const POND_CARD: &str = "algae in a pond, through cooling weather";
const COOLING_CARD: &str = "as the water cools, the rate falls through zero";

const LAW_CONSTANT: &str = r"\frac{dy}{dt}=k\,y";
const LAW_CHANGING: &str = r"\frac{dy}{dt}=r(t)\,y";
const THIS_RATE: &str = r"r(t)=1-t";
const CELL_MULTIPLIER: &str = r"\text{each cell}:\ \times\left(1+r\,w\right)";
const CELL_NOTE: &str = "w is the cell width; r is read at its left edge";
const PRODUCT: &str = r"y_{\text{end}}=(1+r_1w)(1+r_2w)\cdots(1+r_nw)";
const QUESTION: &str = "as they shrink: above, below, or exactly back?";
const CELL_NEAR: &str = r"1+r\,w\approx E(r\,w)";
const SUM_LAW: &str = r"E(a)\,E(b)=E(a+b)";
const AREA_LAW: &str = r"y(t)=E\left(\text{area so far}\right)";
const BACK: &str = r"E(0)=1";
const AREA_DEF: &str = r"R(t)=\int_0^t r(s)\,ds,\qquad R'=r";
const TEST_LINE: &str = r"\frac{d}{dt}\left[y\,E(-R)\right]=\left(y'-r\,y\right)E(-R)=0";
const RESULT: &str = r"y(t)=y(0)\,E\left(R(t)\right)";
const CONTINUOUS: &str = "the rate only has to be continuous";
const NOT_EXPONENTIAL: &str = "not an exponential curve";
const CLOCK: &str = r"y=E\left(R(t)\right),\quad R(t)=t-\frac{t^2}{2}";
const NO_HALF_LIFE: &str = "no single half-life to quote";

fn rate_point(t: f64, r: f64) -> Vec3 {
    v(AXIS_X + T_UNIT * t, RATE_ZERO + RATE_UNIT * r)
}
fn amount_point(t: f64, y: f64) -> Vec3 {
    v(AXIS_X + T_UNIT * t, AMOUNT_ZERO + AMOUNT_UNIT * y)
}
fn polygon_points(n: u32) -> Vec<Vec3> {
    m::stepped(n).into_iter().map(|(t, y)| amount_point(t, y)).collect()
}
fn exact_curve() -> Vec<Vec3> {
    (0..=LIVE_SAMPLES as u32)
        .map(|j| {
            let t = m::HORIZON * f64::from(j) / LIVE_SAMPLES;
            amount_point(t, m::amount(t))
        })
        .collect()
}
fn tick_label(k: &Kit, stage: &mut Stage<'_>, words: &str, x: f64, y: f64) -> fmn::Result<Mob> {
    k.text(stage, words, 21.0, Palette::dim(), v(x, y))
}

/// Both panels' axes and scale marks.
fn frame(k: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let mut out = vec![
        seg(stage, rate_point(0.0, 0.0), rate_point(2.12, 0.0), Palette::dim(), 2.0),
        seg(
            stage,
            rate_point(0.0, -1.12),
            rate_point(0.0, 1.12),
            Palette::dim(),
            2.0,
        ),
        seg(
            stage,
            amount_point(0.0, 0.0),
            amount_point(2.12, 0.0),
            Palette::dim(),
            2.0,
        ),
        seg(
            stage,
            amount_point(0.0, 0.0),
            amount_point(0.0, 1.85),
            Palette::dim(),
            2.0,
        ),
        seg(
            stage,
            amount_point(0.0, 1.0),
            amount_point(2.0, 1.0),
            Palette::faint(),
            2.0,
        ),
    ];
    out.push(k.text(stage, "rate", 23.0, Palette::blue(), v(AXIS_X + 0.55, RATE_ZERO + 1.0))?);
    out.push(k.text(
        stage,
        "amount",
        23.0,
        Palette::green(),
        v(AXIS_X + 0.75, amount_point(0.0, 1.85)[1] + 0.05),
    )?);
    out.push(tick_label(k, stage, "1", AXIS_X - 0.22, rate_point(0.0, 1.0)[1])?);
    out.push(tick_label(
        k,
        stage,
        "\u{2212}1",
        AXIS_X - 0.3,
        rate_point(0.0, -1.0)[1],
    )?);
    out.push(tick_label(k, stage, "1", AXIS_X - 0.22, amount_point(0.0, 1.0)[1])?);
    for t in [1, 2] {
        let x = amount_point(f64::from(t), 0.0)[0];
        out.push(seg(
            stage,
            v(x, AMOUNT_ZERO),
            v(x, AMOUNT_ZERO - 0.08),
            Palette::dim(),
            2.0,
        ));
        out.push(tick_label(k, stage, &t.to_string(), x, AMOUNT_ZERO - 0.27)?);
    }
    let right = amount_point(2.12, 0.0)[0] + 0.42;
    out.push(tick_label(k, stage, "time", right, AMOUNT_ZERO)?);
    out.push(tick_label(k, stage, "time", right, RATE_ZERO)?);
    Ok(out)
}

/// The rate each of `n` cells uses: a flat bar read at the cell's left edge.
fn cell_bars(stage: &mut Stage<'_>, n: u32) -> Vec<Mob> {
    let width = m::HORIZON / f64::from(n);
    let mut out = Vec::new();
    for j in 0..n {
        let t = f64::from(j) * width;
        let r = m::rate(t);
        if r.abs() < 1e-9 {
            continue;
        }
        let corners = [
            rate_point(t, 0.0),
            rate_point(t, r),
            rate_point(t + width, r),
            rate_point(t + width, 0.0),
        ];
        out.push(
            stage
                .arena_mut()
                .add(Polygon::new(corners).style(flat(Palette::blue(), 0.28)).build()),
        );
    }
    out
}

/// One cell's multiplier, in the row under its cell. A row rather than a place on the amount path, so the last
/// multiplier can appear without drawing the ending the viewer is asked to predict.
fn multiplier_label(k: &Kit, stage: &mut Stage<'_>, j: u32) -> fmn::Result<Mob> {
    let width = m::HORIZON / 4.0;
    let t = f64::from(j) * width;
    k.tex(
        stage,
        &format!(r"\times{:.2}", m::multiplier(t, width)),
        23.0,
        Palette::gold(),
        &[],
        v(rate_point(t + width / 2.0, 0.0)[0], MULTIPLIER_ROW),
    )
}

/// The signed area under the rate up to the clock, gold above the axis and red below, redrawn from the model.
fn live_area(stage: &mut Stage<'_>, clock: Mob) -> fmn::Result<Mob> {
    live(stage, clock, |s, now| {
        let t = now.clamp(1e-3, m::HORIZON);
        let g = s.add(VMobject::new());
        let above = t.min(1.0);
        let up = s.add(
            Polygon::new([
                rate_point(0.0, 0.0),
                rate_point(0.0, m::rate(0.0)),
                rate_point(above, m::rate(above)),
                rate_point(above, 0.0),
            ])
            .style(flat(Palette::gold(), 0.45))
            .build(),
        );
        let _ = s.attach(g, up);
        if t > 1.0 {
            let down = s.add(
                Polygon::new([rate_point(1.0, 0.0), rate_point(t, m::rate(t)), rate_point(t, 0.0)])
                    .style(flat(Palette::red(), 0.45))
                    .build(),
            );
            let _ = s.attach(g, down);
        }
        g
    })
}

/// The amount curve up to the clock, drawn from the area formula.
fn live_amount(stage: &mut Stage<'_>, clock: Mob) -> fmn::Result<Mob> {
    live(stage, clock, |s, now| {
        let t = now.clamp(0.0, m::HORIZON);
        let samples = ((t / m::HORIZON) * LIVE_SAMPLES).ceil().max(1.0) as usize;
        let points: Vec<Vec3> = (0..=samples)
            .map(|j| {
                let u = t * j as f64 / samples as f64;
                amount_point(u, m::amount(u))
            })
            .collect();
        s.add(polyline(points).style(line_style(Palette::green(), 4.5)).build())
    })
}

/// The point on the amount curve at the clock.
fn live_marker(stage: &mut Stage<'_>, clock: Mob) -> fmn::Result<Mob> {
    live(stage, clock, |s, now| {
        let t = now.clamp(0.0, m::HORIZON);
        s.add(
            Dot::new()
                .point(amount_point(t, m::amount(t)))
                .radius(0.09)
                .style(flat(Palette::hero(), 1.0))
                .build(),
        )
    })
}

/// 10_1: the constant rate gives way to one that depends on the clock. First a situation, algae in a pond as the
/// water cools (review 721 M5), then the invented straight-line rate that stands for it. Returns the `r(t)` card,
/// which 10_5 fades before its rows take that place (review 721 M8).
fn a_rate_that_moves(k: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    k.say(stage, "10_1")?;
    let heading = title(k, stage, "What if the rate changes?")?;
    fade_in_all(stage, &[heading], 0.5)?;
    k.until_sentence(stage, "10_1", 1)?;
    let constant = line_of_math(k, stage, LAW_CONSTANT, 38.0, Palette::ink(), CARD_X, 1.9)?;
    fade_in_all(stage, &[constant], 0.5)?;

    k.until_sentence(stage, "10_1", 3)?;
    let centre = v(POND.0, POND.1);
    let water = stage.arena_mut().add(
        Dot::new()
            .point(centre)
            .radius(POND_RADIUS)
            .style(flat(Palette::blue(), 0.16))
            .build(),
    );
    let shore = ring(stage, centre, POND_RADIUS, Palette::blue(), 2.5);
    let algae: Vec<Mob> = ALGAE
        .iter()
        .map(|&(dx, dy)| dot(stage, v(POND.0 + dx, POND.1 + dy), ALGAE_RADIUS, Palette::green()))
        .collect();
    let pond_card = k.text(stage, POND_CARD, 24.0, Palette::dim(), v(CARD_X, 0.9))?;
    let mut first = vec![water, shore, pond_card];
    first.extend_from_slice(&algae[..ALGAE_AT_START]);
    fade_in_all(stage, &first, 0.6)?;
    k.until_sentence(stage, "10_1", 4)?;
    fade_in_lagged(stage, &algae[ALGAE_AT_START..], 1.2, 0.3)?;
    k.until_sentence(stage, "10_1", 5)?;
    let cooling = k.text(stage, COOLING_CARD, 24.0, Palette::red(), v(CARD_X, 0.3))?;
    fade_in_all(stage, &[cooling], 0.4)?;
    fade_out_all(stage, &algae[ALGAE_WHILE_DYING..], 0.8)?;

    k.until_sentence(stage, "10_1", 6)?;
    let changing = line_of_math(k, stage, LAW_CHANGING, 38.0, Palette::ink(), CARD_X, 1.9)?;
    swap(stage, &[constant], &[changing], 0.6)?;

    k.until_sentence(stage, "10_1", 9)?;
    let mut numbers = frame(k, stage)?;
    let line = path(
        stage,
        vec![
            rate_point(0.0, m::rate(0.0)),
            rate_point(m::HORIZON, m::rate(m::HORIZON)),
        ],
        Palette::blue(),
        4.5,
    );
    let this_rate = line_of_math(k, stage, THIS_RATE, 32.0, Palette::blue(), CARD_X, 1.0)?;
    let start = dot(stage, amount_point(0.0, 1.0), 0.09, Palette::green());
    numbers.extend([line, this_rate, start]);
    let mut pond = vec![water, shore, pond_card, cooling];
    pond.extend_from_slice(&algae[..ALGAE_WHILE_DYING]);
    swap(stage, &pond, &numbers, 0.7)?;
    k.hold(stage)?;
    Ok(this_rate)
}

/// 10_2: four cells, each a multiplier, then the prediction. The four-cell path is drawn to its end, 1.40625. These
/// crude cells read the rate at their left edges, so they finish above the start, and that conflict is the question:
/// what happens as the cells shrink (review 721 H2).
fn four_cells(k: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "10_2")?;
    k.until_sentence(stage, "10_2", 1)?;
    let bars = cell_bars(stage, 4);
    fade_in_all(stage, &bars, 0.5)?;
    k.until_sentence(stage, "10_2", 2)?;
    let cell = line_of_math(k, stage, CELL_MULTIPLIER, 30.0, Palette::gold(), CARD_X, 0.2)?;
    let note = k.text(stage, CELL_NOTE, 21.0, Palette::dim(), v(CARD_X, -0.3))?;
    let steps = path(stage, polygon_points(4)[..4].to_vec(), Palette::green(), 3.5);
    fade_in_all(stage, &[cell, note, steps], 0.5)?;

    k.until_sentence(stage, "10_2", 3)?;
    let first = [multiplier_label(k, stage, 0)?, multiplier_label(k, stage, 1)?];
    fade_in_all(stage, &first, 0.4)?;
    k.until_sentence(stage, "10_2", 4)?;
    let middle = multiplier_label(k, stage, 2)?;
    fade_in_all(stage, &[middle], 0.4)?;
    k.until_sentence(stage, "10_2", 5)?;
    let last = multiplier_label(k, stage, 3)?;
    fade_in_all(stage, &[last], 0.4)?;

    k.until_sentence(stage, "10_2", 6)?;
    let product = line_of_math(k, stage, PRODUCT, 27.0, Palette::green(), CARD_X, -1.0)?;
    fade_in_all(stage, &[product], 0.5)?;
    k.until_sentence(stage, "10_2", 7)?;
    let last_step = path(stage, polygon_points(4)[3..].to_vec(), Palette::green(), 3.5);
    let end = polygon_points(4)[4];
    let end_value = format!("{:.5}", m::stepped_end(4));
    let end_label = line_of_math(k, stage, &end_value, 23.0, Palette::green(), end[0] + 0.55, end[1])?;
    fade_in_all(stage, &[last_step, end_label], 0.5)?;
    k.until_sentence(stage, "10_2", 11)?;
    let question = k.text(stage, QUESTION, 26.0, Palette::hero(), v(CARD_X, -2.1))?;
    fade_in_all(stage, &[question], 0.5)?;
    k.hold(stage)?;
    k.mark(stage, "predict");
    k.music_breath(stage, THINK_10_2)?;
    let mut shown = bars;
    shown.extend([
        cell, note, steps, last_step, end_label, first[0], first[1], middle, last, product, question,
    ]);
    Ok(shown)
}

/// 10_3: exactly back, and why: the multipliers add their exponents, and the sum is an area. The live sweep draws
/// the true curve from the area, and reaches time two with the area cancelled.
fn the_area_decides(k: &Kit, stage: &mut Stage<'_>, cells: &[Mob]) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "10_3")?;
    let back_dot = dot(stage, amount_point(m::HORIZON, 1.0), 0.12, Palette::hero());
    fade_in_all(stage, &[back_dot], 0.3)?;

    k.until_sentence(stage, "10_3", 2)?;
    fade_out_all(stage, cells, 0.4)?;
    let near = line_of_math(k, stage, CELL_NEAR, 31.0, Palette::gold(), CARD_X, 0.2)?;
    fade_in_all(stage, &[near], 0.5)?;
    k.until_sentence(stage, "10_3", 3)?;
    let sum = line_of_math(k, stage, SUM_LAW, 31.0, Palette::ink(), CARD_X, -0.45)?;
    fade_in_all(stage, &[sum], 0.5)?;

    k.until_sentence(stage, "10_3", 4)?;
    let whole_above = stage.arena_mut().add(
        Polygon::new([
            rate_point(0.0, 0.0),
            rate_point(0.0, m::rate(0.0)),
            rate_point(1.0, 0.0),
        ])
        .style(flat(Palette::gold(), 0.45))
        .build(),
    );
    let whole_below = stage.arena_mut().add(
        Polygon::new([
            rate_point(1.0, 0.0),
            rate_point(m::HORIZON, m::rate(m::HORIZON)),
            rate_point(m::HORIZON, 0.0),
        ])
        .style(flat(Palette::red(), 0.45))
        .build(),
    );
    fade_in_all(stage, &[whole_above, whole_below], 0.5)?;

    k.until_sentence(stage, "10_3", 5)?;
    let law = line_of_math(k, stage, AREA_LAW, 31.0, Palette::green(), CARD_X, -1.2)?;
    fade_out_all(stage, &[whole_above, whole_below, back_dot], 0.3)?;
    let clock = stage.add_value_tracker(0.0);
    let area = live_area(stage, clock)?;
    let curve = live_amount(stage, clock)?;
    let marker = live_marker(stage, clock)?;
    let area_name = line_of_math(k, stage, r"\text{area}=", 28.0, Palette::gold(), CARD_X - 1.1, -2.0)?;
    let area_read = k.live_value(
        stage,
        move |s| m::area(value(s, clock).clamp(0.0, m::HORIZON)),
        3,
        false,
        28.0,
        Palette::gold(),
        v(CARD_X - 0.3, -2.0),
        LEFT,
    )?;
    let amount_name = line_of_math(
        k,
        stage,
        r"\text{amount}=",
        28.0,
        Palette::green(),
        CARD_X - 1.25,
        -2.55,
    )?;
    let amount_read = k.live_value(
        stage,
        move |s| m::amount(value(s, clock).clamp(0.0, m::HORIZON)),
        3,
        false,
        28.0,
        Palette::green(),
        v(CARD_X - 0.3, -2.55),
        LEFT,
    )?;
    fade_in_all(stage, &[law, area_name, amount_name], 0.4)?;
    run(stage, clock, m::HORIZON, SWEEP)?;

    k.until_sentence(stage, "10_3", 6)?;
    let done = line_of_math(k, stage, BACK, 32.0, Palette::hero(), CARD_X + 2.3, -2.27)?;
    fade_in_all(stage, &[done], 0.4)?;
    k.hold(stage)?;
    Ok(vec![
        near,
        sum,
        law,
        area,
        curve,
        marker,
        area_name,
        area_read,
        amount_name,
        amount_read,
        done,
    ])
}

/// 10_4: the exact proof is the chapter 06 test, with the area where k t used to be.
fn the_test_again(k: &Kit, stage: &mut Stage<'_>, argument: &[Mob]) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "10_4")?;
    fade_out_all(stage, argument, 0.4)?;
    k.until_sentence(stage, "10_4", 2)?;
    let definition = line_of_math(k, stage, AREA_DEF, 29.0, Palette::gold(), CARD_X, 0.15)?;
    fade_in_all(stage, &[definition], 0.5)?;
    k.until_sentence(stage, "10_4", 3)?;
    let test = line_of_math(k, stage, TEST_LINE, 27.0, Palette::ink(), CARD_X, -0.75)?;
    fade_in_all(stage, &[test], 0.5)?;
    k.until_sentence(stage, "10_4", 5)?;
    let result = line_of_math(k, stage, RESULT, 33.0, Palette::green(), CARD_X, -1.65)?;
    fade_in_all(stage, &[result], 0.5)?;
    k.until_sentence(stage, "10_4", 6)?;
    let condition = k.text(stage, CONTINUOUS, 24.0, Palette::dim(), v(CARD_X, -2.4))?;
    fade_in_all(stage, &[condition], 0.4)?;
    k.hold(stage)?;
    Ok(vec![definition, test, result, condition])
}

/// 10_5: more cells close in on the curve. The four-cell bars return while the voice gives the reason four cells
/// finish high: each reads the rate at its left edge, the highest point of its cell (review 721 H2). Then the curve
/// is the exponential of a clock, and it is not an exponential.
fn the_curve_is_a_clock(k: &Kit, stage: &mut Stage<'_>, proof: &[Mob]) -> fmn::Result<()> {
    k.say(stage, "10_5")?;
    fade_out_all(stage, proof, 0.4)?;
    let curve = path(stage, exact_curve(), Palette::green(), 4.5);
    fade_in_all(stage, &[curve], 0.4)?;
    let mut drawn: Vec<Mob> = Vec::new();
    let reveal_at = [1_usize, 3, 4, 4];
    let mut said = 0;
    for (i, n) in m::STEP_COUNTS.into_iter().enumerate() {
        if reveal_at[i] != said {
            said = reveal_at[i];
            k.until_sentence(stage, "10_5", said)?;
        }
        let width = if n <= 16 { 2.5 } else { 1.8 };
        let this = path(stage, polygon_points(n), Palette::teal(), width);
        let y = 1.05 - 0.55 * i as f64;
        let row = k.tex(
            stage,
            &format!(r"{n}\ \text{{cells}}:\quad {:.5}", m::stepped_end(n)),
            27.0,
            Palette::teal(),
            &[],
            v(CARD_X, y),
        )?;
        fade_in_all(stage, &[this, row], 0.4)?;
        drawn.push(this);
        if i == 0 {
            k.until_sentence(stage, "10_5", 2)?;
            said = 2;
            let edges = cell_bars(stage, 4);
            fade_in_all(stage, &edges, 0.5)?;
            drawn.extend(edges);
        }
    }
    k.until_sentence(stage, "10_5", 5)?;
    fade_out_all(stage, &drawn, 0.4)?;
    k.until_sentence(stage, "10_5", 6)?;
    let peak = dot(stage, amount_point(1.0, m::amount(1.0)), 0.09, Palette::hero());
    let word = k.text(stage, NOT_EXPONENTIAL, 27.0, Palette::hero(), v(CARD_X, -1.2))?;
    fade_in_all(stage, &[peak, word], 0.4)?;
    k.until_sentence(stage, "10_5", 7)?;
    let clock_line = line_of_math(k, stage, CLOCK, 29.0, Palette::green(), CARD_X, -1.9)?;
    fade_in_all(stage, &[clock_line], 0.5)?;
    k.until_sentence(stage, "10_5", 10)?;
    let no_half = k.text(stage, NO_HALF_LIFE, 25.0, Palette::dim(), v(CARD_X, -2.6))?;
    fade_in_all(stage, &[no_half], 0.4)?;
    Ok(())
}

impl SceneConstruct for ChangingRates {
    fn name(&self) -> &str {
        "10_changing_rates"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let this_rate = a_rate_that_moves(&k, stage)?;
        let cells = four_cells(&k, stage)?;
        let argument = the_area_decides(&k, stage, &cells)?;
        let mut proof = the_test_again(&k, stage, &argument)?;
        // Review 721 M8: the r(t) card leaves with the proof, before the first 10_5 row lands where it stood.
        proof.push(this_rate);
        the_curve_is_a_clock(&k, stage, &proof)?;
        k.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::ending_draw::{assert_in_order, scene_source};
    const SOURCE: &str = include_str!("changing_rates.rs");
    use super::{ALGAE, ALGAE_AT_START, ALGAE_RADIUS, ALGAE_WHILE_DYING, POND_RADIUS, THINK_10_2};
    use crate::changing_rate_model as m;

    #[test]
    fn the_prediction_comes_before_the_endpoint_is_drawn() {
        assert_in_order(
            SOURCE,
            &[
                r#"k.until_sentence(stage, "10_2", 11)?;"#,
                "k.text(stage, QUESTION",
                "k.hold(stage)?;",
                r#"k.mark(stage, "predict");"#,
                "k.music_breath(stage, THINK_10_2)?;",
                r#"k.say(stage, "10_3")?;"#,
                "let back_dot = dot(stage, amount_point(m::HORIZON, 1.0)",
            ],
        );
        // Review 721 M10 proposed three to four seconds for this pair; audio-review decides the final length.
        const { assert!(THINK_10_2 >= 3.0) };
        const { assert!(THINK_10_2 <= 4.0) };
    }
    #[test]
    fn the_four_cell_path_reaches_its_end_before_the_question() {
        // Review 721 H2: the crude ending is on screen, said, and only then questioned.
        assert_in_order(
            SOURCE,
            &[
                "fn four_cells",
                "polygon_points(4)[..4]",
                r#"k.until_sentence(stage, "10_2", 7)?;"#,
                "polygon_points(4)[3..]",
                "m::stepped_end(4)",
                r#"k.until_sentence(stage, "10_2", 11)?;"#,
                "QUESTION",
                "fn the_area_decides",
            ],
        );
        assert_eq!(format!("{:.5}", m::stepped_end(4)), "1.40625");
    }
    #[test]
    fn the_left_edge_bars_return_with_the_four_cell_row() {
        assert_in_order(
            SOURCE,
            &[
                "fn the_curve_is_a_clock",
                "let reveal_at = [1_usize, 3, 4, 4];",
                "if i == 0 {",
                r#"k.until_sentence(stage, "10_5", 2)?;"#,
                "cell_bars(stage, 4)",
                "drawn.extend(edges);",
                r#"k.until_sentence(stage, "10_5", 5)?;"#,
                "fade_out_all(stage, &drawn, 0.4)?;",
            ],
        );
        // Each bar is read at its cell's left edge, where this falling rate is highest.
        let width = m::HORIZON / 4.0;
        for j in 0..4 {
            let left = f64::from(j) * width;
            assert!(m::rate(left) > m::rate(left + width / 2.0));
        }
    }
    #[test]
    fn the_rate_card_leaves_before_the_rows_take_its_place() {
        // Review 721 M8.
        assert_in_order(
            SOURCE,
            &[
                "fn a_rate_that_moves",
                "let this_rate = line_of_math(k, stage, THIS_RATE",
                "Ok(this_rate)",
                "fn the_curve_is_a_clock",
                "fade_out_all(stage, proof, 0.4)?;",
                "m::STEP_COUNTS",
                "let this_rate = a_rate_that_moves(&k, stage)?;",
                "proof.push(this_rate);",
                "the_curve_is_a_clock(&k, stage, &proof)?;",
            ],
        );
    }
    #[test]
    fn the_pond_comes_first_and_does_not_show_the_ending() {
        // Review 721 M5: the situation, then the invented numbers.
        assert_in_order(
            SOURCE,
            &[
                r#"k.until_sentence(stage, "10_1", 3)?;"#,
                "POND_CARD",
                r#"k.until_sentence(stage, "10_1", 4)?;"#,
                "&algae[ALGAE_AT_START..]",
                r#"k.until_sentence(stage, "10_1", 5)?;"#,
                "&algae[ALGAE_WHILE_DYING..]",
                r#"k.until_sentence(stage, "10_1", 6)?;"#,
                "LAW_CHANGING",
                r#"k.until_sentence(stage, "10_1", 9)?;"#,
                "THIS_RATE",
                "swap(stage, &pond, &numbers, 0.7)?;",
            ],
        );
        // The full pond is the peak of the model, E(1/2) times the start, and the thinned pond is still above the
        // start, so the picture cannot answer the 10_2 question.
        assert_eq!((ALGAE_AT_START as f64 * m::amount(1.0)).round() as usize, ALGAE.len());
        const { assert!(ALGAE_AT_START < ALGAE_WHILE_DYING) };
        const { assert!(ALGAE_WHILE_DYING < ALGAE.len()) };
        for (dx, dy) in ALGAE {
            assert!(dx.hypot(dy) + ALGAE_RADIUS < POND_RADIUS);
        }
    }
    #[test]
    fn the_sweep_is_drawn_from_the_area_formula() {
        let drawn = scene_source(SOURCE)
            .split("fn live_amount")
            .nth(1)
            .unwrap()
            .split("fn live_marker")
            .next()
            .unwrap();
        assert!(drawn.contains("m::amount(u)"));
    }
    #[test]
    fn the_proof_cues_follow_the_voice() {
        assert_in_order(
            SOURCE,
            &[
                r#"k.say(stage, "10_4")?;"#,
                "AREA_DEF",
                "TEST_LINE",
                "RESULT",
                "CONTINUOUS",
                r#"k.say(stage, "10_5")?;"#,
                "m::STEP_COUNTS",
                "NOT_EXPONENTIAL",
                "CLOCK",
                "NO_HALF_LIFE",
            ],
        );
    }
}
