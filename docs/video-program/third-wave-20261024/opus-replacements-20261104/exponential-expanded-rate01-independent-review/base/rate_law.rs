//! Expanded native scene candidate; not compiled or visually accepted.
//! Integration requires `mod rate_model`, registered glyphs, the five exact
//! script-slice cues, and freshly source-bound direction/narration plans.
use crate::rate_model::{STEP_COUNTS, constant_steps, proportional_steps};
use explainer_kit::{Kit, Palette, Timed, stroke_style, v};
use fmn::prelude::*;

pub struct RateLaw {
    pub kit: Kit,
}
const LEFT: f64 = -5.4;
const RIGHT: f64 = 0.5;
const WIDTH: f64 = 4.5;
const BOTTOM: f64 = -1.8;
const HEIGHT: f64 = 3.1;
const TOP_VALUE: f64 = 3.0;
fn point(left: f64, t: f64, y: f64) -> Vec3 {
    v(left + WIDTH * t, BOTTOM + HEIGHT * y / TOP_VALUE)
}
fn line(stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb, width: f64) -> fmn::Result<Mob> {
    Ok(stage
        .arena_mut()
        .add(Line::new(a, b).style(stroke_style(color, width)).build()?))
}
fn show(stage: &mut Stage<'_>, mobs: &[Mob], seconds: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let a = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(seconds))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}
fn morph(stage: &mut Stage<'_>, pairs: &[(Mob, Mob)], seconds: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &(a, b) in pairs {
        anims.push(stage.prepare(Transform::new(a, b).rt(seconds))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}
fn slope(stage: &mut Stage<'_>, left: f64, t: f64, y: f64, rate: f64, color: Srgb) -> fmn::Result<Mob> {
    let run = 0.12;
    line(
        stage,
        point(left, t, y),
        point(left, t + run, y + run * rate),
        color,
        5.0,
    )
}
impl SceneConstruct for RateLaw {
    fn name(&self) -> &str {
        "01_rate_law"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "01_1")?;
        let mut frame = vec![
            kit.text(
                stage,
                "Same start, two different rules",
                35.0,
                Palette::ink(),
                v(0.0, 3.1),
            )?,
            kit.text(stage, "constant rate = 1", 27.0, Palette::gold(), v(-3.15, 2.2))?,
            kit.text(stage, "rate = current height", 27.0, Palette::blue(), v(2.75, 2.2))?,
            kit.text(
                stage,
                "straight steps: an approximation",
                25.0,
                Palette::dim(),
                v(0.0, -3.0),
            )?,
        ];
        for left in [LEFT, RIGHT] {
            frame.push(line(
                stage,
                point(left, 0.0, 0.0),
                point(left, 1.0, 0.0),
                Palette::dim(),
                2.0,
            )?);
            frame.push(line(
                stage,
                point(left, 0.0, 0.0),
                point(left, 0.0, TOP_VALUE),
                Palette::dim(),
                2.0,
            )?);
            for y in 0..=3 {
                frame.push(kit.text(
                    stage,
                    &y.to_string(),
                    19.0,
                    Palette::dim(),
                    v(left - 0.25, point(left, 0.0, f64::from(y)).y),
                )?);
            }
            frame.push(kit.text(
                stage,
                "time: 0 to 1",
                21.0,
                Palette::dim(),
                v(left + WIDTH / 2.0, -2.25),
            )?);
        }
        show(stage, &frame, 0.6)?;
        let p = proportional_steps(STEP_COUNTS[0]);
        let c = constant_steps(STEP_COUNTS[0]);
        let left_slope = slope(stage, LEFT, 0.0, 1.0, 1.0, Palette::gold())?;
        let right_slope = slope(stage, RIGHT, 0.0, 1.0, 1.0, Palette::blue())?;
        let left_read = kit.text(stage, "slope 1.00", 23.0, Palette::gold(), v(-3.15, 1.7))?;
        let right_read = kit.text(stage, "slope 1.00", 23.0, Palette::blue(), v(2.75, 1.7))?;
        let question = kit.text(
            stage,
            "Will the two curves stay together?",
            27.0,
            Palette::hero(),
            v(0.0, -2.65),
        )?;
        show(stage, &[left_slope, right_slope, left_read, right_read, question], 0.4)?;
        kit.hold(stage)?;
        kit.mark(stage, "predict");
        stage.wait(4.0)?;
        let fade = fade_out(stage.arena_mut(), question, ORIGIN, 1.0)?;
        stage.play(fade.rt(0.3))?;
        let mut old_right = Vec::new();
        for k in 1..p.len() {
            if k == 1 {
                kit.say(stage, "01_2")?;
            }
            if k == 2 {
                kit.hold(stage)?;
                kit.say(stage, "01_3")?;
            }
            let left_seg = line(
                stage,
                point(LEFT, c[k - 1].0, c[k - 1].1),
                point(LEFT, c[k].0, c[k].1),
                Palette::gold(),
                3.5,
            )?;
            let right_seg = line(
                stage,
                point(RIGHT, p[k - 1].0, p[k - 1].1),
                point(RIGHT, p[k].0, p[k].1),
                Palette::blue(),
                3.5,
            )?;
            show(stage, &[left_seg, right_seg], 0.6)?;
            old_right.push(right_seg);
            if k == 1 {
                kit.until_sentence(stage, "01_2", 2)?;
            }
            let sl = slope(stage, LEFT, c[k].0, c[k].1, 1.0, Palette::gold())?;
            let sr = slope(stage, RIGHT, p[k].0, p[k].1, p[k].1, Palette::blue())?;
            let read = kit.text(
                stage,
                &format!("slope {:.2}", p[k].1),
                23.0,
                Palette::blue(),
                v(2.75, 1.7),
            )?;
            morph(stage, &[(left_slope, sl), (right_slope, sr), (right_read, read)], 0.6)?;
            stage.wait(0.6)?;
        }
        kit.hold(stage)?;
        kit.say(stage, "01_4")?;
        // Remove the coarse endpoint slope before refining: retaining it would
        // falsely label the new approximation with the old endpoint slope.
        let fade = fade_out(stage.arena_mut(), right_slope, ORIGIN, 1.0)?;
        stage.play(fade.rt(0.3))?;
        for (pass, n) in [STEP_COUNTS[1], STEP_COUNTS[2]].into_iter().enumerate() {
            kit.until_sentence(stage, "01_4", if pass == 0 { 2 } else { 4 })?;
            let mut fades: Vec<Box<dyn Animation>> = Vec::new();
            for &mob in &old_right {
                let a = mob
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.4),
                        ..AnimateArgs::default()
                    })?
                    .set_color_with_opacity(Palette::dim(), 0.5)?;
                fades.push(stage.prepare(a)?);
            }
            stage.play_prepared(fades)?;
            if pass == 0 {
                kit.until_sentence(stage, "01_4", 3)?;
            }
            let refined = proportional_steps(n);
            let endpoint = refined.last().expect("positive step count").1;
            let label = kit.text(
                stage,
                &format!("{n} steps: end {endpoint:.3}"),
                23.0,
                Palette::blue(),
                v(2.75, 1.7),
            )?;
            morph(stage, &[(right_read, label)], 0.3)?;
            old_right.clear();
            for pair in refined.windows(2) {
                let m = line(
                    stage,
                    point(RIGHT, pair[0].0, pair[0].1),
                    point(RIGHT, pair[1].0, pair[1].1),
                    Palette::blue(),
                    3.5,
                )?;
                show(stage, &[m], 0.15)?;
                old_right.push(m);
            }
            stage.wait(1.0)?;
        }
        kit.hold(stage)?;
        kit.say(stage, "01_5")?;
        let promise = kit.text(
            stage,
            "Next: construct an exact solution",
            28.0,
            Palette::green(),
            v(0.0, 2.65),
        )?;
        show(stage, &[promise], 0.4)?;
        kit.finish(stage)
    }
}
