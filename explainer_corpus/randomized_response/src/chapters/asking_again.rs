//! Chapter 10, asking again: one student asked three times with fresh coins
//! (10_1); every one of the 64 equally likely ways the six coins can land,
//! for each truth (10_2); the prediction, how many times as likely three
//! yeses are from someone who copied (10_3), and its answer, 27, counted
//! (10_4); each fresh answer can multiply it by three, and three noes do the
//! same the other way (10_5); the class's three-yes outcomes, 108 and 16, so
//! twenty-seven in thirty-one against one yes's three in seven (10_6, 10_7);
//! the majority right 27 times in 32 for either truth (10_8).
//!
//! Chapter 11, one flip: a hundred fresh answers about one secret that never
//! changes, seventy-five yes (11_1, 11_2), leave 3⁵⁰ and, from even odds,
//! twenty-three zeros after the point (11_3); the prediction, one flip whose
//! answer is repeated (11_4), and its answer, three yeses still three times
//! as likely (11_5), for one unchanged secret under this rule only (11_6);
//! what RAPPOR does (11_7). The memoized columns stay faint, and its card
//! waits, until 11_5: nothing answers the prediction during its hold.
//!
//! Every square is `model::answer` run on one truth and one outcome of the
//! asks' coins; the drawn squares are tallied against
//! `model::repeated_answers` and `model::memoized_answers` before any count
//! is labelled. Chapter 11's numbers are `model::posterior` and the ratio
//! three, raised to the yeses and divided by the noes (`R-F22`).

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, thousands, v};
use fmn::prelude::*;

use super::draw::{
    coin, disc, face, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, lane, pause, place, read_by, say,
    square, swap, tex_q, think, wait_until,
};
use crate::facts::{self, Average, RAPPOR_EXAMPLE};
use crate::model::{self, ASKED_AGAIN, COIN_PAIRS, Q};
use crate::palette;

pub struct AskingAgain {
    pub kit: Kit,
}

pub struct OneFlip {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const COPIED: &str = "copied";
const DIDNT: &str = "didn't copy";
const ASK: &str = "ask";
const YES: &str = "yes";
const NO: &str = "no";
const MAJORITY: &str = "majority right";
const ONE_YES: &str = "after one yes";
const FRESH: &str = "fresh coins";
const EVERY_DAY: &str = "the same secret, fresh coins every day";
const EVEN_ODDS: &str = "from even odds, the chance they didn't copy:";
const REMEMBERED: &str = "a value: randomized once, remembered";
const AGAIN: &str = "every report: randomized again";

/// How many times the student is asked, as the grids draw it.
const ASKS: usize = ASKED_AGAIN as usize;
// the grids lay three asks out as blocks, rows and columns
const _: () = assert!(ASKS == 3, "the grids draw exactly three asks");

// ------------------------------------------------------------------ layout

/// 10_1: the student, and the three asks to its right, each a place with its
/// own two coins and its name under it.
const STUDENT_X: f64 = -3.4;
const ASK_X0: f64 = -1.8;
const ASK_DX: f64 = 1.6;
const ASK_Y: f64 = 1.4;
const PLACE_W: f64 = 1.1;
const PLACE_H: f64 = 0.7;
const COIN_R: f64 = 0.16;
const COIN_DX: f64 = 0.24;
const ASK_LABEL_DY: f64 = 0.65;
/// The two grids of outcomes, copied on the left: 2 × 2 blocks (the first
/// ask's coins), each 4 × 4 squares (rows the second ask's, columns the
/// third's); the grids' names above them.
const PANEL_X: [f64; 2] = [-4.3, -1.0];
const PANEL_Y: f64 = 1.3;
const SIDE: f64 = 0.24;
const PITCH: f64 = 0.3;
const BLOCK_GAP: f64 = 0.18;
const HEAD_Y: f64 = 3.0;
/// Each grid's counts under it, two rows; the class's products under them.
const LABEL_Y: [f64; 2] = [-0.45, -0.85];
const CLASS_Y: f64 = -1.55;
/// The right column: the total, then the ratios and chances; the think time.
const RIGHT_X: f64 = 4.3;
const RIGHT_Y: [f64; 4] = [2.6, 1.5, 0.45, -0.65];
const THINK_Y: f64 = 1.2;
/// The memoized answer (11_4): per truth, one column per outcome of the one
/// flip, its faces above, three squares (the asks) of the one answer.
const MEMO_DX: f64 = 0.55;
const MEMO_Y0: f64 = 1.9;
const MEMO_DY: f64 = 0.4;
const MEMO_SIDE: f64 = 0.3;
const MEMO_FACE_Y: f64 = 2.45;
const MEMO_LABEL_Y: f64 = 0.5;
/// Chapter 11's hundred days: one student over a 10 × 10 strip of squares
/// at the left, its caption over the student, the tally under the strip;
/// the mathematics in a column at the right.
const STRIP_X: f64 = -4.65;
const STRIP_COLUMNS: usize = 10;
const STRIP_Y0: f64 = 2.15;
const DAY_DX: f64 = 0.3;
const DAY_SIDE: f64 = 0.22;
const STUDENT_Y: f64 = 2.75;
const CAPTION_Y: f64 = 3.25;
const TALLY_Y: f64 = -1.05;
const MATH_X: f64 = 2.6;
const MATH_Y: [f64; 7] = [2.85, 2.05, 1.55, 0.9, 0.3, -0.35, -0.85];
/// Chapter 11's closing diagram: two boxes, an arrow between them.
const BOX_X: f64 = 3.0;
const BOX_Y: f64 = 0.9;
const BOX_W: f64 = 4.6;
const BOX_H: f64 = 1.0;

/// Where day `d` of the strip sits: rows of ten from the top left.
fn day_at(d: usize) -> Vec3 {
    let (row, col) = (d / STRIP_COLUMNS, d % STRIP_COLUMNS);
    v(
        STRIP_X + DAY_DX * (col as f64 - (STRIP_COLUMNS as f64 - 1.0) / 2.0),
        STRIP_Y0 - DAY_DX * row as f64,
    )
}

/// 11_2's exact version: everything the hundred answers' arithmetic assumes.
fn odds_card() -> String {
    "assumes fresh coins each day, one secret that never changes, the rule followed, and even odds at the start"
        .to_owned()
}

/// 11_7's exact version: what RAPPOR does, and that it is not this rule.
fn rappor_card() -> String {
    "RAPPOR: a randomized answer memoized per value, then randomized again on each report; many bits per report, \
     not this one-bit rule"
        .to_owned()
}

/// The coin pairs of outcome `o`, one per ask, the first ask's first: the
/// base-4 digits of `o`, most significant first.
fn pairs_of(o: usize) -> [usize; ASKS] {
    let per = COIN_PAIRS.len();
    let mut out = [0; ASKS];
    for (j, slot) in out.iter_mut().enumerate() {
        *slot = o / per.pow((ASKS - 1 - j) as u32) % per;
    }
    out
}

/// The answers someone whose truth is `copied` gives on outcome `o`: bit j
/// is ask j's answer, yes.
fn answers_of(copied: bool, o: usize) -> usize {
    pairs_of(o).iter().enumerate().fold(0, |said, (j, &pair)| {
        let (first, second) = COIN_PAIRS[pair];
        if model::answer(copied, first, second) {
            said | (1 << j)
        } else {
            said
        }
    })
}

/// Where outcome `o` sits in grid `p`: its block by the first ask's coins,
/// its row by the second's, its column by the third's.
fn cell_at(p: usize, o: usize) -> Vec3 {
    let [first, second, third] = pairs_of(o);
    let side = COIN_PAIRS.len();
    let (block_col, block_row) = (first % 2, first / 2);
    let col = (block_col * side + third) as f64;
    let row = (block_row * side + second) as f64;
    v(
        PANEL_X[p] + (col - 3.5) * PITCH + (block_col as f64 - 0.5) * BLOCK_GAP,
        PANEL_Y - (row - 3.5) * PITCH - (block_row as f64 - 0.5) * BLOCK_GAP,
    )
}

/// Where the memoized answer's square for flip outcome `k` and ask `j` sits
/// in grid `p`.
fn memo_at(p: usize, k: usize, j: usize) -> Vec3 {
    v(PANEL_X[p] + (k as f64 - 1.5) * MEMO_DX, MEMO_Y0 - MEMO_DY * j as f64)
}

/// A count under a grid: "3 yes: 27 of 64".
fn count_label(what: &str, count: u64, of: usize) -> String {
    format!("{what}: {count} of {of}")
}

/// 10_2's exact version: what a square is, and how the grid is laid out.
fn grid_card(total: usize) -> String {
    format!(
        "each square is one way all {} coins can land, {total} equally likely. Block: the first answer's coins; \
         row: the second's; column: the third's",
        ASKS * 2
    )
}

/// 10_5's exact version: every pattern's ratio by its yeses, and the most
/// after each number of fresh answers (`worst`, as the facts list it).
fn ratios_card(ratios: &[Q], worst: &str) -> String {
    let listed: Vec<String> = ratios.iter().map(Q::to_string).collect();
    format!(
        "0 to {ASKS} yeses are {} times as likely from someone who copied; the most after 1 to {} fresh answers: {}",
        listed.join(", "),
        worst.split(',').count(),
        worst.replace(',', ", ")
    )
}

/// 10_7's exact version: the chance is the class's (`copiers` of `n`); from
/// an even start, and from a class where nobody copied, the same yeses leave
/// `at_half` and `none`.
fn prior_card(after: Q, copiers: usize, n: usize, at_half: Q, none: Q) -> String {
    format!(
        "{after} is the chance in this class, where {copiers} of {n} copied; from an even 1/2 the same {ASKS} \
         yeses would leave {at_half}, and if nobody copied, {none}"
    )
}

/// 11_5's exact version (the old 04_9's, shown only once the prediction is
/// answered): what a memoized answer is, and how far the chapter claims it.
fn memo_card() -> String {
    "memoized: one flip for this secret, its answer re-sent each ask, so repeats add no evidence; only for one \
     unchanged secret, under this rule"
        .to_owned()
}

/// One outcome as drawn: whose truth, its answers (bit j: ask j said yes),
/// its square now.
struct Outcome {
    copied: bool,
    said: usize,
    at: Vec3,
    mob: Mob,
}

impl Outcome {
    fn yeses(&self) -> usize {
        self.said.count_ones() as usize
    }
}

/// Redraw the outcomes `which` picks in the colour `color` gives them
/// (fresh squares over the old), together.
fn recolor(
    stage: &mut Stage<'_>,
    outcomes: &mut [Outcome],
    which: impl Fn(&Outcome) -> bool,
    color: impl Fn(&Outcome) -> Srgb,
    rt: f64,
) -> fmn::Result<()> {
    let (mut old, mut new) = (Vec::new(), Vec::new());
    for o in outcomes.iter_mut() {
        if !which(&*o) {
            continue;
        }
        let fresh = square(stage, color(&*o), o.at, SIDE)?;
        old.push(o.mob);
        new.push(fresh);
        o.mob = fresh;
    }
    swap(stage, &old, &new, rt)
}

/// The squares of the outcomes `which` picks.
fn squares(outcomes: &[Outcome], which: impl Fn(&Outcome) -> bool) -> Vec<Mob> {
    outcomes.iter().filter(|&o| which(o)).map(|o| o.mob).collect()
}

/// How many drawn outcomes `which` picks.
fn count(outcomes: &[Outcome], which: impl Fn(&Outcome) -> bool) -> u64 {
    outcomes.iter().filter(|&o| which(o)).count() as u64
}

/// The drawn outcomes counted per answer sequence, as `[didn't, copied]`.
fn tally(outcomes: &[Outcome]) -> Vec<[u64; 2]> {
    let mut c = vec![[0u64; 2]; 1 << ASKS];
    for o in outcomes {
        c[o.said][usize::from(o.copied)] += 1;
    }
    c
}

/// A grid's name: `who` before it when counting the class.
fn grid_name(kit: &Kit, stage: &mut Stage<'_>, p: usize, who: Option<usize>) -> fmn::Result<Mob> {
    let (word, color) = if p == 0 {
        (COPIED, palette::copied())
    } else {
        (DIDNT, palette::student())
    };
    let text = who.map_or_else(|| word.to_owned(), |k| format!("{k} {word}"));
    kit.text(stage, &text, 24.0, color, v(PANEL_X[p], HEAD_Y))
}

impl SceneConstruct for AskingAgain {
    fn name(&self) -> &str {
        "10_asking_again"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let a: Average = facts::average().map_err(fail)?;
        let (n, copiers) = (a.class.size(), a.class.copiers());
        let others = n - copiers;
        let prior = Q::new(copiers as i128, n as i128);
        let total = COIN_PAIRS.len().pow(ASKED_AGAIN);
        let f = facts::facts().map_err(fail)?;
        let fact = |key: &str| -> fmn::Result<String> {
            f.get(key)
                .map(str::to_owned)
                .ok_or_else(|| fail(format!("no fact {key}")))
        };

        // 10_1: one student, asked three times, fresh coins each time
        let line = say(&kit, stage, "10_1")?;
        let student = disc(stage, palette::student(), v(STUDENT_X, ASK_Y));
        fade_in_all(stage, &[student], 0.3)?;
        let mut strip = vec![student];
        line.wait_for(stage, "three times")?;
        let mut asked = Vec::with_capacity(2 * ASKS);
        for k in 0..ASKS {
            let x = ASK_X0 + ASK_DX * k as f64;
            asked.push(place(stage, v(x, ASK_Y), PLACE_W, PLACE_H)?);
            asked.push(kit.text(
                stage,
                &format!("{ASK} {}", k + 1),
                20.0,
                Palette::dim(),
                v(x, ASK_Y - ASK_LABEL_DY),
            )?);
        }
        fade_in_lagged(stage, &asked, 1.0, 0.3)?;
        strip.extend(asked);
        line.wait_for(stage, "fresh coins")?;
        let mut coins = Vec::with_capacity(2 * ASKS);
        for k in 0..ASKS {
            let x = ASK_X0 + ASK_DX * k as f64;
            coins.push(coin(stage, palette::student(), v(x - COIN_DX, ASK_Y), COIN_R));
            coins.push(coin(stage, palette::forced(), v(x + COIN_DX, ASK_Y), COIN_R));
        }
        fade_in_lagged(stage, &coins, 0.9, 0.2)?;
        strip.extend(coins);

        // 10_2: every way the six coins can land, for each truth
        let line = say(&kit, stage, "10_2")?;
        let mut names = [grid_name(&kit, stage, 0, None)?, grid_name(&kit, stage, 1, None)?];
        let card = precisely(&kit, stage, &grid_card(total))?;
        let mut shown = names.to_vec();
        shown.push(card);
        swap(stage, &strip, &shown, 0.4)?;
        let mut outcomes = Vec::with_capacity(2 * total);
        for (p, copied) in [(0, true), (1, false)] {
            for o in 0..total {
                let at = cell_at(p, o);
                outcomes.push(Outcome {
                    copied,
                    said: answers_of(copied, o),
                    at,
                    mob: square(stage, palette::ghost(), at, SIDE)?,
                });
            }
        }
        let counted = model::repeated_answers(ASKED_AGAIN).map_err(fail)?;
        if tally(&outcomes) != counted {
            return Err(fail(format!(
                "the drawn outcomes count {:?}, the model {counted:?}",
                tally(&outcomes)
            )));
        }
        fade_in_lagged(stage, &squares(&outcomes, |_| true), 1.6, 0.01)?;
        line.wait_for(stage, "sixty-four")?;
        let per = COIN_PAIRS.len().to_string();
        let total_tex = kit.tex(
            stage,
            &format!(r"{} = {total}", [per.as_str(); ASKS].join(r" \times ")),
            34.0,
            Palette::ink(),
            &[],
            v(RIGHT_X, RIGHT_Y[0]),
        )?;
        fade_in_all(stage, &[total_tex], 0.4)?;

        // 10_3: the prediction; no square is coloured, no yes counted
        let _line = say(&kit, stage, "10_3")?;
        let q = think(&kit, stage, v(RIGHT_X, THINK_Y))?;

        // 10_4: twenty-seven outcomes against one
        let line = say(&kit, stage, "10_4")?;
        fade_out_all(stage, &[q], 0.2)?;
        let all_yes = |o: &Outcome| o.yeses() == ASKS;
        recolor(
            stage,
            &mut outcomes,
            |o| o.copied && all_yes(o),
            |_| palette::yes(),
            0.4,
        )?;
        let yes_copied = count(&outcomes, |o| o.copied && all_yes(o));
        let yes_didnt = count(&outcomes, |o| !o.copied && all_yes(o));
        let three_yes = format!("{ASKS} {YES}");
        let yes_labels = [
            kit.text(
                stage,
                &count_label(&three_yes, yes_copied, total),
                20.0,
                palette::yes(),
                v(PANEL_X[0], LABEL_Y[0]),
            )?,
            kit.text(
                stage,
                &count_label(&three_yes, yes_didnt, total),
                20.0,
                palette::yes(),
                v(PANEL_X[1], LABEL_Y[0]),
            )?,
        ];
        fade_in_all(stage, &yes_labels[..1], 0.3)?;
        line.wait_for(stage, "anyone else in one")?;
        recolor(
            stage,
            &mut outcomes,
            |o| !o.copied && all_yes(o),
            |_| palette::yes(),
            0.4,
        )?;
        let ratio = Q::int(i128::from(yes_copied)) / Q::int(i128::from(yes_didnt));
        if Some(ratio) != model::pattern_ratio(counted[(1 << ASKS) - 1]) {
            return Err(fail(format!("{ratio} drawn for three yeses, unlike the model")));
        }
        let ratio_tex = kit.tex(
            stage,
            &format!(r"\frac{{{yes_copied}}}{{{yes_didnt}}} = {ratio}"),
            36.0,
            Palette::ink(),
            &[],
            v(RIGHT_X, RIGHT_Y[1]),
        )?;
        fade_in_all(stage, &[yes_labels[1], ratio_tex], 0.4)?;

        // 10_5: each fresh answer can multiply it by three; three noes back
        let line = say(&kit, stage, "10_5")?;
        let each = model::likelihood_ratio(true);
        if each.pow(ASKED_AGAIN) != ratio {
            return Err(fail(format!("{ratio} is not {each} to the power {ASKS}")));
        }
        let each_text = each.to_string();
        let power = kit.tex(
            stage,
            &format!(r"{ratio} = {}", [each_text.as_str(); ASKS].join(r" \times ")),
            34.0,
            Palette::ink(),
            &[],
            v(RIGHT_X, RIGHT_Y[2]),
        )?;
        let ratios: Vec<Q> = model::by_yeses(&counted)
            .into_iter()
            .map(|c| model::pattern_ratio(c).ok_or_else(|| fail("a pattern no one who didn't copy gives".into())))
            .collect::<fmn::Result<_>>()?;
        let card2 = precisely(&kit, stage, &ratios_card(&ratios, &fact("R-F9.worst_ratio_by_asks")?))?;
        fade_out_all(stage, &[card], 0.2)?;
        fade_in_all(stage, &[power, card2], 0.2)?;
        line.wait_for(stage, "Three noes")?;
        let all_no = |o: &Outcome| o.yeses() == 0;
        recolor(stage, &mut outcomes, all_no, |_| palette::no(), 0.4)?;
        let three_no = format!("{ASKS} {NO}");
        let no_labels = [
            kit.text(
                stage,
                &count_label(&three_no, count(&outcomes, |o| o.copied && all_no(o)), total),
                20.0,
                palette::no(),
                v(PANEL_X[0], LABEL_Y[1]),
            )?,
            kit.text(
                stage,
                &count_label(&three_no, count(&outcomes, |o| !o.copied && all_no(o)), total),
                20.0,
                palette::no(),
                v(PANEL_X[1], LABEL_Y[1]),
            )?,
        ];
        fade_in_all(stage, &no_labels, 0.3)?;

        // 10_6: the class's three-yes outcomes, 108 and 16
        let line = say(&kit, stage, "10_6")?;
        let named = [
            grid_name(&kit, stage, 0, Some(copiers))?,
            grid_name(&kit, stage, 1, Some(others))?,
        ];
        let mut leaving = vec![total_tex, ratio_tex, power];
        leaving.extend(no_labels);
        leaving.extend(names);
        fade_out_all(stage, &leaving, 0.2)?;
        fade_in_all(stage, &named, 0.2)?;
        names = named;
        recolor(stage, &mut outcomes, all_no, |_| palette::ghost(), 0.3)?;
        let class_yes = [others as u64 * yes_didnt, copiers as u64 * yes_copied];
        line.wait_for(stage, "a hundred and eight")?;
        flash_all(stage, &squares(&outcomes, |o| o.copied && all_yes(o)), 0.5)?;
        let products = [
            kit.tex(
                stage,
                &format!(r"{copiers} \times {yes_copied} = {}", class_yes[1]),
                34.0,
                palette::yes(),
                &[],
                v(PANEL_X[0], CLASS_Y),
            )?,
            kit.tex(
                stage,
                &format!(r"{others} \times {yes_didnt} = {}", class_yes[0]),
                34.0,
                palette::yes(),
                &[],
                v(PANEL_X[1], CLASS_Y),
            )?,
        ];
        fade_in_all(stage, &products[..1], 0.3)?;
        line.wait_for(stage, "the sixteen others")?;
        flash_all(stage, &squares(&outcomes, |o| !o.copied && all_yes(o)), 0.5)?;
        fade_in_all(stage, &products[1..], 0.3)?;

        // 10_7: twenty-seven in thirty-one; after one yes, three in seven
        let line = say(&kit, stage, "10_7")?;
        let after = Q::int(i128::from(class_yes[1])) / Q::int(i128::from(class_yes[0] + class_yes[1]));
        if after != model::posterior(prior, ASKED_AGAIN, ASKED_AGAIN) {
            return Err(fail(format!("{after} counted, unlike the closed form")));
        }
        line.wait_for(stage, "twenty-seven in thirty-one")?;
        let chance = kit.tex(
            stage,
            &format!(
                r"\frac{{{}}}{{{} + {}}} = {}",
                class_yes[1],
                class_yes[1],
                class_yes[0],
                tex_q(after, false)
            ),
            36.0,
            Palette::ink(),
            &[],
            v(RIGHT_X, RIGHT_Y[1]),
        )?;
        let card3 = precisely(
            &kit,
            stage,
            &prior_card(
                after,
                copiers,
                n,
                model::posterior(Q::new(1, 2), ASKED_AGAIN, ASKED_AGAIN),
                model::posterior(Q::ZERO, ASKED_AGAIN, ASKED_AGAIN),
            ),
        )?;
        fade_out_all(stage, &[card2], 0.2)?;
        fade_in_all(stage, &[chance, card3], 0.2)?;
        line.wait_for(stage, "After one")?;
        let one_yes = kit.text(
            stage,
            &format!("{ONE_YES}: {}", model::posterior(prior, 1, 1)),
            20.0,
            Palette::dim(),
            v(RIGHT_X, RIGHT_Y[2]),
        )?;
        fade_in_all(stage, &[one_yes], 0.3)?;

        // 10_8: the majority, right twenty-seven times in thirty-two either way
        let line = say(&kit, stage, "10_8")?;
        let plain = [grid_name(&kit, stage, 0, None)?, grid_name(&kit, stage, 1, None)?];
        let mut leaving = products.to_vec();
        leaving.extend(yes_labels);
        leaving.extend(names);
        fade_out_all(stage, &leaving, 0.2)?;
        fade_in_all(stage, &plain, 0.2)?;
        let majority = |o: &Outcome| {
            if o.copied {
                2 * o.yeses() > ASKS
            } else {
                2 * o.yeses() < ASKS
            }
        };
        recolor(
            stage,
            &mut outcomes,
            |_| true,
            |o| {
                if majority(o) {
                    palette::answer(o.copied)
                } else {
                    palette::ghost()
                }
            },
            0.5,
        )?;
        let right = [
            count(&outcomes, |o| o.copied && majority(o)),
            count(&outcomes, |o| !o.copied && majority(o)),
        ];
        let model_right = model::majority_right(ASKED_AGAIN).map_err(fail)?;
        if right
            .iter()
            .any(|&r| Q::new(i128::from(r), total as i128) != model_right)
        {
            return Err(fail(format!(
                "a majority right in {right:?} of {total}, not {model_right}"
            )));
        }
        let majority_labels = [
            kit.text(
                stage,
                &count_label(MAJORITY, right[0], total),
                20.0,
                palette::yes(),
                v(PANEL_X[0], LABEL_Y[0]),
            )?,
            kit.text(
                stage,
                &count_label(MAJORITY, right[1], total),
                20.0,
                palette::no(),
                v(PANEL_X[1], LABEL_Y[0]),
            )?,
        ];
        fade_in_all(stage, &majority_labels, 0.3)?;
        line.wait_for(stage, "twenty-seven times")?;
        let majority_tex = kit.tex(
            stage,
            &format!(r"\frac{{{}}}{{{total}}} = {}", right[0], tex_q(model_right, false)),
            36.0,
            Palette::ink(),
            &[],
            v(RIGHT_X, RIGHT_Y[3]),
        )?;
        fade_in_all(stage, &[majority_tex], 0.4)?;
        line.wait_for(stage, "whichever the truth")?;
        flash_all(stage, &majority_labels, 0.5)?;

        line.wait_end(stage)?;
        pause(stage, 1.5)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

impl SceneConstruct for OneFlip {
    fn name(&self) -> &str {
        "11_one_flip"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (many_yes, many_asks) = RAPPOR_EXAMPLE;
        let many_no = many_asks - many_yes;
        let three = model::likelihood_ratio(true);
        if model::likelihood_ratio(false) != three {
            return Err(fail("a yes and a no carry different ratios".into()));
        }
        // each yes multiplies the evidence by three, each no divides it
        let ratio = three.pow(many_yes) / three.pow(many_no);
        let no_chance = Q::ONE - model::posterior(Q::new(1, 2), many_yes, many_asks);
        if Q::ONE / (ratio + Q::ONE) != no_chance {
            return Err(fail(format!("{no_chance} is not 1/({ratio} + 1)")));
        }
        let zeros = model::zeros_after_point(no_chance).map_err(fail)?;

        // 11_1: one student, asked every day with fresh coins, one secret
        let _line = say(&kit, stage, "11_1")?;
        let student = disc(stage, palette::student(), v(STRIP_X, STUDENT_Y));
        let caption = kit.text(stage, EVERY_DAY, 20.0, Palette::dim(), v(STRIP_X, CAPTION_Y))?;
        fade_in_all(stage, &[student, caption], 0.4)?;
        let days = many_asks as usize;
        let mut strip = Vec::with_capacity(days);
        for d in 0..days {
            strip.push(square(stage, palette::ghost(), day_at(d), DAY_SIDE)?);
        }
        fade_in_lagged(stage, &strip, 1.5, 0.01)?;

        // 11_2: seventy-five yes, twenty-five no; fifty threes multiplied
        let line = say(&kit, stage, "11_2")?;
        let mut answered = Vec::with_capacity(days);
        for d in 0..days {
            answered.push(square(
                stage,
                palette::answer(d < many_yes as usize),
                day_at(d),
                DAY_SIDE,
            )?);
        }
        let tally = kit.text(
            stage,
            &format!("{many_yes} yes, {many_no} no"),
            20.0,
            Palette::ink(),
            v(STRIP_X, TALLY_Y),
        )?;
        let mut coming = answered.clone();
        coming.push(tally);
        swap(stage, &strip, &coming, 0.5)?;
        line.wait_for(stage, "Each yes")?;
        let digits = ratio.num().to_string().len();
        let evidence = [
            kit.tex(
                stage,
                &format!(
                    r"\frac{{3^{{{many_yes}}}}}{{3^{{{many_no}}}}} = 3^{{{}}}",
                    many_yes - many_no
                ),
                34.0,
                Palette::ink(),
                &[],
                v(MATH_X, MATH_Y[0]),
            )?,
            kit.tex(
                stage,
                &format!(
                    r"3^{{{}}} = {}",
                    many_yes - many_no,
                    thousands(ratio.num().unsigned_abs()).replace(',', "{,}")
                ),
                24.0,
                Palette::ink(),
                &[],
                v(MATH_X, MATH_Y[1]),
            )?,
            kit.text(
                stage,
                &format!("{digits} digits"),
                20.0,
                Palette::dim(),
                v(MATH_X, MATH_Y[2]),
            )?,
        ];
        let odds_text = odds_card();
        let odds = precisely(&kit, stage, &odds_text)?;
        fade_in_lagged(stage, &evidence, 1.2, 0.5)?;
        fade_in_all(stage, &[odds], 0.4)?;
        let odds_read = read_by(stage, &odds_text);

        // 11_3: from even odds, twenty-three zeros after the decimal point
        let line = say(&kit, stage, "11_3")?;
        let sig = format!("{:.2e}", no_chance.to_f64());
        let mantissa: String = sig.split('e').next().unwrap_or("").replace('.', "");
        let decimal = format!("0.{}{mantissa}", "0".repeat(zeros as usize));
        let chance = [
            kit.text(stage, EVEN_ODDS, 20.0, Palette::dim(), v(MATH_X, MATH_Y[3]))?,
            kit.tex(
                stage,
                &format!(
                    r"\frac{{1}}{{3^{{{}}} + 1}} \approx {} \times 10^{{{}}}",
                    many_yes - many_no,
                    &sig[..4],
                    sig.split('e').nth(1).unwrap_or("")
                ),
                30.0,
                Palette::ink(),
                &[],
                v(MATH_X, MATH_Y[4]),
            )?,
            kit.text(stage, &decimal, 24.0, Palette::ink(), v(MATH_X, MATH_Y[5]))?,
            kit.text(
                stage,
                &format!("{zeros} zeros after the point"),
                20.0,
                palette::copied(),
                v(MATH_X, MATH_Y[6]),
            )?,
        ];
        line.wait_for(stage, "the chance they didn't")?;
        fade_in_lagged(stage, &chance, 1.2, 0.4)?;
        line.wait_end(stage)?;
        // the decimal stands a moment (a named hold)
        pause(stage, 1.5)?;

        // 11_4: one flip repeated: the prediction; the columns faint, no card
        let line = say(&kit, stage, "11_4")?;
        line.wait_for(stage, "flips only once")?;
        wait_until(stage, odds_read)?;
        let mut leaving = vec![student, caption, tally, odds];
        leaving.extend(answered);
        leaving.extend(evidence);
        leaving.extend(chance);
        fade_out_all(stage, &leaving, 0.3)?;
        let names = [grid_name(&kit, stage, 0, None)?, grid_name(&kit, stage, 1, None)?];
        let mut faces = Vec::with_capacity(2 * COIN_PAIRS.len());
        let mut columns = Vec::new();
        let mut memo_said = vec![[0u64; 2]; 1 << ASKS];
        for (p, copied) in [(0, true), (1, false)] {
            for (k, &(first, second)) in COIN_PAIRS.iter().enumerate() {
                let said = model::answer(copied, first, second);
                faces.push(kit.text(
                    stage,
                    &format!("{} {}", face(first), face(second)),
                    20.0,
                    Palette::dim(),
                    v(memo_at(p, k, 0)[0], MEMO_FACE_Y),
                )?);
                for j in 0..ASKS {
                    let at = memo_at(p, k, j);
                    columns.push((square(stage, palette::ghost(), at, MEMO_SIDE)?, at, said));
                }
                memo_said[if said { (1 << ASKS) - 1 } else { 0 }][usize::from(copied)] += 1;
            }
        }
        if memo_said != model::memoized_answers(ASKED_AGAIN).map_err(fail)? {
            return Err(fail(format!("the drawn flips count {memo_said:?}, unlike the model")));
        }
        let mut faint = names.to_vec();
        faint.extend(faces.iter().copied());
        faint.extend(columns.iter().map(|&(blank, _, _)| blank));
        fade_in_all(stage, &faint, 0.3)?;
        let q = think(&kit, stage, v(RIGHT_X, THINK_Y))?;

        // 11_5: three times as likely, the same as one; the columns and card now
        let line = say(&kit, stage, "11_5")?;
        let [memo_didnt, memo_copied] = memo_said[(1 << ASKS) - 1];
        let memo_ratio = Q::int(i128::from(memo_copied)) / Q::int(i128::from(memo_didnt));
        if memo_ratio != three {
            return Err(fail(format!(
                "a memoized three yeses give {memo_ratio}, unlike one yes"
            )));
        }
        let mut old = vec![q];
        let mut coloured = Vec::with_capacity(columns.len());
        for &(blank, at, said) in &columns {
            old.push(blank);
            coloured.push(square(stage, palette::answer(said), at, MEMO_SIDE)?);
        }
        let three_yes = format!("{ASKS} {YES}");
        let flips = COIN_PAIRS.len();
        let memo_labels = [
            kit.text(
                stage,
                &count_label(&three_yes, memo_copied, flips),
                20.0,
                palette::yes(),
                v(PANEL_X[0], MEMO_LABEL_Y),
            )?,
            kit.text(
                stage,
                &count_label(&three_yes, memo_didnt, flips),
                20.0,
                palette::yes(),
                v(PANEL_X[1], MEMO_LABEL_Y),
            )?,
        ];
        let memo_tex = kit.tex(
            stage,
            &format!(r"\frac{{{memo_copied}}}{{{memo_didnt}}} = {memo_ratio}"),
            36.0,
            Palette::ink(),
            &[],
            v(RIGHT_X, RIGHT_Y[1]),
        )?;
        let memo_text = memo_card();
        let card = precisely(&kit, stage, &memo_text)?;
        let mut new = coloured.clone();
        new.extend(memo_labels);
        new.extend([memo_tex, card]);
        swap(stage, &old, &new, 0.4)?;
        let memo_read = read_by(stage, &memo_text);
        line.wait_for(stage, "the same as one")?;
        flash_all(stage, &[memo_tex], 0.5)?;
        let fresh_ratio = three.pow(ASKED_AGAIN);
        let fresh = kit.text(
            stage,
            &format!("{FRESH}: {fresh_ratio}"),
            20.0,
            Palette::dim(),
            v(RIGHT_X, RIGHT_Y[2]),
        )?;
        fade_in_all(stage, &[fresh], 0.3)?;

        // 11_6: for one unchanged secret, under this rule
        let line = say(&kit, stage, "11_6")?;
        line.wait_for(stage, "the secret")?;
        flash_all(stage, &names, 0.5)?;

        // 11_7: a 2014 design builds on this: remembered, then randomized again
        let line = say(&kit, stage, "11_7")?;
        let mut leaving = coloured;
        leaving.extend(memo_labels);
        leaving.extend([memo_tex, fresh]);
        leaving.extend(names);
        leaving.extend(faces);
        fade_out_all(stage, &leaving, 0.3)?;
        line.wait_for(stage, "builds on this")?;
        let diagram = [
            place(stage, v(-BOX_X, BOX_Y), BOX_W, BOX_H)?,
            kit.text(stage, REMEMBERED, 20.0, Palette::ink(), v(-BOX_X, BOX_Y))?,
            lane(
                stage,
                vec![
                    v(-BOX_X + BOX_W / 2.0 + 0.1, BOX_Y),
                    v(BOX_X - BOX_W / 2.0 - 0.1, BOX_Y),
                ],
                Palette::dim(),
            ),
            place(stage, v(BOX_X, BOX_Y), BOX_W, BOX_H)?,
            kit.text(stage, AGAIN, 20.0, Palette::ink(), v(BOX_X, BOX_Y))?,
        ];
        fade_in_lagged(stage, &diagram, 1.0, 0.3)?;
        let rappor_text = rappor_card();
        let rappor = precisely(&kit, stage, &rappor_text)?;
        wait_until(stage, memo_read)?;
        swap(stage, &[card], &[rappor], 0.4)?;
        let rappor_read = read_by(stage, &rappor_text);
        line.wait_end(stage)?;
        pause(stage, 1.0)?;
        wait_until(stage, rappor_read)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;

    /// Half a grid's width (and height), its outer squares' edges included.
    const PANEL_HALF: f64 = 3.5 * PITCH + BLOCK_GAP / 2.0 + SIDE / 2.0;

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("10_1", "three times"),
            ("10_1", "fresh coins"),
            ("10_2", "sixty-four"),
            ("10_4", "anyone else in one"),
            ("10_5", "Three noes"),
            ("10_6", "a hundred and eight"),
            ("10_6", "the sixteen others"),
            ("10_7", "twenty-seven in thirty-one"),
            ("10_7", "After one"),
            ("10_8", "twenty-seven times"),
            ("10_8", "whichever the truth"),
            ("11_2", "Each yes"),
            ("11_3", "the chance they didn't"),
            ("11_4", "flips only once"),
            ("11_5", "the same as one"),
            ("11_6", "the secret"),
            ("11_7", "builds on this"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The grids are every outcome once, each where only it sits, and their
    /// answers are the model's: 27 of 64 all-yes for someone who copied, 1
    /// for anyone else, three 3 × 3 blocks on the left.
    #[test]
    fn the_grids_are_the_models_outcomes() {
        let total = COIN_PAIRS.len().pow(ASKED_AGAIN);
        assert_eq!(total, 64);
        assert_eq!(pairs_of(0), [0, 0, 0]);
        assert_eq!(pairs_of(total - 1), [3, 3, 3]);
        assert_eq!(pairs_of(0b10_01_11), [2, 1, 3]);
        let counted = model::repeated_answers(ASKED_AGAIN).unwrap();
        for (p, copied) in [(0, true), (1, false)] {
            let mut c = [[0u64; 2]; 1 << ASKS];
            for o in 0..total {
                c[answers_of(copied, o)][usize::from(copied)] += 1;
            }
            for (seq, row) in c.iter().enumerate() {
                assert_eq!(row[usize::from(copied)], counted[seq][usize::from(copied)], "{p} {seq}");
            }
            // every square its own place, inside its grid
            let at: Vec<Vec3> = (0..total).map(|o| cell_at(p, o)).collect();
            for (i, a) in at.iter().enumerate() {
                assert!((a[0] - PANEL_X[p]).abs() + SIDE / 2.0 <= PANEL_HALF + 1e-9);
                assert!((a[1] - PANEL_Y).abs() + SIDE / 2.0 <= PANEL_HALF + 1e-9);
                for b in &at[i + 1..] {
                    assert!((a[0] - b[0]).abs() > SIDE || (a[1] - b[1]).abs() > SIDE, "{a:?} {b:?}");
                }
            }
        }
        // the copier's three yeses: the first three outcomes of every ask
        let all_yes: Vec<usize> = (0..total).filter(|&o| answers_of(true, o) == (1 << ASKS) - 1).collect();
        assert_eq!(all_yes.len(), 27);
        assert!(all_yes.iter().all(|&o| pairs_of(o).iter().all(|&k| k < 3)));
        // anyone else's: tails then heads, every time
        let lone: Vec<usize> = (0..total)
            .filter(|&o| answers_of(false, o) == (1 << ASKS) - 1)
            .collect();
        assert_eq!(lone, [0b10_10_10]);
    }

    /// The words on screen say what the facts say.
    #[test]
    fn the_cards_are_built_from_the_facts() {
        let f = facts::facts().unwrap();
        assert_eq!(f.get("R-F9.outcomes"), Some("64"));
        assert_eq!(
            grid_card(64),
            "each square is one way all 6 coins can land, 64 equally likely. Block: the first answer's coins; row: \
             the second's; column: the third's"
        );
        let counted = model::repeated_answers(ASKED_AGAIN).unwrap();
        let ratios: Vec<Q> = model::by_yeses(&counted)
            .into_iter()
            .map(|c| model::pattern_ratio(c).unwrap())
            .collect();
        assert_eq!(f.get("R-F9.ratios"), Some("1/27,1/3,3,27"));
        let worst = f.get("R-F9.worst_ratio_by_asks").unwrap();
        assert_eq!(worst, "3,9,27,81,243");
        assert_eq!(
            ratios_card(&ratios, worst),
            "0 to 3 yeses are 1/27, 1/3, 3, 27 times as likely from someone who copied; the most after 1 to 5 fresh \
             answers: 3, 9, 27, 81, 243"
        );
        let after = model::posterior(Q::new(1, 5), 3, 3);
        assert_eq!(f.get("R-F9.posterior.all_yes"), Some(after.to_string().as_str()));
        let card = prior_card(
            after,
            4,
            20,
            model::posterior(Q::new(1, 2), 3, 3),
            model::posterior(Q::ZERO, 3, 3),
        );
        assert_eq!(
            card,
            "27/31 is the chance in this class, where 4 of 20 copied; from an even 1/2 the same 3 yeses would leave \
             27/28, and if nobody copied, 0"
        );
        assert_eq!(f.get("R-F15.posterior.all_yes_at_half"), Some("27/28"));
        assert_eq!(f.get("R-F15.posterior.all_yes_none_copied"), Some("0"));
        assert_eq!(f.get("R-F15.all_yes.copied"), Some("108"));
        assert_eq!(f.get("R-F15.all_yes.didnt"), Some("16"));
        assert_eq!(f.get("R-F9.majority.outcomes"), Some("54"));
        assert_eq!(f.get("R-F9.majority_right.didnt"), Some("27/32"));
        assert_eq!(f.get("R-F10.memoized.ratio"), Some("3"));
        assert_eq!(f.get("R-F10.memoized.posterior"), Some("3/7"));
        assert!(memo_card().starts_with("memoized: one flip for this secret"));
        assert!(memo_card().ends_with("only for one unchanged secret, under this rule"));
        assert_eq!(count_label("3 yes", 27, 64), "3 yes: 27 of 64");
    }

    /// The cards stay two lines at every wrap the lower-third can narrow to.
    #[test]
    fn the_cards_are_two_lines() {
        let counted = model::repeated_answers(ASKED_AGAIN).unwrap();
        let ratios: Vec<Q> = model::by_yeses(&counted)
            .into_iter()
            .map(|c| model::pattern_ratio(c).unwrap())
            .collect();
        let after = model::posterior(Q::new(1, 5), 3, 3);
        for text in [
            grid_card(64),
            ratios_card(&ratios, "3,9,27,81,243"),
            prior_card(after, 4, 20, Q::new(27, 28), Q::ZERO),
            memo_card(),
            odds_card(),
            rappor_card(),
        ] {
            for width in (72..=92).rev().step_by(4) {
                let lines = explainer_kit::animatic::wrap(&text, width, 3);
                assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
            }
        }
    }

    /// Chapter 11's numbers are the facts': 75 yes and 25 no leave 3⁵⁰, a
    /// 24-digit number; from even odds the chance of no copying is
    /// 1/(3⁵⁰ + 1), with 23 zeros after the point (RAPPOR's 1.39e-24); and
    /// the hundred days sit in a 10 × 10 strip.
    #[test]
    fn a_hundred_answers_are_the_facts() {
        let f = facts::facts().unwrap();
        let (yes, asks) = RAPPOR_EXAMPLE;
        let ratio = model::likelihood_ratio(true).pow(yes) / model::likelihood_ratio(false).pow(asks - yes);
        assert_eq!(f.get("R-F22.ratio"), Some(ratio.to_string().as_str()));
        assert_eq!(
            f.get("R-F22.ratio_digits"),
            Some(ratio.num().to_string().len().to_string().as_str())
        );
        let no = Q::ONE - model::posterior(Q::new(1, 2), yes, asks);
        assert_eq!(f.get("R-F22.posterior_no"), Some(no.to_string().as_str()));
        assert_eq!(
            f.get("R-F22.zeros"),
            Some(model::zeros_after_point(no).unwrap().to_string().as_str())
        );
        assert_eq!(
            f.get("R-X3.rappor.no_after_75_of_100"),
            Some(format!("{:.2e}", no.to_f64()).as_str())
        );
        assert_eq!(asks as usize, STRIP_COLUMNS * STRIP_COLUMNS);
        assert!(rappor_card().ends_with("not this one-bit rule"));
        assert_eq!(f.get("R-X7.rappor.memoized"), Some("Permanent randomized response"));
    }

    /// The layout keeps its pieces apart and inside the frame.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        use crate::chapters::draw::DISC_R;
        const BAND_TOP: f64 = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0;
        // 04_1: the student clear of the first ask; the coins inside their places
        const { assert!(STUDENT_X + DISC_R < ASK_X0 - PLACE_W / 2.0 - 0.3, "the student apart") };
        const { assert!(ASK_DX > PLACE_W + 0.3, "the asks apart") };
        const { assert!(COIN_DX + COIN_R < PLACE_W / 2.0 - 0.05, "the coins inside") };
        const { assert!(COIN_DX > COIN_R + 0.02, "the two coins apart") };
        const { assert!(COIN_R < PLACE_H / 2.0 - 0.05, "the coins inside, vertically") };
        // the grids: apart, inside the title-safe edge, under their names,
        // over their counts
        const {
            assert!(
                PANEL_X[0] + PANEL_HALF < PANEL_X[1] - PANEL_HALF - 0.5,
                "the grids apart"
            )
        };
        const {
            assert!(
                PANEL_X[0] - PANEL_HALF > -FRAME_X_RADIUS + 0.25 + 0.6,
                "the left grid and its counts inside the title-safe edge"
            )
        };
        const { assert!(PANEL_Y + PANEL_HALF < HEAD_Y - 0.25, "the grids under their names") };
        const { assert!(HEAD_Y + 0.15 < FRAME_Y_RADIUS - 0.25, "the names inside the top edge") };
        const { assert!(PANEL_Y - PANEL_HALF > LABEL_Y[0] + 0.25, "the counts under the grids") };
        const {
            assert!(
                LABEL_Y[0] - LABEL_Y[1] > 0.3 && LABEL_Y[1] - CLASS_Y > 0.5,
                "the counts apart"
            )
        };
        const { assert!(CLASS_Y - 0.25 > BAND_TOP + 0.3, "the products over the band") };
        const { assert!(PITCH > SIDE + 0.04, "squares apart") };
        // the right column: clear of the right grid, inside the frame, its
        // slots apart and over the band; the think ring under the total
        const {
            assert!(
                PANEL_X[1] + PANEL_HALF < RIGHT_X - 1.8,
                "the right column clear of the grid"
            )
        };
        const {
            assert!(
                RIGHT_X + 1.8 < FRAME_X_RADIUS - 0.25,
                "the right column inside the frame"
            )
        };
        const {
            assert!(
                RIGHT_Y[0] + 0.25 < FRAME_Y_RADIUS - 0.25,
                "the total inside the top edge"
            )
        };
        const {
            assert!(
                RIGHT_Y[0] - RIGHT_Y[1] > 0.9 && RIGHT_Y[1] - RIGHT_Y[2] > 0.9 && RIGHT_Y[2] - RIGHT_Y[3] > 0.9,
                "the right column's slots apart"
            )
        };
        const { assert!(RIGHT_Y[3] - 0.45 > BAND_TOP + 0.3, "the last fraction over the band") };
        const { assert!(THINK_Y + 0.62 < RIGHT_Y[0] - 0.3, "the think ring under the total") };
        const {
            assert!(
                RIGHT_X - 0.62 > PANEL_X[1] + PANEL_HALF + 0.5,
                "the think ring clear of the grid"
            )
        };
        // the memoized columns: inside their grid's place, apart, between the
        // names and the counts under them
        const {
            assert!(
                1.5 * MEMO_DX + MEMO_SIDE / 2.0 < PANEL_HALF,
                "the columns inside the grid's place"
            )
        };
        const { assert!(MEMO_DX > MEMO_SIDE + 0.15, "the columns apart") };
        const { assert!(MEMO_DY > MEMO_SIDE + 0.05, "the squares apart") };
        const {
            assert!(
                MEMO_FACE_Y - 0.12 > MEMO_Y0 + MEMO_SIDE / 2.0 + 0.1,
                "the faces over the columns"
            )
        };
        const { assert!(MEMO_FACE_Y + 0.12 < HEAD_Y - 0.25, "the faces under the names") };
        const {
            assert!(
                MEMO_Y0 - 2.0 * MEMO_DY - MEMO_SIDE / 2.0 > MEMO_LABEL_Y + 0.25,
                "the counts under the columns"
            )
        };
        // chapter 11's strip: inside the left edge, under the student and its
        // caption, over the tally, the tally over the band
        let (first, last) = (day_at(0), day_at(STRIP_COLUMNS * STRIP_COLUMNS - 1));
        assert!(first[0] - DAY_SIDE / 2.0 > -FRAME_X_RADIUS + 0.25);
        assert!(first[1] + DAY_SIDE / 2.0 < STUDENT_Y - DISC_R - 0.15);
        assert!(last[1] - DAY_SIDE / 2.0 > TALLY_Y + 0.25);
        const { assert!(TALLY_Y - 0.15 > BAND_TOP + 0.2, "the tally over the band") };
        const { assert!(CAPTION_Y + 0.15 < FRAME_Y_RADIUS - 0.25, "the caption inside the top") };
        const { assert!(CAPTION_Y - 0.15 > STUDENT_Y + DISC_R, "the caption over the student") };
        const { assert!(DAY_DX > DAY_SIDE + 0.05, "the days apart") };
        // the mathematics at the right: its rows apart, the last over the band;
        // the 24-digit number (about 7.8 wide at 24 points) inside the frame,
        // right of the strip
        for pair in MATH_Y.windows(2) {
            assert!(pair[0] - pair[1] >= 0.45, "{pair:?}");
        }
        const { assert!(MATH_Y[0] + 0.4 < FRAME_Y_RADIUS - 0.25, "the fraction inside the top") };
        const { assert!(MATH_Y[6] - 0.15 > BAND_TOP + 0.2, "the zeros' words over the band") };
        assert!(MATH_X + 3.9 < FRAME_X_RADIUS - 0.25);
        assert!(MATH_X - 3.9 > last[0] + DAY_SIDE / 2.0 + 0.2);
        // the closing diagram's boxes inside the frame, apart, over the band
        const { assert!(BOX_X + BOX_W / 2.0 < FRAME_X_RADIUS - 0.25, "the boxes inside") };
        const { assert!(BOX_X - BOX_W / 2.0 > 0.3, "the boxes apart") };
        const { assert!(BOX_Y - BOX_H / 2.0 > BAND_TOP + 0.3, "the boxes over the band") };
    }
}
