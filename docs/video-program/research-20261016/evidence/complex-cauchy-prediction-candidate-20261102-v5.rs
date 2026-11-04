//! Draft native FrankenManim Cauchy-formula illustration.
//! Unintegrated, uncompiled, unrendered, unlistened and unreviewed.
//! Cue boundaries 03_1/03_2 are proposed text, not verified audio cuts.
use explainer_kit::{Kit, Palette, Timed, ink_style, stroke_style, v};
use fmn::prelude::*;

pub struct Cauchy { pub kit: Kit }

fn appear(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_in(stage.arena_mut(),mob,ORIGIN,1.0)?;
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

/// A finite illustration for f(z)=1+z+z², not a proof of Cauchy's formula.
fn sampled_average(n: u32) -> (f64,f64) {
    let mut sum=(0.0,0.0);
    for k in 0..n {
        let t=std::f64::consts::TAU*f64::from(k)/f64::from(n);
        sum.0+=1.0+t.cos()+(2.0*t).cos();
        sum.1+=t.sin()+(2.0*t).sin();
    }
    (sum.0/f64::from(n),sum.1/f64::from(n))
}



impl SceneConstruct for Cauchy {
    fn name(&self) -> &str { "03_cauchy" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit=self.kit.clone();
        let center=(-3.1_f64,0.0_f64);let radius=1.7_f64;let n=8_u32;
        let title=kit.text(stage,"What can the boundary tell us?",34.0,Palette::ink(),v(0.0,2.9))?;
        let ring=stage.arena_mut().add(Circle::new().radius(radius).style(stroke_style(Palette::gold(),3.0)).build().moved_to(v(center.0,center.1)));
        let middle=stage.arena_mut().add(Circle::new().radius(0.10).style(ink_style(Palette::green())).build().moved_to(v(center.0,center.1)));
        let point=stage.arena_mut().add(Circle::new().radius(0.12).style(ink_style(Palette::blue())).build().moved_to(v(center.0+radius,center.1)));
        let example=kit.text(stage,"f(z) = 1 + z + z²",26.0,Palette::blue(),v(2.2,1.6))?;
        let known=kit.text(stage,"All boundary values known",24.0,Palette::gold(),v(2.2,1.6))?;
        let question=kit.text(stage,"Can boundary values recover f(0)?",24.0,Palette::hero(),v(2.2,0.7))?;
        // Keep the function itself hidden while the viewer considers what
        // boundary values alone could reveal about the center.
        appear(stage,&[title,ring,middle,point,known,question])?;
        // Research draft: 03_0 is a new spoken question, not yet recorded.
        kit.say(stage,"03_0")?;
        kit.hold(stage)?;
        kit.mark(stage,"predict");
        stage.wait(3.0)?;
        kit.say(stage,"03_1")?;
        kit.hold(stage)?;
        let hide_known=fade_out(stage.arena_mut(),known,ORIGIN,1.0)?;
        let hide_known=stage.prepare(hide_known.rt(0.5))?;
        stage.play_prepared(vec![hide_known])?;
        appear(stage,&[example])?;
        for k in 1..=n {
            let t=std::f64::consts::TAU*f64::from(k)/f64::from(n);
            let anim=point.animate().set_anim_args(AnimateArgs {run_time:Some(0.25),..AnimateArgs::default()})?.move_to(v(center.0+radius*t.cos(),center.1+radius*t.sin()),ORIGIN)?;
            let anim=stage.prepare(anim)?;stage.play_prepared(vec![anim])?;
        }
        let average=sampled_average(n);
        if average.1.abs() > 1e-9 { return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("unexpected sample imaginary average"))); }
        let samples=kit.text(stage,&format!("8 sample values average {:.2} (imag ≈ 0)",average.0),22.0,Palette::green(),v(1.9,-0.3))?;
        let note=kit.text(stage,"Finite example, not a general proof",21.0,Palette::dim(),v(2.2,-1.0))?;
        let formula=kit.text(stage,"f(0) = (1 / 2π) ∫ from 0 to 2π f(e^(it)) dt",24.0,Palette::hero(),v(0.0,-2.3))?;
        let domain=kit.text(stage,"Holomorphic on a neighborhood of the closed disk",21.0,Palette::ink(),v(0.0,-3.0))?;
        appear(stage,&[samples,note,formula,domain])?;
        kit.say(stage,"03_2")?;
        let result=kit.text(stage,"Boundary values determine every derivative at the center",22.0,Palette::green(),v(2.2,-1.6))?;
        appear(stage,&[result])?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::sampled_average;
    #[test]
    fn eight_samples_cancel_nonconstant_terms() {
        let (re,im)=sampled_average(8);
        assert!((re-1.0).abs()<1e-12 && im.abs()<1e-12);
    }
}
