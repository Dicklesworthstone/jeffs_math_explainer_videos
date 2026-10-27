//! Chapter 13, "Nothing unbiased does better": could a cleverer unbiased
//! recipe beat the gap recipe (13_1)? Unbiased must hold at every count the
//! company could have made, which "always 71" fails (13_2). For pairs, a
//! recipe of the larger number is forced count by count: the company of two
//! boxes fixes it at two (13_3), three boxes at three and a half (13_4);
//! the prediction: what must four boxes force (13_5)? Five, and five boxes
//! force six and a half (13_6). The step works at every count and handful
//! size, because the new handfuls all contain the new top box, a share k ÷ N
//! that is never zero (13_7), so only the gap recipe survives (13_8). Any
//! unbiased recipe, three times the smaller minus one (13_9), averaged in
//! the groups of the larger number, is free of the count and still
//! unbiased, so it is the staircase again (13_10); that is the whole
//! argument (13_11). Ties at the smallest company (13_12); a biased recipe,
//! seven sixths of the largest minus one, misses by less at 71 boxes
//! (13_13) but not at five, and its squared misses are bigger from five to
//! ten; card I says what is proved (13_14).
//!
//! The scene solves every company line and computes every rival estimate,
//! group average, miss, share and width from the facts and the model, and
//! refuses a picture they contradict. Highlights are halos and colour
//! changes are replacements: no typeset text is indicated, recoloured or
//! scaled in place.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, choose, fade_all, rect, v};
use fmn::prelude::*;

use super::averaging_the_wobble::{AVERAGED_IS, GAP_HEAD, averaged_is_colors, linear, mean, rule_text, shown};
use super::draw::{
    LABEL_SIZE, fade_in_all, fade_in_lagged, fade_out_all, fail, flat, line_style, pause, say, stroke, think,
    wait_until,
};
use super::friends_line::{
    ARROW_SPREAD, ARROW_W, BADGE_AT, PANEL_SIZE, PANEL_X, at, badge, handfuls, list, now, printed, read_s, whole,
};
use super::gaps_trade_places::glow_each;
use super::one_estimate::{BAR_O, approx, fraction, grow_bars};
use super::what_we_assume::{formula_colors, glow, place_row, replace, sum_texts, words_left};
use crate::facts::{self, TINY};
use crate::model::{self, Q};
use crate::palette;

pub struct NothingDoesBetter {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const ANY_UNBIASED: &str = "every unbiased recipe: ?";
const UNBIASED_IS: &str = "unbiased: right on average at every count";
const OF_LARGER: &str = "recipe of the larger number: ?";
/// The general step (13_7): what the new handfuls are, what their largest
/// number is, and their share.
const GENERAL: [&str; 3] = [
    "new handfuls: the ones that contain N",
    "their largest number: always N",
    "their share: k ÷ N, never 0",
];
/// The gap recipe for any handful size (13_8), chapter 04's formula with
/// the size left as k.
const GENERAL_FORMULA: &str = "estimate = largest + (largest − k) ÷ k";
const FLOW: [&str; 3] = [
    "any unbiased recipe",
    "the gap recipe",
    "misses never bigger, plain or squared",
];
const FLOW_STEP: &str = "average in the groups";
const TIE_AVERAGED: &str = "gap recipe, averaged: the gap recipe";
const FRIENDS_ROW: &str = "friends' five";
const MISS_ROW: &str = "average miss";
const SQUARED_ROW: &str = "average squared miss";
/// Card I (13_14): what is proved, the fairness first, after the staircase,
/// the rival and the biased recipe have shown it.
const CARD_I: &str = "fair handful: a recipe unbiased at every N ≥ k, averaged over each largest M, is the only \
                      unbiased recipe of M, M + (M − k) ÷ k. So none has a smaller average miss or squared miss \
                      at any N.";

/// The companies the staircase climbs (13_3 to 13_6), one a beat, and the
/// one the prediction asks about.
const STEPS: usize = 4;
const PREDICT_STEP: usize = 2;
/// The counts every claim about "always 71" and the biased recipe is
/// checked at, from the handful size up (the oracle goes on to 300).
const CHECK_TO: u64 = 100;

/// The triangle of pairs (as in chapter 11): column `m` for the larger
/// number from `GRID_X0` by `GRID_STEP`, row `s` for the smaller from
/// `GRID_Y0` up by `GRID_ROW`; the gold column numbers under the columns,
/// the smaller numbers left of the first; the rival's values right of the
/// dots.
const GRID_X0: f64 = -5.6;
const GRID_STEP: f64 = 1.1;
const GRID_Y0: f64 = 0.9;
const GRID_ROW: f64 = 0.42;
const PAIR_R: f64 = 0.09;
const NUMBER_Y: f64 = 0.5;
const ROW_LABEL_DX: f64 = 0.45;
const RIVAL_DX: f64 = 0.3;
/// The staircase's bars under the columns: their foot, height a unit, width,
/// the value over each; the caption under the foot.
const BAR_FOOT: f64 = -2.0;
const BAR_UNIT: f64 = 0.2;
const BAR_W: f64 = 0.5;
const VALUE_DY: f64 = 0.2;
const CAPTION_Y: f64 = -2.35;
/// The panel's lines from the top; the general lines (13_7, 13_8) under the
/// company lines; the flow (13_11) and its arrow; the countdown.
const LINES_Y: [f64; 7] = [3.5, 3.05, 2.6, 2.15, 1.7, 1.25, 0.8];
const GENERAL_Y: [f64; 4] = [1.1, 0.65, 0.2, -0.25];
const FLOW_Y: [f64; 3] = [1.25, 0.2, -0.25];
const FLOW_X: f64 = 0.6;
const FLOW_PAD: f64 = 0.21;
const FLOW_HEAD: f64 = 0.14;
const FLOW_TEXT_DX: f64 = 0.25;
const THINK_SPOT: [f64; 2] = [5.2, 0.2];
/// The space between the pieces of a small line, a space at the label size.
const SMALL_GAP: f64 = 0.11;
/// The table across the frame (13_13, 13_14): the labels' left edge, each
/// recipe's column centre, the heads, the rows and the line under them.
const TABLE_LEFT: f64 = -5.6;
const TABLE_X: [f64; 2] = [0.0, 3.0];
const TABLE_HEAD_Y: f64 = 2.6;
const TABLE_Y: [f64; 5] = [2.05, 1.5, 0.95, 0.4, -0.15];
const LOSES_Y: f64 = -0.8;
/// The inspection hold (13_14) and the beat after the last line.
const HOLD_S: f64 = 2.5;
const END_BEAT: f64 = 0.6;

/// What chapter 13 shows, from the facts: the "always" recipe; the pair
/// size, the staircase's companies, their pairs and new pairs, the forced
/// values and how the bars show them, the company of the prediction, the
/// general step proved, the friends' handful size, the count at least
/// sixty and the share of the new handfuls there; the tiny company's pairs,
/// the rival recipe, its estimates, their range, average and group
/// averages, its average miss and the gap recipe's as printed; the friends'
/// numbers and the gap recipe's estimate, the count of chapter 08, the gap
/// recipe's average there and its average miss about; the biased recipe and
/// its factor, its estimate for the friends, its average and bias at the
/// count, its average miss about, its average squared miss and about, the
/// gap recipe's about; the smallest count for the friends' handful, both
/// recipes there and the second about; the counts where its squared misses
/// are bigger.
#[derive(Clone, Debug)]
struct Picture {
    always: u64,
    seen: u64,
    companies: Vec<u64>,
    company_pairs: Vec<u64>,
    new_pairs: Vec<u64>,
    forced: Vec<Q>,
    forced_shown: Vec<String>,
    predict: u64,
    proved: bool,
    handful: u64,
    sixty: u64,
    share: Q,
    pairs: Vec<Vec<u64>>,
    rival: (Q, Q),
    rival_estimates: Vec<Q>,
    rival_range: Vec<u64>,
    rival_average: Q,
    rival_grouped: Vec<Q>,
    rival_miss: String,
    gap_miss: String,
    friends: Vec<u64>,
    gap_friends: u64,
    count: u64,
    gap_average: Q,
    gap_miss_count: String,
    shrunk: (Q, Q),
    factor: Q,
    shrunk_friends: Q,
    shrunk_average: Q,
    bias: Q,
    shrunk_miss: String,
    shrunk_squared: Q,
    shrunk_squared_shown: String,
    gap_squared_shown: String,
    edge: u64,
    gap_edge: Q,
    shrunk_edge: Q,
    shrunk_edge_shown: String,
    loses_squared: Vec<u64>,
}

fn picture() -> Result<Picture, String> {
    let f = facts::facts()?;
    let q = |key: &str| -> Result<Q, String> { fraction(&printed(&f, key)?) };
    let qs = |key: &str| -> Result<Vec<Q>, String> { printed(&f, key)?.split(',').map(fraction).collect() };
    let companies = list(&f, "G-F13.companies")?;
    let company_pairs = list(&f, "G-F13.company_pairs")?;
    // the smallest company's pairs are all new; each later company's new
    // pairs are the facts'
    let mut new_pairs = company_pairs.first().copied().into_iter().collect::<Vec<_>>();
    for c in companies.iter().skip(1) {
        new_pairs.push(whole(&f, &format!("G-F13.new_pairs.{c}"))?);
    }
    checked(Picture {
        always: whole(&f, "G-F13.always")?,
        seen: whole(&f, "G-F5.seen")?,
        companies,
        company_pairs,
        new_pairs,
        forced: qs("G-F13.forced")?,
        forced_shown: printed(&f, "G-F13.forced_shown")?
            .split(',')
            .map(str::to_owned)
            .collect(),
        predict: whole(&f, "G-F13.predict_company")?,
        proved: printed(&f, "G-F13.recurrence_proved")? == "true",
        handful: whole(&f, "G-F1.handful")?,
        sixty: whole(&f, "G-F1.at_least")?,
        share: q("G-F13.share_at_60")?,
        pairs: handfuls(&f, "G-F5.pair_list")?,
        rival: linear(&printed(&f, "G-F13.smaller_recipe")?, "smaller")?,
        rival_estimates: qs("G-F13.smaller_estimates")?,
        rival_range: list(&f, "G-F13.smaller_range")?,
        rival_average: q("G-F13.smaller_average")?,
        rival_grouped: qs("G-F13.smaller_grouped")?,
        rival_miss: printed(&f, "G-F13.smaller_miss_decimal")?,
        gap_miss: printed(&f, "G-F12.grouped_miss")?,
        friends: list(&f, "G-F1.numbers")?,
        gap_friends: whole(&f, "G-F3.estimate")?,
        count: whole(&f, "G-F8.count")?,
        gap_average: q("G-F8.average")?,
        gap_miss_count: printed(&f, "G-F8.average_miss_decimal")?,
        shrunk: linear(&printed(&f, "G-F13.shrunk_recipe")?, "largest")?,
        factor: q("G-F13.shrunk_factor")?,
        shrunk_friends: q("G-F13.shrunk_friends")?,
        shrunk_average: q("G-F13.shrunk_average_71")?,
        bias: q("G-F13.shrunk_bias_71")?,
        shrunk_miss: printed(&f, "G-F13.shrunk_miss_71_shown")?,
        shrunk_squared: q("G-F13.shrunk_squared_71")?,
        shrunk_squared_shown: printed(&f, "G-F13.shrunk_squared_71_shown")?,
        gap_squared_shown: printed(&f, "G-F13.gap_squared_71_shown")?,
        edge: whole(&f, "G-F13.edge_count")?,
        gap_edge: q("G-F13.gap_at_edge")?,
        shrunk_edge: q("G-F13.shrunk_at_edge")?,
        shrunk_edge_shown: printed(&f, "G-F13.shrunk_at_edge_shown")?,
        loses_squared: list(&f, "G-F13.shrunk_loses_squared_at")?,
    })
}

/// The company lines' left side up to company `i`: what the earlier
/// companies' new pairs say plus the new pairs times their unknown value.
fn company_total(pic: &Picture, i: usize) -> Q {
    pic.new_pairs
        .iter()
        .zip(&pic.forced)
        .take(i + 1)
        .fold(Q::ZERO, |s, (&p, &g)| s + Q::int(p as i128) * g)
}

/// A recipe's average squared miss over the fair handfuls of `h` from
/// `1..=n`, a recipe of the largest number.
fn squared_at(n: u64, h: u64, f: &dyn Fn(u64) -> Q) -> Q {
    let count = Q::int(n as i128);
    model::mean_by_largest(n, h, |m| {
        let d = f(m) - count;
        d * d
    })
}

/// The same for the plain miss.
fn plain_at(n: u64, h: u64, f: &dyn Fn(u64) -> Q) -> Q {
    let count = Q::int(n as i128);
    model::mean_by_largest(n, h, |m| (f(m) - count).abs())
}

/// The picture only if the scene can draw it truthfully: "always" is wrong
/// on average at every count but its own, which is bigger than the tiny
/// company; the companies are every count from the pair size to the tiny
/// company, one a beat, the prediction's the third, each holding its pairs,
/// its new pairs the ones its top box tops, never none; the forced values
/// are the model's, solve every company line exactly, start at the pair
/// size, are the gap recipe's and are shown exactly; the general step is
/// proved and the new handfuls' share at sixty boxes is the handful size
/// over the count, one in a whole number; the tiny company's pairs are
/// every pair, the rival's estimates its rule's, ranging and averaging as
/// printed, unbiased there, its group averages the staircase, its average
/// miss bigger than the gap recipe's; the biased recipe has the factor
/// printed, gives the friends' estimate printed while the gap recipe gives
/// its own, averages short of the count by the bias, misses by less there,
/// plain and squared, as printed, and at the friends' handful size as count
/// the gap recipe is exact while it is not, its squared misses bigger
/// exactly at a run of counts from there.
fn checked(pic: Picture) -> Result<Picture, String> {
    let k = pic.seen;
    let always = Q::int(pic.always as i128);
    if pic.always <= TINY
        || pic.always > CHECK_TO
        || (k..=CHECK_TO).any(|n| (model::mean_by_largest(n, k, |_| always) == Q::int(n as i128)) != (n == pic.always))
    {
        return Err(format!(
            "a recipe that always says {} must be right on average only at {}",
            pic.always, pic.always
        ));
    }
    let n = pic.companies.len();
    if pic.companies != (k..=TINY).collect::<Vec<_>>()
        || n != STEPS
        || [
            pic.company_pairs.len(),
            pic.new_pairs.len(),
            pic.forced.len(),
            pic.forced_shown.len(),
        ] != [n; 4]
        || pic.companies.get(PREDICT_STEP) != Some(&pic.predict)
        || pic
            .companies
            .iter()
            .zip(&pic.company_pairs)
            .any(|(&c, &p)| choose(c, k) != u128::from(p))
        || pic
            .companies
            .iter()
            .zip(&pic.new_pairs)
            .any(|(&c, &p)| p == 0 || model::with_largest(c, k) != Q::int(p as i128))
        || (1..n).any(|i| pic.new_pairs[i] + pic.company_pairs[i - 1] != pic.company_pairs[i])
    {
        return Err(format!(
            "the companies {:?} must hold {:?} pairs, {:?} of them new",
            pic.companies, pic.company_pairs, pic.new_pairs
        ));
    }
    if pic.forced != model::forced_values(k, TINY)
        || pic.forced.first() != Some(&Q::int(k as i128))
        || pic
            .companies
            .iter()
            .enumerate()
            .any(|(i, &c)| company_total(&pic, i) != Q::int((pic.company_pairs[i] * c) as i128))
        || pic
            .companies
            .iter()
            .zip(&pic.forced)
            .any(|(&c, &g)| model::gap_of(c, k) != g)
        || pic
            .forced
            .iter()
            .zip(&pic.forced_shown)
            .any(|(&g, s)| shown(g).as_deref() != Ok(s.as_str()))
    {
        return Err(format!(
            "the staircase {:?} must solve every company line and be the gap recipe's",
            pic.forced
        ));
    }
    // the new handfuls at sixty boxes, those that contain box sixty, as a
    // share of all of them
    let new_share = (pic.handful >= 2 && pic.sixty > pic.handful)
        .then(|| Q::count(choose(pic.sixty - 1, pic.handful - 1)) / Q::count(choose(pic.sixty, pic.handful)));
    if !pic.proved
        || Q::new(pic.handful as i128, pic.sixty as i128) != pic.share
        || new_share != Some(pic.share)
        || (Q::ONE / pic.share).whole().is_none()
    {
        return Err(format!(
            "the share of new handfuls at {} boxes must be {} ÷ {}, one in a whole number, and the step proved",
            pic.sixty, pic.handful, pic.sixty
        ));
    }
    let (a, b) = pic.rival;
    let tiny = model::all_handfuls(TINY, k as usize);
    let count = Q::int(TINY as i128);
    let rival: Vec<Q> = tiny.iter().map(|p| a * Q::int(p[0] as i128) + b).collect();
    let gap: Vec<Q> = tiny.iter().map(|p| model::gap_recipe(p)).collect();
    let grouped: Vec<Q> = pic
        .companies
        .iter()
        .map(|&m| {
            let members: Vec<Q> = tiny
                .iter()
                .zip(&rival)
                .filter(|(p, _)| p[p.len() - 1] == m)
                .map(|(_, &e)| e)
                .collect();
            mean(&members)
        })
        .collect();
    let miss = |xs: &[Q]| mean(&xs.iter().map(|&e| (e - count).abs()).collect::<Vec<_>>());
    let range: Vec<u64> = [rival.iter().min(), rival.iter().max()]
        .iter()
        .filter_map(|e| e.and_then(|x| x.whole()).and_then(|w| u64::try_from(w).ok()))
        .collect();
    if pic.pairs != tiny
        || pic.rival_estimates != rival
        || range != pic.rival_range
        || mean(&rival) != pic.rival_average
        || pic.rival_average != count
        || grouped != pic.rival_grouped
        || grouped != pic.forced
        || miss(&rival).decimal1().as_deref() != Ok(pic.rival_miss.as_str())
        || miss(&gap).decimal1().as_deref() != Ok(pic.gap_miss.as_str())
        || miss(&rival) <= miss(&gap)
    {
        return Err(format!(
            "the rival {:?} must give {:?}, averaging {} and the staircase in the groups, missing by more",
            pic.rival, rival, pic.rival_average
        ));
    }
    let (a, b) = pic.shrunk;
    let h = pic.handful;
    let at_count = Q::int(pic.count as i128);
    let shrunk = |m: u64| a * Q::int(m as i128) + b;
    let gap_of = |m: u64| model::gap_of(m, h);
    let largest = pic.friends.last().copied().unwrap_or(0);
    if a != pic.factor
        || pic.friends.len() as u64 != h
        || shrunk(largest) != pic.shrunk_friends
        || model::gap_recipe(&pic.friends) != Q::int(pic.gap_friends as i128)
        || model::mean_by_largest(pic.count, h, gap_of) != pic.gap_average
        || pic.gap_average != at_count
        || model::mean_by_largest(pic.count, h, shrunk) != pic.shrunk_average
        || pic.shrunk_average - at_count != pic.bias
        || pic.bias >= Q::ZERO
        || plain_at(pic.count, h, &shrunk).about1().as_deref() != Ok(pic.shrunk_miss.as_str())
        || plain_at(pic.count, h, &gap_of).about1().as_deref() != Ok(pic.gap_miss_count.as_str())
        || plain_at(pic.count, h, &shrunk) >= plain_at(pic.count, h, &gap_of)
        || squared_at(pic.count, h, &shrunk) != pic.shrunk_squared
        || pic.shrunk_squared.about1().as_deref() != Ok(pic.shrunk_squared_shown.as_str())
        || squared_at(pic.count, h, &gap_of).about1().as_deref() != Ok(pic.gap_squared_shown.as_str())
        || pic.shrunk_squared >= squared_at(pic.count, h, &gap_of)
    {
        return Err(format!(
            "the biased recipe {:?} at {} boxes must average {} and miss by less, plain and squared",
            pic.shrunk, pic.count, pic.shrunk_average
        ));
    }
    let loses: Vec<u64> = (pic.edge..=CHECK_TO)
        .filter(|&c| squared_at(c, h, &shrunk) > squared_at(c, h, &gap_of))
        .collect();
    if pic.edge != h
        || gap_of(pic.edge) != pic.gap_edge
        || pic.gap_edge != Q::int(pic.edge as i128)
        || shrunk(pic.edge) != pic.shrunk_edge
        || pic.shrunk_edge == pic.gap_edge
        || pic.shrunk_edge.about1().as_deref() != Ok(pic.shrunk_edge_shown.as_str())
        || loses != pic.loses_squared
        || pic.loses_squared.first() != Some(&pic.edge)
        || pic.loses_squared.windows(2).any(|w| w[1] != w[0] + 1)
    {
        return Err(format!(
            "the biased recipe must be beaten at {} boxes and lose squared exactly at {:?}, not {:?}",
            pic.edge, loses, pic.loses_squared
        ));
    }
    Ok(pic)
}

// ------------------------------------------------------------------ words

fn always_text(pic: &Picture) -> String {
    format!("always {}: right on average only at {}", pic.always, pic.always)
}

/// A company's caption under the bars (13_3 to 13_6).
fn caption_text(pic: &Picture, i: usize) -> String {
    let p = pic.company_pairs[i];
    format!(
        "{} boxes: {p} {}",
        pic.companies[i],
        if p == 1 { "pair" } else { "pairs" }
    )
}

/// Company `i`'s line in three pieces: what the earlier values and the new
/// pairs times their value add to, that value ("?" until it is solved) and
/// what they must make, "2 boxes: 1 ×", "?", "= 1 × 2".
fn company_pieces(pic: &Picture, i: usize, solved: bool) -> [String; 3] {
    let mut terms: Vec<String> = (0..i)
        .map(|j| format!("{} × {}", pic.new_pairs[j], pic.forced_shown[j]))
        .collect();
    terms.push(format!("{} ×", pic.new_pairs[i]));
    let c = pic.companies[i];
    [
        format!("{c} boxes: {}", terms.join(" + ")),
        if solved {
            pic.forced_shown[i].clone()
        } else {
            "?".to_owned()
        },
        format!("= {} × {c}", pic.company_pairs[i]),
    ]
}

/// The share at sixty boxes (13_7).
fn share_text(pic: &Picture) -> Result<String, String> {
    let one_in = (Q::ONE / pic.share)
        .whole()
        .ok_or_else(|| format!("{} is not one in a whole number", pic.share))?;
    Ok(format!(
        "{h} seen, {n} boxes: {h} ÷ {n} = 1 in {one_in}",
        h = pic.handful,
        n = pic.sixty
    ))
}

fn rival_text(pic: &Picture) -> String {
    rule_text("smaller", pic.rival)
}

/// The rival's estimates averaged (13_9).
fn rival_sum(pic: &Picture) -> String {
    let values: Vec<String> = pic.rival_estimates.iter().map(Q::to_string).collect();
    format!("({}) ÷ {} = {}", values.join(" + "), values.len(), pic.rival_average)
}

/// Group `g`'s average (13_10) in two pieces, its average green: "topped by
/// 3: (2 + 5) ÷ 2 =", "3.5"; a lone value without brackets.
fn group_pieces(pic: &Picture, g: usize) -> Result<[String; 2], String> {
    let m = pic.companies[g];
    let k = pic.seen as usize;
    let values: Vec<String> = pic
        .pairs
        .iter()
        .zip(&pic.rival_estimates)
        .filter(|(p, _)| p[k - 1] == m)
        .map(|(_, e)| e.to_string())
        .collect();
    let sum = if values.len() == 1 {
        values.join("")
    } else {
        format!("({})", values.join(" + "))
    };
    Ok([
        format!("topped by {m}: {sum} ÷ {} =", values.len()),
        shown(pic.rival_grouped[g])?,
    ])
}

/// The two average misses (13_11), the rival's then the gap recipe's.
fn miss_texts(pic: &Picture) -> [String; 2] {
    [
        format!("{MISS_ROW}, {}: {}", rival_text(pic), pic.rival_miss),
        format!("{MISS_ROW}, {GAP_HEAD}: {}", pic.gap_miss),
    ]
}

/// The tie at the smallest company (13_12).
fn tie_text(pic: &Picture) -> String {
    format!(
        "{} boxes: every unbiased recipe says {}",
        pic.companies[0], pic.forced_shown[0]
    )
}

fn shrunk_text(pic: &Picture) -> String {
    rule_text("largest", pic.shrunk)
}

/// The table's row labels (13_13, 13_14).
fn row_labels(pic: &Picture) -> [String; 5] {
    [
        FRIENDS_ROW.to_owned(),
        format!("average, {} boxes", pic.count),
        MISS_ROW.to_owned(),
        SQUARED_ROW.to_owned(),
        format!("every handful, {} boxes", pic.edge),
    ]
}

/// The table's cells, the gap recipe's then the biased recipe's.
fn cells(pic: &Picture) -> [[String; 2]; 5] {
    [
        [pic.gap_friends.to_string(), pic.shrunk_friends.to_string()],
        [pic.gap_average.to_string(), pic.shrunk_average.to_string()],
        [format!("≈ {}", pic.gap_miss_count), format!("≈ {}", pic.shrunk_miss)],
        [
            format!("≈ {}", pic.gap_squared_shown),
            format!("≈ {}", pic.shrunk_squared_shown),
        ],
        [pic.gap_edge.to_string(), format!("≈ {}", pic.shrunk_edge_shown)],
    ]
}

/// Where the biased recipe's squared misses are bigger (13_14).
fn loses_text(pic: &Picture) -> String {
    let first = pic.loses_squared.first().copied().unwrap_or(pic.edge);
    let last = pic.loses_squared.last().copied().unwrap_or(pic.edge);
    format!("{}: squared misses bigger at counts {first} to {last}", pic.factor)
}

// ------------------------------------------------------------ positions

/// Column `m`'s centre (the pairs topped by `m`).
fn col_x(m: u64, k: u64) -> f64 {
    GRID_X0 + GRID_STEP * (m as f64 - k as f64)
}

/// Row `s`'s centre (the pairs whose smaller number is `s`).
fn row_y(s: u64) -> f64 {
    GRID_Y0 + GRID_ROW * (s as f64 - 1.0)
}

/// The caption's centre, under the middle of the bars.
fn caption_x(pic: &Picture) -> f64 {
    (col_x(pic.companies[0], pic.seen) + col_x(TINY, pic.seen)) / 2.0
}

fn line_at(j: usize) -> Vec3 {
    v(PANEL_X, LINES_Y[j])
}

/// A bar's height for value `g`.
fn bar_h(g: Q) -> Result<f64, String> {
    Ok(BAR_UNIT * approx(g)?)
}

/// The flow's down arrow (13_11), from under its first line to over its
/// second, its head at the bottom.
fn flow_points() -> Vec<Vec3> {
    let (top, end) = (FLOW_Y[0] - FLOW_PAD, FLOW_Y[1] + FLOW_PAD);
    let s = ARROW_SPREAD * FLOW_HEAD;
    vec![
        v(FLOW_X, top),
        v(FLOW_X, end),
        v(FLOW_X - s, end + FLOW_HEAD),
        v(FLOW_X, end),
        v(FLOW_X + s, end + FLOW_HEAD),
    ]
}

// ------------------------------------------------------------- builders

/// A small line in pieces, each its own text in its colour, laid out from
/// `x` on height `y`. Detached.
fn small_pieces(kit: &Kit, stage: &mut Stage<'_>, pieces: &[(String, Srgb)], x: f64, y: f64) -> fmn::Result<Vec<Mob>> {
    let mut mobs = Vec::new();
    for (s, c) in pieces {
        mobs.push(kit.text(stage, s, LABEL_SIZE, *c, v(x, y))?);
    }
    place_row(stage, &mobs, x, y, SMALL_GAP);
    Ok(mobs)
}

/// Company `i`'s line on panel line `i + 1`, its value green once solved.
/// Detached.
fn company_line(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture, i: usize, solved: bool) -> fmn::Result<Vec<Mob>> {
    let [head, value, tail] = company_pieces(pic, i, solved);
    let value_color = if solved { palette::estimate() } else { Palette::ink() };
    small_pieces(
        kit,
        stage,
        &[(head, Palette::ink()), (value, value_color), (tail, Palette::ink())],
        PANEL_X,
        LINES_Y[i + 1],
    )
}

/// One company's column (13_3 to 13_6): the dots of the pairs it tops, its
/// gold number under them, the smaller number its top pair brings in, and
/// its bar with its value. Detached.
struct Column {
    dots: Vec<Mob>,
    number: Mob,
    row_label: Mob,
    bar: Mob,
    value: Mob,
}

impl Column {
    fn build(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture, i: usize) -> fmn::Result<Column> {
        let k = pic.seen;
        let m = pic.companies[i];
        let x = col_x(m, k);
        let mut dots = Vec::new();
        for p in pic.pairs.iter().filter(|p| p[p.len() - 1] == m) {
            let d = Dot::new()
                .point(v(x, row_y(p[0])))
                .radius(PAIR_R)
                .style(flat(palette::seen(), 1.0))
                .build();
            dots.push(stage.arena_mut().add(d));
        }
        let number = kit.text(stage, &m.to_string(), LABEL_SIZE, palette::seen(), v(x, NUMBER_Y))?;
        // the column topped by m brings in the smaller number m − 1
        let row_label = kit.text(
            stage,
            &(m - 1).to_string(),
            LABEL_SIZE,
            Palette::dim(),
            v(col_x(k, k) - ROW_LABEL_DX, row_y(m - 1)),
        )?;
        let h = bar_h(pic.forced[i]).map_err(fail)?;
        let r = rect(v(x - BAR_W / 2.0, BAR_FOOT), BAR_W, h, flat(palette::estimate(), BAR_O))?;
        let bar = stage.arena_mut().add(r);
        let value = kit.text(
            stage,
            &pic.forced_shown[i],
            LABEL_SIZE,
            palette::estimate(),
            v(x, BAR_FOOT + h + VALUE_DY),
        )?;
        Ok(Column {
            dots,
            number,
            row_label,
            bar,
            value,
        })
    }

    /// The dots, the number and the row's label: what lands with the
    /// company.
    fn landing(&self) -> Vec<Mob> {
        let mut m = self.dots.clone();
        m.extend([self.number, self.row_label]);
        m
    }

    fn step(&self) -> [Mob; 2] {
        [self.bar, self.value]
    }

    fn all(&self) -> Vec<Mob> {
        let mut m = self.landing();
        m.extend(self.step());
        m
    }
}

/// Company `i` lands: its column's dots, number and smaller number, and its
/// caption in place of `caption`, which it returns.
fn land(
    kit: &Kit,
    stage: &mut Stage<'_>,
    pic: &Picture,
    columns: &[Column],
    i: usize,
    caption: Mob,
) -> fmn::Result<Mob> {
    let next = kit.text(
        stage,
        &caption_text(pic, i),
        LABEL_SIZE,
        Palette::ink(),
        v(caption_x(pic), CAPTION_Y),
    )?;
    fade_in_all(stage, &columns[i].landing(), 0.5)?;
    replace(stage, &[caption], &[next], 0.4)?;
    Ok(next)
}

/// Company `i`'s line `open` gives way to the solved line, which it
/// returns, and the column's bar rises with its value.
fn solve(
    kit: &Kit,
    stage: &mut Stage<'_>,
    pic: &Picture,
    columns: &[Column],
    i: usize,
    open: &[Mob],
) -> fmn::Result<Vec<Mob>> {
    let done = company_line(kit, stage, pic, i, true)?;
    replace(stage, open, &done, 0.5)?;
    grow_bars(stage, &[columns[i].bar], 0.8, 0.0)?;
    fade_in_all(stage, &[columns[i].value], 0.3)?;
    Ok(done)
}

/// The rival's value beside each pair's dot (13_9), in table order.
/// Detached.
fn rival_values(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture) -> fmn::Result<Vec<Mob>> {
    let k = pic.seen;
    let mut mobs = Vec::new();
    for (p, e) in pic.pairs.iter().zip(&pic.rival_estimates) {
        let spot = v(col_x(p[p.len() - 1], k) + RIVAL_DX, row_y(p[0]));
        mobs.push(kit.text(stage, &e.to_string(), LABEL_SIZE, Palette::ink(), spot)?);
    }
    Ok(mobs)
}

/// The table across the frame (13_13, 13_14): the heads, then a row a
/// label and the two recipes' values. Detached.
fn table(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture) -> fmn::Result<(Vec<Mob>, Vec<Vec<Mob>>)> {
    let colors = [palette::estimate(), Palette::ink()];
    let heads = vec![
        kit.text(stage, GAP_HEAD, PANEL_SIZE, colors[0], v(TABLE_X[0], TABLE_HEAD_Y))?,
        kit.text(
            stage,
            &shrunk_text(pic),
            PANEL_SIZE,
            colors[1],
            v(TABLE_X[1], TABLE_HEAD_Y),
        )?,
    ];
    let mut rows = Vec::new();
    for ((label, row), &y) in row_labels(pic).iter().zip(cells(pic)).zip(&TABLE_Y) {
        let mut mobs = vec![words_left(kit, stage, label, &[], v(TABLE_LEFT, y))?];
        for (c, s) in row.iter().enumerate() {
            mobs.push(kit.text(stage, s, PANEL_SIZE, colors[c], v(TABLE_X[c], y))?);
        }
        rows.push(mobs);
    }
    Ok((heads, rows))
}

/// Every word of the table and the line under it, for its reading time.
fn table_text(pic: &Picture) -> String {
    let mut words = vec![GAP_HEAD.to_owned(), shrunk_text(pic)];
    for (label, row) in row_labels(pic).into_iter().zip(cells(pic)) {
        words.push(label);
        words.extend(row);
    }
    words.push(loses_text(pic));
    words.join(" ")
}

impl SceneConstruct for NothingDoesBetter {
    fn name(&self) -> &str {
        "13_nothing_does_better"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let pic = picture().map_err(fail)?;
        let columns = (0..pic.companies.len())
            .map(|i| Column::build(&kit, stage, &pic, i))
            .collect::<fmn::Result<Vec<_>>>()?;

        // 13_1: the badge; what averaging did, and the question
        let line = say(&kit, stage, "13_1")?;
        let corner = badge(&kit, stage, at(BADGE_AT))?;
        let did = words_left(&kit, stage, AVERAGED_IS, &averaged_is_colors(), line_at(0))?;
        fade_in_all(stage, &[corner, did], 0.6)?;
        line.wait_for(stage, "every recipe that's unbiased")?;
        let asked = words_left(&kit, stage, ANY_UNBIASED, &[], line_at(1))?;
        fade_in_all(stage, &[asked], 0.5)?;

        // 13_2: what unbiased means; "always 71" and a halo round it
        let line = say(&kit, stage, "13_2")?;
        line.wait_for(stage, "Unbiased has to hold at every count")?;
        let meaning = words_left(&kit, stage, UNBIASED_IS, &[], line_at(2))?;
        fade_in_all(stage, &[meaning], 0.5)?;
        line.wait_for(stage, "A recipe that always says seventy-one")?;
        let always = words_left(&kit, stage, &always_text(&pic), &[], line_at(3))?;
        fade_in_all(stage, &[always], 0.5)?;
        line.wait_for(stage, "So it isn't unbiased")?;
        glow(stage, &[always], Palette::ink())?;

        // 13_3: the smallest company's pair, its line solved, its bar
        let line = say(&kit, stage, "13_3")?;
        line.wait_for(stage, "Start with pairs")?;
        fade_out_all(stage, &[did, asked, meaning, always], 0.4)?;
        let of_larger = words_left(&kit, stage, OF_LARGER, &[], line_at(0))?;
        fade_in_all(stage, &[of_larger], 0.5)?;
        line.wait_for(stage, "The smallest company with a pair")?;
        let mut caption = kit.text(
            stage,
            &caption_text(&pic, 0),
            LABEL_SIZE,
            Palette::ink(),
            v(caption_x(&pic), CAPTION_Y),
        )?;
        let mut landed = columns[0].landing();
        landed.push(caption);
        fade_in_all(stage, &landed, 0.5)?;
        line.wait_for(stage, "To average two")?;
        let mut open = company_line(&kit, stage, &pic, 0, false)?;
        fade_in_all(stage, &open, 0.4)?;
        line.wait_for(stage, "must say two when the larger number is two")?;
        let mut solved = vec![solve(&kit, stage, &pic, &columns, 0, &open)?];

        // 13_4: the second company; a halo round the bar it keeps; its line
        // solved and its bar
        let line = say(&kit, stage, "13_4")?;
        line.wait_for(stage, "Three boxes make three pairs")?;
        caption = land(&kit, stage, &pic, &columns, 1, caption)?;
        line.wait_for(stage, "The pair topped by two must still say two")?;
        glow(stage, &columns[0].step(), palette::estimate())?;
        line.wait_for(stage, "all three estimates must average three")?;
        open = company_line(&kit, stage, &pic, 1, false)?;
        fade_in_all(stage, &open, 0.4)?;
        line.wait_for(stage, "those two must say three and a half")?;
        solved.push(solve(&kit, stage, &pic, &columns, 1, &open)?);

        // 13_5: the third company; a halo round the values it keeps; its
        // line with the unknown; the countdown, nothing of the answer
        let line = say(&kit, stage, "13_5")?;
        line.wait_for(stage, "Four boxes make six pairs")?;
        caption = land(&kit, stage, &pic, &columns, PREDICT_STEP, caption)?;
        line.wait_for(stage, "The other three already have their values")?;
        let known: Vec<Mob> = columns[..PREDICT_STEP].iter().flat_map(Column::step).collect();
        glow(stage, &known, palette::estimate())?;
        open = company_line(&kit, stage, &pic, PREDICT_STEP, false)?;
        fade_in_all(stage, &open, 0.4)?;
        let q = think(&kit, stage, at(THINK_SPOT))?;

        // 13_6: the answer, its bar and a halo round its line; the last
        // company, its line solved and its bar
        let line = say(&kit, stage, "13_6")?;
        fade_out_all(stage, &[q], 0.3)?;
        line.wait_for(stage, "Five")?;
        let answer = solve(&kit, stage, &pic, &columns, PREDICT_STEP, &open)?;
        line.wait_for(stage, "the only value")?;
        glow(stage, &answer, palette::estimate())?;
        solved.push(answer);
        let last = pic.companies.len() - 1;
        line.wait_for(stage, "at five boxes")?;
        caption = land(&kit, stage, &pic, &columns, last, caption)?;
        open = company_line(&kit, stage, &pic, last, false)?;
        fade_in_all(stage, &open, 0.4)?;
        line.wait_for(stage, "forced to six and a half")?;
        solved.push(solve(&kit, stage, &pic, &columns, last, &open)?);

        // 13_7: the general step on the panel; a halo round the newest
        // column; the share at sixty boxes
        let line = say(&kit, stage, "13_7")?;
        let cues = [
            "the only new handfuls",
            "it's always their largest number",
            "Their share is the handful size over the count",
        ];
        let mut general = Vec::new();
        for (j, (words, cue)) in GENERAL.iter().zip(cues).enumerate() {
            line.wait_for(stage, cue)?;
            let m = words_left(&kit, stage, words, &[], v(PANEL_X, GENERAL_Y[j]))?;
            fade_in_all(stage, &[m], 0.5)?;
            general.push(m);
        }
        line.wait_for(stage, "exactly one unknown value")?;
        let newest = columns.last().map(Column::all).unwrap_or_default();
        glow(stage, &newest, palette::estimate())?;
        line.wait_for(stage, "five seen from exactly sixty boxes")?;
        let share = words_left(
            &kit,
            stage,
            &share_text(&pic).map_err(fail)?,
            &[],
            v(PANEL_X, GENERAL_Y[3]),
        )?;
        fade_in_all(stage, &[share], 0.5)?;
        general.push(share);

        // 13_8: a halo round each bar in turn; the general formula and a
        // halo round the staircase
        let line = say(&kit, stage, "13_8")?;
        line.wait_for(stage, "Each count fixes the next value")?;
        for column in &columns {
            glow(stage, &column.step(), palette::estimate())?;
        }
        line.wait_for(stage, "it's the gap recipe")?;
        fade_out_all(stage, &general, 0.4)?;
        let rule = words_left(
            &kit,
            stage,
            GENERAL_FORMULA,
            &formula_colors(),
            v(PANEL_X, GENERAL_Y[0]),
        )?;
        fade_in_all(stage, &[rule], 0.5)?;
        let bars: Vec<Mob> = columns.iter().flat_map(Column::step).collect();
        glow(stage, &bars, palette::estimate())?;

        // 13_9: the panel clears; the rival and its values beside the dots;
        // halos round its smallest and largest; its average
        let line = say(&kit, stage, "13_9")?;
        line.wait_for(stage, "Now take any unbiased recipe at all")?;
        let mut gone = vec![of_larger, rule];
        gone.extend(solved.concat());
        fade_out_all(stage, &gone, 0.4)?;
        line.wait_for(stage, "three times the smaller number, minus one")?;
        let rival = words_left(&kit, stage, &rival_text(&pic), &[], line_at(0))?;
        fade_in_all(stage, &[rival], 0.5)?;
        let values = rival_values(&kit, stage, &pic)?;
        fade_in_lagged(stage, &values, 1.5, 0.15)?;
        line.wait_for(stage, "from two all the way to eleven")?;
        let ends: Vec<(Vec<Mob>, Srgb)> = pic
            .rival_range
            .iter()
            .map(|&r| {
                let at_end: Vec<Mob> = values
                    .iter()
                    .zip(&pic.rival_estimates)
                    .filter(|&(_, &e)| e == Q::int(r as i128))
                    .map(|(&m, _)| m)
                    .collect();
                (at_end, Palette::ink())
            })
            .collect();
        glow_each(stage, &ends, 0.3, 0.8)?;
        line.wait_for(stage, "they still average five")?;
        let average = kit.text_left(stage, &rival_sum(&pic), LABEL_SIZE, Palette::ink(), line_at(1))?;
        fade_in_all(stage, &[average], 0.5)?;

        // 13_10: its averages in the groups, one after another; halos round
        // the badge and the staircase
        let line = say(&kit, stage, "13_10")?;
        line.wait_for(stage, "Average it inside the groups")?;
        let mut groups = Vec::new();
        for (g, &y) in (0..pic.companies.len()).zip(&LINES_Y[2..]) {
            let [left, avg] = group_pieces(&pic, g).map_err(fail)?;
            groups.extend(sum_texts(
                &kit,
                stage,
                &[(left, Palette::ink()), (avg, palette::estimate())],
                PANEL_X,
                y,
            )?);
        }
        fade_in_lagged(stage, &groups, 1.6, 0.3)?;
        line.wait_for(stage, "With a fair draw")?;
        glow(stage, &[corner], Palette::ink())?;
        line.wait_for(stage, "So they have to be the staircase again")?;
        glow(stage, &bars, palette::estimate())?;

        // 13_11: the two misses; the argument as a flow, a halo round its end
        let line = say(&kit, stage, "13_11")?;
        line.wait_for(stage, "The smaller-number recipe misses by two point four")?;
        fade_out_all(stage, &groups, 0.4)?;
        let [rival_miss, gap_miss] = miss_texts(&pic);
        let misses = vec![
            words_left(&kit, stage, &rival_miss, &[], line_at(2))?,
            words_left(
                &kit,
                stage,
                &gap_miss,
                &[(gap_miss.clone(), palette::estimate())],
                line_at(3),
            )?,
        ];
        fade_in_lagged(stage, &misses, 0.8, 0.4)?;
        line.wait_for(stage, "Averaging any unbiased recipe")?;
        let from = words_left(&kit, stage, FLOW[0], &[], v(PANEL_X, FLOW_Y[0]))?;
        fade_in_all(stage, &[from], 0.4)?;
        line.wait_for(stage, "inside the groups gives the gap recipe")?;
        let arrow = stroke(stage, flow_points(), line_style(Palette::ink(), ARROW_W, 1.0));
        let step = kit.text_left(
            stage,
            FLOW_STEP,
            LABEL_SIZE,
            Palette::dim(),
            v(FLOW_X + FLOW_TEXT_DX, (FLOW_Y[0] + FLOW_Y[1]) / 2.0),
        )?;
        let to = words_left(
            &kit,
            stage,
            FLOW[1],
            &[(FLOW[1].to_owned(), palette::estimate())],
            v(PANEL_X, FLOW_Y[1]),
        )?;
        fade_in_all(stage, &[arrow, step, to], 0.5)?;
        line.wait_for(stage, "never makes its misses bigger")?;
        let never = words_left(&kit, stage, FLOW[2], &[], v(PANEL_X, FLOW_Y[2]))?;
        fade_in_all(stage, &[never], 0.4)?;
        line.wait_for(stage, "So no unbiased recipe misses by less")?;
        glow(stage, &[to], palette::estimate())?;

        // 13_12: the flow goes; the tie at the smallest company, a halo round
        // its column
        let line = say(&kit, stage, "13_12")?;
        line.wait_for(stage, "Best doesn't mean every rival does worse")?;
        fade_out_all(stage, &[from, arrow, step, to, never], 0.4)?;
        line.wait_for(stage, "With exactly as many boxes as the handful holds")?;
        let mut first = columns[0].all();
        first.extend(
            values
                .iter()
                .zip(&pic.pairs)
                .filter(|(_, p)| p[p.len() - 1] == pic.companies[0])
                .map(|(&m, _)| m),
        );
        glow(stage, &first, Palette::ink())?;
        let tie = words_left(&kit, stage, &tie_text(&pic), &[], line_at(5))?;
        fade_in_all(stage, &[tie], 0.5)?;
        line.wait_for(stage, "doesn't change when you average it")?;
        let fixed = words_left(&kit, stage, TIE_AVERAGED, &[], line_at(6))?;
        fade_in_all(stage, &[fixed], 0.5)?;

        // 13_13: everything but the badge goes; the table, row by row, a
        // halo round the biased recipe's average
        let line = say(&kit, stage, "13_13")?;
        line.wait_for(stage, "A recipe that accepts a little bias")?;
        let mut gone: Vec<Mob> = columns.iter().flat_map(Column::all).collect();
        gone.extend(&values);
        gone.extend(&misses);
        gone.extend([caption, rival, average, tie, fixed]);
        fade_out_all(stage, &gone, 0.6)?;
        let (heads, rows) = table(&kit, stage, &pic)?;
        line.wait_for(stage, "Take seven sixths of the largest number")?;
        fade_in_all(stage, &heads, 0.5)?;
        line.wait_for(stage, "it says sixty-nine")?;
        fade_in_all(stage, &rows[0], 0.4)?;
        line.wait_for(stage, "it averages sixty-nine")?;
        fade_in_all(stage, &rows[1], 0.4)?;
        line.wait_for(stage, "two short of the count")?;
        glow(stage, &[rows[1][2]], Palette::ink())?;
        line.wait_for(stage, "about eight point seven")?;
        fade_in_all(stage, &rows[2], 0.4)?;
        line.wait_for(stage, "its squared misses are smaller too")?;
        fade_in_all(stage, &rows[3], 0.4)?;

        // 13_14: the smallest count's row and where squares lose; card I,
        // a halo round the badge, held until both can be read
        let line = say(&kit, stage, "13_14")?;
        line.wait_for(stage, "With exactly five boxes")?;
        fade_in_all(stage, &rows[4], 0.4)?;
        line.wait_for(stage, "bigger at every count from five to ten")?;
        let loses = words_left(&kit, stage, &loses_text(&pic), &[], v(TABLE_LEFT, LOSES_Y))?;
        fade_in_all(stage, &[loses], 0.5)?;
        let table_up = now(stage);
        line.wait_for(stage, "Among recipes that are right on average")?;
        let card = precisely(&kit, stage, CARD_I)?;
        fade_in_all(stage, &[card], 0.4)?;
        let card_up = now(stage);
        line.wait_for(stage, "as long as the handful is fair")?;
        glow(stage, &[corner], Palette::ink())?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;
        // card I and the table stay until they can be read
        wait_until(
            stage,
            (card_up + read_s(CARD_I)).max(table_up + read_s(&table_text(&pic))),
        )?;
        pause(stage, END_BEAT)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use explainer_kit::Script;
    use explainer_kit::animatic::wrap;
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;
    use crate::chapters::draw::{PAUSE_NOTE, THINK_R};
    use crate::chapters::friends_line::{BADGE, BADGE_PAD, BADGE_SIZE, est_width};
    use crate::chapters::what_we_assume::{HALO_PAD, SUM_GAP};
    use crate::facts::TINY_SEEN;

    /// A pair's size, as the companies' counts are written.
    const TINY_PAIR: u64 = TINY_SEEN as u64;
    /// The countdown's note, at the size `think` writes it.
    const NOTE_SIZE: f64 = 18.0;
    /// A line's half height at the panel's size and at the label size, and
    /// the space kept between things that must not touch.
    const HALF_LINE: f64 = 0.16;
    const HALF_SMALL: f64 = 0.12;
    const CLEAR: f64 = 0.1;
    /// A "precisely:" card's line, beside its label: the kit starts at 92
    /// characters and narrows by 4 until the widest line fits; its top over
    /// the frame's foot (0.2, then 0.3 plus 0.36 a line, three lines).
    const CARD_CHARS_WIDEST: usize = 92;
    const CARD_CHARS_NARROWEST: usize = 64;
    const CARD_TOP: f64 = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 3.0;

    fn good() -> Picture {
        picture().unwrap()
    }

    /// The scene draws the facts, and its lines read as the voice says
    /// them: "always 71", the captions and the company lines before and
    /// after they are solved, the share, the rival and its averages, the
    /// misses, the tie, the table and the line under it.
    #[test]
    fn the_picture_is_the_facts() {
        let pic = good();
        assert_eq!(always_text(&pic), "always 71: right on average only at 71");
        let captions: Vec<String> = (0..pic.companies.len()).map(|i| caption_text(&pic, i)).collect();
        assert_eq!(
            captions,
            [
                "2 boxes: 1 pair",
                "3 boxes: 3 pairs",
                "4 boxes: 6 pairs",
                "5 boxes: 10 pairs"
            ]
        );
        assert_eq!(
            company_pieces(&pic, 0, false),
            ["2 boxes: 1 ×", "?", "= 1 × 2"].map(str::to_owned)
        );
        assert_eq!(
            company_pieces(&pic, 0, true),
            ["2 boxes: 1 ×", "2", "= 1 × 2"].map(str::to_owned)
        );
        assert_eq!(
            company_pieces(&pic, 1, false),
            ["3 boxes: 1 × 2 + 2 ×", "?", "= 3 × 3"].map(str::to_owned)
        );
        assert_eq!(company_pieces(&pic, 1, true)[1], "3.5");
        assert_eq!(
            company_pieces(&pic, 2, false),
            ["4 boxes: 1 × 2 + 2 × 3.5 + 3 ×", "?", "= 6 × 4"].map(str::to_owned)
        );
        assert_eq!(company_pieces(&pic, 2, true)[1], "5");
        assert_eq!(
            company_pieces(&pic, 3, true),
            ["5 boxes: 1 × 2 + 2 × 3.5 + 3 × 5 + 4 ×", "6.5", "= 10 × 5"].map(str::to_owned)
        );
        assert_eq!(share_text(&pic).unwrap(), "5 seen, 60 boxes: 5 ÷ 60 = 1 in 12");
        assert_eq!(rival_text(&pic), "3 × smaller − 1");
        assert_eq!(rival_sum(&pic), "(2 + 2 + 2 + 2 + 5 + 5 + 5 + 8 + 8 + 11) ÷ 10 = 5");
        let groups: Vec<[String; 2]> = (0..pic.companies.len())
            .map(|g| group_pieces(&pic, g).unwrap())
            .collect();
        assert_eq!(
            groups,
            [
                ["topped by 2: 2 ÷ 1 =", "2"],
                ["topped by 3: (2 + 5) ÷ 2 =", "3.5"],
                ["topped by 4: (2 + 5 + 8) ÷ 3 =", "5"],
                ["topped by 5: (2 + 5 + 8 + 11) ÷ 4 =", "6.5"]
            ]
            .map(|p| p.map(str::to_owned))
        );
        assert_eq!(
            miss_texts(&pic),
            ["average miss, 3 × smaller − 1: 2.4", "average miss, gap recipe: 1.2"].map(str::to_owned)
        );
        assert_eq!(tie_text(&pic), "2 boxes: every unbiased recipe says 2");
        assert_eq!(shrunk_text(&pic), "7/6 × largest − 1");
        assert_eq!(
            row_labels(&pic),
            [
                "friends' five",
                "average, 71 boxes",
                "average miss",
                "average squared miss",
                "every handful, 5 boxes"
            ]
            .map(str::to_owned)
        );
        assert_eq!(
            cells(&pic),
            [
                ["71", "69"],
                ["71", "69"],
                ["≈ 9.2", "≈ 8.7"],
                ["≈ 135.8", "≈ 132.3"],
                ["5", "≈ 4.8"]
            ]
            .map(|r| r.map(str::to_owned))
        );
        assert_eq!(loses_text(&pic), "7/6: squared misses bigger at counts 5 to 10");
    }

    /// The negative controls: each picture the words or the scene would
    /// contradict is refused, one wrong fact at a time, for its reason.
    #[test]
    fn a_picture_the_words_contradict_is_refused() {
        let pic = good();
        let refused = |p: Picture, why: &str| {
            let e = checked(p).expect_err(why);
            assert!(e.contains(why), "refused for {e:?}, not {why:?}");
        };
        assert!(checked(pic.clone()).is_ok());
        // "always": a count inside the tiny company, or past every count checked
        refused(
            Picture {
                always: 4,
                ..pic.clone()
            },
            "always says",
        );
        refused(
            Picture {
                always: CHECK_TO + 1,
                ..pic.clone()
            },
            "always says",
        );
        // the companies: one short, a wrong pair count, a wrong new count,
        // the prediction moved
        refused(
            Picture {
                companies: vec![2, 3, 4],
                ..pic.clone()
            },
            "the companies",
        );
        refused(
            Picture {
                company_pairs: vec![1, 3, 6, 11],
                ..pic.clone()
            },
            "the companies",
        );
        refused(
            Picture {
                new_pairs: vec![1, 2, 3, 5],
                ..pic.clone()
            },
            "the companies",
        );
        refused(
            Picture {
                predict: 5,
                ..pic.clone()
            },
            "the companies",
        );
        // the staircase: a value off, a value rounded
        refused(
            Picture {
                forced: vec![Q::int(2), Q::new(7, 2), Q::int(5), Q::int(7)],
                ..pic.clone()
            },
            "the staircase",
        );
        let mut rounded = pic.forced_shown.clone();
        rounded[1] = "4".to_owned();
        refused(
            Picture {
                forced_shown: rounded,
                ..pic.clone()
            },
            "the staircase",
        );
        // the share, and the step unproved
        refused(
            Picture {
                share: Q::new(1, 11),
                ..pic.clone()
            },
            "the share of new handfuls",
        );
        refused(
            Picture {
                proved: false,
                ..pic.clone()
            },
            "the share of new handfuls",
        );
        // the rival: estimates swapped, a range, group averages or a gap
        // miss the words contradict; a rival that is biased, three times
        // the smaller, with its own estimates, range and average
        let mut swapped = pic.rival_estimates.clone();
        swapped.swap(0, 9);
        refused(
            Picture {
                rival_estimates: swapped,
                ..pic.clone()
            },
            "the rival",
        );
        refused(
            Picture {
                rival_range: vec![2, 8],
                ..pic.clone()
            },
            "the rival",
        );
        refused(
            Picture {
                rival_grouped: vec![Q::int(2), Q::new(7, 2), Q::int(5), Q::int(7)],
                ..pic.clone()
            },
            "the rival",
        );
        refused(
            Picture {
                gap_miss: "1.4".to_owned(),
                ..pic.clone()
            },
            "the rival",
        );
        let tripled: Vec<Q> = pic.pairs.iter().map(|p| Q::int(3 * p[0] as i128)).collect();
        let tripled_range = vec![3, 3 * (TINY - 1)];
        refused(
            Picture {
                rival: (Q::int(3), Q::ZERO),
                rival_average: mean(&tripled),
                rival_estimates: tripled,
                rival_range: tripled_range,
                ..pic.clone()
            },
            "the rival",
        );
        // the biased recipe at the count
        refused(
            Picture {
                shrunk_friends: Q::int(70),
                ..pic.clone()
            },
            "must average",
        );
        refused(
            Picture {
                gap_friends: 70,
                ..pic.clone()
            },
            "must average",
        );
        refused(
            Picture {
                bias: Q::int(-1),
                ..pic.clone()
            },
            "must average",
        );
        refused(
            Picture {
                shrunk_miss: "8.8".to_owned(),
                ..pic.clone()
            },
            "must average",
        );
        refused(
            Picture {
                shrunk_squared_shown: "132.4".to_owned(),
                ..pic.clone()
            },
            "must average",
        );
        refused(
            Picture {
                factor: Q::new(8, 7),
                ..pic.clone()
            },
            "must average",
        );
        // where it loses: a run cut short, a rounding, another count
        refused(
            Picture {
                loses_squared: (5..=9).collect(),
                ..pic.clone()
            },
            "must be beaten",
        );
        refused(
            Picture {
                shrunk_edge_shown: "4.9".to_owned(),
                ..pic.clone()
            },
            "must be beaten",
        );
        refused(Picture { edge: 6, ..pic }, "must be beaten");
    }

    /// The general step (13_7) at every handful size from 1 to 6 and every
    /// count to 40: the new handfuls, those that contain N, are a share
    /// k ÷ N, never none, so the step's one unknown is fixed, and it is the
    /// gap recipe's value; solving count by count gives the model's
    /// staircase. A finite control only: the proof for every count is the
    /// polynomial identity the oracle and facts.rs check
    /// (G-F13.recurrence_proved).
    #[test]
    fn each_count_fixes_one_value() {
        for k in 1..=6u64 {
            let mut g: Vec<Q> = Vec::new();
            for n in k..=40u64 {
                let new = model::with_largest(n, k);
                let all = Q::count(choose(n, k));
                assert!(new > Q::ZERO, "k {k}, N {n}");
                assert_eq!(new / all, Q::new(k as i128, n as i128), "k {k}, N {n}");
                let known = (k..n)
                    .zip(&g)
                    .fold(Q::ZERO, |s, (m, &value)| s + model::with_largest(m, k) * value);
                let value = (Q::int(n as i128) * all - known) / new;
                assert_eq!(value, model::gap_of(n, k), "k {k}, N {n}");
                g.push(value);
            }
            assert_eq!(g, model::forced_values(k, 40), "k {k}");
        }
    }

    /// The negative control: a staircase for pairs bumped at one value is
    /// biased at exactly the counts from that value up, so no value can be
    /// changed without breaking unbiasedness somewhere.
    #[test]
    fn a_staircase_off_at_one_value_is_biased_from_there_up() {
        let k = TINY_PAIR;
        for bump in k..=8u64 {
            let g = |m: u64| model::gap_of(m, k) + if m == bump { Q::ONE } else { Q::ZERO };
            for n in k..=12u64 {
                let biased = model::mean_by_largest(n, k, g) != Q::int(n as i128);
                assert_eq!(biased, n >= bump, "bump {bump}, N {n}");
            }
        }
    }

    /// The rival for pairs is unbiased at every count from 2 to 12, its
    /// group averages are the staircase at every count, and it misses at
    /// least as much as the gap recipe, plain and squared, its squared
    /// misses strictly more wherever it is not already a recipe of the
    /// larger number (every count from three boxes).
    #[test]
    fn the_rival_averages_back_to_the_staircase() {
        let pic = good();
        let (a, b) = pic.rival;
        for c in TINY_PAIR..=12u64 {
            let pairs = model::all_handfuls(c, TINY_PAIR as usize);
            let n = Q::int(c as i128);
            let rival: Vec<Q> = pairs.iter().map(|p| a * Q::int(p[0] as i128) + b).collect();
            let gap: Vec<Q> = pairs.iter().map(|p| model::gap_recipe(p)).collect();
            assert_eq!(mean(&rival), n, "c {c}");
            for m in TINY_PAIR..=c {
                let group: Vec<Q> = pairs
                    .iter()
                    .zip(&rival)
                    .filter(|(p, _)| p[1] == m)
                    .map(|(_, &e)| e)
                    .collect();
                assert_eq!(mean(&group), model::gap_of(m, TINY_PAIR), "c {c}, m {m}");
            }
            let off = |xs: &[Q], f: &dyn Fn(Q) -> Q| mean(&xs.iter().map(|&e| f(e - n)).collect::<Vec<_>>());
            let plain = |x: Q| x.abs();
            let square = |x: Q| x * x;
            assert!(off(&rival, &plain) >= off(&gap, &plain), "c {c}");
            assert!(off(&rival, &square) >= off(&gap, &square), "c {c}");
            assert_eq!(off(&rival, &square) > off(&gap, &square), c > TINY_PAIR, "c {c}");
        }
    }

    /// The biased recipe averages seven sixths of the largest number's
    /// average, less one, short of the count at every count from five to a
    /// hundred; it misses by less than the gap recipe at 71 boxes, plain and
    /// squared, but at five boxes the gap recipe is exact and it is not. The
    /// facts list the counts where its plain miss is not smaller (5 and 11):
    /// bigger only at the smallest count, a tie at the other, so the scene
    /// claims nothing about plain losses. It does not dominate, and nothing
    /// here says it is better everywhere.
    #[test]
    fn the_biased_recipe_wins_somewhere_and_loses_somewhere() {
        let pic = good();
        let h = pic.handful;
        let (a, b) = pic.shrunk;
        let shrunk = |m: u64| a * Q::int(m as i128) + b;
        let gap = |m: u64| model::gap_of(m, h);
        for n in h..=CHECK_TO {
            let largest_average = Q::new((h * (n + 1)) as i128, (h + 1) as i128);
            let average = model::mean_by_largest(n, h, shrunk);
            assert_eq!(average, a * largest_average + b, "N {n}");
            assert!(average < Q::int(n as i128), "N {n}");
            assert_eq!(model::mean_by_largest(n, h, gap), Q::int(n as i128), "N {n}");
        }
        assert_eq!(
            model::mean_by_largest(pic.count, h, shrunk) - Q::int(pic.count as i128),
            pic.bias
        );
        assert!(plain_at(pic.count, h, &shrunk) < plain_at(pic.count, h, &gap));
        assert!(squared_at(pic.count, h, &shrunk) < squared_at(pic.count, h, &gap));
        assert_eq!(squared_at(pic.edge, h, &gap), Q::ZERO);
        assert!(squared_at(pic.edge, h, &shrunk) > Q::ZERO);
        assert!(plain_at(pic.edge, h, &shrunk) > plain_at(pic.edge, h, &gap));
        let not_less: Vec<u64> = (h..=CHECK_TO)
            .filter(|&n| plain_at(n, h, &shrunk) >= plain_at(n, h, &gap))
            .collect();
        let bigger: Vec<u64> = (h..=CHECK_TO)
            .filter(|&n| plain_at(n, h, &shrunk) > plain_at(n, h, &gap))
            .collect();
        let f = facts::facts().unwrap();
        assert_eq!(not_less, list(&f, "G-F13.shrunk_loses_plain_at").unwrap());
        assert_eq!(bigger, [pic.edge]);
        // squared, the words' "bigger" is strict: no count ties
        let squared_not_less: Vec<u64> = (h..=CHECK_TO)
            .filter(|&n| squared_at(n, h, &shrunk) >= squared_at(n, h, &gap))
            .collect();
        assert_eq!(squared_not_less, pic.loses_squared);
    }

    /// Every phrase the scene waits for is in its line of script.tsv, once,
    /// and a line's phrases come in the order the scene waits for them; the
    /// prediction's line shows nothing of the answer.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let waits: &[(&str, &[&str])] = &[
            ("13_1", &["every recipe that's unbiased"]),
            (
                "13_2",
                &[
                    "Unbiased has to hold at every count",
                    "A recipe that always says seventy-one",
                    "So it isn't unbiased",
                ],
            ),
            (
                "13_3",
                &[
                    "Start with pairs",
                    "The smallest company with a pair",
                    "To average two",
                    "must say two when the larger number is two",
                ],
            ),
            (
                "13_4",
                &[
                    "Three boxes make three pairs",
                    "The pair topped by two must still say two",
                    "all three estimates must average three",
                    "those two must say three and a half",
                ],
            ),
            (
                "13_5",
                &["Four boxes make six pairs", "The other three already have their values"],
            ),
            (
                "13_6",
                &["Five", "the only value", "at five boxes", "forced to six and a half"],
            ),
            (
                "13_7",
                &[
                    "the only new handfuls",
                    "it's always their largest number",
                    "Their share is the handful size over the count",
                    "exactly one unknown value",
                    "five seen from exactly sixty boxes",
                ],
            ),
            ("13_8", &["Each count fixes the next value", "it's the gap recipe"]),
            (
                "13_9",
                &[
                    "Now take any unbiased recipe at all",
                    "three times the smaller number, minus one",
                    "from two all the way to eleven",
                    "they still average five",
                ],
            ),
            (
                "13_10",
                &[
                    "Average it inside the groups",
                    "With a fair draw",
                    "So they have to be the staircase again",
                ],
            ),
            (
                "13_11",
                &[
                    "The smaller-number recipe misses by two point four",
                    "Averaging any unbiased recipe",
                    "inside the groups gives the gap recipe",
                    "never makes its misses bigger",
                    "So no unbiased recipe misses by less",
                ],
            ),
            (
                "13_12",
                &[
                    "Best doesn't mean every rival does worse",
                    "With exactly as many boxes as the handful holds",
                    "doesn't change when you average it",
                ],
            ),
            (
                "13_13",
                &[
                    "A recipe that accepts a little bias",
                    "Take seven sixths of the largest number",
                    "it says sixty-nine",
                    "it averages sixty-nine",
                    "two short of the count",
                    "about eight point seven",
                    "its squared misses are smaller too",
                ],
            ),
            (
                "13_14",
                &[
                    "With exactly five boxes",
                    "bigger at every count from five to ten",
                    "Among recipes that are right on average",
                    "as long as the handful is fair",
                ],
            ),
        ];
        for &(id, phrases) in waits {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            let mut last = 0;
            for p in phrases {
                assert_eq!(text.matches(p).count(), 1, "{id}: {p:?}");
                let i = text.find(p).unwrap();
                assert!(i >= last, "{id}: {p:?} comes after the phrase before it");
                last = i;
            }
        }
        let asked = script.text("13_5").unwrap();
        for answer in ["Five", "five", "5"] {
            assert!(!asked.contains(answer), "{answer}");
        }
    }

    /// The grid, the bars, the panel and the table keep apart: the smaller
    /// numbers left of the first column, each rival value clear of its dot
    /// and of the next column, the top row under the badge, the numbers
    /// between the dots and the tallest bar's value, the caption under the
    /// bars; every panel line, small line and the flow in the title-safe
    /// area and apart; the countdown under the prediction's line; the
    /// table's columns apart and its last line over card I.
    #[test]
    fn the_grid_the_bars_the_panel_and_the_table_fit_the_frame() {
        let pic = good();
        let k = pic.seen;
        let safe_x = FRAME_X_RADIUS - 0.25;
        let w = |s: &str| est_width(s, PANEL_SIZE);
        let ws = |s: &str| est_width(s, LABEL_SIZE);
        let badge_half = est_width(BADGE, BADGE_SIZE) / 2.0 + BADGE_PAD[0];
        let badge_foot = BADGE_AT[1] - 0.15 - BADGE_PAD[1];
        assert!(BADGE_AT[0] - badge_half > -safe_x);
        assert!(BADGE_AT[0] + badge_half < PANEL_X - CLEAR);
        // the grid
        let label_x = col_x(k, k) - ROW_LABEL_DX;
        let label_half = ws(&(TINY - 1).to_string()) / 2.0;
        assert!(label_x - label_half > -safe_x);
        assert!(label_x + label_half + CLEAR < col_x(k, k) - PAIR_R);
        let value_half = pic
            .rival_estimates
            .iter()
            .map(|e| ws(&e.to_string()))
            .fold(0.0, f64::max)
            / 2.0;
        assert!(PAIR_R + CLEAR / 2.0 < RIVAL_DX - value_half);
        for (&m, &next) in pic.companies.iter().zip(pic.companies.iter().skip(1)) {
            assert!(
                col_x(m, k) + RIVAL_DX + value_half + CLEAR < col_x(next, k) - PAIR_R,
                "{m}"
            );
        }
        assert!(col_x(TINY, k) + RIVAL_DX + value_half + CLEAR < PANEL_X);
        assert!(row_y(TINY - 1) + PAIR_R + CLEAR < badge_foot);
        const { assert!(2.0 * HALF_SMALL + 2.0 * HALO_PAD < GRID_ROW) };
        const { assert!(NUMBER_Y + HALF_SMALL + CLEAR < GRID_Y0 - PAIR_R) };
        // the bars and the caption
        let tallest = pic.forced.iter().map(|&g| bar_h(g).unwrap()).fold(0.0, f64::max);
        assert!(BAR_FOOT + tallest + VALUE_DY + HALF_SMALL + CLEAR < NUMBER_Y - HALF_SMALL);
        const { assert!(BAR_W + CLEAR < GRID_STEP) };
        const { assert!(CAPTION_Y + HALF_SMALL + CLEAR / 2.0 < BAR_FOOT) };
        const { assert!(CAPTION_Y - HALF_SMALL > -FRAME_Y_RADIUS + 0.25) };
        let caption_half = (0..pic.companies.len())
            .map(|i| ws(&caption_text(&pic, i)))
            .fold(0.0, f64::max)
            / 2.0;
        assert!(caption_x(&pic) - caption_half > -safe_x);
        assert!(caption_x(&pic) + caption_half < PANEL_X);
        // the panel's lines at its size
        let mut lines = vec![
            AVERAGED_IS.to_owned(),
            ANY_UNBIASED.to_owned(),
            UNBIASED_IS.to_owned(),
            always_text(&pic),
            OF_LARGER.to_owned(),
            share_text(&pic).unwrap(),
            GENERAL_FORMULA.to_owned(),
            rival_text(&pic),
            tie_text(&pic),
            TIE_AVERAGED.to_owned(),
        ];
        lines.extend(GENERAL.iter().chain(&FLOW).map(|s| (*s).to_owned()));
        lines.extend(miss_texts(&pic));
        for text in &lines {
            assert!(PANEL_X + w(text) < safe_x, "{text}");
        }
        for g in 0..pic.companies.len() {
            let [a, b] = group_pieces(&pic, g).unwrap();
            assert!(PANEL_X + w(&a) + SUM_GAP + w(&b) < safe_x, "{a}");
        }
        // the small lines: the company lines, the rival's average, the flow's step
        for i in 0..pic.companies.len() {
            for solved in [false, true] {
                let pieces = company_pieces(&pic, i, solved);
                let width: f64 = pieces.iter().map(|s| ws(s)).sum::<f64>() + 2.0 * SMALL_GAP;
                assert!(PANEL_X + width < safe_x, "{}", pieces.join(" "));
            }
        }
        assert!(PANEL_X + ws(&rival_sum(&pic)) < safe_x);
        assert!(FLOW_X + FLOW_TEXT_DX + ws(FLOW_STEP) < safe_x);
        for p in LINES_Y.windows(2) {
            assert!(p[0] - HALF_LINE > p[1] + HALF_LINE);
        }
        const { assert!(LINES_Y[STEPS] - HALF_SMALL - CLEAR > GENERAL_Y[0] + HALF_LINE) };
        for p in GENERAL_Y.windows(2) {
            assert!(p[0] - HALF_LINE > p[1] + HALF_LINE);
        }
        const { assert!(GENERAL_Y[3] - HALF_LINE > -FRAME_Y_RADIUS + 0.25) };
        // the flow (13_11) under the misses, its arrow between its lines
        const { assert!(FLOW_Y[0] + HALF_LINE + CLEAR < LINES_Y[3] - HALF_LINE) };
        const { assert!(FLOW_PAD > HALF_LINE) };
        const { assert!(FLOW_Y[0] - FLOW_PAD - (FLOW_Y[1] + FLOW_PAD) > 2.0 * FLOW_HEAD) };
        const { assert!(FLOW_Y[1] - HALF_LINE > FLOW_Y[2] + HALF_LINE) };
        const { assert!(FLOW_X - ARROW_SPREAD * FLOW_HEAD > PANEL_X) };
        // the countdown (13_5) under the prediction's line
        let note_half = est_width(PAUSE_NOTE, NOTE_SIZE) / 2.0;
        const { assert!(THINK_SPOT[1] + THINK_R + CLEAR < LINES_Y[PREDICT_STEP + 1] - HALF_SMALL) };
        assert!(THINK_SPOT[0] + note_half.max(THINK_R) < safe_x);
        assert!(THINK_SPOT[0] - note_half.max(THINK_R) > col_x(TINY, k) + RIVAL_DX);
        const { assert!(THINK_SPOT[1] - THINK_R - 0.3 - HALF_SMALL > -FRAME_Y_RADIUS + 0.25) };
        // the table (13_13, 13_14)
        let label_end = TABLE_LEFT + row_labels(&pic).iter().map(|s| w(s)).fold(0.0, f64::max);
        let all = cells(&pic);
        let heads = [GAP_HEAD.to_owned(), shrunk_text(&pic)];
        let col = |c: usize| all.iter().map(|r| w(&r[c])).chain([w(&heads[c])]).fold(0.0, f64::max) / 2.0;
        const { assert!(TABLE_LEFT > -(FRAME_X_RADIUS - 0.25)) };
        assert!(label_end + CLEAR < TABLE_X[0] - col(0));
        assert!(TABLE_X[0] + col(0) + CLEAR < TABLE_X[1] - col(1));
        assert!(TABLE_X[1] + col(1) < safe_x);
        const { assert!(TABLE_HEAD_Y + HALF_LINE < BADGE_AT[1] - 0.15 - BADGE_PAD[1] - CLEAR) };
        const { assert!(TABLE_HEAD_Y - HALF_LINE > TABLE_Y[0] + HALF_LINE) };
        for p in TABLE_Y.windows(2) {
            assert!(p[0] - HALF_LINE > p[1] + HALF_LINE);
        }
        const { assert!(TABLE_Y[4] - HALF_LINE > LOSES_Y + HALF_LINE) };
        const { assert!(LOSES_Y - HALF_LINE > CARD_TOP + CLEAR) };
        assert!(TABLE_LEFT + w(&loses_text(&pic)) < safe_x);
    }

    /// Card I says what is proved, the fairness first, unbiased at every
    /// count, and no more (no strict gain, no rival beaten everywhere), in
    /// at most three lines at every width the lower third might narrow to,
    /// and is up long enough to be read.
    #[test]
    fn card_i_fits_and_is_read() {
        for chars in (CARD_CHARS_NARROWEST..=CARD_CHARS_WIDEST).step_by(4) {
            assert!(wrap(CARD_I, chars, 4).len() <= 3, "{chars}");
        }
        assert!(CARD_I.starts_with("fair handful"));
        assert!(CARD_I.contains("every N ≥ k") && CARD_I.contains("the only unbiased recipe of M"));
        assert!(!CARD_I.contains("strictly") && !CARD_I.contains("every rival"));
        assert!(read_s(CARD_I) < 15.0);
        // the negative control: one clause more would not fit
        let longer = format!("{CARD_I} The count is one number.");
        assert!(wrap(&longer, CARD_CHARS_NARROWEST, 4).len() > 3);
    }
}
