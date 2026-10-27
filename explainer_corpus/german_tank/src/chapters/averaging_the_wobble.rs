//! Chapter 12, "Averaging away the wobble": the tiny company's ten pairs
//! again, now with the doubling recipe's estimates, for a pair the total
//! less one (12_1); they average five but wobble from two to eight (12_2),
//! because doubling listens to the smaller number, which carries no news
//! once the larger is known (12_3). The rows move, each pair whole, into four
//! groups by their larger number (12_4); the prediction: what do the biggest
//! group's four estimates average (12_5)? Six and a half, the gap recipe's
//! estimate, and so for every group (12_6, 12_7), the same in a company of
//! six boxes or eight because a fair draw makes the smaller numbers equally
//! likely whatever the count (12_8). Weighted by their chances the groups
//! give five again, equally weighted four and a quarter (12_9, 12_10). The
//! misses as arrows from the count: cancelling in the group topped by four,
//! all one way in the group topped by five (12_11, 12_12); the average
//! miss falls from 1.4 to 1.2 (12_13), by a rule (12_14), and the squared
//! miss from 3 to 2.25, by exactly the wobble inside the groups (12_15);
//! card H says it for any recipe (12_16).
//!
//! The pairs are listed whole (small cases only). The scene computes every
//! estimate, group, average, weight, miss and width from the pairs and
//! refuses a picture the facts contradict. The rows move as they are, so a
//! pair keeps its squares, estimate and dot; no stroke is on the screen
//! while they move. Highlights are halos and colour changes are
//! replacements: no typeset text is indicated, recoloured or scaled in
//! place.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, choose, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    BODY_O, LABEL_SIZE, fade_in_all, fade_in_lagged, fade_out_all, fail, flat, line_style, move_all, pause, say,
    stroke, think, wait_until,
};
use super::friends_line::{
    ARROW_HEAD, ARROW_SPREAD, ARROW_W, PANEL_SIZE, THINK_AT, at, badge, handfuls, list, now, printed, read_s, whole,
};
use super::gaps_trade_places::glow_each;
use super::one_estimate::{approx, fraction};
use super::what_we_assume::{formula, formula_colors, glow, sum_texts, words_left};
use crate::facts::{self, TINY, TINY_SEEN};
use crate::model::{self, Q};
use crate::palette;

pub struct AveragingTheWobble {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts); chapter 13
// opens on the line that says what averaging did, and heads its table with
// the gap recipe's name
const DOUBLING_HEAD: &str = "doubling";
const AVERAGED_HEAD: &str = "averaged";
pub const GAP_HEAD: &str = "gap recipe";
pub const AVERAGED_IS: &str = "averaged doubling = gap recipe";
const ROWS: [&str; 3] = ["average miss", "average squared miss", "standard deviation"];
const RULE: [&str; 2] = ["an average's arrow is never longer", "than the arrows' average length"];
const WOBBLE: &str = "wobble inside groups";
/// Card H (12_16): the general statement, after the ten pairs have shown
/// it, the fairness first.
const CARD_H: &str = "fair handful, any recipe: averaged over each largest M, it becomes a recipe of M alone, \
                      free of N. At every N its average stays, and its average miss and squared miss never grow.";

/// The groups' tops in the order 12_6 names their averages: the biggest
/// group first (the prediction's), the lone pair last.
const SPOKEN_TOPS: [u64; 4] = [5, 3, 4, 2];

/// The tiny table (12_1): row `j`'s centre in table order, the squares'
/// side, step and first centre, the column numbers `HEAD_DY` over the first
/// row; the doubling estimates' column and the averaged one beside it; each
/// row's plot, value 1 at `PLOT_X0` and `PLOT_STEP` per unit, its purple
/// dot `DOT_DY` over the row's centre and its green dot as far under it.
const TABLE_TOP: f64 = 2.9;
const ROW_STEP: f64 = 0.45;
const GROUP_GAP: f64 = 0.25;
const SQ: f64 = 0.28;
const SQ_STEP: f64 = 0.4;
const SQ_X0: f64 = -6.4;
const SQ_R: f64 = 0.04;
const SEEN_O: f64 = 0.9;
const HEAD_DY: f64 = 0.4;
const EST_X: [f64; 2] = [-4.15, -3.15];
const EST_SIZE: f64 = 24.0;
const PLOT_X0: f64 = -2.75;
const PLOT_STEP: f64 = 0.28;
const PLOT_R: f64 = 0.05;
const DOT_DY: f64 = 0.09;
/// The groups' labels (12_4), their left edge right of the plots; the
/// chances and the misses (12_9 to 12_12) from the panel's left edge at
/// each group's middle, a group's two miss lines `MISS_DY` over and under
/// it.
const GROUP_X: f64 = -0.55;
const RIGHT_X: f64 = 0.9;
const MISS_DY: f64 = 0.225;
/// The panel's lines on the right, under the badge at the top right.
const RIGHT_Y: [f64; 7] = [2.6, 2.15, 1.7, 1.25, 0.8, 0.35, -0.1];
const BADGE_SPOT: [f64; 2] = [5.3, 3.5];
/// The weighted and the equal sums (12_10), under the groups on the right.
const SUM_Y: [f64; 2] = [-2.65, -3.15];
/// The count line (12_11), from `COUNT_PAD` over the first row to as far
/// under the last.
const COUNT_PAD: f64 = 0.2;
const COUNT_W: f64 = 2.0;
const COUNT_O: f64 = 0.7;
/// The widths table (12_13 to 12_15): each recipe's column centre.
const TABLE_X: [f64; 2] = [4.5, 5.95];
/// The regroup, the inspection holds (12_15, 12_16), the beat after the
/// last line.
const SORT_S: f64 = 1.6;
const HOLD_S: f64 = 2.5;
const END_BEAT: f64 = 0.6;

/// What chapter 12 shows, from the facts: the tiny company, how many boxes
/// a pair sees, its pairs in table order; the doubling recipe and its rule
/// for a pair, each pair's doubling estimate, their average and range; the
/// groups' tops and sizes, each group's doubling values, their averages and
/// how the table shows them; the larger number of the biggest group and the
/// bigger companies tried; the biggest group's chance, the weighted and the
/// equal average and how the panel shows the equal one; the group whose
/// misses cancel and its misses, the group whose misses are one-sided, its
/// misses and their average; the two average misses as printed, the two
/// average squared misses, the second as printed, the standard deviations
/// (the first about, the second exact and as printed) and the wobble inside
/// the groups.
#[derive(Clone, Debug)]
struct Picture {
    boxes: u64,
    seen: usize,
    pairs: Vec<Vec<u64>>,
    recipe: (Q, Q),
    pair_rule: (Q, Q),
    estimates: Vec<Q>,
    average: Q,
    range: Vec<u64>,
    tops: Vec<u64>,
    sizes: Vec<u64>,
    values: Vec<Vec<Q>>,
    averages: Vec<Q>,
    shown: Vec<String>,
    pair_top: u64,
    more: Vec<u64>,
    top_chance: Q,
    weighted: Q,
    equal: Q,
    equal_shown: String,
    cancel: u64,
    cancel_misses: Vec<Q>,
    same_side: u64,
    same_misses: Vec<Q>,
    same_average: Q,
    misses: [String; 2],
    squared: [Q; 2],
    squared_shown: String,
    sd_about: String,
    sd: Q,
    sd_shown: String,
    spread: Q,
}

fn picture() -> Result<Picture, String> {
    let f = facts::facts()?;
    let q = |key: &str| -> Result<Q, String> { fraction(&printed(&f, key)?) };
    let qs = |key: &str| -> Result<Vec<Q>, String> { printed(&f, key)?.split(',').map(fraction).collect() };
    let tops = list(&f, "G-F12.group_tops")?;
    checked(Picture {
        boxes: whole(&f, "G-F5.boxes")?,
        seen: whole(&f, "G-F5.seen")? as usize,
        pairs: handfuls(&f, "G-F5.pair_list")?,
        recipe: linear(&printed(&f, "G-F10.recipe")?, "average")?,
        pair_rule: linear(&printed(&f, "G-F12.pair_doubling")?, "total")?,
        estimates: qs("G-F12.doubling_estimates")?,
        average: q("G-F12.doubling_average")?,
        range: list(&f, "G-F12.doubling_range")?,
        sizes: list(&f, "G-F12.group_sizes")?,
        values: tops
            .iter()
            .map(|m| qs(&format!("G-F12.ends_in_{m}")))
            .collect::<Result<_, _>>()?,
        averages: qs("G-F12.group_averages")?,
        shown: printed(&f, "G-F12.group_averages_shown")?
            .split(',')
            .map(str::to_owned)
            .collect(),
        tops,
        pair_top: whole(&f, "G-F11.pair_top")?,
        more: list(&f, "G-F11.pair_more")?,
        top_chance: q("G-F12.top_group_chance")?,
        weighted: q("G-F12.weighted_average")?,
        equal: q("G-F12.equal_weight_average")?,
        equal_shown: printed(&f, "G-F12.equal_weight_average_shown")?,
        cancel: whole(&f, "G-F12.cancel_group")?,
        cancel_misses: qs("G-F12.cancel_misses")?,
        same_side: whole(&f, "G-F12.same_side_group")?,
        same_misses: qs("G-F12.same_side_misses")?,
        same_average: q("G-F12.same_side_average_miss")?,
        misses: [printed(&f, "G-F12.doubling_miss")?, printed(&f, "G-F12.grouped_miss")?],
        squared: [q("G-F12.doubling_squared_miss")?, q("G-F12.grouped_squared_miss")?],
        squared_shown: printed(&f, "G-F12.grouped_squared_miss_shown")?,
        sd_about: printed(&f, "G-F12.doubling_sd_about")?,
        sd: q("G-F12.grouped_sd")?,
        sd_shown: printed(&f, "G-F12.grouped_sd_decimal")?,
        spread: q("G-F12.spread_inside")?,
    })
}

/// A rule's multiplier and added number as the facts print it:
/// "2*average-1" for `var` "average" is (2, −1).
pub fn linear(s: &str, var: &str) -> Result<(Q, Q), String> {
    let bad = || format!("{s:?} is not a multiple of {var} plus a number");
    let (a, b) = s.split_once(&format!("*{var}")).ok_or_else(bad)?;
    Ok((fraction(a).map_err(|_| bad())?, fraction(b).map_err(|_| bad())?))
}

/// A rule as the panel writes it: "2 × average − 1", the multiplier left
/// out when it is one ("total − 1").
pub fn rule_text(var: &str, (a, b): (Q, Q)) -> String {
    let times = if a == Q::ONE {
        var.to_owned()
    } else {
        format!("{a} × {var}")
    };
    if b < Q::ZERO {
        format!("{times} − {}", -b)
    } else {
        format!("{times} + {b}")
    }
}

/// How a table shows an exact value: a whole number as it is, any other
/// with its one decimal ("3.5") when that is exact.
pub fn shown(q: Q) -> Result<String, String> {
    match q.whole() {
        Some(w) => Ok(w.to_string()),
        None => q.decimal1(),
    }
}

/// The average of `values` (of none, zero).
pub fn mean(values: &[Q]) -> Q {
    if values.is_empty() {
        return Q::ZERO;
    }
    values.iter().fold(Q::ZERO, |s, &x| s + x) / Q::int(values.len() as i128)
}

/// Each group's rows in table order, a group per top.
fn groups_of(pic: &Picture) -> Vec<Vec<usize>> {
    let k = pic.seen;
    pic.tops
        .iter()
        .map(|&m| (0..pic.pairs.len()).filter(|&j| pic.pairs[j][k - 1] == m).collect())
        .collect()
}

/// Each pair's averaged estimate: its group's average, in table order.
fn grouped(pic: &Picture, groups: &[Vec<usize>]) -> Vec<Q> {
    let mut out = vec![Q::ZERO; pic.pairs.len()];
    for (members, &avg) in groups.iter().zip(&pic.averages) {
        for &j in members {
            out[j] = avg;
        }
    }
    out
}

/// Where group `top` is among the groups.
fn group_index(pic: &Picture, top: u64) -> Option<usize> {
    pic.tops.iter().position(|&t| t == top)
}

/// The picture only if the scene can draw it truthfully: the tiny company's
/// pairs are every pair, in table order; doubling for a pair is the total
/// less one, the estimates are doubling's, whole, averaging the count, from
/// the smallest to the largest; the groups are topped by every number from
/// the pair size to the count, each as big as the pairs it tops, in growing
/// size, their values the doubling estimates of their pairs in table order,
/// their averages the gap recipe's, shown exactly, and the biggest group's
/// values the same in every bigger company tried; the chances are the
/// groups' sizes in the number of pairs, weighting the averages back to
/// the count, while equal weights give another number, shown exactly; the
/// misses cancel in one group, pointing both ways, and point one way in
/// another, their average their average length; the averaged estimates are
/// the gap recipe's, their average miss smaller, their average squared miss
/// smaller by the wobble inside the groups, the doubling standard deviation
/// not exact and the averaged one exact.
fn checked(pic: Picture) -> Result<Picture, String> {
    let k = pic.seen;
    let n = Q::int(pic.boxes as i128);
    let (a, b) = pic.recipe;
    let average_of = |p: &[u64]| Q::new(p.iter().sum::<u64>() as i128, p.len() as i128);
    let total_of = |p: &[u64]| Q::int(p.iter().sum::<u64>() as i128);
    let doubled: Vec<Q> = pic.pairs.iter().map(|p| model::doubling_recipe(p)).collect();
    let whole_of = |q: Option<&Q>| q.and_then(|x| x.whole()).and_then(|w| u64::try_from(w).ok());
    if pic.boxes != TINY
        || k != TINY_SEEN
        || pic.pairs != model::all_handfuls(pic.boxes, k)
        || pic.pairs.len() as u128 != choose(pic.boxes, k as u64)
        || pic.pairs.iter().any(|p| {
            a * average_of(p) + b != model::doubling_recipe(p)
                || pic.pair_rule.0 * total_of(p) + pic.pair_rule.1 != model::doubling_recipe(p)
        })
        || pic.estimates != doubled
        || pic.estimates.iter().any(|e| e.whole().is_none())
        || mean(&pic.estimates) != pic.average
        || pic.average != n
        || pic.range
            != [
                whole_of(pic.estimates.iter().min()),
                whole_of(pic.estimates.iter().max()),
            ]
            .into_iter()
            .collect::<Option<Vec<u64>>>()
            .unwrap_or_default()
    {
        return Err(format!(
            "the doubling estimates of the {} pairs must be {:?}, averaging {}, from {:?}",
            pic.pairs.len(),
            doubled,
            pic.average,
            pic.range
        ));
    }
    let groups = groups_of(&pic);
    let biggest = pic.tops.len().saturating_sub(1);
    let topped_in = |c: u64| -> Vec<Q> {
        model::all_handfuls(c, k)
            .iter()
            .filter(|p| p[k - 1] == pic.pair_top)
            .map(|p| model::doubling_recipe(p))
            .collect()
    };
    let mut spoken = SPOKEN_TOPS.to_vec();
    spoken.sort_unstable();
    if pic.tops != (k as u64..=pic.boxes).collect::<Vec<_>>()
        || spoken != pic.tops
        || group_index(&pic, SPOKEN_TOPS[0]) != Some(biggest)
        || group_index(&pic, SPOKEN_TOPS[SPOKEN_TOPS.len() - 1]).map(|g| pic.sizes[g]) != Some(1)
        || [pic.sizes.len(), pic.values.len(), pic.averages.len(), pic.shown.len()] != [pic.tops.len(); 4]
        || groups.iter().zip(&pic.sizes).any(|(r, &s)| r.len() as u64 != s)
        || pic
            .tops
            .iter()
            .zip(&pic.sizes)
            .any(|(&m, &s)| model::with_largest(m, k as u64) != Q::int(s as i128))
        || pic.sizes.windows(2).any(|w| w[1] <= w[0])
        || groups
            .iter()
            .zip(&pic.values)
            .any(|(r, vals)| r.iter().map(|&j| pic.estimates[j]).collect::<Vec<_>>() != *vals)
        || pic
            .values
            .iter()
            .zip(&pic.averages)
            .any(|(vals, &avg)| mean(vals) != avg)
        || pic
            .tops
            .iter()
            .zip(&pic.averages)
            .any(|(&m, &avg)| model::gap_of(m, k as u64) != avg)
        || pic
            .averages
            .iter()
            .zip(&pic.shown)
            .any(|(&avg, s)| shown(avg).as_deref() != Ok(s.as_str()))
        || pic.tops.get(biggest) != Some(&pic.pair_top)
        || pic.more.is_empty()
        || pic.more.len() + 3 > RIGHT_Y.len()
        || pic
            .more
            .iter()
            .any(|&c| c <= pic.boxes || topped_in(c) != pic.values[biggest])
    {
        return Err(format!(
            "the groups topped by {:?} must hold {:?} pairs averaging the gap recipe's {:?}, the same in companies {:?}",
            pic.tops, pic.sizes, pic.averages, pic.more
        ));
    }
    let pairs = Q::int(pic.pairs.len() as i128);
    let chances: Vec<Q> = pic.sizes.iter().map(|&s| Q::int(s as i128) / pairs).collect();
    let weighted = chances
        .iter()
        .zip(&pic.averages)
        .fold(Q::ZERO, |s, (&c, &avg)| s + c * avg);
    if chances.get(biggest) != Some(&pic.top_chance)
        || weighted != pic.weighted
        || pic.weighted != pic.average
        || mean(&pic.averages) != pic.equal
        || pic.equal == pic.weighted
        || pic.equal.decimal2().as_deref() != Ok(pic.equal_shown.as_str())
        || chances
            .iter()
            .zip(&pic.averages)
            .any(|(&c, &avg)| (c * avg).decimal1().is_err())
    {
        return Err(format!(
            "the groups' chances {chances:?} must weight their averages back to {}, not {} as equal weights do",
            pic.average, pic.equal
        ));
    }
    let misses_of = |m: u64| group_index(&pic, m).map(|g| pic.values[g].iter().map(|&e| e - n).collect::<Vec<_>>());
    let lengths = |xs: &[Q]| mean(&xs.iter().map(|x| x.abs()).collect::<Vec<_>>());
    if misses_of(pic.cancel).as_ref() != Some(&pic.cancel_misses)
        || mean(&pic.cancel_misses) != Q::ZERO
        || !pic.cancel_misses.iter().any(|&x| x < Q::ZERO)
        || !pic.cancel_misses.iter().any(|&x| x > Q::ZERO)
        || misses_of(pic.same_side).as_ref() != Some(&pic.same_misses)
        || pic.same_misses.iter().any(|&x| x < Q::ZERO)
        || mean(&pic.same_misses) != pic.same_average
        || lengths(&pic.same_misses) != pic.same_average
        || pic.same_side == pic.cancel
        || pic.same_misses.len() < 2
    {
        return Err(format!(
            "the misses of the group topped by {} must cancel, {:?}, and those topped by {} point one way, {:?}",
            pic.cancel, pic.cancel_misses, pic.same_side, pic.same_misses
        ));
    }
    let averaged = grouped(&pic, &groups);
    let off = |xs: &[Q], f: fn(Q) -> Q| mean(&xs.iter().map(|&e| f(e - n)).collect::<Vec<_>>());
    let plain: fn(Q) -> Q = Q::abs;
    let square: fn(Q) -> Q = |x| x * x;
    let wobble = mean(
        &pic.estimates
            .iter()
            .zip(&averaged)
            .map(|(&e, &g)| (e - g) * (e - g))
            .collect::<Vec<_>>(),
    );
    if pic.pairs.iter().zip(&averaged).any(|(p, &g)| model::gap_recipe(p) != g)
        || off(&pic.estimates, plain).decimal1().as_deref() != Ok(pic.misses[0].as_str())
        || off(&averaged, plain).decimal1().as_deref() != Ok(pic.misses[1].as_str())
        || off(&averaged, plain) >= off(&pic.estimates, plain)
        || [off(&pic.estimates, square), off(&averaged, square)] != pic.squared
        || pic.squared[1].decimal2().as_deref() != Ok(pic.squared_shown.as_str())
        || pic.squared[1].exact_sqrt() != Some(pic.sd)
        || pic.sd.decimal1().as_deref() != Ok(pic.sd_shown.as_str())
        || pic.squared[0].exact_sqrt().is_some()
        || pic.squared[0].sqrt_about1().as_deref() != Ok(pic.sd_about.as_str())
        || wobble != pic.spread
        || pic.squared[0] - pic.squared[1] != pic.spread
        || pic.spread.decimal2().is_err()
    {
        return Err(format!(
            "the widths must be {:?} plain and {:?} squared, the drop {} the wobble inside the groups",
            pic.misses, pic.squared, pic.spread
        ));
    }
    Ok(pic)
}

// ------------------------------------------------------------------ words

/// The recipe on the panel (12_1), its name purple.
fn recipe_text(pic: &Picture) -> String {
    format!("{DOUBLING_HEAD}: {}", rule_text("average", pic.recipe))
}

/// What averaging did (12_7, 13_1), each recipe's name in its colour.
pub fn averaged_is_colors() -> Vec<(String, Srgb)> {
    vec![
        (DOUBLING_HEAD.to_owned(), palette::doubling()),
        (GAP_HEAD.to_owned(), palette::estimate()),
    ]
}

/// The same recipe for a pair (12_1).
fn pair_text(pic: &Picture) -> String {
    format!("for a pair: {}", rule_text("total", pic.pair_rule))
}

fn average_text(pic: &Picture) -> String {
    format!("average: {}", pic.average)
}

fn group_label(m: u64) -> String {
    format!("topped by {m}")
}

/// A miss with its sign ("−1", "0", "+1").
fn signed(x: Q) -> String {
    if x > Q::ZERO {
        format!("+{x}")
    } else if x < Q::ZERO {
        format!("−{}", -x)
    } else {
        x.to_string()
    }
}

/// Values added up as the panel writes them: "−1 + 0 + 1".
fn sum_of(values: &[Q]) -> String {
    let mut out = String::new();
    for (i, &x) in values.iter().enumerate() {
        if i == 0 {
            out.push_str(&signed(x).replacen('+', "", 1));
        } else if x < Q::ZERO {
            out.push_str(&format!(" − {}", -x));
        } else {
            out.push_str(&format!(" + {x}"));
        }
    }
    out
}

/// An average worked out, its left side and its result: "(5 + 6 + 7 + 8) ÷
/// 4 =" and "6.5".
fn mean_pieces(values: &[Q], result: &str) -> [String; 2] {
    [format!("({}) ÷ {} =", sum_of(values), values.len()), result.to_owned()]
}

/// The biggest group's values in a bigger company (12_8).
fn company_text(pic: &Picture, c: u64) -> String {
    let biggest = pic.tops.len() - 1;
    let values: Vec<String> = pic.values[biggest].iter().map(Q::to_string).collect();
    format!("{c} boxes: {} average {}", values.join(", "), pic.shown[biggest])
}

/// Group `g`'s chance line (12_9, 12_10), piece by piece: its chance, "×",
/// its average, "=" and their product.
fn chance_pieces(pic: &Picture, g: usize) -> Result<[String; 5], String> {
    let pairs = pic.pairs.len() as i128;
    let product = Q::new(pic.sizes[g] as i128, pairs) * pic.averages[g];
    Ok([
        format!("{} in {pairs}", pic.sizes[g]),
        "×".to_owned(),
        pic.shown[g].clone(),
        "=".to_owned(),
        product.decimal1()?,
    ])
}

/// The weighted sum (12_10): the groups' products, adding up to the count.
fn weighted_text(pic: &Picture) -> Result<String, String> {
    let products = (0..pic.tops.len())
        .map(|g| chance_pieces(pic, g).map(|p| p[4].clone()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(format!("{} = {}", products.join(" + "), pic.weighted))
}

/// The equally weighted average (12_10).
fn equal_text(pic: &Picture) -> String {
    format!(
        "({}) ÷ {} = {}",
        pic.shown.join(" + "),
        pic.shown.len(),
        pic.equal_shown
    )
}

/// A group's misses as signed numbers (12_11, 12_12).
fn misses_text(misses: &[Q]) -> String {
    misses.iter().map(|&x| signed(x)).collect::<Vec<_>>().join(", ")
}

/// The widths table's values (12_13 to 12_15): the average miss, the
/// average squared miss and the standard deviation, doubling's then the
/// averaged recipe's.
fn table_cells(pic: &Picture) -> Result<[[String; 2]; 3], String> {
    Ok([
        pic.misses.clone(),
        [shown(pic.squared[0])?, pic.squared_shown.clone()],
        [format!("≈ {}", pic.sd_about), pic.sd_shown.clone()],
    ])
}

/// The squares' drop and what it is (12_15).
fn wobble_text(pic: &Picture) -> Result<String, String> {
    Ok(format!(
        "{} − {} = {}: {WOBBLE}",
        shown(pic.squared[0])?,
        pic.squared_shown,
        pic.spread.decimal2()?
    ))
}

/// Every word of the widths table and the line under it, for its reading
/// time.
fn table_text(pic: &Picture) -> Result<String, String> {
    let mut words = vec![DOUBLING_HEAD.to_owned(), GAP_HEAD.to_owned()];
    for (label, row) in ROWS.iter().zip(table_cells(pic)?) {
        words.push((*label).to_owned());
        words.extend(row);
    }
    words.push(wobble_text(pic)?);
    Ok(words.join(" "))
}

// ------------------------------------------------------------ positions

/// Row `j`'s centre in table order.
fn table_y(j: usize) -> f64 {
    TABLE_TOP - ROW_STEP * j as f64
}

/// Each row's centre once the rows are sorted into their groups (12_4): the
/// groups in order of their top, each group's rows in table order, a gap
/// between groups.
fn sorted_y(groups: &[Vec<usize>], rows: usize) -> Vec<f64> {
    let mut y = vec![0.0; rows];
    let mut order = 0;
    for (g, members) in groups.iter().enumerate() {
        for &j in members {
            y[j] = TABLE_TOP - ROW_STEP * order as f64 - GROUP_GAP * g as f64;
            order += 1;
        }
    }
    y
}

/// The middle of a sorted group.
fn middle_y(ys: &[f64], members: &[usize]) -> f64 {
    match (members.first(), members.last()) {
        (Some(&a), Some(&b)) => (ys[a] + ys[b]) / 2.0,
        _ => TABLE_TOP,
    }
}

/// Column `n`'s centre (box `n` of the tiny company).
fn sq_x(n: u64) -> f64 {
    SQ_X0 + SQ_STEP * (n as f64 - 1.0)
}

/// Where value `e` sits on a row's plot.
fn plot_x(e: f64) -> f64 {
    PLOT_X0 + PLOT_STEP * (e - 1.0)
}

fn right(j: usize) -> Vec3 {
    v(RIGHT_X, RIGHT_Y[j])
}

/// A miss's arrow at height `y` from the count's place `from` to `to`, its
/// head at `to`, stopping `PLOT_R` short of the dot there; none when that
/// leaves nothing to draw.
fn arrow_points(from: f64, to: f64, y: f64) -> Option<Vec<Vec3>> {
    let dir = if to < from { -1.0 } else { 1.0 };
    let end = to - dir * PLOT_R;
    let len = (end - from) * dir;
    if len <= 0.0 {
        return None;
    }
    let h = ARROW_HEAD.min(len / 2.0);
    Some(vec![
        v(from, y),
        v(end, y),
        v(end - dir * h, y + ARROW_SPREAD * h),
        v(end, y),
        v(end - dir * h, y - ARROW_SPREAD * h),
    ])
}

// ------------------------------------------------------------- builders

/// One pair's row: its five squares (the pair's two gold), its purple
/// estimate and its purple dot.
struct Row {
    squares: Vec<Mob>,
    estimate: Mob,
    dot: Mob,
}

impl Row {
    /// The squares and the estimate, for a halo.
    fn marks(&self) -> Vec<Mob> {
        let mut m = self.squares.clone();
        m.push(self.estimate);
        m
    }
}

/// The tiny table (12_1), each row at its place in table order: the column
/// numbers, the purple head and each pair's row. Detached.
struct Table {
    head: Vec<Mob>,
    doubling_head: Mob,
    rows: Vec<Row>,
}

impl Table {
    fn build(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture) -> fmn::Result<Table> {
        let head_y = TABLE_TOP + HEAD_DY;
        let mut head = Vec::new();
        for n in 1..=pic.boxes {
            head.push(kit.text(stage, &n.to_string(), LABEL_SIZE, Palette::dim(), v(sq_x(n), head_y))?);
        }
        let doubling_head = kit.text(
            stage,
            DOUBLING_HEAD,
            LABEL_SIZE,
            palette::doubling(),
            v(EST_X[0], head_y),
        )?;
        let mut rows = Vec::new();
        for (j, (pair, &e)) in pic.pairs.iter().zip(&pic.estimates).enumerate() {
            let y = table_y(j);
            let mut squares = Vec::new();
            for n in 1..=pic.boxes {
                let style = if pair.contains(&n) {
                    flat(palette::seen(), SEEN_O)
                } else {
                    flat(palette::outline(), BODY_O)
                };
                let sq = Rectangle::new()
                    .width(SQ)
                    .height(SQ)
                    .corner_radius(SQ_R)
                    .style(style)
                    .build()?
                    .moved_to(v(sq_x(n), y));
                squares.push(stage.arena_mut().add(sq));
            }
            let estimate = kit.text(stage, &e.to_string(), EST_SIZE, palette::doubling(), v(EST_X[0], y))?;
            let d = Dot::new()
                .point(v(plot_x(approx(e).map_err(fail)?), y + DOT_DY))
                .radius(PLOT_R)
                .style(flat(palette::doubling(), 1.0))
                .build();
            let dot = stage.arena_mut().add(d);
            rows.push(Row { squares, estimate, dot });
        }
        Ok(Table {
            head,
            doubling_head,
            rows,
        })
    }

    /// Where every mobject of row `j` goes when the row moves to height
    /// `y`, unchanged otherwise.
    fn moves_to(&self, pic: &Picture, j: usize, y: f64) -> fmn::Result<Vec<(Mob, Vec3)>> {
        let row = &self.rows[j];
        let mut moves: Vec<(Mob, Vec3)> = row
            .squares
            .iter()
            .zip(1..=pic.boxes)
            .map(|(&sq, n)| (sq, v(sq_x(n), y)))
            .collect();
        moves.push((row.estimate, v(EST_X[0], y)));
        let x = plot_x(approx(pic.estimates[j]).map_err(fail)?);
        moves.push((row.dot, v(x, y + DOT_DY)));
        Ok(moves)
    }
}

/// The averaged column (12_6): each pair's group average in green beside
/// its purple estimate and a green dot under its purple one, at the row's
/// sorted height, and the green head. Detached.
struct Averaged {
    head: Mob,
    values: Vec<Mob>,
    dots: Vec<Mob>,
}

impl Averaged {
    fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        pic: &Picture,
        groups: &[Vec<usize>],
        ys: &[f64],
    ) -> fmn::Result<Averaged> {
        let head = kit.text(
            stage,
            AVERAGED_HEAD,
            LABEL_SIZE,
            palette::estimate(),
            v(EST_X[1], TABLE_TOP + HEAD_DY),
        )?;
        let mut values = Vec::new();
        let mut dots = Vec::new();
        for (&g, &y) in grouped(pic, groups).iter().zip(ys) {
            values.push(kit.text(
                stage,
                &shown(g).map_err(fail)?,
                EST_SIZE,
                palette::estimate(),
                v(EST_X[1], y),
            )?);
            let d = Dot::new()
                .point(v(plot_x(approx(g).map_err(fail)?), y - DOT_DY))
                .radius(PLOT_R)
                .style(flat(palette::estimate(), 1.0))
                .build();
            dots.push(stage.arena_mut().add(d));
        }
        Ok(Averaged { head, values, dots })
    }

    /// Group `members`' values and dots.
    fn of(&self, members: &[usize]) -> Vec<Mob> {
        members.iter().flat_map(|&j| [self.values[j], self.dots[j]]).collect()
    }
}

/// Each row's miss arrow in `color`, from the count to `values[j]` at the
/// row's sorted height offset by `dy`; none where the value is the count.
/// Detached.
fn miss_arrows(
    stage: &mut Stage<'_>,
    values: &[Q],
    count: Q,
    ys: &[f64],
    dy: f64,
    color: Srgb,
) -> fmn::Result<Vec<Option<Mob>>> {
    let from = plot_x(approx(count).map_err(fail)?);
    values
        .iter()
        .zip(ys)
        .map(|(&e, &y)| {
            if e == count {
                return Ok(None);
            }
            let to = plot_x(approx(e).map_err(fail)?);
            Ok(arrow_points(from, to, y + dy).map(|p| stroke(stage, p, line_style(color, ARROW_W, 1.0))))
        })
        .collect()
}

/// The arrows of group `members`.
fn arrows_of(arrows: &[Option<Mob>], members: &[usize]) -> Vec<Mob> {
    members.iter().filter_map(|&j| arrows[j]).collect()
}

/// An average worked out on the panel from `left` at height `y`, its result
/// in green. Detached.
fn mean_line(kit: &Kit, stage: &mut Stage<'_>, pieces: &[String; 2], x: f64, y: f64) -> fmn::Result<Vec<Mob>> {
    sum_texts(
        kit,
        stage,
        &[
            (pieces[0].clone(), Palette::ink()),
            (pieces[1].clone(), palette::estimate()),
        ],
        x,
        y,
    )
}

/// The widths table (12_13 to 12_15): the heads on the panel's first line,
/// then a row per width, its label at the panel's edge and each recipe's
/// value under its head. Detached.
fn widths_table(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture) -> fmn::Result<(Vec<Mob>, Vec<Vec<Mob>>)> {
    let colors = [palette::doubling(), palette::estimate()];
    let mut heads = Vec::new();
    for (c, h) in [DOUBLING_HEAD, GAP_HEAD].iter().enumerate() {
        heads.push(kit.text(stage, h, PANEL_SIZE, colors[c], v(TABLE_X[c], RIGHT_Y[0]))?);
    }
    let mut rows = Vec::new();
    for (r, row) in table_cells(pic).map_err(fail)?.iter().enumerate() {
        let mut mobs = vec![words_left(kit, stage, ROWS[r], &[], right(r + 1))?];
        for (c, s) in row.iter().enumerate() {
            mobs.push(kit.text(stage, s, PANEL_SIZE, colors[c], v(TABLE_X[c], RIGHT_Y[r + 1]))?);
        }
        rows.push(mobs);
    }
    Ok((heads, rows))
}

impl SceneConstruct for AveragingTheWobble {
    fn name(&self) -> &str {
        "12_averaging_the_wobble"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let pic = picture().map_err(fail)?;
        let groups = groups_of(&pic);
        let ys = sorted_y(&groups, pic.pairs.len());
        let count = Q::int(pic.boxes as i128);
        let index = |m: u64| group_index(&pic, m).ok_or_else(|| fail(format!("no group topped by {m}")));
        let biggest = pic.tops.len() - 1;
        let marks_of = |table: &Table, members: &[usize]| -> Vec<Mob> {
            members.iter().flat_map(|&j| table.rows[j].marks()).collect()
        };

        // 12_1: the badge; the ten rows, then doubling's estimates and dots;
        // the recipe and its rule for a pair on the panel
        let line = say(&kit, stage, "12_1")?;
        let corner = badge(&kit, stage, at(BADGE_SPOT))?;
        fade_in_all(stage, &[corner], 0.6)?;
        let table = Table::build(&kit, stage, &pic)?;
        line.wait_for(stage, "Here are its ten pairs again")?;
        fade_in_all(stage, &table.head, 0.4)?;
        let squares: Vec<Mob> = table.rows.iter().flat_map(|r| r.squares.clone()).collect();
        fade_in_lagged(stage, &squares, 1.6, 0.05)?;
        line.wait_for(stage, "the doubling recipe's estimates")?;
        let mut marks = vec![table.doubling_head];
        marks.extend(table.rows.iter().flat_map(|r| [r.estimate, r.dot]));
        fade_in_lagged(stage, &marks, 1.2, 0.1)?;
        line.wait_for(stage, "doubling the average just gives the total")?;
        let recipe = words_left(
            &kit,
            stage,
            &recipe_text(&pic),
            &[(DOUBLING_HEAD.to_owned(), palette::doubling())],
            right(0),
        )?;
        fade_in_all(stage, &[recipe], 0.5)?;
        line.wait_for(stage, "adds the two numbers")?;
        let pair = words_left(&kit, stage, &pair_text(&pic), &[], right(1))?;
        fade_in_all(stage, &[pair], 0.5)?;

        // 12_2: the average; halos round the smallest and the largest
        let line = say(&kit, stage, "12_2")?;
        line.wait_for(stage, "They average five")?;
        let average = words_left(&kit, stage, &average_text(&pic), &[], right(2))?;
        fade_in_all(stage, &[average], 0.5)?;
        line.wait_for(stage, "from two all the way up to eight")?;
        let ends: Vec<(Vec<Mob>, Srgb)> = pic
            .range
            .iter()
            .filter_map(|&r| pic.estimates.iter().position(|&e| e == Q::int(r as i128)))
            .map(|j| (vec![table.rows[j].estimate], palette::doubling()))
            .collect();
        glow_each(stage, &ends, 0.3, 0.6)?;

        // 12_3: a halo round each row topped by five
        let line = say(&kit, stage, "12_3")?;
        line.wait_for(stage, "so the four pairs topped by five")?;
        let topped: Vec<(Vec<Mob>, Srgb)> = groups[biggest]
            .iter()
            .map(|&j| (table.rows[j].marks(), Palette::ink()))
            .collect();
        glow_each(stage, &topped, 0.3, 0.8)?;

        // 12_4: the rows move, each whole, into their groups; the groups'
        // labels
        let line = say(&kit, stage, "12_4")?;
        line.wait_for(stage, "So sort the pairs")?;
        fade_out_all(stage, &[pair, average], 0.4)?;
        let mut moves = Vec::new();
        for (j, &y) in ys.iter().enumerate() {
            moves.extend(table.moves_to(&pic, j, y)?);
        }
        move_all(stage, &moves, 1.0, SORT_S)?;
        line.wait_for(stage, "That makes four groups")?;
        let mut labels = Vec::new();
        for (&m, members) in pic.tops.iter().zip(&groups) {
            labels.push(kit.text_left(
                stage,
                &group_label(m),
                LABEL_SIZE,
                Palette::ink(),
                v(GROUP_X, middle_y(&ys, members)),
            )?);
        }
        fade_in_lagged(stage, &labels, 1.0, 0.3)?;

        // 12_5: halos round the biggest group and its estimates; the
        // countdown
        let line = say(&kit, stage, "12_5")?;
        line.wait_for(stage, "Look at the biggest group")?;
        glow(stage, &marks_of(&table, &groups[biggest]), Palette::ink())?;
        line.wait_for(stage, "five, six, seven and eight")?;
        let estimates: Vec<Mob> = groups[biggest].iter().map(|&j| table.rows[j].estimate).collect();
        glow(stage, &estimates, palette::doubling())?;
        let q = think(&kit, stage, at(THINK_AT))?;

        // 12_6: the averaged column group by group, as the voice names each
        // average, the worked averages on the panel
        let line = say(&kit, stage, "12_6")?;
        fade_out_all(stage, &[q], 0.3)?;
        let green = Averaged::build(&kit, stage, &pic, &groups, &ys)?;
        let worked = |g: usize| mean_pieces(&pic.values[g], &pic.shown[g]);
        let cues = [
            "Six and a half",
            "Three and four average",
            "Four, five and six average five",
            "the lone pair topped by two",
        ];
        let mut means = Vec::new();
        let mut worked_row = 1;
        for (i, (&m, cue)) in SPOKEN_TOPS.iter().zip(cues).enumerate() {
            let g = index(m)?;
            line.wait_for(stage, cue)?;
            let mut landed = green.of(&groups[g]);
            if i == 0 {
                landed.push(green.head);
            }
            if pic.values[g].len() > 1 {
                let worked_line = mean_line(&kit, stage, &worked(g), RIGHT_X, RIGHT_Y[worked_row])?;
                worked_row += 1;
                landed.extend(&worked_line);
                means.extend(worked_line);
            }
            fade_in_all(stage, &landed, 0.5)?;
            if i == 0 {
                line.wait_for(stage, "every pair topped by five")?;
                let values: Vec<Mob> = groups[g].iter().map(|&j| green.values[j]).collect();
                glow(stage, &values, palette::estimate())?;
            }
        }

        // 12_7: the gap recipe's formula and a halo round the green column;
        // what averaging did
        let line = say(&kit, stage, "12_7")?;
        line.wait_for(stage, "the gap recipe's estimate")?;
        fade_out_all(stage, &means, 0.4)?;
        let rule = words_left(&kit, stage, &formula(pic.seen), &formula_colors(), right(1))?;
        fade_in_all(stage, &[rule], 0.5)?;
        glow(stage, &green.values, palette::estimate())?;
        line.wait_for(stage, "turns the doubling recipe into the gap recipe")?;
        let averaged_is = words_left(&kit, stage, AVERAGED_IS, &averaged_is_colors(), right(2))?;
        fade_in_all(stage, &[averaged_is], 0.5)?;

        // 12_8: a halo round the badge; the biggest group in bigger companies
        let line = say(&kit, stage, "12_8")?;
        line.wait_for(stage, "With a fair draw")?;
        glow(stage, &[corner], Palette::ink())?;
        line.wait_for(stage, "a company of six boxes, or eight")?;
        let mut companies = Vec::new();
        for (i, &c) in pic.more.iter().enumerate() {
            companies.push(words_left(&kit, stage, &company_text(&pic, c), &[], right(3 + i))?);
        }
        fade_in_lagged(stage, &companies, 0.8, 0.4)?;

        // 12_9: the panel clears; halos round the smallest and the biggest
        // group; each group's chance at its middle, a halo round the biggest
        let line = say(&kit, stage, "12_9")?;
        line.wait_for(stage, "Is the averaged recipe still unbiased?")?;
        let mut gone = vec![recipe, rule, averaged_is];
        gone.extend(&companies);
        fade_out_all(stage, &gone, 0.4)?;
        line.wait_for(stage, "The groups aren't equally likely")?;
        glow_each(
            stage,
            &[
                (marks_of(&table, &groups[0]), Palette::ink()),
                (marks_of(&table, &groups[biggest]), Palette::ink()),
            ],
            0.3,
            0.6,
        )?;
        let mut chances = Vec::new();
        for (g, members) in groups.iter().enumerate() {
            let pieces = chance_pieces(&pic, g).map_err(fail)?;
            let colored: Vec<(String, Srgb)> = pieces
                .iter()
                .enumerate()
                .map(|(i, s)| (s.clone(), if i == 2 { palette::estimate() } else { Palette::ink() }))
                .collect();
            chances.push(sum_texts(&kit, stage, &colored, RIGHT_X, middle_y(&ys, members))?);
        }
        line.wait_for(stage, "a group's chance is its number of pairs in ten")?;
        let firsts: Vec<Mob> = chances.iter().map(|c| c[0]).collect();
        fade_in_lagged(stage, &firsts, 0.8, 0.3)?;
        line.wait_for(stage, "four in ten")?;
        glow(stage, &[chances[biggest][0]], Palette::ink())?;

        // 12_10: each chance times its average; the weighted sum, then the
        // equal one; halos round the lone pair and the biggest group
        let line = say(&kit, stage, "12_10")?;
        line.wait_for(stage, "Weight each group's average by its chance")?;
        let rest: Vec<Mob> = chances.iter().flat_map(|c| c[1..].to_vec()).collect();
        fade_in_lagged(stage, &rest, 1.0, 0.2)?;
        line.wait_for(stage, "comes back, exactly five")?;
        let weighted = words_left(
            &kit,
            stage,
            &weighted_text(&pic).map_err(fail)?,
            &[],
            v(RIGHT_X, SUM_Y[0]),
        )?;
        fade_in_all(stage, &[weighted], 0.5)?;
        line.wait_for(stage, "Give the four groups equal weight")?;
        let equal = words_left(&kit, stage, &equal_text(&pic), &[], v(RIGHT_X, SUM_Y[1]))?;
        fade_in_all(stage, &[equal], 0.5)?;
        line.wait_for(stage, "the lone pair would count as much")?;
        glow_each(
            stage,
            &[
                (marks_of(&table, &groups[0]), Palette::ink()),
                (marks_of(&table, &groups[biggest]), Palette::ink()),
            ],
            0.3,
            0.6,
        )?;

        // 12_11: the panel clears; the count line and doubling's arrows; the
        // group topped by four, its misses and their average
        let line = say(&kit, stage, "12_11")?;
        line.wait_for(stage, "Now the misses")?;
        let mut gone = chances.concat();
        gone.extend([weighted, equal]);
        fade_out_all(stage, &gone, 0.4)?;
        line.wait_for(stage, "an arrow from five")?;
        let x5 = plot_x(approx(count).map_err(fail)?);
        let bottom = ys.iter().copied().fold(TABLE_TOP, f64::min);
        let count_line = stroke(
            stage,
            vec![v(x5, TABLE_TOP + COUNT_PAD), v(x5, bottom - COUNT_PAD)],
            line_style(Palette::dim(), COUNT_W, COUNT_O),
        );
        let count_label = kit.text(
            stage,
            &format!("count = {}", pic.boxes),
            LABEL_SIZE,
            Palette::dim(),
            v(x5, TABLE_TOP + HEAD_DY),
        )?;
        fade_in_all(stage, &[count_line, count_label], 0.4)?;
        let purple = miss_arrows(stage, &pic.estimates, count, &ys, DOT_DY, palette::doubling())?;
        let drawn: Vec<Mob> = purple.iter().flatten().copied().collect();
        fade_in_lagged(stage, &drawn, 1.2, 0.15)?;
        let cancel = index(pic.cancel)?;
        let cancel_y = middle_y(&ys, &groups[cancel]);
        line.wait_for(stage, "the group topped by four")?;
        glow(stage, &marks_of(&table, &groups[cancel]), Palette::ink())?;
        line.wait_for(stage, "minus one, zero and plus one")?;
        let cancel_text = misses_text(&pic.cancel_misses);
        let cancel_misses = words_left(
            &kit,
            stage,
            &cancel_text,
            &[(cancel_text.clone(), palette::doubling())],
            v(RIGHT_X, cancel_y + MISS_DY),
        )?;
        fade_in_all(stage, &[cancel_misses], 0.4)?;
        line.wait_for(stage, "misses by nothing at all")?;
        let cancel_mean = mean_line(
            &kit,
            stage,
            &mean_pieces(&pic.cancel_misses, &mean(&pic.cancel_misses).to_string()),
            RIGHT_X,
            cancel_y - MISS_DY,
        )?;
        fade_in_all(stage, &cancel_mean, 0.4)?;

        // 12_12: the group topped by five, its misses, its green arrows and
        // their average
        let line = say(&kit, stage, "12_12")?;
        let same = index(pic.same_side)?;
        let same_y = middle_y(&ys, &groups[same]);
        line.wait_for(stage, "the group topped by five")?;
        glow(stage, &marks_of(&table, &groups[same]), Palette::ink())?;
        line.wait_for(stage, "zero, one, two and three")?;
        let same_text = misses_text(&pic.same_misses);
        let same_misses = words_left(
            &kit,
            stage,
            &same_text,
            &[(same_text.clone(), palette::doubling())],
            v(RIGHT_X, same_y + MISS_DY),
        )?;
        fade_in_all(stage, &[same_misses], 0.4)?;
        line.wait_for(stage, "misses by one and a half")?;
        let averaged = grouped(&pic, &groups);
        let greens = miss_arrows(stage, &averaged, count, &ys, -DOT_DY, palette::estimate())?;
        let mut landed = arrows_of(&greens, &groups[same]);
        let same_mean = mean_line(
            &kit,
            stage,
            &mean_pieces(&pic.same_misses, &shown(pic.same_average).map_err(fail)?),
            RIGHT_X,
            same_y - MISS_DY,
        )?;
        landed.extend(&same_mean);
        fade_in_all(stage, &landed, 0.5)?;

        // 12_13: the miss lines go and the other green arrows come in; the
        // widths table's heads and its first row
        let line = say(&kit, stage, "12_13")?;
        line.wait_for(stage, "Add it up over all ten pairs")?;
        let mut gone = vec![cancel_misses, same_misses];
        gone.extend(&cancel_mean);
        gone.extend(&same_mean);
        fade_out_all(stage, &gone, 0.4)?;
        let others: Vec<Mob> = groups
            .iter()
            .enumerate()
            .filter(|&(g, _)| g != same)
            .flat_map(|(_, members)| arrows_of(&greens, members))
            .collect();
        fade_in_lagged(stage, &others, 0.8, 0.2)?;
        line.wait_for(stage, "Ignoring direction")?;
        let (heads, rows) = widths_table(&kit, stage, &pic)?;
        fade_in_all(stage, &heads, 0.4)?;
        line.wait_for(stage, "doubling misses by one point four")?;
        fade_in_all(stage, &rows[0][..2], 0.4)?;
        line.wait_for(stage, "the gap recipe by one point two")?;
        fade_in_all(stage, &rows[0][2..], 0.4)?;

        // 12_14: the rule low on the panel; halos round the cancelling and
        // the one-sided arrows
        let line = say(&kit, stage, "12_14")?;
        line.wait_for(stage, "The arrow of an average")?;
        let mut rule_lines = Vec::new();
        for (i, words) in RULE.iter().enumerate() {
            rule_lines.push(words_left(&kit, stage, words, &[], right(5 + i))?);
        }
        fade_in_all(stage, &rule_lines, 0.5)?;
        line.wait_for(stage, "It's shorter when they point both ways")?;
        glow(stage, &arrows_of(&purple, &groups[cancel]), palette::doubling())?;
        line.wait_for(stage, "just as long when they all point the same way")?;
        glow(stage, &arrows_of(&purple, &groups[same]), palette::doubling())?;

        // 12_15: the squared and the standard deviation rows; the drop;
        // held to be read
        let line = say(&kit, stage, "12_15")?;
        line.wait_for(stage, "squared misses average three")?;
        fade_in_all(stage, &rows[1], 0.4)?;
        line.wait_for(stage, "so the standard deviation falls")?;
        fade_in_all(stage, &rows[2], 0.4)?;
        line.wait_for(stage, "the drop, three quarters")?;
        let drop_line = words_left(&kit, stage, &wobble_text(&pic).map_err(fail)?, &[], right(4))?;
        fade_in_all(stage, &[drop_line], 0.5)?;
        let table_up = now(stage);
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;
        wait_until(stage, table_up + read_s(&table_text(&pic).map_err(fail)?))?;

        // 12_16: card H, held until it can be read
        let line = say(&kit, stage, "12_16")?;
        line.wait_for(stage, "keeps its average and never makes its misses bigger")?;
        let card = precisely(&kit, stage, CARD_H)?;
        fade_in_all(stage, &[card], 0.4)?;
        let card_up = now(stage);
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;
        // card H stays until it can be read
        wait_until(stage, card_up + read_s(CARD_H))?;
        pause(stage, END_BEAT)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use explainer_kit::Script;
    use explainer_kit::animatic::wrap;
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;
    use crate::chapters::draw::{PAUSE_NOTE, THINK_R};
    use crate::chapters::friends_line::{BADGE, BADGE_PAD, BADGE_SIZE, est_width};
    use crate::chapters::what_we_assume::{HALO_PAD, SUM_GAP};
    use crate::facts::PAIR_MORE;

    /// The countdown's note, at the size `think` writes it.
    const NOTE_SIZE: f64 = 18.0;
    /// A line's half height at the panel's size and at the label size, an
    /// estimate's at its size, and the space kept between things that must
    /// not touch.
    const HALF_LINE: f64 = 0.16;
    const HALF_SMALL: f64 = 0.12;
    const HALF_EST: f64 = 0.14;
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
    /// them: the pairs, the groups, the bigger companies, every line of the
    /// panel, the chances and sums, the misses, the widths table and card H.
    #[test]
    fn the_picture_is_the_facts() {
        let pic = good();
        assert_eq!(pic.pairs, model::all_handfuls(TINY, TINY_SEEN));
        assert_eq!(pic.more, PAIR_MORE);
        assert_eq!(groups_of(&pic), [vec![0], vec![1, 4], vec![2, 5, 7], vec![3, 6, 8, 9]]);
        assert_eq!(recipe_text(&pic), "doubling: 2 × average − 1");
        assert_eq!(pair_text(&pic), "for a pair: total − 1");
        assert_eq!(average_text(&pic), "average: 5");
        assert_eq!(group_label(5), "topped by 5");
        let worked: Vec<[String; 2]> = [3, 1, 2]
            .iter()
            .map(|&g| mean_pieces(&pic.values[g], &pic.shown[g]))
            .collect();
        assert_eq!(
            worked,
            [
                ["(5 + 6 + 7 + 8) ÷ 4 =", "6.5"],
                ["(3 + 4) ÷ 2 =", "3.5"],
                ["(4 + 5 + 6) ÷ 3 =", "5"]
            ]
            .map(|p| p.map(str::to_owned))
        );
        assert_eq!(formula(pic.seen), "estimate = largest + (largest − 2) ÷ 2");
        assert_eq!(company_text(&pic, 6), "6 boxes: 5, 6, 7, 8 average 6.5");
        assert_eq!(company_text(&pic, 8), "8 boxes: 5, 6, 7, 8 average 6.5");
        let chances: Vec<[String; 5]> = (0..4).map(|g| chance_pieces(&pic, g).unwrap()).collect();
        assert_eq!(
            chances,
            [
                ["1 in 10", "×", "2", "=", "0.2"],
                ["2 in 10", "×", "3.5", "=", "0.7"],
                ["3 in 10", "×", "5", "=", "1.5"],
                ["4 in 10", "×", "6.5", "=", "2.6"]
            ]
            .map(|p| p.map(str::to_owned))
        );
        assert_eq!(weighted_text(&pic).unwrap(), "0.2 + 0.7 + 1.5 + 2.6 = 5");
        assert_eq!(equal_text(&pic), "(2 + 3.5 + 5 + 6.5) ÷ 4 = 4.25");
        assert_eq!(misses_text(&pic.cancel_misses), "−1, 0, +1");
        assert_eq!(misses_text(&pic.same_misses), "0, +1, +2, +3");
        assert_eq!(
            mean_pieces(&pic.cancel_misses, "0"),
            ["(−1 + 0 + 1) ÷ 3 =", "0"].map(str::to_owned)
        );
        assert_eq!(
            mean_pieces(&pic.same_misses, &shown(pic.same_average).unwrap()),
            ["(0 + 1 + 2 + 3) ÷ 4 =", "1.5"].map(str::to_owned)
        );
        assert_eq!(
            table_cells(&pic).unwrap(),
            [["1.4", "1.2"], ["3", "2.25"], ["≈ 1.7", "1.5"]].map(|r| r.map(str::to_owned))
        );
        assert_eq!(wobble_text(&pic).unwrap(), "3 − 2.25 = 0.75: wobble inside groups");
        assert_eq!(
            table_text(&pic).unwrap(),
            "doubling gap recipe average miss 1.4 1.2 average squared miss 3 2.25 standard deviation ≈ 1.7 1.5 \
             3 − 2.25 = 0.75: wobble inside groups"
        );
        assert_eq!(rule_text("smaller", (Q::int(3), Q::int(-1))), "3 × smaller − 1");
        assert_eq!(rule_text("largest", (Q::new(7, 6), Q::int(-1))), "7/6 × largest − 1");
        assert_eq!(rule_text("total", (Q::ONE, Q::int(2))), "total + 2");
        assert_eq!(sum_of(&[Q::int(2), Q::int(-3)]), "2 − 3");
        assert_eq!(linear("7/6*largest-1", "largest"), Ok((Q::new(7, 6), Q::int(-1))));
        assert!(linear("2*average-1", "total").is_err());
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
        let mut swapped = pic.estimates.clone();
        swapped.swap(0, 9);
        refused(
            Picture {
                estimates: swapped,
                ..pic.clone()
            },
            "the doubling estimates",
        );
        refused(
            Picture {
                pair_rule: (Q::int(2), Q::int(-1)),
                ..pic.clone()
            },
            "the doubling estimates",
        );
        refused(
            Picture {
                range: vec![2, 7],
                ..pic.clone()
            },
            "the doubling estimates",
        );
        refused(
            Picture {
                sizes: vec![1, 2, 3, 5],
                ..pic.clone()
            },
            "the groups topped by",
        );
        let mut shuffled = pic.values.clone();
        shuffled[3] = vec![Q::int(8), Q::int(7), Q::int(6), Q::int(5)];
        refused(
            Picture {
                values: shuffled,
                ..pic.clone()
            },
            "the groups topped by",
        );
        let mut rounded = pic.shown.clone();
        rounded[3] = "7".to_owned();
        refused(
            Picture {
                shown: rounded,
                ..pic.clone()
            },
            "the groups topped by",
        );
        // a "bigger" company that is the tiny one itself
        refused(
            Picture {
                more: vec![5, 8],
                ..pic.clone()
            },
            "the groups topped by",
        );
        // the biggest group's chance and the weights
        refused(
            Picture {
                top_chance: Q::new(1, 4),
                ..pic.clone()
            },
            "the groups' chances",
        );
        refused(
            Picture {
                equal: pic.weighted,
                ..pic.clone()
            },
            "the groups' chances",
        );
        refused(
            Picture {
                equal_shown: "4.3".to_owned(),
                ..pic.clone()
            },
            "the groups' chances",
        );
        // misses that do not cancel, or that point both ways
        refused(
            Picture {
                cancel: 5,
                ..pic.clone()
            },
            "the misses of the group",
        );
        refused(
            Picture {
                same_side: 4,
                same_misses: pic.cancel_misses.clone(),
                ..pic.clone()
            },
            "the misses of the group",
        );
        refused(
            Picture {
                same_average: Q::ONE,
                ..pic.clone()
            },
            "the misses of the group",
        );
        // the widths: rounded, swapped, an exact "about"
        refused(
            Picture {
                misses: ["1.2".to_owned(), "1.4".to_owned()],
                ..pic.clone()
            },
            "the widths",
        );
        refused(
            Picture {
                squared: [Q::int(3), Q::new(5, 2)],
                ..pic.clone()
            },
            "the widths",
        );
        refused(
            Picture {
                sd_about: "1.8".to_owned(),
                ..pic.clone()
            },
            "the widths",
        );
        refused(
            Picture {
                spread: Q::new(1, 2),
                ..pic
            },
            "the widths",
        );
    }

    /// The regroup moves each pair whole and keeps every pair: each row
    /// gets its own height, the groups' rows stay together in table order,
    /// a group's gap is wider than a row's step, the rows of a group are as
    /// many as it tops, and every row's squares, estimate and dot move by
    /// the same step.
    #[test]
    fn the_regroup_moves_each_pair_whole() {
        let pic = good();
        let groups = groups_of(&pic);
        let ys = sorted_y(&groups, pic.pairs.len());
        let mut seen: Vec<f64> = ys.clone();
        seen.sort_by(f64::total_cmp);
        seen.dedup();
        assert_eq!(seen.len(), pic.pairs.len());
        let order: Vec<usize> = groups.concat();
        let mut every = order.clone();
        every.sort_unstable();
        assert_eq!(every, (0..pic.pairs.len()).collect::<Vec<_>>());
        for w in order.windows(2) {
            assert!(ys[w[0]] > ys[w[1]], "{w:?}");
        }
        for members in &groups {
            for w in members.windows(2) {
                assert!(w[0] < w[1]);
                assert!((ys[w[0]] - ys[w[1]] - ROW_STEP).abs() < 1e-9);
            }
        }
        for w in groups.windows(2) {
            let (a, b) = (*w[0].last().unwrap(), w[1][0]);
            assert!((ys[a] - ys[b] - ROW_STEP - GROUP_GAP).abs() < 1e-9);
        }
        for (members, &m) in groups.iter().zip(&pic.tops) {
            assert_eq!(members.len() as u64, m - 1);
            assert!(members.iter().all(|&j| pic.pairs[j][TINY_SEEN - 1] == m));
        }
        // the groups as fair pairs from any company from the top up
        for top in 2..=6u64 {
            let base: Vec<Q> = model::all_handfuls(top, TINY_SEEN)
                .iter()
                .filter(|p| p[1] == top)
                .map(|p| model::doubling_recipe(p))
                .collect();
            for c in top..=12 {
                let here: Vec<Q> = model::all_handfuls(c, TINY_SEEN)
                    .iter()
                    .filter(|p| p[1] == top)
                    .map(|p| model::doubling_recipe(p))
                    .collect();
                assert_eq!(here, base, "top {top}, company {c}");
            }
            assert_eq!(mean(&base), model::gap_of(top, TINY_SEEN as u64), "top {top}");
        }
    }

    /// Averaging inside the groups keeps the average and never makes a miss
    /// bigger, for the doubling recipe and for three times the smaller
    /// minus one, pairs from every company of 2 to 10 boxes, weighted by
    /// the groups' true chances; equal weights would not keep it. The plain
    /// miss shrinks only where some group's misses point both ways.
    #[test]
    fn averaging_keeps_the_average_and_never_widens() {
        let smaller = |p: &[u64]| Q::int(3 * p[0] as i128 - 1);
        type Recipe<'a> = &'a dyn Fn(&[u64]) -> Q;
        let recipes: [Recipe<'_>; 2] = [&|p: &[u64]| model::doubling_recipe(p), &smaller];
        for recipe in recipes {
            for c in 2..=10u64 {
                let pairs = model::all_handfuls(c, TINY_SEEN);
                let n = Q::int(c as i128);
                let mut by_top: BTreeMap<u64, Vec<Q>> = BTreeMap::new();
                for p in &pairs {
                    by_top.entry(p[1]).or_default().push(recipe(p.as_slice()));
                }
                let raw: Vec<Q> = pairs.iter().map(|p| recipe(p.as_slice())).collect();
                let avg: Vec<Q> = pairs.iter().map(|p| mean(&by_top[&p[1]])).collect();
                let off = |xs: &[Q], f: &dyn Fn(Q) -> Q| mean(&xs.iter().map(|&e| f(e - n)).collect::<Vec<_>>());
                let plain = |x: Q| x.abs();
                let square = |x: Q| x * x;
                assert_eq!(mean(&raw), n, "c {c}");
                assert_eq!(mean(&avg), n, "c {c}");
                assert!(off(&avg, &plain) <= off(&raw, &plain), "c {c}");
                assert!(off(&avg, &square) <= off(&raw, &square), "c {c}");
                let both_ways = by_top
                    .values()
                    .any(|g| g.iter().any(|&e| e < n) && g.iter().any(|&e| e > n));
                assert_eq!(off(&avg, &plain) < off(&raw, &plain), both_ways, "c {c}");
                let tops: Vec<Q> = by_top.values().map(|g| mean(g)).collect();
                assert_eq!(mean(&tops) == n, c == TINY_SEEN as u64, "c {c}");
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
                "12_1",
                &[
                    "Here are its ten pairs again",
                    "the doubling recipe's estimates",
                    "doubling the average just gives the total",
                    "adds the two numbers",
                ],
            ),
            ("12_2", &["They average five", "from two all the way up to eight"]),
            ("12_3", &["so the four pairs topped by five"]),
            ("12_4", &["So sort the pairs", "That makes four groups"]),
            ("12_5", &["Look at the biggest group", "five, six, seven and eight"]),
            (
                "12_6",
                &[
                    "Six and a half",
                    "every pair topped by five",
                    "Three and four average",
                    "Four, five and six average five",
                    "the lone pair topped by two",
                ],
            ),
            (
                "12_7",
                &[
                    "the gap recipe's estimate",
                    "turns the doubling recipe into the gap recipe",
                ],
            ),
            ("12_8", &["With a fair draw", "a company of six boxes, or eight"]),
            (
                "12_9",
                &[
                    "Is the averaged recipe still unbiased?",
                    "The groups aren't equally likely",
                    "a group's chance is its number of pairs in ten",
                    "four in ten",
                ],
            ),
            (
                "12_10",
                &[
                    "Weight each group's average by its chance",
                    "comes back, exactly five",
                    "Give the four groups equal weight",
                    "the lone pair would count as much",
                ],
            ),
            (
                "12_11",
                &[
                    "Now the misses",
                    "an arrow from five",
                    "the group topped by four",
                    "minus one, zero and plus one",
                    "misses by nothing at all",
                ],
            ),
            (
                "12_12",
                &[
                    "the group topped by five",
                    "zero, one, two and three",
                    "misses by one and a half",
                ],
            ),
            (
                "12_13",
                &[
                    "Add it up over all ten pairs",
                    "Ignoring direction",
                    "doubling misses by one point four",
                    "the gap recipe by one point two",
                ],
            ),
            (
                "12_14",
                &[
                    "The arrow of an average",
                    "It's shorter when they point both ways",
                    "just as long when they all point the same way",
                ],
            ),
            (
                "12_15",
                &[
                    "squared misses average three",
                    "so the standard deviation falls",
                    "the drop, three quarters",
                ],
            ),
            ("12_16", &["keeps its average and never makes its misses bigger"]),
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
        let asked = script.text("12_5").unwrap();
        for answer in ["half", "6.5", "gap recipe"] {
            assert!(!asked.contains(answer), "{answer}");
        }
    }

    /// The table, the plots, the panel and the countdown keep apart: the
    /// squares, both estimate columns and the plots in order across, every
    /// row in the frame and over card H, a row's halo clear of the next
    /// one's in table order; the labels right of the plots and left of the
    /// panel; every panel line, chance line, sum and miss line in the
    /// title-safe area, under the badge, the widths table's columns apart
    /// and clear of its labels; the countdown under the recipe line and
    /// right of it.
    #[test]
    fn the_table_the_plots_and_the_panel_fit_the_frame() {
        let pic = good();
        let groups = groups_of(&pic);
        let ys = sorted_y(&groups, pic.pairs.len());
        let safe_x = FRAME_X_RADIUS - 0.25;
        let w = |s: &str| est_width(s, PANEL_SIZE);
        let ws = |s: &str| est_width(s, LABEL_SIZE);
        let we = |s: &str| est_width(s, EST_SIZE);
        // across: the squares, the two estimate columns, the plots, the labels
        assert!(sq_x(1) - SQ / 2.0 > -safe_x);
        let widest = |vals: &[String]| vals.iter().map(|s| we(s)).fold(0.0, f64::max) / 2.0;
        let purple: Vec<String> = pic.estimates.iter().map(Q::to_string).collect();
        assert!(sq_x(pic.boxes) + SQ / 2.0 + CLEAR < EST_X[0] - widest(&purple));
        assert!(EST_X[0] + widest(&purple) + CLEAR < EST_X[1] - widest(&pic.shown));
        // the head row: the last column number, both heads, the count's label
        let last_number = sq_x(pic.boxes) + ws(&pic.boxes.to_string()) / 2.0;
        assert!(last_number + CLEAR < EST_X[0] - ws(DOUBLING_HEAD) / 2.0);
        assert!(EST_X[0] + ws(DOUBLING_HEAD) / 2.0 + CLEAR < EST_X[1] - ws(AVERAGED_HEAD) / 2.0);
        let x5 = plot_x(pic.boxes as f64);
        let count_half = ws(&format!("count = {}", pic.boxes)) / 2.0;
        assert!(EST_X[1] + ws(AVERAGED_HEAD) / 2.0 + CLEAR < x5 - count_half);
        assert!(x5 + count_half < RIGHT_X);
        let lowest = pic.estimates.iter().chain(&pic.averages).min().copied().unwrap();
        let highest = pic.estimates.iter().max().copied().unwrap();
        assert!(EST_X[1] + widest(&pic.shown) + CLEAR < plot_x(approx(lowest).unwrap()) - PLOT_R);
        assert!(plot_x(approx(highest).unwrap()) + PLOT_R + CLEAR < GROUP_X);
        let label_w = pic.tops.iter().map(|&m| ws(&group_label(m))).fold(0.0, f64::max);
        assert!(GROUP_X + label_w + CLEAR < RIGHT_X);
        // up and down: the head, the rows in both orders, card H, halos
        const { assert!(TABLE_TOP + HEAD_DY + HALF_SMALL < FRAME_Y_RADIUS - 0.25) };
        const { assert!(TABLE_TOP + COUNT_PAD < TABLE_TOP + HEAD_DY - HALF_SMALL) };
        let lowest_row = ys.iter().copied().fold(TABLE_TOP, f64::min);
        assert!(table_y(pic.pairs.len() - 1) > lowest_row);
        assert!(lowest_row - COUNT_PAD > CARD_TOP + CLEAR);
        assert!(lowest_row - SQ / 2.0 > CARD_TOP + CLEAR);
        const { assert!(SQ + 2.0 * HALO_PAD < ROW_STEP) };
        const { assert!(2.0 * HALF_EST + 2.0 * HALO_PAD < ROW_STEP) };
        // arrows and dots over and under a row's centre stay inside its row
        const { assert!(DOT_DY + ARROW_HEAD * ARROW_SPREAD < ROW_STEP / 2.0) };
        const { assert!(DOT_DY - ARROW_HEAD * ARROW_SPREAD > 0.0) };
        // the panel and the badge
        let badge_half = est_width(BADGE, BADGE_SIZE) / 2.0 + BADGE_PAD[0];
        assert!(BADGE_SPOT[0] + badge_half < safe_x);
        assert!(BADGE_SPOT[0] - badge_half > RIGHT_X);
        const { assert!(RIGHT_Y[0] + HALF_LINE < BADGE_SPOT[1] - 0.15 - BADGE_PAD[1] - CLEAR) };
        // the first group's chance line (12_9) sits higher than the panel's
        // first line, still under the badge
        assert!(middle_y(&ys, &groups[0]) + HALF_LINE < BADGE_SPOT[1] - 0.15 - BADGE_PAD[1] - CLEAR);
        let mut lines = vec![
            recipe_text(&pic),
            pair_text(&pic),
            average_text(&pic),
            formula(pic.seen),
            AVERAGED_IS.to_owned(),
            weighted_text(&pic).unwrap(),
            equal_text(&pic),
            wobble_text(&pic).unwrap(),
        ];
        lines.extend(RULE.iter().map(|s| (*s).to_owned()));
        lines.extend(pic.more.iter().map(|&c| company_text(&pic, c)));
        for (g, (values, s)) in pic.values.iter().zip(&pic.shown).enumerate() {
            let pieces = chance_pieces(&pic, g).unwrap();
            let gaps = SUM_GAP * (pieces.len() - 1) as f64;
            assert!(RIGHT_X + w(&pieces.join("")) + gaps < safe_x);
            let [a, b] = mean_pieces(values, s);
            assert!(RIGHT_X + w(&a) + SUM_GAP + w(&b) < safe_x);
        }
        lines.push(misses_text(&pic.same_misses));
        for text in &lines {
            assert!(RIGHT_X + w(text) < safe_x, "{text}");
        }
        for p in RIGHT_Y.windows(2) {
            assert!(p[0] - HALF_LINE > p[1] + HALF_LINE);
        }
        const { assert!(SUM_Y[1] - HALF_LINE > -FRAME_Y_RADIUS + 0.25) };
        const { assert!(SUM_Y[0] - HALF_LINE > SUM_Y[1] + HALF_LINE) };
        assert!(SUM_Y[0] + HALF_LINE < middle_y(&ys, &groups[groups.len() - 1]) - HALF_LINE);
        // a group's two miss lines and the next group's chance line
        const { assert!(2.0 * MISS_DY > 2.0 * HALF_LINE) };
        for w2 in groups.windows(2) {
            let (a, b) = (middle_y(&ys, &w2[0]), middle_y(&ys, &w2[1]));
            assert!(a - MISS_DY - HALF_LINE > b + MISS_DY + HALF_LINE);
        }
        // the widths table: its labels left of the columns, the columns apart
        let label_end = RIGHT_X + ROWS.iter().map(|s| w(s)).fold(0.0, f64::max);
        let cells = table_cells(&pic).unwrap();
        let col = |c: usize| {
            cells
                .iter()
                .map(|r| w(&r[c]))
                .chain([w([DOUBLING_HEAD, GAP_HEAD][c])])
                .fold(0.0, f64::max)
                / 2.0
        };
        assert!(label_end + CLEAR < TABLE_X[0] - col(0));
        assert!(TABLE_X[0] + col(0) + CLEAR < TABLE_X[1] - col(1));
        assert!(TABLE_X[1] + col(1) < safe_x);
        // the countdown (12_5), with only the recipe line on the panel: under
        // it, right of the groups' labels, inside the frame
        let note_half = est_width(PAUSE_NOTE, NOTE_SIZE) / 2.0;
        const { assert!(THINK_AT[1] + THINK_R < RIGHT_Y[0] - HALF_LINE - CLEAR) };
        assert!(THINK_AT[0] - note_half.max(THINK_R) > GROUP_X + label_w + CLEAR);
        assert!(THINK_AT[0] + note_half.max(THINK_R) < safe_x);
    }

    /// Card H says the averaging theorem exactly, the fairness first, in at
    /// most three lines at every width the lower third might narrow to, and
    /// is up long enough to be read.
    #[test]
    fn card_h_fits_and_is_read() {
        for chars in (CARD_CHARS_NARROWEST..=CARD_CHARS_WIDEST).step_by(4) {
            assert!(wrap(CARD_H, chars, 4).len() <= 3, "{chars}");
        }
        assert!(CARD_H.starts_with("fair handful"));
        assert!(CARD_H.contains("free of N") && CARD_H.contains("never grow"));
        assert!(read_s(CARD_H) < 15.0);
        // the negative control: one clause more would not fit
        let longer = format!("{CARD_H} The count is one number.");
        assert!(wrap(&longer, CARD_CHARS_NARROWEST, 4).len() > 3);
    }
}
