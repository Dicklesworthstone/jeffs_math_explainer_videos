//! Chapter 06, a group that asks too much: is checking each column on its
//! own always enough? Rows and columns 4, 4, 1, 1, 1 (06_1); no count is zero
//! or five, so no row or column forces a square, and each column of four has
//! five rows with water (06_2, a prediction: does any picture fit?); no: the
//! two columns of four together ask for eight (06_3), and a row gives that
//! pair at most min(its count, 2): two, two, one, one and one, seven (06_4);
//! no picture fits, and those single-row and single-column checks didn't
//! catch it (06_5); the water agrees: it stops at ten of eleven and cannot
//! reach exactly those two columns (06_6); Gale's count for any group of
//! columns (06_7), his table, failing at the size of two (06_8), and why the
//! hungriest group of each size is the one to check (06_9).
//!
//! The shadows are `model::STALL_ROWS` and `STALL_COLS`; the pair is
//! `model::bottleneck`; the water is `facts::stall_flow` (the paths
//! `model::max_flow` sent, replayed by `model::replay`), its stop
//! `Flow::cut`; the table `model::gale_table`.

use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{Board, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, pause, say, swap, think};
use super::stops::{
    ASK_Y, ASKS_DY, GIVE_RULE, GIVE_Y, K_Y, RULE_Y, SORTED_Y, columns_outline, counters, reach_strokes, recolor,
    table_labels, table_line,
};
use super::water::{AT, CELL, TEXT_X, THINK_AT, THROUGH_Y, Water, drain_label, send_label, send_quick, tap_at, through_text};
use crate::facts::{self, OTHER_PAIRS};
use crate::model;
use crate::palette;

pub struct AGroupThatAsksTooMuch {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const NO_PICTURE: &str = "no picture fits";

/// The words on the right, top to bottom (06_2 to 06_5).
const WORDS_Y: [f64; 4] = [2.1, 1.5, 0.9, 0.3];
/// The other pairs' words (06_9), under the table.
const PAIRS_Y: [f64; 2] = [-1.2, -1.7];
/// How far right of the grid a row's share for the pair stands (06_4).
const SHARE_DX: f64 = 0.35;

/// "every count is between 1 and 4" (06_2).
fn between_text(least: usize, most: usize) -> String {
    format!("every count is between {least} and {most}")
}

/// "each column of 4: 5 rows with water" (06_2).
fn room_text(column: usize, rows: usize) -> String {
    format!("each column of {column}: {rows} rows with water")
}

fn ask_text(asks: usize) -> String {
    format!("ask {asks}")
}

/// "2 + 2 + 1 + 1 + 1 = 7, they ask 8" (06_4).
fn shares_text(shares: &[usize], asks: usize) -> String {
    let parts: Vec<String> = shares.iter().map(ToString::to_string).collect();
    format!("{} = {}, they ask {asks}", parts.join(" + "), shares.iter().sum::<usize>())
}

/// A column's ordinal, as the words name it (06_9).
fn ordinal(c: usize) -> &'static str {
    ["first", "second", "third", "fourth", "fifth"].get(c).copied().unwrap_or("next")
}

/// "first and third: ask 5, rows can give 7" (06_9).
fn pair_text(pair: [usize; 2], asks: usize, gives: usize) -> String {
    format!(
        "{} and {}: ask {asks}, rows can give {gives}",
        ordinal(pair[0]),
        ordinal(pair[1])
    )
}

impl SceneConstruct for AGroupThatAsksTooMuch {
    fn name(&self) -> &str {
        "06_a_group_that_asks_too_much"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let s = facts::stall();
        let total = s.row_total();
        if total != s.col_total() {
            return Err(fail("the stall shadows' totals differ".into()));
        }
        let (rows, cols) = (s.rows.len(), s.cols.len());
        let solve = model::line_solve(&s);
        if solve.settled() != 0 || solve.contradiction.is_some() {
            return Err(fail("reasoning one line at a time settles a square of the stall shadows".into()));
        }
        let least = s.rows.iter().chain(&s.cols).copied().min().unwrap_or(0);
        let most = s.rows.iter().chain(&s.cols).copied().max().unwrap_or(0);
        if least == 0 || most >= rows.min(cols) {
            return Err(fail(format!("a count of the stall shadows is 0 or full: {least} to {most}")));
        }
        let pair = model::bottleneck(&s).ok_or_else(|| fail("the stall shadows have no bottleneck".into()))?;
        let &[p0, p1] = pair.as_slice() else {
            return Err(fail(format!("the bottleneck is {pair:?}, not a pair")));
        };
        if p1 != p0 + 1 || s.cols[p0] != s.cols[p1] {
            return Err(fail(format!("the bottleneck {pair:?} is not two neighbouring equal columns")));
        }
        let column = s.cols[p0];
        let with_water = s.rows.iter().filter(|&&r| r > 0).count();
        if column > with_water {
            return Err(fail("a column of the pair alone asks more than the rows with water".into()));
        }
        let asks: usize = pair.iter().map(|&j| s.cols[j]).sum();
        let shares: Vec<usize> = s.rows.iter().map(|&r| r.min(pair.len())).collect();
        let gives = model::can_give(&s.rows, pair.len());
        if gives >= asks || shares.iter().sum::<usize>() != gives {
            return Err(fail(format!("the pair asks {asks} and the rows can give {gives}")));
        }
        let (flow, states) = facts::stall_flow().map_err(fail)?;
        let cut = flow.cut(&s);
        if cut.cut_off != pair || cut.value() != flow.value || flow.value >= total {
            return Err(fail(format!("the stall water stops at {cut:?}, not cutting off the pair")));
        }
        let board = Board::new(AT, CELL, rows, cols);

        // 06_1: the plumbing, empty; rows and columns 4, 4, 1, 1, 1
        let line = say(&kit, stage, "06_1")?;
        let grid = board.lines(stage);
        fade_in_all(stage, &grid, 0.5)?;
        line.wait_for(stage, "rows of four")?;
        let taps = counters(&kit, stage, &s.rows, |r| tap_at(&board, r))?;
        let to_send = send_label(&kit, stage, &board)?;
        fade_in_lagged(stage, &taps, 0.8, 0.3)?;
        fade_in_all(stage, &[to_send], 0.3)?;
        line.wait_for(stage, "columns the same")?;
        let drains = counters(&kit, stage, &s.cols, |c| board.col_count_at(c))?;
        let to_drain = drain_label(&kit, stage, &board)?;
        let counter = kit.text_left(
            stage,
            &through_text(0, total),
            26.0,
            Palette::ink(),
            v(TEXT_X, THROUGH_Y),
        )?;
        fade_in_lagged(stage, &drains, 0.8, 0.3)?;
        fade_in_all(stage, &[to_drain, counter], 0.3)?;

        // 06_2: no count is 0 or full, so no line settles a square; each
        // column of four has five rows with water; the question
        let line = say(&kit, stage, "06_2")?;
        line.wait_for(stage, "No count")?;
        let between = kit.text_left(
            stage,
            &between_text(least, most),
            24.0,
            Palette::ink(),
            v(TEXT_X, WORDS_Y[0]),
        )?;
        fade_in_all(stage, &[between], 0.3)?;
        line.wait_for(stage, "each column of four")?;
        let room = kit.text_left(
            stage,
            &room_text(column, with_water),
            24.0,
            palette::matches(),
            v(TEXT_X, WORDS_Y[1]),
        )?;
        flash_all(stage, &[drains[p0], drains[p1]], 0.4)?;
        fade_in_all(stage, &[room], 0.3)?;
        let q = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 06_3: no: the two columns of four together ask for eight
        let line = say(&kit, stage, "06_3")?;
        fade_out_all(stage, &[q], 0.3)?;
        line.wait_for(stage, "Take the two columns")?;
        let outline = columns_outline(stage, &board, p0, p1, palette::impossible());
        let middle = (board.at(0, p0)[0] + board.at(0, p1)[0]) / 2.0;
        let ask = kit.text(
            stage,
            &ask_text(asks),
            22.0,
            palette::impossible(),
            v(middle, board.col_count_at(p0)[1] + ASKS_DY),
        )?;
        fade_in_all(stage, &[outline, ask], 0.4)?;

        // 06_4: a row of four gives the pair at most two, a row of one at
        // most one: two, two, one, one and one make seven
        let line = say(&kit, stage, "06_4")?;
        line.wait_for(stage, "A row of four")?;
        let big: Vec<usize> = (0..rows).filter(|&r| s.rows[r] >= pair.len()).collect();
        let small: Vec<usize> = (0..rows).filter(|&r| s.rows[r] < pair.len()).collect();
        let mut rings = Vec::with_capacity(big.len() * pair.len());
        for &r in &big {
            for &c in &pair {
                rings.push(board.ring(stage, r, c, Palette::ink()));
            }
        }
        let share_at = |r: usize| v(board.right() + SHARE_DX, board.at(r, 0)[1]);
        let mut share_marks = Vec::with_capacity(rows);
        let big_marks = big
            .iter()
            .map(|&r| kit.text(stage, &shares[r].to_string(), 24.0, Palette::ink(), share_at(r)))
            .collect::<fmn::Result<Vec<Mob>>>()?;
        fade_in_all(stage, &rings, 0.3)?;
        fade_in_lagged(stage, &big_marks, 0.6, 0.3)?;
        share_marks.extend(big_marks);
        line.wait_for(stage, "A row of one")?;
        let small_marks = small
            .iter()
            .map(|&r| kit.text(stage, &shares[r].to_string(), 24.0, Palette::ink(), share_at(r)))
            .collect::<fmn::Result<Vec<Mob>>>()?;
        fade_in_lagged(stage, &small_marks, 0.6, 0.3)?;
        share_marks.extend(small_marks);
        line.wait_for(stage, "make seven")?;
        let sum = kit.text_left(
            stage,
            &shares_text(&shares, asks),
            24.0,
            Palette::ink(),
            v(TEXT_X, WORDS_Y[2]),
        )?;
        fade_in_all(stage, &[sum], 0.4)?;

        // 06_5: eight asked, seven possible: no picture fits, and neither
        // check the viewer ran caught it
        let line = say(&kit, stage, "06_5")?;
        line.wait_for(stage, "No picture fits")?;
        let verdict = kit.text_left(stage, NO_PICTURE, 28.0, palette::impossible(), v(TEXT_X, WORDS_Y[3]))?;
        fade_in_all(stage, &[verdict], 0.4)?;
        flash_all(stage, &[ask, sum], 0.6)?;

        // 06_6: the water agrees: the method's real units, ten of eleven;
        // the columns it cannot reach are exactly the pair
        let line = say(&kit, stage, "06_6")?;
        let mut clearing = rings;
        clearing.extend(share_marks);
        clearing.extend([between, room, sum, verdict]);
        fade_out_all(stage, &clearing, 0.3)?;
        let mut water = Water::new(board, taps, drains, &s);
        water.through = Some(counter);
        for (path, state) in flow.paths.iter().zip(&states) {
            send_quick(&kit, stage, &mut water, &s, path, state)?;
        }
        line.wait_for(stage, "can't reach")?;
        let reach = reach_strokes(stage, &board, &cut)?;
        fade_in_lagged(stage, &reach, 0.8, 0.2)?;
        flash_all(stage, &[outline, ask], 0.6)?;
        line.wait_to_end(stage)?;
        pause(stage, 1.0)?;

        // 06_7: the same count for any group of columns: a row gives k
        // columns at most min(its count, k); what the rows can give, k = 1 to 5
        let line = say(&kit, stage, "06_7")?;
        fade_out_all(stage, &reach, 0.4)?;
        let table = model::gale_table(&s);
        let sizes: Vec<usize> = (1..=table.len()).collect();
        let gives_line: Vec<usize> = table.iter().map(|&(_, g)| g).collect();
        let k_line = table_line(&kit, stage, &sizes, K_Y, Palette::dim())?;
        let labels = table_labels(&kit, stage)?;
        let mut give_line = table_line(&kit, stage, &gives_line, GIVE_Y, Palette::ink())?;
        let mut shown = k_line.clone();
        shown.extend([labels[0], labels[3]]);
        fade_in_all(stage, &shown, 0.4)?;
        line.wait_for(stage, "Each row gives")?;
        let rule = kit.text_left(stage, GIVE_RULE, 22.0, Palette::ink(), v(TEXT_X, RULE_Y))?;
        fade_in_all(stage, &[rule], 0.3)?;
        fade_in_lagged(stage, &give_line, 0.9, 0.3)?;

        // 06_8: the k largest columns, what they ask; the size of two fails;
        // the hungriest group of each size is enough
        let line = say(&kit, stage, "06_8")?;
        let sorted = facts::largest_first(&s.cols);
        let asks_line: Vec<usize> = table.iter().map(|&(a, _)| a).collect();
        let sorted_line = table_line(&kit, stage, &sorted, SORTED_Y, Palette::ink())?;
        let mut ask_line = table_line(&kit, stage, &asks_line, ASK_Y, Palette::ink())?;
        let mut arriving = sorted_line.clone();
        arriving.extend(&ask_line);
        arriving.extend([labels[1], labels[2]]);
        fade_in_lagged(stage, &arriving, 0.9, 0.2)?;
        line.wait_for(stage, "no picture fits")?;
        let failing: Vec<usize> = (0..table.len()).filter(|&i| table[i].0 > table[i].1).collect();
        if failing != [pair.len() - 1] {
            return Err(fail(format!("Gale's table fails at {failing:?}, not only at the pair's size")));
        }
        for &i in &failing {
            let (a, g) = table[i];
            ask_line[i] = recolor(&kit, stage, ask_line[i], a, i + 1, ASK_Y, palette::impossible())?;
            give_line[i] = recolor(&kit, stage, give_line[i], g, i + 1, GIVE_Y, palette::impossible())?;
        }
        line.wait_for(stage, "the hungriest")?;
        flash_all(stage, &sorted_line, 0.6)?;

        // 06_9: for a given size the rows can give every group the same, so
        // the hungriest is the one to check: two other pairs, lit in turn
        let line = say(&kit, stage, "06_9")?;
        line.wait_for(stage, "every group the same")?;
        let mut lit: Vec<Mob> = Vec::new();
        for (other, y) in OTHER_PAIRS.iter().zip(PAIRS_Y) {
            let other_asks: usize = other.iter().map(|&j| s.cols[j]).sum();
            let other_gives = model::can_give(&s.rows, other.len());
            if other_gives != gives || other_asks >= asks {
                return Err(fail(format!(
                    "the pair {other:?} asks {other_asks} and is given {other_gives}, not less than the \
                     hungriest and the same"
                )));
            }
            let marks: Vec<Mob> = other
                .iter()
                .map(|&c| columns_outline(stage, &board, c, c, palette::beam()))
                .collect();
            let words = kit.text_left(
                stage,
                &pair_text(*other, other_asks, other_gives),
                22.0,
                Palette::ink(),
                v(TEXT_X, y),
            )?;
            let mut coming = marks.clone();
            coming.push(words);
            swap(stage, &lit, &coming, 0.3)?;
            pause(stage, 0.8)?;
            lit = marks;
        }
        line.wait_for(stage, "the hungriest")?;
        fade_out_all(stage, &lit, 0.2)?;
        flash_all(stage, &sorted_line, 0.6)?;
        line.wait_to_end(stage)?;
        pause(stage, 1.2)?;

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
            ("06_1", "rows of four"),
            ("06_1", "columns the same"),
            ("06_2", "No count"),
            ("06_2", "each column of four"),
            ("06_3", "Take the two columns"),
            ("06_4", "A row of four"),
            ("06_4", "A row of one"),
            ("06_4", "make seven"),
            ("06_5", "No picture fits"),
            ("06_6", "can't reach"),
            ("06_7", "Each row gives"),
            ("06_8", "no picture fits"),
            ("06_8", "the hungriest"),
            ("06_9", "every group the same"),
            ("06_9", "the hungriest"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The words on screen are the facts'.
    #[test]
    fn the_words_on_screen_are_built_from_the_facts() {
        let f = facts::facts().unwrap();
        let get = |key: &str| -> usize { f.get(key).unwrap().parse().unwrap() };
        assert_eq!(f.get("DT-F15.counts"), Some("1,4"));
        assert_eq!(between_text(1, 4), "every count is between 1 and 4");
        assert_eq!(
            room_text(get("DT-F15.largest"), get("DT-F15.rows_with_water")),
            "each column of 4: 5 rows with water"
        );
        assert_eq!(ask_text(get("DT-F15.pair.asks")), "ask 8");
        assert_eq!(f.get("DT-F15.pair.by_row"), Some("2,2,1,1,1"));
        assert_eq!(
            shares_text(&[2, 2, 1, 1, 1], get("DT-F15.pair.asks")),
            "2 + 2 + 1 + 1 + 1 = 7, they ask 8"
        );
        assert_eq!(f.get("DT-F15.other_pairs"), Some("0,2;2,3"));
        assert_eq!(f.get("DT-F15.other_pairs.asks"), Some("5,2"));
        assert_eq!(pair_text([0, 2], 5, 7), "first and third: ask 5, rows can give 7");
        assert_eq!(pair_text([2, 3], 2, 7), "third and fourth: ask 2, rows can give 7");
    }

    /// The chapter's layout keeps its pieces apart and inside the
    /// title-safe area (about 0.0045 units a point a character).
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        let board = Board::new(AT, CELL, 5, 5);
        let safe_x = FRAME_X_RADIUS - 0.25;
        let safe_y = FRAME_Y_RADIUS - 0.25;
        let ends = |x: f64, text: &str, size: f64| x + text.chars().count() as f64 * size * 0.0045;
        // the words on the right end inside and clear the think ring's
        // column; the shares stand between the grid and the words
        assert!(ends(TEXT_X, &room_text(4, 5), 24.0) < safe_x);
        assert!(ends(TEXT_X, &shares_text(&[2, 2, 1, 1, 1], 8), 24.0) < safe_x);
        assert!(ends(TEXT_X, &pair_text([2, 3], 2, 7), 22.0) < safe_x);
        assert!(WORDS_Y[0] + 0.2 < safe_y);
        assert!(board.right() + SHARE_DX - 0.15 > board.right() + 0.1);
        assert!(board.right() + SHARE_DX + 0.15 < TEXT_X - 0.1);
        // the words stay apart from the units-through counter and from the
        // table's lines above them
        assert!(WORDS_Y[3] - 0.15 > THROUGH_Y + 0.2);
        assert!(PAIRS_Y[0] + 0.15 < THROUGH_Y - 0.2);
        assert!(PAIRS_Y[1] - 0.15 > -safe_y);
        assert!(RULE_Y - 0.15 > THROUGH_Y + 0.2);
        // the pair's ask, under its drains, inside
        assert!(board.col_count_at(0)[1] + ASKS_DY - 0.15 > -safe_y);
    }
}
