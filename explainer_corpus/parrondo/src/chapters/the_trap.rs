//! Chapter 10, the trap: why corner zero collects players. The picture 09
//! left: B's triangle with the round-number coins' chances and the badge,
//! and the 26 dots at 10, 4 and 12, still split by the chances. Why does
//! corner 0 collect extra players? (corner 0 ringed with its dots) The
//! crowd dims and the same player, as 08_1 left them, stands out on corner
//! 0: the teal token with 9 dollars, "9 → remainder 0". Where is that player
//! most likely to be one round later? (10_1, a prediction.) At corner 2,
//! nine times in ten: the token takes the back arrow 0 → 2 leaving a red
//! trace, 8 dollars; from corner 2 the good coin usually wins, and the
//! forward arrow 2 → 0 carries them around to corner 0, a green trace, 9
//! dollars again (10_2). That was one player's likeliest two rounds, labelled
//! as an illustration, not the only way they can go; the whole crowd comes
//! back, settled at 10, 4 and 12 (10_3). In that settled crowd the good coin
//! wins for 75 % of 4 = 3 players at corner 1 and 75 % of 12 = 9 at corner
//! 2, so 9 of its 3 + 9 wins, 3/4, happen at corner 2, and every one of them
//! is the 2 → 0 arrow back to the bad coin (10_4). That is the trap: 0 → 2
//! → 0 (10_5, held). One change: a die picks B's coin instead of the
//! dollars, faces 1 and 2 the bad coin, 3 to 6 the good, the same two
//! coins; the triangle dims under a veil, its corners no longer choosing
//! (10_6). The bad coin gets 1/3 of the rounds wherever the player stands
//! and the die game wins 1/3 · 10 % + 2/3 · 75 % ≈ 53.3 % (10_7, held). Same
//! two coins, only the rule that picks between them changed; the die goes,
//! the triangle returns, and reading the money takes B from 53.3 % to 50 %
//! (10_8).
//!
//! The illustration is not the crowd's claim, and the scene keeps them
//! apart: the token's two moves are the likeliest of the four two-round
//! paths from corner 0 (checked, and not certain), labelled as one player's;
//! the 3/4 is the settled split crowd's good-coin wins, under the panel's
//! "split by the chances (expected numbers)". Every count, chance, flow and
//! percentage comes from `crate::model` and is checked against
//! `crate::facts`; the scene refuses to draw if the words would stop being
//! true of them. No typeset text is indicated, scaled or recoloured: dots,
//! traces, die faces and rings pulse (a corner or a coin by a separate
//! hollow ring, never its filled disc), words get an outline, a changed line
//! is replaced. The token and its trace read one tracker, so the trace's tip
//! is the token in every frame.

use explainer_kit::{BACKGROUND, Kit, Palette, Timed, fade_all, hex, rect, tween, v};
use fmn::core::constants::FRAME_X_RADIUS;
use fmn::prelude::*;

use super::draw::{
    arrow, coin, fade_in_all, fade_out_all, fade_to, fail, flash_all, flat, keep_between, line_style, now, pause, say,
    stand, think, token,
};
use super::looks_hopeless::{EDGE, POINT_PAD, ROOM, frame_around, point_at};
use super::one_player_to_a_crowd::{
    CORNER_WORD, PANEL_X, PLAYERS_NUM_X, PLAYERS_Y, dot, panel_head, places, ring_corners, round_badge,
    round_badge_text,
};
use super::three_corners::{
    B_AT, CORNER_R, Chances, Which, apex, arrow_ends, chance_labels, chances, corner, corner_of, counter,
    counter_number, edge_of, keep_apart, printed_remainder, remainder_line, swap, triangle, walk,
};
use crate::facts;
use crate::model::{self, Game, MODULUS, ROUND, Ratio};
use crate::palette;

pub struct TheTrap {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const QUESTION: &str = "one round later: which corner?";
const PATH_LABEL: &str = "illustration: one player's likeliest two rounds";
const WINS_HEAD: &str = "the good coin's wins each round:";
const SHARE_HEAD: &str = "the share of them at corner 2:";
const TRAP: &str = "the trap:";
const RULE_B: &str = "game B: your dollars pick the coin";
const RULE_DIE: &str = "die game: a die picks the coin";
const BAD_FACE: &str = "bad";
const GOOD_FACE: &str = "good";
const OF_ROUNDS: &str = "of rounds";
const DIE_WINS: &str = "the die game wins";
const READING: &str = "reading your money:";
const PERCENT: &str = "%";

/// Where each picture lands: (line, the phrase it waits for).
const AT_COLLECT: (&str, &str) = ("10_1", "Why does corner zero collect extra players");
const AT_FOLLOW: (&str, &str) = ("10_1", "Follow one who has just arrived there");
const AT_WHERE: (&str, &str) = ("10_1", "Where is that player most likely");
const AT_AT_TWO: (&str, &str) = ("10_2", "At corner two");
const AT_NINE_TEN: (&str, &str) = ("10_2", "nine times in ten");
const AT_KNOCKS: (&str, &str) = ("10_2", "The bad coin knocks them back");
const AT_USUALLY: (&str, &str) = ("10_2", "the good coin usually wins");
const AT_AROUND: (&str, &str) = ("10_2", "around to zero again");
const AT_LIKELIEST: (&str, &str) = ("10_3", "one player's likeliest two rounds");
const AT_WHOLE: (&str, &str) = ("10_3", "go back to the whole crowd");
const AT_SETTLED: (&str, &str) = ("10_3", "settled at ten, four and twelve");
const AT_THREE: (&str, &str) = ("10_4", "three players at corner one");
const AT_NINE: (&str, &str) = ("10_4", "nine at corner two");
const AT_NINE_OF: (&str, &str) = ("10_4", "nine happen at corner two");
const AT_BACK_ZERO: (&str, &str) = ("10_4", "brings a player back to zero");
const AT_WAITS: (&str, &str) = ("10_4", "where the bad coin waits");
const AT_TRAP: (&str, &str) = ("10_5", "the trap in game B");
const AT_CLIMBING: (&str, &str) = ("10_5", "climbing back to a multiple of three");
const AT_ONE_CHANGE: (&str, &str) = ("10_6", "One change removes the trap");
const AT_DIE: (&str, &str) = ("10_6", "Let a die choose B's coin");
const AT_INSTEAD: (&str, &str) = ("10_6", "instead of your money");
const AT_ONE_TWO: (&str, &str) = ("10_6", "A one or a two means the bad coin");
const AT_ELSE: (&str, &str) = ("10_6", "anything else the good coin");
const AT_IN_THREE: (&str, &str) = ("10_7", "one round in three");
const AT_WHEREVER: (&str, &str) = ("10_7", "wherever you stand");
const AT_DIE_WINS: (&str, &str) = ("10_7", "this game wins");
const AT_SAME: (&str, &str) = ("10_8", "Same two coins");
const AT_RULE: (&str, &str) = ("10_8", "Only the rule that picks between them changed");
const AT_READING: (&str, &str) = ("10_8", "Reading your money");
const AT_HALF: (&str, &str) = ("10_8", "down to exactly half");
#[cfg(test)]
const CUES: [(&str, &str); 30] = [
    AT_COLLECT,
    AT_FOLLOW,
    AT_WHERE,
    AT_AT_TWO,
    AT_NINE_TEN,
    AT_KNOCKS,
    AT_USUALLY,
    AT_AROUND,
    AT_LIKELIEST,
    AT_WHOLE,
    AT_SETTLED,
    AT_THREE,
    AT_NINE,
    AT_NINE_OF,
    AT_BACK_ZERO,
    AT_WAITS,
    AT_TRAP,
    AT_CLIMBING,
    AT_ONE_CHANGE,
    AT_DIE,
    AT_INSTEAD,
    AT_ONE_TWO,
    AT_ELSE,
    AT_IN_THREE,
    AT_WHEREVER,
    AT_DIE_WINS,
    AT_SAME,
    AT_RULE,
    AT_READING,
    AT_HALF,
];

// ------------------------------------------------------------------ layout

/// The left panel: the question, then the path's label; the good coin's
/// wins (a head, a line per corner, the share's head and its fraction); the
/// trap's name.
const QUESTION_Y: f64 = 2.15;
const THINK_AT: Vec3 = [-3.9, 0.55, 0.0];
const PATH_Y: f64 = 2.0;
const WINS_YS: [f64; 3] = [1.45, 1.0, 0.55];
const SHARE_HEAD_Y: f64 = 0.05;
const SHARE_TEX_Y: f64 = -0.55;
const TRAP_Y: f64 = -1.3;
/// The panel's text keeps left of this (B's corner 2, its name and arrows).
const PANEL_RIGHT: f64 = -0.7;
/// A corner's count beside it: right of corners 0 and 1, left of corner 2,
/// raised this much at the two lower corners (their names sit below them).
const COUNT_DY: [f64; 3] = [0.0, 0.35, 0.35];
/// The die section: the two rules at the top, then a row of die faces per
/// coin, an arrow to the coin, its chance and its share of the rounds; the
/// sum, the verdict and B's own result below.
const RULE_YS: [f64; 2] = [2.75, 2.3];
const ROW_YS: [f64; 2] = [1.45, 0.6];
const FACE: f64 = 0.42;
const FACE_PITCH: f64 = 0.52;
const FACE_X0: f64 = -6.5;
const PIP_R: f64 = 0.04;
const PIP_OFF: f64 = 0.12;
const ARROW_GAP: f64 = 0.12;
const COIN_X: f64 = -3.4;
const COIN_R: f64 = 0.32;
const CHANCE_X: f64 = COIN_X + COIN_R + 0.15;
const SHARE_X: f64 = -2.10;
const DIE_TEX_Y: f64 = -0.3;
const DIE_WINS_Y: f64 = -0.9;
const FROM_TO_Y: f64 = -1.5;
/// The veil that dims B's triangle while the die picks the coin: a
/// background-coloured sheet over the triangle (below the badge, right of
/// the panel), so the arrows' strokes are never refilled by an opacity
/// change and nothing needs restoring.
const VEIL_LEFT: f64 = -0.6;
const VEIL_BOTTOM: f64 = -2.75;
const VEIL_TOP: f64 = 3.25;
const VEIL_O: f64 = 0.72;
/// A trace of the player's path over its arrow, and the crowd's stream.
const TRACE_W: f64 = 7.0;
const TRACE_O: f64 = 0.75;
const TRACE_DIM: f64 = 0.25;
const STREAM_MIN_W: f64 = 1.5;
const STREAM_MAX_W: f64 = 16.0;
const STREAM_O: f64 = 0.7;
/// The crowd while one player is followed.
const CROWD_DIM: f64 = 0.25;
/// Seconds: one step of the token, the beat after its two rounds, the hold
/// on the trap, the hold on the die game's verdict.
const STEP_S: f64 = 1.6;
const BEAT: f64 = 1.0;
const TRAP_HOLD: f64 = 1.5;
const VERDICT_HOLD: f64 = 1.5;

// ---------------------------------------------------------------- numbers

/// The chapter's numbers, checked against the words they illustrate.
struct Numbers {
    round_badge: String,
    b_round: Chances,
    n: usize,
    counts: [usize; 3],
    forward: [usize; 3],
    /// The player's dollars over the likeliest two rounds, and their corners.
    dollars: [i64; 3],
    path: [usize; 3],
    wins_lines: [String; 2],
    share_tex: String,
    trap: String,
    /// The die's faces for the bad coin and for the good one.
    faces: [Vec<u8>; 2],
    chances: [String; 2],
    shares: [String; 2],
    die_tex: String,
    from_to: String,
}

fn csv<T: ToString>(values: impl IntoIterator<Item = T>) -> String {
    values.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
}

fn whole(x: Ratio, what: &str) -> Result<usize, String> {
    if !x.is_integer() {
        return Err(format!("{what} is not a whole number of players"));
    }
    usize::try_from(x.num()).map_err(|e| format!("{what}: {e}"))
}

/// The chance of one step of `g` from corner `from` to corner `to` (one
/// forward on a win, one back on a loss).
fn step_chance(g: &Game, from: usize, to: usize) -> Ratio {
    if to == (from + 1) % 3 {
        g.chance(from)
    } else {
        Ratio::ONE - g.chance(from)
    }
}

/// The four two-round paths of `g` from corner `start`, each with its
/// chance, in the order forward-forward, forward-back, back-forward,
/// back-back.
fn two_rounds(g: &Game, start: usize) -> Vec<([usize; 3], Ratio)> {
    let mut out = Vec::with_capacity(4);
    for first in [1, 2] {
        for second in [1, 2] {
            let c1 = (start + first) % 3;
            let c2 = (c1 + second) % 3;
            out.push(([start, c1, c2], step_chance(g, start, c1) * step_chance(g, c1, c2)));
        }
    }
    out
}

/// Of the good coin's wins in a crowd settled under `g` (corners 1 and 2),
/// the share at corner 2.
fn good_wins_at2(g: &Game) -> Ratio {
    let w = g.shares();
    let (at1, at2) = (w[1] * g.chance(1), w[2] * g.chance(2));
    at2 / (at1 + at2)
}

fn numbers() -> Result<Numbers, String> {
    let f = facts::facts()?;
    let get = |k: &str| f.get(k).map(str::to_owned).ok_or_else(|| format!("no fact {k}"));
    let g = ROUND.game_b();
    let n = model::smallest_whole_crowd(&g);
    if get("crowd.size")? != n.to_string() || n != 26 {
        return Err("the crowd is twenty-six".into());
    }
    let counts_r = g.shares().map(|w| Ratio::int(n) * w);
    let (forward_r, _) = model::flows(&g, n);
    if get("crowd.counts")? != csv(counts_r) || get("crowd.forward")? != csv(forward_r) {
        return Err("the crowd's counts or flows disagree with the facts".into());
    }
    let [c0, c1, c2] = counts_r.map(|x| whole(x, "a corner's crowd"));
    let counts = [c0?, c1?, c2?];
    let [f0, f1, f2] = forward_r.map(|x| whole(x, "a forward flow"));
    let forward = [f0?, f1?, f2?];
    // 10_3 and 10_4: settled at ten, four and twelve; three and nine win
    // with the good coin
    if counts != [10, 4, 12] || forward != [1, 3, 9] {
        return Err("10_3 and 10_4 say ten, four and twelve, and three and nine".into());
    }
    // 10_1: the player who has just arrived at corner 0: 08_1's one more win
    // from 06's 8 dollars, 9 dollars, nothing left over
    let here = walk()[3] + 1;
    let dollars = [here, here - 1, here];
    let examples = get("remainder.examples")?;
    for &d in &dollars {
        if printed_remainder(&examples, d)? != d.rem_euclid(MODULUS) {
            return Err(format!("the remainder of {d} disagrees with the facts"));
        }
    }
    let path = dollars.map(corner_of);
    if path != [0, 2, 0] {
        return Err("10_2 says corner 0, then corner 2, then corner 0 again".into());
    }
    // 10_2: from corner 0, back to corner 2 nine times in ten; from corner
    // 2, the good coin usually wins and goes forward, around to corner 0
    if step_chance(&g, 0, 2) != Ratio::new(9, 10) || step_chance(&g, 0, 2) <= step_chance(&g, 0, 1) {
        return Err("10_2 says corner 2, nine times in ten".into());
    }
    // the facts list the coins' tails as A's, the bad coin's, the good one's
    let bad_tails = (Ratio::ONE - g.chance(0)).exact_pct()?;
    if get("coins.round.tails.pct")?.split(',').nth(1) != Some(bad_tails.as_str()) {
        return Err("corner 0's ninety percent disagrees with the facts".into());
    }
    if step_chance(&g, 2, 0) <= Ratio::new(1, 2) || edge_of(2, 0) != (2, true) || edge_of(0, 2) != (2, false) {
        return Err("10_2 says the good coin usually carries corner 2 forward to corner 0".into());
    }
    // 10_3: the likeliest two rounds from corner 0, not the only way
    let paths = two_rounds(&g, 0);
    let total = paths.iter().fold(Ratio::ZERO, |a, (_, p)| a + *p);
    let (best, chance) = paths
        .iter()
        .copied()
        .max_by_key(|&(_, p)| p)
        .ok_or("no two-round paths")?;
    if best != path || total != Ratio::ONE || chance >= Ratio::ONE {
        return Err("10_3 says the path shown is the likeliest two rounds, not the only way".into());
    }
    // 10_4: in the settled crowd the good coin wins for 3 players at corner
    // 1 and 9 at corner 2; of its twelve wins a round, nine are at corner 2
    let good = g.chance(1);
    if g.chance(2) != good || good <= Ratio::new(1, 2) {
        return Err("the good coin, at corners 1 and 2, wins more often than not".into());
    }
    for i in [1, 2] {
        if Ratio::int(counts[i] as i64) * good != Ratio::int(forward[i] as i64) {
            return Err(format!(
                "corner {i}'s good-coin wins are not its count times the chance"
            ));
        }
    }
    let at2 = Ratio::new(forward[2] as i128, (forward[1] + forward[2]) as i128);
    if forward[1] + forward[2] != 12
        || forward[2] != 9
        || get("crowd.good_wins_at2")? != at2.to_string()
        || at2 != good_wins_at2(&g)
    {
        return Err("10_4 says nine of the good coin's twelve wins a round".into());
    }
    let good_pct = good.exact_pct()?;
    let wins_lines = [1, 2].map(|i| {
        format!(
            "{CORNER_WORD} {i}: {good_pct} {PERCENT} of {} = {}",
            counts[i], forward[i]
        )
    });
    let share_tex = format!(
        r"\tfrac{{{}}}{{{} + {}}} = \tfrac{{{}}}{{{}}}",
        forward[2],
        forward[1],
        forward[2],
        at2.num(),
        at2.den()
    );
    // 10_6 and 10_7: a die picks the coin: a one or a two the bad coin, one
    // round in three wherever the player stands; the same two coins
    let bad_faces: Vec<u8> = (1..=6).filter(|&k| k <= 2).collect();
    let good_faces: Vec<u8> = (1..=6).filter(|&k| k > 2).collect();
    let bad_share = Ratio::new(bad_faces.len() as i128, 6);
    let third = Ratio::new(1, 3);
    if bad_share != third || get("thirds.pct")? != third.pct(1) {
        return Err("10_7 says one round in three".into());
    }
    let [_, bad_coin, good_coin] = facts::coins(&ROUND);
    if bad_coin != g.chance(0) || good_coin != good {
        return Err("the die game tosses game B's own two coins".into());
    }
    let die = bad_share * bad_coin + (Ratio::ONE - bad_share) * good_coin;
    if die != ROUND.die_chosen_win()
        || get("round.die.win")? != die.to_string()
        || die != facts::thirds_win(&ROUND)
        || die.pct(1) != get("round.b.thirds.win.pct")?
    {
        return Err("10_7 says the die game wins fifty-three point three percent".into());
    }
    // 10_8: reading the money takes B from that down to exactly half
    if g.win() != Ratio::new(1, 2) || get("round.b.win")? != g.win().to_string() || die <= g.win() {
        return Err("10_8 says reading your money drags B down to exactly half".into());
    }
    let equals = if die.exact_pct().is_ok() { "=" } else { r"\approx" };
    let good_share = Ratio::ONE - bad_share;
    let (bad_pct, good_coin_pct) = (bad_coin.exact_pct()?, good_coin.exact_pct()?);
    let die_tex = format!(
        r"\tfrac{{{}}}{{{}}} \cdot {bad_pct}\% + \tfrac{{{}}}{{{}}} \cdot {good_coin_pct}\% {equals} {}\%",
        bad_share.num(),
        bad_share.den(),
        good_share.num(),
        good_share.den(),
        die.pct(1)
    );
    Ok(Numbers {
        round_badge: round_badge_text(&f)?,
        b_round: chances(&g)?,
        n: usize::try_from(n).map_err(|e| format!("the crowd: {e}"))?,
        counts,
        forward,
        dollars,
        path,
        wins_lines,
        share_tex,
        trap: format!("{TRAP} {} \u{2192} {} \u{2192} {}", path[0], path[1], path[2]),
        faces: [bad_faces, good_faces],
        chances: [format!("{bad_pct} {PERCENT}"), format!("{good_coin_pct} {PERCENT}")],
        shares: [
            format!("{}/{} {OF_ROUNDS}", bad_share.num(), bad_share.den()),
            format!("{}/{} {OF_ROUNDS}", good_share.num(), good_share.den()),
        ],
        die_tex,
        from_to: format!(
            "{READING} {} {PERCENT} \u{2192} {} {PERCENT}",
            die.pct(1),
            g.win().exact_pct()?
        ),
    })
}

// --------------------------------------------------------------- pictures

/// The shaft of the arrow along edge `e` of B's triangle as `curved_arrow`
/// draws it: the quadratic from the arrow's start to its end, its middle
/// at the bow's apex, 33 points to 0.94 of the way (the head covers the
/// rest).
pub(super) fn shaft(e: usize, forward: bool) -> Vec<Vec3> {
    let (from, to) = arrow_ends(B_AT, e, forward);
    let top = apex(B_AT, e, forward);
    let control = [
        2.0 * top[0] - (from[0] + to[0]) / 2.0,
        2.0 * top[1] - (from[1] + to[1]) / 2.0,
    ];
    (0..=32)
        .map(|k| {
            let t = f64::from(k) / 32.0 * 0.94;
            let (a, b, c) = ((1.0 - t) * (1.0 - t), 2.0 * (1.0 - t) * t, t * t);
            v(
                a * from[0] + b * control[0] + c * to[0],
                a * from[1] + b * control[1] + c * to[1],
            )
        })
        .collect()
}

/// The way a step from corner `from` to corner `to` goes: from the corner's
/// centre onto the arrow the step uses, along its shaft to its tip, and
/// into the other corner's centre. The shaft and tip are points 1 to
/// `len - 2`.
fn route(from: usize, to: usize) -> Vec<Vec3> {
    let (e, forward) = edge_of(from, to);
    let mut out = vec![corner(B_AT, from)];
    out.extend(shaft(e, forward));
    out.push(arrow_ends(B_AT, e, forward).1);
    out.push(corner(B_AT, to));
    out
}

/// The arc length along `points` to each of them, from the first.
fn lengths(points: &[Vec3]) -> Vec<f64> {
    let mut out = Vec::with_capacity(points.len());
    let mut d = 0.0;
    out.push(d);
    for pair in points.windows(2) {
        d += (pair[1][0] - pair[0][0]).hypot(pair[1][1] - pair[0][1]);
        out.push(d);
    }
    out
}

/// The point at arc length `d` along `points` (`cum` their lengths), and the
/// segment it lies on.
fn at_length(points: &[Vec3], cum: &[f64], d: f64) -> (Vec3, usize) {
    let end = cum.last().copied().unwrap_or(0.0);
    let d = d.clamp(0.0, end);
    let k = cum
        .partition_point(|&c| c <= d)
        .saturating_sub(1)
        .min(points.len().saturating_sub(2));
    let span = (cum[k + 1] - cum[k]).max(1e-12);
    let f = ((d - cum[k]) / span).clamp(0.0, 1.0);
    let (a, b) = (points[k], points[k + 1]);
    (v(a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f), k)
}

/// The trace of a route walked to arc length `d`: the shaft and tip points
/// passed so far and the walker's place, at least two points.
fn trace_to(points: &[Vec3], cum: &[f64], d: f64) -> Vec<Vec3> {
    let (here, k) = at_length(points, cum, d);
    let (first, tip) = (1, points.len() - 2);
    let mut out: Vec<Vec3> = Vec::with_capacity(points.len());
    if k >= first {
        out.extend(&points[first..=k.min(tip)]);
        if k < tip {
            out.push(here);
        }
    }
    if out.len() < 2 {
        out = vec![points[first], points[first]];
    }
    out
}

fn trace_style(color: Srgb, o: f64) -> Style {
    Style::default().fill(color, 0.0).stroke(color, TRACE_W, o)
}

/// The token walks one step from corner `from` to corner `to` along its
/// arrow in `rt` seconds while `with` plays, drawing a `color` trace over the
/// arrow up to where the token is: one tracker drives both. Returns the
/// trace.
fn travel(
    stage: &mut Stage<'_>,
    player: Mob,
    from: usize,
    to: usize,
    color: Srgb,
    rt: f64,
    with: Vec<Box<dyn Animation>>,
) -> fmn::Result<Mob> {
    let points = route(from, to);
    let cum = lengths(&points);
    let total = cum.last().copied().unwrap_or(0.0);
    let start = trace_to(&points, &cum, 0.0);
    let trace = stage
        .arena_mut()
        .add(polyline(start).style(trace_style(color, TRACE_O)).build());
    stage.add_to_scene(trace)?;
    // the token above its trace
    stage.scene_mut().remove(&[player]);
    stage.add_to_scene(player)?;
    let along = stage.add_value_tracker(0.0);
    let anchor = stage.arena_mut().add(VMobject::new());
    let mut last = None;
    stage.add_updater(
        anchor,
        move |s, _| {
            let f = s.tracker_value(along).unwrap_or(0.0).clamp(0.0, 1.0);
            if last == Some(f) {
                return;
            }
            let (here, _) = at_length(&points, &cum, f * total);
            s.move_to(player, here, ORIGIN);
            let _ = s.set_points(trace, polyline(trace_to(&points, &cum, f * total)).build().points());
            last = Some(f);
        },
        true,
    )?;
    stage.add_to_scene(anchor)?;
    let mut anims = with;
    anims.push(stage.prepare(tween(along, 1.0, rt)?)?);
    stage.play_prepared(anims)?;
    stage.clear_updaters(anchor, true);
    stage.scene_mut().remove(&[anchor]);
    Ok(trace)
}

/// Strokes brought to opacity `o` together, `rt` seconds (their fill stays
/// empty: an opacity change would fill an open curve).
fn stroke_to(stage: &mut Stage<'_>, mobs: &[Mob], o: f64, rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::with_capacity(mobs.len());
    for &m in mobs {
        let b = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .set_stroke(None, None, Some(o))?;
        anims.push(stage.prepare(b)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Where corner `j`'s count stands: (its anchor, whether that is its right
/// edge).
fn count_at(j: usize) -> (Vec3, bool) {
    let c = corner(B_AT, j);
    let reach = CORNER_R + ROOM;
    if j == 2 {
        (v(c[0] - reach, c[1] + COUNT_DY[j]), true)
    } else {
        (v(c[0] + reach, c[1] + COUNT_DY[j]), false)
    }
}

/// The pips of a die face showing `k`, as offsets from its centre in units
/// of [`PIP_OFF`].
fn pips(k: u8) -> &'static [(f64, f64)] {
    match k {
        1 => &[(0.0, 0.0)],
        2 => &[(-1.0, 1.0), (1.0, -1.0)],
        3 => &[(-1.0, 1.0), (0.0, 0.0), (1.0, -1.0)],
        4 => &[(-1.0, 1.0), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)],
        5 => &[(-1.0, 1.0), (1.0, 1.0), (0.0, 0.0), (-1.0, -1.0), (1.0, -1.0)],
        _ => &[
            (-1.0, 1.0),
            (1.0, 1.0),
            (-1.0, 0.0),
            (1.0, 0.0),
            (-1.0, -1.0),
            (1.0, -1.0),
        ],
    }
}

/// A die face showing `k` centred at `at`: its outline and its pips.
/// Detached.
fn face(stage: &mut Stage<'_>, k: u8, at: Vec3) -> fmn::Result<Vec<Mob>> {
    let edge = rect(
        v(at[0] - FACE / 2.0, at[1] - FACE / 2.0),
        FACE,
        FACE,
        line_style(Palette::ink(), 2.0),
    )?;
    let mut out = vec![stage.arena_mut().add(edge)];
    for &(dx, dy) in pips(k) {
        out.push(
            stage.arena_mut().add(
                Circle::new()
                    .radius(PIP_R)
                    .arc_center(v(at[0] + dx * PIP_OFF, at[1] + dy * PIP_OFF))
                    .style(flat(Palette::ink(), 1.0))
                    .build(),
            ),
        );
    }
    Ok(out)
}

/// Point at shapes with separate hollow rings of radius `r` about `centres`
/// whose strokes pulse, while `with` flash, all together: a coin or a corner
/// is never flashed itself (its filled disc would drown what is on it).
fn rings_at(stage: &mut Stage<'_>, centres: &[Vec3], r: f64, with: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut rings = Vec::with_capacity(centres.len());
    let mut anims: Vec<Box<dyn Animation>> = Vec::with_capacity(centres.len() + with.len());
    for &c in centres {
        let ring = Circle::new()
            .radius(r)
            .arc_center(c)
            .style(
                Style::default()
                    .fill(Palette::hero(), 0.0)
                    .stroke(Palette::hero(), 3.0, 0.0),
            )
            .build();
        let ring = stage.arena_mut().add(ring);
        stage.add_to_scene(ring)?;
        let pulse = ring
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                rate_func: Some(fmn::core::rate::there_and_back),
                ..AnimateArgs::default()
            })?
            .set_stroke(None, None, Some(1.0))?;
        anims.push(stage.prepare(pulse)?);
        rings.push(ring);
    }
    for &m in with {
        let a = fmn::animation::indicate(stage.arena_mut(), m, 1.25, None)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    stage.scene_mut().remove(&rings);
    Ok(())
}

/// Refuse a veil that would leave any of `mobs` (B's triangle, its labels
/// and the crowd) showing past its edges, or that would reach the badge
/// (whose bottom edge is `badge_bottom`).
fn under_veil(stage: &Stage<'_>, mobs: &[Mob], badge_bottom: f64) -> fmn::Result<()> {
    if VEIL_TOP >= badge_bottom {
        return Err(fail(format!(
            "the veil's top {VEIL_TOP} reaches the badge at {badge_bottom}"
        )));
    }
    for &m in mobs {
        let (l, r) = (
            stage.get_bounding_box_point(m, LEFT)[0],
            stage.get_bounding_box_point(m, RIGHT)[0],
        );
        let (b, t) = (
            stage.get_bounding_box_point(m, DOWN)[1],
            stage.get_bounding_box_point(m, UP)[1],
        );
        if l < VEIL_LEFT || r > FRAME_X_RADIUS || b < VEIL_BOTTOM || t > VEIL_TOP {
            return Err(fail(format!(
                "the veil leaves a part of B's picture showing: x {l:.3} to {r:.3}, y {b:.3} to {t:.3}"
            )));
        }
    }
    Ok(())
}

/// A line of the panel, its left edge at [`PANEL_X`], kept left of
/// [`PANEL_RIGHT`]. Detached.
fn panel_line(kit: &Kit, stage: &mut Stage<'_>, words: &str, size: f64, color: Srgb, y: f64) -> fmn::Result<Mob> {
    let m = kit.text_left(stage, words, size, color, v(PANEL_X, y))?;
    keep_between(stage, m, PANEL_X - 0.01, PANEL_RIGHT, words)?;
    Ok(m)
}

impl SceneConstruct for TheTrap {
    fn name(&self) -> &str {
        "10_the_trap"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let n = numbers().map_err(fail)?;
        let size = n.n;

        // 10_1: 09's crowd at 10, 4 and 12; why does corner 0 collect extra
        // players? The crowd dims and the same player, 9 dollars, stands
        // out on corner 0; where one round later? (a prediction)
        let line = say(&kit, stage, "10_1")?;
        let b = triangle(&kit, stage, Which::B)?;
        let labels = chance_labels(&kit, stage, Which::B, &n.b_round)?;
        let tag = round_badge(&kit, stage, &n.round_badge)?;
        let head_words = panel_head(&kit, stage)?;
        let total = kit.text_left(
            stage,
            &size.to_string(),
            22.0,
            Palette::ink(),
            v(PLAYERS_NUM_X, PLAYERS_Y),
        )?;
        let head = [head_words[0], head_words[1], total];
        let mut corner_of_player = Vec::with_capacity(size);
        for (i, &c) in n.counts.iter().enumerate() {
            corner_of_player.extend(std::iter::repeat_n(i, c));
        }
        // each corner's players listed with its forward movers first
        let at_corners = places(&corner_of_player, |i| corner(B_AT, i));
        let dots: Vec<Mob> = at_corners.iter().map(|&p| dot(stage, p)).collect();
        let mut texts: Vec<Mob> = labels.to_vec();
        texts.extend(b.names);
        texts.extend([b.title, tag[1], head[0], head[1], total]);
        keep_apart(stage, &texts, 0.08, "09's picture")?;
        let mut arriving = b.parts();
        arriving.extend(labels);
        arriving.extend(tag);
        arriving.extend(head);
        arriving.extend(&dots);
        fade_in_all(stage, &arriving, 0.6)?;
        let first = |i: usize| n.counts[..i].iter().sum::<usize>();
        let at_corner = |i: usize| &dots[first(i)..first(i) + n.counts[i]];
        line.wait_for(stage, AT_COLLECT.1)?;
        ring_corners(stage, &[b.discs[0]], at_corner(0), 0.7)?;
        line.wait_for(stage, AT_FOLLOW.1)?;
        let mut dimming = Vec::with_capacity(size);
        for &d in &dots {
            dimming.push(fade_to(d, CROWD_DIM, 0.6)?);
        }
        let mut anims: Vec<Box<dyn Animation>> = Vec::with_capacity(size + 2);
        for b2 in dimming {
            anims.push(stage.prepare(b2)?);
        }
        anims.extend(swap(stage, &head, &[], 0.6)?);
        stage.play_prepared(anims)?;
        let player = token(stage, corner(B_AT, n.path[0]));
        let hud = counter(&kit, stage, n.dollars[0])?;
        let rem = remainder_line(&kit, stage, n.dollars[0])?;
        fade_in_all(stage, &[player, hud[0], hud[1], rem], 0.5)?;
        rings_at(stage, &[corner(B_AT, n.path[0])], CORNER_R + 0.05, &[player], 0.7)?;
        line.wait_for(stage, AT_WHERE.1)?;
        let question = panel_line(&kit, stage, QUESTION, 22.0, Palette::ink(), QUESTION_Y)?;
        fade_in_all(stage, &[question], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 10_2: back to corner 2 nine times in ten (a red trace, 8 dollars);
        // then the good coin usually carries them forward, around to 0 (a
        // green trace, 9 dollars)
        let line = say(&kit, stage, "10_2")?;
        line.wait_for(stage, AT_AT_TWO.1)?;
        let eight = counter_number(&kit, stage, n.dollars[1])?;
        let rem_eight = remainder_line(&kit, stage, n.dollars[1])?;
        fade_out_all(stage, &[hud[0], rem], 0.15)?;
        let with = swap(stage, &[q, question], &[], 0.4)?;
        let lost = travel(stage, player, n.path[0], n.path[1], palette::bad(), STEP_S, with)?;
        fade_in_all(stage, &[eight, rem_eight], 0.15)?;
        // corner 0's loss is the back arrow along edge 2; corner 2's win the
        // forward arrow along it
        let (back_e, _) = edge_of(n.path[0], n.path[1]);
        let (fwd_e, _) = edge_of(n.path[1], n.path[2]);
        line.wait_for(stage, AT_NINE_TEN.1)?;
        let pointer = frame_around(stage, &[labels[3 + back_e]], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[lost], 0.7)?;
        line.wait_for(stage, AT_KNOCKS.1)?;
        ring_corners(stage, &[b.discs[n.path[0]]], &[], 0.6)?;
        line.wait_for(stage, AT_USUALLY.1)?;
        let pointer = frame_around(stage, &[labels[fwd_e]], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.7)?;
        line.wait_for(stage, AT_AROUND.1)?;
        let nine = counter_number(&kit, stage, n.dollars[2])?;
        let rem_nine = remainder_line(&kit, stage, n.dollars[2])?;
        fade_out_all(stage, &[eight, rem_eight], 0.15)?;
        let won = travel(stage, player, n.path[1], n.path[2], palette::good(), STEP_S, Vec::new())?;
        fade_in_all(stage, &[nine, rem_nine], 0.15)?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 10_3: one player's likeliest two rounds, labelled as such; then
        // the whole crowd, settled at 10, 4 and 12
        let line = say(&kit, stage, "10_3")?;
        line.wait_for(stage, AT_LIKELIEST.1)?;
        let path_label = panel_line(&kit, stage, PATH_LABEL, 18.0, palette::player(), PATH_Y)?;
        fade_in_all(stage, &[path_label], 0.5)?;
        flash_all(stage, &[lost, won], 0.7)?;
        line.wait_for(stage, AT_WHOLE.1)?;
        // the player rejoins the crowd: the token gives way to its dot
        let mut brighten = Vec::with_capacity(size);
        for &d in &dots {
            brighten.push(fade_to(d, 1.0, 0.7)?);
        }
        let mut anims: Vec<Box<dyn Animation>> = Vec::with_capacity(size + 6);
        for b2 in brighten {
            anims.push(stage.prepare(b2)?);
        }
        anims.extend(swap(stage, &[player, nine, hud[1], rem_nine, path_label], &head, 0.7)?);
        stage.play_prepared(anims)?;
        stroke_to(stage, &[lost, won], TRACE_DIM, 0.5)?;
        line.wait_for(stage, AT_SETTLED.1)?;
        let mut count_labels = Vec::with_capacity(3);
        for (j, &c) in n.counts.iter().enumerate() {
            let (anchor, right) = count_at(j);
            let m = kit.text(stage, &c.to_string(), 24.0, palette::player(), ORIGIN)?;
            stage.move_to(m, anchor, if right { RIGHT } else { LEFT });
            keep_between(
                stage,
                m,
                -FRAME_X_RADIUS + EDGE,
                FRAME_X_RADIUS - EDGE,
                "a corner's count",
            )?;
            count_labels.push(m);
        }
        let mut all_texts = texts.clone();
        all_texts.extend(&count_labels);
        keep_apart(stage, &all_texts, 0.08, "the settled counts")?;
        fade_in_all(stage, &count_labels, 0.4)?;
        ring_corners(stage, &b.discs, &dots, 0.7)?;

        // 10_4: the settled crowd's good-coin wins: 3 at corner 1, 9 at
        // corner 2, so 9 of 3 + 9 are at corner 2, and each one is the 2 → 0
        // arrow back to the bad coin
        let line = say(&kit, stage, "10_4")?;
        let wins_head = panel_line(&kit, stage, WINS_HEAD, 20.0, Palette::dim(), WINS_YS[0])?;
        fade_in_all(stage, &[wins_head], 0.4)?;
        let mut wins = Vec::with_capacity(2);
        for (k, at) in [AT_THREE, AT_NINE].into_iter().enumerate() {
            let i = k + 1;
            line.wait_for(stage, at.1)?;
            let m = panel_line(&kit, stage, &n.wins_lines[k], 22.0, Palette::ink(), WINS_YS[k + 1])?;
            fade_in_all(stage, &[m], 0.4)?;
            // the corner's forward movers, listed first at it
            ring_corners(stage, &[b.discs[i]], &at_corner(i)[..n.forward[i]], 0.7)?;
            wins.push(m);
        }
        line.wait_for(stage, AT_NINE_OF.1)?;
        let share_head = panel_line(&kit, stage, SHARE_HEAD, 20.0, Palette::dim(), SHARE_HEAD_Y)?;
        let share = kit.tex(stage, &n.share_tex, 32.0, palette::good(), &[], ORIGIN)?;
        stage.move_to(share, v(PANEL_X, SHARE_TEX_Y), LEFT);
        keep_between(
            stage,
            share,
            PANEL_X - 0.01,
            PANEL_RIGHT,
            "the good coin's share at corner 2",
        )?;
        fade_in_all(stage, &[share_head, share], 0.5)?;
        line.wait_for(stage, AT_BACK_ZERO.1)?;
        let stream_w = STREAM_MIN_W + (STREAM_MAX_W - STREAM_MIN_W) * n.forward[2] as f64 / size as f64;
        let mut along: Vec<Vec3> = route(n.path[1], n.path[2]);
        along.truncate(along.len() - 1);
        along.remove(0);
        let stream = stage.arena_mut().add(
            polyline(along)
                .style(
                    Style::default()
                        .fill(palette::player(), 0.0)
                        .stroke(palette::player(), stream_w, STREAM_O),
                )
                .build(),
        );
        stage.play(show_creation(stream).rt(0.9))?;
        line.wait_for(stage, AT_WAITS.1)?;
        ring_corners(stage, &[b.discs[0]], at_corner(0), 0.7)?;
        let mut all_texts = texts.clone();
        all_texts.extend(&count_labels);
        all_texts.extend([wins_head, share_head, share]);
        all_texts.extend(&wins);
        keep_apart(stage, &all_texts, 0.06, "the good coin's wins")?;

        // 10_5: the trap, 0 → 2 → 0, the good coin's wins spent climbing
        // back to a multiple of three (held)
        let line = say(&kit, stage, "10_5")?;
        line.wait_for(stage, AT_TRAP.1)?;
        let trap = panel_line(&kit, stage, &n.trap, 24.0, palette::bad(), TRAP_Y)?;
        all_texts.push(trap);
        keep_apart(stage, &all_texts, 0.06, "the trap's name")?;
        fade_in_all(stage, &[trap], 0.4)?;
        stroke_to(stage, &[lost, won], TRACE_O, 0.4)?;
        flash_all(stage, &[lost, won], 0.8)?;
        line.wait_for(stage, AT_CLIMBING.1)?;
        ring_corners(stage, &[b.discs[0]], at_corner(0), 0.8)?;
        kit.hold(stage)?;
        pause(stage, TRAP_HOLD)?;

        // 10_6: one change: a die picks B's coin instead of the dollars;
        // faces 1 and 2 the bad coin, 3 to 6 the good; the triangle dims
        let line = say(&kit, stage, "10_6")?;
        line.wait_for(stage, AT_ONE_CHANGE.1)?;
        let mut leaving = vec![
            lost, won, stream, trap, wins_head, share_head, share, head[0], head[1], total,
        ];
        leaving.extend(&wins);
        leaving.extend(&count_labels);
        fade_out_all(stage, &leaving, 0.6)?;
        let rule_b = panel_line(&kit, stage, RULE_B, 22.0, palette::game_b(), RULE_YS[0])?;
        fade_in_all(stage, &[rule_b], 0.4)?;
        line.wait_for(stage, AT_DIE.1)?;
        let mut veiled = b.parts();
        veiled.extend(labels);
        veiled.extend(&dots);
        under_veil(stage, &veiled, stage.get_bounding_box_point(tag[0], DOWN)[1])?;
        let veil = rect(
            v(VEIL_LEFT, VEIL_BOTTOM),
            FRAME_X_RADIUS - VEIL_LEFT,
            VEIL_TOP - VEIL_BOTTOM,
            flat(hex(BACKGROUND), VEIL_O),
        )?;
        let veil = stage.arena_mut().add(veil);
        let rule_die = panel_line(&kit, stage, RULE_DIE, 22.0, Palette::ink(), RULE_YS[1])?;
        fade_in_all(stage, &[veil, rule_die], 0.6)?;
        line.wait_for(stage, AT_INSTEAD.1)?;
        let pointer = frame_around(stage, &[rule_b], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.7)?;
        // a row per coin: its faces, an arrow, the coin and its chance
        let mut rows: Vec<Vec<Mob>> = Vec::with_capacity(2);
        let mut row_faces: Vec<Vec<Mob>> = Vec::with_capacity(2);
        let mut coins: Vec<[Mob; 2]> = Vec::with_capacity(2);
        let mut chance_words = Vec::with_capacity(2);
        for (k, at) in [AT_ONE_TWO, AT_ELSE].into_iter().enumerate() {
            line.wait_for(stage, at.1)?;
            let y = ROW_YS[k];
            let mut parts = Vec::new();
            let mut edges = Vec::new();
            for (j, &shown) in n.faces[k].iter().enumerate() {
                let f = face(stage, shown, v(FACE_X0 + FACE_PITCH * j as f64, y))?;
                edges.push(f[0]);
                parts.extend(f);
            }
            let faces_right = FACE_X0 + FACE_PITCH * (n.faces[k].len() - 1) as f64 + FACE / 2.0;
            let pointer = arrow(
                stage,
                v(faces_right + ARROW_GAP, y),
                v(COIN_X - COIN_R - ARROW_GAP, y),
                Palette::dim(),
                2.5,
            );
            parts.extend(pointer);
            let (color, words) = if k == 0 {
                (palette::bad(), BAD_FACE)
            } else {
                (palette::good(), GOOD_FACE)
            };
            let c = coin(&kit, stage, v(COIN_X, y), COIN_R, color, words)?;
            keep_between(stage, c[1], COIN_X - COIN_R, COIN_X + COIN_R, "a coin's face")?;
            let chance = kit.text_left(stage, &n.chances[k], 20.0, color, v(CHANCE_X, y))?;
            keep_between(stage, chance, CHANCE_X - 0.01, SHARE_X - 0.1, "a coin's chance")?;
            parts.extend(c);
            parts.push(chance);
            fade_in_all(stage, &parts, 0.5)?;
            rows.push(parts);
            row_faces.push(edges);
            coins.push(c);
            chance_words.push(chance);
        }

        // 10_7: the bad coin one round in three wherever the player stands;
        // the die game wins 1/3 · 10 % + 2/3 · 75 % ≈ 53.3 % (held)
        let line = say(&kit, stage, "10_7")?;
        line.wait_for(stage, AT_IN_THREE.1)?;
        let mut shares = Vec::with_capacity(2);
        for (k, words) in n.shares.iter().enumerate() {
            let m = kit.text_left(stage, words, 18.0, Palette::ink(), v(SHARE_X, ROW_YS[k]))?;
            keep_between(
                stage,
                m,
                SHARE_X - 0.01,
                VEIL_LEFT - 0.1,
                "a coin's share of the rounds",
            )?;
            shares.push(m);
        }
        fade_in_all(stage, &shares, 0.5)?;
        flash_all(stage, &row_faces[0], 0.7)?;
        line.wait_for(stage, AT_WHEREVER.1)?;
        let corners: Vec<Vec3> = (0..3).map(|i| corner(B_AT, i)).collect();
        rings_at(stage, &corners, CORNER_R + 0.05, &[], 0.8)?;
        line.wait_for(stage, AT_DIE_WINS.1)?;
        let sum = kit.tex(stage, &n.die_tex, 30.0, Palette::ink(), &[], ORIGIN)?;
        stage.move_to(sum, v(PANEL_X, DIE_TEX_Y), LEFT);
        keep_between(stage, sum, PANEL_X - 0.01, PANEL_RIGHT, "the die game's sum")?;
        let verdict = panel_line(&kit, stage, DIE_WINS, 22.0, palette::good(), DIE_WINS_Y)?;
        fade_in_all(stage, &[sum, verdict], 0.5)?;
        let mut panel_texts = vec![rule_b, rule_die, sum, verdict];
        panel_texts.extend(&chance_words);
        panel_texts.extend(&shares);
        panel_texts.extend(coins.iter().map(|c| c[1]));
        keep_apart(stage, &panel_texts, 0.06, "the die game's panel")?;
        kit.hold(stage)?;
        pause(stage, VERDICT_HOLD)?;

        // 10_8: the same two coins, only the rule changed; the die goes, the
        // triangle returns, and reading the money takes B from 53.3 % to 50 %
        let line = say(&kit, stage, "10_8")?;
        line.wait_for(stage, AT_SAME.1)?;
        let centres: Vec<Vec3> = ROW_YS.iter().map(|&y| v(COIN_X, y)).collect();
        rings_at(stage, &centres, COIN_R + 0.06, &[], 0.8)?;
        line.wait_for(stage, AT_RULE.1)?;
        let pointer = frame_around(stage, &[rule_b, rule_die], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_READING.1)?;
        let mut leaving: Vec<Mob> = rows.iter().flatten().copied().collect();
        leaving.extend(&shares);
        leaving.push(veil);
        fade_out_all(stage, &leaving, 0.6)?;
        let pointer = frame_around(stage, &[rule_b], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.7)?;
        line.wait_for(stage, AT_HALF.1)?;
        let from_to = panel_line(&kit, stage, &n.from_to, 22.0, Palette::ink(), FROM_TO_Y)?;
        fade_in_all(stage, &[from_to], 0.5)?;
        let up = now(stage);
        let pointer = frame_around(stage, &[from_to], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        let mut last_texts = texts.clone();
        last_texts.retain(|m| !head.contains(m));
        last_texts.extend([rule_b, rule_die, sum, verdict, from_to]);
        keep_apart(stage, &last_texts, 0.06, "the chapter's last picture")?;
        stand(&kit, stage, up)?;
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

    use super::super::draw::THINK_S;
    use super::super::three_corners::{arrow_along, name_at};
    use super::*;
    use crate::model::PUBLISHED;

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

    /// The chapter's words are true of its numbers: the player arrives on
    /// corner 0 with 9 dollars and the likeliest two rounds go 9, 8, 9
    /// dollars, corners 0, 2, 0; the settled crowd's good coin wins for 75 %
    /// of 4 = 3 and 75 % of 12 = 9, so 9 of 3 + 9 are at corner 2; a die's
    /// one or two is a third of the rounds and the die game wins about
    /// 53.3 %, where reading the money gives 50 %.
    #[test]
    fn the_trap_numbers_are_the_words() {
        let n = numbers().unwrap();
        assert_eq!(n.n, 26);
        assert_eq!(n.counts, [10, 4, 12]);
        assert_eq!(n.forward, [1, 3, 9]);
        assert_eq!(n.dollars, [9, 8, 9]);
        assert_eq!(n.path, [0, 2, 0]);
        assert_eq!(n.wins_lines, ["corner 1: 75 % of 4 = 3", "corner 2: 75 % of 12 = 9"]);
        assert_eq!(n.share_tex, r"\tfrac{9}{3 + 9} = \tfrac{3}{4}");
        assert_eq!(n.trap, "the trap: 0 \u{2192} 2 \u{2192} 0");
        assert_eq!(n.faces, [vec![1, 2], vec![3, 4, 5, 6]]);
        assert_eq!(n.chances, ["10 %", "75 %"]);
        assert_eq!(n.shares, ["1/3 of rounds", "2/3 of rounds"]);
        assert_eq!(
            n.die_tex,
            r"\tfrac{1}{3} \cdot 10\% + \tfrac{2}{3} \cdot 75\% \approx 53.3\%"
        );
        assert_eq!(n.from_to, "reading your money: 53.3 % \u{2192} 50 %");
        // a fn, not a closure: a closure cannot return what it borrows
        // (root's native10 compile, Mail 6893)
        fn words(c: &[String; 3]) -> [&str; 3] {
            c.each_ref().map(String::as_str)
        }
        assert_eq!(words(&n.b_round.tails), ["90", "25", "25"]);
    }

    /// One player's path is an illustration, and the crowd's share is
    /// another claim. The path shown is the likeliest of the four two-round
    /// paths from corner 0, but only 27 times in 40: 0 → 2 → 1 (9 in 40),
    /// 0 → 1 → 2 and 0 → 1 → 0 also happen. The settled crowd's 3/4 comes
    /// from its counts. Failed controls: counting the bad coin's win too
    /// gives 9/13, not 3/4; the 3/4 equals the good coin's 75 % only by the
    /// round-number coins' accident (with a 70 % good coin the share is
    /// 97/134, not 7/10); and with the published coins the share prints
    /// 75.0 % without being exactly 3/4.
    #[test]
    fn the_path_is_one_player_and_the_share_is_the_crowd() {
        let g = ROUND.game_b();
        let paths = two_rounds(&g, 0);
        let chance_of = |p: [usize; 3]| paths.iter().find(|(q, _)| *q == p).unwrap().1;
        assert_eq!(chance_of([0, 2, 0]), Ratio::new(27, 40));
        assert_eq!(chance_of([0, 2, 1]), Ratio::new(9, 40));
        assert_eq!(chance_of([0, 1, 2]), Ratio::new(3, 40));
        assert_eq!(chance_of([0, 1, 0]), Ratio::new(1, 40));
        assert_eq!(paths.iter().fold(Ratio::ZERO, |a, (_, p)| a + *p), Ratio::ONE);
        let n = numbers().unwrap();
        let at2 = Ratio::new(n.forward[2] as i128, (n.forward[1] + n.forward[2]) as i128);
        assert_eq!(at2, good_wins_at2(&g));
        assert_eq!(at2, Ratio::new(3, 4));
        assert_ne!(at2, chance_of([0, 2, 0]));
        // counting the bad coin's one win as well
        let all_wins: usize = n.forward.iter().sum();
        assert_eq!(Ratio::new(n.forward[2] as i128, all_wins as i128), Ratio::new(9, 13));
        // another good coin: the share is not the coin's chance
        let other = Game {
            up: [20, 140, 140],
            den: 200,
        };
        assert_eq!(good_wins_at2(&other), Ratio::new(97, 134));
        assert_ne!(good_wins_at2(&other), other.chance(1));
        // the published coins
        let published = good_wins_at2(&PUBLISHED.game_b());
        assert_eq!(published.pct(1), "75.0");
        assert_ne!(published, Ratio::new(3, 4));
    }

    /// The die changes only the rule. With the same two coins it wins 8/15
    /// wherever the player stands, more than half; reading the money gives
    /// exactly half. As a game on the triangle the die's chance is the same
    /// at every corner, so its crowd settles at equal thirds: the corners no
    /// longer pick the coin. Failed control: B's own crowd does not sit at
    /// thirds. With the published coins the die game also wins (317/600)
    /// while game B loses.
    #[test]
    fn the_die_changes_only_the_rule() {
        let n = numbers().unwrap();
        let g = ROUND.game_b();
        let die = ROUND.die_chosen_win();
        assert_eq!(die, Ratio::new(8, 15));
        assert!(die > g.win() && g.win() == Ratio::new(1, 2));
        let per_corner = i64::try_from(die.num() * 600 / die.den()).unwrap();
        let as_game = Game {
            up: [per_corner; 3],
            den: 600,
        };
        assert_eq!(as_game.chance(0), die);
        assert_eq!(as_game.shares(), [Ratio::new(1, 3); 3]);
        assert_ne!(g.shares(), [Ratio::new(1, 3); 3]);
        assert_eq!(n.faces[0].len() * 3, 6);
        assert_eq!(PUBLISHED.die_chosen_win(), Ratio::new(317, 600));
        assert!(PUBLISHED.die_chosen_win() > Ratio::new(1, 2) && PUBLISHED.game_b().mean() < Ratio::ZERO);
    }

    /// The token walks on the arrows the steps use: a route starts and ends
    /// at the corners' centres, its shaft points are the arrow shaft's (both
    /// built from the arrow's ends and its bow's apex), and its trace grows
    /// from the shaft's start to the arrow's tip, never into a corner.
    #[test]
    fn the_token_walks_on_its_arrows() {
        for (from, to) in [(0, 2), (2, 0)] {
            let points = route(from, to);
            assert_eq!(points[0], corner(B_AT, from));
            assert_eq!(*points.last().unwrap(), corner(B_AT, to));
            let cum = lengths(&points);
            let total = *cum.last().unwrap();
            assert_eq!(trace_to(&points, &cum, 0.0), vec![points[1], points[1]]);
            let full = trace_to(&points, &cum, total);
            assert_eq!(full, points[1..points.len() - 1].to_vec());
            let half = trace_to(&points, &cum, total / 2.0);
            assert!(half.len() > 2 && half.len() < full.len());
            let (mid, _) = at_length(&points, &cum, total / 2.0);
            assert_eq!(*half.last().unwrap(), mid);
        }
        run(|stage| {
            for (from, to) in [(0, 2), (2, 0)] {
                let (e, forward) = edge_of(from, to);
                let [shaft_mob, _] = arrow_along(stage, Which::B, e, forward);
                // a polyline's points are its corners with handles between
                // them: every corner of ours is one of them, the ends match
                let drawn = stage.get_points(shaft_mob).unwrap();
                let points = route(from, to);
                let ours = &points[1..points.len() - 2];
                let near = |p: Vec3, q: Vec3| (p[0] - q[0]).abs() < 1e-6 && (p[1] - q[1]).abs() < 1e-6;
                assert!(near(drawn[0], ours[0]) && near(*drawn.last().unwrap(), *ours.last().unwrap()));
                for q in ours {
                    assert!(
                        drawn.iter().any(|&p| near(p, *q)),
                        "{from} → {to}: no drawn point at {q:?}"
                    );
                }
            }
            Ok(())
        });
    }

    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in CUES {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The layout keeps its rooms: the panel's lines stack without touching
    /// and end left of the veil; the die faces stand apart in their rows and
    /// clear of the arrows to the coins; the coins, chances and shares sit in
    /// order; the counts stand beside their corners clear of the names; the
    /// veil covers B's triangle and stops below the badge.
    #[test]
    fn the_layout_keeps_its_rooms() {
        const {
            assert!(QUESTION_Y - 0.15 > THINK_AT[1] + 0.62 + 0.1);
            assert!(PATH_Y - WINS_YS[0] >= 0.5);
            assert!(WINS_YS[0] - WINS_YS[1] >= 0.4 && WINS_YS[1] - WINS_YS[2] >= 0.4);
            assert!(WINS_YS[2] - SHARE_HEAD_Y >= 0.45 && SHARE_HEAD_Y - SHARE_TEX_Y >= 0.55);
            assert!(SHARE_TEX_Y - 0.35 > TRAP_Y + 0.2);
            assert!(RULE_YS[0] - RULE_YS[1] >= 0.4 && RULE_YS[1] - 0.15 > ROW_YS[0] + FACE / 2.0 + 0.1);
            assert!(ROW_YS[0] - ROW_YS[1] > FACE + 0.3 && ROW_YS[0] - ROW_YS[1] > 2.0 * COIN_R + 0.15);
            assert!(ROW_YS[1] - COIN_R > DIE_TEX_Y + 0.35);
            assert!(DIE_TEX_Y - 0.35 > DIE_WINS_Y + 0.15 && DIE_WINS_Y - FROM_TO_Y >= 0.5);
            assert!(FROM_TO_Y - 0.15 > -FRAME_Y_RADIUS + 0.3);
            assert!(FACE_PITCH > FACE + 0.05 && PIP_OFF + PIP_R < FACE / 2.0 - 0.03);
            assert!(FACE_X0 - FACE / 2.0 > PANEL_X - 0.01);
            assert!(FACE_X0 + 3.0 * FACE_PITCH + FACE / 2.0 + ARROW_GAP + 0.3 < COIN_X - COIN_R - ARROW_GAP);
            assert!(CHANCE_X + 0.6 < SHARE_X - 0.1);
            assert!(PANEL_RIGHT < VEIL_LEFT);
            assert!(B_AT[1] - super::super::three_corners::TITLE_DY - 0.2 > VEIL_BOTTOM);
            assert!(super::super::one_player_to_a_crowd::DOT_R < super::super::draw::TOKEN_R);
        }
        // the veil over B's triangle: every corner's disc inside it (the
        // scene's `under_veil` checks every part's box as typeset, and the
        // badge, when it runs)
        for i in 0..3 {
            let c = corner(B_AT, i);
            assert!(
                c[0] - CORNER_R > VEIL_LEFT && c[0] + CORNER_R < FRAME_X_RADIUS,
                "corner {i}"
            );
            assert!(
                c[1] + CORNER_R < VEIL_TOP && c[1] - CORNER_R > VEIL_BOTTOM,
                "corner {i}"
            );
        }
        // the counts beside their corners, clear of the discs and names
        for j in 0..3 {
            let (anchor, right) = count_at(j);
            let c = corner(B_AT, j);
            assert!((anchor[0] - c[0]).abs() >= CORNER_R + ROOM - 1e-12);
            assert_eq!(right, j == 2);
            let name = name_at(B_AT, j, 0.15, 0.25);
            assert!((anchor[1] - name[1]).abs() > 0.3, "corner {j}'s count meets its name");
        }
        // the panel's right edge against B's corner 2
        let c2 = corner(B_AT, 2);
        assert!(PANEL_RIGHT < c2[0] - CORNER_R - ROOM);
    }

    /// The chapter runs headless to its end with the scratch voice and real
    /// typesetting (so every `keep_between` and `keep_apart` guard meets the
    /// true text widths): every line of chapter 10 is said once, in order;
    /// the prediction (10_1) gets its silent think time; the two rounds get a
    /// beat after 10_2; the trap is held after 10_5 and the die game's
    /// verdict after 10_7.
    #[test]
    fn the_chapter_runs_and_leaves_room_to_think() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = TheTrap { kit: kit.clone() };
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut NullSceneSink).unwrap_or_else(|e| panic!("10_the_trap: {e}"));
        let said = narrator.said();
        let ids: Vec<&str> = said.iter().map(|(id, _)| id.as_str()).collect();
        let expected: Vec<String> = (1..=8).map(|k| format!("10_{k}")).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        let at = |id: &str| said.iter().find(|(l, _)| l == id).unwrap().1;
        let end_of = |id: &str| at(id) + narrator.length(id).unwrap();
        assert!(
            at("10_2") - end_of("10_1") >= THINK_S,
            "{} s to think",
            at("10_2") - end_of("10_1")
        );
        assert!(at("10_3") - end_of("10_2") >= BEAT, "the two rounds get no beat");
        assert!(at("10_6") - end_of("10_5") >= TRAP_HOLD, "the trap is not held");
        assert!(at("10_8") - end_of("10_7") >= VERDICT_HOLD, "the verdict is not held");
        assert!(kit.finished.borrow().is_some(), "the chapter did not finish");
    }
}
