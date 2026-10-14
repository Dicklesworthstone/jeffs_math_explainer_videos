//! Last chapter — the takeaway (`02_1`) and credits (`02_2`).

use explainer_kit::{Kit, Timed, fade_all, play, v};
use fmn::prelude::*;

use crate::palette;

pub const TAKEAWAY: &str = "Every number here was computed, never typed.";
pub const CREDIT: &str = "franken_manim  ·  FrankenTTS";

pub struct Outro {
    pub kit: Kit,
}

impl SceneConstruct for Outro {
    fn name(&self) -> &str {
        "02_outro"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let takeaway = kit.text(stage, TAKEAWAY, 40.0, palette::ink(), v(0.0, 0.6))?;
        kit.say(stage, "02_1")?;
        play!(stage; write(stage.arena(), takeaway).rt(1.6));
        stage.wait(0.8)?;

        let credit = kit.text(stage, CREDIT, 30.0, palette::dim(), v(0.0, -0.8))?;
        kit.say(stage, "02_2")?;
        play!(stage; fade_in(stage.arena_mut(), credit, v(0.0, 0.2), 1.0)?.rt(1.2));
        stage.wait(1.0)?;

        fade_all(stage, 1.0)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chapters::glyphs;

    #[test]
    fn glyph_probe_covers_outro_strings() {
        for text in [TAKEAWAY, CREDIT] {
            assert!(glyphs::TEXT.contains(&text), "not probed: {text}");
        }
    }
}
