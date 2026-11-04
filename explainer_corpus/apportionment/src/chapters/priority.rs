//! Jefferson's exact sequential priorities. Unbuilt and unrendered candidate.
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;
use super::visual::fade;

pub struct Priority { pub kit: Kit }

fn priorities(kit: &Kit, stage: &mut Stage<'_>, seats: [u32; 3], y: f64) -> fmn::Result<Vec<Mob>> {
    let mut result = Vec::new();
    let colors = [Palette::blue(), Palette::gold(), Palette::green()];
    for (i, x) in [-3.8, 0.0, 3.8].into_iter().enumerate() {
        let text = format!("{}: {} / {}", ['A', 'B', 'C'][i], crate::facts::POPULATIONS[i], seats[i] + 1);
        result.push(kit.text(stage, &text, 31.0, colors[i], v(x, y))?);
    }
    Ok(result)
}

impl SceneConstruct for Priority {
    fn name(&self) -> &str { "04_priority" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let bad = |_| fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("Jefferson priority tie"));
        let s14 = crate::facts::jefferson(14).map_err(bad)?;
        let s15 = crate::facts::jefferson(15).map_err(bad)?;
        let s16 = crate::facts::jefferson(16).map_err(bad)?;
        if s14 != [1, 3, 10] || s15 != [1, 3, 11] || s16 != [1, 4, 11] {
            return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("unexpected Jefferson example")));
        }
        kit.say(stage, "04_1")?;
        let title = kit.text(stage, "A different promise: award seats one at a time", 35.0, Palette::ink(), v(0.0, 2.8))?;
        let rule = kit.text(stage, "priority = population / (seats already won + 1)", 28.0, Palette::blue(), v(0.0, 1.6))?;
        fade(stage, &[title, rule], true)?;

        kit.say(stage, "04_2")?;
        let state = kit.text(stage, &format!("After 14:  {} / {} / {}", s14[0], s14[1], s14[2]), 28.0, Palette::ink(), v(0.0, 0.6))?;
        let mut panel = priorities(&kit, stage, s14, -0.4)?;
        panel.push(state);
        fade(stage, &panel, true)?;
        kit.until_sentence(stage, "04_2", 3)?;
        let winner = kit.text(stage, &format!("Seat 15: C wins → {} / {} / {}", s15[0], s15[1], s15[2]), 28.0, Palette::green(), v(0.0, -1.5))?;
        fade(stage, &[winner], true)?;
        panel.push(winner);

        kit.say(stage, "04_3")?;
        fade(stage, &panel, false)?;
        let new_state = kit.text(stage, &format!("After 15:  {} / {} / {}", s15[0], s15[1], s15[2]), 28.0, Palette::ink(), v(0.0, 0.6))?;
        let mut next = priorities(&kit, stage, s15, -0.4)?;
        next.push(new_state);
        fade(stage, &next, true)?;
        kit.until_sentence(stage, "04_3", 2)?;
        let winner = kit.text(stage, &format!("Seat 16: B wins → {} / {} / {}", s16[0], s16[1], s16[2]), 28.0, Palette::gold(), v(0.0, -1.5))?;
        fade(stage, &[winner], true)?;

        kit.say(stage, "04_4")?;
        let note = kit.text(stage, "The first 15 awards stay in the first 16", 27.0, Palette::ink(), v(0.0, -2.55))?;
        fade(stage, &[note], true)?;
        kit.finish(stage)
    }
}
