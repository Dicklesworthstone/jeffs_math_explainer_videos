//! A prediction first, followed by the three computed host-policy cases.
use explainer_kit::{Kit, Palette, Timed, rect, stroke_style, v};
use five_math_paradoxes::monty_cases;
use fmn::prelude::*;

pub struct MontyHall {
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

impl SceneConstruct for MontyHall {
    fn name(&self) -> &str {
        "01_monty_hall"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let cases = monty_cases();
        let facts = crate::facts::facts()
            .map_err(|_| fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("Monty facts failed")))?;
        kit.say(stage, "01_1")?;
        let mut prediction = vec![kit.text(stage, "Should you switch?", 42.0, Palette::ink(), v(0.0, 2.8))?];
        for (i, case) in cases.iter().enumerate() {
            let x = (i as f64 - 1.0) * 3.0;
            let door = rect(v(x - 0.7, -0.3), 1.4, 2.0, stroke_style(if case.car == case.initial { Palette::hero() } else { Palette::blue() }, 4.0))?;
            prediction.push(stage.arena_mut().add(door));
            prediction.push(kit.text(stage, &case.car.to_string(), 32.0, Palette::ink(), v(x, 0.7))?);
        }
        prediction.push(kit.text(stage, "You choose the first door", 25.0, Palette::hero(), v(0.0, -1.0))?);
        fade_in_all(stage, &prediction, 0.6)?;
        // One illustrative legal host action, not a claim that door 3 is
        // opened in every case. The later table computes each case separately.
        stage.wait(5.0)?;
        let host_example = kit.text(
            stage, "Example: host reveals a goat", 24.0, Palette::gold(), v(2.5, -1.8),
        )?;
        let goat = kit.text(stage, "GOAT", 20.0, Palette::gold(), v(3.0, 0.0))?;
        fade_in_all(stage, &[host_example, goat], 0.5)?;
        prediction.push(host_example);
        prediction.push(goat);
        // This hold follows the spoken question and precedes every result.
        kit.hold(stage)?;
        stage.wait(3.0)?;
        fade_out_all(stage, &prediction, 0.5)?;

        kit.say(stage, "01_2")?;
        let headers = ["Car behind", "Host opens", "Switch to", "Result"];
        let xs = [-4.8, -1.6, 1.6, 4.8];
        let mut heading = Vec::new();
        for (text, x) in headers.iter().zip(xs) {
            heading.push(kit.text(stage, text, 24.0, Palette::dim(), v(x, 2.3))?);
        }
        fade_in_all(stage, &heading, 0.5)?;
        let mut car_cells = Vec::new();
        for (i, case) in cases.iter().enumerate() {
            car_cells.push(kit.text(
                stage,
                &case.car.to_string(),
                31.0,
                Palette::ink(),
                v(xs[0], 1.4 - i as f64),
            )?);
        }
        fade_in_all(stage, &car_cells, 0.5)?;

        kit.say(stage, "01_3")?;
        for (i, case) in cases.iter().enumerate() {
            // Recognition-derived sentence marks guide this review preview;
            // they are not evidence of listening or accepted fine sync.
            if i == 1 { kit.until_sentence(stage, "01_3", 1)?; }
            let y = 1.4 - i as f64;
            let cells = [
                kit.text(stage, &case.revealed.to_string(), 31.0, Palette::dim(), v(xs[1], y))?,
                kit.text(stage, &case.switched.to_string(), 31.0, Palette::hero(), v(xs[2], y))?,
                kit.text(
                    stage,
                    if case.switch_wins { "WIN" } else { "LOSE" },
                    27.0,
                    if case.switch_wins {
                        Palette::green()
                    } else {
                        Palette::red()
                    },
                    v(xs[3], y),
                )?,
            ];
            fade_in_all(stage, &cells, 0.6)?;
            stage.wait(0.3)?;
        }
        kit.until_sentence(stage, "01_3", 3)?;
        let wins = facts.get("monty.switch_wins").unwrap_or("?");
        let total = facts.get("monty.doors").unwrap_or("?");
        let answer = kit.text(
            stage,
            &format!("{wins} / {total}"),
            42.0,
            Palette::green(),
            v(0.0, -2.2),
        )?;
        fade_in_all(stage, &[answer], 0.5)?;

        kit.say(stage, "01_4")?;
        let condition = kit.text(
            stage,
            "The host knows, reveals a goat, and always offers a switch.",
            22.0,
            Palette::dim(),
            v(0.0, -3.1),
        )?;
        fade_in_all(stage, &[condition], 0.5)?;
        kit.finish(stage)
    }
}
