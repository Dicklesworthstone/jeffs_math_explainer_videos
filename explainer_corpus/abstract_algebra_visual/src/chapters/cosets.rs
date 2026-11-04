//! Native equivalence-class and left-coset bridge; build and review pending.
//! Proposed 05_1 is the first source sentence, 05_2 the rest of its first
//! paragraph, 05_3 the second paragraph. Retained ElevenLabs takes stay whole;
//! no proposed cue boundary is a listening-based audio cut.
use crate::model::{Symmetry, all, left_coset};
use explainer_kit::{Kit, Palette, Timed, ink_style, stroke_style, v};
use fmn::prelude::*;

pub struct Cosets {
    pub kit: Kit,
}

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

fn symbol(action: Symmetry) -> String {
    let rotation = match action.turns {
        0 => "",
        1 => "r",
        2 => "r²",
        3 => "r³",
        _ => unreachable!("model normalizes turns"),
    };
    match (rotation, action.reflected) {
        ("", false) => "e".into(),
        ("", true) => "s".into(),
        (r, false) => r.into(),
        (r, true) => format!("{r}s"),
    }
}

impl SceneConstruct for Cosets {
    fn name(&self) -> &str {
        "05_equivalence_cosets"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let modulus = crate::facts::CLOCK_MODULUS;
        let first = crate::facts::CLOCK_FIRST;
        let numbers = [first, first + modulus, first + 2 * modulus];
        let position = first.rem_euclid(modulus);
        if numbers.iter().any(|number| number.rem_euclid(modulus) != position) {
            return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("clock classes differ")));
        }
        kit.say(stage, "05_1")?;
        let title = kit.text(stage, "Which positions match?", 36.0, Palette::ink(), v(0.0, 2.8))?;
        let ring = Circle::new().radius(1.8).style(stroke_style(Palette::dim(), 3.0)).build().moved_to(v(-2.7, 0.0));
        let ring = stage.arena_mut().add(ring);
        let mut clock = vec![title, ring];
        for n in 0..modulus {
            let angle = std::f64::consts::FRAC_PI_2 - std::f64::consts::TAU * f64::from(n) / f64::from(modulus);
            let x = -2.7 + 1.3 * angle.cos();
            let y = 1.3 * angle.sin();
            clock.push(kit.text(stage, &n.to_string(), 25.0, Palette::ink(), v(x, y))?);
        }
        clock.push(kit.text(stage, &format!("{}, {}, {}", numbers[0], numbers[1], numbers[2]), 33.0, Palette::hero(), v(2.6, 0.7))?);
        clock.push(kit.text(stage, "On the same four-step clock?", 25.0, Palette::ink(), v(2.6, -0.3))?);
        fade(stage, &clock, true)?;
        kit.hold(stage)?;
        stage.wait(3.0)?;

        kit.say(stage, "05_2")?;
        let angle = std::f64::consts::FRAC_PI_2 - std::f64::consts::TAU * f64::from(position) / f64::from(modulus);
        let marker = Circle::new().radius(0.12).style(ink_style(Palette::green())).build()
            .moved_to(v(-2.7 + 0.9 * angle.cos(), 0.9 * angle.sin()));
        let marker = stage.arena_mut().add(marker);
        let match_text = kit.text(stage, &format!("All land at {position}"), 28.0, Palette::green(), v(2.6, -1.2))?;
        let answer = [marker, match_text];
        fade(stage, &answer, true)?;
        kit.until_sentence(stage, "05_2", 1)?;
        let properties = kit.text(stage, "Reflexive · symmetric · transitive", 26.0, Palette::dim(), v(0.0, -2.8))?;
        fade(stage, &[properties], true)?;

        kit.say(stage, "05_3")?;
        clock.extend(answer);
        clock.push(properties);
        fade(stage, &clock, false)?;
        let subgroup = [Symmetry::ID, Symmetry::S];
        let group = all();
        let mut cosets: Vec<Vec<Symmetry>> = Vec::new();
        for action in group {
            if !cosets.iter().any(|coset| coset.contains(&action)) {
                cosets.push(left_coset(action, &subgroup));
            }
        }
        if group.iter().any(|action| cosets.iter().filter(|coset| coset.contains(action)).count() != 1)
            || cosets.iter().any(|coset| coset.len() != subgroup.len())
        {
            return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("invalid coset partition")));
        }
        let heading = [
            kit.text(stage, "A subgroup partitions the square moves", 34.0, Palette::ink(), v(0.0, 2.7))?,
            kit.text(stage, "H = {e, s}; classes are left cosets gH", 27.0, Palette::dim(), v(0.0, 1.8))?,
        ];
        fade(stage, &heading, true)?;
        kit.until_sentence(stage, "05_3", 1)?;
        let mut rows = Vec::new();
        for (index, coset) in cosets.iter().enumerate() {
            let members: Vec<String> = coset.iter().copied().map(symbol).collect();
            let x = if index % 2 == 0 { -2.8 } else { 2.8 };
            let y = 0.7 - (index / 2) as f64 * 1.2;
            rows.push(kit.text(stage, &format!("{{{}}}", members.join(", ")), 35.0, Palette::blue(), v(x, y))?);
        }
        fade(stage, &rows, true)?;
        kit.until_sentence(stage, "05_3", 3)?;
        let count = kit.text(stage, &format!("{} classes of {} moves", cosets.len(), subgroup.len()), 28.0, Palette::green(), v(0.0, -2.1))?;
        fade(stage, &[count], true)?;
        kit.finish(stage)
    }
}
