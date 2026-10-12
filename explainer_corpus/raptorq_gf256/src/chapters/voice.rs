//! Pictures that land on the voice's sentences. `kit.say` starts a line;
//! [`Spoken`] uses the narrator's assembled sentence offsets, so a chapter
//! waits for "sentence 3" instead of guessing seconds, and the picture stays
//! with the voice when a real take replaces the scratch voice. Phrase cues
//! interpolate within that sentence, not across differently paced reads.
//! Scratch narration supplies estimated offsets; phrase timing is still an
//! estimate, not word alignment.

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
    starts: Vec<f64>,
}

impl Spoken {
    /// Say line `id` of `board` (after the previous line, as `kit.say`
    /// does), using its measured (or scratch-estimated) sentence starts.
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
        let shares = sentence_shares(&words);
        let starts = if kit.narrator.is_some() {
            let starts = kit.sentence_starts(id);
            if starts.len() != shares.len() {
                let m = format!(
                    "line {id} has {} sentences but {} narrator sentence offsets",
                    shares.len(),
                    starts.len()
                );
                return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
                    m.into_boxed_str(),
                ))));
            }
            starts
        } else {
            vec![0.0; shares.len()]
        };
        Ok(Self {
            id: id.to_owned(),
            start,
            len,
            shares,
            starts,
            words,
        })
    }

    /// Wait until sentence `k` (from 0) begins; no wait once it has.
    pub fn at(&self, stage: &mut Stage<'_>, k: usize) -> fmn::Result<()> {
        let offset = self.starts.get(k).copied().unwrap_or(self.len);
        self.until(stage, self.start + offset)
    }

    /// Wait until `share` of the line (0 to 1) has been spoken.
    pub fn at_share(&self, stage: &mut Stage<'_>, share: f64) -> fmn::Result<()> {
        self.until(stage, self.start + share.clamp(0.0, 1.0) * self.len)
    }

    /// Estimate a phrase's start within its own sentence. Sentence boundaries
    /// come from the narrator; characters interpolate only between adjacent
    /// boundaries. An absent phrase fails rather than leaving a stale cue.
    pub fn at_phrase(&self, stage: &mut Stage<'_>, phrase: &str) -> fmn::Result<()> {
        let share = phrase_share(&self.words, phrase).ok_or_else(|| {
            let m = format!("line {} does not say {phrase:?}", self.id);
            fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(m.into_boxed_str())))
        })?;
        let k = self.shares.partition_point(|&s| s <= share).saturating_sub(1);
        let left = self.shares[k];
        let right = self.shares.get(k + 1).copied().unwrap_or(1.0);
        let from = self.starts[k];
        let to = self.starts.get(k + 1).copied().unwrap_or(self.len);
        let offset = from + (to - from) * (share - left) / (right - left);
        self.until(stage, self.start + offset)
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
    fn scene_cues_use_assembled_offsets_for_uneven_sentence_reads() {
        use explainer_kit::narration::{Narrator, Script};
        use explainer_score::{Stereo, wav};

        // Real WAV/report loading and native stage timing, using a private
        // silence fixture. This does not generate or qualify spoken audio.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("gf256_sentence_cues_{}_{nanos}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        wav::write(&dir.join("01_1.wav"), &Stereo::silence(72_000), wav::Format::F32).unwrap();
        let report = r#"{"meta":{"rate":48000},"lines":{"01_1":[
            {"sentence":"One.","out_samples":48000,"gap_after_samples":12000},
            {"sentence":"Two.","out_samples":12000,"gap_after_samples":0}]}}"#;
        std::fs::write(dir.join("report.json"), report).unwrap();
        let script = Script::parse("01_1\tOne. Two.\n").unwrap();
        let narrator = Narrator::load(&script, &dir, 0.0).unwrap();
        assert_eq!(narrator.sentence_starts("01_1"), [0.0, 1.25]);
        let kit = Kit::new().unwrap().with_narrator(narrator);
        let board = explainer_kit::storyboard::parse(
            "## 01_test\n| id | words | picture | features | hold/listen |\n\
             |---|---|---|---|---|\n| 01_1 | One. Two. | | | |\n",
        )
        .unwrap()
        .remove(0);
        struct Probe {
            kit: Kit,
            board: ChapterBoard,
        }
        impl SceneConstruct for Probe {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                let line = Spoken::say(&self.kit, stage, &self.board, "01_1")?;
                let origin = stage.scene().time().frames();
                line.at(stage, 1)?;
                let t = stage.scene().time();
                assert_eq!(
                    t.frames() - origin,
                    75,
                    "second sentence starts at 1.25 s, not its text share"
                );
                line.at_phrase(stage, "Two")?;
                assert_eq!(
                    stage.scene().time().frames() - origin,
                    75,
                    "sentence-opening phrase is exact"
                );
                line.at_phrase(stage, "wo")?;
                assert_eq!(
                    stage.scene().time().frames() - origin,
                    79,
                    "phrase interpolates inside sentence two"
                );
                line.at(stage, 0)?;
                assert_eq!(stage.scene().time().frames() - origin, 79, "past cues never rewind");
                assert!(line.at_phrase(stage, "Missing").is_err(), "stale phrase must refuse");
                Ok(())
            }
        }
        let config = fmn::scene::RuntimeConfig {
            fps: 60,
            ..Default::default()
        };
        fmn::run_scene(&mut Probe { kit, board }, config, 7, &mut fmn::scene::NullSceneSink)
            .expect("real sentence-cue scene");
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
