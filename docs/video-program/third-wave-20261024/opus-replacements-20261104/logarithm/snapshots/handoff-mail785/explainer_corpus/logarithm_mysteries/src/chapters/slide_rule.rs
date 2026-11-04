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
//! Rebuild candidate for `mx-video-program-program-87do.63`; uncompiled,
//! unrendered, unlistened.

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
    seg(stage, v(x, D_Y - 0.62), v(x, C_Y + 0.62), c, 2.5)
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
        let mut offset = 0.0;
        kit.until_sentence(stage, "02_1", 1)?;
        let full = seg(stage, v(d.x(1.0), D_BAR_Y), v(d.x(10.0), D_BAR_Y), blue, 5.0);
        let full_word = kit.text(
            stage,
            "one full length: from 1 to 10, ×10",
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
            fact("slide.pos_a")?,
            22.0,
            blue,
            v((d.x(1.0) + d.x(a)) / 2.0, D_BAR_Y - 0.32),
        )?;
        fade_in_all(stage, &[ask, d_len, d_len_word], 0.6)?;
        let shift_a = LEN * a.log10();
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
        shift_all(stage, &[c_len, c_len_word], v(0.0, D_BAR_Y - C_BAR_Y), 1.2)?;
        let c_word_fix = kit.text(
            stage,
            fact("slide.pos_b")?,
            22.0,
            blue,
            v((c.x(1.0) + c.x(b)) / 2.0, D_BAR_Y - 0.32),
        )?;
        swap(stage, &[c_len_word], &[c_word_fix], 0.3)?;
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
        shift_all(stage, &[moving, c_name], v(-offset, 0.0), 1.0)?;
        offset = 0.0;
        let ask2 = kit.text(
            stage,
            &format!("{} × {} = ?", fact("slide.c")?, fact("slide.d")?),
            34.0,
            Palette::ink(),
            v(4.7, 2.3),
        )?;
        swap(stage, &[solved], &[ask2], 0.5)?;
        kit.until_sentence(stage, "02_4", 1)?;
        let shift_c = LEN * c4.log10();
        shift_all(stage, &[moving, c_name], v(shift_c, 0.0), 1.6)?;
        offset += shift_c;
        let c = c_scale(offset);
        let index = ring(stage, c.at(1.0), 0.16, teal, 3.0);
        let six = ring(stage, c.at(d6), 0.16, teal, 3.0);
        fade_in_all(stage, &[index, six], 0.5)?;
        kit.until_sentence(stage, "02_4", 2)?;
        let end = seg(stage, v(d.x(10.0), D_Y - 0.5), v(d.x(10.0), D_Y + 0.15), red, 4.0);
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
        let stops = kit.text(stage, "D stops at 10", 22.0, red, v(d.x(10.0), D_Y - 0.85))?;
        fade_in_all(stage, &[ghost, stops], 0.5)?;

        // 02_5: the other index; six over 2.4; the decimal point is ours
        kit.say(stage, "02_5")?;
        kit.until_sentence(stage, "02_5", 1)?;
        fade_out_all(stage, &[six, index, off_line, off, ghost, beyond], 0.4)?;
        shift_all(stage, &[moving, c_name], v(-LEN, 0.0), 1.8)?;
        offset -= LEN;
        let c = c_scale(offset);
        let ten = ring(stage, c.at(10.0), 0.16, teal, 3.0);
        let four = ring(stage, d.at(c4), 0.16, green, 3.0);
        fade_in_all(stage, &[ten, four], 0.5)?;
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
        let same = kit.text(stage, "one mark", 20.0, Palette::dim(), v(x + 1.1, D_Y - 1.47))?;
        super::draw::fade_in_lagged(stage, &tags, 1.2, 0.4)?;
        fade_in_all(stage, &[same], 0.4)?;
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
}
