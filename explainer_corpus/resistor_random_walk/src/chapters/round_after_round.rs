//! Chapter 03, average and average again: the circuit with a guess of zero
//! at both middle junctions, and a table of rounds (03_1); the rule for a
//! round, every middle junction at once from the previous round's values
//! (03_2); round one, the far ends' old values sliding in (03_3); rounds two
//! to four (03_4); the bigger gap shrinks to a quarter a round, because one
//! wire at each junction leads to an unknown (03_5); the prediction (03_6);
//! never exactly, though within a thousandth after five rounds (03_7); the
//! limit (03_8); why the ends stay fixed: averaged too, every value drifts to
//! one number (03_9).

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    END_LABEL_GAP, LABEL_SIZE, Picture, battery, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, move_all,
    pause, picture, recolor, say, slide_in, small_tex, swap, tex, think, tokens, value_texts, volts_text, wait_until,
};
use crate::facts::{self, ROUNDS};
use crate::model::{BOTTOM, LEFT, Q, RIGHT, TOP};
use crate::palette;

pub struct RoundAfterRound {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const HEADERS: [&str; 3] = ["round", "left", "right"];
const GAP_HEADER: &str = "bigger gap";
const LIMIT: &str = "limit";

/// Where the circuit stands (as chapter 02 had it), and the table: its
/// columns (round, left, right, bigger gap), its header, its rows and the
/// limit row set apart under them.
const PICTURE_AT: [f64; 2] = [-4.0, 0.3];
const COLUMN_X: [f64; 4] = [0.85, 2.3, 3.8, 5.45];
const HEADER_Y: f64 = 2.85;
const ROW0_Y: f64 = 2.3;
const ROW_STEP: f64 = 0.55;
const LIMIT_Y: f64 = -1.1;
const CELL_SIZE: f64 = 30.0;
const HEADER_SIZE: f64 = 22.0;
/// The prediction's countdown, under the table's first five rows.
const THINK_AT: [f64; 2] = [3.2, -1.35];
/// How long a round takes once the rule is shown (03_4's third and fourth,
/// 03_7's fifth), and a round of the ends' drift (03_9).
const QUICK: f64 = 0.35;
const DRIFT: f64 = 0.35;
/// A lower-third's fade in and out, the gap column's arrival before 03_5's
/// card, and the beat after the last line.
const CARD_IN: f64 = 0.4;
const CARD_OUT: f64 = 0.3;
const GAP_COLUMN_IN: f64 = 1.2;
const END_BEAT: f64 = 0.6;

/// Seconds a lower-third needs on screen to be read: fifteen characters a
/// second, a quick reader (the video can be paused). Each card stays up at
/// least this long from the moment it is fully in, over the lines after it
/// where need be.
fn read_s(text: &str) -> f64 {
    text.chars().count() as f64 / 15.0
}

/// The y of round `n`'s row.
fn row_y(n: usize) -> f64 {
    ROW0_Y - ROW_STEP * n as f64
}

/// A table cell, TeX at column `c` and height `y`. Detached.
fn cell(kit: &Kit, stage: &mut Stage<'_>, src: &str, c: usize, y: f64) -> fmn::Result<Mob> {
    kit.tex(stage, src, CELL_SIZE, Palette::ink(), &[], v(COLUMN_X[c], y))
}

/// Round `n`'s row: its number and the two middle junctions' values.
/// Detached.
fn round_row(kit: &Kit, stage: &mut Stage<'_>, n: usize, values: &[Q]) -> fmn::Result<Vec<Mob>> {
    let y = row_y(n);
    Ok(vec![
        cell(kit, stage, &n.to_string(), 0, y)?,
        cell(kit, stage, &small_tex(values[LEFT]), 1, y)?,
        cell(kit, stage, &small_tex(values[RIGHT]), 2, y)?,
    ])
}

/// The labels of the middle junctions in `which`, now reading `values`,
/// swapped in for the old ones, each junction's colour following its value.
fn relabel(
    kit: &Kit,
    stage: &mut Stage<'_>,
    pic: &Picture,
    labels: &mut [Mob; 2],
    which: &[usize],
    values: &[Q],
    rt: f64,
) -> fmn::Result<()> {
    let (mut old, mut new, mut colours) = (Vec::new(), Vec::new(), Vec::new());
    for &x in which {
        let i = usize::from(x != LEFT);
        let next = kit.tex(
            stage,
            &tex(values[x]),
            LABEL_SIZE,
            Palette::ink(),
            &[],
            pic.label_spot(x, END_LABEL_GAP),
        )?;
        old.push(labels[i]);
        new.push(next);
        labels[i] = next;
        colours.push((pic.junctions[x], palette::value(values[x])));
    }
    fade_out_all(stage, &old, rt / 2.0)?;
    fade_in_all(stage, &new, rt / 2.0)?;
    recolor(stage, &colours, rt)
}

/// 03_2's exact version: which updates this chapter uses, and the other
/// kind for comparison (from the facts).
fn jacobi_card() -> Result<String, String> {
    let seq = facts::sequential_rounds(8)?;
    let steady = facts::steady_factor(&facts::gaps(&seq)?, facts::SEQUENTIAL_FROM)?;
    Ok(format!(
        "every middle junction updates at once from the previous round (Jacobi's method). One at a time with the \
         newest values (Gauss-Seidel), the gap shrinks to {steady} a round from round {} on.",
        facts::SEQUENTIAL_FROM
    ))
}

/// The last round the table shows before the prediction: the 03_5 card
/// stops there, so round 5 is revealed only after the countdown, and the
/// 03_7 card cites nothing later.
const BEFORE_PREDICTION: usize = ROUNDS - 1;

/// 03_5's exact version: why the bigger gap shrinks to exactly a quarter
/// on this network (every far end but the other junction is the top or the
/// bottom, which never change), with the gaps of the rounds already on
/// screen.
fn gap_card() -> Result<String, String> {
    let wires = facts::wires_at_middle()?;
    let (unknown, weight) = facts::unknown_link()?;
    let fixed = wires
        .checked_sub(unknown)
        .ok_or("more wires to an unknown than wires")?;
    let gaps = facts::gaps(&facts::rounds(BEFORE_PREDICTION)?)?;
    let shrink = facts::steady_factor(&gaps, 1)?;
    let shown: Vec<String> = gaps.iter().map(Q::to_string).collect();
    let (last, rest) = shown.split_last().ok_or("no gaps to show")?;
    Ok(format!(
        "{fixed} of each junction's {wires} far ends never change, so its new gap is {weight} of the other's old \
         one: the bigger gap is exactly {shrink} of the last, {} and {last} by round {BEFORE_PREDICTION}.",
        rest.join(", ")
    ))
}

/// 03_7's exact version: why no round reaches the answer, and what does.
/// Each new value is the other junction's previous value plus the whole
/// number its fixed far ends add to, divided by its four wires; from the
/// guess of 0 every value stays over a power of 2. Its examples are rows
/// already on screen when it comes: round 5's arrives later in the line.
fn never_card() -> Result<String, String> {
    let rounds = facts::rounds(BEFORE_PREDICTION)?;
    let wires = facts::wires_at_middle()?;
    let v = facts::voltages()?;
    Ok(format!(
        "each new value is the other junction's previous value plus a whole number, divided by {wires}, so from \
         the guess of {} every value is over a power of 2, like {} and {}, and never {}. The gaps go to {}, so the \
         values have a limit.",
        rounds[0][LEFT],
        rounds[2][LEFT],
        rounds[BEFORE_PREDICTION][LEFT],
        v[LEFT],
        Q::ZERO
    ))
}

/// 03_9's exact version: what would happen were the top and the bottom
/// averaged too, from the solved values: every value drifts to their mean,
/// each junction counted by its wires (within [`facts::tolerance`] after
/// the rounds the picture plays).
fn drift_card() -> Result<String, String> {
    let net = facts::network()?;
    let (free, settle) = facts::drift()?;
    let start = free.first().ok_or("no free rounds")?;
    let wires = |x: usize| net.total(x);
    Ok(format!(
        "with the top and bottom averaged too, {}, {}, {}, {} drift to {settle}, their mean weighted by wires {}, \
         {}, {}, {}.",
        start[TOP],
        start[LEFT],
        start[RIGHT],
        start[BOTTOM],
        wires(TOP),
        wires(LEFT),
        wires(RIGHT),
        wires(BOTTOM)
    ))
}

impl SceneConstruct for RoundAfterRound {
    fn name(&self) -> &str {
        "03_round_after_round"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let net = facts::network().map_err(fail)?;
        let volts = facts::voltages().map_err(fail)?;
        let rounds = facts::rounds(ROUNDS).map_err(fail)?;
        let gaps = facts::gaps(&rounds).map_err(fail)?;

        // 03_1: the circuit, the guess of zero, and the table's first row
        let line = say(&kit, stage, "03_1")?;
        let guess = &rounds[0];
        let colors: Vec<Srgb> = (0..net.size())
            .map(|x| match x {
                TOP => palette::top(),
                BOTTOM => palette::bottom(),
                _ => palette::value(guess[x]),
            })
            .collect();
        let pic = picture(stage, &net, PICTURE_AT, &colors)?;
        let mut cell_mobs = battery(stage, pic.centre)?;
        cell_mobs.push(kit.text(
            stage,
            &volts_text(volts[TOP]),
            24.0,
            Palette::ink(),
            pic.label_spot(TOP, END_LABEL_GAP),
        )?);
        cell_mobs.push(kit.text(
            stage,
            &volts_text(volts[BOTTOM]),
            24.0,
            Palette::ink(),
            pic.label_spot(BOTTOM, END_LABEL_GAP),
        )?);
        let mut labels = [
            kit.tex(
                stage,
                &tex(guess[LEFT]),
                LABEL_SIZE,
                Palette::ink(),
                &[],
                pic.label_spot(LEFT, END_LABEL_GAP),
            )?,
            kit.tex(
                stage,
                &tex(guess[RIGHT]),
                LABEL_SIZE,
                Palette::ink(),
                &[],
                pic.label_spot(RIGHT, END_LABEL_GAP),
            )?,
        ];
        let mut header = Vec::new();
        for (c, h) in HEADERS.iter().enumerate() {
            header.push(kit.text(stage, h, HEADER_SIZE, Palette::dim(), v(COLUMN_X[c], HEADER_Y))?);
        }
        let first_row = round_row(&kit, stage, 0, guess)?;
        let mut arriving = pic.mobs();
        arriving.extend(&cell_mobs);
        arriving.extend(labels);
        arriving.extend(&header);
        arriving.extend(&first_row);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, "Guess zero")?;
        let mut zeros = labels.to_vec();
        zeros.extend(&first_row[1..]);
        flash_all(stage, &zeros, 0.6)?;

        // 03_2: the rule for a round, every junction at once from the round
        // before; the other kind of update, for comparison (its card stays
        // over round one, which shows it, until it has been read)
        let line = say(&kit, stage, "03_2")?;
        flash_all(stage, &pic.wires_at(LEFT), 0.5)?;
        flash_all(stage, &pic.wires_at(RIGHT), 0.5)?;
        line.wait_for(stage, "using the values")?;
        let rule_text = jacobi_card().map_err(fail)?;
        let rule = precisely(&kit, stage, &rule_text)?;
        fade_in_all(stage, &[rule], CARD_IN)?;
        let rule_read = stage.scene().time().to_f64() + read_s(&rule_text);

        // 03_3: round one; both junctions' far ends read from round 0, at
        // once; the left takes its average, then the right, whose token from
        // the left still reads the old value
        let line = say(&kit, stage, "03_3")?;
        let left_in = tokens(
            &kit,
            stage,
            &pic,
            LEFT,
            &value_texts(&facts::far_values(guess, LEFT).map_err(fail)?),
        )?;
        let right_in = tokens(
            &kit,
            stage,
            &pic,
            RIGHT,
            &value_texts(&facts::far_values(guess, RIGHT).map_err(fail)?),
        )?;
        let mut both = left_in.clone();
        both.extend(&right_in);
        fade_in_all(stage, &both, 0.4)?;
        line.wait_for(stage, "The left averages")?;
        slide_in(stage, &left_in, pic.spot(LEFT), 0.6)?;
        relabel(&kit, stage, &pic, &mut labels, &[LEFT], &rounds[1], 0.3)?;
        line.wait_for(stage, "The right averages")?;
        slide_in(stage, &right_in, pic.spot(RIGHT), 0.6)?;
        relabel(&kit, stage, &pic, &mut labels, &[RIGHT], &rounds[1], 0.3)?;
        let mut table = vec![first_row];
        table.push(round_row(&kit, stage, 1, &rounds[1])?);
        fade_in_all(stage, &table[1], 0.3)?;

        // 03_4: round two, both junctions together from round one; rounds
        // three and four without tokens
        wait_until(stage, rule_read)?;
        let line = say(&kit, stage, "03_4")?;
        let left_in = tokens(
            &kit,
            stage,
            &pic,
            LEFT,
            &value_texts(&facts::far_values(&rounds[1], LEFT).map_err(fail)?),
        )?;
        let right_in = tokens(
            &kit,
            stage,
            &pic,
            RIGHT,
            &value_texts(&facts::far_values(&rounds[1], RIGHT).map_err(fail)?),
        )?;
        let mut both = left_in.clone();
        both.extend(&right_in);
        swap(stage, &[rule], &both, CARD_OUT)?;
        let mut moves: Vec<(Mob, Vec3)> = left_in.iter().map(|&m| (m, pic.spot(LEFT))).collect();
        moves.extend(right_in.iter().map(|&m| (m, pic.spot(RIGHT))));
        move_all(stage, &moves, 0.6)?;
        fade_out_all(stage, &both, 0.2)?;
        relabel(&kit, stage, &pic, &mut labels, &[LEFT, RIGHT], &rounds[2], 0.3)?;
        table.push(round_row(&kit, stage, 2, &rounds[2])?);
        fade_in_all(stage, &table[2], 0.3)?;
        line.wait_for(stage, "Then nineteen")?;
        for (n, values) in rounds.iter().enumerate().take(ROUNDS).skip(3) {
            relabel(&kit, stage, &pic, &mut labels, &[LEFT, RIGHT], values, QUICK)?;
            let row = round_row(&kit, stage, n, values)?;
            fade_in_all(stage, &row, QUICK)?;
            table.push(row);
        }

        // 03_5: the bigger gap, a quarter of the last every round, its card
        // in with the column; the one wire at each junction that leads to an
        // unknown
        let line = say(&kit, stage, "03_5")?;
        let mut gap_column =
            vec![kit.text(stage, GAP_HEADER, HEADER_SIZE, Palette::dim(), v(COLUMN_X[3], HEADER_Y))?];
        for (n, &g) in gaps.iter().enumerate().take(ROUNDS) {
            gap_column.push(cell(&kit, stage, &small_tex(g), 3, row_y(n))?);
        }
        fade_in_lagged(stage, &gap_column, GAP_COLUMN_IN, 0.3)?;
        let bound_text = gap_card().map_err(fail)?;
        let bound = precisely(&kit, stage, &bound_text)?;
        fade_in_all(stage, &[bound], CARD_IN)?;
        let bound_read = stage.scene().time().to_f64() + read_s(&bound_text);
        line.wait_for(stage, "only one wire")?;
        let unknown_wire = pic.strand_between(LEFT, RIGHT)?.mob;
        flash_all(stage, &[unknown_wire], 0.6)?;

        // 03_6: the prediction; the card stays while the question is asked
        // and goes as it ends, so the table holds rounds 0 to 4 and nothing
        // else through the countdown
        let question = say(&kit, stage, "03_6")?;
        wait_until(stage, bound_read.max(question.start + question.len - CARD_OUT))?;
        fade_out_all(stage, &[bound], CARD_OUT)?;
        let countdown = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 03_7: never exactly, and why (its card stays over the limit, 03_8);
        // but round five is within a thousandth
        let line_never = say(&kit, stage, "03_7")?;
        let never_text = never_card().map_err(fail)?;
        let never = precisely(&kit, stage, &never_text)?;
        swap(stage, &[countdown], &[never], CARD_OUT)?;
        let never_read = stage.scene().time().to_f64() + read_s(&never_text);
        line_never.wait_for(stage, "But after five rounds")?;
        let mut last_row = round_row(&kit, stage, ROUNDS, &rounds[ROUNDS])?;
        let last_gap = cell(&kit, stage, &small_tex(gaps[ROUNDS]), 3, row_y(ROUNDS))?;
        last_row.push(last_gap);
        fade_in_all(stage, &last_row, 0.3)?;
        relabel(&kit, stage, &pic, &mut labels, &[LEFT, RIGHT], &rounds[ROUNDS], QUICK)?;
        flash_all(stage, &[last_gap], 0.6)?;

        // 03_8: where the rounds go, set apart from them
        let line_limit = say(&kit, stage, "03_8")?;
        line_limit.wait_for(stage, "three fifths and two fifths")?;
        let limit_row = vec![
            kit.text(stage, LIMIT, HEADER_SIZE, Palette::dim(), v(COLUMN_X[0], LIMIT_Y))?,
            cell(&kit, stage, &small_tex(volts[LEFT]), 1, LIMIT_Y)?,
            cell(&kit, stage, &small_tex(volts[RIGHT]), 2, LIMIT_Y)?,
            cell(&kit, stage, &small_tex(Q::ZERO), 3, LIMIT_Y)?,
        ];
        fade_in_all(stage, &limit_row, 0.4)?;
        flash_all(stage, &limit_row[1..3], 0.6)?;

        // 03_9: the ends averaged too. The battery-held table (its rounds and
        // its limit) goes as the counterfactual's card comes, so no number
        // left on screen belongs to the battery-free network but the card's;
        // then the battery and the labels go, and the four colours run
        // together round by round
        let line_ends = say(&kit, stage, "03_9")?;
        wait_until(stage, never_read)?;
        let lost_text = drift_card().map_err(fail)?;
        let lost = precisely(&kit, stage, &lost_text)?;
        let mut going = vec![never];
        going.extend(
            header
                .iter()
                .chain(table.iter().flatten())
                .chain(&gap_column)
                .chain(&last_row)
                .chain(&limit_row),
        );
        fade_out_all(stage, &going, CARD_OUT / 2.0)?;
        fade_in_all(stage, &[lost], CARD_OUT / 2.0)?;
        let lost_read = stage.scene().time().to_f64() + read_s(&lost_text);
        line_ends.wait_for(stage, "Average them too")?;
        let mut leaving = cell_mobs;
        leaving.extend(labels);
        fade_out_all(stage, &leaving, 0.4)?;
        let (free, _) = facts::drift().map_err(fail)?;
        for values in &free {
            let colours: Vec<(Mob, Srgb)> = (0..net.size())
                .map(|x| (pic.junctions[x], palette::value(values[x])))
                .collect();
            recolor(stage, &colours, DRIFT)?;
        }
        wait_until(stage, line_ends.start + line_ends.len)?;
        pause(stage, END_BEAT)?;
        wait_until(stage, lost_read)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;

    /// Every phrase the scene waits for is in its line of script.tsv, once.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("03_1", "Guess zero"),
            ("03_2", "using the values"),
            ("03_3", "The left averages"),
            ("03_3", "The right averages"),
            ("03_4", "Then nineteen"),
            ("03_5", "only one wire"),
            ("03_7", "But after five rounds"),
            ("03_8", "three fifths and two fifths"),
            ("03_9", "Average them too"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert_eq!(text.matches(phrase).count(), 1, "{id} must say {phrase:?} once: {text}");
        }
    }

    /// The table's rows are the facts' rounds, and its gaps the facts' gaps.
    #[test]
    fn the_table_shows_the_real_rounds() {
        let rounds = facts::rounds(ROUNDS).unwrap();
        let left: Vec<String> = rounds.iter().map(|r| small_tex(r[LEFT])).collect();
        assert_eq!(
            left,
            [
                "0",
                r"\tfrac{1}{2}",
                r"\tfrac{9}{16}",
                r"\tfrac{19}{32}",
                r"\tfrac{153}{256}",
                r"\tfrac{307}{512}"
            ]
        );
        let right: Vec<String> = rounds.iter().map(|r| small_tex(r[RIGHT])).collect();
        assert_eq!(
            right,
            [
                "0",
                r"\tfrac{1}{4}",
                r"\tfrac{3}{8}",
                r"\tfrac{25}{64}",
                r"\tfrac{51}{128}",
                r"\tfrac{409}{1024}"
            ]
        );
        let gaps: Vec<String> = facts::gaps(&rounds).unwrap().iter().map(|&g| small_tex(g)).collect();
        assert_eq!(gaps[ROUNDS], r"\tfrac{3}{5120}");
        // round one's tokens read round zero: the right's token from the
        // left still says 0 after the left has become 1/2
        let right_in = facts::far_values(&rounds[0], RIGHT).unwrap();
        assert_eq!(right_in, [Q::ONE, Q::ZERO, Q::ZERO, Q::ZERO]);
        assert_eq!(rounds[1][LEFT], Q::new(1, 2));
    }

    /// The exact versions say what the facts say, and fit a lower-third of
    /// three lines.
    #[test]
    fn the_exact_versions_are_built_from_the_facts_and_fit() {
        let jacobi = jacobi_card().unwrap();
        assert!(
            jacobi.starts_with("every middle junction updates at once from the previous round (Jacobi's method)"),
            "{jacobi}"
        );
        assert!(
            jacobi.contains("(Gauss-Seidel), the gap shrinks to 1/16 a round from round 2 on"),
            "{jacobi}"
        );
        let gap = gap_card().unwrap();
        assert!(
            gap.starts_with(
                "3 of each junction's 4 far ends never change, so its new gap is 1/4 of the other's old one"
            ),
            "{gap}"
        );
        assert!(
            gap.contains("exactly 1/4 of the last, 3/5, 3/20, 3/80, 3/320 and 3/1280 by round 4"),
            "{gap}"
        );
        // the card comes before the prediction: no round-5 number in it
        assert!(!gap.contains("3/5120") && !gap.contains("307/512"), "{gap}");
        let never = never_card().unwrap();
        assert!(
            never.contains("the other junction's previous value plus a whole number, divided by 4"),
            "{never}"
        );
        assert!(
            never.contains("from the guess of 0 every value is over a power of 2"),
            "{never}"
        );
        // its examples are rows already shown when it comes (round 5's row
        // comes later in the line)
        assert!(never.contains("like 9/16 and 153/256, and never 3/5"), "{never}");
        assert!(!never.contains("307/512") && !never.contains("409/1024"), "{never}");
        let drift = drift_card().unwrap();
        assert_eq!(
            drift,
            "with the top and bottom averaged too, 1, 3/5, 2/5, 0 drift to 1/2, their mean weighted by wires 3, 4, \
             4, 3."
        );
        // the drift the picture plays: from the solved values, within a
        // thousandth of 1/2 after 6 rounds, not before
        let (free, settle) = facts::drift().unwrap();
        assert_eq!((free.len() - 1, settle), (6, Q::new(1, 2)));
        let off = |values: &[Q]| values.iter().map(|&x| (x - settle).max(settle - x)).max().unwrap();
        assert!(off(&free[6]) < facts::tolerance() && off(&free[5]) >= facts::tolerance());
        for text in [jacobi, gap, never, drift] {
            let lines = explainer_kit::animatic::wrap(&text, 84, 10);
            assert!(lines.len() <= 3, "{} lines: {text}", lines.len());
        }
    }

    /// Every card can be read at fifteen characters a second in the time it
    /// stays up, by the scratch voice's timing of the lines it stays over
    /// (with the thought-following gaps this video uses), so the scene's
    /// guards add no time to the draft; with a real voice they hold a card
    /// longer when the lines run short.
    #[test]
    fn every_card_stays_up_long_enough_to_read() {
        let script = explainer_kit::Script::parse(crate::SCRIPT)
            .unwrap()
            .with_line_gaps(crate::EXPLAINER.line_gaps);
        let voice = explainer_kit::Narrator::estimate(&script);
        let len = |id: &str| voice.length(id).unwrap_or_else(|| panic!("no line {id}"));
        let gap = |a: &str, b: &str| voice.line_gap(a, b);
        // what is left of a line once `phrase` begins, read evenly
        let after = |id: &str, phrase: &str| {
            let text = script.text(id).unwrap();
            let at = text[..text.find(phrase).unwrap()].chars().count() as f64;
            len(id) * (1.0 - at / text.chars().count() as f64)
        };
        let windows = [
            // in at "using the values", up over round one, out as 03_4 begins
            (
                jacobi_card().unwrap(),
                after("03_2", "using the values") - CARD_IN + gap("03_2", "03_3") + len("03_3") + gap("03_3", "03_4"),
            ),
            // in after the gap column, out as the question ends
            (
                gap_card().unwrap(),
                len("03_5") - GAP_COLUMN_IN - CARD_IN + gap("03_5", "03_6") + len("03_6") - CARD_OUT,
            ),
            // in with the answer, up over the limit, out as 03_9 begins
            (
                never_card().unwrap(),
                len("03_7") - CARD_OUT + gap("03_7", "03_8") + len("03_8") + gap("03_8", "03_9"),
            ),
            // in as 03_9 begins, up to the beat after it
            (drift_card().unwrap(), len("03_9") - CARD_OUT + END_BEAT),
        ];
        for (text, up) in windows {
            let need = read_s(&text);
            assert!(need <= up, "{need:.2} s to read, {up:.2} s up: {text}");
        }
    }

    /// The table stays right of the battery and inside the title-safe area,
    /// its limit row apart from round five's and above a three-line
    /// lower-third, and the countdown clears the rows above it.
    #[test]
    fn the_table_keeps_clear_of_the_picture_and_the_card() {
        use crate::chapters::draw::{BATTERY_X, THINK_R};
        const { assert!(COLUMN_X[0] - 0.45 > PICTURE_AT[0] + BATTERY_X + 0.3) };
        const { assert!(COLUMN_X[3] + 0.7 < FRAME_X_RADIUS - 0.25) };
        const { assert!(HEADER_Y + 0.2 < FRAME_Y_RADIUS - 0.15) };
        const { assert!(LIMIT_Y + 0.3 < ROW0_Y - ROW_STEP * 5.0 - 0.25) };
        const { assert!(LIMIT_Y - 0.3 > -FRAME_Y_RADIUS + 1.58) };
        // the countdown under round four's row, its note above the frame's
        // safe bottom
        const { assert!(THINK_AT[1] + THINK_R < ROW0_Y - ROW_STEP * 4.0 - 0.25) };
        const { assert!(THINK_AT[1] - THINK_R - 0.45 > -FRAME_Y_RADIUS + 0.15) };
        assert!((row_y(ROUNDS) - (ROW0_Y - ROW_STEP * 5.0)).abs() < 1e-12);
    }
}
