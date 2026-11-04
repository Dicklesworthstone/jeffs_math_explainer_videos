//! Historical alternative calculations and the close. Unbuilt draft.
use explainer_kit::{Kit, Palette, ink_style, rect, v};
use fmn::prelude::*;
use super::visual::fade;

pub struct Alabama { pub kit: Kit }

impl SceneConstruct for Alabama {
    fn name(&self) -> &str { "08_alabama" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "08_1")?;
        let title = kit.text(stage, "After the 1880 census", 37.0, Palette::ink(), v(0.0, 2.8))?;
        let left = stage.arena_mut().add(rect(v(-5.5, -0.2), 4.7, 0.06, ink_style(Palette::blue()))?);
        let right = stage.arena_mut().add(rect(v(0.8, -0.2), 4.7, 0.06, ink_style(Palette::gold()))?);
        let a = kit.text(stage, "299-seat alternative", 29.0, Palette::blue(), v(-3.15, 1.15))?;
        let b = kit.text(stage, "300-seat alternative", 29.0, Palette::gold(), v(3.15, 1.15))?;
        let c = kit.text(stage, "Alabama: 8", 31.0, Palette::ink(), v(-3.15, 0.3))?;
        let d = kit.text(stage, "Alabama: 7", 31.0, Palette::ink(), v(3.15, 0.3))?;
        let note = kit.text(stage, "Hypothetical alternatives, not an enacted seat change", 25.0, Palette::dim(), v(0.0, -1.0))?;
        fade(stage, &[title, left, right, a, b, c, d, note], true)?;

        kit.say(stage, "08_2")?;
        let closing = kit.text(stage, "The rounding rule changed the allocation", 30.0, Palette::hero(), v(0.0, -2.25))?;
        fade(stage, &[closing], true)?;
        kit.finish(stage)
    }
}
