//! First native scene candidate. Unbuilt, unrendered and not independently reviewed.
use explainer_kit::{Kit, Palette, Timed, ink_style, rect, v};
use fmn::prelude::*;

pub struct SeatDisappears { pub kit: Kit }

fn appear(stage: &mut Stage<'_>, mobs: &[Mob], seconds: f64) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(seconds))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn seat_row(kit: &Kit, stage: &mut Stage<'_>, seats: [u32; 3], y: f64) -> fmn::Result<Vec<Mob>> {
    let mut mobs = Vec::new();
    let centers = [-3.4, 0.0, 3.4];
    let colors = [Palette::blue(), Palette::gold(), Palette::green()];
    for i in 0..3 {
        mobs.push(kit.text(stage, &seats[i].to_string(), 26.0, colors[i], v(centers[i], y + 0.52))?);
        for seat in 0..seats[i] {
            let x = centers[i] - 0.56 + f64::from(seat % 5) * 0.28;
            let row_y = y - f64::from(seat / 5) * 0.26;
            mobs.push(stage.arena_mut().add(rect(v(x, row_y), 0.19, 0.19, ink_style(colors[i]))?));
        }
    }
    Ok(mobs)
}

impl SceneConstruct for SeatDisappears {
    fn name(&self) -> &str { "01_seat_disappears" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let failure = |_| fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("apportionment example"));
        let before = crate::facts::hamilton(15).map_err(failure)?;
        let after = crate::facts::hamilton(16).map_err(failure)?;
        if before != [2, 3, 10] || after != [1, 4, 11] {
            return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("unexpected seat example")));
        }

        kit.say(stage, "01_1")?;
        let heading = kit.text(stage, "Can adding a seat take one away?", 36.0, Palette::ink(), v(0.0, 3.0))?;
        let mut population = vec![heading];
        for (i, x) in [-3.4, 0.0, 3.4].into_iter().enumerate() {
            let label = format!("{}: population {}", ['A', 'B', 'C'][i], crate::facts::POPULATIONS[i]);
            population.push(kit.text(stage, &label, 25.0, Palette::ink(), v(x, 2.05))?);
        }
        appear(stage, &population, 0.5)?;

        kit.say(stage, "01_2")?;
        let before_label = kit.text(stage, "15 seats", 23.0, Palette::dim(), v(-6.0, 0.8))?;
        let question = kit.text(stage, "Add one seat. What happens to A?", 29.0, Palette::hero(), v(0.0, -2.8))?;
        let mut first = seat_row(&kit, stage, before, 0.65)?;
        first.extend([before_label, question]);
        appear(stage, &first, 0.6)?;
        kit.hold(stage)?;
        stage.wait(3.0)?;

        kit.say(stage, "01_3")?;
        let after_label = kit.text(stage, "16 seats", 23.0, Palette::dim(), v(-6.0, -1.25))?;
        let mut second = seat_row(&kit, stage, after, -1.25)?;
        second.push(after_label);
        appear(stage, &second, 0.6)?;
        kit.finish(stage)
    }
}
