//! Illustrative square-symmetry application; assumptions and audio need review.
use crate::model::Symmetry;
use explainer_kit::{Kit, Palette, Timed, hex, ink_style, rect, stroke_style, v};
use fmn::prelude::*;

pub struct Applications { pub kit: Kit }

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

impl SceneConstruct for Applications {
    fn name(&self) -> &str { "08_applications" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let corners = [(-1, -1), (1, -1), (1, 1), (-1, 1)];
        let colors = [Palette::blue(), Palette::gold(), Palette::green(), Palette::hero()];
        kit.say(stage, "08_1")?;
        let mut picture = vec![kit.text(stage, "Ignore rotation, keep reflection", 34.0, Palette::ink(), v(0.0, 2.8))?];
        fade(stage, &picture, true)?;
        let actions = [Symmetry::ID, Symmetry::R, Symmetry::S];
        let captions = ["original", "quarter turn", "mirror image"];
        for (index, action) in actions.iter().enumerate() {
            if index > 0 {
                kit.until_sentence(stage, "08_1", index)?;
            }
            let x = (index as f64 - 1.0) * 4.0;
            let mut panel = vec![stage.arena_mut().add(rect(v(x - 0.9, -0.9), 1.8, 1.8, stroke_style(Palette::dim(), 3.0))?)];
            for (i, corner) in corners.iter().enumerate() {
                let (cx, cy) = action.apply(*corner);
                let pos = v(x + f64::from(cx) * 0.65, f64::from(cy) * 0.65);
                panel.push(stage.arena_mut().add(Circle::new().radius(0.17).style(ink_style(colors[i])).build().moved_to(pos)));
                panel.push(kit.text(stage, &(i + 1).to_string(), 17.0, hex("#0E1116"), pos)?);
            }
            panel.push(kit.text(stage, captions[index], 24.0, Palette::ink(), v(x, -1.4))?);
            fade(stage, &panel, true)?;
            picture.extend(panel);
        }
        kit.until_sentence(stage, "08_1", 3)?;
        let grouping = [
            kit.text(stage, "same rotation class", 23.0, Palette::green(), v(-2.0, -2.2))?,
            kit.text(stage, "different class", 23.0, Palette::gold(), v(4.0, -2.2))?,
        ];
        fade(stage, &grouping, true)?;
        kit.until_sentence(stage, "08_1", 5)?;
        picture.extend(grouping);
        fade(stage, &picture, false)?;
        let mut symmetric = vec![kit.text(stage, "A symmetric mark can look unchanged", 34.0, Palette::ink(), v(0.0, 2.7))?];
        for (index, x) in [-2.3, 2.3].iter().enumerate() {
            symmetric.push(stage.arena_mut().add(rect(v(*x - 0.9, -0.9), 1.8, 1.8, stroke_style(Palette::dim(), 3.0))?));
            for dx in [-0.5, 0.5] {
                symmetric.push(stage.arena_mut().add(Circle::new().radius(0.18).style(ink_style(Palette::blue())).build().moved_to(v(*x + dx, 0.0))));
            }
            symmetric.push(kit.text(stage, if index == 0 { "original" } else { "mirror image" }, 26.0, Palette::ink(), v(*x, -1.4))?);
        }
        symmetric.push(kit.text(stage, "Same image; different moves", 28.0, Palette::gold(), v(0.0, -2.2))?);
        fade(stage, &symmetric, true)?;
        kit.say(stage, "08_2")?;
        fade(stage, &symmetric, false)?;
        let contexts = [
            kit.text(stage, "The same language, different groups", 34.0, Palette::ink(), v(0.0, 2.4))?,
            kit.text(stage, "Molecular symmetries", 28.0, Palette::blue(), v(0.0, 0.9))?,
            kit.text(stage, "Error-detecting codes", 28.0, Palette::green(), v(0.0, 0.0))?,
            kit.text(stage, "Engineering transformations", 28.0, Palette::hero(), v(0.0, -0.9))?,
            kit.text(stage, "Each application has its own assumptions", 23.0, Palette::dim(), v(0.0, -2.2))?,
        ];
        fade(stage, &contexts, true)?;
        kit.finish(stage)
    }
}
