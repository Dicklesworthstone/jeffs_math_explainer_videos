//! Chapter 08, "One estimate is not the average": on the friends' line,
//! their estimate a green tick at seventy-one, the promise is the recipe's,
//! so imagine a company that made exactly seventy-one boxes (08_1). Twenty
//! handfuls drawn by computer land one after another, each estimate a green
//! dot dropping onto a stack over the line (08_2): from about forty-five to
//! eighty-four point two, three exactly seventy-one, averaging about
//! seventy-two (08_3). Every handful counted by its largest number: one
//! tops out at five, five at six, over nine hundred thousand at
//! seventy-one; sixty-seven stacks, each a bar at its estimate, the one at
//! exactly seventy-one kept back under a "?", held to be inspected (08_4).
//! The prediction, how often an estimate is exactly right (08_5); about one
//! handful in twenty-nine (08_6). The bars balance at seventy-one and are
//! lopsided, held to be inspected (08_7); the average miss and the standard
//! deviation, held to be read (08_8); three ideas kept apart, with card E
//! (08_9).
//!
//! The bars are built from how many handfuls have each largest number,
//! never by listing handfuls, and the scene checks every value it shows
//! against the facts. Highlights are halos and colour changes are
//! replacements: no typeset text is indicated, recoloured or scaled in
//! place. Chapter 09 draws its bars, its count mark and its ranges with the
//! builders here.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, Timed, choose, fade_all, rect, thousands, v};
use fmn::prelude::*;

use super::draw::{
    DOT_R, STRIP_Y, TICK_W, dot_opacity, fade_in_all, fade_in_lagged, fade_out_all, fail, flat, line_style, pause, say,
    stroke, think, wait_until, x_at,
};
use super::friends_line::{
    BADGE_AT, DROP, FriendsLine, PANEL_SIZE, Start, at, badge, handfuls, is_handful, list, now, panel, printed, read_s,
    ring, value, whole,
};
use super::gaps_trade_places::glow_each;
use super::what_we_assume::{formula, formula_colors, glow, replace, words_left};
use crate::facts;
use crate::model::{self, LINE_DOTS, Q};
use crate::palette;

pub struct OneEstimate {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const QUESTION: &str = "?";
const HEADS: [&str; 3] = ["largest", "handfuls", "estimate"];
const SD_DEF: &str = "standard deviation = √(average squared miss)";

/// The friends' estimate's tick across the line and its number over it
/// (as chapters 04 and 07 draw them).
const TICK_DOWN: f64 = 0.12;
const TICK_UP: f64 = 0.3;
const ESTIMATE_DY: f64 = 0.55;
/// The company's count: an upright ink mark from just under the line to
/// just over the drawn rows (09 reaches it over its ranges).
const MARK_W: f64 = 2.0;
const MARK_O: f64 = 0.8;
const MARK_DOWN: f64 = 0.1;
/// The drawn rows (08_2) land at one height; each estimate drops onto a
/// stack of green dots over its place on the line, three high at most.
const DRAW_Y: f64 = 0.55;
const MARK_TOP: f64 = DRAW_Y + 0.15;
const STACK_Y0: f64 = STRIP_Y + 0.16;
const STACK_STEP: f64 = 0.14;
const STACK_MAX: usize = 3;
const EST_R: f64 = 0.055;
/// The lowest and highest draws' values under the line, and the draws'
/// average: a short green mark under the line with its words under it.
const UNDER_Y: f64 = STRIP_Y - 0.35;
const AVERAGE_MARK: [f64; 2] = [0.1, 0.55];
const AVERAGE_Y: f64 = STRIP_Y - 0.8;
/// The bars (08_4): green, standing on the line at their estimates, the
/// tallest `BAR_H` high; the "?" over the empty place at the count, clear
/// of its neighbours' tops.
const BAR_W: f64 = 0.13;
const BAR_H: f64 = 2.2;
pub const BAR_O: f64 = 0.85;
const HIDDEN_DY: f64 = 0.35;
/// The table at the upper left (08_4): its three columns' centres, its
/// heads' height and its rows', and the two lines at its foot (08_6, 08_7).
const TABLE_X: [f64; 3] = [-6.0, -4.6, -3.1];
const TABLE_Y: [f64; 4] = [2.85, 2.5, 2.15, 1.8];
const TABLE_SIZE: f64 = 20.0;
const FOOT_X: f64 = -6.3;
const FOOT_Y: [f64; 2] = [1.4, 1.05];
/// The countdown, at the left under the table, where the bars are lowest.
const THINK_08: [f64; 2] = [-4.5, 0.6];
/// Under the line (08_7, 08_8): the fulcrum at the count with its words to
/// its left; the two sides' brackets and their words; then the two widths,
/// each a span about the count with its words under it.
const FULCRUM: [f64; 2] = [0.14, 0.3];
const FULCRUM_GAP: f64 = 0.04;
const BALANCE_Y: f64 = STRIP_Y - 0.2;
const BALANCE_GAP: f64 = 0.3;
const SIDE_Y: f64 = STRIP_Y - 0.65;
const SIDE_WORDS_Y: f64 = STRIP_Y - 0.95;
const SPAN_Y: [f64; 2] = [STRIP_Y - 0.65, STRIP_Y - 1.25];
const SPAN_WORDS_DY: f64 = 0.25;
pub const SPAN_TICK: f64 = 0.1;
pub const SPAN_W: f64 = 3.0;
/// A row landing; the inspection holds (08_4, 08_7, 08_8) and the beat
/// after the last line.
const ROW_S: f64 = 0.3;
const HOLD_S: f64 = 2.5;
const END_BEAT: f64 = 0.6;

/// What chapter 08 shows, from the facts: the friends' five, the line's
/// first number, the handful size and their estimate; the company's count
/// and how many handfuls it has; the twenty draws, their estimates as
/// printed, how many, the lowest, highest, average and how many exact; the
/// stacks, how many handfuls top out at the handful size, at the next
/// number and at the count, the lowest and highest estimates; over every
/// handful the average, the exact share and its "1 in", the share over and
/// its percent, how far over and under on average, the average miss as a
/// fraction and as shown, the variance and its root as shown.
#[derive(Clone, Debug)]
struct Picture {
    numbers: Vec<u64>,
    first: u64,
    k: usize,
    estimate: u64,
    count: u64,
    handfuls: u64,
    draws: Vec<Vec<u64>>,
    drawn: Vec<String>,
    draw_count: usize,
    draw_lowest: String,
    draw_highest: String,
    draw_average: Q,
    draw_exact: usize,
    stacks: usize,
    smallest: u64,
    next: u64,
    next_count: u64,
    top_count: u64,
    lowest: u64,
    highest: String,
    average: Q,
    exact_share: Q,
    one_in: u64,
    over_share: Q,
    over_percent: u64,
    overshoot: Q,
    overshoot_about: u64,
    undershoot: Q,
    miss: Q,
    miss_shown: String,
    variance: Q,
    sd: String,
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
        handfuls: whole(&f, "G-F8.handfuls")?,
        draws: handfuls(&f, "G-F8.again_handfuls")?,
        drawn: printed(&f, "G-F8.again_estimates")?
            .split(',')
            .map(str::to_owned)
            .collect(),
        draw_count: whole(&f, "G-F8.again_draws")? as usize,
        draw_lowest: printed(&f, "G-F8.again_lowest")?,
        draw_highest: printed(&f, "G-F8.again_highest")?,
        draw_average: q("G-F8.again_average")?,
        draw_exact: whole(&f, "G-F8.again_exact")? as usize,
        stacks: whole(&f, "G-F8.stacks")? as usize,
        smallest: whole(&f, "G-F8.topped_by_smallest")?,
        next: whole(&f, "G-F8.next_largest")?,
        next_count: whole(&f, "G-F8.topped_by_next")?,
        top_count: whole(&f, "G-F8.topped_by_count")?,
        lowest: whole(&f, "G-F8.lowest_estimate")?,
        highest: printed(&f, "G-F8.highest_estimate")?,
        average: q("G-F8.average")?,
        exact_share: q("G-F8.exactly_right")?,
        one_in: whole(&f, "G-F8.exactly_right_one_in")?,
        over_share: q("G-F8.overestimate_share")?,
        over_percent: whole(&f, "G-F8.overestimate_percent")?,
        overshoot: q("G-F8.overshoot")?,
        overshoot_about: whole(&f, "G-F8.overshoot_about")?,
        undershoot: q("G-F8.undershoot")?,
        miss: q("G-F8.average_miss")?,
        miss_shown: printed(&f, "G-F8.average_miss_decimal")?,
        variance: q("G-F8.variance")?,
        sd: printed(&f, "G-F8.sd_about")?,
    })
}

/// A fraction as the facts print it: "13/2", or "5" when whole.
pub fn fraction(s: &str) -> Result<Q, String> {
    let bad = || format!("{s:?} is not a fraction");
    match s.split_once('/') {
        Some((n, d)) => {
            let n: i128 = n.parse().map_err(|_| bad())?;
            let d: i128 = d.parse().map_err(|_| bad())?;
            if d == 0 {
                return Err(bad());
            }
            Ok(Q::new(n, d))
        }
        None => Ok(Q::int(s.parse().map_err(|_| bad())?)),
    }
}

/// A fraction's value as a float, only to place it on screen (every
/// comparison the scene makes is exact): from the form it prints in.
pub fn approx(q: Q) -> Result<f64, String> {
    let s = q.to_string();
    match s.split_once('/') {
        Some((n, d)) => Ok(value(n)? / value(d)?),
        None => value(&s),
    }
}

/// Exactly two decimal places ("72.32"), only when that is exact.
fn decimal2(q: Q) -> Result<String, String> {
    let h = (q * Q::int(100))
        .whole()
        .ok_or_else(|| format!("{q} has no exact two-decimal form"))?;
    if h < 0 {
        return Err(format!("{q}: decimal2 is for positive values"));
    }
    Ok(format!("{}.{:02}", h / 100, h % 100))
}

/// The stacks of every handful of `k` from `n` boxes, one per largest
/// number from `k` to `n` in order: how many handfuls it tops, C(m − 1,
/// k − 1), and the estimate they all give.
fn stacks_of(n: u64, k: u64) -> Result<Vec<(u64, Q)>, String> {
    (k..=n)
        .map(|m| {
            let many = model::with_largest(m, k)
                .whole()
                .and_then(|w| u64::try_from(w).ok())
                .ok_or_else(|| format!("the handfuls topped by {m} are not a count"))?;
            Ok((many, model::gap_of(m, k)))
        })
        .collect()
}

/// Where the lowest and the highest of `values` first stand.
fn extremes(values: &[Q]) -> Result<(usize, usize), String> {
    let (Some(lo), Some(hi)) = (values.iter().min(), values.iter().max()) else {
        return Err("no values to find the lowest and highest of".to_owned());
    };
    let at = |x: &Q| values.iter().position(|y| y == x).unwrap_or(0);
    Ok((at(lo), at(hi)))
}

/// Each value's place in its stack: how many equal values came before it.
fn stack_levels(values: &[Q]) -> Vec<usize> {
    values
        .iter()
        .enumerate()
        .map(|(j, x)| values[..j].iter().filter(|&y| y == x).count())
        .collect()
}

/// The picture only if the scene can draw it truthfully: the friends' five
/// are a handful on the line from 1, their estimate the gap recipe's, and
/// the company's count that estimate; its handfuls are every handful of
/// five; the stacks run from the handful size to the count, the counts the
/// table shows are theirs, the last the tallest, and one stack is exactly
/// the count; over every handful the estimates average the count and the
/// exact, over and under shares, the two sides' averages and both widths
/// are the facts (the average miss squared under the variance, which is
/// the closed form); the draws are handfuls from the company, their
/// estimates printed exactly, the lowest, highest, average and exact count
/// theirs, and no stack of them taller than the scene leaves room for.
fn checked(pic: Picture) -> Result<Picture, String> {
    let k = pic.k as u64;
    let n = pic.count;
    let target = Q::int(n as i128);
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
    if u128::from(pic.handfuls) != choose(n, k) {
        return Err(format!(
            "{n} boxes have {} handfuls of {k}, not {}",
            choose(n, k),
            pic.handfuls
        ));
    }
    let stacks = stacks_of(n, k)?;
    let tallest = stacks.iter().map(|&(many, _)| many).max().unwrap_or(0);
    let exact_stacks = stacks.iter().filter(|&&(_, e)| e == target).count();
    if pic.stacks != stacks.len()
        || stacks.first().map(|s| s.0) != Some(pic.smallest)
        || pic.next != k + 1
        || stacks.get(1).map(|s| s.0) != Some(pic.next_count)
        || stacks.last().map(|s| s.0) != Some(pic.top_count)
        || pic.top_count != tallest
        || stacks.iter().rev().skip(1).any(|&(many, _)| many == tallest)
        || model::gap_of(k, k) != Q::int(pic.lowest as i128)
        || model::gap_of(n, k).decimal1()? != pic.highest
        || exact_stacks != 1
        || stacks.iter().map(|&(many, _)| u128::from(many)).sum::<u128>() != choose(n, k)
    {
        return Err(format!(
            "the stacks must be {} from {k} to {n}, topped by {}, {} and {} handfuls, the last the tallest, \
             and one exactly {n}",
            pic.stacks, pic.smallest, pic.next_count, pic.top_count
        ));
    }
    let est = |m: u64| model::gap_of(m, k);
    let exact = model::share_by_largest(n, k, |m| est(m) == target);
    let over = model::share_by_largest(n, k, |m| est(m) > target);
    let under = model::share_by_largest(n, k, |m| est(m) < target);
    if model::mean_by_largest(n, k, est) != target
        || pic.average != target
        || pic.exact_share != exact
        || pic.one_in as i128 != (Q::ONE / exact).nearest()
        || pic.over_share != over
        || pic.over_percent as i128 != (over * Q::int(100)).nearest()
    {
        return Err(format!(
            "over every handful the estimates must average {n}, be exactly {n} for {exact} of them (1 in {}) \
             and over it for {over} ({}%)",
            pic.one_in, pic.over_percent
        ));
    }
    let over_by = model::mean_by_largest(n, k, |m| (est(m) - target).max(Q::ZERO)) / over;
    let under_by = model::mean_by_largest(n, k, |m| (target - est(m)).max(Q::ZERO)) / under;
    if pic.overshoot != over_by || pic.overshoot_about as i128 != over_by.nearest() || pic.undershoot != under_by {
        return Err(format!(
            "the estimates over {n} must overshoot by {over_by} on average and those under fall short by {under_by}"
        ));
    }
    let miss = model::mean_by_largest(n, k, |m| (est(m) - target).abs());
    let variance = model::mean_by_largest(n, k, |m| (est(m) - target) * (est(m) - target));
    let closed = Q::new(((n - k) * (n + 1)) as i128, (k * (k + 2)) as i128);
    if pic.miss != miss
        || pic.miss_shown != miss.about1()?
        || pic.variance != variance
        || variance != closed
        || pic.sd != variance.sqrt_about1()?
        || pic.miss * pic.miss >= pic.variance
    {
        return Err(format!(
            "the average miss {} ({}) and the variance {} ({}) must be the widths of every handful's estimate",
            pic.miss, pic.miss_shown, pic.variance, pic.sd
        ));
    }
    if pic.draws.len() != pic.draw_count || !pic.draws.iter().all(|h| is_handful(h, pic.k, n)) {
        return Err(format!(
            "the draws {:?} must be {} handfuls of {k} from {n}",
            pic.draws, pic.draw_count
        ));
    }
    let ests: Vec<Q> = pic.draws.iter().map(|h| model::gap_recipe(h)).collect();
    let shown: Result<Vec<String>, String> = ests.iter().map(|q| q.decimal1()).collect();
    let (Some(&lo), Some(&hi)) = (ests.iter().min(), ests.iter().max()) else {
        return Err("the draws' estimates must exist".to_owned());
    };
    let mean = ests.iter().fold(Q::ZERO, |s, &q| s + q) / Q::int(ests.len() as i128);
    if pic.drawn != shown?
        || pic.draw_lowest != lo.decimal1()?
        || pic.draw_highest != hi.decimal1()?
        || pic.draw_average != mean
        || decimal2(mean).is_err()
        || pic.draw_exact != ests.iter().filter(|&&q| q == target).count()
    {
        return Err(format!(
            "the draws' estimates must be {:?}, from {} to {}, average {} and {} exactly {n}",
            pic.drawn, pic.draw_lowest, pic.draw_highest, pic.draw_average, pic.draw_exact
        ));
    }
    if stack_levels(&ests).into_iter().any(|l| l >= STACK_MAX) {
        return Err(format!(
            "the draws' stacks of equal estimates must be at most {STACK_MAX} high"
        ));
    }
    Ok(pic)
}

// ------------------------------------------------------------------ words

/// A count fixed for an experiment, on the panel (08, 09).
pub fn count_text(n: u64) -> String {
    format!("count = {n}")
}

fn handfuls_text(handfuls: u64, k: usize) -> String {
    format!("{} handfuls of {k}", thousands(handfuls))
}

/// The table's rows (08_4): the handful size, the next number and the
/// count, each with how many handfuls it tops and their estimate.
fn table_rows(pic: &Picture) -> Result<[[String; 3]; 3], String> {
    let next = model::gap_of(pic.next, pic.k as u64).decimal1()?;
    Ok([
        [pic.k.to_string(), pic.smallest.to_string(), pic.lowest.to_string()],
        [pic.next.to_string(), pic.next_count.to_string(), next],
        [pic.count.to_string(), thousands(pic.top_count), pic.highest.clone()],
    ])
}

fn exact_text(n: u64, one_in: u64) -> String {
    format!("exactly {n}: 1 in {one_in}")
}

fn draws_exact_text(draws: usize, exact: usize, n: u64) -> String {
    format!("{draws} draws: {exact} exactly {n}")
}

fn draws_average_text(draws: usize, average: &str) -> String {
    format!("{draws} draws: average {average}")
}

fn balance_text(average: Q) -> String {
    format!("balance point: {average}")
}

fn high_text(percent: u64, by: u64) -> String {
    format!("{percent}% too high, by about {by}")
}

fn low_text(by: Q) -> String {
    format!("too low: by {by}")
}

fn miss_label(shown: &str) -> String {
    format!("average miss: {shown}")
}

fn sd_label(shown: &str) -> String {
    format!("standard deviation: {shown}")
}

fn miss_def(n: u64) -> String {
    format!("average miss = average distance from {n}")
}

fn single_text(n: u64) -> String {
    format!("our {n}: a single estimate")
}

/// Card E (08_9), from the facts.
fn card_e(pic: &Picture) -> String {
    let k = pic.k;
    format!(
        "for N = {} and handfuls of {k}, all {} estimates average exactly N; they miss by {} on average, and their \
         standard deviation is √((N − {k})(N + 1) ÷ {}) ≈ {}.",
        pic.count,
        thousands(pic.handfuls),
        pic.miss_shown,
        k * (k + 2),
        pic.sd
    )
}

// ------------------------------------------------------------- builders

/// An estimate's green tick at number `n` on a line shown to `end`, and its
/// number over it. Detached.
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

/// A plain dot of the line at number `n`, as the line draws a number nobody
/// saw (08, 09). Detached.
pub fn plain_dot(stage: &mut Stage<'_>, n: u64, end: f64) -> Mob {
    let x = x_at(n as f64, end);
    let d = Dot::new()
        .point(v(x, STRIP_Y))
        .radius(DOT_R)
        .style(flat(palette::line(), dot_opacity(x)))
        .build();
    stage.arena_mut().add(d)
}

/// A count fixed for an experiment (08, 09): an upright ink mark at number
/// `n` on a line shown to `end`, from just under the line up to `top`.
/// Detached.
pub fn count_mark(stage: &mut Stage<'_>, n: u64, end: f64, top: f64) -> Mob {
    let x = x_at(n as f64, end);
    stroke(
        stage,
        vec![v(x, STRIP_Y - MARK_DOWN), v(x, top)],
        line_style(Palette::ink(), MARK_W, MARK_O),
    )
}

/// A bar standing on the line, centred at `x`, `w` wide and `h` tall, a
/// flat fill at opacity `o`. Detached.
pub fn bar(stage: &mut Stage<'_>, x: f64, w: f64, h: f64, color: Srgb, o: f64) -> fmn::Result<Mob> {
    let r = rect(v(x - w / 2.0, STRIP_Y), w, h, flat(color, o))?;
    Ok(stage.arena_mut().add(r))
}

/// Bars rise from the line, one after another, the whole taking `rt` (as
/// the kit grows a histogram).
pub fn grow_bars(stage: &mut Stage<'_>, bars: &[Mob], rt: f64, lag: f64) -> fmn::Result<()> {
    if bars.is_empty() {
        return Ok(());
    }
    for &b in bars {
        stage.add_to_scene(b)?;
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &b in bars {
        let a = fmn::animation::grow_from_edge(stage.arena_mut(), b, DOWN, None)?;
        anims.push(stage.prepare(a)?);
    }
    let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, lag)?.with_run_time(rt);
    stage.play(g)?;
    Ok(())
}

/// A span under the line from `x0` to `x1` at height `y`, with a short
/// upward tick at each end (a bracket, a width, a range).
pub fn span_points(x0: f64, x1: f64, y: f64) -> Vec<Vec3> {
    vec![v(x0, y + SPAN_TICK), v(x0, y), v(x1, y), v(x1, y + SPAN_TICK)]
}

/// Move `m` so its right edge is at `right`, centred on height `y`.
pub fn right_at(stage: &mut Stage<'_>, m: Mob, right: f64, y: f64) {
    let w = stage.get_width(m);
    stage.move_to(m, v(right - w / 2.0, y), ORIGIN);
}

/// A drawn row landing (08_2), the row before it going and that row's
/// estimate dropping `fall` onto its stack, together.
fn land_row(stage: &mut Stage<'_>, row: &[Mob], gone: &[Mob], dot: Option<(Mob, f64)>, rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in row {
        let a = fade_in(stage.arena_mut(), m, v(0.0, -DROP), 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    for &m in gone {
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    if let Some((d, fall)) = dot {
        let a = fade_in(stage.arena_mut(), d, v(0.0, -fall), 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    if anims.is_empty() {
        return Ok(());
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// A dot's height in a stack over the line.
fn stack_y(level: usize) -> f64 {
    STACK_Y0 + STACK_STEP * level as f64
}

/// The bars (08_4): one per stack, green, at its estimate on the line
/// shown to `end`, as tall as its share of the tallest. Those below the
/// count's estimate, left to right; the one at exactly the count, kept back
/// until the answer; those above it, the tallest last; where the "?" waits
/// over the empty place; and the two sides' outer edges.
struct Bars {
    under: Vec<Mob>,
    exact: Mob,
    over: Vec<Mob>,
    hidden_at: Vec3,
    under_x: [f64; 2],
    over_x: [f64; 2],
}

impl Bars {
    fn build(stage: &mut Stage<'_>, pic: &Picture, end: f64) -> fmn::Result<Bars> {
        let target = Q::int(pic.count as i128);
        let stacks = stacks_of(pic.count, pic.k as u64).map_err(fail)?;
        let (mut under, mut over, mut exact) = (Vec::new(), Vec::new(), None);
        let (mut under_xs, mut over_xs, mut beside) = (Vec::new(), Vec::new(), Vec::new());
        for (j, &(many, e)) in stacks.iter().enumerate() {
            let x = x_at(approx(e).map_err(fail)?, end);
            let h = BAR_H * many as f64 / pic.top_count as f64;
            let b = bar(stage, x, BAR_W, h, palette::estimate(), BAR_O)?;
            if e < target {
                under.push(b);
                under_xs.push(x);
            } else if e > target {
                over.push(b);
                over_xs.push(x);
            } else {
                exact = Some((b, x));
                for i in [j.wrapping_sub(1), j + 1] {
                    if let Some(&(m, _)) = stacks.get(i) {
                        beside.push(BAR_H * m as f64 / pic.top_count as f64);
                    }
                }
            }
        }
        let (exact, x) = exact.ok_or_else(|| fail(format!("no stack at exactly {}", pic.count)))?;
        let top = beside.into_iter().fold(0.0, f64::max);
        let edges = |xs: &[f64]| -> fmn::Result<[f64; 2]> {
            match (xs.first(), xs.last()) {
                (Some(&a), Some(&b)) => Ok([a - BAR_W / 2.0, b + BAR_W / 2.0]),
                _ => Err(fail("a side of the count with no stacks".to_owned())),
            }
        };
        Ok(Bars {
            under_x: edges(&under_xs)?,
            over_x: edges(&over_xs)?,
            under,
            exact,
            over,
            hidden_at: v(x, STRIP_Y + top + HIDDEN_DY),
        })
    }

    /// The tallest, the count's own stack.
    fn tallest(&self) -> Vec<Mob> {
        self.over.last().copied().into_iter().collect()
    }

    /// Every other bar but the count's estimate, left to right.
    fn rest(&self) -> Vec<Mob> {
        let mut all = self.under.clone();
        all.extend(&self.over[..self.over.len().saturating_sub(1)]);
        all
    }
}

impl SceneConstruct for OneEstimate {
    fn name(&self) -> &str {
        "08_one_estimate"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let pic = picture().map_err(fail)?;
        let end = LINE_DOTS as f64;
        let target = Q::int(pic.count as i128);

        // the friends' line as chapter 03 left it, their estimate's green
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
                widest: end,
            },
        )?;
        let corner = badge(&kit, stage, at(BADGE_AT))?;
        let ours = estimate_tick(&kit, stage, pic.estimate, end)?;
        let mut opened = strip.opening();
        opened.extend(ours);
        opened.push(corner);
        fade_in_all(stage, &opened, 0.8)?;

        // 08_1: a halo round our seventy-one; the boxes, their ticks and
        // the estimate go, the gold dots turn plain, and the company's mark
        // rises at its count; a halo round the badge
        let line = say(&kit, stage, "08_1")?;
        line.wait_for(stage, "not about our seventy-one")?;
        glow(stage, &[ours[1]], palette::estimate())?;
        line.wait_for(stage, "imagine a company")?;
        let plain: Vec<Mob> = pic.numbers.iter().map(|&n| plain_dot(stage, n, end)).collect();
        let mut gone: Vec<Mob> = pic
            .numbers
            .iter()
            .map(|&n| strip.dots[(n - pic.first) as usize])
            .collect();
        gone.extend(&strip.bodies);
        gone.extend(&strip.labels);
        gone.extend(&strip.ticks);
        gone.extend(ours);
        replace(stage, &gone, &plain, 0.6)?;
        let mark = count_mark(stage, pic.count, end, MARK_TOP);
        let count = words_left(&kit, stage, &count_text(pic.count), &[], panel(0))?;
        fade_in_all(stage, &[mark, count], 0.5)?;
        line.wait_for(stage, "buy five fresh boxes")?;
        glow(stage, &[corner], Palette::ink())?;

        // 08_2: twenty rows land one after another; as each lands the one
        // before it goes and its estimate drops onto its stack; the rows
        // keep landing after the words end
        let line = say(&kit, stage, "08_2")?;
        line.wait_for(stage, "Here are twenty such handfuls")?;
        let ests: Vec<Q> = pic.draws.iter().map(|h| model::gap_recipe(h)).collect();
        let levels = stack_levels(&ests);
        let spots: Vec<f64> = pic
            .drawn
            .iter()
            .map(|s| value(s))
            .collect::<Result<_, _>>()
            .map_err(fail)?;
        let mut dots = Vec::new();
        let mut last: Vec<Mob> = Vec::new();
        let mut falling: Option<(Mob, f64)> = None;
        for (j, hand) in pic.draws.iter().enumerate() {
            let row: Vec<Mob> = hand
                .iter()
                .map(|&n| ring(stage, v(x_at(n as f64, end), DRAW_Y)))
                .collect();
            land_row(stage, &row, &last, falling.take(), ROW_S)?;
            let y = stack_y(levels[j]);
            let d = Dot::new()
                .point(v(x_at(spots[j], end), y))
                .radius(EST_R)
                .style(flat(palette::estimate(), 1.0))
                .build();
            let dot = stage.arena_mut().add(d);
            falling = Some((dot, DRAW_Y - y));
            dots.push(dot);
            last = row;
        }
        land_row(stage, &[], &last, falling.take(), ROW_S)?;

        // 08_3: halos round the lowest and the highest, their values under
        // the line; a halo round the stack on the mark; the draws' average
        let line = say(&kit, stage, "08_3")?;
        line.wait_for(stage, "Their estimates land between")?;
        let (lo, hi) = extremes(&ests).map_err(fail)?;
        let lo_label = kit.text(
            stage,
            &pic.draw_lowest,
            PANEL_SIZE,
            palette::estimate(),
            v(x_at(spots[lo], end), UNDER_Y),
        )?;
        let hi_label = kit.text(
            stage,
            &pic.draw_highest,
            PANEL_SIZE,
            palette::estimate(),
            v(x_at(spots[hi], end), UNDER_Y),
        )?;
        fade_in_all(stage, &[lo_label, hi_label], 0.4)?;
        glow_each(
            stage,
            &[
                (vec![dots[lo]], palette::estimate()),
                (vec![dots[hi]], palette::estimate()),
            ],
            0.3,
            0.5,
        )?;
        line.wait_for(stage, "three of them land exactly")?;
        let on: Vec<Mob> = dots
            .iter()
            .zip(&ests)
            .filter(|&(_, &q)| q == target)
            .map(|(&d, _)| d)
            .collect();
        glow(stage, &on, palette::estimate())?;
        line.wait_for(stage, "Together they average")?;
        let ax = x_at(approx(pic.draw_average).map_err(fail)?, end);
        let average_mark = stroke(
            stage,
            vec![v(ax, STRIP_Y - AVERAGE_MARK[0]), v(ax, STRIP_Y - AVERAGE_MARK[1])],
            line_style(palette::estimate(), TICK_W, 1.0),
        );
        let average_words = kit.text(
            stage,
            &format!("average {}", decimal2(pic.draw_average).map_err(fail)?),
            PANEL_SIZE,
            palette::estimate(),
            v(ax, AVERAGE_Y),
        )?;
        fade_in_all(stage, &[average_mark, average_words], 0.5)?;

        // 08_4: the draws go; every handful, the recipe; the table row by
        // row and the tallest bar; the other bars, but the count's own
        // place stays empty under a "?"; held to be inspected
        let line = say(&kit, stage, "08_4")?;
        line.wait_for(stage, "Twenty is only twenty")?;
        let mut gone = dots.clone();
        gone.extend([lo_label, hi_label, average_mark, average_words]);
        fade_out_all(stage, &gone, 0.5)?;
        line.wait_for(stage, "over thirteen million")?;
        let all = words_left(&kit, stage, &handfuls_text(pic.handfuls, pic.k), &[], panel(1))?;
        fade_in_all(stage, &[all], 0.5)?;
        line.wait_for(stage, "the recipe looks only at")?;
        let recipe = words_left(&kit, stage, &formula(pic.k), &formula_colors(), panel(2))?;
        fade_in_all(stage, &[recipe], 0.5)?;
        let colors = [palette::seen(), Palette::ink(), palette::estimate()];
        let mut heads = Vec::new();
        for (c, h) in HEADS.iter().enumerate() {
            heads.push(kit.text(stage, h, TABLE_SIZE, Palette::dim(), v(TABLE_X[c], TABLE_Y[0]))?);
        }
        let mut table = Vec::new();
        for (r, row) in table_rows(&pic).map_err(fail)?.iter().enumerate() {
            let mut cells = Vec::new();
            for (c, s) in row.iter().enumerate() {
                cells.push(kit.text(stage, s, TABLE_SIZE, colors[c], v(TABLE_X[c], TABLE_Y[r + 1]))?);
            }
            table.push(cells);
        }
        let bars = Bars::build(stage, &pic, end)?;
        for (r, phrase) in [
            "Just one handful tops out",
            "Five handfuls top out",
            "Over nine hundred thousand",
        ]
        .into_iter()
        .enumerate()
        {
            line.wait_for(stage, phrase)?;
            let mut shown = table[r].clone();
            if r == 0 {
                shown.extend(&heads);
            }
            fade_in_lagged(stage, &shown, 0.6, 0.2)?;
        }
        grow_bars(stage, &bars.tallest(), 0.6, 0.0)?;
        line.wait_for(stage, "sixty-seven stacks")?;
        grow_bars(stage, &bars.rest(), 1.6, 0.05)?;
        let hidden = kit.text(stage, QUESTION, PANEL_SIZE, palette::top_gap(), bars.hidden_at)?;
        fade_in_all(stage, &[hidden], 0.4)?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;

        // 08_5: the prediction; the countdown at the left, under the table
        say(&kit, stage, "08_5")?;
        let q = think(&kit, stage, at(THINK_08))?;

        // 08_6: the count's own bar rises in the empty place, a halo round
        // it; one in twenty-nine; the draws' three, in dim under the table
        let line = say(&kit, stage, "08_6")?;
        fade_out_all(stage, &[q, hidden], 0.3)?;
        grow_bars(stage, &[bars.exact], 0.5, 0.0)?;
        line.wait_for(stage, "About one handful")?;
        let exact = words_left(&kit, stage, &exact_text(pic.count, pic.one_in), &[], panel(3))?;
        fade_in_all(stage, &[exact], 0.4)?;
        glow(stage, &[bars.exact], Palette::ink())?;
        line.wait_for(stage, "Three in twenty was lucky")?;
        let three = kit.text_left(
            stage,
            &draws_exact_text(pic.draw_count, pic.draw_exact, pic.count),
            TABLE_SIZE,
            Palette::dim(),
            v(FOOT_X, FOOT_Y[0]),
        )?;
        fade_in_all(stage, &[three], 0.4)?;

        // 08_7: the fulcrum at the count; the draws' average in dim; halos
        // round the two sides; each side's bracket and its words; held to
        // be inspected
        let line = say(&kit, stage, "08_7")?;
        line.wait_for(stage, "balance at seventy-one exactly")?;
        let fx = x_at(approx(pic.average).map_err(fail)?, end);
        let tip = STRIP_Y - FULCRUM_GAP;
        let fulcrum = stroke(
            stage,
            vec![
                v(fx - FULCRUM[0], tip - FULCRUM[1]),
                v(fx + FULCRUM[0], tip - FULCRUM[1]),
                v(fx, tip),
                v(fx - FULCRUM[0], tip - FULCRUM[1]),
            ],
            line_style(palette::estimate(), MARK_W, 1.0),
        );
        let balance = kit.text(
            stage,
            &balance_text(pic.average),
            PANEL_SIZE,
            palette::estimate(),
            v(fx, BALANCE_Y),
        )?;
        right_at(stage, balance, fx - BALANCE_GAP, BALANCE_Y);
        fade_in_all(stage, &[fulcrum, balance], 0.5)?;
        line.wait_for(stage, "our twenty draws averaged")?;
        let draws_average = kit.text_left(
            stage,
            &draws_average_text(pic.draw_count, &decimal2(pic.draw_average).map_err(fail)?),
            TABLE_SIZE,
            Palette::dim(),
            v(FOOT_X, FOOT_Y[1]),
        )?;
        fade_in_all(stage, &[draws_average], 0.4)?;
        line.wait_for(stage, "The bars are lopsided")?;
        glow_each(
            stage,
            &[
                (bars.over.clone(), Palette::ink()),
                (bars.under.clone(), Palette::ink()),
            ],
            0.3,
            0.6,
        )?;
        line.wait_for(stage, "Fifty-eight percent of handfuls")?;
        let high_span = stroke(
            stage,
            span_points(bars.over_x[0], bars.over_x[1], SIDE_Y),
            line_style(Palette::dim(), SPAN_W, 1.0),
        );
        let high = kit.text(
            stage,
            &high_text(pic.over_percent, pic.overshoot_about),
            PANEL_SIZE,
            Palette::ink(),
            v(0.0, SIDE_WORDS_Y),
        )?;
        right_at(stage, high, bars.over_x[1], SIDE_WORDS_Y);
        fade_in_all(stage, &[high_span, high], 0.5)?;
        line.wait_for(stage, "the estimates that fall short")?;
        let low_span = stroke(
            stage,
            span_points(bars.under_x[0], bars.under_x[1], SIDE_Y),
            line_style(Palette::dim(), SPAN_W, 1.0),
        );
        let low = kit.text(
            stage,
            &low_text(pic.undershoot),
            PANEL_SIZE,
            Palette::ink(),
            v((bars.under_x[0] + bars.under_x[1]) / 2.0, SIDE_WORDS_Y),
        )?;
        fade_in_all(stage, &[low_span, low], 0.5)?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;

        // 08_8: the brackets go; the average miss and the standard
        // deviation as spans about the count; the panel's definitions;
        // held to be read
        let line = say(&kit, stage, "08_8")?;
        fade_out_all(stage, &[high_span, high, low_span, low], 0.4)?;
        let middle = approx(target).map_err(fail)?;
        let miss = approx(pic.miss).map_err(fail)?;
        let sd = approx(pic.variance).map_err(fail)?.sqrt();
        let mut spans = Vec::new();
        for (j, (width, words, color, phrase)) in [
            (
                miss,
                miss_label(&pic.miss_shown),
                Palette::ink(),
                "misses by about nine boxes",
            ),
            (sd, sd_label(&pic.sd), Palette::dim(), "the standard deviation"),
        ]
        .into_iter()
        .enumerate()
        {
            line.wait_for(stage, phrase)?;
            let (a, b) = (x_at(middle - width, end), x_at(middle + width, end));
            let s = stroke(stage, span_points(a, b, SPAN_Y[j]), line_style(color, SPAN_W, 1.0));
            let w = kit.text(stage, &words, PANEL_SIZE, color, v(fx, SPAN_Y[j] - SPAN_WORDS_DY))?;
            fade_in_all(stage, &[s, w], 0.5)?;
            spans.extend([s, w]);
        }
        line.wait_for(stage, "It squares each miss")?;
        let defs = [
            words_left(&kit, stage, &miss_def(pic.count), &[], panel(1))?,
            words_left(&kit, stage, SD_DEF, &[], panel(2))?,
        ];
        replace(stage, &[all, recipe, exact], &defs, 0.5)?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;

        // 08_9: halos round the balance and round the widths; our single
        // estimate on the panel; card E, held until it can be read
        let line = say(&kit, stage, "08_9")?;
        line.wait_for(stage, "The recipe is unbiased")?;
        glow(stage, &[fulcrum, balance], palette::estimate())?;
        line.wait_for(stage, "They spread out around it")?;
        glow(stage, &spans, Palette::ink())?;
        line.wait_for(stage, "And our seventy-one")?;
        let single = kit.text_left(
            stage,
            &single_text(pic.estimate),
            PANEL_SIZE,
            palette::estimate(),
            panel(3),
        )?;
        fade_in_all(stage, &[single], 0.5)?;
        let text = card_e(&pic);
        let card = precisely(&kit, stage, &text)?;
        fade_in_all(stage, &[card], 0.4)?;
        let card_up = now(stage);
        wait_until(stage, line.start + line.len)?;
        pause(stage, END_BEAT)?;
        // card E stays until it can be read
        wait_until(stage, card_up + read_s(&text))?;

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
    use crate::chapters::friends_line::{BADGE, BADGE_PAD, BADGE_SIZE, PANEL_X, PANEL_Y, RING_R, est_width};
    use crate::model::{AGAIN_STREAM, FRIENDS, KNOWN, all_handfuls, drawn_handfuls, gap_recipe};

    /// The countdown's note, at the size `think` writes it.
    const NOTE_SIZE: f64 = 18.0;
    /// A line's half height at the panel's size and at the table's, and the
    /// space kept between things that must not touch.
    const HALF_LINE: f64 = 0.16;
    const HALF_TABLE: f64 = 0.12;
    const CLEAR: f64 = 0.1;
    /// A "precisely:" card's line, beside its label: the kit starts at 92
    /// characters and narrows by 4 until the widest line fits.
    const CARD_CHARS_WIDEST: usize = 92;
    const CARD_CHARS_NARROWEST: usize = 64;

    fn good() -> Picture {
        picture().unwrap()
    }

    /// The scene draws the facts, and its lines read as the voice says
    /// them: the friends' five and the company are the model's, the twenty
    /// draws are stream 3 of the generator, and every line the panel, the
    /// table, the words under the line and card E show.
    #[test]
    fn the_picture_is_the_facts() {
        let pic = good();
        assert_eq!(pic.numbers, FRIENDS);
        assert_eq!(pic.count, KNOWN);
        assert_eq!(pic.draws, drawn_handfuls(AGAIN_STREAM, KNOWN, pic.k, pic.draw_count));
        assert_eq!(count_text(pic.count), "count = 71");
        assert_eq!(handfuls_text(pic.handfuls, pic.k), "13,019,909 handfuls of 5");
        assert_eq!(formula(pic.k), "estimate = largest + (largest − 5) ÷ 5");
        let rows = [["5", "1", "5"], ["6", "5", "6.2"], ["71", "916,895", "84.2"]];
        assert_eq!(table_rows(&pic).unwrap(), rows.map(|r| r.map(str::to_owned)));
        assert_eq!(exact_text(pic.count, pic.one_in), "exactly 71: 1 in 29");
        assert_eq!(
            draws_exact_text(pic.draw_count, pic.draw_exact, pic.count),
            "20 draws: 3 exactly 71"
        );
        assert_eq!(
            draws_average_text(pic.draw_count, &decimal2(pic.draw_average).unwrap()),
            "20 draws: average 72.32"
        );
        assert_eq!((pic.draw_lowest.as_str(), pic.draw_highest.as_str()), ("44.6", "84.2"));
        assert_eq!(balance_text(pic.average), "balance point: 71");
        assert_eq!(
            high_text(pic.over_percent, pic.overshoot_about),
            "58% too high, by about 8"
        );
        assert_eq!(low_text(pic.undershoot), "too low: by 12");
        assert_eq!(miss_label(&pic.miss_shown), "average miss: 9.2");
        assert_eq!(sd_label(&pic.sd), "standard deviation: 11.7");
        assert_eq!(miss_def(pic.count), "average miss = average distance from 71");
        assert_eq!(single_text(pic.estimate), "our 71: a single estimate");
        assert_eq!(
            card_e(&pic),
            "for N = 71 and handfuls of 5, all 13,019,909 estimates average exactly N; they miss by 9.2 on \
             average, and their standard deviation is √((N − 5)(N + 1) ÷ 35) ≈ 11.7."
        );
    }

    /// Fractions are read as the facts print them, two decimals only when
    /// exact, and the stacks and extremes are found where they are.
    #[test]
    fn fractions_and_decimals_are_read_and_shown_exactly() {
        assert_eq!(fraction("13/2").unwrap(), Q::new(13, 2));
        assert_eq!(fraction("71").unwrap(), Q::int(71));
        for bad in ["1/0", "x", "1/x", "", "2/"] {
            assert!(fraction(bad).is_err(), "{bad:?}");
        }
        assert_eq!(decimal2(Q::new(1808, 25)).unwrap(), "72.32");
        assert_eq!(decimal2(Q::new(7, 100)).unwrap(), "0.07");
        assert_eq!(decimal2(Q::int(71)).unwrap(), "71.00");
        assert!(decimal2(Q::new(1, 3)).is_err());
        assert!(decimal2(Q::new(1, 1000)).is_err());
        assert!(decimal2(Q::new(-1, 4)).is_err());
        assert!((approx(Q::new(1808, 25)).unwrap() - 72.32).abs() < 1e-12);
        assert!((approx(Q::int(5)).unwrap() - 5.0).abs() < 1e-12);
        let qs = [Q::int(1), Q::int(2), Q::int(1), Q::int(1)];
        assert_eq!(stack_levels(&qs), [0, 0, 1, 2]);
        assert_eq!(extremes(&[Q::int(3), Q::int(1), Q::int(4), Q::int(1)]).unwrap(), (1, 2));
        assert!(extremes(&[]).is_err());
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
                count: 70,
                ..pic.clone()
            },
            "friends' estimate",
        );
        refused(
            Picture {
                handfuls: pic.handfuls + 1,
                ..pic.clone()
            },
            "boxes have",
        );
        refused(
            Picture {
                stacks: pic.stacks + 1,
                ..pic.clone()
            },
            "the stacks",
        );
        refused(
            Picture {
                next_count: 4,
                ..pic.clone()
            },
            "the stacks",
        );
        refused(
            Picture {
                top_count: 900_000,
                ..pic.clone()
            },
            "the stacks",
        );
        refused(
            Picture {
                highest: "84.0".to_owned(),
                ..pic.clone()
            },
            "the stacks",
        );
        // "right on average" read as "usually right"
        refused(
            Picture {
                exact_share: Q::new(1, 2),
                ..pic.clone()
            },
            "over every handful",
        );
        refused(
            Picture {
                one_in: 28,
                ..pic.clone()
            },
            "over every handful",
        );
        // the draws' average is not the balance point
        refused(
            Picture {
                average: Q::new(1808, 25),
                ..pic.clone()
            },
            "over every handful",
        );
        refused(
            Picture {
                over_percent: 50,
                ..pic.clone()
            },
            "over every handful",
        );
        refused(
            Picture {
                undershoot: Q::int(8),
                ..pic.clone()
            },
            "overshoot",
        );
        refused(
            Picture {
                overshoot_about: 9,
                ..pic.clone()
            },
            "overshoot",
        );
        // the standard deviation is not the average miss, nor the variance
        refused(
            Picture {
                sd: pic.miss_shown.clone(),
                ..pic.clone()
            },
            "average miss",
        );
        refused(
            Picture {
                miss: pic.variance,
                ..pic.clone()
            },
            "average miss",
        );
        refused(
            Picture {
                miss_shown: "9.3".to_owned(),
                ..pic.clone()
            },
            "average miss",
        );
        refused(
            Picture {
                draw_count: pic.draw_count - 1,
                ..pic.clone()
            },
            "handfuls of 5 from 71",
        );
        let mut draws = pic.draws.clone();
        draws[0] = vec![6, 10, 43, 44, 72];
        refused(Picture { draws, ..pic.clone() }, "handfuls of 5 from 71");
        let mut drawn = pic.drawn.clone();
        drawn[1] = "81.0".to_owned();
        refused(Picture { drawn, ..pic.clone() }, "draws' estimates");
        refused(
            Picture {
                draw_exact: pic.draw_exact - 1,
                ..pic.clone()
            },
            "draws' estimates",
        );
        refused(
            Picture {
                draw_average: Q::int(72),
                ..pic.clone()
            },
            "draws' estimates",
        );
        // a fourth exact draw, every fact made to agree with it: its stack
        // would rise past the room the scene leaves under the rows
        let mut draws = pic.draws.clone();
        draws[1] = draws[0].clone();
        let mut drawn = pic.drawn.clone();
        drawn[1] = drawn[0].clone();
        let sum = draws.iter().fold(Q::ZERO, |s, h| s + gap_recipe(h));
        refused(
            Picture {
                draw_average: sum / Q::int(draws.len() as i128),
                draws,
                drawn,
                draw_exact: pic.draw_exact + 1,
                ..pic
            },
            "at most 3 high",
        );
    }

    /// The bars need no list of handfuls: for every small company
    /// (handfuls of 1 to 4 from up to 12 boxes), counting by the largest
    /// number gives the same stacks, estimates, average and average miss as
    /// listing every handful, and the variance is (N − k)(N + 1) ÷ (k(k +
    /// 2)). The negative controls: the average miss squared falls short of
    /// the variance once the estimates spread unevenly (equal only for one
    /// box from two), and the 71-box company's middle estimate is not its
    /// balance point.
    #[test]
    fn counting_by_the_largest_number_is_listing_every_handful() {
        for k in 1..=4u64 {
            for n in k..=12 {
                let hands = all_handfuls(n, k as usize);
                let count = Q::int(hands.len() as i128);
                let target = Q::int(n as i128);
                for (j, &(many, e)) in stacks_of(n, k).unwrap().iter().enumerate() {
                    let m = k + j as u64;
                    let topped: Vec<&Vec<u64>> = hands.iter().filter(|h| h[h.len() - 1] == m).collect();
                    assert_eq!(topped.len() as u64, many, "k {k}, n {n}, m {m}");
                    assert!(topped.iter().all(|h| gap_recipe(h) == e), "k {k}, n {n}, m {m}");
                }
                let mean = hands.iter().fold(Q::ZERO, |s, h| s + gap_recipe(h)) / count;
                let miss = hands.iter().fold(Q::ZERO, |s, h| s + (gap_recipe(h) - target).abs()) / count;
                let var = hands.iter().fold(Q::ZERO, |s, h| {
                    let d = gap_recipe(h) - target;
                    s + d * d
                }) / count;
                assert_eq!(mean, target, "k {k}, n {n}");
                assert_eq!(
                    model::mean_by_largest(n, k, |m| (model::gap_of(m, k) - target).abs()),
                    miss
                );
                assert_eq!(
                    var,
                    Q::new(((n - k) * (n + 1)) as i128, (k * (k + 2)) as i128),
                    "k {k}, n {n}"
                );
                if n >= k + 2 {
                    assert!(miss * miss < var, "k {k}, n {n}");
                } else {
                    assert!(miss * miss <= var, "k {k}, n {n}");
                }
            }
        }
        let stacks = stacks_of(KNOWN, 5).unwrap();
        let total: u64 = stacks.iter().map(|s| s.0).sum();
        assert_eq!(u128::from(total), choose(KNOWN, 5));
        let mut acc = 0;
        let middle = stacks
            .iter()
            .find(|s| {
                acc += s.0;
                2 * acc >= total
            })
            .unwrap()
            .1;
        assert_eq!(middle, Q::new(373, 5));
        assert_ne!(middle, Q::int(KNOWN as i128));
    }

    /// Every phrase the scene waits for is in its line of script.tsv, once,
    /// and a line's phrases come in the order the scene waits for them.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let waits: &[(&str, &[&str])] = &[
            (
                "08_1",
                &["not about our seventy-one", "imagine a company", "buy five fresh boxes"],
            ),
            ("08_2", &["Here are twenty such handfuls"]),
            (
                "08_3",
                &[
                    "Their estimates land between",
                    "three of them land exactly",
                    "Together they average",
                ],
            ),
            (
                "08_4",
                &[
                    "Twenty is only twenty",
                    "over thirteen million",
                    "the recipe looks only at",
                    "Just one handful tops out",
                    "Five handfuls top out",
                    "Over nine hundred thousand",
                    "sixty-seven stacks",
                ],
            ),
            ("08_6", &["About one handful", "Three in twenty was lucky"]),
            (
                "08_7",
                &[
                    "balance at seventy-one exactly",
                    "our twenty draws averaged",
                    "The bars are lopsided",
                    "Fifty-eight percent of handfuls",
                    "the estimates that fall short",
                ],
            ),
            (
                "08_8",
                &[
                    "misses by about nine boxes",
                    "the standard deviation",
                    "It squares each miss",
                ],
            ),
            (
                "08_9",
                &[
                    "The recipe is unbiased",
                    "They spread out around it",
                    "And our seventy-one",
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
        // the prediction's line shows nothing of the answer
        let asked = script.text("08_5").unwrap();
        for answer in ["twenty-nine", "rarely", "one in"] {
            assert!(!asked.contains(answer), "{answer}");
        }
    }

    /// The panel, the table, the countdown and the bars keep apart: every
    /// panel line in the title-safe area, the badge left of it; the table
    /// under the badge, its columns apart and left of the panel, its foot
    /// under it; the countdown under the table and over the line; every
    /// bar inside the frame, under the panel, and too low to reach the
    /// table or the countdown where they stand over it; the "?" over the
    /// empty place clear of its neighbours and the count's mark; the draws'
    /// dot stacks between the line and the drawn rows.
    #[test]
    fn the_panel_the_table_and_the_bars_fit_the_frame() {
        let pic = good();
        let safe_x = FRAME_X_RADIUS - 0.25;
        let end = LINE_DOTS as f64;
        for text in [
            count_text(pic.count),
            handfuls_text(pic.handfuls, pic.k),
            formula(pic.k),
            exact_text(pic.count, pic.one_in),
            miss_def(pic.count),
            SD_DEF.to_owned(),
            single_text(pic.estimate),
        ] {
            assert!(PANEL_X + est_width(&text, PANEL_SIZE) < safe_x, "{text}");
        }
        let badge_right = BADGE_AT[0] + est_width(BADGE, BADGE_SIZE) / 2.0 + BADGE_PAD[0];
        assert!(badge_right < PANEL_X - CLEAR);
        // the table
        const { assert!(TABLE_Y[0] + HALF_TABLE < BADGE_AT[1] - 0.15 - BADGE_PAD[1]) };
        let rows = table_rows(&pic).unwrap();
        let widths: Vec<f64> = (0..3)
            .map(|c| {
                rows.iter()
                    .map(|r| est_width(&r[c], TABLE_SIZE))
                    .fold(est_width(HEADS[c], TABLE_SIZE), f64::max)
            })
            .collect();
        for c in 0..2 {
            assert!(
                TABLE_X[c] + widths[c] / 2.0 + CLEAR < TABLE_X[c + 1] - widths[c + 1] / 2.0,
                "column {c}"
            );
        }
        let table_right = TABLE_X[2] + widths[2] / 2.0;
        assert!(table_right < PANEL_X - CLEAR);
        assert!(TABLE_X[0] - widths[0] / 2.0 > -safe_x);
        for w in TABLE_Y.windows(2) {
            assert!(w[0] - w[1] > 2.0 * HALF_TABLE + 0.1);
        }
        const { assert!(FOOT_Y[0] + HALF_TABLE < TABLE_Y[3] - HALF_TABLE) };
        const { assert!(FOOT_Y[1] + HALF_TABLE < FOOT_Y[0] - HALF_TABLE) };
        let average = decimal2(pic.draw_average).unwrap();
        for text in [
            draws_exact_text(pic.draw_count, pic.draw_exact, pic.count),
            draws_average_text(pic.draw_count, &average),
        ] {
            assert!(FOOT_X + est_width(&text, TABLE_SIZE) < PANEL_X - CLEAR, "{text}");
        }
        // the countdown: under the table's last row and over the line, with
        // its note (the foot lines come once it has gone)
        let note_y = THINK_08[1] - THINK_R - 0.3;
        const { assert!(THINK_08[1] + THINK_R < TABLE_Y[3] - HALF_TABLE - CLEAR) };
        assert!(note_y - 0.1 > STRIP_Y + CLEAR);
        let note_half = est_width(PAUSE_NOTE, NOTE_SIZE) / 2.0;
        // the bars
        let stacks = stacks_of(pic.count, pic.k as u64).unwrap();
        let height = |many: u64| BAR_H * many as f64 / pic.top_count as f64;
        let mut xs = Vec::new();
        for &(many, e) in &stacks {
            let x = x_at(approx(e).unwrap(), end);
            let top = STRIP_Y + height(many);
            assert!(x + BAR_W / 2.0 < safe_x && x - BAR_W / 2.0 > -safe_x, "{e}");
            assert!(top < PANEL_Y[3] - HALF_LINE - CLEAR, "{e}");
            if x - BAR_W / 2.0 < table_right {
                assert!(top < FOOT_Y[1] - HALF_TABLE - CLEAR, "{e} under the table");
            }
            if (x - THINK_08[0]).abs() < note_half.max(THINK_R) + BAR_W {
                assert!(top < note_y - 0.1 - CLEAR, "{e} under the countdown");
            }
            xs.push(x);
        }
        for w in xs.windows(2) {
            assert!(w[1] - w[0] > BAR_W + 0.02);
        }
        // the "?" over the empty place: above both neighbours and the mark
        let target = Q::int(pic.count as i128);
        let j = stacks.iter().position(|s| s.1 == target).unwrap();
        let beside = height(stacks[j - 1].0).max(height(stacks[j + 1].0));
        let hidden_y = STRIP_Y + beside + HIDDEN_DY;
        assert!(hidden_y - HALF_LINE > STRIP_Y + beside + CLEAR);
        assert!(hidden_y - HALF_LINE > MARK_TOP);
        assert!(hidden_y + HALF_LINE < PANEL_Y[3] - HALF_LINE);
        // the draws' stacks between the line and the rows; equal neighbours'
        // dots apart
        const { assert!(STACK_Y0 - EST_R > STRIP_Y + DOT_R) };
        const { assert!(STACK_Y0 + STACK_STEP * (STACK_MAX - 1) as f64 + EST_R < DRAW_Y - RING_R - CLEAR) };
        const { assert!(STACK_STEP > 2.0 * EST_R) };
        assert!(x_at(1.0 + 1.2, end) - x_at(1.0, end) > 2.0 * EST_R + 0.02);
    }

    /// Under the line: the draws' lowest and highest values apart and the
    /// average's words clear of them (08_3); the balance's words left of
    /// the fulcrum, both over the brackets, the two sides' words apart
    /// (08_7); the two widths' spans and words in turn, and card E's three
    /// lines under all of it (08_8, 08_9).
    #[test]
    fn the_words_under_the_line_keep_apart() {
        let pic = good();
        let end = LINE_DOTS as f64;
        let safe_x = FRAME_X_RADIUS - 0.25;
        let ests: Vec<Q> = pic.draws.iter().map(|h| gap_recipe(h)).collect();
        let (lo, hi) = extremes(&ests).unwrap();
        let lo_x = x_at(approx(ests[lo]).unwrap(), end);
        let hi_x = x_at(approx(ests[hi]).unwrap(), end);
        let lo_w = est_width(&pic.draw_lowest, PANEL_SIZE);
        let hi_w = est_width(&pic.draw_highest, PANEL_SIZE);
        assert!(lo_x + lo_w / 2.0 + CLEAR < hi_x - hi_w / 2.0);
        assert!(hi_x + hi_w / 2.0 < safe_x);
        let ax = x_at(approx(pic.draw_average).unwrap(), end);
        assert!(
            (ax - hi_x).abs() > hi_w / 2.0 + CLEAR,
            "the average's mark clear of the highest's value"
        );
        const { assert!(AVERAGE_Y + HALF_LINE < UNDER_Y - HALF_LINE) };
        const { assert!(STRIP_Y - AVERAGE_MARK[1] > AVERAGE_Y + HALF_LINE) };
        // 08_7
        let fx = x_at(approx(pic.average).unwrap(), end);
        let balance_w = est_width(&balance_text(pic.average), PANEL_SIZE);
        assert!(fx - BALANCE_GAP - balance_w > -safe_x);
        assert!(fx - BALANCE_GAP < fx - FULCRUM[0] - CLEAR);
        const { assert!(BALANCE_Y - HALF_LINE > SIDE_Y + SPAN_TICK + CLEAR / 2.0) };
        const { assert!(STRIP_Y - FULCRUM_GAP - FULCRUM[1] > SIDE_Y + SPAN_TICK + CLEAR) };
        const { assert!(SIDE_WORDS_Y + HALF_LINE < SIDE_Y - CLEAR / 2.0) };
        let stacks = stacks_of(pic.count, pic.k as u64).unwrap();
        let target = Q::int(pic.count as i128);
        let first_over = stacks.iter().find(|s| s.1 > target).unwrap().1;
        let last_under = stacks.iter().rev().find(|s| s.1 < target).unwrap().1;
        let (over_left, over_right) = (
            x_at(approx(first_over).unwrap(), end) - BAR_W / 2.0,
            x_at(approx(model::gap_of(pic.count, pic.k as u64)).unwrap(), end) + BAR_W / 2.0,
        );
        let under_right = x_at(approx(last_under).unwrap(), end) + BAR_W / 2.0;
        let under_left = x_at(approx(model::gap_of(pic.k as u64, pic.k as u64)).unwrap(), end) - BAR_W / 2.0;
        assert!(under_right < fx - BAR_W / 2.0 && over_left > fx + BAR_W / 2.0);
        let high_w = est_width(&high_text(pic.over_percent, pic.overshoot_about), PANEL_SIZE);
        let low_w = est_width(&low_text(pic.undershoot), PANEL_SIZE);
        assert!(over_right < safe_x);
        assert!((under_left + under_right) / 2.0 + low_w / 2.0 + CLEAR < over_right - high_w);
        // 08_8 and card E
        let miss = approx(pic.miss).unwrap();
        let sd = approx(pic.variance).unwrap().sqrt();
        for (width, words) in [(miss, miss_label(&pic.miss_shown)), (sd, sd_label(&pic.sd))] {
            let w = est_width(&words, PANEL_SIZE);
            assert!(fx + w / 2.0 < safe_x && fx + width * (x_at(2.0, end) - x_at(1.0, end)) < safe_x);
        }
        const { assert!(SPAN_Y[0] + SPAN_TICK < STRIP_Y - FULCRUM_GAP - FULCRUM[1] - CLEAR) };
        const { assert!(SPAN_Y[1] + SPAN_TICK < SPAN_Y[0] - SPAN_WORDS_DY - HALF_LINE - CLEAR / 2.0) };
        // card E's three lines (0.3 plus 0.36 a line, 0.2 over the frame's
        // foot) stay under the second width's words
        const { assert!(-FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 3.0 < SPAN_Y[1] - SPAN_WORDS_DY - HALF_LINE) };
    }

    /// Card E says the exact widths in at most three lines at every width
    /// the lower third might narrow to, and is up long enough to be read.
    #[test]
    fn card_e_fits_and_is_read() {
        let text = card_e(&good());
        for chars in (CARD_CHARS_NARROWEST..=CARD_CHARS_WIDEST).step_by(4) {
            assert!(wrap(&text, chars, 4).len() <= 3, "{chars}");
        }
        assert!(read_s(&text) < 15.0);
        // the negative control: one clause more would not fit
        let longer = format!("{text} The friends' estimate is one value, not a count.");
        assert!(wrap(&longer, CARD_CHARS_NARROWEST, 4).len() > 3);
    }
}
