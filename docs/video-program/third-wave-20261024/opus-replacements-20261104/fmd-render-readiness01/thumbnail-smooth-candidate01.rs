//! Phase-color thumbnail for z²; provisional, awaiting editorial/device review.
use explainer_kit::{Kit, Palette, v};
use super::draw::{portrait, Plane};
use crate::model::Func;
use fmn::prelude::*;

pub fn make(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail { kit })
}
struct Thumbnail {
    kit: Kit,
}

impl SceneConstruct for Thumbnail {
    fn name(&self) -> &str {
        "thumbnail"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let headline = kit.text_styled(
            stage,
            "THE BEAUTY OF",
            60.0,
            Palette::ink(),
            v(-3.75, 2.65),
            true,
            false,
        )?;
        let complex = kit.text_styled(stage, "COMPLEX", 83.0, Palette::ink(), v(-3.75, 1.25), true, false)?;
        let analysis = kit.text_styled(stage, "ANALYSIS", 83.0, Palette::gold(), v(-3.75, -0.05), true, false)?;
        let sub = kit.text_styled(
            stage,
            "A PLANE IN COLOR",
            40.0,
            Palette::blue(),
            v(-3.75, -1.75),
            true,
            false,
        )?;
        for mob in [headline, complex, analysis, sub] {
            stage.add_to_scene(mob)?;
        }

        // Use the film's continuous phase colour field and fine blended native
        // strips, rather than quantizing phase into twelve colours on 51 tiles.
        // Same footprint as the prior thumbnail, now centred exactly.
        let plane = Plane::new(v(3.675, 0.325), 4.65, (-1.0, 1.0), (-1.0, 1.0));
        let field = portrait(stage, Func::Square, &plane, 0.0)?;
        stage.add_to_scene(field.group)?;
        let label = kit.text_styled(
            stage,
            "z²: HUE = OUTPUT ANGLE",
            29.0,
            Palette::ink(),
            v(3.675, -2.7),
            true,
            false,
        )?;
        stage.add_to_scene(label)?;
        stage.wait(1.0)?;
        Ok(())
    }
}

