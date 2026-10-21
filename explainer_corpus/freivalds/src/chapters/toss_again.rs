//! Chapters 07 and 08: toss again, and a toss seen first.
//!
//! 07, toss again. One toss misses the slip half the time, so toss again
//! with fresh coins (07_1). Two tosses land 64 ways: how many miss both
//! times? (07_2, a prediction.) Sixteen, exactly a quarter: the four misses
//! of the first toss times the four of the second (07_3). That is exact for
//! this slip; any wrong C gets past two fresh tosses at most a quarter of
//! the time, the theorem's bound (07_4). One toss reused for both checks is
//! the grid's diagonal, still half (07_5). Each fresh toss at least halves
//! the chance: twenty all miss less than once in a million (07_6). That is
//! a chance over the coins for a fixed C, not the chance C is wrong (07_7);
//! the check errs one way only (07_8). Its name (07_9).
//!
//! 08, a toss seen first. Fresh also means tossed after C is handed over
//! (08_1). Seen heads, heads, tails: where can a wrong entry hide? (08_2, a
//! prediction.) Anywhere under the tails coin (08_3); under all heads a row
//! that cancels, the sum of every column again (08_4). Every toss can be
//! beaten when seen first, yet fresh tosses catch each of those wrong Cs
//! half the time (08_5): the protection is that the toss is not known
//! (08_6), and what a programmer must do (08_7).

use std::collections::BTreeSet;

use explainer_kit::props::{lower_third, names_to_search, precisely};
use explainer_kit::{Kit, Palette, fade_all, rect, v};
use fmn::prelude::*;

use super::draw::{
    COIN_R, Coin, Grid, cell, cell_box, coin_mobs, column_box, fade_in_all, fade_in_lagged, fade_out_all, fade_to,
    fail, flash_all, flat, fly, grid_colored, line_style, now, num, read_s, ring, row_box, say, stand, swap, tex_of,
    tex_thousands, think, together, toss_coins, wait_until,
};
use crate::facts::{self, AGAIN, MILLION, TOSS_NAMES};
use crate::model::{self, A, A2, B, B2, C, FIRST_TOSS, Matrix, SIDE, Toss};
use crate::palette;

pub struct TossAgain {
    pub kit: Kit,
}

pub struct SeenFirst {
    pub kit: Kit,
}

// the chapters' fixed words (numbers are built from the model and facts)
const MISTAKE_NAME: &str = r"C - AB";
/// Over the name, from 08_3 on: the eight grids are mistakes, not Cs.
const GRID_NOTE: &str = "each grid shows";
const MISS: &str = "miss";
const CAUGHT: &str = "caught";
const SEEN_CARD: &str = "from two by two up, any toss seen first can be beaten: one wrong entry under a tails coin, or \
                         under all heads a cancelling pair in one row.";
const TOO_SMALL: &str = "too small to see";
const MEANS: [&str; 2] = [
    "a chance over the coins, for this fixed C",
    "not: the chance that C is wrong after passing",
];
const MEANS_CARD: &str = "the bound is a chance over the coins for a fixed C; it is not the chance that C is wrong \
                          after passing. A caught toss proves C wrong";
const ONE_WAY: [&str; 2] = [
    "C right: every toss matches",
    "C wrong: a caught toss proves it; a miss proves nothing",
];
const NAME: &str = "Freivalds' algorithm";
const NAMES: [&str; 3] = ["Freivalds' algorithm", "randomized algorithm", "one-sided error"];
const HIDE_QUESTION: &str = "Where can a wrong entry hide from heads, heads, tails?";
const ANY_HERE: &str = "any nonzero entry here";
const ALL_HEADS_TAG: &str = "all heads = the sum of every column";
const PROGRAMMERS: &str = "for programmers:";
const COINS_CARD: &str = "the coins must not depend on C: drawn after C is fixed, from a source its maker cannot \
                          predict, fair, and new for every check";

// ------------------------------------------------------------------ layout

/// The tosses of three coins, and so each side of the grid of ways.
const TOSS_COUNT: usize = 1 << SIDE;
/// The slip's mistake at the top left (07_1 to 07_5), and its name.
const SLIP_X: f64 = -5.55;
const SLIP_Y: f64 = 2.45;
const SLIP_NAME_Y: f64 = 1.25;
/// The grid of the 64 ways two tosses land (07_2 to 07_5): a square for each
/// first toss (rows) and second toss (columns), in `facts::miss_first`
/// order, and the tosses' labels along its left and top sides.
const WAY: f64 = 0.46;
const WAY_GAP: f64 = 0.06;
const WAYS_X: f64 = -0.9;
const WAYS_Y: f64 = -0.35;
const WAYS_HALF: f64 = WAY * TOSS_COUNT as f64 / 2.0;
const LABEL_SIZE: f64 = 17.0;
const LEFT_LABEL_DX: f64 = 0.32;
const TOP_LABEL_DY: f64 = 0.27;
const AXIS_SIZE: f64 = 22.0;
const FIRST_TITLE_AT: Vec3 = [-4.1, WAYS_Y, 0.0];
const SECOND_TITLE_AT: Vec3 = [WAYS_X, WAYS_Y + WAYS_HALF + TOP_LABEL_DY + 0.42, 0.0];
/// The outlines of the first and second tosses' misses (07_3), just
/// outside their squares.
const BAND_PAD: f64 = 0.03;
/// The squares off the diagonal, dimmed while one toss is reused (07_5).
const DIMMED: f64 = 0.25;
/// The words at the right of the grid, one line a step, and the think ring;
/// the two stacked fraction lines (07_4) a little farther apart.
const PANEL_X: f64 = 1.55;
const PANEL_YS: [f64; 6] = [1.6, 1.05, 0.5, -0.05, -0.7, -1.25];
/// Over the one-toss line (07_1 to 07_5), the toss labels' colors in words.
const KEY_NOTE: &str = "toss labels:";
const KEY_Y: f64 = 2.1;
const KEY_GAP: f64 = 0.25;
const PANEL_SIZE: f64 = 24.0;
const PANEL_TEX: f64 = 28.0;
const QUESTION_SIZE: f64 = 26.0;
const THINK_AT: Vec3 = [4.6, -0.95, 0.0];
/// The halving ladder (07_6): one, two and twenty tosses.
const LADDER: [u32; 3] = [1, 2, AGAIN];
const LADDER_YS: [f64; 3] = [0.1, -0.75, -1.6];
const LADDER_LABEL_X: f64 = -6.1;
const LADDER_X: f64 = -3.2;
/// A chance of one, as a bar's length.
const LADDER_W: f64 = 4.0;
const BAR_H: f64 = 0.3;
/// The narrowest bar the ladder draws, scene units: about a pixel at 1080p
/// (eight units are 1080 pixels there).
const MIN_BAR_W: f64 = 0.01;
/// What the ladder dims to under what it means (07_7).
const LADDER_DIM: f64 = 0.3;
/// 07_7, 07_8: what the chance means, and the one way it errs.
const MEANS_YS: [f64; 2] = [2.6, 2.0];
const ONE_WAY_YS: [f64; 2] = [1.35, 0.85];
const NAME_AT: Vec3 = [0.0, 0.6, 0.0];
/// The eight tosses seen first (08_1 to 08_8): two rows of four, each one's
/// coins over the columns of the mistake written for it, the verdict under.
const SPOT_XS: [f64; 4] = [-5.1, -2.5, 0.1, 2.7];
const SPOT_COIN_YS: [f64; 2] = [3.2, 0.4];
const SPOT_GRID_DY: f64 = -1.1;
const SPOT_VERDICT_DY: f64 = -2.25;
/// 08_2: the ring around the toss seen first.
const SEEN_R: f64 = 0.85;
const SEEN_THINK_AT: Vec3 = [5.5, -0.6, 0.0];
const TALLY_X: f64 = 5.5;
const TALLY_YS: [f64; 2] = [2.75, 2.3];
/// What the eight grids are (08_3 to 08_7), under the tally: the note, and
/// the mistake's name under it.
const GRID_NOTE_AT: [Vec3; 2] = [[5.5, 1.6, 0.0], [5.5, 1.12, 0.0]];
const NOTE_SIZE: f64 = 20.0;
const FRESH_X: f64 = -1.2;
const FRESH_Y: f64 = -2.3;
/// 08_6: the coins dim, and a "?" beside each toss's coins (over them it
/// would meet the verdicts of the row above).
const COINS_DIM: f64 = 0.3;
const UNKNOWN_DX: f64 = 1.15;

/// The centre of the square for the `i`-th first toss and the `j`-th
/// second toss, in the grid's order.
fn way_at(i: usize, j: usize) -> Vec3 {
    let mid = (TOSS_COUNT as f64 - 1.0) / 2.0;
    v(WAYS_X + (j as f64 - mid) * WAY, WAYS_Y + (mid - i as f64) * WAY)
}

/// Where the `i`-th first toss's label stands, left of its row.
fn first_label_at(i: usize) -> Vec3 {
    v(WAYS_X - WAYS_HALF - LEFT_LABEL_DX, way_at(i, 0)[1])
}

/// Where the `j`-th second toss's label stands, over its column.
fn second_label_at(j: usize) -> Vec3 {
    v(way_at(0, j)[0], WAYS_Y + WAYS_HALF + TOP_LABEL_DY)
}

/// Where seen toss `k` (counting order) stands: two rows of four.
fn spot_place(k: usize) -> (f64, f64) {
    (SPOT_XS[k % SPOT_XS.len()], SPOT_COIN_YS[k / SPOT_XS.len()])
}

/// The x of each column of a mistake's grid centred at `x`, where the
/// toss's coins stand over them.
fn spot_coin_xs(x: f64) -> Vec<f64> {
    (0..SIDE).map(|j| cell(v(x, 0.0), SIDE, SIDE, 0, j)[0]).collect()
}

/// The words at the right of the grid of ways (07_1 to 07_5).
struct Panel {
    one_toss: String,
    question: [String; 2],
    answer: String,
    product: String,
    this_slip: String,
    any_wrong: String,
    reused: String,
}

/// The panel, from the model and the theorem's bound. Refused unless both
/// tosses miss in exactly the misses of each, and this slip's share of the
/// ways meets the bound for two tosses (the line says both are a quarter).
fn panel() -> Result<Panel, String> {
    let checks = facts::all_checks();
    let tosses = checks.len();
    let misses = checks.iter().filter(|(_, ch)| ch.passes()).count();
    let (both, ways) = model::both_pass(&A, &B, &C);
    if both != misses * misses {
        return Err(format!("{both} ways miss both times, not {misses} times {misses}"));
    }
    let (num, den) = facts::theorem_bound()?;
    let k = TOSS_NAMES.len();
    let per = tex_of(&facts::ratio(num as i64, den as i64)?);
    let bound = facts::ratio(num.pow(k as u32) as i64, den.pow(k as u32) as i64)?;
    let share = facts::ratio(both as i64, ways as i64)?;
    if share != bound {
        return Err(format!(
            "this slip misses two tosses {share} of the time, not the bound {bound}"
        ));
    }
    let (reused, diagonal) = model::same_toss_twice(&A, &B, &C);
    let factors: Vec<&str> = std::iter::repeat_n(per.as_str(), k).collect();
    Ok(Panel {
        one_toss: format!("one toss: {misses} of {tosses} miss"),
        question: [format!("How many of these {ways} ways"), "miss both times?".to_owned()],
        answer: format!("{both} of {ways} ways miss both times"),
        product: format!(r"{misses} \times {misses} = {both}"),
        this_slip: format!(r"\text{{this slip: }} \frac{{{both}}}{{{ways}}} = {}", tex_of(&share)),
        any_wrong: format!(
            r"\text{{any wrong }} C\text{{: }} \le {} = {}",
            factors.join(r" \cdot "),
            tex_of(&bound)
        ),
        reused: format!(
            "same toss twice: {reused} of {diagonal}, still {}",
            facts::ratio(reused as i64, diagonal as i64)?
        ),
    })
}

/// The exact version of 07_4: the conditions the chance needs, and the bound
/// for k tosses, the theorem's per-toss bound to the k.
fn again_card() -> Result<String, String> {
    let (num, den) = facts::theorem_bound()?;
    Ok(format!(
        "in exact arithmetic, with fair coins, all independent and tossed after C is fixed, k tosses all miss any \
         fixed wrong C with chance at most {num}/{den} to the k."
    ))
}

/// The ladder's bar for a chance of one in `odds`, or None when it would be
/// narrower than [`MIN_BAR_W`]: such a chance is labelled too small to see,
/// never drawn wider than it is.
fn bar_width(odds: u64) -> Option<f64> {
    let w = LADDER_W / odds as f64;
    (w >= MIN_BAR_W).then_some(w)
}

/// The bound on the ladder's row for `k` tosses, at `per` to one a toss:
/// "≤ 1/2", and for the last row the million it beats.
fn ladder_tex(k: u32, per: u64) -> Result<String, String> {
    let odds = per.pow(k);
    if k == AGAIN {
        if odds <= MILLION {
            return Err(format!("1/{odds} is not less than one in a million"));
        }
        return Ok(format!(
            r"\le \frac{{1}}{{{}}} < \frac{{1}}{{{}}}",
            tex_thousands(odds),
            tex_thousands(MILLION)
        ));
    }
    Ok(format!(r"\le \frac{{1}}{{{odds}}}"))
}

/// The exact version of 08_1 and 08_3, refused unless every toss of three
/// coins (`spots`) and of two is beaten by its blind spot. A single heads
/// coin has none, hence "from two by two up".
fn seen_card(spots: &[(Toss<SIDE>, Matrix<SIDE>)]) -> Result<&'static str, String> {
    let ab2 = model::product(&A2, &B2).0;
    let two_by_two = model::tosses::<2>()
        .iter()
        .all(|t| model::blind_spot(t).is_some_and(|d| model::check(&A2, &B2, &model::plus(&ab2, &d), t).passes()));
    if spots.len() != TOSS_COUNT || !two_by_two {
        return Err("a toss seen first is not always beaten from two by two up".into());
    }
    Ok(SEEN_CARD)
}

/// The words of 08_4 and 08_5.
struct SeenWords {
    tally: [String; 2],
    /// The all-heads toss's cancelling row, "1 + (−1) = 0", and which row.
    cancel: String,
    cancel_row: usize,
    fresh: String,
}

/// The words for the tosses seen first, from their blind spots. Refused
/// unless every toss lets its own through, the all-heads one cancels in a
/// single row, and fresh tosses catch each of them exactly half the time.
fn seen_words(spots: &[(Toss<SIDE>, Matrix<SIDE>)]) -> Result<SeenWords, String> {
    let ab = facts::true_product();
    let tosses = model::tosses::<SIDE>().len();
    let beaten = spots
        .iter()
        .filter(|(t, d)| model::check(&A, &B, &model::plus(&ab, d), t).passes())
        .count();
    if beaten != tosses {
        return Err(format!("{beaten} of {tosses} tosses are beaten when seen first"));
    }
    let heads = spots
        .iter()
        .find(|(t, _)| !t.contains(&false))
        .map(|(_, d)| *d)
        .ok_or("no toss is all heads")?;
    let wrong = model::wrong_entries(&heads);
    let rows: Vec<usize> = wrong
        .iter()
        .map(|&(i, _)| i)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let values: Vec<i64> = wrong.iter().map(|&(i, j)| heads[i][j]).collect();
    let sum: i64 = values.iter().sum();
    let cancel_row = match rows[..] {
        [row] if sum == 0 => row,
        _ => return Err("under all heads the mistake must cancel in one row".into()),
    };
    let terms: Vec<String> = values
        .iter()
        .map(|&x| if x < 0 { format!("({})", num(x)) } else { num(x) })
        .collect();
    let catches: Vec<usize> = spots
        .iter()
        .map(|(_, d)| tosses - model::passing(&A, &B, &model::plus(&ab, d)).len())
        .collect();
    let catch = *catches.first().ok_or("no toss is seen first")?;
    if catches.iter().any(|&c| c != catch) || 2 * catch != tosses {
        return Err(format!(
            "fresh tosses catch these wrong Cs {catches:?} times, not half each"
        ));
    }
    Ok(SeenWords {
        tally: [
            format!("{beaten} of {tosses} tosses beaten"),
            "when seen first".to_owned(),
        ],
        cancel: format!("{} = {}", terms.join(" + "), num(sum)),
        cancel_row,
        fresh: format!("fresh tosses catch each of these wrong Cs: {catch} of {tosses}"),
    })
}

/// A mistake drawn as a grid, its zeros dim and its wrong entries plain,
/// each wrong entry outlined in the mistake role. Detached.
fn mistake_grid(kit: &Kit, stage: &mut Stage<'_>, d: &Matrix<SIDE>, centre: Vec3) -> fmn::Result<(Grid, Vec<Mob>)> {
    let values = d.iter().map(|row| row.to_vec()).collect();
    let g = grid_colored(kit, stage, values, centre, |x| {
        if x == 0 { Palette::dim() } else { Palette::ink() }
    })?;
    let mut boxes = Vec::new();
    for (i, j) in model::wrong_entries(d) {
        boxes.push(cell_box(stage, &g, i, j, palette::mistake())?);
    }
    Ok((g, boxes))
}

/// Toss `t` written as its coins, "101", red if it misses the slip and
/// green if it catches it. Detached.
fn toss_label(kit: &Kit, stage: &mut Stage<'_>, t: &Toss<SIDE>, at: Vec3) -> fmn::Result<Mob> {
    let color = if model::check(&A, &B, &C, t).passes() {
        palette::mistake()
    } else {
        palette::caught()
    };
    kit.text(stage, &model::toss_text(t), LABEL_SIZE, color, at)
}

/// The key to [`toss_label`]'s colors, left to right from the panel's edge:
/// "toss labels:", then "miss" in the miss color and "caught" in the catch
/// color, so color is not the only sign. Detached.
fn label_key(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let mut x = PANEL_X;
    let mut key = Vec::with_capacity(3);
    for (word, color) in [
        (KEY_NOTE, Palette::dim()),
        (MISS, palette::mistake()),
        (CAUGHT, palette::caught()),
    ] {
        let m = kit.text_left(stage, word, NOTE_SIZE, color, v(x, KEY_Y))?;
        x += stage.get_width(m) + KEY_GAP;
        key.push(m);
    }
    Ok(key)
}

/// A square of side `side` centred at `at`. Detached.
fn square(stage: &mut Stage<'_>, at: Vec3, side: f64, style: Style) -> fmn::Result<Mob> {
    let r = rect(v(at[0] - side / 2.0, at[1] - side / 2.0), side, side, style)?;
    Ok(stage.arena_mut().add(r))
}

/// An outline `w` × `h` from its lower-left corner `dl`. Detached.
fn band(stage: &mut Stage<'_>, dl: Vec3, w: f64, h: f64, color: Srgb) -> fmn::Result<Mob> {
    let r = rect(dl, w, h, line_style(color, 2.5))?;
    Ok(stage.arena_mut().add(r))
}

impl SceneConstruct for TossAgain {
    fn name(&self) -> &str {
        "07_toss_again"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ab = facts::true_product();
        let words = panel().map_err(fail)?;
        let all = model::tosses::<SIDE>();
        let order = facts::miss_first();
        if all.len() != TOSS_COUNT || order.len() != TOSS_COUNT {
            return Err(fail(format!("the grid of ways is drawn for {TOSS_COUNT} tosses")));
        }
        let misses_slip = |t: &Toss<SIDE>| model::check(&A, &B, &C, t).passes();
        let misses = order.iter().filter(|&t| misses_slip(t)).count();
        let slip = model::mistake(&C, &ab);
        let per = match facts::theorem_bound().map_err(fail)? {
            (1, per) => per,
            (num, den) => {
                return Err(fail(format!(
                    "a per-toss bound of {num}/{den} is not one in a whole number"
                )));
            }
        };

        // 07_1: one toss misses the slip half the time; toss again
        let line = say(&kit, stage, "07_1")?;
        let (slip_grid, slip_boxes) = mistake_grid(&kit, stage, &slip, v(SLIP_X, SLIP_Y))?;
        let slip_name = kit.tex(
            stage,
            MISTAKE_NAME,
            30.0,
            palette::mistake(),
            &[],
            v(SLIP_X, SLIP_NAME_Y),
        )?;
        let mut arriving = slip_grid.mobs();
        arriving.extend(&slip_boxes);
        arriving.push(slip_name);
        fade_in_all(stage, &arriving, 0.5)?;
        let mut firsts = Vec::with_capacity(TOSS_COUNT);
        for (k, t) in all.iter().enumerate() {
            firsts.push(toss_label(&kit, stage, t, first_label_at(k))?);
        }
        fade_in_lagged(stage, &firsts, 0.9, 0.2)?;
        let one_toss = kit.text_left(
            stage,
            &words.one_toss,
            PANEL_SIZE,
            Palette::ink(),
            v(PANEL_X, PANEL_YS[0]),
        )?;
        let key = label_key(&kit, stage)?;
        let mut top_lines = vec![one_toss];
        top_lines.extend(&key);
        fade_in_all(stage, &top_lines, 0.3)?;
        line.wait_for(stage, "So toss again")?;
        // the column sorts, misses first, and a copy turns to run along the top
        let rank = |t: &Toss<SIDE>| order.iter().position(|u| u == t).unwrap_or(0);
        let mut moves = Vec::with_capacity(TOSS_COUNT);
        for (&m, t) in firsts.iter().zip(&all) {
            moves.push(fly(m, first_label_at(rank(t)), 0.7)?);
        }
        together(stage, moves)?;
        let first_misses: Vec<Mob> = firsts
            .iter()
            .zip(&all)
            .filter(|&(_, t)| misses_slip(t))
            .map(|(&m, _)| m)
            .collect();
        let mut seconds = Vec::with_capacity(TOSS_COUNT);
        let mut moves = Vec::with_capacity(TOSS_COUNT);
        for (j, t) in order.iter().enumerate() {
            let m = toss_label(&kit, stage, t, first_label_at(j))?;
            stage.add_to_scene(m)?;
            moves.push(fly(m, second_label_at(j), 0.8)?);
            seconds.push(m);
        }
        together(stage, moves)?;
        let titles = [
            kit.text(stage, TOSS_NAMES[0], AXIS_SIZE, Palette::dim(), FIRST_TITLE_AT)?,
            kit.text(stage, TOSS_NAMES[1], AXIS_SIZE, Palette::dim(), SECOND_TITLE_AT)?,
        ];
        fade_in_all(stage, &titles, 0.3)?;

        // 07_2: the 64 ways two tosses land; the question, and the think
        // time, with no square red
        let line = say(&kit, stage, "07_2")?;
        let mut squares = Vec::with_capacity(TOSS_COUNT * TOSS_COUNT);
        for i in 0..TOSS_COUNT {
            for j in 0..TOSS_COUNT {
                squares.push(square(
                    stage,
                    way_at(i, j),
                    WAY - WAY_GAP,
                    flat(Palette::faint(), 0.45),
                )?);
            }
        }
        fade_in_lagged(stage, &squares, 1.2, 0.05)?;
        line.wait_for(stage, "How many of them")?;
        let question = [
            kit.text_left(
                stage,
                &words.question[0],
                QUESTION_SIZE,
                Palette::ink(),
                v(PANEL_X, PANEL_YS[1]),
            )?,
            kit.text_left(
                stage,
                &words.question[1],
                QUESTION_SIZE,
                Palette::ink(),
                v(PANEL_X, PANEL_YS[2]),
            )?,
        ];
        fade_in_all(stage, &question, 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 07_3: sixteen, a quarter; the misses of the first toss times the
        // misses of the second, filling the corner
        let line = say(&kit, stage, "07_3")?;
        let passes: Vec<bool> = order.iter().map(misses_slip).collect();
        let mut red = Vec::new();
        let mut red_diagonal = Vec::new();
        let mut red_off = Vec::new();
        for (i, &first) in passes.iter().enumerate() {
            for (j, &second) in passes.iter().enumerate() {
                let both = first && second;
                if both != (i < misses && j < misses) {
                    return Err(fail("the ways both tosses miss must fill the grid's corner".into()));
                }
                if both {
                    let m = square(stage, way_at(i, j), WAY - WAY_GAP, flat(palette::mistake(), 0.85))?;
                    if i == j {
                        red_diagonal.push(m);
                    } else {
                        red_off.push(m);
                    }
                    red.push(m);
                }
            }
        }
        fade_out_all(stage, &[q], 0.2)?;
        fade_in_lagged(stage, &red, 0.8, 0.1)?;
        let answer = kit.text_left(
            stage,
            &words.answer,
            PANEL_SIZE,
            palette::mistake(),
            v(PANEL_X, PANEL_YS[1]),
        )?;
        fade_out_all(stage, &question, 0.2)?;
        fade_in_all(stage, &[answer], 0.2)?;
        line.wait_for(stage, "four ways for the first toss")?;
        let side = 2.0 * WAYS_HALF + 2.0 * BAND_PAD;
        let corner = misses as f64 * WAY + 2.0 * BAND_PAD;
        let (left, top, bottom) = (
            WAYS_X - WAYS_HALF - BAND_PAD,
            WAYS_Y + WAYS_HALF + BAND_PAD,
            WAYS_Y - WAYS_HALF - BAND_PAD,
        );
        let rows = band(stage, v(left, top - corner), side, corner, palette::mistake())?;
        fade_in_all(stage, &[rows], 0.3)?;
        flash_all(stage, &first_misses, 0.5)?;
        line.wait_for(stage, "times four for the second")?;
        let cols = band(stage, v(left, bottom), corner, side, palette::mistake())?;
        fade_in_all(stage, &[cols], 0.3)?;
        flash_all(stage, &seconds[..misses], 0.5)?;
        let product = kit.tex_left(
            stage,
            &words.product,
            PANEL_TEX,
            Palette::ink(),
            &[],
            v(PANEL_X, PANEL_YS[2]),
        )?;
        fade_in_all(stage, &[product], 0.4)?;

        // 07_4: exact for this slip; at most a quarter for any wrong C, the
        // theorem's bound, with its exact version
        let line = say(&kit, stage, "07_4")?;
        fade_out_all(stage, &[rows, cols], 0.3)?;
        let this_slip = kit.tex_left(
            stage,
            &words.this_slip,
            PANEL_TEX,
            Palette::ink(),
            &[],
            v(PANEL_X, PANEL_YS[3]),
        )?;
        fade_in_all(stage, &[this_slip], 0.4)?;
        line.wait_for(stage, "Any wrong C")?;
        let any_wrong = kit.tex_left(
            stage,
            &words.any_wrong,
            PANEL_TEX,
            Palette::ink(),
            &[],
            v(PANEL_X, PANEL_YS[4]),
        )?;
        let again_text = again_card().map_err(fail)?;
        let again = precisely(&kit, stage, &again_text)?;
        fade_in_all(stage, &[any_wrong, again], 0.4)?;
        let again_up = now(stage);

        // 07_5: one toss reused for both checks is the diagonal: still half
        let line = say(&kit, stage, "07_5")?;
        let (reused, _) = model::same_toss_twice(&A, &B, &C);
        if red_diagonal.len() != reused {
            return Err(fail(format!(
                "{} squares of the diagonal are red, but {reused} tosses miss twice",
                red_diagonal.len()
            )));
        }
        let mut dims = Vec::new();
        for (n, &m) in squares.iter().enumerate() {
            if n / TOSS_COUNT != n % TOSS_COUNT {
                dims.push(fade_to(m, DIMMED, 0.5)?);
            }
        }
        for &m in &red_off {
            dims.push(fade_to(m, DIMMED, 0.5)?);
        }
        together(stage, dims)?;
        let mut outlines = Vec::with_capacity(TOSS_COUNT);
        for k in 0..TOSS_COUNT {
            outlines.push(square(
                stage,
                way_at(k, k),
                WAY - 0.02,
                line_style(Palette::hero(), 2.5),
            )?);
        }
        fade_in_all(stage, &outlines, 0.4)?;
        line.wait_for(stage, "Four of the eight")?;
        flash_all(stage, &red_diagonal, 0.6)?;
        let reused_text = kit.text_left(
            stage,
            &words.reused,
            PANEL_SIZE,
            palette::mistake(),
            v(PANEL_X, PANEL_YS[5]),
        )?;
        fade_in_all(stage, &[reused_text], 0.4)?;

        // 07_6: each fresh toss at least halves the chance; the ladder of the
        // theorem's bound, one in `per` a toss
        let line = say(&kit, stage, "07_6")?;
        let mut leaving = slip_grid.mobs();
        leaving.extend(slip_boxes);
        leaving.push(slip_name);
        leaving.extend(firsts);
        leaving.extend(seconds);
        leaving.extend(titles);
        leaving.extend(squares);
        leaving.extend(red);
        leaving.extend(outlines);
        leaving.extend([one_toss, answer, product, this_slip, any_wrong, reused_text]);
        leaving.extend(key);
        fade_out_all(stage, &leaving, 0.5)?;
        let mut ladder = Vec::new();
        for (n, (&k, &y)) in LADDER.iter().zip(&LADDER_YS).enumerate() {
            if n == 2 {
                line.wait_for(stage, "Twenty fresh tosses")?;
            }
            let label = kit.text_left(
                stage,
                &format!("{k} {}", if k == 1 { "toss" } else { "tosses" }),
                26.0,
                Palette::ink(),
                v(LADDER_LABEL_X, y),
            )?;
            let track = stage.arena_mut().add(rect(
                v(LADDER_X, y - BAR_H / 2.0),
                LADDER_W,
                BAR_H,
                flat(Palette::faint(), 0.6),
            )?);
            let bound = kit.tex_left(
                stage,
                &ladder_tex(k, per).map_err(fail)?,
                30.0,
                Palette::ink(),
                &[],
                v(LADDER_X + LADDER_W + 0.3, y),
            )?;
            // the bar to scale, or, for twenty tosses, the honest note that
            // it is too small to see (painted over the track)
            let fill = match bar_width(per.pow(k)) {
                Some(w) => stage.arena_mut().add(rect(
                    v(LADDER_X, y - BAR_H / 2.0),
                    w,
                    BAR_H,
                    flat(palette::mistake(), 0.9),
                )?),
                None => kit.text_left(stage, TOO_SMALL, NOTE_SIZE, Palette::dim(), v(LADDER_X + 0.15, y))?,
            };
            fade_in_all(stage, &[label, track, fill, bound], 0.5)?;
            ladder.extend([label, track, fill, bound]);
        }

        // 07_7: a chance over the coins for a fixed C, not the chance that C
        // is wrong; its exact version once the promise card is read
        let line = say(&kit, stage, "07_7")?;
        let mut dims = Vec::with_capacity(ladder.len());
        for &m in &ladder {
            dims.push(fade_to(m, LADDER_DIM, 0.4)?);
        }
        together(stage, dims)?;
        let means = kit.text(stage, MEANS[0], 24.0, Palette::ink(), v(0.0, MEANS_YS[0]))?;
        fade_in_all(stage, &[means], 0.4)?;
        line.wait_for(stage, "It doesn't say")?;
        let not_means = kit.text(stage, MEANS[1], 24.0, Palette::dim(), v(0.0, MEANS_YS[1]))?;
        let w = stage.get_width(not_means);
        let strike = stage.arena_mut().add(
            polyline(vec![v(-w / 2.0, MEANS_YS[1]), v(w / 2.0, MEANS_YS[1])])
                .style(line_style(Palette::dim(), 2.5))
                .build(),
        );
        fade_in_all(stage, &[not_means], 0.4)?;
        fade_in_all(stage, &[strike], 0.3)?;
        wait_until(stage, again_up + read_s(&again_text))?;
        let means_card = precisely(&kit, stage, MEANS_CARD)?;
        fade_out_all(stage, &[again], 0.2)?;
        fade_in_all(stage, &[means_card], 0.2)?;
        let means_up = now(stage);

        // 07_8: the check errs one way only
        let line = say(&kit, stage, "07_8")?;
        line.wait_for(stage, "Once a toss catches C")?;
        let one_way = [
            kit.text(stage, ONE_WAY[0], 22.0, Palette::ink(), v(0.0, ONE_WAY_YS[0]))?,
            kit.text(stage, ONE_WAY[1], 22.0, Palette::ink(), v(0.0, ONE_WAY_YS[1]))?,
        ];
        fade_in_lagged(stage, &one_way, 0.6, 0.4)?;

        // 07_9: its name, and the names to search
        say(&kit, stage, "07_9")?;
        let mut leaving = ladder;
        leaving.extend([means, not_means, strike]);
        leaving.extend(one_way);
        fade_out_all(stage, &leaving, 0.4)?;
        let name = kit.text(stage, NAME, 48.0, Palette::hero(), NAME_AT)?;
        fade_in_all(stage, &[name], 0.5)?;
        wait_until(stage, means_up + read_s(MEANS_CARD))?;
        let names = names_to_search(&kit, stage, &NAMES)?;
        fade_out_all(stage, &[means_card], 0.2)?;
        fade_in_all(stage, &[names], 0.2)?;
        let names_up = now(stage);
        stand(&kit, stage, names_up)?;
        wait_until(stage, names_up + read_s(&NAMES.join(" · ")))?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

impl SceneConstruct for SeenFirst {
    fn name(&self) -> &str {
        "08_seen_first"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ab = facts::true_product();
        let spots = facts::blind_spots().map_err(fail)?;
        let seen_text = seen_words(&spots).map_err(fail)?;
        let card_text = seen_card(&spots).map_err(fail)?;
        let seen_k = spots.iter().position(|(t, _)| *t == FIRST_TOSS).ok_or_else(|| {
            fail(format!(
                "toss {} is not among the tosses",
                model::toss_text(&FIRST_TOSS)
            ))
        })?;
        let tails: Vec<usize> = (0..SIDE).filter(|&j| !FIRST_TOSS[j]).collect();
        let [tails_column] = tails[..] else {
            return Err(fail("the toss seen first must have one tails coin".into()));
        };

        // 08_1: fresh also means after C is handed over; the eight tosses
        let line = say(&kit, stage, "08_1")?;
        line.wait_for(stage, "Someone who sees your toss first")?;
        let mut coin_sets: Vec<Vec<Coin>> = Vec::with_capacity(spots.len());
        for (k, (t, _)) in spots.iter().enumerate() {
            let (x, y) = spot_place(k);
            coin_sets.push(toss_coins(&kit, stage, t, &spot_coin_xs(x), y)?);
        }
        let all_coins: Vec<Mob> = coin_sets.iter().flat_map(|c| coin_mobs(c.as_slice())).collect();
        fade_in_lagged(stage, &all_coins, 1.2, 0.15)?;

        // 08_2: seen heads, heads, tails: where can a wrong entry hide? An
        // empty mistake under it, and the think time with nothing marked
        let line = say(&kit, stage, "08_2")?;
        let (sx, sy) = spot_place(seen_k);
        let seen_ring = ring(stage, v(sx, sy), SEEN_R, Palette::hero());
        let (empty, _) = mistake_grid(&kit, stage, &[[0; SIDE]; SIDE], v(sx, sy + SPOT_GRID_DY))?;
        let mut arriving = vec![seen_ring];
        arriving.extend(empty.mobs());
        fade_in_all(stage, &arriving, 0.4)?;
        line.wait_for(stage, "Where could they")?;
        let question = kit.text(stage, HIDE_QUESTION, 24.0, Palette::ink(), v(FRESH_X, FRESH_Y))?;
        fade_in_all(stage, &[question], 0.4)?;
        let q = think(&kit, stage, SEEN_THINK_AT)?;

        // 08_3: anywhere under the tails coin, with the seen card (held
        // back until now, since it names the answer); then every toss's own
        // blind spot, and the tails coins over them
        let line = say(&kit, stage, "08_3")?;
        fade_out_all(stage, &[q, question], 0.2)?;
        let seen = precisely(&kit, stage, card_text)?;
        let here = column_box(stage, &empty, tails_column, palette::mistake())?;
        let here_note = kit.text(
            stage,
            ANY_HERE,
            NOTE_SIZE,
            palette::mistake(),
            v(sx, sy + SPOT_VERDICT_DY),
        )?;
        let tails_ring = ring(
            stage,
            coin_sets[seen_k][tails_column].at,
            COIN_R + 0.07,
            palette::mistake(),
        );
        fade_in_all(stage, &[seen, here, here_note, tails_ring], 0.4)?;
        let card_up = now(stage);
        line.wait_for(stage, "The toss never looks there")?;
        let mut leaving = vec![seen_ring, here, here_note, tails_ring];
        leaving.extend(empty.mobs());
        fade_out_all(stage, &leaving, 0.3)?;
        let mut grids = Vec::with_capacity(spots.len());
        let mut verdicts = Vec::with_capacity(spots.len());
        let mut arriving = Vec::new();
        for (k, (t, d)) in spots.iter().enumerate() {
            let (x, y) = spot_place(k);
            let (g, boxes) = mistake_grid(&kit, stage, d, v(x, y + SPOT_GRID_DY))?;
            let (word, color) = if model::check(&A, &B, &model::plus(&ab, d), t).passes() {
                (MISS, palette::mistake())
            } else {
                (CAUGHT, palette::caught())
            };
            let verdict = kit.text(stage, word, 22.0, color, v(x, y + SPOT_VERDICT_DY))?;
            arriving.extend(g.mobs());
            arriving.extend(boxes);
            arriving.push(verdict);
            verdicts.push(verdict);
            grids.push(g);
        }
        // the grids are each wrong C's mistake, C − AB, not the C itself;
        // the label stays to the end
        arriving.extend([
            kit.text(stage, GRID_NOTE, NOTE_SIZE, Palette::dim(), GRID_NOTE_AT[0])?,
            kit.tex(stage, MISTAKE_NAME, 30.0, palette::mistake(), &[], GRID_NOTE_AT[1])?,
        ]);
        fade_in_lagged(stage, &arriving, 1.6, 0.05)?;
        let mut rings = Vec::new();
        for ((t, d), coins) in spots.iter().zip(&coin_sets) {
            if !t.contains(&false) {
                continue;
            }
            for (_, j) in model::wrong_entries(d) {
                if t[j] {
                    return Err(fail(format!(
                        "toss {}'s wrong entry is not under a tails coin",
                        model::toss_text(t)
                    )));
                }
                rings.push(ring(stage, coins[j].at, COIN_R + 0.07, palette::mistake()));
            }
        }
        fade_in_lagged(stage, &rings, 0.8, 0.3)?;

        // 08_4: under all heads, a row that cancels: the sum of every column
        let line = say(&kit, stage, "08_4")?;
        let heads = spots
            .iter()
            .position(|(t, _)| !t.contains(&false))
            .ok_or_else(|| fail("no toss is all heads".into()))?;
        let g = &grids[heads];
        let row_mark = row_box(stage, g, seen_text.cancel_row, Palette::hero())?;
        let cancel = kit.text_left(
            stage,
            &seen_text.cancel,
            PANEL_SIZE,
            Palette::hero(),
            v(g.centre[0] + g.half_width() + 0.3, g.at(seen_text.cancel_row, 0)[1]),
        )?;
        fade_in_all(stage, &[row_mark, cancel], 0.4)?;
        line.wait_for(stage, "That's the sum of every column again")?;
        let tag = kit.text(stage, ALL_HEADS_TAG, NOTE_SIZE, Palette::hero(), v(FRESH_X, FRESH_Y))?;
        fade_in_all(stage, &[tag], 0.4)?;

        // 08_5: every toss beaten when seen first; fresh tosses still catch
        // each of those wrong Cs half the time
        let line = say(&kit, stage, "08_5")?;
        flash_all(stage, &verdicts, 0.6)?;
        let tally = [
            kit.text(
                stage,
                &seen_text.tally[0],
                22.0,
                palette::mistake(),
                v(TALLY_X, TALLY_YS[0]),
            )?,
            kit.text(
                stage,
                &seen_text.tally[1],
                22.0,
                Palette::dim(),
                v(TALLY_X, TALLY_YS[1]),
            )?,
        ];
        fade_in_all(stage, &tally, 0.4)?;
        line.wait_for(stage, "Yet against fresh tosses")?;
        let fresh = kit.text(
            stage,
            &seen_text.fresh,
            PANEL_SIZE,
            palette::caught(),
            v(FRESH_X, FRESH_Y),
        )?;
        swap(stage, &[tag], &[fresh], 0.4)?;

        // 08_6: the protection was that the toss was not known
        let line = say(&kit, stage, "08_6")?;
        line.wait_for(stage, "couldn't know")?;
        let mut dims = Vec::with_capacity(all_coins.len());
        for &m in &all_coins {
            dims.push(fade_to(m, COINS_DIM, 0.4)?);
        }
        together(stage, dims)?;
        let mut unknown = Vec::with_capacity(spots.len());
        for k in 0..spots.len() {
            let (x, y) = spot_place(k);
            unknown.push(kit.text(stage, "?", 28.0, Palette::hero(), v(x + UNKNOWN_DX, y))?);
        }
        fade_in_lagged(stage, &unknown, 0.6, 0.2)?;

        // 08_7: for programmers, once the seen card is read
        say(&kit, stage, "08_7")?;
        wait_until(stage, card_up + read_s(card_text))?;
        let coins_card = lower_third(&kit, stage, PROGRAMMERS, COINS_CARD)?;
        fade_out_all(stage, &[seen], 0.2)?;
        fade_in_all(stage, &[coins_card], 0.2)?;
        let coins_up = now(stage);
        stand(&kit, stage, coins_up)?;
        wait_until(stage, coins_up + read_s(COINS_CARD))?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The words at the right of the grid say what the model and the
    /// theorem's bound say.
    #[test]
    fn the_panel_is_built_from_the_model() {
        let p = panel().unwrap();
        assert_eq!(p.one_toss, "one toss: 4 of 8 miss");
        assert_eq!(p.question, ["How many of these 64 ways", "miss both times?"]);
        assert_eq!(p.answer, "16 of 64 ways miss both times");
        assert_eq!(p.product, r"4 \times 4 = 16");
        assert_eq!(p.this_slip, r"\text{this slip: } \frac{16}{64} = \frac{1}{4}");
        assert_eq!(
            p.any_wrong,
            r"\text{any wrong } C\text{: } \le \frac{1}{2} \cdot \frac{1}{2} = \frac{1}{4}"
        );
        assert_eq!(p.reused, "same toss twice: 4 of 8, still 1/2");
    }

    /// The ladder is the theorem's bound, and a chance too small to see is
    /// labelled, never drawn wider than it is (about a pixel at 1080p).
    #[test]
    fn the_ladder_draws_only_bars_it_can_show() {
        use fmn::core::constants::FRAME_Y_RADIUS;
        assert_eq!(ladder_tex(1, 2).unwrap(), r"\le \frac{1}{2}");
        assert_eq!(ladder_tex(2, 2).unwrap(), r"\le \frac{1}{4}");
        assert_eq!(
            ladder_tex(AGAIN, 2).unwrap(),
            r"\le \frac{1}{1{,}048{,}576} < \frac{1}{1{,}000{,}000}"
        );
        // a check that never halves the chance would not beat a million
        assert!(ladder_tex(AGAIN, 1).is_err());
        assert_eq!(bar_width(2), Some(LADDER_W / 2.0));
        assert_eq!(bar_width(1 << AGAIN), None);
        const { assert!(MIN_BAR_W >= 2.0 * FRAME_Y_RADIUS / 1080.0, "a pixel at 1080p") };
    }

    /// The cards keep every condition, word for word, and fit two lines.
    #[test]
    fn the_cards_keep_their_conditions() {
        let again = again_card().unwrap();
        assert_eq!(
            again,
            "in exact arithmetic, with fair coins, all independent and tossed after C is fixed, k tosses all miss \
             any fixed wrong C with chance at most 1/2 to the k."
        );
        let spots = facts::blind_spots().unwrap();
        assert_eq!(seen_card(&spots).unwrap(), SEEN_CARD);
        assert!(MEANS_CARD.contains("for a fixed C"), "{MEANS_CARD}");
        assert!(MEANS_CARD.contains("not the chance that C is wrong"), "{MEANS_CARD}");
        assert!(COINS_CARD.contains("drawn after C is fixed"), "{COINS_CARD}");
        for text in [again.as_str(), SEEN_CARD, MEANS_CARD, COINS_CARD] {
            for width in [80, 76] {
                let lines = explainer_kit::animatic::wrap(text, width, 3);
                assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
            }
            assert!(read_s(text) < 12.0, "{text}");
            assert!(!text.contains('^'), "{text}");
        }
        // the names card keeps the name and fits one line
        assert_eq!(NAMES[0], NAME);
        assert_eq!(explainer_kit::animatic::wrap(&NAMES.join(" · "), 80, 3).len(), 1);
        // one by one, a heads coin has no blind spot: "from two by two up"
        assert_eq!(model::blind_spot(&[true]), None);
    }

    /// The ways both tosses miss fill the grid's corner, four by four, and
    /// the diagonal (one toss reused) holds as many as one toss misses.
    #[test]
    fn the_ways_both_miss_fill_the_corner() {
        let order = facts::miss_first();
        let passes: Vec<bool> = order.iter().map(|t| model::check(&A, &B, &C, t).passes()).collect();
        let misses = passes.iter().filter(|&&p| p).count();
        assert_eq!((order.len(), misses), (TOSS_COUNT, 4));
        let mut corner = 0;
        for (i, &first) in passes.iter().enumerate() {
            for (j, &second) in passes.iter().enumerate() {
                if first && second {
                    assert!(i < misses && j < misses, "({i}, {j})");
                    corner += 1;
                }
            }
        }
        assert_eq!(corner, model::both_pass(&A, &B, &C).0);
        assert_eq!(misses, model::same_toss_twice(&A, &B, &C).0);
        assert!(way_at(0, 1)[0] > way_at(0, 0)[0]);
        assert!(way_at(1, 0)[1] < way_at(0, 0)[1]);
    }

    /// Every toss seen first is beaten: one wrong entry under a tails coin,
    /// or under all heads a row that cancels; fresh tosses catch each of
    /// those wrong Cs half the time. The toss 08_2 asks about has one tails
    /// coin, over the third column.
    #[test]
    fn every_toss_seen_first_is_beaten() {
        let spots = facts::blind_spots().unwrap();
        assert_eq!(spots.len(), TOSS_COUNT);
        for (t, d) in &spots {
            if t.contains(&false) {
                let wrong = model::wrong_entries(d);
                assert_eq!(wrong.len(), 1, "{}", model::toss_text(t));
                assert!(wrong.iter().all(|&(_, j)| !t[j]), "{}", model::toss_text(t));
            }
        }
        assert_eq!(
            spots.iter().position(|(t, _)| !t.contains(&false)),
            Some(TOSS_COUNT - 1)
        );
        let w = seen_words(&spots).unwrap();
        assert_eq!(w.tally, ["8 of 8 tosses beaten", "when seen first"]);
        assert_eq!(w.cancel, "1 + (\u{2212}1) = 0");
        assert_eq!(w.cancel_row, 0);
        assert_eq!(w.fresh, "fresh tosses catch each of these wrong Cs: 4 of 8");
        assert_eq!(model::toss_text(&FIRST_TOSS), "110");
        assert_eq!(FIRST_TOSS.iter().position(|&h| !h), Some(2));
    }

    /// Every phrase the scenes wait for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("07_1", "So toss again"),
            ("07_2", "How many of them"),
            ("07_3", "four ways for the first toss"),
            ("07_3", "times four for the second"),
            ("07_4", "Any wrong C"),
            ("07_5", "Four of the eight"),
            ("07_6", "Twenty fresh tosses"),
            ("07_7", "It doesn't say"),
            ("07_8", "Once a toss catches C"),
            ("08_1", "Someone who sees your toss first"),
            ("08_2", "Where could they"),
            ("08_3", "The toss never looks there"),
            ("08_4", "That's the sum of every column again"),
            ("08_5", "Yet against fresh tosses"),
            ("08_6", "couldn't know"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The layout keeps its pieces apart and inside the title-safe area.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        use super::super::draw::{BRACKET_GAP, DX, DY};
        use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
        const CARD_2: f64 = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0;
        const CARD_3: f64 = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 3.0;
        const HALF: f64 = 1.5 * DX + BRACKET_GAP;
        // the grid of ways over even a three-line card, its titles under the top
        const { assert!(WAYS_Y - WAYS_HALF - 0.1 > CARD_3, "grid over the card") };
        const {
            assert!(
                SECOND_TITLE_AT[1] + 0.15 < FRAME_Y_RADIUS - 0.15,
                "titles under the top edge"
            )
        };
        const {
            assert!(
                FIRST_TITLE_AT[0] + 0.7 < WAYS_X - WAYS_HALF - LEFT_LABEL_DX - 0.2,
                "title left of the labels"
            )
        };
        const { assert!(SLIP_Y - 1.5 * DY - 0.1 > WAYS_Y + 0.2, "slip over the first title") };
        const { assert!(SLIP_NAME_Y - 0.25 > WAYS_Y + 0.2, "slip's name over the first title") };
        const { assert!(SLIP_X - HALF > -FRAME_X_RADIUS + 0.15, "slip inside the left edge") };
        const { assert!(3.0 * 0.11 < WAY - WAY_GAP, "labels apart") };
        const { assert!(PANEL_X > WAYS_X + WAYS_HALF + 0.5, "panel right of the grid") };
        const { assert!(PANEL_YS[5] - 0.15 > CARD_2, "panel over the card") };
        const { assert!(KEY_Y - 0.15 > PANEL_YS[0] + 0.2, "key over the one-toss line") };
        const { assert!(KEY_Y + 0.15 < FRAME_Y_RADIUS - 0.15, "key under the top edge") };
        const KEY_W: f64 = (KEY_NOTE.len() + MISS.len() + CAUGHT.len()) as f64 * 0.12 + 2.0 * KEY_GAP;
        const { assert!(PANEL_X + KEY_W < FRAME_X_RADIUS - 0.15, "key inside the right edge") };
        const { assert!(THINK_AT[0] - 0.62 > WAYS_X + WAYS_HALF + 0.3, "ring right of the grid") };
        // the ladder over a two-line card; what it means over the ladder
        const { assert!(LADDER_YS[2] - 0.4 > CARD_2, "ladder over the card") };
        const { assert!(LADDER_LABEL_X >= -FRAME_X_RADIUS + 0.25) };
        const { assert!(ONE_WAY_YS[1] - 0.2 > LADDER_YS[0] + 0.2, "lines over the ladder") };
        const { assert!(MEANS_YS[1] - 0.2 > ONE_WAY_YS[0] + 0.2, "meaning over the one way") };
        const { assert!(MEANS_YS[0] + 0.2 < FRAME_Y_RADIUS - 0.15, "meaning under the top edge") };
        // the tosses seen first: coins, grids and verdicts apart and inside
        const {
            assert!(
                SPOT_COIN_YS[0] + COIN_R <= FRAME_Y_RADIUS - 0.15,
                "coins inside the safe area"
            )
        };
        const { assert!(SPOT_GRID_DY + DY + 0.15 < -COIN_R - 0.05, "grid under its coins") };
        const {
            assert!(
                SPOT_VERDICT_DY + 0.15 < SPOT_GRID_DY - 1.5 * DY,
                "verdict under its grid"
            )
        };
        const {
            assert!(
                SPOT_COIN_YS[0] + SPOT_VERDICT_DY - 0.15 > SPOT_COIN_YS[1] + COIN_R + 0.05,
                "rows apart"
            )
        };
        const { assert!(SPOT_XS[1] - SPOT_XS[0] > 2.0 * HALF + 0.3, "grids apart") };
        const { assert!(SPOT_XS[0] - HALF > -FRAME_X_RADIUS + 0.15, "grids inside the left edge") };
        const {
            assert!(
                FRESH_Y + 0.15 < SPOT_COIN_YS[1] + SPOT_VERDICT_DY - 0.15,
                "fresh line under the verdicts"
            )
        };
        const { assert!(FRESH_Y - 0.15 > CARD_2, "fresh line over the card") };
        const {
            assert!(
                TALLY_X - 1.4 > SPOT_XS[3] + DX + COIN_R + 0.1,
                "tally right of the last coins"
            )
        };
        const { assert!(TALLY_X + 1.4 < FRAME_X_RADIUS - 0.15, "tally inside the right edge") };
        // the ring around the toss seen first stays off its neighbours' coins
        const { assert!(SEEN_R < SPOT_XS[1] - SPOT_XS[0] - DX - COIN_R, "ring off the next toss") };
        const {
            assert!(
                SEEN_THINK_AT[0] - 0.62 > SPOT_XS[3] + HALF + 0.1,
                "ring right of the grids"
            )
        };
        const {
            assert!(
                SPOT_XS[0] + DX + COIN_R + 0.1 < SPOT_XS[0] + UNKNOWN_DX - 0.15,
                "? right of its coins"
            )
        };
        const {
            assert!(
                SPOT_XS[0] + UNKNOWN_DX + 0.15 < SPOT_XS[1] - DX - COIN_R,
                "? left of the next toss"
            )
        };
        const { assert!(SPOT_XS[3] + UNKNOWN_DX + 0.15 < TALLY_X - 1.4, "? left of the tally") };
        const NOTE_HALF: f64 = GRID_NOTE.len() as f64 * 0.12 / 2.0;
        const { assert!(GRID_NOTE_AT[0][1] + 0.15 < TALLY_YS[1] - 0.15, "label under the tally") };
        const { assert!(GRID_NOTE_AT[1][1] - GRID_NOTE_AT[0][1] < -0.35, "name under the note") };
        const {
            assert!(
                GRID_NOTE_AT[1][1] - 0.2 > SPOT_COIN_YS[1] + SPOT_GRID_DY + 1.5 * DY,
                "label over the last grid"
            )
        };
        const {
            assert!(
                GRID_NOTE_AT[0][0] - NOTE_HALF > SPOT_XS[3] + HALF + 0.2,
                "label right of the grids"
            )
        };
        const {
            assert!(
                GRID_NOTE_AT[0][0] + NOTE_HALF < FRAME_X_RADIUS - 0.15,
                "label inside the right edge"
            )
        };
    }
}
