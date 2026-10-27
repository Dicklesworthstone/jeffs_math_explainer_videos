//! Chapter 10, a crowd of sticks. From 09's end (the stick lying along the
//! lines, its arch flattened, the bet on its average) back to the coast
//! (10_1): looked at closely, any smooth curve is almost straight. An inset
//! shows the bay three times over round its lowest point, and the coast is
//! cut into short sticks, each running one way across the first lines.
//! Every crossing is a crossing of one stick (10_2): the seven sticks that
//! hold the first placement's seven crossings light up. Each stick runs one
//! way across the lines, as 08's straight stick did, so 08's shadow rule
//! holds for it, and the coast's average count over every slide is all its
//! sticks' shadows added up, over the gap (10_3). The shadows are laid end
//! to end along a bar on 08's console, one gap of shadow to each frame unit
//! marked under it, starting at the cove (10_4). The beach adds nothing.
//! Round the bay the way down and the way back up both go in, as the lower
//! line crosses the bay twice (10_5). Will the average come out above seven
//! or below? (10_6, a prediction, with 04's readings 7, 9, 8, 7 and a mark
//! at seven gaps.) Above (10_7): the bar passes the mark. It comes to
//! 15.26 cm, 7.63 crossings on average over every slide (10_8), for lines
//! running this one way (10_9). The bar, its halos and the trace along the
//! coast are drawn every frame from the sticks' shadows at the sheet's
//! direction, so chapter 11 turns them with the sheet. [`Shore`] builds
//! what both chapters draw on. Starts where chapter 09 ends. Not registered
//! yet: root wires it in (`docs/candidate-10-11.md`).

use std::f64::consts::PI;
use std::ops::Range;

use explainer_kit::{Kit, Palette, tween, v};
use fmn::core::rate::linear;
use fmn::prelude::*;

use super::draw::{
    EVERY, card_panel, card_point, dim_line, fade_in_all, fade_in_lagged, fade_out_all, fade_to, fail, flat,
    line_style, now, path, say, stand, swap, think, together,
};
use super::placement::{
    self, COUNT_RIGHT, CROSSINGS, HALO_OPACITY, HALO_W, Placement, Pose, WORD_GAP, at_once, faded_line, glide,
    halo_along, tracked,
};
use super::sheet::{self, COUNT_SIZE, LABEL_SIZE, Sheet, THINK_AT, TOP_Y};
use super::straight_sticks::{ALONG, HOME, LENGTH, ends, shadow};
use super::the_shadow_rule::{
    CONSOLE, RULE, RULE_AT, RULER_X, RULER_Y, TEST_SHADOW, TILTED, USES_AT, hidden_back, left_at, reach, row, tilted_to,
};
use super::turning_the_stick::{self, arch_x, arch_y, fill_points};
use crate::facts;
use crate::model::{self, BAY, BEACH, CARD_UNIT, Curve, GAP, P2, Part, Seg};
use crate::palette;

pub struct ACrowdOfSticks {
    pub kit: Kit,
}

// ---------------------------------------------------------------- the model

/// The longest stick, cm: each stretch of the coast between two cuts is
/// split into the fewest equal sticks no longer than this.
pub const LONGEST: f64 = 0.45;

/// The coast from `a` to `b` cm along as a curve of its own, measured from
/// `a`: the parts it runs through, the first and last cut where it starts
/// and ends.
pub fn piece(coast: &Curve, a: f64, b: f64) -> Curve {
    let (a, b) = (a.max(0.0), b.min(coast.length));
    let parts = coast
        .parts
        .iter()
        .filter_map(|p| {
            let len = p.seg.len();
            let (u0, u1) = (((a - p.start) / len).max(0.0), ((b - p.start) / len).min(1.0));
            (u1 - u0 > 1e-12).then(|| {
                let seg = match p.seg {
                    Seg::Line { .. } => Seg::Line {
                        from: p.seg.point(u0),
                        to: p.seg.point(u1),
                    },
                    Seg::Arc {
                        centre,
                        radius,
                        from,
                        to,
                    } => Seg::Arc {
                        centre,
                        radius,
                        from: from + u0 * (to - from),
                        to: from + u1 * (to - from),
                    },
                };
                Part {
                    seg,
                    start: p.start + u0 * len - a,
                }
            })
        })
        .collect();
    Curve {
        parts,
        length: (b - a).max(0.0),
    }
}

/// Where the coast is cut, cm along, in order: its two ends, its joints,
/// and each point where an arc lies level with the first lines (0°), its
/// angle a quarter turn off the lines' normal. Between two cuts the coast
/// runs one way across those lines.
pub fn cuts(coast: &Curve) -> Vec<f64> {
    let mut out = vec![0.0];
    for p in &coast.parts {
        if let Seg::Arc { radius, from, to, .. } = p.seg {
            let (lo, hi) = (from.min(to), from.max(to));
            let mut k = ((lo - PI / 2.0) / PI).ceil();
            while PI / 2.0 + k * PI < hi {
                let level = PI / 2.0 + k * PI;
                if level - lo > 1e-9 && hi - level > 1e-9 {
                    out.push(p.start + radius * (level - from).abs());
                }
                k += 1.0;
            }
        }
        out.push(p.end());
    }
    out.sort_by(f64::total_cmp);
    out
}

/// The sticks, each [from, to] cm along, end to end from the cove.
pub fn sticks(coast: &Curve) -> Vec<[f64; 2]> {
    let mut out = Vec::new();
    for w in cuts(coast).windows(2) {
        let (a, b) = (w[0], w[1]);
        let n = ((b - a) / LONGEST - 1e-9).ceil().max(1.0) as usize;
        let at = |i: usize| a + (b - a) * i as f64 / n as f64;
        out.extend((0..n).map(|i| [at(i), at(i + 1)]));
    }
    out
}

/// Each stick's shadow across lines running at `beta` radians, cm: how far
/// its part up to `upto` cm along the coast travels across them (nothing
/// past `upto`).
pub fn shadows(coast: &Curve, sticks: &[[f64; 2]], beta: f64, upto: f64) -> Vec<f64> {
    sticks
        .iter()
        .map(|&[a, b]| {
            if upto > a {
                piece(coast, a, b.min(upto)).travel(beta.to_degrees())
            } else {
                0.0
            }
        })
        .collect()
}

/// Where each stick's shadow lies along the bar, frame x: the shadows of
/// [`shadows`] laid end to end from [`BAR_X0`].
pub fn bar_spans(coast: &Curve, sticks: &[[f64; 2]], beta: f64, upto: f64) -> Vec<[f64; 2]> {
    let mut x = BAR_X0;
    shadows(coast, sticks, beta, upto)
        .into_iter()
        .map(|w| {
            let span = [x, x + w * BAR_PER_CM];
            x = span[1];
            span
        })
        .collect()
}

/// The sticks from `from` to `to` cm along (each lying inside), by index.
fn sticks_within(sticks: &[[f64; 2]], from: f64, to: f64) -> Range<usize> {
    let first = sticks.iter().position(|s| s[0] >= from - 1e-9).unwrap_or(sticks.len());
    let end = sticks.iter().rposition(|s| s[1] <= to + 1e-9).map_or(first, |i| i + 1);
    first..end.max(first)
}

/// What the scenes pick out on the coast and the bar, from the model: the
/// bay's lowest point; the sticks holding the first placement's crossings,
/// the beach's, and the bay's on the way down and on the way back up; the
/// line through the bay and where it crosses it; and where the beach and
/// the whole coast end on the bar at the first lines.
pub struct Marks {
    pub bottom: f64,
    pub held: Vec<usize>,
    pub beach: Range<usize>,
    pub down: Range<usize>,
    pub up: Range<usize>,
    pub lower: i64,
    pub bay_crossings: [P2; 2],
    pub beach_x: f64,
    pub end_x: f64,
}

impl Marks {
    /// Refused unless the bay has one lowest point, every crossing at the
    /// first placement lies inside a stick, clear of its ends, and the
    /// line through the bay crosses it once on the way down and once on
    /// the way back up.
    pub fn new(coast: &Curve, sticks: &[[f64; 2]], first: Pose) -> Result<Self, String> {
        let bay = coast.parts[BAY];
        let inside: Vec<f64> = cuts(coast)
            .into_iter()
            .filter(|&c| c > bay.start + 1e-9 && c < bay.end() - 1e-9)
            .collect();
        let [bottom] = inside.as_slice() else {
            return Err(format!("the bay should turn back once: cuts {inside:?}"));
        };
        let bottom = *bottom;
        let crossings = sheet::crossings(coast, first.beta, first.slide);
        let mut held = Vec::with_capacity(crossings.len());
        for c in &crossings {
            let i = sticks
                .iter()
                .position(|s| s[0] + STICK_CLEAR < c.along && c.along < s[1] - STICK_CLEAR)
                .ok_or_else(|| format!("the crossing {:.4} cm along lies at a stick's end", c.along))?;
            held.push(i);
        }
        let in_bay: Vec<_> = crossings.iter().filter(|c| c.piece == BAY).collect();
        let [d, u] = in_bay.as_slice() else {
            return Err(format!("10_5 says the bay is crossed twice: {}", in_bay.len()));
        };
        if d.line != u.line || d.along >= bottom || u.along <= bottom {
            return Err("10_5 says one line crosses the bay on the way down and back up".into());
        }
        let beach = sticks_within(sticks, coast.parts[BEACH].start, coast.parts[BEACH].end());
        let spans = bar_spans(coast, sticks, first.beta, coast.length);
        Ok(Self {
            bottom,
            held,
            beach_x: spans[beach.start][0],
            beach,
            down: sticks_within(sticks, bay.start, bottom),
            up: sticks_within(sticks, bottom, bay.end()),
            lower: d.line,
            bay_crossings: [d.at, u.at],
            end_x: spans[spans.len() - 1][1],
        })
    }
}

/// A crossing lies at least this far inside the stick holding it, cm.
const STICK_CLEAR: f64 = 0.02;

// ------------------------------------------------------------------ words

/// 10_3's rule for the whole coast, on the console's lower row.
pub const COAST_RULE: &str = r"\text{average for the coast} = \text{all its shadows} \div \text{gap}";

/// The chapter's words that carry numbers, each from the facts. Refused
/// unless the model gives what the voice says: every stick runs one way
/// across the first lines, their shadows add up to the coast's, seven is
/// the first count and the lowest the slide read, the average is above
/// it, the beach adds nothing, and the shown result follows from its shown
/// operands.
pub struct Words {
    /// The gaps under the bar, "0" to "9".
    pub numbers: Vec<String>,
    /// 04's readings in the order the slide passed them.
    pub readings: Vec<String>,
    /// "beach 0 cm"; "15.26 cm".
    pub beach: String,
    pub total: String,
    /// 10_8's result row, and 10_9's "lines at 0°".
    pub result: String,
    pub tag: String,
    /// The first placement's count.
    pub first: usize,
}

pub fn words(first: Pose) -> Result<Words, String> {
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
    let sticks = sticks(&coast);

    // every stick runs one way across the first lines: its travel is the
    // height between its ends across them, as a straight stick's shadow is
    let across = [-first.beta.sin(), first.beta.cos()];
    for &[a, b] in &sticks {
        let ([xa, ya], [xb, yb]) = (coast.at(a), coast.at(b));
        let rise = ((xb - xa) * across[0] + (yb - ya) * across[1]).abs();
        if (piece(&coast, a, b).travel(first.beta.to_degrees()) - rise).abs() > 1e-9 {
            return Err(format!(
                "the stick from {a:.4} to {b:.4} cm turns back across the lines"
            ));
        }
    }
    let total = coast.travel(first.beta.to_degrees());
    let laid: f64 = shadows(&coast, &sticks, first.beta, coast.length).iter().sum();
    if (laid - total).abs() > 1e-9 {
        return Err(format!(
            "the sticks' shadows add up to {laid}, the coast's travel is {total}"
        ));
    }
    agree("sticks.count", sticks.len().to_string())?;
    let count = sheet::crossings(&coast, first.beta, first.slide).len();
    agree("open.crossings", count.to_string())?;
    let readings: Vec<String> = get("slide.sequence")?.split(',').map(str::to_owned).collect();
    let read = readings
        .iter()
        .map(|r| r.parse::<usize>().map_err(|e| format!("{r}: {e}")))
        .collect::<Result<Vec<_>, _>>()?;
    if read.first() != Some(&count) || read.iter().any(|&r| r < count) {
        return Err(format!(
            "10_6 calls {count} the first count and 10_9 a low reading: {read:?}"
        ));
    }
    if total / GAP <= count as f64 {
        return Err(format!("10_7 says above {count}: the average is {}", total / GAP));
    }
    let reach = agree("turn.0.reach", fixed(total, 2))?;
    let mean = agree("turn.0.slide_mean", fixed(total / GAP, 2))?;
    let gap = agree("sheet.gap", fixed(GAP, 0))?;
    let beach = &coast.parts[BEACH];
    let zero = agree(
        "beach.shadow_0",
        fixed(
            piece(&coast, beach.start, beach.end()).travel(first.beta.to_degrees()),
            0,
        ),
    )?;

    // what the screen shows, from its shown operands: 15.26 ÷ 2 = 7.63
    let number = |s: &str| s.parse::<f64>().map_err(|e| format!("{s}: {e}"));
    let from_shown = fixed(number(&reach)? / number(&gap)?, 2);
    if from_shown != mean {
        return Err(format!("the screen shows {mean}, its shown operands give {from_shown}"));
    }

    Ok(Words {
        numbers: (0..=GAPS).map(|k| k.to_string()).collect(),
        readings,
        beach: format!("beach {zero} cm"),
        total: format!("{reach} cm"),
        result: format!(r"{reach} \div {gap} = {mean}\quad\text{{crossings, the average over every slide}}"),
        tag: format!(r"\text{{lines at }} {}^\circ", fixed(first.beta.to_degrees(), 0)),
        first: count,
    })
}

// ------------------------------------------------------------------ layout

/// The bar on 08's console: its left end, frame x; its middle, frame y,
/// and its thickness; half a frame unit a centimetre, so one gap of shadow
/// is one frame unit. Under it a dim axis with a tick at every gap, 0 to
/// [`GAPS`], numbered under the ticks with "gaps" after the last; the
/// stick shadows alternate full and lighter.
pub const BAR_X0: f64 = -5.6;
pub const BAR_Y: f64 = -2.2;
const BAR_H: f64 = 0.14;
pub const BAR_PER_CM: f64 = 1.0 / GAP;
const BAR_LIGHT: f64 = 0.55;
pub const GAPS: usize = 9;
const AXIS_Y: f64 = BAR_Y - BAR_H / 2.0 - 0.03;
const AXIS_W: f64 = 2.0;
const TICK_DOWN: f64 = 0.1;
const NUMBER_Y: f64 = -2.6;
const NUMBER_SIZE: f64 = 18.0;
const GAPS_WORD: &str = "gaps";
const GAPS_AT: Vec3 = [BAR_X0 + GAPS as f64 + 0.35, NUMBER_Y, 0.0];
/// Words over the bar, centred on what they name, and the console's two
/// rows, left-aligned at its left: the band over the bar, and under the
/// numbers.
pub const ABOVE_Y: f64 = -1.85;
pub const ABOVE_SIZE: f64 = 20.0;
pub const LEFT_X: f64 = -5.9;
pub const TOP_ROW: Vec3 = [LEFT_X, ABOVE_Y, 0.0];
pub const LOW_ROW: Vec3 = [LEFT_X, -3.05, 0.0];
/// The mark at the first count's number of gaps, dashed gold across the
/// bar.
const MARK_W: f64 = 2.5;
const MARK_DASH: f64 = 0.05;

/// The ticks that cut the coast into sticks, cm across it, ink.
const TICK: f64 = 0.18;
const TICK_W: f64 = 2.0;
/// The inset: the coast three times over round the bay's lowest point, on
/// a round opaque backing over the empty card above the bay, ringed; a
/// ring round the part of the bay it shows; the ticks across it and the
/// two whole sticks in it, as chords. Frame units.
const INSET_AT: Vec3 = [2.9, 2.35, 0.0];
const INSET_R: f64 = 1.2;
const ZOOM: f64 = 3.0;
const INSET_TICK: f64 = 0.3;
const RING_W: f64 = 2.5;
const CHORD_W: f64 = 3.0;
/// The rings round the bay's two crossings, as 04 ringed its crossings.
const CROSSING_RING_R: f64 = 0.17;
/// 04's readings over the coast, where 04 left its log: gold, left of the
/// countdown.
const LOG_X: f64 = -0.5;
const LOG_STEP: f64 = 0.5;
const LOG_SIZE: f64 = 26.0;
/// 10_9's "lines at 0°", where the countdown stood.
const TAG_SIZE: f64 = 24.0;
/// The card at full strength, as `card_panel` makes it.
const CARD_OPACITY: f64 = 0.55;

// where 08 left the bench (its private sizes, copied): the coast's width
// and how far 07 dimmed it and the card; the stick, its light and its
// point of shadow; the ruler
const COAST_W: f64 = 4.0;
const DIMMED: f64 = 0.25;
const STICK_W_08: f64 = 6.0;
const LIGHT_W_08: f64 = 1.5;
const LIGHT_OPACITY_08: f64 = 0.7;
const DASH_08: f64 = 0.1;
const DASH_GAP_08: f64 = 0.08;
const POINT_R_08: f64 = 0.06;
const RULER_TICK_08: f64 = 0.12;
const RULER_W_08: f64 = 2.5;
// where 09 left its plot, rows and question (its private sizes, copied)
pub const BACKS_09: [(Vec3, f64, f64); 2] = [([3.1, 2.025, 0.0], 4.2, 3.19), ([5.64, 2.61, 0.0], 1.28, 2.02)];
const PLOT_X_09: [f64; 2] = [1.45, 4.85];
const PLOT_SIZE_09: f64 = 18.0;
const TICK_Y_09: f64 = 0.63;
const HEIGHT_X_09: f64 = 1.2;
const TITLE_AT_09: Vec3 = [1.1, 3.4, 0.0];
const TITLE_09: &str = "shadow in cm";
const RIGHT_X_09: f64 = 4.95;
const RIGHT_SIZE_09: f64 = 20.0;
const PERCENT_RISE_09: f64 = 0.32;
const AXIS_W_09: f64 = 2.0;
const ARCH_W_09: f64 = 4.0;
const HALF_W_09: f64 = 2.0;
const MEAN_W_09: f64 = 2.5;
const ARCH_SAMPLES_09: usize = 360;
const FILL_OPACITY_09: f64 = 0.3;
const MEASURED_R_09: f64 = 0.07;
const LENGTH_AT_09: P2 = [HOME[0], HOME[1] + 0.6];
const ONE_AT_09: Vec3 = [-5.9, -2.65, 0.0];
const RATIO_AT_09: Vec3 = [-3.4, 3.0, 0.0];
const RATIO_SIZE_09: f64 = 28.0;
const COIN_AT_09: Vec3 = [-1.25, 3.0, 0.0];
const COIN_R_09: f64 = 0.25;
const COIN_W_09: f64 = 4.0;
const ASK_AT_09: Vec3 = [-0.65, 3.0, 0.0];
const ASK_SIZE_09: f64 = 34.0;

// ------------------------------------------------------------------ timing

const FADE_S: f64 = 0.4;
/// The ticks along the whole coast, one after another.
const TICKS_S: f64 = 2.5;
const TICK_LAG: f64 = 0.1;
/// The bar's laying, at about 1.1 cm of coast a second: the cove, the
/// beach, the climb to the fall in silence, each half of the bay, and the
/// rest after the answer.
const COVE_S: f64 = 2.5;
const BEACH_S: f64 = 3.0;
const ON_S: f64 = 7.0;
const HALF_S: f64 = 2.5;
const REST_S: f64 = 6.0;

// --------------------------------------------------------- what is drawn

/// The coast laid so far, from the cove to `laid` cm along, under a halo,
/// at the opacity `alpha` holds. In the scene: made before the coast, so
/// the coast is drawn over it.
fn live_laid(stage: &mut Stage<'_>, coast: Curve, laid: Mob, alpha: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let (o, to) = (tracked(s, alpha), s.tracker_value(laid).unwrap_or(0.0));
        let halo: Vec<VMobject> = (o > 1e-3 && to > 0.01)
            .then(|| {
                polyline(coast.upto(to, EVERY).into_iter().map(card_point))
                    .style(faded_line(Palette::ink(), HALO_W, HALO_OPACITY * o))
                    .build()
            })
            .into_iter()
            .collect();
        s.add(v_group(halo))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The bar at the sheet's direction: every stick's shadow, its part up to
/// `laid` cm along, end to end, alternate sticks full and lighter, at the
/// opacity `alpha` holds. In the scene.
fn live_bar(
    stage: &mut Stage<'_>,
    coast: Curve,
    sticks: Vec<[f64; 2]>,
    place: Placement,
    laid: Mob,
    alpha: Mob,
) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let o = tracked(s, alpha);
        let upto = s.tracker_value(laid).unwrap_or(0.0);
        let parts: Vec<VMobject> = bar_spans(&coast, &sticks, place.pose(s).beta, upto)
            .into_iter()
            .enumerate()
            .filter(|(_, [a, b])| o > 1e-3 && b - a > 1e-9)
            .map(|(i, [a, b])| {
                let (lo, hi) = (BAR_Y - BAR_H / 2.0, BAR_Y + BAR_H / 2.0);
                let shade = if i % 2 == 0 { 1.0 } else { BAR_LIGHT };
                Polygon::new(vec![v(a, lo), v(b, lo), v(b, hi), v(a, hi)])
                    .style(flat(palette::stick(), o * shade))
                    .build()
            })
            .collect();
        s.add(v_group(parts))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// A halo along the bar over the shadows of the sticks `which`, at the
/// sheet's direction and as far as `laid`, at the opacity `alpha` holds.
/// In the scene: made before the bar, so the bar is drawn over it.
fn live_span(
    stage: &mut Stage<'_>,
    coast: Curve,
    sticks: Vec<[f64; 2]>,
    place: Placement,
    laid: Mob,
    which: Range<usize>,
    alpha: Mob,
) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let o = tracked(s, alpha);
        let upto = s.tracker_value(laid).unwrap_or(0.0);
        let spans = bar_spans(&coast, &sticks, place.pose(s).beta, upto);
        let (a, b) = (spans[which.start][0], spans[which.end - 1][1]);
        let halo: Vec<VMobject> = (o > 1e-3 && b - a > 1e-6)
            .then(|| {
                polyline(vec![v(a, BAR_Y), v(b, BAR_Y)])
                    .style(faded_line(Palette::ink(), HALO_W, HALO_OPACITY * o))
                    .build()
            })
            .into_iter()
            .collect();
        s.add(v_group(halo))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// A still halo along the coast from `a` to `b` cm along, in the scene
/// and unseen: brought up with [`dim_line`], so it stays behind the coast
/// drawn after it.
fn hidden_halo(stage: &mut Stage<'_>, coast: &Curve, a: f64, b: f64) -> fmn::Result<Mob> {
    let m = halo_along(stage, coast.between(a, b, EVERY).into_iter().map(card_point).collect());
    stage.set_family_opacity(m, 0.0);
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The tick across the coast at `s` cm along, `half` either side of it, in
/// the frame through `to_frame`.
fn tick_across(coast: &Curve, s: f64, half: f64, to_frame: impl Fn(P2) -> Vec3) -> [Vec3; 2] {
    let (p, q) = (
        to_frame(coast.at((s - 0.01).max(0.0))),
        to_frame(coast.at((s + 0.01).min(coast.length))),
    );
    let c = to_frame(coast.at(s));
    let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
    let d = dx.hypot(dy);
    let n = [-dy / d * half, dx / d * half];
    [v(c[0] - n[0], c[1] - n[1]), v(c[0] + n[0], c[1] + n[1])]
}

/// Frame points as one open polyline in `style`. Detached.
fn frame_line(stage: &mut Stage<'_>, points: Vec<Vec3>, style: Style) -> Mob {
    stage.arena_mut().add(polyline(points).style(style).build())
}

/// The ticks at both ends of every stick, across the coast. Detached.
pub fn stick_ticks(stage: &mut Stage<'_>, coast: &Curve, sticks: &[[f64; 2]]) -> Vec<Mob> {
    let ends = sticks.iter().map(|s| s[0]).chain(sticks.last().map(|s| s[1]));
    let half = TICK * CARD_UNIT / 2.0;
    ends.map(|s| {
        let pts = tick_across(coast, s, half, card_point).to_vec();
        frame_line(stage, pts, line_style(Palette::ink(), TICK_W))
    })
    .collect()
}

/// What the inset shows, frame points: where it looks on the card (the
/// bay's lowest point); the bay inside it, magnified; the ticks whose ends
/// both lie inside it; the sticks lying wholly inside it, as chords.
struct Lens {
    low: Vec3,
    curve: Vec<Vec3>,
    ticks: Vec<[Vec3; 2]>,
    chords: Vec<[Vec3; 2]>,
}

/// Inside the inset, by this share of its radius (so nothing touches its
/// ring).
const INSIDE: f64 = 0.97;

impl Lens {
    fn new(coast: &Curve, sticks: &[[f64; 2]], bottom: f64) -> Self {
        let low = card_point(coast.at(bottom));
        let magnified = move |p: P2| {
            let q = card_point(p);
            v(
                INSET_AT[0] + ZOOM * (q[0] - low[0]),
                INSET_AT[1] + ZOOM * (q[1] - low[1]),
            )
        };
        let inside = |q: Vec3| (q[0] - INSET_AT[0]).hypot(q[1] - INSET_AT[1]) < INSET_R * INSIDE;
        let bay = coast.parts[BAY];
        let curve = coast
            .between(bay.start, bay.end(), EVERY / 4.0)
            .into_iter()
            .map(magnified)
            .filter(|&q| inside(q))
            .collect();
        let ends = sticks.iter().map(|s| s[0]).chain(sticks.last().map(|s| s[1]));
        let ticks = ends
            .map(|s| tick_across(coast, s, INSET_TICK / 2.0, magnified))
            .filter(|t| t.iter().all(|&q| inside(q)))
            .collect();
        let chords = sticks
            .iter()
            .map(|&[a, b]| [magnified(coast.at(a)), magnified(coast.at(b))])
            .filter(|c| c.iter().all(|&q| inside(q)))
            .collect();
        Self {
            low,
            curve,
            ticks,
            chords,
        }
    }
}

/// The inset (backing, ring, the ring on the bay, the coast in it, its
/// ticks, its whole sticks as chords), magnified round the bay's lowest
/// point `bottom` cm along. Detached, in drawing order.
fn inset(stage: &mut Stage<'_>, coast: &Curve, sticks: &[[f64; 2]], bottom: f64) -> Vec<Mob> {
    let lens = Lens::new(coast, sticks, bottom);
    let disc = Circle::new()
        .radius(INSET_R)
        .arc_center(INSET_AT)
        .style(flat(Palette::faint(), 1.0))
        .build();
    let mut out = vec![stage.arena_mut().add(disc)];
    for (at, r) in [(INSET_AT, INSET_R), (lens.low, INSET_R / ZOOM)] {
        let ring = Circle::new()
            .radius(r)
            .arc_center(at)
            .style(line_style(Palette::ink(), RING_W))
            .build();
        out.push(stage.arena_mut().add(ring));
    }
    out.push(frame_line(stage, lens.curve, line_style(palette::coast(), COAST_W)));
    for t in lens.ticks {
        out.push(frame_line(stage, t.to_vec(), line_style(Palette::ink(), TICK_W)));
    }
    for c in lens.chords {
        out.push(frame_line(stage, c.to_vec(), line_style(palette::stick(), CHORD_W)));
    }
    out
}

/// The bar's axis, its ticks, their numbers and "gaps". Detached.
pub fn bar_axis(kit: &Kit, stage: &mut Stage<'_>, words: &Words) -> fmn::Result<Vec<Mob>> {
    let right = BAR_X0 + GAPS as f64;
    let mut out = vec![frame_line(
        stage,
        vec![v(BAR_X0, AXIS_Y), v(right, AXIS_Y)],
        line_style(Palette::dim(), AXIS_W),
    )];
    for (k, n) in words.numbers.iter().enumerate() {
        let x = BAR_X0 + k as f64;
        out.push(frame_line(
            stage,
            vec![v(x, AXIS_Y), v(x, AXIS_Y - TICK_DOWN)],
            line_style(Palette::dim(), AXIS_W),
        ));
        out.push(kit.text(stage, n, NUMBER_SIZE, Palette::dim(), v(x, NUMBER_Y))?);
    }
    let gaps = kit.text(stage, GAPS_WORD, NUMBER_SIZE, Palette::dim(), GAPS_AT)?;
    out.push(left_at(stage, gaps, GAPS_AT));
    Ok(out)
}

/// Words over the bar, centred at frame x `x`. Detached.
pub fn above(kit: &Kit, stage: &mut Stage<'_>, words: &str, x: f64) -> fmn::Result<Mob> {
    kit.text(stage, words, ABOVE_SIZE, palette::stick(), v(x, ABOVE_Y))
}

/// 04's readings over the coast, gold, where 04 logged them. Detached.
pub fn readings(kit: &Kit, stage: &mut Stage<'_>, words: &Words) -> fmn::Result<Vec<Mob>> {
    words
        .readings
        .iter()
        .enumerate()
        .map(|(i, n)| {
            kit.text(
                stage,
                n,
                LOG_SIZE,
                palette::touch(),
                v(LOG_X + i as f64 * LOG_STEP, TOP_Y),
            )
        })
        .collect()
}

/// The mark at `count` gaps along the bar. Detached.
pub fn count_mark(stage: &mut Stage<'_>, count: usize) -> fmn::Result<Mob> {
    let x = BAR_X0 + count as f64;
    let mark = DashedLine::new(v(x, AXIS_Y - TICK_DOWN), v(x, BAR_Y + BAR_H / 2.0 + 0.12))
        .dash_length(MARK_DASH)
        .style(line_style(palette::touch(), MARK_W))
        .build()?;
    Ok(stage.arena_mut().add(mark))
}

/// 10_9's "lines at 0°". Detached.
pub fn direction_tag(kit: &Kit, stage: &mut Stage<'_>, words: &Words) -> fmn::Result<Mob> {
    kit.tex(stage, &words.tag, TAG_SIZE, Palette::dim(), &[], THINK_AT)
}

/// A gold ring round a crossing, card cm. Detached.
fn ring(stage: &mut Stage<'_>, at: P2) -> Mob {
    let ring = Circle::new()
        .radius(CROSSING_RING_R)
        .arc_center(card_point(at))
        .style(line_style(palette::touch(), RING_W))
        .build();
    stage.arena_mut().add(ring)
}

/// Lay the coast onto the bar as far as `to` cm along, `rt` seconds at a
/// steady pace; it ends exactly there.
fn lay(stage: &mut Stage<'_>, laid: Mob, to: f64, rt: f64) -> fmn::Result<()> {
    stage.play(glide(laid, to, rt, linear)?)?;
    stage.set_tracker_value(laid, to)?;
    Ok(())
}

/// What chapters 10 and 11 draw on, bottom to top: the card; the trace of
/// the coast laid so far and the still halos on the coast (the sticks
/// holding the first crossings, the bay's way down and way back up, the
/// beach); the coast; the clear sheet, a halo on the line through the bay
/// and the lines at the placement shown; their crossings; 08's console
/// and 09's two panels; the bar's halos (the bay's two ways, the beach)
/// and the bar; the count. Built as 09 ends it (`ended` false: the card
/// and the coast dimmed as 07 left them, the console and the panels up,
/// nothing else of it seen) or as 10 ends it (the card and the coast at
/// full strength, the console, the crossings, the count and the whole bar
/// up, the ticks on the coast).
pub struct Shore {
    pub card: Mob,
    pub pieces: Vec<Mob>,
    pub place: Placement,
    pub laid: Mob,
    pub trace_alpha: Mob,
    pub held: Vec<Mob>,
    pub bay: [Mob; 2],
    pub beach: Mob,
    pub line_alpha: Mob,
    pub dots_alpha: Mob,
    pub console: Mob,
    pub panels: [Mob; 2],
    pub bay_bar: [Mob; 2],
    pub beach_bar: Mob,
    pub bar_alpha: Mob,
    pub count_alpha: Mob,
}

impl Shore {
    pub fn new(
        kit: &Kit,
        stage: &mut Stage<'_>,
        coast: &Curve,
        sticks: &[[f64; 2]],
        first: Pose,
        ended: bool,
    ) -> fmn::Result<Self> {
        let marks = Marks::new(coast, sticks, first).map_err(fail)?;
        let up = if ended { 1.0 } else { 0.0 };
        let card = card_panel(stage)?;
        if !ended {
            stage.set_family_opacity(card, DIMMED);
        }
        stage.add_to_scene(card)?;
        let laid = stage.add_value_tracker(if ended { coast.length } else { 0.0 });
        let trace_alpha = stage.add_value_tracker(0.0);
        live_laid(stage, coast.clone(), laid, trace_alpha)?;
        let mut held = Vec::with_capacity(marks.held.len());
        for &i in &marks.held {
            held.push(hidden_halo(stage, coast, sticks[i][0], sticks[i][1])?);
        }
        let bay_part = coast.parts[BAY];
        let bay = [
            hidden_halo(stage, coast, bay_part.start, marks.bottom)?,
            hidden_halo(stage, coast, marks.bottom, bay_part.end())?,
        ];
        let beach_part = coast.parts[BEACH];
        let beach = hidden_halo(stage, coast, beach_part.start, beach_part.end())?;
        let mut pieces = Vec::with_capacity(coast.parts.len());
        for p in &coast.parts {
            let m = path(
                stage,
                &coast.between(p.start, p.end(), EVERY),
                palette::coast(),
                COAST_W,
            );
            if !ended {
                stage.set_color_lanes(m, "stroke_rgba", None, Some(DIMMED), true);
            }
            stage.add_to_scene(m)?;
            pieces.push(m);
        }
        let panel = Sheet::new(stage, first.slide, 0.0)?.panel;
        stage.add_to_scene(panel)?;
        let place = Placement::new(stage, first);
        let [line_alpha, lines_alpha, dots_alpha] = [0.0, 1.0, up].map(|o| stage.add_value_tracker(o));
        placement::live_halo(stage, place, marks.lower, line_alpha)?;
        placement::live_lines(stage, place, lines_alpha)?;
        placement::live_dots(stage, place, coast, dots_alpha)?;
        let console = hidden_back(stage, CONSOLE)?;
        stage.set_family_opacity(console, 1.0);
        let panels = [hidden_back(stage, BACKS_09[0])?, hidden_back(stage, BACKS_09[1])?];
        if !ended {
            for &m in &panels {
                stage.set_family_opacity(m, 1.0);
            }
        }
        let [down_bar, up_bar, beach_bar, bar_alpha, count_alpha] =
            [0.0, 0.0, 0.0, up, up].map(|o| stage.add_value_tracker(o));
        for (which, alpha) in [(&marks.down, down_bar), (&marks.up, up_bar), (&marks.beach, beach_bar)] {
            live_span(stage, coast.clone(), sticks.to_vec(), place, laid, which.clone(), alpha)?;
        }
        live_bar(stage, coast.clone(), sticks.to_vec(), place, laid, bar_alpha)?;
        let curve = coast.clone();
        let counted = move |s: &fmn::mobject::Stage| {
            let p = place.pose(s);
            sheet::crossings(&curve, p.beta, p.slide).len()
        };
        placement::live_count(kit, stage, counted, count_alpha, COUNT_SIZE, COUNT_RIGHT)?;
        let word_at = v(COUNT_RIGHT[0] + WORD_GAP, COUNT_RIGHT[1]);
        placement::live_word(kit, stage, CROSSINGS, count_alpha, COUNT_SIZE, word_at)?;
        if ended {
            for m in stick_ticks(stage, coast, sticks) {
                stage.add_to_scene(m)?;
            }
        }
        Ok(Self {
            card,
            pieces,
            place,
            laid,
            trace_alpha,
            held,
            bay,
            beach,
            line_alpha,
            dots_alpha,
            console,
            panels,
            bay_bar: [down_bar, up_bar],
            beach_bar,
            bar_alpha,
            count_alpha,
        })
    }
}

/// The pieces of 08's light between the points `a` and `b`, frame, as 08
/// dashes it.
fn dashes_08(a: Vec3, b: Vec3) -> Vec<Vec<Vec3>> {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let length = dx.hypot(dy);
    let mut out = Vec::new();
    let mut from = 0.0;
    while from < length {
        let to = (from + DASH_08).min(length);
        let at = |d: f64| v(a[0] + dx * d / length, a[1] + dy * d / length);
        out.push(vec![at(from), at(to)]);
        from = to + DASH_GAP_08;
    }
    out
}

/// Where 09 ended, over the shore: the stick lying along the lines, its
/// light and its point of shadow on the ruler, the ruler; the rule; the
/// flattened fill, then the arch, its axes and words, 08's two measured
/// shadows on it, the half line, the fill's top and words; the rows;
/// "6 cm" over the stick; the share, the coin and its question. Still, as
/// 09's last frame shows them, in 09's drawing order. Returns all of it,
/// for 10_1 to fade.
fn where_09_ended(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let words = turning_the_stick::words().map_err(fail)?;
    let mut out = Vec::new();
    // 08's light and shadow, then the stick and the ruler
    let mut light: Vec<VMobject> = Vec::new();
    for end in ends(HOME, ALONG) {
        for pts in dashes_08(card_point(end), card_point([RULER_X, end[1]])) {
            light.push(
                polyline(pts)
                    .style(faded_line(Palette::dim(), LIGHT_W_08, LIGHT_OPACITY_08))
                    .build(),
            );
        }
    }
    let [lo, _] = shadow(HOME, ALONG);
    light.push(
        Circle::new()
            .radius(POINT_R_08)
            .arc_center(card_point([RULER_X, lo]))
            .style(flat(palette::stick(), 1.0))
            .build(),
    );
    out.push(stage.arena_mut().add(v_group(light)));
    let along = ends(HOME, ALONG).map(card_point).to_vec();
    out.push(frame_line(stage, along, faded_line(palette::stick(), STICK_W_08, 1.0)));
    out.push(path(
        stage,
        &[[RULER_X, RULER_Y[0]], [RULER_X, RULER_Y[1]]],
        Palette::ink(),
        RULER_W_08,
    ));
    let half = (LENGTH / 2.0) as i64;
    for k in -half..=half {
        let y = HOME[1] + k as f64;
        out.push(path(
            stage,
            &[[RULER_X, y], [RULER_X + RULER_TICK_08, y]],
            Palette::ink(),
            RULER_W_08,
        ));
    }
    out.push(row(kit, stage, RULE, Palette::ink(), RULE_AT)?);
    // 09's fill, made before its arch, so the arch, the measured shadows
    // and the half line are drawn over it
    let fill = Polygon::new(fill_points(words.mean_cm, 1.0))
        .style(flat(palette::stick(), FILL_OPACITY_09))
        .build();
    out.push(stage.arena_mut().add(fill));
    // 09's arch, drawn before its axes and words
    let arch: Vec<Vec3> = (0..=ARCH_SAMPLES_09)
        .map(|j| {
            let theta = PI * j as f64 / ARCH_SAMPLES_09 as f64;
            v(arch_x(theta), arch_y(reach(theta)))
        })
        .collect();
    out.push(frame_line(stage, arch, faded_line(palette::stick(), ARCH_W_09, 1.0)));
    let [x0, x1] = PLOT_X_09;
    let (y0, top) = (arch_y(0.0), arch_y(LENGTH) + 0.12);
    let dim = line_style(Palette::dim(), AXIS_W_09);
    out.push(frame_line(stage, vec![v(x0, y0), v(x1, y0)], dim));
    out.push(frame_line(stage, vec![v(x0, y0), v(x0, top)], dim));
    for (tex, theta) in words.ticks.iter().zip([0.0, PI / 2.0, PI]) {
        out.push(kit.tex(
            stage,
            tex,
            PLOT_SIZE_09,
            Palette::dim(),
            &[],
            v(arch_x(theta), TICK_Y_09),
        )?);
    }
    for (text, cm) in words.heights.iter().zip([0.0, LENGTH / 2.0, LENGTH]) {
        out.push(kit.text(stage, text, PLOT_SIZE_09, Palette::dim(), v(HEIGHT_X_09, arch_y(cm)))?);
    }
    let title = kit.text(stage, TITLE_09, PLOT_SIZE_09, palette::stick(), TITLE_AT_09)?;
    out.push(left_at(stage, title, TITLE_AT_09));
    for theta in [TILTED, tilted_to(TEST_SHADOW)] {
        let dot = Circle::new()
            .radius(MEASURED_R_09)
            .arc_center(v(arch_x(theta), arch_y(reach(theta))))
            .style(flat(palette::touch(), 1.0))
            .build();
        out.push(stage.arena_mut().add(dot));
    }
    let half_y = arch_y(LENGTH / 2.0);
    let half_line = DashedLine::new(v(x0, half_y), v(x1, half_y))
        .dash_length(0.1)
        .style(line_style(Palette::dim(), HALF_W_09))
        .build()?;
    out.push(stage.arena_mut().add(half_line));
    let mean_y = arch_y(words.mean_cm);
    out.push(frame_line(
        stage,
        vec![v(x0, mean_y), v(x1, mean_y)],
        line_style(Palette::ink(), MEAN_W_09),
    ));
    for (text, y) in [(&words.mean, mean_y), (&words.percent, mean_y + PERCENT_RISE_09)] {
        let at = v(RIGHT_X_09, y);
        let m = kit.text(stage, text, RIGHT_SIZE_09, Palette::ink(), at)?;
        out.push(left_at(stage, m, at));
    }
    out.push(row(kit, stage, &words.over_directions, Palette::ink(), USES_AT[0])?);
    out.push(row(kit, stage, &words.estimate, palette::length(), USES_AT[1])?);
    out.push(kit.text(
        stage,
        &words.length,
        LABEL_SIZE,
        palette::length(),
        card_point(LENGTH_AT_09),
    )?);
    out.push(row(kit, stage, &words.one, Palette::ink(), ONE_AT_09)?);
    out.push(kit.tex(stage, &words.ratio, RATIO_SIZE_09, Palette::ink(), &[], RATIO_AT_09)?);
    let coin = Circle::new()
        .radius(COIN_R_09)
        .arc_center(COIN_AT_09)
        .style(line_style(palette::coin(), COIN_W_09))
        .build();
    out.push(stage.arena_mut().add(coin));
    out.push(kit.text(stage, "?", ASK_SIZE_09, palette::touch(), ASK_AT_09)?);
    for &m in &out {
        stage.add_to_scene(m)?;
    }
    Ok(out)
}

impl SceneConstruct for ACrowdOfSticks {
    fn name(&self) -> &str {
        "10_a_crowd_of_sticks"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let coast = model::coast();
        let first = Pose::new(0.0, sheet::shown_slide(&coast, 0.0));
        let sticks = sticks(&coast);
        let words = words(first).map_err(fail)?;
        let marks = Marks::new(&coast, &sticks, first).map_err(fail)?;

        // where 09 ended: the shore under it, the card and the coast as 07
        // dimmed them, the console and the panels up; 09's bench, plot and
        // rows over it
        let shore = Shore::new(&kit, stage, &coast, &sticks, first, false)?;
        let left = where_09_ended(&kit, stage)?;

        // 10_1: back to the coast; close up, the bay is almost straight;
        // the coast cut into sticks
        let line = say(&kit, stage, "10_1")?;
        line.wait_for(stage, "Back to the coast")?;
        let mut moves = vec![
            fade_to(shore.card, CARD_OPACITY, FADE_S)?,
            fade_to(shore.console, 0.0, FADE_S)?,
        ];
        for &m in &shore.panels {
            moves.push(fade_to(m, 0.0, FADE_S)?);
        }
        for &m in &shore.pieces {
            moves.push(dim_line(m, 1.0, FADE_S)?);
        }
        at_once(stage, &[], &left, moves, FADE_S)?;
        line.wait_for(stage, "Look closely")?;
        let lens = inset(stage, &coast, &sticks, marks.bottom);
        fade_in_all(stage, &lens, FADE_S)?;
        line.wait_for(stage, "a crowd of tiny sticks")?;
        let ticks = stick_ticks(stage, &coast, &sticks);
        fade_in_lagged(stage, &ticks, TICKS_S, TICK_LAG)?;

        // 10_2: every crossing is one stick's; seven
        let line = say(&kit, stage, "10_2")?;
        line.wait_for(stage, "Every crossing of the coast")?;
        at_once(stage, &[], &lens, vec![tween(shore.dots_alpha, 1.0, FADE_S)?], FADE_S)?;
        line.wait_for(stage, "one of these sticks")?;
        let lit = shore.held.iter().map(|&m| dim_line(m, HALO_OPACITY, FADE_S));
        together(stage, lit.collect::<fmn::Result<Vec<_>>>()?)?;
        line.wait_for(stage, "here that's seven")?;
        stage.play(tween(shore.count_alpha, 1.0, FADE_S)?)?;

        // 10_3: each stick obeys 08's rule; so does the coast, added up
        let line = say(&kit, stage, "10_3")?;
        line.wait_for(stage, "the shadow rule holds for it")?;
        let rule = row(&kit, stage, RULE, Palette::ink(), TOP_ROW)?;
        let mut moves = vec![fade_to(shore.console, 1.0, FADE_S)?];
        for &m in &shore.held {
            moves.push(dim_line(m, 0.0, FADE_S)?);
        }
        at_once(stage, &[rule], &[], moves, FADE_S)?;
        line.wait_for(stage, "all its sticks' shadows")?;
        let coast_rule = row(&kit, stage, COAST_RULE, Palette::ink(), LOW_ROW)?;
        fade_in_all(stage, &[coast_rule], FADE_S)?;

        // 10_4: the shadows laid end to end from the cove; the beach adds
        // nothing; in silence, the climb to the fall
        let line = say(&kit, stage, "10_4")?;
        line.wait_for(stage, "Lay their shadows end to end")?;
        let axis = bar_axis(&kit, stage, &words)?;
        at_once(
            stage,
            &axis,
            &[rule],
            vec![
                tween(shore.bar_alpha, 1.0, FADE_S)?,
                tween(shore.trace_alpha, 1.0, FADE_S)?,
            ],
            FADE_S,
        )?;
        line.wait_for(stage, "starting at the cove")?;
        lay(stage, shore.laid, coast.parts[BEACH].start, COVE_S)?;
        line.wait_for(stage, "Along the beach")?;
        lay(stage, shore.laid, coast.parts[BEACH].end(), BEACH_S)?;
        line.wait_for(stage, "casts no shadow")?;
        let beach = above(&kit, stage, &words.beach, marks.beach_x)?;
        fade_in_all(stage, &[beach], FADE_S)?;
        kit.hold(stage)?;
        lay(stage, shore.laid, coast.parts[BAY].start, ON_S)?;

        // 10_5: the bay's way down and way back up both go in, each laid
        // as it is named, as its line crosses it twice
        let line = say(&kit, stage, "10_5")?;
        line.wait_for(stage, "Round the bay")?;
        at_once(
            stage,
            &[],
            &[beach],
            vec![
                dim_line(shore.bay[0], HALO_OPACITY, FADE_S)?,
                tween(shore.bay_bar[0], 1.0, FADE_S)?,
                glide(shore.laid, marks.bottom, HALF_S, linear)?,
            ],
            FADE_S,
        )?;
        stage.set_tracker_value(shore.laid, marks.bottom)?;
        line.wait_for(stage, "comes back up")?;
        together(
            stage,
            vec![
                dim_line(shore.bay[1], HALO_OPACITY, FADE_S)?,
                tween(shore.bay_bar[1], 1.0, FADE_S)?,
                glide(shore.laid, coast.parts[BAY].end(), HALF_S, linear)?,
            ],
        )?;
        stage.set_tracker_value(shore.laid, coast.parts[BAY].end())?;
        line.wait_for(stage, "this line through the bay crosses it twice")?;
        stage.play(tween(shore.line_alpha, 1.0, FADE_S)?)?;
        line.wait_for(stage, "once on the way down")?;
        let rings = marks.bay_crossings.map(|at| ring(stage, at));
        fade_in_all(stage, &rings[..1], FADE_S)?;
        line.wait_for(stage, "once on the way back up")?;
        fade_in_all(stage, &rings[1..], FADE_S)?;

        // 10_6: seven, and 04's readings; above seven or below?
        let line = say(&kit, stage, "10_6")?;
        line.wait_for(stage, "Our first count was seven")?;
        at_once(
            stage,
            &[],
            &rings,
            vec![
                dim_line(shore.bay[0], 0.0, FADE_S)?,
                dim_line(shore.bay[1], 0.0, FADE_S)?,
                tween(shore.bay_bar[0], 0.0, FADE_S)?,
                tween(shore.bay_bar[1], 0.0, FADE_S)?,
                tween(shore.line_alpha, 0.0, FADE_S)?,
            ],
            FADE_S,
        )?;
        line.wait_for(stage, "nine and eight as well")?;
        let log = readings(&kit, stage, &words)?;
        fade_in_all(stage, &log, FADE_S)?;
        line.wait_for(stage, "above seven, or below")?;
        let mark = count_mark(stage, words.first)?;
        fade_in_all(stage, &[mark], FADE_S)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 10_7: above; the rest laid on, past the mark
        let line = say(&kit, stage, "10_7")?;
        line.wait_for(stage, "Above")?;
        fade_out_all(stage, &[q], 0.3)?;
        line.wait_for(stage, "The last stretch")?;
        lay(stage, shore.laid, coast.length, REST_S)?;

        // 10_8: 15.26 cm; over the gap, 7.63 crossings over every slide
        let line = say(&kit, stage, "10_8")?;
        line.wait_for(stage, "fifteen point two six centimetres")?;
        let total = above(&kit, stage, &words.total, marks.end_x)?;
        at_once(
            stage,
            &[total],
            &[],
            vec![tween(shore.trace_alpha, 0.0, FADE_S)?],
            FADE_S,
        )?;
        line.wait_for(stage, "seven point six three crossings")?;
        let result = row(&kit, stage, &words.result, Palette::ink(), LOW_ROW)?;
        swap(stage, &[coast_rule], &[result], FADE_S)?;

        // 10_9: a low reading; no sliding needed; only this one way
        let line = say(&kit, stage, "10_9")?;
        line.wait_for(stage, "this one way")?;
        let tag = direction_tag(&kit, stage, &words)?;
        fade_in_all(stage, &[tag], FADE_S)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::THINK_R;
    use super::super::the_shadow_rule::ROW_SIZE;
    use super::*;
    use crate::model::{CARD_CENTRE, CARD_H, CARD_W};

    fn first() -> Pose {
        Pose::new(0.0, sheet::shown_slide(&model::coast(), 0.0))
    }

    fn half_height(size: f64) -> f64 {
        size / 150.0
    }

    fn width(text: &str, size: f64) -> f64 {
        text.chars().count() as f64 * size / 150.0
    }

    fn bounds((at, w, h): (Vec3, f64, f64)) -> [f64; 4] {
        [at[0] - w / 2.0, at[0] + w / 2.0, at[1] - h / 2.0, at[1] + h / 2.0]
    }

    /// The sticks run end to end from the cove to the far end, none longer
    /// than [`LONGEST`], each one way across the first lines (its travel
    /// the height between its ends, as a straight stick's shadow is). Their
    /// shadows add up to the coast's travel at every degree of half a
    /// turn, and so do the parts of them laid up to any point.
    #[test]
    fn the_sticks_add_up_to_the_coast() {
        let coast = model::coast();
        let sticks = sticks(&coast);
        assert_eq!(sticks.len(), 67);
        assert!(sticks[0][0].abs() < 1e-12 && (sticks[sticks.len() - 1][1] - coast.length).abs() < 1e-9);
        for w in sticks.windows(2) {
            assert!((w[0][1] - w[1][0]).abs() < 1e-12);
        }
        for &[a, b] in &sticks {
            assert!(b - a <= LONGEST + 1e-12 && b - a > 0.2, "{a} {b}");
            let rise = (coast.at(b)[1] - coast.at(a)[1]).abs();
            assert!((piece(&coast, a, b).travel(0.0) - rise).abs() < 1e-9, "{a} {b}");
        }
        // fifteen one-way stretches: the joints and four level points
        assert_eq!(cuts(&coast).len(), coast.parts.len() + 1 + 4);
        for d in 0..=180 {
            let beta = f64::from(d).to_radians();
            let laid: f64 = shadows(&coast, &sticks, beta, coast.length).iter().sum();
            assert!((laid - coast.travel(f64::from(d))).abs() < 1e-9, "{d}");
        }
        for s in [0.5, 6.1180, 13.9613, 16.7, 19.0, 25.0] {
            let laid: f64 = shadows(&coast, &sticks, 0.0, s).iter().sum();
            assert!((laid - piece(&coast, 0.0, s).travel(0.0)).abs() < 1e-9, "{s}");
        }
        // a negative control: cut only at the joints, and some stretch
        // turns back across the lines, so its travel is more than the
        // height between its ends
        let bay = coast.parts[BAY];
        let rise = (coast.at(bay.end())[1] - coast.at(bay.start)[1]).abs();
        assert!(piece(&coast, bay.start, bay.end()).travel(0.0) > rise + 1.0);
    }

    /// 10_5: the bay goes down 1.75 cm (its radius, from its left end at
    /// the centre's height to its lowest point) and back up 1.30, both
    /// laid on the bar; the line through it crosses each way once.
    #[test]
    fn the_bay_counts_twice() {
        let coast = model::coast();
        let sticks = sticks(&coast);
        let marks = Marks::new(&coast, &sticks, first()).unwrap();
        let shadows = shadows(&coast, &sticks, 0.0, coast.length);
        let down: f64 = shadows[marks.down.clone()].iter().sum();
        let up: f64 = shadows[marks.up.clone()].iter().sum();
        assert!((down - 1.75).abs() < 1e-9, "{down}");
        assert_eq!(facts::fixed(up, 2), "1.30");
        let bay = coast.parts[BAY];
        assert!((down + up - piece(&coast, bay.start, bay.end()).travel(0.0)).abs() < 1e-9);
        assert_eq!(marks.down.end, marks.up.start);
        assert!(marks.down.len() > 3 && marks.up.len() > 3);
        // the crossing on the way down is left of the lowest point, the
        // one on the way up right of it, both on the lower line
        let low = coast.at(marks.bottom);
        assert!(marks.bay_crossings[0][0] < low[0] && marks.bay_crossings[1][0] > low[0]);
        let [d, u] = marks.bay_crossings;
        assert!((d[1] - u[1]).abs() < 1e-9);
        let line = first().slide + marks.lower as f64 * GAP;
        assert!((d[1] - line).abs() < 1e-9);
    }

    /// 10_4 and 11_2: the beach lies along the first lines and adds
    /// nothing; turned a quarter, its shadow is its whole 3.5 cm.
    #[test]
    fn the_beach_casts_none_then_its_length() {
        let coast = model::coast();
        let sticks = sticks(&coast);
        let marks = Marks::new(&coast, &sticks, first()).unwrap();
        let beach = coast.parts[BEACH];
        let alone = piece(&coast, beach.start, beach.end());
        assert!(alone.travel(0.0).abs() < 1e-12);
        assert!((alone.travel(90.0) - 3.5).abs() < 1e-12);
        let at = |beta: f64| -> f64 {
            shadows(&coast, &sticks, beta, coast.length)[marks.beach.clone()]
                .iter()
                .sum()
        };
        assert!(at(0.0).abs() < 1e-12 && (at(PI / 2.0) - 3.5).abs() < 1e-12);
        assert_eq!(marks.beach.len(), 8);
    }

    /// 10_2: each of the seven crossings lies inside its own stick, clear
    /// of its ends.
    #[test]
    fn every_crossing_is_one_sticks() {
        let coast = model::coast();
        let sticks = sticks(&coast);
        let marks = Marks::new(&coast, &sticks, first()).unwrap();
        assert_eq!(marks.held.len(), 7);
        let mut seen = marks.held.clone();
        seen.dedup();
        assert_eq!(seen.len(), 7);
        // a negative control: a stick's end on a crossing is refused
        let c = sheet::crossings(&coast, 0.0, first().slide)[0].along;
        let mut moved = sticks.clone();
        let i = marks.held[0];
        moved[i][1] = c;
        moved[i + 1][0] = c;
        assert!(Marks::new(&coast, &moved, first()).is_err());
    }

    /// The words on screen say what the facts and the model say.
    #[test]
    fn the_screen_words_are_built_from_the_facts() {
        let w = words(first()).unwrap();
        assert_eq!(w.numbers, ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]);
        assert_eq!(w.readings, ["7", "9", "8", "7"]);
        assert_eq!(w.beach, "beach 0 cm");
        assert_eq!(w.total, "15.26 cm");
        assert_eq!(
            w.result,
            r"15.26 \div 2 = 7.63\quad\text{crossings, the average over every slide}"
        );
        assert_eq!(w.tag, r"\text{lines at } 0^\circ");
        assert_eq!(w.first, 7);
    }

    /// The bar holds the coast's whole shadow at every tenth of a degree
    /// of half a turn, inside its marked gaps and the console; the words
    /// over it, its numbers and the rows inside the console, apart; the
    /// marks where the model puts them; the inset on empty card, clear of
    /// the coast, the count and the lines' band edges; the readings and the
    /// tag in the band over the coast.
    #[test]
    fn the_layout_keeps_its_rooms() {
        let coast = model::coast();
        let sticks = sticks(&coast);
        let marks = Marks::new(&coast, &sticks, first()).unwrap();
        let console = bounds(CONSOLE);
        let longest = (0..=1800)
            .map(|i| coast.travel(f64::from(i) / 10.0))
            .fold(f64::MIN, f64::max);
        assert!(BAR_X0 + longest * BAR_PER_CM < BAR_X0 + GAPS as f64 - 0.2, "{longest}");
        assert!(BAR_X0 > console[0] + 0.3);
        const {
            assert!(GAPS_AT[0] + 4.0 * NUMBER_SIZE / 150.0 < 6.3 - 0.2);
            assert!(ABOVE_Y - ABOVE_SIZE / 150.0 > BAR_Y + BAR_H / 2.0 + 0.1);
            assert!(NUMBER_Y + NUMBER_SIZE / 150.0 < AXIS_Y - TICK_DOWN - 0.03);
            assert!(LOW_ROW[1] + 24.0 / 150.0 < NUMBER_Y - NUMBER_SIZE / 150.0 - 0.1);
        }
        assert!(ABOVE_Y + half_height(ABOVE_SIZE) < console[3] - 0.1);
        assert!(LOW_ROW[1] - half_height(ROW_SIZE) > console[2] + 0.1);
        assert!(LEFT_X > console[0] + 0.2);
        // the rows fit the console: 10_3's two rules and 10_8's result
        let result = words(first()).unwrap().result;
        for tex in [RULE, COAST_RULE, result.as_str()] {
            let shown = tex.replace(r"\text", "").replace(['{', '}', '\\'], "");
            assert!(LEFT_X + width(&shown, ROW_SIZE) < console[1] - 0.2, "{tex}");
        }
        // the words over the bar stay on the console and clear of each
        // other
        let (beach, total) = (width("beach 0 cm", ABOVE_SIZE), width("15.26 cm", ABOVE_SIZE));
        assert!(marks.beach_x - beach / 2.0 > console[0] + 0.1);
        assert!(marks.beach_x + beach / 2.0 < marks.end_x - total / 2.0 - 0.3);
        assert!((marks.end_x - (BAR_X0 + 15.260268 * BAR_PER_CM)).abs() < 1e-5);
        assert!((marks.beach_x - (BAR_X0 + 1.482362 * BAR_PER_CM)).abs() < 1e-5);
        // the inset: inside the card, clear of the coast, the count, the
        // countdown's ring and the readings
        let card = [
            CARD_CENTRE[0] - CARD_UNIT * CARD_W / 2.0,
            CARD_CENTRE[0] + CARD_UNIT * CARD_W / 2.0,
            CARD_CENTRE[1] - CARD_UNIT * CARD_H / 2.0,
            CARD_CENTRE[1] + CARD_UNIT * CARD_H / 2.0,
        ];
        assert!(INSET_AT[0] - INSET_R > card[0] + 0.3 && INSET_AT[0] + INSET_R < card[1] - 0.3);
        assert!(INSET_AT[1] + INSET_R < card[3] - 0.2);
        let n = (coast.length / 0.02).ceil() as usize;
        for i in 0..=n {
            let c = card_point(coast.at(coast.length * i as f64 / n as f64));
            assert!((c[0] - INSET_AT[0]).hypot(c[1] - INSET_AT[1]) > INSET_R + 0.3, "{c:?}");
        }
        const {
            assert!(INSET_AT[0] - INSET_R > LOG_X + 3.0 * LOG_STEP + 0.3);
            assert!(LOG_X + 3.0 * LOG_STEP + 0.3 < THINK_AT[0] - THINK_R);
        }
        // the readings and the tag between the lines over the coast, the
        // countdown at the band's right
        let lines: Vec<f64> = sheet::lines_on_card(first().slide)
            .iter()
            .map(|&h| CARD_CENTRE[1] + CARD_UNIT * h)
            .collect();
        for y in [TOP_Y, THINK_AT[1]] {
            assert!(lines.iter().all(|&l| (l - y).abs() > 0.3), "{y}");
        }
        assert!(THINK_AT[0] - width("lines at 0°", TAG_SIZE) / 2.0 > LOG_X + 3.0 * LOG_STEP + 0.3);
    }

    /// The inset looks at the bay's lowest point, where a stick ends, and
    /// shows the two whole sticks there as chords with all three of their
    /// end ticks, each chord within a line's width of the magnified bay; the
    /// coast inside it is the bay alone.
    #[test]
    fn the_inset_shows_two_whole_sticks() {
        let coast = model::coast();
        let sticks = sticks(&coast);
        let marks = Marks::new(&coast, &sticks, first()).unwrap();
        assert!(sticks.iter().any(|s| (s[0] - marks.bottom).abs() < 1e-9));
        let lens = Lens::new(&coast, &sticks, marks.bottom);
        assert_eq!(lens.chords.len(), 2);
        assert_eq!(lens.ticks.len(), 3);
        for [a, b] in &lens.chords {
            assert!(
                lens.ticks
                    .iter()
                    .any(|t| ((t[0][0] + t[1][0]) / 2.0 - a[0]).abs() < 1e-9)
            );
            assert!(
                lens.ticks
                    .iter()
                    .any(|t| ((t[0][0] + t[1][0]) / 2.0 - b[0]).abs() < 1e-9)
            );
            // the bay's farthest point from the chord, magnified
            let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
            let off = lens
                .curve
                .iter()
                .filter(|q| ((q[0] - a[0]) * dx + (q[1] - a[1]) * dy) / (dx * dx + dy * dy) > 0.0)
                .filter(|q| ((q[0] - a[0]) * dx + (q[1] - a[1]) * dy) / (dx * dx + dy * dy) < 1.0)
                .map(|q| ((q[0] - a[0]) * dy - (q[1] - a[1]) * dx).abs() / dx.hypot(dy))
                .fold(0.0, f64::max);
            assert!(off > 0.01 && off < 0.04, "{off}");
        }
        // only the bay comes within the inset's reach on the card
        let reach = INSET_R / ZOOM;
        for (i, p) in coast.parts.iter().enumerate().filter(|&(i, _)| i != BAY) {
            for q in coast.between(p.start, p.end(), EVERY) {
                let c = card_point(q);
                assert!((c[0] - lens.low[0]).hypot(c[1] - lens.low[1]) > reach, "{i}");
            }
        }
    }

    /// 09's stick lies level along the lines, so 08 drew its shadow as a
    /// point, as the rebuild does; its "6 cm" stands over the stick.
    #[test]
    fn the_rebuild_draws_a_point_of_shadow() {
        let [lo, hi] = shadow(HOME, ALONG);
        assert!(hi - lo <= 1e-6);
        let [a, b] = ends(HOME, ALONG);
        assert!(LENGTH_AT_09[0] > a[0].min(b[0]) && LENGTH_AT_09[0] < a[0].max(b[0]));
        assert!(LENGTH_AT_09[1] > a[1].max(b[1]));
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("10_1", "Back to the coast"),
            ("10_1", "Look closely"),
            ("10_1", "a crowd of tiny sticks"),
            ("10_2", "Every crossing of the coast"),
            ("10_2", "one of these sticks"),
            ("10_2", "here that's seven"),
            ("10_3", "the shadow rule holds for it"),
            ("10_3", "all its sticks' shadows"),
            ("10_4", "Lay their shadows end to end"),
            ("10_4", "starting at the cove"),
            ("10_4", "Along the beach"),
            ("10_4", "casts no shadow"),
            ("10_5", "Round the bay"),
            ("10_5", "comes back up"),
            ("10_5", "this line through the bay crosses it twice"),
            ("10_5", "once on the way down"),
            ("10_5", "once on the way back up"),
            ("10_6", "Our first count was seven"),
            ("10_6", "nine and eight as well"),
            ("10_6", "above seven, or below"),
            ("10_7", "Above"),
            ("10_7", "The last stretch"),
            ("10_8", "fifteen point two six centimetres"),
            ("10_8", "seven point six three crossings"),
            ("10_9", "this one way"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }
}
