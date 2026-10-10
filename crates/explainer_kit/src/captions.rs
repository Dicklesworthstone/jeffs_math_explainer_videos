//! Captions (plan §13): SRT and WebVTT for a whole video, sentence by
//! sentence, saying what was actually spoken.
//!
//! The narration skill's `report.json` lists each line's sentences as they
//! are displayed (its `sentence`; respellings live only in `tts`) and the
//! span each was spoken over in its own take (`end − start − pause_cut_s`).
//! The assembled line WAV joins those spans with gaps it does not record, so
//! each sentence is found in the line's audio: the voice-activity detector's
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
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path)?).map_err(|e| bad(format!("{}: {e}", path.display())))?;
    let lines = v
        .as_object()
        .ok_or_else(|| bad("report.json: not an object of lines"))?;
    let mut out = HashMap::new();
    for (id, sentences) in lines {
        let list = sentences
            .as_array()
            .ok_or_else(|| bad(format!("report.json: line {id} is not a list")))?;
        let mut spoken = Vec::new();
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
            if text.trim().is_empty() || seconds <= 0.0 {
                return Err(bad(format!("{id} sentence {k}: empty or {seconds} s long")));
            }
            spoken.push(Spoken {
                text: text.trim().to_owned(),
                seconds,
            });
        }
        out.insert(id.clone(), spoken);
    }
    Ok(out)
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

/// One sentence as captions over `[start, end)`: [`MAX_LINES`] lines at a
/// time, the span shared in proportion to each caption's characters.
pub fn sentence_captions(text: &str, start: f64, end: f64) -> Vec<Caption> {
    let lines = wrap(text);
    let groups: Vec<Vec<String>> = lines.chunks(MAX_LINES).map(<[String]>::to_vec).collect();
    let weight = |g: &[String]| g.iter().map(|l| l.chars().count()).sum::<usize>() as f64;
    let total: f64 = groups.iter().map(|g| weight(g)).sum();
    let mut t = start;
    groups
        .into_iter()
        .map(|g| {
            let e = t + (end - start) * weight(&g) / total;
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

/// The video's captions: every chapter in order (each starting where the
/// ones before it end), every line at its placement, every sentence where
/// its line's audio (the timeline's WAV) puts it.
pub fn video_captions(chapters: &[ChapterTimeline], report: &HashMap<String, Vec<Spoken>>) -> io::Result<Vec<Caption>> {
    let mut out = Vec::new();
    let mut offset = 0.0;
    for ch in chapters {
        let fps = f64::from(ch.fps);
        for line in &ch.lines {
            let sentences = report
                .get(&line.id)
                .ok_or_else(|| bad(format!("{}: line {} is not in the report", ch.chapter, line.id)))?;
            let d = wav::read(&line.wav)?;
            let mono = resample(&d.mono(), d.rate, SAMPLE_RATE);
            let spans = sentence_spans(&Stereo::from_mono(&mono), sentences);
            if spans.len() != sentences.len() {
                return Err(bad(format!("{}: no speech found in {}", line.id, line.wav.display())));
            }
            let at = offset + line.start_frame as f64 / fps;
            for (s, (a, b)) in sentences.iter().zip(spans) {
                out.extend(sentence_captions(&s.text, at + a, at + b));
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
            })
            .collect()
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
