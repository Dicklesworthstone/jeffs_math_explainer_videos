//! Chapter 01, where does three go: the ratio ruler built from doublings
//! (01_1), the one rule pos(ab) = pos(a) + pos(b) placing ½ and √2 (01_2),
//! the three question (01_3), twelve threes against nineteen twos (01_4),
//! the keyboard's nineteen-key near-tripling and the true 1.585 (01_5), and
//! the name: base two, base ten, the same ruler stretched (01_6).
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`; picture repairs
//! from the independent review staged in `logarithm-repair01-04`. Type-checked
//! against stand-ins only: not compiled against fmn, unrendered, unlistened.

use explainer_kit::{Kit, Palette, play, thousands, tween, v};
use fmn::prelude::*;

use super::draw::{
    answer_c, fact, fact_f64, fade_in_all, fade_in_lagged, fade_out_all, faded_line, line_style, number_c, position_c,
    predict, seg, swap, title, value,
};
use crate::model;

pub struct BuildingRuler {
    pub kit: Kit,
}

/// The ruler: where 1 sits, and the scene length of one doubling.
const ONE_X: f64 = -3.6;
const STEP: f64 = 2.0;
const SPINE_Y: f64 = 0.3;
const LEFT: f64 = -6.3;
const RIGHT: f64 = 6.3;
/// Where the step words sit: under both rows of lengths.
const WORD_Y: f64 = SPINE_Y - 0.92;
/// The blue lengths: each stops `INSET` short of its marks. A stroke has round
/// caps of half its width, so this inset is what leaves equal lengths a gap.
const LENGTH_W: f64 = 6.0;
const INSET: f64 = 0.06;
/// Where the hunter rests during the question: the plain middle of the gap, a
/// guess the viewer can reject, not the answer.
const HUNT_REST: f64 = 1.5;
/// The compressed rows of 01_4: nineteen doublings span `ROW_SPAN` from
/// `ROW_LEFT`; their labels start at `ROW_LABEL_X`.
const ROW_LEFT: f64 = -6.0;
const ROW_SPAN: f64 = 11.0;
const ROW_W: f64 = 5.0;
const ROW_INSET: f64 = 0.05;
const ROW_LABEL_X: f64 = 5.15;

fn at(x: f64) -> f64 {
    ONE_X + STEP * model::position(x, 2.0)
}

/// A labelled mark on the ruler: tick up, number above in green.
fn mark(kit: &Kit, stage: &mut Stage<'_>, x: f64, label: &str, c: Srgb) -> fmn::Result<[Mob; 2]> {
    let px = at(x);
    let tick = seg(stage, v(px, SPINE_Y), v(px, SPINE_Y + 0.3), c, 3.0);
    let word = kit.text(stage, label, 28.0, c, v(px, SPINE_Y + 0.65))?;
    Ok([tick, word])
}

/// The height of row `row` of lengths (0 just under the spine).
fn row_y(row: f64) -> f64 {
    SPINE_Y - 0.3 - 0.28 * row
}

/// A blue length under the ruler from position `a` to `b` (in steps), on
/// row `row`, with visible gaps to its neighbours.
fn length(stage: &mut Stage<'_>, a: f64, b: f64, row: f64) -> Mob {
    let y = row_y(row);
    seg(
        stage,
        v(ONE_X + STEP * a + INSET, y),
        v(ONE_X + STEP * b - INSET, y),
        position_c(),
        LENGTH_W,
    )
}

/// Piece `k` of a compressed row, `len` long from `x`: alternate pieces are
/// dimmer, so the pieces can be counted.
fn piece(stage: &mut Stage<'_>, x: f64, len: f64, y: f64, c: Srgb, k: u32) -> Mob {
    let o = if k.is_multiple_of(2) { 1.0 } else { 0.55 };
    stage.arena_mut().add(
        polyline(vec![v(x + ROW_INSET, y), v(x + len - ROW_INSET, y)])
            .style(faded_line(c, ROW_W, o))
            .build(),
    )
}

impl SceneConstruct for BuildingRuler {
    fn name(&self) -> &str {
        "01_building_ruler"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (green, blue, gold) = (number_c(), position_c(), answer_c());

        // 01_1: one at the start; each doubling adds one step
        kit.say(stage, "01_1")?;
        let heading = title(&kit, stage, "Building the ruler")?;
        let spine = seg(stage, v(LEFT, SPINE_Y), v(RIGHT, SPINE_Y), Palette::dim(), 3.0);
        fade_in_all(stage, &[heading, spine], 0.6)?;
        kit.until_sentence(stage, "01_1", 1)?;
        let one = mark(&kit, stage, 1.0, "1", green)?;
        fade_in_all(stage, &one, 0.5)?;
        kit.until_sentence(stage, "01_1", 2)?;
        let step_word = kit.text(stage, "one step", 24.0, blue, v(ONE_X + STEP / 2.0, WORD_Y))?;
        let first = length(stage, 0.0, 1.0, 0.0);
        fade_in_all(stage, &[first, step_word], 0.5)?;
        kit.until_sentence(stage, "01_1", 3)?;
        let mut doublings = Vec::new();
        let mut later_steps = Vec::new();
        for (k, n) in [(1.0, "2"), (2.0, "4"), (3.0, "8")] {
            let m = mark(&kit, stage, 2f64.powf(k), n, green)?;
            let l = length(stage, k - 1.0, k, 0.0);
            doublings.extend(m);
            doublings.push(l);
            later_steps.push(l);
        }
        fade_in_lagged(stage, &doublings[..3], 0.8, 0.3)?;
        fade_in_lagged(stage, &doublings[3..], 1.4, 0.3)?;
        kit.until_sentence(stage, "01_1", 4)?;
        let each = kit.text(
            stage,
            "every doubling adds the same length",
            26.0,
            Palette::ink(),
            v(0.0, -2.9),
        )?;
        fade_in_all(stage, &[each], 0.5)?;

        // 01_2: the rule; a half one step back; √2 half a step on
        kit.say(stage, "01_2")?;
        let rule = kit.tex(
            stage,
            r"\mathrm{pos}(a \times b) = \mathrm{pos}(a) + \mathrm{pos}(b)",
            44.0,
            Palette::ink(),
            &[(r"\mathrm{pos}", blue)],
            v(0.0, 2.35),
        )?;
        swap(stage, &[each], &[rule], 0.6)?;
        kit.until_sentence(stage, "01_2", 1)?;
        let half = mark(&kit, stage, 0.5, "½", green)?;
        let back = length(stage, -1.0, 0.0, 1.0);
        fade_in_all(stage, &[half[0], half[1], back], 0.6)?;
        kit.until_sentence(stage, "01_2", 2)?;
        let root = mark(&kit, stage, 2f64.sqrt(), "√2", green)?;
        let h1 = length(stage, 0.0, 0.5, 1.0);
        let h2 = length(stage, 0.5, 1.0, 1.0);
        fade_in_all(stage, &[root[0], root[1]], 0.4)?;
        fade_in_lagged(stage, &[h1, h2], 0.9, 0.5)?;
        let both = kit.text(
            stage,
            "√2 × √2 = 2:  two half steps make one",
            24.0,
            blue,
            v(ONE_X + 1.0, SPINE_Y - 1.35),
        )?;
        fade_in_all(stage, &[both], 0.5)?;

        // 01_3: where does three go? A marker hunts between two and four
        kit.say(stage, "01_3")?;
        fade_out_all(stage, &[both, back, h1, h2], 0.4)?;
        kit.until_sentence(stage, "01_3", 1)?;
        let heading2 = title(&kit, stage, "Where does three go?")?;
        swap(stage, &[heading], &[heading2], 0.5)?;
        let hunt = stage.add_value_tracker(1.2);
        let hunter = stage.always_redraw(move |s| {
            let p = ONE_X + STEP * value(s, hunt);
            s.add(
                Polygon::new([
                    v(p, SPINE_Y + 0.05),
                    v(p - 0.14, SPINE_Y - 0.3),
                    v(p + 0.14, SPINE_Y - 0.3),
                ])
                .style(super::draw::flat(gold, 1.0))
                .build(),
            )
        });
        let three_q = kit.text(stage, "3 ?", 30.0, gold, v(ONE_X + STEP * 1.5, SPINE_Y - 0.75))?;
        stage.add_to_scene(hunter)?;
        fade_in_all(stage, &[three_q], 0.4)?;
        kit.until_sentence(stage, "01_3", 2)?;
        play!(stage; tween(hunt, 1.85, 1.0)?);
        play!(stage; tween(hunt, 1.3, 1.0)?);
        play!(stage; tween(hunt, HUNT_REST, 0.8)?);
        predict(&kit, stage, v(3.8, -2.9), 3.0)?;

        // 01_4: twelve threes against nineteen twos, laid end to end
        kit.say(stage, "01_4")?;
        let (p3, p2) = (fact("ruler.three_pow12")?, fact("ruler.two_pow19")?);
        let (n3, n2) = (fact_f64("ruler.three_pow")? as u32, fact_f64("ruler.two_pow")? as u32);
        let p3n: u64 = p3.parse().map_err(|_| super::draw::fail("ruler.three_pow12".into()))?;
        let p2n: u64 = p2.parse().map_err(|_| super::draw::fail("ruler.two_pow19".into()))?;
        kit.until_sentence(stage, "01_4", 1)?;
        let threes = kit.tex(
            stage,
            &format!(r"3^{{{n3}}} = {}", thousands(p3n).replace(',', "{,}")),
            40.0,
            gold,
            &[],
            v(-3.2, -1.45),
        )?;
        fade_in_all(stage, &[threes], 0.6)?;
        kit.until_sentence(stage, "01_4", 2)?;
        let twos = kit.tex(
            stage,
            &format!(r"2^{{{n2}}} = {}", thousands(p2n).replace(',', "{,}")),
            40.0,
            blue,
            &[],
            v(3.2, -1.45),
        )?;
        fade_in_all(stage, &[twos], 0.6)?;
        kit.until_sentence(stage, "01_4", 3)?;
        let gap = kit.text(
            stage,
            &format!("{}% apart", fact("ruler.power_gap_percent")?),
            26.0,
            Palette::ink(),
            v(0.0, -1.45),
        )?;
        fade_in_all(stage, &[gap], 0.4)?;
        kit.until_sentence(stage, "01_4", 4)?;
        // the compressed rows: nineteen blue steps; twelve gold copies of
        // three's (true) length, which overshoot by a sliver
        let unit = ROW_SPAN / f64::from(n2);
        let (row_b, row_g) = (-2.35, -2.7);
        let mut steps = Vec::new();
        for k in 0..n2 {
            steps.push(piece(stage, ROW_LEFT + unit * f64::from(k), unit, row_b, blue, k));
        }
        let three_len = unit * model::position(3.0, 2.0);
        let mut copies = Vec::new();
        for k in 0..n3 {
            copies.push(piece(
                stage,
                ROW_LEFT + three_len * f64::from(k),
                three_len,
                row_g,
                gold,
                k,
            ));
        }
        let rows_label_b = kit.text_left(stage, &format!("{n2} doublings"), 22.0, blue, v(ROW_LABEL_X, row_b))?;
        let rows_label_g = kit.text_left(stage, &format!("{n3} threes"), 22.0, gold, v(ROW_LABEL_X, row_g))?;
        fade_out_all(stage, &[three_q], 0.3)?;
        fade_in_lagged(stage, &steps, 1.6, 0.08)?;
        fade_in_lagged(stage, &copies, 1.6, 0.12)?;
        fade_in_all(stage, &[rows_label_b, rows_label_g], 0.4)?;
        // three's mark lands at nineteen twelfths of a step
        let estimate = f64::from(n2) / f64::from(n3);
        play!(stage; tween(hunt, estimate, 1.2)?);
        let est_word = kit.tex(
            stage,
            &format!(
                r"\mathrm{{pos}}(3) \approx \tfrac{{{n2}}}{{{n3}}} \approx {}",
                fact("ruler.nineteen_twelfths")?
            ),
            36.0,
            gold,
            &[],
            v(ONE_X + STEP * estimate + 1.7, SPINE_Y - 0.75),
        )?;
        fade_in_all(stage, &[est_word], 0.5)?;

        // 01_5: the keyboard callback; then the true position
        kit.say(stage, "01_5")?;
        kit.until_sentence(stage, "01_5", 1)?;
        let keys = kit.text(
            stage,
            &format!(
                "A at {a} Hz, 19 semitones up: {} Hz    3 × {a} = {} Hz",
                fact("music.nineteen_keys_hz")?,
                fact("music.three_times_hz")?,
                a = fact("music.start_hz")?,
            ),
            26.0,
            Palette::ink(),
            v(0.0, -1.45),
        )?;
        swap(stage, &[threes, twos, gap], &[keys], 0.6)?;
        kit.until_sentence(stage, "01_5", 2)?;
        let truth = model::position(3.0, 2.0);
        play!(stage; tween(hunt, truth, 1.0)?);
        let true_word = kit.tex(
            stage,
            &format!(r"\mathrm{{pos}}(3) \approx {}", fact("ruler.log2_3")?),
            36.0,
            gold,
            &[],
            v(ONE_X + STEP * truth + 1.7, SPINE_Y - 0.75),
        )?;
        swap(stage, &[est_word], &[true_word], 0.5)?;
        let three = mark(&kit, stage, 3.0, "3", green)?;
        fade_in_all(stage, &three, 0.4)?;
        kit.until_sentence(stage, "01_5", 3)?;
        let fixed = kit.text(
            stage,
            "bigger numbers further along: the rule fixes every mark",
            24.0,
            Palette::dim(),
            v(0.0, 1.55),
        )?;
        fade_in_all(stage, &[fixed], 0.5)?;

        // 01_6: the name; base ten is the same ruler stretched
        kit.say(stage, "01_6")?;
        let mut clear = steps.clone();
        clear.extend(copies.iter().copied());
        clear.extend([rows_label_b, rows_label_g, keys, fixed, hunter]);
        fade_out_all(stage, &clear, 0.5)?;
        kit.until_sentence(stage, "01_6", 1)?;
        let name = kit.tex(
            stage,
            r"\mathrm{pos}(x) = \log_2 x",
            44.0,
            Palette::ink(),
            &[(r"\log_2", blue)],
            v(0.0, 2.35),
        )?;
        let heading3 = title(&kit, stage, "The logarithm")?;
        swap(stage, &[rule, heading2], &[name, heading3], 0.6)?;
        kit.until_sentence(stage, "01_6", 2)?;
        let samples = kit.tex(
            stage,
            &format!(
                r"\log_2 8 = {} \qquad \log_2 3 \approx {}",
                fact("ruler.log2_8")?,
                fact("ruler.log2_3")?
            ),
            36.0,
            blue,
            &[],
            v(0.0, -1.45),
        )?;
        fade_in_all(stage, &[samples], 0.5)?;
        kit.until_sentence(stage, "01_6", 3)?;
        // A new first step changes every position, so everything measured in
        // doublings goes first. The marks stay exactly where they are: only the
        // unit changes. The first step's bar grows until it lands on 4, base 4.
        let reach = stage.add_value_tracker(1.0);
        let step_bar = stage.always_redraw(move |s| {
            let (y, to) = (row_y(0.0), ONE_X + STEP * value(s, reach) - INSET);
            s.add(
                polyline(vec![v(ONE_X + INSET, y), v(to, y)])
                    .style(line_style(blue, LENGTH_W))
                    .build(),
            )
        });
        stage.add_to_scene(step_bar)?;
        let mut retired = later_steps.clone();
        retired.extend([first, name, samples, true_word]);
        fade_out_all(stage, &retired, 0.5)?;
        play!(stage; tween(reach, model::position(4.0, 2.0), 1.2)?);
        let four_step = kit.text(stage, "one step = ×4", 24.0, blue, v(ONE_X + STEP, WORD_Y))?;
        swap(stage, &[step_word], &[four_step], 0.4)?;
        kit.until_sentence(stage, "01_6", 4)?;
        // ... then on to 10, on the same marks: three now sits at log10 3 of a step
        let ten = mark(&kit, stage, 10.0, "10", green)?;
        fade_in_all(stage, &ten, 0.4)?;
        fade_out_all(stage, &[four_step], 0.3)?;
        play!(stage; tween(reach, model::position(10.0, 2.0), 1.6)?);
        let ten_step = kit.text(
            stage,
            "now one step = ×10",
            24.0,
            blue,
            v((at(1.0) + at(10.0)) / 2.0, WORD_Y),
        )?;
        fade_in_all(stage, &[ten_step], 0.4)?;
        kit.until_sentence(stage, "01_6", 5)?;
        let stretch = kit.tex(
            stage,
            r"\log_{10} x = \frac{\log_2 x}{\log_2 10}",
            40.0,
            Palette::ink(),
            &[],
            v(3.6, -2.6),
        )?;
        let same = kit.text(
            stage,
            "every base: the same ruler, stretched",
            26.0,
            Palette::ink(),
            v(-2.6, -2.6),
        )?;
        fade_in_all(stage, &[stretch, same], 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The scene code without these tests, for checks on the order of its beats.
    fn scene_source() -> &'static str {
        include_str!("building_ruler.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap_or("")
    }

    fn found(text: &str) -> usize {
        scene_source()
            .find(text)
            .unwrap_or_else(|| panic!("not in the scene: {text}"))
    }

    /// Half the printed width of a 28-point ruler label (ems × size × 0.01).
    fn half_width(label: &str) -> f64 {
        let ems: f64 = label.chars().map(|c| if c == '√' { 0.83 } else { 0.5 }).sum();
        ems * 28.0 * 0.01 / 2.0
    }

    #[test]
    fn every_mark_from_a_half_to_ten_fits_the_ruler() {
        for x in [0.5, 1.0, 2f64.sqrt(), 2.0, 3.0, 4.0, 8.0, 10.0] {
            assert!(at(x) > LEFT && at(x) < RIGHT, "{x}");
        }
    }

    #[test]
    fn neighbouring_labels_never_touch_so_eight_and_ten_stay_two_numbers() {
        let marks = [
            (0.5, "½"),
            (1.0, "1"),
            (2f64.sqrt(), "√2"),
            (2.0, "2"),
            (3.0, "3"),
            (4.0, "4"),
            (8.0, "8"),
            (10.0, "10"),
        ];
        for pair in marks.windows(2) {
            let gap = at(pair[1].0) - at(pair[0].0) - half_width(pair[0].1) - half_width(pair[1].1);
            assert!(gap > 0.2, "{} and {}: {gap}", pair[0].1, pair[1].1);
        }
    }

    #[test]
    fn the_step_bar_lands_on_the_mark_its_words_name() {
        let end = |reach: f64| ONE_X + STEP * reach;
        assert!((end(model::position(4.0, 2.0)) - at(4.0)).abs() < 1e-12);
        assert!((end(model::position(10.0, 2.0)) - at(10.0)).abs() < 1e-12);
        // drawn ends, caps included, sit within a hair of the ticks they name
        let shortfall = INSET - LENGTH_W * 0.01 / 2.0;
        assert!((0.0..0.05).contains(&shortfall), "{shortfall}");
        // in the new unit each mark's position is its common logarithm
        let unit = at(10.0) - at(1.0);
        for x in [0.5, 2.0, 3.0, 8.0] {
            assert!(((at(x) - at(1.0)) / unit - f64::log10(x)).abs() < 1e-12, "{x}");
        }
    }

    #[test]
    fn equal_lengths_keep_a_visible_gap_after_their_round_caps() {
        let (big, small) = (2.0 * INSET - LENGTH_W * 0.01, 2.0 * ROW_INSET - ROW_W * 0.01);
        assert!(big >= 0.05, "{big}");
        assert!(small >= 0.05, "{small}");
    }

    #[test]
    fn the_half_steps_clear_the_step_words_and_the_caption_below() {
        let bar_bottom = row_y(1.0) - LENGTH_W * 0.01 / 2.0;
        let word_half = 0.88 * 24.0 * 0.01 / 2.0;
        assert!(bar_bottom - (WORD_Y + word_half) >= 0.1, "{bar_bottom}");
        // the √2 caption sits at SPINE_Y - 1.35
        assert!(WORD_Y - word_half - (SPINE_Y - 1.35 + word_half) >= 0.05);
    }

    #[test]
    fn the_hunter_rests_mid_gap_well_away_from_the_answer() {
        assert!((HUNT_REST - 1.5).abs() < 1e-12);
        let miss = (model::position(3.0, 2.0) - HUNT_REST).abs() * STEP;
        assert!(miss >= 0.15, "{miss}");
    }

    #[test]
    fn twelve_threes_overshoot_nineteen_doublings_by_a_sliver() {
        let unit = ROW_SPAN / 19.0;
        let over = 12.0 * unit * model::position(3.0, 2.0) - 19.0 * unit;
        assert!(over > 0.0 && over < 0.03, "{over}");
    }

    #[test]
    fn the_rows_stop_short_of_labels_that_stay_inside_the_safe_frame() {
        let longest = ROW_LEFT + 12.0 * ROW_SPAN / 19.0 * model::position(3.0, 2.0);
        assert!(ROW_LABEL_X - longest >= 0.1, "{longest}");
        // "19 doublings" is 5.51 ems at 22 points
        const { assert!(ROW_LABEL_X + 5.51 * 22.0 * 0.01 <= 6.86) };
        const { assert!(ROW_LEFT >= -6.86) };
    }

    #[test]
    fn base_two_claims_leave_before_the_step_changes_and_no_mark_moves() {
        let sentence = found(r#"until_sentence(stage, "01_6", 3)"#);
        let retired = found("retired.extend([first, name, samples, true_word])");
        let fade = found("fade_out_all(stage, &retired, 0.5)");
        let change = found("tween(reach, model::position(4.0, 2.0)");
        assert!(sentence < retired && retired < fade && fade < change);
        assert!(
            !scene_source().contains("move_to("),
            "the marks stay put; only the unit changes"
        );
    }

    #[test]
    fn the_later_doublings_go_with_the_first_step_so_one_bar_is_left() {
        // only `step_bar` is left under the ruler once `retired` has gone
        assert!(found("let mut retired = later_steps.clone()") < found("fade_out_all(stage, &retired, 0.5)"));
        assert_eq!(scene_source().matches("later_steps.push(l)").count(), 1);
    }
}
