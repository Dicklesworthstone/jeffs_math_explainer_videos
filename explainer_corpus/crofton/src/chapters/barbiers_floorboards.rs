//! Chapter 15, Barbier's floorboards. From 14's end (the Reuleaux triangle
//! a corner up between its pressed rulers, "width 2.00 cm", the coin at
//! rest) the sheet's lines are 2 cm apart, the shape's width, so its rulers
//! hold one gap: one line runs between them, and having no dents the shape
//! is crossed twice, its count "2 crossings" (15_1). Carried and turned to
//! four places, its rulers always hold one gap and the count stays 2
//! (15_2). Almost always: as the sheet slides one gap, a line passes
//! exactly through a corner, and once the rulers lie along two lines and
//! the rim only touches them. Each is a single slide with no chance, and
//! these isolated positions do not change the average (15_3). So it averages two crossings: what is
//! its rim? (15_4, a prediction.) 2 = length ÷ π, so 2π ≈ 6.28 cm, the
//! coin's rim (15_5). Checked another way, exactly: each arc turns through
//! the triangle's sixty degrees, and laid end to end the three make half a
//! circle of radius 2, 2π again (15_6). And from below, by chains with
//! their corners on the rim: 6.00, 6.21 and 6.27 cm, short of it (15_7).
//! Barbier's note of 1860 says that a curve of constant width has a length
//! depending only on its width; a circle as wide makes it π times the width
//! (15_8, the convex fine print below). The count and the laid arcs agree,
//! and the same counting gave the coast back its 25.81 cm, the coast bright
//! again with its crossings (15_9). Counts here are 12's: an arc counts a
//! line that meets it strictly inside its sweep, so a line exactly through
//! a corner (two arcs' end) or touching the rim (where it is tangent) is
//! not counted the usual way, its count left to rounding. Those are single
//! slides with probability zero under a continuous uniform slide. The
//! bounded model sampling below checks proximity to these positions;
//! it does not certify every rendered frame. Starts where chapter 14 ends.

use std::f64::consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_6, PI, TAU};

use explainer_kit::props::{lower_third, precisely};
use explainer_kit::{Kit, Palette, tween};
use fmn::core::rate::{linear, smooth};
use fmn::prelude::*;

use super::a_crowd_of_sticks::{LOW_ROW, TOP_ROW};
use super::draw::{
    card_point, dim_line, fade_in_all, fade_out_all, fail, flat, line_style, now, path, pause, say, stand, think,
    together, wait_until,
};
use super::placement::{HALO_OPACITY, HALO_W, Placement, Pose, at_once, faded_line, glide, tracked};
use super::sheet::{self, DOT_R, LABEL_SIZE};
use super::the_bent_stick;
use super::the_same_width::{
    self, BENCH, Bench, COAST_DIMMED, END_TURN, NAME_AT, RULER_OVER, SIDE, SIDES_SHOWN, Shape, THINK_RIGHT, corners,
    reuleaux, rulers,
};
use super::the_shadow_rule::{ROW_SIZE, row};
use super::why_every_curve::{self, BARBIER_YEAR, COIN_AT, bright_shore, coin_at_rest, coin_rim, count_at, counter};
use crate::facts;
use crate::model::{self, Curve, GAP, P2, Part, Seg};
use crate::palette;

pub struct BarbiersFloorboards {
    pub kit: Kit,
}

// --------------------------------------------------------------- the model

/// 15_2's stops, each a move then a still: where the shape is carried and
/// how far it is turned, degrees (14 leaves it at 240, a corner up). The
/// last brings it home a tenth of a turn's third past a corner up, so its
/// corners lie at three different heights for 15_3.
pub const STOPS: [(P2, f64); 4] = [
    ([-2.9, 1.15], 295.0),
    ([-3.9, 0.05], 365.0),
    ([-2.95, 0.95], 430.0),
    (BENCH, 490.0),
];

/// 15_3's slide of one whole gap, the lines moving down, in three legs:
/// cm and seconds each. The first passes one corner, the second another
/// corner and the touching slide, the third nothing.
pub const LEGS: [(f64, f64); 3] = [(-0.57, 4.0), (-0.43, 4.0), (-1.0, 3.0)];

/// Where 15_6 turns it (a corner down) before the arcs are lifted off,
/// radians, and where they are laid: the middle of the half circle they
/// make, card cm, under the shape.
pub const ARRANGE_TURN: f64 = 3.0 * PI;
pub const HALF_AT: P2 = [-3.4, -0.35];

/// 15_7's chains: how many corners each has on the rim, the same number on
/// each arc (the triangle is the first).
pub const CHAINS: [usize; 3] = [3, 6, 12];

/// The renders' frame rate (the kit's default, `--fps 60`).
pub const FPS: f64 = 60.0;

fn mix(a: f64, b: f64, u: f64) -> f64 {
    a + (b - a) * u
}

/// How close a line of the sheet slid by `slide` (lines across the card)
/// comes to a corner or a ruler of the shape at `at` turned by `turn`, cm:
/// where it is zero the line runs through a corner or along a ruler, and
/// is not counted the usual way.
pub fn exceptional(at: P2, turn: f64, slide: f64) -> f64 {
    let r = rulers(&reuleaux(at, turn));
    corners(at, turn)
        .iter()
        .map(|c| c[1])
        .chain(r.heights)
        .map(|h| {
            let t = (h - slide) / GAP;
            (t - t.round()).abs() * GAP
        })
        .fold(f64::INFINITY, f64::min)
}

/// Whether the lines, sliding from `from` by `by`, carry one over the
/// height `h` (strictly inside the slide).
pub fn passes(h: f64, from: f64, by: f64) -> bool {
    let (lo, hi) = (from.min(from + by), from.max(from + by));
    let r = (h - lo).rem_euclid(GAP);
    r > 0.0 && lo + r < hi
}

/// A bounded grid over a move of `rt` seconds at `fps`, including half
/// steps. This is model sampling, not the renderer's actual frame clock.
fn frame_shares(rt: f64, fps: f64) -> impl Iterator<Item = f64> {
    let n = (rt * fps).round().max(1.0) as usize;
    (0..=2 * n).map(move |j| j as f64 / (2 * n) as f64)
}

/// Counts sampled along the planned moves and slide legs at `fps`, and
/// how near any sampled placement comes to a line through a corner or
/// along a ruler, cm. Actual media requires separate inspection.
fn counts_at_fps(first: Pose, fps: f64) -> (Vec<usize>, f64) {
    let mut seen: Vec<usize> = Vec::new();
    let mut nearest = f64::INFINITY;
    let mut look = |at: P2, turn: f64, slide: f64| {
        let n = count_at(&reuleaux(at, turn), Pose::new(first.beta, slide));
        if !seen.contains(&n) {
            seen.push(n);
        }
        nearest = nearest.min(exceptional(at, turn, slide));
    };
    let (mut at, mut turn) = (BENCH, END_TURN);
    for (to, degrees) in STOPS {
        let goal = degrees.to_radians();
        for u in frame_shares(MOVE_S, fps) {
            let s = smooth(u);
            look(
                [mix(at[0], to[0], s), mix(at[1], to[1], s)],
                mix(turn, goal, s),
                first.slide,
            );
        }
        (at, turn) = (to, goal);
    }
    let mut slide = first.slide;
    for (by, rt) in LEGS {
        for u in frame_shares(rt, fps) {
            look(at, turn, slide + by * u);
        }
        slide += by;
    }
    seen.sort_unstable();
    (seen, nearest)
}

/// As [`counts_at_fps`] at the renders' rate.
pub fn shown_counts(first: Pose) -> (Vec<usize>, f64) {
    counts_at_fps(first, FPS)
}

/// The rim's arcs at `u` (0 to 1) of the way from the shape at `at`,
/// turned by `turn` a corner down, to the upper half of the circle of
/// radius [`SIDE`] round [`HALF_AT`]: each arc's centre carried straight
/// there and the arc turned about it, so it keeps its radius and its sixth
/// of a turn at every `u`. Each goes to the sixth of the half circle
/// nearest its own direction: the arc over the lower corner stays level,
/// the other two turn a sixth of a turn up.
pub fn laid(at: P2, turn: f64, u: f64) -> Curve {
    let goals = [FRAC_PI_6, FRAC_PI_2, 5.0 * FRAC_PI_6];
    let mut parts = Vec::with_capacity(3);
    let mut start = 0.0;
    for part in reuleaux(at, turn).parts {
        if let Seg::Arc {
            centre,
            radius,
            from,
            to,
        } = part.seg
        {
            let mid = ((from + to) / 2.0).rem_euclid(TAU);
            let d = goals
                .iter()
                .map(|&g| (g - mid + PI).rem_euclid(TAU) - PI)
                .min_by(|a, b| a.abs().total_cmp(&b.abs()))
                .unwrap_or(0.0);
            let seg = Seg::Arc {
                centre: [mix(centre[0], HALF_AT[0], u), mix(centre[1], HALF_AT[1], u)],
                radius,
                from: from + d * u,
                to: to + d * u,
            };
            parts.push(Part { seg, start });
            start += seg.len();
        }
    }
    Curve { parts, length: start }
}

/// A closed chain with `count` corners on the rim of the shape at `at`
/// turned by `turn`, the same number evenly along each arc from its start
/// (the chain of three is the triangle).
pub fn chain(at: P2, turn: f64, count: usize) -> Curve {
    let per = (count / 3).max(1);
    let mut points: Vec<P2> = reuleaux(at, turn)
        .parts
        .iter()
        .flat_map(|p| (0..per).map(move |j| p.seg.point(j as f64 / per as f64)))
        .collect();
    points.push(points[0]);
    let mut parts = Vec::with_capacity(points.len() - 1);
    let mut start = 0.0;
    for w in points.windows(2) {
        let seg = Seg::Line { from: w[0], to: w[1] };
        parts.push(Part { seg, start });
        start += seg.len();
    }
    Curve { parts, length: start }
}

/// The corner leg `i` of 15_3 passes, with the shape home after 15_2.
fn leg_corners(first: Pose, i: usize) -> Vec<usize> {
    let turn = STOPS[STOPS.len() - 1].1.to_radians();
    let from = first.slide + LEGS[..i].iter().map(|l| l.0).sum::<f64>();
    let rulers_at = rulers(&reuleaux(BENCH, turn)).touches;
    corners(BENCH, turn)
        .iter()
        .enumerate()
        .filter(|(_, c)| rulers_at.iter().all(|t| apart(**c, *t) > 1e-9))
        .filter(|(_, c)| passes(c[1], from, LEGS[i].0))
        .map(|(k, _)| k)
        .collect()
}

/// Whether leg `i` passes the touching slide (both rulers on lines).
fn leg_touches(first: Pose, i: usize) -> bool {
    let turn = STOPS[STOPS.len() - 1].1.to_radians();
    let from = first.slide + LEGS[..i].iter().map(|l| l.0).sum::<f64>();
    passes(rulers(&reuleaux(BENCH, turn)).heights[0], from, LEGS[i].0)
}

fn apart(a: P2, b: P2) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// How many directions the average is found over (a midpoint rule over
/// direction, exact over the slide).
pub const SHARE_DIRECTIONS: usize = 3600;

// ------------------------------------------------------------------ words

/// The chapter's words that carry numbers, from the facts. Refused unless
/// the model gives what the screen and the voice say: the lines as far
/// apart as the shape is wide; at home two crossings and one line between
/// the rulers; every frame of the wander and the slide counting 2, none on
/// a line through a corner or along a ruler; the slide's legs passing a
/// corner, then a corner and the touching slide, then nothing, a whole gap
/// in all; its average 2 at every direction found, so its rim 2π, the
/// coin's; three arcs of sixty degrees, laid end to end over half a turn;
/// the chains' lengths rising and short of the rim; 1860; and the coast's
/// average times π giving its length.
pub struct Words {
    pub gap: String,
    pub convex: String,
    pub chance: String,
    pub average: String,
    pub solve: String,
    pub length: String,
    pub coin: String,
    pub angle: String,
    pub turns: String,
    pub laid: String,
    pub chains: Vec<String>,
    pub source: String,
    pub barbier: String,
    pub loops: String,
    pub rim: String,
}

const SOURCE_LABEL: &str = "source:";
const LOOPS: &str = "the rim of a convex shape, its width read off rulers in every direction. A closed curve with \
                     the same supporting rulers in every direction but inward detours is longer";
const CONVEX: &str = "convex: every straight segment between two points of the shape stays inside it, so a line \
                      through its inside meets the rim at two points";
const CHANCE: &str = "a line exactly through a corner, or two lines lying along the rulers, is a single slide \
                      among a whole gap of them: it has no chance, and changes no average";
const TURNS: &str = r"3 \times \tfrac{1}{6}\text{ turn} = \tfrac{1}{2}\text{ turn}";
const BARBIER: &str = r"\text{length} = \pi \times \text{width}";

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
    let number = |s: &str| s.parse::<f64>().map_err(|e| format!("{s}: {e}"));
    let coast = model::coast();
    let first = Pose::new(0.0, sheet::shown_slide(&coast, 0.0));

    // 15_1: the gap is the width; at home, one line between the rulers and
    // two crossings
    let gap = agree("sheet.gap", fixed(GAP, 0))?;
    let side = agree("reuleaux.width", fixed(SIDE, 0))?;
    if (SIDE - GAP).abs() > 1e-12 || gap != side {
        return Err(format!(
            "15_1 says the lines are {gap} cm apart, the shape's width {side}"
        ));
    }
    let home = reuleaux(BENCH, END_TURN);
    agree("reuleaux.home_count", count_at(&home, first).to_string())?;
    let r = rulers(&home);
    let between = (-4..=4)
        .filter(|&k| (r.heights[0]..r.heights[1]).contains(&(first.slide + f64::from(k) * GAP)))
        .count();
    agree("reuleaux.lines_between", between.to_string())?;

    // 15_2 and 15_3: every frame shown counts 2, none at an exceptional
    // slide; the legs pass what the voice names; a whole gap
    let (seen, nearest) = shown_counts(first);
    let seen: Vec<String> = seen.iter().map(usize::to_string).collect();
    agree("reuleaux.wander_counts", seen.join(","))?;
    if nearest < 1e-6 {
        return Err(format!(
            "15_3: a frame comes {nearest:.1e} cm from a line through a corner or a ruler"
        ));
    }
    let passed: Vec<(usize, bool)> = (0..LEGS.len())
        .map(|i| (leg_corners(first, i).len(), leg_touches(first, i)))
        .collect();
    if passed != [(1, false), (1, true), (0, false)] {
        return Err(format!("15_3's legs pass {passed:?} corners and touches"));
    }
    if (LEGS.iter().map(|l| l.0).sum::<f64>() + GAP).abs() > 1e-12 {
        return Err("15_3 slides the sheet one whole gap".into());
    }

    // 15_4 and 15_5: its average 2, every stretch counting 2; so its rim
    // is 2π, the coin's
    let shares = home.count_shares(SHARE_DIRECTIONS, Some(GAP));
    let mean = model::moments(&shares).0;
    let two = agree("reuleaux.mean_crossings", fixed(mean, 0))?;
    if shares.iter().enumerate().any(|(k, &p)| k != 2 && p > 1e-12) {
        return Err("15_4 says it averages two because it counts two almost everywhere".into());
    }
    let rim = agree("reuleaux.rim", fixed(home.length, 2))?;
    let coin = agree("coin.rim", fixed(coin_rim(COIN_AT).length, 2))?;
    if (mean * model::worth(GAP) - home.length).abs() > 1e-9 || rim != coin || fixed(number(&two)? * PI, 2) != rim {
        return Err(format!("15_5 says {two} × π ≈ {rim}, the coin's {coin}"));
    }

    // 15_6: three arcs of sixty degrees; laid end to end, half a turn round
    // one middle
    let arcs = agree("reuleaux.arcs", home.parts.len().to_string())?;
    let mut spans: Vec<(f64, f64)> = Vec::new();
    for part in laid(BENCH, ARRANGE_TURN, 1.0).parts {
        if let Seg::Arc {
            centre,
            radius,
            from,
            to,
        } = part.seg
        {
            if apart(centre, HALF_AT) > 1e-12 || (radius - SIDE).abs() > 1e-12 {
                return Err("15_6 lays every arc on one circle of radius 2".into());
            }
            // where it starts, between −π and π, so a start on 0 stays first
            spans.push(((from + PI).rem_euclid(TAU) - PI, (to - from).abs()));
        }
    }
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));
    let degrees = agree("reuleaux.arc_degrees", fixed(spans[0].1.to_degrees(), 0))?;
    let mut end = 0.0;
    for &(from, span) in &spans {
        if (from - end).abs() > 1e-9 {
            return Err(format!("15_6's arcs leave a gap at {end:.6}"));
        }
        end = from + span;
    }
    if (end - PI).abs() > 1e-9 || (number(&arcs)? * number(&degrees)? - 180.0).abs() > 1e-9 {
        return Err(format!("15_6 says {arcs} × {degrees} degrees make half a turn"));
    }
    let laid_row = format!(r"{arcs} \times \tfrac{{1}}{{6}} \times 2\pi \times {side} = 2\pi");

    // 15_7: the chains' lengths, rising and short of the rim; the triangle
    // is the first
    let turned = reuleaux(BENCH, ARRANGE_TURN);
    let lengths: Vec<f64> = CHAINS.iter().map(|&n| chain(BENCH, ARRANGE_TURN, n).length).collect();
    let corners_key: Vec<String> = CHAINS.iter().map(usize::to_string).collect();
    agree("reuleaux.chain_corners", corners_key.join(","))?;
    let shown: Vec<String> = lengths.iter().map(|&l| fixed(l, 2)).collect();
    agree("reuleaux.chain_lengths", shown.join(","))?;
    let plain = the_same_width::triangle(BENCH, ARRANGE_TURN).length;
    agree("triangle.perimeter", fixed(plain, 0))?;
    if lengths.windows(2).any(|w| w[1] <= w[0])
        || lengths.iter().any(|&l| l >= turned.length)
        || (lengths[0] - plain).abs() > 1e-12
    {
        return Err("15_7 says the chains rise, short of the rim, from the triangle".into());
    }

    // 15_8 and 15_9: Barbier's year; the coast's average times π
    agree("barbier.year", BARBIER_YEAR.to_string())?;
    agree("crofton.mean_crossings_4dp", fixed(coast.length / PI, 4))?;
    agree("coast.length_2dp", fixed(coast.length, 2))?;

    Ok(Words {
        gap: format!("{gap} cm"),
        convex: CONVEX.into(),
        chance: CHANCE.into(),
        average: format!(r"\text{{average count}} = {two}"),
        solve: format!(r"{two} = \text{{length}} \div \pi"),
        length: format!(r"\text{{length}} = {two}\pi \approx {rim}\text{{ cm}}"),
        coin: format!("rim {coin} cm"),
        angle: format!("{degrees}°"),
        turns: TURNS.into(),
        laid: laid_row,
        chains: CHAINS
            .iter()
            .zip(&shown)
            .map(|(n, l)| format!("{n} corners, {l} cm"))
            .collect(),
        source: format!(
            "Barbier, {BARBIER_YEAR}, J. Math. Pures Appl., pages 283 to 285: for curves of constant width, the \
             length depends only on the width"
        ),
        barbier: BARBIER.into(),
        loops: LOOPS.into(),
        rim: format!(r"{two} \times \pi \approx {rim}\text{{ cm}}"),
    })
}

// ------------------------------------------------------------------ layout

/// 15_1's mark of one gap, between the lines at 0.45 and 2.45, left of the
/// shape, its ticks' half width and its label, card cm.
const GAP_X: f64 = -5.2;
const GAP_TICK: f64 = 0.12;
const GAP_LABEL_AT: P2 = [-5.75, 1.45];
const MARK_W: f64 = 2.5;
/// The coin's rim under it (15_5), card cm.
const COIN_LABEL_AT: P2 = [2.0, 0.95];
const COIN_LABEL_SIZE: f64 = 20.0;
/// 15_3's gold rings, frame.
const RING_R: f64 = 0.17;
/// 15_6's angle mark at the lower corner and its label above it, card cm.
const ANGLE_R: f64 = 0.45;
const ANGLE_LABEL_UP: f64 = 0.8;
const ANGLE_SIZE: f64 = 20.0;
const RIM_W: f64 = 4.0;
const CHAIN_W: f64 = 3.0;
/// How much lighter 15_6 draws the second and third arcs, so the three
/// show apart.
const SHADES: [f64; 3] = [0.0, 0.25, 0.5];

// ------------------------------------------------------------------ timing

const FADE_S: f64 = 0.4;
const MOVE_S: f64 = 2.5;
const STILL_S: f64 = 1.2;
const TURN_S: f64 = 2.0;
const LAY_S: f64 = 4.0;

// --------------------------------------------------------- what is drawn

/// The shape's crossings at the placement shown, gold dots, at the opacity
/// `alpha` holds. In the scene.
fn live_rim_dots(stage: &mut Stage<'_>, shape: Shape, place: Placement, alpha: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let (o, p) = (tracked(s, alpha), place.pose(s));
        let dots: Vec<VMobject> = sheet::crossings(&shape.rim_curve(s), p.beta, p.slide)
            .iter()
            .map(|c| {
                Circle::new()
                    .radius(DOT_R)
                    .arc_center(card_point(c.at))
                    .style(flat(palette::touch(), o))
                    .build()
            })
            .collect();
        s.add(v_group(dots))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The shape's count at the placement shown.
fn rim_count(shape: Shape, place: Placement) -> impl Fn(&fmn::mobject::Stage) -> usize + Clone + 'static {
    move |s| count_at(&shape.rim_curve(s), place.pose(s))
}

/// A halo along every line strictly between the shape's rulers, across
/// them, at the opacity `alpha` holds (the lines run across the card in
/// this chapter). In the scene.
fn live_between(stage: &mut Stage<'_>, shape: Shape, place: Placement, alpha: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let (o, p) = (tracked(s, alpha), place.pose(s));
        let r = rulers(&shape.rim_curve(s));
        let [left, right] = [r.sides[0] - RULER_OVER, r.sides[1] + RULER_OVER];
        let k0 = ((r.heights[0] - p.slide) / GAP).floor() as i64;
        let halos: Vec<VMobject> = (k0..=k0 + 2)
            .map(|k| p.slide + k as f64 * GAP)
            .filter(|&h| h > r.heights[0] && h < r.heights[1])
            .map(|h| {
                polyline(vec![card_point([left, h]), card_point([right, h])])
                    .style(faded_line(Palette::ink(), HALO_W, HALO_OPACITY * o))
                    .build()
            })
            .collect();
        s.add(v_group(halos))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The arcs laid toward the half circle, `u` of the way (from 15_6's
/// pose), the second and third lighter by `shade`'s share of [`SHADES`],
/// at the opacity `alpha` holds. In the scene.
fn live_laid(stage: &mut Stage<'_>, u: Mob, shade: Mob, alpha: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let (o, k) = (tracked(s, alpha), tracked(s, shade));
        let at = s.tracker_value(u).unwrap_or(0.0);
        let arcs: Vec<VMobject> = laid(BENCH, ARRANGE_TURN, at)
            .parts
            .iter()
            .zip(SHADES)
            .map(|(p, lighter)| {
                let pts: Vec<Vec3> = (0..=40).map(|j| card_point(p.seg.point(f64::from(j) / 40.0))).collect();
                polyline(pts)
                    .style(faded_line(palette::coin(), RIM_W, o * (1.0 - lighter * k)))
                    .build()
            })
            .collect();
        s.add(v_group(arcs))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// A gold ring round a card point. Detached.
fn ring(stage: &mut Stage<'_>, at: P2) -> Mob {
    let r = Circle::new()
        .radius(RING_R)
        .arc_center(card_point(at))
        .style(line_style(palette::touch(), 3.0))
        .build();
    stage.arena_mut().add(r)
}

/// 15_1's mark of one gap: the upright between two lines and its ticks.
/// Detached.
fn gap_mark(stage: &mut Stage<'_>, low: f64) -> Vec<Mob> {
    let high = low + GAP;
    vec![
        path(stage, &[[GAP_X, low], [GAP_X, high]], Palette::ink(), MARK_W),
        path(
            stage,
            &[[GAP_X - GAP_TICK, low], [GAP_X + GAP_TICK, low]],
            Palette::ink(),
            MARK_W,
        ),
        path(
            stage,
            &[[GAP_X - GAP_TICK, high], [GAP_X + GAP_TICK, high]],
            Palette::ink(),
            MARK_W,
        ),
    ]
}

/// 15_6's two radii from the lower corner to the others, and the sixty
/// degrees between them. Detached.
fn angle_marks(stage: &mut Stage<'_>) -> (Vec<Mob>, P2) {
    let c = corners(BENCH, ARRANGE_TURN);
    let low = (0..3).min_by(|&a, &b| c[a][1].total_cmp(&c[b][1])).unwrap_or(0);
    let (p, q) = (c[(low + 1) % 3], c[(low + 2) % 3]);
    let o = c[low];
    let arc: Vec<P2> = (0..=24)
        .map(|j| {
            let a = FRAC_PI_3 + FRAC_PI_3 * f64::from(j) / 24.0;
            [o[0] + ANGLE_R * a.cos(), o[1] + ANGLE_R * a.sin()]
        })
        .collect();
    let marks = vec![
        path(stage, &[o, p], Palette::ink(), MARK_W),
        path(stage, &[o, q], Palette::ink(), MARK_W),
        path(stage, &arc, Palette::ink(), MARK_W),
    ];
    (marks, [o[0], o[1] + ANGLE_LABEL_UP])
}

/// The sheet slid `by` from where it stands, `rt` seconds at an even rate,
/// then named the controls' way.
fn slide(stage: &mut Stage<'_>, place: Placement, by: f64, rt: f64) -> fmn::Result<()> {
    place.begin(stage, 0.0, by, [0.0, 0.0])?;
    place.run(stage, &[(1.0, rt)], linear)?;
    place.settle(stage).map(|_| ())
}

impl SceneConstruct for BarbiersFloorboards {
    fn name(&self) -> &str {
        "15_barbiers_floorboards"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let coast = model::coast();
        let first = Pose::new(0.0, sheet::shown_slide(&coast, 0.0));
        let words = words().map_err(fail)?;
        let named = the_same_width::words().map_err(fail)?;
        let earlier = why_every_curve::words().map_err(fail)?;
        let bent = the_bent_stick::words().map_err(fail)?;

        // where 14 ended: the shore with the coast dimmed, the coin at rest,
        // the shape a corner up between its pressed rulers, its name
        let shore = bright_shore(&kit, stage, &coast, first, false)?;
        for &m in &shore.pieces {
            stage.set_color_lanes(m, "stroke_rgba", None, Some(COAST_DIMMED), true);
        }
        let place = shore.place;
        for m in coin_at_rest(stage, COIN_AT) {
            stage.add_to_scene(m)?;
        }
        let bench = Bench::new(&kit, stage)?;
        bench.as_14_ends(stage)?;
        let shape = bench.shape;
        let name = kit.text(stage, &named.name, LABEL_SIZE, palette::coin(), card_point(NAME_AT))?;
        stage.add_to_scene(name)?;
        let [between, dots, counts, laid_u, shade, laid_alpha] =
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0].map(|x| stage.add_value_tracker(x));
        live_between(stage, shape, place, between)?;
        live_rim_dots(stage, shape, place, dots)?;
        counter(&kit, stage, rim_count(shape, place), counts)?;
        live_laid(stage, laid_u, shade, laid_alpha)?;

        // 15_1: the lines a width apart; one gap between the rulers, one
        // line; no dents, so it is crossed twice
        let line = say(&kit, stage, "15_1")?;
        line.wait_for(stage, "Now look at the sheet's lines")?;
        fade_out_all(stage, &[name], FADE_S)?;
        line.wait_for(stage, "They're two centimetres apart")?;
        let mut marks = gap_mark(stage, first.slide);
        marks.push(kit.text(stage, &words.gap, LABEL_SIZE, Palette::ink(), card_point(GAP_LABEL_AT))?);
        fade_in_all(stage, &marks, FADE_S)?;
        line.wait_for(stage, "One line runs between them")?;
        stage.play(tween(between, 1.0, FADE_S)?)?;
        line.wait_for(stage, "It has no dents")?;
        let convex = precisely(&kit, stage, &words.convex)?;
        fade_in_all(stage, &[convex], FADE_S)?;
        let convex_read = now(stage) + words.convex.chars().count() as f64 / 15.0;
        line.wait_for(stage, "and crosses the rim twice")?;
        together(stage, vec![tween(dots, 1.0, FADE_S)?, tween(counts, 1.0, FADE_S)?])?;

        // 15_2: carried and turned to four places, each a still; the count
        // and the line between the rulers follow it
        wait_until(stage, convex_read)?;
        let line = say(&kit, stage, "15_2")?;
        line.wait_for(stage, "Slide it and turn it")?;
        marks.push(convex);
        fade_out_all(stage, &marks, FADE_S)?;
        for (at, degrees) in STOPS {
            together(stage, shape.moves(at, degrees.to_radians(), MOVE_S)?)?;
            pause(stage, STILL_S)?;
        }

        // 15_3: almost always: the sheet slides a whole gap in three legs,
        // past a corner (ringed), past the touching slide (its touches
        // ringed), then on; the exact statement below
        let line = say(&kit, stage, "15_3")?;
        let home = STOPS[STOPS.len() - 1];
        let turn = home.1.to_radians();
        let [corner] = leg_corners(first, 0)[..] else {
            return Err(fail("15_3's first leg passes one corner".into()));
        };
        line.wait_for(stage, "As the sheet slides")?;
        let lit = ring(stage, corners(BENCH, turn)[corner]);
        fade_in_all(stage, &[lit], FADE_S)?;
        slide(stage, place, LEGS[0].0, LEGS[0].1)?;
        line.wait_for(stage, "for that one slide")?;
        fade_out_all(stage, &[lit], FADE_S)?;
        line.wait_for(stage, "And once in every gap")?;
        let touches = rulers(&reuleaux(BENCH, turn)).touches.map(|t| ring(stage, t));
        fade_in_all(stage, &touches, FADE_S)?;
        slide(stage, place, LEGS[1].0, LEGS[1].1)?;
        line.wait_for(stage, "and the rim only touches them")?;
        fade_out_all(stage, &touches, FADE_S)?;
        slide(stage, place, LEGS[2].0, LEGS[2].1)?;
        line.wait_for(stage, "Each of those is one slide")?;
        let chance = precisely(&kit, stage, &words.chance)?;
        fade_in_all(stage, &[chance], FADE_S)?;
        let chance_read = now(stage) + words.chance.chars().count() as f64 / 15.0;

        // 15_4: it averages two: what is its rim?
        wait_until(stage, chance_read)?;
        let line = say(&kit, stage, "15_4")?;
        line.wait_for(stage, "So like the coin")?;
        let theorem = row(&kit, stage, &earlier.theorem, Palette::ink(), TOP_ROW)?;
        at_once(
            stage,
            &[theorem],
            &[chance],
            vec![tween(bench.press.readout, 0.0, FADE_S)?],
            FADE_S,
        )?;
        line.wait_for(stage, "it averages two crossings")?;
        let average = row(&kit, stage, &words.average, Palette::ink(), LOW_ROW)?;
        fade_in_all(stage, &[average], FADE_S)?;
        line.wait_for(stage, "What does that make the length of its rim")?;
        let q = think(&kit, stage, THINK_RIGHT)?;

        // 15_5: 2 = length ÷ π, so 2π ≈ 6.28 cm, the coin's rim
        let line = say(&kit, stage, "15_5")?;
        let solve = row(&kit, stage, &words.solve, Palette::ink(), LOW_ROW)?;
        at_once(stage, &[solve], &[q, average], vec![], FADE_S)?;
        line.wait_for(stage, "about six point two eight")?;
        let length = kit.tex(stage, &words.length, ROW_SIZE, palette::length(), &[], LOW_ROW)?;
        stage.next_to(length, solve, RIGHT, 0.5, ORIGIN);
        fade_in_all(stage, &[length], FADE_S)?;
        line.wait_for(stage, "the same as the coin's rim")?;
        let coin_label = kit.text(
            stage,
            &words.coin,
            COIN_LABEL_SIZE,
            palette::coin(),
            card_point(COIN_LABEL_AT),
        )?;
        fade_in_all(stage, &[coin_label], FADE_S)?;

        // 15_6: another way: the rows, rulers, count and dots go and it
        // turns a corner down; the radii and the sixty degrees; the arcs
        // lifted off and laid end to end, half a circle of radius 2
        let line = say(&kit, stage, "15_6")?;
        line.wait_for(stage, "Check it another way")?;
        let mut moves = bench.press.shown(0.0, FADE_S)?;
        moves.extend([
            tween(dots, 0.0, FADE_S)?,
            tween(counts, 0.0, FADE_S)?,
            tween(between, 0.0, FADE_S)?,
        ]);
        at_once(stage, &[], &[theorem, solve, length, coin_label], moves, FADE_S)?;
        together(stage, shape.moves(BENCH, ARRANGE_TURN, TURN_S)?)?;
        line.wait_for(stage, "Each arc is centred on a corner")?;
        let (mut angle, label_at) = angle_marks(stage);
        let radii = vec![angle[0], angle[1]];
        fade_in_all(stage, &radii, FADE_S)?;
        line.wait_for(stage, "make a triangle with equal sides")?;
        stage.play(tween(shape.sides, SIDES_SHOWN, FADE_S)?)?;
        line.wait_for(stage, "so its angles are each sixty degrees")?;
        let label = kit.text(stage, &words.angle, ANGLE_SIZE, Palette::ink(), card_point(label_at))?;
        fade_in_all(stage, &[angle[2], label], FADE_S)?;
        angle.push(label);
        line.wait_for(stage, "Lift the three arcs off")?;
        // the laid arcs replace the rim where it lies, the same strokes
        stage.set_tracker_value(laid_alpha, 1.0)?;
        stage.set_tracker_value(shape.rim, 0.0)?;
        at_once(
            stage,
            &[],
            &angle,
            vec![tween(shape.sides, 0.0, FADE_S)?, tween(shade, 1.0, FADE_S)?],
            FADE_S,
        )?;
        line.wait_for(stage, "lay them end to end")?;
        stage.play(glide(laid_u, 1.0, LAY_S, smooth)?)?;
        line.wait_for(stage, "make half of that circle")?;
        let turns = row(&kit, stage, &words.turns, Palette::ink(), TOP_ROW)?;
        fade_in_all(stage, &[turns], FADE_S)?;
        line.wait_for(stage, "So half of it is two pi again")?;
        let laid_row = row(&kit, stage, &words.laid, palette::length(), LOW_ROW)?;
        fade_in_all(stage, &[laid_row], FADE_S)?;

        // 15_7: the arcs back on the rim; chains with corners on it, the
        // triangle first, each short of it
        let line = say(&kit, stage, "15_7")?;
        line.wait_for(stage, "And a third way")?;
        at_once(
            stage,
            &[],
            &[turns, laid_row],
            vec![glide(laid_u, 0.0, LAY_S, smooth)?],
            FADE_S,
        )?;
        stage.play(tween(shade, 0.0, FADE_S)?)?;
        stage.set_tracker_value(shape.rim, 1.0)?;
        stage.set_tracker_value(laid_alpha, 0.0)?;
        let drawn: Vec<Mob> = CHAINS
            .iter()
            .map(|&n| {
                let c = chain(BENCH, ARRANGE_TURN, n);
                let mut pts = vec![c.at(0.0)];
                pts.extend(c.parts.iter().map(|p| p.seg.point(1.0)));
                path(stage, &pts, palette::stick(), CHAIN_W)
            })
            .collect();
        let rows: Vec<Mob> = words
            .chains
            .iter()
            .map(|w| {
                let m = kit.text(stage, w, LABEL_SIZE, Palette::ink(), TOP_ROW)?;
                stage.move_to(m, TOP_ROW, LEFT);
                Ok(m)
            })
            .collect::<fmn::Result<_>>()?;
        line.wait_for(stage, "The triangle itself is such a chain")?;
        fade_in_all(stage, &[drawn[0], rows[0]], FADE_S)?;
        for (k, phrase) in [(1, "With six corners"), (2, "and with twelve")] {
            line.wait_for(stage, phrase)?;
            at_once(
                stage,
                &[drawn[k], rows[k]],
                &[drawn[k - 1], rows[k - 1]],
                vec![],
                FADE_S,
            )?;
        }

        // 15_8: Barbier's note, for every convex curve of constant width; a
        // circle as wide, the coin, gives π times the width; the convex fine
        // print in place of the source
        let line = say(&kit, stage, "15_8")?;
        line.wait_for(stage, "Barbier made the general point")?;
        let source = lower_third(&kit, stage, SOURCE_LABEL, &words.source)?;
        at_once(stage, &[source], &[drawn[2], rows[2]], vec![], FADE_S)?;
        let source_read = now(stage) + words.source.chars().count() as f64 / 15.0;
        line.wait_for(stage, "A circle of the same width")?;
        let coin_again = kit.text(
            stage,
            &words.coin,
            COIN_LABEL_SIZE,
            palette::coin(),
            card_point(COIN_LABEL_AT),
        )?;
        fade_in_all(stage, &[coin_again], FADE_S)?;
        line.wait_for(stage, "pi times the width")?;
        let barbier = row(&kit, stage, &words.barbier, palette::length(), TOP_ROW)?;
        fade_in_all(stage, &[barbier], FADE_S)?;
        line.wait_for(stage, "Ours is two centimetres wide")?;
        wait_until(stage, source_read)?;
        let loops = precisely(&kit, stage, &words.loops)?;
        at_once(stage, &[loops], &[source], vec![], FADE_S)?;
        let loops_read = now(stage) + words.loops.chars().count() as f64 / 15.0;

        // 15_9: the count and the arcs agree; the coast bright again, its
        // crossings and 13's row. Its crossings come back, not its count:
        // this placement's seven is not the average the row uses
        wait_until(stage, loops_read)?;
        let line = say(&kit, stage, "15_9")?;
        line.wait_for(stage, "So counting crossings and laying out the arcs agree")?;
        let rim = row(&kit, stage, &words.rim, palette::length(), TOP_ROW)?;
        let bright: Vec<AnimBuilder> = shore
            .pieces
            .iter()
            .map(|&m| dim_line(m, 1.0, FADE_S))
            .collect::<fmn::Result<_>>()?;
        at_once(stage, &[rim], &[loops, coin_again, barbier], bright, FADE_S)?;
        line.wait_for(stage, "On our coast")?;
        let back = row(&kit, stage, &bent.coast, palette::length(), LOW_ROW)?;
        at_once(stage, &[back], &[], vec![tween(shore.dots_alpha, 1.0, FADE_S)?], FADE_S)?;
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
    use crate::model::{CARD_H, CARD_UNIT, CARD_W};

    fn first() -> Pose {
        Pose::new(0.0, sheet::shown_slide(&model::coast(), 0.0))
    }

    fn half_height(size: f64) -> f64 {
        size / 150.0
    }

    fn width(text: &str, size: f64) -> f64 {
        text.chars().count() as f64 * size / 150.0
    }

    /// A TeX row's shown characters, roughly: its commands dropped.
    fn shown(tex: &str) -> String {
        tex.replace(r"\text", "")
            .replace(r"\tfrac{1}{6}", "⅙")
            .replace(r"\tfrac{1}{2}", "½")
            .replace(r"\div", "÷")
            .replace(r"\times", "×")
            .replace(r"\approx", "≈")
            .replace(r"\pi", "π")
            .replace(['{', '}', '\\'], "")
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
            .flat_map(|&p| shore.iter().map(move |&q| apart(p, q)))
            .fold(f64::MAX, f64::min)
    }

    fn points(c: &Curve) -> Vec<P2> {
        c.parts
            .iter()
            .flat_map(|p| (0..=30).map(move |k| p.seg.point(f64::from(k) / 30.0)))
            .collect()
    }

    fn polygon(points: &[P2]) -> Curve {
        let mut parts = Vec::new();
        let mut start = 0.0;
        for w in points.windows(2) {
            let seg = Seg::Line { from: w[0], to: w[1] };
            parts.push(Part { seg, start });
            start += seg.len();
        }
        Curve { parts, length: start }
    }

    #[test]
    fn the_screen_words_are_built_from_the_facts() {
        let w = words().unwrap();
        assert_eq!(w.gap, "2 cm");
        assert_eq!(w.average, r"\text{average count} = 2");
        assert_eq!(w.solve, r"2 = \text{length} \div \pi");
        assert_eq!(w.length, r"\text{length} = 2\pi \approx 6.28\text{ cm}");
        assert_eq!(w.coin, "rim 6.28 cm");
        assert_eq!(w.angle, "60°");
        assert_eq!(w.laid, r"3 \times \tfrac{1}{6} \times 2\pi \times 2 = 2\pi");
        assert_eq!(
            w.chains,
            ["3 corners, 6.00 cm", "6 corners, 6.21 cm", "12 corners, 6.27 cm"]
        );
        assert_eq!(w.rim, r"2 \times \pi \approx 6.28\text{ cm}");
    }

    /// At home, a corner up, one line between the rulers and two crossings;
    /// every stop of the wander keeps its corners and rulers more than
    /// 0.12 cm from a line, and counts 2.
    #[test]
    fn the_stops_keep_clear_of_the_lines() {
        let f = first();
        assert!(exceptional(BENCH, END_TURN, f.slide) > 0.5);
        for (at, degrees) in STOPS {
            let t = degrees.to_radians();
            assert!(exceptional(at, t, f.slide) > 0.12, "{at:?} {degrees}");
            assert_eq!(count_at(&reuleaux(at, t), f), 2, "{at:?} {degrees}");
        }
        // the last stop is a tenth of a third of a turn past a corner up
        let last = STOPS[3].1.to_radians().rem_euclid(TAU / 3.0);
        assert!((last - 10f64.to_radians()).abs() < 1e-12);
    }

    /// Every frame of the wander and the slide counts 2 at 60, 30 and 24
    /// frames a second, the nearest coming more than a millionth of a
    /// centimetre from a line through a corner or along a ruler.
    #[test]
    fn every_frame_shown_counts_two() {
        for fps in [60.0, 30.0, 24.0, 15.0] {
            let (seen, nearest) = counts_at_fps(first(), fps);
            assert_eq!(seen, [2], "{fps}");
            assert!(nearest > 1e-6, "{fps}: {nearest}");
        }
    }

    /// The exceptional slides the voice names are real: the first leg's
    /// line through a corner passes through the rim there (the rim just
    /// before the corner and just after it lie either side of the line),
    /// where two arcs end, so the strict count, which takes an arc's
    /// crossing only strictly inside it, leaves that instant to rounding;
    /// at the touching slide both rulers lie on lines, a gap apart, and the
    /// rim stays between them, only touching. A millionth of a centimetre
    /// either side of each the count is 2, and no frame is shown nearer.
    /// The first leg passes one corner, the second one corner and the
    /// touching slide, the third neither.
    #[test]
    fn the_exceptional_slides_are_single_slides() {
        let f = first();
        let turn = STOPS[3].1.to_radians();
        let shape = reuleaux(BENCH, turn);
        let [k] = leg_corners(f, 0)[..] else {
            panic!("one corner")
        };
        let corner = corners(BENCH, turn)[k];
        // arc k + 1 ends at corner k and arc k + 2 starts there
        let (before, after) = (shape.parts[(k + 1) % 3].seg, shape.parts[(k + 2) % 3].seg);
        assert!(apart(before.point(1.0), corner) < 1e-12 && apart(after.point(0.0), corner) < 1e-12);
        let (y0, y1) = (
            before.point(1.0 - 1e-3)[1] - corner[1],
            after.point(1e-3)[1] - corner[1],
        );
        assert!(y0 * y1 < 0.0, "{y0} {y1}");
        let r = rulers(&shape);
        assert!((r.width() - GAP).abs() < 1e-12);
        assert!(
            points(&shape)
                .iter()
                .all(|p| p[1] > r.heights[0] - 1e-12 && p[1] < r.heights[1] + 1e-12)
        );
        for h in [corner[1], r.heights[0]] {
            for d in [-1e-6, 1e-6] {
                assert_eq!(count_at(&shape, Pose::new(0.0, h + d)), 2, "{h} {d}");
            }
        }
        assert!(leg_touches(f, 1) && !leg_touches(f, 0) && !leg_touches(f, 2));
        assert_eq!(leg_corners(f, 1).len(), 1);
        assert!(leg_corners(f, 2).is_empty());
    }

    /// Its average over every slide and 3,600 directions is 2, every
    /// stretch counting 2, so on this sheet its rim is 2π. Controls: the
    /// plain triangle averages 6 ÷ π and sometimes misses; a 2 cm square,
    /// 8 ÷ π, sometimes 4; a rim with one arc dented inward keeps 2π of rim
    /// and its 2 on average but is crossed 4 times at some placements (no
    /// dents is needed for "twice"); lines 1 and 4 cm apart give 4 and 1,
    /// each times that sheet's worth 2π, times this sheet's π not.
    #[test]
    fn its_average_is_two_and_its_rim_two_pi() {
        let shape = reuleaux([0.0, 0.0], 0.3);
        let sh = shape.count_shares(SHARE_DIRECTIONS, Some(GAP));
        assert!((sh[2] - 1.0).abs() < 1e-9 && sh.iter().enumerate().all(|(k, &p)| k == 2 || p < 1e-12));
        let mean = model::moments(&sh).0;
        assert!((mean * model::worth(GAP) - 2.0 * PI).abs() < 1e-9);

        let tri = the_same_width::triangle([0.0, 0.0], 0.3).count_shares(SHARE_DIRECTIONS, Some(GAP));
        assert!((model::moments(&tri).0 - 6.0 / PI).abs() < 1e-6 && tri[0] > 0.01);
        let c = [[1.0, 1.0], [-1.0, 1.0], [-1.0, -1.0], [1.0, -1.0], [1.0, 1.0]];
        let sq = polygon(&c).count_shares(SHARE_DIRECTIONS, Some(GAP));
        assert!((model::moments(&sq).0 - 8.0 / PI).abs() < 1e-6 && sq.get(4).is_some_and(|&p| p > 0.01));

        // the dent: arc 0 reflected across its chord
        let cs = corners([0.0, 0.0], 0.0);
        let mut dented = reuleaux([0.0, 0.0], 0.0);
        let inner = [cs[1][0] + cs[2][0] - cs[0][0], cs[1][1] + cs[2][1] - cs[0][1]];
        let from = (cs[1][1] - inner[1]).atan2(cs[1][0] - inner[0]);
        let to = (cs[2][1] - inner[1]).atan2(cs[2][0] - inner[0]);
        let to = from + ((to - from + PI).rem_euclid(TAU) - PI);
        dented.parts[0].seg = Seg::Arc {
            centre: inner,
            radius: SIDE,
            from,
            to,
        };
        let joined = apart(dented.parts[0].seg.point(0.0), cs[1]) < 1e-12
            && apart(dented.parts[0].seg.point(1.0), cs[2]) < 1e-12;
        assert!(joined && (dented.parts[0].seg.len() - SIDE * FRAC_PI_3).abs() < 1e-12);
        let ds = dented.count_shares(SHARE_DIRECTIONS, Some(GAP));
        assert!((model::moments(&ds).0 - 2.0).abs() < 1e-6, "{ds:?}");
        assert!(ds.get(4).is_some_and(|&p| p > 1e-4) && ds[0] > 1e-4, "{ds:?}");

        for (gap, want) in [(1.0, 4.0), (4.0, 1.0)] {
            let m = model::moments(&shape.count_shares(SHARE_DIRECTIONS, Some(gap))).0;
            assert!((m - want).abs() < 1e-9, "{gap}: {m}");
            assert!((m * model::worth(gap) - 2.0 * PI).abs() < 1e-9 && (m * PI - 2.0 * PI).abs() > 1.0);
        }
    }

    /// The perimeter found without the formula: chains with their corners
    /// on the rim, from the points' coordinates, rise toward 2π from below,
    /// and chains of tangents (each corner where two neighbouring tangent
    /// lines meet) fall toward it from above; at 3,072 corners both are
    /// within a hundred-thousandth. A finite chain only bounds the rim.
    #[test]
    fn the_rim_is_squeezed_by_chains() {
        let shape = reuleaux(BENCH, ARRANGE_TURN);
        let mut last = (0.0, f64::MAX);
        for k in 0..=10 {
            let per = 1usize << k;
            let mut inner = 0.0;
            let mut outer = 0.0;
            for part in &shape.parts {
                let Seg::Arc { centre, .. } = part.seg else {
                    panic!("arcs")
                };
                let pts: Vec<P2> = (0..=per).map(|j| part.seg.point(j as f64 / per as f64)).collect();
                for w in pts.windows(2) {
                    inner += apart(w[0], w[1]);
                    // each tangent runs square to its radius
                    let t = |p: P2| [-(p[1] - centre[1]), p[0] - centre[0]];
                    let (a, b, ta, tb) = (w[0], w[1], t(w[0]), t(w[1]));
                    let det = ta[0] * tb[1] - ta[1] * tb[0];
                    let s = ((b[0] - a[0]) * tb[1] - (b[1] - a[1]) * tb[0]) / det;
                    let q = [a[0] + s * ta[0], a[1] + s * ta[1]];
                    outer += apart(a, q) + apart(q, b);
                }
            }
            assert!(inner < 2.0 * PI && outer > 2.0 * PI, "{per}");
            assert!(inner > last.0 && outer < last.1, "{per}");
            last = (inner, outer);
        }
        assert!(2.0 * PI - last.0 < 1e-5 && last.1 - 2.0 * PI < 1e-5);
        let chains: Vec<f64> = CHAINS.iter().map(|&n| chain(BENCH, ARRANGE_TURN, n).length).collect();
        assert_eq!(
            chains.iter().map(|&l| facts::fixed(l, 2)).collect::<Vec<_>>(),
            ["6.00", "6.21", "6.27"]
        );
    }

    /// Laid end to end, the arcs keep their radius and their sixth of a
    /// turn at every share; they start as the rim and end as the upper
    /// half of one circle, turned by −60, 0 and 60 degrees; and they keep
    /// off the coast all the way.
    #[test]
    fn the_arcs_make_half_a_circle() {
        let shore = shore();
        let rim = reuleaux(BENCH, ARRANGE_TURN);
        for i in 0..=100 {
            let u = f64::from(i) / 100.0;
            let c = laid(BENCH, ARRANGE_TURN, u);
            for part in &c.parts {
                let Seg::Arc { radius, from, to, .. } = part.seg else {
                    panic!("arcs")
                };
                assert!((radius - SIDE).abs() < 1e-12 && ((to - from).abs() - FRAC_PI_3).abs() < 1e-12);
            }
            assert!((c.length - 2.0 * PI).abs() < 1e-12);
            assert!(near(&points(&c), &shore) > 0.5, "{u}");
        }
        let start = laid(BENCH, ARRANGE_TURN, 0.0);
        for (a, b) in start.parts.iter().zip(&rim.parts) {
            for u in [0.0, 0.5, 1.0] {
                assert!(apart(a.seg.point(u), b.seg.point(u)) < 1e-12);
            }
        }
        let mut turned: Vec<i64> = laid(BENCH, ARRANGE_TURN, 1.0)
            .parts
            .iter()
            .zip(&rim.parts)
            .map(|(a, b)| match (a.seg, b.seg) {
                (Seg::Arc { from: x, .. }, Seg::Arc { from: y, .. }) => (x - y).to_degrees().round() as i64,
                _ => 0,
            })
            .collect();
        turned.sort_unstable();
        assert_eq!(turned, [-60, 0, 60]);
        // the shape is turned a corner down there, its lower corner's arc
        // the one left level
        let c = corners(BENCH, ARRANGE_TURN);
        assert!(c.iter().any(|p| (p[0] - BENCH[0]).abs() < 1e-9 && p[1] < BENCH[1]));
    }

    /// The wander, the gap's mark and its label, the coin's label, the
    /// angle's label and the chains keep off the coast and inside the card;
    /// the angle's label inside the triangle under the line at 0.45.
    #[test]
    fn what_is_drawn_keeps_its_room() {
        let shore = shore();
        let (mut at, mut turn) = (BENCH, END_TURN);
        for (to, degrees) in STOPS {
            let goal = degrees.to_radians();
            for i in 0..=20 {
                let s = smooth(f64::from(i) / 20.0);
                let p = [mix(at[0], to[0], s), mix(at[1], to[1], s)];
                let c = reuleaux(p, mix(turn, goal, s));
                assert!(near(&points(&c), &shore) > 0.5, "{p:?}");
                let r = rulers(&c);
                let ends = [
                    [r.sides[0] - RULER_OVER, r.heights[0]],
                    [r.sides[1] + RULER_OVER, r.heights[0]],
                    [r.sides[0] - RULER_OVER, r.heights[1]],
                    [r.sides[1] + RULER_OVER, r.heights[1]],
                ];
                assert!(near(&ends, &shore) > 0.4, "{p:?}");
                assert!(
                    ends.iter()
                        .all(|e| e[0].abs() < CARD_W / 2.0 - 0.3 && e[1].abs() < CARD_H / 2.0 - 0.25)
                );
                // the rulers stay under the count and left of the coin
                assert!(r.heights[1] < 3.0 && r.sides[1] + RULER_OVER < COIN_AT[0] - 1.5);
            }
            (at, turn) = (to, goal);
        }
        let f = first();
        let mark: Vec<P2> = (0..=20).map(|j| [GAP_X, f.slide + GAP * f64::from(j) / 20.0]).collect();
        assert!(near(&mark, &shore) > 1.0 && near(&[GAP_LABEL_AT], &shore) > 1.0);
        assert!(GAP_LABEL_AT[0] + width("2 cm", LABEL_SIZE) / 2.0 / CARD_UNIT < GAP_X - 0.1);
        let shape = rulers(&reuleaux(BENCH, END_TURN));
        assert!(GAP_X + GAP_TICK < shape.sides[0] - RULER_OVER - 0.3);
        // the coin's label between its rim and the line at 0.45
        let h = half_height(COIN_LABEL_SIZE) / CARD_UNIT;
        assert!(COIN_LABEL_AT[1] + h < COIN_AT[1] - 1.0 - 0.1 && COIN_LABEL_AT[1] - h > f.slide + 0.1);
        // the angle's label above its mark, under the line
        let low = corners(BENCH, ARRANGE_TURN)
            .iter()
            .map(|c| c[1])
            .fold(f64::MAX, f64::min);
        let label = low + ANGLE_LABEL_UP;
        let h = half_height(ANGLE_SIZE) / CARD_UNIT;
        assert!(label - h > low + ANGLE_R + 0.05 && label + h < f.slide - 0.1);
        for n in CHAINS {
            assert!(near(&points(&chain(BENCH, ARRANGE_TURN, n)), &shore) > 0.5);
        }
    }

    /// The rows fit the console; the lower thirds keep to two lines.
    #[test]
    fn the_console_keeps_its_rows() {
        let w = words().unwrap();
        let console = bounds(CONSOLE);
        for tex in [&w.average, &w.turns, &w.laid, &w.barbier, &w.rim] {
            assert!(TOP_ROW[0] + width(&shown(tex), ROW_SIZE) < console[1] - 0.2, "{tex}");
        }
        let pair = width(&shown(&w.solve), ROW_SIZE) + 0.5 + width(&shown(&w.length), ROW_SIZE);
        assert!(LOW_ROW[0] + pair < console[1] - 0.2);
        for c in &w.chains {
            assert!(TOP_ROW[0] + width(c, LABEL_SIZE) < console[1] - 0.2);
        }
        for t in [&w.convex, &w.chance, &w.source, &w.loops] {
            assert!(t.chars().count() <= 2 * 92, "{t}");
        }
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("15_1", "Now look at the sheet's lines"),
            ("15_1", "They're two centimetres apart"),
            ("15_1", "One line runs between them"),
            ("15_1", "It has no dents"),
            ("15_1", "and crosses the rim twice"),
            ("15_2", "Slide it and turn it"),
            ("15_3", "As the sheet slides"),
            ("15_3", "for that one slide"),
            ("15_3", "And once in every gap"),
            ("15_3", "and the rim only touches them"),
            ("15_3", "Each of those is one slide"),
            ("15_4", "So like the coin"),
            ("15_4", "it averages two crossings"),
            ("15_4", "What does that make the length of its rim"),
            ("15_5", "about six point two eight"),
            ("15_5", "the same as the coin's rim"),
            ("15_6", "Check it another way"),
            ("15_6", "Each arc is centred on a corner"),
            ("15_6", "make a triangle with equal sides"),
            ("15_6", "so its angles are each sixty degrees"),
            ("15_6", "Lift the three arcs off"),
            ("15_6", "lay them end to end"),
            ("15_6", "make half of that circle"),
            ("15_6", "So half of it is two pi again"),
            ("15_7", "And a third way"),
            ("15_7", "The triangle itself is such a chain"),
            ("15_7", "With six corners"),
            ("15_7", "and with twelve"),
            ("15_8", "Barbier made the general point"),
            ("15_8", "A circle of the same width"),
            ("15_8", "pi times the width"),
            ("15_8", "Ours is two centimetres wide"),
            ("15_9", "So counting crossings and laying out the arcs agree"),
            ("15_9", "On our coast"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }
}
