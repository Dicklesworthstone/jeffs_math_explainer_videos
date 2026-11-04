//! Priority cutoff and modified divisor, using the exact 15-seat example.
use explainer_kit::{Kit, Palette, ink_style, rect, v};
use fmn::prelude::*;
use super::visual::fade;

pub struct Divisor { pub kit: Kit }

impl SceneConstruct for Divisor {
    fn name(&self) -> &str { "05_divisor" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let bad = |_| fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("Jefferson priority tie"));
        let seats = crate::facts::jefferson(15).map_err(bad)?;
        // 9/4 < 23/10 < 26/11, using integer comparisons.
        if crate::facts::POPULATIONS[1] * 10 >= 23 * (u64::from(seats[1]) + 1)
            || 23 * u64::from(seats[2]) >= crate::facts::POPULATIONS[2] * 10 {
            return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("invalid divisor cutoff")));
        }
        let computed = crate::facts::POPULATIONS.map(|p| p * 10 / 23);
        if computed != seats.map(u64::from) {
            return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("divisor allocation mismatch")));
        }

        kit.say(stage, "05_1")?;
        let title = kit.text(stage, "Where did the divisor go?", 38.0, Palette::ink(), v(0.0, 2.8))?;
        let subtitle = kit.text(stage, "A cutoff in the same priority list", 28.0, Palette::blue(), v(0.0, 1.8))?;
        fade(stage, &[title, subtitle], true)?;

        kit.say(stage, "05_2")?;
        let x = |value: f64| -4.0 + (value - 2.2) * 32.0;
        let axis = stage.arena_mut().add(rect(v(-4.0, 0.2), 8.0, 0.04, ink_style(Palette::dim()))?);
        let mut marks = vec![axis];
        for (value, label, color) in [
            (9.0 / 4.0, "next: 9/4", Palette::gold()),
            (23.0 / 10.0, "divisor: 2.3", Palette::hero()),
            (26.0 / 11.0, "last: 26/11", Palette::green()),
        ] {
            marks.push(stage.arena_mut().add(rect(v(x(value), 0.0), 0.06, 0.42, ink_style(color))?));
            marks.push(kit.text(stage, label, 23.0, color, v(x(value), -0.65))?);
        }
        fade(stage, &marks, true)?;
        kit.until_sentence(stage, "05_2", 3)?;
        let result = kit.text(stage, &format!("floor(population / 2.3) = {} / {} / {}", seats[0], seats[1], seats[2]), 28.0, Palette::ink(), v(0.0, -1.65))?;
        fade(stage, &[result], true)?;

        kit.say(stage, "05_3")?;
        let explanation = kit.text(stage, "One seat for each priority at or above the cutoff", 25.0, Palette::blue(), v(0.0, -2.65))?;
        fade(stage, &[explanation], true)?;
        kit.finish(stage)
    }
}
