//! Provisional native thumbnail, awaiting visual and editorial review.
use explainer_kit::{Kit, Palette, ink_style, stroke_style, v};
use fmn::prelude::*;

pub fn make(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail { kit })
}

struct Thumbnail { kit: Kit }

impl SceneConstruct for Thumbnail {
    fn name(&self) -> &str { "thumbnail" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let title = kit.text_styled(stage, "THE MAGIC OF", 67.0, Palette::ink(), v(-3.5, 2.3), true, false)?;
        let exponential = kit.tex(stage, r"e^x", 220.0, Palette::gold(), &[], v(-3.6, 0.15))?;
        let subtitle = kit.text_styled(stage, "GROWTH & SLIDE RULES", 40.0, Palette::blue(), v(-3.5, -2.25), true, false)?;
        for mob in [title, exponential, subtitle] { stage.add_to_scene(mob)?; }

        // Plot actual e^x and its tangent at zero, the fact explained by the film.
        let origin = (2.8_f64, -2.05_f64);
        let map = |x: f64, y: f64| v(origin.0 + 1.4 * x, origin.1 + y);
        let x_axis = stage.arena_mut().add(Line::new(map(-1.9, 0.0), map(1.8, 0.0)).style(stroke_style(Palette::dim(), 2.5)).build()?);
        let y_axis = stage.arena_mut().add(Line::new(map(0.0, -0.15), map(0.0, 4.1)).style(stroke_style(Palette::dim(), 2.5)).build()?);
        stage.add_to_scene(x_axis)?;
        stage.add_to_scene(y_axis)?;
        let segments = 96_u32;
        for i in 0..segments {
            let a = -1.8 + 3.0 * f64::from(i) / f64::from(segments);
            let b = -1.8 + 3.0 * f64::from(i + 1) / f64::from(segments);
            let line = stage.arena_mut().add(Line::new(map(a, a.exp()), map(b, b.exp())).style(stroke_style(Palette::blue(), 6.0)).build()?);
            stage.add_to_scene(line)?;
        }
        let tangent = stage.arena_mut().add(Line::new(map(-0.8, 0.2), map(1.2, 2.2)).style(stroke_style(Palette::gold(), 3.0)).build()?);
        stage.add_to_scene(tangent)?;
        let point = stage.arena_mut().add(Circle::new().radius(0.11).style(ink_style(Palette::hero())).build().moved_to(map(0.0, 1.0)));
        stage.add_to_scene(point)?;
        stage.wait(1.0)?;
        Ok(())
    }
}
