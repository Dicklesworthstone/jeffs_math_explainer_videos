//! Chapter 05, a search can miss: the made-up sea with its hundred dots
//! and its blue chances (05_1); cell A is a canyon and a search of it sees
//! one time in four, a flat cell two in three, each shown as a spare dot
//! crossed by the strip (05_2); the third slot opens for the chance of a
//! find (05_3); the "your turn" asks which cell gives the better one, and
//! the answer lands at zero (05_4); the dots a search would see take
//! rings and the gold numbers land (05_5); and the three slots read as one
//! sum (05_6).
//!
//! Every ring is a dot, every number a count of them: A's ten rings are a
//! quarter of its forty, B's twenty are two thirds of its thirty.

use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    CARRY_HOLD_S, QuizFrame, Sea, Strip, fade_from_black, fade_in_all, fade_out_all, fail, find, flat, in_cell,
    line_style, map, pause, percent_tex, say, sea_dots, sea_rect, sea_slot, sea_split, search, tween_linear,
};
use crate::model::{Q, sea};

pub struct ASearchCanMiss {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 6] = ["05_1", "05_2", "05_3", "05_4", "05_5", "05_6"];

// the phrases the pictures land on (a test finds each in its line)
const P_CANYON: &str = "Cell A is a canyon";
const P_ONE_IN_FOUR: &str = "A search there sees";
const P_FLAT: &str = "In the flat cells";
const P_BETTER: &str = "So ask a better question";
const P_QUARTER: &str = "A quarter of forty";
const P_TWO_THIRDS: &str = "Two thirds of thirty";
const P_TWICE: &str = "Twice the chance";
const P_TIMES: &str = "The chance of a find is";

// fixed words
const FIND_WORDS: &str = "chance of a find";
const SUM_THERE: &str = "chance it is there";
const SUM_SEEING: &str = "× chance of seeing it";
const SUM_FIND: &str = "= chance of a find";
const DETECTION: &str = "detection probability";
const DETECTION_WORDS: &str = "the Coast Guard\u{2019}s names: probability of success = containment × detection";
/// A spare dot's radius against a sea dot's.
const SPARE_SCALE: f64 = 1.4;
const SPARE_RING_R: f64 = 0.27;
/// One pass of the strip over a cell in a trial, and the pause on a catch.
const PASS_S: f64 = 0.5;
const CATCH_S: f64 = 0.35;
/// Where the sum reads, right of the sea.
const SUM_X: f64 = 3.3;
const SUM_Y: [f64; 3] = [1.7, 1.05, 0.4];
/// Where the "chance of a find" label stands, between the bottom cells.
const FIND_LABEL: [f64; 2] = [-3.45, -2.3];

/// How a trial of a search of seeing `seeing` goes: the passes of the strip
/// (the denominator) and which of them catch the spare dot (the
/// numerator's worth, spread evenly through the passes).
pub fn caught_passes(seeing: Q) -> (usize, Vec<usize>) {
    let text = seeing.to_string();
    let (n, d) = match text.split_once('/') {
        Some((n, d)) => (n.parse::<usize>().unwrap_or(0), d.parse::<usize>().unwrap_or(1)),
        None => (text.parse::<usize>().unwrap_or(0), 1),
    };
    let caught = (0..n).map(|i| ((2 * i + 1) * d) / (2 * n)).collect();
    (d, caught)
}

/// "1 in 4": a seeing chance as the narration says it.
pub fn one_in(seeing: Q) -> String {
    let (passes, caught) = caught_passes(seeing);
    format!("{} in {passes}", caught.len())
}

/// A trial in `cell`: a spare dot (not one of the hundred), the strip
/// crossing the cell once for each pass, and a teal ring catching the dot
/// on the passes that see it.
fn trial(stage: &mut Stage<'_>, cell: usize, seeing: Q) -> fmn::Result<()> {
    let (x0, y0, x1, y1) = sea_rect(cell);
    let at = v((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    let spare = stage.arena_mut().add(
        Dot::new()
            .point(at)
            .radius(super::draw::SEA_DOT_R * SPARE_SCALE)
            .style(flat(Palette::ink(), 1.0))
            .build(),
    );
    fade_in_all(stage, &[spare], 0.3)?;
    let strip = Strip::build(stage, (x0, y0, x1, y1), 0.1 * (x1 - x0))?;
    for m in strip.mobs() {
        stage.add_to_scene(m)?;
    }
    let (passes, caught) = caught_passes(seeing);
    for k in 0..passes {
        stage
            .arena_mut()
            .set_tracker_value(strip.phase, 0.0)
            .map_err(|e| fail(format!("the trial's strip: {e:?}")))?;
        stage.play(tween_linear(strip.phase, 1.0, PASS_S)?)?;
        if caught.contains(&k) {
            let ring = stage.arena_mut().add(
                Circle::new()
                    .radius(SPARE_RING_R)
                    .style(line_style(search(), 4.0))
                    .build()
                    .moved_to(at),
            );
            fade_in_all(stage, &[ring], 0.12)?;
            pause(stage, CATCH_S)?;
            fade_out_all(stage, &[ring], 0.2)?;
        }
    }
    fade_out_all(stage, &[spare, strip.wash, strip.bar], 0.3)?;
    Ok(())
}

impl SceneConstruct for ASearchCanMiss {
    fn name(&self) -> &str {
        "05_a_search_can_miss"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let facts = crate::facts::facts().map_err(fail)?;
        let seeing = sea::seeing();
        let prior = sea::prior();
        let finds = sea::find(&prior);
        let lit: [u32; 4] = prior.map(|p| (p * Q::int(sea::DOTS)).fixed(0).parse().unwrap_or(0));

        // 05_1: back to the made-up sea, its hundred dots and blue chances
        let line = say(&kit, stage, LINES[0])?;
        let sea_map = Sea::build(&kit, stage, lit)?;
        let dots = sea_dots(stage, sea_split());
        for m in sea_map.mobs() {
            stage.add_to_scene(m)?;
        }
        stage.add_to_scene(sea_map.tag)?;
        for d in &dots {
            stage.add_to_scene(d.dot)?;
        }
        for c in 0..4 {
            sea_map.percent_readout(&kit, stage, c, 28.0, map(), sea_slot(c, 0));
        }
        // the percents are recounted every frame and cannot fade in, so the
        // whole picture comes up from the dark under one veil
        fade_from_black(stage, 0.7)?;
        line.wait_end(stage)?;

        // 05_2: A is a canyon, seen one time in four; the flat cells two in
        // three; each a spare dot crossed by the strip
        let line = say(&kit, stage, LINES[1])?;
        line.wait_for(stage, P_CANYON)?;
        fade_in_all(stage, &[sea_map.canyon], 0.8)?;
        line.wait_for(stage, P_ONE_IN_FOUR)?;
        trial(stage, 0, seeing[0])?;
        let note_a = kit.text(stage, &one_in(seeing[0]), 26.0, search(), sea_slot(0, 1))?;
        fade_in_all(stage, &[note_a], 0.4)?;
        line.wait_for(stage, P_FLAT)?;
        trial(stage, 1, seeing[1])?;
        let mut flat_notes = Vec::new();
        for (c, &s) in seeing.iter().enumerate().skip(1) {
            flat_notes.push(kit.text(stage, &one_in(s), 26.0, search(), sea_slot(c, 1))?);
        }
        fade_in_all(stage, &flat_notes, 0.4)?;
        line.wait_end(stage)?;

        // 05_3: the third slot opens, empty and gold
        let line = say(&kit, stage, LINES[2])?;
        line.wait_for(stage, P_BETTER)?;
        let mut open = Vec::new();
        for c in 0..4 {
            open.push(kit.text(stage, "?", 28.0, find(), sea_slot(c, 2))?);
        }
        let label = kit.text(stage, FIND_WORDS, 24.0, find(), v(FIND_LABEL[0], FIND_LABEL[1]))?;
        let mut ins = open.clone();
        ins.push(label);
        fade_in_all(stage, &ins, 0.5)?;
        line.wait_end(stage)?;

        // 05_4: your turn: which cell gives the better chance of a find? B's
        // border turns gold as the answer lands and the frame lifts
        let line = say(&kit, stage, LINES[3])?;
        let frame = QuizFrame::your_turn(&kit, stage, &line)?;
        let answer = facts
            .get("F-Q.first_search.answer")
            .ok_or_else(|| fail("no fact F-Q.first_search.answer".into()))?;
        let best = sea::CELLS
            .iter()
            .position(|c| *c == answer)
            .ok_or_else(|| fail(format!("the quiz's answer {answer} is not a cell")))?;
        let mut anims = frame.lift(stage, 0.5)?;
        let gold = sea_map.borders[best]
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.45),
                ..AnimateArgs::default()
            })?
            .set_stroke(Some(find()), Some(5.0), Some(1.0))?;
        anims.push(stage.prepare(gold)?);
        stage.play_prepared(anims)?;

        // 05_5: a quarter of A's forty and two thirds of B's thirty take a
        // ring; the gold numbers land: 10%, 20%, then C's and D's thirds
        let line = say(&kit, stage, LINES[4])?;
        let mut gone = vec![open[0]];
        let rings_for = |cell: usize| -> Vec<Mob> {
            let mine = in_cell(&dots, cell);
            let n = (Q::int(i128::from(lit[cell])) * seeing[cell])
                .fixed(0)
                .parse::<usize>()
                .unwrap_or(0);
            mine.iter().take(n).map(|d| d.ring).collect()
        };
        line.wait_for(stage, P_QUARTER)?;
        let mut ins = rings_for(0);
        ins.push(kit.tex(stage, &percent_tex(finds[0]), 30.0, find(), &[], sea_slot(0, 2))?);
        fade_out_all(stage, &gone, 0.2)?;
        fade_in_all(stage, &ins, 0.6)?;
        line.wait_for(stage, P_TWO_THIRDS)?;
        gone = vec![open[1]];
        let mut ins = rings_for(1);
        ins.push(kit.tex(stage, &percent_tex(finds[1]), 30.0, find(), &[], sea_slot(1, 2))?);
        fade_out_all(stage, &gone, 0.2)?;
        fade_in_all(stage, &ins, 0.6)?;
        line.wait_for(stage, P_TWICE)?;
        gone = vec![open[2], open[3]];
        let mut ins = Vec::new();
        for c in [2usize, 3] {
            ins.push(kit.tex(stage, &percent_tex(finds[c]), 30.0, find(), &[], sea_slot(c, 2))?);
        }
        fade_out_all(stage, &gone, 0.2)?;
        fade_in_all(stage, &ins, 0.6)?;
        line.wait_end(stage)?;

        // 05_6: the three slots read as one sum
        let line = say(&kit, stage, LINES[5])?;
        line.wait_for(stage, P_TIMES)?;
        let sum = [
            kit.text(stage, SUM_THERE, 32.0, map(), v(SUM_X, SUM_Y[0]))?,
            kit.text(stage, SUM_SEEING, 32.0, search(), v(SUM_X, SUM_Y[1]))?,
            kit.text(stage, SUM_FIND, 32.0, find(), v(SUM_X, SUM_Y[2]))?,
        ];
        fade_in_all(stage, &sum, 0.7)?;
        line.wait_end(stage)?;
        kit.hold(stage)?;
        let third = lower_third(&kit, stage, DETECTION, DETECTION_WORDS)?;
        fade_in_all(stage, &[third], 0.5)?;
        pause(stage, CARRY_HOLD_S - 0.5)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// (line, phrase): every phrase a picture lands on.
    const PHRASES: [(&str, &str); 7] = [
        ("05_2", P_CANYON),
        ("05_2", P_ONE_IN_FOUR),
        ("05_2", P_FLAT),
        ("05_3", P_BETTER),
        ("05_5", P_QUARTER),
        ("05_5", P_TWO_THIRDS),
        ("05_5", P_TWICE),
    ];
    use crate::chapters::draw::{QUIZ_FRAME, QUIZ_LINES_Y, SEA_CELL, percent_tex};
    use explainer_kit::Script;

    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("05_"))
            .collect();
        assert_eq!(ids, LINES);
    }

    #[test]
    fn every_phrase_the_pictures_land_on_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in PHRASES {
            let text = script.text(id).unwrap();
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
        // 05_6's sum is introduced by this phrase, which `Spoken::at` needs
        assert!(script.text("05_6").unwrap().contains(P_TIMES));
    }

    /// 05_2's trials: A is crossed four times and seen once, a flat cell
    /// three times and seen twice; the catches are distinct passes.
    #[test]
    fn a_trial_is_the_chance_as_passes_and_catches() {
        let s = sea::seeing();
        assert_eq!(caught_passes(s[0]), (4, vec![2]));
        assert_eq!(caught_passes(s[1]), (3, vec![0, 2]));
        assert_eq!(one_in(s[0]), "1 in 4");
        assert_eq!(one_in(s[1]), "2 in 3");
        for q in s {
            let (passes, caught) = caught_passes(q);
            assert!(caught.iter().all(|&k| k < passes));
            let mut sorted = caught.clone();
            sorted.dedup();
            assert_eq!(sorted, caught, "no pass catches twice");
        }
        // the trials fit the lines they play in: seeing is the fraction
        assert_eq!(caught_passes(Q::lowest(1, 1)), (1, vec![0]));
        assert_eq!(caught_passes(Q::ZERO).1, Vec::<usize>::new());
    }

    /// The numbers the chapter shows are the facts: the seeing, the chance
    /// of a find as the gold slot prints it, the quiz's answer and the
    /// count of dots each ring set holds.
    #[test]
    fn what_the_slots_show_is_what_the_facts_say() {
        let f = crate::facts::facts().unwrap();
        assert_eq!(f.get("F-M.seeing"), Some("1/4,2/3,2/3,2/3"));
        assert_eq!(f.get("F-M.find.fraction"), Some("1/10,1/5,2/15,1/15"));
        assert_eq!(f.get("F-M.find.percent"), Some("10.0,20.0,13.3,6.7"));
        assert_eq!(f.get("F-M.first.over_likeliest"), Some("2"));
        assert_eq!(f.get("F-Q.first_search.answer"), Some("B"));
        let finds = sea::find(&sea::prior());
        let shown: Vec<String> = finds.iter().map(|&q| percent_tex(q)).collect();
        assert_eq!(shown, ["10\\%", "20\\%", "13\\frac{1}{3}\\%", "6\\frac{2}{3}\\%"]);
        // the rings: a quarter of A's forty, two thirds of B's thirty
        let (p, d) = (sea::prior(), sea::seeing());
        let rings: Vec<String> = (0..2).map(|c| (p[c] * Q::int(sea::DOTS) * d[c]).fixed(0)).collect();
        assert_eq!(rings, ["10", "20"]);
    }

    /// 05_6: "the best cell to search is not always the likeliest". At its
    /// edge: where every cell is seen equally well the likeliest cell is
    /// the best; in the canyon's sea it is not, and the best is B.
    #[test]
    fn the_best_cell_is_not_always_the_likeliest() {
        let p = sea::prior();
        let argmax = |v: [Q; 4]| (0..4).max_by_key(|&i| (v[i], std::cmp::Reverse(i))).unwrap();
        let equal = [Q::lowest(2, 3); 4];
        let flat_finds: [Q; 4] = std::array::from_fn(|i| p[i] * equal[i]);
        assert_eq!(argmax(flat_finds), argmax(p));
        let finds = sea::find(&p);
        assert_eq!(argmax(p), 0);
        assert_eq!(argmax(finds), 1);
        assert_eq!(finds[1] / finds[0], Q::int(2), "twice the chance");
    }

    /// The sum reads right of the sea, clear of it and of the frame, and
    /// the "chance of a find" label sits between the bottom cells, above
    /// the question's lines.
    #[test]
    fn the_sum_and_the_label_sit_clear_of_the_sea_and_the_question() {
        assert!(
            SUM_X - 2.6 > sea_rect(1).2,
            "the sum's widest line starts right of the sea"
        );
        assert!(SUM_X + 2.6 < QUIZ_FRAME.0);
        assert!(SUM_Y.windows(2).all(|w| w[0] - w[1] >= 0.5), "rows do not touch");
        // the label clears the question's first line and sits between the
        // bottom cells; a trial's ring fits inside a cell (constants,
        // checked as the crate builds)
        const {
            assert!(FIND_LABEL[1] - 0.2 > QUIZ_LINES_Y[0] + 0.2);
            assert!(SPARE_RING_R < SEA_CELL.1 / 2.0);
        }
        assert!(FIND_LABEL[0] > sea_rect(2).0 && FIND_LABEL[0] < sea_rect(3).2);
    }
}
