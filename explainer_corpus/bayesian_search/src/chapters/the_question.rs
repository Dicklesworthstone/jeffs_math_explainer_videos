//! Chapter 01, the question: the owner's subject line, set large one
//! sentence at a time (01_1); then, with the voice stopped, a search of the
//! real 1968 map that finds nothing and still changes it, every other cell
//! deepening in the instant the searched cell's dots go out (01_2); then
//! the title over the map, which dims (01_3). No labels and no numbers on
//! the map: it is shown before it is described.

use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    Heat, Lattice, Probe, carry_pause, fade_in_all, fade_in_lagged, fade_out_all, fail, fit_width, say, veil,
    wait_until,
};
use super::glyphs;
use crate::facts::shown;
use crate::model::y1968;

pub struct TheQuestion {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order (01_3, the title,
/// has no words).
pub const LINES: [&str; 2] = ["01_1", "01_2"];

/// The three sentences' heights and sizes: the first two in `dim`, the
/// question in ink and a little larger.
const SENTENCE_Y: [f64; 3] = [1.7, 0.3, -1.1];
const SENTENCE_PT: [f64; 3] = [50.0, 50.0, 58.0];
/// The widest a sentence runs on the stage.
const ROOM: f64 = 12.8;
/// A row of the lattice, in scene units: sixteen rows fill the stage.
const ROW_H: f64 = 0.46;
/// The strip's crossing, the dots going out, and the title's size.
const CROSS_S: f64 = 1.0;
const MOVE_S: f64 = 0.9;
const TITLE_PT: f64 = 64.0;
/// The title's listen window: the motif, unanswered.
const TITLE_LISTEN_S: f64 = 3.0;

/// The search of the cold open: the map's brightest cell, searched once by
/// a search that sees what is there two times in three (the video's own
/// model, `F-R.model.seeing`).
pub fn cold_open_probe() -> fmn::Result<Probe> {
    let (row, col, _) = y1968::cells()
        .max_by_key(|&(_, _, n)| n)
        .ok_or_else(|| fail("the 1968 map has no cells".into()))?;
    Ok(Probe {
        row,
        col,
        seeing: shown::MAP_SEEING,
    })
}

impl SceneConstruct for TheQuestion {
    fn name(&self) -> &str {
        "01_the_question"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();

        // 01_1: three sentences, each set as it is said; the question in ink
        let line = say(&kit, stage, LINES[0])?;
        let words = line.sentence_texts();
        if words.len() != SENTENCE_Y.len() {
            return Err(fail(format!(
                "01_1 has {} sentences, the chapter sets {}",
                words.len(),
                SENTENCE_Y.len()
            )));
        }
        let mut set = Vec::new();
        for (j, text) in words.iter().enumerate() {
            let color = if j + 1 == words.len() {
                Palette::ink()
            } else {
                Palette::dim()
            };
            let m = kit.text(stage, text, SENTENCE_PT[j], color, v(0.0, SENTENCE_Y[j]))?;
            fit_width(stage, m, ROOM);
            line.wait_sentence(stage, j)?;
            fade_in_all(stage, &[m], 0.4)?;
            set.push(m);
        }
        // a question to carry: the subject line, held about two seconds
        carry_pause(&kit, stage)?;
        fade_out_all(stage, &set, 0.5)?;

        // 01_2: the voice stops between the two sentences; the map is built,
        // searched and changed in that gap, and the second sentence lands
        // on the changed map
        let line = say(&kit, stage, LINES[1])?;
        let probe = cold_open_probe()?;
        let map = Lattice::build(stage, v(0.0, 0.0), ROW_H, &[probe], Heat::LATTICE)?;
        fade_in_lagged(stage, &map.lines, 0.9, 0.6)?;
        let mut under = vec![map.anchor];
        under.extend(&map.heat);
        fade_in_all(stage, &under, 0.5)?;
        // thick near the middle: the cells nearest it first
        fade_in_lagged(stage, &map.dots, 1.1, 0.5)?;

        // the search lands in the silence, so the changed map is there as
        // the second sentence begins
        wait_until(stage, line.sentence(1)? - CROSS_S - MOVE_S)?;
        // most of that cell's dots go out; in the same instant every other
        // cell's blue deepens (recounted from the dots that are still lit)
        map.search(&kit, stage, 0, CROSS_S, MOVE_S)?;
        line.wait_end(stage)?;

        // 01_3: the title over the map, which dims
        let dim = veil(stage)?;
        let title = kit.text(stage, glyphs::TITLE, TITLE_PT, Palette::ink(), v(0.0, 0.0))?;
        fit_width(stage, title, ROOM);
        fade_in_all(stage, &[dim, title], 0.8)?;
        kit.listen(stage, TITLE_LISTEN_S, "01_the_question:01_3")?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chapters::draw::{LAT_COLS, counts};
    use crate::model::Q;
    use explainer_kit::Script;

    /// The chapter speaks exactly the script's 01_ lines, in the script's
    /// order (01_3 is the title: no words).
    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("01_"))
            .collect();
        assert_eq!(ids, LINES);
    }

    /// The cold open searches the cell that holds one dot in eight (E5),
    /// with the search of `F-R.model.seeing`, and puts out 833 of its 1,250
    /// dots (two thirds of them, in whole dots).
    #[test]
    fn the_cold_open_searches_the_hottest_cell_with_the_models_seeing() {
        let probe = cold_open_probe().unwrap();
        assert_eq!(y1968::name(probe.row, probe.col), "E5");
        let f = crate::facts::facts().unwrap();
        assert_eq!(f.get("F-R.E5"), Some("1250"));
        assert_eq!(f.get("F-R.model.seeing"), Some("2/3"));
        assert_eq!(probe.seeing, Q::lowest(2, 3));
        let c = counts(&y1968::POINTS, &[probe]);
        assert_eq!(c.out, vec![833]);
        assert_eq!(c.lit[1][probe.row * LAT_COLS + probe.col], 1250 - 833);
    }

    /// The sentences the chapter sets are the narration's own.
    #[test]
    fn the_subject_line_is_three_sentences_and_the_question_is_last() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let text = script.text("01_1").unwrap();
        assert_eq!(text.matches(". ").count() + text.matches("? ").count(), 2);
        assert!(text.ends_with('?'));
        assert_eq!(SENTENCE_Y.len(), 3);
    }
}
