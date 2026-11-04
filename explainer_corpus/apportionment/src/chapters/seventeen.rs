//! A comparison with no seat loss, with the answer held back. Unbuilt draft.
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;
use super::visual::fade;

pub struct Seventeen { pub kit: Kit }

impl SceneConstruct for Seventeen {
    fn name(&self) -> &str { "03_seventeen" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let bad = |_| fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("Hamilton cutoff tie"));
        let before = crate::facts::hamilton(16).map_err(bad)?;
        let after = crate::facts::hamilton(17).map_err(bad)?;
        if !(0..3).all(|i| after[i] >= before[i]) {
            return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("expected no seat loss")));
        }
        kit.say(stage, "03_1")?;
        let heading = kit.text(stage, "What if we add one more?", 36.0, Palette::ink(), v(0.0, 2.8))?;
        let previous = kit.text(stage, &format!("16 seats:  {} / {} / {}", before[0], before[1], before[2]), 33.0, Palette::blue(), v(0.0, 1.0))?;
        let question = kit.text(stage, "17 seats:  ? / ? / ?", 33.0, Palette::hero(), v(0.0, -0.4))?;
        fade(stage, &[heading, previous, question], true)?;
        kit.hold(stage)?;
        stage.wait(3.0)?;

        kit.say(stage, "03_2")?;
        kit.until_sentence(stage, "03_2", 2)?;
        fade(stage, &[question], false)?;
        let result = kit.text(stage, &format!("17 seats:  {} / {} / {}", after[0], after[1], after[2]), 33.0, Palette::green(), v(0.0, -0.4))?;
        fade(stage, &[result], true)?;
        kit.say(stage, "03_3")?;
        let conclusion = kit.text(stage, "This time nobody loses a seat", 27.0, Palette::ink(), v(0.0, -1.9))?;
        fade(stage, &[conclusion], true)?;
        kit.finish(stage)
    }
}
