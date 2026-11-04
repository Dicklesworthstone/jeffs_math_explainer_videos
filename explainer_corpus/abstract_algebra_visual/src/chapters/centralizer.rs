//! Centralizer scene draft for native FrankenManim and `explainer_kit`.
//! Build, render, synchronization, and independent review are pending.
//! Requires the existing `crate::model` square-symmetry model. Cue 04_1 is
//! the first retained centralizer paragraph; 04_2 is the second. A script
//! boundary is not yet an approved cut in either retained full recording.
use crate::model::{Symmetry, all};
use explainer_kit::{Kit, Palette, Timed, hex, ink_style, rect, stroke_style, v};
use fmn::prelude::*;

pub struct Centralizer {
    pub kit: Kit,
}

fn appear(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn disappear(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn label(action: Symmetry) -> String {
    match (action.turns, action.reflected) {
        (0, false) => "identity".into(),
        (1, false) => "quarter turn".into(),
        (2, false) => "half turn".into(),
        (3, false) => "three-quarter turn".into(),
        (0, true) => "horizontal flip".into(),
        (1, true) => "diagonal flip A".into(),
        (2, true) => "vertical flip".into(),
        (3, true) => "diagonal flip B".into(),
        _ => unreachable!("model normalizes every symmetry to zero through three turns"),
    }
}

impl SceneConstruct for Centralizer {
    fn name(&self) -> &str {
        "04_centralizer"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let flip = Symmetry::S;
        let actions = all();
        kit.say(stage, "04_1")?;
        let title = kit.text(
            stage,
            "Which moves commute with the horizontal flip?",
            32.0,
            Palette::ink(),
            v(0.0, 2.8),
        )?;
        let question = kit.text(
            stage,
            "Predict: does a quarter turn commute?",
            27.0,
            Palette::hero(),
            v(0.0, -2.0),
        )?;
        appear(stage, &[title, question])?;
        let mut names = Vec::new();
        for (index, action) in actions.iter().enumerate() {
            let x = (index % 2) as f64 * 5.4 - 2.7;
            let y = 1.7 - (index / 2) as f64 * 0.8;
            names.push(kit.text(stage, &label(*action), 24.0, Palette::ink(), v(x, y))?);
        }
        appear(stage, &names)?;
        kit.hold(stage)?;
        stage.wait(3.0)?;

        kit.say(stage, "04_2")?;
        let mut results = Vec::new();
        for (index, action) in actions.iter().enumerate() {
            let x = (index % 2) as f64 * 5.4 - 2.7;
            let y = 1.7 - (index / 2) as f64 * 0.8 - 0.31;
            let commutes = action.compose(flip) == flip.compose(*action);
            results.push(kit.text(
                stage,
                if commutes { "same result" } else { "different result" },
                22.0,
                if commutes { Palette::green() } else { Palette::gold() },
                v(x, y),
            )?);
        }
        appear(stage, &results)?;
        let count = actions.iter().filter(|action| action.compose(flip) == flip.compose(**action)).count();
        let summary = kit.text(
            stage,
            &format!("{count} moves commute with the chosen flip"),
            25.0,
            Palette::green(),
            v(0.0, -2.9),
        )?;
        appear(stage, &[summary])?;
        stage.wait(2.0)?;
        // Hold the commuting examples until the quarter-turn sentence;
        // scratch narration interpolates this boundary, real audio must be heard.
        kit.until_sentence(stage, "04_2", 2)?;
        let mut old = vec![title, question, summary];
        old.extend(names);
        old.extend(results);
        disappear(stage, &old)?;

        // `a.compose(b)` means b acts first. Draw both orders directly from
        // the same square model used for the centralizer calculation.
        let orders = [Symmetry::R.compose(flip), flip.compose(Symmetry::R)];
        let descriptions = ["Flip, then turn", "Turn, then flip"];
        let corners = [(-1, -1), (1, -1), (1, 1), (-1, 1)];
        let colors = [Palette::blue(), Palette::gold(), Palette::green(), Palette::hero()];
        let mut comparison = vec![kit.text(stage, "Order changes the result", 35.0, Palette::ink(), v(0.0, 2.8))?];
        for (index, action) in orders.iter().enumerate() {
            let center = if index == 0 { -2.6 } else { 2.6 };
            comparison.push(stage.arena_mut().add(rect(v(center - 1.0, -0.9), 2.0, 2.0, stroke_style(Palette::dim(), 3.0))?));
            for (corner_index, point) in corners.iter().enumerate() {
                let (x, y) = action.apply(*point);
                let position = v(center + f64::from(x) * 0.74, 0.1 + f64::from(y) * 0.74);
                let dot = Circle::new().radius(0.18).style(ink_style(colors[corner_index])).build().moved_to(position);
                comparison.push(stage.arena_mut().add(dot));
                comparison.push(kit.text(stage, &(corner_index + 1).to_string(), 17.0, hex("#0E1116"), position)?);
            }
            comparison.push(kit.text(stage, descriptions[index], 25.0, Palette::ink(), v(center, -1.5))?);
        }
        comparison.push(kit.text(stage, "The quarter turn is outside this centralizer", 24.0, Palette::hero(), v(0.0, -2.5))?);
        appear(stage, &comparison)?;
        kit.finish(stage)
    }
}
