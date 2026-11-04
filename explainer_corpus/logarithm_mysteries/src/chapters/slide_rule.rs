//! Chapter 02, the slide rule as a worked machine: a fixed D scale and a
//! sliding C scale (02_1); 2.5 × 3.2 by sliding C's 1 over D's 2.5 and
//! reading under C's 3.2, with a prediction hold (02_2); the reveal, the two
//! blue lengths laid end to end on D (02_3); 4 × 6 running off D's end
//! (02_4); the other index, the mark 2.4, and the decimal point the person
//! must supply (02_5).
//!
//! Lengths are blue, numbers on D green, the sliding scale teal, the answer
//! hero, the overflow red. Every value shown is a facts.rs key.
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`; picture repairs
//! from the independent review staged in `logarithm-repair01-04`. Type-checked
//! against stand-ins only: not compiled against fmn, unrendered, unlistened.

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    Decade, answer_c, dashed, fact, fact_f64, fade_in_all, fade_out_all, number_c, position_c, precisely, predict,
    ring, seg, shift_all, slide_c, swap, title, warn_c,
};

pub struct SlideRule {
    pub kit: Kit,
}

/// D's 1 sits at `ONE_X`; one decade is `LEN` scene units. D is drawn on
/// the left so that the 4 × 6 overflow lands on screen, past D's end.
const ONE_X: f64 = -6.0;
const LEN: f64 = 8.0;
const D_Y: f64 = 0.3;
const C_Y: f64 = 0.62;
/// Where the blue lengths ride: under D's labels, over C's labels.
const D_BAR_Y: f64 = -0.72;
const C_BAR_Y: f64 = 1.62;
/// How far a reading line reaches past each scale: past the ticks (0.32) and
/// short of the numbers (about 0.5), so it points at the digit and leaves it whole.
const HAIR_REACH: f64 = 0.42;
/// C's name once its 1 has gone off the frame: just past its 10.
const C_NAME_DX: f64 = 0.5;
/// The words the first slide waits for, in 02_2's first sentence.
const SLIDE_PHRASE: &str = "slide the moving";
/// The words the nudge waits for, in 02_1's first sentence.
const NUDGE_PHRASE: &str = "one that slides";
const SCRIPT: &str = include_str!("../../script.tsv");

fn d_scale() -> Decade {
    Decade {
        one: v(ONE_X, D_Y),
        length: LEN,
        side: -1.0,
    }
}

/// C, slid right by `offset` scene units.
fn c_scale(offset: f64) -> Decade {
    Decade {
        one: v(ONE_X + offset, C_Y),
        length: LEN,
        side: 1.0,
    }
}

/// The reading hairline through both scales at `x`.
fn hairline(stage: &mut Stage<'_>, x: f64, c: Srgb) -> Mob {
    seg(stage, v(x, D_Y - HAIR_REACH), v(x, C_Y + HAIR_REACH), c, 2.5)
}

/// When `phrase`, in the first sentence of `cue`, begins (scene seconds): the
/// narration's phrase mark at that byte when one exists, otherwise the phrase's
/// share of the sentence's characters (the house rule, as in the algebra film).
fn phrase_time(kit: &Kit, cue: &str, phrase: &str) -> Option<f64> {
    let narrator = kit.narrator.as_ref()?;
    let text = SCRIPT
        .lines()
        .find_map(|row| row.strip_prefix(cue)?.strip_prefix('\t'))?;
    let byte = text.find(phrase)?;
    let said = narrator.sentence_at(cue, 0)? - narrator.sentence_starts(cue).first().copied()?;
    if let Some(&seconds) = narrator.phrase_marks(cue).get(&byte) {
        return Some(said + seconds);
    }
    let sentence_end = text.find(". ").map_or(text.len(), |k| k + 2);
    let start = narrator.sentence_at(cue, 0)?;
    let end = match narrator.sentence_at(cue, 1) {
        Some(next) => next,
        None => said + narrator.length(cue)?,
    };
    let share = text[..byte].chars().count() as f64 / text[..sentence_end].chars().count() as f64;
    Some(start + (end - start) * share)
}

/// Wait for `phrase` in the first sentence of `cue`. Without a narrator, or if
/// the phrase is missing (a test guards it), this does nothing; nothing waits
/// once the moment has passed.
fn until_phrase(stage: &mut Stage<'_>, kit: &Kit, cue: &str, phrase: &str) -> fmn::Result<()> {
    let Some(at) = phrase_time(kit, cue, phrase) else {
        return Ok(());
    };
    let (now, fps) = (stage.scene().time().to_f64(), stage.scene().time().fps());
    if at > now {
        wait_frames(stage, frames_covering(at - now, fps))?;
    }
    Ok(())
}

impl SceneConstruct for SlideRule {
    fn name(&self) -> &str {
        "02_slide_rule"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (green, blue, gold, teal, red) = (number_c(), position_c(), answer_c(), slide_c(), warn_c());
        let (a, b) = (fact_f64("slide.a")?, fact_f64("slide.b")?);
        let (c4, d6) = (fact_f64("slide.c")?, fact_f64("slide.d")?);
        let d = d_scale();

        // 02_1: two copies of the ruler, one fixed, one sliding
        kit.say(stage, "02_1")?;
        let heading = title(&kit, stage, "A slide rule")?;
        let fixed = d.build(&kit, stage, green, 22.0)?;
        let d_name = kit.text(stage, "D", 26.0, green, v(ONE_X - 0.5, D_Y))?;
        fade_in_all(stage, &[heading, fixed, d_name], 0.7)?;
        let moving = c_scale(0.0).build(&kit, stage, teal, 22.0)?;
        let c_name = kit.text(stage, "C", 26.0, teal, v(ONE_X - 0.5, C_Y))?;
        fade_in_all(stage, &[moving, c_name], 0.7)?;
        // "one that slides along it": it does, a little, and comes back
        until_phrase(stage, &kit, "02_1", NUDGE_PHRASE)?;
        shift_all(stage, &[moving, c_name], v(0.8, 0.0), 0.6)?;
        shift_all(stage, &[moving, c_name], v(-0.8, 0.0), 0.6)?;
        let mut offset = 0.0;
        kit.until_sentence(stage, "02_1", 1)?;
        let full = seg(stage, v(d.x(1.0), D_BAR_Y), v(d.x(10.0), D_BAR_Y), blue, 5.0);
        let full_word = kit.text(
            stage,
            "one full length = 1: from 1 to 10, ×10",
            22.0,
            blue,
            v(ONE_X + LEN / 2.0, D_BAR_Y - 0.35),
        )?;
        fade_in_all(stage, &[full, full_word], 0.6)?;
        kit.until_sentence(stage, "02_1", 2)?;
        fade_out_all(stage, &[full, full_word], 0.5)?;

        // 02_2: 2.5 × 3.2 — C's 1 over D's 2.5, then read under C's 3.2
        kit.say(stage, "02_2")?;
        let ask = kit.text(
            stage,
            &format!("{} × {} = ?", fact("slide.a")?, fact("slide.b")?),
            34.0,
            Palette::ink(),
            v(4.7, 2.3),
        )?;
        let d_len = seg(stage, v(d.x(1.0), D_BAR_Y), v(d.x(a), D_BAR_Y), blue, 6.0);
        let d_len_word = kit.text(
            stage,
            &format!("{} of a full length", fact("slide.pos_a")?),
            22.0,
            blue,
            v((d.x(1.0) + d.x(a)) / 2.0, D_BAR_Y - 0.32),
        )?;
        fade_in_all(stage, &[ask, d_len, d_len_word], 0.6)?;
        let shift_a = LEN * a.log10();
        until_phrase(stage, &kit, "02_2", SLIDE_PHRASE)?;
        shift_all(stage, &[moving, c_name], v(shift_a, 0.0), 1.6)?;
        offset += shift_a;
        let index = ring(stage, c_scale(offset).at(1.0), 0.16, teal, 3.0);
        fade_in_all(stage, &[index], 0.3)?;
        kit.until_sentence(stage, "02_2", 1)?;
        let carried = super::draw::arrow(
            stage,
            v(ONE_X, C_BAR_Y + 0.35),
            v(ONE_X + offset, C_BAR_Y + 0.35),
            blue,
            3.0,
        )?;
        let carried_word = kit.text(
            stage,
            &format!("every mark moved by {}'s length", fact("slide.a")?),
            22.0,
            blue,
            v(ONE_X + offset / 2.0, C_BAR_Y + 0.7),
        )?;
        fade_in_all(stage, &[carried, carried_word], 0.6)?;
        kit.until_sentence(stage, "02_2", 2)?;
        let c = c_scale(offset);
        let target = ring(stage, c.at(b), 0.16, teal, 3.0);
        let c_len = seg(stage, v(c.x(1.0), C_BAR_Y), v(c.x(b), C_BAR_Y), blue, 6.0);
        let c_len_word = kit.text(
            stage,
            fact("slide.pos_b")?,
            22.0,
            blue,
            v((c.x(1.0) + c.x(b)) / 2.0, C_BAR_Y + 0.32),
        )?;
        fade_out_all(stage, &[carried, carried_word], 0.3)?;
        fade_in_all(stage, &[target, c_len, c_len_word], 0.6)?;
        kit.until_sentence(stage, "02_2", 3)?;
        let read_x = c.x(b);
        let line = hairline(stage, read_x, gold);
        let below = kit.text(stage, "?", 30.0, gold, v(read_x, D_Y - 1.1))?;
        fade_in_all(stage, &[line, below], 0.5)?;
        predict(&kit, stage, v(4.7, -2.75), 3.0)?;

        // 02_3: eight; the two lengths, end to end on D, reach 8
        kit.say(stage, "02_3")?;
        let eight = ring(stage, d.at(fact_f64("slide.ab")?), 0.18, gold, 3.5);
        let solved = kit.text(
            stage,
            &format!("{} × {} = {}", fact("slide.a")?, fact("slide.b")?, fact("slide.ab")?),
            34.0,
            gold,
            v(4.7, 2.3),
        )?;
        swap(stage, &[ask, below], &[solved, eight], 0.5)?;
        kit.until_sentence(stage, "02_3", 1)?;
        // C's length drops onto D's row, starting where D's length ends
        fade_out_all(stage, &[c_len_word], 0.2)?;
        shift_all(stage, &[c_len], v(0.0, D_BAR_Y - C_BAR_Y), 1.2)?;
        let c_word_fix = kit.text(
            stage,
            fact("slide.pos_b")?,
            22.0,
            blue,
            v((c.x(1.0) + c.x(b)) / 2.0, D_BAR_Y - 0.32),
        )?;
        fade_in_all(stage, &[c_word_fix], 0.3)?;
        let sum = kit.tex(
            stage,
            &format!(
                r"{} + {} = {} = \mathrm{{pos}}({})",
                fact("slide.pos_a")?,
                fact("slide.pos_b")?,
                fact("slide.pos_ab")?,
                fact("slide.ab")?
            ),
            32.0,
            blue,
            &[],
            v(3.2, -1.65),
        )?;
        fade_in_all(stage, &[sum], 0.6)?;
        kit.until_sentence(stage, "02_3", 3)?;
        let moral = kit.text(
            stage,
            "sliding adds the lengths; the marks turn the sum back into a product",
            24.0,
            Palette::ink(),
            v(0.0, -2.6),
        )?;
        fade_in_all(stage, &[moral], 0.6)?;

        // 02_4: 4 × 6 — the six runs off D's end
        kit.say(stage, "02_4")?;
        fade_out_all(
            stage,
            &[
                d_len, d_len_word, c_len, c_word_fix, sum, moral, line, eight, index, target,
            ],
            0.5,
        )?;
        let ask2 = kit.text(
            stage,
            &format!("{} × {} = ?", fact("slide.c")?, fact("slide.d")?),
            34.0,
            Palette::ink(),
            v(4.7, 2.3),
        )?;
        swap(stage, &[solved], &[ask2], 0.5)?;
        shift_all(stage, &[moving, c_name], v(-offset, 0.0), 1.0)?;
        offset = 0.0;
        kit.until_sentence(stage, "02_4", 1)?;
        let shift_c = LEN * c4.log10();
        shift_all(stage, &[moving, c_name], v(shift_c, 0.0), 1.6)?;
        offset += shift_c;
        let c = c_scale(offset);
        let index = ring(stage, c.at(1.0), 0.16, teal, 3.0);
        let six = ring(stage, c.at(d6), 0.16, teal, 3.0);
        fade_in_all(stage, &[index, six], 0.5)?;
        kit.until_sentence(stage, "02_4", 2)?;
        let end = seg(
            stage,
            v(d.x(10.0), D_Y - HAIR_REACH),
            v(d.x(10.0), D_Y + 0.15),
            red,
            4.0,
        );
        let beyond = dashed(
            stage,
            v(d.x(10.0), D_Y),
            v(c.x(d6) + 0.6, D_Y),
            Palette::faint(),
            2.5,
            9,
        )?;
        let off_line = hairline(stage, c.x(d6), red);
        let off = kit.text(stage, "off the end", 24.0, red, v(c.x(d6), D_Y - 1.1))?;
        fade_in_all(stage, &[end, beyond, off_line, off], 0.6)?;
        kit.until_sentence(stage, "02_4", 3)?;
        let ghost = kit.text(
            stage,
            &format!("{} would sit here", fact("slide.cd")?),
            22.0,
            gold,
            v(c.x(d6), D_Y - 1.5),
        )?;
        let stops = kit.text(stage, "D stops at 10", 22.0, red, v(d.x(10.0), D_Y - 1.1))?;
        fade_in_all(stage, &[ghost, stops], 0.5)?;

        // 02_5: the other index; six over 2.4; the decimal point is ours
        kit.say(stage, "02_5")?;
        kit.until_sentence(stage, "02_5", 1)?;
        // the overflow is over, so its marks go; C's name would slide off the frame
        fade_out_all(
            stage,
            &[six, index, off_line, off, ghost, beyond, end, stops, c_name],
            0.4,
        )?;
        shift_all(stage, &[moving], v(-LEN, 0.0), 1.8)?;
        offset -= LEN;
        let c = c_scale(offset);
        let ten = ring(stage, c.at(10.0), 0.16, teal, 3.0);
        let four = ring(stage, d.at(c4), 0.16, green, 3.0);
        let c_again = kit.text(stage, "C", 26.0, teal, v(c.x(10.0) + C_NAME_DX, C_Y))?;
        fade_in_all(stage, &[ten, four, c_again], 0.5)?;
        kit.until_sentence(stage, "02_5", 2)?;
        let digits = fact_f64("slide.cd_digits")?;
        let line = hairline(stage, c.x(d6), gold);
        let six = ring(stage, c.at(d6), 0.16, teal, 3.0);
        let mark = ring(stage, d.at(digits), 0.18, gold, 3.5);
        fade_in_all(stage, &[line, six, mark], 0.6)?;
        kit.until_sentence(stage, "02_5", 3)?;
        let digits_word = kit.text(
            stage,
            &format!(
                "the rule gives the digits: {}",
                fact("slide.cd_digits")?.replace('.', " ")
            ),
            24.0,
            Palette::ink(),
            v(3.2, -1.2),
        )?;
        fade_in_all(stage, &[digits_word], 0.5)?;
        kit.until_sentence(stage, "02_5", 4)?;
        let x = d.x(digits);
        let mut tags = Vec::new();
        for (k, key) in ["slide.cd_digits", "slide.cd"].iter().enumerate() {
            tags.push(kit.text(
                stage,
                fact(key)?,
                24.0,
                Palette::dim(),
                v(x, D_Y - 1.05 - 0.42 * k as f64),
            )?);
        }
        let hundreds = super::draw::fact_list("dial.values")?;
        let last = hundreds.last().copied().unwrap_or(0.0);
        tags.push(kit.text(stage, &format!("{last}"), 24.0, Palette::dim(), v(x, D_Y - 1.89))?);
        // a bracket over all three rows: they are one mark
        let bracket = super::draw::path(
            stage,
            vec![
                v(x + 0.48, D_Y - 0.93),
                v(x + 0.6, D_Y - 0.93),
                v(x + 0.6, D_Y - 2.01),
                v(x + 0.48, D_Y - 2.01),
            ],
            Palette::dim(),
            2.0,
        );
        let same = kit.text_left(stage, "one mark", 20.0, Palette::dim(), v(x + 0.72, D_Y - 1.47))?;
        super::draw::fade_in_lagged(stage, &tags, 1.2, 0.4)?;
        fade_in_all(stage, &[bracket, same], 0.4)?;
        kit.until_sentence(stage, "02_5", 5)?;
        let reason = kit.text(
            stage,
            &format!("{} × {} lies between 10 and 100", fact("slide.c")?, fact("slide.d")?),
            24.0,
            Palette::ink(),
            v(3.2, -1.7),
        )?;
        let picked = kit.text(stage, fact("slide.cd")?, 24.0, gold, v(x, D_Y - 1.47))?;
        let answer = kit.text(
            stage,
            &format!("{} × {} = {}", fact("slide.c")?, fact("slide.d")?, fact("slide.cd")?),
            34.0,
            gold,
            v(4.7, 2.3),
        )?;
        fade_in_all(stage, &[reason], 0.5)?;
        swap(stage, &[tags[1], ask2], &[picked, answer], 0.5)?;
        let fine = precisely(
            &kit,
            stage,
            "a working slide rule reads about three significant figures; the user places the decimal point",
        )?;
        fade_in_all(stage, &[fine], 0.5)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The scene code without these tests, for checks on the order of its beats.
    fn scene_source() -> &'static str {
        include_str!("slide_rule.rs").split("#[cfg(test)]").next().unwrap_or("")
    }

    fn found(text: &str) -> usize {
        scene_source()
            .find(text)
            .unwrap_or_else(|| panic!("not in the scene: {text}"))
    }

    #[test]
    fn sliding_to_two_point_five_puts_three_point_two_over_eight() {
        let c = c_scale(LEN * 2.5f64.log10());
        assert!((c.x(1.0) - d_scale().x(2.5)).abs() < 1e-12);
        assert!((c.x(3.2) - d_scale().x(8.0)).abs() < 1e-9);
    }

    #[test]
    fn four_times_six_overflows_onto_the_screen_then_the_other_index_reads_two_point_four() {
        let d = d_scale();
        let left = c_scale(LEN * 4f64.log10());
        let over = left.x(6.0);
        assert!(over > d.x(10.0) + 1.0 && over < 6.6, "{over}");
        assert!((over - d.x(24.0)).abs() < 1e-9);
        let right = c_scale(LEN * 4f64.log10() - LEN);
        assert!((right.x(10.0) - d.x(4.0)).abs() < 1e-9);
        assert!((right.x(6.0) - d.x(2.4)).abs() < 1e-9);
        // the visible part of C after the shift includes its 6 and its 10
        assert!(right.x(6.0) > -7.0 && right.x(10.0) < 7.0);
    }

    #[test]
    fn the_scales_fit_the_frame() {
        let d = d_scale();
        assert!(d.x(1.0) - 0.6 > -7.1 && d.x(10.0) < 7.1);
        // C at the 2.5 slide still shows its 3.2 and its 10 on screen
        let c = c_scale(LEN * 2.5f64.log10());
        assert!(c.x(10.0) < 7.1, "{}", c.x(10.0));
    }

    #[test]
    fn the_reading_line_passes_the_ticks_and_stops_short_of_the_numbers() {
        // Decade::build: numbers centred 0.58 from the spine; ticks reach 0.32.
        // Digits are 0.69 em tall at 22 points; the line's round cap adds 0.0125.
        let number_starts = 0.58 - 0.69 * 22.0 * 0.01 / 2.0;
        let line_ends = HAIR_REACH + 2.5 * 0.01 / 2.0;
        assert!(line_ends > 0.32 + 0.05, "{line_ends}");
        assert!(number_starts - line_ends >= 0.05, "{number_starts} {line_ends}");
        // the red end tick uses the same reach, so it leaves D's 10 whole too
        assert!(found("v(d.x(10.0), D_Y - HAIR_REACH)") > 0);
    }

    #[test]
    fn the_slide_waits_for_words_that_are_in_the_first_sentence() {
        let text = SCRIPT
            .lines()
            .find_map(|row| row.strip_prefix("02_2")?.strip_prefix('\t'))
            .expect("02_2 is in the script");
        let phrase = text.find(SLIDE_PHRASE).expect("the phrase is in 02_2");
        assert!(phrase < text.find(". ").expect("02_2 has two sentences"));
        assert!(found("until_phrase(stage, &kit, \"02_2\", SLIDE_PHRASE)") < found("v(shift_a, 0.0)"));
    }

    #[test]
    fn the_nudge_waits_for_its_words_and_is_back_before_the_next_sentence() {
        let text = SCRIPT
            .lines()
            .find_map(|row| row.strip_prefix("02_1\t"))
            .expect("02_1 is in the script");
        let phrase = text.find(NUDGE_PHRASE).expect("the phrase is in 02_1");
        assert!(phrase < text.find(". ").expect("02_1 has several sentences"));
        let wait = found("until_phrase(stage, &kit, \"02_1\", NUDGE_PHRASE)");
        let out = found("v(0.8, 0.0)");
        let back = found("v(-0.8, 0.0)");
        let next = found("kit.until_sentence(stage, \"02_1\", 1)");
        assert!(wait < out && out < back && back < next);
    }

    #[test]
    fn the_overflow_marks_leave_before_the_other_index_moves_in() {
        let fade = found("&[six, index, off_line, off, ghost, beyond, end, stops, c_name]");
        assert!(fade < found("v(-LEN, 0.0)"));
    }

    #[test]
    fn the_card_changes_on_the_new_question_before_the_slide_home() {
        assert!(found("swap(stage, &[solved], &[ask2], 0.5)") < found("v(-offset, 0.0)"));
    }

    #[test]
    fn c_keeps_its_name_on_screen_clear_of_the_ten_ring() {
        let c = c_scale(LEN * 4f64.log10() - LEN);
        let at = c.x(10.0) + C_NAME_DX;
        assert!(at.abs() < 6.86, "{at}");
        let half_letter = 0.722 * 26.0 * 0.01 / 2.0;
        assert!(at - half_letter - (c.x(10.0) + 0.16) >= 0.1, "{at}");
    }

    #[test]
    fn the_printed_lengths_are_fractions_of_one_full_length() {
        let d = d_scale();
        for (x, printed) in [(2.5, 0.398), (3.2, 0.505), (8.0, 0.903)] {
            assert!(((d.x(x) - d.x(1.0)) / LEN - printed).abs() < 0.001, "{x}");
        }
        assert!(found("one full length = 1") > 0);
        // the first length carries its unit: "0.398 of a full length" is 9.06 ems at
        // 22 points, narrower than its own bar
        assert!(found("\"{} of a full length\"") > 0);
        assert!(9.06 * 22.0 * 0.01 < d.x(2.5) - d.x(1.0));
    }
}
