//! Draft native fmn Zeno chapter; not installed or built yet.
use explainer_kit::{Kit, Palette, Timed, ink_style, rect, v};
use fmn::prelude::*;

pub struct Zeno {
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

impl SceneConstruct for Zeno {
    fn name(&self) -> &str {
        "04_zeno"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let facts = crate::facts::facts().map_err(|_| {
            fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("Zeno facts failed"))
        })?;
        let get = |key: &str| facts.get(key).unwrap_or("?");
        kit.say(stage, "04_1")?;
        let prediction = [
            kit.text(stage, "Can Achilles catch the tortoise?", 37.0, Palette::ink(), v(0.0, 2.6))?,
            kit.text(stage, &format!("head start: {} metres", get("zeno.head_start")), 29.0, Palette::dim(), v(0.0, 1.2))?,
            kit.text(stage, &format!("Achilles: {} m/s", get("zeno.pursuer_speed")), 28.0, Palette::blue(), v(-2.8, 0.2))?,
            kit.text(stage, &format!("tortoise: {} m/s", get("zeno.other_speed")), 28.0, Palette::gold(), v(2.8, 0.2))?,
        ];
        fade_in_all(stage, &prediction, 0.6)?;
        kit.hold(stage)?;
        stage.wait(3.0)?;
        fade_out_all(stage, &prediction, 0.4)?;

        kit.say(stage, "04_2")?;
        let heading = kit.text(stage, "Each stage takes a tenth as long", 32.0, Palette::ink(), v(0.0, 2.8))?;
        fade_in_all(stage, &[heading], 0.4)?;
        let values = ["zeno.stage0", "zeno.stage1", "zeno.stage2", "zeno.stage3"];
        let first = get(values[0]).parse::<f64>().map_err(|_| {
            fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("invalid first stage"))
        })?;
        kit.until_sentence(stage, "04_2", 1)?;
        let mut chart = vec![heading];
        for (i, key) in values.iter().enumerate() {
            let y = 1.8 - i as f64 * 0.75;
            let seconds = get(key).parse::<f64>().map_err(|_| {
                fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("invalid stage"))
            })?;
            let bar = rect(v(-4.4, y - 0.12), 6.0 * seconds / first, 0.24, ink_style(Palette::blue()))?;
            let bar = stage.arena_mut().add(bar);
            let value = kit.text(stage, &format!("{} s", get(key)), 25.0, Palette::ink(), v(3.5, y))?;
            fade_in_all(stage, &[bar, value], 0.35)?;
            chart.extend([bar, value]);
            stage.wait(1.0)?;
        }
        kit.until_sentence(stage, "04_2", 2)?;
        let sum = kit.text(stage, &format!("sum = {} s ≈ {} s", get("zeno.catch_fraction"), get("zeno.catch_decimal")), 31.0, Palette::green(), v(0.0, -1.8))?;
        fade_in_all(stage, &[sum], 0.5)?;
        chart.push(sum);

        kit.say(stage, "04_3")?;
        fade_out_all(stage, &chart, 0.4)?;
        let number = |key: &str| get(key).parse::<f64>().map_err(|_| {
            fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("invalid race value"))
        });
        let head = number("zeno.head_start")?;
        let fast = number("zeno.pursuer_speed")?;
        let slow = number("zeno.other_speed")?;
        if !(fast > slow && slow >= 0.0 && head > 0.0) {
            return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("invalid race assumptions")));
        }
        let catch_time = head / (fast - slow);
        let track_left = -4.5;
        let scale = 9.0 / (fast * catch_time);
        let lane = stage.arena_mut().add(Line::new(v(track_left, 0.0), v(4.7, 0.0)).style(explainer_kit::stroke_style(Palette::dim(), 3.0)).build()?);
        let achilles = stage.arena_mut().add(Circle::new().radius(0.18).style(ink_style(Palette::blue())).build().moved_to(v(track_left, 0.4)));
        let tortoise = stage.arena_mut().add(Circle::new().radius(0.18).style(ink_style(Palette::gold())).build().moved_to(v(track_left + head * scale, -0.4)));
        let race = [
            lane, achilles, tortoise,
            kit.text(stage, "Achilles", 25.0, Palette::blue(), v(-3.5, 1.8))?,
            kit.text(stage, "Tortoise", 25.0, Palette::gold(), v(3.5, 1.8))?,
            kit.text(stage, "Two lanes show position; time accelerated", 22.0, Palette::dim(), v(0.0, -2.9))?,
        ];
        fade_in_all(stage, &race, 0.4)?;
        for step in 1..=16 {
            let elapsed = catch_time * f64::from(step) / 16.0;
            let mut animations: Vec<Box<dyn Animation>> = Vec::new();
            for (mob, x, y) in [
                (achilles, track_left + fast * elapsed * scale, 0.4),
                (tortoise, track_left + (head + slow * elapsed) * scale, -0.4),
            ] {
                let animation = mob.animate().set_anim_args(AnimateArgs { run_time: Some(0.15), ..AnimateArgs::default() })?.move_to(v(x, y), ORIGIN)?;
                animations.push(stage.prepare(animation)?);
            }
            stage.play_prepared(animations)?;
        }
        let meet = kit.text(stage, "Same position at the catch", 27.0, Palette::green(), v(0.0, -1.3))?;
        fade_in_all(stage, &[meet], 0.4)?;
        stage.wait(0.5)?;
        let relative = kit.text(stage, &format!("relative speed: {} m/s", get("zeno.relative_speed")), 25.0, Palette::hero(), v(0.0, -2.1))?;
        fade_in_all(stage, &[relative], 0.4)?;
        kit.finish(stage)
    }
}
