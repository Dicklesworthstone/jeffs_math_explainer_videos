//! Narration and the cue log (plan §3.3, §7.4, §7.7).
//!
//! - [`Script`]: `script.tsv` (`<line id>\t<text>`), the single source of
//!   every spoken line, compiled into each video with `include_str!`.
//! - [`Narrator`]: the picture follows the voice. `say` waits for the line
//!   before it (plus a breath) and places `<id>.wav`; `hold` lets a line
//!   finish before a wipe; `mark`, `music_breath` and `listen` author what the
//!   score answers.
//! - Every sound a scene places goes through [`Narrator::sound`], which checks
//!   the file before franken_manim sees it (size, format, finite samples)
//!   and logs it exactly: integer frames and fps, offset and gain as exact
//!   f64 bits, the decoded format, a SHA-256 of the bytes. Never `{:.3}`
//!   seconds: one 60 fps frame printed that way is 0.017 s, which is 816
//!   samples, not 800.
//! - At the end of `construct`, [`Narrator::timeline`] gives the score
//!   engine its `ChapterTimeline` and [`Narrator::cue_log`] the PCM master
//!   its cue list.
//!
//! franken_manim's facade has no construct-only mode at the pinned revision
//! (`render` always builds its runtime with skipping off), so the timeline
//! pass is a tiny render (64×36 at the real fps): the clock, and so every
//! frame number, is the same at any resolution.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use explainer_score::timeline::{ChapterTimeline, LinePlacement, Mark, Window};
use explainer_score::units::{frame_to_sample, round_half_away};
use explainer_score::{SAMPLE_RATE, digest, wav};
use fmn::prelude::*;

/// Silence between consecutive lines (a paragraph pause), seconds.
pub const BREATH: f64 = 0.6;
/// Scratch-voice speaking rate (words per second) for drafts before TTS.
pub const SCRATCH_WPS: f64 = 2.6;
/// Narration level in the mix under the new score engine (plan D-21): 0 dB,
/// so V, the voice's loudness as placed, is the files' own −23 LUFS.
pub const GAIN_DB: f64 = 0.0;

// ----------------------------------------------------------------- script

/// The spoken lines of a video, in order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Script {
    lines: Vec<(String, String)>,
}

impl Script {
    /// Parse `script.tsv`: one `<id>\t<text>` per line; `#` lines and blank
    /// lines are skipped. A missing tab, a second tab, an empty id or text,
    /// or a repeated id is an error naming the line.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut lines = Vec::new();
        let mut seen = HashSet::new();
        for (i, raw) in text.lines().enumerate() {
            let n = i + 1;
            let line = raw.trim_end_matches('\r');
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((id, rest)) = line.split_once('\t') else {
                return Err(format!("script.tsv line {n}: no tab between id and text"));
            };
            if rest.contains('\t') {
                return Err(format!("script.tsv line {n}: a stray tab inside the text"));
            }
            let (id, rest) = (id.trim(), rest.trim());
            if id.is_empty() || rest.is_empty() {
                return Err(format!("script.tsv line {n}: empty id or text"));
            }
            if !seen.insert(id.to_owned()) {
                return Err(format!("script.tsv line {n}: id {id:?} repeats"));
            }
            lines.push((id.to_owned(), rest.to_owned()));
        }
        Ok(Self { lines })
    }

    pub fn lines(&self) -> &[(String, String)] {
        &self.lines
    }

    pub fn text(&self, id: &str) -> Option<&str> {
        self.lines.iter().find(|l| l.0 == id).map(|l| l.1.as_str())
    }

    /// The requested lines in script order, for a partial chapter render.
    pub fn select(&self, keep: impl Fn(&str) -> bool) -> Self {
        Self {
            lines: self.lines.iter().filter(|(id, _)| keep(id)).cloned().collect(),
        }
    }
}

// ----------------------------------------------------------------- cues

/// A sound file as checked before franken_manim loads it.
#[derive(Clone, Debug, PartialEq)]
pub struct CueFile {
    pub path: PathBuf,
    pub sha256: String,
    pub bytes: u64,
    pub rate: u32,
    pub channels: u16,
    pub frames: usize,
}

impl CueFile {
    pub fn seconds(&self) -> f64 {
        self.frames as f64 / f64::from(self.rate)
    }
}

/// Check a sound file the way a render would need it: at most
/// franken_manim's 64 MiB cue limit, a WAV the decoder takes (PCM
/// u8/s16/s24/s32 or float, mono or stereo, whole frames), finite samples.
/// fmn itself reports bad cues only after the whole render; this refuses
/// them when they are placed.
pub fn inspect_cue(path: &Path) -> io::Result<CueFile> {
    let ctx = |e: io::Error| io::Error::new(e.kind(), format!("{}: {e}", path.display()));
    // the size first, from the file system: a mistaken multi-gigabyte file is
    // refused without being read
    let len = std::fs::metadata(path).map_err(ctx)?.len();
    if len > wav::MAX_CUE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "{}: {len} bytes, over franken_manim's {} byte cue limit",
                path.display(),
                wav::MAX_CUE_BYTES
            ),
        ));
    }
    let bytes = std::fs::read(path).map_err(ctx)?;
    let d = wav::parse(&bytes).map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", path.display())))?;
    Ok(CueFile {
        path: path.to_path_buf(),
        sha256: digest::sha256_hex(&bytes),
        bytes: bytes.len() as u64,
        rate: d.rate,
        channels: d.channels,
        frames: d.frames(),
    })
}

/// One sound a scene placed, exactly as franken_manim placed it.
#[derive(Clone, Debug, PartialEq)]
pub struct Cue {
    pub file: CueFile,
    /// The scene clock at the call, in whole frames.
    pub frame: u64,
    pub fps: u32,
    /// `add_sound`'s time offset, seconds.
    pub offset: f64,
    pub gain_db: Option<f64>,
}

impl Cue {
    /// The cue's first sample at 48 kHz, as fmn's mixer computes it:
    /// `round(frame · 48000 / fps) + round_half_away(offset · 48000)`. Below
    /// zero the head is clipped, not shifted.
    pub fn start_sample(&self) -> i64 {
        frame_to_sample(self.frame, self.fps) as i64 + round_half_away(self.offset * f64::from(SAMPLE_RATE))
    }
}

// ----------------------------------------------------------------- narrator

#[derive(Default)]
struct Log {
    lines: Vec<LinePlacement>,
    marks: Vec<Mark>,
    breaths: Vec<Window>,
    listens: Vec<Window>,
    cues: Vec<Cue>,
}

/// The narrator of one video: line lengths, when the current line ends, and
/// the log of the chapter being rendered. Shared by clones of the `Kit`.
pub struct Narrator {
    /// `None`: scratch voice (estimated lengths, no audio placed).
    dir: Option<PathBuf>,
    files: HashMap<String, CueFile>,
    lengths: HashMap<String, f64>,
    /// Where each sentence of a line starts in it (seconds from the line's
    /// start): from the narration's `report.json`, or estimated.
    starts: HashMap<String, Vec<f64>>,
    /// When each line said in this chapter started (scene seconds).
    said_at: RefCell<HashMap<String, f64>>,
    gain_db: f64,
    busy_until: Cell<f64>,
    log: RefCell<Log>,
}

fn clock(stage: &Stage<'_>) -> (u64, u32, f64) {
    let t = stage.scene().time();
    (t.frames().max(0) as u64, t.fps(), t.to_f64())
}

/// Whole frames needed to reach `seconds` (a hair of float noise above a
/// frame boundary does not cost a frame).
pub fn frames_covering(seconds: f64, fps: u32) -> u64 {
    (seconds * f64::from(fps) - 1e-6).ceil().max(0.0) as u64
}

/// Advance the scene clock by exactly `k` frames. fmn rounds a wait up to
/// whole frames on the exact value of the f64 (its BN-02), so `k / fps` can
/// cost k + 1 frames when the f64 lands a hair above the boundary; half a
/// frame less always lands on exactly k.
pub fn wait_frames(stage: &mut Stage<'_>, k: u64) -> fmn::Result<()> {
    if k > 0 {
        let fps = f64::from(stage.scene().time().fps());
        stage.wait((k as f64 - 0.5) / fps)?;
    }
    Ok(())
}

/// The sentences of a line's text: split after `.`, `?` or `!` followed by
/// a space (or the end).
fn sentences(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut from = 0;
    let b = text.as_bytes();
    for (i, &c) in b.iter().enumerate() {
        if matches!(c, b'.' | b'?' | b'!') && (i + 1 == b.len() || b[i + 1] == b' ') {
            let s = text[from..=i].trim();
            if !s.is_empty() {
                out.push(s);
            }
            from = i + 1;
        }
    }
    let rest = text[from..].trim();
    if !rest.is_empty() {
        out.push(rest);
    }
    out
}

/// A line's sentence onsets when only its length is known: each sentence
/// starts at its share of `len` by characters before it.
fn estimated_starts(text: &str, len: f64) -> Vec<f64> {
    let parts = sentences(text);
    let total: usize = parts.iter().map(|s| s.chars().count()).sum::<usize>().max(1);
    let mut before = 0usize;
    parts
        .iter()
        .map(|s| {
            let at = len * before as f64 / total as f64;
            before += s.chars().count();
            at
        })
        .collect()
}

/// Every line's sentence onsets and total length (seconds) from the
/// narration skill's `report.json` (`meta.rate`; per sentence
/// `out_samples` and `gap_after_samples`, the take assembled in order).
pub fn report_starts(json: &str) -> io::Result<HashMap<String, (Vec<f64>, f64)>> {
    let bad = |m: String| io::Error::new(io::ErrorKind::InvalidData, m);
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| bad(format!("report.json: {e}")))?;
    let rate = v
        .pointer("/meta/rate")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| bad("report.json: no meta.rate".into()))?;
    if !rate.is_finite() || rate <= 0.0 {
        return Err(bad("report.json: meta.rate must be positive and finite".into()));
    }
    let lines = v
        .get("lines")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| bad("report.json: no lines".into()))?;
    let mut out = HashMap::new();
    for (id, list) in lines {
        let Some(list) = list.as_array() else {
            return Err(bad(format!("report.json: line {id} is not a list")));
        };
        let mut t = 0usize;
        let mut starts = Vec::with_capacity(list.len());
        for (k, s) in list.iter().enumerate() {
            let n = |name: &str| {
                s.get(name)
                    .and_then(serde_json::Value::as_u64)
                    .ok_or_else(|| bad(format!("report.json: {id} sentence {k}: no {name}")))
            };
            starts.push(t as f64 / rate);
            t += (n("out_samples")? + n("gap_after_samples")?) as usize;
        }
        out.insert(id.clone(), (starts, t as f64 / rate));
    }
    Ok(out)
}

/// Bind the report's display text to the script, not its pronunciation
/// respellings (`tts`). Existing WAV names alone do not identify a read:
/// rewording a line keeps its id, so it must require new narration.
fn check_report_script(json: &str, script: &Script) -> io::Result<()> {
    let bad = |m: String| io::Error::new(io::ErrorKind::InvalidData, m);
    let report: serde_json::Value = serde_json::from_str(json).map_err(|e| bad(format!("report.json: {e}")))?;
    let lines = report
        .get("lines")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| bad("report.json: no lines".into()))?;
    // Director reads may change sentence punctuation, including quotes.
    // Small integers may be written as words or digits in a selected read.
    // Keep other words, internal apostrophes/decimal points, and number signs
    // exact: "don't" must not become "do", nor -1 become 1.
    let normalize = |text: &str| {
        // A director may write "ticket-holder" for "ticket holder".
        // Split only a dash between letters, never a numeric minus sign.
        let mut words = String::new();
        let mut chars = text.chars().peekable();
        let mut previous: Option<char> = None;
        while let Some(c) = chars.next() {
            let between_letters = matches!(c, '-' | '‐' | '‑' | '–' | '—')
                && previous.is_some_and(char::is_alphabetic)
                && chars.peek().is_some_and(|next| next.is_alphabetic());
            words.push(if between_letters { ' ' } else { c });
            previous = Some(c);
        }
        words
            .split_whitespace()
            .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric() && !matches!(c, '-' | '+' | '−')))
            .filter(|word| !word.is_empty())
            // Exact standalone spellings only: no ASR homophones, compound
            // number parsing, sign removal, or decimal rounding. In particular,
            // "two", "02", "2.0" and "-2" are not all the same spoken text.
            .map(|word| match word.to_ascii_lowercase().as_str() {
                "zero" => "0".to_owned(),
                "one" => "1".to_owned(),
                "two" => "2".to_owned(),
                "three" => "3".to_owned(),
                "four" => "4".to_owned(),
                "five" => "5".to_owned(),
                "six" => "6".to_owned(),
                "seven" => "7".to_owned(),
                "eight" => "8".to_owned(),
                "nine" => "9".to_owned(),
                "ten" => "10".to_owned(),
                "eleven" => "11".to_owned(),
                "twelve" => "12".to_owned(),
                "thirteen" => "13".to_owned(),
                "fourteen" => "14".to_owned(),
                "fifteen" => "15".to_owned(),
                "sixteen" => "16".to_owned(),
                "seventeen" => "17".to_owned(),
                "eighteen" => "18".to_owned(),
                "nineteen" => "19".to_owned(),
                _ => word.to_owned(),
            })
            .collect::<Vec<_>>()
            .join(" ")
    };
    for (id, text) in script.lines() {
        let sentences = lines
            .get(id)
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| bad(format!("report.json: no sentence list for scripted line {id}")))?;
        let spoken = sentences
            .iter()
            .enumerate()
            .map(|(k, s)| {
                s.get("sentence")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| bad(format!("report.json: {id} sentence {k}: no display text")))
            })
            .collect::<io::Result<Vec<_>>>()?
            .join(" ");
        if normalize(text) != normalize(&spoken) {
            return Err(bad(format!(
                "report.json: narration text for {id} differs from script.tsv; reassemble this line from the current script before rendering"
            )));
        }
    }
    Ok(())
}

impl Narrator {
    /// Real narration from `dir/<id>.wav` for every line of `script`, placed
    /// at `gain_db` ([`GAIN_DB`] under the new score). Every file is checked
    /// now: a missing or unusable line is an error before anything renders.
    /// The report must describe this script and account for each WAV's
    /// length, so a reworded line cannot silently reuse an old read.
    pub fn load(script: &Script, dir: &Path, gain_db: f64) -> io::Result<Rc<Self>> {
        let mut files = HashMap::new();
        let mut lengths = HashMap::new();
        for (id, _) in script.lines() {
            let f = inspect_cue(&dir.join(format!("{id}.wav")))?;
            lengths.insert(id.clone(), f.seconds());
            files.insert(id.clone(), f);
        }
        let report_path = dir.join("report.json");
        let report = std::fs::read_to_string(&report_path)
            .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", report_path.display())))?;
        check_report_script(&report, script)?;
        let report = report_starts(&report)?;
        let mut starts = HashMap::new();
        for (id, _) in script.lines() {
            let len = lengths.get(id).copied().unwrap_or(0.0);
            let (onsets, total) = report.get(id).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, format!("report.json: no timing for {id}"))
            })?;
            if (total - len).abs() >= 0.05 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "report.json: {id} accounts for {total:.3} s, but its WAV is {len:.3} s; reassemble narration before rendering"
                    ),
                ));
            }
            starts.insert(id.clone(), onsets.clone());
        }
        Ok(Rc::new(Self {
            dir: Some(dir.to_path_buf()),
            files,
            lengths,
            starts,
            said_at: RefCell::new(HashMap::new()),
            gain_db,
            busy_until: Cell::new(0.0),
            log: RefCell::new(Log::default()),
        }))
    }

    /// A scratch voice for drafts: lengths from word counts, no audio.
    pub fn estimate(script: &Script) -> Rc<Self> {
        let speech = |text: &str| text.split_whitespace().count() as f64 / SCRATCH_WPS;
        let lengths = script
            .lines()
            .iter()
            .map(|(id, text)| (id.clone(), speech(text) + 0.4))
            .collect();
        let starts = script
            .lines()
            .iter()
            .map(|(id, text)| (id.clone(), estimated_starts(text, speech(text))))
            .collect();
        Rc::new(Self {
            dir: None,
            files: HashMap::new(),
            lengths,
            starts,
            said_at: RefCell::new(HashMap::new()),
            gain_db: GAIN_DB,
            busy_until: Cell::new(0.0),
            log: RefCell::new(Log::default()),
        })
    }

    /// Every chapter is its own scene whose clock starts at zero.
    pub fn reset(&self) {
        self.busy_until.set(0.0);
        *self.log.borrow_mut() = Log::default();
        self.said_at.borrow_mut().clear();
    }

    /// Where each sentence of line `id` starts, seconds from the line's
    /// start (the first is 0): the narration report's exact sample counts,
    /// or, for the scratch voice, shares of the line by sentence length.
    pub fn sentence_starts(&self, id: &str) -> Vec<f64> {
        self.starts.get(id).cloned().unwrap_or_default()
    }

    /// The scene time sentence `j` of line `id` starts at, once the line has
    /// been said in this chapter.
    pub fn sentence_at(&self, id: &str, j: usize) -> Option<f64> {
        let at = self.said_at.borrow().get(id).copied()?;
        self.starts.get(id)?.get(j).map(|s| at + s)
    }

    /// Whether this narrator places real audio (false for the scratch voice).
    pub fn has_audio(&self) -> bool {
        self.dir.is_some()
    }

    /// The length of line `id`, seconds.
    pub fn length(&self, id: &str) -> Option<f64> {
        self.lengths.get(id).copied()
    }

    /// Wait until the current line, plus a breath, has finished.
    pub fn hold(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let (frame, fps, _) = clock(stage);
        let target = frames_covering(self.busy_until.get() + BREATH, fps);
        wait_frames(stage, target.saturating_sub(frame))
    }

    /// Speak line `id` now, once the previous line has finished. Call it
    /// before the animations that illustrate the line.
    ///
    /// # Errors
    /// If `id` is not in the script (a typo must fail the render, never drop
    /// a line silently), or was already said in this chapter (line ids are
    /// the join keys of the timeline, captions and QA: give a repeat its own
    /// id in script.tsv).
    pub fn say(&self, stage: &mut Stage<'_>, id: &str) -> fmn::Result<()> {
        let fail = |text: String| fmn::Error::Scene(SceneError::InvalidConfig(Box::leak(text.into_boxed_str())));
        let len = self
            .length(id)
            .ok_or_else(|| fail(format!("narration line {id:?} is not in script.tsv")))?;
        if self.said_at.borrow().contains_key(id) {
            return Err(fail(format!(
                "narration line {id:?} is said twice in one chapter; give the repeat its own id in script.tsv"
            )));
        }
        self.hold(stage)?;
        let (frame, _, now) = clock(stage);
        self.said_at.borrow_mut().insert(id.to_owned(), now);
        if let (Some(dir), Some(file)) = (&self.dir, self.files.get(id)) {
            let path = dir.join(format!("{id}.wav"));
            self.place(stage, file.clone(), 0.0, Some(self.gain_db))?;
            self.log.borrow_mut().lines.push(LinePlacement {
                id: id.to_owned(),
                start_frame: frame,
                gain_db: self.gain_db,
                wav: path,
            });
        }
        self.busy_until.set(now + len);
        Ok(())
    }

    fn place(&self, stage: &mut Stage<'_>, file: CueFile, offset: f64, gain_db: Option<f64>) -> fmn::Result<()> {
        let (frame, fps, _) = clock(stage);
        stage.scene_mut().add_sound(file.path.clone(), offset, gain_db, None)?;
        self.log.borrow_mut().cues.push(Cue {
            file,
            frame,
            fps,
            offset,
            gain_db,
        });
        Ok(())
    }

    /// Place any other sound (a score stem, a sonification) through the same
    /// check and log. A stem laid under the whole scene goes in at the end of
    /// `construct` with `offset = -scene seconds`, taken from the scene clock.
    pub fn sound(&self, stage: &mut Stage<'_>, path: &Path, offset: f64, gain_db: Option<f64>) -> io::Result<()> {
        let file = inspect_cue(path)?;
        self.place(stage, file, offset, gain_db)
            .map_err(|e| io::Error::other(e.to_string()))
    }

    /// An authored moment the score may answer (`kind:name`, see the score's
    /// cue kinds; a bare name is a reveal).
    pub fn mark(&self, stage: &Stage<'_>, label: &str) {
        let (frame, _, _) = clock(stage);
        self.log.borrow_mut().marks.push(Mark {
            label: label.to_owned(),
            frame,
        });
    }

    /// Let the current line finish, then give the music `seconds` of its
    /// own: the duck releases there (plan §7.2's authored gaps).
    pub fn music_breath(&self, stage: &mut Stage<'_>, seconds: f64) -> fmn::Result<()> {
        self.window(stage, seconds, None)
    }

    /// A listen window: `seconds` with no narration for a sonification that
    /// teaches, played at full level (the duck releases).
    pub fn listen(&self, stage: &mut Stage<'_>, seconds: f64, label: &str) -> fmn::Result<()> {
        self.window(stage, seconds, Some(label))
    }

    /// A listen window that carries its sonification, `wav`: it opens once the
    /// current line has finished and lasts `seconds`, but does not wait, so
    /// the scene animates through it (and must fill it: the next line cannot
    /// start before the window ends). The score plays `wav` there at the
    /// identity's listen level; this call does not place it.
    pub fn sonify(&self, stage: &mut Stage<'_>, seconds: f64, label: &str, wav: &Path) -> fmn::Result<()> {
        inspect_cue(wav).map_err(|e| {
            fmn::Error::Scene(SceneError::InvalidConfig(Box::leak(
                format!("sonification {label}: {e}").into_boxed_str(),
            )))
        })?;
        self.hold(stage)?;
        let (frame, _, now) = clock(stage);
        self.log.borrow_mut().listens.push(Window {
            frame,
            seconds,
            label: label.to_owned(),
            sound: Some(wav.to_owned()),
        });
        self.busy_until.set(now + seconds - BREATH);
        Ok(())
    }

    fn window(&self, stage: &mut Stage<'_>, seconds: f64, label: Option<&str>) -> fmn::Result<()> {
        self.hold(stage)?;
        let (frame, _, now) = clock(stage);
        let w = Window {
            frame,
            seconds,
            label: label.unwrap_or_default().to_owned(),
            sound: None,
        };
        if label.is_some() {
            self.log.borrow_mut().listens.push(w);
        } else {
            self.log.borrow_mut().breaths.push(w);
        }
        wait_frames(stage, frames_covering(seconds, clock(stage).1))?;
        // the next line may start right after the window, not a breath later
        self.busy_until.set(now + seconds - BREATH);
        Ok(())
    }

    /// The chapter's timeline for the score engine, as of now (call at the end
    /// of `construct`: `frames` is the scene's length so far).
    pub fn timeline(&self, stage: &Stage<'_>, chapter: &str) -> ChapterTimeline {
        let (frames, fps, _) = clock(stage);
        let log = self.log.borrow();
        ChapterTimeline {
            chapter: chapter.to_owned(),
            fps,
            frames,
            lines: log.lines.clone(),
            marks: log.marks.clone(),
            breaths: log.breaths.clone(),
            listens: log.listens.clone(),
        }
    }

    /// Every cue placed so far.
    pub fn cues(&self) -> Vec<Cue> {
        self.log.borrow().cues.clone()
    }

    /// The cue log as TSV, one cue per line in placement order: path,
    /// SHA-256, bytes, frame, fps, offset and gain (readable and as exact f64
    /// bits), decoded rate, channels and frames, and the start sample fmn's
    /// mixer will use.
    pub fn cue_log(&self) -> String {
        let mut s = String::from(
            "# path\tsha256\tbytes\tframe\tfps\toffset\toffset_bits\tgain_db\tgain_bits\trate\tchannels\tframes\tstart_sample\n",
        );
        for c in &self.log.borrow().cues {
            let (gain, gain_bits) = match c.gain_db {
                Some(g) => (format!("{g:?}"), format!("{:#018x}", g.to_bits())),
                None => ("-".into(), "-".into()),
            };
            let _ = writeln!(
                s,
                "{}\t{}\t{}\t{}\t{}\t{:?}\t{:#018x}\t{gain}\t{gain_bits}\t{}\t{}\t{}\t{}",
                c.file.path.display(),
                c.file.sha256,
                c.file.bytes,
                c.frame,
                c.fps,
                c.offset,
                c.offset.to_bits(),
                c.file.rate,
                c.file.channels,
                c.file.frames,
                c.start_sample()
            );
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_report_binds_every_displayed_line_and_ignores_tts_respellings() {
        let script = Script::parse("01_1\tLeslie Lamport.\n01_2\tOne. Two.\n").unwrap();
        let report = r#"{"lines":{"01_1":[{"sentence":"Leslie  Lamport!","tts":"Leslie Lam-port."}],
            "01_2":[{"sentence":"One,"},{"sentence":"Two."}]}}"#;
        check_report_script(report, &script).unwrap();
        let hyphenated = Script::parse("01_1\tA ticket holder has thirty-one.\n").unwrap();
        let report_hyphen = r#"{"lines":{"01_1":[{"sentence":"A ticket-holder has thirty one!"}]}}"#;
        check_report_script(report_hyphen, &hyphenated).unwrap();
        let changed = Script::parse("01_1\tLeslie Lamport.\n01_2\tOne. Three.\n").unwrap();
        assert!(
            check_report_script(report, &changed)
                .unwrap_err()
                .to_string()
                .contains("01_2")
        );
        for report in [
            r#"{"lines":{"01_1":[{"sentence":"Leslie Lamport."}]}}"#,
            r#"{"lines":{"01_1":[{"sentence":"Leslie Lamport."}],"01_2":[]}}"#,
            r#"{"lines":{"01_1":[{"sentence":"Leslie Lamport."}],"01_2":[{"tts":"One. Two."}]}}"#,
        ] {
            assert!(
                check_report_script(report, &script)
                    .unwrap_err()
                    .to_string()
                    .contains("01_2")
            );
        }
        assert!(check_report_script("not JSON", &script).is_err());
        for (expected, heard) in [("-1.", "1."), ("Don't.", "Do."), ("2.5.", "25.")] {
            let script = Script::parse(&format!("01_1\t{expected}\n")).unwrap();
            let report = serde_json::json!({"lines": {"01_1": [{"sentence": heard}]}}).to_string();
            assert!(check_report_script(&report, &script).is_err(), "{expected} vs {heard}");
        }
    }

    #[test]
    fn number_spelling_does_not_require_a_different_take_or_hide_a_changed_claim() {
        // Video C 08_1's selected director read uses "2"; its ASR transcript
        // and the current script use "two". Keep that read and its metadata.
        let script = Script::parse(
            "08_1\tWhy can two customers never be in the room at once? Picture Blue inside, and Green holding a ticket.\n",
        )
        .unwrap();
        let report = r#"{"lines":{"08_1":[
            {"sentence":"Why can 2 customers never be in the room at once?","tts":"Why can 2 customers never be in the room at once?"},
            {"sentence":"Picture Blue inside, and Green holding a ticket."}]}}"#;
        check_report_script(report, &script).unwrap();
        for (expected, selected) in [("Zero.", "0."), ("10.", "Ten."), ("Nineteen.", "19.")] {
            let script = Script::parse(&format!("01_1\t{expected}\n")).unwrap();
            let report = serde_json::json!({"lines": {"01_1": [{"sentence": selected}]}}).to_string();
            check_report_script(&report, &script).unwrap();
        }
        for (expected, selected) in [
            ("Two.", "3."),
            ("Two.", "-2."),
            ("Two.", "+2."),
            ("Two.", "02."),
            ("Two.", "2.0."),
            ("Two.", "too."),
            ("Four.", "for."),
            ("One hundred.", "100."),
            ("Two never enter.", "2 enter."),
        ] {
            let script = Script::parse(&format!("01_1\t{expected}\n")).unwrap();
            let report = serde_json::json!({"lines": {"01_1": [{"sentence": selected}]}}).to_string();
            assert!(
                check_report_script(&report, &script).is_err(),
                "{expected} vs {selected}"
            );
        }
    }

    #[test]
    fn matching_audio_file_keeps_the_assembled_sentence_onsets() {
        let dir = tmp("sentence_onsets");
        std::fs::create_dir_all(&dir).unwrap();
        wav::write(
            &dir.join("01_1.wav"),
            &explainer_score::Stereo::silence(36_000),
            wav::Format::F32,
        )
        .unwrap();
        let report = r#"{"meta":{"rate":48000},"lines":{"01_1":[
            {"sentence":"One.","tts":"Wun.","out_samples":12000,"gap_after_samples":6000},
            {"sentence":"Two.","out_samples":18000,"gap_after_samples":0}]}}"#;
        std::fs::write(dir.join("report.json"), report).unwrap();
        let script = Script::parse("01_1\tOne. Two.\n").unwrap();
        let n = Narrator::load(&script, &dir, 0.0).unwrap();
        assert_eq!(n.starts["01_1"], [0.0, 0.375]);
        assert_eq!(n.lengths["01_1"], 0.75);
        for rate in [0, -1] {
            assert!(report_starts(&format!(r#"{{"meta":{{"rate":{rate}}},"lines":{{}}}}"#)).is_err());
        }
    }

    #[test]
    fn sentence_onsets_come_from_the_report_or_from_shares_of_the_line() {
        // Video A's 05_4 as the narration report records it (24 kHz)
        let json = r#"{"meta": {"rate": 24000}, "lines": {"05_4": [
            {"out_samples": 125520, "gap_after_samples": 9120},
            {"out_samples": 56401, "gap_after_samples": 9600},
            {"out_samples": 45121, "gap_after_samples": 9600},
            {"out_samples": 49442, "gap_after_samples": 0}]}}"#;
        let r = report_starts(json).unwrap();
        let (starts, total) = &r["05_4"];
        let want = [0.0, 5.61, 8.36, 10.64];
        assert!(
            starts.iter().zip(want).all(|(a, b)| (a - b).abs() < 0.005),
            "{starts:?}"
        );
        assert!((total - 12.7).abs() < 0.001, "{total}");
        // the estimate: sentences split at their ends, onsets by length
        let text = "One two. Three four five? Six!";
        assert_eq!(sentences(text), ["One two.", "Three four five?", "Six!"]);
        let est = estimated_starts(text, 6.0);
        assert_eq!(est.len(), 3);
        assert_eq!(est[0], 0.0);
        assert!(est[1] < est[2] && est[2] < 6.0);
        // a decimal point is not a sentence end
        assert_eq!(sentences("It is 2.5 fish. Yes.").len(), 2);
    }

    #[test]
    fn malformed_scripts_are_refused_with_their_line_number() {
        let ok = Script::parse("# c\n01_1\tHello there.\n\n01_2\tSecond line.\n").unwrap();
        assert_eq!(ok.lines().len(), 2);
        assert_eq!(ok.text("01_2"), Some("Second line."));
        for (bad, line) in [
            ("01_1\tok\n01_2 no tab\n", "line 2"),
            ("01_1\ttwo\ttabs\n", "line 1"),
            ("01_1\t   \n", "line 1"),
            ("01_1\tone\n01_1\tagain\n", "line 2"),
        ] {
            let e = Script::parse(bad).unwrap_err();
            assert!(e.contains(line), "{bad:?}: {e}");
        }
    }

    #[test]
    fn a_cue_at_frame_one_of_a_60_fps_scene_starts_at_sample_800() {
        let file = CueFile {
            path: "x.wav".into(),
            sha256: String::new(),
            bytes: 0,
            rate: 48_000,
            channels: 2,
            frames: 1,
        };
        let cue = |frame, offset| Cue {
            file: file.clone(),
            frame,
            fps: 60,
            offset,
            gain_db: None,
        };
        assert_eq!(cue(1, 0.0).start_sample(), 800);
        assert_eq!(cue(3, 0.0).start_sample(), 2_400);
        // a stem laid from t = 0 at the end of a 90-frame scene
        assert_eq!(cue(90, -1.5).start_sample(), 0);
        // offsets round half away from zero, as fmn's mixer does (2^-16 s is
        // exactly 0.732421875 samples)
        let tiny = 2f64.powi(-16);
        assert_eq!(cue(0, tiny).start_sample(), 1);
        assert_eq!(cue(1, -tiny).start_sample(), 799);
    }

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("explainer_kit_narr_{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        d.join(name)
    }

    #[test]
    fn cue_validation_refuses_what_a_render_would_choke_on() {
        // a good f32 stereo stem passes and is described exactly
        let good = tmp("good.wav");
        let s = explainer_score::Stereo::silence(4_800);
        wav::write(&good, &s, wav::Format::F32).unwrap();
        let f = inspect_cue(&good).unwrap();
        assert_eq!((f.rate, f.channels, f.frames), (48_000, 2, 4_800));
        assert_eq!(f.sha256, digest::sha256_hex(&std::fs::read(&good).unwrap()));
        // planted negatives: over 64 MiB; a NaN sample; a truncated header
        let big = tmp("big.wav");
        let mut bytes = std::fs::read(&good).unwrap();
        bytes.resize((wav::MAX_CUE_BYTES + 1) as usize, 0);
        std::fs::write(&big, &bytes).unwrap();
        assert!(inspect_cue(&big).unwrap_err().to_string().contains("cue limit"));
        let nan = tmp("nan.wav");
        let mut bytes = std::fs::read(&good).unwrap();
        let k = bytes.len() - 4;
        bytes[k..].copy_from_slice(&f32::NAN.to_le_bytes());
        std::fs::write(&nan, &bytes).unwrap();
        assert!(inspect_cue(&nan).is_err(), "a NaN sample");
        let short = tmp("short.wav");
        std::fs::write(&short, &std::fs::read(&good).unwrap()[..30]).unwrap();
        assert!(inspect_cue(&short).is_err(), "a truncated header");
        assert!(inspect_cue(&tmp("missing.wav")).is_err());
    }
}
