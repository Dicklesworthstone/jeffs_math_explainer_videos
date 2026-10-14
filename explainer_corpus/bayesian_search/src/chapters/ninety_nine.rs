//! Chapter 02, ninety-nine: the Navy had to ask that question for real. A
//! submarine, ninety-nine men, a track across the Atlantic and a line of
//! dates (02_1); due on the twenty-seventh, did not arrive, nine days of
//! searching as if the crew might be saved (02_2); they could not be, she
//! had gone down five days earlier, and the stage clears to a ring and
//! "99 men", in a silence (02_3); where do you look (02_4); not everywhere:
//! the grid, and the strip the sled saw, two hundred feet wide, twenty-six
//! passes to a cell (02_5); one pass over the whole patch is thousands of
//! miles, more than four times all the towing before the find (02_6); so
//! where would you look first (02_7).
//!
//! The truth comes in the third line, as the order says, and nothing
//! sounds for two and a half seconds after it. The 99 are a number in ink
//! on a cleared stage: no dot, ring of light or map cell stands for a man.

use explainer_kit::{Kit, Palette, thousands, v};
use fmn::prelude::*;

use super::draw::{
    CAPTION_Y, NAVY_CENTRE, NAVY_ROW_H, NavyGrid, balanced_lines, carry_pause, carry_question, fade_in_all,
    fade_in_lagged, fade_out_all, faded_line, fail, flat, line_style, map, pause, say, search, text_lines, true_place,
    tween_linear,
};
use crate::facts::cited;
use crate::model::{days, y1968};

pub struct NinetyNine {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 7] = ["02_1", "02_2", "02_3", "02_4", "02_5", "02_6", "02_7"];

// the phrases the pictures land on (a test finds each in its line)
const P_SCORPION: &str = "The U S S Scorpion";
const P_DUE: &str = "She was due at Norfolk";
const P_NOT_ARRIVE: &str = "did not arrive";
const P_NINE_DAYS: &str = "For nine days";
const P_OCEAN: &str = "searched the ocean";
const P_FIVE_DAYS: &str = "five days before she was due";
const P_ALL: &str = "All ninety-nine men";
const P_NOBODY: &str = "Nobody could be sure";
const P_QUESTION: &str = "the first question of any search";
const P_WHERE: &str = "Where do you look?";
const P_NOT_EVERYWHERE: &str = "Not everywhere";
const P_STRIP: &str = "It saw a strip";
const P_ONE_PASS: &str = "One pass over the whole patch";
const P_SEVERAL: &str = "several times all the open-sea searching";

// fixed words
const SCORPION_LABEL: &str = "USS Scorpion (SSN-589), May 1968";
const AZORES: &str = "the Azores";
const NORFOLK: &str = "Norfolk, Virginia";
const DUE: &str = "due";
const WENT_DOWN: &str = "went down";
const DAYS: &str = "days";
const SEARCHERS: &str = "aircraft and dozens of ships";
const MEN: &str = "men";
const MILES_OFF: &str = "miles southwest of the Azores";
const ABOUT: &str = "about";
const CELLS_BY: &str = "cells, each a mile by";
const OF_A_MILE: &str = "of a mile";
const AFTER_WORDS: &str = "strip as measured afterwards:";
const CAMERA: &str = "camera";
const FEET: &str = "ft";
const MAGNETOMETER: &str = "magnetometer";
const ONE_CELL: &str = "one cell:";
const SEA_MILES_OF_TRACK: &str = "sea miles of track (model: perfect strips)";
const WIDE: &str = "wide";
const TRACK_WORDS: &str = "broad-area search track, June to October";
const SEA_MILES: &str = "sea miles towed";
const ALL_CELLS: &str = "all";
const ALL_CELLS_WORDS: &str = "cells:";
const ALL_MILES: &str = "sea miles (6,076 feet each)";
const MODEL_NOTE: &str =
    "Model: one sensor\u{2019}s 200-foot strip; the camera and the magnetic detector together saw somewhat wider";
/// The model note's line length: 18 pt runs 0.081 of a unit a character, so
/// 56 characters fill the column's 4.8 without shrinking.
const MODEL_NOTE_CHARS: usize = 56;

/// The Atlantic, drawn small: where the Azores and Norfolk stand and the
/// patch southwest of the Azores.
const AZORES_AT: [f64; 2] = [4.4, 1.5];
const NORFOLK_AT: [f64; 2] = [-4.6, 0.4];
const PATCH_AT: [f64; 2] = [3.3, 0.45];
const PATCH_SIZE: [f64; 2] = [1.1, 0.9];
/// The date line: from 21 May (her last message) to 5 June, in [`LINE_DAYS`]
/// days across [`LINE_X`], at height [`LINE_Y`].
const LINE_X: [f64; 2] = [-5.5, 5.5];
const LINE_Y: f64 = -2.35;
const LINE_DAYS: f64 = 15.0;
/// The ring of the 99, and the inset cell of 02_5.
pub const RING_AT: [f64; 2] = [0.0, 0.6];
pub const RING_R: f64 = 0.5;
/// How far under the ring's centre the number stands.
pub const NUMBER_DROP: f64 = 1.3;
const INSET_AT: [f64; 2] = [4.4, 0.6];
const INSET_W: f64 = 2.2;
/// The right-hand column of 02_5 and 02_6, its middle and the widest a line
/// of it runs (text is shrunk to fit).
const COLUMN_X: f64 = 4.4;
const COLUMN_ROOM: f64 = 4.8;

/// How many days after 21 May a date falls (the line's unit).
pub fn day_offset(date: (i64, i64, i64)) -> f64 {
    days::between(cited::LAST_MESSAGE, date) as f64
}

/// Where a day offset stands along the date line.
pub fn day_x(offset: f64) -> f64 {
    LINE_X[0] + (LINE_X[1] - LINE_X[0]) * offset / LINE_DAYS
}

/// "27 May": a date of this chapter in words (only May and June occur).
pub fn date_words(date: (i64, i64, i64)) -> String {
    let month = match date.1 {
        5 => "May",
        6 => "June",
        _ => "?",
    };
    format!("{} {month}", date.2)
}

/// The number of whole passes of a strip `strip_feet` wide that fill a cell
/// (its east-west side, in feet, over the strip, rounded up).
pub fn passes_to_fill(strip_feet: u32) -> usize {
    let feet = y1968::CELL_MILES.1.f64() * 6076.1;
    (feet / f64::from(strip_feet)).ceil() as usize
}

/// A fact's value, or the error that names it.
fn fact(facts: &explainer_kit::Facts, key: &str) -> fmn::Result<String> {
    facts
        .get(key)
        .map(str::to_owned)
        .ok_or_else(|| fail(format!("no fact {key}")))
}

impl SceneConstruct for NinetyNine {
    fn name(&self) -> &str {
        "02_ninety_nine"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let facts = crate::facts::facts().map_err(fail)?;
        let dim = Palette::dim();
        let ink = Palette::ink();

        // the Atlantic: two places and the track between them
        let azores = stage.arena_mut().add(
            Dot::new()
                .point(v(AZORES_AT[0], AZORES_AT[1]))
                .radius(0.09)
                .style(flat(dim, 1.0))
                .build(),
        );
        let norfolk = stage.arena_mut().add(
            Dot::new()
                .point(v(NORFOLK_AT[0], NORFOLK_AT[1]))
                .radius(0.09)
                .style(flat(dim, 1.0))
                .build(),
        );
        let azores_label = kit.text(stage, AZORES, 24.0, dim, v(AZORES_AT[0], AZORES_AT[1] + 0.4))?;
        let norfolk_label = kit.text(stage, NORFOLK, 24.0, dim, v(NORFOLK_AT[0], NORFOLK_AT[1] + 0.4))?;
        let track = stage.arena_mut().add(
            polyline(vec![v(AZORES_AT[0], AZORES_AT[1]), v(NORFOLK_AT[0], NORFOLK_AT[1])])
                .style(faded_line(dim, 2.0, 0.7))
                .build(),
        );
        let atlantic = [track, azores, norfolk, azores_label, norfolk_label];

        // the date line: a rule, a tick a day, and three named days
        let rule = stage.arena_mut().add(
            polyline(vec![v(LINE_X[0], LINE_Y), v(LINE_X[1], LINE_Y)])
                .style(line_style(dim, 2.5))
                .build(),
        );
        let mut date_mobs = vec![rule];
        for d in 0..=(LINE_DAYS as usize) {
            let x = day_x(d as f64);
            date_mobs.push(
                stage.arena_mut().add(
                    polyline(vec![v(x, LINE_Y - 0.07), v(x, LINE_Y + 0.07)])
                        .style(line_style(dim, 2.0))
                        .build(),
                ),
            );
        }
        let due_x = day_x(day_offset(cited::DUE));
        let sank_x = day_x(day_offset(cited::SANK));
        let presumed_x = day_x(day_offset(cited::PRESUMED_LOST));
        let due_label = kit.text(stage, &date_words(cited::DUE), 22.0, dim, v(due_x, LINE_Y - 0.4))?;
        let presumed_label = kit.text(
            stage,
            &date_words(cited::PRESUMED_LOST),
            22.0,
            dim,
            v(presumed_x, LINE_Y - 0.4),
        )?;
        let due_word = kit.text(stage, DUE, 24.0, ink, v(due_x, LINE_Y + 0.4))?;

        // 02_1: the submarine, heading home, with ninety-nine men aboard
        let line = say(&kit, stage, LINES[0])?;
        fade_in_all(stage, &atlantic, 0.7)?;
        let mut line_in = date_mobs.clone();
        line_in.extend([due_label, presumed_label]);
        fade_in_lagged(stage, &line_in, 1.0, 0.6)?;
        line.wait_for(stage, P_SCORPION)?;
        let point = stage.arena_mut().add(
            Dot::new()
                .point(v(AZORES_AT[0], AZORES_AT[1]))
                .radius(0.13)
                .style(flat(true_place(), 1.0))
                .build(),
        );
        let caption = kit.text(stage, SCORPION_LABEL, 26.0, dim, v(0.0, CAPTION_Y))?;
        fade_in_all(stage, &[point, caption], 0.4)?;
        // she is heading home: the point moves a good way along the track
        let along = |t: f64| {
            v(
                AZORES_AT[0] + (NORFOLK_AT[0] - AZORES_AT[0]) * t,
                AZORES_AT[1] + (NORFOLK_AT[1] - AZORES_AT[1]) * t,
            )
        };
        let a = point
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some((line.end() - line.at(P_SCORPION)?).max(1.0)),
                ..AnimateArgs::default()
            })?
            .move_to(along(0.62), ORIGIN)?;
        stage.play(a)?;
        line.wait_end(stage)?;

        // 02_2: due on the twenty-seventh, and did not arrive; nine days of
        // searching, one mark for each along the date line
        let line = say(&kit, stage, LINES[1])?;
        line.wait_for(stage, P_DUE)?;
        fade_in_all(stage, &[due_word], 0.4)?;
        line.wait_for(stage, P_NOT_ARRIVE)?;
        fade_out_all(stage, &[point], 0.5)?;
        line.wait_for(stage, P_NINE_DAYS)?;
        let searched_days = days::between(cited::DUE, cited::PRESUMED_LOST) as usize;
        let mut marks = Vec::new();
        for k in 1..=searched_days {
            let m = Rectangle::new()
                .width(0.22)
                .height(0.22)
                .style(flat(search(), 1.0))
                .build()?
                .moved_to(v(day_x(day_offset(cited::DUE) + k as f64), LINE_Y + 0.28));
            marks.push(stage.arena_mut().add(m));
        }
        fade_in_lagged(stage, &marks, (line.end() - line.at(P_NINE_DAYS)?).max(1.5), 0.8)?;
        line.wait_for(stage, P_OCEAN)?;
        let searchers = kit.text(stage, SEARCHERS, 26.0, dim, v(0.0, -1.6))?;
        fade_in_all(stage, &[searchers], 0.5)?;
        line.wait_end(stage)?;

        // 02_3: they could not be: she had gone down five days before she was
        // due; all ninety-nine men were lost; the stage clears to a ring and
        // the number, and nothing sounds for two and a half seconds
        let line = say(&kit, stage, LINES[2])?;
        line.wait_for(stage, P_FIVE_DAYS)?;
        let sank_label = kit.text(stage, &date_words(cited::SANK), 22.0, ink, v(sank_x, LINE_Y - 0.4))?;
        let went_down = kit.text(stage, WENT_DOWN, 24.0, ink, v(sank_x, LINE_Y + 0.4))?;
        let gap = days::between(cited::SANK, cited::DUE);
        let bracket = stage.arena_mut().add(
            polyline(vec![
                v(sank_x, LINE_Y + 0.85),
                v(sank_x, LINE_Y + 0.95),
                v(due_x, LINE_Y + 0.95),
                v(due_x, LINE_Y + 0.85),
            ])
            .style(line_style(ink, 2.5))
            .build(),
        );
        let gap_label = kit.text(
            stage,
            &format!("{gap} {DAYS}"),
            26.0,
            ink,
            v((sank_x + due_x) / 2.0, LINE_Y + 1.3),
        )?;
        let sank_in = [sank_label, went_down, bracket, gap_label];
        // the search marks fade as the line runs back to the day she sank
        let mut marks_gone: Vec<Mob> = marks.clone();
        marks_gone.push(searchers);
        fade_out_all(stage, &marks_gone, 0.5)?;
        fade_in_all(stage, &sank_in, 0.6)?;
        line.wait_for(stage, P_ALL)?;
        // the stage clears to a small ring and the number
        let mut clearing: Vec<Mob> = atlantic.to_vec();
        clearing.extend(&date_mobs);
        clearing.extend([due_label, presumed_label, due_word, caption]);
        clearing.extend(sank_in);
        fade_out_all(stage, &clearing, 0.8)?;
        let ring = stage.arena_mut().add(
            Circle::new()
                .radius(RING_R)
                .style(line_style(true_place(), 4.0))
                .build()
                .moved_to(v(RING_AT[0], RING_AT[1])),
        );
        let ninety_nine = kit.text(
            stage,
            &format!("{} {MEN}", cited::CREW),
            64.0,
            ink,
            v(RING_AT[0], RING_AT[1] - NUMBER_DROP),
        )?;
        fade_in_all(stage, &[ring, ninety_nine], 0.8)?;
        line.wait_end(stage)?;
        // two and a half seconds in which nothing sounds: no music breath
        kit.mark(stage, "silence:ninety_nine");
        pause(stage, 2.5)?;
        kit.mark(stage, "silence_end:ninety_nine");

        // 02_4: nobody could be sure of that yet; where do you look? The
        // ring goes out; a patch of sea southwest of the Azores is outlined
        let line = say(&kit, stage, LINES[3])?;
        line.wait_for(stage, P_NOBODY)?;
        fade_out_all(stage, &[ring, ninety_nine], 0.6)?;
        line.wait_for(stage, P_QUESTION)?;
        fade_in_all(stage, &atlantic, 0.6)?;
        line.wait_for(stage, P_WHERE)?;
        let patch = Rectangle::new()
            .width(PATCH_SIZE[0])
            .height(PATCH_SIZE[1])
            .style(faded_line(map(), 4.0, 1.0))
            .build()?
            .moved_to(v(PATCH_AT[0], PATCH_AT[1]));
        let patch = stage.arena_mut().add(patch);
        let off = kit.text(
            stage,
            &format!("{ABOUT} {} {MILES_OFF}", cited::MILES_FROM_AZORES),
            24.0,
            dim,
            v(1.2, -1.0),
        )?;
        fade_in_all(stage, &[patch, off], 0.7)?;
        line.wait_end(stage)?;

        // 02_5: not everywhere: the patch becomes the grid; the sled saw a
        // strip about two hundred feet wide, twenty-six of them to a cell
        let line = say(&kit, stage, LINES[4])?;
        line.wait_for(stage, P_NOT_EVERYWHERE)?;
        let mut leaving: Vec<Mob> = atlantic.to_vec();
        leaving.extend([patch, off]);
        fade_out_all(stage, &leaving, 0.5)?;
        let grid = NavyGrid::build(stage, NAVY_CENTRE, NAVY_ROW_H)?;
        fade_in_lagged(stage, &grid.lines, 1.0, 0.5)?;
        let by = kit.text(
            stage,
            &format!(
                "{} by {} {CELLS_BY} {} {OF_A_MILE}",
                y1968::GRID_ROWS.len(),
                y1968::GRID_COLS.len(),
                y1968::CELL_MILES.1.fixed(2)
            ),
            22.0,
            dim,
            v(NAVY_CENTRE[0], NAVY_CENTRE[1] - grid.size().1 / 2.0 - 0.35),
        )?;
        fade_in_all(stage, &[by], 0.5)?;
        line.wait_for(stage, P_STRIP)?;
        // one cell, drawn large, and the strips that fill it side by side
        let strip_feet: u32 = fact(&facts, "F-H.strip.feet")?
            .parse()
            .map_err(|_| fail("F-H.strip.feet".into()))?;
        // the passes that fill a cell, as the strip and the cell's width say
        // (a test pins it to `F-H.once.passes_per_cell`)
        let passes = passes_to_fill(strip_feet);
        let cell_h = INSET_W / (y1968::CELL_MILES.1.f64() / y1968::CELL_MILES.0.f64());
        let frame = Rectangle::new()
            .width(INSET_W)
            .height(cell_h)
            .style(faded_line(dim, 2.5, 1.0))
            .build()?
            .moved_to(v(INSET_AT[0], INSET_AT[1]));
        let frame = stage.arena_mut().add(frame);
        let mut strips = vec![frame];
        let w = INSET_W / passes as f64;
        for k in 0..passes {
            let r = Rectangle::new()
                .width(w)
                .height(cell_h)
                .style(flat(search(), if k % 2 == 0 { 0.8 } else { 0.5 }))
                .build()?
                .moved_to(v(INSET_AT[0] - INSET_W / 2.0 + w * (k as f64 + 0.5), INSET_AT[1]));
            strips.push(stage.arena_mut().add(r));
        }
        // two lines at full size, not one shrunk to the column (the glyph
        // pages: sixty-four characters at 20 pt are 5.8 units, the room is 4.8)
        let after = text_lines(
            &kit,
            stage,
            &[
                AFTER_WORDS.to_owned(),
                format!(
                    "{CAMERA} {} {FEET}, {MAGNETOMETER} {} {FEET}",
                    cited::SWEEP_AFTER.0,
                    cited::SWEEP_AFTER.1
                ),
            ],
            20.0,
            dim,
            INSET_AT[0],
            INSET_AT[1] + cell_h / 2.0 + 0.7,
            0.32,
        )?;
        let per_cell = fact(&facts, "F-H.once.per_cell")?;
        let one_cell = kit.text(
            stage,
            &format!("{ONE_CELL} {per_cell} {SEA_MILES_OF_TRACK}"),
            20.0,
            dim,
            v(INSET_AT[0], INSET_AT[1] - cell_h / 2.0 - 0.4),
        )?;
        super::draw::fit_width(stage, one_cell, COLUMN_ROOM);
        let strip_wide = kit.text(
            stage,
            &format!("{strip_feet} {FEET} {WIDE}"),
            22.0,
            Palette::ink(),
            v(INSET_AT[0], INSET_AT[1] - cell_h / 2.0 - 0.85),
        )?;
        let mut inset = strips.clone();
        inset.extend([after, one_cell, strip_wide]);
        fade_in_lagged(stage, &inset, 1.4, 0.15)?;
        line.wait_end(stage)?;

        // 02_6: one pass over the whole patch is thousands of miles: the
        // strip runs over the grid painting cell by cell, under a counter
        let line = say(&kit, stage, LINES[5])?;
        line.wait_for(stage, P_ONE_PASS)?;
        fade_out_all(stage, &inset, 0.5)?;
        let miles = stage.add_value_tracker(0.0);
        let cells_painted: usize = fact(&facts, "F-H.track.cells")?
            .parse()
            .map_err(|_| fail("F-H.track.cells".into()))?;
        let track_miles = f64::from(cited::TRACK_MILES);
        kit.live_count(stage, miles, 64.0, true_place(), v(COLUMN_X, 1.5))?;
        let counter_words = kit.text(stage, SEA_MILES, 24.0, dim, v(COLUMN_X, 0.85))?;
        let track_words = kit.text(stage, TRACK_WORDS, 20.0, dim, v(COLUMN_X, 0.4))?;
        super::draw::fit_width(stage, track_words, COLUMN_ROOM);
        fade_in_all(stage, &[counter_words, track_words], 0.4)?;
        // the whole run in one move: the cells wash in as the counter runs
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        let painted: Vec<Mob> = grid.washes.iter().take(cells_painted).copied().collect();
        let mut cell_anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &painted {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            cell_anims.push(stage.prepare(a)?);
        }
        let run_s = 3.2;
        let lagged = AnimationGroup::with_lag_ratio(stage.arena_mut(), cell_anims, 0.9)?.with_run_time(run_s);
        anims.push(Box::new(lagged));
        anims.push(stage.prepare(tween_linear(miles, track_miles, run_s)?)?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, P_SEVERAL)?;
        let once_miles: u64 = fact(&facts, "F-H.once.miles")?
            .parse()
            .map_err(|_| fail("F-H.once.miles".into()))?;
        let all = kit.text(
            stage,
            &format!(
                "{ALL_CELLS} {} {ALL_CELLS_WORDS} {} {ALL_MILES}",
                y1968::GRID_ROWS.len() * y1968::GRID_COLS.len(),
                thousands(once_miles)
            ),
            24.0,
            Palette::ink(),
            v(COLUMN_X, -0.4),
        )?;
        super::draw::fit_width(stage, all, COLUMN_ROOM);
        let model = text_lines(
            &kit,
            stage,
            &balanced_lines(MODEL_NOTE, MODEL_NOTE_CHARS),
            18.0,
            dim,
            COLUMN_X,
            -1.15,
            0.3,
        )?;
        fade_in_all(stage, &[all, model], 0.6)?;
        line.wait_end(stage)?;

        // 02_7: so where would you look first? A question to carry, the
        // grid a quarter painted
        let line = say(&kit, stage, LINES[6])?;
        let q = carry_question(&kit, stage, LINES[6])?;
        fade_in_all(stage, &[q], 0.4)?;
        line.wait_end(stage)?;
        carry_pause(&kit, stage)?;
        // chapter 03 opens on this grid, so it stays through the cut
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chapters::draw::QUIZ_FRAME;
    use explainer_kit::Script;

    const PHRASES: [(&str, &str); 14] = [
        ("02_1", P_SCORPION),
        ("02_2", P_DUE),
        ("02_2", P_NOT_ARRIVE),
        ("02_2", P_NINE_DAYS),
        ("02_2", P_OCEAN),
        ("02_3", P_FIVE_DAYS),
        ("02_3", P_ALL),
        ("02_4", P_NOBODY),
        ("02_4", P_QUESTION),
        ("02_4", P_WHERE),
        ("02_5", P_NOT_EVERYWHERE),
        ("02_5", P_STRIP),
        ("02_6", P_ONE_PASS),
        ("02_6", P_SEVERAL),
    ];

    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("02_"))
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
    }

    /// The date line is 15 days from the last message to "presumed lost":
    /// she was due 5 days after she sank and the search ran 9 days after
    /// that, each as the facts count them.
    #[test]
    fn the_date_line_counts_the_days_as_the_facts_do() {
        let f = crate::facts::facts().unwrap();
        assert_eq!(day_offset(cited::PRESUMED_LOST), LINE_DAYS);
        assert_eq!(day_offset(cited::LAST_MESSAGE), 0.0);
        assert_eq!(
            days::between(cited::SANK, cited::DUE).to_string(),
            f.get("F-S.days.sank_to_due").unwrap()
        );
        assert_eq!(days::between(cited::SANK, cited::DUE), 5);
        assert_eq!(days::between(cited::DUE, cited::PRESUMED_LOST), 9);
        assert_eq!(f.get("F-S.due"), Some("1968-05-27"));
        assert_eq!(date_words(cited::DUE), "27 May");
        assert_eq!(date_words(cited::SANK), "22 May");
        assert_eq!(date_words(cited::PRESUMED_LOST), "5 June");
        // the line's ends and its named days are in order along the stage
        let (sank, due, presumed) = (
            day_x(day_offset(cited::SANK)),
            day_x(day_offset(cited::DUE)),
            day_x(day_offset(cited::PRESUMED_LOST)),
        );
        assert!(LINE_X[0] < sank && sank < due && due < presumed && presumed <= LINE_X[1]);
        // the nine search marks (one a day after the 27th) fit before the end
        assert!(day_x(day_offset(cited::DUE) + 9.0) <= LINE_X[1] + 1e-9);
        assert_eq!(f.get("F-S.crew"), Some(cited::CREW.to_string().as_str()));
        assert_eq!(f.get("F-S.miles_from_azores"), Some("400"));
    }

    /// 02_5: a strip of 200 feet needs 26 passes to cross a cell 0.84 of a
    /// mile wide (25.5 rounded up) and one cell is 26 sea miles of track;
    /// all 180 cells are 4,594 and the 1,026 the search towed is 40 cells.
    #[test]
    fn the_strip_the_passes_and_the_miles_are_the_facts() {
        let f = crate::facts::facts().unwrap();
        assert_eq!(passes_to_fill(200), 26);
        assert_eq!(f.get("F-H.once.passes_per_cell"), Some("26"));
        assert_eq!(f.get("F-H.strip.feet"), Some("200"));
        assert_eq!(f.get("F-H.once.per_cell"), Some("26"));
        assert_eq!(f.get("F-H.once.miles"), Some("4594"));
        assert_eq!(f.get("F-H.track.miles"), Some("1026"));
        assert_eq!(f.get("F-H.track.cells"), Some("40"));
        assert_eq!(f.get("F-H.once.over_track"), Some("4.5"));
        assert_eq!(y1968::GRID_ROWS.len() * y1968::GRID_COLS.len(), 180);
        assert_eq!(thousands(4594u64), "4,594");
        assert_eq!(cited::SWEEP_AFTER, (185, 220, 0));
        // the strips fill the inset cell exactly
        let w = INSET_W / 26.0;
        assert!((w * 26.0 - INSET_W).abs() < 1e-12);
    }

    /// The glyph pages showed the model note squeezed to 11 pt in a 4.8-unit
    /// column. Each text of the column is now either wrapped or shrunk by
    /// no more than a tenth, and none is set under 18 pt.
    #[test]
    fn the_columns_texts_stay_legible() {
        use super::super::draw::CHAR_UNITS_PER_POINT;
        let rows: [(String, f64); 3] = [
            (format!("{ONE_CELL} 26 {SEA_MILES_OF_TRACK}"), 20.0),
            (
                format!("{ALL_CELLS} 180 {ALL_CELLS_WORDS} {} {ALL_MILES}", thousands(4594u64)),
                24.0,
            ),
            (TRACK_WORDS.to_owned(), 20.0),
        ];
        for (text, size) in rows {
            let natural = text.chars().count() as f64 * size * CHAR_UNITS_PER_POINT;
            let set = size * (COLUMN_ROOM / natural).min(1.0);
            assert!(set >= 18.0 && set >= 0.9 * size, "{text:?} is set at {set:.1} pt");
        }
        // the model note and the "afterwards" label are lines, not shrunk
        let note = balanced_lines(MODEL_NOTE, MODEL_NOTE_CHARS);
        assert_eq!(note.len(), 2, "{note:?}");
        assert_eq!(note.join(" "), MODEL_NOTE);
        for l in &note {
            assert!(
                l.chars().count() as f64 * 18.0 * CHAR_UNITS_PER_POINT <= COLUMN_ROOM + 1e-9,
                "{l:?}"
            );
        }
        let after = format!("{CAMERA} 185 {FEET}, {MAGNETOMETER} 220 {FEET}");
        for l in [AFTER_WORDS, after.as_str()] {
            assert!(
                l.chars().count() as f64 * 20.0 * CHAR_UNITS_PER_POINT <= COLUMN_ROOM,
                "{l:?}"
            );
        }
        // and the apostrophe is a typographic one
        assert!(MODEL_NOTE.contains('\u{2019}') && !MODEL_NOTE.contains('\''));
    }

    /// Everything stands on the stage: the line's labels, the Atlantic, the
    /// inset and the right-hand column; the ring sits above its number.
    #[test]
    fn the_pictures_sit_on_the_stage() {
        for x in LINE_X {
            assert!(x.abs() < QUIZ_FRAME.0);
        }
        for p in [AZORES_AT, NORFOLK_AT, PATCH_AT] {
            assert!(p[0].abs() + 1.0 < QUIZ_FRAME.0 && p[1].abs() < QUIZ_FRAME.1);
        }
        // the patch lies southwest of the Azores
        const {
            assert!(PATCH_AT[0] < AZORES_AT[0] && PATCH_AT[1] < AZORES_AT[1]);
        }
        // the inset cell stays right of the grid and inside the frame
        let grid_right = NAVY_CENTRE[0] + 15.0 * NAVY_ROW_H * 0.84 / 2.0;
        assert!(INSET_AT[0] - INSET_W / 2.0 > grid_right);
        assert!(INSET_AT[0] + INSET_W / 2.0 < QUIZ_FRAME.0);
        assert!(COLUMN_X - COLUMN_ROOM / 2.0 > grid_right);
        assert!(COLUMN_X + COLUMN_ROOM / 2.0 < QUIZ_FRAME.0);
        // the inset and its labels share the column's middle and room
        assert!((INSET_AT[0] - COLUMN_X).abs() < 1e-9);
        // the ring is above its number: the number's type (64 pt) reaches
        // about 0.45 above its own centre (constants, checked as the crate
        // builds)
        const NUMBER_REACH: f64 = 0.45;
        const {
            assert!(RING_AT[1] - RING_R > RING_AT[1] - NUMBER_DROP + NUMBER_REACH);
        }
    }
}
