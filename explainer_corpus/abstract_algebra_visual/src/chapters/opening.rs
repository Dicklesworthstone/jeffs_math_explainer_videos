//! Opening draft; text cue boundaries and the animation still need audio review.
use crate::model::{Symmetry, all};
use explainer_kit::{Kit, Palette, Timed, hex, ink_style, rect, stroke_style, v};
use fmn::prelude::*;

pub struct Opening { pub kit: Kit }

fn fade(stage: &mut Stage<'_>, mobs: &[Mob], incoming: bool) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = if incoming { fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)? }
            else { fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)? };
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

impl SceneConstruct for Opening {
    fn name(&self) -> &str { "01_square" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let colors = [Palette::blue(), Palette::gold(), Palette::green(), Palette::hero()];
        let corners = [(-1, -1), (1, -1), (1, 1), (-1, 1)];
        kit.say(stage, "01_1")?;
        let title = kit.text(stage, "How many ways can a square fit itself?", 34.0, Palette::ink(), v(0.0, 2.8))?;
        fade(stage, &[title], true)?;
        let mut demo = vec![stage.arena_mut().add(rect(v(-1.2, -1.2), 2.4, 2.4, stroke_style(Palette::dim(), 3.0))?)];
        for (i, &(x, y)) in corners.iter().enumerate() {
            let pos = v(f64::from(x) * 0.9, f64::from(y) * 0.9);
            demo.push(stage.arena_mut().add(Circle::new().radius(0.2).style(ink_style(colors[i])).build().moved_to(pos)));
            demo.push(kit.text(stage, &(i + 1).to_string(), 20.0, hex("#0E1116"), pos)?);
        }
        let explanation = kit.text(stage, "Turn, then flip · motion paths schematic", 23.0, Palette::hero(), v(0.0, -2.2))?;
        demo.push(explanation);
        fade(stage, &demo, true)?;
        for phase in 0..2 {
            // Estimated sentence starts are review guides, not heard sync.
            if phase == 1 { kit.until_sentence(stage, "01_1", 1)?; }
            for step in 1..=8 {
                let t = f64::from(step) / 8.0;
                let angle = std::f64::consts::FRAC_PI_2 * t;
                let mut animations: Vec<Box<dyn Animation>> = Vec::new();
                for (i, &(x, y)) in corners.iter().enumerate() {
                    let (x, y) = (f64::from(x) * 0.9, f64::from(y) * 0.9);
                    let (px, py) = if phase == 0 {
                        (x * angle.cos() - y * angle.sin(), x * angle.sin() + y * angle.cos())
                    } else {
                        let (rx, ry) = (-y, x);
                        (rx + 0.32 * (std::f64::consts::PI * t).sin() * ry, ry * (1.0 - 2.0 * t))
                    };
                    for &mob in &demo[1 + 2 * i..=2 + 2 * i] {
                        let animation = mob.animate().set_anim_args(AnimateArgs { run_time: Some(0.15), ..AnimateArgs::default() })?.move_to(v(px, py), ORIGIN)?;
                        animations.push(stage.prepare(animation)?);
                    }
                }
                stage.play_prepared(animations)?;
            }
        }
        kit.until_sentence(stage, "01_1", 3)?;
        fade(stage, &demo, false)?;
        let mut cards = Vec::new();
        for (index, action) in all().iter().enumerate() {
            let center_x = (index % 4) as f64 * 3.4 - 5.1;
            let center_y = if index < 4 { 1.0 } else { -1.2 };
            let mut card = vec![stage.arena_mut().add(rect(v(center_x - 0.65, center_y - 0.65), 1.3, 1.3, stroke_style(Palette::dim(), 2.5))?)];
            for (i, corner) in corners.iter().enumerate() {
                let (x, y) = action.apply(*corner);
                let pos = v(center_x + f64::from(x) * 0.47, center_y + f64::from(y) * 0.47);
                let dot = Circle::new().radius(0.12).style(ink_style(colors[i])).build().moved_to(pos);
                card.push(stage.arena_mut().add(dot));
                card.push(kit.text(stage, &(i + 1).to_string(), 14.0, hex("#0E1116"), pos)?);
            }
            cards.push(card);
        }
        // Reveal each model-generated move separately so the eight are countable.
        for card in &cards { fade(stage, card, true)?; }
        let count = kit.text(stage, &format!("{} rotations + {} reflections", all().iter().filter(|s| !s.reflected).count(), all().iter().filter(|s| s.reflected).count()), 27.0, Palette::hero(), v(0.0, -2.8))?;
        fade(stage, &[count], true)?;
        kit.say(stage, "01_2")?;
        let prompt = kit.text(stage, "How do these moves combine?", 31.0, Palette::green(), v(0.0, 2.1))?;
        fade(stage, &[prompt], true)?;
        // Check the first displayed card is the model identity.
        if all()[0] != Symmetry::ID { return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("unexpected identity order"))); }
        kit.finish(stage)
    }
}
