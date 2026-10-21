//! Chapter 11, what two shadows can promise: a program checks that the
//! totals match and sends the water (11_1); every unit through, the wet
//! squares are a picture that fits, and if not, where the water stops names a
//! group of columns asking too much (11_2); then a switch in that picture:
//! none means the only one, any means others, and switches reach them all
//! (11_3); the four steps on the face's shadows (11_4, a prediction: what does
//! the program report?); every unit through, but the water's picture has
//! twelve switches: a picture that fits, not the only one (11_5); what no
//! program can say is which of the sixty-four was really there, and two of
//! the face's own squares are filled in only twelve of them (11_6); the two
//! questions kept apart, inside the bounded model (11_7).
//!
//! The water is chapter 03's (`facts::face_flow`), its switches
//! `model::switches`; how often each square is filled `facts::filled_in`.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{Board, fade_in_all, fade_out_all, fail, flash_all, pause, say, swap, think, wait_until};
use super::same_shadows::{QUESTIONS, counts, place};
use super::stops::read_s;
use super::water::{AT, CELL, PROGRAMMERS, TEXT_X, tap_at, through_text};
use crate::facts;
use crate::model;
use crate::palette;

pub struct WhatTwoShadowsPromise {
    pub kit: Kit,
}

/// The four steps a program follows with two shadows, as the screen lists
/// them (11_1 to 11_3; `oracle.py` reads their number from here).
pub const STEPS: [&str; 4] = [
    "1. do both totals match?",
    "2. send the water",
    "3. every unit through? a picture that fits",
    "4. no switch? the only one",
];
/// The second halves of steps 3 and 4.
const DETAILS: [&str; 2] = [
    "otherwise the columns it can't reach ask too much",
    "a switch? others, and switches reach them all",
];
/// The answers 11_7 gives chapter 01's two questions: for the whole video
/// (root's wording, 2026-10-20: "these four shadows singled out the face",
/// no claim about other directions); the second in two lines.
const ANSWERS: [&str; 3] = [
    "the water says, when the totals match",
    "with rows and columns, a switch says;",
    "these four shadows singled out the face",
];
const NOT_SAID: &str = "which of the 64 was there? the shadows don't say";

/// The list on the right: its heading, each step, and each second half.
const HEADING_Y: f64 = 2.6;
const STEP_Y: [f64; 4] = [2.0, 1.4, 0.8, -0.2];
const DETAIL_Y: [f64; 2] = [0.35, -0.65];
const DETAIL_DX: f64 = 0.3;
/// The think ring's centre (11_4), under the list's right end.
const THINK_AT: [f64; 2] = [5.3, -1.9];
/// 11_5 and 11_6's words on the right.
const REPORT_Y: [f64; 5] = [2.0, 1.4, 0.8, 0.0, -0.6];
/// The closing questions and answers (11_7), top to bottom.
const CLOSING_Y: [f64; 5] = [2.0, 1.45, 0.6, 0.05, -0.45];

fn through_all(through: usize, total: usize) -> String {
    format!("every unit {}", through_text(through, total))
}

fn switches_text(n: usize) -> String {
    format!("the water's picture: {n} switches")
}

fn report_text(only: bool) -> &'static str {
    if only {
        "a picture that fits, the only one"
    } else {
        "a picture that fits, not the only one"
    }
}

fn rarest_text(filled: usize, of: usize) -> String {
    format!("each filled in only {filled} of {of}")
}

/// The exact bounds of the video's model, kept on screen (11_7): every grid
/// it shows, not only the face's.
fn model_claim() -> String {
    "every picture in this video is a small grid of filled and empty squares, counted exactly along rows, \
     columns and, for the face, diagonals; nothing here models a scanner"
        .to_string()
}

impl SceneConstruct for WhatTwoShadowsPromise {
    fn name(&self) -> &str {
        "11_what_two_shadows_promise"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let face = model::face();
        let s = face.shadows();
        let total = s.row_total();
        let (_, states) = facts::face_flow().map_err(fail)?;
        let last = states
            .last()
            .ok_or_else(|| fail("the face's flow sends nothing".into()))?;
        let built = last.wet.clone();
        let found = model::switches(&built);
        let fits = model::fits(&s);
        if last.through() != total || found.is_empty() || !fits.contains(&built) {
            return Err(fail("the face's water does not report a picture that fits with switches".into()));
        }
        let only = fits.len() == 1;
        let filled = facts::filled_in();
        let of = facts::gallery().len();
        let rarest = face
            .filled()
            .iter()
            .map(|&(r, c)| filled[r][c])
            .min()
            .ok_or_else(|| fail("the face has no square".into()))?;
        let rarest_squares: Vec<(usize, usize)> = face
            .filled()
            .into_iter()
            .filter(|&(r, c)| filled[r][c] == rarest)
            .collect();
        if rarest_squares.len() != 2 || 2 * rarest >= of {
            return Err(fail(format!(
                "the face's rarest squares are {rarest_squares:?}, filled in {rarest} of {of}"
            )));
        }
        let board = Board::new(AT, CELL, face.rows(), face.cols());

        // 11_1: the face's counts round an empty grid; the list builds as it
        // is spoken: check the totals, send the water
        let line = say(&kit, stage, "11_1")?;
        let grid = board.lines(stage);
        let mut face_counts = counts(&kit, stage, &s.rows, Palette::ink(), |r| tap_at(&board, r))?;
        face_counts.extend(counts(&kit, stage, &s.cols, Palette::ink(), |c| board.col_count_at(c))?);
        let mut shown = grid.clone();
        shown.extend(&face_counts);
        let heading = kit.text_left(stage, PROGRAMMERS, 24.0, palette::switch(), v(TEXT_X, HEADING_Y))?;
        shown.push(heading);
        fade_in_all(stage, &shown, 0.5)?;
        let mut list = Vec::with_capacity(STEPS.len() + DETAILS.len());
        for (k, phrase) in [(0, "Check that the totals match"), (1, "send the water")] {
            line.wait_for(stage, phrase)?;
            let step = kit.text_left(stage, STEPS[k], 24.0, Palette::ink(), v(TEXT_X, STEP_Y[k]))?;
            fade_in_all(stage, &[step], 0.3)?;
            list.push(step);
        }

        // 11_2: every unit through gives a picture; if not, the columns the
        // water cannot reach ask too much
        let line = say(&kit, stage, "11_2")?;
        line.wait_for(stage, "If every unit")?;
        let third = kit.text_left(stage, STEPS[2], 24.0, Palette::ink(), v(TEXT_X, STEP_Y[2]))?;
        fade_in_all(stage, &[third], 0.3)?;
        list.push(third);
        line.wait_for(stage, "If not")?;
        let otherwise = kit.text_left(
            stage,
            DETAILS[0],
            20.0,
            Palette::dim(),
            v(TEXT_X + DETAIL_DX, DETAIL_Y[0]),
        )?;
        fade_in_all(stage, &[otherwise], 0.3)?;
        list.push(otherwise);

        // 11_3: then a switch: none, the only one; any, others, all reached
        let line = say(&kit, stage, "11_3")?;
        line.wait_for(stage, "look for a switch")?;
        let fourth = kit.text_left(stage, STEPS[3], 24.0, Palette::ink(), v(TEXT_X, STEP_Y[3]))?;
        fade_in_all(stage, &[fourth], 0.3)?;
        list.push(fourth);
        line.wait_for(stage, "Any switch")?;
        let others = kit.text_left(
            stage,
            DETAILS[1],
            20.0,
            Palette::dim(),
            v(TEXT_X + DETAIL_DX, DETAIL_Y[1]),
        )?;
        fade_in_all(stage, &[others], 0.3)?;
        list.push(others);
        line.wait_to_end(stage)?;
        pause(stage, 1.0)?;

        // 11_4: the four steps on the face's shadows; the question
        let line = say(&kit, stage, "11_4")?;
        line.wait_for(stage, "the face's shadows")?;
        flash_all(stage, &face_counts, 0.5)?;
        let q = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 11_5: every unit gets through; the water's picture has twelve
        // switches; so a picture that fits, not the only one
        let line = say(&kit, stage, "11_5")?;
        let mut going = list;
        going.extend([heading, q]);
        fade_out_all(stage, &going, 0.3)?;
        line.wait_for(stage, "Every unit")?;
        let water_squares = place(stage, &board, &built)?;
        let squares: Vec<Mob> = built.filled().iter().map(|rc| water_squares[rc]).collect();
        let through = kit.text_left(
            stage,
            &through_all(last.through(), total),
            24.0,
            palette::matches(),
            v(TEXT_X, REPORT_Y[0]),
        )?;
        fade_in_all(stage, &squares, 0.5)?;
        fade_in_all(stage, &[through], 0.3)?;
        line.wait_for(stage, "twelve switches")?;
        let switches = kit.text_left(
            stage,
            &switches_text(found.len()),
            24.0,
            palette::switch(),
            v(TEXT_X, REPORT_Y[1]),
        )?;
        fade_in_all(stage, &[switches], 0.3)?;
        line.wait_for(stage, "So it reports")?;
        let report = kit.text_left(stage, report_text(only), 24.0, Palette::ink(), v(TEXT_X, REPORT_Y[2]))?;
        fade_in_all(stage, &[report], 0.3)?;

        // 11_6: which of the 64 was there, no program can say; the face's
        // own two squares filled in only 12 of them
        let line = say(&kit, stage, "11_6")?;
        line.wait_for(stage, "which of the sixty-four")?;
        let not_said = kit.text_left(stage, NOT_SAID, 24.0, Palette::ink(), v(TEXT_X, REPORT_Y[3]))?;
        fade_in_all(stage, &[not_said], 0.3)?;
        line.wait_for(stage, "Two of the face's own squares")?;
        let face_squares = place(stage, &board, &face)?;
        let face_mobs: Vec<Mob> = face_squares.values().copied().collect();
        swap(stage, &squares, &face_mobs, 0.5)?;
        let rings: Vec<Mob> = rarest_squares
            .iter()
            .map(|&(r, c)| board.ring(stage, r, c, Palette::ink()))
            .collect();
        let rare = kit.text_left(
            stage,
            &rarest_text(rarest, of),
            24.0,
            Palette::ink(),
            v(TEXT_X, REPORT_Y[4]),
        )?;
        fade_in_all(stage, &rings, 0.3)?;
        fade_in_all(stage, &[rare], 0.3)?;

        // 11_7: the bounded model on the card; the two questions with the
        // whole video's answers; held until read, then the fade
        let line = say(&kit, stage, "11_7")?;
        let mut leaving = grid;
        leaving.extend(face_counts);
        leaving.extend(face_mobs);
        leaving.extend(rings);
        leaving.extend([through, switches, report, not_said, rare]);
        fade_out_all(stage, &leaving, 0.4)?;
        let claim = model_claim();
        let card = precisely(&kit, stage, &claim)?;
        fade_in_all(stage, &[card], 0.4)?;
        let card_up = stage.scene().time().to_f64();
        let closing = [
            kit.text_left(stage, QUESTIONS[0], 28.0, Palette::ink(), v(TEXT_X, CLOSING_Y[0]))?,
            kit.text_left(stage, ANSWERS[0], 22.0, Palette::ink(), v(TEXT_X, CLOSING_Y[1]))?,
            kit.text_left(stage, QUESTIONS[1], 28.0, Palette::ink(), v(TEXT_X, CLOSING_Y[2]))?,
            kit.text_left(stage, ANSWERS[1], 22.0, Palette::ink(), v(TEXT_X, CLOSING_Y[3]))?,
            kit.text_left(stage, ANSWERS[2], 22.0, Palette::ink(), v(TEXT_X, CLOSING_Y[4]))?,
        ];
        line.wait_for(stage, "Does any picture")?;
        fade_in_all(stage, &closing[..2], 0.4)?;
        let shown_up = stage.scene().time().to_f64();
        line.wait_for(stage, "And is it")?;
        fade_in_all(stage, &closing[2..], 0.4)?;
        line.wait_to_end(stage)?;
        let all_words = [QUESTIONS[0], ANSWERS[0], QUESTIONS[1], ANSWERS[1], ANSWERS[2]].join(" ");
        wait_until(stage, (card_up + read_s(&claim)).max(shown_up + read_s(&all_words)))?;
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
            ("11_1", "Check that the totals match"),
            ("11_1", "send the water"),
            ("11_2", "If every unit"),
            ("11_2", "If not"),
            ("11_3", "look for a switch"),
            ("11_3", "Any switch"),
            ("11_4", "the face's shadows"),
            ("11_5", "Every unit"),
            ("11_5", "twelve switches"),
            ("11_5", "So it reports"),
            ("11_6", "which of the sixty-four"),
            ("11_6", "Two of the face's own squares"),
            ("11_7", "Does any picture"),
            ("11_7", "And is it"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The steps are numbered in order and as many as 11_4 says; the words
    /// on screen are the facts'; the card bounds every grid the video shows
    /// in whole lines and it and the closing words are held under twelve
    /// seconds each.
    #[test]
    fn the_words_on_screen_are_built_from_the_facts() {
        for (k, step) in STEPS.iter().enumerate() {
            assert!(step.starts_with(&format!("{}. ", k + 1)), "{step}");
        }
        let f = facts::facts().unwrap();
        let get = |key: &str| -> usize { f.get(key).unwrap().parse().unwrap() };
        assert_eq!(get("DT-F19.steps"), STEPS.len());
        assert_eq!(through_all(get("DT-F8.through"), 9), "every unit through: 9 of 9");
        assert_eq!(switches_text(get("DT-F10.face.switches")), "the water's picture: 12 switches");
        assert_eq!(f.get("DT-F8.built.only_one"), Some("false"));
        assert_eq!(report_text(false), "a picture that fits, not the only one");
        assert_eq!(
            rarest_text(get("DT-F12.face.rarest"), get("DT-F3.fits")),
            "each filled in only 12 of 64"
        );
        assert_eq!(f.get("DT-F12.face.rarest.count"), Some("2"));
        assert!(ANSWERS[2].contains("these four shadows singled out the face"));
        let claim = model_claim();
        for part in ["small grid of filled and empty squares", "counted exactly", "nothing here models a scanner"] {
            assert!(claim.contains(part), "{part}: {claim}");
        }
        for width in (76..=92).step_by(4) {
            let lines = wrap(&claim, width, 3);
            assert!(!lines.iter().any(|l| l.ends_with('…')), "{width}: {lines:?}");
        }
        let all_words = [QUESTIONS[0], ANSWERS[0], QUESTIONS[1], ANSWERS[1], ANSWERS[2]].join(" ");
        for text in [claim.as_str(), all_words.as_str()] {
            assert!(read_s(text) < 12.0, "{text}");
        }
    }

    /// The chapter's layout keeps its pieces apart and inside the
    /// title-safe area (about 0.0045 units a point a character).
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        let board = Board::new(AT, CELL, 5, 5);
        let safe_x = FRAME_X_RADIUS - 0.25;
        let safe_y = FRAME_Y_RADIUS - 0.25;
        let ends = |x: f64, text: &str, size: f64| x + text.chars().count() as f64 * size * 0.0045;
        assert!(TEXT_X > board.right() + 0.5);
        for step in STEPS {
            assert!(ends(TEXT_X, step, 24.0) < safe_x, "{step}");
        }
        for detail in DETAILS {
            assert!(ends(TEXT_X + DETAIL_DX, detail, 20.0) < THINK_AT[0] - 0.7, "{detail}");
        }
        for answer in ANSWERS {
            assert!(ends(TEXT_X, answer, 22.0) < safe_x, "{answer}");
        }
        assert!(ends(TEXT_X, NOT_SAID, 24.0) < safe_x);
        assert!(HEADING_Y + 0.2 < safe_y);
        // the think ring and its note stay under the list, inside
        assert!(THINK_AT[1] + 0.62 < DETAIL_Y[1] - 0.15);
        assert!(THINK_AT[0] + 1.1 < safe_x);
        assert!(THINK_AT[1] - 0.62 - 0.3 - 0.15 > -safe_y);
        // the closing words stay over a three-line card
        let card_top = -FRAME_Y_RADIUS + 0.2 + 0.3 + 3.0 * 0.36;
        assert!(CLOSING_Y[4] - 0.15 > card_top);
        assert!(board.col_count_at(0)[1] - 0.2 > card_top);
    }
}
