//! Chapter 8 — the summary table, D's properties, and the credits.

use fmn::prelude::*;

use crate::kit::{Frame2, Kit, Palette, Timed, dot_cloud, fade_all, lagged, point_colors, v};
use crate::play;
use crate::stats::{self, Shape};

pub struct Outro {
    pub kit: Kit,
}

/// A tiny scatter glyph for the summary table.
fn icon(shapes: &[Shape], center: Vec3) -> VMobject {
    let f = Frame2 {
        center,
        width: 1.15,
        height: 0.8,
        x: (-1.2, 1.2),
        y: (-1.2, 1.2),
    };
    let mut pts = Vec::new();
    for &s in shapes {
        pts.extend(
            stats::shape_points(s, 36, 5)
                .into_iter()
                .map(|(x, y)| f.p(x, y)),
        );
    }
    dot_cloud(&pts, &point_colors(pts.len()), 0.022)
}

impl SceneConstruct for Outro {
    fn name(&self) -> &str {
        "08_outro"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let title = kit.text(
            stage,
            "Which measure sees what",
            42.0,
            Palette::ink(),
            v(0.0, 3.4),
        )?;
        kit.say(stage, "o1")?;
        play!(stage; write(stage.arena(), title).rt(1.1));

        let rows: [(&str, Srgb, &str, &[Shape]); 3] = [
            (
                "Pearson r",
                Palette::pearson(),
                "straight lines",
                &[Shape::Line],
            ),
            (
                "Spearman ρ, Kendall τ",
                Palette::spearman(),
                "any monotone curve",
                &[Shape::Monotone],
            ),
            (
                "Hoeffding's D",
                Palette::hoeffding(),
                "any dependence: rings, crosses, waves, …",
                &[Shape::Ring, Shape::Cross],
            ),
        ];
        for (i, &(name, color, sees, shapes)) in rows.iter().enumerate() {
            let y = 2.1 - 1.15 * i as f64;
            if i == 2 {
                kit.say(stage, "o2")?;
            }
            let glyph = stage.arena_mut().add(icon(shapes, v(-5.9, y)));
            let n = kit.text_left(stage, name, 34.0, color, v(-4.95, y))?;
            let s = kit.text_left(stage, sees, 30.0, Palette::ink(), v(-0.35, y))?;
            play!(stage;
                fade_in(stage.arena_mut(), glyph, ORIGIN, 0.5)?.rt(0.9).lag(0.05),
                fade_in(stage.arena_mut(), n, v(-0.25, 0.0), 1.0)?.rt(0.9),
                fade_in(stage.arena_mut(), s, v(0.25, 0.0), 1.0)?.rt(0.9),
            );
            stage.wait(0.5)?;
        }
        let rule = Line::new(v(-6.6, -1.05), v(6.6, -1.05))
            .color(Palette::faint())
            .build()?;
        let rule = stage.arena_mut().add(rule);
        play!(stage; show_creation(rule).rt(0.6));

        let props = [
            "symmetric:  D(X, Y) = D(Y, X)",
            "rank-based:  robust to outliers and to any monotone rescaling",
            "≈ 0 under independence, 1 for perfectly monotone data, never below −0.5",
            "the price: it reasons about groups of points, not pairs; worth it near the top of a ranking",
        ];
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for (i, p) in props.iter().enumerate() {
            let m = kit.text_left(
                stage,
                p,
                26.0,
                Palette::dim(),
                v(-6.0, -1.55 - 0.55 * i as f64),
            )?;
            let a = fade_in(stage.arena_mut(), m, v(0.0, 0.15), 1.0)?;
            anims.push(stage.prepare(a)?);
        }
        let g = lagged(stage, anims, 0.45)?;
        kit.say(stage, "o3")?;
        play!(stage; g.rt(3.2));
        stage.wait(1.5)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;

        // Credits.
        let made = kit.text(
            stage,
            "Made entirely with franken_manim",
            52.0,
            Palette::hoeffding(),
            v(0.0, 1.3),
        )?;
        let how = kit.text(
            stage,
            "pure Rust  ·  native TeX mathematics, no LaTeX  ·  no Python  ·  encoded by ffmpeg",
            28.0,
            Palette::ink(),
            v(0.0, 0.3),
        )?;
        let live = kit.text(
            stage,
            "every number on screen was computed by the scene itself",
            26.0,
            Palette::dim(),
            v(0.0, -0.4),
        )?;
        let refs = kit.text(
            stage,
            "W. Hoeffding (1948)  ·  J. Emanuel, “My Favorite Statistical Measure: Hoeffding's D”",
            22.0,
            Palette::dim(),
            v(0.0, -1.6),
        )?;
        kit.say(stage, "o4")?;
        kit.chime(stage, 659.25)?;
        play!(stage; write(stage.arena(), made).rt(1.6));
        play!(stage; fade_in(stage.arena_mut(), how, v(0.0, 0.2), 1.0)?.rt(1.0));
        play!(stage; fade_in(stage.arena_mut(), live, v(0.0, 0.2), 1.0)?.rt(1.0));
        play!(stage; fade_in(stage.arena_mut(), refs, ORIGIN, 1.0)?.rt(1.0));
        stage.wait(1.5)?;
        kit.hold(stage)?;
        stage.wait(1.0)?;
        fade_all(stage, 1.5)?;
        stage.wait(0.5)?;
        kit.pad(stage, 7)?;
        Ok(())
    }
}
