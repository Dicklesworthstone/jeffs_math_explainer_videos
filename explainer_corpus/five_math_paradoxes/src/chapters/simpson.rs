//! Draft native fmn Simpson chapter; not installed or built yet.
use explainer_kit::{Kit, Palette, Timed, v};
use fmn::prelude::*;

pub struct Simpson {
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

impl SceneConstruct for Simpson {
    fn name(&self) -> &str {
        "03_simpson"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let facts = crate::facts::facts().map_err(|_| {
            fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("Simpson facts failed"))
        })?;
        let get = |key: &str| facts.get(key).unwrap_or("?");
        kit.say(stage, "03_1")?;
        let heading = [
            kit.text(stage, "Inspection pass rates", 38.0, Palette::ink(), v(0.0, 2.9))?,
            kit.text(stage, "Machine A", 28.0, Palette::blue(), v(-1.8, 2.1))?,
            kit.text(stage, "Machine B", 28.0, Palette::gold(), v(2.5, 2.1))?,
        ];
        fade_in_all(stage, &heading, 0.5)?;
        kit.until_sentence(stage, "03_1", 1)?;
        let easy = [
            kit.text(stage, "Easy parts", 26.0, Palette::dim(), v(-5.0, 1.1))?,
            kit.text(stage, get("simpson.easy.a"), 30.0, Palette::blue(), v(-1.8, 1.1))?,
            kit.text(stage, get("simpson.easy.b"), 30.0, Palette::gold(), v(2.5, 1.1))?,
        ];
        fade_in_all(stage, &easy, 0.5)?;

        kit.say(stage, "03_2")?;
        let hard = [
            kit.text(stage, "Hard parts", 26.0, Palette::dim(), v(-5.0, 0.0))?,
            kit.text(stage, get("simpson.hard.a"), 30.0, Palette::blue(), v(-1.8, 0.0))?,
            kit.text(stage, get("simpson.hard.b"), 30.0, Palette::gold(), v(2.5, 0.0))?,
        ];
        fade_in_all(stage, &hard, 0.5)?;
        kit.hold(stage)?;
        let question = kit.text(stage, "Which machine leads overall?", 28.0, Palette::hero(), v(0.0, -1.2))?;
        fade_in_all(stage, &[question], 0.4)?;
        stage.wait(3.0)?;
        fade_out_all(stage, &[question], 0.4)?;

        kit.say(stage, "03_3")?;
        kit.until_sentence(stage, "03_3", 1)?;
        let combined = [
            kit.text(stage, "All parts", 26.0, Palette::dim(), v(-5.0, -1.2))?,
            kit.text(stage, get("simpson.total.a"), 30.0, Palette::blue(), v(-1.8, -1.2))?,
            kit.text(stage, get("simpson.total.b"), 30.0, Palette::gold(), v(2.5, -1.2))?,
        ];
        fade_in_all(stage, &combined, 0.5)?;

        // These counts explain the reversal; derive them from the same groups
        // as the pass rates so the picture cannot silently drift from them.
        let (a, b) = five_math_paradoxes::inspection_groups();
        let (Some(a_total), Some(b_total)) = (five_math_paradoxes::combine(&a), five_math_paradoxes::combine(&b)) else {
            return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("invalid inspection groups")));
        };
        kit.until_sentence(stage, "03_3", 4)?;
        let weights = [
            kit.text(stage, &format!("A: {} of {} hard", a[1].total, a_total.total), 30.0, Palette::blue(), v(-1.8, -2.05))?,
            kit.text(stage, &format!("B: {} of {} easy", b[0].total, b_total.total), 30.0, Palette::gold(), v(2.5, -2.05))?,
        ];
        fade_in_all(stage, &weights, 0.5)?;

        kit.say(stage, "03_4")?;
        let caution = kit.text(stage, "Different group weights; no causal conclusion by itself", 23.0, Palette::dim(), v(0.0, -2.8))?;
        fade_in_all(stage, &[caution], 0.5)?;
        kit.finish(stage)
    }
}
