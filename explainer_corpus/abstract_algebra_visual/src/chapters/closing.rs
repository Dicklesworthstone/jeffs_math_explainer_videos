//! Closing draft; timing, pedagogy and listening remain unreviewed.
use explainer_kit::{Kit, Palette, Timed, v};
use fmn::prelude::*;

pub struct Closing { pub kit: Kit }

impl SceneConstruct for Closing {
    fn name(&self) -> &str { "10_closing" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "10_1")?;
        let heading = kit.text(stage, "Which differences matter?", 36.0, Palette::ink(), v(0.0, 2.7))?;
        let heading_anim = fade_in(stage.arena_mut(), heading, ORIGIN, 1.0)?;
        stage.play(heading_anim.rt(0.5))?;
        let lines = [
            ("Group", "combine the moves"),
            ("Centralizer", "find moves that trade order"),
            ("Equivalence class", "choose what counts as the same"),
            ("Normal subgroup", "combine classes without ambiguity"),
        ];
        for (i, (label, meaning)) in lines.iter().enumerate() {
            // Provisional sentence timing; the full take still needs listening.
            kit.until_sentence(stage, "10_1", i + 1)?;
            let y = 1.4 - i as f64 * 1.0;
            let lhs = kit.text(stage, label, 26.0, Palette::blue(), v(-3.4, y))?;
            let rhs = kit.text(stage, meaning, 25.0, Palette::ink(), v(2.0, y))?;
            let mut animations: Vec<Box<dyn Animation>> = Vec::new();
            for mob in [lhs, rhs] {
                let animation = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
                animations.push(stage.prepare(animation.rt(0.5))?);
            }
            stage.play_prepared(animations)?;
        }
        kit.finish(stage)
    }
}
