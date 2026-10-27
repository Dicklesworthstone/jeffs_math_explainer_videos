//! Chapter 07, "Right on average, for any count": on the friends' line as
//! chapter 06 left it, the count becomes a letter, N, and the five counts
//! and the top gap add up to N minus five (07_1); the matching from the
//! table holds for handfuls of five (07_2); every handful, and so every
//! list, equally likely, so every gap averages N minus five, over six, held
//! to be read (07_3). The recipe can't see the top gap, so it guesses it
//! with the average of the five it can see: eleven for the friends, a guess
//! arc over sixty-one to seventy-one (07_4). Eleven is one handful's
//! average: in the small company the guess and the top gap agree in only
//! three pairs of ten, held to be inspected (07_5), yet both average one,
//! and over all handfuls the guess averages what the top gap does, held to
//! be read (07_6). So the estimate minus the count is the guess minus the
//! top gap, which averages zero: unbiased for every count, held to be read
//! (07_7). The prediction on a handful of one box (07_8); it doubles the
//! number and subtracts one, thirty-seven to seventy-three (07_9); and the
//! general handful size, with card D (07_10).
//!
//! Each line on the panel is built from the facts and the scene checks it
//! against them; nothing shows the one-box answer before the prediction is
//! answered. Highlights are halos and colour changes are replacements: no
//! typeset text is indicated, recoloured or scaled in place.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, choose, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    STRIP_Y, TICK_W, fade_in_all, fade_in_lagged, fade_out_all, fail, line_style, move_all, pause, say, stroke, think,
    wait_until, x_at,
};
use super::friends_line::{
    ARC_H, ARC_LIFT, ARC_W, BADGE_AT, PANEL_SIZE, PANEL_X, PANEL_Y, THINK_AT, arc_points, at, badge, gap_spans,
    handfuls, is_handful, list, now, panel, printed, read_s, whole,
};
use super::gaps_trade_places::{GapLine, glow_each, list_of, matched};
use super::what_we_assume::{
    HALO_HOLD_S, HALO_IN_S, formula, formula_colors, glow, halo, instance, replace, roles, sum_texts, words_left,
};
use crate::facts;
use crate::model::{LINE_DOTS, Q};
use crate::palette;

pub struct RightOnAverage {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const COUNT_N: &str = "count = N";
const HANDFUL: &str = "handful";
const TOP_GAP: &str = "top gap";
const GUESS: &str = "guess";
const UNBIASED: &str = "unbiased: average estimate = count";
const COUNT_IS: &str = "count = largest + top gap";
const ESTIMATE_IS: &str = "estimate = largest + guess";
const DIFFERENCE: &str = "estimate − count = guess − top gap";
const TWICE: &str = "= twice the largest −";
/// The general handful size (07_10): its gaps, each gap's average and the
/// recipe, with `k` for the size.
const GAPS_K: &str = "k numbers: k + 1 gaps";
const EACH_K: &str = "each gap averages (N − k) ÷ (k + 1)";
const RECIPE_K: &str = "estimate = largest + (largest − k) ÷ k";
const CARD_D: &str = "for a fair handful of k from boxes 1 to N, the top gap and the guess (largest − k) ÷ k both \
                      average (N − k) ÷ (k + 1), so the estimates average exactly N, for every N of at least k.";
/// The inset's rows (07_5): the pairs, each one's guess and its top gap.
const INSET_LABELS: [&str; 3] = ["pair", GUESS, TOP_GAP];

/// The small company's even share (07_3) at the upper left, under the
/// badge, and the definition (07_7) where it was once the inset goes.
const SMALL_AT: [f64; 2] = [-6.3, 2.85];
const UNBIASED_AT: [f64; 2] = [-6.3, 2.6];
/// The guess's arc (07_4, 07_9), at half the strength of a seen gap's, and
/// its count over it.
const GUESS_O: f64 = 0.5;
const GUESS_DY: f64 = 0.2;
/// The inset (07_5, 07_6): its labels' left edge, its first value and the
/// step between pairs, its three rows, its size, and each row's average.
const INSET_LABEL_X: f64 = -6.3;
const INSET_X0: f64 = -4.9;
const INSET_STEP: f64 = 0.47;
const INSET_Y: [f64; 3] = [1.75, 1.35, 0.95];
const INSET_SIZE: f64 = 20.0;
const INSET_AVERAGE_X: f64 = -0.3;
/// An estimate's tick across the line and its number over it (as chapter
/// 04 draws them).
const TICK_DOWN: f64 = 0.12;
const TICK_UP: f64 = 0.3;
const ESTIMATE_DY: f64 = 0.55;
/// The inspection holds (07_3, 07_5, 07_6, 07_7) and the beat after the
/// last line.
const HOLD_S: f64 = 2.5;
const END_BEAT: f64 = 0.6;

/// What chapter 07 shows, from the facts: the friends' five, the line's
/// first number, the handful size and the largest; their gaps, the unseen
/// total, the average gap and the estimate; how many gaps with the top one
/// and how many seen; each gap's average as a formula in N; the tiny
/// company, its pairs, each pair's guess as the inset shows it, its top
/// gap, how many agree and both averages; and the one-box handful, its
/// size, number, gap below, how many gaps, its estimate and its recipe.
#[derive(Clone, Debug)]
struct Picture {
    numbers: Vec<u64>,
    first: u64,
    k: usize,
    largest: u64,
    gaps: Vec<u64>,
    unseen: u64,
    average_gap: u64,
    estimate: u64,
    positions: usize,
    seen_gaps: usize,
    gap_average: String,
    boxes: u64,
    seen: usize,
    pairs: Vec<Vec<u64>>,
    guesses: Vec<String>,
    tops: Vec<u64>,
    agree: usize,
    guess_average: u64,
    top_average: u64,
    one_handful: usize,
    one: u64,
    one_below: u64,
    one_gaps: usize,
    one_estimate: u64,
    one_recipe: String,
}

fn picture() -> Result<Picture, String> {
    let f = facts::facts()?;
    checked(Picture {
        numbers: list(&f, "G-F1.numbers")?,
        first: whole(&f, "G-F1.first")?,
        k: whole(&f, "G-F1.handful")? as usize,
        largest: whole(&f, "G-F1.largest")?,
        gaps: list(&f, "G-F3.gaps")?,
        unseen: whole(&f, "G-F3.unseen")?,
        average_gap: whole(&f, "G-F3.average_gap")?,
        estimate: whole(&f, "G-F3.estimate")?,
        positions: whole(&f, "G-F6.friends_positions")? as usize,
        seen_gaps: whole(&f, "G-F6.friends_seen_gaps")? as usize,
        gap_average: printed(&f, "G-F7.gap_average_formula")?,
        boxes: whole(&f, "G-F5.boxes")?,
        seen: whole(&f, "G-F5.seen")? as usize,
        pairs: handfuls(&f, "G-F5.pair_list")?,
        guesses: printed(&f, "G-F7.tiny_guesses_shown")?
            .split(',')
            .map(str::to_owned)
            .collect(),
        tops: list(&f, "G-F7.tiny_tops")?,
        agree: whole(&f, "G-F7.tiny_agree")? as usize,
        guess_average: whole(&f, "G-F7.tiny_guess_average")?,
        top_average: whole(&f, "G-F7.tiny_top_average")?,
        one_handful: whole(&f, "G-F7.one_box_handful")? as usize,
        one: whole(&f, "G-F7.one_box")?,
        one_below: whole(&f, "G-F7.one_box_below")?,
        one_gaps: whole(&f, "G-F7.one_box_gaps")? as usize,
        one_estimate: whole(&f, "G-F7.one_box_estimate")?,
        one_recipe: printed(&f, "G-F7.one_box_recipe")?,
    })
}

/// How the inset shows an exact guess: a whole number as it is, any other
/// with its one decimal ("1.5") when that is exact.
fn shown_of(q: Q) -> Result<String, String> {
    match q.whole() {
        Some(w) => Ok(w.to_string()),
        None => q.decimal1(),
    }
}

/// A pair's guess for its top gap: the average of its gaps below its
/// larger number.
fn guess_of(pair: &[u64], boxes: u64) -> Q {
    let l = list_of(pair, boxes);
    let below: u64 = l[..l.len() - 1].iter().sum();
    Q::new(below as i128, pair.len() as i128)
}

/// The picture only if the scene can draw it truthfully: the friends' five
/// are a handful on the line from 1, the largest last, with five non-empty
/// gaps below it holding the unseen total, its average whole, and the
/// estimate the largest plus that; one more gap with the top one; each
/// gap's average is (N − k) over k + 1; the tiny company's pairs are every
/// pair once, in order, each guess the average of its gaps below and shown
/// exactly, each top gap the boxes above its larger number; they agree in
/// the rows counted, which are not all of them, and both average the even
/// share; the one-box handful has one box on the line with its gap below
/// and its top gap, and its estimate twice its number less one, on the line.
fn checked(pic: Picture) -> Result<Picture, String> {
    let k = pic.k as u64;
    let sizes: Vec<u64> = gap_spans(&pic.numbers).iter().map(|&(a, b)| b - a + 1).collect();
    if pic.first != 1
        || !is_handful(&pic.numbers, pic.k, LINE_DOTS)
        || pic.numbers.last() != Some(&pic.largest)
        || pic.gaps != sizes
        || pic.gaps.len() != pic.k
        || pic.seen_gaps != pic.k
        || pic.positions != pic.k + 1
        || pic.gaps.iter().sum::<u64>() != pic.unseen
        || pic.unseen + k != pic.largest
    {
        return Err(format!(
            "the friends' {:?} must be {} numbers with the gaps {:?}, holding {}, and {} gaps with the top one",
            pic.numbers, pic.k, pic.gaps, pic.unseen, pic.positions
        ));
    }
    if pic.average_gap * k != pic.unseen || pic.estimate != pic.largest + pic.average_gap {
        return Err(format!(
            "the guess {} must be the average of the five gaps, the estimate {} the largest plus it",
            pic.average_gap, pic.estimate
        ));
    }
    if pic.gap_average != format!("(N-{k})/{}", pic.positions) {
        return Err(format!(
            "each gap must average (N − {k}) ÷ {}, not {}",
            pic.positions, pic.gap_average
        ));
    }
    if pic.pairs.len() as u128 != choose(pic.boxes, pic.seen as u64)
        || !pic.pairs.iter().all(|p| is_handful(p, pic.seen, pic.boxes))
        || pic.pairs.windows(2).any(|w| w[0] >= w[1])
    {
        return Err(format!(
            "the pairs {:?} must be every pair from {} boxes once, in order",
            pic.pairs, pic.boxes
        ));
    }
    let guesses: Vec<Q> = pic.pairs.iter().map(|p| guess_of(p, pic.boxes)).collect();
    let tops: Vec<u64> = pic.pairs.iter().map(|p| pic.boxes - p[p.len() - 1]).collect();
    let shown: Result<Vec<String>, String> = guesses.iter().map(|&q| shown_of(q)).collect();
    if pic.guesses != shown? || pic.tops != tops {
        return Err(format!(
            "the guesses {:?} and top gaps {:?} must be each pair's average gap below and boxes above",
            pic.guesses, pic.tops
        ));
    }
    let rows = Q::int(pic.pairs.len() as i128);
    let agree = guesses
        .iter()
        .zip(&tops)
        .filter(|&(&q, &t)| q == Q::int(t as i128))
        .count();
    let share = Q::new((pic.boxes - pic.seen as u64) as i128, pic.seen as i128 + 1);
    let mean_guess = guesses.iter().fold(Q::ZERO, |s, &q| s + q) / rows;
    let mean_top = Q::int(tops.iter().sum::<u64>() as i128) / rows;
    if pic.agree != agree
        || agree >= pic.pairs.len()
        || mean_guess != Q::int(pic.guess_average as i128)
        || mean_top != Q::int(pic.top_average as i128)
        || mean_guess != share
        || mean_top != share
    {
        return Err(format!(
            "the guess and the top gap agree in {} of {} rows, and average {} and {}, both {share}",
            pic.agree,
            pic.pairs.len(),
            pic.guess_average,
            pic.top_average
        ));
    }
    let one = [pic.one];
    if pic.one_handful != one.len()
        || pic.one_below != pic.one - 1
        || pic.one_gaps != one.len() + 1
        || pic.one_estimate != pic.one + pic.one_below / one.len() as u64
        || pic.one_estimate != 2 * pic.one - 1
        || pic.one_recipe != format!("2m-{}", pic.one_handful)
        || !is_handful(&one, 1, LINE_DOTS)
        || pic.one_estimate >= LINE_DOTS
    {
        return Err(format!(
            "the one box {} must leave {} below and {} gaps, its estimate {} twice it less one, on the line",
            pic.one, pic.one_below, pic.one_gaps, pic.one_estimate
        ));
    }
    Ok(pic)
}

// ------------------------------------------------------------------ words

/// 07_1's sum: the five counts and the top gap add up to N minus the
/// handful size; the counts blue, the top gap hero.
fn all_gaps(gaps: &[u64], k: usize) -> Vec<(String, Srgb)> {
    let mut out = Vec::new();
    for g in gaps {
        out.push((g.to_string(), palette::unseen()));
        out.push(("+".to_owned(), Palette::ink()));
    }
    out.push((TOP_GAP.to_owned(), palette::top_gap()));
    out.push(("=".to_owned(), Palette::ink()));
    out.push((format!("N − {k}"), Palette::ink()));
    out
}

/// 07_2's matching: a handful and its list of gaps.
fn lists_word(positions: usize, k: usize) -> String {
    format!("{positions} gaps adding to N − {k}")
}

/// Each gap's average, (N − k) over the gaps (07_3), as the panel writes it.
fn each_line(k: usize, positions: usize) -> String {
    format!("each gap averages {}", average_of(k, positions))
}

fn average_of(k: usize, positions: usize) -> String {
    format!("(N − {k}) ÷ {positions}")
}

/// The small company's even share (07_3).
fn small_line(unseen: u64, positions: usize, each: u64) -> String {
    format!("small company: {unseen} ÷ {positions} = {each}")
}

/// 07_4's guess for the friends, piece by piece: the guess and the counts
/// blue, the rest ink.
fn guess_pieces(gaps: &[u64], k: usize, guess: u64) -> Vec<(String, Srgb)> {
    let mut out = vec![
        (GUESS.to_owned(), palette::unseen()),
        ("=".to_owned(), Palette::ink()),
        ("(".to_owned(), Palette::ink()),
    ];
    for (i, g) in gaps.iter().enumerate() {
        if i > 0 {
            out.push(("+".to_owned(), Palette::ink()));
        }
        out.push((g.to_string(), palette::unseen()));
    }
    out.extend([
        (")".to_owned(), Palette::ink()),
        ("÷".to_owned(), Palette::ink()),
        (k.to_string(), Palette::ink()),
        ("=".to_owned(), Palette::ink()),
        (guess.to_string(), palette::unseen()),
    ]);
    out
}

/// 07_6: the guess averages what each gap does.
fn guess_too(k: usize, positions: usize) -> String {
    format!("{GUESS}: averages {} too", average_of(k, positions))
}

/// 07_7's last step: the two averages cancel.
fn on_average(k: usize, positions: usize) -> String {
    let a = average_of(k, positions);
    format!("on average: {a} − {a} = 0")
}

/// The one-box recipe (07_9): twice the largest less the handful size.
fn twice_line(k: usize) -> String {
    format!("{TWICE} {k}")
}

/// The friends' case under the general recipe (07_10).
fn friends_case(k: usize, largest: u64, unseen: u64, estimate: u64) -> String {
    format!("k = {k}: {largest} + {unseen} ÷ {k} = {estimate}")
}

/// A pair as the inset shows it.
fn pair_label(pair: &[u64]) -> String {
    let shown: Vec<String> = pair.iter().map(u64::to_string).collect();
    shown.join(",")
}

/// The names of a derivation line in their roles' colours.
fn derivation_colors(text: &str) -> Vec<(String, Srgb)> {
    roles(
        text,
        &[
            ("estimate".to_owned(), palette::estimate()),
            ("largest".to_owned(), palette::seen()),
            (TOP_GAP.to_owned(), palette::top_gap()),
            (GUESS.to_owned(), palette::unseen()),
        ],
    )
}

// ------------------------------------------------------------ positions

/// Pair `j`'s column in the inset.
fn inset_x(j: usize) -> f64 {
    INSET_X0 + INSET_STEP * j as f64
}

// ------------------------------------------------------------- builders

/// The inset (07_5, 07_6): three rows, each its label then one value per
/// pair, the pairs dim, the guesses blue, the top gaps hero; and the
/// guesses' and the top gaps' averages after their rows. Detached.
struct Inset {
    rows: Vec<Vec<Mob>>,
    averages: Vec<Mob>,
}

impl Inset {
    fn build(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture) -> fmn::Result<Inset> {
        let colors = [Palette::dim(), palette::unseen(), palette::top_gap()];
        let values: [Vec<String>; 3] = [
            pic.pairs.iter().map(|p| pair_label(p)).collect(),
            pic.guesses.clone(),
            pic.tops.iter().map(u64::to_string).collect(),
        ];
        let mut rows = Vec::new();
        for (r, (label, color)) in INSET_LABELS.iter().zip(colors).enumerate() {
            let y = INSET_Y[r];
            let mut row = vec![kit.text_left(stage, label, INSET_SIZE, color, v(INSET_LABEL_X, y))?];
            for (j, s) in values[r].iter().enumerate() {
                row.push(kit.text(stage, s, INSET_SIZE, color, v(inset_x(j), y))?);
            }
            rows.push(row);
        }
        let mut averages = Vec::new();
        for (r, a) in [(1, pic.guess_average), (2, pic.top_average)] {
            averages.push(kit.text_left(
                stage,
                &format!("average {a}"),
                INSET_SIZE,
                colors[r],
                v(INSET_AVERAGE_X, INSET_Y[r]),
            )?);
        }
        Ok(Inset { rows, averages })
    }

    /// Pair `j`'s column: its pair, guess and top gap.
    fn column(&self, j: usize) -> Vec<Mob> {
        self.rows.iter().map(|r| r[j + 1]).collect()
    }

    fn all(&self) -> Vec<Mob> {
        let mut all: Vec<Mob> = self.rows.iter().flatten().copied().collect();
        all.extend(&self.averages);
        all
    }
}

/// A guess's arc over numbers `a` to `b` on a line shown to `end`, at half
/// strength. Detached.
fn guess_arc(stage: &mut Stage<'_>, a: u64, b: u64, end: f64) -> Mob {
    stroke(
        stage,
        arc_points(x_at(a as f64 - 0.5, end), x_at(b as f64 + 0.5, end), STRIP_Y, ARC_H),
        line_style(palette::unseen(), ARC_W, GUESS_O),
    )
}

/// Where a guess's count sits, over the middle of its arc.
fn over_arc(a: u64, b: u64, end: f64) -> Vec3 {
    v(
        (x_at(a as f64 - 0.5, end) + x_at(b as f64 + 0.5, end)) / 2.0,
        STRIP_Y + ARC_LIFT + ARC_H + GUESS_DY,
    )
}

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

impl SceneConstruct for RightOnAverage {
    fn name(&self) -> &str {
        "07_right_on_average"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let pic = picture().map_err(fail)?;
        let end = LINE_DOTS as f64;

        // the friends' line as chapter 06 left it, every gap drawn, and the
        // badge
        let line6 = GapLine::build(&kit, stage, pic.first, LINE_DOTS, &pic.numbers)?;
        let corner = badge(&kit, stage, at(BADGE_AT))?;
        let mut opened = line6.all();
        opened.push(corner);
        fade_in_all(stage, &opened, 0.8)?;

        // 07_1: the count is a letter; the gaps add up to it less the five,
        // piece by piece; halos round the counts seen and the top gap
        let line = say(&kit, stage, "07_1")?;
        line.wait_for(stage, "Call the count N")?;
        let count = words_left(&kit, stage, COUNT_N, &[], panel(0))?;
        fade_in_all(stage, &[count], 0.5)?;
        line.wait_for(stage, "unseen boxes into six gaps")?;
        let sum = sum_texts(&kit, stage, &all_gaps(&pic.gaps, pic.k), PANEL_X, PANEL_Y[1])?;
        fade_in_lagged(stage, &sum, 1.6, 0.2)?;
        line.wait_for(stage, "five they can see")?;
        glow(stage, &line6.counts, palette::unseen())?;
        line.wait_for(stage, "the top gap they can't")?;
        glow(stage, &[line6.unknown], palette::top_gap())?;

        // 07_2: the matching for handfuls of five; a halo round the boxes
        let line = say(&kit, stage, "07_2")?;
        line.wait_for(stage, "The matching from the table")?;
        let matching = matched(
            &kit,
            stage,
            (HANDFUL, palette::seen()),
            (&lists_word(pic.positions, pic.k), palette::unseen()),
            panel(2),
        )?;
        fade_in_all(stage, &matching, 0.5)?;
        line.wait_for(stage, "builds back its own handful")?;
        glow(stage, &line6.strip.bodies, palette::seen())?;

        // 07_3: the condition, a halo round the badge; the total, a halo
        // round it; each gap's average, and the small company's beside it;
        // held to be read
        let line = say(&kit, stage, "07_3")?;
        line.wait_for(stage, "Every handful is equally likely")?;
        glow(stage, &[corner], Palette::ink())?;
        line.wait_for(stage, "Together they always hold")?;
        let total = sum[sum.len() - 1];
        glow(stage, &[total], Palette::ink())?;
        line.wait_for(stage, "so each gap averages")?;
        let each = words_left(&kit, stage, &each_line(pic.k, pic.positions), &[], panel(3))?;
        let tiny_unseen = pic.boxes - pic.seen as u64;
        let small = kit.text_left(
            stage,
            &small_line(tiny_unseen, pic.seen + 1, pic.top_average),
            PANEL_SIZE,
            Palette::dim(),
            at(SMALL_AT),
        )?;
        fade_in_all(stage, &[each, small], 0.5)?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;

        // 07_4: the middle lines go and each gap's average moves up; the top
        // gap, a halo; the friends' guess piece by piece, and its arc over
        // the hidden top gap
        let line = say(&kit, stage, "07_4")?;
        let mut middle = sum.clone();
        middle.extend(&matching);
        fade_out_all(stage, &middle, 0.4)?;
        let w = stage.get_width(each);
        move_all(stage, &[(each, v(PANEL_X + w / 2.0, PANEL_Y[1]))], 1.0, 0.6)?;
        line.wait_for(stage, "That includes the top gap")?;
        glow(stage, &[line6.unknown], palette::top_gap())?;
        line.wait_for(stage, "the average of the five gaps")?;
        let guess = sum_texts(
            &kit,
            stage,
            &guess_pieces(&pic.gaps, pic.k, pic.average_gap),
            PANEL_X,
            PANEL_Y[2],
        )?;
        fade_in_lagged(stage, &guess, 1.6, 0.2)?;
        line.wait_for(stage, "that's eleven")?;
        let (a, b) = (pic.largest + 1, pic.largest + pic.average_gap);
        let arc = guess_arc(stage, a, b, end);
        let arc_count = kit.text(
            stage,
            &pic.average_gap.to_string(),
            PANEL_SIZE,
            palette::unseen(),
            over_arc(a, b, end),
        )?;
        fade_in_all(stage, &[arc, arc_count], 0.5)?;

        // 07_5: one handful's guess, a halo; the small company's pairs,
        // guesses and top gaps, row by row; halos round the pairs where
        // they agree, standing through the hold
        let line = say(&kit, stage, "07_5")?;
        line.wait_for(stage, "But eleven is")?;
        glow(stage, &[guess[guess.len() - 1]], palette::unseen())?;
        let inset = Inset::build(&kit, stage, &pic)?;
        for (r, phrase) in [
            "In the small company",
            "the average of its two gaps below",
            "Row by row",
        ]
        .into_iter()
        .enumerate()
        {
            line.wait_for(stage, phrase)?;
            fade_in_lagged(stage, &inset.rows[r], 1.0, 0.1)?;
        }
        line.wait_for(stage, "agree only three times")?;
        let mut agreeing = Vec::new();
        for j in 0..pic.pairs.len() {
            if pic.guesses[j] == pic.tops[j].to_string() {
                agreeing.push(halo(stage, &inset.column(j), Palette::ink())?);
            }
        }
        fade_in_all(stage, &agreeing, HALO_IN_S)?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;
        fade_out_all(stage, &agreeing, HALO_IN_S)?;

        // 07_6: both averages, a halo round them; the friends' guess
        // averages what each gap does; halos round the top gap and the
        // guess's arc; held to be read
        let line = say(&kit, stage, "07_6")?;
        line.wait_for(stage, "the guesses average one box")?;
        fade_in_all(stage, &[inset.averages[0]], 0.4)?;
        line.wait_for(stage, "so do the top gaps")?;
        fade_in_all(stage, &[inset.averages[1]], 0.4)?;
        glow(stage, &inset.averages, Palette::ink())?;
        line.wait_for(stage, "so the average of the five")?;
        let too_text = guess_too(pic.k, pic.positions);
        let too = words_left(
            &kit,
            stage,
            &too_text,
            &[(GUESS.to_owned(), palette::unseen())],
            panel(3),
        )?;
        fade_in_all(stage, &[too], 0.5)?;
        line.wait_for(stage, "exactly like the top gap")?;
        glow_each(
            stage,
            &[
                (vec![line6.unknown], palette::top_gap()),
                (vec![arc], palette::unseen()),
            ],
            HALO_IN_S,
            HALO_HOLD_S,
        )?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;

        // 07_7: the inset goes; the panel gives way to the derivation, line
        // by line; the definition at the upper left, a halo round it; held
        // to be read
        let line = say(&kit, stage, "07_7")?;
        let mut gone = inset.all();
        gone.push(small);
        fade_out_all(stage, &gone, 0.4)?;
        let steps = [
            COUNT_IS.to_owned(),
            ESTIMATE_IS.to_owned(),
            DIFFERENCE.to_owned(),
            on_average(pic.k, pic.positions),
        ];
        let mut derived = Vec::new();
        for (j, text) in steps.iter().enumerate() {
            derived.push(words_left(&kit, stage, text, &derivation_colors(text), panel(j))?);
        }
        let phrases = [
            "the count is the largest number",
            "the estimate is the largest number",
            "Over all handfuls",
            "have the same average",
        ];
        let mut panel_now = vec![count, each];
        panel_now.extend(&guess);
        panel_now.push(too);
        for (j, phrase) in phrases.into_iter().enumerate() {
            line.wait_for(stage, phrase)?;
            if j == 0 {
                replace(stage, &panel_now, &[derived[0]], 0.5)?;
            } else {
                fade_in_all(stage, &[derived[j]], 0.5)?;
            }
        }
        line.wait_for(stage, "The recipe is unbiased")?;
        let named = [("average estimate".to_owned(), palette::estimate())];
        let unbiased = words_left(&kit, stage, UNBIASED, &named, at(UNBIASED_AT))?;
        fade_in_all(stage, &[unbiased], 0.5)?;
        glow(stage, &[unbiased], Palette::ink())?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;

        // 07_8: everything but the badge goes; a fresh line with one box,
        // its arc below with no count, the top gap's bracket and its "?";
        // nothing shows the gap's size or an estimate; the countdown
        say(&kit, stage, "07_8")?;
        let mut cleared = line6.all();
        cleared.extend([arc, arc_count, unbiased]);
        cleared.extend(&derived);
        fade_out_all(stage, &cleared, 0.5)?;
        let lone = GapLine::build(&kit, stage, pic.first, LINE_DOTS, &[pic.one])?;
        let mut shown = lone.strip.opening();
        shown.extend(&lone.arcs);
        shown.extend(&lone.dashes);
        shown.push(lone.unknown);
        fade_in_all(stage, &shown, 0.8)?;
        let q = think(&kit, stage, at(THINK_AT))?;

        // 07_9: the recipe for one box, and twice the largest less one; halos
        // round the two gaps; the guess as long as the gap below; its count;
        // the sum piece by piece and the green tick at seventy-three
        let line = say(&kit, stage, "07_9")?;
        let alone = words_left(&kit, stage, &formula(pic.one_handful), &formula_colors(), panel(0))?;
        let twice_text = twice_line(pic.one_handful);
        let twice = words_left(
            &kit,
            stage,
            &twice_text,
            &[("largest".to_owned(), palette::seen())],
            panel(1),
        )?;
        replace(stage, &[q], &[alone, twice], 0.5)?;
        line.wait_for(stage, "One box leaves two gaps")?;
        glow_each(
            stage,
            &[
                (lone.arcs.clone(), palette::unseen()),
                (lone.dashes.clone(), palette::top_gap()),
            ],
            HALO_IN_S,
            HALO_HOLD_S,
        )?;
        line.wait_for(stage, "the only clue")?;
        let clue = guess_arc(stage, pic.one + 1, pic.one + pic.one_below, end);
        fade_in_all(stage, &[clue], 0.5)?;
        line.wait_for(stage, "thirty-six boxes below it")?;
        fade_in_all(stage, &lone.counts, 0.4)?;
        line.wait_for(stage, "the recipe says")?;
        let says = sum_texts(
            &kit,
            stage,
            &instance(pic.one, pic.one_below, pic.one_handful, &pic.one_estimate.to_string()),
            PANEL_X,
            PANEL_Y[2],
        )?;
        fade_in_lagged(stage, &says, 1.2, 0.3)?;
        let [tick, mark] = estimate_tick(&kit, stage, pic.one_estimate, end)?;
        fade_in_all(stage, &[tick, mark], 0.4)?;

        // 07_10: the panel clears; the general handful size line by line,
        // the friends' case in dim; a halo round the badge; card D, held
        // until it can be read
        let line = say(&kit, stage, "07_10")?;
        let mut panel_now = vec![alone, twice];
        panel_now.extend(&says);
        fade_out_all(stage, &panel_now, 0.4)?;
        line.wait_for(stage, "one more gap than there are numbers")?;
        let gaps_k = words_left(&kit, stage, GAPS_K, &[], panel(0))?;
        fade_in_all(stage, &[gaps_k], 0.5)?;
        line.wait_for(stage, "all with the same average")?;
        let each_k = words_left(&kit, stage, EACH_K, &[], panel(1))?;
        fade_in_all(stage, &[each_k], 0.5)?;
        line.wait_for(stage, "So the recipe always adds")?;
        let recipe_k = words_left(&kit, stage, RECIPE_K, &formula_colors(), panel(2))?;
        let case = kit.text_left(
            stage,
            &friends_case(pic.k, pic.largest, pic.unseen, pic.estimate),
            PANEL_SIZE,
            Palette::dim(),
            panel(3),
        )?;
        fade_in_all(stage, &[recipe_k, case], 0.5)?;
        line.wait_for(stage, "it's right on average")?;
        let card = precisely(&kit, stage, CARD_D)?;
        fade_in_all(stage, &[card], 0.4)?;
        let card_up = now(stage);
        line.wait_for(stage, "as long as the handful is fair")?;
        glow(stage, &[corner], Palette::ink())?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, END_BEAT)?;
        // card D stays until it can be read
        wait_until(stage, card_up + read_s(CARD_D))?;

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
    use crate::chapters::draw::{BOX_H, BOX_RISE, BOX_W, BRACKET_DY, SMALL};
    use crate::chapters::friends_line::{BADGE, BADGE_PAD, BADGE_SIZE, est_width};
    use crate::chapters::gaps_trade_places::{MATCH_ARROW, MATCH_GAP};
    use crate::chapters::what_we_assume::SUM_GAP;
    use crate::facts::{ONE_BOX, TINY, TINY_SEEN};
    use crate::model::{FRIENDS, all_handfuls, gap_recipe, gaps};

    /// A line's half height at the panel's size, at the inset's, and the
    /// space kept between things that must not touch.
    const HALF_LINE: f64 = 0.16;
    const HALF_INSET: f64 = 0.12;
    const CLEAR: f64 = 0.1;
    /// A "precisely:" card's line, beside its label: the kit starts at 92
    /// characters and narrows by 4 until the widest line fits.
    const CARD_CHARS_WIDEST: usize = 92;
    const CARD_CHARS_NARROWEST: usize = 64;

    fn good() -> Picture {
        picture().unwrap()
    }

    fn joined(pieces: &[(String, Srgb)]) -> String {
        pieces.iter().map(|(s, _)| s.as_str()).collect::<Vec<_>>().join(" ")
    }

    fn sum_width(pieces: &[(String, Srgb)]) -> f64 {
        pieces.iter().map(|(s, _)| est_width(s, PANEL_SIZE)).sum::<f64>() + SUM_GAP * (pieces.len() - 1) as f64
    }

    /// The scene draws the facts, and its lines read as the voice says
    /// them: the friends' five and their gaps are the model's; each pair's
    /// guess plus its larger number is the model's estimate; the general
    /// lines with k = 5 are the friends' lines.
    #[test]
    fn the_picture_is_the_facts() {
        let pic = good();
        assert_eq!(pic.numbers, FRIENDS);
        assert_eq!(pic.gaps, gaps(&FRIENDS));
        assert_eq!((pic.boxes, pic.seen), (TINY, TINY_SEEN));
        assert_eq!(pic.one, ONE_BOX);
        assert_eq!(gap_recipe(&[pic.one]).whole(), Some(pic.one_estimate as i128));
        for p in &pic.pairs {
            let larger = Q::int(p[p.len() - 1] as i128);
            assert_eq!(larger + guess_of(p, pic.boxes), gap_recipe(p), "{p:?}");
        }
        assert_eq!(
            joined(&all_gaps(&pic.gaps, pic.k)),
            "13 + 12 + 5 + 17 + 8 + top gap = N − 5"
        );
        assert_eq!(lists_word(pic.positions, pic.k), "6 gaps adding to N − 5");
        assert_eq!(each_line(pic.k, pic.positions), "each gap averages (N − 5) ÷ 6");
        assert_eq!(small_line(3, 3, 1), "small company: 3 ÷ 3 = 1");
        assert_eq!(
            joined(&guess_pieces(&pic.gaps, pic.k, pic.average_gap)),
            "guess = ( 13 + 12 + 5 + 17 + 8 ) ÷ 5 = 11"
        );
        assert_eq!(guess_too(pic.k, pic.positions), "guess: averages (N − 5) ÷ 6 too");
        assert_eq!(
            on_average(pic.k, pic.positions),
            "on average: (N − 5) ÷ 6 − (N − 5) ÷ 6 = 0"
        );
        assert_eq!(formula(pic.one_handful), "estimate = largest + (largest − 1) ÷ 1");
        assert_eq!(twice_line(pic.one_handful), "= twice the largest − 1");
        assert_eq!(
            joined(&instance(
                pic.one,
                pic.one_below,
                pic.one_handful,
                &pic.one_estimate.to_string()
            )),
            "37 + 36 ÷ 1 = 73"
        );
        assert_eq!(
            friends_case(pic.k, pic.largest, pic.unseen, pic.estimate),
            "k = 5: 60 + 55 ÷ 5 = 71"
        );
        // the general lines are the friends' with k for five
        assert_eq!(RECIPE_K.replace('k', "5"), formula(pic.k));
        assert_eq!(
            EACH_K.replace("(k + 1)", "6").replace('k', "5"),
            each_line(pic.k, pic.positions)
        );
        assert_eq!(pair_label(&pic.pairs[0]), "1,2");
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
                gaps: vec![13, 12, 5, 17, 9],
                ..pic.clone()
            },
            "friends'",
        );
        refused(
            Picture {
                positions: pic.k,
                ..pic.clone()
            },
            "friends'",
        );
        refused(
            Picture {
                average_gap: pic.average_gap + 1,
                estimate: pic.estimate + 1,
                ..pic.clone()
            },
            "average of the five",
        );
        refused(
            Picture {
                gap_average: "(N-5)/5".to_owned(),
                ..pic.clone()
            },
            "each gap must average",
        );
        refused(
            Picture {
                pairs: pic.pairs[1..].to_vec(),
                ..pic.clone()
            },
            "every pair",
        );
        // the widest gap below as the guess: a different number
        let mut widest = pic.guesses.clone();
        widest[4] = "1".to_owned();
        refused(
            Picture {
                guesses: widest,
                ..pic.clone()
            },
            "average gap below",
        );
        let mut rounded = pic.guesses.clone();
        rounded[1] = "1".to_owned();
        refused(
            Picture {
                guesses: rounded,
                ..pic.clone()
            },
            "average gap below",
        );
        refused(
            Picture {
                agree: pic.agree + 1,
                ..pic.clone()
            },
            "agree",
        );
        refused(
            Picture {
                guess_average: 2,
                ..pic.clone()
            },
            "agree",
        );
        refused(
            Picture {
                one_estimate: 2 * pic.one,
                ..pic.clone()
            },
            "one box",
        );
        refused(
            Picture {
                one_gaps: 1,
                ..pic.clone()
            },
            "one box",
        );
        refused(
            Picture {
                one_recipe: "2m".to_owned(),
                ..pic.clone()
            },
            "one box",
        );
        // a box past the middle of the line: its estimate falls off it
        refused(
            Picture {
                one: 43,
                one_below: 42,
                one_estimate: 85,
                ..pic
            },
            "one box",
        );
    }

    /// The proof's steps hold for every small company (handfuls of 1 to 4
    /// from up to 10 boxes): in every handful the estimate minus the count
    /// is the guess minus the top gap, and over all handfuls the guess and
    /// the top gap both average (N − k) ÷ (k + 1), so the estimates average
    /// N. The negative control: the widest gap below as the guess averages
    /// more, so the largest plus it overestimates on average.
    #[test]
    fn the_guess_and_the_top_gap_average_the_same() {
        for k in 1..=4usize {
            for n in k as u64..=10 {
                let hands = all_handfuls(n, k);
                let count = Q::int(hands.len() as i128);
                let (mut guesses, mut tops, mut estimates, mut widest) = (Q::ZERO, Q::ZERO, Q::ZERO, Q::ZERO);
                for h in &hands {
                    let m = h[k - 1];
                    let top = Q::int((n - m) as i128);
                    let guess = guess_of(h, n);
                    assert_eq!(gap_recipe(h) - Q::int(n as i128), guess - top, "{h:?} from {n}");
                    guesses = guesses + guess;
                    tops = tops + top;
                    estimates = estimates + gap_recipe(h);
                    let l = list_of(h, n);
                    widest = widest + Q::int(*l[..k].iter().max().unwrap() as i128);
                }
                let share = Q::new((n - k as u64) as i128, k as i128 + 1);
                assert_eq!(guesses / count, share, "k {k}, n {n}");
                assert_eq!(tops / count, share, "k {k}, n {n}");
                assert_eq!(estimates / count, Q::int(n as i128), "k {k}, n {n}");
                if k >= 2 && n >= k as u64 + 2 {
                    assert!(widest / count > share, "k {k}, n {n}: the widest gap guesses high");
                }
            }
        }
    }

    /// One box: the recipe doubles it and subtracts one for every number
    /// on the line, and is right on average for every count from it; the
    /// negative control: just doubling overestimates every count.
    #[test]
    fn one_box_doubles_and_subtracts_one() {
        for m in 1..=LINE_DOTS {
            assert_eq!(gap_recipe(&[m]), Q::int(2 * m as i128 - 1), "{m}");
        }
        for n in 1..=LINE_DOTS {
            let all: Q = (1..=n).map(|m| gap_recipe(&[m])).fold(Q::ZERO, |s, e| s + e);
            assert_eq!(all / Q::int(n as i128), Q::int(n as i128), "{n}");
            let doubled = Q::int((1..=n).map(|m| 2 * m as i128).sum()) / Q::int(n as i128);
            assert!(doubled > Q::int(n as i128));
        }
    }

    /// Every phrase the scene waits for is in its line of script.tsv, once,
    /// and a line's phrases come in the order the scene waits for them.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let waits: &[(&str, &[&str])] = &[
            (
                "07_1",
                &[
                    "Call the count N",
                    "unseen boxes into six gaps",
                    "five they can see",
                    "the top gap they can't",
                ],
            ),
            ("07_2", &["The matching from the table", "builds back its own handful"]),
            (
                "07_3",
                &[
                    "Every handful is equally likely",
                    "Together they always hold",
                    "so each gap averages",
                ],
            ),
            (
                "07_4",
                &[
                    "That includes the top gap",
                    "the average of the five gaps",
                    "that's eleven",
                ],
            ),
            (
                "07_5",
                &[
                    "But eleven is",
                    "In the small company",
                    "the average of its two gaps below",
                    "Row by row",
                    "agree only three times",
                ],
            ),
            (
                "07_6",
                &[
                    "the guesses average one box",
                    "so do the top gaps",
                    "so the average of the five",
                    "exactly like the top gap",
                ],
            ),
            (
                "07_7",
                &[
                    "the count is the largest number",
                    "the estimate is the largest number",
                    "Over all handfuls",
                    "have the same average",
                    "The recipe is unbiased",
                ],
            ),
            (
                "07_9",
                &[
                    "One box leaves two gaps",
                    "the only clue",
                    "thirty-six boxes below it",
                    "the recipe says",
                ],
            ),
            (
                "07_10",
                &[
                    "one more gap than there are numbers",
                    "all with the same average",
                    "So the recipe always adds",
                    "it's right on average",
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
        // the prediction's line shows nothing of the answer
        let asked = script.text("07_8").unwrap();
        for answer in ["thirty-six", "seventy-three", "doubles"] {
            assert!(!asked.contains(answer), "{answer}");
        }
    }

    /// The panel: every line and sum in the title-safe area, the badge
    /// left of it; the inset under the panel's lowest line and over the
    /// boxes, its values apart, its averages after its last value; the
    /// upper left's lines under the badge and left of the panel.
    #[test]
    fn the_panel_and_the_inset_fit_the_frame() {
        let pic = good();
        let safe_x = FRAME_X_RADIUS - 0.25;
        let match_w = est_width(HANDFUL, PANEL_SIZE)
            + 2.0 * MATCH_GAP
            + MATCH_ARROW
            + est_width(&lists_word(pic.positions, pic.k), PANEL_SIZE);
        let mut widths = vec![
            sum_width(&all_gaps(&pic.gaps, pic.k)),
            sum_width(&guess_pieces(&pic.gaps, pic.k, pic.average_gap)),
            sum_width(&instance(
                pic.one,
                pic.one_below,
                pic.one_handful,
                &pic.one_estimate.to_string(),
            )),
            match_w,
        ];
        for text in [
            COUNT_N.to_owned(),
            each_line(pic.k, pic.positions),
            guess_too(pic.k, pic.positions),
            COUNT_IS.to_owned(),
            ESTIMATE_IS.to_owned(),
            DIFFERENCE.to_owned(),
            on_average(pic.k, pic.positions),
            formula(pic.one_handful),
            twice_line(pic.one_handful),
            GAPS_K.to_owned(),
            EACH_K.to_owned(),
            RECIPE_K.to_owned(),
            friends_case(pic.k, pic.largest, pic.unseen, pic.estimate),
        ] {
            widths.push(est_width(&text, PANEL_SIZE));
        }
        for w in widths {
            assert!(PANEL_X + w < safe_x, "{w}");
        }
        let badge_right = BADGE_AT[0] + est_width(BADGE, BADGE_SIZE) / 2.0 + BADGE_PAD[0];
        assert!(badge_right < PANEL_X - CLEAR);
        // the upper left: under the badge (its words' half height 0.15 and
        // its pad), left of the panel
        const { assert!(SMALL_AT[1] + HALF_LINE < BADGE_AT[1] - 0.15 - BADGE_PAD[1]) };
        const { assert!(UNBIASED_AT[1] + HALF_LINE < BADGE_AT[1] - 0.15 - BADGE_PAD[1]) };
        let small = small_line(pic.boxes - pic.seen as u64, pic.seen + 1, pic.top_average);
        assert!(SMALL_AT[0] + est_width(&small, PANEL_SIZE) < PANEL_X - CLEAR);
        assert!(UNBIASED_AT[0] + est_width(UNBIASED, PANEL_SIZE) < PANEL_X - CLEAR);
        // the inset: under the panel's lowest line, over the boxes, its
        // labels left of its values, its values apart
        const { assert!(INSET_Y[0] + HALF_INSET < PANEL_Y[3] - HALF_LINE) };
        const { assert!(INSET_Y[2] - HALF_INSET > STRIP_Y + BOX_RISE + SMALL * BOX_H / 2.0 + CLEAR) };
        for w in INSET_Y.windows(2) {
            assert!(w[0] - w[1] > 2.0 * HALF_INSET + 0.1);
        }
        let label_w = INSET_LABELS
            .iter()
            .map(|l| est_width(l, INSET_SIZE))
            .fold(0.0, f64::max);
        let value_w = pic
            .pairs
            .iter()
            .map(|p| est_width(&pair_label(p), INSET_SIZE))
            .chain(pic.guesses.iter().map(|g| est_width(g, INSET_SIZE)))
            .fold(0.0, f64::max);
        assert!(INSET_LABEL_X + label_w < inset_x(0) - value_w / 2.0 - CLEAR);
        assert!(INSET_STEP > value_w + 0.1);
        let last = inset_x(pic.pairs.len() - 1) + value_w / 2.0;
        assert!(INSET_AVERAGE_X > last + CLEAR);
        let average_w = est_width(&format!("average {}", pic.guess_average), INSET_SIZE);
        assert!(INSET_AVERAGE_X + average_w < safe_x);
        const { assert!(INSET_LABEL_X > -(FRAME_X_RADIUS - 0.25)) };
    }

    /// The friends' guess and the one box: the guess's arc over the hidden
    /// top gap and its count clear the largest box; the one box's estimate
    /// is on the line, its tick and number clear of the box, the countdown
    /// and the "?"; the guess's arc for one box is as long as the gap below.
    #[test]
    fn the_guesses_and_the_ticks_fit_the_line() {
        let pic = good();
        let end = LINE_DOTS as f64;
        let box_half = SMALL * BOX_W / 2.0;
        let (a, b) = (pic.largest + 1, pic.largest + pic.average_gap);
        let count_at = over_arc(a, b, end);
        let w = est_width(&pic.average_gap.to_string(), PANEL_SIZE);
        assert!(count_at[0] - w / 2.0 > x_at(pic.largest as f64, end) + box_half + CLEAR);
        assert!(count_at[1] + HALF_LINE < PANEL_Y[3] - HALF_LINE);
        // the one box's guess spans as many numbers as its gap below
        let (c, d) = (pic.one + 1, pic.one + pic.one_below);
        assert_eq!(d - c + 1, pic.one_below);
        assert!(d < LINE_DOTS && d == pic.one_estimate);
        let x = x_at(pic.one_estimate as f64, end);
        let label_w = est_width(&pic.one_estimate.to_string(), PANEL_SIZE);
        assert!(x - label_w / 2.0 > x_at(pic.one as f64, end) + box_half + CLEAR);
        assert!(x + label_w / 2.0 < FRAME_X_RADIUS - 0.25);
        // the estimate's number over the guess's arc, wherever the arc is
        const { assert!(STRIP_Y + ESTIMATE_DY - HALF_LINE > STRIP_Y + ARC_LIFT + ARC_H) };
        // card D's three lines stay under the one box's bracket and its
        // "?" (the card is 0.3 plus 0.36 a line, 0.2 over the frame's foot)
        const { assert!(-FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 3.0 < STRIP_Y - BRACKET_DY - 0.6) };
        // the prediction's line: the one box's arc covers its gap below, and
        // nothing else is on it until the answer
        let spans = gap_spans(&[pic.one]);
        assert_eq!(spans, [(1, pic.one_below)]);
    }

    /// Card D says the general result in at most three lines at every width
    /// the lower third might narrow to, and is up long enough to be read.
    #[test]
    fn card_d_fits_and_is_read() {
        for chars in (CARD_CHARS_NARROWEST..=CARD_CHARS_WIDEST).step_by(4) {
            assert!(wrap(CARD_D, chars, 4).len() <= 3, "{chars}");
        }
        assert!(read_s(CARD_D) < 15.0);
        assert!(CARD_D.contains("(largest − k) ÷ k") && CARD_D.contains("(N − k) ÷ (k + 1)"));
        // the negative control: one clause more would not fit
        let longer = format!("{CARD_D} The friends' estimate is one value, not a count.");
        assert!(wrap(&longer, CARD_CHARS_NARROWEST, 4).len() > 3);
    }
}
