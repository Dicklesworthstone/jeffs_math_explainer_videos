//! A video's storyboard (`docs/storyboard.md`), the hand-edited source of its
//! script: per chapter (a `## NN_key…` heading), one table row per beat,
//! `| id | words | picture | features | hold/listen |`.
//!
//! `tools/storyboard.py` checks it (gate G2) and writes `script.tsv` from the
//! words column; [`Animatic`](crate::animatic::Animatic) reads it at render
//! time to stand in for a chapter until the real one is built. The words
//! column may carry `*(annotations)*` (never spoken) and `⟨predict⟩`, which
//! marks a prediction question: the row must end with that question, and the
//! picture holds after it.

/// The marker that opens a prediction question in the words column.
pub const PREDICT: &str = "⟨predict⟩";

/// One beat: a narration line (or a silent card) and what the screen shows.
#[derive(Clone, Debug, PartialEq)]
pub struct Beat {
    pub id: String,
    pub words: String,
    pub picture: String,
    pub features: String,
    pub hold: String,
}

impl Beat {
    /// The words as spoken: annotations and the predict marker removed,
    /// spaces collapsed; `None` for a silent beat (a title, a card).
    pub fn spoken(&self) -> Option<String> {
        let mut s = String::new();
        let mut rest = self.words.as_str();
        while let Some(i) = rest.find("*(") {
            s.push_str(&rest[..i]);
            match rest[i..].find(")*") {
                Some(j) => rest = &rest[i + j + 2..],
                None => {
                    rest = "";
                }
            }
        }
        s.push_str(rest);
        let s = s.replace(PREDICT, " ");
        let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
        (!s.is_empty()).then_some(s)
    }

    /// The beat asks the viewer to predict (the question ends the line).
    pub fn predict(&self) -> bool {
        self.words.contains(PREDICT)
    }

    /// Seconds of a listen window (`listen 3 s`), if the beat has one.
    pub fn listen_s(&self) -> Option<f64> {
        let i = self.hold.find("listen")?;
        let rest = self.hold[i + "listen".len()..].trim_start();
        let end = rest
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .unwrap_or(rest.len());
        rest[..end].parse().ok()
    }

    /// An authored music breath after the beat (`hold`).
    pub fn holds(&self) -> bool {
        self.hold.split(|c: char| !c.is_alphanumeric()).any(|w| w == "hold")
    }
}

/// One chapter of the storyboard.
#[derive(Clone, Debug, PartialEq)]
pub struct ChapterBoard {
    /// `NN_name`, the chapter's key.
    pub key: String,
    /// The heading after the key (may be empty).
    pub title: String,
    pub beats: Vec<Beat>,
}

/// A table row's cells: split on every `|` not escaped as `\|`, trimmed,
/// without the empty strings outside the first and last pipe.
fn split_cells(line: &str) -> Vec<&str> {
    let mut cells = Vec::new();
    let (mut start, mut prev) = (0, '\0');
    for (i, c) in line.char_indices() {
        if c == '|' && prev != '\\' {
            cells.push(line[start..i].trim());
            start = i + 1;
        }
        prev = c;
    }
    cells.push(line[start..].trim());
    // drop the text before the first pipe, and after the last one when the
    // row closes with a pipe (a row may leave the closing pipe out)
    if !cells.is_empty() {
        cells.remove(0);
    }
    if line.trim_end().ends_with('|') {
        cells.pop();
    }
    cells
}

fn is_chapter_heading(line: &str) -> Option<&str> {
    let h = line.strip_prefix("## ")?;
    let b = h.as_bytes();
    (b.len() > 3 && b[0].is_ascii_digit() && b[1].is_ascii_digit() && b[2] == b'_').then_some(h)
}

/// Parse a storyboard. Every row's id must start with its chapter's `NN_`;
/// ids are unique; a ⟨predict⟩ row must end with its question.
pub fn parse(md: &str) -> Result<Vec<ChapterBoard>, String> {
    let mut chapters: Vec<ChapterBoard> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut fence = false;
    for (n, line) in md.lines().enumerate() {
        if line.starts_with("```") {
            fence = !fence;
            continue;
        }
        if fence {
            continue;
        }
        if let Some(h) = is_chapter_heading(line) {
            let key_end = h
                .find(|c: char| c == ':' || c.is_whitespace() || c == '(')
                .unwrap_or(h.len());
            chapters.push(ChapterBoard {
                key: h[..key_end].to_owned(),
                title: h[key_end..].trim_start_matches([':', ' ']).trim().to_owned(),
                beats: Vec::new(),
            });
            continue;
        }
        let Some(row) = line.strip_prefix("| ") else { continue };
        let b = row.as_bytes();
        if !(b.len() > 3 && b[0].is_ascii_digit() && b[1].is_ascii_digit() && b[2] == b'_') {
            continue;
        }
        let Some(ch) = chapters.last_mut() else {
            return Err(format!("storyboard line {}: a beat before any chapter heading", n + 1));
        };
        let cells = split_cells(line);
        let cell = |i: usize| cells.get(i).copied().unwrap_or("").to_owned();
        let beat = Beat {
            id: cell(0),
            words: cell(1),
            picture: cell(2),
            features: cell(3),
            hold: cell(4),
        };
        if !beat.id.starts_with(&ch.key[..3]) {
            return Err(format!(
                "storyboard line {}: beat {} is under chapter {}",
                n + 1,
                beat.id,
                ch.key
            ));
        }
        if !seen.insert(beat.id.clone()) {
            return Err(format!("storyboard line {}: beat {} repeats", n + 1, beat.id));
        }
        if beat.predict() {
            let spoken = beat.spoken().unwrap_or_default();
            if !spoken.ends_with('?') {
                return Err(format!(
                    "storyboard line {}: beat {} has ⟨predict⟩ but does not end with its question \
                     (put the answer in the next beat, so the picture can hold after the question)",
                    n + 1,
                    beat.id
                ));
            }
        }
        ch.beats.push(beat);
    }
    if fence {
        return Err("storyboard: an unclosed ``` block".into());
    }
    Ok(chapters)
}

/// The script the storyboard speaks: `(id, words)` for every spoken beat.
pub fn script_lines(chapters: &[ChapterBoard]) -> Vec<(String, String)> {
    chapters
        .iter()
        .flat_map(|c| c.beats.iter())
        .filter_map(|b| b.spoken().map(|s| (b.id.clone(), s)))
        .collect()
}

/// The checks every video's tests run on its storyboard: it parses, its
/// chapters are the registry's (same keys, same order), and `script.tsv` is
/// exactly what the storyboard speaks (no line edited by hand, none stale).
pub fn check_video(storyboard: &str, script_tsv: &str, registry_keys: &[&str]) -> Result<(), String> {
    let chapters = parse(storyboard)?;
    let keys: Vec<&str> = chapters.iter().map(|c| c.key.as_str()).collect();
    if keys != registry_keys {
        return Err(format!("storyboard chapters {keys:?} != registry {registry_keys:?}"));
    }
    let script = crate::narration::Script::parse(script_tsv)?;
    let want = script_lines(&chapters);
    let have: Vec<(String, String)> = script.lines().to_vec();
    if have != want {
        let first = have
            .iter()
            .zip(&want)
            .position(|(a, b)| a != b)
            .unwrap_or(have.len().min(want.len()));
        return Err(format!(
            "script.tsv is not the storyboard's words (first difference at line {}: {:?} vs {:?}); \
             run python3 -I tools/storyboard.py",
            first + 1,
            have.get(first),
            want.get(first)
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const MD: &str = "\
# Storyboard — Toy

## Vocabulary (one word per idea)

marked, catch

## 01_hook: The fish nobody caught (~75 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | A thousand fish are caught. *(the seeded lake)* | Top-down lake | swarm | |
| 01_2 | *(title)* | **Title** | title card | hold |
| 01_3 | Try two other spots. ⟨predict⟩ Same answer? | Three estimates | | listen 2.5 s |

- notes are skipped

```rust
| 01_9 | not a row inside a fence |
```

## 02_the_marks

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Here. | A bar \\| 100 of 1,000 | prop | |
";

    #[test]
    fn a_storyboard_parses_into_chapters_beats_and_script_lines() {
        let ch = parse(MD).unwrap();
        assert_eq!(ch.len(), 2);
        assert_eq!(
            (ch[0].key.as_str(), ch[0].title.as_str()),
            ("01_hook", "The fish nobody caught (~75 s)")
        );
        assert_eq!(ch[1].key, "02_the_marks");
        assert_eq!(ch[0].beats.len(), 3, "the fenced row is not a beat");
        let b = &ch[0].beats;
        assert_eq!(b[0].spoken().as_deref(), Some("A thousand fish are caught."));
        assert_eq!(b[1].spoken(), None);
        assert!(b[1].holds() && !b[0].holds());
        assert!(b[2].predict());
        assert_eq!(b[2].spoken().as_deref(), Some("Try two other spots. Same answer?"));
        assert_eq!(b[2].listen_s(), Some(2.5));
        assert_eq!(ch[1].beats[0].picture, "A bar \\| 100 of 1,000");
        let s = script_lines(&ch);
        assert_eq!(s.len(), 3);
        assert_eq!(s[2], ("02_1".to_owned(), "Here.".to_owned()));
    }

    #[test]
    fn a_predict_beat_must_end_on_its_question_and_ids_must_fit_their_chapter() {
        let bad = MD.replace("⟨predict⟩ Same answer?", "⟨predict⟩ Same answer? Not even close.");
        assert!(parse(&bad).unwrap_err().contains("does not end with its question"));
        let misplaced = MD.replace("| 02_1 |", "| 03_1 |");
        assert!(parse(&misplaced).unwrap_err().contains("is under chapter 02_the_marks"));
        // an annotation may hold parentheses; a row may leave out its closing pipe
        let paren = "## 01_x\n| 01_1 | Hi *(odds (tr. 1902), verbatim)* there. | p | f | listen 3 s\n";
        let b = &parse(paren).unwrap()[0].beats[0];
        assert_eq!(b.spoken().as_deref(), Some("Hi there."));
        assert_eq!(b.listen_s(), Some(3.0));
        let dup = MD.replace("| 01_3 |", "| 01_1 |");
        assert!(parse(&dup).unwrap_err().contains("repeats"));
        assert!(
            parse("| 01_1 | a | b | c | |")
                .unwrap_err()
                .contains("before any chapter")
        );
        assert!(parse("## 01_x\n```\n").unwrap_err().contains("unclosed"));
    }
}
