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
    // the narration skill writes {"meta": {..}, "lines": {id: [..]}}; a shipped
    // narration (Hoeffding v5) is the bare {id: [..]}
    let lines = match v.get("lines").and_then(serde_json::Value::as_object) {
        Some(lines) => lines,
        None => v
            .as_object()
            .ok_or_else(|| bad("report.json: not an object of lines"))?,
    };
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

/// The characters a caption's lines hold (the weight of its share of the
/// sentence's span).
fn weight(lines: &[String]) -> usize {
    lines.iter().map(|l| l.chars().count()).sum()
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
