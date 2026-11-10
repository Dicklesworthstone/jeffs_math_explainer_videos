//! Phase-color thumbnail for z²; provisional, awaiting editorial/device review.
use explainer_kit::{Kit, Palette, hex, ink_style, rect, v};
use fmn::prelude::*;

pub fn make(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail { kit })
}
struct Thumbnail {
    kit: Kit,
}

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

        // Same phase-color convention as the film: hue is arg(z²), with
        // constant brightness and the zero marked separately.
        let colors = [
            "#ED5B62", "#F08A51", "#E7B44E", "#B7C75B", "#71C979", "#4DC3A5", "#4ABACD", "#579BE2", "#7A83E7",
            "#A373D8", "#C66DB3", "#E3658C",
        ];
        let n = 51_i32;
        let size = 4.65_f64 / f64::from(n);
        let half = f64::from(n - 1) / 2.0;
        for row in 0..n {
            for col in 0..n {
                let x = (f64::from(col) - half) / half;
                let y = (f64::from(row) - half) / half;
                let color = phase_index(x, y, colors.len())
                    .map(|index| hex(colors[index]))
                    .unwrap_or(Palette::faint());
                let tile = stage.arena_mut().add(rect(
                    v(1.35 + f64::from(col) * size, -2.0 + f64::from(row) * size),
                    size,
                    size,
                    ink_style(color),
                )?);
                stage.set_anti_alias_width(tile, 0.5, false);
                stage.add_to_scene(tile)?;
            }
        }
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
