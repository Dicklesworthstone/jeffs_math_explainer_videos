//! Chapter 08, the product rule keeps a lap count: z at three quarters of a
//! half turn, squared, lands at 3π/2, and the viewer predicts the angle
//! Log reports for z² (08_1); not 3π/2 but −π/2, one lap lower, shown on an
//! angle line in units of π (08_2); the slide rule's dropped decade, the
//! surviving rule Log(zw) = Log z + Log w + 2πik, and k = −1 for this square
//! (08_3); the −1 = 1 puzzle with a prediction (08_4) and the broken step
//! explained by the lost lap (08_5).
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`; uncompiled,
//! unrendered, unlistened.

use std::f64::consts::PI;

use explainer_kit::{Kit, Palette, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    Plane, answer_c, arc_points, arrow, dot, fact, fade_in_all, fade_in_lagged, fade_out_all, flat, lap_c, line_style,
    number_c, position_c, precisely, predict, seg, swap, title, value, warn_c,
};
use crate::model::C;

pub struct ProductRule {
    pub kit: Kit,
}

const Z: Plane = Plane {
    centre: [-4.2, 0.1, 0.0],
    scale: 1.55,
    half_w: 2.2,
    half_h: 2.2,
};
/// The angle line: units of π, −2 to 2, window (−1, 1] marked.
const LINE_Y: f64 = -2.35;
const LINE_X: f64 = -3.6;
const PER_PI: f64 = 1.55;
/// How far right of the vertical axis the label of z² sits, scene units.
const ZZ_ASIDE: f64 = 0.36;
/// Where the definition of √z is centred on the vertical axis, plane units:
/// above the axis's top end, so the axis does not run through it.
const DEF_HEIGHT: f64 = 1.66;

fn angle_x(units: f64) -> f64 {
    LINE_X + PER_PI * (units + 2.0)
}

/// A fact like "3/4", "-1/2" or "2" as a number.
fn ratio(s: &str) -> f64 {
    match s.split_once('/') {
        Some((p, q)) => p.trim().parse::<f64>().unwrap_or(f64::NAN) / q.trim().parse::<f64>().unwrap_or(f64::NAN),
        None => s.trim().parse::<f64>().unwrap_or(f64::NAN),
    }
}

/// The same fact as TeX for that multiple of π: "3/4" → \tfrac{3}{4}\pi.
fn pi_tex(s: &str) -> String {
    let (neg, body) = match s.trim().strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", s.trim()),
    };
    match body.split_once('/') {
        Some((p, q)) => format!(r"{neg}\tfrac{{{p}}}{{{q}}}\pi"),
        None if body == "1" => format!(r"{neg}\pi"),
        None => format!(r"{neg}{body}\pi"),
    }
}

/// The lap count k in Log(z²) = Log z + Log z + 2πi·k, from the angles in
/// units of π: k = (principal − sum)/2, which must be whole.
fn lap_count(sum: f64, principal: f64) -> fmn::Result<i64> {
    let k = (principal - sum) / 2.0;
    if !k.is_finite() || (k - k.round()).abs() > 1e-12 {
        return Err(super::draw::fail(format!("product: lap count {k} is not whole")));
    }
    Ok(k.round() as i64)
}

/// 08_3's worked instance as screen text: "for z²: k = −1, one lap down".
fn k_line(k: i64) -> String {
    let laps = if k.abs() == 1 {
        "one lap".to_string()
    } else {
        format!("{} laps", k.abs())
    };
    let way = if k < 0 { "down" } else { "up" };
    format!("for z²: k = {}, {laps} {way}", super::draw::shown(&k.to_string()))
}

impl SceneConstruct for ProductRule {
    fn name(&self) -> &str {
        "08_product_rule"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (green, blue, gold, purple, red) = (number_c(), position_c(), answer_c(), lap_c(), warn_c());
        let (a_s, sum_s, p_s, lap_s) = (
            fact("product.angle_pi")?,
            fact("product.sum_pi")?,
            fact("product.principal_pi")?,
            fact("product.lap_pi")?,
        );
        let (a, sum, principal) = (ratio(a_s), ratio(sum_s), ratio(p_s));
        let right = 0.6;

        // 08_1: does Log of a product survive the cut? z at 3π/4, squared
        kit.say(stage, "08_1")?;
        let heading = title(&kit, stage, "Does the product rule survive?")?;
        let real_rule = kit.tex(
            stage,
            r"\ln(ab) = \ln a + \ln b \quad (a, b > 0)",
            32.0,
            Palette::ink(),
            &[],
            v(3.6, 2.4),
        )?;
        fade_in_all(stage, &[heading, real_rule], 0.6)?;
        kit.until_sentence(stage, "08_1", 1)?;
        let axes = Z.axes(stage)?;
        let circle = Z.unit_circle(stage);
        let cut = seg(stage, Z.centre, Z.p(C::new(-Z.half_w / Z.scale, 0.0)), red, 5.0);
        fade_in_all(stage, &[axes, circle, cut], 0.5)?;
        kit.until_sentence(stage, "08_1", 2)?;
        let z = C::polar(1.0, a * PI);
        let z_arrow = arrow(stage, Z.centre, Z.p(z), green, 4.0)?;
        let z_word = kit.tex(
            stage,
            &format!(r"z: \ {}", pi_tex(a_s)),
            28.0,
            green,
            &[],
            Z.p(z.scale(1.35)),
        )?;
        let z_arc = super::draw::path(stage, arc_points(Z.centre, 0.45, 0.0, a * PI, 30), blue, 3.0);
        fade_in_all(stage, &[z_arrow, z_word, z_arc], 0.5)?;
        kit.until_sentence(stage, "08_1", 3)?;
        // squaring: the arrow keeps turning by the same angle again
        let turn = stage.add_value_tracker(a);
        let sweep = stage.always_redraw(move |s: &mut Arena| {
            let t = value(s, turn);
            let g = s.add(VMobject::new());
            let arc = s.add(
                polyline(arc_points(Z.centre, 0.62, a * PI, t * PI, 60))
                    .style(line_style(blue, 3.0))
                    .build(),
            );
            let tip = Z.p(C::polar(1.0, t * PI));
            let d = s.add(Dot::new().point(tip).radius(0.09).style(flat(gold, 1.0)).build());
            let _ = s.attach(g, arc);
            let _ = s.attach(g, d);
            g
        });
        stage.add_to_scene(sweep)?;
        play!(stage; tween(turn, sum, 2.2)?);
        let zz = C::polar(1.0, sum * PI);
        let zz_arrow = arrow(stage, Z.centre, Z.p(zz), gold, 4.0)?;
        // beside the arrow's tip, not on the vertical axis the arrow runs along
        let zz_at = Z.p(zz.scale(1.3));
        let zz_word = kit.tex(stage, r"z^{2}", 30.0, gold, &[], v(zz_at[0] + ZZ_ASIDE, zz_at[1]))?;
        fade_in_all(stage, &[zz_arrow, zz_word], 0.4)?;
        kit.until_sentence(stage, "08_1", 4)?;
        let adds = kit.tex(
            stage,
            &format!(
                r"\text{{length }} 1 \times 1 = 1, \quad \text{{angle }} {} + {} = {}",
                pi_tex(a_s),
                pi_tex(a_s),
                pi_tex(sum_s)
            ),
            28.0,
            Palette::ink(),
            &[],
            v(3.6, 1.55),
        )?;
        fade_in_all(stage, &[adds], 0.5)?;
        kit.until_sentence(stage, "08_1", 5)?;
        // the angle line, in units of π, with the principal window
        let mut line = vec![seg(
            stage,
            v(angle_x(-2.0), LINE_Y),
            v(angle_x(2.0), LINE_Y),
            Palette::faint(),
            2.5,
        )];
        line.push(seg(stage, v(angle_x(-1.0), LINE_Y), v(angle_x(1.0), LINE_Y), gold, 6.0));
        for k in -2..=2 {
            let x = angle_x(f64::from(k));
            line.push(seg(stage, v(x, LINE_Y - 0.1), v(x, LINE_Y + 0.1), Palette::dim(), 2.0));
            let label = match k {
                0 => "0".to_string(),
                1 => "π".to_string(),
                -1 => "−π".to_string(),
                _ => format!("{}π", super::draw::shown(&k.to_string())),
            };
            line.push(kit.text(stage, &label, 20.0, Palette::dim(), v(x, LINE_Y - 0.4))?);
        }
        line.push(ring_open(stage, v(angle_x(-1.0), LINE_Y), gold));
        line.push(dot(stage, v(angle_x(1.0), LINE_Y), 0.08, gold));
        // kept out of `line`: it leaves at 08_2, and a second fade-out with the
        // line at 08_4 would root it again at full strength for one beat
        let window_word = kit.text(stage, "principal window", 20.0, gold, v(angle_x(0.0), LINE_Y + 0.42))?;
        let mut window_shown = line.clone();
        window_shown.push(window_word);
        fade_in_all(stage, &window_shown, 0.6)?;
        let one_a = dot(stage, v(angle_x(a), LINE_Y), 0.1, green);
        let sum_dot = dot(stage, v(angle_x(sum), LINE_Y), 0.11, red);
        fade_in_all(stage, &[one_a], 0.3)?;
        fade_in_all(stage, &[sum_dot], 0.4)?;
        kit.until_sentence(stage, "08_1", 6)?;
        // prediction: which angle does the principal logarithm report?
        let ask = kit.tex(
            stage,
            r"\operatorname{Log}(z^{2}) = \ ?\; i",
            30.0,
            gold,
            &[],
            v(3.6, 1.08),
        )?;
        fade_in_all(stage, &[ask], 0.4)?;
        predict(&kit, stage, v(3.6, 0.6), 3.0)?;

        // 08_2: not 3π/2, outside the window; Log reports −π/2; one lap apart
        kit.say(stage, "08_2")?;
        let outside = kit.text(stage, "outside", 20.0, red, v(angle_x(sum), LINE_Y + 0.42))?;
        fade_in_all(stage, &[outside], 0.4)?;
        kit.until_sentence(stage, "08_2", 1)?;
        let p_dot = dot(stage, v(angle_x(principal), LINE_Y), 0.11, gold);
        let same = kit.tex(
            stage,
            &format!(r"\text{{same direction as }} {}", pi_tex(p_s)),
            26.0,
            gold,
            &[],
            v(3.6, 1.08),
        )?;
        swap(stage, &[ask], &[p_dot, same], 0.5)?;
        kit.until_sentence(stage, "08_2", 2)?;
        let compare = kit.tex(
            stage,
            &format!(
                r"\operatorname{{Log}} z + \operatorname{{Log}} z = {}\,i, \qquad \operatorname{{Log}}(z^{{2}}) = {}\,i",
                pi_tex(sum_s),
                pi_tex(p_s)
            ),
            28.0,
            Palette::ink(),
            &[],
            v(3.6, 0.6),
        )?;
        fade_in_all(stage, &[compare], 0.5)?;
        kit.until_sentence(stage, "08_2", 3)?;
        let lap_arrow = arrow(
            stage,
            v(angle_x(sum), LINE_Y + 0.85),
            v(angle_x(principal), LINE_Y + 0.85),
            purple,
            3.5,
        )?;
        let lap_word = kit.tex(
            stage,
            &format!(r"{}\,i: \text{{one lap}}", pi_tex(lap_s)),
            26.0,
            purple,
            &[],
            v(angle_x((sum + principal) / 2.0), LINE_Y + 1.2),
        )?;
        fade_out_all(stage, &[window_word], 0.2)?;
        fade_in_all(stage, &[lap_arrow, lap_word], 0.5)?;

        // 08_3: the slide rule's decimal point again; the surviving rule
        kit.say(stage, "08_3")?;
        fade_out_all(stage, &[real_rule, adds, same], 0.4)?;
        kit.until_sentence(stage, "08_3", 1)?;
        let rule_a = kit.text_left(
            stage,
            "slide rule: keeps the digits, drops a factor of 10",
            22.0,
            Palette::ink(),
            v(right, 2.4),
        )?;
        let rule_b = kit.text_left(
            stage,
            "Log: keeps the direction, drops a lap of 2πi",
            22.0,
            purple,
            v(right, 1.95),
        )?;
        fade_in_lagged(stage, &[rule_a, rule_b], 1.0, 0.5)?;
        kit.until_sentence(stage, "08_3", 2)?;
        let survives = kit.tex(
            stage,
            r"\operatorname{Log}(zw) = \operatorname{Log} z + \operatorname{Log} w + 2\pi i\,k",
            34.0,
            gold,
            &[(r"2\pi i\,k", purple)],
            v(3.6, 1.2),
        )?;
        fade_in_all(stage, &[survives], 0.6)?;
        let which_k = precisely(&kit, stage, "for nonzero z, w on the principal branch, k is −1, 0 or 1")?;
        fade_in_all(stage, &[which_k], 0.4)?;
        kit.until_sentence(stage, "08_3", 4)?;
        let keeps = kit.text_left(stage, "the rule keeps a lap count, k", 22.0, purple, v(right, -0.2))?;
        fade_in_all(stage, &[keeps], 0.4)?;
        kit.until_sentence(stage, "08_3", 5)?;
        // our square: Log(z²) = Log z + Log z + 2πi·k, so k = (principal − sum)/2
        let ours = kit.text_left(
            stage,
            &k_line(lap_count(sum, principal)?),
            22.0,
            purple,
            v(right, -0.62),
        )?;
        fade_in_all(stage, &[ours], 0.4)?;

        // 08_4: the puzzle: −1 = i·i = √−1·√−1 = √((−1)(−1)) = √1 = 1
        kit.say(stage, "08_4")?;
        let mut away = line.clone();
        away.extend([
            one_a, sum_dot, p_dot, outside, compare, lap_arrow, lap_word, rule_a, rule_b, keeps, ours,
        ]);
        away.extend([sweep, zz_arrow, zz_word, z_arrow, z_word, z_arc, survives, which_k]);
        fade_out_all(stage, &away, 0.5)?;
        let heading2 = title(&kit, stage, "A famous puzzle")?;
        swap(stage, &[heading], &[heading2], 0.4)?;
        let steps_src = [
            r"-1 = i \cdot i",
            r"= \sqrt{-1}\,\sqrt{-1}",
            r"= \sqrt{(-1)(-1)}",
            r"= \sqrt{1} = 1",
        ];
        let mut steps = Vec::new();
        for (k, src) in steps_src.iter().enumerate() {
            // the "=" signs line up under the first line's "="
            let x = if k == 0 { right } else { right + 0.78 };
            steps.push(kit.tex_left(stage, src, 34.0, Palette::ink(), &[], v(x, 1.9 - 0.85 * k as f64))?);
        }
        for (k, &m) in steps.iter().enumerate() {
            kit.until_sentence(stage, "08_4", k + 1)?;
            fade_in_all(stage, &[m], 0.5)?;
        }
        kit.until_sentence(stage, "08_4", 5)?;
        let absurd = kit.tex(stage, r"\text{so } -1 = 1\ ?", 36.0, red, &[], v(right + 2.0, -1.65))?;
        fade_in_all(stage, &[absurd], 0.4)?;
        kit.until_sentence(stage, "08_4", 6)?;
        predict(&kit, stage, v(right + 2.5, -2.75), 3.0)?;

        // 08_5: the combining step; square roots as half logarithms
        kit.say(stage, "08_5")?;
        let broken = ring_box(stage, v(right + 2.2, 1.9 - 0.85 * 2.0), 4.6, 0.62, red)?;
        fade_in_all(stage, &[broken], 0.4)?;
        kit.until_sentence(stage, "08_5", 1)?;
        let def = kit.tex(
            stage,
            r"\sqrt{z} = e^{\frac{1}{2}\operatorname{Log} z}",
            30.0,
            blue,
            &[],
            Z.p(C::new(0.0, DEF_HEIGHT)),
        )?;
        fade_in_all(stage, &[def], 0.5)?;
        kit.until_sentence(stage, "08_5", 2)?;
        // each root of −1 turns a quarter; together a half turn: −1
        let half = stage.add_value_tracker(0.0);
        let roots = stage.always_redraw(move |s: &mut Arena| {
            let t = value(s, half);
            let g = s.add(VMobject::new());
            let first = t.min(0.5);
            let a1 = s.add(
                polyline(arc_points(Z.centre, 0.5, 0.0, first * PI, 30))
                    .style(line_style(green, 4.0))
                    .build(),
            );
            let _ = s.attach(g, a1);
            if t > 0.5 {
                let a2 = s.add(
                    polyline(arc_points(Z.centre, 0.68, 0.5 * PI, t * PI, 30))
                        .style(line_style(green, 4.0))
                        .build(),
                );
                let _ = s.attach(g, a2);
            }
            let d = s.add(
                Dot::new()
                    .point(Z.p(C::polar(1.0, t * PI)))
                    .radius(0.1)
                    .style(flat(green, 1.0))
                    .build(),
            );
            let _ = s.attach(g, d);
            g
        });
        stage.add_to_scene(roots)?;
        play!(stage; tween(half, 1.0, 2.4)?);
        let product = kit.tex(
            stage,
            &format!(
                r"\sqrt{{-1}}\,\sqrt{{-1}} = e^{{i\pi/2}} e^{{i\pi/2}} = e^{{i\pi}} = {}",
                fact("sqrt.product_of_roots")?
            ),
            26.0,
            green,
            &[],
            v(right + 3.0, -1.2),
        )?;
        fade_out_all(stage, &[absurd], 0.2)?;
        fade_in_all(stage, &[product], 0.5)?;
        kit.until_sentence(stage, "08_5", 3)?;
        let root1 = kit.tex(
            stage,
            &format!(
                r"\sqrt{{1}} = e^{{\frac{{1}}{{2}} \operatorname{{Log}} 1}} = e^{{0}} = {}",
                fact("sqrt.root_of_product")?
            ),
            26.0,
            gold,
            &[],
            v(right + 3.0, -1.7),
        )?;
        let not_two = kit.tex(
            stage,
            r"\operatorname{Log} 1 = 0,\ \text{not } 2\pi i",
            24.0,
            purple,
            &[],
            v(right + 3.0, -2.3),
        )?;
        // the product's root has its own place on the ring, opposite the two roots' product
        let one_dot = dot(stage, Z.p(C::new(1.0, 0.0)), DOT_R, gold);
        fade_in_all(stage, &[root1, not_two, one_dot], 0.5)?;
        kit.until_sentence(stage, "08_5", 4)?;
        // "Taking half of the lost lap": the lap, halved, and no sign yet
        let halved = kit.tex(stage, HALVED, 26.0, purple, &[], Z.p(C::new(0.0, -1.55)))?;
        let halving = halved_part(&kit, stage, halved)?;
        fade_in_all(stage, &[halving], 0.5)?;
        // "leaves a half turn": the turn itself, from the root of the product, +1, the
        // way e^{πi} goes, round to the product of the roots, −1
        until_phrase(&kit, stage, "08_5", HALF_TURN_PHRASE)?;
        let half_turn = stage.add_value_tracker(0.0);
        let half_arc = stage.always_redraw(move |s: &mut Arena| {
            let t = value(s, half_turn);
            let g = s.add(VMobject::new());
            if t > 1e-3 {
                for pts in [half_turn_arc(t), half_turn_head(t)] {
                    let m = s.add(polyline(pts).style(line_style(purple, HALF_TURN_W)).build());
                    let _ = s.attach(g, m);
                }
            }
            g
        });
        stage.add_to_scene(half_arc)?;
        play!(stage; tween(half_turn, 1.0, HALF_TURN_S)?);
        let half_word = half_turn_word(&kit, stage)?;
        fade_in_all(stage, &[half_word], 0.3)?;
        // "a factor of minus one": now the sign
        until_phrase(&kit, stage, "08_5", FACTOR_PHRASE)?;
        swap(stage, &[halving], &[halved], 0.3)?;
        kit.until_sentence(stage, "08_5", 5)?;
        let only_sign = kit.tex(
            stage,
            r"\sqrt{zw} = \pm \sqrt{z}\,\sqrt{w}",
            34.0,
            gold,
            &[],
            v(right + 3.0, -3.0),
        )?;
        fade_in_all(stage, &[only_sign], 0.5)?;
        kit.finish(stage)
    }
}

/// An open endpoint: a small ring for "not included".
fn ring_open(stage: &mut Stage<'_>, at: Vec3, c: Srgb) -> Mob {
    super::draw::ring(stage, at, 0.08, c, 2.5)
}

/// A rectangle outline around a line of the puzzle.
fn ring_box(stage: &mut Stage<'_>, centre: Vec3, w: f64, h: f64, c: Srgb) -> fmn::Result<Mob> {
    let r = Rectangle::new()
        .width(w)
        .height(h)
        .style(line_style(c, 3.0))
        .build()?
        .moved_to(centre);
    Ok(stage.arena_mut().add(r))
}

/// 08_5's purple line whole, and its first stage: the lost lap halved, before the sign.
const HALVED: &str = r"\tfrac{1}{2}(2\pi i) = \pi i, \quad e^{\pi i} = -1";
const HALVED_LAP: &str = r"\tfrac{1}{2}(2\pi i) = \pi i";

/// The first stage of 08_5's purple line, on the whole line's lower left corner, so
/// that what the two share stays put when the whole line replaces it.
fn halved_part(kit: &Kit, stage: &mut Stage<'_>, whole: Mob) -> fmn::Result<Mob> {
    let corner = stage.get_corner(whole, DL);
    let m = kit.tex(stage, HALVED_LAP, 26.0, lap_c(), &[], corner)?;
    stage.move_to(m, corner, DL);
    Ok(m)
}

/// The dots at +1 and −1 on 08_5's ring; the half turn runs between them.
const DOT_R: f64 = 0.1;
/// The half turn's stroke, how long it takes, how far short of each dot's centre
/// it stops, in radians, and the length of its arrow head.
const HALF_TURN_W: f64 = 5.0;
const HALF_TURN_S: f64 = 1.6;
const HALF_TURN_GAP: f64 = (DOT_R + 0.07) / Z.scale;
const HALF_TURN_HEAD: f64 = 0.16;

/// The angle the half turn has reached, a fraction `t` of the way from +1 to −1.
fn half_turn_end(t: f64) -> f64 {
    HALF_TURN_GAP + (PI - 2.0 * HALF_TURN_GAP) * t.clamp(0.0, 1.0)
}

/// The half turn so far, along the unit circle through the top of the ring.
fn half_turn_arc(t: f64) -> Vec<Vec3> {
    arc_points(Z.centre, Z.scale, HALF_TURN_GAP, half_turn_end(t), 60)
}

/// The arrow head at the half turn's moving end. It grows with the arc, so it never
/// reaches back over the dot at +1.
fn half_turn_head(t: f64) -> Vec<Vec3> {
    let end = half_turn_end(t);
    let reach = HALF_TURN_HEAD.min(Z.scale * (end - HALF_TURN_GAP));
    let tip = Z.p(C::polar(1.0, end));
    let (back, out) = ([end.sin(), -end.cos()], [end.cos(), end.sin()]);
    let wing = |side: f64| {
        v(
            tip[0] + reach * (0.87 * back[0] + 0.5 * side * out[0]),
            tip[1] + reach * (0.87 * back[1] + 0.5 * side * out[1]),
        )
    };
    vec![wing(1.0), tip, wing(-1.0)]
}

/// The half turn's name, outside the ring at its upper right.
fn half_turn_word(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    kit.text_left(stage, "half turn", 20.0, lap_c(), v(-2.95, 1.45))
}

/// The words of 08_5's fifth sentence that bring the half turn and its sign.
const HALF_TURN_PHRASE: &str = "leaves a half turn";
const FACTOR_PHRASE: &str = "a factor of minus one";
const SCRIPT: &str = include_str!("../../script.tsv");

/// When `phrase` starts, in seconds into its line. A measured word mark at its byte
/// wins. Without one this is the phrase's character share of its measured sentence:
/// provisional, and never a heard boundary.
fn phrase_seconds(
    text: &str,
    phrase: &str,
    marks: &std::collections::HashMap<usize, f64>,
    starts: &[f64],
    len: f64,
) -> Result<f64, String> {
    let byte = text
        .find(phrase)
        .ok_or_else(|| format!("the line has no phrase {phrase:?}"))?;
    if let Some(&seconds) = marks.get(&byte) {
        return Ok(seconds);
    }
    let mut opens = vec![0];
    for (i, pair) in text.as_bytes().windows(2).enumerate() {
        if matches!(pair[0], b'.' | b'?' | b'!') && pair[1] == b' ' {
            opens.push(i + 2);
        }
    }
    if opens.len() != starts.len() {
        return Err(format!(
            "the line has {} sentences, but the narrator timed {}",
            opens.len(),
            starts.len()
        ));
    }
    let j = opens.iter().rposition(|&open| open <= byte).unwrap_or(0);
    let close = opens.get(j + 1).copied().unwrap_or(text.len());
    let (t0, t1) = (starts[j], starts.get(j + 1).copied().unwrap_or(len));
    let before = text[opens[j]..byte].chars().count() as f64;
    let whole = text[opens[j]..close].trim_end().chars().count().max(1) as f64;
    Ok(t0 + (t1 - t0) * before / whole)
}

fn phrase_time(kit: &Kit, id: &str, phrase: &str) -> fmn::Result<Option<f64>> {
    let Some(n) = kit.narrator.as_ref() else {
        return Ok(None);
    };
    let starts = n.sentence_starts(id);
    let (Some(&first), Some(said), Some(len)) = (starts.first(), n.sentence_at(id, 0), n.length(id)) else {
        return Ok(None);
    };
    let script = explainer_kit::Script::parse(SCRIPT).map_err(super::draw::fail)?;
    let text = script
        .text(id)
        .ok_or_else(|| super::draw::fail(format!("narration line {id} is not in script.tsv")))?;
    let seconds = phrase_seconds(text, phrase, &n.phrase_marks(id), &starts, len)
        .map_err(|e| super::draw::fail(format!("line {id}: {e}")))?;
    Ok(Some(said - first + seconds))
}

fn until_phrase(kit: &Kit, stage: &mut Stage<'_>, id: &str, phrase: &str) -> fmn::Result<()> {
    let Some(at) = phrase_time(kit, id, phrase)? else {
        return Ok(());
    };
    let (now, fps) = (stage.scene().time().to_f64(), stage.scene().time().fps());
    if at > now {
        explainer_kit::narration::wait_frames(stage, explainer_kit::narration::frames_covering(at - now, fps))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::*;

    /// Run `f` as a scene's `construct`, headless, at 10 fps.
    fn run(f: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) {
        struct Probe<F>(F);
        impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                (self.0)(stage)
            }
        }
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut Probe(f), config, 7, &mut NullSceneSink).expect("the probe scene runs");
    }

    /// A mobject's typeset box: left, right, bottom, top.
    fn box_of(stage: &Stage<'_>, m: Mob) -> [f64; 4] {
        [
            stage.get_bounding_box_point(m, LEFT)[0],
            stage.get_bounding_box_point(m, RIGHT)[0],
            stage.get_bounding_box_point(m, DOWN)[1],
            stage.get_bounding_box_point(m, UP)[1],
        ]
    }

    /// The film's source, without its tests.
    fn scene_source() -> &'static str {
        include_str!("product_rule.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap_or("")
    }

    fn found(text: &str) -> usize {
        scene_source()
            .find(text)
            .unwrap_or_else(|| panic!("not in the scene: {text}"))
    }

    #[test]
    fn the_facts_read_as_the_angles_drawn() {
        assert_eq!(ratio("3/4"), 0.75);
        assert_eq!(ratio("-1/2"), -0.5);
        assert_eq!(ratio("2"), 2.0);
        assert_eq!(pi_tex("3/4"), r"\tfrac{3}{4}\pi");
        assert_eq!(pi_tex("-1/2"), r"-\tfrac{1}{2}\pi");
        assert_eq!(pi_tex("2"), r"2\pi");
    }

    #[test]
    fn squaring_three_quarters_lands_a_lap_above_the_principal_value() {
        let z = C::polar(1.0, 0.75 * PI);
        let zz = z.mul(z);
        assert!((zz.log().im - (-0.5 * PI)).abs() < 1e-12);
        let sum = z.log().im + z.log().im;
        assert!((sum - 1.5 * PI).abs() < 1e-12);
        assert!(((sum - zz.log().im) - 2.0 * PI).abs() < 1e-12);
    }

    #[test]
    fn the_square_drops_exactly_one_lap_so_k_is_minus_one() {
        let sum = ratio(fact("product.sum_pi").unwrap());
        let principal = ratio(fact("product.principal_pi").unwrap());
        assert_eq!(lap_count(sum, principal).unwrap(), -1);
        assert_eq!(k_line(-1), "for z²: k = \u{2212}1, one lap down");
        // the rule with that k gives back the principal value
        let k = f64::from(i32::try_from(lap_count(sum, principal).unwrap()).unwrap());
        assert!((sum + 2.0 * k - principal).abs() < 1e-12);
        // a half lap is not a lap count
        assert!(lap_count(1.5, 0.5).is_err());
        // the line fits in the right-hand column
        let width = k_line(-1).chars().count() as f64 * 0.55 * 22.0 / 144.0;
        assert!(0.6 + width < 6.8);
    }

    #[test]
    fn the_angle_line_and_its_marks_fit() {
        assert!(angle_x(-2.0) > -7.1 && angle_x(2.0) < 7.1);
        assert!(angle_x(1.5) > angle_x(1.0));
    }

    #[test]
    fn the_axis_runs_through_neither_label() {
        // half extents read from preview02 frames 130 and 300, scene units
        let (zz_half_w, def_half_h, title_bottom) = (0.15, 0.19, 3.2);
        assert!(ZZ_ASIDE - zz_half_w > 0.15);
        let def_y = Z.centre[1] + Z.scale * DEF_HEIGHT;
        // clear of the axis's top end below, clear of the title above
        assert!(def_y - def_half_h > Z.centre[1] + Z.half_h + 0.15);
        assert!(def_y + def_half_h < title_bottom - 0.2);
    }

    #[test]
    fn principal_roots_of_minus_one_multiply_to_minus_one() {
        let r = C::new(-1.0, 0.0).sqrt();
        let p = r.mul(r);
        assert!((p.re + 1.0).abs() < 1e-12 && p.im.abs() < 1e-12);
        let s = C::new(1.0, 0.0).sqrt();
        assert!((s.re - 1.0).abs() < 1e-12);
    }

    #[test]
    fn the_half_turn_and_its_sign_wait_for_their_words_and_the_whole_line_stays() {
        let script = explainer_kit::Script::parse(SCRIPT).unwrap();
        let text = script.text("08_5").expect("08_5 is in the script");
        // once each, starting a word, in spoken order, both inside "Taking half ..."
        let open = text.find("Taking half of the lost lap").expect("08_5 has the halving");
        let close = text.find("The root of a product").expect("08_5 has its last sentence");
        let mut after = open;
        for phrase in [HALF_TURN_PHRASE, FACTOR_PHRASE] {
            assert_eq!(text.matches(phrase).count(), 1, "{phrase}");
            let byte = text.find(phrase).unwrap();
            assert!(byte > after && byte < close, "{phrase} is out of place");
            assert_eq!(&text[byte - 1..byte], " ", "{phrase} does not start a word");
            after = byte;
        }
        let order = [
            found("kit.until_sentence(stage, \"08_5\", 3)"),
            found("fade_in_all(stage, &[root1, not_two, one_dot], 0.5)"),
            found("kit.until_sentence(stage, \"08_5\", 4)"),
            found("fade_in_all(stage, &[halving], 0.5)"),
            found("until_phrase(&kit, stage, \"08_5\", HALF_TURN_PHRASE)"),
            found("tween(half_turn, 1.0, HALF_TURN_S)"),
            found("fade_in_all(stage, &[half_word], 0.3)"),
            found("until_phrase(&kit, stage, \"08_5\", FACTOR_PHRASE)"),
            found("swap(stage, &[halving], &[halved], 0.3)"),
            found("kit.until_sentence(stage, \"08_5\", 5)"),
            found("kit.finish(stage)"),
        ];
        assert!(order.windows(2).all(|w| w[0] < w[1]), "{order:?}");
        // nothing of the mathematics is lost: the first stage is the whole line's start
        assert!(HALVED.starts_with(HALVED_LAP));
        assert!(HALVED.contains(r"e^{\pi i} = -1") && !HALVED_LAP.contains('-'));
    }

    #[test]
    fn the_halved_lap_stands_on_the_whole_lines_corner_and_holds_no_sign() {
        run(|stage| {
            let kit = Kit::new()?;
            let whole = kit.tex(stage, HALVED, 26.0, lap_c(), &[], Z.p(C::new(0.0, -1.55)))?;
            let part = halved_part(&kit, stage, whole)?;
            let [left, right, bottom, _] = box_of(stage, whole);
            let [l, r, b, _] = box_of(stage, part);
            assert!((l - left).abs() < 1e-6 && (b - bottom).abs() < 1e-6, "{l} {b}");
            assert!(r < right - 0.5, "the first stage reaches where the sign goes");
            Ok(())
        });
    }

    #[test]
    fn the_half_turn_runs_between_the_two_answers_and_touches_nothing_on_the_way() {
        // its ends are the two answers, and e^{πi} carries one to the other
        assert_eq!(fact("sqrt.root_of_product").unwrap(), "1");
        assert_eq!(fact("sqrt.product_of_roots").unwrap(), "-1");
        let turned = C::new(1.0, 0.0).mul(C::new(0.0, PI).exp());
        assert!((turned.re + 1.0).abs() < 1e-12 && turned.im.abs() < 1e-12);
        // it stops the same way short of each dot, having swept all that lies between
        assert!((half_turn_end(0.0) - HALF_TURN_GAP).abs() < 1e-12);
        assert!((half_turn_end(1.0) - (PI - HALF_TURN_GAP)).abs() < 1e-12);
        let (plus, minus) = (Z.p(C::new(1.0, 0.0)), Z.p(C::new(-1.0, 0.0)));
        let cap = HALF_TURN_W * 0.01 / 2.0;
        for k in 0..=50 {
            let t = f64::from(k) / 50.0;
            for p in half_turn_arc(t).iter().chain(half_turn_head(t).iter()) {
                for at in [plus, minus] {
                    let far = (p[0] - at[0]).hypot(p[1] - at[1]);
                    assert!(far - cap > DOT_R + 0.02, "{t}: {p:?} reaches a dot");
                }
                let from_centre = (p[0] - Z.centre[0]).hypot(p[1] - Z.centre[1]);
                assert!(from_centre > 0.68 + 0.2, "{t}: {p:?} reaches the green arcs");
                // through the top only: never on the cut, and under the axis's top end
                assert!(p[1] > Z.centre[1] + 0.05 && p[1] < Z.centre[1] + Z.half_h, "{t}: {p:?}");
            }
        }
    }

    #[test]
    fn the_half_turns_name_clears_the_ring_the_axis_the_definition_and_the_column() {
        run(|stage| {
            let kit = Kit::new()?;
            let word = half_turn_word(&kit, stage)?;
            let [left, right, bottom, top] = box_of(stage, word);
            // the corner nearest the ring, outside the ring and the half turn's stroke
            let near = (left - Z.centre[0]).hypot(bottom - Z.centre[1]);
            assert!(near > Z.scale + 0.1, "the ring reaches the word: {near}");
            assert!(left > Z.centre[0] + 0.1, "the upright axis reaches the word");
            assert!(right < 0.5, "the word reaches the right-hand column");
            assert!(bottom > Z.centre[1] + 0.1, "the level axis reaches the word");
            assert!(
                top < Z.centre[1] + Z.scale * DEF_HEIGHT - 0.3,
                "the word reaches the definition"
            );
            Ok(())
        });
    }

    #[test]
    fn a_phrase_lands_on_its_mark_or_on_its_share_of_the_measured_sentence() {
        let text = "One two. Three four five six. Seven.";
        let starts = [0.2, 2.0, 6.0];
        let none = std::collections::HashMap::new();
        // eleven of the sentence's twenty characters come before "five"
        let t = phrase_seconds(text, "five", &none, &starts, 8.0).unwrap();
        assert!((t - (2.0 + 4.0 * 11.0 / 20.0)).abs() < 1e-12);
        // a measured word mark wins
        let marks = std::collections::HashMap::from([(text.find("five").unwrap(), 3.25)]);
        let t = phrase_seconds(text, "five", &marks, &starts, 8.0).unwrap();
        assert!((t - 3.25).abs() < 1e-12);
        // never a guess
        assert!(phrase_seconds(text, "nine", &none, &starts, 8.0).is_err());
        assert!(phrase_seconds(text, "five", &none, &starts[..2], 8.0).is_err());
    }
}
