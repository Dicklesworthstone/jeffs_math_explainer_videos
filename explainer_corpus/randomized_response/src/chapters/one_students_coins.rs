//! Chapter 02, one student's coins: the rule with fair, independent, private
//! coins (02_1, 02_2); one student who copied through all four outcomes
//! (02_3); the prediction, how many of the four give someone who didn't copy
//! a yes (02_4), and its answer, one (02_5); no answer is a confession, but
//! a yes is three times as likely from someone who copied (02_6); a no the
//! reverse, never more than three times, whatever the survey believed
//! (02_7); and why the coins must stay private (02_8).
//!
//! The grid is the rule run on one truth and one coin outcome per cell
//! (`model::answer`), its route words `model::route`'s. The didn't row stays
//! faint and wordless until its answer (02_5): nothing on screen answers the
//! prediction during its hold.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    cell, disc, face, fade_in_all, fade_out_all, fail, flash_all, outline, pause, read_by, ring, say, swap, think,
    wait_until,
};
use super::the_coin::{RulePicture, seat_at};
use crate::facts::{self, Average};
use crate::model::{self, COIN_PAIRS, Channel, Route};
use crate::palette;

pub struct OneStudentsCoins {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const TWO_COINS: &str = "the two coins:";
const COPIED: &str = "copied";
const DIDNT: &str = "didn't";
const YES: &str = "yes";
const NO: &str = "no";
const TRUTH_ROUTE: &str = "truth";
const COIN_ROUTE: &str = "second coin";
const SEEN: &str = "seen heads: a confession";

// ------------------------------------------------------------------ layout

/// The grid of the four coin outcomes: one column each, a row for someone
/// who copied and a row for someone who didn't; each column's route over
/// its faces.
const GRID_X: [f64; 4] = [-1.95, -0.65, 0.65, 1.95];
const GRID_HEAD_Y: f64 = 1.75;
const ROUTE_Y: f64 = 2.2;
const GRID_ROW_Y: [f64; 2] = [0.8, -0.35];
const GRID_W: f64 = 1.1;
const GRID_H: f64 = 0.75;
const GRID_LABEL_X: f64 = -4.6;
const SHARE_X: f64 = 3.35;
const TIMES_X: f64 = 4.75;
/// The think time, right of the grid's head.
const THINK_X: f64 = 5.6;
const THINK_Y: f64 = 1.9;
/// The gold rings around a seen heads-first cell, and their words.
const SEEN_R: f64 = 0.62;
const SEEN_Y: f64 = -1.4;

/// 02_2's exact version: the rule's hypothesis (root's clarification 1).
fn hypothesis_card() -> String {
    "assumed: fair coins, independent across people and across asks, flipped where nobody else can see them".to_owned()
}

/// 02_6's exact version: both answers' ratios, and the bound (the old
/// 01_11's).
fn ratio_note() -> fmn::Result<String> {
    let worst = Channel::two_coins()
        .worst_ratio()
        .ok_or_else(|| fail("the two coins leave a ratio unbounded".into()))?;
    Ok(format!(
        "a yes: {} of the {} coin outcomes if you copied, {} of {} if you didn't: {} times as likely, never more. \
         A no is {} times as likely if you didn't",
        model::outcomes_saying(true, true),
        COIN_PAIRS.len(),
        model::outcomes_saying(false, true),
        COIN_PAIRS.len(),
        worst,
        model::likelihood_ratio(false)
    ))
}

/// 02_8's exact version: how far the bound reaches, and what it needs.
fn private_card() -> String {
    "for one answer under this rule, whatever was believed before; it needs coins nobody else sees".to_owned()
}

/// A share beside a row: "3 of 4", or "no: 1 of 4" for the answer no.
fn share(copied: bool, said: bool) -> String {
    let count = model::outcomes_saying(copied, said);
    if said {
        format!("{count} of {}", COIN_PAIRS.len())
    } else {
        format!("{NO}: {count} of {}", COIN_PAIRS.len())
    }
}

/// The words for a coin outcome's route: the truth, or the second coin.
fn route_word(route: Route) -> (&'static str, Srgb) {
    if route == Route::Truth {
        (TRUTH_ROUTE, palette::truth())
    } else {
        (COIN_ROUTE, palette::forced())
    }
}

/// One answered cell of the grid: its panel and its word. Detached.
fn answered(kit: &Kit, stage: &mut Stage<'_>, said: bool, at: Vec3) -> fmn::Result<[Mob; 2]> {
    let c = cell(stage, palette::answer(said), at, GRID_W, GRID_H)?;
    let w = kit.text(stage, if said { YES } else { NO }, 24.0, palette::answer(said), at)?;
    Ok([c, w])
}

impl SceneConstruct for OneStudentsCoins {
    fn name(&self) -> &str {
        "02_one_students_coins"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let a: Average = facts::average().map_err(fail)?;
        let n = a.class.size();
        let rule = RulePicture::build(&kit, stage)?;

        // 02_1: the class as chapter 01 left it; a fair coin; heads, the truth
        let line = say(&kit, stage, "02_1")?;
        let mut discs = Vec::with_capacity(n);
        let mut rings = Vec::with_capacity(a.class.copiers());
        for i in 0..n {
            discs.push(disc(stage, palette::student(), seat_at(i)));
            if a.class.copied(i) {
                rings.push(ring(stage, seat_at(i)));
            }
        }
        let mut opening = vec![rule.question, rule.key_ring, rule.key_text];
        opening.extend(discs.iter().copied());
        opening.extend(rings.iter().copied());
        fade_in_all(stage, &opening, 0.5)?;
        fade_in_all(stage, &[rule.first_coin, rule.coin_label], 0.4)?;
        line.wait_for(stage, "Heads, tell")?;
        fade_in_all(stage, &rule.truth_parts(), 0.6)?;

        // 02_2: tails, a second fair coin, independent, answers
        let line = say(&kit, stage, "02_2")?;
        let hypothesis_text = hypothesis_card();
        let hypothesis = precisely(&kit, stage, &hypothesis_text)?;
        let mut tails = rule.tails_parts();
        tails.push(hypothesis);
        fade_in_all(stage, &tails, 0.6)?;
        let hypothesis_read = read_by(stage, &hypothesis_text);
        line.wait_for(stage, "Heads for yes")?;
        fade_in_all(stage, &[rule.heads_word], 0.3)?;
        flash_all(stage, &[rule.yes_head, rule.heads_word], 0.5)?;
        line.wait_for(stage, "tails for no")?;
        fade_in_all(stage, &[rule.tails_word], 0.3)?;
        flash_all(stage, &[rule.no_head, rule.tails_word], 0.5)?;

        // 02_3: one student who copied, through all four outcomes
        let line = say(&kit, stage, "02_3")?;
        let first_copier = (0..n)
            .find(|&i| a.class.copied(i))
            .ok_or_else(|| fail("nobody copied".into()))?;
        flash_all(stage, &[discs[first_copier], rings[0]], 0.6)?;
        wait_until(stage, hypothesis_read)?;
        let mut leaving = rule.all();
        leaving.extend(discs.iter().copied());
        leaving.extend(rings.iter().copied());
        leaving.push(hypothesis);
        fade_out_all(stage, &leaving, 0.4)?;
        let mut head = vec![kit.text_left(stage, TWO_COINS, 22.0, Palette::dim(), v(GRID_LABEL_X, GRID_HEAD_Y))?];
        for (&(first, second), &x) in COIN_PAIRS.iter().zip(&GRID_X) {
            let faces = format!("{} {}", face(first), face(second));
            head.push(kit.text(stage, &faces, 24.0, Palette::dim(), v(x, GRID_HEAD_Y))?);
        }
        head.push(kit.text_left(stage, COPIED, 26.0, palette::copied(), v(GRID_LABEL_X, GRID_ROW_Y[0]))?);
        fade_in_all(stage, &head, 0.4)?;
        // the copied row, outcome by outcome as the words name them
        let mut copied_cells: Vec<[Mob; 2]> = Vec::with_capacity(COIN_PAIRS.len());
        for (phrase, outcomes) in [
            ("Heads first", 0..2),
            ("Tails then heads", 2..3),
            ("Tails then tails", 3..4),
        ] {
            line.wait_for(stage, phrase)?;
            let mut shown = Vec::new();
            for o in outcomes {
                let (first, second) = COIN_PAIRS[o];
                let said = model::answer(true, first, second);
                let cells = answered(&kit, stage, said, v(GRID_X[o], GRID_ROW_Y[0]))?;
                let (word, color) = route_word(model::route(first, second));
                shown.extend(cells);
                shown.push(kit.text(stage, word, 18.0, color, v(GRID_X[o], ROUTE_Y))?);
                copied_cells.push(cells);
            }
            fade_in_all(stage, &shown, 0.4)?;
        }
        if copied_cells.len() != COIN_PAIRS.len() {
            return Err(fail(format!("{} cells drawn for four outcomes", copied_cells.len())));
        }

        // 02_4: the prediction; the didn't row faint and wordless
        let line = say(&kit, stage, "02_4")?;
        let copied_share = kit.text(
            stage,
            &share(true, true),
            28.0,
            palette::yes(),
            v(SHARE_X, GRID_ROW_Y[0]),
        )?;
        fade_in_all(stage, &[copied_share], 0.3)?;
        line.wait_for(stage, "Now someone")?;
        let mut faint = vec![kit.text_left(stage, DIDNT, 26.0, palette::student(), v(GRID_LABEL_X, GRID_ROW_Y[1]))?];
        let mut blanks = Vec::with_capacity(COIN_PAIRS.len());
        for &x in &GRID_X {
            let c = cell(stage, palette::ghost(), v(x, GRID_ROW_Y[1]), GRID_W, GRID_H)?;
            faint.push(c);
            blanks.push(c);
        }
        fade_in_all(stage, &faint, 0.4)?;
        let q = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 02_5: one: only tails then heads says yes
        let _line = say(&kit, stage, "02_5")?;
        let mut didnt_cells = Vec::with_capacity(COIN_PAIRS.len());
        let mut coloured = Vec::new();
        for (o, &(first, second)) in COIN_PAIRS.iter().enumerate() {
            let cells = answered(
                &kit,
                stage,
                model::answer(false, first, second),
                v(GRID_X[o], GRID_ROW_Y[1]),
            )?;
            coloured.extend(cells);
            didnt_cells.push(cells);
        }
        let didnt_share = kit.text(
            stage,
            &share(false, true),
            28.0,
            palette::yes(),
            v(SHARE_X, GRID_ROW_Y[1]),
        )?;
        coloured.push(didnt_share);
        let mut gone = blanks.clone();
        gone.push(q);
        swap(stage, &gone, &coloured, 0.4)?;

        // 02_6: no confession, but three times as often (the old 01_11)
        let line = say(&kit, stage, "02_6")?;
        let yes_of = |cells: &[[Mob; 2]], copied: bool| -> Vec<Mob> {
            cells
                .iter()
                .zip(COIN_PAIRS)
                .filter(|&(_, (f, s))| model::answer(copied, f, s))
                .flat_map(|(c, _)| c.iter().copied())
                .collect()
        };
        line.wait_for(stage, "But someone who copied")?;
        let ratio_text = ratio_note()?;
        let card = precisely(&kit, stage, &ratio_text)?;
        fade_in_all(stage, &[card], 0.4)?;
        let ratio_read = read_by(stage, &ratio_text);
        flash_all(stage, &yes_of(&copied_cells, true), 0.6)?;
        line.wait_for(stage, "as someone who didn't")?;
        flash_all(stage, &yes_of(&didnt_cells, false), 0.5)?;
        let times = kit.tex(
            stage,
            &format!(r"\times {}", model::likelihood_ratio(true)),
            40.0,
            Palette::ink(),
            &[],
            v(TIMES_X, (GRID_ROW_Y[0] + GRID_ROW_Y[1]) / 2.0),
        )?;
        fade_in_all(stage, &[times], 0.4)?;

        // 02_7: a no the reverse; never more than three, whatever was believed
        let line = say(&kit, stage, "02_7")?;
        if model::likelihood_ratio(false) != model::likelihood_ratio(true) {
            return Err(fail(
                "the two answers' ratios differ; the shared × label would be wrong".into(),
            ));
        }
        let no_shares = [
            kit.text(
                stage,
                &share(true, false),
                28.0,
                palette::no(),
                v(SHARE_X, GRID_ROW_Y[0]),
            )?,
            kit.text(
                stage,
                &share(false, false),
                28.0,
                palette::no(),
                v(SHARE_X, GRID_ROW_Y[1]),
            )?,
        ];
        swap(stage, &[copied_share, didnt_share], &no_shares, 0.4)?;
        let no_of = |cells: &[[Mob; 2]], copied: bool| -> Vec<Mob> {
            cells
                .iter()
                .zip(COIN_PAIRS)
                .filter(|&(_, (f, s))| !model::answer(copied, f, s))
                .flat_map(|(c, _)| c.iter().copied())
                .collect()
        };
        let mut no_cells = no_of(&copied_cells, true);
        no_cells.extend(no_of(&didnt_cells, false));
        flash_all(stage, &no_cells, 0.6)?;
        flash_all(stage, &[times], 0.4)?;
        line.wait_for(stage, "whatever the survey believed")?;
        flash_all(stage, &[times], 0.5)?;

        // 02_8: it needs private coins: a seen heads first is a confession
        let line = say(&kit, stage, "02_8")?;
        let private_text = private_card();
        let private = precisely(&kit, stage, &private_text)?;
        wait_until(stage, ratio_read)?;
        swap(stage, &[card], &[private], 0.4)?;
        let private_read = read_by(stage, &private_text);
        line.wait_for(stage, "Seeing heads first")?;
        let mut seen = Vec::new();
        for (o, &(first, second)) in COIN_PAIRS.iter().enumerate() {
            if model::route(first, second) == Route::Truth {
                seen.push(outline(stage, palette::copied(), v(GRID_X[o], GRID_ROW_Y[0]), SEEN_R));
            }
        }
        seen.push(kit.text(stage, SEEN, 22.0, palette::copied(), v(0.0, SEEN_Y))?);
        fade_in_all(stage, &seen, 0.4)?;
        flash_all(stage, &seen, 0.5)?;
        line.wait_end(stage)?;
        pause(stage, 1.0)?;
        wait_until(stage, private_read)?;

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
            ("02_1", "Heads, tell"),
            ("02_2", "Heads for yes"),
            ("02_2", "tails for no"),
            ("02_3", "Heads first"),
            ("02_3", "Tails then heads"),
            ("02_3", "Tails then tails"),
            ("02_4", "Now someone"),
            ("02_6", "But someone who copied"),
            ("02_6", "as someone who didn't"),
            ("02_7", "whatever the survey believed"),
            ("02_8", "Seeing heads first"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The words on screen say what the facts say: both answers' ratios,
    /// each row's share, the routes the rule takes, and the ratio's bound
    /// for one answer.
    #[test]
    fn the_exact_versions_are_built_from_the_facts() {
        let ratio = ratio_note().unwrap();
        assert!(
            ratio.contains("3 of the 4 coin outcomes if you copied, 1 of 4"),
            "{ratio}"
        );
        assert!(ratio.contains("3 times as likely, never more"), "{ratio}");
        assert!(ratio.contains("A no is 3 times as likely"), "{ratio}");
        assert_eq!(share(true, true), "3 of 4");
        assert_eq!(share(false, true), "1 of 4");
        assert_eq!(share(true, false), "no: 1 of 4");
        assert_eq!(share(false, false), "no: 3 of 4");
        let f = facts::facts().unwrap();
        assert_eq!(f.get("R-F2.yes_outcomes.copied"), Some("3"));
        assert_eq!(f.get("R-F2.yes_outcomes.didnt"), Some("1"));
        assert_eq!(f.get("R-F5.worst_ratio"), Some("3"));
        assert_eq!(
            f.get("R-X8.rappor.prior"),
            Some("irrespective of any attacker's prior knowledge")
        );
        // heads first is the truth either way; tails hands over to the second coin
        let words: Vec<&str> = COIN_PAIRS
            .iter()
            .map(|&(a, b)| route_word(model::route(a, b)).0)
            .collect();
        assert_eq!(words, [TRUTH_ROUTE, TRUTH_ROUTE, COIN_ROUTE, COIN_ROUTE]);
        assert!(hypothesis_card().contains("independent across people and across asks"));
        const { assert!(GRID_X.len() == COIN_PAIRS.len(), "one grid column per coin outcome") };
    }

    /// The cards stay two lines at every wrap the lower-third can narrow to.
    #[test]
    fn the_cards_are_two_lines() {
        for text in [hypothesis_card(), ratio_note().unwrap(), private_card()] {
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
        // the grid's cells sit apart, between its labels and its shares
        for pair in GRID_X.windows(2) {
            assert!(pair[1] - pair[0] > GRID_W + 0.1);
        }
        const {
            assert!(
                GRID_X[0] - GRID_W / 2.0 > GRID_LABEL_X + 1.6,
                "the row labels clear the grid"
            )
        };
        const { assert!(GRID_X[3] + GRID_W / 2.0 < SHARE_X - 0.6, "the shares clear the grid") };
        const { assert!(GRID_ROW_Y[1] - GRID_H / 2.0 > BAND_TOP + 0.3, "the grid over the band") };
        // the routes over the faces, inside the top edge
        const { assert!(ROUTE_Y - 0.15 > GRID_HEAD_Y + 0.15, "the routes over the faces") };
        const { assert!(ROUTE_Y + 0.15 < FRAME_Y_RADIUS - 0.25, "the routes inside the top") };
        // the think ring right of the routes and clear of the shares
        const { assert!(THINK_X - 0.62 > GRID_X[3] + 0.7, "the think ring clear of the routes") };
        const { assert!(THINK_X + 0.62 < FRAME_X_RADIUS - 0.25, "the think ring inside") };
        // its note (about 2 wide, under the ring) right of the shares
        const { assert!(THINK_X - 1.0 > SHARE_X + 0.5, "the think note clear of the shares") };
        const { assert!(THINK_Y - 0.62 - 0.45 > BAND_TOP, "the think note over the band") };
        // the seen rings around the copied row, their words under the grid
        const {
            assert!(
                SEEN_R < (GRID_X[1] - GRID_X[0]) / 2.0 + 0.05,
                "neighbouring rings barely touch"
            )
        };
        const {
            assert!(
                GRID_ROW_Y[0] - SEEN_R > GRID_ROW_Y[1] - GRID_H / 2.0 - 0.1,
                "rings around one row"
            )
        };
        const {
            assert!(
                SEEN_Y + 0.15 < GRID_ROW_Y[1] - GRID_H / 2.0 - 0.1,
                "the words under the grid"
            )
        };
        const { assert!(SEEN_Y - 0.15 > BAND_TOP, "the words over the band") };
    }
}
