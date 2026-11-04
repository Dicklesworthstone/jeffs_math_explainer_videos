//! Native title and closing scenes; timing and pictures remain unreviewed.
use explainer_kit::{Kit, Palette, Timed, rect, stroke_style, v};
use fmn::prelude::*;

pub struct Opening { pub kit: Kit }
pub struct Closing { pub kit: Kit }

fn appear(stage: &mut Stage<'_>, mob: Mob) -> fmn::Result<()> {
    let animation = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
    let animation = stage.prepare(animation.rt(0.6))?;
    stage.play_prepared(vec![animation])?;
    Ok(())
}

impl SceneConstruct for Opening {
    fn name(&self) -> &str { "00_opening" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "00_1")?;
        let title = kit.text(stage, "Five paradoxes, made clear", 40.0, Palette::ink(), v(0.0, 2.7))?;
        appear(stage, title)?;
        // Three small visual questions foreshadow the later detailed scenes.
        let mut doors = Vec::new();
        for index in 0..3 {
            let x = -4.2 + (index as f64 - 1.0) * 0.7;
            doors.push(stage.arena_mut().add(rect(v(x - 0.27, -0.3), 0.54, 1.15, stroke_style(Palette::blue(), 3.0))?));
        }
        doors.push(kit.text(stage, "Switch doors?", 25.0, Palette::hero(), v(-4.2, -0.9))?);
        let mut animations: Vec<Box<dyn Animation>> = Vec::new();
        for mob in doors {
            let animation = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
            animations.push(stage.prepare(animation.rt(0.5))?);
        }
        stage.play_prepared(animations)?;
        // Sentence positions are scratch/ASR estimates until actual listening.
        kit.until_sentence(stage, "00_1", 1)?;
        for (text, y) in [("A leads in each group", 0.5), ("B leads overall", -0.1), ("How can both be true?", -0.9)] {
            let item = kit.text(stage, text, 24.0, Palette::gold(), v(0.0, y))?;
            appear(stage, item)?;
        }
        kit.until_sentence(stage, "00_1", 2)?;
        for (text, y) in [("Infinitely many stages", 0.5), ("A finite catch time", -0.1), ("How can that add up?", -0.9)] {
            let item = kit.text(stage, text, 24.0, Palette::green(), v(4.1, y))?;
            appear(stage, item)?;
        }
        kit.until_sentence(stage, "00_1", 5)?;
        let promise = kit.text(stage, "Five surprises, five explanations", 28.0, Palette::ink(), v(0.0, -2.3))?;
        appear(stage, promise)?;
        kit.finish(stage)
    }
}

// Marks are provisional until a listener checks the take. For scratch narration,
// fall back to the phrase's share of the first sentence instead of fixed seconds.
fn until_closing_phrase(kit: &Kit, stage: &mut Stage<'_>, phrase: &str) -> fmn::Result<()> {
    let Some(narrator) = &kit.narrator else { return Ok(()); };
    let text = include_str!("../../script.tsv").lines()
        .find_map(|line| line.split_once('\t').filter(|(id, _)| *id == "06_1").map(|(_, text)| text))
        .expect("closing cue in script");
    let byte = text.find(phrase).expect("closing phrase in script");
    let Some(start) = narrator.sentence_at("06_1", 0) else { return Ok(()); };
    let second = narrator.sentence_starts("06_1").get(1).copied()
        .or_else(|| narrator.length("06_1")).unwrap_or(0.0);
    let first_sentence_bytes = text.find("? ").map_or(text.len(), |i| i + 1);
    let relative = narrator.phrase_marks("06_1").get(&byte).copied()
        .unwrap_or(second * byte as f64 / first_sentence_bytes as f64);
    let now = stage.scene().time().to_f64();
    if start + relative > now {
        explainer_kit::narration::wait_frames(stage, explainer_kit::narration::frames_covering(start + relative - now, stage.scene().time().fps()))?;
    }
    Ok(())
}

impl SceneConstruct for Closing {
    fn name(&self) -> &str { "06_closing" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "06_1")?;
        let title = kit.text(stage, "One useful question can change the picture", 34.0, Palette::hero(), v(0.0, 2.5))?;
        appear(stage, title)?;
        for (i, (phrase, label)) in [
            ("what does the host know", "Host rules"),
            ("which pairs count", "Matching pairs"),
            ("how are averages weighted", "Weighted averages"),
            ("what happens to the total time", "Infinite stages"),
            ("can every guest be matched", "Matching rooms"),
        ].iter().enumerate() {
            until_closing_phrase(&kit, stage, phrase)?;
            let item = kit.text(stage, label, 26.0, Palette::ink(), v(0.0, 1.2 - i as f64 * 0.75))?;
            appear(stage, item)?;
        }
        kit.finish(stage)
    }
}
