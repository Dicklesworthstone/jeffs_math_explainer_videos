//! Chapter 13: a rate of i. The velocity is the arrow turned a quarter turn, so the distance from zero never changes
//! and the walk has speed one: the circle is earned from the rate law, not assumed. The heading, the prediction and
//! the left-plane/right-card layout come from the earlier draft's `rotation.rs`; its stepped dot becomes a walk at
//! constant speed. Author source, not built or rendered.
use super::ending_draw::{
    Plane, arrow, arrow_parts, dot, fade_in_all, fade_in_lagged, fade_out_all, flat, line_of_math, line_style, live,
    path, seg, swap, title, walk,
};
use crate::turning_model::{self as t, C};
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;
use std::f64::consts::{PI, TAU};

pub struct TurningRate {
    pub kit: Kit,
}

const CARD_X: f64 = 3.4;
const ZERO: C = C::new(0.0, 0.0);
/// The side of the right-angle marks of 13_1, in scene units.
const CORNER: f64 = 0.24;
/// Review 721 L5: the 13_2 inward arrow runs this far below the position arrow it would otherwise cover. The two
/// shafts are about 0.05 wide, so a smaller offset would leave almost no visible gap.
const INWARD_OFFSET: f64 = 0.1;
/// Think time after the 13_2 question, in seconds. Review 721 M10 proposed two to three seconds for this pair. The
/// value is authored here and still has to be set against Frederick Surrey takes in audio-review.
const THINK_13_2: f64 = 3.0;
/// The 13_1 spring, left of the cards, where the plane stands later (review 721 M4).
const SPRING_WALL: f64 = -6.1;
const BLOCK_LEFT: f64 = -4.0;
const COIL_TEETH: u32 = 8;
const SPRING_LAW: &str = r"\text{acceleration}=-\text{position}";
const NO_REAL_RATE: &str = r"r\cdot r=r^2,\ \text{never negative}";
const NEW_NUMBER: &str = r"i^2=-1";
/// Review 721 H1: what the machinery really asked for, one card per sentence, then the claim.
const REQUIREMENTS: [&str; 3] = [
    "adds and multiplies, in either order",
    "divides by whole numbers",
    "measures sizes: the term sizes have a finite sum",
];
const COMPLEX_DOES: &str = "complex numbers do all of it";
const GENERAL_RULE: &str = r"i\,(x+iy)=-y+ix";
/// Review 721 W8: named once 13_4 has earned it.
const FORMULA_NAME: &str = "Euler's formula";
/// How far the live walk has gone when the formula is named.
const WALKED: f64 = 2.2;
/// The right-hand trace panels of 13_5.
const TRACE_LEFT: f64 = 0.9;
const TRACE_UNIT: f64 = 0.85;
const TRACE_HEIGHT: f64 = 0.6;
const COSINE_Y: f64 = 2.0;
const SINE_Y: f64 = 0.45;
const TRACE_SECONDS: f64 = 5.0;

const SERIES: &str = r"E(it)=\sum_{k=0}^{\infty}\frac{(it)^k}{k!}";
const CEILING: &str = r"\left|\frac{(it)^k}{k!}\right|=\frac{|t|^k}{k!}";
const TERM_BY_TERM: &str = r"\frac{d}{dt}\,E(it)=i\,E(it)";
const RATE_LAW: &str = r"z=E(it):\quad z'=i\,z";
const COORDINATES: &str = r"z=x+iy";
const COMPONENTS: &str = r"x'=-y,\qquad y'=x";
const NO_CHANGE: &str = r"\frac{d}{dt}\left(x^2+y^2\right)=2x(-y)+2y(x)=0";
const STAYS: &str = r"|z|=1\ \text{forever}";
const SPEED: &str = r"|z'|=|iz|=1";
const EULER: &str = r"E(it)=\cos t+i\sin t";
const SLOPES: &str = r"(\cos t)'=-\sin t,\qquad (\sin t)'=\cos t";
const TWICE: &str = r"z''=i\cdot i\,z=-z";

fn plane() -> Plane {
    Plane {
        centre: v(-3.3, -0.25),
        scale: 1.45,
        half_w: 3.0,
        half_h: 2.55,
    }
}
fn name_at(k: &Kit, stage: &mut Stage<'_>, src: &str, z: C, dx: f64, dy: f64) -> fmn::Result<Mob> {
    let at = plane().p(z);
    k.tex(stage, src, 27.0, Palette::ink(), &[], v(at[0] + dx, at[1] + dy))
}
fn card(k: &Kit, stage: &mut Stage<'_>, words: &str, y: f64, c: Srgb) -> fmn::Result<Mob> {
    k.text(stage, words, 25.0, c, v(CARD_X, y))
}
fn point(z: C, c: Srgb, stage: &mut Stage<'_>) -> Mob {
    dot(stage, plane().p(z), 0.09, c)
}

/// The tip of the walk at time `t` on the unit circle, rebuilt from the tracker.
fn live_dot(stage: &mut Stage<'_>, clock: Mob) -> fmn::Result<Mob> {
    live(stage, clock, |s, now| {
        s.add(
            Dot::new()
                .point(plane().p(t::on_circle(now)))
                .radius(0.1)
                .style(flat(Palette::hero(), 1.0))
                .build(),
        )
    })
}
/// The arrow from zero to the walk's tip.
fn live_position(stage: &mut Stage<'_>, clock: Mob) -> fmn::Result<Mob> {
    live(stage, clock, |s, now| {
        arrow_parts(
            s,
            plane().p(ZERO),
            plane().p(t::on_circle(now)),
            Palette::gold(),
            4.5,
            0.18,
        )
    })
}
/// The velocity at the tip: the position arrow turned a quarter turn, drawn from the tip at true length.
fn live_velocity(stage: &mut Stage<'_>, clock: Mob) -> fmn::Result<Mob> {
    live(stage, clock, |s, now| {
        let z = t::on_circle(now);
        arrow_parts(
            s,
            plane().p(z),
            plane().p(z.plus(z.times_i())),
            Palette::hero(),
            4.5,
            0.18,
        )
    })
}
/// The distance already walked along the circle.
fn live_arc(stage: &mut Stage<'_>, clock: Mob) -> fmn::Result<Mob> {
    live(stage, clock, |s, now| {
        let points = plane().arc(0.0, now.max(1e-3));
        s.add(polyline(points).style(line_style(Palette::gold(), 6.0)).build())
    })
}

/// A small square at zero, between the directions of `from` and `to`, which meet at a right angle.
fn right_angle(stage: &mut Stage<'_>, from: C, to: C) -> Mob {
    let (along, across) = (from.scale(CORNER / from.abs()), to.scale(CORNER / to.abs()));
    let at = |z: C| plane().p(z.scale(1.0 / plane().scale));
    path(
        stage,
        vec![at(along), at(along.plus(across)), at(across)],
        Palette::ink(),
        2.5,
    )
}
/// An arrow from zero, a dot at its tip and the tip's name: one still position, no motion.
fn still_arrow(k: &Kit, stage: &mut Stage<'_>, z: C, name: &str, dx: f64, dy: f64, c: Srgb) -> fmn::Result<[Mob; 2]> {
    Ok([
        arrow(stage, plane().p(ZERO), plane().p(z), c, 5.0),
        name_at(k, stage, name, z, dx, dy)?,
    ])
}
/// A wall, a coil and the weight, drawn where the plane stands later.
fn spring_sketch(stage: &mut Stage<'_>) -> Vec<Mob> {
    let wall = seg(stage, v(SPRING_WALL, 0.35), v(SPRING_WALL, 1.65), Palette::dim(), 4.0);
    let floor = seg(stage, v(SPRING_WALL, 0.35), v(-2.4, 0.35), Palette::faint(), 2.0);
    let (coil_start, coil_end) = (SPRING_WALL + 0.2, BLOCK_LEFT - 0.2);
    let mut coil = vec![v(SPRING_WALL, 1.0), v(coil_start, 1.0)];
    for j in 1..COIL_TEETH {
        let x = coil_start + (coil_end - coil_start) * f64::from(j) / f64::from(COIL_TEETH);
        coil.push(v(x, if j.is_multiple_of(2) { 0.82 } else { 1.18 }));
    }
    coil.extend([v(coil_end, 1.0), v(BLOCK_LEFT, 1.0)]);
    let spring = path(stage, coil, Palette::ink(), 3.0);
    let block = stage.arena_mut().add(
        Polygon::new([
            v(BLOCK_LEFT, 0.35),
            v(BLOCK_LEFT, 1.35),
            v(BLOCK_LEFT + 1.0, 1.35),
            v(BLOCK_LEFT + 1.0, 0.35),
        ])
        .style(flat(Palette::gold(), 0.85))
        .build(),
    );
    vec![wall, floor, spring, block]
}

/// 13_1: a spring asks for a rate whose square is negative, so i joins the numbers. The machinery's real needs are
/// listed and complex numbers meet them (review 721 H1, M4). Multiplying by i is shown as still jumps, 1 to i to -1,
/// with no tracker, so nothing traces the circle before the 13_2 question (review 721 M3). Then a worked case.
fn the_quarter_turn(k: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    k.say(stage, "13_1")?;
    let heading = title(k, stage, "What happens when the exponent turns?")?;
    fade_in_all(stage, &[heading], 0.5)?;

    k.until_sentence(stage, "13_1", 1)?;
    let signs = [
        card(k, stage, "rate above zero: growth", 2.3, Palette::green())?,
        card(k, stage, "rate below zero: decay", 1.75, Palette::red())?,
    ];
    fade_in_all(stage, &signs, 0.4)?;
    k.until_sentence(stage, "13_1", 2)?;
    let mut spring = spring_sketch(stage);
    spring.push(line_of_math(k, stage, SPRING_LAW, 27.0, Palette::ink(), -4.25, -0.3)?);
    fade_in_all(stage, &spring, 0.5)?;
    k.until_sentence(stage, "13_1", 3)?;
    let no_real_rate = line_of_math(k, stage, NO_REAL_RATE, 28.0, Palette::dim(), CARD_X, 1.1)?;
    fade_in_all(stage, &[no_real_rate], 0.4)?;
    k.until_sentence(stage, "13_1", 4)?;
    let new_number = line_of_math(k, stage, NEW_NUMBER, 36.0, Palette::hero(), CARD_X, -0.3)?;
    fade_in_all(stage, &[new_number], 0.5)?;

    k.until_sentence(stage, "13_1", 5)?;
    fade_out_all(stage, &[signs[0], signs[1], no_real_rate], 0.3)?;
    let mut needs = Vec::new();
    for (j, (words, y)) in REQUIREMENTS.into_iter().zip([2.3, 1.75, 1.2]).enumerate() {
        k.until_sentence(stage, "13_1", 6 + j)?;
        let need = card(k, stage, words, y, Palette::ink())?;
        fade_in_all(stage, &[need], 0.4)?;
        needs.push(need);
    }
    k.until_sentence(stage, "13_1", 9)?;
    let complex = card(k, stage, COMPLEX_DOES, 0.6, Palette::hero())?;
    fade_in_all(stage, &[complex], 0.4)?;

    k.until_sentence(stage, "13_1", 10)?;
    let axes = plane().axes(stage)?;
    swap(stage, &spring, &[axes], 0.6)?;
    k.until_sentence(stage, "13_1", 11)?;
    let foot = C::new(t::EXAMPLE.re, 0.0);
    let legs = [
        arrow(stage, plane().p(ZERO), plane().p(t::EXAMPLE), Palette::gold(), 5.0),
        seg(stage, plane().p(ZERO), plane().p(foot), Palette::blue(), 3.0),
        seg(stage, plane().p(foot), plane().p(t::EXAMPLE), Palette::teal(), 3.0),
        name_at(k, stage, "x", C::new(t::EXAMPLE.re / 2.0, 0.0), 0.0, -0.27)?,
        name_at(k, stage, "y", C::new(t::EXAMPLE.re, t::EXAMPLE.im / 2.0), 0.25, 0.0)?,
        name_at(k, stage, "x+iy", t::EXAMPLE, 0.5, 0.2)?,
    ];
    fade_in_all(stage, &legs, 0.5)?;

    k.until_sentence(stage, "13_1", 12)?;
    fade_out_all(stage, &legs, 0.3)?;
    let quarter = t::ONE.times_i();
    let half = quarter.times_i();
    let one = still_arrow(k, stage, t::ONE, "1", 0.2, -0.27, Palette::gold())?;
    fade_in_all(stage, &one, 0.4)?;
    k.until_sentence(stage, "13_1", 13)?;
    let mut jumps = one.to_vec();
    jumps.extend(still_arrow(k, stage, quarter, "i", 0.25, 0.1, Palette::hero())?);
    jumps.push(right_angle(stage, t::ONE, quarter));
    fade_in_all(stage, &jumps[2..], 0.4)?;
    k.until_sentence(stage, "13_1", 14)?;
    let to_minus_one = still_arrow(k, stage, half, "-1", -0.32, 0.2, Palette::hero())?;
    let second_turn = right_angle(stage, quarter, half);
    fade_in_all(stage, &[to_minus_one[0], to_minus_one[1], second_turn], 0.4)?;
    jumps.extend(to_minus_one);
    jumps.push(second_turn);

    // Review 721 M4: a worked case with numbers, before the letters. (1.5, 1) stays inside the plane when turned.
    k.until_sentence(stage, "13_1", 15)?;
    fade_out_all(stage, &jumps, 0.3)?;
    let turned = t::EXAMPLE.times_i();
    let start = still_arrow(
        k,
        stage,
        t::EXAMPLE,
        &format!("{}+i", t::EXAMPLE.re),
        0.5,
        0.2,
        Palette::gold(),
    )?;
    fade_in_all(stage, &start, 0.4)?;
    k.until_sentence(stage, "13_1", 16)?;
    let worked_name = format!("{}+{}i", turned.re, turned.im);
    let mut pair = start.to_vec();
    pair.extend(still_arrow(
        k,
        stage,
        turned,
        &worked_name,
        -0.55,
        0.25,
        Palette::hero(),
    )?);
    pair.push(right_angle(stage, t::EXAMPLE, turned));
    let worked = line_of_math(
        k,
        stage,
        &format!(r"i\,({}+i)={worked_name}", t::EXAMPLE.re),
        30.0,
        Palette::hero(),
        CARD_X,
        -1.1,
    )?;
    fade_in_all(stage, &[pair[2], pair[3], pair[4], worked], 0.5)?;
    k.until_sentence(stage, "13_1", 17)?;
    let rule = line_of_math(k, stage, GENERAL_RULE, 32.0, Palette::hero(), CARD_X, -1.85)?;
    fade_in_all(stage, &[rule], 0.5)?;
    k.hold(stage)?;

    let mut leaving = needs;
    leaving.extend([complex, new_number, worked, rule, heading]);
    leaving.extend(pair);
    fade_out_all(stage, &leaving, 0.4)
}

/// 13_2: the series obeys a rate law with a rate of i, then three rates at one point and the prediction. No circle
/// and no path are drawn before the hold.
fn three_rates(k: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "13_2")?;
    let mut cards = vec![line_of_math(k, stage, SERIES, 31.0, Palette::ink(), CARD_X, 2.3)?];
    fade_in_all(stage, &cards[..1], 0.5)?;
    k.until_sentence(stage, "13_2", 1)?;
    cards.push(line_of_math(k, stage, CEILING, 29.0, Palette::dim(), CARD_X, 1.2)?);
    fade_in_all(stage, &cards[1..2], 0.5)?;
    k.until_sentence(stage, "13_2", 2)?;
    cards.push(line_of_math(
        k,
        stage,
        TERM_BY_TERM,
        30.0,
        Palette::ink(),
        CARD_X,
        0.35,
    )?);
    fade_in_all(stage, &cards[2..3], 0.5)?;
    k.until_sentence(stage, "13_2", 3)?;
    cards.push(line_of_math(k, stage, RATE_LAW, 33.0, Palette::green(), CARD_X, -0.45)?);
    fade_in_all(stage, &cards[3..4], 0.5)?;

    k.until_sentence(stage, "13_2", 4)?;
    let position = arrow(stage, plane().p(ZERO), plane().p(t::ONE), Palette::gold(), 4.5);
    let start = point(t::ONE, Palette::gold(), stage);
    fade_in_all(stage, &[position, start], 0.4)?;

    k.until_sentence(stage, "13_2", 5)?;
    let out_end = t::ONE.plus(t::ONE);
    let out = [
        arrow(stage, plane().p(t::ONE), plane().p(out_end), Palette::blue(), 5.0),
        card(k, stage, "rate 1: straight out", -1.15, Palette::blue())?,
    ];
    fade_in_all(stage, &out, 0.4)?;
    k.until_sentence(stage, "13_2", 6)?;
    // Review 721 L5: drawn just below the gold position arrow, which it would otherwise cover exactly.
    let below = |z: C| {
        let p = plane().p(z);
        v(p[0], p[1] - INWARD_OFFSET)
    };
    let inward = [
        arrow(stage, below(t::ONE), below(ZERO), Palette::red(), 5.0),
        card(k, stage, "rate \u{2212}1: straight in", -1.6, Palette::red())?,
    ];
    fade_in_all(stage, &inward, 0.4)?;
    k.until_sentence(stage, "13_2", 7)?;
    let sideways = arrow(
        stage,
        plane().p(t::ONE),
        plane().p(t::ONE.plus(t::ONE.times_i())),
        Palette::hero(),
        5.0,
    );
    let sideways_name = card(k, stage, "rate i: the arrow, a quarter turn on", -2.05, Palette::hero())?;
    fade_in_all(stage, &[sideways, sideways_name], 0.4)?;
    k.until_sentence(stage, "13_2", 8)?;
    let question = k.text(
        stage,
        "Where will the output travel?",
        28.0,
        Palette::hero(),
        v(CARD_X, -2.8),
    )?;
    fade_in_all(stage, &[question], 0.5)?;
    k.hold(stage)?;
    k.mark(stage, "predict");
    k.music_breath(stage, THINK_13_2)?;

    cards.extend(out);
    cards.extend(inward);
    cards.extend([sideways_name, question, sideways, position, start]);
    Ok(cards)
}

/// 13_3: a sideways push keeps the distance, in a picture and in coordinates.
fn distance_never_changes(k: &Kit, stage: &mut Stage<'_>, last: &[Mob]) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "13_3")?;
    fade_out_all(stage, last, 0.4)?;
    let mut field = Vec::new();
    for r in [0.55, 1.0, 1.45] {
        for j in 0..8 {
            let z = t::on_circle(f64::from(j) * PI / 4.0).scale(r);
            field.push(arrow(
                stage,
                plane().p(z),
                plane().p(z.plus(z.times_i().scale(0.45))),
                Palette::faint(),
                2.5,
            ));
        }
    }
    fade_in_lagged(stage, &field, 1.0, 0.03)?;

    k.until_sentence(stage, "13_3", 1)?;
    let clock = stage.add_value_tracker(0.0);
    let live_parts = [
        live_position(stage, clock)?,
        live_velocity(stage, clock)?,
        live_dot(stage, clock)?,
    ];
    walk(stage, clock, 1.4, 3.0)?;

    k.until_sentence(stage, "13_3", 3)?;
    let coordinates = line_of_math(k, stage, COORDINATES, 33.0, Palette::ink(), CARD_X, 2.3)?;
    fade_in_all(stage, &[coordinates], 0.4)?;
    k.until_sentence(stage, "13_3", 4)?;
    let components = line_of_math(k, stage, COMPONENTS, 33.0, Palette::ink(), CARD_X, 1.45)?;
    fade_in_all(stage, &[components], 0.4)?;
    k.until_sentence(stage, "13_3", 5)?;
    let no_change = line_of_math(k, stage, NO_CHANGE, 27.0, Palette::green(), CARD_X, 0.5)?;
    fade_in_all(stage, &[no_change], 0.5)?;
    k.until_sentence(stage, "13_3", 7)?;
    let stays = line_of_math(k, stage, STAYS, 32.0, Palette::hero(), CARD_X, -0.45)?;
    fade_in_all(stage, &[stays], 0.4)?;
    k.until_sentence(stage, "13_3", 8)?;
    let speed = line_of_math(k, stage, SPEED, 32.0, Palette::hero(), CARD_X, -1.3)?;
    fade_in_all(stage, &[speed], 0.4)?;
    k.hold(stage)?;

    let mut shown = field;
    shown.extend(live_parts);
    shown.extend([coordinates, components, no_change, stays, speed]);
    Ok(shown)
}

/// 13_4: speed one on the unit circle is arc length, so the point sits at angle t. The cosine and sine are the
/// coordinates of that point, and that is the whole proof.
fn the_walk(k: &Kit, stage: &mut Stage<'_>, last: &[Mob]) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "13_4")?;
    fade_out_all(stage, last, 0.4)?;
    let circle = plane().unit_circle(stage);
    let start = point(t::ONE, Palette::gold(), stage);
    let start_name = name_at(k, stage, "1", t::ONE, 0.2, -0.27)?;
    fade_in_all(stage, &[circle, start, start_name], 0.5)?;
    let clock = stage.add_value_tracker(0.0);
    let arc = live_arc(stage, clock)?;
    let tip = live_dot(stage, clock)?;
    let radius = live_position(stage, clock)?;

    k.until_sentence(stage, "13_4", 1)?;
    walk(stage, clock, 1.1, 2.0)?;
    k.until_sentence(stage, "13_4", 2)?;
    let name = line_of_math(
        k,
        stage,
        r"\text{distance walked}=t=",
        30.0,
        Palette::gold(),
        CARD_X - 0.55,
        2.3,
    )?;
    let reading = k.live_value(
        stage,
        move |s| s.tracker_value(clock).unwrap_or(0.0),
        2,
        false,
        30.0,
        Palette::gold(),
        v(CARD_X + 1.05, 2.3),
        LEFT,
    )?;
    fade_in_all(stage, &[name], 0.4)?;
    k.until_sentence(stage, "13_4", 3)?;
    let radians = card(
        k,
        stage,
        "distance along the circle is the angle in radians",
        1.5,
        Palette::ink(),
    )?;
    fade_in_all(stage, &[radians], 0.4)?;
    walk(stage, clock, WALKED, 1.8)?;

    k.until_sentence(stage, "13_4", 4)?;
    let at = t::on_circle(WALKED);
    let foot = C::new(at.re, 0.0);
    let legs = [
        seg(stage, plane().p(ZERO), plane().p(foot), Palette::blue(), 5.0),
        seg(stage, plane().p(foot), plane().p(at), Palette::teal(), 5.0),
        name_at(k, stage, r"\cos t", C::new(at.re / 2.0, 0.0), 0.0, -0.3)?,
        name_at(k, stage, r"\sin t", C::new(at.re, at.im / 2.0), -1.1, 0.0)?,
    ];
    fade_in_all(stage, &legs, 0.5)?;

    k.until_sentence(stage, "13_4", 5)?;
    let euler = line_of_math(k, stage, EULER, 40.0, Palette::hero(), CARD_X, 0.25)?;
    fade_in_all(stage, &[euler], 0.6)?;
    k.until_sentence(stage, "13_4", 7)?;
    let sources = [
        k.text(stage, "from the series", 23.0, Palette::ink(), v(CARD_X - 1.4, -0.45))?,
        k.text(stage, "from the circle", 23.0, Palette::ink(), v(CARD_X + 0.75, -0.45))?,
        card(
            k,
            stage,
            "the rate law made them the same point",
            -1.2,
            Palette::green(),
        )?,
    ];
    fade_in_all(stage, &sources, 0.5)?;
    k.until_sentence(stage, "13_4", 8)?;
    let formula_name = card(k, stage, FORMULA_NAME, -1.95, Palette::hero())?;
    fade_in_all(stage, &[formula_name], 0.4)?;
    k.hold(stage)?;

    let mut shown = legs.to_vec();
    shown.extend([
        start,
        start_name,
        arc,
        tip,
        radius,
        name,
        reading,
        radians,
        euler,
        formula_name,
    ]);
    shown.extend(sources);
    Ok(shown)
}

/// 13_5: the two shadows are cosine and sine, and each is the other's slope.
fn two_shadows(k: &Kit, stage: &mut Stage<'_>, last: &[Mob]) -> fmn::Result<()> {
    k.say(stage, "13_5")?;
    fade_out_all(stage, last, 0.4)?;
    let trace = |height: f64, centre: f64, t: f64| v(TRACE_LEFT + TRACE_UNIT * t, centre + TRACE_HEIGHT * height);
    let panels = [
        seg(
            stage,
            trace(0.0, COSINE_Y, 0.0),
            trace(0.0, COSINE_Y, TAU),
            Palette::dim(),
            2.0,
        ),
        seg(
            stage,
            trace(0.0, SINE_Y, 0.0),
            trace(0.0, SINE_Y, TAU),
            Palette::dim(),
            2.0,
        ),
        line_of_math(
            k,
            stage,
            r"x=\cos t",
            25.0,
            Palette::blue(),
            TRACE_LEFT + 0.75,
            COSINE_Y + 0.85,
        )?,
        line_of_math(
            k,
            stage,
            r"y=\sin t",
            25.0,
            Palette::teal(),
            TRACE_LEFT + 0.75,
            SINE_Y + 0.85,
        )?,
    ];
    fade_in_all(stage, &panels, 0.5)?;

    k.until_sentence(stage, "13_5", 1)?;
    let clock = stage.add_value_tracker(0.0);
    let tip = live_dot(stage, clock)?;
    let shadows = [
        live(stage, clock, |s, now| {
            let z = t::on_circle(now);
            s.add(
                Dot::new()
                    .point(plane().p(C::new(z.re, 0.0)))
                    .radius(0.08)
                    .style(flat(Palette::blue(), 1.0))
                    .build(),
            )
        })?,
        live(stage, clock, |s, now| {
            let z = t::on_circle(now);
            s.add(
                Dot::new()
                    .point(plane().p(C::new(0.0, z.im)))
                    .radius(0.08)
                    .style(flat(Palette::teal(), 1.0))
                    .build(),
            )
        })?,
    ];
    let cosine = live(stage, clock, move |s, now| {
        let end = now.max(1e-3);
        let points: Vec<Vec3> = (0..=200)
            .map(|j| {
                let u = end * f64::from(j) / 200.0;
                v(
                    TRACE_LEFT + TRACE_UNIT * u,
                    COSINE_Y + TRACE_HEIGHT * t::on_circle(u).re,
                )
            })
            .collect();
        s.add(polyline(points).style(line_style(Palette::blue(), 4.0)).build())
    })?;
    let sine = live(stage, clock, move |s, now| {
        let end = now.max(1e-3);
        let points: Vec<Vec3> = (0..=200)
            .map(|j| {
                let u = end * f64::from(j) / 200.0;
                v(TRACE_LEFT + TRACE_UNIT * u, SINE_Y + TRACE_HEIGHT * t::on_circle(u).im)
            })
            .collect();
        s.add(polyline(points).style(line_style(Palette::teal(), 4.0)).build())
    })?;
    walk(stage, clock, TAU, TRACE_SECONDS)?;
    stage.clear_updaters(tip, true);
    for m in [shadows[0], shadows[1], cosine, sine] {
        stage.clear_updaters(m, true);
    }

    k.until_sentence(stage, "13_5", 3)?;
    let slopes = line_of_math(k, stage, SLOPES, 29.0, Palette::ink(), CARD_X + 0.1, -0.75)?;
    fade_in_all(stage, &[slopes], 0.5)?;
    k.until_sentence(stage, "13_5", 4)?;
    let same = line_of_math(k, stage, COMPONENTS, 29.0, Palette::dim(), CARD_X + 0.1, -1.3)?;
    fade_in_all(stage, &[same], 0.4)?;
    k.until_sentence(stage, "13_5", 5)?;
    let twice = line_of_math(k, stage, TWICE, 33.0, Palette::hero(), CARD_X + 0.1, -2.0)?;
    fade_in_all(stage, &[twice], 0.5)?;
    k.until_sentence(stage, "13_5", 7)?;
    let spring = k.text(
        stage,
        "ideal mass on a spring, stiffness over mass equal to one",
        21.0,
        Palette::dim(),
        v(CARD_X + 0.1, -2.7),
    )?;
    fade_in_all(stage, &[spring], 0.4)?;
    Ok(())
}

impl SceneConstruct for TurningRate {
    fn name(&self) -> &str {
        "13_turning_rate"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        the_quarter_turn(&k, stage)?;
        let shown = three_rates(&k, stage)?;
        let shown = distance_never_changes(&k, stage, &shown)?;
        let shown = the_walk(&k, stage, &shown)?;
        two_shadows(&k, stage, &shown)?;
        k.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::ending_draw::{assert_in_order, scene_source};
    const SOURCE: &str = include_str!("turning_rate.rs");

    use super::{CORNER, INWARD_OFFSET, THINK_13_2};
    use crate::turning_model as t;

    /// The body of one scene function, from its name to the next function.
    fn body(name: &str, next: &str) -> &'static str {
        scene_source(SOURCE)
            .split(name)
            .nth(1)
            .unwrap()
            .split(next)
            .next()
            .unwrap()
    }

    #[test]
    fn the_circle_is_only_drawn_after_the_question_is_held() {
        assert_in_order(
            SOURCE,
            &[
                r#"k.until_sentence(stage, "13_2", 8)?;"#,
                "Where will the output travel?",
                "k.hold(stage)?;",
                r#"k.mark(stage, "predict");"#,
                "k.music_breath(stage, THINK_13_2)?;",
                r#"k.say(stage, "13_3")?;"#,
                r#"k.say(stage, "13_4")?;"#,
                "plane().unit_circle(stage)",
            ],
        );
        let before = scene_source(SOURCE).split("fn the_walk").next().unwrap();
        assert!(!before.contains(".unit_circle("));
        // Review 721 M10 proposed two to three seconds for this pair; audio-review decides the final length.
        const { assert!(THINK_13_2 >= 2.0) };
        const { assert!(THINK_13_2 <= 3.0) };
    }
    #[test]
    fn multiplying_by_i_is_shown_as_jumps_with_nothing_moving() {
        // Review 721 M3: no tracker, live redraw or circle point inside 13_1 can trace the answer to 13_2.
        let quarter_turn = body("fn the_quarter_turn", "fn three_rates");
        for banned in ["add_value_tracker", "walk(", "live(", "on_circle", "run("] {
            assert!(!quarter_turn.contains(banned), "{banned}");
        }
        assert_in_order(
            SOURCE,
            &[
                r#"k.until_sentence(stage, "13_1", 12)?;"#,
                r#"still_arrow(k, stage, t::ONE, "1""#,
                r#"k.until_sentence(stage, "13_1", 13)?;"#,
                r#"still_arrow(k, stage, quarter, "i""#,
                "right_angle(stage, t::ONE, quarter)",
                r#"k.until_sentence(stage, "13_1", 14)?;"#,
                r#"still_arrow(k, stage, half, "-1""#,
                "right_angle(stage, quarter, half)",
            ],
        );
        const { assert!(CORNER > 0.0) };
    }
    #[test]
    fn the_spring_and_i_come_before_the_letters_and_the_worked_case_before_the_rule() {
        // Review 721 H1 and M4.
        assert_in_order(
            SOURCE,
            &[
                r#"k.until_sentence(stage, "13_1", 2)?;"#,
                "spring_sketch(stage)",
                "SPRING_LAW",
                r#"k.until_sentence(stage, "13_1", 4)?;"#,
                "NEW_NUMBER",
                r#"k.until_sentence(stage, "13_1", 5)?;"#,
                r#"k.until_sentence(stage, "13_1", 6 + j)?;"#,
                r#"k.until_sentence(stage, "13_1", 9)?;"#,
                "COMPLEX_DOES",
                r#"k.until_sentence(stage, "13_1", 10)?;"#,
                "swap(stage, &spring, &[axes], 0.6)?;",
                r#"k.until_sentence(stage, "13_1", 11)?;"#,
                r#""x+iy""#,
                r#"k.until_sentence(stage, "13_1", 15)?;"#,
                "t::EXAMPLE.re",
                r#"k.until_sentence(stage, "13_1", 16)?;"#,
                "right_angle(stage, t::EXAMPLE, turned)",
                r#"k.until_sentence(stage, "13_1", 17)?;"#,
                "GENERAL_RULE",
                "k.hold(stage)?;",
            ],
        );
        assert!(!scene_source(SOURCE).contains("only adds and multiplies"));
        assert!(!scene_source(SOURCE).contains("a+ib"));
        // The worked labels are the spoken case: one and a half plus i, then minus one plus one and a half i.
        let turned = t::EXAMPLE.times_i();
        assert_eq!(format!("{}+i", t::EXAMPLE.re), "1.5+i");
        assert_eq!(format!("{}+{}i", turned.re, turned.im), "-1+1.5i");
        assert_eq!(format!("{}", t::EXAMPLE.im), "1");
    }
    #[test]
    fn the_inward_arrow_runs_beside_the_position_arrow() {
        // Review 721 L5.
        assert!(
            body("fn three_rates", "fn distance_never_changes").contains("arrow(stage, below(t::ONE), below(ZERO)")
        );
        const { assert!(INWARD_OFFSET >= 0.06) };
    }
    #[test]
    fn the_formula_is_named_after_it_is_earned() {
        // Review 721 W8.
        assert_in_order(
            SOURCE,
            &[
                r#"k.until_sentence(stage, "13_4", 7)?;"#,
                "from the series",
                r#"k.until_sentence(stage, "13_4", 8)?;"#,
                "FORMULA_NAME",
                r#"k.say(stage, "13_5")?;"#,
            ],
        );
    }
    #[test]
    fn the_walk_runs_at_constant_speed() {
        let scene = scene_source(SOURCE);
        assert!(!scene.contains("run(stage, clock"));
        assert!(scene.contains("walk(stage, clock, TAU, TRACE_SECONDS)?;"));
    }
    #[test]
    fn the_formula_is_named_after_its_coordinates() {
        assert_in_order(
            SOURCE,
            &[
                r#"k.until_sentence(stage, "13_4", 4)?;"#,
                r"\cos t",
                r#"k.until_sentence(stage, "13_4", 5)?;"#,
                "EULER",
                r#"k.until_sentence(stage, "13_4", 7)?;"#,
                "from the series",
            ],
        );
    }
    #[test]
    fn the_proof_cards_follow_the_voice() {
        assert_in_order(
            SOURCE,
            &[
                r#"k.until_sentence(stage, "13_3", 3)?;"#,
                "COORDINATES",
                r#"k.until_sentence(stage, "13_3", 4)?;"#,
                "COMPONENTS",
                r#"k.until_sentence(stage, "13_3", 5)?;"#,
                "NO_CHANGE",
                r#"k.until_sentence(stage, "13_3", 7)?;"#,
                "STAYS",
                r#"k.until_sentence(stage, "13_3", 8)?;"#,
                "SPEED",
            ],
        );
    }
}
