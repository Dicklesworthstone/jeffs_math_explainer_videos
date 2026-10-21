//! Chapters 09, 10 and 11: what it costs, exact and rounded, and the
//! promise.
//!
//! 09, what it costs. At a thousand rows one check takes a million
//! multiplications, a thousandth of the product's billion (09_1), and twenty
//! a fiftieth: a count of work, not of seconds (09_2), which also includes
//! additions and memory traffic (09_3). At three by three, how many checks
//! cost as much as the product? (09_4, a prediction.) Three (09_5). The
//! whole check as a loop (09_6, 09_7).
//!
//! 10, exact and rounded. The pairing proof needs exact sums (10_1). What
//! is 10^16 + 1 in doubles? (10_2, a prediction.) 10^16: the doubles there
//! are two apart, so the one is lost (10_3). So a right answer can be caught
//! (10_4), and a wrong answer by one passes all four tosses (10_5). A
//! tolerance lets the right one through, but then a wrong C inside it passes
//! every toss, with no half guarantee (10_6).
//!
//! 11, the promise. The promise is for exact arithmetic, on the opening
//! picture (11_1), and what it assumes (11_2). Back to the start (11_3,
//! 11_4). How many fresh checks for one in a billion? (11_5, a prediction.)
//! Thirty, still three hundredths of the product (11_6). A mistake hides
//! only when its heads columns add to zero, and the coin over a nonzero
//! entry of the mistake breaks that (11_7); the names to search and the
//! receipts.

use explainer_kit::props::{CodePanel, lower_third, names_to_search, precisely, receipts_card};
use explainer_kit::{Kit, Palette, fade_all, rect, v};
use fmn::prelude::*;

use super::check_with_coins::{PANEL_X, keep_columns};
use super::draw::{
    COIN_R, cell_box, coin, coin_mobs, column_grid, fade_in_all, fade_in_lagged, fade_out_all, fade_to, fail,
    flash_all, flat, fly, grid_colored, line_style, matrix_grid, now, opening, read_s, ring, say, stand, swap,
    tex_grid, tex_of, tex_thousands, think, together, toss_coins, wait_until,
};
use crate::facts::{self, AGAIN, BIG_SIDE, BILLION, HIDING_TOSS, PROOF_SHAPE, RoundedToss};
use crate::model::{self, A, B, BIG, C, FIRST_TOSS, Matrix, ROUNDED_PASSES, ROUNDED_REJECTS, SIDE};
use crate::palette;

pub struct WhatItCosts {
    pub kit: Kit,
}

pub struct ExactAndRounded {
    pub kit: Kit,
}

pub struct ThePromise {
    pub kit: Kit,
}

// the chapters' fixed words (numbers are built from the model and facts)
const PROGRAMMERS: &str = "for programmers:";
/// 09_1: the counting convention (replaces the earlier note that said
/// every entry is read from memory; a selected-column check need not read
/// the tails columns of B and C).
const COST_CARD: &str = "multiplications counted row by column, n cubed for the product and n squared per check; \
                         time also needs additions and memory traffic";
const BREAK_QUESTION: &str = "At three by three, how many checks cost as much as the product?";
const STEP_LINES: [&str; 2] = [
    "two sums that differ by a nonzero entry are not both 0",
    "with exact sums",
];
const DOUBLES: &str = "doubles";
const EXACT: &str = "exact";
const MATCH: &str = "match";
const NO_MATCH: &str = "no match";
const NAMES: [&str; 3] = ["A", "B", "C"];
const MISTAKE_NAME: &str = r"C - AB";
const RIGHT_LINE: &str = "a right C: every toss matches";
const WRONG_LINE: &str = "a wrong C: each fresh toss misses at most half the time";
const BOTH_FAIL: &str = "in doubles, both can fail";
const ASSUMES: [&str; 3] = ["fair coins", "tossed after C is fixed", "new coins for every check"];
const HIDES: &str = "hides";
const CAUGHT: &str = "caught";
const SEARCH: [&str; 2] = ["Schwartz\u{2013}Zippel lemma", "polynomial identity testing"];
const RECEIPTS: [&str; 3] = [
    "every number in this video was computed by code",
    "and checked by an independent oracle",
    "github.com/Dicklesworthstone/jeffs_math_explainer_videos",
];
/// N = 10^16, set upright, the same N the question and the cards print
/// (review F8: an italic N once stood beside an upright one).
const N_TEX: &str = r"\text{N}";
/// The one added to N = 10^16, which doubles lose (10_2, 10_3).
const PLUS: i64 = 1;
/// The whole numbers around N on the number line, as offsets from N.
const OFFSETS: std::ops::RangeInclusive<i64> = -2..=2;

// ------------------------------------------------------------------ layout

/// 09: n at the top, then three bars to scale, each under its label.
const COST_N_AT: Vec3 = [0.0, 3.05, 0.0];
const BAR_X: f64 = -5.0;
const BAR_W: f64 = 10.0;
const BAR_H: f64 = 0.32;
const LABEL_YS: [f64; 3] = [2.1, 0.8, -0.45];
const BAR_YS: [f64; 3] = [1.55, 0.25, -1.0];
const RATIO_AT: Vec3 = [0.0, -1.8, 0.0];
const SHARE_AT: Vec3 = [0.0, -2.6, 0.0];
/// The narrowest bar drawn, about a pixel at 1080p.
const MIN_BAR_W: f64 = 0.01;
const BREAK_AT: Vec3 = [-0.6, -0.45, 0.0];
const BREAK_THINK_AT: Vec3 = [5.4, -0.9, 0.0];
/// The code panel's type size and width.
const CODE_SIZE: f64 = 22.0;
const CODE_W: f64 = 7.2;
/// 10_1: the proof's step.
const STEP_YS: [f64; 2] = [0.6, -0.1];
/// 10_2, 10_3: N's name at the top left (until 10_6), the number line
/// around N, its doubles, and the question under it.
const N_NOTE_AT: Vec3 = [-5.6, 3.3, 0.0];
const LINE_Y: f64 = 1.6;
const LINE_DX: f64 = 2.4;
const LINE_HALF: f64 = 5.6;
const TICK_H: f64 = 0.24;
const LINE_LABEL_DY: f64 = -0.55;
const DOT_R: f64 = 0.11;
const GAP_LABEL_DY: f64 = 0.45;
const LEGEND_AT: Vec3 = [-5.0, 2.45, 0.0];
const QUESTION_AT: Vec3 = [-1.2, -0.6, 0.0];
const ANSWER_DY: f64 = -0.6;
const THINK_AT: Vec3 = [5.0, -0.6, 0.0];
/// 10_4 to 10_6: A, B, C and the mistake C − AB, two by two; the four
/// tosses of two coins; the verdict rows, exact and in doubles; the work.
const GRIDS_Y: f64 = 2.2;
const GRID_XS: [f64; 4] = [-5.3, -3.1, -0.9, 1.9];
const GRID_NAME_Y: f64 = 1.2;
const TOSS_XS: [f64; 4] = [-3.6, -1.2, 1.2, 3.6];
const COIN_Y: f64 = 0.3;
const COIN_PITCH: f64 = 0.5;
const SHOWN_R: f64 = 0.5;
const ROW_LABEL_X: f64 = -6.5;
const ROW_YS: [f64; 2] = [-0.45, -1.0];
const WORK_YS: [f64; 2] = [-1.7, -2.25];
/// The equality rows, dimmed while the tolerant test stands under them.
const EQUALITY_DIM: f64 = 0.3;
/// 11_1, 11_2: the promise under the opening picture, and its assumptions.
const PROMISE_X: f64 = -1.0;
const PROMISE_YS: [f64; 4] = [-0.15, -0.7, -1.3, -1.95];
const ASSUME_X: f64 = 3.4;
const ASSUME_YS: [f64; 3] = [-0.15, -0.7, -1.25];
/// 11_3, 11_4: back to the start: the coins over C, the counts at the right,
/// the thousand rows under the picture.
const OPEN_COIN_Y: f64 = 3.5;
const START_YS: [f64; 2] = [3.0, 2.4];
const SCALE_YS: [f64; 2] = [-0.45, -1.25];
/// 11_5, 11_6: the question and the answer under them.
const BILLION_AT: Vec3 = [-0.8, -2.25, 0.0];
const BILLION_DY: f64 = -0.75;
const BILLION_THINK_AT: Vec3 = [5.4, -1.95, 0.0];
/// 11_7: the proof's mistake, a toss over it, and its row sums beside it.
const LAST_D_AT: Vec3 = [-3.2, 1.0, 0.0];
const LAST_COIN_DY: f64 = 1.35;
const LAST_SUM_X: f64 = -0.6;
const LAST_WORD_X: f64 = 0.4;
const LAST_NAME_DY: f64 = -1.25;

/// Where offset `k` from N stands on the number line.
fn line_x(k: i64) -> f64 {
    k as f64 * LINE_DX
}

/// A whole number as TeX, near N = 10^16 written from an upright N:
/// "\text{N}", "\text{N} + 1", "-\text{N} + 1"; any other as itself.
fn n_tex(x: i64) -> String {
    let from = |base: i64, name: &str| match x - base {
        0 => Some(name.to_owned()),
        k @ 1..=9 => Some(format!("{name} + {k}")),
        k @ -9..=-1 => Some(format!("{name} - {}", -k)),
        _ => None,
    };
    from(BIG, N_TEX)
        .or_else(|| from(-BIG, &format!("-{N_TEX}")))
        .unwrap_or_else(|| x.to_string())
}

/// A column written across, as TeX: "(\text{N} + 1,\; -\text{N} + 1)".
fn tuple_tex(column: &[i64]) -> String {
    let entries: Vec<String> = column.iter().map(|&x| n_tex(x)).collect();
    format!("({})", entries.join(r",\; "))
}

/// The cost at a thousand rows (09_1, 09_2), from the counted work.
struct Cost {
    n: String,
    redo: String,
    one: String,
    twenty: String,
    one_ratio: String,
    twenty_ratio: String,
    /// One check's and twenty checks' multiplications as shares of the
    /// product's.
    shares: [f64; 2],
}

fn cost() -> Result<Cost, String> {
    let redo = model::product_multiplications(BIG_SIDE);
    let one = model::check_multiplications(BIG_SIDE);
    let again = u64::from(AGAIN) * one;
    let one_share = facts::ratio(one as i64, redo as i64)?;
    let share = facts::ratio(again as i64, redo as i64)?;
    Ok(Cost {
        n: format!("n = {BIG_SIDE}"),
        redo: format!(
            r"\text{{redo the product, row by column: }} n^3 = {}",
            tex_thousands(redo)
        ),
        one: format!(r"\text{{1 check: }} n^2 = {}", tex_thousands(one)),
        twenty: format!(
            r"\text{{{AGAIN} fresh checks: }} {AGAIN} \times n^2 = {}",
            tex_thousands(again)
        ),
        one_ratio: format!(
            r"\frac{{{}}}{{{}}} = {}",
            tex_thousands(one),
            tex_thousands(redo),
            tex_of(&one_share)
        ),
        twenty_ratio: format!(
            r"\frac{{{}}}{{{}}} = {}",
            tex_thousands(again),
            tex_thousands(redo),
            tex_of(&share)
        ),
        shares: [one as f64 / redo as f64, again as f64 / redo as f64],
    })
}

/// The break-even at three by three (09_4, 09_5): the bars' labels, the
/// checks that equal the product, and the share k/n, all from the counted
/// work. Refused unless three checks cost the product's 27.
struct BreakEven {
    n: String,
    redo: String,
    one: String,
    stacked: String,
    rule: String,
    examples: String,
    checks: u64,
}

fn break_even() -> Result<BreakEven, String> {
    let n = SIDE as u64;
    let redo = model::product_multiplications(n);
    let one = model::check_multiplications(n);
    let checks = facts::break_even(n);
    if checks * one != redo {
        return Err(format!("{checks} checks of {one} are not the product's {redo}"));
    }
    let big = tex_of(&facts::ratio(i64::from(AGAIN), BIG_SIDE as i64)?);
    Ok(BreakEven {
        n: format!("n = {n}"),
        redo: format!(r"\text{{redo: }} {n}^3 = {redo}"),
        one: format!(r"\text{{1 check: }} {n}^2 = {one}"),
        stacked: format!(r"{checks} \times {one} = {redo}"),
        rule: r"k \text{ checks: } \frac{k\,n^2}{n^3} = \frac{k}{n}".to_owned(),
        examples: format!(
            r"\frac{{{checks}}}{{{n}}} = {},\;\;\; \frac{{{AGAIN}}}{{{BIG_SIDE}}} = {big}",
            tex_of(&facts::ratio(checks as i64, n as i64)?)
        ),
        checks,
    })
}

/// The loop a programmer writes (09_6, 09_7), in plain ASCII for the
/// typewriter face.
fn code_lines() -> Vec<String> {
    vec![
        format!("repeat {AGAIN} times:"),
        "    r = fresh fair coins, one per column".to_owned(),
        "    br = B's heads columns, added".to_owned(),
        "    x = A * br".to_owned(),
        "    y = C's heads columns, added".to_owned(),
        "    if x != y: return \"C is wrong\"".to_owned(),
        format!("return \"C passed {AGAIN} fresh checks\""),
    ]
}

/// N + 1 in doubles (10_2, 10_3), every number computed in doubles.
struct NearN {
    note: String,
    labels: Vec<String>,
    /// The offsets from N that are doubles.
    doubles: Vec<i64>,
    question: String,
    /// Where N + 1 lands in doubles, as an offset from N.
    landing: i64,
    gap: i64,
    gap_label: String,
    answer: [String; 2],
    card: String,
}

/// Refused unless N is a power of ten, N + 1 lands on the double N, and
/// N + 1 lies halfway between N and the next double up (the line says the
/// one is lost and the doubles are two apart).
fn near_n() -> Result<NearN, String> {
    let digits = BIG.to_string().len() - 1;
    if 10_i64.pow(digits as u32) != BIG {
        return Err(format!("{BIG} is not a power of ten"));
    }
    let near = BIG as f64;
    let doubles: Vec<i64> = OFFSETS
        .filter(|&k| facts::as_whole(&[(BIG + k) as f64])[0] == BIG + k)
        .collect();
    let landing = facts::as_whole(&[near + PLUS as f64])[0] - BIG;
    let gap = facts::as_whole(&[near.next_up() - near])[0];
    let left = facts::as_whole(&[(near + PLUS as f64) - near])[0];
    if landing != 0 || 2 * PLUS != gap || !doubles.contains(&landing) || !doubles.contains(&gap) {
        return Err("in doubles N + 1 must land on N, halfway to the next double".into());
    }
    let bits = f64::MANTISSA_DIGITS;
    let (plus, at, next) = (n_tex(BIG + PLUS), n_tex(BIG + landing), n_tex(BIG + gap));
    // the card is plain text, where N is upright too
    let words = |tex: &str| tex.replace(N_TEX, "N");
    Ok(NearN {
        note: format!("{N_TEX} = 10^{{{digits}}}"),
        labels: OFFSETS.map(|k| n_tex(BIG + k)).collect(),
        doubles,
        question: format!(r"\text{{What is }} {plus} \text{{ in doubles?}}"),
        landing,
        gap,
        gap_label: format!("{gap} apart"),
        answer: [
            format!(r"{plus} \to {at}"),
            format!(r"({plus}) - {} = {left} \text{{ in doubles}}", n_tex(BIG)),
        ],
        card: format!(
            "a double has {bits} significant bits, so above 2 to the {bits}, doubles are at least {gap} apart. \
             {} is halfway between doubles {} and {}; the tie goes to {}.",
            words(&plus),
            words(&at),
            words(&next),
            words(&at)
        ),
    })
}

/// One rounded example (10_4, 10_5) on its four tosses.
struct Example {
    /// C is the exact product of A and B.
    right: bool,
    /// The toss the work is shown for: the first whose verdict doubles change.
    shown: usize,
    exact: Vec<bool>,
    doubled: Vec<bool>,
    work: [String; 2],
}

/// The example `m` = [A, B, C], every column computed both ways. Refused
/// unless doubles change some verdict the wrong way: a right C's columns
/// stop matching, or a wrong C's start to.
fn example(m: &[Matrix<2>; 3]) -> Result<Example, String> {
    let [a, b, c] = m;
    let right = model::product(a, b).0 == *c;
    let tosses: Vec<RoundedToss> = model::tosses::<2>()
        .iter()
        .map(|t| facts::rounded_toss(a, b, c, t))
        .collect();
    let exact: Vec<bool> = tosses.iter().map(|rt| rt.exact.passes()).collect();
    let doubled: Vec<bool> = tosses.iter().map(RoundedToss::passes).collect();
    let shown = (0..tosses.len())
        .find(|&k| exact[k] != doubled[k])
        .ok_or("doubles change no verdict")?;
    let rt = &tosses[shown];
    let work = if right {
        if rt.exact.a_br != rt.exact.cr || rt.a_br == rt.cr {
            return Err("the right C must match exactly and differ in doubles".into());
        }
        [
            format!(
                r"\text{{exact: }} B\,r = {},\;\; A(B\,r) = {} = C\,r",
                tuple_tex(&rt.exact.br),
                tuple_tex(&rt.exact.a_br)
            ),
            format!(
                r"\text{{doubles: }} B\,r = {},\;\; A(B\,r) = {} \neq {} = C\,r",
                tuple_tex(&rt.br),
                tuple_tex(&rt.a_br),
                tuple_tex(&rt.cr)
            ),
        ]
    } else {
        if rt.exact.a_br == rt.exact.cr || rt.a_br != rt.cr {
            return Err("the wrong C must differ exactly and match in doubles".into());
        }
        [
            format!(
                r"\text{{exact: }} A(B\,r) = {} \neq {} = C\,r",
                tuple_tex(&rt.exact.a_br),
                tuple_tex(&rt.exact.cr)
            ),
            format!(r"\text{{doubles: }} A(B\,r) = {} = C\,r", tuple_tex(&rt.a_br)),
        ]
    };
    Ok(Example {
        right,
        shown,
        exact,
        doubled,
        work,
    })
}

/// A verdict word and its color: plain where it is right ("match" for a
/// right C, "no match", caught, for a wrong one), red where it is wrong.
fn verdict(matches: bool, right: bool) -> (&'static str, Srgb) {
    match (matches, right) {
        (true, true) => (MATCH, Palette::ink()),
        (false, false) => (NO_MATCH, palette::caught()),
        (true, false) => (MATCH, palette::mistake()),
        (false, true) => (NO_MATCH, palette::mistake()),
    }
}

/// The tolerance (10_6): the test, what it does to the right C in doubles
/// and to the wrong C in exact sums, and the exact version.
struct Tolerance {
    check: String,
    result: String,
    card: String,
}

/// The tolerance is the smallest that lets the right C through in doubles
/// (`facts::right_tolerance`, its largest residual there). Refused unless
/// with it the right C passes every toss in doubles and the wrong C every
/// toss in exact sums (the line says both).
fn tolerance() -> Result<Tolerance, String> {
    let tol = facts::right_tolerance();
    let tosses = model::tosses::<2>().len();
    let within = |residuals: &[i64]| residuals.iter().filter(|&&r| r <= tol).count();
    let right = within(facts::right_residuals().as_slice());
    let wrong = within(facts::wrong_residuals().as_slice());
    let [a, b, c] = ROUNDED_PASSES;
    if tol <= 0 || right != tosses || wrong != tosses || model::product(&a, &b).0 == c {
        return Err("the tolerance must let the right C through in doubles and the wrong C in exact sums".into());
    }
    Ok(Tolerance {
        check: format!(r"\left|\, A(B\,r) - C\,r \,\right| \le {tol} \text{{ in every row}}"),
        result: format!(
            "the right C passes {right} of {tosses} in doubles; this wrong C, {wrong} of {tosses} in exact sums"
        ),
        card: format!(
            "tolerance {tol}, the right C's largest gap in doubles, lets it through, but then this wrong C passes all \
             {wrong} tosses in exact sums: no half bound."
        ),
    })
}

/// The promise (11_1) and its exact version, from the theorem's bound.
/// Refused unless the bound is one half (the line says half) and a right C
/// passes every toss.
fn promise() -> Result<([String; 4], String), String> {
    let (num, den) = facts::theorem_bound()?;
    let ab = facts::true_product();
    if (num, den) != (1, 2) || model::passing(&A, &B, &ab).len() != model::tosses::<SIDE>().len() {
        return Err("the promise needs half a toss and a right C passing every toss".into());
    }
    Ok((
        [
            RIGHT_LINE.to_owned(),
            WRONG_LINE.to_owned(),
            format!(r"k \text{{ fresh tosses all miss: }} \le \frac{{{num}}}{{{den}^k}}"),
            BOTH_FAIL.to_owned(),
        ],
        format!(
            "in exact arithmetic, with fair coins, all independent and tossed after C is fixed, k tosses all miss \
             any fixed wrong C with chance at most {num}/{den} to the k."
        ),
    ))
}

/// Back to the start (11_3, 11_4) and the billion (11_5, 11_6), from the
/// counted work and the theorem's bound.
struct Recap {
    redo: String,
    one_toss: String,
    scale: [String; 2],
    question: String,
    answer: [String; 2],
}

fn recap() -> Result<Recap, String> {
    let (_, redo) = model::product(&A, &B);
    let first = facts::first_check();
    let rows = first.disagreements();
    let [row] = rows[..] else {
        return Err("the first toss must catch the slip in one row".into());
    };
    let big_redo = model::product_multiplications(BIG_SIDE);
    let twenty = u64::from(AGAIN) * model::check_multiplications(BIG_SIDE);
    let odds = facts::again_odds()?;
    let (k, target_odds) = facts::checks_below(BILLION)?;
    let target_mults = u64::from(k) * model::check_multiplications(BIG_SIDE);
    let share = facts::ratio(target_mults as i64, big_redo as i64)?;
    Ok(Recap {
        redo: format!("redoing it: {redo} multiplications"),
        one_toss: format!("one toss: {}, caught in row {}", first.multiplications, row + 1),
        scale: [
            format!(
                r"n = {BIG_SIDE}:\;\; {AGAIN} \text{{ checks take }} {} \text{{ of }} {} \text{{ multiplications}}",
                tex_thousands(twenty),
                tex_thousands(big_redo)
            ),
            format!(
                r"\text{{a wrong }} C \text{{ passes all {AGAIN}: }} \le \frac{{1}}{{{}}} < \frac{{1}}{{{}}}",
                tex_thousands(odds),
                tex_thousands(facts::MILLION)
            ),
        ],
        question: format!(
            "How many fresh checks for below 1 in {}?",
            explainer_kit::thousands(BILLION)
        ),
        answer: [
            format!(
                r"2^{{{k}}} = {} > {}",
                tex_thousands(target_odds),
                tex_thousands(BILLION)
            ),
            format!(
                r"{k} \times n^2 = {} = {} \text{{ of }} n^3",
                tex_thousands(target_mults),
                tex_of(&share)
            ),
        ],
    })
}

/// A bar `w` long at height `y`, from the bars' left edge. Detached.
fn bar(stage: &mut Stage<'_>, y: f64, w: f64, style: Style) -> fmn::Result<Mob> {
    let r = rect(v(BAR_X, y - BAR_H / 2.0), w, BAR_H, style)?;
    Ok(stage.arena_mut().add(r))
}

/// A filled dot at `at`. Detached.
fn dot(stage: &mut Stage<'_>, at: Vec3, color: Srgb) -> Mob {
    stage.arena_mut().add(
        Circle::new()
            .radius(DOT_R)
            .arc_center(at)
            .style(flat(color, 1.0))
            .build(),
    )
}

/// The number line around N: its base, a tick at each whole number, and
/// their `labels`. Detached.
fn number_line(kit: &Kit, stage: &mut Stage<'_>, labels: &[String]) -> fmn::Result<Vec<Mob>> {
    let base = polyline(vec![v(-LINE_HALF, LINE_Y), v(LINE_HALF, LINE_Y)])
        .style(line_style(Palette::dim(), 2.0))
        .build();
    let mut all = vec![stage.arena_mut().add(base)];
    for (k, label) in OFFSETS.zip(labels) {
        let x = line_x(k);
        let tick = polyline(vec![v(x, LINE_Y - TICK_H / 2.0), v(x, LINE_Y + TICK_H / 2.0)])
            .style(line_style(Palette::dim(), 2.0))
            .build();
        all.push(stage.arena_mut().add(tick));
        all.push(kit.tex(stage, label, 26.0, Palette::ink(), &[], v(x, LINE_Y + LINE_LABEL_DY))?);
    }
    Ok(all)
}

/// Example `m`'s grids, A, B and C with N in their entries and the mistake
/// C − AB (zeros dim, wrong entries outlined), all detached.
fn example_grids(kit: &Kit, stage: &mut Stage<'_>, m: &[Matrix<2>; 3]) -> fmn::Result<Vec<Mob>> {
    let [a, b, c] = m;
    let mut all = Vec::new();
    for (matrix, &x) in [a, b, c].into_iter().zip(&GRID_XS) {
        let values = matrix.iter().map(|row| row.to_vec()).collect();
        all.extend(tex_grid(kit, stage, values, v(x, GRIDS_Y), Palette::ink(), n_tex)?.mobs());
    }
    let d = model::mistake(c, &model::product(a, b).0);
    let values = d.iter().map(|row| row.to_vec()).collect();
    let g = grid_colored(kit, stage, values, v(GRID_XS[3], GRIDS_Y), |x| {
        if x == 0 { Palette::dim() } else { Palette::ink() }
    })?;
    all.extend(g.mobs());
    for (i, j) in model::wrong_entries(&d) {
        all.push(cell_box(stage, &g, i, j, palette::mistake())?);
    }
    Ok(all)
}

/// One verdict row for `matches` of a right or wrong C, at height `y`.
/// Detached.
fn verdict_row(kit: &Kit, stage: &mut Stage<'_>, matches: &[bool], right: bool, y: f64) -> fmn::Result<Vec<Mob>> {
    matches
        .iter()
        .zip(&TOSS_XS)
        .map(|(&m, &x)| {
            let (word, color) = verdict(m, right);
            kit.text(stage, word, 22.0, color, v(x, y))
        })
        .collect()
}

impl SceneConstruct for WhatItCosts {
    fn name(&self) -> &str {
        "09_what_it_costs"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let cost = cost().map_err(fail)?;
        let small = break_even().map_err(fail)?;
        let code = code_lines();
        let code_refs: Vec<&str> = code.iter().map(String::as_str).collect();

        // 09_1: one check at a thousand rows, a thousandth of the product
        let line = say(&kit, stage, "09_1")?;
        let n_label = kit.tex(stage, &cost.n, 34.0, Palette::ink(), &[], COST_N_AT)?;
        let redo_label = kit.tex_left(stage, &cost.redo, 28.0, Palette::ink(), &[], v(BAR_X, LABEL_YS[0]))?;
        let redo_track = bar(stage, BAR_YS[0], BAR_W, flat(Palette::faint(), 0.6))?;
        let redo_fill = bar(stage, BAR_YS[0], BAR_W, flat(Palette::dim(), 0.9))?;
        fade_in_all(stage, &[n_label, redo_label, redo_track, redo_fill], 0.5)?;
        line.wait_for(stage, "one check takes")?;
        let one_label = kit.tex_left(stage, &cost.one, 28.0, Palette::ink(), &[], v(BAR_X, LABEL_YS[1]))?;
        let one_track = bar(stage, BAR_YS[1], BAR_W, flat(Palette::faint(), 0.6))?;
        let one_fill = bar(
            stage,
            BAR_YS[1],
            (BAR_W * cost.shares[0]).max(MIN_BAR_W),
            flat(palette::matrix_a(), 0.95),
        )?;
        let card = lower_third(&kit, stage, PROGRAMMERS, COST_CARD)?;
        fade_in_all(stage, &[one_label, one_track, one_fill, card], 0.5)?;
        let card_up = now(stage);
        line.wait_for(stage, "a thousandth")?;
        let ratio = kit.tex(stage, &cost.one_ratio, 32.0, Palette::ink(), &[], RATIO_AT)?;
        fade_in_all(stage, &[ratio], 0.4)?;

        // 09_2: twenty fresh checks, a fiftieth; work, not seconds
        let line = say(&kit, stage, "09_2")?;
        let twenty_label = kit.tex_left(stage, &cost.twenty, 28.0, Palette::ink(), &[], v(BAR_X, LABEL_YS[2]))?;
        let twenty_track = bar(stage, BAR_YS[2], BAR_W, flat(Palette::faint(), 0.6))?;
        let twenty_fill = bar(
            stage,
            BAR_YS[2],
            BAR_W * cost.shares[1],
            flat(palette::matrix_a(), 0.95),
        )?;
        fade_in_all(stage, &[twenty_label, twenty_track, twenty_fill], 0.5)?;
        line.wait_for(stage, "a fiftieth")?;
        let twenty_ratio = kit.tex(stage, &cost.twenty_ratio, 32.0, Palette::ink(), &[], RATIO_AT)?;
        swap(stage, &[ratio], &[twenty_ratio], 0.4)?;

        // 09_3: the work also includes additions and memory traffic: the
        // card is read with the line, then clears before the question
        say(&kit, stage, "09_3")?;
        kit.hold(stage)?;
        wait_until(stage, card_up + read_s(COST_CARD))?;
        fade_out_all(stage, &[card], 0.3)?;

        // 09_4: three by three: how many checks cost as much? The think
        // time, with nothing stacked
        let line = say(&kit, stage, "09_4")?;
        let leaving = [
            n_label,
            redo_label,
            redo_track,
            redo_fill,
            one_label,
            one_track,
            one_fill,
            twenty_label,
            twenty_track,
            twenty_fill,
            twenty_ratio,
        ];
        fade_out_all(stage, &leaving, 0.4)?;
        let small_n = kit.tex(stage, &small.n, 34.0, Palette::ink(), &[], COST_N_AT)?;
        let small_redo = kit.tex_left(stage, &small.redo, 28.0, Palette::ink(), &[], v(BAR_X, LABEL_YS[0]))?;
        let small_redo_bar = bar(stage, BAR_YS[0], BAR_W, flat(Palette::dim(), 0.9))?;
        let small_one = kit.tex_left(stage, &small.one, 28.0, Palette::ink(), &[], v(BAR_X, LABEL_YS[1]))?;
        let small_one_bar = bar(
            stage,
            BAR_YS[1],
            BAR_W / small.checks as f64,
            flat(palette::matrix_a(), 0.95),
        )?;
        fade_in_all(
            stage,
            &[small_n, small_redo, small_redo_bar, small_one, small_one_bar],
            0.5,
        )?;
        line.wait_for(stage, "how many checks")?;
        let question = kit.text(stage, BREAK_QUESTION, 24.0, Palette::ink(), BREAK_AT)?;
        fade_in_all(stage, &[question], 0.4)?;
        let q = think(&kit, stage, BREAK_THINK_AT)?;

        // 09_5: three, stacked end to end; the share is checks over rows
        let line = say(&kit, stage, "09_5")?;
        fade_out_all(stage, &[q, question], 0.2)?;
        let piece = BAR_W / small.checks as f64;
        let mut stacked = Vec::with_capacity(small.checks as usize);
        for k in 0..small.checks {
            stacked.push(stage.arena_mut().add(rect(
                v(BAR_X + piece * k as f64, BAR_YS[2] - BAR_H / 2.0),
                piece - 0.04,
                BAR_H,
                flat(palette::matrix_a(), 0.95),
            )?));
        }
        let stacked_label = kit.tex_left(stage, &small.stacked, 28.0, Palette::ink(), &[], v(BAR_X, LABEL_YS[2]))?;
        fade_in_lagged(stage, &stacked, 0.9, 0.4)?;
        fade_in_all(stage, &[stacked_label], 0.3)?;
        line.wait_for(stage, "The bigger the matrices")?;
        let share = [
            kit.tex(stage, &small.rule, 30.0, Palette::ink(), &[], RATIO_AT)?,
            kit.tex(stage, &small.examples, 28.0, Palette::ink(), &[], SHARE_AT)?,
        ];
        fade_in_lagged(stage, &share, 0.8, 0.4)?;

        // 09_6, 09_7: the whole check, as a short loop
        let line = say(&kit, stage, "09_6")?;
        let mut leaving = vec![
            small_n,
            small_redo,
            small_redo_bar,
            small_one,
            small_one_bar,
            stacked_label,
        ];
        leaving.extend(stacked);
        leaving.extend(share);
        fade_out_all(stage, &leaving, 0.4)?;
        let panel = CodePanel::build(&kit, stage, &code_refs, CODE_SIZE, CODE_W)?;
        stage.play(panel.slide_in(0.8)?)?;
        for (phrase, at) in [
            ("Draw fresh coins", 1),
            ("Add B's heads columns", 2),
            ("multiply by A", 3),
            ("and add C's heads columns", 4),
        ] {
            line.wait_for(stage, phrase)?;
            flash_all(stage, &[panel.lines[at]], 0.5)?;
        }
        let line = say(&kit, stage, "09_7")?;
        flash_all(stage, &[panel.lines[5]], 0.5)?;
        line.wait_for(stage, "Otherwise, repeat")?;
        flash_all(stage, &[panel.lines[0], panel.lines[6]], 0.6)?;
        let up = now(stage);
        stand(&kit, stage, up)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

impl SceneConstruct for ExactAndRounded {
    fn name(&self) -> &str {
        "10_exact_and_rounded"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let near = near_n().map_err(fail)?;
        let reject = example(&ROUNDED_REJECTS).map_err(fail)?;
        let pass = example(&ROUNDED_PASSES).map_err(fail)?;
        let tol = tolerance().map_err(fail)?;
        if !reject.right || pass.right {
            return Err(fail("the first example must be right and the second wrong".into()));
        }

        // 10_1: the pairing proof needs exact sums
        say(&kit, stage, "10_1")?;
        let step = [
            kit.text(stage, STEP_LINES[0], 26.0, Palette::ink(), v(0.0, STEP_YS[0]))?,
            kit.text(stage, STEP_LINES[1], 26.0, Palette::hero(), v(0.0, STEP_YS[1]))?,
        ];
        fade_in_lagged(stage, &step, 0.8, 0.5)?;

        // 10_2: doubles skip whole numbers; what is N + 1 in doubles? The
        // think time, with no double marked
        let line = say(&kit, stage, "10_2")?;
        fade_out_all(stage, &step, 0.4)?;
        let n_note = kit.tex(stage, &near.note, 28.0, Palette::ink(), &[], N_NOTE_AT)?;
        let the_line = number_line(&kit, stage, &near.labels)?;
        let mut arriving = vec![n_note];
        arriving.extend(&the_line);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, "What is ten to the sixteen")?;
        let question = kit.tex(stage, &near.question, 30.0, Palette::ink(), &[], QUESTION_AT)?;
        fade_in_all(stage, &[question], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 10_3: N, written as it is said, so the answer stands through the
        // line; then the doubles there two apart, and N + 1 landing on N
        let line = say(&kit, stage, "10_3")?;
        let answer = [
            kit.tex(stage, &near.answer[0], 30.0, palette::mistake(), &[], QUESTION_AT)?,
            kit.tex(
                stage,
                &near.answer[1],
                26.0,
                Palette::ink(),
                &[],
                v(QUESTION_AT[0], QUESTION_AT[1] + ANSWER_DY),
            )?,
        ];
        fade_out_all(stage, &[q], 0.2)?;
        fade_out_all(stage, &[question], 0.2)?;
        fade_in_all(stage, &answer, 0.2)?;
        line.wait_for(stage, "Doubles there are two apart")?;
        let mut marks: Vec<Mob> = near
            .doubles
            .iter()
            .map(|&k| dot(stage, v(line_x(k), LINE_Y), palette::heads()))
            .collect();
        marks.push(kit.text_left(stage, DOUBLES, 22.0, palette::heads(), LEGEND_AT)?);
        marks.push(kit.text(
            stage,
            &near.gap_label,
            22.0,
            palette::heads(),
            v(line_x(near.gap) / 2.0, LINE_Y + GAP_LABEL_DY),
        )?);
        fade_in_all(stage, &marks, 0.5)?;
        let gap_card = precisely(&kit, stage, &near.card)?;
        fade_in_all(stage, &[gap_card], 0.2)?;
        let gap_up = now(stage);
        line.wait_for(stage, "so the one is lost")?;
        let lost = dot(stage, v(line_x(PLUS), LINE_Y), palette::mistake());
        fade_in_all(stage, &[lost], 0.3)?;
        together(stage, vec![fly(lost, v(line_x(near.landing), LINE_Y), 0.8)?])?;

        // 10_4: the right C, caught on the toss doubles get wrong
        let line = say(&kit, stage, "10_4")?;
        let mut leaving = the_line;
        leaving.extend(marks);
        leaving.push(lost);
        leaving.extend(answer);
        fade_out_all(stage, &leaving, 0.5)?;
        let grids = example_grids(&kit, stage, &ROUNDED_REJECTS)?;
        let mut names = Vec::with_capacity(GRID_XS.len());
        let colors = [palette::matrix_a(), palette::matrix_b(), palette::claim()];
        for ((name, color), &x) in NAMES.iter().zip(colors).zip(&GRID_XS) {
            names.push(kit.tex(stage, name, 30.0, color, &[], v(x, GRID_NAME_Y))?);
        }
        names.push(kit.tex(
            stage,
            MISTAKE_NAME,
            30.0,
            palette::mistake(),
            &[],
            v(GRID_XS[3], GRID_NAME_Y),
        )?);
        let mut coins = Vec::new();
        for (t, &x) in model::tosses::<2>().iter().zip(&TOSS_XS) {
            let xs = [x - COIN_PITCH / 2.0, x + COIN_PITCH / 2.0];
            coins.extend(coin_mobs(&toss_coins(&kit, stage, t, &xs, COIN_Y)?));
        }
        let row_labels = [
            kit.text_left(stage, EXACT, 22.0, Palette::dim(), v(ROW_LABEL_X, ROW_YS[0]))?,
            kit.text_left(stage, DOUBLES, 22.0, Palette::dim(), v(ROW_LABEL_X, ROW_YS[1]))?,
        ];
        let mut arriving = grids.clone();
        arriving.extend(&names);
        arriving.extend(&coins);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, "This C is A times B")?;
        let mut exact_row = verdict_row(&kit, stage, &reject.exact, reject.right, ROW_YS[0])?;
        exact_row.insert(0, row_labels[0]);
        fade_in_lagged(stage, &exact_row, 0.8, 0.2)?;
        line.wait_for(stage, "with both coins heads")?;
        let mut doubles_row = verdict_row(&kit, stage, &reject.doubled, reject.right, ROW_YS[1])?;
        doubles_row.insert(0, row_labels[1]);
        fade_in_lagged(stage, &doubles_row, 0.8, 0.2)?;
        let shown = ring(stage, v(TOSS_XS[reject.shown], COIN_Y), SHOWN_R, Palette::hero());
        let work = [
            kit.tex(stage, &reject.work[0], 26.0, Palette::ink(), &[], v(0.0, WORK_YS[0]))?,
            kit.tex(
                stage,
                &reject.work[1],
                26.0,
                palette::mistake(),
                &[],
                v(0.0, WORK_YS[1]),
            )?,
        ];
        fade_in_all(stage, &[shown], 0.3)?;
        fade_in_lagged(stage, &work, 0.8, 0.4)?;

        // 10_5: the wrong C by one, passing every toss in doubles
        let line = say(&kit, stage, "10_5")?;
        wait_until(stage, gap_up + read_s(&near.card))?;
        let next_grids = example_grids(&kit, stage, &ROUNDED_PASSES)?;
        let mut old = grids;
        old.extend(exact_row.iter().skip(1));
        old.extend(doubles_row.iter().skip(1));
        old.push(shown);
        old.extend(work);
        old.push(gap_card);
        fade_out_all(stage, &old, 0.25)?;
        fade_in_all(stage, &next_grids, 0.25)?;
        line.wait_for(stage, "This C is too small by one")?;
        let exact_row = verdict_row(&kit, stage, &pass.exact, pass.right, ROW_YS[0])?;
        fade_in_lagged(stage, &exact_row, 0.8, 0.2)?;
        line.wait_for(stage, "all four tosses miss it")?;
        let doubles_row = verdict_row(&kit, stage, &pass.doubled, pass.right, ROW_YS[1])?;
        fade_in_lagged(stage, &doubles_row, 0.8, 0.2)?;
        let shown = ring(stage, v(TOSS_XS[pass.shown], COIN_Y), SHOWN_R, Palette::hero());
        let work = [
            kit.tex(stage, &pass.work[0], 26.0, Palette::ink(), &[], v(0.0, WORK_YS[0]))?,
            kit.tex(stage, &pass.work[1], 26.0, palette::mistake(), &[], v(0.0, WORK_YS[1]))?,
        ];
        fade_in_all(stage, &[shown], 0.3)?;
        fade_in_lagged(stage, &work, 0.8, 0.4)?;

        // 10_6: a tolerance, and the wrong C inside it
        let line = say(&kit, stage, "10_6")?;
        fade_out_all(stage, &work, 0.3)?;
        line.wait_for(stage, "but then a wrong C inside it")?;
        // the rows above are the test of equality; dimmed, they step back
        // for the tolerant test, whose verdicts differ from theirs
        let mut dims = Vec::new();
        for &m in exact_row.iter().chain(&doubles_row).chain(&row_labels) {
            dims.push(fade_to(m, EQUALITY_DIM, 0.4)?);
        }
        together(stage, dims)?;
        let tol_lines = [
            kit.tex(stage, &tol.check, 28.0, Palette::ink(), &[], v(0.0, WORK_YS[0]))?,
            kit.text(stage, &tol.result, 22.0, Palette::ink(), v(0.0, WORK_YS[1]))?,
        ];
        let tol_card = precisely(&kit, stage, &tol.card)?;
        let mut arriving = tol_lines.to_vec();
        arriving.push(tol_card);
        fade_in_all(stage, &arriving, 0.4)?;
        let tol_up = now(stage);
        stand(&kit, stage, tol_up)?;
        wait_until(stage, tol_up + read_s(&tol.card))?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

impl SceneConstruct for ThePromise {
    fn name(&self) -> &str {
        "11_the_promise"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (lines, last_card) = promise().map_err(fail)?;
        let words = recap().map_err(fail)?;
        let three = facts::shape(PROOF_SHAPE).map_err(fail)?;
        let (row, column) = facts::proof_entry().map_err(fail)?;
        let mut flipped = HIDING_TOSS;
        flipped[column] = !flipped[column];
        if !model::hides(&three, &HIDING_TOSS) || model::hides(&three, &flipped) || three[row][column] == 0 {
            return Err(fail(
                "the last toss must hide the mistake, and its flip catch it".into(),
            ));
        }

        // 11_1: the promise, on the opening picture
        let line = say(&kit, stage, "11_1")?;
        let open = opening(&kit, stage, &C)?;
        let (wrong_row, wrong_col) = facts::wrong_entry().map_err(fail)?;
        let slip_box = cell_box(stage, &open.c, wrong_row, wrong_col, palette::mistake())?;
        let mut arriving = open.mobs();
        arriving.push(slip_box);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, "Right answers")?;
        let first = kit.text(stage, &lines[0], 24.0, Palette::ink(), v(PROMISE_X, PROMISE_YS[0]))?;
        fade_in_all(stage, &[first], 0.4)?;
        line.wait_for(stage, "a wrong one gets past")?;
        let rest = [
            kit.text(stage, &lines[1], 24.0, Palette::ink(), v(PROMISE_X, PROMISE_YS[1]))?,
            kit.tex(stage, &lines[2], 28.0, Palette::ink(), &[], v(PROMISE_X, PROMISE_YS[2]))?,
            kit.text(stage, &lines[3], 22.0, Palette::dim(), v(PROMISE_X, PROMISE_YS[3]))?,
        ];
        let last = precisely(&kit, stage, &last_card)?;
        fade_in_lagged(stage, &rest, 1.0, 0.4)?;
        fade_in_all(stage, &[last], 0.3)?;
        let last_up = now(stage);

        // 11_2: what the promise assumes
        say(&kit, stage, "11_2")?;
        let mut assumes = Vec::with_capacity(ASSUMES.len());
        for (text, &y) in ASSUMES.iter().zip(&ASSUME_YS) {
            assumes.push(kit.text_left(stage, text, 22.0, Palette::hero(), v(ASSUME_X, y))?);
        }
        fade_in_lagged(stage, &assumes, 1.2, 0.5)?;

        // 11_3: back to the start: twenty-seven, and one toss of nine that
        // caught the slip
        let line = say(&kit, stage, "11_3")?;
        wait_until(stage, last_up + read_s(&last_card))?;
        let mut leaving = vec![first, last];
        leaving.extend(rest);
        leaving.extend(assumes);
        fade_out_all(stage, &leaving, 0.4)?;
        let redo = kit.text(stage, &words.redo, 22.0, Palette::ink(), v(PANEL_X, START_YS[0]))?;
        fade_in_all(stage, &[redo], 0.4)?;
        line.wait_for(stage, "One toss took nine")?;
        let xs: Vec<f64> = (0..SIDE).map(|j| open.c.at(0, j)[0]).collect();
        let coins = toss_coins(&kit, stage, &FIRST_TOSS, &xs, OPEN_COIN_Y)?;
        fade_in_lagged(stage, &coin_mobs(&coins), 0.6, 0.3)?;
        let kept = keep_columns(stage, &open.c, &FIRST_TOSS, 1.0)?;
        let one = kit.text(stage, &words.one_toss, 22.0, palette::caught(), v(PANEL_X, START_YS[1]))?;
        fade_in_all(stage, &[one], 0.4)?;

        // 11_4: at a thousand rows, twenty fresh checks
        say(&kit, stage, "11_4")?;
        let scale = [
            kit.tex(stage, &words.scale[0], 24.0, Palette::ink(), &[], v(0.0, SCALE_YS[0]))?,
            kit.tex(stage, &words.scale[1], 26.0, Palette::ink(), &[], v(0.0, SCALE_YS[1]))?,
        ];
        fade_in_lagged(stage, &scale, 0.8, 0.5)?;

        // 11_5: how many fresh checks for one in a billion? The think time
        let line = say(&kit, stage, "11_5")?;
        line.wait_for(stage, "How many fresh checks")?;
        let question = kit.text(stage, &words.question, 24.0, Palette::ink(), BILLION_AT)?;
        fade_in_all(stage, &[question], 0.4)?;
        let q = think(&kit, stage, BILLION_THINK_AT)?;

        // 11_6: thirty; still three hundredths of the product
        let line = say(&kit, stage, "11_6")?;
        fade_out_all(stage, &[q], 0.2)?;
        let answer = [
            kit.tex(stage, &words.answer[0], 28.0, Palette::ink(), &[], BILLION_AT)?,
            kit.tex(
                stage,
                &words.answer[1],
                28.0,
                Palette::ink(),
                &[],
                v(BILLION_AT[0], BILLION_AT[1] + BILLION_DY),
            )?,
        ];
        swap(stage, &[question], &[answer[0]], 0.3)?;
        line.wait_for(stage, "thirty checks still take")?;
        fade_in_all(stage, &[answer[1]], 0.4)?;

        // 11_7: a mistake hides only when its heads columns add to zero; the
        // coin over a nonzero entry of the mistake breaks that
        let line = say(&kit, stage, "11_7")?;
        let mut leaving = open.mobs();
        leaving.extend([slip_box, redo, one]);
        leaving.extend(coin_mobs(&coins));
        leaving.extend(kept);
        leaving.extend(scale);
        leaving.extend(answer);
        fade_out_all(stage, &leaving, 0.5)?;
        let d = matrix_grid(&kit, stage, &three, LAST_D_AT, Palette::ink())?;
        let mut boxes = Vec::new();
        for (i, j) in model::wrong_entries(&three) {
            boxes.push(cell_box(stage, &d, i, j, palette::mistake())?);
        }
        let d_name = kit.tex(
            stage,
            MISTAKE_NAME,
            30.0,
            palette::mistake(),
            &[],
            v(LAST_D_AT[0], LAST_D_AT[1] + LAST_NAME_DY),
        )?;
        let coin_xs: Vec<f64> = (0..SIDE).map(|j| d.at(0, j)[0]).collect();
        let coin_y = LAST_D_AT[1] + LAST_COIN_DY;
        let last_coins = toss_coins(&kit, stage, &HIDING_TOSS, &coin_xs, coin_y)?;
        let sums = column_grid(
            &kit,
            stage,
            &model::kept_sum(&three, &HIDING_TOSS),
            v(LAST_SUM_X, LAST_D_AT[1]),
            Palette::ink(),
        )?;
        let hides = kit.text_left(stage, HIDES, 24.0, palette::mistake(), v(LAST_WORD_X, LAST_D_AT[1]))?;
        let mut arriving = d.mobs();
        arriving.extend(&boxes);
        arriving.push(d_name);
        arriving.extend(coin_mobs(&last_coins));
        arriving.extend(sums.mobs());
        arriving.push(hides);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, "Flipping the coin")?;
        let turned = coin(&kit, stage, flipped[column], v(coin_xs[column], coin_y))?;
        let ring_on = ring(stage, v(coin_xs[column], coin_y), COIN_R + 0.07, Palette::hero());
        swap(stage, &last_coins[column].mobs(), &turned.mobs(), 0.4)?;
        fade_in_all(stage, &[ring_on], 0.3)?;
        let flipped_sums = column_grid(
            &kit,
            stage,
            &model::kept_sum(&three, &flipped),
            v(LAST_SUM_X, LAST_D_AT[1]),
            Palette::ink(),
        )?;
        let moved = cell_box(stage, &flipped_sums, row, 0, palette::caught())?;
        let caught = kit.text_left(stage, CAUGHT, 24.0, palette::caught(), v(LAST_WORD_X, LAST_D_AT[1]))?;
        swap(stage, &sums.mobs(), &flipped_sums.mobs(), 0.4)?;
        swap(stage, &[hides], &[caught], 0.3)?;
        fade_in_all(stage, &[moved], 0.3)?;
        // the names to search and the receipts, once the line has ended
        kit.hold(stage)?;
        // only what is on screen: the coin and the sums swapped out above
        // are gone already, and fading them again would bring them back
        let mut leaving = d.mobs();
        leaving.extend(&boxes);
        leaving.extend([d_name, ring_on, moved, caught]);
        for (j, c) in last_coins.iter().enumerate() {
            if j != column {
                leaving.extend(c.mobs());
            }
        }
        leaving.extend(turned.mobs());
        leaving.extend(flipped_sums.mobs());
        fade_out_all(stage, &leaving, 0.5)?;
        let receipts = receipts_card(&kit, stage, &RECEIPTS)?;
        let names = names_to_search(&kit, stage, &SEARCH)?;
        fade_in_all(stage, &[receipts, names], 0.6)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        wait_until(stage, up + read_s(&RECEIPTS.join(" ")).max(read_s(&SEARCH.join(" · "))))?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The cost is the counted work, in the row-by-column convention, and
    /// the bars stand to scale.
    #[test]
    fn the_cost_is_counted_work() {
        let c = cost().unwrap();
        assert_eq!(c.n, "n = 1000");
        assert_eq!(
            c.redo,
            r"\text{redo the product, row by column: } n^3 = 1{,}000{,}000{,}000"
        );
        assert_eq!(c.one, r"\text{1 check: } n^2 = 1{,}000{,}000");
        assert_eq!(c.twenty, r"\text{20 fresh checks: } 20 \times n^2 = 20{,}000{,}000");
        assert_eq!(
            c.one_ratio,
            r"\frac{1{,}000{,}000}{1{,}000{,}000{,}000} = \frac{1}{1000}"
        );
        assert_eq!(
            c.twenty_ratio,
            r"\frac{20{,}000{,}000}{1{,}000{,}000{,}000} = \frac{1}{50}"
        );
        assert!((c.shares[1] * 50.0 - 1.0).abs() < 1e-12, "{:?}", c.shares);
        // one check's bar is about a pixel, twenty checks' a fiftieth
        assert!(BAR_W * c.shares[0] >= MIN_BAR_W - 1e-12);
        assert!(BAR_W * c.shares[1] > 0.1);
        let b = break_even().unwrap();
        assert_eq!(b.n, "n = 3");
        assert_eq!(b.redo, r"\text{redo: } 3^3 = 27");
        assert_eq!(b.one, r"\text{1 check: } 3^2 = 9");
        assert_eq!(b.stacked, r"3 \times 9 = 27");
        assert_eq!(b.examples, r"\frac{3}{3} = 1,\;\;\; \frac{20}{1000} = \frac{1}{50}");
        assert_eq!(b.checks, 3);
    }

    /// The loop is plain ASCII for the typewriter face, short enough for the
    /// panel, and repeats as many times as the ladder says.
    #[test]
    fn the_code_is_plain_and_fits() {
        let code = code_lines();
        assert_eq!(code.len(), 7);
        assert!(code[0].contains(&AGAIN.to_string()));
        for line in &code {
            assert!(line.is_ascii(), "{line}");
            assert!(line.len() <= 44, "{line}");
        }
    }

    /// N + 1 lands on N in doubles, the doubles near N two apart, all
    /// computed, with N upright everywhere (review F8).
    #[test]
    fn n_plus_one_is_lost_in_doubles() {
        let n = near_n().unwrap();
        assert_eq!(n.note, r"\text{N} = 10^{16}");
        assert_eq!(
            n.labels,
            [
                r"\text{N} - 2",
                r"\text{N} - 1",
                r"\text{N}",
                r"\text{N} + 1",
                r"\text{N} + 2"
            ]
        );
        assert_eq!(n.doubles, [-2, 0, 2]);
        assert_eq!(n.question, r"\text{What is } \text{N} + 1 \text{ in doubles?}");
        assert_eq!((n.landing, n.gap), (0, 2));
        assert_eq!(n.gap_label, "2 apart");
        assert_eq!(
            n.answer,
            [
                r"\text{N} + 1 \to \text{N}",
                r"(\text{N} + 1) - \text{N} = 0 \text{ in doubles}"
            ]
        );
        assert_eq!(n_tex(1 - BIG), r"-\text{N} + 1");
        assert_eq!(n_tex(2), "2");
    }

    /// In doubles the right C is caught on toss 11 and the wrong C passes
    /// every toss; exact sums keep both promises.
    #[test]
    fn the_examples_break_both_promises_in_doubles() {
        let r = example(&ROUNDED_REJECTS).unwrap();
        assert!(r.right);
        assert_eq!(r.shown, 3);
        assert_eq!(r.exact, [true; 4]);
        assert_eq!(r.doubled, [true, true, true, false]);
        assert_eq!(
            r.work,
            [
                r"\text{exact: } B\,r = (\text{N} + 1,\; -\text{N} + 1),\;\; A(B\,r) = (2,\; 0) = C\,r",
                r"\text{doubles: } B\,r = (\text{N},\; -\text{N}),\;\; A(B\,r) = (0,\; 0) \neq (2,\; 0) = C\,r",
            ]
        );
        let p = example(&ROUNDED_PASSES).unwrap();
        assert!(!p.right);
        assert_eq!(p.shown, 2);
        assert_eq!(p.exact, [true, true, false, false]);
        assert_eq!(p.doubled, [true; 4]);
        assert_eq!(
            p.work,
            [
                r"\text{exact: } A(B\,r) = (\text{N} + 1,\; 0) \neq (\text{N},\; 0) = C\,r",
                r"\text{doubles: } A(B\,r) = (\text{N},\; 0) = C\,r",
            ]
        );
        assert_eq!(verdict(false, true).1, palette::mistake());
        assert_eq!(verdict(true, false).1, palette::mistake());
        assert_eq!(verdict(false, false), (NO_MATCH, palette::caught()));
    }

    /// The tolerance is the right C's largest gap in doubles: it lets the
    /// right C through on every toss in doubles, and the wrong C on every
    /// toss even in exact sums.
    #[test]
    fn a_tolerance_passes_the_wrong_c() {
        assert_eq!(facts::right_residuals(), [0, 0, 0, 2]);
        assert_eq!(facts::wrong_residuals(), [0, 0, 1, 1]);
        assert_eq!(facts::right_tolerance(), 2);
        let t = tolerance().unwrap();
        assert_eq!(t.check, r"\left|\, A(B\,r) - C\,r \,\right| \le 2 \text{ in every row}");
        assert_eq!(
            t.result,
            "the right C passes 4 of 4 in doubles; this wrong C, 4 of 4 in exact sums"
        );
    }

    /// Back to the start and the billion, from the counted work and the
    /// theorem's bound.
    #[test]
    fn the_recap_is_built_from_the_facts() {
        let r = recap().unwrap();
        assert_eq!(r.redo, "redoing it: 27 multiplications");
        assert_eq!(r.one_toss, "one toss: 9, caught in row 2");
        assert_eq!(
            r.scale[0],
            r"n = 1000:\;\; 20 \text{ checks take } 20{,}000{,}000 \text{ of } 1{,}000{,}000{,}000 \text{ multiplications}"
        );
        assert_eq!(
            r.scale[1],
            r"\text{a wrong } C \text{ passes all 20: } \le \frac{1}{1{,}048{,}576} < \frac{1}{1{,}000{,}000}"
        );
        assert_eq!(r.question, "How many fresh checks for below 1 in 1,000,000,000?");
        assert_eq!(
            r.answer,
            [
                r"2^{30} = 1{,}073{,}741{,}824 > 1{,}000{,}000{,}000",
                r"30 \times n^2 = 30{,}000{,}000 = \frac{3}{100} \text{ of } n^3"
            ]
        );
    }

    /// The promise is the theorem's, and every card keeps its conditions in
    /// two lines that can be read; the cost card states its convention and
    /// no longer claims every entry is read.
    #[test]
    fn the_cards_keep_their_conditions() {
        let (lines, last) = promise().unwrap();
        assert_eq!(lines[2], r"k \text{ fresh tosses all miss: } \le \frac{1}{2^k}");
        assert_eq!(
            last,
            "in exact arithmetic, with fair coins, all independent and tossed after C is fixed, k tosses all miss \
             any fixed wrong C with chance at most 1/2 to the k."
        );
        let n = near_n().unwrap();
        assert_eq!(
            n.card,
            "a double has 53 significant bits, so above 2 to the 53, doubles are at least 2 apart. N + 1 is \
             halfway between doubles N and N + 2; the tie goes to N."
        );
        let t = tolerance().unwrap();
        assert_eq!(
            t.card,
            "tolerance 2, the right C's largest gap in doubles, lets it through, but then this wrong C passes all 4 \
             tosses in exact sums: no half bound."
        );
        assert!(COST_CARD.contains("counted row by column"), "{COST_CARD}");
        assert!(!COST_CARD.contains("every entry"), "{COST_CARD}");
        for text in [COST_CARD, n.card.as_str(), t.card.as_str(), last.as_str()] {
            assert!(!text.contains('^'), "{text}");
            for width in [80, 76] {
                let lines = explainer_kit::animatic::wrap(text, width, 3);
                assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
            }
            assert!(read_s(text) < 12.0, "{text}");
        }
    }

    /// Every phrase the scenes wait for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("09_1", "one check takes"),
            ("09_1", "a thousandth"),
            ("09_2", "a fiftieth"),
            ("09_4", "how many checks"),
            ("09_5", "The bigger the matrices"),
            ("09_6", "Draw fresh coins"),
            ("09_6", "Add B's heads columns"),
            ("09_6", "multiply by A"),
            ("09_6", "and add C's heads columns"),
            ("09_7", "Otherwise, repeat"),
            ("10_2", "What is ten to the sixteen"),
            ("10_3", "Doubles there are two apart"),
            ("10_3", "so the one is lost"),
            ("10_4", "This C is A times B"),
            ("10_4", "with both coins heads"),
            ("10_5", "This C is too small by one"),
            ("10_5", "all four tosses miss it"),
            ("10_6", "but then a wrong C inside it"),
            ("11_1", "Right answers"),
            ("11_1", "a wrong one gets past"),
            ("11_3", "One toss took nine"),
            ("11_5", "How many fresh checks"),
            ("11_6", "thirty checks still take"),
            ("11_7", "Flipping the coin"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The layout keeps its pieces apart and inside the title-safe area.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        use super::super::draw::{BRACKET_GAP, DX, DY, OPEN_NAME_Y, OPEN_XS, OPEN_Y};
        use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
        const CARD_2: f64 = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0;
        // 09: the bars inside the frame, each under its label, the ratio
        // over the card, the share inside the bottom edge
        const {
            assert!(
                BAR_X > -FRAME_X_RADIUS + 0.15 && BAR_X + BAR_W < FRAME_X_RADIUS - 0.15,
                "bars inside"
            )
        };
        const { assert!(COST_N_AT[1] - 0.25 > LABEL_YS[0] + 0.2, "n over the first label") };
        const { assert!(LABEL_YS[0] - 0.2 > BAR_YS[0] + BAR_H / 2.0, "label over its bar") };
        const { assert!(BAR_YS[0] - BAR_H / 2.0 - 0.2 > LABEL_YS[1] + 0.2, "bars apart") };
        const { assert!(LABEL_YS[1] - 0.2 > BAR_YS[1] + BAR_H / 2.0, "label over its bar") };
        const { assert!(BAR_YS[1] - BAR_H / 2.0 - 0.2 > LABEL_YS[2] + 0.2, "bars apart") };
        const { assert!(LABEL_YS[2] - 0.2 > BAR_YS[2] + BAR_H / 2.0, "label over its bar") };
        const { assert!(RATIO_AT[1] - 0.4 > CARD_2, "ratio over the card") };
        const { assert!(SHARE_AT[1] - 0.4 > -FRAME_Y_RADIUS + 0.15, "share inside the frame") };
        const { assert!(BREAK_THINK_AT[1] + 0.62 < LABEL_YS[2] + 0.4, "ring under the bars") };
        // 10: the line, its labels and dots inside, the question and the
        // think ring clear of them and of each other
        const { assert!(LINE_HALF < FRAME_X_RADIUS - 0.15, "line inside") };
        const { assert!(2.0 * LINE_DX + 0.6 < LINE_HALF + 0.3, "outer labels on the line") };
        const { assert!(N_NOTE_AT[1] - 0.2 > LEGEND_AT[1] + 0.2, "N over the legend") };
        const {
            assert!(
                LEGEND_AT[1] - 0.15 > LINE_Y + GAP_LABEL_DY + 0.15,
                "legend over the gap label"
            )
        };
        const {
            assert!(
                LINE_Y + LINE_LABEL_DY - 0.2 > QUESTION_AT[1] + 0.25,
                "labels over the question"
            )
        };
        const {
            assert!(
                THINK_AT[1] + 0.62 < LINE_Y + LINE_LABEL_DY - 0.2,
                "ring under the labels"
            )
        };
        const { assert!(QUESTION_AT[1] + ANSWER_DY - 0.2 > CARD_2, "answer over the card") };
        const {
            assert!(
                GRID_XS[0] - DX - BRACKET_GAP - 0.2 > -FRAME_X_RADIUS + 0.15,
                "grids inside"
            )
        };
        const { assert!(GRID_XS[1] - GRID_XS[0] > 2.0 * (DX + BRACKET_GAP) + 0.3, "grids apart") };
        const { assert!(GRIDS_Y + DY + 0.2 < N_NOTE_AT[1] - 0.2, "grids under N") };
        const { assert!(GRIDS_Y - DY - 0.05 > GRID_NAME_Y + 0.2, "names under the grids") };
        const { assert!(GRID_NAME_Y - 0.2 > COIN_Y + SHOWN_R + 0.05, "coins under the names") };
        const { assert!(COIN_Y - SHOWN_R > ROW_YS[0] + 0.15, "rows under the ring") };
        const { assert!(ROW_YS[0] - ROW_YS[1] > 0.4, "rows apart") };
        const { assert!(ROW_YS[1] - 0.15 > WORK_YS[0] + 0.2, "work under the rows") };
        const { assert!(WORK_YS[1] - 0.2 > CARD_2, "work over the card") };
        const { assert!(SHOWN_R > COIN_PITCH / 2.0 + COIN_R, "the ring around both coins") };
        const { assert!(TOSS_XS[1] - TOSS_XS[0] > 2.0 * SHOWN_R + 0.3, "tosses apart") };
        const { assert!(ROW_LABEL_X + 1.0 < TOSS_XS[0] - 0.6, "row labels left of the verdicts") };
        // 11: the promise under the opening picture's names, over the card;
        // the assumptions right of it; the coins over C; the recap and the
        // billion in order down to the frame
        const { assert!(OPEN_NAME_Y - 0.2 > PROMISE_YS[0] + 0.2, "promise under the names") };
        const { assert!(PROMISE_YS[3] - 0.15 > CARD_2, "promise over the card") };
        const {
            assert!(
                ASSUME_X > OPEN_XS[2] + 1.5 * DX,
                "assumptions right of the promise's middle"
            )
        };
        const { assert!(OPEN_COIN_Y - COIN_R > OPEN_Y + 1.5 * DY + 0.1, "coins over C") };
        const { assert!(OPEN_COIN_Y + COIN_R < FRAME_Y_RADIUS - 0.15, "coins inside") };
        const { assert!(SCALE_YS[0] + 0.2 < OPEN_NAME_Y - 0.25, "recap under the names") };
        const { assert!(SCALE_YS[1] - 0.4 > BILLION_AT[1] + 0.3, "billion under the recap") };
        const {
            assert!(
                BILLION_AT[1] + BILLION_DY - 0.4 > -FRAME_Y_RADIUS + 0.15,
                "answer inside"
            )
        };
        const { assert!(BILLION_THINK_AT[1] - 0.62 - 0.45 > -FRAME_Y_RADIUS + 0.1, "ring inside") };
        const {
            assert!(
                LAST_D_AT[1] + LAST_COIN_DY + COIN_R < FRAME_Y_RADIUS - 0.15,
                "last coins inside"
            )
        };
        const {
            assert!(
                LAST_D_AT[1] + LAST_COIN_DY - COIN_R > LAST_D_AT[1] + 1.5 * DY + 0.05,
                "coins over the mistake"
            )
        };
        const {
            assert!(
                LAST_SUM_X - 0.5 > LAST_D_AT[0] + 1.5 * DX + BRACKET_GAP + 0.2,
                "sums right of the mistake"
            )
        };
        const { assert!(LAST_WORD_X > LAST_SUM_X + 0.6, "word right of the sums") };
    }
}
