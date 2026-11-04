//! Provisional native thumbnail, awaiting visual and editorial review.
use explainer_kit::{Kit, Palette, ink_style, rect, stroke_style, v};
use fmn::prelude::*;

pub fn make(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail { kit })
}

struct Thumbnail {
    kit: Kit,
}

impl SceneConstruct for Thumbnail {
    fn name(&self) -> &str { "thumbnail" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        // The registry has an opening and a closing around the five examples.
        let examples = super::registry().len().saturating_sub(2);
        let count = kit.text_styled(stage, &examples.to_string(), 210.0, Palette::gold(), v(-5.05, 2.3), true, false)?;
        let math = kit.text_styled(stage, "MATH", 93.0, Palette::ink(), v(-2.3, 2.3), true, false)?;
        let title = kit.text_styled(stage, "PARADOXES", 78.0, Palette::ink(), v(-2.55, 0.4), true, false)?;
        let subtitle = kit.text_styled(stage, "MADE CLEAR", 54.0, Palette::blue(), v(-3.3, -1.2), true, false)?;
        for mob in [count, math, title, subtitle] { stage.add_to_scene(mob)?; }

        // Three closed doors suggest the Monty Hall puzzle without revealing
        // a numerical probability or implying an unstated host policy.
        let cases = five_math_paradoxes::monty_cases();
        let left = 1.15;
        for index in 0..cases.len() {
            let x = left + index as f64 * 1.65;
            let door = stage.arena_mut().add(rect(v(x, -0.55), 1.35, 2.65, stroke_style(Palette::blue(), 4.0))?);
            stage.add_to_scene(door)?;
            let mark = kit.text_styled(stage, "?", 80.0, Palette::gold(), v(x + 0.675, 0.75), true, false)?;
            stage.add_to_scene(mark)?;
            let knob = stage.arena_mut().add(rect(v(x + 1.03, -0.22), 0.10, 0.10, ink_style(Palette::gold()))?);
            stage.add_to_scene(knob)?;
        }
        let question = kit.text_styled(stage, "WHY?", 65.0, Palette::hero(), v(3.45, -1.85), true, false)?;
        stage.add_to_scene(question)?;
        stage.wait(1.0)?;
        Ok(())
    }
}
