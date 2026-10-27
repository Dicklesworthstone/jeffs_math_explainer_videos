//! Chapter 07, straight sticks. From 06's end, the card and coast dimmed
//! and the sheet at the first placement, a straight stick 6 cm long lies
//! across the lines (07_1): it crosses three, so the bet says 3 × π, about
//! 9.4 cm, far too long (07_2). Its count and crossings hidden, it turns to
//! lie along the lines between two of them: what does the bet say now?
//! (07_3, a prediction.) Zero: the same stick gives 9.4 or 0, so for a
//! straight stick the count depends on which way it points, which the coin
//! hid by running every way at once (07_4). What matters is how far it
//! reaches across the lines: its shadow on a ruler at right angles to them,
//! the light running along them, none at all lying along (07_5). Turned
//! slowly back across, in silence, its count climbs 0, 1, 2, 3 as the
//! shadow grows (07_6), to the full 6 cm and three crossings (07_7). The
//! stick, its crossings, its count and its shadow are drawn every frame
//! from one tracker, its direction, through this file's model. Starts
//! where chapter 06 ends; integrated by root from
//! `docs/candidate-06-07.md`.

use std::f64::consts::{FRAC_PI_2, PI};
use std::rc::Rc;

use explainer_kit::{Kit, Palette, TextKind, ink_style, tween, v};
use fmn::core::rate::{linear, smooth};
use fmn::prelude::*;

use super::draw::{
    EVERY, card_panel, card_point, fade_in_all, fade_out_all, fail, flat, now, path, say, stand, think, together,
};
use super::placement::{self, COUNT_RIGHT, CROSSINGS, WORD_GAP, at_once, faded_line, glide, tracked};
use super::sheet::{self, COUNT_SIZE, DOT_R, LABEL_SIZE, LOW_Y, Sheet, THINK_AT};
use crate::facts;
use crate::model::{self, COIN, GAP, P2};
use crate::palette;

pub struct StraightSticks {
    pub kit: Kit,
}

// ---------------------------------------------------------------- the stick

/// The stick's length, cm (`stick.length`).
pub const LENGTH: f64 = 6.0;
/// Where its centre lies on the card, cm, all chapter: over the beach's
/// left end, 0.75 cm above the first placement's middle line. Lying
/// across, its ends keep 0.25 cm from the lines and 0.18 cm above the
/// beach; lying along, it keeps 0.75 cm from the line under it.
pub const HOME: P2 = [-4.5, 1.2];
/// Its direction, radians counterclockwise from the lines: straight across
/// them, and lying along them. It turns between the two through the upper
/// left (its upper end swinging left and down), clear of the count.
pub const ACROSS: f64 = FRAC_PI_2;
pub const ALONG: f64 = PI;

/// The stick's two ends, centred at `c`, at direction `theta`.
pub fn ends(c: P2, theta: f64) -> [P2; 2] {
    let (dx, dy) = (LENGTH / 2.0 * theta.cos(), LENGTH / 2.0 * theta.sin());
    [[c[0] - dx, c[1] - dy], [c[0] + dx, c[1] + dy]]
}

/// Its shadow on a ruler at right angles to the lines (the sheet at
/// direction 0), the light running along them: the heights its two ends
/// reach, the lower first.
pub fn shadow(c: P2, theta: f64) -> [f64; 2] {
    let [a, b] = ends(c, theta);
    [a[1].min(b[1]), a[1].max(b[1])]
}

/// Where the stick crosses the lines of the sheet at direction 0 slid by
/// `slide`: one point on each line strictly inside its shadow, the lowest
/// line first, each with the line's number as [`sheet::crossings`] numbers
/// it. Lying along, its shadow is a single height and it crosses none.
pub fn crossings(c: P2, theta: f64, slide: f64) -> Vec<(i64, P2)> {
    let ([lo, hi], [a, b]) = (shadow(c, theta), ends(c, theta));
    let first = ((lo - slide) / GAP).floor() as i64;
    let last = ((hi - slide) / GAP).ceil() as i64;
    (first..=last)
        .filter_map(|k| {
            let h = slide + k as f64 * GAP;
            (lo < h && h < hi).then(|| {
                let t = (h - a[1]) / (b[1] - a[1]);
                (k, [a[0] + t * (b[0] - a[0]), h])
            })
        })
        .collect()
}

/// How far the stick turns from along the lines before each change of its
/// count, radians, nearest first: an end's height reaches a line d from the
/// centre's when sin φ = d / (LENGTH / 2), for each line nearer than half
/// the stick. (The tests time 07_6's turn by it.)
#[cfg(test)]
pub fn turning_angles(c: P2, slide: f64) -> Vec<f64> {
    let half = LENGTH / 2.0;
    let first = ((c[1] - half - slide) / GAP).floor() as i64;
    let last = ((c[1] + half - slide) / GAP).ceil() as i64;
    let mut out: Vec<f64> = (first..=last)
        .map(|k| (slide + k as f64 * GAP - c[1]).abs())
        .filter(|&d| d > 0.0 && d < half)
        .map(|d| (d / half).asin())
        .collect();
    out.sort_by(f64::total_cmp);
    out
}

/// How near a line the stick may lie along, or reach an end to lying
/// across, cm: no shown count is a near miss.
const CLEAR: f64 = 0.2;

/// What the voice says the stick shows: three crossings across (07_2,
/// 07_7), none along (07_4).
const SAID_ACROSS: usize = 3;
const SAID_ALONG: usize = 0;
/// The steps the turn back across is read at for its counts: a hundredth
/// of a degree each.
const TURN_STEPS: usize = 9000;

/// One crossing's worth on the bet, cm, as 06 has it: π × [`COIN`] / 2.
fn rate() -> f64 {
    PI * COIN / 2.0
}

/// The direction a share `u` of the turn from along to across.
fn turned(u: f64) -> f64 {
    ALONG + (ACROSS - ALONG) * u
}

/// The counts the turn from along to across passes through, in order,
/// read at every hundredth of a degree of it, as the oracle reads its own
/// turn. `facts.rs` prints `stick.turn_counts` from it.
pub fn turn_counts(c: P2, slide: f64) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::new();
    for i in 0..=TURN_STEPS {
        let n = crossings(c, turned(i as f64 / TURN_STEPS as f64), slide).len();
        if out.last() != Some(&n) {
            out.push(n);
        }
    }
    out
}

/// How far the nearest line lies from a height, cm.
fn off_lines(h: f64, slide: f64) -> f64 {
    let t = (h - slide) / GAP;
    (t - t.round()).abs() * GAP
}

/// The chapter's words that carry numbers, each from the facts, refused
/// unless the model gives what the voice says at `home` on the sheet at
/// `slide`: three crossings across, none along, the turn passing 0, 1, 2,
/// 3, the shadow the full length across and none along, no shown count a
/// near miss, and the bet's two estimates as the facts round them.
struct Words {
    length: String,
    rows: [String; 2],
    shadows: [String; 2],
}

fn words(home: P2, slide: f64) -> Result<Words, String> {
    let f = facts::facts()?;
    let get = |key: &str| f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"));
    let fixed = facts::fixed;

    let length = get("stick.length")?;
    if length != fixed(LENGTH, 0) {
        return Err(format!("the stick is drawn {LENGTH} cm long; the facts say {length}"));
    }
    let (across, along) = (
        crossings(home, ACROSS, slide).len(),
        crossings(home, ALONG, slide).len(),
    );
    for (key, said, got) in [
        ("stick.home_across", SAID_ACROSS, across),
        ("stick.home_along", SAID_ALONG, along),
    ] {
        if get(key)? != said.to_string() || got != said {
            return Err(format!(
                "the voice says {said} for {key}; the facts say {}, the model {got}",
                get(key)?
            ));
        }
    }
    let [lo, hi] = shadow(home, ACROSS);
    let near = off_lines(lo, slide).min(off_lines(hi, slide));
    if near < CLEAR || off_lines(home[1], slide) < CLEAR {
        return Err(format!(
            "lying across the stick's ends come {near:.3} cm from a line, lying along it lies {:.3} cm from one",
            off_lines(home[1], slide)
        ));
    }
    let counts = turn_counts(home, slide);
    let said: Vec<usize> = (SAID_ALONG..=SAID_ACROSS).collect();
    let printed = counts.iter().map(usize::to_string).collect::<Vec<_>>().join(",");
    if counts != said || get("stick.turn_counts")? != printed {
        return Err(format!(
            "07_7 says the count climbed to three as the stick turned: the model passes {printed}, the facts say {}",
            get("stick.turn_counts")?
        ));
    }
    let shadows = [ALONG, ACROSS].map(|theta| {
        let [lo, hi] = shadow(home, theta);
        fixed(hi - lo, 0)
    });
    for (key, got) in [
        ("stick.shadow_along", &shadows[0]),
        ("stick.shadow_across", &shadows[1]),
    ] {
        if get(key)? != *got {
            return Err(format!("{key}: the model gives {got}, the facts {}", get(key)?));
        }
    }
    let (worth, est, none) = (
        get("coin.worth")?,
        get("stick.across_estimate")?,
        get("stick.along_estimate")?,
    );
    if worth != fixed(rate(), 2) || est != fixed(across as f64 * rate(), 1) || none != fixed(along as f64 * rate(), 0) {
        return Err(format!(
            "the bet at {across} and {along} crossings is {est} and {none} cm in the facts, a crossing {worth}"
        ));
    }
    Ok(Words {
        length: format!("{length} cm"),
        rows: [
            format!(r"\text{{across}}\quad {across} \times \pi\text{{ cm}} \approx {est}\text{{ cm}}"),
            format!(r"\text{{along}}\quad {along} \times \pi\text{{ cm}} = {none}\text{{ cm}}"),
        ],
        shadows: shadows.map(|s| format!("shadow {s} cm")),
    })
}

// ------------------------------------------------------------------ layout

const COAST_W: f64 = 4.0;
/// What 06 left the card and coast dimmed to.
const DIMMED: f64 = 0.25;
const STICK_W: f64 = 6.0;
const SHADOW_W: f64 = 7.0;
const LIGHT_W: f64 = 1.5;
const LIGHT_OPACITY: f64 = 0.7;
/// The light's dashes and the gaps between them, frame units.
const DASH: f64 = 0.1;
const DASH_GAP: f64 = 0.08;
/// A shadow with no length, drawn as a point on the ruler, frame units.
const POINT_R: f64 = 0.06;
/// The ruler: upright at the card's left, just beyond the lines' left
/// ends, from under the stick's lowest reach to over its highest, a tick
/// at each centimetre from the stick's centre height, cm.
const RULER_X: f64 = -7.87;
const RULER_Y: [f64; 2] = [HOME[1] - LENGTH / 2.0 - 0.2, HOME[1] + LENGTH / 2.0 + 0.2];
const RULER_TICK: f64 = 0.12;
const RULER_W: f64 = 2.5;
/// "6 cm" beside the stick lying across, and the shadow's length over the
/// ruler's top, left-aligned, cm.
const LENGTH_AT: P2 = [HOME[0] + 0.7, HOME[1]];
const SHADOW_AT: P2 = [-7.55, 3.45];
/// 07_2's and 07_4's rows, left-aligned in the band under the coast,
/// frame.
const EQUATION_SIZE: f64 = 28.0;
const ROWS_AT: [Vec3; 2] = [[0.4, LOW_Y + 0.22, 0.0], [0.4, LOW_Y - 0.22, 0.0]];

// ------------------------------------------------------------------ timing

const FADE_S: f64 = 0.4;
const LAY_S: f64 = 3.0;
/// 07_6's turn back across, at a steady pace: the briefest count, one
/// crossing, lasts the 10° from 14.5° to 24.6°, 1.1 s.
const BACK_S: f64 = 10.0;

// --------------------------------------------------------- what is drawn

/// The stick's direction, as its tracker holds it.
fn direction(s: &fmn::mobject::Stage, turn: Mob) -> f64 {
    s.tracker_value(turn).unwrap_or(ACROSS)
}

/// The stick at its direction, at the opacity `alpha` holds. In the scene.
fn live_stick(stage: &mut Stage<'_>, turn: Mob, alpha: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let [a, b] = ends(HOME, direction(s, turn));
        let stick = polyline(vec![card_point(a), card_point(b)])
            .style(faded_line(palette::stick(), STICK_W, tracked(s, alpha)))
            .build();
        s.add(v_group(vec![stick]))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The stick's crossings with the sheet at `slide`, gold dots as
/// [`sheet::crossing_dot`] draws them, at the opacity `alpha` holds. In the
/// scene.
fn live_crossings(stage: &mut Stage<'_>, turn: Mob, slide: f64, alpha: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let o = tracked(s, alpha);
        let dots: Vec<VMobject> = crossings(HOME, direction(s, turn), slide)
            .iter()
            .map(|&(_, at)| {
                Circle::new()
                    .radius(DOT_R)
                    .arc_center(card_point(at))
                    .style(flat(palette::touch(), o))
                    .build()
            })
            .collect();
        s.add(v_group(dots))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// "crossing" for a count of one, "crossings" otherwise.
fn noun(n: usize) -> &'static str {
    if n == 1 { "crossing" } else { CROSSINGS }
}

/// The count's word for `count(s)`, redrawn every frame, its left edge at
/// `at`, at the opacity `alpha` holds, as `placement::live_word` draws a
/// word. In the scene.
fn live_noun(
    kit: &Kit,
    stage: &mut Stage<'_>,
    count: impl Fn(&fmn::mobject::Stage) -> usize + 'static,
    alpha: Mob,
    size: f64,
    at: Vec3,
) -> fmn::Result<Mob> {
    let book = Rc::clone(&kit.book);
    let m = stage.always_redraw(move |s| {
        let (o, word) = (tracked(s, alpha), noun(count(s)));
        let t = Text::new(word)
            .font_size(size)
            .style(ink_style(palette::touch()))
            .build(&book)
            .expect("the bundled face sets every word on screen");
        explainer_kit::glyphs::record(TextKind::Text, word);
        let m = s.add(t);
        s.move_to(m, at, LEFT);
        s.set_family_opacity(m, o);
        m
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// Dashes along the frame segment from `a` to `b`.
fn dashes(a: Vec3, b: Vec3) -> Vec<Vec<Vec3>> {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let length = dx.hypot(dy);
    let mut out = Vec::new();
    let mut from = 0.0;
    while from < length {
        let to = (from + DASH).min(length);
        let at = |d: f64| v(a[0] + dx * d / length, a[1] + dy * d / length);
        out.push(vec![at(from), at(to)]);
        from = to + DASH_GAP;
    }
    out
}

/// The stick's shadow on the ruler, a bar from the lower end's height to
/// the upper's (a point where they are level), and the light from each end
/// along the lines to the ruler, dashed, at the opacities `alpha` and
/// `light` hold. In the scene.
fn live_shadow(stage: &mut Stage<'_>, turn: Mob, alpha: Mob, light: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let (o, l, theta) = (tracked(s, alpha), tracked(s, light), direction(s, turn));
        let [lo, hi] = shadow(HOME, theta);
        let mut parts: Vec<VMobject> = Vec::new();
        for end in ends(HOME, theta) {
            for pts in dashes(card_point(end), card_point([RULER_X, end[1]])) {
                parts.push(
                    polyline(pts)
                        .style(faded_line(Palette::dim(), LIGHT_W, LIGHT_OPACITY * l))
                        .build(),
                );
            }
        }
        if hi - lo > 1e-6 {
            parts.push(
                polyline(vec![card_point([RULER_X, lo]), card_point([RULER_X, hi])])
                    .style(faded_line(palette::stick(), SHADOW_W, o))
                    .build(),
            );
        } else {
            parts.push(
                Circle::new()
                    .radius(POINT_R)
                    .arc_center(card_point([RULER_X, lo]))
                    .style(flat(palette::stick(), o))
                    .build(),
            );
        }
        s.add(v_group(parts))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The ruler and its ticks. Detached.
fn ruler(stage: &mut Stage<'_>) -> Vec<Mob> {
    let mut out = vec![path(
        stage,
        &[[RULER_X, RULER_Y[0]], [RULER_X, RULER_Y[1]]],
        Palette::ink(),
        RULER_W,
    )];
    let half = (LENGTH / 2.0) as i64;
    for k in -half..=half {
        let y = HOME[1] + k as f64;
        out.push(path(
            stage,
            &[[RULER_X, y], [RULER_X + RULER_TICK, y]],
            Palette::ink(),
            RULER_W,
        ));
    }
    out
}

/// Turn the stick to `to`, `rt` seconds at `rate`; it ends exactly there.
fn turn_to(stage: &mut Stage<'_>, turn: Mob, to: f64, rt: f64, rate: fn(f64) -> f64) -> fmn::Result<()> {
    stage.play(glide(turn, to, rt, rate)?)?;
    stage.set_tracker_value(turn, to)?;
    Ok(())
}

/// A detached text, its left edge at the frame point `at`.
fn left_at(stage: &mut Stage<'_>, m: Mob, at: Vec3) -> Mob {
    stage.move_to(m, at, LEFT);
    m
}

impl SceneConstruct for StraightSticks {
    fn name(&self) -> &str {
        "07_straight_sticks"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let coast = model::coast();
        let slide = sheet::shown_slide(&coast, 0.0);
        let words = words(HOME, slide).map_err(fail)?;

        // where 06 ended: the dimmed card and coast, the sheet at the
        // first placement; then the stick's trackers and live parts, the
        // shadow under the stick, all unseen
        let card = card_panel(stage)?;
        stage.set_family_opacity(card, DIMMED);
        stage.add_to_scene(card)?;
        for p in &coast.parts {
            let m = path(
                stage,
                &coast.between(p.start, p.end(), EVERY),
                palette::coast(),
                COAST_W,
            );
            stage.set_color_lanes(m, "stroke_rgba", None, Some(DIMMED), true);
            stage.add_to_scene(m)?;
        }
        let ruled = Sheet::new(stage, slide, 0.0)?;
        for m in ruled.all() {
            stage.add_to_scene(m)?;
        }
        let turn = stage.add_value_tracker(ACROSS);
        let [shadow_alpha, light_alpha] = [stage.add_value_tracker(0.0), stage.add_value_tracker(0.0)];
        live_shadow(stage, turn, shadow_alpha, light_alpha)?;
        let stick_alpha = stage.add_value_tracker(0.0);
        live_stick(stage, turn, stick_alpha)?;
        let dots_alpha = stage.add_value_tracker(0.0);
        live_crossings(stage, turn, slide, dots_alpha)?;
        let count_alpha = stage.add_value_tracker(0.0);
        let counted = move |s: &fmn::mobject::Stage| crossings(HOME, direction(s, turn), slide).len();
        placement::live_count(&kit, stage, counted, count_alpha, COUNT_SIZE, COUNT_RIGHT)?;
        let word_at = v(COUNT_RIGHT[0] + WORD_GAP, COUNT_RIGHT[1]);
        live_noun(&kit, stage, counted, count_alpha, COUNT_SIZE, word_at)?;

        // 07_1: a straight stick, 6 cm, across the lines
        let line = say(&kit, stage, "07_1")?;
        line.wait_for(stage, "Take a straight stick")?;
        stage.play(tween(stick_alpha, 1.0, FADE_S)?)?;
        line.wait_for(stage, "six centimetres long")?;
        let length = kit.text(
            stage,
            &words.length,
            LABEL_SIZE,
            palette::length(),
            card_point(LENGTH_AT),
        )?;
        fade_in_all(stage, &[length], FADE_S)?;

        // 07_2: three crossings; the bet says 9.4 cm
        let line = say(&kit, stage, "07_2")?;
        line.wait_for(stage, "It crosses three lines")?;
        together(
            stage,
            vec![tween(dots_alpha, 1.0, FADE_S)?, tween(count_alpha, 1.0, FADE_S)?],
        )?;
        line.wait_for(stage, "the bet says nine point four")?;
        let across = kit.tex(stage, &words.rows[0], EQUATION_SIZE, Palette::ink(), &[], ROWS_AT[0])?;
        let across = left_at(stage, across, ROWS_AT[0]);
        fade_in_all(stage, &[across], FADE_S)?;

        // 07_3: the count and its crossings go before the stick turns, so
        // nothing on screen answers the question; it lies along the lines
        let line = say(&kit, stage, "07_3")?;
        line.wait_for(stage, "Now lay it along")?;
        at_once(
            stage,
            &[],
            &[length],
            vec![tween(dots_alpha, 0.0, FADE_S)?, tween(count_alpha, 0.0, FADE_S)?],
            FADE_S,
        )?;
        turn_to(stage, turn, ALONG, LAY_S, smooth)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 07_4: zero, the count back; the bet's two answers
        let line = say(&kit, stage, "07_4")?;
        line.wait_for(stage, "Zero")?;
        at_once(
            stage,
            &[],
            &[q],
            vec![tween(count_alpha, 1.0, FADE_S)?, tween(dots_alpha, 1.0, FADE_S)?],
            FADE_S,
        )?;
        line.wait_for(stage, "nine point four or zero")?;
        let along = kit.tex(stage, &words.rows[1], EQUATION_SIZE, Palette::ink(), &[], ROWS_AT[1])?;
        let along = left_at(stage, along, ROWS_AT[1]);
        fade_in_all(stage, &[along], FADE_S)?;

        // 07_5: the shadow: the ruler, the light along the lines, and the
        // shadow a single point
        let line = say(&kit, stage, "07_5")?;
        line.wait_for(stage, "Picture its shadow")?;
        let marks = ruler(stage);
        fade_in_all(stage, &marks, FADE_S)?;
        line.wait_for(stage, "with the light running along")?;
        together(
            stage,
            vec![tween(light_alpha, 1.0, FADE_S)?, tween(shadow_alpha, 1.0, FADE_S)?],
        )?;
        line.wait_for(stage, "casts no shadow at all")?;
        let none = kit.text(
            stage,
            &words.shadows[0],
            LABEL_SIZE,
            palette::stick(),
            card_point(SHADOW_AT),
        )?;
        let none = left_at(stage, none, card_point(SHADOW_AT));
        fade_in_all(stage, &[none], FADE_S)?;

        // 07_6: the words, then in silence the turn back across at a steady
        // pace, the crossings, count, light and shadow following every frame
        say(&kit, stage, "07_6")?;
        kit.hold(stage)?;
        fade_out_all(stage, &[none], FADE_S)?;
        turn_to(stage, turn, ACROSS, BACK_S, linear)?;

        // 07_7: the shadow the full 6 cm, three crossings
        let line = say(&kit, stage, "07_7")?;
        line.wait_for(stage, "the full six centimetres")?;
        let full = kit.text(
            stage,
            &words.shadows[1],
            LABEL_SIZE,
            palette::stick(),
            card_point(SHADOW_AT),
        )?;
        let full = left_at(stage, full, card_point(SHADOW_AT));
        fade_in_all(stage, &[full], FADE_S)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::THINK_R;
    use super::super::sheet::TOP_Y;
    use super::*;
    use crate::model::{CARD_CENTRE, CARD_H, CARD_UNIT, CARD_W, Curve, Part, Seg};

    fn first_slide() -> f64 {
        sheet::shown_slide(&model::coast(), 0.0)
    }

    fn dist(a: P2, b: P2) -> f64 {
        (a[0] - b[0]).hypot(a[1] - b[1])
    }

    /// The stick as a one-piece curve, for `sheet::crossings`.
    fn as_curve(c: P2, theta: f64) -> Curve {
        let [from, to] = ends(c, theta);
        Curve {
            parts: vec![Part {
                seg: Seg::Line { from, to },
                start: 0.0,
            }],
            length: LENGTH,
        }
    }

    /// How far the point `p` lies from the segment `a`–`b`.
    fn to_segment(p: P2, [a, b]: [P2; 2]) -> f64 {
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let t = (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
        dist(p, [a[0] + t * dx, a[1] + t * dy])
    }

    /// Across, three crossings on the lines at −1.55, 0.45 and 2.45 cm,
    /// straight over the centre; along, none. At every hundredth of a degree
    /// of the turn the count and the points agree with `sheet::crossings`
    /// on the stick as a straight piece of curve, its own rule for a
    /// straight run, and the count with the oracle's rule: how many line
    /// numbers lie between the two ends' (the floor of each end's height
    /// over the gap), which no sampled direction puts exactly on a line.
    #[test]
    fn the_stick_crosses_as_the_model_says() {
        let s = first_slide();
        let across = crossings(HOME, ACROSS, s);
        assert_eq!(across.iter().map(|&(k, _)| k).collect::<Vec<_>>(), [-1, 0, 1]);
        for (&(_, at), h) in across.iter().zip([-1.55, 0.45, 2.45]) {
            assert!(dist(at, [HOME[0], h]) < 1e-12, "{at:?}");
        }
        assert!(crossings(HOME, ALONG, s).is_empty());
        for i in 0..=9000 {
            let theta = turned(f64::from(i) / 9000.0);
            let mine = crossings(HOME, theta, s);
            let theirs = sheet::crossings(&as_curve(HOME, theta), 0.0, s);
            assert_eq!(mine.len(), theirs.len(), "at {theta}");
            for ((k, at), c) in mine.iter().zip(&theirs) {
                assert_eq!(*k, c.line);
                assert!(dist(*at, c.at) < 1e-9);
            }
            let [a, b] = ends(HOME, theta);
            let line_of = |p: P2| ((p[1] - s) / GAP).floor() as i64;
            assert_eq!(
                (line_of(b) - line_of(a)).unsigned_abs() as usize,
                mine.len(),
                "at {theta}"
            );
        }
    }

    /// The turn from along to across passes 0, 1, 2, 3, changing where an
    /// end's height reaches a line: sin φ = 0.75/3, 1.25/3 and 2.75/3, about
    /// 14.48°, 24.62° and 66.44°. The count only climbs, since each shadow
    /// holds every shadow before it; turned on past across it falls again.
    /// At 07_6's steady pace every count shows for at least a second.
    #[test]
    fn the_turn_counts_climb_where_the_ends_reach_the_lines() {
        let s = first_slide();
        assert_eq!(turn_counts(HOME, s), [0, 1, 2, 3]);
        let angles = turning_angles(HOME, s);
        let want = [0.75_f64, 1.25, 2.75].map(|d| (d / 3.0).asin());
        assert_eq!(angles.len(), 3);
        for (a, w) in angles.iter().zip(want) {
            assert!((a - w).abs() < 1e-12, "{a} {w}");
        }
        assert_eq!(
            angles
                .iter()
                .map(|a| facts::fixed(a.to_degrees(), 2))
                .collect::<Vec<_>>(),
            ["14.48", "24.62", "66.44"]
        );
        for (i, a) in angles.iter().enumerate() {
            let at = |phi: f64| crossings(HOME, ALONG - phi, s).len();
            assert_eq!((at(a - 1e-6), at(a + 1e-6)), (i, i + 1));
        }
        for i in 1..=900 {
            let (a, b) = (turned(f64::from(i - 1) / 900.0), turned(f64::from(i) / 900.0));
            let ([a0, a1], [b0, b1]) = (shadow(HOME, a), shadow(HOME, b));
            assert!(b0 <= a0 + 1e-12 && b1 >= a1 - 1e-12);
        }
        let past: Vec<usize> = (0..=1800)
            .map(|i| crossings(HOME, ALONG - PI * f64::from(i) / 1800.0, s).len())
            .collect();
        assert!(past.windows(2).any(|w| w[1] < w[0]));
        let mut edges = vec![0.0];
        edges.extend(angles.iter().copied());
        edges.push(FRAC_PI_2);
        for w in edges.windows(2) {
            assert!((w[1] - w[0]) / FRAC_PI_2 * BACK_S >= 1.0, "{w:?}");
        }
    }

    /// The shadow is where the ends are: its ends are the ends' heights,
    /// its length 6 |sin θ|; the full length across, none along; and a line
    /// crosses the stick just when it lies inside the shadow.
    #[test]
    fn the_shadow_is_the_stick_seen_along_the_lines() {
        let s = first_slide();
        for i in 0..=180 {
            let theta = PI * f64::from(i) / 180.0;
            let [a, b] = ends(HOME, theta);
            let [lo, hi] = shadow(HOME, theta);
            assert_eq!([lo, hi], [a[1].min(b[1]), a[1].max(b[1])]);
            assert!((hi - lo - LENGTH * theta.sin().abs()).abs() < 1e-12);
            assert!((dist(a, b) - LENGTH).abs() < 1e-12);
            for (k, at) in crossings(HOME, theta, s) {
                let h = s + k as f64 * GAP;
                assert!(lo < h && h < hi && at[1] == h);
            }
        }
        let [lo, hi] = shadow(HOME, ACROSS);
        assert_eq!(facts::fixed(hi - lo, 0), "6");
        let [lo, hi] = shadow(HOME, ALONG);
        assert_eq!(facts::fixed(hi - lo, 0), "0");
    }

    /// The words on screen say what the facts and the model say.
    #[test]
    fn the_screen_words_are_built_from_the_facts() {
        let w = words(HOME, first_slide()).unwrap();
        assert_eq!(w.length, "6 cm");
        assert_eq!(
            w.rows,
            [
                r"\text{across}\quad 3 \times \pi\text{ cm} \approx 9.4\text{ cm}",
                r"\text{along}\quad 0 \times \pi\text{ cm} = 0\text{ cm}",
            ]
        );
        assert_eq!(w.shadows, ["shadow 0 cm", "shadow 6 cm"]);
    }

    /// Homes the voice does not describe are refused: ends on the lines
    /// (1.45 cm up), ends 0.15 cm from them (1.3 cm up), and lying along on
    /// a line (0.45 cm up).
    #[test]
    fn a_home_the_voice_does_not_describe_is_refused() {
        let s = first_slide();
        for y in [1.45, 1.3, 0.45] {
            assert!(words([HOME[0], y], s).is_err(), "{y}");
        }
    }

    /// At every tenth of a degree of the turn the stick keeps to the card,
    /// 0.15 cm from the coast and clear of the count; lying across, its
    /// crossings and the "6 cm" beside it clear each other; the ruler stands
    /// beyond the lines' ends, inside the card, its ticks clear of the
    /// lines; the shadow's label lies over the ruler's top, clear of the
    /// stick across and of the light from its upper end; the rows sit in
    /// the band under the coast.
    #[test]
    fn the_layout_keeps_its_rooms() {
        let coast = model::coast();
        let s = first_slide();
        let n = (coast.length / 0.02).ceil() as usize;
        let pts: Vec<P2> = (0..=n).map(|i| coast.at(coast.length * i as f64 / n as f64)).collect();
        let count_box = [
            COUNT_RIGHT[0] - 0.5,
            COUNT_RIGHT[0] + WORD_GAP + 1.8,
            TOP_Y - 0.3,
            TOP_Y + 0.3,
        ];
        let edge = [CARD_W / 2.0 - sheet::LINE_INSET, CARD_H / 2.0];
        for i in 0..=900 {
            let theta = turned(f64::from(i) / 900.0);
            let seg = ends(HOME, theta);
            let near = pts.iter().map(|&p| to_segment(p, seg)).fold(f64::INFINITY, f64::min);
            assert!(near > 0.15, "at {theta} the stick comes {near} from the coast");
            for e in seg {
                assert!(e[0].abs() < edge[0] && e[1].abs() < edge[1], "{e:?}");
                let f = card_point(e);
                let gap_x = (count_box[0] - f[0]).max(f[0] - count_box[1]).max(0.0);
                let gap_y = (count_box[2] - f[1]).max(f[1] - count_box[3]).max(0.0);
                assert!(gap_x.hypot(gap_y) > 0.15, "the stick's end at {e:?} meets the count");
            }
        }
        // the think ring, from the stick lying along
        for e in ends(HOME, ALONG) {
            let f = card_point(e);
            assert!((f[0] - THINK_AT[0]).hypot(f[1] - THINK_AT[1]) > THINK_R + 0.5);
        }
        // "6 cm" at 22 points (at most 0.6 wide), right of the stick across
        let label = card_point(LENGTH_AT);
        assert!(label[0] - 0.3 > card_point(HOME)[0] + 0.2);
        for (_, at) in crossings(HOME, ACROSS, s) {
            assert!(dist(at, LENGTH_AT) > 0.5);
        }
        // the ruler
        const {
            assert!(RULER_X < -(CARD_W / 2.0 - sheet::LINE_INSET) && RULER_X > -CARD_W / 2.0 + 0.1);
            assert!(RULER_Y[0] > -CARD_H / 2.0 && RULER_Y[1] < CARD_H / 2.0);
        }
        for k in -3..=3 {
            assert!(off_lines(HOME[1] + f64::from(k), s) > 0.2);
        }
        let start = coast.at(0.0);
        assert!(start[0] - RULER_X > 0.5);
        // the shadow's label: "shadow 6 cm" at 22 points, at most 1.62
        // wide, left of the stick across and under the light from its top
        let at = card_point(SHADOW_AT);
        let [lower, upper] = ends(HOME, ACROSS);
        assert!(at[0] + 1.62 < card_point(upper)[0] - 0.3);
        assert!(at[1] + 0.15 < card_point(upper)[1] - 0.2 && at[1] - 0.15 > card_point(lower)[1]);
        assert!(at[0] > card_point([RULER_X + RULER_TICK, 0.0])[0] + 0.1);
        // the rows, between the coast and the lowest line
        let lowest = pts.iter().map(|&p| card_point(p)[1]).fold(f64::INFINITY, f64::min);
        let line = CARD_CENTRE[1] + CARD_UNIT * (s - 2.0 * GAP);
        for row in ROWS_AT {
            assert!(row[1] < lowest - 0.35 && row[1] > line + 0.25, "{row:?}");
        }
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("07_1", "Take a straight stick"),
            ("07_1", "six centimetres long"),
            ("07_2", "It crosses three lines"),
            ("07_2", "the bet says nine point four"),
            ("07_3", "Now lay it along"),
            ("07_4", "Zero"),
            ("07_4", "nine point four or zero"),
            ("07_5", "Picture its shadow"),
            ("07_5", "with the light running along"),
            ("07_5", "casts no shadow at all"),
            ("07_7", "the full six centimetres"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
        assert!(script.text("07_6").is_some());
    }
}
