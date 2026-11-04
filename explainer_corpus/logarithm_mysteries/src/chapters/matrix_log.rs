//! Draft native FrankenManim matrix-logarithm scene.
//! Unintegrated, uncompiled, unrendered, unlistened and unreviewed.
//! Proposed 05_1/05_2 text cues do not establish audio cuts.
use explainer_kit::{Kit, Palette, Timed, ink_style, stroke_style, v};
use fmn::prelude::*;

pub struct MatrixLog { pub kit: Kit }

fn appear(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn disappear(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

impl SceneConstruct for MatrixLog {
    fn name(&self) -> &str { "05_matrix_log" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let center = (-3.1_f64, 0.0_f64);
        let radius = 1.5_f64;
        let steps = 24_u32;
        let title = kit.text(stage, "Can a transformation have a logarithm?", 34.0, Palette::ink(), v(0.0,2.9))?;
        let ring = stage.arena_mut().add(Circle::new().radius(radius).style(stroke_style(Palette::dim(),3.0)).build().moved_to(v(center.0,center.1)));
        let point = stage.arena_mut().add(Circle::new().radius(0.14).style(ink_style(Palette::blue())).build().moved_to(v(center.0+radius,center.1)));
        // A finite lattice makes the action on the plane visible, rather than
        // implying that one returning point proves a transformation is identity.
        let mut grid = Vec::new();
        for x in [-0.9_f64, -0.3, 0.3, 0.9] {
            for y in [-0.9_f64, -0.3, 0.3, 0.9] {
                let color = if x > 0.8 && y > 0.8 { Palette::hero() } else { Palette::dim() };
                let mob = stage.arena_mut().add(Circle::new().radius(0.055).style(ink_style(color)).build().moved_to(v(center.0+x,center.1+y)));
                grid.push((mob, x, y));
            }
        }
        let identity = kit.text(stage, "exp(0) = I", 27.0, Palette::blue(), v(2.2,1.4))?;
        let predict = kit.text(stage, "What does a full turn give?", 25.0, Palette::hero(), v(2.2,0.5))?;
        let mut initial = vec![title,ring,point,identity,predict];
        initial.extend(grid.iter().map(|(mob, _, _)| *mob));
        appear(stage, &initial)?;
        stage.wait(3.0)?;
        kit.say(stage, "05_1")?;
        kit.until_sentence(stage, "05_1", 1)?;
        for step in 1..=steps {
            let t = std::f64::consts::TAU*f64::from(step)/f64::from(steps);
            let animation = point.animate().set_anim_args(AnimateArgs { run_time: Some(0.15), ..AnimateArgs::default() })?.move_to(v(center.0+radius*t.cos(),center.1+radius*t.sin()),ORIGIN)?;
            let mut animations = vec![stage.prepare(animation)?];
            for &(mob, x, y) in &grid {
                let (sin, cos) = t.sin_cos();
                let destination = v(center.0 + x*cos - y*sin, center.1 + x*sin + y*cos);
                let animation = mob.animate().set_anim_args(AnimateArgs { run_time: Some(0.15), ..AnimateArgs::default() })?.move_to(destination, ORIGIN)?;
                animations.push(stage.prepare(animation)?);
            }
            stage.play_prepared(animations)?;
        }
        let generator = kit.text(stage, "J rotates by 90 degrees", 23.0, Palette::ink(), v(2.2,-0.4))?;
        let full_turn = kit.text(stage, "exp(2πJ) = I", 27.0, Palette::green(), v(2.2,-1.2))?;
        appear(stage, &[generator,full_turn])?;
        kit.say(stage, "05_2")?;
        let reflection = kit.text(stage, "Mirror reflection: det = −1", 25.0, Palette::hero(), v(0.0,-2.25))?;
        let obstruction = kit.text(stage, "For real X, det(exp X) = exp(tr X) > 0", 24.0, Palette::ink(), v(0.0,-3.1))?;
        appear(stage, &[reflection,obstruction])?;
        kit.until_sentence(stage, "05_2", 2)?;
        let mut outgoing = vec![ring,point,identity,predict,generator,full_turn,reflection,obstruction];
        outgoing.extend(grid.iter().map(|(mob, _, _)| *mob));
        disappear(stage, &outgoing)?;
        let complex = kit.text(stage, "Allow complex matrices:", 28.0, Palette::ink(), v(0.0,0.9))?;
        let example = kit.text(stage, "exp(diag(iπ, 0)) = diag(−1, 1)", 29.0, Palette::green(), v(0.0,-0.1))?;
        appear(stage, &[complex,example])?;
        kit.finish(stage)
    }
}
