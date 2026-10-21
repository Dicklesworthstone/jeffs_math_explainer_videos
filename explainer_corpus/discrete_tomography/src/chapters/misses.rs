//! Chapter 07, why the test never misses: with equal totals, when the water
//! stops short, the columns it can't reach always form a group that asks
//! too much (07_1); on chapter 06's shadows, the two cut-off columns can only
//! be fed from the rows the water can't reach, three units, or through the
//! four pipes into them from the rows it can (07_2); the water used every one
//! of those, and the three drains it can reach, and still came up one short,
//! so those columns ask for more than three plus four (07_3); that count
//! works whenever the water stops short (07_4, Gale's card); equal totals
//! matter: rows 2, 2 with columns 1, 1 pass every size of the test (07_5, a
//! prediction: does a picture fit?); no, four squares against two (07_6);
//! the face's shadows pass, and the test says that one fits, not which and
//! not whether only one (07_7).
//!
//! The stopped water is `facts::stall_flow`'s last replayed state and its
//! `Flow::cut`; the unequal shadows `facts::unequal`; the tables
//! `model::gale_table`.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{Board, fade_in_all, fade_out_all, fail, flash_all, pause, say, think, wait_until};
use super::stops::{
    ASK_Y, GIVE_Y, K_Y, SORTED_Y, columns_outline, counters, gale_claim, reach_strokes, read_s, table_labels,
    table_line,
};
use super::water::{AT, CELL, TEXT_X, tap_at, water_at};
use crate::facts;
use crate::model;
use crate::palette;

pub struct WhyTheTestNeverMisses {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const NOTHING_FITS: &str = "nothing fits";
const NOT_SAID: &str = "it says one fits, not which, not whether only one";

/// The argument's words on the right, top to bottom (07_2, 07_3), and the
/// last, under the units-through counter.
const WORDS_Y: [f64; 4] = [2.1, 1.5, 0.9, 0.3];
const BOUND_Y: f64 = -1.2;
/// The think ring's centre (07_5), under the small table.
const THINK_LOW: [f64; 2] = [3.4, -1.0];
/// 07_6's words under the small table, and 07_7's under the face's.
const VERDICT_Y: [f64; 2] = [0.15, -0.45];
const CLOSING_Y: [f64; 2] = [-0.45, -1.0];

fn unreached_text(units: usize) -> String {
    format!("rows it can't reach: {units} units")
}

fn pipes_text(pipes: usize) -> String {
    format!("pipes into the pair: {pipes}")
}

/// "through: 10 = 3 + 4 + 3" (07_3).
fn crossing_text(flow: usize, taps: usize, pipes: usize, drains: usize) -> String {
    format!("through: {flow} = {taps} + {pipes} + {drains}")
}

fn short_text(short: usize, total: usize) -> String {
    format!("{short} short of {total}")
}

/// "they ask 11 − 3 = 8, more than 3 + 4 = 7" (07_3).
fn bound_text(total: usize, drains: usize, taps: usize, pipes: usize) -> String {
    format!(
        "they ask {total} − {drains} = {}, more than {taps} + {pipes} = {}",
        total - drains,
        taps + pipes
    )
}

/// "rows: 4, columns: 2" (07_6).
fn totals_text(rows: usize, cols: usize) -> String {
    format!("rows: {rows}, columns: {cols}")
}

fn fits_text(count: usize) -> String {
    format!("{count} pictures fit these shadows")
}

impl SceneConstruct for WhyTheTestNeverMisses {
    fn name(&self) -> &str {
        "07_why_the_test_never_misses"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let s = facts::stall();
        let total = s.row_total();
        let (flow, states) = facts::stall_flow().map_err(fail)?;
        let end = states
            .last()
            .ok_or_else(|| fail("the stall flow sends nothing".into()))?;
        let cut = flow.cut(&s);
        let (&first, &last) = (
            cut.cut_off
                .first()
                .ok_or_else(|| fail("the stall flow cuts off no column".into()))?,
            cut.cut_off.last().unwrap_or(&0),
        );
        let asked: usize = cut.cut_off.iter().map(|&j| s.cols[j]).sum();
        let fed = cut.unreached_taps + cut.pipes;
        // the argument: every crossing is full, the flow one short of the
        // total, so the cut-off columns ask for more than can feed them, and
        // what can feed them bounds any picture: Σ min(r, k) at most
        if cut.value() != flow.value
            || flow.value + 1 != total
            || asked != total - cut.reached_drains
            || asked <= fed
            || fed < model::can_give(&s.rows, cut.cut_off.len())
        {
            return Err(fail(format!(
                "the stall cut {cut:?} does not carry the argument: asked {asked}, fed {fed}"
            )));
        }
        let board = Board::new(AT, CELL, s.rows.len(), s.cols.len());

        // 07_1: chapter 06's stopped water, its reach and the pair outlined
        let line = say(&kit, stage, "07_1")?;
        let (water, mut drawn, _) = water_at(&kit, stage, board, &s, end)?;
        let reach = reach_strokes(stage, &board, &cut)?;
        let outline = columns_outline(stage, &board, first, last, palette::impossible());
        drawn.extend(&reach);
        drawn.push(outline);
        fade_in_all(stage, &drawn, 0.5)?;
        line.wait_for(stage, "the columns it can't reach")?;
        flash_all(stage, &[outline], 0.6)?;

        // 07_2: the pair can only be fed from the rows the water can't reach,
        // or through the pipes into it from the rows it can
        let line = say(&kit, stage, "07_2")?;
        line.wait_for(stage, "the rows the water can't reach")?;
        let unreached: Vec<usize> = (0..s.rows.len()).filter(|r| !cut.rows.contains(r)).collect();
        let unreached_taps: Vec<Mob> = unreached.iter().map(|&r| water.taps[r]).collect();
        let fed_rows = kit.text_left(
            stage,
            &unreached_text(cut.unreached_taps),
            24.0,
            Palette::ink(),
            v(TEXT_X, WORDS_Y[0]),
        )?;
        flash_all(stage, &unreached_taps, 0.5)?;
        fade_in_all(stage, &[fed_rows], 0.3)?;
        line.wait_for(stage, "the four pipes")?;
        let mut pipe_rings = Vec::with_capacity(cut.pipes);
        for &r in &cut.rows {
            for &c in &cut.cut_off {
                pipe_rings.push(board.ring(stage, r, c, Palette::ink()));
            }
        }
        if pipe_rings.len() != cut.pipes {
            return Err(fail(format!("{} pipes ringed, not {}", pipe_rings.len(), cut.pipes)));
        }
        let pipes = kit.text_left(stage, &pipes_text(cut.pipes), 24.0, Palette::ink(), v(TEXT_X, WORDS_Y[1]))?;
        fade_in_all(stage, &pipe_rings, 0.3)?;
        fade_in_all(stage, &[pipes], 0.3)?;

        // 07_3: the water used them all, and the reached drains, and came up
        // one short: so the pair asks for more than those can bring
        let line = say(&kit, stage, "07_3")?;
        line.wait_for(stage, "used every one")?;
        let crossing = kit.text_left(
            stage,
            &crossing_text(flow.value, cut.unreached_taps, cut.pipes, cut.reached_drains),
            24.0,
            Palette::ink(),
            v(TEXT_X, WORDS_Y[2]),
        )?;
        let reached_drains: Vec<Mob> = cut.cols.iter().map(|&c| water.drains[c]).collect();
        flash_all(stage, &reached_drains, 0.5)?;
        fade_in_all(stage, &[crossing], 0.3)?;
        line.wait_for(stage, "one short")?;
        let short = kit.text_left(
            stage,
            &short_text(total - flow.value, total),
            24.0,
            Palette::ink(),
            v(TEXT_X, WORDS_Y[3]),
        )?;
        fade_in_all(stage, &[short], 0.3)?;
        line.wait_for(stage, "more than three plus four")?;
        let bound = kit.text_left(
            stage,
            &bound_text(total, cut.reached_drains, cut.unreached_taps, cut.pipes),
            24.0,
            palette::impossible(),
            v(TEXT_X, BOUND_Y),
        )?;
        fade_in_all(stage, &[bound], 0.4)?;

        // 07_4: that count works whenever the water stops short: Gale's
        // card, held until it can be read
        let line = say(&kit, stage, "07_4")?;
        line.wait_for(stage, "whenever")?;
        let claim = gale_claim();
        let card = precisely(&kit, stage, &claim)?;
        fade_in_all(stage, &[card], 0.4)?;
        let card_up = stage.scene().time().to_f64();
        line.wait_to_end(stage)?;
        wait_until(stage, card_up + read_s(&claim))?;

        // 07_5: equal totals matter: rows 2, 2 with columns 1, 1 pass every
        // size of the test; the question
        let line = say(&kit, stage, "07_5")?;
        let mut leaving = drawn;
        leaving.extend(pipe_rings);
        leaving.extend([fed_rows, pipes, crossing, short, bound, card]);
        fade_out_all(stage, &leaving, 0.4)?;
        let u = facts::unequal();
        let u_table = model::gale_table(&u);
        if u.row_total() == u.col_total() || u_table.iter().any(|&(a, g)| a > g) || !model::fits(&u).is_empty() {
            return Err(fail("the unequal shadows do not pass the table and fit nothing".into()));
        }
        let small = Board::new(AT, CELL, u.rows.len(), u.cols.len());
        let mut tiny = small.lines(stage);
        tiny.extend(counters(&kit, stage, &u.rows, |r| tap_at(&small, r))?);
        tiny.extend(counters(&kit, stage, &u.cols, |c| small.col_count_at(c))?);
        fade_in_all(stage, &tiny, 0.4)?;
        line.wait_for(stage, "pass every size")?;
        let sizes: Vec<usize> = (1..=u_table.len()).collect();
        let labels = table_labels(&kit, stage)?;
        let mut table = table_line(&kit, stage, &sizes, K_Y, Palette::dim())?;
        let asks: Vec<usize> = u_table.iter().map(|&(a, _)| a).collect();
        let gives: Vec<usize> = u_table.iter().map(|&(_, g)| g).collect();
        table.extend(table_line(&kit, stage, &asks, ASK_Y, palette::matches())?);
        table.extend(table_line(&kit, stage, &gives, GIVE_Y, palette::matches())?);
        table.extend([labels[0], labels[2], labels[3]]);
        fade_in_all(stage, &table, 0.5)?;
        let q = think(&kit, stage, v(THINK_LOW[0], THINK_LOW[1]))?;

        // 07_6: no: the rows ask for four squares, the columns for two
        let line = say(&kit, stage, "07_6")?;
        fade_out_all(stage, &[q], 0.3)?;
        line.wait_for(stage, "four squares")?;
        let totals = kit.text_left(
            stage,
            &totals_text(u.row_total(), u.col_total()),
            26.0,
            palette::impossible(),
            v(TEXT_X, VERDICT_Y[0]),
        )?;
        let nothing = kit.text_left(
            stage,
            NOTHING_FITS,
            26.0,
            palette::impossible(),
            v(TEXT_X, VERDICT_Y[1]),
        )?;
        fade_in_all(stage, &[totals, nothing], 0.4)?;

        // 07_7: the face's shadows pass every size; sixty-four pictures fit;
        // the test says that one fits, not which, not whether only one
        let line = say(&kit, stage, "07_7")?;
        let mut going = tiny;
        going.extend(table);
        going.extend([totals, nothing]);
        fade_out_all(stage, &going, 0.4)?;
        let face_shadows = model::face().shadows();
        let face_table = model::gale_table(&face_shadows);
        if !model::gale_ryser(&face_shadows) || face_table.iter().any(|&(a, g)| a > g) {
            return Err(fail("the face's shadows fail Gale's table".into()));
        }
        let mut shown = board.lines(stage);
        shown.extend(counters(&kit, stage, &face_shadows.rows, |r| tap_at(&board, r))?);
        shown.extend(counters(&kit, stage, &face_shadows.cols, |c| board.col_count_at(c))?);
        let face_sizes: Vec<usize> = (1..=face_table.len()).collect();
        let face_labels = table_labels(&kit, stage)?;
        shown.extend(table_line(&kit, stage, &face_sizes, K_Y, Palette::dim())?);
        shown.extend(table_line(
            &kit,
            stage,
            &facts::largest_first(&face_shadows.cols),
            SORTED_Y,
            Palette::ink(),
        )?);
        let face_asks: Vec<usize> = face_table.iter().map(|&(a, _)| a).collect();
        let face_gives: Vec<usize> = face_table.iter().map(|&(_, g)| g).collect();
        shown.extend(table_line(&kit, stage, &face_asks, ASK_Y, palette::matches())?);
        shown.extend(table_line(&kit, stage, &face_gives, GIVE_Y, palette::matches())?);
        shown.extend(face_labels);
        fade_in_all(stage, &shown, 0.5)?;
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
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;
    use crate::chapters::stops::cell_at;
    use crate::chapters::water::THROUGH_Y;

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("07_1", "the columns it can't reach"),
            ("07_2", "the rows the water can't reach"),
            ("07_2", "the four pipes"),
            ("07_3", "used every one"),
            ("07_3", "one short"),
            ("07_3", "more than three plus four"),
            ("07_4", "whenever"),
            ("07_5", "pass every size"),
            ("07_6", "four squares"),
            ("07_7", "sixty-four"),
            ("07_7", "The test says"),
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
        assert_eq!(unreached_text(get("DT-F15.cut.unreached_taps")), "rows it can't reach: 3 units");
        assert_eq!(pipes_text(get("DT-F15.cut.pipes")), "pipes into the pair: 4");
        assert_eq!(
            crossing_text(
                get("DT-F15.flow"),
                get("DT-F15.cut.unreached_taps"),
                get("DT-F15.cut.pipes"),
                get("DT-F15.cut.reached_drains")
            ),
            "through: 10 = 3 + 4 + 3"
        );
        assert_eq!(short_text(get("DT-F15.short"), 11), "1 short of 11");
        assert_eq!(
            bound_text(11, get("DT-F15.cut.reached_drains"), 3, 4),
            "they ask 11 − 3 = 8, more than 3 + 4 = 7"
        );
        assert_eq!(f.get("DT-F16.totals"), Some("4,2"));
        assert_eq!(totals_text(4, 2), "rows: 4, columns: 2");
        assert_eq!(fits_text(get("DT-F3.fits")), "64 pictures fit these shadows");
    }

    /// The chapter's layout keeps its pieces apart and inside the
    /// title-safe area (about 0.0045 units a point a character).
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        let safe_x = FRAME_X_RADIUS - 0.25;
        let ends = |x: f64, text: &str, size: f64| x + text.chars().count() as f64 * size * 0.0045;
        assert!(ends(TEXT_X, &bound_text(11, 3, 3, 4), 24.0) < safe_x);
        assert!(ends(TEXT_X, NOT_SAID, 24.0) < safe_x);
        // the argument's words stay apart from the units-through counter
        assert!(WORDS_Y[3] - 0.15 > THROUGH_Y + 0.2);
        assert!(BOUND_Y + 0.15 < THROUGH_Y - 0.2);
        // the bound and the closing words stay over a three-line card (its
        // top 0.2 + 0.3 + 3 x 0.36 above the frame's bottom)
        let card_top = -FRAME_Y_RADIUS + 0.2 + 0.3 + 3.0 * 0.36;
        assert!(BOUND_Y - 0.15 > card_top);
        assert!(CLOSING_Y[1] - 0.15 > card_top);
        // the think ring and its note sit under the small table's last line
        // and over the frame's foot, clear of the verdict's words
        assert!(THINK_LOW[1] + 0.62 < GIVE_Y - 0.2);
        assert!(THINK_LOW[1] - 0.62 - 0.3 - 0.15 > -FRAME_Y_RADIUS + 0.25);
        assert!(THINK_LOW[0] - 1.1 > ends(TEXT_X, NOTHING_FITS, 26.0));
        // the small table's last entry stands inside; the verdict's words
        // stand under its last line
        assert!(cell_at(2, K_Y)[0] + 0.3 < safe_x);
        assert!(VERDICT_Y[0] + 0.15 < GIVE_Y - 0.2);
    }
}
