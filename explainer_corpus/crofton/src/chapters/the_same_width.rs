//! Chapter 14, the same width every way. From 13's end (the coast's
//! 8.2165 × π ≈ 25.81 cm) the coast dims and the coin comes back where 12
//! laid it: wherever it landed it met the lines at two points, because of
//! its width (14_1). Width is shown, not named: two rulers pressed against
//! the coin from either side, running the way the lines run, stand 2.00 cm
//! apart, and as the coin turns (a mark on it turning) they do not move
//! (14_2). Is a circle the only shape that's the same width every way?
//! (14_3, a prediction.) No: three corners 2 cm apart, then a compass on
//! each corner draws the arc between the other two (14_4), the Reuleaux
//! triangle. Pressed against it, one ruler touches a corner and the other
//! the arc centred on it, a radius apart (14_5). Turned with the rulers
//! kept pressed against it, will the gap change? (14_6, a prediction.)
//! Never: the shape turns half a turn, the rulers ride up and down with it,
//! and the gap read off them stays 2.00 (14_7). The plain triangle inside,
//! its arcs dimmed, cannot: 1.73 with a side flat, 2.00 with a side upright,
//! 1.73 again (14_8). So its width is constant, like the coin's (14_9).
//! The rulers, where they touch and the gap between them are found every
//! frame from the shape's trackers by its support (how far it reaches up
//! and down), never drawn as fixed guides. Starts where chapter 13 ends.
//! Registered with its spoken cues, computed facts and authored pauses.

use std::f64::consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_4, FRAC_PI_6, PI, TAU};
use std::rc::Rc;

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, TextKind, ink_style, tween};
use fmn::core::rate::{linear, smooth};
use fmn::prelude::*;

use super::a_crowd_of_sticks::{LOW_ROW, Shore, TOP_ROW};
use super::draw::{EVERY, card_point, dim_line, fail, flat, now, pause, say, stand, think, together};
use super::placement::{self, PIVOT_R, Pose, at_once, faded_line, glide, tracked};
use super::sheet::{self, LABEL_SIZE};
use super::the_bent_stick;
use super::the_shadow_rule::row;
use super::why_every_curve::{COIN_AT, bright_shore, coin_at_rest, coin_rim, count_at};
use crate::facts;
use crate::model::{self, COIN, Curve, P2, Part, Seg};
use crate::palette;

pub struct TheSameWidth {
    pub kit: Kit,
}

// --------------------------------------------------------------- the model

/// The Reuleaux triangle's side, card cm: as wide as the coin, and so as
/// wide as the sheet's gap.
pub const SIDE: f64 = COIN;

/// Where it is built and where it rests, card cm: the open card left of the
/// headland, its middle on the first placement's line at 0.45.
pub const BENCH: P2 = [-3.4, 0.45];

/// The corners of the triangle with sides [`SIDE`] whose middle is `at`,
/// turned by `turn` radians from a corner straight up, counterclockwise
/// from that corner.
pub fn corners(at: P2, turn: f64) -> [P2; 3] {
    let r = SIDE / 3f64.sqrt();
    [0.0, 1.0, 2.0].map(|k: f64| {
        let a = FRAC_PI_2 + TAU * k / 3.0 + turn;
        [at[0] + r * a.cos(), at[1] + r * a.sin()]
    })
}

/// The Reuleaux triangle on those corners: arc k centred on corner k, of
/// radius [`SIDE`], counterclockwise from corner k + 1 to corner k + 2, a
/// sixth of a turn. The arcs follow each other round the rim.
pub fn reuleaux(at: P2, turn: f64) -> Curve {
    let c = corners(at, turn);
    let mut parts = Vec::with_capacity(3);
    let mut start = 0.0;
    for (k, &centre) in c.iter().enumerate() {
        let from = 4.0 * FRAC_PI_3 + TAU * k as f64 / 3.0 + turn;
        let seg = Seg::Arc {
            centre,
            radius: SIDE,
            from,
            to: from + FRAC_PI_3,
        };
        parts.push(Part { seg, start });
        start += seg.len();
    }
    Curve { parts, length: start }
}

/// The plain triangle on the same corners: side k is arc k's chord.
pub fn triangle(at: P2, turn: f64) -> Curve {
    let c = corners(at, turn);
    let mut parts = Vec::with_capacity(3);
    let mut start = 0.0;
    for k in 0..3 {
        let seg = Seg::Line {
            from: c[(k + 1) % 3],
            to: c[(k + 2) % 3],
        };
        parts.push(Part { seg, start });
        start += seg.len();
    }
    Curve { parts, length: start }
}

/// How far `curve` reaches along the unit direction `n` (its support), and
/// a point where it gets there: the farthest end of any piece, or an arc's
/// point facing `n` when the arc's sweep runs past that direction.
pub fn reach(curve: &Curve, n: P2) -> (f64, P2) {
    let along = |p: P2| p[0] * n[0] + p[1] * n[1];
    let mut best = (f64::NEG_INFINITY, [0.0, 0.0]);
    for part in &curve.parts {
        let mut points = vec![part.seg.point(0.0), part.seg.point(1.0)];
        if let Seg::Arc {
            centre,
            radius,
            from,
            to,
        } = part.seg
        {
            let (lo, span) = (from.min(to), (to - from).abs());
            let u = (n[1].atan2(n[0]) - lo).rem_euclid(TAU);
            if u > 0.0 && u < span {
                points.push([centre[0] + radius * n[0], centre[1] + radius * n[1]]);
            }
        }
        for p in points {
            if along(p) > best.0 + 1e-12 {
                best = (along(p), p);
            }
        }
    }
    best
}

/// Two rulers pressed against a shape from below and above, running across
/// the card as the lines of the first placement do: their heights (lower,
/// upper), where each touches, and how far the shape reaches left and
/// right (the rulers are drawn a little past both).
#[derive(Clone, Copy, Debug)]
pub struct Rulers {
    pub heights: [f64; 2],
    pub touches: [P2; 2],
    pub sides: [f64; 2],
}

pub fn rulers(curve: &Curve) -> Rulers {
    let (top, high) = reach(curve, [0.0, 1.0]);
    let (bottom, low) = reach(curve, [0.0, -1.0]);
    let (right, _) = reach(curve, [1.0, 0.0]);
    let (left, _) = reach(curve, [-1.0, 0.0]);
    Rulers {
        heights: [-bottom, top],
        touches: [low, high],
        sides: [-left, right],
    }
}

fn mix(a: f64, b: f64, u: f64) -> f64 {
    a + (b - a) * u
}

impl Rulers {
    /// The gap between them, cm.
    pub fn width(&self) -> f64 {
        self.heights[1] - self.heights[0]
    }

    /// Part of the way, `u` (0 to 1), to `other`'s.
    fn toward(self, other: Rulers, u: f64) -> Rulers {
        let both = |a: [f64; 2], b: [f64; 2]| [mix(a[0], b[0], u), mix(a[1], b[1], u)];
        Rulers {
            heights: both(self.heights, other.heights),
            touches: [
                both(self.touches[0], other.touches[0]),
                both(self.touches[1], other.touches[1]),
            ],
            sides: both(self.sides, other.sides),
        }
    }
}

/// 14's turns: built with a corner up, half a turn under the rulers, then
/// the plain triangle a twelfth of a turn on to a side upright and a
/// twelfth more, a corner up again, where 15 takes it.
pub const HALF_TURN: f64 = PI;
pub const UPRIGHT: f64 = PI + FRAC_PI_6;
pub const END_TURN: f64 = PI + FRAC_PI_3;

/// How many steps the half turn is checked at (the readout's every shown
/// value).
const TURN_STEPS: u32 = 3600;

// ------------------------------------------------------------------ words

/// The chapter's words that carry numbers, from the facts. Refused unless
/// the model gives what the screen and the voice say: the coin's two
/// meeting points at the first placement and the 2.00 cm its rulers stand
/// apart; three corners two centimetres apart and three arcs of sixty
/// degrees, each from the next corner to the one after; the gap between
/// the rulers 2.00 at every step of the half turn, the touches a radius
/// apart, one above the other, one at a corner; the plain triangle 1.73
/// flat, 2.00 upright, 1.73 flat again.
pub struct Words {
    pub side: String,
    pub name: String,
    pub exact: String,
}

const NAME: &str = "Reuleaux triangle";
const EXACT: &str = "width in a direction: the gap between parallel rulers pressed against both sides. Turning \
                     the shape under rulers that run one way measures every direction";

/// The readout under the rulers: the gap between them, to two places.
pub fn readout(gap: f64) -> String {
    format!("width {} cm", facts::fixed(gap, 2))
}

pub fn words() -> Result<Words, String> {
    let f = facts::facts()?;
    let get = |key: &str| f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"));
    let fixed = facts::fixed;
    let agree = |key: &str, got: String| -> Result<String, String> {
        let want = get(key)?;
        if want == got {
            Ok(want)
        } else {
            Err(format!("{key}: the model gives {got}, the facts {want}"))
        }
    };
    let coast = model::coast();
    let first = Pose::new(0.0, sheet::shown_slide(&coast, 0.0));

    // 14_1 and 14_2: the coin's two meeting points; its rulers' gap
    let coin = coin_rim(COIN_AT);
    agree("coin.crossings", count_at(&coin, first).to_string())?;
    agree("coin.width", fixed(rulers(&coin).width(), 0))?;
    agree("coin.width_2dp", fixed(rulers(&coin).width(), 2))?;

    // 14_4: the corners a side apart; each arc centred on one, from the
    // next round to the one after, a sixth of a turn
    let side = agree("reuleaux.width", fixed(SIDE, 0))?;
    let c = corners(BENCH, 0.0);
    let rim = reuleaux(BENCH, 0.0);
    for i in 0..3 {
        let apart = why_apart(c[i], c[(i + 1) % 3]);
        if (apart - SIDE).abs() > 1e-12 {
            return Err(format!("14_4 says the corners are {side} cm apart, not {apart:.6}"));
        }
        let seg = rim.parts[i].seg;
        let (start, end) = (seg.point(0.0), seg.point(1.0));
        if why_apart(start, c[(i + 1) % 3]) > 1e-12 || why_apart(end, c[(i + 2) % 3]) > 1e-12 {
            return Err(format!(
                "14_4 says arc {i} runs from the next corner round to the one after"
            ));
        }
    }
    agree("reuleaux.arcs", rim.parts.len().to_string())?;
    let turned = match rim.parts[0].seg {
        Seg::Arc { from, to, .. } => (to - from).abs(),
        Seg::Line { .. } => 0.0,
    };
    agree("reuleaux.arc_degrees", fixed(turned.to_degrees(), 0))?;

    // 14_5 and 14_7: the gap at every step of the half turn, as shown; the
    // touches a radius apart, one straight above the other, one a corner
    let shown = get("reuleaux.width_2dp")?;
    for i in 0..=TURN_STEPS {
        let t = HALF_TURN * f64::from(i) / f64::from(TURN_STEPS);
        let r = rulers(&reuleaux(BENCH, t));
        if fixed(r.width(), 2) != shown {
            return Err(format!("14_7 shows {shown} at every turn, not {} at {t:.4}", r.width()));
        }
        let [low, high] = r.touches;
        let at_corner = corners(BENCH, t)
            .iter()
            .any(|&q| why_apart(q, low) < 1e-9 || why_apart(q, high) < 1e-9);
        if (why_apart(low, high) - SIDE).abs() > 1e-9 || (low[0] - high[0]).abs() > 1e-9 || !at_corner {
            return Err(format!(
                "14_7 says a corner touches one ruler and its arc the other, at {t:.4}"
            ));
        }
    }

    // 14_8: the plain triangle flat, upright and flat again
    let flat = agree(
        "triangle.width_flat",
        fixed(rulers(&triangle(BENCH, HALF_TURN)).width(), 2),
    )?;
    agree(
        "triangle.width_upright",
        fixed(rulers(&triangle(BENCH, UPRIGHT)).width(), 2),
    )?;
    if fixed(rulers(&triangle(BENCH, END_TURN)).width(), 2) != flat {
        return Err(format!("14_8 says it drops again to {flat}"));
    }

    Ok(Words {
        side: format!("{side} cm"),
        name: NAME.into(),
        exact: EXACT.into(),
    })
}

fn why_apart(a: P2, b: P2) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

// ------------------------------------------------------------------ layout

/// How far the rulers run past the shape either side, and how far they
/// stand off it before they are pressed, card cm.
pub const RULER_OVER: f64 = 0.25;
pub const STAND_OFF: f64 = 0.35;
const RULER_W: f64 = 3.0;
const RIM_W: f64 = 4.0;
const SIDE_W: f64 = 2.5;
const SPAN_W: f64 = 2.5;
const MARK_W: f64 = 2.5;
const CORNER_R: f64 = 0.05;
/// The sides' opacity drawn, dimmed under the rim, and the rim's dimmed
/// under the plain triangle (stroke only: the live drawing restrokes them).
pub const SIDES_SHOWN: f64 = 0.7;
const SIDES_DIM: f64 = 0.25;
const RIM_DIM: f64 = 0.2;
/// The coast while the shapes are made, as 12 dims it.
pub const COAST_DIMMED: f64 = 0.35;
/// The name over the shape and the side's length inside it, card cm.
pub const NAME_AT: P2 = [-3.4, 2.1];
const SIDE_AT: P2 = [-3.4, 0.15];
const SIDE_SIZE: f64 = 20.0;
/// The countdown, right of the coin's rulers (frame).
pub const THINK_RIGHT: Vec3 = [4.4, 2.9, 0.0];
/// Where the mark on the coin starts, radians.
const MARK_FROM: f64 = FRAC_PI_4;

// ------------------------------------------------------------------ timing

const FADE_S: f64 = 0.4;
const PRESS_S: f64 = 1.5;
const COIN_TURN_S: f64 = 4.0;
const ARC_S: f64 = 3.0;
const ARCS_S: f64 = 5.0;
const HALF_TURN_S: f64 = 12.0;
const PLAIN_S: f64 = 1.5;
const TWELFTH_S: f64 = 3.0;
const UPRIGHT_HOLD: f64 = 1.2;

// --------------------------------------------------------- what is drawn

/// The shape, drawn every frame from trackers: where its middle is, how
/// far it is turned, how much of its rim the compass has drawn (0 to 3, an
/// arc a unit), and the opacity of its rim, its triangle's sides and its
/// corners.
#[derive(Clone, Copy)]
pub struct Shape {
    pub x: Mob,
    pub y: Mob,
    pub turn: Mob,
    pub drawn: Mob,
    pub rim: Mob,
    pub sides: Mob,
    pub corners: Mob,
}

impl Shape {
    /// The shape at `at`, turned by `turn`, `drawn` arcs drawn, unseen.
    pub fn new(stage: &mut Stage<'_>, at: P2, turn: f64, drawn: f64) -> Self {
        let [x, y, turn, drawn, rim, sides, corners] =
            [at[0], at[1], turn, drawn, 0.0, 0.0, 0.0].map(|t| stage.add_value_tracker(t));
        Self {
            x,
            y,
            turn,
            drawn,
            rim,
            sides,
            corners,
        }
    }

    /// Where its middle is and how far it is turned, as its trackers hold
    /// them.
    pub fn pose(&self, s: &fmn::mobject::Stage) -> (P2, f64) {
        let get = |m: Mob| s.tracker_value(m).unwrap_or(0.0);
        ([get(self.x), get(self.y)], get(self.turn))
    }

    /// Its rim as its trackers place it.
    pub fn rim_curve(&self, s: &fmn::mobject::Stage) -> Curve {
        let (at, t) = self.pose(s);
        reuleaux(at, t)
    }

    /// Glides carrying it to `at` and turning it to `turn`, `rt` seconds.
    pub fn moves(&self, at: P2, turn: f64, rt: f64) -> fmn::Result<Vec<AnimBuilder>> {
        Ok(vec![
            glide(self.x, at[0], rt, smooth)?,
            glide(self.y, at[1], rt, smooth)?,
            glide(self.turn, turn, rt, smooth)?,
        ])
    }
}

/// Card points along `seg` from its start to the share `f` of it.
fn seg_points(seg: &Seg, f: f64) -> Vec<Vec3> {
    let n = ((seg.len() * f / EVERY).ceil() as usize).max(1);
    (0..=n)
        .map(|j| card_point(seg.point(f * j as f64 / n as f64)))
        .collect()
}

/// The shape as its trackers hold it: the sides, the rim up to where the
/// compass has drawn it, the corners. In the scene.
fn live_shape(stage: &mut Stage<'_>, shape: Shape) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let (at, t) = shape.pose(s);
        let drawn = s.tracker_value(shape.drawn).unwrap_or(0.0);
        let (rim, sides, dots) = (
            tracked(s, shape.rim),
            tracked(s, shape.sides),
            tracked(s, shape.corners),
        );
        let mut out: Vec<VMobject> = triangle(at, t)
            .parts
            .iter()
            .map(|p| {
                polyline(seg_points(&p.seg, 1.0))
                    .style(faded_line(Palette::ink(), SIDE_W, sides))
                    .build()
            })
            .collect();
        for (k, part) in reuleaux(at, t).parts.iter().enumerate() {
            let f = (drawn - k as f64).clamp(0.0, 1.0);
            if f > 0.0 {
                out.push(
                    polyline(seg_points(&part.seg, f))
                        .style(faded_line(palette::coin(), RIM_W, rim))
                        .build(),
                );
            }
        }
        for c in corners(at, t) {
            out.push(
                Circle::new()
                    .radius(CORNER_R)
                    .arc_center(card_point(c))
                    .style(flat(Palette::ink(), dots))
                    .build(),
            );
        }
        s.add(v_group(out))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The compass drawing the arc the shape's `drawn` has reached: its point
/// on that arc's corner (a small ring) and its leg out to the rim, at the
/// opacity `alpha` holds. In the scene.
fn live_compass(stage: &mut Stage<'_>, shape: Shape, alpha: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let (at, t) = shape.pose(s);
        let drawn = s.tracker_value(shape.drawn).unwrap_or(0.0).clamp(0.0, 3.0);
        let o = tracked(s, alpha);
        let k = (drawn.floor() as usize).min(2);
        let centre = corners(at, t)[k];
        let pencil = reuleaux(at, t).parts[k].seg.point(drawn - k as f64);
        let parts = vec![
            Circle::new()
                .radius(PIVOT_R)
                .arc_center(card_point(centre))
                .style(faded_line(Palette::ink(), MARK_W, o))
                .build(),
            polyline(vec![card_point(centre), card_point(pencil)])
                .style(faded_line(Palette::ink(), MARK_W, o))
                .build(),
        ];
        s.add(v_group(parts))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The trackers a pair of pressed rulers reads: their opacity, how far
/// they stand off what they measure (cm), the rings where they touch
/// (lower, upper), the span between the touches and the readout's.
#[derive(Clone, Copy)]
pub struct Press {
    pub alpha: Mob,
    pub off: Mob,
    pub rings: [Mob; 2],
    pub span: Mob,
    pub readout: Mob,
}

impl Press {
    /// Rulers on what `measure` finds every frame, standing off it, unseen:
    /// the rulers, the rings and span, and the readout, in the scene.
    pub fn new(
        kit: &Kit,
        stage: &mut Stage<'_>,
        measure: impl Fn(&fmn::mobject::Stage) -> Rulers + Clone + 'static,
    ) -> fmn::Result<Self> {
        let [alpha, off, low, high, span, readout] =
            [0.0, STAND_OFF, 0.0, 0.0, 0.0, 0.0].map(|x| stage.add_value_tracker(x));
        let press = Self {
            alpha,
            off,
            rings: [low, high],
            span,
            readout,
        };
        live_rulers(stage, measure.clone(), press)?;
        live_touches(stage, measure.clone(), press)?;
        live_readout(kit, stage, measure, press)?;
        Ok(press)
    }

    /// Every one of its opacities to `o`, `rt` seconds.
    pub fn shown(&self, o: f64, rt: f64) -> fmn::Result<Vec<AnimBuilder>> {
        [self.alpha, self.rings[0], self.rings[1], self.span, self.readout]
            .iter()
            .map(|&m| tween(m, o, rt))
            .collect()
    }
}

/// The rulers where `measure` puts them, `press.off` away from it, drawn a
/// little past the shape both sides. In the scene.
fn live_rulers(
    stage: &mut Stage<'_>,
    measure: impl Fn(&fmn::mobject::Stage) -> Rulers + 'static,
    press: Press,
) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let (r, o) = (measure(s), tracked(s, press.alpha));
        let off = s.tracker_value(press.off).unwrap_or(0.0);
        let [left, right] = [r.sides[0] - RULER_OVER, r.sides[1] + RULER_OVER];
        let lines: Vec<VMobject> = [r.heights[0] - off, r.heights[1] + off]
            .iter()
            .map(|&h| {
                polyline(vec![card_point([left, h]), card_point([right, h])])
                    .style(faded_line(Palette::ink(), RULER_W, o))
                    .build()
            })
            .collect();
        s.add(v_group(lines))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// Where the pressed rulers touch, small rings, and the span between the
/// touches. In the scene.
fn live_touches(
    stage: &mut Stage<'_>,
    measure: impl Fn(&fmn::mobject::Stage) -> Rulers + 'static,
    press: Press,
) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let r = measure(s);
        let rings = press.rings.map(|ring| tracked(s, ring));
        let span = tracked(s, press.span);
        let mut out: Vec<VMobject> = r
            .touches
            .iter()
            .zip(rings)
            .map(|(&at, o)| {
                Circle::new()
                    .radius(PIVOT_R)
                    .arc_center(card_point(at))
                    .style(faded_line(Palette::ink(), MARK_W, o))
                    .build()
            })
            .collect();
        out.push(
            polyline(vec![card_point(r.touches[0]), card_point(r.touches[1])])
                .style(faded_line(Palette::ink(), SPAN_W, span))
                .build(),
        );
        s.add(v_group(out))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The gap between the rulers as they stand (their measure's width and
/// twice the stand-off), "width 2.00 cm", on the console's top row. In the
/// scene.
fn live_readout(
    kit: &Kit,
    stage: &mut Stage<'_>,
    measure: impl Fn(&fmn::mobject::Stage) -> Rulers + 'static,
    press: Press,
) -> fmn::Result<Mob> {
    let book = Rc::clone(&kit.book);
    let m = stage.always_redraw(move |s| {
        let o = tracked(s, press.readout);
        let gap = measure(s).width() + 2.0 * s.tracker_value(press.off).unwrap_or(0.0);
        let words = readout(gap);
        let t = Text::new(&words)
            .font_size(LABEL_SIZE)
            .style(ink_style(Palette::ink()))
            .build(&book)
            .expect("the bundled face sets every word on screen");
        explainer_kit::glyphs::record(TextKind::Text, &words);
        let m = s.add(t);
        s.move_to(m, TOP_ROW, LEFT);
        s.set_family_opacity(m, o);
        m
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The rulers on the shape: the rim's, or part of the way (`plain`, 0 to
/// 1) to the plain triangle's.
pub fn measure(shape: Shape, plain: Mob) -> impl Fn(&fmn::mobject::Stage) -> Rulers + Clone + 'static {
    move |s| {
        let (at, t) = shape.pose(s);
        rulers(&reuleaux(at, t)).toward(rulers(&triangle(at, t)), tracked(s, plain))
    }
}

/// A mark on the coin, from its middle to its rim at `turn` past
/// [`MARK_FROM`], so its turning shows. In the scene.
fn live_coin_mark(stage: &mut Stage<'_>, turn: Mob, alpha: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let a = MARK_FROM + s.tracker_value(turn).unwrap_or(0.0);
        let r = COIN / 2.0;
        let tip = [COIN_AT[0] + r * a.cos(), COIN_AT[1] + r * a.sin()];
        let mark = polyline(vec![card_point(COIN_AT), card_point(tip)])
            .style(faded_line(Palette::ink(), MARK_W, tracked(s, alpha)))
            .build();
        s.add(mark)
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// Everything 14 builds on the bench: the shape, its compass, its rulers
/// and how far they have moved to the plain triangle's. Built unseen, the
/// shape a corner up and undrawn.
#[derive(Clone, Copy)]
pub struct Bench {
    pub shape: Shape,
    pub compass: Mob,
    pub plain: Mob,
    pub press: Press,
}

impl Bench {
    pub fn new(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let shape = Shape::new(stage, BENCH, 0.0, 0.0);
        live_shape(stage, shape)?;
        let [compass, plain] = [0.0, 0.0].map(|x| stage.add_value_tracker(x));
        live_compass(stage, shape, compass)?;
        let press = Press::new(kit, stage, measure(shape, plain))?;
        Ok(Self {
            shape,
            compass,
            plain,
            press,
        })
    }

    /// Set at once as 14 leaves it: drawn and turned to [`END_TURN`], its
    /// rim and corners up, its sides gone, the rulers pressed on the rim
    /// with their rings, span and readout.
    pub fn as_14_ends(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let s = self.shape;
        for (m, x) in [
            (s.turn, END_TURN),
            (s.drawn, 3.0),
            (s.rim, 1.0),
            (s.corners, 1.0),
            (self.press.off, 0.0),
            (self.press.alpha, 1.0),
            (self.press.rings[0], 1.0),
            (self.press.rings[1], 1.0),
            (self.press.span, 1.0),
            (self.press.readout, 1.0),
        ] {
            stage.set_tracker_value(m, x)?;
        }
        Ok(())
    }
}

/// The shore as 13 leaves it, the coast's crossings and count up at the
/// first placement, 13's last row on the console (in the scene).
fn where_13_ended(kit: &Kit, stage: &mut Stage<'_>, coast: &Curve, first: Pose) -> fmn::Result<(Shore, Mob)> {
    let shore = bright_shore(kit, stage, coast, first, false)?;
    stage.set_tracker_value(shore.dots_alpha, 1.0)?;
    stage.set_tracker_value(shore.count_alpha, 1.0)?;
    let earlier = the_bent_stick::words().map_err(fail)?;
    let back = row(kit, stage, &earlier.coast, palette::length(), LOW_ROW)?;
    stage.add_to_scene(back)?;
    Ok((shore, back))
}

impl SceneConstruct for TheSameWidth {
    fn name(&self) -> &str {
        "14_the_same_width"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let coast = model::coast();
        let first = Pose::new(0.0, sheet::shown_slide(&coast, 0.0));
        let words = words().map_err(fail)?;

        // where 13 ended: the shore bright, the coast's seven dots and count,
        // 13's row; the bench and the coin's rulers made unseen
        let (shore, back) = where_13_ended(&kit, stage, &coast, first)?;
        let place = shore.place;
        let bench = Bench::new(&kit, stage)?;
        let shape = bench.shape;
        let rim = coin_rim(COIN_AT);
        let coin_press = Press::new(&kit, stage, move |_s: &fmn::mobject::Stage| rulers(&rim))?;
        let [coin_dots, coin_turn, coin_mark] = [0.0, 0.0, 0.0].map(|x| stage.add_value_tracker(x));
        placement::live_dots(stage, place, &coin_rim(COIN_AT), coin_dots)?;
        live_coin_mark(stage, coin_turn, coin_mark)?;
        let coin = coin_at_rest(stage, COIN_AT);

        // 14_1: the coast dims, its dots, count and row go; the coin comes
        // back where 12 laid it, and its two meeting points
        let line = say(&kit, stage, "14_1")?;
        line.wait_for(stage, "Think back to the coin")?;
        let mut moves = vec![
            tween(shore.dots_alpha, 0.0, FADE_S)?,
            tween(shore.count_alpha, 0.0, FADE_S)?,
        ];
        for &m in &shore.pieces {
            moves.push(dim_line(m, COAST_DIMMED, FADE_S)?);
        }
        at_once(stage, &coin, &[back], moves, FADE_S)?;
        line.wait_for(stage, "it met the lines at two points")?;
        stage.play(tween(coin_dots, 1.0, FADE_S)?)?;

        // 14_2: two rulers pressed on the coin, 2.00 cm apart; the coin
        // turns a whole turn under them and they stay
        let line = say(&kit, stage, "14_2")?;
        line.wait_for(stage, "Press two straight rulers")?;
        together(
            stage,
            vec![tween(coin_dots, 0.0, FADE_S)?, tween(coin_press.alpha, 1.0, FADE_S)?],
        )?;
        stage.play(glide(coin_press.off, 0.0, PRESS_S, smooth)?)?;
        line.wait_for(stage, "The gap between them is the coin's width")?;
        together(
            stage,
            vec![
                tween(coin_press.rings[0], 1.0, FADE_S)?,
                tween(coin_press.rings[1], 1.0, FADE_S)?,
                tween(coin_press.span, 1.0, FADE_S)?,
                tween(coin_press.readout, 1.0, FADE_S)?,
            ],
        )?;
        line.wait_for(stage, "Turn the coin")?;
        stage.play(tween(coin_mark, 1.0, FADE_S)?)?;
        stage.play(glide(coin_turn, TAU, COIN_TURN_S, smooth)?)?;

        // 14_3: is a circle the only one?
        let line = say(&kit, stage, "14_3")?;
        line.wait_for(stage, "whichever way you turn it")?;
        let q = think(&kit, stage, THINK_RIGHT)?;

        // 14_4: no; three corners a side apart, the triangle's sides, then
        // the compass on each corner drawing the arc between the other two
        let line = say(&kit, stage, "14_4")?;
        let mut gone = coin_press.shown(0.0, FADE_S)?;
        gone.push(tween(coin_mark, 0.0, FADE_S)?);
        at_once(stage, &[], &[q], gone, FADE_S)?;
        line.wait_for(stage, "Mark three points")?;
        stage.play(tween(shape.corners, 1.0, FADE_S)?)?;
        line.wait_for(stage, "the corners of a triangle")?;
        let side_label = kit.text(stage, &words.side, SIDE_SIZE, Palette::ink(), card_point(SIDE_AT))?;
        at_once(
            stage,
            &[side_label],
            &[],
            vec![tween(shape.sides, SIDES_SHOWN, FADE_S)?],
            FADE_S,
        )?;
        line.wait_for(stage, "Put the point of a compass on one corner")?;
        together(
            stage,
            vec![tween(bench.compass, 1.0, FADE_S)?, tween(shape.rim, 1.0, FADE_S)?],
        )?;
        line.wait_for(stage, "draw an arc from the second corner")?;
        stage.play(glide(shape.drawn, 1.0, ARC_S, smooth)?)?;
        line.wait_for(stage, "Do the same from each corner")?;
        stage.play(glide(shape.drawn, 3.0, ARCS_S, linear)?)?;
        stage.play(tween(bench.compass, 0.0, FADE_S)?)?;

        // 14_5: its name; the rulers pressed on it, a corner touching one,
        // the arc centred on that corner the other; the radius between
        let line = say(&kit, stage, "14_5")?;
        line.wait_for(stage, "This curve is called a Reuleaux triangle")?;
        let name = kit.text(stage, &words.name, LABEL_SIZE, palette::coin(), card_point(NAME_AT))?;
        at_once(
            stage,
            &[name],
            &[side_label],
            vec![tween(shape.sides, SIDES_DIM, FADE_S)?],
            FADE_S,
        )?;
        line.wait_for(stage, "Press the rulers against it")?;
        stage.play(tween(bench.press.alpha, 1.0, FADE_S)?)?;
        stage.play(glide(bench.press.off, 0.0, PRESS_S, smooth)?)?;
        line.wait_for(stage, "One touches a corner")?;
        stage.play(tween(bench.press.rings[1], 1.0, FADE_S)?)?;
        line.wait_for(stage, "The other touches the arc across from that corner")?;
        stage.play(tween(bench.press.rings[0], 1.0, FADE_S)?)?;
        line.wait_for(stage, "so the gap between them is its radius")?;
        together(
            stage,
            vec![
                tween(bench.press.span, 1.0, FADE_S)?,
                tween(bench.press.readout, 1.0, FADE_S)?,
            ],
        )?;

        // 14_6: turned with the rulers kept pressed, will the gap change?
        let line = say(&kit, stage, "14_6")?;
        line.wait_for(stage, "Will the gap between them ever change")?;
        let q = think(&kit, stage, THINK_RIGHT)?;

        // 14_7: never; half a turn, the rulers riding with it, the readout
        // steady; the exact definition below
        let _line = say(&kit, stage, "14_7")?;
        let exact = precisely(&kit, stage, &words.exact)?;
        at_once(
            stage,
            &[exact],
            &[q],
            vec![glide(shape.turn, HALF_TURN, HALF_TURN_S, linear)?],
            FADE_S,
        )?;

        // 14_8: the plain triangle: its arcs dimmed, the rulers pressed on
        // its sides, 1.73 flat; a twelfth of a turn to 2.00 upright, and a
        // twelfth more to 1.73
        let line = say(&kit, stage, "14_8")?;
        line.wait_for(stage, "The triangle inside it can't do that")?;
        at_once(
            stage,
            &[],
            &[exact],
            vec![
                tween(shape.rim, RIM_DIM, FADE_S)?,
                tween(shape.sides, 1.0, FADE_S)?,
                tween(bench.press.rings[0], 0.0, FADE_S)?,
                tween(bench.press.rings[1], 0.0, FADE_S)?,
                tween(bench.press.span, 0.0, FADE_S)?,
            ],
            FADE_S,
        )?;
        line.wait_for(stage, "Without its arcs")?;
        stage.play(glide(bench.plain, 1.0, PLAIN_S, smooth)?)?;
        line.wait_for(stage, "comes back to two")?;
        stage.play(glide(shape.turn, UPRIGHT, TWELFTH_S, smooth)?)?;
        pause(stage, UPRIGHT_HOLD)?;
        line.wait_for(stage, "and drops again")?;
        stage.play(glide(shape.turn, END_TURN, TWELFTH_S, smooth)?)?;

        // 14_9: the arcs back, the rulers on the rim again, 2.00
        let line = say(&kit, stage, "14_9")?;
        line.wait_for(stage, "So the Reuleaux triangle")?;
        together(
            stage,
            vec![
                tween(shape.rim, 1.0, FADE_S)?,
                tween(shape.sides, 0.0, FADE_S)?,
                glide(bench.plain, 0.0, PLAIN_S, smooth)?,
                tween(bench.press.rings[0], 1.0, FADE_S)?,
                tween(bench.press.rings[1], 1.0, FADE_S)?,
                tween(bench.press.span, 1.0, FADE_S)?,
            ],
        )?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::the_shadow_rule::CONSOLE;
    use super::*;
    use crate::model::{CARD_H, CARD_W};

    fn half_height(size: f64) -> f64 {
        size / 150.0
    }

    fn width(text: &str, size: f64) -> f64 {
        text.chars().count() as f64 * size / 150.0
    }

    fn bounds((at, w, h): (Vec3, f64, f64)) -> [f64; 4] {
        [at[0] - w / 2.0, at[0] + w / 2.0, at[1] - h / 2.0, at[1] + h / 2.0]
    }

    fn shore() -> Vec<P2> {
        let coast = model::coast();
        coast.between(0.0, coast.length, 0.02)
    }

    fn near(points: &[P2], shore: &[P2]) -> f64 {
        points
            .iter()
            .flat_map(|&p| shore.iter().map(move |&q| why_apart(p, q)))
            .fold(f64::MAX, f64::min)
    }

    fn rim_points(c: &Curve) -> Vec<P2> {
        c.parts
            .iter()
            .flat_map(|p| (0..=40).map(move |k| p.seg.point(f64::from(k) / 40.0)))
            .collect()
    }

    /// The rulers' two lines as points, standing `off` away.
    fn ruler_points(r: &Rulers, off: f64) -> Vec<P2> {
        let [left, right] = [r.sides[0] - RULER_OVER, r.sides[1] + RULER_OVER];
        (0..=40)
            .flat_map(|j| {
                let x = left + (right - left) * f64::from(j) / 40.0;
                [[x, r.heights[0] - off], [x, r.heights[1] + off]]
            })
            .collect()
    }

    /// A square of side 2 about the origin, turned by `t`: a control whose
    /// width changes.
    fn square(t: f64) -> Curve {
        let p: Vec<P2> = (0..=4)
            .map(|k| {
                let a = FRAC_PI_4 + FRAC_PI_2 * f64::from(k) + t;
                [2f64.sqrt() * a.cos(), 2f64.sqrt() * a.sin()]
            })
            .collect();
        let mut parts = Vec::new();
        let mut start = 0.0;
        for w in p.windows(2) {
            let seg = Seg::Line { from: w[0], to: w[1] };
            parts.push(Part { seg, start });
            start += seg.len();
        }
        Curve { parts, length: start }
    }

    #[test]
    fn the_screen_words_are_built_from_the_facts() {
        let w = words().unwrap();
        assert_eq!(w.side, "2 cm");
        assert_eq!(w.name, "Reuleaux triangle");
        assert_eq!(readout(rulers(&coin_rim(COIN_AT)).width()), "width 2.00 cm");
        assert_eq!(readout(rulers(&triangle(BENCH, HALF_TURN)).width()), "width 1.73 cm");
        assert_eq!(readout(rulers(&triangle(BENCH, UPRIGHT)).width()), "width 2.00 cm");
    }

    /// The rim is three arcs of radius 2, a sixth of a turn each, end to
    /// end and closed; 2π in all.
    #[test]
    fn the_rim_is_three_arcs_end_to_end() {
        for t in [0.0, 0.3, HALF_TURN, END_TURN] {
            let c = reuleaux(BENCH, t);
            assert_eq!(c.parts.len(), 3);
            for (i, p) in c.parts.iter().enumerate() {
                let next = c.parts[(i + 1) % 3].seg.point(0.0);
                assert!(why_apart(p.seg.point(1.0), next) < 1e-12, "{t} {i}");
                assert!((p.seg.len() - SIDE * FRAC_PI_3).abs() < 1e-12);
            }
            assert!((c.length - 2.0 * PI).abs() < 1e-12);
        }
    }

    /// The width is the same in 3,600 directions, not only across the card
    /// (support either way); the plain triangle's runs from √3 to 2 and a
    /// square's from 2 to 2√2, so a picture of rulers that did not move
    /// would be wrong for them (a negative control).
    #[test]
    fn its_width_is_the_same_every_way() {
        let c = reuleaux(BENCH, 0.3);
        let (mut tri_lo, mut tri_hi, mut sq_lo, mut sq_hi) = (f64::MAX, 0.0f64, f64::MAX, 0.0f64);
        for i in 0..3600 {
            let a = PI * (f64::from(i) + 0.5) / 3600.0;
            let n = [a.cos(), a.sin()];
            let both = |c: &Curve| reach(c, n).0 + reach(c, [-n[0], -n[1]]).0;
            assert!((both(&c) - SIDE).abs() < 1e-12, "{a}");
            let t = both(&triangle(BENCH, 0.3));
            let q = both(&square(0.0));
            (tri_lo, tri_hi, sq_lo, sq_hi) = (tri_lo.min(t), tri_hi.max(t), sq_lo.min(q), sq_hi.max(q));
        }
        // the midpoint directions miss each extreme by under a 7,200th of
        // half a turn
        assert!((tri_lo - 3f64.sqrt()).abs() < 2e-3 && (tri_hi - 2.0).abs() < 2e-3);
        assert!((sq_lo - 2.0).abs() < 2e-3 && (sq_hi - 2.0 * 2f64.sqrt()).abs() < 2e-3);
    }

    /// Through the half turn the rulers move (the shape's top rises and
    /// falls by more than 0.3 cm), so they track the shape rather than
    /// stand still, while the gap between them stays 2.
    #[test]
    fn the_rulers_ride_with_it() {
        let tops: Vec<f64> = (0..=360)
            .map(|i| rulers(&reuleaux(BENCH, HALF_TURN * f64::from(i) / 360.0)).heights[1])
            .collect();
        let (lo, hi) = (
            tops.iter().copied().fold(f64::MAX, f64::min),
            tops.iter().copied().fold(0.0, f64::max),
        );
        assert!(hi - lo > 0.3, "{lo} {hi}");
        let still = rulers(&reuleaux(BENCH, 0.0));
        let turned = reuleaux(BENCH, FRAC_PI_6);
        assert!(
            reach(&turned, [0.0, 1.0]).0 < still.heights[1] - 0.05,
            "fixed guides would stand off it"
        );
    }

    /// The compass draws each arc from its centre corner at radius 2: the
    /// pencil, at every share of the drawing, is a side from its corner.
    #[test]
    fn the_compass_keeps_its_radius() {
        let c = corners(BENCH, 0.0);
        let rim = reuleaux(BENCH, 0.0);
        for i in 0..=300 {
            let drawn = 3.0 * f64::from(i) / 300.0;
            let k = (drawn.floor() as usize).min(2);
            let pencil = rim.parts[k].seg.point(drawn - k as f64);
            assert!((why_apart(c[k], pencil) - SIDE).abs() < 1e-12, "{drawn}");
        }
    }

    /// The shape, its rulers pressed and standing off, keep off the coast
    /// through every turn shown; the coin's rulers too; both inside the
    /// card; the name and the side's length clear of the coast, the rulers
    /// and the lines.
    #[test]
    fn the_bench_keeps_off_the_coast() {
        let shore = shore();
        let name_h = half_height(LABEL_SIZE) / model::CARD_UNIT;
        for i in 0..=60 {
            let t = END_TURN * f64::from(i) / 60.0;
            let c = reuleaux(BENCH, t);
            let r = rulers(&c);
            assert!(near(&rim_points(&c), &shore) > 0.8, "{t}");
            assert!(near(&ruler_points(&r, 0.0), &shore) > 0.5, "{t}");
            assert!(NAME_AT[1] - name_h > r.heights[1] + 0.25, "{t}");
        }
        // the name under the line at 2.45 and the count's row
        let count_low = (sheet::TOP_Y - half_height(sheet::COUNT_SIZE) - model::CARD_CENTRE[1]) / model::CARD_UNIT;
        assert!(NAME_AT[1] + name_h < 2.45 - 0.1 && NAME_AT[1] + name_h < count_low - 0.2);
        let r = rulers(&reuleaux(BENCH, 0.0));
        assert!(near(&ruler_points(&r, STAND_OFF), &shore) > 0.5);
        let coin = rulers(&coin_rim(COIN_AT));
        for off in [0.0, STAND_OFF] {
            assert!(near(&ruler_points(&coin, off), &shore) > 0.3, "{off}");
            for p in ruler_points(&coin, off) {
                assert!(p[0].abs() < CARD_W / 2.0 - 0.3 && p[1].abs() < CARD_H / 2.0 - 0.25);
            }
        }
        // the side's length inside the triangle, clear of its lower side
        // and of the line at 0.45
        let h = half_height(SIDE_SIZE) / model::CARD_UNIT;
        let low_side = corners(BENCH, 0.0)[1][1];
        assert!(SIDE_AT[1] - h > low_side + 0.08 && SIDE_AT[1] + h < BENCH[1] - 0.1);
        assert!(width(&words().unwrap().side, SIDE_SIZE) / model::CARD_UNIT < 1.4);
        assert!(near(&[NAME_AT], &shore) > 1.0);
    }

    /// The countdown at [`THINK_RIGHT`] clears the coin's rulers: its ring
    /// and its note start right of their ends.
    #[test]
    fn the_countdown_clears_the_coin() {
        let coin = rulers(&coin_rim(COIN_AT));
        let right = card_point([coin.sides[1] + RULER_OVER, 0.0])[0];
        let note = width(super::super::draw::PAUSE_NOTE, 18.0) / 2.0;
        assert!(THINK_RIGHT[0] - note > right + 0.5);
        assert!(THINK_RIGHT[0] - super::super::draw::THINK_R > right + 0.8);
        assert!(THINK_RIGHT[0] + note < model::CARD_UNIT * CARD_W / 2.0);
    }

    /// The readout fits the console's top row; the lower third two lines.
    #[test]
    fn the_console_keeps_its_rows() {
        let console = bounds(CONSOLE);
        assert!(TOP_ROW[0] + width(&readout(2.0), LABEL_SIZE) < console[1] - 0.2);
        assert!(TOP_ROW[1] - half_height(LABEL_SIZE) > console[2]);
        assert!(words().unwrap().exact.chars().count() <= 2 * 92);
    }

    /// 14's turns: the half turn ends a corner down, the twelfths a side
    /// upright and then a corner up again, where 15 starts.
    #[test]
    fn the_turns_land_where_the_words_say() {
        let up = |t: f64| {
            corners(BENCH, t)
                .iter()
                .any(|c| (c[0] - BENCH[0]).abs() < 1e-9 && c[1] > BENCH[1])
        };
        let down = |t: f64| {
            corners(BENCH, t)
                .iter()
                .any(|c| (c[0] - BENCH[0]).abs() < 1e-9 && c[1] < BENCH[1])
        };
        assert!(up(0.0) && down(HALF_TURN) && up(END_TURN));
        let c = corners(BENCH, UPRIGHT);
        let upright = (0..3).any(|i| (c[i][0] - c[(i + 1) % 3][0]).abs() < 1e-9);
        assert!(upright);
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("14_1", "Think back to the coin"),
            ("14_1", "it met the lines at two points"),
            ("14_2", "Press two straight rulers"),
            ("14_2", "The gap between them is the coin's width"),
            ("14_2", "Turn the coin"),
            ("14_3", "whichever way you turn it"),
            ("14_4", "Mark three points"),
            ("14_4", "the corners of a triangle"),
            ("14_4", "Put the point of a compass on one corner"),
            ("14_4", "draw an arc from the second corner"),
            ("14_4", "Do the same from each corner"),
            ("14_5", "This curve is called a Reuleaux triangle"),
            ("14_5", "Press the rulers against it"),
            ("14_5", "One touches a corner"),
            ("14_5", "The other touches the arc across from that corner"),
            ("14_5", "so the gap between them is its radius"),
            ("14_6", "Will the gap between them ever change"),
            ("14_8", "The triangle inside it can't do that"),
            ("14_8", "Without its arcs"),
            ("14_8", "comes back to two"),
            ("14_8", "and drops again"),
            ("14_9", "So the Reuleaux triangle"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }
}
