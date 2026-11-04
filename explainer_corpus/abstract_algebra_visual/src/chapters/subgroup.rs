//! Native square-rotation subgroup scene; build and picture review pending.
//! Proposed cues 03_1/03_2 follow the two retained source paragraphs. Their
//! boundaries in the original ElevenLabs recording remain unreviewed.
use crate::model::{Symmetry, all};
use explainer_kit::{Kit, Palette, Timed, hex, ink_style, rect, stroke_style, v};
use fmn::prelude::*;

pub struct RotationSubgroup {
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

fn rotation_name(turns: u8) -> &'static str {
    match turns {
        0 => "e",
        1 => "r",
        2 => "r²",
        3 => "r³",
        _ => unreachable!("model normalizes turns"),
    }
}

impl SceneConstruct for RotationSubgroup {
    fn name(&self) -> &str {
        "03_rotation_subgroup"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let mut rotations = Vec::new();
        let mut current = Symmetry::ID;
        loop {
            rotations.push(current);
            current = Symmetry::R.compose(current);
            if current == Symmetry::ID {
                break;
            }
            if rotations.len() > all().len() {
                return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("rotation did not return")));
            }
        }
        let corners = [(-1, -1), (1, -1), (1, 1), (-1, 1)];
        let colors = [Palette::blue(), Palette::gold(), Palette::green(), Palette::hero()];

        kit.say(stage, "03_1")?;
        let heading = kit.text(stage, "Keep only the rotations", 37.0, Palette::ink(), v(0.0, 2.8))?;
        appear(stage, &[heading])?;
        let mut pictures = Vec::new();
        for (index, action) in rotations.iter().enumerate() {
            let center = (index as f64 - (rotations.len() - 1) as f64 / 2.0) * 3.0;
            pictures.push(stage.arena_mut().add(rect(
                v(center - 0.85, -0.85), 1.7, 1.7, stroke_style(Palette::dim(), 3.0),
            )?));
            for (corner_index, point) in corners.iter().enumerate() {
                let (x, y) = action.apply(*point);
                let position = v(center + f64::from(x) * 0.62, f64::from(y) * 0.62);
                let dot = Circle::new().radius(0.16).style(ink_style(colors[corner_index])).build().moved_to(position);
                pictures.push(stage.arena_mut().add(dot));
                pictures.push(kit.text(stage, &(corner_index + 1).to_string(), 16.0, hex("#0E1116"), position)?);
            }
            pictures.push(kit.text(stage, rotation_name(action.turns), 27.0, Palette::blue(), v(center, -1.35))?);
        }
        appear(stage, &pictures)?;
        let question = kit.text(stage, "How many quarter turns return to the start?", 26.0, Palette::hero(), v(0.0, -2.1))?;
        appear(stage, &[question])?;
        kit.hold(stage)?;
        stage.wait(3.0)?;

        kit.say(stage, "03_2")?;
        let fade = fade_out(stage.arena_mut(), question, ORIGIN, 1.0)?;
        let fade = stage.prepare(fade.rt(0.4))?;
        stage.play_prepared(vec![fade])?;
        let cycle = kit.text(stage, "e → r → r² → r³ → e", 29.0, Palette::hero(), v(0.0, -2.05))?;
        appear(stage, &[cycle])?;
        kit.until_sentence(stage, "03_2", 2)?;
        let element_order = kit.text(stage, &format!("order of r: {}", rotations.len()), 29.0, Palette::green(), v(-2.4, -2.85))?;
        appear(stage, &[element_order])?;
        kit.until_sentence(stage, "03_2", 3)?;
        let group_order = kit.text(stage, &format!("group order: {}", all().len()), 29.0, Palette::blue(), v(2.4, -2.85))?;
        appear(stage, &[group_order])?;
        kit.finish(stage)
    }
}
