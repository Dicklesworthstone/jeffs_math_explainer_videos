//! Pictures that land on the voice's sentences. `kit.say` starts a line;
//! [`Spoken`] also knows where each of the line's sentences begins (its share
//! of the line by characters: a read keeps a near-even pace), so a chapter
//! waits for "sentence 3" instead of guessing seconds, and the picture stays
//! with the voice when a real take replaces the scratch voice. The line's
//! length is the narrator's: the take's, or the scratch voice's estimate.

use explainer_kit::Kit;
use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::storyboard::{Beat, ChapterBoard};
use fmn::prelude::*;

use crate::STORYBOARD;

/// The storyboard's chapter `key` (its beats carry the spoken words); an
/// error if the storyboard does not parse or has no such chapter (the
/// video's tests check both, through `storyboard::check_video`).
pub fn board(key: &str) -> fmn::Result<ChapterBoard> {
    let fail = |m: String| fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(m.into_boxed_str())));
    explainer_kit::storyboard::parse(STORYBOARD)
        .map_err(fail)?
        .into_iter()
        .find(|c| c.key == key)
        .ok_or_else(|| fail(format!("the storyboard has no chapter {key}")))
}

/// Where each sentence of `words` starts, as a share of the whole (by
/// characters); the first is 0.
pub fn sentence_shares(words: &str) -> Vec<f64> {
    let total = words.chars().count().max(1) as f64;
    let chars: Vec<char> = words.chars().collect();
    let mut shares = vec![0.0];
    for i in 1..chars.len() {
        if chars[i] == ' ' && matches!(chars[i - 1], '.' | '?' | '!') {
            shares.push((i + 1) as f64 / total);
        }
    }
    shares
}

/// Where `phrase` starts in `words`, as a share of the whole (by
/// characters); `None` when `words` does not hold it.
pub fn phrase_share(words: &str, phrase: &str) -> Option<f64> {
    let at = words.find(phrase)?;
    let total = words.chars().count().max(1) as f64;
    Some(words[..at].chars().count() as f64 / total)
}

/// A line being spoken: when it started, how long it lasts, and where its
/// sentences begin.
pub struct Spoken {
    id: String,
    start: f64,
    len: f64,
    words: String,
    shares: Vec<f64>,
}

impl Spoken {
    /// Say line `id` of `board` (after the previous line, as `kit.say`
    /// does), noting where its sentences fall.
    pub fn say(kit: &Kit, stage: &mut Stage<'_>, board: &ChapterBoard, id: &str) -> fmn::Result<Self> {
        kit.say(stage, id)?;
        let start = stage.scene().time().to_f64();
        let len = kit.narrator.as_ref().and_then(|n| n.length(id)).unwrap_or(0.0);
        let words = board
            .beats
            .iter()
            .find(|b| b.id == id)
            .and_then(Beat::spoken)
            .unwrap_or_default();
        Ok(Self {
            id: id.to_owned(),
            start,
            len,
            shares: sentence_shares(&words),
            words,
        })
    }

    /// Wait until sentence `k` (from 0) begins; no wait once it has.
    pub fn at(&self, stage: &mut Stage<'_>, k: usize) -> fmn::Result<()> {
        let share = self.shares.get(k).copied().unwrap_or(1.0);
        self.until(stage, self.start + share * self.len)
    }

    /// Wait until `share` of the line (0 to 1) has been spoken.
    pub fn at_share(&self, stage: &mut Stage<'_>, share: f64) -> fmn::Result<()> {
        self.until(stage, self.start + share.clamp(0.0, 1.0) * self.len)
    }

    /// Wait until the words `phrase` begin (their share of the line by
    /// characters, as for sentences); an error when the line does not hold
    /// them, so a reworded line cannot leave its picture behind silently.
    pub fn at_phrase(&self, stage: &mut Stage<'_>, phrase: &str) -> fmn::Result<()> {
        let share = phrase_share(&self.words, phrase).ok_or_else(|| {
            let m = format!("line {} does not say {phrase:?}", self.id);
            fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(m.into_boxed_str())))
        })?;
        self.at_share(stage, share)
    }

    fn until(&self, stage: &mut Stage<'_>, t: f64) -> fmn::Result<()> {
        let now = stage.scene().time().to_f64();
        if t > now + 1e-6 {
            let fps = stage.scene().time().fps();
            wait_frames(stage, frames_covering(t - now, fps))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentences_start_after_their_full_stops() {
        let s = sentence_shares("One two. Three? Four!");
        assert_eq!(s.len(), 3);
        assert_eq!(s[0], 0.0);
        assert!((s[1] - 9.0 / 21.0).abs() < 1e-12, "{s:?}");
        assert!(s.windows(2).all(|w| w[0] < w[1]));
        // a decimal point is not a sentence end
        assert_eq!(sentence_shares("About 4.82 on average.").len(), 1);
    }

    #[test]
    fn a_phrase_starts_at_its_share_of_the_line() {
        assert_eq!(phrase_share("One two. Three", "One"), Some(0.0));
        assert_eq!(phrase_share("One two. Three", "Three"), Some(9.0 / 14.0));
        // shares count characters, not bytes
        assert_eq!(phrase_share("÷ two", "two"), Some(2.0 / 5.0));
        assert_eq!(phrase_share("One two", "three"), None);
    }

    #[test]
    fn every_chapter_of_this_lane_has_its_words() {
        for key in [
            "13_the_byte_lottery",
            "14_the_price_of_density",
            "15_where_the_danger_lives",
            "16_parking_a_piece",
            "17_two_packets",
        ] {
            let b = board(key).expect("a chapter of the storyboard");
            assert!(b.beats.iter().filter_map(Beat::spoken).count() >= 5, "{key}");
        }
    }
}
