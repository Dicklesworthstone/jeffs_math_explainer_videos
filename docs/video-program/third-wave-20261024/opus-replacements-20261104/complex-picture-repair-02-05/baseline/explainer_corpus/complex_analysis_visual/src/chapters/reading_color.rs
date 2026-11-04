//! 04_reading_color: paint the input plane by the output's direction, add
//! brightness bands for size and phase lines every twelfth of a turn, then
//! read zeros, their orders and poles off the colours; the chapter ends on
//! the bump's two poles and the radius-1 circle that touches them
//! (storyboard 04_1–04_7). One picture, recoloured in place throughout.

use std::f64::consts::FRAC_PI_4;

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    Painter, Plane, fact, fade_in_all, fade_out_all, hero, line_style, marks, names_now, pause, pole, precisely_now,
    predict, reading_window, say, wait_until, wheel_legend,
};
use crate::model::{C, Func, ZERO};

pub const DIRECTIONS: [&str; 4] = ["right: red", "up: yellow green", "left: teal", "down: violet"];
pub const BAND: &str = "next band: size × ";
pub const PHASE: &str = "dark lines: every twelfth of a turn";
pub const TURNS: &str = "colour cycles around 0: ";
pub const ORDER_TWO: &str = "zero of order 2";
pub const SIMPLE_POLE: &str = "simple pole";
pub const NEAR_ZERO: &str =
    "near a zero a of order m, f(z) is about c times (z − a) to the power m, so the hue turns m times as z circles a";
pub const ASK: [&str; 2] = ["where are the bump's poles?", "how many turns at each?"];
pub const EACH: &str = "one backwards cycle at each";

/// The picture's window: [−2, 2]², 6.4 units wide, left of centre.
pub fn plane() -> Plane {
    Plane::new(v(-2.6, -0.25), 6.4, (-2.0, 2.0), (-2.0, 2.0))
}

/// The gold loop with a direction arrow, around 0 at radius `r`.
fn loop_with_arrow(stage: &mut Stage<'_>, p: &Plane, r: f64) -> fmn::Result<Vec<Mob>> {
    let ring = p.circle(stage, ZERO, r, hero(), 5.0)?;
    let at = C::polar(r, FRAC_PI_4);
    let ahead = C::polar(r, FRAC_PI_4 + 0.35);
    let arrow = Arrow::new(p.p(at), p.p(ahead)).thickness(5.0).color(hero()).build()?;
    Ok(vec![ring, stage.arena_mut().add(arrow)])
}

pub struct ReadingColor {
    pub kit: Kit,
}

impl SceneConstruct for ReadingColor {
    fn name(&self) -> &str {
        "04_reading_color"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let p = plane();
        let right = 4.55;

        // 04_1: the wheel legend, then the plane painted with f(z) = z
        let line = say(&kit, stage, "04_1")?;
        let legend = wheel_legend(&kit, stage, v(right, 2.2), 0.75)?;
        fade_in_all(stage, &legend, 0.6)?;
        line.wait_sentence(stage, 1)?;
        let painter = Painter::build(stage, Some(Func::Identity), p, 0.0)?;
        let border = p.border(stage, Palette::faint())?;
        painter.show(stage, 1.2)?;
        fade_in_all(stage, &[border], 0.3)?;
        line.wait_for(stage, "Pointing right is red")?;
        let mut names = Vec::new();
        for (i, d) in DIRECTIONS.iter().enumerate() {
            names.push(kit.text(stage, d, 22.0, Palette::ink(), v(right, 1.0 - 0.38 * i as f64))?);
        }
        for (i, &m) in names.iter().enumerate() {
            fade_in_all(stage, &[m], 0.3)?;
            if i + 1 < names.len() {
                pause(stage, 0.5)?;
            }
        }

        // 04_2: brightness bands for size, dark lines every twelfth of a turn
        kit.hold(stage)?;
        let line = say(&kit, stage, "04_2")?;
        line.wait_sentence(stage, 1)?;
        painter.enhance_to(stage, 1.0, 2.0)?;
        line.wait_sentence(stage, 2)?;
        let band = kit.text(
            stage,
            &format!("{BAND}{}", fact("color.band_ratio")?),
            24.0,
            hero(),
            v(right, -0.75),
        )?;
        fade_in_all(stage, &[band], 0.4)?;
        line.wait_sentence(stage, 3)?;
        let phase = kit.text(stage, PHASE, 20.0, Palette::dim(), v(right, -1.2))?;
        fade_in_all(stage, &[phase], 0.4)?;

        // 04_3: z²: every colour twice around zero, in the wheel's order
        kit.hold(stage)?;
        let line = say(&kit, stage, "04_3")?;
        fade_out_all(stage, &names, 0.3)?;
        painter.to(stage, Some(Func::Square), 1.2)?;
        let ring = loop_with_arrow(stage, &p, 0.9)?;
        fade_in_all(stage, &ring, 0.5)?;
        line.wait_sentence(stage, 1)?;
        let count = kit.text(
            stage,
            &format!("{TURNS}{}", fact("color.square.turns")?),
            26.0,
            hero(),
            v(right, 0.6),
        )?;
        fade_in_all(stage, &[count], 0.4)?;

        // 04_4: the order of a zero, read at a glance
        kit.hold(stage)?;
        let line = say(&kit, stage, "04_4")?;
        let zero = marks(stage, Func::Square, &p);
        let order = kit.text(stage, ORDER_TWO, 24.0, Palette::ink(), p.p(C::new(0.0, -0.35)))?;
        fade_in_all(stage, &zero, 0.3)?;
        line.wait_sentence(stage, 1)?;
        fade_in_all(stage, &[order], 0.3)?;
        // the voice is still asking the eye to count colours around the dot, so the card
        // comes only when it has finished, and all of it is read in the window below
        wait_until(stage, line.end())?;
        let exact = precisely_now(&kit, stage, NEAR_ZERO)?;

        // 04_5: 1/z runs the colours backwards around a pole
        reading_window(&kit, stage, &line)?;
        let line = say(&kit, stage, "04_5")?;
        fade_out_all(stage, &[order, exact, count], 0.3)?;
        fade_out_all(stage, &zero, 0.2)?;
        painter.to(stage, Some(Func::Recip), 1.2)?;
        let pole_mark = marks(stage, Func::Recip, &p);
        let simple = kit.text(stage, SIMPLE_POLE, 24.0, pole(), p.p(C::new(0.0, -0.35)))?;
        let back = kit.text(
            stage,
            &format!("{TURNS}{}", fact("color.recip.turns")?),
            26.0,
            pole(),
            v(right, 0.6),
        )?;
        line.wait_sentence(stage, 1)?;
        fade_in_all(stage, &pole_mark, 0.3)?;
        fade_in_all(stage, &[simple, back], 0.4)?;

        // 04_6: before the colours appear, where are the bump's poles?
        kit.hold(stage)?;
        let line = say(&kit, stage, "04_6")?;
        let mut clearing = vec![simple, back];
        clearing.extend(&ring);
        clearing.extend(&pole_mark);
        fade_out_all(stage, &clearing, 0.4)?;
        painter.to(stage, None, 0.8)?;
        let unit = p.circle(stage, ZERO, 1.0, Palette::faint(), 3.0)?;
        let axes = p.axes(stage)?;
        fade_in_all(stage, &[unit], 0.4)?;
        fade_in_all(stage, &axes, 0.3)?;
        let asked = predict(&kit, stage, &line, &ASK, v(right - 0.15, 0.75))?;

        // 04_7: two poles at ±i, one backwards cycle each; radius 1 touches them
        let line = say(&kit, stage, "04_7")?;
        fade_out_all(stage, &asked, 0.3)?;
        painter.to(stage, Some(Func::Bump), 1.4)?;
        let bump_marks = marks(stage, Func::Bump, &p);
        fade_in_all(stage, &bump_marks, 0.3)?;
        let each = kit.text(
            stage,
            &format!(
                "{EACH}: {} and {}",
                fact("color.bump.turns_at_i")?,
                fact("color.bump.turns_at_minus_i")?
            ),
            24.0,
            pole(),
            v(right, 0.6),
        )?;
        fade_in_all(stage, &[each], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let touching = p.circle(stage, ZERO, 1.0, hero(), 5.0)?;
        let segment = stage.arena_mut().add(
            polyline(vec![p.p(C::real(-1.0)), p.p(C::real(1.0))])
                .style(line_style(hero(), 9.0))
                .build(),
        );
        fade_out_all(stage, &[unit], 0.2)?;
        fade_in_all(stage, &[touching, segment], 0.6)?;
        line.wait_sentence(stage, 2)?;
        let search = names_now(&kit, stage, &["domain colouring", "enhanced phase portrait"])?;
        wait_until(stage, line.end())?;
        kit.music_breath(stage, 2.0)?;
        let mut all = vec![border, band, phase, each, touching, segment, search];
        all.extend(legend);
        all.extend(axes);
        all.extend(bump_marks);
        fade_out_all(stage, &all, 0.5)?;
        painter.leave(stage, 0.5)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{I, circle, winding_count};

    #[test]
    fn the_window_holds_every_point_the_chapter_marks() {
        let p = plane();
        for z in [
            ZERO,
            I,
            -I,
            C::real(1.0),
            C::real(-1.0),
            C::polar(0.9, FRAC_PI_4 + 0.35),
        ] {
            assert!(p.contains(z));
        }
        // the picture fits the frame's safe area
        assert!(p.frame.top() <= 3.85 && p.frame.bottom() >= -3.85 && p.frame.left() >= -6.96);
    }

    #[test]
    fn what_the_cards_say_is_what_the_loops_count() {
        let f = crate::facts::facts().unwrap();
        assert_eq!(f.get("color.square.turns"), Some("2"));
        assert_eq!(f.get("color.recip.turns"), Some("-1"));
        // the drawn gold loop (radius 0.9) counts the same as the facts' (0.5)
        let sq = winding_count(|z| Func::Square.eval(z), &circle(ZERO, 0.9, 2048));
        let inv = winding_count(|z| Func::Recip.eval(z), &circle(ZERO, 0.9, 2048));
        assert_eq!((sq, inv), (Some(2), Some(-1)));
    }
}
