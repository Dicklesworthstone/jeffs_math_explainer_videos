//! The quota cost of the same Jefferson allocation. Unbuilt draft.
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;
use super::visual::fade;

pub struct Quota { pub kit: Kit }

impl SceneConstruct for Quota {
    fn name(&self) -> &str { "06_quota" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let bad = |_| fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("Jefferson priority tie"));
        let allocation = crate::facts::jefferson(15).map_err(bad)?;
        let population = crate::facts::POPULATIONS[2];
        let total: u64 = crate::facts::POPULATIONS.iter().sum();
        let numerator = 15 * population;
        if !numerator.is_multiple_of(total) || allocation[2] as u64 <= numerator / total {
            return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("expected quota violation")));
        }
        let quota = numerator / total;
        kit.say(stage, "06_1")?;
        let title = kit.text(stage, "The cost of this rule", 38.0, Palette::ink(), v(0.0, 2.8))?;
        let exact = kit.text(stage, &format!("C's exact share: 15 × {population} / {total} = {quota}"), 31.0, Palette::blue(), v(0.0, 1.25))?;
        let bounds = kit.text(stage, &format!("Allowed by quota: {quota} to {quota}"), 30.0, Palette::ink(), v(0.0, 0.1))?;
        fade(stage, &[title, exact, bounds], true)?;
        kit.until_sentence(stage, "06_1", 1)?;
        let award = kit.text(stage, &format!("Jefferson gives C {}", allocation[2]), 33.0, Palette::hero(), v(0.0, -1.0))?;
        fade(stage, &[award], true)?;

        kit.say(stage, "06_2")?;
        let note = kit.text(stage, "Standard share and modified divisor are different quantities", 26.0, Palette::gold(), v(0.0, -2.3))?;
        fade(stage, &[note], true)?;
        kit.finish(stage)
    }
}
