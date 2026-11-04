//! Chapter 01, where does three go: the ratio ruler built from doublings
//! (01_1), the one rule pos(ab) = pos(a) + pos(b) placing ½ and √2 (01_2),
//! the three question (01_3), twelve threes against nineteen twos (01_4),
//! the keyboard's nineteen-key near-tripling and the true 1.585 (01_5), and
//! the name: base two, base ten, the same ruler stretched (01_6).
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`; uncompiled,
//! unrendered, unlistened.

use explainer_kit::{Kit, Palette, play, thousands, tween, v};
use fmn::prelude::*;

use super::draw::{
    answer_c, fact, fact_f64, fade_in_all, fade_in_lagged, fade_out_all, number_c, position_c, predict, seg, swap,
    title, value,
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

/// A blue length under the ruler from position `a` to `b` (in steps), on
/// row `row` (0 just under the spine).
fn length(stage: &mut Stage<'_>, a: f64, b: f64, row: f64) -> Mob {
    let y = SPINE_Y - 0.3 - 0.28 * row;
    seg(stage, v(ONE_X + STEP * a, y), v(ONE_X + STEP * b, y), position_c(), 6.0)
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
        let heading = title(&kit, stage, "Where does three go?")?;
        let spine = seg(stage, v(LEFT, SPINE_Y), v(RIGHT, SPINE_Y), Palette::dim(), 3.0);
        fade_in_all(stage, &[heading, spine], 0.6)?;
        kit.until_sentence(stage, "01_1", 1)?;
        let one = mark(&kit, stage, 1.0, "1", green)?;
        fade_in_all(stage, &one, 0.5)?;
        kit.until_sentence(stage, "01_1", 2)?;
        let step_word = kit.text(stage, "one step", 24.0, blue, v(ONE_X + STEP / 2.0, SPINE_Y - 0.72))?;
        let first = length(stage, 0.0, 1.0, 0.0);
        fade_in_all(stage, &[first, step_word], 0.5)?;
        kit.until_sentence(stage, "01_1", 3)?;
        let mut doublings = Vec::new();
        for (k, n) in [(1.0, "2"), (2.0, "4"), (3.0, "8")] {
            let m = mark(&kit, stage, 2f64.powf(k), n, green)?;
            let l = length(stage, k - 1.0, k, 0.0);
            doublings.extend(m);
            doublings.push(l);
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
        play!(stage; tween(hunt, 1.55, 0.8)?);
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
        let unit = 11.4 / f64::from(n2);
        let (row_b, row_g) = (-2.35, -2.7);
        let mut steps = Vec::new();
        for k in 0..n2 {
            let x = -5.7 + unit * f64::from(k);
            steps.push(seg(stage, v(x + 0.02, row_b), v(x + unit - 0.02, row_b), blue, 6.0));
        }
        let three_len = unit * model::position(3.0, 2.0);
        let mut copies = Vec::new();
        for k in 0..n3 {
            let x = -5.7 + three_len * f64::from(k);
            copies.push(seg(
                stage,
                v(x + 0.02, row_g),
                v(x + three_len - 0.02, row_g),
                gold,
                6.0,
            ));
        }
        let rows_label_b = kit.text(stage, &format!("{n2} doublings"), 22.0, blue, v(6.35, row_b))?;
        let rows_label_g = kit.text(stage, &format!("{n3} threes"), 22.0, gold, v(6.35, row_g))?;
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
                r"\mathrm{{pos}}(3) \approx \tfrac{{{n2}}}{{{n3}}} = {}",
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
            v(0.0, -3.3),
        )?;
        fade_in_all(stage, &[keys], 0.6)?;
        kit.until_sentence(stage, "01_5", 2)?;
        let truth = model::position(3.0, 2.0);
        play!(stage; tween(hunt, truth, 1.0)?);
        let true_word = kit.tex(
            stage,
            &format!(r"\mathrm{{pos}}(3) = {}", fact("ruler.log2_3")?),
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
        clear.extend([rows_label_b, rows_label_g, threes, twos, gap, keys, fixed, hunter]);
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
        swap(stage, &[rule], &[name], 0.6)?;
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
        kit.until_sentence(stage, "01_6", 4)?;
        // squeeze the ruler about its 1 until ten sits where two was: the
        // marks keep their order and spacing ratios; only the step changes
        let squeeze = model::position(2.0, 10.0);
        let mut marks: Vec<Mob> = Vec::new();
        marks.extend(one);
        marks.extend(half);
        marks.extend(root);
        marks.extend(three);
        marks.extend(doublings.iter().copied());
        let ten = mark(&kit, stage, 10.0, "10", green)?;
        let mut moves: Vec<Box<dyn Animation>> = Vec::new();
        for &m in marks.iter().chain(ten.iter()) {
            let c = stage.get_center(m);
            let to = v(ONE_X + (c[0] - ONE_X) * squeeze, c[1]);
            let a = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(2.2),
                    ..AnimateArgs::default()
                })?
                .move_to(to, ORIGIN)?;
            moves.push(stage.prepare(a)?);
        }
        fade_in_all(stage, &ten, 0.4)?;
        stage.play_prepared(moves)?;
        let ten_step = kit.text(
            stage,
            "now one step = ×10",
            24.0,
            blue,
            v(ONE_X + STEP * 0.5, SPINE_Y - 0.72),
        )?;
        swap(stage, &[step_word], &[ten_step], 0.5)?;
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

    #[test]
    fn every_mark_from_a_half_to_ten_fits_the_ruler() {
        for x in [0.5, 1.0, 2f64.sqrt(), 2.0, 3.0, 4.0, 8.0] {
            assert!(at(x) > LEFT && at(x) < RIGHT, "{x}");
        }
        // ten only appears after the squeeze, where it sits on two's old place
        let squeeze = model::position(2.0, 10.0);
        assert!((ONE_X + (at(10.0) - ONE_X) * squeeze - at(2.0)).abs() < 1e-12);
    }

    #[test]
    fn twelve_threes_overshoot_nineteen_doublings_by_a_sliver() {
        let unit = 11.4 / 19.0;
        let over = 12.0 * unit * model::position(3.0, 2.0) - 19.0 * unit;
        assert!(over > 0.0 && over < 0.03, "{over}");
    }
}
