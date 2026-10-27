//! Chapter 17, what winning promises. Back to the player (17_1: from an
//! empty frame, the filled published badge; a window at its own scale,
//! rounds 0 to 1,000 against the player's dollars, the start at 7 dashed;
//! the teal token rides 03's seeded run of game B to "+16 dollars after
//! 1,000 rounds", "game B: −0.87 cents a round" under it; then a thumbnail
//! of all 100,000 rounds at its own scale ending at "−548 dollars after
//! 100,000"). The question of a fixed number of rounds is answered exactly
//! (17_2): the axis "dollars won or lost" from −100 to +140, "everyone starts
//! here" at 0, "a crowd starts at 0 dollars, on corner 0, and plays the mix",
//! the negative ticks pointed at, "−1 dollar stands on corner 2". After 100
//! rounds the exact distribution grows from the axis (17_3), one bar a dollar
//! wide on every even total: the tallest framed, "+2 dollars: 10.2 %", the
//! share axis, two empty odd slots framed, "odd totals: no one", "all bars
//! together: 100.0 %"; the bars of corner-2 totals outlined, then added up by
//! corner: 34.5, 25.4, 40.1 %, the mix's settled crowd again. Colored by
//! where each player finished, with a readout as each is said: ahead 51.7 %
//! (gold), behind 39.6 % (red), level 8.7 % (dim), and the "precisely" card
//! K08 (17_4). The crowd's average, a line at +1.29 dollars, against "100
//! rounds at 1.57 cents = 1.57 dollars" and "starting on corner 0: −0.28
//! dollars, once"; the red bars and "behind 39.6 %" pointed at (17_5). A
//! thousand rounds: ahead? (17_6, a prediction.) More: "after 1,000 rounds:
//! ahead 68.9 %" (17_7); then the same bars on the same axis run through the
//! exact distributions after 100, 102, … 1,000 rounds, one exact crowd in
//! every frame, the readouts and the average's line following them, the
//! counter counting the rounds (the voice silent). Behind 28.7 % and level
//! 2.4 % pointed at; "900 more rounds: +14.13 dollars, +1.57 cents a round";
//! a bracket one spread either side of +15.42, "spread: 9.3 dollars at 100
//! rounds, 29.6 at 1,000" (17_8). "10 times the rounds, 3.16 times the
//! spread", √10 ≈ 3.16, and K10: square-root growth is a long-run law, not an
//! exact rule (17_9). The player's +16 against a crowd that starts where the
//! player did, 7 dollars on corner 1, and plays game B alone for 1,000
//! rounds: its bars grow, ahead 34.5 %, at least +16 14.6 % (framed), a
//! teal mark at +16 (17_10). Behind after 1,000 rounds from 0: game A alone
//! 61.2 %, game B alone 63.9 %, the mix 28.7 % (17_11). The long run, with
//! probability one, and K09 (17_12). Three different promises (17_13), and
//! B's triangle, with its red corner 0 (17_14).
//!
//! Every distribution is `model::snapshots` (exact in floats, one pass per
//! game), every bar is one total at one share on one fixed pair of axes, and
//! every number is checked against `crate::facts`; the scene refuses to draw
//! if the words would stop being true of them. No typeset text is
//! indicated, scaled or recoloured: words get a frame, shapes pulse or get
//! outlines, a changed label is replaced.

use std::rc::Rc;

use explainer_kit::props::{grow_histogram, precisely};
use explainer_kit::{Frame2, Kit, Palette, fade_all, rect, thousands, v};
use fmn::core::constants::FRAME_X_RADIUS;
use fmn::prelude::*;

use super::draw::{
    dashed, draw_run, fade_in_all, fade_out_all, fail, flat, growing_run, keep_between, line_style, now, pause, read_s,
    say, screen, think, thinned, token, wait_until,
};
use super::looks_hopeless::{POINT_PAD, THIN, badge_text, frame_around, full_badge, grow, point_at};
use super::net_flow::{INSPECT_S, csv, draw_bands};
use super::one_dial::{BEAT, PERCENT, pct_value};
use super::three_corners::{Chances, Which, chance_labels, chances, corner_of, keep_apart, swap, triangle};
use crate::facts::{self, CROWD_START, HORIZON, LONG_HORIZON, SHORT, WINDOW};
use crate::model::{self, PUBLISHED, Play, RUN_ROUNDS, Ratio, START};
use crate::palette;

pub struct WhatWinningPromises {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const ROUNDS: &str = "rounds";
const DOLLAR: &str = "dollar";
const DOLLARS: &str = "dollars";
const DOLLARS_AFTER: &str = "dollars after";
const RUN_TITLE: &str = "the player's dollars: the first";
const OF_GAME_B: &str = "rounds of game B";
const START_WORD: &str = "start";
const ALL: &str = "all";
const GAME_A: &str = "game A";
const GAME_B: &str = "game B";
const CENTS: &str = "cents";
const CENTS_A_ROUND: &str = "cents a round";
const CROWD_STARTS: &str = "a crowd starts at";
const ON_CORNER: &str = "on corner";
const PLAYS_MIX: &str = "and plays the mix";
const PLAYS_B: &str = "and plays game B alone";
const STANDS_ON: &str = "stands on corner";
const EVERYONE: &str = "everyone starts here";
const WON_OR_LOST: &str = "dollars won or lost";
const SHARE_OF_CROWD: &str = "share of the crowd";
const ODD_TOTALS: &str = "odd totals: no one";
const ALL_BARS: &str = "all bars together:";
const ADDED_UP: &str = "added up by corner:";
const CORNER: &str = "corner";
const ROUNDS_PLAYED: &str = "rounds played";
const AHEAD: &str = "ahead";
const BEHIND: &str = "behind";
const LEVEL: &str = "level";
const AVERAGE: &str = "average";
const ROUNDS_AT: &str = "rounds at";
const STARTING_ON: &str = "starting on corner";
const ONCE: &str = "once";
const AFTER: &str = "after";
const MORE_ROUNDS: &str = "more rounds";
const SPREAD: &str = "spread";
const AT: &str = "at";
const TIMES_ROUNDS: &str = "times the rounds";
const TIMES_SPREAD: &str = "times the spread";
const NO_FEWER: &str = "at least";
const THE_PLAYER: &str = "the player";
const FROM: &str = "from";
const ALONE: &str = "alone";
const MIX_ROW: &str = "the mix";
const FOREVER: &str = "one player, playing forever: average per round";
const TENDS: &str = "\u{2192}";
const PROBABILITY_ONE: &str = "with probability one";
const CROWD_AVERAGE: &str = "the crowd's average: grows";
const CHANCE_AHEAD: &str = "chance of being ahead:";
const NEVER_CERTAIN: &str = "never certain";
const ONE_PLAYER: &str = "one player's average per round:";

/// Where each picture lands: (line, the phrase it waits for).
const AT_FIRST_1000: (&str, &str) = ("17_1", "Over the first thousand rounds");
const AT_AHEAD_16: (&str, &str) = ("17_1", "sixteen dollars ahead");
const AT_EVEN_THOUGH: (&str, &str) = ("17_1", "even though game B loses");
const AT_CAUGHT_UP: (&str, &str) = ("17_1", "The long run caught up later");
const AT_DOWN: (&str, &str) = ("17_1", "five hundred forty-eight dollars down");
const AT_FIXED_NUMBER: (&str, &str) = ("17_2", "over a fixed number of rounds");
const AT_CROWD_ZERO: (&str, &str) = ("17_2", "start a whole crowd at zero dollars");
const AT_HUNDRED: (&str, &str) = ("17_2", "play the coin-toss mix for a hundred rounds");
const AT_BELOW_ZERO: (&str, &str) = ("17_2", "Dollars can go below zero");
const AT_MINUS_ONE: (&str, &str) = ("17_2", "a player with minus one dollar");
const AT_SPREAD_OUT: (&str, &str) = ("17_3", "the crowd has spread out");
const AT_EACH_BAR: (&str, &str) = ("17_3", "Each bar is one possible total");
const AT_HEIGHT: (&str, &str) = ("17_3", "its height is the share of the crowd");
const AT_EVEN: (&str, &str) = ("17_3", "Every bar sits on an even number");
const AT_WHOLE: (&str, &str) = ("17_3", "Together the bars hold the whole crowd");
const AT_CORNERS: (&str, &str) = ("17_3", "The heights rise and fall with the corners");
const AT_ADD_UP: (&str, &str) = ("17_3", "add up the bars on each corner");
const AT_MOST: (&str, &str) = ("17_3", "with the most on corner two");
const AT_COLOR: (&str, &str) = ("17_4", "Color them by where each player finished");
const AT_AHEAD: (&str, &str) = ("17_4", "Ahead of the start");
const AT_BEHIND: (&str, &str) = ("17_4", "Behind: thirty-nine point six");
const AT_LEVEL: (&str, &str) = ("17_4", "Exactly level");
const AT_AVERAGE: (&str, &str) = ("17_5", "The crowd's average after a hundred rounds");
const AT_AT_RATE: (&str, &str) = ("17_5", "would make one fifty-seven");
const AT_START_COST: (&str, &str) = ("17_5", "costs about twenty-eight cents, once");
const AT_HIDES: (&str, &str) = ("17_5", "that average hides a lot");
const AT_TWO_IN_FIVE: (&str, &str) = ("17_5", "almost two in five players are behind");
const AT_MORE_OR_LESS: (&str, &str) = ("17_6", "Will more of it finish ahead");
const AT_SIXTY_EIGHT: (&str, &str) = ("17_7", "Sixty-eight point nine percent finish ahead");
const AT_WATCH_BARS: (&str, &str) = ("17_7", "Watch the same bars");
const AT_BEHIND_28: (&str, &str) = ("17_8", "Behind: twenty-eight point seven percent");
const AT_LEVEL_2: (&str, &str) = ("17_8", "Level: two point four");
const AT_KEPT_GROWING: (&str, &str) = ("17_8", "has kept growing by about one and a half cents a round");
const AT_SPREAD_GROWN: (&str, &str) = ("17_8", "the spread of the totals has grown more slowly");
const AT_TEN_TIMES: (&str, &str) = ("17_9", "Ten times the rounds");
const AT_SQRT_TEN: (&str, &str) = ("17_9", "close to the square root of ten");
const AT_LONG_RUNS: (&str, &str) = ("17_9", "For long runs the spread grows");
const AT_PLAYERS_16: (&str, &str) = ("17_10", "Now the player's sixteen dollars");
const AT_START_SEVEN: (&str, &str) = ("17_10", "Start a crowd at seven dollars");
const AT_B_ALONE: (&str, &str) = ("17_10", "play game B alone for a thousand rounds");
const AT_B_AHEAD: (&str, &str) = ("17_10", "Thirty-four point five percent finish ahead");
const AT_AT_LEAST: (&str, &str) = ("17_10", "fourteen point six percent finish at least sixteen dollars up");
const AT_NOT_RARE: (&str, &str) = ("17_10", "lucky, but not rare");
const AT_FROM_ZERO: (&str, &str) = ("17_11", "Over a thousand rounds from zero dollars");
const AT_A_BEHIND: (&str, &str) = ("17_11", "game A alone leaves sixty-one point two percent");
const AT_B_BEHIND: (&str, &str) = ("17_11", "game B alone sixty-three point nine percent");
const AT_MIX_BEHIND: (&str, &str) = ("17_11", "only twenty-eight point seven percent");
const AT_PROBABILITY_ONE: (&str, &str) = ("17_12", "With probability one");
const AT_NOT_CROWD: (&str, &str) = ("17_12", "Not on average over a crowd");
const AT_THREE_THINGS: (&str, &str) = ("17_13", "promises three different things");
const AT_CROWD_AVG: (&str, &str) = ("17_13", "The crowd's average grows");
const AT_CHANCE: (&str, &str) = ("17_13", "The chance of being ahead");
const AT_NEVER_CERTAIN: (&str, &str) = ("17_13", "it never becomes certain");
const AT_OWN_AVERAGE: (&str, &str) = ("17_13", "one player's own average");
const AT_INVENTED: (&str, &str) = ("17_14", "These are invented games");
const AT_WHERE_STAND: (&str, &str) = ("17_14", "depend on where you stand");
#[cfg(test)]
const CUES: [(&str, &str); 56] = [
    AT_FIRST_1000,
    AT_AHEAD_16,
    AT_EVEN_THOUGH,
    AT_CAUGHT_UP,
    AT_DOWN,
    AT_FIXED_NUMBER,
    AT_CROWD_ZERO,
    AT_HUNDRED,
    AT_BELOW_ZERO,
    AT_MINUS_ONE,
    AT_SPREAD_OUT,
    AT_EACH_BAR,
    AT_HEIGHT,
    AT_EVEN,
    AT_WHOLE,
    AT_CORNERS,
    AT_ADD_UP,
    AT_MOST,
    AT_COLOR,
    AT_AHEAD,
    AT_BEHIND,
    AT_LEVEL,
    AT_AVERAGE,
    AT_AT_RATE,
    AT_START_COST,
    AT_HIDES,
    AT_TWO_IN_FIVE,
    AT_MORE_OR_LESS,
    AT_SIXTY_EIGHT,
    AT_WATCH_BARS,
    AT_BEHIND_28,
    AT_LEVEL_2,
    AT_KEPT_GROWING,
    AT_SPREAD_GROWN,
    AT_TEN_TIMES,
    AT_SQRT_TEN,
    AT_LONG_RUNS,
    AT_PLAYERS_16,
    AT_START_SEVEN,
    AT_B_ALONE,
    AT_B_AHEAD,
    AT_AT_LEAST,
    AT_NOT_RARE,
    AT_FROM_ZERO,
    AT_A_BEHIND,
    AT_B_BEHIND,
    AT_MIX_BEHIND,
    AT_PROBABILITY_ONE,
    AT_NOT_CROWD,
    AT_THREE_THINGS,
    AT_CROWD_AVG,
    AT_CHANCE,
    AT_NEVER_CERTAIN,
    AT_OWN_AVERAGE,
    AT_INVENTED,
    AT_WHERE_STAND,
];

// ------------------------------------------------------------------ layout

/// The badge keeps right of this; the top-left lines end before it.
const BADGE_LEFT: f64 = 1.0;
const WORDS_SIZE: f64 = 20.0;
const SMALL: f64 = 18.0;
const TEX_SIZE: f64 = 24.0;
const EDGE: f64 = 0.3;
const TICK_DY: f64 = 0.22;
/// 17_1: the player's first 1,000 rounds at their own scale (its dollars
/// padded to fives), with 0 and 20 dollars marked and the start dashed;
/// the run's words right of its end, a row apart; the thumbnail of all
/// 100,000 rounds at the bottom right, its words above and below it.
const RUN_CENTER: Vec3 = [-1.6, 0.25, 0.0];
const RUN_W: f64 = 8.6;
const RUN_H: f64 = 4.2;
const RUN_PAD: i64 = 5;
const RUN_TICKS: [i64; 2] = [0, 20];
const RUN_STEP: usize = 2;
const RUN_MIN_S: f64 = 2.5;
const END_DX: f64 = 0.45;
const ROW_GAP: f64 = 0.45;
const THUMB_CENTER: Vec3 = [5.1, -1.9, 0.0];
const THUMB_W: f64 = 3.0;
const THUMB_H: f64 = 1.3;
const THUMB_WORDS_DY: f64 = 0.25;
const END_R: f64 = 0.07;
/// 17_2 to 17_10: the histogram, dollars won or lost against the share of
/// the crowd at that total; one bar a dollar wide on each total.
const HIST: Frame2 = Frame2 {
    center: [0.0, 0.45, 0.0],
    width: 12.0,
    height: 3.9,
    x: (-101.0, 141.0),
    y: (0.0, 0.11),
};
const X_TICKS: [i64; 5] = [-100, -50, 0, 50, 100];
const Y_TICKS: [i64; 3] = [0, 5, 10];
const AXIS_NAME_DY: f64 = 0.6;
const BAR_W: f64 = 1.0;
const TEAL_O: f64 = 0.85;
const OUTLINE_W: f64 = 2.0;
/// Corner-2 bars this tall (a share) get an outline at 17_3.
const OUTLINE_MIN: f64 = 0.01;
/// A bar this tall or taller counts as seen when a frame goes round bars.
const SEEN: f64 = 1e-4;
/// The mass a drawn frame may leave outside the window, at most.
const WINDOW_LOSS: f64 = 1e-4;
/// The start (17_2), a dot on the axis with its words over it, and the
/// note on −1 above them, ending left of the dot (a tick at −1 would stand
/// inside the dot: a dollar is 0.05 units here).
const START_R: f64 = 0.1;
const START_DY: f64 = 0.45;
const MINUS_DY: f64 = 0.9;
/// The setup line at the top left; the words beside the tallest bar and the
/// empty slots, under the share axis's top.
const SETUP_AT: Vec3 = [-6.6, 3.3, 0.0];
const LABEL_Y: f64 = 2.3;
/// The readouts at the top right: a label, a value whose right edge is
/// [`VALUE_X`], a unit; rows for the rounds, ahead, behind, level, average.
const LABEL_X: f64 = 3.6;
const VALUE_X: f64 = 5.85;
const UNIT_X: f64 = 5.95;
const READ_YS: [f64; 5] = [3.0, 2.6, 2.2, 1.8, 1.4];
const COUNT_AT: Vec3 = [5.55, 3.0, 0.0];
/// The column right of the bars: 17_3's sums, 17_5's comparison, 17_6's
/// question and its answer, 17_8's rate and spread, 17_9's ratio.
const RIGHT_X: f64 = 1.6;
const COLUMN_YS: [f64; 5] = [1.4, 0.95, 0.5, 0.05, -0.4];
const THINK17: Vec3 = [5.3, -0.25, 0.0];
/// 17_8's bracket, one spread either side of the average, over the bars.
const BRACKET_Y: f64 = -0.15;
const BRACKET_TICK: f64 = 0.08;
const AVERAGE_W: f64 = 3.0;
/// 17_10's mark at the player's +16, its words over the axis's top.
const MARK_DY: f64 = 0.22;
/// 17_11: the three crowds' behind, a red bar each (6 units for the whole
/// crowd); 17_12's line under them; 17_13's three lines.
const TABLE_X: f64 = -6.0;
const BEHIND_HEAD_Y: f64 = 2.4;
const BEHIND_YS: [f64; 3] = [1.5, 0.8, 0.1];
const BEHIND_X: f64 = -2.6;
const BEHIND_U: f64 = 6.0;
const BEHIND_H: f64 = 0.34;
const TABLE_SIZE: f64 = 22.0;
const FOREVER_Y: f64 = -0.9;
const SUMMARY_YS: [f64; 3] = [1.6, 0.6, -0.4];
/// Seconds: the bars' growth and its lag, the transformation, a bar of the
/// table.
const GROW_S: f64 = 1.6;
const GROW_LAG: f64 = 0.02;
const MORPH_S: f64 = 10.0;
const BAR_S: f64 = 0.7;
/// The transformation steps two rounds at a time, so that every frame is the
/// exact crowd after an even number of rounds (every total even).
const MORPH_STEP: usize = 2;
const STEPS: usize = (LONG_HORIZON - HORIZON) / MORPH_STEP;

// ---------------------------------------------------------------- numbers

/// The chapter's numbers, checked against the words they illustrate.
struct Numbers {
    badge: String,
    run: model::Run,
    /// The first window's dollars, padded to fives.
    run_y: (i64, i64),
    run_title: String,
    end_words: String,
    b_loses: String,
    long_words: [String; 2],
    setup: [String; 2],
    minus_one: String,
    /// The mix's crowd after 100, 102, … 1,000 rounds from 0 dollars.
    snaps: Rc<Vec<Vec<f64>>>,
    /// Ahead, behind, level and the average, dollars, for each of them.
    readings: Rc<Vec<[f64; 4]>>,
    tallest: (i64, f64),
    tallest_words: String,
    whole: String,
    corners: [String; 3],
    /// After 100 rounds: ahead, behind, level (%) and the average.
    values: [String; 4],
    at_rate: String,
    start_cost: String,
    question: String,
    answer: String,
    rate: String,
    spread: String,
    /// One spread either side of the average after 1,000 rounds.
    bracket: (f64, f64),
    ratio_words: String,
    sqrt_tex: String,
    b_dist: Vec<f64>,
    b_values: [String; 2],
    lucky: i64,
    at_least: String,
    player_words: String,
    behind_head: String,
    behind_rows: [(f64, String); 3],
    forever: String,
    summary: [String; 3],
    b_chances: Chances,
    k08: String,
    k09: String,
    k10: String,
}

/// A share as a percentage at one place, as the facts print it.
fn pct1(x: f64) -> String {
    format!("{:.1}", 100.0 * x)
}

/// A percentage as the screen writes it, "51.7 %".
fn pct_words(x: &str) -> String {
    format!("{x} {PERCENT}")
}

/// A printed decimal ("+1.29", "0.28") as the exact number it shows.
fn printed(text: &str) -> Result<Ratio, String> {
    Ok(pct_value(text.trim_start_matches('+'))? * Ratio::int(100))
}

/// Whether `x` is within `tol` of `target`.
fn near(x: Ratio, target: Ratio, tol: Ratio) -> bool {
    x - target < tol && target - x < tol
}

/// The totals a histogram draws: those inside [`WINDOW`] that hold some of
/// the crowd, as (dollars won or lost, share). Entry k of a distribution
/// from `model::snapshots` is the total k − 1,000.
fn drawn(dist: &[f64]) -> Vec<(i64, f64)> {
    let last = LONG_HORIZON as i64;
    dist.iter()
        .enumerate()
        .map(|(k, &w)| (k as i64 - last, w))
        .filter(|&(net, w)| w > 0.0 && (WINDOW.0..=WINDOW.1).contains(&net))
        .collect()
}

/// The share of a distribution inside the window.
fn inside(dist: &[f64]) -> f64 {
    drawn(dist).iter().map(|p| p.1).sum()
}

/// A crowd's shares by the corner its total stands on, from `start` dollars.
fn by_corner(dist: &[f64], start: i64) -> [f64; 3] {
    let last = LONG_HORIZON as i64;
    let mut out = [0.0; 3];
    for (k, &w) in dist.iter().enumerate() {
        out[corner_of(start + k as i64 - last)] += w;
    }
    out
}

/// The transformation's rate: its tracker moves in whole steps of
/// [`MORPH_STEP`] rounds, so the counter and the bars read the same round.
fn stepped(t: f64) -> f64 {
    (t * STEPS as f64).round() / STEPS as f64
}

/// The snapshot for a tracker at `rounds` (100 to 1,000).
fn snapshot_at(rounds: f64) -> usize {
    let k = ((rounds - HORIZON as f64) / MORPH_STEP as f64).round();
    k.clamp(0.0, STEPS as f64) as usize
}

/// Where a total stands: ahead gold, behind red, level dim.
fn category_color(net: i64) -> Srgb {
    match net.signum() {
        1 => palette::mix(),
        -1 => palette::bad(),
        _ => Palette::dim(),
    }
}

/// One bar: the total `net`, a dollar wide, `share` of the crowd high.
fn bar(net: i64, share: f64, style: Style) -> fmn::Result<VMobject> {
    let x = net as f64;
    let (dl, ur) = (HIST.p(x - BAR_W / 2.0, 0.0), HIST.p(x + BAR_W / 2.0, share));
    rect(dl, ur[0] - dl[0], ur[1] - dl[1], style)
}

/// The bars of a distribution whose totals `keep`, in `color(total)` at
/// opacity `o`, as one group. Detached (a VMobject).
fn bars(dist: &[f64], keep: impl Fn(i64) -> bool, color: impl Fn(i64) -> Srgb, o: f64) -> fmn::Result<VMobject> {
    let mut out = Vec::new();
    for (net, w) in drawn(dist) {
        if keep(net) {
            out.push(bar(net, w, flat(color(net), o))?);
        }
    }
    Ok(v_group(out))
}

/// An unseen box over the totals from `from` to `to` (dollars) up to
/// `share`, for a frame to go round. Detached.
fn slot(stage: &mut Stage<'_>, from: f64, to: f64, share: f64) -> fmn::Result<Mob> {
    let (dl, ur) = (HIST.p(from, 0.0), HIST.p(to, share));
    let r = rect(dl, ur[0] - dl[0], ur[1] - dl[1], flat(Palette::hero(), 0.0))?;
    Ok(stage.arena_mut().add(r))
}

/// The crowd's average as a line from the axis to the top. Detached.
fn average_line(mean: f64) -> VMobject {
    polyline([HIST.p(mean, HIST.y.0), HIST.p(mean, HIST.y.1)])
        .style(line_style(Palette::ink(), AVERAGE_W))
        .build()
}

/// 17_1's window, at the dollars `y`.
fn run_frame(y: (i64, i64)) -> Frame2 {
    Frame2 {
        center: RUN_CENTER,
        width: RUN_W,
        height: RUN_H,
        x: (0.0, SHORT as f64),
        y: (y.0 as f64, y.1 as f64),
    }
}

/// The thumbnail of the whole run, from its lowest to its highest dollars.
fn thumb_frame(run: &model::Run) -> Frame2 {
    Frame2 {
        center: THUMB_CENTER,
        width: THUMB_W,
        height: THUMB_H,
        x: (0.0, RUN_ROUNDS as f64),
        y: (run.low() as f64, run.high() as f64),
    }
}

fn numbers() -> Result<Numbers, String> {
    let f = facts::facts()?;
    let get = |k: &str| f.get(k).map(str::to_owned).ok_or_else(|| format!("no fact {k}"));
    let (a, b, mix) = (PUBLISHED.game_a(), PUBLISHED.game_b(), PUBLISHED.mix());
    let last = LONG_HORIZON;
    // 17_1: the player's seeded run of game B (03's), at round 1,000 and at
    // its end
    let run = model::player(Play::B);
    if run.dollars[0] != START
        || run.dollars.len() != RUN_ROUNDS + 1
        || START.to_string() != get("run.start")?
        || RUN_ROUNDS.to_string() != get("run.rounds")?
        || SHORT.to_string() != get("run.short")?
    {
        return Err("17_1 replays the player's 100,000 rounds of game B from 7 dollars".into());
    }
    let short = run.net_after(SHORT);
    if short.to_string() != get("run.b.net.short")? || short <= 0 {
        return Err("17_1 says the player was sixteen dollars ahead after 1,000 rounds".into());
    }
    if b.mean() >= Ratio::ZERO || b.mean().cents(2) != get("pub.b.mean.cents")? {
        return Err("17_1 says game B loses".into());
    }
    if run.net().to_string() != get("run.b.net")?
        || run.net() >= 0
        || run.low().to_string() != get("run.b.low")?
        || run.high().to_string() != get("run.b.high")?
    {
        return Err("17_1 says the long run caught up: 548 dollars down after 100,000 rounds".into());
    }
    let first = &run.dollars[..=SHORT];
    let (lo, hi) = (
        *first.iter().min().ok_or("no rounds")?,
        *first.iter().max().ok_or("no rounds")?,
    );
    let run_y = (
        (lo - RUN_PAD).div_euclid(RUN_PAD) * RUN_PAD,
        (hi + 2 * RUN_PAD - 1).div_euclid(RUN_PAD) * RUN_PAD,
    );
    if RUN_TICKS
        .iter()
        .chain([&START])
        .any(|t| !(run_y.0..=run_y.1).contains(t))
    {
        return Err("17_1's ticks and start must lie inside its window".into());
    }
    let short_words = thousands(SHORT as u64);
    let long_rounds = thousands(RUN_ROUNDS as u64);
    // 17_2: the crowd starts at 0 dollars, on corner 0; below zero, a
    // remainder still names the corner
    if CROWD_START.to_string() != get("horizon.start")? || corner_of(CROWD_START) != 0 {
        return Err("17_2 starts the crowd at zero dollars, on corner zero".into());
    }
    if corner_of(-1) != 2 {
        return Err("17_2 says a player with minus one dollar stands on corner two".into());
    }
    if csv([HORIZON, LONG_HORIZON]) != get("horizon.rounds")?
        || (LONG_HORIZON / HORIZON).to_string() != get("horizon.rounds.ratio")?
        || LONG_HORIZON != 10 * HORIZON
    {
        return Err("17_6 and 17_9 say a thousand rounds are ten times a hundred".into());
    }
    let setup = [
        format!(
            "{CROWD_STARTS} {CROWD_START} {DOLLARS}, {ON_CORNER} {}, {PLAYS_MIX}",
            corner_of(CROWD_START)
        ),
        format!(
            "{CROWD_STARTS} {START} {DOLLARS}, {ON_CORNER} {}, {PLAYS_B}",
            corner_of(START)
        ),
    ];
    // 17_3: the exact crowds after every even number of rounds, 100 to
    // 1,000, each a whole crowd on even totals, inside the window and under
    // the share axis's top
    let rounds: Vec<usize> = (HORIZON..=LONG_HORIZON).step_by(MORPH_STEP).collect();
    if rounds.len() != STEPS + 1 || rounds.iter().any(|n| !n.is_multiple_of(2)) {
        return Err("17_7 runs through every even number of rounds from 100 to 1,000".into());
    }
    let snaps = model::snapshots(&mix, CROWD_START, &rounds);
    for (d, n) in snaps.iter().zip(&rounds) {
        let total: f64 = d.iter().sum();
        let odd: f64 = d.iter().skip(1).step_by(2).sum();
        if (total - 1.0).abs() > 1e-9 || odd != 0.0 {
            return Err(format!(
                "17_3 says the bars after {n} rounds hold the whole crowd, on even totals"
            ));
        }
        if inside(d) < 1.0 - WINDOW_LOSS || d.iter().any(|&w| w >= HIST.y.1) {
            return Err(format!("the crowd after {n} rounds does not fit the histogram"));
        }
    }
    let (d100, d1000) = (&snaps[0], &snaps[snaps.len() - 1]);
    let readings: Vec<[f64; 4]> = snaps
        .iter()
        .map(|d| {
            let (up, down, level) = model::categories(d, last);
            [up, down, level, model::moments(d, last).0]
        })
        .collect();
    let (r100, r1000) = (readings[0], readings[readings.len() - 1]);
    let cats = |r: &[f64; 4]| csv(r[..3].iter().map(|&x| pct1(x)));
    if cats(&r100) != get("horizon.short.mix")?
        || cats(&r1000) != get("horizon.long.mix")?
        || pct1(r1000[0]) != get("horizon.long.mix.ahead.pct")?
    {
        return Err("17_4 and 17_7's shares disagree with the facts".into());
    }
    let (top_k, top_w) = d100
        .iter()
        .enumerate()
        .fold((0, 0.0), |best, (k, &w)| if w > best.1 { (k, w) } else { best });
    let top = top_k as i64 - last as i64;
    if format!("{top}:{}", pct1(top_w)) != get("horizon.short.mix.tallest")? {
        return Err("17_3's tallest bar disagrees with the facts".into());
    }
    let b_dist = model::snapshots(&b, START, &[LONG_HORIZON]).remove(0);
    if csv([d100, d1000, &b_dist].map(|d| pct1(inside(d)))) != get("horizon.window.pct")? {
        return Err("17_3 says the bars hold the whole crowd".into());
    }
    // 17_3: added up by corner, the bars are the mix's settled crowd again
    let settled = csv(mix.shares().map(|w| w.pct(1)));
    let corner_sums = by_corner(d100, CROWD_START);
    for d in [d100, d1000] {
        let c = by_corner(d, CROWD_START);
        if csv(c.map(pct1)) != settled || settled != get("pub.mix.weights.pct")? || c[2] <= c[0] || c[2] <= c[1] {
            return Err("17_3 says the bars added up by corner are the settled crowd, the most on corner 2".into());
        }
    }
    // 17_4, 17_5, 17_7, 17_13: just over half ahead, almost two in five
    // behind; more than two in three ahead after 1,000 rounds, more than
    // after 100; never certain, since every round can lose
    if !(0.5..0.55).contains(&r100[0])
        || !(0.35..0.4).contains(&r100[1])
        || r1000[0] <= 2.0 / 3.0
        || r1000[0] <= r100[0]
    {
        return Err("17_4 to 17_7 say just over half, almost two in five, then more than two in three".into());
    }
    if (0..3).any(|i| mix.chance(i) >= Ratio::ONE) || readings.iter().any(|r| r[1] <= 0.0) {
        return Err("17_13 says being ahead never becomes certain".into());
    }
    // 17_5: the crowd's average against 100 rounds at the mix's rate, and
    // the start's cost, the same once at 100 and at 1,000 rounds
    let mean = mix.mean();
    if mean.to_string() != get("pub.mix.mean")? || mean.cents(2) != get("pub.mix.mean.cents")? {
        return Err("17_5 and 17_12 use the mix's average".into());
    }
    let means = [r100[3], r1000[3]].map(|m| format!("{m:+.2}"));
    if csv(means.clone()) != get("horizon.mix.mean")? {
        return Err("17_5 and 17_8's averages disagree with the facts".into());
    }
    let at_rate = [HORIZON, LONG_HORIZON].map(|n| Ratio::int(n as i64) * mean);
    let cost = get("horizon.mix.start_cost")?;
    let paid = printed(&cost)?;
    if csv(at_rate.map(|x| x.fixed(2))) != get("horizon.mix.at_rate")?
        || printed(&at_rate[0].fixed(2))? - paid != printed(&means[0])?
        || [r100[3], r1000[3]]
            .iter()
            .zip(at_rate)
            .any(|(m, r)| (r.to_f64() - m - paid.to_f64()).abs() > 0.005)
    {
        return Err("17_5 says 1.57 less 0.28 is 1.29, a cost paid once".into());
    }
    let rate_cents = mean.cents(2);
    let unsigned = rate_cents.trim_start_matches('+').to_owned();
    // 17_8: the average has grown by the mix's 1.57 cents a round, about one
    // and a half; the spread from about nine dollars to about thirty
    let grown = printed(&means[1])? - printed(&means[0])?;
    let per = grown / Ratio::int((LONG_HORIZON - HORIZON) as i64);
    if per.cents(2) != rate_cents || !near(per, Ratio::new(15, 1000), Ratio::new(1, 1000)) {
        return Err("17_8 says the average kept growing by about one and a half cents a round".into());
    }
    let moms = [model::moments(d100, last), model::moments(d1000, last)];
    let sd = [moms[0].1, moms[1].1];
    if csv(sd.map(|s| format!("{s:.1}"))) != get("horizon.mix.spread")?
        || csv(sd.map(|s| format!("{s:.2}"))) != get("horizon.mix.spread2")?
        || [format!("{:.0}", sd[0]), format!("{:.0}", sd[1])] != ["9", "30"]
    {
        return Err("17_8 says the spread grew from about nine dollars to about thirty".into());
    }
    // 17_9: about 3.16 times the spread, close to the square root of ten
    // and not equal to it: the square-root law is about the long run
    let ratio = sd[1] / sd[0];
    let times = LONG_HORIZON / HORIZON;
    let root = (times as f64).sqrt();
    if format!("{ratio:.2}") != get("horizon.mix.spread.ratio")?
        || format!("{ratio:.4}") != get("horizon.mix.spread.ratio.fine")?
        || format!("{root:.4}") != get("horizon.rounds.sqrt.fine")?
        || format!("{ratio:.2}") != format!("{root:.2}")
        || format!("{ratio:.4}") == format!("{root:.4}")
    {
        return Err("17_9 says close to the square root of ten, not an exact rule".into());
    }
    // 17_10: a crowd from the player's start, 7 dollars on corner 1, game B
    // alone for the player's 1,000 rounds
    if SHORT != LONG_HORIZON || corner_of(START) != 1 {
        return Err("17_10 sets the player's start and rounds against a crowd's".into());
    }
    let (b_up, _, _) = model::categories(&b_dist, last);
    let lucky_from = usize::try_from(last as i64 + short).map_err(|e| e.to_string())?;
    let b_lucky: f64 = b_dist[lucky_from..].iter().sum();
    if csv([b_up, b_lucky].map(pct1)) != get("horizon.long.b.from_start")? {
        return Err("17_10's shares disagree with the facts".into());
    }
    if b_lucky <= 1.0 / 8.0 || b_lucky >= b_up || b_up >= 0.5 {
        return Err("17_10 says lucky, but not rare: more than one in eight did as well".into());
    }
    // 17_11: behind after 1,000 rounds from 0, each game alone and the mix
    let behind = [&a, &b].map(|g| model::categories(&model::snapshots(g, CROWD_START, &[LONG_HORIZON])[0], last).1);
    if pct1(behind[0]) != get("horizon.long.a.behind.pct")?
        || pct1(behind[1]) != get("horizon.long.b.behind.pct")?
        || behind.iter().any(|&x| x <= r1000[1])
    {
        return Err("17_11 says each game alone leaves more of its crowd behind than the mix".into());
    }
    // 17_12, 17_13: the mix's average, about one and a half cents a round
    if !near(mean, Ratio::new(15, 1000), Ratio::new(1, 1000)) {
        return Err("17_13 says about one and a half cents a round".into());
    }
    let after_long = format!("{AFTER} {} {ROUNDS}", thousands(LONG_HORIZON as u64));
    let k08 = format!(
        "exact distribution, not a simulation: a crowd starting at {CROWD_START} dollars, on corner {}, negative \
         totals allowed, plays the mix with the published coins; after {HORIZON} rounds {AHEAD} {} %, {BEHIND} {} \
         %, {LEVEL} {} %",
        corner_of(CROWD_START),
        pct1(r100[0]),
        pct1(r100[1]),
        pct1(r100[2]),
    );
    let k09 = format!(
        "{PROBABILITY_ONE}, one player's average gain per round converges to {mean} dollars, about {unsigned} cents \
         (strong law of large numbers for these games: Ethier & Lee, Electron. J. Probab. 14, 2009)"
    );
    let k10 = format!(
        "spread = standard deviation of the totals: {:.2} dollars at {HORIZON} rounds, {:.2} at {}; ratio {ratio:.4}, \
         square root of {times} = {root:.4}; square-root growth is a long-run law (central limit theorem: Ethier & \
         Lee, 2009)",
        sd[0],
        sd[1],
        thousands(LONG_HORIZON as u64),
    );
    Ok(Numbers {
        badge: badge_text(&f)?,
        run_title: format!("{RUN_TITLE} {short_words} {OF_GAME_B}"),
        end_words: format!("{short:+} {DOLLARS_AFTER} {short_words} {ROUNDS}"),
        b_loses: format!("{GAME_B}: {} {CENTS_A_ROUND}", screen(&b.mean().cents(2))),
        long_words: [
            format!("{ALL} {long_rounds} {ROUNDS}"),
            format!("{} {DOLLARS_AFTER} {long_rounds}", screen(&run.net().to_string())),
        ],
        run,
        run_y,
        setup,
        minus_one: format!("{} {DOLLAR} {STANDS_ON} {}", screen("-1"), corner_of(-1)),
        tallest: (top, top_w),
        tallest_words: format!("{top:+} {DOLLARS}: {}", pct_words(&pct1(top_w))),
        whole: format!("{ALL_BARS} {}", pct_words(&pct1(inside(d100)))),
        corners: [0, 1, 2].map(|i| format!("{CORNER} {i}: {}", pct_words(&pct1(corner_sums[i])))),
        values: [pct1(r100[0]), pct1(r100[1]), pct1(r100[2]), means[0].clone()],
        at_rate: format!(
            "{HORIZON} {ROUNDS_AT} {unsigned} {CENTS} = {} {DOLLARS}",
            at_rate[0].fixed(2)
        ),
        start_cost: format!(
            "{STARTING_ON} {}: {} {DOLLARS}, {ONCE}",
            corner_of(CROWD_START),
            screen(&format!("-{cost}"))
        ),
        question: format!("{after_long}: {AHEAD}?"),
        answer: format!("{after_long}: {AHEAD} {}", pct_words(&pct1(r1000[0]))),
        rate: format!(
            "{} {MORE_ROUNDS}: {} {DOLLARS}, {rate_cents} {CENTS_A_ROUND}",
            LONG_HORIZON - HORIZON,
            grown.signed(2)
        ),
        spread: format!(
            "{SPREAD}: {:.2} {DOLLARS} {AT} {HORIZON} {ROUNDS}, {:.2} {AT} {}",
            sd[0],
            sd[1],
            thousands(LONG_HORIZON as u64)
        ),
        bracket: (moms[1].0 - sd[1], moms[1].0 + sd[1]),
        ratio_words: format!("{times} {TIMES_ROUNDS}, {ratio:.2} {TIMES_SPREAD}"),
        sqrt_tex: format!(r"\sqrt{{{times}}} \approx {root:.2}"),
        b_values: [pct1(b_up), pct1(b_lucky)],
        b_dist,
        lucky: short,
        at_least: format!("{NO_FEWER} {short:+}"),
        player_words: format!("{THE_PLAYER}: {short:+} {DOLLARS}"),
        behind_head: format!("{after_long} {FROM} {CROWD_START} {DOLLARS}, {BEHIND}:"),
        behind_rows: [
            (behind[0], pct_words(&pct1(behind[0]))),
            (behind[1], pct_words(&pct1(behind[1]))),
            (r1000[1], pct_words(&pct1(r1000[1]))),
        ],
        forever: format!("{FOREVER} {TENDS} {rate_cents} {CENTS}, {PROBABILITY_ONE}"),
        summary: [
            format!("{CROWD_AVERAGE} {unsigned} {CENTS_A_ROUND}"),
            format!(
                "{CHANCE_AHEAD} {} {AFTER} {HORIZON} {ROUNDS}, {} {AFTER} {}, {NEVER_CERTAIN}",
                pct_words(&pct1(r100[0])),
                pct_words(&pct1(r1000[0])),
                thousands(LONG_HORIZON as u64)
            ),
            format!("{ONE_PLAYER} {TENDS} {unsigned} {CENTS}, {PROBABILITY_ONE}"),
        ],
        snaps: Rc::new(snaps),
        readings: Rc::new(readings),
        b_chances: chances(&b)?,
        k08,
        k09,
        k10,
    })
}

// --------------------------------------------------------------- pictures

/// 17_1's window: its axes, the ticks at 0 and 20 dollars, the start
/// dashed and marked, the rounds at its foot and the title over it.
/// Detached.
fn run_axes(kit: &Kit, stage: &mut Stage<'_>, frame: &Frame2, title: &str) -> fmn::Result<Vec<Mob>> {
    let (left, right, bottom, top) = (frame.left(), frame.right(), frame.bottom(), frame.top());
    let corner = polyline([v(left, top), v(left, bottom), v(right, bottom)])
        .style(line_style(Palette::dim(), 2.0))
        .build();
    let mut out = vec![stage.arena_mut().add(corner)];
    for t in RUN_TICKS {
        let y = frame.p(0.0, t as f64)[1];
        let m = kit.text(stage, &t.to_string(), SMALL, Palette::dim(), ORIGIN)?;
        stage.move_to(m, v(left - 0.1, y), RIGHT);
        out.push(m);
    }
    let start_y = frame.p(0.0, START as f64)[1];
    out.extend(dashed(
        stage,
        v(left, start_y),
        v(right, start_y),
        0.1,
        palette::player(),
        1.5,
    ));
    let start = kit.text(
        stage,
        &format!("{START_WORD} {START}"),
        SMALL,
        palette::player(),
        ORIGIN,
    )?;
    stage.move_to(start, v(left - 0.1, start_y), RIGHT);
    keep_between(stage, start, -FRAME_X_RADIUS + EDGE, left, "the run's start")?;
    out.push(start);
    out.push(kit.text(stage, "0", SMALL, Palette::dim(), v(left, bottom - TICK_DY))?);
    let end = kit.text(
        stage,
        &format!("{} {ROUNDS}", thousands(SHORT as u64)),
        SMALL,
        Palette::dim(),
        ORIGIN,
    )?;
    stage.move_to(end, v(right, bottom - TICK_DY), RIGHT);
    out.push(end);
    let name = kit.text_left(stage, title, SMALL, Palette::dim(), v(left, top + TICK_DY + 0.08))?;
    keep_between(stage, name, left - 0.01, BADGE_LEFT - 0.2, "the run's title")?;
    out.push(name);
    Ok(out)
}

/// The histogram's furniture: its axes, the dollars ticks and name, the
/// share ticks and name (the share's arrive later).
struct HistAxes {
    axes: Mob,
    x_ticks: Vec<Mob>,
    x_name: Mob,
    y_ticks: Vec<Mob>,
    y_name: Mob,
}

impl HistAxes {
    fn all(&self) -> Vec<Mob> {
        let mut out = vec![self.axes, self.x_name, self.y_name];
        out.extend(&self.x_ticks);
        out.extend(&self.y_ticks);
        out
    }
}

/// The histogram's furniture, detached.
fn hist_axes(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<HistAxes> {
    let h = &HIST;
    let corner = polyline([v(h.left(), h.top()), v(h.left(), h.bottom()), v(h.right(), h.bottom())])
        .style(line_style(Palette::dim(), 2.0))
        .build();
    let axes = stage.arena_mut().add(corner);
    let mut x_ticks = Vec::with_capacity(X_TICKS.len());
    for x in X_TICKS {
        let words = if x > 0 { format!("+{x}") } else { screen(&x.to_string()) };
        let at = h.p(x as f64, 0.0);
        x_ticks.push(kit.text(stage, &words, SMALL, Palette::dim(), v(at[0], at[1] - TICK_DY))?);
    }
    let x_name = kit.text(
        stage,
        WON_OR_LOST,
        SMALL,
        Palette::dim(),
        v(h.center[0], h.bottom() - AXIS_NAME_DY),
    )?;
    let mut y_ticks = Vec::with_capacity(Y_TICKS.len());
    for y in Y_TICKS {
        let m = kit.text(stage, &pct_words(&y.to_string()), SMALL, Palette::dim(), ORIGIN)?;
        stage.move_to(m, v(h.left() - 0.1, h.p(0.0, y as f64 / 100.0)[1]), RIGHT);
        keep_between(stage, m, -FRAME_X_RADIUS + EDGE, h.left(), "a share tick")?;
        y_ticks.push(m);
    }
    let y_name = kit.text_left(
        stage,
        SHARE_OF_CROWD,
        SMALL,
        Palette::dim(),
        v(h.left(), h.top() + 0.25),
    )?;
    Ok(HistAxes {
        axes,
        x_ticks,
        x_name,
        y_ticks,
        y_name,
    })
}

/// A readout row: its label at [`LABEL_X`], its value ending at [`VALUE_X`]
/// and its unit after it, on row `k`. Detached.
fn readout(
    kit: &Kit,
    stage: &mut Stage<'_>,
    k: usize,
    label: &str,
    value: &str,
    unit: &str,
    color: Srgb,
) -> fmn::Result<[Mob; 3]> {
    let y = READ_YS[k];
    let name = kit.text_left(stage, label, WORDS_SIZE, color, v(LABEL_X, y))?;
    let number = kit.text(stage, value, WORDS_SIZE, color, ORIGIN)?;
    stage.move_to(number, v(VALUE_X, y), RIGHT);
    let after = kit.text_left(stage, unit, WORDS_SIZE, Palette::dim(), v(UNIT_X, y))?;
    keep_between(stage, name, LABEL_X - 0.01, VALUE_X - 0.75, "a readout's label")?;
    keep_between(stage, after, UNIT_X - 0.01, FRAME_X_RADIUS - EDGE, "a readout's unit")?;
    Ok([name, number, after])
}

/// A line of the right column, row `k`. Detached.
fn column(kit: &Kit, stage: &mut Stage<'_>, words: &str, k: usize, color: Srgb) -> fmn::Result<Mob> {
    let m = kit.text_left(stage, words, SMALL, color, v(RIGHT_X, COLUMN_YS[k]))?;
    keep_between(
        stage,
        m,
        RIGHT_X - 0.01,
        FRAME_X_RADIUS - EDGE,
        "a line right of the bars",
    )?;
    Ok(m)
}

/// A line of the table at the left (17_11 to 17_13). Detached.
fn table_line(kit: &Kit, stage: &mut Stage<'_>, words: &str, y: f64, color: Srgb) -> fmn::Result<Mob> {
    let m = kit.text_left(stage, words, TABLE_SIZE, color, v(TABLE_X, y))?;
    keep_between(stage, m, TABLE_X - 0.01, FRAME_X_RADIUS - EDGE, "a line of the table")?;
    Ok(m)
}

impl SceneConstruct for WhatWinningPromises {
    fn name(&self) -> &str {
        "17_what_winning_promises"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let n = numbers().map_err(fail)?;
        let (ink, dim, hero) = (Palette::ink(), Palette::dim(), Palette::hero());
        let (teal, gold, red) = (palette::player(), palette::mix(), palette::bad());

        // 17_1: the player's first 1,000 rounds of game B, at their own
        // scale; then all 100,000 in a thumbnail
        let line = say(&kit, stage, "17_1")?;
        let tag = full_badge(&kit, stage, &n.badge, BADGE_LEFT)?;
        let rf = run_frame(n.run_y);
        let axes = run_axes(&kit, stage, &rf, &n.run_title)?;
        let player = token(stage, rf.p(0.0, START as f64));
        let mut arriving = tag.to_vec();
        arriving.extend(&axes);
        arriving.push(player);
        fade_in_all(stage, &arriving, 0.8)?;
        line.wait_for(stage, AT_FIRST_1000.1)?;
        let points = thinned(&n.run.dollars[..=SHORT], RUN_STEP);
        let (path, anchor, progress) = growing_run(stage, &rf, &points, teal, Some(player))?;
        fade_in_all(stage, &[anchor, path], 0.1)?;
        let rt = (line.at(AT_AHEAD_16.1)? - now(stage)).max(RUN_MIN_S);
        draw_run(stage, progress, rt)?;
        stage.clear_updaters(anchor, true);
        line.wait_for(stage, AT_AHEAD_16.1)?;
        let end_y = rf.p(SHORT as f64, n.run.dollars[SHORT] as f64)[1];
        let end = kit.text_left(stage, &n.end_words, WORDS_SIZE, teal, v(rf.right() + END_DX, end_y))?;
        keep_between(stage, end, rf.right(), FRAME_X_RADIUS - EDGE, "the run's end")?;
        fade_in_all(stage, &[end], 0.5)?;
        line.wait_for(stage, AT_EVEN_THOUGH.1)?;
        let loses = kit.text_left(
            stage,
            &n.b_loses,
            SMALL,
            palette::game_b(),
            v(rf.right() + END_DX, end_y - ROW_GAP),
        )?;
        keep_between(stage, loses, rf.right(), FRAME_X_RADIUS - EDGE, "game B's average")?;
        fade_in_all(stage, &[loses], 0.5)?;
        line.wait_for(stage, AT_CAUGHT_UP.1)?;
        let tf = thumb_frame(&n.run);
        let border = stage.arena_mut().add(tf.border(dim)?);
        let whole_run: Vec<Vec3> = thinned(&n.run.dollars, THIN).iter().map(|&(k, d)| tf.p(k, d)).collect();
        let long = stage
            .arena_mut()
            .add(polyline(whole_run).style(line_style(teal, 2.0)).build());
        let all = kit.text_left(
            stage,
            &n.long_words[0],
            SMALL,
            dim,
            v(tf.left(), tf.top() + THUMB_WORDS_DY),
        )?;
        fade_in_all(stage, &[border, all], 0.5)?;
        draw_bands(stage, &[long], 1.2)?;
        line.wait_for(stage, AT_DOWN.1)?;
        let down = kit.text_left(
            stage,
            &n.long_words[1],
            SMALL,
            red,
            v(tf.left(), tf.bottom() - THUMB_WORDS_DY),
        )?;
        keep_between(stage, down, tf.left() - 0.01, FRAME_X_RADIUS - EDGE, "the run's end")?;
        let last_dollars = *n.run.dollars.last().ok_or_else(|| fail("a run has rounds".into()))? as f64;
        let dot = stage.arena_mut().add(
            Circle::new()
                .radius(END_R)
                .arc_center(tf.p(RUN_ROUNDS as f64, last_dollars))
                .style(flat(teal, 1.0))
                .build(),
        );
        fade_in_all(stage, &[down, dot], 0.5)?;
        keep_apart(stage, &[end, loses, all, down, tag[1]], 0.08, "the run's words")?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 17_2: a fixed number of rounds, answered exactly: a crowd from 0
        // dollars on corner 0, its axis, negative totals allowed
        let line = say(&kit, stage, "17_2")?;
        line.wait_for(stage, AT_FIXED_NUMBER.1)?;
        let mut leaving = axes.clone();
        leaving.extend([player, path, anchor, end, loses, border, long, all, down, dot]);
        fade_out_all(stage, &leaving, 0.7)?;
        line.wait_for(stage, AT_CROWD_ZERO.1)?;
        let hist = hist_axes(&kit, stage)?;
        let zero = HIST.p(0.0, 0.0);
        let start_dot = stage.arena_mut().add(
            Circle::new()
                .radius(START_R)
                .arc_center(zero)
                .style(flat(teal, 1.0))
                .build(),
        );
        let start_words = kit.text(stage, EVERYONE, SMALL, teal, v(zero[0], zero[1] + START_DY))?;
        let mut arriving = vec![hist.axes, hist.x_name, start_dot, start_words];
        arriving.extend(&hist.x_ticks);
        fade_in_all(stage, &arriving, 0.7)?;
        line.wait_for(stage, AT_HUNDRED.1)?;
        let setup = kit.text_left(stage, &n.setup[0], WORDS_SIZE, ink, SETUP_AT)?;
        keep_between(stage, setup, SETUP_AT[0] - 0.01, BADGE_LEFT - 0.2, "the crowd's setup")?;
        let rounds_label = kit.text_left(stage, ROUNDS_PLAYED, WORDS_SIZE, dim, v(LABEL_X, READ_YS[0]))?;
        let rounds_count = kit.text(stage, &HORIZON.to_string(), WORDS_SIZE, ink, COUNT_AT)?;
        fade_in_all(stage, &[setup, rounds_label, rounds_count], 0.5)?;
        line.wait_for(stage, AT_BELOW_ZERO.1)?;
        let pointer = frame_around(stage, &hist.x_ticks[..2], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_MINUS_ONE.1)?;
        let minus_x = HIST.p(-1.0, 0.0)[0];
        let minus = kit.text(stage, &n.minus_one, SMALL, ink, ORIGIN)?;
        stage.move_to(minus, v(minus_x - 0.15, zero[1] + MINUS_DY), RIGHT);
        keep_between(stage, minus, HIST.left(), zero[0] - START_R, "minus one's corner")?;
        fade_in_all(stage, &[minus], 0.5)?;
        keep_apart(
            stage,
            &[
                setup,
                start_words,
                minus,
                hist.x_name,
                rounds_label,
                rounds_count,
                tag[1],
            ],
            0.08,
            "the setup's words",
        )?;

        // 17_3: the exact crowd after 100 rounds; what a bar is, parity, the
        // whole crowd, and the corners
        let line = say(&kit, stage, "17_3")?;
        line.wait_for(stage, AT_SPREAD_OUT.1)?;
        fade_out_all(stage, &[start_dot, start_words, minus], 0.4)?;
        let teal_bars = stage.arena_mut().add(bars(&n.snaps[0], |_| true, |_| teal, TEAL_O)?);
        stage.add_to_scene(teal_bars)?;
        let growth = grow_histogram(stage, teal_bars, GROW_LAG)?;
        stage.play(growth.with_run_time(GROW_S))?;
        line.wait_for(stage, AT_EACH_BAR.1)?;
        let (top, top_w) = n.tallest;
        let top_x = top as f64;
        let top_box = slot(stage, top_x - BAR_W / 2.0, top_x + BAR_W / 2.0, top_w)?;
        let tallest = kit.text(stage, &n.tallest_words, SMALL, ink, ORIGIN)?;
        stage.move_to(tallest, v(HIST.p(top_x - 1.0, 0.0)[0] - 0.1, LABEL_Y), RIGHT);
        fade_in_all(stage, &[tallest], 0.4)?;
        let pointer = frame_around(stage, &[top_box], 0.0, hero)?;
        point_at(stage, pointer, &[], 0.9)?;
        line.wait_for(stage, AT_HEIGHT.1)?;
        let mut shares = hist.y_ticks.clone();
        shares.push(hist.y_name);
        fade_in_all(stage, &shares, 0.6)?;
        line.wait_for(stage, AT_EVEN.1)?;
        let odd = kit.text_left(
            stage,
            ODD_TOTALS,
            SMALL,
            dim,
            v(HIST.p(top_x + 2.0, 0.0)[0] + 0.1, LABEL_Y),
        )?;
        keep_between(
            stage,
            odd,
            HIST.p(top_x, 0.0)[0],
            RIGHT_X - 0.1,
            "the odd totals' words",
        )?;
        fade_in_all(stage, &[odd], 0.4)?;
        let d100 = &n.snaps[0];
        let at = |net: i64| d100[usize::try_from(net + LONG_HORIZON as i64).unwrap_or(0)];
        for gap in [top - 1, top - 3] {
            let x = gap as f64;
            let empty = slot(stage, x - BAR_W / 2.0, x + BAR_W / 2.0, at(gap - 1).min(at(gap + 1)))?;
            let pointer = frame_around(stage, &[empty], 0.0, hero)?;
            point_at(stage, pointer, &[], 0.6)?;
        }
        line.wait_for(stage, AT_WHOLE.1)?;
        let whole = column(&kit, stage, &n.whole, 0, ink)?;
        fade_in_all(stage, &[whole], 0.5)?;
        line.wait_for(stage, AT_CORNERS.1)?;
        let mut outlines = Vec::new();
        for (net, w) in drawn(d100) {
            if corner_of(CROWD_START + net) == 2 && w >= OUTLINE_MIN {
                let x = net as f64;
                let (dl, ur) = (HIST.p(x - BAR_W / 2.0, 0.0), HIST.p(x + BAR_W / 2.0, w));
                let r = rect(dl, ur[0] - dl[0], ur[1] - dl[1], line_style(hero, OUTLINE_W))?;
                outlines.push(stage.arena_mut().add(r));
            }
        }
        fade_in_all(stage, &outlines, 0.5)?;
        line.wait_for(stage, AT_ADD_UP.1)?;
        let added = column(&kit, stage, ADDED_UP, 1, dim)?;
        let mut sums = vec![added];
        for (k, words) in n.corners.iter().enumerate() {
            sums.push(column(&kit, stage, words, k + 2, teal)?);
        }
        fade_in_all(stage, &sums, 0.6)?;
        line.wait_for(stage, AT_MOST.1)?;
        let pointer = frame_around(stage, &sums[3..], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        let mut words = vec![tallest, odd, whole, hist.y_name, setup];
        words.extend(&sums);
        keep_apart(stage, &words, 0.06, "the bars' words")?;
        kit.hold(stage)?;
        pause(stage, INSPECT_S)?;
        fade_out_all(stage, &outlines, 0.4)?;

        // 17_4: colored by where each finished, a readout as each is said,
        // and the "precisely" card
        let line = say(&kit, stage, "17_4")?;
        line.wait_for(stage, AT_COLOR.1)?;
        let mut leaving = vec![tallest, odd, whole];
        leaving.extend(&sums);
        fade_out_all(stage, &leaving, 0.5)?;
        let d100 = &n.snaps[0];
        let groups = [
            bars(d100, |net| net > 0, category_color, 1.0)?,
            bars(d100, |net| net < 0, category_color, 1.0)?,
            bars(d100, |net| net == 0, category_color, 1.0)?,
        ]
        .map(|g| stage.arena_mut().add(g));
        let names = [AHEAD, BEHIND, LEVEL];
        let colors = [gold, red, dim];
        let mut rows = Vec::with_capacity(4);
        let mut card = None;
        for (k, at) in [AT_AHEAD, AT_BEHIND, AT_LEVEL].iter().enumerate() {
            line.wait_for(stage, at.1)?;
            let row = readout(&kit, stage, k + 1, names[k], &n.values[k], PERCENT, colors[k])?;
            let mut arriving = vec![groups[k]];
            arriving.extend(row);
            let k08 = if k == 2 {
                Some(precisely(&kit, stage, &n.k08)?)
            } else {
                None
            };
            arriving.extend(k08);
            fade_in_all(stage, &arriving, 0.6)?;
            if let Some(c) = k08 {
                card = Some((c, now(stage)));
            }
            rows.push(row);
        }
        stage.scene_mut().remove(&[teal_bars]);
        let mut readouts: Vec<Mob> = rows.iter().flatten().copied().collect();
        readouts.extend([rounds_label, rounds_count, tag[1]]);
        keep_apart(stage, &readouts, 0.06, "the readouts")?;
        kit.hold(stage)?;
        let (card, card_up) = card.ok_or_else(|| fail("17_4's card".into()))?;
        wait_until(stage, card_up + read_s(&n.k08))?;
        fade_out_all(stage, &[card], 0.4)?;

        // 17_5: the crowd's average, the rate and the start's cost; what
        // the average hides
        let line = say(&kit, stage, "17_5")?;
        line.wait_for(stage, AT_AVERAGE.1)?;
        let avg_line = stage.arena_mut().add(average_line(n.readings[0][3]));
        let avg_row = readout(&kit, stage, 4, AVERAGE, &n.values[3], DOLLARS, ink)?;
        let mut arriving = vec![avg_line];
        arriving.extend(avg_row);
        fade_in_all(stage, &arriving, 0.6)?;
        rows.push(avg_row);
        line.wait_for(stage, AT_AT_RATE.1)?;
        let at_rate = column(&kit, stage, &n.at_rate, 1, gold)?;
        fade_in_all(stage, &[at_rate], 0.5)?;
        line.wait_for(stage, AT_START_COST.1)?;
        let cost = column(&kit, stage, &n.start_cost, 2, red)?;
        fade_in_all(stage, &[cost], 0.5)?;
        line.wait_for(stage, AT_HIDES.1)?;
        let pointer = frame_around(stage, &[groups[1]], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_TWO_IN_FIVE.1)?;
        let pointer = frame_around(stage, &rows[1], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        let mut checked = readouts.clone();
        checked.extend(avg_row);
        checked.extend([at_rate, cost]);
        keep_apart(stage, &checked, 0.06, "the average's words")?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 17_6: a thousand rounds: ahead? (a prediction)
        let line = say(&kit, stage, "17_6")?;
        line.wait_for(stage, AT_MORE_OR_LESS.1)?;
        fade_out_all(stage, &[at_rate, cost], 0.4)?;
        let ask = column(&kit, stage, &n.question, 1, ink)?;
        fade_in_all(stage, &[ask], 0.4)?;
        let q = think(&kit, stage, THINK17)?;

        // 17_7: more; then the same bars on the same axis, from 100 rounds
        // to 1,000, one exact crowd in every frame
        let line = say(&kit, stage, "17_7")?;
        line.wait_for(stage, AT_SIXTY_EIGHT.1)?;
        let answer = column(&kit, stage, &n.answer, 1, gold)?;
        let anims = swap(stage, &[ask, q], &[answer], 0.5)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_WATCH_BARS.1)?;
        // Finish the invitation before the ten-second inspection experiment.
        kit.hold(stage)?;
        let rounds_t = stage.add_value_tracker(HORIZON as f64);
        let snaps = Rc::clone(&n.snaps);
        let live_bars = stage.always_redraw(move |s| {
            let k = snapshot_at(s.tracker_value(rounds_t).unwrap_or(HORIZON as f64));
            s.add(bars(&snaps[k], |_| true, category_color, 1.0).expect("bars of finite size"))
        });
        let readings = Rc::clone(&n.readings);
        let live_line = stage.always_redraw(move |s| {
            let k = snapshot_at(s.tracker_value(rounds_t).unwrap_or(HORIZON as f64));
            s.add(average_line(readings[k][3]))
        });
        stage.add_to_scene(live_bars)?;
        stage.add_to_scene(live_line)?;
        let mut live = Vec::with_capacity(4);
        for (j, color) in [gold, red, dim, ink].into_iter().enumerate() {
            let readings = Rc::clone(&n.readings);
            let (places, sign, scale) = if j == 3 { (2, true, 1.0) } else { (1, false, 100.0) };
            live.push(kit.live_value(
                stage,
                move |s| {
                    let k = snapshot_at(s.tracker_value(rounds_t).unwrap_or(HORIZON as f64));
                    scale * readings[k][j]
                },
                places,
                sign,
                WORDS_SIZE,
                color,
                v(VALUE_X, READ_YS[j + 1]),
                RIGHT,
            )?);
        }
        let live_rounds = kit.live_count(stage, rounds_t, WORDS_SIZE, ink, COUNT_AT)?;
        let mut statics = groups.to_vec();
        statics.extend([avg_line, rounds_count]);
        statics.extend(rows.iter().map(|r| r[1]));
        stage.scene_mut().remove(&statics);
        let morph = rounds_t
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(MORPH_S),
                rate_func: Some(stepped),
                ..AnimateArgs::default()
            })?
            .set_value(LONG_HORIZON as f64)?;
        stage.play(morph)?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 17_8: behind and level after 1,000 rounds; the average's growth;
        // the spread
        let line = say(&kit, stage, "17_8")?;
        line.wait_for(stage, AT_BEHIND_28.1)?;
        let row_at = |k: usize| [rows[k][0], live[k], rows[k][2]];
        let pointer = frame_around(stage, &row_at(1), POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_LEVEL_2.1)?;
        let pointer = frame_around(stage, &row_at(2), POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_KEPT_GROWING.1)?;
        let rate = column(&kit, stage, &n.rate, 2, ink)?;
        fade_in_all(stage, &[rate], 0.5)?;
        let pointer = frame_around(stage, &row_at(3), POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_SPREAD_GROWN.1)?;
        let (from, to) = (HIST.p(n.bracket.0, 0.0)[0], HIST.p(n.bracket.1, 0.0)[0]);
        let bracket = [
            polyline([v(from, BRACKET_Y), v(to, BRACKET_Y)]),
            polyline([v(from, BRACKET_Y - BRACKET_TICK), v(from, BRACKET_Y + BRACKET_TICK)]),
            polyline([v(to, BRACKET_Y - BRACKET_TICK), v(to, BRACKET_Y + BRACKET_TICK)]),
        ]
        .map(|p| stage.arena_mut().add(p.style(line_style(ink, 3.0)).build()));
        let spread = column(&kit, stage, &n.spread, 3, ink)?;
        draw_bands(stage, &bracket, 0.8)?;
        fade_in_all(stage, &[spread], 0.5)?;
        let mut checked = vec![rounds_label, live_rounds, answer, rate, spread, tag[1]];
        for (k, row) in rows.iter().enumerate() {
            checked.extend([row[0], live[k], row[2]]);
        }
        keep_apart(stage, &checked, 0.05, "the readouts after 1,000 rounds")?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 17_9: ten times the rounds, about 3.16 times the spread; K10
        let line = say(&kit, stage, "17_9")?;
        line.wait_for(stage, AT_TEN_TIMES.1)?;
        let ratio = column(&kit, stage, &n.ratio_words, 2, ink)?;
        let anims = swap(stage, &[rate], &[ratio], 0.5)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_SQRT_TEN.1)?;
        let root = kit.tex_left(stage, &n.sqrt_tex, TEX_SIZE, ink, &[], v(RIGHT_X, COLUMN_YS[4]))?;
        keep_between(stage, root, RIGHT_X - 0.01, FRAME_X_RADIUS - EDGE, "the square root")?;
        fade_in_all(stage, &[root], 0.5)?;
        line.wait_for(stage, AT_LONG_RUNS.1)?;
        let card = precisely(&kit, stage, &n.k10)?;
        fade_in_all(stage, &[card], 0.5)?;
        let card_up = now(stage);
        keep_apart(stage, &[answer, ratio, spread, root], 0.05, "the ratio's words")?;
        kit.hold(stage)?;
        wait_until(stage, card_up + read_s(&n.k10))?;
        fade_out_all(stage, &[card], 0.4)?;

        // 17_10: the player's +16 against a crowd from the player's start,
        // game B alone for 1,000 rounds
        let line = say(&kit, stage, "17_10")?;
        line.wait_for(stage, AT_PLAYERS_16.1)?;
        let mut leaving = vec![live_bars, live_line, answer, ratio, spread, root];
        leaving.extend(bracket);
        leaving.extend(&live);
        for row in &rows {
            leaving.extend([row[0], row[2]]);
        }
        fade_out_all(stage, &leaving, 0.6)?;
        let mark_x = HIST.p(n.lucky as f64, 0.0)[0];
        let mark = dashed(stage, v(mark_x, HIST.bottom()), v(mark_x, HIST.top()), 0.1, teal, 2.5);
        let mark_words = kit.text(stage, &n.player_words, SMALL, teal, v(mark_x, HIST.top() + MARK_DY))?;
        let mut arriving = mark.clone();
        arriving.push(mark_words);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, AT_START_SEVEN.1)?;
        let setup_b = kit.text_left(stage, &n.setup[1], WORDS_SIZE, ink, SETUP_AT)?;
        keep_between(
            stage,
            setup_b,
            SETUP_AT[0] - 0.01,
            BADGE_LEFT - 0.2,
            "the crowd's setup",
        )?;
        let anims = swap(stage, &[setup], &[setup_b], 0.6)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_B_ALONE.1)?;
        let b_bars = stage.arena_mut().add(bars(&n.b_dist, |_| true, category_color, 1.0)?);
        stage.add_to_scene(b_bars)?;
        let growth = grow_histogram(stage, b_bars, GROW_LAG)?;
        stage.play(growth.with_run_time(GROW_S))?;
        line.wait_for(stage, AT_B_AHEAD.1)?;
        let b_ahead = readout(&kit, stage, 1, AHEAD, &n.b_values[0], PERCENT, gold)?;
        fade_in_all(stage, &b_ahead, 0.5)?;
        line.wait_for(stage, AT_AT_LEAST.1)?;
        let lucky: Vec<(i64, f64)> = drawn(&n.b_dist)
            .into_iter()
            .filter(|&(net, w)| net >= n.lucky && w >= SEEN)
            .collect();
        let (last_net, tallest_lucky) = (
            lucky.last().map_or(n.lucky, |p| p.0),
            lucky.iter().map(|p| p.1).fold(0.0, f64::max),
        );
        let lucky_box = slot(
            stage,
            n.lucky as f64 - BAR_W / 2.0,
            last_net as f64 + BAR_W / 2.0,
            tallest_lucky,
        )?;
        let b_lucky = readout(&kit, stage, 2, &n.at_least, &n.b_values[1], PERCENT, gold)?;
        fade_in_all(stage, &b_lucky, 0.5)?;
        let pointer = frame_around(stage, &[lucky_box], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.9)?;
        line.wait_for(stage, AT_NOT_RARE.1)?;
        let pointer = frame_around(stage, &[mark_words], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        let mut checked = b_ahead.to_vec();
        checked.extend(b_lucky);
        checked.extend([rounds_label, live_rounds, mark_words, setup_b, hist.y_name, tag[1]]);
        keep_apart(stage, &checked, 0.06, "game B's readouts")?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 17_11: behind after 1,000 rounds from 0: each game alone, the mix
        let line = say(&kit, stage, "17_11")?;
        line.wait_for(stage, AT_FROM_ZERO.1)?;
        let mut leaving = hist.all();
        leaving.extend(mark);
        leaving.extend([mark_words, setup_b, b_bars, rounds_label, live_rounds]);
        leaving.extend(b_ahead);
        leaving.extend(b_lucky);
        fade_out_all(stage, &leaving, 0.6)?;
        let head = table_line(&kit, stage, &n.behind_head, BEHIND_HEAD_Y, dim)?;
        fade_in_all(stage, &[head], 0.5)?;
        let row_names = [
            format!("{GAME_A} {ALONE}"),
            format!("{GAME_B} {ALONE}"),
            MIX_ROW.to_owned(),
        ];
        let row_colors = [palette::game_a(), palette::game_b(), gold];
        let mut table = vec![head];
        for (k, at) in [AT_A_BEHIND, AT_B_BEHIND, AT_MIX_BEHIND].iter().enumerate() {
            line.wait_for(stage, at.1)?;
            let y = BEHIND_YS[k];
            let name = table_line(&kit, stage, &row_names[k], y, row_colors[k])?;
            let (share, words) = &n.behind_rows[k];
            let len = BEHIND_U * *share;
            let bar = rect(v(BEHIND_X, y - BEHIND_H / 2.0), len, BEHIND_H, flat(red, 1.0))?;
            let bar = stage.arena_mut().add(bar);
            let value = kit.text_left(stage, words, TABLE_SIZE, red, v(BEHIND_X + len + 0.15, y))?;
            keep_between(stage, name, TABLE_X - 0.01, BEHIND_X - 0.1, "a crowd's name")?;
            grow(stage, bar, LEFT, BAR_S, &[name, value])?;
            table.extend([name, bar, value]);
        }
        keep_apart(stage, &table, 0.05, "the behind table")?;

        // 17_12: the very long run, with probability one; K09
        let line = say(&kit, stage, "17_12")?;
        line.wait_for(stage, AT_PROBABILITY_ONE.1)?;
        let forever = table_line(&kit, stage, &n.forever, FOREVER_Y, gold)?;
        let card = precisely(&kit, stage, &n.k09)?;
        fade_in_all(stage, &[forever, card], 0.6)?;
        let card_up = now(stage);
        line.wait_for(stage, AT_NOT_CROWD.1)?;
        let pointer = frame_around(stage, &[forever], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        let mut checked = table.clone();
        checked.push(forever);
        keep_apart(stage, &checked, 0.05, "the long run's line")?;
        kit.hold(stage)?;
        wait_until(stage, card_up + read_s(&n.k09))?;
        fade_out_all(stage, &[card], 0.4)?;

        // 17_13: three different promises
        let line = say(&kit, stage, "17_13")?;
        line.wait_for(stage, AT_THREE_THINGS.1)?;
        let mut leaving = table;
        leaving.push(forever);
        fade_out_all(stage, &leaving, 0.6)?;
        let summary_colors = [gold, ink, teal];
        let mut summary = Vec::with_capacity(3);
        for (k, at) in [AT_CROWD_AVG, AT_CHANCE, AT_OWN_AVERAGE].iter().enumerate() {
            line.wait_for(stage, at.1)?;
            let m = table_line(&kit, stage, &n.summary[k], SUMMARY_YS[k], summary_colors[k])?;
            fade_in_all(stage, &[m], 0.5)?;
            summary.push(m);
            if k == 1 {
                line.wait_for(stage, AT_NEVER_CERTAIN.1)?;
                let pointer = frame_around(stage, &[m], POINT_PAD, hero)?;
                point_at(stage, pointer, &[], 0.8)?;
            }
        }
        keep_apart(stage, &summary, 0.1, "the three promises")?;
        kit.hold(stage)?;
        pause(stage, INSPECT_S)?;

        // 17_14: invented games; B's triangle, its red corner 0
        let line = say(&kit, stage, "17_14")?;
        line.wait_for(stage, AT_INVENTED.1)?;
        fade_out_all(stage, &summary, 0.6)?;
        let t = triangle(&kit, stage, Which::B)?;
        let labels = chance_labels(&kit, stage, Which::B, &n.b_chances)?;
        let mut parts = t.parts();
        parts.extend(labels);
        fade_in_all(stage, &parts, 0.8)?;
        line.wait_for(stage, AT_WHERE_STAND.1)?;
        let pointer = frame_around(stage, &[t.discs[0]], POINT_PAD, hero)?;
        point_at(stage, pointer, &[t.discs[0]], 0.9)?;
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
    use fmn::core::constants::FRAME_Y_RADIUS;
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::super::draw::{BADGE_AT, THINK_S};
    use super::*;

    #[test]
    fn the_numbers_are_the_facts() {
        let n = numbers().unwrap();
        assert_eq!(n.run_title, "the player's dollars: the first 1,000 rounds of game B");
        assert_eq!(n.end_words, "+16 dollars after 1,000 rounds");
        assert_eq!(n.b_loses, "game B: \u{2212}0.87 cents a round");
        assert_eq!(
            n.long_words,
            ["all 100,000 rounds", "\u{2212}548 dollars after 100,000"]
        );
        assert_eq!(n.run_y, (-10, 35));
        assert_eq!(
            n.setup,
            [
                "a crowd starts at 0 dollars, on corner 0, and plays the mix",
                "a crowd starts at 7 dollars, on corner 1, and plays game B alone",
            ]
        );
        assert_eq!(n.minus_one, "\u{2212}1 dollar stands on corner 2");
        assert_eq!(n.tallest_words, "+2 dollars: 10.2 %");
        assert_eq!(n.whole, "all bars together: 100.0 %");
        assert_eq!(n.corners, ["corner 0: 34.5 %", "corner 1: 25.4 %", "corner 2: 40.1 %"]);
        assert_eq!(n.values, ["51.7", "39.6", "8.7", "+1.29"]);
        assert_eq!(n.at_rate, "100 rounds at 1.57 cents = 1.57 dollars");
        assert_eq!(n.start_cost, "starting on corner 0: \u{2212}0.28 dollars, once");
        assert_eq!(n.question, "after 1,000 rounds: ahead?");
        assert_eq!(n.answer, "after 1,000 rounds: ahead 68.9 %");
        assert_eq!(n.rate, "900 more rounds: +14.13 dollars, +1.57 cents a round");
        assert_eq!(n.spread, "spread: 9.35 dollars at 100 rounds, 29.55 at 1,000");
        assert_eq!(n.ratio_words, "10 times the rounds, 3.16 times the spread");
        assert_eq!(n.sqrt_tex, r"\sqrt{10} \approx 3.16");
        assert_eq!(n.b_values, ["34.5", "14.6"]);
        assert_eq!((n.lucky, n.at_least.as_str()), (16, "at least +16"));
        assert_eq!(n.player_words, "the player: +16 dollars");
        assert_eq!(n.behind_head, "after 1,000 rounds from 0 dollars, behind:");
        assert_eq!(n.behind_rows.clone().map(|r| r.1), ["61.2 %", "63.9 %", "28.7 %"]);
        assert_eq!(
            n.forever,
            "one player, playing forever: average per round \u{2192} +1.57 cents, with probability one"
        );
        assert_eq!(
            n.summary,
            [
                "the crowd's average: grows 1.57 cents a round",
                "chance of being ahead: 51.7 % after 100 rounds, 68.9 % after 1,000, never certain",
                "one player's average per round: \u{2192} 1.57 cents, with probability one",
            ]
        );
        assert_eq!(
            n.k08,
            "exact distribution, not a simulation: a crowd starting at 0 dollars, on corner 0, negative totals \
             allowed, plays the mix with the published coins; after 100 rounds ahead 51.7 %, behind 39.6 %, level \
             8.7 %"
        );
        assert_eq!(
            n.k09,
            "with probability one, one player's average gain per round converges to 223/14200 dollars, about 1.57 \
             cents (strong law of large numbers for these games: Ethier & Lee, Electron. J. Probab. 14, 2009)"
        );
        assert_eq!(
            n.k10,
            "spread = standard deviation of the totals: 9.35 dollars at 100 rounds, 29.55 at 1,000; ratio 3.1620, \
             square root of 10 = 3.1623; square-root growth is a long-run law (central limit theorem: Ethier & Lee, \
             2009)"
        );
        assert_eq!((n.snaps.len(), n.readings.len()), (STEPS + 1, STEPS + 1));
    }

    /// The bars are whole crowds on even totals: every snapshot holds the
    /// whole crowd, all but 0.01 % of it inside the window, none on an odd
    /// total; 101 bars after 100 rounds (every even total from −100 to
    /// +100) and 121 after 1,000 (to the window's +140); the categories
    /// agree with `model::horizon`, a second exact programme. Failed
    /// controls: a crowd started at 1 dollar (corner 1, away from the bad
    /// coin) is not 51.7 % ahead; after an odd number of rounds every total
    /// is odd; after one round only 29.5 % are ahead, so "always better than
    /// even" is false; single bars are not ordered by corner, so the voice
    /// speaks only of the corners' sums; the spread's ratio is not √10.
    #[test]
    fn the_bars_are_whole_crowds_on_even_totals() {
        let n = numbers().unwrap();
        let mix = PUBLISHED.mix();
        for d in n.snaps.iter() {
            let bars = drawn(d);
            assert!(bars.iter().all(|&(net, _)| net % 2 == 0));
            assert!(bars.iter().map(|p| p.1).sum::<f64>() > 1.0 - WINDOW_LOSS);
        }
        assert_eq!(drawn(&n.snaps[0]).len(), 101);
        assert_eq!(drawn(&n.snaps[STEPS]).len(), 121);
        for (k, rounds) in [(0, HORIZON), (STEPS, LONG_HORIZON)] {
            let (up, down, level) = model::horizon(&mix, rounds, CROWD_START);
            let r = n.readings[k];
            assert!((r[0] - up).abs() < 1e-12 && (r[1] - down).abs() < 1e-12 && (r[2] - level).abs() < 1e-12);
        }
        // failed controls
        assert_ne!(pct1(model::horizon(&mix, HORIZON, 1).0), n.values[0]);
        // after 101 rounds entry k is the total k − 101: even entries are odd
        // totals, and they hold the whole crowd
        let odd = model::snapshots(&mix, CROWD_START, &[HORIZON + 1]).remove(0);
        assert!(odd.iter().skip(1).step_by(2).all(|&w| w == 0.0));
        assert!((odd.iter().step_by(2).sum::<f64>() - 1.0).abs() < 1e-9);
        let one = model::snapshots(&mix, CROWD_START, &[1]).remove(0);
        let (up, _, _) = model::categories(&one, 1);
        assert!((up - mix.chance(0).to_f64()).abs() < 1e-15 && up < 0.5);
        let d100 = &n.snaps[0];
        let at = |net: i64| d100[usize::try_from(net + LONG_HORIZON as i64).unwrap()];
        let unordered = drawn(d100)
            .iter()
            .any(|&(net, w)| corner_of(CROWD_START + net) == 2 && w > 0.002 && (w < at(net - 2) || w < at(net + 2)));
        assert!(unordered, "every corner-2 bar towers over its neighbours");
        let (sd100, sd1000) = (
            model::moments(d100, LONG_HORIZON).1,
            model::moments(&n.snaps[STEPS], LONG_HORIZON).1,
        );
        assert!(((sd1000 / sd100) - 10f64.sqrt()).abs() > 1e-4);
    }

    /// The counter and the bars read one round in every frame: the stepped
    /// rate puts the tracker on even whole rounds only, and the snapshot it
    /// picks is that round's.
    #[test]
    fn the_counter_and_the_bars_show_the_same_round() {
        let (lo, hi) = (HORIZON as f64, LONG_HORIZON as f64);
        assert!(stepped(0.0) == 0.0 && stepped(1.0) == 1.0);
        for i in 0..=6000 {
            let value = lo + (hi - lo) * stepped(f64::from(i) / 6000.0);
            let shown = value.round();
            assert!((value - shown).abs() < 1e-9, "{value}");
            assert!((shown as usize).is_multiple_of(2), "{value}");
            assert_eq!(HORIZON + MORPH_STEP * snapshot_at(value), shown as usize);
        }
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

    /// The layout keeps its rooms: the run's window, its words and the
    /// thumbnail inside the frame; the histogram's names over the card; the
    /// readouts under the badge; the right column, the bracket and the
    /// countdown clear of every bar they stand beside, in every frame of the
    /// transformation; game B's bars under the player's mark.
    #[test]
    fn the_layout_keeps_its_rooms() {
        const {
            let card_top = -FRAME_Y_RADIUS + 0.2 + 0.3 + 3.0 * 0.36;
            assert!(RUN_CENTER[0] - RUN_W / 2.0 - 0.9 > -FRAME_X_RADIUS + EDGE);
            assert!(RUN_CENTER[1] - RUN_H / 2.0 - TICK_DY - 0.15 > -FRAME_Y_RADIUS + 0.3);
            assert!(RUN_CENTER[1] + RUN_H / 2.0 + TICK_DY + 0.25 < BADGE_AT[1] - 0.19);
            assert!(RUN_CENTER[0] + RUN_W / 2.0 + 0.3 < THUMB_CENTER[0] - THUMB_W / 2.0);
            assert!(THUMB_CENTER[0] + THUMB_W / 2.0 < FRAME_X_RADIUS - EDGE);
            assert!(THUMB_CENTER[1] - THUMB_H / 2.0 - THUMB_WORDS_DY - 0.15 > -FRAME_Y_RADIUS + 0.3);
            assert!(HIST.center[1] + HIST.height / 2.0 + 0.25 + 0.15 < SETUP_AT[1] - 0.15);
            assert!(SETUP_AT[1] + 0.15 < FRAME_Y_RADIUS - 0.3);
            assert!(HIST.center[1] - HIST.height / 2.0 - AXIS_NAME_DY - 0.15 > card_top + 0.05);
            assert!(HIST.center[0] - HIST.width / 2.0 - 0.6 > -FRAME_X_RADIUS + EDGE);
            assert!(HIST.center[0] + HIST.width / 2.0 < FRAME_X_RADIUS - EDGE);
            assert!(READ_YS[0] + 0.15 < BADGE_AT[1] - 0.19);
            assert!(READ_YS[4] - 0.15 > COLUMN_YS[1] + 0.15);
            assert!(LABEL_X > RIGHT_X + 1.5);
            assert!(THINK17[1] + 0.62 < COLUMN_YS[1] - 0.15);
            assert!(THINK17[1] - 0.62 - 0.3 - 0.15 > HIST.center[1] - HIST.height / 2.0 + 0.1);
            assert!(THINK17[0] + 1.1 < FRAME_X_RADIUS - EDGE);
            assert!(BEHIND_HEAD_Y + 0.15 < BADGE_AT[1] - 0.19);
            assert!(BEHIND_X + BEHIND_U + 0.15 + 1.0 < FRAME_X_RADIUS - EDGE);
            assert!(BEHIND_YS[2] - FOREVER_Y >= 0.8 && FOREVER_Y - 0.15 > card_top + 0.1);
            assert!(SUMMARY_YS[0] + 0.15 < BADGE_AT[1] - 0.19);
            assert!(SUMMARY_YS[2] - 0.15 > -FRAME_Y_RADIUS + 0.3);
        }
        let n = numbers().unwrap();
        // the run's end words right of the token, the thumbnail's words
        // under them
        let rf = run_frame(n.run_y);
        let end_y = rf.p(SHORT as f64, n.run.dollars[SHORT] as f64)[1];
        const { assert!(END_DX > super::super::draw::TOKEN_R + 0.1) };
        let tf = thumb_frame(&n.run);
        assert!(end_y - ROW_GAP - 0.15 > tf.top() + THUMB_WORDS_DY + 0.15, "{end_y}");
        // every frame of the transformation: the bars right of the column
        // stay under its lowest line, the bracket over the last crowd's bars
        for (k, d) in n.snaps.iter().enumerate() {
            for (net, w) in drawn(d) {
                let x = HIST.p(net as f64 + BAR_W / 2.0, w);
                if x[0] > RIGHT_X - 0.05 {
                    let room = if k == 0 { COLUMN_YS[4] } else { COLUMN_YS[4] - 0.25 };
                    assert!(x[1] < room, "{net} after {} rounds: {}", HORIZON + MORPH_STEP * k, x[1]);
                }
                if x[0] > THINK17[0] - 1.0 && k == 0 {
                    assert!(x[1] < THINK17[1] - 0.62 - 0.45, "{net}");
                }
            }
        }
        let top = drawn(&n.snaps[STEPS]).iter().map(|p| p.1).fold(0.0, f64::max);
        assert!(HIST.p(0.0, top)[1] < BRACKET_Y - BRACKET_TICK - 0.05, "{top}");
        // game B's bars stay under the player's mark's words
        let b_top = drawn(&n.b_dist).iter().map(|p| p.1).fold(0.0, f64::max);
        assert!(HIST.p(0.0, b_top)[1] < HIST.top() - 0.3, "{b_top}");
        // the tallest bar's words stand over the bars they sit beside
        let (top_net, _) = n.tallest;
        for (net, w) in drawn(&n.snaps[0]) {
            if (net - top_net).abs() >= 2 && (net - top_net).abs() <= 30 {
                assert!(HIST.p(0.0, w)[1] < LABEL_Y - 0.15, "{net}: {w}");
            }
        }
    }

    /// The "precisely" cards fit the lower third's three lines (as 15's).
    #[test]
    fn the_cards_fit_three_lines() {
        let n = numbers().unwrap();
        for card in [&n.k08, &n.k09, &n.k10] {
            let lines = explainer_kit::animatic::wrap(card, 76, usize::MAX);
            assert!(lines.len() <= 3, "{lines:?}");
            assert_eq!(&lines.join(" "), card);
        }
    }

    /// The chapter runs headless to its end with the scratch voice and real
    /// typesetting (every `keep_between` and `keep_apart` guard meets the
    /// true text widths): every line of chapter 17 is said once, in order;
    /// the prediction (17_6) gets its silent time; the transformation runs
    /// with the voice silent; the bars' meaning (17_3) stands to be read,
    /// each card for its reading time; the last picture stands.
    #[test]
    fn the_chapter_runs_and_leaves_room_to_think() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = WhatWinningPromises { kit: kit.clone() };
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut NullSceneSink)
            .unwrap_or_else(|e| panic!("17_what_winning_promises: {e}"));
        let said = narrator.said();
        let ids: Vec<&str> = said.iter().map(|(id, _)| id.as_str()).collect();
        let expected: Vec<String> = (1..=14).map(|k| format!("17_{k}")).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        let at = |id: &str| said.iter().find(|(l, _)| l == id).unwrap().1;
        let end_of = |id: &str| at(id) + narrator.length(id).unwrap();
        let think = at("17_7") - end_of("17_6");
        assert!(think >= THINK_S, "{think} s to think");
        let silent = at("17_8") - end_of("17_7");
        assert!(
            silent >= MORPH_S + BEAT,
            "the transformation has {silent} s without the voice"
        );
        assert!(at("17_8") - at("17_7") >= MORPH_S, "the transformation is cut short");
        for (before, after) in [("17_1", "17_2"), ("17_5", "17_6"), ("17_8", "17_9"), ("17_10", "17_11")] {
            let gap = at(after) - end_of(before);
            assert!(gap >= BEAT, "{before}: {gap} s");
        }
        for (before, after) in [("17_3", "17_4"), ("17_13", "17_14")] {
            let gap = at(after) - end_of(before);
            assert!(gap >= INSPECT_S, "{before}: {gap} s");
        }
        let n = numbers().unwrap();
        for (card, line, next) in [
            (&n.k08, "17_4", "17_5"),
            (&n.k10, "17_9", "17_10"),
            (&n.k09, "17_12", "17_13"),
        ] {
            assert!(at(next) - at(line) >= read_s(card), "{line}'s card is not readable");
        }
        let end = kit.finished.borrow().as_ref().expect("a timeline").seconds();
        assert!(
            end - end_of("17_14") >= INSPECT_S,
            "the last picture is not held: {end}"
        );
    }
}
