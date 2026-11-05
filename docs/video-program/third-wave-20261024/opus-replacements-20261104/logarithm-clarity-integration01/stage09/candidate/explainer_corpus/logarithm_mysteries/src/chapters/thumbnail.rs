//! Thumbnail candidate: the film's central picture, the column of
//! logarithms of one number standing over the point it maps to. A spiral
//! climbs one lap per 2π of height above a dial seen at a slant; four gold
//! points, the logarithms (2k + 1)πi of −1 for k = −2 … 1, share one
//! shadow on the floor. Editorial and device review remain open; unrendered.
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{Spiral, dashed, dot, line_style, path};

pub fn make(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail { kit })
}

struct Thumbnail {
    kit: Kit,
}

/// One lap of the spiral per 2π of the logarithm's height.
const HELIX: Spiral = Spiral {
    centre: [3.5, -2.35, 0.0],
    r: 1.8,
    tilt: 0.3,
    rise: 1.15,
    floor_lap: -1.85,
};
const LAPS: (f64, f64) = (-1.8, 1.9);

impl SceneConstruct for Thumbnail {
    fn name(&self) -> &str {
        "thumbnail"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let top = kit.text_styled(stage, "WHERE DO", 60.0, Palette::ink(), v(-3.6, 2.4), true, false)?;
        let top2 = kit.text_styled(stage, "THE LAPS GO?", 60.0, Palette::ink(), v(-3.6, 1.4), true, false)?;
        let formula = kit.tex(stage, r"\log z", 150.0, Palette::gold(), &[], v(-3.6, -0.55))?;
        let sub = kit.text_styled(
            stage,
            "RULERS, TURNS, FLOWS",
            38.0,
            Palette::blue(),
            v(-3.6, -2.55),
            true,
            false,
        )?;
        for mob in [top, top2, formula, sub] {
            stage.add_to_scene(mob)?;
        }
        let floor: Vec<Vec3> = (0..=64).map(|k| HELIX.shadow(f64::from(k) / 64.0)).collect();
        let dial = path(stage, floor, Palette::dim(), 3.0);
        let helix = stage.arena_mut().add(
            polyline(HELIX.points(LAPS.0, LAPS.1, 48))
                .style(line_style(Palette::purple(), 6.0))
                .build(),
        );
        let below = HELIX.shadow(0.5);
        let drop = dashed(stage, HELIX.at(1.5), below, Palette::dim(), 2.5, 14)?;
        stage.add_to_scene(dial)?;
        stage.add_to_scene(drop)?;
        stage.add_to_scene(helix)?;
        for k in -2..=1 {
            let laps = f64::from(2 * k + 1) / 2.0;
            let m = dot(stage, HELIX.at(laps), 0.13, Palette::gold());
            stage.add_to_scene(m)?;
        }
        let point = dot(stage, below, 0.15, Palette::hero());
        let label = kit.text(
            stage,
            "\u{2212}1",
            44.0,
            Palette::ink(),
            v(below[0] + 0.55, below[1] - 0.3),
        )?;
        stage.add_to_scene(point)?;
        stage.add_to_scene(label)?;
        stage.wait(1.0)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_helix_and_its_column_fit_the_frame_right_of_the_words() {
        for p in HELIX.points(LAPS.0, LAPS.1, 48) {
            assert!(p[0] > 1.0 && p[0] < 6.9 && p[1] > -3.8 && p[1] < 3.8, "{p:?}");
        }
        // the four logarithms of −1 share one shadow, one lap apart
        let s = HELIX.shadow(0.5);
        for k in -2..=1 {
            let laps = f64::from(2 * k + 1) / 2.0;
            let q = HELIX.shadow(laps);
            assert!((q[0] - s[0]).abs() < 1e-12 && (q[1] - s[1]).abs() < 1e-12);
            assert!(laps >= LAPS.0 && laps <= LAPS.1);
        }
    }
}
