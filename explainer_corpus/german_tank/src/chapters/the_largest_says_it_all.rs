//! Chapter 11, "The largest number says it all": on the friends' line, only
//! the box at sixty keeps its number and the other four could be any four
//! below it (11_1). Tried on pairs first: the tiny company's ten pairs as a
//! triangle of dots, the four topped by five gold, each one time in four
//! (11_2); companies of six and eight boxes beside it have more pairs, but
//! the pairs topped by five are the same four, each still one time in four
//! (11_3). The same with sixty: over four hundred and fifty thousand groups
//! of four, each 1 in 455,126 at sixty, seventy-one and a thousand boxes,
//! while the chance that sixty is the largest falls from one in twelve,
//! held to be inspected (11_4). The prediction: how much can the other four
//! tell you about the count (11_5)? Nothing; the largest number is
//! sufficient, with card G (11_6). The callback: whatever the other four
//! are, the gaps below sixty hold fifty-five numbers, the friends' and the
//! strange handful's (11_7); all of it needs the handful to be fair (11_8).
//!
//! The pairs are listed whole, small cases only; nothing at the scale of
//! sixty is listed: the groups and chances come from counts of handfuls by
//! their largest number, and the scene checks every value it shows against
//! the facts. Highlights are halos and colour changes are replacements: no
//! typeset text is indicated, recoloured or scaled in place.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, choose, fade_all, thousands, v};
use fmn::prelude::*;

use super::draw::{
    LABEL_SIZE, STRIP_Y, dot_opacity, fade_in_all, fade_out_all, fail, flat, line_style, pause, refill, say, stroke,
    think, wait_until, x_at,
};
use super::friends_line::{
    BADGE_AT, PANEL_SIZE, THINK_AT, at, badge, is_handful, list, now, panel, printed, read_s, whole,
};
use super::gaps_trade_places::{GapLine, glow_each};
use super::one_estimate::{SPAN_W, fraction, span_points};
use super::what_we_assume::{glow, replace, words_left};
use crate::facts::{self, GROUPS_ROUND, TINY_SEEN};
use crate::model::{self, LINE_DOTS, Q};
use crate::palette;

pub struct TheLargestSaysItAll {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const SUFFICIENT: &str = "the largest number: sufficient";
const COUNT_HEAD: &str = "count";
const MILLION_WORD: &str = "million";
/// The phrases that bring in the table's rows (11_4), one per count.
const ROW_CUES: [&str; 3] = ["With exactly sixty boxes", "with seventy-one", "with a thousand"];

/// The far count's chance is said in millions (11_4).
const MILLION: i128 = 1_000_000;
/// "Ours split them fairly evenly" (11_7): no gap of the friends' holds
/// half the unseen numbers; "nearly all of them in a single gap": one of
/// the strange handful's holds more than nine tenths.
const NEARLY_ALL: [u64; 2] = [9, 10];
/// The others (11_1): a dim span under the line from the first number to
/// the one below the largest, its words under it.
const SPAN_Y: f64 = STRIP_Y - 0.35;
const OTHERS_Y: f64 = STRIP_Y - 0.65;
/// The pairs (11_2, 11_3): one triangle per company, its columns the larger
/// numbers from `FIRST_LARGER` up, `GRID_STEP` apart, centred on
/// `GRID_X`, its rows the smaller numbers from 1 up on `GRID_FOOT`; three
/// caption lines under each, the smaller numbers left of the first
/// triangle, and the larger number five over its column.
const FIRST_LARGER: u64 = TINY_SEEN as u64;
const GRID_X: [f64; 3] = [-5.0, -2.2, 0.6];
const GRID_STEP: f64 = 0.22;
const GRID_FOOT: f64 = -2.3;
const PAIR_R: f64 = 0.045;
const CAPTION_Y: [f64; 3] = [-2.65, -2.95, -3.25];
const ROW_LABEL_GAP: f64 = 0.3;
const TOP_DY: f64 = 0.25;
/// The table under the line (11_4): its columns' left edges, its heads'
/// height and the step between rows, over card G's top.
const TABLE_X: [f64; 3] = [-5.9, -4.1, -1.8];
const TABLE_Y0: f64 = -1.15;
const TABLE_DY: f64 = 0.33;
/// The inspection holds (11_4, 11_6) and the chapter's last beat.
const HOLD_S: f64 = 2.5;
const END_BEAT: f64 = 0.6;

/// What chapter 11 shows, from the facts: the friends' five, the line's
/// first number, the handful size, the largest and the lower bound, how
/// many others there are; the company of chapter 08 and the far count; the
/// tiny company and its pairs, the larger number the pairs are topped by,
/// the companies tried, the larger ones and their pairs, how many pairs are
/// topped, their smaller numbers, each one's chance and their shares; the
/// groups of four below the largest and the round number they pass; the
/// three counts, the chance at each that the largest is the friends', each
/// group's chance given it, the "1 in" the voice says for the first two and
/// the millions for the third; the friends' gaps and how many unseen numbers
/// they hold, the strange handful, its gaps and their total.
#[derive(Clone, Debug)]
struct Picture {
    numbers: Vec<u64>,
    first: u64,
    k: usize,
    largest: u64,
    at_least: u64,
    others: usize,
    count: u64,
    thousand: u64,
    tiny: u64,
    tiny_pairs: u64,
    pair_top: u64,
    companies: Vec<u64>,
    more: Vec<u64>,
    more_pairs: Vec<u64>,
    topped: usize,
    smaller: Vec<u64>,
    each: Q,
    shares: Vec<Q>,
    groups: u64,
    groups_over: u64,
    counts: Vec<u64>,
    chances: Vec<Q>,
    per_group: Vec<Q>,
    one_in: Vec<u64>,
    millions: u64,
    gaps: Vec<u64>,
    unseen: u64,
    silly: Vec<u64>,
    silly_gaps: Vec<u64>,
    silly_unseen: u64,
}

fn picture() -> Result<Picture, String> {
    let f = facts::facts()?;
    let q = |key: &str| -> Result<Q, String> { fraction(&printed(&f, key)?) };
    let qs = |key: &str| -> Result<Vec<Q>, String> { printed(&f, key)?.split(',').map(fraction).collect() };
    let counts = list(&f, "G-F11.counts")?;
    let [edge, known, far] = counts[..] else {
        return Err(format!("chapter 11 compares three counts, not {counts:?}"));
    };
    checked(Picture {
        numbers: list(&f, "G-F1.numbers")?,
        first: whole(&f, "G-F1.first")?,
        k: whole(&f, "G-F1.handful")? as usize,
        largest: whole(&f, "G-F1.largest")?,
        at_least: whole(&f, "G-F1.at_least")?,
        others: whole(&f, "G-F10.others")? as usize,
        count: whole(&f, "G-F8.count")?,
        thousand: whole(&f, "G-F1.thousand")?,
        tiny: whole(&f, "G-F5.boxes")?,
        tiny_pairs: whole(&f, "G-F5.pairs")?,
        pair_top: whole(&f, "G-F11.pair_top")?,
        companies: list(&f, "G-F11.pair_companies")?,
        more: list(&f, "G-F11.pair_more")?,
        more_pairs: list(&f, "G-F11.pair_more_pairs")?,
        topped: whole(&f, "G-F11.pairs_topped")? as usize,
        smaller: list(&f, "G-F11.pair_smaller")?,
        each: q("G-F11.pair_each")?,
        shares: qs("G-F11.pair_shares")?,
        groups: whole(&f, "G-F11.others_sets")?,
        groups_over: whole(&f, "G-F11.others_sets_over")?,
        chances: counts
            .iter()
            .map(|n| q(&format!("G-F11.top_chance.{n}")))
            .collect::<Result<_, _>>()?,
        per_group: counts
            .iter()
            .map(|n| q(&format!("G-F11.each_group.{n}")))
            .collect::<Result<_, _>>()?,
        one_in: vec![
            whole(&f, &format!("G-F11.top_chance.{edge}.one_in"))?,
            whole(&f, &format!("G-F11.top_chance.{known}.one_in"))?,
        ],
        millions: whole(&f, &format!("G-F11.top_chance.{far}.millions"))?,
        counts,
        gaps: list(&f, "G-F3.gaps")?,
        unseen: whole(&f, "G-F3.unseen")?,
        silly: list(&f, "G-F10.silly_handful")?,
        silly_gaps: list(&f, "G-F11.silly_gaps")?,
        silly_unseen: whole(&f, "G-F11.silly_unseen")?,
    })
}

/// The other numbers of every handful of `k` from 1 to `n` topped by `top`,
/// in order, listed whole (small cases only: the pairs).
fn others_of(n: u64, k: usize, top: u64) -> Vec<Vec<u64>> {
    model::all_handfuls(n, k)
        .into_iter()
        .filter(|h| h[k - 1] == top)
        .map(|h| h[..k - 1].to_vec())
        .collect()
}

/// The picture only if the scene can draw it truthfully: the friends' five
/// are a handful on the line from 1 topped by the lower bound, with one
/// fewer others; the companies start with the tiny one and their pairs are
/// all the pairs, the pairs topped by its largest number the same in every
/// company, their smaller numbers every number below it, each one over how
/// many there are and their share falling; the groups are every way to pick
/// the others below the largest and the round number is under them; the
/// counts are the edge count, chapter 08's and the far one, the chance the
/// largest is the friends' falls from k over the largest at the edge, each
/// group's chance is one handful's over it and the same at every count,
/// "1 in" exact only at the edge and the millions the far chance's; the
/// gaps below the largest hold the largest less the handful size for the
/// friends and the strange handful alike, the friends' fairly evenly and
/// the strange handful's nearly all in one gap.
fn checked(pic: Picture) -> Result<Picture, String> {
    let k = pic.k as u64;
    let m = pic.largest;
    if pic.first != 1
        || !is_handful(&pic.numbers, pic.k, LINE_DOTS)
        || pic.numbers.last() != Some(&m)
        || pic.at_least != m
        || pic.others + 1 != pic.k
    {
        return Err(format!(
            "the friends' handful {:?} must be topped by {m}, the lower bound, with {} others",
            pic.numbers, pic.others
        ));
    }
    let pair = TINY_SEEN as u64;
    let pairs_of = |n: u64| choose(n, pair);
    let column = others_of(pic.tiny, TINY_SEEN, pic.pair_top);
    let below: Vec<Vec<u64>> = (pic.first..pic.pair_top).map(|s| vec![s]).collect();
    let shares: Vec<Q> = pic
        .companies
        .iter()
        .map(|&n| model::share_by_largest(n, pair, |mm| mm == pic.pair_top))
        .collect();
    if pic.companies.first() != Some(&pic.tiny)
        || pic.companies[1..] != pic.more[..]
        || pic.companies.windows(2).any(|w| w[1] <= w[0])
        || pic.companies.len() != GRID_X.len()
        || pic.pair_top != pic.tiny
        || u128::from(pic.tiny_pairs) != pairs_of(pic.tiny)
        || pic.more_pairs.iter().map(|&p| u128::from(p)).collect::<Vec<_>>()
            != pic.more.iter().map(|&n| pairs_of(n)).collect::<Vec<_>>()
        || pic
            .companies
            .iter()
            .any(|&n| others_of(n, TINY_SEEN, pic.pair_top) != column)
        || column != below
        || column.is_empty()
        || pic.topped != column.len()
        || pic.smaller != column.iter().map(|o| o[0]).collect::<Vec<_>>()
        || pic.each != Q::new(1, pic.topped as i128)
        || pic.shares != shares
        || pic
            .companies
            .iter()
            .zip(&pic.shares)
            .any(|(&n, &s)| s != Q::new(pic.topped as i128, pairs_of(n) as i128))
        || pic.shares.windows(2).any(|w| w[1] >= w[0])
    {
        return Err(format!(
            "the pairs topped by {} must be the same {} in every company {:?}, each {}, their shares {:?}",
            pic.pair_top, pic.topped, pic.companies, pic.each, pic.shares
        ));
    }
    let groups = choose(m - 1, k - 1);
    if u128::from(pic.groups) != groups
        || model::with_largest(m, k) != Q::count(groups)
        || u128::from(pic.groups_over) != groups / GROUPS_ROUND * GROUPS_ROUND
        || pic.groups_over >= pic.groups
    {
        return Err(format!(
            "the {} groups of {} below {m} must be every way to pick them, over {}",
            pic.groups, pic.others, pic.groups_over
        ));
    }
    let top_chance = |n: u64| model::share_by_largest(n, k, |mm| mm == m);
    let one_in = |c: Q, o: u64| c == Q::ONE / Q::int(o as i128);
    if pic.counts != [m, pic.count, pic.thousand]
        || pic.counts.windows(2).any(|w| w[1] <= w[0])
        || pic.chances.len() != pic.counts.len()
        || pic.per_group.len() != pic.counts.len()
        || pic.counts.iter().zip(&pic.chances).any(|(&n, &c)| c != top_chance(n))
        || pic
            .counts
            .iter()
            .zip(pic.chances.iter().zip(&pic.per_group))
            .any(|(&n, (&c, &g))| g != Q::ONE / Q::count(choose(n, k)) / c || g != Q::ONE / Q::count(groups))
        || pic.chances[0] != Q::new(k as i128, m as i128)
        || pic.chances.windows(2).any(|w| w[1] >= w[0])
        || pic.one_in.len() + 1 != pic.counts.len()
        || pic
            .one_in
            .iter()
            .zip(&pic.chances)
            .any(|(&o, &c)| (Q::ONE / c).nearest() != o as i128)
        || !one_in(pic.chances[0], pic.one_in[0])
        || one_in(pic.chances[1], pic.one_in[1])
        || (Q::ONE / pic.chances[2] / Q::int(MILLION)).nearest() != pic.millions as i128
        || Q::ONE / pic.chances[2] == Q::int(pic.millions as i128 * MILLION)
    {
        return Err(format!(
            "the chance that {m} is the largest must be {:?} at the counts {:?}, falling, each group's {:?}",
            pic.chances, pic.counts, pic.per_group
        ));
    }
    let most = |g: &[u64]| g.iter().copied().max().unwrap_or(0);
    if pic.gaps != model::gaps(&pic.numbers)
        || pic.unseen != pic.gaps.iter().sum::<u64>()
        || pic.unseen != m - k
        || !is_handful(&pic.silly, pic.k, LINE_DOTS)
        || pic.silly.last() != Some(&m)
        || pic.silly_gaps != model::gaps(&pic.silly)
        || pic.silly_unseen != pic.silly_gaps.iter().sum::<u64>()
        || pic.silly_unseen != pic.unseen
        || 2 * most(&pic.gaps) >= pic.unseen
        || NEARLY_ALL[1] * most(&pic.silly_gaps) <= NEARLY_ALL[0] * pic.unseen
    {
        return Err(format!(
            "the gaps below {m} must hold {} = {m} − {k} for the friends' {:?} and the strange handful's {:?}",
            pic.unseen, pic.gaps, pic.silly_gaps
        ));
    }
    Ok(pic)
}

// ------------------------------------------------------------------ words

fn largest_text(m: u64) -> String {
    format!("largest: {m}")
}

/// The others' freedom (11_1): any of them from the first number to the one
/// below the largest.
fn others_text(pic: &Picture) -> String {
    format!("any {} of {} to {}", pic.others, pic.first, pic.largest - 1)
}

/// Each triangle's three caption lines (11_2, 11_3): the company and its
/// pairs, the share topped by its largest number, each one's chance.
fn captions(pic: &Picture) -> Result<Vec<[String; 3]>, String> {
    let each = (Q::ONE / pic.each)
        .whole()
        .ok_or_else(|| format!("each pair's chance {} is not 1 in a whole number", pic.each))?;
    let pairs = std::iter::once(pic.tiny_pairs).chain(pic.more_pairs.iter().copied());
    Ok(pic
        .companies
        .iter()
        .zip(pairs)
        .map(|(&n, p)| {
            [
                format!("{n} boxes: {p} pairs"),
                format!("topped by {}: {} of {p}", pic.pair_top, pic.topped),
                format!("each 1 in {each}"),
            ]
        })
        .collect())
}

fn groups_text(pic: &Picture) -> String {
    format!(
        "{} groups of {} below {}",
        thousands(pic.groups),
        pic.others,
        pic.largest
    )
}

/// The table's heads (11_4): the count, the chance the largest is the
/// friends', and each group's chance given it.
fn heads(pic: &Picture) -> [String; 3] {
    [
        COUNT_HEAD.to_owned(),
        format!("largest is {}", pic.largest),
        format!("each group of {}", pic.others),
    ]
}

/// Count `i`'s chance that the largest is the friends', as the voice says
/// it: "1 in" exactly at the edge, "≈" past it, the far one in millions.
fn chance_shown(pic: &Picture, i: usize) -> String {
    match pic.one_in.get(i) {
        Some(&o) if pic.chances[i] == Q::ONE / Q::int(o as i128) => format!("1 in {}", thousands(o)),
        Some(&o) => format!("≈ 1 in {}", thousands(o)),
        None => format!("≈ 1 in {} {MILLION_WORD}", pic.millions),
    }
}

/// The table's rows (11_4), one per count.
fn rows(pic: &Picture) -> Result<Vec<[String; 3]>, String> {
    pic.counts
        .iter()
        .zip(&pic.per_group)
        .enumerate()
        .map(|(i, (&n, &g))| {
            let one_in = (Q::ONE / g)
                .whole()
                .and_then(|w| u128::try_from(w).ok())
                .ok_or_else(|| format!("each group's chance {g} at {n} is not 1 in a whole number"))?;
            Ok([
                thousands(n),
                chance_shown(pic, i),
                format!("1 in {}", thousands(one_in)),
            ])
        })
        .collect()
}

/// Every word of the table, for its reading time.
fn table_text(pic: &Picture) -> Result<String, String> {
    let mut words: Vec<String> = heads(pic).to_vec();
    for row in rows(pic)? {
        words.extend(row);
    }
    Ok(words.join(" "))
}

/// A card's exact statement of sufficiency (11_6), the fairness it rests on
/// first.
fn card_g(pic: &Picture) -> String {
    let m = pic.largest;
    format!(
        "fair handful from any N ≥ {m}, given largest {m}: each group of {} below {m} is the others with chance \
         1/{}. Only the chance of largest {m} depends on N: the largest is sufficient for N.",
        pic.others,
        thousands(pic.groups)
    )
}

fn sum_text(terms: &[u64]) -> String {
    terms.iter().map(u64::to_string).collect::<Vec<_>>().join(" + ")
}

/// The friends' gaps below the largest (11_7) and what they hold.
fn gaps_text(pic: &Picture) -> String {
    format!("{} = {} = {} − {}", sum_text(&pic.gaps), pic.unseen, pic.largest, pic.k)
}

/// The strange handful's others and gaps (11_7).
fn silly_text(pic: &Picture) -> String {
    let others: Vec<String> = pic.silly[..pic.others].iter().map(u64::to_string).collect();
    format!(
        "{}: {} = {}",
        others.join(", "),
        sum_text(&pic.silly_gaps),
        pic.silly_unseen
    )
}

// ------------------------------------------------------------- builders

/// Column `larger` of triangle `g`, a company of `n` boxes.
fn grid_x(g: usize, n: u64, larger: u64) -> f64 {
    GRID_X[g] + (larger as f64 - (FIRST_LARGER + n) as f64 / 2.0) * GRID_STEP
}

/// The row of the smaller number `smaller`.
fn grid_y(smaller: u64) -> f64 {
    GRID_FOOT + (smaller as f64 - 1.0) * GRID_STEP
}

fn table_y(j: usize) -> f64 {
    TABLE_Y0 - TABLE_DY * j as f64
}

/// One pair's dot. Detached.
fn pair_dot(stage: &mut Stage<'_>, at: Vec3, color: Srgb) -> Mob {
    let d = Dot::new().point(at).radius(PAIR_R).style(flat(color, 1.0)).build();
    stage.arena_mut().add(d)
}

/// A company's pairs as a triangle (11_2, 11_3), detached: the dim dots of
/// the pairs not topped by `top`; those topped by it, smallest first, and
/// the gold ones that replace them; the gold number over their column; and
/// its three caption lines.
struct Triangle {
    rest: Vec<Mob>,
    column: Vec<Mob>,
    gold: Vec<Mob>,
    five: Mob,
    captions: [Mob; 3],
}

/// Triangle `g`, for a company of `n` boxes: each pair (smaller, larger) in
/// the column of its larger number and at the height of its smaller.
fn triangle(
    kit: &Kit,
    stage: &mut Stage<'_>,
    g: usize,
    n: u64,
    top: u64,
    words: &[String; 3],
) -> fmn::Result<Triangle> {
    let (mut rest, mut column, mut gold) = (Vec::new(), Vec::new(), Vec::new());
    for pair in model::all_handfuls(n, TINY_SEEN) {
        let (smaller, larger) = (pair[0], pair[TINY_SEEN - 1]);
        let spot = v(grid_x(g, n, larger), grid_y(smaller));
        let dim = pair_dot(stage, spot, Palette::dim());
        if larger == top {
            column.push(dim);
            gold.push(pair_dot(stage, spot, palette::seen()));
        } else {
            rest.push(dim);
        }
    }
    let five = kit.text(
        stage,
        &top.to_string(),
        LABEL_SIZE,
        palette::seen(),
        v(grid_x(g, n, top), grid_y(top - 1) + TOP_DY),
    )?;
    let caption = |i: usize, stage: &mut Stage<'_>| {
        kit.text(stage, &words[i], LABEL_SIZE, Palette::ink(), v(GRID_X[g], CAPTION_Y[i]))
    };
    let captions = [caption(0, stage)?, caption(1, stage)?, caption(2, stage)?];
    Ok(Triangle {
        rest,
        column,
        gold,
        five,
        captions,
    })
}

/// The smaller numbers of the pairs topped by five, dim, left of the first
/// triangle's rows. Detached.
fn row_labels(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture) -> fmn::Result<Vec<Mob>> {
    let x = grid_x(0, pic.tiny, FIRST_LARGER) - ROW_LABEL_GAP;
    let mut labels = Vec::new();
    for &s in &pic.smaller {
        labels.push(kit.text(stage, &s.to_string(), LABEL_SIZE, Palette::dim(), v(x, grid_y(s)))?);
    }
    Ok(labels)
}

/// The table under the line (11_4): the heads, dim, then one row per count,
/// each cell's left edge at its column's. Detached.
fn table(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture) -> fmn::Result<Vec<Vec<Mob>>> {
    let mut cells = vec![heads(pic).to_vec()];
    cells.extend(rows(pic).map_err(fail)?.into_iter().map(|r| r.to_vec()));
    let mut table = Vec::new();
    for (j, row) in cells.iter().enumerate() {
        let color = if j == 0 { Palette::dim() } else { Palette::ink() };
        let mut mobs = Vec::new();
        for (c, s) in row.iter().enumerate() {
            mobs.push(kit.text_left(stage, s, LABEL_SIZE, color, v(TABLE_X[c], table_y(j)))?);
        }
        table.push(mobs);
    }
    Ok(table)
}

impl SceneConstruct for TheLargestSaysItAll {
    fn name(&self) -> &str {
        "11_the_largest_says_it_all"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let pic = picture().map_err(fail)?;
        let end = LINE_DOTS as f64;
        let top_box = pic.numbers.len() - 1;
        let dot = |n: u64| (n - pic.first) as usize;

        // the friends' line, the boxes on their ticks, and the badge; the
        // gaps' arcs and counts (11_7) built with it
        let gl = GapLine::build(&kit, stage, pic.first, LINE_DOTS, &pic.numbers)?;
        let strip = &gl.strip;
        for &piece in &strip.base {
            stage.add_to_scene(piece)?;
        }
        let corner = badge(&kit, stage, at(BADGE_AT))?;
        let mut opened = strip.opening();
        opened.push(corner);
        fade_in_all(stage, &opened, 0.8)?;
        let mut four = strip.bodies[..top_box].to_vec();
        four.extend(&strip.labels[..top_box]);
        four.extend(&strip.ticks[..top_box]);
        let plain: Vec<(Mob, Srgb, f64)> = pic.numbers[..top_box]
            .iter()
            .map(|&n| (strip.dots[dot(n)], palette::line(), dot_opacity(x_at(n as f64, end))))
            .collect();
        let lit: Vec<(Mob, Srgb, f64)> = pic.numbers[..top_box]
            .iter()
            .map(|&n| (strip.dots[dot(n)], palette::seen(), 1.0))
            .collect();

        // 11_1: the four smaller boxes go and their dots turn plain, the
        // largest on the panel; the others' span under the line
        let line = say(&kit, stage, "11_1")?;
        line.wait_for(stage, "all you were told")?;
        fade_out_all(stage, &four, 0.5)?;
        refill(stage, &plain, 0.4)?;
        let largest = words_left(&kit, stage, &largest_text(pic.largest), &[], panel(0))?;
        fade_in_all(stage, &[largest], 0.5)?;
        line.wait_for(stage, "any four different numbers below sixty")?;
        let (x0, x1) = (x_at(pic.first as f64, end), x_at((pic.largest - 1) as f64, end));
        let span = stroke(
            stage,
            span_points(x0, x1, SPAN_Y),
            line_style(Palette::dim(), SPAN_W, 1.0),
        );
        let any = kit.text(
            stage,
            &others_text(&pic),
            PANEL_SIZE,
            Palette::ink(),
            v((x0 + x1) / 2.0, OTHERS_Y),
        )?;
        fade_in_all(stage, &[span, any], 0.5)?;

        // 11_2: the tiny company's pairs; the four topped by five turn gold;
        // their smaller numbers; each one's chance
        let line = say(&kit, stage, "11_2")?;
        line.wait_for(stage, "Try it on pairs")?;
        fade_out_all(stage, &[span, any], 0.4)?;
        let words = captions(&pic).map_err(fail)?;
        let mut grids = Vec::new();
        for (g, (&n, w)) in pic.companies.iter().zip(&words).enumerate() {
            grids.push(triangle(&kit, stage, g, n, pic.pair_top, w)?);
        }
        let tiny = &grids[0];
        let mut landed = tiny.rest.clone();
        landed.extend(&tiny.column);
        landed.push(tiny.captions[0]);
        fade_in_all(stage, &landed, 0.6)?;
        line.wait_for(stage, "four of its ten pairs")?;
        let mut gold = tiny.gold.clone();
        gold.extend([tiny.five, tiny.captions[1]]);
        replace(stage, &tiny.column, &gold, 0.5)?;
        line.wait_for(stage, "the smaller one is one")?;
        let labels = row_labels(&kit, stage, &pic)?;
        fade_in_all(stage, &labels, 0.4)?;
        line.wait_for(stage, "so each is one time in four")?;
        fade_in_all(stage, &[tiny.captions[2]], 0.4)?;

        // 11_3: the larger companies beside it; their shares; their pairs
        // topped by five turn gold, a halo round every such column; each
        // one's chance again
        let line = say(&kit, stage, "11_3")?;
        line.wait_for(stage, "six boxes, or eight")?;
        let larger = &grids[1..];
        let mut landed = Vec::new();
        for t in larger {
            landed.extend(&t.rest);
            landed.extend(&t.column);
            landed.push(t.captions[0]);
        }
        fade_in_all(stage, &landed, 0.6)?;
        line.wait_for(stage, "a larger number of five gets rarer")?;
        let rarer: Vec<Mob> = larger.iter().map(|t| t.captions[1]).collect();
        fade_in_all(stage, &rarer, 0.4)?;
        line.wait_for(stage, "the pairs topped by five")?;
        let (mut out, mut into) = (Vec::new(), Vec::new());
        for t in larger {
            out.extend(&t.column);
            into.extend(&t.gold);
            into.push(t.five);
        }
        replace(stage, &out, &into, 0.5)?;
        let columns: Vec<(Vec<Mob>, Srgb)> = grids.iter().map(|t| (t.gold.clone(), palette::seen())).collect();
        glow_each(stage, &columns, 0.3, 0.6)?;
        line.wait_for(stage, "each is still one time in four")?;
        let still: Vec<Mob> = larger.iter().map(|t| t.captions[2]).collect();
        fade_in_all(stage, &still, 0.4)?;

        // 11_4: the triangles go; the groups on the panel; the table, a row
        // as each count is named, held to be inspected
        let line = say(&kit, stage, "11_4")?;
        line.wait_for(stage, "It's the same with sixty")?;
        let mut gone = labels;
        for t in &grids {
            gone.extend(&t.rest);
            gone.extend(&t.gold);
            gone.push(t.five);
            gone.extend(t.captions);
        }
        fade_out_all(stage, &gone, 0.5)?;
        line.wait_for(stage, "over four hundred and fifty thousand groups")?;
        let groups = words_left(&kit, stage, &groups_text(&pic), &[], panel(1))?;
        fade_in_all(stage, &[groups], 0.5)?;
        let table = table(&kit, stage, &pic)?;
        line.wait_for(stage, "Only the chance that sixty is the largest changes")?;
        fade_in_all(stage, &table[0], 0.5)?;
        for (row, cue) in table[1..].iter().zip(ROW_CUES) {
            line.wait_for(stage, cue)?;
            fade_in_all(stage, row, 0.4)?;
        }
        let table_up = now(stage);
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;
        wait_until(stage, table_up + read_s(&table_text(&pic).map_err(fail)?))?;

        // 11_5: the question, the table standing; the countdown
        say(&kit, stage, "11_5")?;
        let q = think(&kit, stage, at(THINK_AT))?;

        // 11_6: a halo round each group's chance; "sufficient" on the panel
        // and card G, held until it can be read
        let line = say(&kit, stage, "11_6")?;
        fade_out_all(stage, &[q], 0.3)?;
        line.wait_for(stage, "their pattern is the same for every count")?;
        let last: Vec<Mob> = table.iter().filter_map(|r| r.last().copied()).collect();
        glow(stage, &last, Palette::ink())?;
        line.wait_for(stage, "sufficient")?;
        let sufficient = words_left(&kit, stage, SUFFICIENT, &[], panel(2))?;
        let text = card_g(&pic);
        let card = precisely(&kit, stage, &text)?;
        fade_in_all(stage, &[sufficient, card], 0.4)?;
        let card_up = now(stage);
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;
        // card G stays until it can be read
        wait_until(stage, card_up + read_s(&text))?;

        // 11_7: the table, the card and the panel's lower lines go; the four
        // boxes come back with the gaps below sixty; both handfuls' gaps
        let line = say(&kit, stage, "11_7")?;
        let mut gone = table.concat();
        gone.extend([card, groups, sufficient]);
        fade_out_all(stage, &gone, 0.5)?;
        line.wait_for(stage, "That's why the gap recipe")?;
        refill(stage, &lit, 0.4)?;
        let mut back = four;
        back.extend(&gl.arcs);
        back.extend(&gl.counts);
        fade_in_all(stage, &back, 0.6)?;
        line.wait_for(stage, "the gaps below sixty hold")?;
        let ours = words_left(
            &kit,
            stage,
            &gaps_text(&pic),
            &[(pic.unseen.to_string(), palette::unseen())],
            panel(1),
        )?;
        fade_in_all(stage, &[ours], 0.5)?;
        line.wait_for(stage, "the strange handful's put")?;
        let theirs = words_left(&kit, stage, &silly_text(&pic), &[], panel(2))?;
        fade_in_all(stage, &[theirs], 0.5)?;

        // 11_8: halos round the badge and the four smaller boxes
        let line = say(&kit, stage, "11_8")?;
        line.wait_for(stage, "needs the handful to be fair")?;
        glow(stage, &[corner], Palette::ink())?;
        line.wait_for(stage, "they can still warn you")?;
        let mut boxes = strip.bodies[..top_box].to_vec();
        boxes.extend(&strip.labels[..top_box]);
        glow(stage, &boxes, Palette::ink())?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, END_BEAT)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use explainer_kit::Script;
    use explainer_kit::animatic::wrap;
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;
    use crate::chapters::draw::{BOX_W, DOT_R, PAUSE_NOTE, SMALL, THINK_R, box_spot};
    use crate::chapters::friends_line::{BADGE, BADGE_PAD, BADGE_SIZE, PANEL_X, PANEL_Y, est_width};
    use crate::chapters::one_estimate::SPAN_TICK;
    use crate::chapters::what_we_assume::HALO_PAD;
    use crate::facts::{PAIR_MORE, SILLY, TINY};
    use crate::model::{BIG, FRIENDS, KNOWN, all_handfuls};

    /// The countdown's note, at the size `think` writes it.
    const NOTE_SIZE: f64 = 18.0;
    /// A line's half height at the panel's size and at the label size, and
    /// the space kept between things that must not touch.
    const HALF_LINE: f64 = 0.16;
    const HALF_SMALL: f64 = 0.12;
    const CLEAR: f64 = 0.1;
    /// A "precisely:" card's line, beside its label: the kit starts at 92
    /// characters and narrows by 4 until the widest line fits.
    const CARD_CHARS_WIDEST: usize = 92;
    const CARD_CHARS_NARROWEST: usize = 64;

    fn good() -> Picture {
        picture().unwrap()
    }

    /// The scene draws the facts, and its lines read as the voice says
    /// them: the friends' five, the companies, the counts and the strange
    /// handful are the model's, and every line the panel, the span, the
    /// captions, the table and card G show.
    #[test]
    fn the_picture_is_the_facts() {
        let pic = good();
        assert_eq!(pic.numbers, FRIENDS);
        assert_eq!(pic.silly, SILLY);
        assert_eq!(pic.companies, [TINY, PAIR_MORE[0], PAIR_MORE[1]]);
        assert_eq!(pic.counts, [60, KNOWN, BIG]);
        assert_eq!(others_of(TINY, TINY_SEEN, TINY), [[1], [2], [3], [4]]);
        assert_eq!(largest_text(pic.largest), "largest: 60");
        assert_eq!(others_text(&pic), "any 4 of 1 to 59");
        let words = [
            ["5 boxes: 10 pairs", "topped by 5: 4 of 10", "each 1 in 4"],
            ["6 boxes: 15 pairs", "topped by 5: 4 of 15", "each 1 in 4"],
            ["8 boxes: 28 pairs", "topped by 5: 4 of 28", "each 1 in 4"],
        ];
        assert_eq!(captions(&pic).unwrap(), words.map(|r| r.map(str::to_owned)));
        assert_eq!(groups_text(&pic), "455,126 groups of 4 below 60");
        assert_eq!(heads(&pic), ["count", "largest is 60", "each group of 4"]);
        let cells = [
            ["60", "1 in 12", "1 in 455,126"],
            ["71", "≈ 1 in 29", "1 in 455,126"],
            ["1,000", "≈ 1 in 18 million", "1 in 455,126"],
        ];
        assert_eq!(rows(&pic).unwrap(), cells.map(|r| r.map(str::to_owned)));
        assert_eq!(
            table_text(&pic).unwrap(),
            "count largest is 60 each group of 4 60 1 in 12 1 in 455,126 71 ≈ 1 in 29 1 in 455,126 1,000 ≈ 1 in 18 \
             million 1 in 455,126"
        );
        assert_eq!(gaps_text(&pic), "13 + 12 + 5 + 17 + 8 = 55 = 60 − 5");
        assert_eq!(silly_text(&pic), "2, 3, 4, 5: 1 + 0 + 0 + 0 + 54 = 55");
        assert_eq!(
            card_g(&pic),
            "fair handful from any N ≥ 60, given largest 60: each group of 4 below 60 is the others with chance \
             1/455,126. Only the chance of largest 60 depends on N: the largest is sufficient for N."
        );
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
                largest: 59,
                ..pic.clone()
            },
            "the friends' handful",
        );
        refused(
            Picture {
                others: 5,
                ..pic.clone()
            },
            "the friends' handful",
        );
        // a triangle whose column of five differs between companies, or
        // whose chances are not the column's
        refused(
            Picture {
                pair_top: 6,
                ..pic.clone()
            },
            "the pairs topped by",
        );
        refused(
            Picture {
                companies: vec![5, 6, 7],
                ..pic.clone()
            },
            "the pairs topped by",
        );
        refused(
            Picture {
                more_pairs: vec![15, 27],
                ..pic.clone()
            },
            "the pairs topped by",
        );
        refused(
            Picture {
                smaller: vec![1, 2, 3],
                ..pic.clone()
            },
            "the pairs topped by",
        );
        refused(
            Picture {
                each: Q::new(1, 10),
                ..pic.clone()
            },
            "the pairs topped by",
        );
        refused(
            Picture {
                shares: vec![Q::new(2, 5), Q::new(2, 5), Q::new(2, 5)],
                ..pic.clone()
            },
            "the pairs topped by",
        );
        refused(
            Picture {
                groups: pic.groups - 1,
                ..pic.clone()
            },
            "groups of",
        );
        refused(
            Picture {
                groups_over: 460_000,
                ..pic.clone()
            },
            "groups of",
        );
        // the edge count is the largest itself, never above it
        refused(
            Picture {
                counts: vec![61, KNOWN, BIG],
                ..pic.clone()
            },
            "the chance that",
        );
        let mut swapped = pic.chances.clone();
        swapped.swap(0, 1);
        refused(
            Picture {
                chances: swapped,
                ..pic.clone()
            },
            "the chance that",
        );
        // each group's chance must not change with the count
        let mut shifted = pic.per_group.clone();
        shifted[2] = Q::new(1, 455_125);
        refused(
            Picture {
                per_group: shifted,
                ..pic.clone()
            },
            "the chance that",
        );
        refused(
            Picture {
                one_in: vec![12, 28],
                ..pic.clone()
            },
            "the chance that",
        );
        refused(
            Picture {
                one_in: vec![13, 29],
                ..pic.clone()
            },
            "the chance that",
        );
        refused(
            Picture {
                millions: 19,
                ..pic.clone()
            },
            "the chance that",
        );
        refused(
            Picture {
                unseen: 54,
                ..pic.clone()
            },
            "the gaps below",
        );
        refused(
            Picture {
                silly_gaps: vec![1, 0, 0, 1, 53],
                ..pic.clone()
            },
            "the gaps below",
        );
        refused(
            Picture {
                silly_unseen: 54,
                ..pic
            },
            "the gaps below",
        );
    }

    /// Given the largest, the others never depend on the count. Listed
    /// whole on a smaller copy, largest 9 of four: every group of three
    /// below 9 once, the same at every count from the edge 9 to 16. And at
    /// the friends' scale from the counts by largest number: one group's
    /// chance given the largest 60 is 1/C(59, 4) at every count from the
    /// edge 60 to 300, while the chance of the largest 60 falls, from 5/60
    /// at the edge, and is zero below it. Whatever the group, the gaps
    /// below the largest hold it less the handful size.
    #[test]
    fn given_the_largest_the_others_never_depend_on_the_count() {
        let base = others_of(9, 4, 9);
        assert_eq!(base, all_handfuls(8, 3));
        for n in 9..=16 {
            assert_eq!(others_of(n, 4, 9), base, "n {n}");
        }
        for group in &base {
            let mut hand = group.clone();
            hand.push(9);
            assert_eq!(model::gaps(&hand).iter().sum::<u64>(), 9 - 4, "{hand:?}");
        }
        let pic = good();
        let k = pic.k as u64;
        let m = pic.largest;
        let groups = Q::count(choose(m - 1, k - 1));
        let mut before = Q::int(2);
        for n in m..=300 {
            let top = model::share_by_largest(n, k, |mm| mm == m);
            assert_eq!(Q::ONE / Q::count(choose(n, k)) / top, Q::ONE / groups, "n {n}");
            assert!(top < before, "n {n}");
            before = top;
        }
        assert_eq!(model::share_by_largest(m, k, |mm| mm == m), Q::new(5, 60));
        assert_eq!(model::share_by_largest(m - 1, k, |mm| mm == m), Q::ZERO);
    }

    /// Every handful of `k` from 1 to `n` drawn one box at a time, each box
    /// still in the company drawn with chance in proportion to `weight`, and
    /// the chance of each handful.
    fn drawn_in_turn(n: u64, k: usize, weight: fn(u64) -> i128) -> BTreeMap<Vec<u64>, Q> {
        fn draw(n: u64, k: usize, weight: fn(u64) -> i128, seq: &mut Vec<u64>, p: Q, out: &mut BTreeMap<Vec<u64>, Q>) {
            if seq.len() == k {
                let mut hand = seq.clone();
                hand.sort_unstable();
                let e = out.entry(hand).or_insert(Q::ZERO);
                *e = *e + p;
                return;
            }
            let left: i128 = (1..=n).filter(|x| !seq.contains(x)).map(weight).sum();
            for x in 1..=n {
                if !seq.contains(&x) {
                    seq.push(x);
                    draw(n, k, weight, seq, p * Q::new(weight(x), left), out);
                    seq.pop();
                }
            }
        }
        let mut out = BTreeMap::new();
        draw(n, k, weight, &mut Vec::new(), Q::ONE, &mut out);
        out
    }

    /// The others' chances among the handfuls of `drawn` topped by `top`.
    fn given_top(drawn: &BTreeMap<Vec<u64>, Q>, top: u64) -> BTreeMap<Vec<u64>, Q> {
        let topped: Vec<(Vec<u64>, Q)> = drawn
            .iter()
            .filter(|(h, _)| h.last() == Some(&top))
            .map(|(h, &p)| (h[..h.len() - 1].to_vec(), p))
            .collect();
        let total = topped.iter().fold(Q::ZERO, |s, &(_, p)| s + p);
        topped.into_iter().map(|(o, p)| (o, p / total)).collect()
    }

    /// The fairness 11_8 says it all needs, the negative control: drawn one
    /// box at a time with chance in proportion to its number, the others'
    /// chances given the largest are unequal, even at the edge count, and
    /// change with the count (handfuls of 2 topped by 4, of 3 topped by 6,
    /// counts up to three more); drawn the same way with equal chances, the
    /// draw gives every handful 1/C(n, k) and the others given the largest
    /// are every group below it, equally likely, at every count.
    #[test]
    fn unfair_draws_let_the_others_carry_a_clue() {
        let even: fn(u64) -> i128 = |_| 1;
        let bigger: fn(u64) -> i128 = |x| x as i128;
        for (k, top) in [(2usize, 4u64), (3, 6)] {
            let counts = top..=top + 3;
            let weighted: Vec<BTreeMap<Vec<u64>, Q>> = counts
                .clone()
                .map(|n| given_top(&drawn_in_turn(n, k, bigger), top))
                .collect();
            let first = &weighted[0];
            assert!(first.values().collect::<BTreeSet<_>>().len() > 1, "k {k}");
            assert!(weighted.iter().all(|g| g.keys().eq(first.keys())), "k {k}");
            assert!(weighted[1..].iter().all(|g| g != first), "k {k}");
            let each = Q::ONE / Q::count(choose(top - 1, k as u64 - 1));
            for n in counts {
                let fair = drawn_in_turn(n, k, even);
                assert_eq!(fair.len() as u128, choose(n, k as u64), "n {n}");
                assert!(fair.values().all(|&p| p == Q::ONE / Q::count(choose(n, k as u64))));
                let given = given_top(&fair, top);
                assert!(given.keys().eq(all_handfuls(top - 1, k - 1).iter()), "k {k}, n {n}");
                assert!(given.values().all(|&p| p == each), "k {k}, n {n}");
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
            ("11_1", &["all you were told", "any four different numbers below sixty"]),
            (
                "11_2",
                &[
                    "Try it on pairs",
                    "four of its ten pairs",
                    "the smaller one is one",
                    "so each is one time in four",
                ],
            ),
            (
                "11_3",
                &[
                    "six boxes, or eight",
                    "a larger number of five gets rarer",
                    "the pairs topped by five",
                    "each is still one time in four",
                ],
            ),
            (
                "11_4",
                &[
                    "It's the same with sixty",
                    "over four hundred and fifty thousand groups",
                    "Only the chance that sixty is the largest changes",
                    ROW_CUES[0],
                    ROW_CUES[1],
                    ROW_CUES[2],
                ],
            ),
            ("11_6", &["their pattern is the same for every count", "sufficient"]),
            (
                "11_7",
                &[
                    "That's why the gap recipe",
                    "the gaps below sixty hold",
                    "the strange handful's put",
                ],
            ),
            ("11_8", &["needs the handful to be fair", "they can still warn you"]),
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
        let asked = script.text("11_5").unwrap();
        for answer in ["othing", "sufficient"] {
            assert!(!asked.contains(answer), "{answer}");
        }
    }

    /// The panel, the span, the triangles, the table and the countdown keep
    /// apart: every panel line in the title-safe area, the badge left of
    /// it; the span under the dots and its words under it, in the frame;
    /// the triangles under the line, in the frame and apart, each caption
    /// line under them and over the frame's foot, the gold five over its
    /// column and clear of the next one, room for a halo round the column,
    /// the smaller numbers left of the first triangle; the table's columns
    /// apart, under the line and over card G's three lines; the countdown
    /// right of the box at sixty, under the panel and over the line.
    #[test]
    fn the_panel_the_pairs_and_the_table_fit_the_frame() {
        let pic = good();
        let safe_x = FRAME_X_RADIUS - 0.25;
        let end = LINE_DOTS as f64;
        let w = |s: &str| est_width(s, PANEL_SIZE);
        let ws = |s: &str| est_width(s, LABEL_SIZE);
        for text in [
            largest_text(pic.largest),
            groups_text(&pic),
            SUFFICIENT.to_owned(),
            gaps_text(&pic),
            silly_text(&pic),
        ] {
            assert!(PANEL_X + w(&text) < safe_x, "{text}");
        }
        let badge_right = BADGE_AT[0] + est_width(BADGE, BADGE_SIZE) / 2.0 + BADGE_PAD[0];
        assert!(badge_right < PANEL_X - CLEAR);
        // the span and its words (11_1)
        const { assert!(SPAN_Y + SPAN_TICK < STRIP_Y - DOT_R - CLEAR) };
        const { assert!(OTHERS_Y + HALF_LINE < SPAN_Y - CLEAR / 2.0) };
        let mid = (x_at(pic.first as f64, end) + x_at((pic.largest - 1) as f64, end)) / 2.0;
        assert!(mid - w(&others_text(&pic)) / 2.0 > -safe_x);
        // the triangles (11_2, 11_3)
        let words = captions(&pic).unwrap();
        let half = |g: usize| words[g].iter().map(|c| ws(c)).fold(0.0, f64::max) / 2.0;
        let top = pic.pair_top;
        for (g, &n) in pic.companies.iter().enumerate() {
            assert!(GRID_X[g] - half(g) > -safe_x, "{g}");
            if let Some(&next) = pic.companies.get(g + 1) {
                assert!(GRID_X[g] + half(g) + CLEAR < GRID_X[g + 1] - half(g + 1), "{g}");
                assert!(
                    grid_x(g, n, n) + PAIR_R + CLEAR < grid_x(g + 1, next, FIRST_LARGER) - PAIR_R,
                    "{g}"
                );
            }
            assert!(grid_y(n - 1) + PAIR_R + CLEAR < STRIP_Y - DOT_R, "{g}");
            let (x5, y5) = (grid_x(g, n, top), grid_y(top - 1) + TOP_DY);
            assert!(y5 - HALF_SMALL > grid_y(top - 1) + PAIR_R, "{g}");
            assert!(x5 - PAIR_R - HALO_PAD > grid_x(g, n, top - 1) + PAIR_R, "{g}");
            if n > top {
                let next_x = grid_x(g, n, top + 1);
                assert!(x5 + PAIR_R + HALO_PAD < next_x - PAIR_R, "{g}");
                if grid_y(top) + PAIR_R > y5 - HALF_SMALL {
                    assert!(x5 + ws(&top.to_string()) / 2.0 + CLEAR / 2.0 < next_x - PAIR_R, "{g}");
                }
            }
        }
        let last = pic.companies.len() - 1;
        assert!(grid_x(last, pic.companies[last], pic.companies[last]) + PAIR_R < safe_x);
        const { assert!(CAPTION_Y[0] + HALF_SMALL < GRID_FOOT - PAIR_R - CLEAR / 2.0) };
        const { assert!(CAPTION_Y[2] - HALF_SMALL > -FRAME_Y_RADIUS + 0.25) };
        for p in CAPTION_Y.windows(2) {
            assert!(p[0] - HALF_SMALL > p[1] + HALF_SMALL);
        }
        let label_x = grid_x(0, pic.tiny, FIRST_LARGER) - ROW_LABEL_GAP;
        let label_half = pic.smaller.iter().map(|s| ws(&s.to_string())).fold(0.0, f64::max) / 2.0;
        assert!(label_x - label_half > -safe_x);
        assert!(label_x + label_half + CLEAR / 2.0 < grid_x(0, pic.tiny, FIRST_LARGER) - PAIR_R);
        // the table (11_4), under the line and over card G (0.3 plus 0.36
        // a line, 0.2 over the frame's foot)
        let mut cells = vec![heads(&pic).to_vec()];
        cells.extend(rows(&pic).unwrap().into_iter().map(|r| r.to_vec()));
        let widths: Vec<f64> = (0..TABLE_X.len())
            .map(|c| cells.iter().map(|r| ws(&r[c])).fold(0.0, f64::max))
            .collect();
        assert!(TABLE_X[0] > -safe_x);
        for (c, p) in TABLE_X.windows(2).enumerate() {
            assert!(p[0] + widths[c] + CLEAR < p[1], "{c}");
        }
        assert!(TABLE_X[2] + widths[2] < safe_x);
        const { assert!(TABLE_Y0 + HALF_SMALL < STRIP_Y - DOT_R - CLEAR) };
        const { assert!(TABLE_DY > 2.0 * HALF_SMALL) };
        let card_top = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 3.0;
        assert!(table_y(cells.len() - 1) - HALF_SMALL > card_top + CLEAR);
        // the countdown (11_5)
        let note_y = THINK_AT[1] - THINK_R - 0.3;
        let note_half = est_width(PAUSE_NOTE, NOTE_SIZE) / 2.0;
        let top_box = pic.numbers.len() - 1;
        let box_right = box_spot(top_box, pic.numbers[top_box], end, 0.0)[0] + BOX_W * SMALL / 2.0;
        assert!(box_right + CLEAR < THINK_AT[0] - note_half.max(THINK_R));
        const { assert!(THINK_AT[1] + THINK_R < PANEL_Y[1] - HALF_LINE - CLEAR) };
        assert!(note_y - 0.1 > STRIP_Y + DOT_R + CLEAR);
    }

    /// Card G says sufficiency exactly, the fairness first, in at most three
    /// lines at every width the lower third might narrow to, and is up long
    /// enough to be read.
    #[test]
    fn card_g_fits_and_is_read() {
        let text = card_g(&good());
        for chars in (CARD_CHARS_NARROWEST..=CARD_CHARS_WIDEST).step_by(4) {
            assert!(wrap(&text, chars, 4).len() <= 3, "{chars}");
        }
        assert!(text.starts_with("fair handful"));
        assert!(read_s(&text) < 15.0);
        // the negative control: one clause more would not fit
        let longer = format!("{text} The count is one number.");
        assert!(wrap(&longer, CARD_CHARS_NARROWEST, 4).len() > 3);
    }
}
