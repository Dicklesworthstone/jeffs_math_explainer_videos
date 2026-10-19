//! Chapter 03, where the water stops: the face's row counts with columns 5,
//! 1, 1, 1, 1 (03_1); the method's real units go through, eight of nine, and
//! stop (03_2, a prediction: could taking water back get the ninth
//! through?); no: the first column needs five, each row sends it at most
//! one, and only four rows have water (03_3); taking back only moves water
//! between pipes, and the cut where the water stops lets four units into the
//! first column and four out of the other drains, eight in all (03_4); the
//! same count for any group of columns, min(count, k) a row (03_5); Gale's
//! table, the k largest columns against what the rows can give (03_6);
//! complete with equal totals, because where the water stops is always such
//! a group (03_7); the face's shadows pass, and the test says a picture
//! fits, not which and not whether only one (03_8).
//!
//! Every square, counter, route and number comes from `facts::no_fit_flow`
//! (the paths `model::max_flow` sent, replayed by `model::replay`), its
//! `Flow::cut` and `model::gale_table`.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    Board, COUNT_SIZE, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, path_line, pause, say, segment,
    swap, think, wait_until,
};
use super::water::{
    AT, CELL, TAP_GAP, TEXT_X, THINK_AT, THROUGH_Y, TO_DRAIN, TO_SEND, Water, send_quick, tap_at, through_text,
};
use crate::facts::{self, GALE_YEAR, RYSER_YEAR};
use crate::model;
use crate::palette;

pub struct WhereTheWaterStops {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const STILL_REACHES: &str = "where the water can still go, taking back included";
const K_LABEL: &str = "k";
const SORTED_LABEL: &str = "largest columns first";
const ASK_LABEL: &str = "the k largest ask";
const GIVE_LABEL: &str = "rows can give";
const GIVE_RULE: &str = "a row gives k columns at most min(its count, k)";
const EQUAL: &str = "equal totals";
const NOT_SAID: &str = "it says one fits, not which, not whether only one";

/// Gale's table, on the right: the k labels, then three rows of numbers.
const TABLE_X0: f64 = 2.9;
const TABLE_PITCH: f64 = 0.7;
const K_Y: f64 = 2.4;
const SORTED_Y: f64 = 1.85;
const ASK_Y: f64 = 1.3;
const GIVE_Y: f64 = 0.75;
const RULE_Y: f64 = 0.15;
const TABLE_SIZE: f64 = 28.0;
/// 03_4's words, top right, before the table.
const CUT_Y: [f64; 3] = [2.2, 1.6, 1.0];
/// 03_8's words, under the table.
const CLOSING_Y: [f64; 2] = [-0.45, -1.0];
/// Where a column's ask stands under its drain (03_3).
const ASKS_DY: f64 = -0.45;
/// How far a reached row's stroke starts inside the reached columns, clear
/// of the cut-off column's outline (03_4).
const REACH_GAP: f64 = 0.1;

/// The cut's count in words (03_4), from its parts.
fn into_first_text(pipes: usize) -> String {
    format!("{pipes} pipes into the first column")
}

fn other_drains_text(drains: usize) -> String {
    format!("{drains} out of the other drains")
}

fn cut_sum_text(pipes: usize, drains: usize, total: usize) -> String {
    format!("{pipes} + {drains} = {total}, all that can pass")
}

fn fits_text(count: usize) -> String {
    format!("{count} pictures fit these shadows")
}

/// The exact version of 03_6 and 03_7, kept on screen until it can be read
/// (`read_s`). "The k largest columns" is the sorted condition: the first k
/// of the column counts sorted largest first, the table's second line.
fn gale_claim() -> String {
    format!(
        "a picture fits exactly when row and column totals match and, for every k, the k largest columns \
         ask for at most the sum over rows of min(row count, k) (Gale, {GALE_YEAR}; Ryser, {RYSER_YEAR})"
    )
}

/// Seconds a card needs on screen, from its fade-in done, to be read:
/// fifteen characters a second, a quick reader (the corpus's provisional
/// scene convention, not a listening measure; the video can be paused).
/// Chapter 04's card and closing words keep it too.
pub(crate) fn read_s(text: &str) -> f64 {
    text.chars().count() as f64 / 15.0
}

/// Where the table's entry for size `k` (from 1) stands, on line `y`.
fn cell_at(k: usize, y: f64) -> Vec3 {
    v(TABLE_X0 + TABLE_PITCH * (k - 1) as f64, y)
}

/// One line of the table: `values`, the `k`th at its column. Detached.
fn table_line(kit: &Kit, stage: &mut Stage<'_>, values: &[usize], y: f64, color: Srgb) -> fmn::Result<Vec<Mob>> {
    values
        .iter()
        .zip(1..)
        .map(|(n, k)| kit.text(stage, &n.to_string(), TABLE_SIZE, color, cell_at(k, y)))
        .collect()
}

/// Where a reached row's stroke runs, left to right: over the reached
/// columns `cols` only, from just inside the first one's left edge to just
/// past the last one's right edge. `None` when no column is reached, or
/// when the span would cross a cut-off column (one of `cut_off` between two
/// reached columns).
fn row_reach(board: &Board, cols: &[usize], cut_off: &[usize]) -> Option<(f64, f64)> {
    let (&lo, &hi) = (cols.iter().min()?, cols.iter().max()?);
    if cut_off.iter().any(|c| (lo..=hi).contains(c)) {
        return None;
    }
    let half = board.cell / 2.0;
    Some((board.at(0, lo)[0] - half + REACH_GAP, board.at(0, hi)[0] + half + 0.1))
}

/// Redraw `old`, the table entry for size `k` on line `y`, as `n` in `color`.
fn recolor(kit: &Kit, stage: &mut Stage<'_>, old: Mob, n: usize, k: usize, y: f64, color: Srgb) -> fmn::Result<Mob> {
    let new = kit.text(stage, &n.to_string(), TABLE_SIZE, color, cell_at(k, y))?;
    swap(stage, &[old], &[new], 0.3)?;
    Ok(new)
}

/// An outline around column `c` of `board`, top to bottom. Detached.
fn column_outline(stage: &mut Stage<'_>, board: &Board, c: usize, color: Srgb) -> Mob {
    let x = board.at(0, c)[0];
    let (l, r) = (x - board.cell / 2.0, x + board.cell / 2.0);
    let corners = [
        v(l, board.top()),
        v(r, board.top()),
        v(r, board.bottom()),
        v(l, board.bottom()),
        v(l, board.top()),
    ];
    path_line(stage, &corners, color, 5.0)
}

/// Counters for `values` at `at(i)`, in ink. Detached.
fn counters(kit: &Kit, stage: &mut Stage<'_>, values: &[usize], at: impl Fn(usize) -> Vec3) -> fmn::Result<Vec<Mob>> {
    values
        .iter()
        .enumerate()
        .map(|(i, n)| kit.text(stage, &n.to_string(), COUNT_SIZE, Palette::ink(), at(i)))
        .collect()
}

impl SceneConstruct for WhereTheWaterStops {
    fn name(&self) -> &str {
        "03_where_the_water_stops"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let face = model::face();
        let s = facts::no_fit();
        let total = s.row_total();
        if s.rows != face.row_counts() || total != s.col_total() {
            return Err(fail(
                "the no-fit shadows are not the face's rows with the same total".into(),
            ));
        }
        let (flow, states) = facts::no_fit_flow().map_err(fail)?;
        let cut = flow.cut(&s);
        let end = states
            .last()
            .ok_or_else(|| fail("the no-fit flow sends nothing".into()))?;
        // the chapter's shape: units straight down, then a stop one short,
        // with one column cut off and every tap reached
        let &[first] = cut.cut_off.as_slice() else {
            return Err(fail(format!("not one column cut off: {cut:?}")));
        };
        if flow.value + 1 != total || cut.value() != flow.value || cut.unreached_taps != 0 {
            return Err(fail(format!(
                "the no-fit flow does not stop one short at a cut of its size: {cut:?}"
            )));
        }
        let left: Vec<usize> = (0..s.rows.len()).filter(|&r| end.sent[r] < s.rows[r]).collect();
        let &[left_row] = left.as_slice() else {
            return Err(fail(format!("not one row left with water: {left:?}")));
        };
        let board = Board::new(AT, CELL, face.rows(), face.cols());

        // 03_1: the grid, the face's taps, the other columns' drains; the
        // method's real units go through, as in chapter 02
        let line = say(&kit, stage, "03_1")?;
        let grid = board.lines(stage);
        let taps = counters(&kit, stage, &s.rows, |r| tap_at(&board, r))?;
        let send_label = kit.text(
            stage,
            TO_SEND,
            22.0,
            Palette::dim(),
            v(board.left() - TAP_GAP, board.top() + 0.4),
        )?;
        let mut arriving = grid.clone();
        arriving.push(send_label);
        fade_in_all(stage, &arriving, 0.5)?;
        fade_in_lagged(stage, &taps, 0.8, 0.3)?;
        line.wait_for(stage, "and columns asking")?;
        let drains = counters(&kit, stage, &s.cols, |c| board.col_count_at(c))?;
        let drain_label = kit.text(
            stage,
            TO_DRAIN,
            22.0,
            Palette::dim(),
            v(board.left() - 1.0, board.col_count_at(0)[1]),
        )?;
        fade_in_lagged(stage, &drains, 0.9, 0.3)?;
        let counter = kit.text_left(
            stage,
            &through_text(0, total),
            26.0,
            Palette::ink(),
            v(TEXT_X, THROUGH_Y),
        )?;
        fade_in_all(stage, &[drain_label, counter], 0.3)?;
        let mut water = Water::new(board, taps, drains, &s);
        water.through = Some(counter);
        for (path, state) in flow.paths.iter().zip(&states) {
            send_quick(&kit, stage, &mut water, &s, path, state)?;
        }

        // 03_2: stopped one short: the row with water left, the column
        // still asking; the question, and the think time (nothing moves)
        let line = say(&kit, stage, "03_2")?;
        line.wait_for(stage, "stops at eight")?;
        let mut stopped: Vec<Mob> = water.through.into_iter().collect();
        stopped.extend([water.taps[left_row], water.drains[first]]);
        flash_all(stage, &stopped, 0.6)?;
        let q = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 03_3: the first column needs five; each row sends it at most one
        // (its one square there); only four rows have water
        let line = say(&kit, stage, "03_3")?;
        fade_out_all(stage, &[q], 0.3)?;
        line.wait_for(stage, "The first column")?;
        let outline = column_outline(stage, &board, first, palette::impossible());
        let asks_label = kit.text(
            stage,
            &format!("asks {}", s.cols[first]),
            22.0,
            palette::impossible(),
            v(board.at(0, first)[0], board.col_count_at(first)[1] + ASKS_DY),
        )?;
        fade_in_all(stage, &[outline, asks_label], 0.4)?;
        line.wait_for(stage, "each row sends")?;
        let rings: Vec<Mob> = (0..board.rows)
            .map(|r| board.ring(stage, r, first, Palette::ink()))
            .collect();
        fade_in_lagged(stage, &rings, 0.8, 0.3)?;
        line.wait_for(stage, "only four rows")?;
        let dry: Vec<usize> = (0..board.rows).filter(|&r| s.rows[r] == 0).collect();
        let mut crosses = Vec::new();
        for &r in &dry {
            crosses.extend(board.cross(stage, r, first, palette::impossible()));
        }
        fade_in_all(stage, &crosses, 0.3)?;
        let empty_taps: Vec<Mob> = dry.iter().map(|&r| water.taps[r]).collect();
        flash_all(stage, &empty_taps, 0.5)?;

        // 03_4: everywhere the water can still go, taking back included,
        // misses the first column; the cut: four pipes into it, four drains
        // out of the others, eight in all. A reached row's stroke runs over
        // the reached columns only, so no teal crosses the red column
        let line = say(&kit, stage, "03_4")?;
        fade_out_all(stage, &rings, 0.3)?;
        let (reach_from, reach_to) = row_reach(&board, &cut.cols, &cut.cut_off)
            .ok_or_else(|| fail(format!("the reached columns are not one run beside the cut: {cut:?}")))?;
        let mut reach = Vec::with_capacity(cut.rows.len() + cut.cols.len());
        for &r in &cut.rows {
            let y = board.at(r, 0)[1];
            reach.push(segment(stage, v(reach_from, y), v(reach_to, y), palette::beam(), 5.0));
        }
        for &c in &cut.cols {
            let x = board.at(0, c)[0];
            reach.push(segment(
                stage,
                v(x, board.top() + 0.1),
                v(x, board.bottom() - 0.1),
                palette::beam(),
                5.0,
            ));
        }
        let reach_label = kit.text_left(stage, STILL_REACHES, 24.0, palette::beam(), v(TEXT_X, CUT_Y[0]))?;
        fade_in_lagged(stage, &reach, 1.0, 0.2)?;
        fade_in_all(stage, &[reach_label], 0.3)?;
        line.wait_for(stage, "Four into")?;
        let pipes: Vec<Mob> = cut
            .rows
            .iter()
            .filter_map(|&r| water.wet.get(&(r, first)).copied())
            .collect();
        if pipes.len() != cut.pipes {
            return Err(fail(format!(
                "{} wet pipes into the cut-off column, not {}",
                pipes.len(),
                cut.pipes
            )));
        }
        let into_first = kit.text_left(
            stage,
            &into_first_text(cut.pipes),
            24.0,
            Palette::ink(),
            v(TEXT_X, CUT_Y[1]),
        )?;
        flash_all(stage, &pipes, 0.6)?;
        fade_in_all(stage, &[into_first], 0.3)?;
        line.wait_for(stage, "four out of")?;
        let others: Vec<Mob> = cut.cols.iter().map(|&c| water.drains[c]).collect();
        let out_of_others = kit.text_left(
            stage,
            &other_drains_text(cut.reached_drains),
            24.0,
            Palette::ink(),
            v(TEXT_X, CUT_Y[2]),
        )?;
        flash_all(stage, &others, 0.6)?;
        fade_in_all(stage, &[out_of_others], 0.3)?;
        line.wait_for(stage, "Eight is all")?;
        let sum = kit.text_left(
            stage,
            &cut_sum_text(cut.pipes, cut.reached_drains, cut.value()),
            26.0,
            Palette::ink(),
            v(TEXT_X, CUT_Y[2] - 0.6),
        )?;
        fade_in_all(stage, &[sum], 0.4)?;

        // 03_5: the same count for any group of columns: a row gives k
        // columns at most min(its count, k); what the rows can give, k = 1 to 5
        let line = say(&kit, stage, "03_5")?;
        let mut leaving = reach.clone();
        leaving.extend([reach_label, into_first, out_of_others, sum]);
        fade_out_all(stage, &leaving, 0.4)?;
        let table = model::gale_table(&s);
        let sizes: Vec<usize> = (1..=table.len()).collect();
        let gives: Vec<usize> = table.iter().map(|&(_, gives)| gives).collect();
        let k_labels = table_line(&kit, stage, &sizes, K_Y, Palette::dim())?;
        let mut labels = Vec::with_capacity(4);
        for (text, y) in [
            (K_LABEL, K_Y),
            (SORTED_LABEL, SORTED_Y),
            (ASK_LABEL, ASK_Y),
            (GIVE_LABEL, GIVE_Y),
        ] {
            labels.push(kit.text_left(stage, text, 22.0, Palette::dim(), v(TEXT_X, y))?);
        }
        let mut give_line = table_line(&kit, stage, &gives, GIVE_Y, Palette::ink())?;
        let mut shown = k_labels.clone();
        shown.extend([labels[0], labels[3]]);
        fade_in_all(stage, &shown, 0.4)?;
        line.wait_for(stage, "Each row gives")?;
        let rule = kit.text_left(stage, GIVE_RULE, 22.0, Palette::ink(), v(TEXT_X, RULE_Y))?;
        fade_in_all(stage, &[rule], 0.3)?;
        fade_in_lagged(stage, &give_line, 0.9, 0.3)?;

        // 03_6: the k largest columns, what they ask; the single largest
        // already asks too much; the hungriest group of each size is enough
        let line = say(&kit, stage, "03_6")?;
        let sorted = facts::largest_first(&s.cols);
        let asks_now: Vec<usize> = table.iter().map(|&(asks, _)| asks).collect();
        let mut sorted_line = table_line(&kit, stage, &sorted, SORTED_Y, Palette::ink())?;
        let mut ask_line = table_line(&kit, stage, &asks_now, ASK_Y, Palette::ink())?;
        let mut arriving = sorted_line.clone();
        arriving.extend(&ask_line);
        arriving.extend([labels[1], labels[2]]);
        fade_in_lagged(stage, &arriving, 0.9, 0.2)?;
        line.wait_for(stage, "no picture fits")?;
        for (i, &(asks, gives)) in table.iter().enumerate() {
            if asks > gives {
                ask_line[i] = recolor(&kit, stage, ask_line[i], asks, i + 1, ASK_Y, palette::impossible())?;
                give_line[i] = recolor(&kit, stage, give_line[i], gives, i + 1, GIVE_Y, palette::impossible())?;
            }
        }
        line.wait_for(stage, "the hungriest")?;
        flash_all(stage, &sorted_line, 0.6)?;

        // 03_7: with equal totals the test never misses: where the water
        // stops is always a group asking too much (here the first column)
        let line = say(&kit, stage, "03_7")?;
        let last = table.len();
        let (all_ask, all_give) = table[last - 1];
        if all_ask != total || all_give != total {
            return Err(fail(format!(
                "the table's last size is not the equal totals: {all_ask}, {all_give}"
            )));
        }
        let equal = kit.text(stage, EQUAL, 20.0, palette::matches(), cell_at(last, K_Y + 0.45))?;
        fade_in_all(stage, &[equal], 0.3)?;
        ask_line[last - 1] = recolor(
            &kit,
            stage,
            ask_line[last - 1],
            all_ask,
            last,
            ASK_Y,
            palette::matches(),
        )?;
        give_line[last - 1] = recolor(
            &kit,
            stage,
            give_line[last - 1],
            all_give,
            last,
            GIVE_Y,
            palette::matches(),
        )?;
        line.wait_for(stage, "When water stops short")?;
        let claim = gale_claim();
        let card = precisely(&kit, stage, &claim)?;
        fade_in_all(stage, &[card], 0.4)?;
        let card_up = stage.scene().time().to_f64();
        let failing: Vec<Mob> = table
            .iter()
            .zip(ask_line.iter().zip(&give_line))
            .filter(|((asks, gives), _)| asks > gives)
            .flat_map(|(_, (a, g))| [*a, *g])
            .collect();
        let mut bottleneck = vec![outline, asks_label];
        bottleneck.extend(&failing);
        flash_all(stage, &bottleneck, 0.8)?;
        // the card stands whole, the line finished, until it can be read
        line.wait_to_end(stage)?;
        wait_until(stage, card_up + read_s(&claim))?;

        // 03_8: the face's shadows pass every size; sixty-four pictures fit;
        // the test says that one fits, not which, not whether only one
        let line = say(&kit, stage, "03_8")?;
        let face_shadows = face.shadows();
        let face_table = model::gale_table(&face_shadows);
        if !model::gale_ryser(&face_shadows) || face_table.iter().any(|&(asks, gives)| asks > gives) {
            return Err(fail("the face's shadows fail Gale's table".into()));
        }
        let mut clearing: Vec<Mob> = std::mem::take(&mut water.wet).into_values().collect();
        clearing.extend([card, outline, asks_label, equal]);
        clearing.extend(&crosses);
        clearing.extend(water.through);
        fade_out_all(stage, &clearing, 0.4)?;
        let face_taps = counters(&kit, stage, &face_shadows.rows, |r| tap_at(&board, r))?;
        let face_drains = counters(&kit, stage, &face_shadows.cols, |c| board.col_count_at(c))?;
        let face_sorted = table_line(
            &kit,
            stage,
            &facts::largest_first(&face_shadows.cols),
            SORTED_Y,
            Palette::ink(),
        )?;
        let face_asks: Vec<usize> = face_table.iter().map(|&(asks, _)| asks).collect();
        let face_gives: Vec<usize> = face_table.iter().map(|&(_, gives)| gives).collect();
        let face_ask_line = table_line(&kit, stage, &face_asks, ASK_Y, palette::matches())?;
        let face_give_line = table_line(&kit, stage, &face_gives, GIVE_Y, palette::matches())?;
        let mut old = water.taps.clone();
        old.extend(&water.drains);
        old.append(&mut sorted_line);
        old.append(&mut ask_line);
        old.append(&mut give_line);
        let mut new = face_taps;
        new.extend(face_drains);
        new.extend(&face_sorted);
        new.extend(&face_ask_line);
        new.extend(&face_give_line);
        swap(stage, &old, &new, 0.6)?;
        line.wait_for(stage, "sixty-four")?;
        let count = facts::gallery().len();
        let fits = kit.text_left(stage, &fits_text(count), 26.0, Palette::ink(), v(TEXT_X, CLOSING_Y[0]))?;
        fade_in_all(stage, &[fits], 0.4)?;
        line.wait_for(stage, "The test says")?;
        let not_said = kit.text_left(stage, NOT_SAID, 24.0, Palette::dim(), v(TEXT_X, CLOSING_Y[1]))?;
        fade_in_all(stage, &[not_said], 0.4)?;
        line.wait_to_end(stage)?;
        pause(stage, 1.2)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use explainer_kit::animatic::wrap;
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("03_1", "and columns asking"),
            ("03_2", "stops at eight"),
            ("03_3", "The first column"),
            ("03_3", "each row sends"),
            ("03_3", "only four rows"),
            ("03_4", "Four into"),
            ("03_4", "four out of"),
            ("03_4", "Eight is all"),
            ("03_5", "Each row gives"),
            ("03_6", "no picture fits"),
            ("03_6", "the hungriest"),
            ("03_7", "When water stops short"),
            ("03_8", "sixty-four"),
            ("03_8", "The test says"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The words on screen are the facts': the cut's parts, the count of
    /// pictures; the card states both of Gale's conditions and keeps to
    /// three whole lines at every width the band may narrow it to (down to
    /// 76 characters), never breaking "min(row count, k)" across two; its
    /// reading hold stays under twelve seconds.
    #[test]
    fn the_words_on_screen_are_built_from_the_facts() {
        let f = facts::facts().unwrap();
        let get = |key: &str| -> usize { f.get(key).unwrap().parse().unwrap() };
        assert_eq!(
            into_first_text(get("DT-F9.cut.into_unreached")),
            "4 pipes into the first column"
        );
        assert_eq!(
            other_drains_text(get("DT-F9.cut.reached_drains")),
            "4 out of the other drains"
        );
        assert_eq!(
            cut_sum_text(
                get("DT-F9.cut.into_unreached"),
                get("DT-F9.cut.reached_drains"),
                get("DT-F9.cut")
            ),
            "4 + 4 = 8, all that can pass"
        );
        assert_eq!(fits_text(get("DT-F3.fits")), "64 pictures fit these shadows");
        let claim = gale_claim();
        for part in [
            "fits exactly when",
            "row and column totals match",
            "for every k, the k largest columns",
            "at most the sum over rows of min(row count, k)",
            "Gale, 1957",
            "Ryser, 1957",
        ] {
            assert!(claim.contains(part), "{part}: {claim}");
        }
        assert!(read_s(&claim) < 12.0, "{}", read_s(&claim));
        for width in (76..=92).step_by(4) {
            let lines = wrap(&claim, width, 3);
            assert!(!lines.iter().any(|l| l.ends_with('…')), "{width}: {lines:?}");
            assert!(
                lines.iter().any(|l| l.contains("min(row count, k)")),
                "{width}: {lines:?}"
            );
        }
    }

    /// 03_4's row strokes, on the real cut, run over the reached columns
    /// only: they stop clear of the cut-off column's outline and cross
    /// every reached column; a cut-off column inside the run is refused.
    #[test]
    fn the_reach_strokes_stay_off_the_cut_off_column() {
        let (flow, _) = facts::no_fit_flow().unwrap();
        let cut = flow.cut(&facts::no_fit());
        let board = Board::new(AT, CELL, 5, 5);
        let (from, to) = row_reach(&board, &cut.cols, &cut.cut_off).unwrap();
        for &c in &cut.cut_off {
            let x = board.at(0, c)[0];
            let (l, r) = (x - board.cell / 2.0, x + board.cell / 2.0);
            assert!(to < l - 0.05 || from > r + 0.05, "column {c}: {from}..{to}");
        }
        for &c in &cut.cols {
            let x = board.at(0, c)[0];
            assert!(from < x && x < to, "column {c} at {x}: {from}..{to}");
        }
        assert!(row_reach(&board, &[0, 2], &[1]).is_none());
        assert!(row_reach(&board, &[], &[0]).is_none());
    }

    /// The chapter's layout keeps its pieces apart and inside the
    /// title-safe area (text 0.25 in from the frame's edge).
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        let board = Board::new(AT, CELL, 5, 5);
        let safe_x = FRAME_X_RADIUS - 0.25;
        let safe_y = FRAME_Y_RADIUS - 0.25;
        // the table's last entry, and its "equal totals" note, inside
        assert!(cell_at(5, K_Y)[0] + 0.6 < safe_x);
        assert!(K_Y + 0.45 + 0.2 < safe_y);
        // its first entry clears its widest label (21 characters at 22 pt,
        // about 0.0045 units a point a character)
        assert!(TEXT_X + 21.0 * 22.0 * 0.0045 + 0.3 < cell_at(1, K_Y)[0] - 0.2);
        // the words on the right clear the grid's right edge, and the
        // longest of 03_4's (24 pt) ends inside
        assert!(TEXT_X > board.right() + 0.5);
        assert!(TEXT_X + STILL_REACHES.chars().count() as f64 * 24.0 * 0.0045 < safe_x);
        // a column's ask, under its drain, stays over a three-line card
        // (its top 0.2 + 0.3 + 3 x 0.36 above the frame's bottom)
        let card_top = -FRAME_Y_RADIUS + 0.2 + 0.3 + 3.0 * 0.36;
        assert!(board.col_count_at(0)[1] + ASKS_DY - 0.15 > card_top);
        assert!(CLOSING_Y[1] - 0.15 > card_top);
    }
}
