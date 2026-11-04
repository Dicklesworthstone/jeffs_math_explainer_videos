//! Waiting for a phrase inside a narration line, for a sentence that carries more than one reveal.
//! `phrase_seconds`, `phrase_time` and `until_phrase` are the logic of
//! `logarithm_mysteries/src/chapters/unreachable.rs`, kept in one place for this crate.
use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Script};
use fmn::prelude::*;
use std::collections::HashMap;

/// The script the binary narrates: the file `main.rs` includes.
pub const SCRIPT: &str = include_str!("../script.tsv");

fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

/// When `phrase` begins, in seconds from the start of a line with this `text`:
/// the narrator's word mark if it has one for that word, otherwise a share by
/// characters of the measured sentence the phrase sits in. The share is a
/// provisional draft cue, not a heard boundary: fine sync waits for word marks
/// measured on the takes. A phrase the line lacks, or a narrator that splits the
/// line differently, is an error, never a guess.
fn phrase_seconds(
    text: &str,
    phrase: &str,
    marks: &HashMap<usize, f64>,
    starts: &[f64],
    len: f64,
) -> Result<f64, String> {
    let byte = text
        .find(phrase)
        .ok_or_else(|| format!("the line has no phrase {phrase:?}"))?;
    if let Some(&seconds) = marks.get(&byte) {
        return Ok(seconds);
    }
    let mut opens = vec![0];
    for (i, pair) in text.as_bytes().windows(2).enumerate() {
        if matches!(pair[0], b'.' | b'?' | b'!') && pair[1] == b' ' {
            opens.push(i + 2);
        }
    }
    if opens.len() != starts.len() {
        return Err(format!(
            "the line has {} sentences, but the narrator timed {}",
            opens.len(),
            starts.len()
        ));
    }
    let j = opens.iter().rposition(|&open| open <= byte).unwrap_or(0);
    let close = opens.get(j + 1).copied().unwrap_or(text.len());
    let (t0, t1) = (starts[j], starts.get(j + 1).copied().unwrap_or(len));
    let before = text[opens[j]..byte].chars().count() as f64;
    let whole = text[opens[j]..close].trim_end().chars().count().max(1) as f64;
    Ok(t0 + (t1 - t0) * before / whole)
}

/// The scene time `phrase` of line `id` begins, once the line has been said.
fn phrase_time(kit: &Kit, id: &str, phrase: &str) -> fmn::Result<Option<f64>> {
    let Some(n) = kit.narrator.as_ref() else {
        return Ok(None);
    };
    let starts = n.sentence_starts(id);
    let (Some(&first), Some(said), Some(len)) = (starts.first(), n.sentence_at(id, 0), n.length(id)) else {
        return Ok(None);
    };
    let script = Script::parse(SCRIPT).map_err(fail)?;
    let text = script
        .text(id)
        .ok_or_else(|| fail(format!("narration line {id} is not in script.tsv")))?;
    let seconds =
        phrase_seconds(text, phrase, &n.phrase_marks(id), &starts, len).map_err(|e| fail(format!("line {id}: {e}")))?;
    Ok(Some(said - first + seconds))
}

/// Wait until `phrase` of line `id` begins; nothing if that moment has passed.
pub fn until_phrase(kit: &Kit, stage: &mut Stage<'_>, id: &str, phrase: &str) -> fmn::Result<()> {
    let Some(at) = phrase_time(kit, id, phrase)? else {
        return Ok(());
    };
    let (now, fps) = (stage.scene().time().to_f64(), stage.scene().time().fps());
    if at > now {
        wait_frames(stage, frames_covering(at - now, fps))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::phrase_seconds;
    use std::collections::HashMap;

    const LINE: &str = "One two. Three four five.";
    const STARTS: [f64; 2] = [0.0, 2.0];

    #[test]
    fn a_word_mark_wins_over_the_share() {
        let marks = HashMap::from([(15, 2.5)]);
        assert_eq!(phrase_seconds(LINE, "four", &marks, &STARTS, 5.0), Ok(2.5));
    }
    #[test]
    fn a_mark_on_another_word_is_not_used() {
        let marks = HashMap::from([(9, 2.0)]);
        assert_eq!(phrase_seconds(LINE, "four", &marks, &STARTS, 5.0), Ok(3.125));
    }
    #[test]
    fn without_a_mark_the_phrase_takes_its_share_of_its_own_sentence() {
        let none = HashMap::new();
        assert_eq!(phrase_seconds(LINE, "two", &none, &STARTS, 5.0), Ok(1.0));
        assert_eq!(phrase_seconds(LINE, "four", &none, &STARTS, 5.0), Ok(3.125));
    }
    #[test]
    fn a_phrase_the_line_lacks_is_an_error() {
        assert!(phrase_seconds(LINE, "six", &HashMap::new(), &STARTS, 5.0).is_err());
    }
    #[test]
    fn a_narrator_that_splits_the_line_differently_is_an_error() {
        assert!(phrase_seconds(LINE, "four", &HashMap::new(), &[0.0], 5.0).is_err());
    }
}
