//! Draft native FrankenManim analytic-continuation example.
//! Unintegrated, uncompiled, unrendered, unlistened and unreviewed.
//! 06_1 and 06_2 are proposed text cue boundaries, not verified audio cuts.
use explainer_kit::{Kit, Palette, Timed, v};
use fmn::prelude::*;

pub struct Continuation { pub kit: Kit }

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

fn partial_sum(z: f64, n: u32) -> f64 {
    (0..=n).fold((0.0, 1.0), |(sum, power), _| (sum+power, power*z)).0
}



impl SceneConstruct for Continuation {
    fn name(&self) -> &str { "06_continuation" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        // Research draft: 06_0 is a new spoken question, not yet recorded.
        let prediction = [
            kit.text(stage, "Will these sums settle?", 32.0, Palette::ink(), v(0.0,2.9))?,
            kit.text(stage, "1 + z + z² + ...", 28.0, Palette::blue(), v(0.0,2.1))?,
            kit.text(stage, "z = 0.5", 25.0, Palette::blue(), v(-3.2,1.3))?,
            kit.text(stage, "z = 1.2", 25.0, Palette::hero(), v(3.2,1.3))?,
            kit.text(stage, "Which partial sums settle?", 25.0, Palette::hero(), v(0.0,-2.2))?,
        ];
        appear(stage, &prediction)?;
        kit.say(stage, "06_0")?;
        kit.hold(stage)?;
        kit.mark(stage, "predict");
        stage.wait(3.0)?;
        disappear(stage, &prediction)?;
        kit.say(stage, "06_1")?;
        let title = kit.text(stage, "The function can continue; the series may not", 32.0, Palette::ink(), v(0.0,2.9))?;
        let identity = kit.text(stage, "1 + z + z² + ... = 1 / (1 − z),  |z| < 1", 25.0, Palette::blue(), v(0.0,2.1))?;
        let left = kit.text(stage, "z = 0.5: partial sums", 23.0, Palette::blue(), v(-3.2,1.3))?;
        let right = kit.text(stage, "z = 1.2: partial sums", 23.0, Palette::hero(), v(3.2,1.3))?;
        appear(stage, &[title,identity,left,right])?;
        kit.hold(stage)?;
        for (row,n) in [0_u32,1,2,5,10].into_iter().enumerate() {
            let y = 0.7 - row as f64*0.5;
            let a = kit.text(stage, &format!("n={n}: {:.3}",partial_sum(0.5,n)), 23.0, Palette::blue(), v(-3.2,y))?;
            let b = kit.text(stage, &format!("n={n}: {:.3}",partial_sum(1.2,n)), 23.0, Palette::hero(), v(3.2,y))?;
            appear(stage, &[a,b])?;
        }
        let conclusion = kit.text(stage, "The rational formula extends; the series still diverges at 1.2", 23.0, Palette::green(), v(0.0,-3.1))?;
        appear(stage, &[conclusion])?;
        kit.say(stage, "06_2")?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::partial_sum;
    #[test]
    fn partial_sums_are_not_the_continuation_outside_the_disk() {
        assert!((partial_sum(0.5,10)-1.9990234375).abs()<1e-10);
        assert!(partial_sum(1.2,10)>30.0);
        assert!((1.0/(1.0-1.2)+5.0_f64).abs()<1e-10);
    }
}
