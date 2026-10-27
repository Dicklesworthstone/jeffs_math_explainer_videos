//! Captions (plan §13): SRT and WebVTT for a whole video, sentence by
//! sentence, saying what was actually spoken.
//!
//! The narration skill's `report.json` lists each line's sentences as they
//! are displayed (its `sentence`; respellings live only in `tts`) and the
//! span each was spoken over in its own take (`end − start − pause_cut_s`).
//! Current reports record assembled sample counts and gaps; those measured
//! spans drive captions directly. For older reports without them, each
//! sentence is found in the line's audio: the voice-activity detector's
//! speech regions, grouped so the groups' lengths best fit the sentences'.
//! The chapters' timelines say where each line starts.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::io;
use std::path::Path;

use explainer_score::duck;
use explainer_score::resample::resample;
use explainer_score::timeline::ChapterTimeline;
use explainer_score::{SAMPLE_RATE, Stereo, wav};

/// Characters per caption line, and lines per caption.
pub const LINE_CHARS: usize = 42;
pub const MAX_LINES: usize = 2;

/// One sentence of a narration line: its display text and how long it was
/// spoken (seconds).
#[derive(Clone, Debug, PartialEq)]
pub struct Spoken {
    pub text: String,
    pub seconds: f64,
    /// Recorded start, end of source samples, and end of the following gap,
    /// in seconds from this cue's start. Absent in legacy reports.
    pub recorded: Option<(f64, f64, f64)>,
    /// Bound authored insertions, with cue-relative sample times. These are
    /// editorial boundaries, not measured phonetic onsets.
    pub pauses: Vec<CaptionPause>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CaptionPause {
    /// Number of whitespace-separated display words before this pause.
    pub after_words: usize,
    pub start: f64,
    pub end: f64,
}

/// One caption on screen.
#[derive(Clone, Debug, PartialEq)]
pub struct Caption {
    pub start: f64,
    pub end: f64,
    pub lines: Vec<String>,
}

fn bad(m: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, m.into())
}

/// Each line's sentences, from the narration skill's `report.json`.
pub fn read_report(path: &Path) -> io::Result<HashMap<String, Vec<Spoken>>> {
    parse_report(&std::fs::read_to_string(path)?)
}

fn parse_report(json: &str) -> io::Result<HashMap<String, Vec<Spoken>>> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| bad(format!("report.json: {e}")))?;
    // the narration skill writes {"meta": {..}, "lines": {id: [..]}}; a shipped
    // narration (Hoeffding v5) is the bare {id: [..]}
    let lines = match v.get("lines").and_then(serde_json::Value::as_object) {
        Some(lines) => lines,
        None => v
            .as_object()
            .ok_or_else(|| bad("report.json: not an object of lines"))?,
    };
    let mut out = HashMap::new();
    let mut used_directions = std::collections::HashSet::new();
    for (id, sentences) in lines {
        let list = sentences
            .as_array()
            .ok_or_else(|| bad(format!("report.json: line {id} is not a list")))?;
        let mut spoken = Vec::new();
        let measured = list
            .iter()
            .any(|s| s.get("out_samples").is_some() || s.get("gap_after_samples").is_some());
        let rate = if measured {
            v.pointer("/meta/rate")
                .and_then(serde_json::Value::as_u64)
                .filter(|&r| r > 0)
                .ok_or_else(|| bad("sample-timed report requires a positive integer meta.rate"))? as f64
        } else {
            1.0
        };
        let mut cursor = 0u64;
        let mut word_offset = 0usize;
        for (k, s) in list.iter().enumerate() {
            let field = |name: &str| s.get(name).ok_or_else(|| bad(format!("{id} sentence {k}: no {name}")));
            let num = |name: &str| {
                field(name)?
                    .as_f64()
                    .ok_or_else(|| bad(format!("{id} sentence {k}: {name}")))
            };
            let text = field("sentence")?
                .as_str()
                .ok_or_else(|| bad(format!("{id} sentence {k}: sentence")))?;
            let seconds = num("end")? - num("start")? - num("pause_cut_s")?;
            if text.trim().is_empty() || !seconds.is_finite() || seconds <= 0.0 {
                return Err(bad(format!("{id} sentence {k}: empty or {seconds} s long")));
            }
            let sentence_start = cursor;
            let recorded = if measured {
                let samples = |name: &str| {
                    field(name)?
                        .as_u64()
                        .ok_or_else(|| bad(format!("{id} sentence {k}: invalid {name}")))
                };
                let count = samples("out_samples")?;
                if count == 0 {
                    return Err(bad(format!("{id} sentence {k}: zero samples")));
                }
                let end = cursor
                    .checked_add(count)
                    .ok_or_else(|| bad("sentence sample count overflow"))?;
                let after = end
                    .checked_add(samples("gap_after_samples")?)
                    .ok_or_else(|| bad("sentence gap overflow"))?;
                let span = (cursor as f64 / rate, end as f64 / rate, after as f64 / rate);
                cursor = after;
                Some(span)
            } else {
                None
            };
            let pauses = report_pauses(&v, id, s, word_offset, sentence_start, rate, measured)?;
            if let Some(items) = s.get("intra_sentence_pauses").and_then(serde_json::Value::as_array) {
                for item in items {
                    if !used_directions.insert(item["override_index"].as_u64().unwrap()) {
                        return Err(bad("duplicate intra-sentence direction"));
                    }
                }
            }
            word_offset += direction_words(text).len();
            spoken.push(Spoken {
                text: text.trim().to_owned(),
                seconds,
                recorded,
                pauses,
            });
        }
        out.insert(id.clone(), spoken);
    }
    Ok(out)
}

/// Match coherent_narration.py's ASCII words, including one apostrophe suffix.
/// Hyphens separate contract words but remain intact in displayed captions.
fn direction_words(text: &str) -> Vec<String> {
    let mut chars = text.chars().peekable();
    let mut words = Vec::new();
    while let Some(c) = chars.next() {
        if !c.is_ascii_alphanumeric() {
            continue;
        }
        let mut word = c.to_ascii_lowercase().to_string();
        while chars.peek().is_some_and(char::is_ascii_alphanumeric) {
            word.push(chars.next().unwrap().to_ascii_lowercase());
        }
        if chars.peek().is_some_and(|c| matches!(c, '\'' | '’')) {
            chars.next();
            if chars.peek().is_some_and(char::is_ascii_alphabetic) {
                word.push('\'');
                while chars.peek().is_some_and(char::is_ascii_alphabetic) {
                    word.push(chars.next().unwrap().to_ascii_lowercase());
                }
            }
        }
        words.push(word);
    }
    words
}

fn report_pauses(
    report: &serde_json::Value,
    cue: &str,
    row: &serde_json::Value,
    word_offset: usize,
    cue_sample: u64,
    rate: f64,
    measured: bool,
) -> io::Result<Vec<CaptionPause>> {
    let Some(value) = row.get("intra_sentence_pauses") else {
        return Ok(Vec::new());
    };
    let items = value
        .as_array()
        .ok_or_else(|| bad("intra_sentence_pauses must be an array"))?;
    if items.is_empty() {
        return Ok(Vec::new());
    }
    if !measured {
        return Err(bad("intra-sentence pauses require sample timing"));
    }
    let integer =
        |v: &serde_json::Value, key: &str| v[key].as_u64().ok_or_else(|| bad(format!("pause: invalid {key}")));
    let source_in = integer(row, "source_in_sample")?;
    let source_out = integer(row, "source_out_sample")?;
    let text = row["sentence"].as_str().ok_or_else(|| bad("pause: missing sentence"))?;
    let words: Vec<_> = text.split_whitespace().collect();
    let tokens = direction_words(text);
    let resolved = report
        .pointer("/meta/direction_contract/binding/resolved")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| bad("pause: missing resolved directions"))?;
    let dispositions = report
        .pointer("/meta/direction_contract/dispositions")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| bad("pause: missing direction dispositions"))?;
    let mut previous_sample = source_in;
    let mut previous_words = 0;
    let mut inserted = 0u64;
    let mut pauses = Vec::new();
    for item in items {
        let index = integer(item, "override_index")?;
        let sample = integer(item, "source_sample")?;
        let count = integer(item, "inserted_samples")?;
        if sample <= previous_sample || sample >= source_out || count == 0 {
            return Err(bad("pause: unordered or outside source sentence"));
        }
        let bindings: Vec<_> = resolved
            .iter()
            .filter(|v| v["override_index"].as_u64() == Some(index))
            .collect();
        let kinds: Vec<_> = dispositions
            .iter()
            .filter(|v| v["override_index"].as_u64() == Some(index))
            .collect();
        if bindings.len() != 1 || kinds.len() != 1 || kinds[0]["kind"] != "authored_silence" {
            return Err(bad("pause: missing, duplicate or undisposed direction"));
        }
        let binding = bindings[0];
        if binding["cue"] != cue || binding["direction"]["cue"] != cue {
            return Err(bad("pause: direction belongs to another cue"));
        }
        let boundary = usize::try_from(integer(binding, "end_word_index")?)
            .ok()
            .and_then(|i| i.checked_add(1))
            .and_then(|i| i.checked_sub(word_offset))
            .filter(|&i| i > 0 && i < tokens.len())
            .ok_or_else(|| bad("pause: boundary outside sentence"))?;
        let phrase = direction_words(
            binding["direction"]["after"]
                .as_str()
                .ok_or_else(|| bad("pause: missing after phrase"))?,
        );
        if phrase.is_empty() || !tokens[..boundary].ends_with(&phrase) {
            return Err(bad("pause: direction phrase does not match sentence"));
        }
        let mut n = 0;
        let after_words = words
            .iter()
            .position(|w| {
                n += direction_words(w).len();
                n == boundary
            })
            .map(|i| i + 1)
            .filter(|&i| i > previous_words && i < words.len())
            .ok_or_else(|| bad("pause: direction splits a display word or is unordered"))?;
        let start = cue_sample
            .checked_add(sample - source_in)
            .and_then(|n| n.checked_add(inserted))
            .ok_or_else(|| bad("pause sample overflow"))?;
        let end = start.checked_add(count).ok_or_else(|| bad("pause sample overflow"))?;
        inserted = inserted
            .checked_add(count)
            .ok_or_else(|| bad("pause sample overflow"))?;
        pauses.push(CaptionPause {
            after_words,
            start: start as f64 / rate,
            end: end as f64 / rate,
        });
        previous_sample = sample;
        previous_words = after_words;
    }
    if source_out.checked_sub(source_in).and_then(|n| n.checked_add(inserted)) != row["out_samples"].as_u64() {
        return Err(bad("pause: source plus insertions does not equal output samples"));
    }
    Ok(pauses)
}

/// The speech regions of a line's audio (seconds), at 48 kHz.
fn speech(audio: &Stereo) -> Vec<(f64, f64)> {
    duck::regions(&duck::vad(audio), 0.15)
}

/// Where each sentence sits in its line's audio (seconds from the line's
/// start): the speech regions split into consecutive groups whose lengths
/// best fit the sentences' spoken lengths. With fewer regions than
/// sentences, the speech is shared out in proportion to those lengths.
pub fn sentence_spans(audio: &Stereo, sentences: &[Spoken]) -> Vec<(f64, f64)> {
    if sentences.iter().all(|s| s.recorded.is_some()) {
        return sentences
            .iter()
            .filter_map(|s| s.recorded.map(|(a, b, _)| (a, b)))
            .collect();
    }
    let r = speech(audio);
    let n = sentences.len();
    if n == 0 || r.is_empty() {
        return Vec::new();
    }
    if r.len() < n {
        let (a, b) = (r[0].0, r[r.len() - 1].1);
        let total: f64 = sentences.iter().map(|s| s.seconds).sum();
        let mut t = a;
        return sentences
            .iter()
            .map(|s| {
                let e = t + (b - a) * s.seconds / total;
                let span = (t, e);
                t = e;
                span
            })
            .collect();
    }
    // best[k][j]: the least misfit splitting regions 0..j into sentences 0..k
    let m = r.len();
    let mut best = vec![vec![f64::INFINITY; m + 1]; n + 1];
    let mut from = vec![vec![0usize; m + 1]; n + 1];
    best[0][0] = 0.0;
    for k in 1..=n {
        for j in k..=m {
            for i in (k - 1)..j {
                let fit = best[k - 1][i] + (r[j - 1].1 - r[i].0 - sentences[k - 1].seconds).abs();
                if fit < best[k][j] {
                    best[k][j] = fit;
                    from[k][j] = i;
                }
            }
        }
    }
    let mut spans = vec![(0.0, 0.0); n];
    let mut j = m;
    for k in (1..=n).rev() {
        let i = from[k][j];
        spans[k - 1] = (r[i].0, r[j - 1].1);
        j = i;
    }
    spans
}

/// `text` wrapped to lines of at most [`LINE_CHARS`] characters, at spaces
/// (a longer word stands alone).
pub fn wrap(text: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(l) if l.chars().count() + 1 + word.chars().count() <= LINE_CHARS => {
                l.push(' ');
                l.push_str(word);
            }
            _ => lines.push(word.to_owned()),
        }
    }
    lines
}

/// The characters a caption's lines hold (the weight of its share of the
/// sentence's span).
fn weight(lines: &[String]) -> usize {
    lines.iter().map(|l| l.chars().count()).sum()
}

/// Keep a spoken count together across caption changes: "two hundred"
/// must not disappear before "seventy-seven" replaces it. Line wrapping
/// within one caption remains free to use both visible lines.
fn number_word(word: &str) -> bool {
    let word = word
        .trim_matches(|c: char| !c.is_ascii_alphanumeric())
        .to_ascii_lowercase();
    !word.is_empty()
        && word.split('-').all(|part| {
            matches!(
                part,
                "zero"
                    | "one"
                    | "two"
                    | "three"
                    | "four"
                    | "five"
                    | "six"
                    | "seven"
                    | "eight"
                    | "nine"
                    | "ten"
                    | "eleven"
                    | "twelve"
                    | "thirteen"
                    | "fourteen"
                    | "fifteen"
                    | "sixteen"
                    | "seventeen"
                    | "eighteen"
                    | "nineteen"
                    | "twenty"
                    | "thirty"
                    | "forty"
                    | "fifty"
                    | "sixty"
                    | "seventy"
                    | "eighty"
                    | "ninety"
                    | "hundred"
                    | "thousand"
                    | "million"
                    | "billion"
                    | "trillion"
            )
        })
}

/// A sentence's words in the fewest captions that hold them, each of at most
/// [`MAX_LINES`] lines of [`LINE_CHARS`], split where the shortest caption is
/// as long as the limits allow (then the longest as short as they allow).
/// Wrapping the sentence once and taking the lines two at a time leaves
/// the last caption whatever is over: the last word or two of the sentence,
/// on a cue of a fraction of a second ("and forty.", "that.", "half.").
fn balanced_captions(words: &[&str]) -> Vec<Vec<String>> {
    let greedy = wrap(&words.join(" "));
    let count = greedy.len().div_ceil(MAX_LINES);
    let n = words.len();
    if count <= 1 {
        return if greedy.is_empty() { Vec::new() } else { vec![greedy] };
    }
    // best[k][j]: (shortest, longest) caption weight of the best split of
    // words[..j] into k captions; from[k][j]: where its last caption starts
    let mut best: Vec<Vec<Option<(usize, usize)>>> = vec![vec![None; n + 1]; count + 1];
    let mut from = vec![vec![0usize; n + 1]; count + 1];
    best[0][0] = Some((usize::MAX, 0));
    for k in 1..=count {
        for j in k..=n {
            if j < n && number_word(words[j - 1]) && number_word(words[j]) {
                continue;
            }
            for i in (k - 1)..j {
                let Some((lo, hi)) = best[k - 1][i] else {
                    continue;
                };
                let lines = wrap(&words[i..j].join(" "));
                if lines.len() > MAX_LINES {
                    continue;
                }
                let w = weight(&lines);
                let cand = (lo.min(w), hi.max(w));
                let better = match best[k][j] {
                    None => true,
                    Some((blo, bhi)) => cand.0 > blo || (cand.0 == blo && cand.1 < bhi),
                };
                if better {
                    best[k][j] = Some(cand);
                    from[k][j] = i;
                }
            }
        }
    }
    if best[count][n].is_none() {
        // not reachable (the greedy wrap's own chunks are such a split)
        return greedy.chunks(MAX_LINES).map(<[String]>::to_vec).collect();
    }
    let mut groups = Vec::with_capacity(count);
    let mut j = n;
    for k in (1..=count).rev() {
        let i = from[k][j];
        groups.push(wrap(&words[i..j].join(" ")));
        j = i;
    }
    groups.reverse();
    groups
}

/// One sentence as captions over `[start, end)`: at most [`MAX_LINES`] lines
/// at a time, the span shared in proportion to each caption's characters. A
/// sentence of more lines than one caption holds is shared out evenly between
/// the captions it needs, so no cue is the sentence's last word on its own.
pub fn sentence_captions(text: &str, start: f64, end: f64) -> Vec<Caption> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let groups = balanced_captions(&words);
    let share = |g: &[String]| weight(g) as f64;
    let total: f64 = groups.iter().map(|g| share(g)).sum();
    let mut t = start;
    groups
        .into_iter()
        .map(|g| {
            let e = t + (end - start) * share(&g) / total;
            let c = Caption {
                start: t,
                end: e,
                lines: g,
            };
            t = e;
            c
        })
        .collect()
}

fn directed_captions(sentence: &Spoken, start: f64, end: f64) -> Vec<Caption> {
    let words: Vec<_> = sentence.text.split_whitespace().collect();
    let mut captions = Vec::new();
    let mut from = 0;
    let mut at = start;
    for pause in &sentence.pauses {
        let mut before = sentence_captions(&words[from..pause.after_words].join(" "), at, pause.start);
        // Keep the pre-pause thought visible through the authored silence;
        // only the next caption may introduce the post-pause words.
        if let Some(last) = before.last_mut() {
            last.end = pause.end;
        }
        captions.extend(before);
        from = pause.after_words;
        at = pause.end;
    }
    captions.extend(sentence_captions(&words[from..].join(" "), at, end));
    captions
}

fn prediction_end(chapter: &ChapterTimeline, line_index: usize, cue_duration: f64) -> Option<f64> {
    let fps = f64::from(chapter.fps);
    let cue_end = chapter.lines[line_index].start_frame as f64 / fps + cue_duration;
    let next = chapter.lines.get(line_index + 1)?.start_frame as f64 / fps;
    chapter
        .marks
        .iter()
        .any(|mark| {
            let at = mark.frame as f64 / fps;
            mark.label == "predict" && at >= cue_end && at < next
        })
        .then_some(next)
}

/// The video's captions: every chapter in order (each starting where the
/// ones before it end), every line at its placement, every sentence where
/// its line's audio (the timeline's WAV) puts it.
pub fn video_captions(chapters: &[ChapterTimeline], report: &HashMap<String, Vec<Spoken>>) -> io::Result<Vec<Caption>> {
    let mut out = Vec::new();
    let mut offset = 0.0;
    for ch in chapters {
        if ch.fps == 0 {
            return Err(bad("caption timeline has zero fps"));
        }
        let fps = f64::from(ch.fps);
        for (line_index, line) in ch.lines.iter().enumerate() {
            let sentences = report
                .get(&line.id)
                .ok_or_else(|| bad(format!("{}: line {} is not in the report", ch.chapter, line.id)))?;
            let d = wav::read(&line.wav)?;
            let duration = d.mono().len() as f64 / f64::from(d.rate);
            if let Some((_, _, end)) = sentences.last().and_then(|s| s.recorded)
                && (end - duration).abs() > 1.0 / f64::from(d.rate)
            {
                return Err(bad(format!(
                    "{}: recorded captions cover {end:.6} s but WAV has {duration:.6} s",
                    line.id
                )));
            }
            let mono = resample(&d.mono(), d.rate, SAMPLE_RATE);
            let spans = sentence_spans(&Stereo::from_mono(&mono), sentences);
            if spans.len() != sentences.len() {
                return Err(bad(format!("{}: no speech found in {}", line.id, line.wav.display())));
            }
            let at = offset + line.start_frame as f64 / fps;
            let first_caption = out.len();
            for (s, (a, b)) in sentences.iter().zip(spans) {
                out.extend(directed_captions(s, a, b).into_iter().map(|mut caption| {
                    caption.start += at;
                    caption.end += at;
                    caption
                }));
            }
            if let Some(end) = prediction_end(ch, line_index, duration)
                && let Some(last) = out[first_caption..].last_mut()
            {
                last.end = offset + end;
            }
        }
        offset += ch.frames as f64 / fps;
    }
    Ok(out)
}

/// Captions must read: each starts before it ends, after the one before it
/// has ended, with at most [`MAX_LINES`] lines of at most [`LINE_CHARS`].
pub fn validate(captions: &[Caption]) -> Result<(), String> {
    for (k, c) in captions.iter().enumerate() {
        if c.start.partial_cmp(&c.end) != Some(std::cmp::Ordering::Less) {
            return Err(format!("caption {}: {} -> {}", k + 1, c.start, c.end));
        }
        if c.lines.is_empty() || c.lines.len() > MAX_LINES {
            return Err(format!("caption {}: {} lines", k + 1, c.lines.len()));
        }
        if let Some(l) = c
            .lines
            .iter()
            .find(|l| l.chars().count() > LINE_CHARS && l.contains(' '))
        {
            return Err(format!("caption {}: line too long: {l:?}", k + 1));
        }
        if k > 0 && c.start < captions[k - 1].end - 1e-9 {
            return Err(format!(
                "caption {} starts at {:.3} s, before caption {} ends at {:.3} s",
                k + 1,
                c.start,
                k,
                captions[k - 1].end
            ));
        }
    }
    Ok(())
}

fn stamp(t: f64, decimal: char) -> String {
    let ms = (t.max(0.0) * 1000.0).round() as u64;
    format!(
        "{:02}:{:02}:{:02}{decimal}{:03}",
        ms / 3_600_000,
        ms / 60_000 % 60,
        ms / 1000 % 60,
        ms % 1000
    )
}

/// SubRip.
pub fn to_srt(captions: &[Caption]) -> String {
    let mut s = String::new();
    for (k, c) in captions.iter().enumerate() {
        let _ = write!(
            s,
            "{}\n{} --> {}\n{}\n\n",
            k + 1,
            stamp(c.start, ','),
            stamp(c.end, ','),
            c.lines.join("\n")
        );
    }
    s
}

/// WebVTT.
pub fn to_vtt(captions: &[Caption]) -> String {
    let mut s = String::from("WEBVTT\n\n");
    for c in captions {
        let _ = write!(
            s,
            "{} --> {}\n{}\n\n",
            stamp(c.start, '.'),
            stamp(c.end, '.'),
            c.lines.join("\n")
        );
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Speech-like bursts at the given spans (seconds) in `total` seconds.
    fn bursts(spans: &[(f64, f64)], total: f64) -> Stereo {
        let n = (total * 48_000.0) as usize;
        let mut m = vec![0.0; n];
        let mut x: u64 = 9;
        for &(a, b) in spans {
            for v in m.iter_mut().take((b * 48_000.0) as usize).skip((a * 48_000.0) as usize) {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                *v = 0.2 * ((x >> 11) as f64 / (1u64 << 53) as f64 - 0.5);
            }
        }
        Stereo::from_mono(&m)
    }

    fn spoken(seconds: &[f64]) -> Vec<Spoken> {
        seconds
            .iter()
            .enumerate()
            .map(|(k, &s)| Spoken {
                text: format!("Sentence {k}."),
                seconds: s,
                recorded: None,
                pauses: Vec::new(),
            })
            .collect()
    }

    #[test]
    fn recorded_gaps_drive_captions_even_when_vad_would_merge_the_sentences() {
        let report = parse_report(
            r#"{"meta":{"rate":24000},"lines":{"01_1":[
            {"sentence":"First.","start":8,"end":9,"pause_cut_s":0,"out_samples":24000,"gap_after_samples":12000},
            {"sentence":"Second.","start":9,"end":11,"pause_cut_s":0,"out_samples":48000,"gap_after_samples":0}]}}"#,
        )
        .unwrap();
        // Deliberately continuous speech: VAD cannot locate the authored gap.
        let spans = sentence_spans(&bursts(&[(0.0, 3.5)], 3.5), &report["01_1"]);
        assert_eq!(spans, [(0.0, 1.0), (1.5, 3.5)]);
        assert_eq!(report["01_1"][1].recorded, Some((1.5, 3.5, 3.5)));
    }

    fn pause_report() -> serde_json::Value {
        serde_json::json!({"meta":{"rate":24000,"direction_contract":{
            "binding":{"resolved":[{"override_index":46,"cue":"10_7","end_word_index":48,
                "direction":{"cue":"10_7","after":"half the samples"}}]},
            "dispositions":[{"override_index":46,"kind":"authored_silence"}]}},"lines":{"10_7":[
            {"sentence":"For programmers.","start":0,"end":1.2916666666666667,"pause_cut_s":0,"out_samples":31000,"gap_after_samples":0},
            {"sentence":"A sampling profiler checks what your program is doing many times a second, so it measures where the time goes, not how often each function is called.",
                "start":1.2916666666666667,"end":10.083333333333334,"pause_cut_s":0,"out_samples":211000,"gap_after_samples":0},
            {"sentence":"On average, one ten millisecond call shows up as often as five two millisecond calls put together: half the samples, but only a sixth of the calls.",
                "start":10.083333333333334,"end":19.44,"pause_cut_s":0,"source_in_sample":242000,"source_out_sample":466560,
                "out_samples":229360,"gap_after_samples":0,"intra_sentence_pauses":[{"override_index":46,"source_sample":424000,"inserted_samples":4800}]}
        ]}})
    }

    #[test]
    fn profiler_contrast_waits_for_the_bound_pause_and_keeps_call_phrases_together() {
        let report = parse_report(&pause_report().to_string()).unwrap();
        let sentence = &report["10_7"][2];
        let (a, b, _) = sentence.recorded.unwrap();
        let caps = directed_captions(sentence, a, b);
        assert_eq!(caps.len(), 3);
        assert_eq!(
            caps.iter().map(|c| c.lines.join("\n")).collect::<Vec<_>>(),
            [
                "On average, one ten millisecond call shows\nup as often as",
                "five two millisecond calls put together:\nhalf the samples,",
                "but only a sixth of the calls."
            ]
        );
        assert_eq!(caps[1].end, 17.866666666666667);
        assert_eq!(caps[2].start, caps[1].end);
        assert!((caps[0].end + 55.13333333333333 - 68.975).abs() < 0.000501);
        assert!(validate(&caps).is_ok());
        assert_eq!(
            caps.iter()
                .flat_map(|c| &c.lines)
                .cloned()
                .collect::<Vec<_>>()
                .join(" "),
            sentence.text
        );
    }

    #[test]
    fn pause_offsets_include_prior_sentence_gaps_and_prior_insertions() {
        let value = serde_json::json!({"meta":{"rate":100,"direction_contract":{
            "binding":{"resolved":[
                {"override_index":0,"cue":"x","end_word_index":2,"direction":{"cue":"x","after":"One"}},
                {"override_index":1,"cue":"x","end_word_index":5,"direction":{"cue":"x","after":"twenty-one calls"}}]},
            "dispositions":[{"override_index":0,"kind":"authored_silence"},{"override_index":1,"kind":"authored_silence"}]}},
            "lines":{"x":[
                {"sentence":"Warm up.","start":0,"end":1,"pause_cut_s":0,"out_samples":100,"gap_after_samples":50},
                {"sentence":"One twenty-one calls remain.","start":2,"end":10,"pause_cut_s":0,"out_samples":850,"gap_after_samples":0,
                "source_in_sample":200,"source_out_sample":1000,"intra_sentence_pauses":[
                    {"override_index":0,"source_sample":400,"inserted_samples":20},
                    {"override_index":1,"source_sample":800,"inserted_samples":30}]}]}});
        let parsed = parse_report(&value.to_string()).unwrap();
        let s = &parsed["x"][1];
        assert_eq!(
            s.pauses,
            [
                CaptionPause {
                    after_words: 1,
                    start: 3.5,
                    end: 3.7
                },
                CaptionPause {
                    after_words: 3,
                    start: 7.7,
                    end: 8.0
                }
            ]
        );
        let caps = directed_captions(s, 1.5, 10.0);
        assert_eq!(
            caps.iter()
                .map(|c| (c.start, c.end, c.lines.join(" ")))
                .collect::<Vec<_>>(),
            [
                (1.5, 3.7, "One".into()),
                (3.7, 8.0, "twenty-one calls".into()),
                (8.0, 10.0, "remain.".into())
            ]
        );
        let mut invalid = value;
        invalid["meta"]["direction_contract"]["binding"]["resolved"][1]["end_word_index"] = 3.into();
        invalid["meta"]["direction_contract"]["binding"]["resolved"][1]["direction"]["after"] = "twenty".into();
        assert!(
            parse_report(&invalid.to_string())
                .unwrap_err()
                .to_string()
                .contains("display word")
        );
    }

    #[test]
    fn malformed_pause_contracts_are_refused_instead_of_guessing_caption_boundaries() {
        for (pointer, value) in [
            (
                "/meta/direction_contract/binding/resolved/0/cue",
                serde_json::json!("other"),
            ),
            (
                "/meta/direction_contract/binding/resolved/0/end_word_index",
                serde_json::json!(29),
            ),
            (
                "/meta/direction_contract/binding/resolved/0/end_word_index",
                serde_json::json!(55),
            ),
            (
                "/meta/direction_contract/binding/resolved/0/direction/after",
                serde_json::json!("wrong phrase"),
            ),
            (
                "/meta/direction_contract/dispositions/0/kind",
                serde_json::json!("native_visual_hold"),
            ),
            ("/meta/direction_contract/dispositions", serde_json::json!([])),
            (
                "/lines/10_7/2/intra_sentence_pauses/0/source_sample",
                serde_json::json!(242000),
            ),
            (
                "/lines/10_7/2/intra_sentence_pauses/0/inserted_samples",
                serde_json::json!(0),
            ),
            ("/lines/10_7/2/out_samples", serde_json::json!(224560)),
            ("/lines/10_7/2/intra_sentence_pauses", serde_json::json!({})),
        ] {
            let mut input = pause_report();
            *input.pointer_mut(pointer).unwrap() = value;
            assert!(parse_report(&input.to_string()).is_err(), "accepted {pointer}: {input}");
        }
        for pointer in [
            "/meta/direction_contract/binding/resolved",
            "/lines/10_7/2/intra_sentence_pauses",
        ] {
            let mut input = pause_report();
            let array = input.pointer_mut(pointer).unwrap().as_array_mut().unwrap();
            array.push(array[0].clone());
            assert!(
                parse_report(&input.to_string()).is_err(),
                "accepted duplicate {pointer}"
            );
        }
    }

    #[test]
    fn contract_words_match_apostrophes_digits_and_hyphenated_words() {
        assert_eq!(
            direction_words("What's twenty-one? They’re 2nd. 'ab'cd 42'foo9"),
            ["what's", "twenty", "one", "they're", "2nd", "ab'cd", "42'foo", "9"]
        );
    }

    #[test]
    fn video_export_derives_prediction_holds_from_each_timeline_and_chapter_offset() {
        use explainer_score::timeline::{LinePlacement, Mark};
        let path = std::env::temp_dir().join(format!("caption_prediction_{}.wav", std::process::id()));
        wav::write(&path, &Stereo::from_mono(&vec![0.0; 48000]), wav::Format::F32).unwrap();
        let report = parse_report(r#"{"meta":{"rate":48000},"lines":{
            "q":[{"sentence":"What's the average wait there?","start":0,"end":1,"pause_cut_s":0,"out_samples":48000,"gap_after_samples":0}],
            "a":[{"sentence":"The answer.","start":0,"end":1,"pause_cut_s":0,"out_samples":48000,"gap_after_samples":0}]}}"#).unwrap();
        for fps in [15, 60] {
            let factor = u64::from(fps / 15);
            let leading = ChapterTimeline {
                fps,
                frames: u64::from(fps),
                ..Default::default()
            };
            let mut chapter = ChapterTimeline {
                chapter: "question".into(),
                fps,
                frames: 120 * factor,
                lines: vec![
                    LinePlacement {
                        id: "q".into(),
                        start_frame: 7 * factor,
                        gain_db: 0.0,
                        wav: path.clone(),
                    },
                    LinePlacement {
                        id: "a".into(),
                        start_frame: 75 * factor,
                        gain_db: 0.0,
                        wav: path.clone(),
                    },
                ],
                marks: vec![Mark {
                    label: "predict".into(),
                    frame: 30 * factor,
                }],
                ..Default::default()
            };
            let caps = video_captions(&[leading.clone(), chapter.clone()], &report).unwrap();
            assert_eq!(caps.len(), 2);
            assert_eq!(caps[0].end, 6.0);
            assert_eq!(caps[1].start, 6.0);
            assert!(validate(&caps).is_ok());
            // A later final timeline must move the hold, not reuse 15 fps sidecars.
            chapter.lines[1].start_frame += 15 * factor;
            assert_eq!(
                video_captions(&[leading, chapter.clone()], &report).unwrap()[0].end,
                7.0
            );
            for mark in [
                Mark {
                    label: "reveal".into(),
                    frame: 30 * factor,
                },
                Mark {
                    label: "predict".into(),
                    frame: 8 * factor,
                },
                Mark {
                    label: "predict".into(),
                    frame: 90 * factor,
                },
            ] {
                chapter.marks = vec![mark];
                assert!(prediction_end(&chapter, 0, 1.0).is_none());
            }
            assert!(prediction_end(&chapter, 1, 1.0).is_none());
        }
    }

    #[test]
    fn incomplete_sample_timing_is_rejected_instead_of_silently_guessed() {
        let complete = r#"{"meta":{"rate":24000},"lines":{"01_1":[
            {"sentence":"First.","start":0,"end":1,"pause_cut_s":0,"out_samples":24000,"gap_after_samples":0}]}}"#;
        for invalid in [
            complete.replace("\"rate\":24000", "\"rate\":0"),
            complete.replace("\"out_samples\":24000", "\"out_samples\":0"),
            complete.replace(",\"gap_after_samples\":0", ""),
        ] {
            assert!(parse_report(&invalid).is_err(), "{invalid}");
        }
        let legacy = r#"{"01_1":[{"sentence":"First.","start":0,"end":1,"pause_cut_s":0}]}"#;
        assert_eq!(parse_report(legacy).unwrap()["01_1"][0].recorded, None);
    }

    #[test]
    fn sentences_are_found_at_the_silences_that_fit_their_lengths() {
        // two sentences, the first with a comma pause inside it (0.25 s),
        // then a 0.4 s gap: regions 0.3-1.2, 1.45-2.4, 2.8-4.3
        let audio = bursts(&[(0.3, 1.2), (1.45, 2.4), (2.8, 4.3)], 4.6);
        let spans = sentence_spans(&audio, &spoken(&[2.1, 1.5]));
        assert_eq!(spans.len(), 2);
        // the split is at the 0.4 s gap, not at the comma (detector dilation ±30 ms)
        assert!(
            (spans[0].0 - 0.3).abs() < 0.05 && (spans[0].1 - 2.4).abs() < 0.05,
            "{spans:?}"
        );
        assert!(
            (spans[1].0 - 2.8).abs() < 0.05 && (spans[1].1 - 4.3).abs() < 0.05,
            "{spans:?}"
        );
        // speech merged into fewer regions than sentences: shared by length
        let merged = sentence_spans(&bursts(&[(0.2, 3.2)], 3.5), &spoken(&[1.0, 2.0]));
        assert!(((merged[0].1 - merged[0].0) * 2.0 - (merged[1].1 - merged[1].0)).abs() < 0.05);
    }

    #[test]
    fn long_sentences_wrap_and_split_into_two_line_captions() {
        let text = "In the end, D measures exactly what shuffling destroys: the gap between the real pattern and independence, nothing else.";
        let lines = wrap(text);
        assert!(lines.iter().all(|l| l.chars().count() <= LINE_CHARS), "{lines:?}");
        assert_eq!(lines.join(" "), text, "every word once, in order");
        let caps = sentence_captions(text, 10.0, 16.0);
        assert_eq!(caps.len(), 2);
        assert!(caps.iter().all(|c| c.lines.len() <= MAX_LINES));
        assert!((caps[0].start - 10.0).abs() < 1e-12 && (caps[1].end - 16.0).abs() < 1e-12);
        assert!((caps[0].end - caps[1].start).abs() < 1e-12, "back to back");
        assert!(validate(&caps).is_ok());
    }

    #[test]
    fn the_medical_opening_keeps_its_resident_count_in_one_caption() {
        let text = "A survey published in two thousand seven tested two hundred seventy-seven internal medicine residents in Connecticut.";
        let caps = sentence_captions(text, 0.6, 6.85);
        assert_eq!(caps.len(), 2);
        assert!(validate(&caps).is_ok());
        assert!(
            caps.iter()
                .any(|c| c.lines.join(" ").contains("two hundred seventy-seven"))
        );
        assert_eq!(
            caps.iter()
                .flat_map(|c| c.lines.iter())
                .cloned()
                .collect::<Vec<_>>()
                .join(" "),
            text
        );
    }

    /// The four sentences of Video F's final whose last words stood alone on
    /// a cue of 0.33 to 0.62 s (EmeraldOwl's delivery check of e1b0ad3: cues
    /// 38 'and forty.', 70 'that.', 128 'half.', 227 'hundreds.'), with the
    /// spans their cues covered together: each is shared between the two
    /// captions it needs so that neither is a stub, with the old code's
    /// limits (two lines of 42, back to back, every word once, in order).
    #[test]
    fn a_sentence_is_never_left_with_its_last_words_on_a_cue_of_their_own() {
        let cases = [
            (
                "The next number comes in because thirty-five sits halfway between thirty and forty.",
                117.887,
                122.387,
            ),
            (
                "Twenty times four is eighty already, so twenty times forty has to be ten times that.",
                218.260,
                223.760,
            ),
            (
                "The swap pays when one side is a friendly percent, like a tenth, a quarter or a half.",
                405.557,
                411.097,
            ),
            (
                "Further out, the distance squared runs past two digits, and it spills into the hundreds.",
                725.930,
                731.870,
            ),
        ];
        for (text, start, end) in cases {
            let caps = sentence_captions(text, start, end);
            let words = text.split_whitespace().collect::<Vec<_>>().join(" ");
            assert_eq!(caps.len(), 2, "{text}: the fewest captions that hold it");
            assert!(validate(&caps).is_ok(), "{caps:?}");
            let said = caps
                .iter()
                .flat_map(|c| c.lines.iter().cloned())
                .collect::<Vec<_>>()
                .join(" ");
            assert_eq!(said, words, "every word once, in order");
            assert!((caps[0].start - start).abs() < 1e-9 && (caps[1].end - end).abs() < 1e-9);
            assert!((caps[0].end - caps[1].start).abs() < 1e-9, "back to back");
            let shortest = caps.iter().map(|c| c.end - c.start).fold(f64::INFINITY, f64::min);
            assert!(
                shortest >= 1.5,
                "{text}: a cue of {shortest:.2} s ({:?} then {:?})",
                caps[0].lines,
                caps[1].lines
            );
            // the old split gave the second cue 0.33 to 0.62 s; neither is
            // under a third of the other's characters now
            let chars = |c: &Caption| c.lines.iter().map(|l| l.chars().count()).sum::<usize>();
            assert!(chars(&caps[1]) * 3 >= chars(&caps[0]) && chars(&caps[0]) * 3 >= chars(&caps[1]));
        }
    }

    #[test]
    fn a_sentence_that_fits_one_caption_is_one_and_a_long_one_is_shared_evenly() {
        // two lines or fewer: one caption, the greedy wrap as it was
        let short = sentence_captions("Tens times tens are hundreds.", 0.0, 2.6);
        assert_eq!(short.len(), 1);
        assert_eq!(short[0].lines, ["Tens times tens are hundreds."]);
        let two = "The counts carry the unfairness, and the order is perfectly fair, whatever the coin.";
        let two = sentence_captions(two, 0.0, 6.0);
        assert!(two.iter().all(|c| c.lines.len() <= MAX_LINES));
        // five lines of words: three captions, each at least a quarter of the span
        let long = "Every sentence of the narration is shared out between as few captions as its lines allow, \
                    so that the cue that ends it is as long as the limits let it be, and no word of it is left \
                    alone for a third of a second at the end of a sentence that took a while to say.";
        let caps = sentence_captions(long, 10.0, 22.0);
        assert!(caps.len() >= 3, "{caps:?}");
        assert!(validate(&caps).is_ok(), "{caps:?}");
        let each = 12.0 / caps.len() as f64;
        assert!(
            caps.iter().all(|c| c.end - c.start > each * 0.6),
            "{:?}",
            caps.iter().map(|c| c.end - c.start).collect::<Vec<_>>()
        );
        assert_eq!(sentence_captions("", 0.0, 1.0), Vec::<Caption>::new());
    }

    #[test]
    fn overlapping_or_backwards_captions_are_refused() {
        let c = |a: f64, b: f64| Caption {
            start: a,
            end: b,
            lines: vec!["x".into()],
        };
        assert!(validate(&[c(0.0, 1.0), c(1.0, 2.0)]).is_ok());
        assert!(
            validate(&[c(0.0, 1.5), c(1.0, 2.0)])
                .unwrap_err()
                .contains("before caption 1 ends")
        );
        assert!(validate(&[c(1.0, 1.0)]).is_err());
        let three = Caption {
            lines: vec!["a".into(), "b".into(), "c".into()],
            ..c(0.0, 1.0)
        };
        assert!(validate(&[three]).is_err());
    }

    #[test]
    fn timestamps_are_srt_and_vtt_shaped() {
        let caps = [Caption {
            start: 3_723.456_7,
            end: 3_724.0,
            lines: vec!["one".into(), "two".into()],
        }];
        assert_eq!(to_srt(&caps), "1\n01:02:03,457 --> 01:02:04,000\none\ntwo\n\n");
        assert_eq!(to_vtt(&caps), "WEBVTT\n\n01:02:03.457 --> 01:02:04.000\none\ntwo\n\n");
    }

    #[test]
    fn a_report_with_meta_reads_its_lines_only() {
        let sentence = |text: &str, start: f64| serde_json::json!({"sentence": text, "start": start, "end": start + 2.0, "pause_cut_s": 0.25});
        let report = serde_json::json!({
            "meta": {"sentence_lufs": -23.0, "rate": 24000},
            "lines": {"01_1": [sentence("One.", 0.0), sentence("Two.", 2.5)], "01_2": [sentence("Three.", 0.0)]}
        });
        let path = std::env::temp_dir().join(format!("explainer_kit_report_meta_{}.json", std::process::id()));
        std::fs::write(&path, report.to_string()).unwrap();
        let r = read_report(&path).unwrap();
        assert_eq!(r.len(), 2, "two lines, and meta is not one");
        assert_eq!(r["01_1"].len(), 2);
        assert_eq!(r["01_1"][1].text, "Two.");
        assert!((r["01_2"][0].seconds - 1.75).abs() < 1e-9, "end - start - pause_cut_s");
    }

    #[test]
    fn the_hoeffding_v5_report_reads_as_61_lines_of_display_sentences() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../explainer_corpus/hoeffding_d/narration_v5/report.json");
        let r = read_report(&path).unwrap();
        assert_eq!(r.len(), 61);
        assert_eq!(r.values().map(Vec::len).sum::<usize>(), 131, "131 sentences");
        assert!(r.values().flatten().all(|s| s.seconds > 0.2 && !s.text.is_empty()));
        // the display text, not the respelled TTS input
        assert_eq!(
            r["h1"][0].text,
            "Here are a hundred and fifty measurements of two quantities, X and Y."
        );
    }
}
