//! Chapter 05, the dial: the first coin becomes a dial, the chance t of
//! telling the truth, ours at one half (05_1); three quarters, a spinner of
//! four slices over eight equally likely outcomes (05_2); the prediction,
//! how many times as likely a yes is from someone who copied (05_3), and
//! its answer, seven, counted (05_4); every answer says more as the dial
//! turns up (05_5) while the count gets sharper, our class of twenty
//! missing by about two people, not four (05_6); at a quarter, under twice
//! as likely and a miss of about nine (05_7); the edges, the direct
//! question (05_8) and a pure coin (05_9); the promise for one answer each,
//! everyone following the rule (05_10).
//!
//! The voice's misses are 03_8's: how far the fixed class's count usually
//! strays, the square root of its variance, n times each person's estimate
//! variance that the table shows. That is the coins' variance in a person's
//! part of the estimate, an answer's variance (03_8's 3/16) times (1/t)²:
//! 4 × 3/16 = 3/4 for ours. Every card stays up at least its reading time.
//!
//! Every setting is `model::Channel` at k/4, the spinner's slice standing in
//! for the first coin of `model::answer`. Every square is that rule run on
//! one truth and one outcome (a slice and the second coin's face), and each
//! row's yes count is checked against `Channel::p_said` before it is drawn;
//! every number in the table is `Channel::ratio` or `Channel::coin_variance`.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    bar, coin, face, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, lane, move_all, pause, say, square,
    tex_q, think, wait_until,
};
use crate::facts::{self, DIAL, SLICES};
use crate::model::{self, Channel, Coin, Q};
use crate::palette;

pub struct TheDial {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const DIAL_NAME: &str = "the chance of telling the truth";
const COPIED: &str = "copied";
const DIDNT: &str = "didn't copy";
const YES: &str = "yes";
/// The table's heads. Its variances are each person's part of the estimate
/// (the coins'), not of an answer: "estimate variance" (review 4137).
const HEADS: [&str; 3] = ["the dial", "times as likely", "estimate variance"];
const NO_BOUND: &str = "no bound";
const NO_ESTIMATE: &str = "no estimate";
/// The table's last row: any setting t, its ratio and its coins' variance.
const T_TEX: &str = "t";
const RATIO_TEX: &str = r"\frac{1+t}{1-t}";
const WOBBLE_TEX: &str = r"\frac{1-t^2}{4t^2}";

/// Every setting's equally likely outcomes: a slice, and the second coin's
/// face.
const OUTCOMES: usize = 2 * SLICES as usize;

// ------------------------------------------------------------------ layout

/// The dial across the top left: 0 at `DIAL_X0`, 1 at `DIAL_X0 + DIAL_W`,
/// its name above, its ticks' values below.
const DIAL_X0: f64 = -6.2;
const DIAL_W: f64 = 5.2;
const DIAL_Y: f64 = 3.0;
const DIAL_NAME_Y: f64 = 3.45;
const TICK: f64 = 0.12;
const TICK_LABEL_DY: f64 = 0.42;
const MARKER_R: f64 = 0.14;
/// The spinner's slices, each over its two outcomes; the second coin's
/// faces under them; the two rows of outcomes, copied on top, their names
/// on the left and their yes counts on the right.
const ROW_X0: f64 = -5.2;
const PITCH: f64 = 0.6;
const SIDE: f64 = 0.4;
const SLICE_Y: f64 = 1.75;
const SLICE_H: f64 = 0.4;
const SLICE_GAP: f64 = 0.1;
const FACE_Y: f64 = 1.25;
const ROW_Y: [f64; 2] = [0.7, 0.05];
const NAME_X: f64 = -6.8;
const COUNT_X: f64 = -0.6;
/// The table at the right: its columns, its heads, one row per setting (in
/// the facts' order, 0 to 1) and the general row under them.
const COL_X: [f64; 3] = [1.4, 3.4, 5.6];
const HEAD_Y: f64 = 3.3;
const TABLE_Y0: f64 = 2.75;
const TABLE_DY: f64 = 0.55;
const GENERAL_Y: f64 = -0.3;
/// The think time, under the table.
const THINK_X: f64 = 4.6;
const THINK_Y: f64 = -1.55;
/// The fixed class's misses under the outcome rows: their title, then one
/// line each for 1/2, 3/4 and 1/4.
const MISS_X: f64 = -3.6;
const MISS_TITLE_Y: f64 = -0.6;
const MISS_Y: [f64; 3] = [-1.1, -1.6, -2.1];

/// Where the dial points for the chance `t`.
fn dial_x(t: Q) -> f64 {
    DIAL_X0 + DIAL_W * t.to_f64()
}

/// The x of outcome `i` (slice i / 2, the second coin's face i % 2).
fn square_x(i: usize) -> f64 {
    ROW_X0 + PITCH * i as f64
}

/// The y of the table row for the setting at `r` in `DIAL`.
fn row_y(r: usize) -> f64 {
    TABLE_Y0 - TABLE_DY * r as f64
}

/// The second coin's face in outcome `i`.
fn second(i: usize) -> Coin {
    if i.is_multiple_of(2) { Coin::Heads } else { Coin::Tails }
}

/// What outcome `i` answers for someone whose truth is `copied`, with `k`
/// slices saying tell the truth: the rule, with the slice as its first coin.
fn said(copied: bool, k: i128, i: usize) -> bool {
    let truthful = ((i / 2) as i128) < k;
    let first = if truthful { Coin::Heads } else { Coin::Tails };
    model::answer(copied, first, second(i))
}

/// How many of the outcomes say yes for `copied` with `k` truthful slices,
/// checked against the channel's own chance of a yes.
fn yes_count(copied: bool, k: i128) -> Result<u64, String> {
    let counted = (0..OUTCOMES).filter(|&i| said(copied, k, i)).count() as u64;
    let chance = Channel::new(Q::new(k, SLICES))?.p_said(copied, true);
    if Q::new(i128::from(counted), OUTCOMES as i128) != chance {
        return Err(format!("{counted} of {OUTCOMES} yeses at {k} slices, not {chance}"));
    }
    Ok(counted)
}

/// The setting named `name`: its row in the table, its truthful slices,
/// its channel.
fn setting(name: &str) -> Result<(usize, i128, Channel), String> {
    let (r, &(_, k)) = DIAL
        .iter()
        .enumerate()
        .find(|(_, (d, _))| *d == name)
        .ok_or_else(|| format!("no dial setting {name}"))?;
    Ok((r, k, Channel::new(Q::new(k, SLICES))?))
}

/// A count beside a row: "yes: 7 of 8".
fn count_text(yes: u64) -> String {
    format!("{YES}: {yes} of {OUTCOMES}")
}

/// 05_2's exact version: the spinner and its outcomes, at `k` truthful
/// slices.
fn spinner_card(k: i128) -> String {
    format!(
        "a spinner of {SLICES} equal slices, {k} saying tell the truth and {} handing the answer to the fair second \
         coin: {SLICES} × 2 = {OUTCOMES} equally likely outcomes",
        SLICES - k
    )
}

/// 05_4's exact version: both answers' chances at the setting `c`, and the
/// one ratio they share.
fn ratio_card(c: Channel) -> Result<String, String> {
    let (copied, didnt) = (yes_count(true, k_of(c)?)?, yes_count(false, k_of(c)?)?);
    let ratio = c.ratio(true).ok_or("no ratio")?;
    if c.ratio(false) != Some(ratio) {
        return Err(format!("a yes and a no carry different ratios at {}", c.truth()));
    }
    Ok(format!(
        "at {} a yes is {copied} of {OUTCOMES} outcomes for someone who copied, {didnt} of {OUTCOMES} for anyone else, \
         and a no the reverse: {ratio} times either way, (1 + t)/(1 − t)",
        c.truth()
    ))
}

/// The truthful slices of the setting `c`.
fn k_of(c: Channel) -> Result<i128, String> {
    (c.truth() * Q::int(SLICES))
        .integer()
        .ok_or_else(|| format!("{} is not a whole number of slices", c.truth()))
}

/// 05_6's exact version: the table's variance is one person's part of the
/// estimate, not of their answer (chapter 03's 3/16): at our setting `c`,
/// the answer's variance times (1/t)², 4 × 3/16 = 3/4. A fixed class of `n`
/// has n times it, whose square root is the miss on screen, and its share
/// 1/n of it; a sample adds more (chapter 03, review 4137).
fn variance_card(c: Channel, n: usize) -> Result<String, String> {
    let answer = c.answer_variance(true);
    let per = c
        .coin_variance()
        .ok_or_else(|| format!("no estimate at {}", c.truth()))?;
    let factor = Q::ONE / c.truth();
    let scale = factor * factor;
    if c.answer_variance(false) != answer || scale * answer != per {
        return Err(format!(
            "at {} the estimate's {per} is not {scale} × an answer's {answer}",
            c.truth()
        ));
    }
    Ok(format!(
        "estimate variance per person (1 − t²)/(4t²), {scale} × an answer's {answer} = {per} at {}; class of {n}: {n} × \
         it; share it/{n}; a sample adds p(1 − p)/n",
        c.truth()
    ))
}

/// The title over the misses (05_6), for a class of `n`.
fn miss_title(n: usize) -> String {
    format!("our class of {n} misses by")
}

/// One line of the misses: the setting `c`, then the square root of its
/// fixed class of `n`'s count variance, to two places.
fn miss_tex(c: Channel, n: usize) -> Result<String, String> {
    let per = c.coin_variance().ok_or_else(|| format!("no miss at {}", c.truth()))?;
    let class = Q::int(n as i128) * per;
    Ok(format!(
        r"{}:\quad \sqrt{{{n} \times {}}} \approx {:.2}",
        tex_q(c.truth(), true),
        tex_q(per, true),
        class.to_f64().sqrt()
    ))
}

/// Seconds a lower-third needs on screen to be read: fifteen characters a
/// second (the video can be paused). Each card stays up at least this long
/// from the moment it is fully in, over the lines after it.
fn read_s(text: &str) -> f64 {
    text.chars().count() as f64 / 15.0
}

/// When a card shown now, fully in, has been read.
fn read_by(stage: &Stage<'_>, text: &str) -> f64 {
    stage.scene().time().to_f64() + read_s(text)
}

/// 05_8's exact version: the two edges.
fn edges_card() -> String {
    "at 1 nobody who didn't copy says yes, so nothing bounds what a yes reveals; at 0 everyone says yes half the time, \
     so no estimate exists"
        .to_owned()
}

/// 05_10's exact version: every condition the promise rests on, and
/// chapter 04's warning. It comes in at the line's start and holds its
/// reading time.
fn scope_card() -> String {
    "if everyone follows the rule, with fair, private coins, once each; fresh coins on each repeat multiply the ratio \
     (chapter 4)"
        .to_owned()
}

/// A cell of the table: a value in TeX, or words where there is none.
enum Cell {
    Tex(String),
    Words(&'static str),
}

/// A setting's ratio, or "no bound" where some answer never comes from one
/// of the truths.
fn ratio_cell(c: Channel) -> Cell {
    c.ratio(true)
        .map_or(Cell::Words(NO_BOUND), |r| Cell::Tex(tex_q(r, true)))
}

/// A setting's coins' variance, or "no estimate" where nobody tells the
/// truth.
fn wobble_cell(c: Channel) -> Cell {
    c.coin_variance()
        .map_or(Cell::Words(NO_ESTIMATE), |w| Cell::Tex(tex_q(w, true)))
}

/// One table cell in column `col` at height `y`. Detached.
fn cell(kit: &Kit, stage: &mut Stage<'_>, col: usize, y: f64, value: Cell) -> fmn::Result<Mob> {
    match value {
        Cell::Tex(src) => kit.tex(stage, &src, 28.0, Palette::ink(), &[], v(COL_X[col], y)),
        Cell::Words(w) => kit.text(stage, w, 20.0, Palette::ink(), v(COL_X[col], y)),
    }
}

/// A setting's whole table row. Detached.
fn table_row(kit: &Kit, stage: &mut Stage<'_>, r: usize, c: Channel) -> fmn::Result<[Mob; 3]> {
    Ok([
        cell(kit, stage, 0, row_y(r), Cell::Tex(tex_q(c.truth(), true)))?,
        cell(kit, stage, 1, row_y(r), ratio_cell(c))?,
        cell(kit, stage, 2, row_y(r), wobble_cell(c))?,
    ])
}

/// What the left half shows at one setting: its truthful slices, the
/// spinner, the two rows of outcomes (faint until counted) and their yes
/// counts.
struct Spinner {
    k: i128,
    slices: Vec<Mob>,
    rows: Vec<Mob>,
    counts: Vec<Mob>,
}

impl Spinner {
    /// The left half at `k` truthful slices, its outcomes coloured by their
    /// answers and counted when `counted`. Detached.
    fn new(kit: &Kit, stage: &mut Stage<'_>, k: i128, counted: bool) -> fmn::Result<Spinner> {
        let mut slices = Vec::with_capacity(SLICES as usize);
        for s in 0..SLICES as usize {
            let color = if (s as i128) < k {
                palette::truth()
            } else {
                palette::forced()
            };
            let x = (square_x(2 * s) + square_x(2 * s + 1)) / 2.0;
            slices.push(bar(
                stage,
                color,
                x,
                SLICE_Y - SLICE_H / 2.0,
                2.0 * PITCH - SLICE_GAP,
                SLICE_H,
            )?);
        }
        let (mut rows, mut counts) = (Vec::with_capacity(2 * OUTCOMES), Vec::new());
        for (r, copied) in [(0, true), (1, false)] {
            for i in 0..OUTCOMES {
                let color = if counted {
                    palette::answer(said(copied, k, i))
                } else {
                    palette::ghost()
                };
                rows.push(square(stage, color, v(square_x(i), ROW_Y[r]), SIDE)?);
            }
            if counted {
                let yes = yes_count(copied, k).map_err(fail)?;
                counts.push(kit.text_left(stage, &count_text(yes), 20.0, palette::yes(), v(COUNT_X, ROW_Y[r]))?);
            }
        }
        Ok(Spinner {
            k,
            slices,
            rows,
            counts,
        })
    }

    /// Swap this spinner for the one at `k` (keeping the slices when they do
    /// not change), with `old` and `new` going and coming in the same move.
    fn turn(
        self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        k: i128,
        counted: bool,
        old: &[Mob],
        new: &[Mob],
    ) -> fmn::Result<Spinner> {
        let mut next = Spinner::new(kit, stage, k, counted)?;
        let (mut going, mut coming) = (old.to_vec(), new.to_vec());
        if self.k == k {
            next.slices = self.slices;
        } else {
            going.extend(self.slices);
            coming.extend(next.slices.iter().copied());
        }
        going.extend(self.rows);
        going.extend(self.counts);
        coming.extend(next.rows.iter().copied());
        coming.extend(next.counts.iter().copied());
        fade_out_all(stage, &going, 0.2)?;
        fade_in_all(stage, &coming, 0.2)?;
        Ok(next)
    }
}

impl SceneConstruct for TheDial {
    fn name(&self) -> &str {
        "05_the_dial"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let a = facts::average().map_err(fail)?;
        let n = a.class.size();
        let (ours_row, ours_k, ours) = setting("ours").map_err(fail)?;
        if ours != Channel::two_coins() {
            return Err(fail(format!(
                "the dial's middle, {}, is not the two coins",
                ours.truth()
            )));
        }
        let (up_row, up_k, up) = setting("up").map_err(fail)?;
        let (down_row, down_k, down) = setting("down").map_err(fail)?;
        let (top_row, top_k, top) = setting("direct").map_err(fail)?;
        let (pure_row, pure_k, pure) = setting("pure").map_err(fail)?;

        // 05_1: the dial, ours at one half
        let line = say(&kit, stage, "05_1")?;
        let mut dial = vec![
            lane(
                stage,
                vec![v(DIAL_X0, DIAL_Y), v(DIAL_X0 + DIAL_W, DIAL_Y)],
                Palette::dim(),
            ),
            kit.text(
                stage,
                DIAL_NAME,
                20.0,
                Palette::dim(),
                v(DIAL_X0 + DIAL_W / 2.0, DIAL_NAME_Y),
            )?,
        ];
        for &(_, k) in &DIAL {
            let t = Q::new(k, SLICES);
            let x = dial_x(t);
            dial.push(lane(
                stage,
                vec![v(x, DIAL_Y - TICK), v(x, DIAL_Y + TICK)],
                Palette::dim(),
            ));
            dial.push(kit.tex(
                stage,
                &tex_q(t, true),
                26.0,
                Palette::dim(),
                &[],
                v(x, DIAL_Y - TICK_LABEL_DY),
            )?);
        }
        let marker = coin(stage, palette::truth(), v(dial_x(ours.truth()), DIAL_Y), MARKER_R);
        dial.push(marker);
        fade_in_all(stage, &dial, 0.5)?;
        line.wait_for(stage, "Ours sits")?;
        let mut left = Spinner::new(&kit, stage, ours_k, true)?;
        let mut fixed = Vec::with_capacity(OUTCOMES + 2);
        for i in 0..OUTCOMES {
            fixed.push(kit.text(stage, face(second(i)), 18.0, Palette::dim(), v(square_x(i), FACE_Y))?);
        }
        let names = [
            kit.text_left(stage, COPIED, 20.0, palette::copied(), v(NAME_X, ROW_Y[0]))?,
            kit.text_left(stage, DIDNT, 20.0, palette::student(), v(NAME_X, ROW_Y[1]))?,
        ];
        fixed.extend(names);
        let mut shown = left.slices.clone();
        shown.extend(fixed.iter().copied());
        shown.extend(left.rows.iter().copied());
        shown.extend(left.counts.iter().copied());
        fade_in_lagged(stage, &shown, 1.0, 0.02)?;
        let mut table = Vec::new();
        for (col, head) in HEADS.iter().enumerate() {
            table.push(kit.text(stage, head, 20.0, Palette::dim(), v(COL_X[col], HEAD_Y))?);
        }
        let ours_cells = table_row(&kit, stage, ours_row, ours)?;
        table.extend(ours_cells);
        fade_in_all(stage, &table, 0.4)?;

        // 05_2: three quarters; the outcomes go faint, the row waits
        let line = say(&kit, stage, "05_2")?;
        line.wait_for(stage, "three quarters")?;
        move_all(stage, &[(marker, v(dial_x(up.truth()), DIAL_Y))], 0.6)?;
        let up_t = cell(&kit, stage, 0, row_y(up_row), Cell::Tex(tex_q(up.truth(), true)))?;
        left = left.turn(&kit, stage, up_k, false, &[], &[up_t])?;
        line.wait_for(stage, "a spinner")?;
        let card_text = spinner_card(up_k);
        let card = precisely(&kit, stage, &card_text)?;
        fade_in_all(stage, &[card], 0.4)?;
        let card_read = read_by(stage, &card_text);

        // 05_3: the prediction; no 3/4 ratio and no coloured outcome yet
        let _line = say(&kit, stage, "05_3")?;
        let q = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 05_4: seven, counted, for both answers
        let _line = say(&kit, stage, "05_4")?;
        let up_ratio = cell(&kit, stage, 1, row_y(up_row), ratio_cell(up))?;
        let card2_text = ratio_card(up).map_err(fail)?;
        let card2 = precisely(&kit, stage, &card2_text)?;
        wait_until(stage, card_read)?;
        left = left.turn(&kit, stage, up_k, true, &[q, card], &[up_ratio, card2])?;
        let card2_read = read_by(stage, &card2_text);

        // 05_5: turning up, every answer says more
        let _line = say(&kit, stage, "05_5")?;
        let general = [
            cell(&kit, stage, 0, GENERAL_Y, Cell::Tex(T_TEX.into()))?,
            kit.tex(stage, RATIO_TEX, 30.0, Palette::ink(), &[], v(COL_X[1], GENERAL_Y))?,
        ];
        fade_in_all(stage, &general, 0.4)?;
        flash_all(stage, &[ours_cells[1], up_ratio], 0.6)?;

        // 05_6: the count gets sharper: our class misses by about two, not four
        let line = say(&kit, stage, "05_6")?;
        line.wait_for(stage, "miss by about two")?;
        let up_wobble = cell(&kit, stage, 2, row_y(up_row), wobble_cell(up))?;
        let general_wobble = kit.tex(stage, WOBBLE_TEX, 30.0, Palette::ink(), &[], v(COL_X[2], GENERAL_Y))?;
        let card3_text = variance_card(ours, n).map_err(fail)?;
        let card3 = precisely(&kit, stage, &card3_text)?;
        let title = kit.text(stage, &miss_title(n), 20.0, Palette::dim(), v(MISS_X, MISS_TITLE_Y))?;
        let mut misses = Vec::with_capacity(MISS_Y.len());
        for (c, y) in [ours, up].into_iter().zip(MISS_Y) {
            misses.push(kit.tex(
                stage,
                &miss_tex(c, n).map_err(fail)?,
                28.0,
                Palette::ink(),
                &[],
                v(MISS_X, y),
            )?);
        }
        wait_until(stage, card2_read)?;
        let mut coming = vec![up_wobble, general_wobble, card3, title];
        coming.extend(misses.iter().copied());
        fade_out_all(stage, &[card2], 0.2)?;
        fade_in_all(stage, &coming, 0.2)?;
        let card3_read = read_by(stage, &card3_text);
        flash_all(stage, &misses, 0.6)?;

        // 05_7: down at a quarter: under twice as likely, a miss of about nine
        let line = say(&kit, stage, "05_7")?;
        move_all(stage, &[(marker, v(dial_x(down.truth()), DIAL_Y))], 0.6)?;
        let down_cells = table_row(&kit, stage, down_row, down)?;
        left = left.turn(&kit, stage, down_k, true, &[], &down_cells)?;
        line.wait_for(stage, "about nine")?;
        misses.push(kit.tex(
            stage,
            &miss_tex(down, n).map_err(fail)?,
            28.0,
            Palette::ink(),
            &[],
            v(MISS_X, MISS_Y[2]),
        )?);
        fade_in_all(stage, &misses[2..], 0.3)?;
        flash_all(stage, &[misses[0], misses[2]], 0.6)?;

        // 05_8: all the way up, the direct question. The spinner, its rows
        // and the table row follow the knob at once, as at every other
        // setting; card 4 replaces card 3 only once card 3 has been read
        // (review PinkDuck F2).
        let _line = say(&kit, stage, "05_8")?;
        move_all(stage, &[(marker, v(dial_x(top.truth()), DIAL_Y))], 0.6)?;
        let top_cells = table_row(&kit, stage, top_row, top)?;
        left = left.turn(&kit, stage, top_k, true, &[], &top_cells)?;
        let card4_text = edges_card();
        let card4 = precisely(&kit, stage, &card4_text)?;
        wait_until(stage, card3_read)?;
        fade_out_all(stage, &[card3], 0.2)?;
        fade_in_all(stage, &[card4], 0.2)?;
        let card4_read = read_by(stage, &card4_text);

        // 05_9: all the way down, a pure coin
        let _line = say(&kit, stage, "05_9")?;
        move_all(stage, &[(marker, v(dial_x(pure.truth()), DIAL_Y))], 0.6)?;
        let pure_cells = table_row(&kit, stage, pure_row, pure)?;
        left = left.turn(&kit, stage, pure_k, true, &[], &pure_cells)?;

        // 05_10: in between, the promise for one answer each; its card first
        let line = say(&kit, stage, "05_10")?;
        let card5_text = scope_card();
        let card5 = precisely(&kit, stage, &card5_text)?;
        wait_until(stage, card4_read)?;
        fade_out_all(stage, &[card4], 0.2)?;
        fade_in_all(stage, &[card5], 0.2)?;
        let card5_read = read_by(stage, &card5_text);
        move_all(stage, &[(marker, v(dial_x(ours.truth()), DIAL_Y))], 0.6)?;
        let _left = left.turn(&kit, stage, ours_k, true, &[], &[])?;
        let mut between = down_cells.to_vec();
        between.extend(ours_cells);
        between.extend([up_t, up_ratio, up_wobble]);
        between.extend(misses.iter().copied());
        flash_all(stage, &between, 0.6)?;
        line.wait_end(stage)?;
        pause(stage, 2.0)?;
        wait_until(stage, card5_read)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("05_1", "Ours sits"),
            ("05_2", "three quarters"),
            ("05_2", "a spinner"),
            ("05_6", "miss by about two"),
            ("05_7", "about nine"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The spinner's outcomes are the channel at every setting: its counts
    /// are the channel's chances, and both answers' counted ratios are the
    /// channel's ratio, (1 + t)/(1 − t), none where a count is 0.
    #[test]
    fn the_spinner_is_the_channel() {
        let f = facts::facts().unwrap();
        for &(name, k) in &DIAL {
            let c = Channel::new(Q::new(k, SLICES)).unwrap();
            let (yc, yd) = (yes_count(true, k).unwrap(), yes_count(false, k).unwrap());
            assert_eq!(
                f.get(&format!("R-F16.{name}.yes.copied")),
                Some(yc.to_string().as_str())
            );
            assert_eq!(f.get(&format!("R-F16.{name}.yes.didnt")), Some(yd.to_string().as_str()));
            let outcomes = OUTCOMES as u64;
            let yes_ratio = Q::int(i128::from(yc)).checked_div(Q::int(i128::from(yd)));
            let no_ratio = Q::int(i128::from(outcomes - yd)).checked_div(Q::int(i128::from(outcomes - yc)));
            assert_eq!(yes_ratio, c.ratio(true), "{name}");
            assert_eq!(no_ratio, c.ratio(false), "{name}");
            assert_eq!(k_of(c).unwrap(), k);
        }
        // the edges: nobody who didn't copy says yes at 1; everyone half the time at 0
        assert_eq!(yes_count(false, SLICES).unwrap(), 0);
        assert_eq!(yes_count(true, 0).unwrap(), OUTCOMES as u64 / 2);
        assert_eq!(yes_count(false, 0).unwrap(), OUTCOMES as u64 / 2);
        // a count the channel does not give is refused, not drawn
        assert!(yes_count(true, SLICES + 1).is_err());
    }

    /// The general row's formulas are the channel's, at every twelfth of the
    /// dial: the ratio (1 + t)/(1 − t), none at 1; the coins' variance
    /// (1 − t²)/(4t²), none at 0.
    #[test]
    fn the_formulas_on_screen_are_the_channels() {
        assert_eq!(RATIO_TEX, r"\frac{1+t}{1-t}");
        assert_eq!(WOBBLE_TEX, r"\frac{1-t^2}{4t^2}");
        for j in 0..=12 {
            let t = Q::new(j, 12);
            let c = Channel::new(t).unwrap();
            assert_eq!(c.ratio(true), (Q::ONE + t).checked_div(Q::ONE - t), "{t}");
            assert_eq!(c.ratio(false), c.ratio(true), "{t}");
            assert_eq!(
                c.coin_variance(),
                (Q::ONE - t * t).checked_div(Q::int(4) * t * t),
                "{t}"
            );
        }
        // up the ratio, down the variance, everywhere between the edges
        for j in 1..11 {
            let (a, b) = (
                Channel::new(Q::new(j, 12)).unwrap(),
                Channel::new(Q::new(j + 1, 12)).unwrap(),
            );
            assert!(b.ratio(true).unwrap() > a.ratio(true).unwrap());
            assert!(b.coin_variance().unwrap() < a.coin_variance().unwrap());
        }
    }

    /// The words on screen say what the facts say.
    #[test]
    fn the_cards_are_built_from_the_facts() {
        let f = facts::facts().unwrap();
        assert_eq!(
            spinner_card(3),
            "a spinner of 4 equal slices, 3 saying tell the truth and 1 handing the answer to the fair second coin: 4 \
             × 2 = 8 equally likely outcomes"
        );
        let (_, _, up) = setting("up").unwrap();
        assert_eq!(
            ratio_card(up).unwrap(),
            "at 3/4 a yes is 7 of 8 outcomes for someone who copied, 1 of 8 for anyone else, and a no the reverse: 7 \
             times either way, (1 + t)/(1 − t)"
        );
        assert_eq!(f.get("R-F16.up.ratio"), Some("7"));
        assert_eq!(f.get("R-F16.class.count_variance"), Some("15"));
        assert_eq!(f.get("R-F16.class.share_variance"), Some("3/80"));
        // the estimate's variance per person, not an answer's: 4 × 3/16 = 3/4
        let (_, _, ours) = setting("ours").unwrap();
        assert_eq!(
            variance_card(ours, 20).unwrap(),
            "estimate variance per person (1 − t²)/(4t²), 4 × an answer's 3/16 = 3/4 at 1/2; class of 20: 20 × it; \
             share it/20; a sample adds p(1 − p)/n"
        );
        assert_eq!(f.get("R-F14.answer_variance"), Some("3/16"));
        assert_eq!(f.get("R-F16.ours.coin_variance"), Some("3/4"));
        assert_eq!(f.get("R-F4.factor"), Some("2"));
        assert_eq!(HEADS[2], "estimate variance");
        // where nobody tells the truth there is no estimate to give a variance
        let (_, _, pure) = setting("pure").unwrap();
        assert!(variance_card(pure, 20).is_err());
        // the misses on screen are the facts' (03_8's for ours), to two places
        let (_, _, down) = setting("down").unwrap();
        assert_eq!(miss_title(20), "our class of 20 misses by");
        assert_eq!(
            miss_tex(ours, 20).unwrap(),
            r"\tfrac{1}{2}:\quad \sqrt{20 \times \tfrac{3}{4}} \approx 3.87"
        );
        assert_eq!(
            miss_tex(up, 20).unwrap(),
            r"\tfrac{3}{4}:\quad \sqrt{20 \times \tfrac{7}{36}} \approx 1.97"
        );
        assert_eq!(
            miss_tex(down, 20).unwrap(),
            r"\tfrac{1}{4}:\quad \sqrt{20 \times \tfrac{15}{4}} \approx 8.66"
        );
        for (name, miss) in [("ours", "3.87"), ("up", "1.97"), ("down", "8.66")] {
            assert_eq!(f.get(&format!("R-F16.{name}.miss_2dp")), Some(miss));
        }
        assert_eq!(f.get("R-F7.sd_estimate"), Some("3.87"));
        // the spoken "about two people, not four" and "about nine"
        assert_eq!(f.get("R-F16.up.miss_rounded"), Some("2"));
        assert_eq!(f.get("R-F16.ours.miss_rounded"), Some("4"));
        assert_eq!(f.get("R-F16.down.miss_rounded"), Some("9"));
        assert!(miss_tex(pure, 20).is_err());
        // the edges card's words: at 1 no yes from anyone who didn't copy,
        // so no bound; at 0 a yes half the time from everyone, no estimate
        assert_eq!(f.get("R-F16.direct.yes.didnt"), Some("0"));
        assert_eq!(f.get("R-F16.direct.ratio"), Some("unbounded"));
        assert_eq!(f.get("R-F16.pure.yes.copied"), Some("4"));
        assert_eq!(f.get("R-F16.pure.yes.didnt"), Some("4"));
        assert_eq!(f.get("R-F16.pure.coin_variance"), Some("none"));
        assert_eq!(f.get("R-X1.pure_coin.estimate"), Some("none"));
        assert!(edges_card().starts_with("at 1 nobody who didn't copy says yes"));
        assert!(edges_card().ends_with("at 0 everyone says yes half the time, so no estimate exists"));
        assert_eq!(
            scope_card(),
            "if everyone follows the rule, with fair, private coins, once each; fresh coins on each repeat multiply \
             the ratio (chapter 4)"
        );
        assert_eq!(count_text(7), "yes: 7 of 8");
        // fifteen characters a second
        assert!((read_s(&scope_card()) - scope_card().chars().count() as f64 / 15.0).abs() < 1e-12);
        // the table's cells: words exactly where the channel has no number
        let (_, _, top) = setting("direct").unwrap();
        assert!(matches!(ratio_cell(top), Cell::Words(NO_BOUND)));
        assert!(matches!(wobble_cell(pure), Cell::Words(NO_ESTIMATE)));
        assert!(matches!(ratio_cell(pure), Cell::Tex(ref s) if s == "1"));
        assert!(matches!(wobble_cell(top), Cell::Tex(ref s) if s == "0"));
    }

    /// The cards stay two lines at every wrap the lower-third can narrow to.
    #[test]
    fn the_cards_are_two_lines() {
        let (_, _, up) = setting("up").unwrap();
        let (_, _, ours) = setting("ours").unwrap();
        for text in [
            spinner_card(3),
            ratio_card(up).unwrap(),
            variance_card(ours, 20).unwrap(),
            edges_card(),
            scope_card(),
        ] {
            for width in (72..=92).rev().step_by(4) {
                let lines = explainer_kit::animatic::wrap(&text, width, 3);
                assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
            }
        }
    }

    /// The layout keeps its pieces apart and inside the frame.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        const BAND_TOP: f64 = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0;
        // the dial: inside the title-safe edges, its name and labels clear
        const {
            assert!(
                DIAL_X0 - 0.3 > -FRAME_X_RADIUS + 0.25,
                "the dial's 0 inside the left edge"
            )
        };
        const {
            assert!(
                DIAL_NAME_Y + 0.15 < FRAME_Y_RADIUS - 0.25,
                "the dial's name inside the top"
            )
        };
        const {
            assert!(
                DIAL_NAME_Y - 0.12 > DIAL_Y + MARKER_R + 0.05,
                "the name over the marker"
            )
        };
        const {
            assert!(
                DIAL_Y - TICK_LABEL_DY - 0.25 > SLICE_Y + SLICE_H / 2.0 + 0.1,
                "the tick values over the slices"
            )
        };
        // the spinner over its faces, the faces over the rows, the rows apart
        const { assert!(SLICE_Y - SLICE_H / 2.0 > FACE_Y + 0.15, "the slices over the faces") };
        const { assert!(FACE_Y - 0.15 > ROW_Y[0] + SIDE / 2.0, "the faces over the rows") };
        const { assert!(ROW_Y[0] - ROW_Y[1] > SIDE + 0.15, "the rows apart") };
        const { assert!(PITCH > SIDE + 0.1, "the squares apart") };
        const { assert!(ROW_Y[1] - SIDE / 2.0 > BAND_TOP + 0.3, "the rows over the band") };
        // the names left of the squares; the counts right of them, left of the table
        const { assert!(NAME_X > -FRAME_X_RADIUS + 0.25, "the names inside the left edge") };
        assert!(NAME_X + 1.1 < square_x(0) - SIDE / 2.0);
        assert!(square_x(OUTCOMES - 1) + SIDE / 2.0 < COUNT_X - 0.1);
        assert!(square_x(OUTCOMES - 1) + SIDE / 2.0 < DIAL_X0 + DIAL_W + 0.3);
        const { assert!(COUNT_X + 1.2 < COL_X[0] - 0.3, "the counts left of the table") };
        // the table: its columns apart and inside the right edge, its rows
        // under the heads, the general row under the last, all over the band
        const {
            assert!(
                COL_X[1] - COL_X[0] > 1.6 && COL_X[2] - COL_X[1] > 1.8,
                "the columns apart"
            )
        };
        const {
            assert!(
                COL_X[2] + 0.8 < FRAME_X_RADIUS - 0.25,
                "the last column inside the right edge"
            )
        };
        const { assert!(HEAD_Y + 0.15 < FRAME_Y_RADIUS - 0.25, "the heads inside the top") };
        const { assert!(HEAD_Y - 0.15 > TABLE_Y0 + 0.25, "the rows under the heads") };
        assert!(row_y(DIAL.len() - 1) - 0.25 > GENERAL_Y + 0.4);
        const { assert!(GENERAL_Y - 0.45 > BAND_TOP + 0.3, "the general row over the band") };
        // the think ring under the table's last rows, clear of the counts and the band
        const {
            assert!(
                THINK_Y + 0.62 < GENERAL_Y - 0.45 + 0.1,
                "the think ring under the general row"
            )
        };
        const { assert!(THINK_X - 0.62 > COUNT_X + 1.2, "the think ring clear of the counts") };
        const { assert!(THINK_Y - 0.62 - 0.45 > BAND_TOP, "the think note over the band") };
        // the misses under the rows: their title under the second row, the
        // lines apart, the last over the band, all left of the think ring
        const {
            assert!(
                MISS_TITLE_Y + 0.12 < ROW_Y[1] - SIDE / 2.0 - 0.2,
                "the misses' title under the rows"
            )
        };
        const { assert!(MISS_TITLE_Y - 0.12 > MISS_Y[0] + 0.3, "the title over the first miss") };
        const {
            assert!(
                MISS_Y[0] - MISS_Y[1] > 0.45 && MISS_Y[1] - MISS_Y[2] > 0.45,
                "the misses apart"
            )
        };
        const { assert!(MISS_Y[2] - 0.3 > BAND_TOP + 0.05, "the last miss over the band") };
        const {
            assert!(
                MISS_X - 2.0 > -FRAME_X_RADIUS + 0.25 && MISS_X + 2.0 < THINK_X - 0.62 - 0.3,
                "the misses inside the left edge, clear of the think ring"
            )
        };
        // every setting's mark on the dial apart from the next
        let xs: Vec<f64> = DIAL.iter().map(|&(_, k)| dial_x(Q::new(k, SLICES))).collect();
        for pair in xs.windows(2) {
            assert!(pair[1] - pair[0] > 0.9, "{pair:?}");
        }
    }
}
