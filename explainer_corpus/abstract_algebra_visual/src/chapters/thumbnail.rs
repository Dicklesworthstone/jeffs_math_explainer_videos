//! Provisional square-symmetry thumbnail, awaiting editorial/device review.
use explainer_kit::{Kit, Palette, hex, ink_style, rect, stroke_style, v};
use fmn::prelude::*;
use crate::model::Symmetry;

pub fn make(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail { kit })
}
struct Thumbnail { kit: Kit }
impl SceneConstruct for Thumbnail {
    fn name(&self) -> &str { "thumbnail" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let title = kit.text_styled(stage, "ABSTRACT", 78.0, Palette::ink(), v(-3.7, 2.0), true, false)?;
        let subject = kit.text_styled(stage, "ALGEBRA", 91.0, Palette::gold(), v(-3.7, 0.65), true, false)?;
        let subtitle = kit.text_styled(stage, "MADE VISUAL", 47.0, Palette::blue(), v(-3.7, -1.15), true, false)?;
        for mob in [title, subject, subtitle] { stage.add_to_scene(mob)?; }

        // Symmetry::compose is right-to-left: s∘r is turn then flip.
        let r = Symmetry::R;
        let s = Symmetry::S;
        let corners = [(-1, -1), (1, -1), (1, 1), (-1, 1)];
        let colors = [Palette::blue(), Palette::gold(), Palette::green(), Palette::hero()];
        for (center, action, label, color) in [
            (1.5, s.compose(r), "TURN → FLIP", Palette::blue()),
            (4.7, r.compose(s), "FLIP → TURN", Palette::gold()),
        ] {
            let caption = kit.text_styled(stage, label, 36.0, color, v(center, 2.2), true, false)?;
            stage.add_to_scene(caption)?;
            let frame = stage.arena_mut().add(rect(v(center - 1.1, -0.9), 2.2, 2.2, stroke_style(Palette::dim(), 3.0))?);
            stage.add_to_scene(frame)?;
            for (i, corner) in corners.iter().enumerate() {
                let (x, y) = action.apply(*corner);
                let pos = v(center + f64::from(x)*0.83, 0.2 + f64::from(y)*0.83);
                let dot = stage.arena_mut().add(Circle::new().radius(0.23).style(ink_style(colors[i])).build().moved_to(pos));
                stage.add_to_scene(dot)?;
                let number = kit.text(stage, &(i + 1).to_string(), 24.0, hex("#0E1116"), pos)?;
                stage.add_to_scene(number)?;
            }
        }
        let difference = kit.text_styled(stage, "≠", 74.0, Palette::hero(), v(3.1, 0.2), true, false)?;
        let conclusion = kit.text_styled(stage, "ORDER MATTERS", 44.0, Palette::ink(), v(3.1, -2.05), true, false)?;
        stage.add_to_scene(difference)?;
        stage.add_to_scene(conclusion)?;
        stage.wait(1.0)?;
        Ok(())
    }
}
