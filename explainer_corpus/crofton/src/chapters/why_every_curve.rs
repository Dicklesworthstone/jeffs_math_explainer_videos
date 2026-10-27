//! Chapter 12, why every curve. From 11's end (the computer's 3,600
//! directions, "computed, not proved") the plot, the list and the rows go
//! and the claim comes up: the average over every slide and direction is
//! the length times one number, the same for every curve (12_1). Barbier's
//! route, in 1860, needs no integral (12_2). Crossings add: cut at its
//! joints, the coast's pieces are crossed 1, 0, 1, 1, 0, 1, 2, 0, 0, 0, 1
//! times, the bay twice, seven in all (12_3), and as the sheet slides a gap
//! the live sum keeps adding up (12_4). Our 6 cm stick, moved, crosses two
//! instead of three, and put back with the sheet moved the other way, two
//! again (12_5): the average gives every placement an equal share, so only
//! a stick's length can matter (12_6). Its halves count their own
//! crossings, and as a line passes the cut one loses what the other gains
//! (12_7); laid across each other, a line through their meeting point
//! counts for both (12_8). Equal pieces, equal averages: thirds, quarters,
//! three quarters of the stick (12_9). For a length that is no fraction,
//! the line between the coast's ends, a 14 cm stick along it is never
//! crossed more often and a 15 cm stick never less, at any placement
//! (12_10), so its average is squeezed between tenths and hundredths of
//! the number (12_11).
//! A chain of sticks with its corners on the coast averages exactly its
//! length times the number; at the joints it is 21.39 cm and here crosses
//! five, its chord across the bay missing both of the bay's crossings
//! (12_12); finer chains, 23.91 and 25.71 cm, catch all seven and are never
//! crossed more than the coast (12_13), and in the limit the coast's
//! average is its length times the same number (12_14). Which curve would
//! find the number? (12_15, a prediction.) The coin: two meeting points
//! wherever it lands, a rim of 2π, so the number is 1/π (12_16); every
//! crossing is worth π, and the coast's 25.81 ÷ π ≈ 8.22 is what the
//! computer found (12_17). Every count, dot and wire is drawn from the
//! model at the sheet's placement and the wire's trackers. Starts where
//! chapter 11 ends. Not registered yet: root wires it in
//! (`docs/candidate-12-13.md`).

use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU};
use std::rc::Rc;

use explainer_kit::props::{lower_third, precisely};
use explainer_kit::{Kit, Palette, TextKind, ink_style, tween, v};
use fmn::core::rate::{linear, smooth};
use fmn::prelude::*;

use super::a_crowd_of_sticks::{LEFT_X, LOW_ROW, Shore, TOP_ROW, cuts, stick_ticks, sticks};
use super::draw::{
    card_point, dim_line, fade_in_all, fade_out_all, fade_to, fail, flat, line_style, now, path, say, stand, think,
    together,
};
use super::placement::{
    self, COUNT_RIGHT, CROSSINGS, HALO_OPACITY, Placement, Pose, WORD_GAP, at_once, faded_line, glide, halo_along,
    tracked,
};
use super::sheet::{self, COUNT_SIZE, DOT_R, THINK_AT};
use super::straight_sticks::{ACROSS, HOME, LENGTH};
use super::the_shadow_rule::{ROW_SIZE, left_at, row};
use super::turning_the_sheet::{self, GRID, plot_x, plot_y};
use crate::facts;
use crate::model::{self, BAY, COIN, Curve, GAP, P2, Part, Seg};
use crate::palette;

pub struct WhyEveryCurve {
    pub kit: Kit,
}

// ---------------------------------------------------------------- the wire

/// A wire on the card made of straight pieces: `shape` lays them out at
/// the share `morph` holds, in the wire's own frame (its home point at the
/// origin, its length running along +x), then the wire is turned by `turn`
/// radians and carried to (`x`, `y`), card cm. Its pieces are drawn, dotted
/// and counted every frame from the trackers, one piece at a time, so a
/// line through a point where two pieces meet counts for both.
#[derive(Clone, Copy)]
pub struct Wire {
    pub x: Mob,
    pub y: Mob,
    pub turn: Mob,
    pub morph: Mob,
    /// 0 for one tone; 1 for its odd pieces lighter, so its pieces show.
    pub shade: Mob,
    pub alpha: Mob,
    pub shape: fn(f64) -> Vec<[P2; 2]>,
}

/// How much lighter a shaded wire's odd pieces are drawn.
const SHADE: f64 = 0.5;

/// How far apart two card points are, cm.
pub fn dist(a: P2, b: P2) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// Pieces laid out by `shape` at `morph`, turned by `turn` and carried to
/// `at`, as one curve: its parts in order, each starting where the one
/// before it ends along the wire (whether or not they touch on the card).
pub fn placed(shape: fn(f64) -> Vec<[P2; 2]>, at: P2, turn: f64, morph: f64) -> Curve {
    let (c, s) = (turn.cos(), turn.sin());
    let to_card = |p: P2| [at[0] + c * p[0] - s * p[1], at[1] + s * p[0] + c * p[1]];
    let mut parts = Vec::new();
    let mut start = 0.0;
    for [a, b] in shape(morph) {
        let seg = Seg::Line {
            from: to_card(a),
            to: to_card(b),
        };
        parts.push(Part { seg, start });
        start += seg.len();
    }
    Curve { parts, length: start }
}

impl Wire {
    /// The wire at `at`, turned by `turn`, unmorphed, unshaded and unseen.
    pub fn new(stage: &mut Stage<'_>, at: P2, turn: f64, shape: fn(f64) -> Vec<[P2; 2]>) -> Self {
        let [x, y, turn, morph, shade, alpha] = [at[0], at[1], turn, 0.0, 0.0, 0.0].map(|t| stage.add_value_tracker(t));
        Self {
            x,
            y,
            turn,
            morph,
            shade,
            alpha,
            shape,
        }
    }

    /// The wire as its trackers hold it.
    pub fn curve(&self, s: &fmn::mobject::Stage) -> Curve {
        let get = |m: Mob| s.tracker_value(m).unwrap_or(0.0);
        placed(self.shape, [get(self.x), get(self.y)], get(self.turn), get(self.morph))
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

/// The stick as its two halves, the lower half first, each half as
/// [`LENGTH`] ÷ 2. Past `u` = 0 the lower half swings about its own middle
/// to [`CROSS_TURN`]; past ½ it slides along the stick until its middle
/// lies on the upper half's (at 1).
pub fn halves(u: f64) -> Vec<[P2; 2]> {
    let half = LENGTH / 2.0;
    let swing = CROSS_TURN * (2.0 * u).clamp(0.0, 1.0);
    let centre = -half / 2.0 + half * (2.0 * u - 1.0).clamp(0.0, 1.0);
    let (dx, dy) = (half / 2.0 * swing.cos(), half / 2.0 * swing.sin());
    vec![[[centre - dx, -dy], [centre + dx, dy]], [[0.0, 0.0], [half, 0.0]]]
}

/// How far the lower half swings before it slides across, radians: lying
/// across, from straight up to the upper left, its lower end swinging over
/// the beach.
pub const CROSS_TURN: f64 = FRAC_PI_4;

/// The wire drawn as its trackers hold it. In the scene.
pub fn live_wire(stage: &mut Stage<'_>, wire: Wire) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let (o, shade) = (tracked(s, wire.alpha), tracked(s, wire.shade));
        let pieces: Vec<VMobject> = wire
            .curve(s)
            .parts
            .iter()
            .enumerate()
            .map(|(i, part)| {
                let tone = if i % 2 == 1 { 1.0 - SHADE * shade } else { 1.0 };
                polyline(vec![card_point(part.seg.point(0.0)), card_point(part.seg.point(1.0))])
                    .style(faded_line(palette::stick(), STICK_W, o * tone))
                    .build()
            })
            .collect();
        s.add(v_group(pieces))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The wire's crossings at the placement shown, gold dots as
/// [`placement::live_dots`] draws a curve's, at the opacity `alpha` holds.
/// In the scene.
pub fn live_wire_dots(stage: &mut Stage<'_>, wire: Wire, place: Placement, alpha: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let (o, p) = (tracked(s, alpha), place.pose(s));
        let dots: Vec<VMobject> = sheet::crossings(&wire.curve(s), p.beta, p.slide)
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

/// A fixed curve's crossings at the placement shown, dots in `color` as
/// [`placement::live_dots`] draws them in gold, at the opacity `alpha`
/// holds. In the scene.
fn live_curve_dots(
    stage: &mut Stage<'_>,
    place: Placement,
    curve: &Curve,
    color: Srgb,
    alpha: Mob,
) -> fmn::Result<Mob> {
    let curve = curve.clone();
    let m = stage.always_redraw(move |s| {
        let (o, p) = (tracked(s, alpha), place.pose(s));
        let dots: Vec<VMobject> = sheet::crossings(&curve, p.beta, p.slide)
            .iter()
            .map(|c| {
                Circle::new()
                    .radius(DOT_R)
                    .arc_center(card_point(c.at))
                    .style(flat(color, o))
                    .build()
            })
            .collect();
        s.add(v_group(dots))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The wire's count at the placement shown, piece by piece; with `piece`,
/// that piece's alone.
pub fn wire_count(
    wire: Wire,
    place: Placement,
    piece: Option<usize>,
) -> impl Fn(&fmn::mobject::Stage) -> usize + Clone + 'static {
    move |s| {
        let p = place.pose(s);
        sheet::crossings(&wire.curve(s), p.beta, p.slide)
            .iter()
            .filter(|c| piece.is_none_or(|i| c.piece == i))
            .count()
    }
}

/// "crossing" for a count of one, "crossings" otherwise.
fn noun(n: usize) -> &'static str {
    if n == 1 { "crossing" } else { CROSSINGS }
}

/// Keep moving sheet lines behind a small opaque count badge. The badge
/// follows the same opacity tracker as its digits, including transitions.
fn count_back(stage: &mut Stage<'_>, alpha: Mob, at: Vec3, width: f64, height: f64) -> fmn::Result<()> {
    let panel = Rectangle::new()
        .width(width)
        .height(height)
        .corner_radius(0.06)
        .style(flat(Palette::faint(), 1.0))
        .build()?
        .moved_to(at);
    let m = stage.always_redraw(move |s| {
        let o = tracked(s, alpha);
        let m = s.add(panel.clone());
        s.set_family_opacity(m, o);
        m
    });
    stage.add_to_scene(m)?;
    Ok(())
}

/// A count where the coast's stands, gold digits and their word ("1
/// crossing", "3 crossings"), both redrawn every frame, at the opacity
/// `alpha` holds, as `placement::live_count` and 07's word draw them. In
/// the scene.
pub fn counter(
    kit: &Kit,
    stage: &mut Stage<'_>,
    count: impl Fn(&fmn::mobject::Stage) -> usize + Clone + 'static,
    alpha: Mob,
) -> fmn::Result<()> {
    count_back(stage, alpha, v(COUNT_RIGHT[0] + 0.55, COUNT_RIGHT[1]), 2.0, 0.43)?;
    placement::live_count(kit, stage, count.clone(), alpha, COUNT_SIZE, COUNT_RIGHT)?;
    let book = Rc::clone(&kit.book);
    let at = v(COUNT_RIGHT[0] + WORD_GAP, COUNT_RIGHT[1]);
    let m = stage.always_redraw(move |s| {
        let (o, word) = (tracked(s, alpha), noun(count(s)));
        let t = Text::new(word)
            .font_size(COUNT_SIZE)
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
    Ok(())
}

// --------------------------------------------------------------- the model

/// Where the stick is carried in 12_5 (cm right of home) and the direction
/// it is turned to: it crosses two, not three.
pub const MOVE_RIGHT: f64 = 1.5;
pub const MOVE_TO: f64 = FRAC_PI_4;

/// The sheet carried the other way: the stick's turn undone about home,
/// its shift undone across the lines (along them it changes nothing).
pub fn moved_back(first: Pose) -> placement::Move {
    let mut mv = placement::Move::turn(first, ACROSS - MOVE_TO, HOME);
    mv.by = first.beta.sin() * MOVE_RIGHT;
    mv
}

/// The two lengths laid along the line between the coast's ends (12_10),
/// cm, and the sheet there: turned a quarter about a point on its middle
/// line so its slide comes to the first, then slid to the second.
pub const NESTED: [f64; 2] = [14.0, 15.0];
pub const NESTED_SLIDES: [f64; 2] = [0.0, 1.0];
const NESTED_PIN: P2 = [0.0, 0.45];

/// The line between the coast's ends, from its start: `l` cm of it, as a
/// stick.
pub fn along_ends(coast: &Curve, l: f64) -> Curve {
    let (a, b) = coast.ends();
    let apart = dist(a, b);
    let to = [a[0] + l * (b[0] - a[0]) / apart, a[1] + l * (b[1] - a[1]) / apart];
    Curve {
        parts: vec![Part {
            seg: Seg::Line { from: a, to },
            start: 0.0,
        }],
        length: l,
    }
}

/// Chains with their corners on the coast at the cm along it given, in
/// order: its joints, chapter 10's cuts, and the ends of 10's sticks.
pub fn chain(coast: &Curve, at: &[f64]) -> Curve {
    let pts: Vec<P2> = at.iter().map(|&s| coast.at(s)).collect();
    let mut parts = Vec::with_capacity(pts.len());
    let mut start = 0.0;
    for w in pts.windows(2) {
        let seg = Seg::Line { from: w[0], to: w[1] };
        parts.push(Part { seg, start });
        start += seg.len();
    }
    Curve { parts, length: start }
}

pub fn chain_points(coast: &Curve) -> [Vec<f64>; 3] {
    let mut joints = vec![0.0];
    joints.extend(coast.parts.iter().map(|p| p.end()));
    let mut ends: Vec<f64> = sticks(coast).iter().map(|s| s[0]).collect();
    ends.push(coast.length);
    [joints, cuts(coast), ends]
}

/// The coin where 12_16 lays it, card cm: in the open card at the upper
/// right, over the line above the bay.
pub const COIN_AT: P2 = [2.0, 2.4];

/// The coin's rim as a curve: one arc, the whole way round.
pub fn coin_rim(at: P2) -> Curve {
    let seg = Seg::Arc {
        centre: at,
        radius: COIN / 2.0,
        from: 0.0,
        to: TAU,
    };
    Curve {
        parts: vec![Part { seg, start: 0.0 }],
        length: seg.len(),
    }
}

/// The count at `pose`, piece by piece.
pub fn count_at(curve: &Curve, pose: Pose) -> usize {
    sheet::crossings(curve, pose.beta, pose.slide).len()
}

// ------------------------------------------------------------------ words

/// The chapter's words that carry numbers, each from the facts. Refused
/// unless the model gives what the screen and the voice say: the pieces'
/// counts add up to the coast's; the moved stick and the stick put back
/// under the moved sheet cross alike, unlike the stick at home; the halves
/// add up; the crossed halves count once each at their meeting; equal
/// pieces' lengths; the ends' distance inside each bracket and the nested
/// counts in order; each chain's corners, length and count, never more
/// than the coast's; the coin's two meetings and rim; and the shown length
/// over π giving the shown average.
pub struct Words {
    pub claim: String,
    pub source: String,
    /// 12_3's row: each piece's count (the coast's after them).
    pub by_piece: Vec<usize>,
    pub pieces: String,
    pub ones: String,
    /// 12_10's labels, each before its live count; 12_11's rows.
    pub nested: [String; 3],
    pub brackets: Vec<String>,
    /// 12_12 and 12_13: each chain's length and count.
    pub chains: Vec<String>,
    pub limit: String,
    pub coin: String,
    pub calibrated: String,
    pub theorem: String,
    pub payoff: String,
}

/// Barbier's note: J. Math. Pures Appl. (2) 5 (1860), the argument on
/// pages 275 to 277 (`brief.md`).
pub const BARBIER_YEAR: u32 = 1860;
const SOURCE_LABEL: &str = "source:";
const CLAIM: &str = r"\text{average count} = \text{length} \times ?";
const ONES: &str = r"\text{average count} = \text{length} \times \text{average for 1 cm}";
const COIN_ROW: &str = r"2 = 2\pi \times ?";
const CALIBRATED: &str = r"? = 1 \div \pi";
const THEOREM: &str = r"\text{average count} = \text{length} \div \pi";
const LIMIT: &str = "for a curve with a length: the chains' averages rise to the curve's. Lines through a corner \
                     or an end, or touching the curve, have no chance and change no average";
const CUT_EXACT: &str = "a line exactly through the cut is a single slide among a whole gap of them: it has no \
                         chance, and changes no average";

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
    agree("barbier.year", BARBIER_YEAR.to_string())?;

    // 12_3: the pieces' counts add up to the coast's
    let crossings = sheet::crossings(&coast, first.beta, first.slide);
    let by_piece: Vec<usize> = (0..coast.parts.len())
        .map(|i| crossings.iter().filter(|c| c.piece == i).count())
        .collect();
    let joined: Vec<String> = by_piece.iter().map(usize::to_string).collect();
    agree("open.piece_counts", joined.join(","))?;
    let total = by_piece.iter().sum::<usize>();
    agree("open.crossings", total.to_string())?;
    agree("open.bay_crossings", by_piece[BAY].to_string())?;

    // 12_5: moved, two; at home under the sheet moved back, two; at home
    // under the first placement, three
    let home = |pose: Pose| count_at(&placed(halves, HOME, ACROSS, 0.0), pose);
    let moved = placed(halves, [HOME[0] + MOVE_RIGHT, HOME[1]], MOVE_TO, 0.0);
    let there = agree("move.count", count_at(&moved, first).to_string())?;
    let back = agree("move.sheet_count", home(moved_back(first).end()).to_string())?;
    let at_home = agree("stick.home_across", home(first).to_string())?;
    if there != back || there == at_home {
        return Err(format!("12_5 says {there} moved, {back} put back, not {at_home}"));
    }

    // 12_7 and 12_8: the halves' counts; crossed, one each through their
    // meeting point
    let split = sheet::crossings(&placed(halves, HOME, ACROSS, 0.0), first.beta, first.slide);
    let each = |i: usize| split.iter().filter(|c| c.piece == i).count().to_string();
    agree("halves.counts", format!("{},{}", each(0), each(1)))?;
    let crossed = placed(halves, HOME, ACROSS, 1.0);
    agree("cross.count", count_at(&crossed, first).to_string())?;
    let meet = HOME[1] + LENGTH / 4.0;
    let through = sheet::crossings(&crossed, 0.0, meet.rem_euclid(GAP));
    if through.iter().map(|c| c.piece).collect::<Vec<_>>() != [0, 1] {
        return Err("12_8 says the line through the meeting point crosses each half once".into());
    }
    agree("cross.through", through.len().to_string())?;

    // 12_9: thirds, quarters, three quarters
    let third = agree("pieces.third", fixed(LENGTH / 3.0, 0))?;
    let quarter = agree("pieces.quarter", fixed(LENGTH / 4.0, 1))?;
    let three = agree("pieces.three_quarters", fixed(3.0 * LENGTH / 4.0, 1))?;
    let stick = agree("stick.length", fixed(LENGTH, 0))?;
    if number(&three)? != 3.0 * number(&quarter)? || 3.0 * number(&third)? != number(&stick)? {
        return Err(format!("12_9 shows {three} = 3 × {quarter} and 3 × {third} = {stick}"));
    }
    let pieces = format!(
        r"\text{{average for }} {three}\text{{ cm}} = 3 \times \text{{average for }} {quarter}\text{{ cm}} = \tfrac{{3}}{{4}} \times \text{{average for }} {stick}\text{{ cm}}"
    );

    // 12_10 and 12_11: the line between the ends, in its brackets; the
    // nested sticks' counts at the two placements shown, in order
    let (a, b) = coast.ends();
    let apart = dist(a, b);
    agree("ends.apart_2dp", fixed(apart, 2))?;
    let mut brackets = Vec::new();
    for k in 0..3 {
        let scale = 10f64.powi(k);
        let lo = (apart * scale).floor() / scale;
        let (low, high) = (fixed(lo, k as usize), fixed(lo + 1.0 / scale, k as usize));
        agree(&format!("ends.bracket.{k}"), format!("{low},{high}"))?;
        brackets.push(format!(r"{low} \times ? \le \text{{average}} \le {high} \times ?"));
    }
    if get("ends.bracket.1")?.split(',').next() != Some(get("coast.ends_apart")?.as_str()) {
        return Err("12_10's lower tenth is not 03's distance between the ends".into());
    }
    let nested = [
        along_ends(&coast, NESTED[0]),
        along_ends(&coast, apart),
        along_ends(&coast, NESTED[1]),
    ];
    for (i, mv) in nested_moves(first).iter().enumerate() {
        let counts: Vec<String> = nested.iter().map(|c| count_at(c, mv.end()).to_string()).collect();
        agree(&format!("ends.nested.{i}"), counts.join(","))?;
    }

    // 12_12 and 12_13: each chain's corners, length and count; the joints
    // chain misses the bay's two crossings
    let mut chains = Vec::new();
    for (name, at) in ["joints", "cuts", "sticks"].iter().zip(chain_points(&coast)) {
        let c = chain(&coast, &at);
        agree(&format!("chain.{name}.chords"), c.parts.len().to_string())?;
        let length = agree(&format!("chain.{name}.length"), fixed(c.length, 2))?;
        let n = agree(&format!("chain.{name}.crossings"), count_at(&c, first).to_string())?;
        if number(&n)? > total as f64 || c.length >= coast.length {
            return Err(format!("12_13 says the {name} chain is shorter and crossed no more"));
        }
        chains.push(format!("chain {length} cm, {n} crossings"));
    }
    let joints = chain(&coast, &chain_points(&coast)[0]);
    if count_at(&joints, first) + by_piece[BAY] != total
        || sheet::crossings(&joints, first.beta, first.slide)
            .iter()
            .any(|c| c.piece == BAY)
    {
        return Err("12_12 says the chord across the bay misses its two crossings".into());
    }

    // 12_16: the coin's two meetings and rim; 12_17: the shown length over
    // π gives the shown average
    agree("coin.crossings", count_at(&coin_rim(COIN_AT), first).to_string())?;
    agree("coin.rim", fixed(coin_rim(COIN_AT).length, 2))?;
    let length = agree("coast.length_2dp", fixed(coast.length, 2))?;
    let mean = agree("crofton.mean_crossings", fixed(coast.length / PI, 2))?;
    if fixed(number(&length)? / PI, 2) != mean || (model::worth(GAP) - PI).abs() > 1e-12 {
        return Err(format!("12_17 shows {length} ÷ π ≈ {mean}"));
    }

    Ok(Words {
        claim: CLAIM.into(),
        source: format!("Barbier, {BARBIER_YEAR}, J. Math. Pures Appl., pages 275 to 277"),
        by_piece,
        pieces,
        ones: ONES.into(),
        nested: [
            format!("{} cm", fixed(NESTED[0], 0)),
            "ends".into(),
            format!("{} cm", fixed(NESTED[1], 0)),
        ],
        brackets,
        chains,
        limit: LIMIT.into(),
        coin: COIN_ROW.into(),
        calibrated: CALIBRATED.into(),
        theorem: THEOREM.into(),
        payoff: format!(r"{length} \div \pi \approx {mean}"),
    })
}

/// 12_10's two moves from the first placement: a quarter turn about a
/// point on its middle line, then the slide across.
pub fn nested_moves(first: Pose) -> [placement::Move; 2] {
    let turned = placement::Move::turn(first, FRAC_PI_2, NESTED_PIN);
    let slid = placement::Move::slide(turned.end(), NESTED_SLIDES[1] - NESTED_SLIDES[0]);
    [turned, slid]
}

// ------------------------------------------------------------------ layout

const STICK_W: f64 = 6.0;
const COAST_DIMMED: f64 = 0.35;
const TONE: f64 = 0.5;
const CARD_OPACITY: f64 = 0.55;
/// 12_3's live sum on the console: each piece's count, its digits' right
/// edge a step apart, a plus between; the coast's count after "=".
const SUM_STEP: f64 = 0.62;
const DIGIT_W: f64 = 0.16;
const SIGN_GAP: f64 = 0.23;
/// 12_7: the halves' own counts, left of each half, card cm.
const HALF_COUNT_AT: [P2; 2] = [[-5.4, -0.3], [-5.4, 2.7]];
const HALF_SIZE: f64 = 26.0;
/// 12_8's ring round the meeting point, frame.
const RING_R: f64 = 0.17;
/// 12_9's ticks across the stick, card cm either way.
const TICK_HALF: f64 = 0.18;
const TICK_W: f64 = 3.0;
/// 12_10's labels and live counts on the console's top row.
const NESTED_X: [f64; 3] = [-5.9, -3.4, -0.9];
const NESTED_COUNT_GAP: f64 = 1.3;
const LABEL_SIZE: f64 = 22.0;
const CHAIN_W: f64 = 3.0;
const ENDS_W: f64 = 4.0;
const ENDS_STUB: f64 = 0.35;
/// The coin as 06 draws it at rest.
const COIN_FILL: f64 = 0.16;
const RIM_W: f64 = 4.0;
/// 12_16's half turn about a point on the first placement's middle line,
/// below the coin.
const COIN_PIN: P2 = [COIN_AT[0], 0.45];

/// 11's layout, copied so 12 starts on 11's last frame (11's own are
/// private): its plot, list, rows and caption.
const PLOT_LOW_11: f64 = 14.0;
const PLOT_HIGH_11: f64 = 18.0;
const PLOT_SIZE_11: f64 = 18.0;
const TICK_Y_11: f64 = 0.63;
const HEIGHT_X_11: f64 = 1.2;
const TITLE_AT_11: Vec3 = [1.1, 3.4, 0.0];
const TITLE_11: &str = "total shadow in cm";
const RIGHT_X_11: f64 = 4.95;
const RIGHT_SIZE_11: f64 = 20.0;
const AXIS_W_11: f64 = 2.0;
const CURVE_W_11: f64 = 4.0;
const MEAN_W_11: f64 = 2.5;
const CURVE_SAMPLES_11: usize = 360;
const FILL_OPACITY_11: f64 = 0.3;
const LIST_Y_11: [f64; 2] = [-1.95, -2.5];
const LIST_SIZE_11: f64 = 20.0;
const COLUMN_X_11: [f64; 5] = [-2.4, -1.2, 0.0, 1.2, 2.6];
const ROW_C_11: Vec3 = [LEFT_X, -3.1, 0.0];
const CHAIN_GAP_11: f64 = 0.25;
const CAPTION_AT_11: Vec3 = [3.4, -2.22, 0.0];
const CAPTION_11: &str = "computed, not proved";

// ------------------------------------------------------------------ timing

const FADE_S: f64 = 0.4;
const MOVE_S: f64 = 2.5;
const SLIDE_S: f64 = 8.0;
const CUT_SLIDE_S: f64 = 10.0;
const SWING_S: f64 = 3.0;
const THROUGH_S: f64 = 2.0;
const TURN_S: f64 = 3.0;
const HALF_TURN_S: f64 = 6.0;

// --------------------------------------------------------- what is drawn

/// The shore at full strength, as 11 leaves it: the card and the coast
/// bright, the console up, 09's panels up or not (`panels`), nothing else
/// of it seen. Built as 09 ends it and brightened at once, so the ticks
/// can be made apart (and faded) rather than inside it.
pub fn bright_shore(kit: &Kit, stage: &mut Stage<'_>, coast: &Curve, first: Pose, panels: bool) -> fmn::Result<Shore> {
    let mut shore = Shore::new(kit, stage, coast, &sticks(coast), first, false)?;
    // Leave Shore's original count at its initial zero opacity; install a
    // backed local count after the sheet without changing earlier chapters.
    shore.count_alpha = stage.add_value_tracker(0.0);
    let curve = coast.clone();
    let place = shore.place;
    counter(
        kit,
        stage,
        move |s| {
            let p = place.pose(s);
            sheet::crossings(&curve, p.beta, p.slide).len()
        },
        shore.count_alpha,
    )?;
    stage.set_family_opacity(shore.card, CARD_OPACITY);
    for &m in &shore.pieces {
        stage.set_color_lanes(m, "stroke_rgba", None, Some(1.0), true);
    }
    stage.set_tracker_value(shore.laid, coast.length)?;
    if !panels {
        for &m in &shore.panels {
            stage.set_family_opacity(m, 0.0);
        }
    }
    Ok(shore)
}

/// Frame points as one open polyline. Detached.
fn frame_path(stage: &mut Stage<'_>, points: Vec<Vec3>, color: Srgb, width: f64) -> Mob {
    stage
        .arena_mut()
        .add(polyline(points).style(line_style(color, width)).build())
}

/// What 11 leaves over the shore: its plot (axes, words, the whole curve,
/// the flattened fill, its top and height), its list, its rows and its
/// caption, each as 11 makes it. In the scene; the stick ticks are not.
fn where_11_ended(kit: &Kit, stage: &mut Stage<'_>, coast: &Curve) -> fmn::Result<Vec<Mob>> {
    let w = turning_the_sheet::words().map_err(fail)?;
    let floor = plot_y(PLOT_LOW_11);
    let top = plot_y(PLOT_HIGH_11) + 0.12;
    let mut out = vec![
        frame_path(
            stage,
            vec![v(plot_x(0.0), floor), v(plot_x(PI), floor)],
            Palette::dim(),
            AXIS_W_11,
        ),
        frame_path(
            stage,
            vec![v(plot_x(0.0), floor), v(plot_x(0.0), top)],
            Palette::dim(),
            AXIS_W_11,
        ),
    ];
    for (tex, beta) in w.ticks.iter().zip([0.0, PI / 2.0, PI]) {
        out.push(kit.tex(
            stage,
            tex,
            PLOT_SIZE_11,
            Palette::dim(),
            &[],
            v(plot_x(beta), TICK_Y_11),
        )?);
    }
    let levels = [PLOT_LOW_11, (PLOT_LOW_11 + PLOT_HIGH_11) / 2.0, PLOT_HIGH_11];
    for (text, cm) in w.heights.iter().zip(levels) {
        out.push(kit.text(stage, text, PLOT_SIZE_11, Palette::dim(), v(HEIGHT_X_11, plot_y(cm)))?);
    }
    let title = kit.text(stage, TITLE_11, PLOT_SIZE_11, palette::stick(), TITLE_AT_11)?;
    out.push(left_at(stage, title, TITLE_AT_11));
    let curve: Vec<Vec3> = (0..=CURVE_SAMPLES_11)
        .map(|j| {
            let beta = PI * j as f64 / CURVE_SAMPLES_11 as f64;
            v(plot_x(beta), plot_y(coast.travel(beta.to_degrees())))
        })
        .collect();
    out.push(
        stage.arena_mut().add(
            polyline(curve)
                .style(faded_line(palette::stick(), CURVE_W_11, 1.0))
                .build(),
        ),
    );
    let mut fill = vec![v(plot_x(0.0), floor), v(plot_x(PI), floor)];
    for j in (0..=CURVE_SAMPLES_11).rev() {
        let beta = PI * j as f64 / CURVE_SAMPLES_11 as f64;
        let h = coast.travel(beta.to_degrees());
        fill.push(v(plot_x(beta), plot_y(h + (w.mean_cm - h) * 1.0)));
    }
    // Match chapter 11's repaired paint order: fill behind axes and curve.
    out.insert(
        0,
        stage.arena_mut().add(
            Polygon::new(fill)
                .style(flat(palette::stick(), FILL_OPACITY_11))
                .build(),
        ),
    );
    let level = plot_y(w.mean_cm);
    out.push(frame_path(
        stage,
        vec![v(plot_x(0.0), level), v(plot_x(PI), level)],
        Palette::ink(),
        MEAN_W_11,
    ));
    let mean = kit.text(stage, &w.mean, RIGHT_SIZE_11, Palette::ink(), v(RIGHT_X_11, level))?;
    out.push(left_at(stage, mean, v(RIGHT_X_11, level)));
    for (name, y) in w.names.iter().zip(LIST_Y_11) {
        let m = kit.text(stage, name, LIST_SIZE_11, Palette::dim(), v(LEFT_X, y))?;
        out.push(left_at(stage, m, v(LEFT_X, y)));
    }
    for (i, x) in COLUMN_X_11.iter().enumerate() {
        let shade = if i + 1 == GRID.len() {
            palette::length()
        } else {
            Palette::ink()
        };
        out.push(kit.text(stage, &w.counts[i], LIST_SIZE_11, Palette::ink(), v(*x, LIST_Y_11[0]))?);
        out.push(kit.text(stage, &w.estimates[i], LIST_SIZE_11, shade, v(*x, LIST_Y_11[1]))?);
    }
    let halve = row(kit, stage, &w.halve, Palette::ink(), ROW_C_11)?;
    let times = kit.tex(stage, &w.times, ROW_SIZE, palette::length(), &[], ROW_C_11)?;
    stage.next_to(times, halve, RIGHT, CHAIN_GAP_11, ORIGIN);
    let truth = kit.text(stage, &w.truth, LIST_SIZE_11, palette::length(), ROW_C_11)?;
    stage.next_to(truth, times, RIGHT, CHAIN_GAP_11, ORIGIN);
    let caption = kit.text(stage, CAPTION_11, LIST_SIZE_11, Palette::dim(), CAPTION_AT_11)?;
    out.extend([halve, times, truth, left_at(stage, caption, CAPTION_AT_11)]);
    for &m in &out {
        stage.add_to_scene(m)?;
    }
    Ok(out)
}

/// x of the right edge of the `i`th digit of 12_3's sum (the coast's
/// count is the one after the last piece's).
fn sum_x(i: usize) -> f64 {
    LEFT_X + DIGIT_W + SUM_STEP * i as f64
}

/// 12_3's sum: every piece's live count with a plus after it, "=" and the
/// coast's live count, at the opacity `alpha` holds; refused unless it
/// opens on the words' counts. Its signs detached; its counts in the
/// scene.
fn live_sum(
    kit: &Kit,
    stage: &mut Stage<'_>,
    coast: &Curve,
    words: &Words,
    place: Placement,
    alpha: Mob,
) -> fmn::Result<Vec<Mob>> {
    let y = TOP_ROW[1];
    let n = words.by_piece.len();
    let opening = sheet::crossings(coast, place.pose(stage).beta, place.pose(stage).slide);
    if n != coast.parts.len() || (0..n).any(|i| opening.iter().filter(|c| c.piece == i).count() != words.by_piece[i]) {
        return Err(fail(format!("12_3's row opens on {:?}", words.by_piece)));
    }
    let mut signs = Vec::with_capacity(n);
    for i in 0..n {
        let curve = coast.clone();
        let count = move |s: &fmn::mobject::Stage| {
            let p = place.pose(s);
            sheet::crossings(&curve, p.beta, p.slide)
                .iter()
                .filter(|c| c.piece == i)
                .count()
        };
        placement::live_count(kit, stage, count, alpha, ROW_SIZE, v(sum_x(i), y))?;
        let sign = if i + 1 < n { "+" } else { "=" };
        signs.push(kit.text(stage, sign, ROW_SIZE, Palette::ink(), v(sum_x(i) + SIGN_GAP, y))?);
    }
    let curve = coast.clone();
    let whole = move |s: &fmn::mobject::Stage| {
        let p = place.pose(s);
        sheet::crossings(&curve, p.beta, p.slide).len()
    };
    placement::live_count(kit, stage, whole, alpha, ROW_SIZE, v(sum_x(n), y))?;
    Ok(signs)
}

/// Short ticks across the stick at home lying across, at the heights
/// given (card cm). Detached.
fn ticks_across(stage: &mut Stage<'_>, heights: &[f64]) -> Vec<Mob> {
    heights
        .iter()
        .map(|&y| {
            let pts = vec![
                card_point([HOME[0] - TICK_HALF, y]),
                card_point([HOME[0] + TICK_HALF, y]),
            ];
            frame_path(stage, pts, Palette::ink(), TICK_W)
        })
        .collect()
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

/// The coin at rest at `c`, its disc and rim as 06 draws them. Detached.
pub fn coin_at_rest(stage: &mut Stage<'_>, c: P2) -> Vec<Mob> {
    let (at, r) = (card_point(c), model::CARD_UNIT * COIN / 2.0);
    let disc = Circle::new()
        .radius(r)
        .arc_center(at)
        .style(flat(palette::coin(), COIN_FILL))
        .build();
    let rim = Circle::new()
        .radius(r)
        .arc_center(at)
        .style(line_style(palette::coin(), RIM_W))
        .build();
    [disc, rim].map(|m| stage.arena_mut().add(m)).to_vec()
}

/// 12's last rows: the rule, and the coast's length over π. Detached.
pub fn closing_rows(kit: &Kit, stage: &mut Stage<'_>, words: &Words) -> fmn::Result<[Mob; 2]> {
    Ok([
        row(kit, stage, &words.theorem, Palette::ink(), TOP_ROW)?,
        row(kit, stage, &words.payoff, palette::length(), LOW_ROW)?,
    ])
}

/// The sheet's planned move `mv`, refused unless the sheet stands where it
/// starts, `rt` seconds at `rate`; then named the controls' way (its lines
/// unchanged), so a slide of a whole gap or a half turn ends on the first
/// placement's name.
fn make(
    stage: &mut Stage<'_>,
    place: Placement,
    mv: &placement::Move,
    rt: f64,
    rate: fn(f64) -> f64,
) -> fmn::Result<()> {
    place.make(stage, mv, &[(1.0, rt)], rate)?;
    place.settle(stage).map(|_| ())
}

/// The sheet turned by `turn` about `pin` and slid `by` from where it
/// stands, as [`make`] makes a move.
fn move_sheet(stage: &mut Stage<'_>, place: Placement, turn: f64, by: f64, pin: P2, rt: f64) -> fmn::Result<()> {
    let mut mv = placement::Move::turn(place.pose(stage), turn, pin);
    mv.by = by;
    make(stage, place, &mv, rt, smooth)
}

impl SceneConstruct for WhyEveryCurve {
    fn name(&self) -> &str {
        "12_why_every_curve"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let coast = model::coast();
        let first = Pose::new(0.0, sheet::shown_slide(&coast, 0.0));
        let words = words().map_err(fail)?;

        // where 11 ended: the shore under it with 09's panels up and 10's
        // ticks on the coast; 11's plot, list, rows and caption over it
        let shore = bright_shore(&kit, stage, &coast, first, true)?;
        let ticks = stick_ticks(stage, &coast, &sticks(&coast));
        for &m in &ticks {
            stage.add_to_scene(m)?;
        }
        let left = where_11_ended(&kit, stage, &coast)?;
        let place = shore.place;

        // 12_1: computed on one coast; why every curve? The plot, the list
        // and the rows go; the claim
        let line = say(&kit, stage, "12_1")?;
        line.wait_for(stage, "Why should it work for every curve")?;
        let mut gone = left.clone();
        gone.extend(&ticks);
        let panels = shore.panels.iter().map(|&m| fade_to(m, 0.0, FADE_S));
        at_once(stage, &[], &gone, panels.collect::<fmn::Result<Vec<_>>>()?, FADE_S)?;
        line.wait_for(stage, "Here's the claim")?;
        let claim = row(&kit, stage, &words.claim, Palette::ink(), TOP_ROW)?;
        fade_in_all(stage, &[claim], FADE_S)?;

        // 12_2: Barbier, 1860; crossings add up
        let line = say(&kit, stage, "12_2")?;
        line.wait_for(stage, "Barbier made this argument")?;
        let source = lower_third(&kit, stage, SOURCE_LABEL, &words.source)?;
        fade_in_all(stage, &[source], FADE_S)?;
        line.wait_for(stage, "crossings add up")?;
        fade_out_all(stage, &[source], FADE_S)?;

        // 12_3: cut at the joints, the pieces in two tones; their counts
        // add to the coast's seven, the bay's two on its way down and up
        let line = say(&kit, stage, "12_3")?;
        line.wait_for(stage, "Cut the coast at its joints")?;
        let spans: Vec<[f64; 2]> = coast.parts.iter().map(|p| [p.start, p.end()]).collect();
        let joints = stick_ticks(stage, &coast, &spans);
        let mut tones = Vec::new();
        for (i, &m) in shore.pieces.iter().enumerate() {
            if i % 2 == 1 {
                tones.push(dim_line(m, TONE, FADE_S)?);
            }
        }
        at_once(stage, &joints, &[claim], tones, FADE_S)?;
        line.wait_for(stage, "The cove is crossed")?;
        stage.play(tween(shore.dots_alpha, 1.0, FADE_S)?)?;
        line.wait_for(stage, "the bay is crossed twice")?;
        let bay = shore.bay.iter().map(|&m| dim_line(m, HALO_OPACITY, FADE_S));
        together(stage, bay.collect::<fmn::Result<Vec<_>>>()?)?;
        line.wait_for(stage, "Add up every piece's crossings")?;
        let sum_alpha = stage.add_value_tracker(0.0);
        let signs = live_sum(&kit, stage, &coast, &words, place, sum_alpha)?;
        at_once(stage, &signs, &[], vec![tween(sum_alpha, 1.0, FADE_S)?], FADE_S)?;
        line.wait_for(stage, "the coast's seven")?;
        stage.play(tween(shore.count_alpha, 1.0, FADE_S)?)?;

        // 12_4: true at every placement; the sheet slides a gap, the sum
        // keeping up
        let line = say(&kit, stage, "12_4")?;
        line.wait_for(stage, "Slide the sheet")?;
        let bay = shore.bay.iter().map(|&m| dim_line(m, 0.0, FADE_S));
        together(stage, bay.collect::<fmn::Result<Vec<_>>>()?)?;
        make(stage, place, &placement::Move::slide(first, GAP), SLIDE_S, linear)?;
        line.wait_for(stage, "So the pieces' averages add up")?;

        // 12_5: the stick at home, three; moved, two; back, three; the
        // sheet moved the other way, two
        let line = say(&kit, stage, "12_5")?;
        line.wait_for(stage, "Now take our six centimetre stick")?;
        let wire = Wire::new(stage, HOME, ACROSS, halves);
        live_wire(stage, wire)?;
        let wire_dots = stage.add_value_tracker(0.0);
        live_wire_dots(stage, wire, place, wire_dots)?;
        let wire_counts = stage.add_value_tracker(0.0);
        counter(&kit, stage, wire_count(wire, place, None), wire_counts)?;
        // the coast's count, its dots and the sum go first, then the
        // stick's come where they were (never two counts at once)
        let mut moves = vec![
            tween(sum_alpha, 0.0, FADE_S)?,
            tween(shore.count_alpha, 0.0, FADE_S)?,
            tween(shore.dots_alpha, 0.0, FADE_S)?,
        ];
        for (i, &m) in shore.pieces.iter().enumerate() {
            if i % 2 == 1 {
                moves.push(dim_line(m, 1.0, FADE_S)?);
            }
        }
        let mut gone = signs.clone();
        gone.extend(&joints);
        at_once(stage, &[], &gone, moves, FADE_S)?;
        together(
            stage,
            vec![
                tween(wire.alpha, 1.0, FADE_S)?,
                tween(wire_dots, 1.0, FADE_S)?,
                tween(wire_counts, 1.0, FADE_S)?,
            ],
        )?;
        line.wait_for(stage, "Move it")?;
        together(stage, wire.moves([HOME[0] + MOVE_RIGHT, HOME[1]], MOVE_TO, MOVE_S)?)?;
        line.wait_for(stage, "Put the stick back")?;
        // Complete both sequential moves inside this spoken sentence so
        // the following "Two again" continues without an inserted gap.
        together(stage, wire.moves(HOME, ACROSS, 1.5)?)?;
        line.wait_for(stage, "move the sheet instead")?;
        make(stage, place, &moved_back(first), 1.5, smooth)?;

        // 12_6: every placement an equal share; only the length matters.
        // The sheet goes back
        let line = say(&kit, stage, "12_6")?;
        line.wait_for(stage, "only reshuffles them")?;
        move_sheet(stage, place, MOVE_TO - ACROSS, 0.0, HOME, TURN_S)?;
        line.wait_for(stage, "It can only depend on its length")?;

        // 12_7: cut in half, two tones and a tick at the cut; each half's
        // count beside it; a slide of one gap past the cut
        let line = say(&kit, stage, "12_7")?;
        line.wait_for(stage, "Cut the stick in half")?;
        let cut = ticks_across(stage, &[HOME[1]]);
        let halves_alpha = stage.add_value_tracker(0.0);
        for (i, at) in HALF_COUNT_AT.iter().enumerate() {
            let right = card_point(*at);
            count_back(stage, halves_alpha, v(right[0] - 0.10, right[1]), 0.40, 0.40)?;
            placement::live_count(
                &kit,
                stage,
                wire_count(wire, place, Some(i)),
                halves_alpha,
                HALF_SIZE,
                card_point(*at),
            )?;
        }
        at_once(
            stage,
            &cut,
            &[],
            vec![tween(wire.shade, 1.0, FADE_S)?, tween(halves_alpha, 1.0, FADE_S)?],
            FADE_S,
        )?;
        line.wait_for(stage, "Watch the cut")?;
        make(stage, place, &placement::Move::slide(first, GAP), CUT_SLIDE_S, linear)?;
        let exact = precisely(&kit, stage, CUT_EXACT)?;
        fade_in_all(stage, &[exact], FADE_S)?;

        // 12_8: the lower half across the upper; the sheet slid so a line
        // runs through their meeting point
        let line = say(&kit, stage, "12_8")?;
        line.wait_for(stage, "Lay one half across the other")?;
        let mut gone = cut.clone();
        gone.push(exact);
        fade_out_all(stage, &gone, FADE_S)?;
        stage.play(glide(wire.morph, 1.0, SWING_S, smooth)?)?;
        line.wait_for(stage, "a line through that point")?;
        let meet = [HOME[0], HOME[1] + LENGTH / 4.0];
        let through = meet[1].rem_euclid(GAP) - first.slide;
        make(stage, place, &placement::Move::slide(first, through), THROUGH_S, smooth)?;
        let mark = ring(stage, meet);
        fade_in_all(stage, &[mark], FADE_S)?;

        // 12_9: back in line; thirds, quarters; three quarters, and the
        // rule for every fraction
        let line = say(&kit, stage, "12_9")?;
        line.wait_for(stage, "Equal pieces have equal averages")?;
        at_once(
            stage,
            &[],
            &[mark],
            vec![
                glide(wire.morph, 0.0, SWING_S, smooth)?,
                tween(halves_alpha, 0.0, FADE_S)?,
            ],
            FADE_S,
        )?;
        move_sheet(stage, place, 0.0, -through, [0.0, 0.0], THROUGH_S)?;
        line.wait_for(stage, "each third a third")?;
        let low = HOME[1] - LENGTH / 2.0;
        let thirds = ticks_across(stage, &[low + LENGTH / 3.0, low + 2.0 * LENGTH / 3.0]);
        at_once(stage, &thirds, &[], vec![tween(wire.shade, 0.0, FADE_S)?], FADE_S)?;
        line.wait_for(stage, "each quarter a quarter")?;
        let quarters = ticks_across(
            stage,
            &[low + LENGTH / 4.0, low + LENGTH / 2.0, low + 3.0 * LENGTH / 4.0],
        );
        at_once(stage, &quarters, &thirds, vec![], FADE_S)?;
        line.wait_for(stage, "Three quarters together")?;
        let pts = vec![
            card_point([HOME[0], low]),
            card_point([HOME[0], low + 3.0 * LENGTH / 4.0]),
        ];
        let three = halo_along(stage, pts);
        let pieces = row(&kit, stage, &words.pieces, Palette::ink(), TOP_ROW)?;
        fade_in_all(stage, &[three, pieces], FADE_S)?;
        line.wait_for(stage, "So for every length that's a fraction")?;
        let ones = row(&kit, stage, &words.ones, Palette::ink(), LOW_ROW)?;
        fade_in_all(stage, &[ones], FADE_S)?;

        // 12_10: the line between the ends, 14 and 15 cm along it; the
        // sheet turned a quarter, then slid; the three counts
        let line = say(&kit, stage, "12_10")?;
        line.wait_for(stage, "Take the straight line between the coast's two ends")?;
        let mut dims = vec![
            tween(wire.alpha, 0.0, FADE_S)?,
            tween(wire_dots, 0.0, FADE_S)?,
            tween(wire_counts, 0.0, FADE_S)?,
        ];
        for &m in &shore.pieces {
            dims.push(dim_line(m, COAST_DIMMED, FADE_S)?);
        }
        let (a, b) = coast.ends();
        let apart = dist(a, b);
        let ends = path(stage, &[a, b], palette::length(), ENDS_W);
        at_once(
            stage,
            &[ends],
            &[three, pieces, ones, quarters[0], quarters[1], quarters[2]],
            dims,
            FADE_S,
        )?;
        line.wait_for(stage, "Lay a fourteen and a fifteen centimetre stick")?;
        let nested = [
            along_ends(&coast, NESTED[0]),
            along_ends(&coast, apart),
            along_ends(&coast, NESTED[1]),
        ];
        // each length's tip along the line from the start, and a tick
        // across it there
        let tip = |l: f64| [a[0] + l * (b[0] - a[0]) / apart, a[1] + l * (b[1] - a[1]) / apart];
        let across = |l: f64| {
            let (p, n) = (tip(l), [(a[1] - b[1]) / apart, (b[0] - a[0]) / apart]);
            [
                [p[0] - ENDS_STUB * n[0], p[1] - ENDS_STUB * n[1]],
                [p[0] + ENDS_STUB * n[0], p[1] + ENDS_STUB * n[1]],
            ]
        };
        let mut marks = vec![
            path(stage, &across(NESTED[0]), Palette::ink(), TICK_W),
            path(stage, &across(NESTED[1]), Palette::ink(), TICK_W),
            path(stage, &[b, tip(NESTED[1])], palette::stick(), CHAIN_W),
        ];
        let nested_alpha = stage.add_value_tracker(0.0);
        for (i, curve) in nested.iter().enumerate() {
            let at = v(NESTED_X[i], TOP_ROW[1]);
            let label = kit.text(stage, &words.nested[i], LABEL_SIZE, Palette::dim(), at)?;
            marks.push(left_at(stage, label, at));
            let c = curve.clone();
            let count = move |s: &fmn::mobject::Stage| count_at(&c, place.pose(s));
            let right = v(NESTED_X[i] + NESTED_COUNT_GAP, TOP_ROW[1]);
            placement::live_count(&kit, stage, count, nested_alpha, ROW_SIZE, right)?;
        }
        let ends_dots = stage.add_value_tracker(0.0);
        placement::live_dots(stage, place, &nested[2], ends_dots)?;
        at_once(
            stage,
            &marks,
            &[],
            vec![tween(nested_alpha, 1.0, FADE_S)?, tween(ends_dots, 1.0, FADE_S)?],
            FADE_S,
        )?;
        line.wait_for(stage, "Turn the sheet, slide it")?;
        let [turned, slid] = nested_moves(first);
        make(stage, place, &turned, TURN_S, smooth)?;
        move_sheet(stage, place, 0.0, slid.by, [0.0, 0.0], TURN_S)?;

        // 12_11: squeezed between tenths and hundredths
        let line = say(&kit, stage, "12_11")?;
        line.wait_for(stage, "between fourteen and fifteen times")?;
        let mut bracket = row(&kit, stage, &words.brackets[0], Palette::ink(), LOW_ROW)?;
        fade_in_all(stage, &[bracket], FADE_S)?;
        for (k, phrase) in [
            (1, "Between fourteen point seven"),
            (2, "between fourteen point seven three"),
        ] {
            line.wait_for(stage, phrase)?;
            let next = row(&kit, stage, &words.brackets[k], Palette::ink(), LOW_ROW)?;
            at_once(stage, &[next], &[bracket], vec![], FADE_S)?;
            bracket = next;
        }

        // 12_12: back to the first placement while the rule for chains is
        // said; the chain at the joints, its chord across the bay
        let line = say(&kit, stage, "12_12")?;
        let mut gone = marks.clone();
        gone.extend([ends, bracket]);
        at_once(
            stage,
            &[],
            &gone,
            vec![tween(nested_alpha, 0.0, FADE_S)?, tween(ends_dots, 0.0, FADE_S)?],
            FADE_S,
        )?;
        move_sheet(stage, place, 0.0, -slid.by, [0.0, 0.0], TURN_S)?;
        move_sheet(stage, place, -FRAC_PI_2, 0.0, NESTED_PIN, TURN_S)?;
        line.wait_for(stage, "Here's a chain")?;
        // each chain in the stick's colour, its dots in it too, beside the
        // coast's gold ones and its own count
        let at = chain_points(&coast);
        let chains: Vec<Curve> = at.iter().map(|s| chain(&coast, s)).collect();
        let drawn: Vec<Mob> = chains
            .iter()
            .map(|c| {
                let mut pts = vec![c.at(0.0)];
                pts.extend(c.parts.iter().map(|p| p.seg.point(1.0)));
                path(stage, &pts, palette::stick(), CHAIN_W)
            })
            .collect();
        let chain_dots: Vec<Mob> = chains.iter().map(|_| stage.add_value_tracker(0.0)).collect();
        for (c, &alpha) in chains.iter().zip(&chain_dots) {
            live_curve_dots(stage, place, c, palette::stick(), alpha)?;
        }
        let rows: Vec<Mob> = words
            .chains
            .iter()
            .map(|w| {
                let m = kit.text(stage, w, LABEL_SIZE, Palette::ink(), TOP_ROW)?;
                Ok(left_at(stage, m, TOP_ROW))
            })
            .collect::<fmn::Result<_>>()?;
        at_once(
            stage,
            &[drawn[0], rows[0]],
            &[],
            vec![
                tween(chain_dots[0], 1.0, FADE_S)?,
                tween(shore.count_alpha, 1.0, FADE_S)?,
                tween(shore.dots_alpha, 1.0, FADE_S)?,
            ],
            FADE_S,
        )?;
        line.wait_for(stage, "Its chord across the bay")?;
        let mut lit = vec![dim_line(shore.pieces[BAY], 1.0, FADE_S)?];
        for &m in &shore.bay {
            lit.push(dim_line(m, HALO_OPACITY, FADE_S)?);
        }
        together(stage, lit)?;

        // 12_13: never crossed more than the coast; finer chains
        let line = say(&kit, stage, "12_13")?;
        line.wait_for(stage, "Put more corners on the coast")?;
        for (k, phrase) in [
            (1, "Twenty-three point nine one centimetres"),
            (2, "then twenty-five point seven one"),
        ] {
            line.wait_for(stage, phrase)?;
            let mut moves = vec![
                tween(chain_dots[k - 1], 0.0, FADE_S)?,
                tween(chain_dots[k], 1.0, FADE_S)?,
            ];
            if k == 1 {
                moves.push(dim_line(shore.pieces[BAY], COAST_DIMMED, FADE_S)?);
                for &m in &shore.bay {
                    moves.push(dim_line(m, 0.0, FADE_S)?);
                }
            }
            at_once(stage, &[drawn[k], rows[k]], &[drawn[k - 1], rows[k - 1]], moves, FADE_S)?;
        }

        // 12_14: in the limit; the exact version below
        let line = say(&kit, stage, "12_14")?;
        line.wait_for(stage, "As the corners crowd together")?;
        let exact = precisely(&kit, stage, &words.limit)?;
        fade_in_all(stage, &[exact], FADE_S)?;
        line.wait_for(stage, "for a curve, it's a limit")?;

        // 12_15: which curve finds the number? The chain goes, the coast
        // comes back, the claim; the countdown
        let line = say(&kit, stage, "12_15")?;
        line.wait_for(stage, "One number, the same for every curve")?;
        let claim = row(&kit, stage, &words.claim, Palette::ink(), TOP_ROW)?;
        let mut back = vec![
            tween(chain_dots[2], 0.0, FADE_S)?,
            tween(shore.dots_alpha, 0.0, FADE_S)?,
            tween(shore.count_alpha, 0.0, FADE_S)?,
        ];
        for &m in &shore.pieces {
            back.push(dim_line(m, 1.0, FADE_S)?);
        }
        at_once(stage, &[claim], &[drawn[2], rows[2], exact], back, FADE_S)?;
        line.wait_for(stage, "Which would you use")?;
        let q = think(&kit, stage, THINK_AT)?;

        // 12_16: the coin, two meeting points wherever it lands: the sheet
        // slides a gap and turns half a turn about a point on a line, so
        // its lines end where they began; 2 = 2π × ?, so ? = 1 ÷ π
        let line = say(&kit, stage, "12_16")?;
        line.wait_for(stage, "The coin")?;
        let coin = coin_at_rest(stage, COIN_AT);
        let rim = coin_rim(COIN_AT);
        let [coin_dots, coin_counts] = [0.0, 0.0].map(|x| stage.add_value_tracker(x));
        placement::live_dots(stage, place, &rim, coin_dots)?;
        let counted = rim.clone();
        counter(
            &kit,
            stage,
            move |s: &fmn::mobject::Stage| count_at(&counted, place.pose(s)),
            coin_counts,
        )?;
        at_once(
            stage,
            &coin,
            &[q],
            vec![tween(coin_dots, 1.0, FADE_S)?, tween(coin_counts, 1.0, FADE_S)?],
            FADE_S,
        )?;
        line.wait_for(stage, "Away from exact tangencies")?;
        let tangent_note = precisely(&kit, stage, "exact tangencies change no average")?;
        fade_in_all(stage, &[tangent_note], FADE_S)?;
        make(stage, place, &placement::Move::slide(first, GAP), SLIDE_S / 2.0, smooth)?;
        move_sheet(stage, place, PI, 0.0, COIN_PIN, HALF_TURN_S)?;
        line.wait_for(stage, "So two is two pi times the number")?;
        fade_out_all(stage, &[tangent_note], FADE_S)?;
        let coin_row = row(&kit, stage, &words.coin, Palette::ink(), LOW_ROW)?;
        fade_in_all(stage, &[coin_row], FADE_S)?;
        line.wait_for(stage, "the number is one over pi")?;
        let calibrated = kit.tex(stage, &words.calibrated, ROW_SIZE, palette::length(), &[], LOW_ROW)?;
        stage.next_to(calibrated, coin_row, RIGHT, 0.5, ORIGIN);
        fade_in_all(stage, &[calibrated], FADE_S)?;

        // 12_17: every curve, length over π; the bet was forced; the
        // coast's 25.81 ÷ π ≈ 8.22
        let line = say(&kit, stage, "12_17")?;
        line.wait_for(stage, "every curve averages its length over pi crossings")?;
        let [theorem, payoff] = closing_rows(&kit, stage, &words)?;
        at_once(
            stage,
            &[theorem],
            &[claim, coin_row, calibrated],
            vec![tween(coin_dots, 0.0, FADE_S)?, tween(coin_counts, 0.0, FADE_S)?],
            FADE_S,
        )?;
        line.wait_for(stage, "twenty-five point eight one over pi")?;
        fade_in_all(stage, &[payoff], FADE_S)?;
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
    use crate::model::{CARD_CENTRE, CARD_H, CARD_UNIT, CARD_W};

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
            .replace(r"\div", "÷")
            .replace(r"\times", "×")
            .replace(r"\approx", "≈")
            .replace(r"\le", "≤")
            .replace(r"\tfrac{3}{4}", "¾")
            .replace(r"\pi", "π")
            .replace(['{', '}', '\\'], "")
    }

    fn bounds((at, w, h): (Vec3, f64, f64)) -> [f64; 4] {
        [at[0] - w / 2.0, at[0] + w / 2.0, at[1] - h / 2.0, at[1] + h / 2.0]
    }

    /// Placements on an even grid: `m` directions over half a turn, `n`
    /// slides over a gap, each at its cell's middle.
    fn grid(m: usize, n: usize) -> Vec<Pose> {
        (0..m)
            .flat_map(|i| {
                (0..n).map(move |j| Pose::new(PI * (i as f64 + 0.5) / m as f64, GAP * (j as f64 + 0.5) / n as f64))
            })
            .collect()
    }

    /// The count of a curve drawn as fine chords, each chord against each
    /// line by the floors of its ends' heights (not `sheet::crossings`).
    fn direct(points: &[P2], pose: Pose) -> usize {
        let n = [-pose.beta.sin(), pose.beta.cos()];
        let k = |p: P2| ((p[0] * n[0] + p[1] * n[1] - pose.slide) / GAP).floor() as i64;
        points
            .windows(2)
            .map(|w| (k(w[1]) - k(w[0])).unsigned_abs() as usize)
            .sum()
    }

    #[test]
    fn the_screen_words_are_built_from_the_facts() {
        let w = words().unwrap();
        assert_eq!(w.by_piece, [1, 0, 1, 1, 0, 1, 2, 0, 0, 0, 1]);
        assert_eq!(w.by_piece.iter().sum::<usize>(), 7);
        assert_eq!(
            w.pieces,
            r"\text{average for } 4.5\text{ cm} = 3 \times \text{average for } 1.5\text{ cm} = \tfrac{3}{4} \times \text{average for } 6\text{ cm}"
        );
        assert_eq!(w.nested, ["14 cm", "ends", "15 cm"]);
        assert_eq!(
            w.brackets,
            [
                r"14 \times ? \le \text{average} \le 15 \times ?",
                r"14.7 \times ? \le \text{average} \le 14.8 \times ?",
                r"14.73 \times ? \le \text{average} \le 14.74 \times ?",
            ]
        );
        assert_eq!(
            w.chains,
            [
                "chain 21.39 cm, 5 crossings",
                "chain 23.91 cm, 7 crossings",
                "chain 25.71 cm, 7 crossings"
            ]
        );
        assert_eq!(w.source, "Barbier, 1860, J. Math. Pures Appl., pages 275 to 277");
        assert_eq!(w.payoff, r"25.81 \div \pi \approx 8.22");
    }

    /// Crossings add: at 900 placements the pieces' counts add up to the
    /// coast's, and to a direct count on the coast drawn as fine chords. A
    /// count of crossing points rather than of pieces passing would not
    /// (the bay's two crossings on one line stay two).
    #[test]
    fn crossings_add_piece_by_piece() {
        let coast = model::coast();
        let fine = coast.between(0.0, coast.length, 0.005);
        for pose in grid(36, 25) {
            let all = sheet::crossings(&coast, pose.beta, pose.slide);
            let summed: usize = (0..coast.parts.len())
                .map(|i| all.iter().filter(|c| c.piece == i).count())
                .sum();
            assert_eq!(summed, all.len(), "{pose:?}");
            assert_eq!(direct(&fine, pose), all.len(), "{pose:?}");
        }
        let at_first = sheet::crossings(&coast, 0.0, first().slide);
        let bay: Vec<_> = at_first.iter().filter(|c| c.piece == BAY).collect();
        assert_eq!(bay.len(), 2);
        assert_eq!(bay[0].line, bay[1].line);
    }

    /// Moving the stick and moving the sheet the other way give the same
    /// count, by the sheet's own move (12_5), at the moves shown and at 400
    /// more; the moved stick's count differs from home's.
    #[test]
    fn moving_the_stick_is_moving_the_sheet_back() {
        let p = first();
        let home = placed(halves, HOME, ACROSS, 0.0);
        let moved = placed(halves, [HOME[0] + MOVE_RIGHT, HOME[1]], MOVE_TO, 0.0);
        assert_eq!(count_at(&moved, p), count_at(&home, moved_back(p).end()));
        assert_ne!(count_at(&moved, p), count_at(&home, p));
        let mut seed = 0x1213_u64;
        let mut next = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (seed >> 11) as f64 / (1u64 << 53) as f64
        };
        for _ in 0..400 {
            let (dx, dy, turn) = (3.0 * next() - 1.5, 3.0 * next() - 1.5, PI * next() - FRAC_PI_2);
            let sheet = Pose::new(PI * next(), GAP * next());
            let stick = placed(halves, [HOME[0] + dx, HOME[1] + dy], ACROSS + turn, 0.0);
            let n = [-sheet.beta.sin(), sheet.beta.cos()];
            let mut back = placement::Move::turn(sheet, -turn, HOME);
            back.by = -(n[0] * dx + n[1] * dy);
            assert_eq!(count_at(&stick, sheet), count_at(&home, back.end()));
        }
    }

    /// The halves add up to the whole stick through a slide of a gap,
    /// their own counts changing as a line passes the cut.
    #[test]
    fn the_halves_add_up_through_a_slide() {
        let wire = placed(halves, HOME, ACROSS, 0.0);
        let whole = Curve {
            parts: vec![Part {
                seg: Seg::Line {
                    from: [HOME[0], HOME[1] - LENGTH / 2.0],
                    to: [HOME[0], HOME[1] + LENGTH / 2.0],
                },
                start: 0.0,
            }],
            length: LENGTH,
        };
        let mut seen = std::collections::BTreeSet::new();
        for i in 0..400 {
            let pose = Pose::new(0.0, first().slide + GAP * (i as f64 + 0.5) / 400.0);
            let c = sheet::crossings(&wire, pose.beta, pose.slide);
            let low = c.iter().filter(|x| x.piece == 0).count();
            assert_eq!(c.len(), count_at(&whole, pose));
            seen.insert((low, c.len() - low));
        }
        assert_eq!(seen.into_iter().collect::<Vec<_>>(), [(1, 2), (2, 1)]);
        assert!((wire.length - LENGTH).abs() < 1e-12);
    }

    /// Crossed, the halves meet at the upper half's middle; the line
    /// through it counts each once, two, though it meets the card at one
    /// point there (counting points would give one); still 6 cm, every
    /// morph keeping the halves' lengths.
    #[test]
    fn the_crossed_halves_count_twice_at_their_meeting() {
        let crossed = placed(halves, HOME, ACROSS, 1.0);
        let meet = [HOME[0], HOME[1] + LENGTH / 4.0];
        let c = sheet::crossings(&crossed, 0.0, meet[1].rem_euclid(GAP));
        assert_eq!(c.len(), 2);
        for x in &c {
            assert!(dist(x.at, meet) < 1e-9);
        }
        let mut points: Vec<P2> = c.iter().map(|x| x.at).collect();
        points.dedup_by(|a, b| dist(*a, *b) < 1e-9);
        assert_eq!(points.len(), 1);
        for i in 0..=100 {
            let w = placed(halves, HOME, ACROSS, f64::from(i) / 100.0);
            for part in &w.parts {
                assert!((part.seg.len() - LENGTH / 2.0).abs() < 1e-12);
            }
        }
    }

    /// The line between the ends lies in each bracket; at 3,600 placements
    /// the 14 cm stick is crossed no more than it, nor it than the 15 cm
    /// stick, and the shown placements' every end keeps 0.25 cm from a
    /// line.
    #[test]
    fn nested_sticks_keep_their_order() {
        let coast = model::coast();
        let (a, b) = coast.ends();
        let apart = dist(a, b);
        let nested = [
            along_ends(&coast, NESTED[0]),
            along_ends(&coast, apart),
            along_ends(&coast, NESTED[1]),
        ];
        for pose in grid(120, 30) {
            let n: Vec<usize> = nested.iter().map(|c| count_at(c, pose)).collect();
            assert!(n[0] <= n[1] && n[1] <= n[2], "{pose:?} {n:?}");
        }
        for mv in nested_moves(first()) {
            let p = mv.end();
            let nn = [-p.beta.sin(), p.beta.cos()];
            for c in &nested {
                for end in [c.at(0.0), c.at(c.length)] {
                    let t = (end[0] * nn[0] + end[1] * nn[1] - p.slide) / GAP;
                    assert!((t - t.round()).abs() * GAP > 0.25, "{p:?} {end:?}");
                }
            }
        }
        assert!((nested_moves(first())[0].end().slide - NESTED_SLIDES[0]).abs() < 1e-12);
    }

    /// No chain is crossed more often than the coast at 3,600 placements,
    /// and finer chains match it more often; a chain whose corners sit off
    /// the coast (raised 0.3 cm) can be crossed more, a negative control on
    /// why the corners must lie on it.
    #[test]
    fn chains_are_never_crossed_more_than_the_coast() {
        let coast = model::coast();
        let chains: Vec<Curve> = chain_points(&coast).iter().map(|s| chain(&coast, s)).collect();
        let mut matched = [0; 3];
        for pose in grid(90, 40) {
            let c = count_at(&coast, pose);
            for (k, ch) in chains.iter().enumerate() {
                let n = count_at(ch, pose);
                assert!(n <= c, "{pose:?} {k}");
                matched[k] += usize::from(n == c);
            }
        }
        assert!(
            matched[0] < matched[1] && matched[1] < matched[2] && matched[2] < 3600,
            "{matched:?}"
        );
        let raised = Curve {
            parts: chains[0]
                .parts
                .iter()
                .map(|p| {
                    let (a, b) = (p.seg.point(0.0), p.seg.point(1.0));
                    Part {
                        seg: Seg::Line {
                            from: [a[0], a[1] + 0.3],
                            to: [b[0], b[1] + 0.3],
                        },
                        start: p.start,
                    }
                })
                .collect(),
            length: chains[0].length,
        };
        assert!(grid(90, 40).iter().any(|&p| count_at(&raised, p) > count_at(&coast, p)));
    }

    /// The coin meets the lines twice at every placement tried, its rim 2π
    /// cm, so 1/π a centimetre; the shown 25.81 over π is the shown 8.22,
    /// and the two-place mean times π would not give 25.81 (a negative
    /// control: the screen never multiplies the rounded mean back).
    #[test]
    fn the_coin_finds_the_number() {
        let rim = coin_rim(COIN_AT);
        for pose in grid(60, 50) {
            assert_eq!(count_at(&rim, pose), 2, "{pose:?}");
        }
        assert!((2.0 / rim.length - 1.0 / PI).abs() < 1e-15);
        assert_eq!(facts::fixed(25.81 / PI, 2), "8.22");
        assert_ne!(facts::fixed(8.22 * PI, 2), "25.81");
        // The strict-crossing convention excludes tangencies; the narration
        // now names this exceptional placement rather than saying every one.
        let tangent_rim = coin_rim([0.0, 1.0]);
        assert_eq!(count_at(&tangent_rim, Pose::new(0.0, 0.0)), 0);
        for phase in [-1e-6, 1e-6] {
            assert_eq!(count_at(&tangent_rim, Pose::new(0.0, phase)), 2);
        }
    }

    /// Every wire 12 shows keeps more than 0.15 cm from the coast and at
    /// least 0.3 cm inside the card, through every morph and move. The
    /// crossed pose keeps 0.3 cm from the coast; the coin keeps 0.5 cm.
    #[test]
    fn the_wires_keep_off_the_coast() {
        let coast = model::coast();
        let shore = coast.between(0.0, coast.length, 0.02);
        let near = |c: &Curve| {
            c.parts
                .iter()
                .flat_map(|p| (0..=60).map(move |k| p.seg.point(f64::from(k) / 60.0)))
                .map(|q| shore.iter().map(|&s| dist(s, q)).fold(f64::MAX, f64::min))
                .fold(f64::MAX, f64::min)
        };
        let inside = |c: &Curve| {
            c.parts.iter().all(|p| {
                [p.seg.point(0.0), p.seg.point(1.0)]
                    .iter()
                    // HOME's upper endpoint is exactly 4.2 cm: the
                    // intended 0.3 cm margin in a 9 cm card is inclusive.
                    .all(|q| q[0].abs() <= CARD_W / 2.0 - 0.3 + 1e-12 && q[1].abs() <= CARD_H / 2.0 - 0.3 + 1e-12)
            })
        };
        for i in 0..=40 {
            let u = f64::from(i) / 40.0;
            let w = placed(halves, HOME, ACROSS, u);
            assert!(near(&w) > 0.15 && inside(&w), "{u}");
            let m = placed(
                halves,
                [HOME[0] + MOVE_RIGHT * u, HOME[1]],
                ACROSS + (MOVE_TO - ACROSS) * u,
                0.0,
            );
            assert!(near(&m) > 0.15 && inside(&m), "{u}");
        }
        assert!(near(&placed(halves, HOME, ACROSS, 1.0)) > 0.3);
        assert!(near(&coin_rim(COIN_AT)) > 0.5);
    }

    /// The console rows fit inside it: the live sum, the rows of 12_9,
    /// 12_11, 12_16 and 12_17; the nested labels and counts apart. A
    /// two-line lower third (12_14) keeps clear of the top row, and the
    /// low row is clear of none, so 12 empties it first.
    #[test]
    fn the_console_keeps_its_rows() {
        let w = words().unwrap();
        let console = bounds(CONSOLE);
        assert!(sum_x(w.by_piece.len()) < console[1] - 0.2);
        for tex in [&w.pieces, &w.ones, &w.brackets[2], &w.theorem, &w.payoff, &w.claim] {
            assert!(LEFT_X + width(&shown(tex), ROW_SIZE) < console[1] - 0.2, "{tex}");
        }
        assert!(
            LEFT_X + width(&shown(&w.coin), ROW_SIZE) + 0.5 + width(&shown(&w.calibrated), ROW_SIZE) < console[1] - 0.2
        );
        for c in &w.chains {
            assert!(LEFT_X + width(c, LABEL_SIZE) < console[1] - 0.2);
        }
        for i in 0..3 {
            assert!(NESTED_X[i] + width(&w.nested[i], LABEL_SIZE) < NESTED_X[i] + NESTED_COUNT_GAP - DIGIT_W - 0.1);
            if i > 0 {
                assert!(NESTED_X[i] > NESTED_X[i - 1] + NESTED_COUNT_GAP + 0.3);
            }
        }
        let two_lines_top = -4.0 + 0.2 + 0.3 + 2.0 * 0.36;
        assert!(TOP_ROW[1] - half_height(ROW_SIZE) > two_lines_top + 0.05);
        assert!(LOW_ROW[1] + half_height(ROW_SIZE) > -4.0 + 0.2 + 0.3 + 0.36);
        assert!(w.limit.chars().count() <= 2 * 92 && w.source.chars().count() <= 92);
    }

    /// The halves' counts sit left of the stick, clear of it and the coast,
    /// on the card; the countdown's ring clear of the coin.
    #[test]
    fn the_counts_keep_their_room() {
        let coast = model::coast();
        let shore = coast.between(0.0, coast.length, 0.02);
        for at in HALF_COUNT_AT {
            assert!(HOME[0] - at[0] > 0.6);
            assert!(shore.iter().all(|&s| dist(s, at) > 0.6), "{at:?}");
            assert!(at[0] > -CARD_W / 2.0 + 0.4);
        }
        let ring_card = [
            (THINK_AT[0] - CARD_CENTRE[0]) / CARD_UNIT,
            (THINK_AT[1] - CARD_CENTRE[1]) / CARD_UNIT,
        ];
        assert!(dist(ring_card, COIN_AT) > COIN / 2.0 + super::super::draw::THINK_R / CARD_UNIT + 0.2);
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("12_1", "Why should it work for every curve"),
            ("12_1", "Here's the claim"),
            ("12_2", "Barbier made this argument"),
            ("12_2", "crossings add up"),
            ("12_3", "Cut the coast at its joints"),
            ("12_3", "The cove is crossed"),
            ("12_3", "the bay is crossed twice"),
            ("12_3", "Add up every piece's crossings"),
            ("12_3", "the coast's seven"),
            ("12_4", "Slide the sheet"),
            ("12_4", "So the pieces' averages add up"),
            ("12_5", "Now take our six centimetre stick"),
            ("12_5", "Move it"),
            ("12_5", "Put the stick back"),
            ("12_5", "move the sheet instead"),
            ("12_6", "only reshuffles them"),
            ("12_6", "It can only depend on its length"),
            ("12_7", "Cut the stick in half"),
            ("12_7", "Watch the cut"),
            ("12_8", "Lay one half across the other"),
            ("12_8", "a line through that point"),
            ("12_9", "Equal pieces have equal averages"),
            ("12_9", "each third a third"),
            ("12_9", "each quarter a quarter"),
            ("12_9", "Three quarters together"),
            ("12_9", "So for every length that's a fraction"),
            ("12_10", "Take the straight line between the coast's two ends"),
            ("12_10", "Lay a fourteen and a fifteen centimetre stick"),
            ("12_10", "Turn the sheet, slide it"),
            ("12_11", "between fourteen and fifteen times"),
            ("12_11", "Between fourteen point seven"),
            ("12_11", "between fourteen point seven three"),
            ("12_12", "Here's a chain"),
            ("12_12", "Its chord across the bay"),
            ("12_13", "Put more corners on the coast"),
            ("12_13", "Twenty-three point nine one centimetres"),
            ("12_13", "then twenty-five point seven one"),
            ("12_14", "As the corners crowd together"),
            ("12_14", "for a curve, it's a limit"),
            ("12_15", "One number, the same for every curve"),
            ("12_15", "Which would you use"),
            ("12_16", "The coin"),
            ("12_16", "Away from exact tangencies"),
            ("12_16", "So two is two pi times the number"),
            ("12_16", "the number is one over pi"),
            ("12_17", "every curve averages its length over pi crossings"),
            ("12_17", "twenty-five point eight one over pi"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }
}
