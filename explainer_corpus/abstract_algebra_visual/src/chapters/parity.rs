//! Standalone parity example; source and take still need independent review.
use explainer_kit::{Kit, Palette, Timed, v};
use fmn::prelude::*;

pub struct Parity { pub kit: Kit }
fn fade(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}
// Word marks are provisional until the full take is heard; scratch falls back
// to the sentence boundary rather than a time from a particular recording.
fn until_received_value(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let Some(narrator) = &kit.narrator else { return Ok(()); };
    let text = include_str!("../../script.tsv").lines()
        .find_map(|line| line.split_once('\t').filter(|(id, _)| *id == "09_1").map(|(_, text)| text))
        .expect("parity cue in script");
    let byte = text.find("one zero zero").expect("received value in script");
    let target = narrator.sentence_at("09_1", 0)
        .and_then(|start| narrator.phrase_marks("09_1").get(&byte).map(|offset| start + offset));
    if let Some(target) = target {
        let now = stage.scene().time().to_f64();
        if target > now {
            explainer_kit::narration::wait_frames(stage, explainer_kit::narration::frames_covering(target - now, stage.scene().time().fps()))?;
        }
        Ok(())
    } else {
        kit.until_sentence(stage, "09_1", 2)
    }
}

impl SceneConstruct for Parity {
    fn name(&self) -> &str { "09_parity" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "09_1")?;
        // The displayed strings and counts come from the same two bit patterns.
        let sent: u8 = 0b101;
        let received = sent ^ 0b001;
        let title = kit.text(stage, "A simple parity check", 36.0, Palette::ink(), v(0.0, 2.55))?;
        let rule = kit.text(stage, "Valid: an even number of ones", 30.0, Palette::green(), v(0.0, 1.65))?;
        fade(stage, &[title, rule])?;
        // Scratch sentence timings are estimates until actual listening.
        kit.until_sentence(stage, "09_1", 1)?;
        let send = kit.text(stage, &format!("Send {sent:03b}: {} ones", sent.count_ones()), 32.0, Palette::blue(), v(0.0, 0.65))?;
        fade(stage, &[send])?;
        until_received_value(&kit, stage)?;
        let receive = kit.text(stage, &format!("Receive {received:03b}: {} one", received.count_ones()), 32.0, Palette::hero(), v(0.0, -0.25))?;
        fade(stage, &[receive])?;
        kit.until_sentence(stage, "09_1", 3)?;
        let error = kit.text(stage, "Odd count: error detected", 30.0, Palette::hero(), v(0.0, -1.15))?;
        fade(stage, &[error])?;
        kit.until_sentence(stage, "09_1", 4)?;
        let caution = kit.text(stage, "Two flips can go unnoticed", 28.0, Palette::gold(), v(0.0, -2.15))?;
        fade(stage, &[caution])?;
        kit.finish(stage)
    }
}
