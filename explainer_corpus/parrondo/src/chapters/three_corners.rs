//! Chapter 06, three corners: the remainder becomes a place. The player as
//! 02 left them (the teal token, "player", "7 dollars", the seven chips in
//! rows of three with the leftover row outlined) under the coin badge. Game
//! B reads only what is left over, so the full rows dim, and every chip folds
//! into a small "7 dollars" counter at the top left; then three corners, one
//! for each remainder: 0 at the top, tinted red (the bad coin's corner), 1
//! lower right, 2 lower left (06_1). Seven leaves one, so the token itself
//! steps onto corner 1; a win is one corner forward, the green arrow 1 → 2,
//! and the counter reads 8 (06_2). From corner 2, where does one more win
//! take the player? (06_3, a prediction: no arrow leaves corner 2 yet)
//! Around to corner 0: eight and one is nine, which leaves nothing; a loss is
//! one corner back, the dim inside arrow 0 → 2, and the counter is 8 again;
//! then the other three arrows, every corner with a way forward and a way
//! back (06_4). Game B's chances on its arrows, corner 0's (the bad coin's)
//! in red (06_5). Game A's triangle on the left, as card A was, the same
//! chances from every corner, held beside B's (06_6).
//!
//! The walk is the player's: 7, 8, 9, 8 dollars, each count on the corner
//! of its remainder, the arrows drawn as the walk uses them. Every count,
//! corner and chance comes from `crate::model` and `crate::facts`; each
//! label is placed by its typeset size beyond its arrow's bow, and the
//! scene refuses to draw if the words would stop being true of the facts or
//! two labels would touch. Chapter 07 starts from this picture and builds
//! it with the same functions.

use std::f64::consts::{FRAC_PI_2, TAU};

use explainer_kit::{Kit, Palette, Timed, fade_all, rect, v};
use fmn::core::constants::FRAME_X_RADIUS;
use fmn::prelude::*;

use super::draw::{
    chip, curved_arrow, fade_in_all, fade_out_all, fade_to, fail, flash_all, flat, fly, keep_between, line_style, now,
    pause, say, screen_int, think, together, token, wait_until,
};
use super::looks_hopeless::{EDGE, PLAYER_LABEL_AT, ROOM, TOKEN_AT, badge_text, full_badge};
use crate::facts;
use crate::model::{Game, MODULUS, PUBLISHED, Ratio, START};
use crate::palette;

pub struct ThreeCorners {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const PLAYER: &str = "player";
const DOLLARS: &str = "dollars";
const LEFT_OVER: &str = "left over";
const A_TITLE: &str = "game A";
const B_TITLE: &str = "game B";

/// Where each picture lands: (line, the phrase it waits for).
const AT_LOOKS: (&str, &str) = ("06_1", "only ever looks at the remainder");
const AT_TRACK: (&str, &str) = ("06_1", "track just that");
const AT_DRAW: (&str, &str) = ("06_1", "Draw three corners");
const AT_NAMES: (&str, &str) = ("06_1", "zero, one and two");
const AT_ONE_OVER: (&str, &str) = ("06_2", "leave one over");
const AT_STANDS: (&str, &str) = ("06_2", "stands at corner one");
const AT_WIN: (&str, &str) = ("06_2", "A win moves them");
const AT_TO_TWO: (&str, &str) = ("06_2", "to corner two");
const AT_AROUND: (&str, &str) = ("06_4", "Around to corner zero");
const AT_NINE: (&str, &str) = ("06_4", "nine leaves nothing over");
const AT_LOSS: (&str, &str) = ("06_4", "A loss always moves");
const AT_WRITE: (&str, &str) = ("06_5", "write game B");
const AT_BAD: (&str, &str) = ("06_5", "the bad coin");
const AT_BAD_PCT: (&str, &str) = ("06_5", "forward nine and a half");
const AT_GOOD: (&str, &str) = ("06_5", "the good coin");
const AT_GOOD_PCT: (&str, &str) = ("06_5", "forward seventy-four");
const AT_GAME_A: (&str, &str) = ("06_6", "Game A is the same");
const AT_A_FORWARD: (&str, &str) = ("06_6", "forward forty-nine");
const AT_A_BACK: (&str, &str) = ("06_6", "back fifty and a half");
#[cfg(test)]
const CUES: [(&str, &str); 19] = [
    AT_LOOKS,
    AT_TRACK,
    AT_DRAW,
    AT_NAMES,
    AT_ONE_OVER,
    AT_STANDS,
    AT_WIN,
    AT_TO_TWO,
    AT_AROUND,
    AT_NINE,
    AT_LOSS,
    AT_WRITE,
    AT_BAD,
    AT_BAD_PCT,
    AT_GOOD,
    AT_GOOD_PCT,
    AT_GAME_A,
    AT_A_FORWARD,
    AT_A_BACK,
];

// ------------------------------------------------------------------ layout

/// The player where 01 to 05 kept them, and 02's rows of three.
const COUNT_AT: Vec3 = [-4.9, 1.5, 0.0];
const DOLLARS_X: f64 = -4.1;
const GRID_X0: f64 = -3.9;
const GRID_Y0: f64 = 0.9;
const PITCH: f64 = 0.42;
const LABEL_GAP: f64 = 0.15;
/// The full rows while the leftover one is pointed at.
const DIM: f64 = 0.3;

/// The counter the chips fold into, top left, and the remainder line under
/// it; both keep left of game A's corner 0.
const HUD_COUNT_AT: Vec3 = [-6.75, 3.3, 0.0];
const HUD_DOLLARS_X: f64 = -6.4;
const HUD_REM_AT: Vec3 = [-6.75, 2.85, 0.0];
const HUD_RIGHT: f64 = A_AT[0] - CORNER_R - ROOM;

/// The two triangles, game A's on the left and B's on the right (as the
/// cards were): their centres, the corners' distance from the centre and
/// the corner discs.
pub(super) const A_AT: Vec3 = [-3.9, -0.1, 0.0];
pub(super) const B_AT: Vec3 = [2.3, -0.1, 0.0];
const R: f64 = 2.5;
pub(super) const CORNER_R: f64 = 0.42;
/// An arrow starts and stops this far from a corner's centre, runs this far
/// off its edge (outside forward, inside back) and bows this much more.
const ARROW_GAP: f64 = 0.52;
const ARROW_OFF: f64 = 0.1;
const BOW: f64 = 0.24;
/// A label's box keeps this far beyond its arrow's bow; a corner's name
/// this far from its disc.
const BOW_GAP: f64 = 0.1;
const NAME_GAP: f64 = 0.18;
const CHANCE_SIZE: f64 = 20.0;
const NAME_SIZE: f64 = 24.0;
/// The titles stand under the triangles, this far below their centres.
pub(super) const TITLE_DY: f64 = 2.2;
/// The faint edges drawn before the arrows (fainter than the lint's ink).
const EDGE_INK: f64 = 0.3;
/// 06_3: the prediction, where game A's triangle will stand.
const THINK_AT: Vec3 = [-3.9, 0.0, 0.0];
/// Seconds: the chips folding, one step of the token, the beat after the
/// six arrows, and how long the two triangles stand side by side.
const FOLD_S: f64 = 0.9;
const STEP_S: f64 = 0.7;
const BEAT: f64 = 1.0;
const COMPARE_S: f64 = 4.0;

// ---------------------------------------------------------- the triangles

/// Corner `i` of the triangle about `at`: 0 at the top, then clockwise, 1
/// lower right and 2 lower left (the storyboard's stable object).
pub(super) fn corner(at: Vec3, i: usize) -> Vec3 {
    let a = FRAC_PI_2 - TAU / 3.0 * i as f64;
    v(at[0] + R * a.cos(), at[1] + R * a.sin())
}

/// The corner a count stands on: its remainder 0, 1 or 2, a count below
/// zero included (−1 leaves 2, as K03 says).
pub(super) fn corner_of(count: i64) -> usize {
    usize::try_from(count.rem_euclid(MODULUS)).expect("a remainder is 0, 1 or 2")
}

/// Edge `e`, from corner `e` to corner `e + 1` (a forward step, clockwise):
/// its midpoint, the unit direction forward and the unit normal out of the
/// triangle (clockwise travel keeps the inside on its right).
fn edge(at: Vec3, e: usize) -> (Vec3, [f64; 2], [f64; 2]) {
    let (p, q) = (corner(at, e), corner(at, (e + 1) % 3));
    let len = (q[0] - p[0]).hypot(q[1] - p[1]);
    let u = [(q[0] - p[0]) / len, (q[1] - p[1]) / len];
    (v((p[0] + q[0]) / 2.0, (p[1] + q[1]) / 2.0), u, [-u[1], u[0]])
}

/// Where the arrow along edge `e` starts and ends: the forward one from
/// corner `e` to `e + 1`, just outside the edge; the back one the other way,
/// just inside it.
pub(super) fn arrow_ends(at: Vec3, e: usize, forward: bool) -> (Vec3, Vec3) {
    let (_, u, n) = edge(at, e);
    let (p, q) = (corner(at, e), corner(at, (e + 1) % 3));
    let side = if forward { ARROW_OFF } else { -ARROW_OFF };
    let near_p = v(
        p[0] + u[0] * ARROW_GAP + n[0] * side,
        p[1] + u[1] * ARROW_GAP + n[1] * side,
    );
    let near_q = v(
        q[0] - u[0] * ARROW_GAP + n[0] * side,
        q[1] - u[1] * ARROW_GAP + n[1] * side,
    );
    if forward { (near_p, near_q) } else { (near_q, near_p) }
}

/// The middle of the arrow along edge `e`, where `curved_arrow`'s bow (to
/// the left of travel: out of the triangle forward, into it back) is
/// furthest from the edge: half the bow past the arrow's offset.
pub(super) fn apex(at: Vec3, e: usize, forward: bool) -> Vec3 {
    let (m, _, n) = edge(at, e);
    let s = (ARROW_OFF + BOW / 2.0) * if forward { 1.0 } else { -1.0 };
    v(m[0] + n[0] * s, m[1] + n[1] * s)
}

/// The centre of a `w` by `h` label for the arrow along edge `e`: beyond
/// the bow, outside the triangle for a forward arrow and inside it for a
/// back one, the box's nearest side [`BOW_GAP`] past the bow.
pub(super) fn chance_at(at: Vec3, e: usize, forward: bool, w: f64, h: f64) -> Vec3 {
    let (_, _, n) = edge(at, e);
    let a = apex(at, e, forward);
    let reach = (w / 2.0 * n[0].abs() + h / 2.0 * n[1].abs() + BOW_GAP) * if forward { 1.0 } else { -1.0 };
    v(a[0] + n[0] * reach, a[1] + n[1] * reach)
}

/// The centre of corner `i`'s `w` by `h` name: straight out from the
/// triangle's centre, clear of the disc (and of the token on it).
pub(super) fn name_at(at: Vec3, i: usize, w: f64, h: f64) -> Vec3 {
    let p = corner(at, i);
    let d = [(p[0] - at[0]) / R, (p[1] - at[1]) / R];
    let reach = CORNER_R + NAME_GAP + w / 2.0 * d[0].abs() + h / 2.0 * d[1].abs();
    v(p[0] + d[0] * reach, p[1] + d[1] * reach)
}

/// The arrow a step from corner `from` to corner `to` uses: (edge, forward).
pub(super) fn edge_of(from: usize, to: usize) -> (usize, bool) {
    if to == (from + 1) % 3 {
        (from, true)
    } else {
        (to, false)
    }
}

/// Which game a triangle carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Which {
    A,
    B,
}

impl Which {
    pub(super) fn at(self) -> Vec3 {
        match self {
            Self::A => A_AT,
            Self::B => B_AT,
        }
    }

    fn title(self) -> (&'static str, Srgb) {
        match self {
            Self::A => (A_TITLE, palette::game_a()),
            Self::B => (B_TITLE, palette::game_b()),
        }
    }

    /// Forward arrows: game A's blue; game B's green, the forward steps.
    /// Back arrows are dim in both.
    fn arrow_color(self, forward: bool) -> Srgb {
        match (self, forward) {
            (_, false) => Palette::dim(),
            (Self::A, true) => palette::game_a(),
            (Self::B, true) => palette::good(),
        }
    }

    /// Corner `i`'s disc: game B's corner 0 is tinted red, the bad coin's.
    fn disc(self, i: usize) -> Style {
        if self == Self::B && i == 0 {
            flat(palette::bad(), 0.55)
        } else {
            flat(Palette::faint(), 1.0)
        }
    }

    /// The label on the arrow along edge `e`: forward, the chance of a win
    /// at corner `e`; back, of a loss at corner `e + 1`. Game B's corner 0
    /// (the bad coin) is red both ways; otherwise a label takes its arrow's
    /// color.
    fn chance_color(self, e: usize, forward: bool) -> Srgb {
        let at = if forward { e } else { (e + 1) % 3 };
        if self == Self::B && at == 0 {
            palette::bad()
        } else {
            self.arrow_color(forward)
        }
    }
}

/// A triangle's corner discs. Detached.
pub(super) fn discs(stage: &mut Stage<'_>, which: Which) -> [Mob; 3] {
    [0, 1, 2].map(|i| {
        stage.arena_mut().add(
            Circle::new()
                .radius(CORNER_R)
                .arc_center(corner(which.at(), i))
                .style(which.disc(i))
                .build(),
        )
    })
}

/// Text `words` placed by its typeset size: centred at `place(width,
/// height)`. Detached.
fn placed(
    kit: &Kit,
    stage: &mut Stage<'_>,
    words: &str,
    size: f64,
    color: Srgb,
    place: impl Fn(f64, f64) -> Vec3,
) -> fmn::Result<Mob> {
    let m = kit.text(stage, words, size, color, ORIGIN)?;
    let at = place(stage.get_width(m), stage.get_height(m));
    stage.move_to(m, at, ORIGIN);
    Ok(m)
}

/// The corners' names, "0", "1" and "2", each outside its disc. Detached.
pub(super) fn names(kit: &Kit, stage: &mut Stage<'_>, which: Which) -> fmn::Result<[Mob; 3]> {
    let at = which.at();
    let mut out = Vec::with_capacity(3);
    for i in 0..3 {
        out.push(placed(
            kit,
            stage,
            &i.to_string(),
            NAME_SIZE,
            Palette::ink(),
            |w, h| name_at(at, i, w, h),
        )?);
    }
    out.try_into().map_err(|_| fail("a triangle has three corners".into()))
}

/// The game's name under its triangle. Detached.
pub(super) fn title(kit: &Kit, stage: &mut Stage<'_>, which: Which) -> fmn::Result<Mob> {
    let (words, color) = which.title();
    let at = which.at();
    kit.text(stage, words, 24.0, color, v(at[0], at[1] - TITLE_DY))
}

/// The arrow along edge `e`, forward or back: shaft and head. Detached.
pub(super) fn arrow_along(stage: &mut Stage<'_>, which: Which, e: usize, forward: bool) -> [Mob; 2] {
    let (from, to) = arrow_ends(which.at(), e, forward);
    let width = if forward { 3.0 } else { 2.5 };
    curved_arrow(stage, from, to, BOW, which.arrow_color(forward), width)
}

/// The faint edges of a triangle, drawn before its arrows. Detached.
fn faint_edges(stage: &mut Stage<'_>, which: Which) -> [Mob; 3] {
    let at = which.at();
    [0, 1, 2].map(|e| {
        let (p, q) = (corner(at, e), corner(at, (e + 1) % 3));
        stage.arena_mut().add(
            polyline([p, q])
                .style(
                    Style::default()
                        .fill(Palette::dim(), 0.0)
                        .stroke(Palette::dim(), 2.0, EDGE_INK),
                )
                .build(),
        )
    })
}

/// A whole triangle: discs, names, title, and the forward and back arrows
/// along edges 0, 1 and 2. Detached.
pub(super) struct Triangle {
    pub discs: [Mob; 3],
    pub names: [Mob; 3],
    pub title: Mob,
    pub forward: [[Mob; 2]; 3],
    pub back: [[Mob; 2]; 3],
}

impl Triangle {
    pub(super) fn parts(&self) -> Vec<Mob> {
        let mut all = self.discs.to_vec();
        all.extend(self.names);
        all.push(self.title);
        all.extend(self.forward.iter().flatten());
        all.extend(self.back.iter().flatten());
        all
    }
}

pub(super) fn triangle(kit: &Kit, stage: &mut Stage<'_>, which: Which) -> fmn::Result<Triangle> {
    Ok(Triangle {
        discs: discs(stage, which),
        names: names(kit, stage, which)?,
        title: title(kit, stage, which)?,
        forward: [0, 1, 2].map(|e| arrow_along(stage, which, e, true)),
        back: [0, 1, 2].map(|e| arrow_along(stage, which, e, false)),
    })
}

/// A game's chances at each corner, as exact percentages ("9.5"): of a win
/// (heads) and of a loss.
pub(super) struct Chances {
    pub heads: [String; 3],
    pub tails: [String; 3],
}

pub(super) fn chances(g: &Game) -> Result<Chances, String> {
    let [h0, h1, h2] = [0, 1, 2].map(|i| g.chance(i).exact_pct());
    let [t0, t1, t2] = [0, 1, 2].map(|i| (Ratio::ONE - g.chance(i)).exact_pct());
    Ok(Chances {
        heads: [h0?, h1?, h2?],
        tails: [t0?, t1?, t2?],
    })
}

/// The six labels of a triangle's arrows: forward along edges 0, 1, 2 (the
/// chance of a win at corners 0, 1, 2), then back along them (of a loss at
/// corners 1, 2, 0). Detached.
pub(super) fn chance_labels(kit: &Kit, stage: &mut Stage<'_>, which: Which, c: &Chances) -> fmn::Result<[Mob; 6]> {
    let at = which.at();
    let mut out = Vec::with_capacity(6);
    for forward in [true, false] {
        for e in 0..3 {
            let pct = if forward { &c.heads[e] } else { &c.tails[(e + 1) % 3] };
            out.push(placed(
                kit,
                stage,
                &format!("{pct} %"),
                CHANCE_SIZE,
                which.chance_color(e, forward),
                |w, h| chance_at(at, e, forward, w, h),
            )?);
        }
    }
    out.try_into().map_err(|_| fail("a triangle has six arrows".into()))
}

// ------------------------------------------------------------ the counter

/// The counter the chips fold into: the player's dollars, then "dollars".
/// Detached.
pub(super) fn counter(kit: &Kit, stage: &mut Stage<'_>, count: i64) -> fmn::Result<[Mob; 2]> {
    let number = counter_number(kit, stage, count)?;
    let word = kit.text_left(stage, DOLLARS, 22.0, Palette::dim(), v(HUD_DOLLARS_X, HUD_COUNT_AT[1]))?;
    keep_between(stage, word, HUD_DOLLARS_X - 0.01, HUD_RIGHT, "the counter's word")?;
    Ok([number, word])
}

/// The counter's number alone, rewritten at each step. Detached.
pub(super) fn counter_number(kit: &Kit, stage: &mut Stage<'_>, count: i64) -> fmn::Result<Mob> {
    let m = kit.text_left(stage, &screen_int(count), 28.0, palette::player(), HUD_COUNT_AT)?;
    keep_between(
        stage,
        m,
        -FRAME_X_RADIUS + EDGE,
        HUD_DOLLARS_X - 0.05,
        "the counter's number",
    )?;
    Ok(m)
}

/// "8 → remainder 2", as 02 wrote its counts.
pub(super) fn remainder_text(count: i64) -> String {
    format!("{} \u{2192} remainder {}", screen_int(count), count.rem_euclid(MODULUS))
}

/// The remainder line under the counter, red when nothing is left over.
/// Detached.
pub(super) fn remainder_line(kit: &Kit, stage: &mut Stage<'_>, count: i64) -> fmn::Result<Mob> {
    let color = if corner_of(count) == 0 {
        palette::bad()
    } else {
        Palette::ink()
    };
    let m = kit.text_left(stage, &remainder_text(count), 20.0, color, HUD_REM_AT)?;
    keep_between(stage, m, -FRAME_X_RADIUS + EDGE, HUD_RIGHT, "the remainder line")?;
    Ok(m)
}

/// The remainder the facts print for `count` ("6:0,7:1,…").
pub(super) fn printed_remainder(examples: &str, count: i64) -> Result<i64, String> {
    examples
        .split(',')
        .find_map(|e| e.strip_prefix(&format!("{count}:")))
        .ok_or_else(|| format!("no remainder fact for {count}"))?
        .parse()
        .map_err(|e| format!("remainder of {count}: {e}"))
}

// --------------------------------------------------------------- playback

/// Animations fading `old` out while `new` fades in, `rt` seconds: a text
/// rewritten by replacing it, never by transforming its glyphs.
pub(super) fn swap(stage: &mut Stage<'_>, old: &[Mob], new: &[Mob], rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::with_capacity(old.len() + new.len());
    for &m in old {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    for &m in new {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    Ok(anims)
}

/// Refuse a picture in which two of `mobs` would stand closer than `room`
/// (their boxes as typeset): labels running into each other.
pub(super) fn keep_apart(stage: &Stage<'_>, mobs: &[Mob], room: f64, what: &str) -> fmn::Result<()> {
    let boxes: Vec<[f64; 4]> = mobs
        .iter()
        .map(|&m| {
            [
                stage.get_bounding_box_point(m, LEFT)[0],
                stage.get_bounding_box_point(m, DOWN)[1],
                stage.get_bounding_box_point(m, RIGHT)[0],
                stage.get_bounding_box_point(m, UP)[1],
            ]
        })
        .collect();
    for (i, a) in boxes.iter().enumerate() {
        for b in &boxes[i + 1..] {
            let apart = a[2] + room <= b[0] || b[2] + room <= a[0] || a[3] + room <= b[1] || b[3] + room <= a[1];
            if !apart {
                return Err(fail(format!(
                    "{what}: two labels touch, at x {:.2} to {:.2} and {:.2} to {:.2}",
                    a[0], a[2], b[0], b[2]
                )));
            }
        }
    }
    Ok(())
}

/// The token steps to the corner of `count` on game B's triangle while
/// `old` (the counter's number, perhaps the remainder line) gives way to
/// `new`.
fn step(stage: &mut Stage<'_>, player: Mob, count: i64, old: &[Mob], new: &[Mob]) -> fmn::Result<()> {
    let mut anims = vec![stage.prepare(fly(player, corner(B_AT, corner_of(count)), STEP_S)?)?];
    anims.extend(swap(stage, old, new, STEP_S)?);
    stage.play_prepared(anims)?;
    Ok(())
}

// ---------------------------------------------------------------- numbers

/// The player's dollars through 06_2 and 06_4: seven, a win, a win, then a
/// loss. Chapter 07 starts where the walk ends.
pub(super) fn walk() -> [i64; 4] {
    [START, START + 1, START + 2, START + 1]
}

/// The chapter's numbers, checked against the words they illustrate.
struct Numbers {
    badge: String,
    /// The walk of 06_2 and 06_4 (seven dollars, a win, a win, a loss): the
    /// dollars and the corner after each step, from the start.
    walk: [(i64, usize); 4],
    a: Chances,
    b: Chances,
}

fn numbers() -> Result<Numbers, String> {
    let f = facts::facts()?;
    let get = |k: &str| f.get(k).map(str::to_owned).ok_or_else(|| format!("no fact {k}"));
    if get("rules.modulus")? != "3" || MODULUS != 3 {
        return Err("06_1 draws three corners, but game B divides by another number".into());
    }
    let examples = get("remainder.examples")?;
    for count in walk() {
        if printed_remainder(&examples, count)? != count.rem_euclid(MODULUS) {
            return Err(format!("the remainder fact for {count} is not its remainder"));
        }
    }
    let path = walk().map(|count| (count, corner_of(count)));
    // the words: seven leaves one, eight two, nine nothing, then eight again
    if path != [(7, 1), (8, 2), (9, 0), (8, 2)] {
        return Err(format!(
            "06_2 and 06_4 walk 7, 8, 9, 8 to corners 1, 2, 0, 2, but the facts give {path:?}"
        ));
    }
    // a win is one corner forward and a loss one back, round the triangle
    for pair in path.windows(2) {
        let ((from_count, from), (to_count, to)) = (pair[0], pair[1]);
        let expected = if to_count > from_count {
            (from + 1) % 3
        } else {
            (from + 2) % 3
        };
        if to != expected {
            return Err(format!("{from_count} to {to_count} dollars is not one corner"));
        }
    }
    let (a, b) = (chances(&PUBLISHED.game_a())?, chances(&PUBLISHED.game_b())?);
    // 06_5: the bad coin at corner 0, the good one at 1 and 2; 06_6: game
    // A's one coin at every corner, forward 49.5 and back 50.5
    fn words(c: &[String; 3]) -> [&str; 3] {
        c.each_ref().map(String::as_str)
    }
    if words(&b.heads) != ["9.5", "74.5", "74.5"] || words(&a.heads) != ["49.5"; 3] || words(&a.tails) != ["50.5"; 3] {
        return Err("06_5 and 06_6 say the published coins' chances, but the model's differ".into());
    }
    let heads = format!("{},{},{}", a.heads[0], b.heads[0], b.heads[1]);
    let tails = format!("{},{},{}", a.tails[0], b.tails[0], b.tails[1]);
    if get("coins.published.heads.pct")? != heads || get("coins.published.tails.pct")? != tails {
        return Err("the arrows' chances disagree with the facts".into());
    }
    Ok(Numbers {
        badge: badge_text(&f)?,
        walk: path,
        a,
        b,
    })
}

// --------------------------------------------------------------- 02's rows

/// Chip `i` in 02's rows of three.
fn slot(i: usize) -> Vec3 {
    v(GRID_X0 + PITCH * (i % 3) as f64, GRID_Y0 - PITCH * (i / 3) as f64)
}

/// 02's outlined leftover row for `count` chips and its label. Detached.
fn leftover(kit: &Kit, stage: &mut Stage<'_>, count: usize) -> fmn::Result<[Mob; 2]> {
    let y = GRID_Y0 - PITCH * (count / 3) as f64;
    let outline = rect(
        v(GRID_X0 - PITCH / 2.0, y - PITCH / 2.0),
        3.0 * PITCH,
        PITCH,
        line_style(Palette::hero(), 2.5),
    )?;
    let outline = stage.arena_mut().add(outline);
    let label = kit.text_left(
        stage,
        LEFT_OVER,
        20.0,
        Palette::hero(),
        v(GRID_X0 + 2.5 * PITCH + LABEL_GAP, y),
    )?;
    keep_between(
        stage,
        label,
        GRID_X0 + 2.5 * PITCH,
        corner(B_AT, 2)[0] - CORNER_R - ROOM,
        "the leftover label",
    )?;
    Ok([outline, label])
}

impl SceneConstruct for ThreeCorners {
    fn name(&self) -> &str {
        "06_three_corners"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let n = numbers().map_err(fail)?;
        let start = n.walk[0].0;
        let chips_n = usize::try_from(start).map_err(|_| fail("the player starts below zero".into()))?;

        // 06_1: the player as 02 left them; only the leftover matters to
        // game B, so the chips fold into a small counter; three corners
        let line = say(&kit, stage, "06_1")?;
        let player = token(stage, TOKEN_AT);
        let label = kit.text(stage, PLAYER, 18.0, Palette::dim(), PLAYER_LABEL_AT)?;
        let count = kit.text_left(stage, &screen_int(start), 34.0, palette::player(), COUNT_AT)?;
        let dollars = kit.text_left(stage, DOLLARS, 26.0, Palette::dim(), v(DOLLARS_X, COUNT_AT[1]))?;
        let chips: Vec<Mob> = (0..chips_n).map(|i| chip(stage, slot(i))).collect();
        let rows = leftover(&kit, stage, chips_n)?;
        let tag = full_badge(&kit, stage, &n.badge, 0.0)?;
        let mut arriving = vec![player, label, count, dollars];
        arriving.extend(&chips);
        arriving.extend(rows);
        arriving.extend(tag);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, AT_LOOKS.1)?;
        let full = chips_n / 3 * 3;
        let dimming = chips[..full]
            .iter()
            .map(|&c| fade_to(c, DIM, 0.5))
            .collect::<fmn::Result<Vec<_>>>()?;
        together(stage, dimming)?;
        flash_all(stage, &[rows[0]], 0.6)?;
        line.wait_for(stage, AT_TRACK.1)?;
        // every chip, and the big count with them, flows into the counter
        let shown = counter(&kit, stage, start)?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        let folding: Vec<Mob> = chips.iter().copied().chain([count, dollars]).collect();
        for m in folding {
            let c = stage.get_center(m);
            let into = fade_out(
                stage.arena_mut(),
                m,
                v(HUD_COUNT_AT[0] - c[0], HUD_COUNT_AT[1] - c[1]),
                1.0,
            )?;
            anims.push(stage.prepare(into.rt(FOLD_S))?);
        }
        anims.extend(swap(stage, &rows, &shown, FOLD_S)?);
        stage.play_prepared(anims)?;
        let mut number = shown[0];
        line.wait_for(stage, AT_DRAW.1)?;
        let edges = faint_edges(stage, Which::B);
        let b_discs = discs(stage, Which::B);
        fade_in_all(stage, &edges, 0.4)?;
        for &d in &b_discs {
            fade_in_all(stage, &[d], 0.3)?;
        }
        // the token, added before the corners, stays above them
        stage.scene_mut().bring_to_front(&[player])?;
        line.wait_for(stage, AT_NAMES.1)?;
        let b_names = names(&kit, stage, Which::B)?;
        if stage.get_bounding_box_point(b_names[0], UP)[1] > stage.get_bounding_box_point(tag[0], DOWN)[1] - 0.1 {
            return Err(fail("corner 0's name runs into the coin badge".into()));
        }
        for &m in &b_names {
            fade_in_all(stage, &[m], 0.25)?;
        }

        // 06_2: seven leaves one, so the token itself steps onto corner 1;
        // a win is one corner forward
        let line = say(&kit, stage, "06_2")?;
        line.wait_for(stage, AT_ONE_OVER.1)?;
        let mut rem = remainder_line(&kit, stage, start)?;
        fade_in_all(stage, &[rem], 0.4)?;
        line.wait_for(stage, AT_STANDS.1)?;
        let mut anims = vec![stage.prepare(fly(player, corner(B_AT, n.walk[0].1), 1.0)?)?];
        let away = fade_out(stage.arena_mut(), label, ORIGIN, 1.0)?;
        anims.push(stage.prepare(away.rt(0.5))?);
        stage.play_prepared(anims)?;
        flash_all(stage, &[b_discs[n.walk[0].1]], 0.5)?;
        line.wait_for(stage, AT_WIN.1)?;
        let (e, forward) = edge_of(n.walk[0].1, n.walk[1].1);
        let win = arrow_along(stage, Which::B, e, forward);
        fade_in_all(stage, &win, 0.4)?;
        line.wait_for(stage, AT_TO_TWO.1)?;
        let to = n.walk[1].0;
        let (next_number, next_rem) = (counter_number(&kit, stage, to)?, remainder_line(&kit, stage, to)?);
        step(stage, player, to, &[number, rem], &[next_number, next_rem])?;
        (number, rem) = (next_number, next_rem);

        // 06_3: from corner 2, where does one more win go? (a prediction;
        // no arrow leaves corner 2 yet)
        say(&kit, stage, "06_3")?;
        let q = think(&kit, stage, THINK_AT)?;

        // 06_4: around to corner 0, nine leaves nothing; a loss is one
        // corner back; then every corner's way forward and way back
        let line = say(&kit, stage, "06_4")?;
        fade_out_all(stage, &[q], 0.2)?;
        line.wait_for(stage, AT_AROUND.1)?;
        let (e, forward) = edge_of(n.walk[1].1, n.walk[2].1);
        let wrap = arrow_along(stage, Which::B, e, forward);
        fade_in_all(stage, &wrap, 0.4)?;
        // the count turns 9 and its old remainder goes; the new one comes
        // with its reason
        let to = n.walk[2].0;
        let next_number = counter_number(&kit, stage, to)?;
        step(stage, player, to, &[number, rem], &[next_number])?;
        number = next_number;
        line.wait_for(stage, AT_NINE.1)?;
        rem = remainder_line(&kit, stage, to)?;
        fade_in_all(stage, &[rem], 0.4)?;
        flash_all(stage, &[b_discs[n.walk[2].1]], 0.6)?;
        line.wait_for(stage, AT_LOSS.1)?;
        let (e, forward) = edge_of(n.walk[2].1, n.walk[3].1);
        let loss = arrow_along(stage, Which::B, e, forward);
        fade_in_all(stage, &loss, 0.4)?;
        let to = n.walk[3].0;
        let (next_number, next_rem) = (counter_number(&kit, stage, to)?, remainder_line(&kit, stage, to)?);
        step(stage, player, to, &[number, rem], &[next_number, next_rem])?;
        let shown = [next_number, shown[1], next_rem];
        let used: Vec<(usize, bool)> = n.walk.windows(2).map(|p| edge_of(p[0].1, p[1].1)).collect();
        let mut rest = Vec::new();
        for e in 0..3 {
            for forward in [true, false] {
                if !used.contains(&(e, forward)) {
                    rest.extend(arrow_along(stage, Which::B, e, forward));
                }
            }
        }
        fade_in_all(stage, &rest, 0.6)?;
        fade_out_all(stage, &edges, 0.4)?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 06_5: game B on its triangle: the bad coin's chances from corner
        // 0, the good coin's from 1 and 2
        let line = say(&kit, stage, "06_5")?;
        line.wait_for(stage, AT_WRITE.1)?;
        let b_title = title(&kit, stage, Which::B)?;
        fade_in_all(stage, &[b_title], 0.4)?;
        let b_labels = chance_labels(&kit, stage, Which::B, &n.b)?;
        let mut texts = shown.to_vec();
        texts.push(tag[1]);
        texts.extend(b_names);
        texts.push(b_title);
        texts.extend(b_labels);
        keep_apart(stage, &texts, 0.08, "game B's triangle")?;
        line.wait_for(stage, AT_BAD.1)?;
        flash_all(stage, &[b_discs[0]], 0.6)?;
        line.wait_for(stage, AT_BAD_PCT.1)?;
        // corner 0's two ways: forward along edge 0, back along edge 2
        fade_in_all(stage, &[b_labels[0], b_labels[5]], 0.4)?;
        line.wait_for(stage, AT_GOOD.1)?;
        flash_all(stage, &[b_discs[1], b_discs[2]], 0.6)?;
        line.wait_for(stage, AT_GOOD_PCT.1)?;
        fade_in_all(stage, &b_labels[1..5], 0.4)?;

        // 06_6: game A's triangle on the left, the same chances from every
        // corner; both stand side by side
        let line = say(&kit, stage, "06_6")?;
        line.wait_for(stage, AT_GAME_A.1)?;
        let a = triangle(&kit, stage, Which::A)?;
        let a_labels = chance_labels(&kit, stage, Which::A, &n.a)?;
        texts.extend(a.names);
        texts.push(a.title);
        texts.extend(a_labels);
        keep_apart(stage, &texts, 0.08, "the two triangles")?;
        fade_in_all(stage, &a.parts(), 0.6)?;
        line.wait_for(stage, AT_A_FORWARD.1)?;
        fade_in_all(stage, &a_labels[..3], 0.4)?;
        line.wait_for(stage, AT_A_BACK.1)?;
        fade_in_all(stage, &a_labels[3..], 0.4)?;
        let up = now(stage);
        kit.hold(stage)?;
        wait_until(stage, up + COMPARE_S)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use explainer_kit::{Narrator, Script};
    use fmn::core::constants::FRAME_Y_RADIUS;
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::super::draw::{BADGE_AT, THINK_S, TOKEN_R};
    use super::*;

    /// A label's box at 20 px, at most ("90.5 %", about 0.54 by the glyph
    /// calibration, taken 30 % wider), and a name's ("2"). Back labels sit
    /// further in the wider they are; past about 0.8 the two inner ones of a
    /// triangle would meet, which the headless run's `keep_apart` refuses.
    const LABEL_W: f64 = 0.7;
    const LABEL_H: f64 = 0.22;
    const NAME_W: f64 = 0.2;
    const NAME_H: f64 = 0.25;

    /// An axis-aligned box: (left, bottom, right, top).
    fn around(c: Vec3, w: f64, h: f64) -> [f64; 4] {
        [c[0] - w / 2.0, c[1] - h / 2.0, c[0] + w / 2.0, c[1] + h / 2.0]
    }

    fn apart(a: [f64; 4], b: [f64; 4], room: f64) -> bool {
        a[2] + room <= b[0] || b[2] + room <= a[0] || a[3] + room <= b[1] || b[3] + room <= a[1]
    }

    /// The chapter's words are true of its numbers: seven, eight, nine and
    /// eight dollars stand on corners 1, 2, 0 and 2; game B's bad coin wins
    /// 9.5 % from corner 0 and its good coin 74.5 % from 1 and 2; game A's
    /// coin wins 49.5 % and loses 50.5 % from every corner.
    #[test]
    fn three_corners_numbers_are_the_words() {
        let n = numbers().unwrap();
        assert_eq!(n.walk, [(7, 1), (8, 2), (9, 0), (8, 2)]);
        fn words(c: &[String; 3]) -> [&str; 3] {
            c.each_ref().map(String::as_str)
        }
        assert_eq!(words(&n.b.heads), ["9.5", "74.5", "74.5"]);
        assert_eq!(words(&n.b.tails), ["90.5", "25.5", "25.5"]);
        assert_eq!(words(&n.a.heads), ["49.5"; 3]);
        assert_eq!(words(&n.a.tails), ["50.5"; 3]);
        assert_eq!(remainder_text(7), "7 \u{2192} remainder 1");
        assert_eq!(remainder_text(9), "9 \u{2192} remainder 0");
        assert_eq!(n.badge, "published coins \u{b7} 49.5 \u{b7} 9.5 \u{b7} 74.5 %");
    }

    /// Each corner is a remainder. A count stands on its remainder's
    /// corner; a win is one corner forward and a loss one back, round the
    /// triangle both ways, so a win from corner 2 wraps to corner 0 (06_3's
    /// answer) and a loss from corner 0 wraps to corner 2. The arrows the
    /// walk uses are those: 1 → 2, 2 → 0, then back 0 → 2.
    #[test]
    fn the_corners_are_the_remainders() {
        for count in -30i64..=30 {
            let here = corner_of(count);
            assert!(here < 3);
            assert_eq!(corner_of(count + 1), (here + 1) % 3, "a win from {count}");
            assert_eq!(corner_of(count - 1), (here + 2) % 3, "a loss from {count}");
        }
        let n = numbers().unwrap();
        let used: Vec<(usize, bool)> = n.walk.windows(2).map(|p| edge_of(p[0].1, p[1].1)).collect();
        assert_eq!(used, [(1, true), (2, true), (2, false)]);
        // every arrow starts by the corner it leaves and ends by the one it
        // reaches: forward along edge e from e to e + 1, back from e + 1 to e
        let nearest = |p: Vec3| {
            (0..3)
                .min_by(|&i, &j| {
                    let d = |k: usize| (corner(B_AT, k)[0] - p[0]).hypot(corner(B_AT, k)[1] - p[1]);
                    d(i).total_cmp(&d(j))
                })
                .unwrap()
        };
        for e in 0..3 {
            let (s, t) = arrow_ends(B_AT, e, true);
            assert_eq!((nearest(s), nearest(t)), (e, (e + 1) % 3));
            let (s, t) = arrow_ends(B_AT, e, false);
            assert_eq!((nearest(s), nearest(t)), ((e + 1) % 3, e));
        }
        // corner 0 at the top, 1 lower right, 2 lower left; forward runs
        // clockwise
        let [c0, c1, c2] = [0, 1, 2].map(|i| corner(B_AT, i));
        assert!(c0[1] > c1[1] && (c1[1] - c2[1]).abs() < 1e-9 && c1[0] > c2[0]);
        let turn = (c1[0] - c0[0]) * (c2[1] - c1[1]) - (c1[1] - c0[1]) * (c2[0] - c1[0]);
        assert!(turn < 0.0, "0 → 1 → 2 turns clockwise");
    }

    /// Negative control: a count below zero. Rust's `%` keeps the sign, so
    /// −1 % 3 is −1, a corner that does not exist; the remainder game B
    /// reads (and K03 states) puts −1 on corner 2 and −3 on corner 0, as the
    /// facts print. A player in debt still stands on the triangle, and the
    /// walk goes on past zero dollars.
    #[test]
    fn a_count_below_zero_still_stands_on_a_corner() {
        let naive = |count: i64| count % MODULUS;
        assert_eq!(naive(-1), -1);
        assert_eq!((corner_of(-1), corner_of(-3)), (2, 0));
        let f = facts::facts().unwrap();
        let examples = f.get("remainder.examples").unwrap();
        assert_eq!(printed_remainder(examples, -1), Ok(2));
        assert_eq!(printed_remainder(examples, -3), Ok(0));
        let down: Vec<usize> = (-2..=2).rev().map(corner_of).collect();
        assert_eq!(down, [2, 1, 0, 2, 1]);
    }

    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in CUES {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The layout keeps its rooms, with each label at its widest: every
    /// label sits beyond its arrow's bow, forward ones outside the triangle
    /// and back ones inside, and no two labels, names or titles touch; the
    /// triangles stay inside the frame, below the badge and above a
    /// two-line lower third; the counter and its line keep left of game A's
    /// corner 0; 02's rows and the prediction keep clear of game B's
    /// triangle.
    #[test]
    fn the_layout_keeps_its_rooms() {
        let mut boxes: Vec<[f64; 4]> = Vec::new();
        for at in [A_AT, B_AT] {
            for e in 0..3 {
                for forward in [true, false] {
                    let c = chance_at(at, e, forward, LABEL_W, LABEL_H);
                    let (m, _, n) = edge(at, e);
                    // outside forward, inside back, by the label's reach
                    let out = (c[0] - m[0]) * n[0] + (c[1] - m[1]) * n[1];
                    let reach = LABEL_W / 2.0 * n[0].abs() + LABEL_H / 2.0 * n[1].abs();
                    if forward {
                        assert!(out > ARROW_OFF + BOW / 2.0 + reach, "edge {e}: {out}");
                    } else {
                        assert!(-out > ARROW_OFF + BOW / 2.0 + reach, "edge {e}: {out}");
                    }
                    boxes.push(around(c, LABEL_W, LABEL_H));
                }
            }
            for i in 0..3 {
                let c = name_at(at, i, NAME_W, NAME_H);
                let p = corner(at, i);
                assert!((c[0] - p[0]).hypot(c[1] - p[1]) > CORNER_R + NAME_GAP);
                boxes.push(around(c, NAME_W, NAME_H));
            }
            boxes.push(around(v(at[0], at[1] - TITLE_DY), 1.0, 0.25));
        }
        for (i, a) in boxes.iter().enumerate() {
            for b in &boxes[i + 1..] {
                assert!(apart(*a, *b, 0.08), "{a:?} touches {b:?}");
            }
            assert!(a[0] > -FRAME_X_RADIUS + 0.25 && a[2] < FRAME_X_RADIUS - 0.25, "{a:?}");
            // under the badge, above a two-line lower third (top at −2.78)
            assert!(
                a[3] < BADGE_AT[1] - 0.19 - 0.1 && a[1] > -FRAME_Y_RADIUS + 1.22 + 0.1,
                "{a:?}"
            );
        }
        // the counter and its line keep left of game A's corner 0
        const {
            assert!(HUD_RIGHT < A_AT[0] - CORNER_R && HUD_COUNT_AT[0] > -FRAME_X_RADIUS + 0.25);
        }
        const {
            assert!(HUD_REM_AT[0] + 2.0 < HUD_RIGHT, "a line \"9 → remainder 0\" fits");
        }
        // 02's rows, their label and the prediction (its ring and its note
        // "pause for more time", about 2 wide at 18 px) keep left of B's
        // triangle and inside the frame, and the ring clears the counter
        let b_left = corner(B_AT, 2)[0] - CORNER_R;
        assert!(GRID_X0 + 2.5 * PITCH + LABEL_GAP + 1.2 < b_left - ROOM);
        assert!(THINK_AT[0] + 1.0 + ROOM < b_left && THINK_AT[0] - 1.0 > -FRAME_X_RADIUS + 0.25);
        const {
            assert!(THINK_AT[1] + 0.62 < HUD_REM_AT[1] - 0.2);
        }
        // the token fits inside a corner's disc
        const {
            assert!(TOKEN_R < CORNER_R - 0.08);
        }
    }

    /// The chapter runs headless to its end with the scratch voice and real
    /// typesetting (so every `keep_between` and `keep_apart` guard meets the
    /// true text widths): every line of chapter 06 is said once, in order;
    /// the prediction (06_3) gets its silent think time; the six arrows get
    /// their beat before 06_5; the two triangles stand after the last line.
    #[test]
    fn the_chapter_runs_and_leaves_room_to_think() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = ThreeCorners { kit: kit.clone() };
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut NullSceneSink).unwrap_or_else(|e| panic!("06_three_corners: {e}"));
        let said = narrator.said();
        let ids: Vec<&str> = said.iter().map(|(id, _)| id.as_str()).collect();
        let expected: Vec<String> = (1..=6).map(|k| format!("06_{k}")).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        let at = |id: &str| said.iter().find(|(l, _)| l == id).unwrap().1;
        let end_of = |id: &str| at(id) + narrator.length(id).unwrap();
        assert!(
            at("06_4") - end_of("06_3") >= THINK_S,
            "{} s to think",
            at("06_4") - end_of("06_3")
        );
        assert!(at("06_5") - end_of("06_4") >= BEAT, "the six arrows have no beat");
        let end = kit.finished.borrow().as_ref().expect("a timeline").seconds();
        assert!(end - end_of("06_6") >= 1.0, "the triangles do not stand: {end}");
    }
}
