//! Phase-color thumbnail for z²; provisional, awaiting editorial/device review.
use explainer_kit::{Kit, Palette, v};
use super::draw::{Painter, Plane};
use crate::model::Func;
use fmn::prelude::*;

pub fn make(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail { kit })
}
struct Thumbnail {
    kit: Kit,
}

#[cfg(test)]
fn phase_index(x: f64, y: f64, colors: usize) -> Option<usize> {
    if x == 0.0 && y == 0.0 {
        return None;
    }
    let angle = (2.0 * x * y).atan2(x * x - y * y).rem_euclid(std::f64::consts::TAU);
    Some(((angle / std::f64::consts::TAU * colors as f64).floor() as usize).min(colors - 1))
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

        // Use the same continuously evaluated, filtered native raster as
        // the film, rather than a separate coarse thumbnail-only tile grid.
        let plane = Plane::new(v(3.675, 0.325), 4.65, (-2.0, 2.0), (-2.0, 2.0));
        let painter = Painter::build(stage, Some(Func::Square), plane, 0.0)?;
        painter.show(stage, 0.01)?;
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

#[cfg(test)]
mod tests {
    use super::phase_index;
    #[test]
    fn square_phase_has_two_turns_and_zero_is_uncolored() {
        assert_eq!(phase_index(0.0, 0.0, 12), None);
        assert_eq!(phase_index(1.0, 0.0, 12), Some(0));
        assert_eq!(phase_index(-1.0, 0.0, 12), Some(0));
        assert_eq!(phase_index(0.0, 1.0, 12), Some(6));
        assert_eq!(phase_index(1.0, 1.0, 12), Some(3));
    }
}
