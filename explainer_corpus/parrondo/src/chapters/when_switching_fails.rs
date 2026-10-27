//! Chapter 16, when switching fails. Until now a fair coin chose the game
//! before every round (16_1: from an empty frame, the filled published
//! badge, B's triangle titled "the mix" with the mix's chances 29.5, 62, 62
//! % forward and 70.5, 38, 38 % back, the teal puddles at its settled 34.5,
//! 25.4, 40.1 %, and the panel's first line, "random: a fair coin picks A or
//! B each round", beside a small gold coin). The simplest fixed order: a
//! strip "A B A B …" (A blue, B in B's color) named "strict alternation",
//! the triangle's title "A B A B …"; half the rounds A, as in the mix
//! (16_2). Does it win? (16_3, a prediction.) It loses, "−0.67 cents a
//! round" (16_4). A's round costs a cent wherever the crowd stands ("each A
//! round: −1.00 cent"), so only B's rounds matter, and the dial: a chart of
//! the share on corner 0 before each B round against the B round, 1 to 12,
//! its dashed break-even line at 37.7 %, "B wins" under it, "B loses" over
//! it (16_5). From game B's settled crowd (38.4 % on corner 0) the schedule
//! is played one round at a time, A's coin on A rounds and B's on B rounds:
//! a frame steps along the strip, "round 1 of 24" counts, the puddles move,
//! and before every B round a teal dot lands at corner 0's share: 30.7 %
//! first, then climbing past the line to settle at 38.0 % (16_6). One
//! settled cycle explains it: B's bad coin knocks most of corner 0 back to
//! 2 (a red stream), its good coin carries most of corner 1 on to 2 (a
//! green one), leaving "B leaves on corner 2: 68.95 %"; then A's coin sends
//! 49.5 % of them forward onto corner 0 (blue) and a few step back from 1
//! (dim): "49.5 % × 68.95 % ≈ 34.13 %", "50.5 % × 7.59 % ≈ 3.83 %",
//! "34.13 % + 3.83 % = 37.96 %" on corner 0 when B plays, the settled dots
//! pointed at (16_7). The tempting shortcut, averaging the two games'
//! chances at each corner, gives the coin-toss mix's coins (29.5 %, 62 %,
//! 62 %) and predicts +1.57 cents; it is struck, "actually: −0.67" (16_8).
//! Two A's then two B's from B's settled crowd again: gold dots, all below
//! the line, the settled pair "36.0 %" and "33.3 %", "wins: +1.47 cents a
//! round" (16_9). The three side by side, per round played (a table:
//! schedule, corner 0 when B plays, cents a round) (16_10). The control:
//! the triangle goes, two ordinary coins come in, 49 % and 48 %, which
//! ignore the money, and a column says −3.00 cents for every schedule
//! (16_11). The share matters too: the curve's axes, its ends B alone −0.87
//! and A alone −1.00, the coin-toss mix +1.57 at 50 %, red points at 5 %
//! (−0.23) and 90 % (−0.34) (16_12); the curve, computed at every half
//! percent, crossing zero near 7 % and 85 %, a gold band between where the
//! forward product is bigger (16_13, held); the crossings are 7.03 % and
//! 84.71 % (rounded), the mix wins only strictly between, the edges of the
//! 0.1 % grid under the curve in cents a round and the "precisely" card,
//! K07, held for its reading time (16_14). No general rule: the curve goes
//! and B's triangle returns with the mix's settled crowd (16_15, held).
//!
//! The schedules are played round by round from `model::played` (never by
//! averaged chances), their settled cycles from `model::cycle` (exact), the
//! curve from `PUBLISHED.mix_share` at every 0.5 % (exact, drawn in
//! floats); every number is checked against `crate::facts`, and the scene
//! refuses to draw if the words would stop being true of them. No typeset
//! text is indicated, scaled or recoloured: words get a frame, shapes pulse,
//! a changed label is replaced. The fifth AB dot (37.65 %) and the line
//! (37.69 %) are too close to tell apart, so the voice names no round.

use explainer_kit::props::precisely;
use explainer_kit::{Frame2, Kit, Palette, Timed, fade_all, rect, v};
use fmn::core::constants::FRAME_X_RADIUS;
use fmn::prelude::*;

use super::draw::{
    coin, dashed, fade_in_all, fade_out_all, fade_to, fail, flat, fly, keep_between, line_style, now, pause, read_s,
    say, screen, think, together, wait_until,
};
use super::looks_hopeless::{POINT_PAD, badge_text, frame_around, full_badge, point_at};
use super::net_flow::{
    CHANGE_S, INSPECT_S, PANEL_END, Puddles, THINK_AT, csv, draw_bands, mix_labels, mix_title, puddles, stream, strike,
};
use super::one_dial::{BEAT, PERCENT, pct_value};
use super::one_player_to_a_crowd::{PANEL_X, ring_corners};
use super::three_corners::{B_AT, Chances, TITLE_DY, Which, chances, keep_apart, swap, triangle};
use crate::facts::{self, EDGE_SHARES, ORDINARY, PLAYED_ROUNDS, SHARES, WHOLE_EDGES};
use crate::model::{self, Coins, DEN, Game, PUBLISHED, Ratio, periodic, schedule};
use crate::palette;

pub struct WhenSwitchingFails {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const RANDOM_LINE: &str = "random: a fair coin picks A or B each round";
const AB: &str = "AB";
const AABB: &str = "AABB";
const AB_NAME: &str = "strict alternation";
const AABB_NAME: &str = "two and two";
const ELLIPSIS: &str = "\u{2026}";
const ROUND_WORD: &str = "round";
const OF_WORD: &str = "of";
const CHART_TITLE: &str = "corner 0 when B plays";
const B_ROUND: &str = "B round";
const B_WINS: &str = "B wins";
const B_LOSES: &str = "B loses";
const BREAKS_EVEN: &str = "B breaks even:";
const FROM_B: &str = "from game B's settled crowd";
const SETTLES: &str = "settles at";
const LEAVES: &str = "B leaves on corner 2:";
const AVERAGING: &str = "averaging A's and B's chances at each corner:";
const MIX_COINS: &str = "the coin-toss mix's coins";
const PREDICTS: &str = "predicts";
const ACTUALLY: &str = "actually:";
const CENTS_A_ROUND: &str = "cents a round";
const WHEN_B_PLAYS: &str = "on corner 0 when B plays";
const A_COST: &str = "each A round:";
const CENT: &str = "cent";
const SCHEDULE_HEAD: &str = "schedule";
const RANDOM_ROW: &str = "random (a fair coin)";
const PER_ROUND: &str = "per round played, A's rounds included";
const ORDINARY_HEAD: &str = "ordinary coins (they ignore the money)";
const SHARE_AXIS: &str = "A's share of the rounds (chosen at random)";
const B_ALONE: &str = "B alone";
const A_ALONE: &str = "A alone";
const PRODUCT_BIGGER: &str = "forward product bigger: the mix wins";
const WINS: &str = "wins";
const LOSES: &str = "loses";
const EDGES_HEAD: &str = "cents a round:";

/// Where each picture lands: (line, the phrase it waits for).
const AT_FAIR_COIN: (&str, &str) = ("16_1", "a fair coin chose the game");
const AT_MATTER: (&str, &str) = ("16_1", "Does it matter");
const AT_FIXED: (&str, &str) = ("16_2", "the simplest fixed order");
const AT_HALF: (&str, &str) = ("16_2", "Half the rounds are A");
const AT_ONLY_ORDER: (&str, &str) = ("16_2", "Only the order has changed");
const AT_WIN: (&str, &str) = ("16_3", "does strict alternation win");
const AT_LOSES: (&str, &str) = ("16_4", "It loses");
const AT_ORDER_ALONE: (&str, &str) = ("16_4", "The order alone");
const AT_WATCH: (&str, &str) = ("16_4", "watch the crowd");
const AT_COST: (&str, &str) = ("16_5", "cost one cent each");
const AT_RIDES: (&str, &str) = ("16_5", "everything rides on B's rounds");
const AT_DIAL: (&str, &str) = ("16_5", "the share of the crowd on corner zero");
const AT_BREAKS: (&str, &str) = ("16_5", "B breaks even at");
const AT_BELOW: (&str, &str) = ("16_5", "Below that line");
const AT_ABOVE: (&str, &str) = ("16_5", "and above it");
const AT_START: (&str, &str) = ("16_6", "where game B alone leaves it");
const AT_ALTERNATE: (&str, &str) = ("16_6", "and alternate");
const AT_FIRST_B: (&str, &str) = ("16_6", "The first B round meets");
const AT_COMES_BACK: (&str, &str) = ("16_6", "each time B comes back");
const AT_SETTLE: (&str, &str) = ("16_6", "settle at thirty-eight percent");
const AT_EVERY_LOSES: (&str, &str) = ("16_6", "every B round loses");
const AT_WHY: (&str, &str) = ("16_7", "Why does alternating fill corner zero");
const AT_CYCLE: (&str, &str) = ("16_7", "Follow one cycle");
const AT_KNOCKS: (&str, &str) = ("16_7", "knocks most of corner zero back");
const AT_CARRIES: (&str, &str) = ("16_7", "carries most of corner one forward");
const AT_LEAVES: (&str, &str) = ("16_7", "each B round leaves about sixty-nine percent");
const AT_A_PLAYS: (&str, &str) = ("16_7", "Then A plays");
const AT_SENDS: (&str, &str) = ("16_7", "sends just under half of them forward");
const AT_FEW: (&str, &str) = ("16_7", "a few step back");
const AT_ALMOST: (&str, &str) = ("16_7", "Almost thirty-eight percent");
const AT_SHORTCUT: (&str, &str) = ("16_8", "take a shortcut");
const AT_GIVES: (&str, &str) = ("16_8", "That gives twenty-nine and a half percent");
const AT_PREDICTS: (&str, &str) = ("16_8", "so it predicts the mix's win");
const AT_FIXED_NEXT: (&str, &str) = ("16_8", "the next game is already fixed");
const AT_PREVIOUS_ROUNDS: (&str, &str) = ("16_8", "Its crowd depends on the previous rounds");
const AT_TWO_TWO: (&str, &str) = ("16_9", "play two A's, then two B's");
const AT_SPREAD: (&str, &str) = ("16_9", "Two A rounds in a row");
const AT_36: (&str, &str) = ("16_9", "about thirty-six percent");
const AT_33: (&str, &str) = ("16_9", "only about thirty-three");
const AT_TWO_WINS: (&str, &str) = ("16_9", "Two and two wins");
const AT_SIDE: (&str, &str) = ("16_10", "Put the three side by side");
const AT_ROW_MIX: (&str, &str) = ("16_10", "The coin-toss mix wins one point five seven cents");
const AT_ROW_AB: (&str, &str) = ("16_10", "Strict alternation loses zero point six seven");
const AT_ROW_AABB: (&str, &str) = ("16_10", "Two and two wins one point four seven");
const AT_STANDS: (&str, &str) = ("16_10", "where the crowd stands when B plays");
const AT_CONTROL: (&str, &str) = ("16_11", "Now the control");
const AT_ORDINARY: (&str, &str) = ("16_11", "the two ordinary coins");
const AT_TOSS: (&str, &str) = ("16_11", "Toss for the game");
const AT_ALTERNATE_PLAIN: (&str, &str) = ("16_11", "alternate, or");
const AT_TWO_TWO_PLAIN: (&str, &str) = ("16_11", "play two and two");
const AT_NEVER_ORDER: (&str, &str) = ("16_11", "never the order");
const AT_SHARE: (&str, &str) = ("16_12", "The share of each game matters too");
const AT_ONE_IN_20: (&str, &str) = ("16_12", "one round in twenty");
const AT_NINE_IN_10: (&str, &str) = ("16_12", "nine rounds in ten");
const AT_COMPUTE: (&str, &str) = ("16_13", "Compute the mix at every share in between");
const AT_ABOUT_7: (&str, &str) = ("16_13", "about seven percent");
const AT_ABOUT_85: (&str, &str) = ("16_13", "about eighty-five percent");
const AT_BETWEEN: (&str, &str) = ("16_13", "between the crossings");
const AT_FINE: (&str, &str) = ("16_14", "seven point zero three");
const AT_STRICT: (&str, &str) = ("16_14", "strictly between them");
const AT_EXACTLY_7: (&str, &str) = ("16_14", "At exactly seven percent");
const AT_85: (&str, &str) = ("16_14", "at eighty-five percent it has already");
const AT_NO_RULE: (&str, &str) = ("16_15", "no general rule");
const AT_SOME: (&str, &str) = ("16_15", "some orders and some shares");
const AT_READS: (&str, &str) = ("16_15", "one game reads the money");
#[cfg(test)]
const CUES: [(&str, &str); 65] = [
    AT_FAIR_COIN,
    AT_MATTER,
    AT_FIXED,
    AT_HALF,
    AT_ONLY_ORDER,
    AT_WIN,
    AT_LOSES,
    AT_ORDER_ALONE,
    AT_WATCH,
    AT_COST,
    AT_RIDES,
    AT_DIAL,
    AT_BREAKS,
    AT_BELOW,
    AT_ABOVE,
    AT_START,
    AT_ALTERNATE,
    AT_FIRST_B,
    AT_COMES_BACK,
    AT_SETTLE,
    AT_EVERY_LOSES,
    AT_WHY,
    AT_CYCLE,
    AT_KNOCKS,
    AT_CARRIES,
    AT_LEAVES,
    AT_A_PLAYS,
    AT_SENDS,
    AT_FEW,
    AT_ALMOST,
    AT_SHORTCUT,
    AT_GIVES,
    AT_PREDICTS,
    AT_FIXED_NEXT,
    AT_PREVIOUS_ROUNDS,
    AT_TWO_TWO,
    AT_SPREAD,
    AT_36,
    AT_33,
    AT_TWO_WINS,
    AT_SIDE,
    AT_ROW_MIX,
    AT_ROW_AB,
    AT_ROW_AABB,
    AT_STANDS,
    AT_CONTROL,
    AT_ORDINARY,
    AT_TOSS,
    AT_ALTERNATE_PLAIN,
    AT_TWO_TWO_PLAIN,
    AT_NEVER_ORDER,
    AT_SHARE,
    AT_ONE_IN_20,
    AT_NINE_IN_10,
    AT_COMPUTE,
    AT_ABOUT_7,
    AT_ABOUT_85,
    AT_BETWEEN,
    AT_FINE,
    AT_STRICT,
    AT_EXACTLY_7,
    AT_85,
    AT_NO_RULE,
    AT_SOME,
    AT_READS,
];

// ------------------------------------------------------------------ layout

/// The panel, left of B's triangle (14's): the random line with its coin,
/// the strip and its counter, the schedule's name and verdict, A's cost,
/// the chart's title, then the chart.
const RANDOM_Y: f64 = 3.3;
const STRIP_Y: f64 = 2.85;
const NAME_Y: f64 = 2.4;
const COST_Y: f64 = 1.95;
const CHART_TITLE_Y: f64 = 1.55;
const COIN_R: f64 = 0.13;
const COIN_X: f64 = PANEL_X + COIN_R;
const WORDS_X: f64 = PANEL_X + 0.4;
const WORDS_SIZE: f64 = 20.0;
const SMALL: f64 = 18.0;
/// The strip: twelve cells from [`STRIP_X`], [`PITCH`] apart, then "…";
/// the counter right of them, the verdict right of the schedule's name.
const CELLS: usize = 12;
const STRIP_X: f64 = PANEL_X + 0.15;
const PITCH: f64 = 0.32;
const CELL_SIZE: f64 = 22.0;
const CURSOR_W: f64 = 0.28;
const CURSOR_H: f64 = 0.38;
const COUNTER_X: f64 = -2.45;
const VERDICT_X: f64 = -4.4;
/// The chart: the share on corner 0 before each B round (30 to 40 %)
/// against the B round (1 to 12).
const CHART: Frame2 = Frame2 {
    center: [-3.8, -0.25, 0.0],
    width: 4.4,
    height: 3.0,
    x: (0.5, 12.5),
    y: (30.0, 40.0),
};
const CHART_TICKS: [f64; 3] = [30.0, 35.0, 40.0];
const TICK_DY: f64 = 0.22;
const DOT_R: f64 = 0.06;
const DOT_WORDS: f64 = 0.12;
/// The break-even line's words sit [`NEAR`] − [`LIFT`] above or below it.
const NEAR: f64 = 0.28;
const LIFT: f64 = 0.07;
/// Three lines under the chart (16_7's sums, 16_8's shortcut), and one
/// under the triangle (16_6's start, 16_7's corner 2).
const LOW_YS: [f64; 3] = [-2.45, -2.95, -3.45];
const SUM_SIZE: f64 = 26.0;
const UNDER_Y: f64 = -2.8;
const UNDER_X: f64 = 0.0;
/// Streams along the triangle's arrows (14's widths): the narrowest band,
/// the widest (a whole crowd), their opacity.
const STREAM_MIN_W: f64 = 1.5;
const STREAM_MAX_W: f64 = 16.0;
const STREAM_O: f64 = 0.7;
/// The table (16_10, 16_11): its head, a row per schedule, the note under
/// it; its columns, and the ordinary coins' column right of the panel once
/// the triangle has gone, their two coins beside it.
const HEAD_Y: f64 = 2.9;
const HEAD_SIZE: f64 = 16.0;
const ROW_YS: [f64; 3] = [2.35, 1.8, 1.25];
const NOTE_Y: f64 = 0.7;
const MEETS_X: f64 = -4.2;
const CENTS_X: f64 = -1.95;
const PLAIN_X: f64 = 0.0;
const PLAIN_AT: [Vec3; 2] = [[4.3, 2.4, 0.0], [5.5, 2.4, 0.0]];
const PLAIN_R: f64 = 0.42;
/// The curve (16_12 to 16_14): the mix's cents a round against A's share.
const CURVE: Frame2 = Frame2 {
    center: [0.0, 0.9, 0.0],
    width: 11.6,
    height: 3.4,
    x: (0.0, 100.0),
    y: (-1.4, 2.0),
};
const CURVE_TICKS: [f64; 4] = [-1.0, 0.0, 1.0, 2.0];
const SHARE_TICKS: [f64; 3] = [0.0, 50.0, 100.0];
const CURVE_TITLE_Y: f64 = 2.95;
const AXIS_Y: f64 = -1.4;
const EDGES_Y: f64 = -1.85;
const EDGES_X: [f64; 4] = [-5.0, -2.2, 0.6, 3.5];
const TEX_SIZE: f64 = 24.0;
/// Labels beside the curve: beside its points, never on the line.
const END_DX: f64 = 0.15;
const POINT_R: f64 = 0.08;
const BAND_W: f64 = 10.0;
/// Seconds: the puddles' move to a new crowd, one round's move at its
/// slowest and fastest, the curve's drawing.
const MOVE_S: f64 = 1.0;
const ROUND_S: f64 = 0.8;
const FAST_S: f64 = 0.3;
const CURVE_S: f64 = 3.0;

// ---------------------------------------------------------------- numbers

/// A schedule played from game B's settled crowd: the crowd's shares before
/// each of its rounds (and after the last), and which rounds are B's.
struct Played {
    states: Vec<[f64; 3]>,
    b: Vec<usize>,
}

/// The chapter's numbers, checked against the words they illustrate.
struct Numbers {
    badge: String,
    mix: Chances,
    /// The mix's settled crowd and game B's, as fractions of the crowd.
    shares: [[f64; 3]; 2],
    /// Where B breaks even, a percentage on the chart.
    line: f64,
    fair: String,
    a_cost: String,
    ab: Played,
    aabb: Played,
    ab_verdict: String,
    first_dot: String,
    settles: String,
    /// AB's settled cycle: before an A round, before a B round.
    cycle: [[f64; 3]; 2],
    /// The streams' widths: B's back from 0 and forward from 1, then A's
    /// forward from 2 and back from 1.
    flows: [f64; 4],
    leaves: String,
    refill: [String; 3],
    shortcut: [String; 3],
    actually: String,
    aabb_verdict: String,
    aabb_pair: [String; 2],
    table: [[String; 3]; 3],
    plain_faces: [String; 2],
    plain: [String; 3],
    /// The curve as (A's share in %, cents a round) every half percent.
    curve: Vec<(f64, f64)>,
    ends: [String; 3],
    points: [(f64, f64, String); 2],
    crossings: [f64; 2],
    whole_ticks: [String; 2],
    fine_ticks: [String; 2],
    edges: [(String, bool); 4],
    k07: String,
}

/// A round's gain at shares `v`: twice its chance of a win, less one.
fn gain(g: &Game, v: &[Ratio; 3]) -> Ratio {
    Ratio::int(2) * g.win_at(v) - Ratio::ONE
}

/// Whether `x` is within `tol` of `target`.
fn near(x: Ratio, target: Ratio, tol: Ratio) -> bool {
    x - target < tol && target - x < tol
}

/// The rounds of a schedule `pattern`, repeated, that are B's (of the first
/// `rounds`).
fn b_rounds(pattern: &str, rounds: usize) -> Vec<usize> {
    let letters: Vec<char> = pattern.chars().collect();
    (0..rounds).filter(|&k| letters[k % letters.len()] == 'B').collect()
}

/// A schedule's title: its first four rounds, spaced, then "…".
fn spaced(pattern: &str) -> String {
    let four: Vec<String> = pattern.chars().cycle().take(4).map(String::from).collect();
    format!("{} {ELLIPSIS}", four.join(" "))
}

/// A percentage as the screen writes it, "37.7 %".
fn pct_words(x: &str) -> String {
    format!("{x} {PERCENT}")
}

/// A share as a percentage at one place, as the facts print it.
fn pct1(x: f64) -> String {
    format!("{:.1}", 100.0 * x)
}

/// A stream's width for a flow of `x` of the crowd (14's scale).
fn width(x: Ratio) -> f64 {
    STREAM_MIN_W + (STREAM_MAX_W - STREAM_MIN_W) * x.to_f64()
}

fn numbers() -> Result<Numbers, String> {
    let f = facts::facts()?;
    let get = |k: &str| f.get(k).map(str::to_owned).ok_or_else(|| format!("no fact {k}"));
    let (a, b, mix) = (PUBLISHED.game_a(), PUBLISHED.game_b(), PUBLISHED.mix());
    let pcts = |v: [Ratio; 3]| csv(v.map(|x| x.pct(1)));
    // 16_1: the mix as 11 left it; 16_6: game B's settled crowd
    if pcts(mix.shares()) != get("pub.mix.weights.pct")? || pcts(b.shares()) != get("pub.b.weights.pct")? {
        return Err("16_1 and 16_6 show the mix's and game B's settled crowds".into());
    }
    if b.shares()[0].pct(1) != "38.4" {
        return Err("16_6 says thirty-eight point four percent on corner zero".into());
    }
    // 16_1, 16_2: a fair coin chose the game, so A has half the rounds, as
    // in both schedules
    let half = Ratio::new(1, 2);
    if (0..3).any(|i| mix.chance(i) != PUBLISHED.mix_share(1, 2).chance(i)) {
        return Err("16_1 says a fair coin chose the game".into());
    }
    let a_share = |p: &str| Ratio::new(p.matches('A').count() as i128, p.len() as i128);
    if csv([half, a_share(AB), a_share(AABB)]) != get("sched.a_share")? || a_share(AB) != half || a_share(AABB) != half
    {
        return Err("16_2 says half the rounds are A, exactly as in the mix".into());
    }
    // 16_4: strict alternation loses about two thirds of a cent a round
    let (ab, aabb) = (schedule(AB, &PUBLISHED), schedule(AABB, &PUBLISHED));
    let (ab_mean, ab_back) = periodic(&ab);
    if !ab_back || ab_mean.cents(2) != get("pub.sched.ab.cents")? {
        return Err("16_4's schedule disagrees with the facts".into());
    }
    if !near(ab_mean, Ratio::new(-2, 300), Ratio::new(1, 2000)) {
        return Err("16_4 says it loses about two thirds of a cent a round".into());
    }
    // 16_5: A's coin is the same at every corner, so its round costs a cent
    // wherever the crowd stands; B's round is decided by corner 0's share
    let corners = [[Ratio::ONE, Ratio::ZERO, Ratio::ZERO], b.shares(), mix.shares()];
    if (0..3).any(|i| a.chance(i) != a.chance(0))
        || a.mean() != Ratio::new(-1, 100)
        || corners.iter().any(|v| gain(&a, v) != a.mean())
        || a.mean().cents(2) != get("pub.a.mean.cents")?
    {
        return Err("16_5 says each A round costs one cent, wherever the crowd stands".into());
    }
    let line = facts::fair_share(&PUBLISHED);
    let rest = Ratio::ONE - line;
    if gain(&b, &[line, rest, Ratio::ZERO]) != Ratio::ZERO
        || gain(&b, &[line, Ratio::ZERO, rest]) != Ratio::ZERO
        || line.pct(1) != get("pub.b.fair_share.pct")?
    {
        return Err("16_5 says B breaks even at 37.7 % on corner 0, however the rest stand".into());
    }
    let tiny = Ratio::new(1, 1000);
    if gain(&b, &[line - tiny, rest + tiny, Ratio::ZERO]) <= Ratio::ZERO
        || gain(&b, &[line + tiny, rest - tiny, Ratio::ZERO]) >= Ratio::ZERO
    {
        return Err("16_5 says B's round wins below the line and loses above it".into());
    }
    // 16_6: alternation played round by round from B's settled crowd
    let start = b.shares().map(|w| w.to_f64());
    let ab_played = Played {
        states: model::played(&ab, start, PLAYED_ROUNDS + 1),
        b: b_rounds(AB, PLAYED_ROUNDS),
    };
    let dots: Vec<f64> = ab_played.b.iter().map(|&k| ab_played.states[k][0]).collect();
    let line_f = line.to_f64();
    if pct1(dots[0]) != get("pub.sched.ab.first.b.at0.pct")? || dots[0] >= line_f {
        return Err("16_6 says the first B round meets thirty point seven percent, and wins".into());
    }
    if dots.windows(2).any(|w| w[1] <= w[0]) {
        return Err("16_6 says each time B comes back a little more of the crowd waits on corner 0".into());
    }
    let last = *dots.last().ok_or("no B rounds")?;
    let cycle = model::cycle(&ab);
    let settled = get("pub.sched.ab.cycle.b.pct")?;
    let ab_meets = settled.split(',').next().ok_or("no settled share")?.to_owned();
    if last <= line_f || pct1(last) != cycle[1][0].pct(1) || pct1(last) != ab_meets {
        return Err("16_6 says the dots climb past the line and settle at thirty-eight percent".into());
    }
    if cycle[1][0].pct(0) != "38" || cycle[1][0] <= line {
        return Err("16_6 says thirty-eight percent, just above the line".into());
    }
    // 16_7: one settled cycle, exactly: B's round takes the crowd from its
    // shares before B to those before A, and A's back again
    if model::evolve(&cycle[1], &b) != cycle[0] || model::evolve(&cycle[0], &a) != cycle[1] {
        return Err("16_7 follows one cycle of the settled alternation".into());
    }
    let pct2 = |v: &[Ratio; 3]| csv(v.iter().map(|x| x.pct(2)));
    if pct2(&cycle[0]) != get("pub.sched.ab.cycle.a.pct2")? || pct2(&cycle[1]) != get("pub.sched.ab.cycle.b.pct2")? {
        return Err("16_7's cycle disagrees with the facts".into());
    }
    let b_back0 = cycle[1][0] * (Ratio::ONE - b.chance(0));
    let b_fwd1 = cycle[1][1] * b.chance(1);
    let a_fwd2 = cycle[0][2] * a.chance(2);
    let a_back1 = cycle[0][1] * (Ratio::ONE - a.chance(1));
    if b.chance(0) >= half || b.chance(1) <= half {
        return Err("16_7 says the bad coin knocks most back and the good coin carries most forward".into());
    }
    if cycle[0][2].pct(0) != "69" {
        return Err("16_7 says each B round leaves about sixty-nine percent on corner two".into());
    }
    if a.chance(2) >= half || a.chance(2) < Ratio::new(45, 100) || a_back1 >= Ratio::new(5, 100) {
        return Err("16_7 says A sends just under half forward, and a few step back".into());
    }
    if a_fwd2 + a_back1 != cycle[1][0] || cycle[1][0] >= Ratio::new(38, 100) {
        return Err("16_7 says almost thirty-eight percent wait on corner 0".into());
    }
    let refill = get("pub.sched.ab.refill.pct2")?;
    let parts: Vec<&str> = refill.split(',').collect();
    let a_pct = a.chance(2).exact_pct()?;
    let a_lose = (Ratio::ONE - a.chance(1)).exact_pct()?;
    let (on2, on1) = (cycle[0][2].pct(2), cycle[0][1].pct(2));
    if parts.len() != 3
        || (pct_value(&a_pct)? * pct_value(&on2)?).pct(2) != parts[0]
        || (pct_value(&a_lose)? * pct_value(&on1)?).pct(2) != parts[1]
        || pct_value(parts[0])? + pct_value(parts[1])? != pct_value(parts[2])?
        || cycle[1][0].pct(2) != parts[2]
    {
        return Err("16_7's printed products and sum must agree as printed".into());
    }
    let refill_tex = [
        format!(r"{a_pct}\% \times {on2}\% \approx {}\%", parts[0]),
        format!(r"{a_lose}\% \times {on1}\% \approx {}\%", parts[1]),
        format!(r"{}\% + {}\% = {}\%", parts[0], parts[1], parts[2]),
    ];
    // 16_8: averaging the two games' chances gives the coin-toss mix's
    // coins, and so its average; alternation is not that
    let averaged = [0, 1, 2].map(|i| (a.chance(i) + b.chance(i)) / Ratio::int(2));
    if (0..3).any(|i| averaged[i] != mix.chance(i)) {
        return Err("16_8 says the averaged chances are exactly the coin-toss mix's coins".into());
    }
    let mix_coins = [
        mix.chance(0).exact_pct()?,
        mix.chance(1).exact_pct()?,
        mix.chance(2).exact_pct()?,
    ];
    if mix_coins[..2].join(",") != get("pub.mix.coins.pct")? || mix_coins[1] != mix_coins[2] {
        return Err("16_8 says twenty-nine and a half percent and sixty-two percent".into());
    }
    let mix_mean = mix.mean();
    if mix_mean.cents(2) != get("pub.mix.mean.cents")? || mix_mean <= Ratio::new(15, 1000) || mix_mean == ab_mean {
        return Err("16_8 says the shortcut predicts just over one and a half cents, and is wrong".into());
    }
    let shortcut = [
        AVERAGING.to_owned(),
        format!("{}: {MIX_COINS}", mix_coins.map(|p| pct_words(&p)).join(", ")),
        format!("{PREDICTS} {} {CENTS_A_ROUND}", mix_mean.cents(2)),
    ];
    // 16_9: two and two from B's settled crowd: every B round below the
    // line, the last pair drawn at the settled cycle's percentages
    let aabb_played = Played {
        states: model::played(&aabb, start, PLAYED_ROUNDS + 1),
        b: b_rounds(AABB, PLAYED_ROUNDS),
    };
    if aabb_played.b.len() != ab_played.b.len() || aabb_played.b.iter().any(|&k| aabb_played.states[k][0] >= line_f) {
        return Err("16_9 says both B rounds stay below the line".into());
    }
    let aabb_cycle = model::cycle(&aabb);
    let pair = [aabb_cycle[2][0], aabb_cycle[3][0]];
    if csv(pair.map(|x| x.pct(1))) != get("pub.sched.aabb.b.at0.pct")?
        || pair.map(|x| x.pct(0)) != ["36".to_owned(), "33".to_owned()]
        || pair.iter().any(|&x| x >= line)
    {
        return Err("16_9 says about thirty-six and about thirty-three, both below the line".into());
    }
    let shown = &aabb_played.b[aabb_played.b.len() - 2..];
    if (0..2).any(|k| pct1(aabb_played.states[shown[k]][0]) != pair[k].pct(1)) {
        return Err("16_9's last two dots carry the settled pair's percentages".into());
    }
    let (aabb_mean, aabb_back) = periodic(&aabb);
    if !aabb_back
        || aabb_mean.cents(2) != get("pub.sched.aabb.cents")?
        || !near(aabb_mean, Ratio::new(15, 1000), Ratio::new(1, 1000))
        || aabb_mean >= mix_mean
    {
        return Err("16_9 says two and two wins about one and a half cents, a little less than the mix".into());
    }
    // 16_10: per round played; the coin-toss mix's B rounds meet its own
    // corner-0 share, since the coin that picks B ignores the money
    let table = [
        [
            RANDOM_ROW.to_owned(),
            pct_words(&mix.shares()[0].pct(1)),
            mix_mean.cents(2),
        ],
        [spaced(AB), pct_words(&ab_meets), screen(&ab_mean.cents(2))],
        [
            spaced(AABB),
            pair.map(|x| pct_words(&x.pct(1))).join(", "),
            aabb_mean.cents(2),
        ],
    ];
    if mix.shares()[0] >= line {
        return Err("16_10 says the coin-toss mix's B rounds meet less than the line".into());
    }
    // 16_11: the ordinary coins under each schedule
    let plain = Coins {
        a: ORDINARY[0] * DEN / 100,
        bad: ORDINARY[1] * DEN / 100,
        good: ORDINARY[1] * DEN / 100,
    };
    let plain_means = [
        plain.mix().mean(),
        periodic(&schedule(AB, &plain)).0,
        periodic(&schedule(AABB, &plain)).0,
    ];
    if csv(plain_means.map(|x| x.cents(2))) != get("ordinary.sched.cents")?
        || plain_means.iter().any(|&x| x != Ratio::new(-3, 100))
    {
        return Err("16_11 says each schedule of the ordinary coins loses exactly three cents".into());
    }
    if csv(ORDINARY) != get("ordinary.coins.pct")? {
        return Err("16_11 says forty-nine and forty-eight percent".into());
    }
    // 16_12, 16_13: the curve, exact at every half percent
    let curve: Vec<(f64, f64)> = (0..=200i32)
        .map(|k| {
            let m = PUBLISHED.mix_share(i64::from(k), 200).mean();
            (f64::from(k) / 2.0, 100.0 * m.to_f64())
        })
        .collect();
    let at = |num: i64, den: i64| PUBLISHED.mix_share(num, den).mean();
    if at(0, 1) != b.mean() || at(1, 1) != a.mean() || at(1, 2) != mix_mean {
        return Err("16_12 says the curve runs from B alone through the coin-toss mix to A alone".into());
    }
    let point_means = SHARES.map(|(num, den)| at(num, den));
    if csv(SHARES.map(|(num, den)| Ratio::new(num.into(), den.into()))) != get("pub.share.points")?
        || point_means[0].cents(2) != get("pub.share.1_20.cents")?
        || point_means[1].cents(2) != get("pub.share.9_10.cents")?
        || !near(point_means[0], Ratio::new(-1, 400), Ratio::new(1, 2000))
        || !near(point_means[1], Ratio::new(-1, 300), Ratio::new(1, 2000))
    {
        return Err("16_12 says it loses about a quarter, then about a third of a cent".into());
    }
    let (lo, hi) = (
        model::crossing(&PUBLISHED, 0.0, 0.5),
        model::crossing(&PUBLISHED, 0.5, 1.0),
    );
    let fine = [format!("{:.2}", 100.0 * lo), format!("{:.2}", 100.0 * hi)];
    let whole = [format!("{:.0}", 100.0 * lo), format!("{:.0}", 100.0 * hi)];
    if fine.join(",") != get("pub.share.crossings.fine")? || whole.join(",") != get("pub.share.crossings.whole")? {
        return Err("16_13 and 16_14 say about seven and about eighty-five, 7.03 and 84.71 percent".into());
    }
    // the products decide: the mean's sign is the forward minus backward
    // product's at every point of the curve, and it changes exactly twice
    let mut changes = 0;
    for k in 0..=200i64 {
        let m = PUBLISHED.mix_share(k, 200);
        let (pf, pb) = m.products();
        if m.mean().signum() != (pf - pb).signum() {
            return Err(format!("16_13 says the products decide, but not at {k}/200"));
        }
        if k > 0 && m.mean().signum() != PUBLISHED.mix_share(k - 1, 200).mean().signum() {
            changes += 1;
        }
    }
    if changes != 2 {
        return Err("16_13 says the curve rises above zero and comes back down once".into());
    }
    // 16_14: the edges of the 0.1 % grid, and the whole percentages
    let edge = EDGE_SHARES.map(|(num, den)| at(num, den));
    let edge_pct = EDGE_SHARES.map(|(num, den)| Ratio::new(num.into(), den.into()).pct(1));
    if csv(edge_pct.clone()) != get("pub.share.edge.pct")?
        || csv(edge.map(|m| m.cents(4))) != get("pub.share.edge.cents")?
        || csv(edge.map(|m| m.signum())) != get("pub.share.edge.sign")?
        || edge.map(|m| m.signum()) != [-1, 1, 1, -1]
    {
        return Err("16_14 says 7.0 % loses, 7.1 % and 84.7 % win, 84.8 % loses".into());
    }
    let whole_means = WHOLE_EDGES.map(|(num, den)| at(num, den));
    if csv(whole_means.map(|m| m.cents(4))) != get("pub.share.whole.cents")?
        || whole_means.iter().any(|m| m.signum() >= 0)
    {
        return Err("16_14 says at exactly seven and at eighty-five percent the mix does not win".into());
    }
    if get("pub.share.low.pct")? != edge_pct[0] || get("pub.share.high.pct")? != edge_pct[2] {
        return Err("16_14's grid edges disagree with the facts".into());
    }
    let edges = [0, 1, 2, 3].map(|k| {
        let wins = edge[k] > Ratio::ZERO;
        let words = format!(
            "{}: {} {}",
            pct_words(&edge_pct[k]),
            screen(&edge[k].cents(4)),
            if wins { WINS } else { LOSES }
        );
        (words, wins)
    });
    let k07 = format!(
        "published coins, per round played: the mix wins strictly between its two true crossings, shown \
         rounded as {} % and {} %; it loses at {} % and wins from {} % to {} %",
        fine[0], fine[1], edge_pct[0], edge_pct[1], edge_pct[2]
    );
    Ok(Numbers {
        badge: badge_text(&f)?,
        mix: chances(&mix)?,
        shares: [mix.shares().map(|w| w.to_f64()), start],
        line: 100.0 * line_f,
        fair: format!("{BREAKS_EVEN} {}", pct_words(&line.pct(1))),
        a_cost: format!("{A_COST} {} {CENT}", screen(&a.mean().cents(2))),
        ab: ab_played,
        aabb: aabb_played,
        ab_verdict: format!("{LOSES}: {} {CENTS_A_ROUND}", screen(&ab_mean.cents(2))),
        first_dot: pct_words(&pct1(dots[0])),
        settles: format!("{SETTLES} {}", pct_words(&ab_meets)),
        cycle: [cycle[0].map(|x| x.to_f64()), cycle[1].map(|x| x.to_f64())],
        flows: [b_back0, b_fwd1, a_fwd2, a_back1].map(width),
        leaves: format!("{LEAVES} {}", pct_words(&on2)),
        refill: refill_tex,
        shortcut,
        actually: format!("{ACTUALLY} {}", screen(&ab_mean.cents(2))),
        aabb_verdict: format!("{WINS}: {} {CENTS_A_ROUND}", aabb_mean.cents(2)),
        aabb_pair: pair.map(|x| pct_words(&x.pct(1))),
        table,
        plain_faces: ORDINARY.map(|p| pct_words(&p.to_string())),
        plain: plain_means.map(|x| screen(&x.cents(2))),
        curve,
        ends: [
            format!("{B_ALONE} {}", screen(&b.mean().cents(2))),
            format!("{A_ALONE} {}", screen(&a.mean().cents(2))),
            mix_mean.cents(2),
        ],
        points: [0, 1].map(|k| {
            let (num, den) = SHARES[k];
            (
                100.0 * num as f64 / den as f64,
                100.0 * point_means[k].to_f64(),
                screen(&point_means[k].cents(2)),
            )
        }),
        crossings: [100.0 * lo, 100.0 * hi],
        whole_ticks: whole.map(|w| format!(r"\approx {w}\%")),
        fine_ticks: fine.map(|w| pct_words(&w)),
        edges,
        k07,
    })
}

// --------------------------------------------------------------- pictures

/// Builders made ready to play beside other animations.
fn prepared(stage: &mut Stage<'_>, moves: Vec<AnimBuilder>) -> fmn::Result<Vec<Box<dyn Animation>>> {
    moves.into_iter().map(|m| stage.prepare(m)).collect()
}

/// The crowd's puddles at `shares` (14's), with the mobjects that draw them,
/// so the scene can let them go before the triangle does.
fn crowd_at(stage: &mut Stage<'_>, shares: [f64; 3]) -> fmn::Result<(Puddles, Vec<Mob>)> {
    let before = stage.scene().mobjects().to_vec();
    let crowd = puddles(stage, shares)?;
    let drawn = stage
        .scene()
        .mobjects()
        .iter()
        .filter(|m| !before.contains(m))
        .copied()
        .collect();
    Ok((crowd, drawn))
}

/// The strip of a schedule: twelve cells, A blue and B in B's color, then
/// "…". Detached.
fn strip(kit: &Kit, stage: &mut Stage<'_>, pattern: &str) -> fmn::Result<Vec<Mob>> {
    let letters: Vec<char> = pattern.chars().collect();
    let mut out = Vec::with_capacity(CELLS + 1);
    for k in 0..CELLS {
        let c = letters[k % letters.len()];
        let color = if c == 'A' { palette::game_a() } else { palette::game_b() };
        out.push(kit.text(stage, &c.to_string(), CELL_SIZE, color, cell_at(k))?);
    }
    let dots = kit.text(
        stage,
        ELLIPSIS,
        CELL_SIZE,
        Palette::dim(),
        v(STRIP_X + PITCH * CELLS as f64, STRIP_Y),
    )?;
    keep_between(stage, dots, PANEL_X, COUNTER_X - 0.1, "the strip")?;
    out.push(dots);
    Ok(out)
}

/// The centre of the strip's cell for round `k` (the strip repeats).
fn cell_at(k: usize) -> Vec3 {
    v(STRIP_X + PITCH * (k % CELLS) as f64, STRIP_Y)
}

/// The frame that steps along the strip, round its first cell. Detached.
fn cursor(stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let at = cell_at(0);
    let r = rect(
        v(at[0] - CURSOR_W / 2.0, at[1] - CURSOR_H / 2.0),
        CURSOR_W,
        CURSOR_H,
        line_style(Palette::hero(), 2.5),
    )?;
    Ok(stage.arena_mut().add(r))
}

/// "round 7 of 24". Detached.
fn counter(kit: &Kit, stage: &mut Stage<'_>, k: usize) -> fmn::Result<Mob> {
    let words = format!("{ROUND_WORD} {k} {OF_WORD} {PLAYED_ROUNDS}");
    let m = kit.text_left(stage, &words, SMALL, Palette::dim(), v(COUNTER_X, STRIP_Y))?;
    keep_between(stage, m, COUNTER_X - 0.01, PANEL_END, "the round counter")?;
    Ok(m)
}

/// A line of the panel's words, left at `x`. Detached.
fn words_at(kit: &Kit, stage: &mut Stage<'_>, words: &str, color: Srgb, x: f64, y: f64) -> fmn::Result<Mob> {
    let m = kit.text_left(stage, words, WORDS_SIZE, color, v(x, y))?;
    keep_between(stage, m, x - 0.01, PANEL_END, "a line of the panel")?;
    Ok(m)
}

/// The chart's axes, ticks and title, then the break-even line's dashes
/// and its three labels ("B breaks even", "B wins" under the line, "B
/// loses" over it). Detached.
fn chart(kit: &Kit, stage: &mut Stage<'_>, n: &Numbers) -> fmn::Result<(Vec<Mob>, Vec<Mob>, [Mob; 3])> {
    let c = &CHART;
    let corner = polyline([v(c.left(), c.top()), v(c.left(), c.bottom()), v(c.right(), c.bottom())])
        .style(line_style(Palette::dim(), 2.0))
        .build();
    let mut axes = vec![stage.arena_mut().add(corner)];
    for y in CHART_TICKS {
        let m = kit.text(stage, &pct_words(&format!("{y:.0}")), SMALL, Palette::dim(), ORIGIN)?;
        stage.move_to(m, v(c.left() - 0.1, c.p(c.x.0, y)[1]), RIGHT);
        axes.push(m);
    }
    let bottom = c.bottom() - TICK_DY;
    let first = kit.text(stage, "1", SMALL, Palette::dim(), v(c.p(1.0, c.y.0)[0], bottom))?;
    let last = kit.text(
        stage,
        &CELLS.to_string(),
        SMALL,
        Palette::dim(),
        v(c.p(12.0, c.y.0)[0], bottom),
    )?;
    let name = kit.text(stage, B_ROUND, SMALL, Palette::dim(), v(c.center[0], bottom))?;
    let title = kit.text_left(stage, CHART_TITLE, SMALL, Palette::dim(), v(c.left(), CHART_TITLE_Y))?;
    keep_between(stage, title, c.left() - 0.01, PANEL_END, "the chart's title")?;
    axes.extend([first, last, name, title]);
    let y = c.p(c.x.0, n.line)[1];
    let dashes = dashed(stage, v(c.left(), y), v(c.right(), y), 0.1, palette::bad(), 2.0);
    let fair = kit.text(stage, &n.fair, SMALL, Palette::ink(), ORIGIN)?;
    stage.move_to(fair, v(c.right() - 0.05, y - NEAR + LIFT), RIGHT);
    let wins = kit.text_left(stage, B_WINS, SMALL, palette::mix(), v(c.left() + 0.1, y - NEAR + LIFT))?;
    let loses = kit.text_left(
        stage,
        B_LOSES,
        SMALL,
        palette::bad(),
        v(c.left() + 0.1, y + NEAR - LIFT),
    )?;
    Ok((axes, dashes, [fair, wins, loses]))
}

/// A dot on the chart for B round `j` (from 1) at corner 0's `share`.
/// Detached.
fn chart_dot(stage: &mut Stage<'_>, j: usize, share: f64, color: Srgb) -> Mob {
    stage.arena_mut().add(
        Circle::new()
            .radius(DOT_R)
            .arc_center(CHART.p(j as f64, 100.0 * share))
            .style(flat(color, 1.0))
            .build(),
    )
}

/// Words just right of a dot, on its height. Detached.
fn dot_words(kit: &Kit, stage: &mut Stage<'_>, dot: Mob, words: &str, color: Srgb) -> fmn::Result<Mob> {
    let c = stage.get_center(dot);
    let m = kit.text_left(stage, words, 16.0, color, v(c[0] + DOT_R + DOT_WORDS, c[1]))?;
    keep_between(stage, m, c[0], PANEL_END, "a dot's words")?;
    Ok(m)
}

/// The frame on the strip and the round counter beside it.
struct Playhead {
    cursor: Mob,
    count: Mob,
}

/// Play `rounds` (round indices) of a schedule `run`: for each, the frame
/// goes to its cell and the counter counts; before a B round a dot lands at
/// corner 0's share; then the puddles move to the shares after it. A round
/// takes `rt` seconds. Returns the dots.
#[allow(clippy::too_many_arguments)]
fn play_rounds(
    kit: &Kit,
    stage: &mut Stage<'_>,
    crowd: &Puddles,
    head: &mut Playhead,
    run: &Played,
    rounds: std::ops::Range<usize>,
    color: Srgb,
    rt: f64,
) -> fmn::Result<Vec<Mob>> {
    let mut dots = Vec::new();
    for k in rounds {
        let next = counter(kit, stage, k + 1)?;
        let mut anims = swap(stage, &[head.count], &[next], 0.35 * rt)?;
        anims.push(stage.prepare(fly(head.cursor, cell_at(k), 0.35 * rt)?)?);
        if let Some(j) = run.b.iter().position(|&r| r == k) {
            let dot = chart_dot(stage, j + 1, run.states[k][0], color);
            let a = fade_in(stage.arena_mut(), dot, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.35 * rt))?);
            dots.push(dot);
        }
        stage.play_prepared(anims)?;
        head.count = next;
        together(stage, crowd.to(run.states[k + 1], 0.65 * rt)?)?;
    }
    Ok(dots)
}

/// Seconds a round may take so that `left` more rounds end by scene time
/// `at`, between [`FAST_S`] and [`ROUND_S`].
fn round_time(stage: &Stage<'_>, at: f64, left: usize) -> f64 {
    ((at - now(stage)) / left.max(1) as f64).clamp(FAST_S, ROUND_S)
}

/// The curve's axes, zero line, ticks and titles. Detached.
fn curve_axes(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let c = &CURVE;
    let corner = polyline([v(c.left(), c.top()), v(c.left(), c.bottom()), v(c.right(), c.bottom())])
        .style(line_style(Palette::dim(), 2.0))
        .build();
    let zero = polyline([c.p(c.x.0, 0.0), c.p(c.x.1, 0.0)])
        .style(line_style(Palette::dim(), 1.5))
        .build();
    let mut out = vec![stage.arena_mut().add(corner), stage.arena_mut().add(zero)];
    for y in CURVE_TICKS {
        let words = if y > 0.0 {
            format!("+{y:.0}")
        } else {
            screen(&format!("{y:.0}"))
        };
        let m = kit.text(stage, &words, SMALL, Palette::dim(), ORIGIN)?;
        stage.move_to(m, v(c.left() - 0.1, c.p(c.x.0, y)[1]), RIGHT);
        out.push(m);
    }
    for x in SHARE_TICKS {
        let at = c.p(x, c.y.0);
        out.push(kit.text(
            stage,
            &pct_words(&format!("{x:.0}")),
            SMALL,
            Palette::dim(),
            v(at[0], at[1] - TICK_DY),
        )?);
    }
    out.push(kit.text_left(stage, CENTS_A_ROUND, SMALL, Palette::dim(), v(c.left(), CURVE_TITLE_Y))?);
    out.push(kit.text(stage, SHARE_AXIS, SMALL, Palette::dim(), v(c.center[0], AXIS_Y))?);
    Ok(out)
}

/// A point on the curve at (`share` %, `cents`). Detached.
fn curve_point(stage: &mut Stage<'_>, share: f64, cents: f64, color: Srgb) -> Mob {
    stage.arena_mut().add(
        Circle::new()
            .radius(POINT_R)
            .arc_center(CURVE.p(share, cents))
            .style(flat(color, 1.0))
            .build(),
    )
}

/// How words stand beside a point of the curve: their `edge` set `d` from
/// the point, as text or (`tex`) as TeX.
struct Beside {
    d: (f64, f64),
    edge: Vec3,
    tex: bool,
}

/// Words beside the curve's point `at` (share %, cents). Detached.
fn curve_words(
    kit: &Kit,
    stage: &mut Stage<'_>,
    words: &str,
    color: Srgb,
    at: (f64, f64),
    place: &Beside,
) -> fmn::Result<Mob> {
    let m = if place.tex {
        kit.tex(stage, words, TEX_SIZE, color, &[], ORIGIN)?
    } else {
        kit.text(stage, words, SMALL, color, ORIGIN)?
    };
    let p = CURVE.p(at.0, at.1);
    stage.move_to(m, v(p[0] + place.d.0, p[1] + place.d.1), place.edge);
    keep_between(
        stage,
        m,
        -FRAME_X_RADIUS + 0.3,
        FRAME_X_RADIUS - 0.3,
        "a label of the curve",
    )?;
    Ok(m)
}

/// Under a crossing: right of the first, left of the second.
fn under_crossing(k: usize, tex: bool) -> Beside {
    if k == 0 {
        Beside {
            d: (END_DX, -NEAR),
            edge: LEFT,
            tex,
        }
    } else {
        Beside {
            d: (-END_DX, -NEAR),
            edge: RIGHT,
            tex,
        }
    }
}

impl SceneConstruct for WhenSwitchingFails {
    fn name(&self) -> &str {
        "16_when_switching_fails"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let n = numbers().map_err(fail)?;
        let (ink, dim, hero) = (Palette::ink(), Palette::dim(), Palette::hero());

        // 16_1: the mix as it won, its settled crowd; the random line
        let line = say(&kit, stage, "16_1")?;
        let tag = full_badge(&kit, stage, &n.badge, PANEL_END)?;
        let t = triangle(&kit, stage, Which::B)?;
        let mut title = mix_title(&kit, stage)?;
        let labels = mix_labels(&kit, stage, &n.mix)?;
        let mut arriving = vec![tag[0], tag[1], title];
        arriving.extend(t.discs);
        arriving.extend(t.names);
        arriving.extend(t.forward.iter().flatten());
        arriving.extend(t.back.iter().flatten());
        arriving.extend(&labels);
        fade_in_all(stage, &arriving, 0.8)?;
        let (crowd, drawn) = crowd_at(stage, [0.0; 3])?;
        together(stage, crowd.to(n.shares[0], 0.8)?)?;
        let mut texts = labels.clone();
        texts.extend(t.names);
        texts.push(title);
        keep_apart(stage, &texts, 0.08, "the mix's triangle")?;
        line.wait_for(stage, AT_FAIR_COIN.1)?;
        let toss = stage.arena_mut().add(
            Circle::new()
                .radius(COIN_R)
                .arc_center(v(COIN_X, RANDOM_Y))
                .style(flat(palette::mix(), 1.0))
                .build(),
        );
        let random = words_at(&kit, stage, RANDOM_LINE, ink, WORDS_X, RANDOM_Y)?;
        fade_in_all(stage, &[toss, random], 0.5)?;
        line.wait_for(stage, AT_MATTER.1)?;
        let pointer = frame_around(stage, &[random], POINT_PAD, hero)?;
        point_at(stage, pointer, &[toss], 0.8)?;

        // 16_2: the simplest fixed order, strict alternation
        let line = say(&kit, stage, "16_2")?;
        line.wait_for(stage, AT_FIXED.1)?;
        let mut cells = strip(&kit, stage, AB)?;
        let mut name = words_at(&kit, stage, AB_NAME, ink, PANEL_X, NAME_Y)?;
        let ab_title = kit.text(stage, &spaced(AB), 24.0, ink, v(B_AT[0], B_AT[1] - TITLE_DY))?;
        let mut new = cells.clone();
        new.extend([name, ab_title]);
        let mut old = labels.clone();
        old.push(title);
        let anims = swap(stage, &old, &new, 0.7)?;
        stage.play_prepared(anims)?;
        title = ab_title;
        line.wait_for(stage, AT_HALF.1)?;
        let pointer = frame_around(stage, &cells, POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_ONLY_ORDER.1)?;
        let pointer = frame_around(stage, &[name], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.7)?;

        // 16_3: does it win? (a prediction)
        let line = say(&kit, stage, "16_3")?;
        line.wait_for(stage, AT_WIN.1)?;
        let ask = words_at(
            &kit,
            stage,
            &format!("does {} win?", spaced(AB)),
            ink,
            VERDICT_X,
            NAME_Y,
        )?;
        fade_in_all(stage, &[ask], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 16_4: it loses; the order alone did it
        let line = say(&kit, stage, "16_4")?;
        line.wait_for(stage, AT_LOSES.1)?;
        let mut verdict = words_at(&kit, stage, &n.ab_verdict, palette::bad(), VERDICT_X, NAME_Y)?;
        let anims = swap(stage, &[ask, q], &[verdict], 0.5)?;
        stage.play_prepared(anims)?;
        keep_apart(stage, &[name, verdict], 0.1, "the schedule's verdict")?;
        line.wait_for(stage, AT_ORDER_ALONE.1)?;
        let pointer = frame_around(stage, &cells, POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_WATCH.1)?;
        ring_corners(stage, &t.discs, &[], 0.8)?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 16_5: A's cost is fixed; B's rounds and the dial decide
        let line = say(&kit, stage, "16_5")?;
        line.wait_for(stage, AT_COST.1)?;
        let cost = words_at(&kit, stage, &n.a_cost, palette::game_a(), PANEL_X, COST_Y)?;
        fade_in_all(stage, &[cost], 0.5)?;
        line.wait_for(stage, AT_RIDES.1)?;
        let bs: Vec<Mob> = cells.iter().skip(1).step_by(2).take(CELLS / 2).copied().collect();
        let pointer = frame_around(stage, &bs, POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_DIAL.1)?;
        let (axes, dashes, [fair, wins, loses]) = chart(&kit, stage, &n)?;
        fade_in_all(stage, &axes, 0.7)?;
        line.wait_for(stage, AT_BREAKS.1)?;
        let mut even = dashes.clone();
        even.push(fair);
        fade_in_all(stage, &even, 0.6)?;
        line.wait_for(stage, AT_BELOW.1)?;
        fade_in_all(stage, &[wins], 0.4)?;
        line.wait_for(stage, AT_ABOVE.1)?;
        fade_in_all(stage, &[loses], 0.4)?;
        let mut panel = vec![random, name, verdict, cost, fair, wins, loses];
        panel.extend(&axes[1..]);
        keep_apart(stage, &panel, 0.06, "the chart's words")?;

        // 16_6: alternation played round by round from B's settled crowd
        let line = say(&kit, stage, "16_6")?;
        line.wait_for(stage, AT_START.1)?;
        let note = kit.text(stage, FROM_B, SMALL, palette::player(), v(B_AT[0], UNDER_Y + 0.05))?;
        let mut anims = prepared(stage, crowd.to(n.shares[1], MOVE_S)?)?;
        anims.extend(swap(stage, &[], &[note], MOVE_S)?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_ALTERNATE.1)?;
        let mut head = Playhead {
            cursor: cursor(stage)?,
            count: counter(&kit, stage, 0)?,
        };
        fade_in_all(stage, &[head.cursor, head.count], 0.3)?;
        let teal = palette::player();
        let mut ab_dots = play_rounds(&kit, stage, &crowd, &mut head, &n.ab, 0..2, teal, ROUND_S)?;
        line.wait_for(stage, AT_FIRST_B.1)?;
        let first_words = dot_words(&kit, stage, ab_dots[0], &n.first_dot, teal)?;
        fade_in_all(stage, &[first_words], 0.4)?;
        line.wait_for(stage, AT_COMES_BACK.1)?;
        let rt = round_time(stage, line.at(AT_SETTLE.1)?, PLAYED_ROUNDS - 2);
        ab_dots.extend(play_rounds(
            &kit,
            stage,
            &crowd,
            &mut head,
            &n.ab,
            2..PLAYED_ROUNDS,
            teal,
            rt,
        )?);
        line.wait_for(stage, AT_SETTLE.1)?;
        let settles = kit.text(stage, &n.settles, SMALL, teal, ORIGIN)?;
        let line_y = CHART.p(CHART.x.0, n.line)[1];
        stage.move_to(settles, v(CHART.right() - 0.05, line_y + 1.3 * NEAR), RIGHT);
        fade_in_all(stage, &[settles], 0.4)?;
        line.wait_for(stage, AT_EVERY_LOSES.1)?;
        let above: Vec<Mob> = ab_dots
            .iter()
            .zip(&n.ab.b)
            .filter(|&(_, &k)| 100.0 * n.ab.states[k][0] > n.line)
            .map(|(&m, _)| m)
            .collect();
        let pointer = frame_around(stage, &above, POINT_PAD, hero)?;
        point_at(stage, pointer, &above, 0.9)?;
        let mut chart_words = vec![first_words, settles, fair, wins, loses, head.count];
        chart_words.extend(&axes[1..]);
        keep_apart(stage, &chart_words, 0.05, "the dots' words")?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 16_7: one settled cycle, its streams and sums
        let line = say(&kit, stage, "16_7")?;
        line.wait_for(stage, AT_WHY.1)?;
        let mut moves = crowd.to(n.cycle[1], MOVE_S)?;
        moves.push(fly(head.cursor, cell_at(1), 0.5)?);
        let mut anims = prepared(stage, moves)?;
        anims.extend(swap(stage, &[note], &[], 0.5)?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_CYCLE.1)?;
        let pointer = frame_around(stage, &[cells[1]], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.6)?;
        line.wait_for(stage, AT_KNOCKS.1)?;
        let knock = stream(stage, 2, false, n.flows[0], palette::bad(), STREAM_O);
        draw_bands(stage, &[knock], 1.0)?;
        line.wait_for(stage, AT_CARRIES.1)?;
        let carry = stream(stage, 1, true, n.flows[1], palette::good(), STREAM_O);
        draw_bands(stage, &[carry], 1.0)?;
        line.wait_for(stage, AT_LEAVES.1)?;
        let leaves = kit.text_left(stage, &n.leaves, WORDS_SIZE, teal, v(UNDER_X, UNDER_Y))?;
        keep_between(stage, leaves, UNDER_X - 0.01, FRAME_X_RADIUS - 0.3, "corner 2's share")?;
        let mut anims = prepared(stage, crowd.to(n.cycle[0], CHANGE_S)?)?;
        anims.extend(swap(stage, &[], &[leaves], 0.5)?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_A_PLAYS.1)?;
        let mut anims = vec![stage.prepare(fly(head.cursor, cell_at(2), 0.5)?)?];
        anims.extend(swap(stage, &[knock, carry], &[], 0.5)?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_SENDS.1)?;
        let send = stream(stage, 2, true, n.flows[2], palette::game_a(), STREAM_O);
        draw_bands(stage, &[send], 1.0)?;
        let sum2 = kit.tex_left(stage, &n.refill[0], SUM_SIZE, ink, &[], v(PANEL_X, LOW_YS[0]))?;
        keep_between(stage, sum2, PANEL_X - 0.01, PANEL_END, "a refill sum")?;
        fade_in_all(stage, &[sum2], 0.5)?;
        line.wait_for(stage, AT_FEW.1)?;
        let back = stream(stage, 0, false, n.flows[3], dim, STREAM_O);
        draw_bands(stage, &[back], 0.8)?;
        let sum1 = kit.tex_left(stage, &n.refill[1], SUM_SIZE, ink, &[], v(PANEL_X, LOW_YS[1]))?;
        keep_between(stage, sum1, PANEL_X - 0.01, PANEL_END, "a refill sum")?;
        fade_in_all(stage, &[sum1], 0.5)?;
        line.wait_for(stage, AT_ALMOST.1)?;
        let total = kit.tex_left(stage, &n.refill[2], SUM_SIZE, teal, &[], v(PANEL_X, LOW_YS[2]))?;
        let total_right = stage.get_bounding_box_point(total, RIGHT)[0];
        let unit = kit.text_left(stage, WHEN_B_PLAYS, SMALL, dim, v(total_right + 0.2, LOW_YS[2]))?;
        keep_between(stage, unit, total_right, PANEL_END, "the refill's unit")?;
        let mut moves = crowd.to(n.cycle[1], CHANGE_S)?;
        moves.push(fly(head.cursor, cell_at(3), 0.5)?);
        let mut anims = prepared(stage, moves)?;
        anims.extend(swap(stage, &[], &[total, unit], 0.6)?);
        stage.play_prepared(anims)?;
        let late: Vec<Mob> = ab_dots[ab_dots.len() / 2..].to_vec();
        let pointer = frame_around(stage, &[total, unit], POINT_PAD, hero)?;
        point_at(stage, pointer, &late, 0.9)?;
        let mut sums = vec![sum2, sum1, total, unit, leaves];
        sums.extend(&axes[1..]);
        keep_apart(stage, &sums, 0.05, "the cycle's sums")?;
        kit.hold(stage)?;
        pause(stage, INSPECT_S)?;

        // 16_8: the tempting shortcut, averaged chances; struck
        let line = say(&kit, stage, "16_8")?;
        line.wait_for(stage, AT_SHORTCUT.1)?;
        fade_out_all(stage, &[send, back, sum2, sum1, total, unit, leaves], 0.5)?;
        let averaging = words_at(&kit, stage, &n.shortcut[0], ink, PANEL_X, LOW_YS[0])?;
        fade_in_all(stage, &[averaging], 0.5)?;
        line.wait_for(stage, AT_GIVES.1)?;
        let coins = words_at(&kit, stage, &n.shortcut[1], palette::mix(), PANEL_X, LOW_YS[1])?;
        fade_in_all(stage, &[coins], 0.5)?;
        line.wait_for(stage, AT_PREDICTS.1)?;
        let predicts = words_at(&kit, stage, &n.shortcut[2], palette::mix(), PANEL_X, LOW_YS[2])?;
        fade_in_all(stage, &[predicts], 0.5)?;
        line.wait_for(stage, AT_FIXED_NEXT.1)?;
        let struck = strike(&kit, stage, predicts);
        draw_bands(stage, &[struck], 0.5)?;
        let right = stage.get_bounding_box_point(predicts, RIGHT)[0];
        let actually = words_at(&kit, stage, &n.actually, palette::bad(), right + 0.3, LOW_YS[2])?;
        fade_in_all(stage, &[actually], 0.5)?;
        line.wait_for(stage, AT_PREVIOUS_ROUNDS.1)?;
        let pointer = frame_around(stage, &cells, POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        keep_apart(stage, &[averaging, coins, predicts, actually], 0.05, "the shortcut")?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 16_9: two and two from B's settled crowd again
        let line = say(&kit, stage, "16_9")?;
        line.wait_for(stage, AT_TWO_TWO.1)?;
        let dims = ab_dots
            .iter()
            .map(|&d| fade_to(d, 0.3, 0.5))
            .collect::<fmn::Result<Vec<_>>>()?;
        together(stage, dims)?;
        let new_cells = strip(&kit, stage, AABB)?;
        let new_name = words_at(&kit, stage, AABB_NAME, ink, PANEL_X, NAME_Y)?;
        let new_title = kit.text(stage, &spaced(AABB), 24.0, ink, v(B_AT[0], B_AT[1] - TITLE_DY))?;
        let note = kit.text(stage, FROM_B, SMALL, teal, v(B_AT[0], UNDER_Y + 0.05))?;
        let next = counter(&kit, stage, 0)?;
        let mut old = cells.clone();
        old.extend([name, verdict, title, head.count]);
        old.extend([averaging, coins, predicts, struck, actually, settles, first_words]);
        let mut new = new_cells.clone();
        new.extend([new_name, new_title, note, next]);
        let mut moves = crowd.to(n.shares[1], 0.7)?;
        moves.push(fly(head.cursor, cell_at(0), 0.7)?);
        let mut anims = prepared(stage, moves)?;
        anims.extend(swap(stage, &old, &new, 0.7)?);
        stage.play_prepared(anims)?;
        (cells, name, title, head.count) = (new_cells, new_name, new_title, next);
        line.wait_for(stage, AT_SPREAD.1)?;
        let rt = round_time(stage, line.at(AT_36.1)?, PLAYED_ROUNDS);
        let gold = palette::mix();
        let aabb_dots = play_rounds(&kit, stage, &crowd, &mut head, &n.aabb, 0..PLAYED_ROUNDS, gold, rt)?;
        let (d11, d12) = (aabb_dots[aabb_dots.len() - 2], aabb_dots[aabb_dots.len() - 1]);
        line.wait_for(stage, AT_36.1)?;
        let w36 = dot_words(&kit, stage, d11, &n.aabb_pair[0], gold)?;
        fade_in_all(stage, &[w36], 0.4)?;
        line.wait_for(stage, AT_33.1)?;
        let w33 = dot_words(&kit, stage, d12, &n.aabb_pair[1], gold)?;
        fade_in_all(stage, &[w33], 0.4)?;
        line.wait_for(stage, AT_TWO_WINS.1)?;
        verdict = words_at(&kit, stage, &n.aabb_verdict, gold, VERDICT_X, NAME_Y)?;
        fade_in_all(stage, &[verdict], 0.5)?;
        let mut aabb_words = vec![w36, w33, fair, wins, loses, name, verdict, head.count];
        aabb_words.extend(&axes[1..]);
        keep_apart(stage, &aabb_words, 0.05, "two and two's words")?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 16_10: the three side by side, per round played
        let line = say(&kit, stage, "16_10")?;
        line.wait_for(stage, AT_SIDE.1)?;
        let mut leaving = cells.clone();
        leaving.extend([
            toss,
            random,
            name,
            verdict,
            cost,
            head.cursor,
            head.count,
            note,
            w36,
            w33,
        ]);
        leaving.extend(&axes);
        leaving.extend(&even);
        leaving.extend([wins, loses]);
        leaving.extend(&ab_dots);
        leaving.extend(&aabb_dots);
        fade_out_all(stage, &leaving, 0.6)?;
        let heads = [
            kit.text_left(stage, SCHEDULE_HEAD, HEAD_SIZE, dim, v(PANEL_X, HEAD_Y))?,
            kit.text_left(stage, CHART_TITLE, HEAD_SIZE, dim, v(MEETS_X, HEAD_Y))?,
            kit.text_left(stage, CENTS_A_ROUND, HEAD_SIZE, dim, v(CENTS_X, HEAD_Y))?,
        ];
        keep_between(stage, heads[2], CENTS_X - 0.01, PANEL_END, "the table's head")?;
        let per_round = kit.text_left(stage, PER_ROUND, SMALL, dim, v(PANEL_X, NOTE_Y))?;
        keep_between(stage, per_round, PANEL_X - 0.01, PANEL_END, "the table's note")?;
        fade_in_all(stage, &[heads[0], heads[1], heads[2], per_round], 0.5)?;
        let colors = [palette::mix(), palette::bad(), palette::mix()];
        let mut rows = Vec::with_capacity(3);
        for (k, at) in [AT_ROW_MIX, AT_ROW_AB, AT_ROW_AABB].iter().enumerate() {
            line.wait_for(stage, at.1)?;
            let row = [
                kit.text_left(stage, &n.table[k][0], WORDS_SIZE, ink, v(PANEL_X, ROW_YS[k]))?,
                kit.text_left(stage, &n.table[k][1], WORDS_SIZE, teal, v(MEETS_X, ROW_YS[k]))?,
                kit.text_left(stage, &n.table[k][2], WORDS_SIZE, colors[k], v(CENTS_X, ROW_YS[k]))?,
            ];
            keep_between(stage, row[0], PANEL_X - 0.01, MEETS_X - 0.1, "a schedule's name")?;
            keep_between(stage, row[1], MEETS_X - 0.01, CENTS_X - 0.1, "a schedule's share")?;
            keep_between(stage, row[2], CENTS_X - 0.01, PANEL_END, "a schedule's cents")?;
            fade_in_all(stage, &row, 0.5)?;
            rows.push(row);
        }
        line.wait_for(stage, AT_STANDS.1)?;
        let middle: Vec<Mob> = rows.iter().map(|r| r[1]).chain([heads[1]]).collect();
        let pointer = frame_around(stage, &middle, POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.9)?;
        let mut table: Vec<Mob> = rows.iter().flatten().copied().collect();
        table.extend(heads);
        table.push(per_round);
        keep_apart(stage, &table, 0.05, "the schedules' table")?;
        kit.hold(stage)?;
        pause(stage, INSPECT_S)?;

        // 16_11: the control: the triangle goes, the ordinary coins under
        // every schedule
        let line = say(&kit, stage, "16_11")?;
        line.wait_for(stage, AT_CONTROL.1)?;
        let mut gone = t.parts();
        gone.retain(|&m| m != t.title);
        gone.push(title);
        gone.extend(&drawn);
        let anims = swap(stage, &gone, &[], 0.6)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_ORDINARY.1)?;
        let plain_head = kit.text_left(stage, ORDINARY_HEAD, HEAD_SIZE, dim, v(PLAIN_X, HEAD_Y))?;
        let plain_coins = (0..2)
            .map(|k| coin(&kit, stage, PLAIN_AT[k], PLAIN_R, dim, &n.plain_faces[k]))
            .collect::<fmn::Result<Vec<[Mob; 2]>>>()?;
        let mut arriving = vec![plain_head];
        arriving.extend(plain_coins.iter().flatten());
        fade_in_all(stage, &arriving, 0.6)?;
        let mut column = Vec::with_capacity(4);
        for (k, at) in [AT_TOSS, AT_ALTERNATE_PLAIN, AT_TWO_TWO_PLAIN].iter().enumerate() {
            line.wait_for(stage, at.1)?;
            let m = kit.text_left(stage, &n.plain[k], WORDS_SIZE, palette::bad(), v(PLAIN_X, ROW_YS[k]))?;
            fade_in_all(stage, &[m], 0.4)?;
            column.push(m);
        }
        line.wait_for(stage, AT_NEVER_ORDER.1)?;
        column.push(plain_head);
        let pointer = frame_around(stage, &column, POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.9)?;
        let mut checked = table.clone();
        checked.extend(&column);
        checked.extend(plain_coins.iter().map(|c| c[0]));
        keep_apart(stage, &checked, 0.05, "the control's column")?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 16_12: the share matters too: the curve's axes, its ends, two
        // losing shares
        let line = say(&kit, stage, "16_12")?;
        line.wait_for(stage, AT_SHARE.1)?;
        let mut leaving = table.clone();
        leaving.extend(&column);
        leaving.extend(plain_coins.iter().flatten());
        fade_out_all(stage, &leaving, 0.6)?;
        let axes = curve_axes(&kit, stage)?;
        let (b_end, a_end, mid) = (n.curve[0], n.curve[n.curve.len() - 1], n.curve[100]);
        let ends = [
            curve_point(stage, b_end.0, b_end.1, palette::game_b()),
            curve_point(stage, a_end.0, a_end.1, palette::game_a()),
            curve_point(stage, mid.0, mid.1, palette::mix()),
        ];
        let over = Beside {
            d: (0.0, NEAR),
            edge: DOWN,
            tex: false,
        };
        let end_words = [
            curve_words(
                &kit,
                stage,
                &n.ends[0],
                palette::game_b(),
                b_end,
                &under_crossing(0, false),
            )?,
            curve_words(
                &kit,
                stage,
                &n.ends[1],
                palette::game_a(),
                a_end,
                &under_crossing(1, false),
            )?,
            curve_words(&kit, stage, &n.ends[2], palette::mix(), mid, &over)?,
        ];
        let mut arriving = axes.clone();
        arriving.extend(ends);
        arriving.extend(end_words);
        fade_in_all(stage, &arriving, 0.8)?;
        let mut points = Vec::with_capacity(2);
        let mut point_words = Vec::with_capacity(2);
        for (k, at) in [AT_ONE_IN_20, AT_NINE_IN_10].iter().enumerate() {
            line.wait_for(stage, at.1)?;
            let (s, c) = (n.points[k].0, n.points[k].1);
            let p = curve_point(stage, s, c, palette::bad());
            let mut place = under_crossing(k, false);
            place.d.1 *= 1.3;
            let w = curve_words(&kit, stage, &n.points[k].2, palette::bad(), (s, c), &place)?;
            fade_in_all(stage, &[p, w], 0.5)?;
            points.push(p);
            point_words.push(w);
        }

        // 16_13: the curve at every half percent; the crossings, the band
        let line = say(&kit, stage, "16_13")?;
        line.wait_for(stage, AT_COMPUTE.1)?;
        let path: Vec<Vec3> = n.curve.iter().map(|&(s, c)| CURVE.p(s, c)).collect();
        let curve = stage
            .arena_mut()
            .add(polyline(path).style(line_style(palette::mix(), 4.0)).build());
        draw_bands(stage, &[curve], CURVE_S)?;
        let mut ticks = Vec::with_capacity(2);
        let mut tick_words = Vec::with_capacity(2);
        for (k, at) in [AT_ABOUT_7, AT_ABOUT_85].iter().enumerate() {
            line.wait_for(stage, at.1)?;
            let s = n.crossings[k];
            let tick = stage.arena_mut().add(
                polyline([CURVE.p(s, -0.12), CURVE.p(s, 0.12)])
                    .style(line_style(ink, 3.0))
                    .build(),
            );
            let w = curve_words(&kit, stage, &n.whole_ticks[k], ink, (s, 0.0), &under_crossing(k, true))?;
            fade_in_all(stage, &[tick, w], 0.5)?;
            ticks.push(tick);
            tick_words.push(w);
        }
        line.wait_for(stage, AT_BETWEEN.1)?;
        let band = stage.arena_mut().add(
            polyline([CURVE.p(n.crossings[0], 0.0), CURVE.p(n.crossings[1], 0.0)])
                .style(
                    Style::default()
                        .fill(palette::mix(), 0.0)
                        .stroke(palette::mix(), BAND_W, 0.5),
                )
                .build(),
        );
        draw_bands(stage, &[band], 1.0)?;
        let middle = (n.crossings[0] + n.crossings[1]) / 2.0;
        let below = Beside {
            d: (0.0, -NEAR),
            edge: UP,
            tex: false,
        };
        let bigger = curve_words(&kit, stage, PRODUCT_BIGGER, palette::mix(), (middle, 0.0), &below)?;
        fade_in_all(stage, &[bigger], 0.5)?;
        let mut curve_texts = axes[2..].to_vec();
        curve_texts.extend(end_words);
        curve_texts.extend(&point_words);
        curve_texts.extend(&tick_words);
        curve_texts.push(bigger);
        keep_apart(stage, &curve_texts, 0.05, "the curve's labels")?;
        kit.hold(stage)?;
        pause(stage, INSPECT_S)?;

        // 16_14: not round numbers: the crossings, the grid's edges, K07
        let line = say(&kit, stage, "16_14")?;
        line.wait_for(stage, AT_FINE.1)?;
        let fine: Vec<Mob> = (0..2)
            .map(|k| {
                let at = (n.crossings[k], 0.0);
                curve_words(&kit, stage, &n.fine_ticks[k], ink, at, &under_crossing(k, false))
            })
            .collect::<fmn::Result<_>>()?;
        let anims = swap(stage, &tick_words, &fine, 0.6)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_STRICT.1)?;
        let mut rows = vec![kit.text_left(stage, EDGES_HEAD, SMALL, dim, v(PANEL_X, EDGES_Y))?];
        for (k, (words, wins)) in n.edges.iter().enumerate() {
            let color = if *wins { palette::mix() } else { palette::bad() };
            rows.push(kit.text_left(stage, words, SMALL, color, v(EDGES_X[k], EDGES_Y))?);
        }
        keep_apart(stage, &rows, 0.1, "the grid's edges")?;
        let mut checked = curve_texts.clone();
        checked.retain(|m| !tick_words.contains(m));
        checked.extend(&fine);
        checked.extend(&rows);
        keep_apart(stage, &checked, 0.05, "the curve's labels and edges")?;
        let card = precisely(&kit, stage, &n.k07)?;
        let mut arriving = rows.clone();
        arriving.push(card);
        fade_in_all(stage, &arriving, 0.5)?;
        let card_up = now(stage);
        line.wait_for(stage, AT_EXACTLY_7.1)?;
        let pointer = frame_around(stage, &rows[1..2], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_85.1)?;
        // The last win and first loss bracket the crossing below 85 percent.
        let pointer = frame_around(stage, &rows[3..5], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        kit.hold(stage)?;
        wait_until(stage, card_up + read_s(&n.k07))?;
        fade_out_all(stage, &[card], 0.4)?;

        // 16_15: no general rule; the curve goes, B's triangle returns with
        // the mix's settled crowd
        let line = say(&kit, stage, "16_15")?;
        line.wait_for(stage, AT_NO_RULE.1)?;
        let mut leaving = axes;
        leaving.extend([curve, band, bigger]);
        leaving.extend(ends);
        leaving.extend(end_words);
        leaving.extend(&points);
        leaving.extend(&point_words);
        leaving.extend(&ticks);
        leaving.extend(&fine);
        leaving.extend(&rows);
        fade_out_all(stage, &leaving, 0.8)?;
        line.wait_for(stage, AT_SOME.1)?;
        let back = triangle(&kit, stage, Which::B)?;
        let mix_name = mix_title(&kit, stage)?;
        let mix_chances = mix_labels(&kit, stage, &n.mix)?;
        let mut parts = back.parts();
        parts.retain(|&m| m != back.title);
        parts.push(mix_name);
        parts.extend(&mix_chances);
        fade_in_all(stage, &parts, 0.8)?;
        let (settled, _) = crowd_at(stage, [0.0; 3])?;
        together(stage, settled.to(n.shares[0], 0.8)?)?;
        line.wait_for(stage, AT_READS.1)?;
        let pointer = frame_around(stage, &[back.discs[0]], POINT_PAD, hero)?;
        point_at(stage, pointer, &[back.discs[0]], 0.9)?;
        kit.hold(stage)?;
        pause(stage, INSPECT_S)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use explainer_kit::{Narrator, Script};
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::super::draw::{BADGE_AT, THINK_S};
    use super::super::three_corners::{CORNER_R, corner};
    use super::*;

    #[test]
    fn the_numbers_are_the_facts() {
        let n = numbers().unwrap();
        assert_eq!(n.fair, "B breaks even: 37.7 %");
        assert_eq!(n.a_cost, "each A round: \u{2212}1.00 cent");
        assert_eq!(n.ab_verdict, "loses: \u{2212}0.67 cents a round");
        assert_eq!(n.first_dot, "30.7 %");
        assert_eq!(n.settles, "settles at 38.0 %");
        assert_eq!(n.leaves, "B leaves on corner 2: 68.95 %");
        assert_eq!(
            n.refill,
            [
                r"49.5\% \times 68.95\% \approx 34.13\%",
                r"50.5\% \times 7.59\% \approx 3.83\%",
                r"34.13\% + 3.83\% = 37.96\%",
            ]
        );
        assert_eq!(
            n.shortcut,
            [
                "averaging A's and B's chances at each corner:",
                "29.5 %, 62 %, 62 %: the coin-toss mix's coins",
                "predicts +1.57 cents a round",
            ]
        );
        assert_eq!(n.actually, "actually: \u{2212}0.67");
        assert_eq!(n.aabb_verdict, "wins: +1.47 cents a round");
        assert_eq!(n.aabb_pair, ["36.0 %", "33.3 %"]);
        assert_eq!(n.table[0], ["random (a fair coin)", "34.5 %", "+1.57"]);
        assert_eq!(n.table[1], ["A B A B \u{2026}", "38.0 %", "\u{2212}0.67"]);
        assert_eq!(n.table[2], ["A A B B \u{2026}", "36.0 %, 33.3 %", "+1.47"]);
        assert_eq!(n.plain_faces, ["49 %", "48 %"]);
        assert_eq!(n.plain, ["\u{2212}3.00", "\u{2212}3.00", "\u{2212}3.00"]);
        assert_eq!(n.ends, ["B alone \u{2212}0.87", "A alone \u{2212}1.00", "+1.57"]);
        assert_eq!(
            [n.points[0].2.as_str(), n.points[1].2.as_str()],
            ["\u{2212}0.23", "\u{2212}0.34"]
        );
        assert_eq!(n.whole_ticks, [r"\approx 7\%", r"\approx 85\%"]);
        assert_eq!(n.fine_ticks, ["7.03 %", "84.71 %"]);
        assert_eq!(
            n.edges.clone().map(|e| e.0),
            [
                "7.0 %: \u{2212}0.0034 loses",
                "7.1 %: +0.0074 wins",
                "84.7 %: +0.0005 wins",
                "84.8 %: \u{2212}0.0058 loses",
            ]
        );
        assert_eq!(n.curve.len(), 201);
        assert_eq!((n.ab.b.len(), n.aabb.b.len()), (12, 12));
        assert_eq!(
            (n.ab.states.len(), n.aabb.states.len()),
            (PLAYED_ROUNDS + 1, PLAYED_ROUNDS + 1)
        );
    }

    /// The schedules are played round by round, and the dots are what the
    /// voice says over the first sixty B rounds of alternation, not only the
    /// twelve drawn: they climb strictly, five below the line and every one
    /// after above it, towards the exact cycle's 37.96 %; two and two's B
    /// rounds never reach the line. Failed controls: averaging the games'
    /// chances at each corner (the shortcut) is the coin-toss mix, whose
    /// average wins where alternation loses; the published coins under ABB
    /// and AAB win different amounts, so order and share both matter, while
    /// the ordinary coins give every schedule its count of each coin and
    /// nothing more.
    #[test]
    fn the_schedules_are_played_round_by_round() {
        let (a, b, mix) = (PUBLISHED.game_a(), PUBLISHED.game_b(), PUBLISHED.mix());
        let ab = schedule(AB, &PUBLISHED);
        let start = b.shares().map(|w| w.to_f64());
        let states = model::played(&ab, start, 120);
        let dots: Vec<f64> = b_rounds(AB, 120).iter().map(|&k| states[k][0]).collect();
        let line = facts::fair_share(&PUBLISHED).to_f64();
        assert_eq!(dots.len(), 60);
        // Exact rational evolution increases strictly; f64 eventually rounds
        // consecutive values to the same stationary value.
        assert!(dots[..6].windows(2).all(|w| w[1] > w[0]), "{dots:?}");
        assert!(dots.windows(2).all(|w| w[1] >= w[0]), "{dots:?}");
        assert!(dots[..5].iter().all(|&d| d < line) && dots[5..].iter().all(|&d| d > line));
        let exact = model::cycle(&ab)[1][0].to_f64();
        assert!((dots[59] - exact).abs() < 1e-12);
        // one round of the played crowd is evolve_f64 of that round's game
        for (k, pair) in states.windows(2).take(10).enumerate() {
            let g = if k.is_multiple_of(2) { &a } else { &b };
            let next = model::evolve_f64(&pair[0], g);
            assert!((0..3).all(|i| (next[i] - pair[1][i]).abs() < 1e-15));
        }
        let aabb = schedule(AABB, &PUBLISHED);
        let states = model::played(&aabb, start, 400);
        assert!(b_rounds(AABB, 400).iter().all(|&k| states[k][0] < line));
        // failed control: the averaged coin is the coin-toss mix, not AB
        let averaged = Game {
            up: [0, 1, 2].map(|i| (a.up[i] + b.up[i]) / 2),
            den: DEN,
        };
        assert_eq!(averaged, mix);
        assert_eq!(periodic(&[averaged, averaged]).0, mix.mean());
        assert!(periodic(&ab).0 < Ratio::ZERO && mix.mean() > Ratio::ZERO);
        // order and share matter with the published coins
        let means: Vec<Ratio> = facts::PATTERNS
            .iter()
            .map(|p| periodic(&schedule(p, &PUBLISHED)).0)
            .collect();
        assert!(means[0] < Ratio::ZERO && means[1] > Ratio::ZERO, "{means:?}");
        assert!(
            means[2] > Ratio::ZERO && means[3] > Ratio::ZERO && means[2] != means[3],
            "{means:?}"
        );
        // the ordinary coins: each round's average is 2p − 1 wherever the
        // player stands, so a schedule is its count of each coin
        let plain = Coins {
            a: ORDINARY[0] * DEN / 100,
            bad: ORDINARY[1] * DEN / 100,
            good: ORDINARY[1] * DEN / 100,
        };
        for p in facts::PATTERNS {
            let len = p.len() as i64;
            let n_a = p.matches('A').count() as i64;
            let want = Ratio::new(
                (n_a * (2 * ORDINARY[0] - 100) + (len - n_a) * (2 * ORDINARY[1] - 100)).into(),
                (100 * len).into(),
            );
            assert_eq!(periodic(&schedule(p, &plain)).0, want, "{p}");
        }
    }

    /// The curve is the model's at every half percent and its sign is the
    /// products' everywhere; the mix wins exactly strictly between the two
    /// crossings, which fall inside (7.0, 7.1) and (84.7, 84.8) %. Failed
    /// control: the rounded crossings, 7 and 85 %, both lose, so "about
    /// seven to about eighty-five percent" is not a bound to quote exactly.
    #[test]
    fn the_curve_wins_strictly_between_its_crossings() {
        let n = numbers().unwrap();
        let (lo, hi) = (n.crossings[0], n.crossings[1]);
        assert!(7.0 < lo && lo < 7.1 && 84.7 < hi && hi < 84.8, "{lo} {hi}");
        for &(s, c) in &n.curve {
            let verdict = model::mix_verdict_f64(&PUBLISHED, s / 100.0);
            assert_eq!(c > 0.0, verdict > 0.0, "{s}");
            assert_eq!(c > 0.0, lo < s && s < hi, "{s}");
        }
        for (num, den) in WHOLE_EDGES {
            assert!(PUBLISHED.mix_share(num, den).mean() < Ratio::ZERO);
        }
        let best = n.curve.iter().map(|p| p.1).fold(f64::MIN, f64::max);
        assert!(best > n.curve[100].1, "the best share is not one half");
    }

    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in CUES {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert_eq!(text.matches(phrase).count(), 1, "{id} has {phrase:?} not once: {text}");
        }
        for pair in CUES.windows(2) {
            let [(a, x), (b, y)] = [pair[0], pair[1]];
            if a == b {
                let text = script.text(a).unwrap();
                assert!(text.find(x) < text.find(y), "{a}: {x:?} after {y:?}");
            }
        }
    }

    /// The layout keeps its rooms: the panel's rows stack without touching
    /// above the chart, its x ticks above the three low lines, which stay
    /// above the frame's foot; the strip ends before the counter; the chart
    /// and the table keep left of B's triangle; the curve's rows stand above
    /// a three-line lower third and under the badge.
    #[test]
    fn the_layout_keeps_its_rooms() {
        const {
            let card_top = -FRAME_Y_RADIUS + 0.2 + 0.3 + 3.0 * 0.36;
            assert!(RANDOM_Y + 0.2 < FRAME_Y_RADIUS - 0.3);
            assert!(RANDOM_Y - STRIP_Y >= 0.4 && STRIP_Y - NAME_Y >= 0.4 && NAME_Y - COST_Y >= 0.4);
            assert!(COST_Y - CHART_TITLE_Y >= 0.35);
            assert!(CHART_TITLE_Y - 0.12 > CHART.center[1] + CHART.height / 2.0 + 0.1);
            assert!(CHART.center[1] - CHART.height / 2.0 - TICK_DY - 0.15 > LOW_YS[0] + 0.2);
            assert!(LOW_YS[0] - LOW_YS[1] >= 0.45 && LOW_YS[1] - LOW_YS[2] >= 0.45);
            assert!(LOW_YS[2] - 0.15 > -FRAME_Y_RADIUS + 0.3);
            assert!(STRIP_X + PITCH * CELLS as f64 + 0.15 < COUNTER_X - 0.1);
            assert!(CHART.center[0] + CHART.width / 2.0 < PANEL_END);
            assert!(CHART.center[0] - CHART.width / 2.0 - 0.6 > -FRAME_X_RADIUS + 0.3);
            assert!(HEAD_Y - ROW_YS[0] >= 0.5 && ROW_YS[0] - ROW_YS[1] >= 0.5 && ROW_YS[1] - ROW_YS[2] >= 0.5);
            assert!(ROW_YS[2] - NOTE_Y >= 0.5);
            assert!(MEETS_X - PANEL_X >= 2.5 && CENTS_X - MEETS_X >= 2.2 && PANEL_END - CENTS_X >= 1.1);
            assert!(PLAIN_AT[0][0] - PLAIN_R > PLAIN_X + 3.4 && PLAIN_AT[1][0] + PLAIN_R < FRAME_X_RADIUS - 0.3);
            assert!(PLAIN_AT[0][1] + PLAIN_R < BADGE_AT[1] - 0.19 - 0.1);
            assert!(THINK_AT[1] + 0.62 < BADGE_AT[1] - 0.19 - 0.2);
            assert!(CURVE.center[1] + CURVE.height / 2.0 < CURVE_TITLE_Y - 0.15);
            assert!(CURVE_TITLE_Y + 0.15 < BADGE_AT[1] - 0.19);
            assert!(CURVE.center[1] - CURVE.height / 2.0 - TICK_DY - 0.15 > AXIS_Y + 0.15);
            assert!(AXIS_Y - EDGES_Y >= 0.4 && EDGES_Y - 0.15 > card_top + 0.1);
            assert!(CURVE.center[0] - CURVE.width / 2.0 - 0.6 > -FRAME_X_RADIUS + 0.3);
        }
        // the strip, the chart and the table stand left of corner 2's disc
        let c2 = corner(B_AT, 2);
        assert!(c2[0] - CORNER_R - 0.3 > PANEL_END, "{c2:?}");
        // the line's words: AB's settled dots sit above the line by less
        // than a dot, so "B breaks even" goes under it, clear of two and
        // two's highest dot (36.0 %) by more than a dot's radius
        let n = numbers().unwrap();
        let line_y = CHART.p(CHART.x.0, n.line)[1];
        let words_bottom = line_y - NEAR + LIFT - 0.11;
        let top_aabb = n.aabb.b.iter().map(|&k| n.aabb.states[k][0]).fold(0.0, f64::max);
        assert!(words_bottom - (CHART.p(1.0, 100.0 * top_aabb)[1] + DOT_R) > DOT_R);
        let above = CHART.p(1.0, 100.0 * n.cycle[1][0])[1] - DOT_R - line_y;
        assert!(above > 0.0 && above < 0.05, "{above}");
        // the countdown, its ring and note, right of the triangle's top corner
        let c0 = corner(B_AT, 0);
        assert!(THINK_AT[0] - 0.62 - (c0[0] + CORNER_R) > 1.0);
        // the curve's points stay inside its window
        for &(s, c) in &n.curve {
            assert!(CURVE.y.0 < c && c < CURVE.y.1, "{s}: {c}");
        }
    }

    /// The "precisely" card fits the lower third's three lines (as 15's).
    #[test]
    fn the_card_fits_three_lines() {
        let n = numbers().unwrap();
        let lines = explainer_kit::animatic::wrap(&n.k07, 76, usize::MAX);
        assert!(lines.len() <= 3, "{lines:?}");
        assert_eq!(lines.join(" "), n.k07);
    }

    /// The chapter runs headless to its end with the scratch voice and real
    /// typesetting (every `keep_between` and `keep_apart` guard meets the
    /// true text widths): every line of chapter 16 is said once, in order;
    /// the prediction (16_3) gets its silent time; the reveals and the curve
    /// stand to be read, the card for its reading time; the last picture
    /// stands.
    #[test]
    fn the_chapter_runs_and_leaves_room_to_think() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = WhenSwitchingFails { kit: kit.clone() };
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut NullSceneSink)
            .unwrap_or_else(|e| panic!("16_when_switching_fails: {e}"));
        let said = narrator.said();
        let ids: Vec<&str> = said.iter().map(|(id, _)| id.as_str()).collect();
        let expected: Vec<String> = (1..=15).map(|k| format!("16_{k}")).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        let at = |id: &str| said.iter().find(|(l, _)| l == id).unwrap().1;
        let end_of = |id: &str| at(id) + narrator.length(id).unwrap();
        let think = at("16_4") - end_of("16_3");
        assert!(think >= THINK_S, "{think} s to think");
        for (before, after) in [
            ("16_4", "16_5"),
            ("16_6", "16_7"),
            ("16_8", "16_9"),
            ("16_9", "16_10"),
            ("16_11", "16_12"),
        ] {
            let gap = at(after) - end_of(before);
            assert!(gap >= BEAT, "{before}: {gap} s");
        }
        for (before, after) in [("16_7", "16_8"), ("16_10", "16_11"), ("16_13", "16_14")] {
            let gap = at(after) - end_of(before);
            assert!(gap >= INSPECT_S, "{before}: {gap} s");
        }
        let n = numbers().unwrap();
        assert!(at("16_15") - at("16_14") >= read_s(&n.k07), "the card is not readable");
        let end = kit.finished.borrow().as_ref().expect("a timeline").seconds();
        assert!(
            end - end_of("16_15") >= INSPECT_S,
            "the last picture is not held: {end}"
        );
    }
}
