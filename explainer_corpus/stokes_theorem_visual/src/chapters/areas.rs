//! A two-form sampled on pairs of steps: signed area, order, flattening, and the repeated-differential zero.
//! CC3 scratch draft: uncompiled, unrendered, sentence timing provisional until Frederick takes exist.
use explainer_kit::{Kit, Palette, Timed, v};
use fmn::prelude::*;
use super::opening::{arrow, fade, line};
use super::steps::shown;
use crate::facts::dx_dy;

pub struct SignedAreas { pub kit: Kit }

const O: (f64, f64) = (-6.0, -1.9);
const K: f64 = 1.1;
const U: (f64, f64) = (2.0, 0.0);
const V: (f64, f64) = (1.0, 1.5);
const COLUMN: f64 = 3.3;

fn sp(p: (f64, f64)) -> (f64, f64) { (O.0 + K * p.0, O.1 + K * p.1) }
/// The tip of the second step slides along a straight line from (1, 1.5) to (3, 0).
fn slid(t: f64) -> (f64, f64) { (V.0 + 2.0 * t, V.1 - 1.5 * t) }

/// The second step and the two far sides of the parallelogram spanned by U and `w`.
fn second(stage: &mut Stage<'_>, w: (f64, f64)) -> fmn::Result<Vec<Mob>> {
    let mut mobs = arrow(stage, sp((0.0, 0.0)), sp(w), Palette::green())?;
    mobs.push(line(stage, sp(U), sp((U.0 + w.0, U.1 + w.1)), Palette::faint(), 2.0)?);
    mobs.push(line(stage, sp(w), sp((U.0 + w.0, U.1 + w.1)), Palette::faint(), 2.0)?);
    Ok(mobs)
}

fn morph(stage: &mut Stage<'_>, from: &[Mob], to: &[Mob]) -> fmn::Result<()> {
    assert_eq!(from.len(), to.len());
    let mut moves: Vec<Box<dyn Animation>> = Vec::new();
    for (&a, &b) in from.iter().zip(to) { moves.push(stage.prepare(Transform::new(a, b).rt(1.2))?); }
    stage.play_prepared(moves)?;
    Ok(())
}

impl SceneConstruct for SignedAreas {
    fn name(&self) -> &str { "10_signed_areas" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let title = kit.text(stage, "What a two-form does to two steps", 34.0, Palette::ink(), v(0.0, 3.0))?;
        let first_step = arrow(stage, sp((0.0, 0.0)), sp(U), Palette::blue())?;
        let parallelogram = second(stage, V)?;
        let u_label = kit.text(stage, "u = (2, 0)", 22.0, Palette::blue(), v(sp((1.0, 0.0)).0, O.1 - 0.4))?;
        let v_label = kit.text(stage, "v = (1, 1.5)", 22.0, Palette::green(), v(sp(V).0 - 0.5, sp(V).1 + 0.45))?;
        // The value sits inside the parallelogram; once it is flat, just above the line it lies on.
        let centre = |w: (f64, f64)| {
            let c = sp((0.5 * (U.0 + w.0), 0.5 * (U.1 + w.1)));
            v(c.0, if w.1 == 0.0 { c.1 + 0.35 } else { c.1 })
        };
        let mut value = kit.text(stage, &format!("+{}", shown(dx_dy(U, V))), 30.0, Palette::hero(), centre(V))?;
        let formula = kit.tex(stage, r"dx\wedge dy\,(u,v)=u_x v_y-u_y v_x", 34.0, Palette::ink(), &[], v(COLUMN, 2.1))?;
        let worked = kit.text(stage, &format!("2 × 1.5 − 0 × 1 = {}", shown(dx_dy(U, V))), 26.0, Palette::hero(), v(COLUMN, 1.35))?;
        let mut first = vec![title, u_label];
        first.extend(first_step.iter().copied());
        fade(stage, &first, true)?;
        kit.say(stage, "10_1")?;
        kit.until_sentence(stage, "10_1", 1)?;
        fade(stage, &[formula], true)?;
        kit.until_sentence(stage, "10_1", 2)?;
        let mut spanned = parallelogram.clone();
        spanned.push(v_label);
        fade(stage, &spanned, true)?;
        kit.until_sentence(stage, "10_1", 3)?;
        fade(stage, &[worked, value], true)?;
        kit.hold(stage)?;
        kit.say(stage, "10_2")?;
        let swapped = kit.text(stage, &format!("order v, u: {}", shown(dx_dy(V, U))), 26.0, Palette::green(), v(COLUMN, 0.65))?;
        fade(stage, &[swapped], true)?;
        kit.hold(stage)?;
        kit.say(stage, "10_3")?;
        // v changes from here on, so its label goes; the value leaves while the shape moves.
        fade(stage, &[v_label, value], false)?;
        let halfway = second(stage, slid(0.5))?;
        morph(stage, &parallelogram, &halfway)?;
        let mut slide_rows = Vec::new();
        for (j, t, color, name, y) in [(1, 0.5, Palette::ink(), "halfway", -0.05), (2, 1.0, Palette::dim(), "along u", -0.75)] {
            kit.until_sentence(stage, "10_3", j)?;
            if j == 2 {
                fade(stage, &[value], false)?;
                let flat = second(stage, slid(t))?;
                morph(stage, &parallelogram, &flat)?;
            }
            value = kit.text(stage, &shown(dx_dy(U, slid(t))), 30.0, color, centre(slid(t)))?;
            let row = kit.text(stage, &format!("{name}: {}", shown(dx_dy(U, slid(t)))), 26.0, color, v(COLUMN, y))?;
            fade(stage, &[value, row], true)?;
            slide_rows.push(row);
        }
        kit.hold(stage)?;
        kit.say(stage, "10_4")?;
        // The flat parallelogram stays on the left; the right column turns to three dimensions.
        slide_rows.extend([worked, swapped]);
        fade(stage, &slide_rows, false)?;
        let heading = kit.text(stage, "in three dimensions", 24.0, Palette::dim(), v(COLUMN, 1.3))?;
        let shadow = kit.tex(stage, r"dx\wedge dz\,(u,v)=u_x v_z-u_z v_x", 34.0, Palette::blue(), &[], v(COLUMN, 0.45))?;
        let plane = kit.text(stage, "the shadow on the x z plane", 22.0, Palette::dim(), v(COLUMN, -0.2))?;
        fade(stage, &[heading, shadow, plane], true)?;
        kit.until_sentence(stage, "10_4", 1)?;
        let repeated = kit.tex(stage, r"dx\wedge dx\,(u,v)=u_x v_x-u_x v_x=0", 34.0, Palette::red(), &[], v(COLUMN, -1.15))?;
        fade(stage, &[repeated], true)?;
        kit.until_sentence(stage, "10_4", 2)?;
        let line_note = kit.text(stage, "both factors read only x: a shadow on a line", 22.0, Palette::dim(), v(COLUMN, -1.85))?;
        fade(stage, &[line_note], true)?;
        kit.until_sentence(stage, "10_4", 3)?;
        let rule = kit.text(stage, "a repeated differential gives zero", 26.0, Palette::red(), v(COLUMN, -2.6))?;
        fade(stage, &[rule], true)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_spoken_areas_follow_the_sliding_tip() {
        assert_eq!((dx_dy(U, V), dx_dy(V, U)), (3.0, -3.0));
        assert_eq!(dx_dy(U, slid(0.5)), 1.5);
        assert_eq!(dx_dy(U, slid(1.0)), 0.0);
        assert_eq!(slid(1.0).1, 0.0, "at the end the second step lies along the first");
    }

    #[test]
    fn the_picture_stays_left_of_the_text_column() {
        for t in [0.0, 0.5, 1.0] {
            let far = sp((U.0 + slid(t).0, U.1 + slid(t).1));
            assert!(far.0 < -0.3 && far.1 < 2.4, "{t}: {far:?}");
        }
    }
}
