//! The animatic: a stand-in for a chapter until its real scene is built. It
//! speaks each storyboard line on time (with the real narration, or scratch
//! timing) over a card describing the planned picture, keeps the authored
//! prediction pauses, holds and listen windows, and ends like a real chapter
//! (`Kit::finish`), so drafts, the score's timelines, the master and the
//! captions run end to end from the first day of a video.

use fmn::prelude::*;

use crate::kit::Kit;
use crate::palette::Palette;
use crate::play::{Timed, fade_all};
use crate::plot::v;
use crate::storyboard::{Beat, ChapterBoard};

/// An authored music breath (a `hold` beat), seconds.
pub const HOLD_S: f64 = 2.5;
/// How long a silent card (a title) stays when it has no hold of its own.
pub const SILENT_S: f64 = 2.5;
/// Characters per line of the picture card.
const CARD_CHARS: usize = 58;
/// Lines of the picture card (the rest is cut with an ellipsis).
const CARD_LINES: usize = 7;

/// One chapter as an animatic.
pub struct Animatic {
    pub kit: Kit,
    pub key: String,
    /// The chapter's storyboard, or why it could not be read (a registry or
    /// storyboard error: `construct` reports it; every video's tests catch it
    /// first, through `storyboard::check_video`).
    pub board: Result<ChapterBoard, String>,
}

impl Animatic {
    /// The animatic of chapter `key` from the storyboard text.
    pub fn new(kit: Kit, storyboard: &str, key: &str) -> Self {
        let board = crate::storyboard::parse(storyboard).and_then(|chapters| {
            chapters
                .into_iter()
                .find(|c| c.key == key)
                .ok_or_else(|| format!("the storyboard has no chapter {key}"))
        });
        Self {
            kit,
            key: key.to_owned(),
            board,
        }
    }
}

/// Greedy word wrap.
pub fn wrap(text: &str, width: usize, max_lines: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(l) if l.chars().count() + 1 + word.chars().count() <= width => {
                l.push(' ');
                l.push_str(word);
            }
            _ => lines.push(word.to_owned()),
        }
    }
    if lines.len() > max_lines {
        lines.truncate(max_lines);
        if let Some(l) = lines.last_mut() {
            l.push_str(" …");
        }
    }
    lines
}

/// The card's text: the storyboard's markdown emphasis and escapes removed.
fn plain(s: &str) -> String {
    s.replace("**", "").replace('`', "").replace("\\|", "|")
}

impl SceneConstruct for Animatic {
    fn name(&self) -> &str {
        &self.key
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let board = self
            .board
            .clone()
            .map_err(|m| fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(m.into_boxed_str()))))?;
        let kit = self.kit.clone();
        let header = format!("{}  {}", board.key, board.title);
        let head = kit.text(stage, &header, 26.0, Palette::dim(), v(0.0, 3.55))?;
        stage.add_to_scene(head)?;
        let mut shown: Vec<Mob> = Vec::new();
        for beat in &board.beats {
            let card = self.card(stage, beat)?;
            // the old card stays until the previous line has finished: `say`
            // (or `hold`, for a silent beat) waits for it, then the two cards
            // cross-fade in one move
            let spoken = beat.spoken().is_some();
            if spoken {
                kit.say(stage, &beat.id)?;
            } else {
                kit.hold(stage)?;
            }
            let mut anims: Vec<Box<dyn Animation>> = Vec::new();
            for m in shown.drain(..) {
                let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
                anims.push(stage.prepare(a.rt(0.4))?);
            }
            for &m in &card {
                let a = fade_in(stage.arena_mut(), m, v(0.0, -0.15), 1.0)?;
                anims.push(stage.prepare(a.rt(0.6))?);
            }
            stage.play_prepared(anims)?;
            shown = card;
            if beat.predict() {
                shown.push(crate::props::predict_pause(&kit, stage, v(5.6, -2.6))?);
            }
            if let Some(s) = beat.listen_s() {
                kit.listen(stage, s, &format!("{}:{}", board.key, beat.id))?;
            }
            if beat.holds() {
                kit.music_breath(stage, HOLD_S)?;
            }
            if !spoken && !beat.holds() && beat.listen_s().is_none() {
                crate::narration::wait_frames(
                    stage,
                    crate::narration::frames_covering(SILENT_S, stage.scene().time().fps()),
                )?;
            }
        }
        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

impl Animatic {
    /// The beat's card: its id, the planned picture (wrapped), and its
    /// features in small type.
    fn card(&self, stage: &mut Stage<'_>, beat: &Beat) -> fmn::Result<Vec<Mob>> {
        let kit = &self.kit;
        let mut mobs = Vec::new();
        mobs.push(kit.text_left(stage, &beat.id, 30.0, Palette::hero(), v(-6.6, 2.7))?);
        let lines = wrap(&plain(&beat.picture), CARD_CHARS, CARD_LINES);
        let n = lines.len() as f64;
        for (i, l) in lines.iter().enumerate() {
            let y = 0.45 * (n - 1.0) / 2.0 - 0.45 * i as f64 + 0.3;
            mobs.push(kit.text(stage, l, 34.0, Palette::ink(), v(0.0, y))?);
        }
        if !beat.features.is_empty() && beat.features != "—" {
            let f = wrap(&plain(&beat.features), 80, 1).join(" ");
            mobs.push(kit.text(stage, &f, 24.0, Palette::dim(), v(0.0, -2.75))?);
        }
        Ok(mobs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_card_wraps_and_cuts_long_pictures() {
        assert_eq!(wrap("a bb ccc dddd", 6, 9), ["a bb", "ccc", "dddd"]);
        let w = wrap(&"word ".repeat(100), 20, 3);
        assert_eq!(w.len(), 3);
        assert!(w[2].ends_with(" …"));
        assert!(w.iter().all(|l| l.chars().count() <= 22));
        assert_eq!(plain("**Title** `x` a \\| b"), "Title x a | b");
    }
}
