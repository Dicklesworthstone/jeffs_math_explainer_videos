//! Chapter 1 — the hook: an unmistakable ring that every correlation
//! coefficient calls "no relationship", then the title card.

use fmn::animation::{indicate, show_passing_flash};
use fmn::prelude::*;

use crate::kit::{
    Frame2, Kit, Palette, Timed, dot_cloud, fade_all, point_colors, stroke_style, tween, v,
};
use crate::play;
use crate::stats::{self, Shape};

const N: usize = 150;

pub struct Hook {
    pub kit: Kit,
}

impl SceneConstruct for Hook {
    fn name(&self) -> &str {
        "01_hook"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let frame = Frame2 {
            center: v(-3.3, -0.2),
            width: 6.4,
            height: 6.4,
            x: (-1.2, 1.2),
            y: (-1.2, 1.2),
        };
        let pts = stats::shape_points(Shape::Ring, N, 41);
        let m = stats::measures(&pts);
        let scene_pts: Vec<Vec3> = pts.iter().map(|&(x, y)| frame.p(x, y)).collect();

        // A bare scatter: X across, Y up, a ring of measurements.
        let x_axis = Arrow::new(
            v(frame.left(), frame.bottom()),
            v(frame.right() + 0.2, frame.bottom()),
        )
        .buff(0.0)
        .thickness(4.0)
        .style(
            Style::default()
                .fill(Palette::dim(), 1.0)
                .stroke(Palette::dim(), 0.0, 1.0),
        )
        .build()?;
        let y_axis = Arrow::new(
            v(frame.left(), frame.bottom()),
            v(frame.left(), frame.top() + 0.2),
        )
        .buff(0.0)
        .thickness(4.0)
        .style(
            Style::default()
                .fill(Palette::dim(), 1.0)
                .stroke(Palette::dim(), 0.0, 1.0),
        )
        .build()?;
        let x_axis = stage.arena_mut().add(x_axis);
        let y_axis = stage.arena_mut().add(y_axis);
        let xl = kit.tex(
            stage,
            "X",
            38.0,
            Palette::dim(),
            &[],
            v(frame.right() + 0.25, frame.bottom() - 0.35),
        )?;
        let yl = kit.tex(
            stage,
            "Y",
            38.0,
            Palette::dim(),
            &[],
            v(frame.left() - 0.4, frame.top() + 0.15),
        )?;
        let cloud = stage
            .arena_mut()
            .add(dot_cloud(&scene_pts, &point_colors(N), 0.05));

        kit.say(stage, "h1")?;
        play!(stage;
            fmn::animation::grow_arrow(stage.arena_mut(), x_axis)?.rt(1.0),
            fmn::animation::grow_arrow(stage.arena_mut(), y_axis)?.rt(1.0),
            fade_in(stage.arena_mut(), xl, ORIGIN, 1.0)?,
            fade_in(stage.arena_mut(), yl, ORIGIN, 1.0)?,
        );
        play!(stage; fade_in(stage.arena_mut(), cloud, ORIGIN, 0.2)?.rt(2.4).lag(0.04));

        let question = kit.text(
            stage,
            "Are X and Y related?",
            46.0,
            Palette::ink(),
            v(3.6, 2.6),
        )?;
        kit.say(stage, "h2")?;
        play!(stage; write(stage.arena(), question).rt(1.2));
        stage.wait(0.6)?;

        kit.say(stage, "h3")?;
        // Ask the classics.
        let rows = [
            ("Pearson", Palette::pearson(), m.pearson),
            ("Spearman", Palette::spearman(), m.spearman),
            ("Kendall", Palette::kendall(), m.kendall),
        ];
        let mut trackers = Vec::new();
        for (i, &(name, color, _)) in rows.iter().enumerate() {
            let y = 1.25 - 0.85 * i as f64;
            let label = kit.text_left(stage, name, 36.0, color, v(1.4, y))?;
            play!(stage; fade_in(stage.arena_mut(), label, v(-0.3, 0.0), 1.0)?.rt(0.5));
            let t = stage.add_value_tracker(0.0);
            kit.live_number(stage, t, 3, true, 36.0, color, v(6.2, y), RIGHT)?;
            trackers.push(t);
        }
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for (&t, &(_, _, value)) in trackers.iter().zip(&rows) {
            anims.push(stage.prepare(tween(t, value, 1.6)?)?);
        }
        stage.play_prepared(anims)?;
        kit.say(stage, "h4")?;
        let verdict = kit.text(
            stage,
            "All ≈ 0:  “no relationship”",
            38.0,
            Palette::warn(),
            v(3.8, -1.45),
        )?;
        play!(stage; fade_in(stage.arena_mut(), verdict, v(0.0, 0.2), 1.0)?.rt(1.0));
        stage.wait(1.4)?;

        // ... but the eye disagrees.
        let ring = Circle::new()
            .radius(0.9 * frame.width / 2.4)
            .arc_center(frame.center)
            .style(stroke_style(Palette::q(), 6.0))
            .build();
        let ring = stage.arena_mut().add(ring);
        let eye = kit.text(
            stage,
            "Your eye: obviously yes.",
            38.0,
            Palette::good(),
            v(3.8, -2.35),
        )?;
        kit.say(stage, "h5")?;
        kit.chime(stage, 659.25)?;
        play!(stage;
            show_passing_flash(ring, 0.6).rt(2.0),
            indicate(stage.arena_mut(), cloud, 1.06, Some([1.0, 1.0, 0.0]))?.rt(2.0),
            fade_in(stage.arena_mut(), eye, v(0.0, 0.2), 1.0)?.rt(1.0),
        );
        stage.wait(1.8)?;
        let need = kit.text(
            stage,
            "We need a measure of dependence, not of slope.",
            28.0,
            Palette::ink(),
            v(3.75, -3.2),
        )?;
        kit.say(stage, "h6")?;
        play!(stage; write(stage.arena(), need).rt(1.4));
        stage.wait(1.0)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;

        // Title card.
        let title = kit.text_styled(
            stage,
            "Hoeffding’s D",
            110.0,
            Palette::hoeffding(),
            v(0.0, 0.7),
            false,
            false,
        )?;
        let subtitle = kit.text(
            stage,
            "a measure of dependence that sees shape, not just slope",
            34.0,
            Palette::ink(),
            v(0.0, -0.7),
        )?;
        let credit = kit.text(
            stage,
            "after Wassily Hoeffding, “A Non-Parametric Test of Independence” (1948)",
            24.0,
            Palette::dim(),
            v(0.0, -1.5),
        )?;
        kit.say(stage, "h7")?;
        kit.chime(stage, 880.0)?;
        play!(stage; write(stage.arena(), title).rt(2.0));
        play!(stage; fade_in(stage.arena_mut(), subtitle, v(0.0, 0.25), 1.0)?.rt(1.2));
        play!(stage; fade_in(stage.arena_mut(), credit, ORIGIN, 1.0)?.rt(1.0));
        stage.wait(1.5)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.pad(stage, 0)?;
        Ok(())
    }
}
