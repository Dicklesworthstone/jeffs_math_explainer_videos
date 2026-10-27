//! Chapter 08, one player becomes a crowd. The picture 07 left: both
//! triangles with the round-number coins' chances, the badge, the player's
//! token on B's corner 2 with 8 dollars and, between the top corners, 07_7's
//! hypothetical: with equal thirds at every corner B would win 53.3 %. Is it
//! really a third each? (08_1, the hypothetical outlined.) A's triangle and
//! the hypothetical leave; the same token takes one more win along B's
//! 2 → 0 arrow (9 dollars, remainder 0) and stands on corner 0, where it
//! splits into 26 dots: copies of the player, all starting there. Split
//! exactly by the chances, the dots merge into a teal puddle on corner 0 and
//! a bar beside "corner 0"; the dim label "split by the chances (expected
//! numbers)" stays from here on, and the prediction asks where most of the
//! crowd is after one round (08_2). Round 1 moves slowly: a thin stream of
//! 2.6 along 0 → 1 and a thick one of 23.4 back along 0 → 2, labelled past
//! their chances; a "precisely" card (K04) says what an expected number is,
//! and the voice says no player is cut up (08_3). Round 2 carries three
//! quarters of corner 2 around to corner 0; "players: 26" never changes;
//! rounds 3 to 6 slosh at a second each, streams on every arrow (08_4). The
//! share at corner 0 and its trace come in, swinging, with a dashed line at
//! a third; rounds 7 to 12 quicken (08_5), and from round 13 the crowd runs
//! a tenth of a second a round to the first round from which every readout
//! reads its settled value: 10, 4 and 12, and 38.5 % at corner 0; "within
//! half a player from round 24" (08_6, held). Then a small second triangle,
//! "real tosses, seeded illustration": 26 dots from 10, 4, 12, each player
//! tossing in turn from the film's seed. Its first eight rounds hop in
//! silence, the counts jumping, while the split bars stay at 10, 4, 12
//! (08_7); then its counter runs to 1,000 rounds with a live tally of the
//! rounds that land exactly on 10, 4, 12, and the two result lines say this
//! run's 40 of 1,000 and its averages (08_8, held).
//!
//! One model: every bar, puddle, stream, number, total, share and trace
//! point is read from trackers that glide together to `model::evolve_f64`'s
//! counts round by round (exactly the facts' one-decimal values for the six
//! rounds the facts print), so the total is the crowd's size in every frame.
//! The real crowd is every player's toss from `model::SplitMix64` in
//! `model::real_crowd`'s order, and the scene refuses unless its counts are
//! the model's in all 1,000 rounds and the facts' in the eight shown. No
//! typeset text is indicated, scaled or recoloured: a pointed-at label gets
//! an outline (`point_at`), a changed number is replaced (`swap`), and a
//! corner gets a separate hollow ring (`ring_corners`), never a filled
//! pulse of the disc under its dots. The scene
//! ends as 09 starts: B's triangle with 10, 4 and 12 in its puddles and bars.

use std::f64::consts::{FRAC_PI_2, PI, TAU};
use std::rc::Rc;

use explainer_kit::props::precisely;
use explainer_kit::{Facts, Frame2, Kit, Palette, Timed, fade_all, live_bar, thousands, v};
use fmn::core::constants::FRAME_X_RADIUS;
use fmn::prelude::*;

use super::draw::{
    BADGE_AT, badge, dashed, fade_in_all, fade_out_all, fail, flash_all, flat, fly, keep_between, line_style, now,
    pause, read_s, say, stand, think, together, token, wait_until,
};
use super::looks_hopeless::{EDGE, POINT_PAD, ROOM, frame_around, point_at};
use super::three_corners::{
    A_AT, B_AT, CORNER_R, Chances, Which, apex, arrow_ends, chance_labels, chances, corner, corner_of, counter,
    counter_number, edge_of, keep_apart, remainder_line, swap, triangle, walk,
};
use crate::facts::{self, CROWD_ROUNDS, REAL_ROUNDS, REAL_SHOWN, SETTLE_LOOK, SETTLE_WITHIN};
use crate::model::{self, Game, ROUND, Ratio, SEED, SplitMix64};
use crate::palette;

pub struct OnePlayerToACrowd {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const IF_LINE: &str = "with equal thirds at every corner:";
const WOULD_WIN: &str = "B would win";
const ROUND_BADGE: &str = "round-number coins";
const NO_HANDICAP: &str = "no handicap";
const ROUND_WORD: &str = "round";
pub(super) const PLAYERS: &str = "players:";
pub(super) const SPLIT_LABEL: &str = "split by the chances (expected numbers)";
pub(super) const CORNER_WORD: &str = "corner";
const SHARE_WORDS: &str = "share at corner 0";
const PERCENT: &str = "%";
const ROUNDS_AXIS: &str = "rounds";
const REAL_LABEL: &str = "real tosses, seeded illustration";

/// Where each picture lands: (line, the phrase it waits for).
const AT_THIRD_EACH: (&str, &str) = ("08_1", "really a third each");
const AT_LUCK: (&str, &str) = ("08_1", "full of luck");
const AT_SEE_PAST: (&str, &str) = ("08_1", "To see past it");
const AT_COPIES: (&str, &str) = ("08_1", "twenty-six copies");
const AT_ALL_ZERO: (&str, &str) = ("08_1", "all starting at corner zero");
const AT_GAME_B: (&str, &str) = ("08_1", "all playing game B");
const AT_SPLIT: (&str, &str) = ("08_2", "split exactly by the coins' chances");
const AT_AFTER_ONE: (&str, &str) = ("08_2", "After one round");
const AT_CORNER_TWO: (&str, &str) = ("08_3", "At corner two");
const AT_TENTH: (&str, &str) = ("08_3", "A tenth of the crowd");
const AT_NINE_TENTHS: (&str, &str) = ("08_3", "nine tenths fall back");
const AT_NOT_PEOPLE: (&str, &str) = ("08_3", "not fractions of people");
const AT_EXPECTED: (&str, &str) = ("08_3", "are expected numbers");
const AT_NEXT: (&str, &str) = ("08_4", "The next round");
const AT_QUARTERS: (&str, &str) = ("08_4", "three quarters of corner two");
const AT_STAYS: (&str, &str) = ("08_4", "the total stays twenty-six");
const AT_SLOSHES: (&str, &str) = ("08_4", "sloshes between corners");
const AT_WATCH: (&str, &str) = ("08_5", "Watch the share at corner zero");
const AT_BAD_WAITS: (&str, &str) = ("08_5", "where the bad coin waits");
const AT_SWINGS: (&str, &str) = ("08_5", "swings wildly at first");
const AT_STEADIES: (&str, &str) = ("08_5", "steadies above a third");
const AT_TEN: (&str, &str) = ("08_6", "ten players on corner zero");
const AT_FOUR: (&str, &str) = ("08_6", "four on corner one");
const AT_TWELVE: (&str, &str) = ("08_6", "twelve on corner two");
const AT_NOT_THIRD: (&str, &str) = ("08_6", "Not a third each");
const AT_REAL: (&str, &str) = ("08_7", "Real players don't split");
const AT_TOSS: (&str, &str) = ("08_7", "Toss real coins");
const AT_THIS_RUN: (&str, &str) = ("08_8", "In this run of a thousand rounds");
const AT_FORTY: (&str, &str) = ("08_8", "only forty times");
const AT_AVERAGE: (&str, &str) = ("08_8", "on average");
#[cfg(test)]
const CUES: [(&str, &str); 30] = [
    AT_THIRD_EACH,
    AT_LUCK,
    AT_SEE_PAST,
    AT_COPIES,
    AT_ALL_ZERO,
    AT_GAME_B,
    AT_SPLIT,
    AT_AFTER_ONE,
    AT_CORNER_TWO,
    AT_TENTH,
    AT_NINE_TENTHS,
    AT_NOT_PEOPLE,
    AT_EXPECTED,
    AT_NEXT,
    AT_QUARTERS,
    AT_STAYS,
    AT_SLOSHES,
    AT_WATCH,
    AT_BAD_WAITS,
    AT_SWINGS,
    AT_STEADIES,
    AT_TEN,
    AT_FOUR,
    AT_TWELVE,
    AT_NOT_THIRD,
    AT_REAL,
    AT_TOSS,
    AT_THIS_RUN,
    AT_FORTY,
    AT_AVERAGE,
];

// ------------------------------------------------------------------ layout

/// 07_7's column between the two triangles' top corners, as 07 left it.
const MID_X: f64 = (A_AT[0] + B_AT[0]) / 2.0;
const MID_LEFT: f64 = A_AT[0] + CORNER_R + ROOM;
const MID_RIGHT: f64 = B_AT[0] - CORNER_R - ROOM;
const IF_Y: f64 = 2.85;
const SUM_Y: f64 = 2.3;
const WOULD_Y: f64 = 1.7;
/// The left panel, where game A's triangle stood: the round, the crowd's
/// total and the label that says what kind of numbers these are, then one
/// bar row per corner (name, bar, number), the precision note, the share
/// at corner 0 and its trace.
pub(super) const PANEL_X: f64 = -6.75;
const ROUND_Y: f64 = 3.3;
const ROUND_NUM_X: f64 = -5.95;
pub(super) const PLAYERS_Y: f64 = 2.85;
pub(super) const PLAYERS_NUM_X: f64 = -5.75;
pub(super) const LABEL_Y: f64 = 2.45;
pub(super) const ROW_YS: [f64; 3] = [1.75, 1.2, 0.65];
pub(super) const BAR_X: f64 = -5.45;
/// The whole crowd's bar length, scene units.
pub(super) const BAR_FULL: f64 = 3.0;
pub(super) const BAR_H: f64 = 0.3;
pub(super) const NUM_X: f64 = -2.25;
const NOTE_Y: f64 = 0.25;
const SHARE_Y: f64 = -0.2;
const SHARE_NUM_RIGHT: f64 = -4.1;
const SHARE_PCT_X: f64 = -4.0;
/// The share's trace: rounds 0 to the settled round across, 0 % to 100 % up.
const TRACE_LEFT: f64 = -6.3;
const TRACE_RIGHT: f64 = -1.9;
const TRACE_BOTTOM: f64 = -2.65;
const TRACE_TOP: f64 = -0.65;
const THINK_AT: Vec3 = [-3.9, -1.1, 0.0];
/// A round's stream: a teal band along the arrow, its width (pixels) grows
/// with the players it carries.
const STREAM_MIN_W: f64 = 1.5;
const STREAM_MAX_W: f64 = 16.0;
const STREAM_O: f64 = 0.7;
/// Room between a chance label and the flow number past it.
const FLOW_GAP: f64 = 0.12;
/// The crowd's dots: each a teal disc, a cluster of a whole crowd fills a
/// circle of [`CLUSTER_R`] about its corner.
pub(super) const DOT_R: f64 = 0.045;
pub(super) const CLUSTER_R: f64 = 0.36;
/// The real crowd's own small triangle, in the lower left.
const REAL_AT: Vec3 = [-4.0, -1.65, 0.0];
const REAL_R: f64 = 1.25;
const REAL_LABEL_Y: f64 = 0.22;
const REAL_ROUND_Y: f64 = -0.4;
const REAL_ROUND_NUM_X: f64 = -5.6;
const REAL_COUNT_GAP: f64 = 0.15;
const RESULT_YS: [f64; 2] = [-3.05, -3.45];
/// Seconds: round 1's slow move, round 2's, each of rounds 3 to 6 and its
/// streams' fade, rounds 7 to 12, and each round from 13 to the settled one.
const R1_S: f64 = 2.4;
const R2_S: f64 = 1.6;
const ROUND_S: f64 = 0.7;
const STREAM_FADE: f64 = 0.3;
const QUICK_S: f64 = 0.5;
const FAST_S: f64 = 0.1;
/// The rounds played one a second, and from which the crowd runs fast.
const SLOSH_TO: usize = CROWD_ROUNDS;
const QUICK_TO: usize = 12;
/// Seconds: the listen-quiet hold after "Not a third each", one real round's
/// hop and its rest, the thousand rounds' run, the results' hold.
const SETTLE_HOLD: f64 = 2.0;
const HOP_S: f64 = 0.7;
const HOP_REST: f64 = 0.3;
const RUN_S: f64 = 3.0;
const RESULT_HOLD: f64 = 3.0;
const BEAT: f64 = 1.0;

// ---------------------------------------------------------------- numbers

/// The chapter's numbers, checked against the words they illustrate.
struct Numbers {
    /// The player's dollars as 07 left them.
    here: i64,
    round_badge: String,
    a_round: Chances,
    b_round: Chances,
    sum_tex: String,
    /// The crowd's size.
    n: i64,
    k04: String,
    note: String,
    /// The split crowd at the start and after each round, to the settled
    /// round (`last`).
    rounds: Vec<[f64; 3]>,
    last: usize,
    /// Round 1's flows out of corner 0, as the facts print them: forward
    /// and back.
    first_flows: [String; 2],
    /// The settled counts, the settled share at corner 0, a third, and the
    /// note on how soon the crowd is within half a player.
    settled: [String; 3],
    share_settled: String,
    third: String,
    within: String,
    real: Real,
}

/// The real crowd of 08_7 and 08_8, player by player.
struct Real {
    /// Every player's corner at the start and after each round.
    players: Vec<Vec<usize>>,
    counts: Vec<[usize; 3]>,
    /// The rounds so far that land exactly on the start's counts.
    tally: Vec<usize>,
    so_far: String,
    result: [String; 2],
}

fn csv<T: ToString>(values: impl IntoIterator<Item = T>) -> String {
    values.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
}

/// `x` rounded to `places` decimals half away from zero, as the facts print
/// (`Ratio::fixed`), returned as the float of that decimal, so a readout
/// formats it as printed: the float nearest 5.85 lies below it and would
/// format as 5.8, where the facts print 5.9. The nudge only decides values
/// within a billionth of a half, which the scene refuses unless exact.
pub(super) fn shown(x: f64, places: i32) -> f64 {
    let scale = 10f64.powi(places);
    ((x.abs() * scale + 0.5 + 1e-9).floor() / scale).copysign(x)
}

/// One decimal, as a readout writes it.
fn one_decimal(x: f64) -> String {
    format!("{:.1}", shown(x, 1))
}

/// The split crowd from all `n` players on corner 0, at the start and after
/// each of `rounds` rounds of `g`, in floats (`model::evolve_f64`; the exact
/// fractions outgrow `i128` within a few dozen rounds).
fn split_rounds(g: &Game, n: i64, rounds: usize) -> Vec<[f64; 3]> {
    let mut v = [n as f64, 0.0, 0.0];
    let mut out = Vec::with_capacity(rounds + 1);
    out.push(v);
    for _ in 0..rounds {
        v = model::evolve_f64(&v, g);
        out.push(v);
    }
    out
}

/// The share of the crowd at corner 0, in percent.
fn share_pct(v: &[f64; 3], n: i64) -> f64 {
    100.0 * v[0] / n as f64
}

/// The real crowd player by player: `start[i]` players begin on corner i
/// (listed corner by corner) and every round each player in turn tosses
/// from one SplitMix64 stream, a corner forward on a win, one back on a
/// loss; every player's corner at the start and after each round. This is
/// `model::real_crowd`'s order of players and draws; [`real`] refuses unless
/// the counts are the model's, every round.
fn real_players(g: &Game, start: [usize; 3], rounds: usize, seed: u64) -> Vec<Vec<usize>> {
    let mut rng = SplitMix64::new(seed);
    let mut corners: Vec<usize> = (0..3).flat_map(|i| std::iter::repeat_n(i, start[i])).collect();
    let mut out = Vec::with_capacity(rounds + 1);
    out.push(corners.clone());
    for _ in 0..rounds {
        for c in corners.iter_mut() {
            let toss = rng.next_u64() % g.den as u64;
            *c = if toss < g.up[*c] as u64 {
                (*c + 1) % 3
            } else {
                (*c + 2) % 3
            };
        }
        out.push(corners.clone());
    }
    out
}

/// How many of `players` stand on each corner.
fn count(players: &[usize]) -> [usize; 3] {
    let mut c = [0; 3];
    for &p in players {
        c[p] += 1;
    }
    c
}

fn real(f: &Facts, g: &Game, counts: &[Ratio; 3]) -> Result<Real, String> {
    let get = |k: &str| f.get(k).map(str::to_owned).ok_or_else(|| format!("no fact {k}"));
    let mut start = [0usize; 3];
    for (s, c) in start.iter_mut().zip(counts) {
        if !c.is_integer() {
            return Err("08_7's real players start from whole counts".into());
        }
        *s = usize::try_from(c.num()).map_err(|e| format!("a count: {e}"))?;
    }
    let players = real_players(g, start, REAL_ROUNDS, SEED);
    let counts: Vec<[usize; 3]> = players.iter().map(|p| count(p)).collect();
    if counts[1..] != model::real_crowd(g, start, REAL_ROUNDS, SEED)[..] {
        return Err("the real crowd's players disagree with model::real_crowd".into());
    }
    for (k, c) in counts.iter().enumerate().take(REAL_SHOWN + 1).skip(1) {
        if get(&format!("realcrowd.round.{k}"))? != csv(c) {
            return Err(format!("the real crowd's round {k} disagrees with the facts"));
        }
    }
    // 08_7: the counts jump from round to round
    if counts[..=REAL_SHOWN].windows(2).any(|w| w[0] == w[1]) {
        return Err("08_7 says the real counts jump every round shown".into());
    }
    // 08_8: in this run of a thousand rounds, exactly the start forty times
    if REAL_ROUNDS != 1000 || get("realcrowd.rounds")? != REAL_ROUNDS.to_string() {
        return Err("08_8 says a thousand rounds".into());
    }
    let mut tally = Vec::with_capacity(counts.len());
    let mut so_far = 0;
    for (k, c) in counts.iter().enumerate() {
        so_far += usize::from(k > 0 && *c == start);
        tally.push(so_far);
    }
    let exact = tally[REAL_ROUNDS];
    if exact.to_string() != get("realcrowd.exact")? || exact != 40 {
        return Err("08_8 says the real crowd lands on 10, 4, 12 only forty times".into());
    }
    // ...but on average, that's where they sit (these rounds' averages)
    let mean = [0, 1, 2].map(|i| {
        let total: usize = counts[1..].iter().map(|c| c[i]).sum();
        Ratio::new(total as i128, REAL_ROUNDS as i128).fixed(1)
    });
    if csv(&mean) != get("realcrowd.mean")? || mean != [0, 1, 2].map(|i| Ratio::int(start[i] as i64).fixed(1)) {
        return Err("08_8 says the real crowd's averages are where the split crowd sits".into());
    }
    let rounds = thousands(REAL_ROUNDS as u64);
    let [s0, s1, s2] = start;
    Ok(Real {
        players,
        counts,
        tally,
        so_far: format!("exactly {s0}, {s1}, {s2} so far:"),
        result: [
            format!("this run: exactly {s0}, {s1}, {s2} in {exact} of {rounds} rounds"),
            format!("average over the {rounds} rounds: {}", mean.join(", ")),
        ],
    })
}

/// The round-number coins' badge words, from the facts.
pub(super) fn round_badge_text(f: &Facts) -> Result<String, String> {
    let pcts: Vec<String> = facts::coins(&ROUND)
        .iter()
        .map(|x| x.exact_pct())
        .collect::<Result<_, _>>()?;
    let printed = f.get("coins.round.heads.pct").ok_or("no fact coins.round.heads.pct")?;
    if pcts.join(",") != printed {
        return Err("the round-number badge disagrees with the facts".into());
    }
    Ok(format!(
        "{ROUND_BADGE} \u{b7} {} \u{b7} {} \u{b7} {} % \u{b7} {NO_HANDICAP}",
        pcts[0], pcts[1], pcts[2]
    ))
}

fn numbers() -> Result<Numbers, String> {
    let f = facts::facts()?;
    let get = |k: &str| f.get(k).map(str::to_owned).ok_or_else(|| format!("no fact {k}"));
    let g = ROUND.game_b();
    // 08_1: the player on corner 2 with 8 dollars, one win from corner 0
    let here = walk()[3];
    if corner_of(here) != 2 || corner_of(here + 1) != 0 {
        return Err("08_1's player stands on corner 2 and one win takes them to corner 0".into());
    }
    // 07_7's hypothetical, as 07 wrote it
    let to = facts::coins(&ROUND);
    let thirds = facts::thirds_win(&ROUND);
    if thirds.pct(1) != get("round.b.thirds.win.pct")? {
        return Err("07_7's hypothetical disagrees with the facts".into());
    }
    let equals = if thirds.exact_pct().is_ok() { "=" } else { r"\approx" };
    let sum_tex = format!(
        r"\tfrac{{1}}{{3}} \cdot {}\% + \tfrac{{2}}{{3}} \cdot {}\% {equals} {}\%",
        to[1].exact_pct()?,
        to[2].exact_pct()?,
        thirds.pct(1)
    );
    // 08_1: twenty-six copies of the player
    let n = model::smallest_whole_crowd(&g);
    if get("crowd.size")? != n.to_string() || n != 26 {
        return Err("08_1 imagines twenty-six copies of the player".into());
    }
    // 08_3: a tenth of corner 0 forward, nine tenths back; 08_4: three
    // quarters of corner 2 win their way around to corner 0
    if g.chance(0) != Ratio::new(1, 10) || g.chance(2) != Ratio::new(3, 4) || g.chance(1) != g.chance(2) {
        return Err("08_3 and 08_4 say a tenth at corner 0 and three quarters at the others".into());
    }
    let mut rounds = split_rounds(&g, n, SETTLE_LOOK);
    // 08_4: every player is always somewhere, so the total stays the crowd
    for (k, v) in rounds.iter().enumerate() {
        if (v.iter().sum::<f64>() - n as f64).abs() > 1e-9 {
            return Err(format!("the split crowd is not {n} after round {k}"));
        }
    }
    // the rounds the facts print, exactly, and as the readouts show them
    for (k, v) in facts::crowd_rounds(&g, n, CROWD_ROUNDS).iter().enumerate() {
        if v.iter().fold(Ratio::ZERO, |a, &x| a + x) != Ratio::int(n) {
            return Err(format!("round {} loses or gains a player", k + 1));
        }
        let printed = get(&format!("crowd.round.{}", k + 1))?;
        if csv(v.map(|x| x.fixed(1))) != printed || csv(rounds[k + 1].map(one_decimal)) != printed {
            return Err(format!("round {}'s readouts would not show the facts", k + 1));
        }
        if (0..3).any(|i| (rounds[k + 1][i] - v[i].to_f64()).abs() > 1e-9) {
            return Err(format!("round {}'s floats left the exact crowd", k + 1));
        }
    }
    let first = facts::crowd_rounds(&g, n, 1)[0];
    let tenth = Ratio::int(n) * g.chance(0);
    if first != [Ratio::ZERO, tenth, Ratio::int(n) - tenth] {
        return Err("08_3's first round is a tenth forward and the rest back".into());
    }
    let printed_first: Vec<String> = get("crowd.round.1")?.split(',').map(str::to_owned).collect();
    let first_flows = [printed_first[1].clone(), printed_first[2].clone()];
    // 08_6: the crowd settles at 10, 4 and 12, not a third each
    let counts = g.shares().map(|w| Ratio::int(n) * w);
    if get("crowd.counts")? != csv(counts) || csv(counts) != "10,4,12" || counts[0] == Ratio::new(n.into(), 3) {
        return Err("08_6 says ten, four and twelve, not a third each".into());
    }
    let share_settled = g.shares()[0].pct(1);
    if get("crowd.share0.pct")? != share_settled {
        return Err("the settled share at corner 0 disagrees with the facts".into());
    }
    // the settled round: from it on, every readout reads its settled value
    let settled = counts.map(|c| c.to_string());
    let reads_settled = |v: &[f64; 3]| {
        (0..3).all(|i| one_decimal(v[i]) == counts[i].fixed(1)) && one_decimal(share_pct(v, n)) == share_settled
    };
    let last = (1..=SETTLE_LOOK)
        .find(|&l| rounds[l..].iter().all(&reads_settled))
        .ok_or("the readouts never settle")?;
    rounds.truncate(last + 1);
    // past the printed rounds, no readout may sit on a rounding boundary
    for v in &rounds[CROWD_ROUNDS + 1..] {
        for x in [v[0], v[1], v[2], share_pct(v, n)] {
            if ((x * 10.0).fract() - 0.5).abs() < 1e-6 {
                return Err("a readout sits on a rounding boundary".into());
            }
        }
    }
    // 08_5: the share swings across a third at first, then steadies above
    let third = Ratio::new(1, 3);
    let sides: Vec<bool> = rounds[..=CROWD_ROUNDS]
        .iter()
        .map(|v| share_pct(v, n) > 100.0 * third.to_f64())
        .collect();
    if sides.windows(2).any(|w| w[0] == w[1]) || g.shares()[0] <= third {
        return Err("08_5 says the share swings across a third, then steadies above it".into());
    }
    if get("thirds.pct")? != third.pct(1) {
        return Err("a third disagrees with the facts".into());
    }
    // 08_6: within half a player from round 24
    let within = get("crowd.within_half_from")?;
    let target = counts.map(Ratio::to_f64);
    let from = model::settles_from(&g, n as f64, &target, SETTLE_WITHIN, SETTLE_LOOK);
    if from.map(|r| r.to_string()) != Some(within.clone()) || (SETTLE_WITHIN - 0.5).abs() > 1e-12 || from > Some(last) {
        return Err("the half-player note disagrees with the model".into());
    }
    let k04 = format!(
        "expected numbers: {n} times one player's chance of being at that corner after that round; \
         real players come whole and scatter around them"
    );
    Ok(Numbers {
        here,
        round_badge: round_badge_text(&f)?,
        a_round: chances(&ROUND.game_a())?,
        b_round: chances(&g)?,
        sum_tex,
        n,
        k04,
        note: format!("shown to one decimal; the total is exactly {n}"),
        rounds,
        last,
        first_flows,
        settled,
        share_settled,
        third: format!("{} {PERCENT}", third.pct(1)),
        within: format!(
            "within half a player of {}, {}, {} from round {within}",
            counts[0], counts[1], counts[2]
        ),
        real: real(&f, &g, &counts)?,
    })
}

// --------------------------------------------------------------- pictures

/// The round-number coins' badge, outlined, its right edge kept inside the
/// frame, as 07 left it. Detached.
pub(super) fn round_badge(kit: &Kit, stage: &mut Stage<'_>, text: &str) -> fmn::Result<[Mob; 2]> {
    let tag = badge(kit, stage, text, false)?;
    let right = FRAME_X_RADIUS - ROOM;
    if stage.get_bounding_box_point(tag[0], RIGHT)[0] > right {
        stage.move_to(tag[0], v(right, BADGE_AT[1]), RIGHT);
        stage.move_to(tag[1], v(right - 0.15, BADGE_AT[1]), RIGHT);
    }
    keep_between(stage, tag[0], MID_RIGHT - 1.0, right, "the round-number badge")?;
    Ok(tag)
}

/// One of the crowd's players: a small teal disc. Detached.
pub(super) fn dot(stage: &mut Stage<'_>, at: Vec3) -> Mob {
    stage.arena_mut().add(
        Circle::new()
            .radius(DOT_R)
            .arc_center(at)
            .style(flat(palette::player(), 1.0))
            .build(),
    )
}

/// Point at corners with separate hollow rings whose strokes pulse, while
/// `with` (the dots on them) flash, all together. A disc is never flashed
/// itself: its filled pulse would drown the dots and puddle on it (the
/// defect root's native12 frame of 07 at 35.5 s showed for its thirds,
/// repaired there the same way).
pub(super) fn ring_corners(stage: &mut Stage<'_>, discs: &[Mob], with: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut rings = Vec::with_capacity(discs.len());
    let mut anims: Vec<Box<dyn Animation>> = Vec::with_capacity(discs.len() + with.len());
    for &disc in discs {
        let ring = Circle::new()
            .radius(CORNER_R + 0.05)
            .arc_center(stage.get_center(disc))
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

/// Slot `k` of a cluster about `centre` sized for a crowd of `n`: a
/// sunflower, the first slots nearest the centre, so a corner's players
/// gather in the middle of it and a whole crowd fills [`CLUSTER_R`].
pub(super) fn slot(centre: Vec3, k: usize, n: usize) -> Vec3 {
    let golden = PI * (3.0 - 5f64.sqrt());
    let r = CLUSTER_R * ((k as f64 + 0.5) / n.max(1) as f64).sqrt();
    let a = k as f64 * golden;
    v(centre[0] + r * a.cos(), centre[1] + r * a.sin())
}

/// Where players stand, each one's corner given: each corner's players in
/// the order listed take its slots from the middle out.
pub(super) fn places(players: &[usize], at: impl Fn(usize) -> Vec3) -> Vec<Vec3> {
    let mut taken = [0usize; 3];
    players
        .iter()
        .map(|&c| {
            let p = slot(at(c), taken[c], players.len());
            taken[c] += 1;
            p
        })
        .collect()
}

/// The arrow along edge `e` of B's triangle as a curve: the quadratic
/// through its two ends with its middle at the bow's apex, to the same
/// 0.94 of the way `curved_arrow` draws its shaft (the head covers the
/// rest), so a stream lies on the arrow it follows.
fn arrow_curve(e: usize, forward: bool) -> Vec<Vec3> {
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

/// A stream of `flow` players (of a crowd of `n`) along edge `e`: a teal
/// band over the arrow, wider the more it carries. Detached.
fn stream(stage: &mut Stage<'_>, e: usize, forward: bool, flow: f64, n: i64) -> Mob {
    let w = STREAM_MIN_W + (STREAM_MAX_W - STREAM_MIN_W) * flow / n as f64;
    let style = Style::default()
        .fill(palette::player(), 0.0)
        .stroke(palette::player(), w, STREAM_O);
    stage
        .arena_mut()
        .add(polyline(arrow_curve(e, forward)).style(style).build())
}

/// The streams of the round that starts from `from`: from every corner
/// with players, its share forward along its forward arrow and the rest
/// back along its back arrow. Detached.
fn streams(stage: &mut Stage<'_>, from: &[f64; 3], g: &Game, n: i64) -> Vec<Mob> {
    let p = g.chances_f64();
    let mut out = Vec::new();
    for (i, &here) in from.iter().enumerate() {
        if here < 1e-9 {
            continue;
        }
        for (to, flow) in [((i + 1) % 3, here * p[i]), ((i + 2) % 3, here * (1.0 - p[i]))] {
            let (e, forward) = edge_of(i, to);
            out.push(stream(stage, e, forward, flow, n));
        }
    }
    out
}

/// Move `words` past `label` (an arrow's chance), further from the arrow
/// along edge `e`, [`FLOW_GAP`] between them.
fn past(stage: &mut Stage<'_>, label: Mob, e: usize, forward: bool, words: Mob) {
    let a = apex(B_AT, e, forward);
    let c = stage.get_center(label);
    let len = (c[0] - a[0]).hypot(c[1] - a[1]).max(1e-9);
    let d = [(c[0] - a[0]) / len, (c[1] - a[1]) / len];
    let reach = |w: f64, h: f64| w / 2.0 * d[0].abs() + h / 2.0 * d[1].abs();
    let r = reach(stage.get_width(label), stage.get_height(label))
        + FLOW_GAP
        + reach(stage.get_width(words), stage.get_height(words));
    stage.move_to(words, v(c[0] + d[0] * r, c[1] + d[1] * r), ORIGIN);
}

/// A tracker to `value` in `rt` seconds at `rate`: the kit's `tween` with
/// a rate (an animation's arguments are set once, so `tween` cannot take
/// one afterwards).
fn glide(tracker: Mob, value: f64, rt: f64, rate: fn(f64) -> f64) -> fmn::Result<AnimBuilder> {
    Ok(tracker
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            rate_func: Some(rate),
            ..AnimateArgs::default()
        })?
        .set_value(value)?)
}

/// The split crowd's live state: a progress tracker (the round, gliding
/// between whole rounds) and one tracker per corner (its players), moved
/// together, so everything that reads them moves in step.
struct Crowd {
    progress: Mob,
    count: [Mob; 3],
}

impl Crowd {
    /// Play round `k`: every tracker glides to the model's crowd after
    /// round `k` in the same `rt` at the same `rate` (so the total is the
    /// crowd's size in every frame), `along` drawn along their arrows.
    fn round(
        &self,
        stage: &mut Stage<'_>,
        rounds: &[[f64; 3]],
        k: usize,
        rt: f64,
        rate: fn(f64) -> f64,
        along: &[Mob],
    ) -> fmn::Result<()> {
        let mut anims: Vec<Box<dyn Animation>> = vec![stage.prepare(glide(self.progress, k as f64, rt, rate)?)?];
        for (i, &c) in self.count.iter().enumerate() {
            anims.push(stage.prepare(glide(c, rounds[k][i], rt, rate)?)?);
        }
        for &s in along {
            anims.push(stage.prepare(show_creation(s).rt(rt))?);
        }
        stage.play_prepared(anims)?;
        Ok(())
    }
}

/// The corner's name in the panel, "corner 0" in the bad coin's red.
/// Detached.
pub(super) fn row_name(kit: &Kit, stage: &mut Stage<'_>, i: usize) -> fmn::Result<Mob> {
    let color = if i == 0 { palette::bad() } else { Palette::ink() };
    let m = kit.text_left(stage, &format!("{CORNER_WORD} {i}"), 20.0, color, v(PANEL_X, ROW_YS[i]))?;
    keep_between(stage, m, PANEL_X - 0.01, BAR_X - 0.1, "a corner's name in the panel")?;
    Ok(m)
}

/// The players' total and the label saying what kind of numbers these are.
/// Detached.
pub(super) fn panel_head(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<[Mob; 2]> {
    let players = kit.text_left(stage, PLAYERS, 20.0, Palette::dim(), v(PANEL_X, PLAYERS_Y))?;
    keep_between(stage, players, PANEL_X - 0.01, PLAYERS_NUM_X - 0.1, "the players' word")?;
    let label = kit.text_left(stage, SPLIT_LABEL, 18.0, Palette::dim(), v(PANEL_X, LABEL_Y))?;
    keep_between(
        stage,
        label,
        PANEL_X - 0.01,
        B_AT[0] - 2.5 - CORNER_R - ROOM,
        "the expected-numbers label",
    )?;
    Ok([players, label])
}

/// The real crowd's corner `i`: its small triangle's, 0 at the top.
fn real_corner(i: usize) -> Vec3 {
    let a = FRAC_PI_2 - TAU / 3.0 * i as f64;
    v(REAL_AT[0] + REAL_R * a.cos(), REAL_AT[1] + REAL_R * a.sin())
}

/// The real crowd's triangle: faint edges and three discs, corner 0 tinted
/// red as on B's. Detached.
fn real_triangle(stage: &mut Stage<'_>) -> Vec<Mob> {
    let mut out = Vec::with_capacity(6);
    for e in 0..3 {
        let edge = polyline([real_corner(e), real_corner((e + 1) % 3)])
            .style(
                Style::default()
                    .fill(Palette::dim(), 0.0)
                    .stroke(Palette::dim(), 2.0, 0.3),
            )
            .build();
        out.push(stage.arena_mut().add(edge));
    }
    for i in 0..3 {
        let style = if i == 0 {
            flat(palette::bad(), 0.55)
        } else {
            flat(Palette::faint(), 1.0)
        };
        let disc = Circle::new()
            .radius(CORNER_R)
            .arc_center(real_corner(i))
            .style(style)
            .build();
        out.push(stage.arena_mut().add(disc));
    }
    out
}

/// Where the real crowd stands after round `k`.
fn real_places(players: &[Vec<usize>], k: usize) -> Vec<Vec3> {
    places(&players[k], real_corner)
}

impl SceneConstruct for OnePlayerToACrowd {
    fn name(&self) -> &str {
        "08_one_player_to_a_crowd"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let n = numbers().map_err(fail)?;
        let g = ROUND.game_b();
        let size = n.n;
        let crowd_n = usize::try_from(size).map_err(|e| fail(format!("the crowd's size: {e}")))?;

        // 08_1: 07's picture; is it really a third each? The same player
        // wins once more into corner 0 and becomes twenty-six copies
        let line = say(&kit, stage, "08_1")?;
        let a = triangle(&kit, stage, Which::A)?;
        let b = triangle(&kit, stage, Which::B)?;
        let a_labels = chance_labels(&kit, stage, Which::A, &n.a_round)?;
        let b_labels = chance_labels(&kit, stage, Which::B, &n.b_round)?;
        let hud = counter(&kit, stage, n.here)?;
        let rem = remainder_line(&kit, stage, n.here)?;
        let player = token(stage, corner(B_AT, corner_of(n.here)));
        let tag = round_badge(&kit, stage, &n.round_badge)?;
        let if_line = kit.text(stage, IF_LINE, 20.0, Palette::dim(), v(MID_X, IF_Y))?;
        let sum = kit.tex(stage, &n.sum_tex, 30.0, Palette::ink(), &[], v(MID_X, SUM_Y))?;
        let would = kit.text(stage, WOULD_WIN, 26.0, palette::good(), v(MID_X, WOULD_Y))?;
        for (m, what) in [
            (if_line, "the hypothetical's label"),
            (sum, "the hypothetical"),
            (would, "the verdict"),
        ] {
            keep_between(stage, m, MID_LEFT, MID_RIGHT, what)?;
        }
        let mut texts = vec![hud[0], hud[1], rem, tag[1], a.title, b.title, if_line, sum, would];
        texts.extend(a.names);
        texts.extend(b.names);
        texts.extend(a_labels);
        texts.extend(b_labels);
        keep_apart(stage, &texts, 0.08, "07's picture")?;
        let mut arriving = a.parts();
        arriving.extend(b.parts());
        arriving.extend(a_labels);
        arriving.extend(b_labels);
        arriving.extend([hud[0], hud[1], rem, player, if_line, sum, would]);
        arriving.extend(tag);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, AT_THIRD_EACH.1)?;
        let pointer = frame_around(stage, &[if_line, sum], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_LUCK.1)?;
        flash_all(stage, &[player], 0.5)?;
        line.wait_for(stage, AT_SEE_PAST.1)?;
        let mut leaving = a.parts();
        leaving.extend(a_labels);
        leaving.extend([if_line, sum, would]);
        fade_out_all(stage, &leaving, 0.6)?;
        // the same token, one more win along 2 → 0: 9 dollars, nothing over
        let nine = counter_number(&kit, stage, n.here + 1)?;
        let rem_nine = remainder_line(&kit, stage, n.here + 1)?;
        let zero = corner(B_AT, 0);
        let mut anims = vec![stage.prepare(fly(player, zero, 0.7)?)?];
        anims.extend(swap(stage, &[hud[0], rem], &[nine, rem_nine], 0.6)?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_COPIES.1)?;
        // it splits into the crowd, every copy on corner 0
        let dots: Vec<Mob> = (0..crowd_n).map(|_| dot(stage, zero)).collect();
        for &d in &dots {
            stage.add_to_scene(d)?;
        }
        let mut anims = Vec::with_capacity(crowd_n + 1);
        for (k, &d) in dots.iter().enumerate() {
            anims.push(stage.prepare(fly(d, slot(zero, k, crowd_n), 0.8)?)?);
        }
        let gone = fade_out(stage.arena_mut(), player, ORIGIN, 1.0)?;
        anims.push(stage.prepare(gone.rt(0.5))?);
        stage.play_prepared(anims)?;
        let size_label = kit.text_left(
            stage,
            &size.to_string(),
            24.0,
            palette::player(),
            v(zero[0] + CORNER_R + ROOM, zero[1]),
        )?;
        keep_between(
            stage,
            size_label,
            zero[0] + CORNER_R,
            FRAME_X_RADIUS - EDGE,
            "the crowd's size",
        )?;
        fade_in_all(stage, &[size_label], 0.4)?;
        // the crowd keeps a round and a total, not dollars
        let crowd = Crowd {
            progress: stage.add_value_tracker(0.0),
            count: n.rounds[0].map(|x| stage.add_value_tracker(x)),
        };
        let round_word = kit.text_left(stage, ROUND_WORD, 22.0, Palette::dim(), v(PANEL_X, ROUND_Y))?;
        keep_between(stage, round_word, PANEL_X - 0.01, ROUND_NUM_X - 0.1, "the round's word")?;
        let head = panel_head(&kit, stage)?;
        let anims = swap(stage, &[nine, hud[1], rem_nine], &[round_word, head[0]], 0.5)?;
        let progress = crowd.progress;
        let round_num = kit.live_value(
            stage,
            move |s| {
                // the round being played, or the last one played
                let t = s.tracker_value(progress).unwrap_or(0.0);
                if t <= 1e-9 { 0.0 } else { (t - 1e-9).ceil() }
            },
            0,
            false,
            22.0,
            Palette::ink(),
            v(ROUND_NUM_X, ROUND_Y),
            LEFT,
        )?;
        let cs = crowd.count;
        let total = kit.live_value(
            stage,
            move |s| cs.iter().map(|&c| s.tracker_value(c).unwrap_or(0.0)).sum(),
            0,
            false,
            22.0,
            Palette::ink(),
            v(PLAYERS_NUM_X, PLAYERS_Y),
            LEFT,
        )?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_ALL_ZERO.1)?;
        ring_corners(stage, &[b.discs[0]], &[], 0.5)?;
        line.wait_for(stage, AT_GAME_B.1)?;
        let pointer = frame_around(stage, &[b.title], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;

        // 08_2: split exactly by the chances: the dots merge into a puddle
        // on corner 0 and a bar in the panel, labelled expected numbers;
        // where is most of the crowd after one round? (a prediction)
        let line = say(&kit, stage, "08_2")?;
        line.wait_for(stage, AT_SPLIT.1)?;
        for (i, &c) in crowd.count.iter().enumerate() {
            let at = corner(B_AT, i);
            let whole = size as f64;
            let m = stage.always_redraw(move |s| {
                let share = (s.tracker_value(c).unwrap_or(0.0) / whole).clamp(0.0, 1.0);
                let r = (CORNER_R * share.sqrt()).max(0.004);
                s.add(
                    Circle::new()
                        .radius(r)
                        .arc_center(at)
                        .style(flat(palette::player(), 0.85))
                        .build(),
                )
            });
            stage.add_to_scene(m)?;
        }
        let mut leaving = dots.clone();
        leaving.push(size_label);
        fade_out_all(stage, &leaving, 0.6)?;
        let names: Vec<Mob> = (0..3).map(|i| row_name(&kit, stage, i)).collect::<fmn::Result<_>>()?;
        let note = kit.text_left(stage, &n.note, 16.0, Palette::dim(), v(PANEL_X, NOTE_Y))?;
        keep_between(stage, note, PANEL_X - 0.01, NUM_X + 0.6, "the precision note")?;
        let mut words = names.clone();
        words.extend([head[1], note]);
        fade_in_all(stage, &words, 0.5)?;
        let unit = BAR_FULL / size as f64;
        let mut row_nums = Vec::with_capacity(3);
        for (i, &c) in crowd.count.iter().enumerate() {
            live_bar(stage, c, v(BAR_X, ROW_YS[i]), unit, BAR_H, palette::player())?;
            row_nums.push(kit.live_value(
                stage,
                move |s| shown(s.tracker_value(c).unwrap_or(0.0), 1),
                1,
                false,
                22.0,
                Palette::ink(),
                v(NUM_X, ROW_YS[i]),
                LEFT,
            )?);
        }
        line.wait_for(stage, AT_AFTER_ONE.1)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 08_3: at corner 2: round 1, slow, a tenth forward and nine tenths
        // back, each labelled past its chance; what an expected number is
        let line = say(&kit, stage, "08_3")?;
        line.wait_for(stage, AT_CORNER_TWO.1)?;
        fade_out_all(stage, &[q], 0.2)?;
        let first = streams(stage, &n.rounds[0], &g, size);
        crowd.round(stage, &n.rounds, 1, R1_S, fmn::core::rate::smooth, &first)?;
        // corner 0's forward arrow is edge 0's; its back arrow edge 2's
        let (fw, bk) = (edge_of(0, 1), edge_of(0, 2));
        let forward_label = b_labels[fw.0];
        let back_label = b_labels[3 + bk.0];
        line.wait_for(stage, AT_TENTH.1)?;
        let ahead = kit.text(stage, &n.first_flows[0], 22.0, palette::good(), ORIGIN)?;
        past(stage, forward_label, fw.0, fw.1, ahead);
        fade_in_all(stage, &[ahead], 0.4)?;
        let pointer = frame_around(stage, &[forward_label], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.6)?;
        line.wait_for(stage, AT_NINE_TENTHS.1)?;
        let behind = kit.text(stage, &n.first_flows[1], 22.0, Palette::dim(), ORIGIN)?;
        past(stage, back_label, bk.0, bk.1, behind);
        let mut all_texts: Vec<Mob> = b_labels.to_vec();
        all_texts.extend(b.names);
        all_texts.extend([b.title, tag[1], ahead, behind]);
        keep_apart(stage, &all_texts, 0.08, "round 1's flows")?;
        fade_in_all(stage, &[behind], 0.4)?;
        let pointer = frame_around(stage, &[back_label], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.6)?;
        line.wait_for(stage, AT_NOT_PEOPLE.1)?;
        let k04 = precisely(&kit, stage, &n.k04)?;
        if stage.get_bounding_box_point(k04, UP)[1] > stage.get_bounding_box_point(b.title, DOWN)[1] - 0.1 {
            return Err(fail("K04 runs into game B's title".into()));
        }
        fade_in_all(stage, &[k04], 0.5)?;
        let card_up = now(stage);
        line.wait_for(stage, AT_EXPECTED.1)?;
        let pointer = frame_around(stage, &[head[1]], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        kit.hold(stage)?;

        // 08_4: round 2 carries three quarters of corner 2 around to 0; the
        // total never changes; rounds 3 to 6 slosh, a second each
        let line = say(&kit, stage, "08_4")?;
        line.wait_for(stage, AT_NEXT.1)?;
        let mut leaving = first.clone();
        leaving.extend([ahead, behind]);
        fade_out_all(stage, &leaving, 0.4)?;
        line.wait_for(stage, AT_QUARTERS.1)?;
        let along = streams(stage, &n.rounds[1], &g, size);
        crowd.round(stage, &n.rounds, 2, R2_S, fmn::core::rate::smooth, &along)?;
        fade_out_all(stage, &along, STREAM_FADE)?;
        line.wait_for(stage, AT_STAYS.1)?;
        let pointer = frame_around(stage, &[head[0], total], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_SLOSHES.1)?;
        for k in 3..=SLOSH_TO {
            let along = streams(stage, &n.rounds[k - 1], &g, size);
            crowd.round(stage, &n.rounds, k, ROUND_S, fmn::core::rate::smooth, &along)?;
            fade_out_all(stage, &along, STREAM_FADE)?;
        }
        kit.hold(stage)?;

        // 08_5: the share at corner 0 and its trace, swinging across a
        // dashed third; rounds 7 to 12 quicken
        let line = say(&kit, stage, "08_5")?;
        wait_until(stage, card_up + read_s(&n.k04))?;
        fade_out_all(stage, &[k04], 0.4)?;
        line.wait_for(stage, AT_WATCH.1)?;
        let trace = Frame2 {
            center: v((TRACE_LEFT + TRACE_RIGHT) / 2.0, (TRACE_BOTTOM + TRACE_TOP) / 2.0),
            width: TRACE_RIGHT - TRACE_LEFT,
            height: TRACE_TOP - TRACE_BOTTOM,
            x: (0.0, n.last as f64),
            y: (0.0, 100.0),
        };
        let share_words = kit.text_left(stage, SHARE_WORDS, 20.0, Palette::dim(), v(PANEL_X, SHARE_Y))?;
        keep_between(
            stage,
            share_words,
            PANEL_X - 0.01,
            SHARE_NUM_RIGHT - 0.7,
            "the share's words",
        )?;
        let percent = kit.text_left(stage, PERCENT, 22.0, Palette::ink(), v(SHARE_PCT_X, SHARE_Y))?;
        let base = polyline([trace.p(0.0, 0.0), trace.p(n.last as f64, 0.0)])
            .style(line_style(Palette::dim(), 2.0))
            .build();
        let side = polyline([trace.p(0.0, 0.0), trace.p(0.0, 100.0)])
            .style(line_style(Palette::dim(), 2.0))
            .build();
        let axes = [stage.arena_mut().add(base), stage.arena_mut().add(side)];
        let rounds_axis = kit.text(stage, ROUNDS_AXIS, 16.0, Palette::dim(), ORIGIN)?;
        stage.move_to(rounds_axis, v(TRACE_RIGHT, TRACE_BOTTOM - 0.2), RIGHT);
        let third_y = 100.0 / 3.0;
        let third_line = dashed(
            stage,
            trace.p(0.0, third_y),
            trace.p(n.last as f64, third_y),
            0.08,
            Palette::dim(),
            2.0,
        );
        let third_label = kit.text_left(
            stage,
            &n.third,
            18.0,
            Palette::dim(),
            v(TRACE_RIGHT + 0.1, trace.p(0.0, third_y)[1]),
        )?;
        keep_between(
            stage,
            third_label,
            TRACE_RIGHT,
            B_AT[0] - 2.5 - CORNER_R - ROOM,
            "the third's label",
        )?;
        let mut panel = vec![share_words, percent, rounds_axis, third_label];
        panel.extend(axes);
        panel.extend(&third_line);
        fade_in_all(stage, &panel, 0.5)?;
        let c0 = crowd.count[0];
        let whole = size as f64;
        let share = kit.live_value(
            stage,
            move |s| shown(100.0 * s.tracker_value(c0).unwrap_or(0.0) / whole, 1),
            1,
            false,
            22.0,
            Palette::ink(),
            v(SHARE_NUM_RIGHT, SHARE_Y),
            RIGHT,
        )?;
        // the trace: every whole round played so far, and the tip between
        // the last whole round and the next, read from the same trackers
        let points: Vec<Vec3> = n
            .rounds
            .iter()
            .enumerate()
            .map(|(k, r)| trace.p(k as f64, share_pct(r, size)))
            .collect();
        let trace_line = stage.arena_mut().add(
            polyline([points[0], points[0]])
                .style(line_style(palette::player(), 3.0))
                .build(),
        );
        let trace_anchor = stage.arena_mut().add(VMobject::new());
        let mut last_tip = None;
        stage.add_updater(
            trace_anchor,
            move |s, _| {
                let at = s
                    .tracker_value(progress)
                    .unwrap_or(0.0)
                    .clamp(0.0, (points.len() - 1) as f64);
                let pct = 100.0 * s.tracker_value(c0).unwrap_or(0.0) / whole;
                if last_tip == Some((at, pct)) {
                    return;
                }
                let whole_rounds = at.floor() as usize;
                let mut shown_points = points[..=whole_rounds].to_vec();
                if at > whole_rounds as f64 {
                    shown_points.push(trace.p(at, pct));
                }
                if shown_points.len() < 2 {
                    shown_points.push(shown_points[0]);
                }
                let _ = s.set_points(trace_line, polyline(shown_points).build().points());
                last_tip = Some((at, pct));
            },
            true,
        )?;
        stage.add_to_scene(trace_anchor)?;
        stage.add_to_scene(trace_line)?;
        line.wait_for(stage, AT_BAD_WAITS.1)?;
        ring_corners(stage, &[b.discs[0]], &[], 0.5)?;
        line.wait_for(stage, AT_SWINGS.1)?;
        let pointer = frame_around(stage, &[trace_line], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_STEADIES.1)?;
        for k in SLOSH_TO + 1..=QUICK_TO {
            crowd.round(stage, &n.rounds, k, QUICK_S, fmn::core::rate::smooth, &[])?;
        }
        kit.hold(stage)?;

        // 08_6: round after round to the settled round, a tenth of a second
        // each; 10, 4 and 12, the share 38.5 %, not a third each (held)
        let line = say(&kit, stage, "08_6")?;
        for k in QUICK_TO + 1..=n.last {
            crowd.round(stage, &n.rounds, k, FAST_S, fmn::core::rate::linear, &[])?;
        }
        let mut settled = Vec::with_capacity(3);
        for (i, at) in [AT_TEN, AT_FOUR, AT_TWELVE].into_iter().enumerate() {
            line.wait_for(stage, at.1)?;
            let m = kit.text_left(stage, &n.settled[i], 22.0, palette::player(), v(NUM_X, ROW_YS[i]))?;
            let anims = swap(stage, &[row_nums[i]], &[m], 0.4)?;
            stage.play_prepared(anims)?;
            let pointer = frame_around(stage, &[names[i], m], POINT_PAD, Palette::hero())?;
            point_at(stage, pointer, &[], 0.6)?;
            settled.push(m);
        }
        line.wait_for(stage, AT_NOT_THIRD.1)?;
        let share_fixed = kit.text(stage, &n.share_settled, 22.0, Palette::ink(), ORIGIN)?;
        stage.move_to(share_fixed, v(SHARE_NUM_RIGHT, SHARE_Y), RIGHT);
        let within = kit.text_left(stage, &n.within, 16.0, Palette::dim(), v(PANEL_X, TRACE_BOTTOM - 0.2))?;
        keep_between(stage, within, PANEL_X - 0.01, TRACE_RIGHT - 0.6, "the half-player note")?;
        let mut anims = swap(stage, &[share], &[share_fixed], 0.4)?;
        anims.extend(swap(stage, &[], &[within], 0.4)?);
        stage.play_prepared(anims)?;
        let pointer = frame_around(stage, &[share_words, share_fixed, percent], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        let pointer = frame_around(stage, &[third_label], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.6)?;
        kit.hold(stage)?;
        pause(stage, SETTLE_HOLD)?;

        // 08_7: real players: their own triangle and label, 26 dots from
        // 10, 4, 12 tossing in turn; eight rounds hop in silence
        let line = say(&kit, stage, "08_7")?;
        stage.clear_updaters(trace_anchor, true);
        let mut leaving = vec![
            share_words,
            share_fixed,
            percent,
            rounds_axis,
            third_label,
            within,
            note,
        ];
        leaving.extend(axes);
        leaving.extend(&third_line);
        leaving.extend([trace_line, trace_anchor, round_num, round_word]);
        fade_out_all(stage, &leaving, 0.5)?;
        line.wait_for(stage, AT_REAL.1)?;
        let pointer = frame_around(stage, &[head[1]], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_TOSS.1)?;
        let real = &n.real;
        let shape = real_triangle(stage);
        let label = kit.text_left(stage, REAL_LABEL, 18.0, palette::player(), v(PANEL_X, REAL_LABEL_Y))?;
        keep_between(stage, label, PANEL_X - 0.01, NUM_X + 0.6, "the real crowd's label")?;
        let real_word = kit.text_left(stage, ROUND_WORD, 20.0, Palette::dim(), v(PANEL_X, REAL_ROUND_Y))?;
        let start = real_places(&real.players, 0);
        let real_dots: Vec<Mob> = start.iter().map(|&p| dot(stage, p)).collect();
        let mut arriving = shape.clone();
        arriving.extend([label, real_word]);
        arriving.extend(&real_dots);
        fade_in_all(stage, &arriving, 0.6)?;
        let real_t = stage.add_value_tracker(0.0);
        let table = Rc::new(real.counts.clone());
        let mut real_counts = Vec::with_capacity(3);
        for i in 0..3 {
            let c = real_corner(i);
            let reach = CORNER_R + REAL_COUNT_GAP;
            let (at, edge) = if i == 2 {
                (v(c[0] - reach, c[1]), RIGHT)
            } else {
                (v(c[0] + reach, c[1]), LEFT)
            };
            let counts = Rc::clone(&table);
            real_counts.push(kit.live_value(
                stage,
                move |s| {
                    let k = s.tracker_value(real_t).unwrap_or(0.0).round().max(0.0) as usize;
                    counts[k.min(counts.len() - 1)][i] as f64
                },
                0,
                false,
                22.0,
                Palette::ink(),
                at,
                edge,
            )?);
        }
        let real_round = kit.live_count(stage, real_t, 22.0, Palette::ink(), v(REAL_ROUND_NUM_X, REAL_ROUND_Y))?;
        let mut real_texts = vec![label, real_word, real_round, head[1], head[0]];
        real_texts.extend(&names);
        real_texts.extend(&settled);
        real_texts.extend(&real_counts);
        keep_apart(stage, &real_texts, 0.08, "the real crowd's picture")?;
        kit.hold(stage)?;
        for k in 1..=REAL_SHOWN {
            let to = real_places(&real.players, k);
            let moves = real_dots
                .iter()
                .zip(&to)
                .map(|(&d, &p)| fly(d, p, HOP_S))
                .collect::<fmn::Result<Vec<_>>>()?;
            together(stage, moves)?;
            stage.set_tracker_value(real_t, k as f64)?;
            pause(stage, HOP_REST)?;
        }

        // 08_8: this run of a thousand rounds: a live tally of the rounds
        // that land exactly on 10, 4, 12, then this run's results (held)
        let line = say(&kit, stage, "08_8")?;
        let so_far = kit.text_left(stage, &real.so_far, 20.0, Palette::ink(), v(PANEL_X, RESULT_YS[0]))?;
        let tally_x = stage.get_bounding_box_point(so_far, RIGHT)[0] + 0.15;
        let tallies = Rc::new(real.tally.clone());
        let tally = kit.live_value(
            stage,
            move |s| {
                let k = s.tracker_value(real_t).unwrap_or(0.0).round().max(0.0) as usize;
                tallies[k.min(tallies.len() - 1)] as f64
            },
            0,
            false,
            22.0,
            palette::player(),
            v(tally_x, RESULT_YS[0]),
            LEFT,
        )?;
        fade_in_all(stage, &[so_far], 0.3)?;
        // the dots follow the rounds as the counter runs: each frame, where
        // the crowd stands after the round the counter shows
        let players = Rc::new(real.players.clone());
        let hopping = real_dots.clone();
        let snap = stage.arena_mut().add(VMobject::new());
        let mut snapped = None;
        stage.add_updater(
            snap,
            move |s, _| {
                let k = (s.tracker_value(real_t).unwrap_or(0.0).round().max(0.0) as usize).min(players.len() - 1);
                if snapped == Some(k) {
                    return;
                }
                for (&d, p) in hopping.iter().zip(real_places(&players, k)) {
                    s.move_to(d, p, ORIGIN);
                }
                snapped = Some(k);
            },
            true,
        )?;
        stage.add_to_scene(snap)?;
        line.wait_for(stage, AT_THIS_RUN.1)?;
        stage.play(glide(real_t, REAL_ROUNDS as f64, RUN_S, fmn::core::rate::linear)?)?;
        stage.clear_updaters(snap, true);
        line.wait_for(stage, AT_FORTY.1)?;
        let results = [
            kit.text_left(stage, &real.result[0], 20.0, Palette::ink(), v(PANEL_X, RESULT_YS[0]))?,
            kit.text_left(stage, &real.result[1], 20.0, Palette::ink(), v(PANEL_X, RESULT_YS[1]))?,
        ];
        for &m in &results {
            keep_between(
                stage,
                m,
                PANEL_X - 0.01,
                B_AT[0] - 2.5 - CORNER_R - ROOM,
                "a result line",
            )?;
        }
        let anims = swap(stage, &[so_far, tally], &results[..1], 0.5)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_AVERAGE.1)?;
        fade_in_all(stage, &results[1..], 0.5)?;
        kit.hold(stage)?;
        pause(stage, RESULT_HOLD)?;
        // the real crowd leaves; the split crowd stands at 10, 4, 12
        let mut leaving = shape;
        leaving.extend([label, real_word, real_round]);
        leaving.extend(&real_dots);
        leaving.extend(&real_counts);
        leaving.extend(results);
        fade_out_all(stage, &leaving, 0.6)?;
        let up = now(stage);
        pause(stage, BEAT)?;
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

    /// The chapter's words are true of its numbers: twenty-six copies from
    /// 8 dollars on corner 2; round 1 sends 2.6 forward and 23.4 back; the
    /// crowd settles at 10, 4 and 12 with 38.5 % at corner 0, within half a
    /// player from round 24; the real run lands on 10, 4, 12 in 40 of its
    /// 1,000 rounds and averages 10.0, 4.0, 12.0. The counts first read
    /// their settled one-decimal values for good at round 41, the share at
    /// round 60 (an independent float replay agrees), so the run is 60.
    #[test]
    fn one_player_to_a_crowd_numbers_are_the_words() {
        let n = numbers().unwrap();
        assert_eq!(n.here, 8);
        assert_eq!(n.n, 26);
        assert_eq!(n.first_flows, ["2.6", "23.4"]);
        assert_eq!(n.settled, ["10", "4", "12"]);
        assert_eq!(n.share_settled, "38.5");
        assert_eq!(n.third, "33.3 %");
        assert_eq!(n.within, "within half a player of 10, 4, 12 from round 24");
        assert_eq!(n.note, "shown to one decimal; the total is exactly 26");
        assert_eq!(
            n.round_badge,
            "round-number coins \u{b7} 50 \u{b7} 10 \u{b7} 75 % \u{b7} no handicap"
        );
        assert_eq!(
            n.sum_tex,
            r"\tfrac{1}{3} \cdot 10\% + \tfrac{2}{3} \cdot 75\% \approx 53.3\%"
        );
        assert_eq!(n.last, 60);
        assert_eq!(n.rounds.len(), n.last + 1);
        assert_eq!(n.real.so_far, "exactly 10, 4, 12 so far:");
        assert_eq!(
            n.real.result,
            [
                "this run: exactly 10, 4, 12 in 40 of 1,000 rounds",
                "average over the 1,000 rounds: 10.0, 4.0, 12.0"
            ]
        );
        assert!(
            n.k04.starts_with("expected numbers: 26 times one player's chance"),
            "{}",
            n.k04
        );
        // a fn, not a closure: a closure cannot return what it borrows
        // (root's native10 compile, Mail 6893)
        fn words(c: &[String; 3]) -> [&str; 3] {
            c.each_ref().map(String::as_str)
        }
        assert_eq!(words(&n.b_round.heads), ["10", "75", "75"]);
        assert_eq!(words(&n.b_round.tails), ["90", "25", "25"]);
    }

    /// A readout prints what the facts print. Negative control: the float
    /// nearest 5.85 lies below it, so formatting it prints 5.8 where the
    /// facts' half-away rounding prints 5.9; `shown` prints 5.9. Every
    /// printed round (1 to 6) reads exactly as the facts.
    #[test]
    fn the_readouts_round_as_the_facts_print() {
        assert_eq!(format!("{:.1}", 5.85_f64), "5.8");
        assert_eq!(Ratio::new(117, 20).fixed(1), "5.9");
        assert_eq!(one_decimal(5.85), "5.9");
        assert_eq!(one_decimal(2.6), "2.6");
        assert_eq!(one_decimal(0.0), "0.0");
        assert_eq!(one_decimal(5.849), "5.8");
        assert_eq!(format!("{:.0}", shown(25.999_999_999, 0)), "26");
        let n = numbers().unwrap();
        let f = facts::facts().unwrap();
        for (k, round) in n.rounds.iter().enumerate().take(CROWD_ROUNDS + 1).skip(1) {
            let printed = f.get(&format!("crowd.round.{k}")).unwrap();
            assert_eq!(csv(round.map(one_decimal)), printed, "round {k}");
        }
        assert_eq!(f.get("crowd.round.2"), Some("18.2,5.9,2.0"));
    }

    /// Every player is always somewhere: in every round to the settled one
    /// the streams out of the corners carry exactly the crowd that was
    /// there, and what arrives at each corner is the next round's crowd, so
    /// the total stays 26. Failed controls: a crowd forgetting its back
    /// flows keeps only 2.6 of its 26 after round 1; a crowd split a third
    /// each does not stay put; and the published coins' crowd does not
    /// settle at 10, 4, 12.
    #[test]
    fn the_split_crowd_keeps_every_player() {
        let n = numbers().unwrap();
        let g = ROUND.game_b();
        let p = g.chances_f64();
        assert_eq!(n.rounds.len(), n.last + 1);
        for (k, pair) in n.rounds.windows(2).enumerate() {
            let mut arrive = [0.0; 3];
            let mut leave = 0.0;
            for (i, (&here, &chance)) in pair[0].iter().zip(&p).enumerate() {
                let (fw, bk) = (here * chance, here * (1.0 - chance));
                arrive[(i + 1) % 3] += fw;
                arrive[(i + 2) % 3] += bk;
                leave += fw + bk;
            }
            assert!((leave - 26.0).abs() < 1e-9, "round {} carries {leave}", k + 1);
            for (i, (a, b)) in arrive.iter().zip(&pair[1]).enumerate() {
                assert!((a - b).abs() < 1e-9, "round {}, corner {i}", k + 1);
            }
        }
        // forgetting the back flows
        let forward_only: f64 = (0..3).map(|i| n.rounds[0][i] * p[i]).sum();
        assert!((forward_only - 2.6).abs() < 1e-9);
        // a third each is not where B's crowd stays
        let third = [Ratio::new(26, 3); 3];
        assert_ne!(model::evolve(&third, &g), third);
        // the settled counts do stay, exactly
        let counts = g.shares().map(|w| Ratio::int(26) * w);
        assert_eq!(model::evolve(&counts, &g), counts);
        // the published coins settle elsewhere
        let published = PUBLISHED.game_b().shares().map(|w| Ratio::int(26) * w);
        assert_ne!(published, counts);
    }

    /// A stream is drawn on its arrow: for all six arrows of B's triangle,
    /// the band's points are the arrow shaft's (both built from the arrow's
    /// ends and its bow's apex).
    #[test]
    fn a_stream_lies_on_its_arrow() {
        run(|stage| {
            for e in 0..3 {
                for forward in [true, false] {
                    let [shaft, _] = arrow_along(stage, Which::B, e, forward);
                    let band = stream(stage, e, forward, 13.0, 26);
                    let (a, b) = (stage.get_points(shaft).unwrap(), stage.get_points(band).unwrap());
                    assert_eq!(a.len(), b.len());
                    for (p, q) in a.iter().zip(&b) {
                        assert!(
                            (p[0] - q[0]).abs() < 1e-6 && (p[1] - q[1]).abs() < 1e-6,
                            "edge {e} forward {forward}: {p:?} against {q:?}"
                        );
                    }
                }
            }
            Ok(())
        });
    }

    /// The real crowd is the model's and one seed's illustration: its
    /// counts are `model::real_crowd`'s in every round, every player steps
    /// one corner every round, it leaves 10, 4, 12 in its first round while
    /// the split crowd never does, and another seed lands exactly on 10, 4,
    /// 12 a different number of times (51 for seed 2000), so 40 of 1,000 is
    /// this run's count, not a law.
    #[test]
    fn the_real_crowd_is_one_seeds_illustration() {
        let n = numbers().unwrap();
        let g = ROUND.game_b();
        let real = &n.real;
        let start = real.counts[0];
        assert_eq!(start, [10, 4, 12]);
        assert_eq!(real.counts[1..], model::real_crowd(&g, start, REAL_ROUNDS, SEED)[..]);
        assert_eq!(real.players.len(), REAL_ROUNDS + 1);
        for (k, pair) in real.players.windows(2).enumerate() {
            for (a, b) in pair[0].iter().zip(&pair[1]) {
                assert!(*b == (a + 1) % 3 || *b == (a + 2) % 3, "round {}", k + 1);
            }
        }
        assert_ne!(real.counts[1], start);
        assert_eq!(real.tally[REAL_SHOWN], 0);
        assert_eq!(real.tally[REAL_ROUNDS], 40);
        let other = model::real_crowd(&g, start, REAL_ROUNDS, 2000);
        assert_eq!(other.iter().filter(|&&c| c == start).count(), 51);
    }

    /// K04 fits two lines of the lower third (about 72 characters a line
    /// beside "precisely:" at 26 px), so its top stays below B's title.
    #[test]
    fn the_card_fits_two_lines() {
        let n = numbers().unwrap();
        let lines = explainer_kit::animatic::wrap(&n.k04, 72, usize::MAX);
        assert!(lines.len() <= 2, "{lines:?}");
        assert_eq!(lines.join(" "), n.k04);
    }

    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in CUES {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The layout keeps its rooms: the panel's rows stand apart and clear
    /// of B's corner 2 and its name; the note, the share row, the trace and
    /// the think ring stack without touching; the real crowd's triangle
    /// sits between its label and its result lines, inside the frame; a
    /// whole crowd's slots keep their dots apart, inside a corner's disc.
    #[test]
    fn the_layout_keeps_its_rooms() {
        const {
            assert!(PLAYERS_Y - LABEL_Y >= 0.35 && LABEL_Y - ROW_YS[0] >= 0.5);
            assert!(ROW_YS[0] - ROW_YS[1] > BAR_H + 0.2 && ROW_YS[1] - ROW_YS[2] > BAR_H + 0.2);
            assert!(ROW_YS[2] - BAR_H / 2.0 > NOTE_Y + 0.15 && NOTE_Y - 0.1 > SHARE_Y + 0.15);
            assert!(SHARE_Y - 0.15 > TRACE_TOP + 0.1);
            assert!(BAR_X + BAR_FULL < NUM_X - 0.1);
            assert!(THINK_AT[1] + 0.62 < ROW_YS[2] - BAR_H / 2.0 - 0.2);
            assert!(THINK_AT[1] - 0.62 - 0.3 - 0.1 > -FRAME_Y_RADIUS + 1.3);
            assert!(REAL_AT[1] + REAL_R + CORNER_R < REAL_LABEL_Y - 0.15);
            assert!(REAL_LABEL_Y + 0.1 < ROW_YS[2] - BAR_H / 2.0);
            assert!(REAL_AT[1] - REAL_R / 2.0 - CORNER_R > RESULT_YS[0] + 0.2);
            assert!(RESULT_YS[1] - 0.15 > -FRAME_Y_RADIUS + 0.25);
            assert!(CLUSTER_R + DOT_R <= CORNER_R);
        }
        // the panel's right edge (numbers and the third's label) against
        // B's corner 2 and its name
        let c2 = corner(B_AT, 2);
        let name = name_at(B_AT, 2, 0.15, 0.25);
        assert!(NUM_X + 0.6 < c2[0] - CORNER_R - ROOM);
        assert!(TRACE_RIGHT + 0.1 + 0.6 < name[0] - 0.075 - 0.1, "{name:?}");
        // the real triangle's right corner and count against B's corner 2
        let r1 = real_corner(1);
        assert!(r1[0] + CORNER_R + REAL_COUNT_GAP + 0.4 < c2[0] - CORNER_R - ROOM);
        // a whole crowd's slots: apart, and inside the cluster
        for k in 0..26 {
            let p = slot(ORIGIN, k, 26);
            assert!(p[0].hypot(p[1]) <= CLUSTER_R + 1e-12);
            for j in 0..k {
                let q = slot(ORIGIN, j, 26);
                assert!(
                    (p[0] - q[0]).hypot(p[1] - q[1]) >= 2.0 * DOT_R + 0.01,
                    "slots {j} and {k}"
                );
            }
        }
    }

    /// The chapter runs headless to its end with the scratch voice and real
    /// typesetting (so every `keep_between` and `keep_apart` guard meets the
    /// true text widths): every line of chapter 08 is said once, in order;
    /// the prediction (08_2) gets its silent think time; the settling is
    /// held after 08_6; the real crowd's eight rounds hop in silence
    /// between 08_7 and 08_8; the results hold at the end.
    #[test]
    fn the_chapter_runs_and_leaves_room_to_think() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = OnePlayerToACrowd { kit: kit.clone() };
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut NullSceneSink)
            .unwrap_or_else(|e| panic!("08_one_player_to_a_crowd: {e}"));
        let said = narrator.said();
        let ids: Vec<&str> = said.iter().map(|(id, _)| id.as_str()).collect();
        let expected: Vec<String> = (1..=8).map(|k| format!("08_{k}")).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        let at = |id: &str| said.iter().find(|(l, _)| l == id).unwrap().1;
        let end_of = |id: &str| at(id) + narrator.length(id).unwrap();
        assert!(
            at("08_3") - end_of("08_2") >= THINK_S,
            "{} s to think",
            at("08_3") - end_of("08_2")
        );
        assert!(at("08_7") - end_of("08_6") >= SETTLE_HOLD, "the settling is not held");
        let hops = REAL_SHOWN as f64 * (HOP_S + HOP_REST);
        assert!(
            at("08_8") - end_of("08_7") >= hops,
            "the real rounds do not hop in silence"
        );
        let end = kit.finished.borrow().as_ref().expect("a timeline").seconds();
        assert!(end - end_of("08_8") >= RESULT_HOLD, "08_8 has no hold: {end}");
    }
}
