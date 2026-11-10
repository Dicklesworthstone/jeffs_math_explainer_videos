//! Draft native FrankenManim argument-principle scene.
//! Not integrated, compiled, rendered, narrated, listened to or reviewed.
//! Cue boundaries 04_1/04_2 are proposed text only.
use explainer_kit::{Kit, Palette, Timed, ink_style, stroke_style, v};
use fmn::prelude::*;

pub struct ArgumentPrinciple { pub kit: Kit }

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

/// f(z) = z²/(z-1/2); caller supplies a point on the unit circle.
fn output_direction(theta: f64) -> (f64, f64) {
    let (y, x) = theta.sin_cos();
    let (a, b) = (x*x-y*y, 2.0*x*y);
    let (c, d) = (x-0.5, y);
    let denominator = c*c+d*d;
    let (re, im) = ((a*c+b*d)/denominator, (b*c-a*d)/denominator);
    let magnitude = re.hypot(im);
    (re/magnitude, im/magnitude)
}



impl SceneConstruct for ArgumentPrinciple {
    fn name(&self) -> &str { "04_count_turns" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let left = (-3.0_f64, 0.0_f64);
        let right = (3.0_f64, 0.0_f64);
        let radius = 1.5_f64;
        let steps = 48_u32;
        let heading = kit.text(stage, "Count zeros and poles by following a loop", 34.0, Palette::ink(), v(0.0, 3.0))?;
        let formula = kit.text(stage, "f(z) = z² / (z − 1/2)", 27.0, Palette::blue(), v(0.0, 2.2))?;
        let input_label = kit.text(stage, "Input loop", 24.0, Palette::blue(), v(-3.0, -2.1))?;
        let output_label = kit.text(stage, "Output direction", 24.0, Palette::green(), v(3.0, -2.1))?;
        let left_ring = stage.arena_mut().add(Circle::new().radius(radius).style(stroke_style(Palette::dim(), 3.0)).build().moved_to(v(left.0,left.1)));
        let right_ring = stage.arena_mut().add(Circle::new().radius(radius).style(stroke_style(Palette::dim(), 3.0)).build().moved_to(v(right.0,right.1)));
        let zero = stage.arena_mut().add(Circle::new().radius(0.10).style(ink_style(Palette::blue())).build().moved_to(v(left.0,left.1)));
        let pole = stage.arena_mut().add(Circle::new().radius(0.10).style(ink_style(Palette::hero())).build().moved_to(v(left.0+radius*0.5,left.1)));
        let zero_label = kit.text(stage, "zero ×2", 18.0, Palette::blue(), v(left.0-0.35, -0.45))?;
        let pole_label = kit.text(stage, "pole", 18.0, Palette::hero(), v(left.0+0.85, 0.45))?;
        let input_point = stage.arena_mut().add(Circle::new().radius(0.13).style(ink_style(Palette::blue())).build().moved_to(v(left.0+radius,left.1)));
        let (u,w) = output_direction(0.0);
        let output_point = stage.arena_mut().add(Circle::new().radius(0.13).style(ink_style(Palette::green())).build().moved_to(v(right.0+radius*u,right.1+radius*w)));
        let question = kit.text(stage, "How many net turns around zero?", 26.0, Palette::hero(), v(0.0, -3.0))?;
        appear(stage, &[heading,formula,input_label,output_label,left_ring,right_ring,zero,pole,zero_label,pole_label,input_point,output_point,question])?;
        stage.wait(3.0)?;
        kit.say(stage, "04_1")?;
        for step in 1..=steps {
            let theta = std::f64::consts::TAU*f64::from(step)/f64::from(steps);
            let (u,w) = output_direction(theta);
            let first = input_point.animate().set_anim_args(AnimateArgs { run_time: Some(0.12), ..AnimateArgs::default() })?.move_to(v(left.0+radius*theta.cos(),left.1+radius*theta.sin()),ORIGIN)?;
            let first = stage.prepare(first)?;
            let second = output_point.animate().set_anim_args(AnimateArgs { run_time: Some(0.12), ..AnimateArgs::default() })?.move_to(v(right.0+radius*u,right.1+radius*w),ORIGIN)?;
            let second = stage.prepare(second)?;
            stage.play_prepared(vec![first,second])?;
        }
        kit.until_sentence(stage, "04_1", 2)?;
        let dismiss = fade_out(stage.arena_mut(), question, ORIGIN, 1.0)?;
        let dismiss = stage.prepare(dismiss.rt(0.4))?;
        stage.play_prepared(vec![dismiss])?;
        let result = kit.text(stage, "2 zeros − 1 pole = 1 net turn", 28.0, Palette::hero(), v(0.0, -2.55))?;
        appear(stage, &[result])?;
        kit.say(stage, "04_2")?;
        kit.until_sentence(stage, "04_2", 1)?;
        disappear(stage, &[formula,input_label,output_label,left_ring,right_ring,zero,pole,zero_label,pole_label,input_point,output_point,result])?;
        let explanation = kit.text(stage, "Residues of the logarithmic derivative f′/f", 29.0, Palette::ink(), v(0.0, 1.8))?;
        let zero_residue = kit.text(stage, "zero of order m  →  residue +m", 29.0, Palette::blue(), v(0.0, 0.7))?;
        appear(stage, &[explanation,zero_residue])?;
        kit.until_sentence(stage, "04_2", 2)?;
        let pole_residue = kit.text(stage, "pole of order n  →  residue −n", 29.0, Palette::gold(), v(0.0, -0.2))?;
        appear(stage, &[pole_residue])?;
        kit.until_sentence(stage, "04_2", 3)?;
        let contour_sum = kit.text(stage, "(1 / 2πi) ∮ (f′/f) dz = zeros − poles", 28.0, Palette::hero(), v(0.0, -1.4))?;
        appear(stage, &[contour_sum])?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::output_direction;
    #[test]
    fn sample_directions_make_one_net_turn() {
        let mut turn=0.0_f64;
        let mut previous=output_direction(0.0);
        for k in 1..=96 {
            let next=output_direction(std::f64::consts::TAU*f64::from(k)/96.0);
            turn+=(previous.0*next.1-previous.1*next.0).atan2(previous.0*next.0+previous.1*next.1);
            previous=next;
        }
        assert!((turn-std::f64::consts::TAU).abs()<1e-10);
    }
}
