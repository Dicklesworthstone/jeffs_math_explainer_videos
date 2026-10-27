//! Chapter 10, "A rival recipe": on the friends' line, their estimate a
//! green tick at seventy-one, a second recipe is earned from the middle
//! number. A company's numbers balance at their middle, thirty-six for
//! seventy-one boxes, so the count is twice the middle minus one (10_1); a
//! fair handful's average stands in for the middle: the doubling recipe,
//! twice the average minus one (10_2). The friends' five average
//! thirty-seven, so doubling says seventy-three (10_3), and it is unbiased
//! too (10_4). The prediction: a strange handful keeps the box at sixty and
//! makes the other four two, three, four and five (10_5); doubling says
//! twenty-eight point six, fewer than the sixty boxes the handful proves,
//! where the gap recipe says seventy-one (10_6); in the 71-box company about
//! one handful in five gets a doubling estimate below its own largest number
//! (10_7). Every handful of that company, both recipes on one scale: the
//! line's end slides out, chapter 08's green bars come back and the doubling
//! recipe's handfuls, counted by total, rise as a purple outline from five
//! to a hundred and thirty-seven (10_8). Both balance at seventy-one; the
//! purple is lower and wider: the average miss ignoring direction, 14.3
//! against 9.2, and the standard deviation, 17.8 against 11.7, held to be
//! read (10_9); the recipe that uses only the largest number beats the one
//! that uses all five (10_10).
//!
//! The purple outline is built from how many handfuls have each total, from
//! a sum-of-subsets table, never by listing handfuls, and the scene checks
//! every value it shows against the facts. Highlights are halos and colour
//! changes are replacements: no typeset text is indicated, recoloured or
//! scaled in place.

use explainer_kit::{Kit, Palette, choose, fade_all, tween, v};
use fmn::prelude::*;

use super::draw::{
    STRIP_Y, TICK_W, dot_opacity, fade_in_all, fade_out_all, fail, line_style, pause, refill, say, stroke, think,
    wait_until, x_at,
};
use super::friends_line::{
    BADGE_AT, FriendsLine, PANEL_SIZE, PANEL_Y, Start, THINK_AT, at, badge, bind_to, is_handful, list, now, panel,
    printed, read_s, reshape, ring, whole,
};
use super::gaps_trade_places::glow_each;
use super::one_estimate::{
    BAR_O, SPAN_W, approx, bar, count_mark, count_text, fraction, grow_bars, right_at, span_points,
};
use super::what_we_assume::{glow, replace, words_left};
use crate::facts;
use crate::model::{self, LINE_DOTS, Q};
use crate::palette;

pub struct ARivalRecipe {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const RECIPE: &str = "doubling recipe";
const DOUBLING: &str = "doubling";
const BELOW_WORDS: &str = "doubling below their own largest";
const HEADS: [&str; 2] = ["gap", "doubling"];
const ROWS: [&str; 2] = ["average miss", "standard deviation"];

/// The line's last number once its end has slid out (10_8): past the
/// doubling recipe's highest estimate; never written or said.
const VIEW_LAST: u64 = 140;
const VIEW_END: f64 = VIEW_LAST as f64;
/// The gap recipe's estimate: its green tick across the line and its number
/// over it (as chapters 04, 07 and 08 draw them).
const TICK_DOWN: f64 = 0.12;
const TICK_UP: f64 = 0.3;
const ESTIMATE_DY: f64 = 0.55;
/// The middle (10_1): an ink span under the line from the first number to
/// the count, a fulcrum under it at the middle and the middle's number
/// under that.
const SPAN_Y: f64 = STRIP_Y - 0.4;
const MIDDLE_Y: f64 = SPAN_Y - 0.6;
/// A fulcrum, a small triangle standing on its tip (as chapter 08 draws
/// it), just under what it balances.
const FULCRUM: [f64; 2] = [0.14, 0.3];
const FULCRUM_GAP: f64 = 0.04;
const FULCRUM_W: f64 = 2.0;
/// The doubling recipe's values under the line (10_3, 10_6): a short purple
/// mark starting below the green tick's reach, its number under it.
const MARK_FROM: f64 = STRIP_Y - 0.15;
const MARK_TO: f64 = STRIP_Y - 0.45;
const MARK_NUMBER_Y: f64 = STRIP_Y - 0.7;
/// Words under the line: the strange handful's numbers (10_5) and the
/// doubling estimates' range (10_8).
const UNDER_Y: f64 = STRIP_Y - 0.35;
/// The company's count (10_8): an upright ink mark up to just over the
/// green bars around it.
const COUNT_TOP: f64 = STRIP_Y + 1.35;
/// The bars (10_8): chapter 08's green ones, narrower at this scale, the
/// tallest `BARS_H` high, and the purple outline on the same scale, stroke
/// only, one short stroke per step.
const GREEN_W: f64 = 0.08;
const BARS_H: f64 = 2.2;
const OUTLINE_W: f64 = 2.5;
/// Where the outline's rise starts: just off the line, so no edge between
/// two different steps has zero length.
const RISE_FROM: f64 = 0.01;
/// The balance (10_9): the fulcrum under the line at the count, its words
/// to its left.
const BALANCE_Y: f64 = STRIP_Y - 0.2;
const BALANCE_GAP: f64 = 0.3;
/// The table on the panel (10_9): its columns' centres, right of the
/// longer row label.
const TABLE_X: [f64; 2] = [3.7, 5.3];
/// The faded dots evening out, the zoom, the bars rising; the inspection
/// holds (10_9, 10_10).
const EVEN_S: f64 = 0.4;
const ZOOM_S: f64 = 2.0;
const BARS_S: f64 = 1.6;
const BARS_LAG: f64 = 0.05;
const RISE_S: f64 = 1.6;
const HOLD_S: f64 = 2.5;

/// What chapter 10 shows, from the facts: the friends' five, the line's
/// first number, the handful size and their estimate; the company's count,
/// its middle, the rule that gives the count back from the middle and the
/// doubling recipe; the friends' total, average and doubling estimate; how
/// many others there are, the strange handful, its total, average and
/// doubling estimate exact and as shown, its gap estimate and the lower
/// bound; the share of handfuls doubling puts below their own largest, its
/// "1 in" and its percent; chapter 08's stacks and its tallest; the doubling
/// estimates' range, how many purple bars, the tallest and where; both
/// balance points; both average misses exact and as shown, both variances
/// and their roots as shown, and their ratio.
#[derive(Clone, Debug)]
struct Picture {
    numbers: Vec<u64>,
    first: u64,
    k: usize,
    estimate: u64,
    count: u64,
    middle: Q,
    from_middle: String,
    recipe: String,
    sum: u64,
    average: Q,
    doubling: Q,
    others: usize,
    silly: Vec<u64>,
    silly_sum: u64,
    silly_average: String,
    silly_doubling: Q,
    silly_shown: String,
    silly_gap: Q,
    at_least: u64,
    below: Q,
    below_one_in: u64,
    below_percent: String,
    stacks: usize,
    top_count: u64,
    range: Vec<Q>,
    bars: usize,
    tallest: u64,
    tallest_at: Q,
    balance: Q,
    gap_balance: Q,
    miss: Q,
    miss_shown: String,
    gap_miss: Q,
    gap_miss_shown: String,
    variance: Q,
    sd: String,
    gap_variance: Q,
    gap_sd: String,
    ratio: Q,
}

fn picture() -> Result<Picture, String> {
    let f = facts::facts()?;
    let q = |key: &str| -> Result<Q, String> { fraction(&printed(&f, key)?) };
    checked(Picture {
        numbers: list(&f, "G-F1.numbers")?,
        first: whole(&f, "G-F1.first")?,
        k: whole(&f, "G-F1.handful")? as usize,
        estimate: whole(&f, "G-F3.estimate")?,
        count: whole(&f, "G-F8.count")?,
        middle: q("G-F10.middle_71")?,
        from_middle: printed(&f, "G-F10.from_middle")?,
        recipe: printed(&f, "G-F10.recipe")?,
        sum: whole(&f, "G-F10.sum")?,
        average: q("G-F10.average")?,
        doubling: q("G-F10.doubling")?,
        others: whole(&f, "G-F10.others")? as usize,
        silly: list(&f, "G-F10.silly_handful")?,
        silly_sum: whole(&f, "G-F10.silly_sum")?,
        silly_average: printed(&f, "G-F10.silly_average_decimal")?,
        silly_doubling: q("G-F10.silly_doubling")?,
        silly_shown: printed(&f, "G-F10.silly_doubling_decimal")?,
        silly_gap: q("G-F10.silly_gap")?,
        at_least: whole(&f, "G-F1.at_least")?,
        below: q("G-F10.below_largest_71")?,
        below_one_in: whole(&f, "G-F10.below_largest_71_one_in")?,
        below_percent: printed(&f, "G-F10.below_largest_71_percent1")?,
        stacks: whole(&f, "G-F8.stacks")? as usize,
        top_count: whole(&f, "G-F8.topped_by_count")?,
        range: printed(&f, "G-F10.doubling_range_71")?
            .split(',')
            .map(fraction)
            .collect::<Result<_, _>>()?,
        bars: whole(&f, "G-F10.bars_71")? as usize,
        tallest: whole(&f, "G-F10.tallest_bar_71")?,
        tallest_at: q("G-F10.tallest_bar_71_at")?,
        balance: q("G-F10.doubling_average_71")?,
        gap_balance: q("G-F8.average")?,
        miss: q("G-F10.doubling_miss_71")?,
        miss_shown: printed(&f, "G-F10.doubling_miss_71_decimal")?,
        gap_miss: q("G-F8.average_miss")?,
        gap_miss_shown: printed(&f, "G-F8.average_miss_decimal")?,
        variance: q("G-F10.doubling_variance_71")?,
        sd: printed(&f, "G-F10.doubling_sd_about")?,
        gap_variance: q("G-F8.variance")?,
        gap_sd: printed(&f, "G-F8.sd_about")?,
        ratio: q("G-F10.variance_ratio")?,
    })
}

/// A rule's multiplier and added number as the facts print it: "2*middle-1"
/// for `var` "middle" is (2, −1).
fn linear(s: &str, var: &str) -> Result<(Q, Q), String> {
    let bad = || format!("{s:?} is not a multiple of {var} plus a number");
    let (a, b) = s.split_once(&format!("*{var}")).ok_or_else(bad)?;
    Ok((fraction(a).map_err(|_| bad())?, fraction(b).map_err(|_| bad())?))
}

/// The middle of the numbers 1 to n: their total over how many there are.
fn middle_of(n: u64) -> Q {
    Q::new((n * (n + 1) / 2) as i128, n as i128)
}

/// One step of the purple outline: its edges on the line and how many
/// handfuls it holds.
#[derive(Clone, Copy, Debug)]
struct Step {
    left: f64,
    right: f64,
    many: u128,
}

/// The outline's steps on a line shown to `end`, one per purple bar of the
/// handfuls of `k` from `n` boxes counted by total: (k + 1)/k of estimate
/// wide, as far as a green bar's estimate moves per largest number, centred
/// on the gap recipe's value for the bar's largest number.
fn steps_of(n: u64, k: usize, end: f64) -> Result<Vec<Step>, String> {
    let kk = k as u64;
    let half = Q::new(kk as i128 + 1, 2 * kk as i128);
    facts::purple_bars(&model::handfuls_by_sum(n, k))
        .into_iter()
        .map(|(m, many)| {
            let c = model::gap_of(m, kk);
            Ok(Step {
                left: x_at(approx(c - half)?, end),
                right: x_at(approx(c + half)?, end),
                many,
            })
        })
        .collect()
}

/// Step `i`'s piece of the outline, at the share `rise` of its full height
/// (the tallest green bar, `top` handfuls, is `BARS_H`): up or down its left
/// edge from the step before (the first from the line), then across its
/// top; the last also down its right edge to the line. Neighbours holding
/// as many handfuls share one height, with no edge between them.
fn step_points(steps: &[Step], i: usize, top: u64, rise: f64) -> Vec<Vec3> {
    let y = |many: u128| STRIP_Y + rise * BARS_H * many as f64 / top as f64;
    let s = steps[i];
    let before = if i == 0 { 0 } else { steps[i - 1].many };
    let mut points = Vec::with_capacity(4);
    if before != s.many {
        points.push(v(s.left, y(before)));
    }
    points.extend([v(s.left, y(s.many)), v(s.right, y(s.many))]);
    if i + 1 == steps.len() {
        points.push(v(s.right, STRIP_Y));
    }
    points
}

/// The picture only if the scene can draw it truthfully: the friends' five
/// are a handful on the line from 1, their estimate the gap recipe's and
/// the company's count that estimate; the middle is the numbers' average
/// and the rule gives every count from 1 to 200 back from its middle; the
/// doubling recipe is that rule with the average for the middle, the
/// friends' total, average and estimate are theirs, just above the gap
/// recipe's; both recipes balance at the count; the strange handful keeps
/// the largest box, its total, average and estimates are its own, doubling's
/// under the lower bound and the gap recipe's the friends'; the share below
/// the largest is the facts' and the gap recipe is never below it; the bars
/// hold every handful once, the green ones chapter 08's and the purple ones
/// from the sum table, their range, number and tallest the facts' and the
/// line long enough for them; both widths are the facts', doubling's wider
/// by both, each average miss squared under its variance and the variances
/// in the closed forms' ratio.
fn checked(pic: Picture) -> Result<Picture, String> {
    let k = pic.k as u64;
    let n = pic.count;
    let target = Q::int(n as i128);
    let largest = *pic.numbers.last().ok_or("the friends' handful is empty")?;
    if pic.first != 1
        || !is_handful(&pic.numbers, pic.k, LINE_DOTS)
        || model::gap_recipe(&pic.numbers) != Q::int(pic.estimate as i128)
        || n != pic.estimate
    {
        return Err(format!(
            "the company's count {n} must be the friends' estimate {} from {:?}",
            pic.estimate, pic.numbers
        ));
    }
    let (a, b) = linear(&pic.from_middle, "middle")?;
    if pic.middle != middle_of(n)
        || a * pic.middle + b != target
        || !(1..=200).all(|c| a * middle_of(c) + b == Q::int(c as i128))
        || a.whole().is_none_or(|w| w < 1)
        || b >= Q::ZERO
    {
        return Err(format!(
            "the middle {} of {n} boxes must give every count back as {}",
            pic.middle, pic.from_middle
        ));
    }
    let (ra, rb) = linear(&pic.recipe, "average")?;
    let total: u64 = pic.numbers.iter().sum();
    if (ra, rb) != (a, b)
        || pic.sum != total
        || pic.average != Q::new(total as i128, k as i128)
        || pic.doubling != model::doubling_recipe(&pic.numbers)
        || pic.doubling != ra * pic.average + rb
        || pic.doubling <= Q::int(pic.estimate as i128)
    {
        return Err(format!(
            "the doubling recipe {} must give {} from the total {}, above {}",
            pic.recipe, pic.doubling, pic.sum, pic.estimate
        ));
    }
    if model::mean_by_sum(n, pic.k, |s| model::doubling_of(s, k)) != target
        || model::mean_by_sum(n, pic.k, |s| Q::new(s as i128, k as i128)) != pic.middle
        || pic.balance != target
        || pic.gap_balance != target
        || model::mean_by_largest(n, k, |m| model::gap_of(m, k)) != target
    {
        return Err(format!(
            "both recipes must balance at {n}, not {} and {}",
            pic.balance, pic.gap_balance
        ));
    }
    let silly_total: u64 = pic.silly.iter().sum();
    let silly_average = Q::new(silly_total as i128, k as i128);
    if pic.others + 1 != pic.k
        || !is_handful(&pic.silly, pic.k, LINE_DOTS)
        || pic.silly.last() != Some(&largest)
        || pic.silly_sum != silly_total
        || pic.silly_average != silly_average.decimal1()?
        || pic.silly_doubling != model::doubling_recipe(&pic.silly)
        || pic.silly_doubling != ra * silly_average + rb
        || pic.silly_shown != pic.silly_doubling.decimal1()?
        || pic.at_least != largest
        || pic.silly_doubling >= Q::int(pic.at_least as i128)
        || pic.silly_gap != model::gap_recipe(&pic.silly)
        || pic.silly_gap != Q::int(pic.estimate as i128)
    {
        return Err(format!(
            "the strange handful {:?} must keep the box at {largest} and make doubling say {}, under {}, \
             where the gap recipe says {}",
            pic.silly, pic.silly_shown, pic.at_least, pic.silly_gap
        ));
    }
    let below = model::share_by_largest_and_sum(n, pic.k, |m, s| model::doubling_of(s, k) < Q::int(m as i128));
    if pic.below != below
        || pic.below_one_in as i128 != (Q::ONE / below).nearest()
        || pic.below_percent != (Q::int(100) * below).about1()?
        || (k..=n).any(|m| model::gap_of(m, k) < Q::int(m as i128))
    {
        return Err(format!(
            "about 1 in {} handfuls ({}%) must get a doubling estimate below their own largest, and no gap estimate",
            pic.below_one_in, pic.below_percent
        ));
    }
    let ways = model::handfuls_by_sum(n, pic.k);
    let purple = facts::purple_bars(&ways);
    let sums: Vec<u64> = (0..ways.len() as u64).filter(|&s| ways[s as usize] > 0).collect();
    let tallest = purple.iter().map(|&(_, c)| c).max().unwrap_or(0);
    let tallest_at: Vec<u64> = purple.iter().filter(|&&(_, c)| c == tallest).map(|&(m, _)| m).collect();
    let half = Q::new(k as i128 + 1, 2 * k as i128);
    let green_top = model::with_largest(n, k).whole().unwrap_or(-1);
    let range = match (sums.first(), sums.last()) {
        (Some(&lo), Some(&hi)) => vec![model::doubling_of(lo, k), model::doubling_of(hi, k)],
        _ => Vec::new(),
    };
    if purple.iter().map(|&(_, c)| c).sum::<u128>() != choose(n, k)
        || pic.bars != purple.len()
        || u128::from(pic.tallest) != tallest
        || tallest_at.len() != 1
        || tallest_at.first().map(|&m| model::gap_of(m, k)) != Some(pic.tallest_at)
        || pic.range != range
        || purple.first().map(|&(m, _)| model::gap_of(m, k)) != pic.range.first().copied()
        || purple.last().map(|&(m, _)| model::gap_of(m, k)) != pic.range.last().copied()
        || pic
            .range
            .last()
            .is_none_or(|&hi| hi + half >= Q::int(VIEW_LAST as i128))
        || pic.stacks as u64 != n - k + 1
        || i128::from(pic.top_count) != green_top
        || u128::from(pic.top_count) <= tallest
    {
        return Err(format!(
            "the bars must hold every handful once: {} purple from {:?}, the tallest {} at {}, under {} green \
             topped by {}",
            pic.bars, pic.range, pic.tallest, pic.tallest_at, pic.stacks, pic.top_count
        ));
    }
    let off = |s: u64| model::doubling_of(s, k) - target;
    let miss = model::mean_by_sum(n, pic.k, |s| off(s).abs());
    let variance = model::mean_by_sum(n, pic.k, |s| off(s) * off(s));
    let gap_off = |m: u64| model::gap_of(m, k) - target;
    let gap_miss = model::mean_by_largest(n, k, |m| gap_off(m).abs());
    let gap_variance = model::mean_by_largest(n, k, |m| gap_off(m) * gap_off(m));
    if pic.miss != miss
        || pic.miss_shown != miss.about1()?
        || pic.variance != variance
        || variance != Q::new(((n + 1) * (n - k)) as i128, (3 * k) as i128)
        || pic.sd != variance.sqrt_about1()?
        || pic.gap_miss != gap_miss
        || pic.gap_miss_shown != gap_miss.about1()?
        || pic.gap_variance != gap_variance
        || pic.gap_sd != gap_variance.sqrt_about1()?
        || pic.ratio != variance / gap_variance
        || pic.ratio != Q::new(k as i128 + 2, 3)
        || miss <= gap_miss
        || variance <= gap_variance
        || miss * miss >= variance
        || gap_miss * gap_miss >= gap_variance
        || pic.miss_shown == pic.sd
        || pic.gap_miss_shown == pic.gap_sd
    {
        return Err(format!(
            "the widths must be the average misses {} and {} and the standard deviations {} and {}",
            pic.miss_shown, pic.gap_miss_shown, pic.sd, pic.gap_sd
        ));
    }
    Ok(pic)
}

// ------------------------------------------------------------------ words

/// The middle as the count gives it (10_1), from the rule count = a ×
/// middle + b with b negative.
fn middle_def(a: Q, b: Q) -> String {
    format!("middle = (count + {}) ÷ {a}", -b)
}

fn count_rule(a: Q, b: Q) -> String {
    format!("count = {a} × middle − {}", -b)
}

fn recipe_text(a: Q, b: Q) -> String {
    format!("{RECIPE} = {a} × average − {}", -b)
}

/// A handful's average, as the panel works it (10_3, 10_6).
fn average_text(hand: &[u64], average: &str) -> String {
    let terms: Vec<String> = hand.iter().map(u64::to_string).collect();
    format!("({}) ÷ {} = {average}", terms.join(" + "), hand.len())
}

/// The doubling recipe on an average, as the panel works it.
fn doubling_text(a: Q, b: Q, average: &str, estimate: &str) -> String {
    format!("{a} × {average} − {} = {estimate}", -b)
}

fn strange_text(others: &[u64]) -> String {
    others.iter().map(u64::to_string).collect::<Vec<_>>().join(", ")
}

fn percent_text(percent: &str) -> String {
    format!("{percent}%")
}

fn below_text(n: u64, percent: &str) -> String {
    format!("{n} boxes: {} of handfuls", percent_text(percent))
}

fn balance_text(average: Q) -> String {
    format!("balance point: {average}")
}

/// The table's values (10_9): the average miss, then the standard
/// deviation, each the gap recipe's and the doubling recipe's.
fn table_cells(pic: &Picture) -> [[String; 2]; 2] {
    [
        [pic.gap_miss_shown.clone(), pic.miss_shown.clone()],
        [pic.gap_sd.clone(), pic.sd.clone()],
    ]
}

/// Every word of the table, for its reading time.
fn table_text(pic: &Picture) -> String {
    let mut words: Vec<String> = HEADS.iter().map(|h| (*h).to_owned()).collect();
    for (label, row) in ROWS.iter().zip(table_cells(pic)) {
        words.push((*label).to_owned());
        words.extend(row);
    }
    words.join(" ")
}

// ------------------------------------------------------------- builders

/// The gap recipe's estimate `n` on a line shown to `end`: its green tick
/// across the line and its number over it. Detached.
fn estimate_tick(kit: &Kit, stage: &mut Stage<'_>, n: u64, end: f64) -> fmn::Result<[Mob; 2]> {
    let x = x_at(n as f64, end);
    let tick = stroke(
        stage,
        vec![v(x, STRIP_Y - TICK_DOWN), v(x, STRIP_Y + TICK_UP)],
        line_style(palette::estimate(), TICK_W, 1.0),
    );
    let label = kit.text(
        stage,
        &n.to_string(),
        PANEL_SIZE,
        palette::estimate(),
        v(x, STRIP_Y + ESTIMATE_DY),
    )?;
    Ok([tick, label])
}

/// A doubling value under a line shown to `end`: a short purple mark at
/// `value` and `shown` under it. Detached.
fn under_mark(kit: &Kit, stage: &mut Stage<'_>, value: Q, shown: &str, end: f64) -> fmn::Result<[Mob; 2]> {
    let x = x_at(approx(value).map_err(fail)?, end);
    let mark = stroke(
        stage,
        vec![v(x, MARK_FROM), v(x, MARK_TO)],
        line_style(palette::doubling(), TICK_W, 1.0),
    );
    let number = kit.text(stage, shown, PANEL_SIZE, palette::doubling(), v(x, MARK_NUMBER_Y))?;
    Ok([mark, number])
}

/// A fulcrum standing on its tip at (`x`, `tip`), in `color`. Detached.
fn fulcrum(stage: &mut Stage<'_>, x: f64, tip: f64, color: Srgb) -> Mob {
    stroke(
        stage,
        vec![
            v(x - FULCRUM[0], tip - FULCRUM[1]),
            v(x + FULCRUM[0], tip - FULCRUM[1]),
            v(x, tip),
            v(x - FULCRUM[0], tip - FULCRUM[1]),
        ],
        line_style(color, FULCRUM_W, 1.0),
    )
}

/// Chapter 08's green bars on a line shown to `end`: one per largest number
/// m from the handful size to the count, at its estimate, as tall as its
/// C(m − 1, k − 1) handfuls' share of the tallest. Detached.
fn green_bars(stage: &mut Stage<'_>, pic: &Picture, end: f64) -> fmn::Result<Vec<Mob>> {
    let k = pic.k as u64;
    let mut bars = Vec::new();
    for m in k..=pic.count {
        let many = approx(model::with_largest(m, k)).map_err(fail)?;
        let x = x_at(approx(model::gap_of(m, k)).map_err(fail)?, end);
        let h = BARS_H * many / pic.top_count as f64;
        bars.push(bar(stage, x, GREEN_W, h, palette::estimate(), BAR_O)?);
    }
    Ok(bars)
}

/// The purple outline (10_8): one short stroke per step, bound to the
/// tracker `rise`, the share of its full height. Detached.
fn outline(stage: &mut Stage<'_>, steps: Vec<Step>, top: u64, rise: Mob) -> fmn::Result<Vec<Mob>> {
    let pieces: Vec<Mob> = (0..steps.len())
        .map(|i| {
            stroke(
                stage,
                step_points(&steps, i, top, RISE_FROM),
                line_style(palette::doubling(), OUTLINE_W, 1.0),
            )
        })
        .collect();
    let placed = pieces.clone();
    bind_to(stage, &[rise], move |s, now| {
        for (i, &p) in placed.iter().enumerate() {
            reshape(s, p, step_points(&steps, i, top, now[0]));
        }
    })?;
    Ok(pieces)
}

/// The table on the panel (10_9): the heads on line 1 with the average
/// miss's row on line 2, then the standard deviation's row on line 3; each
/// row's label at the panel's edge and each recipe's value under its head.
/// Detached.
fn table(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture) -> fmn::Result<[Vec<Mob>; 2]> {
    let colors = [palette::estimate(), palette::doubling()];
    let mut rows: [Vec<Mob>; 2] = [Vec::new(), Vec::new()];
    for (c, h) in HEADS.iter().enumerate() {
        rows[0].push(kit.text(stage, h, PANEL_SIZE, colors[c], v(TABLE_X[c], PANEL_Y[1]))?);
    }
    for (r, row) in table_cells(pic).iter().enumerate() {
        rows[r].push(words_left(kit, stage, ROWS[r], &[], panel(r + 2))?);
        for (c, s) in row.iter().enumerate() {
            rows[r].push(kit.text(stage, s, PANEL_SIZE, colors[c], v(TABLE_X[c], PANEL_Y[r + 2]))?);
        }
    }
    Ok(rows)
}

impl SceneConstruct for ARivalRecipe {
    fn name(&self) -> &str {
        "10_a_rival_recipe"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let pic = picture().map_err(fail)?;
        let end = LINE_DOTS as f64;
        let (a, b) = linear(&pic.from_middle, "middle").map_err(fail)?;
        let (ra, rb) = linear(&pic.recipe, "average").map_err(fail)?;
        let top_box = pic.numbers.len() - 1;
        let dot = |n: u64| (n - pic.first) as usize;

        // the friends' line as chapter 08 opened it, their estimate's green
        // tick at seventy-one, and the badge
        let strip = FriendsLine::build(
            &kit,
            stage,
            pic.first,
            LINE_DOTS,
            &pic.numbers,
            &[],
            Start {
                end,
                fan: 0.0,
                rise: 1.0,
                widest: VIEW_END,
            },
        )?;
        for &piece in &strip.base {
            stage.add_to_scene(piece)?;
        }
        let corner = badge(&kit, stage, at(BADGE_AT))?;
        let ours = estimate_tick(&kit, stage, pic.estimate, end)?;
        let mut opened = strip.opening();
        opened.extend(ours);
        opened.push(corner);
        fade_in_all(stage, &opened, 0.8)?;

        // 10_1: the middle's definition; the span from one to the count with
        // its fulcrum at the middle; the rule back to the count
        let line = say(&kit, stage, "10_1")?;
        line.wait_for(stage, "they balance at the middle number")?;
        let middle_line = words_left(&kit, stage, &middle_def(a, b), &[], panel(0))?;
        fade_in_all(stage, &[middle_line], 0.5)?;
        line.wait_for(stage, "With seventy-one boxes")?;
        let span = stroke(
            stage,
            span_points(x_at(pic.first as f64, end), x_at(pic.count as f64, end), SPAN_Y),
            line_style(Palette::ink(), SPAN_W, 1.0),
        );
        let mx = x_at(approx(pic.middle).map_err(fail)?, end);
        let pivot = fulcrum(stage, mx, SPAN_Y - FULCRUM_GAP, Palette::ink());
        let middle_number = kit.text(
            stage,
            &pic.middle.to_string(),
            PANEL_SIZE,
            Palette::ink(),
            v(mx, MIDDLE_Y),
        )?;
        fade_in_all(stage, &[span, pivot, middle_number], 0.5)?;
        line.wait_for(stage, "twice the middle, minus one")?;
        let rule = words_left(&kit, stage, &count_rule(a, b), &[], panel(1))?;
        fade_in_all(stage, &[rule], 0.5)?;

        // 10_2: the span goes; the rule gives way to the doubling recipe, a
        // halo round its line
        let line = say(&kit, stage, "10_2")?;
        line.wait_for(stage, "let the handful's average stand in")?;
        fade_out_all(stage, &[span, pivot, middle_number], 0.4)?;
        line.wait_for(stage, "Twice the average, minus one")?;
        let recipe = words_left(
            &kit,
            stage,
            &recipe_text(ra, rb),
            &[(RECIPE.to_owned(), palette::doubling())],
            panel(1),
        )?;
        replace(stage, &[rule], &[recipe], 0.5)?;
        line.wait_for(stage, "Call it the doubling recipe")?;
        glow(stage, &[recipe], palette::doubling())?;

        // 10_3: the friends' average on the panel and under the line, then
        // their doubling estimate beside the green tick
        let line = say(&kit, stage, "10_3")?;
        line.wait_for(stage, "add up to")?;
        let average = pic.average.to_string();
        let averaged = words_left(&kit, stage, &average_text(&pic.numbers, &average), &[], panel(2))?;
        let average_mark = under_mark(&kit, stage, pic.average, &average, end)?;
        let mut shown = vec![averaged];
        shown.extend(average_mark);
        fade_in_all(stage, &shown, 0.5)?;
        line.wait_for(stage, "So doubling says seventy-three")?;
        let estimate = pic.doubling.to_string();
        let doubled = words_left(
            &kit,
            stage,
            &doubling_text(ra, rb, &average, &estimate),
            &[(estimate.clone(), palette::doubling())],
            panel(3),
        )?;
        let doubled_mark = under_mark(&kit, stage, pic.doubling, &estimate, end)?;
        let mut shown = vec![doubled];
        shown.extend(doubled_mark);
        fade_in_all(stage, &shown, 0.5)?;

        // 10_4: halos round the badge, the middle's line and the recipe
        let line = say(&kit, stage, "10_4")?;
        line.wait_for(stage, "In a fair handful")?;
        glow(stage, &[corner], Palette::ink())?;
        line.wait_for(stage, "the handful's average is the middle number")?;
        glow(stage, &[middle_line], Palette::ink())?;
        line.wait_for(stage, "That makes the doubling recipe average the count")?;
        glow(stage, &[recipe], palette::doubling())?;

        // 10_5: the friends' working goes; a halo round the box at sixty;
        // the other four boxes go and their dots turn plain; the strange
        // four land as rings; the countdown, nothing of the new average
        let line = say(&kit, stage, "10_5")?;
        line.wait_for(stage, "Try a strange handful")?;
        let mut gone = vec![averaged, doubled];
        gone.extend(average_mark);
        gone.extend(doubled_mark);
        fade_out_all(stage, &gone, 0.4)?;
        line.wait_for(stage, "Keep the box at sixty")?;
        let kept = [strip.bodies[top_box], strip.labels[top_box]];
        glow(stage, &kept, palette::seen())?;
        line.wait_for(stage, "make the other four")?;
        let mut four = strip.bodies[..top_box].to_vec();
        four.extend(&strip.labels[..top_box]);
        four.extend(&strip.ticks[..top_box]);
        fade_out_all(stage, &four, 0.5)?;
        let plain: Vec<(Mob, Srgb, f64)> = pic.numbers[..top_box]
            .iter()
            .map(|&n| (strip.dots[dot(n)], palette::line(), dot_opacity(x_at(n as f64, end))))
            .collect();
        refill(stage, &plain, 0.4)?;
        line.wait_for(stage, "two, three, four and five")?;
        let others = &pic.silly[..pic.others];
        let rings: Vec<Mob> = others
            .iter()
            .map(|&n| ring(stage, v(x_at(n as f64, end), STRIP_Y)))
            .collect();
        let (lo, hi) = (x_at(others[0] as f64, end), x_at(others[pic.others - 1] as f64, end));
        let strange = kit.text(
            stage,
            &strange_text(others),
            PANEL_SIZE,
            Palette::ink(),
            v((lo + hi) / 2.0, UNDER_Y),
        )?;
        let mut landed = rings.clone();
        landed.push(strange);
        fade_in_all(stage, &landed, 0.5)?;
        let q = think(&kit, stage, at(THINK_AT))?;

        // 10_6: the strange handful's working and its doubling estimate under
        // the line; a teal halo round the box at sixty; a green halo round
        // the gap recipe's seventy-one
        let line = say(&kit, stage, "10_6")?;
        fade_out_all(stage, &[q], 0.3)?;
        line.wait_for(stage, "Twenty-eight point six")?;
        let silly_averaged = words_left(
            &kit,
            stage,
            &average_text(&pic.silly, &pic.silly_average),
            &[],
            panel(2),
        )?;
        let silly_doubled = words_left(
            &kit,
            stage,
            &doubling_text(ra, rb, &pic.silly_average, &pic.silly_shown),
            &[(pic.silly_shown.clone(), palette::doubling())],
            panel(3),
        )?;
        let silly_mark = under_mark(&kit, stage, pic.silly_doubling, &pic.silly_shown, end)?;
        let mut shown = vec![silly_averaged, silly_doubled];
        shown.extend(silly_mark);
        fade_in_all(stage, &shown, 0.5)?;
        line.wait_for(stage, "fewer than the sixty boxes")?;
        glow(stage, &kept, palette::certain())?;
        line.wait_for(stage, "The gap recipe says seventy-one")?;
        glow(stage, &ours, palette::estimate())?;

        // 10_7: the share below the largest replaces the working; a green
        // halo round the gap recipe's tick
        let line = say(&kit, stage, "10_7")?;
        line.wait_for(stage, "about one handful in five")?;
        let below = [
            words_left(
                &kit,
                stage,
                &below_text(pic.count, &pic.below_percent),
                &[(percent_text(&pic.below_percent), palette::doubling())],
                panel(2),
            )?,
            words_left(
                &kit,
                stage,
                BELOW_WORDS,
                &[(DOUBLING.to_owned(), palette::doubling())],
                panel(3),
            )?,
        ];
        replace(stage, &[silly_averaged, silly_doubled], &below, 0.5)?;
        line.wait_for(stage, "The gap recipe never does")?;
        glow(stage, &ours, palette::estimate())?;

        // 10_8: everything on the line goes and its dots even out as its end
        // slides out; the company's count; chapter 08's green bars, then the
        // purple outline over them and its two ends
        let line = say(&kit, stage, "10_8")?;
        line.wait_for(stage, "put both recipes side by side")?;
        let mut gone = rings;
        gone.push(strange);
        gone.extend(silly_mark);
        gone.extend([strip.bodies[top_box], strip.labels[top_box], strip.ticks[top_box]]);
        gone.extend(ours);
        gone.extend([middle_line, recipe]);
        gone.extend(below);
        fade_out_all(stage, &gone, 0.5)?;
        let even: Vec<(Mob, Srgb, f64)> = (pic.first..=LINE_DOTS)
            .zip(strip.dots.iter().copied())
            .map(|(n, d)| (d, palette::line(), dot_opacity(x_at(n as f64, VIEW_END))))
            .collect();
        refill(stage, &even, EVEN_S)?;
        stage.play(tween(strip.t.end, VIEW_END, ZOOM_S)?)?;
        line.wait_for(stage, "exactly seventy-one boxes")?;
        let count = words_left(&kit, stage, &count_text(pic.count), &[], panel(0))?;
        let mark = count_mark(stage, pic.count, VIEW_END, COUNT_TOP);
        fade_in_all(stage, &[count, mark], 0.5)?;
        line.wait_for(stage, "The gap recipe's sixty-seven bars come back")?;
        let green = green_bars(stage, &pic, VIEW_END)?;
        grow_bars(stage, &green, BARS_S, BARS_LAG)?;
        line.wait_for(stage, "counted by total")?;
        let rise = stage.add_value_tracker(RISE_FROM);
        let steps = steps_of(pic.count, pic.k, VIEW_END).map_err(fail)?;
        let purple = outline(stage, steps, pic.top_count, rise)?;
        for &piece in &purple {
            stage.add_to_scene(piece)?;
        }
        stage.play(tween(rise, 1.0, RISE_S)?)?;
        line.wait_for(stage, "from five all the way to")?;
        let mut ends = Vec::new();
        for &e in &pic.range {
            let x = x_at(approx(e).map_err(fail)?, VIEW_END);
            ends.push(kit.text(stage, &e.to_string(), PANEL_SIZE, palette::doubling(), v(x, UNDER_Y))?);
        }
        fade_in_all(stage, &ends, 0.4)?;

        // 10_9: the fulcrum at the count; halos round both sets of bars,
        // together; the table of both widths, held to be read
        let line = say(&kit, stage, "10_9")?;
        line.wait_for(stage, "Both balance at seventy-one")?;
        let fx = x_at(approx(pic.balance).map_err(fail)?, VIEW_END);
        let balanced = fulcrum(stage, fx, STRIP_Y - FULCRUM_GAP, Palette::ink());
        let balance = kit.text(
            stage,
            &balance_text(pic.balance),
            PANEL_SIZE,
            Palette::ink(),
            v(fx, BALANCE_Y),
        )?;
        right_at(stage, balance, fx - BALANCE_GAP, BALANCE_Y);
        fade_in_all(stage, &[balanced, balance], 0.5)?;
        line.wait_for(stage, "lower and wider")?;
        glow_each(
            stage,
            &[
                (purple.clone(), palette::doubling()),
                (green.clone(), palette::estimate()),
            ],
            0.3,
            0.6,
        )?;
        let [first_rows, second_row] = table(&kit, stage, &pic)?;
        line.wait_for(stage, "Ignoring direction")?;
        fade_in_all(stage, &first_rows, 0.5)?;
        line.wait_for(stage, "The standard deviation")?;
        fade_in_all(stage, &second_row, 0.5)?;
        let table_up = now(stage);
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;

        // 10_10: a green halo round the bars, a purple one round the
        // outline; the table stays until it can be read
        let line = say(&kit, stage, "10_10")?;
        line.wait_for(stage, "uses only the largest number")?;
        glow(stage, &green, palette::estimate())?;
        line.wait_for(stage, "the recipe that uses all five")?;
        glow(stage, &purple, palette::doubling())?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;
        wait_until(stage, table_up + read_s(&table_text(&pic)))?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use explainer_kit::Script;
    use fmn::core::constants::FRAME_X_RADIUS;

    use super::*;
    use crate::chapters::draw::{BOX_W, PAUSE_NOTE, SMALL, THINK_R, box_spot};
    use crate::chapters::friends_line::{BADGE, BADGE_PAD, BADGE_SIZE, PANEL_X, RING_R, est_width};
    use crate::chapters::one_estimate::SPAN_TICK;
    use crate::facts::{SILLY, SUMS_PER_BAR};
    use crate::model::{FRIENDS, KNOWN, all_handfuls, doubling_recipe, gap_recipe};

    /// The countdown's note, at the size `think` writes it.
    const NOTE_SIZE: f64 = 18.0;
    /// A line's half height at the panel's size, and the space kept between
    /// things that must not touch.
    const HALF_LINE: f64 = 0.16;
    const CLEAR: f64 = 0.1;
    const NEAR: f64 = 1e-9;

    fn good() -> Picture {
        picture().unwrap()
    }

    /// The scene draws the facts, and its lines read as the voice says
    /// them: the friends' five, the company and the strange handful are the
    /// model's, and every line the panel, the table and the words under the
    /// line show.
    #[test]
    fn the_picture_is_the_facts() {
        let pic = good();
        assert_eq!(pic.numbers, FRIENDS);
        assert_eq!(pic.count, KNOWN);
        assert_eq!(pic.silly, SILLY);
        assert_eq!(pic.middle.to_string(), "36");
        let (a, b) = linear(&pic.from_middle, "middle").unwrap();
        assert_eq!((a, b), (Q::int(2), Q::int(-1)));
        assert_eq!(middle_def(a, b), "middle = (count + 1) ÷ 2");
        assert_eq!(count_rule(a, b), "count = 2 × middle − 1");
        assert_eq!(recipe_text(a, b), "doubling recipe = 2 × average − 1");
        let average = pic.average.to_string();
        assert_eq!(
            average_text(&pic.numbers, &average),
            "(14 + 27 + 33 + 51 + 60) ÷ 5 = 37"
        );
        assert_eq!(
            doubling_text(a, b, &average, &pic.doubling.to_string()),
            "2 × 37 − 1 = 73"
        );
        assert_eq!(strange_text(&pic.silly[..pic.others]), "2, 3, 4, 5");
        assert_eq!(
            average_text(&pic.silly, &pic.silly_average),
            "(2 + 3 + 4 + 5 + 60) ÷ 5 = 14.8"
        );
        assert_eq!(
            doubling_text(a, b, &pic.silly_average, &pic.silly_shown),
            "2 × 14.8 − 1 = 28.6"
        );
        assert_eq!(below_text(pic.count, &pic.below_percent), "71 boxes: 20.9% of handfuls");
        assert_eq!(count_text(pic.count), "count = 71");
        assert_eq!(balance_text(pic.balance), "balance point: 71");
        assert_eq!(pic.range, [Q::int(5), Q::int(137)]);
        assert_eq!((pic.bars, pic.tallest, pic.tallest_at), (111, 339_265, Q::int(71)));
        let cells = [["9.2", "14.3"], ["11.7", "17.8"]];
        assert_eq!(table_cells(&pic), cells.map(|r| r.map(str::to_owned)));
        assert_eq!(
            table_text(&pic),
            "gap doubling average miss 9.2 14.3 standard deviation 11.7 17.8"
        );
    }

    /// Rules are read as the facts print them, and refused when they are
    /// not a multiple plus a number; the middle of 1 to n is its average.
    #[test]
    fn rules_are_read_as_the_facts_print_them() {
        assert_eq!(linear("2*middle-1", "middle").unwrap(), (Q::int(2), Q::int(-1)));
        assert_eq!(linear("3/2*average+4", "average").unwrap(), (Q::new(3, 2), Q::int(4)));
        for bad in ["2*middle", "2middle-1", "middle-1", "2*average-1", "x*middle-1", ""] {
            assert!(linear(bad, "middle").is_err(), "{bad:?}");
        }
        assert_eq!(middle_of(KNOWN), Q::int(36));
        assert_eq!(middle_of(70), Q::new(71, 2));
        assert_eq!(middle_of(1), Q::ONE);
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
        refused(
            Picture {
                count: 72,
                ..pic.clone()
            },
            "friends' estimate",
        );
        // the friends' median is not the company's middle
        refused(
            Picture {
                middle: Q::int(33),
                ..pic.clone()
            },
            "give every count back",
        );
        refused(
            Picture {
                from_middle: "2*middle+1".to_owned(),
                ..pic.clone()
            },
            "give every count back",
        );
        refused(
            Picture {
                from_middle: "middle-1".to_owned(),
                ..pic.clone()
            },
            "not a multiple of middle",
        );
        refused(
            Picture {
                recipe: "2*average+1".to_owned(),
                ..pic.clone()
            },
            "the doubling recipe",
        );
        refused(
            Picture {
                sum: pic.sum + 1,
                ..pic.clone()
            },
            "the doubling recipe",
        );
        refused(
            Picture {
                doubling: Q::int(71),
                ..pic.clone()
            },
            "the doubling recipe",
        );
        refused(
            Picture {
                balance: Q::int(72),
                ..pic.clone()
            },
            "both recipes must balance",
        );
        refused(
            Picture {
                silly: vec![2, 3, 4, 5, 59],
                ..pic.clone()
            },
            "the strange handful",
        );
        refused(
            Picture {
                silly_shown: "28.0".to_owned(),
                ..pic.clone()
            },
            "the strange handful",
        );
        refused(
            Picture {
                silly_gap: Q::int(70),
                ..pic.clone()
            },
            "the strange handful",
        );
        refused(
            Picture {
                below: Q::new(1, 5),
                ..pic.clone()
            },
            "below their own largest",
        );
        refused(
            Picture {
                below_one_in: 4,
                ..pic.clone()
            },
            "below their own largest",
        );
        refused(
            Picture {
                below_percent: "21.0".to_owned(),
                ..pic.clone()
            },
            "below their own largest",
        );
        refused(
            Picture {
                bars: pic.bars - 1,
                ..pic.clone()
            },
            "every handful once",
        );
        refused(
            Picture {
                tallest: pic.tallest - 1,
                ..pic.clone()
            },
            "every handful once",
        );
        refused(
            Picture {
                tallest_at: Q::int(72),
                ..pic.clone()
            },
            "every handful once",
        );
        refused(
            Picture {
                range: vec![Q::int(5), Q::int(138)],
                ..pic.clone()
            },
            "every handful once",
        );
        refused(
            Picture {
                top_count: 900_000,
                ..pic.clone()
            },
            "every handful once",
        );
        // "misses by about fourteen on average": the average miss is not the
        // standard deviation, and neither recipe's widths are the other's
        refused(
            Picture {
                sd: pic.miss_shown.clone(),
                ..pic.clone()
            },
            "the widths",
        );
        refused(
            Picture {
                miss_shown: "14".to_owned(),
                ..pic.clone()
            },
            "the widths",
        );
        refused(
            Picture {
                miss: pic.gap_miss,
                ..pic.clone()
            },
            "the widths",
        );
        refused(
            Picture {
                variance: pic.gap_variance,
                ..pic.clone()
            },
            "the widths",
        );
        refused(
            Picture {
                gap_sd: "11.6".to_owned(),
                ..pic.clone()
            },
            "the widths",
        );
        refused(
            Picture {
                ratio: Q::int(2),
                ..pic
            },
            "the widths",
        );
    }

    /// The purple bars need no list of handfuls: for every small company,
    /// handfuls of five from up to twelve boxes, counting by total gives
    /// the same bars, each estimate within 2/5 of its bar's centre, the same
    /// average, average miss and average squared miss as listing every
    /// handful, (N + 1)(N − 5) ÷ 15 the last, and the same share below the
    /// largest number when counting by largest and total together.
    #[test]
    fn counting_by_total_is_listing_every_handful() {
        let k = 5usize;
        for n in 5..=12u64 {
            let hands = all_handfuls(n, k);
            let count = Q::int(hands.len() as i128);
            let target = Q::int(n as i128);
            let mut by_bar: BTreeMap<u64, u128> = BTreeMap::new();
            for h in &hands {
                let s: u64 = h.iter().sum();
                let m = Q::new(s as i128, SUMS_PER_BAR as i128).nearest() as u64;
                *by_bar.entry(m).or_insert(0) += 1;
                assert!(
                    (doubling_recipe(h) - model::gap_of(m, 5)).abs() <= Q::new(2, 5),
                    "n {n}, {h:?}"
                );
            }
            let listed: Vec<(u64, u128)> = by_bar.into_iter().collect();
            assert_eq!(facts::purple_bars(&model::handfuls_by_sum(n, k)), listed, "n {n}");
            let mean = hands.iter().fold(Q::ZERO, |s, h| s + doubling_recipe(h)) / count;
            let miss = hands
                .iter()
                .fold(Q::ZERO, |s, h| s + (doubling_recipe(h) - target).abs())
                / count;
            let var = hands.iter().fold(Q::ZERO, |s, h| {
                let d = doubling_recipe(h) - target;
                s + d * d
            }) / count;
            assert_eq!(mean, target, "n {n}");
            assert_eq!(
                model::mean_by_sum(n, k, |s| (model::doubling_of(s, 5) - target).abs()),
                miss,
                "n {n}"
            );
            assert_eq!(var, Q::new(((n + 1) * (n - 5)) as i128, 15), "n {n}");
            let below = hands
                .iter()
                .filter(|h| doubling_recipe(h) < Q::int(h[k - 1] as i128))
                .count();
            assert_eq!(
                model::share_by_largest_and_sum(n, k, |m, s| model::doubling_of(s, 5) < Q::int(m as i128)),
                Q::new(below as i128, hands.len() as i128),
                "n {n}"
            );
        }
    }

    /// The doubling recipe at the edges of what 10 says: its average squared
    /// miss is (N + 1)(N − k) ÷ (3k), (k + 2)/3 of the gap recipe's at every
    /// count past the handful size (the same only for one box), both zero
    /// at N = k where every handful is 1 to N; it never falls below the
    /// largest for one box or a pair, and does for some handful of three or
    /// more from every count past the size. A purple bar's totals move the
    /// doubling estimate as far as one more largest number moves the gap
    /// recipe's.
    #[test]
    fn the_doubling_recipe_at_its_edges() {
        for k in 1..=4u64 {
            for n in k..=11 {
                let hands = all_handfuls(n, k as usize);
                let count = Q::int(hands.len() as i128);
                let target = Q::int(n as i128);
                let squared = |r: &dyn Fn(&[u64]) -> Q| {
                    hands.iter().fold(Q::ZERO, |s, h| {
                        let d = r(h.as_slice()) - target;
                        s + d * d
                    }) / count
                };
                let dbl = squared(&doubling_recipe);
                let gap = squared(&gap_recipe);
                assert_eq!(
                    dbl,
                    Q::new(((n + 1) * (n - k)) as i128, (3 * k) as i128),
                    "k {k}, n {n}"
                );
                if n == k {
                    assert_eq!((dbl, gap), (Q::ZERO, Q::ZERO), "k {k}");
                } else {
                    assert_eq!(dbl / gap, Q::new(k as i128 + 2, 3), "k {k}, n {n}");
                }
                let below = hands
                    .iter()
                    .any(|h| doubling_recipe(h) < Q::int(h[h.len() - 1] as i128));
                assert_eq!(below, k >= 3 && n > k, "k {k}, n {n}");
            }
        }
        assert_eq!(
            Q::new(2, 5) * Q::int(SUMS_PER_BAR as i128),
            model::gap_of(61, 5) - model::gap_of(60, 5)
        );
    }

    /// The outline's pieces join up: the first rises from the line, equal
    /// neighbours share a top with no edge between them, the last comes
    /// down to the line, each top its share of the tallest green bar; at
    /// the start of the rise no piece of the real outline has an edge of
    /// zero length.
    #[test]
    fn the_outline_steps_join_up() {
        let steps = [
            Step {
                left: 0.0,
                right: 1.0,
                many: 2,
            },
            Step {
                left: 1.0,
                right: 2.0,
                many: 2,
            },
            Step {
                left: 2.0,
                right: 3.0,
                many: 4,
            },
        ];
        let top = 4;
        let near = |p: Vec3, x: f64, y: f64| (p[0] - x).abs() < NEAR && (p[1] - y).abs() < NEAR;
        let y = |many: f64| STRIP_Y + BARS_H * many / top as f64;
        let p = step_points(&steps, 0, top, 1.0);
        assert!(p.len() == 3 && near(p[0], 0.0, STRIP_Y) && near(p[1], 0.0, y(2.0)) && near(p[2], 1.0, y(2.0)));
        let p = step_points(&steps, 1, top, 1.0);
        assert!(p.len() == 2 && near(p[0], 1.0, y(2.0)) && near(p[1], 2.0, y(2.0)));
        let p = step_points(&steps, 2, top, 1.0);
        assert!(p.len() == 4 && near(p[0], 2.0, y(2.0)) && near(p[1], 2.0, y(4.0)));
        assert!(near(p[2], 3.0, y(4.0)) && near(p[3], 3.0, STRIP_Y));
        let pic = good();
        let real = steps_of(pic.count, pic.k, VIEW_END).unwrap();
        assert_eq!(real.len(), pic.bars);
        for i in 0..real.len() {
            for w in step_points(&real, i, pic.top_count, RISE_FROM).windows(2) {
                assert!((w[0][0] - w[1][0]).abs() + (w[0][1] - w[1][1]).abs() > 0.0, "step {i}");
            }
        }
    }

    /// Every phrase the scene waits for is in its line of script.tsv, once,
    /// and a line's phrases come in the order the scene waits for them; the
    /// prediction's line shows nothing of the answer.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let waits: &[(&str, &[&str])] = &[
            (
                "10_1",
                &[
                    "they balance at the middle number",
                    "With seventy-one boxes",
                    "twice the middle, minus one",
                ],
            ),
            (
                "10_2",
                &[
                    "let the handful's average stand in",
                    "Twice the average, minus one",
                    "Call it the doubling recipe",
                ],
            ),
            ("10_3", &["add up to", "So doubling says seventy-three"]),
            (
                "10_4",
                &[
                    "In a fair handful",
                    "the handful's average is the middle number",
                    "That makes the doubling recipe average the count",
                ],
            ),
            (
                "10_5",
                &[
                    "Try a strange handful",
                    "Keep the box at sixty",
                    "make the other four",
                    "two, three, four and five",
                ],
            ),
            (
                "10_6",
                &[
                    "Twenty-eight point six",
                    "fewer than the sixty boxes",
                    "The gap recipe says seventy-one",
                ],
            ),
            ("10_7", &["about one handful in five", "The gap recipe never does"]),
            (
                "10_8",
                &[
                    "put both recipes side by side",
                    "exactly seventy-one boxes",
                    "The gap recipe's sixty-seven bars come back",
                    "counted by total",
                    "from five all the way to",
                ],
            ),
            (
                "10_9",
                &[
                    "Both balance at seventy-one",
                    "lower and wider",
                    "Ignoring direction",
                    "The standard deviation",
                ],
            ),
            (
                "10_10",
                &["uses only the largest number", "the recipe that uses all five"],
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
        let asked = script.text("10_5").unwrap();
        for answer in ["twenty-eight", "fourteen", "point"] {
            assert!(!asked.contains(answer), "{answer}");
        }
    }

    /// The panel, the marks under the line, the countdown and the bars keep
    /// apart: every panel line in the title-safe area, the badge left of
    /// it, the table's label, columns and frame edge apart; the purple
    /// marks under the green tick's reach, the friends' two apart, the
    /// span and the middle's number under them, the strange words in the
    /// frame and clear of its estimate; the countdown right of the box at
    /// sixty, under the panel and over the green number; the bars and the
    /// outline in the frame and under the panel, the outline's steps as wide
    /// as a green bar's spacing and touching, the count's mark over the bars
    /// around it, the range's ends and the balance's words apart.
    #[test]
    fn the_panel_the_marks_and_the_bars_fit_the_frame() {
        let pic = good();
        let safe_x = FRAME_X_RADIUS - 0.25;
        let end = LINE_DOTS as f64;
        let k = pic.k as u64;
        let w = |s: &str| est_width(s, PANEL_SIZE);
        let (a, b) = linear(&pic.from_middle, "middle").unwrap();
        let average = pic.average.to_string();
        let estimate = pic.doubling.to_string();
        for text in [
            middle_def(a, b),
            count_rule(a, b),
            recipe_text(a, b),
            average_text(&pic.numbers, &average),
            doubling_text(a, b, &average, &estimate),
            average_text(&pic.silly, &pic.silly_average),
            doubling_text(a, b, &pic.silly_average, &pic.silly_shown),
            below_text(pic.count, &pic.below_percent),
            BELOW_WORDS.to_owned(),
            count_text(pic.count),
        ] {
            assert!(PANEL_X + w(&text) < safe_x, "{text}");
        }
        let badge_right = BADGE_AT[0] + est_width(BADGE, BADGE_SIZE) / 2.0 + BADGE_PAD[0];
        assert!(badge_right < PANEL_X - CLEAR);
        // the table
        let label_right = ROWS.iter().map(|r| PANEL_X + w(r)).fold(0.0, f64::max);
        let cells = table_cells(&pic);
        let widths: Vec<f64> = (0..2)
            .map(|c| cells.iter().map(|r| w(&r[c])).fold(w(HEADS[c]), f64::max))
            .collect();
        assert!(label_right + CLEAR < TABLE_X[0] - widths[0] / 2.0);
        assert!(TABLE_X[0] + widths[0] / 2.0 + CLEAR < TABLE_X[1] - widths[1] / 2.0);
        assert!(TABLE_X[1] + widths[1] / 2.0 < safe_x);
        // under the line, at the opening scale
        const { assert!(MARK_FROM < STRIP_Y - TICK_DOWN) };
        const { assert!(MARK_NUMBER_Y + HALF_LINE < MARK_TO) };
        const { assert!(SPAN_Y + SPAN_TICK < STRIP_Y - TICK_DOWN - CLEAR) };
        const { assert!(MIDDLE_Y + HALF_LINE < SPAN_Y - FULCRUM_GAP - FULCRUM[1]) };
        const { assert!(UNDER_Y + HALF_LINE < STRIP_Y - RING_R - CLEAR / 2.0) };
        let at = |q: Q| x_at(approx(q).unwrap(), end);
        assert!(at(pic.average) + w(&average) / 2.0 + CLEAR < at(pic.doubling) - w(&estimate) / 2.0);
        assert!(at(pic.doubling) + w(&estimate) / 2.0 < safe_x);
        let others = &pic.silly[..pic.others];
        let (lo, hi) = (x_at(others[0] as f64, end), x_at(others[pic.others - 1] as f64, end));
        let strange_w = w(&strange_text(others));
        assert!((lo + hi) / 2.0 - strange_w / 2.0 > -safe_x);
        assert!((lo + hi) / 2.0 + strange_w / 2.0 + CLEAR < at(pic.silly_doubling) - w(&pic.silly_shown) / 2.0);
        assert!(x_at(2.0, end) - x_at(1.0, end) > 2.0 * RING_R);
        // the countdown
        let note_y = THINK_AT[1] - THINK_R - 0.3;
        let note_half = est_width(PAUSE_NOTE, NOTE_SIZE) / 2.0;
        let top_box = pic.numbers.len() - 1;
        let box_right = box_spot(top_box, pic.numbers[top_box], end, 0.0)[0] + BOX_W * SMALL / 2.0;
        assert!(box_right + CLEAR < THINK_AT[0] - note_half.max(THINK_R));
        const { assert!(THINK_AT[1] + THINK_R < PANEL_Y[1] - HALF_LINE - CLEAR) };
        assert!(note_y - 0.1 > STRIP_Y + ESTIMATE_DY + HALF_LINE + CLEAR);
        // the bars, at the zoomed scale
        const { assert!(STRIP_Y + BARS_H < PANEL_Y[3] - HALF_LINE - CLEAR) };
        let green: Vec<f64> = (k..=pic.count)
            .map(|m| x_at(approx(model::gap_of(m, k)).unwrap(), VIEW_END))
            .collect();
        assert!(green[0] - GREEN_W / 2.0 > -safe_x);
        for p in green.windows(2) {
            assert!(p[1] - p[0] > GREEN_W + 0.02);
        }
        let steps = steps_of(pic.count, pic.k, VIEW_END).unwrap();
        assert!(steps[0].left > -safe_x && steps[steps.len() - 1].right < safe_x);
        let spacing = green[1] - green[0];
        assert!(steps.iter().all(|s| (s.right - s.left - spacing).abs() < NEAR));
        assert!(steps.windows(2).all(|p| (p[0].right - p[1].left).abs() < NEAR));
        // the count's mark over the bars at and beside it
        let height = |many: u128| BARS_H * many as f64 / pic.top_count as f64;
        let mark_x = x_at(pic.count as f64, VIEW_END);
        for m in k..=pic.count {
            let x = x_at(approx(model::gap_of(m, k)).unwrap(), VIEW_END);
            if (x - mark_x).abs() < spacing * 1.5 {
                let many = model::with_largest(m, k).whole().unwrap() as u128;
                assert!(STRIP_Y + height(many) + CLEAR < COUNT_TOP, "m {m}");
            }
        }
        for s in &steps {
            if (s.left - mark_x) * (s.right - mark_x) <= 0.0 {
                assert!(STRIP_Y + height(s.many) + CLEAR < COUNT_TOP);
            }
        }
        // the range's ends and the balance's words
        let ends: Vec<f64> = pic.range.iter().map(|&e| x_at(approx(e).unwrap(), VIEW_END)).collect();
        let (lo_w, hi_w) = (w(&pic.range[0].to_string()), w(&pic.range[1].to_string()));
        assert!(ends[0] - lo_w / 2.0 > -safe_x && ends[1] + hi_w / 2.0 < safe_x);
        let fx = x_at(approx(pic.balance).unwrap(), VIEW_END);
        let balance_w = w(&balance_text(pic.balance));
        assert!(fx - BALANCE_GAP - balance_w > ends[0] + lo_w / 2.0 + CLEAR);
        assert!(fx - BALANCE_GAP < fx - FULCRUM[0] - CLEAR);
        assert!(fx + FULCRUM[0] + CLEAR < ends[1] - hi_w / 2.0);
    }
}
