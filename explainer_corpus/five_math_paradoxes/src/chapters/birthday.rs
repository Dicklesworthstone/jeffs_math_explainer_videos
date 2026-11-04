//! Birthday prediction and complement under the stated uniform model.
use explainer_kit::{Kit, Palette, Timed, ink_style, v};
use five_math_paradoxes::{BIRTHDAY_DAYS, birthday_match_probability};
use fmn::prelude::*;

pub struct Birthday {
    pub kit: Kit,
}

fn fade_in_all(stage: &mut Stage<'_>, mobs: &[Mob], duration: f64) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(duration))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn fade_out_all(stage: &mut Stage<'_>, mobs: &[Mob], duration: f64) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(duration))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

impl SceneConstruct for Birthday {
    fn name(&self) -> &str {
        "02_birthday"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let count = (0..=BIRTHDAY_DAYS + 1)
            .find(|&n| birthday_match_probability(n) > 0.5)
            .expect("the pigeonhole case must pass one half");
        let facts = crate::facts::facts()
            .map_err(|_| fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("Birthday facts failed")))?;
        kit.say(stage, "02_1")?;
        let question = kit.text(
            stage,
            "When does a shared birthday become more likely than not?",
            31.0,
            Palette::ink(),
            v(0.0, 1.0),
        )?;
        fade_in_all(stage, &[question], 0.5)?;
        kit.hold(stage)?;
        stage.wait(3.0)?;
        fade_out_all(stage, &[question], 0.5)?;

        kit.say(stage, "02_2")?;
        let count_text = facts.get("birthday.people").unwrap_or("?");
        let heading = kit.text(
            stage,
            &format!("{count_text} people: any matching pair"),
            36.0,
            Palette::ink(),
            v(0.0, 2.7),
        )?;
        fade_in_all(stage, &[heading], 0.4)?;
        let mut dots = Vec::new();
        for i in 0..count {
            let col = i % 8;
            let row = i / 8;
            let x = (col as f64 - 3.5) * 1.0;
            let y = 1.3 - row as f64 * 0.72;
            let dot = Circle::new()
                .radius(0.19)
                .style(ink_style(Palette::blue()))
                .build()
                .moved_to(v(x, y));
            dots.push(stage.arena_mut().add(dot));
        }
        fade_in_all(stage, &dots, 0.9)?;

        kit.say(stage, "02_3")?;
        let days = facts.get("birthday.days").unwrap_or("?");
        let second = facts.get("birthday.second_numerator").unwrap_or("?");
        let third = facts.get("birthday.third_numerator").unwrap_or("?");
        let model = kit.text(
            stage,
            &format!("Independent, uniform birthdays; {days} days"),
            22.0,
            Palette::dim(),
            v(0.0, -1.2),
        )?;
        fade_in_all(stage, &[model], 0.6)?;
        kit.until_sentence(stage, "02_3", 2)?;
        let first_factor = kit.text(
            stage, &format!("{second}/{days}"), 30.0, Palette::ink(), v(-1.4, -1.75),
        )?;
        fade_in_all(stage, &[first_factor], 0.4)?;
        kit.until_sentence(stage, "02_3", 3)?;
        let next_factor = kit.text(
            stage, &format!("× {third}/{days} × …"), 30.0, Palette::ink(), v(1.0, -1.75),
        )?;
        fade_in_all(stage, &[next_factor], 0.4)?;

        kit.say(stage, "02_4")?;
        let no_match = facts.get("birthday.no_match_pct").unwrap_or("?");
        let match_pct = facts.get("birthday.match_pct").unwrap_or("?");
        let complement = kit.text(
            stage,
            &format!("no match: {no_match}%"),
            27.0,
            Palette::dim(),
            v(-2.8, -2.4),
        )?;
        let result = kit.text(
            stage,
            &format!("at least one match: {match_pct}%"),
            27.0,
            Palette::green(),
            v(2.5, -2.4),
        )?;
        kit.until_sentence(stage, "02_4", 1)?;
        fade_in_all(stage, &[complement], 0.5)?;
        kit.until_sentence(stage, "02_4", 2)?;
        fade_in_all(stage, &[result], 0.5)?;

        kit.say(stage, "02_5")?;
        let pairs = facts.get("birthday.pairs").unwrap_or("?");
        let pair_note = kit.text(stage, &format!("{pairs} possible pairs"), 24.0, Palette::hero(), v(-2.7, -3.0))?;
        fade_in_all(stage, &[pair_note], 0.5)?;
        kit.until_sentence(stage, "02_5", 1)?;
        let dependence = kit.text(stage, "Match events are not mutually independent", 24.0, Palette::hero(), v(1.4, -3.0))?;
        fade_in_all(stage, &[dependence], 0.5)?;
        kit.finish(stage)
    }
}
