//! The thumbnail (one still, 1280×720): the gallery's ring of 150 points,
//! which anyone can see is a pattern, beside the three correlations' verdicts
//! on it, each about zero; over them, the title. The readings are
//! `stats::measures` of the points drawn (the `Ring` row of `hoeffding stats`),
//! to two places so they read at 320 px wide.

use fmn::prelude::*;

use super::gallery;
use crate::kit::{Frame2, Kit, Palette, dot_cloud, point_colors, v};
use crate::stats::{self, Shape};

/// The thumbnail's scene, for the explainer's `thumbnail` field.
pub fn make(kit: explainer_kit::Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail {
        kit: Kit::wrap(kit),
    })
}

pub struct Thumbnail {
    pub kit: Kit,
}

/// A correlation to two places, signed, as TeX; one that rounds to zero is
/// `0.00` (never `-0.00`).
fn reading(value: f64) -> String {
    let cents = (value * 100.0).round();
    if cents == 0.0 {
        "0.00".to_owned()
    } else {
        format!("{:+.2}", cents / 100.0)
    }
}

impl SceneConstruct for Thumbnail {
    fn name(&self) -> &str {
        "thumbnail"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let frame = Frame2 {
            center: v(-3.75, -0.85),
            width: 5.9,
            height: 5.9,
            x: (-1.2, 1.2),
            y: (-1.2, 1.2),
        };
        let pts = stats::shape_points(Shape::Ring, gallery::N, gallery::seed(Shape::Ring));
        let m = stats::measures(&pts);
        let scene_pts: Vec<Vec3> = pts.iter().map(|&(x, y)| frame.p(x, y)).collect();

        // the scatter's axes, then the ring: dots big enough to survive 320 px
        for tip in [
            v(frame.right() + 0.2, frame.bottom()),
            v(frame.left(), frame.top() + 0.2),
        ] {
            let axis = Arrow::new(v(frame.left(), frame.bottom()), tip)
                .buff(0.0)
                .thickness(5.0)
                .style(
                    Style::default()
                        .fill(Palette::dim(), 1.0)
                        .stroke(Palette::dim(), 0.0, 1.0),
                )
                .build()?;
            stage.add(axis)?;
        }
        let cloud = stage
            .arena_mut()
            .add(dot_cloud(&scene_pts, &point_colors(gallery::N), 0.085));
        stage.add_to_scene(cloud)?;

        let title = kit.text_styled(
            stage,
            "Hoeffding’s D",
            150.0,
            Palette::hoeffding(),
            v(0.0, 3.0),
            true,
            false,
        )?;
        stage.add_to_scene(title)?;

        // the three correlations on this ring, each in its role's colour
        let rows = [
            ("Pearson", Palette::pearson(), m.pearson),
            ("Spearman", Palette::spearman(), m.spearman),
            ("Kendall", Palette::kendall(), m.kendall),
        ];
        for (i, &(name, color, value)) in rows.iter().enumerate() {
            let y = 0.75 - 1.45 * i as f64;
            let label = kit.text_left(stage, name, 84.0, color, v(0.25, y))?;
            stage.add_to_scene(label)?;
            let number = kit.tex(stage, &reading(value), 92.0, color, &[], v(6.7, y))?;
            stage.move_to(number, v(6.7, y), RIGHT);
            stage.add_to_scene(number)?;
        }

        // one second at the thumbnail's 1 fps: the one captured frame
        stage.wait(1.0)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reading_keeps_its_sign_and_a_zero_has_none() {
        assert_eq!(reading(-0.012), "-0.01");
        assert_eq!(reading(0.991), "+0.99");
        assert_eq!(reading(-0.004), "0.00");
        assert_eq!(reading(0.0), "0.00");
    }

    #[test]
    fn the_ring_reads_about_zero_on_all_three_correlations() {
        let pts = stats::shape_points(Shape::Ring, gallery::N, gallery::seed(Shape::Ring));
        let m = stats::measures(&pts);
        for value in [m.pearson, m.spearman, m.kendall] {
            assert!(value.abs() < 0.02, "{value}");
        }
    }
}
