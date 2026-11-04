//! Exact Hamilton quota comparison candidate. Unbuilt and unrendered.
use explainer_kit::{Kit, Palette, Timed, v};
use fmn::prelude::*;

pub struct Remainders { pub kit: Kit }

fn fade(stage: &mut Stage<'_>, mobs: &[Mob], incoming: bool) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = if incoming {
            fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?
        } else {
            fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)?
        };
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 { (a, b) = (b, a % b); }
    a
}

fn shares(kit: &Kit, stage: &mut Stage<'_>, house: u32) -> fmn::Result<Vec<Mob>> {
    let total: u64 = crate::facts::POPULATIONS.iter().sum();
    let colors = [Palette::blue(), Palette::gold(), Palette::green()];
    let centers = [-3.8, 0.0, 3.8];
    let mut mobs = vec![kit.text(stage, &format!("{house} seats"), 31.0, Palette::ink(), v(0.0, 1.8))?];
    for i in 0..3 {
        let n = u64::from(house) * crate::facts::POPULATIONS[i];
        let whole = n / total;
        let rem = n % total;
        let fraction = if rem == 0 { "0".to_string() } else {
            let d = if house == 16 { 1 } else { gcd(rem, total) };
            format!("{}/{}", rem / d, total / d)
        };
        let label = format!("{}: {whole} + {fraction}", ['A', 'B', 'C'][i]);
        mobs.push(kit.text(stage, &label, 28.0, colors[i], v(centers[i], 0.6))?);
    }
    Ok(mobs)
}

impl SceneConstruct for Remainders {
    fn name(&self) -> &str { "02_remainders" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let bad = |_| fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("Hamilton cutoff tie"));
        let first = crate::facts::hamilton(15).map_err(bad)?;
        let second = crate::facts::hamilton(16).map_err(bad)?;
        kit.say(stage, "02_1")?;
        let heading = kit.text(stage, "Hamilton: whole seats, then largest tails", 35.0, Palette::ink(), v(0.0, 3.0))?;
        fade(stage, &[heading], true)?;
        let mut first_panel = shares(&kit, stage, 15)?;
        fade(stage, &first_panel, true)?;

        kit.say(stage, "02_2")?;
        let answer = kit.text(stage, &format!("A gets the last seat:  {} / {} / {}", first[0], first[1], first[2]), 28.0, Palette::hero(), v(0.0, -1.0))?;
        fade(stage, &[answer], true)?;
        first_panel.push(answer);

        kit.say(stage, "02_3")?;
        fade(stage, &first_panel, false)?;
        let second_panel = shares(&kit, stage, 16)?;
        fade(stage, &second_panel, true)?;

        kit.say(stage, "02_4")?;
        let answer = kit.text(stage, &format!("B and C get the two seats:  {} / {} / {}", second[0], second[1], second[2]), 28.0, Palette::hero(), v(0.0, -1.0))?;
        fade(stage, &[answer], true)?;
        kit.finish(stage)
    }
}
