//! Chapter 2 — "Where correlation falls short": one point cloud morphs
//! through six shapes while four measures update live, each judged
//! against its own shuffled-noise ceiling (a permutation test).

use fmn::animation::replacement_transform;
use fmn::prelude::*;

use crate::kit::{
    Frame2, Kit, Palette, Timed, dot_cloud, fade_all, live_bar, point_colors, stroke_style, tween,
    v,
};
use crate::play;
use crate::stats::{self, Shape};

pub const N: usize = 150;
pub const TRIALS: usize = 300;
const SHAPES: [Shape; 6] = [
    Shape::Line,
    Shape::Parabola,
    Shape::Ring,
    Shape::Cross,
    Shape::Wave,
    Shape::Noise,
];
/// Bars saturate at this many noise ceilings.
const CAP: f64 = 5.0;
const BAR_X: f64 = 4.1;
const BAR_LEN: f64 = 2.75;
const ROW_Y: [f64; 4] = [1.9, 0.95, 0.0, -1.2];

/// One shape's dots in scene space, and its four bar targets.
type ShapeData = (Vec<Vec3>, [(f64, f64); 4]);

pub fn seed(shape: Shape) -> u64 {
    1000 + shape as u64
}

pub struct Gallery {
    pub kit: Kit,
}

struct Row {
    value: Mob,
    ratio: Mob,
}

fn targets(m: &stats::Measures, q: &stats::Measures) -> [(f64, f64); 4] {
    let r = |val: f64, ceil: f64| (val, (val.abs() / ceil).min(CAP));
    [
        r(m.pearson, q.pearson),
        r(m.spearman, q.spearman),
        r(m.kendall, q.kendall),
        r(m.hoeffding, q.hoeffding),
    ]
}

impl SceneConstruct for Gallery {
    fn name(&self) -> &str {
        "02_gallery"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let frame = Frame2 {
            center: v(-3.55, -0.35),
            width: 5.8,
            height: 5.8,
            x: (-1.2, 1.2),
            y: (-1.2, 1.2),
        };
        let colors = point_colors(N);
        let data: Vec<ShapeData> = SHAPES
            .iter()
            .map(|&s| {
                let pts = stats::shape_points(s, N, seed(s));
                let scene: Vec<Vec3> = pts.iter().map(|&(x, y)| frame.p(x, y)).collect();
                let t = targets(&stats::measures(&pts), &stats::null_q99(&pts, TRIALS, 99));
                (scene, t)
            })
            .collect();

        // ---- the stage: window, axes hints, first cloud
        let title = kit.text(
            stage,
            "One question, six shapes",
            44.0,
            Palette::ink(),
            v(0.0, 3.45),
        )?;
        let border = stage.arena_mut().add(frame.border(Palette::faint())?);
        let h = DashedLine::new(
            v(frame.left(), frame.center[1]),
            v(frame.right(), frame.center[1]),
        )
        .style(stroke_style(Palette::faint(), 1.5))
        .build()?;
        let vline = DashedLine::new(
            v(frame.center[0], frame.bottom()),
            v(frame.center[0], frame.top()),
        )
        .style(stroke_style(Palette::faint(), 1.5))
        .build()?;
        let h = stage.arena_mut().add(h);
        let vline = stage.arena_mut().add(vline);
        let xl = kit.tex(
            stage,
            "X",
            32.0,
            Palette::dim(),
            &[],
            v(frame.right() - 0.2, frame.bottom() - 0.3),
        )?;
        let yl = kit.tex(
            stage,
            "Y",
            32.0,
            Palette::dim(),
            &[],
            v(frame.left() - 0.3, frame.top() - 0.2),
        )?;
        let mut label = kit.text(
            stage,
            SHAPES[0].label(),
            34.0,
            Palette::ink(),
            v(frame.center[0], frame.top() + 0.4),
        )?;
        let cloud = stage.arena_mut().add(dot_cloud(&data[0].0, &colors, 0.045));

        kit.say(stage, "g1")?;
        play!(stage; write(stage.arena(), title).rt(1.2));
        play!(stage;
            show_creation(border).rt(1.0),
            fade_in(stage.arena_mut(), h, ORIGIN, 1.0)?,
            fade_in(stage.arena_mut(), vline, ORIGIN, 1.0)?,
            fade_in(stage.arena_mut(), xl, ORIGIN, 1.0)?,
            fade_in(stage.arena_mut(), yl, ORIGIN, 1.0)?,
        );
        play!(stage;
            fade_in(stage.arena_mut(), cloud, ORIGIN, 0.3)?.rt(2.0).lag(0.03),
            fade_in(stage.arena_mut(), label, v(0.0, -0.2), 1.0)?,
        );

        // ---- the panel: four measures, values and noise-relative bars
        let names = [
            (r"\text{Pearson } r", Palette::pearson()),
            (r"\text{Spearman } \rho", Palette::spearman()),
            (r"\text{Kendall } \tau", Palette::kendall()),
            (r"\text{Hoeffding's } D", Palette::hoeffding()),
        ];
        let head_v = kit.text(stage, "value", 26.0, Palette::dim(), v(3.3, 2.65))?;
        let head_b = kit.text(
            stage,
            "strength vs. shuffled noise",
            26.0,
            Palette::dim(),
            v(BAR_X + BAR_LEN / 2.0, 2.65),
        )?;
        let ceiling_x = BAR_X + BAR_LEN / CAP;
        let ceiling = DashedLine::new(v(ceiling_x, 2.35), v(ceiling_x, -1.65))
            .style(stroke_style(Palette::dim(), 2.0))
            .build()?;
        let ceiling = stage.arena_mut().add(ceiling);
        let one = kit.tex(
            stage,
            r"1\times",
            24.0,
            Palette::dim(),
            &[],
            v(ceiling_x, -1.95),
        )?;
        let five = kit.tex(
            stage,
            r"5\times",
            24.0,
            Palette::dim(),
            &[],
            v(BAR_X + BAR_LEN, -1.95),
        )?;
        let zero = Line::new(v(BAR_X, 2.35), v(BAR_X, -1.65))
            .style(stroke_style(Palette::faint(), 2.0))
            .build()?;
        let zero = stage.arena_mut().add(zero);
        let divider = Line::new(v(0.35, -0.6), v(6.95, -0.6))
            .style(stroke_style(Palette::faint(), 1.5))
            .build()?;
        let divider = stage.arena_mut().add(divider);

        let mut rows = Vec::new();
        let mut row_labels = Vec::new();
        for (i, &(src, color)) in names.iter().enumerate() {
            // fmd-math draws \rho as a varrho (fm-5wq.60 family); the text
            // face's upright rho is the faithful glyph today.
            let m = if i == 1 {
                kit.text_left(stage, "Spearman ρ", 31.0, color, v(0.4, ROW_Y[i]))?
            } else {
                kit.tex_left(stage, src, 31.0, color, &[], v(0.4, ROW_Y[i]))?
            };
            row_labels.push(m);
        }
        kit.say(stage, "g2")?;
        play!(stage;
            fade_in(stage.arena_mut(), head_v, ORIGIN, 1.0)?,
            fade_in(stage.arena_mut(), head_b, ORIGIN, 1.0)?,
            show_creation(zero),
            show_creation(ceiling),
            fade_in(stage.arena_mut(), one, ORIGIN, 1.0)?,
            fade_in(stage.arena_mut(), five, ORIGIN, 1.0)?,
            show_creation(divider),
        );
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &row_labels {
            let a = fade_in(stage.arena_mut(), m, v(-0.3, 0.0), 1.0)?;
            anims.push(stage.prepare(a)?);
        }
        let group = crate::kit::lagged(stage, anims, 0.25)?;
        play!(stage; group.rt(1.6));
        for (i, &(_, color)) in names.iter().enumerate() {
            let value = stage.add_value_tracker(0.0);
            let ratio = stage.add_value_tracker(0.0);
            kit.live_number(stage, value, 3, true, 31.0, color, v(3.85, ROW_Y[i]), RIGHT)?;
            live_bar(stage, ratio, v(BAR_X, ROW_Y[i]), BAR_LEN / CAP, 0.34, color)?;
            rows.push(Row { value, ratio });
        }
        let caption = kit.text(
            stage,
            "noise ceiling = 99th percentile of the value over 300 random shuffles of Y",
            20.0,
            Palette::dim(),
            v(3.7, -2.45),
        )?;
        play!(stage; fade_in(stage.arena_mut(), caption, ORIGIN, 1.0)?);
        kit.say(stage, "g3")?;
        settle(stage, &rows, &data[0].1, 2.0)?;
        stage.wait(1.5)?;

        // ---- the morphs
        for (k, &shape) in SHAPES.iter().enumerate().skip(1) {
            let line = match shape {
                Shape::Parabola => "g4",
                Shape::Ring => "g5",
                Shape::Cross => "g7",
                Shape::Wave => "g8",
                _ => "g9",
            };
            kit.say(stage, line)?;
            let new_label = kit.text(
                stage,
                shape.label(),
                34.0,
                Palette::ink(),
                v(frame.center[0], frame.top() + 0.4),
            )?;
            let target = stage.arena_mut().add(dot_cloud(&data[k].0, &colors, 0.045));
            let mut anims: Vec<Box<dyn Animation>> = vec![
                stage.prepare(Transform::new(cloud, target))?,
                stage.prepare(replacement_transform(label, new_label))?,
            ];
            for (row, &(val, ratio)) in rows.iter().zip(&data[k].1) {
                anims.push(stage.prepare(tween(row.value, val, 2.4)?)?);
                anims.push(stage.prepare(tween(row.ratio, ratio, 2.4)?)?);
            }
            stage.play_prepared_with(
                anims,
                PlayOverrides {
                    run_time: Some(2.4),
                    ..PlayOverrides::default()
                },
            )?;
            label = new_label;
            if shape == Shape::Ring {
                ring_verdict(stage, &kit)?;
            } else {
                stage.wait(1.8)?;
            }
        }

        // ---- the moral, on a cleared stage
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.say(stage, "g10")?;
        let moral_a = kit.text(
            stage,
            "Correlation asks:  do X and Y move together?",
            40.0,
            Palette::dim(),
            v(0.0, 0.55),
        )?;
        let moral_b = kit.text(
            stage,
            "Hoeffding's D asks:  could X and Y be independent?",
            40.0,
            Palette::hoeffding(),
            v(0.0, -0.45),
        )?;
        play!(stage; fade_in(stage.arena_mut(), moral_a, v(0.0, 0.25), 1.0)?.rt(1.2));
        play!(stage; fade_in(stage.arena_mut(), moral_b, v(0.0, 0.25), 1.0)?.rt(1.2));
        stage.wait(1.5)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.pad(stage, 1)?;
        Ok(())
    }
}

/// Tween every row's value and bar to `targets`.
fn settle(
    stage: &mut Stage<'_>,
    rows: &[Row],
    targets: &[(f64, f64); 4],
    run_time: f64,
) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for (row, &(val, ratio)) in rows.iter().zip(targets) {
        anims.push(stage.prepare(tween(row.value, val, run_time)?)?);
        anims.push(stage.prepare(tween(row.ratio, ratio, run_time)?)?);
    }
    stage.play_prepared_with(
        anims,
        PlayOverrides {
            run_time: Some(run_time),
            ..PlayOverrides::default()
        },
    )?;
    Ok(())
}

/// On the ring: the correlations sit under the ceiling, D clears it.
fn ring_verdict(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<()> {
    let lost = SurroundingRectangle::from_extent(Some((v(0.4, -0.35), v(6.95, 2.3))))
        .buff(0.05)
        .color(Palette::warn())
        .build();
    let found = SurroundingRectangle::from_extent(Some((v(0.4, -1.5), v(6.95, -0.9))))
        .buff(0.05)
        .color(Palette::good())
        .build();
    let lost = stage.arena_mut().add(lost);
    let found = stage.arena_mut().add(found);
    let lost_t = kit.text(
        stage,
        "inside the noise: no signal",
        28.0,
        Palette::warn(),
        v(3.7, -3.15),
    )?;
    let found_t = kit.text(
        stage,
        "far above the noise: dependence detected",
        28.0,
        Palette::good(),
        v(3.7, -3.15),
    )?;
    play!(stage; show_creation(lost).rt(0.9), fade_in(stage.arena_mut(), lost_t, ORIGIN, 1.0)?);
    stage.wait(1.4)?;
    kit.say(stage, "g6")?;
    kit.chime(stage, 659.25)?;
    play!(stage;
        replacement_transform(lost, found).rt(0.9),
        replacement_transform(lost_t, found_t).rt(0.9),
    );
    stage.wait(1.8)?;
    play!(stage;
        fade_out(stage.arena_mut(), found, ORIGIN, 1.0)?,
        fade_out(stage.arena_mut(), found_t, ORIGIN, 1.0)?,
    );
    Ok(())
}
